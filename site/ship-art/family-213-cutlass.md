# Cutlass Visual Manifest

*Status: catalog production family 213*

## Mechanical identity

- Family/member: `family-213`; `ship-213` Cutlass
- Shipyard path: Path 6, Rogue Tide Yards
- Hull: 50 displacement tons / approximately 700 m³
- Dimensions: 22 m long, 8.5 m wide, and 5.2 m high
- Configuration: streamlined; technology level 11; non-standard design
- Protection: two whole points of small-craft crystaliron armor
- Drives: maneuver sK and power sK / thrust 4; no Jump drive
- Endurance: two weeks of power-plant fuel
- Control and complement: two-person control cabin, two operators, and twenty-
  four passenger acceleration seats
- Mission support: one airlock, one additional fire-control station, and 17.25
  tons cargo
- Armament: one forward-fixed single beam laser; the sole hardpoint is occupied
- Parent vessel: one Cutlass is carried in Revenant's (`ship-87`) 50-ton full
  hangar

Cutlass is a capacious local assault transport rather than a tiny fighter or an
independent starship. Its two operators deliver a seated boarding party and a
substantial captured-stores load at four gravities, but the craft has no
stateroom, Jump capability, long endurance, or independent support plant.

## Canonical assault bus

The 22 × 8.5 × 5.2 m streamlined hull is a compact faceted atmospheric assault
bus: short armored wedge command nose, deep central seated-troop pressure cabin,
broad lower cargo belly, clipped shoulders, and short square engineering stern.
This envelope plausibly encloses approximately 700 m³ while remaining within
Revenant's dedicated cutter hangar. It is an independent family silhouette, not
a miniaturized Blackbeard or Moriarty hull.

Exactly three narrow dark panes form the protected two-operator control brow.
The twenty-four passenger seats and their occupants remain internal. One human-
scale closed boarding airlock/docking collar lies on the forward port flank. A
separate large closed rectangular shutter on the mid-to-aft lower port flank
serves the 17.25-ton cargo hold. Door frames, handles, landing-door seams, and
maintenance access remain human-scale, and the cargo shutter never doubles as
a boarding portal or weapon aperture.

Two-point armor appears as a thin protected edge band and faceted shoulder
plates rather than capital-ship slabs. Mismatched replaceable service panels
are intentional Rogue Tide maintenance language, not stealth coating or battle
damage.

## Fixed weapon coordinate

The single hardpoint is fixed. Its only exterior weapon is one slim cyan-blue
beam-laser optic recessed behind a blackened-chrome rectangular collar in the
forward-port chin/shoulder, aligned along the flight axis. It has no barrel,
circular mounting ring, trainable cup, paired emitter, or dorsal partner.

The standard port plate therefore exposes the design's one and only weapon;
there is no hidden-half turret allocation to infer. Small maintenance nodes,
control panes, airlock controls, cargo hardware, landing seams, and stern
grilles never count as weapons or point defense.

## Drives and Rogue Tide recognition

Four distinct recessed stern maneuver apertures and one separate central power
grille express the four-gravity sK/sK plant. Cutlass has no Jump drive, so it
carries no Jump radiator, coil housing, field vane, or ring. It also has no fuel
scoop or processor. Never add flame, luminous exhaust, an external tank, or a
generic intake.

Rogue Tide Yards uses deep aubergine `#4D294F`, hot vermilion `#D94B35`, mustard
`#D4A62A`, and blackened chrome. Aubergine dominates the armor; vermilion marks
boarding, cargo, fixed-weapon, and machinery boundaries; mustard identifies a
few replaceable service panels, handholds, and datum accents; blackened chrome
protects the laser, door, armor, landing, and drive collars. Saturated opaque
color and selective bright-metal edges establish the shared yard language,
while the assault-bus geometry remains uniquely Cutlass.

## Invariants and production prompts

Preserve the 22 × 8.5 × 5.2 m silhouette; three-pane control brow; plain dorsal-
forward armor; distinct boarding airlock and cargo shutter; one fixed laser
coordinate; four maneuver apertures and one power grille; armor band; landing
seams; camera; crop; backdrop; and lighting. Keep all seats, people, and cargo
internal and every door closed.

```text
Neutral anchor: create the unpainted 22 × 8.5 × 5.2 m, 50-ton streamlined
Cutlass assault bus with short armored command wedge, deep seated-troop cabin,
lower cargo belly, clipped shoulders, square stern, exactly three control panes,
one closed boarding airlock, one separate closed cargo shutter, one unarmed
flush forward-fixed laser socket without a ring, four maneuver apertures, and
one power grille. No other weapon coordinate, Jump equipment, fuel gear, paint,
text, symbol, open door, person, or glow.

Cutlass fit: preserve every anchor coordinate. Convert only the single fixed
socket into one slim cyan optical slit in a blackened-chrome rectangular collar,
and apply Rogue Tide aubergine, vermilion, mustard, and blackened chrome. Keep
the dorsal armor plain, all doors closed, and passengers internal. No turret,
second emitter, point defense, Jump equipment, fuel scoop, pirate insignia,
text, weapon fire, or drive glow.
```

## Production sequence and provenance

1. Generate the neutral assault-bus hull with fixed service, cargo, airlock,
   drive, and weapon coordinates.
2. Remove a surplus fourth control pane, the turret-like dorsal cap, and paired
   decorative nose apertures; establish exactly three panes and one flush fixed
   socket without changing the approved hull.
3. Derive the production fit from the corrected anchor, using Blackbeard only
   as a Rogue Tide finish reference and preserving Cutlass geometry.
4. Review the finished plate for one fixed optic, closed and separate airlock
   and cargo doors, four-drive stern, absent Jump/fuel hardware, and parent-
   hangar compatibility.

Artwork was generated for Cepheus Trader on 2026-08-20 with OpenAI image
generation in built-in mode. The production plate descends directly from the
corrected neutral anchor; Blackbeard supplied finish language only.

| Asset | Purpose | Review status |
| --- | --- | --- |
| `site/ship-art/anchors/family-213-cutlass.png` | Corrected neutral Cutlass assault-cutter chassis | Approved |
| `site/ship-art/masters/ship-213-cutlass.png` | Archival Cutlass catalog master | Approved |
| `site/assets/ships/ship-213-cutlass.webp` | Published Cutlass plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and
`OPEN_GAME_LICENSE.md`.
