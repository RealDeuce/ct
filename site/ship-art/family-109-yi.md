# Yi Visual Manifest

*Status: catalog production singleton family 109*

## Mechanical identity

- Record: `ship-109` Yi; Admiralty Line (P7)
- Hull: 1,000 displacement tons / approximately 14,000 m³; streamlined and
  radiation shielded
- Dimensions: 78.0 × 34.0 × 24.0 m
- Fit: TL11, eight armor points, armored bridge and drives, improved magazine,
  emergency power, Jump H / Jump-2, maneuver P and power S / thrust 4, two
  weeks endurance, advanced electronics, holographic controls, and dual Model
  3 bis/fib computers
- Complement: thirty-five crew, seven staterooms, four crew berthings, armory,
  briefing room, chapel, three emergency berths, office, two-bed medical bay,
  repair drones, and 14.65 tons cargo
- Craft: one Wayfarer Cargo utility boat and one Caduceus fast launch in one
  50-ton full hangar
- Weapons: two single particle-beam turrets, two triple-beam turrets, two
  triple-sandcaster turrets, two 100-ton meson bays, two 50-ton particle-beam
  bays, and ten point-defense lasers

Yi is an independent compact fleet combatant, not a shortened Togo. Its broad
armored shield body, rounded triangular ram prow, huge cheek battery, high
command spine, deep keel, and clustered cylindrical drive tail form a distinct
family. Axial order, recessed machinery, and finish retain Admiralty identity.

## Canonical architecture

The 78 × 34 × 24 m streamlined hull is a broad faceted arrowhead/teardrop
with thick occupied weapon cheeks, a shallow raised command spine and buried
six-pane bridge, full ventral keel, narrowing afterbody, and one integrated
cluster of armored stern cylinders. It has six occupied deck levels and reads
as a substantial naval ship, never a fighter. Do not add wings, tail fins,
external trusswork, a split prow, or detached nacelles.

One closed 12 × 6.5 m aft-port shutter serves shaped internal berths for the
18.8 × 8.4 × 4.0 m Wayfarer Cargo and 17.5 × 6.4 × 4.6 m Caduceus. Both
craft remain internal. A separate 14.65-ton cargo shutter, docking collar,
four of seven stateroom windows, and two of three emergency-berth panels are
visible, closed, and unarmed. Crew berthing is windowless.

## Ten-coordinate coverage map

Five large stations are visible and five hidden:

1. Lower-forward port cheek: one enormous two-deck 100-ton meson-gun bay.
2. Upper-aft port chine: one long shallower 50-ton particle-beam bay.
3. Forward upper-port prow/shoulder: one single particle-beam turret.
4. Midship port flank: one triple-beam turret with exactly three cyan optics.
5. Aft lower-port flank: one triple-sandcaster turret with exactly three short
   perforated projector heads.

The hidden half repeats the same five installations. The meson bay is visibly
about twice the architectural mass of the particle bay, and both dwarf the
turrets. The three trainable pairs cover forward, upper, flank, lower, and aft
arcs without a dorsal row. Ten PD coordinates split five visible/five hidden;
each visible node is a tiny orange-red-rimmed single optic.

Docking, Jump, processor, scoop, door, sensor, and drive features remain
unarmed and must not be mistaken for extra stations.

## Machinery, armor, finish, and plate

Five processors use three visible/two hidden grilles with one protected
streamlined ventral scoop. Internal Jump-H machinery reads as one segmented
service/radiator band and two flush coil housings, never an external ring or
glow. Two visible/two hidden maneuver apertures and larger separate power
grilles form the clustered P/S thrust-4 tail. Eight armor points, radiation
shielding, protected bridge/drives, improved magazines, and emergency power
read through massive cheeks, deep recesses, redundant ribs, chrome load paths,
and divided engineering services.

Admiralty finish uses deep navy `#203B69`, warm white `#E8E5D4`, signal red
`#C43C35`, bright aluminum/chrome, and gunmetal. Use the complete front-port
three-quarter view, restrained near-orthographic perspective, charcoal stars,
faint teal grid, and original late-1970s/early-1980s gouache-and-airbrush
technical art. No text, logo, watermark, open door, external craft, weapon
fire, planet, modern CGI, wing, tail, or Jump ring.

## Invariants and reusable prompts

```text
Neutral anchor: create one independent 78 x 34 x 24 m streamlined 1,000-ton
fleet combatant as a compact armored shield body: broad faceted arrowhead/
teardrop, rounded triangular ram prow, thick occupied weapon cheeks, raised
axial command spine with buried six-pane bridge, deep ventral keel, narrowing
afterbody, and integrated clustered cylindrical thrust-4 tail. Reserve four
bays with one enormous two-deck lower-forward port 100-ton bay and one long
upper-aft port 50-ton bay visible. Reserve six turret rings with three visible
at forward-upper, midship-flank, and aft-lower coordinates; reserve ten PD
covers with five visible. Leave every weapon blank. Add a closed internal
two-berth hangar for Wayfarer Cargo plus Caduceus, cargo shutter, docking
collar, 4-visible/3-hidden stateroom windows, 2-visible/1-hidden emergency
panels, 3-visible/2-hidden processors, scoop, Jump-H band and two coil housings,
and clustered P/S stern. Show armor eight, radiation shielding, protected
bridge/drives, improved magazine, and emergency power. Neutral warm-white
primer and aluminum, period gouache and airbrush.

Yi fit: preserve every anchor coordinate and shield-body geometry. Activate the
visible 100-ton meson bay and 50-ton particle bay, one visible single particle
turret, one visible triple-beam turret with exactly three cyan optics, one
visible triple-sandcaster with exactly three short perforated heads, and five
tiny PD nodes; keep matching installations hidden. Keep doors closed, craft
internal, and non-weapons unarmed. Apply Admiralty navy, warm white, signal
red, bright aluminum/chrome, and gunmetal.
```

## Production sequence and provenance

Generated on 2026-08-21 with OpenAI image generation in built-in mode. The
neutral anchor established the shield body and bay scale. The production pass
preserved the hull but initially rendered two emitters on both flank turrets.
Two scoped beam edits were needed to achieve exactly three optics; one final
single-station edit replaced the lower pair with exactly three perforated
sandcaster heads while preserving the approved plate.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-109-yi.png` | Neutral singleton anchor | Approved |
| `site/ship-art/masters/ship-109-yi.png` | Corrected full-resolution master | Approved |
| `site/assets/ships/ship-109-yi.webp` | Published catalog plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
