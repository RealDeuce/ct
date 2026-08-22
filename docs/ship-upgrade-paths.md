# Ship Upgrade Paths

*Status: current native-path assignment, 2026-07-27*

Every active ship design has one native upgrade path in
[`catalog/ships/upgrade-paths.toml`](../catalog/ships/upgrade-paths.toml).
Each path represents the product doctrine of one specialist manufacturer or
shipyard aligned with a cell of the trade/mixed/combat ×
orderly/contested/chaotic polity matrix. Canonical path and manufacturer names
are assigned in
[`catalog/ships/names.toml`](../catalog/ships/names.toml).

This is a design-origin relationship. It does not restrict who may purchase,
capture, command, license, manufacture, or refit a design. Local availability
is separate universe state.

## Relationship to Families

Family and path membership are independent:

- a family records shared platform lineage;
- a design's native path records the specialist yard and doctrine associated
  with that particular fit; and
- variants from one family may be native to different paths.

Twenty-one of the current 114 families span more than one path. Repeated
copies of the same fit remain in the same native path; cross-path membership
is reserved for actual variants with materially different purposes.

## Product Stages

Every design also carries one coarse `progression_stage`:

| Stage | Mechanical definition |
| --- | --- |
| `auxiliary` | Any catalog small craft |
| `starter` | Ship or starship through 400 tons |
| `light` | 401–999 tons |
| `medium` | 1,000–1,999 tons |
| `heavy` | 2,000–4,999 tons |
| `capital` | 5,000 tons or more |

These stages expose catalog coverage and gaps. They do not assert that every
larger vessel is a better purchase, nor do they replace price, operating cost,
crew, jump performance, authority, or mission suitability. The eventual
player-facing progression can branch within a stage.

## Native Catalog Coverage

| Path | Canonical path / yard | Matrix specialty | Designs | Auxiliary | Starter | Light | Medium | Heavy | Capital |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `upgrade-path-1` | Concord Exchange / Concord Exchange Yards | Orderly trade: scheduled commerce, passengers, bulk freight, and civil logistics | 60 | 21 | 29 | 6 | 2 | 2 | 0 |
| `upgrade-path-2` | Venture Passage / Venture Passage Works | Contested trade: fast, protected, and armed commercial operation | 20 | 6 | 12 | 1 | 1 | 0 | 0 |
| `upgrade-path-3` | Outer Reach / Outer Reach Cooperative | Chaotic trade: austere frontier commerce, mining, and self-support | 14 | 3 | 9 | 2 | 0 | 0 | 0 |
| `upgrade-path-4` | Civic Survey / Civic Survey Works | Orderly mixed: survey, customs, research, medicine, and regulated civil service | 19 | 4 | 11 | 1 | 2 | 1 | 0 |
| `upgrade-path-5` | Marque Marine / Marque Marine Yards | Contested mixed: security, escort, privateering, and armed merchants | 17 | 6 | 6 | 4 | 1 | 0 | 0 |
| `upgrade-path-6` | Rogue Tide / Rogue Tide Yards | Chaotic mixed: covert transport, boarding, and commerce raiding | 10 | 2 | 4 | 4 | 0 | 0 | 0 |
| `upgrade-path-7` | Admiralty Line / Admiralty Line Works | Orderly combat: regular fleets, carriers, patrol commands, and formal logistics | 43 | 11 | 3 | 6 | 13 | 8 | 2 |
| `upgrade-path-8` | Redoubt / Redoubt Shipbuilding | Contested combat: local defense, attack craft, assault ships, and divided authorities | 20 | 6 | 7 | 2 | 4 | 1 | 0 |
| `upgrade-path-9` | Tempest / Tempest Arsenal | Chaotic combat: ambush, missile saturation, torpedo attack, and strike operations | 10 | 3 | 2 | 2 | 2 | 1 | 0 |

The asymmetry is intentional. Only the orderly-combat catalog currently has
native designs in every size stage. Other yards have real gaps; they are not
filled with duplicate or invented hulls merely to make the table regular.

## Backfill

A player may move into a neighboring specialty's design when the current path
has no suitable native step. Exact adjacency and preferred backfill candidates
will be explicit catalog relationships rather than inferred from displacement
alone. They are not assigned in this pass: acquisition price, operating
economics, starter-package balance, and the established manufacturer doctrines
must be considered before claiming that one design is the intended successor
to another.

The absence of a native medium, heavy, or capital design is therefore useful
game information, not an error in the catalog.
