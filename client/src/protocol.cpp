#include "ct/protocol.hpp"

#include "ct/tls_connection.hpp"
#include "ct_rpc.capnp.h"

#include <capnp/message.h>
#include <capnp/serialize.h>
#include <kj/array.h>

#include <algorithm>
#include <array>
#include <cctype>
#include <cstring>
#include <limits>
#include <span>
#include <stdexcept>
#include <vector>

namespace ct
{
namespace
{

constexpr uint16_t PROTOCOL_VERSION = 4;
constexpr size_t MAX_FRAME_BYTES = 1024 * 1024;

void send_frame(TlsConnection& connection, const kj::ArrayPtr<const kj::byte> message)
{
   if(message.size() == 0 || message.size() > MAX_FRAME_BYTES ||
         message.size() > std::numeric_limits<uint32_t>::max()) {
      throw std::runtime_error("invalid outgoing CT-RPC frame size");
   }
   connection.send_frame(
      std::span(reinterpret_cast<const uint8_t*>(message.begin()), message.size()));
}

std::vector<uint8_t> receive_frame_direct(TlsConnection& connection)
{
   const auto header = connection.receive_exact(4);
   const auto size = (static_cast<uint32_t>(header[0]) << 24) |
                     (static_cast<uint32_t>(header[1]) << 16) |
                     (static_cast<uint32_t>(header[2]) << 8) |
                     static_cast<uint32_t>(header[3]);
   if(size == 0 || size > MAX_FRAME_BYTES) {
      throw std::runtime_error("invalid incoming CT-RPC frame size");
   }
   return connection.receive_exact(size);
}

}  // namespace

bool language_selection_matches(const std::string_view requested,
                                const std::string_view selected) noexcept
{
   if(requested.empty() || selected.empty()) {
      return false;
   }
   const auto equal_prefix = [](const std::string_view left,
                                const std::string_view right) {
      return left.size() <= right.size() &&
             std::equal(
                left.begin(), left.end(), right.begin(),
                [](const char a, const char b) {
                   return std::tolower(static_cast<unsigned char>(a)) ==
                          std::tolower(static_cast<unsigned char>(b));
                });
   };
   if(requested.size() == selected.size()) {
      return equal_prefix(requested, selected);
   }
   if(requested.size() < selected.size()) {
      return selected[requested.size()] == '-' &&
             equal_prefix(requested, selected);
   }
   return requested[selected.size()] == '-' &&
          equal_prefix(selected, requested);
}

ServerHello exchange_hello(TlsConnection& connection,
                           const PlayerIdentity& identity,
                           const std::string& client_name,
                           const std::string& language_tag)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   auto hello = envelope.initClientHello();
   auto wire_identity = hello.initIdentity();
   wire_identity.setBbsId(identity.bbs_id);
   wire_identity.setPlayerId(identity.player_id);
   hello.setClientName(client_name);
   hello.setLanguageTag(language_tag);
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());

   const auto frame = receive_frame_direct(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto response_words = kj::heapArray<capnp::word>(word_count);
   std::memset(response_words.begin(), 0, response_words.asBytes().size());
   std::memcpy(response_words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(response_words);
   const auto response = reader.getRoot<rpc::Envelope>();
   if(response.isClose()) {
      const auto close = response.getClose();
      if(close.hasMessage() && close.getMessage().size() != 0) {
         throw std::runtime_error(close.getMessage().cStr());
      }
      const auto legacy = reader.getRoot<rpc::LegacyV2Envelope>();
      if(legacy.isClose()) {
         throw std::runtime_error(legacy.getClose().getReason().cStr());
      }
      throw std::runtime_error("server closed the connection during language negotiation");
   }
   if(response.getProtocolVersion() != PROTOCOL_VERSION) {
      const auto legacy = reader.getRoot<rpc::LegacyV2Envelope>();
      if(legacy.isClose()) {
         throw std::runtime_error(legacy.getClose().getReason().cStr());
      }
      throw std::runtime_error("server selected an unsupported CT-RPC version");
   }
   if(!response.isServerHello()) {
      throw std::runtime_error("expected a CT-RPC ServerHello");
   }
   const auto server_hello = response.getServerHello();
   const auto server_identity = server_hello.getIdentity();
   const auto wire_formatting = server_hello.getFormatting();
   ServerHello result{
      .identity =
      {
         .bbs_id = server_identity.getBbsId(),
         .player_id = server_identity.getPlayerId(),
      },
      .assigned_epoch = server_hello.getAssignedEpoch(),
      .committed_sequence = server_hello.getCommittedSequence(),
      .phase =
      [&server_hello] {
         switch(server_hello.getPhase())
      {
      case rpc::Phase::DISCONNECTED:
         return PlayerPhase::Disconnected;
      case rpc::Phase::JUMP:
         return PlayerPhase::Jump;
      case rpc::Phase::INTERPLANETARY:
         return PlayerPhase::Interplanetary;
      case rpc::Phase::ENCOUNTER:
         return PlayerPhase::Encounter;
      case rpc::Phase::NEW_USER:
         return PlayerPhase::NewUser;
      case rpc::Phase::DOCKED:
         return PlayerPhase::Docked;
      case rpc::Phase::ON_PLANET:
         return PlayerPhase::OnPlanet;
      case rpc::Phase::TERMINAL:
         return PlayerPhase::Terminal;
      default:
         return PlayerPhase::Other;
      }
      }(),
      .language_tag = server_hello.getLanguageTag().cStr(),
      .formatting =
      {
         .decimal_separator = wire_formatting.getDecimalSeparator().cStr(),
         .grouping_separator = wire_formatting.getGroupingSeparator().cStr(),
         .primary_grouping_digits = wire_formatting.getPrimaryGroupingDigits(),
         .secondary_grouping_digits = wire_formatting.getSecondaryGroupingDigits(),
         .game_timestamp_pattern = wire_formatting.getGameTimestampPattern().cStr(),
         .game_duration_pattern = wire_formatting.getGameDurationPattern().cStr(),
         .real_duration_pattern = wire_formatting.getRealDurationPattern().cStr(),
      },
   };
   if(result.identity != identity) {
      throw std::runtime_error("server hello returned a different player identity");
   }
   if(result.assigned_epoch == 0 ||
         response.getSessionEpoch() != result.assigned_epoch) {
      throw std::runtime_error("server hello returned an invalid session epoch");
   }
   if(!language_selection_matches(language_tag, result.language_tag)) {
      throw std::runtime_error("server selected an invalid language tag");
   }
   validate_display_formatting(result.formatting);
   connection.start_dispatch();
   return result;
}

namespace
{

kj::Array<capnp::word> receive_response(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t request_id)
{
   for(;;) {
      auto frame = connection.receive_frame();
      const auto word_count =
         (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
      auto words = kj::heapArray<capnp::word>(word_count);
      std::memset(words.begin(), 0, words.asBytes().size());
      std::memcpy(words.asBytes().begin(), frame.data(), frame.size());
      capnp::FlatArrayMessageReader reader(words);
      const auto envelope = reader.getRoot<rpc::Envelope>();
      if(envelope.getProtocolVersion() != PROTOCOL_VERSION ||
            envelope.getSessionEpoch() != session_epoch) {
         throw std::runtime_error("invalid CT-RPC response envelope");
      }
      if(envelope.isEvent()) {
         connection.defer_event_frame(std::move(frame));
         continue;
      }
      if(envelope.isClose()) {
         throw std::runtime_error(envelope.getClose().getMessage().cStr());
      }
      if(envelope.getRequestId() != request_id || !envelope.isResponse()) {
         throw std::runtime_error("invalid CT-RPC response envelope");
      }
      return words;
   }
}

rpc::Response::Reader checked_response(
   const rpc::Envelope::Reader envelope,
   const std::array<uint8_t, 16>& command_id)
{
   const auto response = envelope.getResponse();
   const auto returned = response.getCommandId();
   if(returned.size() != command_id.size() ||
         !std::equal(returned.begin(), returned.end(), command_id.begin())) {
      throw std::runtime_error("CT-RPC response command ID mismatch");
   }
   if(response.isError()) {
      const auto error = response.getError();
      if(error.getCode() == rpc::ErrorCode::INVALID_COMMAND) {
         throw PlayerRequestRejected(error.getMessage().cStr());
      }
      throw std::runtime_error(error.getMessage().cStr());
   }
   return response;
}

SkillId decode_skill(const rpc::SkillId skill)
{
   return static_cast<SkillId>(static_cast<uint16_t>(skill));
}

rpc::SkillId encode_skill(const SkillId skill)
{
   return static_cast<rpc::SkillId>(static_cast<uint16_t>(skill));
}

PlayerPhase decode_response_phase(const rpc::Phase phase)
{
   switch(phase) {
   case rpc::Phase::DISCONNECTED:
      return PlayerPhase::Disconnected;
   case rpc::Phase::JUMP:
      return PlayerPhase::Jump;
   case rpc::Phase::INTERPLANETARY:
      return PlayerPhase::Interplanetary;
   case rpc::Phase::ENCOUNTER:
      return PlayerPhase::Encounter;
   case rpc::Phase::NEW_USER:
      return PlayerPhase::NewUser;
   case rpc::Phase::DOCKED:
      return PlayerPhase::Docked;
   case rpc::Phase::ON_PLANET:
      return PlayerPhase::OnPlanet;
   case rpc::Phase::TERMINAL:
      return PlayerPhase::Terminal;
   }
   return PlayerPhase::Other;
}

TravelStage decode_travel_stage(const rpc::TravelStage stage)
{
   switch(stage) {
   case rpc::TravelStage::DOCKED:
      return TravelStage::Docked;
   case rpc::TravelStage::DEPARTING_FOR_JUMP:
      return TravelStage::DepartingForJump;
   case rpc::TravelStage::JUMP_SPACE:
      return TravelStage::JumpSpace;
   case rpc::TravelStage::APPROACHING_STARPORT:
      return TravelStage::ApproachingStarport;
   case rpc::TravelStage::REFIT:
      return TravelStage::Refit;
   case rpc::TravelStage::PROPER_REPAIR:
      return TravelStage::ProperRepair;
   case rpc::TravelStage::GAS_GIANT_SKIM:
      return TravelStage::GasGiantSkim;
   case rpc::TravelStage::WILDERNESS_WATER:
      return TravelStage::WildernessWater;
   case rpc::TravelStage::HOLDING:
      return TravelStage::Holding;
   case rpc::TravelStage::ENCOUNTER:
      return TravelStage::Encounter;
   }
   throw std::runtime_error("unknown CT-RPC travel stage");
}

FlightLocus decode_flight_locus(const rpc::FlightLocus::Reader locus)
{
   FlightLocus result{
      .kind = FlightLocusKind::JumpLocus,
      .system_id = locus.getSystemId(),
      .world_id = 0,
      .facility_id = 0,
      .body_id = 0,
      .coreward_parsecs = 0.0,
      .spinward_parsecs = 0.0,
      .north_parsecs = 0.0,
   };
   if(locus.isPort()) {
      const auto port = locus.getPort();
      result.kind = FlightLocusKind::Port;
      result.world_id = port.getWorldId();
      result.facility_id = port.getFacilityId();
   } else if(locus.isBodyId()) {
      result.kind = FlightLocusKind::Body;
      result.body_id = locus.getBodyId();
   } else if(locus.isDeepSpace()) {
      const auto position = locus.getDeepSpace();
      result.kind = FlightLocusKind::DeepSpace;
      result.coreward_parsecs = position.getCoreward();
      result.spinward_parsecs = position.getSpinward();
      result.north_parsecs = position.getNorth();
   }
   return result;
}

void encode_flight_locus(rpc::FlightLocus::Builder target, const FlightLocus& source)
{
   target.setSystemId(source.system_id);
   if(source.kind == FlightLocusKind::Port) {
      auto port = target.initPort();
      port.setWorldId(source.world_id);
      port.setFacilityId(source.facility_id);
   } else if(source.kind == FlightLocusKind::Body) {
      target.setBodyId(source.body_id);
   } else if(source.kind == FlightLocusKind::DeepSpace) {
      auto position = target.initDeepSpace();
      position.setCoreward(source.coreward_parsecs);
      position.setSpinward(source.spinward_parsecs);
      position.setNorth(source.north_parsecs);
   } else {
      target.setJumpLocus();
   }
}

void encode_policy(rpc::EncounterPolicy::Builder target, const EncounterPolicy& source)
{
   target.setHostilePosture(static_cast<rpc::EncounterPosture>(source.hostile_posture));
   auto fallbacks = target.initHostileFallbacks(source.hostile_fallbacks.size());
   for(size_t i = 0; i < source.hostile_fallbacks.size(); ++i) {
      fallbacks.set(i, static_cast<rpc::EncounterFallback>(source.hostile_fallbacks[i]));
   }
   target.setComplyWithInspection(source.comply_with_inspection);
   target.setReportDistress(source.report_distress);
   target.setAssistDistress(source.assist_distress);
}

EncounterPolicy decode_policy(rpc::EncounterPolicy::Reader source)
{
   EncounterPolicy result;
   result.hostile_posture = static_cast<EncounterPosture>(source.getHostilePosture());
   result.hostile_fallbacks.clear();
   for(auto value : source.getHostileFallbacks()) {
      result.hostile_fallbacks.push_back(static_cast<EncounterFallback>(value));
   }
   result.comply_with_inspection = source.getComplyWithInspection();
   result.report_distress = source.getReportDistress();
   result.assist_distress = source.getAssistDistress();
   return result;
}

void encode_proposal(rpc::FlightPlanProposal::Builder target, const FlightPlanProposal& source)
{
   target.setExpectedPlanRevision(source.expected_plan_revision);
   auto steps = target.initSteps(source.steps.size());
   for(size_t i = 0; i < source.steps.size(); ++i) {
      const auto& step = source.steps[i];
      auto item = steps[i];
      encode_flight_locus(item.initLocus(), step.locus);
      item.setAuthority(static_cast<rpc::WaypointAuthority>(step.authority));
      auto action = item.initAction();
      switch(step.action.kind) {
      case FlightPlanActionKind::Hold:
         action.setHold();
         break;
      case FlightPlanActionKind::Jump: {
         auto jump = action.initJump();
         jump.setDestinationSystemId(step.action.destination_system_id);
         jump.setNavigation(static_cast<rpc::JumpNavigationMethod>(
                               step.action.jump_navigation));
         jump.setProceedOnKnownBadPlot(step.action.proceed_on_known_bad_plot);
      }
      break;
      case FlightPlanActionKind::JumpCoordinates: {
         auto jump = action.initJumpCoordinates();
         jump.setNavigation(static_cast<rpc::JumpNavigationMethod>(
                               step.action.jump_navigation));
         jump.setProceedOnKnownBadPlot(step.action.proceed_on_known_bad_plot);
         auto position = jump.initDestination();
         position.setCoreward(step.action.coreward_parsecs);
         position.setSpinward(step.action.spinward_parsecs);
         position.setNorth(step.action.north_parsecs);
         break;
      }
      case FlightPlanActionKind::Dock: {
         auto port = action.initDock();
         port.setWorldId(step.action.world_id);
         port.setFacilityId(step.action.facility_id);
         break;
      }
      case FlightPlanActionKind::Fuel: {
         auto fuel = action.initFuel();
         fuel.setOperation(static_cast<rpc::FuelOperation>(step.action.fuel_operation));
         fuel.setQuantityMillitons(step.action.quantity_millitons);
         break;
      }
      }
   }
   encode_policy(target.initPolicy(), source.policy);
}

FlightPlanStep decode_plan_step(rpc::FlightPlanStep::Reader source)
{
   FlightPlanStep result{
      .locus = decode_flight_locus(source.getLocus()),
      .authority = static_cast<WaypointAuthority>(source.getAuthority()),
      .action = {},
   };
   auto action = source.getAction();
   if(action.isJump()) {
      const auto jump = action.getJump();
      result.action.kind = FlightPlanActionKind::Jump;
      result.action.destination_system_id = jump.getDestinationSystemId();
      result.action.jump_navigation = static_cast<JumpNavigationMethod>(jump.getNavigation());
      result.action.proceed_on_known_bad_plot = jump.getProceedOnKnownBadPlot();
   } else if(action.isJumpCoordinates()) {
      const auto jump = action.getJumpCoordinates();
      const auto position = jump.getDestination();
      result.action.kind = FlightPlanActionKind::JumpCoordinates;
      result.action.jump_navigation = static_cast<JumpNavigationMethod>(jump.getNavigation());
      result.action.proceed_on_known_bad_plot = jump.getProceedOnKnownBadPlot();
      result.action.coreward_parsecs = position.getCoreward();
      result.action.spinward_parsecs = position.getSpinward();
      result.action.north_parsecs = position.getNorth();
   } else if(action.isDock()) {
      result.action.kind = FlightPlanActionKind::Dock;
      result.action.world_id = action.getDock().getWorldId();
      result.action.facility_id = action.getDock().getFacilityId();
   } else if(action.isFuel()) {
      result.action.kind = FlightPlanActionKind::Fuel;
      result.action.fuel_operation = static_cast<FuelOperation>(action.getFuel().getOperation());
      result.action.quantity_millitons = action.getFuel().getQuantityMillitons();
   }
   return result;
}

FlightPlanProposal decode_proposal(rpc::FlightPlanProposal::Reader source)
{
   FlightPlanProposal result{
      .expected_plan_revision = source.getExpectedPlanRevision(),
      .steps = {},
      .policy = decode_policy(source.getPolicy()),
   };

   for(auto step : source.getSteps()) {
      result.steps.push_back(decode_plan_step(step));
   }
   return result;
}

std::optional<ShipActivityStatus> decode_ship_activity(
   const rpc::ShipActivityStatus::Reader source)
{
   if(source.isNone()) {
      return std::nullopt;
   }
   ShipActivityStatus result{
      .activity_id = source.getActivityId(),
      .kind = ShipActivityKind::Refit,
      .subsystem_id = 0,
      .quantity_millitons = 0,
      .opportunity_id = 0,
      .started_second = source.getStartedSecond(),
      .due_second = source.getDueSecond(),
      .cost_credits = source.getCostCredits(),
      .source_body_id = source.getHasSourceBody()
      ? std::optional<uint32_t>(source.getSourceBodyId())
      : std::nullopt,
   };
   if(source.isRefit()) {
      result.kind = ShipActivityKind::Refit;
   } else if(source.isRefurbishment()) {
      result.kind = ShipActivityKind::Refurbishment;
      result.quantity_millitons = source.getRefurbishment();
   } else if(source.isProperRepair()) {
      result.kind = ShipActivityKind::ProperRepair;
      result.subsystem_id = source.getProperRepair();
   } else if(source.isGasGiantSkim()) {
      result.kind = ShipActivityKind::GasGiantSkim;
      result.quantity_millitons = source.getGasGiantSkim();
   } else if(source.isWildernessWater()) {
      result.kind = ShipActivityKind::WildernessWater;
      result.quantity_millitons = source.getWildernessWater();
   } else if(source.isEscortDuty()) {
      result.kind = ShipActivityKind::EscortDuty;
      result.opportunity_id = source.getEscortDuty();
   } else if(source.isFieldRecovery()) {
      result.kind = ShipActivityKind::FieldRecovery;
      result.subsystem_id = source.getFieldRecovery();
   } else {
      throw std::runtime_error("unknown CT-RPC ship activity");
   }
   return result;
}

SkillPool decode_pool(const rpc::SkillPool::Reader pool)
{
   return SkillPool{
      .level3 = pool.getLevel3(),
      .level2 = pool.getLevel2(),
      .level1 = pool.getLevel1(),
      .level0 = pool.getLevel0(),
   };
}

PersonDraft decode_person(const rpc::PersonDraft::Reader person)
{
   const auto characteristics = person.getCharacteristics();
   const auto training = person.getTraining();
   PersonDraft result{
      .name = person.getName().cStr(),
      .characteristics =
      {
         .strength = characteristics.getStrength(),
         .dexterity = characteristics.getDexterity(),
         .endurance = characteristics.getEndurance(),
         .intelligence = characteristics.getIntelligence(),
         .education = characteristics.getEducation(),
         .charisma = characteristics.getCharisma(),
      },
      .skills = {},
      .training =
      {
         .skill = decode_skill(training.getSkill()),
         .needed_weeks = training.getNeededWeeks(),
         .current_weeks = training.getCurrentWeeks(),
      },
   };
   for(const auto rating : person.getSkills()) {
      result.skills.push_back(SkillRating{
         .skill = decode_skill(rating.getSkill()),
         .level = rating.getLevel(),
      });
   }
   return result;
}

void set_person(rpc::PersonDraft::Builder person, const PersonDraft& source)
{
   person.setName(source.name);
   auto characteristics = person.initCharacteristics();
   characteristics.setStrength(source.characteristics.strength);
   characteristics.setDexterity(source.characteristics.dexterity);
   characteristics.setEndurance(source.characteristics.endurance);
   characteristics.setIntelligence(source.characteristics.intelligence);
   characteristics.setEducation(source.characteristics.education);
   characteristics.setCharisma(source.characteristics.charisma);
   auto skills = person.initSkills(source.skills.size());
   for(size_t index = 0; index < source.skills.size(); ++index) {
      auto rating = skills[index];
      rating.setSkill(encode_skill(source.skills[index].skill));
      rating.setLevel(source.skills[index].level);
   }
   auto training = person.initTraining();
   training.setSkill(encode_skill(source.training.skill));
   training.setNeededWeeks(source.training.needed_weeks);
   training.setCurrentWeeks(source.training.current_weeks);
}

void set_creation(rpc::PlayerCreation::Builder target,
                  const PlayerCreation& creation)
{
   target.setSetupRevision(creation.setup_revision);
   target.setStartingOfferId(creation.starting_offer_id);
   set_person(target.initCaptain(), creation.captain);
   target.setShipName(creation.ship_name);
   auto crew = target.initCrew(creation.crew.size());
   for(size_t index = 0; index < creation.crew.size(); ++index) {
      auto entry = crew[index];
      entry.setSlotId(creation.crew[index].slot_id);
      entry.setName(creation.crew[index].name);
      entry.setTrainingSkill(encode_skill(creation.crew[index].training_skill));
   }
   auto refits = target.initRefitOptionIds(creation.refit_option_ids.size());
   for(size_t index = 0; index < creation.refit_option_ids.size(); ++index) {
      refits.set(index, creation.refit_option_ids[index]);
   }
}

PlayerCreation decode_creation(const rpc::PlayerCreation::Reader source)
{
   PlayerCreation result{
      .setup_revision = source.getSetupRevision(),
      .starting_offer_id = source.getStartingOfferId(),
      .captain = decode_person(source.getCaptain()),
      .ship_name = source.getShipName().cStr(),
      .crew = {},
      .refit_option_ids = {},
   };
   for(const auto entry : source.getCrew()) {
      result.crew.push_back(InitialCrewDraft{
         .slot_id = entry.getSlotId(),
         .name = entry.getName().cStr(),
         .training_skill = decode_skill(entry.getTrainingSkill()),
      });
   }
   for(const auto option_id : source.getRefitOptionIds()) {
      result.refit_option_ids.push_back(option_id);
   }
   return result;
}

StartingShipOfferSummary decode_offer(
   const rpc::StartingShipOfferSummary::Reader offer)
{
   const auto career = [&offer] {
      switch(offer.getCareer())
   {
   case rpc::Career::TRADER:
      return Career::Trader;
   case rpc::Career::PRIVATEER:
      return Career::Privateer;
   case rpc::Career::NAVY:
      return Career::Navy;
   }
   throw std::runtime_error("unknown starting career");
}();
   return StartingShipOfferSummary{
      .offer_id = offer.getOfferId(),
      .career = career,
      .package_name = offer.getPackageName().cStr(),
      .ship_catalog_id = offer.getShipCatalogId(),
      .ship_name = offer.getShipName().cStr(),
      .role = offer.getRole().cStr(),
      .rationale = offer.getRationale().cStr(),
      .displacement_tons = offer.getDisplacementTons(),
      .jump_rating = offer.getJumpRating(),
      .thrust_g = offer.getThrustG(),
      .cargo_tons = offer.getCargoTons(),
      .crew_count = offer.getCrewCount(),
      .price_credits = offer.getPriceCredits(),
   };
}

CrewManagementSnapshot decode_crew_management(
   const rpc::Response::Reader response)
{
   if(!response.isCrewManagement()) {
      throw std::runtime_error("expected CrewManagementSnapshot");
   }
   const auto source = response.getCrewManagement();
   CrewManagementSnapshot result{
      .ship_id = source.getShipId(),
      .ship_name = source.getShipName().cStr(),
      .members = {},
      .roles = {},
      .established_complement = source.getEstablishedComplement(),
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto member : source.getMembers()) {
      CrewManagementMember decoded{
         .person_id = member.getPersonId(),
         .slot_id = member.getSlotId(),
         .role = member.getRole().cStr(),
         .represented_positions = member.getRepresentedPositions(),
         .captain = member.getCaptain(),
         .person = decode_person(member.getPerson()),
         .assigned_slot_ids = {},
         .condition = static_cast<PersonCondition>(member.getCondition()),
         .injury_points = member.getInjuryPoints(),
         .fatigue_points = member.getFatiguePoints(),
         .available = member.getAvailable(),
         .current_strength = member.getCurrentStrength(),
         .current_dexterity = member.getCurrentDexterity(),
         .current_endurance = member.getCurrentEndurance(),
         .service_kind = static_cast<CrewServiceKind>(member.getServiceKind()),
         .monthly_salary_credits = member.getMonthlySalaryCredits(),
         .arrears_credits = member.getArrearsCredits(),
         .prize_share_basis_points = member.getPrizeShareBasisPoints(),
         .morale = member.getMorale(),
         .loyalty = member.getLoyalty(),
         .risk_tolerance = member.getRiskTolerance(),
         .availability = static_cast<CrewAvailability>(member.getAvailability()),
         .available_second = member.getAvailableSecond(),
         .service_revision = member.getServiceRevision(),
         .shore_location = member.getShoreLocation().cStr(),
         .role_kind = static_cast<CrewRoleKind>(member.getRoleKind()),
         .location_kind = static_cast<CrewLocationKind>(member.getLocationKind()),
      };
      for(const auto slot_id : member.getAssignedSlotIds()) {
         decoded.assigned_slot_ids.push_back(slot_id);
      }
      result.members.push_back(std::move(decoded));
   }
   for(const auto role : source.getRoles()) {
      result.roles.push_back(CrewRole{
         .slot_id = role.getSlotId(),
         .role = role.getRole().cStr(),
         .represented_positions = role.getRepresentedPositions(),
         .role_kind = static_cast<CrewRoleKind>(role.getRoleKind()),
      });
   }
   return result;
}

ShipSubsystemKind decode_ship_subsystem_kind(
   const rpc::ShipSubsystemKind kind)
{
   switch(kind) {
   case rpc::ShipSubsystemKind::HULL:
      return ShipSubsystemKind::Hull;
   case rpc::ShipSubsystemKind::STRUCTURE:
      return ShipSubsystemKind::Structure;
   case rpc::ShipSubsystemKind::ARMOR:
      return ShipSubsystemKind::Armor;
   case rpc::ShipSubsystemKind::BRIDGE:
      return ShipSubsystemKind::Bridge;
   case rpc::ShipSubsystemKind::COMPUTER:
      return ShipSubsystemKind::Computer;
   case rpc::ShipSubsystemKind::SENSORS:
      return ShipSubsystemKind::Sensors;
   case rpc::ShipSubsystemKind::JUMP_DRIVE:
      return ShipSubsystemKind::JumpDrive;
   case rpc::ShipSubsystemKind::MANEUVER_DRIVE:
      return ShipSubsystemKind::ManeuverDrive;
   case rpc::ShipSubsystemKind::POWER_PLANT:
      return ShipSubsystemKind::PowerPlant;
   case rpc::ShipSubsystemKind::FUEL_SYSTEM:
      return ShipSubsystemKind::FuelSystem;
   case rpc::ShipSubsystemKind::LIFE_SUPPORT:
      return ShipSubsystemKind::LifeSupport;
   case rpc::ShipSubsystemKind::CARGO_HOLD:
      return ShipSubsystemKind::CargoHold;
   case rpc::ShipSubsystemKind::WEAPON_MOUNT:
      return ShipSubsystemKind::WeaponMount;
   case rpc::ShipSubsystemKind::SCREEN:
      return ShipSubsystemKind::Screen;
   case rpc::ShipSubsystemKind::HANGAR:
      return ShipSubsystemKind::Hangar;
   case rpc::ShipSubsystemKind::OTHER:
      return ShipSubsystemKind::Other;
   }
   return ShipSubsystemKind::Other;
}

ShipStatusSnapshot decode_ship_status(const rpc::Response::Reader response)
{
   if(!response.isShipStatus()) {
      throw std::runtime_error("expected ShipStatusSnapshot");
   }
   const auto source = response.getShipStatus();
   ShipStatusSnapshot result{
      .ship_revision = source.getShipRevision(),
      .ship_id = source.getShipId(),
      .ship_name = source.getShipName().cStr(),
      .catalog_id = source.getCatalogId(),
      .catalog_revision = source.getCatalogRevision(),
      .system_id = source.getSystemId(),
      .current_game_second = source.getCurrentGameSecond(),
      .displacement_millitons = source.getDisplacementMillitons(),
      .jump_rating = source.getJumpRating(),
      .thrust_g = source.getThrustG(),
      .fuel_capacity_millitons = source.getFuelCapacityMillitons(),
      .current_fuel_millitons = source.getCurrentFuelMillitons(),
      .jump_fuel_millitons = source.getJumpFuelMillitons(),
      .cargo_capacity_millitons = source.getCargoCapacityMillitons(),
      .monthly_maintenance_credits = source.getMonthlyMaintenanceCredits(),
      .next_maintenance_second = source.getNextMaintenanceSecond(),
      .maintenance_paid_through_second =
      source.getMaintenancePaidThroughSecond(),
            .maintenance_arrears_credits = source.getMaintenanceArrearsCredits(),
            .completed_maintenance_cycles = source.getCompletedMaintenanceCycles(),
            .consecutive_missed_maintenance =
            source.getConsecutiveMissedMaintenance(),
            .commissioned_second = source.getCommissionedSecond(),
            .transit_count = source.getTransitCount(),
            .warranty_expires_second = source.getWarrantyExpiresSecond(),
            .warranty_transit_limit = source.getWarrantyTransitLimit(),
            .warranty_repairs = source.getWarrantyRepairs(),
            .last_refit_second = source.getLastRefitSecond(),
            .completed_refits = source.getCompletedRefits(),
            .active_activity = decode_ship_activity(source.getActiveActivity()),
            .unrefined_fuel_millitons = source.getUnrefinedFuelMillitons(),
            .warranty_voided = source.getWarrantyVoided(),
            .monthly_life_support_credits = source.getMonthlyLifeSupportCredits(),
            .recovery_status = source.getRecoveryStatus().cStr(),
      .ammunition = {},
      .provisions = ShipProvisionStatus{
         .person_days_remaining = source.getProvisions().getPersonDaysRemaining(),
         .capacity_person_days = source.getProvisions().getCapacityPersonDays(),
      },
      .manifested_symptoms = {},
      .subsystems = {},
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto subsystem : source.getSubsystems()) {
      result.subsystems.push_back(ShipSubsystemStatus{
         .subsystem_id = subsystem.getSubsystemId(),
         .kind = decode_ship_subsystem_kind(subsystem.getKind()),
         .label = subsystem.getLabel().cStr(),
         .maximum_hits = subsystem.getMaximumHits(),
         .sustained_hits = subsystem.getSustainedHits(),
         .battlefield_repair_hits = subsystem.getBattlefieldRepairHits(),
         .effective_hits = subsystem.getEffectiveHits(),
         .operational_effect = subsystem.getOperationalEffect().cStr(),
         .last_proper_repair_second = subsystem.getLastProperRepairSecond(),
         .installed_second = subsystem.getInstalledSecond(),
         .last_refit_second = subsystem.getLastRefitSecond(),
         .calendar_age_months = subsystem.getCalendarAgeMonths(),
         .operating_seconds = subsystem.getOperatingSeconds(),
         .duty_cycles = subsystem.getDutyCycles(),
         .skimming_cycles = subsystem.getSkimmingCycles(),
         .neglect_damage_hits = subsystem.getNeglectDamageHits(),
         .displacement_millitons = subsystem.getDisplacementMillitons(),
         .replacement_price_credits = subsystem.getReplacementPriceCredits(),
         .installation_generation = subsystem.getInstallationGeneration(),
         .reconditioned = subsystem.getReconditioned(),
      });
   }
   for(const auto lot : source.getAmmunition()) {
      result.ammunition.push_back(ShipAmmunitionStatus{
         .ammunition_id = lot.getAmmunitionId().cStr(),
         .remaining = lot.getRemaining(),
         .capacity = lot.getCapacity(),
         .pack_units = lot.getPackUnits(),
         .price_per_pack_credits = lot.getPricePerPackCredits(),
      });
   }
   for(const auto symptom : source.getManifestedSymptoms()) {
      result.manifested_symptoms.emplace_back(symptom.cStr());
   }
   return result;
}

DockedFuelServiceKind decode_docked_fuel_kind(const rpc::DockedFuelServiceKind kind)
{
   switch(kind) {
   case rpc::DockedFuelServiceKind::REFINED:
      return DockedFuelServiceKind::Refined;
   case rpc::DockedFuelServiceKind::UNREFINED:
      return DockedFuelServiceKind::Unrefined;
   case rpc::DockedFuelServiceKind::GAS_GIANT:
      return DockedFuelServiceKind::GasGiant;
   case rpc::DockedFuelServiceKind::WILDERNESS_WATER:
      return DockedFuelServiceKind::WildernessWater;
   }
   throw std::runtime_error("unknown docked fuel service");
}

DockedServices decode_docked_services(const rpc::Response::Reader response)
{
   if(!response.isDockedServices()) {
      throw std::runtime_error("expected DockedServices");
   }
   const auto source = response.getDockedServices();
   DockedServices result{
      .ship_revision = source.getShipRevision(),
      .current_game_second = source.getCurrentGameSecond(),
      .fuel = {},
      .ammunition = {},
      .provisions = {
         source.getProvisions().getPersonDaysRemaining(),
         source.getProvisions().getCapacityPersonDays(),
      },
      .provision_package_person_days = source.getProvisionPackagePersonDays(),
      .provision_package_price_credits = source.getProvisionPackagePriceCredits(),
      .provisions_available = source.getProvisionsAvailable(),
      .ammunition_available = source.getAmmunitionAvailable(),
      .repair = {},
      .refit_available = source.getRefitAvailable(),
      .refit_unavailable_reason = source.getRefitUnavailableReason().cStr(),
      .refit_cost_credits = source.getRefitCostCredits(),
      .refit_service_seconds = source.getRefitServiceSeconds(),
   };
   for(const auto item : source.getFuel()) {
      result.fuel.push_back(DockedFuelService{
         .kind = decode_docked_fuel_kind(item.getKind()),
         .label = item.getLabel().cStr(),
         .source_body_id = item.getHasSourceBody()
            ? std::optional<uint32_t>(item.getSourceBodyId())
            : std::nullopt,
         .available = item.getAvailable(),
         .unavailable_reason = item.getUnavailableReason().cStr(),
         .price_per_ton_credits = item.getPricePerTonCredits(),
         .maximum_millitons = item.getMaximumMillitons(),
         .service_seconds = item.getServiceSeconds(),
      });
   }
   for(const auto lot : source.getAmmunition()) {
      result.ammunition.push_back(ShipAmmunitionStatus{
         .ammunition_id = lot.getAmmunitionId().cStr(),
         .remaining = lot.getRemaining(),
         .capacity = lot.getCapacity(),
         .pack_units = lot.getPackUnits(),
         .price_per_pack_credits = lot.getPricePerPackCredits(),
      });
   }
   for(const auto item : source.getRepair()) {
      result.repair.push_back(DockedRepairService{
         .subsystem_id = item.getSubsystemId(),
         .label = item.getLabel().cStr(),
         .available = item.getAvailable(),
         .unavailable_reason = item.getUnavailableReason().cStr(),
         .cost_credits = item.getCostCredits(),
         .service_seconds = item.getServiceSeconds(),
         .replacement = item.getReplacement(),
         .reconditioned = item.getReconditioned(),
      });
   }
   return result;
}

DockedSnapshot decode_docked_snapshot(const rpc::Response::Reader response)
{
   if(!response.isDockedSnapshot()) {
      throw std::runtime_error("expected DockedSnapshot");
   }
   const auto source = response.getDockedSnapshot();
   return DockedSnapshot{
      .ship_id = source.getShipId(),
      .ship_name = source.getShipName().cStr(),
      .system_id = source.getSystemId(),
      .system_name = source.getSystemName().cStr(),
      .world_id = source.getWorldId(),
      .world_name = source.getWorldName().cStr(),
      .facility_id = source.getFacilityId(),
      .facility_name = source.getFacilityName().cStr(),
      .starport = source.getStarport().cStr(),
      .tech_level = source.getTechLevel(),
      .population = source.getPopulation(),
      .law_level = source.getLawLevel(),
      .arrived_second = source.getArrivedSecond(),
      .credits = source.getCredits(),
      .restricted_credits = source.getRestrictedCredits(),
      .debt_credits = source.getDebtCredits(),
      .fuel_millitons = source.getFuelMillitons(),
      .fuel_capacity_millitons = source.getFuelCapacityMillitons(),
      .refined_fuel_price_per_ton = source.getRefinedFuelPricePerTon(),
      .unrefined_fuel_millitons = source.getUnrefinedFuelMillitons(),
      .unrefined_fuel_price_per_ton = source.getUnrefinedFuelPricePerTon(),
      .accrued_berth_fee_credits = source.getAccruedBerthFeeCredits(),
      .facility_revision = source.getFacilityRevision(),
      .personnel_available = source.getPersonnelAvailable(),
      .banking_available = source.getBankingAvailable(),
      .authority_available = source.getAuthorityAvailable(),
      .medical_level = source.getMedicalLevel(),
      .clearance_required = source.getClearanceRequired(),
      .cargo_used_millitons = source.getCargoUsedMillitons(),
      .cargo_capacity_millitons = source.getCargoCapacityMillitons(),
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

KnownDestinations decode_known_destinations(const rpc::Response::Reader response)
{
   if(!response.isKnownDestinations()) {
      throw std::runtime_error("expected KnownDestinations");
   }
   const auto source = response.getKnownDestinations();
   KnownDestinations result{
      .current_system_id = source.getCurrentSystemId(),
      .jump_rating = source.getJumpRating(),
      .systems = {},
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto system : source.getSystems()) {
      result.systems.push_back(KnownSystemSummary{
         .system_id = system.getSystemId(),
         .system_name = system.getSystemName().cStr(),
         .world_name = system.getWorldName().cStr(),
         .distance_parsecs = system.getDistanceParsecs(),
         .within_jump_rating = system.getWithinJumpRating(),
         .starport = system.getStarport().cStr(),
         .population = system.getPopulation(),
         .tech_level = system.getTechLevel(),
         .observed_second = system.getObservedSecond(),
         .source = system.getSource().cStr(),
         .coreward_parsecs = system.getCorewardParsecs(),
         .spinward_parsecs = system.getSpinwardParsecs(),
         .north_parsecs = system.getNorthParsecs(),
         .remote_candidate = system.getRemoteCandidate(),
         .knowledge_source = static_cast<SystemKnowledgeSource>(
            system.getKnowledgeSource()),
         .gas_giant_count = system.getGasGiantCount(),
      });
   }
   return result;
}

CourseFuelSource decode_course_fuel_source(const rpc::CourseFuelSource source)
{
   switch(source) {
   case rpc::CourseFuelSource::NONE:
      return CourseFuelSource::None;
   case rpc::CourseFuelSource::CARRIED:
      return CourseFuelSource::Carried;
   case rpc::CourseFuelSource::REFINED_PORT:
      return CourseFuelSource::RefinedPort;
   case rpc::CourseFuelSource::FRONTIER_SKIMMING:
      return CourseFuelSource::FrontierSkimming;
   case rpc::CourseFuelSource::UNREFINED_PORT:
      return CourseFuelSource::UnrefinedPort;
   }
   throw std::runtime_error("unknown course fuel source");
}

CoursePlan decode_course_plan(const rpc::CoursePlan::Reader source)
{
   CoursePlan result{
      .available = source.getAvailable(),
      .elapsed_seconds = source.getElapsedSeconds(),
      .fuel_cost_credits = source.getFuelCostCredits(),
      .total_milliparsecs = source.getTotalMilliparsecs(),
      .waypoints = {},
   };
   for(const auto waypoint : source.getWaypoints()) {
      result.waypoints.push_back(CourseWaypoint{
         .system_id = waypoint.getSystemId(),
         .system_name = waypoint.getSystemName().cStr(),
         .world_name = waypoint.getWorldName().cStr(),
         .fuel_source = decode_course_fuel_source(waypoint.getFuelSource()),
         .next_leg_milliparsecs = waypoint.getNextLegMilliparsecs(),
      });
   }
   return result;
}

CoursePlot decode_course_plot(const rpc::Response::Reader response)
{
   if(!response.isCoursePlot()) {
      throw std::runtime_error("expected CoursePlot");
   }
   const auto source = response.getCoursePlot();
   return CoursePlot{
      .origin_system_id = source.getOriginSystemId(),
      .destination_system_id = source.getDestinationSystemId(),
      .jump_rating = source.getJumpRating(),
      .fastest = decode_course_plan(source.getFastest()),
      .cheapest = decode_course_plan(source.getCheapest()),
      .current_game_second = source.getCurrentGameSecond(),
      .clock_rate_game_seconds = source.getClockRateGameSeconds(),
      .clock_rate_real_seconds = source.getClockRateRealSeconds(),
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

MessageClass decode_message_class(const rpc::MessageClass source)
{
   switch(source) {
   case rpc::MessageClass::AGENCY_NEWS:
      return MessageClass::AgencyNews;
   case rpc::MessageClass::PUBLIC_SERVICE:
      return MessageClass::PublicService;
   case rpc::MessageClass::CONTRACT_OFFER:
      return MessageClass::ContractOffer;
   case rpc::MessageClass::TRAFFIC_NOTICE:
      return MessageClass::TrafficNotice;
   case rpc::MessageClass::PRIVATE_MESSAGE:
      return MessageClass::Private;
   }
   throw std::runtime_error("unknown message class");
}

MessageImportance decode_message_importance(const rpc::MessageImportance source)
{
   switch(source) {
   case rpc::MessageImportance::ROUTINE:
      return MessageImportance::Routine;
   case rpc::MessageImportance::NOTABLE:
      return MessageImportance::Notable;
   case rpc::MessageImportance::IMPORTANT:
      return MessageImportance::Important;
   case rpc::MessageImportance::HEADLINE:
      return MessageImportance::Headline;
   }
   throw std::runtime_error("unknown message importance");
}

rpc::MessageClass encode_message_class(const MessageClass source)
{
   switch(source) {
   case MessageClass::AgencyNews:
      return rpc::MessageClass::AGENCY_NEWS;
   case MessageClass::PublicService:
      return rpc::MessageClass::PUBLIC_SERVICE;
   case MessageClass::ContractOffer:
      return rpc::MessageClass::CONTRACT_OFFER;
   case MessageClass::TrafficNotice:
      return rpc::MessageClass::TRAFFIC_NOTICE;
   case MessageClass::Private:
      return rpc::MessageClass::PRIVATE_MESSAGE;
   }
   throw std::runtime_error("unknown message class");
}

rpc::MessageImportance encode_message_importance(const MessageImportance source)
{
   switch(source) {
   case MessageImportance::Routine:
      return rpc::MessageImportance::ROUTINE;
   case MessageImportance::Notable:
      return rpc::MessageImportance::NOTABLE;
   case MessageImportance::Important:
      return rpc::MessageImportance::IMPORTANT;
   case MessageImportance::Headline:
      return rpc::MessageImportance::HEADLINE;
   }
   throw std::runtime_error("unknown message importance");
}

MessageClassification decode_message_classification(
   const rpc::MessageClassification source)
{
   switch(source) {
   case rpc::MessageClassification::UNREVIEWED:
      return MessageClassification::Unreviewed;
   case rpc::MessageClassification::IGNORED:
      return MessageClassification::Ignored;
   case rpc::MessageClassification::REVIEW_LATER:
      return MessageClassification::ReviewLater;
   case rpc::MessageClassification::ACTIONED:
      return MessageClassification::Actioned;
   case rpc::MessageClassification::ARCHIVED:
      return MessageClassification::Archived;
   }
   throw std::runtime_error("unknown message classification");
}

rpc::MessageClassification encode_message_classification(
   const MessageClassification source)
{
   switch(source) {
   case MessageClassification::Unreviewed:
      return rpc::MessageClassification::UNREVIEWED;
   case MessageClassification::Ignored:
      return rpc::MessageClassification::IGNORED;
   case MessageClassification::ReviewLater:
      return rpc::MessageClassification::REVIEW_LATER;
   case MessageClassification::Actioned:
      return rpc::MessageClassification::ACTIONED;
   case MessageClassification::Archived:
      return rpc::MessageClassification::ARCHIVED;
   }
   throw std::runtime_error("unknown message classification");
}

MessageItem decode_message_item(const rpc::MessageItem::Reader source)
{
   return MessageItem{
      .message_id = source.getMessageId(),
      .origin_system_id = source.getOriginSystemId(),
      .origin_system_name = source.getOriginSystemName().cStr(),
      .created_second = source.getCreatedSecond(),
      .available_second = source.getAvailableSecond(),
      .expires_second = source.getExpiresSecond(),
      .message_class = decode_message_class(source.getClass()),
      .importance = decode_message_importance(source.getImportance()),
      .subject = source.getSubject().cStr(),
      .body = source.getBody().cStr(),
      .offer_id = source.getOfferId() == 0
      ? std::optional<uint64_t>{}
:
      std::optional<uint64_t>{source.getOfferId()},
      .offer_revision = source.getOfferRevision(),
      .offer_available = source.getOfferAvailable(),
      .classification =
      decode_message_classification(source.getClassification()),
      .previously_seen = source.getPreviouslySeen(),
      .expired = source.getExpired(),
      .action_kind = static_cast<MessageActionKind>(source.getActionKind()),
      .action_reference_id = source.getActionReferenceId(),
   };
}

SystemMappingState decode_system_mapping_state(
   const rpc::SystemMappingState source)
{
   switch(source) {
   case rpc::SystemMappingState::KNOWN_PUBLIC:
      return SystemMappingState::KnownPublic;
   case rpc::SystemMappingState::UNRESOLVED:
      return SystemMappingState::Unresolved;
   case rpc::SystemMappingState::PUBLIC_DISPATCHED:
      return SystemMappingState::PublicDispatched;
   case rpc::SystemMappingState::DIRECT_DISPATCHED:
      return SystemMappingState::DirectDispatched;
   case rpc::SystemMappingState::WITHHELD:
      return SystemMappingState::Withheld;
   case rpc::SystemMappingState::SECRET:
      return SystemMappingState::Secret;
   }
   throw std::runtime_error("unknown system mapping state");
}

SystemMappingStatus decode_system_mapping_status(
   const rpc::SystemMappingStatus::Reader source,
   const rpc::Response::Reader response)
{
   return SystemMappingStatus{
      .system_id = source.getSystemId(),
      .state = decode_system_mapping_state(source.getState()),
      .dispatch_message_id = source.getDispatchMessageId() == 0
      ? std::nullopt
      : std::optional<uint64_t>{
         source.getDispatchMessageId()
      },
      .changed_second = source.getChangedSecond(),
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

ArrivalPacket decode_arrival_packet(const rpc::Response::Reader response)
{
   if(!response.isArrivalPacket()) {
      throw std::runtime_error("expected ArrivalPacket");
   }
   const auto source = response.getArrivalPacket();
   ArrivalPacket result{
      .new_arrival = source.getNewArrival(),
      .system_id = source.getSystemId(),
      .system_name = source.getSystemName().cStr(),
      .arrival_second = source.getArrivalSecond(),
      .mailbag_id = source.getMailbagId() == 0
      ? std::nullopt
      : std::optional<uint64_t>{source.getMailbagId()},
      .mail_delivered = source.getMailDelivered(),
      .mail_forwarded = source.getMailForwarded(),
      .mail_expired = source.getMailExpired(),
      .stipend_credits = source.getStipendCredits(),
      .items = {},
      .mapping_status = decode_system_mapping_status(
         source.getMappingStatus(), response),
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto item : source.getItems()) {
      result.items.push_back(decode_message_item(item));
   }
   return result;
}

rpc::SystemMappingChoice encode_system_mapping_choice(
   const SystemMappingChoice choice)
{
   switch(choice) {
   case SystemMappingChoice::PublicNotification:
      return rpc::SystemMappingChoice::PUBLIC_NOTIFICATION;
   case SystemMappingChoice::DirectEarth:
      return rpc::SystemMappingChoice::DIRECT_EARTH;
   case SystemMappingChoice::Withhold:
      return rpc::SystemMappingChoice::WITHHOLD;
   case SystemMappingChoice::WithholdSecret:
      return rpc::SystemMappingChoice::WITHHOLD_SECRET;
   }
   throw std::runtime_error("unknown system mapping choice");
}

MessageManagement decode_message_management(
   const rpc::Response::Reader response)
{
   if(!response.isMessageManagement()) {
      throw std::runtime_error("expected MessageManagement");
   }
   MessageManagement result{
      .items = {},
      .filters = {},
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto item : response.getMessageManagement().getItems()) {
      result.items.push_back(decode_message_item(item));
   }
   for(const auto filter : response.getMessageManagement().getFilters()) {
      result.filters.push_back(MessageFilter{
         .message_class = decode_message_class(filter.getClass()),
         .minimum_importance =
         decode_message_importance(filter.getMinimumImportance()),
      });
   }
   return result;
}

RadioTransmissionKind decode_radio_kind(const rpc::RadioTransmissionKind kind)
{
   switch(kind) {
   case rpc::RadioTransmissionKind::PLAYER_BROADCAST:
      return RadioTransmissionKind::PlayerBroadcast;
   case rpc::RadioTransmissionKind::INSPECTION_ORDER:
      return RadioTransmissionKind::InspectionOrder;
   case rpc::RadioTransmissionKind::BOARDING_ORDER:
      return RadioTransmissionKind::BoardingOrder;
   case rpc::RadioTransmissionKind::SURRENDER_DEMAND:
      return RadioTransmissionKind::SurrenderDemand;
   }
   throw std::runtime_error("unknown radio transmission kind");
}

SystemRadioSnapshot decode_system_radio(const rpc::Response::Reader response)
{
   if(!response.isSystemRadio()) {
      throw std::runtime_error("expected SystemRadioSnapshot");
   }
   const auto source = response.getSystemRadio();
   SystemRadioSnapshot result{
      .ship_id = source.getShipId(),
      .system_id = source.getSystemId(),
      .current_second = source.getCurrentSecond(),
      .can_transmit = source.getCanTransmit(),
      .unavailable_reason = source.getUnavailableReason().cStr(),
      .entries = {},
      .mutes = {},
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto entry : source.getEntries()) {
      const auto sender = entry.getSender();
      result.entries.push_back(RadioInboxEntry{
         .reception_id = entry.getReceptionId(),
         .transmission_id = entry.getTransmissionId(),
         .receiving_ship_id = entry.getReceivingShipId(),
         .sender_ship_id = entry.getSenderShipId(),
         .sender_ship_name = entry.getSenderShipName().cStr(),
         .sender_transponder = entry.getSenderTransponder().cStr(),
         .sender = {
            .bbs_id = sender.getBbsId(),
            .player_id = sender.getPlayerId(),
         },
         .emitted_second = entry.getEmittedSecond(),
         .received_second = entry.getReceivedSecond(),
         .expires_second = entry.getExpiresSecond(),
         .kind = decode_radio_kind(entry.getKind()),
         .actionable = entry.getActionable(),
         .action_reference_id = entry.getActionReferenceId(),
      });
   }
   for(const auto mute : source.getMutes()) {
      const auto sender = mute.getSender();
      result.mutes.push_back(PlayerIdentity{
         .bbs_id = sender.getBbsId(),
         .player_id = sender.getPlayerId(),
      });
   }
   return result;
}

TaskOffer decode_task_offer(const rpc::TaskOffer::Reader source)
{
   TaskOffer result{
      .offer_id = source.getOfferId(),
      .revision = source.getRevision(),
      .kind = static_cast<TaskKind>(source.getKind()),
      .title = source.getTitle().cStr(),
      .origin_system_id = source.getOriginSystemId(),
      .destination_system_id = source.getDestinationSystemId(),
      .commodity_id = source.getCommodityId(),
      .quantity_millitons = source.getQuantityMillitons(),
      .passenger_count = source.getPassengerCount(),
      .payment_credits = source.getPaymentCredits(),
      .collateral_credits = source.getCollateralCredits(),
      .expires_second = source.getExpiresSecond(),
      .delivery_deadline_second = source.getDeliveryDeadlineSecond(),
      .legal = source.getLegal(),
      .partial_delivery_allowed = source.getPartialDeliveryAllowed(),
      .failure_penalty_credits = source.getFailurePenaltyCredits(),
      .recurrence_seconds = source.getRecurrenceSeconds(),
      .performance_count = source.getPerformanceCount(),
      .passenger_class = static_cast<PassengerClass>(source.getPassengerClass()),
      .late_deduction_per_day_credits = source.getLateDeductionPerDayCredits(),
      .non_delivery_liability_credits = source.getNonDeliveryLiabilityCredits(),
      .passenger_grace_seconds = source.getPassengerGraceSeconds(),
      .declared_value_credits = source.getDeclaredValueCredits(),
      .unavailable_reasons = {},
   };
   for(const auto reason : source.getUnavailableReasons()) {
      result.unavailable_reasons.emplace_back(reason.cStr());
   }
   return result;
}

TaskRecord decode_task_record(const rpc::TaskRecord::Reader source)
{
   return {
      .task_id = source.getTaskId(),
      .offer = decode_task_offer(source.getOffer()),
      .state = static_cast<TaskState>(source.getState()),
      .accepted_second = source.getAcceptedSecond(),
      .delivered_quantity_millitons = source.getDeliveredQuantityMillitons(),
      .reserved_cargo_millitons = source.getReservedCargoMillitons(),
      .reserved_passenger_count = source.getReservedPassengerCount(),
      .reserved_credits = source.getReservedCredits(),
      .status_text = source.getStatusText().cStr(),
      .performances_completed = source.getPerformancesCompleted(),
      .revision = source.getRevision(),
      .claim_message_id = source.getClaimMessageId(),
      .result_message_id = source.getResultMessageId(),
      .known_result = source.getKnownResult(),
      .loaded_second = source.getLoadedSecond(),
      .settled_second = source.getSettledSecond(),
      .insurance_claim_id = source.getInsuranceClaimId(),
      .dispute_message_id = source.getDisputeMessageId(),
      .dispute_effect = source.getDisputeEffect(),
      .adjudication_message_id = source.getAdjudicationMessageId(),
      .performing_ship_id = source.getPerformingShipId(),
   };
}

CarriageDeclaration decode_carriage(
   const rpc::CarriageDeclaration::Reader source)
{
   return {
      .plan_revision = source.getPlanRevision(),
      .destination_system_id = source.getDestinationSystemId(),
      .freight_capacity_millitons = source.getFreightCapacityMillitons(),
      .high_berths = source.getHighBerths(),
      .middle_berths = source.getMiddleBerths(),
      .steerage_berths = source.getSteerageBerths(),
      .low_berths = source.getLowBerths(),
      .accept_electronic_mail = source.getAcceptElectronicMail(),
   };
}

WorkAssignment decode_work_assignment(const rpc::WorkAssignment::Reader source)
{
   return {
      .assignment_id = source.getAssignmentId(),
      .kind = static_cast<MarketSearchKind>(source.getKind()),
      .method = static_cast<MarketSearchMethod>(source.getMethod()),
      .person_id = source.getPersonId(),
      .commodity_id = source.getCommodityId(),
      .destination_system_id = source.getDestinationSystemId(),
      .started_second = source.getStartedSecond(),
      .due_second = source.getDueSecond(),
      .state = static_cast<WorkState>(source.getState()),
      .result_text = source.getResultText().cStr(),
   };
}

MarketSnapshot decode_market(const rpc::Response::Reader response)
{
   if(!response.isMarket()) {
      throw std::runtime_error("expected MarketSnapshot");
   }
   const auto source = response.getMarket();
   MarketSnapshot result{
      .market_revision = source.getMarketRevision(),
      .system_id = source.getSystemId(),
      .world_name = source.getWorldName().cStr(),
      .generated_day = source.getGeneratedDay(),
      .credits = source.getCredits(),
      .cargo_used_millitons = source.getCargoUsedMillitons(),
      .cargo_capacity_millitons = source.getCargoCapacityMillitons(),
      .offers = {},
      .cargo = {},
      .trade_codes = {},
      .tariff_basis_points = source.getTariffBasisPoints(),
      .local_task_offers = {},
      .work_assignments = {},
      .leads = {},
      .events = {},
      .cargo_sale_quotes = {},
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto offer : source.getOffers()) {
      result.offers.push_back(MarketOffer{
         .offer_id = offer.getOfferId(),
         .commodity_id = offer.getCommodityId(),
         .commodity_name = offer.getCommodityName().cStr(),
         .base_price_per_ton = offer.getBasePricePerTon(),
         .purchase_price_per_ton = offer.getPurchasePricePerTon(),
         .sale_price_per_ton = offer.getSalePricePerTon(),
         .available_millitons = offer.getAvailableMillitons(),
         .legality = static_cast<uint8_t>(offer.getLegality()),
      });
   }
   for(const auto lot : source.getCargo()) {
      result.cargo.push_back(CargoLot{
         .cargo_lot_id = lot.getCargoLotId(),
         .commodity_id = lot.getCommodityId(),
         .commodity_name = lot.getCommodityName().cStr(),
         .quantity_millitons = lot.getQuantityMillitons(),
         .purchase_price_per_ton = lot.getPurchasePricePerTon(),
         .origin_system_id = lot.getOriginSystemId(),
         .acquired_second = lot.getAcquiredSecond(),
         .title = static_cast<uint8_t>(lot.getTitle()),
         .task_id = lot.getTaskId(),
         .unique_object_id = lot.getUniqueObjectId(),
         .condition_percent = lot.getConditionPercent(),
         .destination_system_id = lot.getDestinationSystemId(),
      });
   }
   for(const auto quote : source.getCargoSaleQuotes()) {
      result.cargo_sale_quotes.push_back({
         .cargo_lot_id = quote.getCargoLotId(),
         .price_per_ton = quote.getPricePerTon(),
      });
   }
   for(const auto assignment : source.getWorkAssignments()) {
      result.work_assignments.push_back(decode_work_assignment(assignment));
   }
   for(const auto lead : source.getLeads()) {
      result.leads.push_back({
         .lead_id = lead.getLeadId(),
         .revision = lead.getRevision(),
         .side = static_cast<MarketLeadSide>(lead.getSide()),
         .state = static_cast<MarketLeadState>(lead.getState()),
         .system_id = lead.getSystemId(),
         .commodity_id = lead.getCommodityId(),
         .commodity_name = lead.getCommodityName().cStr(),
         .quantity_millitons = lead.getQuantityMillitons(),
         .price_per_ton = lead.getPricePerTon(),
         .discovered_second = lead.getDiscoveredSecond(),
         .expires_second = lead.getExpiresSecond(),
         .reservation_expires_second = lead.getReservationExpiresSecond(),
         .escrow_credits = lead.getEscrowCredits(),
         .source = lead.getSource().cStr(),
         .confidence_percent = lead.getConfidencePercent(),
      });
   }
   for(const auto event : source.getEvents()) {
      result.events.push_back({
         .event_id = event.getEventId(),
         .kind = static_cast<MarketEventKind>(event.getKind()),
         .commodity_id = event.getCommodityId(),
         .commodity_name = event.getCommodityName().cStr(),
         .start_second = event.getStartSecond(),
         .expires_second = event.getExpiresSecond(),
         .stock_multiplier_basis_points = event.getStockMultiplierBasisPoints(),
         .purchase_tier_delta = event.getPurchaseTierDelta(),
         .sale_tier_delta = event.getSaleTierDelta(),
         .supplier_offer_multiplier_basis_points = event.getSupplierOfferMultiplierBasisPoints(),
         .buyer_offer_multiplier_basis_points = event.getBuyerOfferMultiplierBasisPoints(),
         .carriage_offer_multiplier_basis_points = event.getCarriageOfferMultiplierBasisPoints(),
         .headline = event.getHeadline().cStr(),
      });
   }
   for(const auto code : source.getTradeCodes()) {
      result.trade_codes.emplace_back(code.cStr());
   }
   for(const auto offer : source.getLocalTaskOffers()) {
      result.local_task_offers.push_back(decode_task_offer(offer));
   }
   return result;
}

TravelStatus decode_travel_status(const rpc::Response::Reader response)
{
   if(!response.isTravelStatus()) {
      throw std::runtime_error("expected TravelStatus");
   }
   const auto source = response.getTravelStatus();
   return TravelStatus{
      .ship_id = source.getShipId(),
      .ship_name = source.getShipName().cStr(),
      .current_system_id = source.getCurrentSystemId(),
      .current_system_name = source.getCurrentSystemName().cStr(),
      .destination_system_id = source.getDestinationSystemId(),
      .destination_system_name = source.getDestinationSystemName().cStr(),
      .stage = decode_travel_stage(source.getStage()),
      .current_game_second = source.getCurrentGameSecond(),
      .due_second = source.getDueSecond(),
      .current_fuel_millitons = source.getCurrentFuelMillitons(),
      .jump_fuel_millitons = source.getJumpFuelMillitons(),
      .clock_rate_game_seconds = source.getClockRateGameSeconds(),
      .clock_rate_real_seconds = source.getClockRateRealSeconds(),
      .plan_id = source.getPlanId(),
      .plan_revision = source.getPlanRevision(),
      .leg_index = source.getLegIndex(),
      .origin = decode_flight_locus(source.getOrigin()),
      .destination = decode_flight_locus(source.getDestination()),
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

TrafficContact decode_traffic_contact(const rpc::TrafficContact::Reader source)
{
   TrafficMovementKind movement;
   switch(source.getMovement()) {
   case rpc::TrafficMovementKind::ARRIVAL:
      movement = TrafficMovementKind::Arrival;
      break;
   case rpc::TrafficMovementKind::DEPARTURE:
      movement = TrafficMovementKind::Departure;
      break;
   case rpc::TrafficMovementKind::PRESENT:
      movement = TrafficMovementKind::Present;
      break;
   }
   return TrafficContact{
      .contact_id = source.getContactId(),
      .catalog_id = source.getCatalogId(),
      .class_name = source.getClassName().cStr(),
      .ship_name = source.getShipName().cStr(),
      .transponder = source.getTransponder().cStr(),
      .operator_name = source.getOperatorName().cStr(),
      .role = source.getRole().cStr(),
      .displacement_millitons = source.getDisplacementMillitons(),
      .origin_system_id = source.getOriginSystemId(),
      .destination_system_id = source.getDestinationSystemId(),
      .movement = movement,
      .edge_second = source.getEdgeSecond(),
      .resolution = static_cast<TrafficContactResolution>(source.getResolution()),
      .confidence_percent = source.getConfidencePercent(),
      .player_owned = source.getPlayerOwned(),
      .online_controlled = source.getOnlineControlled(),
      .attachment = static_cast<TrafficAttachment>(source.getAttachment()),
   };
}

void initialize_request(rpc::Envelope::Builder envelope,
                        const uint64_t session_epoch,
                        const uint64_t request_id,
                        const std::array<uint8_t, 16>& command_id,
                        rpc::Request::Builder& request)
{
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   envelope.setSessionEpoch(session_epoch);
   request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
}

}  // namespace

PlayerCreated create_player(TlsConnection& connection,
                            const uint64_t session_epoch,
                            const PlayerCreation& creation,
                            const std::array<uint8_t, 16>& command_id,
                            const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   set_creation(request.initCreatePlayer(), creation);
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());

   const auto response_words =
      receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(response_words);
   const auto response_envelope = reader.getRoot<rpc::Envelope>();
   const auto response = checked_response(response_envelope, command_id);
   if(response.getPhase() != rpc::Phase::DOCKED ||
         !response.isPlayerCreated()) {
      throw std::runtime_error("CreatePlayer did not enter the docked phase");
   }
   auto returned = decode_creation(response.getPlayerCreated());
   if(returned != creation) {
      throw std::runtime_error("server returned a different creation record");
   }
   return PlayerCreated{
      .creation = std::move(returned),
      .committed_sequence = response.getCommittedSequence(),
   };
}

CaptainCreationOptions get_captain_creation_options(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetCaptainCreationOptions();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response =
      checked_response(reader.getRoot<rpc::Envelope>(), command_id);
   if(!response.isCaptainCreationOptions()) {
      throw std::runtime_error("expected CaptainCreationOptions");
   }
   const auto source = response.getCaptainCreationOptions();
   const auto point_buy = source.getCharacteristicPointBuy();
   CaptainCreationOptions result{
      .setup_revision = source.getSetupRevision(),
      .characteristic_point_buy =
      {
         .minimum = point_buy.getMinimum(),
         .maximum = point_buy.getMaximum(),
         .neutral = point_buy.getNeutral(),
         .budget = point_buy.getBudget(),
      },
      .skill_pool = decode_pool(source.getSkillPool()),
      .permitted_skills = {},
      .default_captain = decode_person(source.getDefaultCaptain()),
   };
   if(result.characteristic_point_buy.minimum >
         result.characteristic_point_buy.maximum ||
         result.characteristic_point_buy.neutral <
         result.characteristic_point_buy.minimum ||
         result.characteristic_point_buy.neutral >
         result.characteristic_point_buy.maximum) {
      throw std::runtime_error("server returned an invalid characteristic point buy");
   }
   const std::array<uint8_t, 6> default_scores{
      result.default_captain.characteristics.strength,
      result.default_captain.characteristics.dexterity,
      result.default_captain.characteristics.endurance,
      result.default_captain.characteristics.intelligence,
      result.default_captain.characteristics.education,
      result.default_captain.characteristics.charisma,
   };
   int default_cost = 0;
   for(const auto score : default_scores) {
      if(score < result.characteristic_point_buy.minimum ||
            score > result.characteristic_point_buy.maximum) {
         throw std::runtime_error(
            "server returned a default captain outside the characteristic range");
      }
      default_cost += static_cast<int>(score) -
                      static_cast<int>(result.characteristic_point_buy.neutral);
   }
   if(default_cost != result.characteristic_point_buy.budget) {
      throw std::runtime_error(
         "server returned a default captain with the wrong characteristic cost");
   }
   for(const auto definition : source.getPermittedSkills()) {
      result.permitted_skills.push_back(SkillDefinition{
         .id = decode_skill(definition.getId()),
         .name = definition.getName().cStr(),
      });
   }
   return result;
}

StartingShipOffers get_starting_ship_offers(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetStartingShipOffers();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response =
      checked_response(reader.getRoot<rpc::Envelope>(), command_id);
   if(!response.isStartingShipOffers()) {
      throw std::runtime_error("expected StartingShipOffers");
   }
   const auto source = response.getStartingShipOffers();
   const auto origin = source.getOrigin();
   StartingShipOffers result{
      .setup_revision = source.getSetupRevision(),
      .origin =
      {
         .bbs_name = origin.getBbsName().cStr(),
         .polity_name = origin.getPolityName().cStr(),
         .home_system_name = origin.getHomeSystemName().cStr(),
         .home_world_name = origin.getHomeWorldName().cStr(),
         .trade_combat = origin.getTradeCombat(),
         .chaos_order = origin.getChaosOrder(),
      },
      .offers = {},
   };
   for(const auto offer : source.getOffers()) {
      result.offers.push_back(decode_offer(offer));
   }
   return result;
}

StartingShipOptions get_starting_ship_options(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t setup_revision,
   const uint32_t starting_offer_id,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto query = request.initGetStartingShipOptions();
   query.setSetupRevision(setup_revision);
   query.setStartingOfferId(starting_offer_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response =
      checked_response(reader.getRoot<rpc::Envelope>(), command_id);
   if(!response.isStartingShipOptions()) {
      throw std::runtime_error("expected StartingShipOptions");
   }
   const auto source = response.getStartingShipOptions();
   StartingShipOptions result{
      .setup_revision = source.getSetupRevision(),
      .offer = decode_offer(source.getOffer()),
      .description_paragraphs = {},
      .terms = {},
      .refit_groups = {},
   };
   for(const auto paragraph : source.getDescriptionParagraphs()) {
      result.description_paragraphs.emplace_back(paragraph.cStr());
   }
   const auto terms = source.getTerms();
   result.terms = {
      .terms_revision = terms.getTermsRevision(),
      .title = static_cast<StartingShipOptions::TitleKind>(terms.getTitle()),
      .equity_credits = terms.getEquityCredits(),
      .principal_credits = terms.getPrincipalCredits(),
      .monthly_payment_credits = terms.getMonthlyPaymentCredits(),
      .liquid_reserve_credits = terms.getLiquidReserveCredits(),
      .restricted_reserve_credits = terms.getRestrictedReserveCredits(),
      .monthly_compensation_credits = terms.getMonthlyCompensationCredits(),
      .refit_credit_limit = terms.getRefitCreditLimit(),
      .refit_displacement_millitons = terms.getRefitDisplacementMillitons(),
      .authority = terms.getAuthority().cStr(),
      .exit_terms = terms.getExitTerms().cStr(),
      .insurance = terms.getInsurance().cStr(),
   };
   for(const auto group : source.getRefitGroups()) {
      StartingShipOptions::RefitGroup decoded{
         .group_id = group.getGroupId(),
         .name = group.getName().cStr(),
         .required = group.getRequired(),
         .options = {},
      };
      for(const auto option : group.getOptions()) {
         decoded.options.push_back({
            .option_id = option.getOptionId(),
            .name = option.getName().cStr(),
            .description = option.getDescription().cStr(),
            .displacement_delta_millitons = option.getDisplacementDeltaMillitons(),
            .price_delta_credits = option.getPriceDeltaCredits(),
         });
      }
      result.refit_groups.push_back(std::move(decoded));
   }
   return result;
}

StartingCrewPlan get_starting_crew_plan(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t setup_revision,
   const uint32_t starting_offer_id,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto query = request.initGetStartingCrewPlan();
   query.setSetupRevision(setup_revision);
   query.setStartingOfferId(starting_offer_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response =
      checked_response(reader.getRoot<rpc::Envelope>(), command_id);
   if(!response.isStartingCrewPlan()) {
      throw std::runtime_error("expected StartingCrewPlan");
   }
   const auto source = response.getStartingCrewPlan();
   StartingCrewPlan result{
      .setup_revision = source.getSetupRevision(),
      .starting_offer_id = source.getStartingOfferId(),
      .slots = {},
   };
   for(const auto slot : source.getSlots()) {
      result.slots.push_back(StartingCrewSlot{
         .slot_id = slot.getSlotId(),
         .role = slot.getRole().cStr(),
         .represented_positions = slot.getRepresentedPositions(),
         .required = slot.getRequired(),
         .skill_pool = decode_pool(slot.getSkillPool()),
         .default_crew = decode_person(slot.getDefaultCrew()),
         .role_kind = static_cast<CrewRoleKind>(slot.getRoleKind()),
      });
   }
   return result;
}

CrewManagementSnapshot get_crew_management(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetCrewManagement();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_crew_management(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

CrewManagementSnapshot set_crew_training_target(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t person_id,
   const SkillId skill,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initSetCrewTrainingTarget();
   change.setPersonId(person_id);
   change.setSkill(encode_skill(skill));
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_crew_management(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

CrewManagementSnapshot set_crew_assignments(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t person_id,
   const std::vector<uint16_t>& slot_ids,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initSetCrewAssignments();
   change.setPersonId(person_id);
   auto assignments = change.initSlotIds(slot_ids.size());
   for(size_t index = 0; index < slot_ids.size(); ++index) {
      assignments.set(index, slot_ids[index]);
   }
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_crew_management(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

CrewManagementSnapshot apply_personnel_action(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t person_id,
   const uint64_t expected_service_revision,
   const PersonnelActionKind action,
   const uint64_t target_ship_id,
   const uint16_t duration_days,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initApplyPersonnelAction();
   change.setPersonId(person_id);
   change.setExpectedServiceRevision(expected_service_revision);
   change.setAction(static_cast<rpc::PersonnelActionKind>(action));
   change.setTargetShipId(target_ship_id);
   change.setDurationDays(duration_days);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_crew_management(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

ShipStatusSnapshot get_ship_status(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetShipStatus();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_ship_status(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

DockedServices get_docked_services(TlsConnection& connection, const uint64_t session_epoch,
                                   const std::array<uint8_t, 16>& command_id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetDockedServices();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_docked_services(checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

ShipStatusSnapshot commit_docked_service(TlsConnection& connection, const uint64_t session_epoch,
      const DockedServiceOrder& order, const std::array<uint8_t, 16>& command_id,
      const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto target = request.initCommitDockedService();
   target.setExpectedShipRevision(order.expected_ship_revision);
   switch(order.kind) {
   case DockedServiceOrder::Kind::Fuel: {
      auto value = target.initFuel();
      value.setKind(static_cast<rpc::DockedFuelServiceKind>(order.fuel_kind));
      value.setHasSourceBody(order.source_body_id.has_value());
      value.setSourceBodyId(order.source_body_id.value_or(0));
      value.setQuantityMillitons(order.quantity_millitons);
      break;
   }
   case DockedServiceOrder::Kind::Ammunition: {
      auto value = target.initAmmunition();
      value.setAmmunitionId(order.ammunition_id);
      value.setPacks(order.packs);
      break;
   }
   case DockedServiceOrder::Kind::Provisions:
      target.setProvisions(order.packages);
      break;
   case DockedServiceOrder::Kind::ProperRepair:
      target.setProperRepair(order.subsystem_id);
      break;
   case DockedServiceOrder::Kind::Refit:
      target.setRefit();
      break;
   case DockedServiceOrder::Kind::Replacement: {
      auto value = target.initReplacement();
      value.setSubsystemId(order.subsystem_id);
      value.setReconditioned(order.reconditioned);
      break;
   }
   }
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_ship_status(checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

DockedSnapshot get_docked_snapshot(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetDockedSnapshot();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_docked_snapshot(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

KnownDestinations get_known_destinations(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetKnownDestinations();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_known_destinations(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

CoursePlot plot_course(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t origin_system_id,
   const uint64_t destination_system_id,
   const bool use_current_fuel,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto query = request.initPlotCourse();
   query.setOriginSystemId(origin_system_id);
   query.setDestinationSystemId(destination_system_id);
   query.setUseCurrentFuel(use_current_fuel);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_course_plot(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

MarketSnapshot get_market(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetMarket();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_market(checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

MarketSnapshot buy_cargo(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t market_revision,
   const uint64_t offer_id,
   const uint64_t quantity_millitons,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initBuyCargo();
   change.setMarketRevision(market_revision);
   change.setOfferId(offer_id);
   change.setQuantityMillitons(quantity_millitons);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_market(checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

MarketSnapshot sell_cargo(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t market_revision,
   const uint64_t cargo_lot_id,
   const uint64_t quantity_millitons,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initSellCargo();
   change.setMarketRevision(market_revision);
   change.setCargoLotId(cargo_lot_id);
   change.setQuantityMillitons(quantity_millitons);
   change.setBuyerLeadId(0);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_market(checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

MarketSnapshot sell_cargo_to_lead(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t market_revision,
   const uint64_t cargo_lot_id,
   const uint64_t quantity_millitons,
   const uint64_t buyer_lead_id,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initSellCargo();
   change.setMarketRevision(market_revision);
   change.setCargoLotId(cargo_lot_id);
   change.setQuantityMillitons(quantity_millitons);
   change.setBuyerLeadId(buyer_lead_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_market(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

TaskLedger decode_task_ledger(const rpc::Response::Reader response)
{
   if(!response.isTaskLedger()) {
      throw std::runtime_error("expected TaskLedger");
   }
   const auto source = response.getTaskLedger();
   TaskLedger result{
      .current_second = source.getCurrentSecond(),
      .available_credits = source.getAvailableCredits(),
      .reserved_credits = source.getReservedCredits(),
      .reserved_cargo_millitons = source.getReservedCargoMillitons(),
      .reserved_passenger_count = source.getReservedPassengerCount(),
      .tasks = {},
      .local_offers = {},
      .carriage = decode_carriage(source.getCarriage()),
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto task : source.getTasks()) {
      result.tasks.push_back(decode_task_record(task));
   }
   for(const auto offer : source.getLocalOffers()) {
      result.local_offers.push_back(decode_task_offer(offer));
   }
   return result;
}

TaskLedger get_task_ledger(
   TlsConnection& connection,
   const uint64_t epoch,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetTaskLedger();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_task_ledger(
             checked_response(reader.getRoot<rpc::Envelope>(), id));
}

TaskLedger accept_task_offer(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t offer_id,
   const uint64_t revision,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto accept = request.initAcceptTaskOffer();
   accept.setOfferId(offer_id);
   accept.setExpectedRevision(revision);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_task_ledger(
             checked_response(reader.getRoot<rpc::Envelope>(), id));
}

TaskLedger set_carriage_declaration(
   TlsConnection& connection,
   const uint64_t epoch,
   const CarriageDeclaration& value,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto carriage = request.initSetCarriageDeclaration();
   carriage.setExpectedPlanRevision(value.plan_revision);
   carriage.setDestinationSystemId(value.destination_system_id);
   carriage.setFreightCapacityMillitons(value.freight_capacity_millitons);
   carriage.setHighBerths(value.high_berths);
   carriage.setMiddleBerths(value.middle_berths);
   carriage.setSteerageBerths(value.steerage_berths);
   carriage.setLowBerths(value.low_berths);
   carriage.setAcceptElectronicMail(value.accept_electronic_mail);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_task_ledger(
             checked_response(reader.getRoot<rpc::Envelope>(), id));
}

MarketSnapshot begin_market_search(
   TlsConnection& connection,
   const uint64_t epoch,
   const MarketSearchKind kind,
   const MarketSearchMethod method,
   const uint64_t person_id,
   const uint16_t commodity_id,
   const uint64_t destination_system_id,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto search = request.initBeginMarketSearch();
   search.setKind(static_cast<rpc::MarketSearchKind>(kind));
   search.setMethod(static_cast<rpc::MarketSearchMethod>(method));
   search.setPersonId(person_id);
   search.setCommodityId(commodity_id);
   search.setDestinationSystemId(destination_system_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_market(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

MarketSnapshot cancel_work_assignment(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t assignment_id,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initCancelWorkAssignment().setAssignmentId(assignment_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_market(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

MarketSnapshot reserve_market_lead(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t lead_id,
   const uint64_t expected_revision,
   const uint64_t quantity_millitons,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto reservation = request.initReserveMarketLead();
   reservation.setLeadId(lead_id);
   reservation.setExpectedRevision(expected_revision);
   reservation.setQuantityMillitons(quantity_millitons);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_market(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

MarketSnapshot release_market_reservation(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t lead_id,
   const uint64_t expected_revision,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto release = request.initReleaseMarketReservation();
   release.setLeadId(lead_id);
   release.setExpectedRevision(expected_revision);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_market(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

TaskLedger apply_task_action(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t task_id,
   const uint64_t expected_revision,
   const TaskActionKind action,
   const std::string& explanation,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto task_action = request.initApplyTaskAction();
   task_action.setTaskId(task_id);
   task_action.setExpectedRevision(expected_revision);
   task_action.setAction(static_cast<rpc::TaskActionKind>(action));
   task_action.setExplanation(explanation);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_task_ledger(
             checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FinanceSnapshot decode_finance_snapshot(const rpc::Response::Reader response)
{
   if(!response.isFinance()) {
      throw std::runtime_error("expected FinanceSnapshot");
   }
   const auto finance = response.getFinance();
   return {
      .title = static_cast<ShipTitleKind>(finance.getTitle()),
      .liquid_credits = finance.getLiquidCredits(),
      .restricted_credits = finance.getRestrictedCredits(),
      .reserved_credits = finance.getReservedCredits(),
      .original_hull_price_credits = finance.getOriginalHullPriceCredits(),
      .principal_credits = finance.getPrincipalCredits(),
      .monthly_payment_credits = finance.getMonthlyPaymentCredits(),
      .monthly_insurance_escrow_credits = finance.getMonthlyInsuranceEscrowCredits(),
      .next_payment_due_second = finance.getNextPaymentDueSecond(),
      .grace_expires_second = finance.getGraceExpiresSecond(),
      .paid_through_second = finance.getPaidThroughSecond(),
      .in_default = finance.getInDefault(),
      .impound_order_known_locally = finance.getImpoundOrderKnownLocally(),
      .credit_status = finance.getCreditStatus().cStr(),
      .destination_assistance_active = finance.getDestinationAssistanceActive(),
      .destination_assistance_expires_second = finance.getDestinationAssistanceExpiresSecond(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

FinanceSnapshot get_finance(
   TlsConnection& connection,
   const uint64_t epoch,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetFinance();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_finance_snapshot(
             checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FinanceSnapshot purchase_insurance(
   TlsConnection& connection,
   const uint64_t epoch,
   const InsuranceKind kind,
   const bool enabled,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto insurance = request.initPurchaseInsurance();
   insurance.setKind(static_cast<rpc::InsuranceKind>(kind));
   insurance.setEnabled(enabled);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_finance_snapshot(
             checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FinanceSnapshot misappropriate_restricted_credits(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t amount,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initMisappropriateRestrictedCredits().setAmount(amount);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_finance_snapshot(
             checked_response(reader.getRoot<rpc::Envelope>(), id));
}

MarketKnowledge get_market_knowledge(
   TlsConnection& connection,
   const uint64_t epoch,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetMarketKnowledge();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response = checked_response(reader.getRoot<rpc::Envelope>(), id);
   if(!response.isMarketKnowledge()) {
      throw std::runtime_error("expected MarketKnowledge");
   }
   const auto source = response.getMarketKnowledge();
   MarketKnowledge result{
      .current_second = source.getCurrentSecond(),
      .observations = {},
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto observation : source.getObservations()) {
      result.observations.push_back({
         .system_id = observation.getSystemId(),
         .system_name = observation.getSystemName().cStr(),
         .commodity_id = observation.getCommodityId(),
         .commodity_name = observation.getCommodityName().cStr(),
         .observed_second = observation.getObservedSecond(),
         .acquired_second = observation.getAcquiredSecond(),
         .source = observation.getSource().cStr(),
         .confidence_percent = observation.getConfidencePercent(),
         .minimum_price_per_ton = observation.getMinimumPricePerTon(),
         .maximum_price_per_ton = observation.getMaximumPricePerTon(),
         .minimum_available_millitons = observation.getMinimumAvailableMillitons(),
         .maximum_available_millitons = observation.getMaximumAvailableMillitons(),
      });
   }
   return result;
}

ShipMarket decode_ship_market(const rpc::Response::Reader response)
{
   if(!response.isShipMarket()) {
      throw std::runtime_error("expected ShipMarket");
   }
   auto source = response.getShipMarket();
   ShipMarket result{
      .generated_day = source.getGeneratedDay(),
      .current_ship_trade_in_credits = source.getCurrentShipTradeInCredits(),
      .outstanding_lien_credits = source.getOutstandingLienCredits(),
      .offers = {},
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto offer : source.getOffers()) {
      result.offers.push_back({
         .offer_id = offer.getOfferId(),
         .catalog_id = offer.getCatalogId(),
         .class_name = offer.getClassName().cStr(),
         .price_credits = offer.getPriceCredits(),
         .original_price_credits = offer.getOriginalPriceCredits(),
         .used = offer.getUsed(),
         .age_months = offer.getAgeMonths(),
         .visible_condition_percent = offer.getVisibleConditionPercent(),
         .cargo_capacity_millitons = offer.getCargoCapacityMillitons(),
         .jump_rating = offer.getJumpRating(),
         .minimum_crew = offer.getMinimumCrew(),
      });
   }
   return result;
}
ShipMarket get_ship_market(TlsConnection& connection, const uint64_t epoch,
                           const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetShipMarket();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_ship_market(checked_response(reader.getRoot<rpc::Envelope>(), id));
}
ShipMarket purchase_ship(TlsConnection& connection, const uint64_t epoch, const uint64_t offer_id,
                         const bool trade_in, const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto purchase = request.initPurchaseShip();
   purchase.setOfferId(offer_id);
   purchase.setTradeInCurrentShip(trade_in);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_ship_market(checked_response(reader.getRoot<rpc::Envelope>(), id));
}
CrewMarket decode_crew_market(const rpc::Response::Reader response)
{
   if(!response.isCrewMarket()) {
      throw std::runtime_error("expected CrewMarket");
   }
   auto source = response.getCrewMarket();
   CrewMarket result{
      .generated_day = source.getGeneratedDay(),
      .candidates = {},
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto candidate : source.getCandidates()) {
      result.candidates.push_back({
         .candidate_id = candidate.getCandidateId(),
         .role = candidate.getRole().cStr(),
         .name = candidate.getName().cStr(),
         .primary_skill = static_cast<SkillId>(candidate.getPrimarySkill()),
         .skill_level = candidate.getSkillLevel(),
         .monthly_salary_credits = candidate.getMonthlySalaryCredits(),
      });
   }
   return result;
}
CrewMarket get_crew_market(TlsConnection& connection, const uint64_t epoch,
                           const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetCrewMarket();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_crew_market(checked_response(reader.getRoot<rpc::Envelope>(), id));
}
CrewMarket hire_crew(TlsConnection& connection, const uint64_t epoch, const uint64_t candidate_id,
                     const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initHireCrew().setCandidateId(candidate_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_crew_market(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FleetSnapshot decode_fleet(const rpc::Response::Reader response)
{
   if(!response.isFleet()) {
      throw std::runtime_error("expected FleetSnapshot");
   }
   const auto source = response.getFleet();
   FleetSnapshot result{
      .revision = source.getRevision(),
      .active_ship_id = source.getActiveShipId(),
      .ships = {},
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto wire_ship : source.getShips()) {
      result.ships.push_back({
         .ship_id = wire_ship.getShipId(),
         .name = wire_ship.getName().cStr(),
         .class_name = wire_ship.getClassName().cStr(),
         .catalog_id = wire_ship.getCatalogId(),
         .system_id = wire_ship.getSystemId(),
         .system_name = wire_ship.getSystemName().cStr(),
         .location = wire_ship.getLocation().cStr(),
         .title = static_cast<ShipTitleKind>(wire_ship.getTitle()),
         .active = wire_ship.getActive(),
         .commanding_person_id = wire_ship.getCommandingPersonId(),
         .commanding_person_name = wire_ship.getCommandingPersonName().cStr(),
         .standing_order = static_cast<ManagedShipOrderKind>(wire_ship.getStandingOrder()),
         .can_assume_command = wire_ship.getCanAssumeCommand(),
         .fuel_millitons = wire_ship.getFuelMillitons(),
         .fuel_capacity_millitons = wire_ship.getFuelCapacityMillitons(),
         .cargo_used_millitons = wire_ship.getCargoUsedMillitons(),
         .cargo_capacity_millitons = wire_ship.getCargoCapacityMillitons(),
         .provision_person_days = wire_ship.getProvisionPersonDays(),
         .provision_capacity_person_days = wire_ship.getProvisionCapacityPersonDays(),
         .cargo = {},
         .ammunition = {},
         .online_controlled = wire_ship.getOnlineControlled(),
      });
      auto& ship = result.ships.back();
      for(const auto lot : wire_ship.getCargo()) {
         ship.cargo.push_back({
            .cargo_lot_id = lot.getCargoLotId(),
            .commodity_id = lot.getCommodityId(),
            .commodity_name = lot.getCommodityName().cStr(),
            .quantity_millitons = lot.getQuantityMillitons(),
            .purchase_price_per_ton = lot.getPurchasePricePerTon(),
            .origin_system_id = lot.getOriginSystemId(),
            .acquired_second = lot.getAcquiredSecond(),
            .title = static_cast<uint8_t>(lot.getTitle()),
            .task_id = lot.getTaskId(),
            .unique_object_id = lot.getUniqueObjectId(),
            .condition_percent = lot.getConditionPercent(),
            .destination_system_id = lot.getDestinationSystemId(),
         });
      }
      for(const auto lot : wire_ship.getAmmunition()) {
         ship.ammunition.push_back({
            .ammunition_id = lot.getAmmunitionId().cStr(),
            .remaining = lot.getRemaining(),
            .capacity = lot.getCapacity(),
            .pack_units = lot.getPackUnits(),
            .price_per_pack_credits = lot.getPricePerPackCredits(),
         });
      }
   }
   return result;
}

FleetSnapshot get_fleet(
   TlsConnection& connection,
   const uint64_t epoch,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetFleet();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_fleet(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FleetSnapshot set_active_ship(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t expected_revision,
   const uint64_t ship_id,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto change = request.initSetActiveShip();
   change.setExpectedRevision(expected_revision);
   change.setShipId(ship_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_fleet(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FleetSnapshot assign_ship_captain(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t expected_revision,
   const uint64_t ship_id,
   const uint64_t person_id,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto change = request.initAssignShipCaptain();
   change.setExpectedRevision(expected_revision);
   change.setShipId(ship_id);
   change.setPersonId(person_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_fleet(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FleetSnapshot transfer_ship_stores(
   TlsConnection& connection,
   const uint64_t epoch,
   const uint64_t expected_revision,
   const uint64_t from_ship_id,
   const uint64_t to_ship_id,
   const StoreTransferKind kind,
   const uint64_t cargo_lot_id,
   const std::string& item_id,
   const uint64_t quantity,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto transfer = request.initTransferShipStores();
   transfer.setExpectedRevision(expected_revision);
   transfer.setFromShipId(from_ship_id);
   transfer.setToShipId(to_ship_id);
   transfer.setKind(static_cast<rpc::StoreTransferKind>(kind));
   transfer.setCargoLotId(cargo_lot_id);
   transfer.setItemId(item_id);
   transfer.setQuantity(quantity);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_fleet(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

TravelStatus get_travel_status(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetTravelStatus();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_travel_status(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

TravelStatus begin_voyage(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t destination_system_id,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.initBeginVoyage().setDestinationSystemId(destination_system_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_travel_status(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

FlightPlanSnapshot decode_flight_plan_snapshot(const rpc::Response::Reader response)
{
   if(!response.isFlightPlan()) {
      throw std::runtime_error("expected FlightPlanSnapshot");
   }
   auto source = response.getFlightPlan();
   FlightPlanSnapshot result{
      .plan_id = source.getPlanId(),
      .revision = source.getRevision(),
      .current_step = source.getCurrentStep(),
      .state = static_cast<FlightPlanState>(source.getState()),
      .steps = {},
      .policy = decode_policy(source.getPolicy()),
      .suspension_reason = source.getSuspensionReason().cStr(),
      .phase = decode_response_phase(response.getPhase()),
   };

   for(auto step : source.getSteps()) {
      result.steps.push_back(decode_plan_step(step));
   }
   return result;
}

FlightPlanSnapshot get_flight_plan(TlsConnection& connection, const uint64_t epoch,
                                   const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetFlightPlan();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_flight_plan_snapshot(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FlightPlanPreview preview_flight_plan(
   TlsConnection& connection,
   const uint64_t epoch,
   const FlightPlanProposal& proposal,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   encode_proposal(request.initPreviewFlightPlan(), proposal);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response = checked_response(reader.getRoot<rpc::Envelope>(), id);
   if(!response.isFlightPlanPreview()) {
      throw std::runtime_error("expected FlightPlanPreview");
   }
   const auto source = response.getFlightPlanPreview();
   FlightPlanPreview result{
      .proposal = decode_proposal(source.getProposal()),
      .preview_hash = {},
      .elapsed_seconds = source.getElapsedSeconds(),
      .fuel_millitons = source.getFuelMillitons(),
      .warnings = {},
      .carriage_offers = {},
      .carriage_revenue_credits = source.getCarriageRevenueCredits(),
      .carriage_broker_fees_credits = source.getCarriageBrokerFeesCredits(),
   };
   const auto hash = source.getPreviewHash();
   result.preview_hash.assign(hash.begin(), hash.end());
   for(const auto warning : source.getWarnings()) {
      result.warnings.push_back({
         .code = warning.getCode().cStr(),
         .message = warning.getMessage().cStr(),
      });
   }
   for(const auto offer : source.getCarriageOffers()) {
      result.carriage_offers.push_back(decode_task_offer(offer));
   }
   return result;
}

FlightPlanSnapshot commit_flight_plan(TlsConnection& connection, const uint64_t epoch,
                                      const FlightPlanProposal& proposal,
                                      const std::vector<uint8_t>& hash,
                                      const bool acknowledge,
                                      const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto commit = request.initCommitFlightPlan();
   encode_proposal(commit.initProposal(), proposal);
   commit.setPreviewHash(kj::arrayPtr(reinterpret_cast<const kj::byte*>(hash.data()), hash.size()));
   commit.setAcknowledgeWarnings(acknowledge);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_flight_plan_snapshot(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

CheckpointSnapshot acknowledge_checkpoint(TlsConnection& connection, const uint64_t epoch,
      const uint64_t checkpoint_id, const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initAcknowledgeCheckpoint().setCheckpointId(checkpoint_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   auto response = checked_response(reader.getRoot<rpc::Envelope>(), id);
   if(!response.isCheckpoint()) {
      throw std::runtime_error("expected CheckpointSnapshot");
   }
   auto source = response.getCheckpoint();
   return {
      .checkpoint_id = source.getCheckpointId(),
      .plan_id = source.getPlanId(),
      .plan_revision = source.getPlanRevision(),
      .step_index = source.getStepIndex(),
      .locus = decode_flight_locus(source.getLocus()),
      .kind = static_cast<CheckpointKind>(source.getKind()),
      .ready_second = source.getReadySecond(),
      .acknowledged = source.getAcknowledged(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

EncounterSnapshot get_encounter(TlsConnection& connection, const uint64_t epoch,
                                const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetEncounter();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   auto response = checked_response(reader.getRoot<rpc::Envelope>(), id);
   if(!response.isEncounter()) {
      throw std::runtime_error("expected EncounterSnapshot");
   }
   auto s = response.getEncounter();
   auto c = s.getContact();
   return {
      .encounter_id = s.getEncounterId(),
      .revision = s.getRevision(),
      .kind = static_cast<EncounterKind>(s.getKind()),
      .state = static_cast<EncounterState>(s.getState()),
      .started_second = s.getStartedSecond(),
      .next_turn_second = s.getNextTurnSecond(),
      .turn = s.getTurn(),
      .contact = {
         .contact_id = c.getContactId(),
         .ship_name = c.getShipName().cStr(),
         .class_name = c.getClassName().cStr(),
         .transponder = c.getTransponder().cStr(),
         .role = c.getRole().cStr(),
         .range = c.getRange().cStr(),
         .confidence_percent = c.getConfidencePercent(),
      },
      .summary = s.getSummary().cStr(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

EncounterResult resolve_encounter(TlsConnection& connection, const uint64_t epoch,
                                  const uint64_t encounter_id, const uint64_t revision, const EncounterPosture posture,
                                  const std::vector<EncounterFallback>& fallbacks, const std::array<uint8_t, 16>& id,
                                  const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope=message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto resolve = request.initResolveEncounter();
   resolve.setEncounterId(encounter_id);
   resolve.setExpectedRevision(revision);
   resolve.setPosture(static_cast<rpc::EncounterPosture>(posture));
   auto list = resolve.initFallbacks(fallbacks.size());
   for(size_t i = 0; i < fallbacks.size(); ++i) {
      list.set(i, static_cast<rpc::EncounterFallback>(fallbacks[i]));
   }
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   auto response = checked_response(reader.getRoot<rpc::Envelope>(), id);
   if(!response.isEncounterResult()) {
      throw std::runtime_error("expected EncounterResult");
   }
   auto s = response.getEncounterResult();
   return {
      .encounter_id = s.getEncounterId(),
      .resolved = s.getResolved(),
      .terminal = s.getTerminal(),
      .outcome = s.getOutcome().cStr(),
      .turns = s.getTurns(),
      .cargo_lost_millitons = s.getCargoLostMillitons(),
      .fuel_lost_millitons = s.getFuelLostMillitons(),
      .damage_hits = s.getDamageHits(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

namespace
{

void encode_combat_order(rpc::CombatOrderSet::Builder target, const CombatOrderSet& source)
{
   target.setCombatId(source.combat_id);
   target.setViewRevision(source.view_revision);
   target.setUseTacticalController(source.use_tactical_controller);
   auto actions = target.initActions(source.actions.size());
   for(size_t index = 0; index < source.actions.size(); ++index) {
      auto item = actions[index];
      item.setKind(static_cast<rpc::CombatActionKind>(source.actions[index].kind));
      item.setMountId(source.actions[index].mount_id);
      item.setTargetVesselId(source.actions[index].target_vessel_id);
      item.setActorPersonId(source.actions[index].actor_person_id);
   }
   auto reactions = target.initReactions(source.reactions.size());
   for(size_t index = 0; index < source.reactions.size(); ++index) {
      auto item = reactions[index];
      item.setKind(static_cast<rpc::CombatReaction>(source.reactions[index].kind));
      item.setActorPersonId(source.reactions[index].actor_person_id);
   }
}

CombatSnapshot decode_combat(const rpc::Response::Reader response)
{
   if(!response.isCombat()) {
      throw std::runtime_error("expected CombatSnapshot");
   }
   auto source = response.getCombat();
   CombatSnapshot result{
      .combat_id = source.getCombatId(),
      .revision = source.getRevision(),
      .round = source.getRound(),
      .round_started_second = source.getRoundStartedSecond(),
      .order_due_second = source.getOrderDueSecond(),
      .order_window_real_milliseconds = source.getOrderWindowRealMilliseconds(),
      .range = static_cast<CombatRange>(source.getRange()),
      .participants = {},
      .default_order = {},
      .policy = {},
      .player_order_submitted = source.getPlayerOrderSubmitted(),
      .complete = source.getComplete(),
      .log = {},
      .actors = {},
      .phase = decode_response_phase(response.getPhase()),
   };
   for(auto participant : source.getParticipants()) {
      CombatParticipant item{
         .vessel_id = participant.getVesselId(),
         .side = participant.getSide(),
         .name = participant.getName().cStr(),
         .class_name = participant.getClassName().cStr(),
         .initiative = participant.getInitiative(),
         .thrust = participant.getThrust(),
         .hull_remaining = participant.getHullRemaining(),
         .structure_remaining = participant.getStructureRemaining(),
         .armor_remaining = participant.getArmorRemaining(),
         .disposition = static_cast<CombatDisposition>(participant.getDisposition()),
         .weapons = {},
         .commanded = participant.getCommanded(),
         .player_owned = participant.getPlayerOwned(),
         .online_controlled = participant.getOnlineControlled(),
      };
      for(auto mount : participant.getWeapons()) {
         CombatWeaponMount fitted{
            .mount_id = mount.getMountId(),
            .label = mount.getLabel().cStr(),
            .weapons = {},
            .damage_hits = mount.getDamageHits(),
            .ammunition_remaining = mount.getAmmunitionRemaining(),
         };

         for(auto weapon : mount.getWeapons()) {
            fitted.weapons.emplace_back(weapon.cStr());
         }
         item.weapons.push_back(std::move(fitted));
      }
      result.participants.push_back(std::move(item));
   }
   auto order = source.getDefaultOrder();
   result.default_order = {
      .combat_id = order.getCombatId(),
      .view_revision = order.getViewRevision(),
      .actions = {},
      .reactions = {},
      .use_tactical_controller = order.getUseTacticalController(),
   };

   for(auto action : order.getActions()) {
      result.default_order.actions.push_back({
         .kind = static_cast<CombatActionKind>(action.getKind()),
         .mount_id = action.getMountId(),
         .target_vessel_id = action.getTargetVesselId(),
         .actor_person_id = action.getActorPersonId(),
      });
   }

   for(auto reaction : order.getReactions()) {
      result.default_order.reactions.push_back({
         .kind = static_cast<CombatReaction>(reaction.getKind()),
         .actor_person_id = reaction.getActorPersonId(),
      });
   }
   auto policy = source.getPolicy();
   result.policy = {
      .expected_revision = policy.getExpectedRevision(),
      .minimum_victory_percent = policy.getMinimumVictoryPercent(),
      .objective = static_cast<CombatObjective>(policy.getObjective()),
      .permit_surrender = policy.getPermitSurrender(),
      .permit_abandon_ship = policy.getPermitAbandonShip(),
   };
   for(auto line : source.getLog()) {
      result.log.emplace_back(line.cStr());
   }
   for(auto actor : source.getActors()) {
      result.actors.push_back({
         .person_id = actor.getPersonId(),
         .name = actor.getName().cStr(),
         .station = actor.getStation().cStr(),
         .available = actor.getAvailable(),
         .action_budget = actor.getActionBudget(),
         .allowed_actions = {},
         .allowed_reactions = {},
      });
      for(const auto action : actor.getAllowedActions()) {
         result.actors.back().allowed_actions.push_back(
            static_cast<CombatActionKind>(action));
      }
      for(const auto reaction : actor.getAllowedReactions()) {
         result.actors.back().allowed_reactions.push_back(
            static_cast<CombatReaction>(reaction));
      }
   }
   return result;
}

}

CombatSnapshot get_combat(TlsConnection& connection, const uint64_t epoch,
                          const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetCombat();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

CombatSnapshot submit_combat_order(TlsConnection& connection, const uint64_t epoch,
                                   const CombatOrderSet& order,
                                   const std::array<uint8_t, 16>& id,
                                   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   encode_combat_order(request.initSubmitCombatOrder(), order);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

CombatSnapshot set_combat_automation_policy(TlsConnection& connection, const uint64_t epoch,
      const CombatAutomationPolicy& policy, const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto target = request.initSetCombatAutomationPolicy();
   target.setExpectedRevision(policy.expected_revision);
   target.setMinimumVictoryPercent(policy.minimum_victory_percent);
   target.setObjective(static_cast<rpc::CombatObjective>(policy.objective));
   target.setPermitSurrender(policy.permit_surrender);
   target.setPermitAbandonShip(policy.permit_abandon_ship);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

namespace
{

CombatCareerSnapshot decode_combat_career(const rpc::Response::Reader response)
{
   if(!response.isCombatCareer()) {
      throw std::runtime_error("expected CombatCareerSnapshot");
   }
   const auto source = response.getCombatCareer();
   CombatCareerSnapshot result{
      .revision = source.getRevision(),
      .mode = static_cast<CombatCareerMode>(source.getMode()),
      .rank = source.getRank().cStr(),
      .service_points = source.getServicePoints(),
      .monthly_salary_credits = source.getMonthlySalaryCredits(),
      .next_naval_board_second = source.getNextNavalBoardSecond(),
      .public_heat = source.getPublicHeat(),
      .underworld_standing = source.getUnderworldStanding(),
      .crew_pressure = source.getCrewPressure(),
      .opportunities = {},
      .prizes = {},
      .warrants = {},
      .cruise = {},
      .local_enforcement_summary = source.getLocalEnforcementSummary().cStr(),
      .system_contacts = {},
      .local_contacts = {},
      .interception_watch = std::nullopt,
      .phase = decode_response_phase(response.getPhase()),
   };
   for(const auto value : source.getOpportunities()) {
      result.opportunities.push_back({
         .opportunity_id = value.getOpportunityId(),
         .kind = static_cast<CareerOpportunityKind>(value.getKind()),
         .state = static_cast<CareerOpportunityState>(value.getState()),
         .issued_system_id = value.getIssuedSystemId(),
         .target_system_id = value.getTargetSystemId(),
         .target_contact_id = value.getTargetContactId(),
         .issued_second = value.getIssuedSecond(),
         .expires_second = value.getExpiresSecond(),
         .reward_credits = value.getRewardCredits(),
         .service_points = value.getServicePoints(),
         .authority = value.getAuthority().cStr(),
         .objective = value.getObjective().cStr(),
         .objective_kind = static_cast<CareerObjectiveKind>(value.getObjectiveKind()),
         .evidence_kind = static_cast<CareerObjectiveEvidenceKind>(value.getEvidenceKind()),
         .evidence_second = value.getEvidenceSecond(),
         .evidence_vessel_id = value.getEvidenceVesselId(),
         .order_message_id = value.getOrderMessageId(),
         .report_message_id = value.getReportMessageId(),
      });
   }
   for(const auto value : source.getPrizes()) {
      result.prizes.push_back({
         .prize_id = value.getPrizeId(),
         .captured_vessel_id = value.getCapturedVesselId(),
         .surviving_crew_count = value.getSurvivingCrewCount(),
         .catalog_id = value.getCatalogId(),
         .name = value.getName().cStr(),
         .gross_value_credits = value.getGrossValueCredits(),
         .realizable_value_credits = value.getRealizableValueCredits(),
         .condition_percent = value.getConditionPercent(),
         .status = static_cast<PrizeStatus>(value.getStatus()),
         .secured_second = value.getSecuredSecond(),
         .claim_message_id = value.getClaimMessageId(),
         .settlement_credits = value.getSettlementCredits(),
         .advance_credits = value.getAdvanceCredits(),
      });
   }
   for(const auto value : source.getWarrants()) {
      result.warrants.push_back({
         .warrant_id = value.getWarrantId(),
         .issuing_polity_id = value.getIssuingPolityId(),
         .origin_system_id = value.getOriginSystemId(),
         .filed_second = value.getFiledSecond(),
         .message_id = value.getMessageId(),
         .severity = value.getSeverity(),
         .bounty_credits = value.getBountyCredits(),
         .evidence_percent = value.getEvidencePercent(),
         .status = static_cast<WarrantStatus>(value.getStatus()),
         .accusation = value.getAccusation().cStr(),
         .resolution_message_id = value.getResolutionMessageId(),
         .resolved_second = value.getResolvedSecond(),
         .resolving_system_id = value.getResolvingSystemId(),
      });
   }
   const auto cruise = source.getCruise();
   result.cruise = {
      .revision = cruise.getRevision(),
      .active = cruise.getActive(),
      .hunting_system_id = cruise.getHuntingSystemId(),
      .ends_second = cruise.getEndsSecond(),
      .crew_share_percent = cruise.getCrewSharePercent(),
      .ship_fund_percent = cruise.getShipFundPercent(),
      .prohibited_targets = cruise.getProhibitedTargets().cStr(),
   };
   for(const auto contact : source.getSystemContacts()) {
      result.system_contacts.push_back(decode_traffic_contact(contact));
   }
   for(const auto contact : source.getLocalContacts()) {
      result.local_contacts.push_back(decode_traffic_contact(contact));
   }
   if(source.getHasInterceptionWatch()) {
      const auto watch = source.getInterceptionWatch();
      result.interception_watch = InterceptionWatchStatus{
         .started_second = watch.getStartedSecond(),
         .target_contact_id = watch.getTargetContactId(),
         .target_catalog_id = watch.getTargetCatalogId(),
         .target_ship_name = watch.getTargetShipName().cStr(),
         .filter = static_cast<InterceptionWatchFilterKind>(watch.getFilter()),
         .locus = decode_flight_locus(watch.getLocus()),
         .purpose = static_cast<InterceptionPurpose>(watch.getPurpose()),
      };
   }
   return result;
}

}

CombatCareerSnapshot get_combat_career(TlsConnection& connection, const uint64_t epoch,
                                       const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetCombatCareer();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat_career(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

CombatCareerSnapshot accept_career_opportunity(TlsConnection& connection, const uint64_t epoch,
      const uint64_t opportunity_id, const uint64_t revision, const std::array<uint8_t, 16>& id,
      const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto value = request.initAcceptCareerOpportunity();
   value.setOpportunityId(opportunity_id);
   value.setExpectedRevision(revision);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat_career(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

InterceptionStart engage_traffic_contact(TlsConnection& connection, const uint64_t epoch,
                                         const uint64_t contact_id,
                                         const uint64_t revision,
                                         const InterceptionPurpose purpose,
                                         const std::array<uint8_t, 16>& id,
                                         const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto value = request.initEngageTrafficContact();
   value.setContactId(contact_id);
   value.setExpectedCareerRevision(revision);
   value.setPurpose(static_cast<rpc::InterceptionPurpose>(purpose));
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response = checked_response(reader.getRoot<rpc::Envelope>(), id);
   if(response.isCombat()) {
      return decode_combat(response);
   }
   if(response.isCombatCareer()) {
      return decode_combat_career(response);
   }
   if(response.isEncounterResult()) {
      const auto s = response.getEncounterResult();
      return EncounterResult{
         .encounter_id = s.getEncounterId(),
         .resolved = s.getResolved(),
         .terminal = s.getTerminal(),
         .outcome = s.getOutcome().cStr(),
         .turns = s.getTurns(),
         .cargo_lost_millitons = s.getCargoLostMillitons(),
         .fuel_lost_millitons = s.getFuelLostMillitons(),
         .damage_hits = s.getDamageHits(),
         .phase = decode_response_phase(response.getPhase()),
      };
   }
   throw std::runtime_error("expected combat, boarding resolution, or interception-watch response");
}

CombatCareerSnapshot set_interception_watch(TlsConnection& connection, const uint64_t epoch,
                                             const InterceptionWatchSelection selection,
                                             const uint32_t catalog_id,
                                             const InterceptionPurpose purpose,
                                             const uint64_t revision,
                                             const std::array<uint8_t, 16>& id,
                                             const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto value = request.initSetInterceptionWatch();
   value.setExpectedCareerRevision(revision);
   value.setPurpose(static_cast<rpc::InterceptionPurpose>(purpose));
   switch(selection) {
   case InterceptionWatchSelection::Cancel:
      value.setCancel();
      break;
   case InterceptionWatchSelection::AllCraft:
      value.setAllCraft();
      break;
   case InterceptionWatchSelection::CraftClass:
      value.setCatalogId(catalog_id);
      break;
   }
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat_career(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

CombatCareerSnapshot set_pirate_cruise(TlsConnection& connection, const uint64_t epoch,
                                       const PirateCruise& cruise,
                                       const std::array<uint8_t, 16>& id,
                                       const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto value = request.initSetPirateCruise();
   value.setExpectedRevision(cruise.revision);
   value.setActive(cruise.active);
   value.setHuntingSystemId(cruise.hunting_system_id);
   value.setEndsSecond(cruise.ends_second);
   value.setCrewSharePercent(cruise.crew_share_percent);
   value.setShipFundPercent(cruise.ship_fund_percent);
   value.setProhibitedTargets(cruise.prohibited_targets);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat_career(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

CombatCareerSnapshot settle_prize(TlsConnection& connection, const uint64_t epoch,
                                  const uint64_t prize_id, const uint64_t revision, const PrizeSettlementMethod method,
                                  const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto value=request.initSettlePrize();
   value.setPrizeId(prize_id);
   value.setExpectedCareerRevision(revision);
   value.setMethod(static_cast<rpc::PrizeSettlementMethod>(method));
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat_career(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

CombatCareerSnapshot settle_warrant(TlsConnection& connection, const uint64_t epoch,
                                    const uint64_t warrant_id,
                                    const uint64_t revision,
                                    const std::array<uint8_t, 16>& id,
                                    const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto value = request.initSettleWarrant();
   value.setWarrantId(warrant_id);
   value.setExpectedCareerRevision(revision);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat_career(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

CombatCareerSnapshot set_combat_career_mode(
   TlsConnection& connection,
   const uint64_t epoch,
   const CombatCareerMode mode,
   const uint64_t revision,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto value = request.initSetCombatCareerMode();
   value.setMode(static_cast<rpc::CombatCareerMode>(mode));
   value.setExpectedRevision(revision);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_combat_career(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FleetSnapshot recover_command(
   TlsConnection& connection,
   const uint64_t epoch,
   const std::string& successor_name,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initRecoverCommand().setSuccessorName(successor_name);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_fleet(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

FleetSnapshot declare_bankruptcy(
   TlsConnection& connection,
   const uint64_t epoch,
   const std::string& successor_name,
   const std::array<uint8_t, 16>& id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initDeclareBankruptcy().setSuccessorName(successor_name);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_fleet(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

ArrivalPacket open_arrival_packet(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setOpenArrivalPacket();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_arrival_packet(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

MessageManagement get_message_management(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   request.setGetMessageManagement();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_message_management(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

MessageManagement set_message_classification(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t message_id,
   const MessageClassification classification,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initSetMessageClassification();
   change.setMessageId(message_id);
   change.setClassification(encode_message_classification(classification));
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_message_management(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

MessageManagement set_message_filter(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const MessageClass message_class,
   const MessageImportance minimum_importance,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initSetMessageFilter();
   change.setClass(encode_message_class(message_class));
   change.setMinimumImportance(encode_message_importance(minimum_importance));
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_message_management(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

MessageManagement send_private_message(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const PrivateMessageRequest& value,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto private_message = request.initSendPrivateMessage();
   private_message.setRecipientKind(
      static_cast<rpc::PrivateRecipientKind>(value.recipient_kind));
   private_message.setDestinationSystemId(value.destination_system_id);
   auto recipient = private_message.initRecipient();
   recipient.setBbsId(value.recipient.bbs_id);
   recipient.setPlayerId(value.recipient.player_id);
   private_message.setEncryptionKeyId(value.encryption_key_id);
   private_message.setTtlWeeks(value.ttl_weeks);
   private_message.setSubject(value.subject);
   private_message.setBody(value.body);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_message_management(
             checked_response(reader.getRoot<rpc::Envelope>(), command_id));
}

SystemRadioSnapshot get_system_radio(
   TlsConnection& connection, const uint64_t epoch,
   const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.setGetSystemRadio();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_system_radio(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

SystemRadioSnapshot transmit_system_radio(
   TlsConnection& connection, const uint64_t epoch, const std::string& body,
   const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initTransmitSystemRadio().setBody(body);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_system_radio(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

RadioContent peek_radio_reception(
   TlsConnection& connection, const uint64_t epoch, const uint64_t reception_id,
   const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initPeekRadioReception().setReceptionId(reception_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response = checked_response(reader.getRoot<rpc::Envelope>(), id);
   if(!response.isRadioContent()) {
      throw std::runtime_error("expected RadioContent");
   }
   const auto content = response.getRadioContent();
   return {
      .reception_id = content.getReceptionId(),
      .transmission_id = content.getTransmissionId(),
      .body = content.getBody().cStr(),
      .committed_sequence = response.getCommittedSequence(),
      .revision = response.getRevision(),
      .phase = decode_response_phase(response.getPhase()),
   };
}

SystemRadioSnapshot acknowledge_radio_reception(
   TlsConnection& connection, const uint64_t epoch, const uint64_t reception_id,
   const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   request.initAcknowledgeRadioReception().setReceptionId(reception_id);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_system_radio(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

SystemRadioSnapshot set_radio_mute(
   TlsConnection& connection, const uint64_t epoch, const PlayerIdentity& sender,
   const bool muted, const std::array<uint8_t, 16>& id, const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, epoch, request_id, id, request);
   auto change = request.initSetRadioMute();
   auto target = change.initSender();
   target.setBbsId(sender.bbs_id);
   target.setPlayerId(sender.player_id);
   change.setMuted(muted);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   return decode_system_radio(checked_response(reader.getRoot<rpc::Envelope>(), id));
}

SystemMappingStatus set_system_mapping_disclosure(
   TlsConnection& connection,
   const uint64_t session_epoch,
   const uint64_t system_id,
   const SystemMappingChoice choice,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id)
{
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<rpc::Envelope>();
   auto request = envelope.initRequest();
   initialize_request(envelope, session_epoch, request_id, command_id, request);
   auto change = request.initSetSystemMappingDisclosure();
   change.setSystemId(system_id);
   change.setChoice(encode_system_mapping_choice(choice));
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   const auto words = receive_response(connection, session_epoch, request_id);
   capnp::FlatArrayMessageReader reader(words);
   const auto response =
      checked_response(reader.getRoot<rpc::Envelope>(), command_id);
   if(!response.isSystemMappingStatus()) {
      throw std::runtime_error("expected SystemMappingStatus");
   }
   return decode_system_mapping_status(response.getSystemMappingStatus(), response);
}

std::optional<PlayerEvent> poll_event(TlsConnection& connection,
                                      const uint64_t session_epoch)
{
   auto frame = connection.try_deferred_event_frame();
   if(!frame.has_value()) {
      frame = connection.try_receive_frame();
   }
   if(!frame.has_value()) {
      return std::nullopt;
   }
   const auto word_count =
      (frame->size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto words = kj::heapArray<capnp::word>(word_count);
   std::memset(words.begin(), 0, words.asBytes().size());
   std::memcpy(words.asBytes().begin(), frame->data(), frame->size());
   capnp::FlatArrayMessageReader reader(words);
   const auto envelope = reader.getRoot<rpc::Envelope>();
   if(envelope.getProtocolVersion() != PROTOCOL_VERSION ||
         envelope.getSessionEpoch() != session_epoch || !envelope.isEvent()) {
      throw std::runtime_error("invalid unsolicited CT-RPC event envelope");
   }
   const auto event = envelope.getEvent();
   PlayerEvent result{
      .kind = PlayerEventKind::ServerStopping,
      .committed_sequence = event.getCommittedSequence(),
      .travel_status = std::nullopt,
      .traffic_snapshot = std::nullopt,
      .traffic_contact = std::nullopt,
      .checkpoint = std::nullopt,
      .encounter = std::nullopt,
      .observed_second = 0,
      .system_id = 0,
   };
   if(event.isSessionReplaced()) {
      result.kind = PlayerEventKind::SessionReplaced;
   } else if(event.isServerStopping()) {
      result.kind = PlayerEventKind::ServerStopping;
   } else if(event.isPhaseChanged()) {
      result.kind = PlayerEventKind::PhaseChanged;
      const auto changed = event.getPhaseChanged();
      const auto source = changed.getTravelStatus();
      result.travel_status = TravelStatus{
         .ship_id = source.getShipId(),
         .ship_name = source.getShipName().cStr(),
         .current_system_id = source.getCurrentSystemId(),
         .current_system_name = source.getCurrentSystemName().cStr(),
         .destination_system_id = source.getDestinationSystemId(),
         .destination_system_name = source.getDestinationSystemName().cStr(),
         .stage = decode_travel_stage(source.getStage()),
         .current_game_second = source.getCurrentGameSecond(),
         .due_second = source.getDueSecond(),
         .current_fuel_millitons = source.getCurrentFuelMillitons(),
         .jump_fuel_millitons = source.getJumpFuelMillitons(),
         .clock_rate_game_seconds = source.getClockRateGameSeconds(),
         .clock_rate_real_seconds = source.getClockRateRealSeconds(),
         .plan_id = source.getPlanId(),
         .plan_revision = source.getPlanRevision(),
         .leg_index = source.getLegIndex(),
         .origin = decode_flight_locus(source.getOrigin()),
         .destination = decode_flight_locus(source.getDestination()),
         .committed_sequence = event.getCommittedSequence(),
         .revision = changed.getRevision(),
         .phase = decode_response_phase(changed.getPhase()),
      };
   } else if(event.isTrafficSnapshot()) {
      result.kind = PlayerEventKind::TrafficSnapshot;
      const auto source = event.getTrafficSnapshot();
      TrafficSnapshot snapshot{
         .system_id = source.getSystemId(),
         .system_name = source.getSystemName().cStr(),
         .observed_second = source.getObservedSecond(),
         .contacts = {},
      };
      for(const auto contact : source.getContacts()) {
         snapshot.contacts.push_back(decode_traffic_contact(contact));
      }
      result.traffic_snapshot = std::move(snapshot);
   } else if(event.isTrafficMovement()) {
      result.kind = PlayerEventKind::TrafficMovement;
      const auto movement = event.getTrafficMovement();
      result.system_id = movement.getSystemId();
      result.observed_second = movement.getObservedSecond();
      result.traffic_contact = decode_traffic_contact(movement.getContact());
   } else if(event.isCheckpointReady()) {
      result.kind = PlayerEventKind::CheckpointReady;
      auto source = event.getCheckpointReady();
      result.checkpoint = CheckpointSnapshot{
         .checkpoint_id = source.getCheckpointId(),
         .plan_id = source.getPlanId(),
         .plan_revision = source.getPlanRevision(),
         .step_index = source.getStepIndex(),
         .locus = decode_flight_locus(source.getLocus()),
         .kind = static_cast<CheckpointKind>(source.getKind()),
         .ready_second = source.getReadySecond(),
         .acknowledged = source.getAcknowledged(),
         .phase = PlayerPhase::Interplanetary,
      };
   } else if(event.isEncounterReady()) {
      result.kind = PlayerEventKind::EncounterReady;
      auto s = event.getEncounterReady();
      auto c = s.getContact();
      result.encounter = EncounterSnapshot{
         .encounter_id = s.getEncounterId(),
         .revision = s.getRevision(),
         .kind = static_cast<EncounterKind>(s.getKind()),
         .state = static_cast<EncounterState>(s.getState()),
         .started_second = s.getStartedSecond(),
         .next_turn_second = s.getNextTurnSecond(),
         .turn = s.getTurn(),
         .contact = {
            .contact_id = c.getContactId(),
            .ship_name = c.getShipName().cStr(),
            .class_name = c.getClassName().cStr(),
            .transponder = c.getTransponder().cStr(),
            .role = c.getRole().cStr(),
            .range = c.getRange().cStr(),
            .confidence_percent = c.getConfidencePercent(),
         },
         .summary = s.getSummary().cStr(),
         .phase = PlayerPhase::Encounter,
      };
   } else if(event.isRadioUnread()) {
      result.kind = PlayerEventKind::RadioUnread;
      const auto unread = event.getRadioUnread();
      result.ship_id = unread.getShipId();
      result.unread_count = unread.getUnreadCount();
   } else {
      throw std::runtime_error("unknown unsolicited CT-RPC event");
   }
   return result;
}

}  // namespace ct
