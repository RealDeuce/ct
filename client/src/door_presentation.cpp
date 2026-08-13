#include "ct/door_presentation.hpp"

#include <algorithm>
#include <array>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace ct {
namespace {

constexpr std::array<char32_t, 128> CP437_HIGH{
   U'\u00c7', U'\u00fc', U'\u00e9', U'\u00e2', U'\u00e4', U'\u00e0', U'\u00e5', U'\u00e7',
   U'\u00ea', U'\u00eb', U'\u00e8', U'\u00ef', U'\u00ee', U'\u00ec', U'\u00c4', U'\u00c5',
   U'\u00c9', U'\u00e6', U'\u00c6', U'\u00f4', U'\u00f6', U'\u00f2', U'\u00fb', U'\u00f9',
   U'\u00ff', U'\u00d6', U'\u00dc', U'\u00a2', U'\u00a3', U'\u00a5', U'\u20a7', U'\u0192',
   U'\u00e1', U'\u00ed', U'\u00f3', U'\u00fa', U'\u00f1', U'\u00d1', U'\u00aa', U'\u00ba',
   U'\u00bf', U'\u2310', U'\u00ac', U'\u00bd', U'\u00bc', U'\u00a1', U'\u00ab', U'\u00bb',
   U'\u2591', U'\u2592', U'\u2593', U'\u2502', U'\u2524', U'\u2561', U'\u2562', U'\u2556',
   U'\u2555', U'\u2563', U'\u2551', U'\u2557', U'\u255d', U'\u255c', U'\u255b', U'\u2510',
   U'\u2514', U'\u2534', U'\u252c', U'\u251c', U'\u2500', U'\u253c', U'\u255e', U'\u255f',
   U'\u255a', U'\u2554', U'\u2569', U'\u2566', U'\u2560', U'\u2550', U'\u256c', U'\u2567',
   U'\u2568', U'\u2564', U'\u2565', U'\u2559', U'\u2558', U'\u2552', U'\u2553', U'\u256b',
   U'\u256a', U'\u2518', U'\u250c', U'\u2588', U'\u2584', U'\u258c', U'\u2590', U'\u2580',
   U'\u03b1', U'\u00df', U'\u0393', U'\u03c0', U'\u03a3', U'\u03c3', U'\u00b5', U'\u03c4',
   U'\u03a6', U'\u0398', U'\u03a9', U'\u03b4', U'\u221e', U'\u03c6', U'\u03b5', U'\u2229',
   U'\u2261', U'\u00b1', U'\u2265', U'\u2264', U'\u2320', U'\u2321', U'\u00f7', U'\u2248',
   U'\u00b0', U'\u2219', U'\u00b7', U'\u221a', U'\u207f', U'\u00b2', U'\u25a0', U'\u00a0',
};

std::string option_shortcut_sort_key(const std::string_view option) {
   // Sort on the shortcut between brackets, never on the translated label or
   // source-code order. A localized client can therefore choose its displayed
   // shortcuts first and get the appropriate order without English collation.
   // Help remains the conventional final choice.
   const auto opening = option.find('[');
   const auto closing = opening == std::string_view::npos
      ? std::string_view::npos
      : option.find(']', opening + 1);
   const auto shortcut = closing == std::string_view::npos
      ? option
      : option.substr(opening + 1, closing - opening - 1);
   std::string key;
   key.reserve(shortcut.size() + 1);
   const auto first = shortcut.empty() ? '\0' : shortcut.front();
   if(first >= '0' && first <= '9') {
      key.push_back('0');
   } else if((first >= 'A' && first <= 'Z') ||
             (first >= 'a' && first <= 'z')) {
      key.push_back('2');
   } else if(first == '?') {
      key.push_back('3');
   } else {
      key.push_back('1');
   }
   for(const auto character : shortcut) {
      key.push_back(character >= 'a' && character <= 'z'
         ? static_cast<char>(character - 'a' + 'A')
         : character);
   }
   return key;
}

bool is_iso646_invariant(const char32_t value) {
   if((value >= U'0' && value <= U'9') ||
      (value >= U'A' && value <= U'Z') ||
      (value >= U'a' && value <= U'z')) {
      return true;
   }
   constexpr std::string_view punctuation =
      " !\"%&'()*+,-./:;<=>?_";
   return value < 128 &&
          punctuation.find(static_cast<char>(value)) != std::string_view::npos;
}

std::pair<char32_t, size_t> decode_utf8(const std::string_view text,
                                       const size_t offset) {
   const auto first = static_cast<uint8_t>(text[offset]);
   if(first < 0x80) {
      return {
         first,
         1,
      };
   }
   size_t length = 0;
   char32_t value = 0;
   if((first & 0xe0) == 0xc0) {
      length = 2;
      value = first & 0x1f;
   } else if((first & 0xf0) == 0xe0) {
      length = 3;
      value = first & 0x0f;
   } else if((first & 0xf8) == 0xf0) {
      length = 4;
      value = first & 0x07;
   } else {
      return {
         U'\ufffd',
         1,
      };
   }
   if(offset + length > text.size()) {
      return {
         U'\ufffd',
         1,
      };
   }
   for(size_t index = 1; index < length; ++index) {
      const auto next = static_cast<uint8_t>(text[offset + index]);
      if((next & 0xc0) != 0x80) {
         return {
            U'\ufffd',
            1,
         };
      }
      value = (value << 6) | (next & 0x3f);
   }
   const bool overlong =
      (length == 2 && value < 0x80) ||
      (length == 3 && value < 0x800) ||
      (length == 4 && value < 0x10000);
   if(overlong || value > 0x10ffff ||
      (value >= 0xd800 && value <= 0xdfff)) {
      return {
         U'\ufffd',
         1,
      };
   }
   return {
      value,
      length,
   };
}

std::string iso646_text(const char32_t value) {
   if(is_iso646_invariant(value)) {
      return std::string(1, static_cast<char>(value));
   }
   switch(value) {
      case U'#':
         return "No.";
      case U'$':
         return "USD";
      case U'@':
         return "(at)";
      case U'[':
      case U'{':
         return "(";
      case U']':
      case U'}':
         return ")";
      case U'\\':
         return "/";
      case U'^':
         return "up";
      case U'`':
      case U'\u2018':
      case U'\u2019':
         return "'";
      case U'|':
         return "!";
      case U'~':
      case U'\u2010':
      case U'\u2011':
      case U'\u2012':
      case U'\u2013':
      case U'\u2014':
      case U'\u2212':
         return "-";
      case U'\u201c':
      case U'\u201d':
         return "\"";
      case U'\u2026':
         return "...";
      case U'\u00a0':
         return " ";
      case U'\u2190':
         return "<-";
      case U'\u2192':
         return "->";
      case U'\u2191':
         return "up";
      case U'\u2193':
         return "down";
      default:
         return "?";
   }
}

std::string cp437_text(const char32_t value) {
   if(value >= 0x20 && value <= 0x7e) {
      return std::string(1, static_cast<char>(value));
   }
   const auto found = std::find(CP437_HIGH.begin(), CP437_HIGH.end(), value);
   if(found != CP437_HIGH.end()) {
      const auto code = 0x80 + std::distance(CP437_HIGH.begin(), found);
      return std::string(1, static_cast<char>(code));
   }
   switch(value) {
      case U'\u2010':
      case U'\u2011':
      case U'\u2012':
      case U'\u2013':
      case U'\u2014':
      case U'\u2212':
         return "-";
      case U'\u2018':
      case U'\u2019':
         return "'";
      case U'\u201c':
      case U'\u201d':
         return "\"";
      case U'\u2026':
         return "...";
      case U'\u2190':
         return "<-";
      case U'\u2192':
         return "->";
      case U'\u2191':
         return "^";
      case U'\u2193':
         return "v";
      default:
         return "?";
   }
}

std::string encode_text(const std::string_view text,
                        const DoorProfile profile) {
   std::string result;
   result.reserve(text.size());
   for(size_t offset = 0; offset < text.size();) {
      const auto [value, length] = decode_utf8(text, offset);
      offset += length;
      if(value == U'\r' || value == U'\n') {
         result.push_back(static_cast<char>(value));
      } else if(value == U'\t') {
         result.push_back(' ');
      } else if(value < 0x20 || value == 0x7f) {
         result.push_back('?');
      } else {
         result += door_profile_uses_cp437(profile)
                      ? cp437_text(value)
                      : iso646_text(value);
      }
   }
   return result;
}

std::string_view role_sequence(const DoorTextRole role) {
   // A high-contrast BBS palette inspired by TradeWars-era data screens and
   // Yankee Trader's rotating DOS colours. Roles, rather than individual
   // fields, determine colour so later screens remain visually consistent.
   switch(role) {
      case DoorTextRole::Normal:
         return "\x1b[0m";
      case DoorTextRole::Heading:
         return "\x1b[1;36m";
      case DoorTextRole::Accent:
         return "\x1b[1;33m";
      case DoorTextRole::Prompt:
         return "\x1b[1;32m";
      case DoorTextRole::Error:
         return "\x1b[1;31m";
      case DoorTextRole::Muted:
         return "\x1b[37m";
      case DoorTextRole::Label:
         return "\x1b[36m";
      case DoorTextRole::Value:
         return "\x1b[1;37m";
      case DoorTextRole::Number:
         return "\x1b[1;33m";
      case DoorTextRole::Identifier:
         return "\x1b[1;35m";
      case DoorTextRole::Information:
         return "\x1b[32m";
      case DoorTextRole::Success:
         return "\x1b[1;32m";
      case DoorTextRole::Warning:
         return "\x1b[1;31m";
   }
   return "\x1b[0m";
}

}  // namespace

DoorProfile parse_door_profile(const std::string_view text) {
   if(text == "iso646" || text == "plain") {
      return DoorProfile::Iso646;
   }
   if(text == "iso646-color" || text == "color" || text == "colour") {
      return DoorProfile::Iso646Color;
   }
   if(text == "cp437-plain") {
      return DoorProfile::Cp437;
   }
   if(text == "cp437-color" || text == "cp437") {
      return DoorProfile::Cp437Color;
   }
   throw std::invalid_argument(
      "profile must be iso646, iso646-color, cp437-plain, or cp437-color");
}

const char* door_profile_name(const DoorProfile profile) {
   switch(profile) {
      case DoorProfile::Iso646:
         return "iso646";
      case DoorProfile::Iso646Color:
         return "iso646-color";
      case DoorProfile::Cp437:
         return "cp437-plain";
      case DoorProfile::Cp437Color:
         return "cp437-color";
   }
   return "iso646";
}

DoorProfile door_profile_for_capabilities(const bool ansi,
                                           const bool eight_bit) noexcept {
   if(ansi) {
      return eight_bit ? DoorProfile::Cp437Color
                       : DoorProfile::Iso646Color;
   }
   return eight_bit ? DoorProfile::Cp437 : DoorProfile::Iso646;
}

bool door_profile_uses_ansi(const DoorProfile profile) noexcept {
   return profile == DoorProfile::Iso646Color ||
          profile == DoorProfile::Cp437Color;
}

bool door_profile_uses_cp437(const DoorProfile profile) noexcept {
   return profile == DoorProfile::Cp437 ||
          profile == DoorProfile::Cp437Color;
}

std::string door_single_line_field(const std::string_view text) {
   std::string result;
   result.reserve(text.size());
   for(const unsigned char byte : text) {
      if(byte == '\r' || byte == '\n' || byte == '\t') {
         result.push_back(' ');
      } else if(byte < 0x20 || byte == 0x7f) {
         result.push_back('?');
      } else {
         result.push_back(static_cast<char>(byte));
      }
   }
   return result;
}

std::string door_plain_markdown(const std::string_view text) {
   std::string result;
   size_t line_start = 0;
   while(line_start < text.size()) {
      const auto newline = text.find('\n', line_start);
      const auto line_end = newline == std::string_view::npos
                            ? text.size()
                            : newline;
      auto line = text.substr(line_start, line_end - line_start);
      if(!line.empty() && line.back() == '\r') {
         line.remove_suffix(1);
      }
      size_t content_start = 0;
      while(content_start < line.size() && line[content_start] == '#') {
         ++content_start;
      }
      if(content_start == 0 || content_start >= line.size() ||
         line[content_start] != ' ') {
         content_start = 0;
      } else {
         ++content_start;
      }
      for(size_t index = content_start; index < line.size();) {
         if(line[index] == '[') {
            const auto label_end = line.find(']', index + 1);
            if(label_end != std::string_view::npos &&
               label_end + 1 < line.size() && line[label_end + 1] == '(') {
               const auto target_end = line.find(')', label_end + 2);
               if(target_end != std::string_view::npos) {
                  result.append(line.substr(index + 1, label_end - index - 1));
                  index = target_end + 1;
                  continue;
               }
            }
         }
         result.push_back(line[index]);
         ++index;
      }
      if(newline == std::string_view::npos) {
         break;
      }
      result.push_back('\n');
      line_start = newline + 1;
   }
   return result;
}

std::string door_option_prompt(
   const std::span<const std::string_view> options,
   const size_t columns,
   const bool leading_newline)
{
   if(columns == 0) {
      throw std::invalid_argument("door prompt width must be nonzero");
   }

   // Do not consume the terminal's physical last column.  Terminals disagree
   // about whether writing it leaves a deferred wrap pending or advances
   // immediately, so following a full-width line with CR/LF can produce an
   // extra blank line.
   const auto content_columns = columns > 1 ? columns - 1 : size_t{1};
   std::string prompt;
   if(leading_newline) {
      prompt += "\n\r";
   }
   std::vector<std::string_view> sorted_options(options.begin(), options.end());
   std::stable_sort(
      sorted_options.begin(), sorted_options.end(),
      [](const auto left, const auto right) {
         return option_shortcut_sort_key(left) < option_shortcut_sort_key(right);
      });
   size_t line_width = 0;
   for(const auto option : sorted_options) {
      if(option.empty()) {
         continue;
      }
      if(line_width != 0) {
         if(line_width + 2 + option.size() > content_columns) {
            prompt += "\n\r";
            line_width = 0;
         } else {
            prompt += "  ";
            line_width += 2;
         }
      }
      prompt += option;
      line_width += option.size();
   }
   prompt += ": ";
   return prompt;
}

std::string door_option_prompt(
   const std::initializer_list<std::string_view> options,
   const size_t columns,
   const bool leading_newline)
{
   return door_option_prompt(
      std::span<const std::string_view>(options.begin(), options.size()),
      columns,
      leading_newline);
}

DoorPresentation::DoorPresentation(const DoorProfile profile,
                                   const size_t columns,
                                   const size_t rows,
                                   Sink sink)
   : profile_(profile),
     columns_(columns),
     rows_(rows),
     sink_(std::move(sink)) {
   if(columns_ < 40 || rows_ < 24) {
      throw std::invalid_argument(
         "door geometry must be at least 40 columns by 24 rows");
   }
   if(!sink_) {
      throw std::invalid_argument("door presentation requires an output sink");
   }
}

DoorProfile DoorPresentation::profile() const noexcept {
   return profile_;
}

size_t DoorPresentation::columns() const noexcept {
   return columns_;
}

size_t DoorPresentation::content_columns() const noexcept {
   return columns_ - 1;
}

size_t DoorPresentation::rows() const noexcept {
   return rows_;
}

size_t DoorPresentation::page_content_rows(
   const size_t reserved_rows) const noexcept {
   return rows_ > reserved_rows ? rows_ - reserved_rows : 1;
}

const DisplayFormatting& DoorPresentation::display_formatting() const noexcept {
   return display_formatting_;
}

void DoorPresentation::set_display_formatting(DisplayFormatting formatting) {
   validate_display_formatting(formatting);
   display_formatting_ = std::move(formatting);
}

void DoorPresentation::configure_paging(const size_t reserved_rows,
                                        PagePause pause) {
   if(!pause) {
      throw std::invalid_argument("door pagination requires a pause callback");
   }
   paging_content_rows_ = page_content_rows(reserved_rows);
   page_pause_ = std::move(pause);
}

void DoorPresentation::set_paging_enabled(const bool enabled) noexcept {
   paging_enabled_ = enabled;
   if(!paging_enabled_) {
      paging_active_ = false;
   }
}

void DoorPresentation::resume_paging() noexcept {
   paging_active_ = paging_enabled_ && static_cast<bool>(page_pause_) &&
                    !paging_suppressed_until_input_;
}

void DoorPresentation::suspend_paging() noexcept {
   paging_active_ = false;
}

void DoorPresentation::suppress_paging_until_input() noexcept {
   paging_suppressed_until_input_ = true;
   paging_active_ = false;
}

void DoorPresentation::reset_paging() noexcept {
   row_ = 0;
   paging_suppressed_until_input_ = false;
}

void DoorPresentation::reset_after_external_input() noexcept {
   column_ = 0;
   reset_paging();
}

void DoorPresentation::erase_prompt(const size_t visible_columns) {
   flush();
   emit("\r");
   emit(std::string(visible_columns, ' '));
   emit("\r");
   column_ = 0;
   flush();
}

void DoorPresentation::emit(const std::string_view bytes) {
   pending_.append(bytes);
}

void DoorPresentation::flush() {
   if(!pending_.empty()) {
      sink_(pending_);
      pending_.clear();
   }
}

void DoorPresentation::clear() {
   column_ = 0;
   row_ = 0;
   if(!door_profile_uses_ansi(profile_)) {
      emit("\f");
   } else {
      emit("\x1b[0m\x1b[2J\x1b[H");
   }
   flush();
}

void DoorPresentation::newline(const DoorTextRole role) {
   emit("\r\n");
   column_ = 0;
   ++row_;
   if(paging_active_ && !handling_page_pause_ &&
      row_ >= paging_content_rows_) {
      pause_at_page_boundary(role);
   }
}

void DoorPresentation::wrapped_newline(
   const DoorTextRole role,
   const size_t continuation_indent)
{
   newline(role);
   if(!write_aborted_ && continuation_indent != 0) {
      emit(std::string(continuation_indent, ' '));
      column_ = continuation_indent;
   }
}

void DoorPresentation::pause_at_page_boundary(const DoorTextRole role) {
   flush();
   handling_page_pause_ = true;
   paging_active_ = false;
   try {
      const auto action = page_pause_();
      if(action == PagePauseAction::Continuous) {
         paging_suppressed_until_input_ = true;
      } else if(action == PagePauseAction::Abort) {
         write_aborted_ = true;
      }
   } catch(...) {
      handling_page_pause_ = false;
      paging_active_ = paging_enabled_ && static_cast<bool>(page_pause_) &&
                       !paging_suppressed_until_input_;
      throw;
   }
   row_ = 0;
   handling_page_pause_ = false;
   paging_active_ = paging_enabled_ && static_cast<bool>(page_pause_) &&
                    !paging_suppressed_until_input_;
   if(door_profile_uses_ansi(profile_)) {
      emit(role_sequence(role));
   }
}

void DoorPresentation::emit_encoded(const std::string_view bytes,
                                    const DoorTextRole role,
                                    const size_t continuation_indent) {
   const auto line_limit = content_columns();
   for(size_t index = 0; index < bytes.size();) {
      if(write_aborted_) {
         break;
      }
      const char byte = bytes[index];
      if(byte == '\r' || byte == '\n') {
         newline(role);
         if(write_aborted_) {
            break;
         }
         if(index + 1 < bytes.size()) {
            const char next = bytes[index + 1];
            if((byte == '\r' && next == '\n') ||
               (byte == '\n' && next == '\r')) {
               ++index;
            }
         }
         ++index;
         continue;
      }

      if(byte == ' ') {
         const auto first_space = index;
         while(index < bytes.size() && bytes[index] == ' ') {
            ++index;
         }
         const auto space_count = index - first_space;
         auto next_word_end = index;
         while(next_word_end < bytes.size() &&
               bytes[next_word_end] != ' ' &&
               bytes[next_word_end] != '\r' &&
               bytes[next_word_end] != '\n') {
            ++next_word_end;
         }
         const auto next_word_size = next_word_end - index;
         if(column_ != 0 && next_word_size != 0 &&
            column_ + space_count + next_word_size > line_limit) {
            wrapped_newline(role, continuation_indent);
            if(write_aborted_) {
               break;
            }
            continue;
         }
         size_t remaining = space_count;
         while(remaining != 0) {
            if(column_ >= line_limit) {
               wrapped_newline(role, continuation_indent);
               if(write_aborted_) {
                  break;
               }
            }
            const auto count =
               std::min(remaining, line_limit - column_);
            emit(std::string_view(bytes.data() + first_space, count));
            column_ += count;
            remaining -= count;
         }
         continue;
      }

      const auto word_start = index;
      while(index < bytes.size() &&
            bytes[index] != ' ' &&
            bytes[index] != '\r' &&
            bytes[index] != '\n') {
         ++index;
      }
      size_t remaining = index - word_start;
      size_t emitted = 0;
      if(column_ != 0 && column_ + remaining > line_limit) {
         wrapped_newline(role, continuation_indent);
         if(write_aborted_) {
            break;
         }
      }
      while(remaining != 0) {
         if(column_ >= line_limit) {
            wrapped_newline(role, continuation_indent);
            if(write_aborted_) {
               break;
            }
         }
         const auto count =
            std::min(remaining, line_limit - column_);
         emit(std::string_view(bytes.data() + word_start + emitted, count));
         column_ += count;
         emitted += count;
         remaining -= count;
      }
   }
}

bool DoorPresentation::write(const std::string_view text,
                             const DoorTextRole role) {
   write_aborted_ = false;
   const bool color = door_profile_uses_ansi(profile_);
   if(color) {
      emit(role_sequence(role));
   }
   const auto formatted = role == DoorTextRole::Number
                          ? format_number_text(text, display_formatting_)
                          : std::string(text);
   const auto encoded = encode_text(formatted, profile_);
   emit_encoded(encoded, role, 0);
   if(color) {
      emit("\x1b[0m");
   }
   flush();
   const bool completed = !write_aborted_;
   write_aborted_ = false;
   return completed;
}

bool DoorPresentation::write_hanging(
   const std::string_view text,
   const size_t continuation_indent,
   const DoorTextRole role)
{
   write_aborted_ = false;
   if(continuation_indent >= content_columns()) {
      throw std::invalid_argument(
         "continuation indent must leave room for text");
   }
   const bool color = door_profile_uses_ansi(profile_);
   if(color) {
      emit(role_sequence(role));
   }
   const auto formatted = role == DoorTextRole::Number
                          ? format_number_text(text, display_formatting_)
                          : std::string(text);
   const auto encoded = encode_text(formatted, profile_);
   emit_encoded(encoded, role, continuation_indent);
   if(color) {
      emit("\x1b[0m");
   }
   flush();
   const bool completed = !write_aborted_;
   write_aborted_ = false;
   return completed;
}

}  // namespace ct
