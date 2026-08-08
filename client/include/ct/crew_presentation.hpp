#pragma once

#include <cstdint>
#include <string>
#include <string_view>

#include "ct/protocol.hpp"

namespace ct {

struct CrewNamingPresentation {
   std::string role_name;
   std::string appointment;
   std::string assignment;
   std::string explanation;
   std::string prompt;
};

CrewNamingPresentation describe_crew_naming(
   CrewRoleKind role_kind,
   std::string_view role,
   uint16_t represented_positions);

}  // namespace ct
