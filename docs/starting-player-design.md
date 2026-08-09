# Starting Player Design

*Status: provisional design, 2026-07-26*

This note defines the information and choices presented when a player creates
a captain. It intentionally leaves exact allowances, crew-skill price
multipliers, and final ship variants for balance simulation.

## Polity Axes

The BBS polity has two independent orientation values:

- **Trade to combat** describes local institutions, opportunity, doctrine,
  danger, and the normal degree of ship militarization.
- **Chaos to order** describes institutional reach and reliability: contract
  enforcement, title and registry, banking, command legitimacy, prize courts,
  corruption, and the consistency of official support.

The second value is not the Cepheus Engine Law Level. CE Law Level is a
separate world characteristic describing restrictions and enforcement. A
stable low-regulation polity and a failed pirate haven can both have low CE
Law Levels, while an orderly polity can be either permissive or authoritarian.

The two values map startup into a discrete **3 × 3 catalog**. The
trade-to-combat dimension has trade-focused, mixed, and combat-focused bands;
the chaos-to-order dimension has orderly, contested, and chaotic bands. Each
of the nine cells contains exactly three predesigned starter offers:

1. a trader;
2. a privateer; and
3. a navy or other public-service command.

The player sees the three ships in the home BBS's cell, not all 27 offers.
These are 27 starter designs and obligation packages, but they do not need to
be 27 unrelated hulls. Several offers can be variants of the same hull with
different equipment, title, charter, logistics, and crew establishment.

The locally aligned career is intentionally the strongest starting choice:
trade-focused polities favor the trader, mixed polities favor the privateer,
and combat-focused polities favor the navy/public-service command. “Strongest”
means the best expected local prospects after ship efficiency, available work,
financing, support, legal authority, and obligations are considered—not
necessarily the most expensive hull. The other two careers must remain viable
for players who want to work against their home environment, but should pay a
visible mismatch cost. Institutional order changes the form of that advantage
and the risks surrounding it; it is not a second grant of raw ship value.

The canonical design mapping for the 27 offers is stored in
[`catalog/starting-offers.toml`](../catalog/starting-offers.toml):

| Local emphasis | Institutional order | Trader offer | Privateer offer | Navy/public-service offer |
| --- | --- | --- | --- | --- |
| Trade-focused | Orderly | Hudson (`ship-192`) | Sinbad (`ship-45`) | Challenger (`ship-53`) |
| Trade-focused | Contested | Hatteras (`ship-161`) | Sinbad (`ship-45`) | Nemo (`ship-166`) |
| Trade-focused | Chaotic | Crusoe (`ship-193`) | Drake (`ship-66`) | Marco Polo (`ship-38`) |
| Mixed | Orderly | Sinbad (`ship-45`) | Smollett (`ship-72`) | Perry (`ship-204`) |
| Mixed | Contested | Hatteras (`ship-161`) | Nemo (`ship-166`) | Hawkwood (`ship-90`) |
| Mixed | Chaotic | Robur (`ship-168`) | Silver (`ship-140`) | Marque (`ship-74`) |
| Combat-focused | Orderly | Argosy (`ship-61`) | Marque (`ship-74`) | Decatur (`ship-195`) |
| Combat-focused | Contested | Robur (`ship-168`) | Revenant (`ship-87`) | Cook (`ship-95`) |
| Combat-focused | Chaotic | Silver (`ship-140`) | Blackbeard (`ship-206`) | Revenant (`ship-87`) |

Nineteen immutable catalog designs support the 27 packages. Reusing a design
does not make two offers identical: title, equity, financing, refit allowance,
crew terms, legal authority, support, orders, and exit consequences belong to
the versioned offer rather than the ship template.

The selected designs have these rule-derived baseline characteristics:

| Design | Tag | Hull | Jump | Cargo | Catalog role |
| --- | --- | ---: | :---: | ---: | --- |
| Hudson | `ship-192` | 200 tons | J-2 | 53 tons | Trader |
| Sinbad | `ship-45` | 200 tons | J-2 | 42 tons | Trader |
| Challenger | `ship-53` | 120 tons | J-2 | 2 tons | Armed survey courier |
| Hatteras | `ship-161` | 300 tons | J-2 | 67.5 tons | Frontier trader |
| Nemo | `ship-166` | 300 tons | J-2 | 21.7 tons | Patrol ship |
| Crusoe | `ship-193` | 300 tons | J-2 | 92 tons | Frontier trader |
| Drake | `ship-66` | 300 tons | J-2 | 42.5 tons | Raider |
| Marco Polo | `ship-38` | 200 tons | J-2 | 88.7 tons | Trader |
| Smollett | `ship-72` | 400 tons | J-2 | 48 tons | Escort-raider |
| Perry | `ship-204` | 300 tons | J-2 | 22 tons | Patrol frigate |
| Hawkwood | `ship-90` | 550 tons | J-2 | 113.5 tons | Patrol frigate |
| Robur | `ship-168` | 300 tons | J-2 | 63.6 tons | Fast armed trader |
| Silver | `ship-140` | 400 tons | J-2 | 44.5 tons | Boarding raider |
| Marque | `ship-74` | 400 tons | J-2 | 16 tons | Escort brig |
| Argosy | `ship-61` | 400 tons | J-2 | 182 tons | Merchant freighter |
| Decatur | `ship-195` | 300 tons | J-2 | 17 tons | Naval corvette |
| Revenant | `ship-87` | 600 tons | J-2 | 41.5 tons | Heavy raider |
| Cook | `ship-95` | 600 tons | J-2 | 28 tons | Patrol corvette |
| Blackbeard | `ship-206` | 600 tons | J-2 | 55 tons | Raider |

The Hudson and Crusoe are deliberate Jump-2 starter revisions of the
corresponding core examples. The Crusoe is freight-first rather than a
passenger liner: its seven-person establishment includes one steward, and its
12 staterooms leave eight rooms for passengers when the crew uses four
double-occupancy rooms. The removed accommodation and second steward turn the
J-2 revision's hold from 40 tons into 92 tons while retaining charter,
passenger, mail, and evacuation work as supplemental opportunities.

Six earlier slots named hypothetical refits that never became catalog
designs. The mapping uses admitted ships instead: Sinbad for the armed
merchant, Challenger for the Jump-capable customs/public-service craft, Marco
Polo for the improvised militia auxiliary, Argosy and Robur for the two armed
or convoy-merchant concepts, and Silver for the blockade-running trader. No
startup transaction may silently alter those designs to recreate the old
worksheet concepts.

The 800-ton Odysseus armed merchant is deliberately not a starter. Its
361.5-ton hold gives it far more earning leverage than the combat-row trader
should receive. It is a natural early commercial upgrade for a player who has
built enough capital and organization to move beyond the starter scale.

The 300-ton Stahlstadt missile ship is also not a starter offer. Its 0.7-ton
hold is mission stowage rather than meaningful trade capacity, making it
suitable as an encounter or later specialist command. The Marco Polo instead
represents a weak polity's improvised navy: a commercial hull commissioned as
an escort, supply ship, and customs auxiliary while remaining dependent on
freight for much of its useful work.

The catalog is globally defined and balanced. The sysop selects or changes the
polity environment; the sysop does not choose a particular player's ship,
equipment, or credits. CE Law Level, local Tech Level, facilities, diplomacy,
current wars, and market state determine how each fixed offer is described and
which local activities support it, but must not turn the catalog selection
into a per-player grant.

The global catalog also defines nine mechanical upgrade paths, one specialized
for each cell of the trade/mixed/combat crossed with
orderly/contested/chaotic matrix. Each path is presumed to be the work of one
specialist manufacturer or shipyard and can pass through multiple design
families as hull size and capability increase. Design-family membership and
native upgrade-path membership are independent catalog relationships:
variants of a common hull belong to one family and may occupy different
paths, while a path expresses a longer progression and a consistent design
doctrine.

The paths are deliberately allowed to be sparse. A specialist need not offer
a native design at every useful size between a starter and a 5,000-ton ship.
Where a path has a gap, progression may use an explicitly selected design from
an adjacent path without changing that design's family, native manufacturer,
or path identity. This is preferable to manufacturing a redundant catalog
entry merely to complete every ladder.

The sysop may supply local setting prose around those designs: local history,
advertising, institutional associations, reputation, and cultural
interpretation. This does not permit the sysop to change the canonical fit,
price, capabilities, path position, starting resources, or player-specific
offer. Stable catalog IDs remain authoritative even if a local presentation
uses aliases. Players are not locked to their home path and may cross between
paths through ordinary acquisition, prize, service, and refit play.

Each hull and fitted variant referenced here becomes a versioned design
record as defined in [`ship-catalog-records.md`](ship-catalog-records.md).
Starter offers remain separate records: they select a design revision,
allowed customizations, financing or institutional terms, supplies, and crew
pools. They do not duplicate or mutate the canonical ship design.

The four original anchors now have admitted canonical records: Hudson for the
orderly merchant, Crusoe for the frontier merchant, Decatur for the regular
corvette, and Blackbeard for the corrected raider. They are ordinary immutable
catalog designs, not special setup-only reconstructions.

Non-Jump customs cutters and system-defense boats remain useful encounters,
local commands, or short introductory assignments, but are poor general
starting commands for an interstellar game. Challenger fills the trade/orderly
public-service slot without inventing the formerly proposed Jump refit. Cook
is the admitted normalized 600-ton patrol-corvette choice; its standard Jump
drive and all other mechanics come from the active construction catalog.

## The Three Offers

Every new player sees the three viable, persisted offers in the home BBS's
matrix cell. Local terminology can vary, but the mechanical roles remain
recognizable.

### Independent Commercial Charter

This is the trader start. The player receives title or financed equity in a
fully operational ship, a mortgage or comparable local financing obligation,
working capital, and access to ordinary freight, passengers, mail, contracts,
and speculative trade.

An orderly trade polity tends toward an efficient Merchant Trader, reliable
banks, predictable inspections, and dense markets. A chaotic or combat-heavy
polity tends toward an armed, self-sufficient Frontier Trader, weaker credit,
less predictable enforcement, and greater need for fuel and repair autonomy.
A combat-heavy polity can support this role through naval supply, colonial,
or convoy commerce without turning it into a courier or naval career.

### Private Armed Charter

This is the privateer start and the middle of the play spectrum. The player
receives an armed merchant, auxiliary, Q-ship, or raider under a letter of
marque, security contract, syndicate agreement, or local equivalent. The ship
retains meaningful cargo capability.

Orderly versions have narrow rules of engagement, reliable adjudication, and
lower fencing losses. Chaotic versions have broader practical target choice,
weaker recognition abroad, less reliable support, larger crew or investor
shares, and more title and retaliation risk. Prize value is realized through
the existing prize process; the player cannot immediately liquidate the
starter as a debt-free warship.

### Public-Service Commission

This is the navy start. The player commands a polity-owned Jump-capable patrol
ship and receives orders, base access, intelligence, authorized logistics,
salary, and promotion prospects. The player does not own the command.

In an orderly polity this is a regular navy, coast guard, or system service.
In a chaotic polity it may be a militia, clan flotilla, planetary defense
force, or warlord's service whose authority is real locally but poorly
recognized elsewhere. Trade-heavy public service emphasizes customs, rescue,
escort, inspection, and distant anti-piracy work. Combat-heavy service
emphasizes patrol, interception, convoy defense, raids, and fleet operations.

Decatur provides the 300-ton J-2 regular corvette, while Perry provides the
300-ton J-2 patrol-frigate package when fighters and routine patrol are
desired. Lower-capability origins use the commissioned commercial and courier
designs in the matrix rather than silently adding Jump drives to non-Jump
system-defense craft.

## Comparable Resources

Starter balance is based on practical agency and expected progression, not
ship list price. The following are separate resources:

- authority to use a ship;
- player-owned equity in that ship;
- liquid working capital;
- credit, supply, repair, intelligence, and base access;
- legal authority and the expected realizable share of prizes;
- fixed expenses, debt, service duties, investor or crew shares, and exit
  restrictions.

A trader may own substantial equity in an MCr51–86 ship while carrying a
forty-year mortgage. A privateer may command a more valuable armed ship but
hold only a financed or syndicated interest in it. A naval captain may command
an MCr180+ vessel but own none of it. Selling, abandoning, or mutinying with a
ship follows its title and obligation ledger; command access never silently
becomes personal net worth.

The first balance target is comparable expected useful progress over a common
game-time window, with values in the same broad scale. The paths need not have
equal cash flow because merchant wealth, privateer equity and reputation, and
naval rank and command access are different progression currencies.

## Ready-to-Depart Package

Every offer has a complete default configuration. A new player can accept the
recommended fit and crew without understanding the ship-design rules. On
confirmation the ship is:

- undamaged, maintained, registered, and legally operable under its charter;
- fitted with required drives, software, sensors, safety gear, and role
  equipment;
- supplied with refined fuel, standard ammunition where applicable, current
  local navigation data, and an initial life-support reserve;
- staffed to at least the safe operational minimum; and
- provided with enough covered expenses or reserves that the first ordinary
  trip is not an immediate bankruptcy trap.

Customization uses three visibly separate budgets:

1. A **refit allowance** pays only for approved installed equipment. It is not
   cash. Unused state or patron allowance is forfeited; privately financed
   changes alter the ship's equity or debt ledger.
2. A **staffing envelope** controls billets, candidate quality, and continuing
   payroll or crew shares. Navy crew are assigned from an eligible slate;
   merchant crew are hired; privateer crew may negotiate salary and prize
   shares.
3. An **operating reserve** is actual liquid money, trade credit, or an
   authorized service supply account. The UI never presents these as
   interchangeable funds.

Authorized ship costs consume restricted operating credit before the
captain's liquid funds. A naval captain can deliberately misappropriate that
credit only by filing a forged expense receipt; the converted funds become
personal cash, while the durable receipt is exposed to a later accounts audit
and mail-propagated legal consequences. Audit risk combines the absolute false
total, repeated claims, and the fraction of that accounting cycle's ship
expenses represented by fraudulent receipts.

Standard components cannot be stripped and converted into startup cash.
Player-funded later improvements can retain recoverable value according to
the title ledger. This prevents a combat-origin player from taking a costly
warship package, selling its armament, and obtaining a strictly superior
merchant start.

For starter offers derived from 500–2,000-ton Clement designs, replace the
source Z-drive with the minimum CE Jump-2 drive rather than merely relabeling
the larger source installation. The recovered 10 displacement tons become
starter-configurable space. The associated MCr20 construction saving remains
inside package/refit accounting and is not paid to the player as cash. Smaller
source hulls do not have this automatic reserve, so any configurable space
must be deliberately reserved in their starter variant.

Every currently mapped starter is fitted and fueled for at least one Jump-2
transit. This is a starter-package decision, not a declaration that Jump-2 is
the minimum viable player drive. A double-tanked Jump-1 ship may stage through
empty space using the operation defined in
[`interstellar-jump-operations.md`](interstellar-jump-operations.md).
Changing any mapped offer to Jump-1 requires a later package-level review of
fuel endurance, saved drive/refit value, route preview, fifteen-day staged
travel, and the two independent Jump-risk exposures.

## Captain and Crew

Cepheus Engine supplies required crew roles and baseline salaries, but it does
not provide a sufficient market model for hiring crews of different skill.
Cepheus Trader therefore needs a small persistent labor-market layer.
The characteristic and skill alternatives, possible old-CE translations, and
crew-role cross-reference are audited in
`docs/characteristic-skill-audit.md`. Measurements of published captains,
crew, and antagonists are in `docs/character-creation-benchmarks.md`.

Captain customization, fixed starting-crew role templates, the atomic
`CreatePlayer` request, and the normalized `Person` and `CrewService` records
are defined in `docs/player-creation-records.md`. A crew member is not stored
as a name in a ship record: the persistent person owns characteristics,
skills, health, and advancement, while a separate service relationship owns
billets, ship, compensation, morale, loyalty, and availability.

The player's captain has STR, DEX, END, INT, EDU, and CHA plus a compact
functional skill package rather than a full RPG career. Cepheus Trader does
not store core CE Social Standing as a characteristic. Rank, title,
citizenship, reputation, legal status, institutional authority, and
relationships are separately scoped persistent state. Whether specialized
skills use core cascade or Clement base-plus-specialization semantics remains
an explicit design choice. The captain can fill an operational billet,
reducing payroll, but cannot perform simultaneous encounter duties merely
because several skills are present. Each career offer exposes enough fixed
role templates to form a legal minimum crew. Every template assigns the
`[10,9,8,8,7,6]` characteristic multiset appropriately and supplies fixed
role skills.

Starting packages and their setup revision are generated once and stored when
the attested player first enters new-user setup. Reconnecting does not reroll
them. After selecting the ship, the player reviews each initial crew template
and supplies its member's name; the ship determines the visible establishment
and employment terms.

Command loss does not normally replace the captain. A surviving captain
remains the player character through rescue, custody, or parole and resumes
command after the applicable delay with the same career and legal ledger. A
dead captain requires a named successor, who inherits those consequences
rather than receiving a clean identity.

Irrecoverable bankruptcy is available only while docked after a secured ship
account is actually in default. It liquidates the complete managed fleet,
stores, cargo, balances, and financing before creating the named successor.
The successor receives the original starting-offer class under a fresh
80-percent lien, with no retained cash, while career, reputation, and legal
state remain attached to the BBS/player identity. These rules prevent death or
bankruptcy from farming ships, components, cargo, or unusually skilled crew.

## Startup Dossier

The player must be able to understand both the local setting and the
consequences of each offer before committing. The startup dossier is the home
polity's knowledge, not an omniscient view of current universe state.
After creation it becomes initial sourced content in the current ship's
Known Universe repository; see
[`known-universe.md`](known-universe.md).

### Origin and Polity

- the two polity orientation values, with plain-language consequences;
- government, capital, member systems, important factions, and institutional
  stability;
- the homeworld's CE profile: starport, size, atmosphere, hydrology,
  population, government, Law Level, Tech Level, bases, trade codes, and
  travel zone;
- local shipyards, repair and refit limits, fuel sources, banks, insurers,
  exchanges, prize courts or fences, naval bases, and crew market;
- taxes, tariffs, prohibited goods, inspections, transponder and weapon rules,
  privateering law, prize shares, and normal warrant treatment;
- current allies, enemies, wars, recognized letters of marque, border policy,
  and major trade or military commitments; and
- banking and mail-network reach, expected settlement times, and normal
  communication latency.

### Known Neighborhood

- known systems and three-dimensional coordinates;
- distances and one-Jump reachability overlaid separately for each offered
  ship;
- current estimated in-system travel time to important facilities based on
  orbital position;
- known starports, repair levels, safe and frontier fuel sources, trade
  characteristics, bases, and travel advisories;
- expected mail routes and age of available information;
- delayed market ranges, contract density, piracy or conflict reports, and
  confidence rather than hidden current values; and
- a broad indication of where more trade-oriented, combat-oriented, orderly,
  or chaotic regions are believed to lie.

Unknown systems remain unknown. Every dynamic fact should carry its source,
observation time, receipt time, and confidence. Home-system observations may
be current; distant market, political, and danger information is delayed by
ship-carried mail, normally relayed as electronic mailbags through standard
Jump-locus beacons.

### Offer Comparison

For every charter the UI displays:

- hull, drive performance, fuel endurance, cargo and passenger capacity,
  armament, defenses, small craft, and legal restrictions;
- current condition and the complete default fit;
- title, equity, liens, mortgage, charter, command authority, and what happens
  on sale, loss, desertion, retirement, or death;
- minimum and recommended crew, uncovered monthly payroll, life support,
  mortgage, maintenance, fuel, and ammunition costs;
- cash, credit, service supply, and estimated runway;
- reachable systems and plausible first destinations;
- current representative commercial offers, privateer work, or naval orders;
  and
- the progression and exit path: refinance or buy a larger merchant, acquire
  privateer equity and reputation, earn promotion and larger commands, or
  lawfully transfer between careers.

The comparison should show practical consequences such as expected monthly
burn, cargo available after obligations, number of known destinations in
range, and whether the selected crew can safely operate every installed
system. Raw ship price alone is not a useful comparison.

## Setup Flow

The intended new-user flow is:

1. Read the origin and neighborhood dossier.
2. Query `GetCaptainCreationOptions`, then customize and name the captain
   using the returned Starting Captain pools.
3. Query `GetStartingShipOffers` and compare the three persisted career offers
   in the home BBS's matrix cell.
4. Query `GetStartingShipOptions` for an inspected offer, then select, name,
   and customize the ship locally.
5. Query `GetStartingCrewPlan` with the captain and ship drafts, review each
   fixed role template, name each initial crew member, and assign required
   billets. Requery after a relevant ship or captain change.
6. Review title, obligations, monthly burn, reach, and remaining reserves.
7. Submit `CreatePlayer` with the complete proposal and enter the game docked.

The final confirmation is one authoritative `CreatePlayer` transaction.
The preceding query RPCs are read-only committed-snapshot views. Earlier
setup choices are local drafts and reserve nothing. The final request submits
all selections and their setup revision; the server validates the captain
pools, offer, ship options, crew names and slots, establishment, and billets,
then creates
every player, person, ship, employment, and starting-estate record atomically.
The persistent offer and setup revision belong to the server model. Detailed
payload shapes are in `docs/player-creation-records.md`.
