//! Authoritative in-system geometry used to reach a safe Jump locus.

use crate::celestial::{BodyKind, CelestialSystem};

pub const AU_METERS: f64 = 149_597_870_700.0;
pub const STANDARD_GRAVITY_METERS_PER_SECOND_SQUARED: f64 = 9.806_65;
pub const SECONDS_PER_GAME_DAY: f64 = 86_400.0;
pub const JUMP_EXCLUSION_DIAMETERS: f64 = 100.0;
pub const MINIMUM_JUMP_APPROACH_DAYS: f64 = 0.5;
pub const BBS_CORE_MAXIMUM_JUMP_APPROACH_DAYS: f64 = 3.5;

const MAXIMUM_MANEUVER_SECONDS: u64 = 20 * 365 * 24 * 60 * 60;

const LOCUS_CLEARANCE_AU: f64 = 1e-10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpSafetySolution {
    /// Position in the system's seed-derived reference frame.
    pub locus_au: [f64; 3],
    pub distance_au: f64,
    /// Constant-thrust, midpoint-turnover time from the primary world.
    pub travel_days: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KinematicState {
    pub position_au: [f64; 3],
    pub velocity_au_per_second: [f64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ManeuverSolution {
    pub duration_seconds: u64,
    pub turnover_seconds: u64,
    pub first_acceleration_au_per_second_squared: [f64; 3],
    pub second_acceleration_au_per_second_squared: [f64; 3],
}

/// Find the shortest whole-second, two-burn intercept which starts at
/// `origin`, reaches the moving target with its velocity, and never commands
/// more than `thrust_g`. The target callback is evaluated at elapsed seconds,
/// so a common inertial-frame velocity cancels from the relative solution.
pub fn bounded_thrust_intercept<F>(
    origin: KinematicState,
    thrust_g: f64,
    target_at: F,
) -> Option<ManeuverSolution>
where
    F: Fn(u64) -> Option<KinematicState>,
{
    if !thrust_g.is_finite() || thrust_g <= 0.0 {
        return None;
    }
    let maximum_acceleration = thrust_g * STANDARD_GRAVITY_METERS_PER_SECOND_SQUARED / AU_METERS;
    let solve = |duration_seconds| {
        maneuver_for_duration(origin, target_at(duration_seconds)?, duration_seconds).filter(
            |solution| {
                magnitude(solution.first_acceleration_au_per_second_squared)
                    <= maximum_acceleration * (1.0 + 1.0e-12)
                    && magnitude(solution.second_acceleration_au_per_second_squared)
                        <= maximum_acceleration * (1.0 + 1.0e-12)
            },
        )
    };

    let mut upper = 2_u64;
    while upper < MAXIMUM_MANEUVER_SECONDS && solve(upper).is_none() {
        upper = upper.saturating_mul(2).min(MAXIMUM_MANEUVER_SECONDS);
    }
    solve(upper)?;
    let mut lower = 1_u64;
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        if solve(middle).is_some() {
            upper = middle;
        } else {
            lower = middle + 1;
        }
    }
    solve(lower)
}

pub fn maneuver_state_at(
    origin: KinematicState,
    solution: ManeuverSolution,
    elapsed_seconds: u64,
) -> KinematicState {
    let elapsed = elapsed_seconds.min(solution.duration_seconds);
    let first_seconds = solution.turnover_seconds;
    if elapsed <= first_seconds {
        return integrate_acceleration(
            origin,
            solution.first_acceleration_au_per_second_squared,
            elapsed as f64,
        );
    }
    let turnover = integrate_acceleration(
        origin,
        solution.first_acceleration_au_per_second_squared,
        first_seconds as f64,
    );
    integrate_acceleration(
        turnover,
        solution.second_acceleration_au_per_second_squared,
        (elapsed - first_seconds) as f64,
    )
}

fn maneuver_for_duration(
    origin: KinematicState,
    target: KinematicState,
    duration_seconds: u64,
) -> Option<ManeuverSolution> {
    let first_seconds = duration_seconds / 2;
    let second_seconds = duration_seconds - first_seconds;
    if first_seconds == 0 || second_seconds == 0 {
        return None;
    }
    let duration = duration_seconds as f64;
    let first = first_seconds as f64;
    let second = second_seconds as f64;
    let displacement: [f64; 3] = std::array::from_fn(|index| {
        target.position_au[index]
            - origin.position_au[index]
            - origin.velocity_au_per_second[index] * duration
    });
    let velocity_change: [f64; 3] = std::array::from_fn(|index| {
        target.velocity_au_per_second[index] - origin.velocity_au_per_second[index]
    });
    let first_acceleration = std::array::from_fn(|index| {
        (2.0 * displacement[index] - velocity_change[index] * second) / (first * duration)
    });
    let second_acceleration = std::array::from_fn(|index| {
        (velocity_change[index] - first_acceleration[index] * first) / second
    });
    first_acceleration
        .iter()
        .chain(&second_acceleration)
        .all(|value| value.is_finite())
        .then_some(ManeuverSolution {
            duration_seconds,
            turnover_seconds: first_seconds,
            first_acceleration_au_per_second_squared: first_acceleration,
            second_acceleration_au_per_second_squared: second_acceleration,
        })
}

fn integrate_acceleration(
    origin: KinematicState,
    acceleration: [f64; 3],
    seconds: f64,
) -> KinematicState {
    KinematicState {
        position_au: std::array::from_fn(|index| {
            origin.position_au[index]
                + origin.velocity_au_per_second[index] * seconds
                + 0.5 * acceleration[index] * seconds * seconds
        }),
        velocity_au_per_second: std::array::from_fn(|index| {
            origin.velocity_au_per_second[index] + acceleration[index] * seconds
        }),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrontierFuelSolution {
    pub body_local_id: u32,
    pub body_name: String,
    pub body_kind: FrontierFuelBodyKind,
    /// Rest-to-rest travel from the primary-world Jump locus to the nearest
    /// gas giant and back. Collection and processing time are separate.
    pub round_trip_days: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontierFuelBodyKind {
    GasGiant,
    Planet,
    Moon,
    IcyBelt,
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

fn primary_world_safety_on_ray(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
    direction: [f64; 3],
    minimum_distance_au: f64,
) -> JumpSafetySolution {
    assert!(thrust_g.is_finite() && thrust_g > 0.0);
    let direction_magnitude = magnitude(direction);
    assert!(direction_magnitude.is_finite() && direction_magnitude > 0.0);
    let direction = scale(direction, 1.0 / direction_magnitude);
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

    let distance_au =
        safe_distance_at_least_on_ray(origin, direction, &spheres, minimum_distance_au.max(0.0));
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

/// Find the conventional safe departure locus for the primary world.
///
/// The point is outside the 100-diameter exclusion spheres of every star and
/// generated body. It is also outside a thrust-scaled safety sphere whose
/// radius takes exactly half a game day to traverse at constant thrust with a
/// midpoint turnover. By traffic convention departures use the north side of
/// the system ecliptic.
pub fn primary_world_jump_safety(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
) -> JumpSafetySolution {
    primary_world_safety_on_ray(system, game_days, thrust_g, [0.0, 0.0, 1.0], 0.0)
}

/// Find the conventional arrival locus on the south side of the ecliptic.
pub fn primary_world_arrival_safety(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
) -> JumpSafetySolution {
    primary_world_safety_on_ray(system, game_days, thrust_g, [0.0, 0.0, -1.0], 0.0)
}

/// Find a private, deliberately distant arrival point. The seed selects an
/// unpredictable direction; the extra standoff makes the port approach longer
/// than an arrival through the conventional traffic locus.
pub fn primary_world_remote_arrival_safety(
    system: &CelestialSystem,
    game_days: f64,
    thrust_g: f64,
    seed: u64,
) -> JumpSafetySolution {
    fn mixed(mut value: u64) -> u64 {
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
    let component = |salt: u64| {
        let value = mixed(seed ^ salt) >> 11;
        value as f64 / ((1_u64 << 53) - 1) as f64 * 2.0 - 1.0
    };
    let mut direction = [
        component(0x434f_5245_5741_5244),
        component(0x5350_494e_5741_5244),
        component(0x4e4f_5254_4857_4152),
    ];
    if magnitude(direction) < 1.0e-6 {
        direction = [1.0, 1.0, 0.5];
    }
    let minimum_distance = primary_world_arrival_safety(system, game_days, thrust_g).distance_au
        + rest_to_rest_distance_au(MINIMUM_JUMP_APPROACH_DAYS, thrust_g);
    primary_world_safety_on_ray(system, game_days, thrust_g, direction, minimum_distance)
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
                    body_kind: FrontierFuelBodyKind::GasGiant,
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
                body_kind: if matches!(body.kind, BodyKind::PlanetoidBelt { .. }) {
                    FrontierFuelBodyKind::IcyBelt
                } else if body.parent_body_id.is_some() {
                    FrontierFuelBodyKind::Moon
                } else {
                    FrontierFuelBodyKind::Planet
                },
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

fn safe_distance_at_least_on_ray(
    origin: [f64; 3],
    direction: [f64; 3],
    spheres: &[ExclusionSphere],
    minimum_distance_au: f64,
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

    let mut distance = minimum_distance_au;
    for (start, end) in intervals {
        if distance + LOCUS_CLEARANCE_AU >= start && distance <= end + LOCUS_CLEARANCE_AU {
            distance = end + LOCUS_CLEARANCE_AU;
        }
    }
    distance
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

/// Resolve a generated body's barycentric position for physical observations
/// such as in-system radio propagation.
pub fn body_position_au(
    system: &CelestialSystem,
    game_days: f64,
    body_local_id: u32,
) -> Option<[f64; 3]> {
    let stars = star_positions(system, game_days);
    system
        .bodies
        .iter()
        .zip(body_positions(system, game_days, &stars))
        .find_map(|(body, position)| (body.local_id == body_local_id).then_some(position))
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
    fn bounded_intercept_matches_stationary_midpoint_turnover() {
        let origin = KinematicState {
            position_au: [0.0; 3],
            velocity_au_per_second: [0.0; 3],
        };
        let solution = bounded_thrust_intercept(origin, 1.0, |_| {
            Some(KinematicState {
                position_au: [1.0, 0.0, 0.0],
                velocity_au_per_second: [0.0; 3],
            })
        })
        .unwrap();
        let expected = rest_to_rest_travel_days(1.0, 1.0) * SECONDS_PER_GAME_DAY;
        assert!((solution.duration_seconds as f64 - expected).abs() <= 1.0);
        let arrival = maneuver_state_at(origin, solution, solution.duration_seconds);
        assert!((arrival.position_au[0] - 1.0).abs() < 1.0e-10);
        assert!(magnitude(arrival.velocity_au_per_second) < 1.0e-12);
    }

    #[test]
    fn bounded_intercept_depends_on_relative_not_common_velocity() {
        let relative_origin = KinematicState {
            position_au: [0.0; 3],
            velocity_au_per_second: [2.0e-6, 0.0, 0.0],
        };
        let relative = bounded_thrust_intercept(relative_origin, 1.0, |_| {
            Some(KinematicState {
                position_au: [0.5, 0.0, 0.0],
                velocity_au_per_second: [0.0; 3],
            })
        })
        .unwrap();
        let stopped = bounded_thrust_intercept(
            KinematicState {
                velocity_au_per_second: [0.0; 3],
                ..relative_origin
            },
            1.0,
            |_| {
                Some(KinematicState {
                    position_au: [0.5, 0.0, 0.0],
                    velocity_au_per_second: [0.0; 3],
                })
            },
        )
        .unwrap();
        assert_ne!(relative.duration_seconds, stopped.duration_seconds);
        let common_velocity = -7.0e-6;
        let shifted_origin = KinematicState {
            position_au: relative_origin.position_au,
            velocity_au_per_second: [
                relative_origin.velocity_au_per_second[0] + common_velocity,
                0.0,
                0.0,
            ],
        };
        let shifted = bounded_thrust_intercept(shifted_origin, 1.0, |elapsed| {
            Some(KinematicState {
                position_au: [0.5 + common_velocity * elapsed as f64, 0.0, 0.0],
                velocity_au_per_second: [common_velocity, 0.0, 0.0],
            })
        })
        .unwrap();
        assert_eq!(relative.duration_seconds, shifted.duration_seconds);
        for index in 0..3 {
            assert!(
                (relative.first_acceleration_au_per_second_squared[index]
                    - shifted.first_acceleration_au_per_second_squared[index])
                    .abs()
                    < 1.0e-18
            );
            assert!(
                (relative.second_acceleration_au_per_second_squared[index]
                    - shifted.second_acceleration_au_per_second_squared[index])
                    .abs()
                    < 1.0e-18
            );
        }
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

    #[test]
    fn conventional_loci_are_opposite_and_remote_arrival_is_farther() {
        let system = sol();
        let day = 73.0;
        let primary_id = system.primary_world_body().local_id;
        let primary = body_position_au(&system, day, primary_id).unwrap();
        let departure = primary_world_jump_safety(&system, day, 1.0);
        let arrival = primary_world_arrival_safety(&system, day, 1.0);
        let remote = primary_world_remote_arrival_safety(&system, day, 1.0, 0x1234_5678);

        assert!(departure.locus_au[2] > primary[2]);
        assert!(arrival.locus_au[2] < primary[2]);
        assert!(remote.travel_days > arrival.travel_days);
        assert_ne!(remote.locus_au, arrival.locus_au);
    }
}
