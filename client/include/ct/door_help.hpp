#pragma once

#include <cstddef>
#include <span>
#include <string_view>

namespace ct {

enum class DoorHelpTopic : size_t {
   General,
   PlayerRegistration,
   Characteristics,
   Careers,
   StartingShip,
   Crew,
   CommandConsole,
   Ship,
   Tasks,
   Messages,
   KnownUniverse,
   Operations,
   Docked,
   Cargo,
   Fuel,
   FlightPlan,
   Finance,
   Shipyard,
   Personnel,
   Arrival,
   Voyage,
   Encounter,
   Combat,
   CombatOrders,
   Count,
};

struct DoorHelp {
   std::string_view title;
   std::string_view body;
};

const DoorHelp& door_help(DoorHelpTopic topic);
std::span<const DoorHelp> all_door_help();

}  // namespace ct
