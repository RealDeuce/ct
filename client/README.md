# Cepheus Trader client

This directory contains the C++20-or-newer BBS door clients and utilities.
The `cepheus-trader-client` headless protocol exerciser connects with Botan 3 TLS 1.3
external-PSK, generates C++ bindings from the shared Cap'n Proto schema, sends
a CT-RPC `ClientHello`, validates the `ServerHello`, and reports its assigned
session epoch and phase. Its deterministic `create-player` mode queries the
authoritative defaults, submits a complete creation proposal, and validates
the transition to `docked`.

After `ServerHello`, the shared client transport runs an independent receive
dispatcher. Synchronous RPC helpers correlate their response while deferring
interleaved unsolicited events; the door polls those events while waiting for
input. Authoritative phase changes are retained, while local traffic snapshots
and named arrival/departure notices are presentation data and may be dropped
under output backpressure.

The user-facing `cepheus-trader-door` executable links that same client core
to the vendored OpenDoors source as a static library.
Its implemented local mode initializes the OpenDoors terminal,
performs the authenticated hello and, in `newUser`, walks through captain
name and skills, the three BBS-specific starting offers, detailed ship
selection and naming, an editable role/name roster for officers and senior
specialists, review, and atomic submission. Role-specific default callsigns
allow immediate acceptance; selecting a roster letter displays that person's
fixed characteristics and skills and permits renaming or selecting the skill
being trained. The display includes current and required training weeks.

After creation, and for returning players, a ship in the `Docked` phase opens
the Docked Operations menu: Cargo Exchange, Jobs and Passage, Fuel and
Supplies, Shipyard, Personnel, Banking and Accounts, Authorities, and Depart.
`U` opens the common Crew, Ship, Task, Message, and Known Universe managers.
The shell is implemented for every presentation profile. Crew Management now
reads the authoritative roster, changes training targets, and edits
zero-or-more watch roles per named person. Empty duty lists put a person off
watch; several roles support CE role-doubling, while the server enforces a
single Pilot. Task, Message, Known Universe, and the eight docked services
remain descriptive landing pages pending their authoritative RPCs.

Ship Status is also implemented as an authoritative read-only manager. It
shows catalog performance, fuel and cargo capacity, and paginated subsystem
condition and maintenance. Sustained damage, temporary battle-repair coverage,
effective encounter damage, and permanent-service dates are displayed
separately. Damage-control, proper-repair, and maintenance mutations remain
pending.

The initial door screen identifies the game as an Alternate Cepheus Engine
Universe, displays the OGL notice and required trademark/non-affiliation
statement, and provides `L` to view the complete OGL and consolidated
copyright notice. The license remains available from the connected and error
screens.

The implemented presentation is page-oriented and line-oriented, with three
profiles: ISO 646 plain text, ISO 646 plus ECMA-48 colour, and CP437 plus
ECMA-48 colour. A 40×24 terminal is the supported minimum and 80×24 is the
normal target. There is no cursor-addressed TUI: form feed clears a plain
page, while enhanced pages use clear-and-home before emitting wrapped text.
OpenDoors supplies transport, input, colour, and CP437 conversion support,
while the door owns responsive wrapping because OpenDoors' legacy
screen-buffer helpers are fixed at 80×25. See
[`docs/door-presentation.md`](../docs/door-presentation.md).

Enhanced screens use a classic high-contrast BBS palette: cyan labels,
bright-white text values, yellow numeric data, magenta identifiers, green
information/success, and red invalid or dangerous state. These are semantic
roles, and plain ISO 646 never relies on colour to convey meaning.

The `cepheus-trader-admin` executable connects to the server's loopback-only
administrator listener with the raw 32-byte `admin.psk` file. `add-bbs`
prints the committed BBS ID and generated BBS PSK for one-time out-of-band
transfer to the BBS sysop; the output must not be logged or retained as
ordinary configuration. `initialize-universe` performs the explicitly
confirmed destructive reset and creates the initial 35-system Federation
through Tau Ceti with Earth at TL13. A fresh server must be initialized before
`add-bbs`; premature enrollment is rejected without creating a credential. See
[`docs/initial-federation.md`](../docs/initial-federation.md).

The `cepheus-trader-sysop init-credential` command bootstraps the BBS-side
installation. It reads the BBS ID and 64-hex-digit PSK from standard input,
one per line. When standard input is a terminal it prompts for both and
disables terminal echo while reading the PSK. It never reads a PSK from
command-line arguments or the environment.

The sysop utility and player door share one BBS installation configuration,
`cepheus-trader.conf` by default. Its strict `key=value` schema is:

```ini
server=game.example.net
game-port=7323
sysop-port=7325
credential-file=cepheus-trader.credential
```

The configuration contains no PSK. `credential-file` names the separately
protected binary credential and is resolved relative to the configuration
file, so the programs do not depend on their working directory. Blank lines
and lines beginning with `#` are allowed; unknown and duplicate keys are
rejected. Both programs accept `--config FILE` to select a non-default path.
See `cepheus-trader.conf.example`.

`init-credential` accepts `--config FILE` and an optional credential pathname.
If the configuration is absent, it exclusively creates it with `127.0.0.1`,
game port `7323`, sysop port `7325`, and the selected credential pathname. If
the pathname is omitted, the default is `cepheus-trader.credential` beside
the configuration. If the configuration already exists, the command uses its
`credential-file` value; an explicitly supplied pathname must identify that
same file. Missing parent directories are created; newly created Unix
directories are owner-only. Neither file is overwritten. `get-config`,
`set-config`, and explicit `--expected-revision`/`--command-id` retries require
the shared configuration to exist.

The sysop executable connects to the separate BBS-sysop listener using this
configuration:

- `get-config` reads the current BBS configuration and revision; and
- `set-config` accepts the BBS and polity display names and the two
  orientations. The utility reads the current revision and generates the
  idempotency command ID itself.

Both orientation values are integers from 0 through 100. `trade-combat` uses
0 for completely trade-oriented and 100 for completely combat-oriented;
`chaos-order` uses 0 for completely chaotic and 100 for completely
institutionally orderly. CE Law Level is separate generated world state.

The credential file is a versioned 48-byte binary record containing the BBS
ID and 32-byte PSK. Creation is exclusive and refuses to overwrite an
existing path. On Unix it is created as `0600`, without following a symlink;
loading rejects non-regular files, a different owner, group/other
permissions, or multiple hard links. On Windows it receives a protected DACL
for the owner, Local System, and Administrators, and reparse points are
rejected.

The BBS ID and BBS-local player ID are `UInt32` values forming the full player
identity `(BBS ID, local player ID)`. The BBS ID's canonical unsigned decimal
representation is the TLS PSK identity. The server rejects a hello whose BBS
ID does not match the authenticated TLS identity.

Both player executables read this credential file. BBS PSKs are not accepted
on their command lines or through environment variables.

`server/tests/tls_interop.rs` builds this client and runs it against the real
Rust/GnuTLS listener through the complete authenticated hello exchange. It
also launches the OpenDoors door twice in local mode for the same player
identity and verifies both `newUser` sessions. The headless client exercises
the complete option-query and creation transaction. Simulation rules remain
server-side.

Build and run:

```console
cmake -S client -B client/build -G Ninja
cmake --build client/build
client/build/cepheus-trader-admin \
  --psk-file server/server-data/admin.psk initialize-universe
client/build/cepheus-trader-admin \
  --psk-file server/server-data/admin.psk add-bbs "Dark Star"
client/build/cepheus-trader-sysop \
  --config /secure/path/cepheus-trader.conf init-credential \
  /secure/path/ct-bbs.credential
client/build/cepheus-trader-sysop \
  --config /secure/path/cepheus-trader.conf set-config \
  "Dark Star BBS" "Far Reach" 65 25
client/build/cepheus-trader-sysop \
  --config /secure/path/cepheus-trader.conf get-config
client/build/cepheus-trader-client \
  127.0.0.1 7323 /secure/path/ct-bbs.credential 1
client/build/cepheus-trader-door \
  -L -C /secure/path/doors.conf -USERNAME "Alex Mercer"
client/build/cepheus-trader-client \
  127.0.0.1 7323 /secure/path/ct-bbs.credential 1 create-player \
  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa "Alex Mercer" "Far Horizon" Samir Morgan
```

The CMake build prefers an installed Cap'n Proto 1.0-or-newer package. If one
is not available, it downloads the official Cap'n Proto 1.5.0 source archive,
verifies its pinned SHA-256 digest, and builds it inside the build tree. Set
`CT_USE_SYSTEM_CAPNPROTO=OFF` to exercise or force the fetched build. MinGW
cross-builds use host-native `capnp` and `capnpc-c++` programs for schema
generation while linking against the target libraries.

The pinned Botan build uses all processors detected by CMake. Constrained
builders may set `-DCT_BOTAN_BUILD_JOBS=N` to an explicit positive job count.
Compatible build trees can share a completed pinned installation by setting
`-DCT_BOTAN_INSTALL_DIR=path`; CMake requires both its static library and
matching completed-install marker before reusing it.

For a standard local installation, omit the final pathname from
`init-credential`; it creates the parent directory, the shared configuration,
and `/secure/path/cepheus-trader.credential`. Its output names both files and
prints the shape of the next `set-config` command.

The build keeps OpenDoors inside the door executable and copies the applicable
OpenDoors, Botan, and Cap'n Proto notices into the package. Binary packaging
must publish the matching tagged source and satisfy the obligations in
[`THIRD_PARTY_LICENSES.md`](../THIRD_PARTY_LICENSES.md).

Release builders set `CT_PORTABLE_CLIENT=ON`. That forces the pinned Botan and
Cap'n Proto source builds, static OpenDoors linkage, and static GNU C/C++
runtime linkage on supported GNU targets. CPack installs only the player door
and sysop utility; the administrator and headless exerciser are deliberately
excluded from BBS client kits.

OpenDoors owns the door command line and drop-file parsing. The OpenDoors
configuration names the shared Cepheus Trader configuration with
`CTConfig /secure/path/cepheus-trader.conf`; optional `CTProfile`, `CTColumns`,
and `CTRows` directives override presentation. In local mode, `-USERNAME`
supplies the BBS account name. On a real BBS, the door uses the real name and
user-record index supplied by OpenDoors, maps that tuple through the protected
local identity registry, and sends only the resulting UInt32 player ID.

The administrator defaults to `127.0.0.1:7324`, reads
`server-data/admin.psk`, and generates its idempotency command ID. `--host`,
`--port`, and `--psk-file` override the connection defaults. If a request
fails after it may have reached the server, the utility prints the generated
ID; pass it back with `--command-id` for the retry so that the operation
cannot create a second BBS.

The sysop command interactively reads the transferred ID and PSK. Do not put
the PSK into a shell pipeline, command substitution, script, command-line
argument, or environment variable.

For normal `set-config`, the utility first reads the current revision and
generates the command ID. If the update fails after it may have reached the
server, it reports both values. Supply that exact pair with
`--expected-revision REVISION --command-id HEX` when retrying. A stale
revision is rejected and reports the current revision.
