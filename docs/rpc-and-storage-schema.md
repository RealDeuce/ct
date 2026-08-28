# RPC and Storage Schema

*Current implementation: player CT-RPC 10, sysop/admin protocols 2, League Coordinator protocol 1, storage format 2, and independently versioned record codecs.*

## Authority boundary

The Cap'n Proto schema is the client/server contract. It carries commands,
observations, errors, phases, and unsolicited events; it does not expose LMDB
record layout or simulation transaction mechanics. The server alone owns rule
validation, scheduling, persistence, and recovery.

Every player command is classified in `server/src/wire.rs` as either:

- an **observation**, which is ordered and journaled but whose returned
  snapshot is not retained indefinitely; or
- a **transaction**, whose command ID and outcome are retained for exactly-once
  retry across reconnects; or
- an **operational request**, such as browser-alert enrollment, which is
  handled by a separate service and never enters authoritative game storage.

Closed enums and unions carry rule-bearing state. Player-facing prose may
explain state, but it is never the machine-readable discriminator.

## Current protocol

CT-RPC 10 is the only accepted player protocol. The sysop and administrator
protocols likewise accept only version 2. The distinct League Coordinator
endpoint accepts only CT-League version 1 and authenticates a numeric League
ID with that League's PSK. Each player, sysop, and administrator TLS connection begins with a
hello carrying one BCP 47 language tag. The server validates the tag and
returns the selected supported tag. Bare `en` selects the server's `en-US`
default; an explicit regional English tag such as `en-GB` remains selected.
The current clients request `en-US`, and English is the only installed server
language.

League Coordinator requests are deliberately narrower than administrator and
sysop requests: status/member listing, revision-checked league rename,
member-BBS enrollment, and revision-checked member enable/disable. Mutations
use stable 16-byte command IDs for exactly-once replay. The authenticated
League ID determines membership; no request may attach, remove, or transfer an
existing BBS.

The player `ServerHello` also carries the display-formatting profile belonging
to the selected language. It supplies decimal and grouping separators, primary
and secondary grouping sizes, and validated named-field patterns for absolute
game timestamps, game durations, and real durations. Clients apply this
profile only to typed numeric and temporal values. Identifiers and server prose
are never scanned for numbers or dates.

`ServerHello.accountJournalAvailable` advertises the additive account-journal
query. A CT-RPC 10 door connected to an earlier CT-RPC 10 server leaves the
Transaction Journal option hidden and continues to use the older finance
snapshot. It never guesses support from a shared protocol number or sends the
new union arm without the capability.

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
versions. CT-RPC 9 shipped in v0.7.13; current post-release development
therefore uses CT-RPC 10.

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

A generated offer remains a signed, physically delivered message instrument
inside the simulation, but only Task Management presents it to captains. Its
identifier names the persistent offer record and claiming updates that record
in the same authoritative transaction, so another captain cannot accept the
advertised work afterward. The existing Offer `MessageItem` union arm remains
reserved for CT-RPC 10 compatibility; normal Message Management and arrival
responses do not emit it.

Every message also carries an authoritative importance band (`Routine`,
`Notable`, `Important`, or `Headline`) and a persisted article body. Each
captain owns a server-side minimum band for every visible communication class.
Changing a filter is a queued transaction; the four-class filter set returns
with Message Management. The server applies it to both Message Management and
arrival review. Filtered messages are still retained, classified as received,
and allowed to update structured knowledge. The stored offer-class slot remains
for player-record codec compatibility but cannot be configured or presented.

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
- a typed stage and the current leg's due second; and
- fuel aboard plus the ship's effective total tank capacity.

The legacy nominal Jump-fuel allocation remains on the wire for older clients,
but it is not a live reserve and the current door does not display it.

`FinanceSnapshot` also carries current game time and typed pending-income
items with stage, payment, collateral release, estimate kind, and estimated
resolution second. `GetAccountLedger` is a paged observation filtered by
transaction class and vessel; entries contain typed account postings and
resulting balances rather than prose that a client must parse.

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
bankruptcy succession. `GetTerminalReport` is an observation and
`AcknowledgeTerminalReport` is an exactly-once transaction. Their structured
report carries a typed loss cause and captain fate, censored contact evidence,
standing-order and automation use, material and personnel consequences,
recovery eligibility, successor requirement, and the optional chronological
log. Recovery accepts only the matching acknowledged report revision. Crew
snapshots include current physical
characteristics, injury/fatigue, service availability, shore facility,
treatment timing, salary arrears, morale, and service revision. Task records
name their performing ship. Docked service quotations expose persisted
facility capability, including provision and ammunition availability.
Preview and commit are separate ordered calls; commit repeats the proposal and
its 16-byte preview hash so intervening state cannot silently change a warned
choice. Reliable checkpoint and encounter events are emitted on transition and
replayed from current state when a session opens. Automatic non-combat
subsystem damage instead creates a chronological engineering casualty report.
The oldest unacknowledged report is replayed at session opening, a best-effort
event wakes an already connected door, and acknowledgement is an exactly-once
transaction. The persisted queue, not the event socket, remains authoritative.

Encounter snapshots carry sensor resolution, apparent authority, coarse
threat, a capacity-limited cargo-demand breakdown, the exact legal posture and
fallback sets, and any response deadline. These fields are authoritative data,
not client reconstructions from prose. Combat orders add a signed speed change
and helm actor; combat participants add current speed and the visible pursuit
target/attack bonus. Task records carry authenticated piracy-loss evidence and
claim timing. All additions remain within CT-RPC version 8 because that version
was already reserved for the unreleased protocol and the changes are additive.

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

Interstellar travel is three distinct legs: port to departure locus, departure
locus through Jump to an arrival locus, and arrival locus to port. Frontier fueling is also
explicitly three legs: port to source body, work at the body, and return to
port. Each boundary is independently scheduled, so contact checks can attach
to the correct locus without inferring direction from a Boolean.

A Belt Cycle uses the same durable travel-event queue. Its outbound,
prospecting, survey, mining, refining, recovery, and egress purposes are typed
leg states. Global `resource-lodes` records own composition, grade, extent,
and depletion; `resource-observations` separately key private captain
knowledge. No record grants an exclusive claim. Ship records retain exact
power-fuel settlement time and fractional burn, while mined cargo lots retain
source body and lode IDs. Ship codec version 4 adds distinct conventional and
private arrival loci; it reads versions 1 through 3 with zero
cargo provenance, a no-retroactive-burn timestamp sentinel, and legacy frontier
work interpreted as refined with no recorded processing Effect. Version 3
stores explicit collection output, processing Effect/damage, and standalone
processing work.

`FlightPlanSnapshot` is stored separately from the ship's current physical
leg. It owns ordered typed waypoints, bounded actions, hold/through checkpoint
authority, a separate last-step terminal marker, encounter policy, revision,
current step, state, and suspension reason. An active redirected in-system leg
also owns a leg-identity-keyed trajectory record containing its epoch,
position, velocity, turnover, and bounded acceleration vectors. Redirecting the
active normal-space destination cancels superseded travel/contact work and
writes the new leg, trajectory, and plan atomically. Revising only later items
or policy instead retags and preserves the active physical leg, trajectory, or
timed ship activity without changing its due time. A Jump-space emergence leg
is immutable until its checkpoint, but its later plan remains editable. Cargo
and sealed mail remain on the ship and generate preview warnings; replanning
never edits obligations.
Flight Plan codec version 6 adds the remote-arrival choice and distinct
departure/arrival locus role. Version 5 added typed per-encounter standing orders and the
proposal flag that preserves an unchanged active operation while future items
are revised. Version 4 added the per-collection refine choice and standalone
refine action; readers retain versions 1 through 5. Authoritative
preview includes per-step normal and failed fuel-operation timings. Retained
outcome codec version 27 preserves account-ledger observation replay; version
26 preserves the new locus and maneuver stage in addition
to the version-25 timings, quotation, and
activity metadata, terminal reports, BBS/League affiliation, effective fuel
capacity, engineering-casualty acknowledgement results, and a contact's
declared class separately from its sensor classification for idempotent replay.
Version 24 outcomes remain readable but cannot contain the new policy-default
outcome kind.
Version 23 outcomes remain readable with an empty declared class. Version 21
outcomes decode with an unknown
zero fuel capacity, matching the field's absence before CT-RPC 9.

Arrival does not directly rewrite an approaching ship as docked. It creates a
durable checkpoint at the exact port locus. Hold authority waits for an
acknowledgement; Through authority uses standing policy. The terminal bit only
ends the plan after the step. Contact candidates
come from the deterministic traffic window, while actual background carriers
remain persisted in the simulator. Encounter turns are their own scheduled
engine inputs and use the same ingress sequence as every other mutation.

`FlightPlanStep.terminal`, `FlightPlanWarning.stepIndices`, and the terminal
report requests and response are additive CT-RPC fields. The flight-plan
proposal and snapshot record codecs use version 6; versions 1 through 5 remain
readable, and version-1 Terminal authority decodes
as Hold plus a terminal marker. Encounter-record codec version 5 stores the
contact's declared class separately from its sensor classification, in addition
to version-4 Through/automation use and the optional acknowledged
terminal-report snapshot, while retaining readers for versions 1 through 4.
Version-4 and older records decode with an empty declared class. Older terminal
records derive the same report from retained encounter, combat, ship, and
personnel state when first reviewed. Outcome codec version 25 preserves
terminal-report and engineering-casualty acknowledgement replay; version 23
and older outcomes decode absent additive fields as empty or zero. A
pre-field proposal with no explicit terminal bit is normalized at the wire
boundary by marking its last step. CT-RPC 8 added durable ship commissions and
construction activity. CT-RPC 9 adds browser-alert enrollment and effective
fuel capacity. CT-RPC 10 adds durable operational-damage reports, positive
acknowledgement, live-session wake events, and authoritative game-to-real clock
rates on ship status so scheduled yard work can display its wall-time
equivalent. It also adds `EncounterContact.declaredClassName`, keeping a
contact's transmitted registered-class claim distinct from the existing
sensor-derived `className` and confidence. Older servers leave additive clock
and declared-class fields at zero or empty.

Current CT-RPC 10 additionally exposes a ship-revisioned encounter-policy
default, typed per-encounter ordinary/Fight-threshold rules, an explicit
authorization bit when a default can attack non-hostile traffic, the
active-operation preservation flag on flight-plan proposals, and the
sensor-limited combat-outlook percentage on encounter snapshots, distinct
departure/arrival locus semantics, remote- and departure-locus-arrival flags, and a typed local
maneuver stage. Ship defaults
live in the `encounter-policy-defaults` database as codec version 1; absence
means revision zero and the conservative legacy defaults. These are additive
post-v0.7.13 fields, so the player protocol remains 10 until release.

## Persistence contract

The server accepts storage manifest format 2 and the current version of each
record codec, plus only the explicit legacy readers documented for that codec.
It does not perform an implicit universe-wide migration. Opening any
unsupported manifest or record version fails with an instruction to
reinitialize the game store. In particular, v0.7.11 and v0.7.12 have no migration from the
v0.7.10 format-1 database; operators must preserve any desired backup and
initialize a fresh store.

Record versions remain explicit corruption guards and make byte-level audit
straightforward; they are not promises of backward readability. Release notes
define the backup, migration or reinitialization, rollback, and mixed-version
requirements for each shipped compatibility boundary.

Scheduled indexes are written atomically with the authoritative object that
requires them. Startup does not reconstruct missing simulation systems,
maintenance events, training events, or activity events. A missing counterpart
is corruption, not an upgrade opportunity.

Storage format 2 contains the current Task, finite offer, work-assignment, carriage,
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
treatment results, and persistent facility capability overlays. The
independently versioned `operational-damage-reports` records form a
per-captain chronological queue containing the automatic cause, ship,
subsystem hit delta and resulting condition, and Jump route/timing details
when applicable. Reports remain until positive acknowledgement. Adding this
database does not change storage manifest format 2 and does not require a
v0.7.13 universe to be reinitialized; opening an existing universe creates the
empty database and initializes its independent report-ID allocator. These are
direct current formats with no legacy decoder.

The additive `account-ledger` database stores version-1 entries under a
captain identity plus reverse entry ID, giving indefinite newest-first paging.
Opening an existing format-2 universe performs one idempotent reconciliation:
each existing captain receives a single carried-forward entry for current
liquid, per-vessel restricted and principal, and per-vessel reserved balances.
New captains receive their opening entry in the creation transaction. Later
postings are written in the same LMDB transaction as the balance mutation.
The reconciliation has its own metadata version and does not change storage
manifest format 2.

Field-alpha control state includes active/suspended/removed player-access
tombstones, signed sysop directives, an unspendable polity fiscal ledger,
per-system effective polity-policy revisions, and message-linked policy
directives. The generated UWP remains immutable; effective law and encounter
orientation are mutable institutional overlays that advance independently in
each member system when its physical policy mail arrives.

## Transaction recovery

Player commands and admitted scheduled work use one durable engine-input queue
and one monotonically increasing queue sequence. Clock advancement is not an
engine input. Future-event indexes determine eligibility but never execution
priority. While ingress is empty, the scheduler advances logical time directly
in a journalled transaction and leaves future work indexed. A following
transaction moves the now-due timestamp-free payload from its index into
ingress, separately from execution, so both the pre-admission schedule and the
post-admission queued state are crash-recoverable. Authoritative mutation,
journal append, result retention where applicable, outbox publication, and
removal from ingress then share the execution transaction. Rule rejection is a
successful transaction with an error outcome. An unexpected transactional
failure is fatal to queue processing; the server does not roll one input back
and continue with later inputs.

Observation journals retain the encoded request and an explicit marker that
no snapshot follows. Transaction journals retain the encoded request and its
outcome. Queue, journal, result, and outbox codecs each accept only their
current record version.
