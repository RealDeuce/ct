//! Seed-derived observable traffic. Contacts are projections, never records.

use crate::crypto::{CryptoError, SeedStream, derive_seed};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Mutex, OnceLock};

use crate::creation;
use crate::simulation::{SECONDS_PER_DAY, SimulationSystem, traffic_rate_hundredths};
use crate::universe::{Starport, generate_primary_world};

pub const TRAFFIC_ORDER_VERSION: u16 = 1;
pub const CONTACT_VISIBILITY_SECONDS: u64 = 60 * 60;

pub fn transponder_for_id(id: u64) -> String {
    let public_code =
        crate::ship_condition::mix64(id ^ 0x5452_414e_5350_4f4e) & 0x0000_ffff_ffff_ffff;
    format!(
        "CT-{:04X}-{:04X}-{:04X}",
        (public_code >> 32) & 0xffff,
        (public_code >> 16) & 0xffff,
        public_code & 0xffff
    )
}

pub fn registered_ship_name(id: u64, catalog_id: u32) -> String {
    let path_id = TRAFFIC_DESIGNS
        .iter()
        .find(|design| design.catalog_id == catalog_id)
        .map_or(2, |design| design.path_id);
    let names = TRAFFIC_NAMES[usize::from(path_id.saturating_sub(1).min(8))];
    let entropy = crate::ship_condition::mix64(id ^ u64::from(catalog_id).rotate_left(23));
    names[entropy as usize % names.len()].to_owned()
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationalRoute {
    pub system_ids: Vec<u64>,
    pub catalog_id: u32,
    pub jump_rating: u8,
    /// Maximum fuel tankage, in ten-percent-of-displacement jump units.
    pub tank_jump_units: u8,
    /// Exponential rarity measure used by contract frequency and premiums.
    pub scarcity_level: u8,
    /// Route burden beyond an ordinary, refuel-each-stop J-2 service.
    pub capability_level: u8,
}

include!(concat!(env!("OUT_DIR"), "/traffic_catalog.rs"));

fn traffic_specs() -> &'static BTreeMap<u32, creation::ShipStatusSpec> {
    static SPECS: OnceLock<BTreeMap<u32, creation::ShipStatusSpec>> = OnceLock::new();
    SPECS.get_or_init(|| {
        TRAFFIC_DESIGNS
            .iter()
            .filter_map(|design| {
                creation::ship_status_spec(design.catalog_id).map(|spec| (design.catalog_id, spec))
            })
            .collect()
    })
}

fn traffic_spec(catalog_id: u32) -> Option<&'static creation::ShipStatusSpec> {
    traffic_specs().get(&catalog_id)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficMovementKind {
    Arrival,
    Departure,
    Present,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficContactResolution {
    TransponderOnly,
    Approximate,
    Identified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficAttachment {
    Spaceborne,
    Berthed,
    Landed,
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
    pub player_owned: bool,
    pub online_controlled: bool,
    pub attachment: TrafficAttachment,
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
        let design = select_design(&mut random, system, desired_displacement);
        let contact_id = random.next_u64()?;
        let name_pool = TRAFFIC_NAMES[usize::from(design.path_id.saturating_sub(1).min(8))];
        let base_name = name_pool[random.next_u64()? as usize % name_pool.len()];
        let (origin_system_id, destination_system_id) = match movement {
            TrafficMovementKind::Arrival => (neighbor, system.system_id),
            TrafficMovementKind::Departure => (system.system_id, neighbor),
            TrafficMovementKind::Present => unreachable!("generated traffic is always moving"),
        };
        contacts.push(TrafficContact {
            contact_id,
            catalog_id: design.catalog_id,
            class_name: design.class_name.into(),
            ship_name: base_name.to_owned(),
            transponder: transponder_for_id(contact_id),
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
            player_owned: false,
            online_controlled: false,
            attachment: TrafficAttachment::Spaceborne,
        });
    }
    contacts.sort_by_key(|contact| (contact.edge_second, contact.contact_id));
    Ok(contacts)
}

fn commercial_role(role: &str) -> bool {
    [
        "trader",
        "merchant",
        "freighter",
        "cargo",
        "passenger",
        "courier",
        "transport",
    ]
    .iter()
    .any(|needle| role.contains(needle))
}

fn design_scarcity(design: TrafficDesign) -> u8 {
    let Some(spec) = traffic_spec(design.catalog_id) else {
        return u8::MAX;
    };
    let price_level = spec
        .construction_price_credits
        .max(1)
        .ilog2()
        .saturating_sub(22) as u8;
    let tank_units = spec
        .fuel_capacity_millitons
        .saturating_mul(10)
        .checked_div(spec.displacement_millitons.max(1))
        .unwrap_or(0) as u8;
    price_level
        .saturating_add(design.jump_rating.saturating_sub(2).saturating_mul(3))
        .saturating_add(tank_units.saturating_sub(2))
}

fn design_weight(system: &SimulationSystem, design: TrafficDesign, desired: u64) -> u64 {
    let Some(spec) = traffic_spec(design.catalog_id) else {
        return 0;
    };
    if design.tech_level > system.tech_level.max(9) || design.jump_rating == 0 {
        return 0;
    }
    let price_shift = spec
        .construction_price_credits
        .max(1)
        .ilog2()
        .saturating_sub(20)
        .min(24);
    let mut weight = 1_u64 << (24 - price_shift);
    let displacement_ratio = design.displacement_millitons.max(desired)
        / design.displacement_millitons.min(desired).max(1);
    weight = weight
        .checked_div(displacement_ratio.max(1))
        .unwrap_or(0)
        .max(1);
    let trade_centric = system.population >= 5 && system.starport <= Starport::C as u8;
    if trade_centric && commercial_role(design.role) {
        weight = weight.saturating_mul(6);
    } else if !trade_centric
        && ["frontier", "scout", "survey", "patrol", "utility"]
            .iter()
            .any(|needle| design.role.contains(needle))
    {
        weight = weight.saturating_mul(3);
    }
    weight
}

fn endpoint_demand_weight(origin: &SimulationSystem, destination: &SimulationSystem) -> u64 {
    let population = u64::from(origin.population.saturating_add(1))
        .saturating_mul(u64::from(destination.population.saturating_add(1)));
    let port = u64::from(7_u8.saturating_sub(origin.starport.min(6)))
        .saturating_mul(u64::from(7_u8.saturating_sub(destination.starport.min(6))));
    let complementary = u64::from(
        (origin.population >= 6 && destination.population <= 5)
            || (destination.population >= 6 && origin.population <= 5),
    );
    population
        .saturating_mul(port.max(1))
        .saturating_mul(2 + complementary)
        .max(1)
}

fn pair_design_weight(
    origin: &SimulationSystem,
    destination: &SimulationSystem,
    design: TrafficDesign,
) -> u64 {
    let desired =
        nominal_displacement(origin, 1).saturating_add(nominal_displacement(destination, 1)) / 2;
    let origin_weight = design_weight(origin, design, desired);
    let destination_weight = design_weight(destination, design, desired);
    if origin_weight == 0 || destination_weight == 0 {
        return 0;
    }
    origin_weight
        .checked_mul(destination_weight)
        .unwrap_or(u64::MAX)
        .checked_div(1 << 16)
        .unwrap_or(0)
        .max(1)
}

fn select_design(
    random: &mut SeedStream,
    system: &SimulationSystem,
    desired: u64,
) -> TrafficDesign {
    let mut candidates = TRAFFIC_DESIGNS
        .iter()
        .copied()
        .filter(|design| design.jump_rating >= 2 && design_weight(system, *design, desired) > 0)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        candidates.extend(
            TRAFFIC_DESIGNS
                .iter()
                .copied()
                .filter(|design| design.jump_rating >= 2),
        );
    }
    let total = candidates
        .iter()
        .map(|design| design_weight(system, *design, desired))
        .sum::<u64>();
    let mut draw = random.next_u64().unwrap_or(0) % total.max(1);
    candidates.sort_by_key(|design| design.catalog_id);
    for design in candidates.iter().copied() {
        let weight = design_weight(system, design, desired);
        if draw < weight {
            return design;
        }
        draw -= weight;
    }
    candidates[0]
}

fn nominal_displacement(system: &SimulationSystem, calls: u64) -> u64 {
    let population_scale = 10_u64.saturating_pow(u32::from(system.population.min(8)) / 2);
    100_000_u64
        .saturating_add(population_scale.saturating_mul(10_000))
        .saturating_add(calls.saturating_mul(5_000))
        .clamp(100_000, 5_000_000)
}

fn distance_parsecs(left: &SimulationSystem, right: &SimulationSystem) -> f64 {
    left.position_parsecs
        .iter()
        .zip(right.position_parsecs)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn jump_units(distance: f64) -> u8 {
    distance.ceil().clamp(1.0, f64::from(u8::MAX)) as u8
}

fn system_refuels(system: &SimulationSystem, catalog_id: u32) -> bool {
    if system.starport <= Starport::D as u8 {
        return true;
    }
    let Some(spec) = traffic_spec(catalog_id) else {
        return false;
    };
    if !spec.has_fuel_scoop || spec.fuel_processing_millitons_per_day == 0 {
        return false;
    }
    generate_primary_world(
        system.system_id,
        system.system_id,
        system.name.clone(),
        system.generation_seed,
    )
    .is_ok_and(|world| world.gas_giants > 0)
}

fn design_route(
    systems: &[SimulationSystem],
    origin_system_id: u64,
    destination_system_id: u64,
    design: TrafficDesign,
) -> Option<Vec<u64>> {
    let spec = traffic_spec(design.catalog_id)?;
    let by_id = systems
        .iter()
        .map(|system| (system.system_id, system))
        .collect::<BTreeMap<_, _>>();
    by_id.get(&origin_system_id)?;
    by_id.get(&destination_system_id)?;
    let initial = (origin_system_id, spec.fuel_capacity_millitons);
    let refuelling_systems = systems
        .iter()
        .filter(|system| system_refuels(system, design.catalog_id))
        .map(|system| system.system_id)
        .collect::<BTreeSet<_>>();
    let mut frontier = VecDeque::from([initial]);
    let mut visited = BTreeSet::from([initial]);
    let mut previous = BTreeMap::new();
    let fuel_per_unit = spec.displacement_millitons / 10;
    while let Some(state @ (current_id, remaining)) = frontier.pop_front() {
        if current_id == destination_system_id {
            let mut route = vec![current_id];
            let mut cursor = state;
            while cursor != initial {
                cursor = previous[&cursor];
                route.push(cursor.0);
            }
            route.reverse();
            route.dedup();
            return Some(route);
        }
        let current = by_id[&current_id];
        for next in systems {
            if next.system_id == current_id {
                continue;
            }
            let distance = distance_parsecs(current, next);
            let units = jump_units(distance);
            if units > design.jump_rating {
                continue;
            }
            let required = fuel_per_unit.saturating_mul(u64::from(units));
            if required > remaining {
                continue;
            }
            let after = remaining - required;
            let next_remaining = if refuelling_systems.contains(&next.system_id) {
                spec.fuel_capacity_millitons
            } else {
                after
            };
            let next_state = (next.system_id, next_remaining);
            if visited.insert(next_state) {
                previous.insert(next_state, state);
                frontier.push_back(next_state);
            }
        }
    }
    None
}

fn route_capability_level(
    systems: &[SimulationSystem],
    route: &[u64],
    design: TrafficDesign,
) -> u8 {
    let by_id = systems
        .iter()
        .map(|system| (system.system_id, system))
        .collect::<BTreeMap<_, _>>();
    let mut high_jump_burden = 0_u8;
    let mut dry_run = 0_u8;
    let mut maximum_dry_run = 0_u8;
    for pair in route.windows(2) {
        let units = jump_units(distance_parsecs(by_id[&pair[0]], by_id[&pair[1]]));
        high_jump_burden =
            high_jump_burden.saturating_add(units.saturating_sub(2).saturating_mul(2));
        dry_run = dry_run.saturating_add(units);
        maximum_dry_run = maximum_dry_run.max(dry_run);
        if system_refuels(by_id[&pair[1]], design.catalog_id) {
            dry_run = 0;
        }
    }
    high_jump_burden.saturating_add(maximum_dry_run.saturating_sub(2))
}

fn design_can_follow_itinerary(
    systems: &[SimulationSystem],
    itinerary: &[u64],
    design: TrafficDesign,
) -> bool {
    let Some(spec) = traffic_spec(design.catalog_id) else {
        return false;
    };
    let by_id = systems
        .iter()
        .map(|system| (system.system_id, system))
        .collect::<BTreeMap<_, _>>();
    let mut fuel = spec.fuel_capacity_millitons;
    for pair in itinerary.windows(2) {
        let (Some(origin), Some(destination)) = (by_id.get(&pair[0]), by_id.get(&pair[1])) else {
            return false;
        };
        let units = jump_units(distance_parsecs(origin, destination));
        let required = (spec.displacement_millitons / 10).saturating_mul(u64::from(units));
        if units > design.jump_rating || required > fuel {
            return false;
        }
        fuel -= required;
        if system_refuels(destination, design.catalog_id) {
            fuel = spec.fuel_capacity_millitons;
        }
    }
    true
}

pub fn operational_route(
    systems: &[SimulationSystem],
    origin_system_id: u64,
    destination_system_id: u64,
) -> Option<OperationalRoute> {
    static ROUTES: OnceLock<Mutex<BTreeMap<(u64, u64, u64), Option<OperationalRoute>>>> =
        OnceLock::new();
    let fingerprint = systems.iter().fold(systems.len() as u64, |value, system| {
        crate::ship_condition::mix64(
            value
                ^ system.system_id.rotate_left(7)
                ^ system.position_parsecs[0].to_bits()
                ^ system.position_parsecs[1].to_bits().rotate_left(17)
                ^ system.position_parsecs[2].to_bits().rotate_left(31)
                ^ u64::from(system.population).rotate_left(41)
                ^ u64::from(system.tech_level).rotate_left(49)
                ^ u64::from(system.starport).rotate_left(57)
                ^ u64::from_be_bytes(system.generation_seed[..8].try_into().unwrap()),
        )
    });
    let key = (fingerprint, origin_system_id, destination_system_id);
    if let Some(route) = ROUTES
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .expect("traffic route cache lock")
        .get(&key)
        .cloned()
    {
        return route;
    }
    let route = calculate_operational_route(systems, origin_system_id, destination_system_id);
    ROUTES
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .expect("traffic route cache lock")
        .insert(key, route.clone());
    route
}

fn calculate_operational_route(
    systems: &[SimulationSystem],
    origin_system_id: u64,
    destination_system_id: u64,
) -> Option<OperationalRoute> {
    if origin_system_id == destination_system_id {
        return Some(OperationalRoute {
            system_ids: vec![origin_system_id],
            catalog_id: 0,
            jump_rating: 0,
            tank_jump_units: 0,
            scarcity_level: 0,
            capability_level: 0,
        });
    }
    let origin = systems
        .iter()
        .find(|system| system.system_id == origin_system_id)?;
    let destination = systems
        .iter()
        .find(|system| system.system_id == destination_system_id)?;
    let mut capability_classes = BTreeMap::<(u8, u8, bool), TrafficDesign>::new();
    for design in TRAFFIC_DESIGNS
        .iter()
        .copied()
        .filter(|design| pair_design_weight(origin, destination, *design) > 0)
    {
        let Some(spec) = traffic_spec(design.catalog_id) else {
            continue;
        };
        let tank_units = spec
            .fuel_capacity_millitons
            .saturating_mul(10)
            .checked_div(spec.displacement_millitons.max(1))
            .unwrap_or(0) as u8;
        let key = (
            design.jump_rating,
            tank_units,
            spec.has_fuel_scoop && spec.fuel_processing_millitons_per_day > 0,
        );
        let service_weight =
            pair_design_weight(origin, destination, design) >> design_scarcity(design).min(20);
        let replace = capability_classes.get(&key).is_none_or(|current| {
            service_weight
                > (pair_design_weight(origin, destination, *current)
                    >> design_scarcity(*current).min(20))
        });
        if replace {
            capability_classes.insert(key, design);
        }
    }
    capability_classes
        .into_values()
        .filter_map(|design| {
            let route = design_route(systems, origin_system_id, destination_system_id, design)?;
            let spec = traffic_spec(design.catalog_id)?;
            let tank_jump_units =
                spec.fuel_capacity_millitons
                    .saturating_mul(10)
                    .checked_div(spec.displacement_millitons.max(1))? as u8;
            let capability_level = route_capability_level(systems, &route, design);
            Some(OperationalRoute {
                system_ids: route,
                catalog_id: design.catalog_id,
                jump_rating: design.jump_rating,
                tank_jump_units,
                scarcity_level: design_scarcity(design),
                capability_level,
            })
        })
        .min_by_key(|route| {
            (
                route.scarcity_level,
                route.system_ids.len(),
                route.jump_rating,
                route.tank_jump_units,
                route.catalog_id,
            )
        })
}

pub fn departure_destination(
    origin: &SimulationSystem,
    systems: &[SimulationSystem],
    entropy: u64,
) -> Option<u64> {
    let mut candidates = systems
        .iter()
        .filter(|destination| destination.system_id != origin.system_id)
        .filter_map(|system| {
            let route = operational_route(systems, origin.system_id, system.system_id)?;
            let hops = route.system_ids.len().saturating_sub(1).max(1) as u64;
            let burden = hops
                .saturating_mul(hops)
                .saturating_mul(1_u64 << route.capability_level.min(16))
                .saturating_mul(1_u64 << route.scarcity_level.min(12));
            let weight = endpoint_demand_weight(origin, system)
                .saturating_mul(1 << 20)
                .checked_div(burden.max(1))
                .unwrap_or(0)
                .max(1);
            Some((system.system_id, weight))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(system_id, _)| *system_id);
    let total = candidates.iter().map(|(_, weight)| *weight).sum::<u64>();
    let mut draw = entropy % total.max(1);
    for (system_id, weight) in candidates {
        if draw < weight {
            return Some(system_id);
        }
        draw -= weight;
    }
    None
}

pub fn generated_itinerary(
    systems: &[SimulationSystem],
    origin_system_id: u64,
    destination_system_id: u64,
    entropy: u64,
) -> Option<OperationalRoute> {
    let mut route = operational_route(systems, origin_system_id, destination_system_id)?;
    let origin = systems
        .iter()
        .find(|system| system.system_id == origin_system_id)?;
    let destination = systems
        .iter()
        .find(|system| system.system_id == destination_system_id)?;
    let mut candidates = TRAFFIC_DESIGNS
        .iter()
        .copied()
        .filter(|design| design_can_follow_itinerary(systems, &route.system_ids, *design))
        .map(|design| {
            let weight = (pair_design_weight(origin, destination, design)
                >> design_scarcity(design).min(20))
            .max(1);
            (design, weight)
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(design, _)| design.catalog_id);
    let total = candidates.iter().map(|(_, weight)| *weight).sum::<u64>();
    let mut draw = entropy % total.max(1);
    for (design, weight) in candidates {
        if draw < weight {
            let spec = traffic_spec(design.catalog_id)?;
            route.catalog_id = design.catalog_id;
            route.jump_rating = design.jump_rating;
            route.tank_jump_units = (spec.fuel_capacity_millitons.saturating_mul(10)
                / spec.displacement_millitons.max(1)) as u8;
            route.scarcity_level = design_scarcity(design);
            route.capability_level = route_capability_level(systems, &route.system_ids, design);
            return Some(route);
        }
        draw -= weight;
    }
    None
}

pub(crate) fn contract_service_weight(capability_level: u8) -> u64 {
    1_u64 << (16 - capability_level.min(16))
}

pub(crate) fn contract_payment_basis_points(capability_level: u8) -> u64 {
    let capability = u64::from(capability_level);
    10_000_u64
        .saturating_add(capability.saturating_mul(5_000))
        .saturating_add(capability.saturating_mul(capability).saturating_mul(500))
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

    fn positioned_system(
        system_id: u64,
        position: [f64; 3],
        starport: Starport,
        seed: [u8; 32],
    ) -> SimulationSystem {
        SimulationSystem {
            system_id,
            name: format!("Test {system_id}"),
            position_parsecs: position,
            polity_id: 1,
            generation_seed: seed,
            population: 8,
            tech_level: 15,
            starport: starport as u8,
            next_system_day: 0,
            jump_two_neighbors: Vec::new(),
        }
    }

    fn gas_giant_free_seed(system_id: u64) -> [u8; 32] {
        (0_u8..=u8::MAX)
            .map(|byte| [byte; 32])
            .find(|seed| {
                generate_primary_world(system_id, system_id, format!("Test {system_id}"), *seed)
                    .unwrap()
                    .gas_giants
                    == 0
            })
            .expect("a gas-giant-free deterministic test seed")
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

    #[test]
    fn public_ship_labels_do_not_expose_internal_ids() {
        let id = 0x1234_5678_9abc_def0;
        let transponder = transponder_for_id(id);
        let name = registered_ship_name(id, 72);

        assert!(transponder.starts_with("CT-"));
        assert!(!transponder.contains("123456789ABCDEF0"));
        assert!(!name.contains(&id.to_string()));
        assert!(!name.is_empty());
    }

    #[test]
    fn operational_route_requires_tankage_to_cross_a_dry_system() {
        let systems = vec![
            positioned_system(1, [0.0, 0.0, 0.0], Starport::A, [1; 32]),
            positioned_system(2, [1.5, 0.0, 0.0], Starport::X, gas_giant_free_seed(2)),
            positioned_system(3, [3.0, 0.0, 0.0], Starport::A, [3; 32]),
        ];
        let route = operational_route(&systems, 1, 3).expect("extended-range traffic route");
        assert_eq!(route.system_ids, vec![1, 2, 3]);
        assert_eq!(route.jump_rating, 2);
        assert!(route.tank_jump_units >= 4);
        assert!(route.capability_level >= 2);
    }

    #[test]
    fn commercial_systems_favor_cheaper_commercial_designs() {
        let market = system(11);
        let mut commercial = TRAFFIC_DESIGNS
            .iter()
            .copied()
            .filter(|design| commercial_role(design.role))
            .filter_map(|design| traffic_spec(design.catalog_id).map(|spec| (design, spec)))
            .collect::<Vec<_>>();
        commercial.sort_by_key(|(_, spec)| spec.construction_price_credits);
        let (cheap, _) = commercial.first().unwrap();
        let (expensive, _) = commercial.last().unwrap();
        assert!(
            design_weight(&market, *cheap, cheap.displacement_millitons)
                > design_weight(&market, *expensive, expensive.displacement_millitons)
        );
    }

    #[test]
    fn specialist_contracts_are_rarer_and_more_lucrative() {
        assert!(contract_service_weight(0) > contract_service_weight(2));
        assert!(contract_service_weight(2) > contract_service_weight(8));
        assert!(contract_payment_basis_points(0) < contract_payment_basis_points(2));
        assert!(contract_payment_basis_points(2) < contract_payment_basis_points(8));
    }
}
