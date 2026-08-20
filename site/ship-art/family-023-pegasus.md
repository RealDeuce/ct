# Pegasus Visual Manifest

*Status: catalog production family 023*

## Mechanical identity

- Family/member: `family-23`; `ship-23` Pegasus
- Shipyard path: Path 4, Civic Survey Works
- Hull: 50 displacement tons / approximately 700 m³
- Dimensions: 23 m long, 10.5 m wide, and 5.6 m high
- Configuration: streamlined; technology level 11; non-standard design
- Protection: two whole points of small-craft crystaliron armor
- Drives: maneuver sK and power sK / thrust 6; no Jump drive
- Endurance: one week of power-plant fuel
- Control and complement: two-person cockpit, two operators, and twenty-four
  passenger acceleration seats
- Mission support: one airlock, one small-craft fuel processor, and 22.05 tons
  baggage/freight cargo
- Armament: none; no ammunition

Pegasus is a rapid civil-service passenger and freight boat. Twenty-four seated
travelers and a large hold let it move inspection teams, relief personnel,
baggage, and accompanying cargo, while fuel processing supports local refueling.
It still depends on a parent ship or nearby facilities for protection,
maintenance, and all interstellar movement.

## Canonical manta coach

The 23 × 10.5 × 5.6 m streamlined hull is a broad short “manta coach”: blunt
rounded center nose, thick filled swept shoulders, raised arched passenger
arcade, deep central/lower cargo keel, gently pinched waist, and compact broad
stern. The shoulders are occupied integral pressure volume rather than thin
wings. The filled aerodynamic envelope plausibly encloses approximately 700 m³.

Pegasus is mechanically identical to Proteus Passenger (`ship-21`), but family
assignment forbids Proteus geometry. It has no separate forward control block,
rectangular thirty-ton module shell, four clamp rails, module end frames, or
discrete aft block. “Modular” describes reconfigurable internal passenger/cargo
partitions and their service rails; it does not create an exposed or detachable
pod.

Exactly four dark panes form the protected two-person command brow. Exactly six
large dark window bays line the visible port arcade, each grouping internal
seat rows; warm-brown seats may be visible, but no passenger appears. One closed
human-scale pressure airlock sits below the forward arcade. A separate large
closed rectangular shutter on the lower aft-port keel serves the 22.05-ton
hold. One continuous chrome-edged band expresses the two-point armor shell.

The small horizontal dark-teal grille immediately below the forward arcade is
a passenger-cabin environmental exchanger. It is not the fuel processor. The
one installed processor is the separate lower square lime grille beside one
flush closed circular refueling cap. The cap has no projecting plug, barrel,
lens, or weapon function. Handrails, partition-service seams, landing doors,
and maintenance panels remain human-scale.

## Unarmed exterior

Pegasus installs no weapon mount, ammunition, or exteriorized fire-control
station. The small-craft construction scope's potential hardpoint does not
license a visible turret ring when the fitted record supplies none. Never add a
covered circular hardpoint, barrel, emitter, missile shutter, point-defense
node, or gun-like sensor. The round refueling cap and stern field apertures are
service/machinery components and remain visually distinct.

## Drives and Civic Survey recognition

Four recessed maneuver apertures form port/starboard pairs at the broad stern;
the plate shows the two port-side apertures while the far pair is occluded. One
separate central ribbed power-service grille supports the six-gravity sK/sK
plant. There is one processor but no fuel scoop. There is no Jump radiator,
coil housing, ring, external tank, flame, or glow.

Civic Survey Works uses warm white `#F4F0D8`, saturated cyan `#25A9B8`, lime
`#A8C83E`, polished chrome/aluminum, and dark-teal recesses. White organizes the
coach hull; cyan frames the passenger arcade, airlock, cargo shutter, cargo-
keel datum, and stern service panels; lime identifies the processor, refueling
cap, safety latches, partition status, and landing marks; chrome protects armor,
glazing, access, handrail, fuel, landing, and machinery interfaces. The bright
public-service finish makes passengers and cargo operations legible rather than
turning the boat into a liner advertisement.

## Invariants and production prompts

Preserve the 23 × 10.5 × 5.6 m manta-coach silhouette; four-pane command brow;
exact six-bay passenger arcade; seats; distinct airlock and cargo shutter;
dark-teal cabin exchanger; one lower lime processor grille and flush refueling
cap; two-visible/two-hidden field apertures and central power grille; armor
band; landing/service details; absent weapons and modules; camera; crop;
backdrop; and lighting. Keep all people and cargo internal and every door shut.

```text
Neutral anchor: create the unpainted 23 × 10.5 × 5.6 m, 50-ton streamlined
Pegasus manta coach with blunt center nose, thick occupied shoulders, raised
passenger arcade, deep cargo keel, pinched waist, broad stern, exactly four
cockpit panes, exactly six visible passenger-window bays, one closed airlock,
one separate closed cargo shutter, one processor grille and flush refueling cap,
four maneuver apertures, one power grille, and one armor edge band. No weapon,
hardpoint ring, scoop, Jump equipment, external module, paint, text, or glow.

Pegasus fit: preserve every anchor coordinate and keep the craft unarmed. Apply
Civic Survey warm white, cyan, lime, chrome, and dark teal. Mark only the lower
square grille as the fuel processor; render the upper horizontal grille as a
dark-teal cabin exchanger. Keep all four cockpit panes, six passenger bays,
seats, doors, and the fuel cap unchanged. No turret, weapon socket, exposed
person, fuel scoop, Jump equipment, text, slogan, flame, or glow.
```

## Production sequence and provenance

1. Generate the neutral coach hull with fixed glazing, access, cargo, fuel,
   armor, and machinery coordinates.
2. Remove an unnecessary mast/dome, flatten the projecting fuel plug, and
   establish the clean roof/service map.
3. Repartition the cockpit alone to exactly four panes while preserving the six
   passenger bays and every other coordinate.
4. Derive the Civic production fit from the corrected anchor, using Galileo
   only as a finish reference and adding no weapon or module.
5. Recolor only the upper cabin exchanger dark teal so the lower lime fuel-
   processor grille remains the sole processor identifier.

Artwork was generated for Cepheus Trader on 2026-08-20 with OpenAI image
generation in built-in mode. The production plate descends from the corrected
neutral anchor; Galileo supplied finish language only.

| Asset | Purpose | Review status |
| --- | --- | --- |
| `site/ship-art/anchors/family-023-pegasus.png` | Corrected neutral Pegasus manta-coach chassis | Approved |
| `site/ship-art/masters/ship-023-pegasus.png` | Corrected archival Pegasus master | Approved |
| `site/assets/ships/ship-023-pegasus.webp` | Published Pegasus plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and
`OPEN_GAME_LICENSE.md`.
