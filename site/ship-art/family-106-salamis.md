# Salamis Visual Manifest

*Status: catalog production singleton family 106*

## Mechanical identity

- Record: `ship-106` Salamis; Admiralty Line (P7)
- Hull: 1,000 displacement tons / approximately 14,000 m³; standard
- Dimensions: 84.0 × 27.0 × 19.0 m
- Fit: TL11, eight armor points, armored drives and bridge, Jump K / Jump-2,
  maneuver and power S / thrust 5, three weeks endurance, advanced electronics,
  hardened holographic bridge, and dual Model 3 bis/fib computers
- Complement: thirty crew, eleven staterooms, crew berthing, armory, two
  offices, two briefing rooms, three emergency berths, two-bed medical bay,
  probe and repair drones, workshop, and 87 tons cargo
- Craft: one Jason transport and one Charon launch in one 50-ton full hangar
- Weapons: two triple-beam, two triple-missile, and two triple-sandcaster
  turrets; two meson-gun bays; two particle-beam bays; ten point-defense lasers

Salamis is an independent line destroyer, not a Duncan variant despite the two
ships' similar displacement and fleet mission. Its split weapon prow, narrow
magazine waist, recessed command brow, asymmetric hangar shoulder, and long
high-thrust stern establish a separate Admiralty hull.

## Canonical architecture

The 84 × 27 × 19 m filled standard hull is a long, narrow split spear. Two
occupied forward prongs flank a shallow central trench; the recessed bridge
brow sits behind them. A narrow magazine spine leads aft through an offset port
hangar shoulder to an oversized squared engineering stern. It is neither
streamlined nor a winged fighter silhouette, and the prow is not an open fork.

One closed 10 × 6 m shutter serves an internal bay approximately 25 m long and
14 m wide, with separate berths for the 21 × 7.6 × 4.1 m Jason and compact
Charon. Both craft remain internal. A separate 8 × 5 m cargo shutter, docking
collar, support panels, six of eleven stateroom windows, and two of three
emergency-berth panels are visible, closed, and unarmed. Crew berthing is
windowless.

## Ten-coordinate coverage map

Five large stations are visible and five hidden:

1. Lower-forward port prong: one deep meson-gun bay.
2. Upper-forward port prong beside the trench: one long shallow particle-beam
   bay.
3. Forward upper-port shoulder: one triple-beam turret with three cyan optics.
4. Midship lower-port flank: one triple-missile turret with three narrow closed
   shutters.
5. Aft upper-port shoulder: one triple-sandcaster turret with three perforated
   projector heads.

The hidden half repeats one meson bay, particle bay, beam turret, missile
turret, and sandcaster turret. This gives forward, aft, upper, lower, and flank
coverage with no dorsal row. The two axial bay types carry the main attack;
the six trainable turrets protect their cross-axis and stern blind regions.
Ten point-defense coordinates split five visible/five hidden, each visible node
a tiny orange-rimmed single optic.

The flat disk on the farther prow prong is a sensor face. The port docking
collar and Jump housings are also unarmed. Bay apertures remain materially
larger and more deeply rooted than the three turret housings; doors, processors,
scoop, sensors, drive apertures, and support panels must not resemble weapons.

## Machinery, armor, finish, and plate

Five fuel processors use three visible/two hidden ribbed grilles with one
protected ventral scoop. Internal Jump-K machinery reads as a segmented service
band and paired flush coil housings, never an external ring or glow. Three
visible and three hidden maneuver apertures flank separate power grilles in the
S/S thrust-5 stern. Eight armor points and protected drives/bridge read through
layered prow cheeks, a buried brow, recessed bays, redundant ribs, and bright
structural load paths.

Admiralty finish uses deep navy `#203B69`, warm white `#E8E5D4`, signal red
`#C43C35`, bright aluminum/chrome, and gunmetal. Use the complete front-port
three-quarter view, restrained near-orthographic perspective, charcoal stars,
faint teal plotting grid, and original late-1970s/early-1980s gouache-and-
airbrush technical-art finish. No text, logo, watermark, open door, external
craft, weapon fire, planet, modern CGI, wing, tail, or Jump ring.

## Invariants and reusable prompts

```text
Neutral anchor: create one independent 84 x 27 x 19 m standard 1,000-ton line
destroyer as a long narrow split spear: two filled weapon prongs around a
shallow central trench, recessed six-pane command brow, narrow magazine waist,
long spine, asymmetric port hangar shoulder, and oversized squared thrust-5
stern. Add one closed 10 x 6 m internal two-berth hangar for Jason plus Charon,
separate 8 x 5 m cargo shutter, docking collar, support panels, 6-visible/
5-hidden stateroom windows, and 2-visible/1-hidden emergency-berth panels.
Reserve two meson-bay and two particle-bay coordinates with one of each visible;
reserve six turret coordinates with three visible across forward-upper,
midship-lower-flank, and aft-upper arcs; reserve ten PD covers with five visible.
Leave all weapons blank. Show 3-visible/2-hidden processor grilles, ventral
scoop, Jump-K service band and coil housings, 3-visible/3-hidden maneuver
apertures, armor eight, and protected bridge/drives. Neutral warm-white primer
and aluminum, period gouache and airbrush.

Salamis fit: preserve every anchor coordinate and the split-spear chassis.
Activate the visible lower-forward meson bay, upper-forward particle bay,
forward-upper triple beam with exactly three cyan optics, midship lower-port
triple missile with exactly three narrow closed shutters, aft-upper triple
sandcaster with exactly three perforated heads, and exactly five tiny PD nodes;
keep the matching five large stations and five PD nodes hidden. Keep the far
prong sensor disk, docking collar, Jump housings, and every door unarmed. Keep
both carried craft internal. Apply Admiralty deep navy, warm white, signal red,
bright aluminum/chrome, and gunmetal.
```

## Production sequence and provenance

Generated on 2026-08-21 with OpenAI image generation in built-in mode. The
neutral anchor established the distinct split-spear hull and was accepted. The
first production pass preserved its distributed battery but rendered the
missile station as two broad faces. A first scoped correction remained
ambiguous; a second single-object correction replaced only that station with
three narrow closed shutters while preserving the approved ship.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-106-salamis.png` | Neutral singleton anchor | Approved |
| `site/ship-art/masters/ship-106-salamis.png` | Corrected full-resolution master | Approved |
| `site/assets/ships/ship-106-salamis.webp` | Published catalog plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
