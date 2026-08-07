# Cepheus Trader

Cepheus Trader is a planned BBS door game in the spirit of *TradeWars 2002*
and *Yankee Trader*. It is a trading, economy, and space-combat simulator
based on the space-trading rules of the Cepheus Engine, adapted for a
playable terminal game.

“Cepheus Engine” and “Samardan Press” are trademarks of Jason “Flynn” Kemp.
Cepheus Trader is not affiliated with Jason “Flynn” Kemp or Samardan Press.
Cepheus Trader is an Alternate Cepheus Engine Universe.

The project is in early design. The local Cepheus Engine rules reference is
the generated Markdown in [`cepodnew-markdown/`](cepodnew-markdown/index.md).
The living guidance for implementation and for LLM-assisted development is
[`LLM_INSTRUCTIONS.md`](LLM_INSTRUCTIONS.md). Development order, current
focus, and milestone acceptance boundaries are maintained in
[`ROADMAP.md`](ROADMAP.md).

Original software is available under the MIT License. Original names,
rule-bearing content, catalog data, and other game content are Open Game
Content under OGL 1.0a. Cepheus Trader reserves no original Product Identity;
upstream Product Identity and third-party software remain separate. See
[`LICENSE.md`](LICENSE.md),
[`OPEN_GAME_LICENSE.md`](OPEN_GAME_LICENSE.md), and
[`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md). Licensing records are
living release data and must be updated whenever a rules source or dependency
changes. The broader audit of available Cepheus Engine, Clement Sector, and
Earth Sector material is in
[`docs/potential-ogc-sources.md`](docs/potential-ogc-sources.md).

The engine is a single authoritative Rust server process. A separate C++17 or
newer user-interface process will connect to it over a TLS-PSK-protected
protocol; the UI is a client of the engine rather than a second implementation
of game rules. CT-RPC uses Cap'n Proto serialization with a small
project-specific, bidirectional RPC envelope over TLS-PSK. The Rust server
uses GnuTLS 3.x; the OpenDoors-capable C++17 client uses Botan 3. The common
product version is **0.7.0**. The client is
C++ for practical BBS, IPC, and serial-port support.

The repository contains separate projects:

- [`protocol/`](protocol/) — shared Cap'n Proto CT-RPC schema;
- [`server/`](server/) — authoritative Rust server;
- [`client/`](client/) — Botan 3 C++ clients, including the OpenDoors player
  door, headless protocol exerciser, and operator management utilities;
- [`catalog/`](catalog/) — rule-derived Open Game Content shipbuilding data,
  hand-authored design bills of materials, and the master attribution
  registry;
- [`docs/`](docs/) — cross-project design notes;
- [`cepodnew-markdown/`](cepodnew-markdown/) — local CE rules reference.

The initial-universe reset, BBS enrollment, BBS-authenticated sysop
configuration, OpenDoors local mode, and the first complete player-creation
path are implemented.
The merchant implementation includes all generic CE/Bounded Fortune trade
goods, persistent finite daily stock, legality and tariffs, titled cargo lots,
ordinary freight/passenger/mail declarations, task records, scheduled
supplier/buyer research, dated market knowledge, secured finance, and finite
ship and crew exchanges. The 2026-08-02 playability audit has been closed
through Milestone 6. The real TLS/OpenDoors scenario exercises the merchant
voyage, physical mail, all-profile arrival review, dock services, combat-career
entry, generated traffic, named combat actors, and joint orders. Deterministic
fixtures carry the longer merchant, naval, privateer, pirate, legal,
personnel, facility, and succession outcomes across restart. The OpenDoors client
exposes the implemented operations in all three scrolling terminal profiles. While the
server is running, one authoritative live clock advances at exactly four game
weeks per real day; downtime is frozen. Phase changes and seed-derived local
traffic observations arrive asynchronously without coupling connection
receive work to transmit or UI work.
Ship operation also has durable condition ledgers, monthly maintenance,
physical life-support provisions and ammunition, berth fees, hidden age/use
quirks and warranty service, weekly crew training, per-subsystem proper
repair, weeks-long refits, catalog-priced component replacement, port
unrefined fuel, and timed named-body wilderness or gas-giant fueling. These
share the ordered scheduler and survive restart. Latent quirks remain hidden;
reported symptoms appear only after relevant use manifests them.
The reset creates the 35 stellar-component systems from Sol through Tau Ceti
as Federation space and Earth at TL13; its full Jump-2 audit is in
[`docs/initial-federation.md`](docs/initial-federation.md). The authoritative
implementation order is [`ROADMAP.md`](ROADMAP.md).

The standalone build, repository checks, portable client packaging, manual
benchmarks, and tagged multi-platform releases are automated with GitHub
Actions under [`.github/workflows/`](.github/workflows/). Release procedure
and runner boundaries are documented in
[`docs/release-process.md`](docs/release-process.md).

Both the Rust and C++ builds generate bindings from the schema in
[`protocol/`](protocol/). A player is identified by the tuple `(BBS ID, local
player ID)`, with each component represented as a `UInt32`. The server assigns
the BBS ID; its canonical decimal representation is the TLS external-PSK
identity, so the authenticated BBS attests only its local player component.
Observation/transaction classification, exactly-once replay, durable journal,
storage-version, and aggregate-ownership rules are specified in
[`docs/rpc-and-storage-schema.md`](docs/rpc-and-storage-schema.md).

If that compound identity has no stored player, `ServerHello` reports the
`newUser` phase. The implemented protocol supplies revision-tagged captain
pools, the three BBS-cell starter offers, catalog ship descriptions, and the
resulting crew plan. The OpenDoors door walks through captain
characteristics and skills, offer selection, ship naming, an editable
role/name roster of officers and senior specialists, and final confirmation.
Each roster entry has a role-specific default callsign, so Enter accepts even
a large starting crew without forcing the player to name every entry.
`CreatePlayer` revalidates the complete proposal, atomically instantiates
player, person, catalog-backed ship, and crew-service records at the BBS
home, and transitions to `docked`; a protocol `Close` is always valid. The
record design is specified in
[`docs/player-creation-records.md`](docs/player-creation-records.md).

The server also exposes a separate loopback-only TLS-PSK management listener.
It creates a protected `admin.psk` beside its database on first startup.
Authenticated operator commands use a separate schema but enter the same
authoritative engine mailbox. Implemented commands add a BBS, perform an
explicitly confirmed destructive initial-universe reset, report operational
status, and create labeled live same-version backups.

A third TLS listener serves the BBS sysop protocol. It authenticates against
the enrolled BBS ID/PSK, derives authority from the selected canonical BBS ID,
and supports reading and revision-checked updates of the BBS/polity names and
trade-combat and chaos-order orientations. It also manages local account
identity mappings, immediate active/suspended/removed access, and mail-delayed
tax or naval-demotion instruments. It does not use a player identity or the
global `admin.psk`. See
[`docs/field-alpha-operations.md`](docs/field-alpha-operations.md).

The game universe is a true 3D map of stellar-component systems, anchored to
Earth's location in the Milky Way. Each stellar component is a separate game
system because it may have its own planetary system. Coordinates use the
galactic directions
coreward/rimward, spinward/trailing, and galactic north/south. Each system
contains a CE-style planetary system rather than being only a point on a
flat game board. The game will not use CE's sector, subsector, or hex-grid
star-mapping model; those conventions do not define navigation or system
placement here.

Parsecs are the base universe distance unit. Systems use CE's Universal World
Profile and related system fields (trade codes, bases, allegiance, belts, and
gas giants), with CE starport classes and services. The CE hex-map location is
not used for navigation. The universe is procedurally generated once as
systems are materialized and then stored; revisits use the persisted source
data while mutable markets and events remain separate.

The versioned stellar-density function and repeated logarithmic-arm geometry
are defined in
[`docs/stellar-distribution.md`](docs/stellar-distribution.md). It couples
coreward and spinward position through Galactic radius and curved arm phase,
models each arm's changing direction, and applies a separate north/south disk
falloff. The persistent quarter-parsec coverage lattice, six-parsec Jump
arrival oracle, and versioned materialization bitmaps are specified in
[`docs/observed-volume.md`](docs/observed-volume.md).
BBS polity cluster conditioning and gateway topology are specified in
[`docs/bbs-polity-generation.md`](docs/bbs-polity-generation.md).
The Federation must be initialized before a BBS can be enrolled. The first
successful sysop configuration then atomically materializes that BBS's closest
eligible home cluster and registers it with the live simulator;
reinitialization rematerializes all preserved configurations after creating
the Federation. Full immutable
stellar, orbital, planetary, moon, and physical baselines are derived from
each system seed as specified in
[`docs/celestial-system-generation.md`](docs/celestial-system-generation.md).
The implementation adapts the explicitly OGC method on pages 44–149 of
*Unmerciful Frontier: The CCA Sourcebook*: core CE wins wherever procedures
overlap, and hex occupancy and Zimm-specific steps do not enter this game.
First-arrival surveys, densitometer interpretation, the historical population
envelope, empty systems, the Federation's Earth-adjudicated first-discovery
award for settled systems, and future player colonies are specified in
[`docs/settlement-and-system-survey.md`](docs/settlement-and-system-survey.md).

The CE rules directly cover task resolution, skills, ship construction,
crew/ship operations, combat damage, speculative trade, freight, passengers,
mail, charters, fuel, and port services. Merchant progress is primarily wealth
and assets; naval progress is rank, authority, and ship class. CE's ownership,
crew, maintenance, repair, and new-character paths are the baseline, while
banking and macroeconomic effects are game-level additions.

Interstellar travel uses the standard CE Jump drive. Alternative drives such
as warp, teleport, and hyperspace are outside the current design. Standard
Jump drives may use deliberate empty-space staging: a double-tanked Jump-1
ship can make two Jump-1 legs with a required one-day midpoint turnaround.
Paired course tapes, reliability, timing, and the Jump-1-versus-Jump-2
economic trade-off are specified in
[`docs/interstellar-jump-operations.md`](docs/interstellar-jump-operations.md).

There is no ansible. Electronic FTL communication is data physically carried
between systems aboard ordinary ships. Standard beacons at maintained Jump
loci offer small route-dependent stipends, transfer destination mailbags to
departing ships, accept them automatically on arrival, and issue local payment.
Busy polity routes obtain daily or better service from existing traffic;
sparse routes offer larger subsidies or, only when necessary, charter a
scheduled courier. Physical letters and parcels remain ordinary cargo.

The engine tracks mail time between locations. News is timestamped and
arrives late; the older it is when delivered, the more significant it must be
to become a local headline.

Simulation time advances whenever the server is running at exactly 28 game
seconds per real second, or four game weeks per real day; downtime is frozen.
Jump time is the overall universe baseline. Finer local frames cover
interplanetary in-system activity and encounter
activity such as ports, trading, and combat. Encounter participants are
isolated from unrelated universe activity until the encounter resolves, then
receive a relevant news update before returning to their prior frame.
System traffic is concentrated at Jump loci, gas giants, inhabited worlds,
and specific local destinations rather than uniformly distributed through
space. The traffic-field and contact/intercept rules are specified in
[`docs/system-traffic-and-encounters.md`](docs/system-traffic-and-encounters.md).
That document also defines the initial population/TL/trade cargo calibration,
route-level sparse schedules, persistent cargo lots, and authoritative daily
system jobs. Every materialized system advances once per game day through the
scheduled transaction processor; uneventful player visits still do not consume
cargo or move mail, while observations and consequential actions become
authoritative state.

The OpenDoors player interface has three page-oriented output profiles: ISO 646
plain text, ISO 646 with ECMA-48 colour, and CP437 with ECMA-48 colour. There
is no cursor-addressed TUI: plain pages use form feed, and enhanced pages use
clear-and-home before emitting wrapped lines. Every action must remain usable
at 40×24, with 80×24 as the normal target. Presentation profiles, responsive
wrapping, OpenDoors boundaries, and required tests are specified in
[`docs/door-presentation.md`](docs/door-presentation.md).

Each BBS installation represents a home system in a small polity of roughly
ten systems. The polity's highest-Tech-Level planet is in the home system.
When creating a BBS, the sysop chooses a local civilization profile ranging
from combat-focused (naval life and danger) to trade-focused (rare, distant
banditry). The wider universe remains connected, so players can cross the
entire profile spectrum, even if doing so takes many months of game time
rather than months of real-world connection time.

The sysop is the final moderator for the local polity and players originating
on that BBS. They may alter polity settings or demote, tax, suspend, and remove
players, but may not grant players credits, ships, cargo, favorable outcomes,
or other in-game advantages.

The intended pacing is 15–30 minutes of useful play per day, with a soft
45-minute ceiling. A single 30-minute session should support successful play;
longer play or multiple 5–15 minute sessions may provide only a modest bonus.

The game is not a full RPG, but relevant character and crew skills remain
operational factors in combat, trading, navigation, engineering, piloting,
and negotiation.
Post-creation skill improvement uses the core CE elapsed-game-week rule as
its baseline. The adopted formula, examples, rejected Clement point economy,
and intentionally deferred scheduling and pacing decisions are recorded in
[`docs/skill-training.md`](docs/skill-training.md). Initial target selection,
persistent progress, weekly calendar accrual, and course completion are
implemented.

The player-facing command console and the six phase-independent manager
boundaries are specified in
[`docs/universal-managers.md`](docs/universal-managers.md): crew, ship, task,
message, Known Universe, and Operations Ledger management remain reachable in
every operational phase while their individual mutations stay
phase-constrained. The Known Universe `K` interface is a paged navigation
library: it can limit charts to direct-jump destinations and opens an in-world
system dossier with range, port, population, tech level, age, and source.

Every newly discovered system enters the discovering ship's Known Universe
repository. On arrival the captain may broadcast a free public mapping
package, send a paid encrypted direct bounty filing to Earth, withhold it, or
withhold it and add the system to an editable captain-private Secret Systems
list. A public package propagates from the ship; a winning direct filing makes
Earth originate the public package when it awards the bounty. Routine notices
are hidden by default message filters; filtering a public notice never blocks
its structured knowledge update. Electronic-mail tariffs, fixed-system and
mobile addressing, encryption, capturable private credentials, and delayed
revocation are specified in
[`docs/mail-service-and-security.md`](docs/mail-service-and-security.md).

Crew Management is the first implemented vertical slice. It exposes the
committed roster and training progress and persists zero-or-more current watch
roles independently of each person's service appointment. Empty assignments
mean off watch; multiple roles support CE duty doubling; Pilot remains
exclusive. Natural-healing accrual still awaits the health model and scheduled
daily processing.

Ship Status is the second implemented manager slice. Materialized ships have
stable per-subsystem damage and maintenance records. Sustained physical damage
is separate from temporary battlefield-repair coverage, and scheduled service
is separate from both. The manager exposes proper-repair and refit work in
addition to status; transition rules and remaining work are documented in
[`docs/ship-condition-and-maintenance.md`](docs/ship-condition-and-maintenance.md).

The implemented docked phase menu and docked-to-interplanetary contract are
recorded in
[`docs/docked-operations.md`](docs/docked-operations.md). Interplanetary
movement uses committed continuation plans: encounter and readiness
checkpoints always run, but uneventful preauthorized steps such as docking,
skimming, or initiating Jump do not create mandatory UI stops. Those semantics
are specified in
[`docs/interplanetary-operations.md`](docs/interplanetary-operations.md).
Delayed communications remain separate from the subject-oriented, physically
synchronized player knowledge model in
[`docs/known-universe.md`](docs/known-universe.md).

Generated systems and principal worlds use the versioned, hand-maintained
profiles in [`catalog/place-names.toml`](catalog/place-names.toml). Polities and
unaligned regions receive coherent naming profiles, while interstellar system
names are collision-checked across the complete materialized universe. The
rules and compatibility boundary are in
[`docs/place-naming.md`](docs/place-naming.md).

The first authoritative background simulation slice now schedules one daily
job per system and persists ordinary traffic ships physically carrying sealed
mailbags over exact J-2 route legs. Immutable messages fan out through delivery
envelopes, beacon queues, custody records, arrival delivery, and expiry. The
non-interactive tour audits recovery and custody and reports message volume,
thread CPU per system, and whole-universe progression ceilings without
aggregating the underlying events. Its implemented and remaining boundary is
in [`docs/non-interactive-universe-tour.md`](docs/non-interactive-universe-tour.md).

Combat-focused play is a naval command loop as well as ship-to-ship combat:
patrols, escorts, reconnaissance, interception, customs and anti-piracy work,
convoy/mail protection, rescue, tactical command, logistics, and after-action
reporting all provide meaningful activities.

Combat uses CE's one-kilosecond, initiative-ordered vessel activations. Each
activation commits one joint crew order plus standing reaction priorities.
Online players receive editable conservative actions for every crew station;
the persistent tactical controller can instead optimize against a
player-selected minimum estimated chance of completing the encounter
objective. Withdrawal, surrender, and escape craft are real actions rather
than immunity transitions. After an offline battle, a crew retaining control
automatically attempts feasible repair in the order Life Support, Maneuver
drive, Jump drive, then weapons. Full semantics are in
[`docs/combat-control-and-automation.md`](docs/combat-control-and-automation.md).

Other ships may observe and join a combat, but only causally: evidence first
crosses the separation at light speed, then the responder must fly a real
intercept. The current engine deterministically admits qualifying real traffic
on a later combat boundary. Detailed player-owned responder choices and
offline intervention policy are Milestone 7 work.

The middle of the combat-to-trade spectrum is designed around the privateer:
a hybrid role combining authorized naval contracts, convoy escort, commerce
raiding, bounty work, captured cargo, and legitimate trade. Privateering is
distinct from both formal navy service and unlicensed piracy.

The naval captain loop is: receive orders, prepare the ship, choose a route or
patrol, gather information, classify contacts, decide whether to intervene,
resolve the encounter, repair/resupply, and report for new orders.

Pirates remain free to attack any real target they can find and intercept.
Their structured gameplay uses optional intelligence leads, unreliable or
deniable commissions, and crew-defined pirate cruises rather than navy-style
orders. Leads always reference existing traffic, ships, cargo, passengers,
facilities, or scheduled events; they never spawn a victim for the player.
The full pirate loop and its distinction from privateering are specified in
[`docs/pirate-gameplay.md`](docs/pirate-gameplay.md).

Naval wealth can support continued service, retirement, civilian ship
ownership, privateering, or investment. It should not directly upgrade a
state-issued warship, but can enable independent assets and lawful ventures
after a captain leaves or acts outside official service.

In a combat-heavy polity, naval service may be the normal path to owning an
armed merchantman. A player can serve until they can lawfully go private, or
attempt mutiny/ship theft at the cost of warrants, pursuit, hostile ports,
seized assets, and lasting criminal consequences.

Mail clippers are intentionally a low-upside role: fast and reliable courier
work with limited cargo and little path to wealth or progression. They are
specialized/background traffic rather than a competitive alternative to
merchant, privateer, or naval play.

Combat intensity and legal status are separate dimensions. Navy and pirate
play may share tactical combat actions, but differ in authority, missions,
targets, identification, base and intelligence access, logistics, income, and
legal consequences. Law level affects detection and enforcement without being
the sole definition of the difference.

Law level is a CE world/polity parameter under sysop control. Sysop changes
affect detection, enforcement, permissions, and penalties and should be
auditable, without directly granting player resources or changing combat
outcomes.

Cross-polity crime records and warrants travel by physical mail. A fugitive's
destination may have stale or incomplete information, and local enforcement
considers the issuing polity's authority, diplomatic relationship, local
corruption, and possible payoffs. Interstellar banking will make travel and
trade practical while introducing account scrutiny, delayed settlement,
freezes, seizure, and black-market alternatives.

Banking is envisioned as a physical, stagecoach-era/correspondent network:
competing houses, branch agents, bills of exchange, letters of credit, bearer
drafts, and protected courier ships. Couriers and branches can be robbed,
delayed, blockaded, or captured, while powerful banking houses may exert
political influence. Players see instruments, fees, route coverage, delivery
estimates, and status; internal settlement remains opaque. The network can be
faster and more reliable than ordinary mail without becoming instantaneous.

Without an ansible, and with Jump-drive transit times, the setting has a
nineteenth-century-like communications and economic topology at interstellar
scale: large population centres and resource sources anchor regional power,
information arrives late, and local authorities and private shipping/finance
networks matter. This is an analogy, not a literal historical reskin; CE
technology and the 3D galactic setting remain fundamental.

The conversion script and chapter documents should be kept together so that
rule references remain reviewable and reproducible. If the source PDF is
added to this checkout later, record its provenance alongside the generator.
