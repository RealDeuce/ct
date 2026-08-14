#pragma once

#include "ct/display_format.hpp"

#include <array>
#include <cstdint>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace ct
{

class TlsConnection;

class PlayerRequestRejected : public std::runtime_error {
   public:
      using std::runtime_error::runtime_error;
};

struct PlayerIdentity {
   uint32_t bbs_id;
   uint32_t player_id;

   bool operator==(const PlayerIdentity&) const = default;
};

enum class PlayerPhase {
   Disconnected,
   Jump,
   Interplanetary,
   Encounter,
   OnPlanet,
   NewUser,
   Docked,
   Terminal,
   Other,
};

enum class SkillId : uint8_t {
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
};

struct SkillDefinition {
   SkillId id;
   std::string name;
};

struct SkillPool {
   uint8_t level3;
   uint8_t level2;
   uint8_t level1;
   uint8_t level0;
};

struct SkillRating {
   SkillId skill;
   int8_t level;

   bool operator==(const SkillRating&) const = default;
};

struct SkillTraining {
   SkillId skill;
   uint16_t needed_weeks;
   uint16_t current_weeks;

   bool operator==(const SkillTraining&) const = default;
};

struct Characteristics {
   uint8_t strength;
   uint8_t dexterity;
   uint8_t endurance;
   uint8_t intelligence;
   uint8_t education;
   uint8_t charisma;

   bool operator==(const Characteristics&) const = default;
};

struct PersonDraft {
   std::string name;
   Characteristics characteristics;
   std::vector<SkillRating> skills;
   SkillTraining training;

   bool operator==(const PersonDraft&) const = default;
};

struct InitialCrewDraft {
   uint16_t slot_id;
   std::string name;
   SkillId training_skill;

   bool operator==(const InitialCrewDraft&) const = default;
};

struct ServerHello {
   PlayerIdentity identity;
   uint64_t assigned_epoch;
   uint64_t committed_sequence;
   PlayerPhase phase;
   std::string language_tag;
   DisplayFormatting formatting;
};

struct PlayerCreation {
   uint64_t setup_revision;
   uint32_t starting_offer_id;
   PersonDraft captain;
   std::string ship_name;
   std::vector<InitialCrewDraft> crew;
   std::vector<uint32_t> refit_option_ids;

   bool operator==(const PlayerCreation&) const = default;
};

struct PlayerCreated {
   PlayerCreation creation;
   uint64_t committed_sequence;
};

struct CharacteristicPointBuy {
   uint8_t minimum;
   uint8_t maximum;
   uint8_t neutral;
   int16_t budget;
};

struct CaptainCreationOptions {
   uint64_t setup_revision;
   CharacteristicPointBuy characteristic_point_buy;
   SkillPool skill_pool;
   std::vector<SkillDefinition> permitted_skills;
   PersonDraft default_captain;
};

enum class Career {
   Trader,
   Privateer,
   Navy,
};

struct OriginDossier {
   std::string bbs_name;
   std::string polity_name;
   std::string home_system_name;
   std::string home_world_name;
   uint8_t trade_combat;
   uint8_t chaos_order;
};

struct StartingShipOfferSummary {
   uint32_t offer_id;
   Career career;
   std::string package_name;
   uint32_t ship_catalog_id;
   std::string ship_name;
   std::string role;
   std::string rationale;
   uint32_t displacement_tons;
   uint8_t jump_rating;
   uint8_t thrust_g;
   double cargo_tons;
   uint16_t crew_count;
   uint64_t price_credits;
};

struct StartingShipOffers {
   uint64_t setup_revision;
   OriginDossier origin;
   std::vector<StartingShipOfferSummary> offers;
};

struct StartingShipOptions {
   enum class TitleKind { OwnedWithLien, SponsorOwned, InstitutionOwned };
   struct Terms {
      uint64_t terms_revision;
      TitleKind title;
      uint64_t equity_credits;
      uint64_t principal_credits;
      uint64_t monthly_payment_credits;
      uint64_t liquid_reserve_credits;
      uint64_t restricted_reserve_credits;
      uint64_t monthly_compensation_credits;
      uint64_t refit_credit_limit;
      uint64_t refit_displacement_millitons;
      std::string authority;
      std::string exit_terms;
      std::string insurance;
   };
   struct RefitOption {
      uint32_t option_id;
      std::string name;
      std::string description;
      int64_t displacement_delta_millitons;
      int64_t price_delta_credits;
   };
   struct RefitGroup {
      uint32_t group_id;
      std::string name;
      bool required;
      std::vector<RefitOption> options;
   };
   uint64_t setup_revision;
   StartingShipOfferSummary offer;
   std::vector<std::string> description_paragraphs;
   Terms terms;
   std::vector<RefitGroup> refit_groups;
};

enum class CrewRoleKind : uint8_t {
   Command, Pilot, Navigator, Engineer, SensorsOperator, ScreenOperator,
   TurretGunner, BayGunner, Gunner, Medic, Marine, FlightCrew, Steward, Other,
};

struct StartingCrewSlot {
   uint16_t slot_id;
   std::string role;
   uint16_t represented_positions;
   bool required;
   SkillPool skill_pool;
   PersonDraft default_crew;
   CrewRoleKind role_kind;
};

struct StartingCrewPlan {
   uint64_t setup_revision;
   uint32_t starting_offer_id;
   std::vector<StartingCrewSlot> slots;
};

enum class PersonCondition {
   Fit,
   Fatigued,
   Wounded,
   Incapacitated,
   Dead,
};
enum class CrewServiceKind {
   OwnerCaptain,
   Salaried,
   PrizeShare,
   Institutional,
};
enum class CrewAvailability {
   Active,
   ShoreLeave,
   MedicalCare,
   Detached,
   AwaitingRecall,
};
enum class CrewLocationKind { AboardShip, ShoreFacility };
enum class PersonnelActionKind {
   Dismiss,
   Transfer,
   ShoreLeave,
   Recall,
   FirstAid,
   Surgery,
   MedicalCare,
};
struct CrewManagementMember {
   uint64_t person_id;
   uint16_t slot_id;
   std::string role;
   uint16_t represented_positions;
   bool captain;
   PersonDraft person;
   std::vector<uint16_t> assigned_slot_ids;
   PersonCondition condition;
   uint16_t injury_points;
   uint16_t fatigue_points;
   bool available;
   uint8_t current_strength;
   uint8_t current_dexterity;
   uint8_t current_endurance;
   CrewServiceKind service_kind;
   uint64_t monthly_salary_credits;
   uint64_t arrears_credits;
   uint16_t prize_share_basis_points;
   uint8_t morale;
   uint8_t loyalty;
   uint8_t risk_tolerance;
   CrewAvailability availability;
   uint64_t available_second;
   uint64_t service_revision;
   std::string shore_location;
   CrewRoleKind role_kind;
   CrewLocationKind location_kind;
};

struct CrewRole {
   uint16_t slot_id;
   std::string role;
   uint16_t represented_positions;
   CrewRoleKind role_kind;
};

struct CrewManagementSnapshot {
   uint64_t ship_id;
   std::string ship_name;
   std::vector<CrewManagementMember> members;
   std::vector<CrewRole> roles;
   uint16_t established_complement;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class ShipSubsystemKind {
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
};

struct ShipSubsystemStatus {
   uint16_t subsystem_id;
   ShipSubsystemKind kind;
   std::string label;
   uint16_t maximum_hits;
   uint16_t sustained_hits;
   uint16_t battlefield_repair_hits;
   uint16_t effective_hits;
   std::string operational_effect;
   uint64_t last_proper_repair_second;
   uint64_t installed_second;
   uint64_t last_refit_second;
   uint32_t calendar_age_months;
   uint64_t operating_seconds;
   uint32_t duty_cycles;
   uint32_t skimming_cycles;
   uint16_t neglect_damage_hits;
   uint64_t displacement_millitons;
   uint64_t replacement_price_credits;
   uint16_t installation_generation;
   bool reconditioned;
};

struct ShipAmmunitionStatus {
   std::string ammunition_id;
   uint32_t remaining;
   uint32_t capacity;
   uint32_t pack_units;
   uint64_t price_per_pack_credits;
};

struct ShipProvisionStatus {
   uint64_t person_days_remaining;
   uint64_t capacity_person_days;
};

enum class ShipActivityKind {
   Refit,
   Refurbishment,
   ProperRepair,
   GasGiantSkim,
   WildernessWater,
   EscortDuty,
   FieldRecovery,
};

struct ShipActivityStatus {
   uint64_t activity_id;
   ShipActivityKind kind;
   uint16_t subsystem_id;
   uint64_t quantity_millitons;
   uint64_t opportunity_id;
   uint64_t started_second;
   uint64_t due_second;
   uint64_t cost_credits;
   std::optional<uint32_t> source_body_id;
};

struct ShipStatusSnapshot {
   uint64_t ship_revision;
   uint64_t ship_id;
   std::string ship_name;
   uint32_t catalog_id;
   uint64_t catalog_revision;
   uint64_t system_id;
   uint64_t current_game_second;
   uint64_t displacement_millitons;
   uint8_t jump_rating;
   uint8_t thrust_g;
   uint64_t fuel_capacity_millitons;
   uint64_t current_fuel_millitons;
   uint64_t jump_fuel_millitons;
   uint64_t cargo_capacity_millitons;
   uint64_t monthly_maintenance_credits;
   uint64_t next_maintenance_second;
   uint64_t maintenance_paid_through_second;
   uint64_t maintenance_arrears_credits;
   uint32_t completed_maintenance_cycles;
   uint16_t consecutive_missed_maintenance;
   uint64_t commissioned_second;
   uint32_t transit_count;
   uint64_t warranty_expires_second;
   uint32_t warranty_transit_limit;
   uint16_t warranty_repairs;
   uint64_t last_refit_second;
   uint16_t completed_refits;
   std::optional<ShipActivityStatus> active_activity;
   uint64_t unrefined_fuel_millitons;
   bool warranty_voided;
   uint64_t monthly_life_support_credits;
   std::string recovery_status;
   std::vector<ShipAmmunitionStatus> ammunition;
   ShipProvisionStatus provisions;
   std::vector<std::string> manifested_symptoms;
   std::vector<ShipSubsystemStatus> subsystems;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class DockedFuelServiceKind {
   Refined,
   Unrefined,
   GasGiant,
   WildernessWater,
};
struct DockedFuelService {
   DockedFuelServiceKind kind;
   std::string label;
   std::optional<uint32_t> source_body_id;
   bool available;
   std::string unavailable_reason;
   uint64_t price_per_ton_credits;
   uint64_t maximum_millitons;
   uint64_t service_seconds;
};
struct DockedRepairService {
   uint16_t subsystem_id;
   std::string label;
   bool available;
   std::string unavailable_reason;
   uint64_t cost_credits;
   uint64_t service_seconds;
   bool replacement;
   bool reconditioned;
};
struct DockedServices {
   uint64_t ship_revision;
   uint64_t current_game_second;
   std::vector<DockedFuelService> fuel;
   std::vector<ShipAmmunitionStatus> ammunition;
   ShipProvisionStatus provisions;
   uint64_t provision_package_person_days;
   uint64_t provision_package_price_credits;
   bool provisions_available;
   bool ammunition_available;
   std::vector<DockedRepairService> repair;
   bool refit_available;
   std::string refit_unavailable_reason;
   uint64_t refit_cost_credits;
   uint64_t refit_service_seconds;
};
struct DockedServiceOrder {
   enum class Kind { Fuel, Ammunition, Provisions, ProperRepair, Refit, Replacement } kind;
   uint64_t expected_ship_revision = 0;
   DockedFuelServiceKind fuel_kind = DockedFuelServiceKind::Refined;
   std::optional<uint32_t> source_body_id;
   uint64_t quantity_millitons = 0;
   std::string ammunition_id;
   uint32_t packs = 0;
   uint16_t packages = 0;
   uint16_t subsystem_id = 0;
   bool reconditioned = false;
};

struct DockedSnapshot {
   uint64_t ship_id;
   std::string ship_name;
   uint64_t system_id;
   std::string system_name;
   uint64_t world_id;
   std::string world_name;
   uint64_t facility_id;
   std::string facility_name;
   std::string starport;
   uint8_t tech_level;
   uint8_t population;
   uint8_t law_level;
   uint64_t arrived_second;
   uint64_t credits;
   uint64_t restricted_credits;
   uint64_t debt_credits;
   uint64_t fuel_millitons;
   uint64_t fuel_capacity_millitons;
   uint64_t refined_fuel_price_per_ton;
   uint64_t unrefined_fuel_millitons;
   uint64_t unrefined_fuel_price_per_ton;
   uint64_t accrued_berth_fee_credits;
   uint64_t facility_revision;
   bool personnel_available;
   bool banking_available;
   bool authority_available;
   uint8_t medical_level;
   bool clearance_required;
   uint64_t cargo_used_millitons;
   uint64_t cargo_capacity_millitons;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class SystemKnowledgeSource : uint8_t {
   PublishedRecords, CarriedRecords, PrivateObservation, PublicDispatch,
   DirectDispatch, Withheld, SecretChart,
};

struct KnownSystemSummary {
   uint64_t system_id;
   std::string system_name;
   std::string world_name;
   double distance_parsecs;
   bool within_jump_rating;
   std::string starport;
   uint8_t population;
   uint8_t tech_level;
   uint64_t observed_second;
   std::string source;
   double coreward_parsecs;
   double spinward_parsecs;
   double north_parsecs;
   bool remote_candidate;
   SystemKnowledgeSource knowledge_source;
   uint8_t gas_giant_count;
};

struct KnownDestinations {
   uint64_t current_system_id;
   uint8_t jump_rating;
   std::vector<KnownSystemSummary> systems;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class CourseFuelSource {
   None,
   Carried,
   RefinedPort,
   FrontierSkimming,
   UnrefinedPort,
};

struct CourseWaypoint {
   uint64_t system_id;
   std::string system_name;
   std::string world_name;
   CourseFuelSource fuel_source;
   uint64_t next_leg_milliparsecs;
};

struct CoursePlan {
   bool available;
   uint64_t elapsed_seconds;
   uint64_t fuel_cost_credits;
   uint64_t total_milliparsecs;
   std::vector<CourseWaypoint> waypoints;
};

struct CoursePlot {
   uint64_t origin_system_id;
   uint64_t destination_system_id;
   uint8_t jump_rating;
   CoursePlan fastest;
   CoursePlan cheapest;
   uint64_t current_game_second;
   uint64_t clock_rate_game_seconds;
   uint64_t clock_rate_real_seconds;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

struct PriceDistribution {
   uint64_t minimum;
   uint64_t lower_quartile;
   uint64_t median;
   uint64_t upper_quartile;
   uint64_t maximum;
};

struct MarketOffer {
   uint64_t offer_id;
   uint16_t commodity_id;
   std::string commodity_name;
   uint64_t base_price_per_ton;
   uint64_t purchase_price_per_ton;
   uint64_t sale_price_per_ton;
   uint64_t available_millitons;
   uint8_t legality;
   PriceDistribution price_distribution;
};

struct CargoLot {
   uint64_t cargo_lot_id;
   uint16_t commodity_id;
   std::string commodity_name;
   uint64_t quantity_millitons;
   uint64_t purchase_price_per_ton;
   uint64_t origin_system_id;
   uint64_t acquired_second;
   uint8_t title;
   uint64_t task_id;
   uint64_t unique_object_id;
   uint8_t condition_percent;
   uint64_t destination_system_id;
};

struct CargoSaleQuote {
   uint64_t cargo_lot_id;
   uint64_t price_per_ton;
   PriceDistribution price_distribution;
};

enum class TaskKind {
   Freight,
   Passenger,
   PurchaseOrder,
   ForwardSale,
   SupplyCommitment,
   Charter,
   Courier,
   DiscoveryBounty,
   CombatBounty,
};

enum class TaskState {
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
};

enum class PassengerClass {
   None,
   High,
   Middle,
   Steerage,
   Low,
   Charter,
   Courier,
};

enum class TaskActionKind {
   Cancel,
   ReturnCustody,
   DefaultTask,
   FileDispute,
   WithdrawClaim,
};

struct TaskOffer {
   uint64_t offer_id;
   uint64_t revision;
   TaskKind kind;
   std::string title;
   uint64_t origin_system_id;
   uint64_t destination_system_id;
   uint16_t commodity_id;
   uint64_t quantity_millitons;
   uint16_t passenger_count;
   uint64_t payment_credits;
   uint64_t collateral_credits;
   uint64_t expires_second;
   uint64_t delivery_deadline_second;
   bool legal;
   bool partial_delivery_allowed;
   uint64_t failure_penalty_credits;
   uint64_t recurrence_seconds;
   uint16_t performance_count;
   PassengerClass passenger_class;
   uint64_t late_deduction_per_day_credits;
   uint64_t non_delivery_liability_credits;
   uint64_t passenger_grace_seconds;
   uint64_t declared_value_credits;
   std::vector<std::string> unavailable_reasons;
};

struct TaskRecord {
   uint64_t task_id;
   TaskOffer offer;
   TaskState state;
   uint64_t accepted_second;
   uint64_t delivered_quantity_millitons;
   uint64_t reserved_cargo_millitons;
   uint16_t reserved_passenger_count;
   uint64_t reserved_credits;
   std::string status_text;
   uint16_t performances_completed;
   uint64_t revision;
   uint64_t claim_message_id;
   uint64_t result_message_id;
   bool known_result;
   uint64_t loaded_second;
   uint64_t settled_second;
   uint64_t insurance_claim_id;
   uint64_t dispute_message_id;
   int16_t dispute_effect;
   uint64_t adjudication_message_id;
   uint64_t performing_ship_id;
};

struct CarriageDeclaration {
   uint64_t plan_revision = 0;
   uint64_t destination_system_id = 0;
   uint64_t freight_capacity_millitons = 0;
   uint16_t high_berths = 0;
   uint16_t middle_berths = 0;
   uint16_t steerage_berths = 0;
   uint16_t low_berths = 0;
   bool accept_electronic_mail = true;
};

struct TaskRouteAssessment {
   uint64_t offer_id;
   bool pickup_available;
   uint64_t pickup_arrival_second;
   bool delivery_available;
   uint64_t delivery_arrival_second;
};

struct TaskLedger {
   uint64_t current_second;
   uint64_t available_credits;
   uint64_t reserved_credits;
   uint64_t reserved_cargo_millitons;
   uint16_t reserved_passenger_count;
   std::vector<TaskRecord> tasks;
   std::vector<TaskOffer> local_offers;
   CarriageDeclaration carriage;
   std::vector<TaskRouteAssessment> route_assessments;
   PlayerPhase phase;
};

enum class MarketSearchKind {
   Supplier,
   Buyer,
   Freight,
   Passengers,
};

enum class MarketSearchMethod {
   Physical,
   Online,
   BlackMarket,
   HiredBroker,
};

enum class WorkState {
   Scheduled,
   Completed,
   Cancelled,
   Failed,
};
struct WorkAssignment {
   uint64_t assignment_id;
   MarketSearchKind kind;
   MarketSearchMethod method;
   uint64_t person_id;
   uint16_t commodity_id;
   uint64_t destination_system_id;
   uint64_t started_second;
   uint64_t due_second;
   WorkState state;
   std::string result_text;
};
enum class MarketLeadSide {
   Supplier,
   Buyer,
};

enum class MarketLeadState {
   Available,
   Reserved,
   Performed,
   Expired,
   Cancelled,
};

struct MarketLead {
   uint64_t lead_id;
   uint64_t revision;
   MarketLeadSide side;
   MarketLeadState state;
   uint64_t system_id;
   uint16_t commodity_id;
   std::string commodity_name;
   uint64_t quantity_millitons;
   uint64_t price_per_ton;
   uint64_t discovered_second;
   uint64_t expires_second;
   uint64_t reservation_expires_second;
   uint64_t escrow_credits;
   std::string source;
   uint8_t confidence_percent;
};

enum class MarketEventKind {
   Shortage,
   Surplus,
   Disruption,
   Recovery,
};

struct MarketEvent {
   uint64_t event_id;
   MarketEventKind kind;
   uint16_t commodity_id;
   std::string commodity_name;
   uint64_t start_second;
   uint64_t expires_second;
   uint16_t stock_multiplier_basis_points;
   int8_t purchase_tier_delta;
   int8_t sale_tier_delta;
   uint16_t supplier_offer_multiplier_basis_points;
   uint16_t buyer_offer_multiplier_basis_points;
   uint16_t carriage_offer_multiplier_basis_points;
   std::string headline;
};

enum class ShipTitleKind {
   OwnedWithLien,
   SponsorOwned,
   InstitutionOwned,
   OwnedClear,
   PrizeCustody,
   StolenRegistry,
   CourtImpound,
};

enum class ManagedShipOrderKind {
   Hold,
   FollowActive,
   Travel,
   Dock,
   Sell,
};

struct ManagedShipSummary {
   uint64_t ship_id;
   std::string name;
   std::string class_name;
   uint32_t catalog_id;
   uint64_t system_id;
   std::string system_name;
   std::string location;
   ShipTitleKind title;
   bool active;
   uint64_t commanding_person_id;
   std::string commanding_person_name;
   ManagedShipOrderKind standing_order;
   bool can_assume_command;
   uint64_t fuel_millitons;
   uint64_t fuel_capacity_millitons;
   uint64_t cargo_used_millitons;
   uint64_t cargo_capacity_millitons;
   uint64_t provision_person_days;
   uint64_t provision_capacity_person_days;
   std::vector<CargoLot> cargo;
   std::vector<ShipAmmunitionStatus> ammunition;
   bool online_controlled;
};

struct FleetSnapshot {
   uint64_t revision;
   uint64_t active_ship_id;
   std::vector<ManagedShipSummary> ships;
   PlayerPhase phase;
};

enum class StoreTransferKind {
   Cargo,
   Fuel,
   Ammunition,
   Provisions,
};

struct FinanceSnapshot {
   ShipTitleKind title;
   uint64_t liquid_credits;
   uint64_t restricted_credits;
   uint64_t reserved_credits;
   uint64_t original_hull_price_credits;
   uint64_t principal_credits;
   uint64_t monthly_payment_credits;
   uint64_t monthly_insurance_escrow_credits;
   uint64_t next_payment_due_second;
   uint64_t grace_expires_second;
   uint64_t paid_through_second;
   bool in_default;
   bool impound_order_known_locally;
   std::string credit_status;
   bool destination_assistance_active;
   uint64_t destination_assistance_expires_second;
   PlayerPhase phase;
};
struct MarketObservation {
   uint64_t system_id;
   std::string system_name;
   uint16_t commodity_id;
   std::string commodity_name;
   uint64_t observed_second;
   uint64_t acquired_second;
   std::string source;
   uint8_t confidence_percent;
   uint64_t minimum_price_per_ton;
   uint64_t maximum_price_per_ton;
   uint64_t minimum_available_millitons;
   uint64_t maximum_available_millitons;
};
struct MarketKnowledge {
   uint64_t current_second;
   std::vector<MarketObservation> observations;
   PlayerPhase phase;
};
struct ShipMarketOffer {
   uint64_t offer_id;
   uint32_t catalog_id;
   std::string class_name;
   uint64_t price_credits;
   uint64_t original_price_credits;
   bool used;
   uint32_t age_months;
   uint8_t visible_condition_percent;
   uint64_t cargo_capacity_millitons;
   uint8_t jump_rating;
   uint16_t minimum_crew;
};
struct ShipMarket {
   uint64_t generated_day;
   uint64_t current_ship_trade_in_credits;
   uint64_t outstanding_lien_credits;
   std::vector<ShipMarketOffer> offers;
   PlayerPhase phase;
};
struct CrewCandidate {
   uint64_t candidate_id;
   std::string role;
   std::string name;
   SkillId primary_skill;
   int8_t skill_level;
   uint64_t monthly_salary_credits;
};
struct CrewMarket {
   uint64_t generated_day;
   std::vector<CrewCandidate> candidates;
   PlayerPhase phase;
};

struct MarketSnapshot {
   uint64_t market_revision;
   uint64_t system_id;
   std::string world_name;
   uint64_t generated_day;
   uint64_t credits;
   uint64_t cargo_used_millitons;
   uint64_t cargo_capacity_millitons;
   std::vector<MarketOffer> offers;
   std::vector<CargoLot> cargo;
   std::vector<std::string> trade_codes;
   uint16_t tariff_basis_points;
   std::vector<TaskOffer> local_task_offers;
   std::vector<WorkAssignment> work_assignments;
   std::vector<MarketLead> leads;
   std::vector<MarketEvent> events;
   std::vector<CargoSaleQuote> cargo_sale_quotes;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class TravelStage {
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
};

enum class FlightLocusKind {
   Port,
   JumpLocus,
   Body,
   DeepSpace,
};

struct FlightLocus {
   FlightLocusKind kind;
   uint64_t system_id;
   uint64_t world_id;
   uint64_t facility_id;
   uint32_t body_id;
   double coreward_parsecs = 0.0;
   double spinward_parsecs = 0.0;
   double north_parsecs = 0.0;
};

struct TravelStatus {
   uint64_t ship_id;
   std::string ship_name;
   uint64_t current_system_id;
   std::string current_system_name;
   uint64_t destination_system_id;
   std::string destination_system_name;
   TravelStage stage;
   uint64_t current_game_second;
   uint64_t due_second;
   uint64_t current_fuel_millitons;
   uint64_t jump_fuel_millitons;
   uint64_t clock_rate_game_seconds;
   uint64_t clock_rate_real_seconds;
   uint64_t plan_id;
   uint64_t plan_revision;
   uint16_t leg_index;
   FlightLocus origin;
   FlightLocus destination;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class WaypointAuthority {
   Hold,
   Terminal,
   Through,
};

enum class FlightPlanActionKind {
   Hold,
   Jump,
   Dock,
   Fuel,
   JumpCoordinates,
};

enum class JumpNavigationMethod {
   Onboard,
   CommercialTape,
};

enum class FuelOperation {
   GasGiant,
   WildernessWater,
   BuyRefined,
   BuyUnrefined,
};

enum class EncounterPosture {
   Fight,
   Flee,
   Comply,
   Surrender,
   Board,
};

enum class EncounterFallback {
   Surrender,
   Abandon,
   JettisonCargo,
   BreakOff,
};

struct FlightPlanAction {
   FlightPlanActionKind kind = FlightPlanActionKind::Hold;
   uint64_t destination_system_id = 0;
   uint64_t world_id = 0;
   uint64_t facility_id = 0;
   FuelOperation fuel_operation = FuelOperation::GasGiant;
   uint64_t quantity_millitons = 0;
   JumpNavigationMethod jump_navigation = JumpNavigationMethod::Onboard;
   bool proceed_on_known_bad_plot = false;
   double coreward_parsecs = 0.0;
   double spinward_parsecs = 0.0;
   double north_parsecs = 0.0;
};

struct FlightPlanStep {
   FlightLocus locus;
   WaypointAuthority authority;
   FlightPlanAction action;
};
struct EncounterPolicy {
   EncounterPosture hostile_posture = EncounterPosture::Flee;
   std::vector<EncounterFallback> hostile_fallbacks{EncounterFallback::Surrender};
   bool comply_with_inspection = true;
   bool report_distress = true;
   bool assist_distress = false;
};
struct FlightPlanProposal {
   uint64_t expected_plan_revision;
   std::vector<FlightPlanStep> steps;
   EncounterPolicy policy;
};
struct FlightPlanWarning {
   std::string code;
   std::string message;
};
struct FlightPlanPreview {
   FlightPlanProposal proposal;
   std::vector<uint8_t> preview_hash;
   uint64_t elapsed_seconds;
   uint64_t fuel_millitons;
   std::vector<FlightPlanWarning> warnings;
   std::vector<TaskOffer> carriage_offers;
   uint64_t carriage_revenue_credits;
   uint64_t carriage_broker_fees_credits;
};
enum class FlightPlanState {
   Inactive,
   Active,
   Held,
   Checkpoint,
   Encounter,
   Completed,
   Terminal,
};
struct FlightPlanSnapshot {
   uint64_t plan_id;
   uint64_t revision;
   uint16_t current_step;
   FlightPlanState state;
   std::vector<FlightPlanStep> steps;
   EncounterPolicy policy;
   std::string suspension_reason;
   PlayerPhase phase;
};
enum class CheckpointKind {
   PortDeparture,
   InhabitedWorld,
   GasGiant,
   JumpArrival,
   JumpDeparture,
   DeepSpace,
};
struct CheckpointSnapshot {
   uint64_t checkpoint_id;
   uint64_t plan_id;
   uint64_t plan_revision;
   uint16_t step_index;
   FlightLocus locus;
   CheckpointKind kind;
   uint64_t ready_second;
   bool acknowledged;
   PlayerPhase phase;
};
enum class EncounterKind {
   RoutineTraffic,
   TrafficControl,
   Inspection,
   Distress,
   Derelict,
   Hazard,
   Hostile,
   Military,
};

enum class EncounterState {
   AwaitingPosture,
   Resolving,
   Resolved,
};
struct EncounterContact {
   uint64_t contact_id;
   std::string ship_name;
   std::string class_name;
   std::string transponder;
   std::string role;
   std::string range;
   uint8_t confidence_percent;
};
struct EncounterSnapshot {
   uint64_t encounter_id;
   uint64_t revision;
   EncounterKind kind;
   EncounterState state;
   uint64_t started_second;
   uint64_t next_turn_second;
   uint16_t turn;
   EncounterContact contact;
   std::string summary;
   PlayerPhase phase;
};
struct EncounterResult {
   uint64_t encounter_id;
   bool resolved;
   bool terminal;
   std::string outcome;
   uint16_t turns;
   uint64_t cargo_lost_millitons;
   uint64_t fuel_lost_millitons;
   uint16_t damage_hits;
   PlayerPhase phase;
};

enum class CombatRange {
   Adjacent,
   Close,
   Short,
   Medium,
   Long,
   VeryLong,
   Distant,
};

enum class CombatDisposition {
   Active,
   Withdrawing,
   SurrenderOffered,
   Surrendered,
   Abandoned,
   Captured,
   Destroyed,
};

enum class CombatObjective {
   Survive,
   Withdraw,
   Defeat,
   Capture,
   Protect,
   Inspect,
};

enum class CombatActionKind {
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
};

enum class CombatReaction {
   Dodge,
   PointDefense,
   FireSand,
   TriggerNuclearDamper,
   TriggerMesonScreen,
};
struct CombatAction {
   CombatActionKind kind;
   uint16_t mount_id = 0;
   uint64_t target_vessel_id = 0;
   uint64_t actor_person_id = 0;
};
struct CombatReactionOrder {
   CombatReaction kind;
   uint64_t actor_person_id = 0;
};
struct CombatOrderSet {
   uint64_t combat_id;
   uint64_t view_revision;
   std::vector<CombatAction> actions;
   std::vector<CombatReactionOrder> reactions;
   bool use_tactical_controller = false;
};
struct CombatAutomationPolicy {
   uint64_t expected_revision;
   uint8_t minimum_victory_percent;
   CombatObjective objective;
   bool permit_surrender;
   bool permit_abandon_ship;
};
struct CombatWeaponMount {
   uint16_t mount_id;
   std::string label;
   std::vector<std::string> weapons;
   uint8_t damage_hits;
   uint32_t ammunition_remaining;
};
struct CombatParticipant {
   uint64_t vessel_id;
   uint16_t side;
   std::string name;
   std::string class_name;
   int16_t initiative;
   uint8_t thrust;
   uint16_t hull_remaining;
   uint16_t structure_remaining;
   uint16_t armor_remaining;
   CombatDisposition disposition;
   std::vector<CombatWeaponMount> weapons;
   bool commanded;
   bool player_owned;
   bool online_controlled;
};
struct CombatActor {
   uint64_t person_id;
   std::string name;
   std::string station;
   bool available;
   uint8_t action_budget;
   std::vector<CombatActionKind> allowed_actions;
   std::vector<CombatReaction> allowed_reactions;
};
struct CombatSnapshot {
   uint64_t combat_id;
   uint64_t revision;
   uint16_t round;
   uint64_t round_started_second;
   uint64_t order_due_second;
   uint64_t order_window_real_milliseconds;
   CombatRange range;
   std::vector<CombatParticipant> participants;
   CombatOrderSet default_order;
   CombatAutomationPolicy policy;
   bool player_order_submitted;
   bool complete;
   std::vector<std::string> log;
   std::vector<CombatActor> actors;
   PlayerPhase phase;
};

enum class CombatCareerMode {
   Independent,
   Navy,
   Privateer,
   Pirate,
};
struct TrafficContact;
enum class CareerOpportunityKind {
   NavalOrder,
   PrivateerCommission,
   PirateLead,
   PirateCommission,
};

enum class CareerOpportunityState {
   Offered,
   Accepted,
   Succeeded,
   Failed,
   Expired,
   Reporting,
};

enum class CareerObjectiveKind {
   Patrol,
   Inspect,
   Escort,
   Intercept,
   Capture,
   SeizeCargo,
};

enum class CareerObjectiveEvidenceKind {
   None,
   PatrolLog,
   InspectionRecord,
   EscortRelease,
   TargetDrivenOff,
   TargetCaptured,
   CargoSecured,
   CargoDelivered,
};

enum class PrizeStatus {
   Secured,
   ClaimInTransit,
   AwaitingAdjudication,
   Adjudicated,
   ReadyToFence,
   Settled,
   Seized,
};

enum class WarrantStatus {
   Filed,
   Propagating,
   Active,
   Revoked,
   Satisfied,
};

enum class PrizeSettlementMethod {
   FileClaim,
   TakeAdvance,
   Fence,
   CourtSale,
   KeepPrize,
   LaunderRegistry,
};
struct CareerOpportunity {
   uint64_t opportunity_id;
   CareerOpportunityKind kind;
   CareerOpportunityState state;
   uint64_t issued_system_id;
   uint64_t target_system_id;
   uint64_t target_contact_id;
   uint64_t issued_second;
   uint64_t expires_second;
   uint64_t reward_credits;
   uint16_t service_points;
   std::string authority;
   std::string objective;
   CareerObjectiveKind objective_kind;
   CareerObjectiveEvidenceKind evidence_kind;
   uint64_t evidence_second;
   uint64_t evidence_vessel_id;
   uint64_t order_message_id;
   uint64_t report_message_id;
};
struct PrizeRecord {
   uint64_t prize_id;
   uint64_t captured_vessel_id;
   uint16_t surviving_crew_count;
   uint32_t catalog_id;
   std::string name;
   uint64_t gross_value_credits;
   uint64_t realizable_value_credits;
   uint8_t condition_percent;
   PrizeStatus status;
   uint64_t secured_second;
   uint64_t claim_message_id;
   uint64_t settlement_credits;
   uint64_t advance_credits;
};
struct WarrantRecord {
   uint64_t warrant_id;
   uint64_t issuing_polity_id;
   uint64_t origin_system_id;
   uint64_t filed_second;
   uint64_t message_id;
   uint8_t severity;
   uint64_t bounty_credits;
   uint8_t evidence_percent;
   WarrantStatus status;
   std::string accusation;
   uint64_t resolution_message_id;
   uint64_t resolved_second;
   uint64_t resolving_system_id;
};
struct PirateCruise {
   uint64_t revision;
   bool active;
   uint64_t hunting_system_id;
   uint64_t ends_second;
   uint8_t crew_share_percent;
   uint8_t ship_fund_percent;
   std::string prohibited_targets;
};
enum class InterceptionWatchFilterKind {
   NamedVessel,
   CraftClass,
   AllCraft,
};
enum class InterceptionPurpose {
   ArmedAttack,
   BoardingInspection,
   Arrest,
};
enum class WarrantAssociationKind {
   Historical,
   ReportedAboard,
   ConfirmedAboard,
   WantedVessel,
};
enum class BountyCustodyState {
   AtLarge,
   HeldAboard,
   Settled,
};
struct KnownWarrant {
   uint64_t warrant_id;
   uint64_t subject_person_id;
   std::string subject_name;
   std::string subject_role;
   std::string accusation;
   uint64_t bounty_credits;
   uint8_t severity;
   uint8_t evidence_percent;
   uint64_t issuing_polity_id;
   uint64_t origin_system_id;
   uint64_t filed_second;
   uint64_t associated_ship_id;
   std::string associated_ship_name;
   std::string associated_transponder;
   uint64_t last_known_system_id;
   WarrantAssociationKind association;
   BountyCustodyState custody;
   bool generated_target;
};
struct InterceptionWatchStatus {
   uint64_t started_second;
   uint64_t target_contact_id;
   uint32_t target_catalog_id;
   std::string target_ship_name;
   InterceptionWatchFilterKind filter;
   FlightLocus locus;
   InterceptionPurpose purpose;
};
struct CombatCareerSnapshot {
   uint64_t revision;
   CombatCareerMode mode;
   std::string rank;
   uint16_t service_points;
   uint64_t monthly_salary_credits;
   uint64_t next_naval_board_second;
   uint32_t public_heat;
   int16_t underworld_standing;
   uint16_t crew_pressure;
   std::vector<CareerOpportunity> opportunities;
   std::vector<PrizeRecord> prizes;
   std::vector<WarrantRecord> warrants;
   PirateCruise cruise;
   std::string local_enforcement_summary;
   std::vector<TrafficContact> system_contacts;
   std::vector<TrafficContact> local_contacts;
   std::optional<InterceptionWatchStatus> interception_watch;
   std::vector<KnownWarrant> known_warrants;
   PlayerPhase phase;
};

using InterceptionStart = std::variant<CombatSnapshot, CombatCareerSnapshot, EncounterResult>;

enum class InterceptionWatchSelection {
   Cancel,
   AllCraft,
   CraftClass,
};

enum class MessageClass {
   AgencyNews,
   PublicService,
   ContractOffer,
   TrafficNotice,
   Private,
};
enum class PrivateRecipientKind {
   System,
   Captain,
};

struct PrivateMessageRequest {
   PrivateRecipientKind recipient_kind;
   uint64_t destination_system_id;
   PlayerIdentity recipient;
   uint64_t encryption_key_id;
   uint16_t ttl_weeks;
   std::string subject;
   std::string body;
};

enum class RadioTransmissionKind {
   PlayerBroadcast,
   InspectionOrder,
   BoardingOrder,
   SurrenderDemand,
};

struct RadioInboxEntry {
   uint64_t reception_id;
   uint64_t transmission_id;
   uint64_t receiving_ship_id;
   uint64_t sender_ship_id;
   std::string sender_ship_name;
   std::string sender_transponder;
   PlayerIdentity sender;
   uint64_t emitted_second;
   uint64_t received_second;
   uint64_t expires_second;
   RadioTransmissionKind kind;
   bool actionable;
   uint64_t action_reference_id;
};

struct SystemRadioSnapshot {
   uint64_t ship_id;
   uint64_t system_id;
   uint64_t current_second;
   bool can_transmit;
   std::string unavailable_reason;
   std::vector<RadioInboxEntry> entries;
   std::vector<PlayerIdentity> mutes;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

struct RadioContent {
   uint64_t reception_id;
   uint64_t transmission_id;
   std::string body;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class InsuranceKind {
   DestinationAssistance,
};

enum class MessageImportance {
   Routine,
   Notable,
   Important,
   Headline,
};

enum class MessageClassification {
   Unreviewed,
   Ignored,
   ReviewLater,
   Actioned,
   Archived,
};

enum class SystemMappingState {
   KnownPublic,
   Unresolved,
   PublicDispatched,
   DirectDispatched,
   Withheld,
   Secret,
};

enum class SystemMappingChoice {
   PublicNotification,
   DirectEarth,
   Withhold,
   WithholdSecret,
};

struct SystemMappingStatus {
   uint64_t system_id;
   SystemMappingState state;
   std::optional<uint64_t> dispatch_message_id;
   uint64_t changed_second;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class MessageActionKind {
   None,
   ClaimOffer,
   ReviewTask,
   ReviewOperations,
   ReviewFinance,
   ReviewMapping,
};

struct MessageItem {
   uint64_t message_id;
   uint64_t origin_system_id;
   std::string origin_system_name;
   uint64_t created_second;
   uint64_t available_second;
   uint64_t expires_second;
   MessageClass message_class;
   MessageImportance importance;
   std::string subject;
   std::string body;
   std::optional<uint64_t> offer_id;
   uint64_t offer_revision;
   bool offer_available;
   MessageClassification classification;
   bool previously_seen;
   bool expired;
   MessageActionKind action_kind;
   uint64_t action_reference_id;
};

struct MessageFilter {
   MessageClass message_class;
   MessageImportance minimum_importance;
};

struct ArrivalPacket {
   bool new_arrival;
   uint64_t system_id;
   std::string system_name;
   uint64_t arrival_second;
   std::optional<uint64_t> mailbag_id;
   uint64_t mail_delivered;
   uint64_t mail_forwarded;
   uint64_t mail_expired;
   uint64_t stipend_credits;
   std::vector<MessageItem> items;
   SystemMappingStatus mapping_status;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

struct MessageManagement {
   std::vector<MessageItem> items;
   std::vector<MessageFilter> filters;
   uint64_t committed_sequence;
   uint64_t revision;
   PlayerPhase phase;
};

enum class TrafficMovementKind {
   Arrival,
   Departure,
   Present,
};

enum class TrafficContactResolution {
   TransponderOnly,
   Approximate,
   Identified,
};

enum class TrafficAttachment {
   Spaceborne,
   Berthed,
   Landed,
};

struct TrafficContact {
   uint64_t contact_id;
   uint32_t catalog_id;
   std::string class_name;
   std::string ship_name;
   std::string transponder;
   std::string operator_name;
   std::string role;
   uint64_t displacement_millitons;
   uint64_t origin_system_id;
   uint64_t destination_system_id;
   TrafficMovementKind movement;
   uint64_t edge_second;
   TrafficContactResolution resolution;
   uint8_t confidence_percent;
   bool player_owned;
   bool online_controlled;
   TrafficAttachment attachment;
};

struct TrafficSnapshot {
   uint64_t system_id;
   std::string system_name;
   uint64_t observed_second;
   std::vector<TrafficContact> contacts;
};

enum class PlayerEventKind {
   SessionReplaced,
   ServerStopping,
   PhaseChanged,
   TrafficSnapshot,
   TrafficMovement,
   CheckpointReady,
   EncounterReady,
   RadioUnread,
};

struct PlayerEvent {
   PlayerEventKind kind;
   uint64_t committed_sequence;
   std::optional<TravelStatus> travel_status;
   std::optional<TrafficSnapshot> traffic_snapshot;
   std::optional<TrafficContact> traffic_contact;
   std::optional<CheckpointSnapshot> checkpoint;
   std::optional<EncounterSnapshot> encounter;
   uint64_t observed_second = 0;
   uint64_t system_id = 0;
   uint64_t ship_id = 0;
   uint64_t unread_count = 0;
};

std::optional<PlayerEvent> poll_event(TlsConnection& connection,
                                      uint64_t session_epoch);

ServerHello exchange_hello(TlsConnection& connection,
                           const PlayerIdentity& identity,
                           const std::string& client_name,
                           const std::string& language_tag);

bool language_selection_matches(std::string_view requested,
                                std::string_view selected) noexcept;

PlayerCreated create_player(TlsConnection& connection,
                            uint64_t session_epoch,
                            const PlayerCreation& creation,
                            const std::array<uint8_t, 16>& command_id,
                            uint64_t request_id);

CaptainCreationOptions get_captain_creation_options(
   TlsConnection& connection,
   uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

StartingShipOffers get_starting_ship_offers(
   TlsConnection& connection,
   uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

StartingShipOptions get_starting_ship_options(
   TlsConnection& connection,
   uint64_t session_epoch,
   uint64_t setup_revision,
   uint32_t starting_offer_id,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

StartingCrewPlan get_starting_crew_plan(
   TlsConnection& connection,
   uint64_t session_epoch,
   uint64_t setup_revision,
   uint32_t starting_offer_id,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

CrewManagementSnapshot get_crew_management(
   TlsConnection& connection,
   uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

CrewManagementSnapshot set_crew_training_target(
   TlsConnection& connection,
   uint64_t session_epoch,
   uint64_t person_id,
   SkillId skill,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

CrewManagementSnapshot set_crew_assignments(
   TlsConnection& connection,
   uint64_t session_epoch,
   uint64_t person_id,
   const std::vector<uint16_t>& slot_ids,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

CrewManagementSnapshot apply_personnel_action(
   TlsConnection& connection,
   uint64_t session_epoch,
   uint64_t person_id,
   uint64_t expected_service_revision,
   PersonnelActionKind action,
   uint64_t target_ship_id,
   uint16_t duration_days,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

ShipStatusSnapshot get_ship_status(
   TlsConnection& connection,
   uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

DockedServices get_docked_services(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&,
                                   uint64_t);
ShipStatusSnapshot commit_docked_service(TlsConnection&, uint64_t, const DockedServiceOrder&,
      const std::array<uint8_t, 16>&, uint64_t);

DockedSnapshot get_docked_snapshot(TlsConnection& connection,
                                   uint64_t session_epoch,
                                   const std::array<uint8_t, 16>& command_id,
                                   uint64_t request_id);

KnownDestinations get_known_destinations(
   TlsConnection& connection,
   uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

CoursePlot plot_course(TlsConnection& connection,
                       uint64_t session_epoch,
                       uint64_t origin_system_id,
                       uint64_t destination_system_id,
                       bool use_current_fuel,
                       const std::array<uint8_t, 16>& command_id,
                       uint64_t request_id);

MarketSnapshot get_market(TlsConnection& connection,
                          uint64_t session_epoch,
                          const std::array<uint8_t, 16>& command_id,
                          uint64_t request_id);

MarketSnapshot buy_cargo(TlsConnection& connection,
                         uint64_t session_epoch,
                         uint64_t market_revision,
                         uint64_t offer_id,
                         uint64_t quantity_millitons,
                         const std::array<uint8_t, 16>& command_id,
                         uint64_t request_id);

MarketSnapshot sell_cargo(TlsConnection& connection,
                          uint64_t session_epoch,
                          uint64_t market_revision,
                          uint64_t cargo_lot_id,
                          uint64_t quantity_millitons,
                          const std::array<uint8_t, 16>& command_id,
                          uint64_t request_id);

MarketSnapshot sell_cargo_to_lead(
   TlsConnection& connection,
   uint64_t session_epoch,
   uint64_t market_revision,
   uint64_t cargo_lot_id,
   uint64_t quantity_millitons,
   uint64_t buyer_lead_id,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

TaskLedger get_task_ledger(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
TaskLedger accept_task_offer(TlsConnection&, uint64_t, uint64_t, uint64_t,
                             const std::array<uint8_t, 16>&, uint64_t);
TaskLedger set_carriage_declaration(TlsConnection&, uint64_t, const CarriageDeclaration&,
                                    const std::array<uint8_t, 16>&, uint64_t);
MarketSnapshot begin_market_search(TlsConnection&,
                                   uint64_t,
                                   MarketSearchKind,
                                   MarketSearchMethod,
                                   uint64_t,
                                   uint16_t,
                                   uint64_t,
                                   const std::array<uint8_t, 16>&,
                                   uint64_t);
MarketSnapshot cancel_work_assignment(TlsConnection&,
                                      uint64_t,
                                      uint64_t,
                                      const std::array<uint8_t, 16>&,
                                      uint64_t);
MarketSnapshot reserve_market_lead(
   TlsConnection&,
   uint64_t session_epoch,
   uint64_t lead_id,
   uint64_t expected_revision,
   uint64_t quantity_millitons,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
MarketSnapshot release_market_reservation(
   TlsConnection&,
   uint64_t session_epoch,
   uint64_t lead_id,
   uint64_t expected_revision,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
TaskLedger apply_task_action(
   TlsConnection&,
   uint64_t session_epoch,
   uint64_t task_id,
   uint64_t expected_revision,
   TaskActionKind action,
   const std::string& explanation,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
MessageManagement send_private_message(
   TlsConnection&,
   uint64_t session_epoch,
   const PrivateMessageRequest& request,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
SystemRadioSnapshot get_system_radio(
   TlsConnection&, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
SystemRadioSnapshot transmit_system_radio(
   TlsConnection&, uint64_t, const std::string&, const std::array<uint8_t, 16>&, uint64_t);
RadioContent peek_radio_reception(
   TlsConnection&, uint64_t, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
SystemRadioSnapshot acknowledge_radio_reception(
   TlsConnection&, uint64_t, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
SystemRadioSnapshot set_radio_mute(
   TlsConnection&, uint64_t, const PlayerIdentity&, bool,
   const std::array<uint8_t, 16>&, uint64_t);
FinanceSnapshot purchase_insurance(
   TlsConnection&,
   uint64_t session_epoch,
   InsuranceKind kind,
   bool enabled,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
FinanceSnapshot misappropriate_restricted_credits(
   TlsConnection&,
   uint64_t session_epoch,
   uint64_t amount,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
FinanceSnapshot get_finance(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
MarketKnowledge get_market_knowledge(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&,
                                     uint64_t);
ShipMarket get_ship_market(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
ShipMarket purchase_ship(TlsConnection&, uint64_t, uint64_t, bool, const std::array<uint8_t, 16>&,
                         uint64_t);
CrewMarket get_crew_market(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
CrewMarket hire_crew(TlsConnection&, uint64_t, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
FleetSnapshot get_fleet(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
FleetSnapshot set_active_ship(TlsConnection&, uint64_t, uint64_t, uint64_t,
                              const std::array<uint8_t, 16>&, uint64_t);
FleetSnapshot assign_ship_captain(TlsConnection&, uint64_t, uint64_t, uint64_t, uint64_t,
                                  const std::array<uint8_t, 16>&, uint64_t);
FleetSnapshot transfer_ship_stores(TlsConnection&, uint64_t, uint64_t, uint64_t, uint64_t,
                                   StoreTransferKind, uint64_t, const std::string&, uint64_t,
                                   const std::array<uint8_t, 16>&, uint64_t);

TravelStatus get_travel_status(TlsConnection& connection,
                               uint64_t session_epoch,
                               const std::array<uint8_t, 16>& command_id,
                               uint64_t request_id);

TravelStatus begin_voyage(TlsConnection& connection,
                          uint64_t session_epoch,
                          uint64_t destination_system_id,
                          const std::array<uint8_t, 16>& command_id,
                          uint64_t request_id);

FlightPlanSnapshot get_flight_plan(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&,
                                   uint64_t);
FlightPlanPreview preview_flight_plan(TlsConnection&, uint64_t, const FlightPlanProposal&,
                                      const std::array<uint8_t, 16>&, uint64_t);
FlightPlanSnapshot commit_flight_plan(TlsConnection&, uint64_t, const FlightPlanProposal&,
                                      const std::vector<uint8_t>&, bool, const std::array<uint8_t, 16>&, uint64_t);
CheckpointSnapshot acknowledge_checkpoint(TlsConnection&, uint64_t, uint64_t,
      const std::array<uint8_t, 16>&, uint64_t);
EncounterSnapshot get_encounter(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
EncounterResult resolve_encounter(TlsConnection&, uint64_t, uint64_t, uint64_t, EncounterPosture,
                                  const std::vector<EncounterFallback>&, const std::array<uint8_t, 16>&, uint64_t);
CombatSnapshot get_combat(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&, uint64_t);
CombatSnapshot submit_combat_order(TlsConnection&, uint64_t, const CombatOrderSet&,
                                   const std::array<uint8_t, 16>&, uint64_t);
CombatSnapshot set_combat_automation_policy(TlsConnection&, uint64_t, const CombatAutomationPolicy&,
      const std::array<uint8_t, 16>&, uint64_t);
CombatCareerSnapshot get_combat_career(TlsConnection&, uint64_t, const std::array<uint8_t, 16>&,
                                       uint64_t);
CombatCareerSnapshot accept_career_opportunity(TlsConnection&, uint64_t, uint64_t, uint64_t,
      const std::array<uint8_t, 16>&, uint64_t);
InterceptionStart engage_traffic_contact(TlsConnection&, uint64_t, uint64_t, uint64_t,
                                         InterceptionPurpose,
                                         const std::array<uint8_t, 16>&, uint64_t);
CombatCareerSnapshot set_interception_watch(TlsConnection&, uint64_t,
                                            InterceptionWatchSelection, uint32_t,
                                            InterceptionPurpose, uint64_t,
                                            const std::array<uint8_t, 16>&, uint64_t);
CombatCareerSnapshot set_pirate_cruise(TlsConnection&, uint64_t, const PirateCruise&,
                                       const std::array<uint8_t, 16>&, uint64_t);
CombatCareerSnapshot settle_prize(TlsConnection&, uint64_t, uint64_t, uint64_t,
                                  PrizeSettlementMethod, const std::array<uint8_t, 16>&, uint64_t);
CombatCareerSnapshot settle_warrant(TlsConnection&, uint64_t, uint64_t, uint64_t,
                                    const std::array<uint8_t, 16>&, uint64_t);
CombatCareerSnapshot set_combat_career_mode(TlsConnection&, uint64_t, CombatCareerMode, uint64_t,
                                            const std::array<uint8_t, 16>&, uint64_t);
FleetSnapshot recover_command(TlsConnection&, uint64_t, const std::string&,
                              const std::array<uint8_t, 16>&, uint64_t);
FleetSnapshot declare_bankruptcy(TlsConnection&, uint64_t, const std::string&,
                                 const std::array<uint8_t, 16>&, uint64_t);
PlayerPhase abandon_player(TlsConnection&, uint64_t, const std::string&,
                           const std::array<uint8_t, 16>&, uint64_t);

ArrivalPacket open_arrival_packet(TlsConnection& connection,
                                  uint64_t session_epoch,
                                  const std::array<uint8_t, 16>& command_id,
                                  uint64_t request_id);

MessageManagement get_message_management(
   TlsConnection& connection,
   uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

MessageManagement set_message_classification(
   TlsConnection& connection,
   uint64_t session_epoch,
   uint64_t message_id,
   MessageClassification classification,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
MessageManagement set_message_filter(
   TlsConnection& connection,
   uint64_t session_epoch,
   MessageClass message_class,
   MessageImportance minimum_importance,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

SystemMappingStatus set_system_mapping_disclosure(
   TlsConnection& connection,
   uint64_t session_epoch,
   uint64_t system_id,
   SystemMappingChoice choice,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

}  // namespace ct
