#include "ct/first_watch.hpp"

#include <stdexcept>

namespace {

void check(const bool condition)
{
   if(!condition) {
      throw std::runtime_error("First Watch test failed");
   }
}

ct::TaskOffer offer(const uint64_t id,
                    const ct::TaskKind kind,
                    const uint64_t destination,
                    const uint64_t collateral = 0)
{
   return ct::TaskOffer{
      .offer_id = id,
      .revision = 1,
      .kind = kind,
      .title = "Routine duty",
      .origin_system_id = 10,
      .destination_system_id = destination,
      .commodity_id = 0,
      .quantity_millitons = 0,
      .passenger_count = 0,
      .payment_credits = 1000,
      .collateral_credits = collateral,
      .expires_second = 200,
      .delivery_deadline_second = 500,
      .legal = true,
      .partial_delivery_allowed = false,
      .failure_penalty_credits = 10,
      .recurrence_seconds = 0,
      .performance_count = 1,
      .passenger_class = ct::PassengerClass::None,
      .late_deduction_per_day_credits = 0,
      .non_delivery_liability_credits = 20,
      .passenger_grace_seconds = 0,
      .declared_value_credits = 0,
      .unavailable_reasons = {},
   };
}

ct::TaskRouteAssessment route(const uint64_t id,
                              const uint64_t pickup,
                              const uint64_t delivery)
{
   return ct::TaskRouteAssessment{
      .offer_id = id,
      .pickup_available = true,
      .pickup_arrival_second = pickup,
      .delivery_available = true,
      .delivery_arrival_second = delivery,
   };
}

}  // namespace

int main()
{
   ct::FirstWatchPreferenceState state{
      .disposition = ct::FirstWatchDisposition::Active,
   };
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Trader) ==
         ct::FirstWatchFact::Welcome);
   for(const auto fact : {
         ct::FirstWatchFact::Welcome,
         ct::FirstWatchFact::Crew,
         ct::FirstWatchFact::Ship,
         ct::FirstWatchFact::Finance,
      }) {
      ct::mark_first_watch_fact(state, fact);
   }
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Trader) ==
         ct::FirstWatchFact::Messages);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Privateer) ==
         ct::FirstWatchFact::Operations);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Navy) ==
         ct::FirstWatchFact::Operations);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Neutral) ==
         ct::FirstWatchFact::Messages);
   ct::mark_first_watch_fact(state, ct::FirstWatchFact::Operations);
   ct::mark_first_watch_fact(state, ct::FirstWatchFact::Messages);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Trader) ==
         ct::FirstWatchFact::Tasks);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Privateer) ==
         ct::FirstWatchFact::Tasks);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Navy) ==
         ct::FirstWatchFact::KnownUniverse);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Neutral) ==
         ct::FirstWatchFact::Tasks);
   ct::mark_first_watch_fact(state, ct::FirstWatchFact::Tasks);
   ct::mark_first_watch_fact(state, ct::FirstWatchFact::Opportunity);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Trader) ==
         ct::FirstWatchFact::KnownUniverse);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Privateer) ==
         ct::FirstWatchFact::KnownUniverse);
   check(ct::next_first_watch_fact(state, ct::FirstWatchCareer::Navy) ==
         ct::FirstWatchFact::KnownUniverse);
   state.disposition = ct::FirstWatchDisposition::Hidden;
   check(!ct::next_first_watch_fact(state, ct::FirstWatchCareer::Trader));
   state.disposition = ct::FirstWatchDisposition::LocallyComplete;
   check(!ct::next_first_watch_fact(state, ct::FirstWatchCareer::Trader));
   state.disposition = ct::FirstWatchDisposition::Active;

   ct::TaskLedger ledger{};
   ledger.available_credits = 100;
   ledger.local_offers = {
      offer(9, ct::TaskKind::Freight, 30),
      offer(7, ct::TaskKind::Courier, 20),
      offer(5, ct::TaskKind::Courier, 20),
   };
   ledger.route_assessments = {
      route(9, 100, 300),
      route(7, 100, 300),
      route(5, 100, 300),
   };
   ct::KnownDestinations destinations{};
   destinations.current_system_id = 10;
   destinations.jump_rating = 1;
   ct::KnownSystemSummary direct{};
   direct.system_id = 20;
   direct.within_jump_rating = true;
   ct::KnownSystemSummary remote{};
   remote.system_id = 30;
   remote.within_jump_rating = false;
   destinations.systems = {direct, remote};
   check(ct::recommend_first_watch_offer(ledger, destinations) == 5);

   ledger.local_offers[2].unavailable_reasons.push_back("no capacity");
   check(ct::recommend_first_watch_offer(ledger, destinations) == 7);
   ledger.local_offers[1].legal = false;
   check(ct::recommend_first_watch_offer(ledger, destinations) == 9);
   ledger.local_offers[0].destination_system_id = 20;
   check(ct::recommend_first_watch_offer(ledger, destinations) == 9);
   ledger.local_offers[0].collateral_credits = 101;
   check(!ct::recommend_first_watch_offer(ledger, destinations).has_value());
}
