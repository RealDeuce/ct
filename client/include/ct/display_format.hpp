#pragma once

#include <cstdint>
#include <string>
#include <string_view>

namespace ct {

struct DisplayFormatting {
   std::string decimal_separator = ".";
   std::string grouping_separator;
   uint8_t primary_grouping_digits = 0;
   uint8_t secondary_grouping_digits = 0;
   std::string game_timestamp_pattern =
      "Day {day}, {hour}:{minute}:{second}";
   std::string game_duration_pattern =
      "{day} d {hour}:{minute}:{second}";
   std::string real_duration_pattern =
      "{hour}:{minute}:{second}";

   bool operator==(const DisplayFormatting&) const = default;
};

void validate_display_formatting(const DisplayFormatting& formatting);
std::string format_number_text(std::string_view text,
                               const DisplayFormatting& formatting);
std::string format_game_timestamp(uint64_t seconds,
                                  const DisplayFormatting& formatting);
std::string format_game_duration(uint64_t seconds,
                                 const DisplayFormatting& formatting);
std::string format_real_duration(uint64_t seconds,
                                 const DisplayFormatting& formatting);

}  // namespace ct
