//! Source-backed Cepheus Engine Jump timing and success resolution.

use crate::ship_condition::mix64;

pub const MINIMUM_JUMP_HOURS: u64 = 148;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JumpQuality {
    Accurate,
    Inaccurate {
        extra_days: u8,
    },
    Misjump {
        distance_parsecs: u8,
        direction_millionths: [i32; 3],
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JumpResolution {
    pub duration_seconds: u64,
    pub first_die: u8,
    pub second_die: u8,
    pub total: i16,
    pub quality: JumpQuality,
}

fn d6(entropy: u64, lane: u32) -> u8 {
    ((mix64(entropy.rotate_left(lane) ^ u64::from(lane)) % 6) + 1) as u8
}

pub fn resolve(
    entropy: u64,
    engineer_effect: i16,
    plot_age_months: u16,
    jump_drive_hits: u16,
    unrefined_fuel: bool,
    known_bad_plot: bool,
) -> JumpResolution {
    let duration_dice = (0..6)
        .map(|lane| u64::from(d6(entropy ^ 0x4a55_4d50, lane * 7)))
        .sum::<u64>();
    let duration_seconds = (MINIMUM_JUMP_HOURS + duration_dice) * 60 * 60;
    let first_die = d6(entropy ^ 0x5355_4343, 3);
    let second_die = d6(entropy ^ 0x4553_5300, 19);
    let total = if known_bad_plot {
        i16::MIN
    } else {
        i16::from(first_die) + i16::from(second_die) + engineer_effect
            - i16::try_from(plot_age_months).unwrap_or(i16::MAX)
            - i16::try_from(jump_drive_hits.saturating_mul(2)).unwrap_or(i16::MAX)
            - if unrefined_fuel { 2 } else { 0 }
    };
    let quality = if total >= 8 {
        JumpQuality::Accurate
    } else if total > 0 {
        JumpQuality::Inaccurate {
            extra_days: d6(entropy ^ 0x494e_4143, 27),
        }
    } else {
        let distance_parsecs =
            d6(entropy ^ 0x4d49_534a, 5).saturating_mul(d6(entropy ^ 0x554d_5000, 31));
        let raw = [
            i32::from(d6(entropy ^ 0x4449_5231, 2)) - i32::from(d6(entropy ^ 0x4449_5232, 11)),
            i32::from(d6(entropy ^ 0x4449_5233, 17)) - i32::from(d6(entropy ^ 0x4449_5234, 23)),
            i32::from(d6(entropy ^ 0x4449_5235, 29)) - i32::from(d6(entropy ^ 0x4449_5236, 37)),
        ];
        let magnitude = ((i64::from(raw[0]).pow(2)
            + i64::from(raw[1]).pow(2)
            + i64::from(raw[2]).pow(2)) as f64)
            .sqrt();
        let direction_millionths = if magnitude == 0.0 {
            [1_000_000, 0, 0]
        } else {
            raw.map(|value| (f64::from(value) * 1_000_000.0 / magnitude).round() as i32)
        };
        JumpQuality::Misjump {
            distance_parsecs,
            direction_millionths,
        }
    };
    JumpResolution {
        duration_seconds,
        first_die,
        second_die,
        total,
        quality,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_duration_is_the_ce_148_plus_six_dice_hours() {
        for entropy in 0..100 {
            let seconds = resolve(entropy, 0, 0, 0, false, false).duration_seconds;
            assert!(((148 + 6)..=(148 + 36)).contains(&(seconds / 3600)));
        }
    }

    #[test]
    fn source_modifiers_move_the_exact_success_total() {
        let clean = resolve(42, 2, 0, 0, false, false);
        let impaired = resolve(42, 2, 1, 2, true, false);
        assert_eq!(clean.total - impaired.total, 7);
        assert!(matches!(
            resolve(42, 20, 0, 0, false, true).quality,
            JumpQuality::Misjump { .. }
        ));
    }
}
