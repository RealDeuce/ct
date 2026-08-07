# Potential Open Game Content Sources

*Inventory date: 2026-07-26*

This is the working catalogue of local rules material that might inform
Cepheus Trader. It is deliberately broader than
[`ogc-provenance.md`](ogc-provenance.md): a work belongs here when it may be
useful, while it belongs in the active provenance record and the consolidated
Section 15 notice only after material from it enters the repository or game.

The present scope is:

- Cepheus Engine rules and directly relevant mechanical supplements;
- Clement Sector rules, operations, ships, and statistical examples; and
- Earth Sector rules, ships, and statistical examples.

The inventory covers the current, non-`Obsolete` PDFs under:

- `/usr/home/admin/RPG/2D6/Cepheus Engine/`;
- `/usr/home/admin/RPG/2D6/Clement Sector/`;
- `/usr/home/admin/RPG/2D6/Earth Sector/`; and
- Clement Sector adventures found under `/usr/home/admin/RPG/2D6/Adventures/`.

| Current PDF group | PDFs screened | PDFs containing an OGL/OGC/PI declaration |
| --- | ---: | ---: |
| Cepheus Engine | 13 | 4 |
| Clement Sector | 89 | 78 |
| Earth Sector | 32 | 31 |

These counts are files, not distinct works: they include an earlier vehicle
guide and byte-identical ship PDFs stored in both setting directories. The
catalogue collapses those duplicates.

Archived editions remain relevant when a current work inherits their
Section 15 notices, but they are not treated as preferred mechanical sources.
Maps, blank forms, cheat sheets, and other aids without an OGC declaration
are recorded separately rather than silently discarded.

## How to Read This Catalogue

`Potential` does not mean approved for copying. Before activating a source:

1. read its actual OGC and Product Identity declaration;
2. identify the exact mechanics being adapted;
3. copy its complete Section 15 notice, including inherited notices, into
   `OPEN_GAME_LICENSE.md`;
4. add it to `docs/ogc-provenance.md`; and
5. replace protected names and setting expression before material enters
   game data.

The Section 15 column below is an ancestry index, not replacement legal text.
`+ self` means the work also includes its own copyright-notice entry. PDF
spelling and punctuation must be preserved when the exact notice is copied.

## Common Section 15 Ancestry

### `BASE-CE`

The Cepheus Engine SRD notice contains these works:

1. *Open Game License v 1.0a*;
2. *High Guard System Reference Document*;
3. *Mercenary System Reference Document*;
4. *Modern System Reference Document*;
5. *Swords & Wizardry Core Rules*;
6. *System Reference Document* (2000);
7. *System Reference Document* (2000–2003);
8. *T20 — The Traveller's Handbook*;
9. *Traveller System Reference Document*; and
10. *Cepheus Engine System Reference Document*.

Most Independence Games works inherit `BASE-CE`, then add some combination of
the Clement/Earth works relevant to that book.

### `CS-CORE-2026`

The 2026 Clement core books inherit `BASE-CE` and list:

- *The Clement Sector* (2013);
- *Clement Sector: The Rules* (2016);
- *Clement Sector Core Setting Book 2.0*;
- the Cascadia, Franklin, Hub, Sequoyah, and Colonies 2.0 subsector books;
- *Tree of Life: Altrants in Clement Sector*;
- *Wondrous Menagerie: Uplifts in Clement Sector*;
- *Clement Sector* (2021); and
- the applicable 2026 core book or books.

The 2026 setting volume additionally lists *Unmerciful Frontier*,
*Balancing Act*, *The Almighty Credit*, and *Outlaw*.

### `A&F-3`

The third-edition naval-architecture lineage is `BASE-CE` plus:

- *The Clement Sector*;
- *Clement Sector: The Rules*;
- *Clement Sector Core Setting Book 2.0*;
- *The Anderson and Felix Guide to Naval Architecture 2.0*;
- *The Anderson and Felix Optional Components Guide*;
- *Clement Sector* (2021); and
- *The Anderson and Felix Guide to Naval Architecture*, version 3.

Individual ship books frequently add a subsector, fleet guide, older ship
volume, or older edition of that ship to this lineage.

### `EARTH-2023`

*Earth Sector Third Edition* inherits `BASE-CE` and lists:

- *The Clement Sector*;
- *Clement Sector: The Rules*;
- *Clement Sector Core Setting Book 2.0*;
- *The Anderson & Felix Guide to Naval Architecture 2.0*;
- the Cascadia, Franklin, Hub, Sequoyah, and Colonies 2.0 subsector books;
- the five older Wendy's fleet guides for those subsectors;
- *Near Stars*;
- *Unmerciful Frontier*;
- *Earth Sector* (2019); and
- *Earth Sector (for Clement Sector Third Edition)* (2023).

### `TRADE-2023`

*Bounded Fortune* inherits `BASE-CE` and lists:

- *Earth Sector* (2019);
- *The Almighty Credit*;
- *Clement Sector Third Edition*;
- *Diverse Roles*, third edition;
- the third-edition Cascadia, Franklin, Sequoyah, and Colonies books;
- *Outlaw*, third edition;
- *Port of Entry*;
- *Gear*; and
- *Bounded Fortune*.

### `VDG-2019`

The current *Vehicle Design Guide* lists the Modern SRD, both general SRDs,
the Traveller SRD, the *Vehicle Handbook System Reference Document*, and the
*Vehicle Design Guide*. Its declaration opens its rules text but expressly
excludes the supplied vehicle designs, artwork, product titles, and marks.

## Highest-Value Rules Sources

| Source/file | Potential contribution | OGC and Product Identity handling | Section 15 ancestry |
| --- | --- | --- | --- |
| `Cepheus Engine/cepodnew.pdf` | Authoritative base tasks, characteristics, skills, trade, economics, world generation, ships, travel, combat, and encounters | All text is OGC except product titles and the Cepheus Engine and Samardan Press marks | `BASE-CE` |
| `Cepheus Engine/erratapages.pdf` | Corrections already incorporated into `cepodnew.pdf` | No independent declaration; treat as companion errata, not a separate source | Same as the corrected SRD |
| `Cepheus Engine/Low-Tech-Weapons3.pdf` | Low-technology weapon statistics and combat equipment | Expressly designates all text as OGC; artwork has separate attribution | Its own `BASE-CE`-derived notice + *Low Tech Weapons* |
| `Cepheus Engine/Vehicles/VDG27Uploadv3.pdf` | Vehicle construction, chassis, armour, weapons, equipment, movement, and vehicle combat | Rules text is open; included vehicle designs, artwork, titles, and marks are excluded | `VDG-2019` |
| `Clement Sector/Clement Sector Core Rulebook.pdf` | 2026 consolidated task, skill, combat, equipment, vehicle, spacecraft, travel, trade, world, encounter, and referee rules | Mechanics are OGC; all setting names, ship/vehicle names and classes, organizations, maps, fiction, art, and the term “altrant” are PI | `CS-CORE-2026` |
| `Clement Sector/Clement Sector Core Character Creation Book.pdf` | 2026 characteristics, skill and career structure, species mechanics, life events, benefits, and character construction | Mechanics are OGC; characters, species/setting names, fiction, art, and “altrant” are PI | `CS-CORE-2026` |
| `Clement Sector/Clement Sector Core Setting Book2.pdf` | Polity, economy, history, travel, world, and campaign-scale reference; useful mainly as a simulation checklist | Only mechanics are OGC; nearly all expressive setting content is PI | `CS-CORE-2026` plus the setting-volume additions |
| `Clement Sector/Obsolete/Clement_Sector_Third_Edition.pdf` | Earlier combined rules used by existing design audits; useful for checking changes in the 2026 split books | Mechanics are OGC; Clement setting expression and proper names are PI | `BASE-CE` + Clement legacy line + *Clement Sector Third Edition* |
| `Clement Sector/Anderson_and_Felix_Third_Edition.pdf` | Spacecraft design, components, drive performance, cost, crew, large vessels, small craft, and worked designs | Mechanics are OGC; ship names/classes, example setting, diagrams, and art are PI | `A&F-3` |
| `Clement Sector/Ready Reckoner.pdf` | Fast naval-architecture calculations and design references | Mechanics are OGC; named designs and presentation remain PI | `A&F-3` + *Anderson and Felix Naval Architect's Ready Reckoner* |
| `Clement Sector/Bounded Fortune.pdf` | Merchant operations, speculative trade, cargo, contracts, finance, insurance, passengers, crew, and commercial progression | Mechanics are OGC; corporations, people, places, ships/classes, fiction, and art are PI | `TRADE-2023` |
| `Clement Sector/Port_of_Entry.pdf` | Starport classes, facilities, services, traffic, fuel, cargo handling, encounters, and port operations | Mechanics are OGC; named ports, worlds, organizations, people, and art are PI | `BASE-CE` + *21 Starport Places* + Clement/subsector lineage + self |
| `Clement Sector/Hub Federation Navy Third Edition.pdf` | Navy organization, careers, crews, operations, missions, logistics, fleet construction, and ship examples | Mechanics are OGC; the navy, ranks as setting expression, people, locations, ship names/classes, fiction, and art are PI | `BASE-CE` + all three HFN editions + Hub third edition + self |
| `Clement Sector/Skull and Crossbones Third Edition.pdf` | Piracy, privateering, capture, prizes, fencing, pirate careers, bases, and encounters | Mechanics are OGC; named pirates, groups, worlds, ships/classes, plots, and art are PI | `BASE-CE` + older *Skull and Crossbones* + Clement third edition + self |

`Cepheus Engine/Vehicles/VDG27Upload.pdf` is the earlier local copy of the
vehicle guide; use `VDG27Uploadv3.pdf` for mechanical work unless a
version-difference audit specifically requires it.

## Other Mechanical and Operational Sources

| Source/file | Brief contents | Section 15 highlights beyond `BASE-CE` |
| --- | --- | --- |
| `21 Vehicles Third Edition.pdf` | Twenty-one ground, air, sea, and other vehicle designs; useful as design-system tests and price/stat benchmarks | Clement legacy core + *Vehicle Design Guide for Cepheus Engine* + *21 Vehicles* + Clement third edition + self |
| `Action Movie Physics.pdf` | Optional cinematic action, damage, stunt, and survivability rules | *Action Movie Physics* + self |
| `Artificial Robots in Clement Sector 3e.pdf` | Robot construction, chassis, components, programming, careers, operation, and examples | Clement legacy core + *Artificial* (2018) + Clement third edition + self |
| `Badge Law Enforcement in Clement Sector.pdf` | Law-enforcement careers, agencies, investigations, jurisdictions, arrests, warrants, and equipment | Clement third edition, *Manhunters*, *Outlaw*, and self |
| `Covert.pdf` | Espionage organizations, careers, tradecraft, missions, and equipment | Clement third edition and related crime/organization works + self |
| `Cybersneaks Hacking in Clement Sector.pdf` | Computer intrusion, security, hacking actions, careers, hardware, software, and consequences | Clement third edition, *Interface*, and self |
| `Diverse Roles Third Edition.pdf` | Expanded career catalog and skill/benefit progressions; useful for crew-skill benchmarks | Earlier *Diverse Roles* + Clement third edition + self |
| `GEAR.pdf` | General equipment, weapons, armour, survival gear, tools, electronics, and prices | Clement third edition + self |
| `Hub Federation Ground Forces Third Edition.pdf` | Ground-force structure, careers, units, operations, vehicles, support, and equipment | All three ground-force editions + Clement and Hub third editions + self |
| `Interface Third Edition.pdf` | Cybernetics, implants, installation, availability, side effects, and related careers | Earlier *Interface* + Clement third edition + self |
| `Manhunters Third Edition.pdf` | Bounty hunting, warrants, investigations, captures, careers, organizations, and equipment | Earlier *Manhunters* + Clement third edition + self |
| `Outlaw Third Edition.pdf` | Crime, criminal careers, law levels, organizations, illicit markets, punishment, and evasion | Earlier *Outlaw* + Clement third edition + self |
| `Tree of Life Third Edition.pdf` | Alternate-human construction and careers; useful only for generic characteristic/trait mechanics | Earlier *Tree of Life* + Clement third edition + self |
| `Unmerciful Frontier Third Edition.pdf` | Detailed stellar-system construction on pp. 44–149 (explicitly declared OGC), plus frontier administration, exploration, careers, and equipment; primary source to adapt for stars, multiple systems, orbital zones, planets, moons, and orbital mechanics after removing hex and Zimm assumptions | Earlier *Unmerciful Frontier* + Clement third edition + self |
| `Wondrous Menagerie Third Edition.pdf` | Uplift characteristic, trait, career, and equipment mechanics | Earlier *Wondrous Menagerie* + Clement third edition + self |
| `21 Characters CS3.pdf` | Twenty-one complete characters; useful for aggregate skill and crew benchmarks | Clement third edition + *Diverse Roles* third edition + self |
| `21_Villains_OGL.pdf` | Twenty-one complete antagonists; useful for aggregate competence and encounter benchmarks | Older Clement core works + self |
| `21 Organizations2nd.pdf` | Organization creation/examples and institutional resources | *21 Organizations*, *21 More Organizations*, Clement legacy core + self |
| `21 Pirate Groups.pdf` | Pirate-group organization, resources, ships, methods, and encounter examples | Clement and piracy lineages + self |

All of the Independence Games works in this table use the general declaration
that mechanics are OGC while proper names, organizations, locations,
characters, dialogue, plots, artwork, trade dress, vehicle names, starship
names, and starship classes are PI.

## Geographic, Polity, and Fleet Sources

These have a high ratio of Product Identity to reusable mechanics. They are
nevertheless useful for checking world-generation output, polity scale,
facility distribution, fleet composition, and the statistical relationship
between population, technology, law, trade, and military forces.

| Source group | Individual works | Potential contribution | Section 15 pattern |
| --- | --- | --- | --- |
| Clement subsectors | `Cascadia_Subsector_3e.pdf`; `Franklin Third Edition NA.pdf`; `Hub Subsector Third Edition.pdf`; `Sequoyah Third Edition.pdf`; `The Colonies Third Edition.pdf` | World/system records, governments, law, trade, facilities, organizations, and local ship/fleet examples | `BASE-CE` + corresponding older subsector editions + Clement third edition + self |
| Clement fleets | `Wendy's Guide to Cascadia 3e.pdf`; `Wendy's Guide to Franklin 3e.pdf`; `Wendy's Hub 3ea.pdf`; `Wendy's Sequoyah 3e.pdf`; `Wendy's Colonies 3e.pdf` | Fleet organization, doctrine, bases, ship mix, and named designs | `BASE-CE` + Clement/A&F lineage + applicable subsector and earlier fleet guide + self |
| Clement ground forces | `Tim's Guide to Cascadia.pdf`; `Tim's Guide Hub.pdf` | Ground-force organization, installations, readiness, units, and vehicles | `BASE-CE` + applicable subsector and ground-force works + self |
| Earth core | `Earth Sector Third Edition.pdf` | Earth-region history, polities, worlds, law, trade, routes, fleets, and technology | `EARTH-2023` |
| Earlier Earth core | `Earth Sector.pdf` | 2019 predecessor, retained chiefly because later works inherit its notice | `BASE-CE` + Clement/A&F/subsector/fleet lineage + *Earth Sector* |
| Earth subsectors | `Artemis Subsector.pdf`; `Subsector Sourcebook Adroanzi.pdf`; `Subsector Sourcebook Ashima.pdf`; `Subsector Sourcebook Durga.pdf`; `Subsector Sourcebook Earth.pdf`; `Subsector Sourcebook Gansu.pdf`; `Subsector Sourcebook Hecate.pdf` | World/system records, trade, law, governments, facilities, conflicts, and local ships | `EARTH-2023` or its 2019 predecessor + self; exact inherited set varies by publication date |
| Earth fleets | `Wendys Earth Volume 1.pdf`; `Wendy's Earth Vol 2.pdf`; `Wendy Earth 3a.pdf`; `Wendy's Earth 4.pdf` | Fleet structures, bases, readiness, doctrine, and military/auxiliary ship designs | Earth core + Clement/A&F lineage + preceding applicable fleet works + self |
| Earth technology | `Tech Update 2350.pdf` | Later technology, updated components, vehicles, equipment, and design assumptions | Earth core + A&F/optional-component lineage + self |

World names, maps, subsector names, polity names, organizations, named routes,
ship names/classes, histories, and other setting expression in these books
must not become Cepheus Trader data merely because their associated mechanics
are open.

## Ship and Small-Craft Sources

These works are potential sources for unnamed, mechanically equivalent hull
records. Their published names and classes are PI. Numerical construction
records should be independently checked against the applicable open
construction rules, renamed, and stored as Cepheus Trader OGC data.

Most Independence Games ship books use:

`BASE-CE + Clement legacy + applicable A&F edition + applicable
subsector/fleet/older ship work + self`.

The exact additions vary substantially, so the full notice must be copied
from the selected PDF rather than reconstructed from this summary.

| Source/file | Designs or role covered |
| --- | --- |
| `Alcyone class Interstellar Freighter.pdf` | Interstellar freighter and in-system hauler variant |
| `Atlanta Class Carrier.pdf` | Three carrier blocks and their fighters/bomber |
| `Atlas class Freighter.pdf` | Large freighter, armed merchant, colony, and missile variants plus utility craft |
| `Bridgetown and Cape.pdf` | Escort and system-defense-boat designs |
| `Brightwater Class Personal Yacht2.pdf` | Personal yacht |
| `Coner Class Trader.pdf` | Trader and passenger/steerage variants |
| `Contessa Class Fast Trader.pdf` | Fast trader and supporting craft |
| `Copeline Class Merchant Vessel.pdf` | Merchant vessel and supporting craft |
| `Freedom Merchant Ship.pdf` | Merchant vessel |
| `Grand Duke Destroyer Series.pdf` | Related battle, railgun, and missile destroyers |
| `Hercules Heavy Freighter.pdf` | Heavy freighter, tanker, and transport variants |
| `HSOCS 1 Trent OGL.pdf` | Trent destroyer design |
| `Jinsokuna Chirashi Yacht.pdf` | Armed yacht and associated craft |
| `Knox Class Frigate.pdf` | Original and improved frigates |
| `Lance class Gunboat.pdf` | Gunboat |
| `Lion class Battlecruiser.pdf` | Battlecruiser and ship's boat |
| `Loki Class Q Ship.pdf` | Armed merchant/Q-ship and historical operational context |
| `Milligan Hospital Ship 3e.pdf` | Hospital ship and transfer craft |
| `Opportunity Class Light Trader.pdf` | Several generations/variants of a light trader and carried craft |
| `Pleiades Light Freighter.pdf` | Two light-freighter series and in-system haulers |
| `Roosevelt Class Intercept Destroyer.pdf` | High-performance intercept destroyer |
| `Rucker Class Merchant.pdf` | Merchant, cargo, passenger, casino, escort, and missile variants |
| `Trade Empire Commercial Transport.pdf` | Commercial transport and ship's boat |
| `Type 3 Security Cutter.pdf` | Security/law-enforcement cutter |

The same byte-identical PDFs for Alcyone, Atlanta, Atlas, Brightwater, Coner,
Copeline, Freedom, Hercules, Jinsokuna, Knox, Lance, Milligan, Pleiades, and
Rucker appear in both the Clement Sector and Earth Sector directories. They
are one source each, not two license entries.

### Ships of Clement Sector Collections

| Source/file | Contents |
| --- | --- |
| `SOCS 1-3 082618Bk.pdf` | Hub warships: patrol corvette, attack-boat tender, cruiser, and supporting designs |
| `SOCS 4-6 082918Bk.pdf` | Traders, scouts, small craft, workboats, auxiliaries, pirate craft, and yachts |
| `SOCS 7-9 CE 082918Bk.pdf` | System-defense boats and light warships |
| `SOCS 10-12 CE 082918Bk.pdf` | Merchant, intruder, prospector, troop-transport, and other workhorse designs |
| `SOCS 13 Strikemaster.pdf` | Brig |
| `SOCS 14 Boyne.pdf` | Replenishment ship |

The current standalone Milligan, Rucker, and Atlas books replace the older
*Ships of Clement Sector* volumes 15, 16, and 17 as preferred references, but
their notices inherit those older works.

### Source That Is Not Open

`Clement Sector/Independence-Class.pdf` is a Moon Toad *Quick Ship File* for
a 500-ton armed freighter. Despite reproducing an OGL and initially describing
mechanics as open, its final declaration expressly states: “No portion of
this book is open content.” Do not use its design as OGC. The `IAF` row in
`Ships.ods` therefore lacks an acceptable published OGC source.

## `Ships.ods` Coverage

The updated `/usr/home/admin/RPG/2D6/Clement Sector/Ships.ods` contains 191
ship/craft rows attributed to 48 source abbreviations. It is an index and
research aid, not itself an OGC grant.

Those rows supplied the coverage inventory for the complete active catalog.
All entries in [`../catalog/ships/`](../catalog/ships/) were admitted as
static bills of materials evaluated against the construction rules; the index
in that directory gives the exact current count. Upstream ship and class names
remain excluded. The closed-source row is an independently constructed
replacement, not an adaptation of the prohibited source.

The `ship-1` through `ship-191` row-to-tag mapping is complete and remains
permanent. Supplemental core and independently constructed designs begin at
`ship-192`.

| Spreadsheet code | Rows | Source |
| --- | ---: | --- |
| `A&F` | 8 | *Anderson and Felix*, third edition |
| `ACC` | 6 | *Atlanta Class Carrier* |
| `Alcyone` | 4 | *Alcyone-class Interstellar Freighter* |
| `Atlas` | 7 | *Atlas-class Freighter* |
| `B&C` | 3 | *Bridgetown and Cape* |
| `BCPY` | 1 | *Brightwater-class Personal Yacht* |
| `Coner` | 3 | *Coner-class Trader* |
| `Contessa` | 3 | *Contessa-class Fast Trader* |
| `Copeline` | 5 | *Copeline-class Merchant Vessel* |
| `CS` | 6 | *Clement Sector Third Edition* (2021 combined rules) |
| `Freedom` | 3 | *Freedom-class Merchant Vessel* |
| `Grand Duke` | 5 | *Grand Duke of Kyiv-class Destroyer Series* |
| `Hercules` | 3 | *Hercules-class Heavy Freighter* |
| `HFGF` | 4 | *Hub Federation Ground Forces*, third edition |
| `HFN` | 1 | *Hub Federation Navy*, third edition |
| `HSOCS 1` | 1 | *Trent* quick ship/source file |
| `IAF` | 1 | *Independence Armed Freighter*; not open content |
| `Jinsokuna` | 4 | *Jinsokuna Chirashi Yacht* |
| `Knox` | 2 | *Knox-class Frigate* |
| `LCG` | 2 | *Lance-class Gunboat* |
| `Lion` | 5 | *Lion-class Battlecruiser* |
| `Loki` | 1 | *Loki-class Q-ship* |
| `Milligan` | 3 | Current *Milligan-class Hospital Ship* |
| `Opportunity` | 7 | *Opportunity-class Light Trader* |
| `PLF` | 4 | *Pleiades-class Light Freighter* |
| `Roosevelt` | 1 | *Roosevelt-class Intercept Destroyer* |
| `Rucker` | 8 | Current *Rucker-class Merchant* |
| `S&C` | 2 | *Skull and Crossbones*, third edition |
| `SOCS 1-3` | 8 | *Ships of Clement Sector 1–3* |
| `SOCS 4-6` | 28 | *Ships of Clement Sector 4–6* |
| `SoCS 7-9` | 11 | *Ships of Clement Sector 7–9* |
| `SOCS 10-12` | 6 | *Ships of Clement Sector 10–12* |
| `SOCS 13` | 1 | *Ships of Clement Sector 13* |
| `SOCS 14` | 3 | *Ships of Clement Sector 14* |
| `SOCS 15` | 1 | Older Milligan volume; use current standalone book |
| `SOCS 16` | 6 | Older Rucker volume; use current standalone book |
| `SOCS 17` | 5 | Older Atlas volume; use current standalone book |
| `Trade Empire` | 2 | *Trade Empire-class Commercial Transport* |
| `Type 3` | 2 | *Type 3 Security Cutter* |
| `Wendy Earth 1` | 1 | *Wendy's Guide to the Fleets of Earth Sector*, volume 1 |
| `Wendy Earth 2` | 1 | Earth fleet guide, volume 2 |
| `Wendy Earth 3` | 2 | Earth fleet guide, volume 3 |
| `Wendy Earth 4` | 5 | Earth fleet guide, volume 4 |
| `WGtC` | 1 | Wendy's Cascadia fleet guide |
| `WGtF` | 1 | Wendy's Franklin fleet guide |
| `WGttFoSS` | 1 | Wendy's Sequoyah fleet guide |
| `WGttFotC` | 2 | Wendy's Colonies fleet guide |
| `WH` | 1 | Wendy's Hub fleet guide |

## Plot and Adventure Material

Plot, storyline, dialogue, character, and location content is generally PI in
these works. Their potential value is limited to discrete mechanics, generic
encounter structures, rewards, timing, and statistical examples.

| Source group | Individual works | Potential use and Section 15 notes |
| --- | --- | --- |
| `21 Plots` series | `21_Plots_OGL.pdf`; `21_Plots_Too_OGL.pdf`; `21_Plots_III_OGL.pdf`; `21_Plots_5.pdf`; `21_Plots_Go_Forth_OGL.pdf`; `21_Plots_Misbehave_OGL.pdf`; `21_Plots_Planetside_OGL.pdf`; `21_Plots_Samaritan_OGL.pdf`; `21_Plots_CE.pdf` | Mission-shape and complication research only; plots and proper names are PI. Each has its own OGL/Section 15 lineage. |
| Clement adventures | `Cascadia Adventures 2nd.pdf`; `Grand Safari 2 083018Bk.pdf`; `Hells_Paradise_2nd.pdf`; `Long Road to Redemption Third Edition.pdf`; `Rogue World.pdf`; `The Slide.pdf` | Encounter pacing, mission rewards, NPC/ship competence, hazards, and operational procedures. Use only expressly open mechanics; retain each work's full inherited notice if activated. |

General Cepheus-compatible adventures elsewhere in `Adventures/` were
screened but are outside the selected setting families and currently offer
little to the trading/economy/naval simulator. These include *Danger on
Roritura*, *Dark Matter*, *Lineage*, *Shaerael*, and *Wreck in the Ring*.
Sword of Cepheus material is outside scope.

## Supporting or Non-Source Material

The following are useful aids but are not independent OGC sources:

- CE and Clement character sheets;
- vehicle, aircraft, walker, ship, massive-hull, spacecraft-design, and
  station-design worksheets;
- the Action Movie Physics sheet;
- referee screens, player cheat sheets, maps, images, and deck-plan images;
- `History_of_Clement_Sector.pdf`;
- `Intro_to_CS_OGL.pdf`, which is primarily an introduction/quick reference;
- `updated-naval-crew-guidelines.pdf`, which has no visible OGL declaration;
- `CareerBook.pdf`, a locally assembled reference without its own visible OGL
  declaration;
- `Planets.ods`, `Tasks.ods`, and `Ships.ods`, which are private research
  indexes without their own OGC declarations; and
- obsolete duplicate PDFs, except when needed to reproduce an inherited
  Section 15 notice exactly.

`Clement Sector Map-final4.pdf` and `Clement_Sector_Map-final4.pdf` are
byte-identical. `Earth-Sector-Map-V3.pdf` is likewise a map rather than a
mechanical source.

## Maintenance Rules

- Keep potential and active sources separate. Adding a title here does not
  add it to the distributed product's Section 15 notice.
- When a source is activated, record the exact local filename and edition,
  not merely a spreadsheet abbreviation.
- Store a renamed source identifier in future ship data; never use a PI ship
  or class name as the permanent identifier.
- Treat later editions as preferred mechanics but retain every older work
  named by their Section 15 declarations.
- Re-run the `Ships.ods` attribution audit whenever the spreadsheet changes.
- Record unresolved or non-open rows explicitly; do not silently infer a
  license from another book or from the presence of an OGL page.
