//! Ship upkeep, reliability, and warranty policy.
//!
//! The maintenance price and neglect roll come from CE. Refit price, duration,
//! and facility limits come from Clement Sector. The bathtub reliability
//! parameters are a versioned Cepheus Trader policy built around Clement's
//! hidden ship quirks and its five-year/200-transit standard warranty.

pub const MODEL_VERSION: u16 = 1;
pub const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
pub const SECONDS_PER_WEEK: u64 = 7 * SECONDS_PER_DAY;
pub const SECONDS_PER_YEAR: u64 = 365 * SECONDS_PER_DAY;
pub const ACCOUNTING_MONTH_SECONDS: u64 = 30 * SECONDS_PER_DAY;
pub const NEW_WARRANTY_SECONDS: u64 = 5 * SECONDS_PER_YEAR;
pub const NEW_WARRANTY_TRANSITS: u32 = 200;

pub fn monthly_maintenance_credits(purchase_price_credits: u64) -> u64 {
    purchase_price_credits.div_ceil(12_000)
}

pub fn refit_price_credits(purchase_price_credits: u64) -> u64 {
    monthly_maintenance_credits(purchase_price_credits).saturating_mul(4)
}

pub fn minimum_refit_starport(displacement_millitons: u64) -> char {
    match displacement_millitons {
        0..=800_000 => 'C',
        800_001..=2_000_000 => 'B',
        _ => 'A',
    }
}

pub fn refit_duration_seconds(entropy: u64) -> u64 {
    (4 + stable_roll(entropy, 3)) * SECONDS_PER_WEEK
}

/// Keep a yard quotation stable until the quoted ship revision changes.
pub fn refit_duration_for_revision(ship_id: u64, ship_revision: u64) -> u64 {
    refit_duration_seconds(mix64(
        ship_id ^ mix64(ship_revision ^ 0x5245_4649_5451_554f),
    ))
}

/// Ordinary starport berthing is Cr100 for the first six days, then Cr100
/// for each additional day or part of a day.
pub fn berth_fee_credits(arrived_second: u64, departure_second: u64) -> u64 {
    let elapsed = departure_second.saturating_sub(arrived_second);
    let included = 6 * SECONDS_PER_DAY;
    100_u64.saturating_add(elapsed.saturating_sub(included).div_ceil(SECONDS_PER_DAY) * 100)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NeglectCheck {
    pub roll: u8,
    pub damage_hits: u8,
}

pub fn neglect_check(entropy: u64, skipped_months: u16) -> NeglectCheck {
    let first = d6(entropy);
    let second = d6(mix64(entropy ^ 0x6a09_e667_f3bc_c909));
    let roll = u16::from(first)
        .saturating_add(u16::from(second))
        .saturating_add(skipped_months)
        .min(u16::from(u8::MAX)) as u8;
    let damage_die = d6(mix64(entropy ^ 0xbb67_ae85_84ca_a73b));
    let damage_hits = if roll < 8 {
        0
    } else {
        match damage_die {
            1..=3 => 1,
            4..=5 => 2,
            _ => 3,
        }
    };
    NeglectCheck { roll, damage_hits }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReliabilityRegion {
    Shakedown,
    UsefulLife,
    WearOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReliabilityHazard {
    pub region: ReliabilityRegion,
    /// Probability per accounting month, in millionths.
    pub parts_per_million: u32,
}

/// The standard warranty lies at the midpoint of the useful-life plateau.
/// Shakedown occupies the first 20% of that boundary and wear-out begins at
/// 180%, keeping the manufacturer out of the rising-cost portion.
pub fn reliability_hazard(age_seconds: u64, transit_count: u32) -> ReliabilityHazard {
    let calendar_ppm = u128::from(age_seconds) * 1_000_000 / u128::from(NEW_WARRANTY_SECONDS);
    let transit_ppm = u128::from(transit_count) * 1_000_000 / u128::from(NEW_WARRANTY_TRANSITS);
    let normalized = calendar_ppm.max(transit_ppm).min(u128::from(u32::MAX)) as u32;
    if normalized < 200_000 {
        let remaining = 200_000_u64 - u64::from(normalized);
        // 20 ppm plateau plus a declining 1,980 ppm infant-mortality term.
        let infant = 1_980_u64 * remaining * remaining / 200_000_u64.pow(2);
        ReliabilityHazard {
            region: ReliabilityRegion::Shakedown,
            parts_per_million: (20 + infant) as u32,
        }
    } else if normalized <= 1_800_000 {
        ReliabilityHazard {
            region: ReliabilityRegion::UsefulLife,
            parts_per_million: 20,
        }
    } else {
        let excess = u64::from(normalized - 1_800_000);
        let wear = excess.saturating_mul(excess) / 1_000_000_000;
        ReliabilityHazard {
            region: ReliabilityRegion::WearOut,
            parts_per_million: 20_u32.saturating_add(wear.min(999_980) as u32),
        }
    }
}

pub fn quirk_attaches(entropy: u64, hazard: ReliabilityHazard) -> bool {
    stable_roll(entropy, 1_000_000) < u64::from(hazard.parts_per_million)
}

/// Warranty service can discover a latent defect without making ordinary
/// diagnostics omniscient. Detection probability rises once symptoms exist.
pub fn warranty_service_detects(entropy: u64, manifested: bool) -> bool {
    stable_roll(entropy, 6) < if manifested { 4 } else { 1 }
}

pub fn stable_index(entropy: u64, length: usize) -> Option<usize> {
    (length != 0).then(|| stable_roll(entropy, length as u64) as usize)
}

fn d6(entropy: u64) -> u8 {
    (stable_roll(entropy, 6) + 1) as u8
}

fn stable_roll(entropy: u64, sides: u64) -> u64 {
    mix64(entropy) % sides
}

pub fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_backed_cost_and_facility_boundaries_are_exact() {
        assert_eq!(monthly_maintenance_credits(120_000_000), 10_000);
        assert_eq!(refit_price_credits(120_000_000), 40_000);
        assert_eq!(minimum_refit_starport(800_000), 'C');
        assert_eq!(minimum_refit_starport(800_001), 'B');
        assert_eq!(minimum_refit_starport(2_000_001), 'A');
        assert_eq!(berth_fee_credits(100, 100), 100);
        assert_eq!(berth_fee_credits(100, 100 + 6 * SECONDS_PER_DAY), 100);
        assert_eq!(berth_fee_credits(100, 101 + 6 * SECONDS_PER_DAY), 200);
    }

    #[test]
    fn refit_duration_is_stable_for_a_quoted_ship_revision() {
        let duration = refit_duration_for_revision(41, 9);
        assert_eq!(duration, refit_duration_for_revision(41, 9));
        assert!((4 * SECONDS_PER_WEEK..=6 * SECONDS_PER_WEEK).contains(&duration));
        let durations = (0..64)
            .map(|revision| refit_duration_for_revision(41, revision))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(durations.len(), 3);
    }

    #[test]
    fn warranty_sits_in_the_flat_not_the_wearout_rise() {
        let launch = reliability_hazard(0, 0);
        let warranty = reliability_hazard(NEW_WARRANTY_SECONDS, 0);
        let equal_use = reliability_hazard(0, NEW_WARRANTY_TRANSITS);
        let late = reliability_hazard(15 * SECONDS_PER_YEAR, 0);
        assert_eq!(launch.region, ReliabilityRegion::Shakedown);
        assert!(launch.parts_per_million > warranty.parts_per_million);
        assert_eq!(warranty.region, ReliabilityRegion::UsefulLife);
        assert_eq!(warranty, equal_use);
        assert_eq!(late.region, ReliabilityRegion::WearOut);
        assert!(late.parts_per_million > warranty.parts_per_million);
    }

    #[test]
    fn neglect_is_deterministic_and_matches_the_ce_threshold() {
        for seed in 0..10_000 {
            let check = neglect_check(seed, 0);
            assert_eq!(check.damage_hits == 0, check.roll < 8);
            assert!(check.damage_hits <= 3);
            assert_eq!(check, neglect_check(seed, 0));
        }
    }
}
