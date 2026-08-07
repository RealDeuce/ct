# Open Game Content Provenance

*Status: required living release record, 2026-07-26*

This document records which Open Game Content sources inform distributed
Cepheus Trader rule material. Update it in the same change that introduces or
removes a source. Catalog attributions are normalized in
`catalog/ogl-sources.toml`; each catalog entry directly lists its complete set
of source IDs. The resulting exact Section 15 notices must remain consolidated
in `OPEN_GAME_LICENSE.md`.

The broader inventory of locally available works is in
[`potential-ogc-sources.md`](potential-ogc-sources.md). A source appearing in
that catalogue is not active and does not belong in the consolidated notice
until its mechanics are actually adapted.

Consulting a source does not make its Product Identity available. Copy only
material that the source clearly designates as Open Game Content, and keep
mechanical adaptations separate from protected names, setting expression,
characters, organizations, locations, artwork, and trade dress.

## Active Sources

| Source | Open material used | Exclusions and handling |
| --- | --- | --- |
| *Cepheus Engine System Reference Document* | Core task, character, world, trade, ship design/operation, travel, combat, and encounter mechanics; the Markdown reference reproduces upstream OGC | Product titles and the “Cepheus Engine” and “Samardan Press” trademarks remain excluded; compatibility statements follow the separate CSL |
| *Clement Sector Third Edition* | Comparative skill vocabulary, CHA substitution, operational procedures, and Z-drive/J-drive analysis | All Clement proper names, setting, locations, characters, organizations, ship names/classes, art, and trade dress are excluded |
| *The Anderson and Felix Guide to Naval Architecture, version 3* | Construction comparisons, components, large-ship crew abstraction, drive conversion, cargo transfer, and support procedures | Named ships/classes and all other declared Product Identity are excluded |
| *Port of Entry: Starports in Clement Sector* | Starport, service, fuel, cargo, and port-operation mechanics | Starports, locations, organizations, named ships/classes, setting text, art, and trade dress are excluded |
| *Bounded Fortune: Independent Merchants in Clement Sector* | Merchant operations, financing, insurance, cargo, passenger, trade, and crew-role mechanics | Corporations, characters, locations, named ships/classes, setting text, art, and trade dress are excluded |
| *Hub Federation Navy Third Edition* | Naval organization and captain-operation mechanics used for role-loop analysis | The Hub Federation and every other organization, person, place, ship/class name, story element, art, and trade dress are excluded |
| *Skull and Crossbones: Piracy in Clement Sector Third Edition* | Piracy, privateering, capture, fencing, and forced-docking mechanics | All named pirates, factions, places, ships/classes, plots, setting text, art, and trade dress are excluded |
| *21 Characters: Clement Sector* | Aggregate, non-identifying skill-distribution measurements used to calibrate point-buy pools | Characters, names, biographies, likenesses, and art are excluded; no character is reproduced |
| *21 Villains* | Aggregate, non-identifying skill-distribution measurements used to calibrate point-buy pools | Characters, names, biographies, likenesses, and art are excluded; no character is reproduced |
| Static ship-catalog source bundles | Mechanical ship designs adapted from the open portions of the 48 published sources represented by the catalog | The authoritative work-by-work declarations, inherited notices, and exclusions are the `ship-source-*` bundles in `catalog/ogl-sources.toml`; all published names, setting expression, art, and trade dress are excluded |

The normalized vessel-combat values used by the engine live in
`catalog/combat-rules.toml`. That file directly lists its complete source-ID
set and is authoritative OGC game data rather than an extraction artifact.
The setting-neutral personnel vocabulary in `catalog/person-names.toml` is
likewise original OGC catalog data; it imports no named setting character.

## Research Material Without a Distribution Grant

`/home/admin/RPG/2D6/Clement Sector/Ships.ods` is useful private research
material but does not itself provide a clear OGL designation. Do not copy it
into the repository or treat it as the licensing source for a catalogue
entry. Verify mechanics against a published OGL source or independently
construct a replacement from open ship-design rules.

`catalog/ships/` contains the admitted static ship records reconstructed from
construction rules and published OGC. It is intentionally smaller than the
spreadsheet inventory: a row is admitted only after its bill of materials
balances and every selected component resolves to a construction rule. The
spreadsheet itself is not distributed, and its ship/class names are not
copied. Each active record directly lists the applicable source bundles from
`catalog/ogl-sources.toml`.

The current spreadsheet contains one entry sourced to the
*Independence Armed Freighter* quick file. That file expressly declares that
no portion of the book is open content, so none of its design was transcribed.
The corresponding permanent ID, `ship-81`, is an independently constructed
CE armed merchant.

Several current design notes contain Clement ship and class names as research
labels. Their replacement is deliberately deferred to the planned ship
catalogue overhaul. They must not ship as final game catalogue names, and
their presence is not a claim that those names are Open Game Content.

## Adding or Removing a Source

For every proposed addition:

1. Read the source's own Open Game Content and Product Identity declaration.
2. Record exactly which mechanical material is being adapted.
3. Confirm that no protected proper name or setting expression enters game
   data, source identifiers, UI text, or marketing.
4. Add each exact attribution to `catalog/ogl-sources.toml`, directly list the
   complete set of IDs in every affected catalog entry, and run
   `python3 tools/compile_catalog_ogl.py --update-license OPEN_GAME_LICENSE.md`
   to regenerate the consolidated notice.
5. Review whether the rule belongs in human-readable OGC data rather than an
   MIT implementation file.

When a source is no longer represented anywhere in distributed content, it
may be removed only after verifying all derived material and inherited
dependencies. Retaining an unnecessary notice is preferable to accidentally
removing required attribution.
