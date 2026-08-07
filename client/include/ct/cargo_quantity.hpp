#pragma once

#include <cstdint>
#include <optional>
#include <string>
#include <string_view>

namespace ct {

constexpr uint64_t millitons_per_ton = 1000;

// Parse a decimal tonne quantity without passing through floating point.
// At most three fractional digits are accepted.
std::optional<uint64_t> parse_tonnage_millitons(std::string_view text);

// Format an exact milliton quantity while suppressing insignificant zeroes.
std::string format_tonnage(uint64_t millitons);

// Purchase totals round upward to the credit; sale totals round downward.
uint64_t cargo_purchase_cost(uint64_t price_per_ton,
                             uint64_t quantity_millitons) noexcept;
uint64_t maximum_affordable_cargo(uint64_t credits,
                                  uint64_t price_per_ton,
                                  uint64_t upper_millitons) noexcept;

}  // namespace ct
