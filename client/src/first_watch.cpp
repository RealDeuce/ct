#include "ct/first_watch.hpp"

#include <algorithm>
#include <array>
#include <limits>
#include <tuple>
#include <vector>

namespace ct {
namespace {

bool simple_task_kind(const TaskKind kind)
{
   return kind == TaskKind::Courier || kind == TaskKind::Freight ||
          kind == TaskKind::Passenger;
}

unsigned task_kind_rank(const TaskKind kind)
{
   if(kind == TaskKind::Courier) {
      return 0;
   }
   if(kind == TaskKind::Freight) {
      return 1;
   }
   return 2;
}

const TaskRouteAssessment* route_for(const TaskLedger& ledger,
                                     const uint64_t offer_id)
{
   const auto found = std::find_if(
      ledger.route_assessments.begin(), ledger.route_assessments.end(),
      [offer_id](const auto& route) { return route.offer_id == offer_id; });
   return found == ledger.route_assessments.end() ? nullptr : &*found;
}

bool directly_known(const KnownDestinations& destinations,
                    const uint64_t system_id)
{
   if(system_id == destinations.current_system_id) {
      return true;
   }
   const auto found = std::find_if(
      destinations.systems.begin(), destinations.systems.end(),
      [system_id](const auto& system) { return system.system_id == system_id; });
   return found != destinations.systems.end() && found->within_jump_rating;
}

uint64_t saturating_add(const uint64_t left, const uint64_t right)
{
   return right > std::numeric_limits<uint64_t>::max() - left
      ? std::numeric_limits<uint64_t>::max()
      : left + right;
}

struct Candidate {
   const TaskOffer* offer;
   bool direct;
   uint64_t slack;
   uint64_t exposure;
};

}  // namespace

FirstWatchCareer first_watch_career(const CombatCareerMode mode)
{
   switch(mode) {
   case CombatCareerMode::Independent:
      return FirstWatchCareer::Trader;
   case CombatCareerMode::Privateer:
      return FirstWatchCareer::Privateer;
   case CombatCareerMode::Navy:
      return FirstWatchCareer::Navy;
   case CombatCareerMode::Pirate:
      return FirstWatchCareer::Neutral;
   }
   return FirstWatchCareer::Neutral;
}

uint32_t first_watch_fact_bit(const FirstWatchFact fact)
{
   return uint32_t{1} << static_cast<uint8_t>(fact);
}

bool first_watch_fact_seen(const FirstWatchPreferenceState& state,
                           const FirstWatchFact fact)
{
   return (state.seen & first_watch_fact_bit(fact)) != 0;
}

void mark_first_watch_fact(FirstWatchPreferenceState& state,
                           const FirstWatchFact fact)
{
   state.seen |= first_watch_fact_bit(fact);
}

std::optional<FirstWatchFact> next_first_watch_fact(
   const FirstWatchPreferenceState& state,
   const FirstWatchCareer career)
{
   if(state.disposition != FirstWatchDisposition::Active) {
      return std::nullopt;
   }
   constexpr std::array sequence{
      FirstWatchFact::Welcome,
      FirstWatchFact::Crew,
      FirstWatchFact::Ship,
      FirstWatchFact::Finance,
      FirstWatchFact::Operations,
      FirstWatchFact::Messages,
      FirstWatchFact::Tasks,
      FirstWatchFact::Opportunity,
      FirstWatchFact::KnownUniverse,
      FirstWatchFact::Readiness,
      FirstWatchFact::Departure,
   };
   for(const auto fact : sequence) {
      if(fact == FirstWatchFact::Operations &&
         career != FirstWatchCareer::Privateer &&
         career != FirstWatchCareer::Navy) {
         continue;
      }
      if((fact == FirstWatchFact::Tasks ||
          fact == FirstWatchFact::Opportunity) &&
         career == FirstWatchCareer::Navy) {
         continue;
      }
      if(!first_watch_fact_seen(state, fact)) {
         return fact;
      }
   }
   return FirstWatchFact::Departure;
}

std::optional<uint64_t> recommend_first_watch_offer(
   const TaskLedger& ledger,
   const KnownDestinations& destinations)
{
   std::vector<Candidate> candidates;
   for(const auto& offer : ledger.local_offers) {
      const auto* route = route_for(ledger, offer.offer_id);
      if(offer.origin_system_id != destinations.current_system_id ||
         !offer.legal || !simple_task_kind(offer.kind) ||
         !offer.unavailable_reasons.empty() ||
         offer.collateral_credits > ledger.available_credits || route == nullptr ||
         !route->pickup_available || !route->delivery_available ||
         route->pickup_arrival_second > offer.expires_second ||
         route->delivery_arrival_second > offer.delivery_deadline_second) {
         continue;
      }
      const auto pickup_slack = offer.expires_second - route->pickup_arrival_second;
      const auto delivery_slack =
         offer.delivery_deadline_second - route->delivery_arrival_second;
      candidates.push_back(Candidate{
         .offer = &offer,
         .direct = directly_known(destinations, offer.destination_system_id),
         .slack = std::min(pickup_slack, delivery_slack),
         .exposure = saturating_add(
            offer.collateral_credits,
            saturating_add(offer.failure_penalty_credits,
                           offer.non_delivery_liability_credits)),
      });
   }
   if(candidates.empty()) {
      return std::nullopt;
   }
   std::sort(candidates.begin(), candidates.end(), [](const auto& left,
                                                       const auto& right) {
      return std::tuple(!left.direct,
                        task_kind_rank(left.offer->kind),
                        std::numeric_limits<uint64_t>::max() - left.slack,
                        left.exposure,
                        left.offer->offer_id) <
             std::tuple(!right.direct,
                        task_kind_rank(right.offer->kind),
                        std::numeric_limits<uint64_t>::max() - right.slack,
                        right.exposure,
                        right.offer->offer_id);
   });
   return candidates.front().offer->offer_id;
}

}  // namespace ct
