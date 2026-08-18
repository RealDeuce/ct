#include "ct/cargo_quantity.hpp"
#include "ct/crew_presentation.hpp"
#include "ct/door_help.hpp"
#include "ct/door_presentation.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <limits>
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
   check(maximum_visible_width(output) < columns);
   if(!ct::door_profile_uses_ansi(profile)) {
      check(!output.empty() && output.front() == '\f');
      check(output.find('\x1b') == std::string::npos);
   } else {
      check(output.starts_with("\x1b[0m\x1b[2J\x1b[H"));
      check(output.ends_with("\x1b[0m"));
   }
}

}  // namespace

int main() {
   const auto price_plot =
      ct::price_box_plot(900, 1'000, 1'100, 1'200, 1'300, 1'150);
   check(price_plot.size() == 21);
   check(price_plot.front() == 'o');
   check(price_plot.back() == 'o');
   check(price_plot.find('(') != std::string::npos);
   check(price_plot.find(':') != std::string::npos);
   check(price_plot.find(')') != std::string::npos);
   check(price_plot.find('*') != std::string::npos);
   const auto flat_price_plot =
      ct::price_box_plot(1'000, 1'000, 1'000, 1'000, 1'000, 1'000);
   check(flat_price_plot.size() == 21);
   check(flat_price_plot[10] == 'X');
   const auto full_range_price_plot = ct::price_box_plot(
      0,
      0,
      0,
      std::numeric_limits<uint64_t>::max(),
      std::numeric_limits<uint64_t>::max(),
      std::numeric_limits<uint64_t>::max() / 2);
   check(full_range_price_plot.find('*') == 9);

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

   const auto marines = ct::describe_crew_naming(ct::CrewRoleKind::Marine, "marine", 4);
   check(marines.role_name == "Marine");
   check(marines.appointment == "Marine detachment leader");
   check(marines.assignment == "4 marines");
   check(marines.explanation.find("other 3 positions") != std::string::npos);
   check(marines.prompt == "Marine detachment leader name");

   const auto engineer = ct::describe_crew_naming(ct::CrewRoleKind::Engineer, "engineer", 1);
   check(engineer.appointment == "Ship's engineer");
   check(engineer.assignment == "1 engineer");

   const auto engineering = ct::describe_crew_naming(ct::CrewRoleKind::Engineer, "engineer", 4);
   check(engineering.appointment == "Chief engineer");
   check(engineering.assignment == "4 engineers");

   const auto specialist = ct::describe_crew_naming(ct::CrewRoleKind::Other, "survey-specialist", 2);
   check(specialist.role_name == "General crew");
   check(specialist.appointment == "Crew chief");
   check(specialist.assignment == "2 general-crew positions");

   bool rejected_zero = false;
   try {
      static_cast<void>(ct::describe_crew_naming(ct::CrewRoleKind::Marine, "marine", 0));
   } catch(const std::invalid_argument&) {
      rejected_zero = true;
   }
   check(rejected_zero);

   constexpr std::array profiles{
      ct::DoorProfile::Iso646,
      ct::DoorProfile::Iso646Color,
      ct::DoorProfile::Cp437,
      ct::DoorProfile::Cp437Color,
   };
   for(const auto profile : profiles) {
      check_profile_and_width(profile, 40, 24);
      check_profile_and_width(profile, 80, 24);
   }

   const std::string refit_quotation =
      "Refit Quotation - Firefly II\r\n"
      "Operating account charge: Cr149,191\r\n"
      "Yard time: 6 weeks\r\n\r\n"
      "The yard will repair all non-destroyed damage, remove temporary "
      "battlefield patches, and correct minor faults found during the overhaul.\r\n"
      "Destroyed installations are not replaced. Installation age and use are "
      "retained. Routine upkeep continues while the ship is in the yard.\r\n"
      "Destroyed installations not replaced:\r\n"
      "  Maneuver drive\r\n"
      "[Q/Enter] Cancel  [R] Authorize refit  [?] Help";
   for(const auto columns : {size_t{40}, size_t{80}}) {
      const auto quotation_output = render(
         ct::DoorProfile::Iso646,
         columns,
         24,
         refit_quotation,
         ct::DoorTextRole::Information);
      check(quotation_output.find("Operating account charge") != std::string::npos);
      check(quotation_output.find("Authorize refit") != std::string::npos);
      check(maximum_visible_width(quotation_output) < columns);
   }

   const std::string active_operation_status =
      "Active operation: Refit\r\n"
      "Completes: Day 286, 21:16:53\r\n"
      "Time remaining: 3 d 04:17:22\r\n"
      "Refit charge: Cr149,191\r\n"
      "Yard time: 42 d 00:00:00\r\n";
   for(const auto columns : {size_t{40}, size_t{80}}) {
      const auto status_output = render(
         ct::DoorProfile::Iso646,
         columns,
         24,
         active_operation_status,
         ct::DoorTextRole::Information);
      for(const auto expected : {
            "Active operation: Refit",
            "Completes: Day 286, 21:16:53",
            "Time remaining: 3 d 04:17:22",
         }) {
         check(status_output.find(expected) != std::string::npos);
      }
      check(maximum_visible_width(status_output) < columns);
   }

   const std::string fuel_receipt =
      "Fueling complete.\r\n"
      "Loaded: 66.0 t of unrefined fuel\r\n"
      "Tanks: 134.0/200.0 t\r\n"
      "Unrefined aboard: 66.0 t\r\n"
      "Charge: Cr6,600\r\n"
      "Paid from:\r\n"
      "  Restricted operating: Cr4,000\r\n"
      "  Liquid credits: Cr2,600\r\n"
      "Balance after purchase:\r\n"
      "  Restricted operating: Cr145,191\r\n"
      "  Liquid credits: Cr97,400\r\n";
   for(const auto columns : {size_t{40}, size_t{80}}) {
      const auto receipt_output = render(
         ct::DoorProfile::Iso646,
         columns,
         24,
         fuel_receipt,
         ct::DoorTextRole::Information);
      for(const auto expected : {
            "Loaded:",
            "Tanks:",
            "Charge:",
            "Paid from:",
            "Balance after purchase:",
         }) {
         check(receipt_output.find(expected) != std::string::npos);
      }
      check(maximum_visible_width(receipt_output) < columns);
   }

   const std::string provision_receipt =
      "Provisioning complete.\r\n"
      "Monthly packages: 1\r\n"
      "Person-days loaded: 90\r\n"
      "Life-support stores: 570/1080 person-days\r\n"
      "Charge: Cr6,000\r\n"
      "Paid from:\r\n"
      "  Restricted operating: Cr4,000\r\n"
      "  Liquid credits: Cr2,000\r\n"
      "Balance after purchase:\r\n"
      "  Restricted operating: Cr145,191\r\n"
      "  Liquid credits: Cr98,000\r\n";
   for(const auto columns : {size_t{40}, size_t{80}}) {
      const auto receipt_output = render(
         ct::DoorProfile::Iso646,
         columns,
         24,
         provision_receipt,
         ct::DoorTextRole::Information);
      for(const auto expected : {
            "Monthly packages:",
            "Person-days loaded:",
            "Life-support stores:",
            "Charge:",
            "Paid from:",
            "Balance after purchase:",
         }) {
         check(receipt_output.find(expected) != std::string::npos);
      }
      check(maximum_visible_width(receipt_output) < columns);
   }

   const auto help_topics = ct::all_door_help();
   check(help_topics.size() ==
         static_cast<size_t>(ct::DoorHelpTopic::Count));
   const auto& ship_help = ct::door_help(ct::DoorHelpTopic::Ship);
   check(ship_help.beginner_body.find("charged automatically") !=
         std::string_view::npos);
   check(ship_help.beginner_body.find("no monthly yard order is needed") !=
         std::string_view::npos);
   check(ship_help.beginner_body.find("restricted operating credit first") !=
         std::string_view::npos);
   check(ship_help.beginner_body.find("can damage a subsystem") !=
         std::string_view::npos);
   check(ship_help.beginner_body.find("quotation shows its operating-account charge") !=
         std::string_view::npos);
   check(ship_help.beginner_body.find("does not replace destroyed installations") !=
         std::string_view::npos);
   check(ship_help.expert_body.find("automatic every 30 game days") !=
         std::string_view::npos);
   check(ship_help.expert_body.find("requires authorization after quotation") !=
         std::string_view::npos);
   const auto& shipyard_help = ct::door_help(ct::DoorHelpTopic::Shipyard);
   check(shipyard_help.beginner_body.find("Review a refit quotation") !=
         std::string_view::npos);
   check(shipyard_help.expert_body.find("does not replace destroyed installations") !=
         std::string_view::npos);
   const auto& fuel_help = ct::door_help(ct::DoorHelpTopic::Fuel);
   check(fuel_help.beginner_body.find("fueling receipt") != std::string_view::npos);
   for(const auto& help : help_topics) {
      check(!help.title.empty());
      check(!help.group.empty());
      check(!help.beginner_body.empty());
      check(!help.expert_body.empty());
      for(const auto body : {help.beginner_body, help.expert_body}) {
         for(const auto columns : {size_t{40}, size_t{80}}) {
            const auto help_output = render(
               ct::DoorProfile::Iso646,
               columns,
               24,
               std::string("Help - ") + std::string(help.title) + "\r\n\r\n" +
                  std::string(body),
               ct::DoorTextRole::Information);
            check(help_output.find("Help - ") != std::string::npos);
            check(maximum_visible_width(help_output) < columns);
         }
      }
   }
   bool rejected_help_topic = false;
   try {
      static_cast<void>(ct::door_help(ct::DoorHelpTopic::Count));
   } catch(const std::out_of_range&) {
      rejected_help_topic = true;
   }
   check(rejected_help_topic);

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

   std::string hanging;
   ct::DoorPresentation hanging_presentation(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&hanging](const std::string_view bytes) { hanging.append(bytes); });
   hanging_presentation.write("Authority: ", ct::DoorTextRole::Label);
   hanging_presentation.write_hanging(
      "Sponsor command under a limited local commission; prize rights "
      "require adjudication",
      11,
      ct::DoorTextRole::Information);
   check(
      hanging ==
      "Authority: Sponsor command under a\r\n"
      "           limited local commission;\r\n"
      "           prize rights require\r\n"
      "           adjudication");
   check(maximum_visible_width(hanging) < 40);

   std::string exact_margin;
   ct::DoorPresentation margin_presentation(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&exact_margin](const std::string_view bytes) {
         exact_margin.append(bytes);
      });
   margin_presentation.write(std::string(40, 'x') + "\r\nnext");
   check(maximum_visible_width(exact_margin) == 39);
   check(exact_margin == std::string(39, 'x') + "\r\nx\r\nnext");

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
      return ct::DoorPresentation::PagePauseAction::Continue;
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

   pager.reset_paging();
   pager.resume_paging();
   for(unsigned line = 0; line < 22; ++line) {
      pager.write("after input\n\r");
   }
   check(pauses == 1);
   pager.write("page boundary\n\r", ct::DoorTextRole::Prompt);
   check(pauses == 2);

   pager.set_paging_enabled(false);
   pager.reset_paging();
   pager.resume_paging();
   for(unsigned line = 0; line < 46; ++line) {
      pager.write("persistent continuous output\n\r");
   }
   check(pauses == 2);
   pager.set_paging_enabled(true);
   pager.reset_paging();
   pager.resume_paging();
   for(unsigned line = 0; line < 23; ++line) {
      pager.write("paging restored\n\r");
   }
   check(pauses == 3);

   std::string continuous_page;
   unsigned continuous_pauses = 0;
   ct::DoorPresentation continuous_pager(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&continuous_page](const std::string_view bytes) {
         continuous_page.append(bytes);
      });
   continuous_pager.configure_paging(
      1,
      [&continuous_pauses] {
         ++continuous_pauses;
         return ct::DoorPresentation::PagePauseAction::Continuous;
      });
   continuous_pager.resume_paging();
   for(unsigned line = 0; line < 80; ++line) {
      continuous_pager.write("continuous output\n\r");
      continuous_pager.resume_paging();
   }
   check(continuous_pauses == 1);
   continuous_pager.reset_paging();
   continuous_pager.resume_paging();
   for(unsigned line = 0; line < 23; ++line) {
      continuous_pager.write("after keyboard input\n\r");
   }
   check(continuous_pauses == 2);

   std::string aborted_page;
   unsigned abort_pauses = 0;
   ct::DoorPresentation aborting_pager(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&aborted_page](const std::string_view bytes) {
         aborted_page.append(bytes);
      });
   aborting_pager.configure_paging(1, [&abort_pauses] {
      ++abort_pauses;
      return ct::DoorPresentation::PagePauseAction::Abort;
   });
   aborting_pager.resume_paging();
   const auto completed = aborting_pager.write(std::string(40 * 30, 'a'));
   check(!completed);
   check(abort_pauses == 1);
   check(aborted_page.size() < 40 * 30);

   std::string skipped_page;
   unsigned skip_pauses = 0;
   ct::DoorPresentation skipping_pager(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&skipped_page](const std::string_view bytes) {
         skipped_page.append(bytes);
      });
   skipping_pager.configure_paging(1, [&skip_pauses] {
      ++skip_pauses;
      return ct::DoorPresentation::PagePauseAction::SkipToPrompt;
   });
   skipping_pager.resume_paging();
   for(unsigned line = 0; line < 23; ++line) {
      skipping_pager.write("visible record\n\r");
   }
   check(skip_pauses == 1);
   check(!skipping_pager.write("hidden record\n\r"));
   check(skipped_page.find("hidden record") == std::string::npos);
   check(skipping_pager.write("[Q] Back", ct::DoorTextRole::Prompt));
   check(skipped_page.find("(Q) Back") != std::string::npos);
   check(skipping_pager.write("\n\rvisible again"));
   check(skipped_page.find("visible again") != std::string::npos);
   skipping_pager.reset_paging();
   skipping_pager.resume_paging();
   for(unsigned line = 0; line < 23; ++line) {
      skipping_pager.write("second page\n\r");
   }
   check(skip_pauses == 2);
   check(!skipping_pager.write("second hidden record\n\r"));
   skipping_pager.clear();
   check(skipping_pager.write("new screen"));
   check(skipped_page.find("new screen") != std::string::npos);

   std::string wrapped_page;
   unsigned wrapped_pauses = 0;
   ct::DoorPresentation wrapped_pager(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&wrapped_page](const std::string_view bytes) {
         wrapped_page.append(bytes);
      });
   wrapped_pager.configure_paging(1, [&wrapped_pauses] {
      ++wrapped_pauses;
      return ct::DoorPresentation::PagePauseAction::Continue;
   });
   wrapped_pager.resume_paging();
   wrapped_pager.clear();
   wrapped_pager.write(std::string(40 * 24, 'x'));
   check(wrapped_pauses == 1);
   check(std::count(wrapped_page.begin(), wrapped_page.end(), '\f') == 1);

   check(ct::parse_door_profile("plain") == ct::DoorProfile::Iso646);
   check(ct::parse_door_profile("color") == ct::DoorProfile::Iso646Color);
   check(ct::parse_door_profile("cp437-plain") == ct::DoorProfile::Cp437);
   check(ct::parse_door_profile("cp437") == ct::DoorProfile::Cp437Color);
   check(ct::door_profile_for_capabilities(false, false) ==
         ct::DoorProfile::Iso646);
   check(ct::door_profile_for_capabilities(true, false) ==
         ct::DoorProfile::Iso646Color);
   check(ct::door_profile_for_capabilities(false, true) ==
         ct::DoorProfile::Cp437);
   check(ct::door_profile_for_capabilities(true, true) ==
         ct::DoorProfile::Cp437Color);

   ct::DisplayFormatting english_formatting{
      .decimal_separator = ".",
      .grouping_separator = ",",
      .primary_grouping_digits = 3,
      .secondary_grouping_digits = 3,
      .game_timestamp_pattern = "Day {day}, {hour}:{minute}:{second}",
      .game_duration_pattern = "{day} d {hour}:{minute}:{second}",
      .real_duration_pattern = "{hour}:{minute}:{second}",
   };
   ct::validate_display_formatting(english_formatting);
   check(ct::format_number_text("Cr1234567 and 1200000.50", english_formatting) ==
         "Cr1,234,567 and 1,200,000.50");
   check(ct::format_game_timestamp(
            1234ULL * 86400 + 2 * 3600 + 3 * 60 + 4,
            english_formatting) ==
         "Day 1234, 02:03:04");
   check(ct::format_game_duration(
            1234ULL * 86400 + 2 * 3600 + 3 * 60 + 4,
            english_formatting) ==
         "1234 d 02:03:04");
   check(ct::format_real_duration(
            1000ULL * 3600 + 2 * 60 + 3,
            english_formatting) ==
         "1000:02:03");

   auto continental_formatting = english_formatting;
   continental_formatting.decimal_separator = ",";
   continental_formatting.grouping_separator = ".";
   check(ct::format_number_text("1234567.50", continental_formatting) ==
         "1.234.567,50");
   auto indian_grouping = english_formatting;
   indian_grouping.secondary_grouping_digits = 2;
   check(ct::format_number_text("12345678", indian_grouping) ==
         "1,23,45,678");

   auto reordered_time = english_formatting;
   reordered_time.game_timestamp_pattern =
      "{hour}:{minute}:{second}, day {day}";
   check(ct::format_game_timestamp(86400 + 2 * 3600 + 3 * 60 + 4,
                                   reordered_time) ==
         "02:03:04, day 1");

   std::string formatted_output;
   ct::DoorPresentation formatted_presentation(
      ct::DoorProfile::Iso646,
      80,
      24,
      [&formatted_output](const std::string_view bytes) {
         formatted_output.append(bytes);
      });
   formatted_presentation.set_display_formatting(english_formatting);
   formatted_presentation.write("Cr1234567", ct::DoorTextRole::Number);
   formatted_presentation.write(" ID 1234567", ct::DoorTextRole::Identifier);
   formatted_presentation.write(
      std::string(" ") + ct::format_game_timestamp(
              1234ULL * 86400 + 2 * 3600 + 3 * 60 + 4,
              english_formatting),
      ct::DoorTextRole::Number);
   check(formatted_output ==
         "Cr1,234,567 ID 1234567 Day 1,234, 02:03:04");

   std::string continental_output;
   ct::DoorPresentation continental_presentation(
      ct::DoorProfile::Iso646,
      80,
      24,
      [&continental_output](const std::string_view bytes) {
         continental_output.append(bytes);
      });
   continental_presentation.set_display_formatting(continental_formatting);
   continental_presentation.write(
      ct::format_game_timestamp(
         1234ULL * 86400 + 2 * 3600 + 3 * 60 + 4,
         continental_formatting),
      ct::DoorTextRole::Number);
   check(continental_output == "Day 1.234, 02:03:04");

   std::string external_input_output;
   ct::DoorPresentation external_input(
      ct::DoorProfile::Iso646,
      80,
      24,
      [&external_input_output](const std::string_view bytes) {
         external_input_output.append(bytes);
      });
   external_input.write(
      "Offer No. (Q to cancel, ? for help): ",
      ct::DoorTextRole::Prompt);
   external_input_output += "1\r\n";
   external_input.reset_after_external_input();
   external_input.write(
      "insufficient uncommitted passenger accommodation\r\n",
      ct::DoorTextRole::Error);
   check(external_input_output.find(
            "1\r\ninsufficient uncommitted passenger accommodation\r\n") !=
         std::string::npos);
   check(ct::door_single_line_field("ship\r\n\x1b[31m") ==
         "ship  ?[31m");
   check(
      ct::door_plain_markdown(
         "# License\n\n## Designation\nSee [LICENSE.md](LICENSE.md).") ==
      "License\n\nDesignation\nSee LICENSE.md.");
   const auto wide_options =
      ct::door_option_prompt(
         {"[Letter] Listed action",
          "[L] License",
          "[Enter] Refresh",
          "[Q] Return to BBS",
          "[?] Help"},
         80);
   check(
      wide_options ==
      "\n\r[Enter] Refresh  [L] License  [Letter] Listed action  "
      "[Q] Return to BBS\n\r[?] Help: ");
   check(maximum_visible_width(wide_options) < 80);
   check(
      ct::door_option_prompt(
         {"[Letter] Listed action",
          "[L] License",
          "[Enter] Refresh",
          "[Q] Return to BBS",
          "[?] Help"},
         40) ==
      "\n\r[Enter] Refresh  [L] License\n\r"
      "[Letter] Listed action\n\r"
      "[Q] Return to BBS  [?] Help: ");
   check(
      ct::door_option_prompt(
         {"[Z] Alpha label", "[A] Zulu label", "[?] Hilfe"}, 80) ==
      "\n\r[A] Zulu label  [Z] Alpha label  [?] Hilfe: ");
   check(
      ct::door_option_prompt(
         {"[Q] Quit", "[B] Beta", "[?] Help", "[2] Two", "[< >] Page", "[A] Alpha"},
         80) ==
      "\n\r[2] Two  [< >] Page  [A] Alpha  [B] Beta  [Q] Quit  [?] Help: ");
}
