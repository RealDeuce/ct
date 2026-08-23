//! Cepheus Engine physical condition and the deliberately coarse ship-service
//! morale policy used by Cepheus Trader.

use crate::task_resolution::characteristic_dm;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalCondition {
    pub strength: u8,
    pub dexterity: u8,
    pub endurance: u8,
}

impl PhysicalCondition {
    pub fn dead(self) -> bool {
        self.strength == 0 && self.dexterity == 0 && self.endurance == 0
    }

    pub fn unconscious(self) -> bool {
        usize::from(self.strength == 0)
            + usize::from(self.dexterity == 0)
            + usize::from(self.endurance == 0)
            >= 2
    }

    pub fn wounded(self, maximum: Self) -> bool {
        self != maximum
    }

    pub fn seriously_wounded(self, maximum: Self) -> bool {
        self.strength < maximum.strength
            && self.dexterity < maximum.dexterity
            && self.endurance < maximum.endurance
    }

    pub fn missing_points(self, maximum: Self) -> u16 {
        u16::from(maximum.strength.saturating_sub(self.strength))
            + u16::from(maximum.dexterity.saturating_sub(self.dexterity))
            + u16::from(maximum.endurance.saturating_sub(self.endurance))
    }
}

/// Apply abstract damage using CE's mandatory first Endurance track and a
/// conservative automatic allocation thereafter. Starship casualty damage is
/// not an interactive personal-combat choice, so later points are spread to
/// preserve the greatest number of non-zero characteristics.
pub fn apply_damage(current: &mut PhysicalCondition, maximum: PhysicalCondition, mut points: u16) {
    if *current == maximum && points != 0 {
        let applied = points.min(u16::from(current.endurance));
        current.endurance -= applied as u8;
        points -= applied;
    }
    while points != 0 && !current.dead() {
        let target = [current.strength, current.dexterity, current.endurance]
            .into_iter()
            .enumerate()
            .filter(|(_, value)| *value != 0)
            .max_by_key(|(index, value)| (*value, std::cmp::Reverse(*index)))
            .map(|(index, _)| index);
        match target {
            Some(0) => current.strength -= 1,
            Some(1) => current.dexterity -= 1,
            Some(2) => current.endurance -= 1,
            _ => break,
        }
        points -= 1;
    }
}

/// Restore points conservatively: first finish the least-damaged track so a
/// seriously wounded person leaves that state, then restore Endurance,
/// Strength, and Dexterity in that order.
pub fn restore_points(
    current: &mut PhysicalCondition,
    maximum: PhysicalCondition,
    mut points: u16,
) -> u16 {
    let before = current.missing_points(maximum);
    while points != 0 && *current != maximum {
        let deficits = [
            maximum.strength.saturating_sub(current.strength),
            maximum.dexterity.saturating_sub(current.dexterity),
            maximum.endurance.saturating_sub(current.endurance),
        ];
        let target = deficits
            .into_iter()
            .enumerate()
            .filter(|(_, deficit)| *deficit != 0)
            .min_by_key(|(index, deficit)| (*deficit, [1_u8, 2, 0][*index]))
            .map(|(index, _)| index);
        match target {
            Some(0) => current.strength += 1,
            Some(1) => current.dexterity += 1,
            Some(2) => current.endurance += 1,
            _ => break,
        }
        points -= 1;
    }
    before - current.missing_points(maximum)
}

pub fn natural_healing_points(
    current: PhysicalCondition,
    maximum: PhysicalCondition,
    full_rest: bool,
    die: u8,
) -> i16 {
    let endurance_dm = i16::from(characteristic_dm(current.endurance));
    if current.seriously_wounded(maximum) {
        return if full_rest { endurance_dm } else { 0 };
    }
    if full_rest {
        i16::from(die.clamp(1, 6)) + endurance_dm
    } else {
        1 + endurance_dm
    }
}

pub fn medical_care_points(current_endurance: u8, medicine_level: i8) -> i16 {
    2 + i16::from(characteristic_dm(current_endurance)) + i16::from(medicine_level)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StarvationCheck {
    pub first_die: u8,
    pub second_die: u8,
    pub total: i16,
    pub success: bool,
    pub damage: u8,
}

fn entropy_die(entropy: u64, shift: u32) -> u8 {
    ((entropy.rotate_right(shift) % 6) + 1) as u8
}

/// Resolve the daily CE starvation check after the initial three days without
/// food. `previous_checks` is zero for the first check and supplies the
/// cumulative DM-1 on each later day. Water is handled separately by callers.
pub fn starvation_check(
    current_endurance: u8,
    previous_checks: u16,
    entropy: u64,
) -> StarvationCheck {
    let first_die = entropy_die(entropy, 0);
    let second_die = entropy_die(entropy ^ 0x9e37_79b9_7f4a_7c15, 29);
    let prior_penalty = i16::try_from(previous_checks).unwrap_or(i16::MAX);
    let total = i16::from(first_die)
        + i16::from(second_die)
        + i16::from(characteristic_dm(current_endurance))
        + 2
        - prior_penalty;
    let success = total >= 8;
    let damage = if success {
        0
    } else {
        entropy_die(entropy ^ 0xd1b5_4a32_d192_ed03, 17)
    };
    StarvationCheck {
        first_die,
        second_die,
        total,
        success,
        damage,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoraleBand {
    Steady,
    Uneasy,
    Disaffected,
    Defiant,
    Broken,
}

pub const fn morale_band(morale: u8) -> MoraleBand {
    match morale {
        60..=100 => MoraleBand::Steady,
        40..=59 => MoraleBand::Uneasy,
        20..=39 => MoraleBand::Disaffected,
        1..=19 => MoraleBand::Defiant,
        _ => MoraleBand::Broken,
    }
}

pub const fn discretionary_morale_dm(morale: u8) -> i8 {
    match morale_band(morale) {
        MoraleBand::Disaffected => -1,
        MoraleBand::Defiant | MoraleBand::Broken => -2,
        MoraleBand::Steady | MoraleBand::Uneasy => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAXIMUM: PhysicalCondition = PhysicalCondition {
        strength: 8,
        dexterity: 7,
        endurance: 9,
    };

    #[test]
    fn first_damage_uses_endurance_then_spreads_conservatively() {
        let mut current = MAXIMUM;
        apply_damage(&mut current, MAXIMUM, 10);
        assert_eq!(
            current,
            PhysicalCondition {
                strength: 7,
                dexterity: 7,
                endurance: 0
            }
        );
        assert!(!current.unconscious());
    }

    #[test]
    fn healing_leaves_serious_wound_as_soon_as_possible() {
        let mut current = PhysicalCondition {
            strength: 7,
            dexterity: 5,
            endurance: 4,
        };
        assert!(current.seriously_wounded(MAXIMUM));
        assert_eq!(restore_points(&mut current, MAXIMUM, 1), 1);
        assert_eq!(current.strength, 8);
        assert!(!current.seriously_wounded(MAXIMUM));
    }

    #[test]
    fn morale_bands_have_stable_boundaries() {
        assert_eq!(morale_band(60), MoraleBand::Steady);
        assert_eq!(morale_band(40), MoraleBand::Uneasy);
        assert_eq!(morale_band(20), MoraleBand::Disaffected);
        assert_eq!(morale_band(1), MoraleBand::Defiant);
        assert_eq!(morale_band(0), MoraleBand::Broken);
    }

    #[test]
    fn starvation_checks_are_routine_and_worsen_each_day() {
        let first = starvation_check(8, 0, 0x1234_5678);
        let later = starvation_check(8, 4, 0x1234_5678);
        assert_eq!(later.first_die, first.first_die);
        assert_eq!(later.second_die, first.second_die);
        assert_eq!(later.total, first.total - 4);
        for check in [first, later] {
            if check.success {
                assert_eq!(check.damage, 0);
            } else {
                assert!((1..=6).contains(&check.damage));
            }
        }
    }
}
