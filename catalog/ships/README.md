# Active Ship Catalog

This directory contains the ship designs that Cepheus Trader may load. Every
`ship-N.toml` file is a hand-authored bill of materials evaluated from the
construction rules in `../shipbuilding/`; it does not repeat component prices
or displacement values.

Each entry contains:

- a permanent numeric catalog ID and `ship-N` tag;
- a numeric family ID resolving through `families.toml`;
- a native upgrade-path ID and mechanically derived progression stage
  resolving through `upgrade-paths.toml`;
- status, vessel kind, role tags, and two or three original descriptive
  paragraphs;
- a complete list of OGL source IDs and an Open Game Content designation;
- the selected construction rules and quantities; and
- source specification values only as `[assertions]`.

Starter designs also repeat the rules-derived `thrust_g` as a validated
top-level presentation field. The construction evaluator rejects a value that
does not match the selected hull and maneuver drive; the server reads this
field rather than maintaining a second hard-coded starter summary.

`index.toml` is the authoritative admitted inventory. A parent design may use
`[[carried_craft]]` records to include another catalog design. The parent
allocates hangar or docking-clamp capacity through the construction rules;
the referenced craft contributes its already-evaluated fitted price without
receiving the parent's standard-design discount a second time. Validation
orders entries by carried-craft dependency, so permanent catalog ID order
does not constrain which craft a ship may carry.

`families.toml` is the authoritative PI-free family grouping. It contains 113
anonymous families: 39 shared lineages and 74 singleton designs. Every ship
record repeats its family ID, and validation rejects missing, duplicate,
unknown, or inconsistent membership. The reviewed grouping rationale is in
`../../docs/ship-family-grouping.md`.

`upgrade-paths.toml` assigns every design to one of nine anonymous specialist
manufacturer/shipyard doctrines. Paths correspond to the
trade/mixed/combat × orderly/contested/chaotic matrix and are intentionally
sparse. `../../docs/ship-upgrade-paths.md` records their coverage and stage
gaps. Exact adjacent-path backfill choices are a later explicit relationship,
not something inferred from tonnage.

`names.toml` assigns the canonical Open Game Content names for all nine paths
and manufacturers, 113 families, and 213 fitted designs. It also gives each
path a six-stage naming sequence. Every ship repeats its canonical display
name, and validation requires exact agreement. The convention and principal
family examples are documented in `../../docs/ship-catalog-naming.md`.

Tags `ship-1` through `ship-191` contain the complete reconstructed
source-coverage inventory. Rule-derived supplemental and core designs begin
at `ship-192`.

The active set contains all 191 reserved Clement/Earth rows plus 22
supplemental/core designs. It covers merchant, passenger, courier,
utility-craft, research, survey, patrol, raider, system-defense, carrier, and
dreadnought roles. Additional source designs enter this directory only after
their construction-rule bills of materials are reconstructed and balanced.
Completion and normalization decisions are recorded in
`../../docs/ship-catalog-conversion-status.md`.

An assertion may detect a discrepancy in a published design, but it may not
alter the rules-derived result. Invalid published designs are either corrected
as explicitly documented Cepheus Trader variants or omitted.

Validate the active catalog with:

```sh
python3 tools/validate_ship_catalog.py
python3 tools/compile_catalog_ogl.py --check
```
