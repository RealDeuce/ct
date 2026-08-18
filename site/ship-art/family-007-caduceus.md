# Caduceus Family Visual Manifest

*Status: catalog production family 007*

## Mechanical identity

- Family: `family-7`, Caduceus
- Members: `ship-7` Caduceus, `ship-8` Argus, `ship-145` Caduceus,
  `ship-146` Caduceus, `ship-185` Caduceus, and `ship-189` Caduceus
- Hull: 20 displacement tons / approximately 280 m³
- Configuration: streamlined
- Technology level: 11
- Shared maneuver drive: `sF` / thrust 6
- Shared control: one-person cockpit
- Shared accommodation: ten acceleration seats
- Shared access: one pressure airlock
- Shared protection: four points of crystaliron armor
- Shared limitations: no Jump drive and no sleeping accommodation

This is one standard flattened-cylinder fast-launch line represented by six
catalog records and two shipyard paths. Internal endurance and cargo allocation
do not change the exterior pressure body or its standard cargo aperture.

## Exterior reuse matrix

| Plate | Catalog records | Reason |
| --- | --- | --- |
| Concord unarmed Caduceus | `ship-145`, `ship-146` | Mechanically exterior-equivalent P1 fast launches |
| Venture unarmed Caduceus | `ship-7`, `ship-185`, `ship-189` | Same P2 armor, drives, seating, access, and unarmed exterior; fuel/cargo deltas are internal |
| Venture armed Argus | `ship-8` | Same chassis with one single beam-laser turret and `sG` power fit |

Exact image reuse is mandatory within each unarmed row. The Concord and Venture
plates reuse geometry but require different path component housings and finish.

## Canonical family anchor

The Caduceus is a single-deck launch built around a horizontal flattened
cylinder, not the lozenge lifting body of the Aeolus family. Its canonical
maximum dimensions are 17.5 m long, 6.4 m wide, and 4.6 m high. A twelve-meter
oval-section center barrel with rounded bow and stern caps plausibly encloses
the required approximately 280 m³ while leaving shallow aerodynamic fairings
outside the pressure shell.

The cockpit is inset into the blunt forward cap behind three protected panes.
A chrome-edged port passenger rail contains exactly five evenly spaced small
window sockets, representing five seat rows for ten passengers. A rectangular
pressure airlock follows the passenger rail. A broad, low, flush cargo door
occupies the aft port barrel below the airlock line. A narrow underside scoop
grid is faired into the forward lower shell.

One circular dorsal hardpoint socket sits on the centerline behind the occupied
zone. It is closed by a flush armored disk on every Caduceus plate and carries
one compact single beam-laser turret only on Argus. The aft cap contains two
recessed field-drive collars and one central power-service grille. The stronger
Argus `sG` power fit may enlarge the grille inside its fixed socket but may not
move the drive collars or reshape the stern.

### Invariants

Every member preserves exactly:

- the 17.5 × 6.4 × 4.6 m flattened-cylinder envelope and single-deck barrel;
- cockpit position, shape, and three protected panes;
- five-socket passenger rail, airlock, cargo door, and lower scoop grid;
- dorsal hardpoint ring, aft field collars, power-service socket, and seams;
- four-point armored shell depth and component reveals;
- camera, crop, perspective, backdrop, and lighting; and
- the absence of Jump-drive hardware, wings, tail, landing skids, missiles,
  and any weapon other than the Argus beam laser.

## Variant delta: Concord unarmed Caduceus

- Records: `ship-145`, `ship-146`
- Path: P1, Concord Exchange Yards
- Role: fast launch for passengers, cargo, boarding/inspection, or survey support
- Drive fit: `sF` maneuver and `sF` power
- Endurance/cargo: two weeks and 4.0 tons, internal
- Hardpoint: closed flush armored disk
- Livery: warm ivory shell, royal-blue lower barrel, vermilion airlock and
  cargo-door edge, polished aluminum/chrome window rail and drive collars
- Path fit: orderly rectangular service frames and flush commercial sensor strip

## Variant delta: Venture unarmed Caduceus

- Records: `ship-7`, `ship-185`, `ship-189`
- Path: P2, Venture Passage Works
- Role: protected or fast passenger/cargo launch
- Drive fit: `sF` maneuver and `sF` power
- Endurance/cargo: one week with 4.9 tons or two weeks with 4.0 tons, internal
- Hardpoint: closed flush armored disk
- Livery: sunflower upper barrel, cobalt lower fairing, signal-orange airlock
  and cargo edge, protected bright-chrome rail and collars
- Path fit: paired sensor cheeks, clipped protective flank rails, and stronger
  aft fairing edges applied without changing the family barrel

## Variant delta: `ship-8` Argus

- Path: P2, Venture Passage Works
- Role: armed passenger launch
- Drive fit: `sF` maneuver and `sG` power
- Electronics: basic military, model-2 computer, Evade/1, Fire Control/1
- Cargo: 1.5 tons
- Weapon: exactly one compact single turret in the canonical dorsal socket,
  carrying exactly one narrow optical beam-laser barrel
- Power delta: enlarge and rib the central stern power-service grille inside
  its fixed socket; preserve the twin field collars and complete stern outline
- Livery: Venture sunflower/cobalt/signal-orange scheme, with the turret and
  protected sensor cheeks integrated into the same bright frontier-commercial fit
- Prohibited: a second barrel, missile, exposed ordnance, naval-gray repaint,
  new hardpoint, moved window/door, or enlarged combat hull

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

## Production prompts

### Neutral anchor

```text
Use case: stylized-concept. Asset type: original Cepheus Trader ship-family
catalog anchor. Create the unpainted shared Caduceus chassis, a 20-displacement-
ton streamlined fast launch with canonical dimensions 17.5 m long by 6.4 m
wide by 4.6 m high and one occupied deck. Its pressure body is a horizontal
flattened cylinder: a long oval-section center barrel with rounded bow and stern
caps plus shallow aerodynamic fairings, explicitly not a wedge or lozenge.
Inset a cockpit with exactly three protected panes into the blunt forward cap.
On the visible port barrel show one chrome-ready passenger rail with exactly
five evenly spaced blank window sockets, then one rectangular airlock, then one
broad low flush cargo door. Add one narrow faired lower-nose scoop grid. Put one
closed circular armored hardpoint disk on the dorsal centerline behind the
occupied zone. Put exactly two recessed field-drive collars and one central
power-service grille in the aft cap. Show coherent four-point armor depth and
deep component reveals. Neutral warm-aluminum 1970s/1980s gouache-and-airbrush
technical concept plate. Complete front-port three-quarter view from 12 degrees
above centerline, restrained near-orthographic perspective, 3:2 landscape,
generous clearance, warm upper front-port key, weak cool rim, charcoal sparse
starfield with faint teal plotting grid. Preserve plausible 280 m³ volume and
human-scale doors. No paint, text, logo, watermark, open turret, weapon,
missile, jump ring, wing, tail, skid, other craft, planet, flame, motion blur,
wide-angle distortion, or franchise styling.
```

### Variant edit rule

```text
Edit the supplied approved Caduceus anchor. Preserve exactly its flattened-
cylinder silhouette and proportions, cockpit and three panes, five-socket
passenger rail, airlock, cargo door, lower scoop, dorsal hardpoint ring, aft
field collars, power-service socket, armored shell, primary seams, camera,
attitude, crop, backdrop, perspective, and lighting. Change only the explicitly
listed shipyard component housing, livery, hardpoint state, and power-grille
delta. Retain the 1970s/1980s gouache-and-airbrush catalog medium. No text,
logo, watermark, additional weapon, jump hardware, wing, tail, skid, other
craft, planet, flame, motion blur, or hull redesign.
```

## Production sequence

1. Approve the neutral flattened-cylinder family anchor.
2. Produce the Concord unarmed plate and assign it to `ship-145/146`.
3. Produce the Venture unarmed plate and assign it to `ship-7/185/189`.
4. Produce Argus from the same anchor, using the approved Venture plate as the
   path-finish reference and changing only turret state and power grille.
5. Compare all three plates at equal size before publication.

## Asset inventory and provenance

The images were generated for Cepheus Trader on 2026-08-17 with OpenAI image
generation from this manifest. Both unarmed plates were constrained edits of
the approved anchor. Argus used the anchor for geometry and the approved
Venture Caduceus for path finish, changing only the hardpoint and power grille.

| Asset | Purpose | Review status |
| --- | --- | --- |
| `anchors/family-007-caduceus.png` | Neutral shared flattened-cylinder anchor | Approved family geometry and closed hardpoint |
| `masters/family-007-caduceus-concord.png` | Full-resolution P1 unarmed master | Approved for `ship-145/146` |
| `masters/family-007-caduceus-venture.png` | Full-resolution P2 unarmed master | Approved for `ship-7/185/189` |
| `masters/ship-008-argus.png` | Full-resolution P2 armed master | Approved single beam-laser turret fit |
| `../assets/ships/family-007-caduceus-concord.webp` | Derived P1 website plate | Approved for publication |
| `../assets/ships/family-007-caduceus-venture.webp` | Derived P2 website plate | Approved for publication |
| `../assets/ships/ship-008-argus.webp` | Derived Argus website plate | Approved for publication |

The artwork is authored for and distributed as part of Cepheus Trader and is
Open Game Content under `LICENSE.md` and the Open Game License version 1.0a in
`OPEN_GAME_LICENSE.md`.
