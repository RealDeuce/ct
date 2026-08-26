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
   const std::string enrollment_url =
      "https://x.io/#abcdefghijklmnopqrstuv";
   const auto cp437_square =
      ct::door_qr_code(enrollment_url, ct::DoorProfile::Cp437, 40);
   const auto cp437_wide =
      ct::door_qr_code(enrollment_url, ct::DoorProfile::Cp437, 80);
   const auto iso_square =
      ct::door_qr_code(enrollment_url, ct::DoorProfile::Iso646, 40);
   const auto iso_wide =
      ct::door_qr_code(enrollment_url, ct::DoorProfile::Iso646, 80);
   check(cp437_square.has_value());
   check(cp437_wide.has_value());
   check(iso_square.has_value());
   check(iso_wide.has_value());
   check(cp437_wide->size() * 2 >= cp437_square->size());
   check(cp437_wide->size() * 2 <= cp437_square->size() + 1);
   check(iso_square->size() == cp437_square->size());
   check(iso_wide->size() == iso_square->size());
   check(std::ranges::any_of(*cp437_wide, [](const std::string& line) {
      return line.find("\xe2\x96\x80") != std::string::npos ||
             line.find("\xe2\x96\x84") != std::string::npos ||
             line.find("\xe2\x96\x88") != std::string::npos;
   }));
   check(std::ranges::any_of(*iso_square, [](const std::string& line) {
      return line.find('M') != std::string::npos;
   }));
   check(!ct::door_qr_code(
      enrollment_url, ct::DoorProfile::Iso646, 39).has_value());

   for(const auto profile : {ct::DoorProfile::Cp437Color,
                              ct::DoorProfile::Iso646Color}) {
      std::string linked;
      ct::DoorPresentation presentation(
         profile,
         80,
         24,
         [&linked](const std::string_view bytes) { linked.append(bytes); });
      check(presentation.write_hyperlink(enrollment_url));
      check(linked.find("\x1b]8;;" + enrollment_url + "\x1b\\") !=
            std::string::npos);
      check(linked.find(enrollment_url) != std::string::npos);
   }
   std::string plain_link;
   ct::DoorPresentation plain_hyperlink_presentation(
      ct::DoorProfile::Iso646,
      80,
      24,
      [&plain_link](const std::string_view bytes) { plain_link.append(bytes); });
   check(plain_hyperlink_presentation.write_hyperlink(enrollment_url));
   check(plain_link == enrollment_url);

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

   const auto buy_spans = ct::styled_price_box_plot(
      900, 1'000, 1'100, 1'200, 1'300, 1'150, true);
   std::string rebuilt_buy_plot;
   bool found_buy_marker = false;
   bool found_favorable_buy_range = false;
   for(const auto& span : buy_spans) {
      rebuilt_buy_plot += span.text;
      found_favorable_buy_range = found_favorable_buy_range ||
         span.band == ct::PricePlotBand::Favorable;
      if(span.current_marker) {
         check(span.text == "*");
         check(span.band == ct::PricePlotBand::Unfavorable);
         found_buy_marker = true;
      }
   }
   check(rebuilt_buy_plot == price_plot);
   check(found_buy_marker);
   check(found_favorable_buy_range);

   const auto sale_spans = ct::styled_price_box_plot(
      900, 1'000, 1'100, 1'200, 1'300, 1'150, false);
   bool found_sale_marker = false;
   for(const auto& span : sale_spans) {
      if(span.current_marker) {
         check(span.text == "*");
         check(span.band == ct::PricePlotBand::Middling);
         found_sale_marker = true;
      }
   }
   check(found_sale_marker);

   const auto flat_spans = ct::styled_price_box_plot(
      1'000, 1'000, 1'000, 1'000, 1'000, 1'000, true);
   check(flat_spans.size() == 3);
   check(flat_spans.front().band == ct::PricePlotBand::None);
   check(flat_spans[1].text == "X");
   check(flat_spans[1].current_marker);
   check(flat_spans[1].band == ct::PricePlotBand::Unfavorable);
   check(flat_spans.back().band == ct::PricePlotBand::None);

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
      std::pair{ct::DoorTextRole::PagerPrompt, std::string_view{"\x1b[30;46m"}},
      std::pair{ct::DoorTextRole::Value, std::string_view{"\x1b[1;37m"}},
      std::pair{ct::DoorTextRole::Number, std::string_view{"\x1b[1;33m"}},
      std::pair{ct::DoorTextRole::Identifier, std::string_view{"\x1b[1;35m"}},
      std::pair{ct::DoorTextRole::Information, std::string_view{"\x1b[32m"}},
      std::pair{ct::DoorTextRole::Success, std::string_view{"\x1b[1;32m"}},
      std::pair{ct::DoorTextRole::Warning, std::string_view{"\x1b[1;31m"}},
      std::pair{ct::DoorTextRole::PriceFavorable, std::string_view{"\x1b[30;42m"}},
      std::pair{ct::DoorTextRole::PriceMiddling, std::string_view{"\x1b[30;43m"}},
      std::pair{ct::DoorTextRole::PriceUnfavorable, std::string_view{"\x1b[37;41m"}},
      std::pair{ct::DoorTextRole::PriceMarkerFavorable, std::string_view{"\x1b[1;37;42m"}},
      std::pair{ct::DoorTextRole::PriceMarkerMiddling, std::string_view{"\x1b[1;34;43m"}},
      std::pair{ct::DoorTextRole::PriceMarkerUnfavorable, std::string_view{"\x1b[1;33;41m"}},
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

   for(const auto columns : {size_t{40}, size_t{80}}) {
      std::string fields;
      ct::DoorPresentation field_presentation(
         ct::DoorProfile::Iso646,
         columns,
         24,
         [&fields](const std::string_view bytes) { fields.append(bytes); });
      const auto label_column = field_presentation.labeled_field_column(
         {"Service:", "Failure charge:", "Performing ship:"});
      check(label_column == 16);
      field_presentation.write_labeled_field(
         "Service:", label_column,
         {{"Freight contract", ct::DoorTextRole::Value}});
      field_presentation.write_labeled_field(
         "Failure charge:", label_column,
         {{"Cr", ct::DoorTextRole::Label},
          {"25000", ct::DoorTextRole::Number}});
      field_presentation.write_labeled_field(
         "Performing ship:", label_column,
         {{"Far Horizon", ct::DoorTextRole::Identifier}});
      check(
         fields ==
         "Service:          Freight contract\r\n"
         "Failure charge:   Cr25000\r\n"
         "Performing ship:  Far Horizon\r\n");
      check(maximum_visible_width(fields) < columns);
   }

   std::string colored_field;
   ct::DoorPresentation colored_field_presentation(
      ct::DoorProfile::Iso646Color,
      80,
      24,
      [&colored_field](const std::string_view bytes) {
         colored_field.append(bytes);
      });
   colored_field_presentation.write_labeled_field(
      "Payment:", 15,
      {{"Cr", ct::DoorTextRole::Label},
       {"18446744073709551615", ct::DoorTextRole::Number}});
   check(
      colored_field ==
      "\x1b[36mPayment:\x1b[0m"
      "\x1b[36m         \x1b[0m"
      "\x1b[36mCr\x1b[0m"
      "\x1b[1;33m18446744073709551615\x1b[0m"
      "\x1b[0m\r\n\x1b[0m");

   std::string long_field;
   ct::DoorPresentation long_field_presentation(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&long_field](const std::string_view bytes) { long_field.append(bytes); });
   const std::string long_field_label =
      "Extraordinarily Long Administrative Instrument Label:";
   const auto long_field_column =
      long_field_presentation.labeled_field_column({long_field_label});
   check(long_field_column == 29);
   long_field_presentation.write_labeled_field(
      long_field_label,
      long_field_column,
      {{"Ready", ct::DoorTextRole::Success}});
   check(
      long_field ==
      "Extraordinarily Long\r\n"
      "Administrative Instrument\r\n"
      "Label:                         Ready\r\n");
   check(maximum_visible_width(long_field) < 40);

   std::string encoded_fields;
   ct::DoorPresentation encoded_field_presentation(
      ct::DoorProfile::Iso646,
      80,
      24,
      [&encoded_fields](const std::string_view bytes) {
         encoded_fields.append(bytes);
      });
   check(encoded_field_presentation.labeled_field_column(
            {"Hull:", "Reactor #1:"}) == 13);
   ct::DoorPresentation cp437_field_presentation(
      ct::DoorProfile::Cp437,
      80,
      24,
      [&encoded_fields](const std::string_view bytes) {
         encoded_fields.append(bytes);
      });
   check(cp437_field_presentation.labeled_field_column(
            {"Hull:", "R\xc3\xa9" "acteur:"}) == 9);

   std::string aborted_field;
   unsigned field_abort_pauses = 0;
   ct::DoorPresentation aborting_field_presentation(
      ct::DoorProfile::Iso646,
      40,
      24,
      [&aborted_field](const std::string_view bytes) {
         aborted_field.append(bytes);
      });
   aborting_field_presentation.configure_paging(
      23,
      [&field_abort_pauses] {
         ++field_abort_pauses;
         return ct::DoorPresentation::PagePauseAction::Abort;
      });
   aborting_field_presentation.resume_paging();
   check(!aborting_field_presentation.write_labeled_field(
      "Terms:",
      16,
      {{"A value long enough to cross the first page boundary and abort",
        ct::DoorTextRole::Identifier},
       {"hidden", ct::DoorTextRole::Warning}}));
   check(field_abort_pauses == 1);
   check(aborted_field.find("hidden") == std::string::npos);

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

   const auto subsystem_row =
      [](const ct::DoorProfile profile, const size_t columns,
         const std::string& label, const size_t label_width,
         const std::string& status,
         const ct::DoorTextRole status_role) -> std::string {
      std::string row;
      ct::DoorPresentation presentation(
         profile,
         columns,
         24,
         [&row](const std::string_view bytes) { row.append(bytes); });
      presentation.write_ship_subsystem_row(
         'A', label, label_width, status, status_role);
      return row;
   };

   const std::string hull_gap(20, ' ');
   const std::string long_label = "Powered Armor Maintenance Workshop";
   const std::string long_prefix = "A. " + long_label + "  ";

   const struct {
      ct::DoorProfile profile;
      size_t columns;
      std::string label;
      size_t label_width;
      std::string status;
      ct::DoorTextRole status_role;
      std::string expected;
   } row_cases[] = {
      {ct::DoorProfile::Iso646, 40, "Hull", 22, "Ready",
       ct::DoorTextRole::Value, "A. Hull" + hull_gap + "Ready\r\n"},
      {ct::DoorProfile::Iso646, 80, "Hull", 22, "Ready",
       ct::DoorTextRole::Value, "A. Hull" + hull_gap + "Ready\r\n"},
      {ct::DoorProfile::Iso646, 40, "Hull", 22, "Damage 3/5",
       ct::DoorTextRole::Warning, "A. Hull" + hull_gap + "Damage 3/5\r\n"},
      {ct::DoorProfile::Iso646, 80, "Hull", 22, "Damage 3/5",
       ct::DoorTextRole::Warning, "A. Hull" + hull_gap + "Damage 3/5\r\n"},
      {ct::DoorProfile::Iso646, 40, "Hull", 22, "Patched 3/5",
       ct::DoorTextRole::Warning, "A. Hull" + hull_gap + "Patched 3/5\r\n"},
      {ct::DoorProfile::Iso646, 80, "Hull", 22, "Patched 3/5",
       ct::DoorTextRole::Warning, "A. Hull" + hull_gap + "Patched 3/5\r\n"},
      // A label too wide for the column continues on further lines indented
      // under it, and the status sits beside the last of them.
      {ct::DoorProfile::Iso646, 40, long_label, 34, "Ready",
       ct::DoorTextRole::Value,
       "A. Powered Armor Maintenance\r\n   Workshop" + std::string(23, ' ') +
          "Ready\r\n"},
      {ct::DoorProfile::Iso646, 80, long_label, 34, "Ready",
       ct::DoorTextRole::Value, long_prefix + "Ready\r\n"},
      {ct::DoorProfile::Iso646, 40, long_label, 34, "Damage 3/5",
       ct::DoorTextRole::Warning,
       "A. Powered Armor\r\n   Maintenance Workshop" + std::string(6, ' ') +
          "Damage 3/5\r\n"},
      {ct::DoorProfile::Iso646, 80, long_label, 34, "Damage 3/5",
       ct::DoorTextRole::Warning, long_prefix + "Damage 3/5\r\n"},
      {ct::DoorProfile::Iso646, 40, long_label, 34, "Patched 3/5",
       ct::DoorTextRole::Warning,
       "A. Powered Armor\r\n   Maintenance Workshop" + std::string(5, ' ') +
          "Patched 3/5\r\n"},
      {ct::DoorProfile::Iso646, 80, long_label, 34, "Patched 3/5",
       ct::DoorTextRole::Warning, long_prefix + "Patched 3/5\r\n"},
      // The break falls on a word boundary.
      {ct::DoorProfile::Iso646, 40, "Aft Sensor Array Cluster", 23,
       "Patched 3/5", ct::DoorTextRole::Warning,
       "A. Aft Sensor Array\r\n   Cluster" + std::string(18, ' ') +
          "Patched 3/5\r\n"},
      // The longest label the ship catalog can produce takes three lines and
      // loses nothing.
      {ct::DoorProfile::Iso646, 40,
       "Powered Armor Maintenance Workshop group (25)", 23, "Patched 3/5",
       ct::DoorTextRole::Warning,
       "A. Powered Armor\r\n   Maintenance Workshop\r\n   group (25)" +
          std::string(15, ' ') + "Patched 3/5\r\n"},
      {ct::DoorProfile::Iso646, 40, "Underway Replenishment System group (10)",
       23, "Ready", ct::DoorTextRole::Value,
       "A. Underway Replenishment\r\n   System group (10)" +
          std::string(8, ' ') + "Ready\r\n"},
      // A label far beyond anything the catalog holds still loses nothing.
      {ct::DoorProfile::Iso646, 40,
       "Aaaa Bbbb Cccc Dddd Eeee Ffff Gggg Hhhh Iiii Jjjj Kkkk Llll Mmmm", 23,
       "Ready", ct::DoorTextRole::Value,
       "A. Aaaa Bbbb Cccc Dddd\r\n   Eeee Ffff Gggg Hhhh\r\n"
       "   Iiii Jjjj Kkkk Llll\r\n   Mmmm" + std::string(21, ' ') +
          "Ready\r\n"},
      // ISO 646 expands '#' to "No.", so a raw byte count would push the
      // status column three places right for catalog labels containing one.
      {ct::DoorProfile::Iso646, 80, "Cargo Bay", 22, "Ready",
       ct::DoorTextRole::Value, "A. Cargo Bay" + std::string(15, ' ') + "Ready\r\n"},
      {ct::DoorProfile::Iso646, 80, "Reactor #1", 22, "Ready",
       ct::DoorTextRole::Value,
       "A. Reactor No.1" + std::string(12, ' ') + "Ready\r\n"},
      // CP437 contracts multi-byte UTF-8 back to a single byte.
      {ct::DoorProfile::Cp437, 80, "R\xc3\xa9""acteur", 22, "Ready",
       ct::DoorTextRole::Value,
       "A. R\x82""acteur" + std::string(16, ' ') + "Ready\r\n"},
      // Selector, punctuation and name keep their established roles.
      {ct::DoorProfile::Iso646Color, 80, "Reactor #1", 22, "Ready",
       ct::DoorTextRole::Value,
       "\x1b[1;33mA\x1b[0m"
       "\x1b[36m. \x1b[0m"
       "\x1b[1;35mReactor No.1\x1b[0m"
       "\x1b[36m" + std::string(12, ' ') + "\x1b[0m"
       "\x1b[1;37mReady\x1b[0m"
       "\x1b[0m\r\n\x1b[0m"},
      // A label column wider than the terminal must not wrap a short label.
      {ct::DoorProfile::Iso646, 40, "Hull", 34, "Ready",
       ct::DoorTextRole::Value, "A. Hull" + std::string(27, ' ') + "Ready\r\n"},
      // Two rows from one page share a column, so a short label puts Ready and
      // Patched in the same place. Clamping against each row's own status
      // instead would widen the Ready row by one column.
      {ct::DoorProfile::Iso646, 40, "Hull", 23, "Ready",
       ct::DoorTextRole::Value, "A. Hull" + std::string(21, ' ') + "Ready\r\n"},
      {ct::DoorProfile::Iso646, 40, "Hull", 23, "Patched 3/5",
       ct::DoorTextRole::Warning,
       "A. Hull" + std::string(21, ' ') + "Patched 3/5\r\n"},
   };

   // Row height drives the caller's page budget, so it must agree with what
   // the row writer emits.
   const auto row_height =
      [](const std::string& label, const size_t label_width,
         const std::string& status) -> std::pair<size_t, size_t> {
      std::string row;
      ct::DoorPresentation presentation(
         ct::DoorProfile::Iso646,
         40,
         24,
         [&row](const std::string_view bytes) { row.append(bytes); });
      const auto claimed =
         presentation.ship_subsystem_row_lines(label, label_width, status);
      presentation.write_ship_subsystem_row(
         'A', label, label_width, status, ct::DoorTextRole::Value);
      size_t emitted = 0;
      for(const char byte : row) {
         if(byte == '\n') {
            ++emitted;
         }
      }
      return {claimed, emitted};
   };

   check(row_height("Hull", 23, "Ready") == std::pair<size_t, size_t>{1, 1});
   check(row_height("Underway Replenishment System group (10)", 23, "Ready") ==
         std::pair<size_t, size_t>{2, 2});
   // door_single_line_field keeps trailing whitespace, which overflows the
   // column without producing a second line.
   check(row_height("Hull" + std::string(30, ' '), 23, "Ready") ==
         std::pair<size_t, size_t>{1, 1});

   for(const auto& row_case : row_cases) {
      const auto rendered = subsystem_row(
         row_case.profile,
         row_case.columns,
         row_case.label,
         row_case.label_width,
         row_case.status,
         row_case.status_role);
      check(rendered == row_case.expected);
      check(maximum_visible_width(rendered) < row_case.columns);
   }

   // The page-wide label column must depend only on the widest label and the
   // widest status on the page, never on an individual row's status, or rows
   // carrying a shorter status would be given a wider column than their
   // neighbours and the status column would go ragged within one page.
   {
      std::string ignored;
      ct::DoorPresentation narrow(
         ct::DoorProfile::Iso646,
         40,
         24,
         [&ignored](const std::string_view bytes) { ignored.append(bytes); });
      // Content width 39, so a 24-column label and an 11-column status cannot
      // share a line; the column gives way to the status.
      check(narrow.ship_subsystem_label_column(24, 11) == 23);
      check(narrow.ship_subsystem_label_column(24, 5) == 24);
      // A page that fits keeps the widest label intact.
      check(narrow.ship_subsystem_label_column(20, 11) == 20);
      // A status that cannot fit at all collapses the column rather than
      // underflowing the subtraction.
      check(narrow.ship_subsystem_label_column(20, 60) == 0);

      ct::DoorPresentation wide(
         ct::DoorProfile::Iso646,
         80,
         24,
         [&ignored](const std::string_view bytes) { ignored.append(bytes); });
      check(wide.ship_subsystem_label_column(24, 11) == 24);
   }
}
