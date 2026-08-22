//! Asteroid prospecting, extraction, and field-refining rules.
//!
//! This module contains only independently expressed game mechanics.  Lodes
//! are persistent resources; a captain's observation of one is separate from
//! the resource itself so discoveries can remain private without creating an
//! exclusive claim.

pub const WATCH_SECONDS: u64 = 6 * 60 * 60;
pub const DAY_SECONDS: u64 = 24 * 60 * 60;
pub const MAXIMUM_JUMP_SECONDS: u64 = 184 * 60 * 60;
pub const SAFE_MARGIN_SECONDS: u64 = DAY_SECONDS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ResourceKind {
    Silicate = 0,
    Carbonaceous = 1,
    Metal = 2,
    WaterIce = 3,
    Hydrocarbons = 4,
    Crystals = 5,
    PreciousMetals = 6,
    Radioactives = 7,
}

impl ResourceKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Silicate => "silicate and industrial minerals",
            Self::Carbonaceous => "carbonaceous material",
            Self::Metal => "metal-bearing ore",
            Self::WaterIce => "water ice",
            Self::Hydrocarbons => "hydrocarbon-bearing material",
            Self::Crystals => "gem and crystal material",
            Self::PreciousMetals => "precious-metal ore",
            Self::Radioactives => "radioactive ore",
        }
    }

    /// Existing trade-good IDs; an unrefined operation always uses ID 6.
    pub const fn refined_commodity_id(self) -> u16 {
        match self {
            Self::Silicate | Self::Carbonaceous => 5,
            Self::Metal => 40,
            Self::WaterIce => 5,
            Self::Hydrocarbons => 30,
            Self::Crystals => 17,
            Self::PreciousMetals => 33,
            Self::Radioactives => 34,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BeltComposition {
    pub icy: bool,
    pub carbonaceous_percent: u8,
    pub silicate_or_rock_percent: u8,
    pub metal_or_water_ice_percent: u8,
    pub hydrocarbon_percent: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LodeProfile {
    pub kind: ResourceKind,
    /// Feedstock still in place, measured in millitons.
    pub remaining_raw_millitons: u64,
    /// Saleable fraction of extracted feedstock, in whole percent.
    pub grade_percent: u8,
}

fn die(entropy: u64, shift: u32) -> u8 {
    (((entropy.rotate_left(shift) ^ entropy.wrapping_mul(0x9e37_79b9_7f4a_7c15)) % 6) + 1) as u8
}

/// Creates a stable lode profile from server entropy and the generated belt
/// composition.  The extent ladder deliberately spans hand-workable finds to
/// deposits which can support years of traffic.
pub fn prospect(entropy: u64, belt: BeltComposition, exceptional: bool) -> LodeProfile {
    let selector = (entropy % 100) as u8;
    let carbon_end = belt.carbonaceous_percent;
    let silicate_end = carbon_end.saturating_add(belt.silicate_or_rock_percent);
    let metal_end = silicate_end.saturating_add(belt.metal_or_water_ice_percent);
    let hydrocarbon_end = metal_end.saturating_add(belt.hydrocarbon_percent);
    let mut kind = if selector < carbon_end {
        ResourceKind::Carbonaceous
    } else if selector < silicate_end {
        ResourceKind::Silicate
    } else if selector < metal_end {
        if belt.icy {
            ResourceKind::WaterIce
        } else {
            ResourceKind::Metal
        }
    } else if selector < hydrocarbon_end {
        ResourceKind::Hydrocarbons
    } else if belt.icy {
        ResourceKind::WaterIce
    } else {
        ResourceKind::Metal
    };
    if exceptional {
        kind = match die(entropy, 19) {
            1 => ResourceKind::Crystals,
            2 => ResourceKind::PreciousMetals,
            3 => ResourceKind::Radioactives,
            _ => kind,
        };
    }
    let extent_tons = [
        10_u64, 30, 100, 300, 1_000, 3_000, 10_000, 30_000, 100_000, 300_000, 1_000_000,
    ];
    let two_d6 = usize::from(die(entropy, 7) + die(entropy, 31));
    let grade_percent = match kind {
        ResourceKind::Crystals | ResourceKind::PreciousMetals => 2 * die(entropy, 43),
        ResourceKind::Radioactives => die(entropy, 43),
        _ => 5 * (die(entropy, 43) + die(entropy, 53)),
    };
    LodeProfile {
        kind,
        remaining_raw_millitons: extent_tons[two_d6.saturating_sub(2).min(10)] * 1_000,
        grade_percent,
    }
}

/// One installed mining-drone set handles 1D6×10 tons of feedstock per day.
pub fn daily_drone_capacity_millitons(drone_sets: u32, roll: u8) -> u64 {
    u64::from(drone_sets)
        .saturating_mul(u64::from(roll.clamp(1, 6)))
        .saturating_mul(10_000)
}

pub fn refined_output_millitons(raw_millitons: u64, grade_percent: u8, refinery_cap: u64) -> u64 {
    raw_millitons
        .saturating_mul(u64::from(grade_percent))
        .checked_div(100)
        .unwrap_or(0)
        .min(refinery_cap)
}

/// Fuel needed to run the power plant for a duration, rounded up so a ship can
/// never gain endurance from repeated settlement.
pub fn power_fuel_for_duration(
    power_fuel_capacity_millitons: u64,
    endurance_seconds: u64,
    duration_seconds: u64,
) -> u64 {
    if duration_seconds == 0 || power_fuel_capacity_millitons == 0 {
        return 0;
    }
    if endurance_seconds == 0 {
        return power_fuel_capacity_millitons;
    }
    let numerator = u128::from(power_fuel_capacity_millitons) * u128::from(duration_seconds);
    u64::try_from(numerator.div_ceil(u128::from(endurance_seconds))).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prospect_is_stable_and_bounded() {
        let belt = BeltComposition {
            icy: false,
            carbonaceous_percent: 20,
            silicate_or_rock_percent: 50,
            metal_or_water_ice_percent: 30,
            hydrocarbon_percent: 0,
        };
        let a = prospect(42, belt, false);
        assert_eq!(a, prospect(42, belt, false));
        assert!((5..=60).contains(&a.grade_percent));
        assert!((10_000..=1_000_000_000).contains(&a.remaining_raw_millitons));
    }

    #[test]
    fn extraction_and_power_math_round_conservatively() {
        assert_eq!(daily_drone_capacity_millitons(2, 6), 120_000);
        assert_eq!(refined_output_millitons(40_000, 25, 8_000), 8_000);
        assert_eq!(power_fuel_for_duration(1_000, 3, 1), 334);
    }
}
