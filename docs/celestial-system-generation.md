# Derived Celestial-System Generation

*Status: generation version 1 implemented, 2026-08-06*

Celestial baselines are not database rows. A materialized stellar-system
record persists its 256-bit seed, generation version, location, polity, and
mutable display names. The server derives the immutable primary-world,
stellar, orbital, planetary, moon, and physical data whenever it needs them.
Only later changes—population deltas, facilities, ownership, construction,
damage, depletion, surveys, and similar overlays—belong in persistent state.

First-arrival observation, the historical settlement envelope, and the
distinction between an empty generated system and a player-founded settlement
are specified in
[`settlement-and-system-survey.md`](settlement-and-system-survey.md).

`server/src/celestial.rs` adapts the explicitly Open Game Content
system-generation method on pages 44–149 of *Unmerciful Frontier: The CCA
Sourcebook*, third edition. Core Cepheus Engine remains authoritative for the
primary-world UWP and PBG values. Consequently the detailed generator:

- retains the previously accepted CE primary world exactly;
- uses the CE planetoid-belt and gas-giant counts instead of the conflicting
  *Unmerciful Frontier* count rolls;
- rejection-samples a stellar architecture capable of containing an inhabited
  primary world and all CE gas giants;
- omits hex occupancy and every Zimm Point rule;
- uses the source's realistic stellar-class distribution, multiplicity and
  companion-distance tables, main-sequence physical table, zone formulae,
  varied-distance orbital placement, rocky-world zone tables, gas-giant
  classes, belt composition and width tables, moon tables, eccentricities,
  rotation, axial tilt, world physical details, and non-Zimm quirk codes; and
- gives every orbit a deterministic orientation and epoch phase so Keplerian
  positions can be evaluated at arbitrary simulation times.

Generation version 1 truncates the usable habitable zone at 30% of companion
periapsis. An inhabited primary world is rejected unless its final orbit
remains inside both that truncated region and the source habitable zone.
Companion stars have complete Keplerian elements—eccentricity, orientation,
epoch phase, and period—so their actual positions participate in the union of
stellar Jump-exclusion volumes.

Where the source asks a referee to choose among equally valid placements,
generation version 1 chooses deterministically: the primary world anchors the
layout, other rocky worlds alternate inward and outward using varied
distances, and CE-required gas giants and belts are distributed through the
remaining legal range without discarding a CE count.

## Independent streams

The existing CE primary-world stream is frozen. New data uses HMAC-SHA-256
child seeds labeled:

```text
celestial/stellar/v1
celestial/stellar-orbits/v1
celestial/orbits/v1
celestial/bodies/v1
celestial/quirks/v1
```

Adding a draw to one stream cannot alter another. Mutable overlays refer to
stable system-local body identifiers. During undeployed development, a change
to the body graph increments the generation and storage formats and requires
destructive universe reinitialization. Production migration policy must be
defined before the first persistent deployment.

## Sol and Earth

System ID 1 is a code-defined exception and ignores its random generation
seed. It contains the Sun, eight planets, Pluto, the main belt, and the major
operationally relevant moons. Earth is the fixed primary world `A867984-D`,
with its Solar orbit and physical values fixed in the same definition. Mutable
future history still lives in overlays rather than in this astronomical
baseline.

## Stellar-table representation

Main-sequence O through M values are encoded directly from the source table.
Brown- and white-dwarf sequences interpolate the source endpoints. Evolved
luminosity classes retain the source distribution and are calculated from the
corresponding spectral baseline with version-1 luminosity/mass factors rather
than storing hundreds of redundant zone rows; zone columns are calculated
from mass and luminosity. These are declared generation-version rules, not
persisted results.

## Jump-safety geometry

`server/src/navigation.rs` is the authoritative port-to-Jump geometry layer.
At a requested game time it derives absolute positions for the complete star
and body orbit graph, forms the union of every 100-diameter exclusion sphere,
and finds a verified safe locus using a deterministic direction search.
Stellar diameters are derived from luminosity and temperature; rocky,
gas-giant, moon, and major belt-body diameters come from their generated body
records.

The game adds an operational safety sphere centered on the port world. Its
radius is acceleration-aware: under constant thrust with a midpoint turnover,
it takes exactly half a game day to traverse at the ship's actual thrust.
Consequently no drive rating can reduce the port-to-safe-locus operation below
half a game day merely because the physical 100-diameter limit is small.
