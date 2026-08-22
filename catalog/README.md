# Cepheus Trader Catalogs

This directory contains human-readable, schema-validated game catalogs.
Every original name, mechanical field, functional description, role, tag,
table, and other game-content element in the catalogs is Open Game Content
under OGL 1.0a. Cepheus Trader reserves no original catalog Product Identity.
Upstream Product Identity remains excluded as stated in `LICENSE.md`.

[`ogl-sources.toml`](ogl-sources.toml) is the single master registry for OGL
sources used by catalogs. Individual entries contain list-valued OGC
designations and source IDs; they do not duplicate source descriptions or
Section 15 notices. Each source record explicitly lists the notice IDs for
the OGL, applicable SRDs and inherited works, and the source work itself; the
loader does not infer ancestry. Verbatim typography and punctuation variants
remain in the registry for source auditing but explicitly name a canonical
notice when they declare the same work, date, and copyright holder. The game
expands the source bundles, resolves those explicit aliases, deduplicates exact
notices, and constructs the full-game OGL declaration from all loaded catalogs.

The active construction work is split deliberately:

- [`shipbuilding/`](shipbuilding/) contains only rule-derived construction
  mechanics;
- [`ships/`](ships/) contains the active, hand-authored ship catalog whose
  bills of materials select those mechanics; and
- `tools/ship_design.py` evaluates designs and checks source assertions.

The naming model has two independent catalog relationships. The family pass
groups all 215 active designs into 114 design families in
[`ships/families.toml`](ships/families.toml). The native-path pass assigns
every design through [`ships/upgrade-paths.toml`](ships/upgrade-paths.toml) to
one of nine product doctrines, each representing one specialist
manufacturer or shipyard aligned with a cell of the
trade/mixed/combat × orderly/contested/chaotic matrix. Stable IDs and
mechanical relationships are global. Local history, advertising, reputation,
and similar setting prose may be supplied by a BBS sysop as a non-mechanical
presentation overlay. Families may contain designs from several paths. Paths
may have progression gaps and explicitly borrow suitable designs from adjacent
paths without changing the borrowed design's native family, manufacturer, or
path.

[`ships/names.toml`](ships/names.toml) is the canonical Open Game Content
naming registry. It names all paths, manufacturers, families, and designs and
defines a six-stage semantic naming sequence for each path. Ship records repeat
their display names for direct loading, and catalog validation requires exact
agreement with the registry. See
[`docs/ship-catalog-naming.md`](../docs/ship-catalog-naming.md).

[`traffic-names.toml`](traffic-names.toml) supplies the nine OGC name and
operator vocabularies used for deterministic, non-persisted ordinary-traffic
projections. The Rust build validates and compiles this catalog; runtime code
does not carry a duplicate hard-coded naming list.

[`person-names.toml`](person-names.toml) supplies the OGC given-name and
family-name pools used for deterministic materialized personnel. Given and
family selections use separate domain draws, and a displayed hiring slate
cannot contain duplicate full names.

[`starting-offers.toml`](starting-offers.toml) maps all 27 new-player packages
to active catalog designs: trader, privateer, and navy/public-service in each
of the nine polity cells. It selects immutable ship templates only. Financing,
title, authority, reserves, staffing, refit options, and exit consequences are
separate versioned package terms. Run
`python3 tools/validate_starting_offers.py` after changing the mapping.

Run `python3 tools/validate_ship_catalog.py` to validate every active entry.
