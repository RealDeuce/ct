# Argo Visual Manifest

*Status: catalog production family 010*

## Mechanical identity

- Family/member: `family-10`; `ship-10` Argo
- Shipyard path: Path 4, Civic Survey Works
- Hull: 20 displacement tons / approximately 280 m³
- Dimensions: 16.5 m long, 7 m wide, and 4.8 m high
- Configuration: streamlined; technology level 11; non-standard design
- Protection: one crystaliron layer / four armor points
- Drives: maneuver sF and power sG / thrust 6; no Jump drive
- Endurance: two weeks of power-plant fuel
- Control and complement: one-person cockpit, one pilot, and ten passenger
  acceleration seats
- Mission support: one airlock and 2.5 tons cargo
- Armament: one trainable single beam-laser turret; the sole hardpoint is
  occupied
- Parent facilities: Aquarius (`ship-84`) carries one Argo in a dedicated
  20-ton standard hangar; Tsushima (`ship-114`) carries one Argo beside a
  30-ton Wayfarer Cargo (`ship-187`) in one 50-ton full hangar

Argo is a high-thrust civil-service boarding and rescue launch. One pilot can
deliver ten seated personnel through atmosphere or contested local space, but
the launch has no stateroom, Jump drive, fuel processor, or independent support
plant. Its Civic Survey origin remains visible even when an Admiralty tanker or
cruiser carries it.

## Canonical shield launch

The 16.5 × 7 × 4.8 m streamlined hull is a compact shovel-nose lifting wedge
with a broad shield-shaped planform: clipped armored prow, shallow raised
command brow, deep central troop cabin, chamfered side shoulders, tapered
belly, and very short square engineering stern. The tapered envelope plausibly
contains approximately 280 m³ while fitting the two parent hangar allocations.

This independent chassis must not become the 17.5 m Caduceus/Argus family's
flattened cylinder merely because Argo and Argus share displacement, thrust,
seating, armor, and armament. Argo has no barrel pressure body, passenger-window
rail, rounded end caps, or Caduceus service geometry.

Exactly three dark panes form the protected one-pilot control brow. The ten
acceleration seats and occupants remain internal and windowless. One closed
human-scale pressure airlock lies on the forward-to-mid port flank; one separate
small closed cargo shutter lies aft and lower. Chrome datum edges, handholds,
landing-door seams, and maintenance panels remain human-scale. One continuous
edge band expresses the four-point crystaliron shell without turning the launch
into a layered capital citadel.

## Single hardpoint and coverage

One 3–4 m circular ring occupies the dorsal centerline behind the cockpit. Its
low trainable cup carries exactly one short narrow dark-teal/cyan beam optic in
a forward-facing recess. The ring and one-emitter cup are the design's only
weapon coordinate; the standard plate exposes it completely.

The centerline position provides the single available mount with useful
forward, port, starboard, dorsal, and much of the aft field. The hull still has
a ventral blind region because one hardpoint cannot provide complete spherical
coverage. Do not invent a lower partner, paired barrel, point-defense lens,
missile shutter, or gun-like sensor to conceal that limitation.

## Drives and Civic Survey recognition

Four compact maneuver apertures form two port/starboard pairs around a separate
ribbed power grille. The port three-quarter plate shows two apertures; the other
two are occluded on the far half. The enlarged grille communicates the sG power
fit and six-gravity performance without moving the drive pairs or adding flame.

Argo has no Jump drive, scoop, or processor. It therefore has no Jump radiator,
coil housing, field vane, ring, intake grid, processor grille, or external tank.
The armored nose remains deliberately plain.

Civic Survey Works uses warm white `#F4F0D8`, saturated cyan `#25A9B8`, lime
`#A8C83E`, polished chrome/aluminum, and dark-teal recesses. White organizes the
shield hull; cyan marks the shoulder datum, airlock, cargo boundary, stern
service panels, and hardpoint collar; lime identifies restrained safety latches,
service-status inserts, and the turret accent; chrome protects armor, access,
landing, weapon, and drive interfaces. Bright public-service identification
supports inspection, boarding, and rescue rather than disguising the armed fit.

## Invariants and production prompts

Preserve the 16.5 × 7 × 4.8 m shield-planform silhouette; three-pane brow;
plain armored nose; airlock and separate cargo shutter; one dorsal hardpoint
ring and single-emitter cup; two-visible/two-hidden drive-aperture map; separate
power grille; armor band; landing seams; camera; crop; backdrop; and lighting.
Keep all people internal and both doors closed.

```text
Neutral anchor: create the unpainted 16.5 × 7 × 4.8 m, 20-ton streamlined Argo
shield launch with shovel nose, clipped prow, raised one-pilot brow, deep ten-
seat cabin, chamfered shoulders, tapered belly, square stern, exactly three
cockpit panes, one closed port airlock, one separate small aft-lower cargo
shutter, one sealed dorsal turret ring, four maneuver apertures, one power
grille, and a continuous armor edge. No installed weapon, scoop, processor,
Jump equipment, open door, person, paint, text, or glow.

Argo fit: preserve every anchor coordinate. Replace only the ring's blanking
disk with one compact trainable cup carrying exactly one short dark-teal/cyan
beam optic. Apply Civic Survey warm white, cyan, lime, chrome, and dark teal.
Keep the nose plain, both doors closed, two far drive apertures hidden, and all
passengers internal. No second emitter, extra weapon, fuel or Jump equipment,
text, slogan, weapon fire, or drive glow.
```

## Production sequence and provenance

1. Generate the neutral shield-shaped launch with fixed access, hardpoint, and
   machinery coordinates.
2. Correct four cockpit panes to exactly three, close the deep hardpoint well
   with a flush disk, and remove the spurious intake-like nose grille without
   changing the approved hull.
3. Derive the production fit from the corrected anchor, using Galileo only as
   a Civic Survey finish reference and installing one single-optic cup.
4. Review the finished plate for family independence, one weapon, closed and
   separate doors, absent fuel/Jump gear, and parent-hangar compatibility.

Artwork was generated for Cepheus Trader on 2026-08-20 with OpenAI image
generation in built-in mode. The production plate descends directly from the
corrected neutral anchor; Galileo supplied finish language only.

| Asset | Purpose | Review status |
| --- | --- | --- |
| `site/ship-art/anchors/family-010-argo.png` | Corrected neutral Argo shield-launch chassis | Approved |
| `site/ship-art/masters/ship-010-argo.png` | Archival Argo catalog master | Approved |
| `site/assets/ships/ship-010-argo.webp` | Published Argo plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and
`OPEN_GAME_LICENSE.md`.
