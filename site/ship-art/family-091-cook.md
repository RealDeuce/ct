# Cook Family Visual Manifest

*Status: catalog production family 091*

## Mechanical identity

- Members: `ship-91` Cook patrol frigate and `ship-95` Cook patrol corvette
- Path: Admiralty Line Works (P7)
- Hull: 600 displacement tons / approximately 8,400 m³
- Dimensions: 68.0 m long, 23.0 m wide, and 15.0 m high
- Shared: Jump-2, maneuver and power drives S, four armor points, advanced
  electronics, four triple turrets, two particle-beam barbettes, six point-
  defense nodes, and one 30-ton full hangar
- Record 91: TL11, standard configuration, Jump drive H, two-week endurance,
  hardened/holographic bridge, 135 tons cargo, and one internal Jason
- Record 95: TL12, streamlined configuration, Jump drive F, four-week
  endurance, radiation shielding, reinforced structure, two fib/bis computers,
  improved magazine, 28 tons cargo, and one internal Wayfarer Utility

The family relationship is the internal patrol chassis and its fixed station
coordinates. The later streamlined hull is a close fairing over that chassis,
not a second unrelated ship.

## Canonical architecture

The chassis occupies a 68.0 × 23.0 × 15.0 m envelope. Its TL11 baseline is a
blunt axial standard hull: a six-pane armored command block, rectangular
central pressure citadel, broad shoulders, protected boarding waist, and square
aft engineering block. Longitudinal datum ribs tie these volumes together.

The TL12 refit rounds and chamfers the pressure-body steps, adds a smoothly
faceted prow around the same recessed bridge brow, fairs the scoop, and covers
the shoulders with a continuous skin. Bridge, doors, weapons, radiator datum,
aft block, and maximum dimensions remain at the same chassis stations.

Exactly four triple-turret centers are common: two dorsal and one on each upper
shoulder. Two carry beam-laser triples, one a missile triple, and one a
sandcaster triple. Two independent long low particle-beam barbette channels run
along the dorsal flanks. They are not turret rails and each represents one
five-ton fixed barbette. Six small point-defense centers cover all arcs; four
are readable in the catalog view and two are occluded.

Two broad port-side openings remain fixed. The forward shutter serves the
cargo hold. The central/aft two-leaf shutter is one full-hangar door, at least
8.3 × 5.0 m clear with a 22 m internal craft axis. It accepts either the
21.0 m Jason or the 18.8 m Wayfarer Utility. Both craft are internal and absent
from exterior recognition plates.

One segmented aft Jump-service radiator and two coil-service housings mark the
internal Jump installation; no exterior ring or glow is used. Record 95's
smaller F drive makes these fittings more flush without moving them. Four
maneuver apertures, two power grilles, a ventral scoop grid, and recessed
landing doors occupy common boundaries.

### Invariants

Preserve the dimensions, internal chassis proportions, six-pane bridge,
shoulder/waist/aft stations, four turret centers, two barbette channels, six
point-defense centers, cargo and hangar door boundaries, docking collar,
Jump-service locations, stern boundaries, landing geometry, main datum ribs,
camera, crop, backdrop, and lighting. Streamlining may cover steps but may not
move a station, alter the maximum envelope, or add a wing, tail, tank, ring,
hangar, weapon, or externally carried craft.

## Production fits

### Cook patrol frigate — record 91

- Standard hull with exposed chamfered chassis steps and square aft block
- Two beam-laser triples, one missile triple, one sandcaster triple
- Two fixed particle-beam barbettes and six laser point-defense nodes
- Active Jump-service fittings; four-point armor
- Closed hangar containing Jason; separate 135-ton cargo shutter
- Admiralty finish: deep navy `#203B69`, warm white `#E8E5D4`, signal red
  `#C43C35`, and bright aluminum

### Cook patrol corvette — record 95

- Smooth fairing over the exact baseline chassis and station grid
- Same four triple turrets and two particle-beam barbettes
- Three point-defense gatling-laser heads and three ordinary laser heads
- Radiation-shield seam band, doubled structural datum ribs, deeper bridge
  recess, and more flush F-drive service housings
- Closed hangar containing Wayfarer Utility; separate 28-ton cargo shutter
- Same formal Admiralty palette with a more precise TL12 finish

## Catalog plate and reusable prompts

Use a complete front-port three-quarter view from about 12 degrees above,
restrained near-orthographic perspective, 3:2 landscape, warm upper-front-port
key, cool aft rim, charcoal stars, and faint teal grid. Render as original
late-1970s/early-1980s gouache-and-airbrush technical art with saturated fleet
colors and bright metal. No text, logo, watermark, weapon fire, planet, modern
gray rendering, or recognizable franchise styling.

```text
Neutral chassis: create a 68 x 23 x 15 m standard-hull 600-ton patrol ship with
six-pane command block, rectangular citadel, broad shoulders, boarding waist,
square aft block, and fairing datum ribs. Reserve four triple-turret rings, two
separate long particle-barbette channels, six point-defense covers, one cargo
shutter, one 8.3 x 5.0 m full-hangar shutter with 22 m internal axis, docking
gear, Jump radiator, two coil housings, fixed stern apertures, scoop, and
landing doors.

Shared edit rule: preserve the maximum envelope, internal chassis, bridge and
all component centers, doors, aft block, camera, and lighting. Change only the
outer standard/streamlined surface treatment, explicitly listed TL fit, and
finish. Do not move equipment beneath the fairing.

Record 91: retain the blunt standard hull. Activate two beam triples, one
missile triple, one sandcaster triple, two particle barbettes, and six laser
point-defense nodes. Keep active Jump fittings, close both doors, and apply the
Admiralty palette.

Record 95: start from record 91 and add a close smoothly faceted fairing around
the same bridge, steps, shoulders, and scoop. Preserve every weapon and door.
Show three gatling and three ordinary point-defense nodes, a shield seam band,
reinforced datum ribs, and flush F-drive service fittings. Retain the Admiralty
palette and closed internal-craft hangar.
```

## Production sequence and provenance

1. Approve the neutral standard chassis and fixed station grid.
2. Produce record 91 with all weapons and doors countable.
3. Derive record 95 directly from that plate by fairing the inherited chassis;
   do not regenerate it independently.

Generated for Cepheus Trader on 2026-08-18 with OpenAI image generation. The
streamlined master is a constrained edit of the approved standard-hull master.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-091-cook.png` | Neutral standard chassis | Approved |
| `site/ship-art/masters/ship-091-cook.png` | Standard patrol-frigate master | Approved |
| `site/ship-art/masters/ship-095-cook.png` | Streamlined patrol-corvette master | Approved |
| `site/assets/ships/ship-091-cook.webp` | Published record 91 plate | Approved |
| `site/assets/ships/ship-095-cook.webp` | Published record 95 plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
