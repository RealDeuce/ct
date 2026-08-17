#pragma once

#include "ct/player_identity_registry.hpp"
#include "ct/protocol.hpp"

#include <cstdint>
#include <optional>

namespace ct {

enum class FirstWatchCareer : uint8_t {
   Trader,
   Privateer,
   Navy,
   Neutral,
};

enum class FirstWatchFact : uint8_t {
   Welcome,
   Crew,
   Ship,
   Finance,
   Operations,
   Messages,
   Tasks,
   Opportunity,
   KnownUniverse,
   Readiness,
   Departure,
};

constexpr uint32_t FIRST_WATCH_KNOWN_FACTS_MASK =
   (uint32_t{1} << (static_cast<uint8_t>(FirstWatchFact::Departure) + 1)) - 1;

FirstWatchCareer first_watch_career(CombatCareerMode mode);
uint32_t first_watch_fact_bit(FirstWatchFact fact);
bool first_watch_fact_seen(const FirstWatchPreferenceState& state,
                           FirstWatchFact fact);
void mark_first_watch_fact(FirstWatchPreferenceState& state,
                           FirstWatchFact fact);
std::optional<FirstWatchFact> next_first_watch_fact(
   const FirstWatchPreferenceState& state,
   FirstWatchCareer career);

std::optional<uint64_t> recommend_first_watch_offer(
   const TaskLedger& ledger,
   const KnownDestinations& destinations);

}  // namespace ct
