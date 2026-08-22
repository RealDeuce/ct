//! Initial real-neighborhood universe data and persistent record shapes.

use crate::crypto::{CryptoError, SeedStream};

pub const INITIAL_GENERATION_VERSION: u16 = 1;
pub const STELLAR_DISTRIBUTION_VERSION: u16 = 1;
pub const FEDERATION_POLITY_ID: u64 = 1;
pub const SOL_SYSTEM_ID: u64 = 1;
pub const EARTH_WORLD_ID: u64 = 1;

// Version 1 is a deliberately smooth, game-scale approximation of the
// Galactic stellar disk. Distances are parsecs and densities count stellar
// components, including brown dwarfs, rather than unresolved points of light.
pub const SOL_GALACTOCENTRIC_RADIUS_PARSECS: f64 = 8_178.0;
pub const SOL_HEIGHT_ABOVE_MIDPLANE_PARSECS: f64 = 20.8;
pub const LOCAL_COMPONENT_DENSITY_PER_CUBIC_PARSEC: f64 = 0.0906;
pub const DISK_RADIAL_SCALE_LENGTH_PARSECS: f64 = 2_200.0;
pub const THIN_DISK_SCALE_HEIGHT_PARSECS: f64 = 300.0;
pub const THICK_DISK_SCALE_HEIGHT_PARSECS: f64 = 900.0;
pub const THICK_DISK_MIDPLANE_FRACTION: f64 = 0.06;
pub const SPIRAL_ARM_COUNT: u8 = 4;
pub const SPIRAL_ARM_PITCH_RADIANS: f64 = 0.174_532_925_199_432_95;
pub const SPIRAL_ARM_WIDTH_PARSECS: f64 = 350.0;
pub const SPIRAL_ARM_PEAK_OVERDENSITY: f64 = 0.35;
pub const SPIRAL_ARM_SOLAR_PHASE_OFFSET_RADIANS: f64 = std::f64::consts::FRAC_PI_4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GalacticCylindricalPosition {
    pub radius_parsecs: f64,
    pub azimuth_radians: f64,
    pub height_parsecs: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpiralArmGeometry {
    pub arm_index: u8,
    pub signed_normal_distance_parsecs: f64,
    /// Unit vector in the game's coreward, spinward, north coordinate frame.
    pub spinward_tangent: [f64; 3],
}

/// Convert the Earth-centered game frame to a Galactic cylindrical position.
///
/// At Earth, Galactic radius is 8,178 pc and Galactic height is +20.8 pc.
/// Positive game X points coreward, so it subtracts from Galactic radius on
/// the Sun--Galactic-center line. Positive game Y increases azimuth in the
/// spinward direction.
pub fn galactic_cylindrical_position(position_parsecs: [f64; 3]) -> GalacticCylindricalPosition {
    let galactocentric_outward = SOL_GALACTOCENTRIC_RADIUS_PARSECS - position_parsecs[0];
    GalacticCylindricalPosition {
        radius_parsecs: galactocentric_outward.hypot(position_parsecs[1]),
        azimuth_radians: position_parsecs[1].atan2(galactocentric_outward),
        height_parsecs: position_parsecs[2] + SOL_HEIGHT_ABOVE_MIDPLANE_PARSECS,
    }
}

fn spiral_arm_phase(arm_index: u8) -> f64 {
    SPIRAL_ARM_SOLAR_PHASE_OFFSET_RADIANS
        + f64::from(arm_index) * std::f64::consts::TAU / f64::from(SPIRAL_ARM_COUNT)
}

fn wrap_angle(radians: f64) -> f64 {
    (radians + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

fn spiral_phase_coordinate(position: GalacticCylindricalPosition) -> f64 {
    if position.radius_parsecs <= f64::EPSILON {
        return 0.0;
    }
    position.azimuth_radians
        + (position.radius_parsecs / SOL_GALACTOCENTRIC_RADIUS_PARSECS).ln()
            / SPIRAL_ARM_PITCH_RADIANS.tan()
}

/// Radius of one repeated logarithmic arm at the requested Galactic azimuth.
pub fn spiral_arm_radius_parsecs(arm_index: u8, azimuth_radians: f64) -> f64 {
    assert!(arm_index < SPIRAL_ARM_COUNT);
    SOL_GALACTOCENTRIC_RADIUS_PARSECS
        * ((spiral_arm_phase(arm_index) - azimuth_radians) * SPIRAL_ARM_PITCH_RADIANS.tan()).exp()
}

fn arm_spinward_tangent(azimuth_radians: f64) -> [f64; 3] {
    let (sin_azimuth, cos_azimuth) = azimuth_radians.sin_cos();
    let (sin_pitch, cos_pitch) = SPIRAL_ARM_PITCH_RADIANS.sin_cos();

    // A trailing logarithmic spiral moves inward while moving spinward.
    // First form the vector in Galactocentric outward/spinward axes, then
    // negate the outward component to return the game's coreward component.
    let outward = -sin_pitch * cos_azimuth - cos_pitch * sin_azimuth;
    let spinward = -sin_pitch * sin_azimuth + cos_pitch * cos_azimuth;
    [-outward, spinward, 0.0]
}

/// Return the nearest repeated arm and its local direction.
///
/// The normal offset is the logarithmic-spiral phase separation projected
/// onto the arm normal. It is exact on the centerline and is the useful local
/// distance approximation within the arm-width region where its density
/// enhancement is non-negligible.
pub fn nearest_spiral_arm(position_parsecs: [f64; 3]) -> SpiralArmGeometry {
    let position = galactic_cylindrical_position(position_parsecs);
    if position.radius_parsecs <= f64::EPSILON {
        return SpiralArmGeometry {
            arm_index: 0,
            signed_normal_distance_parsecs: 0.0,
            spinward_tangent: [0.0, 1.0, 0.0],
        };
    }

    let phase = spiral_phase_coordinate(position);
    let mut nearest_index = 0;
    let mut nearest_delta = wrap_angle(phase - spiral_arm_phase(0));
    for arm_index in 1..SPIRAL_ARM_COUNT {
        let delta = wrap_angle(phase - spiral_arm_phase(arm_index));
        if delta.abs() < nearest_delta.abs() {
            nearest_index = arm_index;
            nearest_delta = delta;
        }
    }

    SpiralArmGeometry {
        arm_index: nearest_index,
        signed_normal_distance_parsecs: position.radius_parsecs
            * SPIRAL_ARM_PITCH_RADIANS.sin()
            * nearest_delta,
        spinward_tangent: arm_spinward_tangent(position.azimuth_radians),
    }
}

fn vertical_density_factor(height_parsecs: f64) -> f64 {
    let thin_fraction = 1.0 - THICK_DISK_MIDPLANE_FRACTION;
    thin_fraction * (-height_parsecs.abs() / THIN_DISK_SCALE_HEIGHT_PARSECS).exp()
        + THICK_DISK_MIDPLANE_FRACTION
            * (-height_parsecs.abs() / THICK_DISK_SCALE_HEIGHT_PARSECS).exp()
}

fn spiral_density_factor(position: GalacticCylindricalPosition) -> f64 {
    if position.radius_parsecs <= f64::EPSILON {
        return 1.0 + f64::from(SPIRAL_ARM_COUNT) * SPIRAL_ARM_PEAK_OVERDENSITY;
    }
    let phase = spiral_phase_coordinate(position);
    let mut enhancement = 0.0;
    for arm_index in 0..SPIRAL_ARM_COUNT {
        let delta = wrap_angle(phase - spiral_arm_phase(arm_index));
        let normal_distance = position.radius_parsecs * SPIRAL_ARM_PITCH_RADIANS.sin() * delta;
        enhancement += (-0.5 * (normal_distance / SPIRAL_ARM_WIDTH_PARSECS).powi(2)).exp();
    }
    1.0 + SPIRAL_ARM_PEAK_OVERDENSITY * enhancement
}

/// Expected stellar-component density for frontier materialization.
///
/// This continuous intensity is normalized to the observed local density at
/// Earth. It is not itself a random generator: new surveyed volumes draw an
/// inhomogeneous Poisson realization from the OS CSPRNG, then persist both
/// generated components and empty coverage.
pub fn stellar_component_density_per_cubic_parsec(position_parsecs: [f64; 3]) -> f64 {
    if !position_parsecs
        .iter()
        .all(|coordinate| coordinate.is_finite())
    {
        return 0.0;
    }
    let position = galactic_cylindrical_position(position_parsecs);
    let solar_position = galactic_cylindrical_position([0.0; 3]);
    let radial = ((SOL_GALACTOCENTRIC_RADIUS_PARSECS - position.radius_parsecs)
        / DISK_RADIAL_SCALE_LENGTH_PARSECS)
        .exp();
    let vertical = vertical_density_factor(position.height_parsecs)
        / vertical_density_factor(solar_position.height_parsecs);
    let spiral = spiral_density_factor(position) / spiral_density_factor(solar_position);
    LOCAL_COMPONENT_DENSITY_PER_CUBIC_PARSEC * radial * vertical * spiral
}

/// Conservative intensity bound for rejection-sampling a Sol-centered sphere.
///
/// The radial term can increase by at most `exp(radius / scale_length)`, the
/// vertical profile never exceeds its midplane value, and the four Gaussian
/// spiral terms can each contribute at most their peak. This deliberately
/// trades additional rejected candidates for a bound that does not depend on
/// a numerical grid search.
pub fn sol_centered_stellar_density_upper_bound(radius_parsecs: f64) -> f64 {
    if !radius_parsecs.is_finite() || radius_parsecs < 0.0 {
        return 0.0;
    }
    let solar_position = galactic_cylindrical_position([0.0; 3]);
    let radial_upper = (radius_parsecs / DISK_RADIAL_SCALE_LENGTH_PARSECS).exp();
    let vertical_upper = 1.0 / vertical_density_factor(solar_position.height_parsecs);
    let spiral_upper = (1.0 + f64::from(SPIRAL_ARM_COUNT) * SPIRAL_ARM_PEAK_OVERDENSITY)
        / spiral_density_factor(solar_position);
    LOCAL_COMPONENT_DENSITY_PER_CUBIC_PARSEC * radial_upper * vertical_upper * spiral_upper
}

/// CNS5 astrometry transformed into a heliocentric Galactic Cartesian frame.
///
/// X is positive coreward, Y spinward, and Z toward Galactic north. Distances
/// are parsecs. Each stellar component is a separate game system because it
/// can have its own planetary system. Where CNS5 only publishes a combined
/// position for a close multiple, its components share that position here;
/// their eventual local orbital separation belongs to the generated
/// planetary-system model rather than the interstellar map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InitialSystem {
    pub id: u64,
    pub name: &'static str,
    pub cns5_gj_id: &'static str,
    pub position_epoch: f64,
    pub icrs_ra_degrees: f64,
    pub icrs_dec_degrees: f64,
    pub parallax_mas: f64,
    pub position_parsecs: [f64; 3],
}

impl InitialSystem {
    pub fn primary_is_brown_dwarf(&self) -> bool {
        matches!(
            self.name,
            "Luhman 16 A"
                | "Luhman 16 B"
                | "WISE 0855-0714"
                | "Epsilon Indi Ba"
                | "Epsilon Indi Bb"
        )
    }
}

pub const INITIAL_SYSTEMS: &[InitialSystem] = &[
    InitialSystem {
        id: 1,
        name: "Sol",
        cns5_gj_id: "",
        position_epoch: 2016.0,
        icrs_ra_degrees: 0.0,
        icrs_dec_degrees: 0.0,
        parallax_mas: 0.0,
        position_parsecs: [0.0, 0.0, 0.0],
    },
    InitialSystem {
        id: 2,
        name: "Alpha Centauri A",
        cns5_gj_id: "559",
        position_epoch: 1991.25,
        icrs_ra_degrees: 219.9204081301028,
        icrs_dec_degrees: -60.83514521897498,
        parallax_mas: 754.8099975585938,
        position_parsecs: [0.948784353, -0.924527019, -0.015823159],
    },
    InitialSystem {
        id: 3,
        name: "Alpha Centauri B",
        cns5_gj_id: "559",
        position_epoch: 1991.25,
        icrs_ra_degrees: 219.9204081301028,
        icrs_dec_degrees: -60.83514521897498,
        parallax_mas: 754.8099975585938,
        position_parsecs: [0.948784353, -0.924527019, -0.015823159],
    },
    InitialSystem {
        id: 4,
        name: "Proxima Centauri",
        cns5_gj_id: "551",
        position_epoch: 2016.0,
        icrs_ra_degrees: 217.39232147200883,
        icrs_dec_degrees: -62.67607511676666,
        parallax_mas: 768.0665391873573,
        position_parsecs: [0.902700220, -0.937209265, -0.043570287],
    },
    InitialSystem {
        id: 5,
        name: "Barnard's Star",
        cns5_gj_id: "699",
        position_epoch: 2016.0,
        icrs_ra_degrees: 269.44850252543836,
        icrs_dec_degrees: 4.739420051112412,
        parallax_mas: 547.0241867292879,
        position_parsecs: [1.519055691, 0.914524887, 0.444931404],
    },
    InitialSystem {
        id: 6,
        name: "Luhman 16 A",
        cns5_gj_id: "11551",
        position_epoch: 2000.0,
        icrs_ra_degrees: 162.314706,
        icrs_dec_degrees: -53.3183847,
        parallax_mas: 500.510009765625,
        position_parsecs: [0.520960326832, -1.92003273872, 0.184192649836],
    },
    InitialSystem {
        id: 7,
        name: "Luhman 16 B",
        cns5_gj_id: "11551",
        position_epoch: 2000.0,
        icrs_ra_degrees: 162.314706,
        icrs_dec_degrees: -53.3183847,
        parallax_mas: 500.510009765625,
        position_parsecs: [0.520960327, -1.920032739, 0.184192650],
    },
    InitialSystem {
        id: 8,
        name: "WISE 0855-0714",
        cns5_gj_id: "11286",
        position_epoch: 2000.0,
        icrs_ra_degrees: 133.7947598,
        icrs_dec_degrees: -7.2451456,
        parallax_mas: 439.0,
        position_parsecs: [-1.199862997657, -1.712897300787, 0.902861978242],
    },
    InitialSystem {
        id: 9,
        name: "Wolf 359",
        cns5_gj_id: "406",
        position_epoch: 2016.0,
        icrs_ra_degrees: 164.10319030755974,
        icrs_dec_degrees: 7.002726940984864,
        parallax_mas: 415.17941567802137,
        position_parsecs: [-0.587839851, -1.207986195, 1.999138411],
    },
    InitialSystem {
        id: 10,
        name: "Lalande 21185",
        cns5_gj_id: "411",
        position_epoch: 2016.0,
        icrs_ra_degrees: 165.83095967577933,
        icrs_dec_degrees: 35.948653032660104,
        parallax_mas: 392.784297437213,
        position_parsecs: [-1.054141752, -0.095368831, 2.315476694],
    },
    InitialSystem {
        id: 11,
        name: "Sirius A",
        cns5_gj_id: "244",
        position_epoch: 1991.25,
        icrs_ra_degrees: 101.2885410520664,
        icrs_dec_degrees: -16.713143062644765,
        parallax_mas: 379.2099914550781,
        position_parsecs: [-1.769274969, -1.912527837, -0.407425763],
    },
    InitialSystem {
        id: 12,
        name: "Sirius B",
        cns5_gj_id: "244",
        position_epoch: 2016.0,
        icrs_ra_degrees: 101.28662552099249,
        icrs_dec_degrees: -16.720932526023173,
        parallax_mas: 379.2099914550781,
        position_parsecs: [-1.769040595, -1.912695801, -0.407654932],
    },
    InitialSystem {
        id: 13,
        name: "Luyten 726-8 A",
        cns5_gj_id: "65",
        position_epoch: 2016.0,
        icrs_ra_degrees: 24.771554293454546,
        icrs_dec_degrees: -17.948299887129313,
        parallax_mas: 369.92999267578125,
        position_parsecs: [-0.666247053, 0.052193851, -2.619304782],
    },
    InitialSystem {
        id: 14,
        name: "Luyten 726-8 B",
        cns5_gj_id: "65",
        position_epoch: 2016.0,
        icrs_ra_degrees: 24.771674208211856,
        icrs_dec_degrees: -17.947682860008488,
        parallax_mas: 369.92999267578125,
        position_parsecs: [-0.666268327, 0.052213603, -2.619298977],
    },
    InitialSystem {
        id: 15,
        name: "Ross 154",
        cns5_gj_id: "729",
        position_epoch: 2016.0,
        icrs_ra_degrees: 282.4587890175222,
        icrs_dec_degrees: -23.83709744872712,
        parallax_mas: 336.079124666674,
        position_parsecs: [2.870826114, 0.57404499, -0.531384222],
    },
    InitialSystem {
        id: 16,
        name: "Ross 248",
        cns5_gj_id: "905",
        position_epoch: 2016.0,
        icrs_ra_degrees: 355.4800152581559,
        icrs_dec_degrees: 44.170375700747755,
        parallax_mas: 316.5352087815091,
        position_parsecs: [-1.03306032, 2.839954115, -0.920885949],
    },
    InitialSystem {
        id: 17,
        name: "Epsilon Eridani",
        cns5_gj_id: "144.0",
        position_epoch: 2016.0,
        icrs_ra_degrees: 53.22829341517546,
        icrs_dec_degrees: -9.458168216292322,
        parallax_mas: 310.5772928005821,
        position_parsecs: [-2.070440323, -0.587485307, -2.394852177],
    },
    InitialSystem {
        id: 18,
        name: "Lacaille 9352",
        cns5_gj_id: "887",
        position_epoch: 2016.0,
        icrs_ra_degrees: 346.5039166796005,
        icrs_dec_degrees: -35.8471642082214,
        parallax_mas: 304.1626511996916,
        position_parsecs: [1.332588293, 0.118865306, -3.003189472],
    },
    InitialSystem {
        id: 19,
        name: "Ross 128",
        cns5_gj_id: "447",
        position_epoch: 2016.0,
        icrs_ra_degrees: 176.93768799004127,
        icrs_dec_degrees: 0.7991199702364985,
        parallax_mas: 296.3626809123856,
        position_parsecs: [0.004682454, -1.709726755, 2.909009301],
    },
    InitialSystem {
        id: 20,
        name: "EZ Aquarii A",
        cns5_gj_id: "866",
        position_epoch: 2016.0,
        icrs_ra_degrees: 339.650693945,
        icrs_dec_degrees: -15.28964705186,
        parallax_mas: 293.6,
        position_parsecs: [1.263412033, 1.359357716, -2.855999915],
    },
    InitialSystem {
        id: 21,
        name: "EZ Aquarii B",
        cns5_gj_id: "866",
        position_epoch: 2016.0,
        icrs_ra_degrees: 339.650693945,
        icrs_dec_degrees: -15.28964705186,
        parallax_mas: 293.6,
        position_parsecs: [1.263412033, 1.359357716, -2.855999915],
    },
    InitialSystem {
        id: 22,
        name: "EZ Aquarii C",
        cns5_gj_id: "866",
        position_epoch: 2016.0,
        icrs_ra_degrees: 339.650693945,
        icrs_dec_degrees: -15.28964705186,
        parallax_mas: 293.6,
        position_parsecs: [1.263412033, 1.359357716, -2.855999915],
    },
    InitialSystem {
        id: 23,
        name: "61 Cygni A",
        cns5_gj_id: "820",
        position_epoch: 2016.0,
        icrs_ra_degrees: 316.7484792940004,
        icrs_dec_degrees: 38.76386244649797,
        parallax_mas: 285.99494829578117,
        position_parsecs: [0.463489122, 3.447511699, -0.354696376],
    },
    InitialSystem {
        id: 24,
        name: "61 Cygni B",
        cns5_gj_id: "820",
        position_epoch: 2016.0,
        icrs_ra_degrees: 316.753662752556,
        icrs_dec_degrees: 38.75607277205679,
        parallax_mas: 286.0053518616485,
        position_parsecs: [0.463651617, 3.447310807, -0.355182413],
    },
    InitialSystem {
        id: 25,
        name: "Procyon A",
        cns5_gj_id: "280.0",
        position_epoch: 1991.25,
        icrs_ra_degrees: 114.82724201808722,
        icrs_dec_degrees: 5.227507577481227,
        parallax_mas: 284.55999755859375,
        position_parsecs: [-2.848440395, -1.899725553, 0.791841788],
    },
    InitialSystem {
        id: 26,
        name: "Procyon B",
        cns5_gj_id: "280.0",
        position_epoch: 1991.25,
        icrs_ra_degrees: 114.82724201808722,
        icrs_dec_degrees: 5.227507577481227,
        parallax_mas: 284.55999755859375,
        position_parsecs: [-2.848440395, -1.899725553, 0.791841788],
    },
    InitialSystem {
        id: 27,
        name: "Struve 2398 A",
        cns5_gj_id: "725",
        position_epoch: 2016.0,
        icrs_ra_degrees: 280.6830708352289,
        icrs_dec_degrees: 59.638357907754816,
        parallax_mas: 283.860076019069,
        position_parsecs: [0.039499343, 3.212068541, 1.446241771],
    },
    InitialSystem {
        id: 28,
        name: "Struve 2398 B",
        cns5_gj_id: "725",
        position_epoch: 2016.0,
        icrs_ra_degrees: 280.68308624583415,
        icrs_dec_degrees: 59.635145454818776,
        parallax_mas: 283.85911584457847,
        position_parsecs: [0.039692225, 3.212095126, 1.446206471],
    },
    InitialSystem {
        id: 29,
        name: "Groombridge 34 A",
        cns5_gj_id: "15",
        position_epoch: 2016.0,
        icrs_ra_degrees: 4.613226257557736,
        icrs_dec_degrees: 44.02478674398518,
        parallax_mas: 280.7411265779381,
        position_parsecs: [-1.517742398, 3.018929662, -1.127106219],
    },
    InitialSystem {
        id: 30,
        name: "Groombridge 34 B",
        cns5_gj_id: "15",
        position_epoch: 2016.0,
        icrs_ra_degrees: 4.625300681217554,
        icrs_dec_degrees: 44.02874451664357,
        parallax_mas: 280.74451904422403,
        position_parsecs: [-1.518255769, 3.018686457, -1.126930158],
    },
    InitialSystem {
        id: 31,
        name: "DX Cancri",
        cns5_gj_id: "1111",
        position_epoch: 2016.0,
        icrs_ra_degrees: 127.45009240230564,
        icrs_dec_degrees: 26.77328596508202,
        parallax_mas: 279.2596271009328,
        position_parsecs: [-2.890604167, -0.884629311, 1.919547566],
    },
    InitialSystem {
        id: 32,
        name: "Epsilon Indi A",
        cns5_gj_id: "845",
        position_epoch: 2016.0,
        icrs_ra_degrees: 330.8724078795965,
        icrs_dec_degrees: -56.79725466122902,
        parallax_mas: 274.8431415216296,
        position_parsecs: [2.224568305, -0.982746429, -2.706241588],
    },
    InitialSystem {
        id: 33,
        name: "Epsilon Indi Ba",
        cns5_gj_id: "845",
        position_epoch: 2016.0,
        icrs_ra_degrees: 331.07645259656397,
        icrs_dec_degrees: -56.79381206688221,
        parallax_mas: 274.8431415216296,
        position_parsecs: [2.218970845, -0.983597322, -2.710524473],
    },
    InitialSystem {
        id: 34,
        name: "Epsilon Indi Bb",
        cns5_gj_id: "845",
        position_epoch: 2016.0,
        icrs_ra_degrees: 331.07645259656397,
        icrs_dec_degrees: -56.79381206688221,
        parallax_mas: 274.8431415216296,
        position_parsecs: [2.218970845, -0.983597322, -2.710524473],
    },
    InitialSystem {
        id: 35,
        name: "Tau Ceti",
        cns5_gj_id: "71.0",
        position_epoch: 2016.0,
        icrs_ra_degrees: 26.009055057160104,
        icrs_dec_degrees: -15.933680200693857,
        parallax_mas: 273.9599914550781,
        position_parsecs: [-1.032621565, 0.125467384, -3.49881083],
    },
    InitialSystem {
        id: 36,
        name: "YZ Ceti",
        cns5_gj_id: "54.1",
        position_epoch: 2016.0,
        icrs_ra_degrees: 18.13325425370941,
        icrs_dec_degrees: -16.996154872479234,
        parallax_mas: 269.11083964987915,
        position_parsecs: [-0.625799008002, 0.365334508614, -3.644602000431],
    },
    InitialSystem {
        id: 37,
        name: "Ross 614 A",
        cns5_gj_id: "234",
        position_epoch: 1991.25,
        icrs_ra_degrees: 97.3458115916297,
        icrs_dec_degrees: -2.812476748665614,
        parallax_mas: 242.32000732421875,
        position_parsecs: [-3.443696941854, -2.230105768120, -0.444799378921],
    },
    InitialSystem {
        id: 38,
        name: "Ross 614 B",
        cns5_gj_id: "234",
        position_epoch: 1991.25,
        icrs_ra_degrees: 97.3458115916297,
        icrs_dec_degrees: -2.812476748665614,
        parallax_mas: 242.32000732421875,
        position_parsecs: [-3.443696941854, -2.230105768120, -0.444799378921],
    },
    InitialSystem {
        id: 39,
        name: "Gliese 1",
        cns5_gj_id: "1",
        position_epoch: 2016.0,
        icrs_ra_degrees: 1.3832841523481234,
        icrs_dec_degrees: -37.36774402806293,
        parallax_mas: 230.1189580289879,
        position_parsecs: [1.013804578259, -0.301222652462, -4.214916111165],
    },
    InitialSystem {
        id: 40,
        name: "40 Eridani A",
        cns5_gj_id: "166",
        position_epoch: 2016.0,
        icrs_ra_degrees: 63.80795300602079,
        icrs_dec_degrees: -7.6680778150199815,
        parallax_mas: 199.6080124181338,
        position_parsecs: [-3.688169090186, -1.398327810751, -3.088749560258],
    },
    InitialSystem {
        id: 41,
        name: "40 Eridani B",
        cns5_gj_id: "166",
        position_epoch: 2016.0,
        icrs_ra_degrees: 63.830787524383176,
        icrs_dec_degrees: -7.672952135407352,
        parallax_mas: 199.6080124181338,
        position_parsecs: [-3.688583843739, -1.399898770782, -3.087542454670],
    },
    InitialSystem {
        id: 42,
        name: "40 Eridani C",
        cns5_gj_id: "166",
        position_epoch: 2016.0,
        icrs_ra_degrees: 63.82965602087436,
        icrs_dec_degrees: -7.670903382610344,
        parallax_mas: 199.6080124181338,
        position_parsecs: [-3.688656125816, -1.399707773995, -3.087542693311],
    },
    InitialSystem {
        id: 43,
        name: "Gliese 205",
        cns5_gj_id: "205",
        position_epoch: 2016.0,
        icrs_ra_degrees: 82.86754111476276,
        icrs_dec_degrees: -3.6865276019053095,
        parallax_mas: 175.34215555112795,
        position_parsecs: [-4.793911929471, -2.436834618823, -1.898945839430],
    },
];

// Version-1 convex hull of the fixed CNS5 starting catalogue. These are the
// hull vertices, outward face normals, and vertex-index edges used to mark the
// catalogued volume on the quarter-parsec coverage lattice. Regenerate them
// whenever INITIAL_SYSTEMS astrometry changes.
pub const INITIAL_CATALOG_HULL_VERTICES: &[[f64; 3]] = &[
    [0.520960327000, -1.920032739000, 0.184192650000],
    [-1.054141752000, -0.095368831000, 2.315476694000],
    [2.870826114000, 0.574044990000, -0.531384222000],
    [0.004682454000, -1.709726755000, 2.909009301000],
    [1.263412033000, 1.359357716000, -2.855999915000],
    [0.463489122000, 3.447511699000, -0.354696376000],
    [0.463651617000, 3.447310807000, -0.355182413000],
    [-2.848440395000, -1.899725553000, 0.791841788000],
    [0.039499343000, 3.212068541000, 1.446241771000],
    [0.039692225000, 3.212095126000, 1.446206471000],
    [-1.517742398000, 3.018929662000, -1.127106219000],
    [-1.518255769000, 3.018686457000, -1.126930158000],
    [-2.890604167000, -0.884629311000, 1.919547566000],
    [2.224568305000, -0.982746429000, -2.706241588000],
    [2.218970845000, -0.983597322000, -2.710524473000],
    [1.013804578259, -0.301222652462, -4.214916111165],
    [-0.625799008002, 0.365334508614, -3.644602000431],
    [-3.688169090186, -1.398327810751, -3.088749560258],
    [-4.793911929471, -2.436834618823, -1.898945839430],
    [-3.443696941854, -2.230105768120, -0.444799378921],
    [-3.688583843739, -1.399898770782, -3.087542454670],
];

pub const INITIAL_CATALOG_HULL_FACE_NORMALS: &[[f64; 3]] = &[
    [0.754889767506, -0.618431564534, 0.218366295255],
    [0.827793384879, 0.309803014556, -0.467739461797],
    [0.805670970670, 0.297893684581, -0.512009413689],
    [-0.571547354120, -0.620392565171, 0.537072329467],
    [-0.402462484958, -0.687821870030, 0.604090244343],
    [-0.237199025736, 0.968876114699, 0.070821582555],
    [-0.848508208873, 0.477061970730, 0.229010252079],
    [-0.802046649224, 0.522513238883, -0.289311402577],
    [-0.490986597004, 0.543572138317, -0.680780061405],
    [-0.724520472731, 0.563980281710, 0.396227619477],
    [-0.377312800488, 0.912183159477, 0.159865362582],
    [0.766223455158, 0.642505513221, -0.009395863311],
    [0.724765166428, 0.624981774881, -0.290022817377],
    [-0.044807526769, 0.917863909661, -0.394357869055],
    [0.075372772781, 0.777169568585, -0.624761079765],
    [0.696169613610, -0.693514934253, 0.185431672198],
    [0.122255327080, -0.863362396627, -0.489549800419],
    [0.584154802296, -0.453020467124, -0.673450535171],
    [0.175266173240, -0.962198603110, -0.208460108152],
    [0.262669726426, -0.952508057651, -0.154055233372],
    [-0.251065329184, -0.898208638739, 0.360814968892],
    [-0.188883943968, -0.922250923913, 0.337307113847],
    [0.064916532931, -0.994588138348, 0.081118917701],
    [0.063656144057, -0.994015017123, 0.088780859753],
    [-0.275875044682, 0.167071022297, 0.946562324008],
    [-0.101327700413, 0.284078630111, 0.953431711788],
    [-0.336099087792, 0.341784726997, 0.877622130291],
    [0.681250827112, 0.204097920283, 0.703023007799],
    [0.135084272487, 0.281396969060, 0.950035780974],
    [0.748669754813, 0.611473350756, 0.256113138169],
    [-0.117588653089, 0.987865393686, 0.101463651729],
    [0.024086518091, -0.764315529900, -0.644392435090],
    [-0.368307201514, -0.503368647091, -0.781645578532],
    [-0.051237772129, -0.599949448085, -0.798395484988],
    [-0.265243188015, 0.160943145992, -0.950654172120],
    [-0.439534839768, 0.536038612662, -0.720743872930],
    [-0.011039277440, 0.634264941674, -0.773036944859],
    [-0.050917366160, 0.678304552440, -0.733014567360],
];

pub const INITIAL_CATALOG_HULL_EDGES: &[[usize; 2]] = &[
    [0, 3],
    [0, 13],
    [0, 14],
    [0, 18],
    [0, 19],
    [1, 3],
    [1, 8],
    [1, 12],
    [2, 3],
    [2, 4],
    [2, 5],
    [2, 6],
    [2, 9],
    [2, 13],
    [3, 7],
    [3, 8],
    [3, 9],
    [3, 12],
    [3, 13],
    [3, 19],
    [4, 6],
    [4, 10],
    [4, 13],
    [4, 15],
    [4, 16],
    [5, 6],
    [5, 8],
    [5, 9],
    [5, 10],
    [6, 10],
    [7, 12],
    [7, 18],
    [7, 19],
    [8, 9],
    [8, 10],
    [8, 11],
    [8, 12],
    [10, 11],
    [10, 16],
    [10, 17],
    [11, 12],
    [11, 17],
    [11, 18],
    [12, 18],
    [13, 14],
    [13, 15],
    [14, 15],
    [14, 18],
    [15, 16],
    [15, 17],
    [15, 18],
    [15, 20],
    [16, 17],
    [17, 18],
    [17, 20],
    [18, 19],
    [18, 20],
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Polity {
    pub id: u64,
    pub name: String,
    pub naming_profile_id: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StellarSystem {
    pub id: u64,
    pub name: String,
    /// Mutable/display name of the CE primary world. Its physical baseline is
    /// derived from the system seed and is not persisted as a world record.
    pub primary_world_name: String,
    pub position_parsecs: [f64; 3],
    pub polity_id: u64,
    pub generation_seed: [u8; 32],
    pub generation_version: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    pub id: u64,
    pub system_id: u64,
    pub name: String,
    pub starport: Starport,
    pub size: u8,
    pub atmosphere: u8,
    pub hydrographics: u8,
    pub population: u8,
    pub population_multiplier: u8,
    pub government: u8,
    pub law_level: u8,
    pub tech_level: u8,
    pub planetoid_belts: u8,
    pub gas_giants: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Starport {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    X = 5,
}

impl Starport {
    pub fn code(self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
            Self::E => 'E',
            Self::X => 'X',
        }
    }

    pub fn from_record(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::C),
            3 => Some(Self::D),
            4 => Some(Self::E),
            5 => Some(Self::X),
            _ => None,
        }
    }
}

impl World {
    pub fn is_agricultural(&self) -> bool {
        (4..=9).contains(&self.atmosphere)
            && (4..=8).contains(&self.hydrographics)
            && (5..=7).contains(&self.population)
    }

    pub fn is_industrial(&self) -> bool {
        matches!(self.atmosphere, 0..=2 | 4 | 7 | 9) && self.population >= 9
    }

    pub fn is_inhabited(&self) -> bool {
        self.population > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseInitialization {
    pub universe_id: [u8; 16],
    pub polity_count: u32,
    pub system_count: u32,
    pub world_count: u32,
    pub committed_sequence: u64,
}

struct SeedDice {
    stream: SeedStream,
    block: [u8; 32],
    offset: usize,
}

impl SeedDice {
    fn new(seed: [u8; 32]) -> Self {
        Self {
            stream: SeedStream::new(seed),
            block: [0; 32],
            offset: 32,
        }
    }

    fn byte(&mut self) -> Result<u8, CryptoError> {
        if self.offset == self.block.len() {
            self.block = self.stream.next_seed()?;
            self.offset = 0;
        }
        let result = self.block[self.offset];
        self.offset += 1;
        Ok(result)
    }

    fn die(&mut self) -> Result<u8, CryptoError> {
        loop {
            let value = self.byte()?;
            if value < 252 {
                return Ok(value % 6 + 1);
            }
        }
    }

    fn two_dice(&mut self) -> Result<i16, CryptoError> {
        Ok(i16::from(self.die()?) + i16::from(self.die()?))
    }
}

fn clamp_code(value: i16, maximum: u8) -> u8 {
    value.clamp(0, i16::from(maximum)) as u8
}

fn generate_starport(roll: i16) -> Starport {
    match roll {
        i16::MIN..=2 => Starport::X,
        3..=4 => Starport::E,
        5..=6 => Starport::D,
        7..=8 => Starport::C,
        9..=10 => Starport::B,
        _ => Starport::A,
    }
}

fn technology_dm(
    starport: Starport,
    size: u8,
    atmosphere: u8,
    hydrographics: u8,
    population: u8,
    government: u8,
) -> i16 {
    let starport_dm = match starport {
        Starport::A => 6,
        Starport::B => 4,
        Starport::C => 2,
        Starport::X => -4,
        Starport::D | Starport::E => 0,
    };
    let size_dm = match size {
        0..=1 => 2,
        2..=4 => 1,
        _ => 0,
    };
    let atmosphere_dm = if matches!(atmosphere, 0..=3 | 10..=15) {
        1
    } else {
        0
    };
    let hydrographics_dm = match hydrographics {
        0 | 9 => 1,
        10 => 2,
        _ => 0,
    };
    let population_dm = match population {
        1..=5 | 9 => 1,
        10 => 2,
        _ => 0,
    };
    let government_dm = match government {
        0 | 5 => 1,
        7 => 2,
        13..=14 => -2,
        _ => 0,
    };
    starport_dm + size_dm + atmosphere_dm + hydrographics_dm + population_dm + government_dm
}

fn minimum_technology(atmosphere: u8, hydrographics: u8, population: u8) -> u8 {
    let mut minimum = 0;
    if matches!(hydrographics, 0 | 10) && population >= 6 {
        minimum = minimum.max(4);
    }
    if matches!(atmosphere, 4 | 7 | 9) {
        minimum = minimum.max(5);
    }
    if atmosphere <= 3 || (10..=12).contains(&atmosphere) {
        minimum = minimum.max(7);
    }
    if matches!(atmosphere, 13 | 14) && hydrographics == 10 {
        minimum = minimum.max(7);
    }
    minimum
}

/// Generate the CE primary-world data determined by a system generation seed.
///
/// This is the version-1 construction-table order from the core rules. A
/// polity materializer may reject the seed and draw another one when it needs
/// a world with a constrained role; accepted seeds remain the authoritative
/// source for later deterministic expansion of the planetary system.
pub fn generate_primary_world(
    id: u64,
    system_id: u64,
    name: String,
    generation_seed: [u8; 32],
) -> Result<World, CryptoError> {
    let mut dice = SeedDice::new(generation_seed);
    let size = clamp_code(dice.two_dice()? - 2, 10);
    let atmosphere = if size == 0 {
        0
    } else {
        clamp_code(dice.two_dice()? - 7 + i16::from(size), 15)
    };
    let hydrographics = if size <= 1 {
        0
    } else {
        let atmosphere_dm = if matches!(atmosphere, 0 | 1 | 10 | 11 | 12) {
            -4
        } else if atmosphere == 14 {
            -2
        } else {
            0
        };
        clamp_code(dice.two_dice()? - 7 + i16::from(size) + atmosphere_dm, 10)
    };
    let mut population_dm = 0;
    if size <= 2 {
        population_dm -= 1;
    }
    if atmosphere >= 10 {
        population_dm -= 2;
    } else if atmosphere == 6 {
        population_dm += 3;
    } else if matches!(atmosphere, 5 | 8) {
        population_dm += 1;
    }
    if hydrographics == 0 && atmosphere < 3 {
        population_dm -= 2;
    }
    let population = clamp_code(dice.two_dice()? - 2 + population_dm, 10);
    let population_multiplier = if population == 0 {
        0
    } else {
        clamp_code((dice.two_dice()? - 2).max(1), 10)
    };
    let starport = generate_starport(dice.two_dice()? - 7 + i16::from(population));
    let government = if population == 0 {
        0
    } else {
        clamp_code(dice.two_dice()? - 7 + i16::from(population), 15)
    };
    let law_level = if government == 0 {
        0
    } else {
        clamp_code(dice.two_dice()? - 7 + i16::from(government), 15)
    };
    let tech_level = if population == 0 {
        0
    } else {
        let rolled = i16::from(dice.die()?)
            + technology_dm(
                starport,
                size,
                atmosphere,
                hydrographics,
                population,
                government,
            );
        clamp_code(rolled, u8::MAX).max(minimum_technology(atmosphere, hydrographics, population))
    };
    let planetoid_belts = if size == 0 || dice.two_dice()? >= 4 {
        i16::from(dice.die()?).saturating_sub(3).max(1) as u8
    } else {
        0
    };
    let gas_giants = if dice.two_dice()? >= 5 {
        i16::from(dice.die()?).saturating_sub(2).max(1) as u8
    } else {
        0
    };
    Ok(World {
        id,
        system_id,
        name,
        starport,
        size,
        atmosphere,
        hydrographics,
        population,
        population_multiplier,
        government,
        law_level,
        tech_level,
        planetoid_belts,
        gas_giants,
    })
}

pub fn distance_parsecs(first: &InitialSystem, second: &InitialSystem) -> f64 {
    first
        .position_parsecs
        .iter()
        .zip(second.position_parsecs)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_catalog_has_complete_nearby_prefix_and_hull_supplements() {
        assert_eq!(INITIAL_SYSTEMS.len(), 43);
        assert_eq!(INITIAL_SYSTEMS.first().unwrap().name, "Sol");
        assert_eq!(INITIAL_SYSTEMS[34].name, "Tau Ceti");
        assert_eq!(INITIAL_SYSTEMS.last().unwrap().name, "Gliese 205");
        assert!(
            INITIAL_SYSTEMS
                .iter()
                .enumerate()
                .all(|(index, system)| system.id == index as u64 + 1)
        );
        let tau_distance = 1000.0 / INITIAL_SYSTEMS[34].parallax_mas;
        for system in &INITIAL_SYSTEMS[1..35] {
            let distance = system
                .position_parsecs
                .iter()
                .map(|coordinate| coordinate.powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(distance <= tau_distance + 1e-8, "{}", system.name);
        }
        assert_eq!(
            &INITIAL_SYSTEMS[1..4]
                .iter()
                .map(|system| system.name)
                .collect::<Vec<_>>(),
            &["Alpha Centauri A", "Alpha Centauri B", "Proxima Centauri"]
        );
        assert_eq!(
            &INITIAL_SYSTEMS[35..]
                .iter()
                .map(|system| system.name)
                .collect::<Vec<_>>(),
            &[
                "YZ Ceti",
                "Ross 614 A",
                "Ross 614 B",
                "Gliese 1",
                "40 Eridani A",
                "40 Eridani B",
                "40 Eridani C",
                "Gliese 205",
            ]
        );
    }

    #[test]
    fn jump_two_edges_are_symmetric_and_inside_range() {
        let mut edge_count = 0;
        for (index, system) in INITIAL_SYSTEMS.iter().enumerate() {
            for other in &INITIAL_SYSTEMS[index + 1..] {
                if distance_parsecs(system, other) <= 2.0 {
                    edge_count += 1;
                }
            }
        }
        assert_eq!(edge_count, 90);
    }

    #[test]
    fn initial_catalog_has_one_sol_jump_two_network_and_one_jump_three_island() {
        let mut component = vec![false; INITIAL_SYSTEMS.len()];
        component[0] = true;
        let mut frontier = vec![0];
        while let Some(index) = frontier.pop() {
            for (other, system) in INITIAL_SYSTEMS.iter().enumerate() {
                if !component[other]
                    && distance_parsecs(&INITIAL_SYSTEMS[index], system) <= 2.0 + 1.0e-9
                {
                    component[other] = true;
                    frontier.push(other);
                }
            }
        }
        assert_eq!(component.iter().filter(|reachable| **reachable).count(), 36);
        let island = INITIAL_SYSTEMS
            .iter()
            .enumerate()
            .filter(|(index, _)| !component[*index])
            .map(|(_, system)| system.name)
            .collect::<Vec<_>>();
        assert_eq!(
            island,
            [
                "Ross 248",
                "61 Cygni A",
                "61 Cygni B",
                "Struve 2398 A",
                "Struve 2398 B",
                "Groombridge 34 A",
                "Groombridge 34 B",
            ]
        );
        let closest_crossing = INITIAL_SYSTEMS
            .iter()
            .enumerate()
            .filter(|(index, _)| component[*index])
            .flat_map(|(_, connected)| {
                INITIAL_SYSTEMS
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !component[*index])
                    .map(move |(_, disconnected)| distance_parsecs(connected, disconnected))
            })
            .min_by(f64::total_cmp)
            .unwrap();
        assert!(closest_crossing > 2.0);
        assert!(closest_crossing <= 3.0);
        assert!((closest_crossing - 2.858157517724).abs() < 1.0e-9);
    }

    #[test]
    fn stellar_density_is_normalized_at_sol() {
        let density = stellar_component_density_per_cubic_parsec([0.0; 3]);
        assert!((density - LOCAL_COMPONENT_DENSITY_PER_CUBIC_PARSEC).abs() < 1e-12);
    }

    #[test]
    fn sol_centered_density_bound_covers_a_local_grid() {
        let radius = 90.0;
        let upper = sol_centered_stellar_density_upper_bound(radius);
        for x in -9..=9 {
            for y in -9..=9 {
                for z in -9..=9 {
                    let position = [x as f64 * 10.0, y as f64 * 10.0, z as f64 * 10.0];
                    if position
                        .iter()
                        .map(|value| value * value)
                        .sum::<f64>()
                        .sqrt()
                        <= radius
                    {
                        assert!(stellar_component_density_per_cubic_parsec(position) <= upper);
                    }
                }
            }
        }
    }

    #[test]
    fn disk_density_increases_coreward_and_falls_off_vertically() {
        let local = stellar_component_density_per_cubic_parsec([0.0; 3]);
        let coreward = stellar_component_density_per_cubic_parsec([1_000.0, 0.0, 0.0]);
        let rimward = stellar_component_density_per_cubic_parsec([-1_000.0, 0.0, 0.0]);
        let north = stellar_component_density_per_cubic_parsec([0.0, 0.0, 600.0]);
        assert!(coreward > local);
        assert!(rimward < local);
        assert!(north < local);
    }

    #[test]
    fn vertical_density_is_symmetric_about_the_galactic_midplane() {
        let north = stellar_component_density_per_cubic_parsec([
            0.0,
            0.0,
            100.0 - SOL_HEIGHT_ABOVE_MIDPLANE_PARSECS,
        ]);
        let south = stellar_component_density_per_cubic_parsec([
            0.0,
            0.0,
            -100.0 - SOL_HEIGHT_ABOVE_MIDPLANE_PARSECS,
        ]);
        assert!((north - south).abs() < 1e-12);
    }

    #[test]
    fn repeated_logarithmic_arms_have_curved_unit_tangents() {
        let azimuth = 0.6;
        for arm_index in 0..SPIRAL_ARM_COUNT {
            let radius = spiral_arm_radius_parsecs(arm_index, azimuth);
            let position = [
                SOL_GALACTOCENTRIC_RADIUS_PARSECS - radius * azimuth.cos(),
                radius * azimuth.sin(),
                -SOL_HEIGHT_ABOVE_MIDPLANE_PARSECS,
            ];
            let arm = nearest_spiral_arm(position);
            assert_eq!(arm.arm_index, arm_index);
            assert!(arm.signed_normal_distance_parsecs.abs() < 1e-8);
            let tangent_length = arm
                .spinward_tangent
                .iter()
                .map(|component| component.powi(2))
                .sum::<f64>()
                .sqrt();
            assert!((tangent_length - 1.0).abs() < 1e-12);
            assert!(arm.spinward_tangent[2].abs() < f64::EPSILON);
        }

        let first = arm_spinward_tangent(0.0);
        let turned = arm_spinward_tangent(std::f64::consts::FRAC_PI_2);
        assert_ne!(first, turned);
    }

    #[test]
    fn invalid_coordinates_have_no_generation_density() {
        assert_eq!(
            stellar_component_density_per_cubic_parsec([f64::NAN, 0.0, 0.0]),
            0.0
        );
    }

    #[test]
    fn primary_world_generation_is_seed_stable_and_respects_uwp_bounds() {
        for value in 0_u8..=127 {
            let seed = [value; 32];
            let first = generate_primary_world(7, 9, "Primary".into(), seed).unwrap();
            let second = generate_primary_world(7, 9, "Primary".into(), seed).unwrap();
            assert_eq!(first, second);
            assert!(first.size <= 10);
            assert!(first.atmosphere <= 15);
            assert!(first.hydrographics <= 10);
            assert!(first.population <= 10);
            assert!(first.government <= 15);
            assert!(first.law_level <= 15);
            if first.population == 0 {
                assert_eq!(first.population_multiplier, 0);
                assert_eq!(first.government, 0);
                assert_eq!(first.law_level, 0);
                assert_eq!(first.tech_level, 0);
            } else {
                assert!(first.population_multiplier >= 1);
            }
            if first.size <= 1 {
                assert_eq!(first.hydrographics, 0);
            }
        }
    }

    #[test]
    fn ce_trade_code_predicates_identify_complementary_worlds() {
        let agricultural = World {
            id: 1,
            system_id: 1,
            name: "Agricultural".into(),
            starport: Starport::C,
            size: 7,
            atmosphere: 6,
            hydrographics: 6,
            population: 6,
            population_multiplier: 4,
            government: 4,
            law_level: 4,
            tech_level: 9,
            planetoid_belts: 0,
            gas_giants: 1,
        };
        let industrial = World {
            atmosphere: 4,
            hydrographics: 2,
            population: 9,
            name: "Industrial".into(),
            ..agricultural.clone()
        };
        assert!(agricultural.is_agricultural());
        assert!(!agricultural.is_industrial());
        assert!(industrial.is_industrial());
        assert!(!industrial.is_agricultural());
    }
}
