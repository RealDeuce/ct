# Resolved Stellar Volume

*Status: coverage oracle, persistent materialization bits, and bounded bulk
capacity sampling implemented; ordinary Jump-arrival integration pending,
2026-07-31*

The server must decide stellar existence before a Jump completes into a new
region. This requires a fast authoritative test:

```text
mapping_coverage(target_coordinates)
    -> FullyMapped
    -> NeedsMaterialization(missing_chunk_bitmaps)
```

“Resolved” is more precise than “observed” for this state. A resolved bit
means the server has permanently decided which stellar components exist in
that volume. It does not mean that every player or polity knows the result.
Player knowledge, navigation data, survey provenance, and data age remain
separate physical-repository observations as specified in
[`known-universe.md`](known-universe.md).

When resolution produces an in-world discovery, its report and structured
observations propagate through ordinary mail. Materialization never broadcasts
the result directly to remote repositories.

What a first-arrival ship can learn after the server resolves a region is
specified separately in
[`settlement-and-system-survey.md`](settlement-and-system-survey.md).

## Resolution lattice

Coverage uses an internal Cartesian lattice aligned with the existing
Earth-centered coreward/spinward/north coordinates:

| Property | Value |
| --- | ---: |
| Cell edge | 0.25 pc |
| Chunk edge | 8 pc |
| Cells per chunk axis | 32 |
| Cells per chunk | 32,768 |
| Bitmap size per layer | 4,096 bytes |
| Canonical Jump-arrival mapping radius | 6 pc |

Quarter-parsec and eight-parsec boundaries are exactly representable in
binary floating point. Signed cell coordinates use Euclidean floor and
division, so negative rimward, trailing, and south positions behave
symmetrically with positive positions.

A cell belongs to a Jump footprint when its half-open cube has
positive-volume intersection with the six-parsec destination sphere.
Tangency at a zero-volume boundary does not require resolution. A sphere
centered exactly on chunk boundaries touches eight chunks and resolves 63,256
cells.

The lattice is an internal persistence and query structure, not a navigation
grid. System positions and Jump distances remain continuous parsec-valued
coordinates. Resolving every intersected cell can decide space by at most one
cell diagonal, approximately 0.433 pc, beyond the exact spherical boundary.
That internal materialization does not automatically reveal those results to
a player.

## Persistent record

LMDB stores `coverage-chunks` keyed by three signed 64-bit chunk coordinates.
Each record contains one or more non-overlapping layers:

```text
CoverageChunk {
    layers: [
        CoverageLayer {
            stellar_distribution_version: UInt16
            sampler_version: UInt16
            resolved_cells: 4096-byte bitmap
        }
    ]
}
```

A cell appears in exactly one layer. Later generation versions may resolve
previously empty bits in the same chunk but cannot reassign an earlier bit.
The separate `coverage-revision` metadata value changes whenever new cells
are committed. Chunk record encoding rejects empty, duplicate, overlapping,
truncated, or trailing layer data.

The fixed initial CNS5 volume uses distribution and sampler version 1.
Every cell intersecting the sphere from Sol through Tau Ceti is resolved by
that fixed-catalog layer, preventing later generation from inserting another
component into the initial observed neighborhood. A destructive universe
reinitialization clears all coverage chunks and recreates this layer.

## Oracle result

Given destination coordinates, the oracle constructs the canonical
six-parsec footprint and subtracts the union of persisted layer bits:

```text
FullyMapped {
    coverage_revision
    footprint_cells
}

NeedsMaterialization {
    coverage_revision
    footprint_cells
    missing_cells: Map<ChunkCoordinate, CellBitmap>
}
```

The missing bitmaps are the materialization boundary; no explicit polygon,
mesh, or union of spheres is stored. The query rejects non-finite or
out-of-range coordinates before iterating over cells.

## Authoritative update

The implemented store primitive applies materialization bits only inside a
caller-owned LMDB write transaction and only when its expected
`coverage-revision` still matches. It adds previously unresolved cells,
places them in the stated distribution/sampler layer, and advances the
coverage revision once. A stale plan is rejected rather than silently
overwriting work from an earlier arrival.

The Jump-arrival transaction uses this primitive as follows:

1. a due Jump asks the oracle about its target coordinates;
2. the authoritative transaction samples only the returned missing cells;
3. the operation receives fresh OS-CSPRNG entropy and a cryptographic stream
   draws prospective system seeds, of which only accepted components retain
   their seeds;
4. the write transaction applies the current coverage revision;
5. generated systems, resolved and empty cell bits, the journal decision, and
   Jump completion commit atomically;
6. until that commit succeeds, the ship remains in Jump and observes nothing
   at the destination.

This slice intentionally does not expose a standalone “mark mapped” RPC or a
separate unjournaled store mutation. The low-level bit update is private so
future code cannot publish empty coverage without committing the systems and
random decisions that justify it.
