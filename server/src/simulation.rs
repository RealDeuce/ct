//! Durable scheduled simulation, ordinary traffic, and physical mail carriage.
//!
//! This is deliberately a small first calibration model.  It persists the
//! things whose identity matters -- messages, delivery envelopes, traffic
//! ships, mailbags, custody legs, deliveries, and the next scheduled event --
//! while deriving each system day's draws from the system seed and day.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use heed::types::Bytes;
use heed::{Database, Env, RwTxn};
use thiserror::Error;

use crate::crypto::{CryptoError, SeedStream, derive_seed};

pub const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
pub const STANDARD_JUMP_SECONDS: u64 = 7 * SECONDS_PER_DAY;
const MAILBAG_ENVELOPE_LIMIT: usize = 512;
const JUMP_TWO_DISTANCE_SQUARED: f64 = 4.0 + 1e-9;

const META_NEXT_EVENT: &[u8] = b"Znext-event";
const META_NEXT_MESSAGE: &[u8] = b"Znext-message";
const META_NEXT_ENVELOPE: &[u8] = b"Znext-envelope";
const META_NEXT_TRAFFIC_SHIP: &[u8] = b"Znext-traffic-ship";
const META_NEXT_MAILBAG: &[u8] = b"Znext-mailbag";
const META_NEXT_CARRIER_LEG: &[u8] = b"Znext-carrier-leg";
const META_NEXT_DELIVERY: &[u8] = b"Znext-delivery";

const RECORD_SYSTEM: u8 = b'S';
const RECORD_MESSAGE: u8 = b'M';
const RECORD_ENVELOPE: u8 = b'E';
const RECORD_TRAFFIC_SHIP: u8 = b'V';
const RECORD_MAILBAG: u8 = b'B';
const RECORD_CARRIER_LEG: u8 = b'L';
const RECORD_DELIVERY: u8 = b'D';
const RECORD_PLAYER_CARRIER_LEG: u8 = b'P';
const RECORD_QUEUE: u8 = b'Q';

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("LMDB error: {0}")]
    Heed(#[from] heed::Error),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error("corrupt simulation record: {0}")]
    Corrupt(&'static str),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationSystem {
    pub system_id: u64,
    pub name: String,
    pub position_parsecs: [f64; 3],
    pub polity_id: u64,
    pub generation_seed: [u8; 32],
    pub population: u8,
    pub tech_level: u8,
    pub starport: u8,
    pub next_system_day: u64,
    pub jump_two_neighbors: Vec<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MessageClass {
    AgencyNews = 0,
    PublicService = 1,
    ContractOffer = 2,
    TrafficNotice = 3,
    Private = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum MessageImportance {
    Routine = 0,
    Notable = 1,
    Important = 2,
    Headline = 3,
}

impl MessageImportance {
    pub const ALL: [Self; 4] = [
        Self::Routine,
        Self::Notable,
        Self::Important,
        Self::Headline,
    ];

    fn from_u8(value: u8) -> Result<Self, SimulationError> {
        Self::ALL
            .get(value as usize)
            .copied()
            .ok_or(SimulationError::Corrupt("unknown message importance"))
    }
}

impl MessageClass {
    pub const ALL: [Self; 5] = [
        Self::AgencyNews,
        Self::PublicService,
        Self::ContractOffer,
        Self::TrafficNotice,
        Self::Private,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::AgencyNews => "agency-news",
            Self::PublicService => "public-service",
            Self::ContractOffer => "contract-offer",
            Self::TrafficNotice => "traffic-notice",
            Self::Private => "private",
        }
    }

    fn from_u8(value: u8) -> Result<Self, SimulationError> {
        Self::ALL
            .get(value as usize)
            .copied()
            .ok_or(SimulationError::Corrupt("unknown message class"))
    }

    fn lifetime_days(self) -> u64 {
        match self {
            Self::AgencyNews => 120,
            Self::PublicService => 180,
            Self::ContractOffer => 21,
            Self::TrafficNotice => 14,
            Self::Private => 60,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub message_id: u64,
    pub origin_system_id: u64,
    pub created_second: u64,
    pub expires_second: u64,
    pub class: MessageClass,
    pub importance: MessageImportance,
    pub subject: String,
    pub body: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EnvelopeStatus {
    Waiting = 0,
    InTransit = 1,
    Delivered = 2,
    Expired = 3,
}

impl EnvelopeStatus {
    fn from_u8(value: u8) -> Result<Self, SimulationError> {
        Ok(match value {
            0 => Self::Waiting,
            1 => Self::InTransit,
            2 => Self::Delivered,
            3 => Self::Expired,
            _ => return Err(SimulationError::Corrupt("unknown envelope status")),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryEnvelope {
    pub envelope_id: u64,
    pub message_id: u64,
    pub destination_system_id: u64,
    pub route: Vec<u64>,
    /// Index of the system at which the envelope is waiting, or the origin of
    /// its current carrier leg while it is in transit.
    pub route_index: u16,
    pub status: EnvelopeStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum TrafficShipStatus {
    InTransit = 0,
    Arrived = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulatedTrafficShip {
    pub traffic_ship_id: u64,
    pub origin_system_id: u64,
    pub destination_system_id: u64,
    pub departure_second: u64,
    pub arrival_second: u64,
    pub status: TrafficShipStatus,
    pub mailbag_id: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mailbag {
    pub mailbag_id: u64,
    pub origin_system_id: u64,
    pub destination_system_id: u64,
    pub sealed_second: u64,
    pub delivered_second: Option<u64>,
    pub envelope_ids: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarrierLeg {
    pub carrier_leg_id: u64,
    pub mailbag_id: u64,
    pub traffic_ship_id: u64,
    pub origin_system_id: u64,
    pub destination_system_id: u64,
    pub custody_second: u64,
    pub due_second: u64,
    pub delivered_second: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageDelivery {
    pub delivery_id: u64,
    pub envelope_id: Option<u64>,
    pub message_id: u64,
    pub system_id: u64,
    pub available_second: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AvailableMessage {
    pub message: Message,
    pub available_second: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerCarrierLeg {
    pub carrier_leg_id: u64,
    pub mailbag_id: u64,
    pub player_ship_id: u64,
    pub origin_system_id: u64,
    pub destination_system_id: u64,
    pub custody_second: u64,
    pub due_second: u64,
    pub delivered_second: Option<u64>,
    pub advertised_stipend_credits: u64,
    pub paid_second: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerMailPickup {
    pub mailbag_id: u64,
    pub carrier_leg_id: u64,
    pub envelope_count: u16,
    pub advertised_stipend_credits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerMailHandoff {
    pub mailbag_id: u64,
    pub delivered: u64,
    pub forwarded: u64,
    pub expired: u64,
    pub stipend_credits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum EventKind {
    SystemDay {
        system_id: u64,
        day: u64,
    },
    TrafficDeparture {
        origin_system_id: u64,
        destination_system_id: u64,
    },
    TrafficDeparturePlan {
        origin_system_id: u64,
        start_second: u64,
        ordinal: u16,
        total: u16,
        destinations: Vec<u64>,
    },
    TrafficArrival {
        traffic_ship_id: u64,
        mailbag_id: Option<u64>,
        carrier_leg_id: Option<u64>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScheduledEvent {
    event_id: u64,
    due_second: u64,
    entity_id: u64,
    kind: EventKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ScheduledEventHead {
    pub due_second: u64,
    pub event_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedSimulationEvent {
    pub event_id: u64,
    pub due_second: u64,
    entity_id: u64,
    encoded_kind: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedEvent {
    pub event_id: u64,
    pub due_second: u64,
    pub system_id: u64,
    pub kind: SimulationEventKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SimulationEventKind {
    SystemDay,
    TrafficDeparture,
    TrafficArrival,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SimulationReport {
    pub game_second: u64,
    pub system_days_processed: u64,
    pub scheduled_events: u64,
    pub messages_by_class: [u64; 5],
    pub envelopes_waiting: u64,
    pub envelopes_in_transit: u64,
    pub envelopes_delivered: u64,
    pub envelopes_expired: u64,
    pub local_deliveries: u64,
    pub traffic_ships: u64,
    pub traffic_ships_in_transit: u64,
    pub mailbags: u64,
    pub mailbags_delivered: u64,
    pub carrier_legs: u64,
    pub carrier_legs_delivered: u64,
}

impl SimulationReport {
    pub fn message_count(&self) -> u64 {
        self.messages_by_class.iter().sum()
    }

    pub fn remote_envelope_count(&self) -> u64 {
        self.envelopes_waiting
            + self.envelopes_in_transit
            + self.envelopes_delivered
            + self.envelopes_expired
    }
}

#[derive(Clone, Copy)]
pub struct SimulationDatabases {
    events: Database<Bytes, Bytes>,
    records: Database<Bytes, Bytes>,
}

impl SimulationDatabases {
    pub fn create(env: &Env, txn: &mut RwTxn<'_>) -> Result<Self, heed::Error> {
        Ok(Self {
            events: env.create_database(txn, Some("simulation-events"))?,
            records: env.create_database(txn, Some("simulation-records"))?,
        })
    }

    pub fn clear(&self, txn: &mut RwTxn<'_>) -> Result<(), heed::Error> {
        self.events.clear(txn)?;
        self.records.clear(txn)?;
        Ok(())
    }

    pub fn initialize(
        &self,
        txn: &mut RwTxn<'_>,
        mut systems: Vec<SimulationSystem>,
    ) -> Result<(), SimulationError> {
        self.clear(txn)?;
        systems.sort_by_key(|system| system.system_id);
        let neighbors = jump_two_neighbor_lists(&systems);
        for (system, system_neighbors) in systems.iter_mut().zip(neighbors) {
            system.jump_two_neighbors = system_neighbors;
            system.next_system_day = 0;
            self.records.put(
                txn,
                &record_key(RECORD_SYSTEM, system.system_id),
                &encode_system(system)?,
            )?;
        }
        for system in systems {
            self.schedule(
                txn,
                0,
                system.system_id,
                EventKind::SystemDay {
                    system_id: system.system_id,
                    day: 0,
                },
            )?;
        }
        Ok(())
    }

    pub fn add_systems(
        &self,
        txn: &mut RwTxn<'_>,
        mut added: Vec<SimulationSystem>,
        current_second: u64,
    ) -> Result<(), SimulationError> {
        if added.is_empty() {
            return Ok(());
        }
        let mut systems = self.systems(txn)?;
        let existing_ids = systems
            .iter()
            .map(|system| system.system_id)
            .collect::<BTreeSet<_>>();
        if added
            .iter()
            .any(|system| existing_ids.contains(&system.system_id))
        {
            return Err(SimulationError::Corrupt(
                "new simulation system already exists",
            ));
        }
        added.sort_by_key(|system| system.system_id);
        if added
            .windows(2)
            .any(|pair| pair[0].system_id == pair[1].system_id)
        {
            return Err(SimulationError::Corrupt("duplicate new simulation system"));
        }
        let current_day = current_second / SECONDS_PER_DAY;
        for system in &mut added {
            system.next_system_day = current_day;
        }
        let added_ids = added
            .iter()
            .map(|system| system.system_id)
            .collect::<Vec<_>>();
        systems.extend(added);
        systems.sort_by_key(|system| system.system_id);
        let neighbors = jump_two_neighbor_lists(&systems);
        for (system, system_neighbors) in systems.iter_mut().zip(neighbors) {
            system.jump_two_neighbors = system_neighbors;
            self.records.put(
                txn,
                &record_key(RECORD_SYSTEM, system.system_id),
                &encode_system(system)?,
            )?;
        }
        for system_id in added_ids {
            self.schedule(
                txn,
                current_second,
                system_id,
                EventKind::SystemDay {
                    system_id,
                    day: current_day,
                },
            )?;
        }
        Ok(())
    }

    pub fn scheduled_head(
        &self,
        txn: &RwTxn<'_>,
    ) -> Result<Option<ScheduledEventHead>, SimulationError> {
        let mut events = self.events.iter(txn)?;
        let Some(entry) = events.next() else {
            return Ok(None);
        };
        let (key, value) = entry?;
        let event = decode_event(key, value)?;
        Ok(Some(ScheduledEventHead {
            due_second: event.due_second,
            event_id: event.event_id,
        }))
    }

    pub fn take_scheduled(
        &self,
        txn: &mut RwTxn<'_>,
        expected: ScheduledEventHead,
    ) -> Result<QueuedSimulationEvent, SimulationError> {
        let selected = {
            let mut events = self.events.iter(txn)?;
            match events.next() {
                Some(entry) => {
                    let (key, value) = entry?;
                    Some((key.to_vec(), decode_event(key, value)?))
                }
                None => None,
            }
        };
        let Some((key, event)) = selected else {
            return Err(SimulationError::Corrupt("scheduled event disappeared"));
        };
        if (event.due_second, event.event_id) != (expected.due_second, expected.event_id) {
            return Err(SimulationError::Corrupt("scheduled event head changed"));
        }
        self.events.delete(txn, &key)?;
        Ok(QueuedSimulationEvent {
            event_id: event.event_id,
            due_second: event.due_second,
            entity_id: event.entity_id,
            encoded_kind: encode_event_kind(&event),
        })
    }

    pub fn process_queued(
        &self,
        txn: &mut RwTxn<'_>,
        queued: &QueuedSimulationEvent,
    ) -> Result<ProcessedEvent, SimulationError> {
        let event = decode_queued_event(queued)?;
        let (system_id, kind, summary) = match event.kind.clone() {
            EventKind::SystemDay { system_id, day } => (
                system_id,
                SimulationEventKind::SystemDay,
                self.process_system_day(txn, event.due_second, system_id, day)?,
            ),
            EventKind::TrafficDeparture {
                origin_system_id,
                destination_system_id,
            } => (
                origin_system_id,
                SimulationEventKind::TrafficDeparture,
                self.process_departure(
                    txn,
                    event.due_second,
                    origin_system_id,
                    destination_system_id,
                )?,
            ),
            EventKind::TrafficDeparturePlan {
                origin_system_id,
                start_second,
                ordinal,
                total,
                destinations,
            } => {
                let destination_system_id = *destinations
                    .first()
                    .ok_or(SimulationError::Corrupt("empty traffic departure plan"))?;
                if destinations.len() > 1 {
                    let next_ordinal = ordinal
                        .checked_add(1)
                        .ok_or(SimulationError::Corrupt("departure ordinal overflow"))?;
                    let next_event_id = event
                        .event_id
                        .checked_add(1)
                        .ok_or(SimulationError::Corrupt("event identifier overflow"))?;
                    let next_due = start_second.saturating_add(
                        u64::from(next_ordinal).saturating_mul(SECONDS_PER_DAY) / u64::from(total),
                    );
                    let next = ScheduledEvent {
                        event_id: next_event_id,
                        due_second: next_due,
                        entity_id: origin_system_id,
                        kind: EventKind::TrafficDeparturePlan {
                            origin_system_id,
                            start_second,
                            ordinal: next_ordinal,
                            total,
                            destinations: destinations[1..].to_vec(),
                        },
                    };
                    self.events.put(
                        txn,
                        &event_key(next.due_second, next.event_id),
                        &encode_event(&next),
                    )?;
                }
                (
                    origin_system_id,
                    SimulationEventKind::TrafficDeparture,
                    self.process_departure(
                        txn,
                        event.due_second,
                        origin_system_id,
                        destination_system_id,
                    )?,
                )
            }
            EventKind::TrafficArrival {
                traffic_ship_id,
                mailbag_id,
                carrier_leg_id,
            } => {
                let (system_id, summary) = self.process_arrival(
                    txn,
                    event.due_second,
                    traffic_ship_id,
                    mailbag_id,
                    carrier_leg_id,
                )?;
                (system_id, SimulationEventKind::TrafficArrival, summary)
            }
        };
        Ok(ProcessedEvent {
            event_id: event.event_id,
            due_second: event.due_second,
            system_id,
            kind,
            summary,
        })
    }

    pub fn take_scheduled_event_id(&self, txn: &mut RwTxn<'_>) -> Result<u64, SimulationError> {
        self.take_id(txn, META_NEXT_EVENT)
    }

    pub fn report(
        &self,
        txn: &heed::RoTxn<'_>,
        game_second: u64,
    ) -> Result<SimulationReport, SimulationError> {
        let mut scheduled_events = 0_u64;
        for entry in self.events.iter(txn)? {
            let (key, value) = entry?;
            let event = decode_event(key, value)?;
            scheduled_events = scheduled_events
                .checked_add(match event.kind {
                    EventKind::TrafficDeparturePlan { destinations, .. } => {
                        destinations.len() as u64
                    }
                    _ => 1,
                })
                .ok_or(SimulationError::Corrupt("scheduled event count overflow"))?;
        }
        let mut report = SimulationReport {
            game_second,
            scheduled_events,
            ..SimulationReport::default()
        };
        for entry in self.records.iter(txn)? {
            let (key, value) = entry?;
            match key.first().copied() {
                Some(RECORD_SYSTEM) => {
                    report.system_days_processed +=
                        decode_system(key_id(key)?, value)?.next_system_day;
                }
                Some(RECORD_MESSAGE) => {
                    let message = decode_message(key_id(key)?, value)?;
                    report.messages_by_class[message.class as usize] += 1;
                    // Availability at the immutable message's origin and
                    // creation time is inherent; no separate origin-delivery
                    // row is required.
                    report.local_deliveries += 1;
                }
                Some(RECORD_ENVELOPE) => match decode_envelope(key_id(key)?, value)?.status {
                    EnvelopeStatus::Waiting => report.envelopes_waiting += 1,
                    EnvelopeStatus::InTransit => report.envelopes_in_transit += 1,
                    EnvelopeStatus::Delivered => report.envelopes_delivered += 1,
                    EnvelopeStatus::Expired => report.envelopes_expired += 1,
                },
                Some(RECORD_TRAFFIC_SHIP) => {
                    report.traffic_ships += 1;
                    if decode_traffic_ship(key_id(key)?, value)?.status
                        == TrafficShipStatus::InTransit
                    {
                        report.traffic_ships_in_transit += 1;
                    }
                }
                Some(RECORD_MAILBAG) => {
                    report.mailbags += 1;
                    if decode_mailbag(key_id(key)?, value)?
                        .delivered_second
                        .is_some()
                    {
                        report.mailbags_delivered += 1;
                    }
                }
                Some(RECORD_CARRIER_LEG) => {
                    report.carrier_legs += 1;
                    if decode_carrier_leg(key_id(key)?, value)?
                        .delivered_second
                        .is_some()
                    {
                        report.carrier_legs_delivered += 1;
                    }
                }
                Some(RECORD_PLAYER_CARRIER_LEG) => {
                    report.carrier_legs += 1;
                    if decode_player_carrier_leg(key_id(key)?, value)?
                        .delivered_second
                        .is_some()
                    {
                        report.carrier_legs_delivered += 1;
                    }
                }
                Some(RECORD_DELIVERY)
                    if decode_delivery(key_id(key)?, value)?.envelope_id.is_some() =>
                {
                    report.local_deliveries += 1;
                }
                Some(RECORD_DELIVERY) => {}
                _ => {}
            }
        }
        Ok(report)
    }

    pub fn systems(&self, txn: &heed::RoTxn<'_>) -> Result<Vec<SimulationSystem>, SimulationError> {
        let mut systems = Vec::new();
        for entry in self.records.prefix_iter(txn, &[RECORD_SYSTEM])? {
            let (key, value) = entry?;
            systems.push(decode_system(key_id(key)?, value)?);
        }
        systems.sort_by_key(|system| system.system_id);
        Ok(systems)
    }

    pub fn recent_carrier_legs(
        &self,
        txn: &heed::RoTxn<'_>,
        limit: usize,
    ) -> Result<Vec<CarrierLeg>, SimulationError> {
        let mut legs = Vec::new();
        for entry in self.records.prefix_iter(txn, &[RECORD_CARRIER_LEG])? {
            let (key, value) = entry?;
            legs.push(decode_carrier_leg(key_id(key)?, value)?);
        }
        legs.sort_by_key(|leg| (leg.custody_second, leg.carrier_leg_id));
        if legs.len() > limit {
            legs.drain(..legs.len() - limit);
        }
        Ok(legs)
    }

    pub fn player_carrier_leg(
        &self,
        txn: &heed::RoTxn<'_>,
        carrier_leg_id: u64,
    ) -> Result<PlayerCarrierLeg, SimulationError> {
        self.get_player_carrier_leg(txn, carrier_leg_id)
    }

    pub fn mailbag(&self, txn: &heed::RoTxn<'_>, id: u64) -> Result<Mailbag, SimulationError> {
        self.get_mailbag(txn, id)
    }

    /// Seal the messages currently queued for this exact next hop and place
    /// them in a specific player ship's custody. An empty queue means no
    /// custody record and no stipend.
    pub fn pickup_player_mail(
        &self,
        txn: &mut RwTxn<'_>,
        player_ship_id: u64,
        origin_system_id: u64,
        destination_system_id: u64,
        custody_second: u64,
        due_second: u64,
    ) -> Result<Option<PlayerMailPickup>, SimulationError> {
        let envelope_ids = self.take_queued_envelopes(
            txn,
            origin_system_id,
            destination_system_id,
            custody_second,
        )?;
        if envelope_ids.is_empty() {
            return Ok(None);
        }
        let envelope_count = u16::try_from(envelope_ids.len())
            .map_err(|_| SimulationError::Corrupt("player mailbag too large"))?;
        let mailbag_id = self.take_id(txn, META_NEXT_MAILBAG)?;
        let carrier_leg_id = self.take_id(txn, META_NEXT_CARRIER_LEG)?;
        let advertised_stipend_credits =
            self.player_stipend(txn, origin_system_id, destination_system_id, envelope_count)?;
        let mailbag = Mailbag {
            mailbag_id,
            origin_system_id,
            destination_system_id,
            sealed_second: custody_second,
            delivered_second: None,
            envelope_ids,
        };
        let leg = PlayerCarrierLeg {
            carrier_leg_id,
            mailbag_id,
            player_ship_id,
            origin_system_id,
            destination_system_id,
            custody_second,
            due_second,
            delivered_second: None,
            advertised_stipend_credits,
            paid_second: None,
        };
        self.records.put(
            txn,
            &record_key(RECORD_MAILBAG, mailbag_id),
            &encode_mailbag(&mailbag)?,
        )?;
        self.records.put(
            txn,
            &record_key(RECORD_PLAYER_CARRIER_LEG, carrier_leg_id),
            &encode_player_carrier_leg(&leg),
        )?;
        Ok(Some(PlayerMailPickup {
            mailbag_id,
            carrier_leg_id,
            envelope_count,
            advertised_stipend_credits,
        }))
    }

    /// Hand a player-carried bag to its destination beacon. The delivery and
    /// payment markers are written together. Repeating a completed handoff is
    /// harmless and returns no second stipend.
    pub fn handoff_player_mail(
        &self,
        txn: &mut RwTxn<'_>,
        player_ship_id: u64,
        mailbag_id: u64,
        carrier_leg_id: u64,
        destination_system_id: u64,
        now: u64,
    ) -> Result<PlayerMailHandoff, SimulationError> {
        let mut leg = self.get_player_carrier_leg(txn, carrier_leg_id)?;
        if leg.mailbag_id != mailbag_id
            || leg.player_ship_id != player_ship_id
            || leg.destination_system_id != destination_system_id
        {
            return Err(SimulationError::Corrupt(
                "player mail custody records disagree",
            ));
        }
        if leg.delivered_second.is_some() {
            return Ok(PlayerMailHandoff {
                mailbag_id,
                delivered: 0,
                forwarded: 0,
                expired: 0,
                stipend_credits: 0,
            });
        }
        if now < leg.due_second {
            return Err(SimulationError::Corrupt("player mail arrived before due"));
        }
        let (delivered, forwarded, expired) =
            self.deliver_mailbag(txn, mailbag_id, destination_system_id, now)?;
        leg.delivered_second = Some(now);
        leg.paid_second = Some(now);
        self.records.put(
            txn,
            &record_key(RECORD_PLAYER_CARRIER_LEG, carrier_leg_id),
            &encode_player_carrier_leg(&leg),
        )?;
        Ok(PlayerMailHandoff {
            mailbag_id,
            delivered,
            forwarded,
            expired,
            stipend_credits: leg.advertised_stipend_credits,
        })
    }

    /// Return the system feed as it existed at `now`. A message authored in
    /// the system is locally available from creation; remote availability is
    /// established only by a committed delivery row.
    pub fn available_messages(
        &self,
        txn: &heed::RoTxn<'_>,
        system_id: u64,
        now: u64,
        include_expired: bool,
    ) -> Result<Vec<AvailableMessage>, SimulationError> {
        let mut available = BTreeMap::<u64, AvailableMessage>::new();
        for entry in self.records.prefix_iter(txn, &[RECORD_MESSAGE])? {
            let (key, value) = entry?;
            let message = decode_message(key_id(key)?, value)?;
            if message.origin_system_id == system_id
                && message.created_second <= now
                && (include_expired || now < message.expires_second)
            {
                available.insert(
                    message.message_id,
                    AvailableMessage {
                        available_second: message.created_second,
                        message,
                    },
                );
            }
        }
        for entry in self.records.prefix_iter(txn, &[RECORD_DELIVERY])? {
            let (key, value) = entry?;
            let delivery = decode_delivery(key_id(key)?, value)?;
            if delivery.system_id != system_id || delivery.available_second > now {
                continue;
            }
            let message = self.get_message(txn, delivery.message_id)?;
            if !include_expired && now >= message.expires_second {
                continue;
            }
            available
                .entry(message.message_id)
                .and_modify(|item| {
                    item.available_second = item.available_second.min(delivery.available_second)
                })
                .or_insert(AvailableMessage {
                    message,
                    available_second: delivery.available_second,
                });
        }
        let mut result = available.into_values().collect::<Vec<_>>();
        result.sort_by_key(|item| (item.available_second, item.message.message_id));
        Ok(result)
    }

    pub fn message(
        &self,
        txn: &heed::RoTxn<'_>,
        message_id: u64,
    ) -> Result<Message, SimulationError> {
        self.get_message(txn, message_id)
    }

    /// Make a sender-retained copy available when the sender's player ship
    /// physically reaches the addressed system. An independently routed
    /// envelope may already have arrived; stable message IDs make this
    /// delivery idempotent at the system feed.
    pub fn deliver_player_carried_message(
        &self,
        txn: &mut RwTxn<'_>,
        message_id: u64,
        destination_system_id: u64,
        now: u64,
    ) -> Result<bool, SimulationError> {
        let message = self.get_message(txn, message_id)?;
        if now >= message.expires_second
            || self
                .available_messages(txn, destination_system_id, now, true)?
                .iter()
                .any(|available| available.message.message_id == message_id)
        {
            return Ok(false);
        }
        self.get_system(txn, destination_system_id)?;
        self.create_delivery(txn, None, message_id, destination_system_id, now)?;
        Ok(true)
    }

    /// Return messages originated by one system at an exact simulation time.
    ///
    /// The game store uses this immediately after a SystemDay transaction to
    /// attach authoritative instruments to generated communications before
    /// any player can observe them.
    pub fn messages_originated_at(
        &self,
        txn: &heed::RoTxn<'_>,
        system_id: u64,
        created_second: u64,
        class: MessageClass,
    ) -> Result<Vec<Message>, SimulationError> {
        let mut messages = self
            .available_messages(txn, system_id, created_second, true)?
            .into_iter()
            .map(|available| available.message)
            .filter(|message| {
                message.origin_system_id == system_id
                    && message.created_second == created_second
                    && message.class == class
            })
            .collect::<Vec<_>>();
        messages.sort_by_key(|message| message.message_id);
        Ok(messages)
    }

    /// Replace presentation text while a newly generated message is still in
    /// the enclosing SystemDay transaction. Delivery envelopes refer only to
    /// the identifier, so every later copy observes the same signed subject.
    pub fn set_message_subject(
        &self,
        txn: &mut RwTxn<'_>,
        message_id: u64,
        subject: &str,
    ) -> Result<(), SimulationError> {
        let mut message = self.get_message(txn, message_id)?;
        message.subject = subject.to_owned();
        self.records.put(
            txn,
            &record_key(RECORD_MESSAGE, message_id),
            &encode_message(&message)?,
        )?;
        Ok(())
    }

    pub fn set_message_expiry(
        &self,
        txn: &mut RwTxn<'_>,
        message_id: u64,
        expires_second: u64,
    ) -> Result<(), SimulationError> {
        let mut message = self.get_message(txn, message_id)?;
        if expires_second <= message.created_second {
            return Err(SimulationError::Corrupt(
                "message expiry does not follow filing",
            ));
        }
        message.expires_second = expires_second;
        self.records.put(
            txn,
            &record_key(RECORD_MESSAGE, message_id),
            &encode_message(&message)?,
        )?;
        Ok(())
    }

    /// Originate one server-authored message and route a physical envelope to
    /// each requested destination. The immutable origin copy is available
    /// immediately; every remote copy still has to traverse its stored route.
    pub fn dispatch_message(
        &self,
        txn: &mut RwTxn<'_>,
        now: u64,
        origin_system_id: u64,
        class: MessageClass,
        importance: MessageImportance,
        subject: &str,
        body: &str,
        destination_system_ids: &[u64],
    ) -> Result<(u64, u64), SimulationError> {
        let systems = self.systems(txn)?;
        if !systems
            .iter()
            .any(|system| system.system_id == origin_system_id)
        {
            return Err(SimulationError::Corrupt("message origin is missing"));
        }
        let message_id = self.take_id(txn, META_NEXT_MESSAGE)?;
        let expires_second = now
            .checked_add(class.lifetime_days() * SECONDS_PER_DAY)
            .ok_or(SimulationError::Corrupt("message expiry overflow"))?;
        let message = Message {
            message_id,
            origin_system_id,
            created_second: now,
            expires_second,
            class,
            importance,
            subject: subject.to_owned(),
            body: body.to_owned(),
        };
        self.records.put(
            txn,
            &record_key(RECORD_MESSAGE, message_id),
            &encode_message(&message)?,
        )?;

        let mut destinations = destination_system_ids.to_vec();
        destinations.sort_unstable();
        destinations.dedup();
        let mut envelope_count = 0_u64;
        for destination_system_id in destinations {
            if destination_system_id == origin_system_id {
                continue;
            }
            let Some(route) = shortest_route(&systems, origin_system_id, destination_system_id)
            else {
                // Broadcasts cover the physically connected mail graph. A
                // disconnected materialized island cannot receive an
                // envelope until a route exists.
                continue;
            };
            let envelope_id = self.take_id(txn, META_NEXT_ENVELOPE)?;
            let envelope = DeliveryEnvelope {
                envelope_id,
                message_id,
                destination_system_id,
                route,
                route_index: 0,
                status: EnvelopeStatus::Waiting,
            };
            self.put_envelope(txn, &envelope)?;
            self.queue_envelope(txn, &envelope)?;
            envelope_count = envelope_count
                .checked_add(1)
                .ok_or(SimulationError::Corrupt("message envelope count overflow"))?;
        }
        Ok((message_id, envelope_count))
    }

    pub fn route_exists(
        &self,
        txn: &heed::RoTxn<'_>,
        origin_system_id: u64,
        destination_system_id: u64,
    ) -> Result<bool, SimulationError> {
        let systems = self.systems(txn)?;
        Ok(shortest_route(&systems, origin_system_id, destination_system_id).is_some())
    }

    pub fn route_hops(
        &self,
        txn: &heed::RoTxn<'_>,
        origin_system_id: u64,
        destination_system_id: u64,
    ) -> Result<Option<u64>, SimulationError> {
        let systems = self.systems(txn)?;
        Ok(
            shortest_route(&systems, origin_system_id, destination_system_id)
                .map(|route| route.len().saturating_sub(1) as u64),
        )
    }

    /// Validate that every custody leg names a matching persisted ship and
    /// mailbag and that their endpoints and completion times agree.
    pub fn audit_mail_custody(&self, txn: &heed::RoTxn<'_>) -> Result<u64, SimulationError> {
        let mut audited = 0_u64;
        for entry in self.records.prefix_iter(txn, &[RECORD_CARRIER_LEG])? {
            let (key, value) = entry?;
            let leg = decode_carrier_leg(key_id(key)?, value)?;
            let ship = self.get_traffic_ship(txn, leg.traffic_ship_id)?;
            let bag = self.get_mailbag(txn, leg.mailbag_id)?;
            if ship.mailbag_id != Some(bag.mailbag_id)
                || ship.origin_system_id != leg.origin_system_id
                || ship.destination_system_id != leg.destination_system_id
                || ship.departure_second != leg.custody_second
                || ship.arrival_second != leg.due_second
                || bag.origin_system_id != leg.origin_system_id
                || bag.destination_system_id != leg.destination_system_id
                || bag.sealed_second != leg.custody_second
                || bag.delivered_second != leg.delivered_second
            {
                return Err(SimulationError::Corrupt("mail custody records disagree"));
            }
            audited += 1;
        }
        for entry in self
            .records
            .prefix_iter(txn, &[RECORD_PLAYER_CARRIER_LEG])?
        {
            let (key, value) = entry?;
            let leg = decode_player_carrier_leg(key_id(key)?, value)?;
            let bag = self.get_mailbag(txn, leg.mailbag_id)?;
            if bag.origin_system_id != leg.origin_system_id
                || bag.destination_system_id != leg.destination_system_id
                || bag.sealed_second != leg.custody_second
                || bag.delivered_second != leg.delivered_second
                || leg.delivered_second.is_some() != leg.paid_second.is_some()
            {
                return Err(SimulationError::Corrupt(
                    "player mail custody records disagree",
                ));
            }
            audited += 1;
        }
        Ok(audited)
    }

    fn process_system_day(
        &self,
        txn: &mut RwTxn<'_>,
        now: u64,
        system_id: u64,
        day: u64,
    ) -> Result<String, SimulationError> {
        let mut system = self.get_system(txn, system_id)?;
        if system.next_system_day != day {
            return Err(SimulationError::Corrupt("out-of-order SystemDay"));
        }
        self.expire_waiting_at(txn, system_id, now)?;

        let label = format!("simulation/system-day/v1/{day}");
        let seed = derive_seed(system.generation_seed, label.as_bytes())?;
        let mut random = SeedStream::new(seed);
        let departures = sample_hundredths(traffic_rate_hundredths(&system), &mut random)?;
        let systems = self.connected_polity_systems(txn, &system)?;

        let rates = [
            (MessageClass::AgencyNews, u64::from(system.population) * 12),
            (
                MessageClass::PublicService,
                if system.population >= 6 { 8 } else { 1 },
            ),
            (
                MessageClass::ContractOffer,
                u64::from(system.population.saturating_sub(3)) * 20,
            ),
            (MessageClass::TrafficNotice, departures * 25),
            (MessageClass::Private, 20 + departures * 35),
        ];
        let mut generated = 0_u64;
        let mut envelopes = 0_u64;
        for (class, rate) in rates {
            // Even the smallest inhabited market needs a concrete starting
            // opportunity.  Population still controls the continuing offer
            // rate; this floor applies only to the system's first simulated
            // day and prevents a valid low-population BBS capital from opening
            // with an empty task ledger.
            let count = sample_hundredths(rate, &mut random)?.max(u64::from(
                class == MessageClass::ContractOffer && day == 0 && system.population > 0,
            ));
            for ordinal in 0..count {
                envelopes += self.create_message(
                    txn,
                    now,
                    &system,
                    &systems,
                    class,
                    day,
                    ordinal,
                    &mut random,
                )?;
                generated += 1;
            }
        }

        if !system.jump_two_neighbors.is_empty() {
            let mut destinations = Vec::with_capacity(departures as usize);
            for _ in 0..departures {
                let neighbor_index = random.next_u64()? as usize % system.jump_two_neighbors.len();
                destinations.push(system.jump_two_neighbors[neighbor_index]);
            }
            self.schedule_departure_plan(txn, now, system_id, destinations)?;
        }

        system.next_system_day = system
            .next_system_day
            .checked_add(1)
            .ok_or(SimulationError::Corrupt("SystemDay overflow"))?;
        self.records.put(
            txn,
            &record_key(RECORD_SYSTEM, system_id),
            &encode_system(&system)?,
        )?;
        self.schedule(
            txn,
            now.checked_add(SECONDS_PER_DAY)
                .ok_or(SimulationError::Corrupt("SystemDay time overflow"))?,
            system_id,
            EventKind::SystemDay {
                system_id,
                day: day + 1,
            },
        )?;
        Ok(format!(
            "SystemDay system={system_id} day={day} messages={generated} envelopes={envelopes} departures={departures}"
        ))
    }

    fn connected_polity_systems(
        &self,
        txn: &heed::RoTxn<'_>,
        origin: &SimulationSystem,
    ) -> Result<Vec<SimulationSystem>, SimulationError> {
        // Polity zero means genuinely unaligned, not one enormous implicit
        // polity. Its present message model is therefore local-only.
        if origin.polity_id == 0 {
            return Ok(vec![origin.clone()]);
        }
        let mut systems = Vec::new();
        let mut frontier = VecDeque::from([origin.clone()]);
        let mut visited = BTreeSet::from([origin.system_id]);
        while let Some(system) = frontier.pop_front() {
            for neighbor_id in &system.jump_two_neighbors {
                if visited.contains(neighbor_id) {
                    continue;
                }
                let neighbor = self.get_system(txn, *neighbor_id)?;
                if neighbor.polity_id != origin.polity_id {
                    continue;
                }
                visited.insert(*neighbor_id);
                frontier.push_back(neighbor);
            }
            systems.push(system);
        }
        systems.sort_by_key(|system| system.system_id);
        Ok(systems)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_message(
        &self,
        txn: &mut RwTxn<'_>,
        now: u64,
        origin: &SimulationSystem,
        systems: &[SimulationSystem],
        class: MessageClass,
        day: u64,
        ordinal: u64,
        random: &mut SeedStream,
    ) -> Result<u64, SimulationError> {
        let message_id = self.take_id(txn, META_NEXT_MESSAGE)?;
        let expires_second = now
            .checked_add(class.lifetime_days() * SECONDS_PER_DAY)
            .ok_or(SimulationError::Corrupt("message expiry overflow"))?;
        let (importance, subject, body) = generated_message_text(origin, class, day, ordinal);
        let message = Message {
            message_id,
            origin_system_id: origin.system_id,
            created_second: now,
            expires_second,
            class,
            importance,
            subject,
            body,
        };
        self.records.put(
            txn,
            &record_key(RECORD_MESSAGE, message_id),
            &encode_message(&message)?,
        )?;
        let mut candidate_routes = Vec::new();
        for destination in systems {
            if destination.system_id == origin.system_id
                || destination.polity_id != origin.polity_id
            {
                continue;
            }
            let Some(route) = shortest_route(systems, origin.system_id, destination.system_id)
            else {
                continue;
            };
            let hops = route.len().saturating_sub(1);
            let included = match class {
                MessageClass::AgencyNews | MessageClass::PublicService => true,
                MessageClass::ContractOffer => hops <= 2,
                MessageClass::TrafficNotice => hops == 1,
                MessageClass::Private => true,
            };
            if included {
                candidate_routes.push(route);
            }
        }
        if class == MessageClass::Private && !candidate_routes.is_empty() {
            let selected = random.next_u64()? as usize % candidate_routes.len();
            candidate_routes = vec![candidate_routes.swap_remove(selected)];
        }

        let mut created = 0_u64;
        for route in candidate_routes {
            let envelope_id = self.take_id(txn, META_NEXT_ENVELOPE)?;
            let envelope = DeliveryEnvelope {
                envelope_id,
                message_id,
                destination_system_id: *route
                    .last()
                    .ok_or(SimulationError::Corrupt("empty delivery route"))?,
                route,
                route_index: 0,
                status: EnvelopeStatus::Waiting,
            };
            self.put_envelope(txn, &envelope)?;
            self.queue_envelope(txn, &envelope)?;
            created += 1;
        }
        Ok(created)
    }

    fn process_departure(
        &self,
        txn: &mut RwTxn<'_>,
        now: u64,
        origin_system_id: u64,
        destination_system_id: u64,
    ) -> Result<String, SimulationError> {
        let traffic_ship_id = self.take_id(txn, META_NEXT_TRAFFIC_SHIP)?;
        let arrival_second = now
            .checked_add(STANDARD_JUMP_SECONDS)
            .ok_or(SimulationError::Corrupt("traffic arrival overflow"))?;
        let envelope_ids =
            self.take_queued_envelopes(txn, origin_system_id, destination_system_id, now)?;

        let (mailbag_id, carrier_leg_id) = if envelope_ids.is_empty() {
            (None, None)
        } else {
            let mailbag_id = self.take_id(txn, META_NEXT_MAILBAG)?;
            let carrier_leg_id = self.take_id(txn, META_NEXT_CARRIER_LEG)?;
            let mailbag = Mailbag {
                mailbag_id,
                origin_system_id,
                destination_system_id,
                sealed_second: now,
                delivered_second: None,
                envelope_ids,
            };
            let leg = CarrierLeg {
                carrier_leg_id,
                mailbag_id,
                traffic_ship_id,
                origin_system_id,
                destination_system_id,
                custody_second: now,
                due_second: arrival_second,
                delivered_second: None,
            };
            self.records.put(
                txn,
                &record_key(RECORD_MAILBAG, mailbag_id),
                &encode_mailbag(&mailbag)?,
            )?;
            self.records.put(
                txn,
                &record_key(RECORD_CARRIER_LEG, carrier_leg_id),
                &encode_carrier_leg(&leg),
            )?;
            (Some(mailbag_id), Some(carrier_leg_id))
        };
        let ship = SimulatedTrafficShip {
            traffic_ship_id,
            origin_system_id,
            destination_system_id,
            departure_second: now,
            arrival_second,
            status: TrafficShipStatus::InTransit,
            mailbag_id,
        };
        self.records.put(
            txn,
            &record_key(RECORD_TRAFFIC_SHIP, traffic_ship_id),
            &encode_traffic_ship(&ship),
        )?;
        self.schedule(
            txn,
            arrival_second,
            traffic_ship_id,
            EventKind::TrafficArrival {
                traffic_ship_id,
                mailbag_id,
                carrier_leg_id,
            },
        )?;
        Ok(format!(
            "TrafficDeparture ship={traffic_ship_id} {origin_system_id}->{destination_system_id} mailbag={} envelopes={}",
            mailbag_id.unwrap_or(0),
            mailbag_id
                .map(|id| self.get_mailbag(txn, id).map(|bag| bag.envelope_ids.len()))
                .transpose()?
                .unwrap_or(0)
        ))
    }

    fn process_arrival(
        &self,
        txn: &mut RwTxn<'_>,
        now: u64,
        traffic_ship_id: u64,
        mailbag_id: Option<u64>,
        carrier_leg_id: Option<u64>,
    ) -> Result<(u64, String), SimulationError> {
        let mut ship = self.get_traffic_ship(txn, traffic_ship_id)?;
        if ship.status != TrafficShipStatus::InTransit || ship.arrival_second != now {
            return Err(SimulationError::Corrupt("invalid traffic arrival"));
        }
        ship.status = TrafficShipStatus::Arrived;
        self.records.put(
            txn,
            &record_key(RECORD_TRAFFIC_SHIP, traffic_ship_id),
            &encode_traffic_ship(&ship),
        )?;
        let mut delivered = 0_u64;
        let mut forwarded = 0_u64;
        let mut expired = 0_u64;
        if let Some(mailbag_id) = mailbag_id {
            let leg_id = carrier_leg_id.ok_or(SimulationError::Corrupt(
                "mailbag arrival missing carrier leg",
            ))?;
            let mut leg = self.get_carrier_leg(txn, leg_id)?;
            if leg.mailbag_id != mailbag_id
                || leg.traffic_ship_id != traffic_ship_id
                || leg.destination_system_id != ship.destination_system_id
            {
                return Err(SimulationError::Corrupt(
                    "traffic mail custody records disagree",
                ));
            }
            let counts = self.deliver_mailbag(txn, mailbag_id, ship.destination_system_id, now)?;
            leg.delivered_second = Some(now);
            self.records.put(
                txn,
                &record_key(RECORD_CARRIER_LEG, leg_id),
                &encode_carrier_leg(&leg),
            )?;
            (delivered, forwarded, expired) = counts;
        } else if carrier_leg_id.is_some() {
            return Err(SimulationError::Corrupt("carrier leg without mailbag"));
        }
        Ok((
            ship.destination_system_id,
            format!(
                "TrafficArrival ship={traffic_ship_id} system={} delivered={delivered} forwarded={forwarded} expired={expired}",
                ship.destination_system_id
            ),
        ))
    }

    fn expire_waiting_at(
        &self,
        txn: &mut RwTxn<'_>,
        system_id: u64,
        now: u64,
    ) -> Result<(), SimulationError> {
        let mut prefix = vec![RECORD_QUEUE];
        prefix.extend_from_slice(&system_id.to_be_bytes());
        let queued = self
            .records
            .prefix_iter(txn, &prefix)?
            .map(|entry| {
                let (key, _) = entry?;
                Ok((key.to_vec(), key_id(key)?))
            })
            .collect::<Result<Vec<_>, SimulationError>>()?;
        for (key, envelope_id) in queued {
            let mut envelope = self.get_envelope(txn, envelope_id)?;
            if now >= self.get_message(txn, envelope.message_id)?.expires_second {
                self.records.delete(txn, &key)?;
                envelope.status = EnvelopeStatus::Expired;
                self.put_envelope(txn, &envelope)?;
            }
        }
        Ok(())
    }

    fn take_queued_envelopes(
        &self,
        txn: &mut RwTxn<'_>,
        origin_system_id: u64,
        destination_system_id: u64,
        now: u64,
    ) -> Result<Vec<u64>, SimulationError> {
        let prefix = queue_prefix(origin_system_id, destination_system_id);
        let queued = self
            .records
            .prefix_iter(txn, &prefix)?
            .take(MAILBAG_ENVELOPE_LIMIT)
            .map(|entry| {
                let (key, _) = entry?;
                Ok((key.to_vec(), key_id(key)?))
            })
            .collect::<Result<Vec<_>, SimulationError>>()?;
        let mut envelope_ids = Vec::new();
        for (key, envelope_id) in queued {
            let mut envelope = self.get_envelope(txn, envelope_id)?;
            let message = self.get_message(txn, envelope.message_id)?;
            self.records.delete(txn, &key)?;
            if now >= message.expires_second {
                envelope.status = EnvelopeStatus::Expired;
            } else {
                envelope.status = EnvelopeStatus::InTransit;
                envelope_ids.push(envelope_id);
            }
            self.put_envelope(txn, &envelope)?;
        }
        Ok(envelope_ids)
    }

    fn deliver_mailbag(
        &self,
        txn: &mut RwTxn<'_>,
        mailbag_id: u64,
        destination_system_id: u64,
        now: u64,
    ) -> Result<(u64, u64, u64), SimulationError> {
        let mut bag = self.get_mailbag(txn, mailbag_id)?;
        if bag.destination_system_id != destination_system_id {
            return Err(SimulationError::Corrupt("mailbag arrived at wrong beacon"));
        }
        if bag.delivered_second.is_some() {
            return Ok((0, 0, 0));
        }
        bag.delivered_second = Some(now);
        self.records.put(
            txn,
            &record_key(RECORD_MAILBAG, mailbag_id),
            &encode_mailbag(&bag)?,
        )?;
        let mut delivered = 0_u64;
        let mut forwarded = 0_u64;
        let mut expired = 0_u64;
        for envelope_id in bag.envelope_ids {
            let mut envelope = self.get_envelope(txn, envelope_id)?;
            let message = self.get_message(txn, envelope.message_id)?;
            if envelope.status == EnvelopeStatus::Expired || now >= message.expires_second {
                envelope.status = EnvelopeStatus::Expired;
                expired += 1;
            } else {
                envelope.route_index = envelope
                    .route_index
                    .checked_add(1)
                    .ok_or(SimulationError::Corrupt("route index overflow"))?;
                let current = *envelope
                    .route
                    .get(envelope.route_index as usize)
                    .ok_or(SimulationError::Corrupt("carrier left delivery route"))?;
                if current != destination_system_id {
                    return Err(SimulationError::Corrupt("carrier arrived off route"));
                }
                if current == envelope.destination_system_id {
                    envelope.status = EnvelopeStatus::Delivered;
                    self.create_delivery(
                        txn,
                        Some(envelope.envelope_id),
                        envelope.message_id,
                        current,
                        now,
                    )?;
                    delivered += 1;
                } else {
                    envelope.status = EnvelopeStatus::Waiting;
                    self.queue_envelope(txn, &envelope)?;
                    forwarded += 1;
                }
            }
            self.put_envelope(txn, &envelope)?;
        }
        Ok((delivered, forwarded, expired))
    }

    fn player_stipend(
        &self,
        txn: &heed::RoTxn<'_>,
        origin_system_id: u64,
        destination_system_id: u64,
        envelope_count: u16,
    ) -> Result<u64, SimulationError> {
        let origin = self.get_system(txn, origin_system_id)?;
        if !origin.jump_two_neighbors.contains(&destination_system_id) {
            return Err(SimulationError::Corrupt(
                "mailbag route is not a direct beacon hop",
            ));
        }
        // The initial provisional tariff never causes or redirects a voyage.
        // Every direct hop pays the same token handling amount to a ship
        // already making that transit, plus the same amount per envelope.
        // Route activity, urgency, and danger do not alter it.
        Ok(100 + u64::from(envelope_count))
    }

    fn create_delivery(
        &self,
        txn: &mut RwTxn<'_>,
        envelope_id: Option<u64>,
        message_id: u64,
        system_id: u64,
        available_second: u64,
    ) -> Result<(), SimulationError> {
        let delivery_id = self.take_id(txn, META_NEXT_DELIVERY)?;
        let delivery = MessageDelivery {
            delivery_id,
            envelope_id,
            message_id,
            system_id,
            available_second,
        };
        self.records.put(
            txn,
            &record_key(RECORD_DELIVERY, delivery_id),
            &encode_delivery(&delivery),
        )?;
        Ok(())
    }

    fn queue_envelope(
        &self,
        txn: &mut RwTxn<'_>,
        envelope: &DeliveryEnvelope,
    ) -> Result<(), SimulationError> {
        let index = envelope.route_index as usize;
        let current = *envelope.route.get(index).ok_or(SimulationError::Corrupt(
            "delivery route index out of bounds",
        ))?;
        let next = *envelope
            .route
            .get(index + 1)
            .ok_or(SimulationError::Corrupt("queued delivery has no next hop"))?;
        self.records
            .put(txn, &queue_key(current, next, envelope.envelope_id), &[])?;
        Ok(())
    }

    fn put_envelope(
        &self,
        txn: &mut RwTxn<'_>,
        envelope: &DeliveryEnvelope,
    ) -> Result<(), SimulationError> {
        self.records.put(
            txn,
            &record_key(RECORD_ENVELOPE, envelope.envelope_id),
            &encode_envelope(envelope)?,
        )?;
        Ok(())
    }

    fn schedule(
        &self,
        txn: &mut RwTxn<'_>,
        due_second: u64,
        entity_id: u64,
        kind: EventKind,
    ) -> Result<u64, SimulationError> {
        let event_id = self.take_id(txn, META_NEXT_EVENT)?;
        let event = ScheduledEvent {
            event_id,
            due_second,
            entity_id,
            kind,
        };
        self.events
            .put(txn, &event_key(due_second, event_id), &encode_event(&event))?;
        Ok(event_id)
    }

    fn schedule_departure_plan(
        &self,
        txn: &mut RwTxn<'_>,
        start_second: u64,
        origin_system_id: u64,
        destinations: Vec<u64>,
    ) -> Result<(), SimulationError> {
        if destinations.is_empty() {
            return Ok(());
        }
        let total = u16::try_from(destinations.len())
            .map_err(|_| SimulationError::Corrupt("too many daily departures"))?;
        let first_event_id = self.reserve_ids(txn, META_NEXT_EVENT, u64::from(total))?;
        let event = ScheduledEvent {
            event_id: first_event_id,
            due_second: start_second,
            entity_id: origin_system_id,
            kind: EventKind::TrafficDeparturePlan {
                origin_system_id,
                start_second,
                ordinal: 0,
                total,
                destinations,
            },
        };
        self.events.put(
            txn,
            &event_key(event.due_second, event.event_id),
            &encode_event(&event),
        )?;
        Ok(())
    }

    fn take_id(&self, txn: &mut RwTxn<'_>, key: &[u8]) -> Result<u64, SimulationError> {
        self.reserve_ids(txn, key, 1)
    }

    fn reserve_ids(
        &self,
        txn: &mut RwTxn<'_>,
        key: &[u8],
        count: u64,
    ) -> Result<u64, SimulationError> {
        if count == 0 {
            return Err(SimulationError::Corrupt("cannot reserve zero identifiers"));
        }
        let current = self
            .records
            .get(txn, key)?
            .map(decode_exact_u64)
            .transpose()?
            .unwrap_or(1);
        let next = current
            .checked_add(count)
            .ok_or(SimulationError::Corrupt("simulation identifier overflow"))?;
        self.records.put(txn, key, &next.to_be_bytes())?;
        Ok(current)
    }

    fn get_system(
        &self,
        txn: &heed::RoTxn<'_>,
        id: u64,
    ) -> Result<SimulationSystem, SimulationError> {
        self.records
            .get(txn, &record_key(RECORD_SYSTEM, id))?
            .map(|bytes| decode_system(id, bytes))
            .transpose()?
            .ok_or(SimulationError::Corrupt("missing simulation system"))
    }

    fn get_message(&self, txn: &heed::RoTxn<'_>, id: u64) -> Result<Message, SimulationError> {
        self.records
            .get(txn, &record_key(RECORD_MESSAGE, id))?
            .map(|bytes| decode_message(id, bytes))
            .transpose()?
            .ok_or(SimulationError::Corrupt("missing message"))
    }

    fn get_envelope(
        &self,
        txn: &heed::RoTxn<'_>,
        id: u64,
    ) -> Result<DeliveryEnvelope, SimulationError> {
        self.records
            .get(txn, &record_key(RECORD_ENVELOPE, id))?
            .map(|bytes| decode_envelope(id, bytes))
            .transpose()?
            .ok_or(SimulationError::Corrupt("missing delivery envelope"))
    }

    fn get_traffic_ship(
        &self,
        txn: &heed::RoTxn<'_>,
        id: u64,
    ) -> Result<SimulatedTrafficShip, SimulationError> {
        self.records
            .get(txn, &record_key(RECORD_TRAFFIC_SHIP, id))?
            .map(|bytes| decode_traffic_ship(id, bytes))
            .transpose()?
            .ok_or(SimulationError::Corrupt("missing traffic ship"))
    }

    fn get_mailbag(&self, txn: &heed::RoTxn<'_>, id: u64) -> Result<Mailbag, SimulationError> {
        self.records
            .get(txn, &record_key(RECORD_MAILBAG, id))?
            .map(|bytes| decode_mailbag(id, bytes))
            .transpose()?
            .ok_or(SimulationError::Corrupt("missing mailbag"))
    }

    fn get_carrier_leg(
        &self,
        txn: &heed::RoTxn<'_>,
        id: u64,
    ) -> Result<CarrierLeg, SimulationError> {
        self.records
            .get(txn, &record_key(RECORD_CARRIER_LEG, id))?
            .map(|bytes| decode_carrier_leg(id, bytes))
            .transpose()?
            .ok_or(SimulationError::Corrupt("missing carrier leg"))
    }

    fn get_player_carrier_leg(
        &self,
        txn: &heed::RoTxn<'_>,
        id: u64,
    ) -> Result<PlayerCarrierLeg, SimulationError> {
        self.records
            .get(txn, &record_key(RECORD_PLAYER_CARRIER_LEG, id))?
            .map(|bytes| decode_player_carrier_leg(id, bytes))
            .transpose()?
            .ok_or(SimulationError::Corrupt("missing player carrier leg"))
    }
}

pub(crate) fn traffic_rate_hundredths(system: &SimulationSystem) -> u64 {
    let population_rate = [5_u64, 3, 5, 10, 20, 50, 100, 250, 800, 2_000, 5_000]
        [usize::from(system.population.min(10))];
    let technology_percent = 50 + 5 * u64::from(system.tech_level.min(15));
    let starport_percent = match system.starport {
        0 => 150,
        1 => 125,
        2 => 100,
        3 => 75,
        4 => 50,
        _ => 25,
    };
    population_rate * technology_percent * starport_percent / 10_000
}

fn sample_hundredths(rate: u64, random: &mut SeedStream) -> Result<u64, CryptoError> {
    let whole = rate / 100;
    let remainder = rate % 100;
    Ok(whole + u64::from(random.next_u64()? % 100 < remainder))
}

pub(crate) fn shortest_route(
    systems: &[SimulationSystem],
    origin: u64,
    destination: u64,
) -> Option<Vec<u64>> {
    if origin == destination {
        return Some(vec![origin]);
    }
    let by_id = systems
        .iter()
        .map(|system| (system.system_id, system))
        .collect::<BTreeMap<_, _>>();
    let mut frontier = VecDeque::from([origin]);
    let mut previous = BTreeMap::new();
    let mut visited = BTreeSet::from([origin]);
    while let Some(current) = frontier.pop_front() {
        let system = by_id.get(&current)?;
        for neighbor in &system.jump_two_neighbors {
            if !visited.insert(*neighbor) {
                continue;
            }
            previous.insert(*neighbor, current);
            if *neighbor == destination {
                let mut route = vec![destination];
                let mut cursor = destination;
                while cursor != origin {
                    cursor = previous[&cursor];
                    route.push(cursor);
                }
                route.reverse();
                return Some(route);
            }
            frontier.push_back(*neighbor);
        }
    }
    None
}

fn jump_two_neighbor_lists(systems: &[SimulationSystem]) -> Vec<Vec<u64>> {
    // A two-parsec spatial bucket has the useful property that every J-2
    // neighbor is either in the same bucket or one of its 26 immediate
    // neighbors. This preserves the exact Euclidean range test without making
    // universe initialization quadratic in the number of systems.
    let bucket_coordinate = |position: [f64; 3]| {
        (
            (position[0] / 2.0).floor() as i64,
            (position[1] / 2.0).floor() as i64,
            (position[2] / 2.0).floor() as i64,
        )
    };
    let mut buckets = BTreeMap::<(i64, i64, i64), Vec<usize>>::new();
    for (index, system) in systems.iter().enumerate() {
        buckets
            .entry(bucket_coordinate(system.position_parsecs))
            .or_default()
            .push(index);
    }
    let mut neighbors = vec![Vec::<u64>::new(); systems.len()];
    for index in 0..systems.len() {
        let (bucket_x, bucket_y, bucket_z) = bucket_coordinate(systems[index].position_parsecs);
        for offset_x in -1..=1 {
            for offset_y in -1..=1 {
                for offset_z in -1..=1 {
                    let Some(candidates) = buckets.get(&(
                        bucket_x + offset_x,
                        bucket_y + offset_y,
                        bucket_z + offset_z,
                    )) else {
                        continue;
                    };
                    for &other_index in candidates {
                        if other_index <= index {
                            continue;
                        }
                        let distance_squared = systems[index]
                            .position_parsecs
                            .iter()
                            .zip(systems[other_index].position_parsecs)
                            .map(|(left, right)| (left - right).powi(2))
                            .sum::<f64>();
                        if distance_squared <= JUMP_TWO_DISTANCE_SQUARED {
                            neighbors[index].push(systems[other_index].system_id);
                            neighbors[other_index].push(systems[index].system_id);
                        }
                    }
                }
            }
        }
    }
    for list in &mut neighbors {
        list.sort_unstable();
    }
    neighbors
}

fn record_key(prefix: u8, id: u64) -> [u8; 9] {
    let mut key = [0; 9];
    key[0] = prefix;
    key[1..].copy_from_slice(&id.to_be_bytes());
    key
}

fn queue_prefix(system_id: u64, next_hop: u64) -> [u8; 17] {
    let mut key = [0; 17];
    key[0] = RECORD_QUEUE;
    key[1..9].copy_from_slice(&system_id.to_be_bytes());
    key[9..17].copy_from_slice(&next_hop.to_be_bytes());
    key
}

fn queue_key(system_id: u64, next_hop: u64, envelope_id: u64) -> [u8; 25] {
    let mut key = [0; 25];
    key[..17].copy_from_slice(&queue_prefix(system_id, next_hop));
    key[17..].copy_from_slice(&envelope_id.to_be_bytes());
    key
}

fn event_key(due_second: u64, event_id: u64) -> [u8; 16] {
    let mut key = [0; 16];
    key[..8].copy_from_slice(&due_second.to_be_bytes());
    key[8..].copy_from_slice(&event_id.to_be_bytes());
    key
}

fn key_id(key: &[u8]) -> Result<u64, SimulationError> {
    if key.len() < 8 {
        return Err(SimulationError::Corrupt("short simulation key"));
    }
    decode_exact_u64(&key[key.len() - 8..])
}

fn decode_exact_u64(bytes: &[u8]) -> Result<u64, SimulationError> {
    Ok(u64::from_be_bytes(
        bytes
            .try_into()
            .map_err(|_| SimulationError::Corrupt("invalid u64"))?,
    ))
}

fn encode_string(bytes: &mut Vec<u8>, value: &str) -> Result<(), SimulationError> {
    let length = u16::try_from(value.len())
        .map_err(|_| SimulationError::Corrupt("simulation string too long"))?;
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], SimulationError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(SimulationError::Corrupt("simulation decode overflow"))?;
        let result = self
            .bytes
            .get(self.offset..end)
            .ok_or(SimulationError::Corrupt("truncated simulation record"))?;
        self.offset = end;
        Ok(result)
    }

    fn u8(&mut self) -> Result<u8, SimulationError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, SimulationError> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, SimulationError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, SimulationError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn string(&mut self) -> Result<String, SimulationError> {
        let length = self.u16()? as usize;
        Ok(std::str::from_utf8(self.take(length)?)
            .map_err(|_| SimulationError::Corrupt("invalid simulation text"))?
            .to_owned())
    }

    fn finish(self) -> Result<(), SimulationError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(SimulationError::Corrupt("trailing simulation record data"))
        }
    }
}

fn encode_system(system: &SimulationSystem) -> Result<Vec<u8>, SimulationError> {
    let mut bytes = vec![3];
    let default_name = format!("Frontier {}", system.system_id);
    if system.name == default_name {
        bytes.push(0);
    } else {
        bytes.push(1);
        encode_string(&mut bytes, &system.name)?;
    }
    for coordinate in system.position_parsecs {
        bytes.extend_from_slice(&coordinate.to_bits().to_be_bytes());
    }
    bytes.extend_from_slice(&system.polity_id.to_be_bytes());
    bytes.extend_from_slice(&system.generation_seed);
    bytes.push(system.population);
    bytes.push(system.tech_level);
    bytes.push(system.starport);
    bytes.extend_from_slice(&system.next_system_day.to_be_bytes());
    let count = u16::try_from(system.jump_two_neighbors.len())
        .map_err(|_| SimulationError::Corrupt("too many J-2 neighbors"))?;
    bytes.extend_from_slice(&count.to_be_bytes());
    for neighbor in &system.jump_two_neighbors {
        bytes.extend_from_slice(&neighbor.to_be_bytes());
    }
    Ok(bytes)
}

fn decode_system(id: u64, bytes: &[u8]) -> Result<SimulationSystem, SimulationError> {
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 3 {
        return Err(SimulationError::Corrupt("unsupported simulation system"));
    }
    let system_id = id;
    let name = match decoder.u8()? {
        0 => format!("Frontier {system_id}"),
        1 => decoder.string()?,
        _ => return Err(SimulationError::Corrupt("unknown system name encoding")),
    };
    let mut position_parsecs = [0.0; 3];
    for coordinate in &mut position_parsecs {
        *coordinate = f64::from_bits(decoder.u64()?);
    }
    let polity_id = decoder.u64()?;
    let generation_seed = decoder.take(32)?.try_into().unwrap();
    let population = decoder.u8()?;
    let tech_level = decoder.u8()?;
    let starport = decoder.u8()?;
    let next_system_day = decoder.u64()?;
    let count = decoder.u16()? as usize;
    let mut jump_two_neighbors = Vec::with_capacity(count);
    for _ in 0..count {
        jump_two_neighbors.push(decoder.u64()?);
    }
    decoder.finish()?;
    Ok(SimulationSystem {
        system_id,
        name,
        position_parsecs,
        polity_id,
        generation_seed,
        population,
        tech_level,
        starport,
        next_system_day,
        jump_two_neighbors,
    })
}

fn encode_message(message: &Message) -> Result<Vec<u8>, SimulationError> {
    let mut bytes = vec![3];
    bytes.extend_from_slice(&message.origin_system_id.to_be_bytes());
    bytes.extend_from_slice(&message.created_second.to_be_bytes());
    bytes.extend_from_slice(&message.expires_second.to_be_bytes());
    bytes.push(message.class as u8);
    bytes.push(message.importance as u8);
    encode_string(&mut bytes, &message.subject)?;
    encode_string(&mut bytes, &message.body)?;
    Ok(bytes)
}

fn generated_message_text(
    origin: &SimulationSystem,
    class: MessageClass,
    day: u64,
    ordinal: u64,
) -> (MessageImportance, String, String) {
    let bulletin = ordinal + 1;
    match class {
        MessageClass::AgencyNews => (
            MessageImportance::Notable,
            format!("{} market and shipping report", origin.name),
            format!(
                "The {} exchange filed its day {} report. The primary market serves population code {} at tech level {} through a class {} port. This is agency bulletin {} for the reporting day.",
                origin.name,
                day,
                origin.population,
                origin.tech_level,
                crate::universe::Starport::from_record(origin.starport)
                    .map_or('X', crate::universe::Starport::code),
                bulletin,
            ),
        ),
        MessageClass::PublicService => (
            MessageImportance::Routine,
            format!("Public information bulletin from {}", origin.name),
            format!(
                "The {} public information office issued bulletin {} on day {}. Relays are asked to retain this notice until its stated expiry.",
                origin.name, bulletin, day,
            ),
        ),
        MessageClass::ContractOffer => (
            MessageImportance::Notable,
            format!("Commercial offer filed at {}", origin.name),
            "A signed commercial instrument accompanies this notice. Availability and complete terms are recorded below."
                .to_owned(),
        ),
        MessageClass::TrafficNotice => (
            MessageImportance::Routine,
            format!("Traffic advisory from {}", origin.name),
            format!(
                "{} traffic control filed movement advisory {} for operational day {}. The report is historical by the time it reaches another system and must not be treated as a live sensor contact.",
                origin.name, bulletin, day,
            ),
        ),
        MessageClass::Private => (
            MessageImportance::Important,
            format!("Sealed dispatch from {}", origin.name),
            "This encrypted institutional dispatch is available only to its addressed recipient."
                .to_owned(),
        ),
    }
}

fn decode_message(id: u64, bytes: &[u8]) -> Result<Message, SimulationError> {
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 3 {
        return Err(SimulationError::Corrupt("unsupported message"));
    }
    let message_id = id;
    let result = Message {
        message_id,
        origin_system_id: decoder.u64()?,
        created_second: decoder.u64()?,
        expires_second: decoder.u64()?,
        class: MessageClass::from_u8(decoder.u8()?)?,
        importance: MessageImportance::from_u8(decoder.u8()?)?,
        subject: decoder.string()?,
        body: decoder.string()?,
    };
    decoder.finish()?;
    Ok(result)
}

fn encode_envelope(envelope: &DeliveryEnvelope) -> Result<Vec<u8>, SimulationError> {
    let mut bytes = vec![2];
    bytes.extend_from_slice(&envelope.message_id.to_be_bytes());
    bytes.extend_from_slice(&envelope.destination_system_id.to_be_bytes());
    bytes.extend_from_slice(&envelope.route_index.to_be_bytes());
    bytes.push(envelope.status as u8);
    let count = u16::try_from(envelope.route.len())
        .map_err(|_| SimulationError::Corrupt("delivery route too long"))?;
    bytes.extend_from_slice(&count.to_be_bytes());
    for system_id in &envelope.route {
        bytes.extend_from_slice(&system_id.to_be_bytes());
    }
    Ok(bytes)
}

fn decode_envelope(id: u64, bytes: &[u8]) -> Result<DeliveryEnvelope, SimulationError> {
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 2 {
        return Err(SimulationError::Corrupt("unsupported delivery envelope"));
    }
    let envelope_id = id;
    let message_id = decoder.u64()?;
    let destination_system_id = decoder.u64()?;
    let route_index = decoder.u16()?;
    let status = EnvelopeStatus::from_u8(decoder.u8()?)?;
    let count = decoder.u16()? as usize;
    let mut route = Vec::with_capacity(count);
    for _ in 0..count {
        route.push(decoder.u64()?);
    }
    decoder.finish()?;
    if route.is_empty() || usize::from(route_index) >= route.len() {
        return Err(SimulationError::Corrupt("invalid delivery route"));
    }
    Ok(DeliveryEnvelope {
        envelope_id,
        message_id,
        destination_system_id,
        route,
        route_index,
        status,
    })
}

fn encode_traffic_ship(ship: &SimulatedTrafficShip) -> Vec<u8> {
    let mut bytes = vec![2];
    bytes.extend_from_slice(&ship.origin_system_id.to_be_bytes());
    bytes.extend_from_slice(&ship.destination_system_id.to_be_bytes());
    bytes.extend_from_slice(&ship.departure_second.to_be_bytes());
    bytes.extend_from_slice(&ship.arrival_second.to_be_bytes());
    bytes.push(ship.status as u8);
    bytes.extend_from_slice(&ship.mailbag_id.unwrap_or(0).to_be_bytes());
    bytes
}

fn decode_traffic_ship(id: u64, bytes: &[u8]) -> Result<SimulatedTrafficShip, SimulationError> {
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 2 {
        return Err(SimulationError::Corrupt("unsupported traffic ship"));
    }
    let traffic_ship_id = id;
    let origin_system_id = decoder.u64()?;
    let destination_system_id = decoder.u64()?;
    let departure_second = decoder.u64()?;
    let arrival_second = decoder.u64()?;
    let status = match decoder.u8()? {
        0 => TrafficShipStatus::InTransit,
        1 => TrafficShipStatus::Arrived,
        _ => return Err(SimulationError::Corrupt("unknown traffic ship status")),
    };
    let mailbag_id = match decoder.u64()? {
        0 => None,
        id => Some(id),
    };
    decoder.finish()?;
    Ok(SimulatedTrafficShip {
        traffic_ship_id,
        origin_system_id,
        destination_system_id,
        departure_second,
        arrival_second,
        status,
        mailbag_id,
    })
}

fn encode_mailbag(mailbag: &Mailbag) -> Result<Vec<u8>, SimulationError> {
    let mut bytes = vec![2];
    bytes.extend_from_slice(&mailbag.origin_system_id.to_be_bytes());
    bytes.extend_from_slice(&mailbag.destination_system_id.to_be_bytes());
    bytes.extend_from_slice(&mailbag.sealed_second.to_be_bytes());
    bytes.extend_from_slice(&mailbag.delivered_second.unwrap_or(0).to_be_bytes());
    let count = u16::try_from(mailbag.envelope_ids.len())
        .map_err(|_| SimulationError::Corrupt("mailbag too large"))?;
    bytes.extend_from_slice(&count.to_be_bytes());
    for envelope_id in &mailbag.envelope_ids {
        bytes.extend_from_slice(&envelope_id.to_be_bytes());
    }
    Ok(bytes)
}

fn decode_mailbag(id: u64, bytes: &[u8]) -> Result<Mailbag, SimulationError> {
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 2 {
        return Err(SimulationError::Corrupt("unsupported mailbag"));
    }
    let mailbag_id = id;
    let origin_system_id = decoder.u64()?;
    let destination_system_id = decoder.u64()?;
    let sealed_second = decoder.u64()?;
    let delivered_second = match decoder.u64()? {
        0 => None,
        second => Some(second),
    };
    let count = decoder.u16()? as usize;
    let mut envelope_ids = Vec::with_capacity(count);
    for _ in 0..count {
        envelope_ids.push(decoder.u64()?);
    }
    decoder.finish()?;
    Ok(Mailbag {
        mailbag_id,
        origin_system_id,
        destination_system_id,
        sealed_second,
        delivered_second,
        envelope_ids,
    })
}

fn encode_carrier_leg(leg: &CarrierLeg) -> Vec<u8> {
    let mut bytes = vec![2];
    for value in [
        leg.mailbag_id,
        leg.traffic_ship_id,
        leg.origin_system_id,
        leg.destination_system_id,
        leg.custody_second,
        leg.due_second,
        leg.delivered_second.unwrap_or(0),
    ] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes
}

fn decode_carrier_leg(id: u64, bytes: &[u8]) -> Result<CarrierLeg, SimulationError> {
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 2 {
        return Err(SimulationError::Corrupt("unsupported carrier leg"));
    }
    let carrier_leg_id = id;
    let mailbag_id = decoder.u64()?;
    let traffic_ship_id = decoder.u64()?;
    let origin_system_id = decoder.u64()?;
    let destination_system_id = decoder.u64()?;
    let custody_second = decoder.u64()?;
    let due_second = decoder.u64()?;
    let delivered_second = match decoder.u64()? {
        0 => None,
        second => Some(second),
    };
    decoder.finish()?;
    Ok(CarrierLeg {
        carrier_leg_id,
        mailbag_id,
        traffic_ship_id,
        origin_system_id,
        destination_system_id,
        custody_second,
        due_second,
        delivered_second,
    })
}

fn encode_player_carrier_leg(leg: &PlayerCarrierLeg) -> Vec<u8> {
    let mut bytes = vec![1];
    for value in [
        leg.mailbag_id,
        leg.player_ship_id,
        leg.origin_system_id,
        leg.destination_system_id,
        leg.custody_second,
        leg.due_second,
        leg.delivered_second.unwrap_or(0),
        leg.advertised_stipend_credits,
        leg.paid_second.unwrap_or(0),
    ] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes
}

fn decode_player_carrier_leg(id: u64, bytes: &[u8]) -> Result<PlayerCarrierLeg, SimulationError> {
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 1 {
        return Err(SimulationError::Corrupt("unsupported player carrier leg"));
    }
    let result = PlayerCarrierLeg {
        carrier_leg_id: id,
        mailbag_id: decoder.u64()?,
        player_ship_id: decoder.u64()?,
        origin_system_id: decoder.u64()?,
        destination_system_id: decoder.u64()?,
        custody_second: decoder.u64()?,
        due_second: decoder.u64()?,
        delivered_second: match decoder.u64()? {
            0 => None,
            second => Some(second),
        },
        advertised_stipend_credits: decoder.u64()?,
        paid_second: match decoder.u64()? {
            0 => None,
            second => Some(second),
        },
    };
    decoder.finish()?;
    Ok(result)
}

fn encode_delivery(delivery: &MessageDelivery) -> Vec<u8> {
    let mut bytes = vec![2];
    for value in [
        delivery.envelope_id.unwrap_or(0),
        delivery.message_id,
        delivery.system_id,
        delivery.available_second,
    ] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes
}

fn decode_delivery(id: u64, bytes: &[u8]) -> Result<MessageDelivery, SimulationError> {
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 2 {
        return Err(SimulationError::Corrupt("unsupported message delivery"));
    }
    let delivery_id = id;
    let envelope_id = match decoder.u64()? {
        0 => None,
        id => Some(id),
    };
    let result = MessageDelivery {
        delivery_id,
        envelope_id,
        message_id: decoder.u64()?,
        system_id: decoder.u64()?,
        available_second: decoder.u64()?,
    };
    decoder.finish()?;
    Ok(result)
}

fn encode_event(event: &ScheduledEvent) -> Vec<u8> {
    let mut bytes = vec![3];
    bytes.extend_from_slice(&event.entity_id.to_be_bytes());
    bytes.extend_from_slice(&encode_event_kind(event));
    bytes
}

fn encode_event_kind(event: &ScheduledEvent) -> Vec<u8> {
    let mut bytes = Vec::new();
    match event.kind {
        EventKind::SystemDay { system_id, day } => {
            debug_assert_eq!(system_id, event.entity_id);
            bytes.push(0);
            bytes.extend_from_slice(&day.to_be_bytes());
        }
        EventKind::TrafficDeparture {
            origin_system_id,
            destination_system_id,
        } => {
            debug_assert_eq!(origin_system_id, event.entity_id);
            bytes.push(1);
            bytes.extend_from_slice(&destination_system_id.to_be_bytes());
        }
        EventKind::TrafficDeparturePlan {
            origin_system_id,
            start_second,
            ordinal,
            total,
            ref destinations,
        } => {
            debug_assert_eq!(origin_system_id, event.entity_id);
            bytes.push(3);
            bytes.extend_from_slice(&start_second.to_be_bytes());
            bytes.extend_from_slice(&ordinal.to_be_bytes());
            bytes.extend_from_slice(&total.to_be_bytes());
            bytes.extend_from_slice(&(destinations.len() as u16).to_be_bytes());
            for destination in destinations {
                bytes.extend_from_slice(&destination.to_be_bytes());
            }
        }
        EventKind::TrafficArrival {
            traffic_ship_id,
            mailbag_id,
            carrier_leg_id,
        } => {
            debug_assert_eq!(traffic_ship_id, event.entity_id);
            bytes.push(2);
            bytes.extend_from_slice(&mailbag_id.unwrap_or(0).to_be_bytes());
            bytes.extend_from_slice(&carrier_leg_id.unwrap_or(0).to_be_bytes());
        }
    }
    bytes
}

fn decode_event(key: &[u8], bytes: &[u8]) -> Result<ScheduledEvent, SimulationError> {
    if key.len() != 16 {
        return Err(SimulationError::Corrupt("invalid scheduled event key"));
    }
    let due_second = decode_exact_u64(&key[..8])?;
    let event_id = decode_exact_u64(&key[8..])?;
    let mut decoder = Decoder::new(bytes);
    if decoder.u8()? != 3 {
        return Err(SimulationError::Corrupt("unsupported scheduled event"));
    }
    let entity_id = decoder.u64()?;
    let encoded_kind = decoder.take(bytes.len().saturating_sub(9))?;
    decoder.finish()?;
    decode_event_parts(event_id, due_second, entity_id, encoded_kind)
}

fn decode_queued_event(queued: &QueuedSimulationEvent) -> Result<ScheduledEvent, SimulationError> {
    decode_event_parts(
        queued.event_id,
        queued.due_second,
        queued.entity_id,
        &queued.encoded_kind,
    )
}

fn decode_event_parts(
    event_id: u64,
    due_second: u64,
    entity_id: u64,
    encoded_kind: &[u8],
) -> Result<ScheduledEvent, SimulationError> {
    let mut decoder = Decoder::new(encoded_kind);
    let kind = match decoder.u8()? {
        0 => EventKind::SystemDay {
            system_id: entity_id,
            day: decoder.u64()?,
        },
        1 => EventKind::TrafficDeparture {
            origin_system_id: entity_id,
            destination_system_id: decoder.u64()?,
        },
        2 => {
            let traffic_ship_id = entity_id;
            let mailbag_id = match decoder.u64()? {
                0 => None,
                id => Some(id),
            };
            let carrier_leg_id = match decoder.u64()? {
                0 => None,
                id => Some(id),
            };
            EventKind::TrafficArrival {
                traffic_ship_id,
                mailbag_id,
                carrier_leg_id,
            }
        }
        3 => {
            let start_second = decoder.u64()?;
            let ordinal = decoder.u16()?;
            let total = decoder.u16()?;
            let count = decoder.u16()? as usize;
            if total == 0 || ordinal >= total || count != usize::from(total - ordinal) {
                return Err(SimulationError::Corrupt("invalid traffic departure plan"));
            }
            let mut destinations = Vec::with_capacity(count);
            for _ in 0..count {
                destinations.push(decoder.u64()?);
            }
            EventKind::TrafficDeparturePlan {
                origin_system_id: entity_id,
                start_second,
                ordinal,
                total,
                destinations,
            }
        }
        _ => return Err(SimulationError::Corrupt("unknown scheduled event")),
    };
    decoder.finish()?;
    Ok(ScheduledEvent {
        event_id,
        due_second,
        entity_id,
        kind,
    })
}

impl QueuedSimulationEvent {
    pub fn encode(&self, bytes: &mut Vec<u8>) -> Result<(), SimulationError> {
        bytes.extend_from_slice(&self.event_id.to_be_bytes());
        bytes.extend_from_slice(&self.due_second.to_be_bytes());
        bytes.extend_from_slice(&self.entity_id.to_be_bytes());
        let length = u32::try_from(self.encoded_kind.len())
            .map_err(|_| SimulationError::Corrupt("scheduled event is too large"))?;
        bytes.extend_from_slice(&length.to_be_bytes());
        bytes.extend_from_slice(&self.encoded_kind);
        Ok(())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, SimulationError> {
        let mut decoder = Decoder::new(bytes);
        let event_id = decoder.u64()?;
        let due_second = decoder.u64()?;
        let entity_id = decoder.u64()?;
        let length = decoder.u32()? as usize;
        let encoded_kind = decoder.take(length)?.to_vec();
        decoder.finish()?;
        let queued = Self {
            event_id,
            due_second,
            entity_id,
            encoded_kind,
        };
        decode_queued_event(&queued)?;
        Ok(queued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortest_route_is_stable_and_uses_sorted_neighbors() {
        let systems = (1..=4)
            .map(|system_id| SimulationSystem {
                system_id,
                name: system_id.to_string(),
                position_parsecs: [0.0; 3],
                polity_id: 1,
                generation_seed: [system_id as u8; 32],
                population: 5,
                tech_level: 10,
                starport: 2,
                next_system_day: 0,
                jump_two_neighbors: match system_id {
                    1 => vec![2, 3],
                    2 | 3 => vec![1, 4],
                    4 => vec![2, 3],
                    _ => unreachable!(),
                },
            })
            .collect::<Vec<_>>();
        assert_eq!(shortest_route(&systems, 1, 4), Some(vec![1, 2, 4]));
    }

    #[test]
    fn spatial_buckets_preserve_exact_jump_two_neighbors() {
        let positions = [
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [-0.01, 0.0, 0.0],
            [2.000_001, 0.0, 0.0],
            [0.0, 1.2, 1.6],
        ];
        let systems = positions
            .into_iter()
            .enumerate()
            .map(|(index, position_parsecs)| SimulationSystem {
                system_id: index as u64 + 1,
                name: String::new(),
                position_parsecs,
                polity_id: 0,
                generation_seed: [0; 32],
                population: 0,
                tech_level: 0,
                starport: 5,
                next_system_day: 0,
                jump_two_neighbors: Vec::new(),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            jump_two_neighbor_lists(&systems),
            vec![vec![2, 3, 5], vec![1, 4], vec![1], vec![2], vec![1]]
        );
    }
}
