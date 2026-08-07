#include "ct/crew_presentation.hpp"

#include <array>
#include <stdexcept>
#include <string>
#include <string_view>

namespace ct {
namespace {

struct RoleWords {
   std::string_view role;
   std::string_view role_name;
   std::string_view single_appointment;
   std::string_view group_appointment;
   std::string_view singular_position;
   std::string_view plural_positions;
};

constexpr std::array ROLE_WORDS{
   RoleWords{"command", "Command staff", "Executive officer",
             "Executive officer", "command-staff position",
             "command-staff positions"},
   RoleWords{"pilot", "Pilot", "Pilot", "Chief pilot", "pilot", "pilots"},
   RoleWords{"navigator", "Navigator", "Navigator", "Chief navigator",
             "navigator", "navigators"},
   RoleWords{"engineer", "Engineer", "Ship's engineer", "Chief engineer",
             "engineer", "engineers"},
   RoleWords{"sensors-operator", "Sensors operator", "Sensors operator",
             "Chief sensors operator", "sensors-operator position",
             "sensors-operator positions"},
   RoleWords{"screen-operator", "Screen operator", "Screen operator",
             "Chief screen operator", "screen-operator position",
             "screen-operator positions"},
   RoleWords{"turret-gunner", "Turret gunner", "Turret gunner",
             "Gunnery officer", "turret-gunner position",
             "turret-gunner positions"},
   RoleWords{"bay-gunner", "Bay gunner", "Bay gunner", "Senior bay gunner",
             "bay-gunner position", "bay-gunner positions"},
   RoleWords{"gunner", "Gunner", "Gunner", "Gunnery officer", "gunner",
             "gunners"},
   RoleWords{"medic", "Medical", "Ship's medic", "Chief medical officer",
             "medical position", "medical positions"},
   RoleWords{"marine", "Marine", "Marine",
             "Marine detachment leader", "marine", "marines"},
   RoleWords{"flight-crew", "Flight crew", "Flight-crew member",
             "Flight-deck chief", "flight-crew position",
             "flight-crew positions"},
   RoleWords{"steward", "Steward", "Steward", "Chief steward", "steward",
             "stewards"},
   RoleWords{"other", "General crew", "Crew member", "Crew chief",
             "general-crew position", "general-crew positions"},
};

std::string title_role(const std::string_view role) {
   std::string result;
   result.reserve(role.size());
   bool capitalize = true;
   for(const char value : role) {
      if(value == '-') {
         result.push_back(' ');
         capitalize = true;
      } else if(capitalize && value >= 'a' && value <= 'z') {
         result.push_back(static_cast<char>(value - 'a' + 'A'));
         capitalize = false;
      } else {
         result.push_back(value);
         capitalize = false;
      }
   }
   return result.empty() ? "Crew" : result;
}

}  // namespace

CrewNamingPresentation describe_crew_naming(
   const std::string_view role,
   const uint16_t represented_positions) {
   if(represented_positions == 0) {
      throw std::invalid_argument(
         "crew naming requires at least one represented position");
   }
   const RoleWords* words = nullptr;
   for(const auto& candidate : ROLE_WORDS) {
      if(candidate.role == role) {
         words = &candidate;
         break;
      }
   }

   const auto fallback = title_role(role);
   const auto role_name =
      words == nullptr ? fallback : std::string(words->role_name);
   const auto appointment =
      words == nullptr
         ? (represented_positions == 1 ? fallback : "Lead " + fallback)
         : std::string(represented_positions == 1
                          ? words->single_appointment
                          : words->group_appointment);
   const auto position =
      words == nullptr
         ? (represented_positions == 1
               ? role_name + " position"
               : role_name + " positions")
         : std::string(represented_positions == 1
                          ? words->singular_position
                          : words->plural_positions);

   CrewNamingPresentation result{
      .role_name = role_name,
      .appointment = appointment,
      .assignment =
         std::to_string(represented_positions) + " " + position,
      .explanation = {},
      .prompt = appointment + " name",
   };
   if(represented_positions == 1) {
      result.explanation =
         "Name the individual assigned to this position, or accept the "
         "default role callsign.";
   } else {
      const auto supporting = represented_positions - 1;
      result.explanation =
         "Name the " + appointment + ". The other " +
         std::to_string(supporting) +
         (supporting == 1 ? " position is" : " positions are") +
         " supporting personnel and do not need individual names during "
         "initial creation.";
   }
   return result;
}

}  // namespace ct
