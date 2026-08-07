# Ship Construction Audit

The construction audit starts from Chapter 8 rules and reconstructs each
design as a hand-authored bill of materials. Chapter 9 and other ship
specifications supply comparison values only.

| Design | Volume | Rules-derived price | Published price | Result |
| --- | ---: | ---: | ---: | --- |
| CE Merchant Trader source reconstruction | 200 tons | MCr34.929 | MCr34.929 | Exact match after including required Jump Control/1 software, which the prose specification omits. The active Hudson starter revision is instead J-2, carries 53 tons, and costs MCr51.219. |
| CE Frontier Trader source reconstruction | 300 tons | MCr82.319 | MCr82.314 | MCr0.005 conflict: the published total is obtained only by applying the 10% standard-design discount to Cr50,000 of sandcaster ammunition. The active Crusoe starter revision is instead J-2, carries 92 tons, has one steward and 12 staterooms, and costs MCr85.559. |
| CE Merchant Freighter | 400 tons | MCr59.814 | MCr59.814 | Exact match. |
| CE Merchant Liner | 300 tons | MCr70.209 | MCr70.209 | Exact match. |
| CE Courier | 100 tons | MCr35.469 | MCr35.928 | MCr0.459 conflict shared with the Yacht; no construction-table item accounts for it. |
| CE Yacht | 100 tons | MCr25.929 | MCr26.388 | MCr0.459 conflict shared with the Courier; no construction-table item accounts for it. |
| CE Launch | 20 tons | MCr4.797 | MCr4.797 | Exact match. |
| CE Pinnace | 40 tons | MCr18.567 | MCr18.567 | Exact match. |
| CE Ship's Boat | 30 tons | MCr16.677 | MCr16.677 | Exact match when the MCr0.1 control cost per 20 hull tons is rounded up to a whole increment. |
| CE Shuttle | 90 tons | MCr25.587 | MCr25.587 | Exact price match; its published Structure 1 conflicts with the Chapter 8 formula, which gives Structure 2. |
| CE Fighter | 10 tons | MCr11.660 | MCr10.841 | MCr0.819 conflict. The build includes the specification's explicit one-ton fire-control allocation in addition to its zero-ton fixed mount. |
| CE Corvette (corrected) | 300 tons | MCr189.075 | MCr194.445 | The published bill is eight tons over hull at its stated 25-ton cargo. The catalog reduces cargo to 17 tons and prices the corrected rule-derived build. |
| CE Patrol Frigate (corrected) | 300 tons | MCr184.224 | MCr180.675 | The published bill is one ton over hull; cargo is reduced from 23 to 22 tons. The catalog price includes two evaluated fighters. |
| CE System Defense Boat (corrected) | 400 tons | MCr169.866 | MCr171.574 | The published bill is two tons over hull and omits the rules-required navigator; cargo is reduced to 107 tons and the crew is increased to 19. |
| CE Raider concept (corrected) | 600 tons | MCr322.821 | MCr310.851 | Rebuilt as Jump-2 with six hardpoints for its six turrets, a valid three-craft hangar, armor 8, and 55 tons of cargo. |
| CE Research Vessel | 200 tons | MCr57.393 | MCr73.809 | The physical bill balances and includes two evaluated launches; the unexplained published price is retained only as a discrepancy. |
| CE Survey Vessel | 300 tons | MCr104.283 | MCr120.969 | The physical bill balances and includes two evaluated launches; the unexplained published price is retained only as a discrepancy. |
| CE modular Cutter chassis | 50 tons | MCr24.3045 | MCr24.305 | Exact to the source's nearest-Cr1,000 presentation; the 30-ton module berth occupies 33 tons under its 110% support rule. Modules are separate loadouts. |
| CE Dreadnought concept (corrected) | 5,000 tons | MCr2,682.504 | MCr2,768.145 | Core bonded superdense armor is bought in six-point layers, so printed armor 14 is impossible. The catalog uses armor 12, balances remaining volume as cargo, and includes 20 fighters and two cutters. |

The source Frontier Trader's physical design is valid and balances exactly.
Cepheus Trader retains the Chapter 8 ammunition rule, so that source
reconstruction has a value of MCr82.319. The active Crusoe is a deliberate
starter revision whose MCr85.559 value is separately derived from its actual
bill of materials. The other mismatches are treated the same way: the catalog
records the construction result and never inserts an unexplained price or
displacement adjustment merely to reproduce a published total.

Run both reconstructions with:

```sh
python3 tools/validate_ship_catalog.py
```

## Executable composition

`tools/shipbuilding_rules.py` now composes the CE large-ship baseline and the
active expansion modules in the exact order declared by `ct-ruleset.toml`.
Explicit supersession removes the older component, while blocked, excluded,
and source-audit-only records do not enter the executable rules.

The evaluator can currently select fixed-volume expansion equipment and all
encoded ammunition volume/price forms: per tonne, per five tonnes, per twenty
tonnes, and whole reload packs. Reload packs must be purchased in complete
packs; their prices are not divided into invented per-projectile values. The
small-craft evaluator also applies whole MCr0.1 control-cost increments and
can represent a separately allocated fire-control station for a zero-ton
fixed mount.

The remaining evaluator work is formula-specific rather than transcription:
structural and bridge options, whole-point armor, electronics and equipment
upgrades, docking/launch/hangar rules, barbettes, spinal weapons,
point-defense mounts, and specialized cargo spaces. Their normalized
source records remain present, but designs cannot select them until their
constraints and derived effects are implemented.

## Construction-source coverage

The normalized rule layer now contains:

- the complete CE large-ship and small-craft construction baselines;
- the third-edition expansion's compatible structural, bridge, sensor,
  accommodation, internal/external component, hangar, cargo, armament,
  ordnance, bay, spinal, screen, and software tables;
- the later Earth technology update's ship computers, sensors, armament,
  screens, drones, and software refinements; and
- a source-audit-only record of the expansion small-craft equipment and
  armament model where it conflicts with CE's one-hardpoint model.

Overlapping CE rows are referenced rather than duplicated. The expansion
M-drive, P-plant, ordinary computer, sensor, stateroom, basic turret, and
shared weapon rows therefore do not create second components.

## Resolved source defects

The composition policy in `catalog/shipbuilding/ct-ruleset.toml` records the
machine-readable resolution and evidence for each defect. Important examples
include:

- Z-drive K's printed 65 tons is corrected to the CE K drive's 55 tons;
- M-drive S's printed MCr58 is corrected to the CE sequence's MCr68;
- radiation shielding uses the MCr0.1-per-hull-ton prose and worked examples,
  not the MCr0.25 summary cell;
- self-sealing uses the CE/prose MCr0.01-per-hull-ton value, not the MCr0.1
  summary cell;
- the small-craft hull table's missing decimals (`13` and `1`) are retained as
  source corrections, while active construction uses CE's finer table;
- the cargo access-lock sequence corrects the visibly printed `26` to `16`
  tons between the 12-, 20-, and 24-ton rows.

The Mineralogy Suite appears in the *Anderson and Felix Third Edition*
construction table as a five-ton sensor addition, but the price cell is blank.
It also appears as a five-ton fitted component in *Wendy's Earth 4*, again
without a price. *Bounded Fortune* only mentions a “Starship Mineralogy
Package” as cargo flavor. The component therefore remains blocked rather than
being assigned a zero or guessed price.

## Deliberate exclusions and pending choices

All Z-drive routes, points, skip transit, recharge, emitter, military-grade,
and bubble-failure mechanics are excluded. The quantum-entanglement
communicator is excluded because Cepheus Trader has no ansible. Normal CE
Jump, maneuver, and fusion power rules win over capital percentage drives,
reaction drives, solar sails, and alternative power plants.

The expansion hull rows above 5,000 tons describe in-system capital
structures. No Jump-capable ship or Jump-drive performance case above 5,000
tons appears in the selected sources, so there is no such construction scope
to block. One-year fusion power-plant fuel cells are explicitly excluded;
Cepheus Trader uses standard CE power-plant fuel and refueling.
