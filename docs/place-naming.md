# Place naming

*Status: catalogue schema 1 and naming algorithm 1 implemented, 2026-07-31.*

Place names are setting data, not identifiers and not debug labels. Systems,
worlds, and polities retain numeric database identities independently of their
display names. Renaming a polity therefore never renames all of its systems,
and changing a naming catalogue never silently changes already materialized
places.

## Catalogue and profiles

[`catalog/place-names.toml`](../catalog/place-names.toml) is the canonical,
hand-maintained naming vocabulary. The server build validates it and embeds it
as typed static data; the runtime does not repeatedly parse the TOML file.
Catalogue schema 1 defines six profiles:

1. `federation` — broad settlement-era names used for Federation worlds;
2. `lyric` — open vowels and soft consonant clusters;
3. `marcher` — compact names with harder consonants and frontier compounds;
4. `classical` — classical-style phonemes and civic settlement vocabulary;
5. `northern` — northern phonemes and geographic settlement vocabulary; and
6. `mercantile` — trade-oriented phonemes and commercial settlement terms.

Each profile independently defines four system fragments, four world
fragments, and a formatting pattern for each kind. Every fragment position has
20 choices, giving up to 160,000 raw system combinations per profile before
collision retries. Patterns allow profiles to concatenate syllables or use
word boundaries without putting formatting logic in the database layer.

## Assignment

The Federation always uses profile 1. Each newly materialized BBS polity draws
one of profiles 2–6 from a domain-separated stream derived from its
cryptographically random placement seed. The selected profile ID is persisted
in the polity record and survives polity renaming and sysop configuration
changes.

Unaligned space uses 25-parsec regional naming cells. A stable hash of the
signed three-dimensional cell coordinate selects profiles 2–6, so nearby
unaligned systems tend to share a naming culture without requiring a master
universe seed. A BBS polity's adjacent unaligned frontier gateway follows its
regional profile, not the polity profile.

Established astronomical system names in the initial Federation volume remain
fixed. Earth remains fixed. The principal world of a BBS capital retains the
sysop-selected BBS name; other principal worlds use their polity or regional
profile.

## Determinism and uniqueness

Name draws use a domain-separated `SeedStream`. Naming draws never consume the
stream used to choose physical system properties, topology, or conditioned CE
world seeds. Changing vocabulary therefore requires a naming/generation
version change but does not accidentally perturb unrelated physical random
draws.

Before allocating a generated system name, materialization loads every stored
system name into a case-insensitive set. A collision consumes another naming
draw. If the bounded retry budget is ever exhausted, the complete transaction
fails instead of admitting an ambiguous name. This check includes fixed nearby
stellar names, earlier BBS clusters, frontier systems, and systems created in
the current transaction. Concurrent materialization remains serialized by the
database write transaction.

Principal-world names need not be globally unique: navigation and knowledge
always identify them as the `(system, world)` pair, just as real settlements
may share a name. System names—the names used as interstellar destinations—are
globally unique case-insensitively.

## Versioning and future sysop control

`PLACE_NAMING_VERSION`, the BBS polity generator, and the settlement-capacity
sampler all begin at version 1 and use the catalogue-backed system. Every stored
polity carries its selected naming profile explicitly.

A later sysop-facing naming editor may select a profile, add vocabulary, or
rename individual materialized places. It must validate uniqueness and must
not reinterpret physical seeds or retroactively rename places merely because a
profile changes.
