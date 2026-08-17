# Guided First Watch

*Status: Stage 1 Departure Watch is implemented for Milestone 7 field-alpha
work. Stage 2 is the client-guided Arrival Watch. Server-backed continuity and
other protocol additions are separate, evidence-gated extensions; none of
this work reorders the roadmap.*

## Decision

Cepheus Trader's new-player tutorial is an optional **Guided First Watch** in
the player's real ship and the shared live universe. The OpenDoors client owns
the coaching copy, screen transitions, and presentation progress. Ordinary
server snapshots and transactions remain the only source of game facts and
the only way tutorial actions affect the universe.

The first field-alpha implementation should use existing CT-RPC snapshots and
transactions without adding a tutorial protocol. It may remember presentation
progress in the BBS-local player preferences, but it must rediscover current
ship, account, task, knowledge, and flight-plan state whenever guidance is
shown. A later Arrival Watch can use the same client-guided approach and
existing authoritative snapshots after the first journey. A small
server-owned First Watch record or recommendation API may be added
independently if field use demonstrates a need for durable cross-client
progress or policy that cannot be expressed safely from existing snapshots.
Arrival Watch does not, by itself, establish that need.

There is no separate tutorial universe. An isolated, disposable combat
simulator may be designed later as a different feature, but it is not the
foundation of new-player instruction and never transfers money, equipment,
people, authority, or advancement into the live universe.

## Purpose

Character creation explains the captain and starting command, but the first
docked screen exposes many managers, services, obligations, and delayed-
information rules at once. Static Beginner Help describes those systems but
does not help a captain recognize the next useful action in the command's
current circumstances.

Guided First Watch bridges that gap. It leads a new captain through one safe,
real preparation cycle:

1. inspect the command and its crew;
2. understand title, cash, restricted funds, debt, insurance, and operating
   costs;
3. review delivered messages, Tasks, and career obligations;
4. inspect a suitable real opportunity when one is available;
5. choose and preview a reachable destination;
6. verify fuel, provisions, departure cash, deadlines, and warnings; and
7. commit the first flight plan and depart.

The initial **Departure Watch** ends when a valid departure commits. A Jump can take
hours of wall time, so arrival and settlement cannot be prerequisites for
finishing an introductory terminal session. **Arrival Watch** is the next
player-facing stage: it resumes optional contextual guidance when the ship
reaches its destination, but is a separate watch rather than an unfinished
Departure Watch.

## Goals

- Teach the actual menus and actual rules using the captain's persistent ship,
  accounts, crew, information, and opportunities.
- Preserve player agency. Guidance explains and navigates; the captain chooses
  every commitment and confirms every irreversible action normally.
- Adapt to trader, privateer, and naval starting commands without pretending
  that all three have the same finances, authority, or useful first duty.
- Survive changing markets, expiring offers, remote-claim races, mail delay,
  another player's actions, reconnects, and stale client observations.
- Remain useful at 40x24 in every supported output profile and with printable
  navigation keys.
- Be optional, dismissible, and resumable without affecting game eligibility,
  prices, task outcomes, or advancement.
- Gather field-alpha evidence about where players actually stop or become
  confused before committing to a larger protocol and persistence surface.

## Non-goals

- A scripted demonstration with fixed offer IDs, destinations, prices, or
  outcomes.
- A protected economy, risk-free duplicate ship, paused universe, accelerated
  Jump, or exemption from ordinary fees, deadlines, law, mail, maintenance,
  combat, or fuel rules.
- Automatic offer acceptance, purchases, crew changes, course commitment, or
  departure.
- A complete playthrough of cargo delivery, prize adjudication, naval
  promotion, combat, ship acquisition, or long-term progression.
- An omniscient adviser. Guidance may use only current local facts and the
  captain's delivered or carried knowledge.
- A replacement for Beginner Help, the Player Reference, or normal screen
  help.
- A cursor-addressed overlay or a second terminal UI framework.

## Design Principles

### Teach the real game

Every inspected balance, offer, deadline, route, fuel source, and warning is a
normal live-universe value. Every state change uses the same command that an
unguided player uses. Tutorial mode never creates a second authority in the
client and never turns an observation into permission.

### Guide objectives, not keystroke scripts

The stable unit of guidance is an objective such as **review command
readiness**, **inspect available duty**, or **preview a reachable course**.
Exact menu shortcuts and list positions are presentation details. Exact task
and system IDs are ephemeral recommendations that must be refreshed before
use, not durable tutorial steps.

### Explain before commitment

Guidance may offer a shortcut into the relevant manager or filtered view, but
it must not submit the resulting transaction. The ordinary detail screen,
back path, warning, and confirmation remain visible. Leaving a guided screen
does not imply acceptance or completion.

### Degrade without trapping the captain

A captain can have no suitable local offer, can lose an offer race, can change
ships, or can already have departed before guidance resumes. The controller
recomputes its useful next objective and may skip an obsolete lesson. It never
requires a vanished public object to advance.

### Keep game rules out of presentation code

The client may select among server-reported availability and route-assessment
facts according to a documented coaching policy. It must not reproduce fuel,
deadline, capacity, title, law, or phase rules. A standard transaction remains
authoritative and may reject a recommendation that became stale.

## Player Experience

### Invitation and access

After a successful `CreatePlayer`, the client offers an in-world invitation
to begin the command's first watch. Accepting starts guidance at the first
incomplete useful objective. Declining leaves the captain at the ordinary
docked menu and does not ask again during that session.

Guidance remains reachable from the Command Console and Player Preferences.
Those surfaces distinguish:

- **Begin or resume First Watch**;
- **hide First Watch guidance**; and
- **restart presentation lessons**, which resets only locally remembered
  viewing progress and never rewinds game state.

The client should not automatically invite existing captains merely because a
new client version gains the feature. Existing captains may start it
voluntarily. If a later server record is implemented, new-player creation
creates the offered state atomically; absence of a record means no automatic
invitation.

### Presentation

First Watch is a page-oriented in-world briefing from the ship's command
library or computer. It may contain:

- what the captain is trying to establish;
- why that fact matters;
- a short summary of relevant already-known facts;
- the next ordinary screen that can answer the question;
- a visible action to open that screen;
- skip, hide, help, and return choices; and
- a clear statement when circumstances have changed and guidance was
  refreshed.

Player-facing copy must not mention tutorial stages, RPCs, snapshots,
databases, revisions, authoritative state, client inference, or implementation
status. For example, the implementation objective `reviewFinance` may be
presented as "Review the command's accounts before accepting an obligation."

Guidance uses normal responsive wrapping, paging, semantic colours, and form-
feed or enhanced page transitions. It does not reserve a status line, draw a
modal overlay, or assume more than 40 columns and 24 rows.

### Progression

The conceptual progression is deliberately coarse:

| Objective | Evidence and behavior |
| --- | --- |
| Welcome aboard | Explain the live universe, normal time, and the ability to leave or resume guidance. Presentation-only acknowledgement. |
| Review command | Open Crew and Ship Management; notice fuel, provisions, hold, watches, and material limitations. Viewing acknowledgement is sufficient. |
| Review accounts | Open Banking and Accounts; explain the displayed title and career-appropriate obligations. Viewing acknowledgement is sufficient. |
| Review information and duty | Open Messages for every command. Traders inspect Tasks; privateers inspect Operations and then eligible ordinary Tasks; naval captains inspect Operations and issued orders instead of commercial Tasks. Use delivered information only. Viewing acknowledgement is sufficient. |
| Inspect an opportunity | Traders and eligible privateers inspect, but need not accept, a suitable current offer. Naval captains continue from issued duty to navigation without commercial offer discovery. Skip when no applicable opportunity exists. |
| Choose a course | Open Known Universe or a task-oriented course shortcut and obtain a real Flight Plan preview. A current preview is ordinary game state, not tutorial state. |
| Check readiness | Review preview warnings, departure cash, fuel, provisions, crew coverage, and obligation timing. Never suppress a warning to simplify the lesson. |
| Take the watch | The captain confirms and commits an ordinary flight plan and departure. A successful transition out of the initial berth completes First Watch. |

The order is advisory rather than a permission chain. A knowledgeable player
may plot and depart immediately. On the next guidance request, earlier
objectives that no longer matter are summarized or skipped. Guided menus never
disable otherwise legal game commands.

### Career adaptation

The selected starting offer already identifies the command's broad career
context. Guidance changes both its emphasis and its objective sequence without
inventing a separate ruleset:

- **Trader:** title, secured principal, insurance, operating runway, cargo and
  passenger capacity, and feasible commercial Tasks.
- **Privateer:** sponsor ownership, mandatory insurance, restricted operating
  credit, commission and prize authority, and lawful targets come first.
  Ordinary work may then be inspected as optional support between cruises when
  the server reports that the command is eligible for it.
- **Navy:** institution ownership, restricted operating credit, issued orders,
  rank and reporting duties, and the limits of institutional authority. The
  watch proceeds from Operations and messages to navigation and readiness; it
  skips commercial Tasks and offer discovery rather than implying that a naval
  command should carry trade cargo or passengers.

All tracks still inspect ship, crew, information, readiness, and Flight Plan.
Trader and eligible privateer tracks also inspect the Task Ledger and a routine
offer. The tutorial must not push a privateer into immediate combat, present
naval restricted funds as personal purchasing power, or route naval captains
through commercial lessons merely to make the tracks look alike.

## Real Opportunity Discovery

### Existing authoritative inputs

The initial client controller uses the ordinary Task Ledger where appropriate,
Operations Ledger, Message archive, Known Universe, Finance, Ship, Crew, and
Flight Plan snapshots. The Task Ledger already carries contextual availability
and route assessment; trader and eligible privateer guidance consumes those
fields and does not recalculate their rules. Naval guidance does not query it
for a tutorial opportunity.

An opportunity is a teaching candidate, not a reserved tutorial asset. The
client should prefer a candidate with these server-reported properties:

1. issued in the current system, so a valid claim can be awarded locally
   rather than beginning with an interstellar claim race;
2. available to the current ship and crew;
3. no remote pickup before the first delivery;
4. a known destination reachable by a simple course with comfortable fuel and
   deadline margins;
5. no special cargo, passenger, legal, combat, or facility prerequisite the
   captain has not yet been introduced to;
6. affordable bond, loading, berth, fuel, and departure requirements; and
7. the lowest stable offer identifier as the final deterministic tie-breaker.

These are coaching preferences, not additional acceptance rules. The detail
screen shows the real terms and the ordinary server transaction revalidates
them. A recommendation is refreshed after any relevant state change and
immediately before entering its detail screen.

### No suitable opportunity

The First Watch does not block when no suitable opportunity exists. It tells
the captain, in world, that no routine duty currently fits the command and
continues with a known reachable destination, existing obligation, or simple
readiness exercise. Inspecting an offer is educational; accepting a Task is
not required to depart or complete First Watch.

If field-alpha evidence shows that lack of a suitable first duty is common and
materially harms onboarding, a later design may add a private **orientation
dispatch**. Such a dispatch must be a real personal Task with normal custody,
route, time, mail, settlement, and failure behavior. It may avoid a public
claim race and use generous terms, but it receives no magical fulfillment,
no tutorial-only rule exemptions, and no reward unavailable through ordinary
play. That addition requires separate economy and abuse review; it is not part
of the initial design.

### Races and stale recommendations

Another captain may claim a public offer between display and acceptance.
Guidance presents the normal result, refreshes the ledger, and recommends a
new useful objective. It does not silently substitute another offer or retry a
state-changing command. This both preserves the shared economy and teaches
that an offer is information rather than ownership.

## Initial Client Architecture

### First Watch controller

The door adds a `FirstWatchController` or equivalent client-owned component.
It is not part of the shared TLS transport DLL and owns no rule-bearing state.
Its responsibilities are:

- retain whether guidance is active, hidden, or locally complete;
- retain presentation-only acknowledgements for screens the captain has
  viewed;
- request ordinary manager snapshots as needed;
- derive the next presentation objective from those snapshots;
- rank only candidates already assessed by the server;
- open the relevant existing screen or filtered manager view;
- observe successful ordinary client results and refresh guidance; and
- render in-world guidance through the normal door presentation helpers.

The controller must not keep a second copy of player balances, task status,
flight plans, or phases. It may cache one response while rendering a page, but
every return to guidance reacquires facts that can have changed.

### Local preference state

The pilot implementation may store the following non-authoritative fields in
the existing protected BBS-local identity/preferences registry:

```text
first_watch_mode: notOffered | active | hidden | locallyComplete
first_watch_presentation_version: UInt16
first_watch_seen: bit set of presentation-only objectives
```

Stage 1 stores these fields in identity-registry format 4. Readers retain
formats 1 through 3; legacy entries default to `notOffered`, presentation
version 1, and an empty viewed-objective set. Invalid optional First Watch
values recover to those defaults without invalidating an otherwise sound
identity record. Checksums, ownership, BBS identity, and structural corruption
remain hard failures because they protect the identity mapping itself.

This state is scoped to the BBS-local player identity. Losing it can at worst
repeat an explanation; it cannot duplicate a reward, undo an action, or alter
the universe. A changed presentation version may selectively repeat materially
revised safety instruction without changing completion of real actions.

The implementation must preserve the registry's existing Windows access,
locking, and recovery rules. Tutorial persistence is never allowed to make
door startup fail: an unreadable optional preference falls back to hidden or
unacknowledged guidance and reports through the existing diagnostic path.

### Integration boundaries

- `CreatePlayer` remains one atomic authoritative transaction. The invitation
  appears only after its successful result.
- Existing manager and dock-service screens remain independently usable and
  keep their normal back paths.
- A guided shortcut invokes the same function as the corresponding ordinary
  menu selection.
- The controller observes returned typed results rather than parsing rendered
  English text.
- It never changes the C ABI of `cepheus-trader-client-core`; future CT-RPC
  additions remain in the common protocol linked into the executable.
- Headless protocol exercisers require no tutorial presentation behavior.

## Stage 2: Arrival Watch

Arrival Watch is the direct continuation of the player experience, not a
reason to introduce tutorial authority on the server. It begins when a captain
who completed Departure Watch reconnects after reaching the planned
destination. Like Departure Watch, it guides the captain through ordinary
client screens backed by fresh live-universe observations and transactions.

Its initial objectives are to:

1. read the arrival packet and recognize the ship's current condition;
2. inspect local facilities, fuel, provisions, berth costs, and other immediate
   needs;
3. review carried cargo, passengers, task custody, deadlines, and messages;
4. complete or advance an applicable obligation through its normal screen;
5. inspect the resulting accounts and operating position; and
6. identify a sensible next action without requiring a new departure.

The exact sequence adapts to career and circumstances. A captain may arrive
without a deliverable Task, may need repairs or fuel before commerce, or may
have naval or privateering duties instead of cargo settlement. No single
transaction is required merely to satisfy the watch. Arrival Watch completes
after the captain has reviewed the arrival and the useful local follow-up
surfaces; any purchase, delivery, report, claim, or new course remains an
ordinary player choice.

The reconnect boundary is expected. The client remembers only enough local
presentation state to offer Arrival Watch after a qualifying First Watch
departure and to avoid repeatedly presenting completed lessons on the same
BBS installation. It reacquires arrival, ship, account, task, market, message,
and facility facts whenever guidance is shown. Loss of local tutorial state
may repeat guidance but must not change or repeat game transactions.

Arrival Watch should first be implemented with existing CT-RPC observations.
If a required arrival or settlement fact is not available, that specific gap
should be documented and addressed on its own merits. Cross-installation
tutorial persistence, server-selected recommendations, and telemetry are not
prerequisites and are not implicitly included in Stage 2.

## Optional Server Extension

The server extension is a separate possible project, not Stage 2. It is
intentionally deferred until field evidence shows that local guidance is
insufficient. Valid reasons include resuming on a different door installation,
coordinating concurrent sessions, or choosing a career-appropriate
recommendation without duplicating rule-bearing policy. Arrival Watch can be
implemented and shipped without it.

### Authority split

Even with the extension:

- the server owns durable disposition, fact-derived completion, and any
  recommendation that depends on game rules or undisclosed state;
- the client owns prose, layout, menu routing, and presentation-only progress;
  and
- all game actions continue to use their existing commands.

The server returns typed enums and identifiers, never player-facing tutorial
paragraphs. The client maps those values to localized in-world copy.

### Conceptual protocol

The exact schema ordinals are assigned only during implementation. The
conceptual observation is:

```text
GetFirstWatch

FirstWatchSnapshot
  revision: UInt64
  disposition: offered | active | hidden | complete
  track: trader | privateer | navy
  nextObjective: FirstWatchObjective
  completedFacts: List(FirstWatchFact)
  recommendation: FirstWatchRecommendation
  blockers: List(FirstWatchBlocker)

FirstWatchObjective
  welcome
  reviewCommand
  reviewAccounts
  reviewInformation
  inspectOpportunity
  chooseCourse
  checkReadiness
  depart
  complete

FirstWatchRecommendation
  none
  task(taskId, offerRevision)
  operation(recordKind, recordId, revision)
  destination(systemId, knowledgeRevision)
  flightPlan(planRevision)
```

`GetFirstWatch` is an observation. It returns only facts visible through the
captain's current local state and carried knowledge. Recommendations are
advisory references to ordinary records; their revisions allow the client to
recognize staleness but do not reserve them.

Durable disposition changes use an idempotent transaction such as:

```text
SetFirstWatchDisposition
  expectedRevision: UInt64
  disposition: active | hidden
```

The server may mark completion in the same transaction that successfully
commits the qualifying initial departure. It must not mark an objective
complete merely because the client requested a screen. If durable view
acknowledgements are later considered useful, they are explicitly
presentation telemetry, never proof of a gameplay action or a prerequisite
for game permission.

Adding this protocol requires the ordinary CT-RPC compatibility bump and
coordinated client/server release. The client-only pilot deliberately avoids
that cost while the interaction is still being evaluated.

### Persistence

A future durable record should live in a separate versioned database keyed by
player identity rather than expanding the core `PlayerRecord` codec solely for
presentation. A version-one record needs only:

```text
record_version: UInt8
revision: UInt64
disposition: offered | active | hidden | complete
created_second: UInt64
completed_second: optional UInt64
```

Career track and completed gameplay facts are derived from the starting offer,
ship, Tasks, flight plan, and phase rather than copied into the record. Offer,
operation, and destination recommendations are observations and are never
persisted as promises.

The record participates in live backup, player removal, universe
reinitialization, codec validation, and repository compatibility checks. Its
mutations enter the ordered engine queue, use command IDs, and are safe under
replay. It has no scheduled work and never advances simulation time.

## Information and Security Boundaries

- Recommendation may use local current state and already delivered or carried
  knowledge. It cannot query hidden current conditions at a remote system for
  the captain's benefit.
- Tutorial state grants no authorization and is ignored by all ordinary game
  rule checks.
- Hiding, restarting, or completing guidance produces no credits, items,
  reputation, rank, insurance relief, task priority, market reservation, or
  combat advantage.
- Concurrent sessions may disagree temporarily about presentation progress,
  but standard revisions prevent a stale durable preference write from
  overwriting a newer one.
- Diagnostics may identify a broken tutorial record or stale reference;
  ordinary player copy describes only what the command library can currently
  recommend.
- Telemetry, if added, is aggregated operational data. It must not retain
  message bodies, player-entered names, private correspondence, credentials,
  or hidden game information.

## Why There Is No Tutorial Universe

A controlled pocket universe appears deterministic but creates the wrong
engineering and teaching boundary:

- markets, offers, mail, time, travel, facilities, law, encounters, and
  persistence would need a parallel lifecycle or pervasive scenario modes;
- state transfer into the live universe would create duplication and abuse
  boundaries for ships, people, money, authority, knowledge, and rewards;
- a solitary deterministic scenario cannot teach public offer races, stale
  information, ship-carried mail, or a universe that continues while the
  captain is absent;
- fixes to ordinary gameplay could drift from the scenario and teach obsolete
  behavior; and
- deploying and recovering another universe instance materially increases
  field-alpha operational complexity.

An eventual combat simulator is different because it can have a narrow input,
reuse the authoritative combat resolver, discard its output, and transfer
nothing. It should be presented as an in-world simulator and specified with
its own threat, persistence, and test boundaries.

## Failure and Recovery Behavior

- **Optional preference unavailable:** continue the door normally with
  guidance hidden or unacknowledged; do not fail startup.
- **Snapshot request fails:** show the ordinary actionable transport or server
  error and leave tutorial presentation progress unchanged.
- **Recommended record is stale or gone:** return to guidance, refresh, and
  choose a new objective; never substitute during confirmation.
- **Captain changes ship or command:** discard cached recommendations and
  recompute from the newly commanded ship.
- **Captain departs outside guidance:** recognize the real phase on reconnect
  and locally complete or skip the docked First Watch.
- **Captain loses command, dies, or enters succession:** suspend First Watch;
  command recovery and succession take precedence.
- **Encounter begins:** suspend coaching except for ordinary contextual Help.
  First Watch does not alter encounter timing or standing orders.
- **Server restart:** ordinary player, task, plan, and phase records determine
  the resumed objective; losing local view acknowledgements may only repeat
  copy.
- **Client upgrade changes presentation:** use the presentation version to
  repeat only materially changed safety explanations.

## Testing

### Client unit and presentation tests

- Objective selection from representative combinations of phase, finance,
  task, plan, and local presentation state.
- Deterministic candidate ranking from server-assessed offers.
- No-candidate, vanished-candidate, stale-revision, changed-ship, already-
  departed, succession, and encounter cases.
- Begin, hide, resume, restart-copy, and locally complete preference behavior.
- Proof that guided shortcuts call existing screens and do not submit a
  mutation.
- Exact typed-result handling without parsing player-facing prose.
- 40x24 and 80x24 output in plain, colour, and CP437 profiles, including
  paging, wrapping, visible back paths, and confirmation preservation.
- In-world-language review of every new or changed screen.

### Server tests for the optional extension

- Record codec round trip, invalid enum/version rejection, restart, live
  backup, player removal, and universe reinitialization behavior.
- Idempotent disposition mutation, optimistic revision conflict, and command
  replay.
- Track derivation from every starting career cell without copied career
  state.
- Recommendation uses only locally current or carried-known facts.
- Public-offer loss between recommendation and acceptance returns the normal
  rule rejection and does not corrupt First Watch state.
- Departure completes First Watch in the same committed transaction and cannot
  complete twice.
- Hidden, restarted, or complete status never changes rule eligibility or
  awards a game asset.

### End-to-end acceptance

The real TLS/OpenDoors harness creates representative trader, privateer, and
naval captains and verifies that each can:

1. accept or decline the First Watch invitation;
2. navigate the advised real managers with a working return path;
3. inspect live career-correct finances and duty;
4. follow the career-appropriate duty path: commercial Tasks for a trader,
   Operations followed by optional eligible work for a privateer, and issued
   orders without commercial offer discovery for a naval captain;
5. preview a real course and see all normal warnings;
6. back out without mutation;
7. commit through the ordinary confirmation path; and
8. reconnect in the resulting phase without a repeated mandatory tutorial.

Field-alpha review additionally notes where volunteers hide guidance, request
Help, back out, encounter no suitable opportunity, or depart. Those manual
observations inform copy and ordering changes and separately decide whether an
optional server extension or private orientation dispatch is justified.

## Delivery Plan

### Stage 1: client-guided Departure Watch

Implemented in the OpenDoors client. Field-alpha observation is manual; Stage
1 adds no telemetry.

- Add the First Watch invitation, controller, local optional preferences, and
  in-world pages.
- Reuse existing manager snapshots and typed results.
- Rank only server-assessed real opportunities.
- End at a successfully committed initial departure.
- Add door unit, presentation, and real TLS/OpenDoors coverage.
- Do not change CT-RPC, authoritative server storage, or the authoritative
  server.

### Stage 2: client-guided Arrival Watch

- Offer a new optional watch when the first planned journey reaches its
  destination.
- Guide the captain through the arrival packet, immediate ship needs, local
  facilities, carried obligations, settlement opportunities, and accounts.
- Use existing live observations and ordinary transaction screens; do not
  require server-owned tutorial state.
- Remember only presentation progress locally and tolerate its loss safely.
- Add client unit, 40-column presentation, reconnect, and real TLS/OpenDoors
  coverage.

### Evidence-gated follow-up: field reconciliation

- Observe real BBS sessions and document where guidance, inference,
  persistence, or candidate discovery fails.
- Revise copy and objective order without changing game rules.
- Decide each proposed protocol or persistence addition from evidence rather
  than treating it as a prerequisite for Arrival Watch.

### Optional extension: server-backed continuity

- Add the minimal observation, disposition transaction, and separate
  versioned record described above.
- Bump CT-RPC and release client/server together.
- Keep prose and presentation logic in the client.
- Do not add a private orientation Task unless the field evidence separately
  satisfies its economy and abuse review.

After each implementation stage, development returns to the roadmap's current
Milestone 7 field-alpha and operations queue. Completing Guided First Watch
does not advance Milestone 7 or begin Milestone 8.

## Resolved Questions

- **Live or isolated universe?** Live universe.
- **Client or server presentation?** Client presentation and navigation.
- **Initial protocol addition?** None; use existing typed snapshots and
  transactions.
- **What is Stage 2?** Client-guided Arrival Watch using the real arrival and
  ordinary local screens.
- **Does Stage 2 require server tutorial support?** No. Any server-owned
  disposition or recommendation is a separately justified extension.
- **Long-term tutorial authority?** Add server-owned disposition or
  recommendation only if field evidence requires it.
- **Fixed tutorial offer?** No. Discover real opportunities and continue
  safely when none fits.
- **Must the captain accept a Task?** No. Inspection teaches the surface;
  acceptance remains a strategic choice.
- **When is First Watch complete?** On the first valid committed departure.
- **Can an existing captain use it?** Yes, voluntarily; no automatic prompt.
- **Does it award anything?** No.
- **Does it pause or accelerate time?** No.
- **Is combat training included?** No; a disposable simulator would be a
  separate later feature.

## Open Questions for Field Testing

- Do players benefit more from inspecting one recommended offer or from
  comparing two career-appropriate choices?
- Which view-only objectives are worth remembering locally, and which should
  simply be rediscovered from the current context?
- Is departure the right final emotional beat, or should the client leave one
  short reconnect reminder for the eventual arrival packet?
- How often does a new captain have no suitable local opportunity under real
  market and task generation?
- Do concurrent sessions or client reinstalls create enough repeated guidance
  to justify server-backed disposition?
- Do privateer and naval starts need distinct objective ordering beyond their
  finance and Operations briefings?
