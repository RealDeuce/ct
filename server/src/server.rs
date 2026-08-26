//! Development TCP adapter and engine actor.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io;
use std::net::{Shutdown, SocketAddr, TcpStream as StdTcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use fluent_bundle::FluentArgs;
use socket2::{Domain, Protocol, Socket, Type};
use thiserror::Error;
#[cfg(test)]
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Semaphore, mpsc, oneshot, watch};
use tokio::task::JoinHandle;

use crate::engine::{BbsRegistry, LeagueRegistry};
use crate::engine::{Engine, EngineError};
use crate::i18n::{
    LanguageNegotiationError, LocalizationError, NegotiatedLanguage, SUPPORTED_LANGUAGE_TAGS,
    default_language, negotiate_language,
};
use crate::store::{
    BbsConfiguration, BbsCredential, BbsSettings, ConfigureBbsResult, Delivery, LeagueCredential,
    LeagueStatus, OperationalStatus, PlayerAccessRecord, PlayerAccessState, PlayerTravelTransition,
    SetLeagueMemberAccessResult, SetLeagueNameResult, SetPlayerAccessResult, StoreError,
    SysopDirectiveKind, SysopDirectiveRecord,
};
use crate::tls::{PskCredential, TlsServer};
use crate::traffic::{TrafficContact, TrafficSnapshot};
use crate::universe::UniverseInitialization;
use crate::web_push::{PushAlert, WebPushConfig, WebPushError, WebPushHandle, WebPushWorker};
use crate::wire::{
    CloseCode, MAX_FRAME_BYTES, PROTOCOL_VERSION, PlayerIdentity, WireError,
    decode_client_hello_with_version, decode_close, decode_protocol_version, decode_request,
    encode_checkpoint_ready, encode_close_with_code, encode_encounter_ready,
    encode_legacy_close_for_version, encode_phase_changed, encode_radio_unread, encode_response,
    encode_server_hello_with_affiliation, encode_server_stopping, encode_session_replaced,
    encode_traffic_movement, encode_traffic_snapshot,
};
use crate::{admin_wire, league_wire, sysop_wire, wire};

const CONNECTION_QUEUE_DEPTH: usize = 64;
const ENGINE_QUEUE_DEPTH: usize = 256;
const AUTHENTICATION_TIMEOUT: Duration = Duration::from_secs(10);
const LIVE_CLOCK_PULSE: Duration = Duration::from_secs(1);
// Scheduled events are durable transactions and some include substantial mail
// processing. Yield between them so live-clock catch-up cannot monopolize the
// authoritative thread ahead of an interactive request.
const LIVE_CLOCK_EVENT_QUANTUM: u64 = 1;
const MAX_PENDING_GAME_AUTHENTICATIONS: usize = 64;
const MAX_ACTIVE_GAME_SESSIONS: usize = 256;
const MAX_ACTIVE_GAME_SESSIONS_PER_BBS: usize = 64;

fn utc_timestamp(seconds: u64, milliseconds: u32) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(i64::MAX);
    let day_seconds = seconds % 86_400;
    // Convert days since the Unix epoch to a proleptic Gregorian date. This is
    // Howard Hinnant's civil-from-days algorithm with the Unix epoch offset.
    let shifted = days.saturating_add(719_468);
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let hour = day_seconds / 3_600;
    let minute = day_seconds % 3_600 / 60;
    let second = day_seconds % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milliseconds:03}Z")
}

pub fn log(arguments: fmt::Arguments<'_>) {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    eprintln!(
        "{} {arguments}",
        utc_timestamp(elapsed.as_secs(), elapsed.subsec_millis())
    );
}

macro_rules! server_log {
    ($($argument:tt)*) => {
        log(format_args!($($argument)*))
    };
}

fn localized_version_rejection(
    protocol: &str,
    client_version: u16,
    server_version: u16,
) -> Result<String, LocalizationError> {
    let mut arguments = FluentArgs::new();
    arguments.set("protocol", protocol);
    arguments.set("clientVersion", i64::from(client_version));
    arguments.set("serverVersion", i64::from(server_version));
    default_language().format("unsupported-version", Some(&arguments))
}

fn localized_error(
    language: &NegotiatedLanguage,
    key: &str,
    argument_name: &str,
    argument: &str,
) -> Result<String, LocalizationError> {
    let mut arguments = FluentArgs::new();
    arguments.set(argument_name, argument);
    language.format(key, Some(&arguments))
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("network I/O error: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    AdminWire(#[from] admin_wire::AdminWireError),
    #[error(transparent)]
    SysopWire(#[from] sysop_wire::SysopWireError),
    #[error(transparent)]
    LeagueWire(#[from] league_wire::LeagueWireError),
    #[error(transparent)]
    Localization(#[from] LocalizationError),
    #[error("authoritative engine stopped")]
    EngineStopped,
    #[error("authoritative engine failure: {0}")]
    Engine(String),
    #[error("TLS failure: {0}")]
    Tls(String),
    #[error("hello BBS identifier does not match the authenticated TLS PSK identity")]
    BbsIdentityMismatch,
    #[error("authenticated TLS PSK identity is not a canonical BBS identifier")]
    InvalidBbsPskIdentity,
    #[error("authenticated TLS PSK identity is not a canonical league identifier")]
    InvalidLeaguePskIdentity,
    #[error("TLS worker stopped unexpectedly")]
    TlsWorkerStopped,
    #[error("administrator listener must bind to a loopback address")]
    AdminNotLoopback,
    #[error("at least one {0} listener address is required")]
    NoListenerAddresses(&'static str),
    #[error("all network listener tasks stopped unexpectedly")]
    ListenerStopped,
    #[error("game connection limit reached")]
    ConnectionLimit,
    #[error(transparent)]
    WebPush(#[from] WebPushError),
}

#[derive(Clone)]
pub struct AdminTlsConfig {
    pub key: Arc<Vec<u8>>,
    pub backup_root: Arc<PathBuf>,
}

enum EngineMessage {
    ClockPulse {
        sampled_at: Instant,
    },
    OpenSession {
        identity: PlayerIdentity,
        reply: oneshot::Sender<Result<SessionOpening, String>>,
    },
    CloseSession {
        identity: PlayerIdentity,
        epoch: u64,
    },
    Submit {
        identity: PlayerIdentity,
        request: crate::wire::CommandRequest,
    },
    AcknowledgeOutbox(u64),
    AddBbs {
        command_id: [u8; wire::COMMAND_ID_BYTES],
        name: String,
        reply: oneshot::Sender<Result<BbsCredential, String>>,
    },
    AddLeague {
        command_id: [u8; wire::COMMAND_ID_BYTES],
        reply: oneshot::Sender<Result<LeagueCredential, String>>,
    },
    GetLeagueStatus {
        league_id: u32,
        reply: oneshot::Sender<Result<LeagueStatus, String>>,
    },
    SetLeagueName {
        league_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        expected_revision: u64,
        name: String,
        reply: oneshot::Sender<Result<SetLeagueNameResult, String>>,
    },
    AddLeagueBbs {
        league_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        name: String,
        reply: oneshot::Sender<Result<BbsCredential, String>>,
    },
    SetLeagueMemberEnabled {
        league_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        bbs_id: u32,
        expected_revision: u64,
        enabled: bool,
        reason: String,
        reply: oneshot::Sender<Result<SetLeagueMemberAccessResult, String>>,
    },
    InitializeUniverse {
        command_id: [u8; wire::COMMAND_ID_BYTES],
        reply: oneshot::Sender<Result<UniverseInitialization, String>>,
    },
    GetBbsConfiguration {
        bbs_id: u32,
        reply: oneshot::Sender<Result<BbsConfiguration, String>>,
    },
    ConfigureBbs {
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        expected_revision: u64,
        settings: BbsSettings,
        reply: oneshot::Sender<Result<ConfigureBbsResult, String>>,
    },
    GetPlayerAccess {
        identity: PlayerIdentity,
        reply: oneshot::Sender<Result<PlayerAccessRecord, String>>,
    },
    SetPlayerAccess {
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        player_id: u32,
        expected_revision: u64,
        state: PlayerAccessState,
        reason: String,
        reply: oneshot::Sender<Result<SetPlayerAccessResult, String>>,
    },
    GetStatus {
        reply: oneshot::Sender<Result<OperationalStatus, String>>,
    },
    LiveBackup {
        backup_root: PathBuf,
        label: String,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        reply: oneshot::Sender<Result<OperationalStatus, String>>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
    IssueSysopDirective {
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        player_id: u32,
        kind: SysopDirectiveKind,
        reply: oneshot::Sender<Result<SysopDirectiveRecord, String>>,
    },
}

enum EngineEvent {
    Delivery(Box<Delivery>),
    PhaseChanged(Box<PlayerTravelTransition>),
    CheckpointReady {
        identity: PlayerIdentity,
        committed_sequence: u64,
        checkpoint: Box<wire::CheckpointSnapshot>,
    },
    EncounterReady {
        identity: PlayerIdentity,
        committed_sequence: u64,
        encounter: Box<wire::EncounterSnapshot>,
    },
    TrafficSnapshot {
        identity: PlayerIdentity,
        committed_sequence: u64,
        snapshot: Box<TrafficSnapshot>,
    },
    TrafficMovement {
        identity: PlayerIdentity,
        committed_sequence: u64,
        system_id: u64,
        observed_second: u64,
        contact: Box<TrafficContact>,
    },
    RadioUnread {
        identity: PlayerIdentity,
        committed_sequence: u64,
        ship_id: u64,
        unread_count: u64,
    },
    UniverseReset,
    PlayerAccessChanged(Box<PlayerAccessRecord>),
    BbsAccessChanged {
        bbs_id: u32,
        enabled: bool,
        reason: String,
    },
    Fatal(String),
}

fn web_alert_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn shortened_alert_text(value: &str, maximum_characters: usize) -> String {
    let mut characters = value.chars();
    let mut result = characters
        .by_ref()
        .take(maximum_characters)
        .collect::<String>();
    if characters.next().is_some() {
        result.push('…');
    }
    result
}

fn checkpoint_push_alert(
    identity: PlayerIdentity,
    checkpoint: &wire::CheckpointSnapshot,
) -> PushAlert {
    let now = web_alert_now();
    let location = match checkpoint.locus {
        wire::FlightLocus::Port {
            system_id,
            world_id,
            facility_id,
        } => serde_json::json!({
            "type": "port",
            "systemId": system_id,
            "worldId": world_id,
            "facilityId": facility_id,
        }),
        wire::FlightLocus::JumpLocus { system_id } => {
            serde_json::json!({ "type": "jump locus", "systemId": system_id })
        }
        wire::FlightLocus::Body { system_id, body_id } => serde_json::json!({
            "type": "body",
            "systemId": system_id,
            "bodyId": body_id,
        }),
        wire::FlightLocus::DeepSpace { position } => {
            let [coreward, spinward, north] = position.parsecs();
            serde_json::json!({
                "type": "deep space",
                "corewardParsecs": coreward,
                "spinwardParsecs": spinward,
                "northParsecs": north,
            })
        }
    };
    let situation = match checkpoint.kind {
        wire::CheckpointKind::PortDeparture => "Departure preparations are complete.",
        wire::CheckpointKind::InhabitedWorld => "The ship has reached its destination.",
        wire::CheckpointKind::GasGiant => "Gas-giant operations have reached a checkpoint.",
        wire::CheckpointKind::JumpArrival => "The ship has arrived at its jump locus.",
        wire::CheckpointKind::JumpDeparture => "The ship is ready to enter jump space.",
        wire::CheckpointKind::DeepSpace => "The ship has emerged in deep space.",
    };
    PushAlert {
        identity,
        source_key: format!("checkpoint:{}", checkpoint.checkpoint_id),
        kind: "attention-now".into(),
        title: "Captain to the bridge!".into(),
        body: format!("{situation} The bridge is holding for your orders."),
        detail_json: serde_json::json!({
            "checkpointId": checkpoint.checkpoint_id,
            "kind": format!("{:?}", checkpoint.kind),
            "readyGameSecond": checkpoint.ready_second,
            "location": location,
        })
        .to_string(),
        created_unix_second: now,
        expires_unix_second: now.saturating_add(24 * 60 * 60),
        attention_due_unix_second: now,
    }
}

fn encounter_push_alert(
    identity: PlayerIdentity,
    encounter: &wire::EncounterSnapshot,
) -> PushAlert {
    let now = web_alert_now();
    let body = if encounter.kind == wire::EncounterKind::Hostile {
        let contact = if encounter.contact.ship_name.is_empty() {
            "An armed ship".into()
        } else {
            format!("The armed ship {}", encounter.contact.ship_name)
        };
        shortened_alert_text(
            &format!("{contact} is moving to intercept! {}", encounter.summary),
            240,
        )
    } else {
        shortened_alert_text(&format!("Encounter detected: {}", encounter.summary), 240)
    };
    PushAlert {
        identity,
        source_key: format!("encounter:{}", encounter.encounter_id),
        kind: "attention-now".into(),
        title: "Captain to the bridge!".into(),
        body,
        detail_json: serde_json::json!({
            "encounterId": encounter.encounter_id,
            "kind": format!("{:?}", encounter.kind),
            "summary": encounter.summary,
            "contact": {
                "shipName": encounter.contact.ship_name,
                "className": encounter.contact.class_name,
                "transponder": encounter.contact.transponder,
                "role": encounter.contact.role,
                "range": encounter.contact.range,
                "confidencePercent": encounter.contact.confidence_percent,
                "resolution": format!("{:?}", encounter.contact.resolution),
            },
            "authority": format!("{:?}", encounter.authority),
            "threat": format!("{:?}", encounter.threat),
            "demand": encounter.demand.text,
            "responseDeadlineGameSecond": encounter.response_deadline_second,
        })
        .to_string(),
        created_unix_second: now,
        expires_unix_second: now.saturating_add(24 * 60 * 60),
        attention_due_unix_second: now,
    }
}

fn upcoming_attention_push_alert(transition: &PlayerTravelTransition) -> Option<PushAlert> {
    let authority = transition.waypoint_authority_at_due?;
    if transition.status.due_second <= transition.status.current_game_second {
        return None;
    }
    let now = web_alert_now();
    let game_seconds = transition
        .status
        .due_second
        .saturating_sub(transition.status.current_game_second);
    let real_seconds = game_seconds.saturating_add(crate::clock::GAME_SECONDS_PER_RATE_PERIOD - 1)
        / crate::clock::GAME_SECONDS_PER_RATE_PERIOD;
    let due = now.saturating_add(real_seconds);
    let destination = if !transition.status.destination_system_name.is_empty() {
        transition.status.destination_system_name.clone()
    } else {
        match transition.status.destination {
            wire::FlightLocus::DeepSpace { .. } => "the plotted deep-space coordinates".into(),
            wire::FlightLocus::Body { body_id, .. } => format!("body {body_id}"),
            wire::FlightLocus::JumpLocus { system_id } => {
                format!("jump locus in system {system_id}")
            }
            wire::FlightLocus::Port { system_id, .. } => format!("port in system {system_id}"),
        }
    };
    let (kind, title, body) = match authority {
        wire::WaypointAuthority::Hold => (
            "attention-soon",
            "Bridge watch reminder",
            format!(
                "{} will reach {destination} and wait for the captain's orders.",
                transition.status.ship_name
            ),
        ),
        wire::WaypointAuthority::Through => (
            "automation-soon",
            "Standing orders reminder",
            format!(
                "{} will reach {destination}, where standing orders are filed to continue.",
                transition.status.ship_name
            ),
        ),
    };
    Some(PushAlert {
        identity: transition.identity.clone(),
        source_key: format!(
            "attention-due:{}:{}:{}:{}",
            transition.status.plan_id,
            transition.status.plan_revision,
            transition.status.leg_index,
            transition.status.due_second,
        ),
        kind: kind.into(),
        title: title.into(),
        body: shortened_alert_text(&body, 240),
        detail_json: serde_json::json!({
            "shipId": transition.status.ship_id,
            "shipName": transition.status.ship_name,
            "destinationSystemId": transition.status.destination_system_id,
            "destinationSystemName": transition.status.destination_system_name,
            "stage": format!("{:?}", transition.status.stage),
            "waypointAuthority": format!("{authority:?}"),
            "dueGameSecond": transition.status.due_second,
        })
        .to_string(),
        created_unix_second: now,
        expires_unix_second: due.saturating_add(60 * 60),
        attention_due_unix_second: due,
    })
}

async fn enqueue_web_alert(handle: &WebPushHandle, alert: PushAlert) {
    if !handle.configured() {
        return;
    }
    let handle = handle.clone();
    match tokio::task::spawn_blocking(move || handle.enqueue(alert)).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => server_log!("browser alert queue rejected an alert: {error}"),
        Err(error) => server_log!("browser alert queue task failed: {error}"),
    }
}

struct SessionOpening {
    epoch: u64,
    committed_sequence: u64,
    phase: wire::PlayerPhase,
    traffic_snapshot: Option<TrafficSnapshot>,
    checkpoint: Option<wire::CheckpointSnapshot>,
    encounter: Option<wire::EncounterSnapshot>,
    radio_ship_id: u64,
    radio_unread_count: u64,
    affiliation: Option<wire::InstitutionalAffiliation>,
}

struct Observer {
    epoch: u64,
    active_ship_id: u64,
    system_id: Option<u64>,
    last_second: u64,
    radio_unread_count: u64,
}

fn online_ship_ids(observers: &HashMap<PlayerIdentity, Observer>) -> HashSet<u64> {
    observers
        .values()
        .filter_map(|observer| (observer.active_ship_id != 0).then_some(observer.active_ship_id))
        .collect()
}

fn decorate_traffic_contact(contact: &mut TrafficContact, online: &HashSet<u64>) {
    contact.online_controlled = contact.player_owned && online.contains(&contact.contact_id);
}

fn decorate_traffic_snapshot(snapshot: &mut TrafficSnapshot, online: &HashSet<u64>) {
    for contact in &mut snapshot.contacts {
        decorate_traffic_contact(contact, online);
    }
}

fn decorate_delivery(delivery: &mut Delivery, online: &HashSet<u64>) {
    match &mut delivery.outcome.kind {
        wire::OutcomeKind::CombatCareer(snapshot) => {
            for contact in &mut snapshot.system_contacts {
                decorate_traffic_contact(contact, online);
            }
            for contact in &mut snapshot.local_contacts {
                decorate_traffic_contact(contact, online);
            }
        }
        wire::OutcomeKind::Fleet(snapshot) => {
            for ship in &mut snapshot.ships {
                ship.online_controlled = online.contains(&ship.ship_id);
            }
        }
        wire::OutcomeKind::Combat(snapshot) => {
            for participant in &mut snapshot.participants {
                participant.online_controlled =
                    participant.player_owned && online.contains(&participant.vessel_id);
            }
        }
        _ => {}
    }
}

fn emit_best_effort(sender: &mpsc::Sender<EngineEvent>, event: EngineEvent) -> bool {
    match sender.try_send(event) {
        Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => true,
        Err(mpsc::error::TrySendError::Closed(_)) => false,
    }
}

async fn shutdown_signal() -> io::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await
    }
}

fn emit_observer_movements(
    engine: &Engine,
    sender: &mpsc::Sender<EngineEvent>,
    identity: &PlayerIdentity,
    observer: &mut Observer,
    through_second: u64,
) -> Result<bool, EngineError> {
    let Some(system_id) = observer.system_id else {
        observer.last_second = through_second;
        return Ok(true);
    };
    let sequence = engine.committed_sequence()?;
    for contact in engine.traffic_movements(system_id, observer.last_second, through_second)? {
        if !emit_best_effort(
            sender,
            EngineEvent::TrafficMovement {
                identity: identity.clone(),
                committed_sequence: sequence,
                system_id,
                observed_second: contact.edge_second,
                contact: Box::new(contact),
            },
        ) {
            return Ok(false);
        }
    }
    observer.last_second = through_second;
    Ok(true)
}

fn emit_radio_unread_updates(
    engine: &Engine,
    sender: &mpsc::Sender<EngineEvent>,
    observers: &mut HashMap<PlayerIdentity, Observer>,
) -> Result<bool, EngineError> {
    let identities = observers.keys().cloned().collect::<Vec<_>>();
    let sequence = engine.committed_sequence()?;
    for identity in identities {
        let Ok((ship_id, unread_count)) = engine.radio_unread_count(&identity) else {
            continue;
        };
        let Some(observer) = observers.get_mut(&identity) else {
            continue;
        };
        if unread_count == observer.radio_unread_count {
            continue;
        }
        observer.radio_unread_count = unread_count;
        if !emit_best_effort(
            sender,
            EngineEvent::RadioUnread {
                identity,
                committed_sequence: sequence,
                ship_id,
                unread_count,
            },
        ) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn emit_transition(
    engine: &Engine,
    sender: &mpsc::Sender<EngineEvent>,
    observers: &mut HashMap<PlayerIdentity, Observer>,
    transition: PlayerTravelTransition,
) -> Result<bool, EngineError> {
    if let Some(observer) = observers.get_mut(&transition.identity)
        && !emit_observer_movements(
            engine,
            sender,
            &transition.identity,
            observer,
            transition.status.current_game_second,
        )?
    {
        return Ok(false);
    }
    if sender
        .blocking_send(EngineEvent::PhaseChanged(Box::new(transition.clone())))
        .is_err()
    {
        return Ok(false);
    }
    if let Some(checkpoint) = engine.pending_checkpoint(&transition.identity)?
        && sender
            .blocking_send(EngineEvent::CheckpointReady {
                identity: transition.identity.clone(),
                committed_sequence: transition.committed_sequence,
                checkpoint: Box::new(checkpoint),
            })
            .is_err()
    {
        return Ok(false);
    }
    if let Some(encounter) = engine.pending_encounter(&transition.identity)?
        && sender
            .blocking_send(EngineEvent::EncounterReady {
                identity: transition.identity.clone(),
                committed_sequence: transition.committed_sequence,
                encounter: Box::new(encounter),
            })
            .is_err()
    {
        return Ok(false);
    }
    if let Some(observer) = observers.get_mut(&transition.identity) {
        observer.active_ship_id = engine.active_ship_id(&transition.identity)?.unwrap_or(0);
        let old_system = observer.system_id;
        observer.system_id = match transition.phase {
            wire::PlayerPhase::Docked | wire::PlayerPhase::Interplanetary => {
                Some(transition.status.current_system_id)
            }
            wire::PlayerPhase::Jump | wire::PlayerPhase::NewUser | wire::PlayerPhase::Terminal => {
                None
            }
            wire::PlayerPhase::Encounter => Some(transition.status.current_system_id),
        };
        observer.last_second = transition.status.current_game_second;
        if observer.system_id.is_some()
            && observer.system_id != old_system
            && let Some(mut snapshot) = engine.traffic_snapshot(&transition.identity)?
            && !emit_best_effort(
                sender,
                EngineEvent::TrafficSnapshot {
                    identity: transition.identity,
                    committed_sequence: transition.committed_sequence,
                    snapshot: Box::new({
                        decorate_traffic_snapshot(&mut snapshot, &online_ship_ids(observers));
                        snapshot
                    }),
                },
            )
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn emit_advance(
    engine: &Engine,
    sender: &mpsc::Sender<EngineEvent>,
    observers: &mut HashMap<PlayerIdentity, Observer>,
    advance: crate::store::SimulationAdvance,
) -> Result<bool, EngineError> {
    for transition in advance.player_transitions {
        if !emit_transition(engine, sender, observers, transition)? {
            return Ok(false);
        }
    }
    for (identity, observer) in observers.iter_mut() {
        if !emit_observer_movements(engine, sender, identity, observer, advance.ending_second)? {
            return Ok(false);
        }
    }
    emit_radio_unread_updates(engine, sender, observers)
}

#[derive(Clone)]
struct EngineHandle {
    sender: mpsc::Sender<EngineMessage>,
}

impl EngineHandle {
    async fn issue_session(&self, identity: PlayerIdentity) -> Result<SessionOpening, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::OpenSession { identity, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn close_session(&self, identity: PlayerIdentity, epoch: u64) {
        let _ = self
            .sender
            .send(EngineMessage::CloseSession { identity, epoch })
            .await;
    }

    async fn add_bbs(
        &self,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        name: String,
    ) -> Result<BbsCredential, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::AddBbs {
                command_id,
                name,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn add_league(
        &self,
        command_id: [u8; wire::COMMAND_ID_BYTES],
    ) -> Result<LeagueCredential, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::AddLeague { command_id, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn league_status(&self, league_id: u32) -> Result<LeagueStatus, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::GetLeagueStatus { league_id, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn set_league_name(
        &self,
        league_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        expected_revision: u64,
        name: String,
    ) -> Result<SetLeagueNameResult, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::SetLeagueName {
                league_id,
                command_id,
                expected_revision,
                name,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn add_league_bbs(
        &self,
        league_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        name: String,
    ) -> Result<BbsCredential, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::AddLeagueBbs {
                league_id,
                command_id,
                name,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn set_league_member_enabled(
        &self,
        league_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        bbs_id: u32,
        expected_revision: u64,
        enabled: bool,
        reason: String,
    ) -> Result<SetLeagueMemberAccessResult, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::SetLeagueMemberEnabled {
                league_id,
                command_id,
                bbs_id,
                expected_revision,
                enabled,
                reason,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn initialize_universe(
        &self,
        command_id: [u8; wire::COMMAND_ID_BYTES],
    ) -> Result<UniverseInitialization, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::InitializeUniverse { command_id, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn bbs_configuration(&self, bbs_id: u32) -> Result<BbsConfiguration, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::GetBbsConfiguration { bbs_id, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn configure_bbs(
        &self,
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        expected_revision: u64,
        settings: BbsSettings,
    ) -> Result<ConfigureBbsResult, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::ConfigureBbs {
                bbs_id,
                command_id,
                expected_revision,
                settings,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn player_access(
        &self,
        identity: PlayerIdentity,
    ) -> Result<PlayerAccessRecord, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::GetPlayerAccess { identity, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn set_player_access(
        &self,
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        player_id: u32,
        expected_revision: u64,
        state: PlayerAccessState,
        reason: String,
    ) -> Result<SetPlayerAccessResult, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::SetPlayerAccess {
                bbs_id,
                command_id,
                player_id,
                expected_revision,
                state,
                reason,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn status(&self) -> Result<OperationalStatus, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::GetStatus { reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn live_backup(
        &self,
        backup_root: PathBuf,
        label: String,
        command_id: [u8; wire::COMMAND_ID_BYTES],
    ) -> Result<OperationalStatus, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::LiveBackup {
                backup_root,
                label,
                command_id,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn shutdown(&self) {
        let (reply, receiver) = oneshot::channel();
        if self
            .sender
            .send(EngineMessage::Shutdown { reply })
            .await
            .is_ok()
        {
            let _ = receiver.await;
        }
    }

    async fn issue_sysop_directive(
        &self,
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        player_id: u32,
        kind: SysopDirectiveKind,
    ) -> Result<SysopDirectiveRecord, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::IssueSysopDirective {
                bbs_id,
                command_id,
                player_id,
                kind,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }
}

fn spawn_engine(
    data_path: PathBuf,
    bbs_registry: BbsRegistry,
    league_registry: LeagueRegistry,
) -> (
    EngineHandle,
    mpsc::Receiver<EngineEvent>,
    thread::JoinHandle<()>,
    std::sync::mpsc::Receiver<Result<(), String>>,
) {
    let (input_sender, mut input_receiver) = mpsc::channel(ENGINE_QUEUE_DEPTH);
    let (event_sender, event_receiver) = mpsc::channel(ENGINE_QUEUE_DEPTH);
    let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
    let join = thread::Builder::new()
        .name("ct-authoritative-engine".into())
        .spawn(move || {
            let mut ready_sender = Some(ready_sender);
            let run = || -> Result<(), EngineError> {
                let engine =
                    Engine::open_with_registries(data_path, bbs_registry, league_registry)?;
                let recovered = engine.recover()?;
                let mut live_clock = crate::clock::LiveClock::now(engine.game_second()?);
                let mut observers = HashMap::<PlayerIdentity, Observer>::new();
                let mut last_lag_report = Instant::now() - Duration::from_secs(60);
                let mut pending_clock_target = None::<u64>;
                if let Some(sender) = ready_sender.take() {
                    let _ = sender.send(Ok(()));
                }
                for transition in recovered.player_transitions {
                    if !emit_transition(&engine, &event_sender, &mut observers, transition)? {
                        return Ok(());
                    }
                }
                for delivery in engine.pending_outbox()? {
                    if event_sender
                        .blocking_send(EngineEvent::Delivery(Box::new(delivery)))
                        .is_err()
                    {
                        return Ok(());
                    }
                }
                loop {
                    // Foreground work already in the queue takes precedence.
                    // When it is empty, keep advancing toward the most recent
                    // sampled target without waiting for another clock pulse.
                    let message = match input_receiver.try_recv() {
                        Ok(message) => message,
                        Err(mpsc::error::TryRecvError::Empty) => {
                            if let Some(target) = pending_clock_target {
                                let advance = engine.advance_simulation_toward(
                                    target,
                                    LIVE_CLOCK_EVENT_QUANTUM,
                                )?;
                                if advance.ending_second >= target {
                                    pending_clock_target = None;
                                } else if last_lag_report.elapsed() >= Duration::from_secs(60) {
                                    server_log!(
                                        "live clock lag: target={} committed={} lag={}m events={} wall={}ns cpu={}ns",
                                        target,
                                        advance.ending_second,
                                        target - advance.ending_second,
                                        advance.processed_events,
                                        advance.wall_nanoseconds,
                                        advance.thread_cpu_nanoseconds.unwrap_or(0),
                                    );
                                    last_lag_report = Instant::now();
                                }
                                if !emit_advance(
                                    &engine,
                                    &event_sender,
                                    &mut observers,
                                    advance,
                                )? {
                                    return Ok(());
                                }
                                continue;
                            }
                            let Some(message) = input_receiver.blocking_recv() else {
                                break;
                            };
                            message
                        }
                        Err(mpsc::error::TryRecvError::Disconnected) => break,
                    };
                    match message {
                        EngineMessage::ClockPulse { sampled_at } => {
                            let target = live_clock.target_second(sampled_at);
                            pending_clock_target = Some(
                                pending_clock_target
                                    .map_or(target, |pending| pending.max(target)),
                            );
                        }
                        EngineMessage::OpenSession { identity, reply } => {
                            match engine.issue_session(&identity) {
                                Ok((epoch, committed_sequence, phase)) => {
                                    let mut traffic_snapshot = engine.traffic_snapshot(&identity)?;
                                    let current_second = engine.game_second()?;
                                    let (radio_ship_id, radio_unread_count) = engine
                                        .radio_unread_count(&identity)
                                        .unwrap_or((0, 0));
                                    let active_ship_id = engine.active_ship_id(&identity)?.unwrap_or(0);
                                    observers.insert(
                                        identity.clone(),
                                        Observer {
                                            epoch,
                                            active_ship_id,
                                            system_id: traffic_snapshot
                                                .as_ref()
                                                .map(|snapshot| snapshot.system_id),
                                            last_second: current_second,
                                            radio_unread_count,
                                        },
                                    );
                                    if let Some(snapshot) = &mut traffic_snapshot {
                                        decorate_traffic_snapshot(snapshot, &online_ship_ids(&observers));
                                    }
                                    let _ = reply.send(Ok(SessionOpening {
                                        epoch,
                                        committed_sequence,
                                        phase,
                                        traffic_snapshot,
                                        checkpoint: engine.pending_checkpoint(&identity)?,
                                        encounter: engine.pending_encounter(&identity)?,
                                        radio_ship_id,
                                        radio_unread_count,
                                        affiliation: engine.home_affiliation(identity.bbs_id)?,
                                    }));
                                }
                                Err(error @ EngineError::PlayerAccessDenied(_)) => {
                                    let _ = reply.send(Err(error.to_string()));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::CloseSession { identity, epoch } => {
                            if observers
                                .get(&identity)
                                .is_some_and(|observer| observer.epoch == epoch)
                            {
                                observers.remove(&identity);
                            }
                        }
                        EngineMessage::Submit { identity, request } => {
                            let mut batch = engine.submit(identity.clone(), request)?;
                            if let Some(observer) = observers.get_mut(&identity) {
                                observer.active_ship_id =
                                    engine.active_ship_id(&identity)?.unwrap_or(0);
                            }
                            let online = online_ship_ids(&observers);
                            for delivery in &mut batch.deliveries {
                                decorate_delivery(delivery, &online);
                            }
                            for delivery in batch.deliveries {
                                if event_sender
                                    .blocking_send(EngineEvent::Delivery(Box::new(delivery)))
                                    .is_err()
                                {
                                    return Ok(());
                                }
                            }
                            for transition in batch.player_transitions {
                                if !emit_transition(
                                    &engine,
                                    &event_sender,
                                    &mut observers,
                                    transition,
                                )? {
                                    return Ok(());
                                }
                            }
                            if let Some(observer) = observers.get_mut(&identity) {
                                let snapshot = engine.traffic_snapshot(&identity)?;
                                let new_system = snapshot.as_ref().map(|value| value.system_id);
                                if new_system != observer.system_id {
                                    observer.system_id = new_system;
                                    observer.last_second = engine.game_second()?;
                                    if let Some(mut snapshot) = snapshot
                                        && !emit_best_effort(
                                            &event_sender,
                                            EngineEvent::TrafficSnapshot {
                                                identity,
                                                committed_sequence: engine.committed_sequence()?,
                                                snapshot: Box::new({
                                                    decorate_traffic_snapshot(&mut snapshot, &online);
                                                    snapshot
                                                }),
                                            },
                                        )
                                    {
                                        return Ok(());
                                    }
                                }
                            }
                            if !emit_radio_unread_updates(
                                &engine,
                                &event_sender,
                                &mut observers,
                            )? {
                                return Ok(());
                            }
                        }
                        EngineMessage::AcknowledgeOutbox(sequence) => {
                            engine.acknowledge_outbox(sequence)?;
                        }
                        EngineMessage::AddBbs {
                            command_id,
                            name,
                            reply,
                        } => match engine.add_bbs(command_id, &name) {
                            Ok(credential) => {
                                let _ = reply.send(Ok(credential));
                            }
                            Err(error @ EngineError::Store(StoreError::UniverseNotInitialized)) => {
                                let _ = reply.send(Err(error.to_string()));
                            }
                            Err(error) => {
                                let _ = reply.send(Err(error.to_string()));
                                return Err(error);
                            }
                        },
                        EngineMessage::AddLeague { command_id, reply } => {
                            match engine.add_league(command_id) {
                                Ok(credential) => {
                                    let _ = reply.send(Ok(credential));
                                }
                                Err(error @ EngineError::Store(
                                    StoreError::UniverseNotInitialized,
                                )) => {
                                    let _ = reply.send(Err(error.to_string()));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::GetLeagueStatus { league_id, reply } => {
                            let result = engine.league_status(league_id).map_err(|error| error.to_string());
                            let _ = reply.send(result);
                        }
                        EngineMessage::SetLeagueName {
                            league_id,
                            command_id,
                            expected_revision,
                            name,
                            reply,
                        } => {
                            let result = engine
                                .set_league_name(league_id, command_id, expected_revision, &name)
                                .map_err(|error| error.to_string());
                            let _ = reply.send(result);
                        }
                        EngineMessage::AddLeagueBbs {
                            league_id,
                            command_id,
                            name,
                            reply,
                        } => {
                            let result = engine
                                .add_league_bbs(league_id, command_id, &name)
                                .map_err(|error| error.to_string());
                            let _ = reply.send(result);
                        }
                        EngineMessage::SetLeagueMemberEnabled {
                            league_id,
                            command_id,
                            bbs_id,
                            expected_revision,
                            enabled,
                            reason,
                            reply,
                        } => {
                            let result = engine.set_league_member_enabled(
                                league_id,
                                command_id,
                                bbs_id,
                                expected_revision,
                                enabled,
                                &reason,
                            );
                            if matches!(result, Ok(SetLeagueMemberAccessResult::Updated(_)))
                                && event_sender
                                    .blocking_send(EngineEvent::BbsAccessChanged {
                                        bbs_id,
                                        enabled,
                                        reason: reason.clone(),
                                    })
                                    .is_err()
                            {
                                return Ok(());
                            }
                            let _ = reply.send(result.map_err(|error| error.to_string()));
                        }
                        EngineMessage::InitializeUniverse { command_id, reply } => {
                            match engine.initialize_universe(command_id) {
                                Ok(initialization) => {
                                    live_clock.reanchor(engine.game_second()?, Instant::now());
                                    pending_clock_target = None;
                                    observers.clear();
                                    if event_sender
                                        .blocking_send(EngineEvent::UniverseReset)
                                        .is_err()
                                    {
                                        return Ok(());
                                    }
                                    let _ = reply.send(Ok(initialization));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::GetBbsConfiguration { bbs_id, reply } => {
                            match engine.bbs_configuration(bbs_id) {
                                Ok(configuration) => {
                                    let _ = reply.send(Ok(configuration));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::ConfigureBbs {
                            bbs_id,
                            command_id,
                            expected_revision,
                            settings,
                            reply,
                        } => {
                            match engine.configure_bbs(
                                bbs_id,
                                command_id,
                                expected_revision,
                                &settings,
                            ) {
                                Ok(configuration) => {
                                    let _ = reply.send(Ok(configuration));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::GetPlayerAccess { identity, reply } => {
                            match engine.player_access(&identity) {
                                Ok(access) => {
                                    let _ = reply.send(Ok(access));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::SetPlayerAccess {
                            bbs_id,
                            command_id,
                            player_id,
                            expected_revision,
                            state,
                            reason,
                            reply,
                        } => {
                            match engine.set_player_access(
                                bbs_id,
                                command_id,
                                player_id,
                                expected_revision,
                                state,
                                &reason,
                            ) {
                                Ok(result) => {
                                    if let SetPlayerAccessResult::Updated(access) = &result
                                        && access.state != PlayerAccessState::Active
                                        && event_sender
                                            .blocking_send(EngineEvent::PlayerAccessChanged(
                                                Box::new(access.clone()),
                                            ))
                                            .is_err()
                                    {
                                        return Ok(());
                                    }
                                    let _ = reply.send(Ok(result));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::GetStatus { reply } => {
                            match engine.operational_status() {
                                Ok(status) => {
                                    let _ = reply.send(Ok(status));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::LiveBackup {
                            backup_root,
                            label,
                            command_id,
                            reply,
                        } => {
                            match engine.live_backup(&backup_root, &label, &command_id) {
                                Ok(status) => {
                                    let _ = reply.send(Ok(status));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                }
                            }
                        }
                        EngineMessage::Shutdown { reply } => {
                            let _ = reply.send(());
                            return Ok(());
                        }
                        EngineMessage::IssueSysopDirective {
                            bbs_id,
                            command_id,
                            player_id,
                            kind,
                            reply,
                        } => {
                            match engine.issue_sysop_directive(
                                bbs_id,
                                command_id,
                                player_id,
                                kind,
                            ) {
                                Ok(directive) => {
                                    let _ = reply.send(Ok(directive));
                                }
                                Err(error @ EngineError::Store(
                                    StoreError::InvalidSysopDirective(_),
                                )) => {
                                    let _ = reply.send(Err(error.to_string()));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                    }
                }
                Ok(())
            };
            if let Err(error) = run() {
                if let Some(sender) = ready_sender.take() {
                    let _ = sender.send(Err(error.to_string()));
                }
                let _ = event_sender.blocking_send(EngineEvent::Fatal(error.to_string()));
            }
        })
        .expect("spawn authoritative engine thread");
    (
        EngineHandle {
            sender: input_sender,
        },
        event_receiver,
        join,
        ready_receiver,
    )
}

#[derive(Clone)]
struct ActiveSession {
    epoch: u64,
    outbound: mpsc::Sender<Vec<u8>>,
    replaced: watch::Sender<bool>,
    socket: Option<Arc<StdTcpStream>>,
    language: NegotiatedLanguage,
}

#[derive(Default)]
struct Sessions {
    players: Mutex<HashMap<PlayerIdentity, ActiveSession>>,
}

impl Sessions {
    async fn active_count(&self) -> u32 {
        u32::try_from(self.players.lock().await.len()).unwrap_or(u32::MAX)
    }

    async fn replace_limited(
        &self,
        identity: PlayerIdentity,
        session: ActiveSession,
    ) -> Result<Option<ActiveSession>, ServerError> {
        let mut players = self.players.lock().await;
        if !players.contains_key(&identity) {
            if players.len() >= MAX_ACTIVE_GAME_SESSIONS
                || players
                    .keys()
                    .filter(|active| active.bbs_id == identity.bbs_id)
                    .count()
                    >= MAX_ACTIVE_GAME_SESSIONS_PER_BBS
            {
                return Err(ServerError::ConnectionLimit);
            }
        }
        Ok(players.insert(identity, session))
    }

    #[cfg(test)]
    async fn replace(
        &self,
        identity: PlayerIdentity,
        session: ActiveSession,
    ) -> Option<ActiveSession> {
        self.players.lock().await.insert(identity, session)
    }

    async fn remove_if_current(&self, identity: &PlayerIdentity, epoch: u64) {
        let mut players = self.players.lock().await;
        if players
            .get(identity)
            .is_some_and(|session| session.epoch == epoch)
        {
            players.remove(identity);
        }
    }

    async fn deliver(&self, delivery: &Delivery) -> DeliveryDisposition {
        let session = {
            let players = self.players.lock().await;
            let Some(session) = players.get(&delivery.identity) else {
                return DeliveryDisposition::NoSession;
            };
            session.clone()
        };
        if session.epoch > delivery.session_epoch {
            return DeliveryDisposition::Obsolete;
        }
        if session.epoch != delivery.session_epoch {
            return DeliveryDisposition::NoSession;
        }
        let frame = match encode_response(
            delivery.request_id,
            delivery.session_epoch,
            &delivery.outcome,
        ) {
            Ok(frame) => frame,
            Err(_) => return DeliveryDisposition::NoSession,
        };
        if session.outbound.send(frame).await.is_ok() {
            DeliveryDisposition::Enqueued
        } else {
            DeliveryDisposition::NoSession
        }
    }

    async fn phase_changed(&self, transition: &PlayerTravelTransition) {
        let session = {
            let players = self.players.lock().await;
            players.get(&transition.identity).cloned()
        };
        let Some(session) = session else {
            return;
        };
        let Ok(frame) = encode_phase_changed(session.epoch, transition) else {
            return;
        };
        if session.outbound.try_send(frame).is_err() {
            let _ = session.replaced.send(true);
            if let Some(socket) = session.socket {
                let _ = socket.shutdown(Shutdown::Both);
            }
            self.remove_if_current(&transition.identity, session.epoch)
                .await;
        }
    }

    async fn checkpoint_ready(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        checkpoint: &wire::CheckpointSnapshot,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        if let Some(session) = session {
            if let Ok(frame) =
                encode_checkpoint_ready(session.epoch, committed_sequence, checkpoint)
            {
                let _ = session.outbound.try_send(frame);
            }
        }
    }
    async fn encounter_ready(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        encounter: &wire::EncounterSnapshot,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        if let Some(session) = session {
            if let Ok(frame) = encode_encounter_ready(session.epoch, committed_sequence, encounter)
            {
                let _ = session.outbound.try_send(frame);
            }
        }
    }

    async fn traffic_snapshot(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        snapshot: &TrafficSnapshot,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        let Some(session) = session else {
            return;
        };
        if let Ok(frame) = encode_traffic_snapshot(session.epoch, committed_sequence, snapshot) {
            let _ = session.outbound.try_send(frame);
        }
    }

    async fn traffic_movement(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        system_id: u64,
        observed_second: u64,
        contact: &TrafficContact,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        let Some(session) = session else {
            return;
        };
        if let Ok(frame) = encode_traffic_movement(
            session.epoch,
            committed_sequence,
            system_id,
            observed_second,
            contact,
        ) {
            let _ = session.outbound.try_send(frame);
        }
    }

    async fn radio_unread(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        ship_id: u64,
        unread_count: u64,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        let Some(session) = session else {
            return;
        };
        if let Ok(frame) =
            encode_radio_unread(session.epoch, committed_sequence, ship_id, unread_count)
        {
            let _ = session.outbound.try_send(frame);
        }
    }

    async fn close_all_for_universe_reset(&self) {
        let sessions = {
            let mut players = self.players.lock().await;
            players
                .drain()
                .map(|(_, session)| session)
                .collect::<Vec<_>>()
        };
        for session in sessions {
            let _ = session.replaced.send(true);
            if let Some(socket) = session.socket {
                let _ = socket.shutdown(Shutdown::Both);
            }
        }
    }

    async fn close_all_for_shutdown(&self) {
        let sessions = {
            let mut players = self.players.lock().await;
            players
                .drain()
                .map(|(_, session)| session)
                .collect::<Vec<_>>()
        };
        for session in sessions {
            if let Ok(frame) = encode_server_stopping(session.epoch) {
                let _ = session.outbound.send(frame).await;
            }
            let _ = session.replaced.send(true);
        }
    }

    async fn close_for_access_change(&self, access: &PlayerAccessRecord) {
        let session = self.players.lock().await.remove(&access.identity);
        let Some(session) = session else {
            return;
        };
        let state = match access.state {
            PlayerAccessState::Active => "active",
            PlayerAccessState::Suspended => "suspended",
            PlayerAccessState::Removed => "permanently removed",
        };
        let reason = if access.reason.is_empty() {
            format!("player access {state}")
        } else {
            format!("player access {state}: {}", access.reason)
        };
        let reason = localized_error(&session.language, "access-denied", "reason", &reason)
            .unwrap_or(reason);
        if let Ok(frame) =
            encode_close_with_code(session.epoch, CloseCode::AccessDenied, &reason, &[])
        {
            let _ = session.outbound.try_send(frame);
        }
        let _ = session.replaced.send(true);
        if let Some(socket) = session.socket {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }

    async fn close_bbs(&self, bbs_id: u32, disable_reason: &str) {
        let sessions = {
            let mut players = self.players.lock().await;
            let identities = players
                .keys()
                .filter(|identity| identity.bbs_id == bbs_id)
                .cloned()
                .collect::<Vec<_>>();
            identities
                .into_iter()
                .filter_map(|identity| players.remove(&identity))
                .collect::<Vec<_>>()
        };
        for session in sessions {
            let reason = if disable_reason.is_empty() {
                "home BBS access is disabled".to_owned()
            } else {
                format!("home BBS access is disabled: {disable_reason}")
            };
            if let Ok(frame) =
                encode_close_with_code(session.epoch, CloseCode::AccessDenied, &reason, &[])
            {
                let _ = session.outbound.try_send(frame);
            }
            let _ = session.replaced.send(true);
            if let Some(socket) = session.socket {
                let _ = socket.shutdown(Shutdown::Both);
            }
        }
    }
}

#[derive(Default)]
struct BbsControlConnections {
    next_id: AtomicU64,
    sockets: StdMutex<HashMap<u32, HashMap<u64, Arc<StdTcpStream>>>>,
}

impl BbsControlConnections {
    fn register(
        self: &Arc<Self>,
        bbs_id: u32,
        socket: Arc<StdTcpStream>,
    ) -> BbsControlRegistration {
        let connection_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.sockets
            .lock()
            .expect("BBS control connection lock poisoned")
            .entry(bbs_id)
            .or_default()
            .insert(connection_id, socket);
        BbsControlRegistration {
            owner: Arc::clone(self),
            bbs_id,
            connection_id,
        }
    }

    fn close_bbs(&self, bbs_id: u32) {
        let sockets = self
            .sockets
            .lock()
            .expect("BBS control connection lock poisoned")
            .remove(&bbs_id)
            .unwrap_or_default();
        for socket in sockets.into_values() {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}

struct BbsControlRegistration {
    owner: Arc<BbsControlConnections>,
    bbs_id: u32,
    connection_id: u64,
}

impl Drop for BbsControlRegistration {
    fn drop(&mut self) {
        let mut sockets = self
            .owner
            .sockets
            .lock()
            .expect("BBS control connection lock poisoned");
        if let Some(entries) = sockets.get_mut(&self.bbs_id) {
            entries.remove(&self.connection_id);
            if entries.is_empty() {
                sockets.remove(&self.bbs_id);
            }
        }
    }
}

enum DeliveryDisposition {
    Enqueued,
    Obsolete,
    NoSession,
}

#[derive(Clone, Copy)]
enum ListenerRole {
    Game,
    Administrator,
    Sysop,
    League,
}

enum AcceptedConnection {
    Game(TcpStream, SocketAddr),
    Administrator(TcpStream, SocketAddr),
    Sysop(TcpStream, SocketAddr),
    League(TcpStream, SocketAddr),
}

struct AcceptTasks(Vec<JoinHandle<()>>);

impl Drop for AcceptTasks {
    fn drop(&mut self) {
        for task in &self.0 {
            task.abort();
        }
    }
}

fn bind_listener(address: SocketAddr) -> io::Result<TcpListener> {
    let socket = Socket::new(
        Domain::for_address(address),
        Type::STREAM,
        Some(Protocol::TCP),
    )?;
    if address.is_ipv6() {
        socket.set_only_v6(true)?;
    }
    socket.bind(&address.into())?;
    socket.listen(1024)?;
    socket.set_nonblocking(true)?;
    TcpListener::from_std(socket.into())
}

fn bind_listeners(
    addresses: &[SocketAddr],
    description: &'static str,
) -> Result<Vec<TcpListener>, ServerError> {
    if addresses.is_empty() {
        return Err(ServerError::NoListenerAddresses(description));
    }
    addresses
        .iter()
        .copied()
        .map(bind_listener)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ServerError::from)
}

fn listener_address_list(listeners: &[TcpListener]) -> io::Result<String> {
    listeners
        .iter()
        .map(|listener| listener.local_addr().map(|address| address.to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map(|addresses| addresses.join(", "))
}

fn spawn_accept_tasks(
    listeners: Vec<TcpListener>,
    role: ListenerRole,
    sender: &mpsc::Sender<Result<AcceptedConnection, io::Error>>,
    tasks: &mut Vec<JoinHandle<()>>,
) {
    for listener in listeners {
        let sender = sender.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                let result = listener.accept().await.map(|(socket, peer)| match role {
                    ListenerRole::Game => AcceptedConnection::Game(socket, peer),
                    ListenerRole::Administrator => AcceptedConnection::Administrator(socket, peer),
                    ListenerRole::Sysop => AcceptedConnection::Sysop(socket, peer),
                    ListenerRole::League => AcceptedConnection::League(socket, peer),
                });
                let failed = result.is_err();
                if sender.send(result).await.is_err() || failed {
                    break;
                }
            }
        }));
    }
}

pub async fn run(
    game_address: SocketAddr,
    admin_address: SocketAddr,
    sysop_address: SocketAddr,
    data_path: PathBuf,
    admin_tls: AdminTlsConfig,
) -> Result<(), ServerError> {
    run_on_addresses(
        vec![game_address],
        vec![admin_address],
        vec![sysop_address],
        vec![SocketAddr::new(sysop_address.ip(), 7326)],
        data_path,
        admin_tls,
        None,
    )
    .await
}

pub async fn run_on_addresses(
    game_addresses: Vec<SocketAddr>,
    admin_addresses: Vec<SocketAddr>,
    sysop_addresses: Vec<SocketAddr>,
    league_addresses: Vec<SocketAddr>,
    data_path: PathBuf,
    admin_tls: AdminTlsConfig,
    web_push_config: Option<WebPushConfig>,
) -> Result<(), ServerError> {
    if admin_addresses
        .iter()
        .any(|address| !address.ip().is_loopback())
    {
        return Err(ServerError::AdminNotLoopback);
    }
    let game_listeners = bind_listeners(&game_addresses, "game")?;
    let admin_listeners = bind_listeners(&admin_addresses, "administrator")?;
    let sysop_listeners = bind_listeners(&sysop_addresses, "sysop")?;
    let league_listeners = bind_listeners(&league_addresses, "league coordinator")?;
    let game_listener_text = listener_address_list(&game_listeners)?;
    let admin_listener_text = listener_address_list(&admin_listeners)?;
    let sysop_listener_text = listener_address_list(&sysop_listeners)?;
    let league_listener_text = listener_address_list(&league_listeners)?;
    let bbs_registry = BbsRegistry::default();
    let league_registry = LeagueRegistry::default();
    let (web_push, _web_push_worker) = if let Some(config) = web_push_config {
        let (handle, worker) = WebPushWorker::spawn(config)?;
        (handle, Some(worker))
    } else {
        (WebPushHandle::disabled(), None)
    };
    let (engine, mut engine_events, engine_thread, engine_ready) =
        spawn_engine(data_path, bbs_registry.clone(), league_registry.clone());
    tokio::task::spawn_blocking(move || engine_ready.recv())
        .await
        .map_err(|_| ServerError::EngineStopped)?
        .map_err(|_| ServerError::EngineStopped)?
        .map_err(ServerError::Engine)?;
    server_log!(
        "Cepheus Trader game listeners on {game_listener_text}; administrator listeners on \
         {admin_listener_text}; sysop listeners on {sysop_listener_text}; league coordinator \
         listeners on {league_listener_text}"
    );
    let sessions = Arc::new(Sessions::default());
    let bbs_control_connections = Arc::new(BbsControlConnections::default());
    let pending_game_authentications = Arc::new(Semaphore::new(MAX_PENDING_GAME_AUTHENTICATIONS));
    let clock_engine = engine.clone();
    tokio::spawn(async move {
        let start = tokio::time::Instant::now() + LIVE_CLOCK_PULSE;
        let mut interval = tokio::time::interval_at(start, LIVE_CLOCK_PULSE);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            match clock_engine.sender.try_send(EngineMessage::ClockPulse {
                sampled_at: Instant::now(),
            }) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => {}
                Err(mpsc::error::TrySendError::Closed(_)) => break,
            }
        }
    });
    let dispatcher_sessions = Arc::clone(&sessions);
    let dispatcher_engine = engine.clone();
    let dispatcher_bbs_control_connections = Arc::clone(&bbs_control_connections);
    let dispatcher_web_push = web_push.clone();
    let (fatal_sender, mut fatal_receiver) = oneshot::channel();
    tokio::spawn(async move {
        while let Some(event) = engine_events.recv().await {
            match event {
                EngineEvent::Delivery(delivery) => {
                    match dispatcher_sessions.deliver(&delivery).await {
                        DeliveryDisposition::Enqueued | DeliveryDisposition::Obsolete => {
                            if dispatcher_engine
                                .sender
                                .send(EngineMessage::AcknowledgeOutbox(delivery.outbox_sequence))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        DeliveryDisposition::NoSession => {}
                    }
                }
                EngineEvent::PhaseChanged(transition) => {
                    if let Some(alert) = upcoming_attention_push_alert(&transition) {
                        enqueue_web_alert(&dispatcher_web_push, alert).await;
                    }
                    dispatcher_sessions.phase_changed(&transition).await;
                }
                EngineEvent::CheckpointReady {
                    identity,
                    committed_sequence,
                    checkpoint,
                } => {
                    dispatcher_sessions
                        .checkpoint_ready(&identity, committed_sequence, &checkpoint)
                        .await;
                    enqueue_web_alert(
                        &dispatcher_web_push,
                        checkpoint_push_alert(identity, &checkpoint),
                    )
                    .await;
                }
                EngineEvent::EncounterReady {
                    identity,
                    committed_sequence,
                    encounter,
                } => {
                    dispatcher_sessions
                        .encounter_ready(&identity, committed_sequence, &encounter)
                        .await;
                    enqueue_web_alert(
                        &dispatcher_web_push,
                        encounter_push_alert(identity, &encounter),
                    )
                    .await;
                }
                EngineEvent::TrafficSnapshot {
                    identity,
                    committed_sequence,
                    snapshot,
                } => {
                    dispatcher_sessions
                        .traffic_snapshot(&identity, committed_sequence, &snapshot)
                        .await;
                }
                EngineEvent::TrafficMovement {
                    identity,
                    committed_sequence,
                    system_id,
                    observed_second,
                    contact,
                } => {
                    dispatcher_sessions
                        .traffic_movement(
                            &identity,
                            committed_sequence,
                            system_id,
                            observed_second,
                            &contact,
                        )
                        .await;
                }
                EngineEvent::RadioUnread {
                    identity,
                    committed_sequence,
                    ship_id,
                    unread_count,
                } => {
                    dispatcher_sessions
                        .radio_unread(&identity, committed_sequence, ship_id, unread_count)
                        .await;
                }
                EngineEvent::UniverseReset => {
                    dispatcher_sessions.close_all_for_universe_reset().await;
                }
                EngineEvent::PlayerAccessChanged(access) => {
                    dispatcher_sessions.close_for_access_change(&access).await;
                }
                EngineEvent::BbsAccessChanged {
                    bbs_id,
                    enabled,
                    reason,
                } => {
                    if !enabled {
                        dispatcher_sessions.close_bbs(bbs_id, &reason).await;
                        dispatcher_bbs_control_connections.close_bbs(bbs_id);
                    }
                }
                EngineEvent::Fatal(error) => {
                    let _ = fatal_sender.send(error);
                    return;
                }
            }
        }
    });

    let (incoming_sender, mut incoming_receiver) = mpsc::channel(CONNECTION_QUEUE_DEPTH);
    let mut accept_tasks = Vec::new();
    spawn_accept_tasks(
        game_listeners,
        ListenerRole::Game,
        &incoming_sender,
        &mut accept_tasks,
    );
    spawn_accept_tasks(
        admin_listeners,
        ListenerRole::Administrator,
        &incoming_sender,
        &mut accept_tasks,
    );
    spawn_accept_tasks(
        sysop_listeners,
        ListenerRole::Sysop,
        &incoming_sender,
        &mut accept_tasks,
    );
    spawn_accept_tasks(
        league_listeners,
        ListenerRole::League,
        &incoming_sender,
        &mut accept_tasks,
    );
    drop(incoming_sender);
    let _accept_tasks = AcceptTasks(accept_tasks);

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            result = incoming_receiver.recv() => {
                let connection = result.ok_or(ServerError::ListenerStopped)??;
                match connection {
                    AcceptedConnection::Game(socket, peer) => {
                        server_log!("game connection peer={peer} event=accepted");
                        let connection_engine = engine.clone();
                        let connection_sessions = Arc::clone(&sessions);
                        let connection_registry = bbs_registry.clone();
                        let connection_web_push = web_push.clone();
                        let Ok(authentication_permit) = Arc::clone(
                            &pending_game_authentications,
                        )
                        .try_acquire_owned()
                        else {
                            server_log!(
                                "game connection peer={peer} event=rejected reason=authentication-limit"
                            );
                            let _ = socket
                                .into_std()
                                .and_then(|socket| socket.shutdown(Shutdown::Both));
                            continue;
                        };
                        tokio::spawn(async move {
                            if let Err(error) = handle_connection(
                                socket,
                                peer,
                                connection_engine,
                                connection_sessions,
                                connection_registry,
                                connection_web_push,
                                authentication_permit,
                            )
                            .await
                            {
                                server_log!(
                                    "game connection peer={peer} event=failed error={error}"
                                );
                            }
                        });
                    }
                    AcceptedConnection::Administrator(socket, peer) => {
                        let connection_engine = engine.clone();
                        let connection_tls = admin_tls.clone();
                        let connection_sessions = Arc::clone(&sessions);
                        tokio::spawn(async move {
                            if let Err(error) = handle_admin_connection(
                                socket,
                                connection_engine,
                                connection_tls,
                                connection_sessions,
                            )
                            .await
                            {
                                server_log!(
                                    "administrator connection peer={peer} event=failed error={error}"
                                );
                            }
                        });
                    }
                    AcceptedConnection::Sysop(socket, peer) => {
                        let connection_engine = engine.clone();
                        let connection_registry = bbs_registry.clone();
                        let connection_controls = Arc::clone(&bbs_control_connections);
                        tokio::spawn(async move {
                            if let Err(error) = handle_sysop_connection(
                                socket,
                                connection_engine,
                                connection_registry,
                                connection_controls,
                            )
                            .await
                            {
                                server_log!(
                                    "sysop connection peer={peer} event=failed error={error}"
                                );
                            }
                        });
                    }
                    AcceptedConnection::League(socket, peer) => {
                        let connection_engine = engine.clone();
                        let connection_registry = league_registry.clone();
                        tokio::spawn(async move {
                            if let Err(error) = handle_league_connection(
                                socket,
                                connection_engine,
                                connection_registry,
                            ).await {
                                server_log!(
                                    "league coordinator connection peer={peer} event=failed error={error}"
                                );
                            }
                        });
                    }
                }
            }
            result = &mut fatal_receiver => {
                let error = result.unwrap_or_else(|_| "engine event stream stopped".into());
                drop(engine);
                let _ = engine_thread.join();
                return Err(ServerError::Engine(error));
            }
            result = &mut shutdown => {
                result?;
                server_log!("shutdown requested");
                sessions.close_all_for_shutdown().await;
                engine.shutdown().await;
                drop(engine);
                tokio::task::spawn_blocking(move || engine_thread.join())
                    .await
                    .map_err(|_| ServerError::EngineStopped)?
                    .map_err(|_| ServerError::EngineStopped)?;
                return Ok(());
            }
        }
    }
}

async fn handle_connection(
    socket: TcpStream,
    peer: SocketAddr,
    engine: EngineHandle,
    sessions: Arc<Sessions>,
    bbs_registry: BbsRegistry,
    web_push: WebPushHandle,
    authentication_permit: tokio::sync::OwnedSemaphorePermit,
) -> Result<(), ServerError> {
    socket.set_nodelay(true)?;
    let socket = socket.into_std()?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    socket.set_write_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    let socket = Arc::new(socket);
    let handshake_socket = Arc::clone(&socket);
    let credentials: Vec<_> = bbs_registry
        .tls_credentials()
        .into_iter()
        .map(|(identity, key)| PskCredential {
            identity: identity.into_bytes(),
            key,
        })
        .collect();
    let tls = tokio::task::spawn_blocking(move || {
        TlsServer::handshake_many(&*handshake_socket, &credentials)
    })
    .await
    .map_err(|_| ServerError::TlsWorkerStopped)?
    .map_err(|error| ServerError::Tls(error.to_string()))?;
    server_log!("game connection peer={peer} event=tls-authenticated");
    drop(authentication_permit);
    socket.set_read_timeout(None)?;
    socket.set_write_timeout(None)?;
    let tls = Arc::new(tls);
    let (outbound, mut outbound_receiver) = mpsc::channel::<Vec<u8>>(CONNECTION_QUEUE_DEPTH);
    let writer_tls = Arc::clone(&tls);
    let writer_task = tokio::spawn(async move {
        while let Some(frame) = outbound_receiver.recv().await {
            let tls = Arc::clone(&writer_tls);
            tokio::task::spawn_blocking(move || write_tls_frame(&tls, &frame))
                .await
                .map_err(|_| io::Error::other("TLS writer stopped"))??;
        }
        Ok::<(), io::Error>(())
    });

    let hello_frame = read_tls_frame_async(Arc::clone(&tls)).await?;
    let client_version = decode_protocol_version(&hello_frame)?;
    if client_version != PROTOCOL_VERSION {
        server_log!(
            "game connection peer={peer} event=rejected reason=protocol-version client={client_version} server={PROTOCOL_VERSION}"
        );
        let reason = localized_version_rejection("CT-RPC", client_version, PROTOCOL_VERSION)?;
        let _ = outbound
            .send(encode_legacy_close_for_version(client_version, 0, &reason)?)
            .await;
        drop(outbound);
        let _ = writer_task.await;
        let _ = socket.shutdown(Shutdown::Both);
        return Ok(());
    }
    let (_, hello) = match decode_client_hello_with_version(&hello_frame) {
        Ok(hello) => hello,
        Err(error) => {
            server_log!(
                "game connection peer={peer} event=rejected reason=malformed-hello error={error}"
            );
            let reason = localized_error(
                &default_language(),
                "malformed-hello",
                "error",
                &error.to_string(),
            )?;
            let _ = outbound
                .send(encode_close_with_code(
                    0,
                    CloseCode::MalformedHello,
                    &reason,
                    &[],
                )?)
                .await;
            drop(outbound);
            let _ = writer_task.await;
            let _ = socket.shutdown(Shutdown::Both);
            return Ok(());
        }
    };
    let language = match negotiate_language(&hello.language_tag) {
        Ok(language) => language,
        Err(error) => {
            let (code, key) = match error {
                LanguageNegotiationError::Malformed => {
                    (CloseCode::MalformedHello, "malformed-hello")
                }
                LanguageNegotiationError::Unsupported => {
                    (CloseCode::UnsupportedLanguage, "unsupported-language")
                }
            };
            let (argument_name, argument) = if error == LanguageNegotiationError::Malformed {
                ("error", "invalid BCP 47 language tag")
            } else {
                ("languageTag", hello.language_tag.as_str())
            };
            server_log!(
                "game connection peer={peer} event=rejected reason=language tag={:?} error={error}",
                hello.language_tag
            );
            let reason = localized_error(&default_language(), key, argument_name, argument)?;
            let _ = outbound
                .send(encode_close_with_code(
                    0,
                    code,
                    &reason,
                    SUPPORTED_LANGUAGE_TAGS,
                )?)
                .await;
            drop(outbound);
            let _ = writer_task.await;
            let _ = socket.shutdown(Shutdown::Both);
            return Ok(());
        }
    };
    if tls
        .identity()
        .map_err(|error| ServerError::Tls(error.to_string()))?
        != hello.identity.bbs_id.to_string().as_bytes()
    {
        server_log!(
            "game connection peer={peer} event=rejected reason=bbs-identity-mismatch requested-bbs={} player={}",
            hello.identity.bbs_id,
            hello.identity.player_id
        );
        return Err(ServerError::BbsIdentityMismatch);
    }
    let opening = match engine.issue_session(hello.identity.clone()).await {
        Ok(opening) => opening,
        Err(ServerError::Engine(message)) if message.starts_with("player access denied:") => {
            let reason = message
                .strip_prefix("player access denied: ")
                .unwrap_or(&message);
            let reason = localized_error(&language, "access-denied", "reason", reason)?;
            server_log!(
                "game connection peer={peer} event=rejected reason=access-denied bbs={} player={}",
                hello.identity.bbs_id,
                hello.identity.player_id
            );
            let _ = outbound
                .send(encode_close_with_code(
                    0,
                    CloseCode::AccessDenied,
                    &reason,
                    &[],
                )?)
                .await;
            drop(outbound);
            let _ = writer_task.await;
            let _ = socket.shutdown(Shutdown::Both);
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let epoch = opening.epoch;
    let operational_sequence = opening.committed_sequence;
    let operational_phase = opening.phase;
    let (replaced_sender, mut replaced_receiver) = watch::channel(false);
    let active = ActiveSession {
        epoch,
        outbound: outbound.clone(),
        replaced: replaced_sender,
        socket: Some(Arc::clone(&socket)),
        language: language.clone(),
    };
    if let Some(previous) = sessions
        .replace_limited(hello.identity.clone(), active)
        .await?
    {
        if let Ok(event) = encode_session_replaced(previous.epoch) {
            let _ = previous.outbound.try_send(event);
        }
        let _ = previous.replaced.send(true);
        if let Some(socket) = previous.socket {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
    // Disablement removes the live credential before the disconnect event is
    // dispatched. Recheck after registration so a connection cannot slip in
    // between that event's session sweep and this insertion.
    if !bbs_registry.contains(hello.identity.bbs_id) {
        sessions
            .remove_if_current(&hello.identity, opening.epoch)
            .await;
        let _ = outbound
            .send(encode_close_with_code(
                opening.epoch,
                CloseCode::AccessDenied,
                "home BBS access is disabled",
                &[],
            )?)
            .await;
        drop(outbound);
        let _ = writer_task.await;
        let _ = socket.shutdown(Shutdown::Both);
        return Ok(());
    }
    outbound
        .send(encode_server_hello_with_affiliation(
            &hello.identity,
            epoch,
            opening.committed_sequence,
            opening.phase,
            language.tag(),
            &language.display_formatting(),
            opening.affiliation.as_ref(),
        )?)
        .await
        .map_err(|_| {
            ServerError::Io(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "connection writer stopped",
            ))
        })?;
    server_log!(
        "game connection peer={peer} event=session-opened bbs={} player={} epoch={} phase={:?}",
        hello.identity.bbs_id,
        hello.identity.player_id,
        epoch,
        opening.phase
    );
    if let Some(snapshot) = opening.traffic_snapshot {
        outbound
            .send(encode_traffic_snapshot(
                epoch,
                opening.committed_sequence,
                &snapshot,
            )?)
            .await
            .map_err(|_| {
                ServerError::Io(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "connection writer stopped",
                ))
            })?;
    }
    if opening.radio_unread_count != 0 {
        outbound
            .send(encode_radio_unread(
                epoch,
                opening.committed_sequence,
                opening.radio_ship_id,
                opening.radio_unread_count,
            )?)
            .await
            .map_err(|_| {
                ServerError::Io(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "connection writer stopped",
                ))
            })?;
    }
    if let Some(checkpoint) = opening.checkpoint {
        outbound
            .send(encode_checkpoint_ready(
                epoch,
                opening.committed_sequence,
                &checkpoint,
            )?)
            .await
            .map_err(|_| {
                ServerError::Io(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "connection writer stopped",
                ))
            })?;
    }
    if let Some(encounter) = opening.encounter {
        outbound
            .send(encode_encounter_ready(
                epoch,
                opening.committed_sequence,
                &encounter,
            )?)
            .await
            .map_err(|_| {
                ServerError::Io(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "connection writer stopped",
                ))
            })?;
    }

    let result: Result<&'static str, ServerError> = loop {
        let frame = tokio::select! {
            changed = replaced_receiver.changed() => {
                if changed.is_ok() && *replaced_receiver.borrow() {
                    break Ok("session-replaced");
                }
                continue;
            }
            result = read_tls_frame_async(Arc::clone(&tls)) => {
                match result {
                    Ok(frame) => frame,
                    Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                        break Ok("client-eof");
                    }
                    Err(error) => break Err(ServerError::Io(error)),
                }
            }
        };
        if decode_close(&frame)?.is_some() {
            break Ok("client-close");
        }
        match decode_request(&frame) {
            Ok(request) if request.session_epoch == epoch => {
                let operational_kind = match &request.command {
                    wire::Command::GetBrowserAlertStatus => {
                        let handle = web_push.clone();
                        let identity = hello.identity.clone();
                        Some(
                            tokio::task::spawn_blocking(move || {
                                handle
                                    .status(identity)
                                    .map(wire::OutcomeKind::BrowserAlertStatus)
                            })
                            .await
                            .map_err(|_| ServerError::EngineStopped)?,
                        )
                    }
                    wire::Command::CreateBrowserAlertEnrollment => {
                        let handle = web_push.clone();
                        let identity = hello.identity.clone();
                        let command_id = request.command_id;
                        Some(
                            tokio::task::spawn_blocking(move || {
                                handle
                                    .create_enrollment(identity, command_id)
                                    .map(wire::OutcomeKind::BrowserAlertEnrollment)
                            })
                            .await
                            .map_err(|_| ServerError::EngineStopped)?,
                        )
                    }
                    wire::Command::RevokeAllBrowserAlerts => {
                        let handle = web_push.clone();
                        let identity = hello.identity.clone();
                        Some(
                            tokio::task::spawn_blocking(move || {
                                handle
                                    .revoke_all(identity)
                                    .map(wire::OutcomeKind::BrowserAlertStatus)
                            })
                            .await
                            .map_err(|_| ServerError::EngineStopped)?,
                        )
                    }
                    _ => None,
                };
                if let Some(result) = operational_kind {
                    let kind = result.unwrap_or_else(|error| wire::OutcomeKind::Error {
                        code: wire::ErrorCode::InternalFailure,
                        message: error.to_string(),
                    });
                    let response = wire::Outcome {
                        command_id: request.command_id,
                        committed_sequence: operational_sequence,
                        revision: 0,
                        replayed: false,
                        phase: operational_phase,
                        kind,
                    };
                    if outbound
                        .send(encode_response(request.request_id, epoch, &response)?)
                        .await
                        .is_err()
                    {
                        break Err(ServerError::Io(io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            "connection writer stopped",
                        )));
                    }
                    continue;
                }
                if engine
                    .sender
                    .send(EngineMessage::Submit {
                        identity: hello.identity.clone(),
                        request,
                    })
                    .await
                    .is_err()
                {
                    break Err(ServerError::EngineStopped);
                }
            }
            Ok(_) => {
                let _ = outbound
                    .send(encode_close_with_code(
                        epoch,
                        CloseCode::StaleSession,
                        &language.text("wrong-session-epoch")?,
                        &[],
                    )?)
                    .await;
                break Ok("stale-session-epoch");
            }
            Err(error) => {
                let _ = outbound
                    .send(encode_close_with_code(
                        epoch,
                        CloseCode::InvalidRequest,
                        &localized_error(
                            &language,
                            "invalid-request",
                            "error",
                            &error.to_string(),
                        )?,
                        &[],
                    )?)
                    .await;
                break Ok("invalid-request");
            }
        }
    };
    sessions.remove_if_current(&hello.identity, epoch).await;
    engine.close_session(hello.identity.clone(), epoch).await;
    let _ = socket.shutdown(Shutdown::Both);
    drop(outbound);
    let _ = writer_task.await;
    match result {
        Ok(reason) => {
            server_log!(
                "game connection peer={peer} event=session-closed bbs={} player={} epoch={} reason={reason}",
                hello.identity.bbs_id,
                hello.identity.player_id,
                epoch
            );
            Ok(())
        }
        Err(error) => Err(error),
    }
}

async fn handle_admin_connection(
    socket: TcpStream,
    engine: EngineHandle,
    tls_config: AdminTlsConfig,
    sessions: Arc<Sessions>,
) -> Result<(), ServerError> {
    socket.set_nodelay(true)?;
    let socket = socket.into_std()?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    socket.set_write_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    let socket = Arc::new(socket);
    let handshake_socket = Arc::clone(&socket);
    let key = Arc::clone(&tls_config.key);
    let tls = tokio::task::spawn_blocking(move || {
        TlsServer::handshake(&*handshake_socket, b"admin", &key)
    })
    .await
    .map_err(|_| ServerError::TlsWorkerStopped)?
    .map_err(|error| ServerError::Tls(error.to_string()))?;
    socket.set_read_timeout(None)?;
    socket.set_write_timeout(None)?;
    let tls = Arc::new(tls);

    let hello_frame = read_tls_frame_async(Arc::clone(&tls)).await?;
    let client_version = admin_wire::decode_protocol_version(&hello_frame)?;
    if client_version != admin_wire::PROTOCOL_VERSION {
        let reason =
            localized_version_rejection("CT-Admin", client_version, admin_wire::PROTOCOL_VERSION)?;
        let close = admin_wire::encode_legacy_close(client_version, &reason)?;
        let writer = Arc::clone(&tls);
        let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
        return Ok(());
    }
    let (_, requested_language) = match admin_wire::decode_client_hello_with_version(&hello_frame) {
        Ok(hello) => hello,
        Err(error) => {
            let reason = localized_error(
                &default_language(),
                "malformed-hello",
                "error",
                &error.to_string(),
            )?;
            let close = admin_wire::encode_close(CloseCode::MalformedHello, &reason, &[])?;
            let writer = Arc::clone(&tls);
            let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
            return Ok(());
        }
    };
    let language = match negotiate_language(&requested_language) {
        Ok(language) => language,
        Err(error) => {
            let (code, key, argument_name, argument) = match error {
                LanguageNegotiationError::Malformed => (
                    CloseCode::MalformedHello,
                    "malformed-hello",
                    "error",
                    "invalid BCP 47 language tag",
                ),
                LanguageNegotiationError::Unsupported => (
                    CloseCode::UnsupportedLanguage,
                    "unsupported-language",
                    "languageTag",
                    requested_language.as_str(),
                ),
            };
            let reason = localized_error(&default_language(), key, argument_name, argument)?;
            let close = admin_wire::encode_close(code, &reason, SUPPORTED_LANGUAGE_TAGS)?;
            let writer = Arc::clone(&tls);
            let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
            return Ok(());
        }
    };
    let server_hello = admin_wire::encode_server_hello(language.tag())?;
    let hello_writer = Arc::clone(&tls);
    tokio::task::spawn_blocking(move || write_tls_frame(&hello_writer, &server_hello))
        .await
        .map_err(|_| ServerError::TlsWorkerStopped)??;

    loop {
        let frame = match read_tls_frame_async(Arc::clone(&tls)).await {
            Ok(frame) => frame,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(ServerError::Io(error)),
        };
        let request = match admin_wire::decode_request(&frame) {
            Ok(request) => request,
            Err(error) => {
                let reason =
                    localized_error(&language, "invalid-request", "error", &error.to_string())?;
                let close = admin_wire::encode_close(CloseCode::InvalidRequest, &reason, &[])?;
                let writer = Arc::clone(&tls);
                let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
                return Ok(());
            }
        };
        let response = match &request.command {
            admin_wire::AdminCommand::AddBbs { name } => {
                match engine.add_bbs(request.command_id, name.clone()).await {
                    Ok(credential) => admin_wire::encode_bbs_added(&request, &credential)?,
                    Err(ServerError::Engine(message)) => {
                        admin_wire::encode_invalid_request(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
            admin_wire::AdminCommand::InitializeUniverse => {
                let initialization = engine.initialize_universe(request.command_id).await?;
                admin_wire::encode_universe_initialized(&request, &initialization)?
            }
            admin_wire::AdminCommand::Status => {
                let status = engine.status().await?;
                admin_wire::encode_status(&request, &status, sessions.active_count().await)?
            }
            admin_wire::AdminCommand::LiveBackup { label } => {
                match engine
                    .live_backup(
                        (*tls_config.backup_root).clone(),
                        label.clone(),
                        request.command_id,
                    )
                    .await
                {
                    Ok(status) => admin_wire::encode_backup_complete(&request, label, &status)?,
                    Err(ServerError::Engine(message)) => {
                        admin_wire::encode_invalid_request(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
            admin_wire::AdminCommand::AddLeague => {
                match engine.add_league(request.command_id).await {
                    Ok(credential) => admin_wire::encode_league_added(&request, &credential)?,
                    Err(ServerError::Engine(message)) => {
                        admin_wire::encode_invalid_request(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
        };
        let writer = Arc::clone(&tls);
        tokio::task::spawn_blocking(move || write_tls_frame(&writer, &response))
            .await
            .map_err(|_| ServerError::TlsWorkerStopped)??;
    }
}

async fn handle_sysop_connection(
    socket: TcpStream,
    engine: EngineHandle,
    bbs_registry: BbsRegistry,
    control_connections: Arc<BbsControlConnections>,
) -> Result<(), ServerError> {
    socket.set_nodelay(true)?;
    let socket = socket.into_std()?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    socket.set_write_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    let socket = Arc::new(socket);
    let handshake_socket = Arc::clone(&socket);
    let credentials: Vec<_> = bbs_registry
        .tls_credentials()
        .into_iter()
        .map(|(identity, key)| PskCredential {
            identity: identity.into_bytes(),
            key,
        })
        .collect();
    let tls = tokio::task::spawn_blocking(move || {
        TlsServer::handshake_many(&*handshake_socket, &credentials)
    })
    .await
    .map_err(|_| ServerError::TlsWorkerStopped)?
    .map_err(|error| ServerError::Tls(error.to_string()))?;
    socket.set_read_timeout(None)?;
    socket.set_write_timeout(None)?;
    let identity = tls
        .identity()
        .map_err(|error| ServerError::Tls(error.to_string()))?;
    let identity =
        std::str::from_utf8(&identity).map_err(|_| ServerError::InvalidBbsPskIdentity)?;
    let bbs_id = identity
        .parse::<u32>()
        .map_err(|_| ServerError::InvalidBbsPskIdentity)?;
    if bbs_id == 0 || bbs_id.to_string() != identity {
        return Err(ServerError::InvalidBbsPskIdentity);
    }
    if !bbs_registry.contains(bbs_id) {
        return Ok(());
    }
    let _control_registration = control_connections.register(bbs_id, Arc::clone(&socket));
    // Pair the pre-registration check with a post-registration check. If
    // disablement won the race before registration, the event sweep may have
    // already run; if it happens after this check, the registered socket is
    // visible to that sweep.
    if !bbs_registry.contains(bbs_id) {
        return Ok(());
    }
    let tls = Arc::new(tls);

    let hello_frame = read_tls_frame_async(Arc::clone(&tls)).await?;
    let client_version = sysop_wire::decode_protocol_version(&hello_frame)?;
    if client_version != sysop_wire::PROTOCOL_VERSION {
        let reason =
            localized_version_rejection("CT-Sysop", client_version, sysop_wire::PROTOCOL_VERSION)?;
        let close = sysop_wire::encode_legacy_close(client_version, &reason)?;
        let writer = Arc::clone(&tls);
        let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
        return Ok(());
    }
    let (_, requested_language) = match sysop_wire::decode_client_hello_with_version(&hello_frame) {
        Ok(hello) => hello,
        Err(error) => {
            let reason = localized_error(
                &default_language(),
                "malformed-hello",
                "error",
                &error.to_string(),
            )?;
            let close = sysop_wire::encode_close(CloseCode::MalformedHello, &reason, &[])?;
            let writer = Arc::clone(&tls);
            let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
            return Ok(());
        }
    };
    let language = match negotiate_language(&requested_language) {
        Ok(language) => language,
        Err(error) => {
            let (code, key, argument_name, argument) = match error {
                LanguageNegotiationError::Malformed => (
                    CloseCode::MalformedHello,
                    "malformed-hello",
                    "error",
                    "invalid BCP 47 language tag",
                ),
                LanguageNegotiationError::Unsupported => (
                    CloseCode::UnsupportedLanguage,
                    "unsupported-language",
                    "languageTag",
                    requested_language.as_str(),
                ),
            };
            let reason = localized_error(&default_language(), key, argument_name, argument)?;
            let close = sysop_wire::encode_close(code, &reason, SUPPORTED_LANGUAGE_TAGS)?;
            let writer = Arc::clone(&tls);
            let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
            return Ok(());
        }
    };
    let server_hello = sysop_wire::encode_server_hello(language.tag())?;
    let hello_writer = Arc::clone(&tls);
    tokio::task::spawn_blocking(move || write_tls_frame(&hello_writer, &server_hello))
        .await
        .map_err(|_| ServerError::TlsWorkerStopped)??;

    loop {
        let frame = match read_tls_frame_async(Arc::clone(&tls)).await {
            Ok(frame) => frame,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(ServerError::Io(error)),
        };
        let request = match sysop_wire::decode_request(&frame) {
            Ok(request) => request,
            Err(error) => {
                let reason =
                    localized_error(&language, "invalid-request", "error", &error.to_string())?;
                let close = sysop_wire::encode_close(CloseCode::InvalidRequest, &reason, &[])?;
                let writer = Arc::clone(&tls);
                let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
                return Ok(());
            }
        };
        let response = match &request.command {
            sysop_wire::SysopCommand::GetConfiguration => {
                let configuration = engine.bbs_configuration(bbs_id).await?;
                sysop_wire::encode_configuration(&request, &configuration)?
            }
            sysop_wire::SysopCommand::SetConfiguration {
                expected_revision,
                settings,
            } => match engine
                .configure_bbs(
                    bbs_id,
                    request.command_id,
                    *expected_revision,
                    settings.clone(),
                )
                .await?
            {
                ConfigureBbsResult::Updated(configuration) => {
                    sysop_wire::encode_configuration(&request, &configuration)?
                }
                ConfigureBbsResult::Stale(configuration) => {
                    sysop_wire::encode_stale_revision(&request, &configuration)?
                }
                ConfigureBbsResult::NoEligibleSite(configuration) => {
                    sysop_wire::encode_no_eligible_site(&request, &configuration)?
                }
            },
            sysop_wire::SysopCommand::GetPlayerAccess { player_id } => {
                let access = engine
                    .player_access(PlayerIdentity {
                        bbs_id,
                        player_id: *player_id,
                    })
                    .await?;
                sysop_wire::encode_player_access(&request, &access)?
            }
            sysop_wire::SysopCommand::SetPlayerAccess {
                player_id,
                expected_revision,
                state,
                reason,
            } => match engine
                .set_player_access(
                    bbs_id,
                    request.command_id,
                    *player_id,
                    *expected_revision,
                    *state,
                    reason.clone(),
                )
                .await?
            {
                SetPlayerAccessResult::Updated(access) => {
                    sysop_wire::encode_player_access(&request, &access)?
                }
                SetPlayerAccessResult::Stale(access) => {
                    sysop_wire::encode_player_access_stale(&request, &access)?
                }
                SetPlayerAccessResult::PermanentlyRemoved(access) => {
                    sysop_wire::encode_player_permanently_removed(&request, &access)?
                }
            },
            sysop_wire::SysopCommand::IssueDirective { player_id, kind } => {
                match engine
                    .issue_sysop_directive(bbs_id, request.command_id, *player_id, *kind)
                    .await
                {
                    Ok(directive) => sysop_wire::encode_directive_issued(&request, &directive)?,
                    Err(ServerError::Engine(message)) => {
                        sysop_wire::encode_invalid_request(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
        };
        let writer = Arc::clone(&tls);
        tokio::task::spawn_blocking(move || write_tls_frame(&writer, &response))
            .await
            .map_err(|_| ServerError::TlsWorkerStopped)??;
    }
}

async fn handle_league_connection(
    socket: TcpStream,
    engine: EngineHandle,
    league_registry: LeagueRegistry,
) -> Result<(), ServerError> {
    socket.set_nodelay(true)?;
    let socket = socket.into_std()?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    socket.set_write_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    let socket = Arc::new(socket);
    let handshake_socket = Arc::clone(&socket);
    let credentials = league_registry
        .tls_credentials()
        .into_iter()
        .map(|(identity, key)| PskCredential {
            identity: identity.into_bytes(),
            key,
        })
        .collect::<Vec<_>>();
    let tls = tokio::task::spawn_blocking(move || {
        TlsServer::handshake_many(&*handshake_socket, &credentials)
    })
    .await
    .map_err(|_| ServerError::TlsWorkerStopped)?
    .map_err(|error| ServerError::Tls(error.to_string()))?;
    socket.set_read_timeout(None)?;
    socket.set_write_timeout(None)?;
    let identity = tls
        .identity()
        .map_err(|error| ServerError::Tls(error.to_string()))?;
    let identity =
        std::str::from_utf8(&identity).map_err(|_| ServerError::InvalidLeaguePskIdentity)?;
    let league_id = identity
        .parse::<u32>()
        .map_err(|_| ServerError::InvalidLeaguePskIdentity)?;
    if league_id == 0 || league_id.to_string() != identity {
        return Err(ServerError::InvalidLeaguePskIdentity);
    }
    let tls = Arc::new(tls);
    loop {
        let frame = match read_tls_frame_async(Arc::clone(&tls)).await {
            Ok(frame) => frame,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(ServerError::Io(error)),
        };
        let client_version = league_wire::decode_protocol_version(&frame)?;
        if client_version != league_wire::PROTOCOL_VERSION {
            let close = league_wire::encode_close(
                crate::ct_league_capnp::CloseCode::UnsupportedVersion,
                &format!(
                    "unsupported CT-League version {client_version}; server requires {}",
                    league_wire::PROTOCOL_VERSION
                ),
            )?;
            let writer = Arc::clone(&tls);
            let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
            return Ok(());
        }
        let request = match league_wire::decode_request(&frame) {
            Ok(request) => request,
            Err(error) => {
                let close = league_wire::encode_close(
                    crate::ct_league_capnp::CloseCode::InvalidRequest,
                    &error.to_string(),
                )?;
                let writer = Arc::clone(&tls);
                let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
                return Ok(());
            }
        };
        let response = match &request.command {
            league_wire::LeagueCommand::Status => match engine.league_status(league_id).await {
                Ok(status) => league_wire::encode_status(&request, &status)?,
                Err(ServerError::Engine(message)) => league_wire::encode_error(&request, &message)?,
                Err(error) => return Err(error),
            },
            league_wire::LeagueCommand::SetName {
                expected_revision,
                name,
            } => {
                match engine
                    .set_league_name(
                        league_id,
                        request.command_id,
                        *expected_revision,
                        name.clone(),
                    )
                    .await
                {
                    Ok(SetLeagueNameResult::Updated(status)) => {
                        league_wire::encode_name_set(&request, &status, false)?
                    }
                    Ok(SetLeagueNameResult::Stale(status)) => {
                        league_wire::encode_name_set(&request, &status, true)?
                    }
                    Err(ServerError::Engine(message)) => {
                        league_wire::encode_error(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
            league_wire::LeagueCommand::AddBbs { name } => {
                match engine
                    .add_league_bbs(league_id, request.command_id, name.clone())
                    .await
                {
                    Ok(credential) => league_wire::encode_bbs_added(&request, &credential)?,
                    Err(ServerError::Engine(message)) => {
                        league_wire::encode_error(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
            league_wire::LeagueCommand::SetBbsAccess {
                bbs_id,
                expected_revision,
                enabled,
                reason,
            } => {
                match engine
                    .set_league_member_enabled(
                        league_id,
                        request.command_id,
                        *bbs_id,
                        *expected_revision,
                        *enabled,
                        reason.clone(),
                    )
                    .await
                {
                    Ok(SetLeagueMemberAccessResult::Updated(member)) => {
                        league_wire::encode_member_updated(&request, &member, false)?
                    }
                    Ok(SetLeagueMemberAccessResult::Stale(member)) => {
                        league_wire::encode_member_updated(&request, &member, true)?
                    }
                    Err(ServerError::Engine(message)) => {
                        league_wire::encode_error(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
        };
        let writer = Arc::clone(&tls);
        tokio::task::spawn_blocking(move || write_tls_frame(&writer, &response))
            .await
            .map_err(|_| ServerError::TlsWorkerStopped)??;
    }
}

async fn read_tls_frame_async(tls: Arc<TlsServer>) -> io::Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || read_tls_frame(&tls))
        .await
        .map_err(|_| io::Error::other("TLS reader stopped"))?
}

fn read_tls_frame(tls: &TlsServer) -> io::Result<Vec<u8>> {
    let mut length = [0u8; 4];
    read_tls_exact(tls, &mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid CT-RPC frame length",
        ));
    }
    let mut bytes = vec![0; length];
    read_tls_exact(tls, &mut bytes)?;
    Ok(bytes)
}

fn read_tls_exact(tls: &TlsServer, mut data: &mut [u8]) -> io::Result<()> {
    while !data.is_empty() {
        let count = tls
            .receive(data)
            .map_err(|error| io::Error::new(io::ErrorKind::UnexpectedEof, error))?;
        data = &mut data[count..];
    }
    Ok(())
}

fn write_tls_frame(tls: &TlsServer, frame: &[u8]) -> io::Result<()> {
    let length = u32::try_from(frame.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "CT-RPC frame too large"))?;
    write_tls_all(tls, &length.to_be_bytes())?;
    write_tls_all(tls, frame)
}

fn write_tls_all(tls: &TlsServer, mut data: &[u8]) -> io::Result<()> {
    while !data.is_empty() {
        let count = tls
            .send(data)
            .map_err(|error| io::Error::new(io::ErrorKind::BrokenPipe, error))?;
        data = &data[count..];
    }
    Ok(())
}

#[cfg(test)]
async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
    let length = reader.read_u32().await? as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid CT-RPC frame length",
        ));
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes).await?;
    Ok(bytes)
}

#[cfg(test)]
async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, frame: &[u8]) -> io::Result<()> {
    let length = u32::try_from(frame.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "CT-RPC frame too large"))?;
    writer.write_u32(length).await?;
    writer.write_all(frame).await?;
    writer.flush().await
}

#[cfg(test)]
mod tests {
    use tokio::io::{duplex, split};

    use super::*;

    #[test]
    fn server_log_timestamp_is_utc_rfc3339() {
        assert_eq!(utc_timestamp(0, 0), "1970-01-01T00:00:00.000Z");
        assert_eq!(utc_timestamp(946_684_800, 123), "2000-01-01T00:00:00.123Z");
    }

    #[test]
    fn engine_startup_preserves_the_authoritative_error() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let (_engine, mut events, engine_thread, ready) = spawn_engine(
            file.path().to_owned(),
            BbsRegistry::default(),
            LeagueRegistry::default(),
        );

        let startup_error = ready.recv().unwrap().unwrap_err();
        assert!(
            startup_error.starts_with("storage I/O error:"),
            "{startup_error}"
        );
        assert!(matches!(
            events.blocking_recv(),
            Some(EngineEvent::Fatal(error)) if error == startup_error
        ));
        engine_thread.join().unwrap();
    }

    #[test]
    fn obsolete_client_rejection_requires_an_upgrade() {
        let reason = localized_version_rejection("CT-RPC", 3, 4).unwrap();
        assert!(reason.contains("version 3"), "{reason}");
        assert!(
            reason.contains("upgrade your Cepheus Trader client"),
            "{reason}"
        );
        assert!(reason.contains("requires version 4"), "{reason}");
    }

    fn traffic_contact(contact_id: u64, player_owned: bool) -> TrafficContact {
        TrafficContact {
            contact_id,
            catalog_id: 1,
            class_name: "Test class".into(),
            ship_name: "Test ship".into(),
            transponder: "CT-TEST".into(),
            operator_name: "Test registry".into(),
            role: "test vessel".into(),
            displacement_millitons: 100_000,
            origin_system_id: 1,
            destination_system_id: 1,
            movement: crate::traffic::TrafficMovementKind::Present,
            edge_second: 0,
            resolution: crate::traffic::TrafficContactResolution::Identified,
            confidence_percent: 100,
            player_owned,
            online_controlled: false,
            attachment: crate::traffic::TrafficAttachment::Spaceborne,
        }
    }

    #[test]
    fn online_control_marks_only_the_live_player_vessel() {
        let identity = PlayerIdentity {
            bbs_id: 17,
            player_id: 42,
        };
        let observers = HashMap::from([(
            identity,
            Observer {
                epoch: 1,
                active_ship_id: 99,
                system_id: Some(1),
                last_second: 0,
                radio_unread_count: 0,
            },
        )]);
        let online = online_ship_ids(&observers);
        let mut controlled = traffic_contact(99, true);
        let mut standing_orders = traffic_contact(100, true);
        let mut generated = traffic_contact(99, false);
        decorate_traffic_contact(&mut controlled, &online);
        decorate_traffic_contact(&mut standing_orders, &online);
        decorate_traffic_contact(&mut generated, &online);
        assert!(controlled.online_controlled);
        assert!(!standing_orders.online_controlled);
        assert!(!generated.online_controlled);
    }

    fn travel_transition_with_authority(
        authority: Option<wire::WaypointAuthority>,
    ) -> PlayerTravelTransition {
        PlayerTravelTransition {
            identity: PlayerIdentity {
                bbs_id: 17,
                player_id: 42,
            },
            committed_sequence: 9,
            revision: 3,
            phase: wire::PlayerPhase::Interplanetary,
            status: wire::TravelStatus {
                ship_id: 99,
                ship_name: "Far Horizon".into(),
                current_system_id: 1,
                current_system_name: "Origin".into(),
                destination_system_id: 1,
                destination_system_name: "Origin".into(),
                stage: wire::TravelStage::DepartingForJump,
                current_game_second: 1_000,
                due_second: 2_000,
                current_fuel_millitons: 10_000,
                fuel_capacity_millitons: 12_000,
                jump_fuel_millitons: 5_000,
                plan_id: 7,
                plan_revision: 3,
                leg_index: 0,
                origin: wire::FlightLocus::Port {
                    system_id: 1,
                    world_id: 1,
                    facility_id: 1,
                },
                destination: wire::FlightLocus::JumpLocus { system_id: 1 },
            },
            waypoint_authority_at_due: authority,
        }
    }

    #[test]
    fn upcoming_waypoint_alerts_split_hold_and_through_preferences() {
        let hold = upcoming_attention_push_alert(&travel_transition_with_authority(Some(
            wire::WaypointAuthority::Hold,
        )))
        .unwrap();
        assert_eq!(hold.kind, "attention-soon");
        assert!(hold.body.contains("wait for the captain's orders"));

        let through = upcoming_attention_push_alert(&travel_transition_with_authority(Some(
            wire::WaypointAuthority::Through,
        )))
        .unwrap();
        assert_eq!(through.kind, "automation-soon");
        assert!(
            through
                .body
                .contains("standing orders are filed to continue")
        );

        assert!(upcoming_attention_push_alert(&travel_transition_with_authority(None)).is_none());
    }

    #[tokio::test]
    async fn framing_round_trips() {
        let (left, right) = duplex(1024);
        let (_, mut left_writer) = split(left);
        let (mut right_reader, _) = split(right);
        let expected = vec![1, 2, 3, 4, 5];
        write_frame(&mut left_writer, &expected).await.unwrap();
        assert_eq!(read_frame(&mut right_reader).await.unwrap(), expected);
    }

    #[tokio::test]
    async fn framing_rejects_oversized_input_without_allocating_it() {
        let (left, right) = duplex(1024);
        let (_, mut left_writer) = split(left);
        let (mut right_reader, _) = split(right);
        left_writer
            .write_u32((MAX_FRAME_BYTES + 1) as u32)
            .await
            .unwrap();
        let error = read_frame(&mut right_reader).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn multiple_listeners_feed_one_accept_queue() {
        let addresses = [
            "127.0.0.1:0".parse().unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        ];
        let listeners = bind_listeners(&addresses, "test").unwrap();
        let bound = listeners
            .iter()
            .map(|listener| listener.local_addr().unwrap())
            .collect::<Vec<_>>();
        let (sender, mut receiver) = mpsc::channel(CONNECTION_QUEUE_DEPTH);
        let mut tasks = Vec::new();
        spawn_accept_tasks(listeners, ListenerRole::Game, &sender, &mut tasks);
        drop(sender);
        let _tasks = AcceptTasks(tasks);

        let mut clients = Vec::new();
        for address in bound {
            clients.push(TcpStream::connect(address).await.unwrap());
        }
        for _ in 0..clients.len() {
            let accepted = tokio::time::timeout(Duration::from_secs(1), receiver.recv())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert!(matches!(accepted, AcceptedConnection::Game(_, _)));
        }
    }

    #[tokio::test]
    async fn ipv6_listener_does_not_claim_the_ipv4_port() {
        let address = "[::1]:0".parse().unwrap();
        let ipv6 = match bind_listener(address) {
            Ok(listener) => listener,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::AddrNotAvailable | io::ErrorKind::Unsupported
                ) =>
            {
                return;
            }
            Err(error) => panic!("cannot create IPv6 test listener: {error}"),
        };
        let port = ipv6.local_addr().unwrap().port();
        let ipv4 = bind_listener(SocketAddr::from(([127, 0, 0, 1], port))).unwrap();
        assert_eq!(ipv4.local_addr().unwrap().port(), port);
    }

    #[tokio::test]
    async fn replacement_keeps_new_session_when_old_session_exits() {
        let sessions = Sessions::default();
        let identity = PlayerIdentity {
            bbs_id: 17,
            player_id: 42,
        };
        let (old_outbound, _) = mpsc::channel(1);
        let (old_replaced, _) = watch::channel(false);
        assert!(
            sessions
                .replace(
                    identity.clone(),
                    ActiveSession {
                        epoch: 1,
                        outbound: old_outbound,
                        replaced: old_replaced,
                        socket: None,
                        language: default_language(),
                    },
                )
                .await
                .is_none()
        );

        let (new_outbound, _) = mpsc::channel(1);
        let (new_replaced, _) = watch::channel(false);
        let previous = sessions
            .replace(
                identity.clone(),
                ActiveSession {
                    epoch: 2,
                    outbound: new_outbound,
                    replaced: new_replaced,
                    socket: None,
                    language: default_language(),
                },
            )
            .await
            .unwrap();
        assert_eq!(previous.epoch, 1);

        sessions.remove_if_current(&identity, 1).await;
        assert_eq!(sessions.players.lock().await[&identity].epoch, 2);
    }

    #[tokio::test]
    async fn same_local_player_on_different_bbss_has_distinct_sessions() {
        let sessions = Sessions::default();
        let first = PlayerIdentity {
            bbs_id: 17,
            player_id: 42,
        };
        let second = PlayerIdentity {
            bbs_id: 23,
            player_id: 42,
        };
        for identity in [first.clone(), second.clone()] {
            let (outbound, _) = mpsc::channel(1);
            let (replaced, _) = watch::channel(false);
            assert!(
                sessions
                    .replace(
                        identity,
                        ActiveSession {
                            epoch: 1,
                            outbound,
                            replaced,
                            socket: None,
                            language: default_language(),
                        },
                    )
                    .await
                    .is_none()
            );
        }
        let players = sessions.players.lock().await;
        assert!(players.contains_key(&first));
        assert!(players.contains_key(&second));
    }
}
