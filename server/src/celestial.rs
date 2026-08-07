//! Deterministic, non-persistent stellar and planetary-system construction.
//!
//! Core CE owns the primary-world profile.  The surrounding system adapts the
//! Open Game Content construction procedure in Unmerciful Frontier, third
//! edition, pp. 44--149.  Hex occupancy and Zimm Points are deliberately not
//! represented.  All random choices use named child streams of the persisted
//! system seed, so adding one feature cannot shift another.

use std::f64::consts::TAU;

use crate::crypto::{CryptoError, SeedStream, derive_seed};
use crate::universe::{
    EARTH_WORLD_ID, INITIAL_SYSTEMS, SOL_SYSTEM_ID, Starport, StellarSystem, World,
    generate_primary_world,
};

pub const CELESTIAL_GENERATION_VERSION: u16 = 1;
const JULIAN_YEAR_DAYS: f64 = 365.25;
const EARTH_RADIUS_KM: f64 = 6_371.0;
const SOLAR_DIAMETER_KM: f64 = 1_392_700.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpectralClass {
    O,
    B,
    A,
    F,
    G,
    K,
    M,
    L,
    T,
    Y,
    WhiteDwarf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LuminosityClass {
    Ia,
    Ib,
    II,
    III,
    IV,
    V,
    VI,
    D,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StellarZones {
    pub inner_limit_au: f64,
    pub habitable_inner_au: Option<f64>,
    pub habitable_outer_au: Option<f64>,
    pub snow_line_au: Option<f64>,
    pub outer_limit_au: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Star {
    pub id: u16,
    pub parent_star_id: Option<u16>,
    pub companion_separation_au: Option<f64>,
    /// Complete orbit relative to `parent_star_id`; absent for the primary.
    pub orbit: Option<OrbitalElements>,
    pub spectral_class: SpectralClass,
    pub subtype: u8,
    pub luminosity_class: LuminosityClass,
    pub temperature_kelvin: f64,
    pub mass_solar: f64,
    pub luminosity_solar: f64,
    pub zones: StellarZones,
}

impl Star {
    /// Photospheric diameter derived from luminosity and temperature.
    pub fn diameter_km(&self) -> f64 {
        let radius_solar =
            self.luminosity_solar.sqrt() * (5_778.0 / self.temperature_kelvin.max(1.0)).powi(2);
        radius_solar * SOLAR_DIAMETER_KM
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitalElements {
    pub semi_major_axis_au: f64,
    pub eccentricity: f64,
    pub inclination_degrees: f64,
    pub longitude_ascending_node_degrees: f64,
    pub argument_periapsis_degrees: f64,
    pub mean_anomaly_at_epoch_degrees: f64,
    pub epoch_game_days: f64,
    pub period_game_days: f64,
    pub rotation_hours: f64,
    pub axial_tilt_degrees: f64,
}

impl OrbitalElements {
    /// Position relative to the parent at the requested game time.
    ///
    /// This solves Kepler's equation with bounded Newton iteration. Coordinates
    /// are in AU in the system's stable, seed-derived reference plane.
    pub fn position_au(&self, game_days: f64) -> [f64; 3] {
        let elapsed = game_days - self.epoch_game_days;
        let mean = (self.mean_anomaly_at_epoch_degrees.to_radians()
            + TAU * elapsed / self.period_game_days)
            .rem_euclid(TAU);
        let mut eccentric_anomaly = mean;
        for _ in 0..12 {
            let denominator = 1.0 - self.eccentricity * eccentric_anomaly.cos();
            if denominator.abs() < 1e-12 {
                break;
            }
            eccentric_anomaly -=
                (eccentric_anomaly - self.eccentricity * eccentric_anomaly.sin() - mean)
                    / denominator;
        }
        let x = self.semi_major_axis_au * (eccentric_anomaly.cos() - self.eccentricity);
        let y = self.semi_major_axis_au
            * (1.0 - self.eccentricity * self.eccentricity).sqrt()
            * eccentric_anomaly.sin();
        rotate_orbit(
            [x, y, 0.0],
            self.inclination_degrees.to_radians(),
            self.longitude_ascending_node_degrees.to_radians(),
            self.argument_periapsis_degrees.to_radians(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RockyPlanetClass {
    Dwarf,
    Mercurian,
    Subterran,
    Terran,
    Superterran,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GasGiantClass {
    Neptunian,
    Jovian,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrbitalZone {
    Inner,
    Habitable,
    Outer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BodyKind {
    Rocky {
        class: RockyPlanetClass,
        radius_earth: f64,
        mass_earth: f64,
    },
    GasGiant {
        class: GasGiantClass,
        diameter_km: f64,
        ring_count: u8,
        ring_width_km: u32,
    },
    PlanetoidBelt {
        icy: bool,
        carbonaceous_percent: u8,
        silicate_or_rock_percent: u8,
        metal_or_water_ice_percent: u8,
        hydrocarbon_percent: u8,
        major_body_diameter_km: f64,
        width_au: f64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanetaryCore {
    Molten,
    Rocky,
    Icy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtmosphereComposition {
    None,
    Trace,
    NitrogenOxygen,
    Exotic,
    Corrosive,
    Insidious,
    DenseNitrogenOxygen,
    ThinNitrogenOxygen,
    Unusual,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtmosphericTaint {
    Chlorine,
    Fluorine,
    Sulfur,
    HighOxygen,
    Disease,
    PollenOrSpores,
    Biotoxins,
    Dust,
    VolcanicAsh,
    LowOxygen,
    NitrogenOxides,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NativeBiology {
    None,
    AquaticMicrobes,
    TerrestrialMicrobes,
    SimpleMulticellular,
    SmallAquaticAndCoastal,
    LargeAquaticAndFerns,
    Amphibian,
    EarlyLandEcology,
    DiverseVertebrates,
    ComplexEcology,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldPhysicalDetails {
    pub diameter_km: f64,
    pub density_earth: f64,
    pub mass_earth: f64,
    pub surface_gravity_earth: f64,
    pub core: PlanetaryCore,
    pub atmosphere_composition: AtmosphereComposition,
    pub atmospheric_taint: Option<AtmosphericTaint>,
    pub atmospheric_pressure_bar: f64,
    pub albedo: f64,
    pub average_temperature_kelvin: f64,
    pub native_biology: NativeBiology,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CelestialBody {
    /// Stable only within this system-generation version.
    pub local_id: u32,
    pub parent_star_id: u16,
    pub parent_body_id: Option<u32>,
    pub name: String,
    pub is_primary_world: bool,
    pub kind: BodyKind,
    pub orbit: OrbitalElements,
    pub world: Option<World>,
    pub physical: Option<WorldPhysicalDetails>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CelestialSystem {
    pub system_id: u64,
    pub generation_version: u16,
    pub stars: Vec<Star>,
    pub bodies: Vec<CelestialBody>,
    /// Source-table d66 codes. Zimm-only results are rerolled.
    pub quirk_codes: Vec<u8>,
}

impl CelestialSystem {
    pub fn primary_world(&self) -> &World {
        self.bodies
            .iter()
            .find(|body| body.is_primary_world)
            .and_then(|body| body.world.as_ref())
            .expect("derived systems always contain their primary world")
    }

    pub fn body(&self, local_id: u32) -> Option<&CelestialBody> {
        self.bodies.iter().find(|body| body.local_id == local_id)
    }

    pub fn primary_world_body(&self) -> &CelestialBody {
        self.bodies
            .iter()
            .find(|body| body.is_primary_world)
            .expect("derived systems always contain their primary world body")
    }
}

impl CelestialBody {
    pub fn diameter_km(&self) -> f64 {
        body_radius_km(&self.kind) * 2.0
    }
}

struct Dice {
    stream: SeedStream,
    block: [u8; 32],
    offset: usize,
}

impl Dice {
    fn from_label(seed: [u8; 32], label: &[u8]) -> Result<Self, CryptoError> {
        Ok(Self {
            stream: SeedStream::new(derive_seed(seed, label)?),
            block: [0; 32],
            offset: 32,
        })
    }

    fn byte(&mut self) -> Result<u8, CryptoError> {
        if self.offset == self.block.len() {
            self.block = self.stream.next_seed()?;
            self.offset = 0;
        }
        let value = self.block[self.offset];
        self.offset += 1;
        Ok(value)
    }

    fn die(&mut self, sides: u8) -> Result<u8, CryptoError> {
        debug_assert!(sides > 0);
        let limit = u8::MAX - (u8::MAX % sides);
        loop {
            let value = self.byte()?;
            if value < limit {
                return Ok(value % sides + 1);
            }
        }
    }

    fn roll(&mut self, count: u8, sides: u8) -> Result<u16, CryptoError> {
        let mut total = 0;
        for _ in 0..count {
            total += u16::from(self.die(sides)?);
        }
        Ok(total)
    }

    fn d6(&mut self) -> Result<u8, CryptoError> {
        self.die(6)
    }

    fn d10(&mut self) -> Result<u8, CryptoError> {
        self.die(10)
    }

    fn d100(&mut self) -> Result<u8, CryptoError> {
        let value = (u16::from(self.d10()?) - 1) * 10 + u16::from(self.d10()?);
        Ok(if value == 100 { 100 } else { value as u8 })
    }

    fn unit(&mut self) -> Result<f64, CryptoError> {
        let mut bytes = [0; 8];
        for byte in &mut bytes {
            *byte = self.byte()?;
        }
        Ok((u64::from_be_bytes(bytes) as f64) / (u64::MAX as f64))
    }

    fn between(&mut self, low: f64, high: f64) -> Result<f64, CryptoError> {
        Ok(low + (high - low) * self.unit()?)
    }
}

/// Derive the complete immutable celestial baseline for a materialized system.
pub fn derive_celestial_system(system: &StellarSystem) -> Result<CelestialSystem, CryptoError> {
    if system.id == SOL_SYSTEM_ID {
        return Ok(solar_system(system));
    }

    let primary_world = derive_primary_world(system)?;
    let known_brown_dwarf = INITIAL_SYSTEMS
        .iter()
        .find(|initial| initial.id == system.id && initial.name == system.name)
        .is_some_and(|initial| initial.primary_is_brown_dwarf());
    let mut stellar_dice = Dice::from_label(system.generation_seed, b"celestial/stellar/v1")?;
    let mut stellar_orbit_dice =
        Dice::from_label(system.generation_seed, b"celestial/stellar-orbits/v1")?;
    let stars = loop {
        let mut candidate = generate_stars(&mut stellar_dice, known_brown_dwarf)?;
        candidate.sort_by(|left, right| right.mass_solar.total_cmp(&left.mass_solar));
        let primary_mass = candidate[0].mass_solar;
        for (index, star) in candidate.iter_mut().enumerate() {
            star.id = index as u16 + 1;
            if index == 0 {
                star.parent_star_id = None;
                star.companion_separation_au = None;
                star.orbit = None;
            } else {
                star.parent_star_id = Some(1);
                if star.companion_separation_au.is_none() {
                    star.companion_separation_au = Some(companion_separation(&mut stellar_dice)?);
                }
                star.orbit = Some(random_orbit(
                    star.companion_separation_au
                        .expect("candidate companion has a separation"),
                    primary_mass,
                    star.mass_solar,
                    &mut stellar_orbit_dice,
                )?);
            }
        }
        if stellar_architecture_accepts(&candidate, &primary_world) {
            break candidate;
        }
    };

    let mut orbit_dice = Dice::from_label(system.generation_seed, b"celestial/orbits/v1")?;
    let mut detail_dice = Dice::from_label(system.generation_seed, b"celestial/bodies/v1")?;
    let bodies = generate_bodies(
        system,
        &primary_world,
        &stars,
        &mut orbit_dice,
        &mut detail_dice,
    )?;
    let quirk_codes = generate_quirks(system.generation_seed)?;

    Ok(CelestialSystem {
        system_id: system.id,
        generation_version: CELESTIAL_GENERATION_VERSION,
        stars,
        bodies,
        quirk_codes,
    })
}

fn stellar_architecture_accepts(stars: &[Star], world: &World) -> bool {
    let primary = &stars[0];
    let effective_outer = effective_outer_limit(stars);
    if effective_outer <= primary.zones.inner_limit_au {
        return false;
    }
    if world.population > 0 {
        let Some((habitable_inner, _)) = primary
            .zones
            .habitable_inner_au
            .zip(primary.zones.habitable_outer_au)
        else {
            return false;
        };
        if effective_outer <= habitable_inner {
            return false;
        }
    }
    if world.gas_giants > 0
        && primary
            .zones
            .snow_line_au
            .is_none_or(|snow_line| snow_line >= effective_outer)
    {
        return false;
    }
    let required_inner = primary
        .zones
        .snow_line_au
        .unwrap_or(primary.zones.inner_limit_au * 2.0);
    effective_outer > required_inner * 1.05_f64.powi(i32::from(world.gas_giants))
}

fn effective_outer_limit(stars: &[Star]) -> f64 {
    stars
        .iter()
        .skip(1)
        .filter_map(|star| {
            star.orbit
                .map(|orbit| orbit.semi_major_axis_au * (1.0 - orbit.eccentricity))
                .or(star.companion_separation_au)
        })
        .fold(stars[0].zones.outer_limit_au, |outer, periapsis| {
            outer.min(periapsis * 0.3)
        })
}

/// Derive the CE primary-world baseline without constructing the rest of the
/// system. Earth is the fixed exception.
pub fn derive_primary_world(system: &StellarSystem) -> Result<World, CryptoError> {
    if system.id == SOL_SYSTEM_ID {
        Ok(fixed_earth_world())
    } else {
        generate_primary_world(
            system.id,
            system.id,
            system.primary_world_name.clone(),
            system.generation_seed,
        )
    }
}

fn generate_stars(dice: &mut Dice, force_brown_dwarf: bool) -> Result<Vec<Star>, CryptoError> {
    let primary = generate_star(dice, force_brown_dwarf)?;
    let count = match primary.spectral_class {
        SpectralClass::O | SpectralClass::B | SpectralClass::A => match dice.d100()? {
            1..=10 => 1,
            11..=90 => 2,
            91..=98 => 3,
            99 => 4,
            _ => 5,
        },
        SpectralClass::F | SpectralClass::G | SpectralClass::K => match dice.d100()? {
            1..=45 => 1,
            46..=99 => 2,
            _ => 3,
        },
        _ => match dice.d100()? {
            1..=69 => 1,
            70..=98 => 2,
            _ => 3,
        },
    };
    let mut stars = vec![primary];
    for index in 1..count {
        let mut companion = generate_star(dice, false)?;
        companion.id = index + 1;
        companion.parent_star_id = Some(1);
        companion.companion_separation_au = Some(companion_separation(dice)?);
        stars.push(companion);
    }
    Ok(stars)
}

fn generate_star(dice: &mut Dice, force_brown_dwarf: bool) -> Result<Star, CryptoError> {
    let spectral_class = if force_brown_dwarf {
        match dice.d100()? {
            1..=50 => SpectralClass::L,
            51..=75 => SpectralClass::T,
            _ => SpectralClass::Y,
        }
    } else {
        match dice.d100()? {
            1..=80 => SpectralClass::M,
            81..=88 => SpectralClass::K,
            89..=94 => SpectralClass::G,
            95..=97 => SpectralClass::F,
            98 => SpectralClass::A,
            99 => SpectralClass::B,
            _ => SpectralClass::O,
        }
    };
    let subtype = dice.d10()? % 10;
    let luminosity_class = if matches!(
        spectral_class,
        SpectralClass::L | SpectralClass::T | SpectralClass::Y
    ) {
        LuminosityClass::V
    } else {
        match dice.d100()? {
            1..=90 => LuminosityClass::V,
            91..=94 => LuminosityClass::IV,
            95..=96 => LuminosityClass::D,
            97..=99 => LuminosityClass::III,
            _ => match dice.d10()? {
                1..=4 => LuminosityClass::II,
                5..=6 => LuminosityClass::VI,
                7..=8 => LuminosityClass::Ia,
                _ => LuminosityClass::Ib,
            },
        }
    };
    let effective_class = if luminosity_class == LuminosityClass::D {
        SpectralClass::WhiteDwarf
    } else {
        spectral_class
    };
    let effective_luminosity = if luminosity_class == LuminosityClass::VI
        && matches!(
            spectral_class,
            SpectralClass::O | SpectralClass::B | SpectralClass::A | SpectralClass::F
        ) {
        LuminosityClass::V
    } else {
        luminosity_class
    };
    let (temperature_kelvin, mass_solar, luminosity_solar) =
        stellar_properties(effective_class, subtype, effective_luminosity);
    Ok(Star {
        id: 1,
        parent_star_id: None,
        companion_separation_au: None,
        orbit: None,
        spectral_class: effective_class,
        subtype,
        luminosity_class: effective_luminosity,
        temperature_kelvin,
        mass_solar,
        luminosity_solar,
        zones: stellar_zones(effective_class, subtype, mass_solar, luminosity_solar),
    })
}

fn companion_separation(dice: &mut Dice) -> Result<f64, CryptoError> {
    let category = dice.d100()?;
    let index = usize::from((dice.d100()? - 1) / 10).min(9);
    const CLOSE: [f64; 10] = [0.5, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5];
    const NEAR: [f64; 10] = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
    const FAR: [f64; 10] = [
        100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 550.0,
    ];
    const DISTANT: [f64; 10] = [
        600.0, 750.0, 1_000.0, 1_500.0, 2_000.0, 2_500.0, 3_000.0, 4_000.0, 5_000.0, 6_000.0,
    ];
    Ok(match category {
        1..=10 => 0.01,
        11..=30 => CLOSE[index],
        31..=50 => NEAR[index],
        51..=80 => FAR[index],
        _ => DISTANT[index],
    })
}

fn stellar_properties(
    class: SpectralClass,
    subtype: u8,
    luminosity_class: LuminosityClass,
) -> (f64, f64, f64) {
    let index = usize::from(subtype.min(9));
    let (temperature, mass, luminosity) = match class {
        SpectralClass::O => MAIN_SEQUENCE_O[index],
        SpectralClass::B => MAIN_SEQUENCE_B[index],
        SpectralClass::A => MAIN_SEQUENCE_A[index],
        SpectralClass::F => MAIN_SEQUENCE_F[index],
        SpectralClass::G => MAIN_SEQUENCE_G[index],
        SpectralClass::K => MAIN_SEQUENCE_K[index],
        SpectralClass::M => MAIN_SEQUENCE_M[index],
        SpectralClass::L => {
            interpolate_stellar(index, 2_200.0, 1_410.0, 0.08, 0.05, 0.005, 0.00001)
        }
        SpectralClass::T => {
            interpolate_stellar(index, 1_400.0, 700.0, 0.05, 0.03, 0.000006, 0.000001)
        }
        SpectralClass::Y => {
            interpolate_stellar(index, 448.0, 298.0, 0.03, 0.03, 0.0000006, 0.000000007)
        }
        SpectralClass::WhiteDwarf => {
            interpolate_stellar(index, 100_000.0, 5_600.0, 1.1, 0.1, 6.91, 0.0000673)
        }
    };
    let (mass_factor, luminosity_factor) = match luminosity_class {
        LuminosityClass::Ia => (1.8, 100_000.0),
        LuminosityClass::Ib => (1.6, 10_000.0),
        LuminosityClass::II => (1.4, 1_000.0),
        LuminosityClass::III => (1.25, 100.0),
        LuminosityClass::IV => (1.1, 3.0),
        LuminosityClass::V | LuminosityClass::D => (1.0, 1.0),
        LuminosityClass::VI => (0.8, 0.35),
    };
    (
        temperature,
        (mass * mass_factor).max(0.01),
        (luminosity * luminosity_factor).max(1e-12),
    )
}

// Temperature (K), mass (Solar), and luminosity (Solar) from the OGC
// main-sequence stellar-zone tables. The remaining zone columns are exact
// functions of mass and luminosity.
const MAIN_SEQUENCE_O: [(f64, f64, f64); 10] = [
    (50_000.0, 100.0, 1_240_000.0),
    (47_800.0, 97.5, 994_000.0),
    (45_600.0, 95.0, 795_000.0),
    (43_400.0, 92.5, 634_000.0),
    (41_200.0, 90.0, 504_000.0),
    (39_000.0, 60.0, 398_000.0),
    (36_800.0, 37.0, 260_000.0),
    (34_600.0, 30.0, 154_000.0),
    (32_400.0, 23.0, 99_100.0),
    (30_200.0, 20.0, 57_600.0),
];
const MAIN_SEQUENCE_B: [(f64, f64, f64); 10] = [
    (28_000.0, 17.5, 36_200.0),
    (26_190.0, 14.2, 19_400.0),
    (24_380.0, 10.9, 9_360.0),
    (22_570.0, 7.6, 4_890.0),
    (20_760.0, 6.7, 2_290.0),
    (18_950.0, 5.9, 1_160.0),
    (17_140.0, 5.2, 692.0),
    (15_330.0, 4.5, 404.0),
    (13_520.0, 3.8, 211.0),
    (11_710.0, 3.4, 119.0),
];
const MAIN_SEQUENCE_A: [(f64, f64, f64); 10] = [
    (9_900.0, 2.9, 67.4),
    (9_650.0, 2.7, 49.2),
    (9_400.0, 2.5, 39.4),
    (9_150.0, 2.4, 28.9),
    (8_900.0, 2.1, 23.2),
    (8_650.0, 1.9, 17.0),
    (8_400.0, 1.8, 15.1),
    (8_150.0, 1.8, 12.2),
    (7_900.0, 1.8, 10.9),
    (7_650.0, 1.7, 8.85),
];
const MAIN_SEQUENCE_F: [(f64, f64, f64); 10] = [
    (7_400.0, 1.6, 7.94),
    (7_260.0, 1.6, 6.56),
    (7_120.0, 1.5, 5.95),
    (6_980.0, 1.5, 4.94),
    (6_840.0, 1.4, 4.50),
    (6_700.0, 1.4, 3.75),
    (6_560.0, 1.3, 3.13),
    (6_420.0, 1.3, 2.62),
    (6_280.0, 1.2, 2.41),
    (6_140.0, 1.1, 2.03),
];
const MAIN_SEQUENCE_G: [(f64, f64, f64); 10] = [
    (6_000.0, 1.1, 1.72),
    (5_890.0, 1.0, 1.46),
    (5_780.0, 1.0, 1.00),
    (5_670.0, 1.0, 1.00),
    (5_560.0, 0.9, 0.98),
    (5_450.0, 0.9, 0.84),
    (5_340.0, 0.9, 0.79),
    (5_230.0, 0.9, 0.68),
    (5_120.0, 0.8, 0.65),
    (5_010.0, 0.8, 0.57),
];
const MAIN_SEQUENCE_K: [(f64, f64, f64); 10] = [
    (4_900.0, 0.8, 0.54),
    (4_760.0, 0.8, 0.44),
    (4_620.0, 0.7, 0.40),
    (4_480.0, 0.7, 0.34),
    (4_340.0, 0.7, 0.31),
    (4_200.0, 0.7, 0.27),
    (4_060.0, 0.6, 0.21),
    (3_920.0, 0.6, 0.19),
    (3_780.0, 0.6, 0.16),
    (3_640.0, 0.5, 0.14),
];
const MAIN_SEQUENCE_M: [(f64, f64, f64); 10] = [
    (3_500.0, 0.5, 0.125),
    (3_333.0, 0.5, 0.0618),
    (3_167.0, 0.4, 0.0321),
    (3_000.0, 0.3, 0.0178),
    (2_833.0, 0.3, 0.0106),
    (2_667.0, 0.2, 0.00624),
    (2_500.0, 0.2, 0.00450),
    (2_333.0, 0.1, 0.00369),
    (2_167.0, 0.1, 0.00353),
    (2_000.0, 0.1, 0.00315),
];

fn interpolate_stellar(
    subtype: usize,
    hot_temperature: f64,
    cool_temperature: f64,
    hot_mass: f64,
    cool_mass: f64,
    hot_luminosity: f64,
    cool_luminosity: f64,
) -> (f64, f64, f64) {
    let fraction = subtype as f64 / 9.0;
    let geometric = |high: f64, low: f64| high * (low / high).powf(fraction);
    (
        hot_temperature + (cool_temperature - hot_temperature) * fraction,
        geometric(hot_mass, cool_mass),
        geometric(hot_luminosity, cool_luminosity),
    )
}

fn stellar_zones(
    class: SpectralClass,
    subtype: u8,
    mass_solar: f64,
    luminosity_solar: f64,
) -> StellarZones {
    let inner_limit = mass_solar * 0.2;
    let outer_limit = mass_solar * 40.0;
    let root_luminosity = luminosity_solar.sqrt();
    let habitable_inner = 0.95 * root_luminosity;
    let habitable_outer = 1.30 * root_luminosity;
    let snow_line = 5.0 * root_luminosity;
    let source_has_habitable_zone = !(matches!(class, SpectralClass::T | SpectralClass::Y)
        || class == SpectralClass::L && subtype >= 8
        || class == SpectralClass::WhiteDwarf && subtype >= 3);
    StellarZones {
        inner_limit_au: inner_limit,
        habitable_inner_au: (source_has_habitable_zone && habitable_inner <= outer_limit)
            .then_some(habitable_inner),
        habitable_outer_au: (source_has_habitable_zone && habitable_inner <= outer_limit)
            .then_some(habitable_outer.min(outer_limit)),
        snow_line_au: (snow_line <= outer_limit).then_some(snow_line),
        outer_limit_au: outer_limit,
    }
}

fn generate_bodies(
    system: &StellarSystem,
    main_world: &World,
    stars: &[Star],
    orbit_dice: &mut Dice,
    detail_dice: &mut Dice,
) -> Result<Vec<CelestialBody>, CryptoError> {
    let primary = &stars[0];
    let outer_limit = effective_outer_limit(stars);
    let main_orbit = main_world_orbit(main_world, primary, outer_limit);
    let mut specifications = if main_world.size == 0 {
        vec![BodySpecification::Belt {
            orbit_au: main_orbit,
            main_world: true,
        }]
    } else {
        vec![BodySpecification::Rocky {
            orbit_au: main_orbit,
            main_world: true,
        }]
    };

    let rocky_count = usize::from(detail_dice.d6()? + 1).max(1);
    let mut inward = main_orbit;
    let mut outward = main_orbit;
    for index in 1..rocky_count {
        let factor = if detail_dice.d10()? == 10 {
            2.0
        } else {
            1.0 + f64::from(detail_dice.d10()?) / 10.0
        };
        let place_inward = index % 2 == 1;
        let candidate = if place_inward {
            inward /= factor;
            inward
        } else {
            outward *= factor;
            outward
        };
        if candidate >= primary.zones.inner_limit_au && candidate <= outer_limit {
            specifications.push(BodySpecification::Rocky {
                orbit_au: candidate,
                main_world: false,
            });
        }
    }

    let snow = primary
        .zones
        .snow_line_au
        .unwrap_or((main_orbit * 2.0).min(outer_limit));
    let gas_start = snow.max(main_orbit * 1.05).min(outer_limit * 0.95);
    let gas_ratio = if main_world.gas_giants > 1 {
        (outer_limit * 0.98 / gas_start)
            .max(1.0)
            .powf(1.0 / f64::from(main_world.gas_giants - 1))
    } else {
        1.0
    };
    let mut gas_orbit = gas_start;
    for _ in 0..main_world.gas_giants {
        specifications.push(BodySpecification::GasGiant {
            orbit_au: gas_orbit,
        });
        gas_orbit *= gas_ratio;
    }

    let belt_start = if main_world.planetoid_belts > 0 {
        ((main_orbit + snow) / 2.0).max(primary.zones.inner_limit_au)
    } else {
        snow
    };
    let belt_ratio = if main_world.planetoid_belts > 1 {
        (outer_limit * 0.95 / belt_start)
            .max(1.0)
            .powf(1.0 / f64::from(main_world.planetoid_belts - 1))
    } else {
        1.0
    };
    let mut belt_orbit = belt_start.min(outer_limit * 0.95);
    let additional_belts = main_world
        .planetoid_belts
        .saturating_sub(u8::from(main_world.size == 0));
    for _ in 0..additional_belts {
        specifications.push(BodySpecification::Belt {
            orbit_au: belt_orbit,
            main_world: false,
        });
        belt_orbit *= belt_ratio;
    }
    specifications.sort_by(|left, right| left.orbit().total_cmp(&right.orbit()));

    let mut bodies = Vec::new();
    let mut next_id = 1_u32;
    for specification in specifications {
        let orbit_au = specification.orbit();
        let zone = orbital_zone(primary.zones, orbit_au);
        let (kind, world, name, is_primary) = match specification {
            BodySpecification::Rocky {
                main_world: true, ..
            } => (
                rocky_kind_from_uwp(main_world, detail_dice)?,
                Some(main_world.clone()),
                main_world.name.clone(),
                true,
            ),
            BodySpecification::Rocky { .. } => {
                let class = roll_rocky_class(zone, detail_dice)?;
                let world = secondary_world(system, next_id, class, zone, main_world, detail_dice)?;
                (
                    random_rocky_kind(class, detail_dice)?,
                    Some(world),
                    format!("{} {}", system.name, next_id),
                    false,
                )
            }
            BodySpecification::GasGiant { .. } => {
                let class = if detail_dice.d6()? <= 3 {
                    GasGiantClass::Neptunian
                } else {
                    GasGiantClass::Jovian
                };
                (
                    gas_giant_kind(class, detail_dice)?,
                    None,
                    format!("{} Giant {}", system.name, next_id),
                    false,
                )
            }
            BodySpecification::Belt {
                main_world: true, ..
            } => (
                belt_kind(orbit_au >= snow, detail_dice)?,
                Some(main_world.clone()),
                main_world.name.clone(),
                true,
            ),
            BodySpecification::Belt { .. } => (
                belt_kind(orbit_au >= snow, detail_dice)?,
                Some(belt_world(system, next_id, main_world, detail_dice)?),
                format!("{} Belt {}", system.name, next_id),
                false,
            ),
        };
        let orbit = random_orbit(
            orbit_au,
            primary.mass_solar,
            body_mass_solar(&kind),
            orbit_dice,
        )?;
        let physical = if matches!(kind, BodyKind::Rocky { .. }) {
            world
                .as_ref()
                .map(|world| world_physical_details(world, zone, &orbit, primary, detail_dice))
                .transpose()?
        } else {
            None
        };
        bodies.push(CelestialBody {
            local_id: next_id,
            parent_star_id: primary.id,
            parent_body_id: None,
            name,
            is_primary_world: is_primary,
            kind,
            orbit,
            world,
            physical,
        });
        next_id += 1;
    }

    let planets = bodies.clone();
    for planet in planets {
        let moon_count = moon_count(&planet.kind, planet.world.as_ref(), detail_dice)?;
        for moon_index in 0..moon_count {
            let size = moon_size(&planet.kind, planet.world.as_ref(), detail_dice)?;
            let class = rocky_class_from_size(size);
            let zone = orbital_zone(primary.zones, planet.orbit.semi_major_axis_au);
            let mut world = secondary_world(system, next_id, class, zone, main_world, detail_dice)?;
            world.size = size;
            let kind = random_rocky_kind(class, detail_dice)?;
            let planet_radius_km = body_radius_km(&planet.kind);
            let distance_km = planet_radius_km
                * if matches!(planet.kind, BodyKind::GasGiant { .. }) {
                    f64::from(detail_dice.d6()? * 5) + detail_dice.unit()?
                } else {
                    f64::from(detail_dice.roll(2, 10)? * 2)
                };
            let semi_major_axis_au = distance_km / 149_597_870.7;
            let orbit = random_orbit(
                semi_major_axis_au,
                body_mass_solar(&planet.kind),
                body_mass_solar(&kind),
                orbit_dice,
            )?;
            let physical = world_physical_details(&world, zone, &orbit, primary, detail_dice)?;
            bodies.push(CelestialBody {
                local_id: next_id,
                parent_star_id: planet.parent_star_id,
                parent_body_id: Some(planet.local_id),
                name: format!("{}-{}", planet.name, moon_index + 1),
                is_primary_world: false,
                kind,
                orbit,
                world: Some(world),
                physical: Some(physical),
            });
            next_id += 1;
        }
    }
    Ok(bodies)
}

#[derive(Clone, Copy)]
enum BodySpecification {
    Rocky { orbit_au: f64, main_world: bool },
    GasGiant { orbit_au: f64 },
    Belt { orbit_au: f64, main_world: bool },
}

impl BodySpecification {
    fn orbit(self) -> f64 {
        match self {
            Self::Rocky { orbit_au, .. }
            | Self::GasGiant { orbit_au }
            | Self::Belt { orbit_au, .. } => orbit_au,
        }
    }
}

fn main_world_orbit(world: &World, star: &Star, effective_outer: f64) -> f64 {
    let zones = star.zones;
    let habitable = zones
        .habitable_inner_au
        .zip(zones.habitable_outer_au)
        .and_then(|(inner, outer)| {
            let usable_outer = outer.min(effective_outer);
            (usable_outer > inner).then_some((inner + usable_outer) / 2.0)
        });
    let desired = if world.population > 0 || (4..=9).contains(&world.atmosphere) {
        habitable.unwrap_or((zones.inner_limit_au + effective_outer) / 2.0)
    } else if world.hydrographics == 0 {
        habitable
            .map(|center| (zones.inner_limit_au + center) / 2.0)
            .unwrap_or(zones.inner_limit_au * 2.0)
    } else {
        habitable
            .map(|center| (center + effective_outer.min(center * 2.0)) / 2.0)
            .unwrap_or(effective_outer / 2.0)
    };
    desired.clamp(zones.inner_limit_au * 1.01, effective_outer * 0.99)
}

fn orbital_zone(zones: StellarZones, orbit_au: f64) -> OrbitalZone {
    match (zones.habitable_inner_au, zones.habitable_outer_au) {
        (Some(inner), Some(outer)) if (inner..=outer).contains(&orbit_au) => OrbitalZone::Habitable,
        (Some(inner), _) if orbit_au < inner => OrbitalZone::Inner,
        _ => OrbitalZone::Outer,
    }
}

fn roll_rocky_class(zone: OrbitalZone, dice: &mut Dice) -> Result<RockyPlanetClass, CryptoError> {
    let roll = dice.roll(2, 6)?;
    Ok(match zone {
        OrbitalZone::Inner => match roll {
            2..=4 => RockyPlanetClass::Dwarf,
            5..=7 => RockyPlanetClass::Mercurian,
            8..=9 => RockyPlanetClass::Subterran,
            10..=11 => RockyPlanetClass::Terran,
            _ => RockyPlanetClass::Superterran,
        },
        OrbitalZone::Habitable => match roll {
            2 => RockyPlanetClass::Dwarf,
            3 => RockyPlanetClass::Mercurian,
            4..=6 => RockyPlanetClass::Subterran,
            7..=11 => RockyPlanetClass::Terran,
            _ => RockyPlanetClass::Superterran,
        },
        OrbitalZone::Outer => match roll {
            2..=6 => RockyPlanetClass::Dwarf,
            7..=9 => RockyPlanetClass::Mercurian,
            10 => RockyPlanetClass::Subterran,
            11 => RockyPlanetClass::Terran,
            _ => RockyPlanetClass::Superterran,
        },
    })
}

fn secondary_world(
    system: &StellarSystem,
    local_id: u32,
    class: RockyPlanetClass,
    zone: OrbitalZone,
    main: &World,
    dice: &mut Dice,
) -> Result<World, CryptoError> {
    let size = roll_size(class, dice)?;
    let atmosphere = roll_atmosphere(class, zone, size, dice)?;
    let hydrographics = roll_hydrographics(class, zone, atmosphere, size, dice)?;
    let population = if main.tech_level < 10 {
        0
    } else {
        let modifier = match zone {
            OrbitalZone::Habitable => -1,
            OrbitalZone::Inner => -2,
            OrbitalZone::Outer => -3,
        };
        (i16::from(dice.d6()?) + modifier).clamp(0, 10) as u8
    };
    let population_multiplier = if population == 0 {
        0
    } else {
        dice.d10()?.min(9)
    };
    let (government, law_level) = if population == 0 {
        (0, 0)
    } else {
        match dice.roll(2, 6)? {
            2 => (0, 0),
            3..=5 => (1, dice.d6()?.saturating_add(4).min(15)),
            6..=9 => (6, dice.d6()?.saturating_add(4).min(15)),
            10..=11 => (6, dice.d6()?.saturating_add(5).min(15)),
            _ => {
                let government =
                    (dice.roll(2, 6)? as i16 - 7 + i16::from(population)).clamp(0, 15) as u8;
                let law = (dice.roll(2, 6)? as i16 - 7 + i16::from(government)).clamp(0, 15) as u8;
                (government, law)
            }
        }
    };
    let starport = secondary_starport(government, population, main.starport, dice)?;
    Ok(World {
        id: derived_world_id(system.id, local_id),
        system_id: system.id,
        name: format!("{} {}", system.name, local_id),
        starport,
        size,
        atmosphere,
        hydrographics,
        population,
        population_multiplier,
        government,
        law_level,
        tech_level: if population == 0 { 0 } else { main.tech_level },
        planetoid_belts: 0,
        gas_giants: 0,
    })
}

fn roll_size(class: RockyPlanetClass, dice: &mut Dice) -> Result<u8, CryptoError> {
    Ok(match class {
        RockyPlanetClass::Dwarf => dice.d6()? % 2,
        RockyPlanetClass::Mercurian => 2 + dice.d6()? % 2,
        RockyPlanetClass::Subterran => 4 + (dice.d6()? - 1) / 2,
        RockyPlanetClass::Terran => match dice.roll(2, 6)? {
            2..=5 => 7,
            6..=9 => 8,
            _ => 9,
        },
        RockyPlanetClass::Superterran => {
            let roll = dice.roll(2, 10)?;
            match roll {
                2..=3 => 10,
                4..=5 => 11,
                6..=7 => 12,
                8..=9 => 13,
                _ => (roll + 4).min(24) as u8,
            }
        }
    })
}

fn roll_atmosphere(
    class: RockyPlanetClass,
    zone: OrbitalZone,
    size: u8,
    dice: &mut Dice,
) -> Result<u8, CryptoError> {
    Ok(match class {
        RockyPlanetClass::Dwarf => dice.d6()? % 2,
        RockyPlanetClass::Mercurian => match zone {
            OrbitalZone::Outer => dice.d6()?.saturating_sub(2).min(4),
            _ => dice.d6()?.saturating_sub(3).min(3),
        },
        RockyPlanetClass::Subterran => dice.d6()?,
        RockyPlanetClass::Terran => {
            let roll = dice.roll(2, 6)?;
            match zone {
                OrbitalZone::Habitable => {
                    [0, 0, 2, 3, 4, 5, 5, 6, 7, 8, 9, 10, 12][usize::from(roll)]
                }
                OrbitalZone::Inner => [0, 0, 2, 3, 4, 4, 5, 5, 6, 7, 10, 11, 12][usize::from(roll)],
                OrbitalZone::Outer => match dice.d6()? {
                    1 => 1,
                    2 => 2,
                    3 => 10,
                    4 => 11,
                    5 => 12,
                    _ => 13,
                },
            }
        }
        RockyPlanetClass::Superterran => match zone {
            OrbitalZone::Outer => match dice.d6()?.saturating_add((size >= 15) as u8) {
                1 => 14,
                2 => 10,
                3 => 11,
                4 => 12,
                _ => 13,
            },
            _ => {
                let roll = dice.roll(2, 10)?
                    + if size >= 20 {
                        6
                    } else if size >= 15 {
                        3
                    } else {
                        0
                    };
                match roll {
                    2 => 6,
                    3 => 7,
                    4..=6 => 8,
                    7..=9 => 9,
                    10..=12 => 14,
                    13..=15 => 10,
                    16..=17 => 11,
                    18..=19 => 12,
                    _ => 13,
                }
            }
        },
    })
}

fn roll_hydrographics(
    class: RockyPlanetClass,
    zone: OrbitalZone,
    atmosphere: u8,
    size: u8,
    dice: &mut Dice,
) -> Result<u8, CryptoError> {
    if matches!(class, RockyPlanetClass::Dwarf | RockyPlanetClass::Mercurian)
        && zone != OrbitalZone::Outer
    {
        return Ok(0);
    }
    if zone == OrbitalZone::Outer {
        return Ok(match dice.d6()?.saturating_add((size >= 15) as u8) {
            1 => 0,
            2 => 2,
            3 => 4,
            4 => 6,
            5 => 8,
            _ => 10,
        });
    }
    if atmosphere >= 10 || atmosphere <= 3 {
        return Ok(0);
    }
    let tainted_penalty = u8::from(matches!(atmosphere, 4 | 7 | 9));
    Ok(
        (i16::from(dice.d6()?) - 10 - i16::from(tainted_penalty) + i16::from(atmosphere))
            .clamp(0, 10) as u8,
    )
}

fn secondary_starport(
    government: u8,
    population: u8,
    maximum: Starport,
    dice: &mut Dice,
) -> Result<Starport, CryptoError> {
    if population == 0 || government == 0 {
        return Ok(Starport::X);
    }
    let candidate = match government {
        1 | 6 => Starport::C,
        8..=u8::MAX => one_worse(maximum),
        _ => match (dice.roll(2, 6)? as i16 - 7 + i16::from(population)).clamp(0, 15) {
            11.. => Starport::A,
            9..=10 => Starport::B,
            7..=8 => Starport::C,
            5..=6 => Starport::D,
            3..=4 => Starport::E,
            _ => Starport::X,
        },
    };
    Ok(if starport_rank(candidate) < starport_rank(maximum) {
        maximum
    } else {
        candidate
    })
}

fn starport_rank(starport: Starport) -> u8 {
    starport as u8
}

fn one_worse(starport: Starport) -> Starport {
    match starport {
        Starport::A => Starport::B,
        Starport::B => Starport::C,
        Starport::C => Starport::D,
        Starport::D => Starport::E,
        Starport::E | Starport::X => Starport::X,
    }
}

fn rocky_kind_from_uwp(world: &World, dice: &mut Dice) -> Result<BodyKind, CryptoError> {
    random_rocky_kind(rocky_class_from_size(world.size), dice)
}

fn rocky_class_from_size(size: u8) -> RockyPlanetClass {
    match size {
        0..=1 => RockyPlanetClass::Dwarf,
        2..=3 => RockyPlanetClass::Mercurian,
        4..=6 => RockyPlanetClass::Subterran,
        7..=9 => RockyPlanetClass::Terran,
        _ => RockyPlanetClass::Superterran,
    }
}

fn random_rocky_kind(class: RockyPlanetClass, dice: &mut Dice) -> Result<BodyKind, CryptoError> {
    let (radius_low, radius_high, mass_low, mass_high) = match class {
        RockyPlanetClass::Dwarf => (0.03, 0.18, 0.00001, 0.003),
        RockyPlanetClass::Mercurian => (0.2, 0.4, 0.003, 0.1),
        RockyPlanetClass::Subterran => (0.5, 0.7, 0.1, 0.5),
        RockyPlanetClass::Terran => (0.8, 1.1, 0.5, 2.0),
        RockyPlanetClass::Superterran => (1.2, 3.09, 2.0, 10.0),
    };
    Ok(BodyKind::Rocky {
        class,
        radius_earth: dice.between(radius_low, radius_high)?,
        mass_earth: dice.between(mass_low, mass_high)?,
    })
}

fn gas_giant_kind(class: GasGiantClass, dice: &mut Dice) -> Result<BodyKind, CryptoError> {
    let diameter_km = match class {
        GasGiantClass::Neptunian => f64::from(dice.roll(2, 6)? * 5 + 20) * 1_000.0,
        GasGiantClass::Jovian => f64::from(dice.roll(2, 10)? * 10 + 40) * 1_000.0,
    };
    let ring_roll = dice.d10()? + if class == GasGiantClass::Jovian { 2 } else { 0 };
    let (ring_count, ring_width_km) = match ring_roll {
        1..=4 => (0, 0),
        5..=6 => (dice.d10()?, u32::from(dice.d10()?)),
        7..=8 => (dice.d6()?, u32::from(dice.d100()?)),
        9..=10 => (dice.d10()?, u32::from(dice.d100()?)),
        _ => (dice.d10()?, u32::from(dice.d10()?) * 1_000),
    };
    Ok(BodyKind::GasGiant {
        class,
        diameter_km,
        ring_count,
        ring_width_km,
    })
}

fn belt_kind(icy: bool, dice: &mut Dice) -> Result<BodyKind, CryptoError> {
    let roll = dice.roll(2, 6)?;
    let (carbonaceous, rock, metal_or_ice, hydrocarbons) = if icy {
        match roll {
            2 => (0, 70, 10, 20),
            3..=4 => (0, 50, 10, 40),
            5..=7 => (0, 40, 10, 50),
            8..=9 => (0, 30, 20, 60),
            10..=11 => (0, 10, 20, 70),
            _ => (0, 0, 30, 70),
        }
    } else {
        match roll {
            2 => (85, 13, 2, 0),
            3..=4 => (80, 15, 5, 0),
            5..=7 => (75, 17, 8, 0),
            8..=9 => (75, 15, 10, 0),
            10..=11 => (75, 13, 12, 0),
            _ => (78, 10, 12, 0),
        }
    };
    let diameter = [
        0.0, 0.0, 0.001, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 10.0, 25.0, 50.0, 100.0,
    ][usize::from(dice.roll(2, 6)?)];
    let width = if icy {
        [
            0.0, 0.0, 2.0, 4.0, 7.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0,
        ][usize::from(dice.roll(2, 6)?)]
    } else {
        [
            0.0, 0.0, 0.001, 0.005, 0.010, 0.025, 0.050, 0.075, 0.100, 0.125, 0.150, 0.175, 0.200,
        ][usize::from(dice.roll(2, 6)?)]
    };
    Ok(BodyKind::PlanetoidBelt {
        icy,
        carbonaceous_percent: carbonaceous,
        silicate_or_rock_percent: rock,
        metal_or_water_ice_percent: metal_or_ice,
        hydrocarbon_percent: hydrocarbons,
        major_body_diameter_km: diameter,
        width_au: width,
    })
}

fn belt_world(
    system: &StellarSystem,
    local_id: u32,
    main: &World,
    dice: &mut Dice,
) -> Result<World, CryptoError> {
    let population = if main.tech_level >= 10 {
        dice.d6()?.saturating_sub(3)
    } else {
        0
    };
    Ok(World {
        id: derived_world_id(system.id, local_id),
        system_id: system.id,
        name: format!("{} Belt {}", system.name, local_id),
        starport: if population == 0 {
            Starport::X
        } else {
            Starport::C
        },
        size: 0,
        atmosphere: 0,
        hydrographics: 0,
        population,
        population_multiplier: if population == 0 {
            0
        } else {
            dice.d10()?.min(9)
        },
        government: if population == 0 { 0 } else { 1 },
        law_level: if population == 0 {
            0
        } else {
            dice.d6()?.saturating_add(4).min(15)
        },
        tech_level: if population == 0 { 0 } else { main.tech_level },
        planetoid_belts: 0,
        gas_giants: 0,
    })
}

fn moon_count(kind: &BodyKind, world: Option<&World>, dice: &mut Dice) -> Result<u8, CryptoError> {
    Ok(match kind {
        BodyKind::GasGiant {
            class: GasGiantClass::Neptunian,
            ..
        } => dice.roll(2, 6)?.saturating_sub(4) as u8,
        BodyKind::GasGiant {
            class: GasGiantClass::Jovian,
            ..
        } => (dice.roll(2, 6)? + 4).min(20) as u8,
        BodyKind::Rocky { .. } => match world.map(|value| value.size).unwrap_or(0) {
            0 => 0,
            1..=3 => dice.d6()?.saturating_sub(4),
            4..=6 => dice.d6()?.saturating_sub(3),
            _ => dice.d6()?.saturating_sub(2),
        },
        BodyKind::PlanetoidBelt { .. } => 0,
    })
}

fn moon_size(kind: &BodyKind, world: Option<&World>, dice: &mut Dice) -> Result<u8, CryptoError> {
    Ok(match kind {
        BodyKind::GasGiant {
            class: GasGiantClass::Neptunian,
            ..
        } => dice.d6()?,
        BodyKind::GasGiant {
            class: GasGiantClass::Jovian,
            ..
        } => dice.d6()?.saturating_add(1),
        BodyKind::Rocky { .. } => match world.map(|value| value.size).unwrap_or(0) {
            0..=5 => dice.d6()? % 2,
            6..=8 => dice.d6()?.saturating_sub(3),
            9..=15 => dice.d6()?.saturating_sub(2),
            _ => dice.d6()?.saturating_sub(1),
        },
        BodyKind::PlanetoidBelt { .. } => 0,
    })
}

fn random_orbit(
    semi_major_axis_au: f64,
    parent_mass_solar: f64,
    body_mass_solar: f64,
    dice: &mut Dice,
) -> Result<OrbitalElements, CryptoError> {
    let eccentricity = if dice.d6()? <= 3 {
        const ECCENTRICITIES: [f64; 19] = [
            0.002, 0.003, 0.004, 0.005, 0.006, 0.007, 0.008, 0.009, 0.010, 0.020, 0.030, 0.040,
            0.050, 0.070, 0.100, 0.125, 0.150, 0.200, 0.250,
        ];
        ECCENTRICITIES[usize::from(dice.roll(2, 10)? - 2)]
    } else {
        0.0
    };
    let period_years =
        (semi_major_axis_au.powi(3) / (parent_mass_solar + body_mass_solar).max(1e-9)).sqrt();
    Ok(OrbitalElements {
        semi_major_axis_au,
        eccentricity,
        inclination_degrees: dice.between(0.0, 8.0)?,
        longitude_ascending_node_degrees: dice.between(0.0, 360.0)?,
        argument_periapsis_degrees: dice.between(0.0, 360.0)?,
        mean_anomaly_at_epoch_degrees: dice.between(0.0, 360.0)?,
        epoch_game_days: 0.0,
        period_game_days: period_years * JULIAN_YEAR_DAYS,
        rotation_hours: rotation_period_hours(
            semi_major_axis_au,
            parent_mass_solar,
            period_years * JULIAN_YEAR_DAYS,
            dice,
        )?,
        axial_tilt_degrees: axial_tilt(dice)?,
    })
}

fn rotation_period_hours(
    orbit_au: f64,
    parent_mass_solar: f64,
    orbital_period_days: f64,
    dice: &mut Dice,
) -> Result<f64, CryptoError> {
    let mut hours = f64::from(dice.roll(4, 10)? + 5) + parent_mass_solar / orbit_au.max(1e-9);
    if hours > 45.0 {
        hours = match dice.roll(2, 6)? {
            2 => hours,
            3 => f64::from(dice.d10()?) * 5.0,
            4 => f64::from(dice.d10()?) * 10.0,
            5 => f64::from(dice.d10()?) * 20.0,
            6 => f64::from(dice.d10()?) * 30.0,
            7 => f64::from(dice.d10()?) * 24.0,
            8 => f64::from(dice.d10()?) * 5.0 * 24.0,
            9 => f64::from(dice.d10()?) * 10.0 * 24.0,
            10 => f64::from(dice.d10()?) * 20.0 * 24.0,
            11 => f64::from(dice.d10()?) * 30.0 * 24.0,
            _ => -f64::from(dice.d10()?) * 20.0 * 24.0,
        };
    }
    let orbital_hours = orbital_period_days * 24.0;
    Ok(if hours.abs() >= orbital_hours {
        hours.signum() * orbital_hours
    } else {
        hours
    })
}

fn axial_tilt(dice: &mut Dice) -> Result<f64, CryptoError> {
    let roll = dice.roll(2, 10)?;
    let ones = f64::from(dice.d10()? - 1);
    Ok(match roll {
        2..=5 => ones,
        6..=9 => 10.0 + ones,
        10..=13 => 20.0 + ones,
        14..=17 => 30.0 + ones,
        18..=19 => 40.0 + ones,
        _ => match dice.d10()? {
            1..=2 => 50.0 + ones,
            3 => 60.0 + ones,
            4 => 70.0 + ones,
            5 => 80.0 + ones,
            6 => 90.0,
            7 => 100.0 + ones,
            8 => 120.0 + ones,
            9 => 150.0 + ones,
            _ => 180.0,
        },
    })
}

fn world_physical_details(
    world: &World,
    zone: OrbitalZone,
    orbit: &OrbitalElements,
    star: &Star,
    dice: &mut Dice,
) -> Result<WorldPhysicalDetails, CryptoError> {
    let size = f64::from(world.size.max(1));
    let minimum_miles = if world.size == 0 {
        1.0
    } else {
        f64::from(world.size) * 1_000.0 - 500.0
    };
    let random_digits = u16::from(dice.d10()? % 10) * 100
        + u16::from(dice.d10()? % 10) * 10
        + u16::from(dice.d10()? % 10);
    let diameter_km = (minimum_miles + f64::from(random_digits)) * 1.6;
    let core_roll = i16::try_from(dice.roll(2, 6)?).expect("2d6 fits")
        + match (world.size, zone) {
            (0..=5, OrbitalZone::Outer) => 9,
            (0..=5, _) => 3,
            (6..=u8::MAX, OrbitalZone::Inner) => -2,
            (6..=u8::MAX, OrbitalZone::Habitable) => -4,
            _ => 3,
        };
    let core = match core_roll {
        i16::MIN..=6 => PlanetaryCore::Molten,
        7..=15 => PlanetaryCore::Rocky,
        _ => PlanetaryCore::Icy,
    };
    let density_roll = f64::from(dice.roll(2, 10)? - 2);
    let density_earth = match core {
        PlanetaryCore::Molten => (0.86 + density_roll * 0.02).min(1.18),
        PlanetaryCore::Rocky => 0.50 + density_roll * 0.02,
        PlanetaryCore::Icy => 0.12 + density_roll * 0.02,
    };
    let mass_earth = density_earth * (size / 8.0).powi(3);
    let surface_gravity_earth = mass_earth * 64.0 / size.powi(2);
    let atmosphere_composition = atmosphere_composition(world.atmosphere);
    let atmospheric_taint = atmospheric_taint(world.atmosphere, dice)?;
    let atmospheric_pressure_bar = atmosphere_pressure(world.atmosphere, dice)?;
    let albedo = world_albedo(world, dice)?;
    let equilibrium =
        278.5 * star.luminosity_solar.powf(0.25) * (1.0 - albedo).max(0.01).powf(0.25)
            / orbit.semi_major_axis_au.sqrt();
    let greenhouse = if world.atmosphere == 0 {
        0.0
    } else {
        6.0 * atmospheric_pressure_bar / surface_gravity_earth.max(0.1)
    };
    let average_temperature_kelvin = equilibrium + greenhouse;
    let native_biology = native_biology(world, zone, star, average_temperature_kelvin, dice)?;
    Ok(WorldPhysicalDetails {
        diameter_km,
        density_earth,
        mass_earth,
        surface_gravity_earth,
        core,
        atmosphere_composition,
        atmospheric_taint,
        atmospheric_pressure_bar,
        albedo,
        average_temperature_kelvin,
        native_biology,
    })
}

fn atmosphere_composition(code: u8) -> AtmosphereComposition {
    match code {
        0 => AtmosphereComposition::None,
        1 => AtmosphereComposition::Trace,
        2..=9 => AtmosphereComposition::NitrogenOxygen,
        10 => AtmosphereComposition::Exotic,
        11 => AtmosphereComposition::Corrosive,
        12 => AtmosphereComposition::Insidious,
        13 => AtmosphereComposition::DenseNitrogenOxygen,
        14 => AtmosphereComposition::ThinNitrogenOxygen,
        _ => AtmosphereComposition::Unusual,
    }
}

fn atmospheric_taint(code: u8, dice: &mut Dice) -> Result<Option<AtmosphericTaint>, CryptoError> {
    if !matches!(code, 2 | 4 | 7 | 9 | 10) {
        return Ok(None);
    }
    Ok(Some(match dice.roll(2, 6)? {
        2 => AtmosphericTaint::Chlorine,
        3 => AtmosphericTaint::Fluorine,
        4 => AtmosphericTaint::Sulfur,
        5 => AtmosphericTaint::HighOxygen,
        6 => AtmosphericTaint::Disease,
        7 => AtmosphericTaint::PollenOrSpores,
        8 => AtmosphericTaint::Biotoxins,
        9 => AtmosphericTaint::Dust,
        10 => AtmosphericTaint::VolcanicAsh,
        11 => AtmosphericTaint::LowOxygen,
        _ => AtmosphericTaint::NitrogenOxides,
    }))
}

fn atmosphere_pressure(code: u8, dice: &mut Dice) -> Result<f64, CryptoError> {
    let roll = usize::from(dice.roll(2, 6)?);
    Ok(match code {
        0 => 0.0,
        1 => [
            0.0, 0.0, 0.001, 0.002, 0.005, 0.007, 0.01, 0.02, 0.03, 0.05, 0.07, 0.08, 0.09,
        ][roll],
        2..=3 => [
            0.0, 0.0, 0.10, 0.12, 0.14, 0.16, 0.20, 0.22, 0.25, 0.30, 0.35, 0.40, 0.42,
        ][roll],
        4..=5 => [
            0.0, 0.0, 0.43, 0.45, 0.47, 0.50, 0.52, 0.56, 0.60, 0.64, 0.66, 0.68, 0.70,
        ][roll],
        6..=7 => [
            0.0, 0.0, 0.71, 0.75, 0.80, 0.90, 1.00, 1.00, 1.10, 1.20, 1.30, 1.40, 1.49,
        ][roll],
        8..=12 => [
            0.0, 0.0, 1.50, 1.60, 1.70, 1.80, 1.90, 2.00, 2.10, 2.20, 2.30, 2.40, 2.49,
        ][roll],
        13 => [
            0.0, 0.0, 2.50, 3.0, 5.0, 10.0, 20.0, 40.0, 80.0, 100.0, 150.0, 200.0, 250.0,
        ][roll],
        14 => [
            0.0, 0.0, 0.005, 0.007, 0.01, 0.03, 0.05, 0.07, 0.10, 0.20, 0.30, 0.40, 0.50,
        ][roll],
        _ => [
            0.0, 0.0, 0.10, 0.20, 0.40, 0.70, 1.00, 1.50, 2.00, 3.0, 5.0, 7.0, 10.0,
        ][roll],
    })
}

fn world_albedo(world: &World, dice: &mut Dice) -> Result<f64, CryptoError> {
    let base = if (4..=9).contains(&world.atmosphere) {
        match world.hydrographics {
            0..=2 => 0.07,
            3..=5 => 0.13,
            6..=8 => 0.23,
            _ => 0.29,
        }
    } else if world.hydrographics < 5 {
        0.05
    } else {
        0.47
    };
    Ok(base + f64::from(dice.roll(2, 10)? - 2) / 100.0)
}

fn native_biology(
    world: &World,
    zone: OrbitalZone,
    star: &Star,
    temperature_kelvin: f64,
    dice: &mut Dice,
) -> Result<NativeBiology, CryptoError> {
    if world.atmosphere < 4 || world.hydrographics == 0 {
        return Ok(NativeBiology::None);
    }
    let mut roll = i16::try_from(dice.roll(2, 10)?).expect("2d10 fits");
    roll += match world.atmosphere {
        4..=5 => -5,
        6..=9 => 10,
        _ => -20,
    };
    roll += match world.hydrographics {
        1..=2 => -15,
        3..=4 | 10 => -5,
        5..=9 => 5,
        _ => 0,
    };
    if !(273.15..=313.15).contains(&temperature_kelvin) {
        roll -= 10;
    }
    if zone != OrbitalZone::Habitable {
        roll -= 20;
    }
    roll += match star.spectral_class {
        SpectralClass::O | SpectralClass::B | SpectralClass::A | SpectralClass::M => -10,
        SpectralClass::G => 10,
        _ => 0,
    };
    Ok(match roll {
        i16::MIN..=0 => NativeBiology::None,
        1..=3 => NativeBiology::AquaticMicrobes,
        4..=5 => NativeBiology::TerrestrialMicrobes,
        6..=8 => NativeBiology::SimpleMulticellular,
        9..=10 => NativeBiology::SmallAquaticAndCoastal,
        11..=12 => NativeBiology::LargeAquaticAndFerns,
        13..=14 => NativeBiology::Amphibian,
        15..=16 => NativeBiology::EarlyLandEcology,
        17..=18 => NativeBiology::DiverseVertebrates,
        _ => NativeBiology::ComplexEcology,
    })
}

fn body_mass_solar(kind: &BodyKind) -> f64 {
    match kind {
        BodyKind::Rocky { mass_earth, .. } => mass_earth / 332_946.0,
        BodyKind::GasGiant { class, .. } => match class {
            GasGiantClass::Neptunian => 0.000_051_5,
            GasGiantClass::Jovian => 0.000_954_6,
        },
        BodyKind::PlanetoidBelt { .. } => 1e-10,
    }
}

fn body_radius_km(kind: &BodyKind) -> f64 {
    match kind {
        BodyKind::Rocky { radius_earth, .. } => radius_earth * EARTH_RADIUS_KM,
        BodyKind::GasGiant { diameter_km, .. } => diameter_km / 2.0,
        BodyKind::PlanetoidBelt {
            major_body_diameter_km,
            ..
        } => major_body_diameter_km / 2.0,
    }
}

fn derived_world_id(system_id: u64, local_id: u32) -> u64 {
    system_id
        .checked_mul(65_536)
        .and_then(|base| base.checked_add(u64::from(local_id)))
        .expect("system identifier exceeds derived-world namespace")
}

fn generate_quirks(seed: [u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let mut dice = Dice::from_label(seed, b"celestial/quirks/v1")?;
    let count = dice.d6()?.saturating_sub(3);
    let mut quirks = Vec::with_capacity(usize::from(count));
    while quirks.len() < usize::from(count) {
        let code = dice.d6()? * 10 + dice.d6()?;
        if matches!(code, 26 | 32) {
            continue;
        }
        quirks.push(code);
    }
    Ok(quirks)
}

fn rotate_orbit(point: [f64; 3], inclination: f64, node: f64, periapsis: f64) -> [f64; 3] {
    let (sin_w, cos_w) = periapsis.sin_cos();
    let (sin_i, cos_i) = inclination.sin_cos();
    let (sin_o, cos_o) = node.sin_cos();
    let x1 = point[0] * cos_w - point[1] * sin_w;
    let y1 = point[0] * sin_w + point[1] * cos_w;
    let y2 = y1 * cos_i;
    let z2 = y1 * sin_i;
    [x1 * cos_o - y2 * sin_o, x1 * sin_o + y2 * cos_o, z2]
}

fn solar_system(system: &StellarSystem) -> CelestialSystem {
    let sun = Star {
        id: 1,
        parent_star_id: None,
        companion_separation_au: None,
        orbit: None,
        spectral_class: SpectralClass::G,
        subtype: 2,
        luminosity_class: LuminosityClass::V,
        temperature_kelvin: 5_778.0,
        mass_solar: 1.0,
        luminosity_solar: 1.0,
        zones: StellarZones {
            inner_limit_au: 0.2,
            habitable_inner_au: Some(0.95),
            habitable_outer_au: Some(1.30),
            snow_line_au: Some(5.0),
            outer_limit_au: 40.0,
        },
    };
    let earth = fixed_earth_world();
    let specifications = [
        (
            "Mercury",
            0.387_098,
            0.205_630,
            RockyPlanetClass::Mercurian,
            0.383,
            0.0553,
            None,
        ),
        (
            "Venus",
            0.723_332,
            0.006_772,
            RockyPlanetClass::Terran,
            0.949,
            0.815,
            None,
        ),
        (
            "Earth",
            1.0,
            0.016_708_6,
            RockyPlanetClass::Terran,
            1.0,
            1.0,
            Some(earth),
        ),
        (
            "Mars",
            1.523_679,
            0.0934,
            RockyPlanetClass::Subterran,
            0.532,
            0.107,
            None,
        ),
        (
            "Jupiter",
            5.2044,
            0.0489,
            RockyPlanetClass::Superterran,
            11.21,
            317.8,
            None,
        ),
        (
            "Saturn",
            9.5826,
            0.0565,
            RockyPlanetClass::Superterran,
            9.45,
            95.16,
            None,
        ),
        (
            "Uranus",
            19.2184,
            0.0463,
            RockyPlanetClass::Superterran,
            4.01,
            14.54,
            None,
        ),
        (
            "Neptune",
            30.11,
            0.009_456,
            RockyPlanetClass::Superterran,
            3.88,
            17.15,
            None,
        ),
        (
            "Pluto",
            39.482,
            0.2488,
            RockyPlanetClass::Dwarf,
            0.186,
            0.00218,
            None,
        ),
    ];
    let mut bodies = Vec::new();
    for (index, (name, axis, eccentricity, class, radius, mass, world)) in
        specifications.into_iter().enumerate()
    {
        let giant = matches!(name, "Jupiter" | "Saturn" | "Uranus" | "Neptune");
        let kind = if giant {
            BodyKind::GasGiant {
                class: if matches!(name, "Uranus" | "Neptune") {
                    GasGiantClass::Neptunian
                } else {
                    GasGiantClass::Jovian
                },
                diameter_km: radius * EARTH_RADIUS_KM * 2.0,
                ring_count: if name == "Saturn" { 7 } else { 1 },
                ring_width_km: if name == "Saturn" { 282_000 } else { 50_000 },
            }
        } else {
            BodyKind::Rocky {
                class,
                radius_earth: radius,
                mass_earth: mass,
            }
        };
        let physical = (name == "Earth").then_some(WorldPhysicalDetails {
            diameter_km: 12_742.0,
            density_earth: 1.0,
            mass_earth: 1.0,
            surface_gravity_earth: 1.0,
            core: PlanetaryCore::Molten,
            atmosphere_composition: AtmosphereComposition::NitrogenOxygen,
            atmospheric_taint: None,
            atmospheric_pressure_bar: 1.0,
            albedo: 0.306,
            average_temperature_kelvin: 288.0,
            native_biology: NativeBiology::ComplexEcology,
        });
        bodies.push(CelestialBody {
            local_id: index as u32 + 1,
            parent_star_id: 1,
            parent_body_id: None,
            name: name.into(),
            is_primary_world: name == "Earth",
            kind,
            orbit: fixed_orbit(axis, eccentricity, mass / 332_946.0, index as f64 * 37.0),
            world,
            physical,
        });
    }
    bodies.push(CelestialBody {
        local_id: 10,
        parent_star_id: 1,
        parent_body_id: None,
        name: "Main Belt".into(),
        is_primary_world: false,
        kind: BodyKind::PlanetoidBelt {
            icy: false,
            carbonaceous_percent: 75,
            silicate_or_rock_percent: 17,
            metal_or_water_ice_percent: 8,
            hydrocarbon_percent: 0,
            major_body_diameter_km: 939.4,
            width_au: 1.5,
        },
        orbit: fixed_orbit(2.7, 0.0, 1e-10, 0.0),
        world: None,
        physical: None,
    });
    let moon_specs = [
        ("Moon", 3_u32, 384_400.0, 0.0549, 0.273, 0.0123),
        ("Phobos", 4, 9_376.0, 0.0151, 0.00177, 0.0000000018),
        ("Deimos", 4, 23_463.0, 0.0002, 0.00098, 0.00000000025),
        ("Io", 5, 421_700.0, 0.0041, 0.286, 0.015),
        ("Europa", 5, 671_034.0, 0.009, 0.245, 0.008),
        ("Ganymede", 5, 1_070_412.0, 0.0013, 0.413, 0.025),
        ("Callisto", 5, 1_882_709.0, 0.0074, 0.378, 0.018),
        ("Titan", 6, 1_221_870.0, 0.0288, 0.404, 0.0225),
        ("Enceladus", 6, 237_948.0, 0.0047, 0.0395, 0.000018),
        ("Titania", 7, 435_910.0, 0.0011, 0.124, 0.00059),
        ("Oberon", 7, 583_520.0, 0.0014, 0.119, 0.00051),
        ("Triton", 8, 354_759.0, 0.000016, 0.212, 0.00359),
        ("Charon", 9, 19_591.0, 0.0002, 0.095, 0.00027),
    ];
    for (name, parent_id, distance_km, eccentricity, radius, mass) in moon_specs {
        let parent_mass = body_mass_solar(&bodies[usize::try_from(parent_id - 1).unwrap()].kind);
        bodies.push(CelestialBody {
            local_id: bodies.len() as u32 + 1,
            parent_star_id: 1,
            parent_body_id: Some(parent_id),
            name: name.into(),
            is_primary_world: false,
            kind: BodyKind::Rocky {
                class: rocky_class_from_size(if radius > 0.2 { 1 } else { 0 }),
                radius_earth: radius,
                mass_earth: mass,
            },
            orbit: fixed_satellite_orbit(distance_km / 149_597_870.7, eccentricity, parent_mass),
            world: None,
            physical: None,
        });
    }
    CelestialSystem {
        system_id: system.id,
        generation_version: CELESTIAL_GENERATION_VERSION,
        stars: vec![sun],
        bodies,
        quirk_codes: Vec::new(),
    }
}

pub fn fixed_earth_world() -> World {
    World {
        id: EARTH_WORLD_ID,
        system_id: SOL_SYSTEM_ID,
        name: "Earth".into(),
        starport: Starport::A,
        size: 8,
        atmosphere: 6,
        hydrographics: 7,
        population: 9,
        population_multiplier: 8,
        government: 4,
        law_level: 5,
        tech_level: 13,
        planetoid_belts: 1,
        gas_giants: 4,
    }
}

fn fixed_orbit(axis: f64, eccentricity: f64, body_mass: f64, anomaly: f64) -> OrbitalElements {
    OrbitalElements {
        semi_major_axis_au: axis,
        eccentricity,
        inclination_degrees: 0.0,
        longitude_ascending_node_degrees: 0.0,
        argument_periapsis_degrees: 0.0,
        mean_anomaly_at_epoch_degrees: anomaly,
        epoch_game_days: 0.0,
        period_game_days: (axis.powi(3) / (1.0 + body_mass)).sqrt() * JULIAN_YEAR_DAYS,
        rotation_hours: 24.0,
        axial_tilt_degrees: if (axis - 1.0).abs() < 1e-9 {
            23.44
        } else {
            0.0
        },
    }
}

fn fixed_satellite_orbit(axis: f64, eccentricity: f64, parent_mass: f64) -> OrbitalElements {
    OrbitalElements {
        semi_major_axis_au: axis,
        eccentricity,
        inclination_degrees: 0.0,
        longitude_ascending_node_degrees: 0.0,
        argument_periapsis_degrees: 0.0,
        mean_anomaly_at_epoch_degrees: 0.0,
        epoch_game_days: 0.0,
        period_game_days: (axis.powi(3) / parent_mass.max(1e-12)).sqrt() * JULIAN_YEAR_DAYS,
        rotation_hours: 24.0,
        axial_tilt_degrees: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::INITIAL_GENERATION_VERSION;

    fn generated_system(seed: [u8; 32]) -> StellarSystem {
        StellarSystem {
            id: 90,
            name: "Test".into(),
            primary_world_name: "Test Primary".into(),
            position_parsecs: [1.0, 2.0, 3.0],
            polity_id: 1,
            generation_seed: seed,
            generation_version: INITIAL_GENERATION_VERSION,
        }
    }

    #[test]
    fn detailed_system_is_seed_stable_and_contains_the_frozen_primary() {
        let system = generated_system([0x31; 32]);
        let first = derive_celestial_system(&system).unwrap();
        let second = derive_celestial_system(&system).unwrap();
        assert_eq!(first, second);
        assert!(!first.stars.is_empty());
        assert!(!first.bodies.is_empty());
        assert_eq!(
            first.primary_world(),
            &generate_primary_world(
                system.id,
                system.id,
                system.primary_world_name.clone(),
                system.generation_seed,
            )
            .unwrap()
        );
        assert!(first.bodies.iter().all(|body| {
            body.orbit.semi_major_axis_au.is_finite()
                && body.orbit.period_game_days.is_finite()
                && body.orbit.period_game_days > 0.0
        }));
    }

    #[test]
    fn solar_system_and_earth_ignore_the_random_seed() {
        let mut sol = generated_system([0x11; 32]);
        sol.id = SOL_SYSTEM_ID;
        sol.name = "Sol".into();
        sol.primary_world_name = "Earth".into();
        let first = derive_celestial_system(&sol).unwrap();
        sol.generation_seed = [0xee; 32];
        let second = derive_celestial_system(&sol).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.stars[0].spectral_class, SpectralClass::G);
        assert_eq!(first.stars[0].subtype, 2);
        assert_eq!(first.primary_world(), &fixed_earth_world());
        assert!(first.bodies.iter().any(|body| body.name == "Jupiter"));
        assert!(first.bodies.iter().any(|body| body.name == "Moon"));
    }

    #[test]
    fn source_zone_formula_reproduces_the_g2v_anchor() {
        let zones = stellar_zones(SpectralClass::G, 2, 1.0, 1.0);
        assert_eq!(zones.inner_limit_au, 0.2);
        assert_eq!(zones.habitable_inner_au, Some(0.95));
        assert_eq!(zones.habitable_outer_au, Some(1.30));
        assert_eq!(zones.snow_line_au, Some(5.0));
        assert_eq!(zones.outer_limit_au, 40.0);
        assert!(
            stellar_zones(SpectralClass::T, 2, 0.04, 0.0000055)
                .habitable_inner_au
                .is_none()
        );
    }

    #[test]
    fn orbital_position_repeats_after_one_period() {
        let orbit = fixed_orbit(1.0, 0.1, 0.0, 37.0);
        let first = orbit.position_au(12.0);
        let second = orbit.position_au(12.0 + orbit.period_game_days);
        for axis in 0..3 {
            assert!((first[axis] - second[axis]).abs() < 1e-9);
        }
    }

    #[test]
    fn generated_systems_preserve_ce_pbg_and_have_complete_body_links() {
        let mut companion_count = 0;
        for value in 0..64_u8 {
            let system = generated_system([value; 32]);
            let derived = derive_celestial_system(&system).unwrap();
            let primary = derived.primary_world();
            let primary_body = derived.primary_world_body();
            let primary_star = &derived.stars[0];
            if primary.population > 0 {
                let habitable_inner = primary_star.zones.habitable_inner_au.unwrap();
                let habitable_outer = primary_star.zones.habitable_outer_au.unwrap();
                assert!(primary_body.orbit.semi_major_axis_au >= habitable_inner);
                assert!(primary_body.orbit.semi_major_axis_au <= habitable_outer);
                assert!(
                    primary_body.orbit.semi_major_axis_au < effective_outer_limit(&derived.stars)
                );
            }
            for star in &derived.stars {
                if star.parent_star_id.is_some() {
                    companion_count += 1;
                    let orbit = star.orbit.expect("companion orbit");
                    assert_eq!(
                        orbit.semi_major_axis_au,
                        star.companion_separation_au.unwrap()
                    );
                    assert!(
                        orbit
                            .position_au(123.0)
                            .iter()
                            .all(|value| value.is_finite())
                    );
                } else {
                    assert!(star.orbit.is_none());
                }
            }
            let top_level_giants = derived
                .bodies
                .iter()
                .filter(|body| {
                    body.parent_body_id.is_none() && matches!(body.kind, BodyKind::GasGiant { .. })
                })
                .count();
            let top_level_belts = derived
                .bodies
                .iter()
                .filter(|body| {
                    body.parent_body_id.is_none()
                        && matches!(body.kind, BodyKind::PlanetoidBelt { .. })
                })
                .count();
            assert_eq!(top_level_giants, usize::from(primary.gas_giants));
            assert_eq!(top_level_belts, usize::from(primary.planetoid_belts));
            for body in &derived.bodies {
                if let Some(parent) = body.parent_body_id {
                    assert!(derived.body(parent).is_some());
                }
                if matches!(body.kind, BodyKind::Rocky { .. }) && body.world.is_some() {
                    let physical = body.physical.expect("rocky world physical details");
                    assert!(physical.diameter_km > 0.0);
                    assert!(physical.mass_earth >= 0.0);
                    assert!(physical.average_temperature_kelvin.is_finite());
                }
            }
            assert!(
                derived
                    .quirk_codes
                    .iter()
                    .all(|code| !matches!(code, 26 | 32))
            );
        }
        assert!(companion_count > 0);
    }
}
