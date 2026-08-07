# RPC and Storage Schema

*Current implementation: player CT-RPC 2, storage format 1, and record codecs 1.*

## Authority boundary

The Cap'n Proto schema is the client/server contract. It carries commands,
observations, errors, phases, and unsolicited events; it does not expose LMDB
record layout or simulation transaction mechanics. The server alone owns rule
validation, scheduling, persistence, and recovery.

Every player command is classified in `server/src/wire.rs` as either:

- an **observation**, which is ordered and journaled but whose returned
  snapshot is not retained indefinitely; or
- a **transaction**, whose command ID and outcome are retained for exactly-once
  retry across reconnects.

Closed enums and unions carry rule-bearing state. Player-facing prose may
explain state, but it is never the machine-readable discriminator.

## Current protocol

CT-RPC 2 is the only accepted player protocol. Both builds generate bindings
from `protocol/ct_rpc.capnp`; there is no compatibility reader for older wire
shapes. `InitialCrewDraft` contains only the authoritative slot ID, name, and
training skill. Captain options contain the current point-buy and skill-pool
model without retired fields.

An Offer `MessageItem` carries the rendered signed instrument, its offer
identifier, and whether the finite instrument remains claimable. That
identifier names one persistent server record used by Message Management, the
arrival packet, and the Task manager. Claiming updates the record in the same
authoritative transaction, so another captain cannot accept the advertised
work afterward.

Every message also carries an authoritative importance band (`Routine`,
`Notable`, `Important`, or `Headline`) and a persisted article body. Each
captain owns a server-side minimum band for every service class. Changing a
filter is a queued transaction; the complete filter set returns with Message
Management. Arrival filtering changes presentation only: filtered messages
are still retained, classified as received, and allowed to update structured
knowledge.

Every message may also carry one closed typed action reference: claim the
attached finite offer, review a Task, review Operations, review Finance, or
review Mapping. The reference opens a fresh authoritative query or command;
article prose is never parsed as rule state.

Explicit task terms also carry a performance count and recurrence interval.
The accepted task records completed performances, advances each subsequent
deadline through the ordinary scheduled-input queue, and retains collateral
until the last performance settles or the obligation defaults.

`TravelStatus` carries a typed current leg:

- stable `planId`, `planRevision`, and `legIndex` values;
- typed origin and destination loci (port, Jump locus, or celestial body);
- a typed stage and the current leg's due second.

A docked ship has plan/revision/index zero and identical port loci. A ship may
also hold at an exact deep-space coordinate after exploration or a misjump.

The current protocol includes flight-plan preview/commit, typed loci and
coordinate Jumps, onboard versus commercial-tape navigation, explicit
known-bad-plot authority, arrival checkpoints, typed encounters, Task ledgers,
carriage declarations, finance, dated market knowledge, finite ship and crew
markets, combat and career operations, and docked-service quotation/commit.
It also carries finite market leads and reservation state, named market
events, ordinary-carriage manifest previews, typed Task actions and dispute
mail linkage, private-message composition, insurance state, and versioned
starting title/refit terms. It also exposes a revisioned multi-vessel fleet,
physical store transfers, career transitions and legal instruments, named
combat actors and joint orders, command-loss recovery, and irrecoverable
bankruptcy succession. Crew snapshots include current physical
characteristics, injury/fatigue, service availability, shore facility,
treatment timing, salary arrears, morale, and service revision. Task records
name their performing ship. Docked service quotations expose persisted
facility capability, including provision and ammunition availability.
Preview and commit are separate ordered calls; commit repeats the proposal and
its 16-byte preview hash so intervening state cannot silently change a warned
choice. Reliable checkpoint and encounter events are emitted on transition and
replayed from current state when a session opens.

Docked-service quotations return the ship revision, exact named fuel sources,
physical provision and ammunition state, repair/replacement choices, facility
reasons, prices, and elapsed times. One commit presents that revision and a
closed union selecting exactly one order. Ship status includes physical stores,
component identity, replacement basis, installation generation, and manifested
symptoms; it never exposes a latent quirk.

## Authoritative flight state

The ship record stores either a docked location or a complete active
`FlightLegRecord`. A leg owns its plan identity and revision, sequence index,
typed endpoints, start/due seconds, and typed purpose.

Interstellar travel is three distinct legs: port to departure locus, Jump
locus to Jump locus, and arrival locus to port. Frontier fueling is also
explicitly three legs: port to source body, work at the body, and return to
port. Each boundary is independently scheduled, so contact checks can attach
to the correct locus without inferring direction from a Boolean.

`FlightPlanSnapshot` is stored separately from the ship's current physical
leg. It owns ordered typed waypoints, bounded actions, hold/terminal/through
authority, encounter policy, revision, current step, state, and suspension
reason. An outbound revision may replace the not-yet-started Jump destination
without replacing the port-to-locus leg. Once the Jump begins, that physical
leg is immutable until its next checkpoint. Cargo and sealed mail remain on
the ship and generate preview warnings; replanning never edits obligations.

Arrival does not directly rewrite an approaching ship as docked. It creates a
durable checkpoint at the exact port locus. Terminal authority waits for an
acknowledgement; through authority uses standing policy. Contact candidates
come from the deterministic traffic window, while actual background carriers
remain persisted in the simulator. Encounter turns are their own scheduled
engine inputs and use the same ingress sequence as every other mutation.

## Persistence contract during development

The repository has not been deployed. The server therefore accepts exactly
storage format 1 and exactly version 1 of every record codec. It
does not migrate, synthesize, or reinterpret earlier development formats.
Opening any other manifest version fails with an instruction to reinitialize
the game store.

Record versions remain explicit corruption guards and make byte-level audit
straightforward; they are not promises of backward readability. When the game
is first deployed, a release policy must define backup, migration, rollback,
and compatibility guarantees before an incompatible format is shipped.

Scheduled indexes are written atomically with the authoritative object that
requires them. Startup does not reconstruct missing simulation systems,
maintenance events, training events, or activity events. A missing counterpart
is corruption, not an upgrade opportunity.

Storage format 1 contains the current Task, finite offer, work-assignment, carriage,
finance, market, travel, encounter, combat, combat-career, prize, warrant, and
post-combat-recovery records and indexes. It also contains exact deep-space
coordinates, per-system generation seeds, materialized-coverage cells, and
discovery claims. Message records contain immutable
subject, body, class, and importance; player records contain the durable
per-class arrival thresholds. Market consumption is keyed by
actual commodity ID rather than positional array index. Ship records carry
titled cargo, catalog-capacity ammunition, physical provisions, per-component
construction data and warranty, latent quirks, a completed-service ledger,
salaried crew service, combat policy, career state, and recovery state. It
also contains passenger manifests, physical shore assignments, deferred
treatment results, and persistent facility capability overlays. These are
direct current formats with no legacy decoder.

Field-alpha control state includes active/suspended/removed player-access
tombstones, signed sysop directives, an unspendable polity fiscal ledger,
per-system effective polity-policy revisions, and message-linked policy
directives. The generated UWP remains immutable; effective law and encounter
orientation are mutable institutional overlays that advance independently in
each member system when its physical policy mail arrives.

## Transaction recovery

Player commands, admitted scheduled work, and clock advances use one durable
engine-input queue and one monotonically increasing queue sequence. Future
event indexes determine eligibility but never execution priority. A clock
pulse itself is not stored; it produces scheduled-work admissions or, when no
event remains due, an explicit logical-time-advance input. Moving a future
event from its index into ingress commits separately from execution, so the
queued state is crash-recoverable. Authoritative mutation, journal append,
result retention where applicable, outbox publication, and removal from
ingress then share the execution transaction. Rule rejection is a successful
transaction with an error outcome. An unexpected transactional failure is
fatal to queue processing; the server does not roll one input back and
continue with later inputs.

Observation journals retain the encoded request and an explicit marker that
no snapshot follows. Transaction journals retain the encoded request and its
outcome. Queue, journal, result, and outbox codecs each accept only their
current record version.
