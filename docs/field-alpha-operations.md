# Field Alpha and Operations

Milestone 7 uses OpenDoors as the door startup and command-line authority. The
door accepts normal OpenDoors switches such as `-L`, `-D`, and `-C`; it does
not implement a second GNU-style option parser. An OpenDoors configuration can
name the shared game configuration with:

```text
CTConfig /path/to/cepheus-trader.conf
```

The shared configuration selects the protected credential and identity files,
the identity name source (`real-name` or `handle`), and optional presentation
profile/geometry. The default identity key is the BBS real name plus user
record index whenever the drop file supplies both. A missing record index falls
back to name-only identity. Local numeric player IDs are monotonic, nonzero
UInt32 values and are never reused.

The binary identity registry is bound to one BBS ID, versioned, checksummed,
owner-only, and updated under an interprocess lock by atomic replacement. An
exact composite match resolves automatically. A partial collision—same index
with a different name or same name with a different index—stops entry until the
sysop uses `identity-rename`, `identity-reindex`, or `identity-retire`.
Retirement is the BBS-account deletion operation and requires the literal
`RETIRE` confirmation.

On Windows, initial creation explicitly assigns the process user as owner.
Every later atomic replacement reproduces and verifies the established owner,
group, and protected owner/System/Administrators DACL on the temporary file
before it may replace the registry. An elevated update must not transfer
ownership to Administrators or Local System; inability to preserve the security
identity leaves the original registry intact and fails the update.

Server-side player access has active, suspended, and permanently removed
states. Suspension/resumption is reversible. Removal is an irreversible
tombstone and preserves the player and assets. A suspension or removal closes
the active game session immediately. The local BBS identity mapping is retired
separately because only the BBS knows when an account was deleted.

Tax and naval demotion are in-world official instruments, not instantaneous
remote mutation. The home authority emits a private signed message into the
normal store-and-forward mail network. The assessment takes effect when that copy
reaches the captain. Tax takes available cash, records the remainder as
non-interest-bearing arrears, and sends receipts to an unspendable polity
fiscal ledger. Demotion lowers the grade, clamps service points below the next
grade threshold, and expires offered or accepted naval orders that depended on
the former grade. These operations cannot boost a player.

The two civilization axes are also effective polity state rather than merely
creation-screen metadata. A configuration revision originates a signed
public-service order at the capital. The capital applies its local copy at
once; every other member system continues its prior revision until an ordinary
ship physically delivers that system's copy. Later revisions that arrive first
supersede older copies. The trade/combat axis weights local hostile and military
encounter classes. The chaos/order axis shifts the effective CE law level while
preserving the seed-derived differences among worlds, and also weights local
inspection, traffic-control, military, and hostile encounters. The immutable
generated UWP remains the baseline; these per-system records are the mutable
institutional overlay.

## Operator commands

The administrator channel is loopback-only TLS-PSK. Useful observations and
backup commands are:

```text
cepheus-trader-admin status
cepheus-trader-admin live-backup LABEL
```

`LABEL` is a simple ASCII label, not a path. The server writes beneath its
`--backup-dir` (default `server-backups`) at an engine-queue boundary. The
backup contains a durable LMDB `data.mdb` and a manifest with storage format,
committed sequence, game second, and object counts. A retry with the same
command ID and label returns the completed backup; a pre-existing label from a
different command is an error. Restore is an offline,
same-storage-version operation: stop the server, preserve the failed data
directory, place the backup `data.mdb` in a fresh data directory, and start the
same server build. Alpha formats have no migration path.

The server reports committed sequence, game second, durable input depth, BBS,
player, and system counts, active sessions, and storage format. It caps pending
game authentication at 64, active game sessions at 256, and active sessions per
BBS at 64. SIGINT and SIGTERM send `ServerStopping`, drain the authoritative
engine, and join its owner thread before exit.

Capacity-scale tests remain explicit benchmarks, not normal CI. The field-alpha
load exercise is ten BBSs and fifty concurrent player sessions on the intended
VPS class while recording status samples, CPU time per system, queue lag,
storage growth, and disconnect/reconnect behavior.

Build the opt-in connection driver explicitly; it is excluded from the normal
build and is not registered with CTest:

```text
cmake --build client/build --target ct-field-alpha-load
client/build/ct-field-alpha-load 127.0.0.1 7323 60 2 5 \
  bbs-1/cepheus-trader.credential bbs-2/cepheus-trader.credential \
  bbs-3/cepheus-trader.credential bbs-4/cepheus-trader.credential \
  bbs-5/cepheus-trader.credential bbs-6/cepheus-trader.credential \
  bbs-7/cepheus-trader.credential bbs-8/cepheus-trader.credential \
  bbs-9/cepheus-trader.credential bbs-10/cepheus-trader.credential
```

That invocation opens five distinct attested player IDs through each of ten
BBS credentials, holds all fifty connections for sixty seconds, disconnects
them, then reconnects the same identities for a second sixty-second sample.
Run `cepheus-trader-admin status` before `READY`, during each hold, after each
`DISCONNECTED`, and after the final cycle. The active-session readings must be
0, 50, 0, 50, and 0 respectively; durable input depth must drain rather than
grow monotonically.

Use the release-mode universe tour for the simulation-side measurements over
the same ten-BBS database. Its `--show-system-cpu all` output reports CPU time
per system and its observed-ceiling fields report aggregate maximum universe
progression speed. Record LMDB size before and after both exercises. This
separates connection/session capacity from deliberate fast-forward simulation
without weakening either measurement by running both stressors at once.

## Field-alpha deployment profile

The initial approximately USD 50/month sizing target is one general-purpose
VPS with four modern x86-64 vCPUs, 8 GiB RAM, at least 160 GiB of local SSD or
NVMe storage, and at least 1 TiB/month transfer. The game protocol is not
expected to make bandwidth the limiting resource. CPU progression rate,
durable-write latency, and retained message volume decide whether a particular
provider's plan is adequate; the price alone does not.

LMDB data must reside on the VPS's local block filesystem. Do not place the
live database on NFS, SMB, object-storage mounts, or a synchronized desktop
folder. Put `--data` and `--backup-dir` on separate local paths, then copy only
completed live-backup directories to storage in another provider failure
domain. Provider snapshots are useful in addition to the application backup,
but they do not replace the queue-boundary manifest and reopen check.

Expose only the game TLS port publicly. Bind the administrator endpoint to
loopback; expose the sysop endpoint only on the network path actually needed by
participating BBS hosts and filter it at the host firewall. Run the server as a
dedicated unprivileged account whose private data directory is owner-only.
Leave enough free local storage for one complete live backup plus normal growth
between off-host copies. A deployment is accepted for this profile only after
the ten-BBS/fifty-session exercise drains its queue, completes both reconnect
cycles, reopens its backup, and demonstrates a universe-progression ceiling
above the required four game weeks per real day with operational headroom.
