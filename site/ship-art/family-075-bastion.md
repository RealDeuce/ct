# Bastion Visual Manifest

*Status: catalog production singleton family 075*

## Mechanical identity

- Record: `ship-75` Bastion
- Path: Redoubt Shipbuilding (P8)
- Hull: 300 displacement tons / approximately 4,200 m³; streamlined
- Dimensions: 42.0 m long, 24.0 m wide, and 16.0 m high
- Fit: TL11, eight armor points, one reinforced-structure increment, maneuver
  J, power J, no Jump drive, two weeks endurance, and advanced electronics
- Weapons: one 51-ton meson-gun bay, one beam/beam/sand triple turret, one
  missile/missile/sand triple turret, and three point-defense lasers
- Support: fifteen crew, armory, office, one medical bed, repair drones, two
  fuel processors, 55.5 tons cargo, and one 10-ton full hangar containing Charon

Bastion is an independent system-defense-carrier family. It must not reuse
Aegis's shield-vault wedge, Monitor's standard casemate ram, Hawkwood's clipper-
citadel, or another Redoubt hull. The shared yard language appears in armor,
component construction, and finish rather than common chassis geometry.

## Canonical architecture

The hull is a 42.0 × 24.0 × 16.0 m “launch-breakwater citadel”: a short blunt
faceted shield prow, tall five-pane buried command bunker, two thick occupied
shoulders, deep central armored keel, protected port launch bunker, and broad
short recessed-drive stern. The shoulders and keel are filled multi-deck
spacecraft volumes, not wings, fins, tanks, or fighter surfaces.

One closed 6.2 × 4.6 m two-leaf full-hangar shutter opens from the port launch
bunker into an approximately 14 m internal craft axis. It clears the approved
12.5 × 5.2 × 3.8 m Charon without exposing the launch in the catalog plate.
A separate closed 7.5 × 3.8 m cargo/patrol-store shutter serves the 55.5-ton
hold. One protected airlock, armory shutter, office/service panel, plain
emergency-berth panel, plain medical hatch, and repair-drone slot remain closed
and unmarked. One of two stateroom windows is visible; crew berthing is
windowless.

## Meson architecture and coverage map

The three hardpoint-consuming installations occupy complementary arcs:

1. One visible axial lower-forward 51-ton meson-gun bay is integrated into the
   keel.
2. One visible upper-port-shoulder triple turret carries two beam optics and
   one blunt four-port sandcaster.
3. One hidden lower-aft-starboard triple turret carries two missile racks and
   one sandcaster.

The 11 × 5.5 m meson channel uses thick longitudinal load paths, a deep feed
trunk, overlapping radiation baffles, cooling panels, and insulated annular
field-coil collars. Exactly one recessed dark emitter terminus lies inside the
forward collar. It is hull architecture rather than a projecting gun barrel;
there is no second aperture, beam, glow, flame, or exposed magazine.

Three tiny point-defense centers cover forward upper-port shield cheek, lower-
aft port keel, and upper-mid starboard shoulder. The first two are visible and
the third is occluded. Other blue-black fittings are sensors, docking guides,
or cameras and must not become weapons.

## Drive, fuel, armor, and structure

One of two processor grilles is visible and one is hidden starboard. Bastion
has no recorded scoop. It has no Jump drive: never add a radiator/service band,
coil blister, ring, hoop, external tank, portal, or glow. Four deep port
maneuver apertures and four hidden starboard apertures flank two ribbed power
grilles within the stern.

Eight-point armor and reinforced structure read through three layered prow
shields, overlapping shoulder plates, heavy edge guards, deep opening reveals,
redundant keel ribs, and protected machinery. These are integral load paths,
not detachable armor pods.

## Redoubt finish and catalog plate

- Fire red `#B83B2E`: principal citadel, shield, and stern surfaces
- Orange `#E9782E`: meson baffles, weapon collars, recovery, processor, and
  selected drive accents
- Bone `#D8CCA9`: command surround, hangar and cargo shutters, access panels,
  and neutron-shadow plates
- Bright stainless steel: armor edges, meson load paths, opening frames, and
  service interfaces
- Gunmetal/dark teal: recesses, grilles, glazing, apertures, and seams

Use a complete front-port three-quarter view about 12 degrees above with
restrained near-orthographic perspective, 3:2 landscape, warm upper-front-port
key, cool aft rim, charcoal stars, and a faint teal grid. Render as original
late-1970s/early-1980s gouache-and-airbrush technical art. No text, logo,
watermark, open door, weapon fire, planet, modern CGI, or franchise styling.

## Invariants and reusable prompts

```text
Neutral anchor: create one independent 42 x 24 x 16 m streamlined 300-ton
launch-breakwater citadel with blunt shield prow, exactly five bridge panes,
thick occupied shoulders, deep keel, port launch bunker, and short broad stern.
Add one closed 6.2 x 4.6 m Charon hangar, one separate closed 7.5 x 3.8 m cargo
shutter, one visible/one hidden stateroom position, access and service panels,
one 11 x 5.5 m axial lower-forward meson channel, one visible upper-port flank
turret cover, one hidden lower-aft-starboard cover, and three PD covers with two
visible. Express eight armor and reinforced structure. Show one visible/one
hidden processor, no scoop, no Jump hardware, and recessed J/J stern geometry.
Neutral aluminum and bone primer, period gouache and airbrush, front-port view.

Bastion fit: preserve every anchor coordinate. Activate exactly one recessed
meson emitter inside the existing annular field coils. Fit the visible port-
shoulder cup with exactly two beam optics and one four-port sandcaster; keep the
missile/missile/sand cup hidden starboard. Activate only the two visible PD
lasers. Keep Charon internal and every door closed. Apply Redoubt fire red,
orange, bone, stainless, and gunmetal. Add no extra weapon or Jump landmark.
```

## Production sequence and provenance

Generated for Cepheus Trader on 2026-08-20 with OpenAI image generation in
built-in mode. The neutral pass rendered six bridge panes; one scoped correction
reduced the brow to five. The first production pass made a third lower-prow
fitting read as an active point-defense optic; a second scoped correction
blanked that fitting while retaining the intended two visible nodes. The final
plate derives from the corrected anchor and uses Aegis only as a Redoubt finish
reference.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-075-bastion.png` | Corrected neutral singleton anchor | Approved |
| `site/ship-art/masters/ship-075-bastion.png` | Full-resolution production master | Approved |
| `site/assets/ships/ship-075-bastion.webp` | Published catalog plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
