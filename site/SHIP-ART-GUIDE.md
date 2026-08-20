# Ship Catalog Art Direction and Production Guide

*Catalog audit: 2026-08-17; active catalog revision 1*

## Purpose

This guide turns the mechanical ship catalog into a consistent original visual
language for player-facing exterior illustrations. It supplements
[`DESIGN.md`](DESIGN.md). The site-wide direction still applies: these are
useful artifacts from a far future imagined through 1970s and 1980s science
fiction art.

The intended deliverable for each admitted design is one exterior three-quarter
catalog view. The illustration should accurately reflect the ship's scale,
configuration, role, family, native shipyard path, and visible external
systems. It is original Cepheus Trader art, not a reconstruction of excluded
source artwork or trade dress.

The catalog determines what a ship contains and what it can do. It does not
determine exact dimensions, deck geometry, component placement, or appearance.
Those are canonical art decisions. Record them and reuse them; do not imply
that an invented silhouette was calculated uniquely from the rules.

## Sources of truth

Read the sources in this order for every illustration:

1. `catalog/ships/families.toml` determines the shared hull lineage.
2. The individual `catalog/ships/ship-N.toml` determines displacement,
   configuration, role, drives, armor, armament, hangars, carried craft, and
   other fitted systems.
3. `catalog/ships/upgrade-paths.toml` and `names.toml` determine the native
   shipyard path and its narrative identity.
4. The design's original `description_paragraphs` explain intended strengths,
   limitations, and use.
5. The construction tables under `catalog/shipbuilding/` determine component
   volume and relationships.
6. This guide supplies the shared visual grammar where the mechanical sources
   are silent.

Family is a stronger geometric relationship than shipyard path. Path is a
stronger design-language relationship than role. Apply them in this order:

> family chassis -> mechanical configuration -> shipyard fit -> mission fit ->
> individual livery details

Equal tonnage or a similar role never licenses reuse of another family's
silhouette. Conversely, members of one family must not be generated as
independent ships merely because their roles or paths differ.

## Audit summary

The active catalog contains:

| Property | Count |
| --- | ---: |
| Fitted designs | 213 |
| Design families | 113 |
| Shared multi-design lineages | 39 families / 139 designs |
| Singleton families | 74 |
| Families spanning more than one shipyard path | 21 |
| Small craft, 10-95 tons | 62 |
| Ships, 100 tons or more, without Jump | 17 |
| Jump-capable starships | 134 |
| Streamlined / standard / distributed / close-structure hulls | 126 / 75 / 10 / 2 |
| Designs with turret mounts | 164 |
| Designs with barbettes | 23 |
| Designs with weapon bays | 32 |
| Designs with point defense | 57 |
| Designs with hangars | 85 |
| Designs carrying catalogued craft | 82 |
| Designs with docking clamps | 21 |
| Designs with launch or recovery facilities | 4 |

The displacement range is 10 to 5,000 tons:

| Displacement band | Designs | Visual reading |
| --- | ---: | --- |
| 10-95 tons | 62 | Human-scale launches, fighters, cutters, and boats |
| 100-299 tons | 30 | Compact independent ships; doors, gear, and windows remain prominent |
| 300-599 tons | 59 | The most common merchant, patrol, and raider scale |
| 600-999 tons | 22 | Large working ships; hangars and major systems shape the hull |
| 1,000-1,999 tons | 25 | Multi-deck commands and serious combatants |
| 2,000-4,999 tons | 13 | Fleet logistics, carriers, and heavy combatants |
| 5,000 tons | 2 | Catalog capital ships; bays and small-craft operations dominate scale |

The art process must not normalize these into similarly sized vehicles inside
identically cropped frames. A 10-ton launch and a 5,000-ton dreadnought need
recognizably different doors, windows, weapon housings, surface subdivision,
and scale references.

## Physical scale

Cepheus Engine displacement is volume, not mass. One displacement ton is about
14 cubic meters. A 1.5 m by 1.5 m deck square with a 3 m deck height is half a
displacement ton. Use those relationships when establishing a family master.

For every new family, record a canonical length, beam, height, deck direction,
and approximate number of occupied decks. The simplified enclosed hull volumes
must plausibly contain `catalog tons × 14 m³`. Do not compare this value to the
entire bounding box as though tapered noses, gaps, wings, and external booms
were solid. For distributed and close-structure ships, sum the principal
pressure volumes instead.

Dimensions are an art decision until they are adopted into a visual manifest.
Once adopted, they are invariant for equal-displacement members of that family.
Do not let image generation quietly stretch or shrink a variant to make its
mission equipment easier to draw.

### Human and deck cues

Use the following nominal art scale consistently. These values are visual
standards, not new construction rules:

- occupied deck bands are approximately 3 m high;
- an ordinary personnel hatch is approximately 2.2-2.6 m high;
- an ordinary viewport is approximately 0.6-1.2 m across;
- railings, handholds, maintenance panels, landing gear, and docking collars
  remain human-scale at every hull size;
- passenger windows may form a rhythm but must not become building-sized glass
  walls on pressure hulls;
- hull plating and panel seams become finer relative to the whole as ships get
  larger.

Every catalog card should provide a scale device outside the generated art: a
dimension line, a 1.8 m crew silhouette, or a familiar carried craft. Do not
depend on a painted astronaut or incidental scenery that may change between
images.

## Standard catalog plate

Use one production composition so family and scale comparisons remain useful:

- landscape 3:2 master, normally 1536 by 1024 or a larger equivalent;
- front-port three-quarter view, looking slightly down from 10-15 degrees
  above the ship's centerline;
- a restrained near-orthographic perspective, avoiding a dramatic wide-angle
  nose and a vanishingly small stern;
- the complete silhouette visible with generous clearance;
- warm key light from upper front-port and a cooler, weaker rim/fill light;
- a dark neutral, sparse starfield, plotting-paper, or catalog-plate backdrop
  that does not establish a false scale;
- no generated labels, logos, watermarks, artist signatures, or decorative
  spacecraft;
- no weapon fire, open flame, battle damage, or motion blur in the canonical
  plate.

All members of one family use the same camera, attitude, lighting direction,
and crop. If a hidden starboard feature is important, record it in the catalog
data rather than flipping that one variant and losing family comparability.

The rendering medium should resemble a high-quality 1970s/1980s gouache,
airbrush, or painted technical plate: confident shapes, selective hard detail,
opaque color blocking, chrome highlights, and physical-media texture. It
should not resemble a modern gray 3D game render, photobashed franchise art,
or blue-purple synthwave neon.

## Hull configurations

Configuration is mechanical and may not be overridden by shipyard style.

### Streamlined

Use a coherent atmospheric envelope: lifting body, flattened cylinder,
plausible fuselage, or another family-specific aerodynamic form. Fuel scoops,
doors, sensors, weapons, and landing systems should be flush, faired, or
retractable where practical. Streamlining does not require a contemporary jet
or a featureless needle.

### Standard

Use an integrated but non-aerodynamic pressure hull: cylinders, slabs, drums,
faceted volumes, or blocky combinations with recognizable deck direction.
Doors and components can project. A standard hull may enter atmosphere badly;
it should not look as aerodynamically clean as a streamlined sibling.

### Distributed

Use separated pressure volumes, tanks, booms, or working modules joined by a
clear structural frame. Do not hide a distributed ship inside one smooth outer
shell. It has no fuel scoops and is a poor atmospheric craft. The silhouette
should communicate access, repair, and replaceable modules.

### Close structure

Use a compact cluster of adjacent major volumes tied into one dense load path.
It is not a smooth standard hull and not an open distributed truss. It may
operate in a thin atmosphere, but clustered masses and reinforced connections
should remain visible.

## Shared component bible

Component recognition is global. A shipyard may change the housing, edge
treatment, paint, and supporting geometry, but a player should still recognize
the component type.

### Bridges and control spaces

- Keep the bridge in the same location and at the same scale across a family.
- Use limited ports, armored shutters, optical blisters, or sensor-fed control
  spaces rather than a mandatory panoramic glass cockpit.
- Command bridges may have a larger observation/control volume and redundant
  sensor clusters; hardened bridges should be buried or visibly protected.
- A bridge is not automatically at the bow. Family layout governs.

### Drives, fuel, and heat

- Maneuver-drive scale should read through the size and number of family-
  consistent drive nodes, exhaust apertures, or field housings. More thrust
  may enlarge or multiply those features without relocating the whole drive
  block.
- Jump drives are chiefly internal. Show their presence through a stable
  family/path convention such as coil housings, field vanes, radiator bands,
  or service bulges; do not add arbitrary glowing rings.
- Large fuel allocations make the hull bulkier or create tank volumes. They do
  not require exposed spherical tanks unless the family/configuration calls for
  them.
- Fuel scoops are flush intake or field-grid regions on streamlined ships and
  explicit intake structures on suitable standard ships. Distributed and
  close-structure configurations cannot install them under the current rules.

### Turrets

Single, double, triple, and quad turrets each consume one displacement ton in
the rules. Use one common 3-4 m mounting ring and an approximately 1.5-2.5 m
external housing. The ring and basic housing remain the same size; the number
and type of emitters changes.

The exact number of mounts must be allocated in the family art manifest before
generation. Place mounts for useful coverage, preserve those placements across
variants, and explicitly list which are visible in the standard port view.
Hidden mounts still exist and remain in the adjacent catalog statistics.

The standard three-quarter view normally exposes only about half of a ship's
turrets. For example, a broadly symmetric eighteen-mount battery should show
roughly nine mounts, not all eighteen. Front/aft, dorsal/ventral, or role-driven
asymmetry may move that number modestly, but every deviation must follow the
actual station map and camera rather than a desire to display the full rules
inventory. State visible and hidden counts separately in the manifest.

Global weapon recognition:

- **Beam laser:** a narrow optical barrel or recessed lens with a clean collar.
- **Pulse laser:** a shorter, heavier, ribbed or paired-pulse emitter.
- **Particle beam:** a heavier insulated barrel with a substantial field collar.
- **Missile rack:** shuttered box, drum, or cell cluster; do not hang exposed
  contemporary missiles on a spacecraft wing.
- **Sandcaster:** blunt multi-port projector or canister launcher, visually
  distinct from a missile rack.
- **Railgun:** long parallel rails, bracing, and a narrow muzzle gap.
- **Plasma gun:** broad heat-shielded muzzle with cooling structure.
- **Mining laser:** rugged short-range industrial optic with work-light and
  manipulator context where fitted.

A mixed turret uses the same ring with visibly different apertures. Do not turn
three unlike weapons into three identical gun barrels.

### Barbettes

A barbette consumes five displacement tons, about 70 m³. It is a semi-recessed
multi-deck installation, nominally 6-10 m long and 3-5 m wide, visibly larger
and more deeply rooted than a turret. Retractable barbettes need fairing volume
and a closed position. Preserve barbette socket locations within a family even
when the weapon changes.

### Weapon bays

Weapon bays are hull architecture, not oversized turrets. The catalogued
installation includes one displacement ton of fire control above the nominal
weapon size:

- a nominal 50-ton bay occupies 51 displacement tons, about 714 m³;
- a nominal 100-ton bay occupies 101 displacement tons, about 1,414 m³;
- a nominal 500-ton bay occupies 501 displacement tons, about 7,014 m³.

Represent them as multi-deck apertures, armored shutters, longitudinal emitter
channels, or large launch complexes integrated into the primary structure.
Their openings, recoil/load paths, and magazines should visibly influence the
hull. A capital ship with many bays needs repeated architectural batteries,
not a handful of decorative bumps.

### Point defense

Point-defense nodes consume no displacement but require a firing arc. Use
small 0.4-0.8 m optical or gun fixtures distributed around the hull. The
catalog contains up to 50 nodes on one ship. Establish exact positions in the
manifest, but do not make every node a major turret. At catalog-image size they
may read as a repeated ring, blister, or bright lens pattern.

Laser, minigun, and gatling-laser nodes require distinct small silhouettes.
Never rely on image generation to count a large field of nodes from prose;
validate or add the smallest repeated details in a controlled finishing pass.

### Hangars, launch facilities, and carried craft

- A standard hangar allocates 110 percent of contained-craft volume; a full
  hangar allocates 130 percent. The door must be sized for the actual craft,
  not inferred only from the parent ship's role.
- Reuse the carried craft's canonical beam and height when sizing the door,
  lift, recovery slot, or launch tube.
- Full hangars may show service aprons and wider clearance; standard hangars
  should look tighter and workmanlike.
- Flight and recovery decks are dominant exterior structures. They cannot be
  represented by a small generic hatch.
- Docking clamps are external load-bearing machinery. Distinguish the 30-,
  90-, 300-, and 2,000-ton capacities through reach, collar size, and bracing.
- A carried craft does not need to be flying beside the parent in the catalog
  plate. Prefer a closed or partly visible correctly scaled bay.

Generate and approve carried craft before their parent ships. Eighty-two
designs have carried-craft records, so this dependency is part of the normal
production order, not an exception.

### Cargo and working systems

- Cargo volume affects broad door area, loading paths, modular sections, and
  the ratio of plain working hull to occupied windows. It does not mean every
  cargo ton is an external container.
- Tankers and replenishment ships need protected hose, boom, coupling, or
  transfer stations.
- Mining and salvage ships may expose grapples, drones, work lights, crushers,
  or module interfaces when the fitted record supports them.
- Survey and research fits should visibly privilege sensors, antenna fields,
  probe handling, laboratories, and clear observation geometry.
- Medical and passenger fits need clean docking access and recognizable
  occupied zones, not a cruise-liner glass wall pasted onto a freighter.

### Armor, stealth, and finish

- Increasing armor should reduce fragile projections, deepen component
  recesses, strengthen edge radii, and add coherent protective layers. Avoid
  arbitrary modern tank plates on every naval ship.
- Stealth treatment suppresses specular reflections and breaks up exposed
  sensor/thermal features. It may retain strong matte color blocking, but it
  should not use broad mirror chrome.
- Chrome, polished aluminum, bright anodized trim, and glossy enamel are normal
  period-future materials elsewhere. Use them deliberately on leading edges,
  collars, frames, radiator surrounds, and civil trim.
- Weathering follows work: exhaust staining near maneuver systems, ablation at
  atmospheric edges, handling wear around cargo and hangar doors, and patching
  on frontier craft. Do not cover every ship in brown grime.

## Shipyard-path design languages

A native upgrade path is a presumed manufacturer or shipyard doctrine and must
be recognizable before the viewer reads a label. It governs silhouette rhythm,
component housing, construction detail, surface finish, and livery. It does not
erase family geometry or mechanical configuration.

Bright opaque paint and chrome are defaults, not rare civilian exceptions.
Military ships may use disciplined high-contrast color blocking rather than
universal gray. These palettes are canonical starting points; local BBS liveries
may add markings without replacing the underlying yard language.

### 1. Concord Exchange Yards — orderly trade

- **Audit:** 60 designs in 30 families.
- **Form:** balanced modular volumes, long horizontal cargo/occupancy bands,
  clean standardized door spacing, and obvious service access.
- **Components:** rectangular chrome collars, flush commercial sensor strips,
  neatly faired drives, and interchangeable cargo modules.
- **Palette:** warm ivory `#F1E3C2`, vermilion `#E44B2D`, royal blue
  `#2354A3`, with polished aluminum/chrome.
- **Character:** prosperous scheduled transport—bright, maintained, legible,
  and designed to turn around quickly at a port.

### 2. Venture Passage Works — protected frontier commerce

- **Audit:** 20 designs in 13 families.
- **Form:** forward-driving wedges or clipped lozenges wrapped around a clear
  family core, with protected flanks and strong maneuver sections.
- **Components:** armored commercial turrets, recessed cargo doors, paired
  sensor cheeks, and chrome edge rails protected behind the leading surfaces.
- **Palette:** sunflower `#F2C230`, cobalt `#174A8B`, signal orange
  `#E96D2F`, with bright chrome accents.
- **Character:** fast, expensive, and prepared to cross an unreliable frontier
  without looking like a naval vessel.

### 3. Outer Reach Cooperative — austere frontier industry

- **Audit:** 14 designs in 10 families.
- **Form:** cylindrical tanks, work pods, external frames, replaceable bays,
  and plainly accessible machinery; streamlined members retain rugged fairings.
- **Components:** standardized field-repair panels, booms, grapples, heavy door
  frames, exposed service conduits, and dull aluminum pressure vessels.
- **Palette:** avocado `#7A9238`, ochre `#C9902C`, brick red `#A63F2F`,
  with brushed or patch-polished aluminum.
- **Character:** cooperative equipment kept useful far from a dependable yard;
  colorful without being precious.

### 4. Civic Survey Works — science and public service

- **Audit:** 19 designs in 16 families.
- **Form:** calm symmetric primary volumes interrupted by purposeful sensor
  blisters, observation bays, probe ports, and clean docking interfaces.
- **Components:** circular instrument collars, white ceramic radomes, fine
  antenna grids, teal optical glass, and chrome laboratory/service frames.
- **Palette:** warm white `#F4F0D8`, cyan `#25A9B8`, lime `#A8C83E`, with
  clean chrome.
- **Character:** visible public authority, rescue, medicine, and measurement;
  precise rather than militarized.

### 5. Marque Marine Yards — licensed force and escort

- **Audit:** 17 designs in 11 families.
- **Form:** long clipper-like thrust lines, a protected boarding waist, clear
  command prow, and weapon coverage that does not sacrifice cargo utility.
- **Components:** brass/chrome turret collars, shielded docking gear, robust
  boat bays, and decorative but functional edge ribs.
- **Palette:** emerald `#13705B`, cream `#EFE0B5`, burgundy `#8D2E3C`,
  with gold-toned chrome and bright steel.
- **Character:** a commissioned armed merchant—handsome, dangerous, and built
  to display legitimate authority.

### 6. Rogue Tide Yards — covert transport and raiding

- **Audit:** 10 designs in 9 families.
- **Form:** deceptive cargo-like masses, asymmetric service structures,
  concealed weapon recesses, false seams, and sudden high-thrust geometry.
- **Components:** shuttered sensor/weapon apertures, grappling and boarding
  machinery, dark-chrome collars, and deliberately mismatched replaceable
  panels that still follow a recognizable yard pattern.
- **Palette:** aubergine `#4D294F`, hot vermilion `#D94B35`, mustard
  `#D4A62A`, with blackened chrome.
- **Character:** theatrical period color at the dock, ambiguous purpose at a
  distance, and equipment intended to reveal itself only when used.

### 7. Admiralty Line Works — regular fleet service

- **Audit:** 43 designs in 34 families.
- **Form:** axial order, repeated frames, strong centerline structure, clearly
  separated command/engineering batteries, and disciplined weapon arcs.
- **Components:** standardized circular turret rings, ribbed bay shutters,
  paired field housings, formal boat bays, and bright metal datum lines.
- **Palette:** deep navy `#203B69`, warm white `#E8E5D4`, signal red
  `#C43C35`, with bright aluminum.
- **Character:** mass-produced fleet authority. Color is formal identification,
  not camouflage or decoration.

### 8. Redoubt Shipbuilding — local defense and survivability

- **Audit:** 20 designs in 17 families.
- **Form:** compact citadels, broad shoulders, short protected load paths,
  recessed drives, and faceted armor surrounding a recognizable family core.
- **Components:** deep turret wells, overlapping bay shutters, redundant sensor
  blisters, armored landing gear, and heavy stainless edge guards.
- **Palette:** fire red `#B83B2E`, orange `#E9782E`, bone `#D8CCA9`, with
  stainless steel and restrained gunmetal.
- **Character:** a visible promise to hold position—squat, accessible to local
  repair crews, and difficult to disable.

### 9. Tempest Arsenal — ambush and saturation attack

- **Audit:** 10 designs in 6 families.
- **Form:** dart, spear, or predator-like thrust lines wrapped around repeated
  launch cells, deep magazines, and abrupt armored shutters.
- **Components:** gridded missile apertures, long barbette channels, sharply
  faired sensors, bright chrome launch collars, and heat-stained aft structures.
- **Palette:** ultramarine `#3046A6`, violet `#6B3F91`, warning yellow
  `#F2C84B`, with bright chrome.
- **Character:** saturated opaque color and controlled violence, not neon
  cyberpunk glow. The ship should look built around its first strike.

## Role overlays

Role modifies the family/path design without replacing either:

| Role group | Exterior evidence |
| --- | --- |
| Trader / freighter | Broad cargo access, simple loading route, docking aids, large quiet hull areas |
| Passenger / yacht | Regular occupied deck rhythm, clean docking entry, brighter polished trim, fewer visibly improvised repairs |
| Frontier / mining / salvage | Work lights, manipulators where fitted, drone or module access, rugged repairable panels |
| Courier / dispatch | Compact hull, strong drive emphasis, protected data/bridge area, little cargo architecture |
| Survey / research / medical | Sensor fields, probe or boat access, clear service markings, protected working spaces |
| Patrol / privateer | Balanced weapon arcs, boarding access, carried boat support, armor around command and drives |
| Raider / covert | Concealable apertures, prize-cargo or boarding access, misleading civil proportions where plausible |
| Naval combatant | Repeated standardized batteries, protected command spaces, formal hangar and damage-control geometry |
| Carrier | Launch/recovery path and correctly scaled craft doors dominate; carried-craft count affects operations visibly |
| Tanker / replenishment | Tank or freight volume, booms, hose/coupling stations, protected transfer control positions |

Do not add an exterior system merely because it is typical of a role. The
individual fitted record remains authoritative.

## Family-first production

### Mandatory family anchor

Before producing final plates for a shared family, create and approve one
unpainted family anchor showing:

- exact canonical proportions and dimensions;
- bridge, drive block, deck direction, landing/docking geometry, and primary
  seams;
- hardpoint sockets, bay reservations, hangar zones, and module boundaries;
- a plain neutral material treatment so livery does not hide geometry;
- the standard catalog camera and lighting.

This anchor is a reference image, not necessarily a public website asset. Each
variant must be generated or edited with the anchor supplied as an explicit
family reference. A textual instruction saying "same hull" is not sufficient.

For every variant, repeat the invariants:

> Preserve the exact family silhouette, proportions, bridge, deck bands, drive
> block, airlocks, landing/docking geometry, primary seams, camera, perspective,
> and lighting. Change only the listed configuration fairing, shipyard fit,
> mission equipment, weapons, hangars, and livery.

Generate every member of a shared family in the same production run while the
anchor and last approved variant remain available. Do not interleave unrelated
families.

### Audited shared-family batches

These 39 batches contain every multi-design lineage. `P` numbers are native
shipyard paths. Cross-path members keep the family chassis and receive the
appropriate yard fit and livery.

| Family | Required batch | Paths | Special handling |
| --- | --- | --- | --- |
| Daedalus (1) | 1, 2, 3, 4 | P1/P2/P3/P6 | Same 10-ton work-pod chassis; role equipment must not replace the pod |
| Charon (5) | 5, 158 | P1 | Source-equivalent transfer launches; exact exterior reuse is permitted |
| Aeolus (6) | 6, 176, 177 | P1 | One fast-launch platform; command/flag fits are edits |
| Caduceus (7) | 7, 8, 145, 146, 185, 189 | P1/P2 | Preserve the canonical flattened-cylinder chassis |
| Icarus (9) | 9, 11 | P7 | Successive interceptor blocks; evolve details, not the whole silhouette |
| Valkyrie (12) | 12, 15 | P7/P9 | Same aerospace fighter; path-specific weapon and livery fit |
| Argonaut (17) | 17, 19, 178, 179 | P5/P7 | Same aerodynamic 30-ton boat; transport/boarding access changes only |
| Wayfarer (18) | 18, 134, 165, 181, 187, 212 | P1/P2/P5 | Utility, cargo, armed, and boarding fits on one 30-ton boat |
| Proteus (20) | 20, 21, 24, 159, 188, 209 | P1/P3/P4 | `ship-209` is standard; retain module interface while removing streamlined fairing |
| Albatross (22) | 22, 151 | P1 | Same pinnace, differing fitted record; derive together |
| Poe (26) | 26, 33 | P2/P6 | Dispatch hull versus covert-service fit; preserve the 100-ton hull |
| Mercator (27) | 27, 28, 31 | P1 | Repeated light-trader platform; equipment differences only |
| Goliath (30) | 30, 32 | P7 | Source-equivalent assault landers; exact exterior reuse is permitted |
| Humboldt (34) | 34, 43 | P3 | Same prospecting hull; visually reconcile source-fit differences |
| Congreve (35) | 35, 36 | P9 | One 80-ton missile attack boat; retain launch-cell geometry |
| Polo (38) | 38, 39, 40 | P1/P3 | Same 200-ton trader; freight/passenger fit affects doors and occupied decks |
| Sinbad (45) | 45, 46 | P2 | Same free-trader hull; passenger fit is not a new silhouette |
| Trident (48) | 48, 55, 56 | P8 | One 95-ton attack-boat architecture with torpedo, particle, and sensor fits |
| Hanse (49) | 49, 50, 51, 52 | P1 | One 300-ton cargo hull; generations retain main frame and door spacing |
| Verne (54) | 54, 57, 58, 59, 60, 62, 65, 161-164, 166-168 | P1/P2/P3/P5/P9 | Fourteen-member priority batch; lock the 300-ton chassis before any final art |
| Stevenson (68) | 68, 69, 72, 139-141 | P2/P5/P6 | One 400-ton platform across yacht, raider, and escort fits |
| Klondike (78) | 78, 79 | P3/P6 | Same distributed tender; mining and assault modules occupy shared frame sockets |
| Homeric (80) | 80, 82, 83, 86, 88, 152-157 | P1/P3/P5/P9 | Eleven-member modular freighter batch; module and door grid is invariant |
| Hawkwood (90) | 90, 96 | P5/P8 | Interstellar patrol hull and system-defense conversion |
| Cook (91) | 91, 95 | P7 | Configuration changes standard to streamlined; preserve internal chassis landmarks under fairing |
| Nightingale (93) | 93, 160 | P4 | Same hospital design represented twice; reuse if exterior fit validates |
| Baltic League (97) | 97, 98, 101 | P1/P7 | Same 2,000-ton logistics hull; tanker, freight, and fleet support plumbing differs |
| Corbett (99) | 99, 107 | P7 | Distributed versus close-structure versions; share volumes/components, not an identical outer silhouette |
| Duncan (100) | 100, 102 | P7 | Escort and command fit; command sensors/bridge treatment are an edit |
| Hephaestus (104) | 104, 105 | P8 | Same 1,400-ton carrier hull; docking-clamp capacity must visibly differ |
| Vauban (116) | 116, 118 | P7/P8 | Same streamlined cruiser platform with path-specific batteries |
| Aviator (117) | 117, 119, 120 | P7/P9 | Scaled 2,500/3,000/3,800-ton carrier lineage; retain cross-section and module grammar while lengthening/growing |
| Bulwark (121) | 121, 122 | P7/P8 | One 500-ton hull in interstellar escort and system-defense fits |
| Leviathan (126) | 126, 127 | P1 | Same 1,900-ton distributed freight frame; interstellar drive fit changes aft structure |
| Caravanserai (128) | 128, 129, 130 | P2/P7 | Same fast-trader hull in trade, replenishment, and assault roles |
| Bellamy (131) | 131, 132, 133 | P1 | Production/refit generations; preserve all major external landmarks |
| Roman (135) | 135-138 | P7/P8/P9 | One destroyer chassis with battle, torpedo, direct-fire, and missile batteries |
| Endeavour (147) | 147-150, 190 | P1/P4 | One 300-ton modular merchant platform; survey fit changes instruments, not hull |
| Franklin (169) | 169-175 | P1/P2/P3 | Two generations of one light-trader lineage; document which landmarks evolve between generations |

The 74 singleton families still require a family anchor. They receive their
native path language, but they must not borrow the silhouette of another
singleton merely because tonnage and role match.

### Cross-family reuse and duplicate records

An exact image may be reused only after comparing hull configuration, drives,
armor, external equipment, mount fit, hangars, carried craft, and path. A shared
display name is not enough. Several repeated names in the catalog have different
mechanics.

When two records are exterior-equivalent but belong to different paths, reuse
the geometry and component placement but apply the appropriate path fit and
livery. When records differ only internally, exact exterior reuse is desirable:
it tells the player that these are genuinely the same production design.

## Visual manifest

Before final image production, add or generate a visual brief for each family
and variant. It should contain at least:

```text
family anchor:
  family ID and name
  canonical length, beam, height, and deck direction
  configuration and pressure-volume breakdown
  bridge, drive, airlock, landing/docking, module, and primary seam landmarks
  standard camera and visible side

variant delta:
  ship tag, name, native path, role, and description summary
  armor, stealth, and external finish
  every turret/barbette/bay/point-defense placement, including hidden mounts
  visible mounts in the standard view
  cargo, hangar, docking-clamp, launch, scoop, sensor, and working apertures
  carried-craft type and door-clearance dimensions
  family invariants and the exact allowed changes from the anchor
```

Keep this as a separate art-data layer. Do not duplicate or alter the
authoritative construction bill of materials.

## Generation workflow

1. Validate the mechanical catalog before beginning a batch.
2. Build the family/variant visual manifest from the current TOML records.
3. Generate or draw the unpainted family anchor and approve its volume, scale,
   configuration, and landmarks.
4. Generate the simplest fitted member from that anchor.
5. Produce the remaining members as reference-led edits or generations, one
   distinct asset at a time. Do not ask for multiple distinct ships as variants
   of one prompt.
6. Repeat all family invariants in every edit request.
7. Apply native path design language and livery after chassis preservation is
   established.
8. Correct one discrepancy per iteration: silhouette, component count,
   placement, color, or surface finish.
9. Validate the final plate against the manifest and individual ship record.
10. Save the approved family anchor and every final project asset in the
    workspace; do not rely on a transient generated-image location.

### Prompt skeleton: family anchor

```text
Use case: stylized-concept
Asset type: Cepheus Trader ship-family catalog anchor
Primary request: an original unpainted exterior design for <family>, a shared
  <displacement>-ton <configuration> hull family
Subject: <canonical dimensions, pressure volumes, bridge, drives, doors,
  module boundaries, and reserved external-system sockets>
Style/medium: precise 1970s/1980s painted technical concept plate, neutral
  material study, subtle gouache and airbrush texture
Composition/framing: complete front-port three-quarter view, 10-15 degrees
  above centerline, restrained near-orthographic perspective, 3:2 landscape
Lighting/mood: warm upper front-port key, cool weak rim light
Constraints: obey the stated dimensions and configuration; preserve human and
  deck scale; no fitted mission systems beyond the listed sockets; no text,
  logo, watermark, other craft, weapon fire, motion blur, or wide-angle distortion
Avoid: franchise resemblance, generic gray greeble field, contemporary stealth
  aircraft, synthwave neon
```

### Prompt skeleton: family variant

```text
Use case: stylized-concept
Asset type: Cepheus Trader ship catalog exterior
Input images: Image 1: approved family anchor reference; Image 2: previous
  approved family variant reference, if available
Primary request: produce <ship tag and name>, the <role> fit of the shared
  <family> chassis, using the <native path> shipyard language
Subject: <exact visible mission equipment and weapon placements>
Style/medium: 1970s/1980s gouache-and-airbrush science-fiction catalog plate,
  saturated opaque enamel colors, controlled chrome highlights
Composition/framing: exactly preserve the anchor camera, perspective, attitude,
  lighting direction, and crop
Color palette: <native path palette>; <stealth/armor/weathering exceptions>
Constraints: preserve exact family silhouette, dimensions, bridge, deck bands,
  drive block, airlocks, landing/docking geometry, primary seams, and component
  sockets; change only <explicit delta>; render the stated visible weapon count
  and types; no text, logo, watermark, other craft, weapon fire, or motion blur
Avoid: redesigning the hull, moving invariant landmarks, gray military default,
  exposed contemporary missiles, franchise resemblance, synthwave neon
```

## Validation checklist

Reject or revise a plate if any answer is no:

### Identity

- Does it use the approved family anchor rather than merely resemble it?
- Are invariant bridge, drive, deck, door, and seam landmarks preserved?
- Does the native path read through form, components, finish, and color?
- Is the result original and free of excluded source or franchise trade dress?

### Mechanics

- Does its volume plausibly match displacement and canonical dimensions?
- Does the hull visibly honor streamlined, standard, distributed, or
  close-structure configuration?
- Are armor, stealth, sensors, scoops, cargo, hangars, clamps, and launch
  facilities consistent with the fitted record?
- Does every weapon mount have a manifest position, including hidden mounts?
- Do the visible mount count and weapon types match the standard-view list?
- Are bay and barbette installations materially larger than turrets?
- Are carried-craft doors and facilities correctly scaled?

### Presentation

- Does human detail establish the correct scale rather than decorative clutter?
- Are bright paint and chrome used as period-future materials rather than neon
  effects?
- Is the ship completely visible at the family-standard camera and crop?
- Is the image legible at catalog-card size?
- Is it free of generated text, watermarks, battle effects, and unrelated craft?

## Asset records

Use stable filenames such as `ship-054-fogg.webp`; do not key assets only by a
display name that may repeat. Preserve the full-resolution master and generate
web derivatives without overwriting it. Record the approved family anchor,
prompt/brief revision, source ship record revision, artist or generation
provenance, review status, and artwork license beside the asset inventory.

Artwork is not automatically Open Game Content merely because the underlying
ship mechanics and names are. Choose and record the artwork license explicitly
before publication.
