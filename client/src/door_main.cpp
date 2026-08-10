#include "ct/bbs_config.hpp"
#include "ct/bbs_credential.hpp"
#include "ct/cargo_quantity.hpp"
#include "ct/crew_presentation.hpp"
#include "ct/crypto.hpp"
#include "ct/door_help.hpp"
#include "ct/door_presentation.hpp"
#include "ct/legal_text.hpp"
#include "ct/player_identity_registry.hpp"
#include "ct/protocol.hpp"
#include "ct/tls_connection.hpp"

extern "C" {
#include <OpenDoor.h>
}

#include <algorithm>
#include <array>
#include <charconv>
#include <cctype>
#include <chrono>
#include <cmath>
#include <cstdarg>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <exception>
#include <initializer_list>
#include <memory>
#include <limits>
#include <numeric>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <unordered_map>
#include <unordered_set>
#include <vector>

namespace
{

std::string course_duration(uint64_t seconds);
std::string game_date(uint64_t seconds);

std::string bbs_config_path = "cepheus-trader.conf";
std::optional<std::string> opendoors_profile;
std::optional<size_t> opendoors_columns;
std::optional<size_t> opendoors_rows;
std::string opendoors_configuration_error;

void parse_opendoors_dimension(char* options,
                              const size_t minimum,
                              std::optional<size_t>& destination)
{
   size_t value = 0;
   const std::string_view text(options == nullptr ? "" : options);
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() ||
      value < minimum || value > 255) {
      opendoors_configuration_error = "invalid Cepheus Trader terminal dimension";
      return;
   }
   destination = value;
}

void opendoors_config_line(char* keyword, char* options)
{
   if(std::strcmp(keyword, "CTCONFIG") == 0 && options != nullptr &&
      options[0] != '\0') {
      bbs_config_path = options;
   } else if(std::strcmp(keyword, "CTPROFILE") == 0 && options != nullptr) {
      opendoors_profile = options;
   } else if(std::strcmp(keyword, "CTCOLUMNS") == 0) {
      parse_opendoors_dimension(options, 40, opendoors_columns);
   } else if(std::strcmp(keyword, "CTROWS") == 0) {
      parse_opendoors_dimension(options, 24, opendoors_rows);
   }
}

std::unique_ptr<ct::DoorPresentation> presentation;
ct::TlsConnection* event_connection = nullptr;
uint64_t event_session_epoch = 0;
std::optional<ct::TravelStatus> latest_phase_status;
std::optional<ct::TrafficSnapshot> latest_traffic_snapshot;
std::optional<ct::CheckpointSnapshot> latest_checkpoint;
std::optional<ct::EncounterSnapshot> latest_encounter;
std::vector<std::string> pending_traffic_notices;
uint64_t pending_radio_unread = 0;
uint64_t observed_radio_ship_id = 0;
uint64_t observed_radio_unread = 0;
uint64_t phase_event_generation = 0;
uint64_t displayed_phase_event_generation = 0;
std::string active_prompt;
bool active_prompt_on_current_line = false;
ct::DoorHelpTopic active_help_topic = ct::DoorHelpTopic::General;

class HelpScope {
public:
   explicit HelpScope(const ct::DoorHelpTopic topic)
      : previous_(active_help_topic)
   {
      active_help_topic = topic;
   }

   ~HelpScope()
   {
      active_help_topic = previous_;
   }

   HelpScope(const HelpScope&) = delete;
   HelpScope& operator=(const HelpScope&) = delete;

private:
   ct::DoorHelpTopic previous_;
};

void door_write(std::string_view text, ct::DoorTextRole role);
void door_printf(const char* format, ...);
std::string safe_field(std::string_view text);
const char* travel_stage_name(ct::TravelStage stage);
const char* ship_activity_name(ct::ShipActivityKind kind);
const char* phase_name(ct::PlayerPhase phase);
void wait_for_enter(const char* destination = "Previous menu");
bool confirm_return_to_bbs();
void show_context_help();
int door_get_key(BOOL wait);

ct::DoorPresentation& output()
{
   if(!presentation) {
      throw std::logic_error("door presentation is not initialized");
   }
   return *presentation;
}

void collect_player_events()
{
   if(event_connection == nullptr) {
      return;
   }
   while(const auto event = ct::poll_event(*event_connection, event_session_epoch)) {
      switch(event->kind) {
      case ct::PlayerEventKind::SessionReplaced:
         throw std::runtime_error("this player session was replaced");
      case ct::PlayerEventKind::ServerStopping:
         throw std::runtime_error("the game server is stopping");
      case ct::PlayerEventKind::PhaseChanged:
         latest_phase_status = event->travel_status;
         ++phase_event_generation;
         break;
      case ct::PlayerEventKind::TrafficSnapshot:
         latest_traffic_snapshot = event->traffic_snapshot;
         break;
      case ct::PlayerEventKind::TrafficMovement: {
         const auto& contact = *event->traffic_contact;
         std::ostringstream notice;
         const char* movement = "Present";
         if(contact.movement == ct::TrafficMovementKind::Arrival) {
            movement = "Arrival";
         } else if(contact.movement == ct::TrafficMovementKind::Departure) {
            movement = "Departure";
         }
         notice << movement
                << ": " << contact.ship_name << " (" << contact.class_name
                << ", " << contact.role << ")";
         pending_traffic_notices.push_back(notice.str());
         break;
      }
      case ct::PlayerEventKind::CheckpointReady:
         latest_checkpoint = event->checkpoint;
         break;
      case ct::PlayerEventKind::EncounterReady:
         latest_encounter = event->encounter;
         break;
      case ct::PlayerEventKind::RadioUnread:
         if(event->ship_id != observed_radio_ship_id) {
            observed_radio_ship_id = event->ship_id;
            observed_radio_unread = 0;
         }
         if(event->unread_count > observed_radio_unread) {
            pending_radio_unread = event->unread_count;
         }
         observed_radio_unread = event->unread_count;
         break;
      }
   }
}

void flush_player_events()
{
   const bool has_traffic_snapshot = latest_traffic_snapshot.has_value();
   const bool has_traffic_notices = !pending_traffic_notices.empty();
   const bool has_phase_notice =
      latest_phase_status.has_value() &&
      displayed_phase_event_generation != phase_event_generation;
   const bool has_radio_notice = pending_radio_unread != 0;
   if(!has_traffic_snapshot && !has_traffic_notices && !has_phase_notice &&
      !has_radio_notice) {
      return;
   }

   const auto prompt = active_prompt;
   const auto prompt_on_current_line = active_prompt_on_current_line;
   if(prompt_on_current_line && !prompt.empty()) {
      output().erase_prompt(prompt.size());
      active_prompt.clear();
      active_prompt_on_current_line = false;
   }
   output().resume_paging();
   const auto event_prefix = prompt_on_current_line ? "" : "\n\r";
   if(latest_traffic_snapshot.has_value()) {
      door_write(event_prefix, ct::DoorTextRole::Normal);
      door_write("Traffic control report: ", ct::DoorTextRole::Label);
      door_printf("%zu contact%s tracked in %s.\n\r",
                  latest_traffic_snapshot->contacts.size(),
                  latest_traffic_snapshot->contacts.size() == 1 ? "" : "s",
                  safe_field(latest_traffic_snapshot->system_name).c_str());
      latest_traffic_snapshot.reset();
   }
   for(const auto& notice : pending_traffic_notices) {
      door_write(event_prefix, ct::DoorTextRole::Normal);
      door_write("[Traffic] ", ct::DoorTextRole::Heading);
      door_printf("%s\n\r", safe_field(notice).c_str());
   }
   pending_traffic_notices.clear();
   if(pending_radio_unread != 0) {
      door_write(event_prefix, ct::DoorTextRole::Normal);
      door_write("[System Common] ", ct::DoorTextRole::Heading);
      door_printf("%llu unread reception%s.\n\r",
                  static_cast<unsigned long long>(pending_radio_unread),
                  pending_radio_unread == 1 ? "" : "s");
      pending_radio_unread = 0;
   }
   if(latest_phase_status.has_value() &&
         displayed_phase_event_generation != phase_event_generation) {
      door_write(event_prefix, ct::DoorTextRole::Normal);
      door_write("[Ship status] ", ct::DoorTextRole::Heading);
      door_printf("%s - %s (%s)\n\r",
                  phase_name(latest_phase_status->phase),
                  travel_stage_name(latest_phase_status->stage),
                  game_date(latest_phase_status->current_game_second).c_str());
      displayed_phase_event_generation = phase_event_generation;
   }
   active_prompt = prompt;
   if(!active_prompt.empty()) {
      output().write(active_prompt, ct::DoorTextRole::Prompt);
      active_prompt_on_current_line = prompt_on_current_line;
   }
   output().suspend_paging();
}

bool prompt_awaits_input_on_current_line()
{
   return !active_prompt.empty() &&
          active_prompt.back() != '\r' && active_prompt.back() != '\n';
}

void echo_prompt_key(const int key, const bool preserve_prompt)
{
   if(prompt_awaits_input_on_current_line()) {
      std::string response;
      if(key >= 0x20 && key <= 0x7e) {
         response.push_back(static_cast<char>(key));
      }
      response += "\n\r";
      output().write(response, ct::DoorTextRole::Prompt);
   }
   if(!preserve_prompt) {
      active_prompt.clear();
      active_prompt_on_current_line = false;
   }
}

int door_get_live_key()
{
   for(;;) {
      collect_player_events();
      flush_player_events();
      const auto key = od_get_key(FALSE);
      if(key != 0) {
         output().reset_paging();
         output().resume_paging();
      }
      if(key == '?') {
         echo_prompt_key(key, true);
         show_context_help();
         continue;
      }
      if(key != 0) {
         echo_prompt_key(key, false);
         return key;
      }
      od_sleep(10);
   }
}

int door_get_translated_key()
{
   while(true) {
      tODInputEvent event{};
      while(!od_get_input(&event, OD_NO_TIMEOUT, GETIN_NORMAL)) {
      }
      const auto key = static_cast<unsigned char>(event.chKeyPress);
      output().reset_paging();
      output().resume_paging();
      if(key == '?') {
         echo_prompt_key(key, true);
         show_context_help();
         continue;
      }
      echo_prompt_key(key, false);
      return key;
   }
}

void door_clear_screen()
{
   active_prompt.clear();
   active_prompt_on_current_line = false;
   output().clear();
   output().resume_paging();
}

void door_write(const std::string_view text,
                const ct::DoorTextRole role = ct::DoorTextRole::Normal)
{
   if(role != ct::DoorTextRole::Prompt) {
      active_prompt.clear();
      active_prompt_on_current_line = false;
   }
   output().write(text, role);
}

std::string format_door_text(const char* format, va_list arguments)
{
   va_list sizing_arguments;
   va_copy(sizing_arguments, arguments);
   const auto size = std::vsnprintf(nullptr, 0, format, sizing_arguments);
   va_end(sizing_arguments);
   if(size < 0) {
      throw std::runtime_error("could not format door output");
   }
   std::vector<char> buffer(static_cast<size_t>(size) + 1);
   const auto written =
      std::vsnprintf(buffer.data(), buffer.size(), format, arguments);
   if(written != size) {
      throw std::runtime_error("could not format door output");
   }
   return std::string(buffer.data(), static_cast<size_t>(written));
}

void door_printf_role(const ct::DoorTextRole role,
                      const char* format,
                      va_list arguments)
{
   door_write(format_door_text(format, arguments), role);
}

void door_printf(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Normal, format, arguments);
   va_end(arguments);
}

void door_prompt(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   auto text = format_door_text(format, arguments);
   va_end(arguments);
   const auto has_prompt_text =
      text.find_first_not_of("\r\n") != std::string::npos;
   bool removed_line_ending = false;
   if(has_prompt_text) {
      while(!text.empty() && (text.back() == '\r' || text.back() == '\n')) {
         text.pop_back();
         removed_line_ending = true;
      }
      if(removed_line_ending && !text.empty() && text.back() != ' ') {
         text.push_back(' ');
      }
   }
   active_prompt += text;
   active_prompt_on_current_line = false;
   output().write(text, ct::DoorTextRole::Prompt);
   output().suspend_paging();
}

void door_live_prompt(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   const auto text = format_door_text(format, arguments);
   va_end(arguments);
   if(text.find_first_of("\r\n") != std::string::npos) {
      throw std::logic_error("a live prompt must occupy one line");
   }
   if(text.size() > output().content_columns()) {
      throw std::logic_error("a live prompt must fit on one terminal row");
   }
   active_prompt = text;
   active_prompt_on_current_line = true;
   output().write(text, ct::DoorTextRole::Prompt);
   output().suspend_paging();
}

void show_voyage_live_prompt()
{
   constexpr const char* wide_prompt =
      "[F] Revise Flight Plan  [Enter] Command console  [?] Help";
   constexpr const char* narrow_prompt =
      "[F] Plan  [Enter] Console  [?] Help";
   door_write("\n\r", ct::DoorTextRole::Normal);
   door_live_prompt(
      "%s",
      output().content_columns() >= std::strlen(wide_prompt)
         ? wide_prompt
         : narrow_prompt);
}

void door_option_prompt(
   const std::initializer_list<std::string_view> options,
   const bool leading_newline = true)
{
   const auto prompt =
      ct::door_option_prompt(options, output().columns(), leading_newline);
   door_prompt("%s", prompt.c_str());
}

void door_option_prompt(
   const std::vector<std::string_view>& options,
   const bool leading_newline = true)
{
   const auto prompt = ct::door_option_prompt(
      std::span<const std::string_view>(options),
      output().columns(),
      leading_newline);
   door_prompt("%s", prompt.c_str());
}

void render_combat_countdown_prompt(const uint64_t remaining_seconds)
{
   if(active_prompt_on_current_line && !active_prompt.empty()) {
      output().erase_prompt(active_prompt.size());
      active_prompt.clear();
      active_prompt_on_current_line = false;
   }
   const auto minutes = remaining_seconds / 60;
   const auto seconds = remaining_seconds % 60;
   door_live_prompt(
      "Orders take effect in %02llu:%02llu  Command: ",
      static_cast<unsigned long long>(minutes),
      static_cast<unsigned long long>(seconds));
}

int door_get_combat_countdown_key(
   const std::chrono::steady_clock::time_point deadline)
{
   std::optional<uint64_t> rendered_seconds;
   const auto initial_phase_generation = phase_event_generation;
   for(;;) {
      collect_player_events();
      flush_player_events();
      if(phase_event_generation != initial_phase_generation) {
         return 0;
      }

      const auto now = std::chrono::steady_clock::now();
      const auto remaining_milliseconds = now < deadline
         ? static_cast<uint64_t>(std::chrono::duration_cast<std::chrono::milliseconds>(
              deadline - now).count())
         : 0;
      const auto remaining_seconds = (remaining_milliseconds + 999) / 1000;
      if(rendered_seconds != remaining_seconds) {
         render_combat_countdown_prompt(remaining_seconds);
         rendered_seconds = remaining_seconds;
      }

      const auto until_next_tick = remaining_milliseconds == 0
         ? uint64_t{500}
         : std::max<uint64_t>(
              1,
              remaining_milliseconds - (remaining_seconds - 1) * 1000);
      tODInputEvent event{};
      if(!::od_get_input(
            &event,
            static_cast<tODMilliSec>(std::min<uint64_t>(until_next_tick, 1000)),
            GETIN_NORMAL)) {
         if(remaining_milliseconds == 0) {
            return 0;
         }
         continue;
      }
      const auto key = static_cast<unsigned char>(event.chKeyPress);
      if(key == '\n') {
         continue;
      }
      output().reset_paging();
      output().resume_paging();
      if(key == '?') {
         echo_prompt_key(key, true);
         show_context_help();
         rendered_seconds.reset();
         continue;
      }
      echo_prompt_key(key, false);
      return key;
   }
}

std::chrono::steady_clock::time_point combat_order_deadline(
   const ct::CombatSnapshot& combat,
   const uint64_t current_game_second)
{
   const auto turn_game_seconds = combat.order_due_second > combat.round_started_second
      ? combat.order_due_second - combat.round_started_second
      : uint64_t{0};
   const auto remaining_game_seconds = std::min(
      turn_game_seconds,
      combat.order_due_second > current_game_second
         ? combat.order_due_second - current_game_second
         : uint64_t{0});
   const auto remaining_real_milliseconds = turn_game_seconds == 0
      ? uint64_t{0}
      : (combat.order_window_real_milliseconds * remaining_game_seconds
         + turn_game_seconds - 1) / turn_game_seconds;
   return std::chrono::steady_clock::now()
          + std::chrono::milliseconds(remaining_real_milliseconds);
}

void door_error(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Error, format, arguments);
   va_end(arguments);
}

void door_heading(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Heading, format, arguments);
   va_end(arguments);
}

void door_accent(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Accent, format, arguments);
   va_end(arguments);
}

void door_label(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Label, format, arguments);
   va_end(arguments);
}

void door_value(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Value, format, arguments);
   va_end(arguments);
}

void door_number(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Number, format, arguments);
   va_end(arguments);
}

void door_identifier(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Identifier, format, arguments);
   va_end(arguments);
}

void door_information(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Information, format, arguments);
   va_end(arguments);
}

void door_success(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Success, format, arguments);
   va_end(arguments);
}

void door_warning(const char* format, ...)
{
   va_list arguments;
   va_start(arguments, format);
   door_printf_role(ct::DoorTextRole::Warning, format, arguments);
   va_end(arguments);
}

std::string safe_field(const std::string_view text)
{
   return ct::door_single_line_field(text);
}

const char* travel_stage_name(const ct::TravelStage stage)
{
   switch(stage) {
   case ct::TravelStage::Docked:
      return "Docked";
   case ct::TravelStage::DepartingForJump:
      return "Departing for jump locus";
   case ct::TravelStage::JumpSpace:
      return "In jump space";
   case ct::TravelStage::ApproachingStarport:
      return "Approaching destination starport";
   case ct::TravelStage::Refit:
      return "Refit in progress";
   case ct::TravelStage::ProperRepair:
      return "Proper subsystem repair in progress";
   case ct::TravelStage::GasGiantSkim:
      return "Gas-giant skimming expedition";
   case ct::TravelStage::WildernessWater:
      return "Wilderness water/ice expedition";
   case ct::TravelStage::Holding:
      return "Holding for captain";
   case ct::TravelStage::Encounter:
      return "Encounter";
   }
   return "Unknown";
}

const char* ship_activity_name(const ct::ShipActivityKind kind)
{
   switch(kind) {
   case ct::ShipActivityKind::Refit:
      return "Refit";
   case ct::ShipActivityKind::Refurbishment:
      return "Component replacement";
   case ct::ShipActivityKind::ProperRepair:
      return "Proper subsystem repair";
   case ct::ShipActivityKind::GasGiantSkim:
      return "Gas-giant skimming";
   case ct::ShipActivityKind::WildernessWater:
      return "Wilderness water collection";
   case ct::ShipActivityKind::EscortDuty:
      return "Escort duty";
   case ct::ShipActivityKind::FieldRecovery:
      return "Crew field recovery";
   }
   return "Unknown";
}

// Keep the page code readable while routing every byte through the
// profile-aware sanitizer and wrapper.
#define od_clr_scr() door_clear_screen()
#define od_printf(...) door_printf(__VA_ARGS__)

bool confirm_return_to_bbs()
{
   door_prompt("\n\rReturn to the BBS?\n\r");
   door_option_prompt({"[Y] Yes", "[N/Enter] Stay", "[?] Help"}, false);
   while(true) {
      const auto key = static_cast<char>(
         std::toupper(static_cast<unsigned char>(door_get_key(TRUE))));
      if(key == 'Y') {
         od_printf("\n\r");
         return true;
      }
      if(key == 'N' || key == '\r' || key == '\n') {
         od_printf("\n\r");
         return false;
      }
   }
}

const char* phase_name(const ct::PlayerPhase phase)
{
   switch(phase) {
   case ct::PlayerPhase::Disconnected:
      return "Disconnected";
   case ct::PlayerPhase::Jump:
      return "Jump";
   case ct::PlayerPhase::Interplanetary:
      return "Interplanetary";
   case ct::PlayerPhase::Encounter:
      return "Encounter";
   case ct::PlayerPhase::OnPlanet:
      return "On planet";
   case ct::PlayerPhase::NewUser:
      return "Unregistered";
   case ct::PlayerPhase::Docked:
      return "Docked";
   case ct::PlayerPhase::Terminal:
      return "Command lost";
   case ct::PlayerPhase::Other:
      return "Unavailable";
   }
   return "Unavailable";
}

std::string wall_duration(
   const uint64_t game_seconds,
   const uint64_t clock_rate_game_seconds,
   const uint64_t clock_rate_real_seconds)
{
   if(clock_rate_game_seconds == 0 || clock_rate_real_seconds == 0) {
      return "--:--:--";
   }
   const auto scaled_seconds = std::ceil(
                                  static_cast<long double>(game_seconds) *
                                  static_cast<long double>(clock_rate_real_seconds) /
                                  static_cast<long double>(clock_rate_game_seconds));
   const auto total_seconds =
      scaled_seconds >= static_cast<long double>(UINT64_MAX)
      ? UINT64_MAX
      : static_cast<uint64_t>(scaled_seconds);
   return ct::format_real_duration(
      total_seconds, output().display_formatting());
}

std::string real_time_until(const ct::TravelStatus& status)
{
   const auto game_seconds =
      status.due_second > status.current_game_second
      ? status.due_second - status.current_game_second
      : uint64_t{0};
   return wall_duration(
      game_seconds,
      status.clock_rate_game_seconds,
      status.clock_rate_real_seconds);
}

std::vector<std::string> wrap_text(const std::string_view text,
                                   const size_t width)
{
   std::vector<std::string> result;
   std::istringstream input{std::string(text)};
   std::string source_line;
   while(std::getline(input, source_line)) {
      if(source_line.empty()) {
         result.emplace_back();
         continue;
      }
      while(source_line.size() > width) {
         auto split = source_line.rfind(' ', width);
         if(split == std::string::npos) {
            split = width;
         }
         result.emplace_back(source_line.substr(0, split));
         const auto next = source_line.find_first_not_of(' ', split);
         if(next == std::string::npos) {
            source_line.clear();
            break;
         }
         source_line.erase(0, next);
      }
      if(!source_line.empty()) {
         result.emplace_back(std::move(source_line));
      }
   }
   return result;
}

void show_open_game_license()
{
   const auto page_lines = output().page_content_rows(5);
   auto legal_text = ct::door_plain_markdown(ct::OPEN_GAME_LICENSE_TEXT);
   constexpr std::string_view title =
      "Open Game License and Copyright Notices";
   if(legal_text.starts_with(title)) {
      legal_text.erase(0, title.size());
      while(!legal_text.empty() &&
            (legal_text.front() == '\r' || legal_text.front() == '\n')) {
         legal_text.erase(0, 1);
      }
   }
   const auto lines = wrap_text(legal_text, output().content_columns());
   size_t offset = 0;
   while(offset < lines.size()) {
      od_clr_scr();
      output().suspend_paging();
      door_heading("Open Game License and Copyright Notices\n\r");
      door_heading("---------------------------------------\n\r");
      const auto end = std::min(offset + page_lines, lines.size());
      for(; offset < end; ++offset) {
         od_printf("%s\n\r", lines[offset].c_str());
      }
      if(offset < lines.size()) {
         door_option_prompt({"[Space] Next page", "[Q] Return"});
         while(true) {
            const auto key = od_get_key(TRUE);
            if(key == 'q' || key == 'Q') {
               return;
            }
            if(key == ' ') {
               break;
            }
         }
      } else {
         door_prompt("\n\rPress any key to return.\n\r");
         od_get_key(TRUE);
      }
   }
}

void render_startup_notice()
{
   od_clr_scr();
   door_heading("\n\r                 Cepheus Trader\n\r");
   if(ct::door_profile_uses_cp437(output().profile())) {
      door_accent(
         "       \u2500\u2500 An Alternate Cepheus Engine Universe "
         "\u2500\u2500\n\r\n\r");
   } else {
      door_accent("          An Alternate Cepheus Engine Universe\n\r\n\r");
   }
   door_printf(
      "This product contains Open Game Content used under the Open Game "
      "License version 1.0a.\n\r\n\r");
   door_printf(
      "Cepheus Engine and Samardan Press are trademarks of Jason \"Flynn\" "
      "Kemp. Cepheus Trader is not affiliated with Jason \"Flynn\" Kemp or "
      "Samardan Press.\n\r\n\r");
}

bool await_startup_choice()
{
   const HelpScope help_scope(ct::DoorHelpTopic::General);
   render_startup_notice();
   door_option_prompt({"[Enter] Continue", "[L] License", "[Q] Exit", "[?] Help"}, false);
   while(true) {
      const auto key = door_get_key(TRUE);
      if(key == 'q' || key == 'Q') {
         return false;
      }
      if(key == 'l' || key == 'L') {
         show_open_game_license();
         render_startup_notice();
         door_option_prompt(
            {"[Enter] Continue", "[L] License", "[Q] Exit", "[?] Help"}, false);
      } else if(key == '\r' || key == '\n') {
         return true;
      }
   }
}

void initialize_opendoors(const int argc, char** argv)
{
   od_control.od_nocopyright = TRUE;
   od_control.od_status_on = FALSE;
   od_control.od_config_file = INCLUDE_CONFIG_FILE;
   od_control.od_config_function = opendoors_config_line;
   std::snprintf(
      od_control.od_prog_name,
      sizeof(od_control.od_prog_name),
      "%s",
      "Cepheus Trader");
   std::snprintf(
      od_control.od_prog_version,
      sizeof(od_control.od_prog_version),
      "%s",
      CT_PRODUCT_VERSION);
#ifdef _WIN32
   (void)argc;
   (void)argv;
   od_parse_cmd_line(GetCommandLineA());
#else
   od_parse_cmd_line(argc, argv);
#endif
   od_init();
   if(!opendoors_configuration_error.empty()) {
      throw std::runtime_error(opendoors_configuration_error);
   }
   od_set_statusline(STATUS_NONE);
   od_control.od_status_on = FALSE;
}

void initialize_presentation(const ct::BbsConfig& config)
{
   const auto selected_profile =
      opendoors_profile || config.terminal_profile != "auto"
      ? ct::parse_door_profile(
           opendoors_profile.value_or(config.terminal_profile))
      : ct::door_profile_for_capabilities(
           od_control.user_ansi != FALSE,
           od_control.user_8bit != FALSE);
   od_control.user_ansi =
      ct::door_profile_uses_ansi(selected_profile) ? TRUE : FALSE;
   od_control.user_8bit =
      ct::door_profile_uses_cp437(selected_profile) ? TRUE : FALSE;
   const auto reported_columns =
      od_control.user_screenwidth >= 40
      ? static_cast<size_t>(od_control.user_screenwidth)
      : size_t{80};
   const auto reported_rows =
      od_control.user_screen_length >= 24
      ? static_cast<size_t>(od_control.user_screen_length)
      : size_t{24};
   presentation = std::make_unique<ct::DoorPresentation>(
                     selected_profile,
                     opendoors_columns.value_or(config.terminal_columns) == 0
                        ? reported_columns
                        : opendoors_columns.value_or(config.terminal_columns),
                     opendoors_rows.value_or(config.terminal_rows) == 0
                        ? reported_rows
                        : opendoors_rows.value_or(config.terminal_rows),
   [](const std::string_view bytes) {
      const std::string terminated(bytes);
      od_disp_emu(terminated.c_str(), TRUE);
   });
   presentation->configure_paging(1, [] {
      constexpr std::string_view prompt =
         "[Enter/Space] Continue  [C] Continuous";
      output().write(prompt, ct::DoorTextRole::Prompt);
      while(true) {
         const auto key = od_get_key(TRUE);
         if(key == '\r' || key == '\n' || key == ' ') {
            break;
         }
         if(key == 'c' || key == 'C') {
            output().suppress_paging_until_input();
            break;
         }
      }
      output().erase_prompt(prompt.size());
   });
}

void render_hello(const ct::ServerHello& hello, const ct::TlsConnection& connection)
{
   od_clr_scr();
   door_heading("\n\r  Cepheus Trader\n\r");
   door_heading("  ==============\n\r\n\r");
   (void)connection;
   door_success("  Secure communications link established.\n\r");
   door_label("  Ship status: ");
   door_identifier("%s\n\r", phase_name(hello.phase));
   if(hello.phase == ct::PlayerPhase::NewUser) {
      door_information(
         "\n\r  No captain is registered for this local account.\n\r");
   }
}

std::array<uint8_t, 16> random_command_id(ct::CommandIdGenerator& random)
{
   return random.next();
}

void door_input_str(char* input,
                    const INT maximum,
                    const unsigned char minimum_character,
                    const unsigned char maximum_character)
{
   ::od_input_str(
      input, maximum, minimum_character, maximum_character);
   output().reset_after_external_input();
   output().resume_paging();
   active_prompt.clear();
   active_prompt_on_current_line = false;
}

#define od_input_str(...) door_input_str(__VA_ARGS__)

std::optional<std::string> input_text(const char* prompt,
                                      const std::string& default_value,
                                      const size_t maximum = 128)
{
   while(true) {
      door_prompt(
         "%s [%s] (Q to cancel, ? for help): ",
         prompt,
         safe_field(default_value).c_str());
      std::vector<char> input(maximum + 1, '\0');
      od_input_str(input.data(), static_cast<INT>(maximum), 32, 255);
      if(input[0] == '\0') {
         return default_value;
      }
      if(input[0] == '?' && input[1] == '\0') {
         show_context_help();
         door_printf("\n\r");
         continue;
      }
      if((input[0] == 'q' || input[0] == 'Q') && input[1] == '\0') {
         return std::nullopt;
      }
      return input.data();
   }
}

std::optional<unsigned> input_number(const char* prompt,
                                     const unsigned minimum,
                                     const unsigned maximum,
                                     const std::optional<unsigned> default_value = {})
{
   while(true) {
      if(default_value) {
         door_prompt(
            "%s [%u] (Q to cancel, ? for help): ", prompt, *default_value);
      } else {
         door_prompt("%s (Q to cancel, ? for help): ", prompt);
      }
      std::array<char, 16> input{};
      od_input_str(input.data(), static_cast<INT>(input.size() - 1), 32, 255);
      if(input[0] == '\0' && default_value) {
         return default_value;
      }
      if(input[0] == '?' && input[1] == '\0') {
         show_context_help();
         door_printf("\n\r");
         continue;
      }
      if((input[0] == 'q' || input[0] == 'Q') && input[1] == '\0') {
         return std::nullopt;
      }
      unsigned value = 0;
      const auto [end, error] =
         std::from_chars(input.data(), input.data() + std::strlen(input.data()), value);
      if(error == std::errc() && end == input.data() + std::strlen(input.data()) &&
            value >= minimum && value <= maximum) {
         return value;
      }
      door_error("Enter a number from %u through %u.\n\r", minimum, maximum);
   }
}

std::optional<uint64_t> input_credit_amount(const char* prompt, const uint64_t maximum)
{
   while(true) {
      door_prompt("%s (maximum Cr%llu; Q to cancel, ? for help): ",
                  prompt, static_cast<unsigned long long>(maximum));
      std::array<char, 32> input{};
      od_input_str(input.data(), static_cast<INT>(input.size() - 1), 32, 255);
      if(input[0] == '?' && input[1] == '\0') {
         show_context_help();
         door_printf("\n\r");
         continue;
      }
      if((input[0] == 'q' || input[0] == 'Q') && input[1] == '\0') {
         return std::nullopt;
      }
      uint64_t value = 0;
      const auto length = std::strlen(input.data());
      const auto [end, error] =
         std::from_chars(input.data(), input.data() + length, value);
      if(error == std::errc() && end == input.data() + length && value >= 1 && value <= maximum) {
         return value;
      }
      door_error("Enter a credit amount from 1 through %llu.\n\r",
                 static_cast<unsigned long long>(maximum));
   }
}

std::optional<uint64_t> input_tonnage(const char* prompt,
                                      const uint64_t maximum_millitons)
{
   while(true) {
      door_prompt(
         "%s (maximum %s t; Q to cancel, ? for help): ",
         prompt,
         ct::format_tonnage(maximum_millitons).c_str());
      std::array<char, 32> input{};
      od_input_str(input.data(), static_cast<INT>(input.size() - 1), 32, 255);
      if(input[0] == '?' && input[1] == '\0') {
         show_context_help();
         door_printf("\n\r");
         continue;
      }
      if((input[0] == 'q' || input[0] == 'Q') && input[1] == '\0') {
         return std::nullopt;
      }
      const auto quantity = ct::parse_tonnage_millitons(input.data());
      if(quantity && *quantity >= 1 && *quantity <= maximum_millitons) {
         return quantity;
      }
      door_error(
         "Enter a quantity from 0.001 through %s tonnes.\n\r",
         ct::format_tonnage(maximum_millitons).c_str());
   }
}

void print_wrapped(
   const std::string_view text,
   const char* indent = "  ",
   const ct::DoorTextRole role = ct::DoorTextRole::Information)
{
   const auto indent_width = std::strlen(indent);
   door_write(indent, role);
   output().write_hanging(safe_field(text), indent_width, role);
   door_write("\n\r", role);
}

void print_wrapped_field(
   const char* label,
   const std::string_view text,
   const ct::DoorTextRole role = ct::DoorTextRole::Information)
{
   const auto label_width = std::strlen(label);
   door_label("%s", label);
   output().write_hanging(safe_field(text), label_width, role);
   door_write("\n\r", role);
}

void show_context_help()
{
   const auto saved_prompt = active_prompt;
   const auto saved_prompt_on_current_line = active_prompt_on_current_line;
   const auto& help = ct::door_help(active_help_topic);
   active_prompt.clear();
   active_prompt_on_current_line = false;
   output().resume_paging();
   door_write("\n\r\n\r", ct::DoorTextRole::Normal);
   door_heading("Help - %s\n\r", safe_field(help.title).c_str());
   door_heading("%s\n\r\n\r", std::string(help.title.size() + 7, '=').c_str());
   door_write(help.body, ct::DoorTextRole::Information);
   door_write("\n\r", ct::DoorTextRole::Information);
   active_prompt.clear();
   active_prompt_on_current_line = false;
   door_prompt("\n\r[Enter] Resume\n\r");
   while(true) {
      const auto key = ::od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         output().reset_paging();
         output().resume_paging();
         echo_prompt_key(key, false);
         break;
      }
   }
   active_prompt.clear();
   active_prompt_on_current_line = false;
   active_prompt = saved_prompt;
   if(!saved_prompt.empty()) {
      output().write(saved_prompt, ct::DoorTextRole::Prompt);
      active_prompt_on_current_line = saved_prompt_on_current_line;
   }
   output().suspend_paging();
}

int door_get_key(const BOOL wait)
{
   while(true) {
      const auto key = ::od_get_key(wait);
      if(key != 0) {
         output().reset_paging();
         output().resume_paging();
      }
      if(key == '?') {
         echo_prompt_key(key, true);
         show_context_help();
         continue;
      }
      if(key != 0) {
         echo_prompt_key(key, false);
      }
      return key;
   }
}

#define od_get_key(wait) door_get_key(wait)

const char* career_name(const ct::Career career)
{
   switch(career) {
   case ct::Career::Trader:
      return "Trader";
   case ct::Career::Privateer:
      return "Privateer";
   case ct::Career::Navy:
      return "Navy";
   }
   return "Unknown";
}

int characteristic_modifier(const uint8_t score)
{
   return static_cast<int>(score / 3) - 2;
}

int characteristic_cost(const uint8_t score,
                        const ct::CharacteristicPointBuy& point_buy)
{
   return static_cast<int>(score) - static_cast<int>(point_buy.neutral);
}

int characteristic_total_cost(const ct::Characteristics& characteristics,
                              const ct::CharacteristicPointBuy& point_buy)
{
   return characteristic_cost(characteristics.strength, point_buy) +
          characteristic_cost(characteristics.dexterity, point_buy) +
          characteristic_cost(characteristics.endurance, point_buy) +
          characteristic_cost(characteristics.intelligence, point_buy) +
          characteristic_cost(characteristics.education, point_buy) +
          characteristic_cost(characteristics.charisma, point_buy);
}

void edit_characteristics(ct::PersonDraft& person,
                          const ct::CharacteristicPointBuy& point_buy)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Characteristics);
   const auto original = person.characteristics;
   const std::array<const char*, 6> names{"STR", "DEX", "END", "INT", "EDU", "CHA"};
   while(true) {
      std::array<uint8_t*, 6> scores{
         &person.characteristics.strength,
         &person.characteristics.dexterity,
         &person.characteristics.endurance,
         &person.characteristics.intelligence,
         &person.characteristics.education,
         &person.characteristics.charisma,
      };
      od_clr_scr();
      door_heading("Captain Characteristics\n\r");
      door_heading("=======================\n\r\n\r");
      for(size_t index = 0; index < scores.size(); ++index) {
         const auto modifier = characteristic_modifier(*scores[index]);
         const auto cost = characteristic_cost(*scores[index], point_buy);
         door_number("%u. ", static_cast<unsigned>(index + 1));
         door_label("%s ", names[index]);
         door_number("%2u", static_cast<unsigned>(*scores[index]));
         door_label("  DM ");
         if(modifier < 0) {
            door_warning("%+d", modifier);
         } else if(modifier > 0) {
            door_success("%+d", modifier);
         } else {
            door_value("%+d", modifier);
         }
         door_label("  Cost ");
         if(cost > 0) {
            door_warning("%+d", cost);
         } else if(cost < 0) {
            door_success("%+d", cost);
         } else {
            door_value("%+d", cost);
         }
         od_printf("\n\r");
      }
      const auto spent = characteristic_total_cost(person.characteristics, point_buy);
      const auto remaining = static_cast<int>(point_buy.budget) - spent;
      od_printf("\n\r");
      door_label("Budget ");
      door_number("%d", static_cast<int>(point_buy.budget));
      door_label("  Spent ");
      door_number("%d", spent);
      door_label("  Remaining ");
      if(remaining == 0) {
         door_success("%+d\n\r", remaining);
      } else if(remaining < 0) {
         door_warning("%+d\n\r", remaining);
      } else {
         door_number("%+d\n\r", remaining);
      }
      door_option_prompt({
         "[1-6] Change score",
         "[R] Restore defaults",
         "[Enter] Finish",
         "[Q] Cancel",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key >= '1' && key <= '6') {
         const auto index = static_cast<size_t>(key - '1');
         const auto score = input_number(
                               "New score",
                               point_buy.minimum,
                               point_buy.maximum,
                               *scores[index]);
         if(score) {
            *scores[index] = static_cast<uint8_t>(*score);
         }
      } else if(key == 'r' || key == 'R') {
         person.characteristics = original;
      } else if(key == 'q' || key == 'Q') {
         person.characteristics = original;
         return;
      } else if(key == '\r' || key == '\n') {
         if(remaining == 0) {
            return;
         }
         door_error(
            "Spend exactly %d points before finishing. Press any key.\n\r",
            static_cast<int>(point_buy.budget));
         od_get_key(TRUE);
      }
   }
}

const char* canonical_skill_name(const ct::SkillId skill)
{
   switch(skill) {
   case ct::SkillId::Admin:
      return "Admin";
   case ct::SkillId::Advocate:
      return "Advocate";
   case ct::SkillId::Astrogation:
      return "Astrogation";
   case ct::SkillId::Broker:
      return "Broker";
   case ct::SkillId::Carouse:
      return "Carouse";
   case ct::SkillId::Communications:
      return "Communications";
   case ct::SkillId::Computer:
      return "Computer";
   case ct::SkillId::Electronics:
      return "Electronics";
   case ct::SkillId::EngineerJump:
      return "Engineer (Jump Drive)";
   case ct::SkillId::EngineerManeuver:
      return "Engineer (Maneuver Drive)";
   case ct::SkillId::EngineerPower:
      return "Engineer (Power)";
   case ct::SkillId::EngineerLifeSupport:
      return "Engineer (Life Support)";
   case ct::SkillId::Etiquette:
      return "Etiquette";
   case ct::SkillId::GunCombat:
      return "Gun Combat";
   case ct::SkillId::GunnerTurrets:
      return "Gunner (Turrets)";
   case ct::SkillId::GunnerCapital:
      return "Gunner (Capital Weapons)";
   case ct::SkillId::GunnerScreens:
      return "Gunner (Screens)";
   case ct::SkillId::Investigate:
      return "Investigate";
   case ct::SkillId::JackOfAllTrades:
      return "Jack of All Trades";
   case ct::SkillId::Leadership:
      return "Leadership";
   case ct::SkillId::Mechanic:
      return "Mechanic";
   case ct::SkillId::Medicine:
      return "Medicine";
   case ct::SkillId::Melee:
      return "Melee";
   case ct::SkillId::Persuade:
      return "Persuade";
   case ct::SkillId::PilotSpacecraft:
      return "Pilot (Spacecraft)";
   case ct::SkillId::PilotSmallCraft:
      return "Pilot (Small Craft)";
   case ct::SkillId::Recon:
      return "Recon";
   case ct::SkillId::Stealth:
      return "Stealth";
   case ct::SkillId::Streetwise:
      return "Streetwise";
   case ct::SkillId::TacticsMilitary:
      return "Tactics (Military)";
   case ct::SkillId::TacticsNaval:
      return "Tactics (Naval)";
   case ct::SkillId::TradeCargomaster:
      return "Trade (Cargomaster)";
   case ct::SkillId::VaccSuit:
      return "Vacc Suit";
   }
   return "Unknown";
}

std::string skill_name(
   const ct::SkillId skill,
   const std::vector<ct::SkillDefinition>& definitions)
{
   const auto definition = std::find_if(
   definitions.begin(), definitions.end(), [skill](const auto & candidate) {
      return candidate.id == skill;
   });
   return definition == definitions.end()
          ? std::string(canonical_skill_name(skill))
          : safe_field(definition->name);
}

uint16_t required_training_weeks(
   const ct::PersonDraft& person,
   const ct::SkillId target)
{
   const auto rating = std::find_if(
   person.skills.begin(), person.skills.end(), [target](const auto & candidate) {
      return candidate.skill == target;
   });
   if(rating == person.skills.end() ||
         target == ct::SkillId::JackOfAllTrades ||
         rating->level < 0) {
      throw std::runtime_error("person has an invalid training target");
   }
   unsigned skill_total = 0;
   for(const auto& skill : person.skills) {
      if(skill.level > 0) {
         skill_total += static_cast<unsigned>(skill.level);
      }
   }
   const auto required =
      skill_total + static_cast<unsigned>(rating->level) + 1;
   if(required > UINT16_MAX) {
      throw std::runtime_error("training duration exceeds UInt16");
   }
   return static_cast<uint16_t>(required);
}

void reset_training_target(
   ct::PersonDraft& person,
   const ct::SkillId target)
{
   person.training = ct::SkillTraining{
      .skill = target,
      .needed_weeks = required_training_weeks(person, target),
      .current_weeks = 0,
   };
}

void normalize_training_target(ct::PersonDraft& person)
{
   const auto current = std::find_if(
                           person.skills.begin(),
                           person.skills.end(),
   [&person](const auto & rating) {
      return rating.skill == person.training.skill &&
             rating.skill != ct::SkillId::JackOfAllTrades;
   });
   if(current != person.skills.end()) {
      reset_training_target(person, current->skill);
      return;
   }
   const auto replacement = std::find_if(
   person.skills.begin(), person.skills.end(), [](const auto & rating) {
      return rating.skill != ct::SkillId::JackOfAllTrades;
   });
   if(replacement == person.skills.end()) {
      throw std::runtime_error("person has no trainable skill");
   }
   reset_training_target(person, replacement->skill);
}

void render_person(const ct::PersonDraft& person,
                   const std::vector<ct::SkillDefinition>& definitions)
{
   const auto& c = person.characteristics;
   const auto characteristic = [](const char* label, const uint8_t value) {
      door_label("%s ", label);
      door_number("%u", static_cast<unsigned>(value));
   };
   if(output().columns() < 48) {
      od_printf("  ");
      characteristic("STR", c.strength);
      od_printf("  ");
      characteristic("DEX", c.dexterity);
      od_printf("  ");
      characteristic("END", c.endurance);
      od_printf("\n\r  ");
      characteristic("INT", c.intelligence);
      od_printf("  ");
      characteristic("EDU", c.education);
      od_printf("  ");
      characteristic("CHA", c.charisma);
      od_printf("\n\r\n\r");
   } else {
      od_printf("  ");
      characteristic("STR", c.strength);
      od_printf("  ");
      characteristic("DEX", c.dexterity);
      od_printf("  ");
      characteristic("END", c.endurance);
      od_printf("  ");
      characteristic("INT", c.intelligence);
      od_printf("  ");
      characteristic("EDU", c.education);
      od_printf("  ");
      characteristic("CHA", c.charisma);
      od_printf("\n\r\n\r");
   }
   for(const auto& rating : person.skills) {
      const auto name = skill_name(rating.skill, definitions);
      door_value("  %-29s ", name.c_str());
      door_number("%d\n\r", static_cast<int>(rating.level));
   }
   const auto target = std::find_if(
                          person.skills.begin(),
                          person.skills.end(),
   [&person](const auto & rating) {
      return rating.skill == person.training.skill;
   });
   od_printf("\n\r");
   door_label("  Training: ");
   door_identifier(
      "%s", skill_name(person.training.skill, definitions).c_str());
   if(target != person.skills.end()) {
      door_label(" ");
      door_number("%d", static_cast<int>(target->level));
      door_label(" -> ");
      door_number("%d", static_cast<int>(target->level) + 1);
   }
   od_printf("\n\r");
   door_label("  Progress: ");
   door_number("%u", person.training.current_weeks);
   door_label(" / ");
   door_number("%u", person.training.needed_weeks);
   door_label(" training weeks\n\r");
}

void edit_training_target(
   ct::PersonDraft& person,
   const std::vector<ct::SkillDefinition>& definitions)
{
   std::vector<const ct::SkillRating*> trainable;
   for(const auto& rating : person.skills) {
      if(rating.skill != ct::SkillId::JackOfAllTrades) {
         trainable.push_back(&rating);
      }
   }
   while(true) {
      od_clr_scr();
      door_heading("Select Training Target\n\r");
      door_heading("======================\n\r\n\r");
      for(size_t index = 0; index < trainable.size(); ++index) {
         const auto& rating = *trainable[index];
         door_number("%2u. ", static_cast<unsigned>(index + 1));
         door_value("%-29s", skill_name(rating.skill, definitions).c_str());
         door_number("%d", static_cast<int>(rating.level));
         door_label(" -> ");
         door_number("%d", static_cast<int>(rating.level) + 1);
         const auto weeks = required_training_weeks(person, rating.skill);
         door_label("  ");
         door_number("%u", weeks);
         door_label(" weeks\n\r");
      }
      const auto selected = input_number(
                               "Skill number", 1, static_cast<unsigned>(trainable.size()));
      if(!selected) {
         return;
      }
      reset_training_target(person, trainable[*selected - 1]->skill);
      return;
   }
}

void edit_skill_slots(ct::PersonDraft& person,
                      const std::vector<ct::SkillDefinition>& definitions)
{
   const auto original_skills = person.skills;
   std::vector<int8_t> levels;
   levels.reserve(person.skills.size());
   for(const auto& rating : person.skills) {
      levels.push_back(rating.level);
   }
   person.skills.clear();
   person.skills.reserve(levels.size());

   for(size_t slot = 0; slot < levels.size(); ++slot) {
      std::vector<const ct::SkillDefinition*> available;
      available.reserve(definitions.size());
      for(const auto& definition : definitions) {
         if(definition.id != ct::SkillId::JackOfAllTrades ||
               (levels[slot] >= 1 && levels[slot] <= 2)) {
            available.push_back(&definition);
         }
      }
      while(true) {
         od_clr_scr();
         door_heading("Select skill %u of %u (rating %+d)\n\r",
                      static_cast<unsigned>(slot + 1),
                      static_cast<unsigned>(levels.size()),
                      static_cast<int>(levels[slot]));
         door_heading("--------------------------------------\n\r");
         for(size_t index = 0; index < available.size(); ++index) {
            const bool selected = std::any_of(
                                     person.skills.begin(),
                                     person.skills.end(),
            [&available, index](const auto & rating) {
               return rating.skill == available[index]->id;
            });
            const bool paired = output().columns() >= 72;
            door_number("%2u ", static_cast<unsigned>(index + 1));
            door_value(
               "%-24s ", safe_field(available[index]->name).c_str());
            if(selected) {
               door_identifier("%-6s", "USED");
            } else {
               od_printf("%-6s", "");
            }
            od_printf("%s", !paired || index % 2 != 0 ? "\n\r" : "");
         }
         if(output().columns() >= 72 && available.size() % 2 != 0) {
            od_printf("\n\r");
         }
         const auto selected = input_number(
                                  "Skill number", 1, static_cast<unsigned>(available.size()));
         if(!selected) {
            person.skills = original_skills;
            return;
         }
         const auto skill = available[*selected - 1]->id;
         const bool duplicate = std::any_of(
                                   person.skills.begin(),
                                   person.skills.end(),
         [skill](const auto & rating) {
            return rating.skill == skill;
         });
         if(!duplicate) {
            person.skills.push_back(ct::SkillRating{
               .skill = skill,
               .level = levels[slot],
            });
            break;
         }
         door_error("That skill is already selected. Press any key.\n\r");
         od_get_key(TRUE);
      }
   }
}

bool edit_person(ct::PersonDraft& person,
                 const std::vector<ct::SkillDefinition>& definitions,
                 const std::string& heading,
                 const ct::CharacteristicPointBuy* characteristic_point_buy = nullptr)
{
   const auto original = person;
   const auto display_heading = safe_field(heading);
   od_clr_scr();
   door_heading("%s\n\r", display_heading.c_str());
   door_heading(
      "%s\n\r\n\r", std::string(display_heading.size(), '=').c_str());
   if(const auto name = input_text("Name", person.name)) {
      person.name = *name;
   } else {
      return false;
   }
   while(true) {
      od_clr_scr();
      door_heading("%s: ", display_heading.c_str());
      door_accent("%s\n\r\n\r", safe_field(person.name).c_str());
      render_person(person, definitions);
      if(characteristic_point_buy != nullptr) {
         const auto spent =
            characteristic_total_cost(person.characteristics, *characteristic_point_buy);
         od_printf("\n\r");
         door_label("Characteristic budget: ");
         if(spent == static_cast<int>(characteristic_point_buy->budget)) {
            door_success("%d", spent);
         } else {
            door_number("%d", spent);
         }
         door_label(" / ");
         door_number(
            "%d\n\r", static_cast<int>(characteristic_point_buy->budget));
         door_option_prompt({
            "[Enter] Accept",
            "[C] Characteristics",
            "[E] Skills",
            "[T] Training",
            "[Q] Cancel",
            "[?] Help",
         }, false);
      } else {
         door_option_prompt({
            "[Enter] Accept",
            "[E] Skill selections",
            "[T] Training",
            "[Q] Cancel",
            "[?] Help",
         });
      }
      const auto key = od_get_key(TRUE);
      if(key == 'e' || key == 'E') {
         edit_skill_slots(person, definitions);
         normalize_training_target(person);
      } else if((key == 'c' || key == 'C') &&
                characteristic_point_buy != nullptr) {
         edit_characteristics(person, *characteristic_point_buy);
      } else if(key == 't' || key == 'T') {
         edit_training_target(person, definitions);
      } else if(key == '\r' || key == '\n') {
         return true;
      } else if(key == 'q' || key == 'Q') {
         person = original;
         return false;
      }
   }
}

void render_offer_comparison(const ct::StartingShipOffers& offers)
{
   od_clr_scr();
   door_heading("Starting Ship Offers\n\r");
   door_heading("--------------------\n\r");
   door_label("Origin: ");
   door_value("%s", safe_field(offers.origin.polity_name).c_str());
   door_label(", ");
   door_value("%s", safe_field(offers.origin.home_system_name).c_str());
   door_label(" / ");
   door_value("%s\n\r", safe_field(offers.origin.home_world_name).c_str());
   door_label("Polity axes: trade/combat ");
   door_number("%u", offers.origin.trade_combat);
   door_label(", chaos/order ");
   door_number("%u\n\r\n\r", offers.origin.chaos_order);
   for(size_t index = 0; index < offers.offers.size(); ++index) {
      const auto& offer = offers.offers[index];
      if(output().columns() < 64) {
         door_number("%u. ", static_cast<unsigned>(index + 1));
         door_identifier("%s", career_name(offer.career));
         od_printf(" - ");
         door_value("%s\n\r", safe_field(offer.ship_name).c_str());
         od_printf("   ");
         door_number("%u", offer.displacement_tons);
         door_label(" tons  J-");
         door_number("%u", offer.jump_rating);
         door_label("  ");
         door_number("%u", offer.thrust_g);
         door_label("G\n\r");
      } else {
         door_number("%u. ", static_cast<unsigned>(index + 1));
         door_identifier("%-10s ", career_name(offer.career));
         door_value("%-15s ", safe_field(offer.ship_name).c_str());
         door_number("%u", offer.displacement_tons);
         door_label(" tons  J-");
         door_number("%u", offer.jump_rating);
         door_label("  ");
         door_number("%u", offer.thrust_g);
         door_label("G\n\r");
      }
      door_label("   Cargo ");
      door_number("%.1f", offer.cargo_tons);
      door_label(" tons  Crew ");
      door_number("%u", offer.crew_count);
      door_label("  ");
      door_identifier("%s\n\r", safe_field(offer.package_name).c_str());
   }
}

void render_ship_detail(const ct::StartingShipOptions& options)
{
   od_clr_scr();
   const auto& ship = options.offer;
   door_heading("%s", safe_field(ship.ship_name).c_str());
   door_label(" - ");
   door_identifier("%s", career_name(ship.career));
   door_label(" / ");
   door_value("%s\n\r", safe_field(ship.role).c_str());
   if(output().columns() < 56) {
      door_number("%u", ship.displacement_tons);
      door_label(" tons  J-");
      door_number("%u", ship.jump_rating);
      door_label("  ");
      door_number("%u", ship.thrust_g);
      door_label("G\n\rCargo ");
      door_number("%.1f", ship.cargo_tons);
      door_label(" tons  Crew ");
      door_number("%u\n\r\n\r", ship.crew_count);
   } else {
      door_number("%u", ship.displacement_tons);
      door_label(" tons  J-");
      door_number("%u", ship.jump_rating);
      door_label("  ");
      door_number("%u", ship.thrust_g);
      door_label("G  Cargo ");
      door_number("%.1f", ship.cargo_tons);
      door_label(" tons  Crew ");
      door_number("%u\n\r\n\r", ship.crew_count);
   }
   for(const auto& paragraph : options.description_paragraphs) {
      print_wrapped(paragraph);
      od_printf("\n\r");
   }
   door_label("Package: ");
   door_identifier("%s\n\r", safe_field(ship.package_name).c_str());
   print_wrapped(ship.rationale);
   od_printf("\n\r");
   door_label("Starting reserve: ");
   door_number("Cr%llu liquid", static_cast<unsigned long long>(options.terms.liquid_reserve_credits));
   if(options.terms.restricted_reserve_credits != 0) {
      door_label(" + Cr");
      door_number("%llu restricted",
                  static_cast<unsigned long long>(options.terms.restricted_reserve_credits));
   }
   od_printf("\n\r");
   if(options.terms.principal_credits != 0) {
      door_label("Secured principal: ");
      door_number("Cr%llu", static_cast<unsigned long long>(options.terms.principal_credits));
      door_label("; monthly payment Cr");
      door_number("%llu\n\r", static_cast<unsigned long long>(options.terms.monthly_payment_credits));
   }
   print_wrapped_field("Authority: ", options.terms.authority);
   print_wrapped_field("Exit terms: ", options.terms.exit_terms);
}

char crew_menu_key(const size_t index)
{
   if(index >= 26) {
      throw std::runtime_error(
         "starting crew roster exceeds the supported 26 named roles");
   }
   return static_cast<char>('A' + index);
}

void render_crew_roster(
   const ct::StartingCrewPlan& plan,
   const std::vector<ct::InitialCrewDraft>& drafts,
   const size_t page)
{
   const bool compact = output().columns() < 68;
   const size_t rows_per_entry = compact ? 2 : 1;
   const size_t reserved_rows = compact ? 10 : 8;
   const size_t available_rows =
      output().rows() > reserved_rows ? output().rows() - reserved_rows : 1;
   const size_t page_size =
      std::max<size_t>(1, available_rows / rows_per_entry);
   const size_t first = page * page_size;
   const size_t last = std::min(first + page_size, plan.slots.size());
   const size_t page_count =
      std::max<size_t>(1, (plan.slots.size() + page_size - 1) / page_size);

   od_clr_scr();
   door_heading("Starting Crew Roster\n\r");
   door_heading("====================\n\r");
   print_wrapped(
      "Each row names one crew leader or senior specialist. Other positions "
      "shown are supporting personnel. Select a letter to inspect or rename; "
      "Enter accepts all default callsigns.",
      "",
      ct::DoorTextRole::Information);
   if(compact && page_count > 1) {
      door_label("Page ");
      door_number("%u", static_cast<unsigned>(page + 1));
      door_label(" of ");
      door_number("%u\n\r", static_cast<unsigned>(page_count));
   }
   od_printf("\n\r");

   for(size_t index = first; index < last; ++index) {
      const auto& slot = plan.slots[index];
      const auto naming =
         ct::describe_crew_naming(slot.role_kind, slot.role, slot.represented_positions);
      door_number("%c", crew_menu_key(index));
      door_label(". ");
      if(compact) {
         door_identifier("%s\n\r", safe_field(naming.appointment).c_str());
         door_value("   %s", safe_field(drafts[index].name).c_str());
         door_label(" - ");
         door_number("%s", safe_field(naming.assignment).c_str());
         if(!slot.required) {
            door_label(" (optional)");
         }
         od_printf("\n\r");
      } else {
         door_identifier(
            "%-31s", safe_field(naming.appointment).c_str());
         door_value("%-15s", safe_field(drafts[index].name).c_str());
         door_number("%s", safe_field(naming.assignment).c_str());
         if(!slot.required) {
            door_label(" (optional)");
         }
         od_printf("\n\r");
      }
   }

   if(page_count > 1) {
      door_option_prompt({
         "[Letter] Inspect/rename",
         "[< >] Page",
         "[Enter] Accept",
         "[0] Back",
      });
   } else {
      door_option_prompt({
         "[Letter] Inspect/rename",
         "[Enter] Accept",
         "[0] Back",
      });
   }
}

void edit_crew_member(
   const ct::StartingCrewSlot& slot,
   ct::InitialCrewDraft& draft,
   const std::vector<ct::SkillDefinition>& definitions,
   const char menu_key)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Crew);
   const auto naming =
      ct::describe_crew_naming(slot.role_kind, slot.role, slot.represented_positions);
   while(true) {
      auto person = slot.default_crew;
      reset_training_target(person, draft.training_skill);
      od_clr_scr();
      door_heading(
         "%c. %s\n\r",
         menu_key,
         safe_field(naming.appointment).c_str());
      door_heading(
         "%s\n\r\n\r",
         std::string(naming.appointment.size() + 3, '=').c_str());
      door_label("Role:       ");
      door_identifier("%s\n\r", safe_field(naming.role_name).c_str());
      door_label("Assignment: ");
      door_number("%s\n\r", safe_field(naming.assignment).c_str());
      door_label("Name:       ");
      door_value("%s\n\r\n\r", safe_field(draft.name).c_str());
      print_wrapped(naming.explanation, "", ct::DoorTextRole::Information);
      od_printf("\n\r");
      render_person(person, definitions);
      door_option_prompt({
         "[Enter] Roster",
         "[N] Rename",
         "[T] Training target",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         return;
      }
      if(key == 'n' || key == 'N') {
         if(const auto name = input_text(naming.prompt.c_str(), draft.name)) {
            draft.name = *name;
         }
      } else if(key == 't' || key == 'T') {
         edit_training_target(person, definitions);
         draft.training_skill = person.training.skill;
      }
   }
}

std::optional<std::vector<ct::InitialCrewDraft>> edit_crew_roster(
   const ct::StartingCrewPlan& plan,
   const std::vector<ct::SkillDefinition>& definitions,
   std::vector<ct::InitialCrewDraft> drafts = {})
{
   const HelpScope help_scope(ct::DoorHelpTopic::Crew);
   if(plan.slots.size() > 26) {
      throw std::runtime_error(
         "server returned more than 26 named starting crew roles");
   }
   if(drafts.empty()) {
      drafts.reserve(plan.slots.size());
      for(const auto& slot : plan.slots) {
         drafts.push_back(ct::InitialCrewDraft{
            .slot_id = slot.slot_id,
            .name = slot.default_crew.name,
            .training_skill = slot.default_crew.training.skill,
         });
      }
   } else if(drafts.size() != plan.slots.size()) {
      throw std::runtime_error("starting crew draft does not match its plan");
   }

   const bool compact = output().columns() < 68;
   const size_t reserved_rows = compact ? 10 : 8;
   const size_t rows_per_entry = compact ? 2 : 1;
   const size_t available_rows =
      output().rows() > reserved_rows ? output().rows() - reserved_rows : 1;
   const size_t page_size =
      std::max<size_t>(1, available_rows / rows_per_entry);
   const size_t page_count =
      std::max<size_t>(1, (plan.slots.size() + page_size - 1) / page_size);
   size_t page = 0;

   while(true) {
      render_crew_roster(plan, drafts, page);
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         return drafts;
      }
      if(key == '0') {
         return std::nullopt;
      }
      if(key == '>' && page + 1 < page_count) {
         ++page;
         continue;
      }
      if(key == '<' && page > 0) {
         --page;
         continue;
      }
      const auto normalized =
         static_cast<char>(std::toupper(static_cast<unsigned char>(key)));
      if(normalized >= 'A' && normalized <= 'Z') {
         const auto index = static_cast<size_t>(normalized - 'A');
         if(index < plan.slots.size()) {
            edit_crew_member(
               plan.slots[index],
               drafts[index],
               definitions,
               crew_menu_key(index));
            page = index / page_size;
         }
      }
   }
}

bool run_player_creation(ct::TlsConnection& connection,
                         const ct::ServerHello& hello)
{
   const HelpScope help_scope(ct::DoorHelpTopic::PlayerRegistration);
   while(true) {
      door_option_prompt({
         "[Enter] Register captain", "[Q] Return to BBS", "[?] Help"});
      const auto opening = od_get_key(TRUE);
      if(opening == '\r' || opening == '\n') {
         break;
      }
      if((opening == 'q' || opening == 'Q') && confirm_return_to_bbs()) {
         return false;
      }
   }
   ct::CommandIdGenerator random;
   uint64_t request_id = 1;
   auto captain_options = ct::get_captain_creation_options(
                             connection,
                             hello.assigned_epoch,
                             random_command_id(random),
                             request_id++);
   if(captain_options.default_captain.training.current_weeks != 0 ||
         captain_options.default_captain.training.needed_weeks !=
         required_training_weeks(
            captain_options.default_captain,
            captain_options.default_captain.training.skill)) {
      throw std::runtime_error(
         "server did not provide a valid initial training assignment; "
         "restart with the matching setup-revision-6 server");
   }
   auto captain = captain_options.default_captain;
   if(!edit_person(
            captain,
            captain_options.permitted_skills,
            "Customize Captain",
            &captain_options.characteristic_point_buy)) {
      return false;
   }

   const auto offers = ct::get_starting_ship_offers(
                          connection,
                          hello.assigned_epoch,
                          random_command_id(random),
                          request_id++);
   active_help_topic = ct::DoorHelpTopic::StartingShip;
   if(offers.setup_revision != captain_options.setup_revision ||
         offers.offers.size() != 3) {
      throw std::runtime_error("server returned inconsistent starting offers");
   }
   while(true) {
      render_offer_comparison(offers);
      const auto selection = input_number("Choose offer", 1, 3);
      if(!selection) {
         if(!edit_person(
                  captain,
                  captain_options.permitted_skills,
                  "Customize Captain",
                  &captain_options.characteristic_point_buy)) {
            return false;
         }
         continue;
      }
      const auto offer = offers.offers[*selection - 1];
      const auto ship_options = ct::get_starting_ship_options(
                                   connection,
                                   hello.assigned_epoch,
                                   offers.setup_revision,
                                   offer.offer_id,
                                   random_command_id(random),
                                   request_id++);
      const auto crew_plan = ct::get_starting_crew_plan(
                                connection,
                                hello.assigned_epoch,
                                offers.setup_revision,
                                offer.offer_id,
                                random_command_id(random),
                                request_id++);
      auto ship_name = ship_options.offer.ship_name;
      std::vector<ct::InitialCrewDraft> crew;
      std::vector<uint32_t> refit_option_ids;
      bool return_to_offers = false;
      while(!return_to_offers) {
         render_ship_detail(ship_options);
         door_option_prompt({
            "[Enter] Name ship", "[Q] Starting offers", "[?] Help"});
         while(true) {
            const auto key = od_get_key(TRUE);
            if(key == 'q' || key == 'Q') {
               return_to_offers = true;
               break;
            }
            if(key == '\r' || key == '\n') {
               break;
            }
         }
         if(return_to_offers) {
            break;
         }
         const auto entered_name = input_text("Ship name", ship_name);
         if(!entered_name) {
            continue;
         }
         ship_name = *entered_name;
         refit_option_ids.clear();
         bool refit_cancelled = false;
         for(const auto& group : ship_options.refit_groups) {
            output().resume_paging();
            door_identifier("\n\r%s\n\r", safe_field(group.name).c_str());
            for(size_t index = 0; index < group.options.size(); ++index) {
               const auto& option = group.options[index];
               door_number("%zu", index + 1);
               door_label(". ");
               door_value("%s\n\r", safe_field(option.name).c_str());
               print_wrapped(option.description, "   ");
            }
            const auto selected = input_number(
                                     "Starting fit", 1, static_cast<unsigned>(group.options.size()), 1);
            if(!selected) {
               refit_cancelled = true;
               break;
            }
            refit_option_ids.push_back(group.options[*selected - 1].option_id);
         }
         if(refit_cancelled) {
            continue;
         }
         while(true) {
            auto edited_crew = edit_crew_roster(
                                  crew_plan,
                                  captain_options.permitted_skills,
                                  crew);
            if(!edited_crew) {
               break;
            }
            crew = std::move(*edited_crew);
            ct::PlayerCreation creation{
               .setup_revision = offers.setup_revision,
               .starting_offer_id = offer.offer_id,
               .captain = captain,
               .ship_name = ship_name,
               .crew = crew,
               .refit_option_ids = refit_option_ids,
            };
            od_clr_scr();
            door_heading("Confirm New Command\n\r");
            door_heading("===================\n\r\n\r");
            door_label("Captain: ");
            door_value("%s\n\r", safe_field(creation.captain.name).c_str());
            door_label("Career:  ");
            door_identifier("%s\n\r", career_name(offer.career));
            door_label("Ship:    ");
            door_value("%s", safe_field(creation.ship_name).c_str());
            door_label(" (");
            door_identifier("%s", safe_field(offer.ship_name).c_str());
            door_label(")\n\rCrew:    ");
            door_number("%u", static_cast<unsigned>(creation.crew.size()));
            door_label(" named officers and senior specialists\n\r");
            door_prompt(
               "\n\rRegister this captain and starting estate? [Y/N]\n\r");
            const auto registration_answer = od_get_answer("YN");
            output().reset_paging();
            output().resume_paging();
            if(registration_answer != 'Y') {
               continue;
            }
            const auto created = ct::create_player(
                                    connection,
                                    hello.assigned_epoch,
                                    creation,
                                    random_command_id(random),
                                    request_id++);
            od_clr_scr();
            door_success("Command created.\n\r\n\r");
            door_information("Captain ");
            door_value("%s", safe_field(created.creation.captain.name).c_str());
            door_information(" now commands ");
            door_value("%s", safe_field(created.creation.ship_name).c_str());
            door_information(".\n\rThe ship is ");
            door_success("docked and ready to depart");
            door_information(".\n\r\n\r");
            door_prompt("Press any key to enter the command console.\n\r");
            od_get_key(TRUE);
            return true;
         }
      }
   }
}

const ct::CrewRole* find_crew_role(
   const ct::CrewManagementSnapshot& snapshot,
   const uint16_t slot_id)
{
   const auto found = std::find_if(
                         snapshot.roles.begin(),
                         snapshot.roles.end(),
   [slot_id](const auto & role) {
      return role.slot_id == slot_id;
   });
   return found == snapshot.roles.end() ? nullptr : &*found;
}

std::string crew_role_name(const ct::CrewRole& role)
{
   if(role.slot_id == 0) {
      return "Captain";
   }
   return ct::describe_crew_naming(
             role.role_kind, role.role, role.represented_positions)
          .appointment;
}

uint16_t crew_member_living_positions(const ct::CrewManagementMember& member)
{
   return member.condition == ct::PersonCondition::Dead &&
          member.represented_positions != 0
          ? static_cast<uint16_t>(member.represented_positions - 1)
          : member.represented_positions;
}

uint16_t crew_member_established_positions(
   const ct::CrewManagementSnapshot& snapshot,
   const ct::CrewManagementMember& member)
{
   const auto* role = find_crew_role(snapshot, member.slot_id);
   return role == nullptr
          ? member.represented_positions
          : std::max(role->represented_positions, member.represented_positions);
}

std::string crew_assignments(
   const ct::CrewManagementSnapshot& snapshot,
   const ct::CrewManagementMember& member,
   const bool concise = false)
{
   if(member.assigned_slot_ids.empty()) {
      return "Off watch";
   }
   std::string result;
   size_t described = 0;
   for(const auto slot_id : member.assigned_slot_ids) {
      const auto* role = find_crew_role(snapshot, slot_id);
      if(role == nullptr) {
         continue;
      }
      if(concise && described == 1) {
         result += " +" +
                   std::to_string(member.assigned_slot_ids.size() - described);
         break;
      }
      if(!result.empty()) {
         result += ", ";
      }
      result += crew_role_name(*role);
      ++described;
   }
   return result.empty() ? "Off watch" : result;
}

const char* person_condition_name(const ct::PersonCondition condition)
{
   switch(condition) {
   case ct::PersonCondition::Fit:
      return "Fit";
   case ct::PersonCondition::Fatigued:
      return "Fatigued";
   case ct::PersonCondition::Wounded:
      return "Wounded";
   case ct::PersonCondition::Incapacitated:
      return "Incapacitated";
   case ct::PersonCondition::Dead:
      return "Dead";
   }
   return "Unknown";
}

const char* crew_availability_name(const ct::CrewAvailability availability)
{
   switch(availability) {
   case ct::CrewAvailability::Active:
      return "Available";
   case ct::CrewAvailability::ShoreLeave:
      return "On shore leave";
   case ct::CrewAvailability::MedicalCare:
      return "In medical care";
   case ct::CrewAvailability::Detached:
      return "Discharged";
   case ct::CrewAvailability::AwaitingRecall:
      return "Awaiting recall";
   }
   return "Unavailable";
}

const char* crew_service_name(const ct::CrewServiceKind service)
{
   switch(service) {
   case ct::CrewServiceKind::OwnerCaptain:
      return "Owner-captain";
   case ct::CrewServiceKind::Salaried:
      return "Salaried articles";
   case ct::CrewServiceKind::PrizeShare:
      return "Prize-share articles";
   case ct::CrewServiceKind::Institutional:
      return "Service appointment";
   }
   return "Articles";
}

void render_managed_crew_roster(
   const ct::CrewManagementSnapshot& snapshot,
   const size_t page)
{
   const bool compact = output().columns() < 68;
   const size_t rows_per_entry = compact ? 2 : 1;
   const size_t reserved_rows = compact ? 9 : 8;
   const size_t available_rows =
      output().rows() > reserved_rows ? output().rows() - reserved_rows : 1;
   const size_t page_size =
      std::max<size_t>(1, available_rows / rows_per_entry);
   const size_t first = page * page_size;
   const size_t last = std::min(first + page_size, snapshot.members.size());
   const size_t page_count =
      std::max<size_t>(1, (snapshot.members.size() + page_size - 1) / page_size);

   od_clr_scr();
   door_heading("Crew Management - ");
   door_value("%s\n\r", safe_field(snapshot.ship_name).c_str());
   door_heading("=================\n\r");
   door_label("Ship status: ");
   door_identifier("%s", phase_name(snapshot.phase));
   od_printf("\n\r");
   const auto living_positions = std::accumulate(
      snapshot.members.begin(), snapshot.members.end(), uint64_t{0},
      [](const uint64_t total, const auto& member) {
         return total + crew_member_living_positions(member);
      });
   const auto established_positions = snapshot.established_complement == 0
                                      ? living_positions
                                      : uint64_t{snapshot.established_complement};
   door_label("Complement:  ");
   if(living_positions < established_positions) {
      door_warning(
         "%llu/%llu — %llu position%s short",
         static_cast<unsigned long long>(living_positions),
         static_cast<unsigned long long>(established_positions),
         static_cast<unsigned long long>(established_positions - living_positions),
         established_positions - living_positions == 1 ? "" : "s");
   } else {
      door_number(
         "%llu/%llu",
         static_cast<unsigned long long>(living_positions),
         static_cast<unsigned long long>(established_positions));
   }
   door_label(" in ");
   door_number("%zu", snapshot.members.size());
   door_label(" managed appointments\n\r\n\r");
   for(size_t index = first; index < last; ++index) {
      const auto& member = snapshot.members[index];
      auto assignment = crew_assignments(snapshot, member, true);
      const auto living = crew_member_living_positions(member);
      const auto established = crew_member_established_positions(snapshot, member);
      if(established > 1 || living != established) {
         assignment += living == established
                       ? " (" + std::to_string(living) + ")"
                       : " (" + std::to_string(living) + "/" +
                         std::to_string(established) + ")";
      }
      door_number("%c", crew_menu_key(index));
      door_label(". ");
      if(compact) {
         if(living < established) {
            door_warning("%s\n\r", safe_field(assignment).c_str());
         } else {
            door_identifier("%s\n\r", safe_field(assignment).c_str());
         }
         door_value("   %s", safe_field(member.person.name).c_str());
         door_label(" - ");
         door_identifier(
            "%s", canonical_skill_name(member.person.training.skill));
         door_label(" ");
         door_number("%u", member.person.training.current_weeks);
         door_label("/");
         door_number("%u\n\r", member.person.training.needed_weeks);
      } else {
         if(living < established) {
            door_warning("%-30s", safe_field(assignment).c_str());
         } else {
            door_identifier("%-30s", safe_field(assignment).c_str());
         }
         door_value("%-16s", safe_field(member.person.name).c_str());
         door_identifier(
            "%s", canonical_skill_name(member.person.training.skill));
         door_label(" ");
         door_number("%u", member.person.training.current_weeks);
         door_label("/");
         door_number("%u\n\r", member.person.training.needed_weeks);
      }
   }
   if(page_count > 1) {
      door_option_prompt({
         "[Letter] Crew member",
         "[< >] Page",
         "[Enter] Refresh",
         "[Q] Console",
         "[?] Help",
      });
   } else {
      door_option_prompt({
         "[Letter] Crew member",
         "[Enter] Refresh",
         "[Q] Console",
         "[?] Help",
      });
   }
}

std::optional<std::vector<uint16_t>> edit_duty_assignments(
   const ct::CrewManagementSnapshot& snapshot,
   const ct::CrewManagementMember& member)
{
   auto selected = member.assigned_slot_ids;
   const size_t available_rows =
      output().rows() > 9 ? output().rows() - 9 : 1;
   const size_t page_size = std::max<size_t>(1, available_rows);
   const size_t page_count = std::max<size_t>(
                                1, (snapshot.roles.size() + page_size - 1) / page_size);
   size_t page = 0;
   while(true) {
      od_clr_scr();
      door_heading("Duty Assignments - ");
      door_value("%s\n\r", safe_field(member.person.name).c_str());
      door_heading("================\n\r\n\r");
      const size_t first = page * page_size;
      const size_t last =
         std::min(first + page_size, snapshot.roles.size());
      for(size_t index = first; index < last; ++index) {
         const auto& role = snapshot.roles[index];
         const auto pilot_owner = std::find_if(
                                     snapshot.members.begin(),
                                     snapshot.members.end(),
         [&role, &member](const auto & candidate) {
            return candidate.person_id != member.person_id &&
                   role.role_kind == ct::CrewRoleKind::Pilot &&
                   std::find(
                      candidate.assigned_slot_ids.begin(),
                      candidate.assigned_slot_ids.end(),
                      role.slot_id) != candidate.assigned_slot_ids.end();
         });
         const bool captain_only = role.slot_id == 0 && !member.captain;
         const bool pilot_taken = pilot_owner != snapshot.members.end();
         const bool allowed = !captain_only && !pilot_taken;
         const bool assigned =
            std::find(selected.begin(), selected.end(), role.slot_id) !=
            selected.end();
         if(allowed) {
            door_number("%c", crew_menu_key(index));
         } else {
            door_label("-");
         }
         door_label(". [");
         if(assigned) {
            door_identifier("X");
         } else {
            door_label(" ");
         }
         door_label("] ");
         door_identifier("%s", safe_field(crew_role_name(role)).c_str());
         if(captain_only) {
            door_label(" (captain only)");
         } else if(pilot_taken) {
            door_label(" (held by ");
            door_value(
               "%s", safe_field(pilot_owner->person.name).c_str());
            door_label(")");
         }
         od_printf("\n\r");
      }
      door_information(
         "\n\rA person may cover several roles. Off watch permits full rest.\n\r");
      if(page_count > 1) {
         door_option_prompt({
            "[Letter] Toggle",
            "[< >] Page",
            "[-] Off watch",
            "[Enter] Save",
            "[Esc] Cancel",
            "[?] Help",
         }, false);
      } else {
         door_option_prompt({
            "[Letter] Toggle",
            "[-] Off watch",
            "[Enter] Save",
            "[Esc] Cancel",
            "[?] Help",
         }, false);
      }
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         std::sort(selected.begin(), selected.end());
         return selected;
      }
      if(key == 27) {
         return std::nullopt;
      }
      if(key == '-') {
         selected.clear();
         continue;
      }
      if(key == '>' && page + 1 < page_count) {
         ++page;
         continue;
      }
      if(key == '<' && page > 0) {
         --page;
         continue;
      }
      const auto normalized =
         static_cast<char>(std::toupper(static_cast<unsigned char>(key)));
      if(normalized >= 'A' && normalized <= 'Z') {
         const auto index = static_cast<size_t>(normalized - 'A');
         if(index >= first && index < last) {
            const auto slot_id = snapshot.roles[index].slot_id;
            const auto& role = snapshot.roles[index];
            const bool pilot_taken = std::any_of(
                                        snapshot.members.begin(),
                                        snapshot.members.end(),
            [&role, &member](const auto & candidate) {
               return candidate.person_id != member.person_id &&
                      role.role_kind == ct::CrewRoleKind::Pilot &&
                      std::find(
                         candidate.assigned_slot_ids.begin(),
                         candidate.assigned_slot_ids.end(),
                         role.slot_id) != candidate.assigned_slot_ids.end();
            });
            if((slot_id == 0 && !member.captain) || pilot_taken) {
               continue;
            }
            const auto found =
               std::find(selected.begin(), selected.end(), slot_id);
            if(found == selected.end()) {
               selected.push_back(slot_id);
            } else {
               selected.erase(found);
            }
         }
      }
   }
}

void show_crew_member(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   ct::CrewManagementSnapshot& snapshot,
   size_t index)
{
   const std::vector<ct::SkillDefinition> definitions;
   while(true) {
      const auto& member = snapshot.members[index];
      const auto living_positions = crew_member_living_positions(member);
      const auto established_positions =
         crew_member_established_positions(snapshot, member);
      od_clr_scr();
      door_heading("%s\n\r", safe_field(member.person.name).c_str());
      door_heading(
         "%s\n\r\n\r",
         std::string(safe_field(member.person.name).size(), '=').c_str());
      door_label("Service appointment: ");
      door_identifier(
         "%s\n\r",
         safe_field(
            member.captain
            ? std::string("Captain")
            : ct::describe_crew_naming(
               member.role_kind, member.role, established_positions)
            .appointment)
         .c_str());
      door_label("On watch: ");
      door_identifier(
         "%s\n\r", safe_field(crew_assignments(snapshot, member)).c_str());
      door_label("Condition: ");
      if(member.available) {
         door_value("%s", person_condition_name(member.condition));
      } else {
         door_warning("%s", person_condition_name(member.condition));
      }
      door_label("  Injury ");
      door_number("%u", member.injury_points);
      door_label("  Fatigue ");
      door_number("%u\n\r", member.fatigue_points);
      door_label("Physical: STR ");
      door_number("%u", member.current_strength);
      door_label(" / DEX ");
      door_number("%u", member.current_dexterity);
      door_label(" / END ");
      door_number("%u\n\r", member.current_endurance);
      door_label("Availability: ");
      door_identifier("%s\n\r", crew_availability_name(member.availability));
      if(member.location_kind != ct::CrewLocationKind::AboardShip) {
         door_label("Present at: ");
         door_identifier("%s\n\r", safe_field(member.shore_location).c_str());
         if(member.availability != ct::CrewAvailability::AwaitingRecall) {
            door_label("Booked through: ");
            door_number("%s\n\r", game_date(member.available_second).c_str());
         }
      }
      door_label("Articles: ");
      door_identifier("%s", crew_service_name(member.service_kind));
      if(member.monthly_salary_credits != 0) {
         door_label("  Cr");
         door_number("%llu/month", static_cast<unsigned long long>(member.monthly_salary_credits));
      }
      if(member.prize_share_basis_points != 0) {
         door_label("  ");
         door_number("%u.%02u%% prize share",
                     member.prize_share_basis_points / 100,
                     member.prize_share_basis_points % 100);
      }
      od_printf("\n\r");
      door_label("Standing: morale ");
      door_number("%u", member.morale);
      door_label(" / loyalty ");
      door_number("%u", member.loyalty);
      if(member.arrears_credits != 0) {
         door_warning("  ARREARS Cr%llu",
                      static_cast<unsigned long long>(member.arrears_credits));
      }
      od_printf("\n\r");
      if(established_positions > 1 || living_positions != established_positions) {
         door_label("Service complement: ");
         if(living_positions < established_positions) {
            door_warning("%u/%u", living_positions, established_positions);
            door_warning(
               " — %u position%s short",
               established_positions - living_positions,
               established_positions - living_positions == 1 ? "" : "s");
         } else {
            door_number("%u/%u", living_positions, established_positions);
         }
         door_label("\n\r");
      }
      od_printf("\n\r");
      render_person(member.person, definitions);
      door_option_prompt({
         "[T] Training",
         "[A] Duty",
         "[L] Leave",
         "[R] Recall",
         "[F] First aid",
         "[S] Surgery",
         "[M] Medical",
         "[V] Transfer",
         "[D] Dismiss",
         "[Enter] Refresh",
         "[Q] Roster",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         const auto person_id = member.person_id;
         snapshot = ct::get_crew_management(
            connection, session_epoch, random_command_id(random), request_id++);
         const auto refreshed = std::find_if(
            snapshot.members.begin(), snapshot.members.end(),
            [person_id](const auto& candidate) {
               return candidate.person_id == person_id;
            });
         if(refreshed == snapshot.members.end()) {
            return;
         }
         index = static_cast<size_t>(refreshed - snapshot.members.begin());
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == 't' || key == 'T') {
         auto proposed = member.person;
         edit_training_target(proposed, definitions);
         snapshot = ct::set_crew_training_target(
                       connection,
                       session_epoch,
                       member.person_id,
                       proposed.training.skill,
                       random_command_id(random),
                       request_id++);
         return;
      } else if(key == 'a' || key == 'A') {
         const auto assignments =
            edit_duty_assignments(snapshot, member);
         if(!assignments) {
            continue;
         }
         snapshot = ct::set_crew_assignments(
                       connection,
                       session_epoch,
                       member.person_id,
                       *assignments,
                       random_command_id(random),
                       request_id++);
         return;
      } else {
         std::optional<ct::PersonnelActionKind> action;
         uint64_t target_ship_id = 0;
         uint16_t duration_days = 0;
         if(key == 'l' || key == 'L') {
            action = ct::PersonnelActionKind::ShoreLeave;
            const auto days = input_number("Days of leave", 1, 30);
            if(!days) {
               continue;
            }
            duration_days = static_cast<uint16_t>(*days);
         } else if(key == 'r' || key == 'R') {
            action = ct::PersonnelActionKind::Recall;
         } else if(key == 'f' || key == 'F') {
            action = ct::PersonnelActionKind::FirstAid;
         } else if(key == 's' || key == 'S') {
            action = ct::PersonnelActionKind::Surgery;
         } else if(key == 'm' || key == 'M') {
            action = ct::PersonnelActionKind::MedicalCare;
            const auto days = input_number("Days of care", 1, 30);
            if(!days) {
               continue;
            }
            duration_days = static_cast<uint16_t>(*days);
         } else if(key == 'v' || key == 'V') {
            action = ct::PersonnelActionKind::Transfer;
            const auto ship = input_number("Receiving ship number", 1, 999999999);
            if(!ship) {
               continue;
            }
            target_ship_id = *ship;
         } else if(key == 'd' || key == 'D') {
            door_prompt("Discharge this crewmember?\n\r");
            door_option_prompt({"[D] Confirm", "[Q] Keep", "[?] Help"}, false);
            const auto confirm = od_get_key(TRUE);
            od_printf("\n\r");
            if(confirm != 'd' && confirm != 'D') {
               continue;
            }
            action = ct::PersonnelActionKind::Dismiss;
         }
         if(action) {
            try {
               snapshot = ct::apply_personnel_action(
                  connection,
                  session_epoch,
                  member.person_id,
                  member.service_revision,
                  *action,
                  target_ship_id,
                  duration_days,
                  random_command_id(random),
                  request_id++);
               return;
            } catch(const std::exception& error) {
               door_error("%s  Press any key.\n\r", safe_field(error.what()).c_str());
               od_get_key(TRUE);
            }
         }
      }
   }
}

void show_crew_manager(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Crew);
   auto snapshot = ct::get_crew_management(
                      connection,
                      session_epoch,
                      random_command_id(random),
                      request_id++);
   if(snapshot.members.size() > 26) {
      throw std::runtime_error(
         "crew manager supports at most 26 named crew records");
   }
   const bool compact = output().columns() < 68;
   const size_t rows_per_entry = compact ? 2 : 1;
   const size_t reserved_rows = compact ? 8 : 7;
   const size_t available_rows =
      output().rows() > reserved_rows ? output().rows() - reserved_rows : 1;
   const size_t page_size =
      std::max<size_t>(1, available_rows / rows_per_entry);
   size_t page = 0;
   while(true) {
      const size_t page_count =
         std::max<size_t>(1, (snapshot.members.size() + page_size - 1) / page_size);
      page = std::min(page, page_count - 1);
      render_managed_crew_roster(snapshot, page);
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         snapshot = ct::get_crew_management(
            connection, session_epoch, random_command_id(random), request_id++);
         if(snapshot.members.size() > 26) {
            throw std::runtime_error(
               "crew manager supports at most 26 named crew records");
         }
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == '>' && page + 1 < page_count) {
         ++page;
      } else if(key == '<' && page > 0) {
         --page;
      } else {
         const auto normalized =
            static_cast<char>(std::toupper(static_cast<unsigned char>(key)));
         if(normalized >= 'A' && normalized <= 'Z') {
            const auto index = static_cast<size_t>(normalized - 'A');
            if(index < snapshot.members.size()) {
               show_crew_member(
                  connection,
                  session_epoch,
                  random,
                  request_id,
                  snapshot,
                  index);
               page = index / page_size;
            }
         }
      }
   }
}

void print_millitons(const uint64_t millitons)
{
   door_number("%s t", ct::format_tonnage(millitons).c_str());
}

void show_ship_subsystem(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   ct::ShipStatusSnapshot& snapshot,
   const uint16_t subsystem_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Ship);
   while(true) {
      const auto found = std::find_if(
         snapshot.subsystems.begin(), snapshot.subsystems.end(),
         [subsystem_id](const auto& candidate) {
            return candidate.subsystem_id == subsystem_id;
         });
      if(found == snapshot.subsystems.end()) {
         return;
      }
      const auto& subsystem = *found;
      od_clr_scr();
      door_heading("%s\n\r", safe_field(subsystem.label).c_str());
      door_heading(
         "%s\n\r\n\r",
         std::string(safe_field(subsystem.label).size(), '=').c_str());
      door_label("Underlying damage: ");
      door_number(
         "%u/%u hits\n\r", subsystem.sustained_hits, subsystem.maximum_hits);
      door_label("Battlefield coverage: ");
      door_number("%u hits\n\r", subsystem.battlefield_repair_hits);
      door_label("Effective during encounter: ");
      door_number("%u hits\n\r", subsystem.effective_hits);
      door_label("Current effect: ");
      if(subsystem.effective_hits == 0) {
         door_identifier(
            "%s\n\r", safe_field(subsystem.operational_effect).c_str());
      } else {
         door_warning(
            "%s\n\r", safe_field(subsystem.operational_effect).c_str());
      }
      od_printf("\n\r");
      door_label("Last proper repair: ");
      door_number("%s\n\r", game_date(subsystem.last_proper_repair_second).c_str());
      door_label("Installed: ");
      door_number("%s\n\r", game_date(subsystem.installed_second).c_str());
      door_label("Last refit: ");
      door_number("%s\n\r", game_date(subsystem.last_refit_second).c_str());
      door_label("Recorded service: ");
      door_number(
         "%u months / %u duty cycles\n\r",
         subsystem.calendar_age_months,
         subsystem.duty_cycles);
      if(subsystem.neglect_damage_hits > 0) {
         door_warning(
            "\n\r%u underlying hit(s) came from neglected routine upkeep.\n\r",
            subsystem.neglect_damage_hits);
      }
      if(subsystem.battlefield_repair_hits > 0) {
         door_warning(
            "\n\rBattlefield repair only masks underlying damage. It expires "
            "when the encounter ends and does not count as maintenance.\n\r");
      }
      door_option_prompt({
         "[Enter] Refresh",
         "[Q] Subsystem list",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == '\r' || key == '\n') {
         snapshot = ct::get_ship_status(
            connection, session_epoch, random_command_id(random), request_id++);
      }
   }
}

void show_ship_subsystems(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   ct::ShipStatusSnapshot& snapshot)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Ship);
   const size_t reserved_rows = 7;
   const size_t available_rows =
      output().rows() > reserved_rows ? output().rows() - reserved_rows : 1;
   const size_t page_size = std::min<size_t>(26, available_rows);
   const size_t page_count = std::max<size_t>(
                                1, (snapshot.subsystems.size() + page_size - 1) / page_size);
   size_t page = 0;
   while(true) {
      od_clr_scr();
      door_heading("Subsystem Status - ");
      door_value("%s\n\r", safe_field(snapshot.ship_name).c_str());
      door_heading("================\n\r\n\r");
      const size_t first = page * page_size;
      const size_t last =
         std::min(first + page_size, snapshot.subsystems.size());
      for(size_t index = first; index < last; ++index) {
         const auto& subsystem = snapshot.subsystems[index];
         door_number("%c", crew_menu_key(index - first));
         door_label(". ");
         door_identifier("%-22s", safe_field(subsystem.label).c_str());
         if(subsystem.sustained_hits == 0) {
            door_value("Ready");
         } else if(subsystem.battlefield_repair_hits > 0) {
            door_warning(
               "Patched %u/%u",
               subsystem.sustained_hits,
               subsystem.maximum_hits);
         } else if(subsystem.sustained_hits > 0) {
            door_warning(
               "Damage %u/%u",
               subsystem.sustained_hits,
               subsystem.maximum_hits);
         }
         od_printf("\n\r");
      }
      if(page_count > 1) {
         door_option_prompt({
            "[Letter] Details",
            "[< >] Page",
            "[Enter] Refresh",
            "[Q] Ship status",
            "[?] Help",
         });
      } else {
         door_option_prompt({
            "[Letter] Details",
            "[Enter] Refresh",
            "[Q] Ship status",
            "[?] Help",
         });
      }
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         snapshot = ct::get_ship_status(
            connection, session_epoch, random_command_id(random), request_id++);
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == '>' && page + 1 < page_count) {
         ++page;
      } else if(key == '<' && page > 0) {
         --page;
      } else {
         const auto normalized =
            static_cast<char>(std::toupper(static_cast<unsigned char>(key)));
         if(normalized >= 'A' && normalized <= 'Z') {
            const auto index =
               first + static_cast<size_t>(normalized - 'A');
            if(index < last) {
               show_ship_subsystem(
                  connection,
                  session_epoch,
                  random,
                  request_id,
                  snapshot,
                  snapshot.subsystems[index].subsystem_id);
            }
         }
      }
   }
}

const char* ship_title_name(const ct::ShipTitleKind title)
{
   switch(title) {
   case ct::ShipTitleKind::OwnedWithLien:
      return "Private title, secured debt";
   case ct::ShipTitleKind::SponsorOwned:
      return "Sponsor's vessel";
   case ct::ShipTitleKind::InstitutionOwned:
      return "Admiralty property";
   case ct::ShipTitleKind::OwnedClear:
      return "Clear private title";
   case ct::ShipTitleKind::PrizeCustody:
      return "Prize under custody";
   case ct::ShipTitleKind::StolenRegistry:
      return "Contested registry";
   case ct::ShipTitleKind::CourtImpound:
      return "Held by prize court";
   }
   return "Unknown title";
}

void transfer_fleet_stores(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   ct::FleetSnapshot& fleet,
   const size_t source_index)
{
   if(fleet.ships.size() < 2) {
      door_information("No second vessel is alongside for a transfer.\n\r");
      wait_for_enter();
      return;
   }
   output().resume_paging();
   door_heading("Receiving Vessel\n\r================\n\r\n\r");
   std::vector<size_t> destinations;
   for(size_t index = 0; index < fleet.ships.size(); ++index) {
      if(index == source_index) {
         continue;
      }
      destinations.push_back(index);
      door_number("%zu", destinations.size());
      door_label(". ");
      door_identifier("%s", safe_field(fleet.ships[index].name).c_str());
      door_label(" — ");
      door_value("%s, %s\n\r",
                 safe_field(fleet.ships[index].system_name).c_str(),
                 safe_field(fleet.ships[index].location).c_str());
   }
   const auto destination_choice = input_number(
      "Receiving vessel", 1, static_cast<unsigned>(destinations.size()));
   if(!destination_choice) {
      return;
   }
   const auto destination_index = destinations[*destination_choice - 1];
   const auto& source = fleet.ships[source_index];
   const auto& destination = fleet.ships[destination_index];
   door_option_prompt({
      "[1] Cargo",
      "[2] Fuel",
      "[3] Ammunition",
      "[4] Provisions",
   });
   const auto kind_choice = input_number("Stores", 1, 4);
   if(!kind_choice) {
      return;
   }
   auto kind = ct::StoreTransferKind::Cargo;
   uint64_t cargo_lot_id = 0;
   std::string item_id;
   uint64_t quantity = 0;
   if(*kind_choice == 1) {
      if(source.cargo.empty()) {
         door_information("The selected vessel's hold is empty.\n\r");
         wait_for_enter();
         return;
      }
      output().resume_paging();
      for(size_t index = 0; index < source.cargo.size(); ++index) {
         door_number("%zu", index + 1);
         door_label(". ");
         door_identifier("%s  ", safe_field(source.cargo[index].commodity_name).c_str());
         print_millitons(source.cargo[index].quantity_millitons);
         od_printf("\n\r");
      }
      const auto lot_choice = input_number(
         "Cargo lot", 1, static_cast<unsigned>(source.cargo.size()));
      if(!lot_choice) {
         return;
      }
      const auto& lot = source.cargo[*lot_choice - 1];
      const auto amount = input_tonnage(
         "Quantity in tonnes", lot.quantity_millitons);
      if(!amount) {
         return;
      }
      cargo_lot_id = lot.cargo_lot_id;
      quantity = *amount;
   } else if(*kind_choice == 2) {
      kind = ct::StoreTransferKind::Fuel;
      const auto amount = input_tonnage(
         "Fuel in tonnes", source.fuel_millitons);
      if(!amount) {
         return;
      }
      quantity = *amount;
   } else if(*kind_choice == 3) {
      kind = ct::StoreTransferKind::Ammunition;
      std::vector<size_t> stocked;
      output().resume_paging();
      for(size_t index = 0; index < source.ammunition.size(); ++index) {
         if(source.ammunition[index].remaining == 0) {
            continue;
         }
         stocked.push_back(index);
         door_number("%zu", stocked.size());
         door_label(". ");
         door_identifier("%s  ", safe_field(source.ammunition[index].ammunition_id).c_str());
         door_number("%u units\n\r", source.ammunition[index].remaining);
      }
      if(stocked.empty()) {
         door_information("The selected vessel has no ammunition to transfer.\n\r");
         wait_for_enter();
         return;
      }
      const auto lot_choice = input_number(
         "Ammunition", 1, static_cast<unsigned>(stocked.size()));
      if(!lot_choice) {
         return;
      }
      const auto& lot = source.ammunition[stocked[*lot_choice - 1]];
      const auto amount = input_number("Units", 1, lot.remaining);
      if(!amount) {
         return;
      }
      item_id = lot.ammunition_id;
      quantity = *amount;
   } else {
      kind = ct::StoreTransferKind::Provisions;
      const auto amount = input_number(
         "Person-days", 1,
         static_cast<unsigned>(std::min<uint64_t>(
            source.provision_person_days, std::numeric_limits<unsigned>::max())));
      if(!amount) {
         return;
      }
      quantity = *amount;
   }
   try {
      fleet = ct::transfer_ship_stores(
                 connection,
                 session_epoch,
                 fleet.revision,
                 source.ship_id,
                 destination.ship_id,
                 kind,
                 cargo_lot_id,
                 item_id,
                 quantity,
                 random_command_id(random),
                 request_id++);
   } catch(const std::exception& error) {
      door_error("%s\n\r", safe_field(error.what()).c_str());
      wait_for_enter();
   }
}

void show_fleet_manager(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Ship);
   auto fleet = ct::get_fleet(
                   connection,
                   session_epoch,
                   random_command_id(random),
                   request_id++);
   while(true) {
      od_clr_scr();
      door_heading("Vessel Roster\n\r=============\n\r\n\r");
      for(size_t index = 0; index < fleet.ships.size(); ++index) {
         const auto& ship = fleet.ships[index];
         door_number("%zu", index + 1);
         door_label(". ");
         if(ship.online_controlled) {
            door_success("[ONLINE] ");
         } else {
            door_identifier("[PLAYER] ");
         }
         if(ship.active) {
            door_identifier("* %s", safe_field(ship.name).c_str());
         } else {
            door_value("  %s", safe_field(ship.name).c_str());
         }
         door_label(" — ");
         door_value("%s\n\r", safe_field(ship.class_name).c_str());
         door_label("   ");
         door_value("%s, %s; Captain %s\n\r",
                    safe_field(ship.system_name).c_str(),
                    safe_field(ship.location).c_str(),
                    safe_field(ship.commanding_person_name).c_str());
      }
      door_option_prompt({
         "[Number] Vessel",
         "[Enter] Refresh",
         "[Q] Ship status",
         "[?] Help",
      });
      std::array<char, 16> input{};
      od_input_str(input.data(), static_cast<INT>(input.size() - 1), 32, 255);
      if(input[0] == '\0') {
         fleet = ct::get_fleet(
                    connection, session_epoch, random_command_id(random), request_id++);
         continue;
      }
      if(input[0] == '?' && input[1] == '\0') {
         show_context_help();
         continue;
      }
      if((input[0] == 'q' || input[0] == 'Q') && input[1] == '\0') {
         return;
      }
      unsigned choice = 0;
      const auto [end, error] = std::from_chars(
         input.data(), input.data() + std::strlen(input.data()), choice);
      if(error != std::errc() || end != input.data() + std::strlen(input.data()) ||
            choice < 1 || choice > fleet.ships.size()) {
         door_error("Select a vessel number from the roster.\n\r");
         wait_for_enter();
         continue;
      }
      size_t index = choice - 1;
      const auto selected_ship_id = fleet.ships[index].ship_id;
      while(true) {
         const auto& ship = fleet.ships[index];
         od_clr_scr();
         door_heading("%s\n\r", safe_field(ship.name).c_str());
         door_heading("%s\n\r\n\r", std::string(safe_field(ship.name).size(), '=').c_str());
         door_label("Class: ");
         door_value("%s\n\r", safe_field(ship.class_name).c_str());
         door_label("Registry: ");
         door_value("%s\n\r", ship_title_name(ship.title));
         door_label("Station: ");
         door_value("%s, %s\n\r",
                    safe_field(ship.system_name).c_str(),
                    safe_field(ship.location).c_str());
         door_label("Captain: ");
         door_identifier("%s\n\r", safe_field(ship.commanding_person_name).c_str());
         door_label("Control: ");
         if(ship.online_controlled) {
            door_success("Online player command\n\r");
         } else {
            door_value("Player-owned; standing orders\n\r");
         }
         door_label("Fuel: ");
         print_millitons(ship.fuel_millitons);
         door_label(" / ");
         print_millitons(ship.fuel_capacity_millitons);
         od_printf("\n\r");
         door_label("Cargo: ");
         print_millitons(ship.cargo_used_millitons);
         door_label(" / ");
         print_millitons(ship.cargo_capacity_millitons);
         od_printf("\n\r");
         std::vector<std::string_view> options;
         if(!ship.active) {
            options.emplace_back("[C] Assume command");
            options.emplace_back("[A] Assign captain");
         }
         options.emplace_back("[T] Transfer stores");
         options.emplace_back("[Enter] Refresh");
         options.emplace_back("[Q] Roster");
         options.emplace_back("[?] Help");
         door_option_prompt(options);
         const auto key = od_get_key(TRUE);
         if(key == '\r' || key == '\n') {
            fleet = ct::get_fleet(
                       connection, session_epoch, random_command_id(random), request_id++);
            const auto refreshed = std::find_if(
               fleet.ships.begin(), fleet.ships.end(),
               [selected_ship_id](const auto& candidate) {
                  return candidate.ship_id == selected_ship_id;
               });
            if(refreshed == fleet.ships.end()) {
               break;
            }
            index = static_cast<size_t>(refreshed - fleet.ships.begin());
            continue;
         }
         if(key == 'q' || key == 'Q') {
            break;
         }
         if((key == 'c' || key == 'C') && !ship.active) {
            try {
               fleet = ct::set_active_ship(
                          connection,
                          session_epoch,
                          fleet.revision,
                          ship.ship_id,
                          random_command_id(random),
                          request_id++);
               break;
            } catch(const std::exception& error) {
               door_error("%s\n\r", safe_field(error.what()).c_str());
               wait_for_enter();
            }
         } else if((key == 'a' || key == 'A') && !ship.active) {
            auto crew = ct::get_crew_management(
                           connection,
                           session_epoch,
                           random_command_id(random),
                           request_id++);
            if(crew.members.empty()) {
               door_information("No officer is available for assignment.\n\r");
               wait_for_enter();
               continue;
            }
            output().resume_paging();
            for(size_t crew_index = 0; crew_index < crew.members.size(); ++crew_index) {
               door_number("%zu", crew_index + 1);
               door_label(". ");
               door_identifier("%s", safe_field(crew.members[crew_index].person.name).c_str());
               door_label(" — ");
               door_value("%s\n\r", safe_field(crew.members[crew_index].role).c_str());
            }
            const auto officer = input_number(
               "Officer", 1, static_cast<unsigned>(crew.members.size()));
            if(officer) {
               try {
                  fleet = ct::assign_ship_captain(
                             connection,
                             session_epoch,
                             fleet.revision,
                             ship.ship_id,
                             crew.members[*officer - 1].person_id,
                             random_command_id(random),
                             request_id++);
               } catch(const std::exception& error) {
                  door_error("%s\n\r", safe_field(error.what()).c_str());
                  wait_for_enter();
               }
            }
         } else if(key == 't' || key == 'T') {
            transfer_fleet_stores(
               connection, session_epoch, random, request_id, fleet, index);
            break;
         }
      }
   }
}

void show_ship_manager(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Ship);
   auto snapshot = ct::get_ship_status(
                      connection,
                      session_epoch,
                      random_command_id(random),
                      request_id++);
   while(true) {
      const auto damaged = std::count_if(
                              snapshot.subsystems.begin(),
                              snapshot.subsystems.end(),
      [](const auto & subsystem) {
         return subsystem.sustained_hits > 0;
      });
      const auto patched = std::count_if(
                              snapshot.subsystems.begin(),
                              snapshot.subsystems.end(),
      [](const auto & subsystem) {
         return subsystem.battlefield_repair_hits > 0;
      });
      od_clr_scr();
      door_heading("Ship Status - ");
      door_value("%s\n\r", safe_field(snapshot.ship_name).c_str());
      door_heading("=============\n\r\n\r");
      door_label("Displacement: ");
      print_millitons(snapshot.displacement_millitons);
      door_label("  Performance: ");
      door_identifier(
         "J-%u / %uG\n\r", snapshot.jump_rating, snapshot.thrust_g);
      door_label("Fuel: ");
      print_millitons(snapshot.current_fuel_millitons);
      door_label(" / ");
      print_millitons(snapshot.fuel_capacity_millitons);
      door_label("  Jump use: ");
      print_millitons(snapshot.jump_fuel_millitons);
      od_printf("\n\r");
      door_label("Cargo capacity: ");
      print_millitons(snapshot.cargo_capacity_millitons);
      od_printf("\n\r\n\r");
      door_label("Damaged subsystems: ");
      if(damaged == 0) {
         door_value("none\n\r");
      } else {
         door_warning("%zu\n\r", damaged);
      }
      door_label("Battlefield patches: ");
      if(patched == 0) {
         door_value("none\n\r");
      } else {
         door_warning("%zu (temporary)\n\r", patched);
      }
      door_label("Routine upkeep: ");
      if(snapshot.consecutive_missed_maintenance == 0) {
         door_value("paid through %s\n\r",
                    game_date(snapshot.maintenance_paid_through_second).c_str());
      } else {
         door_warning(
            "%u missed cycle(s), %llu Cr arrears\n\r",
            snapshot.consecutive_missed_maintenance,
            static_cast<unsigned long long>(snapshot.maintenance_arrears_credits));
      }
      door_label("Next upkeep account: ");
      door_number(
         "%s (%llu Cr)\n\r",
         game_date(snapshot.next_maintenance_second).c_str(),
         static_cast<unsigned long long>(snapshot.monthly_maintenance_credits));
      door_label("Life-support stores: ");
      if(snapshot.provisions.person_days_remaining == 0) {
         door_warning("empty\n\r");
      } else {
         door_number("%llu/%llu person-days\n\r",
                     static_cast<unsigned long long>(snapshot.provisions.person_days_remaining),
                     static_cast<unsigned long long>(snapshot.provisions.capacity_person_days));
      }
      if(!snapshot.ammunition.empty()) {
         const auto rounds = std::accumulate(snapshot.ammunition.begin(), snapshot.ammunition.end(), uint64_t{0}, [](
         uint64_t total, const auto & lot) {
            return total + lot.remaining;
         });
         const auto capacity = std::accumulate(snapshot.ammunition.begin(), snapshot.ammunition.end(),
         uint64_t{0}, [](uint64_t total, const auto & lot) {
            return total + lot.capacity;
         });
         door_label("Magazine: ");
         door_number("%llu/%llu units\n\r", static_cast<unsigned long long>(rounds),
                     static_cast<unsigned long long>(capacity));
      }
      door_label("Service history: ");
      door_number(
         "%u transits / %u refits\n\r",
         snapshot.transit_count,
         snapshot.completed_refits);
      const bool warranty_active =
         !snapshot.warranty_voided &&
         snapshot.current_game_second <= snapshot.warranty_expires_second &&
         snapshot.transit_count <= snapshot.warranty_transit_limit;
      door_label("Warranty: ");
      if(warranty_active) {
         door_value(
            "active to %s or transit %u\n\r",
            game_date(snapshot.warranty_expires_second).c_str(),
            snapshot.warranty_transit_limit);
      } else {
         door_warning("expired\n\r");
      }
      if(snapshot.active_activity.has_value()) {
         door_label("Active operation: ");
         door_warning(
            "%s until %s\n\r",
            ship_activity_name(snapshot.active_activity->kind),
            game_date(snapshot.active_activity->due_second).c_str());
      }
      if(!snapshot.recovery_status.empty()) {
         door_label("Recovery watch: ");
         door_warning("%s\n\r", safe_field(snapshot.recovery_status).c_str());
      }
      for(const auto& symptom : snapshot.manifested_symptoms) {
         door_label("Reported symptom: ");
         door_warning("%s\n\r", safe_field(symptom).c_str());
      }
      door_option_prompt({
         "[S] Subsystems",
         "[F] Vessel roster",
         "[P] Proper repair",
         "[R] Begin refit",
         "[Enter] Refresh",
         "[Q] Console",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         snapshot = ct::get_ship_status(
            connection, session_epoch, random_command_id(random), request_id++);
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == 's' || key == 'S') {
         show_ship_subsystems(
            connection, session_epoch, random, request_id, snapshot);
      } else if(key == 'f' || key == 'F') {
         show_fleet_manager(connection, session_epoch, random, request_id);
         snapshot = ct::get_ship_status(
                       connection,
                       session_epoch,
                       random_command_id(random),
                       request_id++);
      } else if(key == 'p' || key == 'P') {
         const auto services = ct::get_docked_services(connection, session_epoch, random_command_id(random),
            request_id++);
         if(services.repair.empty()) {
            door_information("No subsystem requires yard work.\n\r");
            wait_for_enter();
            continue;
         }
         output().resume_paging();
         for(size_t index = 0; index < services.repair.size(); ++index) {
            const auto& item = services.repair[index];
            door_number("%zu", index + 1);
            door_label(". ");
            if(item.available) {
               door_identifier("%s", safe_field(item.label).c_str());
               door_number("  Cr%llu / %s\n\r", static_cast<unsigned long long>(item.cost_credits),
                           course_duration(item.service_seconds).c_str());
            } else {
               door_warning("%s — %s\n\r", safe_field(item.label).c_str(),
                            safe_field(item.unavailable_reason).c_str());
            }
         }
         const auto choice = input_number("Yard order", 1, static_cast<unsigned>(services.repair.size()));
         if(choice) {
            try {
               const auto& item = services.repair[*choice - 1];
               if(!item.available) {
                  throw std::runtime_error(item.unavailable_reason);
               }
               ct::DockedServiceOrder order{};
               order.expected_ship_revision = services.ship_revision;
               order.kind = item.replacement ? ct::DockedServiceOrder::Kind::Replacement :
                            ct::DockedServiceOrder::Kind::ProperRepair;
               order.subsystem_id = item.subsystem_id;
               order.reconditioned = item.reconditioned;
               snapshot = ct::commit_docked_service(connection, session_epoch, order, random_command_id(random),
                                                    request_id++);
            } catch(const std::exception& error) {
               door_error("%s  Press any key.\n\r", safe_field(error.what()).c_str());
               od_get_key(TRUE);
            }
         }
      } else if(key == 'r' || key == 'R') {
         try {
            const auto services = ct::get_docked_services(connection, session_epoch, random_command_id(random),
               request_id++);
            if(!services.refit_available) {
               throw std::runtime_error(services.refit_unavailable_reason);
            }
            ct::DockedServiceOrder order{};
            order.expected_ship_revision = services.ship_revision;
            order.kind = ct::DockedServiceOrder::Kind::Refit;
            snapshot = ct::commit_docked_service(connection, session_epoch, order, random_command_id(random),
                                                 request_id++);
         } catch(const std::exception& error) {
            door_error("%s  Press any key.\n\r", safe_field(error.what()).c_str());
            od_get_key(TRUE);
         }
      }
   }
}

const char* task_state_name(const ct::TaskState state)
{
   switch(state) {
   case ct::TaskState::ClaimPending:
      return "Claim pending";
   case ct::TaskState::Accepted:
      return "Accepted";
   case ct::TaskState::Sourcing:
      return "Sourcing";
   case ct::TaskState::Loading:
      return "Loading";
   case ct::TaskState::InTransit:
      return "In transit";
   case ct::TaskState::AwaitingSettlement:
      return "Awaiting settlement";
   case ct::TaskState::Completed:
      return "Completed";
   case ct::TaskState::Expired:
      return "Expired";
   case ct::TaskState::Cancelled:
      return "Cancelled";
   case ct::TaskState::Defaulted:
      return "Defaulted";
   case ct::TaskState::Disputed:
      return "Disputed";
   }
   return "Unknown";
}

const char* task_kind_name(const ct::TaskKind kind)
{
   switch(kind) {
   case ct::TaskKind::Freight:
      return "Freight contract";
   case ct::TaskKind::Passenger:
      return "Passenger contract";
   case ct::TaskKind::PurchaseOrder:
      return "Purchase order";
   case ct::TaskKind::ForwardSale:
      return "Forward sale";
   case ct::TaskKind::SupplyCommitment:
      return "Supply commitment";
   case ct::TaskKind::Charter:
      return "Charter";
   case ct::TaskKind::Courier:
      return "Courier charter";
   case ct::TaskKind::DiscoveryBounty:
      return "Discovery bounty";
   case ct::TaskKind::CombatBounty:
      return "Combat bounty";
   }
   return "Contract";
}

void report_offer_claim(const ct::TaskLedger& ledger, const uint64_t offer_id)
{
   const auto task = std::find_if(
      ledger.tasks.begin(), ledger.tasks.end(),
      [offer_id](const auto& candidate) { return candidate.offer.offer_id == offer_id; });
   if(task != ledger.tasks.end() && task->state == ct::TaskState::ClaimPending) {
      door_success(
         "Claim filed. It is not awarded until the issuing office's reply reaches you.\n\r");
   } else {
      door_success("The issuing office awarded the offer and entered it in the task ledger.\n\r");
   }
}

struct PickupRouteEstimate {
   bool available;
   uint64_t current_second;
   uint64_t elapsed_seconds;
};

struct PickupSlack {
   bool available;
   bool late;
   uint64_t seconds;
};

PickupSlack pickup_slack(const ct::TaskOffer& offer,
                         const PickupRouteEstimate& route)
{
   if(!route.available) {
      return {.available = false, .late = true, .seconds = 0};
   }
   const auto arrival = route.current_second >
                           std::numeric_limits<uint64_t>::max() -
                              route.elapsed_seconds
                        ? std::numeric_limits<uint64_t>::max()
                        : route.current_second + route.elapsed_seconds;
   return arrival <= offer.expires_second
          ? PickupSlack{
               .available = true,
               .late = false,
               .seconds = offer.expires_second - arrival,
            }
          : PickupSlack{
               .available = true,
               .late = true,
               .seconds = arrival - offer.expires_second,
            };
}

void print_pickup_slack(const PickupSlack& slack)
{
   if(!slack.available) {
      door_error("no executable course");
      return;
   }
   const auto text = course_duration(slack.seconds);
   if(slack.late) {
      door_error("late by %s", text.c_str());
   } else if(slack.seconds < 30 * 60) {
      door_error("%s", text.c_str());
   } else if(slack.seconds > 6 * 60 * 60) {
      door_success("%s", text.c_str());
   } else {
      door_warning("%s", text.c_str());
   }
}

std::vector<std::string> task_offer_unavailable_reasons(
   const ct::TaskOffer& offer,
   const PickupRouteEstimate& pickup_route,
   const PickupRouteEstimate* delivery_route)
{
   auto reasons = offer.unavailable_reasons;
   const auto slack = pickup_slack(offer, pickup_route);
   if(!slack.available) {
      reasons.emplace_back("No executable course reaches the pickup system.");
   } else if(slack.late) {
      reasons.emplace_back(
         "The fastest course reaches the pickup system after the offer closes.");
   }
   if(delivery_route != nullptr) {
      if(!delivery_route->available) {
         reasons.emplace_back("No executable course reaches the delivery system.");
      } else {
         const auto arrival = delivery_route->current_second >
                                 std::numeric_limits<uint64_t>::max() -
                                    delivery_route->elapsed_seconds
                              ? std::numeric_limits<uint64_t>::max()
                              : delivery_route->current_second +
                                 delivery_route->elapsed_seconds;
         if(arrival > offer.delivery_deadline_second) {
            reasons.emplace_back(
               "The fastest course reaches the delivery system after the deadline.");
         }
      }
   }
   return reasons;
}

void show_task_offer_detail(const ct::TaskOffer& offer,
                            const PickupSlack& slack,
                            const std::vector<std::string>& unavailable_reasons)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Tasks);
   while(true) {
   od_clr_scr();
   door_heading("Signed Offer Instrument\n\r=======================\n\r\n\r");
   door_label("Service:       ");
   door_value("%s\n\r", task_kind_name(offer.kind));
   door_label("Terms:         ");
   door_identifier("%s\n\r", safe_field(offer.title).c_str());
   if(offer.quantity_millitons != 0) {
      door_label("Cargo:         ");
      print_millitons(offer.quantity_millitons);
      od_printf("\n\r");
   }
   if(offer.passenger_count != 0) {
      door_label("Passengers:    ");
      door_number("%u\n\r", offer.passenger_count);
   }
   door_label("Payment:       Cr");
   door_number("%llu\n\r", static_cast<unsigned long long>(offer.payment_credits));
   door_label("Collateral:    Cr");
   door_number("%llu\n\r", static_cast<unsigned long long>(offer.collateral_credits));
   door_label("Failure charge: Cr");
   door_number("%llu\n\r", static_cast<unsigned long long>(offer.failure_penalty_credits));
   door_label("Liability cap: Cr");
   door_number("%llu\n\r", static_cast<unsigned long long>(offer.non_delivery_liability_credits));
   if(offer.late_deduction_per_day_credits != 0) {
      door_label("Late deduction: Cr");
      door_number("%llu/day\n\r", static_cast<unsigned long long>(offer.late_deduction_per_day_credits));
   }
   if(offer.passenger_grace_seconds != 0) {
      door_label("Passenger grace: ");
      door_number("%llu day(s)\n\r",
                  static_cast<unsigned long long>(offer.passenger_grace_seconds / (24 * 60 * 60)));
   }
   door_label("Claim by:      ");
   door_number("%s\n\r", game_date(offer.expires_second).c_str());
   door_label("Pickup slack:  ");
   print_pickup_slack(slack);
   od_printf("\n\r");
   door_label("Deliver by:    ");
   door_number("%s\n\r", game_date(offer.delivery_deadline_second).c_str());
   door_label("Standing:      ");
   door_value("%s", offer.legal ? "lawful" : "proscribed");
   door_label(", ");
   door_value("%s delivery\n\r", offer.partial_delivery_allowed ? "partial" : "complete");
   if(offer.performance_count > 1) {
      door_label("Schedule:      ");
      door_number("%u deliveries", offer.performance_count);
      door_label(" every ");
      door_number("%llu days\n\r", static_cast<unsigned long long>(offer.recurrence_seconds /
            (24 * 60 * 60)));
   }
   if(!unavailable_reasons.empty()) {
      door_warning("\n\rUnavailable to this captain:\n\r");
      for(const auto& reason : unavailable_reasons) {
         door_warning("  - %s\n\r", safe_field(reason).c_str());
      }
   }
   door_option_prompt({
      "[Enter] Refresh",
      "[Q] Task ledger",
      "[?] Help",
   });
   const auto key = od_get_key(TRUE);
   if(key == 'q' || key == 'Q') {
      return;
   }
   }
}

void show_task_manager(ct::TlsConnection& connection, const uint64_t session_epoch,
                       ct::CommandIdGenerator& random, uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Tasks);
   bool show_unavailable = false;
   while(true) {
      auto ledger = ct::get_task_ledger(connection, session_epoch, random_command_id(random),
                                        request_id++);
      const auto charts = ct::get_known_destinations(
         connection, session_epoch, random_command_id(random), request_id++);
      const auto fleet = ct::get_fleet(
         connection, session_epoch, random_command_id(random), request_id++);
      std::unordered_map<uint64_t, PickupRouteEstimate> pickup_routes;
      for(const auto& offer : ledger.local_offers) {
         if(pickup_routes.contains(offer.origin_system_id)) {
            continue;
         }
         if(offer.origin_system_id == charts.current_system_id) {
            pickup_routes.emplace(
               offer.origin_system_id,
               PickupRouteEstimate{
                  .available = true,
                  .current_second = ledger.current_second,
                  .elapsed_seconds = 0,
               });
            continue;
         }
         const auto origin_known = std::any_of(
            charts.systems.begin(), charts.systems.end(),
            [&offer](const auto& system) {
               return system.system_id == offer.origin_system_id;
            });
         if(!origin_known) {
            pickup_routes.emplace(
               offer.origin_system_id,
               PickupRouteEstimate{
                  .available = false,
                  .current_second = ledger.current_second,
                  .elapsed_seconds = 0,
               });
            continue;
         }
         const auto plot = ct::plot_course(
            connection,
            session_epoch,
            charts.current_system_id,
            offer.origin_system_id,
            true,
            random_command_id(random),
            request_id++);
         pickup_routes.emplace(
            offer.origin_system_id,
            PickupRouteEstimate{
               .available = plot.fastest.available,
               .current_second = plot.current_game_second,
               .elapsed_seconds = plot.fastest.elapsed_seconds,
            });
      }
      std::unordered_map<uint64_t, PickupRouteEstimate> delivery_routes;
      for(const auto& offer : ledger.local_offers) {
         if(offer.origin_system_id != charts.current_system_id ||
               delivery_routes.contains(offer.destination_system_id)) {
            continue;
         }
         if(offer.destination_system_id == charts.current_system_id) {
            delivery_routes.emplace(
               offer.destination_system_id,
               PickupRouteEstimate{
                  .available = true,
                  .current_second = ledger.current_second,
                  .elapsed_seconds = 0,
               });
            continue;
         }
         const auto destination_known = std::any_of(
            charts.systems.begin(), charts.systems.end(),
            [&offer](const auto& system) {
               return system.system_id == offer.destination_system_id;
            });
         if(!destination_known) {
            delivery_routes.emplace(
               offer.destination_system_id,
               PickupRouteEstimate{
                  .available = false,
                  .current_second = ledger.current_second,
                  .elapsed_seconds = 0,
               });
            continue;
         }
         const auto plot = ct::plot_course(
            connection,
            session_epoch,
            charts.current_system_id,
            offer.destination_system_id,
            true,
            random_command_id(random),
            request_id++);
         delivery_routes.emplace(
            offer.destination_system_id,
            PickupRouteEstimate{
               .available = plot.fastest.available,
               .current_second = plot.current_game_second,
               .elapsed_seconds = plot.fastest.elapsed_seconds,
            });
      }
      struct ListedOffer {
         const ct::TaskOffer* offer;
         PickupSlack pickup;
         std::vector<std::string> unavailable_reasons;
      };
      std::vector<ListedOffer> available_offers;
      std::vector<ListedOffer> unavailable_offers;
      for(const auto& offer : ledger.local_offers) {
         const auto delivery = delivery_routes.find(offer.destination_system_id);
         auto listed = ListedOffer{
            .offer = &offer,
            .pickup = pickup_slack(offer, pickup_routes.at(offer.origin_system_id)),
            .unavailable_reasons = task_offer_unavailable_reasons(
               offer,
               pickup_routes.at(offer.origin_system_id),
               delivery == delivery_routes.end() ? nullptr : &delivery->second),
         };
         if(listed.unavailable_reasons.empty()) {
            available_offers.push_back(std::move(listed));
         } else {
            unavailable_offers.push_back(std::move(listed));
         }
      }
      const auto& displayed_offers =
         show_unavailable ? unavailable_offers : available_offers;
      const auto destination_name = [&charts](const uint64_t system_id) {
         const auto found = std::find_if(
            charts.systems.begin(), charts.systems.end(),
            [system_id](const auto& system) { return system.system_id == system_id; });
         return found == charts.systems.end()
                ? std::string("uncharted destination")
                : found->system_name;
      };
      const auto ship_name = [&fleet](const uint64_t ship_id) {
         const auto found = std::find_if(
            fleet.ships.begin(), fleet.ships.end(),
            [ship_id](const auto& ship) { return ship.ship_id == ship_id; });
         return found == fleet.ships.end()
                ? std::string("unlisted vessel")
                : found->name;
      };
      od_clr_scr();
      door_heading("Task Ledger\n\r===========\n\r\n\r");
      door_label("Available cash: ");
      door_number("Cr%llu", static_cast<unsigned long long>(ledger.available_credits));
      door_label("  Reserved: ");
      door_number("Cr%llu\n\r", static_cast<unsigned long long>(ledger.reserved_credits));
      door_identifier("\n\rAccepted obligations\n\r");
      if(ledger.tasks.empty()) {
         door_information("  None\n\r");
      }
      for(const auto& task : ledger.tasks) {
         door_identifier("#%llu ", static_cast<unsigned long long>(task.task_id));
         door_value("%s", safe_field(task.offer.title).c_str());
         door_label(" [");
         door_identifier("%s", task_state_name(task.state));
         door_label("] aboard ");
         door_identifier("%s\n\r   Due ", safe_field(ship_name(
            task.performing_ship_id)).c_str());
         door_number("%s", game_date(task.offer.delivery_deadline_second).c_str());
         door_label("; payment Cr");
         door_number("%llu\n\r", static_cast<unsigned long long>(task.offer.payment_credits));
         print_wrapped(task.status_text, "   ");
      }
      door_identifier("\n\rStanding carriage declaration\n\r");
      if(ledger.carriage.destination_system_id == 0) {
         door_information("  No automatic carriage destination selected\n\r");
      } else {
         door_label("  Destination ");
         door_identifier("%s", safe_field(destination_name(
            ledger.carriage.destination_system_id)).c_str());
         door_label("; freight ");
         print_millitons(ledger.carriage.freight_capacity_millitons);
         door_label("; passengers ");
         door_number("%u", static_cast<unsigned>(ledger.carriage.high_berths + ledger.carriage.middle_berths
                                                 + ledger.carriage.steerage_berths + ledger.carriage.low_berths));
         door_label("; mail ");
         door_value("%s\n\r", ledger.carriage.accept_electronic_mail ? "accepted" : "declined");
      }
      if(show_unavailable) {
         door_identifier("\n\rOffers unavailable to this captain");
         door_label(" (");
         door_number("%zu", available_offers.size());
         door_label(" available hidden)\n\r");
      } else {
         door_identifier("\n\rOffers available here");
         door_label(" (");
         door_number("%zu", unavailable_offers.size());
         door_label(" unavailable hidden)\n\r");
      }
      if(displayed_offers.empty()) {
         door_information("  None\n\r");
      }
      for(size_t i = 0; i < displayed_offers.size(); ++i) {
         const auto& listed = displayed_offers[i];
         const auto& offer = *listed.offer;
         door_number("%zu", i + 1);
         door_label(". ");
         door_value("%s", safe_field(offer.title).c_str());
         door_label("  Cr");
         door_number("%llu", static_cast<unsigned long long>(offer.payment_credits));
         if(offer.quantity_millitons) {
            door_label("  ");
            print_millitons(offer.quantity_millitons);
         }
         if(offer.passenger_count) {
            door_label("  ");
            door_number("%u passenger(s)", offer.passenger_count);
         }
         door_label("\n\r");
         door_label("   Pickup ");
         door_identifier(
            "%s",
            safe_field(destination_name(offer.origin_system_id)).c_str());
         door_label(" slack: ");
         print_pickup_slack(listed.pickup);
         od_printf("\n\r");
         for(const auto& reason : listed.unavailable_reasons) {
            door_warning("   - %s\n\r", safe_field(reason).c_str());
         }
      }
      if(show_unavailable) {
         door_option_prompt({
            "[I] Inspect offer",
            "[A] Accept offer",
            "[V] Available offers",
            "[M] Manage task",
            "[C] Carriage declaration",
            "[Enter] Refresh",
            "[Q] Console",
            "[?] Help",
         });
      } else {
         door_option_prompt({
            "[I] Inspect offer",
            "[A] Accept offer",
            "[V] Unavailable offers",
            "[M] Manage task",
            "[C] Carriage declaration",
            "[Enter] Refresh",
            "[Q] Console",
            "[?] Help",
         });
      }
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == 'v' || key == 'V') {
         show_unavailable = !show_unavailable;
      } else if((key == 'i' || key == 'I') && !displayed_offers.empty()) {
         const auto selected = input_number(
            "Offer", 1, static_cast<unsigned>(displayed_offers.size()));
         if(selected) {
            const auto& listed = displayed_offers[*selected - 1];
            const auto& offer = *listed.offer;
            show_task_offer_detail(
               offer,
               listed.pickup,
               listed.unavailable_reasons);
         }
      } else if((key == 'a' || key == 'A') && !displayed_offers.empty()) {
         const auto selected = input_number(
            "Offer", 1, static_cast<unsigned>(displayed_offers.size()));
         if(selected) {
            const auto& offer = *displayed_offers[*selected - 1].offer;
            try {
               const auto updated = ct::accept_task_offer(
                  connection, session_epoch, offer.offer_id, offer.revision,
                  random_command_id(random), request_id++);
               report_offer_claim(updated, offer.offer_id);
               wait_for_enter();
            } catch(const std::exception& error) {
               door_error("%s\n\r", safe_field(error.what()).c_str());
               wait_for_enter();
            }
         }
      } else if((key == 'm' || key == 'M') && !ledger.tasks.empty()) {
         const auto selected = input_number("Task", 1, static_cast<unsigned>(ledger.tasks.size()));
         if(!selected) {
            continue;
         }
         const auto& task = ledger.tasks[*selected - 1];
         door_option_prompt({
            "[C] Cancel",
            "[R] Return custody",
            "[D] Declare default",
            "[W] Withdraw pending claim",
            "[F] File dispute",
            "[Q] Keep task",
         }, false);
         const auto action_key = od_get_key(TRUE);
         od_printf("\n\r");
         ct::TaskActionKind action;
         std::string explanation;
         if(action_key == 'c' || action_key == 'C') {
            action = ct::TaskActionKind::Cancel;
         } else if(action_key == 'r' || action_key == 'R') {
            action = ct::TaskActionKind::ReturnCustody;
         } else if(action_key == 'd' || action_key == 'D') {
            action = ct::TaskActionKind::DefaultTask;
         } else if(action_key == 'w' || action_key == 'W') {
            action = ct::TaskActionKind::WithdrawClaim;
         } else if(action_key == 'f' || action_key == 'F') {
            action = ct::TaskActionKind::FileDispute;
            const auto entered = input_text("Grounds", "");
            if(!entered) {
               continue;
            }
            explanation = *entered;
         } else {
            continue;
         }
         try {
            ledger = ct::apply_task_action(
                        connection, session_epoch, task.task_id, task.revision, action,
                        explanation, random_command_id(random), request_id++);
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
      } else if(key == 'c' || key == 'C') {
         std::vector<uint64_t> destinations;
         for(const auto& offer : ledger.local_offers) {
            if(std::find(destinations.begin(), destinations.end(),
                         offer.destination_system_id) == destinations.end()) {
               destinations.push_back(offer.destination_system_id);
            }
         }
         door_information("\n\r0. Clear automatic carriage declaration\n\r");
         output().resume_paging();
         for(size_t i = 0; i < destinations.size(); ++i) {
            door_number("%zu", i + 1);
            door_label(". ");
            door_identifier("%s\n\r", safe_field(destination_name(destinations[i])).c_str());
         }
         const auto selected = input_number("Destination", 0, static_cast<unsigned>(destinations.size()));
         if(!selected) {
            continue;
         }
         auto declaration = ledger.carriage;
         declaration.plan_revision = ledger.carriage.plan_revision;
         if(*selected == 0) {
            declaration.destination_system_id = 0;
            declaration.freight_capacity_millitons = 0;
            declaration.high_berths = declaration.middle_berths = declaration.steerage_berths =
            declaration.low_berths = 0;
         } else {
            declaration.destination_system_id = destinations[*selected - 1];
            const auto tons = input_number("Maximum automatic freight tonnes", 0, 1000000,
                                           static_cast<unsigned>(declaration.freight_capacity_millitons / 1000));
            if(!tons) {
               continue;
            }
            declaration.freight_capacity_millitons = static_cast<uint64_t>(*tons) * 1000;
            const auto high = input_number("High passage berths", 0, 65535, declaration.high_berths);
            if(!high) {
               continue;
            }
            declaration.high_berths = static_cast<uint16_t>(*high);
            const auto middle = input_number("Middle passage berths", 0, 65535, declaration.middle_berths);
            if(!middle) {
               continue;
            }
            declaration.middle_berths = static_cast<uint16_t>(*middle);
            const auto steerage = input_number("Steerage berths", 0, 65535, declaration.steerage_berths);
            if(!steerage) {
               continue;
            }
            declaration.steerage_berths = static_cast<uint16_t>(*steerage);
            const auto low = input_number("Low berths", 0, 65535, declaration.low_berths);
            if(!low) {
               continue;
            }
            declaration.low_berths = static_cast<uint16_t>(*low);
            door_prompt("Carry ordinary electronic mail? [Y/n/Q]: ");
            const auto mail = od_get_key(TRUE);
            od_printf("\n\r");
            if(mail == 'q' || mail == 'Q') {
               continue;
            }
            declaration.accept_electronic_mail = !(mail == 'n' || mail == 'N');
         }
         try {
            ledger = ct::set_carriage_declaration(connection, session_epoch, declaration,
                                                  random_command_id(random), request_id++);
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
      }
   }
}

void show_finance(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Finance);
   auto finance = ct::get_finance(
                     connection, session_epoch, random_command_id(random), request_id++);
   const auto destination_assistance_premium = ct::format_number_text(
      "350000", output().display_formatting());
   while(true) {
      od_clr_scr();
      door_heading("Banking and Accounts\n\r");
      door_heading("====================\n\r\n\r");
      door_label("Liquid credits:       ");
      door_number("Cr%llu\n\r", static_cast<unsigned long long>(finance.liquid_credits));
      if(finance.title == ct::ShipTitleKind::InstitutionOwned || finance.restricted_credits != 0) {
         door_label(finance.title == ct::ShipTitleKind::InstitutionOwned
                    ? "Naval service account: "
                    : "Restricted operating:  ");
         door_number("Cr%llu\n\r", static_cast<unsigned long long>(finance.restricted_credits));
      }
      door_label("Reserved credits:     ");
      door_number("Cr%llu\n\r", static_cast<unsigned long long>(finance.reserved_credits));
      door_label("Secured principal:    ");
      door_number("Cr%llu\n\r", static_cast<unsigned long long>(finance.principal_credits));
      door_label("Monthly payment:      ");
      door_number("Cr%llu\n\r", static_cast<unsigned long long>(finance.monthly_payment_credits));
      door_label("Insurance escrow:     ");
      door_number("Cr%llu\n\r", static_cast<unsigned long long>
                  (finance.monthly_insurance_escrow_credits));
      door_label("Destination aid:      ");
      if(finance.destination_assistance_active) {
         door_success("Covered through %s\n\r",
                      game_date(finance.destination_assistance_expires_second).c_str());
      } else {
         door_information("Not covered\n\r");
      }
      door_label("Next payment:         ");
      door_number("%s\n\r", game_date(finance.next_payment_due_second).c_str());
      door_label("Standing:             ");
      if(finance.impound_order_known_locally) {
         door_error("%s\n\r", safe_field(finance.credit_status).c_str());
      } else if(finance.in_default) {
         door_warning("%s\n\r", safe_field(finance.credit_status).c_str());
      } else {
         door_success("%s\n\r", safe_field(finance.credit_status).c_str());
      }
      std::string assistance_option;
      if(finance.destination_assistance_active) {
         assistance_option = "[C] Cancel destination assistance";
      } else {
         assistance_option =
            "[B] Buy one year destination assistance (Cr" +
            destination_assistance_premium + ")";
      }
      std::vector<std::string_view> options{
         assistance_option,
      };
      if(finance.in_default && finance.principal_credits > 0) {
         options.emplace_back("[K] Petition for bankruptcy");
      }
      if(finance.title == ct::ShipTitleKind::InstitutionOwned && finance.restricted_credits != 0) {
         options.emplace_back("[F] Forge expense receipt");
      }
      options.emplace_back("[Enter] Refresh");
      options.emplace_back("[Q] Console");
      options.emplace_back("[?] Help");
      door_option_prompt(options);
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         finance = ct::get_finance(
            connection, session_epoch, random_command_id(random), request_id++);
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if((key == 'f' || key == 'F')
            && finance.title == ct::ShipTitleKind::InstitutionOwned
            && finance.restricted_credits != 0) {
         door_warning(
            "\n\rThis converts naval service credit into personal funds by filing a false "
            "ship-expense receipt. A later accounts audit may uncover the forgery.\n\r");
         const auto amount = input_credit_amount(
            "False receipt amount", finance.restricted_credits);
         if(!amount) {
            continue;
         }
         door_prompt("Forge a receipt for Cr%llu? [y/N]: ",
                     static_cast<unsigned long long>(*amount));
         const auto confirmation = od_get_key(TRUE);
         if(confirmation != 'y' && confirmation != 'Y') {
            continue;
         }
         try {
            finance = ct::misappropriate_restricted_credits(
                         connection, session_epoch, *amount,
                         random_command_id(random), request_id++);
         } catch(const ct::PlayerRequestRejected& error) {
            door_error("\n\r%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter("Banking and Accounts");
         }
         continue;
      }
      if((key == 'k' || key == 'K') && finance.in_default
            && finance.principal_credits > 0) {
         door_warning(
            "\n\rThe court will liquidate every vessel, cargo lot, and financial balance "
            "in this estate. Career standing and legal records survive. The old captain "
            "retires from play and a named successor begins with the original starting "
            "class under a new secured loan.\n\r");
         const auto successor = input_text("Successor captain name", "", 80);
         if(!successor || successor->empty()) {
            continue;
         }
         door_prompt("File this irrevocable petition? [y/N]: ");
         const auto confirmation = od_get_key(TRUE);
         od_printf("\n\r");
         if(confirmation != 'y' && confirmation != 'Y') {
            continue;
         }
         try {
            ct::declare_bankruptcy(
               connection, session_epoch, *successor,
               random_command_id(random), request_id++);
            door_success("The estate is settled. The successor's command is at the berth.\n\r");
            wait_for_enter();
            return;
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
         continue;
      }
      const bool enable = key == 'b' || key == 'B';
      const bool disable = key == 'c' || key == 'C';
      if((enable && !finance.destination_assistance_active)
            || (disable && finance.destination_assistance_active)) {
         if(enable) {
            door_prompt(
               "\n\rPurchase one year of destination assistance for Cr%s? [y/N]: ",
               destination_assistance_premium.c_str());
         } else {
            door_prompt(
               "\n\rCancel destination assistance without a premium refund? [y/N]: ");
         }
         const auto confirmation = od_get_key(TRUE);
         if(confirmation != 'y' && confirmation != 'Y') {
            continue;
         }
         try {
            finance = ct::purchase_insurance(
                         connection, session_epoch,
                         ct::InsuranceKind::DestinationAssistance,
                         enable, random_command_id(random), request_id++);
         } catch(const ct::PlayerRequestRejected& error) {
            door_error("\n\r%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter("Banking and Accounts");
         }
      }
   }
}

void run_shipyard_market(ct::TlsConnection& connection, const uint64_t epoch,
                         ct::CommandIdGenerator& random, uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Shipyard);
   while(true) {
      auto market = ct::get_ship_market(connection, epoch, random_command_id(random), request_id++);
      od_clr_scr();
      door_heading("Shipyard Brokerage\n\r==================\n\r\n\r");
      door_label("Trade-in appraisal: Cr");
      door_number("%llu", static_cast<unsigned long long>(market.current_ship_trade_in_credits));
      door_label("  Lien: Cr");
      door_number("%llu\n\r\n\r", static_cast<unsigned long long>(market.outstanding_lien_credits));
      for(size_t i = 0; i < market.offers.size(); ++i) {
         const auto&o = market.offers[i];
         door_number("%zu", i + 1);
         door_label(". ");
         door_identifier("%s", safe_field(o.class_name).c_str());
         door_label("  J");
         door_number("%u", o.jump_rating);
         door_label("  cargo ");
         print_millitons(o.cargo_capacity_millitons);
         door_label("  Cr");
         door_number("%llu", static_cast<unsigned long long>(o.price_credits));
         if(o.used) {
            door_label("  used ");
            door_number("%u%%", o.visible_condition_percent);
         }
         door_label("\n\r");
      }
      door_option_prompt({
         "[B] Buy with trade-in",
         "[Enter] Refresh",
         "[Q] Port",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if((key == 'b' || key == 'B') && !market.offers.empty()) {
         const auto selected = input_number("Vessel", 1, static_cast<unsigned>(market.offers.size()));
         if(selected) {
            try {
               ct::purchase_ship(connection, epoch, market.offers[*selected - 1].offer_id, true,
                                 random_command_id(random), request_id++);
               door_success("Title transfer completed.\n\r");
               wait_for_enter();
               return;
            } catch(const std::exception& error) {
               door_error("%s\n\r", safe_field(error.what()).c_str());
               wait_for_enter();
            }
         }
      }
   }
}

void run_crew_exchange(ct::TlsConnection& connection, const uint64_t epoch,
                       ct::CommandIdGenerator& random, uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Personnel);
   while(true) {
      auto market = ct::get_crew_market(connection, epoch, random_command_id(random), request_id++);
      od_clr_scr();
      door_heading("Port Crew Exchange\n\r==================\n\r\n\r");
      for(size_t i = 0; i < market.candidates.size(); ++i) {
         const auto&c = market.candidates[i];
         door_number("%zu", i + 1);
         door_label(". ");
         door_identifier("%s", safe_field(c.name).c_str());
         door_label(" — ");
         door_value("%s", safe_field(c.role).c_str());
         door_label("  skill ");
         door_number("%d", static_cast<int>(c.skill_level));
         door_label("  Cr");
         door_number("%llu/month\n\r", static_cast<unsigned long long>(c.monthly_salary_credits));
      }
      door_option_prompt({
         "[H] Hire",
         "[R] Ship roster and personnel actions",
         "[Enter] Refresh",
         "[Q] Port",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if((key == 'h' || key == 'H') && !market.candidates.empty()) {
         const auto selected = input_number("Candidate", 1, static_cast<unsigned>(market.candidates.size()));

         if(selected) {
            try {
               ct::hire_crew(connection, epoch, market.candidates[*selected - 1].candidate_id,
                             random_command_id(random), request_id++);
               door_success("Articles signed; the new crewmember is off watch.\n\r");
               wait_for_enter();
               return;
            } catch(const std::exception& error) {
               door_error("%s\n\r", safe_field(error.what()).c_str());
               wait_for_enter();
            }
         }
      } else if(key == 'r' || key == 'R') {
         show_crew_manager(connection, epoch, random, request_id);
      }
   }
}

const char* message_class_name(const ct::MessageClass message_class)
{
   switch(message_class) {
   case ct::MessageClass::AgencyNews:
      return "News";
   case ct::MessageClass::PublicService:
      return "Public service";
   case ct::MessageClass::ContractOffer:
      return "Offer";
   case ct::MessageClass::TrafficNotice:
      return "Traffic";
   case ct::MessageClass::Private:
      return "Private";
   }
   return "Message";
}

const char* message_importance_name(const ct::MessageImportance importance)
{
   switch(importance) {
   case ct::MessageImportance::Routine:
      return "Routine";
   case ct::MessageImportance::Notable:
      return "Notable";
   case ct::MessageImportance::Important:
      return "Important";
   case ct::MessageImportance::Headline:
      return "Headline";
   }
   return "Routine";
}

const char* message_classification_name(
   const ct::MessageClassification classification)
{
   switch(classification) {
   case ct::MessageClassification::Unreviewed:
      return "Unreviewed";
   case ct::MessageClassification::Ignored:
      return "Ignored";
   case ct::MessageClassification::ReviewLater:
      return "Review later";
   case ct::MessageClassification::Actioned:
      return "Actioned";
   case ct::MessageClassification::Archived:
      return "Archived";
   }
   return "Unreviewed";
}

void change_message_classification(
   ct::TlsConnection& connection,
   uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   uint64_t message_id,
   ct::MessageClassification classification);

std::optional<ct::PlayerPhase> show_combat_operations(
   ct::TlsConnection& connection,
   uint64_t epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id);

void show_known_universe_manager(
   ct::TlsConnection& connection,
   uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id);

std::optional<ct::MessageActionKind> show_message_detail(const ct::MessageItem& item)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Messages);
   while(true) {
   od_clr_scr();
   door_heading("Communications Record\n\r");
   door_heading("=====================\n\r\n\r");
   door_label("Subject:  ");
   door_identifier("%s\n\r", safe_field(item.subject).c_str());
   door_label("Service:  ");
   door_value("%s\n\r", message_class_name(item.message_class));
   door_label("Priority: ");
   door_value("%s\n\r", message_importance_name(item.importance));
   door_label("Origin:   ");
   door_identifier("%s\n\r", safe_field(item.origin_system_name).c_str());
   door_label("Filed:    ");
   door_number("%s\n\r", game_date(item.created_second).c_str());
   door_label("Received: ");
   door_number("%s\n\r", game_date(item.available_second).c_str());
   door_label("Expires:  ");
   door_number("%s", game_date(item.expires_second).c_str());
   if(item.expired) {
      door_warning(" (expired)");
   }
   od_printf("\n\r");
   door_label("Filed as: ");
   door_value("%s\n\r", message_classification_name(item.classification));
   od_printf("\n\r");
   print_wrapped(item.body, "");
   if(item.offer_available && item.offer_id.has_value()) {
      door_option_prompt({
         "[A] Claim signed offer",
         "[Enter] Refresh",
         "[Q] Messages",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key == 'a' || key == 'A') {
         return ct::MessageActionKind::ClaimOffer;
      }
      if(key == 'q' || key == 'Q') {
         return std::nullopt;
      }
      continue;
   }
   const auto action = item.action_kind;
   switch(action) {
   case ct::MessageActionKind::ReviewTask:
      door_option_prompt({
         "[T] Open task ledger", "[Enter] Refresh", "[Q] Messages", "[?] Help"});
      break;
   case ct::MessageActionKind::ReviewFinance:
      door_option_prompt({
         "[B] Open accounts ledger", "[Enter] Refresh", "[Q] Messages", "[?] Help"});
      break;
   case ct::MessageActionKind::ReviewOperations:
      door_option_prompt({
         "[O] Open operations ledger", "[Enter] Refresh", "[Q] Messages", "[?] Help"});
      break;
   case ct::MessageActionKind::ReviewMapping:
      door_option_prompt({
         "[K] Open carried charts", "[Enter] Refresh", "[Q] Messages", "[?] Help"});
      break;
   case ct::MessageActionKind::ClaimOffer:
   case ct::MessageActionKind::None:
      door_option_prompt({"[Enter] Refresh", "[Q] Messages", "[?] Help"});
      break;
   }
   const auto key = od_get_key(TRUE);
   if(key == 'q' || key == 'Q') {
      return std::nullopt;
   }
   if((action == ct::MessageActionKind::ReviewTask && (key == 't' || key == 'T')) ||
         (action == ct::MessageActionKind::ReviewFinance && (key == 'b' || key == 'B')) ||
         (action == ct::MessageActionKind::ReviewOperations && (key == 'o' || key == 'O')) ||
         (action == ct::MessageActionKind::ReviewMapping && (key == 'k' || key == 'K'))) {
      return action;
   }
   }
}

void claim_message_offer(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   const ct::MessageItem& item)
{
   if(!item.offer_id.has_value()) {
      return;
   }
   try {
      const auto ledger = ct::accept_task_offer(
         connection,
         session_epoch,
         *item.offer_id,
         item.offer_revision,
         random_command_id(random),
         request_id++);
      report_offer_claim(ledger, *item.offer_id);
   } catch(const std::exception& error) {
      door_error("%s\n\r", safe_field(error.what()).c_str());
   }
   wait_for_enter();
}

void invoke_message_action(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   const ct::MessageItem& item,
   const ct::MessageActionKind action)
{
   switch(action) {
   case ct::MessageActionKind::ClaimOffer:
      claim_message_offer(connection, session_epoch, random, request_id, item);
      break;
   case ct::MessageActionKind::ReviewTask:
      show_task_manager(connection, session_epoch, random, request_id);
      break;
   case ct::MessageActionKind::ReviewFinance:
      show_finance(connection, session_epoch, random, request_id);
      break;
   case ct::MessageActionKind::ReviewOperations:
      (void)show_combat_operations(
         connection, session_epoch, random, request_id);
      break;
   case ct::MessageActionKind::ReviewMapping:
      show_known_universe_manager(
         connection, session_epoch, random, request_id);
      break;
   case ct::MessageActionKind::None:
      break;
   }
}

void change_message_classification(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   const uint64_t message_id,
   const ct::MessageClassification classification)
{
   (void)ct::set_message_classification(
      connection,
      session_epoch,
      message_id,
      classification,
      random_command_id(random),
      request_id++);
}

void configure_message_filters(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   while(true) {
      const auto snapshot = ct::get_message_management(
                               connection, session_epoch, random_command_id(random), request_id++);
      od_clr_scr();
      door_heading("Arrival Packet Filters\n\r");
      door_heading("======================\n\r\n\r");
      door_information(
         "Messages below a service's threshold remain in the archive but do "
         "not interrupt arrival review.\n\r\n\r");
      for(size_t index = 0; index < snapshot.filters.size(); ++index) {
         const auto& filter = snapshot.filters[index];
         door_number("%zu", index + 1);
         door_label(". ");
         door_value("%-14s", message_class_name(filter.message_class));
         door_label(" at least ");
         door_value("%s\n\r", message_importance_name(filter.minimum_importance));
      }
      door_option_prompt({
         "[1-5] Change service",
         "[Enter] Refresh",
         "[Q] Messages",
         "[?] Help",
      });
      const auto selected = od_get_key(TRUE);
      if(selected == 'q' || selected == 'Q') {
         return;
      }
      if(selected == '\r' || selected == '\n') {
         continue;
      }
      const auto index = static_cast<size_t>(selected - '1');
      if(selected < '1' || selected > '5' || index >= snapshot.filters.size()) {
         continue;
      }
      door_label("Minimum importance\n\r");
      door_option_prompt({
         "[R] Routine",
         "[N] Notable",
         "[I] Important",
         "[H] Headline",
         "[Q] Cancel",
      }, false);
      const auto key = od_get_key(TRUE);
      std::optional<ct::MessageImportance> importance;
      if(key == 'r' || key == 'R') {
         importance = ct::MessageImportance::Routine;
      }
      if(key == 'n' || key == 'N') {
         importance = ct::MessageImportance::Notable;
      }
      if(key == 'i' || key == 'I') {
         importance = ct::MessageImportance::Important;
      }
      if(key == 'h' || key == 'H') {
         importance = ct::MessageImportance::Headline;
      }
      if(importance.has_value()) {
         (void)ct::set_message_filter(
            connection,
            session_epoch,
            snapshot.filters[index].message_class,
            *importance,
            random_command_id(random),
            request_id++);
      }
   }
}

void compose_private_message(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   door_label("Recipient\n\r");
   door_option_prompt({
      "[S] System office", "[C] Captain", "[Q] Cancel", "[?] Help"}, false);
   const auto kind = od_get_key(TRUE);
   od_printf("\n\r");
   if(kind == 'q' || kind == 'Q') {
      return;
   }
   ct::PrivateMessageRequest message{
      .recipient_kind = ct::PrivateRecipientKind::System,
      .destination_system_id = 0,
      .recipient = {},
      .encryption_key_id = 0,
      .ttl_weeks = 1,
      .subject = {},
      .body = {},
   };
   if(kind == 's' || kind == 'S') {
      const auto destination = input_number("Destination system", 1, 4'000'000'000U);
      if(!destination) {
         return;
      }
      message.destination_system_id = *destination;
   } else if(kind == 'c' || kind == 'C') {
      message.recipient_kind = ct::PrivateRecipientKind::Captain;
      const auto bbs = input_number("BBS number", 1, 4'000'000'000U);
      if(!bbs) {
         return;
      }
      const auto player = input_number("Captain number", 1, 4'000'000'000U);
      if(!player) {
         return;
      }
      message.recipient = {
         .bbs_id = *bbs,
         .player_id = *player,
      };
      message.encryption_key_id =
         (static_cast<uint64_t>(*bbs) << 32) | static_cast<uint64_t>(*player);
   } else {
      return;
   }
   const auto ttl = input_number("TTL in weeks", 1, 52, 4);
   if(!ttl) {
      return;
   }
   message.ttl_weeks = static_cast<uint16_t>(*ttl);
   const auto subject = input_text("Subject", "", 120);
   if(!subject) {
      return;
   }
   const auto body = input_text("Message", "", 500);
   if(!body) {
      return;
   }
   message.subject = *subject;
   message.body = *body;
   (void)ct::send_private_message(
      connection, session_epoch, message, random_command_id(random), request_id++);
   door_success("The sealed message was accepted for physical carriage.\n\r");
   wait_for_enter();
}

void show_message_manager(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Messages);
   size_t page = 0;
   constexpr size_t page_size = 7;
   while(true) {
      const auto snapshot = ct::get_message_management(
                               connection,
                               session_epoch,
                               random_command_id(random),
                               request_id++);
      const auto page_count =
         std::max<size_t>(1, (snapshot.items.size() + page_size - 1) / page_size);
      page = std::min(page, page_count - 1);
      const auto first = page * page_size;
      const auto last = std::min(first + page_size, snapshot.items.size());
      od_clr_scr();
      door_heading("Message Management\n\r");
      door_heading("==================\n\r\n\r");
      if(snapshot.items.empty()) {
         door_information("No communications have been received.\n\r");
      } else {
         for(size_t index = first; index < last; ++index) {
            const auto& item = snapshot.items[index];
            door_number("%u", static_cast<unsigned>(index - first + 1));
            door_label(". ");
            door_identifier("%s", safe_field(item.subject).c_str());
            door_label(" [");
            door_value("%s", message_classification_name(item.classification));
            door_label("]\n\r");
         }
      }
      door_label("\n\rPage ");
      door_number("%zu/%zu", page + 1, page_count);
      door_option_prompt({
         "[1-7] Inspect",
         "[N/P] Page",
         "[I] Ignore",
         "[L] Later",
         "[A] Actioned",
         "[R] Archive",
         "[F] Filters",
         "[C] Compose",
         "[Enter] Refresh",
         "[Q] Console",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key >= '1' && key <= '7') {
         const auto index = first + static_cast<size_t>(key - '1');
         if(index < last) {
            if(const auto action = show_message_detail(snapshot.items[index])) {
               invoke_message_action(
                  connection, session_epoch, random, request_id,
                  snapshot.items[index], *action);
            }
         }
      } else if((key == 'n' || key == 'N') && page + 1 < page_count) {
         ++page;
      } else if((key == 'p' || key == 'P') && page > 0) {
         --page;
      } else if(key == 'f' || key == 'F') {
         configure_message_filters(
            connection, session_epoch, random, request_id);
      } else if(key == 'c' || key == 'C') {
         try {
            compose_private_message(
               connection, session_epoch, random, request_id);
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
      } else if(std::string_view{"iIlLaArR"}.find(static_cast<char>(key)) !=
                std::string_view::npos) {
         door_prompt("Message number on this page (1-7), or Q: ");
         const auto selected = od_get_key(TRUE);
         if(selected >= '1' && selected <= '7') {
            const auto index = first + static_cast<size_t>(selected - '1');
            if(index < last) {
               const auto classification =
                  key == 'i' || key == 'I'
                  ? ct::MessageClassification::Ignored
                  : key == 'l' || key == 'L'
                  ? ct::MessageClassification::ReviewLater
                  : key == 'a' || key == 'A'
                  ? ct::MessageClassification::Actioned
                  : ct::MessageClassification::Archived;
               change_message_classification(
                  connection,
                  session_epoch,
                  random,
                  request_id,
                  snapshot.items[index].message_id,
                  classification);
            }
         }
      } else if(key == '\r' || key == '\n') {
         continue;
      } else if(key == 'q' || key == 'Q') {
         return;
      }
   }
}

void show_arrival_packet(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Arrival);
   const auto packet = ct::open_arrival_packet(
                          connection,
                          session_epoch,
                          random_command_id(random),
                          request_id++);
   if(!packet.new_arrival) {
      return;
   }
   std::vector<const ct::MessageItem*> unreviewed;
   for(const auto& item : packet.items) {
      if(!item.previously_seen &&
            item.classification == ct::MessageClassification::Unreviewed) {
         unreviewed.push_back(&item);
      }
   }
   size_t index = 0;
   while(index < unreviewed.size()) {
      const auto& item = *unreviewed[index];
      od_clr_scr();
      door_heading("Arrival Packet - ");
      door_identifier("%s\n\r", safe_field(packet.system_name).c_str());
      door_heading("================\n\r\n\r");
      door_label("Received: ");
      door_number("%zu of %zu\n\r", index + 1, unreviewed.size());
      door_label("Service:  ");
      door_value("%s\n\r", message_class_name(item.message_class));
      door_label("Origin:   ");
      door_identifier("%s\n\r", safe_field(item.origin_system_name).c_str());
      door_label("Age:      ");
      door_number(
         "%llu day(s)\n\r",
         static_cast<unsigned long long>(
            (packet.arrival_second - item.created_second) / (24 * 60 * 60)));
      door_label("Subject:  ");
      door_identifier("%s\n\r", safe_field(item.subject).c_str());
      door_option_prompt({
         "[I/Left] Ignore",
         "[M/Right] Mark for later",
         "[N/Down] Next",
         "[Enter] Inspect",
         "[A] File as actioned",
         "[Q] Stop review",
         "[?] Help",
      });
      const auto key = door_get_translated_key();
      if(key == OD_KEY_LEFT || key == 'i' || key == 'I') {
         change_message_classification(
            connection, session_epoch, random, request_id, item.message_id,
            ct::MessageClassification::Ignored);
         ++index;
      } else if(key == OD_KEY_RIGHT || key == 'm' || key == 'M') {
         change_message_classification(
            connection, session_epoch, random, request_id, item.message_id,
            ct::MessageClassification::ReviewLater);
         ++index;
      } else if(key == OD_KEY_DOWN || key == 'n' || key == 'N') {
         ++index;
      } else if(key == '\r' || key == '\n') {
         if(const auto action = show_message_detail(item)) {
            invoke_message_action(
               connection, session_epoch, random, request_id, item, *action);
            ++index;
         }
      } else if(key == 'a' || key == 'A') {
         change_message_classification(
            connection, session_epoch, random, request_id, item.message_id,
            ct::MessageClassification::Actioned);
         ++index;
      } else if(key == 'q' || key == 'Q') {
         break;
      }
   }
   if(packet.mapping_status.state == ct::SystemMappingState::Unresolved) {
      while(true) {
         od_clr_scr();
         door_heading("System Mapping Notice\n\r");
         door_heading("=====================\n\r\n\r");
         door_information(
            "The ship's records contain no proof that this system has been "
            "published.\n\r");
         door_option_prompt({
            "[P] Send public notice",
            "[D] Send sealed filing to Earth",
            "[W] Withhold",
            "[S] Withhold and mark secret",
            "[Q] Decide later",
            "[?] Help",
         });
         const auto key = od_get_key(TRUE);
         std::optional<ct::SystemMappingChoice> choice;
         if(key == 'p' || key == 'P') {
            choice = ct::SystemMappingChoice::PublicNotification;
         } else if(key == 'd' || key == 'D') {
            choice = ct::SystemMappingChoice::DirectEarth;
         } else if(key == 'w' || key == 'W') {
            choice = ct::SystemMappingChoice::Withhold;
         } else if(key == 's' || key == 'S') {
            choice = ct::SystemMappingChoice::WithholdSecret;
         } else if(key == 'q' || key == 'Q' || key == '\r' || key == '\n') {
            break;
         }
         if(choice) {
            const auto status = ct::set_system_mapping_disclosure(
                                   connection,
                                   session_epoch,
                                   packet.system_id,
                                   *choice,
                                   random_command_id(random),
                                   request_id++);
            door_information("\n\rMapping instructions entered in the ship's log.\n\r");
            if(status.dispatch_message_id) {
               door_label("Dispatch: ");
               door_number(
                  "%llu\n\r",
                  static_cast<unsigned long long>(*status.dispatch_message_id));
            }
            wait_for_enter("Continue");
            break;
         }
      }
   }
   od_clr_scr();
   door_heading("Arrival Communications Receipt\n\r");
   door_heading("==============================\n\r\n\r");
   if(packet.mailbag_id) {
      door_label("Beacon bag:     ");
      door_number(
         "%llu\n\r", static_cast<unsigned long long>(*packet.mailbag_id));
      door_label("Local delivery: ");
      door_number(
         "%llu", static_cast<unsigned long long>(packet.mail_delivered));
      door_label("  Forwarded: ");
      door_number(
         "%llu", static_cast<unsigned long long>(packet.mail_forwarded));
      door_label("  Expired: ");
      door_number(
         "%llu\n\r", static_cast<unsigned long long>(packet.mail_expired));
      door_label("Carrier stipend: Cr");
      door_number(
         "%llu\n\r", static_cast<unsigned long long>(packet.stipend_credits));
   } else {
      door_information("No destination mailbag was carried on this leg.\n\r");
   }
   door_label("New records:    ");
   door_number("%zu\n\r", unreviewed.size());
   wait_for_enter("Continue");
}

double known_system_distance(
   const ct::KnownDestinations& snapshot,
   const std::optional<uint64_t> origin_system_id,
   const ct::KnownSystemSummary& destination)
{
   if(!origin_system_id || *origin_system_id == snapshot.current_system_id) {
      return destination.distance_parsecs;
   }
   const auto origin = std::find_if(
                          snapshot.systems.begin(),
                          snapshot.systems.end(),
   [origin_system_id](const auto & system) {
      return system.system_id == *origin_system_id;
   });
   if(origin == snapshot.systems.end()) {
      return destination.distance_parsecs;
   }
   const auto coreward =
      origin->coreward_parsecs - destination.coreward_parsecs;
   const auto spinward =
      origin->spinward_parsecs - destination.spinward_parsecs;
   const auto north = origin->north_parsecs - destination.north_parsecs;
   return std::sqrt(
      coreward * coreward + spinward * spinward + north * north);
}

void show_planning_system_dossier(
   const ct::KnownDestinations& snapshot,
   const ct::KnownSystemSummary& system,
   const double distance_parsecs)
{
   while(true) {
   od_clr_scr();
   door_heading("System Dossier - ");
   door_identifier("%s\n\r", safe_field(system.system_name).c_str());
   door_heading("================\n\r\n\r");
   door_label("Principal world: ");
   door_value("%s\n\r", safe_field(system.world_name).c_str());
   door_label("Planning range:  ");
   door_number("%.3f parsecs\n\r", distance_parsecs);
   door_label("Direct leg:      ");
   if(distance_parsecs <= static_cast<double>(snapshot.jump_rating) + 1.0e-9) {
      door_success("Within Jump-%u range\n\r", snapshot.jump_rating);
   } else {
      door_warning("Beyond Jump-%u range; a plotted course is required\n\r",
                   snapshot.jump_rating);
   }
   door_label("Starport:        ");
   door_value("%s\n\r", safe_field(system.starport).c_str());
   door_label("Population code: ");
   door_number("%u\n\r", system.population);
   door_label("Tech level:      ");
   door_number("%u\n\r", system.tech_level);
   door_label("Gas giants:      ");
   if(system.gas_giant_count == 0) {
      door_warning("None charted\n\r");
   } else {
      door_success("%u charted\n\r", system.gas_giant_count);
   }
   door_label("Chart received:  ");
   door_number("%s\n\r", game_date(system.observed_second).c_str());
   door_label("Chart source:    ");
   door_value("%s\n\r", safe_field(system.source).c_str());
   door_label("Coordinates:     ");
   door_number("%.6f / %.6f / %.6f pc\n\r",
               system.coreward_parsecs,
               system.spinward_parsecs,
               system.north_parsecs);
   door_information(
      "\n\rPort, population, technical, and gas-giant records may have "
      "changed since this chart was received.\n\r");
   door_option_prompt({
      "[Enter] Refresh",
      "[Q] Destination list",
      "[?] Help",
   });
   const auto key = od_get_key(TRUE);
   if(key == 'q' || key == 'Q') {
      return;
   }
   }
}

std::optional<const ct::KnownSystemSummary*> select_known_primary(
   const ct::KnownDestinations& snapshot,
   const char* title,
   const std::optional<uint64_t> excluded = {},
   const std::optional<uint64_t> distance_origin_system_id = {},
   const bool direct_only = false)
{
   struct Choice {
      const ct::KnownSystemSummary* system;
      double distance_parsecs;
   };
   std::vector<Choice> systems;
   for(const auto& system : snapshot.systems) {
      if(!excluded || system.system_id != *excluded) {
         const auto distance = known_system_distance(
                                  snapshot,
                                  distance_origin_system_id,
                                  system);
         if(!direct_only ||
               distance <= static_cast<double>(snapshot.jump_rating) + 1.0e-9) {
            systems.push_back({&system, distance});
         }
      }
   }
   std::sort(systems.begin(), systems.end(), [](const auto& left, const auto& right) {
      if(left.distance_parsecs != right.distance_parsecs) {
         return left.distance_parsecs < right.distance_parsecs;
      }
      return left.system->system_id < right.system->system_id;
   });
   if(systems.empty()) {
      door_warning("No charted destination matches this leg.\n\r");
      wait_for_enter();
      return std::nullopt;
   }
   size_t page = 0;
   const size_t page_size = output().columns() < 64 ? 4 : 6;
   while(true) {
      const auto page_count =
         std::max<size_t>(1, (systems.size() + page_size - 1) / page_size);
      page = std::min(page, page_count - 1);
      const auto first = page * page_size;
      const auto last = std::min(first + page_size, systems.size());
      od_clr_scr();
      door_heading("%s\n\r", title);
      door_heading("=====================\n\r\n\r");
      for(size_t index = first; index < last; ++index) {
         const auto& choice = systems[index];
         const auto& system = *choice.system;
         door_number("%u", static_cast<unsigned>(index - first + 1));
         door_label(". ");
         door_identifier("%s", safe_field(system.system_name).c_str());
         door_label(" / ");
         door_value("%s", safe_field(system.world_name).c_str());
         if(system.system_id == snapshot.current_system_id) {
            door_information("  (present system)");
         }
         od_printf("\n\r");
         door_label("   ");
         door_number("%.3f pc", choice.distance_parsecs);
         door_label("  Port ");
         door_value("%s", safe_field(system.starport).c_str());
         door_label("  Pop ");
         door_number("%u", system.population);
         door_label("  TL");
         door_number("%u", system.tech_level);
         door_label("  Gas giants ");
         if(system.gas_giant_count == 0) {
            door_warning("0\n\r");
         } else {
            door_success("%u\n\r", system.gas_giant_count);
         }
      }
      if(page_size == 4) {
         door_option_prompt({
            "[1-4] Select",
            "[I] Dossier",
            "[< >] Page",
            "[Enter/Q] Cancel",
            "[?] Help",
         });
      } else {
         door_option_prompt({
            "[1-6] Select",
            "[I] Dossier",
            "[< >] Page",
            "[Enter/Q] Cancel",
            "[?] Help",
         });
      }
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n' || key == 'q' || key == 'Q') {
         return std::nullopt;
      }
      if(key == '>' && page + 1 < page_count) {
         ++page;
      } else if(key == '<' && page > 0) {
         --page;
      } else if(key == 'i' || key == 'I') {
         const auto selected = input_number(
                                  "Dossier entry",
                                  1,
                                  static_cast<unsigned>(last - first));
         if(selected) {
            const auto& choice = systems[first + *selected - 1];
            show_planning_system_dossier(
               snapshot, *choice.system, choice.distance_parsecs);
         }
      } else if(key >= '1' &&
                key < static_cast<int>('1' + page_size)) {
         const auto index = first + static_cast<size_t>(key - '1');
         if(index < last) {
            return systems[index].system;
         }
      }
   }
}

std::string course_duration(const uint64_t seconds)
{
   return ct::format_game_duration(
      seconds, output().display_formatting());
}

std::string game_date(const uint64_t seconds)
{
   return ct::format_game_timestamp(
      seconds, output().display_formatting());
}

const char* course_fuel_source_name(const ct::CourseFuelSource source)
{
   switch(source) {
   case ct::CourseFuelSource::None:
      return "none";
   case ct::CourseFuelSource::Carried:
      return "fuel aboard";
   case ct::CourseFuelSource::RefinedPort:
      return "buy refined fuel";
   case ct::CourseFuelSource::FrontierSkimming:
      return "skim and process fuel";
   case ct::CourseFuelSource::UnrefinedPort:
      return "buy unrefined fuel";
   }
   return "unknown";
}

void show_course_plot(const ct::CoursePlot& plot)
{
   bool fastest = true;
   size_t page = 0;
   constexpr size_t page_size = 6;
   while(true) {
      const auto& plan = fastest ? plot.fastest : plot.cheapest;
      const auto page_count = std::max<size_t>(
                                 1, (plan.waypoints.size() + page_size - 1) / page_size);
      page = std::min(page, page_count - 1);
      const auto first = page * page_size;
      const auto last = std::min(first + page_size, plan.waypoints.size());
      od_clr_scr();
      door_heading("Course Plot - %s\n\r", fastest ? "Fastest" : "Cheapest");
      door_heading("=======================\n\r");
      door_label("Drive: ");
      door_identifier("Jump-%u\n\r", plot.jump_rating);
      if(plan.available) {
         const auto eta_second =
            plan.elapsed_seconds >
               std::numeric_limits<uint64_t>::max() - plot.current_game_second
            ? std::numeric_limits<uint64_t>::max()
            : plot.current_game_second + plan.elapsed_seconds;
         door_label("Current:   ");
         door_number("%s\n\r", game_date(plot.current_game_second).c_str());
         door_label("ETA:       ");
         door_number("%s\n\r", game_date(eta_second).c_str());
         door_label("Trip time: ");
         door_number("%s", course_duration(plan.elapsed_seconds).c_str());
         door_label(" (wall time ");
         door_number(
            "%s",
            wall_duration(
               plan.elapsed_seconds,
               plot.clock_rate_game_seconds,
               plot.clock_rate_real_seconds).c_str());
         door_label(")\n\r");
         door_label("Fuel purchases: ");
         door_number("Cr%llu", static_cast<unsigned long long>(plan.fuel_cost_credits));
         door_label("  Distance: ");
         door_number("%.3f pc\n\r\n\r", plan.total_milliparsecs / 1000.0);
         for(size_t index = first; index < last; ++index) {
            const auto& waypoint = plan.waypoints[index];
            door_number("%u", static_cast<unsigned>(index + 1));
            door_label(". ");
            door_identifier("%s", safe_field(waypoint.system_name).c_str());
            door_label(" / ");
            door_value("%s\n\r", safe_field(waypoint.world_name).c_str());
            if(waypoint.next_leg_milliparsecs != 0) {
               door_label("   ");
               door_information("%s", course_fuel_source_name(waypoint.fuel_source));
               door_label("; next jump ");
               door_number("%.3f pc\n\r", waypoint.next_leg_milliparsecs / 1000.0);
            }
         }
      } else {
         door_warning(
            "No course through known systems can satisfy this ship's range "
            "and refueling requirements.\n\r");
      }
      door_information(
         "\n\rEstimate includes purchased port fuel and mean frontier "
         "skimming/processing time. Payroll, maintenance, fees, hazards, and "
         "encounter delays are excluded.\n\r");
      door_option_prompt({
         "[F] Fastest",
         "[C] Cheapest",
         "[< >] Page",
         "[Enter] Refresh",
         "[Q] Charts",
         "[?] Help",
      }, false);
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == 'f' || key == 'F') {
         fastest = true;
         page = 0;
      } else if(key == 'c' || key == 'C') {
         fastest = false;
         page = 0;
      } else if(key == '>' && page + 1 < page_count) {
         ++page;
      } else if(key == '<' && page > 0) {
         --page;
      }
   }
}

void run_course_plotter(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id,
   const ct::KnownDestinations& snapshot)
{
   const HelpScope help_scope(ct::DoorHelpTopic::FlightPlan);
   od_clr_scr();
   door_heading("Navigation Course Plotter\n\r");
   door_heading("=========================\n\r\n\r");
   door_number("C");
   door_label(". ");
   door_identifier("Present location to a known primary\n\r");
   door_number("P");
   door_label(". ");
   door_identifier("Known primary to known primary\n\r");
   door_option_prompt({
      "[C/P] Plot type",
      "[Enter/Q] Charts",
      "[?] Help",
   });
   const auto mode = od_get_key(TRUE);
   if(mode == '\r' || mode == '\n' || mode == 'q' || mode == 'Q') {
      return;
   }
   const bool use_current = mode == 'c' || mode == 'C';
   if(!use_current && mode != 'p' && mode != 'P') {
      return;
   }
   const auto origin = use_current
                       ? std::optional<uint64_t> {snapshot.current_system_id}
                       :
   [&]() -> std::optional<uint64_t> {
      const auto selected = select_known_primary(snapshot, "Select Course Origin");
return selected ? std::optional<uint64_t>{(*selected)->system_id} :
      std::nullopt;
   }();
   if(!origin) {
      return;
   }
   const auto destination =
      select_known_primary(
         snapshot,
         "Select Course Destination",
         *origin,
         *origin);
   if(!destination) {
      return;
   }
   const auto plot = ct::plot_course(
                        connection,
                        session_epoch,
                        *origin,
                        (*destination)->system_id,
                        use_current,
                        random_command_id(random),
                        request_id++);
   show_course_plot(plot);
}

void show_known_universe_manager(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::KnownUniverse);
   auto snapshot = ct::get_known_destinations(
                      connection,
                      session_epoch,
                      random_command_id(random),
                      request_id++);
   bool reachable_only = false;
   size_t page = 0;
   while(true) {
      std::vector<const ct::KnownSystemSummary*> systems;
      systems.reserve(snapshot.systems.size());
      for(const auto& system : snapshot.systems) {
         if(!reachable_only || system.within_jump_rating) {
            systems.push_back(&system);
         }
      }
      constexpr size_t page_size = 9;
      const auto page_count =
         std::max<size_t>(1, (systems.size() + page_size - 1) / page_size);
      page = std::min(page, page_count - 1);
      const auto first = page * page_size;
      const auto last = std::min(first + page_size, systems.size());

      od_clr_scr();
      door_heading("Ship's Navigation Library\n\r");
      door_heading("=========================\n\r\n\r");
      door_label("Chart selection: ");
      door_identifier("%s", reachable_only ? "direct jumps" : "all known systems");
      door_label("  Drive: ");
      door_identifier("Jump-%u", snapshot.jump_rating);
      door_label("  Page ");
      door_number("%u/%u\n\r\n\r",
                  static_cast<unsigned>(page + 1),
                  static_cast<unsigned>(page_count));
      if(systems.empty()) {
         door_information("No systems match this chart selection.\n\r");
      }
      for(size_t index = first; index < last; ++index) {
         const auto& system = *systems[index];
         door_number("%u", static_cast<unsigned>(index - first + 1));
         door_label(". ");
         if(system.system_id == snapshot.current_system_id) {
            door_information("* ");
         } else if(system.remote_candidate) {
            door_warning("? ");
         } else if(system.within_jump_rating) {
            door_success("J ");
         } else {
            door_label("- ");
         }
         door_identifier("%s", safe_field(system.system_name).c_str());
         door_label(" / ");
         door_value("%s", safe_field(system.world_name).c_str());
         if(output().columns() >= 64) {
            door_label("  ");
            door_number("%.3f pc", system.distance_parsecs);
            door_label("  Port ");
            door_value("%s", safe_field(system.starport).c_str());
            door_label(" TL");
            door_number("%u", system.tech_level);
         }
         od_printf("\n\r");
      }
      door_information(
         "\n\r* marks the present system; J marks a direct jump; ? marks a "
         "newly resolved survey contact.\n\r");
      door_option_prompt({
         "[1-9] Dossier",
         "[P] Plot",
         "[M] Markets",
         "[J] Direct/all",
         "[< >] Page",
         "[Enter] Refresh",
         "[Q] Console",
         "[?] Help",
      }, false);
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         snapshot = ct::get_known_destinations(
            connection, session_epoch, random_command_id(random), request_id++);
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == 'j' || key == 'J') {
         reachable_only = !reachable_only;
         page = 0;
         continue;
      }
      if(key == 'p' || key == 'P') {
         run_course_plotter(connection, session_epoch, random, request_id, snapshot);
         continue;
      }
      if(key == 'm' || key == 'M') {
         while(true) {
            const auto knowledge = ct::get_market_knowledge(
               connection, session_epoch, random_command_id(random), request_id++);
            od_clr_scr();
            door_heading("Carried Market Reports\n\r======================\n\r\n\r");
            if(knowledge.observations.empty()) {
               door_information("No market observations have been entered.\n\r");
            }
            for(const auto& report : knowledge.observations) {
               door_identifier("%s / %s\n\r", safe_field(report.system_name).c_str(),
                               safe_field(report.commodity_name).c_str());
               door_label("  Observed: ");
               door_number("%s", game_date(report.observed_second).c_str());
               door_label("; Cr");
               door_number(
                  "%llu-%llu/t",
                  static_cast<unsigned long long>(report.minimum_price_per_ton),
                  static_cast<unsigned long long>(report.maximum_price_per_ton));
               door_label("; confidence ");
               door_number("%u%%\n\r", report.confidence_percent);
            }
            door_option_prompt({"[Enter] Refresh", "[Q] Charts", "[?] Help"});
            const auto report_key = od_get_key(TRUE);
            if(report_key == 'q' || report_key == 'Q') {
               break;
            }
         }
         continue;
      }
      if(key == '>' && page + 1 < page_count) {
         ++page;
         continue;
      }
      if(key == '<' && page > 0) {
         --page;
         continue;
      }
      if(key < '1' || key > '9') {
         continue;
      }
      const auto index = first + static_cast<size_t>(key - '1');
      if(index >= last) {
         continue;
      }
      const auto& system = *systems[index];
      while(true) {
      od_clr_scr();
      door_heading("System Dossier - ");
      door_identifier("%s\n\r", safe_field(system.system_name).c_str());
      door_heading("================\n\r\n\r");
      door_label("Principal world: ");
      door_value("%s\n\r", safe_field(system.world_name).c_str());
      door_label("Range:           ");
      door_number("%.3f parsecs\n\r", system.distance_parsecs);
      door_label("Navigation:      ");
      if(system.system_id == snapshot.current_system_id) {
         door_information("Present system\n\r");
      } else if(system.within_jump_rating) {
         door_success("Direct Jump-%u passage available\n\r", snapshot.jump_rating);
      } else {
         door_warning("Beyond this ship's direct jump range\n\r");
      }
      door_label("Starport:        ");
      door_value("%s\n\r", safe_field(system.starport).c_str());
      door_label("Population code: ");
      door_number("%u\n\r", system.population);
      door_label("Tech level:      ");
      door_number("%u\n\r", system.tech_level);
      door_label("Gas giants:      ");
      if(system.gas_giant_count == 0) {
         door_warning("None charted\n\r");
      } else {
         door_success("%u charted\n\r", system.gas_giant_count);
      }
      door_label("Chart received:  ");
      door_number("%s\n\r", game_date(system.observed_second).c_str());
      door_label("Chart source:    ");
      door_value("%s\n\r", safe_field(system.source).c_str());
      door_label("Coordinates:     ");
      door_number("%.6f / %.6f / %.6f pc\n\r",
                  system.coreward_parsecs,
                  system.spinward_parsecs,
                  system.north_parsecs);
      door_information(
         "\n\rNavigation, port, population, technical, and gas-giant reports "
         "may have changed since this chart was received.\n\r");
      const bool secret = system.knowledge_source ==
         ct::SystemKnowledgeSource::SecretChart;
      door_option_prompt({
         secret ? "[S] Remove from Secret Systems" : "[S] Add to Secret Systems",
         "[Enter] Refresh",
         "[Q] Return",
         "[?] Help",
      });
      const auto detail_key = od_get_key(TRUE);
      if(detail_key == 'q' || detail_key == 'Q') {
         break;
      }
      if(detail_key == '\r' || detail_key == '\n') {
         continue;
      }
      if(detail_key == 's' || detail_key == 'S') {
         try {
            ct::set_system_mapping_disclosure(
               connection,
               session_epoch,
               system.system_id,
               secret ? ct::SystemMappingChoice::Withhold
               : ct::SystemMappingChoice::WithholdSecret,
               random_command_id(random),
               request_id++);
            snapshot = ct::get_known_destinations(
                          connection,
                          session_epoch,
                          random_command_id(random),
                          request_id++);
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
         break;
      }
      }
   }
}

const char* career_mode_name(const ct::CombatCareerMode mode)
{
   switch(mode) {
   case ct::CombatCareerMode::Independent:
      return "Independent";
   case ct::CombatCareerMode::Navy:
      return "Naval service";
   case ct::CombatCareerMode::Privateer:
      return "Privateer";
   case ct::CombatCareerMode::Pirate:
      return "Pirate";
   }
   return "Unknown";
}

const char* career_order_state_name(const ct::CareerOpportunityState state)
{
   switch(state) {
   case ct::CareerOpportunityState::Offered:
      return "offered";
   case ct::CareerOpportunityState::Accepted:
      return "under orders";
   case ct::CareerOpportunityState::Reporting:
      return "report in transit";
   case ct::CareerOpportunityState::Succeeded:
      return "settled";
   case ct::CareerOpportunityState::Failed:
      return "failed";
   case ct::CareerOpportunityState::Expired:
      return "expired";
   }
   return "unknown";
}

std::optional<ct::PlayerPhase> show_combat_operations(
   ct::TlsConnection& connection, const uint64_t epoch,
   ct::CommandIdGenerator& random, uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Operations);
   auto snapshot = ct::get_combat_career(connection, epoch, random_command_id(random), request_id++);
   auto charts = ct::get_known_destinations(
      connection, epoch, random_command_id(random), request_id++);
   const auto system_name = [&charts](const uint64_t system_id) {
      const auto found = std::find_if(
         charts.systems.begin(), charts.systems.end(),
         [system_id](const auto& system) { return system.system_id == system_id; });
      return found == charts.systems.end()
             ? std::string("an unlisted jurisdiction")
             : found->system_name;
   };
   while(true) {
      od_clr_scr();
      door_heading("Operations Ledger\n\r=================\n\r\n\r");
      door_label("Service: ");
      door_identifier("%s", career_mode_name(snapshot.mode));
      door_label("  Standing: ");
      door_value("%s\n\r", safe_field(snapshot.rank).c_str());
      if(snapshot.mode == ct::CombatCareerMode::Navy) {
         door_label("Pay: Cr");
         door_number("%llu/month", static_cast<unsigned long long>(snapshot.monthly_salary_credits));
         door_label("  Service points: ");
         door_number("%u\n\r", snapshot.service_points);
      } else {
         door_label("Public heat: ");
         door_number("%u", snapshot.public_heat);
         door_label("  Underworld standing: ");
         door_number("%d\n\r", snapshot.underworld_standing);
         if(snapshot.mode == ct::CombatCareerMode::Pirate) {
            door_label("Crew pressure: ");
            door_number("%u/100\n\r", snapshot.crew_pressure);
         }
      }
      door_information("%s\n\r", safe_field(snapshot.local_enforcement_summary).c_str());
      if(snapshot.interception_watch) {
         const auto& watch = *snapshot.interception_watch;
         door_label("Interception watch: ");
         door_identifier("%s ", watch.purpose == ct::InterceptionPurpose::BoardingInspection
                                      ? "board/inspect"
                                      : "armed attack");
         switch(watch.filter) {
         case ct::InterceptionWatchFilterKind::NamedVessel:
            door_warning("departure of %s", safe_field(watch.target_ship_name).c_str());
            break;
         case ct::InterceptionWatchFilterKind::CraftClass:
            door_warning("all %s craft", safe_field(watch.target_ship_name).c_str());
            break;
         case ct::InterceptionWatchFilterKind::AllCraft:
            door_warning("all craft");
            break;
         }
         door_label("  since ");
         door_value("%s\n\r", game_date(watch.started_second).c_str());
      }
      door_identifier("\n\rOrders and intelligence\n\r");
      if(snapshot.opportunities.empty()) {
         door_information("  Nothing actionable is posted.\n\r");
      }
      for(size_t i = 0; i < snapshot.opportunities.size(); ++i) {
         const auto&o = snapshot.opportunities[i];
         door_number("%zu", i + 1);
         door_label(". ");
         door_value("%s", safe_field(o.objective).c_str());
         door_label("  [");
         door_identifier("%s", career_order_state_name(o.state));
         door_label("]");
         door_label("  at ");
         door_identifier("%s", safe_field(system_name(o.target_system_id)).c_str());
         if(o.reward_credits) {
            door_label("  Cr");
            door_number("%llu", static_cast<unsigned long long>(o.reward_credits));
         }
         if(o.service_points) {
            door_label("  +");
            door_number("%u", o.service_points);
            door_label(" service");
         }
         door_label("\n\r");
      }
      door_identifier("\n\rSystem traffic control\n\r");
      if(snapshot.system_contacts.empty()) {
         if(snapshot.phase == ct::PlayerPhase::Jump) {
            door_information("  No traffic-control reception is possible in Jump space.\n\r");
         } else {
            door_information("  No active transponder movement is reported.\n\r");
         }
      }
      for(const auto&c : snapshot.system_contacts) {
         if(c.online_controlled) {
            door_success("  [ONLINE] ");
         } else if(c.player_owned) {
            door_identifier("  [PLAYER] ");
         } else {
            door_label("  ");
         }
         door_identifier("%s", safe_field(c.ship_name).c_str());
         if(c.attachment == ct::TrafficAttachment::Berthed) {
            door_warning("  [BERTHED]");
         } else if(c.attachment == ct::TrafficAttachment::Landed) {
            door_warning("  [LANDED]");
         }
         door_label("  [");
         door_value("%s", safe_field(c.transponder).c_str());
         door_label("]\n\r");
         print_wrapped_field(
            "    Registry: ",
            safe_field(c.operator_name) + " — " + safe_field(c.role));
         door_label("    Status: ");
         if(c.movement == ct::TrafficMovementKind::Present) {
            door_value("present in system\n\r");
         } else {
            door_value("%s  %s\n\r",
                       c.movement == ct::TrafficMovementKind::Arrival
                       ? "arrival"
                       : "departure",
                       game_date(c.edge_second).c_str());
         }
      }
      door_identifier("\n\rLocal contacts\n\r");
      if(snapshot.local_contacts.empty()) {
         if(snapshot.phase == ct::PlayerPhase::Jump) {
            door_information("  No local sensor picture is possible in Jump space.\n\r");
         } else if(snapshot.phase == ct::PlayerPhase::Interplanetary) {
            door_information("  No contact resolves at the ship's present position.\n\r");
         } else {
            door_information("  No contact remains at this traffic locus.\n\r");
         }
      }
      for(size_t i = 0; i < snapshot.local_contacts.size(); ++i) {
         const auto&c = snapshot.local_contacts[i];
         door_number("%zu", i + 1);
         door_label(". ");
         if(c.online_controlled) {
            door_success("[ONLINE] ");
         } else if(c.player_owned) {
            door_identifier("[PLAYER] ");
         }
         door_identifier("%s", safe_field(c.ship_name).c_str());
         if(c.attachment == ct::TrafficAttachment::Berthed) {
            door_warning("  [BERTHED]");
         } else if(c.attachment == ct::TrafficAttachment::Landed) {
            door_warning("  [LANDED]");
         }
         door_label("  [");
         door_value("%s", safe_field(c.transponder).c_str());
         door_label("]\n\r");
         print_wrapped_field(
            "   Registry: ",
            safe_field(c.operator_name) + " — " + safe_field(c.role));
         switch(c.resolution) {
         case ct::TrafficContactResolution::TransponderOnly:
            door_label("   Sensors:  ");
            door_warning("No reliable hull solution; transponder data only.\n\r");
            break;
         case ct::TrafficContactResolution::Approximate:
            door_label("   Sensors:  ");
            door_warning("%s, approximately %.0f t (%u%% confidence)\n\r",
                         safe_field(c.class_name).c_str(),
                         c.displacement_millitons / 1000.0,
                         c.confidence_percent);
            break;
         case ct::TrafficContactResolution::Identified:
            door_label("   Sensors:  ");
            door_value("%s, %.1f t (%u%% confidence)\n\r",
                       safe_field(c.class_name).c_str(),
                       c.displacement_millitons / 1000.0,
                       c.confidence_percent);
            break;
         }
      }
      if(!snapshot.prizes.empty()) {
         door_identifier("\n\rPrizes\n\r");
         for(size_t i = 0; i < snapshot.prizes.size(); ++i) {
            const auto&p = snapshot.prizes[i];
            door_number("%zu", i + 1);
            door_label(". ");
            door_value("%s", safe_field(p.name).c_str());
            door_label("  condition ");
            door_number("%u%%", p.condition_percent);
            door_label("  crew held ");
            door_number("%u", p.surviving_crew_count);
            door_label("  award Cr");
            door_number("%llu\n\r", static_cast<unsigned long long>(p.settlement_credits));
         }
      }
      if(!snapshot.warrants.empty()) {
         door_identifier("\n\rWarrants\n\r");
         for(size_t i = 0; i < snapshot.warrants.size(); ++i) {
            const auto&w = snapshot.warrants[i];
            door_number("%zu", i + 1);
            door_label(". ");
            door_value("%s", safe_field(w.accusation).c_str());
            door_label("  bond basis Cr");
            door_number("%llu", static_cast<unsigned long long>(w.bounty_credits));
            if(w.resolution_message_id != 0) {
               door_label("  satisfaction notice ");
               door_identifier("in transit from %s",
                  safe_field(system_name(w.resolving_system_id)).c_str());
            }
            door_label("\n\r");
         }
      }
      door_option_prompt({
         "[A] Accept order or file report",
         "[I] Intercept traffic",
         "[P] Prize office",
         "[S] Standing interception order",
         "[W] Warrant court",
         "[C] Cruise articles",
         "[M] Service or commission status",
         "[Enter] Refresh",
         "[Q] Console",
         "[?] Help",
      });
      const auto key = static_cast<char>(std::toupper(static_cast<unsigned char>(od_get_key(TRUE))));
      if(key == '\r' || key == '\n') {
         snapshot = ct::get_combat_career(
            connection, epoch, random_command_id(random), request_id++);
         charts = ct::get_known_destinations(
            connection, epoch, random_command_id(random), request_id++);
         continue;
      }
      if(key == 'Q') {
         return std::nullopt;
      }
      try {
         if(key == 'A' && !snapshot.opportunities.empty()) {
            const auto selected = input_number("Entry", 1,
                                               static_cast<unsigned>(snapshot.opportunities.size()));
            if(selected) {
               snapshot = ct::accept_career_opportunity(connection, epoch,
                  snapshot.opportunities[*selected - 1].opportunity_id, snapshot.revision, random_command_id(random),
                  request_id++);
            }
         } else if(key == 'I' && !snapshot.local_contacts.empty()) {
            const auto selected = input_number("Contact", 1,
                                               static_cast<unsigned>(snapshot.local_contacts.size()));
            if(selected) {
               const auto& target = snapshot.local_contacts[*selected - 1];
               if(target.player_owned) {
                  door_warning(
                     "This is another player's real vessel. Combat can kill its crew, destroy "
                     "cargo, or transfer the ship by surrender or boarding.\n\r");
                  if(target.online_controlled) {
                     door_information("Its captain is presently commanding it online.\n\r");
                  } else {
                     door_information("It will fight under its captain's standing combat policy.\n\r");
                  }
               }
               const auto attached = target.attachment != ct::TrafficAttachment::Spaceborne;
               door_option_prompt({"[A] Armed attack", "[B] Board or inspect", "[Q] Cancel"}, false);
               const auto intent_key = static_cast<char>(std::toupper(
                  static_cast<unsigned char>(od_get_key(TRUE))));
               od_printf("\n\r");
               if(intent_key != 'A' && intent_key != 'B') {
                  continue;
               }
               const auto purpose = intent_key == 'B'
                  ? ct::InterceptionPurpose::BoardingInspection
                  : ct::InterceptionPurpose::ArmedAttack;
               if(snapshot.phase == ct::PlayerPhase::Docked) {
                  door_information(
                     "Your ship will clear its berth and pay any charges due before taking station.\n\r");
               }
               if(attached) {
                  door_information(
                     "The target is %s. Your ship will clear any berth, pay charges due, "
                     "and wait at this locus until it departs.\n\r",
                     target.attachment == ct::TrafficAttachment::Berthed ? "berthed" : "landed");
               }
               if(purpose == ct::InterceptionPurpose::BoardingInspection) {
                  door_information(
                     "Combat begins only if the vessel refuses the boarding order. An unlawful "
                     "demand can still produce a warrant even if it complies.\n\r");
               } else {
                  door_warning("An armed %s is an irreversible act.\n\r",
                               attached ? "departure watch" : "intercept");
               }
               door_option_prompt({attached ? "[W] Confirm watch" : "[I] Confirm intercept",
                                   "[Q] Cancel"}, false);
               const auto confirm = static_cast<char>(std::toupper(static_cast<unsigned char>(od_get_key(TRUE))));
               od_printf("\n\r");
               if(confirm == (attached ? 'W' : 'I')) {
                  auto result = ct::engage_traffic_contact(connection, epoch,
                     target.contact_id, snapshot.revision, purpose, random_command_id(random),
                     request_id++);
                  if(std::holds_alternative<ct::CombatSnapshot>(result)) {
                     return std::get<ct::CombatSnapshot>(result).phase;
                  }
                  if(std::holds_alternative<ct::EncounterResult>(result)) {
                     door_information("%s\n\r",
                        safe_field(std::get<ct::EncounterResult>(result).outcome).c_str());
                     wait_for_enter("Continue");
                     snapshot = ct::get_combat_career(
                        connection, epoch, random_command_id(random), request_id++);
                  } else {
                     snapshot = std::get<ct::CombatCareerSnapshot>(std::move(result));
                  }
               }
            }
         } else if(key == 'S') {
            std::vector<const ct::TrafficContact*> classes;
            const auto add_class = [&classes](const ct::TrafficContact& contact) {
               if(contact.catalog_id != 0 && std::none_of(
                     classes.begin(), classes.end(), [&contact](const auto* existing) {
                        return existing->catalog_id == contact.catalog_id;
                     })) {
                  classes.push_back(&contact);
               }
            };
            for(const auto& contact : snapshot.local_contacts) {
               add_class(contact);
            }
            for(const auto& contact : snapshot.system_contacts) {
               add_class(contact);
            }
            door_option_prompt({"[A] Armed-attack watch", "[B] Boarding/inspection watch",
                                "[R] Remove watch", "[Q] Cancel"}, false);
            const auto purpose_key = static_cast<char>(std::toupper(
               static_cast<unsigned char>(od_get_key(TRUE))));
            od_printf("\n\r");
            if(purpose_key == 'R' && snapshot.interception_watch) {
               snapshot = ct::set_interception_watch(
                  connection, epoch, ct::InterceptionWatchSelection::Cancel, 0,
                  ct::InterceptionPurpose::ArmedAttack, snapshot.revision,
                  random_command_id(random), request_id++);
               continue;
            }
            if(purpose_key != 'A' && purpose_key != 'B') {
               continue;
            }
            const auto purpose = purpose_key == 'B'
               ? ct::InterceptionPurpose::BoardingInspection
               : ct::InterceptionPurpose::ArmedAttack;
            door_option_prompt({"[A] All craft", "[C] Observed craft class",
                                "[Q] Cancel"}, false);
            const auto action = static_cast<char>(std::toupper(
               static_cast<unsigned char>(od_get_key(TRUE))));
            od_printf("\n\r");
            if(action == 'A') {
               snapshot = ct::set_interception_watch(
                  connection, epoch, ct::InterceptionWatchSelection::AllCraft, 0,
                  purpose,
                  snapshot.revision, random_command_id(random), request_id++);
            } else if(action == 'C') {
               if(classes.empty()) {
                  door_warning("No catalogued craft class is present in the current traffic picture.\n\r");
                  wait_for_enter("Continue");
                  continue;
               }
               for(size_t index = 0; index < classes.size(); ++index) {
                  door_number("%zu", index + 1);
                  door_label(". ");
                  door_value("%s\n\r", safe_field(classes[index]->class_name).c_str());
               }
               const auto selected = input_number(
                  "Craft class", 1, static_cast<unsigned>(classes.size()));
               if(selected) {
                  snapshot = ct::set_interception_watch(
                     connection, epoch, ct::InterceptionWatchSelection::CraftClass,
                     classes[*selected - 1]->catalog_id, purpose, snapshot.revision,
                     random_command_id(random), request_id++);
               }
            }
         } else if(key == 'P' && !snapshot.prizes.empty()) {
            const auto selected = input_number("Prize", 1, static_cast<unsigned>(snapshot.prizes.size()));
            if(!selected) {
               continue;
            }
            door_option_prompt({
               "[F] File claim",
               "[A] Take advance",
               "[C] Court sale",
               "[K] Keep awarded vessel",
               "[S] Sell to fence",
               "[L] Launder registry",
               "[Q] Cancel",
            }, false);
            const auto action = static_cast<char>(std::toupper(static_cast<unsigned char>(od_get_key(TRUE))));
            od_printf("\n\r");
            if(action == 'Q') {
               continue;
            }
            auto method = ct::PrizeSettlementMethod::FileClaim;
            if(action == 'A') {
               method = ct::PrizeSettlementMethod::TakeAdvance;
            } else if(action == 'C') {
               method = ct::PrizeSettlementMethod::CourtSale;
            } else if(action == 'K') {
               method = ct::PrizeSettlementMethod::KeepPrize;
            } else if(action == 'S') {
               method = ct::PrizeSettlementMethod::Fence;
            } else if(action == 'L') {
               method = ct::PrizeSettlementMethod::LaunderRegistry;
            } else if(action != 'F') {
               continue;
            }
            snapshot = ct::settle_prize(connection, epoch, snapshot.prizes[*selected - 1].prize_id,
                                        snapshot.revision, method, random_command_id(random), request_id++);
         } else if(key == 'W' && !snapshot.warrants.empty()) {
            const auto selected = input_number("Warrant", 1, static_cast<unsigned>(snapshot.warrants.size()));
            if(selected) {
               snapshot = ct::settle_warrant(connection, epoch, snapshot.warrants[*selected - 1].warrant_id,
                                             snapshot.revision, random_command_id(random), request_id++);
            }
         } else if(key == 'C') {
            auto cruise = snapshot.cruise;
            cruise.active = !cruise.active;
            if(!snapshot.local_contacts.empty()) {
               cruise.hunting_system_id = snapshot.local_contacts.front().destination_system_id;
            }
            const auto crew = input_number("Crew shares percent", 0, 100, cruise.crew_share_percent);
            if(!crew) {
               continue;
            }
            cruise.crew_share_percent = static_cast<uint8_t>(*crew);
            const auto fund = input_number("Ship fund percent", 0, 100 - cruise.crew_share_percent,
                                           cruise.ship_fund_percent);
            if(!fund) {
               continue;
            }
            cruise.ship_fund_percent = static_cast<uint8_t>(*fund);
            if(const auto prohibited = input_text("Prohibited targets", cruise.prohibited_targets, 120)) {
               cruise.prohibited_targets = *prohibited;
            }
            snapshot = ct::set_pirate_cruise(connection, epoch, cruise, random_command_id(random),
                                             request_id++);
         } else if(key == 'M') {
            door_option_prompt({
               "[I] Independent",
               "[N] Naval service",
               "[P] Privateer",
               "[R] Pirate",
               "[Q] Cancel",
            }, false);
            const auto choice = static_cast<char>(std::toupper(
               static_cast<unsigned char>(od_get_key(TRUE))));
            od_printf("\n\r");
            auto mode = snapshot.mode;
            if(choice == 'I') {
               mode = ct::CombatCareerMode::Independent;
            } else if(choice == 'N') {
               mode = ct::CombatCareerMode::Navy;
            } else if(choice == 'P') {
               mode = ct::CombatCareerMode::Privateer;
            } else if(choice == 'R') {
               mode = ct::CombatCareerMode::Pirate;
            } else {
               continue;
            }
            if(mode == ct::CombatCareerMode::Pirate && snapshot.mode == ct::CombatCareerMode::Navy) {
               door_warning(
                  "Taking an issued command outside naval authority is mutiny and ship theft.\n\r");
               door_option_prompt({"[R] Confirm mutiny", "[Q] Cancel"}, false);
               const auto confirm = static_cast<char>(std::toupper(
                  static_cast<unsigned char>(od_get_key(TRUE))));
               od_printf("\n\r");
               if(confirm != 'R') {
                  continue;
               }
            }
            snapshot = ct::set_combat_career_mode(
               connection, epoch, mode, snapshot.revision,
               random_command_id(random), request_id++);
         }
      } catch(const std::exception& error) {
         door_error("%s\n\r", safe_field(error.what()).c_str());
         wait_for_enter();
      }
   }
}

const char* radio_kind_name(const ct::RadioTransmissionKind kind)
{
   switch(kind) {
   case ct::RadioTransmissionKind::PlayerBroadcast:
      return "broadcast";
   case ct::RadioTransmissionKind::InspectionOrder:
      return "inspection order";
   case ct::RadioTransmissionKind::BoardingOrder:
      return "boarding order";
   case ct::RadioTransmissionKind::SurrenderDemand:
      return "surrender demand";
   }
   return "radio";
}

void show_system_radio(
   ct::TlsConnection& connection,
   const uint64_t epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Radio);
   size_t page = 0;
   constexpr size_t page_size = 7;
   for(;;) {
      auto snapshot = ct::get_system_radio(
         connection, epoch, random_command_id(random), request_id++);
      const auto page_count = std::max<size_t>(
         1, (snapshot.entries.size() + page_size - 1) / page_size);
      page = std::min(page, page_count - 1);
      const auto first = page * page_size;
      const auto last = std::min(first + page_size, snapshot.entries.size());
      od_clr_scr();
      door_heading("System Common Radio\n\r");
      door_heading("===================\n\r\n\r");
      door_label("Unread receptions: ");
      door_number("%zu\n\r", snapshot.entries.size());
      if(!snapshot.can_transmit) {
         door_warning("%s\n\r", safe_field(snapshot.unavailable_reason).c_str());
      }
      door_information(
         "Opening a reception displays it once and removes this ship's unread copy.\n\r\n\r");
      if(snapshot.entries.empty()) {
         door_information("No unread System Common receptions.\n\r");
      }
      for(size_t index = first; index < last; ++index) {
         const auto& entry = snapshot.entries[index];
         door_number("%zu", index - first + 1);
         door_label(". ");
         door_identifier("%s", safe_field(entry.sender_ship_name).c_str());
         door_label(" [");
         door_value("%s", safe_field(entry.sender_transponder).c_str());
         door_label("]  ");
         door_value("%s\n\r", radio_kind_name(entry.kind));
         door_label("   Received: ");
         door_value("%s", game_date(entry.received_second).c_str());
         if(entry.kind == ct::RadioTransmissionKind::PlayerBroadcast) {
            door_label("  Sender ");
            door_identifier("%u:%u", entry.sender.bbs_id, entry.sender.player_id);
         }
         if(entry.actionable) {
            door_warning("  Action required");
         }
         od_printf("\n\r");
      }
      door_label("\n\rPage ");
      door_number("%zu/%zu", page + 1, page_count);
      door_option_prompt({
         "[1-7] Open once",
         "[N/P] Page",
         "[B] Broadcast",
         "[M] Mute sender",
         "[U] Unmute",
         "[Enter] Refresh",
         "[Q] Console",
         "[?] Help",
      });
      const auto key = static_cast<char>(
         std::toupper(static_cast<unsigned char>(od_get_key(TRUE))));
      if(key == '\r' || key == '\n') {
         continue;
      }
      if(key == 'Q') {
         return;
      }
      try {
         if(key >= '1' && key <= '7') {
            const auto index = first + static_cast<size_t>(key - '1');
            if(index >= last) {
               continue;
            }
            const auto entry = snapshot.entries[index];
            const auto content = ct::peek_radio_reception(
               connection, epoch, entry.reception_id,
               random_command_id(random), request_id++);
            od_clr_scr();
            door_heading("System Common Reception\n\r=======================\n\r\n\r");
            door_label("From: ");
            door_identifier("%s", safe_field(entry.sender_ship_name).c_str());
            door_label(" [");
            door_value("%s", safe_field(entry.sender_transponder).c_str());
            door_label("]\n\r");
            door_label("Emitted: ");
            door_value("%s", game_date(entry.emitted_second).c_str());
            door_label("  Received: ");
            door_value("%s\n\r\n\r", game_date(entry.received_second).c_str());
            door_information("%s\n\r", safe_field(content.body).c_str());
            wait_for_enter("Consume reception");
            (void)ct::acknowledge_radio_reception(
               connection, epoch, entry.reception_id,
               random_command_id(random), request_id++);
         } else if(key == 'N' && page + 1 < page_count) {
            ++page;
         } else if(key == 'P' && page > 0) {
            --page;
         } else if(key == 'B') {
            if(!snapshot.can_transmit) {
               door_error("%s\n\r", safe_field(snapshot.unavailable_reason).c_str());
               wait_for_enter("System Radio");
               continue;
            }
            if(const auto body = input_text("Broadcast", "", 500)) {
               (void)ct::transmit_system_radio(
                  connection, epoch, *body,
                  random_command_id(random), request_id++);
               door_success("Transmission launched on System Common.\n\r");
               wait_for_enter("System Radio");
            }
         } else if(key == 'M' && first < last) {
            const auto selected = input_number(
               "Entry", 1, static_cast<unsigned>(last - first));
            if(selected) {
               const auto& entry = snapshot.entries[first + *selected - 1];
               if(entry.kind != ct::RadioTransmissionKind::PlayerBroadcast) {
                  door_warning("Structured encounter hails cannot be muted.\n\r");
                  wait_for_enter("System Radio");
                  continue;
               }
               const auto& sender = entry.sender;
               (void)ct::set_radio_mute(
                  connection, epoch, sender, true,
                  random_command_id(random), request_id++);
            }
         } else if(key == 'U' && !snapshot.mutes.empty()) {
            od_printf("\n\r");
            for(size_t index = 0; index < snapshot.mutes.size(); ++index) {
               door_number("%zu", index + 1);
               door_label(". Sender ");
               door_value("%u:%u\n\r", snapshot.mutes[index].bbs_id,
                          snapshot.mutes[index].player_id);
            }
            const auto selected = input_number(
               "Mute", 1, static_cast<unsigned>(snapshot.mutes.size()));
            if(selected) {
               (void)ct::set_radio_mute(
                  connection, epoch, snapshot.mutes[*selected - 1], false,
                  random_command_id(random), request_id++);
            }
         }
      } catch(const std::exception& error) {
         door_error("%s\n\r", safe_field(error.what()).c_str());
         wait_for_enter("System Radio");
      }
   }
}

void render_command_console(const ct::ServerHello& hello)
{
   od_clr_scr();
   door_heading("Captain's Command Console\n\r");
   door_heading("=========================\n\r\n\r");
   door_label("Ship status: ");
   door_identifier("%s\n\r", phase_name(hello.phase));
   door_information(
      "These seven managers are available throughout every operational "
      "situation. Available actions depend on the ship's present status.\n\r\n\r");
   door_number("C");
   door_label(". ");
   door_identifier("Crew Management\n\r");
   door_number("S");
   door_label(". ");
   door_identifier("Ship Management\n\r");
   door_number("T");
   door_label(". ");
   door_identifier("Task Management\n\r");
   door_number("M");
   door_label(". ");
   door_identifier("Message Management\n\r");
   door_number("R");
   door_label(". ");
   door_identifier("System Common Radio\n\r");
   door_number("K");
   door_label(". ");
   door_identifier("Known Universe\n\r");
   door_number("O");
   door_label(". ");
   door_identifier("Operations Ledger\n\r");
   door_option_prompt({
      "[C/K/M/O/R/S/T] Manager",
      "[Enter] Refresh",
      "[X] Operational view",
      "[Q] Return to BBS",
      "[?] Help",
   });
}

bool run_command_console(
   ct::TlsConnection& connection,
   ct::ServerHello& hello,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::CommandConsole);
   render_command_console(hello);
   while(true) {
      const auto generation = phase_event_generation;
      const auto key = door_get_live_key();
      if(phase_event_generation != generation) {
         return false;
      }
      if(key == 'c' || key == 'C') {
         show_crew_manager(
            connection,
            hello.assigned_epoch,
            random,
            request_id);
         render_command_console(hello);
      } else if(key == 's' || key == 'S') {
         show_ship_manager(
            connection,
            hello.assigned_epoch,
            random,
            request_id);
         render_command_console(hello);
      } else if(key == 't' || key == 'T') {
         show_task_manager(connection, hello.assigned_epoch, random, request_id);
         render_command_console(hello);
      } else if(key == 'm' || key == 'M') {
         show_message_manager(
            connection,
            hello.assigned_epoch,
            random,
            request_id);
         render_command_console(hello);
      } else if(key == 'r' || key == 'R') {
         show_system_radio(
            connection,
            hello.assigned_epoch,
            random,
            request_id);
         render_command_console(hello);
      } else if(key == 'k' || key == 'K') {
         show_known_universe_manager(
            connection, hello.assigned_epoch, random, request_id);
         render_command_console(hello);
      } else if(key == 'o' || key == 'O') {
         if(const auto phase = show_combat_operations(
                                  connection, hello.assigned_epoch, random, request_id)) {
            hello.phase = *phase;
            return false;
         }
         render_command_console(hello);
      } else if(key == 'l' || key == 'L') {
         show_open_game_license();
         render_command_console(hello);
      } else if(key == 'q' || key == 'Q') {
         if(confirm_return_to_bbs()) {
            return true;
         }
         render_command_console(hello);
      } else if(key == '\r' || key == '\n') {
         render_command_console(hello);
      } else if(key == 'x' || key == 'X') {
         return false;
      }
   }
}

void wait_for_enter(const char* destination)
{
   door_prompt("\n\r[Enter] %s\n\r", destination);
   while(true) {
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         return;
      }
   }
}

void render_docked_snapshot(const ct::DockedSnapshot& snapshot)
{
   door_heading("Docked Operations - ");
   door_value("%s\n\r", safe_field(snapshot.ship_name).c_str());
   door_heading("=================\n\r");
   door_identifier("%s", safe_field(snapshot.facility_name).c_str());
   door_label(" / ");
   door_value("%s", safe_field(snapshot.system_name).c_str());
   door_label("  Port ");
   door_value("%s", safe_field(snapshot.starport).c_str());
   door_label(" TL");
   door_number("%u", snapshot.tech_level);
   door_label(" Law ");
   door_number("%u\n\r", snapshot.law_level);
   door_label("Cash: ");
   door_number("Cr%llu", static_cast<unsigned long long>(snapshot.credits));
   if(snapshot.restricted_credits != 0) {
      door_label("  Ship account: ");
      door_number("Cr%llu", static_cast<unsigned long long>(snapshot.restricted_credits));
   }
   door_label("  Debt: ");
   door_number("Cr%llu\n\r", static_cast<unsigned long long>(snapshot.debt_credits));
   door_label("Berth account: ");
   door_number("Cr%llu due on departure\n\r",
               static_cast<unsigned long long>(snapshot.accrued_berth_fee_credits));
   door_label("Port control: ");
   if(snapshot.clearance_required) {
      door_warning("controlled traffic; clearance is filed with the departure plan\n\r");
   } else {
      door_success("open departure\n\r");
   }
   door_label("Fuel: ");
   print_millitons(snapshot.fuel_millitons);
   door_label("/");
   print_millitons(snapshot.fuel_capacity_millitons);
   door_label("  Cargo: ");
   print_millitons(snapshot.cargo_used_millitons);
   door_label("/");
   print_millitons(snapshot.cargo_capacity_millitons);
   od_printf("\n\r\n\r");
}

void render_market(const ct::MarketSnapshot& market)
{
   od_clr_scr();
   door_heading("Cargo Exchange - ");
   door_value("%s\n\r", safe_field(market.world_name).c_str());
   door_heading("==============\n\r");
   door_label("Cash: ");
   door_number("Cr%llu", static_cast<unsigned long long>(market.credits));
   door_label("  Hold: ");
   print_millitons(market.cargo_used_millitons);
   door_label("/");
   print_millitons(market.cargo_capacity_millitons);
   od_printf("\n\r\n\r");
   door_identifier("Local offers\n\r");
   for(size_t index = 0; index < market.offers.size(); ++index) {
      const auto& offer = market.offers[index];
      door_number("%u", static_cast<unsigned>(index + 1));
      door_label(". ");
      door_value("%s\n\r", safe_field(offer.commodity_name).c_str());
      door_label("   Buy Cr");
      door_number("%llu", static_cast<unsigned long long>(offer.purchase_price_per_ton));
      door_label("/t  Sell Cr");
      door_number("%llu", static_cast<unsigned long long>(offer.sale_price_per_ton));
      door_label("/t  Available ");
      print_millitons(offer.available_millitons);
      od_printf("\n\r");
   }
   door_identifier("\n\rCargo aboard\n\r");
   if(market.cargo.empty()) {
      door_information("  None\n\r");
   }
   for(size_t index = 0; index < market.cargo.size(); ++index) {
      const auto& lot = market.cargo[index];
      door_number("%u", static_cast<unsigned>(index + 1));
      door_label(". ");
      door_value("%s", safe_field(lot.commodity_name).c_str());
      door_label("  ");
      print_millitons(lot.quantity_millitons);
      door_label("  paid Cr");
      door_number("%llu/t\n\r",
                  static_cast<unsigned long long>(lot.purchase_price_per_ton));
   }
   door_identifier("\n\rPort research\n\r");
   if(market.work_assignments.empty()) {
      door_information("  No research assignments recorded\n\r");
   }
   for(size_t index = 0; index < market.work_assignments.size(); ++index) {
      const auto& work = market.work_assignments[index];
      door_number("%zu", index + 1);
      door_label(". Assignment #");
      door_identifier("%llu", static_cast<unsigned long long>(work.assignment_id));
      door_label("  Due: ");
      door_number("%s", game_date(work.due_second).c_str());
      if(!work.result_text.empty()) {
         door_label("  ");
         door_value("%s", safe_field(work.result_text).c_str());
      }
      od_printf("\n\r");
   }
   if(!market.events.empty()) {
      door_identifier("\n\rExchange notices\n\r");
      for(const auto& event : market.events) {
         door_warning("  %s (through %s)\n\r",
                      safe_field(event.headline).c_str(),
                      game_date(event.expires_second).c_str());
      }
   }
   door_identifier("\n\rPrivate market leads\n\r");
   if(market.leads.empty()) {
      door_information("  None\n\r");
   }
   for(size_t index = 0; index < market.leads.size(); ++index) {
      const auto& lead = market.leads[index];
      door_number("%zu", index + 1);
      door_label(". ");
      door_value("%s", safe_field(lead.commodity_name).c_str());
      door_label(lead.side == ct::MarketLeadSide::Supplier ? " supplier " : " buyer ");
      door_number("Cr%llu/t  ", static_cast<unsigned long long>(lead.price_per_ton));
      print_millitons(lead.quantity_millitons);
      if(lead.state == ct::MarketLeadState::Reserved) {
         door_identifier("  RESERVED");
         door_label(" (Cr");
         door_number("%llu", static_cast<unsigned long long>(lead.escrow_credits));
         door_label(" escrow)");
      }
      od_printf("\n\r");
   }
}

void run_cargo_exchange(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Cargo);
   auto market = ct::get_market(
                    connection, session_epoch, random_command_id(random), request_id++);
   while(true) {
      render_market(market);
      door_option_prompt({
         "[B] Buy",
         "[S] Sell",
         "[F] Find market",
         "[R] Reserve lead",
         "[P] Perform lead",
         "[U] Release reservation",
         "[X] Cancel search",
         "[Enter] Refresh",
         "[Q] Docked operations",
         "[?] Help",
      });
      const auto key = od_get_key(TRUE);
      if(key == '\r' || key == '\n') {
         market = ct::get_market(
            connection, session_epoch, random_command_id(random), request_id++);
         continue;
      }
      if(key == 'q' || key == 'Q') {
         return;
      }
      if(key == 'b' || key == 'B') {
         if(market.offers.empty()) {
            continue;
         }
         const auto choice = input_number(
                                "Offer", 1, static_cast<unsigned>(market.offers.size()));
         if(!choice) {
            continue;
         }
         const auto& offer = market.offers[*choice - 1];
         const auto hold_free = market.cargo_capacity_millitons -
                                market.cargo_used_millitons;
         const auto physical_maximum =
            std::min(offer.available_millitons, hold_free);
         const auto maximum_millitons = ct::maximum_affordable_cargo(
                                           market.credits,
                                           offer.purchase_price_per_ton,
                                           physical_maximum);
         if(maximum_millitons == 0) {
            door_error("None of that offer can be loaded and paid for.\n\r");
            wait_for_enter();
            continue;
         }
         const auto quantity = input_tonnage("Tonnes", maximum_millitons);
         if(!quantity) {
            continue;
         }
         market = ct::buy_cargo(
                     connection,
                     session_epoch,
                     market.market_revision,
                     offer.offer_id,
                     *quantity,
                     random_command_id(random),
                     request_id++);
      } else if(key == 'f' || key == 'F') {
         struct SearchGood {
            uint16_t id;
            std::string name;
         };
         std::vector<SearchGood> goods;
         const auto add_good = [&goods](const uint16_t id, const std::string& name) {
            const auto exists = std::any_of(
                                   goods.begin(),
                                   goods.end(),
                                   [id](const auto& good) {
               return good.id == id;
            });
            if(!exists) {
               goods.push_back({
                  id,
                  name,
               });
            }
         };
         for(const auto& offer : market.offers) {
            add_good(offer.commodity_id, offer.commodity_name);
         }
         for(const auto& lot : market.cargo) {
            add_good(lot.commodity_id, lot.commodity_name);
         }
         if(goods.empty()) {
            door_warning("No commodity is available to identify the requested market.\n\r");
            wait_for_enter();
            continue;
         }
         door_information("\n\r1. Locate a supplier\n\r2. Locate a buyer\n\r");
         const auto kind = input_number("Search", 1, 2);
         if(!kind) {
            continue;
         }
         output().resume_paging();
         for(size_t i = 0; i < goods.size(); ++i) {
            door_number("%zu", i + 1);
            door_label(". ");
            door_value("%s\n\r", safe_field(goods[i].name).c_str());
         }
         const auto selected = input_number("Commodity", 1, static_cast<unsigned>(goods.size()));
         if(!selected) {
            continue;
         }
         door_label("Method\n\r");
         door_option_prompt({
            "[P] Port canvass",
            "[O] Data exchange",
            "[B] Private introduction",
            "[H] Hired broker",
            "[Q] Cancel",
         }, false);
         const auto method_key = od_get_key(TRUE);
         od_printf("\n\r");
         if(method_key == 'q' || method_key == 'Q') {
            continue;
         }
         ct::MarketSearchMethod method;
         if(method_key == 'p' || method_key == 'P') {
            method = ct::MarketSearchMethod::Physical;
         } else if(method_key == 'o' || method_key == 'O') {
            method = ct::MarketSearchMethod::Online;
         } else if(method_key == 'b' || method_key == 'B') {
            method = ct::MarketSearchMethod::BlackMarket;
         } else if(method_key == 'h' || method_key == 'H') {
            method = ct::MarketSearchMethod::HiredBroker;
         } else {
            continue;
         }
         const auto crew = ct::get_crew_management(
                              connection,
                              session_epoch,
                              random_command_id(random),
                              request_id++);
         const auto captain = std::find_if(
                                 crew.members.begin(),
                                 crew.members.end(),
                                 [](const auto& member) {
            return member.captain;
         });
         if(captain == crew.members.end()) {
            throw std::runtime_error("captain is absent from the crew manifest");
         }
         const auto search_kind = *kind == 1
                                  ? ct::MarketSearchKind::Supplier
                                  : ct::MarketSearchKind::Buyer;
         market = ct::begin_market_search(
                     connection,
                     session_epoch,
                     search_kind,
                     method,
                     captain->person_id,
                     goods[*selected - 1].id,
                     0,
                     random_command_id(random),
                     request_id++);
      } else if(key == 'x' || key == 'X') {
         std::vector<const ct::WorkAssignment*> active;
         for(const auto& work : market.work_assignments) {
            if(work.state != ct::WorkState::Scheduled) {
               continue;
            }
               active.push_back(&work);
         }
         if(active.empty()) {
            door_information("No active market search can be cancelled.\n\r");
            wait_for_enter();
            continue;
         }
         const auto selected = input_number("Assignment", 1, static_cast<unsigned>(active.size()));
         if(!selected) {
            continue;
         }
         market = ct::cancel_work_assignment(connection, session_epoch, active[*selected - 1]->assignment_id,
                                             random_command_id(random), request_id++);
      } else if(key == 'r' || key == 'R') {
         std::vector<const ct::MarketLead*> available;
         for(const auto& lead : market.leads) {
            if(lead.state == ct::MarketLeadState::Available) {
               available.push_back(&lead);
            }
         }
         if(available.empty()) {
            door_information("No unreserved lead is available.\n\r");
            wait_for_enter();
            continue;
         }
         const auto selected = input_number("Lead", 1, static_cast<unsigned>(available.size()));
         if(!selected) {
            continue;
         }
         const auto* lead = available[*selected - 1];
         const auto quantity = input_tonnage("Tonnes", lead->quantity_millitons);
         if(!quantity) {
            continue;
         }
         market = ct::reserve_market_lead(
                     connection, session_epoch, lead->lead_id, lead->revision, *quantity,
                     random_command_id(random), request_id++);
      } else if(key == 'u' || key == 'U') {
         std::vector<const ct::MarketLead*> reserved;
         for(const auto& lead : market.leads) {
            if(lead.state == ct::MarketLeadState::Reserved) {
               reserved.push_back(&lead);
            }
         }
         if(reserved.empty()) {
            continue;
         }
         const auto selected = input_number("Reservation", 1, static_cast<unsigned>(reserved.size()));
         if(!selected) {
            continue;
         }
         const auto* lead = reserved[*selected - 1];
         market = ct::release_market_reservation(
                     connection, session_epoch, lead->lead_id, lead->revision,
                     random_command_id(random), request_id++);
      } else if(key == 'p' || key == 'P') {
         std::vector<const ct::MarketLead*> reserved;
         for(const auto& lead : market.leads) {
            if(lead.state == ct::MarketLeadState::Reserved) {
               reserved.push_back(&lead);
            }
         }
         if(reserved.empty()) {
            continue;
         }
         const auto selected = input_number("Reservation", 1, static_cast<unsigned>(reserved.size()));
         if(!selected) {
            continue;
         }
         const auto* lead = reserved[*selected - 1];
         if(lead->side == ct::MarketLeadSide::Supplier) {
            market = ct::buy_cargo(
                        connection, session_epoch, market.market_revision, lead->lead_id,
                        lead->quantity_millitons, random_command_id(random), request_id++);
         } else {
            std::vector<const ct::CargoLot*> matching;
            for(const auto& lot : market.cargo) {
               if(lot.commodity_id == lead->commodity_id) {
                  matching.push_back(&lot);
               }
            }
            if(matching.empty()) {
               door_warning("No matching cargo is aboard.\n\r");
               wait_for_enter();
               continue;
            }
            const auto lot_choice = input_number("Cargo lot", 1, static_cast<unsigned>(matching.size()));
            if(!lot_choice) {
               continue;
            }
            const auto* lot = matching[*lot_choice - 1];
            const auto quantity = std::min(lot->quantity_millitons, lead->quantity_millitons);
            market = ct::sell_cargo_to_lead(
                        connection, session_epoch, market.market_revision, lot->cargo_lot_id,
                        quantity, lead->lead_id, random_command_id(random), request_id++);
         }
      } else if(key == 's' || key == 'S') {
         if(market.cargo.empty()) {
            door_warning("There is no cargo aboard to sell.\n\r");
            wait_for_enter();
            continue;
         }
         const auto choice = input_number(
                                "Cargo lot", 1, static_cast<unsigned>(market.cargo.size()));
         if(!choice) {
            continue;
         }
         const auto& lot = market.cargo[*choice - 1];
         if(lot.origin_system_id == market.system_id) {
            door_warning(
               "That speculative lot must be carried to another system before sale.\n\r");
            wait_for_enter();
            continue;
         }
         const auto quantity = input_tonnage("Tonnes", lot.quantity_millitons);
         if(!quantity) {
            continue;
         }
         market = ct::sell_cargo(
                     connection,
                     session_epoch,
                     market.market_revision,
                     lot.cargo_lot_id,
                     *quantity,
                     random_command_id(random),
                     request_id++);
      }
   }
}

std::optional<ct::TravelStatus> run_fuel_service(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Fuel);
   while(true) {
   const auto account = ct::get_docked_snapshot(connection, session_epoch, random_command_id(random),
      request_id++);
   const auto services = ct::get_docked_services(connection, session_epoch, random_command_id(random),
      request_id++);
   od_clr_scr();
   door_heading("Fuel and Supplies\n\r");
   door_heading("=================\n\r\n\r");
   door_label("Operating account: ");
   door_number("Cr%llu\n\r", static_cast<unsigned long long>(account.credits));
   door_label("Tanks: ");
   door_number("%.1f/%.1f t\n\r", account.fuel_millitons / 1000.0,
               account.fuel_capacity_millitons / 1000.0);
   door_label("Life-support stores: ");
   door_number("%llu/%llu person-days\n\r",
               static_cast<unsigned long long>(services.provisions.person_days_remaining),
               static_cast<unsigned long long>(services.provisions.capacity_person_days));
   door_label("Magazine lots: ");
   door_number("%zu\n\r", services.ammunition.size());
   std::vector<std::string_view> options{
      "[F] Fuel source",
   };
   if(services.provisions_available) {
      options.emplace_back("[P] Provisions");
   }
   if(services.ammunition_available) {
      options.emplace_back("[A] Ammunition");
   }
   options.emplace_back("[Enter] Refresh");
   options.emplace_back("[Q] Docked operations");
   options.emplace_back("[?] Help");
   door_option_prompt(options);
   const auto key = static_cast<char>(
                       std::toupper(static_cast<unsigned char>(door_get_live_key())));
   if(key == '\r' || key == '\n') {
      continue;
   }
   if(key == 'Q') {
      return std::nullopt;
   }
   ct::DockedServiceOrder order{};
   order.expected_ship_revision = services.ship_revision;
   if(key == 'F') {
      output().resume_paging();
      for(size_t index = 0; index < services.fuel.size(); ++index) {
         const auto& item = services.fuel[index];
         door_number("%zu", index + 1);
         door_label(". ");
         if(item.available) {
            door_identifier("%s", safe_field(item.label).c_str());
            if(item.price_per_ton_credits) {
               door_number("  Cr%llu/t", static_cast<unsigned long long>(item.price_per_ton_credits));
            } else {
               door_number("  %s", course_duration(item.service_seconds).c_str());
            }
            od_printf("\n\r");
         } else {
            door_warning("%s — %s\n\r", safe_field(item.label).c_str(),
                         safe_field(item.unavailable_reason).c_str());
         }
      }
      const auto choice = input_number("Fuel source", 1, static_cast<unsigned>(services.fuel.size()));
      if(!choice) {
         return std::nullopt;
      }
      const auto& item = services.fuel[*choice - 1];
      if(!item.available) {
         door_warning("That service is not available.\n\r");
         wait_for_enter();
         return std::nullopt;
      }
      const auto max_tons = std::min<uint64_t>(item.maximum_millitons / 1000,
         std::numeric_limits<unsigned>::max());
      const auto tons = input_number("Tonnes", 1, static_cast<unsigned>(max_tons));
      if(!tons) {
         return std::nullopt;
      }
      order.kind = ct::DockedServiceOrder::Kind::Fuel;
      order.fuel_kind = item.kind;
      order.source_body_id = item.source_body_id;
      order.quantity_millitons = uint64_t(*tons) * 1000;
   } else if(key == 'P') {
      if(!services.provisions_available) {
         door_warning("No bonded chandlery supplies starships at this port.\n\r");
         wait_for_enter();
         return std::nullopt;
      }
      const auto remaining = services.provisions.capacity_person_days -
                             services.provisions.person_days_remaining;
      const auto max_packages = services.provision_package_person_days ? remaining /
                                services.provision_package_person_days : 0;
      if(!max_packages) {
         door_information("Installed stores are already full.\n\r");
         wait_for_enter();
         return std::nullopt;
      }
      const auto packages = input_number("Monthly packages", 1,
                                         static_cast<unsigned>(std::min<uint64_t>(
                                            max_packages,
                                            std::numeric_limits<unsigned>::max())));
      if(!packages) {
         return std::nullopt;
      }
      order.kind = ct::DockedServiceOrder::Kind::Provisions;
      order.packages = *packages;
   } else if(key == 'A') {
      if(!services.ammunition_available) {
         door_warning("No licensed starship ordnance dealer operates at this port.\n\r");
         wait_for_enter();
         return std::nullopt;
      }
      std::vector<const ct::ShipAmmunitionStatus*> reloadable;
      for(const auto& lot : services.ammunition) {
         if(lot.remaining >= lot.capacity) {
            continue;
         }
         reloadable.push_back(&lot);
      }
      if(reloadable.empty()) {
         door_information("All catalogued magazines are full.\n\r");
         wait_for_enter();
         return std::nullopt;
      }
      output().resume_paging();
      for(size_t index = 0; index < reloadable.size(); ++index) {
         const auto& lot = *reloadable[index];
         door_number("%zu", index + 1);
         door_label(". ");
         door_identifier("%s ", safe_field(lot.ammunition_id).c_str());
         door_number("%u/%u; %u per Cr%llu\n\r", lot.remaining, lot.capacity, lot.pack_units,
                     static_cast<unsigned long long>(lot.price_per_pack_credits));
      }
      const auto choice = input_number("Ammunition", 1, static_cast<unsigned>(reloadable.size()));
      if(!choice) {
         return std::nullopt;
      }
      const auto& lot = *reloadable[*choice - 1];
      const auto maximum = (lot.capacity - lot.remaining) / lot.pack_units;
      const auto packs = input_number("Packs", 1, maximum);
      if(!packs) {
         return std::nullopt;
      }
      order.kind = ct::DockedServiceOrder::Kind::Ammunition;
      order.ammunition_id = lot.ammunition_id;
      order.packs = *packs;
   } else {
      return std::nullopt;
   }
   try {
      const auto result = ct::commit_docked_service(connection, session_epoch, order,
         random_command_id(random), request_id++);
      if(result.phase == ct::PlayerPhase::Interplanetary) {
         return ct::get_travel_status(connection, session_epoch, random_command_id(random), request_id++);
      }
      door_success("\n\rShip's stores have been loaded and the account settled.\n\r");
      wait_for_enter();
   } catch(const std::exception& error) {
      door_error("%s  Press any key.\n\r", safe_field(error.what()).c_str());
      od_get_key(TRUE);
   }
   return std::nullopt;
   }
}

const char* waypoint_authority_name(const ct::WaypointAuthority authority)
{
   switch(authority) {
   case ct::WaypointAuthority::Hold:
      return "hold";
   case ct::WaypointAuthority::Terminal:
      return "terminal";
   case ct::WaypointAuthority::Through:
      return "through";
   }
   return "unknown";
}

std::string flight_plan_action_name(
   const ct::FlightPlanAction& action,
   const ct::KnownDestinations& destinations)
{
   const auto system_name = [&destinations](const uint64_t system_id) {
      const auto found = std::find_if(
                            destinations.systems.begin(), destinations.systems.end(),
      [system_id](const auto & system) {
         return system.system_id == system_id;
      });
      return found == destinations.systems.end()
             ? std::string("an unlisted system")
             : found->system_name;
   };
   switch(action.kind) {
   case ct::FlightPlanActionKind::Hold:
      return "Hold position";
   case ct::FlightPlanActionKind::Jump:
      return "Jump to " + system_name(action.destination_system_id);
   case ct::FlightPlanActionKind::JumpCoordinates:
      return "Jump to " + std::to_string(action.coreward_parsecs) + ", " +
             std::to_string(action.spinward_parsecs) + ", " +
             std::to_string(action.north_parsecs) + " pc";
   case ct::FlightPlanActionKind::Dock:
      return "Dock at primary facility";
   case ct::FlightPlanActionKind::Fuel:
      switch(action.fuel_operation) {
      case ct::FuelOperation::GasGiant:
         return "Skim " + std::to_string(action.quantity_millitons / 1000) + " t";
      case ct::FuelOperation::WildernessWater:
         return "Collect water/ice " +
                std::to_string(action.quantity_millitons / 1000) + " t";
      case ct::FuelOperation::BuyRefined:
         return "Buy " + std::to_string(action.quantity_millitons / 1000) +
                " t refined fuel";
      case ct::FuelOperation::BuyUnrefined:
         return "Buy " + std::to_string(action.quantity_millitons / 1000) +
                " t unrefined fuel";
      }
      return "Acquire fuel";
   }
   return "Unknown action";
}

ct::FlightPlanStep jump_step(
   const uint64_t origin_system_id,
   const uint64_t destination_system_id)
{
   return ct::FlightPlanStep{
      .locus = ct::FlightLocus{
         .kind = ct::FlightLocusKind::JumpLocus,
         .system_id = origin_system_id,
         .world_id = 0,
         .facility_id = 0,
         .body_id = 0},
      .authority = ct::WaypointAuthority::Through,
      .action = ct::FlightPlanAction{
         .kind = ct::FlightPlanActionKind::Jump,
         .destination_system_id = destination_system_id},
   };
}

ct::FlightPlanStep jump_step_from_locus(
   const ct::FlightLocus& origin,
   const uint64_t destination_system_id)
{
   auto step = jump_step(origin.system_id, destination_system_id);
   step.locus = origin;
   return step;
}

std::optional<double> input_coordinate(
   const char* prompt,
   const double default_value)
{
   const auto text = input_text(prompt, std::to_string(default_value), 32);
   if(!text) {
      return std::nullopt;
   }
   try {
      size_t consumed = 0;
      const auto value = std::stod(*text, &consumed);
      if(consumed == text->size() && std::isfinite(value)) {
         return value;
      }
   } catch(const std::exception&) {
   }
   door_error("Enter a finite coordinate in parsecs.\n\r");
   return std::nullopt;
}

ct::FlightPlanStep coordinate_jump_step(
   const uint64_t origin_system_id,
   const double coreward,
   const double spinward,
   const double north)
{
   return ct::FlightPlanStep{
      .locus = ct::FlightLocus{
         .kind = ct::FlightLocusKind::JumpLocus,
         .system_id = origin_system_id,
         .world_id = 0,
         .facility_id = 0,
         .body_id = 0},
      .authority = ct::WaypointAuthority::Terminal,
      .action = ct::FlightPlanAction{
         .kind = ct::FlightPlanActionKind::JumpCoordinates,
         .destination_system_id = 0,
         .world_id = 0,
         .facility_id = 0,
         .fuel_operation = ct::FuelOperation::GasGiant,
         .quantity_millitons = 0,
         .coreward_parsecs = coreward,
         .spinward_parsecs = spinward,
         .north_parsecs = north},
   };
}

bool configure_jump_navigation(ct::FlightPlanAction& action)
{
   door_option_prompt({
      "[O] Plot aboard",
      "[T] Buy fresh course tape",
      "[Q] Cancel",
   }, false);
   const auto method = static_cast<char>(
                          std::toupper(static_cast<unsigned char>(od_get_key(TRUE))));
   od_printf("\n\r");
   if(method == 'Q' || (method != 'O' && method != 'T')) {
      return false;
   }
   action.jump_navigation = method == 'T'
                            ? ct::JumpNavigationMethod::CommercialTape
                            : ct::JumpNavigationMethod::Onboard;
   door_information("If the computer identifies a bad plot:\n\r");
   door_option_prompt({
      "[R] Replot",
      "[P] Proceed despite certain misjump",
      "[Q] Cancel",
   }, false);
   const auto risk = static_cast<char>(
                        std::toupper(static_cast<unsigned char>(od_get_key(TRUE))));
   od_printf("\n\r");
   if(risk == 'Q' || (risk != 'R' && risk != 'P')) {
      return false;
   }
   action.proceed_on_known_bad_plot = risk == 'P';
   return true;
}

ct::FlightPlanStep primary_dock_step(
   const uint64_t system_id,
   const ct::WaypointAuthority authority)
{
   return ct::FlightPlanStep{
      .locus = ct::FlightLocus{
         .kind = ct::FlightLocusKind::Port,
         .system_id = system_id,
         .world_id = system_id,
         .facility_id = system_id,
         .body_id = 0},
      .authority = authority,
      .action = ct::FlightPlanAction{
         .kind = ct::FlightPlanActionKind::Dock,
         .destination_system_id = 0,
         .world_id = system_id,
         .facility_id = system_id},
   };
}

ct::FlightPlanStep purchase_fuel_step(
   const uint64_t system_id,
   const ct::CourseFuelSource source,
   const uint64_t quantity_millitons)
{
   return ct::FlightPlanStep{
      .locus = ct::FlightLocus{
         .kind = ct::FlightLocusKind::Port,
         .system_id = system_id,
         .world_id = system_id,
         .facility_id = system_id,
         .body_id = 0},
      .authority = ct::WaypointAuthority::Through,
      .action = ct::FlightPlanAction{
         .kind = ct::FlightPlanActionKind::Fuel,
         .destination_system_id = 0,
         .world_id = 0,
         .facility_id = 0,
         .fuel_operation = source == ct::CourseFuelSource::RefinedPort
                           ? ct::FuelOperation::BuyRefined
                           : ct::FuelOperation::BuyUnrefined,
         .quantity_millitons = quantity_millitons},
   };
}

std::optional<ct::TravelStatus> run_flight_plan_editor(
   ct::TlsConnection& connection,
   const uint64_t session_epoch,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::FlightPlan);
   const auto destinations = ct::get_known_destinations(
                                connection, session_epoch, random_command_id(random), request_id++);
   const auto current_plan = ct::get_flight_plan(
                                connection, session_epoch, random_command_id(random), request_id++);
   const auto travel = ct::get_travel_status(
                          connection, session_epoch, random_command_id(random), request_id++);
   ct::FlightPlanProposal proposal{
      .expected_plan_revision = current_plan.revision,
      .steps = current_plan.steps,
      .policy = current_plan.policy,
   };
   while(true) {
      od_clr_scr();
      door_heading("Flight Plan\n\r===========\n\r\n\r");
      door_label("Revision: ");
      door_identifier("%llu", static_cast<unsigned long long>(proposal.expected_plan_revision));
      door_label("  Drive: ");
      door_identifier("Jump-%u\n\r\n\r", destinations.jump_rating);
      if(proposal.steps.empty()) {
         door_information("No route has been entered.\n\r");
      }
      for(size_t index = 0; index < proposal.steps.size(); ++index) {
         const auto& step = proposal.steps[index];
         door_number("%zu", index + 1);
         door_label(". ");
         door_identifier("%s", safe_field(flight_plan_action_name(step.action, destinations)).c_str());
         door_label("  [");
         door_value("%s", waypoint_authority_name(step.authority));
         door_label("]\n\r");
      }
      door_option_prompt({
         "[A] Add charted leg",
         "[C] Import plotted course",
         "[J] Add task destination",
         "[G] Add frontier fuel stop",
         "[X] Explore coordinates",
         "[D] Delete last leg",
         "[T] Last authority",
         "[P] Preview and file",
         "[Enter] Refresh",
         "[Q] Keep existing plan",
         "[?] Help",
      });
      const auto key = static_cast<char>(
                          std::toupper(static_cast<unsigned char>(door_get_live_key())));
      if(key == '\r' || key == '\n') {
         continue;
      }
      if(key == 'Q') {
         return std::nullopt;
      }
      if(key == 'A') {
         const uint64_t origin_system_id = proposal.steps.empty()
                                           ? destinations.current_system_id
                                           : proposal.steps.back().locus.system_id;
         const auto selected = select_known_primary(
                                  destinations,
                                  "Add Charted Leg",
                                  origin_system_id,
                                  origin_system_id,
                                  true);
         if(!selected) {
            continue;
         }
         const auto destination_system_id = (*selected)->system_id;
         auto jump = origin_system_id == 0
                     ? jump_step_from_locus(travel.origin, destination_system_id)
                     : jump_step(origin_system_id, destination_system_id);
         if(!configure_jump_navigation(jump.action)) {
            continue;
         }
         proposal.steps.push_back(jump);
         proposal.steps.push_back(primary_dock_step(
                                     destination_system_id, ct::WaypointAuthority::Terminal));
      } else if(key == 'C') {
         if(destinations.current_system_id == 0) {
            door_warning("Course imports require a charted origin system.\n\r");
            wait_for_enter();
            continue;
         }
         const auto selected = select_known_primary(
                                  destinations, "Import Course Destination", destinations.current_system_id);
         if(!selected) {
            continue;
         }
         try {
            const auto plot = ct::plot_course(
                                 connection, session_epoch, destinations.current_system_id,
                                 (*selected)->system_id, true, random_command_id(random), request_id++);
            door_option_prompt({
               "[F] Fastest course",
               "[C] Cheapest course",
               "[Q] Cancel",
            }, false);
            const auto choice = static_cast<char>(
                                   std::toupper(static_cast<unsigned char>(door_get_live_key())));
            od_printf("\n\r");
            if(choice != 'F' && choice != 'C') {
               continue;
            }
            const auto& course = choice == 'F' ? plot.fastest : plot.cheapest;
            if(!course.available || course.waypoints.size() < 2) {
               door_warning("No executable course was found.\n\r");
               wait_for_enter();
               continue;
            }
            if(std::any_of(course.waypoints.begin(), course.waypoints.end(),
            [](const auto & waypoint) {
            return waypoint.fuel_source == ct::CourseFuelSource::FrontierSkimming;
         })) {
               door_warning(
                  "This course requires a body-specific frontier-fuel stop. "
                  "Add that named body manually before filing.\n\r");
               wait_for_enter();
               continue;
            }
            ct::FlightPlanAction navigation_probe;
            navigation_probe.kind = ct::FlightPlanActionKind::Jump;
            if(!configure_jump_navigation(navigation_probe)) {
               continue;
            }
            const auto jump_one_fuel = destinations.jump_rating == 0
                                       ? 0
                                       : travel.jump_fuel_millitons / destinations.jump_rating;
            if(jump_one_fuel == 0) {
               door_warning("The ship's Jump fuel allocation is unavailable.\n\r");
               wait_for_enter();
               continue;
            }
            const auto leg_fuel = [jump_one_fuel](const uint64_t milliparsecs) {
               return jump_one_fuel * std::max<uint64_t>(1, (milliparsecs + 999) / 1000);
            };
            proposal.steps.clear();
            auto projected_fuel = travel.current_fuel_millitons;
            for(size_t index = 0; index + 1 < course.waypoints.size(); ++index) {
               const auto source = course.waypoints[index].fuel_source;
               if(source == ct::CourseFuelSource::RefinedPort ||
                     source == ct::CourseFuelSource::UnrefinedPort) {
                  auto required = leg_fuel(course.waypoints[index].next_leg_milliparsecs);
                  for(size_t carried = index + 1;
                        carried + 1 < course.waypoints.size() &&
                        course.waypoints[carried].fuel_source ==
                        ct::CourseFuelSource::Carried;
                        ++carried) {
                     required += leg_fuel(course.waypoints[carried].next_leg_milliparsecs);
                  }
                  if(required > projected_fuel) {
                     const auto quantity = required - projected_fuel;
                     proposal.steps.push_back(purchase_fuel_step(
                                                course.waypoints[index].system_id,
                                                source,
                                                quantity));
                     projected_fuel += quantity;
                  }
               }
               auto step = jump_step(
                              course.waypoints[index].system_id,
                              course.waypoints[index + 1].system_id);
               step.action.jump_navigation = navigation_probe.jump_navigation;
               step.action.proceed_on_known_bad_plot =
                  navigation_probe.proceed_on_known_bad_plot;
               proposal.steps.push_back(step);
               proposal.steps.push_back(primary_dock_step(
                                           course.waypoints[index + 1].system_id,
                                           index + 2 == course.waypoints.size()
                                           ? ct::WaypointAuthority::Terminal
                                           : ct::WaypointAuthority::Through));
               projected_fuel -= leg_fuel(course.waypoints[index].next_leg_milliparsecs);
            }
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
      } else if(key == 'J') {
         if(!proposal.steps.empty() || destinations.current_system_id == 0) {
            door_warning(
               "A task shortcut can currently establish only the first leg "
               "from the present system.\n\r");
            wait_for_enter();
            continue;
         }
         try {
            const auto ledger = ct::get_task_ledger(
                                   connection, session_epoch, random_command_id(random), request_id++);
            std::vector<const ct::TaskRecord*> tasks;
            for(const auto& task : ledger.tasks) {
               if(task.offer.destination_system_id != 0 &&
                     task.state != ct::TaskState::Completed &&
                     task.state != ct::TaskState::Cancelled &&
                     task.state != ct::TaskState::Expired &&
                     task.state != ct::TaskState::Defaulted) {
                  tasks.push_back(&task);
               }
            }
            if(tasks.empty()) {
               door_information("No active task names a destination.\n\r");
               wait_for_enter();
               continue;
            }
            output().resume_paging();
            for(size_t index = 0; index < tasks.size(); ++index) {
               door_number("%zu", index + 1);
               door_label(". ");
               door_identifier("%s\n\r", safe_field(tasks[index]->offer.title).c_str());
            }
            const auto selected = input_number(
                                     "Task", 1, static_cast<unsigned>(tasks.size()));
            if(!selected) {
               continue;
            }
            const auto destination_id = tasks[*selected - 1]->offer.destination_system_id;
            const auto chart = std::find_if(
                                  destinations.systems.begin(), destinations.systems.end(),
            [destination_id](const auto & system) {
               return system.system_id == destination_id;
            });
            if(chart == destinations.systems.end() || !chart->within_jump_rating) {
               door_warning(
                  "That destination is not a direct jump. Use Import plotted "
                  "course and select the same primary.\n\r");
               wait_for_enter();
               continue;
            }
            auto jump = jump_step(destinations.current_system_id, destination_id);
            if(!configure_jump_navigation(jump.action)) {
               continue;
            }
            proposal.steps.push_back(jump);
            proposal.steps.push_back(primary_dock_step(
                                        destination_id, ct::WaypointAuthority::Terminal));
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
      } else if(key == 'G') {
         if(!proposal.steps.empty() || destinations.current_system_id == 0 ||
               travel.stage != ct::TravelStage::Docked) {
            door_warning(
               "A frontier-fuel operation can currently be added only as "
               "the first step while docked.\n\r");
            wait_for_enter();
            continue;
         }
         try {
            const auto services = ct::get_docked_services(
                                     connection, session_epoch, random_command_id(random), request_id++);
            std::vector<const ct::DockedFuelService*> sources;
            for(const auto& service : services.fuel) {
               if(service.available && service.source_body_id &&
                     (service.kind == ct::DockedFuelServiceKind::GasGiant ||
                      service.kind == ct::DockedFuelServiceKind::WildernessWater)) {
                  sources.push_back(&service);
               }
            }
            if(sources.empty()) {
               door_warning("No lawful frontier-fuel source is charted here.\n\r");
               wait_for_enter();
               continue;
            }
            output().resume_paging();
            for(size_t index = 0; index < sources.size(); ++index) {
               door_number("%zu", index + 1);
               door_label(". ");
               door_identifier("%s", safe_field(sources[index]->label).c_str());
               door_label("  maximum ");
               door_number("%.1f t\n\r", sources[index]->maximum_millitons / 1000.0);
            }
            const auto selected = input_number(
                                     "Fuel source", 1, static_cast<unsigned>(sources.size()));
            if(!selected) {
               continue;
            }
            const auto* source = sources[*selected - 1];
            const auto maximum_tons = static_cast<unsigned>(
                                         std::min<uint64_t>(source->maximum_millitons / 1000,
                                            std::numeric_limits<unsigned>::max()));
            const auto tons = input_number("Whole tons to collect", 1, maximum_tons);
            if(!tons) {
               continue;
            }
            proposal.steps.push_back(ct::FlightPlanStep{
               .locus = ct::FlightLocus{
                  .kind = ct::FlightLocusKind::Body,
                  .system_id = destinations.current_system_id,
                  .world_id = 0,
                  .facility_id = 0,
                  .body_id = *source->source_body_id},
               .authority = ct::WaypointAuthority::Through,
               .action = ct::FlightPlanAction{
                  .kind = ct::FlightPlanActionKind::Fuel,
                  .destination_system_id = 0,
                  .world_id = 0,
                  .facility_id = 0,
                  .fuel_operation = source->kind == ct::DockedFuelServiceKind::GasGiant
                  ? ct::FuelOperation::GasGiant
                  : ct::FuelOperation::WildernessWater,
                  .quantity_millitons = uint64_t{*tons} * 1000},
            });
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
      } else if(key == 'X') {
         if(!proposal.steps.empty() || destinations.current_system_id == 0) {
            door_warning(
               "An exploratory coordinate Jump must be the first leg filed "
               "from a system.\n\r");
            wait_for_enter();
            continue;
         }
         door_information(
            "Enter Earth-centred Galactic coordinates. Positive axes are "
            "coreward, spinward, and north.\n\r");
         const auto coreward = input_coordinate("Coreward parsecs", 0.0);
         if(!coreward) {
            continue;
         }
         const auto spinward = input_coordinate("Spinward parsecs", 0.0);
         if(!spinward) {
            continue;
         }
         const auto north = input_coordinate("North parsecs", 0.0);
         if(!north) {
            continue;
         }
         auto jump = coordinate_jump_step(
                        destinations.current_system_id, *coreward, *spinward, *north);
         if(!configure_jump_navigation(jump.action)) {
            continue;
         }
         proposal.steps.push_back(jump);
      } else if(key == 'D') {
         if(proposal.steps.size() >= 2 &&
               proposal.steps.back().action.kind == ct::FlightPlanActionKind::Dock &&
               proposal.steps[proposal.steps.size() - 2].action.kind == ct::FlightPlanActionKind::Jump) {
            proposal.steps.pop_back();
            proposal.steps.pop_back();
            if(!proposal.steps.empty() &&
                  proposal.steps.back().action.kind == ct::FlightPlanActionKind::Fuel &&
                  (proposal.steps.back().action.fuel_operation ==
                     ct::FuelOperation::BuyRefined ||
                   proposal.steps.back().action.fuel_operation ==
                     ct::FuelOperation::BuyUnrefined)) {
               proposal.steps.pop_back();
            }
         } else if(!proposal.steps.empty() &&
                   proposal.steps.back().action.kind ==
                   ct::FlightPlanActionKind::JumpCoordinates) {
            proposal.steps.pop_back();
         } else if(!proposal.steps.empty()) {
            proposal.steps.pop_back();
         }
      } else if(key == 'T') {
         if(proposal.steps.empty()) {
            continue;
         }
         auto& authority = proposal.steps.back().authority;
         authority = authority == ct::WaypointAuthority::Hold
                     ? ct::WaypointAuthority::Terminal
                     : authority == ct::WaypointAuthority::Terminal
                     ? ct::WaypointAuthority::Through
                     : ct::WaypointAuthority::Hold;
      } else if(key == 'P') {
         if(proposal.steps.empty()) {
            door_warning("A filed plan must contain at least one waypoint.\n\r");
            wait_for_enter();
            continue;
         }
         try {
            const auto preview = ct::preview_flight_plan(
                                    connection, session_epoch, proposal,
                                    random_command_id(random), request_id++);
            od_clr_scr();
            door_heading("Flight Plan Preview\n\r===================\n\r\n\r");
            door_label("Estimated time: ");
            door_number("%s\n\r", course_duration(preview.elapsed_seconds).c_str());
            door_label("Jump fuel:     ");
            door_number("%.1f t\n\r", preview.fuel_millitons / 1000.0);
            for(const auto& warning : preview.warnings) {
               if(warning.code == "TASK_DEADLINE_MISSED") {
                  print_wrapped_field(
                     "Deadline: ",
                     warning.message,
                     ct::DoorTextRole::Error);
               } else {
                  print_wrapped_field(
                     "Warning: ",
                     warning.message,
                     ct::DoorTextRole::Warning);
               }
            }
            door_option_prompt({
               "[F] File this plan", "[Q/Enter] Revise", "[?] Help"});
            const auto confirm = static_cast<char>(
                                    std::toupper(static_cast<unsigned char>(door_get_live_key())));
            if(confirm != 'F') {
               continue;
            }
            ct::commit_flight_plan(
               connection, session_epoch, proposal, preview.preview_hash, true,
               random_command_id(random), request_id++);
            return ct::get_travel_status(
                      connection, session_epoch, random_command_id(random), request_id++);
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
         }
      }
   }
}

void render_docked_menu(const ct::DockedSnapshot& snapshot)
{
   od_clr_scr();
   render_docked_snapshot(snapshot);
   for(const auto& entry : std::array{
   std::pair{'C', "Cargo Exchange"},
   std::pair{'J', "Jobs and Passage"},
   std::pair{'F', "Fuel and Supplies"},
   std::pair{'Y', "Shipyard"},
   std::pair{'P', "Personnel"},
   std::pair{'B', "Banking and Accounts"},
   std::pair{'A', "Authorities"},
   std::pair{'D', "Depart"},
}) {
      if((entry.first == 'P' && !snapshot.personnel_available) ||
         (entry.first == 'B' && !snapshot.banking_available) ||
         (entry.first == 'A' && !snapshot.authority_available)) {
         continue;
      }
      door_number("%c", entry.first);
      door_label(". ");
      door_identifier("%s\n\r", entry.second);
   }
   door_option_prompt({
      "[Letter] Docked service",
      "[U] Universal managers",
      "[L] License",
      "[Enter] Refresh",
      "[Q] Return to BBS",
      "[?] Help",
   });
}

ct::PlayerPhase run_docked_menu(
   ct::TlsConnection& connection,
   ct::ServerHello& hello)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Docked);
   ct::CommandIdGenerator random;
   uint64_t request_id = 1;
   while(true) {
      const auto snapshot = ct::get_docked_snapshot(
                               connection,
                               hello.assigned_epoch,
                               random_command_id(random),
                               request_id++);
      render_docked_menu(snapshot);
      const auto key = static_cast<char>(
                          std::toupper(static_cast<unsigned char>(door_get_live_key())));
      if(key == 'C') {
         run_cargo_exchange(
            connection, hello.assigned_epoch, random, request_id);
      } else if(key == 'J') {
         show_task_manager(connection, hello.assigned_epoch, random, request_id);
      } else if(key == 'B') {
         if(snapshot.banking_available) {
            show_finance(connection, hello.assigned_epoch, random, request_id);
         } else {
            door_warning("No recognized banking house operates at this port.\n\r");
            wait_for_enter();
         }
      } else if(key == 'Y') {
         run_shipyard_market(connection, hello.assigned_epoch, random, request_id);
      } else if(key == 'P') {
         if(snapshot.personnel_available) {
            run_crew_exchange(connection, hello.assigned_epoch, random, request_id);
         } else {
            door_warning("No crew exchange or shore service operates here.\n\r");
            wait_for_enter();
         }
      } else if(key == 'F') {
         const auto travel = run_fuel_service(
                                connection, hello.assigned_epoch, random, request_id);
         if(travel) {
            od_clr_scr();
            door_success("Fueling expedition underway.\n\r\n\r");
            door_label("Operation: ");
            door_value("%s\n\r", travel_stage_name(travel->stage));
            door_label("Next transition: ");
            door_number("%s", game_date(travel->due_second).c_str());
            door_label(" (");
            door_number("%s", real_time_until(*travel).c_str());
            door_label(")\n\r");
            wait_for_enter();
            return travel->phase;
         }
      } else if(key == 'D') {
         const auto travel = run_flight_plan_editor(
                                connection, hello.assigned_epoch, random, request_id);
         if(travel) {
            od_clr_scr();
            door_success("Departure authorized.\n\r\n\r");
            door_label("Destination: ");
            door_identifier("%s\n\r",
                            safe_field(travel->destination_system_name).c_str());
            door_label("Stage: ");
            door_value("%s\n\r", travel_stage_name(travel->stage));
            door_label("Next transition: ");
            door_number("%s", game_date(travel->due_second).c_str());
            door_label(" (");
            door_number("%s", real_time_until(*travel).c_str());
            door_label(")\n\r");
            door_information(
               "\n\rThe flight plan has been filed. The ship will continue "
               "on schedule if the captain leaves the bridge.\n\r");
            wait_for_enter();
            return travel->phase;
         }
      } else if(key == 'A') {
         if(snapshot.authority_available) {
            if(const auto phase = show_combat_operations(
                  connection, hello.assigned_epoch, random, request_id)) {
               hello.phase = *phase;
               return *phase;
            }
         } else {
            door_warning("No recognized authority maintains an office here.\n\r");
            wait_for_enter();
         }
      } else if(key == 'U') {
         if(run_command_console(
                  connection, hello, random, request_id)) {
            return ct::PlayerPhase::Disconnected;
         }
         if(hello.phase != ct::PlayerPhase::Docked) {
            return hello.phase;
         }
      } else if(key == 'L') {
         show_open_game_license();
      } else if(key == 'Q' && confirm_return_to_bbs()) {
         return ct::PlayerPhase::Disconnected;
      }
   }
}

void show_travel_screen(
   ct::TlsConnection& connection,
   const ct::ServerHello& hello,
   ct::CommandIdGenerator& random,
   uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Voyage);
   auto status = ct::get_travel_status(
                    connection,
                    hello.assigned_epoch,
                    random_command_id(random),
                    request_id++);
   od_clr_scr();
   door_heading("Voyage Status - ");
   door_value("%s\n\r", safe_field(status.ship_name).c_str());
   door_heading("=============\n\r\n\r");
   door_label("Flight state:");
   door_identifier("%s\n\r", phase_name(status.phase));
   door_label("Stage:       ");
   door_value("%s\n\r", travel_stage_name(status.stage));
   door_label("From:        ");
   door_identifier("%s\n\r", safe_field(status.current_system_name).c_str());
   door_label("Destination: ");
   door_identifier("%s\n\r", safe_field(status.destination_system_name).c_str());
   door_label("Ship time:   ");
   door_number("%s\n\r", game_date(status.current_game_second).c_str());
   door_label("Next event:  ");
   door_number("%s", game_date(status.due_second).c_str());
   door_label(" (");
   door_number("%s", real_time_until(status).c_str());
   door_label(")\n\r");
   door_label("Fuel:        ");
   door_number("%.1f t", status.current_fuel_millitons / 1000.0);
   door_label("  Jump use ");
   door_number("%.1f t\n\r", status.jump_fuel_millitons / 1000.0);
   door_information(
      "\n\rCrew, ship, task, message, and Known Universe management remain "
      "available while the scheduled voyage continues.\n\r");
   show_voyage_live_prompt();
   while(true) {
      const auto generation = phase_event_generation;
      const auto key = door_get_live_key();
      if(phase_event_generation != generation && latest_phase_status.has_value()) {
         status = *latest_phase_status;
         od_clr_scr();
         door_success("Voyage status changed: ");
         door_identifier("%s", phase_name(status.phase));
         door_label(" - ");
         door_value("%s\n\r", travel_stage_name(status.stage));
         if(status.phase == ct::PlayerPhase::Docked) {
            door_information("The ship is now docked.\n\r");
            return;
         }
         show_voyage_live_prompt();
         continue;
      }
      if(key == 'F' || key == 'f') {
         if(auto replanned = run_flight_plan_editor(connection, hello.assigned_epoch, random, request_id)) {
            status = *replanned;
         }
         od_clr_scr();
         door_information("Flight Plan updated.\n\r");
         wait_for_enter();
         return;
      }
      if(key == '\r' || key == '\n') {
         return;
      }
   }
}

ct::PlayerPhase run_arrival_checkpoint(ct::TlsConnection& connection, const ct::ServerHello& hello,
                                       ct::CommandIdGenerator& random, uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Arrival);
   if(!latest_checkpoint) {
      return ct::PlayerPhase::Interplanetary;
   }
   while(true) {
      od_clr_scr();
      door_heading("Arrival Checkpoint\n\r==================\n\r\n\r");
      door_information("The ship is holding clear of its destination until the captain takes the arrival watch.\n\r");
      door_label("Ready since: ");
      door_number("%s\n\r", game_date(latest_checkpoint->ready_second).c_str());
      door_option_prompt({
         "[A] Take arrival watch",
         "[Enter] Refresh",
         "[Q] Leave ship holding",
         "[?] Help",
      });
      const auto key = static_cast<char>(
         std::toupper(static_cast<unsigned char>(door_get_live_key())));
      if(key == '\r' || key == '\n') {
         continue;
      }
      if(key == 'Q') {
         return ct::PlayerPhase::Interplanetary;
      }
      if(key == 'A') {
         break;
      }
   }
   auto acknowledged = ct::acknowledge_checkpoint(connection, hello.assigned_epoch,
      latest_checkpoint->checkpoint_id, random_command_id(random), request_id++);
   latest_checkpoint.reset();
   collect_player_events();
   return acknowledged.phase;
}

const char* combat_range_name(const ct::CombatRange range)
{
   switch(range) {
   case ct::CombatRange::Adjacent:
      return "adjacent";
   case ct::CombatRange::Close:
      return "close";
   case ct::CombatRange::Short:
      return "short";
   case ct::CombatRange::Medium:
      return "medium";
   case ct::CombatRange::Long:
      return "long";
   case ct::CombatRange::VeryLong:
      return "very long";
   case ct::CombatRange::Distant:
      return "distant";
   }
   return "unknown";
}

std::optional<uint64_t> select_combat_actor(
   const ct::CombatSnapshot& combat,
   const ct::CombatActionKind kind,
   const std::unordered_set<uint64_t>& assigned)
{
   std::vector<const ct::CombatActor*> eligible;
   for(const auto& actor : combat.actors) {
      if(!actor.available || assigned.find(actor.person_id) != assigned.end()) {
         continue;
      }
      if(std::find(actor.allowed_actions.begin(), actor.allowed_actions.end(), kind) !=
            actor.allowed_actions.end()) {
         eligible.push_back(&actor);
      }
   }
   if(eligible.empty()) {
      door_error("No qualified watchstander remains available for that action.\n\r");
      wait_for_enter();
      return std::nullopt;
   }
   od_clr_scr();
   door_heading("Assign Combat Action\n\r====================\n\r\n\r");
   for(size_t index = 0; index < eligible.size(); ++index) {
      door_number("%zu", index + 1);
      door_label(". ");
      door_identifier("%s", safe_field(eligible[index]->name).c_str());
      door_label(" — ");
      door_value("%s\n\r", safe_field(eligible[index]->station).c_str());
   }
   const auto selected = input_number(
      "Watchstander", 1, static_cast<unsigned>(eligible.size()));
   if(!selected) {
      return std::nullopt;
   }
   return eligible[*selected - 1]->person_id;
}

std::optional<uint64_t> select_combat_reaction_actor(
   const ct::CombatSnapshot& combat,
   const ct::CombatReaction kind)
{
   std::vector<const ct::CombatActor*> eligible;
   for(const auto& actor : combat.actors) {
      if(actor.available &&
         std::find(actor.allowed_reactions.begin(), actor.allowed_reactions.end(), kind) !=
            actor.allowed_reactions.end()) {
         eligible.push_back(&actor);
      }
   }
   if(eligible.empty()) {
      door_error("No qualified watchstander is available for that reaction.\n\r");
      wait_for_enter();
      return std::nullopt;
   }
   od_clr_scr();
   door_heading("Assign Combat Reaction\n\r======================\n\r\n\r");
   for(size_t index = 0; index < eligible.size(); ++index) {
      door_number("%zu", index + 1);
      door_label(". ");
      door_identifier("%s", safe_field(eligible[index]->name).c_str());
      door_label(" — ");
      door_value("%s\n\r", safe_field(eligible[index]->station).c_str());
   }
   const auto selected = input_number(
      "Watchstander", 1, static_cast<unsigned>(eligible.size()));
   if(!selected) {
      return std::nullopt;
   }
   return eligible[*selected - 1]->person_id;
}

std::optional<uint64_t> select_combat_target(const ct::CombatSnapshot& combat)
{
   std::vector<const ct::CombatParticipant*> targets;
   for(const auto& participant : combat.participants) {
      if(!participant.commanded) {
         targets.push_back(&participant);
      }
   }
   if(targets.empty()) {
      return std::nullopt;
   }
   od_clr_scr();
   door_heading("Select Contact\n\r==============\n\r\n\r");
   for(size_t index = 0; index < targets.size(); ++index) {
      door_number("%zu", index + 1);
      door_label(". ");
      door_identifier("%s", safe_field(targets[index]->name).c_str());
      door_label(" — ");
      door_value("%s\n\r", safe_field(targets[index]->class_name).c_str());
   }
   const auto selected = input_number("Contact", 1, static_cast<unsigned>(targets.size()));
   if(!selected) {
      return std::nullopt;
   }
   return targets[*selected - 1]->vessel_id;
}

std::optional<uint16_t> select_combat_mount(const ct::CombatSnapshot& combat)
{
   const auto own = std::find_if(
      combat.participants.begin(), combat.participants.end(),
      [](const auto& participant) { return participant.commanded; });
   if(own == combat.participants.end() || own->weapons.empty()) {
      return std::nullopt;
   }
   od_clr_scr();
   door_heading("Select Weapon Mount\n\r===================\n\r\n\r");
   for(size_t index = 0; index < own->weapons.size(); ++index) {
      const auto& mount = own->weapons[index];
      door_number("%zu", index + 1);
      door_label(". ");
      door_identifier("%s", safe_field(mount.label).c_str());
      door_label("  damage ");
      door_number("%u\n\r", mount.damage_hits);
   }
   const auto selected = input_number(
      "Mount", 1, static_cast<unsigned>(own->weapons.size()));
   if(!selected) {
      return std::nullopt;
   }
   return own->weapons[*selected - 1].mount_id;
}

std::optional<ct::CombatOrderSet> edit_combat_order(const ct::CombatSnapshot& combat)
{
   const HelpScope help_scope(ct::DoorHelpTopic::CombatOrders);
   ct::CombatOrderSet order = combat.default_order;
   order.actions.clear();
   order.reactions = combat.default_order.reactions;
   order.use_tactical_controller = false;
   std::unordered_set<uint64_t> assigned;

   while(true) {
      od_clr_scr();
      door_heading("Joint Order Book\n\r================\n\r\n\r");
      door_label("Actions entered: ");
      door_number("%zu", order.actions.size());
      door_label("  Reactions standing: ");
      door_number("%zu\n\r\n\r", order.reactions.size());
      door_identifier("1. Command and signals\n\r");
      door_identifier("2. Helm and navigation\n\r");
      door_identifier("3. Sensors and electronic warfare\n\r");
      door_identifier("4. Gunnery\n\r");
      door_identifier("5. Damage control\n\r");
      door_identifier("6. Boarding\n\r");
      door_identifier("7. Escape craft\n\r");
      door_identifier("8. Reaction watch\n\r");
      door_success("9. Review and seal orders\n\r");
      door_prompt("\n\rSection (Q to discard, ? for help): ");
      const auto section = door_get_live_key();
      if(section == 'q' || section == 'Q' || section == '\r' || section == '\n') {
         return std::nullopt;
      }
      if(section == '9') {
         if(order.actions.empty()) {
            door_error("At least one action must be entered.\n\r");
            wait_for_enter();
            continue;
         }
         return order;
      }
      if(section == '8') {
         order.reactions.clear();
         while(true) {
            od_clr_scr();
            door_heading("Reaction Watch\n\r==============\n\r\n\r");
            door_identifier("1. Dodge\n\r2. Point defense\n\r3. Fire sand\n\r");
            door_identifier("4. Nuclear damper\n\r5. Meson screen\n\r");
            door_success("6. Finish reaction watch\n\r");
            const auto choice = input_number("Reaction", 1, 6);
            if(!choice || *choice == 6) {
               break;
            }
            const auto kind = static_cast<ct::CombatReaction>(*choice - 1);
            const auto actor = select_combat_reaction_actor(combat, kind);
            if(actor) {
               order.reactions.push_back({kind, *actor});
            }
         }
         continue;
      }

      std::optional<unsigned> choice;
      std::vector<ct::CombatActionKind> kinds;
      if(section == '1') {
         od_clr_scr();
         door_heading("Command and Signals\n\r===================\n\r\n\r");
         door_identifier("1. Coordinate crew\n\r2. Increase initiative\n\r");
         door_identifier("3. Offer surrender\n\r4. Accept surrender\n\r");
         choice = input_number("Action", 1, 4);
         kinds = {ct::CombatActionKind::Coordinate,
                  ct::CombatActionKind::IncreaseInitiative,
                  ct::CombatActionKind::OfferSurrender,
                  ct::CombatActionKind::AcceptSurrender};
      } else if(section == '2') {
         od_clr_scr();
         door_heading("Helm and Navigation\n\r===================\n\r\n\r");
         door_identifier("1. Evasive maneuvers\n\r2. Line up shot\n\r3. Close range\n\r");
         door_identifier("4. Open range\n\r5. Break pursuit\n\r6. Prepare Jump\n\r");
         choice = input_number("Action", 1, 6);
         kinds = {ct::CombatActionKind::EvasiveManeuvers,
                  ct::CombatActionKind::LineUpShot,
                  ct::CombatActionKind::RangeCheckClose,
                  ct::CombatActionKind::RangeCheckOpen,
                  ct::CombatActionKind::BreakPursuit,
                  ct::CombatActionKind::PrepareJump};
      } else if(section == '3') {
         od_clr_scr();
         door_heading("Sensors\n\r=======\n\r\n\r");
         door_identifier("1. Targeting solution\n\r2. Electronic warfare\n\r");
         door_identifier("3. Seal contact inspection\n\r");
         choice = input_number("Action", 1, 3);
         kinds = {ct::CombatActionKind::SensorTargeting,
                  ct::CombatActionKind::ElectronicWarfare,
                  ct::CombatActionKind::InspectContact};
      } else if(section == '4') {
         choice = 1;
         kinds = {ct::CombatActionKind::Attack};
      } else if(section == '5') {
         choice = 1;
         kinds = {ct::CombatActionKind::DamageControl};
      } else if(section == '6') {
         choice = 1;
         kinds = {ct::CombatActionKind::Board};
      } else if(section == '7') {
         choice = 1;
         kinds = {ct::CombatActionKind::LaunchEscapeCraft};
      } else {
         continue;
      }
      if(!choice) {
         continue;
      }
      const auto kind = kinds[*choice - 1];
      const auto actor = select_combat_actor(combat, kind, assigned);
      if(!actor) {
         continue;
      }
      uint16_t mount_id = 0;
      uint64_t target_id = 0;
      if(kind == ct::CombatActionKind::Attack) {
         const auto mount = select_combat_mount(combat);
         if(!mount) {
            continue;
         }
         mount_id = *mount;
      }
      if(kind == ct::CombatActionKind::Attack
            || kind == ct::CombatActionKind::Board
            || kind == ct::CombatActionKind::OfferSurrender
            || kind == ct::CombatActionKind::AcceptSurrender
            || kind == ct::CombatActionKind::InspectContact) {
         const auto target = select_combat_target(combat);
         if(!target) {
            continue;
         }
         target_id = *target;
      }
      assigned.insert(*actor);
      order.actions.push_back({kind, mount_id, target_id, *actor});
   }
}

ct::PlayerPhase run_combat(ct::TlsConnection& connection, const ct::ServerHello& hello,
                           ct::CommandIdGenerator& random, uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Combat);
   auto combat = ct::get_combat(connection, hello.assigned_epoch, random_command_id(random),
                                request_id++);
   od_clr_scr();
   door_heading("Vessel Combat\n\r=============\n\r\n\r");
   door_label("Round: ");
   door_number("%u", combat.round);
   door_label("  Range: ");
   door_value("%s\n\r", combat_range_name(combat.range));
   door_label("Order window: ");
   door_number("%.1f real seconds\n\r", combat.order_window_real_milliseconds / 1000.0);
   for(const auto& vessel : combat.participants) {
      if(vessel.commanded) {
         door_identifier("Your ship: ");
      } else {
         door_warning("Contact: ");
      }
      door_value("%s", safe_field(vessel.name).c_str());
      if(vessel.commanded) {
         door_success(" [YOUR SHIP]");
      }
      if(vessel.online_controlled) {
         door_success(" [ONLINE]");
      } else if(vessel.player_owned) {
         door_information(" [PLAYER]");
      }
      door_label("  ");
      door_value("%s\n\r", safe_field(vessel.class_name).c_str());
      door_label("  initiative ");
      door_number("%d", vessel.initiative);
      door_label("  thrust ");
      door_number("%uG", vessel.thrust);
      door_label("  hull/structure/armor ");
      door_number("%u/%u/%u\n\r", vessel.hull_remaining, vessel.structure_remaining,
                  vessel.armor_remaining);
      if(vessel.commanded) {
         for(const auto& mount : vessel.weapons) {
            door_label("  mount ");
            door_identifier("%u", mount.mount_id);
            door_label(" ");
            door_value("%s", safe_field(mount.label).c_str());
            door_label("  damage ");
            door_number("%u", mount.damage_hits);
            if(mount.ammunition_remaining != std::numeric_limits<uint32_t>::max()) {
               door_label("  ammunition ");
               door_number("%u", mount.ammunition_remaining);
            }
            door_label("\n\r");
         }
      }
   }
   if(!combat.actors.empty()) {
      door_heading("\n\rGeneral quarters\n\r");
      for(size_t index = 0; index < combat.actors.size(); ++index) {
         const auto& actor = combat.actors[index];
         door_number("%zu", index + 1);
         door_label(". ");
         door_identifier("%s", safe_field(actor.name).c_str());
         door_label(" — ");
         door_value("%s", safe_field(actor.station).c_str());
         if(!actor.available) {
            door_warning(" (unavailable)");
         }
         door_label("\n\r");
      }
   }
   if(!combat.log.empty()) {
      door_heading("\n\rSignal and damage log\n\r");
      const auto first = combat.log.size() > 8 ? combat.log.size() - 8 : 0;
      for(size_t i = first; i < combat.log.size(); ++i) {
         door_information("%s\n\r", safe_field(combat.log[i]).c_str());
      }
   }
   if(combat.complete) {
      door_information("\n\rThe engagement has ended.\n\r");
      wait_for_enter();
      return combat.phase;
   }
   if(combat.player_order_submitted) {
      door_information("\n\rJoint orders are sealed. The activation will resolve when the order window closes.\n\r");
      wait_for_enter();
      return combat.phase;
   }
   auto option_lines = ct::door_option_prompt({
      "[D] Conservative defaults",
      "[T] Tactical computer",
      "[E] Edit joint orders",
      "[A] Attack",
      "[W] Withdraw",
      "[B] Board",
      "[S] Offer surrender",
      "[P] Standing policy",
      "[Enter] Refresh",
      "[Q] Console",
      "[?] Help",
   }, output().columns(), true);
   if(option_lines.ends_with(": ")) {
      option_lines.resize(option_lines.size() - 2);
   }
   door_write(option_lines, ct::DoorTextRole::Prompt);
   door_write("\n\r", ct::DoorTextRole::Prompt);
   const auto timing = ct::get_ship_status(
      connection, hello.assigned_epoch, random_command_id(random), request_id++);
   const auto key_value = door_get_combat_countdown_key(
      combat_order_deadline(combat, timing.current_game_second));
   if(key_value == 0) {
      return combat.phase;
   }
   const auto key = static_cast<char>(
      std::toupper(static_cast<unsigned char>(key_value)));
   if(key == '\r' || key == '\n') {
      return combat.phase;
   }
   if(key == 'Q') {
      return combat.phase;
   }
   if(key == 'P') {
      auto threshold = input_number("Minimum success percent", 0, 100);
      if(!threshold) {
         return combat.phase;
      }
      door_label("Objective\n\r");
      door_option_prompt({
         "[S] Survive",
         "[W] Withdraw",
         "[D] Defeat",
         "[C] Capture",
      }, false);
      const auto objective_key = static_cast<char>(std::toupper(static_cast<unsigned char>
         (door_get_live_key())));
      od_printf("\n\r");
      auto objective = ct::CombatObjective::Survive;
      if(objective_key == 'W') {
         objective = ct::CombatObjective::Withdraw;
      } else if(objective_key == 'D') {
         objective = ct::CombatObjective::Defeat;
      } else if(objective_key == 'C') {
         objective = ct::CombatObjective::Capture;
      }
      const ct::CombatAutomationPolicy policy{
         .expected_revision = combat.policy.expected_revision,
         .minimum_victory_percent = static_cast<uint8_t>(*threshold),
         .objective = objective,
         .permit_surrender = true,
         .permit_abandon_ship = true,
      };
      combat = ct::set_combat_automation_policy(
                  connection,
                  hello.assigned_epoch,
                  policy,
                  random_command_id(random),
                  request_id++);
      door_success("Standing combat policy entered in the ship's log.\n\r");
      wait_for_enter();
      return combat.phase;
   }
   ct::CombatOrderSet order = combat.default_order;
   std::unordered_set<uint64_t> assigned_actors;
   const auto actor_for = [&](const ct::CombatActionKind kind) -> uint64_t {
      auto selected = std::find_if(combat.actors.begin(), combat.actors.end(),
         [&](const auto& actor) {
            return actor.available && assigned_actors.find(actor.person_id) == assigned_actors.end()
               && std::find(actor.allowed_actions.begin(), actor.allowed_actions.end(), kind) !=
                  actor.allowed_actions.end();
         });
      if(selected == combat.actors.end()) {
         return 0;
      }
      assigned_actors.insert(selected->person_id);
      return selected->person_id;
   };
   const auto target = std::find_if(
      combat.participants.begin(),
      combat.participants.end(),
      [](const auto& vessel) {
      return !vessel.commanded && vessel.disposition == ct::CombatDisposition::Active;
      });
   if(key == 'E') {
      const auto edited = edit_combat_order(combat);
      if(!edited) {
         return combat.phase;
      }
      order = *edited;
   } else if(key == 'T') {
      order.actions.clear();
      order.reactions.clear();
      order.use_tactical_controller = true;
   } else if(key == 'A' && target != combat.participants.end()) {
      order.actions.clear();
      for(const auto kind : {ct::CombatActionKind::Coordinate,
                             ct::CombatActionKind::LineUpShot,
                             ct::CombatActionKind::SensorTargeting}) {
         const auto actor = actor_for(kind);
         if(actor != 0) {
            order.actions.push_back({kind, 0, 0, actor});
         }
      }
      order.reactions = combat.default_order.reactions;
      const auto own = std::find_if(
         combat.participants.begin(),
         combat.participants.end(),
         [](const auto& vessel) {
            return vessel.commanded;
         });
      if(own != combat.participants.end()) {
         for(const auto& mount : own->weapons) {
            const auto actor = actor_for(ct::CombatActionKind::Attack);
            if(actor == 0) {
               break;
            }
            order.actions.push_back({
               ct::CombatActionKind::Attack,
               mount.mount_id,
               target->vessel_id,
               actor,
            });
         }
      }
   } else if(key == 'W') {
      order = combat.default_order;
   } else if(key == 'B' && target != combat.participants.end()) {
      order.actions.clear();
      if(const auto actor = actor_for(ct::CombatActionKind::RangeCheckClose); actor != 0) {
         order.actions.push_back({ct::CombatActionKind::RangeCheckClose, 0, 0, actor});
      }
      if(const auto actor = actor_for(ct::CombatActionKind::Board); actor != 0) {
         order.actions.push_back({ct::CombatActionKind::Board, 0, target->vessel_id, actor});
      }
      order.reactions = combat.default_order.reactions;
   } else if(key == 'S' && target != combat.participants.end()) {
      const auto actor = actor_for(ct::CombatActionKind::OfferSurrender);
      if(actor == 0) {
         door_error("No officer is available to tender surrender.\n\r");
         wait_for_enter();
         return combat.phase;
      }
      order.actions = {{ct::CombatActionKind::OfferSurrender, 0, target->vessel_id, actor}};
      order.reactions = combat.default_order.reactions;
   }
   combat = ct::submit_combat_order(connection, hello.assigned_epoch, order, random_command_id(random),
                                    request_id++);
   door_success("Joint orders sealed for this activation.\n\r");
   wait_for_enter();
   return combat.phase;
}

ct::PlayerPhase run_encounter(ct::TlsConnection& connection, const ct::ServerHello& hello,
                              ct::CommandIdGenerator& random, uint64_t& request_id)
{
   const HelpScope help_scope(ct::DoorHelpTopic::Encounter);
   auto encounter = latest_encounter.value_or(ct::get_encounter(connection, hello.assigned_epoch,
      random_command_id(random), request_id++));
   latest_encounter = encounter;
   od_clr_scr();
   door_heading("Contact on Arrival\n\r==================\n\r\n\r");
   door_label("Contact: ");
   door_identifier("%s\n\r", safe_field(encounter.contact.ship_name).c_str());
   door_label("Classification: ");
   door_value("%s  confidence %u%%\n\r", safe_field(encounter.contact.class_name).c_str(),
              encounter.contact.confidence_percent);
   door_information("%s\n\r", safe_field(encounter.summary).c_str());
   if(encounter.state == ct::EncounterState::Resolving) {
      try {
         return run_combat(connection, hello, random, request_id);
      } catch(const std::runtime_error&) {
         // Non-combat encounter responses still use the same resolving state
         // while their authoritative turn is queued.
      }
      door_information("\n\rThe ship's declared response is awaiting resolution.\n\r");
      wait_for_enter();
      return ct::PlayerPhase::Encounter;
   }
   door_option_prompt({
      "[F] Fight",
      "[R] Run",
      "[C] Comply",
      "[S] Surrender",
      "[B] Board",
      "[Enter] Refresh",
      "[?] Help",
   });
   char key = static_cast<char>(std::toupper(static_cast<unsigned char>(door_get_live_key())));
   if(key == '\r' || key == '\n') {
      latest_encounter.reset();
      return ct::PlayerPhase::Encounter;
   }
   std::optional<ct::EncounterPosture> posture;
   if(key == 'F') {
      posture = ct::EncounterPosture::Fight;
   } else if(key == 'R') {
      posture = ct::EncounterPosture::Flee;
   } else if(key == 'C') {
      posture = ct::EncounterPosture::Comply;
   } else if(key == 'S') {
      posture = ct::EncounterPosture::Surrender;
   } else if(key == 'B') {
      posture = ct::EncounterPosture::Board;
   }
   if(!posture.has_value()) {
      return ct::PlayerPhase::Encounter;
   }
   const std::vector<ct::EncounterFallback> fallbacks{
      ct::EncounterFallback::JettisonCargo,
      ct::EncounterFallback::Surrender,
   };
   auto result = ct::resolve_encounter(
                    connection,
                    hello.assigned_epoch,
                    encounter.encounter_id,
                    encounter.revision,
                    *posture,
                    fallbacks,
                    random_command_id(random),
                    request_id++);
   door_information("\n\r%s\n\r", safe_field(result.outcome).c_str());
   wait_for_enter();
   latest_encounter.reset();
   return result.phase;
}

void run_operational_loop(ct::TlsConnection& connection, ct::ServerHello& hello)
{
   ct::CommandIdGenerator random;
   uint64_t request_id = 1000;
   uint64_t packet_generation = std::numeric_limits<uint64_t>::max();
   for(;;) {
      collect_player_events();
      if(latest_phase_status.has_value()) {
         hello.phase = latest_phase_status->phase;
      }
      if(packet_generation != phase_event_generation) {
         if(hello.phase == ct::PlayerPhase::Docked ||
               hello.phase == ct::PlayerPhase::Interplanetary) {
            show_arrival_packet(
               connection,
               hello.assigned_epoch,
               random,
               request_id);
         }
         packet_generation = phase_event_generation;
      }
      if(hello.phase == ct::PlayerPhase::Docked) {
         hello.phase = run_docked_menu(connection, hello);
         if(hello.phase == ct::PlayerPhase::Disconnected) {
            return;
         }
         continue;
      }
      if(hello.phase == ct::PlayerPhase::Interplanetary && latest_checkpoint) {
         hello.phase = run_arrival_checkpoint(connection, hello, random, request_id);
         collect_player_events();
         if(latest_encounter) {
            hello.phase = ct::PlayerPhase::Encounter;
         }
         continue;
      }
      if(hello.phase == ct::PlayerPhase::Encounter) {
         hello.phase = run_encounter(connection, hello, random, request_id);
         continue;
      }
      if(hello.phase == ct::PlayerPhase::Terminal) {
         od_clr_scr();
         door_heading("Command Recovery\n\r================\n\r\n\r");
         try {
            const auto encounter = ct::get_encounter(
               connection, hello.assigned_epoch, random_command_id(random), request_id++);
            door_warning("%s\n\r\n\r", safe_field(encounter.summary).c_str());
         } catch(const std::exception&) {
            door_warning("The command is recorded as lost.\n\r\n\r");
         }
         door_information(
            "A surviving captain remains the same person through rescue, custody, or parole. "
            "Only a recorded death opens the estate to a successor.\n\r");
         door_option_prompt({
            "[R] Review recovery or succession",
            "[Enter] Refresh",
            "[Q] Leave game",
         });
         const auto action = static_cast<char>(std::toupper(
            static_cast<unsigned char>(od_get_key(TRUE))));
         if(action == 'Q') {
            if(confirm_return_to_bbs()) {
               return;
            }
            continue;
         }
         if(action != 'R') {
            continue;
         }
         od_printf("\n\r");
         const auto successor = input_text(
            "Successor name (leave empty if captain survived)", "", 80);
         if(!successor) {
            continue;
         }
         try {
            ct::recover_command(
               connection, hello.assigned_epoch, *successor,
               random_command_id(random), request_id++);
            hello.phase = ct::PlayerPhase::Docked;
            continue;
         } catch(const std::exception& error) {
            door_error("%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter();
            continue;
         }
      }
      if(hello.phase == ct::PlayerPhase::Interplanetary ||
            hello.phase == ct::PlayerPhase::Jump) {
         show_travel_screen(connection, hello, random, request_id);
         if(latest_phase_status.has_value()) {
            hello.phase = latest_phase_status->phase;
         }
         if(hello.phase == ct::PlayerPhase::Docked) {
            continue;
         }
      }
      if(run_command_console(connection, hello, random, request_id)) {
         return;
      }
   }
}

void render_error(const std::exception& error)
{
   od_clr_scr();
   door_error("\n\r  Cepheus Trader could not start:\n\r\n\r");
   door_error("  %s\n\r", safe_field(error.what()).c_str());
}

}  // namespace

int main(int argc, char** argv)
{
   if(argc == 2 && std::string_view(argv[1]) == "--version") {
      std::printf("cepheus-trader-door %s\n", CT_PRODUCT_VERSION);
      return 0;
   }
   // OpenDoors local mode is also the executable acceptance boundary. Keep
   // scrolling output observable when stdout is a pipe instead of a terminal;
   // otherwise stdio may retain complete screens until process exit.
   std::setvbuf(stdout, nullptr, _IONBF, 0);
   try {
      initialize_opendoors(argc, argv);
   } catch(const std::exception& error) {
      std::fprintf(stderr, "%s\n", error.what());
      return 2;
   }
   int result = 0;
   try {
      const auto config = ct::read_bbs_config(bbs_config_path);
      od_control.od_inactivity =
         static_cast<INT16>(config.inactivity_timeout_seconds);
      initialize_presentation(config);
      if(!await_startup_choice()) {
         od_exit(0, FALSE);
         return 0;
      }
      auto credential = ct::read_bbs_credential_file(config.credential_path);
      const std::string account_name =
         config.identity_name_field == ct::IdentityNameField::Handle
            ? od_control.user_handle
            : od_control.user_name;
      const std::optional<uint32_t> record_index =
         od_control.user_num == 0
            ? std::nullopt
            : std::optional<uint32_t>(od_control.user_num);
      const auto player_id = ct::resolve_player_identity(
         config.identity_registry_path,
         credential.bbs_id,
         account_name,
         record_index);
      ct::TlsConnection connection(
         config.server,
         config.game_port,
         std::to_string(credential.bbs_id),
         std::move(credential.psk));
      const ct::PlayerIdentity identity{
         .bbs_id = credential.bbs_id,
         .player_id = player_id,
      };
      auto hello = ct::exchange_hello(
         connection,
         identity,
         "cepheus-trader-door/" CT_PRODUCT_VERSION,
         "en-US");
      output().set_display_formatting(hello.formatting);
      event_connection = &connection;
      event_session_epoch = hello.assigned_epoch;
      render_hello(hello, connection);
      bool session_finished = false;
      while(!session_finished) {
         try {
            if(hello.phase == ct::PlayerPhase::NewUser) {
               if(run_player_creation(connection, hello)) {
                  hello.phase = ct::PlayerPhase::Docked;
                  run_operational_loop(connection, hello);
               }
            } else {
               run_operational_loop(connection, hello);
            }
            session_finished = true;
         } catch(const ct::PlayerRequestRejected& error) {
            door_error("\n\r%s\n\r", safe_field(error.what()).c_str());
            wait_for_enter("Continue");
         }
      }
      event_connection = nullptr;
   } catch(const std::exception& error) {
      render_error(error);
      result = 1;
      if(presentation) {
         door_option_prompt({
            "[L] License and copyright notices", "[Q] Return to BBS"});
         while(true) {
            const auto key = od_get_key(TRUE);
            if(key == 'l' || key == 'L') {
               show_open_game_license();
               render_error(error);
               door_option_prompt({
                  "[L] License and copyright notices", "[Q] Return to BBS"});
            } else if((key == 'q' || key == 'Q') &&
                      confirm_return_to_bbs()) {
               break;
            } else if(key == 'q' || key == 'Q') {
               render_error(error);
               door_option_prompt({
                  "[L] License and copyright notices", "[Q] Return to BBS"});
            }
         }
      }
   }
   od_exit(result, FALSE);
   return result;
}
