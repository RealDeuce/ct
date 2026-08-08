# System Traffic and Encounter Geography

*Status: high-level semantics and initial traffic calibration adopted; daily
traffic, deterministic landmark-scoped observer projections, sensor-qualified
identification, and the first checkpoint contact model are implemented, while
economic coefficients and trajectory thresholds remain subject to calibration,
2026-08-08*

Random traffic is not distributed uniformly through a system. It arises from
population, technology, trade, and the movement of ships between a small
number of useful destinations. A random contact becomes an encounter only
when the ships can actually converge.

Two related observations must remain distinct. **System traffic** is the
traffic-control picture assembled from active transponders and movement
reports across the system. **Local contacts** are vessels detected at the
observer's present locus and are the only ordinary candidates for immediate
interception. A civilian contact can contribute transponder identity to both
pictures, but its hull solution still comes from local sensors.

## Unaligned regional character

BBS polities have explicit trade-to-combat and chaos-to-order settings.
Unaligned systems have no equivalent owner-selected settings. Their effective
regional character is derived instead.

In general, the effective profile drifts toward combat and chaos with
increasing separation from a polity's influence. Surrounding inhabited
systems also affect one another. This neighbor influence must permit local
clusters with complementary trade, capable institutions, and mutually
supporting traffic to form natural islands of trade and order without being
assigned to a polity.

The two profile axes describe the composition and organization of existing
traffic. They do not determine its amount:

- trade-to-combat controls the balance among commercial, military, security,
  privateer, and predatory activity;
- chaos-to-order controls organization, reliability, effective authority, and
  the consequences attached to that activity; and
- CE Law Level remains a separate world characteristic governing restrictions
  and enforcement behavior.

The exact distance measure, influence strength, neighbor weighting, decay,
and feedback model remain open. The eventual calculation should operate on
the useful Jump and trade topology rather than assume that every nearby point
has equal access.

## Activity

Traffic activity is based primarily on:

1. population;
2. Tech Level; and
3. realized trade.

Population supplies people, demand, production, and institutions. Tech Level
determines how much of that base can generate and support space traffic.
Realized trade supplies arriving, departing, and through traffic. Distance
from a polity may change trade opportunities and the character of traffic,
but is not itself an independent activity source.

Activity is not one universal encounter number. The same inputs create
related rates for orbital and port traffic, world-to-Jump traffic, wilderness
refuelling, and individual interstellar routes. This permits:

- a high-population, high-TL isolated world to have substantial local traffic
  but few visiting merchants;
- a small high-TL junction to be busy because of through trade;
- a populous low-TL world to have much less locally generated space traffic;
- an uninhabited waypoint to have only its through traffic; and
- an uninhabited system away from established routes to have essentially no
  ordinary traffic.

Starports and facilities distribute, service, and constrain this activity;
they do not replace population, TL, and trade as its basic causes.

## Rules basis and modeling boundary

The source rules provide several useful but differently scoped measurements:

- core CE generates freight and passengers by starport and destination and
  refreshes availability every three days;
- core CE speculative trade generates supplier-sized lots and gives
  complementary trade codes, prices, and route effects;
- *Bounded Fortune* adds an origin/destination starport matrix. Its Class-A to
  Class-A mean is 350 tons per three days, or about 817 tons per week, and it
  explicitly treats independent freight as the residual left after large
  carriers;
- *Bounded Fortune* also derives cargo availability from population,
  influence, solar activity, starport, and law, but does not conserve
  world-wide production and consumption; and
- *Port of Entry* gives real facility capacities. For example, fuel refineries
  produce 10, 15, or 20 tons per facility ton per day at TL7, TL10, and TL12+
  respectively, while manufacturing produces 0.2 tons per facility ton per
  day at TL8 and 0.5 at TL10+. It also specifies storage, docking, and some
  staffing constraints, but not a general world-wide installed industrial
  capacity or cargo-handling rate.

Those rules are enough to calibrate available trade and explicit facilities,
but not enough to derive a conserved planetary economy directly from a UWP.
The initial model is therefore deliberately hybrid:

1. an aggregate planetary macroeconomy produces steady-state route flows;
2. explicit stations and facilities add known production, demand, storage,
   and disruption deltas; and
3. only cargo, passengers, mail, and ships made consequential by schedules,
   observations, or player actions become individual persistent records.

Do not instantiate every notional cargo shipment merely because it exists in
the statistical background economy.

## Initial steady-state cargo model

For each inhabited world, first recover its actual population:

```text
Nworld = population multiplier × 10^PopulationCode
```

Combine the inhabited worlds in one system into a technology-weighted
productive population:

```text
E = Σ(Nworld × 2^(TLworld - 8))
```

The system's initial potential interstellar cargo flow is:

```text
Q = 300 tons/week × sqrt(E / 1,000,000) × R
```

`R` is the realized-trade participation factor, initially interpreted as:

| `R` | Interpretation |
|---:|---|
| 0 | no realized interstellar trade |
| 0.10 | marginal or badly isolated trade |
| 0.50 | weak trade |
| 1.00 | normal established trade |
| 1.25–1.50 | unusually export-oriented trade |

The square root is intentional. Population, demand, and industry grow much
faster than the fraction exposed to independent interstellar shipping.
Technology contributes a factor of `2^((TL - 8) / 2)` after the square root.
The 300-ton coefficient and the `R` derivation are prototype calibration
values, not claims made by CE.

Trade codes divide `Q` into supply and demand commodity vectors.
Complementarity, route cost and time, security, known relationships, port
capacity, and competing destinations then allocate those vectors into
directed route flows. A polity capital or junction may also handle genuine
through traffic; do not add a fixed “capital bonus” when the route solver can
derive that traffic.

Explicit facility output is applied after the planetary baseline. It may add
an export, consume an import, impose storage or docking constraints, or
replace part of an abstract local industry. The same output must not appear
once in `Q` and again as a facility bonus.

### Converting route flow into ships

For a directed route carrying `F` tons per week, use this initial nominal
cargo-capacity estimate:

```text
C = clamp(100 × (F / 100)^(1/3), 50, 1,500) tons
mean carried cargo = 0.75 × C
departures per week = F / mean carried cargo
```

This is a fleet-mix approximation, not a requirement that every vessel have
capacity `C`. Busy routes support larger ships; sparse routes use small ships,
mixed loads, and irregular service. Passenger, mail, military, local, and
ballast traffic are separate additions where applicable.

The calibration gives the following order of magnitude for frontier systems
using a population multiplier of five:

| Population | TL | `R` | Cargo/week | Approximate cargo calls |
|---:|---:|---:|---:|---:|
| `10^1 × 5` | 6 | 0.10 | 0.1 t | one per 6.8 years |
| `10^2 × 5` | 7 | 0.10 | 0.5 t | one per 18 months |
| `10^3 × 5` | 8 | 0.15 | 3.2 t | one per 12 weeks |
| `10^4 × 5` | 8 | 0.25 | 16.8 t | one per 2.5 weeks |
| `10^5 × 5` | 9 | 0.40 | 120 t | 1.5/week |
| `10^6 × 5` | 10 | 0.50 | 671 t | 6/week across two routes |

These are baselines, not hard caps. A subsidized route, mine, refinery,
military base, mail relay, or useful through route can dominate a small local
population. At the busy end, a TL12 system with population `10^8 × 5`, normal
trade, five useful routes, and substantial derived transshipment is on the
order of one hundred cargo arrivals and departures per week rather than one
ever-present generic freighter.

The Class-A-to-Class-A spot-freight figure from *Bounded Fortune* should remain
a player-facing residual within this larger flow. It is not the total commerce
of two major worlds.

## Traffic locations

System traffic is organized around five location classes:

1. Jump arrival locus;
2. gas giants;
3. inhabited world;
4. Jump departure locus; and
5. everywhere else.

These activity levels are interrelated but not equal. A useful conceptual
flow is:

```text
Jump arrival
 ├─> inhabited world ─────────> Jump departure
 ├─> gas giant ─> world ─────> Jump departure
 ├─> gas giant ───────────────> Jump departure
 ├─> everywhere else
 └─> Jump departure
```

Local traffic also connects worlds, gas giants, moons, belts, stations, and
other facilities without necessarily entering or leaving the system.

Each actual gas giant and usable Jump locus is an individual instance of its
class. Current orbital geometry, travel time, fuel price and quality,
security, ship endurance, and destination determine how traffic divides among
them.

The encounter exposure at a location depends conceptually on:

```text
traffic passing through
× time spent exposed
× probability of contact
+ ships deliberately loitering there
```

Incoming and outgoing ship counts may approach one another over a long
period, but their local encounter rates need not. Stops, servicing, refuelling,
convoy assembly, course preparation, inspections, delays, and loitering create
different traffic stocks at different times.

### Jump arrival locus

Incoming route traffic passes through the arrival locus. Most arrivals move
on quickly, while pickets, rescue vessels, customs craft, intelligence ships,
and ambushers may remain nearby.

### Gas giants

Gas-giant activity comes from ships choosing to skim, local fuel operations,
patrols, and predators. Skimming and refining can make encounter exposure
large even when the number of visiting ships is modest.

### Inhabited world

The inhabited world combines interstellar traffic with locally generated
activity from population and TL. Its port, orbital facilities, moons, customs,
and local transport will usually make it the system's busiest region.

### Jump departure locus

Ships reach the departure locus after different local delays. Convoy
assembly, course preparation, inspections, and waiting can make departure
exposure different from arrival exposure even when the long-run flows are
similar.

The locus is always a simulation checkpoint for traffic, encounter, and final
readiness processing, but it is not always a player-interface stop. A valid
continuation plan may initiate its preauthorized Jump automatically when
nothing interrupts it. Docking approaches, skimming operations, and other
convergence points follow the same checkpoint rule; see
[`interplanetary-operations.md`](interplanetary-operations.md).

### Everywhere else

Everywhere else is not a fifth ordinary random-encounter pool. Space outside
the convergence points is overwhelmingly empty. Activity exists only along a
specific corridor or around a specific destination such as another planet, a
belt, station, base, cache, survey target, rendezvous, or current event.

## Contact, intercept, and encounter

### System Common radio

System Common is the one public player radio channel within a star system. A
transmission has a fixed emission position and time, and its spherical
wavefront expands at light speed. Reception is computed from the receiving
ship's physical trajectory rather than delivered system-wide at emission.
Jump space is outside the channel. There are no ambient AI conversations and
no separate player channels; NPC radio is limited to structured encounter
hails such as inspection orders, boarding instructions, and surrender demands.

Content is stored once per transmission. The active wave and each unread
receiving-ship row hold references to that content. The wave reference is
released after the sphere has passed every modeled location in the system;
the content then survives only while unread reception references remain.
Opening a reception removes its ship's reference, and unopened receptions
expire after 196 game days. This makes radio history transient without making
multiple stored copies of a broadcast. Reception rows belong to ships, not
captain identities.

Captains may mute ordinary broadcasts from another captain. Structured hails
bypass mutes, remain observable to other ships reached by the wave, and carry
an actionable encounter reference only for the target vessel. Public radio is
not encrypted; encrypted channels remain deferred until the game has real
public-key support.

Detection does not imply an actionable encounter:

```text
contact   -> sensors reveal another ship
intercept -> a ship can force or arrange converging trajectories
encounter -> the ships reach useful range with manageable relative velocity
```

Two arbitrary deep-space trajectories will normally have enormous separation
or unsuitable relative velocity. A sensor contact may therefore remain only
information. An encounter outside the normal convergence points requires a
specific cause:

- a prearranged rendezvous;
- a convoy or common departure;
- pursuit by a ship with enough performance advantage;
- interception of a known flight plan;
- a distress call or disabled vessel;
- a shared destination such as a cache, base, or survey site; or
- an exceptionally dense local corridor.

Pirates, customs vessels, and naval patrols consequently prefer inhabited
world approaches, refuelling points, and standardized Jump loci, where ships
must converge and are more likely to have compatible velocities. An obscure
route through open space is genuinely useful for evasion, at the cost of time
and fuel.

### Combat observation and reinforcement

Nearby traffic is not frozen out merely because an encounter has begun, but it
cannot observe or join instantaneously. Each combat emission or deliberate
distress signal reaches another ship after the current separation divided by
the speed of light. That ship receives only what its sensors and the signal's
provenance support, including an uncertain assessment of the aggressor.

A response then requires an actual intercept from the observer's position and
velocity. It joins an ongoing combat only if it arrives while the encounter is
still active; otherwise it reaches a pursuit, rescue, arrest, salvage, or
aftermath state. Player ships use online choices or persistent offline
intervention policies. Enforcement traffic generally responds within its
authority, while other computer-controlled ships are most likely to intervene
when they comfortably overmatch the participants and can clearly identify an
aggressor. Detailed policy and combat-frame semantics are in
[`combat-control-and-automation.md`](combat-control-and-automation.md).

## Simulation boundary

Aggregate flows can supply traffic and contact opportunities without
continuously instantiating every background ship. A particular vessel or
encounter becomes persistent when it is observed, affects authoritative
state, or must continue to exist for a later event. Deep-space encounters must
come from the contact-and-intercept conditions above, not from rolling on a
generic system-wide encounter table.

## Sparse traffic and persistent schedules

Fractional traffic is an actual schedule, not a fresh availability probability
rolled for each player. Otherwise a frontier world receiving one ship every
twelve weeks would implausibly have a new ship and a full cargo board whenever
a player arrived.

Traffic fidelity is selected per **directed route**, with hysteresis so a
small rate fluctuation does not continually change representation:

| Directed-route rate | Representation |
|---:|---|
| below 0.5 calls/week | persist every planned arrival and departure |
| 0.5–5 calls/week | persist the near-term schedule; aggregate the distant flow |
| above 5 calls/week | aggregate routine traffic; materialize relevant or disrupted calls |

The port's visible traffic is the union of all its directed routes. A port
with five routes at 0.4 calls per week has sparse route schedules but sees
roughly two total calls per week. Local conspicuousness, staffing, customs,
and encounter exposure use the combined port schedule.

A lightweight `TrafficCall` contains at least:

- origin and destination;
- planned arrival and departure times;
- vessel category and effective cargo capacity;
- scheduled, chartered, or opportunistic service;
- committed cargo, passenger, and mail capacity; and
- planned, arrived, delayed, departed, cancelled, lost, or diverted status.

It does not require a complete NPC ship. Materialize the full vessel, crew,
damage, and cargo only when observation, interception, disruption, or another
authoritative consequence requires them.

Low-volume cargo is likewise real accumulated inventory. A lightweight
`CargoLot` contains at least:

- commodity or generic cargo class;
- origin and intended destination;
- quantity and storage location;
- ready time and expiry or delivery deadline;
- shipper or responsible institution; and
- open, reserved, loaded, delivered, expired, consumed, or lost status.

Production adds inventory; player and background departures remove it. At
3.2 tons per week, for example, a world may have about 13 tons after four
weeks, 26 tons after eight, and 38 tons immediately before its expected
twelve-week call. If a player takes the lot, the background ship later leaves
partly empty, substitutes another load, changes its itinerary, or cancels. It
does not carry a duplicate statistical copy.

Inventory is aged by type:

- durable industrial freight accumulates until storage, demand, or transport
  removes it;
- food and other perishables spoil or are locally consumed;
- prospective passengers abandon or change plans;
- electronic mail accumulates into destination bags and clears through the
  standard departure-locus beacon onto an eligible ordinary carrier;
- contracts cease accepting performance at their expiry; and
- speculative stock can be exhausted until production replenishes it.

Below roughly 0.5 total calls per week, an arrival is also a significant local
event. Brokers, customs, prospective passengers, officials, criminals, and
other interested parties can prepare for a scheduled ship or react visibly to
an unexpected one. Piracy and enforcement in these systems depend more on
schedules and informants than on permanently orbiting random opponents.

## Daily system scheduling and player consequences

Every materialized star system has one authoritative logical update for every
game day. Systems do not need to tick simultaneously, and the update does not
instantiate every resident, cargo movement, or ship. A durable `SystemDay`
job statistically advances the system by one day and emits persistent objects
only when an outcome becomes consequential.

A daily job may:

- produce, consume, age, replenish, and expire inventory;
- create and expire local offers and contracts;
- update short-term market and traffic statistics;
- establish or update scheduled arrivals and departures;
- assemble destination mailbags and adjust beacon stipends;
- resolve aggregate piracy, customs, naval, commercial, and passenger
  activity;
- update facilities and other mutable local state; and
- emit a persistent incident, loss, damage record, warrant, news item,
  contract, cargo lot, or ship when the result must survive independently.

Pirate leads and commissions are derived from this authoritative state after
the traffic, target, criminal demand, or political conflict exists. They never
create a player-specific victim. Their role semantics are specified in
[`pirate-gameplay.md`](pirate-gameplay.md).

The scheduler stores only the next `SystemDay` job for each materialized
system. Completing it advances `last_processed_day` and schedules the
following day; it does not prepopulate the calendar with every future daily
job. An uninhabited system still receives a logical update but normally takes
a very cheap no-activity path. Analytically derived celestial and orbital
positions do not require mutation by the daily job.

Daily jobs, exact-time events, clock advances, and player commands all pass
through the same serialized authoritative input queue. A future-event index
only says when scheduled work becomes eligible for admission; it is not a
second processor and confers no category priority. A clock pulse is merely an
out-of-band scheduler wake-up and is never a queue entry. It moves eligible
work into the durable queue and commits that admission without executing it.
Each admitted item receives the next queue sequence and is later applied only
by the ordinary queue consumer. If no event remains due, the pulse may produce
an explicit logical-time-advance work item; that item, unlike the pulse, goes
through the queue.

Queue sequence is the complete execution order after admission. Neither event
kind, entity ID, nor timestamp may reorder two admitted inputs. When several
future events are eligible together, their global creation IDs provide the
stable admission order; this is bookkeeping determinism, not a semantic
production-before-departure or travel-before-maintenance rule. A rule which
requires one action to follow another must express that causally by scheduling
the dependent action only after its prerequisite commits.

Removing a scheduled item from its due index and inserting it into durable
ingress is one admission transaction. Applying it, journaling it, and removing
it from ingress is a later execution transaction. A crash before admission
leaves the future schedule; a crash after admission leaves recoverable queued
work; a crash during execution commits either the entire input or none of it.
Cross-system consequences are new scheduled inputs rather than direct mutation
of another system in the middle of a daily job.

Background mail remains governed by the persistent due-time delivery queue
specified in the message-store design. A daily system job assembles bags from
mail awaiting departures that exist for non-mail reasons; it does not scan
every message in flight or create traffic to satisfy a service target. A
committed carrier pickup schedules its exact arrival event, and delivery is
not postponed until a player next opens the market.

Ordinary traffic is the only electronic-mail capacity. A beacon advertises the
same token handling payment for every hop, signs custody when a configured ship
already bound for that destination downloads a mailbag, accepts it
automatically at arrival, and issues local payment. Mail data consumes
negligible cargo capacity. No ship departs, diverts, or chooses a route merely
to carry electronic mail, and there are no frequency guarantees, mail-route
subsidies, or dedicated electronic-mail vessels. Urgent physical objects or
passengers are cargo, passage, or contracts rather than this mail service.
Physical parcels remain cargo and do not use the electronic-data capacity.

“Free data capacity” does not make every message free to its sender. News
accepted by an agency, admitted public-service broadcasts, and public-key
distribution carry no sender charge. Private and other non-public-service mail
has a small TTL- and route-dependent charge. Fixed-system mail purchases one
exact path; mobile-address mail purchases a replicated hold sphere. Complete
tariff, retention, encryption, and delayed-revocation semantics are in
[`mail-service-and-security.md`](mail-service-and-security.md).

The daily system checkpoint is not the last player arrival. Merely entering,
passing through, viewing navigation data, or refuelling away from a market
does not consume cargo or move mail. Reading a market board may expose
already-created offers, but it does not reserve or consume them. Cargo changes
only through an authoritative player action or scheduled background event
such as production, reservation, loading, release, theft, destruction,
delivery, consumption, or expiry. Player-carried mail changes only when
accepted, handed off, lost, diverted, or delivered.

Persist player-created changes, structural changes, and facts already exposed
to a player alongside the daily checkpoints. Simulation draws must be
recoverable and independent of query count. Use a persisted RNG cursor or
secret, domain-separated streams keyed by system, domain, and logical day.
Never let repeated reads reroll a day, and never expose a generation or
simulation seed to a client.

Lazy or bulk reconciliation remains a recovery and catch-up optimization, not
the normal source of system history. It may execute several overdue
`SystemDay` jobs in a tight bounded loop, or use a proven aggregate shortcut
only when that produces the same durable consequences, RNG advancement, and
intermediate scheduled events as the logical daily sequence.

## Open decisions

- Choose the distance and route metric used for polity influence.
- Define spatial decay, neighbor reinforcement, and trade/order feedback.
- Calibrate the prototype cargo coefficient, realized-trade calculation,
  commodity vectors, and route allocator against generated test universes.
- Define passenger, mail, naval, local, and non-cargo traffic additions.
- Calibrate dwell times, loitering populations, and the implemented CE
  one-in-six-per-candidate checkpoint contact probability.
- Set traffic-fidelity hysteresis and the exact persistence schemas for
  `SystemDay`, `TrafficCall`, `CargoLot`, and observations. Arrival checkpoints
  and contact/encounter records have closed CT-RPC shapes in player protocol 3
  and storage format 1.
- Define daily-job work budgets, backlog behavior, and the safe boundary for
  bulk catch-up optimization. Event kind never supplies queue priority.
- Define storage limits, inventory aging, substitutions, cancellations, and
  background carrier behavior when a player changes a sparse route.
- Define sensor-contact and feasible-intercept thresholds from actual
  trajectories.
