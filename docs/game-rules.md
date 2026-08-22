# Cepheus Trader game rules

This rulebook describes how a captain's position, obligations, resources,
crew, ship, travel, encounters, and legal standing are resolved in Cepheus
Trader. It explains the game rules rather than the menus; the Player Reference
explains how to issue commands through the door.

Rules from the Cepheus Engine System Reference Document are included when they
apply to play. Adopted third-party Open Game Content is restated in
setting-neutral terms and folded into the applicable chapters. Cepheus Trader
adds specific rules where its persistent shared universe requires them.
Character careers, psionics, referee procedures, personal equipment lists,
planetary adventures, and other tabletop systems that are not used in play are
not part of this rulebook.

All original text and game mechanics on this page are Open Game Content under
the Open Game License version 1.0a. Upstream Product Identity remains excluded.
The complete license, designation, and Section 15 notices are available on the
[Open Game License](open-game-license.html) page.

## Reading the rules

The words **captain** and **player** are not interchangeable. The player is the
person at the BBS; the captain is a person in the game. A captain can be hurt,
captured, relieved, or succeeded without replacing the player's persistent
identity.

The rules use these common terms:

- **2D6** means roll two ordinary six-sided dice and add them, producing a
  total from 2 through 12.
- **D66** means roll two distinguishable six-sided dice and read the first as
  the tens digit and the second as the units digit, producing one of the 36
  table codes from 11 through 66. It is a lookup code, not a sum: a first-die
  2 and second-die 5 is result 25, not 7.
- A **DM** is a dice modifier. Add all applicable DMs before comparing a
  result with its target.
- **Effect** is the final task total minus the target number. Zero or greater
  succeeds; a larger positive or negative Effect measures the result.
- **Cr** means credits. **MCr** means one million credits.
- A **game second, day, week, or month** is simulation time, not real time.
- A **revision** identifies the exact state used for a quote, plan, order, or
  other proposal. If that state changes first, the proposal must be refreshed.

Each chapter names its rules lineage. A lineage identifies mechanical
ancestry, not affiliation and not permission to use an upstream trademark or
Product Identity.

## Core resolution

**Rules lineage:** Cepheus Engine SRD, with Cepheus Trader's persistent and
simultaneous-action procedures.

### Characteristics

People use Strength (STR), Dexterity (DEX), Endurance (END), Intelligence
(INT), Education (EDU), and Charisma (CHA). Cepheus Trader uses CHA in place of
Social Standing. Rank, title, reputation,
citizenship, authority, legal status, and relationships are separate records;
CHA never grants them.

| Characteristic | DM |
| --- | ---: |
| 0 | -3 |
| 1-2 | -2 |
| 3-5 | -1 |
| 6-8 | +0 |
| 9-11 | +1 |
| 12-14 | +2 |
| 15 or more | +3 |

### Tasks

When an uncertain action calls for a task, use:

**2D6 + characteristic DM + skill level + equipment DM + condition DM +
assistance DM**

Compare the total with the task's target number. Most ordinary professional
tasks used by the game are Average, target 8. A rule can set another target,
choose a different characteristic and skill, or supply modifiers for time,
tools, injury, fatigue, morale, environment, damage, opposition, or assistance.
The action screen or result identifies the relevant assignment and important
modifiers.

A trained skill contributes its level, including level 0. An untrained skill
contributes -3. Jack of All Trades reduces only that untrained penalty, by its
level to a maximum relief of 2; it does not add to a trained skill.

Once an action is committed, its roll is made once. Refreshing a screen,
disconnecting, reconnecting, or retrying the same command cannot reroll it.
Commands that spend money, consume supplies, move property, accept obligations,
or change standing commit atomically: either the complete result occurs once,
or none of it does.

### Time and persistence

The universe advances at 28 game seconds per real second while the server is
running: four game weeks pass per real day. Server downtime is frozen game
time. Logging off does not pause travel, deadlines, upkeep, payroll, training,
traffic, or another captain's actions.

Events resolve in the order they are accepted. No player action can overtake
an earlier accepted action. A command prepared from an older revision is
rejected if the relevant state has changed.

## Captains, crew, and training

**Rules lineage:** Cepheus Engine character and training rules; admitted
third-party Open Game Content for CHA and crew operations; Cepheus Trader
creation, service, and recovery rules.

### Creating a command

A new player distributes the displayed characteristic budget, selects skills
from the offered slots, chooses one training target, selects a career offer,
names and fits the ship, and reviews its named crew. The complete displayed
budget must be spent, and every default shown during creation is legal. Captain,
ship, crew, title, finance, and starting stores are created together only
after final confirmation.

Named officers, leaders, and senior specialists have individual records.
Large supporting establishments can be represented as position teams. A team
has a current strength and an established complement; casualties reduce its
real strength rather than creating or deleting decorative names.

### Watches, condition, and morale

A person can be assigned to one or more shipboard duties or placed off watch.
Assignments must cover the positions an operation requires. Injury, fatigue,
absence, shore location, treatment, and inadequate team strength can reduce or
remove a person's contribution.

Off-watch personnel receive full rest. On-watch personnel heal at the active
lifestyle rate; serious wounds improve naturally only during full rest.
Fatigue applies a condition penalty and can make a person unconscious. First
aid, surgery, inpatient care, and ordinary natural recovery are distinct and
require their stated time, skill, and facility.

Crew service is physical and contractual. Wages fall due monthly. If available
cash cannot cover the whole payroll, payment is divided proportionally rather
than favoring the first roster entry. Individual arrears persist and reduce
morale. Shore leave, recall, treatment, reassignment, and discharge occur at a
real facility; a ship that departs does not carry someone left ashore.

### Skill training

A person's **Skill Total** is the sum of all positive skill levels. Level-0
skills add zero. Training to a desired next level requires:

**training weeks = current Skill Total + desired level**

Levels are gained in order. A person trains only one skill during a game week,
and Jack of All Trades cannot be trained. Each completed seven-day boundary
adds at most one week to a valid course. Normal watches, Jump, routine work,
port calls, and brief encounters do not subtract study hours. Changing the
target discards current progress and starts the new course from zero.

At creation, a target must be an existing skill other than Jack of All Trades.
When the required weeks are complete, the skill increases by one level and the
course stops until another target is selected. Point-award advancement systems
from other rulesets are not used.

## Worlds, ports, and facilities

**Rules lineage:** Cepheus Engine world, starport, technology, law, and trade
codes; admitted third-party Open Game Content for starport services; Cepheus
Trader's 3D universe and facility records.

### Systems and charts

The map is three-dimensional and distances are measured in parsecs. There are
no navigation hexes or subsectors. Each stellar component is a separate
system and can contain stars, planets, moons, belts, gas giants, ports, and
other operational loci. A primary world retains a Universal World Profile,
trade codes, starport class, population, law, and technology data.

Charts are observations. A captain knows only systems and details carried by
the ship's records or received through survey and mail. Empty surveyed space
is also knowledge: it prevents a resolved volume from being rerolled merely
because another captain arrives later.

### Technology, law, and services

Technology Level measures local scientific and manufacturing capability. It
does not guarantee a facility, stock, trained labor, authority, or parts.
Imported equipment can exceed local TL, while a damaged or absent yard can
make locally understood work unavailable.

Law Level governs prohibited and restricted goods and contributes to customs
and enforcement. During a compliant customs inspection, prohibited ordinary
cargo is confiscated and a fine of 10% of its base value is collected, limited
by the credits currently available. Restricted cargo remains aboard after a
compliant inspection unless another displayed rule or order says otherwise.

Every port service comes from a persistent facility. A menu, map, or high TL
does not create a bank, yard, hospital, authority office, fuel source, berth,
or market that is not present and capable. Capacity can be occupied by real
work, and quoted work reserves the named facility until completion or valid
cancellation.

## Markets, cargo, and contracts

**Rules lineage:** Cepheus Engine trade and carriage rules; admitted
third-party Open Game Content for merchant finance, passengers, cargo, crew,
ports, and customs; Cepheus Trader persistent-market rules.

### Markets and negotiation

The commodity catalogue contains the six Common Goods and 35 generic D66
results from the adopted merchant trade table. Result 66 is reserved for
unusual, individually recorded objects rather than generic stock. Each day's
market selects uniformly among the 35 generic results, equivalent to rolling
D66 and rerolling 66. Stock is finite, shared, and persistent for a system and
day. Looking at the exchange does not reroll or reserve stock; buying consumes
it.

Quotes depend on the commodity base value, local trade codes, market events,
tariffs, and the captain's Broker skill and CHA. Purchase negotiations use
80%, 90%, 100%, or 120% of base price before local adjustments. Sale outcomes
use base price plus 30%, 15%, 2%, or 0% before local adjustments. When the
exchange both stocks and buys an item, its ordinary bid is capped below its
ask, so immediate unloading cannot manufacture profit. A separately reserved
private buyer can cross that public spread.

Price landmarks show the absolute universe-wide span for that commodity, not
the odds of receiving a particular quote. Low purchase prices and high sale
prices are favorable. Market reports record their place, date, source, and
confidence; they do not update themselves while travelling.

Prohibited goods cannot be bought through the open exchange. Every cargo lot
has an owner, commodity, quantity, physical ship or facility, and provenance.
Speculative cargo belongs to the captain's estate; freight belongs to its
principal. Splitting a transaction cannot duplicate credits or cargo.

### Research and reservations

Supplier or buyer research takes one to six game hours and assigns a named
person and method. A hired local Broker costs Cr500, supplies Broker-2 for the
search, and normally reports in one to three hours; the named crewmember
remains liaison.

A completed lead records a finite quantity, price range, source, observation
date, confidence, expiry, and revision. Reserving it places 10% of estimated
value in escrow. Release or expiry does not refund that opportunity payment.
The reservation can therefore be lost, and old intelligence cannot be reused
as infinite stock.

### Carriage and tasks

Ordinary freight, passengers, and electronic mail use a standing declaration
for one destination. The captain states maximum freight, eligible passenger
capacity, and whether to accept mail. Departure previews a concrete manifest
and brokerage, then loads those exact offers if they remain valid. Freight is
a titled physical lot; passengers occupy actual eligible accommodation; mail
uses the declared route but never causes a voyage.

Other offers become durable Tasks. Current types include freight, passage,
purchase orders, forward sales, supply commitments, charters, couriers, and
bounties. Each offer states closing time, origin, destination, performing
capacity, collateral, payment, deadline, failure penalty, and whether partial
performance is allowed.

A local claim at its issuing office can be awarded immediately. A remote claim
is a signed message that must physically reach that office. Capacity and
collateral remain reserved while it travels. Competing claims are decided by
arrival order. The private award or decline must then reach the captain before
custody transfers. Closure notices also travel, so a stale remote offer can
still accept a claim that will later be declined.

Custody transfers only at the required origin and stays with the performing
ship. A route change never rewrites an obligation. Delivery removes custody
once and sends a settlement filing to the issuing office. Payment and
collateral return only when the remittance reaches the captain. Late,
incomplete, cancelled, returned, defaulted, and disputed work follow the terms
shown in the Task ledger; restart cannot duplicate payment or release.

### Finance and title

An independent trader purchase begins with 20% equity and 80% secured debt.
Monthly principal is purchase price divided by 240, paid over a 480-month
schedule; mandatory insurance is separate. A missed principal or insurance
installment has one accounting month of grace before default action.

Privateer vessels are sponsor-owned and naval vessels are institution-owned.
Restricted operating credit pays authorized vessel expenses before liquid
cash and can pay required insurance, but it does not pay secured principal,
private trade, collateral, private messages, fines, or other personal costs.
Command, possession, title, and debt are separate facts.

## Ship operation and condition

**Rules lineage:** Cepheus Engine ship operation, fuel, maintenance, and
damage; admitted third-party Open Game Content for construction, refits,
services, and warranties; Cepheus Trader condition ledgers.

### Capacity and stores

Cargo, passenger places, crew accommodation, low berths, hangars, fuel tanks,
magazines, and carried craft are physical installed capacity. The Ship Catalog
publishes each vessel's complete fitted record. An action cannot use spare
capacity that another lot, passenger, prisoner, store, or craft already
occupies.

Fuel is refined or unrefined and remains in the ship's tanks. Provisions are a
physical store consumed by crew and awake passengers; people in low berths use
their berth support instead. Ammunition belongs to installed compatible
weapons and is removed when fired. Departure rejects a known shortage rather
than creating supplies through a background charge.

### Routine upkeep, wear, and warranty

Routine maintenance is continuing onboard work charged every 30 game days at
one twelfth of 0.1% of ship value. Paying it prevents neglect-related
degradation; it does not heal combat damage, reset age, conceal a known defect,
or replace a destroyed component. If it cannot be paid, the normal neglect
check applies and arrears remain visible.

Installations separately record calendar age, operating time, Jump and
maneuver cycles, and stressful skimming cycles. Hidden construction quirks can
manifest only through relevant use. Once manifested, their symptoms are
reported; routine upkeep does not erase them.

A component can carry a five-year or 200-transit warranty reference. Gas-giant
skimming voids the ordinary new-ship warranty. A qualifying manifested defect
can be removed without charge by a capable facility while coverage remains;
warranty expiry does not itself cause a failure.

### Damage, repair, refit, and replacement

Battlefield repair, proper repair, refit, refurbishment, and replacement are
different operations.

- A battlefield repair temporarily covers a sustained combat result and
  expires when the encounter ends.
- A proper repair removes eligible underlying damage with required skill,
  facility, time, and supplies; it does not change age or service history.
- A refit overhauls the ship, takes four to six game weeks, and costs four
  monthly maintenance payments. It clears eligible wear penalties but does
  not replace destroyed installations.
- Refurbishment or replacement buys the actual catalog component work needed
  for a destroyed or exhausted installation.

Away from a yard, post-combat recovery chooses the best conscious qualified
person and attempts an EDU/Mechanic task with injury, fatigue, and field-tool
modifiers. Each attempt takes one full game day and removes one underlying
sustained hit only on success. Destroyed installations require refurbishment
or replacement.

Automatic recovery works in dependency-aware order: life support, maneuver
drive, Jump drive, then weapons. Hull containment, power, bridge, fuel, or
control work needed by a higher priority belongs to that priority.

## Flight, fuel, and Jump

**Rules lineage:** Cepheus Engine maneuver, Jump, course plotting, fuel, and
misjump rules; compatible admitted third-party operational procedures; Cepheus
Trader flight plans and 3D geometry. Alternate interstellar-drive systems are
not used.

### Flight plans

A Flight Plan is the ship's executable route, separate from Tasks and Known
Universe. Each step names a destination and whether the captain has authorized
a terminal stop, a hold for attention, or unattended continuation. Filing a
plan commits only the reviewed revision. Completed steps and accepted custody
cannot be undone by editing later steps.

A plan can include port purchases, wilderness water or ice collection,
gas-giant skimming, Jump loci, known systems, surveyed coordinates, and
imported plotted courses. Preview calculates known time, fuel, purchases, and
obligation warnings. If a required service, source, payment, or course is no
longer valid when reached, the plan holds instead of silently substituting
another one.

Ordinary interplanetary transfer uses continuous acceleration to the midpoint
and continuous deceleration afterward. For a stationary endpoint estimate:

**travel time in seconds = 2 x square root of (distance in meters / acceleration
in meters per second squared)**

Flight time uses the orbital positions at departure and the vessel's actual
acceleration, so a moving destination or later departure can change the
answer.

### Jump range and fuel

A Jump drive rating is the maximum parsecs in one Jump. Any positive
sub-parsec Jump counts as Jump-1. Required fuel is:

**Jump fuel tons = 0.1 x hull displacement tons x leg distance in parsecs**

Jump can begin or end only outside the union of the 100-diameter exclusion
zones of relevant massive bodies. Maneuver flight between a port and the
legal Jump locus uses the ship's real thrust, position, time, and fuel rules.

An onboard plot is an Easy EDU/Astrogation task modified by Jump number and
plot age. A fresh commercial course tape costs Cr1,000 per Jump number and is
sold only at Class-D-or-better ports for populated destinations. Initiation is
an Average EDU/Engineer (Jump) task taking 10-60 seconds. Its Effect contributes
to the Jump success result. Drive damage, known bad plots, unrefined fuel, and
other displayed conditions apply normally.

The Jump success total is:

**2D6 + initiation Effect - plot age in months - 2 per Jump-drive hit - 2 if
any Jump fuel burned is unrefined**

| Success total | Result |
| --- | --- |
| 8 or more | Accurate Jump |
| 1-7 | Inaccurate arrival; add 1D6 game days of normal-space travel |
| 0 or less | Misjump 1D6 x 1D6 parsecs in a random 3D direction |

A known bad plot automatically misjumps. The 100-diameter rule is enforced
before initiation, so an ordinary filed plan cannot exchange an unsafe origin
for a dice penalty.

A normal Jump lasts **148 + 6D6 hours**. Inaccurate emergence, misjump, and
transition critical damage persist as physical results; disconnecting cannot
avoid them.

### Staged Jump and frontier fuel

A destination can be empty space. A double-tanked Jump-1 ship can cross two
parsecs with two separately resolved Jump-1 legs and a mandatory one-game-day
midpoint turnaround. The midpoint supplies no port, fuel, market, mail,
traffic, or rescue. It consumes ordinary endurance, provisions, crew time,
and contract time. Two fresh Jump-1 tapes have a baseline price of Cr2,000.

Refined port fuel is safest. Unrefined port fuel, lawful wilderness water or
ice, and gas-giant skimming can reduce purchase cost but require their actual
source, equipment, time, and authority. Collection rights do not follow from
hydrographics alone. Skimming adds wear and voids the ordinary new-ship
warranty. Each collection operation is a named Flight Plan step; failure or a
changed legal condition holds the plan.

## Information, mail, and discovery

**Rules lineage:** Cepheus Engine mail and communication assumptions;
Cepheus Trader causal communication, encryption, mapping, and discovery
procedures.

### Light and carriage

There is no faster-than-light radio. Within normal space, transmissions and
sensor evidence travel no faster than light. Across a Jump boundary,
electronic information must be physically carried by a ship. A report is not
known at its destination until its signal or carrier arrives.

Maintained Jump-locus beacons exchange sealed route-eligible mailbags with
departing and arriving vessels and can pay a local carriage stipend. Accepting
a mailbag never chooses or changes the vessel's route. News, offers, warrants,
market reports, and institutional replies keep their event, observation,
dispatch, and arrival times; later local facts do not rewrite an old message.

Private correspondence costs Cr1 per started KiB per charged hop per started
TTL week. TTL is selected from one through 52 game weeks. Fixed-system mail
uses its known route; mobile addressee mail propagates within its reachable TTL
sphere. A destination-assistance policy costs Cr350,000 per year and can be
bound or cancelled only while docked; cancellation has no refund.

### Survey and mapping

First arrival records the ship's real observations and surveyed empty volume.
The captain can keep a system secret, share it directly, make it public, or
file a private discovery claim. A disclosure becomes known only through its
ordinary physical messages.

The first valid private claim for a newly discovered settled system is judged
at Earth. Its award is Cr218,000. Competing filings are decided when they
physically reach the adjudicating office, and the public notice begins its own
outward carriage only after judgment.

## Traffic and encounters

**Rules lineage:** Cepheus Engine encounter checks and opposed crew tasks;
admitted third-party Open Game Content for traffic, ports, naval operations,
and piracy; Cepheus Trader contact geography and causal response.

### Where traffic exists

Traffic concentrates at ports, inhabited worlds, gas giants, Jump arrival
loci, and Jump departure loci. Ordinary interplanetary space is sparse unless
a ship's actual trajectory makes contact or interception possible. Jump space
contains no ordinary encounters.

At an arrival checkpoint with **N** eligible nearby traffic vessels, the
standard one-in-six candidate check is combined as:

**contact chance = 1 - (5/6)^N**

An encounter still requires sensor contact and a feasible relative-position
solution. Refreshing the arrival screen does not make another independent
traffic roll. Sparse traffic follows persistent route schedules rather than
appearing once per visitor.

Strongly ordered systems more often challenge arrivals with customs or naval
pickets. Pirate pickets favor uncontrolled arrival and frontier-fuel loci and
retreat as traffic and enforcement make them untenable. A pirate compares the
observed target with its own capability and can abandon a plainly disastrous
intercept.

### Interception and response

Contact does not teleport ships together. Interception uses actual position,
velocity, thrust, fuel, and projected motion. A distress call or weapons
emission reaches an observer only after separation divided by the speed of
light, and identifies no more than the observer's sensors and provenance
support.

A responder must then fly a feasible intercept. If it arrives before combat
ends, it joins at that time; otherwise it can reach only pursuit, rescue,
arrest, salvage, or aftermath. Computer and absent-player ships evaluate the
same delayed, uncertain evidence available to a captain, not hidden truth.

## Vessel combat

**Rules lineage:** Cepheus Engine space-combat turns, actions, reactions,
range, attacks, missiles, damage, and boarding; admitted third-party Open Game
Content for weapons, large crews, naval operations, privateering, and piracy;
Cepheus Trader simultaneous joint orders and offline control. The complete
weapon table follows this chapter.

### Turns, initiative, and orders

A space-combat turn is 1,000 game seconds. All active vessels submit one joint
crew order against the same combat revision, then resolve in initiative order.
At the fixed clock rate the shared real-time order window is about 35.7
seconds. Each crew member or station team receives its legal minor and
significant actions. Dodge, Fire Sand, Point Defense, and Trigger Screens use
the vessel's limited prioritized reactions when their triggers occur.

The joint order commits atomically. It can change range, attack, defend,
perform electronic warfare or damage control, prepare withdrawal, board,
surrender, or use another action legal to the assigned station and current
situation. Missiles have delayed impact and can be opposed by point defense.
Weapon ammunition, damage, casualties, emissions, and legal evidence commit
with the turn.

Starting initiative is 2D6. Higher initiative acts first; ties go to higher
Thrust and otherwise resolve in the same initiative position. Initiative is
not rerolled each turn. Increase Initiative adds the positive Effect of an
Average Leadership task.

| Initiative | Reactions per turn |
| --- | ---: |
| 4 or less | 1 |
| 5-8 | 2 |
| 9-12 | 3 |
| 13 or more | 4 |

The available joint-order actions are Hold, Coordinate, Increase Initiative,
Evasive Maneuvers, Line Up Shot, close or open range, Break Pursuit, Sensor
Targeting, Electronic Warfare, Damage Control, Attack, Board, Prepare Jump,
Launch Escape Craft, Offer or Accept Surrender, and Inspect Contact. The
available reaction priorities are Dodge, Point Defense, Fire Sand, Trigger
Nuclear Damper, and Trigger Meson Screen.

Successful Evasive Maneuvers applies -1 to incoming attacks, or -2 with Effect
6 or more. Successful Line Up Shot and Sensor Targeting each applies +1 to
attacks, or +2 with Effect 6 or more. Coordinate applies +1 to targeting on
success. A successful open-range action at Distant range completes withdrawal.

An Average Damage Control task creates one battlefield repair on success, two
at Effect 3, or three at Effect 6 or more. Coverage goes first to power,
maneuver, Jump, bridge, sensors, fuel, hold, and then a weapon mount. This is
temporary coverage only and expires at combat end.

### Range and missiles

| Range | Separation |
| --- | ---: |
| Adjacent | Less than 1 km |
| Close | 1-10 km |
| Short | 10-1,250 km |
| Medium | 1,250-10,000 km |
| Long | 10,000-25,000 km |
| Very Long | 25,000-50,000 km |
| Distant | More than 50,000 km |

Each usable weapon in a mount makes a 2D6 attack against 8, adding the assigned
gunner's task DM, the range DM in the weapon appendix, targeting and line-up
bonuses, target evasion, and -2 if the mount has one uncovered hit. A mount
fires no more than once per turn. A mount with two uncovered hits is disabled.
Every ammunition weapon consumes one compatible round when it fires.

Missile launch Effect sets the later to-hit target. Effect -6 or worse needs
11+; Effect -5 through -1 needs 10+; Effect 0 needs 8+; Effect 1-5 needs 7+;
and Effect 6 or more needs 6+. Missiles launched from Short through Long range
arrive one turn later. Missiles from Very Long or Distant range arrive two
turns later. Point defense is resolved on the impact turn and spends one
reaction; a successful target-8 point-defense task destroys that missile.

### Default and automated control

An online activation opens with a complete conservative legal order. It
protects life support and escape capability, evades or withdraws when useful,
uses defensive electronic warfare and reactions, and holds unnecessary
offensive fire. It does not ordinarily choose pursuit, ramming, hostile
docking, offensive boarding, surrender, or abandonment.

Each commanded ship also has a persistent risk-directed controller. Its
default minimum estimated objective-success threshold is 70%. If the estimated
chance meets the threshold, it pursues the objective; otherwise it attempts a
real withdrawal. If escape becomes infeasible, it can choose surrender or
abandonment only when that is the best modeled survival result.

The controller knows only what the captain could know. It considers up to 64
legal joint orders, then tests the strongest eight against 256 three-round
projections apiece before choosing. If no order is submitted, the controller
acts for the ship; disconnecting does not freeze combat or reveal hidden enemy
state.

### Damage, capture, and loss

Combat damage applies to the real vessel, installations, ammunition, crew,
cargo, and carried craft. Radiation weapons add the applicable crew hazard;
meson weapons ignore armor and begin on the internal damage table. Surrender
or successful boarding transfers the existing vessel, cargo, unique objects,
prisoners, and prize title; it never creates a duplicate.

If the crew retains control, automatic recovery begins under the ship-condition
rules. A surviving captain returns after seven game days following
abandonment, 14 days after other destruction or loss, or 30 days after capture
or surrender. A dead captain requires a named successor. Career, warrant, and
estate history persist through succession.

Roll weapon damage, subtract current armor unless the weapon has the meson
trait, and convert the penetrating total into grouped system hits. Every hit in
one Double or Triple group uses the same location.

| Penetrating damage | Hit groups |
| --- | --- |
| 0 or less | None |
| 1-4 | One Single |
| 5-8 | Two Singles |
| 9-12 | One Double |
| 13-16 | Three Singles |
| 17-20 | Two Singles and one Double |
| 21-24 | Two Doubles |
| 25-28 | One Triple |
| 29-32 | One Triple and one Single |
| 33-36 | One Triple and one Double |
| 37-40 | One Triple, one Double, and one Single |
| 41-44 | Two Triples |
| Each complete 6 above 44 | One additional Double |
| A remaining complete 3 above 44 | One additional Single |

Hull and Structure each begin at one point per 50 displacement tons, with a
minimum of one. Vessels of 100 tons or more use the external column while Hull
remains, then the internal column. Craft below 100 tons use the small-craft
column while Hull remains, then resolve internal hits. Meson hits always use
internal resolution.

| 2D6 total | External vessel | Internal vessel | Small craft |
| ---: | --- | --- | --- |
| 2 | Hull | Structure | Hull |
| 3 | Sensors | Power plant | Power plant |
| 4 | Maneuver drive | Jump drive | Maneuver drive |
| 5 | Weapon mount | Weapon mount | Fuel |
| 6 | Hull | Structure | Hull |
| 7 | Armor | Crew | Armor |
| 8 | Hull | Structure | Hull |
| 9 | Fuel | Hold | Weapon mount |
| 10 | Maneuver drive | Jump drive | Maneuver drive |
| 11 | Sensors | Power plant | Structure |
| 12 | Hull | Crew | Hull |

Structure at zero destroys the vessel. Armor hits remove one current armor
point and become Hull hits when no armor remains. Weapon mounts take -2 to
attacks after one hit, are disabled after two, and retain further damage.
Maneuver hits reduce real thrust, then disable and destroy the drive according
to its condition record. Jump, power, sensors, bridge, fuel, and hold hits
apply the staged impairment reported in Ship Management and remain after the
encounter unless properly repaired.

Boarding can begin only at Adjacent range. Each boarding round resolves opposed
2D6 totals with the current side bonus. Effect +6 captures the defender;
Effect -6 drives off the attackers. Any other result removes one Structure,
continues the boarding, and gives +2 on the next round to the side that won
that round. Destruction at zero Structure still applies during boarding.

## Authority, careers, and force

**Rules lineage:** Admitted third-party Open Game Content for naval
organization, captain operations, piracy, privateering, capture, fencing, and
forced docking; Cepheus Engine tasks and combat; Cepheus Trader jurisdiction
and message propagation.

Weapons provide capability, not authority. A lawful use of force depends on
the actor, jurisdiction, order, commission, warrant, target, evidence, and
time. Authorities know only instruments and reports that have reached them.
A warrant can therefore exist at one office while a distant captain or port
has not yet received it.

Naval captains command public property under orders. Service funds are
restricted, and rank or authority can be changed through service procedure.
Privateers act under a sponsor's commission and prize terms. The commission's
scope, expiry, targets, reporting duties, and exit terms bound its authority;
force outside them is not legalized by the ship's armament.

Piracy can arise through free predation, accepted leads, or a pirate
commission. It uses the same physical interception, combat, boarding, cargo,
damage, prisoner, and evidence rules as every other career. A pirate result is
not made safe or profitable by selecting a career label.

A captured ship or cargo becomes a prize claim only when real custody and
required evidence exist. Adjudication, bounty settlement, impound, assessment,
appeal, and notice happen at their named offices and propagate through the
mail system. Paying an assessment or delivering a prisoner cannot instantly
erase enforcement at an office that has not received the signed instrument.

## Ship and weapon reference

The [Ship Catalog](ships.html) gives each available vessel's hull, drives,
fuel, accommodation, equipment, weapons, ammunition, software, performance,
and price. Those listed statistics and fittings govern the vessel in play;
they are not sample configurations.

The weapon appendix below is the reference used in vessel combat. `--` means
that a weapon cannot attack at that range. Damage is expressed as dice plus a
fixed modifier. Traits identify rules such as beam defense, delayed missiles,
radiation, meson penetration, bay mounts, and physical ammunition.

### Hidden information

Unrevealed contacts, hidden defects, undiscovered systems, unarrived messages,
and other hidden facts are not player knowledge. They become known only
through the observations, discoveries, messages, and other means described in
these rules.
