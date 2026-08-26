# Docked Operations

## Purpose

Every new player is materialized aboard the selected ready-to-depart ship at
the BBS polity's capital-world starport. Docked operations are therefore the
first phase-specific gameplay surface after creation and the natural next
vertical slice after the universal Crew and Ship managers.

`Docked` must be derived from concrete ship location, not stored as a
free-standing player flag. The location record needs:

- system and celestial-body IDs;
- starport or other facility ID;
- berth ID where individual berth tracking matters;
- landed, orbital-dock, or other attachment state;
- docking or arrival game time and accrued port fees; and
- the local services currently available to this ship.

The starting ship is docked at a valid facility on the BBS prime world. Later
ships may be docked at orbital stations, naval bases, repair yards, tenders,
or other installations with different services.

## Docked Menu

The OpenDoors client now presents the phase-specific docked menu whenever the
authoritative `ServerHello` phase is `Docked`:

- `C` — **Cargo Exchange:** reconcile automatic delivery, sell speculative
  cargo, and buy available trade goods.
- `J` — **Jobs and Passage:** ordinary freight, passengers, and mail plus
  contract, charter, courier, order, and bounty offers available locally.
- `F` — **Fuel and Supplies:** refined or unrefined fuel, life support,
  ammunition, repair supplies, and other consumables.
- `Y` — **Shipyard:** proper system repairs, scheduled per-subsystem
  maintenance, and facility-permitted refits.
- `P` — **Personnel:** hiring, dismissal, shore leave, medical treatment, and
  other local crew services.
- `B` — **Banking and Accounts:** cash, mortgage, local or mail-mediated
  transfers, escrow, insurance where available, and prize settlements.
- `A` — **Authorities:** customs, warrants, licenses, naval reporting, port
  control, taxes, and locally required clearances.
- `D` — **Depart:** settle immediate berth obligations and submit a complete
  continuation plan beginning with release from the facility.

Crew, Ship, Task, Message, and Known Universe remain universal managers and
are not duplicated as docked submenus. A docked action may link to a filtered
view of one of those managers, but it remains a server-authoritative
phase-specific transaction.

`U` opens those universal managers from the docked menu. Cargo Exchange,
Depart, and Fuel and Supplies are authoritative. Fuel and Supplies loads
physical provisions and ammunition as well as fuel. The Ship manager commits
proper repair, refit, and new or reconditioned component replacement. These
services share one revision-checked quotation/commit RPC rather than exposing
separate mutation calls for each menu item. Personnel commits hiring,
discharge, transfer, leave, recall, first aid, surgery, and inpatient care.
Banking commits insurance and the implemented finance actions. Authorities
opens the local career, warrant, prize, and traffic office. Services absent
from the facility are omitted from the menu and remain server-rejected if a
stale or forged command requests them.

Docked is an attachment state inside the port traffic locus, not permission to
fight from a berth. An immediate intercept order against spaceborne local
traffic first settles the accrued berth charge and clears the ship into the
port locus. Selecting another berthed vessel instead establishes a departure
watch: the interceptor waits outside and combat can begin only when the target
clears its attachment. Cancelling a port watch returns the interceptor to a
berth and starts a new berth-fee interval.

The docked header is an authoritative snapshot of the commanded ship, system,
primary world, persistent primary-world facility, UWP-derived
starport/TL/population/law data, account balance and provisional debt,
refined/unrefined fuel, accrued departure-settled berth fee, and cargo
utilization. Facility revision, operational state, fuel depots, chandlery,
licensed ordnance dealer, repair shop, maximum supported hull and component
TL, medical level, personnel exchange, bank, authority office, and controlled
traffic status are persistent state. Individual berth assignment and service
queues remain later depth.

Cargo Exchange implements the six CE Common Goods and 35 generic revised
*Bounded Fortune* trade goods. System/day stock is seed-derived and finite;
consumption and market revision are persistent and shared. Captain-specific
quotes apply Broker, Charisma, world trade codes, legality, and the displayed
local tariff. A read never rerolls or consumes stock. Purchases create titled,
identifiable cargo lots, spend account credits, consume hold capacity, and are
exactly-once under command replay. Sales in a system other than a speculative
lot's origin remove a selected quantity and credit the locally negotiated
price. Quantities retain milliton (0.001-tonne) resolution. Purchase totals
round upward and sale proceeds downward at the final credit, so splitting a
transaction cannot create money. Timed supplier/buyer research and its dated
Known Universe observations are described in
[`merchant-economy-and-tasks.md`](merchant-economy-and-tasks.md).

Refined fuel costs the CE Cr500 per ton and is sold only by class A or B
starports. Unrefined port fuel costs Cr100 per ton at class A through C ports.
The server accepts vendor quantities at 0.001-ton resolution, rounds the final
proportional charge up to a whole credit, and validates tank capacity, source,
and available credits. Frontier collection and onboard processing remain
whole-ton work orders. The server tracks the unrefined fraction in the tanks
rather than treating all fuel as interchangeable bookkeeping.

The Fuel and Supplies screen can instead schedule a bounded gas-giant or
wilderness water/ice expedition. The server lists every qualifying named body
with its type (gas giant, planet, moon, or icy belt) and marks routine sources
as unoccupied wilderness access. Unavailable sources are explanatory only and
are not numbered. The displayed maximum is the remaining tank room, not a
promise that every body can supply an arbitrary amount.

The server calculates the exact selected-quantity round trip at catalogued
thrust and separately previews travel, collection, processing, normal total,
and failed-processing total. A captain with an installed processor explicitly
chooses whether to refine during collection; a ship without one can still
collect with scoops, but receives unrefined fuel. Refining is an Average (8+)
Engineer (Power)/EDU task. Failure doubles processing time, and Effect -6 or
worse also damages the Jump drive (falling back to the maneuver drive, then
fuel system, if no operational Jump drive exists). Completion occurs exactly
once. Gas skimming adds drive/fuel-system duty and a skimming cycle and voids
the standard warranty.

Purchased or collected unrefined fuel may also be refined later as a Flight
Plan action while docked or safely holding. It cannot overlap active travel,
Jump, an encounter, or another ship activity. Away from a berth, the server
protects the selected batch and requires enough other fuel for the worst-case
processing-time power-plant burn. An unrefined purchase embedded in a Flight
Plan defaults to processing that purchased batch before continuing; a plan can
explicitly retain it as unrefined instead.

Installed life-support provisions and catalogued ammunition are physical
quantities in the ship record. Provisions are bought in physical
awake-accommodation-based monthly packages. Away from a berth they are
consumed by living crew and awake passengers through queued daily work.
Docked ordinary crew arrange their own meals; the captain uses one ship
person-day or automatically pays liquid credits for an ashore meal at twice
the package-average daily rate. An unfed character receives the CE starvation
grace period, checks, and damage. Ammunition is bought only
in the fitted component's pack size, cannot exceed magazine capacity, is
consumed by combat, and is never regenerated merely by materializing another
encounter.

A chandlery or licensed ordnance dealer must exist both when the server
quotes the service and when the order commits.

Wilderness-water planning never infers access from the primary world's
hydrographic code alone. The implemented lawful service selects only a known
unoccupied rocky body with water or an icy belt. Populated worlds normally
control their water. Permission, ownership claims, naval or government
authority, and hostile extraction remain explicit encounter/legal operations,
not a shortcut through the fuel-service command.

## Departure Transaction

The complete target design is a multi-page client wizard that collapses into
one atomic command.
It finalizes loading, immediate fees, clearance and readiness; selects one or
more in-system destinations; and optionally preauthorizes bounded actions such
as docking, skimming, or initiating a specified Jump.

Accepting the command releases the berth and records the continuation plan. It
does not resolve the whole trip in the same transaction. Port traffic, the
jump-departure locus, destination approaches, and other convergence areas are
scheduled checkpoints. They always perform authoritative encounter and
readiness processing, but they interrupt the player only when an encounter or
validation problem occurs, the plan ends there, or the player changes it.

Consequently, a through plan that names a Jump destination may initiate that
Jump automatically if every checkpoint is uneventful. A plan that only names
the jump locus holds there and does not need an onward system. The same rule
allows an uneventful plan to dock, land, skim a declared amount of fuel, or
perform another explicitly authorized terminal action without an artificial
extra confirmation. Full semantics are in
[`interplanetary-operations.md`](interplanetary-operations.md).

The implemented first continuation is deliberately narrower: the captain
selects one known system within the ship's jump rating and confirms one
through voyage. The server atomically validates knowledge, range, current
location, and one-jump fuel, then schedules departure to the seed-derived safe
jump locus. Separate durable transactions enter Jump, deduct the catalogued
jump-fuel quantity, complete the standard one-week Jump, schedule the
destination's seed-derived safe-locus approach, and dock at its primary-world
starport. Cargo survives every transition. Encounter, clearance, astrogation,
jump-tape, failure, and replanning checkpoints have not yet been inserted.

## Facility-Driven Availability

Docking does not imply a shipyard, refined fuel, a bank, customs, or a liquid
market. Menu entries disclose the local service state and may be unavailable
when the facility cannot provide them.

CE starport class, bases, local TL, population, law, damage, ownership, and
current statistical activity establish the baseline. Mutable facility state,
inventories significant at player scale, queues, closures, corruption, and
political restrictions modify it. Proper repair must honor the distinction
between battlefield patches and real work described in
[`ship-condition-and-maintenance.md`](ship-condition-and-maintenance.md).

## First Implementation Order

1. Implement the docked landing page across all four terminal profiles.
   **Done.**
2. Persist ship location and derive phase from it. **Done for docked,
   departure, Jump, arrival approach, and destination docking.**
3. Return an authoritative local port/facility snapshot. **Done, including
   persistent mutable capability overlays and revisioned availability.**
4. Implement berth fees, fuel, routine supplies, and departure. **Done for
   accrued berth fees, refined and unrefined port fuel, life-support supplies,
   wilderness collection, gas-giant skimming, and the first through-Jump
   continuation.**
5. Implement proper repair and scheduled-maintenance work orders. **Done for
   repairable subsystem damage, monthly upkeep, four-to-six-week refits, and
   catalog-priced new/reconditioned replacement of destroyed components.**
6. Add cargo, ordinary carriage, contracts, personnel, banking, and authority
   transactions as their underlying simulations become available. **Done for
   the current mechanically playable boundary.**

This order produces a complete docked-to-interplanetary transition before the
larger market and contract systems are required.
