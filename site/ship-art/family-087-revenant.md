# Revenant Visual Manifest

*Status: catalog production singleton family 087*

## Mechanical identity

- Record: `ship-87` Revenant
- Path: Rogue Tide (P6)
- Hull: 600 displacement tons / approximately 8,400 m³; streamlined
- Dimensions: 70.0 m long, 27.0 m wide, and 16.5 m high
- Fit: TL11, four armor points, Jump H / Jump-2, maneuver and power M /
  thrust 4, two weeks endurance, basic civilian electronics, and Model 3/bis
- Complement: two staterooms, two crew berthings, fifty-place barracks,
  fifteen-place recreation, sixty-two-place galley, briefing room, office,
  armory, four-cell brig, two emergency low berths, twelve low berths, and a
  two-bed medical bay
- Capture and route support: breaching tube, 41.5 tons prize cargo, six fuel
  processors, repair drones, and one Cutlass in a 50-ton full hangar
- Weapons: four double-beam turrets, one double-missile turret, and one double-
  sandcaster turret; all six hardpoints used and no point defense

Revenant is an independent heavy-raider family. Its architecture must express
the Rogue Tide practice of disguising assault machinery inside a plausible
working hull: deceptive cargo masses, asymmetric service shoulders, false
seams, concealed weapon recesses, and enough stern area for sudden high thrust.

## Canonical architecture

The hull is a 70.0 × 27.0 × 16.5 m “boarding hook”: a deep clipped spear prow,
filled central pressure body, asymmetric port assault shoulder, balancing
starboard barracks shoulder, and broad four-gravity stern. It is a wingless
streamlined spacecraft, not an aircraft, seagoing vessel, open-frame tanker,
or detachable module train.

A low faceted command brow spans the prow. The port assault shoulder contains
one closed 10 × 6.2 m Cutlass hangar shutter with an approximately 24 m internal
axis, sufficient for the approved 22 × 8.5 × 5.2 m armed cutter. The Cutlass
remains internal. A separate stowed lateral breaching-tube collar sits forward
and below the hangar; its annular capture seal, mechanical locks, and protected
telescoping trunk must never be interpreted or rendered as a gun.

One smaller closed shutter serves the 41.5-ton prize hold. Barracks service
panels, two stateroom positions, low-berth panels, brig access, armory, medical
hatch, drone slot, and airlocks remain closed and visually subordinate. False
cargo seams help the assault fit pass at distance without inventing detachable
containers.

## Six-coordinate coverage map

Six hardpoint coordinates divide evenly between the visible and occluded hull
halves. The three visible mounts are deliberately distributed across the upper
and lower port arcs rather than collected in a dorsal row:

1. Forward upper-port flank: double-beam cup with exactly two cyan optics.
2. Higher, farther-aft port shoulder: double-sandcaster cup with two blunt
   square projectors, each showing four dark ports in a 2 × 2 array.
3. Lower-port/keel flank: double-missile mount with two closed rectangular
   launch shutters.

The hidden starboard and ventral half contains the other three double-beam
turrets. This exposes about half of the ship and exactly half of its six
hardpoints while providing overlapping fore, flank, upper, lower, and aft
fields instead of a crown-only battery.

There are no point-defense nodes. The breaching collar, processors, hangar and
cargo latches, bridge glazing, sensors, radiator panels, Jump service covers,
and maneuver apertures remain unarmed. Missiles and sand canisters remain
internal.

## Fuel, Jump, drive, and armor

Six processors use three circular ribbed port grilles and three hidden
starboard grilles, with a separate coupling and manifold. Revenant has no fuel
scoop and therefore receives no intake-like ventral feature.

The internal Jump-H drive reads through a segmented transverse radiator/service
band at the aft waist and paired flush coil-service housings. There is no
exterior Jump ring, hoop, portal, luminous gateway, or exposed tank. Four
visible and four hidden maneuver apertures flank distinct power-plant grilles
at the thrust-four stern.

Four armor points read through overlapping serviceable plates, recessed seams,
reinforced command and hangar frames, armored edge bands, and protected weapon
cups. The result is a hard-used privateer hull rather than a pristine naval
citadel.

## Rogue Tide finish and catalog plate

- Aubergine `#4D294F`: primary pressure hull and prow planes
- Vermilion `#D94B35`: hangar frame, breaching machinery, weapon collars, and
  selected service seams
- Mustard `#D4A62A`: warning plates, access status, and small recognition fields
- Blackened chrome: edge bands, machinery guards, weapon faces, radiator and
  drive interfaces
- Charcoal and cyan: recesses, glazing, grilles, and beam optics

Allow visibly mismatched replacement panels and hand-maintained edges, but do
not turn the ship into a uniformly black stealth craft. Use a complete front-
port three-quarter view about 12 degrees above, restrained near-orthographic
perspective, 3:2 landscape, warm upper-front-port key, cool aft rim, charcoal
stars, and faint teal drafting grid. Render as original late-1970s/early-1980s
gouache-and-airbrush technical art. No text, logo, watermark, open door, weapon
fire, planet, modern CGI, or franchise styling.

## Invariants and reusable prompts

```text
Neutral anchor: create one independent 70 x 27 x 16.5 m streamlined 600-ton
boarding-hook hull with deep clipped spear prow, low faceted command brow,
filled pressure body, asymmetric port cutter-and-boarding shoulder, balancing
starboard barracks shoulder, and broad thrust-four stern. Add one closed 10 x
6.2 m Cutlass hangar with 24 m internal axis, separate closed prize-cargo
shutter, and one stowed lateral breaching-tube collar that is plainly nonweapon
machinery. Reserve six hardpoints, three visible and three hidden. Visible:
forward upper-port flank, higher farther-aft port shoulder, and lower-port/keel;
leave all blank. Add no PD. Show 3-visible/3-hidden processor grilles, no scoop,
an aft Jump-H service band and coil housings, 4-visible/4-hidden maneuver
apertures, and four-point armor. Neutral aluminum and pale primer, period
gouache and airbrush, front-port catalog plate.

Revenant fit: preserve every anchor coordinate and keep the Cutlass internal
behind its closed shutter. Fit the forward upper-port cup with exactly two beam
optics. Fit the higher farther-aft port cup with exactly two blunt sandcaster
projectors, each having exactly four dark ports in a 2 x 2 array. Fit the lower
port/keel cup with exactly two closed rectangular missile shutters. Keep three
double-beam cups hidden. Keep the breaching collar unarmed. Apply Rogue Tide
aubergine, vermilion, mustard, blackened chrome, charcoal, and cyan, including
mismatched replacement panels. Add no PD or extra weapon.
```

## Production sequence and provenance

Generated for Cepheus Trader on 2026-08-21 with OpenAI image generation in
built-in mode. The neutral singleton anchor was accepted without correction.
The first production pass duplicated the visible beam language and omitted a
readable sandcaster. A scoped edit converted only the higher, farther-aft cup;
a final single-object correction made its two four-port projector faces legible
without moving the beam mount, missile shutters, hangar, or breaching collar.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-087-revenant.png` | Neutral singleton anchor | Approved |
| `site/ship-art/masters/ship-087-revenant.png` | Corrected full-resolution master | Approved |
| `site/assets/ships/ship-087-revenant.webp` | Published catalog plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
