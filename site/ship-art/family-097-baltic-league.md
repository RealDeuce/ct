# Baltic League Family Visual Manifest

*Status: catalog production family 097*

## Mechanical identity

- Members: `ship-97` Riga, `ship-98` Visby, and `ship-101` Novgorod
- Paths: Admiralty Line Works (P7) and Concord Exchange Yards (P1)
- Hull: 2,000 displacement tons / approximately 28,000 m³; standard
- Dimensions: 104.0 m long, 34.0 m wide, and 24.0 m high
- Shared: TL11, Jump-Q / Jump-2, maneuver N, power Q, three-week endurance,
  two armor points, advanced electronics, five mixed triple turrets, one
  30-ton full hangar, and one internal Jason

The records resolve to two exterior fits. Riga and Novgorod have the same
drives, armor, weapons, hangar, craft, twenty processors, and ten replenishment
systems; crew and cargo allocation are internal, so exact art reuse is
desirable. Visby keeps the chassis and weapons but replaces the replenishment
grid with freight doors and adds an air-raft hangar.

| Fit | Records | Path | Visible distinction |
| --- | --- | --- | --- |
| Baltic fleet logistics | 97, 101 | P7 | Ten coupling/boom stations; Admiralty finish |
| Visby bulk freighter | 98 | P1 | Ten freight leaves and low air-raft door; Concord finish |

## Canonical architecture

The family is an integrated 104 × 34 × 24 m heavy logistics hull with a blunt
eight-pane command block, long deep cargo/tank citadel, strong axial spine and
protected load rails, and a broad squared engineering block. It is neither
streamlined nor a distributed truss.

Five identical large utility frames occupy the visible port citadel and five
mirrored frames occupy starboard. The frames, centers, load rails, and door
outlines never move. P7 ships fit one circular coupling, stowed short boom,
protected hose reel, and isolation valves inside every frame, producing exactly
ten underway-replenishment systems. Visby closes every frame as a bulk-freight
shutter and retains no transfer gear.

Exactly five compact mixed triple turrets occupy fixed dorsal centerline rings.
Each has one narrow beam emitter, one closed missile tube, and one blunt
sandcaster. A separate 8.5 × 4.8 m aft-port full-hangar shutter and 22 m
internal axis accept Jason. The craft remains internal. A small low-port
reservation is inactive on the P7 ships and becomes Visby's air-raft door.

The internal Jump drive appears as one segmented transverse radiator band and
two flush coil-service blisters, never a ring or glow. Four maneuver apertures,
two power grilles, a ventral scoop grid, landing doors, and two-point armor are
common.

### Invariants

Preserve the dimensions, integrated silhouette, command block, citadel, spine,
rails, aft block, ten utility-frame coordinates, five turret centers, Jason
hangar, air-raft reservation, Jump landmarks, stern boundaries, scoop, landing
geometry, primary seams, camera, crop, backdrop, and lighting. Never add a
sixth frame or turret, external tank, deployed hose, external craft, second
hangar, wing, tail, truss, or Jump ring.

## Production fits

### Riga and Novgorod — records 97 and 101

- Five visible plus five mirrored replenishment stations
- Five mixed triple turrets; closed Jason hangar; inactive air-raft reservation
- Admiralty deep navy `#203B69`, warm white `#E8E5D4`, signal red
  `#C43C35`, and bright aluminum/chrome

### Visby — record 98

- All ten frames are closed freight shutters with handling sockets and rails;
  no coupling, hose, valve, reel, or boom remains
- Same five mixed turrets and closed Jason hangar; active low air-raft door
- Concord warm ivory `#F1E3C2`, royal blue `#2354A3`, vermilion
  `#E44B2D`, and polished chrome

## Catalog plate and reusable prompts

Use a complete front-port three-quarter view from about 12 degrees above,
restrained near-orthographic perspective, 3:2 landscape, warm front-port key,
cool aft rim, charcoal stars, and faint teal grid. Render as original
late-1970s/early-1980s gouache-and-airbrush technical art with saturated enamel
and selective chrome. No text, logo, watermark, weapon fire, planet, modern
gray rendering, or recognizable franchise styling.

```text
Neutral anchor: create the 104 x 34 x 24 m standard logistics hull with an
eight-pane command block, deep citadel, spine, load rails, aft block, exactly
five visible and five mirrored blank utility leaves, five closed dorsal turret
rings, separate Jason hangar, low air-raft reservation, Jump radiator, two coil
blisters, fixed stern apertures, scoop, and landing doors.

Shared edit rule: preserve every hull and frame coordinate, five turret rings,
hangar, air-raft boundary, machinery landmark, camera, and lighting. Change only
utility-leaf contents, air-raft door state, path finish, and listed equipment.

Riga/Novgorod: fit all ten leaves as replenishment stations with stowed gear;
activate five mixed triple turrets; close both small-craft doors; apply the
Admiralty palette. Use one exact plate for both records.

Visby: edit the approved logistics plate. Remove every coupling, boom, hose,
reel, pipe, and valve; convert all ten leaves to closed freight shutters.
Preserve all five turrets and the Jason hangar, activate the small air-raft
door, and apply the Concord palette.
```

## Production sequence and provenance

1. Approve the neutral hull and exact ten-leaf utility grid.
2. Produce the shared P7 logistics plate for records 97 and 101.
3. Derive Visby directly from it by changing only leaf contents, air-raft door,
   and path finish.

Generated for Cepheus Trader on 2026-08-19 with OpenAI image generation.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-097-baltic-league.png` | Neutral shared frame | Approved |
| `site/ship-art/masters/family-097-baltic-logistics.png` | Shared Riga/Novgorod master | Approved |
| `site/ship-art/masters/ship-098-visby.png` | Visby freight master | Approved |
| `site/assets/ships/family-097-baltic-logistics.webp` | Published shared plate | Approved |
| `site/assets/ships/ship-098-visby.webp` | Published Visby plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
