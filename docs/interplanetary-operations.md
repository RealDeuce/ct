# Interplanetary Operations and Continuation Plans

## Purpose

`Interplanetary` is primarily a decision and continuation phase. A ship has an
authoritative physical state and may also have a committed continuation plan.
The player can inspect that plan, continue a suspended plan, append later
legs, replace any step the engine has not already processed, or hold. Routine
travel must not require a login at every simulation boundary.

A plan is assembled through client prompts but submitted as one atomic
command. Committing that command records bounded future intent; it does not
hold one database transaction open while game time passes. Each later
movement, checkpoint, encounter, and terminal action is a separate scheduled
engine transaction derived from the committed plan.

## Plan Contents

A continuation plan records at least:

- the ship-state revision from which it was prepared;
- the current position, velocity, and attachment state used as its origin;
- one or more ordered destinations or intercept targets;
- flight profiles, acceleration limits, and applicable standing policies;
- any preauthorized action at each destination;
- bounded completion conditions, quantities, and safety limits;
- an idempotent command ID and the player/session authority that committed it;
  and
- the current step, suspension reason, and next scheduled checkpoint.

The client may gather these choices through several pages and make
authoritative read-only queries while doing so. Nothing changes until the
complete proposal is accepted. A revision conflict or invalid choice rejects
the proposal without partially settling fees, releasing a berth, changing
course, or reserving resources.

## Flight Plan Interface

The phase-level **Flight Plan** interface owns the executable route. It is not
a universal manager: `Depart` invokes it while docked, and the travelling
phase keeps it available while the ship is underway. Known Universe may copy a
calculated route into it, and Task Management may open it with a task's
destination and deadline, but neither owns the accepted plan.

`Depart` commits an initial system before ordinary cargo, passengers, and mail
are auto-accepted for that destination, then begins the maneuver toward the
departure locus. It does not require the captain to settle the entire later
itinerary. During that maneuver the captain may append systems or atomically
replace any unprocessed routing, including the next system.

Changing the route does not rewrite what the captain agreed to carry. Cargo
remains aboard and passenger, mail, contract, and order destinations remain
outstanding. The proposal must show resulting delays, breaches, refunds, or
other known consequences before submission. Only movement and checkpoints
already processed by the engine are immutable; queue ordering decides whether
a replan beats a due checkpoint.

The interface distinguishes course calculation, checkpoint authority, and plan
completion. Hold waits for arrival watch; Through permits standing encounter
policy and continuation while the captain is away. A separate terminal marker
belongs to the last step and ends the plan after that step completes. It does
not determine whether a contact waits for the captain. Generated task and
ordinary routes default to Through throughout, with separate BBS-local
preferences able to generate Hold checkpoints instead.

## Destinations and Preauthorized Actions

A destination and an action at that destination are separate choices. A plan
may end by holding at a location, or it may authorize the server to perform a
bounded action immediately after arrival. Examples include:

- approach a jump-departure locus and hold;
- approach a jump-departure locus, perform final readiness checks, and
  initiate a specified Jump using the selected plot or tape;
- approach a port and hold for further orders;
- request clearance and dock or land at a specified facility;
- reach a gas giant, deploy the selected scoops or boats, and skim a specified
  quantity of fuel subject to declared safety limits;
- reach a known accessible water or ice source on an unoccupied world, moon,
  or ice-bearing belt, land or deploy suitable craft, and collect or process
  a specified quantity of fuel;
- rendezvous, join a convoy, rescue, transfer cargo, or perform another
  explicitly supported local operation; or
- perform several ordered legs, such as departing port, skimming a declared
  fuel quantity, proceeding to the inhabited world, and docking.

The Docked `Depart` wizard always collects an initial system because ordinary
carriage is offered against that commitment. Whether it also contains a valid
authorized Jump operation is separate. The captain may finish or revise that
operation from Flight Plan while outbound; reaching the departure locus
without one causes the ship to hold there.

Hydrographics is not permission to take water. An inhabited world's oceans,
ice caps, and other water reserves are controlled resources and are
unavailable for routine wilderness fueling unless the captain has an explicit
license, contract, naval/public authority, or emergency authorization. The
normal frontier-water targets are unoccupied planets or moons and surveyed
ice-bearing asteroid belts. Claims and local law can still restrict an
unoccupied source.

A captain may attempt to extract controlled water by force, but that is not a
fuel-service shortcut. It is a hostile local operation requiring the ship to
overmatch or evade the relevant planetary authority and can create combat,
crime, warrants, political consequences, and news. A sufficiently armed ship
is capable of imposing its will; that capability does not make the extraction
lawful. Government-backed naval or public-service ships may instead carry
authority that grants access.

Every automatic action must be finite and parameterized. A plan may say “skim
200 tons unless hull damage reaches this limit”; it must not say “keep doing
whatever is profitable.”

## Checkpoints Are Not Mandatory UI Stops

Every plan crosses authoritative checkpoints where the simulator:

1. advances the ship to the checkpoint time and physical state;
2. processes all globally due work through that time in queue order;
3. evaluates traffic, detection, interception, and encounter opportunities;
4. revalidates ship, crew, fuel, law, clearance, navigation, and target state;
   and
5. either schedules the next authorized step or suspends the plan.

Typical checkpoints include facility departure, departure traffic and
customs, convergence zones, arrival approaches, gas-giant operations,
jump-departure loci, and docking or landing approaches.

A checkpoint does not itself require player input. If no encounter occurs,
the plan remains valid, and the next step was already authorized, execution
continues automatically. This is true for initiating Jump, docking, landing,
skimming, rendezvous, and other supported terminal actions.

The plan is suspended when:

- an encounter begins;
- arrival requires the captain to decide whether to disclose a system not
  known by the current repository to be publicly mapped;
- validation fails or an expected service, target, or clearance is gone;
- damage, fuel, crew readiness, cargo, or legal state invalidates a limit;
- the bounded plan completes at a hold point; or
- the player replaces, cancels, or deliberately pauses it.

An encounter retains the suspended continuation. After resolution, the server
revalidates it against the resulting state. If it remains viable, the player
may continue all still-authorized steps without rebuilding the itinerary. If
it is no longer viable, the UI explains the conflict and offers replanning.

The mapping-disclosure prompt is similarly an intentional interruption. Send
publicly, send directly to Earth, withhold, and withhold-and-mark-secret are
information-policy decisions that a route plan cannot make implicitly. A
Secret Systems entry suppresses the prompt for that captain; otherwise a
disconnect or timeout withholds the notification and leaves the continuation
suspended.

## Interplanetary Commands

The initial phase surface should provide:

- **Continue plan:** resume the still-valid committed continuation, including
  any terminal action already authorized.
- **Flight Plan:** inspect, append, or atomically replace unprocessed route
  steps and through-point authority.
- **Hold:** cancel future movement and remain at the current physical state,
  with the resulting exposure and operational consequences.
- **Local action:** construct a new bounded operation appropriate to the
  current location.
- **Universal managers:** Crew, Ship, Task, Message, and Known Universe.

The player may inspect and replace future intent while routine transit is
underway. Queue ordering decides whether a replanning command commits before
an already-due checkpoint; the client cannot retroactively cancel an event
that the engine has processed.

## Implemented Flight Plans and Legs

The current server persists a typed, bounded `FlightPlanSnapshot` separately
from the ship's active physical leg. The door constructs a proposal, previews
elapsed time, fuel, carriage warnings, and standing encounter authority, then
commits the unchanged proposal with its preview hash. A revision mismatch
rejects the whole transaction. The first actionable step may be a Jump or a
bounded frontier-fuel operation. A Jump begins at the exact departure port and
runs toward a typed Jump locus using current seed-derived celestial positions,
catalogued thrust, and the half-day safety minimum.

During any in-system maneuver, the captain may atomically replace the active
destination and all later waypoints. The replacement begins from the ship's
current relative position and velocity, solves a new bounded-thrust intercept
to the destination's moving orbital state, and replaces the old scheduled
movement. This permits returning to the primary port or diverting directly to
the Jump locus, a selected planetoid belt, or a lawful frontier-fuel source.
Only a physical Jump-space leg is immutable until breakout. Carried cargo and
sealed mail remain authoritative ship state and produce diversion/custody
warnings rather than being altered by the route editor.

The due-time scheduler then commits three transitions independently:

1. outbound approach completes, one catalogued jump-fuel load is consumed,
   and the ship enters a standard one-week Jump;
2. Jump completes in the destination system and a fresh safe-locus-to-primary-
   world approach is scheduled from destination celestial positions at that
   game time; and
3. the approach completes at a durable arrival checkpoint at the synthesized
   primary-world starport facility.

A Hold checkpoint waits until the captain takes arrival watch, even if the
maneuver therefore takes longer. A Through checkpoint applies standing policy
offline. Either may also be the terminal step: the terminal marker ends the
plan after the checkpoint's selected authority has completed. The CE contact baseline is one
chance in six per candidate and is scaled over the deterministic ±60-minute
local traffic pool as `1 - (5/6)^N`; empty space has no comparable arbitrary
roll. Routine traffic, control, inspection, distress, derelict, hazard,
military, and hostile contact types all have closed wire discriminants.

A committed voyage schedules contact-check work at port departure, the
Jump-departure convergence window, Jump arrival, and frontier-fuel arrival;
the terminal inhabited-world approach creates its own checkpoint. Every check
is a separately admitted durable engine input. Non-consequential traffic is
recorded without inventing a pause. A hostile result changes the plan to
Encounter, and if its turns overlap a physical-leg due time that leg is
re-timed rather than allowed to execute through the fight.

Hostile posture and ordered fallbacks are submitted once. The server then
resolves opposed crew actions in individual one-kilosecond scheduled engine
inputs. Each turn has its own ingress sequence, logical time, revision,
journal entry, damage mutation, and opportunity for later causal observation.
Resolution either resumes the still-valid continuation, docks, or persists a
terminal destroyed/captured/command-lost state.

The implementation is derived from the Open Game Content portions of the
Cepheus Engine SRD for encounter probability, detection, opposed ship tasks,
damage, pursuit, surrender, and boarding. Generic port/gas-giant/Jump-locus
encounter categories were cross-checked against the Open Game Content in
*Skull and Crossbones Third Edition*. No setting names, named characters,
setting prose, or other Product Identity is encoded. The canonical Section 15
records remain in `catalog/ogl-sources.toml` and are merged into the shared
build attribution.

Player travel events and background system/mail/traffic events enter the same
authoritative input queue. Due time controls eligibility only. If several
scheduled events become eligible together, their globally allocated creation
IDs determine admission order; event type supplies no hidden priority. A
dependent operation is created only after its prerequisite commits—for
example, completing the return from frontier fueling schedules the final ship
activity after the ship is docked. Every transition advances the game clock
and receives its own committed sequence, revision, and journal record. The
typed location record and its scheduled event survive shutdown. The
integration test carries a purchased cargo lot through the circuit and checks
that exactly one jump-fuel quantity is consumed.

`GetTravelStatus` and `GetFlightPlan` are phase-independent and return the carried current system,
destination, stage, current/due game seconds, fuel, one-jump requirement,
plan identity/revision, leg index, and typed origin/destination loci. The door
presents it on an underway login, while the universal managers remain
available. The live server advances the same authoritative scheduler at 28
game seconds per real second, so a standard one-week Jump completes in six
real hours without requiring a connection or administrator command. Downtime
is frozen: restart resumes from the last committed game second and anchors a
fresh monotonic process clock there.

Gas-giant skimming and lawful wilderness water/ice collection use the same
typed scheduling machinery. The accepted command fixes a whole-ton quantity
and whether the collected batch is to be refined, then creates three explicit
legs: port to source body, collection/optional processing at the body, and body
to the original port. Collection and processing overlap where possible; the
service phase lasts the longer of the two. Completion of the return leg places
the ship at the port; the same-minute activity event then fills the tanks,
records relevant duty and skimming cycles, voids the ordinary warranty after
gas skimming, and consumes the activity atomically. These bounded operations
also execute as Flight Plan body waypoints. Preview validates the named body,
operation, projected tank capacity, equipment, time, and fuel state. Execution
uses that exact source rather than silently substituting the nearest body; on
return to port, the activity completion transaction starts the next authorized
plan step or leaves the plan held with the validation failure.

Preview reports the exact selected-quantity travel, collection, normal
processing, failed processing, and resulting total durations. It does not
reveal the deterministic task roll before commitment. Standalone processing
of fuel already aboard is a separate stationary Flight Plan action allowed at
a berth or safe holding locus. Failure doubles processing time; Effect -6 or
worse damages the Jump drive, with maneuver-drive and fuel-system fallbacks for
hulls without an operational Jump drive.

The two collection phases have different encounter geography. A gas skimmer
remains spaceborne at the body locus and may be intercepted during skimming.
A wilderness water/ice collector is landed and cannot be attacked as though it
were hovering in local space; a named interceptor may wait at that body locus
and engage when the collector lifts for the return leg. Ports and downports
use the same attachment-versus-locus distinction.

## Phase Results

Successful terminal actions select the next authoritative state:

- initiating Jump enters `Jump`;
- completing docking or landing enters `Docked`;
- beginning an encounter enters `Encounter` with the continuation suspended;
- finishing a bounded local operation without an onward leg remains
  `Interplanetary` at that location; and
- continuing travel remains `Interplanetary` with the next checkpoint
  scheduled.

Thus `Interplanetary` is not an animation of empty travel and not a mandatory
prompt between every step. It is the persistent physical and planning context
between attachments, Jumps, and encounters.
