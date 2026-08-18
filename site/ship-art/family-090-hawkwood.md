# Hawkwood Family Visual Manifest

*Status: catalog production family 090*

## Mechanical identity

- Members: `ship-90` Hawkwood and `ship-96` Condottiere
- Paths: Marque Marine (P5) and Redoubt Shipbuilding (P8)
- Hull: 550 displacement tons / approximately 7,700 m³; streamlined
- Dimensions: 64.0 m long, 22.0 m wide, and 14.0 m high
- Shared: TL11, advanced electronics, hardened bridge, four triple turrets,
  five point-defense lasers, one 20-ton full hangar, and one carried Caduceus
- Hawkwood: Jump drive F / Jump-2, thrust 4, eight armor points, one
  particle-beam barbette, and 113.5 tons cargo
- Condottiere: no Jump drive, eleven armor points, one 51-ton meson-gun bay,
  and 182.875 tons cargo

The lineage is one patrol chassis and a system-defense conversion. It is not
two unrelated frigates. Removing the Jump installation releases internal
volume and service margin for Condottiere's larger weapon, heavier armor, and
cargo, but does not license a new pressure hull.

## Canonical architecture

The family is a 64.0 × 22.0 × 14.0 m long faceted clipper-citadel. A clear
six-pane armored command prow leads into a narrowed protected boarding waist,
broad integral shoulders, and a compact squared engineering block. It is a
multi-deck streamlined spacecraft without aircraft wings or tail surfaces.

Four common circular triple-turret centers remain visible in the catalog view:
two lie along the dorsal centerline and one occupies each upper shoulder. The
forward dorsal and port-shoulder positions carry beam-laser triples; the aft
dorsal position carries the missile triple; the starboard shoulder carries the
sandcaster triple. Five tiny point-defense centers cover prow, waist, dorsal,
aft, and underside arcs. Three are directly readable from the standard camera;
two are occluded.

One 11 m forward axial reservation contains a small five-ton barbette insert
inside a larger 51-ton bay boundary. Hawkwood activates only the compact
particle-beam insert. Condottiere removes it and activates the full boundary as
one deeply recessed meson-gun aperture. The outer prow and main structural
datum do not move.

One closed 7.2 × 5.3 m two-leaf full-hangar shutter occupies the aft-port
shoulder beneath its turret. An 18 m internal craft axis accepts the approved
17.5 × 6.4 × 4.6 m Caduceus. The launch is carried internally and therefore
does not appear outside either frigate. A separate port boarding airlock and
shielded docking collar sit forward of the hangar.

Hawkwood's internal Jump drive reads externally through a segmented aft
radiator/service strip and two flush coil-service blisters. It has no ring and
no glow. Condottiere covers those exact service locations with flush armor
plates. Four maneuver apertures and two power grilles remain inside one common
stern boundary.

### Invariants

Preserve exactly the dimensions and silhouette, six-pane bridge, waist,
shoulders, aft block, four turret centers, five point-defense centers, nested
axial weapon reservation, hangar frame, boarding airlock, docking collar,
Jump-service locations, stern boundaries, landing doors, primary seams,
camera, crop, backdrop, and lighting. Never add a wing, tail, external tank,
Jump ring, second bay, second hangar, externally carried craft, or unrecorded
weapon.

## Production fits

### Hawkwood — record 90

- Two beam-laser triple turrets, one missile-rack triple, one sandcaster triple
- One compact particle-beam barbette inside the forward axial reservation
- Five point-defense nodes; three visible and two occluded
- Active segmented Jump-service strip and two flush coil blisters
- Eight-point armor reveals; closed hangar with internal Caduceus
- Marque finish: emerald `#13705B`, cream `#EFE0B5`, burgundy `#8D2E3C`,
  gold-toned chrome, and bright steel edge ribs

### Condottiere — record 96

- Exact same four triple turrets and five point-defense centers
- Full 51-ton meson-gun bay active in the inherited axial reservation; compact
  particle barbette absent
- Jump-service strip and coil locations blanked by flush armor
- Eleven-point overlapping faceted armor and deeper common turret wells, kept
  within the same silhouette; closed hangar with internal Caduceus
- Redoubt finish: fire red `#B83B2E`, orange `#E9782E`, bone `#D8CCA9`,
  heavy stainless edge guards, and restrained gunmetal recesses

## Catalog plate and reusable prompts

Use a complete front-port three-quarter view from about 12 degrees above with
restrained near-orthographic perspective, 3:2 landscape, warm upper-front-port
key, cool aft rim, charcoal stars, and a faint teal grid. Render as original
late-1970s/early-1980s gouache-and-airbrush technical art with saturated enamel
and bright metal. No text, logo, watermark, weapon fire, planet, modern gray
rendering, or recognizable franchise styling.

```text
Neutral anchor: create a 64 x 22 x 14 m streamlined 550-ton clipper-citadel
with a six-pane command prow, protected waist, broad shoulders, and compact aft
block. Reserve four closed triple-turret rings, five point-defense covers, one
11 m axial weapon zone with a small barbette insert nested in a 51-ton bay
boundary, one closed 7.2 x 5.3 m aft-port hangar door for a 17.5 m Caduceus,
one boarding airlock and docking collar, one segmented Jump-service strip, two
flush coil blisters, fixed stern apertures, scoop, and landing doors.

Shared edit rule: preserve all anchor coordinates, silhouette, apertures,
hangar, weapon centers, camera, and lighting. Change only the selected axial
weapon state, Jump-service state, armor layer treatment, drive openings inside
their fixed boundaries, and shipyard finish.

Hawkwood: activate two triple beam turrets, one triple missile turret, one
triple sandcaster turret, one compact particle-beam barbette, and five point-
defense nodes. Keep Jump-service landmarks active, the hangar closed, and use
the Marque palette.

Condottiere: retain the exact four turrets and five point-defense nodes. Remove
the compact barbette and activate one large recessed meson-gun bay inside the
same axial boundary. Armor-blank all Jump-service fittings, deepen armor inside
the unchanged silhouette, keep the hangar closed, and use the Redoubt palette.
```

## Production sequence and review notes

1. Approve the neutral hull with all four turret rings and the fixed hangar.
2. Produce Hawkwood as the complete Jump-capable patrol fit.
3. Correct the missile and sandcaster housings to exactly three apertures each.
4. Derive Condottiere directly from the corrected Hawkwood plate, retaining
   every turret while changing only the conversion equipment and path finish.

The first neutral pass made the hangar too quiet; the approved anchor adds the
required shutter without changing the shoulder. The first Hawkwood pass showed
four missile apertures and two sandcaster projectors; the approved master has
three of each. Condottiere was derived only after that correction.

## Asset inventory and provenance

Generated for Cepheus Trader on 2026-08-18 with OpenAI image generation.
Condottiere derives from the approved Hawkwood master so its inherited
silhouette, hangar, and four turret centers are genuinely shared.

| Asset | Purpose | Status |
| --- | --- | --- |
| `site/ship-art/anchors/family-090-hawkwood.png` | Neutral shared patrol chassis | Approved |
| `site/ship-art/masters/ship-090-hawkwood.png` | Jump patrol-frigate master | Approved |
| `site/ship-art/masters/ship-096-condottiere.png` | System-defense conversion master | Approved |
| `site/assets/ships/ship-090-hawkwood.webp` | Published Hawkwood plate | Approved |
| `site/assets/ships/ship-096-condottiere.webp` | Published Condottiere plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and the Open Game License
version 1.0a in `OPEN_GAME_LICENSE.md`.
