# Cepheus Trader Development Roadmap

*Last reconciled: 2026-08-03*

## Purpose and Authority

This is the authoritative ordering and status document for Cepheus Trader
development. Read this file before beginning a new implementation slice and
return to the current milestone after an unrelated fix, investigation, or
tooling task.

Detailed documents under [`docs/`](docs/) own the mechanics and persistence
contracts for their domains. They do not change implementation priority merely
by calling something “next.”

Status terms are:

- **Complete:** its acceptance boundary is implemented and tested.
- **Current:** the next gameplay-critical slice. There must be only one.
- **Next:** ordered work after the current milestone.
- **Later:** necessary work whose prerequisites are not complete.
- **Reopened:** previously claimed complete, but an end-to-end playability
  audit found missing acceptance behavior.
- **Deferred:** deliberately outside the current playable-game path.

## Resume Here

**Current work: complete Milestone 7 field alpha and operations.**

The live server now advances at the fixed baseline of four game weeks per real
day while the server process is running. A standard one-week Jump therefore
takes six real hours. Server downtime is frozen game time: startup anchors the
monotonic process clock at the last committed game second and never infers
elapsed time from a saved wall timestamp.

The 2026-08-02 end-to-end audit reopened player-facing completion claims that
were not mechanically true. Milestones 2 through 5 have since passed their
redux boundaries. The merchant closure includes finite reservable leads,
dated intelligence, causal market events, atomic ordinary carriage, physical
remote-claim races, typed task termination/dispute actions, addressed private
mail, versioned starting finance/refits, and a real 40-column TLS door
playthrough. Milestone 6 now has the same real-door acceptance for entering a
combat career, selecting generated traffic, starting combat, assigning named
crew, and sealing a joint order; deterministic authoritative fixtures cover
the longer naval, prize-court, pirate-economy, legal-mail, and succession
outcomes. The reopened cross-cutting boundary is now closed: people have
physical condition, service, pay, morale, treatment, and shore location;
Tasks retain ship-bound cargo/passenger custody; messages carry typed action
references; and persistent facilities control every advertised dock service.

### Immediate Work Queue

1. Validate the Milestone 7 Field Alpha and Operations boundary in deployed
   BBS sessions.
2. Preserve the completed gameplay acceptance suites while adding real BBS
   startup, moderation, operations, and deployment behavior.

## Milestone Sequence

| Milestone | Status | Playable outcome |
| --- | --- | --- |
| 0. Authoritative foundation | Complete | A sysop can bootstrap a BBS, create a player, trade, fuel, and submit a persistent voyage. |
| 1. Live simulation clock and pacing | Complete | The voyage progresses fairly in a running game without manual time advancement. |
| 2. Player-carried mail and arrival packet | Complete | Travel physically moves information, and arrival presents actionable information that reached the system. |
| 3. Operating costs, maintenance, and fueling | Complete | Ship operation has meaningful time, cost, wear, repair, stores, and refueling choices. |
| 4. Travel contacts and encounter checkpoints | Complete | A routine voyage can be interrupted by traffic, authorities, danger, or opportunity. |
| 5. Complete merchant loop and task economy | Complete | Freight, passengers, richer trade, and concurrent obligations support sustained merchant play. |
| 6. Combat, navy, privateer, and pirate loops | Complete | The three combat-side careers and their shared personnel, task, port, and communications surfaces are mechanically playable. |
| 7. Multi-BBS gameplay alpha | Current | Real BBS sessions can sustain the intended small shared universe. |
| 8. Balance, scale, packaging, and release | Later | The game is economical to operate, progression is calibrated, and releases are reproducible. |

## Milestone 0 — Authoritative Foundation

**Status: Complete.**

The completed foundation includes:

- separate Rust server, C++/OpenDoors client, shared Cap'n Proto schemas, and
  TLS 1.3 external-PSK transport;
- one ordered, journaled authoritative engine with command epochs,
  idempotency, recovery, and independent connection transmit/receive work;
- protected administrator, BBS, and sysop credentials, including installation
  bootstrap and revision-checked sysop configuration;
- destructive initial-universe initialization, the 35-system Federation,
  seeded celestial-system derivation, BBS polity placement, and the observed-
  volume oracle;
- normalized shipbuilding rules, the active OGC ship catalog, design families,
  upgrade paths, starting-offer matrix, licensing, and provenance records;
- the complete new-player captain, ship, and crew creation transaction;
- all three scrolling terminal profiles at 40×24 and 80×24;
- universal Crew and read-only Ship managers, plus the initial Known Universe
  repository;
- durable systems, daily jobs, ordinary traffic ships, messages, envelopes,
  beacon queues, mailbags, custody legs, delivery, and expiry;
- instrumentation of the ten-BBS settlement envelope and its CPU/storage
  limits; and
- a persistent Common Goods market, cargo lots, account balance, refined fuel,
  concrete location, and one docked-to-docked Jump continuation.

This milestone does not imply that the background simulator is production-
scalable. Live progression is the Milestone 1 boundary below.

## Milestone 1 — Live Simulation Clock and Pacing

**Status: Complete.**

The live clock preserves the transaction and recovery model already in use.
Monotonic process time is only an input to authoritative advancement, never an
alternative source of game-state writes.

Required implementation:

- a versioned fixed-rate contract of 28 game seconds per real second;
- startup re-anchoring at committed logical time, deliberately freezing server
  downtime rather than persisting a catch-up wall-time anchor;
- deterministic admission of eligible time-advance work into the same ordered
  input queue used by player ingress, with no event-category priority;
- no multiplication of universe speed merely because several players or BBSs
  are active;
- one global baseline clock; future isolated encounter frames may hide and
  reconcile outside events but may not independently advance that baseline;
- phase/status results that let reconnecting clients learn every committed
  transition without reconstructing it locally; and
- a fake-clock test surface with no dependence on real sleeps.

The implementation uses one coalescing pulse source and bounded scheduler
slices on the authoritative engine thread. If work falls behind, commands run
at the last committed minute; the server reports lag and catches up without
allowing a command to observe an uncommitted target time. Phase changes are
reliable unsolicited protocol events. Connected observers in Docked or
Interplanetary phases also receive seed-derived traffic snapshots and
arrival/departure notices spread through each game day; those presentation
events do not create a second stored traffic schedule.

Acceptance:

- buy cargo and fuel, depart, disconnect, and later reconnect at the correct
  phase or destination without an explicit administrative time command;
- restart at every scheduled transition and obtain the same final state and
  event order;
- demonstrate that no input overtakes an earlier queue entry and that future
  work is ordered only when a clock pulse admits it;
- demonstrate the adopted inactive-universe behavior; and
- demonstrate that concurrent sessions do not advance the clock more than
  once.

## Milestone 2 — Player-Carried Mail and Arrival Packet

**Status: Complete.**

Background ships already carry real mailbags. This milestone gives player
ships the same physical custody and makes delayed information part of normal
play.

Required implementation:

- route-eligible beacon pickup at the departure locus;
- a sealed player-carried mailbag and exactly-once custody leg;
- automatic arrival handoff, onward routing, expiry, and local stipend;
- an arrival packet containing only news, offers, dangers, and structured
  knowledge that have actually reached the destination;
- persistent per-player message classification and unseen state;
- authoritative Message Management and mail-synchronized Known Universe
  updates; and
- the initial public/direct/withheld system-mapping notification path.

Acceptance:

- the same player voyage moves both cargo and a specific mailbag;
- restart cannot duplicate custody, delivery, stipend, or visibility;
- old information can arrive after newer local events without being silently
  rewritten as current; and
- the door can ignore, mark, inspect, and act on arrival items using all three
  output profiles.

This milestone completes the first **playable merchant prototype**: create a
captain, buy cargo, verify fuel and readiness, depart, wait through
authoritative server progression, carry information, arrive, read the packet,
inspect the destination's available fuel service, and sell the cargo. Starting
ships have full tanks; complete port/frontier refueling alternatives remain
Milestone 3.

Implemented details include sealed player custody on the same persisted ship
record as cargo, atomic handoff/payment, arrival receipts, chronological stale
information, durable per-captain classification, mail-synchronized Known
Universe provenance, all-profile door triage, and public/direct/withheld/
secret mapping choices. Public mapping notices and sealed Earth filings enter
the ordinary physical envelope queues; committed dispatches cannot be
retracted. The current route-invariant carrier stipend is explicitly
provisional and versioned for later balance work; it never causes or redirects
a voyage. Acceptance is exercised by the real TLS/OpenDoors harness: the same
persisted voyage carries speculative cargo and physical mail state across
restart, presents actionable arrival copy in every output profile, retains
classifications, and completes destination sale. The deterministic store
fixture supplies the exact non-empty-mailbag and stale-information cases that
a live randomized departure beacon cannot be required to contain.

## Milestone 3 — Operating Costs, Maintenance, and Fueling

**Status: Complete.**

This milestone makes ship condition and service choices mechanically real
before combat begins producing large amounts of damage.

Required implementation:

- proper per-subsystem repair work orders distinct from temporary battlefield
  repairs;
- continuing routine upkeep, accumulated calendar/use wear, facility
  capability, elapsed work time, material use, and payment;
- weeks-long refit/overhaul and component-level refurbishment/replacement,
  distinct from routine maintenance;
- berth fees, life support, ammunition, repair stores, and routine supplies;
- unrefined port fuel, wilderness water/ice collection, and gas-giant
  skimming;
- source ownership, claims, permits, institutional authority, and hostile
  extraction consequences for water/ice collection—never access inferred
  from hydrographics alone;
- skimming time, equipment requirements, wear, and failure exposure;
- coarse calendar-week training independent of watch assignment, plus natural
  healing under the separate full-rest/activity rules; and
- visible operating-cost and readiness summaries before departure.

Acceptance:

- a captain can compare refined purchase, wilderness collection, and skimming
  as genuine convenience/cost/time/maintenance alternatives;
- damage, temporary repair, permanent repair, routine upkeep, accumulated
  wear, refit, and replacement remain independent after recovery; and
- a work order cannot complete twice or at a facility incapable of doing it.

Implemented details include persistent catalog-capacity ammunition consumed
by combat; installed life-support provisions consumed by the daily system
transaction; berth and routine-upkeep accounting; independent damage,
battlefield patches, age/use wear, refit, and component replacement state;
source-backed latent quirks that manifest through use; warranty service; and
one revision-checked dock-service quotation/commit contract. Component prices
and installation times come from the normalized construction catalog and the
admitted Clement yard rules. Ordinary repair items remain included in the
source-defined monthly upkeep allowance; no unsupported per-hit starship
material price was invented.

The door lists refined and unrefined port fuel plus every lawful, named gas-
giant and unoccupied water/ice source, with exact availability and maximum-
fill time. Hostile extraction from an inhabited or claimed water source still
requires the contact, law, and combat machinery of Milestones 4 and 6.
Physical-store transfer between separately owned ships follows multi-ship
ownership rather than blocking this one-commanded-ship milestone.

## Milestone 4 — Travel Contacts and Encounter Checkpoints

**Status: Complete (redux implementation, 2026-08-02).**

The continuation plan already contains scheduled checkpoints. This milestone
puts real contact and interruption decisions at those points without requiring
the full combat engine.

Required implementation:

- a phase-level **Flight Plan** interface, invoked by `Depart` and available
  throughout travel, which owns the executable route separately from Tasks;
- initial destination commitment before ordinary cargo, passengers, and mail
  are accepted, followed by en-route editing of every route step the engine
  has not already processed;
- preservation and explicit warning of carriage obligations when replanning
  changes or delays their destination, rather than silently rewriting Tasks;
- transfer of candidate routes from Known Universe and task-oriented shortcuts
  into Flight Plan without making either subsystem own the active route;
- explicit hold/continue/through-point authority and standing offline behavior
  within the plan, rather than treating a displayed course as authorization;
- traffic generation and sensor contacts at ports, inhabited worlds, gas
  giants, Jump arrival loci, and Jump departure loci;
- relative-position and intercept feasibility rather than encounters sampled
  uniformly from empty space;
- traffic control, customs, inspections, pickets, distress calls, sightings,
  and avoidance or cooperation choices;
- continuation-plan suspension, replacement, resumption, and automatic
  progress when a checkpoint is uneventful;
- causal light-speed observation and delayed intervention by nearby ships; and
- an initial non-combat encounter resolution record suitable for later combat.

Acceptance:

- `Depart` can commit a useful first destination and begin the outbound
  maneuver without requiring the captain to finish a multi-system itinerary;
- while outbound or otherwise travelling, the captain can atomically change
  all unprocessed routing, including the next destination, while accepted
  cargo, passenger, mail, and contract obligations remain intact and their
  diversion consequences are shown;
- an uneventful through plan still reaches its destination without redundant
  confirmations;
- offline encounters occur only at explicitly authorized through-points, and
  the terminal arrival waits until the player is connected;
- a consequential checkpoint suspends at the exact committed location and can
  safely resume after reconnect or restart; and
- “everywhere else” remains sparse unless a real intercept is possible.

Implemented details include the current CT-RPC preview/commit snapshots, durable
revisioned plans independent of active physical legs, atomic outbound
replanning, explicit hold/terminal/through authority, warning-preserved cargo
and sealed-mail custody, and terminal-arrival acknowledgement. Arrival checks
draw from the deterministic ±60-minute local traffic projection and apply the
CE one-in-six candidate check as `1 - (5/6)^N`. Checkpoints, contacts, posture,
fallbacks, damage, capture/destruction outcomes, and terminal command loss are
current storage-format-10 records.

Body waypoints are executable rather than descriptive. A committed
gas-giant or wilderness-water step validates and retains its exact source,
runs the explicit outbound/work/return legs, and resumes the next authorized
Jump after the completed activity commits. A vanished or invalid service
condition holds the continuation instead of substituting a different body.

Consequential contacts suspend the route. Connected terminal arrivals wait
for acknowledgement; authorized through-points acknowledge under standing
orders and continue automatically when possible. Hostile contacts resolve as
headless CE-style opposed crew actions in separately admitted one-kilosecond
engine inputs, so no complete fight is hidden inside one transaction. The
door exposes plan preview, time/fuel/warnings, en-route revision, arrival
watch, encounter posture, queued-resolution, and terminal screens in the same
scrolling 40-column-compatible presentation used elsewhere.

The redux closes the remaining playable-navigation gaps. The door now edits
the durable plan rather than launching a direct-Jump shortcut; imports
fastest/cheapest Known Universe courses and task destinations; inserts named
gas-giant and wilderness-water operations; and supports coordinate exploration.
Coordinate breakout resolves the canonical six-parsec coverage boundary with
fresh OS entropy, commits generated systems and simulation work atomically,
and puts resolved contacts into the captain's carried charts. A deep-space
hold can then plot a subsequent Jump to a surveyed candidate.

Each Jump step records onboard plotting or a fresh commercial course tape and
an explicit replot/proceed-on-known-bad-plot instruction. The authoritative
transaction applies the CE Astrogation and Engineer (Jump) tasks, their stated
times, the engineer Effect, actual leg fuel, unrefined-fuel and drive-hit DMs,
the `148+6D6`-hour duration, inaccurate breakout, misjump, and transition
critical hit. Standard tapes cost Cr1,000 per Jump number and are available
only from Class-D-or-better ports to populated worlds.

Mapping claims now travel as physical mail. Private filings use the settled
Cr1 per started KiB per hop per TTL-week tariff; the first valid settled-system
filing is adjudicated only when it reaches Earth, where the Cr218,000 award is
posted and the Federation public notice originates. Secret Systems entries
can be changed from the navigation dossier before a notice is dispatched.
Terminal contacts of every typed encounter class suspend for the captain;
only a waypoint explicitly filed `through` authorizes standing-order
resolution while unattended.

## Milestone 5 — Complete Merchant Loop and Task Economy

**Status: Complete (redux implementation, 2026-08-03).**

Required implementation:

- CE Trade Goods beyond the six Common Goods;
- supplier search, reservations, delivery, background consumption, legality,
  customs, brokers, commissions, and market events;
- ordinary freight, bulk passengers, and ordinary mail capacity declarations;
- charters, couriers, purchases, sales, bounties, and other accepted offers as
  authoritative Tasks;
- contract deadlines, non-performance consequences, and concurrent contracts
  in different phases;
- market history with observation dates and source confidence in Known
  Universe;
- title, debt, reserves, compensation, institutional authority, and refit
  terms in versioned starting offers; and
- exactly one ordinary `apple pie (made from scratch)` cargo object in the
  universe, moved by normal traffic and available at its current port without
  exceptional price, signaling, or protection. Its destruction invokes the
  authoritative full-universe reset while preserving BBS control state; that
  causal rule is never disclosed through in-game text or behavior.

Acceptance is a sustainable trader loop in which route knowledge helps but
does not collapse play into repeatedly selecting one statistically optimal
route. The ship computer may organize known facts and calculate possibilities;
it must not choose commitments or conceal uncertainty from the player.

The implemented boundary is detailed in
[`docs/merchant-economy-and-tasks.md`](docs/merchant-economy-and-tasks.md).
The door exposes Tasks and standing carriage, the full cargo exchange,
scheduled supplier/buyer research, dated Known Universe market reports,
banking, finite ship brokerage, and finite crew hiring. Default enforcement is
physically propagated private mail. Combat proof and combat-side settlement
are supplied by the existing Milestone 6 machinery.

The redux boundary adds finite reservable market leads with forfeitable
escrow; exact named market-event effects and causal news; previewed atomic
loading of ordinary freight, passage, and mail; remote claims whose sealed
filings and replies physically race through the mail graph; explicit cancel,
withdraw, return-custody, default, and sealed-dispute transactions; capped
non-delivery liabilities; priced fixed-system and mobile-recipient private
mail; destination assistance; and 27 nonzero, role-correct versioned starting
term/refit packages. Persistence tests cover restart, expiry, reservation,
settlement, default, and message custody. The real 40-column C++ door/TLS
playthrough covers banking, addressed correspondence, speculative cargo, a
filed voyage, mail custody, arrival, and store reopen.

## Milestone 6 — Combat and the Three Combat-Side Careers

**Status: Complete (playability completion closed, 2026-08-03).**

Required implementation:

- CE one-kilosecond vessel activations, joint crew orders, reactions, damage,
  withdrawal, surrender, boarding, and escape craft;
- conservative online defaults and persistent risk-directed offline control;
- post-combat repair priorities and causal third-party intervention;
- naval orders, reporting, authority, rank, logistics, and issued ships;
- privateer commissions, prizes, adjudication, and lawful commerce raiding;
- pirate leads, commissions, cruises, fencing, crew pressure, and free
  predation; and
- crime, warrants, corruption, bounty propagation, banking response, and
  cross-polity enforcement.

Acceptance requires economically meaningful navy, privateer, and pirate loops,
not merely a combat demonstration. Capturing a ship must be worth pursuing
without making an intact debt-free prize an uncontrolled progression shortcut.

Implemented details include the normalized combat component catalog; durable
multi-vessel combat state; simultaneous-view joint orders; initiative,
missiles, point defense, damage, withdrawal, surrender, boarding, and escape
actions; conservative defaults; a deterministic fixed-budget risk controller;
real-traffic delayed intervention; and ordered recovery watches that retain
the battlefield/proper-repair distinction. Naval pay, six-month boards,
rank-limited issued hulls, and institutional logistics are separate from
privateer commissions and physically adjudicated 10/20/30-percent prize
claims. Pirates receive real-traffic leads, optional commissions, cruise
articles, 10–30-percent fences, crew pressure, and unrestricted predation.
Unlawful attacks create physical-mail warrants; foreign polities apply a
stronger recognition threshold, local law/corruption changes enforcement, and
the door exposes settlement, prizes, orders, traffic, and cruise controls.

The real TLS/OpenDoors acceptance scenario changes career status, selects a
generated local traffic contact, confirms an irreversible intercept, renders
the named general-quarters roster, and seals an authoritative joint order. It
then verifies the durable career, physical warrant, combat actors, and stored
order through the server. Deterministic store fixtures carry the longer loops
through mailed naval reporting and promotion, physical privateer adjudication,
pirate fencing and shares, delayed warrant resolution, command loss, and
bankruptcy succession.

The reopened playability boundary additionally uses one shared CE task
resolver for navigation, combat, market work, dispute handling, treatment,
and recovery. Duty, injury, fatigue, and morale affect qualified actors.
Personnel transactions cover transfer, discharge, leave/recall, first aid,
inpatient care, scheduled surgery, proportional payroll arrears, and
desertion. Merchant obligations name one performing ship and retain physical
freight/passenger custody across restart; partial and recurring settlement is
exactly once, and title transfer cannot move entrusted custody. Typed message
references open the corresponding offer, Task, Finance, Mapping, or Operations
surface. Persistent facility records govern fuel, chandlery, ordnance, yards,
medical care, personnel, banking, and authority availability in quotation and
commit.

Acceptance evidence includes
`store::tests::entrusted_cargo_and_passengers_remain_ship_bound_across_restart_and_settle_once`,
`store::tests::sourced_contracts_consume_only_delivered_goods_and_pay_proportionally`,
`store::tests::shore_leave_remains_at_its_real_berth_until_the_ship_returns`,
`store::tests::surgery_changes_condition_only_when_its_queued_hours_complete`,
`store::tests::unpaid_payroll_creates_personal_arrears_and_reduces_morale`, and
`store::tests::persisted_facility_capabilities_govern_both_quotes_and_commits`,
plus the real `tests/tls_interop.rs` three-profile door scenario.

The optional Web Push companion is a Milestone 8 packaging convenience. It is
not a prerequisite for authoritative combat or any timing guarantee.

## Milestone 7 — Field Alpha and Operations

**Status: Current.**

Required implementation:

- real OpenDoors/BBS drop-file startup and local player-ID attestation;
- reconnect and presentation behavior in actual remote terminal sessions;
- audited sysop moderation for originating players and polity state without
  advantage-granting operations;
- BBS founding announcements and inter-polity discovery propagation;
- operational monitoring, live same-version backup, recovery, and safe
  shutdown procedures; incompatible alpha stores are reinitialized rather
  than migrated until the first deployed persistence contract;
- load testing around the target of ten BBSs and fifty active players; and
- a deployment profile suitable for an approximately USD 50/month mainstream
  VPS when the target workload permits it.

## Milestone 8 — Balance, Scale, Packaging, and Release

**Status: Later.**

Required implementation:

- trader wealth and ship-upgrade pacing through the fully fitted 5,000-ton
  dreadnought target, with navy and outlaw paths calibrated to comparable
  achievement rather than identical credits;
- session pacing around a successful 15–30 minute daily session and a soft
  45-minute useful-action ceiling;
- scalable execution/storage for the large settlement envelope while
  preserving daily logical jobs, intermediate events, queue order, and fair
  recovery;
- long-duration economy, message-volume, storage-growth, and CPU validation;
- dependency, OGL source, catalog attribution, and generated Section 15 audits;
- reproducible builds and packages for the server, administrator utility,
  sysop utility, and OpenDoors door; and
- the optional HTTPS/Web Push activation notifier; and
- upgrade, migration, backup, and release documentation.

The measured 238,812-system ten-BBS fixture is an instrumentation baseline,
not a scalability success: the current one-transaction-per-event form projects
roughly 752 million events and 34.5 wall-days for one simulated year. Later
optimization may change representation and execution strategy, but not the
adopted logical outcomes or recovery fairness without an explicit design
decision.

## Supporting Work That Does Not Reorder Milestones

The following work may be done when it directly supports the current slice,
but completing it does not by itself advance the roadmap:

- schema additions driven by a client/server feature;
- catalog corrections and newly corroborated construction rules;
- licensing and OGC attribution maintenance;
- test infrastructure, diagnostics, metrics, and deterministic fixtures;
- standalone-repository, dependency-vendoring, CI, and portable-package
  foundations required to deploy the current field alpha, without claiming
  Milestone 8 balance or scale completion;
- documentation corrections and terminology cleanup;
- security hardening and credential usability; and
- performance improvements that preserve the existing logical contract.

When side work ends, resume the **Resume Here** milestone rather than selecting
the most recently edited subsystem document as the new priority.

## Deliberately Deferred Scope

These are not prerequisites for the first gameplay prototypes:

- player colonies and large-scale base construction;
- complete macroeconomic simulation of every background purchase;
- millions-of-player service architecture;
- a cursor-addressed TUI;
- ansibles or any other FTL communication not physically carried by ships;
- Zimm, warp, teleport, hyperspace, or other alternative drives; and
- flat sector, subsector, or hex-board navigation.

## Maintaining This Roadmap

When a milestone begins, completes, or changes order:

1. update **Resume Here** and the milestone table in the same change;
2. mark a milestone Complete only after its acceptance boundary passes;
3. update the relevant detailed design document and
   [`LLM_INSTRUCTIONS.md`](LLM_INSTRUCTIONS.md) when a design decision changes;
4. record newly discovered work under its actual prerequisite milestone rather
   than silently making it the current task; and
5. keep implementation inventories and measured results in their detailed
   documents instead of expanding this file into a second design specification.
