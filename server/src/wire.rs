//! CT-RPC wire encoding owned by the server crate.

use std::io::Cursor;

use capnp::message::{Builder, ReaderOptions};
use capnp::{Error as CapnpError, serialize};
use thiserror::Error;

use crate::ct_rpc_capnp::{
    ErrorCode as SchemaErrorCode, Phase, ShipSubsystemKind as SchemaShipSubsystemKind, envelope,
    person_draft, player_creation, request,
};
use crate::i18n::DisplayFormatting;

pub const PROTOCOL_VERSION: u16 = 12;
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;
pub const COMMAND_ID_BYTES: usize = 16;
pub const MAX_NAME_BYTES: usize = 128;
pub const MAX_INITIAL_CREW: usize = 64;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("Cap'n Proto message error: {0}")]
    Capnp(#[from] CapnpError),
    #[error("unknown schema discriminant {0}")]
    NotInSchema(#[from] capnp::NotInSchema),
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u16),
    #[error("expected {0}")]
    Expected(&'static str),
    #[error("invalid UTF-8 text")]
    InvalidText,
    #[error("command ID must contain exactly {COMMAND_ID_BYTES} bytes")]
    InvalidCommandId,
    #[error("captain, ship, and crew names must contain 1..={MAX_NAME_BYTES} bytes")]
    InvalidName,
    #[error("initial crew cannot contain more than {MAX_INITIAL_CREW} names")]
    TooManyCrew,
    #[error("frame exceeds {MAX_FRAME_BYTES} bytes")]
    FrameTooLarge,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PlayerIdentity {
    pub bbs_id: u32,
    pub player_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientHello {
    pub identity: PlayerIdentity,
    pub client_name: String,
    pub language_tag: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseCode {
    Unspecified,
    UnsupportedVersion,
    MalformedHello,
    UnsupportedLanguage,
    AccessDenied,
    InvalidRequest,
    StaleSession,
    ServerStopping,
    SessionReplaced,
    InternalFailure,
}

fn schema_close_code(code: CloseCode) -> crate::ct_rpc_capnp::CloseCode {
    use crate::ct_rpc_capnp::CloseCode as Schema;
    match code {
        CloseCode::Unspecified => Schema::Unspecified,
        CloseCode::UnsupportedVersion => Schema::UnsupportedVersion,
        CloseCode::MalformedHello => Schema::MalformedHello,
        CloseCode::UnsupportedLanguage => Schema::UnsupportedLanguage,
        CloseCode::AccessDenied => Schema::AccessDenied,
        CloseCode::InvalidRequest => Schema::InvalidRequest,
        CloseCode::StaleSession => Schema::StaleSession,
        CloseCode::ServerStopping => Schema::ServerStopping,
        CloseCode::SessionReplaced => Schema::SessionReplaced,
        CloseCode::InternalFailure => Schema::InternalFailure,
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum SkillId {
    Admin,
    Advocate,
    Astrogation,
    Broker,
    Carouse,
    Communications,
    Computer,
    Electronics,
    EngineerJump,
    EngineerManeuver,
    EngineerPower,
    EngineerLifeSupport,
    Etiquette,
    GunCombat,
    GunnerTurrets,
    GunnerCapital,
    GunnerScreens,
    Investigate,
    JackOfAllTrades,
    Leadership,
    Mechanic,
    Medicine,
    Melee,
    Persuade,
    PilotSpacecraft,
    PilotSmallCraft,
    Recon,
    Stealth,
    Streetwise,
    TacticsMilitary,
    TacticsNaval,
    TradeCargomaster,
    VaccSuit,
    TradeProspector,
}

impl SkillId {
    pub const ALL: [Self; 34] = [
        Self::Admin,
        Self::Advocate,
        Self::Astrogation,
        Self::Broker,
        Self::Carouse,
        Self::Communications,
        Self::Computer,
        Self::Electronics,
        Self::EngineerJump,
        Self::EngineerManeuver,
        Self::EngineerPower,
        Self::EngineerLifeSupport,
        Self::Etiquette,
        Self::GunCombat,
        Self::GunnerTurrets,
        Self::GunnerCapital,
        Self::GunnerScreens,
        Self::Investigate,
        Self::JackOfAllTrades,
        Self::Leadership,
        Self::Mechanic,
        Self::Medicine,
        Self::Melee,
        Self::Persuade,
        Self::PilotSpacecraft,
        Self::PilotSmallCraft,
        Self::Recon,
        Self::Stealth,
        Self::Streetwise,
        Self::TacticsMilitary,
        Self::TacticsNaval,
        Self::TradeCargomaster,
        Self::VaccSuit,
        Self::TradeProspector,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Admin => "Admin",
            Self::Advocate => "Advocate",
            Self::Astrogation => "Astrogation",
            Self::Broker => "Broker",
            Self::Carouse => "Carouse",
            Self::Communications => "Communications",
            Self::Computer => "Computer",
            Self::Electronics => "Electronics",
            Self::EngineerJump => "Engineer (Jump Drive)",
            Self::EngineerManeuver => "Engineer (Maneuver Drive)",
            Self::EngineerPower => "Engineer (Power)",
            Self::EngineerLifeSupport => "Engineer (Life Support)",
            Self::Etiquette => "Etiquette",
            Self::GunCombat => "Gun Combat",
            Self::GunnerTurrets => "Gunner (Turrets)",
            Self::GunnerCapital => "Gunner (Capital Weapons)",
            Self::GunnerScreens => "Gunner (Screens)",
            Self::Investigate => "Investigate",
            Self::JackOfAllTrades => "Jack of All Trades",
            Self::Leadership => "Leadership",
            Self::Mechanic => "Mechanic",
            Self::Medicine => "Medicine",
            Self::Melee => "Melee",
            Self::Persuade => "Persuade",
            Self::PilotSpacecraft => "Pilot (Spacecraft)",
            Self::PilotSmallCraft => "Pilot (Small Craft)",
            Self::Recon => "Recon",
            Self::Stealth => "Stealth",
            Self::Streetwise => "Streetwise",
            Self::TacticsMilitary => "Tactics (Military)",
            Self::TacticsNaval => "Tactics (Naval)",
            Self::TradeCargomaster => "Trade (Cargomaster)",
            Self::VaccSuit => "Vacc Suit",
            Self::TradeProspector => "Trade (Prospector)",
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        Self::ALL.get(usize::from(value)).copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillPool {
    pub level3: u8,
    pub level2: u8,
    pub level1: u8,
    pub level0: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillRating {
    pub skill: SkillId,
    pub level: i8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillTraining {
    pub skill: SkillId,
    pub needed_weeks: u16,
    pub current_weeks: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Characteristics {
    pub strength: u8,
    pub dexterity: u8,
    pub endurance: u8,
    pub intelligence: u8,
    pub education: u8,
    pub charisma: u8,
}

impl Characteristics {
    pub fn values(self) -> [u8; 6] {
        [
            self.strength,
            self.dexterity,
            self.endurance,
            self.intelligence,
            self.education,
            self.charisma,
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonDraft {
    pub name: String,
    pub characteristics: Characteristics,
    pub skills: Vec<SkillRating>,
    pub training: SkillTraining,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialCrewDraft {
    pub slot_id: u16,
    pub name: String,
    pub training_skill: SkillId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerCreation {
    pub setup_revision: u64,
    pub starting_offer_id: u32,
    pub captain: PersonDraft,
    pub ship_name: String,
    pub crew: Vec<InitialCrewDraft>,
    pub refit_option_ids: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacteristicPointBuy {
    pub minimum: u8,
    pub maximum: u8,
    pub neutral: u8,
    pub budget: i16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptainCreationOptions {
    pub setup_revision: u64,
    pub characteristic_point_buy: CharacteristicPointBuy,
    pub skill_pool: SkillPool,
    pub default_captain: PersonDraft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Career {
    Trader,
    Privateer,
    Navy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OriginDossier {
    pub bbs_name: String,
    pub polity_name: String,
    pub home_system_name: String,
    pub home_world_name: String,
    pub trade_combat: u8,
    pub chaos_order: u8,
    pub league_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstitutionalAffiliation {
    pub polity_name: String,
    pub bbs_name: String,
    pub league_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingShipOfferSummary {
    pub offer_id: u32,
    pub career: Career,
    pub package_name: String,
    pub ship_catalog_id: u32,
    pub ship_name: String,
    pub role: String,
    pub rationale: String,
    pub displacement_tons: u32,
    pub jump_rating: u8,
    pub thrust_g: u8,
    pub cargo_millitons: u32,
    pub crew_count: u16,
    pub price_credits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingShipOffers {
    pub setup_revision: u64,
    pub origin: OriginDossier,
    pub offers: Vec<StartingShipOfferSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingShipOptions {
    pub setup_revision: u64,
    pub offer: StartingShipOfferSummary,
    pub description_paragraphs: Vec<String>,
    pub terms: StartingOfferTerms,
    pub refit_groups: Vec<StartingRefitGroup>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartingTitleKind {
    OwnedWithLien,
    SponsorOwned,
    InstitutionOwned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingOfferTerms {
    pub terms_revision: u64,
    pub title: StartingTitleKind,
    pub equity_credits: u64,
    pub principal_credits: u64,
    pub monthly_payment_credits: u64,
    pub liquid_reserve_credits: u64,
    pub restricted_reserve_credits: u64,
    pub monthly_compensation_credits: u64,
    pub refit_credit_limit: u64,
    pub refit_displacement_millitons: u64,
    pub authority: String,
    pub exit_terms: String,
    pub insurance: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingRefitOption {
    pub option_id: u32,
    pub name: String,
    pub description: String,
    pub displacement_delta_millitons: i64,
    pub price_delta_credits: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingRefitGroup {
    pub group_id: u16,
    pub name: String,
    pub required: bool,
    pub options: Vec<StartingRefitOption>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingCrewSlot {
    pub slot_id: u16,
    pub role: String,
    pub represented_positions: u16,
    pub required: bool,
    pub skill_pool: SkillPool,
    pub default_crew: PersonDraft,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingCrewPlan {
    pub setup_revision: u64,
    pub starting_offer_id: u32,
    pub slots: Vec<StartingCrewSlot>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrewRoleKind {
    Command,
    Pilot,
    Navigator,
    Engineer,
    SensorsOperator,
    ScreenOperator,
    TurretGunner,
    BayGunner,
    Gunner,
    Medic,
    Marine,
    FlightCrew,
    Steward,
    Other,
}

impl CrewRoleKind {
    pub fn from_slug(role: &str) -> Self {
        match role {
            "captain" | "command" => Self::Command,
            "pilot" => Self::Pilot,
            "navigator" | "astrogator" => Self::Navigator,
            "engineer" | "damage-control" => Self::Engineer,
            "sensors-operator" | "communications-operator" => Self::SensorsOperator,
            "screen-operator" => Self::ScreenOperator,
            "turret-gunner" => Self::TurretGunner,
            "bay-gunner" => Self::BayGunner,
            "gunner" => Self::Gunner,
            "medic" => Self::Medic,
            "marine" | "security" => Self::Marine,
            "flight-crew" => Self::FlightCrew,
            "steward" => Self::Steward,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrewManagementMember {
    pub person_id: u64,
    pub slot_id: u16,
    pub role: String,
    pub represented_positions: u16,
    pub captain: bool,
    pub person: PersonDraft,
    pub assigned_slot_ids: Vec<u16>,
    pub condition: PersonCondition,
    pub injury_points: u16,
    pub fatigue_points: u16,
    pub unfed_days: u16,
    pub available: bool,
    pub current_strength: u8,
    pub current_dexterity: u8,
    pub current_endurance: u8,
    pub service_kind: CrewServiceKind,
    pub monthly_salary_credits: u64,
    pub arrears_credits: u64,
    pub prize_share_basis_points: u16,
    pub morale: u8,
    pub loyalty: u8,
    pub risk_tolerance: u8,
    pub availability: CrewAvailability,
    pub available_second: u64,
    pub service_revision: u64,
    pub shore_location: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrewServiceKind {
    OwnerCaptain,
    Salaried,
    PrizeShare,
    Institutional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrewAvailability {
    Active,
    ShoreLeave,
    MedicalCare,
    Detached,
    AwaitingRecall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonnelActionKind {
    Dismiss,
    Transfer,
    ShoreLeave,
    Recall,
    FirstAid,
    Surgery,
    MedicalCare,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonCondition {
    Fit,
    Fatigued,
    Wounded,
    Incapacitated,
    Dead,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrewRole {
    pub slot_id: u16,
    pub role: String,
    pub represented_positions: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrewManagementSnapshot {
    pub ship_id: u64,
    pub ship_name: String,
    pub members: Vec<CrewManagementMember>,
    pub roles: Vec<CrewRole>,
    pub established_complement: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipSubsystemKind {
    Hull,
    Structure,
    Armor,
    Bridge,
    Computer,
    Sensors,
    JumpDrive,
    ManeuverDrive,
    PowerPlant,
    FuelSystem,
    LifeSupport,
    CargoHold,
    WeaponMount,
    Screen,
    Hangar,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipSubsystemStatus {
    pub subsystem_id: u16,
    pub kind: ShipSubsystemKind,
    pub label: String,
    pub maximum_hits: u16,
    pub sustained_hits: u16,
    pub battlefield_repair_hits: u16,
    pub effective_hits: u16,
    pub operational_effect: String,
    pub last_proper_repair_second: u64,
    pub installed_second: u64,
    pub last_refit_second: u64,
    pub calendar_age_months: u32,
    pub operating_seconds: u64,
    pub duty_cycles: u32,
    pub skimming_cycles: u32,
    pub neglect_damage_hits: u16,
    pub displacement_millitons: u64,
    pub replacement_price_credits: u64,
    pub installation_generation: u16,
    pub reconditioned: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipAmmunitionStatus {
    pub ammunition_id: String,
    pub remaining: u32,
    pub capacity: u32,
    pub pack_units: u32,
    pub price_per_pack_credits: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShipProvisionStatus {
    pub person_days_remaining: u64,
    pub capacity_person_days: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipStatusSnapshot {
    pub ship_revision: u64,
    pub ship_id: u64,
    pub ship_name: String,
    pub catalog_id: u32,
    pub catalog_revision: u64,
    pub system_id: u64,
    pub current_game_second: u64,
    pub displacement_millitons: u64,
    pub jump_rating: u8,
    pub thrust_g: u8,
    pub fuel_capacity_millitons: u64,
    pub current_fuel_millitons: u64,
    pub jump_fuel_millitons: u64,
    pub cargo_capacity_millitons: u64,
    pub monthly_maintenance_credits: u64,
    pub next_maintenance_second: u64,
    pub maintenance_paid_through_second: u64,
    /// Retired wire slot retained for older encoded outcomes and clients.
    pub maintenance_arrears_credits: u64,
    pub completed_maintenance_cycles: u32,
    pub consecutive_missed_maintenance: u16,
    pub commissioned_second: u64,
    pub transit_count: u32,
    pub warranty_expires_second: u64,
    pub warranty_transit_limit: u32,
    pub warranty_repairs: u16,
    pub last_refit_second: u64,
    pub completed_refits: u16,
    pub active_activity: Option<ShipActivityStatus>,
    pub unrefined_fuel_millitons: u64,
    pub warranty_voided: bool,
    pub monthly_life_support_credits: u64,
    pub recovery_status: String,
    pub ammunition: Vec<ShipAmmunitionStatus>,
    pub provisions: ShipProvisionStatus,
    pub manifested_symptoms: Vec<String>,
    pub subsystems: Vec<ShipSubsystemStatus>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DockedFuelServiceKind {
    Refined,
    Unrefined,
    GasGiant,
    WildernessWater,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FuelSourceBodyKind {
    NotApplicable,
    GasGiant,
    Planet,
    Moon,
    IcyBelt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FuelAccessKind {
    PortSale,
    RoutineWilderness,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockedFuelService {
    pub kind: DockedFuelServiceKind,
    pub label: String,
    pub source_body_id: Option<u32>,
    pub available: bool,
    pub unavailable_reason: String,
    pub price_per_ton_credits: u64,
    pub maximum_millitons: u64,
    pub service_seconds: u64,
    pub body_kind: FuelSourceBodyKind,
    pub access_kind: FuelAccessKind,
    pub can_refine: bool,
    pub round_trip_distance_micro_au: u64,
    pub round_trip_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockedRepairService {
    pub subsystem_id: u16,
    pub label: String,
    pub available: bool,
    pub unavailable_reason: String,
    pub cost_credits: u64,
    pub service_seconds: u64,
    pub replacement: bool,
    pub reconditioned: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockedServices {
    pub ship_revision: u64,
    pub current_game_second: u64,
    pub fuel: Vec<DockedFuelService>,
    pub ammunition: Vec<ShipAmmunitionStatus>,
    pub provisions: ShipProvisionStatus,
    pub provision_package_person_days: u64,
    pub provision_package_price_credits: u64,
    pub provisions_available: bool,
    pub ammunition_available: bool,
    pub repair: Vec<DockedRepairService>,
    pub refit_available: bool,
    pub refit_unavailable_reason: String,
    pub refit_cost_credits: u64,
    pub refit_service_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DockedServiceOrderKind {
    Fuel {
        kind: DockedFuelServiceKind,
        source_body_id: Option<u32>,
        quantity_millitons: u64,
    },
    Ammunition {
        ammunition_id: String,
        packs: u32,
    },
    Provisions {
        packages: u16,
    },
    ProperRepair {
        subsystem_id: u16,
    },
    Refit,
    Replacement {
        subsystem_id: u16,
        reconditioned: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockedServiceOrder {
    pub expected_ship_revision: u64,
    pub kind: DockedServiceOrderKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FuelPurchaseReceipt {
    pub kind: DockedFuelServiceKind,
    pub quantity_millitons: u64,
    pub current_fuel_millitons: u64,
    pub unrefined_fuel_millitons: u64,
    pub fuel_capacity_millitons: u64,
    pub cost_credits: u64,
    pub restricted_payment_credits: u64,
    pub liquid_payment_credits: u64,
    pub restricted_balance_credits: u64,
    pub liquid_balance_credits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionPurchaseReceipt {
    pub packages: u16,
    pub person_days_loaded: u64,
    pub person_days_remaining: u64,
    pub capacity_person_days: u64,
    pub cost_credits: u64,
    pub restricted_payment_credits: u64,
    pub liquid_payment_credits: u64,
    pub restricted_balance_credits: u64,
    pub liquid_balance_credits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DockedServiceReceiptDetail {
    Generic,
    FuelPurchase(FuelPurchaseReceipt),
    ProvisionPurchase(ProvisionPurchaseReceipt),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockedServiceReceipt {
    pub ship_status: ShipStatusSnapshot,
    pub detail: DockedServiceReceiptDetail,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipActivityStatus {
    pub activity_id: u64,
    pub kind: ShipActivityKind,
    pub started_second: u64,
    pub due_second: u64,
    pub cost_credits: u64,
    pub source_body_id: Option<u32>,
    pub refine_collected: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipActivityKind {
    Construction,
    Refit,
    Refurbishment { component_count: u16 },
    ProperRepair { subsystem_id: u16 },
    GasGiantSkim { quantity_millitons: u64 },
    WildernessWater { quantity_millitons: u64 },
    EscortDuty { opportunity_id: u64 },
    FieldRecovery { subsystem_id: u16 },
    FuelProcessing { quantity_millitons: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockedSnapshot {
    pub ship_id: u64,
    pub ship_name: String,
    pub system_id: u64,
    pub system_name: String,
    pub world_id: u64,
    pub world_name: String,
    pub facility_id: u64,
    pub facility_name: String,
    pub starport: String,
    pub tech_level: u8,
    pub population: u8,
    pub law_level: u8,
    pub arrived_second: u64,
    pub current_game_second: u64,
    pub credits: u64,
    pub restricted_credits: u64,
    pub debt_credits: u64,
    pub fuel_millitons: u64,
    pub fuel_capacity_millitons: u64,
    pub refined_fuel_price_per_ton: u64,
    pub cargo_used_millitons: u64,
    pub cargo_capacity_millitons: u64,
    pub unrefined_fuel_millitons: u64,
    pub unrefined_fuel_price_per_ton: u64,
    pub accrued_berth_fee_credits: u64,
    pub export_tariff_due_credits: u64,
    pub facility_revision: u64,
    pub personnel_available: bool,
    pub banking_available: bool,
    pub authority_available: bool,
    pub medical_level: u8,
    pub clearance_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownSystemSummary {
    pub system_id: u64,
    pub system_name: String,
    pub world_name: String,
    pub distance_milliparsecs: u64,
    pub within_jump_rating: bool,
    pub starport: String,
    pub population: u8,
    pub tech_level: u8,
    pub observed_second: u64,
    pub source: String,
    pub position: Coordinate3,
    pub remote_candidate: bool,
    pub knowledge_source: SystemKnowledgeSource,
    pub gas_giant_count: u8,
    pub affiliation: Option<InstitutionalAffiliation>,
    pub navigation_targets: Vec<InSystemNavigationTarget>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InSystemNavigationTargetKind {
    RockyBody,
    GasGiant,
    PlanetoidBelt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InSystemNavigationTarget {
    pub body_id: u32,
    pub name: String,
    pub kind: InSystemNavigationTargetKind,
    pub primary_world: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemKnowledgeSource {
    PublishedRecords,
    CarriedRecords,
    PrivateObservation,
    PublicDispatch,
    DirectDispatch,
    Withheld,
    SecretChart,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownDestinations {
    pub current_system_id: u64,
    pub jump_rating: u8,
    pub systems: Vec<KnownSystemSummary>,
    /// Catalogued planetoid belts in the ship's present system.
    pub belts: Vec<KnownBelt>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownBelt {
    pub system_id: u64,
    pub body_id: u32,
    pub name: String,
    pub icy: bool,
    pub carbonaceous_percent: u8,
    pub silicate_or_rock_percent: u8,
    pub metal_or_water_ice_percent: u8,
    pub hydrocarbon_percent: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CourseFuelSource {
    None,
    Carried,
    RefinedPort,
    FrontierSkimming,
    UnrefinedPort,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CourseWaypoint {
    pub system_id: u64,
    pub system_name: String,
    pub world_name: String,
    pub fuel_source: CourseFuelSource,
    pub next_leg_milliparsecs: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoursePlan {
    pub available: bool,
    pub elapsed_seconds: u64,
    pub fuel_cost_credits: u64,
    pub total_milliparsecs: u64,
    pub waypoints: Vec<CourseWaypoint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoursePlot {
    pub origin_system_id: u64,
    pub destination_system_id: u64,
    pub jump_rating: u8,
    pub fastest: CoursePlan,
    pub cheapest: CoursePlan,
    pub current_game_second: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PriceDistribution {
    pub minimum: u64,
    pub lower_quartile: u64,
    pub median: u64,
    pub upper_quartile: u64,
    pub maximum: u64,
}

impl PriceDistribution {
    pub const fn flat(price: u64) -> Self {
        Self {
            minimum: price,
            lower_quartile: price,
            median: price,
            upper_quartile: price,
            maximum: price,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketOffer {
    pub offer_id: u64,
    pub commodity_id: u16,
    pub commodity_name: String,
    pub base_price_per_ton: u64,
    pub purchase_price_per_ton: u64,
    pub sale_price_per_ton: u64,
    pub available_millitons: u64,
    pub legality: CommodityLegality,
    pub price_distribution: PriceDistribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommodityLegality {
    Legal,
    Restricted,
    Prohibited,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CargoTitle {
    PlayerOwned,
    Freight,
    Contract,
    UniqueObject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CargoAcquisitionKind {
    Purchased,
    Extracted,
    Captured,
    Entrusted,
    Unique,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoLot {
    pub cargo_lot_id: u64,
    pub commodity_id: u16,
    pub commodity_name: String,
    pub quantity_millitons: u64,
    pub purchase_price_per_ton: u64,
    pub origin_system_id: u64,
    pub acquired_second: u64,
    pub title: CargoTitle,
    pub task_id: u64,
    pub unique_object_id: u64,
    pub condition_percent: u8,
    pub destination_system_id: u64,
    /// Nonzero only for material recovered from a celestial body.
    pub source_body_id: u32,
    /// Nonzero only for material traced to a persistent resource lode.
    pub source_lode_id: u64,
    pub acquisition_kind: CargoAcquisitionKind,
    pub acquisition_market_id: u64,
    pub export_tariff_paid: bool,
    pub valuation_basis_per_ton: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CargoSaleQuote {
    pub cargo_lot_id: u64,
    pub price_per_ton: u64,
    pub price_distribution: PriceDistribution,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketSnapshot {
    pub market_revision: u64,
    pub system_id: u64,
    pub world_name: String,
    pub generated_day: u64,
    pub credits: u64,
    pub cargo_used_millitons: u64,
    pub cargo_capacity_millitons: u64,
    pub offers: Vec<MarketOffer>,
    pub cargo: Vec<CargoLot>,
    pub trade_codes: Vec<String>,
    pub tariff_basis_points: u16,
    pub import_tariff_basis_points: u16,
    pub export_tariff_basis_points: u16,
    pub local_task_offers: Vec<TaskOffer>,
    pub work_assignments: Vec<WorkAssignment>,
    pub leads: Vec<MarketLead>,
    pub events: Vec<MarketEvent>,
    pub cargo_sale_quotes: Vec<CargoSaleQuote>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketLeadSide {
    Supplier,
    Buyer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketLeadState {
    Available,
    Reserved,
    Performed,
    Expired,
    Cancelled,
    Negotiating,
    Quoted,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketLead {
    pub lead_id: u64,
    pub revision: u64,
    pub side: MarketLeadSide,
    pub state: MarketLeadState,
    pub system_id: u64,
    pub commodity_id: u16,
    pub commodity_name: String,
    pub quantity_millitons: u64,
    pub price_per_ton: u64,
    pub discovered_second: u64,
    pub expires_second: u64,
    pub reservation_expires_second: u64,
    pub escrow_credits: u64,
    pub source: String,
    pub confidence_percent: u8,
    pub counterparty_id: u64,
    pub cargo_lot_id: u64,
    pub penalty_until_second: u64,
    pub illegal: bool,
    pub loader_fee_credits: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketEventKind {
    Shortage,
    Surplus,
    Disruption,
    Recovery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketEvent {
    pub event_id: u64,
    pub kind: MarketEventKind,
    pub commodity_id: u16,
    pub commodity_name: String,
    pub start_second: u64,
    pub expires_second: u64,
    pub stock_multiplier_basis_points: u16,
    pub purchase_tier_delta: i8,
    pub sale_tier_delta: i8,
    pub supplier_offer_multiplier_basis_points: u16,
    pub buyer_offer_multiplier_basis_points: u16,
    pub carriage_offer_multiplier_basis_points: u16,
    pub headline: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketSearchKind {
    Supplier,
    Buyer,
    Freight,
    Passengers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketSearchMethod {
    Physical,
    Online,
    BlackMarket,
    HiredBroker,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkState {
    Scheduled,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkAssignment {
    pub assignment_id: u64,
    pub kind: MarketSearchKind,
    pub method: MarketSearchMethod,
    pub person_id: u64,
    pub commodity_id: u16,
    pub destination_system_id: u64,
    pub started_second: u64,
    pub due_second: u64,
    pub state: WorkState,
    pub result_text: String,
    pub lead_id: u64,
    pub maximum_quantity_millitons: u64,
    pub cargo_lot_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskKind {
    Freight,
    Passenger,
    PurchaseOrder,
    ForwardSale,
    SupplyCommitment,
    Charter,
    Courier,
    DiscoveryBounty,
    CombatBounty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskState {
    ClaimPending,
    Accepted,
    Sourcing,
    Loading,
    InTransit,
    AwaitingSettlement,
    Completed,
    Expired,
    Cancelled,
    Defaulted,
    Disputed,
    LossDocumented,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskOffer {
    pub offer_id: u64,
    pub revision: u64,
    pub kind: TaskKind,
    pub title: String,
    pub origin_system_id: u64,
    pub destination_system_id: u64,
    pub commodity_id: u16,
    pub quantity_millitons: u64,
    pub passenger_count: u16,
    pub payment_credits: u64,
    pub collateral_credits: u64,
    pub expires_second: u64,
    pub delivery_deadline_second: u64,
    pub legal: bool,
    pub partial_delivery_allowed: bool,
    pub failure_penalty_credits: u64,
    pub recurrence_seconds: u64,
    pub performance_count: u16,
    pub passenger_class: PassengerClass,
    pub late_deduction_per_day_credits: u64,
    pub non_delivery_liability_credits: u64,
    pub passenger_grace_seconds: u64,
    pub declared_value_credits: u64,
    pub unavailable_reasons: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassengerClass {
    None,
    High,
    Middle,
    Steerage,
    Low,
    Charter,
    Courier,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskActionKind {
    Cancel,
    ReturnCustody,
    DefaultTask,
    FileDispute,
    WithdrawClaim,
    FileLossClaim,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRecord {
    pub task_id: u64,
    pub offer: TaskOffer,
    pub state: TaskState,
    pub accepted_second: u64,
    pub delivered_quantity_millitons: u64,
    pub reserved_cargo_millitons: u64,
    pub reserved_passenger_count: u16,
    pub reserved_credits: u64,
    pub status_text: String,
    pub performances_completed: u16,
    pub revision: u64,
    pub claim_message_id: u64,
    pub result_message_id: u64,
    pub known_result: bool,
    pub loaded_second: u64,
    pub settled_second: u64,
    pub insurance_claim_id: u64,
    pub dispute_message_id: u64,
    pub dispute_effect: i16,
    pub adjudication_message_id: u64,
    pub performing_ship_id: u64,
    pub piracy_encounter_id: u64,
    pub piracy_incident_second: u64,
    pub piracy_contact_id: u64,
    pub piracy_threat: EncounterThreat,
    pub piracy_posture: EncounterPosture,
    pub piracy_quantity_millitons: u64,
    pub loss_claim_deadline_second: u64,
    pub loss_claim_effect: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CarriageDeclaration {
    pub plan_revision: u64,
    pub destination_system_id: u64,
    pub freight_capacity_millitons: u64,
    pub high_berths: u16,
    pub middle_berths: u16,
    pub steerage_berths: u16,
    pub low_berths: u16,
    pub accept_electronic_mail: bool,
}

impl Default for CarriageDeclaration {
    fn default() -> Self {
        Self {
            plan_revision: 0,
            destination_system_id: 0,
            freight_capacity_millitons: 0,
            high_berths: 0,
            middle_berths: 0,
            steerage_berths: 0,
            low_berths: 0,
            accept_electronic_mail: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskLedger {
    pub current_second: u64,
    pub available_credits: u64,
    pub reserved_credits: u64,
    pub reserved_cargo_millitons: u64,
    pub reserved_passenger_count: u16,
    pub tasks: Vec<TaskRecord>,
    pub local_offers: Vec<TaskOffer>,
    pub carriage: CarriageDeclaration,
    pub route_assessments: Vec<TaskRouteAssessment>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskRouteAssessment {
    pub offer_id: u64,
    pub pickup_available: bool,
    pub pickup_arrival_second: u64,
    pub delivery_available: bool,
    pub delivery_arrival_second: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ShipTitleKind {
    OwnedWithLien,
    SponsorOwned,
    InstitutionOwned,
    OwnedClear,
    PrizeCustody,
    StolenRegistry,
    CourtImpound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ManagedShipOrderKind {
    Hold,
    FollowActive,
    Travel,
    Dock,
    Sell,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedShipSummary {
    pub ship_id: u64,
    pub name: String,
    pub class_name: String,
    pub catalog_id: u32,
    pub system_id: u64,
    pub system_name: String,
    pub location: String,
    pub title: ShipTitleKind,
    pub active: bool,
    pub commanding_person_id: u64,
    pub commanding_person_name: String,
    pub standing_order: ManagedShipOrderKind,
    pub can_assume_command: bool,
    pub fuel_millitons: u64,
    pub fuel_capacity_millitons: u64,
    pub cargo_used_millitons: u64,
    pub cargo_capacity_millitons: u64,
    pub provision_person_days: u64,
    pub provision_capacity_person_days: u64,
    pub cargo: Vec<CargoLot>,
    pub ammunition: Vec<ShipAmmunitionStatus>,
    pub online_controlled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FleetSnapshot {
    pub revision: u64,
    pub active_ship_id: u64,
    pub ships: Vec<ManagedShipSummary>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum StoreTransferKind {
    Cargo,
    Fuel,
    Ammunition,
    Provisions,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinanceSnapshot {
    pub title: ShipTitleKind,
    pub liquid_credits: u64,
    pub restricted_credits: u64,
    pub reserved_credits: u64,
    pub original_hull_price_credits: u64,
    pub principal_credits: u64,
    pub monthly_payment_credits: u64,
    pub monthly_insurance_escrow_credits: u64,
    pub next_payment_due_second: u64,
    pub grace_expires_second: u64,
    pub paid_through_second: u64,
    pub in_default: bool,
    pub impound_order_known_locally: bool,
    pub credit_status: String,
    pub destination_assistance_active: bool,
    pub destination_assistance_expires_second: u64,
    pub current_second: u64,
    pub pending_income: Vec<PendingIncome>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingIncomeStage {
    FilingToOffice,
    RemittanceToCaptain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IncomeEstimateKind {
    Projected,
    Scheduled,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingIncome {
    pub task_id: u64,
    pub payment_credits: u64,
    pub reserved_release_credits: u64,
    pub stage: PendingIncomeStage,
    pub estimated_resolution_second: u64,
    pub estimate_kind: IncomeEstimateKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountTransactionClass {
    All,
    Opening,
    Income,
    Expense,
    Transfer,
    Hold,
    Financing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountKind {
    Liquid,
    RestrictedOperating,
    Reserved,
    SecuredPrincipal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountChangeKind {
    Increase,
    Decrease,
    BalanceForward,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountPosting {
    pub account: AccountKind,
    pub change: AccountChangeKind,
    pub amount_credits: u64,
    pub balance_after_credits: u64,
    pub ship_id: u64,
    pub ship_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountLedgerEntry {
    pub entry_id: u64,
    pub occurred_second: u64,
    pub class: AccountTransactionClass,
    pub summary: String,
    pub subject_ship_id: u64,
    pub subject_ship_name: String,
    pub postings: Vec<AccountPosting>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountLedgerVessel {
    pub ship_id: u64,
    pub ship_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountLedgerRequest {
    pub before_entry_id: u64,
    pub limit: u16,
    pub class: AccountTransactionClass,
    pub ship_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountLedgerPage {
    pub current_second: u64,
    pub entries: Vec<AccountLedgerEntry>,
    pub next_before_entry_id: u64,
    pub has_more: bool,
    pub vessels: Vec<AccountLedgerVessel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketObservation {
    pub system_id: u64,
    pub system_name: String,
    pub commodity_id: u16,
    pub commodity_name: String,
    pub observed_second: u64,
    pub acquired_second: u64,
    pub source: String,
    pub confidence_percent: u8,
    pub minimum_price_per_ton: u64,
    pub maximum_price_per_ton: u64,
    pub minimum_available_millitons: u64,
    pub maximum_available_millitons: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketKnowledge {
    pub current_second: u64,
    pub observations: Vec<MarketObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipMarketOffer {
    pub offer_id: u64,
    pub catalog_id: u32,
    pub class_name: String,
    pub price_credits: u64,
    pub original_price_credits: u64,
    pub used: bool,
    pub age_months: u32,
    pub visible_condition_percent: u8,
    pub cargo_capacity_millitons: u64,
    pub jump_rating: u8,
    pub minimum_crew: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipMarket {
    pub generated_day: u64,
    pub current_ship_trade_in_credits: u64,
    pub outstanding_lien_credits: u64,
    pub offers: Vec<ShipMarketOffer>,
    pub commissionable_designs: Vec<ShipCommissionDesign>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipCommissionDesign {
    pub catalog_id: u32,
    pub class_name: String,
    pub tech_level: u8,
    pub price_credits: u64,
    pub deposit_credits: u64,
    pub construction_seconds: u64,
    pub displacement_millitons: u64,
    pub jump_rating: u8,
    pub fuel_capacity_millitons: u64,
    pub jump_fuel_millitons: u64,
    pub cargo_capacity_millitons: u64,
    pub minimum_crew: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrewCandidate {
    pub candidate_id: u64,
    pub role: String,
    pub name: String,
    pub primary_skill: SkillId,
    pub skill_level: i8,
    pub monthly_salary_credits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrewMarket {
    pub generated_day: u64,
    pub candidates: Vec<CrewCandidate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TravelStatus {
    pub ship_id: u64,
    pub ship_name: String,
    pub current_system_id: u64,
    pub current_system_name: String,
    pub destination_system_id: u64,
    pub destination_system_name: String,
    pub stage: TravelStage,
    pub current_game_second: u64,
    pub due_second: u64,
    pub current_fuel_millitons: u64,
    pub fuel_capacity_millitons: u64,
    pub jump_fuel_millitons: u64,
    pub plan_id: u64,
    pub plan_revision: u64,
    pub leg_index: u16,
    pub origin: FlightLocus,
    pub destination: FlightLocus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Coordinate3 {
    pub coreward_bits: u64,
    pub spinward_bits: u64,
    pub north_bits: u64,
}

impl Coordinate3 {
    pub fn from_parsecs(value: [f64; 3]) -> Self {
        Self {
            coreward_bits: value[0].to_bits(),
            spinward_bits: value[1].to_bits(),
            north_bits: value[2].to_bits(),
        }
    }

    pub fn parsecs(self) -> [f64; 3] {
        [
            f64::from_bits(self.coreward_bits),
            f64::from_bits(self.spinward_bits),
            f64::from_bits(self.north_bits),
        ]
    }

    pub fn is_finite(self) -> bool {
        self.parsecs().into_iter().all(f64::is_finite)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlightLocus {
    Port {
        system_id: u64,
        world_id: u64,
        facility_id: u64,
    },
    JumpLocus {
        system_id: u64,
    },
    ArrivalLocus {
        system_id: u64,
        remote: bool,
    },
    Body {
        system_id: u64,
        body_id: u32,
    },
    DeepSpace {
        position: Coordinate3,
    },
}

impl FlightLocus {
    pub fn system_id(self) -> u64 {
        match self {
            Self::Port { system_id, .. }
            | Self::JumpLocus { system_id }
            | Self::ArrivalLocus { system_id, .. }
            | Self::Body { system_id, .. } => system_id,
            Self::DeepSpace { .. } => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TravelStage {
    Docked,
    DepartingForJump,
    JumpSpace,
    ApproachingStarport,
    Refit,
    ProperRepair,
    GasGiantSkim,
    WildernessWater,
    Holding,
    Encounter,
    BeltProspecting,
    BeltSurvey,
    BeltMining,
    BeltRefining,
    BeltRecovery,
    BeltEgress,
    FuelProcessing,
    Maneuvering,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaypointAuthority {
    Hold,
    Through,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FuelOperation {
    GasGiant,
    WildernessWater,
    BuyRefined,
    BuyUnrefined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JumpNavigationMethod {
    Onboard,
    CommercialTape,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlightPlanAction {
    Hold,
    Jump {
        destination_system_id: u64,
        navigation: JumpNavigationMethod,
        proceed_on_known_bad_plot: bool,
        remote_arrival: bool,
        departure_locus_arrival: bool,
    },
    JumpCoordinates {
        destination: Coordinate3,
        navigation: JumpNavigationMethod,
        proceed_on_known_bad_plot: bool,
    },
    Dock {
        world_id: u64,
        facility_id: u64,
    },
    Fuel {
        operation: FuelOperation,
        quantity_millitons: u64,
        refine_collected: bool,
    },
    BeltCycle {
        body_id: u32,
    },
    RefineFuel {
        quantity_millitons: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightPlanStep {
    pub locus: FlightLocus,
    pub authority: WaypointAuthority,
    pub action: FlightPlanAction,
    pub terminal: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterPosture {
    Fight,
    Flee,
    Comply,
    Surrender,
    Board,
    Pursue,
    ContinueCourse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterFallback {
    Surrender,
    Abandon,
    JettisonCargo,
    BreakOff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterFightMode {
    Never,
    Always,
    EstimatedAtLeast,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncounterStandingOrder {
    pub kind: EncounterKind,
    pub ordinary_posture: EncounterPosture,
    pub fight_mode: EncounterFightMode,
    pub minimum_outlook_percent: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterPolicy {
    pub hostile_posture: EncounterPosture,
    pub hostile_fallbacks: Vec<EncounterFallback>,
    pub comply_with_inspection: bool,
    pub report_distress: bool,
    pub assist_distress: bool,
    pub standing_orders: Vec<EncounterStandingOrder>,
}

impl Default for EncounterPolicy {
    fn default() -> Self {
        Self {
            hostile_posture: EncounterPosture::Flee,
            hostile_fallbacks: vec![EncounterFallback::Surrender],
            comply_with_inspection: true,
            report_distress: true,
            assist_distress: false,
            standing_orders: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterPolicyDefaultSnapshot {
    pub ship_id: u64,
    pub revision: u64,
    pub policy: EncounterPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetEncounterPolicyDefaultRequest {
    pub expected_revision: u64,
    pub policy: EncounterPolicy,
    pub acknowledge_nonhostile_fight: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightPlanProposal {
    pub expected_plan_revision: u64,
    pub steps: Vec<FlightPlanStep>,
    pub policy: EncounterPolicy,
    pub preserve_active_step: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitFlightPlanRequest {
    pub proposal: FlightPlanProposal,
    pub preview_hash: Vec<u8>,
    pub acknowledge_warnings: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightPlanWarning {
    pub code: String,
    pub message: String,
    pub step_indices: Vec<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FuelOperationTiming {
    pub step_index: u16,
    pub round_trip_seconds: u64,
    pub collection_seconds: u64,
    pub processing_seconds: u64,
    pub failed_processing_seconds: u64,
    pub normal_total_seconds: u64,
    pub failed_total_seconds: u64,
    pub output_refined: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightPlanPreview {
    pub proposal: FlightPlanProposal,
    pub preview_hash: Vec<u8>,
    pub elapsed_seconds: u64,
    pub fuel_millitons: u64,
    pub warnings: Vec<FlightPlanWarning>,
    pub carriage_offers: Vec<TaskOffer>,
    pub carriage_revenue_credits: u64,
    pub carriage_broker_fees_credits: u64,
    pub fuel_timings: Vec<FuelOperationTiming>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlightPlanState {
    Inactive,
    Active,
    Held,
    Checkpoint,
    Encounter,
    Completed,
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightPlanSnapshot {
    pub plan_id: u64,
    pub revision: u64,
    pub current_step: u16,
    pub state: FlightPlanState,
    pub steps: Vec<FlightPlanStep>,
    pub policy: EncounterPolicy,
    pub suspension_reason: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointKind {
    PortDeparture,
    InhabitedWorld,
    GasGiant,
    JumpArrival,
    JumpDeparture,
    DeepSpace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointSnapshot {
    pub checkpoint_id: u64,
    pub plan_id: u64,
    pub plan_revision: u64,
    pub step_index: u16,
    pub locus: FlightLocus,
    pub kind: CheckpointKind,
    pub ready_second: u64,
    pub acknowledged: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterKind {
    RoutineTraffic,
    TrafficControl,
    Inspection,
    Distress,
    Derelict,
    Hazard,
    Hostile,
    Military,
    DepartingContact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterState {
    AwaitingPosture,
    Resolving,
    Resolved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterContact {
    pub contact_id: u64,
    pub ship_name: String,
    pub class_name: String,
    pub declared_class_name: String,
    pub transponder: String,
    pub role: String,
    pub range: String,
    pub confidence_percent: u8,
    pub resolution: EncounterResolution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterResolution {
    RadioOnly,
    TransponderOnly,
    Approximate,
    Identified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterAuthority {
    None,
    Pirate,
    TrafficControl,
    Customs,
    Naval,
    Warrant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum EncounterThreat {
    #[default]
    Unknown,
    Favorable,
    Comparable,
    Dangerous,
    Overwhelming,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct EncounterDemand {
    pub present: bool,
    pub player_owned_percent: u8,
    pub player_owned_millitons: u64,
    pub entrusted_millitons: u64,
    pub unique_object_count: u16,
    pub text: String,
    pub entrusted_liability_credits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterSnapshot {
    pub encounter_id: u64,
    pub revision: u64,
    pub kind: EncounterKind,
    pub state: EncounterState,
    pub started_second: u64,
    pub next_turn_second: u64,
    pub turn: u16,
    pub contact: EncounterContact,
    pub summary: String,
    pub authority: EncounterAuthority,
    pub threat: EncounterThreat,
    pub demand: EncounterDemand,
    pub available_postures: Vec<EncounterPosture>,
    pub available_fallbacks: Vec<EncounterFallback>,
    pub response_deadline_second: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolveEncounterRequest {
    pub encounter_id: u64,
    pub expected_revision: u64,
    pub posture: EncounterPosture,
    pub fallbacks: Vec<EncounterFallback>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterResult {
    pub encounter_id: u64,
    pub resolved: bool,
    pub terminal: bool,
    pub outcome: String,
    pub turns: u16,
    pub cargo_lost_millitons: u64,
    pub fuel_lost_millitons: u64,
    pub damage_hits: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandLossKind {
    Destroyed,
    Captured,
    Surrendered,
    Abandoned,
    Bankruptcy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptainFate {
    Survived,
    Dead,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalReport {
    pub encounter_id: u64,
    pub revision: u64,
    pub acknowledged: bool,
    pub started_second: u64,
    pub resolved_second: u64,
    pub system_id: u64,
    pub system_name: String,
    pub location: String,
    pub contact: EncounterContact,
    pub authority: EncounterAuthority,
    pub threat: EncounterThreat,
    pub standing_orders_used: bool,
    pub posture: Option<EncounterPosture>,
    pub fallbacks: Vec<EncounterFallback>,
    pub automated_combat_used: bool,
    pub outcome: String,
    pub ship_name: String,
    pub loss_kind: CommandLossKind,
    pub owned_cargo_lost_millitons: u64,
    pub entrusted_cargo_lost_millitons: u64,
    pub unique_objects_lost: u16,
    pub fuel_lost_millitons: u64,
    pub passengers_affected: u16,
    pub damage_hits: u16,
    pub captain_name: String,
    pub captain_fate: CaptainFate,
    pub other_crew_total: u16,
    pub other_crew_dead: u16,
    pub other_crew_injured: u16,
    pub other_crew_surviving: u16,
    pub recovery_ready_second: u64,
    pub successor_required: bool,
    pub incident_log: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatActionKind {
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
    Attack,
    Board,
    PrepareJump,
    LaunchEscapeCraft,
    OfferSurrender,
    AcceptSurrender,
    InspectContact,
    Pursuit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatReaction {
    Dodge,
    PointDefense,
    FireSand,
    TriggerNuclearDamper,
    TriggerMesonScreen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombatAction {
    pub kind: CombatActionKind,
    pub mount_id: u16,
    pub target_vessel_id: u64,
    pub actor_person_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombatReactionOrder {
    pub kind: CombatReaction,
    pub actor_person_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatOrderSet {
    pub combat_id: u64,
    pub view_revision: u64,
    pub actions: Vec<CombatAction>,
    pub reactions: Vec<CombatReactionOrder>,
    pub use_tactical_controller: bool,
    pub speed_adjustment: i16,
    pub speed_actor_person_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatAutomationPolicy {
    pub expected_revision: u64,
    pub minimum_victory_percent: u8,
    pub objective: crate::combat::Objective,
    pub permit_surrender: bool,
    pub permit_abandon_ship: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatWeaponMount {
    pub mount_id: u16,
    pub label: String,
    pub weapons: Vec<String>,
    pub damage_hits: u8,
    pub ammunition_remaining: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatParticipant {
    pub vessel_id: u64,
    pub side: u16,
    pub name: String,
    pub class_name: String,
    pub initiative: i16,
    pub thrust: u8,
    pub hull_remaining: u16,
    pub structure_remaining: u16,
    pub armor_remaining: u16,
    pub disposition: crate::combat::VesselDisposition,
    pub weapons: Vec<CombatWeaponMount>,
    pub commanded: bool,
    pub player_owned: bool,
    pub online_controlled: bool,
    pub speed: i16,
    pub pursuit_target_vessel_id: u64,
    pub pursuit_attack_bonus: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatActor {
    pub person_id: u64,
    pub name: String,
    pub station: String,
    pub available: bool,
    pub action_budget: u8,
    pub role_kind: CrewRoleKind,
    pub captain: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatSnapshot {
    pub combat_id: u64,
    pub revision: u64,
    pub round: u16,
    pub round_started_second: u64,
    pub order_due_second: u64,
    pub order_window_real_milliseconds: u64,
    pub range: crate::combat::RangeBand,
    pub participants: Vec<CombatParticipant>,
    pub default_order: CombatOrderSet,
    pub policy: CombatAutomationPolicy,
    pub player_order_submitted: bool,
    pub complete: bool,
    pub log: Vec<String>,
    pub actors: Vec<CombatActor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatCareerSnapshot {
    pub state: crate::careers::CareerState,
    pub rank: String,
    pub monthly_salary_credits: u64,
    pub local_enforcement_summary: String,
    pub system_contacts: Vec<crate::traffic::TrafficContact>,
    pub local_contacts: Vec<crate::traffic::TrafficContact>,
    pub interception_watch: Option<InterceptionWatchStatus>,
    pub known_warrants: Vec<KnownWarrant>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WarrantAssociationKind {
    Historical,
    ReportedAboard,
    ConfirmedAboard,
    WantedVessel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BountyCustodyState {
    AtLarge,
    HeldAboard,
    Settled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownWarrant {
    pub warrant_id: u64,
    pub subject_person_id: u64,
    pub subject_name: String,
    pub subject_role: String,
    pub accusation: String,
    pub bounty_credits: u64,
    pub severity: u8,
    pub evidence_percent: u8,
    pub issuing_polity_id: u64,
    pub origin_system_id: u64,
    pub filed_second: u64,
    pub associated_ship_id: u64,
    pub associated_ship_name: String,
    pub associated_transponder: String,
    pub last_known_system_id: u64,
    pub association: WarrantAssociationKind,
    pub custody: BountyCustodyState,
    pub generated_target: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterceptionWatchFilterKind {
    NamedVessel,
    CraftClass,
    AllCraft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterceptionPurpose {
    ArmedAttack,
    BoardingInspection,
    Arrest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterceptionWatchStatus {
    pub started_second: u64,
    pub target_contact_id: u64,
    pub target_catalog_id: u32,
    pub target_ship_name: String,
    pub filter: InterceptionWatchFilterKind,
    pub locus: FlightLocus,
    pub purpose: InterceptionPurpose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterceptionWatchRequest {
    Cancel {
        expected_revision: u64,
    },
    AllCraft {
        expected_revision: u64,
        purpose: InterceptionPurpose,
    },
    CraftClass {
        expected_revision: u64,
        catalog_id: u32,
        purpose: InterceptionPurpose,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrizeSettlementMethod {
    FileClaim,
    TakeAdvance,
    Fence,
    CourtSale,
    KeepPrize,
    LaunderRegistry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageClass {
    AgencyNews,
    PublicService,
    ContractOffer,
    TrafficNotice,
    Private,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageImportance {
    Routine,
    Notable,
    Important,
    Headline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MessageClassification {
    Unreviewed,
    Ignored,
    ReviewLater,
    Actioned,
    Archived,
}

impl MessageClassification {
    pub const ALL: [Self; 5] = [
        Self::Unreviewed,
        Self::Ignored,
        Self::ReviewLater,
        Self::Actioned,
        Self::Archived,
    ];

    pub fn from_u8(value: u8) -> Option<Self> {
        Self::ALL.get(usize::from(value)).copied()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageItem {
    pub message_id: u64,
    pub origin_system_id: u64,
    pub origin_system_name: String,
    pub created_second: u64,
    pub available_second: u64,
    pub expires_second: u64,
    pub class: MessageClass,
    pub importance: MessageImportance,
    pub subject: String,
    pub body: String,
    pub offer_id: Option<u64>,
    pub offer_revision: u64,
    pub offer_available: bool,
    pub classification: MessageClassification,
    pub previously_seen: bool,
    pub expired: bool,
    pub action_kind: MessageActionKind,
    pub action_reference_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageActionKind {
    None,
    ClaimOffer,
    ReviewTask,
    ReviewOperations,
    ReviewFinance,
    ReviewMapping,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrivalPacket {
    pub new_arrival: bool,
    pub system_id: u64,
    pub system_name: String,
    pub arrival_second: u64,
    pub mailbag_id: Option<u64>,
    pub mail_delivered: u64,
    pub mail_forwarded: u64,
    pub mail_expired: u64,
    pub stipend_credits: u64,
    pub items: Vec<MessageItem>,
    pub mapping_status: SystemMappingStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageManagement {
    pub items: Vec<MessageItem>,
    pub filters: Vec<MessageFilter>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageFilter {
    pub class: MessageClass,
    pub minimum_importance: MessageImportance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateRecipientKind {
    System,
    Captain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateMessageRequest {
    pub recipient_kind: PrivateRecipientKind,
    pub destination_system_id: u64,
    pub recipient: PlayerIdentity,
    pub encryption_key_id: u64,
    pub ttl_weeks: u16,
    pub subject: String,
    pub body: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioTransmissionKind {
    PlayerBroadcast,
    InspectionOrder,
    BoardingOrder,
    PirateDemand,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioInboxEntry {
    pub reception_id: u64,
    pub transmission_id: u64,
    pub receiving_ship_id: u64,
    pub sender_ship_id: u64,
    pub sender_ship_name: String,
    pub sender_transponder: String,
    pub sender: PlayerIdentity,
    pub emitted_second: u64,
    pub received_second: u64,
    pub expires_second: u64,
    pub kind: RadioTransmissionKind,
    pub actionable: bool,
    pub action_reference_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemRadioSnapshot {
    pub ship_id: u64,
    pub system_id: u64,
    pub current_second: u64,
    pub can_transmit: bool,
    pub unavailable_reason: String,
    pub entries: Vec<RadioInboxEntry>,
    pub mutes: Vec<PlayerIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioContent {
    pub reception_id: u64,
    pub transmission_id: u64,
    pub body: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InsuranceKind {
    DestinationAssistance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SystemMappingState {
    KnownPublic,
    Unresolved,
    PublicDispatched,
    DirectDispatched,
    Withheld,
    Secret,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemMappingChoice {
    PublicNotification,
    DirectEarth,
    Withhold,
    WithholdSecret,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemMappingStatus {
    pub system_id: u64,
    pub state: SystemMappingState,
    pub dispatch_message_id: Option<u64>,
    pub changed_second: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalDamageCause {
    JumpTransition,
    FuelProcessing,
    MaintenanceNeglect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationalDamageReport {
    pub present: bool,
    pub report_id: u64,
    pub occurred_second: u64,
    pub ship_id: u64,
    pub ship_name: String,
    pub cause: OperationalDamageCause,
    pub origin_system_id: u64,
    pub origin_system_name: String,
    pub destination_system_id: u64,
    pub destination_system_name: String,
    pub inaccurate_extra_days: u8,
    pub misjump: bool,
    pub subsystem_id: u16,
    pub subsystem_kind: ShipSubsystemKind,
    pub subsystem_label: String,
    pub damage_hits: u16,
    pub sustained_hits: u16,
    pub maximum_hits: u16,
    pub operational_effect: String,
}

impl OperationalDamageReport {
    pub fn none() -> Self {
        Self {
            present: false,
            report_id: 0,
            occurred_second: 0,
            ship_id: 0,
            ship_name: String::new(),
            cause: OperationalDamageCause::JumpTransition,
            origin_system_id: 0,
            origin_system_name: String::new(),
            destination_system_id: 0,
            destination_system_name: String::new(),
            inaccurate_extra_days: 0,
            misjump: false,
            subsystem_id: 0,
            subsystem_kind: ShipSubsystemKind::Other,
            subsystem_label: String::new(),
            damage_hits: 0,
            sustained_hits: 0,
            maximum_hits: 0,
            operational_effect: String::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Ping,
    CreatePlayer(PlayerCreation),
    GetCaptainCreationOptions,
    GetStartingShipOffers,
    GetStartingShipOptions {
        setup_revision: u64,
        starting_offer_id: u32,
    },
    GetStartingCrewPlan {
        setup_revision: u64,
        starting_offer_id: u32,
    },
    GetCrewManagement,
    SetCrewTrainingTarget {
        person_id: u64,
        skill: SkillId,
    },
    SetCrewAssignments {
        person_id: u64,
        slot_ids: Vec<u16>,
    },
    ApplyPersonnelAction {
        person_id: u64,
        expected_service_revision: u64,
        action: PersonnelActionKind,
        target_ship_id: u64,
        duration_days: u16,
    },
    GetShipStatus,
    GetDockedSnapshot,
    GetKnownDestinations,
    GetMarket,
    BuyCargo {
        market_revision: u64,
        offer_id: u64,
        quantity_millitons: u64,
    },
    SellCargo {
        market_revision: u64,
        cargo_lot_id: u64,
        quantity_millitons: u64,
        buyer_lead_id: u64,
    },
    GetTravelStatus,
    BeginVoyage {
        destination_system_id: u64,
    },
    PlotCourse {
        origin_system_id: u64,
        destination_system_id: u64,
        use_current_fuel: bool,
    },
    SuggestTaskCourse,
    OpenArrivalPacket,
    GetMessageManagement,
    SetMessageClassification {
        message_id: u64,
        classification: MessageClassification,
    },
    SetMessageFilter {
        class: MessageClass,
        minimum_importance: MessageImportance,
    },
    SetSystemMappingDisclosure {
        system_id: u64,
        choice: SystemMappingChoice,
    },
    GetFlightPlan,
    GetEncounterPolicyDefault,
    SetEncounterPolicyDefault(SetEncounterPolicyDefaultRequest),
    PreviewFlightPlan(FlightPlanProposal),
    CommitFlightPlan(CommitFlightPlanRequest),
    AcknowledgeCheckpoint {
        checkpoint_id: u64,
    },
    GetEncounter,
    GetTerminalReport,
    AcknowledgeTerminalReport {
        encounter_id: u64,
        expected_revision: u64,
    },
    GetOperationalDamageReport,
    AcknowledgeOperationalDamageReport {
        report_id: u64,
    },
    ResolveEncounter(ResolveEncounterRequest),
    GetCombat,
    SubmitCombatOrder(CombatOrderSet),
    SetCombatAutomationPolicy(CombatAutomationPolicy),
    GetCombatCareer,
    AcceptCareerOpportunity {
        opportunity_id: u64,
        expected_revision: u64,
    },
    EngageTrafficContact {
        contact_id: u64,
        expected_career_revision: u64,
        purpose: InterceptionPurpose,
    },
    SetInterceptionWatch(InterceptionWatchRequest),
    SetPirateCruise(crate::careers::PirateCruise),
    SettlePrize {
        prize_id: u64,
        expected_career_revision: u64,
        method: PrizeSettlementMethod,
    },
    SettleWarrant {
        warrant_id: u64,
        expected_career_revision: u64,
    },
    SetCombatCareerMode {
        mode: crate::careers::CombatCareerMode,
        expected_revision: u64,
    },
    RecoverCommand {
        successor_name: String,
    },
    DeclareBankruptcy {
        successor_name: String,
    },
    AbandonPlayer {
        confirmation: String,
    },
    GetTaskLedger,
    AcceptTaskOffer {
        offer_id: u64,
        expected_revision: u64,
    },
    SetCarriageDeclaration(CarriageDeclaration),
    GetFinance,
    GetAccountLedger(AccountLedgerRequest),
    CureFinanceDefault,
    GetMarketKnowledge,
    GetShipMarket,
    PurchaseShip {
        offer_id: u64,
        trade_in_current_ship: bool,
    },
    CommissionShip {
        catalog_id: u32,
    },
    GetCrewMarket,
    HireCrew {
        candidate_id: u64,
    },
    BeginMarketSearch {
        kind: MarketSearchKind,
        method: MarketSearchMethod,
        person_id: u64,
        commodity_id: u16,
        destination_system_id: u64,
        maximum_quantity_millitons: u64,
        cargo_lot_id: u64,
    },
    BeginMarketNegotiation {
        lead_id: u64,
        expected_revision: u64,
        person_id: u64,
    },
    AcceptMarketQuote {
        lead_id: u64,
        expected_revision: u64,
    },
    RejectMarketQuote {
        lead_id: u64,
        expected_revision: u64,
    },
    CancelWorkAssignment {
        assignment_id: u64,
    },
    GetDockedServices,
    CommitDockedService(DockedServiceOrder),
    ReserveMarketLead {
        lead_id: u64,
        expected_revision: u64,
        quantity_millitons: u64,
    },
    ReleaseMarketReservation {
        lead_id: u64,
        expected_revision: u64,
    },
    ApplyTaskAction {
        task_id: u64,
        expected_revision: u64,
        action: TaskActionKind,
        explanation: String,
    },
    SendPrivateMessage(PrivateMessageRequest),
    PurchaseInsurance {
        kind: InsuranceKind,
        enabled: bool,
    },
    MisappropriateRestrictedCredits {
        amount: u64,
    },
    GetFleet,
    SetActiveShip {
        expected_revision: u64,
        ship_id: u64,
    },
    AssignShipCaptain {
        expected_revision: u64,
        ship_id: u64,
        person_id: u64,
    },
    TransferShipStores {
        expected_revision: u64,
        from_ship_id: u64,
        to_ship_id: u64,
        kind: StoreTransferKind,
        cargo_lot_id: u64,
        item_id: String,
        quantity: u64,
    },
    GetSystemRadio,
    TransmitSystemRadio {
        body: String,
    },
    PeekRadioReception {
        reception_id: u64,
    },
    AcknowledgeRadioReception {
        reception_id: u64,
    },
    SetRadioMute {
        sender: PlayerIdentity,
        muted: bool,
    },
    GetBrowserAlertStatus,
    CreateBrowserAlertEnrollment,
    RevokeAllBrowserAlerts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandPersistence {
    /// Served by a non-authoritative subsystem and never admitted to the game
    /// queue or journal.
    Operational,
    /// A read of authoritative state. It remains ordered and journaled, but
    /// its potentially large response is not retained forever for replay.
    Observation,
    /// A request that can change authoritative state. Its result is retained
    /// by command ID so retries are exactly once across reconnects.
    Transaction,
}

impl Command {
    pub fn persistence(&self) -> CommandPersistence {
        match self {
            Self::Ping
            | Self::GetCaptainCreationOptions
            | Self::GetStartingShipOffers
            | Self::GetStartingShipOptions { .. }
            | Self::GetStartingCrewPlan { .. }
            | Self::GetCrewManagement
            | Self::GetShipStatus
            | Self::GetDockedSnapshot
            | Self::GetKnownDestinations
            | Self::GetMarket
            | Self::GetTravelStatus
            | Self::PlotCourse { .. }
            | Self::SuggestTaskCourse
            | Self::GetMessageManagement
            | Self::GetFlightPlan
            | Self::GetEncounterPolicyDefault
            | Self::PreviewFlightPlan(_)
            | Self::GetEncounter
            | Self::GetTerminalReport
            | Self::GetOperationalDamageReport
            | Self::GetCombat
            | Self::GetCombatCareer => CommandPersistence::Observation,
            Self::GetTaskLedger
            | Self::GetFinance
            | Self::GetAccountLedger(_)
            | Self::GetMarketKnowledge
            | Self::GetShipMarket
            | Self::GetCrewMarket => CommandPersistence::Observation,
            Self::GetFleet => CommandPersistence::Observation,
            Self::GetDockedServices | Self::GetSystemRadio | Self::PeekRadioReception { .. } => {
                CommandPersistence::Observation
            }
            Self::CreatePlayer(_)
            | Self::SetCrewTrainingTarget { .. }
            | Self::SetCrewAssignments { .. }
            | Self::ApplyPersonnelAction { .. }
            | Self::BuyCargo { .. }
            | Self::SellCargo { .. }
            | Self::BeginVoyage { .. }
            | Self::OpenArrivalPacket
            | Self::SetMessageClassification { .. }
            | Self::SetMessageFilter { .. }
            | Self::SetSystemMappingDisclosure { .. }
            | Self::CommitFlightPlan(_)
            | Self::SetEncounterPolicyDefault(_)
            | Self::AcknowledgeCheckpoint { .. }
            | Self::ResolveEncounter(_) => CommandPersistence::Transaction,
            Self::AcknowledgeTerminalReport { .. }
            | Self::AcknowledgeOperationalDamageReport { .. } => CommandPersistence::Transaction,
            Self::SubmitCombatOrder(_) | Self::SetCombatAutomationPolicy(_) => {
                CommandPersistence::Transaction
            }
            Self::AcceptCareerOpportunity { .. }
            | Self::EngageTrafficContact { .. }
            | Self::SetInterceptionWatch(_)
            | Self::SetPirateCruise(_)
            | Self::SettlePrize { .. }
            | Self::SettleWarrant { .. }
            | Self::SetCombatCareerMode { .. }
            | Self::RecoverCommand { .. }
            | Self::DeclareBankruptcy { .. }
            | Self::AbandonPlayer { .. } => CommandPersistence::Transaction,
            Self::AcceptTaskOffer { .. }
            | Self::SetCarriageDeclaration(_)
            | Self::PurchaseShip { .. }
            | Self::CommissionShip { .. }
            | Self::HireCrew { .. }
            | Self::BeginMarketSearch { .. }
            | Self::BeginMarketNegotiation { .. }
            | Self::AcceptMarketQuote { .. }
            | Self::RejectMarketQuote { .. }
            | Self::CancelWorkAssignment { .. } => CommandPersistence::Transaction,
            Self::CommitDockedService(_) => CommandPersistence::Transaction,
            Self::ReserveMarketLead { .. }
            | Self::ReleaseMarketReservation { .. }
            | Self::ApplyTaskAction { .. }
            | Self::SendPrivateMessage(_)
            | Self::PurchaseInsurance { .. }
            | Self::MisappropriateRestrictedCredits { .. }
            | Self::CureFinanceDefault => CommandPersistence::Transaction,
            Self::SetActiveShip { .. }
            | Self::AssignShipCaptain { .. }
            | Self::TransferShipStores { .. }
            | Self::TransmitSystemRadio { .. }
            | Self::AcknowledgeRadioReception { .. }
            | Self::SetRadioMute { .. } => CommandPersistence::Transaction,
            Self::GetBrowserAlertStatus
            | Self::CreateBrowserAlertEnrollment
            | Self::RevokeAllBrowserAlerts => CommandPersistence::Operational,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRequest {
    pub request_id: u64,
    pub session_epoch: u64,
    pub command_id: [u8; COMMAND_ID_BYTES],
    pub command: Command,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    InvalidCommand,
    StaleSession,
    MalformedMessage,
    UnsupportedVersion,
    InternalFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutcomeKind {
    Pong,
    PlayerCreated(PlayerCreation),
    CaptainCreationOptions(CaptainCreationOptions),
    StartingShipOffers(StartingShipOffers),
    StartingShipOptions(StartingShipOptions),
    StartingCrewPlan(StartingCrewPlan),
    CrewManagement(CrewManagementSnapshot),
    ShipStatus(ShipStatusSnapshot),
    DockedSnapshot(DockedSnapshot),
    KnownDestinations(KnownDestinations),
    Market(MarketSnapshot),
    TravelStatus(TravelStatus),
    CoursePlot(CoursePlot),
    ArrivalPacket(ArrivalPacket),
    MessageManagement(MessageManagement),
    SystemMappingStatus(SystemMappingStatus),
    FlightPlan(FlightPlanSnapshot),
    FlightPlanPreview(FlightPlanPreview),
    EncounterPolicyDefault(EncounterPolicyDefaultSnapshot),
    Checkpoint(CheckpointSnapshot),
    Encounter(EncounterSnapshot),
    EncounterResult(EncounterResult),
    TerminalReport(TerminalReport),
    Combat(CombatSnapshot),
    CombatCareer(CombatCareerSnapshot),
    TaskLedger(TaskLedger),
    Finance(FinanceSnapshot),
    AccountLedger(AccountLedgerPage),
    MarketKnowledge(MarketKnowledge),
    ShipMarket(ShipMarket),
    CrewMarket(CrewMarket),
    DockedServices(DockedServices),
    DockedServiceReceipt(DockedServiceReceipt),
    Fleet(FleetSnapshot),
    SystemRadio(SystemRadioSnapshot),
    RadioContent(RadioContent),
    BrowserAlertStatus(crate::web_push::BrowserAlertStatus),
    BrowserAlertEnrollment(crate::web_push::BrowserAlertEnrollment),
    OperationalDamageReport(OperationalDamageReport),
    Error { code: ErrorCode, message: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerPhase {
    NewUser,
    Docked,
    Interplanetary,
    Jump,
    Encounter,
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outcome {
    pub command_id: [u8; COMMAND_ID_BYTES],
    pub committed_sequence: u64,
    pub revision: u64,
    pub replayed: bool,
    pub phase: PlayerPhase,
    pub kind: OutcomeKind,
}

fn message_reader(
    bytes: &[u8],
) -> Result<capnp::message::Reader<capnp::serialize::OwnedSegments>, WireError> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(WireError::FrameTooLarge);
    }
    let mut options = ReaderOptions::new();
    options
        .traversal_limit_in_words(Some(MAX_FRAME_BYTES / 8))
        .nesting_limit(32);
    Ok(serialize::read_message(&mut Cursor::new(bytes), options)?)
}

fn check_version(version: u16) -> Result<(), WireError> {
    if version == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(WireError::UnsupportedVersion(version))
    }
}

fn decode_flight_locus(
    reader: crate::ct_rpc_capnp::flight_locus::Reader<'_>,
) -> Result<FlightLocus, WireError> {
    let system_id = reader.get_system_id();
    Ok(match reader.which()? {
        crate::ct_rpc_capnp::flight_locus::Port(port) => {
            let port = port?;
            FlightLocus::Port {
                system_id,
                world_id: port.get_world_id(),
                facility_id: port.get_facility_id(),
            }
        }
        crate::ct_rpc_capnp::flight_locus::JumpLocus(()) => match reader.get_jump_role()? {
            crate::ct_rpc_capnp::JumpLocusRole::Departure => FlightLocus::JumpLocus { system_id },
            crate::ct_rpc_capnp::JumpLocusRole::Arrival => FlightLocus::ArrivalLocus {
                system_id,
                remote: reader.get_remote_arrival(),
            },
        },
        crate::ct_rpc_capnp::flight_locus::BodyId(body_id) => {
            FlightLocus::Body { system_id, body_id }
        }
        crate::ct_rpc_capnp::flight_locus::DeepSpace(position) => {
            let position = position?;
            FlightLocus::DeepSpace {
                position: Coordinate3::from_parsecs([
                    position.get_coreward(),
                    position.get_spinward(),
                    position.get_north(),
                ]),
            }
        }
    })
}

fn decode_encounter_posture(value: crate::ct_rpc_capnp::EncounterPosture) -> EncounterPosture {
    match value {
        crate::ct_rpc_capnp::EncounterPosture::Fight => EncounterPosture::Fight,
        crate::ct_rpc_capnp::EncounterPosture::Flee => EncounterPosture::Flee,
        crate::ct_rpc_capnp::EncounterPosture::Comply => EncounterPosture::Comply,
        crate::ct_rpc_capnp::EncounterPosture::Surrender => EncounterPosture::Surrender,
        crate::ct_rpc_capnp::EncounterPosture::Board => EncounterPosture::Board,
        crate::ct_rpc_capnp::EncounterPosture::Pursue => EncounterPosture::Pursue,
        crate::ct_rpc_capnp::EncounterPosture::ContinueCourse => EncounterPosture::ContinueCourse,
    }
}

fn decode_encounter_fallback(value: crate::ct_rpc_capnp::EncounterFallback) -> EncounterFallback {
    match value {
        crate::ct_rpc_capnp::EncounterFallback::Surrender => EncounterFallback::Surrender,
        crate::ct_rpc_capnp::EncounterFallback::Abandon => EncounterFallback::Abandon,
        crate::ct_rpc_capnp::EncounterFallback::JettisonCargo => EncounterFallback::JettisonCargo,
        crate::ct_rpc_capnp::EncounterFallback::BreakOff => EncounterFallback::BreakOff,
    }
}

fn decode_encounter_kind(value: crate::ct_rpc_capnp::EncounterKind) -> EncounterKind {
    match value {
        crate::ct_rpc_capnp::EncounterKind::RoutineTraffic => EncounterKind::RoutineTraffic,
        crate::ct_rpc_capnp::EncounterKind::TrafficControl => EncounterKind::TrafficControl,
        crate::ct_rpc_capnp::EncounterKind::Inspection => EncounterKind::Inspection,
        crate::ct_rpc_capnp::EncounterKind::Distress => EncounterKind::Distress,
        crate::ct_rpc_capnp::EncounterKind::Derelict => EncounterKind::Derelict,
        crate::ct_rpc_capnp::EncounterKind::Hazard => EncounterKind::Hazard,
        crate::ct_rpc_capnp::EncounterKind::Hostile => EncounterKind::Hostile,
        crate::ct_rpc_capnp::EncounterKind::Military => EncounterKind::Military,
        crate::ct_rpc_capnp::EncounterKind::DepartingContact => EncounterKind::DepartingContact,
    }
}

fn decode_encounter_policy(
    reader: crate::ct_rpc_capnp::encounter_policy::Reader<'_>,
) -> Result<EncounterPolicy, WireError> {
    let standing_orders = reader
        .get_standing_orders()?
        .iter()
        .map(|order| {
            Ok(EncounterStandingOrder {
                kind: decode_encounter_kind(order.get_kind()?),
                ordinary_posture: decode_encounter_posture(order.get_ordinary_posture()?),
                fight_mode: match order.get_fight_mode()? {
                    crate::ct_rpc_capnp::EncounterFightMode::Never => EncounterFightMode::Never,
                    crate::ct_rpc_capnp::EncounterFightMode::Always => EncounterFightMode::Always,
                    crate::ct_rpc_capnp::EncounterFightMode::EstimatedAtLeast => {
                        EncounterFightMode::EstimatedAtLeast
                    }
                },
                minimum_outlook_percent: order.get_minimum_outlook_percent(),
            })
        })
        .collect::<Result<Vec<_>, WireError>>()?;
    Ok(EncounterPolicy {
        hostile_posture: decode_encounter_posture(reader.get_hostile_posture()?),
        hostile_fallbacks: reader
            .get_hostile_fallbacks()?
            .iter()
            .map(|value| {
                value
                    .map(decode_encounter_fallback)
                    .map_err(WireError::from)
            })
            .collect::<Result<Vec<_>, _>>()?,
        comply_with_inspection: reader.get_comply_with_inspection(),
        report_distress: reader.get_report_distress(),
        assist_distress: reader.get_assist_distress(),
        standing_orders,
    })
}

fn decode_flight_plan_proposal(
    reader: crate::ct_rpc_capnp::flight_plan_proposal::Reader<'_>,
) -> Result<FlightPlanProposal, WireError> {
    let mut steps = reader
        .get_steps()?
        .iter()
        .map(|step| {
            let locus = decode_flight_locus(step.get_locus()?)?;
            let (authority, terminal) = match step.get_authority()? {
                crate::ct_rpc_capnp::WaypointAuthority::Hold => {
                    (WaypointAuthority::Hold, step.get_terminal())
                }
                crate::ct_rpc_capnp::WaypointAuthority::Terminal => (WaypointAuthority::Hold, true),
                crate::ct_rpc_capnp::WaypointAuthority::Through => {
                    (WaypointAuthority::Through, step.get_terminal())
                }
            };
            let action = match step.get_action()?.which()? {
                crate::ct_rpc_capnp::flight_plan_action::Hold(()) => FlightPlanAction::Hold,
                crate::ct_rpc_capnp::flight_plan_action::Jump(jump) => {
                    let jump = jump?;
                    FlightPlanAction::Jump {
                        destination_system_id: jump.get_destination_system_id(),
                        navigation: match jump.get_navigation()? {
                            crate::ct_rpc_capnp::JumpNavigationMethod::Onboard => {
                                JumpNavigationMethod::Onboard
                            }
                            crate::ct_rpc_capnp::JumpNavigationMethod::CommercialTape => {
                                JumpNavigationMethod::CommercialTape
                            }
                        },
                        proceed_on_known_bad_plot: jump.get_proceed_on_known_bad_plot(),
                        remote_arrival: jump.get_remote_arrival(),
                        departure_locus_arrival: jump.get_departure_locus_arrival(),
                    }
                }
                crate::ct_rpc_capnp::flight_plan_action::Dock(port) => {
                    let port = port?;
                    FlightPlanAction::Dock {
                        world_id: port.get_world_id(),
                        facility_id: port.get_facility_id(),
                    }
                }
                crate::ct_rpc_capnp::flight_plan_action::Fuel(fuel) => {
                    let fuel = fuel?;
                    FlightPlanAction::Fuel {
                        operation: match fuel.get_operation()? {
                            crate::ct_rpc_capnp::FuelOperation::GasGiant => FuelOperation::GasGiant,
                            crate::ct_rpc_capnp::FuelOperation::WildernessWater => {
                                FuelOperation::WildernessWater
                            }
                            crate::ct_rpc_capnp::FuelOperation::BuyRefined => {
                                FuelOperation::BuyRefined
                            }
                            crate::ct_rpc_capnp::FuelOperation::BuyUnrefined => {
                                FuelOperation::BuyUnrefined
                            }
                        },
                        quantity_millitons: fuel.get_quantity_millitons(),
                        refine_collected: fuel.get_refine_collected(),
                    }
                }
                crate::ct_rpc_capnp::flight_plan_action::JumpCoordinates(jump) => {
                    let jump = jump?;
                    let position = jump.get_destination()?;
                    FlightPlanAction::JumpCoordinates {
                        destination: Coordinate3::from_parsecs([
                            position.get_coreward(),
                            position.get_spinward(),
                            position.get_north(),
                        ]),
                        navigation: match jump.get_navigation()? {
                            crate::ct_rpc_capnp::JumpNavigationMethod::Onboard => {
                                JumpNavigationMethod::Onboard
                            }
                            crate::ct_rpc_capnp::JumpNavigationMethod::CommercialTape => {
                                JumpNavigationMethod::CommercialTape
                            }
                        },
                        proceed_on_known_bad_plot: jump.get_proceed_on_known_bad_plot(),
                    }
                }
                crate::ct_rpc_capnp::flight_plan_action::BeltCycle(body_id) => {
                    FlightPlanAction::BeltCycle { body_id }
                }
                crate::ct_rpc_capnp::flight_plan_action::RefineFuel(quantity_millitons) => {
                    FlightPlanAction::RefineFuel { quantity_millitons }
                }
            };
            Ok(FlightPlanStep {
                locus,
                authority,
                action,
                terminal,
            })
        })
        .collect::<Result<Vec<_>, WireError>>()?;
    if !steps.is_empty() && !steps.iter().any(|step| step.terminal) {
        // Pre-terminal-bit clients used either the legacy Terminal enum or
        // simply ended the list. Preserve the latter shape at the wire edge.
        steps.last_mut().expect("non-empty steps").terminal = true;
    }
    Ok(FlightPlanProposal {
        expected_plan_revision: reader.get_expected_plan_revision(),
        steps,
        policy: decode_encounter_policy(reader.get_policy()?)?,
        preserve_active_step: reader.get_preserve_active_step(),
    })
}

pub fn decode_client_hello(bytes: &[u8]) -> Result<ClientHello, WireError> {
    let (version, hello) = decode_client_hello_with_version(bytes)?;
    check_version(version)?;
    Ok(hello)
}

pub fn decode_protocol_version(bytes: &[u8]) -> Result<u16, WireError> {
    let message = message_reader(bytes)?;
    Ok(message
        .get_root::<envelope::Reader>()?
        .get_protocol_version())
}

pub fn decode_client_hello_with_version(bytes: &[u8]) -> Result<(u16, ClientHello), WireError> {
    let message = message_reader(bytes)?;
    let envelope = message.get_root::<envelope::Reader>()?;
    let version = envelope.get_protocol_version();
    let envelope::ClientHello(hello) = envelope.which()? else {
        return Err(WireError::Expected("clientHello envelope"));
    };
    let hello = hello?;
    let identity = hello.get_identity()?;
    let bbs_id = identity.get_bbs_id();
    let player_id = identity.get_player_id();
    let client_name = hello
        .get_client_name()?
        .to_str()
        .map_err(|_| WireError::InvalidText)?
        .to_owned();
    let language_tag = hello
        .get_language_tag()?
        .to_str()
        .map_err(|_| WireError::InvalidText)?
        .to_owned();
    Ok((
        version,
        ClientHello {
            identity: PlayerIdentity { bbs_id, player_id },
            client_name,
            language_tag,
        },
    ))
}

fn decode_market_search_kind(value: crate::ct_rpc_capnp::MarketSearchKind) -> MarketSearchKind {
    match value {
        crate::ct_rpc_capnp::MarketSearchKind::Supplier => MarketSearchKind::Supplier,
        crate::ct_rpc_capnp::MarketSearchKind::Buyer => MarketSearchKind::Buyer,
        crate::ct_rpc_capnp::MarketSearchKind::Freight => MarketSearchKind::Freight,
        crate::ct_rpc_capnp::MarketSearchKind::Passengers => MarketSearchKind::Passengers,
    }
}

fn decode_market_search_method(
    value: crate::ct_rpc_capnp::MarketSearchMethod,
) -> MarketSearchMethod {
    match value {
        crate::ct_rpc_capnp::MarketSearchMethod::Physical => MarketSearchMethod::Physical,
        crate::ct_rpc_capnp::MarketSearchMethod::Online => MarketSearchMethod::Online,
        crate::ct_rpc_capnp::MarketSearchMethod::BlackMarket => MarketSearchMethod::BlackMarket,
        crate::ct_rpc_capnp::MarketSearchMethod::HiredBroker => MarketSearchMethod::HiredBroker,
    }
}

pub fn decode_request(bytes: &[u8]) -> Result<CommandRequest, WireError> {
    let message = message_reader(bytes)?;
    let envelope = message.get_root::<envelope::Reader>()?;
    check_version(envelope.get_protocol_version())?;
    let request_id = envelope.get_request_id();
    let session_epoch = envelope.get_session_epoch();
    let envelope::Request(request) = envelope.which()? else {
        return Err(WireError::Expected("request envelope"));
    };
    let request = request?;
    let command_id_data = request.get_command_id()?;
    let command_id: [u8; COMMAND_ID_BYTES] = command_id_data
        .try_into()
        .map_err(|_| WireError::InvalidCommandId)?;
    let command = match request.which()? {
        request::Ping(()) => Command::Ping,
        request::CreatePlayer(creation) => {
            Command::CreatePlayer(decode_player_creation(creation?)?)
        }
        request::GetCaptainCreationOptions(()) => Command::GetCaptainCreationOptions,
        request::GetStartingShipOffers(()) => Command::GetStartingShipOffers,
        request::GetStartingShipOptions(query) => {
            let query = query?;
            Command::GetStartingShipOptions {
                setup_revision: query.get_setup_revision(),
                starting_offer_id: query.get_starting_offer_id(),
            }
        }
        request::GetStartingCrewPlan(query) => {
            let query = query?;
            Command::GetStartingCrewPlan {
                setup_revision: query.get_setup_revision(),
                starting_offer_id: query.get_starting_offer_id(),
            }
        }
        request::GetCrewManagement(()) => Command::GetCrewManagement,
        request::SetCrewTrainingTarget(change) => {
            let change = change?;
            Command::SetCrewTrainingTarget {
                person_id: change.get_person_id(),
                skill: decode_skill(change.get_skill()?)?,
            }
        }
        request::SetCrewAssignments(change) => {
            let change = change?;
            Command::SetCrewAssignments {
                person_id: change.get_person_id(),
                slot_ids: change.get_slot_ids()?.iter().collect(),
            }
        }
        request::ApplyPersonnelAction(change) => {
            let change = change?;
            let action = match change.get_action()? {
                crate::ct_rpc_capnp::PersonnelActionKind::Dismiss => PersonnelActionKind::Dismiss,
                crate::ct_rpc_capnp::PersonnelActionKind::Transfer => PersonnelActionKind::Transfer,
                crate::ct_rpc_capnp::PersonnelActionKind::ShoreLeave => {
                    PersonnelActionKind::ShoreLeave
                }
                crate::ct_rpc_capnp::PersonnelActionKind::Recall => PersonnelActionKind::Recall,
                crate::ct_rpc_capnp::PersonnelActionKind::FirstAid => PersonnelActionKind::FirstAid,
                crate::ct_rpc_capnp::PersonnelActionKind::Surgery => PersonnelActionKind::Surgery,
                crate::ct_rpc_capnp::PersonnelActionKind::MedicalCare => {
                    PersonnelActionKind::MedicalCare
                }
            };
            Command::ApplyPersonnelAction {
                person_id: change.get_person_id(),
                expected_service_revision: change.get_expected_service_revision(),
                action,
                target_ship_id: change.get_target_ship_id(),
                duration_days: change.get_duration_days(),
            }
        }
        request::GetShipStatus(()) => Command::GetShipStatus,
        request::GetDockedSnapshot(()) => Command::GetDockedSnapshot,
        request::GetKnownDestinations(()) => Command::GetKnownDestinations,
        request::GetMarket(()) => Command::GetMarket,
        request::BuyCargo(change) => {
            let change = change?;
            Command::BuyCargo {
                market_revision: change.get_market_revision(),
                offer_id: change.get_offer_id(),
                quantity_millitons: change.get_quantity_millitons(),
            }
        }
        request::SellCargo(change) => {
            let change = change?;
            Command::SellCargo {
                market_revision: change.get_market_revision(),
                cargo_lot_id: change.get_cargo_lot_id(),
                quantity_millitons: change.get_quantity_millitons(),
                buyer_lead_id: change.get_buyer_lead_id(),
            }
        }
        request::GetTravelStatus(()) => Command::GetTravelStatus,
        request::BeginVoyage(change) => Command::BeginVoyage {
            destination_system_id: change?.get_destination_system_id(),
        },
        request::PlotCourse(query) => {
            let query = query?;
            Command::PlotCourse {
                origin_system_id: query.get_origin_system_id(),
                destination_system_id: query.get_destination_system_id(),
                use_current_fuel: query.get_use_current_fuel(),
            }
        }
        request::OpenArrivalPacket(()) => Command::OpenArrivalPacket,
        request::GetMessageManagement(()) => Command::GetMessageManagement,
        request::SetMessageClassification(change) => {
            let change = change?;
            Command::SetMessageClassification {
                message_id: change.get_message_id(),
                classification: decode_message_classification(change.get_classification()?),
            }
        }
        request::SetMessageFilter(change) => {
            let change = change?;
            Command::SetMessageFilter {
                class: decode_message_class(change.get_class()?),
                minimum_importance: decode_message_importance(change.get_minimum_importance()?),
            }
        }
        request::SetSystemMappingDisclosure(change) => {
            let change = change?;
            Command::SetSystemMappingDisclosure {
                system_id: change.get_system_id(),
                choice: match change.get_choice()? {
                    crate::ct_rpc_capnp::SystemMappingChoice::PublicNotification => {
                        SystemMappingChoice::PublicNotification
                    }
                    crate::ct_rpc_capnp::SystemMappingChoice::DirectEarth => {
                        SystemMappingChoice::DirectEarth
                    }
                    crate::ct_rpc_capnp::SystemMappingChoice::Withhold => {
                        SystemMappingChoice::Withhold
                    }
                    crate::ct_rpc_capnp::SystemMappingChoice::WithholdSecret => {
                        SystemMappingChoice::WithholdSecret
                    }
                },
            }
        }
        request::GetFlightPlan(()) => Command::GetFlightPlan,
        request::PreviewFlightPlan(proposal) => {
            Command::PreviewFlightPlan(decode_flight_plan_proposal(proposal?)?)
        }
        request::CommitFlightPlan(commit) => {
            let commit = commit?;
            Command::CommitFlightPlan(CommitFlightPlanRequest {
                proposal: decode_flight_plan_proposal(commit.get_proposal()?)?,
                preview_hash: commit.get_preview_hash()?.to_vec(),
                acknowledge_warnings: commit.get_acknowledge_warnings(),
            })
        }
        request::AcknowledgeCheckpoint(ack) => Command::AcknowledgeCheckpoint {
            checkpoint_id: ack?.get_checkpoint_id(),
        },
        request::GetEncounter(()) => Command::GetEncounter,
        request::GetTerminalReport(()) => Command::GetTerminalReport,
        request::AcknowledgeTerminalReport(value) => {
            let value = value?;
            Command::AcknowledgeTerminalReport {
                encounter_id: value.get_encounter_id(),
                expected_revision: value.get_expected_revision(),
            }
        }
        request::ResolveEncounter(resolve) => {
            let resolve = resolve?;
            Command::ResolveEncounter(ResolveEncounterRequest {
                encounter_id: resolve.get_encounter_id(),
                expected_revision: resolve.get_expected_revision(),
                posture: decode_encounter_posture(resolve.get_posture()?),
                fallbacks: resolve
                    .get_fallbacks()?
                    .iter()
                    .map(|value| {
                        value
                            .map(decode_encounter_fallback)
                            .map_err(WireError::from)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
        request::GetTaskLedger(()) => Command::GetTaskLedger,
        request::SuggestTaskCourse(()) => Command::SuggestTaskCourse,
        request::AcceptTaskOffer(accept) => {
            let accept = accept?;
            Command::AcceptTaskOffer {
                offer_id: accept.get_offer_id(),
                expected_revision: accept.get_expected_revision(),
            }
        }
        request::SetCarriageDeclaration(declaration) => {
            let declaration = declaration?;
            Command::SetCarriageDeclaration(CarriageDeclaration {
                plan_revision: declaration.get_expected_plan_revision(),
                destination_system_id: declaration.get_destination_system_id(),
                freight_capacity_millitons: declaration.get_freight_capacity_millitons(),
                high_berths: declaration.get_high_berths(),
                middle_berths: declaration.get_middle_berths(),
                steerage_berths: declaration.get_steerage_berths(),
                low_berths: declaration.get_low_berths(),
                accept_electronic_mail: declaration.get_accept_electronic_mail(),
            })
        }
        request::GetFinance(()) => Command::GetFinance,
        request::GetAccountLedger(query) => {
            let query = query?;
            Command::GetAccountLedger(AccountLedgerRequest {
                before_entry_id: query.get_before_entry_id(),
                limit: query.get_limit(),
                class: decode_account_transaction_class(query.get_class()?),
                ship_id: query.get_ship_id(),
            })
        }
        request::CureFinanceDefault(()) => Command::CureFinanceDefault,
        request::GetMarketKnowledge(()) => Command::GetMarketKnowledge,
        request::GetShipMarket(()) => Command::GetShipMarket,
        request::PurchaseShip(purchase) => {
            let purchase = purchase?;
            Command::PurchaseShip {
                offer_id: purchase.get_offer_id(),
                trade_in_current_ship: purchase.get_trade_in_current_ship(),
            }
        }
        request::CommissionShip(commission) => Command::CommissionShip {
            catalog_id: commission?.get_catalog_id(),
        },
        request::GetCrewMarket(()) => Command::GetCrewMarket,
        request::HireCrew(hire) => Command::HireCrew {
            candidate_id: hire?.get_candidate_id(),
        },
        request::BeginMarketSearch(search) => {
            let search = search?;
            Command::BeginMarketSearch {
                kind: decode_market_search_kind(search.get_kind()?),
                method: decode_market_search_method(search.get_method()?),
                person_id: search.get_person_id(),
                commodity_id: search.get_commodity_id(),
                destination_system_id: search.get_destination_system_id(),
                maximum_quantity_millitons: search.get_maximum_quantity_millitons(),
                cargo_lot_id: search.get_cargo_lot_id(),
            }
        }
        request::BeginMarketNegotiation(value) => {
            let value = value?;
            Command::BeginMarketNegotiation {
                lead_id: value.get_lead_id(),
                expected_revision: value.get_expected_revision(),
                person_id: value.get_person_id(),
            }
        }
        request::AcceptMarketQuote(value) => {
            let value = value?;
            Command::AcceptMarketQuote {
                lead_id: value.get_lead_id(),
                expected_revision: value.get_expected_revision(),
            }
        }
        request::RejectMarketQuote(value) => {
            let value = value?;
            Command::RejectMarketQuote {
                lead_id: value.get_lead_id(),
                expected_revision: value.get_expected_revision(),
            }
        }
        request::CancelWorkAssignment(cancel) => Command::CancelWorkAssignment {
            assignment_id: cancel?.get_assignment_id(),
        },
        request::GetCombat(()) => Command::GetCombat,
        request::SubmitCombatOrder(order) => {
            Command::SubmitCombatOrder(decode_combat_order(order?)?)
        }
        request::SetCombatAutomationPolicy(policy) => {
            let policy = policy?;
            Command::SetCombatAutomationPolicy(CombatAutomationPolicy {
                expected_revision: policy.get_expected_revision(),
                minimum_victory_percent: policy.get_minimum_victory_percent(),
                objective: decode_combat_objective(policy.get_objective()?),
                permit_surrender: policy.get_permit_surrender(),
                permit_abandon_ship: policy.get_permit_abandon_ship(),
            })
        }
        request::GetCombatCareer(()) => Command::GetCombatCareer,
        request::AcceptCareerOpportunity(value) => {
            let value = value?;
            Command::AcceptCareerOpportunity {
                opportunity_id: value.get_opportunity_id(),
                expected_revision: value.get_expected_revision(),
            }
        }
        request::EngageTrafficContact(value) => {
            let value = value?;
            Command::EngageTrafficContact {
                contact_id: value.get_contact_id(),
                expected_career_revision: value.get_expected_career_revision(),
                purpose: match value.get_purpose()? {
                    crate::ct_rpc_capnp::InterceptionPurpose::ArmedAttack => {
                        InterceptionPurpose::ArmedAttack
                    }
                    crate::ct_rpc_capnp::InterceptionPurpose::BoardingInspection => {
                        InterceptionPurpose::BoardingInspection
                    }
                    crate::ct_rpc_capnp::InterceptionPurpose::Arrest => InterceptionPurpose::Arrest,
                },
            }
        }
        request::SetInterceptionWatch(value) => {
            let value = value?;
            let expected_revision = value.get_expected_career_revision();
            let purpose = match value.get_purpose()? {
                crate::ct_rpc_capnp::InterceptionPurpose::ArmedAttack => {
                    InterceptionPurpose::ArmedAttack
                }
                crate::ct_rpc_capnp::InterceptionPurpose::BoardingInspection => {
                    InterceptionPurpose::BoardingInspection
                }
                crate::ct_rpc_capnp::InterceptionPurpose::Arrest => InterceptionPurpose::Arrest,
            };
            Command::SetInterceptionWatch(match value.which()? {
                crate::ct_rpc_capnp::interception_watch_request::Cancel(()) => {
                    InterceptionWatchRequest::Cancel { expected_revision }
                }
                crate::ct_rpc_capnp::interception_watch_request::AllCraft(()) => {
                    InterceptionWatchRequest::AllCraft {
                        expected_revision,
                        purpose,
                    }
                }
                crate::ct_rpc_capnp::interception_watch_request::CatalogId(catalog_id) => {
                    InterceptionWatchRequest::CraftClass {
                        expected_revision,
                        catalog_id,
                        purpose,
                    }
                }
            })
        }
        request::SetPirateCruise(value) => {
            let value = value?;
            Command::SetPirateCruise(crate::careers::PirateCruise {
                revision: value.get_expected_revision(),
                active: value.get_active(),
                hunting_system_id: value.get_hunting_system_id(),
                ends_second: value.get_ends_second(),
                crew_share_percent: value.get_crew_share_percent(),
                ship_fund_percent: value.get_ship_fund_percent(),
                prohibited_targets: value
                    .get_prohibited_targets()?
                    .to_str()
                    .map_err(|_| WireError::InvalidText)?
                    .to_owned(),
            })
        }
        request::SettlePrize(value) => {
            let value = value?;
            Command::SettlePrize {
                prize_id: value.get_prize_id(),
                expected_career_revision: value.get_expected_career_revision(),
                method: match value.get_method()? {
                    crate::ct_rpc_capnp::PrizeSettlementMethod::FileClaim => {
                        PrizeSettlementMethod::FileClaim
                    }
                    crate::ct_rpc_capnp::PrizeSettlementMethod::TakeAdvance => {
                        PrizeSettlementMethod::TakeAdvance
                    }
                    crate::ct_rpc_capnp::PrizeSettlementMethod::Fence => {
                        PrizeSettlementMethod::Fence
                    }
                    crate::ct_rpc_capnp::PrizeSettlementMethod::CourtSale => {
                        PrizeSettlementMethod::CourtSale
                    }
                    crate::ct_rpc_capnp::PrizeSettlementMethod::KeepPrize => {
                        PrizeSettlementMethod::KeepPrize
                    }
                    crate::ct_rpc_capnp::PrizeSettlementMethod::LaunderRegistry => {
                        PrizeSettlementMethod::LaunderRegistry
                    }
                },
            }
        }
        request::SettleWarrant(value) => {
            let value = value?;
            Command::SettleWarrant {
                warrant_id: value.get_warrant_id(),
                expected_career_revision: value.get_expected_career_revision(),
            }
        }
        request::SetCombatCareerMode(value) => {
            let value = value?;
            Command::SetCombatCareerMode {
                mode: match value.get_mode()? {
                    crate::ct_rpc_capnp::CombatCareerMode::Independent => {
                        crate::careers::CombatCareerMode::Independent
                    }
                    crate::ct_rpc_capnp::CombatCareerMode::Navy => {
                        crate::careers::CombatCareerMode::Navy
                    }
                    crate::ct_rpc_capnp::CombatCareerMode::Privateer => {
                        crate::careers::CombatCareerMode::Privateer
                    }
                    crate::ct_rpc_capnp::CombatCareerMode::Pirate => {
                        crate::careers::CombatCareerMode::Pirate
                    }
                },
                expected_revision: value.get_expected_revision(),
            }
        }
        request::RecoverCommand(value) => Command::RecoverCommand {
            successor_name: value?
                .get_successor_name()?
                .to_str()
                .map_err(|_| WireError::InvalidText)?
                .to_owned(),
        },
        request::DeclareBankruptcy(value) => Command::DeclareBankruptcy {
            successor_name: value?
                .get_successor_name()?
                .to_str()
                .map_err(|_| WireError::InvalidText)?
                .to_owned(),
        },
        request::AbandonPlayer(value) => Command::AbandonPlayer {
            confirmation: value?
                .get_confirmation()?
                .to_str()
                .map_err(|_| WireError::InvalidText)?
                .to_owned(),
        },
        request::GetDockedServices(()) => Command::GetDockedServices,
        request::CommitDockedService(order) => {
            let order = order?;
            let kind = match order.which()? {
                crate::ct_rpc_capnp::docked_service_order::Fuel(value) => {
                    let value = value?;
                    DockedServiceOrderKind::Fuel {
                        kind: match value.get_kind()? {
                            crate::ct_rpc_capnp::DockedFuelServiceKind::Refined => {
                                DockedFuelServiceKind::Refined
                            }
                            crate::ct_rpc_capnp::DockedFuelServiceKind::Unrefined => {
                                DockedFuelServiceKind::Unrefined
                            }
                            crate::ct_rpc_capnp::DockedFuelServiceKind::GasGiant => {
                                DockedFuelServiceKind::GasGiant
                            }
                            crate::ct_rpc_capnp::DockedFuelServiceKind::WildernessWater => {
                                DockedFuelServiceKind::WildernessWater
                            }
                        },
                        source_body_id: value
                            .get_has_source_body()
                            .then(|| value.get_source_body_id()),
                        quantity_millitons: value.get_quantity_millitons(),
                    }
                }
                crate::ct_rpc_capnp::docked_service_order::Ammunition(value) => {
                    let value = value?;
                    DockedServiceOrderKind::Ammunition {
                        ammunition_id: value
                            .get_ammunition_id()?
                            .to_str()
                            .map_err(|_| WireError::InvalidText)?
                            .to_owned(),
                        packs: value.get_packs(),
                    }
                }
                crate::ct_rpc_capnp::docked_service_order::Provisions(packages) => {
                    DockedServiceOrderKind::Provisions { packages }
                }
                crate::ct_rpc_capnp::docked_service_order::ProperRepair(subsystem_id) => {
                    DockedServiceOrderKind::ProperRepair { subsystem_id }
                }
                crate::ct_rpc_capnp::docked_service_order::Refit(()) => {
                    DockedServiceOrderKind::Refit
                }
                crate::ct_rpc_capnp::docked_service_order::Replacement(value) => {
                    let value = value?;
                    DockedServiceOrderKind::Replacement {
                        subsystem_id: value.get_subsystem_id(),
                        reconditioned: value.get_reconditioned(),
                    }
                }
            };
            Command::CommitDockedService(DockedServiceOrder {
                expected_ship_revision: order.get_expected_ship_revision(),
                kind,
            })
        }
        request::ReserveMarketLead(value) => {
            let value = value?;
            Command::ReserveMarketLead {
                lead_id: value.get_lead_id(),
                expected_revision: value.get_expected_revision(),
                quantity_millitons: value.get_quantity_millitons(),
            }
        }
        request::ReleaseMarketReservation(value) => {
            let value = value?;
            Command::ReleaseMarketReservation {
                lead_id: value.get_lead_id(),
                expected_revision: value.get_expected_revision(),
            }
        }
        request::ApplyTaskAction(value) => {
            let value = value?;
            Command::ApplyTaskAction {
                task_id: value.get_task_id(),
                expected_revision: value.get_expected_revision(),
                action: match value.get_action()? {
                    crate::ct_rpc_capnp::TaskActionKind::Cancel => TaskActionKind::Cancel,
                    crate::ct_rpc_capnp::TaskActionKind::ReturnCustody => {
                        TaskActionKind::ReturnCustody
                    }
                    crate::ct_rpc_capnp::TaskActionKind::DefaultTask => TaskActionKind::DefaultTask,
                    crate::ct_rpc_capnp::TaskActionKind::FileDispute => TaskActionKind::FileDispute,
                    crate::ct_rpc_capnp::TaskActionKind::WithdrawClaim => {
                        TaskActionKind::WithdrawClaim
                    }
                    crate::ct_rpc_capnp::TaskActionKind::FileLossClaim => {
                        TaskActionKind::FileLossClaim
                    }
                },
                explanation: value
                    .get_explanation()?
                    .to_str()
                    .map_err(|_| WireError::InvalidText)?
                    .to_owned(),
            }
        }
        request::SendPrivateMessage(value) => {
            let value = value?;
            let recipient = value.get_recipient()?;
            Command::SendPrivateMessage(PrivateMessageRequest {
                recipient_kind: match value.get_recipient_kind()? {
                    crate::ct_rpc_capnp::PrivateRecipientKind::System => {
                        PrivateRecipientKind::System
                    }
                    crate::ct_rpc_capnp::PrivateRecipientKind::Captain => {
                        PrivateRecipientKind::Captain
                    }
                },
                destination_system_id: value.get_destination_system_id(),
                recipient: PlayerIdentity {
                    bbs_id: recipient.get_bbs_id(),
                    player_id: recipient.get_player_id(),
                },
                encryption_key_id: value.get_encryption_key_id(),
                ttl_weeks: value.get_ttl_weeks(),
                subject: value
                    .get_subject()?
                    .to_str()
                    .map_err(|_| WireError::InvalidText)?
                    .to_owned(),
                body: value
                    .get_body()?
                    .to_str()
                    .map_err(|_| WireError::InvalidText)?
                    .to_owned(),
            })
        }
        request::PurchaseInsurance(value) => {
            let value = value?;
            Command::PurchaseInsurance {
                kind: match value.get_kind()? {
                    crate::ct_rpc_capnp::InsuranceKind::DestinationAssistance => {
                        InsuranceKind::DestinationAssistance
                    }
                },
                enabled: value.get_enabled(),
            }
        }
        request::MisappropriateRestrictedCredits(value) => {
            Command::MisappropriateRestrictedCredits {
                amount: value?.get_amount(),
            }
        }
        request::GetFleet(()) => Command::GetFleet,
        request::SetActiveShip(value) => {
            let value = value?;
            Command::SetActiveShip {
                expected_revision: value.get_expected_revision(),
                ship_id: value.get_ship_id(),
            }
        }
        request::AssignShipCaptain(value) => {
            let value = value?;
            Command::AssignShipCaptain {
                expected_revision: value.get_expected_revision(),
                ship_id: value.get_ship_id(),
                person_id: value.get_person_id(),
            }
        }
        request::TransferShipStores(value) => {
            let value = value?;
            Command::TransferShipStores {
                expected_revision: value.get_expected_revision(),
                from_ship_id: value.get_from_ship_id(),
                to_ship_id: value.get_to_ship_id(),
                kind: match value.get_kind()? {
                    crate::ct_rpc_capnp::StoreTransferKind::Cargo => StoreTransferKind::Cargo,
                    crate::ct_rpc_capnp::StoreTransferKind::Fuel => StoreTransferKind::Fuel,
                    crate::ct_rpc_capnp::StoreTransferKind::Ammunition => {
                        StoreTransferKind::Ammunition
                    }
                    crate::ct_rpc_capnp::StoreTransferKind::Provisions => {
                        StoreTransferKind::Provisions
                    }
                },
                cargo_lot_id: value.get_cargo_lot_id(),
                item_id: value
                    .get_item_id()?
                    .to_str()
                    .map_err(|_| WireError::InvalidText)?
                    .to_owned(),
                quantity: value.get_quantity(),
            }
        }
        request::GetSystemRadio(()) => Command::GetSystemRadio,
        request::TransmitSystemRadio(value) => Command::TransmitSystemRadio {
            body: value?
                .get_body()?
                .to_str()
                .map_err(|_| WireError::InvalidText)?
                .to_owned(),
        },
        request::PeekRadioReception(value) => Command::PeekRadioReception {
            reception_id: value?.get_reception_id(),
        },
        request::AcknowledgeRadioReception(value) => Command::AcknowledgeRadioReception {
            reception_id: value?.get_reception_id(),
        },
        request::SetRadioMute(value) => {
            let value = value?;
            let sender = value.get_sender()?;
            Command::SetRadioMute {
                sender: PlayerIdentity {
                    bbs_id: sender.get_bbs_id(),
                    player_id: sender.get_player_id(),
                },
                muted: value.get_muted(),
            }
        }
        request::GetBrowserAlertStatus(()) => Command::GetBrowserAlertStatus,
        request::CreateBrowserAlertEnrollment(()) => Command::CreateBrowserAlertEnrollment,
        request::RevokeAllBrowserAlerts(()) => Command::RevokeAllBrowserAlerts,
        request::GetOperationalDamageReport(()) => Command::GetOperationalDamageReport,
        request::AcknowledgeOperationalDamageReport(value) => {
            Command::AcknowledgeOperationalDamageReport {
                report_id: value?.get_report_id(),
            }
        }
        request::GetEncounterPolicyDefault(()) => Command::GetEncounterPolicyDefault,
        request::SetEncounterPolicyDefault(value) => {
            let value = value?;
            Command::SetEncounterPolicyDefault(SetEncounterPolicyDefaultRequest {
                expected_revision: value.get_expected_revision(),
                policy: decode_encounter_policy(value.get_policy()?)?,
                acknowledge_nonhostile_fight: value.get_acknowledge_nonhostile_fight(),
            })
        }
    };
    Ok(CommandRequest {
        request_id,
        session_epoch,
        command_id,
        command,
    })
}

pub fn decode_close(bytes: &[u8]) -> Result<Option<String>, WireError> {
    let message = message_reader(bytes)?;
    let envelope = message.get_root::<envelope::Reader>()?;
    check_version(envelope.get_protocol_version())?;
    let envelope::Close(close) = envelope.which()? else {
        return Ok(None);
    };
    Ok(Some(
        close?
            .get_message()?
            .to_str()
            .map_err(|_| WireError::InvalidText)?
            .to_owned(),
    ))
}

fn finish_message(message: &Builder<capnp::message::HeapAllocator>) -> Result<Vec<u8>, WireError> {
    let mut bytes = Vec::new();
    serialize::write_message(&mut bytes, message)?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(WireError::FrameTooLarge);
    }
    Ok(bytes)
}

pub fn encode_server_hello(
    identity: &PlayerIdentity,
    epoch: u64,
    committed_sequence: u64,
    phase: PlayerPhase,
    language_tag: &str,
    formatting: &DisplayFormatting,
) -> Result<Vec<u8>, WireError> {
    encode_server_hello_with_affiliation(
        identity,
        epoch,
        committed_sequence,
        phase,
        language_tag,
        formatting,
        None,
    )
}

pub fn encode_server_hello_with_affiliation(
    identity: &PlayerIdentity,
    epoch: u64,
    committed_sequence: u64,
    phase: PlayerPhase,
    language_tag: &str,
    formatting: &DisplayFormatting,
    affiliation: Option<&InstitutionalAffiliation>,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut hello = envelope.init_server_hello();
    let mut hello_identity = hello.reborrow().init_identity();
    hello_identity.set_bbs_id(identity.bbs_id);
    hello_identity.set_player_id(identity.player_id);
    hello.set_assigned_epoch(epoch);
    hello.set_committed_sequence(committed_sequence);
    hello.set_phase(schema_phase(phase));
    hello.set_language_tag(language_tag);
    hello.set_account_journal_available(true);
    let mut wire_formatting = hello.reborrow().init_formatting();
    wire_formatting.set_decimal_separator(formatting.decimal_separator);
    wire_formatting.set_grouping_separator(formatting.grouping_separator);
    wire_formatting.set_primary_grouping_digits(formatting.primary_grouping_digits);
    wire_formatting.set_secondary_grouping_digits(formatting.secondary_grouping_digits);
    wire_formatting.set_game_timestamp_pattern(formatting.game_timestamp_pattern);
    wire_formatting.set_game_duration_pattern(formatting.game_duration_pattern);
    wire_formatting.set_real_duration_pattern(formatting.real_duration_pattern);
    if let Some(affiliation) = affiliation {
        let mut wire = hello.init_affiliation();
        wire.set_polity_name(&affiliation.polity_name);
        wire.set_bbs_name(&affiliation.bbs_name);
        wire.set_league_name(affiliation.league_name.as_deref().unwrap_or(""));
    }
    finish_message(&message)
}

pub fn encode_response(
    request_id: u64,
    epoch: u64,
    outcome: &Outcome,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request_id);
    envelope.set_session_epoch(epoch);
    let mut response = envelope.init_response();
    response.set_command_id(&outcome.command_id);
    response.set_committed_sequence(outcome.committed_sequence);
    response.set_revision(outcome.revision);
    response.set_phase(schema_phase(outcome.phase));
    response.set_replayed(outcome.replayed);
    match &outcome.kind {
        OutcomeKind::Pong => response.set_pong(()),
        OutcomeKind::PlayerCreated(profile) => {
            set_player_creation(response.reborrow().init_player_created(), profile)?;
        }
        OutcomeKind::CaptainCreationOptions(options) => {
            set_captain_options(response.reborrow().init_captain_creation_options(), options)?;
        }
        OutcomeKind::StartingShipOffers(offers) => {
            set_starting_ship_offers(response.reborrow().init_starting_ship_offers(), offers)?;
        }
        OutcomeKind::StartingShipOptions(options) => {
            set_starting_ship_options(response.reborrow().init_starting_ship_options(), options)?;
        }
        OutcomeKind::StartingCrewPlan(plan) => {
            set_starting_crew_plan(response.reborrow().init_starting_crew_plan(), plan)?;
        }
        OutcomeKind::CrewManagement(snapshot) => {
            set_crew_management(response.reborrow().init_crew_management(), snapshot)?;
        }
        OutcomeKind::ShipStatus(snapshot) => {
            set_ship_status(response.reborrow().init_ship_status(), snapshot)?;
        }
        OutcomeKind::DockedSnapshot(snapshot) => {
            set_docked_snapshot(response.reborrow().init_docked_snapshot(), snapshot);
        }
        OutcomeKind::KnownDestinations(snapshot) => {
            set_known_destinations(response.reborrow().init_known_destinations(), snapshot)?;
        }
        OutcomeKind::Market(snapshot) => {
            set_market(response.reborrow().init_market(), snapshot)?;
        }
        OutcomeKind::TravelStatus(snapshot) => {
            set_travel_status(response.reborrow().init_travel_status(), snapshot);
        }
        OutcomeKind::CoursePlot(plot) => {
            set_course_plot(response.reborrow().init_course_plot(), plot)?;
        }
        OutcomeKind::ArrivalPacket(packet) => {
            set_arrival_packet(response.reborrow().init_arrival_packet(), packet)?;
        }
        OutcomeKind::MessageManagement(snapshot) => {
            set_message_management(response.reborrow().init_message_management(), snapshot)?;
        }
        OutcomeKind::SystemMappingStatus(status) => {
            set_system_mapping_status(response.reborrow().init_system_mapping_status(), *status);
        }
        OutcomeKind::FlightPlan(snapshot) => {
            set_flight_plan_snapshot(response.reborrow().init_flight_plan(), snapshot)?;
        }
        OutcomeKind::FlightPlanPreview(preview) => {
            set_flight_plan_preview(response.reborrow().init_flight_plan_preview(), preview)?;
        }
        OutcomeKind::EncounterPolicyDefault(snapshot) => {
            let mut target = response.reborrow().init_encounter_policy_default();
            target.set_ship_id(snapshot.ship_id);
            target.set_revision(snapshot.revision);
            set_encounter_policy(target.init_policy(), &snapshot.policy)?;
        }
        OutcomeKind::Checkpoint(checkpoint) => {
            set_checkpoint_snapshot(response.reborrow().init_checkpoint(), checkpoint);
        }
        OutcomeKind::Encounter(encounter) => {
            set_encounter_snapshot(response.reborrow().init_encounter(), encounter);
        }
        OutcomeKind::EncounterResult(result) => {
            set_encounter_result(response.reborrow().init_encounter_result(), result);
        }
        OutcomeKind::TerminalReport(report) => {
            set_terminal_report(response.reborrow().init_terminal_report(), report)?;
        }
        OutcomeKind::BrowserAlertStatus(status) => {
            set_browser_alert_status(response.reborrow().init_browser_alert_status(), status);
        }
        OutcomeKind::BrowserAlertEnrollment(enrollment) => {
            let mut wire = response.reborrow().init_browser_alert_enrollment();
            set_browser_alert_status(
                wire.reborrow().init_status(),
                &crate::web_push::BrowserAlertStatus {
                    configured: true,
                    active_devices: enrollment.active_devices,
                    maximum_devices: enrollment.maximum_devices,
                },
            );
            wire.set_url(&enrollment.url);
            wire.set_expires_unix_second(enrollment.expires_unix_second);
        }
        OutcomeKind::OperationalDamageReport(report) => {
            set_operational_damage_report(
                response.reborrow().init_operational_damage_report(),
                report,
            );
        }
        OutcomeKind::Combat(snapshot) => {
            set_combat_snapshot(response.reborrow().init_combat(), snapshot)?;
        }
        OutcomeKind::CombatCareer(snapshot) => {
            set_combat_career_snapshot(response.reborrow().init_combat_career(), snapshot)?;
        }
        OutcomeKind::TaskLedger(ledger) => {
            set_task_ledger(response.reborrow().init_task_ledger(), ledger)?;
        }
        OutcomeKind::Finance(finance) => {
            set_finance(response.reborrow().init_finance(), finance);
        }
        OutcomeKind::AccountLedger(page) => {
            set_account_ledger(response.reborrow().init_account_ledger(), page)?;
        }
        OutcomeKind::MarketKnowledge(knowledge) => {
            set_market_knowledge(response.reborrow().init_market_knowledge(), knowledge)?;
        }
        OutcomeKind::ShipMarket(market) => {
            set_ship_market(response.reborrow().init_ship_market(), market)?;
        }
        OutcomeKind::CrewMarket(market) => {
            set_crew_market(response.reborrow().init_crew_market(), market)?;
        }
        OutcomeKind::DockedServices(snapshot) => {
            set_docked_services(response.reborrow().init_docked_services(), snapshot)?;
        }
        OutcomeKind::DockedServiceReceipt(receipt) => {
            set_docked_service_receipt(response.reborrow().init_docked_service_receipt(), receipt)?;
        }
        OutcomeKind::Fleet(snapshot) => {
            set_fleet(response.reborrow().init_fleet(), snapshot)?;
        }
        OutcomeKind::SystemRadio(snapshot) => {
            set_system_radio(response.reborrow().init_system_radio(), snapshot)?;
        }
        OutcomeKind::RadioContent(content) => {
            let mut result = response.reborrow().init_radio_content();
            result.set_reception_id(content.reception_id);
            result.set_transmission_id(content.transmission_id);
            result.set_body(&content.body);
        }
        OutcomeKind::Error { code, message } => {
            let mut error = response.reborrow().init_error();
            error.set_code(match code {
                ErrorCode::InvalidCommand => SchemaErrorCode::InvalidCommand,
                ErrorCode::StaleSession => SchemaErrorCode::StaleSession,
                ErrorCode::MalformedMessage => SchemaErrorCode::MalformedMessage,
                ErrorCode::UnsupportedVersion => SchemaErrorCode::UnsupportedVersion,
                ErrorCode::InternalFailure => SchemaErrorCode::InternalFailure,
            });
            error.set_message(message);
        }
    }
    finish_message(&message)
}

pub fn encode_session_replaced(epoch: u64) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    envelope.init_event().set_session_replaced(());
    finish_message(&message)
}

pub fn encode_server_stopping(epoch: u64) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    envelope.init_event().set_server_stopping(());
    finish_message(&message)
}

pub fn encode_phase_changed(
    epoch: u64,
    transition: &crate::store::PlayerTravelTransition,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut event = envelope.init_event();
    event.set_committed_sequence(transition.committed_sequence);
    let mut changed = event.init_phase_changed();
    changed.set_revision(transition.revision);
    changed.set_phase(schema_phase(transition.phase));
    set_travel_status(changed.init_travel_status(), &transition.status);
    finish_message(&message)
}

pub fn encode_checkpoint_ready(
    epoch: u64,
    committed_sequence: u64,
    checkpoint: &CheckpointSnapshot,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut event = envelope.init_event();
    event.set_committed_sequence(committed_sequence);
    set_checkpoint_snapshot(event.init_checkpoint_ready(), checkpoint);
    finish_message(&message)
}

pub fn encode_encounter_ready(
    epoch: u64,
    committed_sequence: u64,
    encounter: &EncounterSnapshot,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut event = envelope.init_event();
    event.set_committed_sequence(committed_sequence);
    set_encounter_snapshot(event.init_encounter_ready(), encounter);
    finish_message(&message)
}

pub fn encode_traffic_snapshot(
    epoch: u64,
    committed_sequence: u64,
    snapshot: &crate::traffic::TrafficSnapshot,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut event = envelope.init_event();
    event.set_committed_sequence(committed_sequence);
    let mut builder = event.init_traffic_snapshot();
    builder.set_system_id(snapshot.system_id);
    builder.set_system_name(&snapshot.system_name);
    builder.set_observed_second(snapshot.observed_second);
    let count = u32::try_from(snapshot.contacts.len())
        .map_err(|_| WireError::Expected("fewer observable traffic contacts"))?;
    let mut contacts = builder.init_contacts(count);
    for (index, contact) in snapshot.contacts.iter().enumerate() {
        set_traffic_contact(contacts.reborrow().get(index as u32), contact);
    }
    finish_message(&message)
}

pub fn encode_traffic_movement(
    epoch: u64,
    committed_sequence: u64,
    system_id: u64,
    observed_second: u64,
    contact: &crate::traffic::TrafficContact,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut event = envelope.init_event();
    event.set_committed_sequence(committed_sequence);
    let mut movement = event.init_traffic_movement();
    movement.set_system_id(system_id);
    movement.set_observed_second(observed_second);
    set_traffic_contact(movement.init_contact(), contact);
    finish_message(&message)
}

pub fn encode_radio_unread(
    epoch: u64,
    committed_sequence: u64,
    ship_id: u64,
    unread_count: u64,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut event = envelope.init_event();
    event.set_committed_sequence(committed_sequence);
    let mut unread = event.init_radio_unread();
    unread.set_ship_id(ship_id);
    unread.set_unread_count(unread_count);
    finish_message(&message)
}

pub fn encode_operational_damage_ready(
    epoch: u64,
    committed_sequence: u64,
    report: &OperationalDamageReport,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut event = envelope.init_event();
    event.set_committed_sequence(committed_sequence);
    set_operational_damage_report(event.init_operational_damage_ready(), report);
    finish_message(&message)
}

pub fn encode_close(epoch: u64, message: &str) -> Result<Vec<u8>, WireError> {
    encode_close_with_code(epoch, CloseCode::Unspecified, message, &[])
}

pub fn encode_close_with_code(
    epoch: u64,
    code: CloseCode,
    message: &str,
    supported_language_tags: &[&str],
) -> Result<Vec<u8>, WireError> {
    let mut capnp_message = Builder::new_default();
    let mut envelope = capnp_message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_session_epoch(epoch);
    let mut close = envelope.init_close();
    close.set_code(schema_close_code(code));
    close.set_message(message);
    let count = u32::try_from(supported_language_tags.len())
        .map_err(|_| WireError::Expected("fewer supported language tags"))?;
    let mut tags = close.init_supported_language_tags(count);
    for (index, tag) in supported_language_tags.iter().enumerate() {
        tags.set(index as u32, tag);
    }
    finish_message(&capnp_message)
}

pub fn encode_legacy_close_for_version(
    protocol_version: u16,
    epoch: u64,
    reason: &str,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<crate::ct_rpc_capnp::legacy_v2_envelope::Builder>();
    envelope.set_protocol_version(protocol_version);
    envelope.set_session_epoch(epoch);
    envelope.init_close().set_reason(reason);
    finish_message(&message)
}

#[cfg(test)]
pub fn encode_client_hello(identity: &PlayerIdentity) -> Result<Vec<u8>, WireError> {
    encode_client_hello_for_version(PROTOCOL_VERSION, identity)
}

#[cfg(test)]
fn encode_client_hello_for_version(
    protocol_version: u16,
    identity: &PlayerIdentity,
) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(protocol_version);
    let mut hello = envelope.init_client_hello();
    let mut hello_identity = hello.reborrow().init_identity();
    hello_identity.set_bbs_id(identity.bbs_id);
    hello_identity.set_player_id(identity.player_id);
    hello.set_client_name("test");
    hello.set_language_tag("en");
    finish_message(&message)
}

#[cfg(test)]
pub fn encode_request(request: &CommandRequest) -> Result<Vec<u8>, WireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    envelope.set_session_epoch(request.session_epoch);
    let mut builder = envelope.init_request();
    builder.set_command_id(&request.command_id);
    match request.command {
        Command::Ping => builder.set_ping(()),
        Command::CreatePlayer(ref profile) => {
            set_player_creation(builder.init_create_player(), profile)?;
        }
        Command::GetCaptainCreationOptions => builder.set_get_captain_creation_options(()),
        Command::GetStartingShipOffers => builder.set_get_starting_ship_offers(()),
        Command::GetStartingShipOptions {
            setup_revision,
            starting_offer_id,
        } => {
            let mut query = builder.init_get_starting_ship_options();
            query.set_setup_revision(setup_revision);
            query.set_starting_offer_id(starting_offer_id);
        }
        Command::GetStartingCrewPlan {
            setup_revision,
            starting_offer_id,
        } => {
            let mut query = builder.init_get_starting_crew_plan();
            query.set_setup_revision(setup_revision);
            query.set_starting_offer_id(starting_offer_id);
        }
        Command::GetCrewManagement => builder.set_get_crew_management(()),
        Command::SetCrewTrainingTarget { person_id, skill } => {
            let mut change = builder.init_set_crew_training_target();
            change.set_person_id(person_id);
            change.set_skill(encode_skill(skill));
        }
        Command::SetCrewAssignments {
            person_id,
            ref slot_ids,
        } => {
            let mut change = builder.init_set_crew_assignments();
            change.set_person_id(person_id);
            let count = u32::try_from(slot_ids.len())
                .map_err(|_| WireError::Expected("fewer crew assignments"))?;
            let mut assigned = change.init_slot_ids(count);
            for (index, slot_id) in slot_ids.iter().enumerate() {
                assigned.set(index as u32, *slot_id);
            }
        }
        Command::ApplyPersonnelAction {
            person_id,
            expected_service_revision,
            action,
            target_ship_id,
            duration_days,
        } => {
            let mut change = builder.init_apply_personnel_action();
            change.set_person_id(person_id);
            change.set_expected_service_revision(expected_service_revision);
            change.set_action(match action {
                PersonnelActionKind::Dismiss => crate::ct_rpc_capnp::PersonnelActionKind::Dismiss,
                PersonnelActionKind::Transfer => crate::ct_rpc_capnp::PersonnelActionKind::Transfer,
                PersonnelActionKind::ShoreLeave => {
                    crate::ct_rpc_capnp::PersonnelActionKind::ShoreLeave
                }
                PersonnelActionKind::Recall => crate::ct_rpc_capnp::PersonnelActionKind::Recall,
                PersonnelActionKind::FirstAid => crate::ct_rpc_capnp::PersonnelActionKind::FirstAid,
                PersonnelActionKind::Surgery => crate::ct_rpc_capnp::PersonnelActionKind::Surgery,
                PersonnelActionKind::MedicalCare => {
                    crate::ct_rpc_capnp::PersonnelActionKind::MedicalCare
                }
            });
            change.set_target_ship_id(target_ship_id);
            change.set_duration_days(duration_days);
        }
        Command::GetShipStatus => builder.set_get_ship_status(()),
        Command::GetDockedSnapshot => builder.set_get_docked_snapshot(()),
        Command::GetKnownDestinations => builder.set_get_known_destinations(()),
        Command::GetMarket => builder.set_get_market(()),
        Command::BuyCargo {
            market_revision,
            offer_id,
            quantity_millitons,
        } => {
            let mut change = builder.init_buy_cargo();
            change.set_market_revision(market_revision);
            change.set_offer_id(offer_id);
            change.set_quantity_millitons(quantity_millitons);
        }
        Command::SellCargo {
            market_revision,
            cargo_lot_id,
            quantity_millitons,
            buyer_lead_id,
        } => {
            let mut change = builder.init_sell_cargo();
            change.set_market_revision(market_revision);
            change.set_cargo_lot_id(cargo_lot_id);
            change.set_quantity_millitons(quantity_millitons);
            change.set_buyer_lead_id(buyer_lead_id);
        }
        Command::GetTravelStatus => builder.set_get_travel_status(()),
        Command::BeginVoyage {
            destination_system_id,
        } => builder
            .init_begin_voyage()
            .set_destination_system_id(destination_system_id),
        Command::PlotCourse {
            origin_system_id,
            destination_system_id,
            use_current_fuel,
        } => {
            let mut query = builder.init_plot_course();
            query.set_origin_system_id(origin_system_id);
            query.set_destination_system_id(destination_system_id);
            query.set_use_current_fuel(use_current_fuel);
        }
        Command::SuggestTaskCourse => builder.set_suggest_task_course(()),
        Command::OpenArrivalPacket => builder.set_open_arrival_packet(()),
        Command::GetMessageManagement => builder.set_get_message_management(()),
        Command::SetMessageClassification {
            message_id,
            classification,
        } => {
            let mut change = builder.init_set_message_classification();
            change.set_message_id(message_id);
            change.set_classification(encode_message_classification(classification));
        }
        Command::SetMessageFilter {
            class,
            minimum_importance,
        } => {
            let mut change = builder.init_set_message_filter();
            change.set_class(encode_message_class(class));
            change.set_minimum_importance(encode_message_importance(minimum_importance));
        }
        Command::SetSystemMappingDisclosure { system_id, choice } => {
            let mut change = builder.init_set_system_mapping_disclosure();
            change.set_system_id(system_id);
            change.set_choice(match choice {
                SystemMappingChoice::PublicNotification => {
                    crate::ct_rpc_capnp::SystemMappingChoice::PublicNotification
                }
                SystemMappingChoice::DirectEarth => {
                    crate::ct_rpc_capnp::SystemMappingChoice::DirectEarth
                }
                SystemMappingChoice::Withhold => crate::ct_rpc_capnp::SystemMappingChoice::Withhold,
                SystemMappingChoice::WithholdSecret => {
                    crate::ct_rpc_capnp::SystemMappingChoice::WithholdSecret
                }
            });
        }
        Command::GetFlightPlan => builder.set_get_flight_plan(()),
        Command::GetEncounterPolicyDefault => builder.set_get_encounter_policy_default(()),
        Command::SetEncounterPolicyDefault(ref request) => {
            let mut value = builder.init_set_encounter_policy_default();
            value.set_expected_revision(request.expected_revision);
            set_encounter_policy(value.reborrow().init_policy(), &request.policy)?;
            value.set_acknowledge_nonhostile_fight(request.acknowledge_nonhostile_fight);
        }
        Command::PreviewFlightPlan(ref proposal) => {
            set_flight_plan_proposal(builder.init_preview_flight_plan(), proposal)?;
        }
        Command::CommitFlightPlan(ref request) => {
            let mut commit = builder.init_commit_flight_plan();
            set_flight_plan_proposal(commit.reborrow().init_proposal(), &request.proposal)?;
            commit.set_preview_hash(&request.preview_hash);
            commit.set_acknowledge_warnings(request.acknowledge_warnings);
        }
        Command::AcknowledgeCheckpoint { checkpoint_id } => builder
            .init_acknowledge_checkpoint()
            .set_checkpoint_id(checkpoint_id),
        Command::GetEncounter => builder.set_get_encounter(()),
        Command::GetTerminalReport => builder.set_get_terminal_report(()),
        Command::AcknowledgeTerminalReport {
            encounter_id,
            expected_revision,
        } => {
            let mut value = builder.init_acknowledge_terminal_report();
            value.set_encounter_id(encounter_id);
            value.set_expected_revision(expected_revision);
        }
        Command::ResolveEncounter(ref request) => {
            let mut resolve = builder.init_resolve_encounter();
            resolve.set_encounter_id(request.encounter_id);
            resolve.set_expected_revision(request.expected_revision);
            resolve.set_posture(encode_encounter_posture(request.posture));
            let count = u32::try_from(request.fallbacks.len())
                .map_err(|_| WireError::Expected("fewer encounter fallbacks"))?;
            let mut values = resolve.init_fallbacks(count);
            for (index, value) in request.fallbacks.iter().enumerate() {
                values.set(index as u32, encode_encounter_fallback(*value));
            }
        }
        Command::GetTaskLedger => builder.set_get_task_ledger(()),
        Command::AcceptTaskOffer {
            offer_id,
            expected_revision,
        } => {
            let mut accept = builder.init_accept_task_offer();
            accept.set_offer_id(offer_id);
            accept.set_expected_revision(expected_revision);
        }
        Command::SetCarriageDeclaration(declaration) => {
            let mut value = builder.init_set_carriage_declaration();
            value.set_expected_plan_revision(declaration.plan_revision);
            value.set_destination_system_id(declaration.destination_system_id);
            value.set_freight_capacity_millitons(declaration.freight_capacity_millitons);
            value.set_high_berths(declaration.high_berths);
            value.set_middle_berths(declaration.middle_berths);
            value.set_steerage_berths(declaration.steerage_berths);
            value.set_low_berths(declaration.low_berths);
            value.set_accept_electronic_mail(declaration.accept_electronic_mail);
        }
        Command::GetFinance => builder.set_get_finance(()),
        Command::GetAccountLedger(ref query) => {
            let mut value = builder.init_get_account_ledger();
            value.set_before_entry_id(query.before_entry_id);
            value.set_limit(query.limit);
            value.set_class(schema_account_transaction_class(query.class));
            value.set_ship_id(query.ship_id);
        }
        Command::CureFinanceDefault => builder.set_cure_finance_default(()),
        Command::GetMarketKnowledge => builder.set_get_market_knowledge(()),
        Command::GetShipMarket => builder.set_get_ship_market(()),
        Command::PurchaseShip {
            offer_id,
            trade_in_current_ship,
        } => {
            let mut purchase = builder.init_purchase_ship();
            purchase.set_offer_id(offer_id);
            purchase.set_trade_in_current_ship(trade_in_current_ship);
        }
        Command::CommissionShip { catalog_id } => {
            builder.init_commission_ship().set_catalog_id(catalog_id)
        }
        Command::GetCrewMarket => builder.set_get_crew_market(()),
        Command::HireCrew { candidate_id } => {
            builder.init_hire_crew().set_candidate_id(candidate_id)
        }
        Command::BeginMarketSearch {
            kind,
            method,
            person_id,
            commodity_id,
            destination_system_id,
            maximum_quantity_millitons,
            cargo_lot_id,
        } => {
            let mut search = builder.init_begin_market_search();
            search.set_kind(match kind {
                MarketSearchKind::Supplier => crate::ct_rpc_capnp::MarketSearchKind::Supplier,
                MarketSearchKind::Buyer => crate::ct_rpc_capnp::MarketSearchKind::Buyer,
                MarketSearchKind::Freight => crate::ct_rpc_capnp::MarketSearchKind::Freight,
                MarketSearchKind::Passengers => crate::ct_rpc_capnp::MarketSearchKind::Passengers,
            });
            search.set_method(match method {
                MarketSearchMethod::Physical => crate::ct_rpc_capnp::MarketSearchMethod::Physical,
                MarketSearchMethod::Online => crate::ct_rpc_capnp::MarketSearchMethod::Online,
                MarketSearchMethod::BlackMarket => {
                    crate::ct_rpc_capnp::MarketSearchMethod::BlackMarket
                }
                MarketSearchMethod::HiredBroker => {
                    crate::ct_rpc_capnp::MarketSearchMethod::HiredBroker
                }
            });
            search.set_person_id(person_id);
            search.set_commodity_id(commodity_id);
            search.set_destination_system_id(destination_system_id);
            search.set_maximum_quantity_millitons(maximum_quantity_millitons);
            search.set_cargo_lot_id(cargo_lot_id);
        }
        Command::BeginMarketNegotiation {
            lead_id,
            expected_revision,
            person_id,
        } => {
            let mut value = builder.init_begin_market_negotiation();
            value.set_lead_id(lead_id);
            value.set_expected_revision(expected_revision);
            value.set_person_id(person_id);
        }
        Command::AcceptMarketQuote {
            lead_id,
            expected_revision,
        } => {
            let mut value = builder.init_accept_market_quote();
            value.set_lead_id(lead_id);
            value.set_expected_revision(expected_revision);
        }
        Command::RejectMarketQuote {
            lead_id,
            expected_revision,
        } => {
            let mut value = builder.init_reject_market_quote();
            value.set_lead_id(lead_id);
            value.set_expected_revision(expected_revision);
        }
        Command::CancelWorkAssignment { assignment_id } => builder
            .init_cancel_work_assignment()
            .set_assignment_id(assignment_id),
        Command::GetCombat => builder.set_get_combat(()),
        Command::SubmitCombatOrder(ref order) => {
            set_combat_order(builder.init_submit_combat_order(), order)?;
        }
        Command::SetCombatAutomationPolicy(ref policy) => {
            set_combat_policy(builder.init_set_combat_automation_policy(), policy);
        }
        Command::GetCombatCareer => builder.set_get_combat_career(()),
        Command::AcceptCareerOpportunity {
            opportunity_id,
            expected_revision,
        } => {
            let mut value = builder.init_accept_career_opportunity();
            value.set_opportunity_id(opportunity_id);
            value.set_expected_revision(expected_revision);
        }
        Command::EngageTrafficContact {
            contact_id,
            expected_career_revision,
            purpose,
        } => {
            let mut value = builder.init_engage_traffic_contact();
            value.set_contact_id(contact_id);
            value.set_expected_career_revision(expected_career_revision);
            value.set_purpose(match purpose {
                InterceptionPurpose::ArmedAttack => {
                    crate::ct_rpc_capnp::InterceptionPurpose::ArmedAttack
                }
                InterceptionPurpose::BoardingInspection => {
                    crate::ct_rpc_capnp::InterceptionPurpose::BoardingInspection
                }
                InterceptionPurpose::Arrest => crate::ct_rpc_capnp::InterceptionPurpose::Arrest,
            });
        }
        Command::SetInterceptionWatch(request) => {
            let mut value = builder.init_set_interception_watch();
            match request {
                InterceptionWatchRequest::Cancel { expected_revision } => {
                    value.set_expected_career_revision(expected_revision);
                    value.set_cancel(());
                }
                InterceptionWatchRequest::AllCraft {
                    expected_revision,
                    purpose,
                } => {
                    value.set_expected_career_revision(expected_revision);
                    value.set_purpose(match purpose {
                        InterceptionPurpose::ArmedAttack => {
                            crate::ct_rpc_capnp::InterceptionPurpose::ArmedAttack
                        }
                        InterceptionPurpose::BoardingInspection => {
                            crate::ct_rpc_capnp::InterceptionPurpose::BoardingInspection
                        }
                        InterceptionPurpose::Arrest => {
                            crate::ct_rpc_capnp::InterceptionPurpose::Arrest
                        }
                    });
                    value.set_all_craft(());
                }
                InterceptionWatchRequest::CraftClass {
                    expected_revision,
                    catalog_id,
                    purpose,
                } => {
                    value.set_expected_career_revision(expected_revision);
                    value.set_purpose(match purpose {
                        InterceptionPurpose::ArmedAttack => {
                            crate::ct_rpc_capnp::InterceptionPurpose::ArmedAttack
                        }
                        InterceptionPurpose::BoardingInspection => {
                            crate::ct_rpc_capnp::InterceptionPurpose::BoardingInspection
                        }
                        InterceptionPurpose::Arrest => {
                            crate::ct_rpc_capnp::InterceptionPurpose::Arrest
                        }
                    });
                    value.set_catalog_id(catalog_id);
                }
            }
        }
        Command::SetPirateCruise(ref cruise) => {
            let mut value = builder.init_set_pirate_cruise();
            value.set_expected_revision(cruise.revision);
            value.set_active(cruise.active);
            value.set_hunting_system_id(cruise.hunting_system_id);
            value.set_ends_second(cruise.ends_second);
            value.set_crew_share_percent(cruise.crew_share_percent);
            value.set_ship_fund_percent(cruise.ship_fund_percent);
            value.set_prohibited_targets(&cruise.prohibited_targets);
        }
        Command::SettlePrize {
            prize_id,
            expected_career_revision,
            method,
        } => {
            let mut value = builder.init_settle_prize();
            value.set_prize_id(prize_id);
            value.set_expected_career_revision(expected_career_revision);
            value.set_method(match method {
                PrizeSettlementMethod::FileClaim => {
                    crate::ct_rpc_capnp::PrizeSettlementMethod::FileClaim
                }
                PrizeSettlementMethod::TakeAdvance => {
                    crate::ct_rpc_capnp::PrizeSettlementMethod::TakeAdvance
                }
                PrizeSettlementMethod::Fence => crate::ct_rpc_capnp::PrizeSettlementMethod::Fence,
                PrizeSettlementMethod::CourtSale => {
                    crate::ct_rpc_capnp::PrizeSettlementMethod::CourtSale
                }
                PrizeSettlementMethod::KeepPrize => {
                    crate::ct_rpc_capnp::PrizeSettlementMethod::KeepPrize
                }
                PrizeSettlementMethod::LaunderRegistry => {
                    crate::ct_rpc_capnp::PrizeSettlementMethod::LaunderRegistry
                }
            });
        }
        Command::SettleWarrant {
            warrant_id,
            expected_career_revision,
        } => {
            let mut value = builder.init_settle_warrant();
            value.set_warrant_id(warrant_id);
            value.set_expected_career_revision(expected_career_revision);
        }
        Command::SetCombatCareerMode {
            mode,
            expected_revision,
        } => {
            let mut value = builder.init_set_combat_career_mode();
            value.set_mode(match mode {
                crate::careers::CombatCareerMode::Independent => {
                    crate::ct_rpc_capnp::CombatCareerMode::Independent
                }
                crate::careers::CombatCareerMode::Navy => {
                    crate::ct_rpc_capnp::CombatCareerMode::Navy
                }
                crate::careers::CombatCareerMode::Privateer => {
                    crate::ct_rpc_capnp::CombatCareerMode::Privateer
                }
                crate::careers::CombatCareerMode::Pirate => {
                    crate::ct_rpc_capnp::CombatCareerMode::Pirate
                }
            });
            value.set_expected_revision(expected_revision);
        }
        Command::RecoverCommand { ref successor_name } => {
            builder
                .init_recover_command()
                .set_successor_name(successor_name);
        }
        Command::DeclareBankruptcy { ref successor_name } => {
            builder
                .init_declare_bankruptcy()
                .set_successor_name(successor_name);
        }
        Command::AbandonPlayer { ref confirmation } => {
            builder.init_abandon_player().set_confirmation(confirmation);
        }
        Command::GetDockedServices => builder.set_get_docked_services(()),
        Command::CommitDockedService(ref order) => {
            let mut target = builder.init_commit_docked_service();
            target.set_expected_ship_revision(order.expected_ship_revision);
            match &order.kind {
                DockedServiceOrderKind::Fuel {
                    kind,
                    source_body_id,
                    quantity_millitons,
                } => {
                    let mut value = target.init_fuel();
                    value.set_kind(schema_docked_fuel_kind(*kind));
                    value.set_has_source_body(source_body_id.is_some());
                    value.set_source_body_id(source_body_id.unwrap_or(0));
                    value.set_quantity_millitons(*quantity_millitons);
                }
                DockedServiceOrderKind::Ammunition {
                    ammunition_id,
                    packs,
                } => {
                    let mut value = target.init_ammunition();
                    value.set_ammunition_id(ammunition_id);
                    value.set_packs(*packs);
                }
                DockedServiceOrderKind::Provisions { packages } => target.set_provisions(*packages),
                DockedServiceOrderKind::ProperRepair { subsystem_id } => {
                    target.set_proper_repair(*subsystem_id)
                }
                DockedServiceOrderKind::Refit => target.set_refit(()),
                DockedServiceOrderKind::Replacement {
                    subsystem_id,
                    reconditioned,
                } => {
                    let mut value = target.init_replacement();
                    value.set_subsystem_id(*subsystem_id);
                    value.set_reconditioned(*reconditioned);
                }
            }
        }
        Command::ReserveMarketLead {
            lead_id,
            expected_revision,
            quantity_millitons,
        } => {
            let mut value = builder.init_reserve_market_lead();
            value.set_lead_id(lead_id);
            value.set_expected_revision(expected_revision);
            value.set_quantity_millitons(quantity_millitons);
        }
        Command::ReleaseMarketReservation {
            lead_id,
            expected_revision,
        } => {
            let mut value = builder.init_release_market_reservation();
            value.set_lead_id(lead_id);
            value.set_expected_revision(expected_revision);
        }
        Command::ApplyTaskAction {
            task_id,
            expected_revision,
            action,
            ref explanation,
        } => {
            let mut value = builder.init_apply_task_action();
            value.set_task_id(task_id);
            value.set_expected_revision(expected_revision);
            value.set_action(match action {
                TaskActionKind::Cancel => crate::ct_rpc_capnp::TaskActionKind::Cancel,
                TaskActionKind::ReturnCustody => crate::ct_rpc_capnp::TaskActionKind::ReturnCustody,
                TaskActionKind::DefaultTask => crate::ct_rpc_capnp::TaskActionKind::DefaultTask,
                TaskActionKind::FileDispute => crate::ct_rpc_capnp::TaskActionKind::FileDispute,
                TaskActionKind::WithdrawClaim => crate::ct_rpc_capnp::TaskActionKind::WithdrawClaim,
                TaskActionKind::FileLossClaim => crate::ct_rpc_capnp::TaskActionKind::FileLossClaim,
            });
            value.set_explanation(explanation);
        }
        Command::SendPrivateMessage(ref message) => {
            let mut value = builder.init_send_private_message();
            value.set_recipient_kind(match message.recipient_kind {
                PrivateRecipientKind::System => crate::ct_rpc_capnp::PrivateRecipientKind::System,
                PrivateRecipientKind::Captain => crate::ct_rpc_capnp::PrivateRecipientKind::Captain,
            });
            value.set_destination_system_id(message.destination_system_id);
            let mut recipient = value.reborrow().init_recipient();
            recipient.set_bbs_id(message.recipient.bbs_id);
            recipient.set_player_id(message.recipient.player_id);
            value.set_encryption_key_id(message.encryption_key_id);
            value.set_ttl_weeks(message.ttl_weeks);
            value.set_subject(&message.subject);
            value.set_body(&message.body);
        }
        Command::PurchaseInsurance { kind, enabled } => {
            let mut value = builder.init_purchase_insurance();
            value.set_kind(match kind {
                InsuranceKind::DestinationAssistance => {
                    crate::ct_rpc_capnp::InsuranceKind::DestinationAssistance
                }
            });
            value.set_enabled(enabled);
        }
        Command::MisappropriateRestrictedCredits { amount } => {
            builder
                .init_misappropriate_restricted_credits()
                .set_amount(amount);
        }
        Command::GetFleet => builder.set_get_fleet(()),
        Command::SetActiveShip {
            expected_revision,
            ship_id,
        } => {
            let mut value = builder.init_set_active_ship();
            value.set_expected_revision(expected_revision);
            value.set_ship_id(ship_id);
        }
        Command::AssignShipCaptain {
            expected_revision,
            ship_id,
            person_id,
        } => {
            let mut value = builder.init_assign_ship_captain();
            value.set_expected_revision(expected_revision);
            value.set_ship_id(ship_id);
            value.set_person_id(person_id);
        }
        Command::TransferShipStores {
            expected_revision,
            from_ship_id,
            to_ship_id,
            kind,
            cargo_lot_id,
            ref item_id,
            quantity,
        } => {
            let mut value = builder.init_transfer_ship_stores();
            value.set_expected_revision(expected_revision);
            value.set_from_ship_id(from_ship_id);
            value.set_to_ship_id(to_ship_id);
            value.set_kind(match kind {
                StoreTransferKind::Cargo => crate::ct_rpc_capnp::StoreTransferKind::Cargo,
                StoreTransferKind::Fuel => crate::ct_rpc_capnp::StoreTransferKind::Fuel,
                StoreTransferKind::Ammunition => crate::ct_rpc_capnp::StoreTransferKind::Ammunition,
                StoreTransferKind::Provisions => crate::ct_rpc_capnp::StoreTransferKind::Provisions,
            });
            value.set_cargo_lot_id(cargo_lot_id);
            value.set_item_id(item_id);
            value.set_quantity(quantity);
        }
        Command::GetSystemRadio => builder.set_get_system_radio(()),
        Command::TransmitSystemRadio { ref body } => {
            builder.init_transmit_system_radio().set_body(body);
        }
        Command::PeekRadioReception { reception_id } => {
            builder
                .init_peek_radio_reception()
                .set_reception_id(reception_id);
        }
        Command::AcknowledgeRadioReception { reception_id } => {
            builder
                .init_acknowledge_radio_reception()
                .set_reception_id(reception_id);
        }
        Command::SetRadioMute { ref sender, muted } => {
            let mut value = builder.init_set_radio_mute();
            let mut target = value.reborrow().init_sender();
            target.set_bbs_id(sender.bbs_id);
            target.set_player_id(sender.player_id);
            value.set_muted(muted);
        }
        Command::GetBrowserAlertStatus => builder.set_get_browser_alert_status(()),
        Command::CreateBrowserAlertEnrollment => builder.set_create_browser_alert_enrollment(()),
        Command::RevokeAllBrowserAlerts => builder.set_revoke_all_browser_alerts(()),
        Command::GetOperationalDamageReport => builder.set_get_operational_damage_report(()),
        Command::AcknowledgeOperationalDamageReport { report_id } => builder
            .init_acknowledge_operational_damage_report()
            .set_report_id(report_id),
    }
    finish_message(&message)
}

fn decode_player_creation(
    creation: player_creation::Reader<'_>,
) -> Result<PlayerCreation, WireError> {
    let captain = decode_person(creation.get_captain()?)?;
    let ship_name = decode_name(creation.get_ship_name()?)?;
    let crew = creation.get_crew()?;
    if crew.len() as usize > MAX_INITIAL_CREW {
        return Err(WireError::TooManyCrew);
    }
    let crew = crew
        .iter()
        .map(|entry| {
            Ok::<InitialCrewDraft, WireError>(InitialCrewDraft {
                slot_id: entry.get_slot_id(),
                name: decode_name(entry.get_name()?)?,
                training_skill: decode_skill(entry.get_training_skill()?)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let refit_option_ids = creation.get_refit_option_ids()?.iter().collect();
    Ok(PlayerCreation {
        setup_revision: creation.get_setup_revision(),
        starting_offer_id: creation.get_starting_offer_id(),
        captain,
        ship_name,
        crew,
        refit_option_ids,
    })
}

fn decode_name(name: capnp::text::Reader<'_>) -> Result<String, WireError> {
    let name = name
        .to_str()
        .map_err(|_| WireError::InvalidText)?
        .to_owned();
    if name.is_empty() || name.len() > MAX_NAME_BYTES {
        return Err(WireError::InvalidName);
    }
    Ok(name)
}

fn decode_person(person: person_draft::Reader<'_>) -> Result<PersonDraft, WireError> {
    let characteristics = person.get_characteristics()?;
    let training = person.get_training()?;
    let skills = person
        .get_skills()?
        .iter()
        .map(|rating| {
            Ok(SkillRating {
                skill: decode_skill(rating.get_skill()?)?,
                level: rating.get_level(),
            })
        })
        .collect::<Result<Vec<_>, WireError>>()?;
    Ok(PersonDraft {
        name: decode_name(person.get_name()?)?,
        characteristics: Characteristics {
            strength: characteristics.get_strength(),
            dexterity: characteristics.get_dexterity(),
            endurance: characteristics.get_endurance(),
            intelligence: characteristics.get_intelligence(),
            education: characteristics.get_education(),
            charisma: characteristics.get_charisma(),
        },
        skills,
        training: SkillTraining {
            skill: decode_skill(training.get_skill()?)?,
            needed_weeks: training.get_needed_weeks(),
            current_weeks: training.get_current_weeks(),
        },
    })
}

fn set_player_creation(
    mut builder: player_creation::Builder<'_>,
    creation: &PlayerCreation,
) -> Result<(), WireError> {
    builder.set_setup_revision(creation.setup_revision);
    builder.set_starting_offer_id(creation.starting_offer_id);
    set_person(builder.reborrow().init_captain(), &creation.captain)?;
    builder.set_ship_name(&creation.ship_name);
    let crew_count = u32::try_from(creation.crew.len()).map_err(|_| WireError::TooManyCrew)?;
    let mut crew = builder.reborrow().init_crew(crew_count);
    for (index, entry) in creation.crew.iter().enumerate() {
        let mut item = crew.reborrow().get(index as u32);
        item.set_slot_id(entry.slot_id);
        item.set_name(&entry.name);
        item.set_training_skill(encode_skill(entry.training_skill));
    }
    let refit_count = u32::try_from(creation.refit_option_ids.len())
        .map_err(|_| WireError::Expected("fewer refit selections"))?;
    let mut refits = builder.reborrow().init_refit_option_ids(refit_count);
    for (index, option_id) in creation.refit_option_ids.iter().enumerate() {
        refits.set(index as u32, *option_id);
    }
    Ok(())
}

fn set_person(
    mut builder: person_draft::Builder<'_>,
    person: &PersonDraft,
) -> Result<(), WireError> {
    builder.set_name(&person.name);
    let mut characteristics = builder.reborrow().init_characteristics();
    characteristics.set_strength(person.characteristics.strength);
    characteristics.set_dexterity(person.characteristics.dexterity);
    characteristics.set_endurance(person.characteristics.endurance);
    characteristics.set_intelligence(person.characteristics.intelligence);
    characteristics.set_education(person.characteristics.education);
    characteristics.set_charisma(person.characteristics.charisma);
    let count =
        u32::try_from(person.skills.len()).map_err(|_| WireError::Expected("fewer skills"))?;
    let mut skills = builder.reborrow().init_skills(count);
    for (index, rating) in person.skills.iter().enumerate() {
        let mut item = skills.reborrow().get(index as u32);
        item.set_skill(encode_skill(rating.skill));
        item.set_level(rating.level);
    }
    let mut training = builder.init_training();
    training.set_skill(encode_skill(person.training.skill));
    training.set_needed_weeks(person.training.needed_weeks);
    training.set_current_weeks(person.training.current_weeks);
    Ok(())
}

fn encode_skill(skill: SkillId) -> crate::ct_rpc_capnp::SkillId {
    use crate::ct_rpc_capnp::SkillId as Wire;
    match skill {
        SkillId::Admin => Wire::Admin,
        SkillId::Advocate => Wire::Advocate,
        SkillId::Astrogation => Wire::Astrogation,
        SkillId::Broker => Wire::Broker,
        SkillId::Carouse => Wire::Carouse,
        SkillId::Communications => Wire::Communications,
        SkillId::Computer => Wire::Computer,
        SkillId::Electronics => Wire::Electronics,
        SkillId::EngineerJump => Wire::EngineerJump,
        SkillId::EngineerManeuver => Wire::EngineerManeuver,
        SkillId::EngineerPower => Wire::EngineerPower,
        SkillId::EngineerLifeSupport => Wire::EngineerLifeSupport,
        SkillId::Etiquette => Wire::Etiquette,
        SkillId::GunCombat => Wire::GunCombat,
        SkillId::GunnerTurrets => Wire::GunnerTurrets,
        SkillId::GunnerCapital => Wire::GunnerCapital,
        SkillId::GunnerScreens => Wire::GunnerScreens,
        SkillId::Investigate => Wire::Investigate,
        SkillId::JackOfAllTrades => Wire::JackOfAllTrades,
        SkillId::Leadership => Wire::Leadership,
        SkillId::Mechanic => Wire::Mechanic,
        SkillId::Medicine => Wire::Medicine,
        SkillId::Melee => Wire::Melee,
        SkillId::Persuade => Wire::Persuade,
        SkillId::PilotSpacecraft => Wire::PilotSpacecraft,
        SkillId::PilotSmallCraft => Wire::PilotSmallCraft,
        SkillId::Recon => Wire::Recon,
        SkillId::Stealth => Wire::Stealth,
        SkillId::Streetwise => Wire::Streetwise,
        SkillId::TacticsMilitary => Wire::TacticsMilitary,
        SkillId::TacticsNaval => Wire::TacticsNaval,
        SkillId::TradeCargomaster => Wire::TradeCargomaster,
        SkillId::VaccSuit => Wire::VaccSuit,
        SkillId::TradeProspector => Wire::TradeProspector,
    }
}

fn decode_skill(skill: crate::ct_rpc_capnp::SkillId) -> Result<SkillId, WireError> {
    use crate::ct_rpc_capnp::SkillId as Wire;
    Ok(match skill {
        Wire::Admin => SkillId::Admin,
        Wire::Advocate => SkillId::Advocate,
        Wire::Astrogation => SkillId::Astrogation,
        Wire::Broker => SkillId::Broker,
        Wire::Carouse => SkillId::Carouse,
        Wire::Communications => SkillId::Communications,
        Wire::Computer => SkillId::Computer,
        Wire::Electronics => SkillId::Electronics,
        Wire::EngineerJump => SkillId::EngineerJump,
        Wire::EngineerManeuver => SkillId::EngineerManeuver,
        Wire::EngineerPower => SkillId::EngineerPower,
        Wire::EngineerLifeSupport => SkillId::EngineerLifeSupport,
        Wire::Etiquette => SkillId::Etiquette,
        Wire::GunCombat => SkillId::GunCombat,
        Wire::GunnerTurrets => SkillId::GunnerTurrets,
        Wire::GunnerCapital => SkillId::GunnerCapital,
        Wire::GunnerScreens => SkillId::GunnerScreens,
        Wire::Investigate => SkillId::Investigate,
        Wire::JackOfAllTrades => SkillId::JackOfAllTrades,
        Wire::Leadership => SkillId::Leadership,
        Wire::Mechanic => SkillId::Mechanic,
        Wire::Medicine => SkillId::Medicine,
        Wire::Melee => SkillId::Melee,
        Wire::Persuade => SkillId::Persuade,
        Wire::PilotSpacecraft => SkillId::PilotSpacecraft,
        Wire::PilotSmallCraft => SkillId::PilotSmallCraft,
        Wire::Recon => SkillId::Recon,
        Wire::Stealth => SkillId::Stealth,
        Wire::Streetwise => SkillId::Streetwise,
        Wire::TacticsMilitary => SkillId::TacticsMilitary,
        Wire::TacticsNaval => SkillId::TacticsNaval,
        Wire::TradeCargomaster => SkillId::TradeCargomaster,
        Wire::VaccSuit => SkillId::VaccSuit,
        Wire::TradeProspector => SkillId::TradeProspector,
    })
}

fn set_skill_pool(mut builder: crate::ct_rpc_capnp::skill_pool::Builder<'_>, pool: SkillPool) {
    builder.set_level3(pool.level3);
    builder.set_level2(pool.level2);
    builder.set_level1(pool.level1);
    builder.set_level0(pool.level0);
}

fn set_captain_options(
    mut builder: crate::ct_rpc_capnp::captain_creation_options::Builder<'_>,
    options: &CaptainCreationOptions,
) -> Result<(), WireError> {
    builder.set_setup_revision(options.setup_revision);
    let mut point_buy = builder.reborrow().init_characteristic_point_buy();
    point_buy.set_minimum(options.characteristic_point_buy.minimum);
    point_buy.set_maximum(options.characteristic_point_buy.maximum);
    point_buy.set_neutral(options.characteristic_point_buy.neutral);
    point_buy.set_budget(options.characteristic_point_buy.budget);
    set_skill_pool(builder.reborrow().init_skill_pool(), options.skill_pool);
    let mut skills = builder
        .reborrow()
        .init_permitted_skills(SkillId::ALL.len() as u32);
    for (index, skill) in SkillId::ALL.iter().enumerate() {
        let mut definition = skills.reborrow().get(index as u32);
        definition.set_id(encode_skill(*skill));
        definition.set_name(skill.name());
    }
    set_person(builder.init_default_captain(), &options.default_captain)
}

fn set_offer_summary(
    mut builder: crate::ct_rpc_capnp::starting_ship_offer_summary::Builder<'_>,
    offer: &StartingShipOfferSummary,
) {
    use crate::ct_rpc_capnp::Career as WireCareer;
    builder.set_offer_id(offer.offer_id);
    builder.set_career(match offer.career {
        Career::Trader => WireCareer::Trader,
        Career::Privateer => WireCareer::Privateer,
        Career::Navy => WireCareer::Navy,
    });
    builder.set_package_name(&offer.package_name);
    builder.set_ship_catalog_id(offer.ship_catalog_id);
    builder.set_ship_name(&offer.ship_name);
    builder.set_role(&offer.role);
    builder.set_rationale(&offer.rationale);
    builder.set_displacement_tons(offer.displacement_tons);
    builder.set_jump_rating(offer.jump_rating);
    builder.set_thrust_g(offer.thrust_g);
    builder.set_cargo_tons(f64::from(offer.cargo_millitons) / 1000.0);
    builder.set_crew_count(offer.crew_count);
    builder.set_price_credits(offer.price_credits);
}

fn set_starting_ship_offers(
    mut builder: crate::ct_rpc_capnp::starting_ship_offers::Builder<'_>,
    offers: &StartingShipOffers,
) -> Result<(), WireError> {
    builder.set_setup_revision(offers.setup_revision);
    let mut origin = builder.reborrow().init_origin();
    origin.set_bbs_name(&offers.origin.bbs_name);
    origin.set_polity_name(&offers.origin.polity_name);
    origin.set_home_system_name(&offers.origin.home_system_name);
    origin.set_home_world_name(&offers.origin.home_world_name);
    origin.set_trade_combat(offers.origin.trade_combat);
    origin.set_chaos_order(offers.origin.chaos_order);
    origin.set_league_name(offers.origin.league_name.as_deref().unwrap_or(""));
    let count =
        u32::try_from(offers.offers.len()).map_err(|_| WireError::Expected("fewer offers"))?;
    let mut list = builder.init_offers(count);
    for (index, offer) in offers.offers.iter().enumerate() {
        set_offer_summary(list.reborrow().get(index as u32), offer);
    }
    Ok(())
}

fn set_starting_ship_options(
    mut builder: crate::ct_rpc_capnp::starting_ship_options::Builder<'_>,
    options: &StartingShipOptions,
) -> Result<(), WireError> {
    builder.set_setup_revision(options.setup_revision);
    set_offer_summary(builder.reborrow().init_offer(), &options.offer);
    let count = u32::try_from(options.description_paragraphs.len())
        .map_err(|_| WireError::Expected("fewer description paragraphs"))?;
    let mut paragraphs = builder.reborrow().init_description_paragraphs(count);
    for (index, paragraph) in options.description_paragraphs.iter().enumerate() {
        paragraphs.set(index as u32, paragraph);
    }
    let terms = &options.terms;
    let mut t = builder.reborrow().init_terms();
    t.set_terms_revision(terms.terms_revision);
    t.set_title(match terms.title {
        StartingTitleKind::OwnedWithLien => crate::ct_rpc_capnp::StartingTitleKind::OwnedWithLien,
        StartingTitleKind::SponsorOwned => crate::ct_rpc_capnp::StartingTitleKind::SponsorOwned,
        StartingTitleKind::InstitutionOwned => {
            crate::ct_rpc_capnp::StartingTitleKind::InstitutionOwned
        }
    });
    t.set_equity_credits(terms.equity_credits);
    t.set_principal_credits(terms.principal_credits);
    t.set_monthly_payment_credits(terms.monthly_payment_credits);
    t.set_liquid_reserve_credits(terms.liquid_reserve_credits);
    t.set_restricted_reserve_credits(terms.restricted_reserve_credits);
    t.set_monthly_compensation_credits(terms.monthly_compensation_credits);
    t.set_refit_credit_limit(terms.refit_credit_limit);
    t.set_refit_displacement_millitons(terms.refit_displacement_millitons);
    t.set_authority(&terms.authority);
    t.set_exit_terms(&terms.exit_terms);
    t.set_insurance(&terms.insurance);
    let group_count = u32::try_from(options.refit_groups.len())
        .map_err(|_| WireError::Expected("fewer refit groups"))?;
    let mut groups = builder.reborrow().init_refit_groups(group_count);
    for (index, group) in options.refit_groups.iter().enumerate() {
        let mut g = groups.reborrow().get(index as u32);
        g.set_group_id(group.group_id);
        g.set_name(&group.name);
        g.set_required(group.required);
        let option_count = u32::try_from(group.options.len())
            .map_err(|_| WireError::Expected("fewer refit options"))?;
        let mut entries = g.init_options(option_count);
        for (option_index, option) in group.options.iter().enumerate() {
            let mut o = entries.reborrow().get(option_index as u32);
            o.set_option_id(option.option_id);
            o.set_name(&option.name);
            o.set_description(&option.description);
            o.set_displacement_delta_millitons(option.displacement_delta_millitons);
            o.set_price_delta_credits(option.price_delta_credits);
        }
    }
    Ok(())
}

fn schema_crew_role_kind(role: &str) -> crate::ct_rpc_capnp::CrewRoleKind {
    use crate::ct_rpc_capnp::CrewRoleKind as Kind;
    match CrewRoleKind::from_slug(role) {
        CrewRoleKind::Command => Kind::Command,
        CrewRoleKind::Pilot => Kind::Pilot,
        CrewRoleKind::Navigator => Kind::Navigator,
        CrewRoleKind::Engineer => Kind::Engineer,
        CrewRoleKind::SensorsOperator => Kind::SensorsOperator,
        CrewRoleKind::ScreenOperator => Kind::ScreenOperator,
        CrewRoleKind::TurretGunner => Kind::TurretGunner,
        CrewRoleKind::BayGunner => Kind::BayGunner,
        CrewRoleKind::Gunner => Kind::Gunner,
        CrewRoleKind::Medic => Kind::Medic,
        CrewRoleKind::Marine => Kind::Marine,
        CrewRoleKind::FlightCrew => Kind::FlightCrew,
        CrewRoleKind::Steward => Kind::Steward,
        CrewRoleKind::Other => Kind::Other,
    }
}

fn set_starting_crew_plan(
    mut builder: crate::ct_rpc_capnp::starting_crew_plan::Builder<'_>,
    plan: &StartingCrewPlan,
) -> Result<(), WireError> {
    builder.set_setup_revision(plan.setup_revision);
    builder.set_starting_offer_id(plan.starting_offer_id);
    let count =
        u32::try_from(plan.slots.len()).map_err(|_| WireError::Expected("fewer crew slots"))?;
    let mut slots = builder.init_slots(count);
    for (index, slot) in plan.slots.iter().enumerate() {
        let mut item = slots.reborrow().get(index as u32);
        item.set_slot_id(slot.slot_id);
        item.set_role(&slot.role);
        item.set_role_kind(schema_crew_role_kind(&slot.role));
        item.set_represented_positions(slot.represented_positions);
        item.set_required(slot.required);
        set_skill_pool(item.reborrow().init_skill_pool(), slot.skill_pool);
        set_person(item.init_default_crew(), &slot.default_crew)?;
    }
    Ok(())
}

fn set_crew_management(
    mut builder: crate::ct_rpc_capnp::crew_management_snapshot::Builder<'_>,
    snapshot: &CrewManagementSnapshot,
) -> Result<(), WireError> {
    builder.set_ship_id(snapshot.ship_id);
    builder.set_ship_name(&snapshot.ship_name);
    builder.set_established_complement(snapshot.established_complement);
    let count = u32::try_from(snapshot.members.len())
        .map_err(|_| WireError::Expected("fewer crew members"))?;
    let mut members = builder.reborrow().init_members(count);
    for (index, member) in snapshot.members.iter().enumerate() {
        let mut item = members.reborrow().get(index as u32);
        item.set_person_id(member.person_id);
        item.set_slot_id(member.slot_id);
        item.set_role(&member.role);
        item.set_role_kind(schema_crew_role_kind(&member.role));
        item.set_represented_positions(member.represented_positions);
        item.set_captain(member.captain);
        item.set_condition(match member.condition {
            PersonCondition::Fit => crate::ct_rpc_capnp::PersonCondition::Fit,
            PersonCondition::Fatigued => crate::ct_rpc_capnp::PersonCondition::Fatigued,
            PersonCondition::Wounded => crate::ct_rpc_capnp::PersonCondition::Wounded,
            PersonCondition::Incapacitated => crate::ct_rpc_capnp::PersonCondition::Incapacitated,
            PersonCondition::Dead => crate::ct_rpc_capnp::PersonCondition::Dead,
        });
        item.set_injury_points(member.injury_points);
        item.set_fatigue_points(member.fatigue_points);
        item.set_unfed_days(member.unfed_days);
        item.set_available(member.available);
        item.set_current_strength(member.current_strength);
        item.set_current_dexterity(member.current_dexterity);
        item.set_current_endurance(member.current_endurance);
        item.set_service_kind(match member.service_kind {
            CrewServiceKind::OwnerCaptain => crate::ct_rpc_capnp::CrewServiceKind::OwnerCaptain,
            CrewServiceKind::Salaried => crate::ct_rpc_capnp::CrewServiceKind::Salaried,
            CrewServiceKind::PrizeShare => crate::ct_rpc_capnp::CrewServiceKind::PrizeShare,
            CrewServiceKind::Institutional => crate::ct_rpc_capnp::CrewServiceKind::Institutional,
        });
        item.set_monthly_salary_credits(member.monthly_salary_credits);
        item.set_arrears_credits(member.arrears_credits);
        item.set_prize_share_basis_points(member.prize_share_basis_points);
        item.set_morale(member.morale);
        item.set_loyalty(member.loyalty);
        item.set_risk_tolerance(member.risk_tolerance);
        item.set_availability(match member.availability {
            CrewAvailability::Active => crate::ct_rpc_capnp::CrewAvailability::Active,
            CrewAvailability::ShoreLeave => crate::ct_rpc_capnp::CrewAvailability::ShoreLeave,
            CrewAvailability::MedicalCare => crate::ct_rpc_capnp::CrewAvailability::MedicalCare,
            CrewAvailability::Detached => crate::ct_rpc_capnp::CrewAvailability::Detached,
            CrewAvailability::AwaitingRecall => {
                crate::ct_rpc_capnp::CrewAvailability::AwaitingRecall
            }
        });
        item.set_available_second(member.available_second);
        item.set_service_revision(member.service_revision);
        item.set_shore_location(&member.shore_location);
        item.set_location_kind(if member.availability == CrewAvailability::Active {
            crate::ct_rpc_capnp::CrewLocationKind::AboardShip
        } else {
            crate::ct_rpc_capnp::CrewLocationKind::ShoreFacility
        });
        let assignment_count = u32::try_from(member.assigned_slot_ids.len())
            .map_err(|_| WireError::Expected("fewer crew assignments"))?;
        let mut assignments = item.reborrow().init_assigned_slot_ids(assignment_count);
        for (assignment_index, slot_id) in member.assigned_slot_ids.iter().enumerate() {
            assignments.set(assignment_index as u32, *slot_id);
        }
        set_person(item.init_person(), &member.person)?;
    }
    let role_count =
        u32::try_from(snapshot.roles.len()).map_err(|_| WireError::Expected("fewer crew roles"))?;
    let mut roles = builder.init_roles(role_count);
    for (index, role) in snapshot.roles.iter().enumerate() {
        let mut item = roles.reborrow().get(index as u32);
        item.set_slot_id(role.slot_id);
        item.set_role(&role.role);
        item.set_role_kind(schema_crew_role_kind(&role.role));
        item.set_represented_positions(role.represented_positions);
    }
    Ok(())
}

fn set_ship_status(
    mut builder: crate::ct_rpc_capnp::ship_status_snapshot::Builder<'_>,
    snapshot: &ShipStatusSnapshot,
) -> Result<(), WireError> {
    builder.set_ship_revision(snapshot.ship_revision);
    builder.set_ship_id(snapshot.ship_id);
    builder.set_ship_name(&snapshot.ship_name);
    builder.set_catalog_id(snapshot.catalog_id);
    builder.set_catalog_revision(snapshot.catalog_revision);
    builder.set_system_id(snapshot.system_id);
    builder.set_current_game_second(snapshot.current_game_second);
    builder.set_displacement_millitons(snapshot.displacement_millitons);
    builder.set_jump_rating(snapshot.jump_rating);
    builder.set_thrust_g(snapshot.thrust_g);
    builder.set_fuel_capacity_millitons(snapshot.fuel_capacity_millitons);
    builder.set_current_fuel_millitons(snapshot.current_fuel_millitons);
    builder.set_jump_fuel_millitons(snapshot.jump_fuel_millitons);
    builder.set_cargo_capacity_millitons(snapshot.cargo_capacity_millitons);
    builder.set_monthly_maintenance_credits(snapshot.monthly_maintenance_credits);
    builder.set_next_maintenance_second(snapshot.next_maintenance_second);
    builder.set_maintenance_paid_through_second(snapshot.maintenance_paid_through_second);
    builder.set_maintenance_arrears_credits(snapshot.maintenance_arrears_credits);
    builder.set_completed_maintenance_cycles(snapshot.completed_maintenance_cycles);
    builder.set_consecutive_missed_maintenance(snapshot.consecutive_missed_maintenance);
    builder.set_commissioned_second(snapshot.commissioned_second);
    builder.set_transit_count(snapshot.transit_count);
    builder.set_warranty_expires_second(snapshot.warranty_expires_second);
    builder.set_warranty_transit_limit(snapshot.warranty_transit_limit);
    builder.set_warranty_repairs(snapshot.warranty_repairs);
    builder.set_last_refit_second(snapshot.last_refit_second);
    builder.set_completed_refits(snapshot.completed_refits);
    let mut activity = builder.reborrow().init_active_activity();
    if let Some(source) = &snapshot.active_activity {
        activity.set_activity_id(source.activity_id);
        activity.set_started_second(source.started_second);
        activity.set_due_second(source.due_second);
        activity.set_cost_credits(source.cost_credits);
        activity.set_has_source_body(source.source_body_id.is_some());
        activity.set_source_body_id(source.source_body_id.unwrap_or(0));
        activity.set_refine_collected(source.refine_collected);
        match source.kind {
            ShipActivityKind::Construction => activity.set_construction(()),
            ShipActivityKind::Refit => activity.set_refit(()),
            ShipActivityKind::Refurbishment { component_count } => {
                activity.set_refurbishment(component_count)
            }
            ShipActivityKind::ProperRepair { subsystem_id } => {
                activity.set_proper_repair(subsystem_id)
            }
            ShipActivityKind::GasGiantSkim { quantity_millitons } => {
                activity.set_gas_giant_skim(quantity_millitons)
            }
            ShipActivityKind::WildernessWater { quantity_millitons } => {
                activity.set_wilderness_water(quantity_millitons)
            }
            ShipActivityKind::FuelProcessing { quantity_millitons } => {
                activity.set_fuel_processing(quantity_millitons)
            }
            ShipActivityKind::EscortDuty { opportunity_id } => {
                activity.set_escort_duty(opportunity_id)
            }
            ShipActivityKind::FieldRecovery { subsystem_id } => {
                activity.set_field_recovery(subsystem_id)
            }
        }
    } else {
        activity.set_none(());
    }
    builder.set_unrefined_fuel_millitons(snapshot.unrefined_fuel_millitons);
    builder.set_warranty_voided(snapshot.warranty_voided);
    builder.set_monthly_life_support_credits(snapshot.monthly_life_support_credits);
    builder.set_clock_rate_game_seconds(crate::clock::GAME_SECONDS_PER_RATE_PERIOD);
    builder.set_clock_rate_real_seconds(crate::clock::RATE_PERIOD.as_secs());
    builder.set_recovery_status(&snapshot.recovery_status);
    let ammunition_count = u32::try_from(snapshot.ammunition.len())
        .map_err(|_| WireError::Expected("fewer ammunition lots"))?;
    let mut ammunition = builder.reborrow().init_ammunition(ammunition_count);
    for (index, lot) in snapshot.ammunition.iter().enumerate() {
        set_ammunition_status(ammunition.reborrow().get(index as u32), lot);
    }
    let mut provisions = builder.reborrow().init_provisions();
    provisions.set_person_days_remaining(snapshot.provisions.person_days_remaining);
    provisions.set_capacity_person_days(snapshot.provisions.capacity_person_days);
    let symptom_count = u32::try_from(snapshot.manifested_symptoms.len())
        .map_err(|_| WireError::Expected("fewer ship symptoms"))?;
    let mut symptoms = builder.reborrow().init_manifested_symptoms(symptom_count);
    for (index, symptom) in snapshot.manifested_symptoms.iter().enumerate() {
        symptoms.set(index as u32, symptom);
    }
    let count = u32::try_from(snapshot.subsystems.len())
        .map_err(|_| WireError::Expected("fewer ship subsystems"))?;
    let mut subsystems = builder.init_subsystems(count);
    for (index, subsystem) in snapshot.subsystems.iter().enumerate() {
        let mut item = subsystems.reborrow().get(index as u32);
        item.set_subsystem_id(subsystem.subsystem_id);
        item.set_kind(encode_ship_subsystem_kind(subsystem.kind));
        item.set_label(&subsystem.label);
        item.set_maximum_hits(subsystem.maximum_hits);
        item.set_sustained_hits(subsystem.sustained_hits);
        item.set_battlefield_repair_hits(subsystem.battlefield_repair_hits);
        item.set_effective_hits(subsystem.effective_hits);
        item.set_operational_effect(&subsystem.operational_effect);
        item.set_last_proper_repair_second(subsystem.last_proper_repair_second);
        item.set_installed_second(subsystem.installed_second);
        item.set_last_refit_second(subsystem.last_refit_second);
        item.set_calendar_age_months(subsystem.calendar_age_months);
        item.set_operating_seconds(subsystem.operating_seconds);
        item.set_duty_cycles(subsystem.duty_cycles);
        item.set_skimming_cycles(subsystem.skimming_cycles);
        item.set_neglect_damage_hits(subsystem.neglect_damage_hits);
        item.set_displacement_millitons(subsystem.displacement_millitons);
        item.set_replacement_price_credits(subsystem.replacement_price_credits);
        item.set_installation_generation(subsystem.installation_generation);
        item.set_reconditioned(subsystem.reconditioned);
    }
    Ok(())
}

fn set_ammunition_status(
    mut builder: crate::ct_rpc_capnp::ship_ammunition_status::Builder<'_>,
    lot: &ShipAmmunitionStatus,
) {
    builder.set_ammunition_id(&lot.ammunition_id);
    builder.set_remaining(lot.remaining);
    builder.set_capacity(lot.capacity);
    builder.set_pack_units(lot.pack_units);
    builder.set_price_per_pack_credits(lot.price_per_pack_credits);
}

fn schema_docked_fuel_kind(
    kind: DockedFuelServiceKind,
) -> crate::ct_rpc_capnp::DockedFuelServiceKind {
    match kind {
        DockedFuelServiceKind::Refined => crate::ct_rpc_capnp::DockedFuelServiceKind::Refined,
        DockedFuelServiceKind::Unrefined => crate::ct_rpc_capnp::DockedFuelServiceKind::Unrefined,
        DockedFuelServiceKind::GasGiant => crate::ct_rpc_capnp::DockedFuelServiceKind::GasGiant,
        DockedFuelServiceKind::WildernessWater => {
            crate::ct_rpc_capnp::DockedFuelServiceKind::WildernessWater
        }
    }
}

fn schema_fuel_source_body_kind(
    kind: FuelSourceBodyKind,
) -> crate::ct_rpc_capnp::FuelSourceBodyKind {
    match kind {
        FuelSourceBodyKind::NotApplicable => crate::ct_rpc_capnp::FuelSourceBodyKind::NotApplicable,
        FuelSourceBodyKind::GasGiant => crate::ct_rpc_capnp::FuelSourceBodyKind::GasGiant,
        FuelSourceBodyKind::Planet => crate::ct_rpc_capnp::FuelSourceBodyKind::Planet,
        FuelSourceBodyKind::Moon => crate::ct_rpc_capnp::FuelSourceBodyKind::Moon,
        FuelSourceBodyKind::IcyBelt => crate::ct_rpc_capnp::FuelSourceBodyKind::IcyBelt,
    }
}

fn schema_fuel_access_kind(kind: FuelAccessKind) -> crate::ct_rpc_capnp::FuelAccessKind {
    match kind {
        FuelAccessKind::PortSale => crate::ct_rpc_capnp::FuelAccessKind::PortSale,
        FuelAccessKind::RoutineWilderness => crate::ct_rpc_capnp::FuelAccessKind::RoutineWilderness,
    }
}

fn set_docked_services(
    mut builder: crate::ct_rpc_capnp::docked_services::Builder<'_>,
    snapshot: &DockedServices,
) -> Result<(), WireError> {
    builder.set_ship_revision(snapshot.ship_revision);
    builder.set_current_game_second(snapshot.current_game_second);
    let fuel_count = u32::try_from(snapshot.fuel.len())
        .map_err(|_| WireError::Expected("fewer fuel services"))?;
    let mut fuel = builder.reborrow().init_fuel(fuel_count);
    for (index, offer) in snapshot.fuel.iter().enumerate() {
        let mut item = fuel.reborrow().get(index as u32);
        item.set_kind(schema_docked_fuel_kind(offer.kind));
        item.set_label(&offer.label);
        item.set_has_source_body(offer.source_body_id.is_some());
        item.set_source_body_id(offer.source_body_id.unwrap_or(0));
        item.set_available(offer.available);
        item.set_unavailable_reason(&offer.unavailable_reason);
        item.set_price_per_ton_credits(offer.price_per_ton_credits);
        item.set_maximum_millitons(offer.maximum_millitons);
        item.set_service_seconds(offer.service_seconds);
        item.set_body_kind(schema_fuel_source_body_kind(offer.body_kind));
        item.set_access_kind(schema_fuel_access_kind(offer.access_kind));
        item.set_can_refine(offer.can_refine);
        item.set_round_trip_distance_micro_au(offer.round_trip_distance_micro_au);
        item.set_round_trip_seconds(offer.round_trip_seconds);
    }
    let ammunition_count = u32::try_from(snapshot.ammunition.len())
        .map_err(|_| WireError::Expected("fewer ammunition lots"))?;
    let mut ammunition = builder.reborrow().init_ammunition(ammunition_count);
    for (index, lot) in snapshot.ammunition.iter().enumerate() {
        set_ammunition_status(ammunition.reborrow().get(index as u32), lot);
    }
    let mut provisions = builder.reborrow().init_provisions();
    provisions.set_person_days_remaining(snapshot.provisions.person_days_remaining);
    provisions.set_capacity_person_days(snapshot.provisions.capacity_person_days);
    builder.set_provision_package_person_days(snapshot.provision_package_person_days);
    builder.set_provision_package_price_credits(snapshot.provision_package_price_credits);
    builder.set_provisions_available(snapshot.provisions_available);
    builder.set_ammunition_available(snapshot.ammunition_available);
    let repair_count = u32::try_from(snapshot.repair.len())
        .map_err(|_| WireError::Expected("fewer repair services"))?;
    let mut repair = builder.reborrow().init_repair(repair_count);
    for (index, offer) in snapshot.repair.iter().enumerate() {
        let mut item = repair.reborrow().get(index as u32);
        item.set_subsystem_id(offer.subsystem_id);
        item.set_label(&offer.label);
        item.set_available(offer.available);
        item.set_unavailable_reason(&offer.unavailable_reason);
        item.set_cost_credits(offer.cost_credits);
        item.set_service_seconds(offer.service_seconds);
        item.set_replacement(offer.replacement);
        item.set_reconditioned(offer.reconditioned);
    }
    builder.set_refit_available(snapshot.refit_available);
    builder.set_refit_unavailable_reason(&snapshot.refit_unavailable_reason);
    builder.set_refit_cost_credits(snapshot.refit_cost_credits);
    builder.set_refit_service_seconds(snapshot.refit_service_seconds);
    Ok(())
}

fn set_docked_service_receipt(
    mut builder: crate::ct_rpc_capnp::docked_service_receipt::Builder<'_>,
    receipt: &DockedServiceReceipt,
) -> Result<(), WireError> {
    set_ship_status(builder.reborrow().init_ship_status(), &receipt.ship_status)?;
    let mut builder = builder.init_detail();
    match &receipt.detail {
        DockedServiceReceiptDetail::Generic => builder.set_generic(()),
        DockedServiceReceiptDetail::FuelPurchase(source) => {
            let mut target = builder.init_fuel_purchase();
            target.set_kind(schema_docked_fuel_kind(source.kind));
            target.set_quantity_millitons(source.quantity_millitons);
            target.set_current_fuel_millitons(source.current_fuel_millitons);
            target.set_unrefined_fuel_millitons(source.unrefined_fuel_millitons);
            target.set_fuel_capacity_millitons(source.fuel_capacity_millitons);
            target.set_cost_credits(source.cost_credits);
            target.set_restricted_payment_credits(source.restricted_payment_credits);
            target.set_liquid_payment_credits(source.liquid_payment_credits);
            target.set_restricted_balance_credits(source.restricted_balance_credits);
            target.set_liquid_balance_credits(source.liquid_balance_credits);
        }
        DockedServiceReceiptDetail::ProvisionPurchase(source) => {
            let mut target = builder.init_provision_purchase();
            target.set_packages(source.packages);
            target.set_person_days_loaded(source.person_days_loaded);
            target.set_person_days_remaining(source.person_days_remaining);
            target.set_capacity_person_days(source.capacity_person_days);
            target.set_cost_credits(source.cost_credits);
            target.set_restricted_payment_credits(source.restricted_payment_credits);
            target.set_liquid_payment_credits(source.liquid_payment_credits);
            target.set_restricted_balance_credits(source.restricted_balance_credits);
            target.set_liquid_balance_credits(source.liquid_balance_credits);
        }
    }
    Ok(())
}

fn set_docked_snapshot(
    mut builder: crate::ct_rpc_capnp::docked_snapshot::Builder<'_>,
    snapshot: &DockedSnapshot,
) {
    builder.set_ship_id(snapshot.ship_id);
    builder.set_ship_name(&snapshot.ship_name);
    builder.set_system_id(snapshot.system_id);
    builder.set_system_name(&snapshot.system_name);
    builder.set_world_id(snapshot.world_id);
    builder.set_world_name(&snapshot.world_name);
    builder.set_facility_id(snapshot.facility_id);
    builder.set_facility_name(&snapshot.facility_name);
    builder.set_starport(&snapshot.starport);
    builder.set_tech_level(snapshot.tech_level);
    builder.set_population(snapshot.population);
    builder.set_law_level(snapshot.law_level);
    builder.set_arrived_second(snapshot.arrived_second);
    builder.set_current_game_second(snapshot.current_game_second);
    builder.set_credits(snapshot.credits);
    builder.set_debt_credits(snapshot.debt_credits);
    builder.set_fuel_millitons(snapshot.fuel_millitons);
    builder.set_fuel_capacity_millitons(snapshot.fuel_capacity_millitons);
    builder.set_refined_fuel_price_per_ton(snapshot.refined_fuel_price_per_ton);
    builder.set_cargo_used_millitons(snapshot.cargo_used_millitons);
    builder.set_cargo_capacity_millitons(snapshot.cargo_capacity_millitons);
    builder.set_unrefined_fuel_millitons(snapshot.unrefined_fuel_millitons);
    builder.set_unrefined_fuel_price_per_ton(snapshot.unrefined_fuel_price_per_ton);
    builder.set_accrued_berth_fee_credits(snapshot.accrued_berth_fee_credits);
    builder.set_export_tariff_due_credits(snapshot.export_tariff_due_credits);
    builder.set_restricted_credits(snapshot.restricted_credits);
    builder.set_facility_revision(snapshot.facility_revision);
    builder.set_personnel_available(snapshot.personnel_available);
    builder.set_banking_available(snapshot.banking_available);
    builder.set_authority_available(snapshot.authority_available);
    builder.set_medical_level(snapshot.medical_level);
    builder.set_clearance_required(snapshot.clearance_required);
}

fn set_known_destinations(
    mut builder: crate::ct_rpc_capnp::known_destinations::Builder<'_>,
    snapshot: &KnownDestinations,
) -> Result<(), WireError> {
    builder.set_current_system_id(snapshot.current_system_id);
    builder.set_jump_rating(snapshot.jump_rating);
    let count = u32::try_from(snapshot.systems.len())
        .map_err(|_| WireError::Expected("fewer known systems"))?;
    let mut systems = builder.reborrow().init_systems(count);
    for (index, system) in snapshot.systems.iter().enumerate() {
        let mut item = systems.reborrow().get(index as u32);
        item.set_system_id(system.system_id);
        item.set_system_name(&system.system_name);
        item.set_world_name(&system.world_name);
        item.set_distance_parsecs(system.distance_milliparsecs as f64 / 1_000.0);
        item.set_within_jump_rating(system.within_jump_rating);
        item.set_starport(&system.starport);
        item.set_population(system.population);
        item.set_tech_level(system.tech_level);
        item.set_observed_second(system.observed_second);
        item.set_source(&system.source);
        let [coreward, spinward, north] = system.position.parsecs();
        item.set_coreward_parsecs(coreward);
        item.set_spinward_parsecs(spinward);
        item.set_north_parsecs(north);
        item.set_remote_candidate(system.remote_candidate);
        item.set_knowledge_source(match system.knowledge_source {
            SystemKnowledgeSource::PublishedRecords => {
                crate::ct_rpc_capnp::SystemKnowledgeSource::PublishedRecords
            }
            SystemKnowledgeSource::CarriedRecords => {
                crate::ct_rpc_capnp::SystemKnowledgeSource::CarriedRecords
            }
            SystemKnowledgeSource::PrivateObservation => {
                crate::ct_rpc_capnp::SystemKnowledgeSource::PrivateObservation
            }
            SystemKnowledgeSource::PublicDispatch => {
                crate::ct_rpc_capnp::SystemKnowledgeSource::PublicDispatch
            }
            SystemKnowledgeSource::DirectDispatch => {
                crate::ct_rpc_capnp::SystemKnowledgeSource::DirectDispatch
            }
            SystemKnowledgeSource::Withheld => crate::ct_rpc_capnp::SystemKnowledgeSource::Withheld,
            SystemKnowledgeSource::SecretChart => {
                crate::ct_rpc_capnp::SystemKnowledgeSource::SecretChart
            }
        });
        item.set_gas_giant_count(system.gas_giant_count);
        if let Some(affiliation) = &system.affiliation {
            let mut wire = item.reborrow().init_affiliation();
            wire.set_polity_name(&affiliation.polity_name);
            wire.set_bbs_name(&affiliation.bbs_name);
            wire.set_league_name(affiliation.league_name.as_deref().unwrap_or(""));
        }
        let target_count = u32::try_from(system.navigation_targets.len())
            .map_err(|_| WireError::Expected("fewer in-system navigation targets"))?;
        let mut targets = item.reborrow().init_navigation_targets(target_count);
        for (target_index, target) in system.navigation_targets.iter().enumerate() {
            let mut wire = targets.reborrow().get(target_index as u32);
            wire.set_body_id(target.body_id);
            wire.set_name(&target.name);
            wire.set_kind(match target.kind {
                InSystemNavigationTargetKind::RockyBody => {
                    crate::ct_rpc_capnp::InSystemNavigationTargetKind::RockyBody
                }
                InSystemNavigationTargetKind::GasGiant => {
                    crate::ct_rpc_capnp::InSystemNavigationTargetKind::GasGiant
                }
                InSystemNavigationTargetKind::PlanetoidBelt => {
                    crate::ct_rpc_capnp::InSystemNavigationTargetKind::PlanetoidBelt
                }
            });
            wire.set_primary_world(target.primary_world);
        }
    }
    let count = u32::try_from(snapshot.belts.len())
        .map_err(|_| WireError::Expected("fewer known belts"))?;
    let mut belts = builder.init_belts(count);
    for (index, belt) in snapshot.belts.iter().enumerate() {
        let mut item = belts.reborrow().get(index as u32);
        item.set_system_id(belt.system_id);
        item.set_body_id(belt.body_id);
        item.set_name(&belt.name);
        item.set_icy(belt.icy);
        item.set_carbonaceous_percent(belt.carbonaceous_percent);
        item.set_silicate_or_rock_percent(belt.silicate_or_rock_percent);
        item.set_metal_or_water_ice_percent(belt.metal_or_water_ice_percent);
        item.set_hydrocarbon_percent(belt.hydrocarbon_percent);
    }
    Ok(())
}

fn set_course_plan(
    mut builder: crate::ct_rpc_capnp::course_plan::Builder<'_>,
    plan: &CoursePlan,
) -> Result<(), WireError> {
    builder.set_available(plan.available);
    builder.set_elapsed_seconds(plan.elapsed_seconds);
    builder.set_fuel_cost_credits(plan.fuel_cost_credits);
    builder.set_total_milliparsecs(plan.total_milliparsecs);
    let count = u32::try_from(plan.waypoints.len())
        .map_err(|_| WireError::Expected("fewer course waypoints"))?;
    let mut waypoints = builder.init_waypoints(count);
    for (index, waypoint) in plan.waypoints.iter().enumerate() {
        let mut item = waypoints.reborrow().get(index as u32);
        item.set_system_id(waypoint.system_id);
        item.set_system_name(&waypoint.system_name);
        item.set_world_name(&waypoint.world_name);
        item.set_fuel_source(match waypoint.fuel_source {
            CourseFuelSource::None => crate::ct_rpc_capnp::CourseFuelSource::None,
            CourseFuelSource::Carried => crate::ct_rpc_capnp::CourseFuelSource::Carried,
            CourseFuelSource::RefinedPort => crate::ct_rpc_capnp::CourseFuelSource::RefinedPort,
            CourseFuelSource::FrontierSkimming => {
                crate::ct_rpc_capnp::CourseFuelSource::FrontierSkimming
            }
            CourseFuelSource::UnrefinedPort => crate::ct_rpc_capnp::CourseFuelSource::UnrefinedPort,
        });
        item.set_next_leg_milliparsecs(waypoint.next_leg_milliparsecs);
    }
    Ok(())
}

fn set_course_plot(
    mut builder: crate::ct_rpc_capnp::course_plot::Builder<'_>,
    plot: &CoursePlot,
) -> Result<(), WireError> {
    builder.set_origin_system_id(plot.origin_system_id);
    builder.set_destination_system_id(plot.destination_system_id);
    builder.set_jump_rating(plot.jump_rating);
    builder.set_current_game_second(plot.current_game_second);
    builder.set_clock_rate_game_seconds(crate::clock::GAME_SECONDS_PER_RATE_PERIOD);
    builder.set_clock_rate_real_seconds(crate::clock::RATE_PERIOD.as_secs());
    set_course_plan(builder.reborrow().init_fastest(), &plot.fastest)?;
    set_course_plan(builder.init_cheapest(), &plot.cheapest)?;
    Ok(())
}

fn schema_task_kind(value: TaskKind) -> crate::ct_rpc_capnp::TaskKind {
    use crate::ct_rpc_capnp::TaskKind as Wire;
    match value {
        TaskKind::Freight => Wire::Freight,
        TaskKind::Passenger => Wire::Passenger,
        TaskKind::PurchaseOrder => Wire::PurchaseOrder,
        TaskKind::ForwardSale => Wire::ForwardSale,
        TaskKind::SupplyCommitment => Wire::SupplyCommitment,
        TaskKind::Charter => Wire::Charter,
        TaskKind::Courier => Wire::Courier,
        TaskKind::DiscoveryBounty => Wire::DiscoveryBounty,
        TaskKind::CombatBounty => Wire::CombatBounty,
    }
}

fn schema_task_state(value: TaskState) -> crate::ct_rpc_capnp::TaskState {
    use crate::ct_rpc_capnp::TaskState as Wire;
    match value {
        TaskState::ClaimPending => Wire::ClaimPending,
        TaskState::Accepted => Wire::Accepted,
        TaskState::Sourcing => Wire::Sourcing,
        TaskState::Loading => Wire::Loading,
        TaskState::InTransit => Wire::InTransit,
        TaskState::AwaitingSettlement => Wire::AwaitingSettlement,
        TaskState::Completed => Wire::Completed,
        TaskState::Expired => Wire::Expired,
        TaskState::Cancelled => Wire::Cancelled,
        TaskState::Defaulted => Wire::Defaulted,
        TaskState::Disputed => Wire::Disputed,
        TaskState::LossDocumented => Wire::LossDocumented,
    }
}

fn set_task_offer(mut builder: crate::ct_rpc_capnp::task_offer::Builder<'_>, offer: &TaskOffer) {
    builder.set_offer_id(offer.offer_id);
    builder.set_revision(offer.revision);
    builder.set_kind(schema_task_kind(offer.kind));
    builder.set_title(&offer.title);
    builder.set_origin_system_id(offer.origin_system_id);
    builder.set_destination_system_id(offer.destination_system_id);
    builder.set_commodity_id(offer.commodity_id);
    builder.set_quantity_millitons(offer.quantity_millitons);
    builder.set_passenger_count(offer.passenger_count);
    builder.set_payment_credits(offer.payment_credits);
    builder.set_collateral_credits(offer.collateral_credits);
    builder.set_expires_second(offer.expires_second);
    builder.set_delivery_deadline_second(offer.delivery_deadline_second);
    builder.set_legal(offer.legal);
    builder.set_partial_delivery_allowed(offer.partial_delivery_allowed);
    builder.set_failure_penalty_credits(offer.failure_penalty_credits);
    builder.set_recurrence_seconds(offer.recurrence_seconds);
    builder.set_performance_count(offer.performance_count);
    builder.set_passenger_class(match offer.passenger_class {
        PassengerClass::None => crate::ct_rpc_capnp::PassengerClass::None,
        PassengerClass::High => crate::ct_rpc_capnp::PassengerClass::High,
        PassengerClass::Middle => crate::ct_rpc_capnp::PassengerClass::Middle,
        PassengerClass::Steerage => crate::ct_rpc_capnp::PassengerClass::Steerage,
        PassengerClass::Low => crate::ct_rpc_capnp::PassengerClass::Low,
        PassengerClass::Charter => crate::ct_rpc_capnp::PassengerClass::Charter,
        PassengerClass::Courier => crate::ct_rpc_capnp::PassengerClass::Courier,
    });
    builder.set_late_deduction_per_day_credits(offer.late_deduction_per_day_credits);
    builder.set_non_delivery_liability_credits(offer.non_delivery_liability_credits);
    builder.set_passenger_grace_seconds(offer.passenger_grace_seconds);
    builder.set_declared_value_credits(offer.declared_value_credits);
    let reason_count = u32::try_from(offer.unavailable_reasons.len())
        .expect("fewer task-offer unavailability reasons");
    let mut reasons = builder.init_unavailable_reasons(reason_count);
    for (index, reason) in offer.unavailable_reasons.iter().enumerate() {
        reasons.set(index as u32, reason);
    }
}

fn set_task_record(mut builder: crate::ct_rpc_capnp::task_record::Builder<'_>, task: &TaskRecord) {
    builder.set_task_id(task.task_id);
    set_task_offer(builder.reborrow().init_offer(), &task.offer);
    builder.set_state(schema_task_state(task.state));
    builder.set_accepted_second(task.accepted_second);
    builder.set_delivered_quantity_millitons(task.delivered_quantity_millitons);
    builder.set_reserved_cargo_millitons(task.reserved_cargo_millitons);
    builder.set_reserved_passenger_count(task.reserved_passenger_count);
    builder.set_reserved_credits(task.reserved_credits);
    builder.set_status_text(&task.status_text);
    builder.set_performances_completed(task.performances_completed);
    builder.set_revision(task.revision);
    builder.set_claim_message_id(task.claim_message_id);
    builder.set_result_message_id(task.result_message_id);
    builder.set_known_result(task.known_result);
    builder.set_loaded_second(task.loaded_second);
    builder.set_settled_second(task.settled_second);
    builder.set_insurance_claim_id(task.insurance_claim_id);
    builder.set_dispute_message_id(task.dispute_message_id);
    builder.set_dispute_effect(task.dispute_effect);
    builder.set_adjudication_message_id(task.adjudication_message_id);
    builder.set_performing_ship_id(task.performing_ship_id);
    builder.set_piracy_encounter_id(task.piracy_encounter_id);
    builder.set_piracy_incident_second(task.piracy_incident_second);
    builder.set_piracy_contact_id(task.piracy_contact_id);
    builder.set_piracy_threat(match task.piracy_threat {
        EncounterThreat::Unknown => crate::ct_rpc_capnp::EncounterThreat::Unknown,
        EncounterThreat::Favorable => crate::ct_rpc_capnp::EncounterThreat::Favorable,
        EncounterThreat::Comparable => crate::ct_rpc_capnp::EncounterThreat::Comparable,
        EncounterThreat::Dangerous => crate::ct_rpc_capnp::EncounterThreat::Dangerous,
        EncounterThreat::Overwhelming => crate::ct_rpc_capnp::EncounterThreat::Overwhelming,
    });
    builder.set_piracy_posture(encode_encounter_posture(task.piracy_posture));
    builder.set_piracy_quantity_millitons(task.piracy_quantity_millitons);
    builder.set_loss_claim_deadline_second(task.loss_claim_deadline_second);
    builder.set_loss_claim_effect(task.loss_claim_effect);
}

fn set_carriage(
    mut builder: crate::ct_rpc_capnp::carriage_declaration::Builder<'_>,
    declaration: CarriageDeclaration,
) {
    builder.set_plan_revision(declaration.plan_revision);
    builder.set_destination_system_id(declaration.destination_system_id);
    builder.set_freight_capacity_millitons(declaration.freight_capacity_millitons);
    builder.set_high_berths(declaration.high_berths);
    builder.set_middle_berths(declaration.middle_berths);
    builder.set_steerage_berths(declaration.steerage_berths);
    builder.set_low_berths(declaration.low_berths);
    builder.set_accept_electronic_mail(declaration.accept_electronic_mail);
}

fn set_task_ledger(
    mut builder: crate::ct_rpc_capnp::task_ledger::Builder<'_>,
    ledger: &TaskLedger,
) -> Result<(), WireError> {
    builder.set_current_second(ledger.current_second);
    builder.set_available_credits(ledger.available_credits);
    builder.set_reserved_credits(ledger.reserved_credits);
    builder.set_reserved_cargo_millitons(ledger.reserved_cargo_millitons);
    builder.set_reserved_passenger_count(ledger.reserved_passenger_count);
    let task_count =
        u32::try_from(ledger.tasks.len()).map_err(|_| WireError::Expected("fewer tasks"))?;
    let mut tasks = builder.reborrow().init_tasks(task_count);
    for (index, task) in ledger.tasks.iter().enumerate() {
        set_task_record(tasks.reborrow().get(index as u32), task);
    }
    let offer_count = u32::try_from(ledger.local_offers.len())
        .map_err(|_| WireError::Expected("fewer task offers"))?;
    let mut offers = builder.reborrow().init_local_offers(offer_count);
    for (index, offer) in ledger.local_offers.iter().enumerate() {
        set_task_offer(offers.reborrow().get(index as u32), offer);
    }
    let assessment_count = u32::try_from(ledger.route_assessments.len())
        .map_err(|_| WireError::Expected("fewer task route assessments"))?;
    let mut assessments = builder.reborrow().init_route_assessments(assessment_count);
    for (index, assessment) in ledger.route_assessments.iter().enumerate() {
        let mut item = assessments.reborrow().get(index as u32);
        item.set_offer_id(assessment.offer_id);
        item.set_pickup_available(assessment.pickup_available);
        item.set_pickup_arrival_second(assessment.pickup_arrival_second);
        item.set_delivery_available(assessment.delivery_available);
        item.set_delivery_arrival_second(assessment.delivery_arrival_second);
    }
    set_carriage(builder.init_carriage(), ledger.carriage);
    Ok(())
}

fn set_work_assignment(
    mut builder: crate::ct_rpc_capnp::work_assignment::Builder<'_>,
    assignment: &WorkAssignment,
) {
    builder.set_assignment_id(assignment.assignment_id);
    builder.set_kind(match assignment.kind {
        MarketSearchKind::Supplier => crate::ct_rpc_capnp::MarketSearchKind::Supplier,
        MarketSearchKind::Buyer => crate::ct_rpc_capnp::MarketSearchKind::Buyer,
        MarketSearchKind::Freight => crate::ct_rpc_capnp::MarketSearchKind::Freight,
        MarketSearchKind::Passengers => crate::ct_rpc_capnp::MarketSearchKind::Passengers,
    });
    builder.set_method(match assignment.method {
        MarketSearchMethod::Physical => crate::ct_rpc_capnp::MarketSearchMethod::Physical,
        MarketSearchMethod::Online => crate::ct_rpc_capnp::MarketSearchMethod::Online,
        MarketSearchMethod::BlackMarket => crate::ct_rpc_capnp::MarketSearchMethod::BlackMarket,
        MarketSearchMethod::HiredBroker => crate::ct_rpc_capnp::MarketSearchMethod::HiredBroker,
    });
    builder.set_person_id(assignment.person_id);
    builder.set_commodity_id(assignment.commodity_id);
    builder.set_destination_system_id(assignment.destination_system_id);
    builder.set_started_second(assignment.started_second);
    builder.set_due_second(assignment.due_second);
    builder.set_state(match assignment.state {
        WorkState::Scheduled => crate::ct_rpc_capnp::WorkState::Scheduled,
        WorkState::Completed => crate::ct_rpc_capnp::WorkState::Completed,
        WorkState::Cancelled => crate::ct_rpc_capnp::WorkState::Cancelled,
        WorkState::Failed => crate::ct_rpc_capnp::WorkState::Failed,
    });
    builder.set_result_text(&assignment.result_text);
    builder.set_lead_id(assignment.lead_id);
    builder.set_maximum_quantity_millitons(assignment.maximum_quantity_millitons);
    builder.set_cargo_lot_id(assignment.cargo_lot_id);
}

fn set_finance(
    mut builder: crate::ct_rpc_capnp::finance_snapshot::Builder<'_>,
    finance: &FinanceSnapshot,
) {
    builder.set_title(match finance.title {
        ShipTitleKind::OwnedWithLien => crate::ct_rpc_capnp::ShipTitleKind::OwnedWithLien,
        ShipTitleKind::SponsorOwned => crate::ct_rpc_capnp::ShipTitleKind::SponsorOwned,
        ShipTitleKind::InstitutionOwned => crate::ct_rpc_capnp::ShipTitleKind::InstitutionOwned,
        ShipTitleKind::OwnedClear => crate::ct_rpc_capnp::ShipTitleKind::OwnedClear,
        ShipTitleKind::PrizeCustody => crate::ct_rpc_capnp::ShipTitleKind::PrizeCustody,
        ShipTitleKind::StolenRegistry => crate::ct_rpc_capnp::ShipTitleKind::StolenRegistry,
        ShipTitleKind::CourtImpound => crate::ct_rpc_capnp::ShipTitleKind::CourtImpound,
    });
    builder.set_liquid_credits(finance.liquid_credits);
    builder.set_restricted_credits(finance.restricted_credits);
    builder.set_reserved_credits(finance.reserved_credits);
    builder.set_original_hull_price_credits(finance.original_hull_price_credits);
    builder.set_principal_credits(finance.principal_credits);
    builder.set_monthly_payment_credits(finance.monthly_payment_credits);
    builder.set_monthly_insurance_escrow_credits(finance.monthly_insurance_escrow_credits);
    builder.set_next_payment_due_second(finance.next_payment_due_second);
    builder.set_grace_expires_second(finance.grace_expires_second);
    builder.set_paid_through_second(finance.paid_through_second);
    builder.set_in_default(finance.in_default);
    builder.set_impound_order_known_locally(finance.impound_order_known_locally);
    builder.set_credit_status(&finance.credit_status);
    builder.set_destination_assistance_active(finance.destination_assistance_active);
    builder
        .set_destination_assistance_expires_second(finance.destination_assistance_expires_second);
    builder.set_current_second(finance.current_second);
    let mut pending = builder.reborrow().init_pending_income(
        u32::try_from(finance.pending_income.len()).expect("fewer pending receivables"),
    );
    for (index, source) in finance.pending_income.iter().enumerate() {
        let mut target = pending.reborrow().get(index as u32);
        target.set_task_id(source.task_id);
        target.set_payment_credits(source.payment_credits);
        target.set_reserved_release_credits(source.reserved_release_credits);
        target.set_stage(match source.stage {
            PendingIncomeStage::FilingToOffice => {
                crate::ct_rpc_capnp::PendingIncomeStage::FilingToOffice
            }
            PendingIncomeStage::RemittanceToCaptain => {
                crate::ct_rpc_capnp::PendingIncomeStage::RemittanceToCaptain
            }
        });
        target.set_estimated_resolution_second(source.estimated_resolution_second);
        target.set_estimate_kind(match source.estimate_kind {
            IncomeEstimateKind::Projected => crate::ct_rpc_capnp::IncomeEstimateKind::Projected,
            IncomeEstimateKind::Scheduled => crate::ct_rpc_capnp::IncomeEstimateKind::Scheduled,
            IncomeEstimateKind::Unavailable => crate::ct_rpc_capnp::IncomeEstimateKind::Unavailable,
        });
    }
}

fn schema_account_transaction_class(
    value: AccountTransactionClass,
) -> crate::ct_rpc_capnp::AccountTransactionClass {
    match value {
        AccountTransactionClass::All => crate::ct_rpc_capnp::AccountTransactionClass::All,
        AccountTransactionClass::Opening => crate::ct_rpc_capnp::AccountTransactionClass::Opening,
        AccountTransactionClass::Income => crate::ct_rpc_capnp::AccountTransactionClass::Income,
        AccountTransactionClass::Expense => crate::ct_rpc_capnp::AccountTransactionClass::Expense,
        AccountTransactionClass::Transfer => crate::ct_rpc_capnp::AccountTransactionClass::Transfer,
        AccountTransactionClass::Hold => crate::ct_rpc_capnp::AccountTransactionClass::Hold,
        AccountTransactionClass::Financing => {
            crate::ct_rpc_capnp::AccountTransactionClass::Financing
        }
    }
}

fn decode_account_transaction_class(
    value: crate::ct_rpc_capnp::AccountTransactionClass,
) -> AccountTransactionClass {
    match value {
        crate::ct_rpc_capnp::AccountTransactionClass::All => AccountTransactionClass::All,
        crate::ct_rpc_capnp::AccountTransactionClass::Opening => AccountTransactionClass::Opening,
        crate::ct_rpc_capnp::AccountTransactionClass::Income => AccountTransactionClass::Income,
        crate::ct_rpc_capnp::AccountTransactionClass::Expense => AccountTransactionClass::Expense,
        crate::ct_rpc_capnp::AccountTransactionClass::Transfer => AccountTransactionClass::Transfer,
        crate::ct_rpc_capnp::AccountTransactionClass::Hold => AccountTransactionClass::Hold,
        crate::ct_rpc_capnp::AccountTransactionClass::Financing => {
            AccountTransactionClass::Financing
        }
    }
}

fn set_account_ledger(
    mut builder: crate::ct_rpc_capnp::account_ledger_page::Builder<'_>,
    page: &AccountLedgerPage,
) -> Result<(), WireError> {
    builder.set_current_second(page.current_second);
    builder.set_next_before_entry_id(page.next_before_entry_id);
    builder.set_has_more(page.has_more);
    let mut entries = builder.reborrow().init_entries(
        u32::try_from(page.entries.len())
            .map_err(|_| WireError::Expected("fewer ledger entries"))?,
    );
    for (index, source) in page.entries.iter().enumerate() {
        let mut target = entries.reborrow().get(index as u32);
        target.set_entry_id(source.entry_id);
        target.set_occurred_second(source.occurred_second);
        target.set_class(schema_account_transaction_class(source.class));
        target.set_summary(&source.summary);
        target.set_subject_ship_id(source.subject_ship_id);
        target.set_subject_ship_name(&source.subject_ship_name);
        let mut postings = target.reborrow().init_postings(
            u32::try_from(source.postings.len())
                .map_err(|_| WireError::Expected("fewer account postings"))?,
        );
        for (posting_index, source) in source.postings.iter().enumerate() {
            let mut target = postings.reborrow().get(posting_index as u32);
            target.set_account(match source.account {
                AccountKind::Liquid => crate::ct_rpc_capnp::AccountKind::Liquid,
                AccountKind::RestrictedOperating => {
                    crate::ct_rpc_capnp::AccountKind::RestrictedOperating
                }
                AccountKind::Reserved => crate::ct_rpc_capnp::AccountKind::Reserved,
                AccountKind::SecuredPrincipal => crate::ct_rpc_capnp::AccountKind::SecuredPrincipal,
            });
            target.set_change(match source.change {
                AccountChangeKind::Increase => crate::ct_rpc_capnp::AccountChangeKind::Increase,
                AccountChangeKind::Decrease => crate::ct_rpc_capnp::AccountChangeKind::Decrease,
                AccountChangeKind::BalanceForward => {
                    crate::ct_rpc_capnp::AccountChangeKind::BalanceForward
                }
            });
            target.set_amount_credits(source.amount_credits);
            target.set_balance_after_credits(source.balance_after_credits);
            target.set_ship_id(source.ship_id);
            target.set_ship_name(&source.ship_name);
        }
    }
    let mut vessels = builder.reborrow().init_vessels(
        u32::try_from(page.vessels.len())
            .map_err(|_| WireError::Expected("fewer ledger vessels"))?,
    );
    for (index, source) in page.vessels.iter().enumerate() {
        let mut target = vessels.reborrow().get(index as u32);
        target.set_ship_id(source.ship_id);
        target.set_ship_name(&source.ship_name);
    }
    Ok(())
}

fn set_fleet(
    mut builder: crate::ct_rpc_capnp::fleet_snapshot::Builder<'_>,
    fleet: &FleetSnapshot,
) -> Result<(), WireError> {
    builder.set_revision(fleet.revision);
    builder.set_active_ship_id(fleet.active_ship_id);
    let count = u32::try_from(fleet.ships.len())
        .map_err(|_| WireError::Expected("fewer managed vessels"))?;
    let mut ships = builder.init_ships(count);
    for (index, source) in fleet.ships.iter().enumerate() {
        let mut ship = ships.reborrow().get(index as u32);
        ship.set_ship_id(source.ship_id);
        ship.set_name(&source.name);
        ship.set_class_name(&source.class_name);
        ship.set_catalog_id(source.catalog_id);
        ship.set_system_id(source.system_id);
        ship.set_system_name(&source.system_name);
        ship.set_location(&source.location);
        ship.set_title(match source.title {
            ShipTitleKind::OwnedWithLien => crate::ct_rpc_capnp::ShipTitleKind::OwnedWithLien,
            ShipTitleKind::SponsorOwned => crate::ct_rpc_capnp::ShipTitleKind::SponsorOwned,
            ShipTitleKind::InstitutionOwned => crate::ct_rpc_capnp::ShipTitleKind::InstitutionOwned,
            ShipTitleKind::OwnedClear => crate::ct_rpc_capnp::ShipTitleKind::OwnedClear,
            ShipTitleKind::PrizeCustody => crate::ct_rpc_capnp::ShipTitleKind::PrizeCustody,
            ShipTitleKind::StolenRegistry => crate::ct_rpc_capnp::ShipTitleKind::StolenRegistry,
            ShipTitleKind::CourtImpound => crate::ct_rpc_capnp::ShipTitleKind::CourtImpound,
        });
        ship.set_active(source.active);
        ship.set_commanding_person_id(source.commanding_person_id);
        ship.set_commanding_person_name(&source.commanding_person_name);
        ship.set_standing_order(match source.standing_order {
            ManagedShipOrderKind::Hold => crate::ct_rpc_capnp::ManagedShipOrderKind::Hold,
            ManagedShipOrderKind::FollowActive => {
                crate::ct_rpc_capnp::ManagedShipOrderKind::FollowActive
            }
            ManagedShipOrderKind::Travel => crate::ct_rpc_capnp::ManagedShipOrderKind::Travel,
            ManagedShipOrderKind::Dock => crate::ct_rpc_capnp::ManagedShipOrderKind::Dock,
            ManagedShipOrderKind::Sell => crate::ct_rpc_capnp::ManagedShipOrderKind::Sell,
        });
        ship.set_can_assume_command(source.can_assume_command);
        ship.set_fuel_millitons(source.fuel_millitons);
        ship.set_fuel_capacity_millitons(source.fuel_capacity_millitons);
        ship.set_cargo_used_millitons(source.cargo_used_millitons);
        ship.set_cargo_capacity_millitons(source.cargo_capacity_millitons);
        ship.set_provision_person_days(source.provision_person_days);
        ship.set_provision_capacity_person_days(source.provision_capacity_person_days);
        ship.set_online_controlled(source.online_controlled);
        let cargo_count = u32::try_from(source.cargo.len())
            .map_err(|_| WireError::Expected("fewer managed-vessel cargo lots"))?;
        let mut cargo = ship.reborrow().init_cargo(cargo_count);
        for (cargo_index, lot) in source.cargo.iter().enumerate() {
            let mut item = cargo.reborrow().get(cargo_index as u32);
            item.set_cargo_lot_id(lot.cargo_lot_id);
            item.set_commodity_id(lot.commodity_id);
            item.set_commodity_name(&lot.commodity_name);
            item.set_quantity_millitons(lot.quantity_millitons);
            item.set_purchase_price_per_ton(lot.purchase_price_per_ton);
            item.set_origin_system_id(lot.origin_system_id);
            item.set_acquired_second(lot.acquired_second);
            item.set_title(match lot.title {
                CargoTitle::PlayerOwned => crate::ct_rpc_capnp::CargoTitle::PlayerOwned,
                CargoTitle::Freight => crate::ct_rpc_capnp::CargoTitle::Freight,
                CargoTitle::Contract => crate::ct_rpc_capnp::CargoTitle::Contract,
                CargoTitle::UniqueObject => crate::ct_rpc_capnp::CargoTitle::UniqueObject,
            });
            item.set_task_id(lot.task_id);
            item.set_unique_object_id(lot.unique_object_id);
            item.set_condition_percent(lot.condition_percent);
            item.set_destination_system_id(lot.destination_system_id);
            item.set_source_body_id(lot.source_body_id);
            item.set_source_lode_id(lot.source_lode_id);
            item.set_acquisition_kind(match lot.acquisition_kind {
                CargoAcquisitionKind::Purchased => {
                    crate::ct_rpc_capnp::CargoAcquisitionKind::Purchased
                }
                CargoAcquisitionKind::Extracted => {
                    crate::ct_rpc_capnp::CargoAcquisitionKind::Extracted
                }
                CargoAcquisitionKind::Captured => {
                    crate::ct_rpc_capnp::CargoAcquisitionKind::Captured
                }
                CargoAcquisitionKind::Entrusted => {
                    crate::ct_rpc_capnp::CargoAcquisitionKind::Entrusted
                }
                CargoAcquisitionKind::Unique => crate::ct_rpc_capnp::CargoAcquisitionKind::Unique,
            });
            item.set_acquisition_market_id(lot.acquisition_market_id);
            item.set_export_tariff_paid(lot.export_tariff_paid);
            item.set_valuation_basis_per_ton(lot.valuation_basis_per_ton);
        }
        let ammunition_count = u32::try_from(source.ammunition.len())
            .map_err(|_| WireError::Expected("fewer managed-vessel ammunition lots"))?;
        let mut ammunition = ship.init_ammunition(ammunition_count);
        for (ammunition_index, lot) in source.ammunition.iter().enumerate() {
            set_ammunition_status(ammunition.reborrow().get(ammunition_index as u32), lot);
        }
    }
    Ok(())
}

fn set_market_knowledge(
    mut builder: crate::ct_rpc_capnp::market_knowledge::Builder<'_>,
    knowledge: &MarketKnowledge,
) -> Result<(), WireError> {
    builder.set_current_second(knowledge.current_second);
    let count = u32::try_from(knowledge.observations.len())
        .map_err(|_| WireError::Expected("fewer market observations"))?;
    let mut observations = builder.init_observations(count);
    for (index, observation) in knowledge.observations.iter().enumerate() {
        let mut item = observations.reborrow().get(index as u32);
        item.set_system_id(observation.system_id);
        item.set_system_name(&observation.system_name);
        item.set_commodity_id(observation.commodity_id);
        item.set_commodity_name(&observation.commodity_name);
        item.set_observed_second(observation.observed_second);
        item.set_acquired_second(observation.acquired_second);
        item.set_source(&observation.source);
        item.set_confidence_percent(observation.confidence_percent);
        item.set_minimum_price_per_ton(observation.minimum_price_per_ton);
        item.set_maximum_price_per_ton(observation.maximum_price_per_ton);
        item.set_minimum_available_millitons(observation.minimum_available_millitons);
        item.set_maximum_available_millitons(observation.maximum_available_millitons);
    }
    Ok(())
}

fn set_ship_market(
    mut builder: crate::ct_rpc_capnp::ship_market::Builder<'_>,
    market: &ShipMarket,
) -> Result<(), WireError> {
    builder.set_generated_day(market.generated_day);
    builder.set_current_ship_trade_in_credits(market.current_ship_trade_in_credits);
    builder.set_outstanding_lien_credits(market.outstanding_lien_credits);
    let count =
        u32::try_from(market.offers.len()).map_err(|_| WireError::Expected("fewer ship offers"))?;
    let mut offers = builder.reborrow().init_offers(count);
    for (index, offer) in market.offers.iter().enumerate() {
        let mut item = offers.reborrow().get(index as u32);
        item.set_offer_id(offer.offer_id);
        item.set_catalog_id(offer.catalog_id);
        item.set_class_name(&offer.class_name);
        item.set_price_credits(offer.price_credits);
        item.set_original_price_credits(offer.original_price_credits);
        item.set_used(offer.used);
        item.set_age_months(offer.age_months);
        item.set_visible_condition_percent(offer.visible_condition_percent);
        item.set_cargo_capacity_millitons(offer.cargo_capacity_millitons);
        item.set_jump_rating(offer.jump_rating);
        item.set_minimum_crew(offer.minimum_crew);
    }
    let count = u32::try_from(market.commissionable_designs.len())
        .map_err(|_| WireError::Expected("fewer commissionable ship designs"))?;
    let mut designs = builder.init_commissionable_designs(count);
    for (index, design) in market.commissionable_designs.iter().enumerate() {
        let mut item = designs.reborrow().get(index as u32);
        item.set_catalog_id(design.catalog_id);
        item.set_class_name(&design.class_name);
        item.set_tech_level(design.tech_level);
        item.set_price_credits(design.price_credits);
        item.set_deposit_credits(design.deposit_credits);
        item.set_construction_seconds(design.construction_seconds);
        item.set_displacement_millitons(design.displacement_millitons);
        item.set_jump_rating(design.jump_rating);
        item.set_fuel_capacity_millitons(design.fuel_capacity_millitons);
        item.set_jump_fuel_millitons(design.jump_fuel_millitons);
        item.set_cargo_capacity_millitons(design.cargo_capacity_millitons);
        item.set_minimum_crew(design.minimum_crew);
    }
    Ok(())
}

fn set_crew_market(
    mut builder: crate::ct_rpc_capnp::crew_market::Builder<'_>,
    market: &CrewMarket,
) -> Result<(), WireError> {
    builder.set_generated_day(market.generated_day);
    let count = u32::try_from(market.candidates.len())
        .map_err(|_| WireError::Expected("fewer crew candidates"))?;
    let mut candidates = builder.init_candidates(count);
    for (index, candidate) in market.candidates.iter().enumerate() {
        let mut item = candidates.reborrow().get(index as u32);
        item.set_candidate_id(candidate.candidate_id);
        item.set_role(&candidate.role);
        item.set_name(&candidate.name);
        item.set_primary_skill(encode_skill(candidate.primary_skill));
        item.set_skill_level(candidate.skill_level);
        item.set_monthly_salary_credits(candidate.monthly_salary_credits);
    }
    Ok(())
}

fn set_market(
    mut builder: crate::ct_rpc_capnp::market_snapshot::Builder<'_>,
    snapshot: &MarketSnapshot,
) -> Result<(), WireError> {
    builder.set_market_revision(snapshot.market_revision);
    builder.set_system_id(snapshot.system_id);
    builder.set_world_name(&snapshot.world_name);
    builder.set_generated_day(snapshot.generated_day);
    builder.set_credits(snapshot.credits);
    builder.set_cargo_used_millitons(snapshot.cargo_used_millitons);
    builder.set_cargo_capacity_millitons(snapshot.cargo_capacity_millitons);
    let offer_count = u32::try_from(snapshot.offers.len())
        .map_err(|_| WireError::Expected("fewer market offers"))?;
    let mut offers = builder.reborrow().init_offers(offer_count);
    for (index, offer) in snapshot.offers.iter().enumerate() {
        let mut item = offers.reborrow().get(index as u32);
        item.set_offer_id(offer.offer_id);
        item.set_commodity_id(offer.commodity_id);
        item.set_commodity_name(&offer.commodity_name);
        item.set_base_price_per_ton(offer.base_price_per_ton);
        item.set_purchase_price_per_ton(offer.purchase_price_per_ton);
        item.set_sale_price_per_ton(offer.sale_price_per_ton);
        item.set_available_millitons(offer.available_millitons);
        item.set_legality(match offer.legality {
            CommodityLegality::Legal => crate::ct_rpc_capnp::CommodityLegality::Legal,
            CommodityLegality::Restricted => crate::ct_rpc_capnp::CommodityLegality::Restricted,
            CommodityLegality::Prohibited => crate::ct_rpc_capnp::CommodityLegality::Prohibited,
        });
        set_price_distribution(
            item.reborrow().init_price_distribution(),
            offer.price_distribution,
        );
    }
    let cargo_count =
        u32::try_from(snapshot.cargo.len()).map_err(|_| WireError::Expected("fewer cargo lots"))?;
    let mut cargo = builder.reborrow().init_cargo(cargo_count);
    for (index, lot) in snapshot.cargo.iter().enumerate() {
        let mut item = cargo.reborrow().get(index as u32);
        item.set_cargo_lot_id(lot.cargo_lot_id);
        item.set_commodity_id(lot.commodity_id);
        item.set_commodity_name(&lot.commodity_name);
        item.set_quantity_millitons(lot.quantity_millitons);
        item.set_purchase_price_per_ton(lot.purchase_price_per_ton);
        item.set_origin_system_id(lot.origin_system_id);
        item.set_acquired_second(lot.acquired_second);
        item.set_title(match lot.title {
            CargoTitle::PlayerOwned => crate::ct_rpc_capnp::CargoTitle::PlayerOwned,
            CargoTitle::Freight => crate::ct_rpc_capnp::CargoTitle::Freight,
            CargoTitle::Contract => crate::ct_rpc_capnp::CargoTitle::Contract,
            CargoTitle::UniqueObject => crate::ct_rpc_capnp::CargoTitle::UniqueObject,
        });
        item.set_task_id(lot.task_id);
        item.set_unique_object_id(lot.unique_object_id);
        item.set_condition_percent(lot.condition_percent);
        item.set_destination_system_id(lot.destination_system_id);
        item.set_source_body_id(lot.source_body_id);
        item.set_source_lode_id(lot.source_lode_id);
        item.set_acquisition_kind(match lot.acquisition_kind {
            CargoAcquisitionKind::Purchased => crate::ct_rpc_capnp::CargoAcquisitionKind::Purchased,
            CargoAcquisitionKind::Extracted => crate::ct_rpc_capnp::CargoAcquisitionKind::Extracted,
            CargoAcquisitionKind::Captured => crate::ct_rpc_capnp::CargoAcquisitionKind::Captured,
            CargoAcquisitionKind::Entrusted => crate::ct_rpc_capnp::CargoAcquisitionKind::Entrusted,
            CargoAcquisitionKind::Unique => crate::ct_rpc_capnp::CargoAcquisitionKind::Unique,
        });
        item.set_acquisition_market_id(lot.acquisition_market_id);
        item.set_export_tariff_paid(lot.export_tariff_paid);
        item.set_valuation_basis_per_ton(lot.valuation_basis_per_ton);
    }
    let quote_count = u32::try_from(snapshot.cargo_sale_quotes.len())
        .map_err(|_| WireError::Expected("fewer cargo sale quotes"))?;
    let mut quotes = builder.reborrow().init_cargo_sale_quotes(quote_count);
    for (index, quote) in snapshot.cargo_sale_quotes.iter().enumerate() {
        let mut item = quotes.reborrow().get(index as u32);
        item.set_cargo_lot_id(quote.cargo_lot_id);
        item.set_price_per_ton(quote.price_per_ton);
        set_price_distribution(
            item.reborrow().init_price_distribution(),
            quote.price_distribution,
        );
    }
    let code_count = u32::try_from(snapshot.trade_codes.len())
        .map_err(|_| WireError::Expected("fewer trade codes"))?;
    let mut codes = builder.reborrow().init_trade_codes(code_count);
    for (index, code) in snapshot.trade_codes.iter().enumerate() {
        codes.set(index as u32, code);
    }
    builder.set_tariff_basis_points(snapshot.tariff_basis_points);
    builder.set_import_tariff_basis_points(snapshot.import_tariff_basis_points);
    builder.set_export_tariff_basis_points(snapshot.export_tariff_basis_points);
    let task_count = u32::try_from(snapshot.local_task_offers.len())
        .map_err(|_| WireError::Expected("fewer task offers"))?;
    let mut tasks = builder.reborrow().init_local_task_offers(task_count);
    for (index, offer) in snapshot.local_task_offers.iter().enumerate() {
        set_task_offer(tasks.reborrow().get(index as u32), offer);
    }
    let work_count = u32::try_from(snapshot.work_assignments.len())
        .map_err(|_| WireError::Expected("fewer work assignments"))?;
    let mut work = builder.reborrow().init_work_assignments(work_count);
    for (index, assignment) in snapshot.work_assignments.iter().enumerate() {
        set_work_assignment(work.reborrow().get(index as u32), assignment);
    }
    let lead_count = u32::try_from(snapshot.leads.len())
        .map_err(|_| WireError::Expected("fewer market leads"))?;
    let mut leads = builder.reborrow().init_leads(lead_count);
    for (index, lead) in snapshot.leads.iter().enumerate() {
        let mut item = leads.reborrow().get(index as u32);
        item.set_lead_id(lead.lead_id);
        item.set_revision(lead.revision);
        item.set_side(match lead.side {
            MarketLeadSide::Supplier => crate::ct_rpc_capnp::MarketLeadSide::Supplier,
            MarketLeadSide::Buyer => crate::ct_rpc_capnp::MarketLeadSide::Buyer,
        });
        item.set_state(match lead.state {
            MarketLeadState::Available => crate::ct_rpc_capnp::MarketLeadState::Available,
            MarketLeadState::Reserved => crate::ct_rpc_capnp::MarketLeadState::Reserved,
            MarketLeadState::Performed => crate::ct_rpc_capnp::MarketLeadState::Performed,
            MarketLeadState::Expired => crate::ct_rpc_capnp::MarketLeadState::Expired,
            MarketLeadState::Cancelled => crate::ct_rpc_capnp::MarketLeadState::Cancelled,
            MarketLeadState::Negotiating => crate::ct_rpc_capnp::MarketLeadState::Negotiating,
            MarketLeadState::Quoted => crate::ct_rpc_capnp::MarketLeadState::Quoted,
            MarketLeadState::Rejected => crate::ct_rpc_capnp::MarketLeadState::Rejected,
        });
        item.set_system_id(lead.system_id);
        item.set_commodity_id(lead.commodity_id);
        item.set_commodity_name(&lead.commodity_name);
        item.set_quantity_millitons(lead.quantity_millitons);
        item.set_price_per_ton(lead.price_per_ton);
        item.set_discovered_second(lead.discovered_second);
        item.set_expires_second(lead.expires_second);
        item.set_reservation_expires_second(lead.reservation_expires_second);
        item.set_escrow_credits(lead.escrow_credits);
        item.set_source(&lead.source);
        item.set_confidence_percent(lead.confidence_percent);
        item.set_counterparty_id(lead.counterparty_id);
        item.set_cargo_lot_id(lead.cargo_lot_id);
        item.set_penalty_until_second(lead.penalty_until_second);
        item.set_illegal(lead.illegal);
        item.set_loader_fee_credits(lead.loader_fee_credits);
    }
    let event_count = u32::try_from(snapshot.events.len())
        .map_err(|_| WireError::Expected("fewer market events"))?;
    let mut events = builder.reborrow().init_events(event_count);
    for (index, event) in snapshot.events.iter().enumerate() {
        let mut item = events.reborrow().get(index as u32);
        item.set_event_id(event.event_id);
        item.set_kind(match event.kind {
            MarketEventKind::Shortage => crate::ct_rpc_capnp::MarketEventKind::Shortage,
            MarketEventKind::Surplus => crate::ct_rpc_capnp::MarketEventKind::Surplus,
            MarketEventKind::Disruption => crate::ct_rpc_capnp::MarketEventKind::Disruption,
            MarketEventKind::Recovery => crate::ct_rpc_capnp::MarketEventKind::Recovery,
        });
        item.set_commodity_id(event.commodity_id);
        item.set_commodity_name(&event.commodity_name);
        item.set_start_second(event.start_second);
        item.set_expires_second(event.expires_second);
        item.set_stock_multiplier_basis_points(event.stock_multiplier_basis_points);
        item.set_purchase_tier_delta(event.purchase_tier_delta);
        item.set_sale_tier_delta(event.sale_tier_delta);
        item.set_supplier_offer_multiplier_basis_points(
            event.supplier_offer_multiplier_basis_points,
        );
        item.set_buyer_offer_multiplier_basis_points(event.buyer_offer_multiplier_basis_points);
        item.set_carriage_offer_multiplier_basis_points(
            event.carriage_offer_multiplier_basis_points,
        );
        item.set_headline(&event.headline);
    }
    Ok(())
}

fn set_price_distribution(
    mut builder: crate::ct_rpc_capnp::price_distribution::Builder<'_>,
    distribution: PriceDistribution,
) {
    builder.set_minimum(distribution.minimum);
    builder.set_lower_quartile(distribution.lower_quartile);
    builder.set_median(distribution.median);
    builder.set_upper_quartile(distribution.upper_quartile);
    builder.set_maximum(distribution.maximum);
}

fn set_travel_status(
    mut builder: crate::ct_rpc_capnp::travel_status::Builder<'_>,
    snapshot: &TravelStatus,
) {
    builder.set_ship_id(snapshot.ship_id);
    builder.set_ship_name(&snapshot.ship_name);
    builder.set_current_system_id(snapshot.current_system_id);
    builder.set_current_system_name(&snapshot.current_system_name);
    builder.set_destination_system_id(snapshot.destination_system_id);
    builder.set_destination_system_name(&snapshot.destination_system_name);
    builder.set_stage(match snapshot.stage {
        TravelStage::Docked => crate::ct_rpc_capnp::TravelStage::Docked,
        TravelStage::DepartingForJump => crate::ct_rpc_capnp::TravelStage::DepartingForJump,
        TravelStage::JumpSpace => crate::ct_rpc_capnp::TravelStage::JumpSpace,
        TravelStage::ApproachingStarport => crate::ct_rpc_capnp::TravelStage::ApproachingStarport,
        TravelStage::Refit => crate::ct_rpc_capnp::TravelStage::Refit,
        TravelStage::ProperRepair => crate::ct_rpc_capnp::TravelStage::ProperRepair,
        TravelStage::GasGiantSkim => crate::ct_rpc_capnp::TravelStage::GasGiantSkim,
        TravelStage::WildernessWater => crate::ct_rpc_capnp::TravelStage::WildernessWater,
        TravelStage::Holding => crate::ct_rpc_capnp::TravelStage::Holding,
        TravelStage::Encounter => crate::ct_rpc_capnp::TravelStage::Encounter,
        TravelStage::BeltProspecting => crate::ct_rpc_capnp::TravelStage::BeltProspecting,
        TravelStage::BeltSurvey => crate::ct_rpc_capnp::TravelStage::BeltSurvey,
        TravelStage::BeltMining => crate::ct_rpc_capnp::TravelStage::BeltMining,
        TravelStage::BeltRefining => crate::ct_rpc_capnp::TravelStage::BeltRefining,
        TravelStage::BeltRecovery => crate::ct_rpc_capnp::TravelStage::BeltRecovery,
        TravelStage::BeltEgress => crate::ct_rpc_capnp::TravelStage::BeltEgress,
        TravelStage::FuelProcessing => crate::ct_rpc_capnp::TravelStage::FuelProcessing,
        TravelStage::Maneuvering => crate::ct_rpc_capnp::TravelStage::Maneuvering,
    });
    builder.set_current_game_second(snapshot.current_game_second);
    builder.set_due_second(snapshot.due_second);
    builder.set_current_fuel_millitons(snapshot.current_fuel_millitons);
    builder.set_fuel_capacity_millitons(snapshot.fuel_capacity_millitons);
    builder.set_jump_fuel_millitons(snapshot.jump_fuel_millitons);
    builder.set_clock_rate_game_seconds(crate::clock::GAME_SECONDS_PER_RATE_PERIOD);
    builder.set_clock_rate_real_seconds(crate::clock::RATE_PERIOD.as_secs());
    builder.set_plan_id(snapshot.plan_id);
    builder.set_plan_revision(snapshot.plan_revision);
    builder.set_leg_index(snapshot.leg_index);
    set_flight_locus(builder.reborrow().init_origin(), snapshot.origin);
    set_flight_locus(builder.init_destination(), snapshot.destination);
}

fn set_flight_locus(
    mut builder: crate::ct_rpc_capnp::flight_locus::Builder<'_>,
    locus: FlightLocus,
) {
    match locus {
        FlightLocus::Port {
            system_id,
            world_id,
            facility_id,
        } => {
            builder.set_system_id(system_id);
            let mut port = builder.init_port();
            port.set_world_id(world_id);
            port.set_facility_id(facility_id);
        }
        FlightLocus::JumpLocus { system_id } => {
            builder.set_system_id(system_id);
            builder.set_jump_locus(());
            builder.set_jump_role(crate::ct_rpc_capnp::JumpLocusRole::Departure);
        }
        FlightLocus::ArrivalLocus { system_id, remote } => {
            builder.set_system_id(system_id);
            builder.set_jump_locus(());
            builder.set_jump_role(crate::ct_rpc_capnp::JumpLocusRole::Arrival);
            builder.set_remote_arrival(remote);
        }
        FlightLocus::Body { system_id, body_id } => {
            builder.set_system_id(system_id);
            builder.set_body_id(body_id);
        }
        FlightLocus::DeepSpace { position } => {
            builder.set_system_id(0);
            let [coreward, spinward, north] = position.parsecs();
            let mut target = builder.init_deep_space();
            target.set_coreward(coreward);
            target.set_spinward(spinward);
            target.set_north(north);
        }
    }
}

fn encode_encounter_posture(value: EncounterPosture) -> crate::ct_rpc_capnp::EncounterPosture {
    use crate::ct_rpc_capnp::EncounterPosture as Wire;
    match value {
        EncounterPosture::Fight => Wire::Fight,
        EncounterPosture::Flee => Wire::Flee,
        EncounterPosture::Comply => Wire::Comply,
        EncounterPosture::Surrender => Wire::Surrender,
        EncounterPosture::Board => Wire::Board,
        EncounterPosture::Pursue => Wire::Pursue,
        EncounterPosture::ContinueCourse => Wire::ContinueCourse,
    }
}

fn encode_encounter_fallback(value: EncounterFallback) -> crate::ct_rpc_capnp::EncounterFallback {
    use crate::ct_rpc_capnp::EncounterFallback as Wire;
    match value {
        EncounterFallback::Surrender => Wire::Surrender,
        EncounterFallback::Abandon => Wire::Abandon,
        EncounterFallback::JettisonCargo => Wire::JettisonCargo,
        EncounterFallback::BreakOff => Wire::BreakOff,
    }
}

fn encode_encounter_kind(value: EncounterKind) -> crate::ct_rpc_capnp::EncounterKind {
    match value {
        EncounterKind::RoutineTraffic => crate::ct_rpc_capnp::EncounterKind::RoutineTraffic,
        EncounterKind::TrafficControl => crate::ct_rpc_capnp::EncounterKind::TrafficControl,
        EncounterKind::Inspection => crate::ct_rpc_capnp::EncounterKind::Inspection,
        EncounterKind::Distress => crate::ct_rpc_capnp::EncounterKind::Distress,
        EncounterKind::Derelict => crate::ct_rpc_capnp::EncounterKind::Derelict,
        EncounterKind::Hazard => crate::ct_rpc_capnp::EncounterKind::Hazard,
        EncounterKind::Hostile => crate::ct_rpc_capnp::EncounterKind::Hostile,
        EncounterKind::Military => crate::ct_rpc_capnp::EncounterKind::Military,
        EncounterKind::DepartingContact => crate::ct_rpc_capnp::EncounterKind::DepartingContact,
    }
}

fn set_encounter_policy(
    mut builder: crate::ct_rpc_capnp::encounter_policy::Builder<'_>,
    policy: &EncounterPolicy,
) -> Result<(), WireError> {
    builder.set_hostile_posture(encode_encounter_posture(policy.hostile_posture));
    let count = u32::try_from(policy.hostile_fallbacks.len())
        .map_err(|_| WireError::Expected("fewer encounter fallbacks"))?;
    let mut list = builder.reborrow().init_hostile_fallbacks(count);
    for (index, value) in policy.hostile_fallbacks.iter().enumerate() {
        list.set(index as u32, encode_encounter_fallback(*value));
    }
    builder.set_comply_with_inspection(policy.comply_with_inspection);
    builder.set_report_distress(policy.report_distress);
    builder.set_assist_distress(policy.assist_distress);
    let order_count = u32::try_from(policy.standing_orders.len())
        .map_err(|_| WireError::Expected("fewer encounter standing orders"))?;
    let mut orders = builder.reborrow().init_standing_orders(order_count);
    for (index, order) in policy.standing_orders.iter().enumerate() {
        let mut target = orders.reborrow().get(index as u32);
        target.set_kind(encode_encounter_kind(order.kind));
        target.set_ordinary_posture(encode_encounter_posture(order.ordinary_posture));
        target.set_fight_mode(match order.fight_mode {
            EncounterFightMode::Never => crate::ct_rpc_capnp::EncounterFightMode::Never,
            EncounterFightMode::Always => crate::ct_rpc_capnp::EncounterFightMode::Always,
            EncounterFightMode::EstimatedAtLeast => {
                crate::ct_rpc_capnp::EncounterFightMode::EstimatedAtLeast
            }
        });
        target.set_minimum_outlook_percent(order.minimum_outlook_percent);
    }
    Ok(())
}

fn set_flight_plan_action(
    mut builder: crate::ct_rpc_capnp::flight_plan_action::Builder<'_>,
    action: &FlightPlanAction,
) {
    match action {
        FlightPlanAction::Hold => builder.set_hold(()),
        FlightPlanAction::Jump {
            destination_system_id,
            navigation,
            proceed_on_known_bad_plot,
            remote_arrival,
            departure_locus_arrival,
        } => {
            let mut jump = builder.init_jump();
            jump.set_destination_system_id(*destination_system_id);
            jump.set_navigation(match navigation {
                JumpNavigationMethod::Onboard => crate::ct_rpc_capnp::JumpNavigationMethod::Onboard,
                JumpNavigationMethod::CommercialTape => {
                    crate::ct_rpc_capnp::JumpNavigationMethod::CommercialTape
                }
            });
            jump.set_proceed_on_known_bad_plot(*proceed_on_known_bad_plot);
            jump.set_remote_arrival(*remote_arrival);
            jump.set_departure_locus_arrival(*departure_locus_arrival);
        }
        FlightPlanAction::JumpCoordinates {
            destination,
            navigation,
            proceed_on_known_bad_plot,
        } => {
            let [coreward, spinward, north] = destination.parsecs();
            let mut jump = builder.init_jump_coordinates();
            jump.set_navigation(match navigation {
                JumpNavigationMethod::Onboard => crate::ct_rpc_capnp::JumpNavigationMethod::Onboard,
                JumpNavigationMethod::CommercialTape => {
                    crate::ct_rpc_capnp::JumpNavigationMethod::CommercialTape
                }
            });
            jump.set_proceed_on_known_bad_plot(*proceed_on_known_bad_plot);
            let mut target = jump.init_destination();
            target.set_coreward(coreward);
            target.set_spinward(spinward);
            target.set_north(north);
        }
        FlightPlanAction::Dock {
            world_id,
            facility_id,
        } => {
            let mut port = builder.init_dock();
            port.set_world_id(*world_id);
            port.set_facility_id(*facility_id);
        }
        FlightPlanAction::Fuel {
            operation,
            quantity_millitons,
            refine_collected,
        } => {
            let mut fuel = builder.init_fuel();
            fuel.set_operation(match operation {
                FuelOperation::GasGiant => crate::ct_rpc_capnp::FuelOperation::GasGiant,
                FuelOperation::WildernessWater => {
                    crate::ct_rpc_capnp::FuelOperation::WildernessWater
                }
                FuelOperation::BuyRefined => crate::ct_rpc_capnp::FuelOperation::BuyRefined,
                FuelOperation::BuyUnrefined => crate::ct_rpc_capnp::FuelOperation::BuyUnrefined,
            });
            fuel.set_quantity_millitons(*quantity_millitons);
            fuel.set_refine_collected(*refine_collected);
        }
        FlightPlanAction::BeltCycle { body_id } => builder.set_belt_cycle(*body_id),
        FlightPlanAction::RefineFuel { quantity_millitons } => {
            builder.set_refine_fuel(*quantity_millitons)
        }
    }
}

fn set_flight_plan_step(
    mut builder: crate::ct_rpc_capnp::flight_plan_step::Builder<'_>,
    step: &FlightPlanStep,
) {
    set_flight_locus(builder.reborrow().init_locus(), step.locus);
    builder.set_authority(match step.authority {
        WaypointAuthority::Hold => crate::ct_rpc_capnp::WaypointAuthority::Hold,
        WaypointAuthority::Through => crate::ct_rpc_capnp::WaypointAuthority::Through,
    });
    builder.set_terminal(step.terminal);
    set_flight_plan_action(builder.init_action(), &step.action);
}

fn set_flight_plan_proposal(
    mut builder: crate::ct_rpc_capnp::flight_plan_proposal::Builder<'_>,
    proposal: &FlightPlanProposal,
) -> Result<(), WireError> {
    builder.set_expected_plan_revision(proposal.expected_plan_revision);
    let count = u32::try_from(proposal.steps.len())
        .map_err(|_| WireError::Expected("fewer flight-plan steps"))?;
    let mut steps = builder.reborrow().init_steps(count);
    for (index, step) in proposal.steps.iter().enumerate() {
        set_flight_plan_step(steps.reborrow().get(index as u32), step);
    }
    builder.set_preserve_active_step(proposal.preserve_active_step);
    set_encounter_policy(builder.init_policy(), &proposal.policy)
}

fn set_flight_plan_snapshot(
    mut builder: crate::ct_rpc_capnp::flight_plan_snapshot::Builder<'_>,
    snapshot: &FlightPlanSnapshot,
) -> Result<(), WireError> {
    builder.set_plan_id(snapshot.plan_id);
    builder.set_revision(snapshot.revision);
    builder.set_current_step(snapshot.current_step);
    builder.set_state(match snapshot.state {
        FlightPlanState::Inactive => crate::ct_rpc_capnp::FlightPlanState::Inactive,
        FlightPlanState::Active => crate::ct_rpc_capnp::FlightPlanState::Active,
        FlightPlanState::Held => crate::ct_rpc_capnp::FlightPlanState::Held,
        FlightPlanState::Checkpoint => crate::ct_rpc_capnp::FlightPlanState::Checkpoint,
        FlightPlanState::Encounter => crate::ct_rpc_capnp::FlightPlanState::Encounter,
        FlightPlanState::Completed => crate::ct_rpc_capnp::FlightPlanState::Completed,
        FlightPlanState::Terminal => crate::ct_rpc_capnp::FlightPlanState::Terminal,
    });
    let count = u32::try_from(snapshot.steps.len())
        .map_err(|_| WireError::Expected("fewer flight-plan steps"))?;
    let mut steps = builder.reborrow().init_steps(count);
    for (index, step) in snapshot.steps.iter().enumerate() {
        set_flight_plan_step(steps.reborrow().get(index as u32), step);
    }
    set_encounter_policy(builder.reborrow().init_policy(), &snapshot.policy)?;
    builder.set_suspension_reason(&snapshot.suspension_reason);
    Ok(())
}

fn set_flight_plan_preview(
    mut builder: crate::ct_rpc_capnp::flight_plan_preview::Builder<'_>,
    preview: &FlightPlanPreview,
) -> Result<(), WireError> {
    set_flight_plan_proposal(builder.reborrow().init_proposal(), &preview.proposal)?;
    builder.set_preview_hash(&preview.preview_hash);
    builder.set_elapsed_seconds(preview.elapsed_seconds);
    builder.set_fuel_millitons(preview.fuel_millitons);
    let count = u32::try_from(preview.warnings.len())
        .map_err(|_| WireError::Expected("fewer flight-plan warnings"))?;
    let mut warnings = builder.reborrow().init_warnings(count);
    for (index, warning) in preview.warnings.iter().enumerate() {
        let mut item = warnings.reborrow().get(index as u32);
        item.set_code(&warning.code);
        item.set_message(&warning.message);
        let step_count = u32::try_from(warning.step_indices.len())
            .map_err(|_| WireError::Expected("fewer warning step references"))?;
        let mut step_indices = item.init_step_indices(step_count);
        for (step_index, value) in warning.step_indices.iter().enumerate() {
            step_indices.set(step_index as u32, *value);
        }
    }
    let offer_count = u32::try_from(preview.carriage_offers.len())
        .map_err(|_| WireError::Expected("fewer carriage offers"))?;
    let mut offers = builder.reborrow().init_carriage_offers(offer_count);
    for (index, offer) in preview.carriage_offers.iter().enumerate() {
        set_task_offer(offers.reborrow().get(index as u32), offer);
    }
    builder.set_carriage_revenue_credits(preview.carriage_revenue_credits);
    builder.set_carriage_broker_fees_credits(preview.carriage_broker_fees_credits);
    let timing_count = u32::try_from(preview.fuel_timings.len())
        .map_err(|_| WireError::Expected("fewer fuel-operation timings"))?;
    let mut timings = builder.reborrow().init_fuel_timings(timing_count);
    for (index, timing) in preview.fuel_timings.iter().enumerate() {
        let mut item = timings.reborrow().get(index as u32);
        item.set_step_index(timing.step_index);
        item.set_round_trip_seconds(timing.round_trip_seconds);
        item.set_collection_seconds(timing.collection_seconds);
        item.set_processing_seconds(timing.processing_seconds);
        item.set_failed_processing_seconds(timing.failed_processing_seconds);
        item.set_normal_total_seconds(timing.normal_total_seconds);
        item.set_failed_total_seconds(timing.failed_total_seconds);
        item.set_output_refined(timing.output_refined);
    }
    Ok(())
}

fn set_checkpoint_snapshot(
    mut builder: crate::ct_rpc_capnp::checkpoint_snapshot::Builder<'_>,
    value: &CheckpointSnapshot,
) {
    builder.set_checkpoint_id(value.checkpoint_id);
    builder.set_plan_id(value.plan_id);
    builder.set_plan_revision(value.plan_revision);
    builder.set_step_index(value.step_index);
    set_flight_locus(builder.reborrow().init_locus(), value.locus);
    builder.set_kind(match value.kind {
        CheckpointKind::PortDeparture => crate::ct_rpc_capnp::CheckpointKind::PortDeparture,
        CheckpointKind::InhabitedWorld => crate::ct_rpc_capnp::CheckpointKind::InhabitedWorld,
        CheckpointKind::GasGiant => crate::ct_rpc_capnp::CheckpointKind::GasGiant,
        CheckpointKind::JumpArrival => crate::ct_rpc_capnp::CheckpointKind::JumpArrival,
        CheckpointKind::JumpDeparture => crate::ct_rpc_capnp::CheckpointKind::JumpDeparture,
        CheckpointKind::DeepSpace => crate::ct_rpc_capnp::CheckpointKind::DeepSpace,
    });
    builder.set_ready_second(value.ready_second);
    builder.set_acknowledged(value.acknowledged);
}

fn set_encounter_snapshot(
    mut builder: crate::ct_rpc_capnp::encounter_snapshot::Builder<'_>,
    value: &EncounterSnapshot,
) {
    builder.set_encounter_id(value.encounter_id);
    builder.set_revision(value.revision);
    builder.set_kind(match value.kind {
        EncounterKind::RoutineTraffic => crate::ct_rpc_capnp::EncounterKind::RoutineTraffic,
        EncounterKind::TrafficControl => crate::ct_rpc_capnp::EncounterKind::TrafficControl,
        EncounterKind::Inspection => crate::ct_rpc_capnp::EncounterKind::Inspection,
        EncounterKind::Distress => crate::ct_rpc_capnp::EncounterKind::Distress,
        EncounterKind::Derelict => crate::ct_rpc_capnp::EncounterKind::Derelict,
        EncounterKind::Hazard => crate::ct_rpc_capnp::EncounterKind::Hazard,
        EncounterKind::Hostile => crate::ct_rpc_capnp::EncounterKind::Hostile,
        EncounterKind::Military => crate::ct_rpc_capnp::EncounterKind::Military,
        EncounterKind::DepartingContact => crate::ct_rpc_capnp::EncounterKind::DepartingContact,
    });
    builder.set_state(match value.state {
        EncounterState::AwaitingPosture => crate::ct_rpc_capnp::EncounterState::AwaitingPosture,
        EncounterState::Resolving => crate::ct_rpc_capnp::EncounterState::Resolving,
        EncounterState::Resolved => crate::ct_rpc_capnp::EncounterState::Resolved,
    });
    builder.set_started_second(value.started_second);
    builder.set_next_turn_second(value.next_turn_second);
    builder.set_turn(value.turn);
    let mut contact = builder.reborrow().init_contact();
    contact.set_contact_id(value.contact.contact_id);
    contact.set_ship_name(&value.contact.ship_name);
    contact.set_class_name(&value.contact.class_name);
    contact.set_declared_class_name(&value.contact.declared_class_name);
    contact.set_transponder(&value.contact.transponder);
    contact.set_role(&value.contact.role);
    contact.set_range(&value.contact.range);
    contact.set_confidence_percent(value.contact.confidence_percent);
    contact.set_resolution(match value.contact.resolution {
        EncounterResolution::RadioOnly => crate::ct_rpc_capnp::EncounterResolution::RadioOnly,
        EncounterResolution::TransponderOnly => {
            crate::ct_rpc_capnp::EncounterResolution::TransponderOnly
        }
        EncounterResolution::Approximate => crate::ct_rpc_capnp::EncounterResolution::Approximate,
        EncounterResolution::Identified => crate::ct_rpc_capnp::EncounterResolution::Identified,
    });
    builder.set_summary(&value.summary);
    builder.set_authority(match value.authority {
        EncounterAuthority::None => crate::ct_rpc_capnp::EncounterAuthority::None,
        EncounterAuthority::Pirate => crate::ct_rpc_capnp::EncounterAuthority::Pirate,
        EncounterAuthority::TrafficControl => {
            crate::ct_rpc_capnp::EncounterAuthority::TrafficControl
        }
        EncounterAuthority::Customs => crate::ct_rpc_capnp::EncounterAuthority::Customs,
        EncounterAuthority::Naval => crate::ct_rpc_capnp::EncounterAuthority::Naval,
        EncounterAuthority::Warrant => crate::ct_rpc_capnp::EncounterAuthority::Warrant,
    });
    builder.set_threat(match value.threat {
        EncounterThreat::Unknown => crate::ct_rpc_capnp::EncounterThreat::Unknown,
        EncounterThreat::Favorable => crate::ct_rpc_capnp::EncounterThreat::Favorable,
        EncounterThreat::Comparable => crate::ct_rpc_capnp::EncounterThreat::Comparable,
        EncounterThreat::Dangerous => crate::ct_rpc_capnp::EncounterThreat::Dangerous,
        EncounterThreat::Overwhelming => crate::ct_rpc_capnp::EncounterThreat::Overwhelming,
    });
    let mut demand = builder.reborrow().init_demand();
    demand.set_present(value.demand.present);
    demand.set_player_owned_percent(value.demand.player_owned_percent);
    demand.set_player_owned_millitons(value.demand.player_owned_millitons);
    demand.set_entrusted_millitons(value.demand.entrusted_millitons);
    demand.set_unique_object_count(value.demand.unique_object_count);
    demand.set_text(&value.demand.text);
    demand.set_entrusted_liability_credits(value.demand.entrusted_liability_credits);
    let mut postures = builder
        .reborrow()
        .init_available_postures(value.available_postures.len() as u32);
    for (index, posture) in value.available_postures.iter().enumerate() {
        postures.set(index as u32, encode_encounter_posture(*posture));
    }
    let mut fallbacks = builder
        .reborrow()
        .init_available_fallbacks(value.available_fallbacks.len() as u32);
    for (index, fallback) in value.available_fallbacks.iter().enumerate() {
        fallbacks.set(index as u32, encode_encounter_fallback(*fallback));
    }
    builder.set_response_deadline_second(value.response_deadline_second);
    builder.set_estimated_combat_outlook_percent(match value.threat {
        EncounterThreat::Unknown => 0,
        EncounterThreat::Favorable => 70,
        EncounterThreat::Comparable => 50,
        EncounterThreat::Dangerous => 33,
        EncounterThreat::Overwhelming => 12,
    });
}

fn set_encounter_result(
    mut builder: crate::ct_rpc_capnp::encounter_result::Builder<'_>,
    value: &EncounterResult,
) {
    builder.set_encounter_id(value.encounter_id);
    builder.set_resolved(value.resolved);
    builder.set_terminal(value.terminal);
    builder.set_outcome(&value.outcome);
    builder.set_turns(value.turns);
    builder.set_cargo_lost_millitons(value.cargo_lost_millitons);
    builder.set_fuel_lost_millitons(value.fuel_lost_millitons);
    builder.set_damage_hits(value.damage_hits);
}

fn set_terminal_report(
    mut builder: crate::ct_rpc_capnp::terminal_report::Builder<'_>,
    value: &TerminalReport,
) -> Result<(), WireError> {
    builder.set_encounter_id(value.encounter_id);
    builder.set_revision(value.revision);
    builder.set_acknowledged(value.acknowledged);
    builder.set_started_second(value.started_second);
    builder.set_resolved_second(value.resolved_second);
    builder.set_system_id(value.system_id);
    builder.set_system_name(&value.system_name);
    builder.set_location(&value.location);
    let mut contact = builder.reborrow().init_contact();
    contact.set_contact_id(value.contact.contact_id);
    contact.set_ship_name(&value.contact.ship_name);
    contact.set_class_name(&value.contact.class_name);
    contact.set_declared_class_name(&value.contact.declared_class_name);
    contact.set_transponder(&value.contact.transponder);
    contact.set_role(&value.contact.role);
    contact.set_range(&value.contact.range);
    contact.set_confidence_percent(value.contact.confidence_percent);
    contact.set_resolution(match value.contact.resolution {
        EncounterResolution::RadioOnly => crate::ct_rpc_capnp::EncounterResolution::RadioOnly,
        EncounterResolution::TransponderOnly => {
            crate::ct_rpc_capnp::EncounterResolution::TransponderOnly
        }
        EncounterResolution::Approximate => crate::ct_rpc_capnp::EncounterResolution::Approximate,
        EncounterResolution::Identified => crate::ct_rpc_capnp::EncounterResolution::Identified,
    });
    builder.set_authority(match value.authority {
        EncounterAuthority::None => crate::ct_rpc_capnp::EncounterAuthority::None,
        EncounterAuthority::Pirate => crate::ct_rpc_capnp::EncounterAuthority::Pirate,
        EncounterAuthority::TrafficControl => {
            crate::ct_rpc_capnp::EncounterAuthority::TrafficControl
        }
        EncounterAuthority::Customs => crate::ct_rpc_capnp::EncounterAuthority::Customs,
        EncounterAuthority::Naval => crate::ct_rpc_capnp::EncounterAuthority::Naval,
        EncounterAuthority::Warrant => crate::ct_rpc_capnp::EncounterAuthority::Warrant,
    });
    builder.set_threat(match value.threat {
        EncounterThreat::Unknown => crate::ct_rpc_capnp::EncounterThreat::Unknown,
        EncounterThreat::Favorable => crate::ct_rpc_capnp::EncounterThreat::Favorable,
        EncounterThreat::Comparable => crate::ct_rpc_capnp::EncounterThreat::Comparable,
        EncounterThreat::Dangerous => crate::ct_rpc_capnp::EncounterThreat::Dangerous,
        EncounterThreat::Overwhelming => crate::ct_rpc_capnp::EncounterThreat::Overwhelming,
    });
    builder.set_standing_orders_used(value.standing_orders_used);
    builder.set_has_posture(value.posture.is_some());
    builder.set_posture(encode_encounter_posture(
        value.posture.unwrap_or(EncounterPosture::Fight),
    ));
    let mut fallbacks = builder
        .reborrow()
        .init_fallbacks(value.fallbacks.len() as u32);
    for (index, fallback) in value.fallbacks.iter().enumerate() {
        fallbacks.set(index as u32, encode_encounter_fallback(*fallback));
    }
    builder.set_automated_combat_used(value.automated_combat_used);
    builder.set_outcome(&value.outcome);
    builder.set_ship_name(&value.ship_name);
    builder.set_loss_kind(match value.loss_kind {
        CommandLossKind::Destroyed => crate::ct_rpc_capnp::CommandLossKind::Destroyed,
        CommandLossKind::Captured => crate::ct_rpc_capnp::CommandLossKind::Captured,
        CommandLossKind::Surrendered => crate::ct_rpc_capnp::CommandLossKind::Surrendered,
        CommandLossKind::Abandoned => crate::ct_rpc_capnp::CommandLossKind::Abandoned,
        CommandLossKind::Bankruptcy => crate::ct_rpc_capnp::CommandLossKind::Bankruptcy,
    });
    builder.set_owned_cargo_lost_millitons(value.owned_cargo_lost_millitons);
    builder.set_entrusted_cargo_lost_millitons(value.entrusted_cargo_lost_millitons);
    builder.set_unique_objects_lost(value.unique_objects_lost);
    builder.set_fuel_lost_millitons(value.fuel_lost_millitons);
    builder.set_passengers_affected(value.passengers_affected);
    builder.set_damage_hits(value.damage_hits);
    builder.set_captain_name(&value.captain_name);
    builder.set_captain_fate(match value.captain_fate {
        CaptainFate::Survived => crate::ct_rpc_capnp::CaptainFate::Survived,
        CaptainFate::Dead => crate::ct_rpc_capnp::CaptainFate::Dead,
    });
    builder.set_other_crew_total(value.other_crew_total);
    builder.set_other_crew_dead(value.other_crew_dead);
    builder.set_other_crew_injured(value.other_crew_injured);
    builder.set_other_crew_surviving(value.other_crew_surviving);
    builder.set_recovery_ready_second(value.recovery_ready_second);
    builder.set_successor_required(value.successor_required);
    let count = u32::try_from(value.incident_log.len())
        .map_err(|_| WireError::Expected("fewer terminal incident log entries"))?;
    let mut log = builder.init_incident_log(count);
    for (index, line) in value.incident_log.iter().enumerate() {
        log.set(index as u32, line);
    }
    Ok(())
}

pub fn set_operational_damage_report(
    mut builder: crate::ct_rpc_capnp::operational_damage_report::Builder<'_>,
    value: &OperationalDamageReport,
) {
    builder.set_present(value.present);
    builder.set_report_id(value.report_id);
    builder.set_occurred_second(value.occurred_second);
    builder.set_ship_id(value.ship_id);
    builder.set_ship_name(&value.ship_name);
    builder.set_cause(match value.cause {
        OperationalDamageCause::JumpTransition => {
            crate::ct_rpc_capnp::OperationalDamageCause::JumpTransition
        }
        OperationalDamageCause::FuelProcessing => {
            crate::ct_rpc_capnp::OperationalDamageCause::FuelProcessing
        }
        OperationalDamageCause::MaintenanceNeglect => {
            crate::ct_rpc_capnp::OperationalDamageCause::MaintenanceNeglect
        }
    });
    builder.set_origin_system_id(value.origin_system_id);
    builder.set_origin_system_name(&value.origin_system_name);
    builder.set_destination_system_id(value.destination_system_id);
    builder.set_destination_system_name(&value.destination_system_name);
    builder.set_inaccurate_extra_days(value.inaccurate_extra_days);
    builder.set_misjump(value.misjump);
    builder.set_subsystem_id(value.subsystem_id);
    builder.set_subsystem_kind(encode_ship_subsystem_kind(value.subsystem_kind));
    builder.set_subsystem_label(&value.subsystem_label);
    builder.set_damage_hits(value.damage_hits);
    builder.set_sustained_hits(value.sustained_hits);
    builder.set_maximum_hits(value.maximum_hits);
    builder.set_operational_effect(&value.operational_effect);
}

fn set_browser_alert_status(
    mut builder: crate::ct_rpc_capnp::browser_alert_status::Builder<'_>,
    status: &crate::web_push::BrowserAlertStatus,
) {
    builder.set_configured(status.configured);
    builder.set_active_devices(status.active_devices);
    builder.set_maximum_devices(status.maximum_devices);
}

fn decode_combat_action_kind(value: crate::ct_rpc_capnp::CombatActionKind) -> CombatActionKind {
    use crate::ct_rpc_capnp::CombatActionKind as Wire;
    match value {
        Wire::Hold => CombatActionKind::Hold,
        Wire::Coordinate => CombatActionKind::Coordinate,
        Wire::IncreaseInitiative => CombatActionKind::IncreaseInitiative,
        Wire::EvasiveManeuvers => CombatActionKind::EvasiveManeuvers,
        Wire::LineUpShot => CombatActionKind::LineUpShot,
        Wire::RangeCheckClose => CombatActionKind::RangeCheckClose,
        Wire::RangeCheckOpen => CombatActionKind::RangeCheckOpen,
        Wire::BreakPursuit => CombatActionKind::BreakPursuit,
        Wire::SensorTargeting => CombatActionKind::SensorTargeting,
        Wire::ElectronicWarfare => CombatActionKind::ElectronicWarfare,
        Wire::DamageControl => CombatActionKind::DamageControl,
        Wire::Attack => CombatActionKind::Attack,
        Wire::Board => CombatActionKind::Board,
        Wire::PrepareJump => CombatActionKind::PrepareJump,
        Wire::LaunchEscapeCraft => CombatActionKind::LaunchEscapeCraft,
        Wire::OfferSurrender => CombatActionKind::OfferSurrender,
        Wire::AcceptSurrender => CombatActionKind::AcceptSurrender,
        Wire::InspectContact => CombatActionKind::InspectContact,
        Wire::Pursuit => CombatActionKind::Pursuit,
    }
}

fn encode_combat_action_kind(value: CombatActionKind) -> crate::ct_rpc_capnp::CombatActionKind {
    use crate::ct_rpc_capnp::CombatActionKind as Wire;
    match value {
        CombatActionKind::Hold => Wire::Hold,
        CombatActionKind::Coordinate => Wire::Coordinate,
        CombatActionKind::IncreaseInitiative => Wire::IncreaseInitiative,
        CombatActionKind::EvasiveManeuvers => Wire::EvasiveManeuvers,
        CombatActionKind::LineUpShot => Wire::LineUpShot,
        CombatActionKind::RangeCheckClose => Wire::RangeCheckClose,
        CombatActionKind::RangeCheckOpen => Wire::RangeCheckOpen,
        CombatActionKind::BreakPursuit => Wire::BreakPursuit,
        CombatActionKind::SensorTargeting => Wire::SensorTargeting,
        CombatActionKind::ElectronicWarfare => Wire::ElectronicWarfare,
        CombatActionKind::DamageControl => Wire::DamageControl,
        CombatActionKind::Attack => Wire::Attack,
        CombatActionKind::Board => Wire::Board,
        CombatActionKind::PrepareJump => Wire::PrepareJump,
        CombatActionKind::LaunchEscapeCraft => Wire::LaunchEscapeCraft,
        CombatActionKind::OfferSurrender => Wire::OfferSurrender,
        CombatActionKind::AcceptSurrender => Wire::AcceptSurrender,
        CombatActionKind::InspectContact => Wire::InspectContact,
        CombatActionKind::Pursuit => Wire::Pursuit,
    }
}

fn decode_combat_reaction(value: crate::ct_rpc_capnp::CombatReaction) -> CombatReaction {
    use crate::ct_rpc_capnp::CombatReaction as Wire;
    match value {
        Wire::Dodge => CombatReaction::Dodge,
        Wire::PointDefense => CombatReaction::PointDefense,
        Wire::FireSand => CombatReaction::FireSand,
        Wire::TriggerNuclearDamper => CombatReaction::TriggerNuclearDamper,
        Wire::TriggerMesonScreen => CombatReaction::TriggerMesonScreen,
    }
}

fn encode_combat_reaction(value: CombatReaction) -> crate::ct_rpc_capnp::CombatReaction {
    use crate::ct_rpc_capnp::CombatReaction as Wire;
    match value {
        CombatReaction::Dodge => Wire::Dodge,
        CombatReaction::PointDefense => Wire::PointDefense,
        CombatReaction::FireSand => Wire::FireSand,
        CombatReaction::TriggerNuclearDamper => Wire::TriggerNuclearDamper,
        CombatReaction::TriggerMesonScreen => Wire::TriggerMesonScreen,
    }
}

fn decode_combat_objective(
    value: crate::ct_rpc_capnp::CombatObjective,
) -> crate::combat::Objective {
    use crate::ct_rpc_capnp::CombatObjective as Wire;
    match value {
        Wire::Survive => crate::combat::Objective::Survive,
        Wire::Withdraw => crate::combat::Objective::Withdraw,
        Wire::Defeat => crate::combat::Objective::Defeat,
        Wire::Capture => crate::combat::Objective::Capture,
        Wire::Protect => crate::combat::Objective::Protect,
        Wire::Inspect => crate::combat::Objective::Inspect,
    }
}

fn encode_combat_objective(
    value: crate::combat::Objective,
) -> crate::ct_rpc_capnp::CombatObjective {
    use crate::ct_rpc_capnp::CombatObjective as Wire;
    match value {
        crate::combat::Objective::Survive => Wire::Survive,
        crate::combat::Objective::Withdraw => Wire::Withdraw,
        crate::combat::Objective::Defeat => Wire::Defeat,
        crate::combat::Objective::Capture => Wire::Capture,
        crate::combat::Objective::Protect => Wire::Protect,
        crate::combat::Objective::Inspect => Wire::Inspect,
    }
}

fn decode_combat_order(
    reader: crate::ct_rpc_capnp::combat_order_set::Reader<'_>,
) -> Result<CombatOrderSet, WireError> {
    let actions = reader
        .get_actions()?
        .iter()
        .map(|action| {
            Ok(CombatAction {
                kind: decode_combat_action_kind(action.get_kind()?),
                mount_id: action.get_mount_id(),
                target_vessel_id: action.get_target_vessel_id(),
                actor_person_id: action.get_actor_person_id(),
            })
        })
        .collect::<Result<Vec<_>, WireError>>()?;
    let reactions = reader
        .get_reactions()?
        .iter()
        .map(|reaction| {
            Ok::<CombatReactionOrder, WireError>(CombatReactionOrder {
                kind: decode_combat_reaction(reaction.get_kind()?),
                actor_person_id: reaction.get_actor_person_id(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CombatOrderSet {
        combat_id: reader.get_combat_id(),
        view_revision: reader.get_view_revision(),
        actions,
        reactions,
        use_tactical_controller: reader.get_use_tactical_controller(),
        speed_adjustment: reader.get_speed_adjustment(),
        speed_actor_person_id: reader.get_speed_actor_person_id(),
    })
}

fn set_combat_order(
    mut builder: crate::ct_rpc_capnp::combat_order_set::Builder<'_>,
    order: &CombatOrderSet,
) -> Result<(), WireError> {
    builder.set_combat_id(order.combat_id);
    builder.set_view_revision(order.view_revision);
    builder.set_use_tactical_controller(order.use_tactical_controller);
    builder.set_speed_adjustment(order.speed_adjustment);
    builder.set_speed_actor_person_id(order.speed_actor_person_id);
    let mut actions = builder.reborrow().init_actions(
        u32::try_from(order.actions.len())
            .map_err(|_| WireError::Expected("fewer combat actions"))?,
    );
    for (index, action) in order.actions.iter().enumerate() {
        let mut item = actions.reborrow().get(index as u32);
        item.set_kind(encode_combat_action_kind(action.kind));
        item.set_mount_id(action.mount_id);
        item.set_target_vessel_id(action.target_vessel_id);
        item.set_actor_person_id(action.actor_person_id);
    }
    let mut reactions = builder.init_reactions(
        u32::try_from(order.reactions.len())
            .map_err(|_| WireError::Expected("fewer combat reactions"))?,
    );
    for (index, reaction) in order.reactions.iter().enumerate() {
        let mut item = reactions.reborrow().get(index as u32);
        item.set_kind(encode_combat_reaction(reaction.kind));
        item.set_actor_person_id(reaction.actor_person_id);
    }
    Ok(())
}

fn set_combat_policy(
    mut builder: crate::ct_rpc_capnp::combat_automation_policy::Builder<'_>,
    policy: &CombatAutomationPolicy,
) {
    builder.set_expected_revision(policy.expected_revision);
    builder.set_minimum_victory_percent(policy.minimum_victory_percent);
    builder.set_objective(encode_combat_objective(policy.objective));
    builder.set_permit_surrender(policy.permit_surrender);
    builder.set_permit_abandon_ship(policy.permit_abandon_ship);
}

fn combat_role_allows_action(actor: &CombatActor, kind: CombatActionKind) -> bool {
    if actor.captain {
        return true;
    }
    let role = actor.role_kind;
    match kind {
        CombatActionKind::Hold => true,
        CombatActionKind::Coordinate
        | CombatActionKind::IncreaseInitiative
        | CombatActionKind::OfferSurrender
        | CombatActionKind::AcceptSurrender
        | CombatActionKind::LaunchEscapeCraft => false,
        CombatActionKind::EvasiveManeuvers
        | CombatActionKind::LineUpShot
        | CombatActionKind::BreakPursuit
        | CombatActionKind::Pursuit => role == CrewRoleKind::Pilot,
        CombatActionKind::RangeCheckClose
        | CombatActionKind::RangeCheckOpen
        | CombatActionKind::PrepareJump => role == CrewRoleKind::Navigator,
        CombatActionKind::SensorTargeting
        | CombatActionKind::ElectronicWarfare
        | CombatActionKind::InspectContact => role == CrewRoleKind::SensorsOperator,
        CombatActionKind::DamageControl => role == CrewRoleKind::Engineer,
        CombatActionKind::Attack => matches!(
            role,
            CrewRoleKind::Gunner | CrewRoleKind::TurretGunner | CrewRoleKind::BayGunner
        ),
        CombatActionKind::Board => role == CrewRoleKind::Marine,
    }
}

fn combat_role_allows_reaction(actor: &CombatActor, kind: CombatReaction) -> bool {
    if actor.captain {
        return true;
    }
    let role = actor.role_kind;
    match kind {
        CombatReaction::Dodge => role == CrewRoleKind::Pilot,
        _ => matches!(
            role,
            CrewRoleKind::Gunner | CrewRoleKind::TurretGunner | CrewRoleKind::BayGunner
        ),
    }
}

fn set_combat_snapshot(
    mut builder: crate::ct_rpc_capnp::combat_snapshot::Builder<'_>,
    snapshot: &CombatSnapshot,
) -> Result<(), WireError> {
    builder.set_combat_id(snapshot.combat_id);
    builder.set_revision(snapshot.revision);
    builder.set_round(snapshot.round);
    builder.set_round_started_second(snapshot.round_started_second);
    builder.set_order_due_second(snapshot.order_due_second);
    builder.set_order_window_real_milliseconds(snapshot.order_window_real_milliseconds);
    builder.set_range(match snapshot.range {
        crate::combat::RangeBand::Adjacent => crate::ct_rpc_capnp::CombatRange::Adjacent,
        crate::combat::RangeBand::Close => crate::ct_rpc_capnp::CombatRange::Close,
        crate::combat::RangeBand::Short => crate::ct_rpc_capnp::CombatRange::Short,
        crate::combat::RangeBand::Medium => crate::ct_rpc_capnp::CombatRange::Medium,
        crate::combat::RangeBand::Long => crate::ct_rpc_capnp::CombatRange::Long,
        crate::combat::RangeBand::VeryLong => crate::ct_rpc_capnp::CombatRange::VeryLong,
        crate::combat::RangeBand::Distant => crate::ct_rpc_capnp::CombatRange::Distant,
    });
    let mut participants = builder.reborrow().init_participants(
        u32::try_from(snapshot.participants.len())
            .map_err(|_| WireError::Expected("fewer combat participants"))?,
    );
    for (index, participant) in snapshot.participants.iter().enumerate() {
        let mut item = participants.reborrow().get(index as u32);
        item.set_vessel_id(participant.vessel_id);
        item.set_side(participant.side);
        item.set_name(&participant.name);
        item.set_class_name(&participant.class_name);
        item.set_initiative(participant.initiative);
        item.set_thrust(participant.thrust);
        item.set_hull_remaining(participant.hull_remaining);
        item.set_structure_remaining(participant.structure_remaining);
        item.set_armor_remaining(participant.armor_remaining);
        item.set_disposition(match participant.disposition {
            crate::combat::VesselDisposition::Active => {
                crate::ct_rpc_capnp::CombatDisposition::Active
            }
            crate::combat::VesselDisposition::Withdrawing => {
                crate::ct_rpc_capnp::CombatDisposition::Withdrawing
            }
            crate::combat::VesselDisposition::SurrenderOffered => {
                crate::ct_rpc_capnp::CombatDisposition::SurrenderOffered
            }
            crate::combat::VesselDisposition::Surrendered => {
                crate::ct_rpc_capnp::CombatDisposition::Surrendered
            }
            crate::combat::VesselDisposition::Abandoned => {
                crate::ct_rpc_capnp::CombatDisposition::Abandoned
            }
            crate::combat::VesselDisposition::Captured => {
                crate::ct_rpc_capnp::CombatDisposition::Captured
            }
            crate::combat::VesselDisposition::Destroyed => {
                crate::ct_rpc_capnp::CombatDisposition::Destroyed
            }
        });
        item.set_commanded(participant.commanded);
        item.set_player_owned(participant.player_owned);
        item.set_online_controlled(participant.online_controlled);
        item.set_speed(participant.speed);
        item.set_pursuit_target_vessel_id(participant.pursuit_target_vessel_id);
        item.set_pursuit_attack_bonus(participant.pursuit_attack_bonus);
        let mut mounts = item.reborrow().init_weapons(
            u32::try_from(participant.weapons.len())
                .map_err(|_| WireError::Expected("fewer combat mounts"))?,
        );
        for (mount_index, mount) in participant.weapons.iter().enumerate() {
            let mut fitted = mounts.reborrow().get(mount_index as u32);
            fitted.set_mount_id(mount.mount_id);
            fitted.set_label(&mount.label);
            fitted.set_damage_hits(mount.damage_hits);
            fitted.set_ammunition_remaining(mount.ammunition_remaining);
            let mut names = fitted.init_weapons(
                u32::try_from(mount.weapons.len())
                    .map_err(|_| WireError::Expected("fewer fitted weapons"))?,
            );
            for (weapon_index, name) in mount.weapons.iter().enumerate() {
                names.set(weapon_index as u32, name);
            }
        }
    }
    set_combat_order(
        builder.reborrow().init_default_order(),
        &snapshot.default_order,
    )?;
    set_combat_policy(builder.reborrow().init_policy(), &snapshot.policy);
    builder.set_player_order_submitted(snapshot.player_order_submitted);
    builder.set_complete(snapshot.complete);
    let mut log = builder.reborrow().init_log(
        u32::try_from(snapshot.log.len()).map_err(|_| WireError::Expected("shorter combat log"))?,
    );
    for (index, line) in snapshot.log.iter().enumerate() {
        log.set(index as u32, line);
    }
    let mut actors = builder.init_actors(
        u32::try_from(snapshot.actors.len())
            .map_err(|_| WireError::Expected("fewer combat actors"))?,
    );
    for (index, actor) in snapshot.actors.iter().enumerate() {
        let mut item = actors.reborrow().get(index as u32);
        item.set_person_id(actor.person_id);
        item.set_name(&actor.name);
        item.set_station(&actor.station);
        item.set_available(actor.available);
        item.set_action_budget(actor.action_budget);
        const ACTIONS: [CombatActionKind; 19] = [
            CombatActionKind::Hold,
            CombatActionKind::Coordinate,
            CombatActionKind::IncreaseInitiative,
            CombatActionKind::EvasiveManeuvers,
            CombatActionKind::LineUpShot,
            CombatActionKind::RangeCheckClose,
            CombatActionKind::RangeCheckOpen,
            CombatActionKind::BreakPursuit,
            CombatActionKind::SensorTargeting,
            CombatActionKind::ElectronicWarfare,
            CombatActionKind::DamageControl,
            CombatActionKind::Attack,
            CombatActionKind::Board,
            CombatActionKind::PrepareJump,
            CombatActionKind::LaunchEscapeCraft,
            CombatActionKind::OfferSurrender,
            CombatActionKind::AcceptSurrender,
            CombatActionKind::InspectContact,
            CombatActionKind::Pursuit,
        ];
        let allowed_actions = ACTIONS
            .iter()
            .copied()
            .filter(|kind| combat_role_allows_action(actor, *kind))
            .collect::<Vec<_>>();
        let mut actions = item
            .reborrow()
            .init_allowed_actions(allowed_actions.len() as u32);
        for (action_index, action) in allowed_actions.iter().enumerate() {
            actions.set(action_index as u32, encode_combat_action_kind(*action));
        }
        const REACTIONS: [CombatReaction; 5] = [
            CombatReaction::Dodge,
            CombatReaction::PointDefense,
            CombatReaction::FireSand,
            CombatReaction::TriggerNuclearDamper,
            CombatReaction::TriggerMesonScreen,
        ];
        let allowed_reactions = REACTIONS
            .iter()
            .copied()
            .filter(|kind| combat_role_allows_reaction(actor, *kind))
            .collect::<Vec<_>>();
        let mut reactions = item.init_allowed_reactions(allowed_reactions.len() as u32);
        for (reaction_index, reaction) in allowed_reactions.iter().enumerate() {
            reactions.set(reaction_index as u32, encode_combat_reaction(*reaction));
        }
    }
    Ok(())
}

fn set_combat_career_snapshot(
    mut builder: crate::ct_rpc_capnp::combat_career_snapshot::Builder<'_>,
    snapshot: &CombatCareerSnapshot,
) -> Result<(), WireError> {
    let state = &snapshot.state;
    builder.set_revision(state.revision);
    builder.set_mode(match state.mode {
        crate::careers::CombatCareerMode::Independent => {
            crate::ct_rpc_capnp::CombatCareerMode::Independent
        }
        crate::careers::CombatCareerMode::Navy => crate::ct_rpc_capnp::CombatCareerMode::Navy,
        crate::careers::CombatCareerMode::Privateer => {
            crate::ct_rpc_capnp::CombatCareerMode::Privateer
        }
        crate::careers::CombatCareerMode::Pirate => crate::ct_rpc_capnp::CombatCareerMode::Pirate,
    });
    builder.set_rank(&snapshot.rank);
    builder.set_service_points(state.service_points);
    builder.set_monthly_salary_credits(snapshot.monthly_salary_credits);
    builder.set_next_naval_board_second(state.next_naval_board_second);
    builder.set_public_heat(state.public_heat);
    builder.set_underworld_standing(state.underworld_standing);
    builder.set_crew_pressure(state.crew_pressure);
    builder.set_local_enforcement_summary(&snapshot.local_enforcement_summary);
    let mut system_contacts = builder.reborrow().init_system_contacts(
        u32::try_from(snapshot.system_contacts.len())
            .map_err(|_| WireError::Expected("fewer system traffic contacts"))?,
    );
    for (index, contact) in snapshot.system_contacts.iter().enumerate() {
        set_traffic_contact(system_contacts.reborrow().get(index as u32), contact);
    }
    let mut contacts = builder.reborrow().init_local_contacts(
        u32::try_from(snapshot.local_contacts.len())
            .map_err(|_| WireError::Expected("fewer local traffic contacts"))?,
    );
    for (index, contact) in snapshot.local_contacts.iter().enumerate() {
        set_traffic_contact(contacts.reborrow().get(index as u32), contact);
    }
    builder.set_has_interception_watch(snapshot.interception_watch.is_some());
    if let Some(watch) = &snapshot.interception_watch {
        let mut target = builder.reborrow().init_interception_watch();
        target.set_started_second(watch.started_second);
        target.set_target_contact_id(watch.target_contact_id);
        target.set_target_catalog_id(watch.target_catalog_id);
        target.set_target_ship_name(&watch.target_ship_name);
        target.set_filter(match watch.filter {
            InterceptionWatchFilterKind::NamedVessel => {
                crate::ct_rpc_capnp::InterceptionWatchFilterKind::NamedVessel
            }
            InterceptionWatchFilterKind::CraftClass => {
                crate::ct_rpc_capnp::InterceptionWatchFilterKind::CraftClass
            }
            InterceptionWatchFilterKind::AllCraft => {
                crate::ct_rpc_capnp::InterceptionWatchFilterKind::AllCraft
            }
        });
        target.set_purpose(match watch.purpose {
            InterceptionPurpose::ArmedAttack => {
                crate::ct_rpc_capnp::InterceptionPurpose::ArmedAttack
            }
            InterceptionPurpose::BoardingInspection => {
                crate::ct_rpc_capnp::InterceptionPurpose::BoardingInspection
            }
            InterceptionPurpose::Arrest => crate::ct_rpc_capnp::InterceptionPurpose::Arrest,
        });
        set_flight_locus(target.init_locus(), watch.locus);
    }
    let mut warrants = builder.reborrow().init_known_warrants(
        u32::try_from(snapshot.known_warrants.len())
            .map_err(|_| WireError::Expected("fewer locally known warrants"))?,
    );
    for (index, warrant) in snapshot.known_warrants.iter().enumerate() {
        let mut target = warrants.reborrow().get(index as u32);
        target.set_warrant_id(warrant.warrant_id);
        target.set_subject_person_id(warrant.subject_person_id);
        target.set_subject_name(&warrant.subject_name);
        target.set_subject_role(&warrant.subject_role);
        target.set_accusation(&warrant.accusation);
        target.set_bounty_credits(warrant.bounty_credits);
        target.set_severity(warrant.severity);
        target.set_evidence_percent(warrant.evidence_percent);
        target.set_issuing_polity_id(warrant.issuing_polity_id);
        target.set_origin_system_id(warrant.origin_system_id);
        target.set_filed_second(warrant.filed_second);
        target.set_associated_ship_id(warrant.associated_ship_id);
        target.set_associated_ship_name(&warrant.associated_ship_name);
        target.set_associated_transponder(&warrant.associated_transponder);
        target.set_last_known_system_id(warrant.last_known_system_id);
        target.set_association(match warrant.association {
            WarrantAssociationKind::Historical => {
                crate::ct_rpc_capnp::WarrantAssociationKind::Historical
            }
            WarrantAssociationKind::ReportedAboard => {
                crate::ct_rpc_capnp::WarrantAssociationKind::ReportedAboard
            }
            WarrantAssociationKind::ConfirmedAboard => {
                crate::ct_rpc_capnp::WarrantAssociationKind::ConfirmedAboard
            }
            WarrantAssociationKind::WantedVessel => {
                crate::ct_rpc_capnp::WarrantAssociationKind::WantedVessel
            }
        });
        target.set_custody(match warrant.custody {
            BountyCustodyState::AtLarge => crate::ct_rpc_capnp::BountyCustodyState::AtLarge,
            BountyCustodyState::HeldAboard => crate::ct_rpc_capnp::BountyCustodyState::HeldAboard,
            BountyCustodyState::Settled => crate::ct_rpc_capnp::BountyCustodyState::Settled,
        });
        target.set_generated_target(warrant.generated_target);
    }
    let mut opportunities = builder.reborrow().init_opportunities(
        u32::try_from(state.opportunities.len())
            .map_err(|_| WireError::Expected("fewer career opportunities"))?,
    );
    for (index, source) in state.opportunities.iter().enumerate() {
        let mut item = opportunities.reborrow().get(index as u32);
        item.set_opportunity_id(source.opportunity_id);
        item.set_kind(match source.kind {
            crate::careers::OpportunityKind::NavalOrder => {
                crate::ct_rpc_capnp::CareerOpportunityKind::NavalOrder
            }
            crate::careers::OpportunityKind::PrivateerCommission => {
                crate::ct_rpc_capnp::CareerOpportunityKind::PrivateerCommission
            }
            crate::careers::OpportunityKind::PirateLead => {
                crate::ct_rpc_capnp::CareerOpportunityKind::PirateLead
            }
            crate::careers::OpportunityKind::PirateCommission => {
                crate::ct_rpc_capnp::CareerOpportunityKind::PirateCommission
            }
        });
        item.set_state(match source.state {
            crate::careers::OpportunityState::Offered => {
                crate::ct_rpc_capnp::CareerOpportunityState::Offered
            }
            crate::careers::OpportunityState::Accepted => {
                crate::ct_rpc_capnp::CareerOpportunityState::Accepted
            }
            crate::careers::OpportunityState::Succeeded => {
                crate::ct_rpc_capnp::CareerOpportunityState::Succeeded
            }
            crate::careers::OpportunityState::Failed => {
                crate::ct_rpc_capnp::CareerOpportunityState::Failed
            }
            crate::careers::OpportunityState::Expired => {
                crate::ct_rpc_capnp::CareerOpportunityState::Expired
            }
            crate::careers::OpportunityState::Reporting => {
                crate::ct_rpc_capnp::CareerOpportunityState::Reporting
            }
        });
        item.set_issued_system_id(source.issued_system_id);
        item.set_target_system_id(source.target_system_id);
        item.set_target_contact_id(source.target_contact_id);
        item.set_issued_second(source.issued_second);
        item.set_expires_second(source.expires_second);
        item.set_reward_credits(source.reward_credits);
        item.set_service_points(source.service_points);
        item.set_authority(&source.authority);
        item.set_objective(&source.objective);
        item.set_objective_kind(match source.objective_kind {
            crate::careers::ObjectiveKind::Patrol => {
                crate::ct_rpc_capnp::CareerObjectiveKind::Patrol
            }
            crate::careers::ObjectiveKind::Inspect => {
                crate::ct_rpc_capnp::CareerObjectiveKind::Inspect
            }
            crate::careers::ObjectiveKind::Escort => {
                crate::ct_rpc_capnp::CareerObjectiveKind::Escort
            }
            crate::careers::ObjectiveKind::Intercept => {
                crate::ct_rpc_capnp::CareerObjectiveKind::Intercept
            }
            crate::careers::ObjectiveKind::Capture => {
                crate::ct_rpc_capnp::CareerObjectiveKind::Capture
            }
            crate::careers::ObjectiveKind::SeizeCargo => {
                crate::ct_rpc_capnp::CareerObjectiveKind::SeizeCargo
            }
        });
        item.set_evidence_kind(match source.evidence_kind {
            crate::careers::ObjectiveEvidenceKind::None => {
                crate::ct_rpc_capnp::CareerObjectiveEvidenceKind::None
            }
            crate::careers::ObjectiveEvidenceKind::PatrolLog => {
                crate::ct_rpc_capnp::CareerObjectiveEvidenceKind::PatrolLog
            }
            crate::careers::ObjectiveEvidenceKind::InspectionRecord => {
                crate::ct_rpc_capnp::CareerObjectiveEvidenceKind::InspectionRecord
            }
            crate::careers::ObjectiveEvidenceKind::EscortRelease => {
                crate::ct_rpc_capnp::CareerObjectiveEvidenceKind::EscortRelease
            }
            crate::careers::ObjectiveEvidenceKind::TargetDrivenOff => {
                crate::ct_rpc_capnp::CareerObjectiveEvidenceKind::TargetDrivenOff
            }
            crate::careers::ObjectiveEvidenceKind::TargetCaptured => {
                crate::ct_rpc_capnp::CareerObjectiveEvidenceKind::TargetCaptured
            }
            crate::careers::ObjectiveEvidenceKind::CargoSecured => {
                crate::ct_rpc_capnp::CareerObjectiveEvidenceKind::CargoSecured
            }
            crate::careers::ObjectiveEvidenceKind::CargoDelivered => {
                crate::ct_rpc_capnp::CareerObjectiveEvidenceKind::CargoDelivered
            }
        });
        item.set_evidence_second(source.evidence_second);
        item.set_evidence_vessel_id(source.evidence_vessel_id);
        item.set_order_message_id(source.order_message_id);
        item.set_report_message_id(source.report_message_id);
    }
    let mut prizes = builder.reborrow().init_prizes(
        u32::try_from(state.prizes.len()).map_err(|_| WireError::Expected("fewer prizes"))?,
    );
    for (index, source) in state.prizes.iter().enumerate() {
        let mut item = prizes.reborrow().get(index as u32);
        item.set_prize_id(source.prize_id);
        item.set_captured_vessel_id(source.captured_vessel_id);
        item.set_surviving_crew_count(
            u16::try_from(source.captured_person_ids.len()).unwrap_or(u16::MAX),
        );
        item.set_catalog_id(source.catalog_id);
        item.set_name(&source.name);
        item.set_gross_value_credits(source.gross_value_credits);
        item.set_realizable_value_credits(source.realizable_value_credits);
        item.set_condition_percent(source.condition_percent);
        item.set_status(match source.status {
            crate::careers::PrizeStatus::Secured => crate::ct_rpc_capnp::PrizeStatus::Secured,
            crate::careers::PrizeStatus::ClaimInTransit => {
                crate::ct_rpc_capnp::PrizeStatus::ClaimInTransit
            }
            crate::careers::PrizeStatus::AwaitingAdjudication => {
                crate::ct_rpc_capnp::PrizeStatus::AwaitingAdjudication
            }
            crate::careers::PrizeStatus::Adjudicated => {
                crate::ct_rpc_capnp::PrizeStatus::Adjudicated
            }
            crate::careers::PrizeStatus::ReadyToFence => {
                crate::ct_rpc_capnp::PrizeStatus::ReadyToFence
            }
            crate::careers::PrizeStatus::Settled => crate::ct_rpc_capnp::PrizeStatus::Settled,
            crate::careers::PrizeStatus::Seized => crate::ct_rpc_capnp::PrizeStatus::Seized,
        });
        item.set_secured_second(source.secured_second);
        item.set_claim_message_id(source.claim_message_id);
        item.set_settlement_credits(source.settlement_credits);
        item.set_advance_credits(source.advance_credits);
    }
    let mut warrants = builder.reborrow().init_warrants(
        u32::try_from(state.warrants.len()).map_err(|_| WireError::Expected("fewer warrants"))?,
    );
    for (index, source) in state.warrants.iter().enumerate() {
        let mut item = warrants.reborrow().get(index as u32);
        item.set_warrant_id(source.warrant_id);
        item.set_issuing_polity_id(source.issuing_polity_id);
        item.set_origin_system_id(source.origin_system_id);
        item.set_filed_second(source.filed_second);
        item.set_message_id(source.message_id);
        item.set_severity(source.severity);
        item.set_bounty_credits(source.bounty_credits);
        item.set_evidence_percent(source.evidence_percent);
        item.set_status(match source.status {
            crate::careers::WarrantStatus::Filed => crate::ct_rpc_capnp::WarrantStatus::Filed,
            crate::careers::WarrantStatus::Propagating => {
                crate::ct_rpc_capnp::WarrantStatus::Propagating
            }
            crate::careers::WarrantStatus::Active => crate::ct_rpc_capnp::WarrantStatus::Active,
            crate::careers::WarrantStatus::Revoked => crate::ct_rpc_capnp::WarrantStatus::Revoked,
            crate::careers::WarrantStatus::Satisfied => {
                crate::ct_rpc_capnp::WarrantStatus::Satisfied
            }
        });
        item.set_accusation(&source.accusation);
        item.set_resolution_message_id(source.resolution_message_id);
        item.set_resolved_second(source.resolved_second);
        item.set_resolving_system_id(source.resolving_system_id);
    }
    let mut cruise = builder.init_cruise();
    cruise.set_revision(state.cruise.revision);
    cruise.set_active(state.cruise.active);
    cruise.set_hunting_system_id(state.cruise.hunting_system_id);
    cruise.set_ends_second(state.cruise.ends_second);
    cruise.set_crew_share_percent(state.cruise.crew_share_percent);
    cruise.set_ship_fund_percent(state.cruise.ship_fund_percent);
    cruise.set_prohibited_targets(&state.cruise.prohibited_targets);
    Ok(())
}

fn encode_message_class(class: MessageClass) -> crate::ct_rpc_capnp::MessageClass {
    match class {
        MessageClass::AgencyNews => crate::ct_rpc_capnp::MessageClass::AgencyNews,
        MessageClass::PublicService => crate::ct_rpc_capnp::MessageClass::PublicService,
        MessageClass::ContractOffer => crate::ct_rpc_capnp::MessageClass::ContractOffer,
        MessageClass::TrafficNotice => crate::ct_rpc_capnp::MessageClass::TrafficNotice,
        MessageClass::Private => crate::ct_rpc_capnp::MessageClass::PrivateMessage,
    }
}

fn decode_message_class(class: crate::ct_rpc_capnp::MessageClass) -> MessageClass {
    match class {
        crate::ct_rpc_capnp::MessageClass::AgencyNews => MessageClass::AgencyNews,
        crate::ct_rpc_capnp::MessageClass::PublicService => MessageClass::PublicService,
        crate::ct_rpc_capnp::MessageClass::ContractOffer => MessageClass::ContractOffer,
        crate::ct_rpc_capnp::MessageClass::TrafficNotice => MessageClass::TrafficNotice,
        crate::ct_rpc_capnp::MessageClass::PrivateMessage => MessageClass::Private,
    }
}

fn encode_message_importance(
    importance: MessageImportance,
) -> crate::ct_rpc_capnp::MessageImportance {
    match importance {
        MessageImportance::Routine => crate::ct_rpc_capnp::MessageImportance::Routine,
        MessageImportance::Notable => crate::ct_rpc_capnp::MessageImportance::Notable,
        MessageImportance::Important => crate::ct_rpc_capnp::MessageImportance::Important,
        MessageImportance::Headline => crate::ct_rpc_capnp::MessageImportance::Headline,
    }
}

fn decode_message_importance(
    importance: crate::ct_rpc_capnp::MessageImportance,
) -> MessageImportance {
    match importance {
        crate::ct_rpc_capnp::MessageImportance::Routine => MessageImportance::Routine,
        crate::ct_rpc_capnp::MessageImportance::Notable => MessageImportance::Notable,
        crate::ct_rpc_capnp::MessageImportance::Important => MessageImportance::Important,
        crate::ct_rpc_capnp::MessageImportance::Headline => MessageImportance::Headline,
    }
}

fn encode_message_classification(
    classification: MessageClassification,
) -> crate::ct_rpc_capnp::MessageClassification {
    match classification {
        MessageClassification::Unreviewed => crate::ct_rpc_capnp::MessageClassification::Unreviewed,
        MessageClassification::Ignored => crate::ct_rpc_capnp::MessageClassification::Ignored,
        MessageClassification::ReviewLater => {
            crate::ct_rpc_capnp::MessageClassification::ReviewLater
        }
        MessageClassification::Actioned => crate::ct_rpc_capnp::MessageClassification::Actioned,
        MessageClassification::Archived => crate::ct_rpc_capnp::MessageClassification::Archived,
    }
}

fn decode_message_classification(
    classification: crate::ct_rpc_capnp::MessageClassification,
) -> MessageClassification {
    match classification {
        crate::ct_rpc_capnp::MessageClassification::Unreviewed => MessageClassification::Unreviewed,
        crate::ct_rpc_capnp::MessageClassification::Ignored => MessageClassification::Ignored,
        crate::ct_rpc_capnp::MessageClassification::ReviewLater => {
            MessageClassification::ReviewLater
        }
        crate::ct_rpc_capnp::MessageClassification::Actioned => MessageClassification::Actioned,
        crate::ct_rpc_capnp::MessageClassification::Archived => MessageClassification::Archived,
    }
}

fn set_message_item(
    mut builder: crate::ct_rpc_capnp::message_item::Builder<'_>,
    item: &MessageItem,
) {
    builder.set_message_id(item.message_id);
    builder.set_origin_system_id(item.origin_system_id);
    builder.set_origin_system_name(&item.origin_system_name);
    builder.set_created_second(item.created_second);
    builder.set_available_second(item.available_second);
    builder.set_expires_second(item.expires_second);
    builder.set_class(encode_message_class(item.class));
    builder.set_importance(encode_message_importance(item.importance));
    builder.set_subject(&item.subject);
    builder.set_body(&item.body);
    builder.set_offer_id(item.offer_id.unwrap_or(0));
    builder.set_offer_revision(item.offer_revision);
    builder.set_offer_available(item.offer_available);
    builder.set_classification(encode_message_classification(item.classification));
    builder.set_previously_seen(item.previously_seen);
    builder.set_expired(item.expired);
    builder.set_action_kind(match item.action_kind {
        MessageActionKind::None => crate::ct_rpc_capnp::MessageActionKind::None,
        MessageActionKind::ClaimOffer => crate::ct_rpc_capnp::MessageActionKind::ClaimOffer,
        MessageActionKind::ReviewTask => crate::ct_rpc_capnp::MessageActionKind::ReviewTask,
        MessageActionKind::ReviewOperations => {
            crate::ct_rpc_capnp::MessageActionKind::ReviewOperations
        }
        MessageActionKind::ReviewFinance => crate::ct_rpc_capnp::MessageActionKind::ReviewFinance,
        MessageActionKind::ReviewMapping => crate::ct_rpc_capnp::MessageActionKind::ReviewMapping,
    });
    builder.set_action_reference_id(item.action_reference_id);
}

fn set_arrival_packet(
    mut builder: crate::ct_rpc_capnp::arrival_packet::Builder<'_>,
    packet: &ArrivalPacket,
) -> Result<(), WireError> {
    builder.set_system_id(packet.system_id);
    builder.set_system_name(&packet.system_name);
    builder.set_arrival_second(packet.arrival_second);
    builder.set_mailbag_id(packet.mailbag_id.unwrap_or(0));
    builder.set_mail_delivered(packet.mail_delivered);
    builder.set_mail_forwarded(packet.mail_forwarded);
    builder.set_mail_expired(packet.mail_expired);
    builder.set_stipend_credits(packet.stipend_credits);
    builder.set_new_arrival(packet.new_arrival);
    set_system_mapping_status(
        builder.reborrow().init_mapping_status(),
        packet.mapping_status,
    );
    let count = u32::try_from(packet.items.len())
        .map_err(|_| WireError::Expected("fewer arrival messages"))?;
    let mut items = builder.reborrow().init_items(count);
    for (index, item) in packet.items.iter().enumerate() {
        set_message_item(items.reborrow().get(index as u32), item);
    }
    Ok(())
}

fn set_system_mapping_status(
    mut builder: crate::ct_rpc_capnp::system_mapping_status::Builder<'_>,
    status: SystemMappingStatus,
) {
    builder.set_system_id(status.system_id);
    builder.set_state(match status.state {
        SystemMappingState::KnownPublic => crate::ct_rpc_capnp::SystemMappingState::KnownPublic,
        SystemMappingState::Unresolved => crate::ct_rpc_capnp::SystemMappingState::Unresolved,
        SystemMappingState::PublicDispatched => {
            crate::ct_rpc_capnp::SystemMappingState::PublicDispatched
        }
        SystemMappingState::DirectDispatched => {
            crate::ct_rpc_capnp::SystemMappingState::DirectDispatched
        }
        SystemMappingState::Withheld => crate::ct_rpc_capnp::SystemMappingState::Withheld,
        SystemMappingState::Secret => crate::ct_rpc_capnp::SystemMappingState::Secret,
    });
    builder.set_dispatch_message_id(status.dispatch_message_id.unwrap_or(0));
    builder.set_changed_second(status.changed_second);
}

fn set_message_management(
    mut builder: crate::ct_rpc_capnp::message_management::Builder<'_>,
    snapshot: &MessageManagement,
) -> Result<(), WireError> {
    let count = u32::try_from(snapshot.items.len())
        .map_err(|_| WireError::Expected("fewer retained messages"))?;
    {
        let mut items = builder.reborrow().init_items(count);
        for (index, item) in snapshot.items.iter().enumerate() {
            set_message_item(items.reborrow().get(index as u32), item);
        }
    }
    let count = u32::try_from(snapshot.filters.len())
        .map_err(|_| WireError::Expected("fewer message filters"))?;
    let mut filters = builder.init_filters(count);
    for (index, filter) in snapshot.filters.iter().enumerate() {
        let mut item = filters.reborrow().get(index as u32);
        item.set_class(encode_message_class(filter.class));
        item.set_minimum_importance(encode_message_importance(filter.minimum_importance));
    }
    Ok(())
}

fn schema_radio_kind(kind: RadioTransmissionKind) -> crate::ct_rpc_capnp::RadioTransmissionKind {
    match kind {
        RadioTransmissionKind::PlayerBroadcast => {
            crate::ct_rpc_capnp::RadioTransmissionKind::PlayerBroadcast
        }
        RadioTransmissionKind::InspectionOrder => {
            crate::ct_rpc_capnp::RadioTransmissionKind::InspectionOrder
        }
        RadioTransmissionKind::BoardingOrder => {
            crate::ct_rpc_capnp::RadioTransmissionKind::BoardingOrder
        }
        RadioTransmissionKind::PirateDemand => {
            crate::ct_rpc_capnp::RadioTransmissionKind::PirateDemand
        }
    }
}

fn set_system_radio(
    mut builder: crate::ct_rpc_capnp::system_radio_snapshot::Builder<'_>,
    snapshot: &SystemRadioSnapshot,
) -> Result<(), WireError> {
    builder.set_ship_id(snapshot.ship_id);
    builder.set_system_id(snapshot.system_id);
    builder.set_current_second(snapshot.current_second);
    builder.set_can_transmit(snapshot.can_transmit);
    builder.set_unavailable_reason(&snapshot.unavailable_reason);
    let mut entries = builder
        .reborrow()
        .init_entries(snapshot.entries.len() as u32);
    for (index, entry) in snapshot.entries.iter().enumerate() {
        let mut target = entries.reborrow().get(index as u32);
        target.set_reception_id(entry.reception_id);
        target.set_transmission_id(entry.transmission_id);
        target.set_receiving_ship_id(entry.receiving_ship_id);
        target.set_sender_ship_id(entry.sender_ship_id);
        target.set_sender_ship_name(&entry.sender_ship_name);
        target.set_sender_transponder(&entry.sender_transponder);
        let mut sender = target.reborrow().init_sender();
        sender.set_bbs_id(entry.sender.bbs_id);
        sender.set_player_id(entry.sender.player_id);
        target.set_emitted_second(entry.emitted_second);
        target.set_received_second(entry.received_second);
        target.set_expires_second(entry.expires_second);
        target.set_kind(schema_radio_kind(entry.kind));
        target.set_actionable(entry.actionable);
        target.set_action_reference_id(entry.action_reference_id);
    }
    let mut mutes = builder.reborrow().init_mutes(snapshot.mutes.len() as u32);
    for (index, mute) in snapshot.mutes.iter().enumerate() {
        let mut sender = mutes.reborrow().get(index as u32).init_sender();
        sender.set_bbs_id(mute.bbs_id);
        sender.set_player_id(mute.player_id);
    }
    Ok(())
}

fn set_traffic_contact(
    mut builder: crate::ct_rpc_capnp::traffic_contact::Builder<'_>,
    contact: &crate::traffic::TrafficContact,
) {
    builder.set_contact_id(contact.contact_id);
    builder.set_catalog_id(contact.catalog_id);
    builder.set_class_name(&contact.class_name);
    builder.set_ship_name(&contact.ship_name);
    builder.set_transponder(&contact.transponder);
    builder.set_operator_name(&contact.operator_name);
    builder.set_role(&contact.role);
    builder.set_displacement_millitons(contact.displacement_millitons);
    builder.set_origin_system_id(contact.origin_system_id);
    builder.set_destination_system_id(contact.destination_system_id);
    builder.set_movement(match contact.movement {
        crate::traffic::TrafficMovementKind::Arrival => {
            crate::ct_rpc_capnp::TrafficMovementKind::Arrival
        }
        crate::traffic::TrafficMovementKind::Departure => {
            crate::ct_rpc_capnp::TrafficMovementKind::Departure
        }
        crate::traffic::TrafficMovementKind::Present => {
            crate::ct_rpc_capnp::TrafficMovementKind::Present
        }
    });
    builder.set_edge_second(contact.edge_second);
    builder.set_resolution(match contact.resolution {
        crate::traffic::TrafficContactResolution::TransponderOnly => {
            crate::ct_rpc_capnp::TrafficContactResolution::TransponderOnly
        }
        crate::traffic::TrafficContactResolution::Approximate => {
            crate::ct_rpc_capnp::TrafficContactResolution::Approximate
        }
        crate::traffic::TrafficContactResolution::Identified => {
            crate::ct_rpc_capnp::TrafficContactResolution::Identified
        }
    });
    builder.set_confidence_percent(contact.confidence_percent);
    builder.set_player_owned(contact.player_owned);
    builder.set_online_controlled(contact.online_controlled);
    builder.set_attachment(match contact.attachment {
        crate::traffic::TrafficAttachment::Spaceborne => {
            crate::ct_rpc_capnp::TrafficAttachment::Spaceborne
        }
        crate::traffic::TrafficAttachment::Berthed => {
            crate::ct_rpc_capnp::TrafficAttachment::Berthed
        }
        crate::traffic::TrafficAttachment::Landed => crate::ct_rpc_capnp::TrafficAttachment::Landed,
    });
}

fn encode_ship_subsystem_kind(kind: ShipSubsystemKind) -> SchemaShipSubsystemKind {
    match kind {
        ShipSubsystemKind::Hull => SchemaShipSubsystemKind::Hull,
        ShipSubsystemKind::Structure => SchemaShipSubsystemKind::Structure,
        ShipSubsystemKind::Armor => SchemaShipSubsystemKind::Armor,
        ShipSubsystemKind::Bridge => SchemaShipSubsystemKind::Bridge,
        ShipSubsystemKind::Computer => SchemaShipSubsystemKind::Computer,
        ShipSubsystemKind::Sensors => SchemaShipSubsystemKind::Sensors,
        ShipSubsystemKind::JumpDrive => SchemaShipSubsystemKind::JumpDrive,
        ShipSubsystemKind::ManeuverDrive => SchemaShipSubsystemKind::ManeuverDrive,
        ShipSubsystemKind::PowerPlant => SchemaShipSubsystemKind::PowerPlant,
        ShipSubsystemKind::FuelSystem => SchemaShipSubsystemKind::FuelSystem,
        ShipSubsystemKind::LifeSupport => SchemaShipSubsystemKind::LifeSupport,
        ShipSubsystemKind::CargoHold => SchemaShipSubsystemKind::CargoHold,
        ShipSubsystemKind::WeaponMount => SchemaShipSubsystemKind::WeaponMount,
        ShipSubsystemKind::Screen => SchemaShipSubsystemKind::Screen,
        ShipSubsystemKind::Hangar => SchemaShipSubsystemKind::Hangar,
        ShipSubsystemKind::Other => SchemaShipSubsystemKind::Other,
    }
}

fn schema_phase(phase: PlayerPhase) -> Phase {
    match phase {
        PlayerPhase::NewUser => Phase::NewUser,
        PlayerPhase::Docked => Phase::Docked,
        PlayerPhase::Interplanetary => Phase::Interplanetary,
        PlayerPhase::Jump => Phase::Jump,
        PlayerPhase::Encounter => Phase::Encounter,
        PlayerPhase::Terminal => Phase::Terminal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_round_trips_to_owned_data() {
        let identity = PlayerIdentity {
            bbs_id: 17,
            player_id: 42,
        };
        let frame = encode_client_hello(&identity).unwrap();
        let decoded = decode_client_hello(&frame).unwrap();
        assert_eq!(decoded.identity, identity);
        assert_eq!(decoded.client_name, "test");
        assert_eq!(decoded.language_tag, "en");
    }

    #[test]
    fn hello_version_can_be_read_before_it_is_rejected() {
        let identity = PlayerIdentity {
            bbs_id: 17,
            player_id: 42,
        };
        let frame = encode_client_hello_for_version(1, &identity).unwrap();

        let (version, decoded) = decode_client_hello_with_version(&frame).unwrap();
        assert_eq!(version, 1);
        assert_eq!(decoded.identity, identity);
        assert!(matches!(
            decode_client_hello(&frame),
            Err(WireError::UnsupportedVersion(1))
        ));
    }

    #[test]
    fn server_hello_carries_the_negotiated_display_formatting() {
        let identity = PlayerIdentity {
            bbs_id: 17,
            player_id: 42,
        };
        let language = crate::i18n::negotiate_language("en-GB").unwrap();
        let formatting = language.display_formatting();
        let frame = encode_server_hello(
            &identity,
            7,
            9,
            PlayerPhase::Docked,
            language.tag(),
            &formatting,
        )
        .unwrap();
        let message = message_reader(&frame).unwrap();
        let envelope = message.get_root::<envelope::Reader>().unwrap();
        assert_eq!(envelope.get_protocol_version(), PROTOCOL_VERSION);
        let envelope::ServerHello(hello) = envelope.which().unwrap() else {
            panic!("expected server hello");
        };
        let hello = hello.unwrap();
        assert_eq!(hello.get_language_tag().unwrap().to_str().unwrap(), "en-GB");
        let formatting = hello.get_formatting().unwrap();
        assert_eq!(
            formatting
                .get_grouping_separator()
                .unwrap()
                .to_str()
                .unwrap(),
            ","
        );
        assert_eq!(formatting.get_primary_grouping_digits(), 3);
        assert_eq!(
            formatting
                .get_game_timestamp_pattern()
                .unwrap()
                .to_str()
                .unwrap(),
            "Day {day}, {hour}:{minute}:{second}"
        );
    }

    #[test]
    fn compatibility_close_uses_the_clients_protocol_version() {
        let reason = "upgrade your Cepheus Trader client";
        let frame = encode_legacy_close_for_version(1, 0, reason).unwrap();
        let message = message_reader(&frame).unwrap();
        let envelope = message
            .get_root::<crate::ct_rpc_capnp::legacy_v2_envelope::Reader>()
            .unwrap();

        assert_eq!(envelope.get_protocol_version(), 1);
        let crate::ct_rpc_capnp::legacy_v2_envelope::Close(close) = envelope.which().unwrap()
        else {
            panic!("expected close envelope");
        };
        assert_eq!(
            close.unwrap().get_reason().unwrap().to_str().unwrap(),
            reason
        );
    }

    #[test]
    fn structured_close_carries_code_message_and_languages() {
        let frame =
            encode_close_with_code(0, CloseCode::UnsupportedLanguage, "unsupported", &["en"])
                .unwrap();
        let message = message_reader(&frame).unwrap();
        let envelope = message.get_root::<envelope::Reader>().unwrap();
        let envelope::Close(close) = envelope.which().unwrap() else {
            panic!("expected close envelope");
        };
        let close = close.unwrap();
        assert_eq!(
            close.get_code().unwrap(),
            crate::ct_rpc_capnp::CloseCode::UnsupportedLanguage
        );
        assert_eq!(
            close.get_message().unwrap().to_str().unwrap(),
            "unsupported"
        );
        let tags = close.get_supported_language_tags().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags.get(0).unwrap().to_str().unwrap(), "en");
    }

    #[test]
    fn combat_eligibility_does_not_depend_on_display_station() {
        let pilot = CombatActor {
            person_id: 1,
            name: "Pilot".into(),
            station: "Pilote de chasse".into(),
            available: true,
            action_budget: 1,
            role_kind: CrewRoleKind::Pilot,
            captain: false,
        };
        assert!(combat_role_allows_action(
            &pilot,
            CombatActionKind::EvasiveManeuvers
        ));
        assert!(!combat_role_allows_action(&pilot, CombatActionKind::Attack));

        let captain = CombatActor {
            role_kind: CrewRoleKind::Other,
            captain: true,
            station: "Commandant".into(),
            ..pilot
        };
        assert!(combat_role_allows_action(
            &captain,
            CombatActionKind::Attack
        ));
        assert!(combat_role_allows_reaction(
            &captain,
            CombatReaction::PointDefense
        ));
    }

    #[test]
    fn request_round_trips_to_owned_data() {
        for expected in [
            CommandRequest {
                request_id: 17,
                session_epoch: 23,
                command_id: [0xa5; COMMAND_ID_BYTES],
                command: Command::Ping,
            },
            CommandRequest {
                request_id: 18,
                session_epoch: 23,
                command_id: [0xa6; COMMAND_ID_BYTES],
                command: Command::GetCrewManagement,
            },
            CommandRequest {
                request_id: 19,
                session_epoch: 23,
                command_id: [0xa7; COMMAND_ID_BYTES],
                command: Command::SetCrewAssignments {
                    person_id: 81,
                    slot_ids: vec![2, 4, 7],
                },
            },
            CommandRequest {
                request_id: 20,
                session_epoch: 23,
                command_id: [0xa8; COMMAND_ID_BYTES],
                command: Command::GetShipStatus,
            },
            CommandRequest {
                request_id: 201,
                session_epoch: 23,
                command_id: [0xb8; COMMAND_ID_BYTES],
                command: Command::MisappropriateRestrictedCredits { amount: 12_345 },
            },
            CommandRequest {
                request_id: 204,
                session_epoch: 23,
                command_id: [0xbb; COMMAND_ID_BYTES],
                command: Command::CureFinanceDefault,
            },
            CommandRequest {
                request_id: 202,
                session_epoch: 23,
                command_id: [0xb9; COMMAND_ID_BYTES],
                command: Command::SetInterceptionWatch(InterceptionWatchRequest::CraftClass {
                    expected_revision: 17,
                    catalog_id: 72,
                    purpose: InterceptionPurpose::BoardingInspection,
                }),
            },
            CommandRequest {
                request_id: 203,
                session_epoch: 23,
                command_id: [0xba; COMMAND_ID_BYTES],
                command: Command::AbandonPlayer {
                    confirmation: "ABANDON EVERYTHING".into(),
                },
            },
            CommandRequest {
                request_id: 21,
                session_epoch: 23,
                command_id: [0xa9; COMMAND_ID_BYTES],
                command: Command::BuyCargo {
                    market_revision: 7,
                    offer_id: 91,
                    quantity_millitons: 12_000,
                },
            },
            CommandRequest {
                request_id: 22,
                session_epoch: 23,
                command_id: [0xaa; COMMAND_ID_BYTES],
                command: Command::BeginVoyage {
                    destination_system_id: 44,
                },
            },
            CommandRequest {
                request_id: 23,
                session_epoch: 23,
                command_id: [0xab; COMMAND_ID_BYTES],
                command: Command::PlotCourse {
                    origin_system_id: 7,
                    destination_system_id: 44,
                    use_current_fuel: false,
                },
            },
            CommandRequest {
                request_id: 204,
                session_epoch: 23,
                command_id: [0xbb; COMMAND_ID_BYTES],
                command: Command::SuggestTaskCourse,
            },
            CommandRequest {
                request_id: 24,
                session_epoch: 23,
                command_id: [0xac; COMMAND_ID_BYTES],
                command: Command::SetMessageClassification {
                    message_id: 108,
                    classification: MessageClassification::ReviewLater,
                },
            },
            CommandRequest {
                request_id: 25,
                session_epoch: 23,
                command_id: [0xad; COMMAND_ID_BYTES],
                command: Command::SetSystemMappingDisclosure {
                    system_id: 44,
                    choice: SystemMappingChoice::DirectEarth,
                },
            },
            CommandRequest {
                request_id: 26,
                session_epoch: 23,
                command_id: [0xae; COMMAND_ID_BYTES],
                command: Command::GetDockedServices,
            },
            CommandRequest {
                request_id: 27,
                session_epoch: 23,
                command_id: [0xaf; COMMAND_ID_BYTES],
                command: Command::CommitDockedService(DockedServiceOrder {
                    expected_ship_revision: 3,
                    kind: DockedServiceOrderKind::Fuel {
                        kind: DockedFuelServiceKind::GasGiant,
                        source_body_id: Some(9),
                        quantity_millitons: 20_000,
                    },
                }),
            },
            CommandRequest {
                request_id: 28,
                session_epoch: 23,
                command_id: [0xb0; COMMAND_ID_BYTES],
                command: Command::GetSystemRadio,
            },
            CommandRequest {
                request_id: 29,
                session_epoch: 23,
                command_id: [0xb1; COMMAND_ID_BYTES],
                command: Command::TransmitSystemRadio {
                    body: "Traffic advisory".into(),
                },
            },
            CommandRequest {
                request_id: 30,
                session_epoch: 23,
                command_id: [0xb2; COMMAND_ID_BYTES],
                command: Command::PeekRadioReception { reception_id: 81 },
            },
            CommandRequest {
                request_id: 31,
                session_epoch: 23,
                command_id: [0xb3; COMMAND_ID_BYTES],
                command: Command::AcknowledgeRadioReception { reception_id: 81 },
            },
            CommandRequest {
                request_id: 32,
                session_epoch: 23,
                command_id: [0xb4; COMMAND_ID_BYTES],
                command: Command::SetRadioMute {
                    sender: PlayerIdentity {
                        bbs_id: 4,
                        player_id: 9,
                    },
                    muted: true,
                },
            },
            CommandRequest {
                request_id: 33,
                session_epoch: 23,
                command_id: [0xb5; COMMAND_ID_BYTES],
                command: Command::CommissionShip { catalog_id: 214 },
            },
            CommandRequest {
                request_id: 34,
                session_epoch: 23,
                command_id: [0xb6; COMMAND_ID_BYTES],
                command: Command::GetBrowserAlertStatus,
            },
            CommandRequest {
                request_id: 35,
                session_epoch: 23,
                command_id: [0xb7; COMMAND_ID_BYTES],
                command: Command::CreateBrowserAlertEnrollment,
            },
            CommandRequest {
                request_id: 36,
                session_epoch: 23,
                command_id: [0xb8; COMMAND_ID_BYTES],
                command: Command::RevokeAllBrowserAlerts,
            },
            CommandRequest {
                request_id: 37,
                session_epoch: 23,
                command_id: [0xb9; COMMAND_ID_BYTES],
                command: Command::GetOperationalDamageReport,
            },
            CommandRequest {
                request_id: 38,
                session_epoch: 23,
                command_id: [0xba; COMMAND_ID_BYTES],
                command: Command::AcknowledgeOperationalDamageReport { report_id: 91 },
            },
            CommandRequest {
                request_id: 39,
                session_epoch: 23,
                command_id: [0xbb; COMMAND_ID_BYTES],
                command: Command::GetAccountLedger(AccountLedgerRequest {
                    before_entry_id: 771,
                    limit: 17,
                    class: AccountTransactionClass::Expense,
                    ship_id: 44,
                }),
            },
        ] {
            let frame = encode_request(&expected).unwrap();
            assert_eq!(decode_request(&frame).unwrap(), expected);
        }
    }

    #[test]
    fn flight_plan_and_encounter_requests_round_trip() {
        let proposal = FlightPlanProposal {
            expected_plan_revision: 7,
            steps: vec![
                FlightPlanStep {
                    locus: FlightLocus::ArrivalLocus {
                        system_id: 11,
                        remote: false,
                    },
                    authority: WaypointAuthority::Through,
                    action: FlightPlanAction::Jump {
                        destination_system_id: 22,
                        navigation: JumpNavigationMethod::CommercialTape,
                        proceed_on_known_bad_plot: true,
                        remote_arrival: false,
                        departure_locus_arrival: true,
                    },
                    terminal: false,
                },
                FlightPlanStep {
                    locus: FlightLocus::Port {
                        system_id: 22,
                        world_id: 2,
                        facility_id: 3,
                    },
                    authority: WaypointAuthority::Hold,
                    action: FlightPlanAction::Dock {
                        world_id: 2,
                        facility_id: 3,
                    },
                    terminal: true,
                },
            ],
            policy: EncounterPolicy {
                hostile_posture: EncounterPosture::Fight,
                hostile_fallbacks: vec![EncounterFallback::BreakOff, EncounterFallback::Surrender],
                comply_with_inspection: true,
                report_distress: true,
                assist_distress: false,
                standing_orders: vec![EncounterStandingOrder {
                    kind: EncounterKind::Military,
                    ordinary_posture: EncounterPosture::Comply,
                    fight_mode: EncounterFightMode::EstimatedAtLeast,
                    minimum_outlook_percent: 65,
                }],
            },
            preserve_active_step: true,
        };
        for command in [
            Command::PreviewFlightPlan(proposal.clone()),
            Command::CommitFlightPlan(CommitFlightPlanRequest {
                proposal: proposal.clone(),
                preview_hash: vec![0x5a; 16],
                acknowledge_warnings: true,
            }),
            Command::ResolveEncounter(ResolveEncounterRequest {
                encounter_id: 91,
                expected_revision: 4,
                posture: EncounterPosture::Board,
                fallbacks: vec![EncounterFallback::JettisonCargo],
            }),
            Command::GetTerminalReport,
            Command::AcknowledgeTerminalReport {
                encounter_id: 91,
                expected_revision: 5,
            },
            Command::GetEncounterPolicyDefault,
            Command::SetEncounterPolicyDefault(SetEncounterPolicyDefaultRequest {
                expected_revision: 6,
                policy: proposal.policy.clone(),
                acknowledge_nonhostile_fight: true,
            }),
        ] {
            let request = CommandRequest {
                request_id: 8,
                session_epoch: 9,
                command_id: [10; COMMAND_ID_BYTES],
                command,
            };
            assert_eq!(
                decode_request(&encode_request(&request).unwrap()).unwrap(),
                request
            );
        }

        let mut legacy_proposal = proposal;
        legacy_proposal.steps.last_mut().unwrap().terminal = false;
        let legacy_request = CommandRequest {
            request_id: 10,
            session_epoch: 9,
            command_id: [11; COMMAND_ID_BYTES],
            command: Command::PreviewFlightPlan(legacy_proposal),
        };
        let decoded = decode_request(&encode_request(&legacy_request).unwrap()).unwrap();
        let Command::PreviewFlightPlan(decoded_proposal) = decoded.command else {
            panic!("expected flight-plan preview request");
        };
        assert!(decoded_proposal.steps.last().unwrap().terminal);
    }

    #[test]
    fn create_player_round_trips_to_owned_data() {
        let mut captain = crate::creation::captain_options().default_captain;
        captain.name = "Alex Mercer".into();
        let plan = crate::creation::crew_plan(1, crate::creation::SETUP_REVISION, 1).unwrap();
        let expected = CommandRequest {
            request_id: 18,
            session_epoch: 24,
            command_id: [0x5a; COMMAND_ID_BYTES],
            command: Command::CreatePlayer(PlayerCreation {
                setup_revision: crate::creation::SETUP_REVISION,
                starting_offer_id: 1,
                captain,
                ship_name: "Far Horizon".into(),
                crew: plan
                    .slots
                    .into_iter()
                    .map(|slot| InitialCrewDraft {
                        slot_id: slot.slot_id,
                        training_skill: slot.default_crew.training.skill,
                        name: slot.default_crew.name,
                    })
                    .collect(),
                refit_option_ids: vec![1],
            }),
        };
        let frame = encode_request(&expected).unwrap();
        assert_eq!(decode_request(&frame).unwrap(), expected);
    }

    #[test]
    fn command_id_length_is_strict() {
        let mut message = Builder::new_default();
        let mut envelope = message.init_root::<envelope::Builder>();
        envelope.set_protocol_version(PROTOCOL_VERSION);
        envelope.init_request().set_command_id(&[1, 2, 3]);
        let frame = finish_message(&message).unwrap();
        assert!(matches!(
            decode_request(&frame),
            Err(WireError::InvalidCommandId)
        ));
    }
}
