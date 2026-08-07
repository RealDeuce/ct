# Non-Interactive Universe Tour

## Status

The first mail-and-traffic simulation substrate is implemented. It is an
authoritative server domain exercised by the `universe_tour` executable, not a
report script that reimplements universe rules. The authoritative store and
interactive door now add the complete generic trade-goods catalogue,
persistent titled player cargo and Tasks, and one safe-locus/Jump/docking
itinerary, verified by store integration tests. The tour report does not yet
provision and print that player circuit.
Player-carried mailbag custody and arrival presentation are implemented and
covered by authoritative store integration tests; ordinary passenger carriage
is now part of the player Task economy rather than this background tour.

## Implemented Records

The server now persists:

- one next `SystemDay` event for every materialized system;
- immutable messages with origin system, creation time, class, subject, and
  absolute expiry time;
- route-specific delivery envelopes, allowing one news or public-service
  message to fan out without copying its immutable content;
- beacon queues keyed by current system and exact next hop;
- ordinary simulated traffic ships with origin, destination, departure,
  arrival, status, and optional mailbag;
- sealed mailbags containing bounded lists of delivery envelopes;
- carrier custody legs naming the specific ship, bag, endpoints, custody time,
  due time, and delivery time;
- player-carrier legs with advertised stipend and exactly-once payment time,
  plus player arrival receipts, message classifications, feed cursors, and
  system-mapping disclosure; and
- local message deliveries, including immediate availability at the origin.

Every background departure is a real persistent traffic-ship record. It takes
up to 512 eligible envelopes from the beacon queue for its exact route, seals
a bag, and signs a custody leg. Its arrival is a separately scheduled event
one standard Jump week later. Arrival either delivers an envelope at its final
system, queues it for the next route hop, or records expiry. Empty ordinary
ships still travel but do not create fictional mailbags.

The custody audit checks every carrier leg against its ship and mailbag,
including endpoints and all three times. Messages and expired envelopes remain
in the archive. Route and envelope state, rather than a global “mail arrived”
counter, is the mechanism by which information moves.

## Scheduling and Recovery

Future work is indexed by due game second. When work becomes eligible, the
scheduler admits the work itself into the same durable ordered input queue as
player commands; event kind supplies no priority. Simultaneously eligible work
uses its stable global creation ID only to make admission order deterministic.
Each admitted item currently commits as its own LMDB transaction and receives
the next authoritative sequence and state revision. The engine refuses
explicit time advancement while durable player ingress remains queued. A
command cannot therefore use the tour interface to jump ahead of accepted
commands.

System-day draws are derived from a domain-separated stream keyed by the
persisted system seed and logical day. Reads do not consume or reroll it.
Messages, envelopes, ships, bags, custody, deliveries, scheduled successors,
game time, and the journal commit atomically. The deterministic test runs two
fresh databases from the same seeds, compares their reports and custody legs,
then reopens one and compares the committed state again.

No event aggregation or transaction batching has been adopted. Performance
measurements expose the current semantics; they do not authorize changing
them.

The interactive server now drives this same scheduler at 28 game seconds per
real second. A single coalescing pulse source admits bounded due work to the
authoritative engine queue. It does not multiply with sessions or BBSs,
and server downtime does not advance the game. The tour retains explicit
advancement because it is a deterministic capacity instrument.

## Initial Calibration Model

This is a first measurement model, not settled game balance.

Traffic frequency is derived from CE population, TL, and starport. The
population table ranges from extremely sparse calls at population 0–3 through
busy capital traffic at population 9–10. TL applies a 50%–125% factor and
starport class applies a 25%–150% factor. Fractional daily rates are resolved
deterministically from the system/day stream.

Daily game-visible message rates are presently:

- agency news: `population × 0.12` unique items/day;
- public service: `0.08/day` at population 6+, otherwise `0.01/day`;
- contract offers: `max(population - 3, 0) × 0.20/day`;
- traffic notices: `0.25` per generated departure; and
- private fixed-system traffic: `0.20/day + 0.35` per generated departure.

Agency news and public service fan out throughout the connected polity.
Contracts fan out to systems no more than two J-2 route hops away. Traffic
notices go to immediate J-2 neighbors. Each private item chooses one reachable
system. Every item is immediately available at its origin. These scopes are
deliberately visible constants to calibrate before the full news-significance,
contract, recipient, paid-route, and mobile-hold-sphere models replace them.

## CPU and Progression Instrumentation

The tour measures calling-thread CPU time with
`CLOCK_THREAD_CPUTIME_ID` on POSIX and `GetThreadTimes` on Windows. It also
measures elapsed wall time separately. Per-system rows include CPU time,
`SystemDay` count, departures, arrivals, and CPU per system-day. Aggregate
output includes:

- events per wall second;
- system-days per CPU second;
- whole-universe days per CPU second; and
- whole-universe days per wall second.

“Whole-universe day” means one processed `SystemDay` for every system in the
reported universe. The reported rate is an observed no-player-load ceiling for
that build, database, host, universe, and calibration. It is not a promise or
an automatic batching threshold.

The 2026-07-31 release-build baseline for the deterministic 35-system initial
Federation advanced 28 calendar days and processed the initial day-zero tick
plus 28 successors:

| Measurement | Result |
|---|---:|
| Ordered events | 8,219 |
| `SystemDay` events | 1,015 |
| Departures | 4,114 |
| Arrivals | 3,090 |
| Calling-thread CPU | 2.853 seconds |
| Wall time with durable commits | 31.183 seconds |
| System-days/CPU-second | 355.824 |
| 35-system universe-days/CPU-second | 10.166 |
| 35-system universe-days/wall-second | 0.930 |
| Unique messages | 3,777 |
| Remote delivery envelopes | 16,113 |
| Nonempty mailbags/custody legs | 1,082 |
| Delivered remote envelopes | 7,415 |
| Expired remote envelopes | 232 |

The per-system profile is intentionally uneven. In this sample, low-activity
systems consumed roughly 10–20 CPU ms over the run, while the busiest generated
system consumed about 454 CPU ms. The tour prints all system rows so universe
size alone is never mistaken for simulation cost.

## Running the Tour

From `server/`:

```console
cargo run --release --bin cepheus-trader-universe-tour -- \
  --data /an/explicit/tour/database --days 28 --show-legs 8
```

`--data` is mandatory. The executable initializes a deterministic Federation
only when the database contains no systems. It never resets or deletes an
existing database. Running it again advances the same committed universe. At
the end it audits custody, closes the store, reopens it, and compares the
complete cumulative report.

`--bbs-count N` provisions and configures that many deterministic BBS polities
before initializing an empty database. It never adds or removes BBSs in an
existing database. `--settlement-edge` materializes the Sol-centered `3E`
capacity envelope using the furthest BBS prime as `E`; it therefore requires at
least one BBS. `--initialize-only` performs and reopens that state without
processing the due day-zero jobs. `--max-events N` stops an attempted advance
after exactly that many individually committed events, leaving a valid prefix
for capacity measurement. `--map-size-gib N` selects the sparse LMDB
address-space ceiling; it does not preallocate or report that amount as used
storage. `--list-systems` prints every generated system;
`--show-system-cpu N|all` controls the descending per-system CPU table. The
tour also reports LMDB's actual data-file bytes before and after advancement,
the byte delta, and growth per processed system-day. Growth per completed
calendar day is printed only when the requested target was actually reached.

The deterministic ten-BBS fixture first materializes 145 conditioned systems.
Its furthest BBS prime fixes `E = 28.993534 pc`; normal CE population applies
through `2E = 57.987068 pc`, the adopted linear seed-conditioned falloff spans
`2E`--`3E`, and ordinary population is zero at the `86.980601 pc` edge. The
inhomogeneous Poisson realization uses the implemented Galactic density and a
conservative analytic rejection bound. The 2026-07-31 release run produced:

| Initialization measurement | Result |
|---|---:|
| Homogeneous candidates sampled | 664,672 |
| Stellar components added | 238,667 |
| Total systems including conditioned bootstrap | 238,812 |
| Added inhabited primary worlds | 135,224 |
| Added systems in the falloff band | 166,299 |
| Added worlds forced to CE Population 0 | 93,580 |
| Quarter-parsec coverage cells resolved | 177,307,145 |
| Materialization thread CPU | 13.772 seconds |
| Materialization wall time | 13.961 seconds |
| LMDB bytes after initialization | 77,991,936 |
| LMDB byte growth during bulk materialization | 77,434,880 |

The reopen check recovered all 238,812 simulation systems exactly.

### One-year capacity attempt

The year target is day 365. Including the due day-zero job, that is
`238,812 × 366 = 87,405,192` system-day transactions before accounting for
departures and arrivals. A bounded, durable prefix of 10,000 day-zero jobs
took 39.572 wall seconds and 3.220 thread-CPU seconds:

| Prefix measurement | Result |
|---|---:|
| Committed events/system-days | 10,000 |
| Events per wall second | 252.704 |
| System-days per CPU second | 3,106.052 |
| Retained byte growth | 5,914,624 |
| Retained bytes per system-day | 591.462 |
| Unique messages generated | 32,493 |
| Remote envelopes generated | 3,500 |
| Future departures scheduled | 38,508 |
| Physical scheduled-event records | 241,341 |

At the observed rate, the system-day transactions alone are a lower bound of
4.00 wall-days and 7.82 CPU-hours. Straight-line retained growth from that
prefix is a lower bound of 51.697 GB and about 284 million messages for the
year; it excludes every traffic ship, mailbag, custody leg, and arrival.
The prefix scheduled 3.8508 departures per system-day. Extending that rate
through 365 departure days and 358 arrival-eligible days gives a first-order
total of about 752.3 million individually committed events and 34.5 wall-days
at the prefix event rate. This is a projection, not a completed checkpoint:
departure work, LMDB growth, cache behavior, and mail queues can change the
rate substantially. It is sufficient evidence that the present fine-grained
model cannot practically reach the one-year checkpoint unchanged.

### Storage-schema optimization pass

The compact representation is deliberately below the game-state contract:

- coverage chunks select a full sentinel, sparse set list, sparse clear list,
  or raw bitmap, whichever is smallest;
- record values omit identifiers already present in their LMDB keys, and
  deterministic `Frontier N` names use compact tags rather than repeated text;
- scheduled-event values omit due time and event ID already present in their
  ordered keys; entity IDs remain in values, and category priority does not
  exist;
- the simulator journal stores typed binary facts rather than diagnostic prose;
- origin-system message availability is inferred from the immutable message,
  rather than duplicated as a delivery row; and
- all departures produced by one daily system job are stored as one ordered
  plan. At each due time it yields exactly one ordinary departure transaction
  and stores the remainder at the next due time. Reports expand plans to their
  logical event count.

The optimized database had 241,341 physical scheduled-event records after the
prefix: 238,812 daily jobs plus 2,529 departure plans. Those plans represented
38,508 logical future departures, so the public report still showed 277,320
scheduled events.

### Deployment-budget interpretation

The 238,812-system settlement-edge fixture is a deliberately materialized
capacity ceiling, not the expected state of a ten-BBS game. The deployment
target is approximately USD 50 per month for ten BBSs and fifty active players
on a mainstream cloud provider. The production universe therefore remains
generate-on-demand and should provision storage incrementally rather than
allocating the projected fully explored footprint at startup.

Capacity-envelope rasterization is deliberately excluded from ordinary
`cargo test` runs. Run its ignored check explicitly with
`cargo test settlement_footprint_supports_the_capacity_envelope -- --ignored`,
and use the release-mode `cepheus-trader-universe-tour --settlement-edge`
workflow above for meaningful CPU, storage, and progression measurements.
Neither belongs in the normal CI latency budget.

At the measured two-real-year projection, 10% materialization is roughly
290 GB while 25% is roughly 720 GB; full materialization is roughly 2.89 TB.
Those figures remain lower bounds for the persistence types not yet
implemented. Production telemetry must report materialized-system count,
retained bytes per game day and real day, event throughput, and projected
budget exhaustion. The cost target is a design constraint, not permission to
discard durable facts or weaken the established simulation contract.

At its initial day-zero tick, the older unexpanded 145-system fixture used
2,269,184 LMDB data bytes after generating 1,118 messages, 3,741 remote
envelopes, 73 traffic ships, and 65 mailbags. This remains a small-universe
diagnostic datum, not a projection.

## Remaining Complete-Tour Work

- Extend the implemented Common Goods daily stock, persistent cargo lots, and
  exactly-once loading/sale boundary to Trade Goods, reservations, delivery,
  and background consumption without statistical duplication.
- Add passengers and their abandonment/expiry behavior.
- Print the implemented player ship, cargo, fuel, safe-locus approach, Jump,
  arrival, and docking itinerary in the tour; add player mail custody.
- Show all five traffic-location classes and persistent sparse-route calls.
- Demonstrate that inert passage changes neither cargo nor mail, while an
  accepted player-like cargo lot or mailbag changes custody exactly once.
- Extend the live-clock tests and operator telemetry as encounter frames and
  player-carried mail add new scheduler work. The server now drains work due
  at committed logical time around every ingress batch and advances at four
  game weeks per real day through one coalescing pulse source; the tour keeps
  explicit advancement for deterministic capacity measurements.
