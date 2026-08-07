# Rule-Derived Ship Construction Model

## Authority Boundary

Shipbuilding rules are authoritative for mechanics. Ship specifications are
authoritative only as claims about a particular published design.

The construction path is:

1. curate a construction rule from its rule section into
   `catalog/shipbuilding/`;
2. hand-author a bill of materials in `catalog/ships/`;
3. evaluate every derived value from the selected rules; and
4. compare the result with published assertions.

A mismatch is reported as a source discrepancy. It is never reconciled with a
synthetic component, displacement delta, price adjustment, or value copied
from the specification.

## Units and Identity

All displacement is integer millitons and all money is integer credits. Rule
IDs name stable mechanical concepts. Design quantity never changes identity:
one hundred canisters select the same `sandcaster-canister` rule with quantity
100.

Drive codes are table variants. A mount is an assembly containing zero or
more weapon IDs up to its capacity. Fuel, cargo, unarmed fire-control
stations, ammunition, and other allocations remain typed design fields.

## Fixed and Parameterized Rules

Fixed or per-unit equipment provides a unit name, displacement per unit, and
price per unit. Parameterized equipment exposes only parameters named by its
rule:

- a custom hangar takes contained vehicle/craft volume;
- a launch tube takes the largest craft volume; and
- repair drones derive their volume from hull size.

The evaluator owns the corresponding formulas and rejects extra fields.
Designs cannot provide a rate, multiplier, component total, or override.

## Evaluation

`tools/ship_design.py` currently evaluates normal CE ships using:

- standard hulls and configurations;
- armor and hull options;
- normal Jump, maneuver, and fusion power-plant tables;
- drive performance and fuel;
- bridges, computers, software, and electronics;
- accommodations, facilities, hangars, and parameterized equipment;
- turret/fixed/pop-up mounts, weapons, bays, screens, and ammunition;
- hardpoint limits, cargo, construction time, and standard-design discount.

The evaluator requires exact volume accounting. It verifies TL minima, drive
compatibility, Jump Control capacity, computer rating, fuel-scoop constraints,
mount capacity, and hardpoint limits. Source assertions are redundant checks
and cannot participate in any calculation.

The core small-craft hull, drive, performance, cockpit/control-cabin, airlock,
crew, hardpoint, and energy-weapon rules are also executable. The core launch
reconstructs to exactly 20 tons and MCr4.797.

Expansion construction data is composed rather than copied over the core:

1. a source table is normalized into an extension module;
2. every overlap is matched to the corresponding core concept;
3. `catalog/shipbuilding/ct-ruleset.toml` selects the core value, adopts an
   extension, corrects a demonstrated source error, excludes the rule, or
   blocks it;
4. only active records may be admitted to the evaluator; and
5. a published specification is then reconstructed from those admitted
   rules.

This is checked by `tools/validate_shipbuilding_rules.py`. In particular, the
validator derives the minimum CE Jump-2 drive for every Z-to-J conversion,
rejects active Z-drive/ansible records, resolves all source IDs through the
master OGL registry, and checks module and supersession references.
