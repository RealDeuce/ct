# Orion Visual Manifest

*Status: catalog production family 013*

## Mechanical identity

- Family/member: `family-13`; `ship-13` Orion
- Shipyard path: Path 7, Admiralty Line Works
- Hull: 10 displacement tons / approximately 140 m³
- Dimensions: 10.8 m long, 6.6 m wide, and 3.2 m high
- Configuration: streamlined; technology level 11; non-standard design
- Protection: one whole point of small-craft crystaliron armor
- Drives: maneuver sC and power sC / thrust 6; no Jump drive
- Endurance: one week of power-plant fuel
- Control: one pilot in a one-person cockpit; basic-military electronics,
  Model/2 computer, Evade/1, and Fire Control/1
- Capacity: no passengers or airlock; 0.475 tons residual mission stores
- Armament: one trainable single missile-rack turret and twelve standard
  missiles; the sole hardpoint is occupied

Orion is a numerous local patrol screen for traffic inspection and interception
near a carrier or base. Its one pilot, one-week endurance, tiny stores locker,
and internal twelve-round magazine leave no margin for independent operation.

## Canonical patrol seed

The 10.8 × 6.6 × 3.2 m streamlined hull is a thick double-convex almond or
“flying seed”: short rounded observation prow, widest pressure volume at
midships, continuous lenticular edge, and smoothly pinched compact stern. Its
filled oval envelope plausibly encloses approximately 140 m³ without wings,
arrowhead shoulders, a long axial dart, cylindrical barrel, tail, fin, or pod.

Orion is mechanically almost identical to Icarus I (`ship-9`), but family
assignment is the stronger geometric authority. It must never reuse Icarus's
11.8 m wingless lifting body, three-pane canopy, centerline ring, shoulder
chines, or twin circular drive layout. It is likewise distinct from Valkyrie's
arrowhead and Thermopylae's axial dart.

Exactly four dark panes form a shallow observation brow in one low armored
cockpit blister. The canopy seam is the only personnel access; there is no
pressure airlock, side hatch, passenger window, or cargo shutter. The residual
stores compartment is internal. Flush landing-pad seams, recovery sockets,
tie-downs, and maintenance panels remain human-scale. One continuous bright
edge band expresses the single armor point.

## Off-center missile hardpoint

The one 3–4 m circular hardpoint ring sits on the upper port shoulder beside
and behind the cockpit, clearly off centerline and canted slightly outward. Its
low trainable cup carries one broad closed signal-red rectangular armored
launch shutter. Shallow horizontal ribs reinforce that one shutter; they are
not open missile cells. All twelve missiles remain internal and invisible.

This placement gives the only mount useful forward, port, dorsal, and aft
coverage while preserving a clear observation brow and distinct patrol
silhouette. The single mount inevitably leaves ventral and far-starboard blind
regions. Do not add a centerline partner, lower cup, second shutter, visible
missile, gun barrel, or point-defense node. The small flush forward-shoulder
optical flat is a barrel-free sensor, not a weapon.

## Drives and Admiralty recognition

Three shallow transverse oval maneuver-field apertures cross the pinched stern,
with one separate narrow ribbed power-service grille. This transverse field
layout is unique to Orion among the published Admiralty fighters. Recovery
points and landing doors align to formal datum seams. There is no fuel scoop,
processor, Jump radiator, coil housing, ring, external tank, flame, or glow.

Admiralty Line Works uses deep navy `#203B69`, warm white `#E8E5D4`, signal red
`#C43C35`, and bright aluminum/chrome. Navy covers most of the lens shell; white
defines the observation prow and an orderly mid-hull recognition sector; red
marks the missile shutter, recovery points, and restrained drive safety details;
aluminum protects the armor edge, hardpoint, canopy, sensor, landing, and field
collars. Formal high-contrast blocks communicate fleet manufacture rather than
camouflage.

## Invariants and production prompts

Preserve the 10.8 × 6.6 × 3.2 m almond silhouette; four-pane brow and canopy
seam; off-center upper-port hardpoint; one-shutter missile cup; shoulder sensor
flat; three transverse stern apertures and separate power grille; armor band;
recovery and landing geometry; camera; crop; backdrop; and lighting. Keep the
pilot, magazine, and stores internal.

```text
Neutral anchor: create the unpainted 10.8 × 6.6 × 3.2 m, 10-ton streamlined
Orion patrol seed as one thick double-convex almond pressure hull with rounded
observation prow, wide middle, pinched stern, exactly four cockpit panes, canopy-
only access, one sealed off-center upper-port turret ring, one flush shoulder
sensor flat, three transverse maneuver apertures, one power grille, one armor
edge band, landing seams, recovery sockets, and tie-downs. No installed weapon,
airlock, side door, cargo shutter, wing, tail, Jump/fuel gear, text, or glow.

Orion fit: preserve every anchor coordinate. Replace only the off-center ring's
blank disk with one low trainable single missile-rack cup carrying one broad
closed signal-red shutter; keep all twelve missiles internal. Apply Admiralty
navy, warm white, red, and aluminum. No second shutter or mount, centerline
weapon, visible cell or missile, barrel, point defense, text, insignia, weapon
fire, or drive glow.
```

## Production sequence and provenance

1. Generate and approve the neutral lenticular patrol chassis with fixed
   cockpit, off-center hardpoint, sensor, recovery, and machinery coordinates.
2. Derive the production fit from that anchor, using Icarus I only as an
   Admiralty finish reference and fitting one closed missile shutter.
3. Review the finished plate against the mechanically equivalent Icarus I to
   prove family independence, then verify one weapon and no false access or
   fuel/Jump features.

Artwork was generated for Cepheus Trader on 2026-08-20 with OpenAI image
generation in built-in mode. The production plate descends directly from the
approved neutral anchor; Icarus I supplied finish language only.

| Asset | Purpose | Review status |
| --- | --- | --- |
| `site/ship-art/anchors/family-013-orion.png` | Neutral Orion patrol-seed chassis | Approved |
| `site/ship-art/masters/ship-013-orion.png` | Archival Orion catalog master | Approved |
| `site/assets/ships/ship-013-orion.webp` | Published Orion plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and
`OPEN_GAME_LICENSE.md`.
