# Universal Player Managers

## Status

The OpenDoors client has a common command-console shell with seven managers.
Crew Management has an authoritative snapshot plus training, watch, service,
location, treatment, pay, and morale mutations. Ship Management has an
authoritative condition and dock-service surface. Message Management reads the
captain's delivered archive, persists classifications, and carries typed links
to the authoritative Task, Finance, Mapping, or Operations record. Known
Universe has its carried system set, mail-driven provenance, course plotting,
and mapping-disclosure state. Task Management has an authoritative ledger,
offer acceptance, physical custody, settlement, default, and dispute actions.
Operations Ledger exposes the implemented naval, privateer, pirate, prize,
warrant, and traffic-interception state. System Common Radio carries transient
in-system broadcasts and structured encounter hails at light speed.

## Availability and Authority

After player creation, the following managers are reachable from every
operational phase:

1. Crew Management
2. Ship Management
3. Task Management
4. Message Management
5. Known Universe
6. Operations Ledger
7. System Common Radio

“Available everywhere” means that the player may always inspect the manager
using the knowledge and state available to the current frame of reference. It
does not mean that every mutation is legal everywhere.

The server remains authoritative. Each manager query returns a committed
snapshot tagged with its revision and current phase. Each mutation is an
ordinary command entering the ordered engine queue and is accepted or rejected
against the phase in which it is processed. The same standard error payload
reports the resulting current phase.

Opening a manager does not pause simulation, advance time, or create a separate
client-side authority. When encounter semantics are eventually selected,
manager use must obey the same turn or live-time rules as other interface
activity.

## Crew Management

Crew Management owns the player-visible view of people aboard and their
current service relationships:

- roster, departments, watches, duty stations, vacancies, and role coverage;
- captain and crew training targets, required weeks, and current weeks;
- injuries, fatigue, availability, morale, and emergency substitutions; and
- hiring, dismissal, transfer, shore leave and recall, first aid, surgery,
  inpatient care, wage arrears, and replacement.

Roster and training status are inspectable everywhere. Reassignments may be
restricted during an encounter, while personnel transactions generally require
an appropriate port or facility. A planned change may be retained without
pretending it has already taken effect.

Aggregate supporting positions remain distinct from the named officer, leader,
or senior specialist representing their role. Assignment and casualty rules
must not silently give every supporting person the representative’s identity or
statistics.

The persistent service/home appointment is distinct from current watch duty.
A named person has a list of zero or more active duty roles:

- an empty list means **off watch** and makes the person eligible for full rest;
- several roles mean the person is doubling duties, subject to the ordinary CE
  simultaneous-action penalties when both demand attention;
- several people may cover the same role; and
- Pilot is the CE exception and may have only one assigned person.

The captain's institutional command authority is not a watch assignment.
Taking the captain off watch does not transfer or abandon command. Starter
ships whose captain also supplies the required pilot begin with both Captain
and Pilot active. The role catalog and assignments are authoritative server
state; the door only submits selected role IDs.

The queued daily system job now applies natural healing to the actual people
aboard ships in that system. An empty assignment list receives the CE full-rest
rate and clears the coarse fatigue state; an on-watch person uses the CE active
lifestyle rate. Serious wounds only improve naturally during full rest and may
deteriorate when current Endurance produces a negative DM. The server persists
the resulting injury/death state in the same transaction; merely opening the
UI never awards recovery. First aid resolves once within its CE time window.
Surgery records its result but applies the physical change only when its queued
hours finish. Inpatient care charges for the recovery actually supplied. Leave
and treatment retain the exact shore facility: a ship that departs leaves that
person awaiting recall at the real berth. Monthly payroll is proportional when
funds are short, preserving personal arrears and reducing morale without
database-order preference. Condition and morale DMs feed the shared CE task
resolver.

## Ship Management

Ship Management owns the player-visible technical and logistical state of the
currently commanded ship:

- hull, drives, power, life support, weapons, defenses, and small craft;
- active damage, failures, isolation state, casualties, and repair priorities;
- fuel, cargo, passengers, mail, ammunition, spares, and other consumables;
- maintenance due, endurance, and operational limitations; and
- proposed repair, configuration, and refit work.

Status remains inspectable everywhere. Damage-control commands may be
encounter-time actions; routine maintenance requires time and personnel; major
repair or refit may require a suitable facility.

Ship condition uses separate authoritative fields for sustained physical
damage, temporary battlefield-repair coverage, and per-subsystem scheduled
maintenance. A combat patch changes effective encounter damage but never
removes the underlying hits, advances a service date, or survives the end of
the encounter. Proper repair and scheduled service are also separate work
results. The record and transition rules are specified in
[`ship-condition-and-maintenance.md`](ship-condition-and-maintenance.md).

After an offline-controlled battle, a crew that retains the ship automatically
attempts feasible recovery in the order Life Support, Maneuver drive, Jump
drive, then weapons. Prerequisite work and real crew, time, tools, and supplies
still apply. Combat orders and recovery automation are specified in
[`combat-control-and-automation.md`](combat-control-and-automation.md).

## Task Management

Task Management owns obligations accepted from other parties and the
captain's standing operational policies. It does not own the executable ship
route:

- active contracts, naval or polity orders, missions, and bounties;
- deadlines, penalties, committed cargo or passengers, and completion state;
- route, reserve, risk, fuel, maintenance, and encounter policies, including
  the minimum estimated objective-success probability for automatic combat;
  offline rules for observing, reporting, investigating, rescuing, or joining
  nearby combat; evidence thresholds; and diversion/resource limits;
  and
- constrained automatic choices such as accepting ordinary mail or passengers
  already travelling along the declared route.

Standing policies are defaults evaluated by the server when a relevant choice
occurs. They are not scripts that bypass phase checks, capacity, law, money,
information delay, or ordered transaction processing. “Prefer frontier fuel”
is a preference among legal alternatives; “always accept mail” means accept
eligible ordinary mail when capacity and route permit it.

Tasks derived from a message retain a reference to that message, but task state
is not merely a message classification. Accepting a contract creates an
obligation even if its original offer is later archived.

Every accepted obligation names its performing ship. Entrusted freight is a
titled cargo lot and passenger, charter, and courier parties are physical
manifests. Loading, transit, partial delivery, recurring performance, default,
custody return, and settlement mutate those objects and the Task atomically.
Changing active command or owning another vessel cannot teleport custody, and
a ship carrying an active obligation cannot be traded in.

The phase-level **Flight Plan** interface owns the active destination,
waypoints, hold points, and bounded through-point authorizations. Task
Management may offer **Plot course for this task** and similar shortcuts, but
they only pass requirements into Flight Plan. Replanning never rewrites an
accepted obligation: its cargo, passengers, mail, destination, deadline, and
consequences remain authoritative even when the captain diverts.

## Message Management

Message Management owns the complete collection of material already delivered
to the player:

- news and headlines;
- market reports and observations;
- contract offers, orders, missions, and bounties;
- danger, traffic, customs, and navigation notices;
- personal and institutional mail; and
- expired or historically archived material.

The player can browse untagged, ignored, marked-for-later, accepted-task, and
archived classifications; search old material; and filter by origin, age,
distance, topic, authority, urgency, significance, expiry, or authenticity.
Classification and filters do not delete the underlying retained message.
Routine system-discovery notices are hidden by the default filters, but their
delivered structured observations still update the recipient's Known Universe
repository.

The implemented first filter surface stores one minimum importance band for
each service class. The four bands are Routine, Notable, Important, and
Headline. The door edits them from Message Management, and the server applies
them only when assembling arrival review. Origin, age, distance, topic,
authority, expiry, and authenticity filters remain later refinements; the
archive browser deliberately continues to show copy suppressed at arrival.

The manager exposes provenance including origin system, origin date, delivery
age or path where known, expiry, and authentication. During an isolated
encounter it shows only messages received before entering that frame. Opening
it cannot observe mail or news that arrived elsewhere while the encounter was
being resolved.

Typed message action references contain only the authoritative record kind and
identifier. The door may open Claim Offer, Task, Finance, Mapping, or
Operations from the article, but it never reconstructs or mutates rule state
from article prose.

## System Common Radio

System Common is the public, shared-medium radio channel within the current
star system. It is separate from interstellar mail. A transmission expands
from its emitter at light speed, so ships at different local landmarks receive
the same broadcast at different game times. It is unavailable in Jump space,
and a ship with no functioning communications installation cannot receive or
transmit.

Player broadcasts are limited to 500 printable ASCII characters and one
transmission every 15 seconds of real time (420 game seconds). Captains may
persistently mute another captain's ordinary broadcasts. Inspection orders,
boarding instructions, and surrender demands are structured safety-critical
hails and cannot be muted. They use the same observable medium: another ship
within the wave may overhear a hail, but only its intended ship receives the
associated encounter action.

Radio is deliberately transient. The manager lists unread receptions without
injecting their bodies into unrelated screens. Opening one displays it once
and consumes that ship's copy; retaining it outside the game is the captain's
responsibility. Unread receptions expire after 196 game days. They belong to
the receiving ship rather than the login, and therefore remain with that ship
if command or ownership changes.

## Known Universe

Known Universe is the current ship's sourced, potentially stale operational
model of systems, routes, worlds, polities, facilities, markets, traffic, and
hazards. It is organized around subjects, observations, comparisons, and
route-planning questions rather than chronological correspondence.

Messages and knowledge observations share delayed-delivery and provenance
problems, and one delivered data packet may produce both records. They remain
separate stores and interfaces: archiving a source message does not delete an
observation, and direct sensor observations need not create artificial
messages.

Knowledge is repository-scoped. It does not automatically synchronize among
distant ships owned by one player and never exposes current authoritative
server state merely because the server has materialized it. Full storage,
physical synchronization, projection, and interface rules are in
[`known-universe.md`](known-universe.md).

Known Universe also owns mapping disclosure. Arrival in a system not known by
the current repository to be publicly mapped prompts the captain to send a
free public notification, send a paid encrypted direct filing to Earth,
withhold, or withhold and mark the system secret. The captain-private Secret
Systems list is editable from this universal manager at any time and does not
synchronize to distant player-owned ships.

## Operations Ledger

Operations Ledger owns the combat-career view: naval grade, service points,
pay and available orders; privateer commissions and prize-court claims; pirate
leads, cruises, underworld standing and crew pressure; system traffic-control
reports; local real-contact interception opportunities; and physically
delivered warrants and local settlement choices. It does not create targets or
instant legal knowledge.
Local contacts are restricted to the ship's current port, Jump locus, or body;
ordinary interplanetary transit and Jump space return no local target list.
Civilian registry information comes from transponders, while hull class and
tonnage remain sensor observations with equipment- and damage-dependent
confidence. The system-wide traffic-control feed is a separate transponder
picture and does not make a remote vessel interceptable. Prize claims move by
ordinary mail, and a warrant has no local effect before its message arrives.

## Door Navigation

The common command console uses:

- `C` — Crew Management
- `S` — Ship Management
- `T` — Task Management
- `M` — Message Management
- `R` — System Common Radio
- `K` — Known Universe
- `O` — Operations Ledger
- `L` — license and copyright notices
- `Q` — return to the BBS

The shell is shared by the ISO 646, ISO 646 plus ECMA-48 colour, and CP437 plus
ECMA-48 profiles and remains usable at 40×24. Location- and phase-specific
menus will add their own commands without replacing these seven managers.
