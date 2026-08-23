# RPC and Storage Schema

*Current implementation: player CT-RPC 8, sysop/admin protocols 2, storage format 1, and record codecs 1.*

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

CT-RPC 8 is the only accepted player protocol. The sysop and administrator
protocols likewise accept only version 2. Each TLS connection begins with a
hello carrying one BCP 47 language tag. The server validates the tag and
returns the selected supported tag. Bare `en` selects the server's `en-US`
default; an explicit regional English tag such as `en-GB` remains selected.
The current clients request `en-US`, and English is the only installed server
language.

The player `ServerHello` also carries the display-formatting profile belonging
to the selected language. It supplies decimal and grouping separators, primary
and secondary grouping sizes, and validated named-field patterns for absolute
game timestamps, game durations, and real durations. Clients apply this
profile only to typed numeric and temporal values. Identifiers and server prose
are never scanned for numbers or dates.

Unsupported versions receive a reason-only close encoded in the obsolete
protocol's wire format. The reason says that the Cepheus Trader client must be
upgraded and reports both the rejected and required versions. This
compatibility path exists only to report the required upgrade and cannot
establish a reduced or degraded session. Current protocols use typed close
codes, localized detail, and a supported-language list for negotiation failure.
Established connections never interpret legacy envelopes.

During one release-development cycle, incompatible schema work shares the next
protocol number. The number advances again only after that contract has shipped
in a release; intermediate feature commits do not consume additional protocol
versions.

Both builds generate bindings from `protocol/ct_rpc.capnp`.
`InitialCrewDraft` contains only the authoritative slot ID, name, and
training skill. Captain options contain the current point-buy and skill-pool
model without retired fields.

Crew roles and locations, system-knowledge sources, and combat-actor
eligibility have closed typed fields. Their accompanying labels remain display
text and must not be parsed by clients. Server connection and protocol-control
messages use the embedded Fluent English bundle. Persisted articles and
simulation narrative remain English until they can be stored as semantic
message identifiers and typed arguments, then rendered for each connection.

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

Traffic contacts carry a closed attachment value (`spaceborne`, `berthed`, or
`landed`) separately from their locus. Combat-career snapshots optionally
carry a typed interception watch with its filter and locus. A direct intercept
response is a combat snapshot when engagement begins, an encounter result when
a boarding demand resolves without combat, or a combat-career snapshot when
selecting an attached player vessel establishes a named departure watch. Both
direct intercepts and persisted watches carry the closed purpose value
`armedAttack` or `boardingInspection`. The revisioned interception-watch
command selects cancel, all craft, or an exact catalog ID; clients do not infer
any of these states from explanatory text. Stored watch records and retained
transaction outcomes carry independent format revisions so a restart cannot
reinterpret a previously committed purpose.

Docked-service quotations return the ship revision, exact named fuel sources,
their body type, port-sale or routine-wilderness access class, whether onboard
refining is possible, tank-room maximum, physical provision and ammunition
state, repair/replacement choices, facility reasons, prices, and elapsed times.
One commit presents that revision and a
closed union selecting exactly one order. Its retained receipt contains the
resulting ship status and a closed detail union. A port-fuel receipt additionally
records the fuel kind, quantity, tank state, price, exact restricted-versus-liquid
payment allocation, and both resulting balances. A provision receipt records the
monthly-package count, person-days loaded, resulting stores and capacity, price,
the same exact payment allocation, and both resulting balances. A retry therefore
never reconstructs an accounting result from later observations. Ship status
includes physical stores, component identity, replacement basis, installation
generation, and manifested symptoms; it never exposes a latent quirk.

## Authoritative flight state

The ship record stores either a docked location or a complete active
`FlightLegRecord`. A leg owns its plan identity and revision, sequence index,
typed endpoints, start/due seconds, and typed purpose.

Interstellar travel is three distinct legs: port to departure locus, Jump
locus to Jump locus, and arrival locus to port. Frontier fueling is also
explicitly three legs: port to source body, work at the body, and return to
port. Each boundary is independently scheduled, so contact checks can attach
to the correct locus without inferring direction from a Boolean.

A Belt Cycle uses the same durable travel-event queue. Its outbound,
prospecting, survey, mining, refining, recovery, and egress purposes are typed
leg states. Global `resource-lodes` records own composition, grade, extent,
and depletion; `resource-observations` separately key private captain
knowledge. No record grants an exclusive claim. Ship records retain exact
power-fuel settlement time and fractional burn, while mined cargo lots retain
source body and lode IDs. Ship codec version 3 reads versions 1 and 2 with zero
cargo provenance, a no-retroactive-burn timestamp sentinel, and legacy frontier
work interpreted as refined with no recorded processing Effect. Version 3
stores explicit collection output, processing Effect/damage, and standalone
processing work.

`FlightPlanSnapshot` is stored separately from the ship's current physical
leg. It owns ordered typed waypoints, bounded actions, hold/through checkpoint
authority, a separate last-step terminal marker, encounter policy, revision, current step, state, and suspension
reason. An outbound revision may replace the not-yet-started Jump destination
without replacing the port-to-locus leg. Once the Jump begins, that physical
leg is immutable until its next checkpoint. Cargo and sealed mail remain on
the ship and generate preview warnings; replanning never edits obligations.
Flight Plan codec version 4 adds the per-collection refine choice and the
standalone refine action; readers retain versions 1 through 3. Authoritative
preview includes per-step normal and failed fuel-operation timings. Retained
outcome codec version 18 preserves those timings and the added quotation and
activity metadata for idempotent replay.

Arrival does not directly rewrite an approaching ship as docked. It creates a
durable checkpoint at the exact port locus. Hold authority waits for an
acknowledgement; Through authority uses standing policy. The terminal bit only
ends the plan after the step. Contact candidates
come from the deterministic traffic window, while actual background carriers
remain persisted in the simulator. Encounter turns are their own scheduled
engine inputs and use the same ingress sequence as every other mutation.

`FlightPlanStep.terminal` and `FlightPlanWarning.stepIndices` are additive
CT-RPC fields. The flight-plan proposal and snapshot record codecs use version
3; versions 1 and 2 remain readable, and version-1 Terminal authority decodes
as Hold plus a terminal marker. Outcome codec version 16 preserves warning
step references, catalogued belts, cargo provenance, and the capable-yard
commission catalog; older outcomes decode absent additive fields as empty or
zero. A
pre-field proposal with no explicit terminal bit is normalized at the wire
boundary by marking its last step. CT-RPC 8 adds durable ship commissions and
construction activity. No universe-wide storage migration is required.

## Persistence contract during development

The repository has not been deployed. The server accepts storage manifest
format 1 and the current version of each record codec, plus only the explicit
legacy readers documented for that codec. It does not perform an implicit
universe-wide migration. Opening any unsupported manifest or record version
fails with an instruction to reinitialize the game store.

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
