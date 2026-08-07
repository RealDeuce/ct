#include "ct/cargo_quantity.hpp"

#include <charconv>
#include <limits>

namespace ct {
namespace {

bool checked_add(uint64_t& target, const uint64_t value) noexcept {
   if(value > std::numeric_limits<uint64_t>::max() - target) {
      target = std::numeric_limits<uint64_t>::max();
      return false;
   }
   target += value;
   return true;
}

uint64_t partial_credit_cost(const uint64_t price_per_ton,
                             const uint64_t remainder_millitons) noexcept {
   const auto large = price_per_ton / millitons_per_ton;
   const auto small = price_per_ton % millitons_per_ton;
   if(remainder_millitons != 0 &&
      large > std::numeric_limits<uint64_t>::max() / remainder_millitons) {
      return std::numeric_limits<uint64_t>::max();
   }
   auto result = large * remainder_millitons;
   const auto small_product = small * remainder_millitons;
   const auto rounded = small_product / millitons_per_ton +
                        (small_product % millitons_per_ton != 0 ? 1 : 0);
   checked_add(result, rounded);
   return result;
}

}  // namespace

std::optional<uint64_t> parse_tonnage_millitons(const std::string_view text) {
   if(text.empty()) {
      return std::nullopt;
   }
   const auto point = text.find('.');
   if(point != std::string_view::npos && text.find('.', point + 1) != std::string_view::npos) {
      return std::nullopt;
   }
   const auto whole_text = point == std::string_view::npos ? text : text.substr(0, point);
   const auto fraction_text =
      point == std::string_view::npos ? std::string_view{} : text.substr(point + 1);
   if(whole_text.empty() && fraction_text.empty()) {
      return std::nullopt;
   }
   if(fraction_text.size() > 3) {
      return std::nullopt;
   }

   uint64_t whole = 0;
   if(!whole_text.empty()) {
      const auto [end, error] =
         std::from_chars(whole_text.data(), whole_text.data() + whole_text.size(), whole);
      if(error != std::errc() || end != whole_text.data() + whole_text.size()) {
         return std::nullopt;
      }
   }
   uint64_t fraction = 0;
   if(!fraction_text.empty()) {
      const auto [end, error] = std::from_chars(
         fraction_text.data(), fraction_text.data() + fraction_text.size(), fraction);
      if(error != std::errc() || end != fraction_text.data() + fraction_text.size()) {
         return std::nullopt;
      }
      for(size_t digits = fraction_text.size(); digits < 3; ++digits) {
         fraction *= 10;
      }
   }
   if(whole > (std::numeric_limits<uint64_t>::max() - fraction) /
                 millitons_per_ton) {
      return std::nullopt;
   }
   return whole * millitons_per_ton + fraction;
}

std::string format_tonnage(const uint64_t millitons) {
   auto result = std::to_string(millitons / millitons_per_ton);
   auto remainder = millitons % millitons_per_ton;
   if(remainder == 0) {
      return result;
   }
   result.push_back('.');
   result.push_back(static_cast<char>('0' + remainder / 100));
   result.push_back(static_cast<char>('0' + remainder / 10 % 10));
   result.push_back(static_cast<char>('0' + remainder % 10));
   while(result.back() == '0') {
      result.pop_back();
   }
   return result;
}

uint64_t cargo_purchase_cost(const uint64_t price_per_ton,
                             const uint64_t quantity_millitons) noexcept {
   const auto whole = quantity_millitons / millitons_per_ton;
   if(price_per_ton != 0 &&
      whole > std::numeric_limits<uint64_t>::max() / price_per_ton) {
      return std::numeric_limits<uint64_t>::max();
   }
   auto result = whole * price_per_ton;
   checked_add(
      result,
      partial_credit_cost(price_per_ton, quantity_millitons % millitons_per_ton));
   return result;
}

uint64_t maximum_affordable_cargo(const uint64_t credits,
                                  const uint64_t price_per_ton,
                                  const uint64_t upper_millitons) noexcept {
   if(price_per_ton == 0) {
      return upper_millitons;
   }
   uint64_t low = 0;
   uint64_t high = upper_millitons;
   while(low < high) {
      const auto middle = low + (high - low) / 2 + (high - low) % 2;
      if(cargo_purchase_cost(price_per_ton, middle) <= credits) {
         low = middle;
      } else {
         high = middle - 1;
      }
   }
   return low;
}

}  // namespace ct
