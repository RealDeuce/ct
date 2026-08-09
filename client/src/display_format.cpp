#include "ct/display_format.hpp"

#include <array>
#include <cstdio>
#include <initializer_list>
#include <stdexcept>
#include <string>
#include <utility>

namespace ct {
namespace {

constexpr size_t MAX_SEPARATOR_BYTES = 8;
constexpr size_t MAX_PATTERN_BYTES = 128;

std::string grouped_integer(const std::string_view digits,
                            const DisplayFormatting& formatting)
{
   if(formatting.grouping_separator.empty() ||
      formatting.primary_grouping_digits == 0 ||
      formatting.secondary_grouping_digits == 0 ||
      digits.size() <= formatting.primary_grouping_digits) {
      return std::string(digits);
   }

   const size_t primary = formatting.primary_grouping_digits;
   const size_t secondary = formatting.secondary_grouping_digits;
   const size_t leading_size = digits.size() - primary;
   size_t first_size = leading_size % secondary;
   if(first_size == 0) {
      first_size = secondary;
   }

   std::string result;
   result.reserve(
      digits.size() +
      (digits.size() / secondary) * formatting.grouping_separator.size());
   result.append(digits.substr(0, first_size));
   size_t offset = first_size;
   while(offset < leading_size) {
      result.append(formatting.grouping_separator);
      result.append(digits.substr(offset, secondary));
      offset += secondary;
   }
   result.append(formatting.grouping_separator);
   result.append(digits.substr(leading_size, primary));
   return result;
}

std::string two_digits(const uint64_t value)
{
   std::array<char, 3> result{};
   std::snprintf(
      result.data(), result.size(), "%02llu",
      static_cast<unsigned long long>(value));
   return result.data();
}

std::string render_pattern(
   const std::string_view pattern,
   const std::initializer_list<std::pair<std::string_view, std::string_view>> values)
{
   std::string result;
   result.reserve(pattern.size() + 16);
   for(size_t offset = 0; offset < pattern.size();) {
      if(pattern[offset] == '{') {
         if(offset + 1 < pattern.size() && pattern[offset + 1] == '{') {
            result.push_back('{');
            offset += 2;
            continue;
         }
         const auto close = pattern.find('}', offset + 1);
         if(close == std::string_view::npos) {
            throw std::invalid_argument("display pattern has an unmatched '{'");
         }
         const auto name = pattern.substr(offset + 1, close - offset - 1);
         bool found = false;
         for(const auto& [candidate, value] : values) {
            if(name == candidate) {
               result.append(value);
               found = true;
               break;
            }
         }
         if(!found) {
            throw std::invalid_argument(
               "display pattern contains an unsupported field");
         }
         offset = close + 1;
         continue;
      }
      if(pattern[offset] == '}') {
         if(offset + 1 < pattern.size() && pattern[offset + 1] == '}') {
            result.push_back('}');
            offset += 2;
            continue;
         }
         throw std::invalid_argument("display pattern has an unmatched '}'");
      }
      result.push_back(pattern[offset]);
      ++offset;
   }
   return result;
}

void require_fields(
   const std::string_view pattern,
   const std::initializer_list<std::string_view> fields)
{
   for(const auto field : fields) {
      if(pattern.find(std::string("{") + std::string(field) + "}") ==
         std::string_view::npos) {
         throw std::invalid_argument(
            "display pattern omits a required field");
      }
   }
}

void validate_pattern(
   const std::string_view pattern,
   const std::initializer_list<std::string_view> required_fields)
{
   if(pattern.empty() || pattern.size() > MAX_PATTERN_BYTES) {
      throw std::invalid_argument(
         "display patterns must contain 1..128 UTF-8 bytes");
   }
   require_fields(pattern, required_fields);
}

}  // namespace

void validate_display_formatting(const DisplayFormatting& formatting)
{
   if(formatting.decimal_separator.empty() ||
      formatting.decimal_separator.size() > MAX_SEPARATOR_BYTES ||
      formatting.grouping_separator.size() > MAX_SEPARATOR_BYTES ||
      formatting.decimal_separator == formatting.grouping_separator) {
      throw std::invalid_argument("invalid display number separators");
   }
   if(formatting.grouping_separator.empty()) {
      if(formatting.primary_grouping_digits != 0 ||
         formatting.secondary_grouping_digits != 0) {
         throw std::invalid_argument(
            "display grouping sizes require a grouping separator");
      }
   } else if(formatting.primary_grouping_digits == 0 ||
             formatting.primary_grouping_digits > 9 ||
             formatting.secondary_grouping_digits == 0 ||
             formatting.secondary_grouping_digits > 9) {
      throw std::invalid_argument("invalid display number grouping sizes");
   }

   validate_pattern(
      formatting.game_timestamp_pattern,
      {"day", "hour", "minute", "second"});
   validate_pattern(
      formatting.game_duration_pattern,
      {"day", "hour", "minute", "second"});
   validate_pattern(
      formatting.real_duration_pattern,
      {"hour", "minute", "second"});

   const std::string day = "1";
   const std::string hour = "02";
   const std::string minute = "03";
   const std::string second = "04";
   (void)render_pattern(
      formatting.game_timestamp_pattern,
      {{"day", day}, {"hour", hour}, {"minute", minute}, {"second", second}});
   (void)render_pattern(
      formatting.game_duration_pattern,
      {{"day", day}, {"hour", hour}, {"minute", minute}, {"second", second}});
   (void)render_pattern(
      formatting.real_duration_pattern,
      {{"hour", hour}, {"minute", minute}, {"second", second}});
}

std::string format_number_text(const std::string_view text,
                               const DisplayFormatting& formatting)
{
   std::string result;
   result.reserve(text.size() + text.size() / 3);
   for(size_t offset = 0; offset < text.size();) {
      const auto byte = static_cast<unsigned char>(text[offset]);
      if(byte < '0' || byte > '9') {
         result.push_back(text[offset]);
         ++offset;
         continue;
      }

      const auto integer_start = offset;
      while(offset < text.size() && text[offset] >= '0' && text[offset] <= '9') {
         ++offset;
      }
      result.append(grouped_integer(
         text.substr(integer_start, offset - integer_start), formatting));
      if(offset < text.size() && text[offset] == '.' &&
         offset + 1 < text.size() &&
         text[offset + 1] >= '0' && text[offset + 1] <= '9') {
         result.append(formatting.decimal_separator);
         ++offset;
         while(offset < text.size() &&
               text[offset] >= '0' && text[offset] <= '9') {
            result.push_back(text[offset]);
            ++offset;
         }
      }
   }
   return result;
}

std::string format_game_timestamp(const uint64_t seconds,
                                  const DisplayFormatting& formatting)
{
   const auto day = std::to_string(seconds / 86400);
   const auto hour = two_digits(seconds / 3600 % 24);
   const auto minute = two_digits(seconds / 60 % 60);
   const auto second = two_digits(seconds % 60);
   return render_pattern(
      formatting.game_timestamp_pattern,
      {{"day", day}, {"hour", hour}, {"minute", minute}, {"second", second}});
}

std::string format_game_duration(const uint64_t seconds,
                                 const DisplayFormatting& formatting)
{
   const auto day = std::to_string(seconds / 86400);
   const auto hour = two_digits(seconds / 3600 % 24);
   const auto minute = two_digits(seconds / 60 % 60);
   const auto second = two_digits(seconds % 60);
   return render_pattern(
      formatting.game_duration_pattern,
      {{"day", day}, {"hour", hour}, {"minute", minute}, {"second", second}});
}

std::string format_real_duration(const uint64_t seconds,
                                 const DisplayFormatting& formatting)
{
   const auto hour = std::to_string(seconds / 3600);
   const auto padded_hour = hour.size() < 2 ? "0" + hour : hour;
   const auto minute = two_digits(seconds / 60 % 60);
   const auto second = two_digits(seconds % 60);
   return render_pattern(
      formatting.real_duration_pattern,
      {{"hour", padded_hour}, {"minute", minute}, {"second", second}});
}

}  // namespace ct
