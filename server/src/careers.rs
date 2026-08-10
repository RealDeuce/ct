//! Combat-side career, prize, and legal-domain rules.

use crate::traffic::TrafficContact;
use crate::wire::Career;

pub const CAREER_RULES_REVISION: u16 = 1;
pub const NAVAL_BOARD_DAYS: u64 = 180;
pub const NAVAL_BASE_MONTHLY_SALARY: u64 = 6_000;
pub const NAVAL_GRADE_MONTHLY_INCREMENT: u64 = 2_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CombatCareerMode {
    Independent,
    Navy,
    Privateer,
    Pirate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OpportunityKind {
    NavalOrder,
    PrivateerCommission,
    PirateLead,
    PirateCommission,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OpportunityState {
    Offered,
    Accepted,
    Succeeded,
    Failed,
    Expired,
    Reporting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ObjectiveKind {
    Patrol,
    Inspect,
    Escort,
    Intercept,
    Capture,
    SeizeCargo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ObjectiveEvidenceKind {
    None,
    PatrolLog,
    InspectionRecord,
    EscortRelease,
    TargetDrivenOff,
    TargetCaptured,
    CargoSecured,
    CargoDelivered,
}

pub fn objective_evidence_description(evidence: ObjectiveEvidenceKind) -> &'static str {
    match evidence {
        ObjectiveEvidenceKind::None => "no operational evidence",
        ObjectiveEvidenceKind::PatrolLog => "the certified patrol log",
        ObjectiveEvidenceKind::InspectionRecord => "the signed inspection record",
        ObjectiveEvidenceKind::EscortRelease => "the escorted vessel's release receipt",
        ObjectiveEvidenceKind::TargetDrivenOff => "the contact and engagement log",
        ObjectiveEvidenceKind::TargetCaptured => "the target's capture papers",
        ObjectiveEvidenceKind::CargoSecured => "the seized cargo inventory",
        ObjectiveEvidenceKind::CargoDelivered => "the cargo delivery receipt",
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CareerOpportunity {
    pub opportunity_id: u64,
    pub kind: OpportunityKind,
    pub state: OpportunityState,
    pub issued_system_id: u64,
    pub target_system_id: u64,
    pub target_contact_id: u64,
    pub issued_second: u64,
    pub expires_second: u64,
    pub reward_credits: u64,
    pub service_points: u16,
    pub authority: String,
    pub objective: String,
    pub objective_kind: ObjectiveKind,
    pub evidence_kind: ObjectiveEvidenceKind,
    pub evidence_second: u64,
    pub evidence_vessel_id: u64,
    pub order_message_id: u64,
    pub report_message_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PrizeStatus {
    Secured,
    ClaimInTransit,
    AwaitingAdjudication,
    Adjudicated,
    ReadyToFence,
    Settled,
    Seized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrizeRecord {
    pub prize_id: u64,
    pub captured_vessel_id: u64,
    /// Surviving people physically held with the captured vessel until the
    /// court, fence, or registry settlement transfers their custody.
    pub captured_person_ids: Vec<u64>,
    pub catalog_id: u32,
    pub name: String,
    pub gross_value_credits: u64,
    pub realizable_value_credits: u64,
    pub condition_percent: u8,
    pub status: PrizeStatus,
    pub secured_second: u64,
    pub claim_message_id: u64,
    pub settlement_credits: u64,
    pub advance_credits: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WarrantStatus {
    Filed,
    Propagating,
    Active,
    Revoked,
    Satisfied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarrantRecord {
    pub warrant_id: u64,
    pub issuing_polity_id: u64,
    pub origin_system_id: u64,
    pub filed_second: u64,
    pub message_id: u64,
    pub severity: u8,
    pub bounty_credits: u64,
    pub evidence_percent: u8,
    pub status: WarrantStatus,
    pub accusation: String,
    /// The court action is authoritative at this system immediately, but
    /// other systems continue to enforce the warrant until this message
    /// physically reaches them.
    pub resolution_message_id: u64,
    pub resolved_second: u64,
    pub resolving_system_id: u64,
}

/// Whether local authorities may act on a warrant using only the instruments
/// that have actually reached their system.
pub fn warrant_is_enforceable(
    warrant: &WarrantRecord,
    warrant_received: bool,
    resolution_received: bool,
) -> bool {
    if !warrant_received {
        return false;
    }
    if matches!(
        warrant.status,
        WarrantStatus::Filed | WarrantStatus::Propagating | WarrantStatus::Active
    ) {
        return true;
    }
    warrant.resolution_message_id == 0 || !resolution_received
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PirateCruise {
    pub revision: u64,
    pub active: bool,
    pub hunting_system_id: u64,
    pub ends_second: u64,
    pub crew_share_percent: u8,
    pub ship_fund_percent: u8,
    pub prohibited_targets: String,
}

impl Default for PirateCruise {
    fn default() -> Self {
        Self {
            revision: 1,
            active: false,
            hunting_system_id: 0,
            ends_second: 0,
            crew_share_percent: 50,
            ship_fund_percent: 20,
            prohibited_targets: "hospital, rescue, and surrendering vessels".into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CareerState {
    pub revision: u64,
    pub mode: CombatCareerMode,
    pub service_points: u16,
    /// Promotions are awarded only at a scheduled naval board.  Service
    /// points may make a captain eligible, but do not silently confer rank.
    pub naval_grade_index: u8,
    pub next_naval_board_second: u64,
    pub next_salary_second: u64,
    pub public_heat: u32,
    pub underworld_standing: i16,
    pub crew_pressure: u16,
    pub opportunities: Vec<CareerOpportunity>,
    pub prizes: Vec<PrizeRecord>,
    pub warrants: Vec<WarrantRecord>,
    pub cruise: PirateCruise,
}

impl CareerState {
    pub fn for_starting_career(career: Career, current_second: u64, seconds_per_day: u64) -> Self {
        let mode = match career {
            Career::Navy => CombatCareerMode::Navy,
            Career::Privateer => CombatCareerMode::Privateer,
            Career::Trader => CombatCareerMode::Independent,
        };
        Self {
            revision: 1,
            mode,
            service_points: 0,
            naval_grade_index: 0,
            next_naval_board_second: current_second
                .saturating_add(NAVAL_BOARD_DAYS * seconds_per_day),
            next_salary_second: current_second.saturating_add(30 * seconds_per_day),
            public_heat: 0,
            underworld_standing: 0,
            crew_pressure: 0,
            opportunities: Vec::new(),
            prizes: Vec::new(),
            warrants: Vec::new(),
            cruise: PirateCruise::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NavalGrade {
    pub index: u8,
    pub name: &'static str,
    pub minimum_points: u16,
    pub maximum_command_tons: Option<u32>,
}

pub const NAVAL_GRADES: [NavalGrade; 5] = [
    NavalGrade {
        index: 0,
        name: "Lieutenant",
        minimum_points: 0,
        maximum_command_tons: Some(400),
    },
    NavalGrade {
        index: 1,
        name: "Lieutenant Commander",
        minimum_points: 10,
        maximum_command_tons: Some(1_000),
    },
    NavalGrade {
        index: 2,
        name: "Commander",
        minimum_points: 25,
        maximum_command_tons: Some(3_000),
    },
    NavalGrade {
        index: 3,
        name: "Captain",
        minimum_points: 45,
        maximum_command_tons: None,
    },
    NavalGrade {
        index: 4,
        name: "Commodore",
        minimum_points: 70,
        maximum_command_tons: None,
    },
];

pub fn naval_grade(points: u16) -> NavalGrade {
    NAVAL_GRADES
        .iter()
        .rev()
        .find(|grade| points >= grade.minimum_points)
        .copied()
        .unwrap_or(NAVAL_GRADES[0])
}

pub fn naval_grade_by_index(index: u8) -> NavalGrade {
    NAVAL_GRADES
        .get(usize::from(index))
        .copied()
        .unwrap_or(NAVAL_GRADES[0])
}

pub fn eligible_naval_grade_index(points: u16) -> u8 {
    naval_grade(points).index
}

pub fn naval_monthly_salary(points: u16) -> u64 {
    NAVAL_BASE_MONTHLY_SALARY + u64::from(naval_grade(points).index) * NAVAL_GRADE_MONTHLY_INCREMENT
}

pub fn naval_salary_for_grade(index: u8) -> u64 {
    NAVAL_BASE_MONTHLY_SALARY
        + u64::from(naval_grade_by_index(index).index) * NAVAL_GRADE_MONTHLY_INCREMENT
}

pub fn directive_service_points(difficulty: u8) -> u16 {
    match difficulty {
        0..=2 => 1,
        3..=5 => 3,
        6..=8 => 6,
        _ => 10,
    }
}

/// Offers reference an already projected traffic contact. They never create a
/// victim. If the contact has moved before interception, the offer is stale.
pub fn opportunity_for_contact(
    mode: CombatCareerMode,
    system_id: u64,
    current_second: u64,
    seconds_per_day: u64,
    contact: &TrafficContact,
    entropy: u64,
) -> Option<CareerOpportunity> {
    let (kind, authority, objective_kind, base_reward, points): (_, _, _, u64, u16) = match mode {
        CombatCareerMode::Navy => (
            OpportunityKind::NavalOrder,
            "Admiralty traffic office",
            [
                ObjectiveKind::Patrol,
                ObjectiveKind::Inspect,
                ObjectiveKind::Escort,
                ObjectiveKind::Intercept,
                ObjectiveKind::Capture,
            ][entropy as usize % 5],
            0,
            directive_service_points((entropy % 10) as u8),
        ),
        CombatCareerMode::Privateer => (
            OpportunityKind::PrivateerCommission,
            "the issuing prize office",
            if entropy & 1 == 0 {
                ObjectiveKind::Capture
            } else {
                ObjectiveKind::SeizeCargo
            },
            10_000,
            0,
        ),
        CombatCareerMode::Pirate if entropy % 3 == 0 => (
            OpportunityKind::PirateCommission,
            "an unnamed patron",
            if entropy & 1 == 0 {
                ObjectiveKind::Capture
            } else {
                ObjectiveKind::SeizeCargo
            },
            25_000,
            0,
        ),
        CombatCareerMode::Pirate => (
            OpportunityKind::PirateLead,
            "a port-side informant",
            ObjectiveKind::Intercept,
            0,
            0,
        ),
        CombatCareerMode::Independent => return None,
    };
    let objective = match objective_kind {
        ObjectiveKind::Patrol => "challenge the named contact and file the patrol log",
        ObjectiveKind::Inspect => "obtain and retain a sensor inspection of the named contact",
        ObjectiveKind::Escort => "escort the protected vessel through the named traffic locus",
        ObjectiveKind::Intercept => "intercept and drive off the named contact",
        ObjectiveKind::Capture => "capture the named vessel and retain physical custody",
        ObjectiveKind::SeizeCargo => {
            "capture the named vessel and deliver its seized cargo to the issuing authority"
        }
    };
    Some(CareerOpportunity {
        opportunity_id: mix64(contact.contact_id ^ entropy ^ current_second),
        kind,
        state: OpportunityState::Offered,
        issued_system_id: system_id,
        target_system_id: system_id,
        target_contact_id: contact.contact_id,
        issued_second: current_second,
        expires_second: contact.edge_second.saturating_add(2 * seconds_per_day),
        reward_credits: base_reward.saturating_add(contact.displacement_millitons / 20),
        service_points: points,
        authority: authority.into(),
        objective: objective.into(),
        objective_kind,
        evidence_kind: ObjectiveEvidenceKind::None,
        evidence_second: 0,
        evidence_vessel_id: 0,
        order_message_id: 0,
        report_message_id: 0,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrizeTerms {
    pub realizable_value_credits: u64,
    pub settlement_credits: u64,
    pub maximum_advance_credits: u64,
    pub percent: u8,
}

pub fn prize_terms(
    mode: CombatCareerMode,
    gross: u64,
    condition_percent: u8,
    law_level: u8,
    entropy: u64,
) -> PrizeTerms {
    let realizable = gross.saturating_mul(u64::from(condition_percent.min(100))) / 100;
    let percent = match mode {
        CombatCareerMode::Privateer => [10, 20, 30][entropy as usize % 3],
        CombatCareerMode::Navy => [5, 10, 15][entropy as usize % 3],
        CombatCareerMode::Pirate => {
            let corruption_bonus = u64::from(15_u8.saturating_sub(law_level.min(15)));
            (10 + (entropy + corruption_bonus) % 21) as u8
        }
        _ => 0,
    };
    let settlement = realizable.saturating_mul(u64::from(percent)) / 100;
    PrizeTerms {
        realizable_value_credits: realizable,
        settlement_credits: settlement,
        maximum_advance_credits: settlement / 2,
        percent,
    }
}

pub fn combat_objective_evidence(
    objective: ObjectiveKind,
    command_lost: bool,
    target: Option<crate::combat::VesselDisposition>,
    inspection_recorded: bool,
) -> Option<ObjectiveEvidenceKind> {
    match objective {
        ObjectiveKind::Patrol if !command_lost => Some(ObjectiveEvidenceKind::PatrolLog),
        ObjectiveKind::Inspect if inspection_recorded => {
            Some(ObjectiveEvidenceKind::InspectionRecord)
        }
        ObjectiveKind::Intercept
            if target.is_some_and(|disposition| {
                disposition != crate::combat::VesselDisposition::Active
            }) =>
        {
            Some(ObjectiveEvidenceKind::TargetDrivenOff)
        }
        ObjectiveKind::Capture
            if target.is_some_and(|disposition| {
                matches!(
                    disposition,
                    crate::combat::VesselDisposition::Captured
                        | crate::combat::VesselDisposition::Surrendered
                )
            }) =>
        {
            Some(ObjectiveEvidenceKind::TargetCaptured)
        }
        ObjectiveKind::SeizeCargo
            if target.is_some_and(|disposition| {
                matches!(
                    disposition,
                    crate::combat::VesselDisposition::Captured
                        | crate::combat::VesselDisposition::Surrendered
                )
            }) =>
        {
            Some(ObjectiveEvidenceKind::CargoSecured)
        }
        ObjectiveKind::Escort
        | ObjectiveKind::Patrol
        | ObjectiveKind::Inspect
        | ObjectiveKind::Intercept
        | ObjectiveKind::Capture
        | ObjectiveKind::SeizeCargo => None,
    }
}

pub fn warrant_for_unlawful_attack(
    warrant_id: u64,
    polity_id: u64,
    system_id: u64,
    current_second: u64,
    target_value: u64,
    evidence_percent: u8,
) -> WarrantRecord {
    let severity = ((target_value / 10_000_000).min(9) + 1) as u8;
    WarrantRecord {
        warrant_id,
        issuing_polity_id: polity_id,
        origin_system_id: system_id,
        filed_second: current_second,
        message_id: 0,
        severity,
        bounty_credits: target_value.saturating_mul(u64::from(evidence_percent)) / 1_000,
        evidence_percent,
        status: WarrantStatus::Filed,
        accusation: "unlawful armed interception of registered traffic".into(),
        resolution_message_id: 0,
        resolved_second: 0,
        resolving_system_id: 0,
    }
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod objective_tests {
    use super::*;
    use crate::combat::VesselDisposition;

    #[test]
    fn combat_objectives_require_their_own_typed_evidence() {
        assert_eq!(
            combat_objective_evidence(
                ObjectiveKind::Capture,
                false,
                Some(VesselDisposition::Withdrawing),
                false,
            ),
            None
        );
        assert_eq!(
            combat_objective_evidence(
                ObjectiveKind::Capture,
                false,
                Some(VesselDisposition::Captured),
                false,
            ),
            Some(ObjectiveEvidenceKind::TargetCaptured)
        );
        assert_eq!(
            combat_objective_evidence(
                ObjectiveKind::Inspect,
                false,
                Some(VesselDisposition::Destroyed),
                false,
            ),
            None
        );
        assert_eq!(
            combat_objective_evidence(
                ObjectiveKind::Inspect,
                false,
                Some(VesselDisposition::Active),
                true,
            ),
            Some(ObjectiveEvidenceKind::InspectionRecord)
        );
        assert_eq!(
            combat_objective_evidence(
                ObjectiveKind::SeizeCargo,
                false,
                Some(VesselDisposition::Surrendered),
                false,
            ),
            Some(ObjectiveEvidenceKind::CargoSecured)
        );
    }
}

#[cfg(test)]
mod warrant_tests {
    use super::*;

    #[test]
    fn a_resolution_only_stops_enforcement_after_its_mail_arrives() {
        let mut warrant = warrant_for_unlawful_attack(1, 2, 3, 4, 10_000_000, 75);
        warrant.message_id = 20;
        warrant.status = WarrantStatus::Satisfied;
        warrant.resolution_message_id = 21;

        assert!(warrant_is_enforceable(&warrant, true, false));
        assert!(!warrant_is_enforceable(&warrant, true, true));
        assert!(!warrant_is_enforceable(&warrant, false, false));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traffic::{TrafficContactResolution, TrafficMovementKind};

    fn contact() -> TrafficContact {
        TrafficContact {
            contact_id: 7,
            catalog_id: 72,
            class_name: "Smollett".into(),
            ship_name: "Test".into(),
            transponder: "CT-7".into(),
            operator_name: "Test".into(),
            role: "merchant".into(),
            displacement_millitons: 400_000,
            origin_system_id: 1,
            destination_system_id: 2,
            movement: TrafficMovementKind::Departure,
            edge_second: 500,
            resolution: TrafficContactResolution::Identified,
            confidence_percent: 100,
            player_owned: false,
            online_controlled: false,
            attachment: crate::traffic::TrafficAttachment::Spaceborne,
        }
    }

    #[test]
    fn ranks_salary_and_command_limits_follow_settled_thresholds() {
        assert_eq!(naval_grade(0).name, "Lieutenant");
        assert_eq!(naval_grade(10).maximum_command_tons, Some(1_000));
        assert_eq!(naval_grade(70).name, "Commodore");
        assert_eq!(naval_monthly_salary(70), 14_000);
        assert_eq!(eligible_naval_grade_index(45), 3);
        assert_eq!(naval_salary_for_grade(2), 10_000);
    }

    #[test]
    fn opportunities_refer_to_existing_contacts() {
        let offered =
            opportunity_for_contact(CombatCareerMode::Pirate, 1, 100, 86_400, &contact(), 5)
                .unwrap();
        assert_eq!(offered.target_contact_id, 7);
        assert!(matches!(
            offered.kind,
            OpportunityKind::PirateLead | OpportunityKind::PirateCommission
        ));
    }

    #[test]
    fn prize_percentages_are_bounded_by_career() {
        let privateer = prize_terms(CombatCareerMode::Privateer, 1_000_000, 100, 10, 2);
        assert_eq!(privateer.percent, 30);
        assert_eq!(
            privateer.maximum_advance_credits,
            privateer.settlement_credits / 2
        );
        for entropy in 0..100 {
            assert!((10..=30).contains(
                &prize_terms(CombatCareerMode::Pirate, 1_000_000, 100, 5, entropy).percent
            ));
        }
    }
}
