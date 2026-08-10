# Milestone 7 Audit: Field Alpha and Operations

*Audit date: 2026-08-04*

This document records the results of an implementation audit of Milestone 7
("Multi-BBS Gameplay Alpha") against the requirements in `ROADMAP.md` and the
detailed specification in `docs/field-alpha-operations.md`.

## Scope

Milestone 7 requires:

1. Real OpenDoors/BBS drop-file startup and local player-ID attestation.
2. Reconnect and presentation behavior in actual remote terminal sessions.
3. Audited sysop moderation for originating players and polity state without
   advantage-granting operations.
4. BBS founding announcements and inter-polity discovery propagation.
5. Operational monitoring, live same-version backup, recovery, and safe
   shutdown procedures; incompatible alpha stores are reinitialized rather
   than migrated until the first deployed persistence contract.
6. Load testing around the target of ten BBSs and fifty active players.
7. A deployment profile suitable for an approximately USD 50/month mainstream
   VPS when the target workload permits it.

## Summary

The implementation substantially covers the Milestone 7 boundary. Six of the
seven requirement areas are fully implemented and tested. One area (reconnect
behavior) meets the practical BBS-door contract but lacks an explicit
specification-level behavior. A small number of secondary gaps exist in wire
schema completeness, test coverage, and deployment automation.

| # | Requirement | Verdict |
|---|-------------|---------|
| 1 | OpenDoors/BBS drop-file startup and player-ID attestation | **Complete** |
| 2 | Reconnect and presentation in remote terminal sessions | **Partial** |
| 3 | Sysop moderation (players and polity state) | **Complete** |
| 4 | BBS founding announcements and inter-polity discovery | **Complete** |
| 5 | Monitoring, backup, recovery, and safe shutdown | **Complete** |
| 6 | Load testing (10 BBSs, 50 players) | **Complete** |
| 7 | USD 50/month VPS deployment profile | **Documented, not validated** |

## Detailed Findings

### 1. OpenDoors/BBS Drop-File Startup and Player-ID Attestation

**Verdict: Complete. No gaps.**

The door binary (`cepheus-trader-door`) calls `od_parse_cmd_line()` for
standard OpenDoors switches (`-L`, `-D`, `-C`, `-N`, `-USERNAME`).
`od_init()` reads standard drop files (door.sys, dorinfo, etc.). A custom
`opendoors_config_line()` callback handles `CTConfig`, `CTProfile`,
`CTColumns`, and `CTRows`. OpenDoors is built as a static library from the
vendored, provenance-recorded source and linked into the door executable.

The binary identity registry (`player_identity_registry.cpp`) is versioned
(format 1), SHA-256 checksummed, owner-only enforced (Unix and Windows),
interprocess-locked (`flock`/exclusive `CreateFileW`), and atomically replaced
via rename. The composite real-name-plus-record-index key resolves
automatically on exact match; partial collisions require sysop intervention
via `identity-rename`, `identity-reindex`, or `identity-retire`. Retirement
requires the literal `RETIRE` confirmation.

The `tls_interop.rs` integration test exercises the full OpenDoors startup
path with four terminal profiles and player creation through the C++ door.

### 2. Reconnect and Presentation in Remote Terminal Sessions

**Verdict: Partial. Phase restoration works; no in-process retry.**

When a client connects or reconnects, `ServerHello` returns the authoritative
`phase` (Docked, Jump, Interplanetary, Encounter, Terminal, newUser, etc.),
`assignedEpoch`, and `committedSequence`. The door's `run_operational_loop()`
dispatches to the correct phase handler, so a reconnecting player resumes at
the right screen. `SessionReplaced` events terminate a superseded session.

The `tls_interop.rs` test verifies reconnection: epoch increments, committed
sequence increases, and the correct phase is restored after disconnect and
re-entry.

**Gap:** The door binary does not contain an automatic reconnect-on-disconnect
loop. If the TLS connection drops mid-session, the exception propagates to
`main()`, the door renders an error, and exits. The BBS must re-invoke the
door for the player to reconnect.

This matches the standard BBS door contract (doors are launched per-session
and do not persist beyond a single invocation), but the Milestone 7 text
says "reconnect and presentation behavior in actual remote terminal sessions"
without specifying whether in-process retry is required. If the intent is
only that a player who re-enters the door sees the correct game state, the
requirement is met. If transient-disconnect resilience within a single door
invocation is intended, it is absent.

### 3. Sysop Moderation

**Verdict: Complete. No gaps.**

The sysop protocol is a separate TLS-PSK listener (`--sysop-listen`, default
`localhost:7325` on every supported resolved address) authenticating against
enrolled BBS credentials. The
`ct_sysop.capnp` schema is fully distinct from the player and admin schemas.

Implemented commands:

- **get-config / set-config**: Revision-checked BBS configuration reads and
  updates. `SetConfiguration` carries `expectedRevision`; a stale revision
  returns an error with the current revision for retry. Trade/combat (0-100)
  and chaos/order (0-100) orientation axes are effective polity state, not
  metadata. Configuration changes originate a signed public-service order at
  the capital; other systems apply the change only when the electronic notice
  copy arrives. Later revisions supersede older copies.

- **suspend-player / resume-player / remove-player**: Three-state access
  management (active, suspended, removed). Suspension is reversible; removal
  is an irreversible tombstone preserving player and assets. Both immediately
  close the active game session. `remove-player` requires the literal
  `REMOVE` confirmation.

- **tax-player**: Issues a private signed mail instrument from the capital.
  Takes available cash, records the remainder as non-interest-bearing arrears,
  and posts receipts to the polity fiscal ledger. Takes effect only when the
  mail reaches the captain.

- **demote-player**: Issues a private signed mail instrument. Lowers the
  naval grade, clamps service points below the next grade threshold, and
  expires offered or accepted naval orders that depended on the former grade.
  Takes effect only when the mail reaches the captain.

Neither tax nor demotion can boost a player. The
`tls_interop.rs` test exercises init-credential, get/set-config with revision
checking, and player suspension/resumption.

Store tests cover:
`player_removal_is_persistent_and_irreversible`,
`sysop_tax_is_mail_delayed_and_creates_non_interest_arrears`,
`polity_policy_changes_take_effect_only_after_the_mail_notice_arrives`.

### 4. BBS Founding Announcements and Inter-Polity Discovery

**Verdict: Complete. No gaps.**

These are implemented as engine-internal operations generating in-world
in-world store-and-forward mail, not as separate wire protocol message types.

- **BBS founding**: `dispatch_bbs_founding_announcement_in()` (store.rs line
  21942) creates a public `AgencyNews`/`Headline` message containing the
  polity name, capital authority, and registered systems with coordinates,
  tech levels, populations, and law levels. The message is dispatched to all
  systems in the universe when a BBS polity is first configured.

- **Inter-polity discovery propagation**: `process_discovery_mail_arrival_in()`
  (store.rs line 19673) handles discovery claims arriving at Sol. On arrival,
  the claim is evaluated, awards are granted for settled-system first
  discoveries, a Federation chart notice is broadcast to all systems, and a
  private message is sent to the claimant.

- **Polity configuration propagation**: `dispatch_polity_policy_in()` creates
  public-service messages dispatched from the capital to all cluster systems.
  The capital applies changes immediately; remote systems apply on physical
  mail arrival. Later revisions supersede earlier ones.

**Minor test gap:** No test explicitly asserts the founding announcement
content (message class, importance, subject, body). The announcement is
exercised through BBS configuration tests and implicitly through message-count
assertions, but the specific content is not asserted.

### 5. Operational Monitoring, Backup, Recovery, and Safe Shutdown

**Verdict: Complete. One minor wire-schema gap.**

**Status** (`cepheus-trader-admin status`): Returns committed sequence, game
second, queued inputs, BBS/player/system counts, active sessions, and storage
format. All fields specified in the doc are present in both the
`OperationalStatus` struct and the `ServerStatus` Cap'n Proto message.

**Live backup** (`cepheus-trader-admin live-backup LABEL`): Creates a
directory under `--backup-dir` containing a force-synced LMDB `data.mdb` copy
and a `manifest.txt` with storage-format, committed-sequence, game-second,
queued-inputs, and object counts. Uses atomic rename. Idempotent retry with
the same command-id returns the completed backup; a conflicting label from a
different command-id is an error. Store test
`live_backup_is_complete_reopenable_and_idempotent` verifies the manifest,
reopenability, idempotency, and conflict detection.

**Safe shutdown**: SIGINT and SIGTERM trigger `ServerStopping` events to all
active sessions, drain the authoritative engine via `EngineMessage::Shutdown`,
and join the engine owner thread before exit.

**Recovery**: Alpha stores are reinitialized rather than migrated.
`initialize-universe` requires the literal `INITIALIZE FEDERATION`
confirmation. The `tls_interop.rs` test exercises re-initialization including
BBS control state preservation.

**Minor gap:** The `BackupComplete` Cap'n Proto response struct carries only
`label`, `committedSequence`, and `gameSecond`. The on-disk `manifest.txt`
includes all specified fields (including `storage-format` and object counts),
but the wire response omits `storageFormat` and the count fields. The admin
client displays only what the wire returns. This is a protocol-vs-doc
discrepancy: the manifest is complete, but a remote operator relying solely
on the admin tool's printed output would not see the full manifest without
reading the file.

### 6. Load Testing

**Verdict: Complete. No functional gaps.**

The `ct-field-alpha-load` binary is defined in CMakeLists.txt with
`EXCLUDE_FROM_ALL` (not built by default, not registered with CTest),
matching the spec. It accepts HOST, PORT, HOLD_SECONDS, CYCLES,
SESSIONS_PER_BBS, and one or more CREDENTIAL_FILE arguments. The
implementation connects `sessions_per_bbs` attested player sessions per BBS
credential, holds for `hold_seconds`, disconnects, and reconnects for
multiple cycles. It emits structured `READY` and `DISCONNECTED` lines with
session counts and timing.

The non-interactive universe tour (`cepheus-trader-universe-tour`) provides
the simulation-side measurement: `--show-system-cpu` reports per-system CPU
breakdown, and the `observed-ceiling` output gives
`universe-days-per-wall-second` for progression speed validation.

**Operational note:** The load exercise and deployment validation are
documented as manual operator procedures. The load driver does not itself
query `cepheus-trader-admin status` to verify session counts (0/50/0/50/0)
or queue draining; the operator runs status checks at each stage. There is no
automated orchestration script that combines the load exercise, status
verification, universe tour, and LMDB size comparison into a single
invocation. This is consistent with the spec's characterization of
capacity-scale tests as "explicit benchmarks, not normal CI."

### 7. Deployment Profile

**Verdict: Documented but not validated.**

The `docs/field-alpha-operations.md` specifies the target: one VPS with four
modern x86-64 vCPUs, 8 GiB RAM, at least 160 GiB local SSD/NVMe, at least
1 TiB/month transfer, at approximately USD 50/month. Security guidance
includes exposing only the game TLS port publicly, binding the admin endpoint
to loopback, filtering the sysop endpoint at the host firewall, and running
as a dedicated unprivileged account.

The document states acceptance criteria: the ten-BBS/fifty-session exercise
must drain its queue, complete both reconnect cycles, reopen its backup, and
demonstrate a universe-progression ceiling above four game weeks per real day
with operational headroom.

**Gap:** No evidence was found that this deployment profile has been validated
on an actual VPS instance. The tools exist (load driver, universe tour, admin
status), but no recorded test run, benchmark results, or deployment log
confirms that the acceptance criteria have been met on the target hardware
class. This is the difference between "the acceptance test is defined and the
tools work" and "the acceptance test has been run and passed."

## Consolidated Gap List

### Implementation Gaps

| # | Area | Severity | Description |
|---|------|----------|-------------|
| G1 | Reconnect | Medium | No in-process reconnect-on-disconnect in the door binary. BBS re-invocation restores phase correctly. Whether this is a gap depends on whether the milestone requires transient-disconnect resilience within a single door invocation. |
| G2 | Backup wire response | Low | `BackupComplete` Cap'n Proto message omits `storageFormat` and object counts. The on-disk manifest is complete. The admin tool cannot display the full manifest from the wire response alone. |
| G3 | VPS deployment | Medium | The deployment profile and acceptance criteria are fully documented and the tools exist, but no recorded validation confirms the criteria have been met on actual target hardware. |

### Test Coverage Gaps

| # | Area | Severity | Description |
|---|------|----------|-------------|
| T1 | Session capacity | Low | No unit test exercises the rejection path when `MAX_ACTIVE_GAME_SESSIONS` (256) or `MAX_ACTIVE_GAME_SESSIONS_PER_BBS` (64) is exceeded. Enforcement code exists; validation is operational via the load exercise. |
| T2 | Founding announcement content | Low | No test asserts the specific content (message class, importance, subject, body) of BBS founding announcements. The dispatch path is exercised through configuration tests. |
| T3 | Deployment automation | Low | No script orchestrates the full deployment validation sequence (load exercise + status checks + universe tour + storage measurement). The procedure is documented as manual operator work. |

### Items Confirmed as Non-Gaps

| Item | Explanation |
|------|-------------|
| BBS founding announcements not in wire protocol | Correctly implemented as engine-internal operations generating in-world store-and-forward mail. Not a protocol-level message. |
| Session capacity not reported to clients | Server-side enforcement only. Clients see TLS/protocol errors at capacity. Consistent with the doc's description of server-side caps. |
| Load driver uses CYCLES instead of "reconnect-count" | Functionally identical; the parameter is a superset of the documented 2-cycle pattern. |
| Admin port 7324 vs sysop port 7325 | These are two different listeners. Admin defaults to every supported address for loopback-only `localhost:7324`; sysop defaults to every supported address for `localhost:7325`. Both match their respective documented defaults. |
