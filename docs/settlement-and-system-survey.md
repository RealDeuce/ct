# Settlement Envelope and System Survey

*Status: design decisions recorded; the settlement falloff and deterministic
capacity fixture are implemented, while survey gameplay, Federation discovery
awards, live frontier integration, and colonization remain incomplete,
2026-07-31*

This document defines how an arriving ship can discover a previously
unvisited system, where ordinary inhabited worlds can be generated, and what
players can eventually do with otherwise empty systems.

## No precursor survey network

The setting does not contain a galaxy-spanning network of self-replicating
probes. In particular, system materialization must not assume that a von
Neumann swarm visited the system first and left a data beacon.

Self-replicating probes create more setting problems than they solve:

- their launch date and propagation frontier would impose a hard boundary on
  exploration;
- ownership, security, corruption, malfunction, and ecological effects would
  become dominant setting concerns;
- newly materialized chains of systems would still require an explanation for
  who observed them; and
- universal infrastructure would make genuinely unvisited systems difficult
  to justify.

Ordinary navigation and survey beacons may be constructed locally. They can
improve later observations, but they are not prerequisites for discovering a
system and do not reproduce themselves.

## First-arrival survey

Remote astronomy can provide a target star and an approximate stellar model.
The arriving ship performs the local survey. No observer or beacon already
inside the system is required.

The *Unmerciful Frontier* survey timings provide useful layers:

| Survey layer | Typical time | Game meaning |
|---|---:|---|
| Basic system scan | 10–60 minutes | Detect the major mass concentrations and gross system architecture |
| Survey sensors and probes | 10–60 hours | Improve body identification and orbital solutions |
| Detailed planetary survey | 1–6 days | Determine atmosphere, hydrographics, and other useful planetary facts |
| Life survey | 1–6 weeks | Search seriously for native life |

The 10–60 minute scan is not a perfect catalogue. A useful progression is:

1. Minutes reveal the star, major planets, gas giants, and immediate hazards.
2. Hours to days refine intercepts, orbits, moons, and planetary properties.
3. Days to weeks establish biological and resource information.
4. Small bodies and a mature navigation catalogue can continue improving for
   months or years.

This permits a ship to enter a genuinely unknown system without requiring
magic sensors while avoiding months of blind searching for the major planets.

## Federation Discovery Award

Exploration is a minor but economically rational activity. The Federation
maintains a standing award for the first valid notification received on Earth
of a previously unknown settled system. No mission or contract must be
accepted in advance.

Discovery does not force disclosure. After the first local scan, the captain
may broadcast a public mapping notification, send an encrypted direct filing
to Earth, withhold it, or withhold it and place the system on the captain's
Secret Systems list. Either sent form can be the filing. To be valid for the
award it must contain:

- the canonical three-dimensional position and enough stellar observations to
  distinguish aliases and duplicate reports;
- authenticated claimant and discovering-ship identities;
- observation and dispatch times plus custody and mail-route provenance; and
- sufficient survey evidence that at least one world has an established
  population, rather than a transient ship, cache, camp, or newly founded
  player base.

The authoritative race ends when a valid filing commits to the Federation's
Earth repository. Discovery time, dispatch time, arrival at a nearer
Federation system, and a claimant's physical return do not determine priority.
The first valid Earth receipt wins; simultaneous receipts use normal engine
queue order and stable message IDs. An invalid or incomplete filing neither
wins nor reserves the system.

Earth validates against the Federation repository as it existed immediately
before receipt, not against the server's omniscient materialized-system table.
If an earlier structured package already taught Earth about the settlement,
its human-readable notice may have been hidden by message filters, but the
later filing is not first. Canonical coordinate and stellar matching prevents
renaming or submitting a second report about another world in the same system.
BBS/bootstrap materialization and administrative polity announcements are not
player exploration claims.

### Award value

The award is intentionally a cost-covering bonus, not a major progression
engine. Its published value is:

```text
award = round_up_to_Cr1,000(1.10 × reference_two_jump_cost)
```

The initial 10% margin is the concrete meaning of “slightly more” and is a
balance parameter. The reference vessel is the active `ship-72` Smollett, the
privateer offer in the orderly/mixed polity cell: the locally aligned
privateer in a well-enforced region. The benchmark is two complete standard
port-to-port Jump-2 legs, approximately one four-week operating month under
the adopted one-week local plus one-week Jump cadence for each leg.

`reference_two_jump_cost` includes the ordinary costs borne by that offer's
operator:

- refined fuel for two maximum-range Jump-2 transits and the associated
  power-plant endurance;
- normal crew compensation and life support for the reference period;
- accrued routine maintenance;
- two ordinary Jump-2 plots or their standard navigation-data equivalent; and
- ordinary departure, arrival, and berth charges included in the reference
  itinerary.

It expressly excludes mortgage principal, interest, charter payments, debt
service, depreciation, and every other capital or financing cost. It also
excludes ammunition expenditure, combat damage, misjump loss, bribes, fines,
speculative-cargo capital, extraordinary repairs, and claimant-specific
detours or luxury spending. Free skimming or unusually favorable employment
terms do not reduce the published award. The active Smollett construction
model is Cr229,220,000, which supplies the maintenance basis, but the first
exact credit award waits only on completing the direct operating-cost audit.

The award schedule in force when the filing reaches Earth controls the amount.
That schedule itself propagates outward by mail, so a distant explorer may be
working from an older advertised figure.

### Payment and notification

Payment is committed on Earth at the same game time as successful
adjudication. The claimant need not return and may continue exploring while
the filing travels. Earth credits the claimant's Federation account or creates
a payable claim there; it does not create instantly spendable funds aboard a
distant ship. The award decision, receipt, and any requested banking transfer
return through physical mail and ordinary interstellar clearing.

A public notification begins its free public-service propagation from the
ship's current system and continues whether or not it later earns an award. A
direct notification is private paid mail to Earth; intermediate systems do not
learn its observations. If the direct filing wins, Earth pays and originates
the free authoritative public mapping announcement, so public propagation
begins on Earth at award time. Routine public discovery notices remain hidden
by default Message Management filters; a claimant can explicitly enable
discovery and award-result notifications. Mail classes, direct-route charges,
encryption, and key compromise are specified in
[`mail-service-and-security.md`](mail-service-and-security.md).

## Mapping Disclosure and Secret Systems

Arriving in a system that the active ship does not know to be publicly mapped
creates a disclosure decision after the basic local scan. The UI must not ask
the authoritative Earth repository whether the system is already known; that
would be an ansible. It asks based on the current ship and captain's delayed
knowledge and clearly warns that another filing may already be in transit.

The arrival prompt provides:

1. **Send Public Notification:** broadcast the structured observations as a
   free public-service message from the current system. This is also a bounty
   filing when it reaches Earth.
2. **Send Direct Notification:** encrypt the observations to the Federation
   discovery office and submit paid point-to-point mail to Earth. Nothing is
   added to intermediate Known Universe repositories; a winning award causes
   Earth to originate the public announcement.
3. **Do not send:** retain the observations locally without creating outbound
   mail. The question may be presented again on a later visit.
4. **Do not send and mark secret:** retain the observations locally and add
   the system directly to Secret Systems, suppressing later automatic prompts
   and mapping notifications for this captain.

No response defaults to withholding; a disconnect or timeout must never
publish a system accidentally. If the current continuation plan would
otherwise leave the system automatically, this unresolved prompt suspends the
plan before any notification is emitted.

Secret Systems is an editable captain-private list reachable through Known
Universe at any time. Entries identify a locally known system and may carry a
private note, creation time, and the repository observation that established
the identity. Adding an entry does not delete local navigation knowledge.
Removing one does not send anything automatically; the captain may then send
the mapping notification explicitly or answer the prompt on a later arrival.
Adding a system after a filing was dispatched cannot recall mail already in
flight or prevent Earth from publishing a winning direct filing.

The list travels physically with the captain's private repository. It is not
an account-global list visible aboard distant player-owned ships, because its
contents would themselves reveal secret systems. It controls only this
captain's disclosure behavior; another player, NPC, institution, or ship may
discover and report the same system independently.

## Densitometer interpretation

The Cepheus Engine densitometer is treated as fictional gravitic tomography,
not ordinary radar. A survey ship points it at the system, models and
subtracts the known stellar mass, and searches the residual field for
concentrated mass.

Two scaling rules are useful:

- Simple mass presence scales approximately as `signal ∝ mass / range²`, so
  detection range scales as the square root of effective mass.
- Resolving internal density structure is a gradient problem and scales more
  nearly as `anomaly ∝ density contrast mass / range³`, so useful range scales
  as the cube root of the effective anomaly.

For similarly composed bodies, the second relationship makes coarse
structure range roughly proportional to physical size. Signal processing and
the Mineralogy Suite can improve interpretation, but they do not remove
uncertainty.

The rules still need a game calibration point: a steel-equivalent detectable
mass or volume at a reference range. Any Earth-core or interstellar ranges
derived before that calibration are illustrations, not adopted sensor
constants.

## Historical settlement envelope

The normal settled region uses the Cepheus Engine population generation
unchanged. There is no additional inhabited-system roll. An exhaustive
enumeration of the current population table and environmental modifiers gives
approximately:

| Result | Probability |
|---|---:|
| Uninhabited | 6.767% |
| Inhabited | 93.233% |
| Population 4+ (at least tens of thousands) | 67.342% |
| Population 6+ (at least millions) | 41.483% |
| Population 9+ (at least billions) | 11.050% |

This high inhabited rate is intentional inside the setting's established
human reach.

Let:

- `E₀` be the initial Federation's settlement extent from Earth; and
- `E` be the greater of `E₀` and the distance from Earth to the furthest BBS
  polity's prime system.

Population generation then follows these rules:

1. At distances through `2E`, use the normal CE population roll and modifiers.
2. Across `2E < r < 3E`, linearly mix ordinary CE seeds with CE seeds
   conditioned to Population 0. The conditioned fraction is `(r - 2E) / E`.
3. At `3E`, ordinary generated population is zero.
4. A newly added BBS polity is a conditioned exception and can expand `E`.
5. Systems already materialized never have their historical population
   rewritten merely because `E` later grows.

This represents discovery of the extent of historical settlement, not people
appearing when a BBS is added.

Conditioned seed selection freezes the result without adding mutable
population state: every primary world remains reproducible from its stored
system seed. The deterministic capacity fixture implements this full rule over
a Sol-centered `3E` sphere. Live frontier materialization must apply the same
rule using fresh operation entropy and commit new systems, coverage, and the
causing Jump transaction atomically. The initial value of `E₀` remains a tuning
decision.

## Empty systems

An uninhabited system is deliberately sparse gameplay space. Without a local
population it has no routine market, passengers, contracts, repairs, news
production, or local enforcement.

It can still support:

- hiding and covert rendezvous;
- unsupported water or gas-giant refuelling;
- route staging and double-jump operations;
- caches and pirate anchorages; and
- player activity or other transient encounters.

Such systems should impose little continuing simulation cost until an event
or player creates persistent state there.
The conditions under which through traffic or a specific destination can
still produce a contact are defined in
[`system-traffic-and-encounters.md`](system-traffic-and-encounters.md).

## Player bases and colonies

Player-built bases are a possible late-game credit sink, not part of the
initial implementation. A remote base begins as a liability requiring
construction, personnel, power, maintenance, defences, spare parts, and
recurring imports. Its early benefits are strategic: fuel, storage, repair,
concealment, and intelligence.

Existing ships can establish caches or small supplied outposts through
repeated trips. They cannot casually create a durable, independent planetary
economy.

The current catalogue illustrates the scale:

- The 800-ton Scheria colony transport carries 480 low berths and 141 tons of
  cargo. It assumes that substantial support already exists at the
  destination.
- The largest admitted freight design is the 4,500-ton Silk Road, with
  2,647.2 tons of cargo.
- The catalogue's current 5,000-ton hulls are dreadnoughts, but a clean-sheet
  5,000-ton freighter or colony ship is feasible under the construction rules.

A purpose-built 5,000-ton J-2 colony ship might provide roughly 3,000 tons of
colonist-and-cargo payload. The following are scale illustrations, not fixed
designs:

| Colonists | Low-berth mass | Remaining equipment and supplies |
|---:|---:|---:|
| 1,000 | about 500 tons | about 2,500 tons |
| 2,000 | about 1,000 tons | about 2,000 tons |
| 3,000 | about 1,500 tons | about 1,500 tons |

Packing five or six thousand sleepers would leave too little equipment to
found a useful settlement. A credible colony load instead needs habitats,
reactors, workshops, vehicles, medical and agricultural systems, industrial
seed equipment, spares, genetic archives, surveys, and follow-up shipping.

One or two thousand colonists with substantial equipment could establish a
Population 3 settlement on a suitable world. A heavily capitalized mine might
export raw material within years while remaining dependent on imports, but a
diversified, self-sustaining trade economy is a generational project. Exact
growth, supply, ownership, failure, and economic rules are intentionally left
for a later colonization design.

## Open decisions

- Select `E₀` for the initial Federation.
- Integrate the adopted `2E`--`3E` seed-conditioning rule into ordinary
  Jump-arrival materialization.
- Calibrate densitometer sensitivity and survey confidence.
- Define which detailed survey results beyond the mandatory system-discovery
  package are automatically shared, sold, or kept secret.
- Complete the Smollett direct operating-cost audit, excluding all financing,
  then publish the first exact Federation discovery award.
- Design base and colony ownership, supply, growth, failure, and abandonment
  rules before allowing players to found them.
