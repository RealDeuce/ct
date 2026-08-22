# Ship Catalog Conversion Status

The Clement/Earth conversion is complete. All 191 identities from the private
`Ships.ods` coverage inventory are present as active, PI-free, rule-derived
catalog entries. Tags `ship-1` through `ship-191` permanently preserve the
inventory row mapping. The 23 supplemental/core and original designs occupy
`ship-192` through `ship-214`.

The spreadsheet remains an external coverage and provenance aid. Active
entries are static bills of materials evaluated from `catalog/shipbuilding/`;
they are not generated from specification prose.

## Totals

| Category | Count |
| --- | ---: |
| Reserved Clement/Earth inventory rows | 191 |
| Admitted reserved rows | 191 |
| Reserved rows remaining | 0 |
| Admitted supplemental/core designs | 22 |
| Total active catalog entries | 214 |
| Design families | 114 |
| Multi-design shared lineages | 39 |
| Singleton design families | 74 |
| Native upgrade paths | 9 |
| Canonically named paths/manufacturers | 9 |
| Canonically named families | 114 |
| Canonically named fitted designs | 214 |

`catalog/ships/index.toml` revision 45 is the authoritative inventory.
`catalog/ships/families.toml` revision 1 is the authoritative family grouping.
`catalog/ships/upgrade-paths.toml` revision 1 is the authoritative native-path
assignment. `catalog/ships/names.toml` revision 1 is the authoritative naming
registry. The rationales are recorded in `docs/ship-family-grouping.md`,
`docs/ship-upgrade-paths.md`, and `docs/ship-catalog-naming.md`.

## Completed Source Coverage

| Source bundle | Reserved rows | Status |
| --- | ---: | --- |
| `ship-source-a-f` | 8 | Complete |
| `ship-source-acc` | 6 | Complete |
| `ship-source-alcyone` | 4 | Complete |
| `ship-source-atlas` | 7 | Complete |
| `ship-source-b-c` | 3 | Complete |
| `ship-source-bcpy` | 1 | Complete |
| `ship-source-coner` | 3 | Complete |
| `ship-source-contessa` | 3 | Complete |
| `ship-source-copeline` | 5 | Complete |
| `ship-source-cs` | 6 | Complete |
| `ship-source-freedom` | 3 | Complete |
| `ship-source-grand-duke` | 5 | Complete |
| `ship-source-hercules` | 3 | Complete |
| `ship-source-hfgf` | 4 | Complete |
| `ship-source-hfn` | 1 | Complete |
| `ship-source-hsocs-1` | 1 | Complete |
| `ship-source-jinsokuna` | 4 | Complete |
| `ship-source-knox` | 2 | Complete |
| `ship-source-lcg` | 2 | Complete |
| `ship-source-lion` | 5 | Complete |
| `ship-source-loki` | 1 | Complete |
| `ship-source-milligan` | 3 | Complete |
| `ship-source-opportunity` | 7 | Complete |
| `ship-source-plf` | 4 | Complete |
| `ship-source-roosevelt` | 1 | Complete |
| `ship-source-rucker` | 8 | Complete |
| `ship-source-s-c` | 2 | Complete |
| `ship-source-socs-1-3` | 8 | Complete |
| `ship-source-socs-4-6` | 28 | Complete |
| `ship-source-socs-7-9` | 11 | Complete |
| `ship-source-socs-10-12` | 6 | Complete |
| `ship-source-socs-13` | 1 | Complete |
| `ship-source-socs-14` | 3 | Complete |
| `ship-source-socs-15` | 1 | Complete |
| `ship-source-socs-16` | 6 | Complete |
| `ship-source-socs-17` | 5 | Complete |
| `ship-source-trade-empire` | 2 | Complete |
| `ship-source-type-3` | 2 | Complete |
| `ship-source-wendy-earth-1` | 1 | Complete |
| `ship-source-wendy-earth-2` | 1 | Complete |
| `ship-source-wendy-earth-3` | 2 | Complete |
| `ship-source-wendy-earth-4` | 5 | Complete |
| `ship-source-wgtc` | 1 | Complete |
| `ship-source-wgtf` | 1 | Complete |
| `ship-source-wgttfoss` | 1 | Complete |
| `ship-source-wgttfotc` | 2 | Complete |
| `ship-source-wh` | 1 | Complete |
| Independent OGC replacement | 1 | Complete |

The independent replacement occupies the row whose publication expressly
declared that no portion was Open Game Content. It was constructed solely from
admitted generic rules and does not adapt the prohibited design.

## Normalization Decisions

Every active entry has exact displacement accounting. The recurring source
conflicts were resolved consistently:

- Zimm/skip/reaction drives became ordinary CE Jump and maneuver drives;
- one-year fuel cells and instantaneous communication were removed;
- drive performance above the standard CE table was reduced to the legal
  rating for the loaded hull, including externally carried craft;
- fractional or above-TL armor became legal whole-point armor;
- excess hardpoints, small-craft weapon slots, quad mounts, retractable
  installations, and overlapping hardpoint labels became legal CE mounts;
- TL12-only fusion weapons and gatling point defense on TL11 designs became
  TL11 particle weapons and laser point-defense nodes;
- the heavy-railgun family and its ammunition were removed, with affected
  combat variants receiving explicit particle-weapon replacements;
- carried craft are separate catalog entries whose displacement, capacity,
  loaded-drive effects, and fitted cost are validated;
- rounded, over-allocated, or incomplete source totals were resolved by
  changing the actual fit and assigning only the exact remaining volume to
  cargo—never by a displacement or price adjustment.

Published names, class histories, setting prose, and other upstream Product
Identity are not present. Every active entry now has a canonical name drawn
from history, mythology, geography, scientific history, or public-domain
literature. All new path, manufacturer, family, and design names are expressly
Open Game Content. Every entry has list-valued source attribution and an
original Open Game Content designation. The compiled Section 15 declaration
is generated from `catalog/ogl-sources.toml`.

## Verification

Run:

```sh
python3 tools/validate_shipbuilding_rules.py
python3 tools/validate_ship_catalog.py
python3 tools/compile_catalog_ogl.py --check
python3 -m unittest discover -s tools -p 'test_*.py'
```

The catalog is complete only while all four checks pass and the reserved count
remains exactly 191.
