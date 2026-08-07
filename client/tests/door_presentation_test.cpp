#include "ct/cargo_quantity.hpp"
#include "ct/crew_presentation.hpp"
#include "ct/door_presentation.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>

namespace {

void check(const bool condition) {
   if(!condition) {
      throw std::runtime_error("door presentation test failed");
   }
}

std::string render(const ct::DoorProfile profile,
                   const size_t columns,
                   const size_t rows,
                   const std::string_view text,
                   const ct::DoorTextRole role = ct::DoorTextRole::Normal) {
   std::string result;
   ct::DoorPresentation presentation(
      profile,
      columns,
      rows,
      [&result](const std::string_view bytes) { result.append(bytes); });
   presentation.clear();
   presentation.write(text, role);
   return result;
}

size_t maximum_visible_width(const std::string_view output) {
   size_t maximum = 0;
   size_t current = 0;
   unsigned escape_state = 0;
   for(const unsigned char byte : output) {
      if(escape_state == 1) {
         escape_state = byte == '[' ? 2 : 0;
      } else if(escape_state == 2) {
         if(byte >= 0x40 && byte <= 0x7e) {
            escape_state = 0;
         }
      } else if(byte == 0x1b) {
         escape_state = 1;
      } else if(byte == '\r' || byte == '\n' || byte == '\f') {
         maximum = std::max(maximum, current);
         current = 0;
      } else {
         ++current;
      }
   }
   return std::max(maximum, current);
}

void check_profile_and_width(const ct::DoorProfile profile,
                             const size_t columns,
                             const size_t rows) {
   const auto output = render(
      profile,
      columns,
      rows,
      "A deliberately long market, crew, contract, and ship record with "
      "12345678901234567890 credits at coordinates "
      "-123456789.125/+987654321.875/-444444444.500.");
   check(maximum_visible_width(output) <= columns);
   if(profile == ct::DoorProfile::Iso646) {
      check(!output.empty() && output.front() == '\f');
      check(output.find('\x1b') == std::string::npos);
   } else {
      check(output.starts_with("\x1b[0m\x1b[2J\x1b[H"));
      check(output.ends_with("\x1b[0m"));
   }
}

}  // namespace

int main() {
   check(ct::parse_tonnage_millitons("1") == 1000);
   check(ct::parse_tonnage_millitons("1.25") == 1250);
   check(ct::parse_tonnage_millitons(".001") == 1);
   check(!ct::parse_tonnage_millitons("1.0001"));
   check(!ct::parse_tonnage_millitons("one"));
   check(ct::format_tonnage(1000) == "1");
   check(ct::format_tonnage(1250) == "1.25");
   check(ct::format_tonnage(1) == "0.001");
   check(ct::cargo_purchase_cost(1100, 1) == 2);
   check(ct::cargo_purchase_cost(25'000, 1001) == 25'025);
   check(ct::maximum_affordable_cargo(25'024, 25'000, 2000) == 1000);
   check(ct::maximum_affordable_cargo(25'025, 25'000, 2000) == 1001);

   const auto marines = ct::describe_crew_naming("marine", 4);
   check(marines.role_name == "Marine");
   check(marines.appointment == "Marine detachment leader");
   check(marines.assignment == "4 marines");
   check(marines.explanation.find("other 3 positions") != std::string::npos);
   check(marines.prompt == "Marine detachment leader name");

   const auto engineer = ct::describe_crew_naming("engineer", 1);
   check(engineer.appointment == "Ship's engineer");
   check(engineer.assignment == "1 engineer");

   const auto engineering = ct::describe_crew_naming("engineer", 4);
   check(engineering.appointment == "Chief engineer");
   check(engineering.assignment == "4 engineers");

   const auto specialist = ct::describe_crew_naming("survey-specialist", 2);
   check(specialist.role_name == "Survey Specialist");
   check(specialist.appointment == "Lead Survey Specialist");
   check(specialist.assignment == "2 Survey Specialist positions");

   bool rejected_zero = false;
   try {
      static_cast<void>(ct::describe_crew_naming("marine", 0));
   } catch(const std::invalid_argument&) {
      rejected_zero = true;
   }
   check(rejected_zero);

   constexpr std::array profiles{
      ct::DoorProfile::Iso646,
      ct::DoorProfile::Iso646Color,
      ct::DoorProfile::Cp437Color,
   };
   for(const auto profile : profiles) {
      check_profile_and_width(profile, 40, 24);
      check_profile_and_width(profile, 80, 24);
   }

   const auto plain = render(
      ct::DoorProfile::Iso646,
      40,
      24,
      "Name [x] @ port: caf\xc3\xa9 \x1b[31mFORGED");
   check(plain.find('\x1b') == std::string::npos);
   check(plain.find("Name (x) (at) port: caf? ?(31mFORGED") !=
         std::string::npos);

   const auto cp437 = render(
      ct::DoorProfile::Cp437Color,
      80,
      24,
      "caf\xc3\xa9 \xe2\x94\x80");
   check(cp437.find(static_cast<char>(0x82)) != std::string::npos);
   check(cp437.find(static_cast<char>(0xc4)) != std::string::npos);

   const auto colored = render(
      ct::DoorProfile::Iso646Color,
      40,
      24,
      "Alert",
      ct::DoorTextRole::Error);
   check(colored.find("\x1b[1;31mAlert\x1b[0m") != std::string::npos);
   check(maximum_visible_width(colored) == 5);

   const std::array semantic_roles{
      std::pair{ct::DoorTextRole::Label, std::string_view{"\x1b[36m"}},
      std::pair{ct::DoorTextRole::Value, std::string_view{"\x1b[1;37m"}},
      std::pair{ct::DoorTextRole::Number, std::string_view{"\x1b[1;33m"}},
      std::pair{ct::DoorTextRole::Identifier, std::string_view{"\x1b[1;35m"}},
      std::pair{ct::DoorTextRole::Information, std::string_view{"\x1b[32m"}},
      std::pair{ct::DoorTextRole::Success, std::string_view{"\x1b[1;32m"}},
      std::pair{ct::DoorTextRole::Warning, std::string_view{"\x1b[1;31m"}},
   };
   for(const auto& [role, sequence] : semantic_roles) {
      const auto sample =
         render(ct::DoorProfile::Iso646Color, 40, 24, "sample", role);
      check(sample.find(std::string(sequence) + "sample\x1b[0m") !=
            std::string::npos);
      const auto uncolored =
         render(ct::DoorProfile::Iso646, 40, 24, "sample", role);
      check(uncolored == "\fsample");
   }

   std::string page;
   ct::DoorPresentation presentation(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&page](const std::string_view bytes) { page.append(bytes); });
   check(presentation.page_content_rows(5) == 19);
   presentation.write("first\r\n\r\nsecond");
   check(page == "first\r\n\r\nsecond");

   std::string paged;
   unsigned pauses = 0;
   ct::DoorPresentation pager(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&paged](const std::string_view bytes) { paged.append(bytes); });
   pager.configure_paging(1, [&pager, &pauses] {
      ++pauses;
      pager.write("[Enter/Space] Continue", ct::DoorTextRole::Prompt);
      pager.erase_prompt(22);
   });
   pager.resume_paging();
   pager.clear();
   for(unsigned line = 0; line < 24; ++line) {
      pager.write("record\n\r", ct::DoorTextRole::Value);
   }
   check(pauses == 1);
   check(std::count(paged.begin(), paged.end(), '\f') == 1);
   check(
      paged.find(
         "(Enter/Space) Continue\r                      \rrecord") !=
      std::string::npos);
   pager.suspend_paging();
   for(unsigned line = 0; line < 40; ++line) {
      pager.write("unpaged\n\r");
   }
   check(pauses == 1);

   std::string wrapped_page;
   unsigned wrapped_pauses = 0;
   ct::DoorPresentation wrapped_pager(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&wrapped_page](const std::string_view bytes) {
         wrapped_page.append(bytes);
      });
   wrapped_pager.configure_paging(1, [&wrapped_pauses] { ++wrapped_pauses; });
   wrapped_pager.resume_paging();
   wrapped_pager.clear();
   wrapped_pager.write(std::string(40 * 24, 'x'));
   check(wrapped_pauses == 1);
   check(std::count(wrapped_page.begin(), wrapped_page.end(), '\f') == 1);

   check(ct::parse_door_profile("plain") == ct::DoorProfile::Iso646);
   check(ct::parse_door_profile("color") == ct::DoorProfile::Iso646Color);
   check(ct::parse_door_profile("cp437") == ct::DoorProfile::Cp437Color);
   check(ct::door_single_line_field("ship\r\n\x1b[31m") ==
         "ship  ?[31m");
}
