# Togo Visual Manifest

*Status: catalog production singleton family 108*

## Mechanical identity

- Record: `ship-108` Togo; Admiralty Line (P7)
- Hull: 1,200 displacement tons / approximately 16,800 m³; streamlined,
  self-sealing, and radiation shielded
- Dimensions: 92.0 × 30.0 × 23.0 m
- Fit: TL11, eight armor points, armored bridge and drives, improved magazine,
  emergency power, Jump L / Jump-2, maneuver and power T / thrust 5, four
  weeks endurance, advanced electronics, hardened holographic bridge, and dual
  Model 3 bis/fib computers
- Complement: 106 crew, ten staterooms, twelve crew berthings, two armories,
  two briefing rooms, sensor-control room, four-place brig, two workshops,
  three-bed medical bay, repair drones, and 3.5 tons cargo
- Craft: one Wayfarer Cargo utility boat and one Caduceus fast launch in one
  50-ton full hangar
- Weapons: six triple-beam turrets; two mixed missile/missile/sandcaster triple
  turrets; two meson-gun bays; two particle-beam bays; twelve point-defense
  lasers

Togo is an independent intercept-destroyer family, not a streamlined Salamis
or Duncan. Its armored-skipjack lifting body, canoe prow, buried command
blister, deep lower keel, and paired stern cheeks define a dedicated pursuit
hull while its axial order and finish identify Admiralty Line construction.

## Canonical architecture

The 92 × 30 × 23 m hull is one continuous flattened lifting-body lance with a
long faceted canoe prow, recessed six-pane command blister, broad occupied
midbody shoulders, deep smooth ventral keel, gently pinched aft waist, and
paired armored stern cheeks around a squared drive block. It is streamlined
without becoming a fighter: six to seven deck levels, human-scale windows,
fine plating, and substantial enclosed volume remain legible. Never add wings,
a tail, an external truss, or an open fork.

One closed 12 × 6.5 m port shutter serves a shaped internal two-berth bay for
the 18.8 × 8.4 × 4.0 m Wayfarer Cargo and 17.5 × 6.4 × 4.6 m Caduceus.
Both craft remain internal. A separate cargo/service shutter, docking collar,
five of ten stateroom windows, and one of two emergency-berth panels are
visible, closed, and unarmed. Crew berthing is windowless.

## Twelve-coordinate coverage map

Six large stations are visible and six hidden:

1. Deep lower-forward port cheek: one meson-gun bay.
2. Long upper-forward port chine: one particle-beam bay.
3. Forward upper-port shoulder: one triple-beam turret with three cyan optics.
4. Midship port flank: second triple-beam turret with three cyan optics.
5. Aft lower-port flank: third triple-beam turret with three cyan optics.
6. Aft upper-port shoulder: one mixed triple turret with two closed missile
   shutters and one blunt perforated sandcaster projector.

The hidden half repeats one meson bay, one particle bay, three beam turrets, and
one mixed turret. The axial bays carry the pursuit attack while the trainable
battery overlaps forward, upper, lower, flank, and aft arcs rather than forming
a dorsal row. Twelve PD coordinates split six visible/six hidden; every visible
node is a tiny orange-red-rimmed single optic.

Bay apertures remain materially larger and more deeply rooted than turrets.
The docking collar, Jump housings, processor grilles, scoop, doors, sensors,
stern apertures, and support panels are not weapons.

## Machinery, armor, finish, and plate

Ten fuel processors use five visible/five hidden grilles with one protected
streamlined ventral scoop. Internal Jump-L machinery reads as a segmented
service/radiator band and four flush coil housings, never an external ring or
glow. Three visible and three hidden maneuver apertures surround separate
power grilles in the T/T thrust-5 stern. Eight armor points, radiation
shielding, self-sealing skin, protected bridge/drives, and emergency power read
through layered canoe-prow cheeks, recessed openings, redundant ribs, bright
load paths, and a divided engineering block.

Admiralty finish uses deep navy `#203B69`, warm white `#E8E5D4`, signal red
`#C43C35`, bright aluminum/chrome, and gunmetal. Use the complete front-port
three-quarter view, restrained near-orthographic perspective, charcoal stars,
faint teal plotting grid, and original late-1970s/early-1980s gouache-and-
airbrush technical-art finish. No text, logo, watermark, open door, external
craft, weapon fire, planet, modern CGI, wing, tail, or Jump ring.

## Invariants and reusable prompts

```text
Neutral anchor: create one independent 92 x 30 x 23 m streamlined 1,200-ton
intercept destroyer as an armored skipjack: continuous flattened lifting-body
lance, canoe-like faceted prow, buried six-pane command blister, broad occupied
shoulders, deep ventral keel, pinched aft waist, paired armored stern cheeks,
and squared thrust-5 drive block. Reserve four bay coordinates with one deep
lower-forward port cheek and one long upper-forward port chine visible. Reserve
eight turret rings with four visible at forward-upper shoulder, midship flank,
aft-lower flank, and aft-upper shoulder; reserve twelve PD covers with six
visible. Leave every weapon blank. Add a closed 12 x 6.5 m internal two-berth
hangar for Wayfarer Cargo plus Caduceus, cargo/service shutter, docking collar,
5-visible/5-hidden stateroom windows, 1-visible/1-hidden emergency panels,
5-visible/5-hidden processors, streamlined scoop, Jump-L service band and four
coil housings, and 3-visible/3-hidden maneuver apertures. Show armor eight,
radiation shielding, self-sealing skin, and protected bridge/drives. Neutral
warm-white primer and aluminum, period gouache and airbrush.

Togo fit: preserve every anchor coordinate and the complete armored-skipjack
hull. Activate the visible lower-forward meson bay and upper-forward particle
bay; activate exactly three visible triple-beam turrets, each with three cyan
optics, and one visible mixed triple turret with exactly two closed missile
shutters plus one perforated sandcaster projector. Keep the matching six large
stations hidden. Activate exactly six tiny PD nodes and keep six hidden. Keep
all doors closed, both craft internal, and docking, Jump, processor, scoop, and
stern features unarmed. Apply Admiralty deep navy, warm white, signal red,
bright aluminum/chrome, and gunmetal.
```

## Production sequence and provenance

Generated on 2026-08-21 with OpenAI image generation in built-in mode. The
neutral anchor established the independent armored-skipjack pursuit hull. One
constrained production pass preserved its geometry, distributed the visible
battery across four arcs, distinguished the mixed turret's three apertures,
and applied the Admiralty finish; no correction pass was required.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-108-togo.png` | Neutral singleton anchor | Approved |
| `site/ship-art/masters/ship-108-togo.png` | Full-resolution production master | Approved |
| `site/assets/ships/ship-108-togo.webp` | Published catalog plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
