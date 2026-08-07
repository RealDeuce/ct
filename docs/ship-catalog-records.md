# Rule-Derived Ship Catalog Records

*Active format, schema version 1*

## Boundary

A catalog entry is an immutable, fully fitted design template. Ownership,
mortgage, captain, assigned crew, current fuel, cargo, passengers, ammunition,
damage, maintenance, location, legal status, and refit history belong to
persistent records for an individual vessel.

The catalog is static Open Game Content. It is not regenerated from
`Ships.ods`, PDFs, or specification text. Permanent `ship-N` tags remain
stable independently of human-readable names.

## Design Relationships

The naming model preserves two independent relationships:

- A **design family** groups a common hull lineage and its fitted variants.
  Family membership answers questions such as whether two records are cargo,
  passenger, patrol, or missile versions of the same underlying design.
- An **upgrade path** is a progression ladder across multiple families and
  hull sizes. Each path is associated with one manufacturer or shipyard whose
  designs specialize in one cell of the trade/mixed/combat crossed with
  orderly/contested/chaotic polity matrix.

An upgrade path is not merely a list sorted by price or displacement. Its
ships should express the specialist's consistent doctrine and give players
useful improvements in capacity, survivability, reach, support requirements,
or authority as they progress. A family can contain designs assigned to
several different paths—for example, commercial and privateer variants of the
same underlying hull architecture. A family is therefore not owned by a
single path or manufacturer. Family identity, each design's native path,
progression position, and individual catalog-record identity must be stored
separately.

The authoritative family registry is `catalog/ships/families.toml`. Every ship
record repeats its numeric `family_id`; validation requires registry coverage
of every admitted design, exactly one family per design, and exact agreement
between both representations. Anonymous `family-N` tags use the
lowest-numbered current member as a stable anchor. The grouping criteria and
human-readable crosswalk are recorded in
`docs/ship-family-grouping.md`.

The authoritative native-path registry is
`catalog/ships/upgrade-paths.toml`. Every ship record repeats its numeric
`upgrade_path_id` and its mechanically derived `progression_stage`.
Validation requires exactly nine matrix paths, complete and unique native
membership, agreement with every ship record, and the correct size stage.
Coverage, doctrines, and intentional gaps are recorded in
`docs/ship-upgrade-paths.md`.

No path is required to contain a native design at every progression point from
starter craft through 5,000-ton capital ships. A sparse path may explicitly
use a suitable design from an adjacent path to fill a gap. Backfilling does
not reassign the borrowed design's native path, manufacturer, or family, and
it should not cause a duplicate near-identical design to be invented merely
to make the ladder visually complete. The eventual catalog relationship must
record backfill eligibility explicitly rather than infer it solely from hull
size or matrix coordinates.

The nine paths describe design specializations and expected local progression,
not restrictions on ownership or travel. A player may buy, capture, earn
command of, or refit ships outside the path most closely aligned with the home
polity.

`catalog/ships/names.toml` is the authoritative canonical naming registry. It
names every path and its presumed manufacturer, every family, and every fitted
design, and it assigns each path a six-stage semantic naming sequence.
Validation requires complete registry coverage and exact agreement with each
ship's repeated display name. The historical, mythic, geographic, scientific,
and public-domain literary convention is documented in
`docs/ship-catalog-naming.md`.

Canonical catalog data provides the mechanics, stable IDs, names, family/path
relationships, and neutral functional descriptions needed to recognize and
compare designs throughout the shared universe. Local history, advertising,
reputation, cultural interpretation, aliases, and other setting prose may be
authored by the individual BBS sysop. Such presentation must remain an
overlay: it cannot change a design's bill of materials, price, capabilities,
progression position, or balance. Interfaces must retain the canonical name
and stable ID for unambiguous inter-polity use.

## Record Shape

Each `catalog/ships/ship-N.toml` file is a complete, hand-authored bill of
materials accepted by `tools/ship_design.py`. Its top-level fields select a
versioned ruleset, sources, technology level, standard-design treatment, hull,
drives, fuel, computer, software, electronics, installed components, cargo,
crew, and any source assertions.

`catalog/ship-runtime.toml` is the checked-in runtime projection of every
active Jump-capable starship. It contains the evaluator-confirmed price,
displacement, Jump rating, thrust, fuel, cargo, minimum crew, name, and TL that
the Rust server needs to instantiate non-starter vessels. It is not a second
design source and is never edited by hand. Regenerate it only through
`tools/validate_ship_catalog.py --runtime-index catalog/ship-runtime.toml`;
the hand-authored bill of materials and construction tables remain
authoritative. The server build embeds both the projection and the underlying
component records, so ship brokerage is not limited to starting offers.

The `[catalog]` table supplies:

- the numeric ID, matching `ship-N` tag, numeric design-family and native-path
  references, progression stage, revision status, and vessel kind;
- the current PI-free display name;
- primary and secondary roles plus mission tags;
- one or more Open Game Content designations; and
- two or three original paragraphs explaining the intended role, strengths,
  and limitations.

Money is integer credits and volume is integer millitons. A design never
repeats a component's price, displacement, rating, or formula. Those values
come exclusively from `catalog/shipbuilding/`.

`catalog/ships/index.toml` lists every admitted record and is checked against
the files and their embedded metadata. `[[carried_craft]]` references another
admitted `ship-N` entry by tag. Hangar displacement is selected separately
from the construction rules, its capacity must cover the referenced craft,
and each craft's fitted price is included exactly once.

The 191 tags assigned during source coverage are all populated by admitted
rule-derived designs. Unrelated core and original designs use tags beginning
at `ship-192`.

## Assertions

Published specifications are comparison material. Exact claimed totals may be
recorded in `[assertions]`; the evaluator recomputes them and rejects a
disagreement. Assertions cannot supply a missing component, reconcile unused
volume, add a price adjustment, or override a construction rule.

If a published design is invalid, the catalog must either omit it or contain a
clearly documented Cepheus Trader correction built from valid rules. A
correction is an original design variant, not a disguised transcription
adjustment.

## Licensing

Top-level `source_ids` lists every rules and ship-design source used by the
entry. `[catalog].open_game_content_designations` is always a list. The OGL
compiler expands those source IDs through `catalog/ogl-sources.toml` and
deduplicates exact Section 15 notices.

Publisher names, published class names, setting history, artwork, and other
Product Identity do not enter catalog data. Source IDs are internal
attribution keys, not displayed game names.

## Validation

Run:

```sh
python3 tools/validate_shipbuilding_rules.py
python3 tools/validate_ship_catalog.py
python3 tools/compile_catalog_ogl.py --check
```

The catalog validator checks metadata, source IDs, filename/ID agreement,
vessel classification, complete displacement accounting, construction price,
crew, drive/software compatibility, component constraints, and every declared
assertion.
