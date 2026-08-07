# Shipbuilding Rules

This directory contains construction rules, not ship specifications.
`ce-core.toml` and `ce-small-craft.toml` are normalized Open Game Content
transcriptions of the normal Jump-drive and small-craft construction
sequences in Chapter 8 of the Cepheus Engine SRD. Published ships in Chapter
9 and other books are not allowed to create or modify records here.

The composed game rules are declared by `ct-ruleset.toml`. It names the active
modules and records every known collision with the core rules:

- `af3-components.toml` contains compatible hull, structural, bridge, sensor,
  accommodation, facility, external, hangar, and cargo additions;
- `af3-armament.toml` contains compatible mounts, weapons, ordnance, bays,
  spinal weapons, screens, and ship software;
- `earth-tech-2350.toml` contains later component refinements and explicitly
  excludes its instantaneous communicator; and
- `af3-small-craft-source.toml` and `af3-technology-source.toml` preserve
  incompatible small-craft and generic technology-adjustment tables for
  source audits only. They are not part of the active game rules.

No active expansion record defines a Z-drive. Source Z-drive letters exist
only in `ct-ruleset.toml`'s conversion table. Game designs select ordinary CE
Jump drives and use CE Jump fuel, time, plotting, and failure behavior.

A rule record represents one actual construction concept. A fuel processor is
one quantity-bearing rule; a pile of 29 processors is not a new component.
Drive letters are variants from the drive table. Mounts contain weapons.
Ammunition quantity belongs to the design.

Rules use integer millitons and credits. There is no adjustment, override, or
unparsed-description field. If a design names equipment whose construction
rule has not yet been curated, evaluation stops with an unknown-rule error.

Evaluate a hand-authored design with:

```sh
python3 tools/ship_design.py --pretty \
  catalog/ships/ship-192.toml
```

Designs selecting `ruleset_id = "cepheus-trader.shipbuilding"` automatically
compose the CE large-ship baseline with the three active modules in the order
declared by `ct-ruleset.toml`. Explicit `supersedes_id` relationships replace
the older record; blocked, excluded, and source-audit-only records never enter
the composed ruleset.

Validate the rule composition and its attributions with:

```sh
python3 tools/validate_shipbuilding_rules.py
python3 tools/compile_catalog_ogl.py --check
```

The evaluator executes the core large-ship and small-craft construction
sequences plus the expansion formulas currently required by admitted ships.
Those include whole-point armor, hull-percentage and protected-system
structure, percentage bridge and emergency-power options, redundant
computers, fixed and parameterized equipment, hangars, launch facilities,
docking clamps, turrets, barbettes, bays, screens, point-defense nodes,
ammunition packs, improved magazines, and carried-craft cost and capacity
checks.

More specialized records remain unavailable until their evaluator behavior
is implemented. These currently include electronics and equipment upgrades,
secondary bridges, retractable barbettes, spinal weapons, screens with
upgrade options, and specialized cargo installations. Records marked `blocked-*` or
`source-audit-only`, and records under `excluded_component`, must never be
accepted by the game ruleset.
