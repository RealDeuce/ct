# Marathon Visual Manifest

*Status: catalog production singleton family 103*

## Mechanical identity

- Record: `ship-103` Marathon; Redoubt Shipbuilding (P8)
- Hull: 1,200 displacement tons / approximately 16,800 m³; standard
- Dimensions: 88.0 × 36.0 × 24.0 m
- Fit: TL11, eight armor points, armored drives and magazines, Jump L / Jump-2,
  maneuver and power S / thrust 4, three weeks endurance, advanced electronics,
  hardened bridge, and dual Model 3 bis/fib computers
- Complement: thirty-six crew, twelve staterooms, thirty-place barracks, three
  armories, two briefing rooms, six emergency berths, two-bed medical bay,
  probe and repair drones, three ATV hangars, and 193.9 tons cargo
- Craft: one Proteus Cargo and two Jason transports in one 110-ton full hangar
- Weapons: four triple-beam, two triple-missile, and two triple-sandcaster
  turrets; four particle-beam barbettes; twelve point-defense lasers

Marathon is an independent siege-ferry cruiser, not a stretched Bastion,
Sentinel, or Cochrane. Its triple shield prow, two armor/load spines, recessed
boarding waist, tall mission citadel, and three-berth launch shoulder define a
distinct Redoubt assault hull.

## Canonical architecture

The 88 × 36 × 24 m filled standard hull has a three-layer blunt shield prow,
deeply buried six-pane command slit, thick occupied weapon shoulders, protected
waist, central troop/cargo citadel, broad asymmetric port launch shoulder,
paired longitudinal armor spines, and deep squared engineering stern. It is not
streamlined, winged, modular, or an open truss.

One closed 15 × 8.5 m hangar shutter serves a branched internal bay about 34 m
deep and 24 m wide. Three berths clear the approved 26 × 9.5 × 5.4 m Proteus
Cargo and two 21 × 7.6 × 4.1 m Jasons; all remain internal. Two ATV doors show
portside and one is hidden starboard. A separate cargo shutter, docking collar,
support hatches, six of twelve stateroom windows, and three of six emergency-
berth panels remain closed and unarmed. Barracks are windowless.

## Twelve-coordinate coverage map

Six large stations are visible and six hidden:

1. Lower-forward port cheek: one long low particle-beam barbette channel.
2. Upper-aft port armor spine: second long particle-beam barbette channel.
3. Forward upper-port shoulder: triple-beam turret with three cyan optics.
4. Midship port flank: second triple-beam turret with three cyan optics.
5. Upper-mid port shoulder: triple-missile turret with three closed shutters.
6. Lower-aft port flank: triple-sandcaster turret with three perforated heads.

The hidden half repeats two particle barbettes, two beam turrets, one missile,
and one sandcaster. The battery therefore overlaps upper, lower, forward, aft,
and flank fields without becoming a dorsal row. Twelve PD coordinates split six
visible/six hidden; each visible node is a tiny orange-rimmed single optic.

The large side circle is a docking collar. Two large flush disks on the aft
crown are Jump coil-service housings. Neither is a weapon. Processor grilles,
scoop, doors, sensors, drive apertures, and support panels also remain unarmed.

## Machinery, armor, finish, and plate

Five processors use three visible/two hidden ribbed grilles and one protected
ventral scoop. Internal Jump-L machinery reads as one segmented transverse
service band and four flush coil housings, never a ring or glow. Two visible and
two hidden maneuver apertures flank separate power grilles at the S/S stern.
Eight armor points and protected bulkheads read through layered shields, deep
recesses, redundant ribs, neutron-shadow plates, and stainless load paths.

Redoubt finish uses fire red `#B83B2E`, orange `#E9782E`, bone `#D8CCA9`,
bright stainless steel, and gunmetal/dark teal. Use the complete front-port
three-quarter view, restrained near-orthographic perspective, charcoal stars,
faint teal grid, and original late-1970s/early-1980s gouache-and-airbrush
technical-art finish. No text, logo, watermark, open door, external craft,
weapon fire, planet, modern CGI, wing, tail, or Jump ring.

## Invariants and reusable prompts

```text
Neutral anchor: create one independent 88 x 36 x 24 m standard 1,200-ton
siege-ferry citadel with triple shield prow, six-pane buried bridge, two weapon
shoulders, recessed boarding waist, tall troop/cargo citadel, two armor spines,
port three-berth hangar shoulder, and deep stern. Add one closed 15 x 8.5 m
hangar for Proteus Cargo plus two Jasons, 2-visible/1-hidden ATV doors, cargo and
support panels, 6-visible/6-hidden staterooms, and 3-visible/3-hidden emergency
berths. Reserve four particle-barbette channels with two visible; eight turret
coordinates with four visible across upper/flank/lower arcs; twelve PD covers
with six visible. Leave all weapons blank. Show 3-visible/2-hidden processors,
scoop, Jump-L band and four coil housings, S/S stern, eight armor, and protected
bulkheads. Neutral bone primer and aluminum, period gouache and airbrush.

Marathon fit: preserve every anchor coordinate. Activate two visible particle
barbettes, two triple beams, one triple missile with three closed shutters, one
triple sandcaster with three perforated heads, and exactly six tiny PD nodes;
keep the matching battery hidden. Keep docking collar and aft coil disks
unarmed, all three craft internal, and every door closed. Apply Redoubt fire
red, orange, bone, stainless, and gunmetal.
```

## Production sequence and provenance

Generated on 2026-08-21 with OpenAI image generation in built-in mode. The
neutral anchor was accepted. The first production pass preserved the distributed
battery but reduced the lower-aft sandcaster station to PD scale; one scoped
single-object edit restored the missing full-size triple sandcaster.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-103-marathon.png` | Neutral singleton anchor | Approved |
| `site/ship-art/masters/ship-103-marathon.png` | Corrected full-resolution master | Approved |
| `site/assets/ships/ship-103-marathon.webp` | Published catalog plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
