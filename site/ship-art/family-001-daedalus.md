# Daedalus Family Visual Manifest

*Status: initial catalog production family*

## Mechanical identity

- Family: `family-1`, Daedalus
- Members: `ship-1` Hermes, `ship-2` Labyrinth, `ship-3` Knossos,
  `ship-4` Minotaur
- Hull: 10 displacement tons / approximately 140 m³
- Configuration: distributed
- Technology level: 11
- Shared drives: `sA` maneuver and `sA` power
- Shared control: two-person cockpit
- Shared endurance: one week
- Shared limitations: no Jump drive and no installed weapon mount

## Canonical family anchor

The Daedalus is a compact orbital work-pod chassis rather than an aircraft. It
has an open load-bearing spine connecting three visually separate systems:

1. a forward faceted pressure cockpit with two seats abreast, two small angled
   forward viewport panes, and a direct personnel hatch;
2. a central rectangular mission-module cradle with two chrome locking rails;
3. two short cylindrical aft drive/fuel drums mounted symmetrically above and
   below the spine, with recessed circular maneuver-field faces.

Canonical maximum dimensions are 9.6 m long, 6.2 m wide across the cradle and
equipment shoulders, and 4.2 m high. The sparse bounding volume is not solid;
the cockpit, mission module, drive drums, and connecting service volumes sum to
the required approximately 140 m³.

The family silhouette is a blunt faceted head, narrow exposed waist, boxy
mission cradle, and paired aft drums. It has no wings, tail, atmospheric
fairing, panoramic glass canopy, external rocket nozzles, landing skids, or
weapon barrels. It operates between ships, stations, and local installations.

### Invariants

Every member preserves:

- exact 9.6 × 6.2 × 4.2 m envelope and component proportions;
- forward cockpit shape, two-pane viewport, and direct hatch location;
- central spine and two chrome module rails;
- paired aft drive/fuel drum dimensions and positions;
- four small attitude-control blisters;
- principal frame joints, service panels, camera, perspective, and lighting;
- a front-port three-quarter view from 12 degrees above centerline;
- no weapons and no visual implication of a Jump drive.

## Variant deltas

### `ship-1` Hermes — Venture Passage Works

- Role: dispatch courier
- Cargo: 4.9 tons
- Fit: a sealed priority-cargo box fills the mission cradle; one broad flush
  loading door faces port, with small protected status lenses and no passenger
  windows.
- Livery: sunflower `#F2C230` primary, cobalt `#174A8B` structural panels,
  signal orange `#E96D2F` loading-door edge, bright chrome rails and collars.
- Finish: maintained commercial enamel with slight handling wear at the door.

### `ship-2` Labyrinth — Outer Reach Cooperative

- Role: utility tug for towing, positioning, salvage, and mining support
- Cargo: 2.9 tons
- Fit: the module cradle contains a rugged tool/cargo locker and the folded
  root of one large two-joint grappling arm. The arm lies along the port side
  when stowed and ends in a three-jaw industrial claw. It is machinery, not a
  weapon.
- Livery: avocado `#7A9238` main panels, ochre `#C9902C` machinery guards,
  brick red `#A63F2F` moving-part warnings, brushed aluminum frame and drums.
- Finish: field-repair panels, polished contact surfaces, and restrained work
  wear without generalized grime.

### `ship-3` Knossos — Concord Exchange Yards

- Role: passenger launch for eight travelers and two crew
- Cargo: 0.9 tons
- Fit: a clean enclosed passenger module fills the cradle, with four small
  evenly spaced port viewports, a clearly human-scale direct boarding hatch,
  and a bright docking handrail. No cargo door, luxury glazing, or airlock.
- Livery: warm ivory `#F1E3C2` main shell, vermilion `#E44B2D` passenger-door
  surround, royal blue `#2354A3` lower panels, polished chrome rails/collars.
- Finish: clean high-turnover civil service with minimal edge wear.

### `ship-4` Minotaur — Rogue Tide Yards

- Role: boarding tug
- Cargo: 1.9 tons
- Fit: the same grappling-arm geometry as Labyrinth, plus a compact cylindrical
  airlock/docking collar mounted on the starboard face of the mission module.
  The port-view arm remains visible; the collar must not enlarge the family
  envelope. Tool lockers replace part of the Labyrinth cargo module.
- Livery: aubergine `#4D294F` main panels, hot vermilion `#D94B35` grapple and
  airlock warnings, mustard `#D4A62A` service panels, blackened chrome rails.
- Finish: deliberate mismatched access panels and polished grapple contact
  surfaces; no stealth claim, weapons, spikes, or pirate insignia.

## Catalog plate

- Canvas: 3:2 landscape
- View: complete front-port three-quarter, 12 degrees above centerline
- Perspective: restrained near-orthographic
- Lighting: warm upper front-port key, weak cool aft rim
- Backdrop: charcoal-black sparse starfield and faint teal plotting grid
- Medium: original 1970s/1980s gouache-and-airbrush technical catalog plate
- Prohibited: text, labels, logos, watermarks, other craft, weapon fire, motion
  blur, planets, dramatic lens distortion, modern military-gray default, or
  recognizable franchise design

## Production sequence

1. Approve the unpainted family anchor.
2. Derive Hermes and Knossos from the anchor.
3. Derive Labyrinth with the family grappling-arm installation.
4. Derive Minotaur from the approved Labyrinth geometry, adding only the
   airlock/docking collar, reduced locker volume, and Rogue Tide finish.
5. Validate the complete family side by side before publishing any member.

## Asset inventory and provenance

All five images were generated for Cepheus Trader on 2026-08-17 with OpenAI
image generation from this manifest. The manifest is the retained production
brief: later revisions must record any changed geometry, prompt constraints,
or mechanical source before replacing an approved asset.

| Asset | Purpose | Review status |
| --- | --- | --- |
| `anchors/family-001-daedalus.png` | Unpainted shared chassis anchor | Approved family geometry |
| `masters/ship-001-hermes.png` | Full-resolution Hermes master | Approved for catalog |
| `masters/ship-002-labyrinth.png` | Full-resolution Labyrinth master | Approved for catalog |
| `masters/ship-003-knossos.png` | Full-resolution Knossos master | Approved for catalog |
| `masters/ship-004-minotaur.png` | Full-resolution Minotaur master, corrected collar position | Approved for catalog |

The derived WebP publication files are under `site/assets/ships/`. The artwork
was authored for and distributed as part of Cepheus Trader and is Open Game
Content under the repository designation in `LICENSE.md` and the Open Game
License version 1.0a in `OPEN_GAME_LICENSE.md`.
