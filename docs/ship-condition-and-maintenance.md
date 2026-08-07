# Ship Condition and Maintenance

## Rules Boundary

Cepheus Engine and the admitted Clement Sector sources define several related
but different operations:

1. Space-combat Damage Control can repair one to three hits after a successful
   Mechanics check. CE explicitly calls these **battlefield repairs** and says
   they break down when the battle ends unless repaired properly.
2. Proper repair removes underlying damage. The detailed vehicle-repair rules
   distinguish jury-rigging a damaged system from workshop repair of a
   destroyed system; the space-combat chapter does not provide a complete
   permanent starship work-order and facility model.
3. Routine ship maintenance is continuing onboard upkeep. CE accounts for it
   monthly at 1/12 of 0.1% of ship value per month; missed CE maintenance can
   cause system degradation.
4. A **refit** is an overhaul: extensive maintenance of all ship systems that
   restores operational efficiency. It is not the monthly maintenance charge.
5. A **refurbishment** replaces or upgrades major and minor systems and is the
   Clement mechanism explicitly associated with extending a ship's useful
   life.

Cepheus Trader must not represent these as one counter, one “repaired” flag,
or one service date. Paying routine maintenance forever does not make a hull
or installation ageless.

## Clement Sector Source Audit

The Milestone 3 audit checked the current Clement core material and the
rules-bearing supplements rather than inferring procedures from ship prose.
The useful results are:

- *The Anderson & Felix Guide to Naval Architecture*, p.109, treats the
  monthly maintenance figure as an operating cost. *Port of Entry*, p.24,
  clarifies that this figure represents parts and repair items when the
  onboard crew performs the work. Port maintenance adds labor, testing, and
  possibly special-berth charges; technician labor is Cr75–150 per hour.
  This expands the terse core CE statement that maintenance “requires a
  shipyard.” Cepheus Trader adopts the more detailed Clement treatment:
  routine onboard upkeep is normal, while yard work is purchased when needed.
- *Anderson & Felix*, p.113, defines a refit as extensive maintenance of all
  ship systems, normally performed when a warship returns from deployment.
  It takes four to six standard weeks and costs four monthly maintenance
  payments.
- *Anderson & Felix*, pp.113–115, defines refurbishment as life extension by
  replacement, upgrade, or reconfiguration. It supplies component-specific
  cost and time rules, a secondhand/reconditioned component market, and a
  monthly failure check for suspiciously cheap components.
- *Port of Entry*, pp.24–25, makes ordinary damage a minor repair, one
  destroyed system a refit, and multiple destroyed systems or a critical hit
  a refurbishment. It guarantees minor repair at class C or better and gives
  refit/refurbishment size limits of 800 tonnes at C, 2,000 tonnes at B, and
  any size at A. A particular non-port yard may exceed its port's guarantee.
- *Bounded Fortune*, pp.22–26 and 39–42, confirms that previous use creates
  wear, defects, and loss of value. New-ship warranties are commonly limited
  by both five years and 200 interstellar transits; used-ship warranties use
  two years or 50 transits. Its used-ship quirks are an initial-condition
  generator, not a general wear-over-time procedure.
- The Clement core vehicle-repair rules provide useful workshop, spare-part,
  time, and destroyed-system precedents, but they are vehicle rules. They do
  not by themselves establish the material quantity or price of permanent
  starship repairs.

These sources give us a strong service taxonomy, facility limits, and much of
the economy. They do **not** give a complete dynamic starship wear algorithm,
a scheduled overhaul interval, or a general repair-material quantity per
starship hit. Those remaining rules must be identified as Cepheus Trader
adaptations when designed.

There is also a repair-price seam in the Clement material. *Port of Entry*
calls one destroyed ship system a refit, while *Anderson & Felix* prices a
refit at only four monthly maintenance payments and separately prices actual
component replacement under refurbishment. Applying the inexpensive refit
price as though it bought a destroyed Jump drive would be implausible. The
implementation must distinguish restoring a repairable installation from
buying and installing a replacement rather than silently choosing the cheaper
reading.

## Persistent Subsystem Record

Every materialized ship has stable subsystem records for its hull, structure,
armor, bridge, computer, sensors, drives, power plant, fuel system, life
support, cargo hold, installed equipment groups, weapon mounts, screens, and
hangars. Refits will add or retire records rather than renumbering surviving
subsystems.

The implemented record contains:

```text
subsystem_id
kind
label
maximum_hits
sustained_hits
battlefield_repair_hits
last_proper_repair_second
installed_second
last_refit_second
calendar_age_months
operating_seconds
duty_cycles
skimming_cycles
neglect_damage_hits
component_kind
component_id
displacement_millitons
replacement_price_credits
installation_generation
reconditioned
component_warranty_expires_second
```

`sustained_hits` is the authoritative physical damage. A completed proper
repair reduces it. `battlefield_repair_hits` is temporary operational coverage
and may never exceed sustained damage. Effective encounter damage is:

```text
sustained_hits - battlefield_repair_hits
```

The underlying sustained value is retained and displayed even while a patch
is working. Ending the encounter clears battlefield coverage; it does not add
new damage because the covered damage was never removed.

## Condition and Service Ledgers

Each subsystem has independent state for:

- combat or accident damage;
- temporary battlefield repair coverage;
- routine-maintenance performance and arrears;
- accumulated calendar and usage wear;
- installation age and significant duty cycles; and
- repair, refit, refurbishment, and replacement history.

Routine maintenance is normally performed continuously by the engineering
and maintenance crew while the ship remains in service. The month is its
accounting and degradation-check period, not a claim that the ship enters a
yard every month. Adequate routine maintenance prevents neglect-related
degradation; it does not remove combat damage, erase accumulated wear, reset
component age, or substitute for overhaul.

Calendar age, operating minutes, Jump and maneuver duty cycles, and stressful
fuel-skimming cycles are recorded on the installations they actually stress.
The Clement warranty limits support tracking both years and transit count,
but warranty expiration is not itself a failure probability.

A refit is the closest Clement rule to a conventional overhaul. It restores
operational efficiency through extensive service, but it does not make the
ship newly constructed. A refurbishment may replace selected installations;
only a replaced installation receives a new installation age. Hull age and
the age of untouched systems survive both operations.

The version-1 reliability policy uses a bathtub curve. Shakedown occupies the
first fifth of the standard five-year/200-transit warranty reference,
followed by a broad flat useful-life interval. Wear-out begins at 180% of that
reference, well after the warranty. Calendar age and transit use take the
more severe normalized value. This curve is a versioned Cepheus Trader rule,
not a number supplied by CE or Clement.
For the use axis, each gas-skimming cycle counts as one additional transit
cycle; this is also a versioned Cepheus Trader policy. It gives the source's
warranty-voiding operation real wear exposure without inventing a separate
immediate skimming-failure table.

A condition event may attach a hidden quirk to an eligible installation.
Ordinary diagnostics and `GetShipStatus` never enumerate latent quirks.
Qualified routine service at a class C or better port can discover a quirk
while the warranty is active; a successful warranty claim removes it without
charge. Gas-giant skimming voids the ordinary new-ship warranty, following
the warranty treatment in *Bounded Fortune*. Warranty expiration itself does
not cause a failure.

The three state transitions remain independent:

- a battlefield repair changes only temporary coverage;
- a proper repair changes sustained damage but does not automatically claim
  that routine maintenance, a refit, or refurbishment was performed;
- routine maintenance pays for and performs continuing upkeep without
  resetting wear or age;
- a refit performs a weeks-long overhaul but does not replace every system;
  and
- refurbishment or explicit replacement resets only the installations that
  were actually replaced.

Proper repair is a per-subsystem scheduled work order at a class C or better
port. It removes physical and neglect damage and expires any temporary patch,
but does not change age, routine-upkeep history, or refit history. The monthly
upkeep allowance already represents ordinary parts and repair items, so the
first implementation accounts for elapsed shop access without inventing an
unsupported per-hit material-price curve. A destroyed installation is
explicitly rejected: it requires component refurbishment or replacement with
a catalog-derived component price.

Refits enforce the *Port of Entry* hull-size limits, take a deterministic four
to six weeks, and cost four monthly maintenance payments. A refit clears
repairable physical damage, temporary patches, neglect damage, and minor
latent quirks while retaining installation age, use counters, destroyed
systems, and quirks that require deeper component work. Its scheduled event
and ship ledger are consumed atomically, so a restart or repeated simulation
drain cannot complete it twice.

## Offline Post-Combat Recovery

When an offline-controlled encounter ends and the player's surviving crew
retains control of the ship, the server automatically schedules as much
feasible onboard recovery as possible. Encounter-only
`battlefield_repair_hits` expire before the planner assesses the ship; a
temporary patch never silently becomes a permanent repair.

The capability priority is:

1. Life support
2. Maneuver drive
3. Jump drive
4. Weapons

This order is dependency-aware. Work on hull containment, power, controls,
fuel systems, or the bridge that is necessary to restore a higher-priority
capability belongs to that capability's repair goal. The planner uses actual
qualified crew, watches, tools, spares, repair supplies, access, and game time,
and stops when no further safe supported work is possible. Damage requiring a
facility remains an outstanding work order. Surrendered, abandoned, captured,
or crewless ships do not run the player's recovery plan.

The controller, scheduling, audit, and reconnect behavior are specified in
[`combat-control-and-automation.md`](combat-control-and-automation.md).

## Current Implementation

`GetShipStatus` returns an ordered, phase-tagged observation with the ship
revision, catalog performance, refined/unrefined fuel, physical provisions,
catalog-capacity ammunition, every subsystem's physical and temporary damage,
component identity and replacement basis, age/use counters, routine-upkeep
ledger, warranty, refit history, and typed active activity. Latent quirks are
not exposed. A quirk becomes a reported symptom only after the installation is
used in a relevant voyage or encounter; the status response still does not
reveal the hidden cause or component directly. The OpenDoors Ship manager
provides summaries, subsystem detail, symptoms, proper repair, refit, and
new/reconditioned replacement paths at 40×24.

The server schedules one routine-upkeep transaction every 30 game days. It
charges 1/12 of 0.1% of construction price, ages installations, performs the
CE neglect check when maintenance cannot be paid, evaluates the versioned
hidden reliability curve, and schedules the next accounting boundary. Payment
prevents neglect damage; it does not erase combat damage or accumulated age.
There is no virtual or remote life-support invoice.

Installed provisions are a physical ship store. The initial load is 30
person-days per physical awake-accommodation place, and the installed limit is
180 person-days per place. Physical capacity counts both occupants of an
ordinary or compact stateroom, all fitted crew berths, and every fitted
high-class or steerage place; low berths use their own life support instead.
The queued daily system transaction consumes one person-day for each
represented living crewmember and each awake passenger aboard. Departure
preview includes both crew and passengers in the required person-days, and
commitment rejects a shortage. Dockside monthly packages use the accommodation
costs from *Anderson & Felix*, p.109:
Cr2,000 per ordinary/compact stateroom, Cr3,000 per crew-berthing or barracks
allocation, Cr5,000 per high-class stateroom, Cr100 per low berth, and Cr100
per emergency-berth capacity. Commercial passage remains single occupancy: an
available ordinary, compact, or high-class room supplies one saleable passenger
berth, while steerage supplies its explicit fitted places. Passenger
double-occupancy and luxury dining supplements wait for actual passenger
loading rather than being charged to an empty room.

Ammunition is likewise part of the persistent ship record. Initial magazine
capacity, pack size, and pack price come from normalized catalog components.
Combat materialization reads the remaining quantity and writes consumption
back to the same record; dockside loading accepts only fitted ammunition and
cannot exceed catalogued capacity. Neither reconnect nor a new encounter
regenerates a magazine.

`GetDockedServices` quotes every facility-legal service against one ship
revision. `CommitDockedService` must present that revision and orders exactly
one refined/unrefined fuel load, named frontier fuel expedition, ammunition
load, provision package, proper repair, refit, or component replacement. A
stale quotation is rejected before mutation. Frontier orders must name the
quoted celestial body. Proper repair and refit preserve the distinctions
above; a destroyed installation can only enter the new or reconditioned
component market. Replacement price, displacement, installation generation,
component warranty, and completion ledger are all persisted, and completion
is one scheduled queue transaction.

Automatic post-combat repair planning is implemented. It clears temporary
battlefield coverage and persists the survival/dependency/M-drive/J-drive/
weapon priority. Away from a yard it selects the best conscious named
crewmember, resolves an EDU/Mechanic task with injury, fatigue, and field-tool
modifiers, and spends one game day on each safe attempt against underlying
damage. At a capable arrival facility it instead chains ordinary proper-repair
work. A failed field attempt remains recorded and waits for a new order or a
yard; destroyed installations remain blocked until a capable yard and a
catalog-priced new or reconditioned component are selected.
