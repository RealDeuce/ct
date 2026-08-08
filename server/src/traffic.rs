//! Seed-derived observable traffic. Contacts are projections, never records.

use crate::crypto::{CryptoError, SeedStream, derive_seed};
use crate::simulation::{SECONDS_PER_DAY, SimulationSystem, traffic_rate_hundredths};

pub const TRAFFIC_ORDER_VERSION: u16 = 1;
pub const CONTACT_VISIBILITY_SECONDS: u64 = 60 * 60;

#[derive(Clone, Copy, Debug)]
pub struct TrafficDesign {
    pub catalog_id: u32,
    pub class_name: &'static str,
    pub role: &'static str,
    pub path_id: u8,
    pub tech_level: u8,
    pub jump_rating: u8,
    pub displacement_millitons: u64,
}

include!(concat!(env!("OUT_DIR"), "/traffic_catalog.rs"));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficMovementKind {
    Arrival,
    Departure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficContactResolution {
    TransponderOnly,
    Approximate,
    Identified,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrafficContact {
    pub contact_id: u64,
    pub catalog_id: u32,
    pub class_name: String,
    pub ship_name: String,
    pub transponder: String,
    pub operator_name: String,
    pub role: String,
    pub displacement_millitons: u64,
    pub origin_system_id: u64,
    pub destination_system_id: u64,
    pub movement: TrafficMovementKind,
    pub edge_second: u64,
    pub resolution: TrafficContactResolution,
    pub confidence_percent: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrafficSnapshot {
    pub system_id: u64,
    pub system_name: String,
    pub observed_second: u64,
    pub contacts: Vec<TrafficContact>,
}

pub fn snapshot(
    system: &SimulationSystem,
    game_second: u64,
) -> Result<Vec<TrafficContact>, CryptoError> {
    let start = game_second.saturating_sub(CONTACT_VISIBILITY_SECONDS);
    let end = game_second.saturating_add(CONTACT_VISIBILITY_SECONDS);
    movements(system, start, end)
}

pub fn movements(
    system: &SimulationSystem,
    after_second: u64,
    through_second: u64,
) -> Result<Vec<TrafficContact>, CryptoError> {
    if through_second < after_second || system.jump_two_neighbors.is_empty() {
        return Ok(Vec::new());
    }
    let first_day = after_second / SECONDS_PER_DAY;
    let last_day = through_second / SECONDS_PER_DAY;
    let mut contacts = Vec::new();
    for day in first_day..=last_day {
        contacts.extend(day_contacts(system, day)?.into_iter().filter(|contact| {
            contact.edge_second >= after_second && contact.edge_second <= through_second
        }));
    }
    contacts.sort_by_key(|contact| (contact.edge_second, contact.contact_id));
    Ok(contacts)
}

fn day_contacts(system: &SimulationSystem, day: u64) -> Result<Vec<TrafficContact>, CryptoError> {
    // The call count is the same first draw used by SystemDay. Everything
    // describing its observer-only ordering is on a separate derived stream,
    // so adding presentation detail cannot perturb the authoritative day job.
    let day_label = format!("simulation/system-day/v1/{day}");
    let mut day_random =
        SeedStream::new(derive_seed(system.generation_seed, day_label.as_bytes())?);
    let rate = traffic_rate_hundredths(system);
    let calls = rate / 100 + u64::from(day_random.next_u64()? % 100 < rate % 100);
    let total = calls.saturating_mul(2);
    if total == 0 {
        return Ok(Vec::new());
    }
    let label = format!(
        "observable-traffic/v{TRAFFIC_ORDER_VERSION}/{}/{day}",
        system.system_id
    );
    let mut random = SeedStream::new(derive_seed(system.generation_seed, label.as_bytes())?);
    let day_start = day.saturating_mul(SECONDS_PER_DAY);
    let mut contacts = Vec::with_capacity(usize::try_from(total).unwrap_or(usize::MAX));
    for ordinal in 0..total {
        let jitter = random.next_u64()? % SECONDS_PER_DAY;
        let offset = (ordinal
            .saturating_mul(SECONDS_PER_DAY)
            .saturating_add(jitter))
            / total;
        let movement = if ordinal % 2 == 0 {
            TrafficMovementKind::Arrival
        } else {
            TrafficMovementKind::Departure
        };
        let neighbor = system.jump_two_neighbors
            [random.next_u64()? as usize % system.jump_two_neighbors.len()];
        let desired_displacement = nominal_displacement(system, calls);
        let design = select_design(&mut random, system.tech_level, desired_displacement);
        let contact_id = random.next_u64()?;
        let name_pool = TRAFFIC_NAMES[usize::from(design.path_id.saturating_sub(1).min(8))];
        let base_name = name_pool[random.next_u64()? as usize % name_pool.len()];
        let suffix = contact_id % 10_000;
        let (origin_system_id, destination_system_id) = match movement {
            TrafficMovementKind::Arrival => (neighbor, system.system_id),
            TrafficMovementKind::Departure => (system.system_id, neighbor),
        };
        contacts.push(TrafficContact {
            contact_id,
            catalog_id: design.catalog_id,
            class_name: design.class_name.into(),
            ship_name: format!("{base_name} {suffix:04}"),
            transponder: format!("CT-{contact_id:016X}"),
            operator_name: format!(
                "{} {} Operations",
                system.name,
                path_operator(design.path_id)
            ),
            role: design.role.into(),
            displacement_millitons: design.displacement_millitons,
            origin_system_id,
            destination_system_id,
            movement,
            edge_second: day_start.saturating_add(offset.min(SECONDS_PER_DAY - 1)),
            resolution: TrafficContactResolution::Identified,
            confidence_percent: 100,
        });
    }
    contacts.sort_by_key(|contact| (contact.edge_second, contact.contact_id));
    Ok(contacts)
}

fn select_design(random: &mut SeedStream, tech_level: u8, desired: u64) -> TrafficDesign {
    let mut candidates = TRAFFIC_DESIGNS
        .iter()
        .copied()
        .filter(|design| design.tech_level <= tech_level.max(9) && design.jump_rating >= 2)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        candidates.extend(
            TRAFFIC_DESIGNS
                .iter()
                .copied()
                .filter(|design| design.jump_rating >= 2),
        );
    }
    candidates.sort_by_key(|design| design.displacement_millitons.abs_diff(desired));
    let band = candidates.len().min(8);
    candidates[random.next_u64().unwrap_or(0) as usize % band]
}

fn nominal_displacement(system: &SimulationSystem, calls: u64) -> u64 {
    let population_scale = 10_u64.saturating_pow(u32::from(system.population.min(8)) / 2);
    100_000_u64
        .saturating_add(population_scale.saturating_mul(10_000))
        .saturating_add(calls.saturating_mul(5_000))
        .clamp(100_000, 5_000_000)
}

fn path_operator(path: u8) -> &'static str {
    TRAFFIC_OPERATORS
        .get(usize::from(path.saturating_sub(1)))
        .copied()
        .unwrap_or("Transport")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system(seed: u8) -> SimulationSystem {
        SimulationSystem {
            system_id: 1,
            name: "Sol".into(),
            position_parsecs: [0.0; 3],
            polity_id: 1,
            generation_seed: [seed; 32],
            population: 10,
            tech_level: 13,
            starport: 0,
            next_system_day: 0,
            jump_two_neighbors: vec![2, 3, 4],
        }
    }

    #[test]
    fn traffic_is_repeatable_named_and_spread_through_day() {
        let first = movements(&system(7), 0, SECONDS_PER_DAY - 1).unwrap();
        let second = movements(&system(7), 0, SECONDS_PER_DAY - 1).unwrap();
        assert_eq!(first, second);
        assert!(first.len() > 20);
        assert!(first.first().unwrap().edge_second < 120 * 60);
        assert!(first.last().unwrap().edge_second > SECONDS_PER_DAY - 120 * 60);
        assert!(first.iter().all(|contact| !contact.ship_name.is_empty()));
        assert!(
            first
                .windows(2)
                .any(|pair| pair[0].edge_second != pair[1].edge_second)
        );
    }

    #[test]
    fn snapshot_is_a_pure_projection() {
        let current = 10 * SECONDS_PER_DAY + 12 * 60 * 60;
        assert_eq!(
            snapshot(&system(9), current).unwrap(),
            snapshot(&system(9), current).unwrap()
        );
    }
}
