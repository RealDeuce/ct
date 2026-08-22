# Ship Catalog Naming

The canonical naming registry is
[`catalog/ships/names.toml`](../catalog/ships/names.toml). It assigns Open Game
Content names to all nine specialist paths, their presumed manufacturers, all
114 design families, and all 215 fitted designs. Permanent `ship-N`,
`family-N`, and `upgrade-path-N` identities remain the machine keys; names are
human-readable catalog data and may change only through an intentional catalog
revision.

The names are drawn from history, mythology, geography, scientific history,
and public-domain speculative or adventure literature. They do not retain
published class or variant names from the source inventory. Cepheus Trader
claims no Product Identity in the new path, manufacturer, family, or design
names: the registry expressly designates them as Open Game Content.

## Two-Level Convention

A path supplies a recognizable specialist vocabulary across otherwise
unrelated families. Its six-part `naming_sequence` corresponds, in order, to
the auxiliary, starter, light, medium, heavy, and capital progression stages.
The sequence is a semantic progression rather than a mandatory grammatical
form: it tells a future author what kinds of names belong at each point.

A family supplies the stronger relationship among variants of one platform.
When members of a family occupy different paths, their names remain connected
even when that means a particular design uses the family's literary,
historical, or conceptual vocabulary instead of the most literal word from
the path sequence. A repeated source record of the same fit may use the same
design name; a materially different variant gets a related name or a clear
functional suffix.

## The Nine Paths

| Path | Canonical path | Presumed yard | Naming progression |
| --- | --- | --- | --- |
| 1 | Concord Exchange | Concord Exchange Yards | exchange tools; merchants and travelers; trade cities; trade leagues; trade routes; civil prosperity |
| 2 | Venture Passage | Venture Passage Works | messengers; adventurous travelers; protected expeditions; hazardous passages; frontier entrepôts; enterprise and survival |
| 3 | Outer Reach | Outer Reach Cooperative | prospecting tools; frontier figures; remote settlements; wilderness passages; lost cities; self-reliance |
| 4 | Civic Survey | Civic Survey Works | scientific instruments; scientists and investigators; research expeditions; service institutions; great surveys; knowledge and public service |
| 5 | Marque Marine | Marque Marine Yards | boarding and escort tools; privateers; letters and commissions; mercenary captains; prize courts; licensed war |
| 6 | Rogue Tide | Rogue Tide Yards | boarding weapons; tricksters and smugglers; pirates and raiders; raider havens; outlaw passages; dangerous freedom |
| 7 | Admiralty Line | Admiralty Line Works | military messengers and pilots; patrol captains; admirals; naval battles; naval theorists; decisive fleet actions |
| 8 | Redoubt | Redoubt Shipbuilding | defensive weapons; guards and wardens; forts; fortresses; defensive systems; last stands |
| 9 | Tempest | Tempest Arsenal | projectiles and predators; ambush hunters; storms and volcanoes; catastrophes; cosmic explosions; apocalyptic concepts |

The sequence is visible in such names as `Mercator`, `Lübeck`, `Visby`, and
`Silk Road` on the orderly-trade path; `Raleigh`, `Hawkwood`, and `Cochrane`
on the privateer path; `Corbett`, `Togo`, `Yi`, `Trafalgar`, and `Jutland` on
the regular-navy path; and `Congreve`, `Stahlstadt`, `Cyclops`, `Onager`, and
`Vesuvius` on the missile-and-strike path.

## Family Examples

- The **Daedalus** work-pod family uses `Hermes`, `Labyrinth`, `Knossos`, and
  `Minotaur`, tying courier, mining, transfer, and maintenance fits to one
  mythic complex.
- The **Verne** 300-ton lineage uses `Fogg`, `Hatteras`, `Aouda`,
  `Passepartout`, `Nemo`, `Stahlstadt`, and `Robur`. Those public-domain
  literary references let its passenger, frontier, bulk, patrol, missile, and
  fast-armed variants cross several paths without losing family resemblance.
- The **Stevenson** 400-ton lineage uses `Silver`, `Trelawney`, and `Smollett`
  for raider, armed-passenger, and escort fits.
- The **Homeric** 800-ton lineage uses `Ithaca`, `Scheria`, `Odysseus`,
  `Cyclops`, `Calypso`, and `Nausithous` across freight, colony, privateer,
  missile, extended-range, and passenger roles.
- The **Proteus**, **Caduceus**, and **Wayfarer** small-craft families use
  stable family names plus functional suffixes where immediate recognition is
  more useful than another proper noun.
- The **Caravanserai** fast-trader family uses the Silk Road cities
  `Samarkand`, `Bukhara`, and `Merv` for trader, replenishment, and assault
  variants.
- The **Roman** destroyer family uses `Scipio`, `Onager`, `Corvus`, and
  `Vesuvius`, connecting its battle, torpedo, direct-fire, and missile fits
  while allowing them to occupy different combat paths.
- The **Franklin** light-trader family uses `Franklin I`, `Poor Richard`,
  `Franklin II`, `Pennsylvania`, `Deborah`, `Postmaster`, and `Gulf Stream`
  for its two generations and specialist commercial fits.

## Local Presentation

A sysop may provide local aliases, advertising, reputation, history, and
other setting prose. That overlay does not replace the canonical registry and
must not change family membership, native path, mechanics, availability, or
balance. Interfaces should retain a way to reveal the canonical name and
stable tag so ships remain unambiguous across BBS polities.

## Validation

`tools/validate_ship_catalog.py` requires:

- exactly one canonical name for every known path, family, and design;
- one six-stage naming sequence for every path;
- no missing, duplicate-identity, unknown, or placeholder design-name record;
  and
- exact agreement between each design's catalog display name and the
  canonical registry.

Adding a ship, family, or path therefore requires updating the naming registry
in the same catalog change.
