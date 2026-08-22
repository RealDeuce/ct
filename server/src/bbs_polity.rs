//! Conditioned placement geometry for BBS polities.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::universe::{
    SOL_SYSTEM_ID, StellarSystem, galactic_cylindrical_position,
    stellar_component_density_per_cubic_parsec,
};

pub const BBS_POLITY_GENERATION_VERSION: u16 = 2;
pub const BBS_COVERAGE_SAMPLER_VERSION: u16 = 1;
pub const BBS_POLITY_SYSTEM_COUNT: usize = 10;
pub const BBS_POLITY_TOTAL_SEEDED_SYSTEMS: usize = BBS_POLITY_SYSTEM_COUNT + 1;
pub const BBS_GALACTOCENTRIC_MIN_PARSECS: f64 = 6_000.0;
pub const BBS_GALACTOCENTRIC_MAX_PARSECS: f64 = 11_000.0;
pub const BBS_GUARD_RADIUS_PARSECS: f64 = 3.0;
pub const BBS_LOCAL_DENSITY_MIN: f64 = 0.020;
pub const BBS_LOCAL_DENSITY_MAX: f64 = 0.300;
pub const BBS_MAX_SEED_DRAWS_PER_ROLE: usize = 65_536;

pub const CAPITAL_INDEX: usize = 2;
pub const FIRST_COMPANION_INDEX: usize = 3;
pub const SECOND_COMPANION_INDEX: usize = 4;

// The inward gateway is index 0. Index 8 is the outward gateway and the last
// point is an unaligned, already-resolved frontier system through which a
// later polity may attach beyond this polity's immutable guard volume.
const CLUSTER_TEMPLATE: [[f64; 3]; BBS_POLITY_SYSTEM_COUNT] = [
    [1.75, 0.0, 0.0],
    [3.25, 0.0, 0.0],
    [4.50, 0.0, 0.0],
    [4.50, 0.75, 0.0],
    [4.50, -0.75, 0.0],
    [5.00, 1.50, 0.50],
    [5.00, -1.50, -0.50],
    [5.00, 0.0, -1.50],
    [7.00, 0.0, 1.50],
    [5.00, 0.0, 1.50],
];
const FRONTIER_STUB_TEMPLATE: [f64; 3] = [8.75, 0.0, 1.50];

#[derive(Clone, Debug, PartialEq)]
pub struct BbsPolitySite {
    pub anchor_system_id: u64,
    pub cluster_positions_parsecs: [[f64; 3]; BBS_POLITY_SYSTEM_COUNT],
    pub frontier_stub_position_parsecs: [f64; 3],
    pub gateway_crossings: u8,
    pub nearest_polity_distance_parsecs: f64,
    pub conditioning_cost: f64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BbsHome {
    pub bbs_id: u32,
    pub polity_id: u64,
    pub capital_system_id: u64,
    pub capital_world_id: u64,
    pub companion_system_ids: [u64; 2],
    pub cluster_system_ids: [u64; BBS_POLITY_SYSTEM_COUNT],
    pub frontier_stub_system_id: u64,
    pub placement_seed: [u8; 32],
    pub generation_version: u16,
}

fn distance_squared(first: [f64; 3], second: [f64; 3]) -> f64 {
    first
        .iter()
        .zip(second)
        .map(|(left, right)| (left - right).powi(2))
        .sum()
}

fn cross(first: [f64; 3], second: [f64; 3]) -> [f64; 3] {
    [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    let length = vector.iter().map(|value| value.powi(2)).sum::<f64>().sqrt();
    vector.map(|value| value / length)
}

fn quantize(value: f64) -> f64 {
    (value * 4.0).round() / 4.0
}

fn transform(
    anchor: [f64; 3],
    local: [f64; 3],
    forward: [f64; 3],
    side: [f64; 3],
    vertical: [f64; 3],
) -> [f64; 3] {
    std::array::from_fn(|axis| {
        quantize(
            anchor[axis]
                + local[0] * forward[axis]
                + local[1] * side[axis]
                + local[2] * vertical[axis],
        )
    })
}

fn gcd(mut left: i8, mut right: i8) -> i8 {
    left = left.abs();
    right = right.abs();
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn primitive_directions() -> Vec<[f64; 3]> {
    let mut directions = Vec::new();
    for x in -2_i8..=2 {
        for y in -2_i8..=2 {
            for z in -2_i8..=2 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }
                if gcd(gcd(x, y), z) != 1 {
                    continue;
                }
                directions.push(normalize([f64::from(x), f64::from(y), f64::from(z)]));
            }
        }
    }
    directions
}

fn template_at(anchor: &StellarSystem, forward: [f64; 3]) -> BbsPolitySite {
    let reference = if forward[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let side = normalize(cross(reference, forward));
    let vertical = normalize(cross(forward, side));
    let cluster_positions_parsecs = CLUSTER_TEMPLATE
        .map(|point| transform(anchor.position_parsecs, point, forward, side, vertical));
    let frontier_stub_position_parsecs = transform(
        anchor.position_parsecs,
        FRONTIER_STUB_TEMPLATE,
        forward,
        side,
        vertical,
    );
    BbsPolitySite {
        anchor_system_id: anchor.id,
        cluster_positions_parsecs,
        frontier_stub_position_parsecs,
        gateway_crossings: 0,
        nearest_polity_distance_parsecs: f64::INFINITY,
        conditioning_cost: f64::INFINITY,
    }
}

fn internally_jump_two_connected(positions: &[[f64; 3]; BBS_POLITY_SYSTEM_COUNT]) -> bool {
    let mut visited = [false; BBS_POLITY_SYSTEM_COUNT];
    visited[0] = true;
    loop {
        let before = visited;
        for index in 0..BBS_POLITY_SYSTEM_COUNT {
            if !visited[index] {
                continue;
            }
            for other in 0..BBS_POLITY_SYSTEM_COUNT {
                if distance_squared(positions[index], positions[other]) <= 4.0 + 1e-9 {
                    visited[other] = true;
                }
            }
        }
        if visited == before {
            break;
        }
    }
    visited.into_iter().all(|value| value)
}

fn jump_two_reachable_from_sol(existing: &[StellarSystem]) -> HashSet<u64> {
    let bucket_coordinate = |position: [f64; 3]| {
        (
            (position[0] / 2.0).floor() as i64,
            (position[1] / 2.0).floor() as i64,
            (position[2] / 2.0).floor() as i64,
        )
    };
    let mut buckets = HashMap::<(i64, i64, i64), Vec<usize>>::new();
    for (index, system) in existing.iter().enumerate() {
        buckets
            .entry(bucket_coordinate(system.position_parsecs))
            .or_default()
            .push(index);
    }

    let Some(sol_index) = existing
        .iter()
        .position(|system| system.id == SOL_SYSTEM_ID)
    else {
        return HashSet::new();
    };
    let mut reachable = HashSet::from([SOL_SYSTEM_ID]);
    let mut frontier = VecDeque::from([sol_index]);
    while let Some(index) = frontier.pop_front() {
        let position = existing[index].position_parsecs;
        let (bx, by, bz) = bucket_coordinate(position);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let Some(neighbors) = buckets.get(&(bx + dx, by + dy, bz + dz)) else {
                        continue;
                    };
                    for &neighbor in neighbors {
                        let system = &existing[neighbor];
                        if !reachable.contains(&system.id)
                            && distance_squared(position, system.position_parsecs) <= 4.0 + 1e-9
                        {
                            reachable.insert(system.id);
                            frontier.push_back(neighbor);
                        }
                    }
                }
            }
        }
    }
    reachable
}

fn geometrically_eligible(
    site: &mut BbsPolitySite,
    existing: &[StellarSystem],
    sol_reachable: &HashSet<u64>,
) -> bool {
    if !internally_jump_two_connected(&site.cluster_positions_parsecs) {
        return false;
    }
    if distance_squared(
        site.cluster_positions_parsecs[CAPITAL_INDEX],
        site.cluster_positions_parsecs[FIRST_COMPANION_INDEX],
    ) > 1.0 + 1e-9
        || distance_squared(
            site.cluster_positions_parsecs[CAPITAL_INDEX],
            site.cluster_positions_parsecs[SECOND_COMPANION_INDEX],
        ) > 1.0 + 1e-9
    {
        return false;
    }

    for (index, position) in site.cluster_positions_parsecs.iter().enumerate() {
        let galactic_radius = galactic_cylindrical_position(*position).radius_parsecs;
        if galactic_radius - BBS_GUARD_RADIUS_PARSECS < BBS_GALACTOCENTRIC_MIN_PARSECS
            || galactic_radius + BBS_GUARD_RADIUS_PARSECS > BBS_GALACTOCENTRIC_MAX_PARSECS
        {
            return false;
        }
        for other in &site.cluster_positions_parsecs[index + 1..] {
            if distance_squared(*position, *other) < 0.25_f64.powi(2) {
                return false;
            }
        }
    }
    let stub_radius =
        galactic_cylindrical_position(site.frontier_stub_position_parsecs).radius_parsecs;
    if !(BBS_GALACTOCENTRIC_MIN_PARSECS..=BBS_GALACTOCENTRIC_MAX_PARSECS).contains(&stub_radius) {
        return false;
    }

    let density =
        stellar_component_density_per_cubic_parsec(site.cluster_positions_parsecs[CAPITAL_INDEX]);
    if !(BBS_LOCAL_DENSITY_MIN..=BBS_LOCAL_DENSITY_MAX).contains(&density) {
        return false;
    }
    site.conditioning_cost = (density / crate::universe::LOCAL_COMPONENT_DENSITY_PER_CUBIC_PARSEC)
        .ln()
        .abs();

    let mut gateway_crossings = 0_u8;
    let mut sol_gateway = false;
    for cluster in site.cluster_positions_parsecs {
        for external in existing {
            let squared = distance_squared(cluster, external.position_parsecs);
            if squared < 0.25_f64.powi(2) {
                return false;
            }
            if squared <= 9.0 + 1e-9 {
                if squared > 4.0 + 1e-9 {
                    return false;
                }
                gateway_crossings = match gateway_crossings.checked_add(1) {
                    Some(value) => value,
                    None => return false,
                };
                if sol_reachable.contains(&external.id) {
                    sol_gateway = true;
                }
            }
        }
        let squared = distance_squared(cluster, site.frontier_stub_position_parsecs);
        if squared < 0.25_f64.powi(2) {
            return false;
        }
        if squared <= 9.0 + 1e-9 {
            if squared > 4.0 + 1e-9 {
                return false;
            }
            gateway_crossings = match gateway_crossings.checked_add(1) {
                Some(value) => value,
                None => return false,
            };
        }
    }
    if !sol_gateway || !(1..=3).contains(&gateway_crossings) {
        return false;
    }
    site.gateway_crossings = gateway_crossings;

    site.nearest_polity_distance_parsecs = existing
        .iter()
        .filter(|system| system.polity_id != 0)
        .map(|system| {
            distance_squared(
                site.cluster_positions_parsecs[CAPITAL_INDEX],
                system.position_parsecs,
            )
            .sqrt()
        })
        .fold(f64::INFINITY, f64::min);
    site.nearest_polity_distance_parsecs.is_finite()
}

pub fn candidate_sites(existing: &[StellarSystem]) -> Vec<BbsPolitySite> {
    let directions = primitive_directions();
    let sol_reachable = jump_two_reachable_from_sol(existing);
    let mut candidates = Vec::new();
    for anchor in existing {
        for direction in &directions {
            let mut site = template_at(anchor, *direction);
            if geometrically_eligible(&mut site, existing, &sol_reachable) {
                candidates.push(site);
            }
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system(id: u64, position_parsecs: [f64; 3], polity_id: u64) -> StellarSystem {
        StellarSystem {
            id,
            name: format!("System {id}"),
            primary_world_name: format!("System {id} Primary"),
            position_parsecs,
            polity_id,
            generation_seed: [id as u8; 32],
            generation_version: 1,
        }
    }

    #[test]
    fn canonical_template_has_capital_routes_and_two_gateways() {
        let existing = vec![system(1, [0.0; 3], 1)];
        let sol_reachable = jump_two_reachable_from_sol(&existing);
        let mut site = template_at(&existing[0], [1.0, 0.0, 0.0]);
        assert!(geometrically_eligible(&mut site, &existing, &sol_reachable));
        assert_eq!(site.gateway_crossings, 2);
        assert!(internally_jump_two_connected(
            &site.cluster_positions_parsecs
        ));
        assert!(
            distance_squared(
                site.cluster_positions_parsecs[CAPITAL_INDEX],
                site.cluster_positions_parsecs[FIRST_COMPANION_INDEX],
            ) <= 1.0
        );
    }

    #[test]
    fn an_undesigned_jump_three_crossing_rejects_a_site() {
        let existing = vec![system(1, [0.0; 3], 1), system(2, [1.0, 2.5, 0.0], 1)];
        let sol_reachable = jump_two_reachable_from_sol(&existing);
        let mut site = template_at(&existing[0], [1.0, 0.0, 0.0]);
        assert!(!geometrically_eligible(
            &mut site,
            &existing,
            &sol_reachable
        ));
    }

    #[test]
    fn gateway_into_a_disconnected_existing_component_rejects_a_site() {
        let existing = vec![
            system(SOL_SYSTEM_ID, [-2.25, 0.0, 0.0], 1),
            system(2, [0.0; 3], 1),
        ];
        let sol_reachable = jump_two_reachable_from_sol(&existing);
        assert_eq!(sol_reachable, HashSet::from([SOL_SYSTEM_ID]));

        let mut site = template_at(&existing[1], [1.0, 0.0, 0.0]);
        assert!(!geometrically_eligible(
            &mut site,
            &existing,
            &sol_reachable
        ));
    }

    #[test]
    fn sol_component_follows_successive_jump_two_legs() {
        let existing = vec![
            system(SOL_SYSTEM_ID, [0.0, 0.0, 0.0], 1),
            system(2, [2.0, 0.0, 0.0], 1),
            system(3, [4.0, 0.0, 0.0], 1),
            system(4, [6.0, 0.0, 0.0], 2),
            system(5, [9.0, 0.0, 0.0], 1),
        ];

        assert_eq!(
            jump_two_reachable_from_sol(&existing),
            HashSet::from([SOL_SYSTEM_ID, 2, 3, 4])
        );
    }

    #[test]
    fn direction_catalog_is_canonical_and_symmetric() {
        let directions = primitive_directions();
        assert_eq!(directions.len(), 98);
        for direction in &directions {
            assert!((direction.iter().map(|value| value.powi(2)).sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(directions.iter().any(|other| {
                other
                    .iter()
                    .zip(direction)
                    .all(|(left, right)| (*left + *right).abs() < 1e-12)
            }));
        }
    }
}
