#pragma once

#include "ct/player_identity_registry.hpp"

#include <cstddef>
#include <span>
#include <string_view>

namespace ct {

enum class DoorHelpTopic : size_t {
   General,
   Orientation,
   Controls,
   FirstSession,
   GuidedFirstWatch,
   HelpBrowser,
   PlayerPreferences,
   PlayerRegistration,
   Characteristics,
   SkillsTraining,
   Careers,
   StartingShip,
   StartingFit,
   StartingCrew,
   RegistrationConfirmation,
   Crew,
   CrewMember,
   CommandConsole,
   Ship,
   ShipSubsystems,
   Fleet,
   Tasks,
   TaskOffer,
   Messages,
   MessageDetail,
   Radio,
   KnownUniverse,
   SystemDossier,
   CoursePlotter,
   Operations,
   LocalContacts,
   Warrants,
   Pickets,
   Docked,
   Cargo,
   Fuel,
   FlightPlan,
   FlightPlanPreview,
   Finance,
   Shipyard,
   Personnel,
   Arrival,
   ArrivalPacket,
   OperationalDamageReport,
   Voyage,
   Encounter,
   Combat,
   CombatOrders,
   CombatRecovery,
   ConceptPersistentWorld,
   ConceptTaskChecks,
   ConceptShipsCrew,
   ConceptTechLevels,
   ConceptTravelFuel,
   ConceptTradeContracts,
   ConceptMailInformation,
   ConceptSensorsTraffic,
   ConceptLawAuthority,
   ConceptCombatBoarding,
   GlossaryAF,
   GlossaryGL,
   GlossaryMR,
   GlossarySZ,
   Count,
};

enum class DoorHelpCategory : size_t {
   GettingStarted,
   MenusScreens,
   Concepts,
   Glossary,
   Count,
};

struct DoorHelp {
   std::string_view title;
   std::string_view group;
   DoorHelpCategory category;
   std::string_view beginner_body;
   std::string_view expert_body;
};

const DoorHelp& door_help(DoorHelpTopic topic);
std::string_view door_help_body(const DoorHelp& help, HelpLevel level);
std::span<const DoorHelp> all_door_help();
std::string_view door_help_category_name(DoorHelpCategory category);

}  // namespace ct
