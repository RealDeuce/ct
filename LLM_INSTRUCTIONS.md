# Cepheus Trader: LLM Instructions

Before beginning or resuming implementation work, read
[`ROADMAP.md`](ROADMAP.md). It is the authoritative source for current focus,
milestone order, and acceptance boundaries. Detailed subsystem documents own
mechanics but do not reorder the roadmap. After a side task, return to the
roadmap's **Resume Here** milestone. Update the roadmap whenever a milestone
begins, completes, or changes order.

This is a living project document. Update it when the game's identity,
rules translations, architecture, data model, commands, balance policy, or
development conventions become clearer. Keep the document concise enough to
be useful, and record decisions rather than relying on conversation history.

## Project identity

**Cepheus Trader** is the tentative name of a BBS door game inspired by
*TradeWars 2002* and *Yankee Trader*. It is a trading, economy, and
space-combat simulator, not an RPG.

The goal is to use the Cepheus Engine (CE) space-trading and space-combat
rules wherever practical while making the result readable, fair, persistent,
and fun in a terminal session. CE is the rules reference; it is not a demand
to reproduce every RPG subsystem.

## Scope and boundaries

The core game may include ships, cargo, routes, ports, prices, fuel, travel,
encounters, combat, damage, repairs, upgrades, credits, reputation-like
market state, and persistent player/galaxy state.

Do not silently turn the project into a character RPG. Character careers,
referee-led adventures, social role-play, and narrative campaign mechanics
are out of scope unless explicitly adapted as a simulation feature and
documented as such. Character skills may remain when they directly affect
operational outcomes such as combat, trading, navigation, engineering,
piloting, or negotiation; they should be compact, testable modifiers rather
than a requirement to reproduce the full RPG career framework.

The full character applicability audit is in
`docs/characteristic-skill-audit.md`. It compares core CE with the newer
Clement vocabulary but does not itself adopt a rules change. Core CE remains
the default until a deviation is explicitly discussed and recorded. The
explicit exception is the adopted Clement change from SOC to CHA: individually
modeled people use STR, DEX, END, INT, EDU, and CHA, while rank, title,
citizenship, reputation, legal status, authority, and relationships are
separately scoped persistent state. Standard cascade versus Clement
base-plus-specialization and wholesale skill-name normalization remain open
choices. Do not turn another audit recommendation or newer sourcebook variant
into an implemented rule without a design decision.
Published-character calibration for any future point-buy system is recorded
in `docs/character-creation-benchmarks.md`; its measured ranges are evidence,
not adopted starting budgets.

The economy should be internally consistent and strategically interesting,
not intentionally absurd. Balance and accessibility may require bounded
markets, anti-runaway safeguards, or simplified CE procedures; document each
translation rather than inventing an unexplained rule.

The universe is a genuine three-dimensional map of star systems. Each system
has a planetary system described using the applicable Cepheus Engine world,
starport, trade-code, and orbital information. The map is anchored to Earth's
position in the Milky Way rather than treating the galaxy as a flat board.
There is no flat-board universe abstraction, and the CE sector, subsector, and
hex-grid star-mapping conventions are explicitly not used for navigation or
system placement. CE material about worlds and planetary systems may still be
used where it describes the contents of a system.
Use these galactic axes consistently: **coreward/rimward**, **spinward/trailing**,
and **galactic north/south**. Coordinate conventions, units,
precision, and the source or generation method for system data must be
documented before they are exposed in gameplay.

The parsec is the universe's base distance unit because CE Jump drives are
rated in parsecs. System-to-system distances, Jump ranges, mail routing, and
long-distance travel calculations should use parsecs rather than introducing
an unrelated map unit. CE does provide a standard **Universal World Profile
(UWP)** for a system's primary world, plus associated system fields such as
population multiplier, planetoid-belt count, gas-giant count, allegiance,
bases, and trade codes. Starport class and services are generated from the
world data. Use those CE-defined fields as the canonical descriptive data for
system, planet, and starport records, while omitting the CE hex-map location
field from navigation.

The universe is an expanding, visit-order-dependent frontier rather than a
fully generated galaxy. When a BBS is materialized, its home system, local
polity, and contiguous surrounding volume are generated. When travel reaches
outside the materialized universe, the server populates the volume out to the
game's maximum Jump radius from the visited system. Stellar occupancy follows
the chosen Milky Way density model, so empty systems and wilderness are normal.
Visit order is an intentional part of world formation; it is not a defect to be
removed by imposing a global master seed.

A new BBS polity is a deliberately rare, locally conditioned cluster on the
current inhabited frontier. It has one to three boundary gateway crossings
within J-2 range (possibly J-1); every other cluster-to-outside system pair is
more than three parsecs apart, requiring J-4 or better for direct entry. At
least one gateway must join the existing system-to-system J-2 component that
contains Sol, making the internally J-2-connected cluster and its capital
reachable from Sol. Its TL12 capital has two different, economically
complementary inhabited systems within J-1 range so a subsistence Jump-1 trade
loop is always available. The union of the three-parsec neighborhoods around
the cluster must be resolved in the founding transaction so later generation
cannot add an accidental J-1, J-2, or J-3 entrance. Empty-space staged Jumps
remain possible under their normal extra fuel, time, and risk. Full
requirements are in `docs/bbs-polity-generation.md`.

The complete BBS cluster and its three-parsec guard volume must lie within
`6,000 pc <= Rgc <= 11,000 pc`, where
`Rgc = hypot(8,178 pc - coreward_x, spinward_y)`. This is a hard eligibility
boundary for the special defensible topology, not a boundary on travel or
ordinary civilization. A separate versioned local-density and conditioning-
likelihood budget must still reject implausible sites caused by spiral-arm,
inter-arm, or Galactic-height conditions.

Each ordinary independently materialized system receives a cryptographically
random seed from the server's operating-system CSPRNG. A conditioned batch,
such as a BBS polity, instead receives one independent 256-bit operation seed
and uses a cryptographic counter stream to draw prospective system seeds until
the required worlds are found. Only accepted per-system seeds become universe
state. Every accepted seed and generation version is persisted with its
system. Deterministic, domain-separated derivations from that seed generate
the star, planetary system, orbital elements, polity baseline, and future
feature streams; use HMAC-SHA-256, ChaCha20, or an equivalently cryptographic
construction rather than a predictable gameplay PRNG. Adding a new named
stream must not change existing generated values. Seeds are server-side data
and are never sent to clients.

Materialization records both generated systems and surveyed empty volume so
that an already resolved region cannot be rerolled. Frontier expansion must
be transactional when multiple players arrive concurrently. Keep generated
baseline data separate from mutable markets, population, ownership, damage,
facilities, and events. Orbital positions are calculated from persisted
elements and simulation time, not redrawn on each visit.

Generated place names are setting content, not debug labels. A system's
polity membership is stored separately and must not be expressed by naming
every member `<polity> N` or `<polity> Capital`. Likewise, a principal world
must have its own name rather than `<system> Primary`. The canonical vocabulary
is `catalog/place-names.toml`: six versioned profiles define separate system
and world morphology. The Federation has a fixed profile, each BBS polity has
a persisted seed-selected profile, and unaligned 25-parsec regions have stable
coordinate-selected profiles. Generated system names are globally unique
case-insensitively; a collision redraws within the same profile and exhaustion
aborts the transaction. A BBS prime world retains the sysop-selected BBS name,
and explicitly established astronomical names such as Sol and Earth remain
hard-coded. Full semantics are in `docs/place-naming.md`.

The authoritative server will be implemented in Rust. The user interface will
be implemented in **C++17 or newer** so it can use OpenDoors and retain
practical BBS door, cross-platform IPC, and serial-port support without
committing the client to C-era ownership patterns. Keep the server/UI boundary
language-neutral and protocol-driven; do not move game rules into the UI to
work around library limitations.

The cross-cutting RPC, idempotency, transaction, storage-versioning, and
scheduled-index rules are in `docs/rpc-and-storage-schema.md`. Every new player
RPC must be explicitly classified as an observation or transaction. Use
closed unions/enums for rule-bearing state rather than English status strings,
and do not persist observation snapshots indefinitely. Travel state uses typed,
revisioned legs between port, body, and Jump loci; contact work attaches to
those explicit boundaries.

The implemented `cepheus-trader-door` links the common protocol layer and the
vendored OpenDoors source under `client/third_party/opendoors/` as static
libraries. Botan TLS and cryptography remain in the shared
`cepheus-trader-client-core` transport library, whose exported ABI is C-only:
opaque handles, fixed-size values, caller-copied buffers, status codes, and
structured error snapshots. C++ exceptions and standard-library objects must
never cross that shared-library boundary.
OpenDoors owns standard command-line and drop-file parsing; its configuration
accepts `CTConfig` to locate the shared game configuration. The door resolves
the BBS real-name-plus-record-index composite (or configured handle) through a
protected BBS-local registry to a monotonic nonzero UInt32, reaches the
server's `newUser` phase, renders the authenticated hello, and reconnects
cleanly. It now implements the
captain, starter-offer, ship, crew, review, and confirmation screens. Keep the
separate headless player protocol exerciser for automated protocol tests.

The sysop `init-credential` command bootstraps a missing shared installation
configuration and protected credential, creating missing parent directories.
The exclusive defaults use `localhost`, game port `7323`, sysop port `7325`,
the sibling `cepheus-trader.credential`, and the sibling protected
`cepheus-trader.identities`; none is overwritten. Initial creation accepts
explicit server, game-port, and sysop-port overrides.
`get-config`, `set-config`, and explicit revision/command-ID retries require an
existing configuration so recovery cannot silently change its target.

League Coordinators are a distinct authority. The server's dedicated
TLS-external-PSK endpoint defaults to port `7326` and accepts numeric League
identities provisioned by the global administrator. CT-League version 1 permits
only status/member listing, revision-checked League naming, creation of new
member BBS credentials, and revision-checked member enable/disable. Membership
is derived from the authenticated League PSK; existing BBSs cannot be attached,
removed, or transferred. New-member creation returns its BBS ID and PSK only
in the exactly-once response for private transfer to the sysop. PSKs never
belong in arguments, environment variables, ordinary logs, or member listings.

The door is a page-oriented, line-oriented interface with exactly three output
profiles: ISO 646 plain text, ISO 646 plus ECMA-48 SGR colour, and CP437 plus
ECMA-48 SGR colour. There is no cursor-addressed TUI or general Unicode
profile. OpenDoors may translate the CP437 repertoire to UTF-8 for transport,
but that does not expand the semantic character set. Every operation is
playable at 40×24; 80×24 is the normal target. The door owns wrapping,
responsive record layout, sanitization, and printable-key fallbacks because
OpenDoors' width field is not supplied by every drop format and its
fixed-coordinate screen helpers assume 80×25. Full presentation semantics and
tests are in `docs/door-presentation.md`. Plain page transitions use form
feed. Enhanced page transitions use reset, clear-screen, and cursor-home;
there is no other coordinate-based rendering or redraw.
Ordinary automatic continuation pauses are enabled by default but can be
durably disabled per local BBS identity in Player Preferences. Disabling them
only streams paged output continuously; it never bypasses an action menu,
confirmation, indexed-page control, or other required input.

New-player interactive guidance follows
[`docs/guided-first-watch.md`](docs/guided-first-watch.md): Guided First Watch
uses the captain's real ship and shared live universe, keeps coaching prose and
screen routing in the door, and performs every game action through the normal
authoritative command. The initial field pilot uses existing typed snapshots
and locally optional presentation progress; it neither creates a tutorial
universe nor requires accepting a fixed offer. Server-owned tutorial
persistence or recommendation is added only if field evidence justifies a
coordinated protocol and storage change.

Enhanced presentation uses stable semantic colours inspired by the
high-contrast *TradeWars 2002* and *Yankee Trader* style: cyan labels,
bright-white values, bright-yellow numbers, bright-magenta identifiers,
green information/success, and red warnings/errors. Do not assign arbitrary
per-screen colours, and never make colour the only carrier of meaning.

All ordinary player-facing copy must be written from within the game world.
Describe what the captain, crew, ship's computer, port, bank, authority, or
communications service knows and does; never explain a screen in terms of the
server, client, RPCs, snapshots, database records, revisions, phase checks,
implementation status, or "authoritative" state. Technical vocabulary is
permitted only in an explicitly identified diagnostic, operator, licensing,
or fatal-error context. A feature that has only a placeholder screen should
say that the corresponding station or service is unavailable, not discuss
unimplemented code. Treat every newly added or materially changed door screen
as requiring an in-world-language review.

### Player-facing rules synchronization

[`docs/game-rules.md`](docs/game-rules.md) is the player-facing rules subset
published as `rules.html`. Any change to code or rule-bearing data that alters
what a player can do, how an outcome is calculated, or what persists in the
universe must update this rulebook in the same change. This includes added,
removed, or renamed actions; dice and D66 procedures; targets and modifiers;
prices, payments, penalties, and deadlines; eligibility and capacity rules;
travel, encounter, combat, damage, recovery, and automation behavior; and the
information a player or offline captain can know. A refactor with no observable
rules effect does not require new rules prose, but must not leave existing prose
describing an implementation that no longer exists.

Write the rulebook for players, not maintainers. State the rule and its visible
consequences directly. Do not include bug-report instructions, implementation
status, code paths, database or protocol details, test strategy, generated-data
plumbing, or advice about which artifact should be corrected after a mismatch.
Do not document a planned feature as though it is available. The rulebook may
explain persistence, ordering, hidden information, and automated control when
those facts affect player decisions, but it must do so in game terms.

The rulebook is a used-rules subset, not a dump of every available source. Full
Cepheus Engine SRD text may remain in the repository, but publish only the CE
rules that Cepheus Trader actually uses. Before any rule text derived from a
non-CE source is committed to the distributable rulebook, remove Product
Identity and restate the mechanic in setting-neutral Cepheus Trader language.
Keep exact source titles and required notices in provenance and OGL Section 15,
not in the player rules. Never use raw third-party prose or a third-party PDF as
a site input.

Prefer links or generated tables when another public artifact already owns
authoritative player-visible data. In particular, ship statistics belong in
the Ship Catalog and the vessel-combat weapon appendix comes from
`catalog/combat-rules.toml`. When the rulebook changes, extend
`tools/test_site_build.py` for material terminology, structure, source-data, or
Product Identity safeguards; rebuild the site; and run the relevant game-rule
tests as well as the site tests. A rule-affecting change is incomplete while
the implementation, rule-bearing data, player-facing rulebook, generated site,
and applicable OGL/provenance records disagree.

Every door prompt must provide a visible way to return to the immediately
preceding screen before a state-changing command is submitted. Use `Q` where
it does not collide with a menu selection, another explicit printable key
where it does, or an empty input where empty has no accepted meaning. Backing
out of a multi-stage wizard preserves choices accepted in earlier stages and
must not silently jump several menus or submit a partial proposal. Once a
state-changing command has been submitted, show its result rather than
pretending that local cancellation can retract it.

Menu options are ordered by the shortcut actually displayed to the player,
not by English labels or source-code order. Shortcut selection and translation
therefore happen before layout and sorting; numbered and symbolic selectors
precede alphabetic shortcuts, and `?` help remains last.

### Licensing boundary

Keep `LICENSE.md`, `OPEN_GAME_LICENSE.md`,
`THIRD_PARTY_LICENSES.md`, `docs/ogc-provenance.md`, and
`docs/potential-ogc-sources.md` current. The potential-source catalogue is an
inventory, not permission to use every listed work and not a list of active
Section 15 obligations. Original software implementation is MIT-licensed.
Rule text, rule tables, mechanical translations, rule-bearing data, original
names, descriptions, and other authored game content are OGL 1.0a Open Game
Content. Cepheus Trader reserves no original game content as Product
Identity. Upstream Product Identity and third-party software remain excluded
from both grants except under their own licenses.

Do not copy expressive OGL material into MIT implementation files. Store it
in clearly identified human-readable rule data and have the implementation
consume it. A change that adds or removes an OGC source must update provenance
and the consolidated Section 15 notice. Consult the potential-source
catalogue for known declarations, inherited works, superseded editions,
duplicates, and exceptions, but re-read the actual source before activation.
A direct or transitive production dependency change must update the
third-party inventory.

Catalog OGL attribution is normalized in `catalog/ogl-sources.toml`.
Every catalog entry has list-valued OGC designations and a `source_ids` list.
Each master source record has a complete, list-valued `notice_ids`
attribution containing the OGL itself, applicable SRDs and inherited works,
and the source work's own notice. Notice text is stored once in atomic notice
records. The loader must not infer omitted ancestry. It rejects missing or
duplicate IDs, expands source bundles, resolves explicit canonical aliases for
equivalent source variants, deduplicates exact notices across
entries, and produces the full-game OGL declaration deterministically.
Cepheus Trader catalog entries have no original Product Identity field
because all authored game content, including class and variant names, is OGC.
Ship naming has two independent structural relationships: a design family
groups a common hull lineage and its variants, while an upgrade path spans
families and hull sizes as a progression ladder. There are nine upgrade paths,
one for each trade/mixed/combat crossed with orderly/contested/chaotic polity
cell, and each is presumed to be the work of one specialist manufacturer or
shipyard. A family may contain designs native to several paths. Paths may be
sparse rather than spanning every size from starter through 5,000 tons; an
explicit adjacent-path design may backfill a gap without being reassigned or
duplicated. Stable IDs, mechanics, family membership, native path membership,
progression positions, and backfill relationships are global catalog data.
Local history, advertising, reputation, cultural interpretation, and similar
setting prose may be supplied by individual BBS sysops only as a presentation
overlay and must never alter canonical mechanics, balance, availability state,
or player resources.

The completed family pass is authoritative in
`catalog/ships/families.toml` and explained in
`docs/ship-family-grouping.md`: 215 designs form 114 families,
including 39 multi-design shared lineages and 75 singleton families. Every
ship record repeats its numeric `family_id`, and validation requires complete,
one-to-one membership agreement. Family grouping requires actual lineage
evidence; equal displacement, role, or normalized statistics is insufficient.
Path membership must not regroup designs merely to make a path complete.

The native-path pass is authoritative in
`catalog/ships/upgrade-paths.toml` and explained in
`docs/ship-upgrade-paths.md`. Every design has exactly one native path and one
mechanically derived size stage: auxiliary, starter, light, medium, heavy, or
capital. The nine paths are intentionally asymmetric and sparse. Native path
does not restrict ownership, availability, capture, command, licensing, or
refit. Exact adjacency and preferred backfill successors remain explicit
future catalog relationships; never infer them from tonnage alone.

Canonical PI-free names are authoritative in `catalog/ships/names.toml` and
explained in `docs/ship-catalog-naming.md`. The registry names all nine paths
and manufacturers, all 114 families, and all 215 fitted designs. Names use
historical, mythic, geographic, scientific, and public-domain literary
references. A path supplies a six-stage semantic naming sequence; family
vocabulary takes precedence when variants of one hull occupy several paths.
Every ship repeats its canonical display name, and validation requires exact
agreement. Sysops may add local aliases and setting prose only as a
non-mechanical overlay; interfaces must preserve access to the canonical name
and stable tag.

Do not link GPL, AGPL, SSPL, or comparable strong-copyleft code into a
production executable without an explicit compatibility review. LGPL
libraries require an explicit compliance plan. Official client packages
statically link the vendored OpenDoors source and publish the exact tagged
source, build files, modifications, and license beside the binaries so a
recipient can rebuild and relink. Every release package must contain its
applicable license texts and notices, even though the door also embeds an OGL
viewer.

Universe coordinates are three IEEE-style double-precision values in parsecs.
Positive directions are coreward, spinward, and galactic north respectively;
negative values are rimward, trailing, and galactic south. Earth is the game
origin at Galactocentric radius 8,178 pc and height +20.8 pc; the complete
density transform is specified below. Coordinate serialization precision
still needs to be specified, but no additional player-facing map coordinate
system should be introduced.

The version-1 stellar distribution is implemented in `server/src/universe.rs`
and specified in `docs/stellar-distribution.md`. It converts the Earth-centered
frame to Galactocentric radius, spinward azimuth, and height only for density
calculation. It uses a locally normalized exponential disk, a two-component
vertical falloff, and four identical trailing logarithmic arms separated by
quarter turns. The arms have a ten-degree pitch, curved centerlines, and a
derived local spinward tangent; do not replace them with independent
coreward/spinward stripes. New frontier volume is eventually sampled as an
inhomogeneous Poisson process using OS cryptographic randomness, then systems
and empty surveyed coverage are persisted under the distribution version.

Resolved stellar volume uses the internal lattice specified in
`docs/observed-volume.md`: quarter-parsec cells, 32 cells per axis in an
eight-parsec chunk, and one 4,096-byte bitmap per distribution/sampler layer.
The canonical Jump-arrival footprint contains every cell with positive-volume
intersection with a six-parsec sphere around the target coordinates. The
implemented oracle returns either `FullyMapped` or revision-tagged missing
chunk masks. Applying those masks is a private caller-transaction primitive;
future code must commit generated systems, coverage, journal decisions, and
Jump completion atomically rather than exposing an unjournaled mark-mapped
operation. Resolved server geometry never implies universal player knowledge.

### CE coverage audit

The local CE reference covers much of the mechanical foundation, but not every
persistent-door policy:

- **Skills and action resolution — covered.** CE supplies 2D6 task checks,
  characteristic modifiers, skill levels, untrained penalties, time frames,
  multiple-action penalties, and skill training rules.
- **Characters and progression — covered after removing the RPG framing.** CE
  supplies characteristics, skills, careers, ranks, commissions, advancement,
  aging, injuries, benefits, and retirement. In this game, merchant progress is
  primarily wealth and assets; naval progress is rank, authority, and access
  to larger or more capable ship classes. We should preserve CE skill/rank
  mechanics while omitting career-play as the main ongoing loop.
- **Ships and crews — covered.** CE supplies ship construction,
  standard designs, drive/fuel requirements, crew positions and salaries,
  mortgages, life support, port fees, maintenance, revenue, and detailed
  combat damage. Ship ownership, replacement, crew retention, and repairs can
  use CE's ship shares/mortgages, material benefits, crew model, maintenance,
  and Engineer/Mechanics repair procedures.
- **Failure and recovery — covered by the operating model.** CE supplies
  misjumps, combat damage, crew injuries, fuel/cargo loss, maintenance
  degradation, battlefield damage control, arrests, and sentencing. An
  Engineer and repair facilities are the normal recovery path; bankruptcy or
  character death starts a new character rather than requiring an additional
  RPG-style recovery subsystem.
- **Economy — covered at the trading layer.** CE supplies speculative goods,
  trade-code price modifiers, suppliers/buyers, brokers, bulk freight,
  passengers, mail, charters, fuel, port services, and ship revenue. It does
  not provide a persistent macroeconomic production/consumption model, banking
  network, or polity budgets—and we do not need one for a single ship to move
  planetary prices. Buyer searches are inventory-backed negotiations: they
  require matching player-owned speculative cargo aboard, and their leads
  cannot exceed the matching quantity still carried when the search completes.
  A single ship's cargo capacity should remain too small to swing a planetary
  economy; CE's local trading layer is the intended baseline.

### TL14 Dreadnought affordability audit (working estimate)

The local Common Vessels chapter lists a **TL14 Dreadnought**, not a TL15
vessel. Its listed price is MCr2,768.145. The description includes the ship's
weapons and carried craft, but CE states that ammunition and fuel are separate
from the standard-design discount. Adding 3,600 smart missiles at Cr2,500
each and 1,096 tons of refined fuel at Cr500 per ton gives an initial
fully-stocked estimate of **MCr2,777.693**.

Using CE's commercial rhythm of one week in Jump and one week in normal space,
the gross income comparisons are intentionally stark:

| Role/assumption | Gross per two-week leg | Legs to cover MCr2,777.693 | Game time |
| --- | ---: | ---: | ---: |
| Bulk freight, optimistic full 412-ton load | Cr412,000 | 6,742 | 259 years |
| Bulk freight, one average Class-A destination (105 tons) | Cr105,000 | 26,455 | 1,017 years |
| CE mail contract | Cr25,000 | 111,108 | 4,273 years |

These are gross figures before fuel, crew, life support, maintenance, port
fees, cargo acquisition, and damage. Speculative trade is the only plausible
commercial path, but CE does not define a typical profit: it depends on skill
levels, working capital, supplier/buyer availability, trade codes, price-roll
results, cargo mix, and risk. For scale, net profits of MCr10, MCr50, and MCr100
per two-week leg would take about 5.3, 2.1, and 1.1 game years respectively;
those are sensitivity examples, not CE averages.

The CE mortgage would be about MCr11.534 per month on the listed hull price,
with a one-fifth acquisition value of about MCr553.629 before operating
reserves. A Dreadnought therefore makes sense as a polity-owned naval asset
assigned through rank/ship class, not as a normal merchant purchase. CE does
not provide naval mission payouts or privateer prize valuations, so those
roles require separate reward tables rather than pretending the trade rules
can price a warship.

### Frontier Trader to Dreadnought: trade-only progression

The intended merchant progression now starts with a mortgaged **TL9 Frontier
Trader** and uses only CE speculative trade and cargo activity to reach a fully
stocked TL14 Dreadnought. This is a game goal, not a result that CE specifies
directly. CE's trade tables provide a distribution of outcomes, not a typical
profit, so completion time must be measured by a seeded simulation (or reported
as a range) after we choose starting cash, broker skill, route quality, risk,
ship-sale/debt rules, and an upgrade ladder.

The starting numbers expose an important constraint:

| Hull | Listed price | CE 1/5 share value (if applicable) | Monthly mortgage | Cargo | Known annual floor* | Full-load bulk gross/year** |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| TL9 Frontier Trader | MCr82.314 | MCr16.463 | MCr0.343 | 75 tons | MCr5.646 | MCr1.950 |
| TL9 Merchant Freighter | MCr59.814 | MCr11.963 | MCr0.249 | 261 tons | MCr3.953 | MCr6.786 |
| TL14 Dreadnought | MCr2,768.145 | MCr553.629 | MCr11.534 | 412 tons | MCr22.384 | MCr10.712 |

\* Mortgage, listed crew salaries, life support, 0.1% annual maintenance,
and 26 refined-fuel legs per year; excludes repairs, port fees, cargo purchase,
and ammunition. The Dreadnought floor assumes its listed 223 active crew at a
minimum gunner wage, so its true crew bill is higher.

The 1/5 figures are CE ship-share/benefit values, not an assumed down payment.
With no shares, the ordinary mortgage payment is calculated from the listed
cash price; shares reduce the financed cash price if the character actually
has them.

\*\* Twenty-six two-week legs per game year, all cargo capacity filled, at CE's
Cr1,000/ton bulk-delivery fee. Unrefined or skimmed fuel lowers the fuel line,
but does not change the conclusion.

Bulk cargo therefore cannot even service the Frontier Trader's known annual
floor: it needs about **Cr142,000 of net speculative-trade profit per leg**
after bulk revenue merely to break even. The Merchant Freighter is a useful
first commercial upgrade because it is cheaper than the Frontier Trader while
carrying 261 tons, but the Common Vessels list then jumps to mostly military
hulls and finally the Dreadnought. It does not define a natural sequence of
larger merchant ships. We must either add game-specific commercial designs
(using CE Chapter 8 construction rules) or make some upgrades polity-owned or
mission-earned rather than purchases.

The Clement Sector expansion changes that particular result. Its expanded bulk
cargo rule pays **Cr3,500 per ton per parsec**, with a port-pair availability
table rather than CE's flat Cr1,000/ton delivery fee. On a one-parsec A-to-A
route, a full Frontier Trader load grosses Cr262,500 per leg; even if the game
conservatively charges both the listed 5–8% broker fee and the Cr100–300/ton
shipper fee against that rate, the result is roughly Cr219,000–242,000 per leg.
That is just enough to cover the Frontier Trader's known annual floor at 26
legs/year, but leaves little capital for upgrades. The same calculation gives a
Dreadnought roughly Cr31–35M of bulk gross per year before expenses, above its
CE-only operating floor but nowhere near a private Dreadnought mortgage. The
Bounded Fortune financing rules require comprehensive, crew, passenger, cargo,
and public-liability insurance for financed ships. The listed comprehensive and
public policies alone each cost 0.5% of hull value per year; adding the listed
crew and passenger policies raises the known annual floors to approximately
MCr6.493 for the Frontier Trader, MCr4.572 for the Merchant Freighter, and
MCr50.357 for the Dreadnought, before cargo-liability premiums, taxes, port
fees, or repairs. A financed Dreadnought therefore needs state or institutional
support even under the expanded bulk rate.

This means the trade-only progression is plausible only when the game adopts a
bounded speculative-trade model as well as expanded bulk cargo. The Clement
expansion's price procedure is materially less explosive than CE's original
20%–400% modified-price table: purchase checks change the base price by -20%,
-10%, 0%, or +20%, while sale checks produce a 30%, 15%, 2%, or 0% markup.
For example, a 100,000-credit/ton lot bought at -10% and sold at +15% yields
Cr25,000/ton before loading, travel, and other costs. This is a useful balancing
baseline, but it must be chosen explicitly; do not mix both price procedures in
one market.

The Clement Sector `Ships.ods` catalogue suggests a useful *candidate* ladder:
Pleiades Light Freighter (141-ton cargo, MCr91), Lee Merchant Vessel (160
tons, MCr123.8), Atlas Freighter (345 tons, MCr276.9), Aranui Medium Cargo
Vessel (500 tons, MCr336.9), and Hercules Heavy Freighter (1,070 tons,
MCr615.1). These figures are source-data leads, not adopted CE prices.
Non-capital Zimm-drive ships through 2,000 tons can normally retain their
published drive letter, displacement, and fuel when converted to CE Jump-2,
subject to the cleanup in `docs/zimm-to-jump-conversion.md`; prices and
operating schedules still require an audit. The Dreadnought remains a military
endpoint with only 412 tons of cargo, so “larger” should not be confused with
“more profitable” at that stage.

CE also does not specify a resale price, early mortgage payoff, or transfer of
an encumbered hull. The game must define those before “upgrade” has a precise
meaning; the conservative accounting rule is that usable equity is resale
proceeds minus the outstanding mortgage balance and any liens.

The Dreadnought is also not self-supporting as a private cargo ship under these
assumptions: its full-load bulk gross is below its known annual floor. If it is
privately financed, the mortgage adds about MCr138.407/year, requiring roughly
MCr5.772 of net speculative profit per leg even after optimistic bulk revenue.
That strongly favors treating the Dreadnought as a late polity/naval transfer,
with the trade game funding the player's equity, influence, or purchase share,
rather than pretending bulk cargo alone can buy and operate one.

For implementation, the progression metric should be **net speculative profit
per two-week leg** after purchase price, sale proceeds, cargo loss, and the
ship's operating costs. For a fixed net rate `P`, a deliberately simplified
sanity check is:

`game years = (target cash - starting equity) / (26 × P)`

The result is only a bound; a real run must roll CE supplier availability,
purchase and sale prices, cargo quantities, broker modifiers, travel risk, and
the timing of each upgrade. Do not present a single “CE completion time” until
those game-specific assumptions are recorded and the seeded trade simulation
exists.

### Economy-first balance baseline

The merchant path is the first balance instrument. It has the clearest
measurable inputs and outputs, so naval, privateer, and pirate progression
should be calibrated against it rather than balanced independently. The first
balance artifact should be a seeded cohort of mortgaged TL9 Frontier Traders,
not a hand-picked profitable route.

If route quality is modeled, the cohort should include a discovery/learning
phase and then assume convergence toward the best dependable route or route
network available to that trader. Do not dilute the merchant baseline by
randomly selecting mediocre routes forever. Controllable factors include the
port pair, cargo category and mix, broker, departure timing, financing,
insurance, and risk posture. The model should still charge the time, money,
and information cost of discovering or switching routes.

Convergence does not mean perfect prediction. Market rolls, lot availability,
delays, encounters, law changes, competition, and other external events remain
stochastic. An “excellent route” is one that remains profitable after those
risks and the ship's operating costs, not one that depends on a single lucky
price roll. If a route degrades, the trader should be able to re-optimize after
the appropriate scouting and mail/information delay.

Each run should declare its starting cash and working-capital reserve, broker
and relevant crew skills, route/port quality, cargo policy, financing terms,
insurance, taxes and fees, encounter risk, and the available upgrade ladder.
The simulator should roll the selected Bounded Fortune speculative procedure,
expanded bulk freight, standard CE Jump travel, cargo losses, repairs, and
delays. It should record net cash flow after cargo purchase, sale, fuel, crew,
life support, port services, maintenance, insurance, taxes, mortgage, and
losses—not gross trade revenue.

The first reports should include median and 10th/90th-percentile time to each
upgrade, probability of a liquidity failure or default, repair and cargo-loss
frequency, time spent waiting for a usable opportunity, and the fraction of
daily actions that produce a meaningful decision. Report usable equity as
resale proceeds less outstanding debt and liens; keep a separate working-
capital reserve so a nominally wealthy but cash-poor ship is not treated as
ready to upgrade. A single lucky run is not evidence of a healthy economy.

The progression ladder should be evaluated in stages: staying solvent in the
Frontier Trader, acquiring a first commercial upgrade, reaching a profitable
mid-size freighter, and finally acquiring the wealth/influence needed for a
military endpoint. Exact game-time targets remain open until this cohort is
simulated. If the ladder has no natural CE commercial hulls, use explicitly
labelled CE-construction adaptations or polity-owned transfers rather than
quietly treating a warship as the next merchant ship.

Once the merchant distribution is acceptable, compare other paths by
risk-adjusted expected value and useful session decisions. Naval progression
should pay in rank, authority, mission access, and ship class, with personal
wealth as a secondary result. Privateering should have merchant-comparable
expected value plus legal/combat choices and prize volatility. Piracy may have
higher gross upside, but capture, seizure, warrants, hostile ports, and
settlement risk must pull its reliable median back toward the intended
merchant baseline. No path should bypass the merchant economy without an
explicit decision about what replaces its working-capital, debt, and failure
constraints.

The current pre-simulation sanity check supports a real commercial path. With
favorable trade-code modifiers and Broker skill around 3–4, the Bounded Fortune
purchase/sale outcomes imply roughly 13–20% gross speculative margin on the
base cargo cost before loading, broker commissions, and ship overhead. A
Frontier Trader carrying 75 tons at roughly Cr50,000–150,000 per ton therefore
has about Cr0.5–2.3M of gross trade margin per leg; its known operating floor is
about Cr0.22M per leg. After transaction costs this can remain positive, but
only when the route supplies enough valuable cargo and the trader has adequate
working capital. Low-value cargo or hired-broker costs can reduce the lower end
to break-even.

Consequently, exceptional cargo, prizes, missions, or windfalls should be
accelerators and recovery opportunities, not prerequisites for the ordinary
merchant upgrade ladder. If a competent trader cannot progress on the best
repeatable route with ordinary outcomes, the route economics, financing, or
hull ladder should be repaired before adding jackpots. A privately owned
Dreadnought remains a separate institutional or naval endpoint; ordinary trade
need not make its mortgage and insurance self-supporting.

### Merchant contracts and enterprise scale

Speculative route selection alone is not a sufficient merchant game loop. CE's
individual Broker checks are RPG task resolutions, so the simulator should use
them as modifiers to bounded economic outcomes rather than as the whole
economy. The primary ship-scale activity should be generated **contracts**,
not authored fetch quests. A fixed “move this cargo for this payment” job is
freight hauling and should remain a dependable, low-upside option rather than
the merchant progression engine. The important contracts are purchase orders,
forward sales, shortage positions, supply commitments, consignment, and
letters of credit: the trader must source or buy the goods, finance the
position, choose transport and timing, and bear inventory, price, delivery,
and settlement risk. Passenger charters, secure-data carriage added to an
independently viable voyage, relief shipments, salvage, convoy work, and lawful
or unlawful high-risk cargo are additional roles, not the default definition
of trading.

Each market commitment should expose its origin and destination, commodity,
quantity and quality, bid or floor price, delivery window, working-capital
requirement, collateral, insurance, legal status, failure penalty, and known
route/encounter risks. News, shortages, wars, infrastructure work, and polity
policy should generate temporary offers; mail delay should determine when the
offer is visible and how much of its premium remains. A contract is a market
position with state and consequences, not a narrative quest. The merchant loop
is therefore: inspect offers and news → decide whether to take price and
inventory risk → source or buy → finance, insure, and load → travel and manage
incidents → deliver or liquidate, settle, and re-evaluate.

Contracts are competitive and time-sensitive. They have a lifecycle of
publication, delayed visibility, claim or acceptance, reservation, fulfillment,
settlement, expiry, cancellation, or default. An offer received from another
system may already be reserved or fulfilled by the time the trader attempts to
claim it. A remote claim must therefore be a real action with a communication
delay, not an instantaneous UI reservation. Early traders can accept this race
or travel to the issuing market; established houses can pay for a local agent,
correspondent, escrow, or protected bank message to reserve an offer, while
still risking stale information and competing claims.

Non-performance must have graduated consequences: forfeited deposits or
collateral, cargo liquidation, contract penalties, insurance claims, damaged
credit, reduced access to future offers, higher financing costs, and—when the
contract is public or politically important—legal action or a warrant. Partial
delivery, renegotiation, assignment, force majeure, and an open-market sale
should be possible where the contract and local law permit them. The penalty
must be legible before acceptance so a trader is choosing risk, not discovering
an arbitrary punishment afterward.

### Passenger and passenger-courier services

Ordinary passenger transport is a capacity market. The captain advertises an
itinerary, departure window, service class, and available berths—for example,
“four middle-passage berths to Spicio II”—and accepts matching bookings. The
booking reserves passenger capacity and creates a normal manifest obligation;
it is not an authored quest. Physical loading and disembarkation can be
automatic for declared passengers, but payment, inspection, no-shows, medical
issues, and contract exceptions remain explicit settlement states.

Passenger capacity is separate from cargo tonnage and includes high/middle
passage, low berths, bulk groups such as workers or refugees, and full or
partial charters. Passenger obligations may include life support, steward or
medic service, baggage, security, privacy, and destination restrictions.

Occasional high-security or time-sensitive passenger transport belongs in the
contracts/orders system. A banker, diplomat, VIP, or fugitive may require a
private cabin or exclusive charter, a hard deadline, alias or identity checks,
confidentiality, escort, permitted stopovers, escrow, and penalties for delay,
disclosure, or loss. A fugitive contract can carry a large premium but creates
warrant, inspection, seizure, and bounty risk; a legitimate banker or VIP may
instead pay for discretion and punctuality. These contracts consume capacity
and restrict the captain's flexibility, but do not replace ordinary trading.

### Merchant port-call loop and concurrent obligations

The normal merchant turn is a port-call state machine:

1. **Arrive:** reconcile the in-transit manifest, active contracts, delivered
   mail, news age, and any incidents from the jump or interplanetary leg.
2. **Choose commitments:** inspect newly visible offers and status changes;
   claim or reject opportunities, reserving the required funds, cargo space,
   collateral, and delivery window. A claim may be made before unloading when
   the ship can prove it has enough capacity and liquidity.
3. **Reach the port:** handle control, customs, and local services; travel to
   the planet or orbital facility as required.
4. **Settle the old load:** unload, inspect, and deliver cargo whose destination
   or contract is this port; collect payment, settle claims, and record any
   late or damaged delivery.
5. **Turn the ship:** source or buy cargo for accepted commitments, load and
   secure it, refuel, repair, insure, hire services, and decide whether to
   remain for another opportunity or depart.
6. **Advance obligations:** move active contracts to their next state and
   account for deadlines, mail claims, financing, and exposure during the next
   leg.

Several contracts may be outstanding in different states—claim pending,
accepted, sourcing, loaded, in transit, awaiting settlement, or disputed. This
is the merchant's higher-performance play: planning a portfolio and a
multi-stop itinerary instead of waiting for one contract to finish before
looking for the next. It must not create impossible parallelism. A single ship
still has one physical location, finite cargo and fuel capacity, finite
working capital, and a bounded crew/action rate. Every accepted contract
reserves the capacity, funds, or credit it needs, and overlapping deadlines or
cargo claims can create real default risk.

### Ship-computer assistance and automation boundary

Resource management should not require the player to maintain an external
spreadsheet, but the computer must not silently play the merchant. The
ship-computer layer should provide authoritative bookkeeping and planning:
manifest and contract ledgers, reserved cash and cargo, fuel and route
feasibility, deadline countdowns, exposure and default warnings, offer filters,
itinerary proposals, and what-if profit/loss estimates. It may execute
explicitly authorized routine policies—such as a fuel minimum or a spending
cap—but every policy must be visible, bounded, and reversible.

For a desired commodity or accepted purchase order, the computer may also show
a source-planning view for known nearby worlds and ports. Each candidate should
include the last-observed market timestamp and provenance, data age, estimated
availability and lot-size range, expected purchase-price range, search and
loading time, travel time and fuel, port and legal risks, required working
capital, and an estimated probability of arriving with a usable load before the
contract deadline. Delivered-cost and margin estimates should be ranges derived
from those uncertainties, not a single guaranteed route.

Remote market data remains a stale observation. Local contact, paid brokers,
agents, and fresh mail can improve it, each with its own cost and delay; the
actual market state is resolved only when the trader reaches or communicates
with the source. The computer must display freshness and confidence prominently
and must not expose future prices, hidden offers, or an infallible “best source.”

The player must still decide whether to claim a market position, accept its
risk, choose suppliers and destinations, allocate scarce cargo and working
capital, depart, renegotiate, liquidate, or default. Those choices are the
merchant gameplay and must not be hidden behind an “optimal route” button.

Helper scripts are optional clients, not a second rules engine. The protocol
may expose read-only state subscriptions, calculations, alerts, and draft
plans so a player can build personal tools, but the server must validate every
state-changing command, enforce the same action/rate limits as the built-in UI,
hide no information from the normal client, and record the resulting player
decision. Scripts must not win remote contract races by issuing faster or
privileged claims; if an action is allowed to automation, it must be equally
available through the ordinary ship-computer interface.

The higher-level progression should unlock a **merchant house** or trade
enterprise after the player demonstrates solvency, credit, and reliable
delivery. It provides scaling mechanisms rather than an arbitrary income
multiplier: consignment and purchase orders, letters of credit, warehouses and
depots, exclusive or standing freight contracts, hired captains and agents,
chartered or owned additional ships, and the ability to manage several
contracts concurrently. This creates compounding growth through throughput,
capital reinvestment, and controlled leverage. Debt service, insurance,
counterparty default, delayed settlement, management capacity, political
access, and loss events must scale with the enterprise so growth is powerful
but not risk-free.

This gives merchant play a parallel to naval orders without turning it into an
RPG quest chain: the navy receives missions for a state ship, while the
merchant takes positions and economic obligations for private assets.
Privateers can mix market positions, escort, bounty, and prize work.
Exceptional cargo and windfalls may accelerate an enterprise, but the normal
scaling path is controlling inventory and credit risk while increasing
dependable throughput. A merchant who only wants guaranteed transport can haul
freight, but that path should have the deliberately limited wealth curve
already assigned to mail and clipper work.

The spread is not academic: CE permits a 20% purchase and 400% sale result
(16+ on both checks). A full load of manufacturing equipment would then gross
Cr2.85M per ton before costs, while an ordinary low-value cargo can lose money;
the high-value goods also arrive in small, randomly rolled lots. This is why the
progression needs route/skill/starting-capital scenarios and percentile results,
not one hand-picked “profitable trip.”

### Supplemental trade, port, naval, and piracy audit

The following local supplements have now been inspected. They are
Cepheus-compatible references and design inputs, not replacements for the local
CE chapters. Any rule adopted from them must be labeled as a game adaptation and
must be rebuilt around our three-dimensional map and standard Jump drive.

**Clement Sector, Third Edition** (`Clement_Sector_Third_Edition.pdf`) expands
the merchant loop in ways that are directly useful to Cepheus Trader:

- Bulk cargo is obtained with a Broker/EDU task taking 1–6 hours; exceptional
  success doubles availability, success gives the listed amount, failure halves
  it, and exceptional failure yields none. Attempts can be repeated every three
  days, but a third attempt causes earlier cargo to be reassigned as late.
- Bulk availability is determined by origin/destination starport classes and
  the rate is Cr3,500/ton/parsec. This is a strong candidate for the game's
  distance-sensitive cargo baseline.
- Speculative suppliers provide port-dependent lots (including common and
  trade goods), with success tiers controlling whether trade lots are present.
  Purchase price changes by -20%, -10%, 0%, or +20%; sale price offers a 30%,
  15%, 2%, or 0% markup. This bounded procedure is preferable for an economy
  intended to support a predictable daily play loop, but it must not be mixed
  with CE's original modified-price table.
- Smuggling adds black-market sourcing, customs, bribery, forged documents,
  inspection risk, and law-dependent consequences. It is a natural optional
  branch for the privateer/pirate end of the spectrum.

**Port of Entry: Starports in Clement Sector** (`Port_of_Entry.pdf`) supplies
the missing port-state layer: E-class ports provide essentially only a landing
area; D-class adds unrefined fuel, limited repairs, trade kiosks, and warehouses;
C-class adds a small shipyard and major repairs; B-class adds refined fuel and a
shipyard up to 2,000 tons; A-class can build ships of any size. It also gives
customs/security expectations by law level, freight handling, warehouse storage,
broker offices, and port service delays. Brokers commonly take 5–8% of the
shipping rate, with an additional Cr100–300/ton shipper-side fee. These details
should become in-system actions and costs rather than flavor text.

**Anderson & Felix Guide to Naval Architecture**
(`Anderson_and_Felix_Third_Edition.pdf`) provides useful commercial cargo
options: superior holds (Cr10,000/ton), armored holds (Cr0.1M/ton and harder
cargo theft), liquid holds (Cr8,000/ton), freezer holds (Cr9,000/ton), concealed
compartments (Cr20,000/ton, capped at 5% of hull), modular holds (+10% space and
Cr0.2M per attachment ton), and cargo-handling equipment. It also explicitly
distinguishes cargo-maximizing commercial ships from armor- and weapon-focused
naval capital ships. These are candidates for ship refits and differentiated
trade roles. Non-capital hulls normally permit direct same-letter Z-to-J
conversion, while Zimm-only equipment, software names, surcharges, and
operating assumptions must be replaced as described in
`docs/zimm-to-jump-conversion.md`.

**Hub Federation Navy, Third Edition**
(`Hub Federation Navy Third Edition.pdf`) is primarily an organizational and
career supplement rather than an economy system. Its useful non-RPG material is
the command taxonomy: patrol corvettes, frigates, destroyers, tenders, tankers,
replenishment ships, cruisers, carriers, and monitors, each with typical roles
and command ranks. Its fleet model uses squadrons, battle squadrons, battle
fleets, flotillas, and system-defense commands, with rotating deployments. We
can use this to structure naval assignments, logistics, and rank-gated ship
access, but it supplies no ready-made mission payout table.

**Skull and Crossbones: Piracy in Clement Sector**
(`Skull and Crossbones Third Edition.pdf`) gives the privateer/pirate side the
economic and legal hooks that CE lacks:

- Cargo raiding, ship theft, commerce raiding, and marauding are distinct
  activities with different encounter and consequence profiles.
- A stolen ship may sell for up to 25% of original value at a sympathetic haven;
  stolen cargo may be liquidated at about 10% of accepted value in one example.
- A letter of marque and prize court can make a capture lawful. Civilian
  captures may receive roughly 10% of ship value, licensed privateers up to 30%,
  and some governments transfer the captured ship instead of paying cash.
- Boarding, cargo jettison/negotiation, ship capture, pirate havens, warrants,
  and severe anti-piracy penalties provide the encounter aftermath and legal
  state needed for the privateer midpoint.
- It does not provide a formal pirate mission generator or rank ladder.
  Articles of agreement, no-prey/no-pay shares, havens, patrons, adventure
  premises, and the four operation types instead support the adopted
  free-predation/lead/commission/cruise structure.

These rules reinforce the intended spectrum: the merchant uses bulk and
speculative cargo, the privateer adds licensed capture and prize adjudication,
and the pirate accepts black-market liquidity, warrants, and hostile ports.
They do not change the standard-drive, no-ansible, physical-mail decisions.
The PI-free mechanical adaptation and role loop are specified in
`docs/pirate-gameplay.md`; do not import the source's named pirates, groups,
places, ships, plots, or setting prose.

**Bounded Fortune: Independent Merchants in Clement Sector**
(`Bounded Fortune.pdf`) is the most useful supplement for the actual merchant
progression. It adds several systems that CE leaves open:

- Ship financing includes leasing, lease-to-buy, corporate/government funding,
  partners, venture capital, used ships, repossessions, auctions, trade-ins,
  haggling, down payments, insurance, default, and repossession. Typical used
  dealers offer 40–60% of original value, independent sellers 30–50%, and
  trade-ins about 40% before condition deductions; auctions can be cheaper but
  are “as is” and add a buyer fee. These rules give “upgrade” a real resale and
  debt-equity model, subject to an explicit game choice about which financing
  paths are allowed.
- Bank transfers are local or system-limited; interstellar settlement requires
  physical couriers and can take weeks or months. Credit reports propagate by
  courier as well. This matches our physical-mail/banking model and supplies
  delayed credit, default, and repossession consequences.
- Freight hauling adds dedicated brokers, Trade Characteristic modifiers,
  repeatable freight searches, contract freight, and established freight from
  corporate/government funding. A successful small-company contract can reserve
  a route for 1–6 years and pay Cr3,000–3,500 per ton per parsec. These are good
  optional steady-income paths, but the first progression experiment should
  keep them separate from speculative trade so we can measure each role.
- The updated speculative-trade table is explicitly recommended over the
  Clement Sector Third Edition table and adds detailed D66 sub-tables for the
  contents of broad trade categories. It should be our preferred expanded
  speculative-trade reference if we adopt that adaptation.
- Courier and mail rules model electronic messages as data carried on the ship,
  physical letters and parcels as cargo, and standard contracts as 5 tons for
  Cr25,000 per delivery. This is directly compatible with the no-ansible rule;
  we should retain our polity-dependent schedules and add the supplement's
  contract, capacity, and reputation checks.
- Trade Routes introduces a route-quality signal: Infrastructure, Solidarity,
  Population, Law Level, and Starport Factor combine into a Trade Characteristic
  that predicts cargo availability (`PER = INF × SOL × POP`; `TC = (PER ×
  Starport Factor) / Law Level`). We should use the same idea with generated 3D
  systems, while recalculating route distance and connectivity from our actual
  Jump network rather than importing Clement's fixed routes.
- It defines cargomaster and broker roles, typical merchant crews, hiring and
  wage procedures, container/pallet handling, random in-system encounters, and
  an arrival procedure. The procedure is Zimm-specific, but its sequence—scan,
  choose fuel source, navigate in-system, contact control, handle mail, dock,
  and process cargo—maps cleanly onto our jump/interplanetary/encounter frames
  after replacing Zimm transitions with standard CE Jump operations.

Interstellar travel uses the standard Cepheus Engine Jump drive. The optional
alternative drives (warp, teleport, hyperspace, and similar variants) are out
of scope unless a later decision explicitly brings one back. The standard
Jump's approximately one-week transit is therefore the game's baseline FTL
time abstraction; do not introduce alternative-drive assumptions while
implementing travel or trade.

Standard Jump operations may deliberately target empty-space staging volumes;
a destination mass, star, or Zimm point is not required. A double-tanked
Jump-1 ship may cover up to two parsecs as two separately resolved Jump-1
legs. Planned two-leg travel includes a mandatory one-game-day midpoint
turnaround for position fixing, scanning, drive/grid inspection, problem
resolution, and correction of the second plot. A double-Jump course tape is a
fresh, departure-window-specific bundle of two Jump-1 plots whose second
solution is time-indexed for the expected midpoint turnaround and includes a
limited correction envelope. Both legs retain independent Jump-success
exposure: if either leg has clean-success probability `p`, end-to-end clean
success is `p²`. Exact semantics and open pricing/UI questions are in
[`docs/interstellar-jump-operations.md`](docs/interstellar-jump-operations.md).

There is no ansible and no instantaneous faster-than-light communication.
Interstellar electronic messages travel as data physically carried by ships.
They consume negligible displacement and normally ride ordinary commercial,
naval, privateer, passenger, and other traffic rather than a dedicated mail
fleet. Electronic mail never causes, redirects, or economically justifies a
transit: a carrier takes a bag only when it is already making that exact hop
for another reason. Large polities obtain daily or better service only where
their ordinary traffic already supplies it. Outside them, messages wait as
long as necessary for a suitable departure. Physical letters, parcels, secure
objects, and urgent passengers remain ordinary cargo, passage, or contracts.
Communication latency is therefore a gameplay consequence of geography,
route coverage, schedules, and ship traffic, not merely a network setting.

The engine must track **mail time** between points in the universe. News and
other messages have an origin time, a dispatch time, a route/carrier history,
and a delivery time; they are not instantaneous state updates for distant
players. News selection is consequently age-aware: as an item becomes older
by the time it reaches a local market or audience, it must be increasingly
important to qualify for a local headline. Headline selection should account
for both event significance and information age, while leaving the exact
scoring and threshold policy configurable.

News propagation is not the same as event visibility. A newly materialized BBS
creates a polity, capital, and political actor, so its founding/discovery is a
high-significance institutional event that becomes headline-level news at any
distance once carried through the physical mail network. It is not
instantaneous: established polities receive it quickly, frontier regions more
slowly, and isolated regions may not receive it until a ship carries the
announcement. Every newly discovered system produces local Known Universe
observations. On arrival, a captain who does not know it to be publicly mapped
is prompted to broadcast a free public notification, send a paid encrypted
direct filing to Earth, withhold it, or withhold it and add the system to a
captain-private Secret Systems list. A public package propagates from the
ship's system. A direct package remains opaque to intermediate repositories;
if it wins the bounty, Earth originates the public announcement at award time.
A routine uninhabited-system discovery normally creates no visible public
headline: default Message Management filters hide the low-significance public
notice, but they do not block ingestion. A settlement, resource site,
strategic base, or witnessed pirate incident can generate regional news;
publicity, evidence, target importance, and political consequences determine
whether that incident escalates into a warrant or wider headline.

The Federation makes exploration a minor cost-positive activity through a
standing award for the first valid notice of a newly discovered settled
system. The structured discovery package doubles as the filing. Priority is
the order in which valid filings commit to the Federation repository on Earth,
not observation or dispatch time and not receipt at a nearer Federation
system. Earth pays when it accepts the filing; the explorer need not return,
but the credit exists at Earth and its receipt and transfer still travel by
physical mail. The published award is initially 110% of the ordinary total
cost of two complete Jump-2 legs in the orderly/mixed privateer starter
(`ship-72`, Smollett), rounded up to Cr1,000. Include the reference offer's
fuel, normal four-week payroll/life support, maintenance, navigation data, and
ordinary port costs. Exclude mortgage, interest, charter payments, debt
service, depreciation, every other capital/financing cost, exceptional damage,
ammunition, bribes, fines, and cargo capital. Exact credits await the direct
operating-cost audit. Full rules are in
`docs/settlement-and-system-survey.md`.

### Mail delivery and message-store architecture

#### Beacon-mediated common carriage

Maintained routes have standard mail beacons at their useful Jump departure
and arrival loci. A ship configured to carry mail authenticates at departure,
selects or confirms its immediate destination, and downloads a signed,
encrypted destination mailbag. On arrival it uploads the mailbag to the local
beacon; validation records the completed carrier leg and automatically issues
the advertised stipend. This transfer should normally occur without cargo
handling or a port visit.

The beacon network is routing, custody, and clearing infrastructure, not a
separate fleet. Its stipend is a token handling payment to a ship already
making the exact hop. The hop price is constant: route scarcity, desired
frequency, urgency, danger, and recent departures do not change it. There are
no electronic-mail service guarantees, route subsidies, charters, or dedicated
mail transits. If no suitable ship is going, the mail waits. Anything important
enough to justify a voyage becomes a passenger, physical cargo, or explicit
contract rather than ordinary electronic mail.

Electronic mail may be copied to several authorized carriers for resilience.
Custody and payment are nevertheless idempotent by mailbag, carrier, and leg.
The reward policy must state whether the first timely delivery receives the
main stipend and whether commissioned redundant copies receive a smaller
guaranteed payment; a ship may not repeatedly upload the same copy for more
credit. Beacons sign departure receipts and arrival acknowledgements, and
mailbags authenticate their contents and intended leg.

Automatic credit at arrival does not create an ansible or universal live bank
balance. The destination mail authority pays locally or issues a locally
redeemable signed credit; institutional clearing and reimbursement propagate
later through the same mail system. Beacon failure, isolation, or destruction
can require a later port handoff and can delay both delivery and payment.

Merely passing a beacon has no effect. The ship must accept a mailbag and
create a signed custody record. Once accepted, loss, diversion, lateness, and
delivery are authoritative consequences. Carrying routine mail should be a
configurable ship default rather than repetitive player busywork, while
high-security passengers, physical packages, and special courier work remain
explicit contracts.

Mail distribution is event-driven, not a periodic sweep. A message is retained
in an immutable archive with a stable message ID, origin system, event time,
dispatch/issue time, type, payload, propagation policy, and absolute expiry
time when applicable. Dispatch creates a persistent delivery plan for the first
mail leg. Each leg has a due game-time and a destination (relay, polity hub, or
final system); scheduled departures use the known route timetable and
opportunistic carriers use the carrier/route decision made when the mail
leaves. The plan records route and carrier history without changing the
message itself. A beacon pickup commits that carrier decision and enqueues the
expected arrival; authorized redundant copies use distinct carrier-leg
records pointing to the same immutable message or mailbag.

The delivery plan is indexed by due time in a persistent priority queue. When
the simulator advances from time `T1` to `T2`, it drains due mail events in
chronological order and enqueues only the next hop for each delivered item.
There is no hourly or daily polling pass, and entering a system never scans all
messages that are still in flight. A due event appends the message (or a compact
mailbag batch) to a system/relay feed indexed by `(system_id, available_at,
feed_sequence)`, then schedules any onward legs. A player arrival reads that
already-materialized feed by system and cursor. The same ordered simulator
mailbox owns these delivery events, so a command cannot observe a clock in which
an already-due delivery has been skipped.

Propagation fan-out is bounded by the mail graph rather than by every stellar
system. Normal mail follows explicit carrier legs and branches only at actual
mail depots or correspondents. Major news uses a route-scope/TTL and may be
represented as a shared mailbag or route-frontier event; it is expanded at
scheduled relay times, with per-system feed rows materialized only for
mail-connected/populated endpoints. This makes broad news prompt without
requiring a message-by-system bitmask or a synchronous write for every empty
system. The authoritative event remains queryable even when a delivery is
expired; expiry prevents new delivery or acceptance but does not erase history.
All message records are retained indefinitely unless a later archival policy is
explicitly adopted. Where the design says **TTL**, it means a time-to-live: use
an absolute `expires_at` in game time, not a hop count.

A validated universal broadcast has one monotonic `universally_seen` bit.
Before completion it retains only its sparse live route frontier and pending
exceptions. Once it is available in every currently applicable public
repository, one transaction sets the bit and removes completed per-system
propagation rows. Public mail follows discovery automatically: a repository
created for a later-discovered system starts at the completed universal-feed
checkpoint and can access the immutable universal archive, so a completed
message never becomes incomplete again. A still-propagating broadcast adds a
newly discovered applicable system to its live frontier. This is repository
availability, not proof that any player read the message, and it does not apply
to regional, polity-scoped, local, contract, private, or mobile-sphere mail.

Delivery records are separate from player state. An arrival packet is the full
set of non-expired messages already available in that system, filtered for the
player's classifications before presentation; every previously unseen item is
eligible for the player's default/unreviewed filter. A per-player, per-system
feed cursor tells which feed sequence has been presented there. Do not infer a
universal “all messages to this player have been seen” watermark: a local-only
message in an unvisited system makes that definition meaningless. Sparse
per-player message rows hold cross-system classifications such as ignore,
review, pinned, or actioned. At the expected population scale this is simple
and cheap, while avoiding a message-by-system bitmask or a dense
player-by-system-by-message table; profile before adding compressed bitmaps.
Contract acceptance, payment, and other side effects are authoritative commands
against the contract record, not properties of a copied mail item.

Sender tariffs distinguish four services. Actual news accepted by a news
agency, validated public-service broadcasts, and constrained public-key
distribution are free to the sender. Private and other non-public-service
messages have a small deterministic dispatch charge based on payload class,
absolute game-time TTL, and delivery plan. A system-addressed message buys one
exact known route and pays only its per-hop charges. A captain-, ship-, or
other mobile-addressed message buys a replicated encrypted hold sphere; every
covered system retains a copy until authenticated delivery, a later receipt,
or expiry, and the sender pays for the full fan-out and TTL. Copies cannot be
cancelled instantaneously, so stable message IDs make repeat delivery and all
side effects idempotent. Full rules are in
`docs/mail-service-and-security.md`.

Non-public payloads use end-to-end encryption and authenticated signatures.
Public keys and signed bindings/revocations are free public data. Public keys
are not secrets and cannot decrypt messages; the capturable gameplay assets
are a captain or ship's private signing/decryption keys and credential stores.
When capture becomes known, law enforcement or the issuer emits a signed
revocation, but each remote institution honors it only after physical mail
arrives. A compromised key may therefore decrypt or impersonate in regions
with stale key state. Relays cannot re-encrypt old ciphertext to a replacement
key, and revocation cannot undo plaintext or signatures already accepted.

Population and traffic are separate from stellar existence. Generated systems
may be uninhabited, resource sites, outposts/bases, settlements, or polity
hubs. Trade, mail, passenger demand, and contracts should emerge from those
activity tiers and sustained routes; an empty system should not receive a
market or mail feed merely because it exists. A player may nevertheless visit
for fuel, salvage, strategic operations, survey work, or the opportunity to
seed a new route or settlement.

The initial steady-state cargo calibration is specified in
`docs/system-traffic-and-encounters.md`. It uses actual population, a
technology-weighted productive population, sublinear scaling, and a
realized-trade factor to create directed route flows. Trade codes allocate
commodity supply and demand; explicit facilities add non-duplicated deltas.
This is a hybrid aggregate model because CE and the supplements provide cargo
availability and explicit station production but no conserved world-wide
industrial capacity and consumption model.

Traffic below 0.5 calls per week on a directed route is an actual persistent
schedule, not a probability rerolled for each visitor. Rates from 0.5 through
5 calls per week retain a near-term schedule and aggregate their distant
future; higher routine rates may remain aggregate until consequential.
Hysteresis is required. Port-level conspicuousness and encounters use the
union of its route schedules. Persist lightweight traffic calls and cargo lots
without materializing a complete NPC ship until observation or consequences
require it. Player and background loading draw from the same inventory.
Scheduled background mail handoffs remain authoritative due-time events;
they may not wait for a later player market query.

Every materialized star system has a durable `SystemDay` job for every game
day. The job advances aggregate production, consumption, inventory aging,
offers, traffic schedules, mailbag preparation and incentives, piracy,
enforcement, facilities, and other mutable local activity. It creates
individual persistent records only for consequential outcomes. Store only
each system's next daily job; processing it advances `last_processed_day` and
schedules the next. Empty systems retain the logical heartbeat but normally
take a cheap no-activity path. Derived celestial positions do not need a tick.

Player commands and due scheduled events use one serialized authoritative
input queue. Future-event indexes establish eligibility only. While ingress is
empty, the scheduler advances logical time directly in a durable journalled
transaction; future work remains in its due-time index during that commit. A
following transaction admits the now-due timestamp-free payload, and the
ordinary queue consumer executes it later. Once admitted, queue sequence is
the entire execution order; event kind and entity ID never reorder inputs.
Simultaneously eligible future work uses its global creation ID as a stable
admission order, not a semantic priority. Dependencies must be causal: commit
the prerequisite before scheduling its consequence.

The daily checkpoint is not the last player arrival. Passing through, viewing
navigation data, or using a remote refuelling point does not consume cargo or
move mail. Market and mail state changes only through an authoritative player
action or scheduled background event such as production, reservation, loading,
release, handoff, delivery, theft, destruction, consumption, diversion, or
expiry. Persist player consequences, structural changes, and already exposed
facts. Daily RNG is recoverable and query-count-independent; a read cannot
reroll a day or a fact already shown to a player.

### Background simulation and structural events

The server maintains a logical daily heartbeat per materialized system, but
not a live per-NPC simulation. Most purchases, ordinary contracts, wars,
patrol activity, population changes, facility damage, and cargo movement
remain aggregate statistical state within that daily job. Materialize
individual ships, cargo lots, or actors only when a player, an encounter, a
contract, a legal record, a schedule, or a news event makes them relevant.
Market and facility state may be a persisted baseline plus daily and
time-stamped deltas.

Large events that affect the structure of the world are different. Maintain a
low-volume, persistent structural-event schedule with entries such as a major
pirate offensive, war, polity collapse or succession, infrastructure failure,
embargo, or population movement. Schedule these at the polity, route, or region
level rather than one event per star system. When simulation time advances, the
authoritative simulator processes due structural events forward and in
chronological order. A structural event performs an aggregate state transition
(for example, splitting a polity or closing a route) and immediately emits its
news/warrant/mail-propagation work; it is never reconstructed backwards when a
player happens to observe its consequences.

Catch-up work is checkpointed and resumable. It normally processes overdue
logical days in bounded loops; an aggregate shortcut is allowed only when it
preserves the same durable consequences, RNG advancement, and intermediate
scheduled events as the daily sequence. Structural events remain individually
ordered. Broad news uses the mail route-frontier/mailbag mechanism above
rather than synchronously creating rows for every empty system.

### Authoritative data domains

Keep the following domains separate in storage and in the simulation API:

- **Generated universe:** system identifiers and three-dimensional coordinates,
  persisted per-system seeds and generation versions, stellar/planetary/orbital
  data, CE UWP/system descriptors, and surveyed empty volume. Generated
  celestial data is stable; it is not rerolled on visit.
- **Mutable places and institutions:** populated-system tier, starports and
  facilities, polity membership and borders, law/security, ownership, route
  connectivity, damage, construction, and other persistent world state.
- **People and assets:** player identity/origin/moderation state, characters
  and operational skills, ships and ownership/debt, crew, cargo, passengers,
  supplies, fuel, damage, repairs, encounters, and current phase.
- **Obligations and information:** contracts/offers, messages/news, warrants,
  banking instruments and settlement records, mail delivery plans and feeds,
  player message cursors/classifications, and audit records.
- **Simulation control:** the authoritative game clock, ordered ingress
  sequence, session epochs, future-due mail and structural events, generation
  migrations, and resumable catch-up checkpoints.

Economy state is primarily statistical rather than a physical inventory of
every planetary item. Store the aggregate market/facility baseline and the
player-relevant deltas; materialize individual cargo or NPC activity only when
the rules need an identifiable object. Derived indexes such as system feeds,
route tables, and market snapshots may be rebuilt from authoritative records
and must never become a second source of truth.

### Simulation time and frames

The implemented baseline simulation clock advances continuously while the
server process is running at exactly 28 game seconds per real second: four game
weeks per real day. Server downtime is frozen game time. On startup the server
anchors a monotonic process clock at the last committed game second; it does
not persist or consult a wall timestamp and therefore performs no restart
catch-up. Changing this fixed rate requires an explicit versioned clock-format
migration rather than silently changing a configuration value. The design
uses nested time frames with deliberately different compression and
observability:

1. **Jump time** — the overall universe clock and interstellar travel. A CE
   Jump is compressed to a playable duration rather than requiring a literal
   week of real time.
2. **Interplanetary time** — in-system maneuvering, fuel skimming, and travel
   between orbital locations or worlds.
3. **Encounter time** — local port activity, trading, and combat.

These frames are not equally observable. Entering an encounter establishes a
local frame in which the participants are unaware of unrelated activity in
the rest of the universe. The encounter resolves in its own time scale; when
it ends, participants return to their previous frame of reference and receive
a news update describing relevant events that occurred during the isolated
frame. The server remains authoritative across all frames and must reconcile
state transitions without leaking out-of-frame information.

Jump is an asynchronous ship lock rather than a period for which the player
must remain connected. Under the approximate one-real-day/one-game-month
compression, a one-game-week Jump should normally occupy several real hours
and may complete overnight. While in Jump, the UI remains usable in **planning
mode**: the player may review cached information, annotate and triage it,
organize contracts, compare routes, draft manifests, configure filters, and
prepare arrival actions. The player cannot accept new offers, buy cargo, alter
the ship, or react to fresh local information until arrival and the next data
packet. There is no ansible; a connected UI exposes cached ship information,
not live distant state. Routine conditional policies may be drafted, but
commitments and risk decisions require current-state confirmation after
arrival.

Each BBS installation represents one home system. Its home system contains a
planet with the highest Tech Level in that polity and belongs to a small local
polity of roughly ten systems. During BBS creation, the sysop chooses the
local region's civilization profile on a continuous combat-focused to
trade-focused axis. At the combat extreme, naval service and military
protection are central to survival; at the trade extreme, banditry is uncommon
and generally remote. This profile changes local generation, encounters,
security, and economic conditions; it must not fence the player into one
playstyle. The generated universe must always provide a viable route across
the spectrum, even if crossing it takes many months of **game time**. This is
simulated time, not a requirement that a player remain connected for months of
real time.

The BBS sysop is the final moderator for the polity and for players whose
origin is that BBS. The sysop may directly modify local polity settings and
may demote, tax, suspend, or remove originating players at any time. These are
administrative actions, not ordinary in-game abilities. A sysop must not be
able to boost their own players, create credits/cargo/ships, waive costs,
change combat outcomes, or otherwise grant an in-game advantage. The server
must enforce this separation, record moderation actions in an audit trail, and
make destructive actions recoverable or reviewable where practical.

The civilization-profile spectrum must support a strong middle role: the
**privateer**. A privateer operates between formal naval service and ordinary
merchant trading, combining letters-of-marque or polity contracts, convoy
escort, commerce raiding against authorized targets, bounty work, captured
cargo, and legitimate trade. Privateer play must be viable and coherent at
the midpoint of the profile, not merely a weakened navy or a pirate with a
different label. The game must distinguish authorized privateering from
unlicensed piracy through polity status, rules of engagement, targets,
rewards, and consequences.

Combat intensity and legal status are separate dimensions. A combat-heavy
region can contain formal navies, privateers, mercenaries, insurgents, and
pirates; a trade-heavy region can still have organized security forces and
criminal raiders. Navy and pirate play may share the same tactical combat
verbs, but they must differ operationally in authority, mission source,
authorized targets, identification/transponders, access to bases and
intelligence, logistics, income, and consequences. Law level modifies
detection, enforcement, and penalties; it is not the only distinction between
lawful and unlawful combatants.

Law level is a Cepheus Engine world/polity parameter under sysop control. The
sysop may change it as an administrative world-state decision; changes should
be auditable and should affect detection, enforcement, permissions, and
penalties rather than directly minting player resources or altering combat
outcomes.

### Starting career offers

The starting-player design is maintained in
`docs/starting-player-design.md`. Its current model is provisional: each BBS
polity has independent trade-to-combat and chaos-to-order orientation values.
For startup, each dimension maps to one of three bands, producing a 3 × 3
catalog. Every one of the nine cells has exactly three predesigned offers: a
trader, a privateer, and a navy/public-service command. The player sees only
the three ships in the home BBS's cell. These are 27 designs and obligation
packages, though variants may share a base hull. Institutional order is
distinct from CE Law Level; the former describes reliable title, banking,
command, contracts, and prize adjudication, while the latter remains a world
characteristic governing restrictions and enforcement.

The locally aligned career has the strongest expected starting prospects:
trade-focused favors the trader, mixed favors the privateer, and
combat-focused favors navy/public service. Compare total local prospects
rather than sticker price: ship efficiency, work, financing, support, legal
authority, and obligations all count. Mismatched careers remain viable but
pay an explicit environmental cost. Order/chaos changes how those advantages
and risks are expressed; it does not simply add raw starting value.

Starter balance is based on practical agency and expected progression rather
than nominal hull price. Ship-use authority, player equity, liquid capital,
institutional support, legal authority, and obligations are distinct. A navy
captain does not own the assigned warship, and a privateer's access to an
expensive armed vessel does not make it immediately saleable as a debt-free
asset.

Starter variants derived from Clement ships use the minimum CE Jump-2
installation. When the actual source drive is oversized, the recovered
displacement is reserved for starting customization and the drive saving
remains package/refit value rather than becoming player cash. The current
Anderson & Felix chart normally recovers 10 tons and MCr20 on 500–2,000-ton
hulls, but older published designs vary; compare the actual stat block. Ships
without recoverable drive space need an explicit configurable-space
reservation where desired.

Every offer supplies a ready-to-depart default ship and valid minimum crew,
then separates non-cash refit allowance, continuing staffing envelope, and
liquid operating reserve. Offers and the setup revision are generated once
and persisted so reconnecting or recreating a character cannot reroll or farm
assets. After choosing the ship, the player reviews each fixed initial-crew
role template and supplies the crew member's name and training target. Startup
shows a time-stamped home-polity dossier and only the neighborhood knowledge available
through local observation and physical mail. Reachability and likely first
opportunities are previewed for each offered ship without revealing unknown
systems or current distant state.

The canonical construction boundary is implemented in
`catalog/shipbuilding/`, `catalog/ships/`, and `tools/ship_design.py`.
Shipbuilding mechanics come only from the applicable construction rules. A
published ship specification is a validation target and may provide
assertions, but it must never create component definitions or override a
rule's price, displacement, performance, or TL. Unknown equipment blocks the
design until its construction rule is found and curated.

A construction record represents a real rule concept or table row. Quantity
belongs to a design: 29 sandcaster-canister piles are not 29 component types,
and a fuel processor remains one per-ton rule. Mounts contain weapon
instances. Use integer credits and millitons, require exact volume accounting,
and reject synthetic displacement deltas, source-pricing adjustments,
unparsed descriptions, and filler components.

`catalog/ships/` contains hand-authored bills of materials. It must not be
regenerated from `Ships.ods`, PDFs, or ship-description prose. Published
summaries are checked after reconstruction and may reveal errors or omitted
items; they are not assumed correct. Only the rule-derived construction
catalog and admitted hand-authored designs are game inputs. Reconstruct new
entries against those rules before admitting them.

Each eventual catalog design remains an immutable, fully fitted template with
standard loadout, carried craft, crew establishment, functional description,
and licensing provenance; it is not a mutable ship instance. Active entries
will have two or three original plain-text paragraphs explaining intended
role, concrete strengths, and meaningful limitations. Never copy publisher
descriptions or embed Markdown/HTML. Every admitted entry retains all
applicable source bundles from `catalog/ogl-sources.toml`.

The conversion of all 191 reserved Clement/Earth inventory identities
(`ship-1` through `ship-191`) is complete. Each identity is an admitted,
rule-derived design or, for the source that declared no Open Game Content, an
independently authored replacement. Supplemental/core designs begin at
`ship-192`. Future catalog changes must preserve the permanent identity
mapping, exact displacement validation, PI-free presentation, and current
`docs/ship-catalog-conversion-status.md` and compiled OGL attribution.

### Interstellar law, warrants, and banking

Crime and finance must work across polity borders. A crime should produce a
jurisdictional record that can include the offender's identity, ship, offense,
evidence, bounty, issuing polity, diplomatic standing, and expiry or review
conditions. Warrants and crime reports travel through the physical-mail
network, so a destination may have incomplete or stale information when a
fugitive arrives. Local authorities should weigh the issuing polity's
legitimacy and relationship with the local polity rather than treating every
foreign warrant as universally authoritative.

Local corruption is a deliberate variable. It can reduce investigation
quality, make bribes or payoffs possible, change which warrants are acted on,
and affect the risk of confiscation or release. Corruption must create
tradeoffs and uncertainty, not a guaranteed purchase of immunity.

The interstellar banking system should make travel and trade practical while
creating consequences for lawbreaking. Candidate mechanisms include polity
bank accounts, delayed settlement, letters of credit, ship mortgages,
identity/KYC records, account freezes, asset seizure, foreign exchange, and
black-market finance. Banking access, warrant status, diplomatic relations,
mail latency, and local corruption should interact without making one
universal bank an ansible-like source of instant information.

Banks may maintain secure, private courier networks separate from ordinary
merchant mail. The model is closer to stagecoach-era correspondent banking
than to a modern instant-transfer service: competing banking houses, branch
agents, correspondent accounts, bills of exchange, letters of credit, bearer
drafts, and protected mail ships move value between locations. Couriers and
bank branches can be robbed, delayed, blockaded, or captured, and powerful
banking houses may exert political influence like robber barons.

Players see available instruments, fees, route coverage, delivery estimates,
and settlement status, but not the bank's internal ledgers, netting, courier
assignments, or correspondent negotiations. Secure banking mail can be more
reliable or faster than ordinary mail, but it still has routes, capacity,
delays, outages, and jurisdictional limits; it is not instantaneous
communication.

Taken together, no ansible plus Jump-drive transit creates a broadly
nineteenth-century-like communications and economic topology at interstellar
scale: large population centres and resource sources anchor regional power,
information is delayed, local authorities have substantial autonomy, and
private courier, banking, shipping, and security networks matter. This is a
design analogy, not a requirement to copy nineteenth-century institutions or
technology; CE tech levels, species, ships, and the 3D galactic geography
remain the setting's own.

### Session and action budget

Design for a daily useful-action budget rather than continuous play. The
normal session target is **15–30 minutes**, and a player should be able to
make meaningful progress with one 30-minute session per day. Useful action
should generally taper by about **45 minutes per day**; the game must not make
longer attendance mandatory for competitiveness. A player may receive a small
bonus for using up to that longer budget or for splitting play into multiple
5–15 minute sessions, but the bonus must not outweigh the value of a reliable
daily 30-minute session. Avoid chores, waiting, or artificial click volume as
ways to fill the budget.

### Candidate in-system activity surface

The following activities are supported by the CE travel, trade, ship, world,
and starship-encounter material and are candidates for the normal session
interface. They should be exposed as meaningful decisions, with CE-derived
skill effects retained where they materially affect combat, trade, or ship
operations rather than as a sequence of irrelevant RPG rolls.

### Trader UI loop

On entering a system, the ship receives a local data packet containing news,
contracts, offers, dangers, sightings, and other reports available through
local observation and delayed mail. Each item carries age, source, confidence,
and propagation context. The player can filter the packet and rapidly triage
items into **ignore**, **review later**, and **unreviewed** buckets. A compact
keyboard interaction may map left to ignore, right to review later, down to
advance, and Enter to take the item action (accept, reject, claim, inspect,
or open the relevant order).

The player then chooses an in-system destination—port, planet, orbital site,
gas giant, frontier fuel point, or Jump point—and handles transit, traffic
control, and ship encounters. Movement uses a committed, bounded continuation
plan. Facility departure, convergence areas, destination approaches, and
terminal operations are authoritative encounter/readiness checkpoints, but
they are not mandatory UI stops: uneventful steps continue when already
authorized. At a port or planet, declared cargo may be
physically offloaded automatically, but receipts, contract settlement,
inspection, illicit/concealed cargo, shortages, and payment remain explicit.
The captain's office/exchange presents maintenance, refit, crew, banking,
speculative and illicit trade, passenger, mail, charter, and contract choices.
The player confirms the departure manifest, including reserved obligations,
then builds a continuation plan. That plan may end at the Jump point or may
preauthorize a specified Jump using purchased navigation data, player
calculation, or a visible configured default. An uneventful through plan
initiates the Jump automatically; an encounter or failed validation suspends
it for continuation or replanning. Committing the Jump locks the ship into
asynchronous planning mode until arrival. The same plan semantics allow
preauthorized docking, landing, skimming, rendezvous, and other bounded
terminal operations. Full semantics are in
`docs/interplanetary-operations.md`.

**Core activities**

- inspect the system, mainworld, starport services, local law/security,
  bases, hazards, news, and available mail;
- move between the mainworld, orbital port, jump point, gas giants, planetoid
  belts, and other generated in-system locations;
- buy fuel, skim or process fuel where legal and equipped, replenish life
  support, pay berth/port charges, and perform routine maintenance;
- accept bulk freight, passengers, physical-parcel and special-courier
  contracts, and configure routine beacon-mediated mail carriage;
- find suppliers or buyers and buy/sell speculative cargo, including legal or
  black-market opportunities where local law and security permit them;
- use brokers or port services, compare routes and prices, and choose when to
  depart rather than waiting for every possible offer;
- repair, refit, upgrade, insure, finance, or sell a ship when an appropriate
  yard or service is available;
- communicate: read local news, send ship-carried electronic or physical mail,
  receive delayed reports, and decide what exceptional information or
  contracts to carry onward;
- respond to encounters by scanning, hailing, evading, docking, rescuing,
  boarding, fighting, surrendering, or resolving a distress signal.

### Fuel as an in-system tradeoff

Refueling should be a meaningful convenience, cost, and maintenance choice,
not a single button. CE provides three practical source methods, plus an
onboard conversion step:

- **Port fuel:** refined fuel is the fastest and safest option, but costs
  Cr500/ton at A/B ports (plus Cr100/ton if ferried to the ship). Unrefined
  fuel is cheaper at A/B/C ports, but carries the normal unrefined-fuel drive
  risk until processed.
- **Surface water or ice:** on a world with hydrographics 1+, a suitable ship
  may land near an open body of water or ice and pump free unrefined fuel.
  Landing, local law, exposure, and atmospheric operations make this a real
  choice rather than universally available free fuel.
- **Gas-giant skimming:** a ship with fuel scoops, or a larger ship using a
  streamlined fuel shuttle, can gather free unrefined hydrogen. CE takes 1D6
  hours per 40 tons and identifies gas giants as common pirate ambush points.
- **Processing:** fuel processors convert unrefined fuel to refined fuel at a
  rate determined by installed equipment, consuming time and ship capacity;
  the captain explicitly chooses whether to process a collected batch and may
  later process unrefined fuel while docked or safely holding. CE describes
  refined fuel as reducing drive problems, but its explicit resolution rule is
  the Jump DM. Cepheus Trader therefore applies no separate unrefined-fuel roll
  to ordinary power-plant consumption.

The Clement expansion adds the failure model that makes frontier refueling a
real risk: a Difficult Pilot/DEX task for gas-giant entry can cause 1D6 ship
damage on failure or 3D6 on exceptional failure; a Difficult water-landing task
has the same damage consequences. Refining is an Average Engineer (Power)/EDU
task; failure doubles the time, while Effect -6 or worse damages the Jump drive,
falling back to the maneuver drive and then fuel system if necessary. These
are supplemental adaptations, not direct CE core rules. Their original damage
text is written for the Zimm drive, so Cepheus Trader must translate it to
standard CE machinery. That refining translation is implemented; the separate
gas-entry and water-landing damage checks remain deferred.

Streamlined hulls include scoops and are better at atmospheric operations;
standard hulls can perform them with difficulty and distributed hulls cannot
mount scoops. The computer should compare total delivered cost, time, route
detour, exposure, piracy risk, processor capacity, and drive reliability for
each option. A captain may buy refined fuel to protect a deadline, skim at a
gas giant to preserve cash, or use a surface source in a frontier system when
the extra operational risk is acceptable. See [Off-World Travel](cepodnew-markdown/06-off-world-travel.md#fuel)
and [Fuel Scoops](cepodnew-markdown/08-ship-design-and-construction.md#fuel-scoops).

A distributed hull can carry dedicated skimming boats without enclosing them
in full hangars. External docking clamps or rugged cradles can carry the boats
through Jump and provide power, data, fuel hoses, and boarding umbilicals;
full hangars are primarily an onboard repair and maintenance capability. The
boats remain non-Jump craft and operate in parallel, so a small flotilla can
refill a large ship in roughly the same active skimming time as a direct
streamlined hull, provided the parent ship has enough processing capacity.
External craft still impose loaded-mass penalties on maneuver performance and
Jump fuel/drive sizing, and are more exposed to combat damage. Their mass and
the docking system must be included consistently in the standard-drive
calculation. A docking clamp alone does not provide fuel transfer; use UNREP or
an equivalent transfer system. A 3,000-ton distributed ship carrying several
40-ton skimmers is therefore a logistics tradeoff, not a refueling dead end.

The economy does not price energy as a separate commodity: a ship's power
plant already represents the energy system, and processing water or ice does
not create an additional fuel bill. Therefore water or ice should generally be
the cheapest frontier source per ton when it is accessible. Its real costs are
world availability, collection hardware, landing or atmospheric risk, local
law, processing time, ship wear, and the opportunity cost of remaining in the
system. Gas-giant skimming remains valuable when no accessible water exists,
surface operations are prohibited or dangerous, or the ship can scoop without
an expensive detour.

The consulted sources do **not** provide a separate numeric per-tonne
surface-to-orbit or fuel-tanker tariff. CE's closest figure is the **Cr100/ton
surcharge when fuel must be ferried out to a ship**. *Port of Entry* describes
downport tankers, highport hookups, and space-elevator logistics but refers
back to the Clement fuel formula without adding an orbital delivery rate.
Anderson & Felix gives an UNREP transfer capacity of 20 tons per hour per
installed ton and an installation cost of MCr0.5 per ton, not a service price.
Any orbit/downport surcharge in Cepheus Trader is therefore a labelled game
adaptation, not a CE fact.

CE's interplanetary model also does not price delta-v or acceleration
propellant: the M-Drive changes travel time, while power-plant fuel is sized by
plant tonnage and operating weeks. We should therefore avoid inventing a large
mass-proportional “energy to orbit” charge. Surface-to-orbit fueling should be
priced primarily through equipment, labor, transfer throughput, port capacity,
service time, and operational risk; a per-ton handling fee is acceptable as a
port tariff, but it should not pretend to represent an absent propellant cost.

### Commercial fuel-range baseline

The standard CE designs generally carry **one Jump plus four weeks of power-
plant fuel**, so refueling is expected at each normal destination. For
example, the TL9 Frontier Trader carries 42 tons: 12 tons for four weeks of
its C power plant and 30 tons for one Jump-1. The 400-ton Merchant Freighter
carries 48 tons: 8 tons for four weeks of its B plant and 40 tons for one
Jump-1. The 200-ton Merchant Trader carries 24 tons: 4 tons plus 20 tons for
one Jump-1. The TL11 Survey Vessel is an exception that carries 72 tons for
four weeks and two Jump-1 jumps.

Extra jumps therefore require deliberately enlarged fuel tankage, a fuel stop,
or a design with unusual range. A normal merchant itinerary should treat
“arrive with the previous load → refuel or frontier-source fuel → reload and
depart” as the ordinary port rhythm, not as an emergency state.

Two carried Jump-1 allocations consume the same 20% of hull displacement as
one maximum Jump-2 allocation, but the Jump-1 drive is smaller and cheaper and
may permit a smaller power plant. The price of that capacity advantage is two
Jump-success resolutions, roughly two weeks in Jump, and a required one-day
empty-space turnaround. Jump-2 is therefore a speed, reliability, and route-
flexibility purchase rather than a fuel-volume saving.

**Optional depth**

- land or launch from a world, use surface facilities, and choose wilderness
  versus controlled starport operations;
- smuggling, customs/law enforcement, piracy, salvage, and security actions;
- ship security and cyber-defense configuration;
- crew and passenger management, provided it remains an operational
  simulation; retain relevant crew skills as operational modifiers without
  importing the full character-career system.

The first playable in-system loop should likely be: arrive, inspect news and
market, service/refuel, choose cargo or contracts, optionally resolve an
encounter, and depart or remain for another local opportunity. Time costs for
each action should be measured against the 15–30 minute daily target.

### Candidate naval-captain activity surface

Combat-focused regions need a complete command loop, not just random fights.
The following activities combine CE starship-combat/encounter actions with
deliberate game-level naval adaptations:

- receive or select patrol, escort, interception, reconnaissance, customs,
  anti-piracy, rescue, convoy, and base-defense assignments;
- plan a patrol or response route through the 3D system map, choosing fuel,
  ammunition, maintenance windows, rules of engagement, and risk;
- scan and classify contacts, monitor sensor/electronic intelligence, identify
  unknown vessels, hail or challenge them, and decide whether to shadow,
  divert, inspect, escort, or engage;
- escort merchant, passenger, and mail traffic; protect a convoy; respond to
  distress calls; and coordinate with bases or nearby patrol groups;
- intercept, pursue, evade, break pursuit, blockade, quarantine, or enforce
  customs where the polity authorizes those actions;
- command combat decisions: initiative, range, speed, positioning, attacks,
  electronic warfare, evasive maneuvers, point defense, damage control,
  boarding, capture, surrender, and withdrawal;
- coordinate crew stations and allied vessels, including automated positions,
  communications, and task-group decisions;
- manage readiness between contacts: refuel, rearm, repair, resupply, rotate
  crew, review damage, and return to a base or continue the patrol;
- resolve the aftermath: rescue survivors, secure prizes or evidence, report
  intelligence, update local news, claim rewards, and accept consequences for
  collateral damage or a missed objective.

Routine naval play should offer meaningful choices even when no battle occurs:
where to patrol, what to investigate, whom to protect, when to withdraw, and
which risks to accept. Blockades, fleet command, and large-scale strategic
operations are optional later layers rather than prerequisites for the first
naval loop.

Ship combat uses CE's one-kilosecond turns and initiative-ordered vessel
activations. A vessel submits one atomic joint crew order containing the legal
minor/significant actions for its named crew and aggregate station teams plus
prioritized standing reactions. An online player starts with the
most-conservative complete legal order and may change any crew action before
committing it. A separate classic, rules-based tactical controller uses a
player-selected minimum estimated probability of satisfying the current
encounter objective. It searches only the censored state available to the
captain, may be invoked online, and takes real withdrawal actions before
considering surrender or escape craft. It is not an LLM and receives no
hidden-state or rules bonus.

After an offline-controlled encounter, a crew retaining control automatically
attempts all feasible onboard recovery. Temporary battle-patch coverage
expires first. Capability priority is Life Support, Maneuver drive, Jump drive,
then weapons, with hull, power, control, fuel, or bridge prerequisites charged
to the higher-priority goal. Real skill, watches, tools, spares, supplies,
access, and time constrain the work; destroyed or facility-dependent systems
remain damaged. Full semantics are in
[`docs/combat-control-and-automation.md`](docs/combat-control-and-automation.md)
and [`docs/ship-condition-and-maintenance.md`](docs/ship-condition-and-maintenance.md).

Combat can attract third parties, but it never broadcasts authoritative truth
instantaneously. Each detectable emission or distress transmission reaches an
observer after `separation / c`; sensors then produce delayed and possibly
ambiguous evidence. A responding ship must execute a real intercept and joins
only at its physical arrival time. If the battle has ended, it may instead
reach pursuit, rescue, arrest, salvage, or aftermath. Online captains choose a
response and offline captains use persistent intervention policies covering
authority, allies, aggressor-confidence and risk thresholds, and diversion
limits. Enforcement vessels generally investigate incidents in their remit.
Other computer-controlled ships generally join only when well armed relative
to the participants and the aggressor is clear, subject to allegiance, law,
mission, crew, damage, fuel, and ammunition.

The agreed historical captain loop is: **receive orders → prepare the ship →
choose a route or patrol → gather information → classify contacts → decide
whether to intervene → resolve the encounter → repair/resupply → report and
receive new orders**. This is the naval analogue to the merchant arrival,
market, service, trade, and departure loop.

### Naval wealth and post-service paths

Naval prize or mission wealth must have useful destinations even though a
state-issued warship is not purchased personally. A wealthy naval captain may
remain in service for rank, authority, prestige, and access to missions; retire
on half-pay or a pension; buy or finance a civilian ship; become a merchant or
privateer; invest in shipping, banking, or resource ventures; or fund political,
intelligence, and expeditionary influence. These paths should be optional
strategic transitions, not mandatory RPG careers. Personal wealth must not
directly buy better combat outcomes for a state ship, but it may support
independent assets and lawful ventures after the captain leaves or acts outside
official service.

In a combat-heavy polity, the sysop may configure naval service as the normal
entry path for a player who wants to command a privately owned armed
merchantman. The legitimate path is to serve, earn rank and wealth, and leave
or transfer with enough capital and standing to acquire a vessel. The illicit
alternative is mutiny or ship theft, which must be possible only with major
consequences: warrants, pursuit, loss of lawful status, frozen or seized
assets, hostile ports, crew/faction reactions, and a lasting reputation as a
criminal. This is a transition between authority states, not a shortcut to
free equipment.

### Pirate action structure

Pirates do not receive mandatory navy-style orders. They always retain free
predation against any real target they can locate and intercept, supplemented
by two optional sources of direction:

- **leads** are fallible, time-sensitive intelligence about an existing
  `TrafficCall`, ship, cargo lot, passenger, facility, or scheduled event; and
- **commissions** are unreliable or deniable patron bargains for a specified
  result, with explicit proof, deadline, compensation, and betrayal or
  nonpayment risk.

A lead or commission never spawns or duplicates its target. `SystemDay`
processing and ordinary simulation create the traffic and circumstances first;
underworld contacts, observation, corruption, moles, buyers, and patrons may
then expose them. Mail delay, competitors, changed schedules, traps, and
intervening events can make intelligence stale.

The pirate analogue to a naval deployment is a crew-defined **pirate cruise**.
Its articles record a hunting region or return condition, target and conduct
constraints, accepted commissions and important leads, prize shares, the ship
fund, and participation terms. Ignoring the articles, concealing prizes,
producing no prey, or refusing shares may affect crew loyalty and mutiny risk,
but the mechanic must not become repetitive role-playing busywork.

The operational loop is **obtain intelligence or a commission → set cruise
terms and choose a hunting region → locate and intercept real traffic →
intimidate, disable, board, capture, strip, or destroy → escape or manage the
response → reach a haven, fence, buyer, or patron → realize and divide proceeds
→ absorb heat, underworld-standing, crew, and political consequences**.
Pirates may ignore the structure and run amok; the structure makes targets
discoverable and gives them reasons to travel. Full semantics and open schemas
are in `docs/pirate-gameplay.md`.

### Pirate prize economics and heat

Piracy is not simply a high-payout combat mission. Its loop is **find a
target → assess value and response risk → intercept or board → capture, strip,
or destroy → realize the prize → evade and manage heat**. The relevant reward
is the net realized value of ship and cargo after combat damage, casualties,
crew replacement, towing or refit, registry and identity work, fence or haven
fees, bribes, and the probability of confiscation—not the target's listed hull
price.

The polity gradient should change both sides of that calculation. Rich or
powerful polities offer more valuable ships, cargo, insurance, and commercial
traffic, but have better patrols, faster mail-borne warrants, coordinated
navies and bounty hunters, stronger asset tracing, and lower fence percentages.
Frontier or corrupt regions offer poorer average prizes and less reliable
markets, but better black-market liquidity, sympathetic havens, slower or
fragmented enforcement, and a higher percentage of a stolen prize. Poor ports
also make repairs, crew, and resale less reliable.

Heat is a persistent, jurisdictional risk rather than a single wanted flag.
It should depend on evidence, publicity, target importance, prize value,
repetition, witnesses, identity/transponder exposure, and the issuing polity's
ability to propagate a warrant. Escalation can move from local alert to system
warrant, polity bounty, cross-polity notice, dedicated hunters, and a naval
task force. Mail latency creates an escape window but does not erase the crime.
Heat may decline through time, concealment, changed identity, restitution,
corruption, or political settlement, each with cost and uncertainty.

Captured ships remain liabilities: they need a crew, fuel, maintenance,
registry treatment, and a destination willing to accept them. Cargo and hulls
may carry liens, tracking evidence, or ownership claims. A lawful privateer
uses the same capture mechanics but receives a prize share or adjudicated
transfer and avoids criminal heat when the target, rules of engagement, and
court process are valid. Pirate balance must therefore use risk-adjusted net
prize value and survival rate, not gross capture payouts.

Complete-ship capture is an exceptional strategic event, not a routine cash
source. A captured hull should create separate choices: strip it for parts,
ransom or return it, sell a hot prize through a haven, retain it for personal
use, or pursue lawful adjudication. The fence percentage is a ceiling on a
gross offer, not cash in hand. Net realization must subtract condition and
combat damage, crew and fuel, towing or refit, transponder and registry work,
legal or syndicate fees, liens and ownership claims, delayed settlement, and
the chance of seizure or betrayal. A no-debt ship is still encumbered by title,
registration, insurance records, and identifying hardware.

Large or high-profile hulls should require a capable haven or syndicate and
may be paid in installments, credit, or a revenue share rather than an
instantaneous credit balance. A pirate who retains a prize takes on its full
operating and heat burden; a lawful privateer receives only the adjudicated
share or transfer authorized by the prize court. This keeps the economic value
of a captured ship from bypassing the merchant ladder while preserving the
possibility of a dramatic windfall.

Privateers are the deliberate exception to the pirate windfall model. A
letter-of-marque captain should encounter enemy shipping often enough for
prizes to be a normal part of expected income. The reward should come through
an adjudicated prize account rather than an informal fence: the polity or
prize court assesses the recoverable value of the captured cargo and hull,
deducts damage, salvage, evidence, court, and delivery costs, and awards the
licensed share or an authorized ship transfer. The assessment is based on
realizable value and condition, not the design's list price.

To make the prize worth pursuing without creating an instant fortune, a
privateer can receive a modest operating advance after securing the prize,
followed by a delayed adjudication payment or transfer after delivery and
review. Intact capture, low collateral damage, high-priority enemy logistics,
and good evidence can improve the award; disputed ownership, misconduct,
losses in transit, or a denied claim reduce it. The state can absorb a useful
captured hull into its navy or logistics service, while the captain receives
wealth, reputation, rank credit, or a controlled transfer rather than a free
unencumbered asset. Balance privateering against the merchant baseline using
expected net prize income, capture frequency, and mission risk.

Mail clippers are a deliberate low-upside role. They are fast, reliable
courier ships carrying secure or time-sensitive mail, but have limited cargo
and weak wealth/progression economics. Clipper work may provide dependable
contracts, travel access, and information timing, but should not offer a
substantial path to wealth, rank, ship-class advancement, or independent
power. Clippers are useful background traffic and a specialized option, not a
competitive replacement for merchant, privateer, or naval play.

## Using the Cepheus reference

Start with [`cepodnew-markdown/index.md`](cepodnew-markdown/index.md), then
read the relevant chapter before designing or changing a rule. In particular,
consult the travel/trade, trade-and-commerce, ship-design, common-vessel,
space-combat, world/port, and encounter chapters for their respective
features.

Distinguish three things in design notes and code comments:

1. **CE rule** — directly supported by the local reference.
2. **Game adaptation** — a deliberate simplification or balancing change.
3. **Open assumption** — not yet decided and subject to revision.

Link to the local Markdown source when citing a rule, table, or formula, for
example `[Trade Goods](cepodnew-markdown/07-trade-and-commerce.md#table-trade-goods)`.
If the source conversion changes, regenerate and review the affected chapter;
do not casually hand-edit generated reference content.

The local `/home/admin/RPG/2D6/Clement Sector/` library is a useful source of
Cepheus-compatible supplemental hulls and setting ideas, but it is not the
primary rules reference. In particular, its ship catalogue and many of its
designs assume a **Zimm drive**. We use the standard CE Jump drive, M-Drive,
and power-plant rules. For 100–2,000-ton source ships, the prescribed
same-letter Z-drive produces CE Jump-2 and uses the same interstellar fuel;
preserving the hull normally needs only the conversion cleanup documented in
`docs/zimm-to-jump-conversion.md`. Capital ships above 2,000 tons require
recalculation. Never import Zimm Points, in-system skip capability, recharge,
travel schedule, military-grade surcharge, emitter damage, or bubble-failure
rules into the game.

## Engineering guidance

- Keep simulation/domain rules separate from persistence, the game loop,
  terminal/UI protocol, and configuration/data.
- Keep one-time universe generation separate from mutable simulation state;
  persist generated system identifiers and source records before gameplay can
  mutate them.
- Represent system coordinates and inter-system travel in three dimensions;
  do not collapse distance or adjacency to a 2D map, sector, subsector, or
  hex grid for convenience. Keep astronomical/map data separate from mutable
  economic and player state.
- Use a single authoritative game server for all game state and rules. The
  user interface is a separate process that connects to the server over a
  TLS-PSK-protected connection; it must not reimplement or bypass game logic.
- Use a strict-schema binary application protocol over the TLS-PSK connection.
  The preferred design is modern ASN.1-like type expressiveness, with explicit
  enums, unions, constraints, optional fields, and compatibility rules. ASN.1
  encodings, protobuf, and Cap'n Proto remain serialization candidates; the
  RPC/data-flow model is a separate decision.
- Native Cap'n Proto RPC is not an acceptable protocol candidate for this
  architecture. Its connection/event-loop affinity conflates a logical
  connection with the executor that services it, conflicting with the required
  independent receive queue and transmit worker pool. Cap'n Proto's wire
  serialization may still be used with a small custom CT-RPC envelope if the
  preferred RPC stack cannot be made to run over TLS-PSK.
- The leading protocol candidate is Cap'n Proto serialization with a small
  project-specific CT-RPC envelope, not native Cap'n Proto RPC. Each framed
  message carries an explicit kind (handshake, request, response, event,
  cancellation, or close), a connection-scoped request ID where applicable,
  and any required ordering/epoch metadata. A single logical session may use
  one long-lived bidirectional stream; the wire protocol does not expose
  capabilities, promises, or object references.
- The receive half parses and validates owned Cap'n Proto messages, then puts
  commands into the single ordered simulator mailbox. The transmit half reads
  an independent per-connection queue and serializes responses/events. A
  connection is a lifetime/cancellation object, never a thread identity; no
  connection-affine event loop is required. Multiple workers may produce
  outbound messages, while the transport guarantees one ordered byte stream
  per connection.
- CT-RPC must define only the semantics we actually need: message framing and
  size limits, request/response correlation, event sequencing, backpressure,
  cancellation, malformed-message handling, graceful close, and behavior when
  the connection disappears before a response is sent. Keep these semantics
  independent of simulator persistence and game state.
- A player CT-RPC version advance must update the server declaration in
  `server/src/wire.rs`, the client declaration in `client/src/protocol.cpp`,
  both CT-RPC compatibility assertions in `tools/check_repository.py`, and the
  current-version statements in `docs/rpc-and-storage-schema.md` in the same
  change. Regenerate both language bindings through the normal builds and run
  `python3 tools/check_repository.py`. The repository check is an intentional
  manual compatibility tripwire; leaving its expected version behind will fail
  CI even when the client and server agree. Continue following the protocol
  numbering policy in `docs/rpc-and-storage-schema.md`: incompatible schema
  work shares one version during a release-development cycle and advances again
  only after that contract ships.
- Do not replace TLS-PSK with certificate TLS merely to use a larger RPC stack.
  The custom envelope runs over the already-authenticated TLS-PSK byte stream;
  Cap'n Proto remains responsible for schema evolution and binary encoding,
  while the CT-RPC adapter owns only transport and lifetime mechanics.
- Use GnuTLS 3.x for the Rust server's TLS-PSK transport and Botan 3 for the
  C++17 client's TLS-PSK transport. Negotiate their common TLS 1.3 external-PSK
  profile with ephemeral key exchange; disable zero-RTT application data so
  commands cannot be replayed during connection establishment. The PSK
  identity identifies the originating BBS, and the authenticated BBS then
  attests the player identifier in the CT-RPC hello. Maintain cross-library
  handshake, application-data, closure, malformed-input, and reconnect tests.
- After the handshake, a GnuTLS session may have exactly one receive worker
  and one transmit worker operating concurrently. No third operation may use
  that session concurrently. Reauthentication is disabled; traffic-key
  updates and orderly shutdown require an explicit coordination barrier so
  neither overlaps normal send/receive calls. Keep the per-connection outbound
  queue bounded and preserve its byte ordering.
- Treat serialization and RPC semantics as separate decisions. Define message
  framing, envelope/request IDs, operation namespaces, authentication/identity,
  command validation, errors, notifications, authentication timeouts,
  idempotency, and graceful shutdown before depending on UI features. The BBS
  authenticates with its TLS-PSK secret and attests the player identifier; the
  server does not implement reconnect policy. It enforces a configurable
  maximum number of connections per BBS and at most one logical connection per
  player. A newer authenticated connection invalidates the older one; its
  socket is removed from demultiplexing and handed to a separate cleanup
  worker, which cannot enqueue commands. Use TCP keep-alive for transport
  liveness; do not add application heartbeats or idle-timeout policy beyond a
  short authentication timeout.
- Feed all authenticated sessions and scheduled simulation events into one
  ordered simulator mailbox. The simulator is the sole state mutator. Each
  ingress item receives a monotonic sequence number. A player's session epoch
  is assigned by the server when the connection is authenticated; when the
  first command with a newer epoch is processed, the player's stored epoch is
  advanced. Commands with older epochs are then rejected, while equal-epoch
  commands remain valid. Advance the epoch before validating the first newer
  command so a bad first command still takes ownership. Delayed bytes from an
  old socket are harmless because their queue entries carry the old epoch.
- Make the engine transactional from the beginning. Player commands, due
  calendar events, mail deliveries, combat results, economy transitions, and
  simulator results execute as transactions with an explicit snapshot/revision
  and private changes, then commit atomically at one simulation-time point.
  Commits remain serialized through the authoritative state owner, but the
  transaction interface must not depend on locks or thread affinity.
- A transaction may be represented by a coroutine and may yield only with an
  owned continuation and owned snapshot-derived data. It must never retain a
  mutable world reference, database transaction, Cap'n Proto reader, or other
  borrowed state across an await. Long activities such as jumps, encounters,
  and mail transit are staged as multiple transactions connected by scheduled
  continuation events rather than one transaction held open for game time. A
  suspended continuation must have a serializable state-machine form in the
  journal; an in-memory Rust future or native coroutine stack is never the sole
  recovery record.
- After a transaction commits, publish its durable outbox (responses, events,
  mail, and worker jobs) to independent fan-out queues. No external observer
  may see an outcome before its state commit, and an abandoned connection must
  not roll back an already committed game transaction.
- Treat rollback as an authoritative recovery signal, not a client-visible
  command error. A speculative worker calculation that has not entered the
  transaction engine may be discarded or recomputed. Once an authoritative
  transaction, queue position, session epoch, or client-visible outcome must be
  undone, stop accepting new work and fail-stop the server; recover from the
  last durable boundary and replay rather than notifying failure and continuing
  with a potentially divergent queue. A rule rejection is instead a normally
  completed no-op (or bookkeeping-only) transaction: record its result/audit
  data and emit the rejection response, but do not call it a rollback.
- The fairness invariant is that the game never advances past the last durable
  committed transaction. Recovery reports or records that commit sequence as
  the authoritative boundary; all later in-flight work is replayed in order or
  treated as never having happened. This may be disruptive, but it prevents a
  player from receiving a partial outcome or being charged for state that was
  subsequently lost.
- Assign each dequeued command an immutable ingress sequence and transaction
  ID. Never reinsert a rolled-back command as a new queue item. Internal
  speculative retries retain the original sequence/ID; if an authoritative
  rollback is required, the server stops and restores the queue/session state
  to the durable boundary before replaying it. Any immediate protocol
  acknowledgement means only receipt/processing-started, not an authoritative
  game result.
- Never silently roll back an already committed journal sequence. If recovery
  finds a corrupt or ambiguous committed history, fail closed into maintenance
  or recovery mode rather than continuing from a potentially divergent state.
  A historical restore is an explicit administrative operation that invalidates
  active sessions; corrections after restore are new compensating transactions.
- Request/transaction IDs must resolve crash ambiguity: replaying a committed
  ID returns its recorded result, while an ID with no committed record may be
  retried. No irreversible outbox effect may be emitted before the associated
  commit is durable.
- Treat the transaction journal as the recovery authority. A durable commit
  record must identify the transaction/cause sequence, base revision,
  simulation time, deterministic inputs or decisions (including random-stream
  draws), state changes, and outbox entries. Write and make the commit durable
  before acknowledging the command. An incomplete or uncommitted record is
  ignored during recovery.
- Take periodic authoritative checkpoints containing the last committed
  sequence and simulation-control state, then replay committed journal records
  after that sequence on startup. Rebuild feeds, indexes, worker queues, and
  other derived state from authoritative records. Every external effect uses
  an idempotency key derived from its commit/transaction ID so replay cannot
  duplicate mail, payments, or notifications.
- Treat durability as a deployment contract, not a property software can
  guarantee by itself. An acknowledged transaction requires the configured
  storage stack to honor flush/barrier semantics and to survive the stated
  failure model. Production guidance should include ECC memory, a UPS or
  equivalent power protection, power-loss-protected storage, and mirrored or
  replicated journal/checkpoint storage; backups must cover site loss and
  operator error. A protected write cache is acceptable only when its battery
  or capacitor-backed persistence is part of the failure model.
- Document durability tiers and recovery objectives (local crash recovery,
  device/host failure, and site loss). Exercise them with crash/power-loss
  tests, checksum validation, restore drills, and journal-replay tests. Never
  promise that software journaling alone provides an absolute guarantee.
- The baseline useful-server deployment assumes ECC memory, UPS protection,
  and mirrored storage/RAID, but those cover only some failure modes. Durable
  acknowledged commits also depend on power-loss-protected drive/controller
  caches, honest flush/barrier behavior, filesystem or journal checksums,
  redundant power and cooling where practical, and backups outside the failure
  domain. Provide a portable journal/checkpoint implementation with simple
  backup/restore hooks; higher redundancy and replication remain optional
  deployment tiers. After catastrophic host loss, restoring the latest
  available backup (and accepting its rollback window) is an explicit, honest
  outcome.
- Parallel workers are allowed for read-only or speculative work: procedural
  generation, route/path calculations, market projections, news ranking, AI
  planning, serialization, and persistence I/O. Such work runs against an
  immutable snapshot and returns a result tagged with the snapshot revision,
  simulation time, cause sequence, and dependencies. The simulator validates
  those tags before committing the result; stale or conflicting results are
  discarded or recomputed. Assign deterministic tie-break keys when multiple
  workers produce events for the same time.
- A bounded MPSC queue or equivalent well-tested channel is sufficient; the
  simulation design does not require every queue to be lock-free. Optimize
  queue contention only after profiling demonstrates that the authoritative
  commit loop is a bottleneck. This preserves a future path to sharding or
  optimistic transactions without making the first implementation depend on
  them.
- Persist future-due mail deliveries and structural world events in a
  time-indexed event calendar. Advancing simulation time drains that calendar
  in order before processing the newly visible command; do not implement mail
  propagation as an hourly/daily scan of all in-flight messages. Delivery
  handlers materialize indexed system-feed entries and schedule only their next
  hops, with aggregation available for large news fan-out. The per-system
  daily job may assemble mailbags, adjust stipends, and schedule carrier
  pickups, but it never scans the in-flight archive.
- Filter the effective command set by ship phase, player permissions, entity,
  and local authority. The UI may hide unavailable commands, but the server
  always enforces them. A standard `InvalidCommand` response covers unknown,
  currently-invalid, unauthorized, malformed, and stale commands; every error
  includes the authoritative current phase (and should include request/state
  correlation data). The current phase is determined when the simulator
  processes the command, not when the network layer receives it. Encounter
  disconnect behavior remains open until the live-versus-turn-based model is
  chosen.
- Prefer data-driven CE tables and formulas over duplicated magic numbers.
- Seed each independent production materialization operation with 256 bits
  from the operating-system CSPRNG. Ordinary single systems may use that value
  directly; conditioned batches use a cryptographic counter stream to draw
  prospective system seeds and persist only accepted results. Derive named
  feature/event streams cryptographically and persist generation versions and
  resolved outcomes. Test and development servers may use explicit
  deterministic seeds; never use a predictable gameplay PRNG for hidden-world
  or security-sensitive values.
- Treat server state as authoritative; validate every client command.
- Design for ANSI/ASCII terminals, latency, reconnects, concurrent players,
  persistence, and safe recovery from interrupted turns.
- Keep economy and combat state transitions explicit, bounded, and testable.
- Use CE terminology consistently, while explaining any game-specific meaning.
- Keep sysop administration separate from player simulation state and expose
  no privileged path that can mint player resources or alter fair outcomes.

## Development workflow

Every product release preparation must add a tracked
`docs/releases/v<version>.md` body for the exact product version. It must
contain an evidence-based Compatibility notice, curated Highlights, and the
exact previous-to-current Full changelog link. State unchanged compatibility
counters explicitly, identify supported mixed-version client/server pairings
and upgrade order, and describe any store migration or reinitialization. The
tag workflow publishes this file verbatim and must fail rather than fall back
to generic generated notes. Follow `docs/release-process.md` for the complete
gate.

For a non-trivial feature:

If a required or desired compiler, build tool, system library, package, or
other external dependency is actually not installed, tell the user exactly
what is missing and ask them to install it, then wait. An item is not
"missing" merely because its source is absent from a language package
manager's local cache: declare normal project dependencies and let the
project's package manager resolve them. If that resolution fails because an
external tool or system package is absent, do not install, vendor, replace, or
work around it without explicit user direction.

1. Inspect the relevant chapter and table in `cepodnew-markdown/`.
2. Write the intended behavior, CE source, adaptations, and acceptance
   criteria in a design note or issue.
3. Implement the smallest useful vertical slice.
4. Add tests for formulas, state transitions, boundaries, and failure cases.
5. Run the available formatter, linter, tests, and build checks.
6. Update this document or a decision record when the settled behavior
   changes the project conventions.

C++ source must remain readable without editor-assisted reformatting. Do not
compress structs, enums, function bodies, loops, or aggregate returns onto a
single line. Put struct members and non-trivial enum values on separate lines,
use braces for loop and conditional bodies, and format designated aggregate
initializers with one field per line when they do not fit comfortably on one
short line. Generated bindings are exempt; hand-written protocol adapters and
door code are not.

Tests should cover CE examples, minimum and maximum values, conservation of
credits/cargo/hull state, seeded randomness, invalid commands, reconnects,
duplicate commands, queue ordering, stale session epochs, phase-gated
commands, frontier materialization races, cryptographic per-system seed
stability, orbital positions at different simulation times, and multi-turn
economy behavior.

## Living decisions

| Date | Decision | Status |
| --- | --- | --- |
| 2026-08-24 | League Coordinator authority uses its own CT-League version-1 TLS-PSK endpoint on default port 7326. A global administrator provisions each numeric League ID and 32-byte PSK after universe initialization. The LC may rename only that League, create permanent member BBSs, and revision-check enable/disable state; it cannot attach, remove, transfer, configure, moderate, or act as another authority. Disabled members retain all state and remain placement anchors but lose game/sysop authentication immediately. BBS-polity generator version 4 ranks later member capitals nearest an existing materialized League capital within the first viable geometry variant; independent and first-member placement retain ordinary closest fit. Current player/dossier affiliation is typed and mail-gated, while old news remains immutable. Player CT-RPC remains 8, admin/sysop remain 2, and storage format is 2. | Current |
| 2026-08-15 | Refresh the vendored OpenDoors production surface from `RealDeuce/OpenDoors` commit `3edf9008a6df2a7d71674f8b43e307d1fc2f721d`, retaining the documented `user_8bit` extension. The shared client library exports only a C transport ABI so independently static-linked Windows GNU runtimes never exchange C++ exceptions, STL objects, or allocations. Errors cross as category, optional native code, and an exact-length caller-copied UTF-8 message. | Current |
| 2026-08-06 | The common initial product version is `0.7.0`, matching the current Milestone 7 development stage. Product SemVer is separate from protocol, storage, record-codec, and generator compatibility counters. | Current |
| 2026-08-06 | The standalone repository uses GitHub and GitHub Actions. It vendors the audited production OpenDoors source surface at Synchronet commit `47feab1e8bf776175b44f40dffebbc9560322e20`, statically links it, and includes neither upstream `ex_*` examples nor `xpdev`, which those examples alone use. Portable client packages build pinned static Botan 3.12.0 and Cap'n Proto 1.5.0, contain the door, sysop, and League Coordinator programs, and publish checksums, dependency reports, debug-symbol archives, notices, and exact tagged-source correspondence. | Current |
| 2026-08-06 | With no deployed compatibility contract, player, administrator, and sysop protocols; LMDB storage; every internal record codec; the clock; universe/celestial/BBS-polity generators; coverage, settlement, CNS5, and frontier samplers; and player-setup revision all begin at compatibility version 1. There are no legacy readers or migration paths. | Current |
| 2026-08-09 | Crew Management reports effective headcount against the established ship complement and shows current/established strength for aggregate appointments. Combat crew hits select by represented headcount: supporting casualties reduce the aggregate team one position at a time, while selection of the named leader uses individual injury state. A dead leader does not erase surviving supporting personnel from provisions or complement accounting. | Current |
| 2026-07-25 | Tentative project name is **Cepheus Trader**. | Current |
| 2026-07-25 | The game targets a BBS door/terminal experience. | Current |
| 2026-07-29 | The door supports exactly three page-oriented profiles: ISO 646 plain text, ISO 646 with ECMA-48 SGR colour, and CP437 with ECMA-48 SGR colour. Plain page transitions use form feed; enhanced transitions use reset, clear-screen, and cursor-home before emitting wrapped lines. A coordinate-rendered TUI and general Unicode profile are out of scope. Every action is usable at 40×24, with 80×24 as the normal target, responsive wrapping owned by the door, and printable-key alternatives to extended keys. OpenDoors supplies connection, input, colour, and optional CP437-to-UTF-8 conversion, but its unreliable legacy width field and fixed 80×25 screen helpers are not the layout authority. | Current |
| 2026-07-25 | CE space-trading and combat rules are the primary rules reference. | Current |
| 2026-07-25 | The product is a simulator, not an RPG. | Current |
| 2026-07-25 | Relevant character and crew skills remain as operational modifiers for combat, trade, and ship operations; the full career/adventure RPG framework does not. | Current |
| 2026-07-25 | `cepodnew-markdown/` is the local, reviewable CE rules reference. | Current |
| 2026-07-25 | A single authoritative game server owns all game logic; a separate UI process connects over TLS-PSK. | Current |
| 2026-07-25 | The BBS authenticates with its TLS-PSK secret and attests the player identifier; the server enforces a BBS connection cap and one logical connection per player, while the door owns reconnect/exit behavior. TCP keep-alive is sufficient for liveness; only authentication has an application timeout. | Current |
| 2026-07-25 | All sessions and scheduled simulation events feed one ordered simulator mailbox; the simulator is the sole state mutator. Per-player epochs are advanced when the first command from a newer authenticated connection is processed, and older epochs are rejected thereafter. | Current |
| 2026-07-26 | The repository root organizes separate projects. The authoritative Rust crate and build live under `server/`; the C++17-or-newer/OpenDoors project lives under `client/`; both generate bindings from the shared Cap'n Proto schema under `protocol/`; cross-project notes and the CE Markdown reference remain at the root. | Current |
| 2026-07-26 | The first server vertical slice implements CT-RPC framing, independent RX/TX queues, the ordered engine, LMDB persistence, session epochs, deduplication, journaling, and recovery. Its listener now uses the system GnuTLS 3.x TLS 1.3 external-PSK adapter; there is no cleartext listener mode. | Current |
| 2026-07-26 | GnuTLS 3.x is the selected TLS implementation for the Rust server; Botan 3 is selected for the C++17/OpenDoors client. Their common profile is TLS 1.3 external PSK with X25519 ephemeral key exchange and no zero-RTT, verified by cross-library interoperability tests. On the server, one RX worker and one TX worker may share a GnuTLS session under its documented restrictions; key updates, reauthentication, and shutdown must not race normal I/O. | Current |
| 2026-07-26 | A full player identity is the tuple `(BBS ID: UInt32, local player ID: UInt32)`. The server assigns the BBS ID; its canonical unsigned decimal representation is the TLS external-PSK identity and must match the BBS ID in `ClientHello`. Sessions, persistent player state, epochs, and deduplication keys use the complete eight-byte tuple. Names and aliases are separate display data. The C++ client and Rust server both generate bindings from `protocol/ct_rpc.capnp` and the implemented first client slice completes `ClientHello`/`ServerHello`. | Current |
| 2026-07-26 | Global operator management uses a separate TLS 1.3 external-PSK listener bound only to an IP loopback address. Its PSK is a raw 32-byte `admin.psk` beside the database, generated from the OS cryptographic random source if absent. The data directory/key use `0700`/`0600` on Unix and a protected owner/System/Administrators DACL on Windows. Management commands use `protocol/ct_admin.capnp`, carry idempotent command IDs and authenticated operator authority, and enter the same authoritative engine mailbox. The first implemented command atomically adds a BBS, assigns its UInt32 ID, stores its generated PSK, journals the non-secret metadata, and activates the credential without restart. | Current |
| 2026-07-26 | A newly issued BBS ID and PSK are transferred from the central operator to the BBS sysop out of band. `cepheus-trader-sysop [--config FILE] [--server HOST] [--game-port PORT] [--sysop-port PORT] init-credential [CREDENTIAL_FILE]` reads both from stdin, suppresses PSK echo on a terminal, creates missing parent directories, and bootstraps the shared non-secret installation configuration plus a versioned secure credential file (`0600` with strict owner/type/link checks on Unix; protected owner/System/Administrators DACL and no reparse points on Windows). The default credential is `cepheus-trader.credential` beside the selected configuration; an existing configuration's values are authoritative. The player client reads that file. BBS PSKs are absolutely prohibited in command-line arguments and environment variables. | Current |
| 2026-07-31 | A fresh game must run the explicitly confirmed `InitializeUniverse` transaction before `AddBbs`. Premature enrollment is a normal rejected administrator request: it allocates no ID or credential, does not terminate the engine, and tells the operator to initialize first. The supported bootstrap order is initialize Federation, enroll BBS, transfer/bootstrap credential, configure BBS, then admit players. Initial BBS configuration atomically adds its materialized stellar systems to the live traffic/mail simulator and recomputes J-2 neighbors. Destructive reinitialization preserves and rematerializes existing BBS control state. Missing simulation counterparts are corruption; startup does not rebuild them. | Current |
| 2026-07-26 | Starting-player design uses independent trade-to-combat and chaos-to-order polity orientations. Institutional order is not CE Law Level; CE Law Level remains a separate world restriction/enforcement characteristic. | Provisional |
| 2026-07-26 | Startup uses a 3 × 3 catalog: trade-focused/mixed/combat-focused crossed with orderly/contested/chaotic. Every cell contains exactly three predesigned offers—a trader, a privateer, and a navy/public-service command—so there are 27 starter designs or variants. The player sees only the home BBS cell. | Provisional |
| 2026-07-26 | The locally aligned career has the best expected starting prospects: trader in trade-focused polities, privateer in mixed polities, and navy/public service in combat-focused polities. Other careers remain viable but pay an environmental mismatch cost. Balance includes local work, support, authority, financing, and obligations rather than hull price alone. | Provisional |
| 2026-07-26 | Starter balance separates ship-use authority, player equity, working capital, institutional support, legal authority, and liabilities; nominal ship price is not treated as player wealth. | Provisional |
| 2026-07-26 | Clement non-capital ships from 100–2,000 tons can treat a source Z-drive letter that yields CE J-2 as the corresponding Jump drive and retain the existing two-parsec fuel allocation. Starter variants install the minimum CE J-2 drive and reserve any actual recovered space for customization; current Anderson & Felix designs commonly recover 10 tons/MCr20 above 400 tons, but older source blocks vary and must be checked individually. Savings are package value, not cash. Zimm-only software names, points, skip transits, recharge, and failure rules are removed. Ships above 2,000 tons require CE recalculation. | Current |
| 2026-07-27 | The Clement/Earth ship conversion is complete: all reserved identities `ship-1` through `ship-191` are active rule-derived, PI-free catalog entries, with 24 supplemental/core and original designs at `ship-192` through `ship-215`. The closed-content inventory row is an independent generic-rules replacement. Exact catalog status and recurring corrections are recorded in `docs/ship-catalog-conversion-status.md`. | Current |
| 2026-07-27 | Every converted ship uses the standard CE drive-performance table at its actual loaded displacement. Published high-performance custom drives do not survive merely by relabeling; in particular, the 5,000-ton dreadnought conversion is J-2/M-2 with standard Z drives. Excess armor and hardpoints, one-year cells, TL12-only weapons on TL11 hulls, and the deleted heavy-railgun family are likewise corrected in the actual bill of materials. | Current |
| 2026-07-27 | Catalog naming will model design families and upgrade paths separately. A family groups a hull lineage and may contain variants native to several paths. Each of nine upgrade paths crosses families and hull sizes as the product ladder of one manufacturer/shipyard specializing in a trade/mixed/combat × orderly/contested/chaotic matrix cell. A path may have gaps and explicitly backfill them with adjacent-path designs rather than duplicate designs merely to span starter through 5,000 tons. Stable mechanics and relationships are global, while sysops may author non-mechanical local setting prose and aliases. | Current |
| 2026-07-27 | The family pass groups all 215 active designs into 114 stable-numbered families: 39 shared lineages containing 140 designs and 75 singleton families. `catalog/ships/families.toml` and each ship's `family_id` are cross-validated; `docs/ship-family-grouping.md` records the PI-free rationale. Similar size, role, or normalized statistics alone do not establish a family. | Current |
| 2026-07-27 | All 215 designs now have one native assignment among the nine specialist manufacturer/shipyard paths and a validated auxiliary/starter/light/medium/heavy/capital size stage. The paths are intentionally sparse; only orderly combat currently has native coverage at every stage. Native path is design origin/doctrine, not an ownership or availability restriction. Exact neighboring-path backfill edges remain to be selected explicitly after economic and starter-balance review. | Current |
| 2026-07-27 | The catalog naming pass assigns canonical Open Game Content names to all nine paths/manufacturers, 114 families, and 215 fitted designs in `catalog/ships/names.toml`. Names use historical, mythic, geographic, scientific, and public-domain literary references; source-publication class and variant names remain excluded. Each path has a six-stage semantic naming sequence, while related family vocabulary takes precedence for a lineage spanning several paths. Ship records repeat their canonical name and are cross-validated. Sysops may add local aliases and setting prose without replacing the canonical identity or changing mechanics. | Current |
| 2026-07-27 | `catalog/starting-offers.toml` is the canonical 3 × 3 × 3 design mapping: every polity cell has trader, privateer, and navy/public-service packages, for 27 offers backed by 19 active Jump-capable catalog designs. Repeated designs represent different title/support/obligation packages. Six hypothetical refits from the old PI-labelled worksheet were not invented: admitted Sinbad, Challenger, Marco Polo, Argosy, Robur, and Silver designs fill those roles. Offer economics, refit choices, staffing, and legal terms remain separate versioned package data. | Current |
| 2026-07-26 | Individually modeled characters use STR, DEX, END, INT, EDU, and CHA. Core CE Social Standing is not stored as a characteristic; rank, title, citizenship, reputation, legal status, institutional authority, and relationships are separately scoped persistent state. Old CE checks using SOC translate to CHA when they measure personal influence. | Current |
| 2026-07-26 | Authoritative game-state mutations use a single ordered commit loop; parallel workers may calculate against immutable snapshots and submit revision-tagged results for validation, but no general cross-shard transaction model is required for the initial engine. | Current |
| 2026-07-26 | Engine tasks use owned, yieldable transaction continuations with private changes and serialized authoritative commits; long activities are staged across transactions, and post-commit outboxes feed independent fan-out workers. | Current |
| 2026-07-26 | Crash recovery uses a durable transaction journal plus periodic authoritative checkpoints; committed outboxes are replayable and external effects are idempotent by transaction ID. | Current |
| 2026-07-26 | Durability is an explicit deployment contract: software journaling cannot guarantee physical persistence without suitable power protection, ECC, power-loss-protected storage, redundancy, backups, and tested recovery procedures. | Current |
| 2026-07-26 | The baseline useful-server deployment assumes ECC, UPS, and RAID, while recognizing that PLP caches, honest flushes, checksums, redundancy, and off-host backups are still required for stronger durability; catastrophic host loss may require restoring a backup with an accepted rollback window. | Current |
| 2026-07-26 | Rule rejection is a normally completed no-op/bookkeeping transaction; an authoritative rollback is a fail-stop recovery signal, not a client-visible error. Never silently continue after undoing queue/session state or committed history. | Current |
| 2026-07-26 | Dequeued commands retain immutable ingress sequence/transaction IDs; speculative work may retry before commit, while an authoritative rollback restores queue/session state to the durable boundary and replays rather than reordering the queue. | Current |
| 2026-07-26 | Fairness invariant: the game never advances beyond the last durable committed transaction; recovery replays later ingress in order or treats it as uncommitted work, even at the cost of disruptive session loss. | Current |
| 2026-07-25 | The universe is a true 3D system map anchored to Earth's Milky Way position, using coreward/rimward, spinward/trailing, and galactic north/south axes. Each system has a CE-style planetary system. | Current |
| 2026-07-28 | The implemented, explicitly confirmed `InitializeUniverse` administrator transaction deletes gameplay/universe state, disconnects player sessions, and preserves BBS enrollment, credentials, and sysop-selected BBS settings. It creates one Federation polity, Earth at TL13, and 43 separately seeded stellar-component systems: the CNS5 prefix from Sol through Tau Ceti plus every known component inside the convex hull extended through five real J-2 bridge groupings. Each stellar component is a game system, so Alpha Centauri contributes Alpha Centauri A, Alpha Centauri B, and Proxima Centauri. Brown- and sub-brown-dwarf components are retained because their generated local bodies may support skimming and other operations. The resulting map has a 36-system Sol J-2 component and a fully catalogued seven-system Ross 248/61 Cygni island reachable by J-3; unattended BBS growth attaches only to Sol's J-2 component. CNS5 astrometry and the complete range audit are documented in `docs/initial-federation.md`. | Current |
| 2026-07-28 | Stellar distribution version 1 is a continuous locally normalized component density: 0.0906 stars-plus-brown-dwarfs per cubic parsec at Earth, Solar Galactocentric radius 8,178 pc, Solar height +20.8 pc, 2,200 pc radial scale, 300/900 pc vertical scales with a 6% thick midplane fraction, and four identical trailing logarithmic arms. Arms are quarter-turn copies with ten-degree pitch, 350 pc Gaussian width, 0.35 peak overdensity, curved centerlines, and local coreward/spinward tangents. The formula is implemented in `server/src/universe.rs` and specified in `docs/stellar-distribution.md`; frontier Poisson sampling remains to be implemented. | Current |
| 2026-07-28 | Resolved stellar volume is stored in versioned bitmaps over quarter-parsec cells grouped into 8-pc, 32×32×32-cell chunks. Given Jump target coordinates, the implemented oracle tests every cell with positive-volume intersection with the canonical six-parsec arrival sphere and returns either fully mapped or revision-tagged missing chunk masks. Materialization bits use a private compare-and-set transaction primitive; stellar sampling and atomic integration with Jump completion remain pending. Server resolution is distinct from player knowledge. Full semantics are in `docs/observed-volume.md`. | Current |
| 2026-07-28 | A BBS polity is a locally conditioned, roughly ten-system frontier cluster rather than the first rare cluster found by expanding a search sphere around Earth. It has one to three boundary crossings at J-2 or J-1; all other cluster-to-outside system pairs exceed three parsecs and require J-4+ for direct entry. Its TL12 capital has two complementary inhabited J-1 neighbors supporting subsistence trade. The founding transaction resolves a three-parsec guard volume around every member so later materialization cannot create another J-1 through J-3 entrance. Empty-space staging remains possible at its normal operational cost. | Current |
| 2026-07-29 | Celestial generation version 1 rejects inhabited primary worlds outside the companion-truncated habitable region and gives every companion star complete Keplerian orbital elements. The authoritative navigation geometry evaluates the current three-dimensional union of every stellar and body 100-diameter exclusion. A thrust-scaled operational clearance guarantees at least one-half game day of constant-thrust, midpoint-turnover travel between a primary-world port and a safe Jump locus. | Current |
| 2026-07-31 | Place-naming catalogue schema 1 and naming algorithm 1 define six coherent system/world profiles. The Federation uses its fixed profile; BBS polities persist a profile selected from domain-separated placement entropy; unaligned space uses stable 25-parsec regional profiles. System names are allocated case-insensitively against the complete stored universe and collision exhaustion aborts materialization. BBS-polity generator versions 1 and 2 and settlement-capacity sampler version 1 separate naming draws from physical-generation draws. | Current |
| 2026-08-02 | Materialized non-player personnel use `catalog/person-names.toml` schema 1. Its 95 given and 95 family names provide 9,025 deterministic full-name combinations through independent domain draws. A six-person port hiring slate may not contain duplicate full names. Starter-crew role callsigns remain an intentional editable creation convenience and are not the generated legal names of unrelated personnel. | Current |
| 2026-07-28 | Immutable celestial baselines are not persisted. A system stores its seed, generation version, coordinates, polity, and mutable names; stars, primary and secondary worlds, bodies, moons, and orbital elements are reproducible derived views. Only mutable overlays such as population changes, facilities, ownership, construction, damage, depletion, and survey/player knowledge become database state. | Current |
| 2026-07-28 | The special BBS-polity topology is eligible only when its complete cluster and three-parsec guard volume lie within the Galactocentric annulus `6,000 pc <= Rgc <= 11,000 pc`; on the direct Earth radial axis this is about 2,178 pc coreward and 2,822 pc rimward. The annulus excludes the bar/bulge regime and the first major outer-disk break, but does not limit travel or other civilization types. Candidate sites inside it must also pass a versioned local-density and conditioning-likelihood budget accounting for arms and Galactic height. | Current |
| 2026-07-29 | Unknown systems require no precursor observer. The setting has no universal von Neumann survey swarm; ordinary locally built beacons may improve later navigation but are neither self-replicating nor required for discovery. A first-arrival ship uses remote stellar data plus local survey: minutes reveal major mass concentrations, hours or days refine the architecture, and longer work establishes detailed planetary, biological, and small-body data. Densitometers are interpreted as gravitic tomography with mass-presence and density-gradient scaling; their exact game sensitivity remains to be calibrated. | Current |
| 2026-07-29 | Normal CE population generation remains unchanged through twice the active settlement extent `E`, where `E` is the greater of the initial Federation extent and the furthest BBS prime-system distance from Earth. Across `2E < r < 3E`, linearly mix normal CE seeds with CE seeds conditioned to Population 0 using conditioned fraction `(r - 2E) / E`; at `3E`, ordinary population is zero. This keeps every result derived solely from its stored system seed. BBS-conditioned systems may extend the envelope, but already-materialized populations are never rewritten. The initial Federation extent `E₀` remains open. | Current |
| 2026-07-29 | Uninhabited systems have no routine market, passengers, contracts, repairs, news, or local enforcement; their primary uses are concealment, unsupported refuelling, staging, caches, and transient activity. Player bases and colonies are deferred late-game credit sinks and initially import-dependent liabilities. Existing ships can establish supplied outposts, while a purpose-built near-5,000-ton colony hull can found only a small settlement whose diversified economy develops over generations. Exact colonization rules remain open. | Current |
| 2026-07-29 | Unaligned-system traffic character is a regional field rather than an independently random polity profile. It generally drifts toward combat and chaos beyond polity influence, while mutually supporting neighboring systems can form natural islands of trade and order. Traffic amount is separate from those axes and is based primarily on population, Tech Level, and realized trade. CE Law Level remains distinct. | Current |
| 2026-07-29 | In-system traffic is tracked among five interrelated but unequal location classes: Jump arrival loci, individual gas giants, the inhabited world, Jump departure loci, and everywhere else. Activity at each depends on routed flow, exposure or dwell time, contact conditions, and deliberate loitering. Orbital geometry and operational choices divide traffic among actual location instances. | Current |
| 2026-07-29 | Everywhere else is not a generic random-encounter pool. A sensor contact becomes an encounter only after a feasible intercept produces converging trajectories and manageable relative velocity. Deep-space encounters require a cause such as rendezvous, convoy, pursuit, a known flight plan, distress, shared destination, or an exceptionally dense corridor. Background traffic remains aggregate until observation or a consequential event requires a persistent vessel. | Current |
| 2026-07-29 | Initial steady-state cargo flow uses actual population and `E = Σ(Nworld × 2^(TLworld - 8))`, then `Q = 300 tons/week × sqrt(E / 1,000,000) × R`, where `R` is realized trade participation. Trade codes and route conditions allocate `Q` into directed flows; explicit facilities add non-duplicated deltas. The coefficient and `R` derivation are prototype calibration values. Route flow is converted to an initial fleet mix by `C = clamp(100 × (F/100)^(1/3), 50, 1,500)` nominal cargo tons, 75% mean loading, and `F/(0.75C)` departures per week. Full source basis and caveats are in `docs/system-traffic-and-encounters.md`. | Current |
| 2026-07-29 | Directed routes below 0.5 calls/week persist every planned arrival and departure; routes from 0.5 through 5 persist the near-term schedule and aggregate their distant flow; busier routine traffic may remain aggregate until consequential. Representation changes require hysteresis. Persist lightweight `TrafficCall` and `CargoLot` records, materializing a complete NPC ship only when observation or consequences require it. Player and background carriers consume the same accumulated inventory, so a player taking sparse cargo changes what the scheduled background carrier can do. | Current |
| 2026-07-29 | Every materialized system has one durable `SystemDay` job for every game day. It advances aggregate production, consumption, inventory, offers, traffic, mail preparation and incentives, piracy, enforcement, and mutable facilities, while materializing only consequential outcomes. The scheduler stores only the next job per system; completion advances `last_processed_day` and schedules the following day. Empty systems take a cheap logical no-activity path, and derived orbital positions do not tick. Lazy or bulk execution is only a recovery/catch-up optimization and must preserve daily RNG advancement, durable consequences, and intermediate scheduled events. | Current |
| 2026-07-31 | The initial authoritative simulation/mail substrate is implemented. `SystemDay` deterministically generates game-visible message classes and ordinary departures from the persisted system seed and logical day. Immutable messages fan out into route-specific envelopes; beacon queues load up to 512 eligible envelopes into a sealed mailbag; a named simulated traffic ship and custody leg carry it over a separately scheduled one-week Jump; arrival delivers, forwards, or expires each envelope. Every event currently remains an individual ordered LMDB transaction. The tour audits custody and reopen recovery and reports unique/copy message volume, thread CPU per system, and observed whole-universe progression rates. Common Goods cargo, player itineraries, player mail custody, and live clock advancement now exist; passengers and presentation of the player circuit in the non-interactive tour remain pending. Full boundary and calibration are in `docs/non-interactive-universe-tour.md`. | Current |
| 2026-08-01 | Roadmap Milestone 2 is complete. At the outbound Jump locus, a player ship atomically takes up to one nonempty exact-next-hop beacon bag; the sealed bag and carrier-leg identity persist in the ship record across restart. Jump arrival atomically delivers, forwards, or expires its envelopes, clears ship custody, records an arrival receipt, credits the advertised stipend once, and exposes the physically available destination feed without rewriting message creation time. Replayed handoff or packet commands cannot duplicate delivery, payment, or visibility. Per-captain first-seen/classification state persists independently of institutional delivery; received public-origin records update Known Universe provenance. The door implements arrival triage and Message Management through the shared ISO 646, ISO 646/ECMA-48, and CP437/ECMA-48 output paths. Public mapping notices and sealed direct Earth filings use ordinary physical envelopes; withholding and secret choices dispatch nothing, and committed disclosure is irreversible. The initial carrier tariff is provisional but route-invariant: Cr100 per nonempty exact-hop bag plus Cr1 per envelope. | Current |
| 2026-07-31 | Capacity baselines may deterministically provision multiple configured BBS polities and report LMDB used bytes in addition to event and CPU rates. The ten-BBS settlement-edge fixture fixes `E = 28.993534 pc`, resolves the full `3E = 86.980601 pc` sphere with the Galactic inhomogeneous Poisson sampler, and contains 238,812 systems in 77,991,936 initial LMDB bytes. A 10,000-event day-zero prefix retained 5,914,624 bytes and projected at least 87.4 million system-day transactions plus enough departures/arrivals for roughly 752 million total events and 34.5 wall-days for one year under the current individual-commit model. These are measured-prefix projections, not authorization to change simulation semantics. Because messages are retained, storage must be reported at an age and as growth per game day/year rather than as a timeless maximum. Full results are in `docs/non-interactive-universe-tour.md`. | Current |
| 2026-07-31 | Internal persistent encodings may remove facts already determined by ordered LMDB keys or immutable parent records, provided logical records, event IDs, due-time eligibility, public reports, and transaction boundaries are unchanged. During undeployed development only the current encoding is readable; incompatible edits advance the storage format rather than adding dead migration paths. The implemented pass uses hybrid coverage encodings, key-derived record/event fields, compact deterministic frontier names, typed journal records, inferred origin delivery, and packed future-departure plans. A plan reserves every logical departure ID up front and yields one ordinary input at each original due time; it is storage packing, not event batching. | Current |
| 2026-07-31 | The deployment target is approximately USD 50 per month on a mainstream cloud provider for a ten-BBS universe with fifty active players. The 238,812-system fully materialized settlement-edge fixture is a stress ceiling, not an expected provisioned footprint; production remains generate-on-demand and storage grows incrementally. At four game weeks per real day, the current compact-schema lower bound after two real years is about 2.89 TB at full materialization, 720 GB at 25%, and 290 GB at 10%. Telemetry and capacity planning must expose materialized-system count, retained growth, event throughput, and projected budget exhaustion. The cost target does not authorize deletion of retained facts or changes to simulation semantics. | Current |
| 2026-08-23 | Player commands and due scheduled events pass through one durable ordered input queue; logical-time advancement does not. Future-event indexes establish eligibility only. While ingress is empty, the scheduler advances the authoritative clock directly in its own journalled transaction and leaves future work indexed. A following transaction admits the now-due timestamp-free payload, after which only the ordinary queue consumer applies it. Admitted queue sequence is final and event category never reorders it. Simultaneously eligible schedules use their global creation IDs solely for stable admission. Rule dependencies are expressed causally by scheduling a consequence after its prerequisite commits. This supersedes the 2026-07-29 queued-clock formulation. | Current |
| 2026-07-25 | CE sector/subsector/hex star mapping is not used; navigation and placement use the game's 3D map model. | Current |
| 2026-07-25 | Interstellar travel uses the standard CE Jump drive; alternative drives are out of scope. | Current |
| 2026-08-13 | Flight Plan can request one server-generated route through every active accepted-task stop assigned to the commanded ship. A bounded beam-search heuristic, capped at 48 partial candidates, preserves pickup-before-delivery precedence, ranks deadline risk before estimated travel time, consolidates shared stops, and permits revisits. It is intentionally fast rather than factorial or provably optimal. The selected order is then plotted with the authoritative known-system, ship, elapsed-time, and continuously carried-fuel model using directly importable carried or port fuel; Flight Plan preview remains the authoritative deadline check. | Current |
| 2026-08-22 | Flight Plan checkpoint authority and plan completion are orthogonal. Hold waits for arrival watch; Through permits standing-policy continuation while absent. Exactly one terminal marker belongs to the last step and ends the plan only after that Hold or Through behavior completes. Generated task and ordinary routes default independently to Through, configurable as BBS-local proposal preferences. Task deadlines require completed docking, and preview uses numbered warning footnotes with typed step references to identify manual arrival-watch requirements. Legacy Terminal authority decodes as Hold plus terminal. | Current |
| 2026-08-22 | BBS-polity generation version 2 accepts a site only when at least one inward gateway endpoint is already in Sol's system-to-system J-2 component. Because the cluster template is internally J-2-connected, every newly generated BBS capital has a J-2 path to Sol and later clusters extend the same connected frontier. Persisted version-1 clusters remain unchanged historical universe state. | Current |
| 2026-08-24 | BBS-polity generation version 3 considers eligible existing unvisited, unaligned, inhabited frontier systems in Sol's J-2 component as inward polity members, with the version-2 external-anchor form retained as a fallback. An existing member keeps its exact coordinates, immutable seed, celestial details, and names while only its polity affiliation changes. Reused-member and external-anchor candidates both have outward and lateral families, four rotations, and bounded deterministic three-dimensional variation; sixteen ordered geometry variants are tried only as needed, and invariant checks use a three-parsec spatial index. This avoids repeated straight or coplanar clusters, allows placement alongside materialized space, and keeps ordinary BBS additions below the five-minute ceiling. The founding contact's shortest plotted J-2 route to Sol remains the visited bridge, and persisted version-1 and version-2 homes remain historical state. | Current |
| 2026-08-23 | Universe Atlas frontier systems are materialized/plotted systems that have not been established or physically visited. A versioned `system-visits` aggregate retains only the earliest visit second, never player identity; the fixed initial catalogue begins visited at game second zero, every BBS polity member and its founding contact route begin visited at that polity's materialization second, and Jump arrival marks its destination. Atlas snapshots expose only a visited boolean and can toggle violet diamond markers around unvisited systems. Storage-format-1 backfill uses the initial catalogue, current player arrivals, committed disclosure choices, and mapping publications; older visits with no surviving evidence remain unrecoverable. | Current |
| 2026-08-23 | Every player Jump breakout, whether at plotted deep-space coordinates or a stellar system, resolves the canonical six-parsec sphere around the actual arrival position in the same transaction as Jump completion. Six parsecs is the rules-supported universe maximum and does not shrink to the active ship's rating. Every stellar contact in that sphere enters the captain's carried charts as a private unresolved observation unless an existing mapping state is already stronger. Repeated arrivals are coverage-idempotent; historical storage-format-1 holes are repaired on revisit rather than bulk-generated during startup. J-2 remains a connectivity and BBS-gateway constraint, not the frontier materialization radius. | Current |
| 2026-08-23 | Fresh-universe initialization first resolves every cell intersecting the convex hull of the 43 fixed CNS5 systems, then materializes only the still-unresolved portions of the six-parsec sphere around each fixed system. Domain-separated initialization entropy makes this shell stable for a given initialization input. Generated shell contacts are unaligned, universally plotted at game second zero, inherited in new-captain charts, and remain unvisited until a player ship physically arrives. The hull-first order prevents the frontier sampler from inserting systems inside the hardcoded catalogue volume. | Current |
| 2026-08-23 | Away from a berth, represented living crew and awake passengers consume physical provisions. Docked ordinary crew arrange their own food and do not draw ship stores; the captain consumes one ship person-day by default, falling back when stores are empty to an automatic liquid-credit meal at twice the ship's monthly-package price per person-day, rounded up. Without either source the captain goes unfed. Every represented character persists consecutive unfed days: the first three cause no damage, then CE Routine (+2) Endurance checks occur daily with cumulative DM-1 per prior check; failure causes 1D6 damage that cannot recover until fed. Ordinary shipboard water is assumed available, so food shortage does not also trigger dehydration. | Current |
| 2026-08-23 | Generated traffic may originate, arrive, or transit only through systems in the monotonic visit aggregate. Plotting, publication, settlement, and local daily simulation do not cross that boundary. The fixed catalogue, every complete BBS polity, and the plotted J-2 route actually crossed by its founding contact are seeded as visited; ordinary frontier systems enter only on a player arrival. The simulation's generated-traffic topology and observer projections are derived from the same aggregate and expose links only between visited endpoints. | Current |
| 2026-08-23 | BBS materialization resolves its protected three-parsec core before materializing the still-unresolved part of a six-parsec shell around every polity member. The complete polity begins visited and traffic-active; its unaligned stub and generated shell remain frontier and enter new local captains' starting charts. When the site lies beyond an existing plotted frontier, a deterministic shortest J-2 route from its inward anchor to Sol represents the founding contact voyage and becomes visited, creating a narrow traffic-active bridge without rerolling the plotted map. One polity-originated headline mail packet carries the complete registered-system dossier; receiving any public packet from a polity merges all of that polity's member systems together, not its unaligned frontier. | Current |
| 2026-07-28 | Standard Jump drives may target empty-space staging volumes; no star, mass, beacon, or Zimm point is required. A double-tanked Jump-1 ship can traverse up to two parsecs in two Jump-1 legs without refueling. A planned double Jump has a mandatory one-game-day midpoint turnaround for position fixing, inspection, problem resolution, and second-plot correction. A purchasable double-Jump tape contains two independently resolved, time-indexed Jump-1 plots and is valid only for its departure/turnaround window; end-to-end clean success is `p²` when each leg succeeds with probability `p`. Two Jump-1 allocations use the same jump-fuel volume as one Jump-2 allocation, so Jump-1 trades lower drive/capital volume for about fifteen days of travel and two risk exposures. Jump-2 is not a universal minimum. Full semantics and open details are in `docs/interstellar-jump-operations.md`. | Current |
| 2026-07-28 | All 27 current starting offers select ships fitted and fueled for at least one Jump-2 transit. Hudson `ship-192` revision 2 is J-2 with 53 tons of cargo and costs MCr51.219. Crusoe `ship-193` revision 2 is a freight-first J-2 frontier trader with 92 tons of cargo, 12 staterooms, 12 low berths, one steward in a seven-person crew, and costs MCr85.559; passenger, mail, charter, and evacuation work are supplemental rather than its defining business. Jump-1 remains valid for non-starter designs and possible later reviewed packages. | Current |
| 2026-07-29 | Initial crew naming uses a complete role/name roster. The server supplies role-specific default callsigns. Enter accepts the entire default roster; selecting a letter shows that senior crew member's fixed characteristics and skills and permits renaming. Additional positions represented by a role are aggregate supporting personnel and are not individually named during creation. | Current |
| 2026-07-29 | Setup revision 1 assigns every new captain and named crew member one existing non-Jack-of-all-Trades skill to improve. The default is the role package's first/primary skill (Leadership for the captain). CT-RPC and the person record carry target skill, CE-derived needed weeks, and current weeks; current starts at zero. Captain proposals carry the complete assignment and are validated against `Skill Total + desired level`. Crew proposals carry the selected target only, and the server derives the duration from the fixed role package. The default level-2 courses require 15 weeks for the captain and 11 weeks for crew. The door permits changing either target and displays progress. Each queued calendar week advances training independently of watch status and completes the course atomically at the required total. | Current |
| 2026-07-29 | Every instantiated player has six universally reachable managers: Crew, Ship, Task, Message, Known Universe, and Operations Ledger. Universal availability permits inspection of committed state known in the current frame; it does not make every mutation legal in every phase. Crew covers roster, assignments, training, and readiness. Ship covers status, damage control, cargo/supplies, maintenance, and refit planning. Task covers accepted obligations and standing policies but no longer owns the executable route. Message covers discrete delivered material; Known Universe is the sourced, potentially stale operational model; Operations Ledger covers combat-career orders, traffic, prizes, cruises, crew pressure, and physically delivered warrants. An isolated encounter cannot reveal information received outside that frame. Full boundaries are in `docs/universal-managers.md`. | Current |
| 2026-07-29 | Crew service/home appointments and active watch duties are separate. Each named person has zero or more active role IDs: empty means off watch and eligible for full rest; several roles support CE duty doubling and its ordinary simultaneous-action consequences. Roles may be staffed by several named people except Pilot, which permits only one. Captaincy and command authority are not watch roles and persist while the captain is off watch. People persist current STR/DEX/END, injury, fatigue, treatment, service availability, shore location, salary arrears, morale, and loyalty. Daily healing distinguishes rest from active duty; first aid, inpatient care, and queued surgery are real transactions. | Current |
| 2026-07-29 | `Docked` is derived from a concrete ship attachment to a body and facility, including landed/orbital state, berth where relevant, arrival time, fees, and locally available services. The phase menu provides Cargo Exchange, Jobs and Passage, Fuel and Supplies, Shipyard, Personnel, Banking and Accounts, Authorities, and Depart. Universal Crew, Ship, Task, Message, and Known Universe views are linked rather than duplicated. Implementation order is location, facility snapshot, menu, fuel/supplies/departure, proper repair/maintenance, then the larger economy. Full design is in `docs/docked-operations.md`. | Current |
| 2026-07-29 | Message Management and Known Universe share delayed delivery, time, authentication, confidence, and provenance machinery but are separate stores and interfaces. Messages are discrete chronological communications; Known Universe is a subject-oriented collection of immutable observations plus deterministic cached projections. Observations retain observed and acquired times, source, confidence, conflicts, and optional source-message references. Knowledge is scoped to physical ship or institutional repositories and synchronizes only by contact, carried data, or mail; common player ownership never creates an ansible. Full design is in `docs/known-universe.md`. | Current |
| 2026-07-29 | The OpenDoors client implements a Docked Operations menu with `C/J/F/Y/P/B/A/D` for Cargo Exchange, Jobs and Passage, Fuel and Supplies, Shipyard, Personnel, Banking and Accounts, Authorities, and Depart. `U` enters the universal-manager shell; that shell includes `K` for Known Universe. Every offered action uses server-supplied state. Persistent facility capabilities control fuel, chandlery, ordnance, yard, medical, personnel, banking, and authority availability in both quotation and commit; absent services are hidden and remain authoritatively rejected. | Current |
| 2026-07-31 | Ordinary player-facing door text is always in-world. It names captain/crew actions and shipboard, port, financial, legal, navigation, or communications concepts—not clients, servers, RPCs, snapshots, database records, revisions, phase checks, implementation status, or authoritative state. Explicit diagnostic, operator, licensing, and fatal-error contexts are the only exceptions. The Known Universe screen is a paged navigation library with direct-jump filtering and system dossiers rather than a raw record catalogue. | Current |
| 2026-07-31 | Known Universe includes a read-only, ship-specific course plotter from the present system to a known primary or between any two known primaries. It returns fastest (elapsed time, then fuel cost) and cheapest (purchased-fuel credits, then time) routes through only the captain's carried knowledge. Routing accounts for Jump rating, tank endurance, current fuel for present-location plots, A/B refined-fuel stops, primary-to-locus time, and gas-giant skimming only when the ship has scoops and processing capacity. Frontier estimates use the nearest generated gas giant, mean CE skimming time, processor throughput, and current orbital geometry; actual operations still roll time and failures. Until their models exist, payroll, maintenance, port fees, hazards, and encounter delays are explicitly excluded rather than hidden behind a misleading total-cost figure. | Current |
| 2026-08-13 | Task-offer route availability is a server-produced, offer-keyed ledger assessment. A checkpoint-aware fastest-course search carries elapsed time and remaining fuel from the commanded ship's current system through a required remote pickup and onward to delivery, rejects pickup after offer closure, and reports both pickup and final arrival. The client does not issue independent per-offer plots or skip delivery feasibility for remote origins. | Current |
| 2026-07-31 | Generated systems and principal worlds receive independent seed-derived setting names. Polity affiliation is data and is never substituted for a name with `<polity> N`/`<polity> Capital`; principal worlds are never called `<system> Primary`. BBS capital worlds retain the sysop-selected BBS name, while Sol, Earth, and other explicitly established astronomical system names remain fixed. Cultural naming overlays may later be placed under sysop control without changing physical generation. | Current |
| 2026-07-31 | Every player prompt has a visible back/cancel action until its state-changing command is submitted. Cancellation returns exactly one stage, preserves previously accepted wizard choices, and never sends a partial proposal. `Q` is preferred when it does not collide with a valid selection; another printable key is explicitly labeled when it does. | Current |
| 2026-07-29 | Interplanetary movement and local operations use atomic, bounded continuation plans. A client may collect several prompts into one proposal containing ordered destinations and preauthorized terminal actions. Each later movement and checkpoint is a separate scheduled engine transaction; no database transaction remains open over game time. Facility traffic, convergence zones, destination approaches, Jump loci, docking, landing, and skimming always run encounter and readiness checks but do not force a UI stop when the next action was authorized and remains valid. Encounters, failed validation, plan completion, and deliberate replanning suspend execution. A plan naming an onward system may initiate Jump automatically; a plan ending at the Jump locus holds there. The same rule applies to docking, skimming, rendezvous, and other finite parameterized actions. Full semantics are in `docs/interplanetary-operations.md`. | Current |
| 2026-08-01 | The executable route belongs to a phase-level Flight Plan interface invoked by `Depart` and available while travelling, not to Task Management or any universal manager. `Depart` commits an initial system so ordinary cargo, passengers, and mail can be selected for it, then starts the outbound maneuver without requiring a complete later itinerary. Until a movement or checkpoint is processed, the captain may append or replace routing en route, including the next system; existing carriage and task obligations remain unchanged and known diversion consequences must be shown. Known Universe supplies candidate routes and Tasks supply objectives through shortcuts. A calculated course grants no offline authority: only explicit through-points use standing encounter policy while absent, and the terminal arrival waits until the player is connected. This contract is implemented by Roadmap Milestone 4. | Superseded by the 2026-08-22 authority/completion split |
| 2026-08-26 | Only a physical Jump-space leg prevents maneuver replanning. During every in-system leg or hold, a replacement Flight Plan may point the ship to the primary port, Jump locus, a selected belt, or a lawful frontier-fuel body. The server derives the ship's current position and velocity, evaluates the destination's moving orbital position and velocity, and solves the shortest whole-second two-burn intercept bounded by effective thrust in G; gravity is omitted and a common inertial-frame velocity cancels from the relative calculation. The exact trajectory coefficients are persisted against the active leg so restart and repeated diversion preserve momentum. Superseded travel, activity, and contact schedules are cancelled atomically; carried obligations are not rewritten. | Current |
| 2026-08-01 | The authoritative market implements all six Common Goods and 35 generic revised *Bounded Fortune* trade goods without optional PI examples. Finite system/day stock and shared consumption are durable; trade-code, Broker/Charisma, legality, and generated tariff effects are applied. Purchase outcomes are 80/90/100/120 percent and sale markups are 30/15/2/0 percent. Supplier/buyer research is timed queued work which produces dated confidence ranges in Known Universe. Exact milliton cargo and whole-credit rounding remain authoritative. | Current |
| 2026-08-13 | Cargo Exchange buy and sell rows show an absolute universe-wide commodity market-value span (minimum, Q1, median, Q3, maximum) with the captain-specific current quote marked. The span is not conditioned on the current captain; help states that Broker, Charisma, local trade codes, events, tariffs, and the ordinary spread can change the actual quote. Purchase favorability runs low-to-high while sales reverse it, and every green/yellow/red band also has a textual price judgment. Mean is intentionally omitted. | Current |
| 2026-08-03 | Merchant research produces finite, expiring, revisioned leads; reservation escrows ten percent and does not refund an expired or released opportunity payment. Named market events persist exact stock/price effects and causal agency news. Ordinary freight, passage, and mail use a previewed atomic manifest for the committed destination. Remote offer claims and their replies physically race through mail; no custody transfers until the captain receives the award. Task withdrawal, cancellation, custody return, capped default, and sealed dispute filing are revision-checked transactions; disputed obligations retain collateral and capacity. Fixed-system and mobile-captain private mail is physical and priced at Cr1 per started KiB per charged hop per started TTL week within a one-to-52-week TTL. Destination assistance is an optional Cr350,000 annual dockside policy without cancellation refund. | Current |
| 2026-08-01 | Starting title/finance is authoritative: traders have 20% equity and 80% debt, monthly principal of price/240 over a 480-month schedule, plus mandatory insurance escrow; privateers are sponsor-owned and navy ships institution-owned with restricted service funds. One standard accounting month of grace precedes default. An impound order originates as private mail at the captain's home and has no authority in a remote system until physically delivered there. | Current |
| 2026-07-31 | Revisioned Flight Plan preview/commit owns ordered waypoint intent separately from the current physical leg. Acceptance schedules seed-derived safe-locus travel using catalogued thrust. Separate journaled events consume one catalogued jump-fuel load, enter a standard one-week Jump, approach the destination, and create a terminal/hold/through arrival checkpoint. A body waypoint may name an exact gas giant or lawful wilderness water/ice source and whole-ton quantity; the three-leg fueling operation retains that body and resumes the next authorized route step after returning to port. Outbound unprocessed destinations may be revised atomically; committed legs may not. Player travel and background work use the same input queue; due time controls eligibility, global creation ID stabilizes simultaneous admission, and no category has priority. `GetTravelStatus`, `GetFlightPlan`, and the door expose carried stage/due/fuel/plan state. | Superseded in authority shape by the 2026-08-22 split; scheduling remains current |
| 2026-07-31 | The live clock is fixed at 28 game seconds per real second, exactly four game weeks per real day and six real hours per standard Jump week. It advances whenever the server process is running, independently of connection count, and freezes during downtime. Startup re-anchors monotonic process time at the last committed game second; no wall timestamp is persisted and no downtime catch-up occurs. One coalescing pulse source feeds bounded work slices through the authoritative engine queue. If simulation is behind, commands execute at committed time while lag is reported and drained; target wall time is never exposed as committed state. | Current |
| 2026-08-03 | Player protocol version 2 carries reliable phase/checkpoint/encounter events, typed merchant/combat-career operations, message action references, market leads/reservations/events, carriage manifests, ship-bound Tasks, physical crew condition/service/shore state, private correspondence, insurance, starting terms, revisioned facility-backed dock services, exact Galactic coordinates, deep-space holds, explicit Jump navigation/risk authority, and executable plotted fuel purchases. The C++ transport retains its independent receive dispatcher. Storage format 1 and every internal record codec begin at 1; no legacy readers or migration paths exist. | Superseded for the Flight Plan codec by the 2026-08-22 legacy Terminal reader |
| 2026-08-01 | Internal simulation invariant, never player-facing: exactly one one-kilogram, Cr1 cargo object named `apple pie (made from scratch)` exists universe-wide. It moves through ordinary background shipping and is offered as ordinary cargo at its current port, with no exceptional value, protection, signaling, or AI behavior. Sale, theft, jettison, wreckage, and custody changes do not reset anything. Actual destruction sets a durable reset condition; the engine invokes the normal full-universe initialization boundary while preserving BBS enrollment, credentials, and sysop configuration. Never explain or hint at this relationship in game UI, news, help, or setting prose. | Current |
| 2026-07-31 | Newly materializing a system changes authoritative simulation state but tells no remote repository. Discovery creates local observations. On arriving somewhere not known by the current repository to be publicly mapped, the captain must be prompted to send a free public mapping notification, send a paid encrypted direct filing to Earth, withhold, or withhold and add the system to the editable captain-private Secret Systems list. No response withholds; disclosure is never implicit. The list travels with the captain's physical repository. A public package propagates and merges from the ship's system even when default filters hide it. A direct filing follows one private route, remains opaque to relays, and causes Earth to originate the public announcement only if it wins the bounty. Another observer may independently report a withheld system. A new BBS polity remains automatic headline-level news. | Current |
| 2026-07-31 | The Federation pays a standing award for the first valid discovery filing about a previously unknown settled system to commit to its Earth repository. The structured discovery package is the filing; Earth-receipt queue order, not observation, dispatch, nearer-system receipt, or claimant return, determines priority. Payment is committed on Earth and its receipt or transfer propagates normally by mail. The initial published award formula is `round_up_Cr1,000(1.10 × reference_two_jump_cost)`, using two complete standard J-2 legs and only direct fuel, four-week crew/life-support, maintenance, navigation, and ordinary port costs of `ship-72` Smollett, the orderly/mixed privateer starter. Mortgage, interest, charter payment, debt service, depreciation, and all other capital/financing costs are excluded, as are exceptional loss and cargo capital. The exact first credit value awaits the direct operating-cost audit. BBS/bootstrap materialization is ineligible. | Current |
| 2026-07-31 | Electronic-mail sender tariffs have four classes: news accepted by an agency is free; validated broadcast public-service messages are free; constrained public-key distribution and revocation are free; private or other non-public-service mail pays a small payload-, TTL-, and route-dependent charge. Fixed-system mail purchases one exact known path and is charged only for its hops. Mobile-identity mail purchases the full replicated hold sphere; covered systems retain encrypted copies until authenticated delivery, receipt propagation, or expiry. TTL is always elapsed game time, not hops. Stable IDs make duplicate delivery harmless. | Current |
| 2026-07-31 | A message admitted to universal broadcast retains sparse per-system/frontier state only while propagating. When every currently applicable public repository has it, a monotonic `universally_seen` bit is set and the completed per-system rows are discarded. Mail follows discovery automatically: later-created repositories inherit the completed universal-feed checkpoint and immutable archive, so the bit never reopens. In-progress broadcasts add newly discovered applicable systems normally. This state means institutional availability, never that every player has read the item, and scoped or private mail is excluded. | Current |
| 2026-07-31 | Private mail is encrypted to published encryption keys and authenticated with signing keys. Public keys are freely distributed and have no capture value by themselves; capturable ship/captain assets are private keys, protected credential stores, and unlocked sessions. Compromise permits decryption or impersonation until signed revocation reaches each relying system through ordinary mail. Replacement keys do not decrypt old ciphertext, and revocation cannot undo prior disclosure or accepted signatures. | Current |
| 2026-08-03 | `ROADMAP.md` is the authoritative development-order and milestone-status document. Detailed subsystem documents own mechanics and implementation inventories but cannot implicitly change priority by naming their own next step. Milestones 0 through 6 are complete and Milestone 7 is current. After side work, development resumes at the roadmap's `Resume Here` section. | Current |
| 2026-08-03 | Player RPCs are exhaustively classified as observations or transactions. Both remain ordered and journaled; only transactions retain exactly-once command results and full journal deliveries, while observation journals retain the request without an indefinitely stored snapshot. Player CT-RPC version 8 and storage format 2 accept only current shapes; the undeployed server performs no compatibility migrations or startup index reconstruction. See `docs/rpc-and-storage-schema.md`. | Current |
| 2026-08-03 | Milestone 7 is Field Alpha and Operations, not initial multi-BBS support. OpenDoors owns door switches/drop-file parsing and reads a `CTConfig` custom directive. The BBS-local protected binary identity registry binds real-name-plus-record-index (or configured handle) to monotonic nonreused UInt32 IDs; partial identity changes require explicit sysop rename/reindex/retire. Server access is active, suspended, or irreversibly removed, with immediate session ejection. Tax and naval demotion are signed private mail instruments and do not apply until physically delivered; tax arrears bear no interest and proceeds enter an unspendable polity ledger. Civilization-axis changes are signed public-service orders: the capital changes locally at issue, while other member systems retain their prior per-system policy until the physical copy arrives. Trade/combat weights hostile and military encounters; chaos/order shifts effective seed-derived CE law and enforcement encounter weights. See `docs/field-alpha-operations.md`. | Current |
| 2026-08-03 | The admin status RPC reports committed/game time, durable queue depth, BBS/player/system counts, sessions, and storage format. Live backups are labeled server-side LMDB copies plus a manifest at an engine-queue boundary and restore only into the same alpha storage version. Default limits are 64 pending authentications, 256 game sessions globally, and 64 per BBS. SIGINT/SIGTERM notify players and stop/join the authoritative engine. Capacity scale remains a manual benchmark. | Current |
| 2026-08-03 | Every accepted merchant obligation names one performing ship. Freight and passage custody is physical cargo or a passenger manifest, sourced-goods contracts consume only matching player-owned lots, partial terms pay only the delivered fraction, and recurring performances reset individually. Settlement/default is exactly once across restart. Trade-in is forbidden while the hull has active obligations, entrusted cargo, or passengers. | Current |
| 2026-08-03 | Personnel remain at an exact ship or shore facility. A departed ship leaves completed leave/treatment personnel awaiting recall at that berth. Surgery resolves at booking but applies its injury change only through the queued completion event. Payroll shortfalls are proportional and persist per-person arrears; arrears lower morale, and injury, fatigue, and morale feed the shared CE resolver. | Current |
| 2026-08-02 | The combat baseline uses 1,000-game-second shared-view joint orders, initiative resolution, a 70% default offline risk policy, and fixed deterministic search of at most 64 candidates, eight finalists, and 256 three-round rollouts each. It implements named crew actors and CE task DMs, catalog weapons and persistent ammunition, missiles and defenses, damage, withdrawal, surrender, boarding, escape craft, delayed real-traffic intervention, and persistent post-combat repair priorities. The shared personnel, task-evidence, port-facility, communications, and player-facing playability boundary is complete. | Current |
| 2026-08-02 | Naval careers receive monthly Cr6,000 plus Cr2,000 per awarded grade, promotion boards every 180 days at 0/10/25/45/70 service points, rank-limited issued hulls, and institutional operating logistics. Privateers receive scoped commissions and physically mailed prize-court claims worth 10/20/30% of realizable value with advances capped at half. Pirates receive real-traffic leads, optional commissions, free predation, cruise articles and crew pressure, and 10–30% fences. | Current |
| 2026-08-02 | Unlawful attacks create warrants that propagate through physical mail. Same-polity enforcement applies a lower recognition threshold than foreign enforcement; foreign action requires stronger severity/evidence, and local law/corruption can alter settlement. A warrant has no local effect before its message arrives. The door's Operations Ledger exposes orders, traffic interception, prizes, cruises, crew pressure, warrants, and settlement. | Current |
| 2026-08-09 | Ship attachment is orthogonal to traffic locus. A berthed or landed vessel is present at its port/body locus but cannot be attacked inside the attachment through ordinary interception. Selecting an attached player vessel creates a named departure watch after the interceptor clears its own berth and settles accrued charges. Ships holding at a modeled locus may also keep persistent all-craft or exact-catalog-class pickets; matching deterministic traffic and player arrivals or departures enter the ordinary combat path. Gas skimming is spaceborne and exposed, while wilderness surface fueling is landed until liftoff. | Current |
| 2026-08-22 | A capable operational yard may accept a player commission for any admitted runtime starship whose TL and displacement fit that facility. The contract takes a 20% deposit and creates a persistent undelivered fleet hull with 80% secured principal; the hull cannot receive command, crew, or stores until its catalog construction time completes. Maintenance, warranty, finance payments, and berth aging begin at delivery. Newly materialized BBS capitals require an A/B/C starport as well as TL12, guaranteeing a local capable yard without changing existing universes. CT-RPC 8 carries the commission catalog and order; outcome codec 15 reads older ship-market outcomes as having no commission list. `ship-214` Leavitt is the TL11, 200-ton minimum exact J-3 pathfinder: two J-3 fuel loads, four plant weeks, five actual crew positions in three staterooms, integral scoop, six tons of processors, one ammo-free beam laser, and five tons cargo. | Current |
| 2026-08-22 | Asteroid extraction is a Flight Plan Belt Cycle, not an instantaneous dock action. Generated lodes are persistent shared physical resources, observations are private, and discovery grants no exclusive claim. Mining drones handle 1D6 x 10 tons of feedstock per set per day; Trade (Prospector) governs six-hour searches and daily extraction, mineral refineries produce composition-specific trade goods, and an unrefined operation stows Basic Unrefined Ore. Off-berth power fuel burns continuously. The protected egress reserve prices every remaining filed Jump at its 184-hour maximum plus one day. Field recovery success resumes the loop; failure immediately takes the prevalidated egress. `ship-215` Humboldt Foundry is the TL11 commissionable 200-ton refinery conversion. Manual mining, laser drills, and formal claims/licenses remain deferred. Closed MgT2 material is not cited or added to OGL provenance; the implemented mechanics and prose are independently expressed. | Current |
| 2026-08-23 | Routine fuel quotations identify gas giants, planets, moons, and icy belts; only usable sources are selectable, and wilderness entries explicitly represent unoccupied routine access. A scoop can collect without a processor. The captain explicitly chooses whether to refine a collected batch, and may later refine unrefined fuel while docked or safely holding. Preview reports exact selected-quantity travel/collection/processing and both normal and failed totals without exposing the roll. Refining is Average (8+) Engineer (Power)/EDU; failure doubles time and Effect -6 or worse damages the Jump drive, falling back to maneuver drive then fuel system. Mixed tanks burn refined and unrefined fuel proportionally, and every Jump applies the normal -2 DM whenever its actual proportional burn includes unrefined fuel. | Current |
| 2026-08-09 | Interception intent is explicit. Armed attacks enter combat immediately; boarding/inspection pickets issue a heave-to demand and enter combat only on refusal, including the offline inspection-response-to-combat-policy cascade. Lawful inspection authority is institutional: a refusing target receives a warrant, the local picket withdraws, and capable enforcement traffic responds on modeled movement time. Pirate admission follows security geography from incoming Jump loci through gas giants to exclusion from safe systems, and pirates decline targets that clearly overmatch them. | Current |
| 2026-08-03 | A satisfied or revoked warrant is not removed by omniscient state change. The resolving authority files a second signed public-service instrument, and each system continues enforcing the original until that resolution physically arrives. Unpaid customs assessments and mutiny or theft of an institution-owned command enter the same delayed warrant pipeline. | Current |
| 2026-08-03 | Terminal command loss retains a surviving captain through rescue, custody, or parole and applies a seven-, fourteen-, or thirty-day recovery interval according to disposition. Death requires a named successor. Docked irrecoverable bankruptcy is allowed only on an actually defaulted secured account; it liquidates the complete managed fleet and all balances/stores before issuing the named successor the original starting-offer class under a fresh 80-percent lien. Career, reputation, and legal consequences remain attached to the BBS/player identity. | Current |
| 2026-08-01 | Routine ship maintenance is continuing onboard upkeep with a monthly accounting interval; it is not a monthly overhaul and does not erase age or accumulated wear. Ship condition must separately represent damage, temporary repair, routine-upkeep performance, calendar/use wear, installation age/cycles, refit/overhaul, refurbishment, and replacement. Clement sources define refits as four-to-six-week extensive service costing four monthly maintenance payments, refurbishment as system replacement/life extension, and port capability by hull size. They do not provide a complete dynamic wear curve or general starship repair-material quantity, so those must be explicit Cepheus Trader adaptations. See `docs/ship-condition-and-maintenance.md`. | Current |
| 2026-08-02 | Ammunition and life-support provisions are physical quantities in the persistent ship record. Combat consumes and writes back fitted ammunition; queued system days consume represented-crew person-days. Docked purchases, fuel, proper repair, refit, and new/reconditioned component replacement share one stale-revision-checked service contract. Frontier fuel orders name an exact quoted body. Routine-upkeep pricing includes ordinary repair items, so the game does not invent a per-hit starship-spares quantity. | Current |
| 2026-08-01 | CE game-play training is a coarse calendar rule: one selected skill per game week for the formula's required number of weeks. The source does not require off-watch status or daily/hourly activity accounting. Normal duties, travel, and brief encounters do not subtract study hours; only genuinely prolonged impossible states may interrupt a week. Natural healing still uses the separate rest/activity distinction. See `docs/skill-training.md`. | Current |
| 2026-08-01 | A hydrographic code is evidence of water, not permission to take it. Populated worlds normally control their water and do not permit routine free wilderness fueling. Ordinary water/ice collection uses known accessible unoccupied worlds, moons, or ice-bearing asteroid belts, still subject to ownership claims and local law. Explicit licenses, contracts, emergency permission, or naval/public authority can grant access. Extraction by force is a hostile operation that may require combat or evasion and creates legal, political, and news consequences; it is never an ordinary cheapest-route stop. | Current |
| 2026-07-25 | The engine tracks mail time; delayed news must be increasingly significant to become a local headline. | Current |
| 2026-07-25 | Mail is propagated by a persistent due-time event queue and next-hop delivery records, not by periodic scans; system arrival reads a materialized feed, while broad news may use aggregated route-frontier/mailbag events. | Current |
| 2026-07-29 | Interstellar electronic mail uses beacon-mediated opportunistic common carriage, never a dedicated fleet. A configured ship already making an exact hop accepts a signed destination mailbag at the departure locus and uploads it automatically at the arrival beacon for the route-invariant token payment. Mail never causes or redirects a transit and has no guaranteed frequency: where ordinary traffic is frequent, mail is frequent; where no suitable ship departs, it waits indefinitely. Urgent passengers, physical parcels, and secure objects use passage, cargo, or contracts instead. Inter-authority financial clearing still travels by mail rather than creating an ansible-like bank. | Current |
| 2026-07-25 | Messages are retained indefinitely by default; TTL means an absolute game-time expiry, which stops delivery/acceptance without deleting the historical record. | Current |
| 2026-07-25 | BBS materialization is a high-significance institutional news event that propagates by mail and remains headline-worthy at distance; a player discovery remains local if withheld, while a dispatched routine uninhabited-system notice propagates but is hidden by default filters. | Current |
| 2026-07-25 | Server/UI communication is a strict-schema binary protocol; ASN.1-style expressiveness is preferred, while serialization and RPC/data-flow are separate decisions. | Current |
| 2026-07-26 | Native Cap'n Proto RPC is unsuitable because connection/event-loop affinity conflicts with independent receive and transmit execution; Cap'n Proto serialization remains a fallback for a small custom CT-RPC envelope. | Current |
| 2026-07-26 | The leading protocol candidate is Cap'n Proto serialization plus a small project-specific CT-RPC envelope over TLS-PSK; native Cap'n Proto RPC and larger HTTP/2 RPC stacks are out of scope unless this minimal layer proves inadequate. | Provisional |
| 2026-07-25 | Effective RPC availability is filtered by ship phase and permissions; a standard invalid-command response covers unknown/currently-invalid commands and includes the authoritative current phase. | Current |
| 2026-07-25 | Jump is an asynchronous ship lock lasting several real hours; the player may remain connected in planning mode using cached data, but cannot commit fresh trades or ship actions until arrival. | Current |
| 2026-07-25 | Each BBS represents a home system in a roughly ten-system polity; its highest-TL planet anchors the polity. | Current |
| 2026-07-25 | A sysop selects the local combat-to-trade civilization profile, but all profiles remain connected over potentially many months of game time, not real time. | Current |
| 2026-07-25 | The originating BBS sysop is the final moderator for their polity and originating players, but cannot grant gameplay advantages. | Current |
| 2026-07-25 | The center of the combat-to-trade spectrum is a viable privateer role combining authorized naval work, raiding, bounty activity, and trade. | Current |
| 2026-07-25 | Combat intensity and legal status are independent: navy, privateer, mercenary, and pirate operations can share tactics but differ in authority, targets, support, rewards, and consequences. | Current |
| 2026-07-25 | Naval play follows a captain loop of orders, readiness, patrol, information, contact decisions, encounters, repair/resupply, and reporting. | Current |
| 2026-07-31 | Space combat follows CE's one-kilosecond turns and initiative-ordered vessel activations. Each activation commits one atomic joint crew order with prioritized standing reactions. Online play begins with a complete conservative legal order for every named crew member or aggregate station team, which the player may edit before submission. | Current |
| 2026-07-31 | A classic rules-based tactical controller may be invoked online and acts on deadline or disconnect. The player sets a minimum estimated probability of satisfying the current encounter objective. The controller uses only captain-visible censored state and ordinary legal actions; below the threshold it attempts real withdrawal, then may surrender or abandon to escape craft when those are the best survivable final actions. | Current |
| 2026-08-02 | A small HTTPS companion web app is a Milestone 8 quality-of-life feature. It pairs a browser to a player for standards-based Web Push activation-soon and activation-ready notices without becoming a gameplay client. Delivery is optional and best-effort, reveals no tactical detail on the lock screen, and never changes deadlines or offline-controller behavior. | Current |
| 2026-08-25 | Browser alert preferences separate advance Hold warnings, advance Through/standing-orders warnings, and immediate interruption notices. Both advance classes use the receiver's chosen lead time; a captain may disable routine delegated-action notices while retaining warnings for every boundary that will wait for orders. Encounters and other consequential interruptions originate immediate notices regardless of the selected waypoint authority. | Current |
| 2026-08-25 | The portable communicator exposes a receiver-scoped temporary inbox backed by the Web Push delivery ledger. It lists only unexpired alerts successfully delivered to that authenticated browser session and opens their existing detail; pending, failed, expired, and other-receiver alerts are excluded. Receiver revocation invalidates the session, while later push-service revocation does not erase its already delivered entries. Existing alert expiry remains the deletion boundary, so this recovers dismissed notifications without creating a permanent message archive. | Current |
| 2026-07-31 | After offline combat, a crew retaining control automatically attempts feasible recovery in capability order: Life Support, Maneuver drive, Jump drive, weapons. Temporary battle repairs expire first; prerequisite work is dependency-aware, every step consumes actual time and resources, and unavailable permanent or facility repair remains outstanding. | Current |
| 2026-07-31 | Third parties observe combat emissions only after `separation / c` and may receive ambiguous evidence rather than authoritative aggressor identity. A response requires a physical intercept and joins only at actual arrival; a late response becomes pursuit, rescue, arrest, salvage, or aftermath. Player ships have online choices and persistent offline intervention policies. Enforcement vessels generally investigate within their remit; other computer-controlled ships generally intervene only with clear aggression and comfortable relative overmatch, modified by allegiance, law, orders, crew, condition, and resources. | Current |
| 2026-07-29 | Pirate play is a hybrid of unrestricted free predation, fallible leads to existing authoritative targets, unreliable or deniable patron commissions, and a crew-defined pirate cruise. Leads and commissions never spawn victims; they reference real traffic, ships, cargo, passengers, facilities, or events generated by ordinary simulation and may become stale, contested, or trapped. The cruise articles define hunting scope, conduct, shares, ship funds, and participation expectations. Navy captains receive orders, privateers receive legally scoped commissions and prize adjudication, and pirates remain free to ignore all structure and run amok. Full semantics are in `docs/pirate-gameplay.md`. | Current |
| 2026-07-25 | Naval wealth can support continued service, retirement, civilian ship ownership, privateering, or investment; it does not directly upgrade a state-issued warship. | Current |
| 2026-07-25 | In combat-heavy polities, armed merchantman ownership normally follows naval service; mutiny/theft is possible only as a severe-consequence alternative. | Current |
| 2026-07-25 | Mail clippers are a fast, low-upside courier role with little wealth or progression; they are not a primary competitive career. | Current |
| 2026-07-25 | Trader progression is the first economy balance baseline; naval, privateer, and pirate paths are calibrated against its seeded net-income and session-value distribution. | Current |
| 2026-07-25 | Merchant balance assumes traders discover and converge on the best dependable controllable route factors, while market, availability, travel, legal, and encounter conditions remain stochastic. | Current |
| 2026-07-25 | Ordinary merchant trade should support the commercial upgrade ladder; exceptional cargo, prizes, missions, and windfalls accelerate or rescue progression but are not required for it. | Provisional |
| 2026-07-25 | Merchant play will use generated economic contracts at ship scale and a later merchant-house layer for compounding throughput, credit, assets, and agents; it will not depend on authored RPG quests or arbitrary income multipliers. | Provisional |
| 2026-07-25 | The merchant turn is an arrival/offer/claim/port-settlement/refuel-and-load/departure loop; multiple contracts may overlap, constrained by physical capacity, liquidity, deadlines, and communication races. | Current |
| 2026-07-25 | Ordinary passengers are capacity-market bookings (“four berths to Spicio II”); high-security or time-sensitive passenger transport is handled as explicit private/charter contracts with premium and legal risk. | Current |
| 2026-07-25 | Refueling is a three-way in-system choice—buy refined/unrefined fuel, gather free surface water/ice, or skim a gas giant—with cost, time, legality, piracy, equipment, and unrefined-fuel reliability tradeoffs. | Current |
| 2026-07-25 | Since energy is not separately priced, accessible water/ice is normally the cheapest frontier fuel source; landing, collection, processing, legal, wear, and opportunity costs determine whether it is actually optimal. | Current |
| 2026-07-25 | Surface-to-orbit fuel movement is primarily equipment, labor, throughput, time, and risk; no mass-proportional delta-v charge is imported into the CE model. | Current |
| 2026-07-25 | Standard commercial hulls normally carry one Jump plus four weeks of power-plant fuel; routine merchant play therefore includes a refueling decision at each destination. | Current |
| 2026-07-25 | Frontier skimming, water landing, and refining use Clement's explicit failure/damage checks as a provisional adaptation, with Zimm-drive damage rewritten for standard CE drive and hull systems. | Provisional |
| 2026-07-25 | Distributed hulls may carry external skimming boats in docking clamps or cradles rather than full hangars; the boats need transfer equipment and impose loaded-mass/Jump penalties, while full hangars primarily enable onboard repair. | Current |
| 2026-07-25 | The ship computer supplies ledgers, forecasts, alerts, and bounded routine policies; players retain commitment and risk decisions, while helper scripts receive no faster or privileged state-changing path. | Provisional |
| 2026-07-25 | The ship computer may present time-stamped, probabilistic source estimates for commodities and contracts, with stale-data, travel, availability, cost, and deadline uncertainty made explicit. | Provisional |
| 2026-07-25 | Complete-ship captures are risky prize claims, not clean cash: title, registry, condition, crew, fencing capacity, settlement delay, heat, and seizure risk determine realized value. | Provisional |
| 2026-07-25 | Privateer prizes are expected recurring income through delayed adjudicated shares or authorized transfers; assessment uses realizable net value, not hull list price. | Provisional |
| 2026-07-25 | CE law level is sysop-controlled world/polity state, with auditable effects on detection, enforcement, permissions, and penalties. | Current |
| 2026-07-25 | Cross-polity crime records and warrants travel by mail; foreign enforcement considers issuing-polity authority, diplomacy, local corruption, and possible payoffs. | Current |
| 2026-07-25 | Interstellar banking is a gameplay system that provides mobility while exposing fugitives to freezes, seizure, delayed settlement, and identity scrutiny. | Current |
| 2026-07-25 | Banking uses competing stagecoach/robber-baron-style houses, branches, correspondents, bills, and protected couriers; internal settlement remains opaque. | Current |
| 2026-07-25 | The no-ansible/J-Drive model produces a nineteenth-century-like delayed-information and regional-autonomy topology at interstellar scale, without being a literal historical reskin. | Current |
| 2026-07-25 | Target useful play is 15–30 minutes daily, with a soft ceiling around 45 minutes and viable progress from one 30-minute session. | Current |
| 2026-07-25 | Parsecs are the universe's base distance unit; the materialized universe expands in visit order to the maximum Jump radius around visited systems, following the Milky Way density model. | Current |
| 2026-07-25 | There is no master universe seed. Ordinary systems receive independent persisted 256-bit CSPRNG seeds; conditioned batches draw prospective seeds from a cryptographic stream rooted in fresh operation entropy and persist only accepted per-system seeds. Named cryptographic feature streams preserve existing values when generation expands. | Current |
| 2026-07-25 | Stellar existence, population/activity, and trade/mail connectivity are separate. Empty systems are normal; population tiers generate markets, traffic, and reasons to visit. | Current |
| 2026-07-25 | The economy and routine world activity use persisted aggregate statistical state; major polity/region events use a proactive, ordered structural-event schedule and emit mail/news when they occur, rather than being reconstructed on observation. | Current |
| 2026-07-25 | Generated celestial data, mutable places/institutions, player/assets, obligations/information, and simulation-control state are separate authoritative domains; feeds and snapshots are rebuildable indexes. | Current |
| 2026-07-25 | The authoritative server is Rust; the OpenDoors-capable UI is C++17 or newer with a language-neutral IPC protocol. | Current |
| 2026-07-25 | Universe coordinates are three parsec-valued doubles, positive coreward/spinward/galactic north. | Current |
| 2026-07-25 | CE coverage audit: merchant wealth, naval rank/ship-class progression, ownership, crew, engineering repairs, and new-character recovery are compatible with CE; macroeconomics beyond local trade is unnecessary for the baseline. | Current |

When a decision changes before deployment, replace or remove the obsolete
entry. Keep this document limited to instructions and decisions that still
govern the current code and design.

## Open questions

Track unresolved choices explicitly rather than allowing accidental defaults:

- Rust server runtime details and C++17 OpenDoors UI/door-host integration;
- first-deployment persistence migration, backup, and rollback policy;
- post-bootstrap TLS-PSK rotation and revocation policy;
- combat disconnect grace, optional notifications, and asynchronous return
  presentation;
- combat-emission and sensor-evidence rules, default player intervention
  profiles, NPC overmatch doctrine, and pursuit/rescue/aftermath transitions;
- astronomical source data, universe ownership, and generation coverage;
- coordinate serialization precision and astronomical position epochs;
- any Earth Sector corrections to the implemented *Unmerciful Frontier*
  celestial generator and the future mutable world-overlay schema;
- atomic Jump-arrival integration of the implemented inhomogeneous-Poisson
  sampler and density bound, per-system seed storage/backup, and generation
  migration rules;
- initial Federation settlement extent `E₀`, route/mail connectivity
  thresholds, and the mechanics for turning
  a discovered resource or settlement into a sustained trade route;
- mail routing, message queues, delivery schedules, and polity boundaries;
- exact mail-feed schema, per-system/player cursor retention, route-frontier
  aggregation, and delivery-queue checkpoint/idempotency policy;
- event timestamps, mail-time calculation, news aging, and headline scoring;
- structural-event types and rates, polity/route aggregate transitions,
  `SystemDay` work and catch-up budgets, scheduler fairness, and safe aggregate
  catch-up equivalence;
- simulation clock advancement, frame transitions, compression, concurrency,
  and out-of-frame observability;
- exact frame rates, pause/idle behavior, and conflict resolution when groups
  act at different rates;
- BBS home-system generation, polity size, civilization-profile effects, and
  long-distance connectivity across the profile spectrum;
- sysop permissions, player-origin ownership, moderation actions, audit logs,
  appeals/recovery, and anti-advantage enforcement;
- exact CE-to-simulation simplifications;
- traffic-field decay and neighbor feedback, population/TL/trade activity
  scaling, location-specific flow and dwell rates, contact/intercept
  thresholds, encounter frequency, combat pacing, and economy balancing;
- seeded trader-cohort assumptions, commercial upgrade ladder, resale/debt
  treatment, working-capital reserve, insurance/tax/fee policy, and the
  percentile game-time targets for each progression stage;
- generated merchant-contract types, event/news-driven offers, deadlines,
  advances, collateral, penalties, merchant-house unlock conditions, fleet or
  agent scaling, remote-claim races, communication/escrow delays, management
  capacity, and anti-runaway leverage limits;
- market-intelligence snapshot schema, freshness/decay, source-availability
  distributions, broker/agent updates, confidence display, and the boundary
  between known data and hidden current market state;
- deeper prize title/registry and lien checks, regional fence capacity,
  installment or syndicate settlement, and the risk-adjusted
  pirate-versus-merchant progression target;
- pirate intelligence confidence, staleness, competition and traps; patron
  reliability; detailed crew shares and ship funds; loyalty/mutiny outcomes;
  and regional underworld facilities;
- daily action-budget accounting, split-session bonuses, and anti-grind rules;
- naval mission generation, patrol objectives, authority/ROE, convoy traffic,
  rewards, collateral consequences, and fleet-scale progression;
- deeper privateer reputation, target-authority diplomacy, and conflicts among
  letters of marque;
- naval prize/mission wealth, retirement or half-pay, transfer out of service,
  personal ship ownership, investment, and influence sinks;
- combat-heavy starting careers, lawful naval-to-private transitions, mutiny/
  theft consequences, warrants, pursuit, asset seizure, and crew/faction
  reactions;
- independent combat-intensity versus legitimacy/faction dimensions, including
  recognition, transponders, bases, intelligence, logistics, and enforcement;
- persistent per-BBS door profile and geometry defaults during drop-file
  startup, plus operator control of CP437 transport conversion;
- interstellar identity, crime records, warrant propagation, diplomatic
  recognition, banking instruments, settlement delays, seizure, bribery, and
  corruption effects;
- secure bank-mail routes, remittances, delivery status, fees,
  branch/correspondent networks, courier risk, pickup/authentication, and
  opaque internal settlement;
- banking-house competition, bills/letters/drafts, branch access, courier
  security, robbery, blockade, and political influence;
- test strategy and continuous-integration environment.

## Stable encounter-intelligence and pursuit decisions (2026-08-23)

- Arrival and en-route encounters require one real local traffic projection;
  never fabricate a fallback hull when the projected set is empty.
- Contact hull data is sensor-qualified. Radio-only and transponder-only
  results expose no hull class, approximate results expose a generic size
  class, and positive identification still exposes only that generic size
  class alongside any identified vessel or transponder identity. The server
  never sends an exact hostile catalog identity or hull class, including in
  combat views.
- Pirates assess the player with their own sensor result and break off when the
  estimate does not support at least even odds. A detected break-off offers
  Pursue and Continue Course; a Through course defaults to Continue Course only
  after one combat-turn response window.
- Pirate demands take all entrusted freight plus a tiered, milliton-rounded
  percentage of owned lots, limited by the pirate hull's available cargo
  capacity. Allocate by realizable value per ton. Unique objects are
  indivisible and are skipped when they do not fit. Only physical freight loss
  creates authenticated claim evidence.
- Running and chasing use the same CE spacecraft-pursuit relation with roles
  reversed. Establish and break are opposed Pilot tasks; ties preserve the
  status quo. Maintaining pursuit is a significant action, adds +1 attack DM
  after the first maintained turn to a maximum +4, and ends at Medium range, a
  seven-point target speed advantage, departure, or a successful break. A ship
  may change speed each turn by no more than effective maneuver thrust.
- Documented entrusted-freight loss is filed from the Task ledger by the best
  of Admin or Advocate against difficulty 8. Authenticated encounter/custody
  evidence supplies +2; voluntary fight, flight, or boarding before loss adds
  +1; threat adjusts the task from -1 favorable through +2 overwhelming.
  Sustained claims excuse performance and release collateral; denied claims
  use the existing capped default path.
- These additive fields remain CT-RPC version 8 for the current unreleased
  line; do not bump the protocol solely for this work. The radio ordinal
  formerly named `surrenderDemand` is semantically `pirateDemand` without
  changing its numeric value.

## Instructions for LLM-assisted work

Inspect the repository and the relevant CE chapter before proposing changes.
Cite local files and distinguish source facts from adaptations and
assumptions. Prefer small, reversible changes and avoid scope creep. Ask for
clarification when an ambiguity would materially change the game; otherwise
choose a reasonable reversible default and record it here. Update this
document when a design choice becomes stable, and leave a concise note about
remaining risks or follow-up work.
