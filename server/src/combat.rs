//! Deterministic CE vessel-combat rules.
//!
//! This module is deliberately independent of sockets and storage.  A combat
//! round is one ordered engine transaction fed with joint orders prepared from
//! a shared round-start view.  Persistence can therefore journal the input and
//! resulting state without giving any participant a later initiative slot an
//! information advantage while orders are being collected.

use std::collections::{BTreeMap, BTreeSet};

use crate::creation::{ship_combat_spec, ship_status_spec};

pub const COMBAT_RULES_REVISION: u16 = 1;
pub const COMBAT_TURN_SECONDS: u64 = 1_000;
pub const ORDER_WINDOW_REAL_MILLISECONDS: u64 = 35_714;
pub const DEFAULT_VICTORY_THRESHOLD_PERCENT: u8 = 70;
pub const MAX_SEARCH_CANDIDATES: usize = 64;
pub const SEARCH_FINALISTS: usize = 8;
pub const SEARCH_ROLLOUTS: usize = 256;
pub const SEARCH_HORIZON_ROUNDS: u8 = 3;

const RULES: &str = include_str!("../../catalog/combat-rules.toml");

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum RangeBand {
    Adjacent,
    Close,
    Short,
    Medium,
    Long,
    VeryLong,
    Distant,
}

impl RangeBand {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn increase(self) -> Self {
        match self {
            Self::Adjacent => Self::Close,
            Self::Close => Self::Short,
            Self::Short => Self::Medium,
            Self::Medium => Self::Long,
            Self::Long => Self::VeryLong,
            Self::VeryLong | Self::Distant => Self::Distant,
        }
    }

    pub fn decrease(self) -> Self {
        match self {
            Self::Adjacent | Self::Close => Self::Adjacent,
            Self::Short => Self::Close,
            Self::Medium => Self::Short,
            Self::Long => Self::Medium,
            Self::VeryLong => Self::Long,
            Self::Distant => Self::VeryLong,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DamageDice {
    pub dice: u8,
    pub modifier: i8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeaponRule {
    pub id: String,
    pub damage: DamageDice,
    pub difficulty_dm: [i8; 7],
    pub beam: bool,
    pub missile: bool,
    pub radiation: bool,
    pub meson: bool,
    pub sand: bool,
    pub bay: bool,
    pub ammunition_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeaponMount {
    pub mount_id: u16,
    pub catalog_mount_id: String,
    pub weapons: Vec<WeaponRule>,
    pub damage_hits: u8,
    pub battlefield_repair_hits: u8,
    pub fired_round: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmmunitionLot {
    pub ammunition_id: String,
    pub remaining: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VesselDisposition {
    Active,
    Withdrawing,
    SurrenderOffered,
    Surrendered,
    Abandoned,
    Captured,
    Destroyed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VesselState {
    pub vessel_id: u64,
    pub side: u16,
    pub name: String,
    pub catalog_id: u32,
    pub displacement_millitons: u64,
    pub thrust: u8,
    pub initiative: i16,
    pub hull_remaining: u16,
    pub structure_remaining: u16,
    pub armor_remaining: u16,
    pub bridge_hits: u8,
    pub sensors_hits: u8,
    pub maneuver_hits: u8,
    pub jump_hits: u8,
    pub power_hits: u8,
    pub fuel_hits: u8,
    pub hold_hits: u8,
    /// Temporary CE battlefield coverage for bridge, sensors, maneuver,
    /// jump, power, fuel, and hold damage, in that order.
    pub battlefield_repairs: [u8; 7],
    pub crew_hits: u16,
    pub weapons: Vec<WeaponMount>,
    pub ammunition: Vec<AmmunitionLot>,
    pub screens: BTreeSet<String>,
    pub reactions_remaining: u8,
    pub evasive_dm: i8,
    pub targeting_dm: i8,
    pub line_up_dm: i8,
    pub disposition: VesselDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Objective {
    Survive,
    Withdraw,
    Defeat,
    Capture,
    Protect,
    Inspect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrewAction {
    Hold,
    Coordinate,
    IncreaseInitiative,
    EvasiveManeuvers,
    LineUpShot,
    RangeCheckClose,
    RangeCheckOpen,
    BreakPursuit,
    SensorTargeting,
    ElectronicWarfare,
    DamageControl,
    Attack { mount_id: u16, target_id: u64 },
    Board { target_id: u64 },
    PrepareJump,
    LaunchEscapeCraft,
    OfferSurrender { to_vessel_id: u64 },
    AcceptSurrender { vessel_id: u64 },
    InspectContact { target_id: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReactionKind {
    Dodge,
    PointDefense,
    FireSand,
    TriggerNuclearDamper,
    TriggerMesonScreen,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JointOrder {
    pub vessel_id: u64,
    pub view_revision: u64,
    pub actions: Vec<CrewAction>,
    pub action_dms: Vec<i8>,
    pub reactions: Vec<ReactionKind>,
    pub reaction_dms: Vec<i8>,
    pub automated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingMissile {
    pub missile_id: u64,
    pub source_vessel_id: u64,
    pub target_vessel_id: u64,
    pub weapon: WeaponRule,
    pub hit_target: u8,
    pub impact_round: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardingState {
    pub attacker_id: u64,
    pub defender_id: u64,
    pub attacker_bonus: i8,
    pub defender_bonus: i8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatState {
    pub combat_id: u64,
    pub revision: u64,
    pub round: u16,
    pub round_started_second: u64,
    pub range: RangeBand,
    pub vessels: Vec<VesselState>,
    pub missiles: Vec<PendingMissile>,
    pub boarding: Vec<BoardingState>,
    pub complete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CombatEvent {
    Action {
        vessel_id: u64,
        description: String,
    },
    AttackMissed {
        vessel_id: u64,
        target_id: u64,
        mount_id: u16,
    },
    MissileLaunched {
        vessel_id: u64,
        target_id: u64,
        impact_round: u16,
    },
    Damage {
        vessel_id: u64,
        target_id: u64,
        damage: i16,
        hits: u16,
    },
    Withdrawal {
        vessel_id: u64,
    },
    SurrenderOffered {
        vessel_id: u64,
        to_vessel_id: u64,
    },
    SurrenderAccepted {
        vessel_id: u64,
        by_vessel_id: u64,
    },
    BoardingStarted {
        attacker_id: u64,
        defender_id: u64,
    },
    BattlefieldRepair {
        vessel_id: u64,
        targets: Vec<RepairTarget>,
    },
    InspectionCompleted {
        vessel_id: u64,
        target_id: u64,
    },
    VesselDestroyed {
        vessel_id: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairTarget {
    Bridge,
    Sensors,
    ManeuverDrive,
    JumpDrive,
    PowerPlant,
    FuelSystem,
    CargoHold,
    WeaponMount(u16),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoundResolution {
    pub state: CombatState,
    pub events: Vec<CombatEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationPolicy {
    pub revision: u64,
    pub minimum_victory_percent: u8,
    pub objective: Objective,
    pub permit_surrender: bool,
    pub permit_abandon_ship: bool,
}

impl Default for AutomationPolicy {
    fn default() -> Self {
        Self {
            revision: 1,
            minimum_victory_percent: DEFAULT_VICTORY_THRESHOLD_PERCENT,
            objective: Objective::Survive,
            permit_surrender: true,
            permit_abandon_ship: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationDecision {
    pub algorithm_revision: u16,
    pub view_revision: u64,
    pub estimated_success_percent: u8,
    pub branch: String,
    pub order: JointOrder,
}

pub fn weapon_rules() -> Result<Vec<WeaponRule>, String> {
    let mut rules = Vec::new();
    for section in RULES.split("[[weapon]]").skip(1) {
        let id = text(section, "id").ok_or("weapon rule without id")?;
        let dice = integer(section, "dice").ok_or_else(|| format!("{id}: missing dice"))?;
        let modifier =
            signed_integer(section, "modifier").ok_or_else(|| format!("{id}: missing modifier"))?;
        let difficulty = signed_array(section, "difficulty_dm");
        let difficulty_dm: [i8; 7] = difficulty
            .try_into()
            .map_err(|_| format!("{id}: difficulty_dm must contain seven entries"))?;
        let traits = string_array(section, "traits");
        rules.push(WeaponRule {
            id,
            damage: DamageDice {
                dice: dice.try_into().map_err(|_| "weapon dice overflow")?,
                modifier: modifier
                    .try_into()
                    .map_err(|_| "weapon modifier overflow")?,
            },
            difficulty_dm,
            beam: traits.iter().any(|value| value == "beam"),
            missile: traits.iter().any(|value| value == "missile"),
            radiation: traits.iter().any(|value| value == "radiation"),
            meson: traits.iter().any(|value| value == "meson"),
            sand: traits.iter().any(|value| value == "sand"),
            bay: traits.iter().any(|value| value == "bay"),
            ammunition_id: text(section, "ammunition"),
        });
    }
    rules.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(rules)
}

pub fn materialize_vessel(
    vessel_id: u64,
    side: u16,
    name: impl Into<String>,
    catalog_id: u32,
    initiative: i16,
) -> Result<VesselState, String> {
    let status = ship_status_spec(catalog_id)
        .ok_or_else(|| format!("unknown ship catalog id {catalog_id}"))?;
    let combat = ship_combat_spec(catalog_id)
        .ok_or_else(|| format!("ship {catalog_id} has no combat specification"))?;
    let rules = weapon_rules()?;
    let rule_map = rules
        .into_iter()
        .map(|rule| (rule.id.clone(), rule))
        .collect::<BTreeMap<_, _>>();
    let weapons = combat
        .weapons
        .iter()
        .enumerate()
        .map(|(index, mount)| {
            let fitted = mount
                .weapons
                .iter()
                .map(|id| {
                    rule_map
                        .get(id)
                        .cloned()
                        .ok_or_else(|| format!("ship {catalog_id} installs unknown weapon {id}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(WeaponMount {
                mount_id: u16::try_from(index + 1).map_err(|_| "too many weapon mounts")?,
                catalog_mount_id: mount.mount_id.clone(),
                weapons: fitted,
                damage_hits: 0,
                battlefield_repair_hits: 0,
                fired_round: 0,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let hull = u16::try_from((status.displacement_millitons / 50_000).max(1)).unwrap_or(u16::MAX);
    Ok(VesselState {
        vessel_id,
        side,
        name: name.into(),
        catalog_id,
        displacement_millitons: status.displacement_millitons,
        thrust: status.thrust_g,
        initiative,
        hull_remaining: hull,
        structure_remaining: hull,
        armor_remaining: combat.armor_points,
        bridge_hits: 0,
        sensors_hits: 0,
        maneuver_hits: 0,
        jump_hits: 0,
        power_hits: 0,
        fuel_hits: 0,
        hold_hits: 0,
        battlefield_repairs: [0; 7],
        crew_hits: 0,
        weapons,
        ammunition: combat
            .ammunition
            .into_iter()
            .map(|lot| AmmunitionLot {
                ammunition_id: lot.ammunition_id,
                remaining: lot.quantity,
            })
            .collect(),
        screens: combat.screens.into_iter().collect(),
        reactions_remaining: reactions_for_initiative(initiative),
        evasive_dm: 0,
        targeting_dm: 0,
        line_up_dm: 0,
        disposition: VesselDisposition::Active,
    })
}

pub fn reactions_for_initiative(initiative: i16) -> u8 {
    match initiative {
        i16::MIN..=4 => 1,
        5..=8 => 2,
        9..=12 => 3,
        _ => 4,
    }
}

/// Produces a safe, complete order that spends no ordnance and makes no
/// irreversible choice.
pub fn conservative_order(state: &CombatState, vessel_id: u64) -> Result<JointOrder, String> {
    let vessel = state
        .vessels
        .iter()
        .find(|vessel| vessel.vessel_id == vessel_id)
        .ok_or("vessel is not a participant")?;
    let mut actions = vec![CrewAction::Coordinate, CrewAction::EvasiveManeuvers];
    if vessel.hull_remaining <= 1 || vessel.structure_remaining <= 1 {
        actions.push(CrewAction::DamageControl);
    } else {
        actions.push(CrewAction::RangeCheckOpen);
        actions.push(CrewAction::BreakPursuit);
    }
    let action_dms = vec![0; actions.len()];
    Ok(JointOrder {
        vessel_id,
        view_revision: state.revision,
        actions,
        action_dms,
        reactions: vec![
            ReactionKind::PointDefense,
            ReactionKind::FireSand,
            ReactionKind::TriggerNuclearDamper,
            ReactionKind::TriggerMesonScreen,
            ReactionKind::Dodge,
        ],
        reaction_dms: vec![0; 5],
        automated: false,
    })
}

/// Fixed-work tactical policy.  Candidate enumeration is capped, the best
/// eight candidates receive exactly 256 deterministic three-round rollouts,
/// and no hidden vessel fields absent from `state` are consulted.
pub fn risk_directed_order(
    state: &CombatState,
    vessel_id: u64,
    policy: &AutomationPolicy,
) -> Result<AutomationDecision, String> {
    if policy.minimum_victory_percent > 100 {
        return Err("victory threshold exceeds 100 percent".into());
    }
    let vessel = state
        .vessels
        .iter()
        .find(|candidate| candidate.vessel_id == vessel_id)
        .ok_or("vessel is not a participant")?;
    let enemies = state
        .vessels
        .iter()
        .filter(|candidate| {
            candidate.side != vessel.side && candidate.disposition == VesselDisposition::Active
        })
        .collect::<Vec<_>>();
    if enemies.is_empty() {
        return Err("vessel has no active opponent".into());
    }
    let mut candidates = vec![conservative_order(state, vessel_id)?];
    for enemy in enemies.iter().take(MAX_SEARCH_CANDIDATES.saturating_sub(1)) {
        candidates.push(attack_order(state, vessel_id, enemy.vessel_id, true)?);
        candidates.push(attack_order(state, vessel_id, enemy.vessel_id, false)?);
        if state.range == RangeBand::Adjacent {
            candidates.push(capture_order(state, vessel_id, enemy.vessel_id)?);
        }
        if candidates.len() >= MAX_SEARCH_CANDIDATES {
            break;
        }
    }
    candidates.truncate(SEARCH_FINALISTS.min(candidates.len()));
    let mut estimates = Vec::with_capacity(candidates.len());
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        let mut successes = 0_u32;
        for rollout in 0..SEARCH_ROLLOUTS {
            successes += u32::from(rollout_candidate(
                state,
                vessel_id,
                policy.objective,
                candidate,
                candidate_index,
                rollout,
            ));
        }
        estimates
            .push(((successes * 100 + SEARCH_ROLLOUTS as u32 / 2) / SEARCH_ROLLOUTS as u32) as u8);
    }
    let best = estimates
        .iter()
        .enumerate()
        .max_by_key(|(index, estimate)| (**estimate, std::cmp::Reverse(*index)))
        .map(|(index, estimate)| (index, *estimate))
        .ok_or("tactical controller generated no candidate orders")?;
    let withdraw_estimate = estimates[0];
    let (branch, selected, estimate) = if best.1 >= policy.minimum_victory_percent {
        ("pursue-objective", best.0, best.1)
    } else {
        ("withdraw", 0, withdraw_estimate)
    };
    let mut order = candidates[selected].clone();
    order.automated = true;
    let target_id = enemies.first().map(|enemy| enemy.vessel_id);
    if effective_system_hits(vessel, RepairTarget::ManeuverDrive) >= 3
        && effective_system_hits(vessel, RepairTarget::JumpDrive) >= 3
    {
        if policy.permit_surrender {
            if let Some(target) = target_id {
                order.actions.push(CrewAction::OfferSurrender {
                    to_vessel_id: target,
                });
                order.action_dms.push(0);
            }
        } else if policy.permit_abandon_ship {
            order.actions.push(CrewAction::LaunchEscapeCraft);
            order.action_dms.push(0);
        }
    }
    Ok(AutomationDecision {
        algorithm_revision: COMBAT_RULES_REVISION,
        view_revision: state.revision,
        estimated_success_percent: estimate,
        branch: branch.into(),
        order,
    })
}

fn attack_order(
    state: &CombatState,
    vessel_id: u64,
    target_id: u64,
    supported: bool,
) -> Result<JointOrder, String> {
    let vessel = state
        .vessels
        .iter()
        .find(|candidate| candidate.vessel_id == vessel_id)
        .ok_or("vessel is not a participant")?;
    let mut actions = if supported {
        vec![
            CrewAction::Coordinate,
            CrewAction::LineUpShot,
            CrewAction::SensorTargeting,
        ]
    } else {
        Vec::new()
    };
    actions.extend(vessel.weapons.iter().map(|mount| CrewAction::Attack {
        mount_id: mount.mount_id,
        target_id,
    }));
    let action_dms = vec![0; actions.len()];
    let defensive = conservative_order(state, vessel_id)?;
    Ok(JointOrder {
        vessel_id,
        view_revision: state.revision,
        actions,
        action_dms,
        reactions: defensive.reactions,
        reaction_dms: defensive.reaction_dms,
        automated: true,
    })
}

fn capture_order(
    state: &CombatState,
    vessel_id: u64,
    target_id: u64,
) -> Result<JointOrder, String> {
    let defensive = conservative_order(state, vessel_id)?;
    Ok(JointOrder {
        vessel_id,
        view_revision: state.revision,
        actions: vec![CrewAction::Coordinate, CrewAction::Board { target_id }],
        action_dms: vec![0; 2],
        reactions: defensive.reactions,
        reaction_dms: defensive.reaction_dms,
        automated: true,
    })
}

fn rollout_candidate(
    state: &CombatState,
    vessel_id: u64,
    objective: Objective,
    candidate: &JointOrder,
    candidate_index: usize,
    rollout: usize,
) -> bool {
    let mut simulated = state.clone();
    simulated.combat_id = mix64(
        state.combat_id
            ^ (candidate_index as u64).rotate_left(21)
            ^ (rollout as u64).rotate_left(39),
    );
    for vessel in &mut simulated.vessels {
        let entropy = mix64(simulated.combat_id ^ vessel.vessel_id);
        vessel.initiative = vessel.initiative.saturating_add((entropy % 3) as i16 - 1);
    }
    for _ in 0..SEARCH_HORIZON_ROUNDS {
        if simulated.complete {
            break;
        }
        let Some(player) = simulated
            .vessels
            .iter()
            .find(|vessel| vessel.vessel_id == vessel_id)
        else {
            return false;
        };
        if !matches!(
            player.disposition,
            VesselDisposition::Active | VesselDisposition::Withdrawing
        ) {
            break;
        }
        let mut orders = Vec::new();
        for vessel in simulated
            .vessels
            .iter()
            .filter(|vessel| vessel.disposition == VesselDisposition::Active)
        {
            let mut order = if vessel.vessel_id == vessel_id {
                candidate.clone()
            } else {
                let target = simulated
                    .vessels
                    .iter()
                    .find(|target| {
                        target.side != vessel.side
                            && target.disposition == VesselDisposition::Active
                    })
                    .map(|target| target.vessel_id);
                match target.and_then(|target| {
                    attack_order(&simulated, vessel.vessel_id, target, false).ok()
                }) {
                    Some(order) => order,
                    None => match conservative_order(&simulated, vessel.vessel_id) {
                        Ok(order) => order,
                        Err(_) => return false,
                    },
                }
            };
            order.view_revision = simulated.revision;
            orders.push(order);
        }
        let Ok(resolution) = resolve_round(&simulated, &orders) else {
            return false;
        };
        simulated = resolution.state;
    }
    let side = simulated
        .vessels
        .iter()
        .find(|vessel| vessel.vessel_id == vessel_id)
        .map_or(0, |vessel| vessel.side);
    objective_satisfied(&simulated, vessel_id, side, objective)
}

fn objective_satisfied(
    state: &CombatState,
    vessel_id: u64,
    player_side: u16,
    objective: Objective,
) -> bool {
    let Some(player) = state
        .vessels
        .iter()
        .find(|vessel| vessel.vessel_id == vessel_id)
    else {
        return false;
    };
    let survives = !matches!(
        player.disposition,
        VesselDisposition::Destroyed
            | VesselDisposition::Captured
            | VesselDisposition::Surrendered
            | VesselDisposition::Abandoned
    );
    let active_enemy = state.vessels.iter().any(|vessel| {
        vessel.side != player_side && vessel.disposition == VesselDisposition::Active
    });
    match objective {
        Objective::Survive | Objective::Protect => survives,
        Objective::Withdraw => {
            survives && (player.disposition == VesselDisposition::Withdrawing || !active_enemy)
        }
        Objective::Defeat | Objective::Inspect => survives && !active_enemy,
        Objective::Capture => {
            survives
                && state.vessels.iter().any(|vessel| {
                    vessel.side != player_side
                        && matches!(
                            vessel.disposition,
                            VesselDisposition::Captured | VesselDisposition::Surrendered
                        )
                })
        }
    }
}

pub fn resolve_round(
    state: &CombatState,
    orders: &[JointOrder],
) -> Result<RoundResolution, String> {
    if state.complete {
        return Err("combat is complete".into());
    }
    let mut result = state.clone();
    let expected = result
        .vessels
        .iter()
        .filter(|vessel| vessel.disposition == VesselDisposition::Active)
        .map(|vessel| vessel.vessel_id)
        .collect::<BTreeSet<_>>();
    let submitted = orders
        .iter()
        .map(|order| order.vessel_id)
        .collect::<BTreeSet<_>>();
    if submitted != expected || orders.len() != submitted.len() {
        return Err("one joint order is required for every active vessel".into());
    }
    if orders
        .iter()
        .any(|order| order.view_revision != state.revision)
    {
        return Err("joint order was prepared from a stale combat view".into());
    }
    if orders.iter().any(|order| {
        order.actions.len() != order.action_dms.len()
            || order.reactions.len() != order.reaction_dms.len()
    }) {
        return Err("combat task modifiers do not match the joint order".into());
    }
    let mut ordered = orders.to_vec();
    ordered.sort_by(|left, right| {
        let l = state
            .vessels
            .iter()
            .find(|v| v.vessel_id == left.vessel_id)
            .unwrap();
        let r = state
            .vessels
            .iter()
            .find(|v| v.vessel_id == right.vessel_id)
            .unwrap();
        r.initiative
            .cmp(&l.initiative)
            .then_with(|| r.thrust.cmp(&l.thrust))
            .then_with(|| l.vessel_id.cmp(&r.vessel_id))
    });
    for vessel in &mut result.vessels {
        vessel.reactions_remaining = reactions_for_initiative(vessel.initiative);
        vessel.evasive_dm = 0;
        vessel.targeting_dm = 0;
        vessel.line_up_dm = 0;
    }
    let mut events = Vec::new();
    resolve_missile_impacts(&mut result, &ordered, &mut events)?;
    for order in &ordered {
        if result
            .vessels
            .iter()
            .find(|v| v.vessel_id == order.vessel_id)
            .is_none_or(|v| v.disposition != VesselDisposition::Active)
        {
            continue;
        }
        for (action_index, action) in order.actions.iter().enumerate() {
            resolve_action(
                &mut result,
                order,
                *action,
                order.action_dms[action_index],
                action_index,
                &mut events,
            )?;
        }
    }
    resolve_boarding(&mut result, &mut events);
    result.round = result.round.saturating_add(1);
    result.round_started_second = result
        .round_started_second
        .saturating_add(COMBAT_TURN_SECONDS);
    result.revision = result.revision.saturating_add(1);
    let active_sides = result
        .vessels
        .iter()
        .filter(|v| v.disposition == VesselDisposition::Active)
        .map(|v| v.side)
        .collect::<BTreeSet<_>>();
    result.complete = active_sides.len() <= 1;
    Ok(RoundResolution {
        state: result,
        events,
    })
}

fn resolve_action(
    state: &mut CombatState,
    order: &JointOrder,
    action: CrewAction,
    task_dm: i8,
    action_index: usize,
    events: &mut Vec<CombatEvent>,
) -> Result<(), String> {
    let actor_index = state
        .vessels
        .iter()
        .position(|v| v.vessel_id == order.vessel_id)
        .ok_or("acting vessel disappeared")?;
    match action {
        CrewAction::Hold | CrewAction::PrepareJump | CrewAction::ElectronicWarfare => {}
        CrewAction::Coordinate => {
            if action_check(state, order, action_index, task_dm, 8) >= 0 {
                state.vessels[actor_index].targeting_dm =
                    state.vessels[actor_index].targeting_dm.saturating_add(1);
                events.push(CombatEvent::Action {
                    vessel_id: order.vessel_id,
                    description: "captain coordinates the crew".into(),
                });
            }
        }
        CrewAction::IncreaseInitiative => {
            let effect = action_check(state, order, action_index, task_dm, 8);
            if effect > 0 {
                state.vessels[actor_index].initiative =
                    state.vessels[actor_index].initiative.saturating_add(effect);
            }
        }
        CrewAction::EvasiveManeuvers => {
            let effect = action_check(state, order, action_index, task_dm, 8);
            if effect >= 0 {
                state.vessels[actor_index].evasive_dm = if effect >= 6 { -2 } else { -1 };
            }
        }
        CrewAction::LineUpShot => {
            let effect = action_check(state, order, action_index, task_dm, 8);
            if effect >= 0 {
                state.vessels[actor_index].line_up_dm = if effect >= 6 { 2 } else { 1 };
            }
        }
        CrewAction::SensorTargeting => {
            let effect = action_check(state, order, action_index, task_dm, 8);
            if effect >= 0 {
                state.vessels[actor_index].targeting_dm = if effect >= 6 { 2 } else { 1 };
            }
        }
        CrewAction::RangeCheckClose => {
            if action_check(state, order, action_index, task_dm, 8) >= 0 {
                state.range = state.range.decrease();
            }
        }
        CrewAction::RangeCheckOpen | CrewAction::BreakPursuit => {
            if action_check(state, order, action_index, task_dm, 8) < 0 {
                return Ok(());
            }
            state.range = state.range.increase();
            if state.range == RangeBand::Distant {
                state.vessels[actor_index].disposition = VesselDisposition::Withdrawing;
                events.push(CombatEvent::Withdrawal {
                    vessel_id: order.vessel_id,
                });
            }
        }
        CrewAction::DamageControl => {
            let effect = action_check(state, order, action_index, task_dm, 8);
            if effect >= 0 {
                let attempts = 1 + u8::try_from(effect / 3).unwrap_or(0).min(2);
                let mut targets = Vec::new();
                for _ in 0..attempts {
                    if let Some(target) =
                        apply_one_battlefield_repair(&mut state.vessels[actor_index])
                    {
                        targets.push(target);
                    }
                }
                if !targets.is_empty() {
                    events.push(CombatEvent::BattlefieldRepair {
                        vessel_id: order.vessel_id,
                        targets,
                    });
                } else {
                    events.push(CombatEvent::Action {
                        vessel_id: order.vessel_id,
                        description: "damage-control parties find no uncovered battle damage"
                            .into(),
                    });
                }
            } else {
                events.push(CombatEvent::Action {
                    vessel_id: order.vessel_id,
                    description: "damage-control parties cannot establish a battlefield repair"
                        .into(),
                });
            }
        }
        CrewAction::Attack {
            mount_id,
            target_id,
        } => resolve_attack(state, order, mount_id, target_id, task_dm, events)?,
        CrewAction::Board { target_id } => {
            if state.range != RangeBand::Adjacent {
                events.push(CombatEvent::Action {
                    vessel_id: order.vessel_id,
                    description: "boarding parties cannot engage after the range opens".into(),
                });
                return Ok(());
            }
            state.boarding.push(BoardingState {
                attacker_id: order.vessel_id,
                defender_id: target_id,
                attacker_bonus: 0,
                defender_bonus: 0,
            });
            events.push(CombatEvent::BoardingStarted {
                attacker_id: order.vessel_id,
                defender_id: target_id,
            });
        }
        CrewAction::LaunchEscapeCraft => {
            state.vessels[actor_index].disposition = VesselDisposition::Abandoned
        }
        CrewAction::OfferSurrender { to_vessel_id } => {
            state.vessels[actor_index].disposition = VesselDisposition::SurrenderOffered;
            events.push(CombatEvent::SurrenderOffered {
                vessel_id: order.vessel_id,
                to_vessel_id,
            });
        }
        CrewAction::AcceptSurrender { vessel_id } => {
            let surrendered = state
                .vessels
                .iter_mut()
                .find(|v| v.vessel_id == vessel_id)
                .ok_or("surrendering vessel is absent")?;
            if surrendered.disposition != VesselDisposition::SurrenderOffered {
                return Err("target has not offered surrender".into());
            }
            surrendered.disposition = VesselDisposition::Surrendered;
            events.push(CombatEvent::SurrenderAccepted {
                vessel_id,
                by_vessel_id: order.vessel_id,
            });
        }
        CrewAction::InspectContact { target_id } => {
            let target = state
                .vessels
                .iter()
                .find(|vessel| vessel.vessel_id == target_id)
                .ok_or("inspection target is absent")?;
            if target.vessel_id == order.vessel_id {
                return Err("a vessel cannot inspect itself".into());
            }
            if action_check(state, order, action_index, task_dm, 8) >= 0 {
                events.push(CombatEvent::InspectionCompleted {
                    vessel_id: order.vessel_id,
                    target_id,
                });
            }
        }
    }
    Ok(())
}

fn repair_index(target: RepairTarget) -> Option<usize> {
    match target {
        RepairTarget::Bridge => Some(0),
        RepairTarget::Sensors => Some(1),
        RepairTarget::ManeuverDrive => Some(2),
        RepairTarget::JumpDrive => Some(3),
        RepairTarget::PowerPlant => Some(4),
        RepairTarget::FuelSystem => Some(5),
        RepairTarget::CargoHold => Some(6),
        RepairTarget::WeaponMount(_) => None,
    }
}

fn system_hits(vessel: &VesselState, target: RepairTarget) -> u8 {
    match target {
        RepairTarget::Bridge => vessel.bridge_hits,
        RepairTarget::Sensors => vessel.sensors_hits,
        RepairTarget::ManeuverDrive => vessel.maneuver_hits,
        RepairTarget::JumpDrive => vessel.jump_hits,
        RepairTarget::PowerPlant => vessel.power_hits,
        RepairTarget::FuelSystem => vessel.fuel_hits,
        RepairTarget::CargoHold => vessel.hold_hits,
        RepairTarget::WeaponMount(mount_id) => vessel
            .weapons
            .iter()
            .find(|mount| mount.mount_id == mount_id)
            .map_or(0, |mount| mount.damage_hits),
    }
}

fn effective_system_hits(vessel: &VesselState, target: RepairTarget) -> u8 {
    match repair_index(target) {
        Some(index) => {
            system_hits(vessel, target).saturating_sub(vessel.battlefield_repairs[index])
        }
        None => {
            let RepairTarget::WeaponMount(mount_id) = target else {
                return system_hits(vessel, target);
            };
            vessel
                .weapons
                .iter()
                .find(|mount| mount.mount_id == mount_id)
                .map_or(0, |mount| {
                    mount
                        .damage_hits
                        .saturating_sub(mount.battlefield_repair_hits)
                })
        }
    }
}

fn apply_one_battlefield_repair(vessel: &mut VesselState) -> Option<RepairTarget> {
    for target in [
        RepairTarget::PowerPlant,
        RepairTarget::ManeuverDrive,
        RepairTarget::JumpDrive,
        RepairTarget::Bridge,
        RepairTarget::Sensors,
        RepairTarget::FuelSystem,
        RepairTarget::CargoHold,
    ] {
        let index = repair_index(target).unwrap();
        if vessel.battlefield_repairs[index] < system_hits(vessel, target) {
            vessel.battlefield_repairs[index] = vessel.battlefield_repairs[index].saturating_add(1);
            return Some(target);
        }
    }
    if let Some(mount) = vessel
        .weapons
        .iter_mut()
        .find(|mount| mount.battlefield_repair_hits < mount.damage_hits)
    {
        mount.battlefield_repair_hits = mount.battlefield_repair_hits.saturating_add(1);
        return Some(RepairTarget::WeaponMount(mount.mount_id));
    }
    None
}

fn resolve_attack(
    state: &mut CombatState,
    order: &JointOrder,
    mount_id: u16,
    target_id: u64,
    task_dm: i8,
    events: &mut Vec<CombatEvent>,
) -> Result<(), String> {
    let actor_index = state
        .vessels
        .iter()
        .position(|v| v.vessel_id == order.vessel_id)
        .ok_or("attacker is absent")?;
    let target_index = state
        .vessels
        .iter()
        .position(|v| v.vessel_id == target_id)
        .ok_or("target is absent")?;
    if state.vessels[actor_index].side == state.vessels[target_index].side {
        return Err("friendly vessel is not a legal target".into());
    }
    let mount_index = state.vessels[actor_index]
        .weapons
        .iter()
        .position(|mount| mount.mount_id == mount_id)
        .ok_or("weapon mount is absent")?;
    if state.vessels[actor_index].weapons[mount_index].fired_round == state.round {
        return Err("weapon mount has already fired this round".into());
    }
    if state.vessels[actor_index].weapons[mount_index]
        .damage_hits
        .saturating_sub(state.vessels[actor_index].weapons[mount_index].battlefield_repair_hits)
        >= 2
    {
        return Err("weapon mount is disabled".into());
    }
    state.vessels[actor_index].weapons[mount_index].fired_round = state.round;
    let weapons = state.vessels[actor_index].weapons[mount_index]
        .weapons
        .clone();
    for (weapon_index, weapon) in weapons.into_iter().enumerate() {
        let difficulty = weapon.difficulty_dm[state.range.index()];
        if difficulty <= -90 {
            continue;
        }
        if let Some(ammunition_id) = &weapon.ammunition_id {
            let Some(lot) = state.vessels[actor_index]
                .ammunition
                .iter_mut()
                .find(|lot| lot.ammunition_id == *ammunition_id && lot.remaining != 0)
            else {
                continue;
            };
            lot.remaining -= 1;
        }
        let entropy = mix64(
            state.combat_id
                ^ state.revision.rotate_left(5)
                ^ order.vessel_id.rotate_left(13)
                ^ target_id.rotate_left(27)
                ^ u64::from(mount_id).rotate_left(39)
                ^ weapon_index as u64,
        );
        let roll = d6(entropy, 0) + d6(entropy, 8);
        let attack_total = roll
            + i16::from(difficulty)
            + i16::from(task_dm)
            + i16::from(
                state.vessels[actor_index].targeting_dm
                    + state.vessels[actor_index].line_up_dm
                    + state.vessels[target_index].evasive_dm,
            )
            - if state.vessels[actor_index].weapons[mount_index]
                .damage_hits
                .saturating_sub(
                    state.vessels[actor_index].weapons[mount_index].battlefield_repair_hits,
                )
                == 1
            {
                2
            } else {
                0
            };
        if weapon.missile {
            let effect = attack_total - 8;
            let hit_target = if effect <= -6 {
                11
            } else if effect < 0 {
                10
            } else if effect == 0 {
                8
            } else if effect < 6 {
                7
            } else {
                6
            };
            let delay = if state.range >= RangeBand::VeryLong {
                2
            } else {
                1
            };
            state.missiles.push(PendingMissile {
                missile_id: entropy,
                source_vessel_id: order.vessel_id,
                target_vessel_id: target_id,
                weapon,
                hit_target,
                impact_round: state.round.saturating_add(delay),
            });
            events.push(CombatEvent::MissileLaunched {
                vessel_id: order.vessel_id,
                target_id,
                impact_round: state.round.saturating_add(delay),
            });
        } else if attack_total >= 8 {
            apply_weapon_damage(
                state,
                actor_index,
                target_index,
                &weapon,
                entropy.rotate_left(17),
                events,
            );
        } else {
            events.push(CombatEvent::AttackMissed {
                vessel_id: order.vessel_id,
                target_id,
                mount_id,
            });
        }
    }
    Ok(())
}

fn resolve_missile_impacts(
    state: &mut CombatState,
    orders: &[JointOrder],
    events: &mut Vec<CombatEvent>,
) -> Result<(), String> {
    let mut future = Vec::new();
    for missile in std::mem::take(&mut state.missiles) {
        if missile.impact_round > state.round {
            future.push(missile);
            continue;
        }
        let Some(source) = state
            .vessels
            .iter()
            .position(|v| v.vessel_id == missile.source_vessel_id)
        else {
            continue;
        };
        let Some(target) = state
            .vessels
            .iter()
            .position(|v| v.vessel_id == missile.target_vessel_id)
        else {
            continue;
        };
        let entropy = mix64(missile.missile_id ^ state.revision);
        let mut destroyed = false;
        if let Some(order) = orders
            .iter()
            .find(|order| order.vessel_id == missile.target_vessel_id)
        {
            if let Some(reaction_index) = order
                .reactions
                .iter()
                .position(|reaction| *reaction == ReactionKind::PointDefense)
                && state.vessels[target].reactions_remaining > 0
            {
                state.vessels[target].reactions_remaining -= 1;
                destroyed = d6(entropy, 16)
                    + d6(entropy, 24)
                    + i16::from(order.reaction_dms[reaction_index])
                    >= 8;
            }
        }
        if !destroyed && d6(entropy, 0) + d6(entropy, 8) >= i16::from(missile.hit_target) {
            apply_weapon_damage(
                state,
                source,
                target,
                &missile.weapon,
                entropy.rotate_left(31),
                events,
            );
        }
    }
    state.missiles = future;
    Ok(())
}

fn apply_weapon_damage(
    state: &mut CombatState,
    source: usize,
    target: usize,
    weapon: &WeaponRule,
    entropy: u64,
    events: &mut Vec<CombatEvent>,
) {
    let rolled = (0..weapon.damage.dice)
        .map(|index| d6(entropy, u32::from(index) * 8))
        .sum::<i16>()
        + i16::from(weapon.damage.modifier);
    let armor = if weapon.meson {
        0
    } else {
        state.vessels[target].armor_remaining as i16
    };
    let penetrating = (rolled - armor).max(0);
    let hit_groups = damage_hit_groups(penetrating);
    let mut hits = 0;
    for (ordinal, group) in hit_groups.into_iter().enumerate() {
        apply_hit_group(
            &mut state.vessels[target],
            group,
            mix64(entropy ^ ordinal as u64),
            weapon.meson,
        );
        hits += u16::from(group);
    }
    if weapon.radiation && penetrating > 0 {
        state.vessels[target].crew_hits = state.vessels[target].crew_hits.saturating_add(1);
    }
    events.push(CombatEvent::Damage {
        vessel_id: state.vessels[source].vessel_id,
        target_id: state.vessels[target].vessel_id,
        damage: penetrating,
        hits,
    });
    if state.vessels[target].structure_remaining == 0 {
        state.vessels[target].disposition = VesselDisposition::Destroyed;
        events.push(CombatEvent::VesselDestroyed {
            vessel_id: state.vessels[target].vessel_id,
        });
    }
}

pub fn damage_hit_groups(damage: i16) -> Vec<u8> {
    match damage {
        i16::MIN..=0 => vec![],
        1..=4 => vec![1],
        5..=8 => vec![1, 1],
        9..=12 => vec![2],
        13..=16 => vec![1, 1, 1],
        17..=20 => vec![1, 1, 2],
        21..=24 => vec![2, 2],
        25..=28 => vec![3],
        29..=32 => vec![3, 1],
        33..=36 => vec![3, 2],
        37..=40 => vec![3, 2, 1],
        41..=44 => vec![3, 3],
        _ => {
            let mut groups = vec![3, 3];
            let extra = damage - 44;
            groups.extend(std::iter::repeat_n(
                2,
                usize::try_from(extra / 6).unwrap_or(0),
            ));
            groups.extend(std::iter::repeat_n(
                1,
                usize::try_from((extra % 6) / 3).unwrap_or(0),
            ));
            groups
        }
    }
}

fn apply_hit_group(vessel: &mut VesselState, count: u8, entropy: u64, force_internal: bool) {
    let roll = d6(entropy, 0) + d6(entropy, 8);
    for _ in 0..count {
        let internal = force_internal || vessel.hull_remaining == 0;
        match (roll, internal, vessel.displacement_millitons < 100_000) {
            (2 | 6 | 8 | 12, false, _) | (2 | 6 | 8, _, true) => {
                vessel.hull_remaining = vessel.hull_remaining.saturating_sub(1)
            }
            (2 | 6 | 8, true, _) => {
                vessel.structure_remaining = vessel.structure_remaining.saturating_sub(1)
            }
            (3 | 11, false, false) => vessel.sensors_hits = vessel.sensors_hits.saturating_add(1),
            (3 | 11, true, false) => vessel.power_hits = vessel.power_hits.saturating_add(1),
            (4 | 10, false, _) => vessel.maneuver_hits = vessel.maneuver_hits.saturating_add(1),
            (4 | 10, true, false) => vessel.jump_hits = vessel.jump_hits.saturating_add(1),
            (5, false, false) | (9, _, true) => hit_random_mount(vessel, entropy),
            (5, true, false) => hit_random_mount(vessel, entropy),
            (7, false, _) => {
                if vessel.armor_remaining > 0 {
                    vessel.armor_remaining -= 1
                } else {
                    vessel.hull_remaining = vessel.hull_remaining.saturating_sub(1)
                }
            }
            (7 | 11 | 12, true, _) => vessel.crew_hits = vessel.crew_hits.saturating_add(1),
            (9, false, false) | (5, _, true) => {
                vessel.fuel_hits = vessel.fuel_hits.saturating_add(1)
            }
            (9, true, false) | (4, _, true) => {
                vessel.hold_hits = vessel.hold_hits.saturating_add(1)
            }
            (10, _, true) => vessel.maneuver_hits = vessel.maneuver_hits.saturating_add(1),
            (3, _, true) => vessel.power_hits = vessel.power_hits.saturating_add(1),
            _ => vessel.structure_remaining = vessel.structure_remaining.saturating_sub(1),
        }
    }
}

fn hit_random_mount(vessel: &mut VesselState, entropy: u64) {
    if vessel.weapons.is_empty() {
        vessel.hull_remaining = vessel.hull_remaining.saturating_sub(1);
    } else {
        let index = entropy as usize % vessel.weapons.len();
        vessel.weapons[index].damage_hits = vessel.weapons[index].damage_hits.saturating_add(1);
    }
}

fn resolve_boarding(state: &mut CombatState, events: &mut Vec<CombatEvent>) {
    let mut remaining = Vec::new();
    for mut boarding in std::mem::take(&mut state.boarding) {
        let entropy = mix64(
            state.combat_id
                ^ state.revision
                ^ boarding.attacker_id.rotate_left(19)
                ^ boarding.defender_id.rotate_left(37),
        );
        let attacker = d6(entropy, 0) + d6(entropy, 8) + i16::from(boarding.attacker_bonus);
        let defender = d6(entropy, 16) + d6(entropy, 24) + i16::from(boarding.defender_bonus);
        let effect = attacker - defender;
        if effect >= 6 {
            if let Some(vessel) = state
                .vessels
                .iter_mut()
                .find(|v| v.vessel_id == boarding.defender_id)
            {
                vessel.disposition = VesselDisposition::Captured;
            }
        } else if effect <= -6 {
            // Driven off: the boarding record is intentionally discarded.
        } else {
            if effect >= 0 {
                boarding.attacker_bonus = 2;
            } else {
                boarding.defender_bonus = 2;
            }
            if let Some(vessel) = state
                .vessels
                .iter_mut()
                .find(|v| v.vessel_id == boarding.defender_id)
            {
                vessel.structure_remaining = vessel.structure_remaining.saturating_sub(1);
                if vessel.structure_remaining == 0 {
                    vessel.disposition = VesselDisposition::Destroyed;
                    events.push(CombatEvent::VesselDestroyed {
                        vessel_id: vessel.vessel_id,
                    });
                }
            }
            remaining.push(boarding);
        }
    }
    state.boarding = remaining;
}

fn d6(entropy: u64, shift: u32) -> i16 {
    ((entropy.rotate_right(shift) % 6) + 1) as i16
}

fn action_check(
    state: &CombatState,
    order: &JointOrder,
    action_index: usize,
    task_dm: i8,
    difficulty: i16,
) -> i16 {
    let entropy = mix64(
        state.combat_id
            ^ state.revision.rotate_left(11)
            ^ order.vessel_id.rotate_left(23)
            ^ (action_index as u64).rotate_left(37),
    );
    d6(entropy, 0) + d6(entropy, 8) + i16::from(task_dm) - difficulty
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn integer(source: &str, key: &str) -> Option<u64> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

fn signed_integer(source: &str, key: &str) -> Option<i64> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

fn text(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key).then(|| value.trim().trim_matches('"').to_owned())
    })
}

fn string_array(source: &str, key: &str) -> Vec<String> {
    text(source, key)
        .unwrap_or_default()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

fn signed_array(source: &str, key: &str) -> Vec<i8> {
    text(source, key)
        .unwrap_or_default()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .filter_map(|item| item.trim().parse().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_armament_resolves_to_combat_rules() {
        let mut admitted = 0;
        for catalog_id in 1..=10_000 {
            let Some(spec) = ship_combat_spec(catalog_id) else {
                continue;
            };
            admitted += 1;
            let rules = weapon_rules()
                .unwrap()
                .into_iter()
                .map(|rule| rule.id)
                .collect::<BTreeSet<_>>();
            for mount in spec.weapons {
                for weapon in mount.weapons {
                    assert!(
                        rules.contains(&weapon),
                        "ship {catalog_id} installs unresolved weapon {weapon}"
                    );
                }
            }
        }
        assert_eq!(admitted, 213);
    }

    #[test]
    fn materializes_real_catalog_loadout_and_ammunition() {
        let vessel = materialize_vessel(1, 1, "Smollett", 72, 9).unwrap();
        assert_eq!(vessel.weapons.len(), 4);
        assert!(vessel.weapons.iter().any(|mount| {
            mount
                .weapons
                .iter()
                .any(|weapon| weapon.id == "particle-beam-barbette")
        }));
        assert!(
            vessel
                .ammunition
                .iter()
                .any(|lot| lot.ammunition_id == "standard-missile" && lot.remaining == 72)
        );
    }

    #[test]
    fn stale_or_partial_joint_orders_are_rejected() {
        let first = materialize_vessel(1, 1, "A", 72, 10).unwrap();
        let second = materialize_vessel(2, 2, "B", 72, 8).unwrap();
        let state = CombatState {
            combat_id: 9,
            revision: 4,
            round: 1,
            round_started_second: 100,
            range: RangeBand::Short,
            vessels: vec![first, second],
            missiles: vec![],
            boarding: vec![],
            complete: false,
        };
        assert!(resolve_round(&state, &[conservative_order(&state, 1).unwrap()]).is_err());
        let mut stale = conservative_order(&state, 1).unwrap();
        stale.view_revision -= 1;
        assert!(resolve_round(&state, &[stale, conservative_order(&state, 2).unwrap()]).is_err());
    }

    #[test]
    fn round_is_deterministic_and_advances_one_kilosecond() {
        let first = materialize_vessel(1, 1, "A", 72, 10).unwrap();
        let second = materialize_vessel(2, 2, "B", 72, 8).unwrap();
        let state = CombatState {
            combat_id: 9,
            revision: 4,
            round: 1,
            round_started_second: 100,
            range: RangeBand::Short,
            vessels: vec![first, second],
            missiles: vec![],
            boarding: vec![],
            complete: false,
        };
        let orders = [
            risk_directed_order(&state, 1, &AutomationPolicy::default())
                .unwrap()
                .order,
            risk_directed_order(&state, 2, &AutomationPolicy::default())
                .unwrap()
                .order,
        ];
        let a = resolve_round(&state, &orders).unwrap();
        let b = resolve_round(&state, &orders).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.state.round_started_second, 1_100);
    }

    #[test]
    fn opening_range_before_a_boarding_order_fails_the_action_not_the_round() {
        let first = materialize_vessel(1, 1, "A", 72, 10).unwrap();
        let second = materialize_vessel(2, 2, "B", 72, 8).unwrap();
        let state = CombatState {
            combat_id: 10,
            revision: 4,
            round: 1,
            round_started_second: 100,
            range: RangeBand::Adjacent,
            vessels: vec![first, second],
            missiles: vec![],
            boarding: vec![],
            complete: false,
        };
        let open_range = JointOrder {
            vessel_id: 1,
            view_revision: state.revision,
            actions: vec![CrewAction::RangeCheckOpen],
            action_dms: vec![100],
            reactions: vec![],
            reaction_dms: vec![],
            automated: false,
        };
        let board = JointOrder {
            vessel_id: 2,
            view_revision: state.revision,
            actions: vec![CrewAction::Board { target_id: 1 }],
            action_dms: vec![0],
            reactions: vec![],
            reaction_dms: vec![],
            automated: false,
        };

        let resolution = resolve_round(&state, &[open_range, board]).unwrap();

        assert!(resolution.state.boarding.is_empty());
        assert!(resolution.events.iter().any(|event| matches!(
            event,
            CombatEvent::Action {
                vessel_id: 2,
                description
            } if description == "boarding parties cannot engage after the range opens"
        )));
    }

    #[test]
    fn automation_has_a_fixed_reproducible_estimate() {
        let first = materialize_vessel(1, 1, "A", 72, 10).unwrap();
        let second = materialize_vessel(2, 2, "B", 72, 8).unwrap();
        let state = CombatState {
            combat_id: 99,
            revision: 1,
            round: 1,
            round_started_second: 0,
            range: RangeBand::Short,
            vessels: vec![first, second],
            missiles: vec![],
            boarding: vec![],
            complete: false,
        };
        let a = risk_directed_order(&state, 1, &AutomationPolicy::default()).unwrap();
        let b = risk_directed_order(&state, 1, &AutomationPolicy::default()).unwrap();
        assert_eq!(a, b);
        assert!(a.estimated_success_percent <= 100);

        let policy = AutomationPolicy::default();
        let mut candidates = vec![conservative_order(&state, 1).unwrap()];
        candidates.push(attack_order(&state, 1, 2, true).unwrap());
        candidates.push(attack_order(&state, 1, 2, false).unwrap());
        let selected_index = candidates
            .iter()
            .position(|candidate| {
                candidate.actions == a.order.actions
                    && candidate.reactions == a.order.reactions
                    && candidate.action_dms == a.order.action_dms
                    && candidate.reaction_dms == a.order.reaction_dms
            })
            .unwrap();
        let successes = (0..SEARCH_ROLLOUTS)
            .filter(|rollout| {
                rollout_candidate(
                    &state,
                    1,
                    policy.objective,
                    &candidates[selected_index],
                    selected_index,
                    *rollout,
                )
            })
            .count();
        let expected = ((successes * 100 + SEARCH_ROLLOUTS / 2) / SEARCH_ROLLOUTS) as u8;
        assert_eq!(a.estimated_success_percent, expected);

        let all_estimates = candidates
            .iter()
            .enumerate()
            .map(|(candidate_index, candidate)| {
                let successes = (0..SEARCH_ROLLOUTS)
                    .filter(|rollout| {
                        rollout_candidate(
                            &state,
                            1,
                            policy.objective,
                            candidate,
                            candidate_index,
                            *rollout,
                        )
                    })
                    .count();
                ((successes * 100 + SEARCH_ROLLOUTS / 2) / SEARCH_ROLLOUTS) as u8
            })
            .collect::<Vec<_>>();
        let best = *all_estimates.iter().max().unwrap();
        assert_eq!(
            a.branch,
            if best >= 70 {
                "pursue-objective"
            } else {
                "withdraw"
            }
        );
    }

    #[test]
    fn damage_control_creates_temporary_coverage_without_erasing_damage() {
        let mut first = materialize_vessel(1, 1, "A", 72, 10).unwrap();
        first.power_hits = 2;
        let second = materialize_vessel(2, 2, "B", 72, 8).unwrap();
        let state = CombatState {
            combat_id: 111,
            revision: 1,
            round: 1,
            round_started_second: 0,
            range: RangeBand::Short,
            vessels: vec![first, second],
            missiles: vec![],
            boarding: vec![],
            complete: false,
        };
        let repair = JointOrder {
            vessel_id: 1,
            view_revision: 1,
            actions: vec![CrewAction::DamageControl],
            action_dms: vec![100],
            reactions: Vec::new(),
            reaction_dms: Vec::new(),
            automated: false,
        };
        let hold = JointOrder {
            vessel_id: 2,
            view_revision: 1,
            actions: vec![CrewAction::Hold],
            action_dms: vec![0],
            reactions: Vec::new(),
            reaction_dms: Vec::new(),
            automated: false,
        };
        let result = resolve_round(&state, &[repair, hold]).unwrap();
        let repaired = result
            .state
            .vessels
            .iter()
            .find(|vessel| vessel.vessel_id == 1)
            .unwrap();
        assert_eq!(repaired.power_hits, 2);
        assert_eq!(repaired.battlefield_repairs[4], 2);
        assert!(
            result
                .events
                .iter()
                .any(|event| matches!(event, CombatEvent::BattlefieldRepair { vessel_id: 1, .. }))
        );
    }

    #[test]
    fn damage_table_preserves_grouped_hits() {
        assert_eq!(damage_hit_groups(0), Vec::<u8>::new());
        assert_eq!(damage_hit_groups(8), vec![1, 1]);
        assert_eq!(damage_hit_groups(12), vec![2]);
        assert_eq!(damage_hit_groups(44), vec![3, 3]);
        assert!(!damage_hit_groups(50).is_empty());
    }
}
