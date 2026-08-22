# Universe Atlas Snapshots

`cepheus-trader-universe-site` is a one-shot Rust exporter for an
operator-hosted, interactive universe atlas. It opens an existing Cepheus
Trader LMDB environment read-only, writes a complete static-site directory,
and exits. The resulting site has no server process, database connection,
cookies, or write API. An ordinary HTTP server only needs to serve its four
files.

The atlas uses a dependency-free Canvas renderer rather than a hosted script
or WebGL framework. In a modern browser it provides three-dimensional
orbiting, panning, zooming, system search and selection, world details, and
configurable jump-range links. Its route computer accepts typed endpoints or
two systems picked directly from the map, finds a reproducible fewest-jump
route at the current jump setting, and displays every leg and its total
distance. It replots automatically when the jump setting changes. The route
uses geometric reachability among systems visible in that snapshot; it does
not account for fuel availability, travel time, hazards, or information hidden
by the selected visibility scope.

Coordinates follow the game axes: +X is coreward, +Y is spinward, and +Z is
galactic north. The data export contains derived public world characteristics,
but never deterministic generation seeds or private player records.

## Build and generate

Build the standalone exporter with:

```console
cargo build --release --manifest-path server/Cargo.toml \
  --bin cepheus-trader-universe-site
```

Generate a normal public snapshot from the live data directory:

```console
server/target/release/cepheus-trader-universe-site \
  --data /srv/cepheus-trader/data \
  --output /srv/http/cepheus-trader/atlas-next
```

The default visibility is `universally-known`. The exporter includes only
systems whose public mapping broadcast has completed across every currently
applicable system. That state is monotonic: a system does not disappear from a
later public snapshot. The initial 35-system Federation is universally known
at game second zero.

An operator-only snapshot can deliberately bypass that filter:

```console
server/target/release/cepheus-trader-universe-site \
  --data /srv/cepheus-trader/data \
  --output /srv/http/cepheus-trader/atlas-omniscient-next \
  --visibility omniscient
```

`omniscient` exposes every materialized stellar system, including systems no
player has made public. It should not be published at a player-visible URL.
The chosen scope is embedded in `universe.json` and displayed prominently in
the viewer. On a storage-format-1 database that predates publication tracking,
omniscient mode still exports every stored system. The conservative
universally-known compatibility view exports the fixed 35-system baseline
until the current server has backfilled the publication index.

For a documentation site or a new universe without a database, generate the
fixed initial map with:

```console
server/target/release/cepheus-trader-universe-site \
  --initial-universe \
  --output site/_site/atlas
```

This mode is always `universally-known` and intentionally omits procedurally
derived world details except for fixed Earth data.

## Deployment behavior

The output consists of `index.html`, `atlas.css`, `atlas.js`,
`atlas-routes.js`, and `universe.json`. Serve the directory over HTTP or HTTPS;
opening `index.html` directly as a `file:` URL may prevent the browser from
fetching the JSON file.

The output path must not already exist. This prevents a typo from replacing a
live web root or mixing files from different snapshots. A typical deployment
generates a new sibling directory, verifies it, and then changes the web
server alias or symlink to that directory. Old snapshot directories can be
retained for rollback and removed by the operator's normal deployment tooling.

Each run is a point-in-time export. Nothing in the web application writes to
the game server, and refreshing the browser does not refresh the universe.
Run the exporter again on the operator's preferred schedule and deploy the new
directory to publish another snapshot. The generator can run while the game
server is online; LMDB provides a consistent read transaction while the
authoritative database continues advancing.

The JSON schema currently has `schemaVersion: 1`. Web-server compression is
recommended for large universes. The viewer avoids an always-running render
loop, but neighbor highlighting is necessarily linear in the exported system
count when selection or jump range changes.
