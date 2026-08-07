//! Shared Cepheus Engine 2D6 task resolution.
//!
//! Player-facing systems pass an assigned person and the rule-specific
//! characteristic/skill pair here. The caller owns equipment, environmental,
//! time, assistance, and consequence policy; this module owns the one common
//! characteristic DM, trained/untrained handling, Jack-of-All-Trades relief,
//! dice, target, and effect calculation.

use crate::wire::{PersonDraft, SkillId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskRequest {
    pub characteristic: u8,
    pub skill: SkillId,
    pub difficulty: i16,
    pub equipment_dm: i8,
    pub condition_dm: i8,
    pub assistance_dm: i8,
    pub entropy: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskResult {
    pub first_die: u8,
    pub second_die: u8,
    pub characteristic_dm: i8,
    pub skill_dm: i8,
    pub total: i16,
    pub effect: i16,
    pub success: bool,
}

pub const fn characteristic_dm(value: u8) -> i8 {
    match value {
        0 => -3,
        1..=2 => -2,
        3..=5 => -1,
        6..=8 => 0,
        9..=11 => 1,
        12..=14 => 2,
        _ => 3,
    }
}

fn skill_level(person: &PersonDraft, skill: SkillId) -> Option<i8> {
    person
        .skills
        .iter()
        .find(|rating| rating.skill == skill)
        .map(|rating| rating.level)
}

fn skill_dm(person: &PersonDraft, skill: SkillId) -> i8 {
    if let Some(level) = skill_level(person, skill) {
        return level;
    }
    let jack = skill_level(person, SkillId::JackOfAllTrades)
        .unwrap_or(0)
        .clamp(0, 2);
    -3 + jack
}

fn die(entropy: u64, shift: u32) -> u8 {
    ((entropy.rotate_right(shift) % 6) + 1) as u8
}

pub fn resolve(person: &PersonDraft, request: TaskRequest) -> TaskResult {
    let first_die = die(request.entropy, 0);
    let second_die = die(request.entropy ^ 0x9e37_79b9_7f4a_7c15, 29);
    let characteristic_dm = characteristic_dm(request.characteristic);
    let skill_dm = skill_dm(person, request.skill);
    let total = i16::from(first_die)
        + i16::from(second_die)
        + i16::from(characteristic_dm)
        + i16::from(skill_dm)
        + i16::from(request.equipment_dm)
        + i16::from(request.condition_dm)
        + i16::from(request.assistance_dm);
    let effect = total - request.difficulty;
    TaskResult {
        first_die,
        second_die,
        characteristic_dm,
        skill_dm,
        total,
        effect,
        success: effect >= 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{Characteristics, SkillRating, SkillTraining};

    fn person(skills: Vec<SkillRating>) -> PersonDraft {
        PersonDraft {
            name: "Task Tester".into(),
            characteristics: Characteristics {
                strength: 8,
                dexterity: 8,
                endurance: 8,
                intelligence: 8,
                education: 8,
                charisma: 8,
            },
            skills,
            training: SkillTraining {
                skill: SkillId::Admin,
                needed_weeks: 1,
                current_weeks: 0,
            },
        }
    }

    #[test]
    fn ce_characteristic_bands_are_exact() {
        assert_eq!(characteristic_dm(0), -3);
        assert_eq!(characteristic_dm(2), -2);
        assert_eq!(characteristic_dm(5), -1);
        assert_eq!(characteristic_dm(8), 0);
        assert_eq!(characteristic_dm(11), 1);
        assert_eq!(characteristic_dm(14), 2);
        assert_eq!(characteristic_dm(15), 3);
    }

    #[test]
    fn jack_of_all_trades_only_relieves_untrained_penalty() {
        let trained = person(vec![
            SkillRating {
                skill: SkillId::Broker,
                level: 1,
            },
            SkillRating {
                skill: SkillId::JackOfAllTrades,
                level: 2,
            },
        ]);
        let base = TaskRequest {
            characteristic: 8,
            skill: SkillId::Broker,
            difficulty: 8,
            equipment_dm: 0,
            condition_dm: 0,
            assistance_dm: 0,
            entropy: 7,
        };
        assert_eq!(resolve(&trained, base).skill_dm, 1);
        assert_eq!(
            resolve(
                &trained,
                TaskRequest {
                    skill: SkillId::Mechanic,
                    ..base
                }
            )
            .skill_dm,
            -1
        );
    }

    #[test]
    fn resolution_is_deterministic_and_reports_effect() {
        let operator = person(vec![SkillRating {
            skill: SkillId::Mechanic,
            level: 2,
        }]);
        let request = TaskRequest {
            characteristic: 10,
            skill: SkillId::Mechanic,
            difficulty: 8,
            equipment_dm: 1,
            condition_dm: -1,
            assistance_dm: 1,
            entropy: 0x1234,
        };
        let result = resolve(&operator, request);
        assert_eq!(result, resolve(&operator, request));
        assert_eq!(result.effect, result.total - 8);
        assert_eq!(result.success, result.effect >= 0);
    }
}
