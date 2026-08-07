//! Authoritative in-system geometry used to reach a safe Jump locus.

use crate::celestial::{BodyKind, CelestialSystem};

pub const AU_METERS: f64 = 149_597_870_700.0;
pub const STANDARD_GRAVITY_METERS_PER_SECOND_SQUARED: f64 = 9.806_65;
pub const SECONDS_PER_GAME_DAY: f64 = 86_400.0;
pub const JUMP_EXCLUSION_DIAMETERS: f64 = 100.0;
pub const MINIMUM_JUMP_APPROACH_DAYS: f64 = 0.5;
pub const BBS_CORE_MAXIMUM_JUMP_APPROACH_DAYS: f64 = 3.5;

const DIRECTION_SAMPLES: usize = 1_024;
const LOCUS_CLEARANCE_AU: f64 = 1e-10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpSafetySolution {
    /// Position in the system's seed-derived reference frame.
    pub locus_au: [f64; 3],
    pub distance_au: f64,
    /// Constant-thrust, midpoint-turnover time from the primary world.
    pub travel_days: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrontierFuelSolution {
    pub body_local_id: u32,
    pub body_name: String,
    /// Rest-to-rest travel from the primary-world Jump locus to the nearest
    /// gas giant and back. Collection and processing time are separate.
    pub round_trip_days: f64,
}

#[derive(Clone, Copy)]
struct ExclusionSphere {
    center_au: [f64; 3],
    radius_au: f64,
}

/// Distance covered by a rest-to-rest midpoint-turnover trip.
pub fn rest_to_rest_distance_au(travel_days: f64, thrust_g: f64) -> f64 {
    let seconds = travel_days * SECONDS_PER_GAME_DAY;
    thrust_g * STANDARD_GRAVITY_METERS_PER_SECOND_SQUARED * seconds * seconds / (4.0 * AU_METERS)
}

/// Duration of a rest-to-rest midpoint-turnover trip.
pub fn rest_to_rest_travel_days(distance_au: f64, thrust_g: f64) -> f64 {
    let seconds = 2.0
        * (distance_au * AU_METERS / (thrust_g * STANDARD_GRAVITY_METERS_PER_SECOND_SQUARED))
            .sqrt();
    seconds / SECONDS_PER_GAME_DAY
}

/// Find a safe departure/arrival locus for the primary world at `game_days`.
///
/// The point is outside the 100-diameter exclusion spheres of every star and
/// generated body. It is also outside a thrust-scaled safety sphere whose
/// radius takes exactly half a game day to traverse at constant thrust with a
/// midpoint turnover. The direction search is deterministic. Every returned
/// point is verified safe; seed conditioning may conservatively reject a
/// system if the sampled search misses a shorter valid route.
pub fn primary_world_jump_safety(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
) -> JumpSafetySolution {
    assert!(thrust_g.is_finite() && thrust_g > 0.0);
    let star_positions = star_positions(system, game_days);
    let body_positions = body_positions(system, game_days, &star_positions);
    let primary_index = system
        .bodies
        .iter()
        .position(|body| body.is_primary_world)
        .expect("derived systems always contain a primary world body");
    let origin = body_positions[primary_index];

    let mut spheres = Vec::with_capacity(system.stars.len() + system.bodies.len() + 1);
    for (star, position) in system.stars.iter().zip(&star_positions) {
        spheres.push(ExclusionSphere {
            center_au: *position,
            radius_au: jump_exclusion_radius_au(star.diameter_km()),
        });
    }
    for (body, position) in system.bodies.iter().zip(&body_positions) {
        spheres.push(ExclusionSphere {
            center_au: *position,
            radius_au: jump_exclusion_radius_au(body.diameter_km()),
        });
    }
    spheres.push(ExclusionSphere {
        center_au: origin,
        radius_au: rest_to_rest_distance_au(MINIMUM_JUMP_APPROACH_DAYS, thrust_g),
    });

    let mut directions = Vec::with_capacity(DIRECTION_SAMPLES + spheres.len() + 6);
    push_direction(&mut directions, subtract(origin, star_positions[0]));
    for axis in 0..3 {
        let mut positive = [0.0; 3];
        positive[axis] = 1.0;
        directions.push(positive);
        directions.push(positive.map(|value| -value));
    }
    for sphere in &spheres {
        push_direction(&mut directions, subtract(origin, sphere.center_au));
    }
    // A Fibonacci sphere provides stable, approximately uniform coverage.
    let golden_angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    for index in 0..DIRECTION_SAMPLES {
        let y = 1.0 - 2.0 * (index as f64 + 0.5) / DIRECTION_SAMPLES as f64;
        let radius = (1.0 - y * y).sqrt();
        let angle = golden_angle * index as f64;
        directions.push([radius * angle.cos(), y, radius * angle.sin()]);
    }

    let (distance_au, direction) = directions
        .into_iter()
        .map(|direction| {
            (
                first_safe_distance_on_ray(origin, direction, &spheres),
                direction,
            )
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .expect("direction set is nonempty");
    let distance_au = distance_au + LOCUS_CLEARANCE_AU;
    let locus_au = add(origin, scale(direction, distance_au));
    assert!(spheres.iter().all(|sphere| {
        magnitude(subtract(locus_au, sphere.center_au)) + 1e-12 >= sphere.radius_au
    }));
    JumpSafetySolution {
        locus_au,
        distance_au,
        travel_days: rest_to_rest_travel_days(distance_au, thrust_g)
            .max(MINIMUM_JUMP_APPROACH_DAYS),
    }
}

/// Find the quickest gas-giant skimming detour from the primary world's safe
/// Jump locus at the requested orbital epoch.
pub fn nearest_gas_giant_fuel_source(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
) -> Option<FrontierFuelSolution> {
    gas_giant_fuel_sources(system, game_days, thrust_g)
        .into_iter()
        .min_by(|left, right| left.round_trip_days.total_cmp(&right.round_trip_days))
}

/// Resolve a specifically plotted gas-giant fuel source at the current
/// orbital epoch. This is deliberately distinct from choosing the nearest
/// source: a committed flight plan must continue to name the body the captain
/// selected when the plan was accepted.
pub fn gas_giant_fuel_source(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
    body_local_id: u32,
) -> Option<FrontierFuelSolution> {
    gas_giant_fuel_sources(system, game_days, thrust_g)
        .into_iter()
        .find(|source| source.body_local_id == body_local_id)
}

pub fn gas_giant_fuel_sources(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
) -> Vec<FrontierFuelSolution> {
    let jump_locus = primary_world_jump_safety(system, game_days, thrust_g).locus_au;
    let star_positions = star_positions(system, game_days);
    let positions = body_positions(system, game_days, &star_positions);
    system
        .bodies
        .iter()
        .zip(positions)
        .filter_map(|(body, position)| {
            matches!(body.kind, BodyKind::GasGiant { .. }).then(|| {
                let radius_au = body.diameter_km() * 500.0 / AU_METERS;
                let distance_au = (magnitude(subtract(position, jump_locus)) - radius_au).max(0.0);
                FrontierFuelSolution {
                    body_local_id: body.local_id,
                    body_name: body.name.clone(),
                    round_trip_days: 2.0 * rest_to_rest_travel_days(distance_au, thrust_g),
                }
            })
        })
        .collect()
}

/// Find the quickest uninhabited water/ice source from the primary world's
/// safe Jump locus. Populated worlds are deliberately excluded: hydrographics
/// describes physical water, not permission to take it.
pub fn nearest_wilderness_water_source(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
) -> Option<FrontierFuelSolution> {
    wilderness_water_sources(system, game_days, thrust_g)
        .into_iter()
        .min_by(|left, right| left.round_trip_days.total_cmp(&right.round_trip_days))
}

/// Resolve a specifically plotted uninhabited water or ice source.
pub fn wilderness_water_source(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
    body_local_id: u32,
) -> Option<FrontierFuelSolution> {
    wilderness_water_sources(system, game_days, thrust_g)
        .into_iter()
        .find(|source| source.body_local_id == body_local_id)
}

pub fn wilderness_water_sources(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
) -> Vec<FrontierFuelSolution> {
    let jump_locus = primary_world_jump_safety(system, game_days, thrust_g).locus_au;
    let star_positions = star_positions(system, game_days);
    let positions = body_positions(system, game_days, &star_positions);
    system
        .bodies
        .iter()
        .zip(positions)
        .filter_map(|(body, position)| {
            let usable = match &body.kind {
                BodyKind::PlanetoidBelt { icy, .. } => *icy,
                BodyKind::Rocky { .. } => body
                    .world
                    .as_ref()
                    .is_some_and(|world| !world.is_inhabited() && world.hydrographics > 0),
                BodyKind::GasGiant { .. } => false,
            };
            usable.then(|| FrontierFuelSolution {
                body_local_id: body.local_id,
                body_name: body.name.clone(),
                round_trip_days: 2.0
                    * rest_to_rest_travel_days(magnitude(subtract(position, jump_locus)), thrust_g),
            })
        })
        .collect()
}

/// Deterministic orbital-phase guard used when conditioning a BBS core.
///
/// It includes epoch zero and each quadrature of the primary world's and all
/// companion stars' orbits. The returned value is the longest safe 1G
/// approach found at those phases.
pub fn bbs_core_jump_guard_days(system: &CelestialSystem) -> f64 {
    let mut sample_days = vec![0.0];
    add_orbit_quadratures(&mut sample_days, system.primary_world_body().orbit);
    for orbit in system.stars.iter().filter_map(|star| star.orbit) {
        add_orbit_quadratures(&mut sample_days, orbit);
    }
    sample_days
        .into_iter()
        .map(|game_days| primary_world_jump_safety(system, game_days, 1.0).travel_days)
        .max_by(f64::total_cmp)
        .expect("epoch-zero sample is present")
}

fn add_orbit_quadratures(sample_days: &mut Vec<f64>, orbit: crate::celestial::OrbitalElements) {
    for target_mean_anomaly in [0.0, 90.0, 180.0, 270.0] {
        let phase =
            (target_mean_anomaly - orbit.mean_anomaly_at_epoch_degrees).rem_euclid(360.0) / 360.0;
        sample_days.push(orbit.epoch_game_days + phase * orbit.period_game_days);
    }
}

fn jump_exclusion_radius_au(diameter_km: f64) -> f64 {
    diameter_km * 1_000.0 * JUMP_EXCLUSION_DIAMETERS / AU_METERS
}

fn first_safe_distance_on_ray(
    origin: [f64; 3],
    direction: [f64; 3],
    spheres: &[ExclusionSphere],
) -> f64 {
    let mut intervals = spheres
        .iter()
        .filter_map(|sphere| {
            let offset = subtract(origin, sphere.center_au);
            let projection = dot(offset, direction);
            let discriminant =
                projection * projection - (dot(offset, offset) - sphere.radius_au.powi(2));
            if discriminant < 0.0 {
                return None;
            }
            let root = discriminant.sqrt();
            let start = -projection - root;
            let end = -projection + root;
            (end >= 0.0).then_some((start.max(0.0), end))
        })
        .collect::<Vec<_>>();
    intervals.sort_by(|left, right| left.0.total_cmp(&right.0));

    let mut first_gap = 0.0;
    for (start, end) in intervals {
        if start > first_gap + 1e-12 {
            break;
        }
        first_gap = first_gap.max(end);
    }
    first_gap
}

fn star_positions(system: &CelestialSystem, game_days: f64) -> Vec<[f64; 3]> {
    let mut positions = vec![None; system.stars.len()];
    for _ in 0..system.stars.len() {
        let mut progressed = false;
        for (index, star) in system.stars.iter().enumerate() {
            if positions[index].is_some() {
                continue;
            }
            let position = match (star.parent_star_id, star.orbit) {
                (None, None) => Some([0.0; 3]),
                (Some(parent_id), Some(orbit)) => system
                    .stars
                    .iter()
                    .position(|parent| parent.id == parent_id)
                    .and_then(|parent_index| positions[parent_index])
                    .map(|parent| add(parent, orbit.position_au(game_days))),
                _ => None,
            };
            if let Some(position) = position {
                positions[index] = Some(position);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    positions
        .into_iter()
        .map(|position| position.expect("generated stellar orbit graph is complete"))
        .collect()
}

fn body_positions(
    system: &CelestialSystem,
    game_days: f64,
    star_positions: &[[f64; 3]],
) -> Vec<[f64; 3]> {
    let mut positions = vec![None; system.bodies.len()];
    for _ in 0..system.bodies.len() {
        let mut progressed = false;
        for (index, body) in system.bodies.iter().enumerate() {
            if positions[index].is_some() {
                continue;
            }
            let parent = body
                .parent_body_id
                .and_then(|parent_id| {
                    system
                        .bodies
                        .iter()
                        .position(|candidate| candidate.local_id == parent_id)
                        .and_then(|parent_index| positions[parent_index])
                })
                .or_else(|| {
                    system
                        .stars
                        .iter()
                        .position(|star| star.id == body.parent_star_id)
                        .map(|star_index| star_positions[star_index])
                });
            if let Some(parent) = parent {
                positions[index] = Some(add(parent, body.orbit.position_au(game_days)));
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    positions
        .into_iter()
        .map(|position| position.expect("generated body orbit graph is complete"))
        .collect()
}

fn push_direction(directions: &mut Vec<[f64; 3]>, direction: [f64; 3]) {
    let length = magnitude(direction);
    if length > 1e-12 {
        directions.push(direction.map(|value| value / length));
    }
}

fn add(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn subtract(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale(vector: [f64; 3], factor: f64) -> [f64; 3] {
    vector.map(|value| value * factor)
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn magnitude(vector: [f64; 3]) -> f64 {
    dot(vector, vector).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::celestial::derive_celestial_system;
    use crate::universe::{INITIAL_GENERATION_VERSION, SOL_SYSTEM_ID, StellarSystem};

    fn sol() -> CelestialSystem {
        derive_celestial_system(&StellarSystem {
            id: SOL_SYSTEM_ID,
            name: "Sol".into(),
            primary_world_name: "Earth".into(),
            position_parsecs: [0.0; 3],
            polity_id: 1,
            generation_seed: [0; 32],
            generation_version: INITIAL_GENERATION_VERSION,
        })
        .unwrap()
    }

    #[test]
    fn midpoint_turnover_math_round_trips() {
        let distance = rest_to_rest_distance_au(0.5, 1.0);
        assert!((rest_to_rest_travel_days(distance, 1.0) - 0.5).abs() < 1e-12);
        assert!((distance - 0.030_59).abs() < 0.000_01);
    }

    #[test]
    fn earth_has_at_least_the_half_day_jump_approach() {
        for thrust_g in [1.0, 2.0, 6.0] {
            for day in [0.0, 91.0, 182.0, 273.0] {
                let solution = primary_world_jump_safety(&sol(), day, thrust_g);
                assert!(solution.travel_days >= MINIMUM_JUMP_APPROACH_DAYS);
                assert!(solution.travel_days < 0.51);
            }
        }
    }
}
