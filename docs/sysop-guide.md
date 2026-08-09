# Cepheus Trader sysop guide

This guide is for invited BBS sysops participating in the Cepheus Trader
field test. The game is still in alpha and is not yet balanced for public
enrollment. The server operator gives selected sysops a BBS ID and BBS PSK
privately over Discord.

The shared field-test server is `ct.bbsdev.net`. Player doors connect to port
`7323`; the sysop utility connects to port `7325`.

Feedback, bug reports, and attaboys should be filed as
[GitHub issues](https://github.com/RealDeuce/ct/issues) during the field test.
Include the client version, host operating system and architecture, BBS
software, launch method, and relevant error text when reporting a problem.

## Install the client package

Download the archive for the BBS host from the
[Cepheus Trader releases](https://github.com/RealDeuce/ct/releases):

- `linux-x86_64` or `linux-aarch64` for Linux;
- `darwin-x86_64` or `darwin-arm64` for macOS; or
- `windows-x86` or `windows-x86_64` for Windows.

Extract the complete archive into a dedicated door directory and use that
directory as the door's working directory. The examples in this guide assume
the current directory is that extracted top-level directory. Its `bin`
directory contains:

- `cepheus-trader-door`, the OpenDoors player door;
- `cepheus-trader-sysop`, the BBS management utility; and
- `libcepheus-trader-client-core.so`,
  `libcepheus-trader-client-core.dylib`, or
  `cepheus-trader-client-core.dll`, depending on the platform.

On Windows the two programs have an `.exe` suffix. Keep the shared client-core
library in the same directory as the programs. The `share/cepheus-trader`
directory contains the player and sysop guides, the example configuration,
source correspondence, and license notices.

Check the installation before adding it to the BBS:

```console
bin/cepheus-trader-door --version
bin/cepheus-trader-sysop --version
```

Run the corresponding `.exe` files on Windows. Both commands must report the
same product version.

## Create the BBS credential and configuration

Place the extracted package at a permanent location such as
`/srv/bbs/doors/cepheus-trader`, change to that directory, and configure the
BBS to use it as the door's working directory. The default filenames then work
for both programs without repeated path options.

Run `init-credential` interactively:

```console
cd /srv/bbs/doors/cepheus-trader
bin/cepheus-trader-sysop \
  --server ct.bbsdev.net --game-port 7323 --sysop-port 7325 \
  init-credential
```

For example, a Windows installation may use:

```console
cd /d C:\BBS\Doors\CepheusTrader
bin\cepheus-trader-sysop.exe --server ct.bbsdev.net --game-port 7323 --sysop-port 7325 init-credential
```

Enter the assigned BBS ID and 64-hex-digit BBS PSK when prompted. The utility
hides the PSK while reading it. Do not put the PSK in a command-line argument,
environment variable, command substitution, script, or ordinary log.

The command creates three files without overwriting an existing installation:

- `cepheus-trader.conf`, the non-secret shared configuration;
- `cepheus-trader.credential`, the BBS ID and PSK; and
- `cepheus-trader.identities`, the local BBS-account-to-player registry.

On Unix, newly created directories are owner-only and the credential is mode
`0600`. On Windows, the credential receives a protected DACL for its owner,
Local System, and Administrators. Run the door and sysop utility under the
same dedicated account, and ensure that other BBS users cannot read or replace
these files.

### Shared configuration

The generated `cepheus-trader.conf` has this strict `key=value` form:

```ini
server=ct.bbsdev.net
game-port=7323
sysop-port=7325
credential-file=cepheus-trader.credential
identity-file=cepheus-trader.identities
identity-name=real-name
terminal-profile=auto
terminal-columns=0
terminal-rows=0
inactivity-timeout-seconds=300
```

Blank lines and lines beginning with `#` are allowed. Every key except
`inactivity-timeout-seconds` is required; configurations created before this
setting was introduced use the 300-second default. Unknown and duplicate keys
are errors.

`credential-file` and `identity-file` may be absolute or relative. The default
relative names are resolved from the directory containing
`cepheus-trader.conf`.

`identity-name` selects the BBS account field used in the local identity
registry. Use `real-name` or `handle`. Keep this choice stable after players
have entered the game; use the identity-management commands below to resolve
an intentional account rename.

Leave `terminal-profile=auto`, `terminal-columns=0`, and `terminal-rows=0` for
normal BBS operation. The door then uses the ANSI capability and terminal
dimensions OpenDoors reads from the drop file. If a drop-file format does not
provide credible dimensions, the door uses 80x24.

`inactivity-timeout-seconds` controls how long the door waits without keyboard
input before disconnecting the caller. The default is 300 seconds (five
minutes). Values from 1 through 32767 select a timeout in seconds; 0 disables
the inactivity timeout. Edit the value directly or update it with the sysop
utility:

```console
bin/cepheus-trader-sysop set-inactivity-timeout 600
```

## Configure the BBS in the game

Set the BBS and polity names and the two orientation values:

```console
bin/cepheus-trader-sysop set-config "Dark Star BBS" "Far Reach" 65 25
```

Both orientation values range from 0 through 100:

- `trade-combat` is 0 for completely trade-oriented and 100 for completely
  combat-oriented.
- `chaos-order` is 0 for completely chaotic and 100 for completely
  institutionally orderly.

Review the committed configuration:

```console
bin/cepheus-trader-sysop get-config
```

## Configure the BBS launcher

With the standard working directory, the door reads `cepheus-trader.conf`
without an additional path option.

Configure the BBS to start the process in the Cepheus Trader installation
directory and pass the absolute path of the current session's drop file:

```console
bin/cepheus-trader-door -D /absolute/path/to/bbs/node/CURRENT-DROPFILE
```

Passing the file itself ensures that OpenDoors reads the current drop file
when a node directory also contains stale files from earlier sessions or other
drop-file formats.

### Synchronet installation

The client package includes `install-xtrn.ini` for Synchronet. Complete the
credential and game configuration above, then run Synchronet's installer with
the extracted package directory:

```console
jsexec install-xtrn /srv/bbs/doors/cepheus-trader
```

From a Synchronet terminal logged in as a sysop, the equivalent command is:

```text
;exec ?install-xtrn /srv/bbs/doors/cepheus-trader
```

The installer registers a native, multi-user external program and uses the
directory containing `install-xtrn.ini` as its working directory. It configures
Synchronet to generate `CHAIN.TXT`, use Socket I/O with its protocol-neutral
passthrough socket, translate the door's output for the caller's character set,
and suppress a local display window. Its installed command line is:

```text
bin/cepheus-trader-door%. -D %f -SOCKET %h -SILENT
```

Synchronet expands `%.` to `.exe` on Windows and to an empty string on Unix,
`%f` to the current `CHAIN.TXT` path, and `%h` to the inherited passthrough
socket handle. The caller may be connected to Synchronet through Telnet, but
the door-side passthrough socket is not a Telnet stream: Synchronet handles
Telnet negotiation, IAC expansion, and output translation itself.

### Supported drop files

The door supports the drop-file formats recognized by its vendored OpenDoors
library:

| Drop file | Supported variants | Identifies a Telnet connection? |
| --- | --- | --- |
| `DOOR32.SYS` | Local, serial, or Telnet communication types; there is no non-Telnet socket type | Yes. Communication type `2` identifies a Telnet socket; the next line supplies its descriptor or handle. |
| `DOOR.SYS` | GAP/PCBoard, DoorWay, and Wildcat! | No. The GAP form can identify socket transport, but cannot distinguish a Telnet stream from a non-Telnet raw stream. |
| `DORINFOx.DEF` | Node-specific DORINFO files | No |
| `EXITINFO.BBS` | QuickBBS 2.6/2.75+ and RemoteAccess 1.x/2.x, with the accompanying `DORINFO1.DEF` | No |
| `CHAIN.TXT` | WWIV | No |
| `SFDOORS.DAT` | Spitfire; `SFMAIN.DAT`, `SFFILE.DAT`, `SFMESS.DAT`, and `SFSYSOP.DAT` are also accepted entry filenames | No |
| `CALLINFO.BBS` | Wildcat! | No |
| `TRIBBS.SYS` | TriBBS | No |

Cepheus Trader uses the account name, optional BBS user-record number, ANSI
capability, data width/framing, and terminal dimensions that OpenDoors obtains
at startup. The following table records what the current OpenDoors parser
obtains from each format:

| Drop file | Real name | Handle | User-record number | ANSI | Data width/framing | Terminal dimensions |
| --- | --- | --- | --- | --- | --- | --- |
| `DOOR32.SYS` | Yes | Yes | Yes | Yes | Not supplied | None |
| `DOOR.SYS` | Yes | Wildcat! extended format only | GAP/PCBoard and Wildcat! only | Yes | GAP/PCBoard data-bits field; not supplied by other variants | Rows in GAP/PCBoard and Wildcat!; no columns |
| `DORINFOx.DEF` | Yes | No | No | Yes | Baud/parity/data/stop field | None |
| `EXITINFO.BBS` | Yes | RemoteAccess 1.x extended and 2.x only | RemoteAccess 2.x only | Yes | From the companion `DORINFO1.DEF` | Columns in RemoteAccess 1.x extended and 2.x; no rows |
| `CHAIN.TXT` | Yes | Yes | Yes | Yes | Data/parity/stop field | Columns and rows |
| `SFDOORS.DAT` and the other Spitfire entry files | Yes | No | Yes | Yes | Not supplied | None |
| `CALLINFO.BBS` | Yes | No | No | Yes | Not supplied | Rows only |
| `TRIBBS.SYS` | Yes | Yes | Yes | Yes | Not supplied | None |

ANSI and data width are independent. ANSI says the caller can process
ECMA-48 control sequences; an explicit eight-data-bit framing value says the
path can carry CP437 bytes. When a format supplies neither data width nor
framing, automatic selection keeps the ISO 646 repertoire even if ANSI is
available.

The generated configuration defaults to `identity-name=real-name`, which works
with every supported format. Set `identity-name=handle` only when the selected
format supplies a handle. `-USERNAME` supplies the real-name field, not the
handle field. When a BBS cannot generate a handle-bearing format, either keep
the real-name identity or generate a per-call OpenDoors configuration file
containing `Alias CALLER_HANDLE` and pass that file with `-C`.

The user-record number is optional. With a format that omits it, the identity
registry consistently identifies the account by name alone. Do not add a
record number to that mapping with `identity-reindex` unless the BBS is also
changed to provide the same number on every subsequent launch.

When dimensions are absent, or OpenDoors reports fewer than 40 columns or 24
rows, the door uses 80x24. Fixed dimensions can be set with `terminal-columns`
and `terminal-rows` in `cepheus-trader.conf`. For dimensions that vary by call,
generate a per-call OpenDoors configuration file containing, for example:

```text
CTColumns 132
CTRows 50
```

and pass it with `-C`. `CTProfile iso646`, `CTProfile iso646-color`,
`CTProfile cp437-plain`, or `CTProfile cp437-color` can override the detected
combination for that call; `terminal-profile` supplies a fixed BBS-wide
override.

When the BBS passes the caller's raw Telnet socket directly to the door, it
must generate `DOOR32.SYS` with communication type `2` and launch the door with
the path to that file:

```console
bin/cepheus-trader-door -D /absolute/path/to/bbs/node/DOOR32.SYS
```

The other formats cannot indicate that Telnet IAC expansion is required.

`DOOR32.SYS` does not define a communication type for a non-Telnet TCP socket.
To pass an already-open non-Telnet socket, use another supported drop file for
the caller and terminal information and add the socket option to the command
line. For example:

```console
bin/cepheus-trader-door \
  -D /absolute/path/to/bbs/node/DORINFO1.DEF \
  -SOCKET DESCRIPTOR
```

`DESCRIPTOR` is the numeric socket descriptor or handle inherited from the
BBS. `-SOCKET` selects socket transport but does not identify the stream as
Telnet, so do not use it to pass a raw Telnet connection. OpenDoors also
accepts its normal node, time, graphics, and silent-mode options. Quote paths
according to the host operating system.

The door reads the account name and, when available, the user-record index
from OpenDoors. It records that local identity and sends only the assigned
numeric player ID to the game server.

### Local startup test

Test configuration loading and terminal presentation without a drop file:

```console
bin/cepheus-trader-door -L -USERNAME "Test User"
```

At the opening screen, confirm that the title and menu render correctly, then
quit. Entering the game assigns a persistent identity to the test name.

## Manage local player identities

List the local identity registry:

```console
bin/cepheus-trader-sysop identity-list
```

Each line reports the numeric player ID, active or retired state, BBS record
index (or `none`), and account name. If a BBS account is renamed or its record
number changes, update the existing mapping:

```console
bin/cepheus-trader-sysop identity-rename PLAYER_ID "New Name"
bin/cepheus-trader-sysop identity-reindex PLAYER_ID RECORD_INDEX
```

Use `none` as the record index when the BBS does not provide one. Retire the
mapping after its BBS account has been permanently deleted:

```console
bin/cepheus-trader-sysop identity-retire PLAYER_ID RETIRE
```

Retirement is permanent, and numeric player IDs are never reused. It changes
the local BBS identity mapping; it does not delete the server-side captain or
assets.

## Manage player access and directives

Inspect a player's server-side access state:

```console
bin/cepheus-trader-sysop player-access PLAYER_ID
```

Suspend or resume access with a recorded reason:

```console
bin/cepheus-trader-sysop suspend-player PLAYER_ID "REASON"
bin/cepheus-trader-sysop resume-player PLAYER_ID "REASON"
```

Permanent removal requires the literal confirmation `REMOVE`:

```console
bin/cepheus-trader-sysop remove-player PLAYER_ID REMOVE "REASON"
```

Removal is an irreversible server-side tombstone. The player and assets remain
in the persistent game history. Retire the local identity separately when the
BBS account is also deleted.

Tax and naval-demotion directives are delivered through the game's physical
mail system rather than applied instantly:

```console
bin/cepheus-trader-sysop tax-player PLAYER_ID CREDITS
bin/cepheus-trader-sysop demote-player PLAYER_ID NAVAL_GRADE_INDEX
```

If `set-config` may have reached the server before a connection failure, the
utility reports the revision and command ID needed for an idempotent retry.
Repeat the same command with the exact reported `--expected-revision` and
`--command-id`.

## Back up and upgrade a BBS installation

Back up these BBS-specific files together and keep the backup private:

- `cepheus-trader.conf`;
- `cepheus-trader.credential`; and
- `cepheus-trader.identities`.

Prevent new door sessions while taking or restoring this backup. Preserve the
credential's ownership and access controls. The identity registry is necessary
to reconnect existing BBS accounts to their established game identities.

To upgrade, stop new door launches, extract the new release into a separate
directory, and replace the door, sysop utility, and shared client-core library
as one matching set. Preserve the three BBS-specific files and the OpenDoors
configuration. Run both `--version` checks before reopening the door.

## Troubleshooting

### The shared client-core library cannot be loaded

Restore the shared `.so`, `.dylib`, or `.dll` from the same client archive and
place it beside both executables. Do not mix files from different releases or
architectures.

### The shared configuration cannot be opened

Confirm that the BBS starts the door in the installation directory containing
`cepheus-trader.conf`, and that its account can traverse and read that
directory.

### The credential is rejected

Restore the original credential rather than creating another one. On Unix,
confirm that it is a regular file owned by the door account with mode `0600`
and a single hard link. On Windows, confirm that it has not become a reparse
point and retains its protected access control list.

### The server connection fails

Confirm DNS resolution for `ct.bbsdev.net` and outbound TCP access to port
`7323` for the player door and port `7325` for the sysop utility. Keep the
hostname in the configuration so the client can use the IPv4 or IPv6 addresses
published for the service.

If the door reports that its CT-RPC version is no longer supported, install the
current client archive and replace the door, sysop utility, and shared
client-core library as one matching set.

### A renamed or renumbered BBS account is rejected

Run `identity-list`, identify the established numeric player ID, and use
`identity-rename` or `identity-reindex` to make the registry match the current
BBS account. Do not retire and recreate the mapping for an ordinary rename.
