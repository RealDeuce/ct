# Ballista Visual Manifest

*Status: catalog production family 042*

## Mechanical identity

- Family/member: `family-42`; `ship-42` Ballista
- Shipyard path: Path 8, Redoubt Shipbuilding
- Hull: 80 displacement tons / approximately 1,120 m³
- Dimensions: 32 m long, 11.5 m wide, and 6.7 m high
- Configuration: streamlined; technology level 11; non-standard design
- Protection: four whole points of small-craft crystaliron armor
- Drives: maneuver sQ and power sQ / thrust 5; no Jump drive
- Endurance: one week of power-plant fuel
- Control and complement: four-person control cabin; two pilots, one turret
  gunner, and one other crew member
- Accommodation: three small-craft staterooms and one emergency low berth
- Mission support: one airlock, one small-craft fuel processor, and 32.3 tons
  cargo/stores
- Armament: one trainable single missile-rack turret with twelve standard
  missiles in an internal magazine

Ballista is a persistent local missile escort rather than an oversized fighter.
Its cabins and unusually large stores hold support patrol, screening, rescue
cargo, and mission equipment between sorties. It still depends on a base or
carrier for interstellar movement and full missile replenishment.

## Canonical armored-anvil skiff

The 32 × 11.5 × 6.7 m hull is a low broad “armored anvil/skiff”: long blunt
hexagonal hammer prow, thick occupied swept shoulder blocks, low protected
command ridge, deep rectangular stores keel with clipped aerodynamic corners,
narrowed engineering waist, and short wide stern with twin integral
buttresses. This coherent streamlined envelope plausibly contains about
1,120 m³.

Ballista is not a stretched Myrmidon, enlarged Bellerophon, or reduced Casemate.
It must retain its low anvil profile, dominant stores keel, and separate cabin
ridge rather than borrowing another Redoubt family's beetle, fighter-dart, or
patrol-citadel geometry.

Exactly five dark panes form the buried four-person command brow. Exactly three
smaller protected stateroom ports run along the visible port upper flank. One
closed human-scale pressure airlock and one separate closed emergency-low-berth
panel lie forward and low. A single closed segmented 7 × 3 m shutter dominates
the mid-to-aft port stores keel and serves the 32.3-ton hold. Its sill, latches,
and handling tie-downs remain heavy and human-readable.

## Missile station and coverage

The sole 3–4 m ring lies on the aft upper port flank/shoulder, off the dorsal
centerline and canted outward. A subtle inboard fairing protects the internal
twelve-round magazine. The production fit carries one low closed trainable
missile-rack drum with abrupt dark gridded shutters. No missile is visible.

The port three-quarter view exposes the one of one turret; zero weapon mounts
are hidden. Its flank position gives useful port, aft, dorsal, and cross-forward
escort coverage while accepting ventral/far-starboard blind regions. Never add
a mate, dorsal row, point-defense node, optical barrel, exposed missile, or
gun-like sensor.

## Fuel, drives, and absent systems

One square ribbed processor bank and adjacent flush circular refueling coupling
sit on the lower aft port belly. The bank may contain paired vent faces but is
one processor installation. Ballista fits no scoop. Four stern maneuver-field
apertures form port/starboard vertical pairs: the port two are visible and the
starboard two hidden. Two smaller separate ribbed grilles serve sQ/sQ power.

There is no Jump ring, radiator, coil housing, external tank, flame, or glow.
Paired compact prow sensor flats, landing doors, recovery sockets, bumpers,
tie-downs, handholds, and maintenance seams remain distinct from weapons.

## Redoubt recognition finish

- Major prow, shoulder, ridge, and upper armor blocks: fire red `#B83B2E`
- Lower/stores panels, recovery, weapon and fuel accents: orange `#E9782E`
- Accommodation, airlock, berth, and inset service panels: bone `#D8CCA9`
- Two armor bands, prow guards, cargo sill, glazing/access, hardpoint, fuel,
  and drive edges: heavy stainless with restrained chrome
- Shutters, grilles, landing hardware, and recesses: gunmetal

The finish shares Redoubt's rugged bright-metal and saturated-enamel language
with Bellerophon and Myrmidon, while the anvil/skiff chassis and side-mounted
missile drum remain Ballista-specific.

## Invariants and reusable prompts

Preserve the hull; five-pane bridge; three cabin ports; airlock and berth panel;
large stores shutter; one closed missile drum and magazine fairing; one
processor bank and coupling; absent scoop/Jump equipment; two-visible/two-
hidden drive map; armor, sensors, landing/service details; camera; crop;
backdrop; and lighting.

```text
Neutral anchor: create the original 32 × 11.5 × 6.7 m, 80-ton streamlined
Ballista armored-anvil/skiff with blunt hexagonal hammer prow, occupied
shoulders, low command ridge, deep stores keel, narrow waist, and short twin-
buttress stern. Use exactly five bridge panes, exactly three small port cabin
ports, one closed airlock, one closed emergency-berth panel, one large closed
7 × 3 m stores shutter, one processor grille bank and refueling coupling, and
one sealed aft-upper-port flank turret ring with internal magazine fairing.
Show two visible/two hidden maneuver apertures and distinct power grilles. Four
armor points use two heavy bands and overlapping plates. No installed weapon,
visible missile, second ring, point defense, scoop, Jump hardware, external
tank, open door, person, cargo, wing, tail, text, logo, fire, flame, or glow.
Neutral late-1970s/early-1980s gouache-and-airbrush technical plate, complete
front-port three-quarter on the standard plotting-grid starfield.

Ballista fit: preserve every anchor coordinate. Apply Redoubt fire red
#B83B2E, orange #E9782E, bone #D8CCA9, heavy stainless/restrained chrome, and
gunmetal recesses. In the existing aft upper-port ring install exactly one low
closed trainable single missile-rack drum with abrupt dark gridded shutters;
the twelve missiles remain internal and invisible. Use Myrmidon only for
Redoubt finish language and Bellerophon only for closed missile-rack vocabulary,
never their geometry. Keep five bridge panes, three cabin ports, all doors,
fuel hardware, armor, drives, and service details fixed. No second weapon,
beam optic, visible missile, point defense, scoop, Jump hardware, open boundary,
text, logo, weapon fire, flame, glow, modern CGI, or franchise design.
```

The first neutral pass produced six bridge panes. A scoped correction merged
only the two center panes, establishing the approved five-pane brow without
changing any other coordinate.

## Production sequence and provenance

1. Generate the neutral independent anvil/skiff hull and fixed equipment map.
2. Correct only the bridge partitions from six panes to five.
3. Save the corrected result as the family anchor.
4. Derive Ballista using Myrmidon only for Redoubt finish and Bellerophon only
   for closed missile-rack vocabulary; install the drum only in the flank ring.
5. Verify the one-visible/zero-hidden weapon map and all closed boundaries.

Artwork was generated for Cepheus Trader on 2026-08-20 with OpenAI image
generation in built-in mode. The reusable prompts are retained above.

| Asset | Purpose | Review status |
| --- | --- | --- |
| `site/ship-art/anchors/family-042-ballista.png` | Corrected neutral armored-anvil/skiff chassis | Approved |
| `site/ship-art/masters/ship-042-ballista.png` | Archival Redoubt missile-escort master | Approved |
| `site/assets/ships/ship-042-ballista.webp` | Published Ballista catalog plate | Approved |

The artwork is authored for and distributed as part of Cepheus Trader and is
Open Game Content under `LICENSE.md` and the Open Game License version 1.0a in
`OPEN_GAME_LICENSE.md`.
