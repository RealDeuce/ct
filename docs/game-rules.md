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
Social Standing.

| Characteristic | What it measures | Typical use in play |
| --- | --- | --- |
| STR | Physical force | Personal injury capacity, boarding, and physical work |
| DEX | Coordination and reaction | Piloting, gunnery, and precise physical action |
| END | Health and stamina | First damage track, healing, fatigue, and sustained work |
| INT | Reasoning and adaptability | Electronic warfare, investigation, and judgment |
| EDU | Learned knowledge | Astrogation, engineering, medicine, and formal procedure |
| CHA | Personal presence and influence | Leadership, negotiation, hiring, and persuasion |

Rank, title, reputation, citizenship, authority, legal status, and
relationships are separate facts; CHA never grants them.

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

Tasks named **Routine**, **Average**, **Difficult**, **Very Difficult**, and
**Formidable** use effective targets 6, 8, 10, 12, and 14 respectively. A task
can instead state its target directly. When a rule gives a time as 1D6 seconds,
minutes, kiloseconds, hours, days, or weeks, roll once in that unit. A
kilosecond is 1,000 seconds, or one vessel-combat turn.

The following skills exist on people in Cepheus Trader. A specialization is a
separate skill: Pilot (Spacecraft) does not grant Pilot (Small Craft), and one
Engineer or Gunner specialty does not grant the others.

| Skill | Current rule use |
| --- | --- |
| Admin | Task-dispute filings; command-role eligibility |
| Advocate | Competes with Admin for the better Task-dispute filing Effect |
| Astrogation | Jump plotting, Flight Plan watch coverage, and combat range actions |
| Broker | Commodity quotes and non-online supplier or buyer research |
| Carouse | Recorded and trainable; no independent task currently uses it |
| Communications | Combat targeting and inspection; electronic-warfare assignment currently has no resolved effect |
| Computer | Online supplier or buyer research |
| Electronics | Recorded and trainable; traffic contact quality currently uses the ship's fitted Electronics DM instead |
| Engineer (Jump Drive) | Jump initiation, operation, and drive work |
| Engineer (Maneuver Drive) | Required Flight Plan watch coverage |
| Engineer (Power) | Required Flight Plan watch coverage |
| Engineer (Life Support) | Recorded and trainable; no independent task currently uses it |
| Etiquette | Qualifies high- and middle-passage staffing; level does not change the check |
| Gun Combat | Recorded and trainable; no personal-combat task is currently resolved |
| Gunner (Turrets) | Turrets, barbettes, point defense, and sand |
| Gunner (Capital Weapons) | Bay and other capital-scale weapons |
| Gunner (Screens) | Screen-reaction assignment; those reactions currently have no resolved effect |
| Investigate | Can supply the best skill level for an arrest search party |
| Jack of All Trades | Relief of the untrained penalty only |
| Leadership | Combat coordination and initiative actions |
| Mechanic | Field recovery and combat damage control |
| Medicine | First aid, surgery, recovery, and low-passage staffing |
| Melee | Recorded and trainable; no personal-combat task is currently resolved |
| Persuade | Recorded and trainable; no independent task currently uses it |
| Pilot (Spacecraft) | Flight Plan watch coverage, hostile encounters, and combat piloting |
| Pilot (Small Craft) | Recorded and trainable; no separate small-craft task currently uses it |
| Recon | Can supply the best skill level for an arrest search party |
| Stealth | Captain's concealment level against arriving warrant enforcement |
| Streetwise | Can supply the best skill level for an arrest search party |
| Tactics (Military) | Boarding actor assignment; its DM is not yet applied to boarding rounds |
| Tactics (Naval) | Can be the captain's highest skill in a hostile arrival encounter |
| Trade (Cargomaster) | Recorded and trainable; loading and custody currently use capacity and ledger checks |
| Trade (Prospector) | Belt prospecting, survey, and mining-drone watches |
| Vacc Suit | Recorded and trainable; no EVA task is currently resolved |

One person assigned to concurrent actions takes -2 on every task for each
additional action when the combination is physically possible. Actions that
must occur at the same instant from separate stations require separate people,
a station team, or applicable automation.

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

A new captain has six characteristic scores from 2 through 12. Begin each at
7 and distribute exactly 12 points among them; equivalently, the six final
scores must total 54. A captain then selects three different skills at level
2, six at level 1, and three at level 0. Jack of All Trades can be selected
only at level 1 or 2. The captain also chooses one existing non-Jack skill as
the initial training course.

Every home polity offers one independent-trader command, one privateer
command, and one naval command. The actual hull, fit, title, reserves,
obligations, authority, and exit terms shown with an offer are part of that
offer. A displayed endurance refit converts ten tons of cargo capacity into
ten tons of permanent fuel capacity; it is offered only when the hull has ten
tons available to convert.

Initial crew roles are fixed by the chosen fitted ship. Each initial crew
package assigns the characteristic array 10, 9, 8, 8, 7, and 6 to suit its
role and contains two skills at level 2, four at level 1, and three at level
0. The player names each person and selects one of that person's existing
non-Jack skills for training; creation does not rewrite the role package.
Captain, ship, crew, title, finance, and starting stores are created together
only after final confirmation.

Initial trader crew are salaried; initial privateer crew have the same monthly
salary plus a 5% prize-share term; naval crew are institutionally supplied.
The salary basis is Cr1,000 per represented position each accounting month.
The recorded individual privateer share is not currently deducted or
distributed by prize settlement; only the aggregate crew share of a Pirate
cruise has a settlement effect.

Named officers, leaders, and senior specialists have individual records.
Large supporting establishments can be represented as position teams. A team
has a current strength and an established complement; casualties reduce its
real strength rather than creating or deleting decorative names.

### Watches, condition, and morale

A person can be assigned to one or more shipboard duties or placed off watch.
Assignments must cover the positions an operation requires. Injury, fatigue,
absence, shore location, treatment, and inadequate team strength can reduce or
remove a person's contribution.

Physical damage reduces current END first. Further damage is spread among the
remaining nonzero STR, DEX, and END values. A person with any two of those
three values at zero is unconscious; a person with all three at zero is dead.
Someone below the normal value of all three physical characteristics is
seriously wounded. Injury changes the characteristic used by a physical task,
and any fatigue applies a further -2 condition DM. Fatigue 2 incapacitates the
person.

Recovery is checked once per game day. Full rest clears the coarse fatigue
counter. A wounded person at full rest naturally restores **1D6 + current END
DM** characteristic points; an active person restores **1 + current END DM**.
A serious wound receives no natural recovery while active and, at full rest,
restores only **current END DM**; a negative result can worsen the wound.
Facility medical care restores **2 + current END DM + (facility Medical rating
- 1)** points per day and costs Cr5,000 for each point actually restored.

First aid is an Average EDU/Medicine task that must be attempted within one
game hour of injury and only once for those wounds. Positive Effect restores
that many characteristic points, doubled when aid begins within five minutes.
Surgery is available only to a seriously wounded living patient at a Medical-3
facility. It is an Average EDU/Medicine task with +1 equipment DM, takes 1D6
hours, restores twice its positive Effect with a minimum of two points on
success, and inflicts the negative Effect as damage on failure. It costs
Cr5,000 per point restored. Inpatient care and shore leave can each be booked
for one through 30 days.

Crew service is physical and contractual. Wages fall due monthly. If available
cash cannot cover the whole payroll, payment is divided proportionally rather
than favoring the first roster entry. Individual arrears persist and reduce
morale by 10 each unpaid month; clearing prior arrears restores 5 morale.

Morale 60-100 is Steady, 40-59 Uneasy, 20-39 Disaffected, 1-19 Defiant, and 0
Broken. Disaffected service applies -1 to discretionary crew tasks; Defiant or
Broken service applies -2. A salaried or prize-share crewmember at a port has
a daily **20 - morale** percent chance to quit when morale is below 20. Recorded
wage arrears remain a service claim after departure.

Loyalty and risk tolerance are separate 0-100 service records. They describe
the person's relationship to the command and appetite for danger, but the
current rules attach no independent task DM, desertion roll, or automation
override to either value; morale and arrears supply the active modifiers above.

Completed shore leave restores 5 morale; early recall costs 5 morale. Recall,
treatment, reassignment, transfer, and discharge occur at a real shared
facility; a ship that departs does not carry someone left ashore. A newly
hired specialist's monthly salary is **Cr1,000 x (1 + skill level)^2**.

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
other operational loci. One planet, moon, habitat, or belt is the system's
**primary world** for interstellar records and commerce; it can be
uninhabited. Other bodies can have their own settlements and facilities, but
they do not replace the primary world's profile.

A **polity** is the named interstellar authority associated with a system and
its service institutions. It is separate from the primary world's Government
code, which describes that world's broad internal form, and from the
Chaos-Order and Trade-Combat orientations defined under traffic.

Every newly founded BBS home polity is internally connected by Jump-2 legs
and joins the existing system-to-system Jump-2 network that reaches Sol. Its
capital therefore has a Jump-2 route to Sol through its own cluster and an
already plotted contact route. The founding ship's stops on that route become
visited together with every member of the new polity, creating a narrow
traffic-active bridge through the former frontier without moving or
regenerating older systems. The polity's protected three-parsec core is
resolved before its remaining six-parsec frontier is materialized.

Charts are observations. A captain knows only systems and details carried by
the ship's records or received through survey and mail. Empty surveyed space
is also knowledge: it prevents a resolved volume from being rerolled merely
because another captain arrives later.

At the founding epoch, the complete volume inside the fixed 43-system
catalogue's convex boundary is settled first. Survey stations then plot the
remaining space out to six parsecs from every catalogue system. Contacts found
in that outer shell enter the public charts but remain frontier systems until a
player ship physically visits them. The shell survey cannot insert another
system inside the already settled catalogue volume.

### Universal World Profile

A **Universal World Profile**, or **UWP**, is the primary world's compact set
of social and physical characteristics. Traditional compact notation puts the
starport first, then six world codes, a hyphen, and Technology Level:

**Starport-Size-Atmosphere-Hydrographics-Population-Government-Law-TL**
**A867945-C**, for example, means a Class A port, Size 8, Atmosphere 6,
Hydrographics 7, Population 9, Government 4, Law 5, and TL12. The separators
between the first seven fields are omitted in an actual compact profile. Codes
10 through 15 can be written A through F, but Cepheus Trader's screens
ordinarily show labeled decimal values instead of requiring the captain to
decode a single line.

| Field | What the code establishes | Direct game relevance |
| --- | --- | --- |
| Starport | Broad capability of the primary port, from A to E or X | Baseline fuel, repair, medical, yard, market, and office availability |
| Size | World diameter and approximate surface gravity, 0-10 | Helps determine atmosphere, water, population, TL, and trade codes |
| Atmosphere | Pressure, composition, and habitability, 0-15 | Helps determine water, population, minimum TL, and trade codes |
| Hydrographics | Surface water coverage, 0-10 | Helps determine population, minimum TL, and trade codes |
| Population | Order of magnitude of inhabitants, 0-10 | Affects port class, government, traffic, markets, and trade codes |
| Government | Broad form of local government, 0-15 | Affects Law Level and TL; authority itself is recorded separately |
| Law | Restrictiveness and enforcement environment, 0-15 | Affects cargo legality, customs, tariffs, clearance, and enforcement |
| TL | Local scientific and manufacturing capability | Limits locally manufactured ships, components, repairs, and other work |

The profile is generated in dependency order. Size is **2D6-2**. Atmosphere is
**2D6-7 + Size**; Hydrographics is **2D6-7 + Size** with penalties for hostile
atmospheres; and Population is **2D6-2** with modifiers for Size, Atmosphere,
and Hydrographics. Population then modifies Starport and Government,
Government modifies Law, and the entire profile modifies TL. Values are
limited to their stated code ranges. A Population-0 world also has Government
0, Law 0, and TL0. Resolving or revisiting a system does not reroll its accepted
profile.

### Physical world codes

Size is a diameter code, not a mass or a count of worlds. The gravity values
are broad surface comparisons used to give the code physical meaning.

| Size | Approximate diameter | Approximate gravity |
| ---: | ---: | ---: |
| 0 | 800 km or an asteroid body | Negligible |
| 1 | 1,600 km | 0.05g |
| 2 | 3,200 km | 0.15g |
| 3 | 4,800 km | 0.25g |
| 4 | 6,400 km | 0.35g |
| 5 | 8,000 km | 0.45g |
| 6 | 9,600 km | 0.70g |
| 7 | 11,200 km | 0.90g |
| 8 | 12,800 km | 1.00g |
| 9 | 14,400 km | 1.25g |
| 10 | 16,000 km | 1.40g |

Atmosphere describes the conditions at the inhabited world's reference
surface. Tainted air requires a filter; very thin air requires a respirator;
exotic air requires an independent air supply; and vacuum, corrosive, or
insidious conditions require a sealed suit. Planetary excursion and personal
survival procedures are outside the current game, but these codes still
determine population, technology minimums, and trade classifications.

| Atmosphere | Meaning | Ordinary protection indicated |
| ---: | --- | --- |
| 0 | None | Sealed suit |
| 1 | Trace | Sealed suit |
| 2 | Very thin, tainted | Respirator and filter |
| 3 | Very thin | Respirator |
| 4 | Thin, tainted | Filter |
| 5 | Thin | None |
| 6 | Standard | None |
| 7 | Standard, tainted | Filter |
| 8 | Dense | None |
| 9 | Dense, tainted | Filter |
| 10 | Exotic | Independent air supply |
| 11 | Corrosive | Sealed suit |
| 12 | Insidious | Sealed suit; the environment also attacks equipment |
| 13 | Dense at low altitude, breathable only in suitable highlands | Habitat-dependent |
| 14 | Thin at altitude, breathable only in suitable lowlands | Habitat-dependent |
| 15 | Unusual | Varies |

Hydrographics measures the percentage of the surface covered by liquid. Code
0 means 0%-5%; codes 1 through 9 mean 6%-15%, 16%-25%, and so on in ten-point
bands; code 10 means 96%-100%. Size 0 and Size 1 worlds have Hydrographics 0.
Atmospheres 0, 1, and 10-12 apply -4 to generation, while Atmosphere 14 applies
-2.

### Population and government

Population is an order-of-magnitude code. It is not the number of people and
does not describe how evenly they are distributed.

| Population code | Broad population | Base order of magnitude |
| ---: | --- | ---: |
| 0 | Uninhabited | 0 |
| 1 | A few people | 10 |
| 2 | Hundreds | 100 |
| 3 | Thousands | 1,000 |
| 4 | Tens of thousands | 10,000 |
| 5 | Hundreds of thousands | 100,000 |
| 6 | Millions | 1,000,000 |
| 7 | Tens of millions | 10,000,000 |
| 8 | Hundreds of millions | 100,000,000 |
| 9 | Billions | 1,000,000,000 |
| 10 | Tens of billions | 10,000,000,000 |

An inhabited primary world also has a Population Multiplier from 1 through
10. Its estimated inhabitants are **Population Multiplier x 10 raised to the
Population code**. Thus multiplier 4 and Population 8 mean about 400,000,000
people. The multiplier is zero on an uninhabited world. The code, rather than
that estimate, is used for traffic, trade-code, and profile calculations.

Population generation receives -1 for Size 0-2, -2 for Atmosphere 10 or more,
+3 for Atmosphere 6, +1 for Atmosphere 5 or 8, and another -2 when
Hydrographics is 0 and Atmosphere is below 3. The final code is limited to
0-10.

Government is a broad description of the primary world's governing system. It
does not by itself grant a captain authority, allegiance, citizenship, rank,
or immunity.

| Government | Broad form |
| ---: | --- |
| 0 | None |
| 1 | Company or corporation |
| 2 | Participating democracy |
| 3 | Self-perpetuating oligarchy |
| 4 | Representative democracy |
| 5 | Feudal technocracy |
| 6 | Captive government |
| 7 | Balkanized governments |
| 8 | Civil-service bureaucracy |
| 9 | Impersonal bureaucracy |
| 10 | Charismatic dictator |
| 11 | Non-charismatic leader |
| 12 | Charismatic oligarchy |
| 13 | Religious dictatorship |
| 14 | Religious autocracy |
| 15 | Totalitarian oligarchy |

On an inhabited world, Government is **2D6-7 + Population**, limited to 0-15.
Law is then **2D6-7 + Government**, also limited to 0-15. Government 0 forces
Law 0.

### Starport class and baseline services

Starport class is generated from **2D6-7 + Population**: 2 or less is X, 3-4
is E, 5-6 is D, 7-8 is C, 9-10 is B, and 11 or more is A. The class creates
the primary facility's baseline, shown below. These are starting capabilities,
not promises that capacity remains open: damage, occupation, missing stock,
local events, and later development can change a particular facility.

| Class | Broad quality | Fuel sold | Repair and yard baseline | Other baseline services |
| --- | --- | --- | --- | --- |
| A | Excellent | Refined and unrefined | Repair shop; yard up to 100,000 tons; medical 3 | Chandlery, ordnance, personnel, banking, inhabited-world authority office |
| B | Good | Refined and unrefined | Repair shop; yard up to 20,000 tons; medical 3 | Chandlery, ordnance, personnel, banking, inhabited-world authority office |
| C | Routine | Unrefined | Repair shop; yard up to 5,000 tons; medical 2 | Chandlery, ordnance, personnel, banking, inhabited-world authority office |
| D | Poor | Unrefined | No repair shop or yard; medical 1 | Chandlery, banking, inhabited-world authority office |
| E | Frontier | None | No repair shop or yard; medical 0 | Inhabited-world authority office only |
| X | No operational primary port | None | None | None |

The class also sets the amount of ordinary daily market stock before purchases
consume it. Common-stock lots are 6D at A, 4D at B, 2D at C, 1D at D or E,
and zero at X. Each common lot is 2D x 10 tons at A and 1D x 10 tons elsewhere.
Other trade-stock lots are 4D at A, 3D at B, 2D at C, 1D at D or E, and zero
at X; each commodity supplies its own lot-size roll.

### Technology Level

Technology Level measures local scientific and manufacturing capability. It
does not guarantee a facility, stock, trained labor, authority, or parts.
Imported equipment can exceed local TL, while a damaged or absent yard can
make locally understood work unavailable.

| TL | Broad local capability |
| ---: | --- |
| 0 | No technological base; uninhabited worlds have TL0 |
| 1-3 | Primitive technology through early steam-powered industry |
| 4-6 | Industrial technology through electrification, combustion, fission, and increasingly capable computers |
| 7-8 | Reliable orbital flight, followed by practical travel within the stellar system |
| 9 | Gravity technology and the first steps toward Jump; the lowest TL in the current vessel catalogue |
| 10-11 | Early interstellar technology and increasingly autonomous computers |
| 12-14 | Mature interstellar technology, planetary engineering, armor, computers, and weapons |
| 15 or more | High stellar technology |

For an inhabited world, TL begins with **1D6**. Apply +6 for Starport A, +4
for B, +2 for C, and -4 for X. Apply +2 for Size 0-1 or +1 for Size 2-4; +1
for Atmosphere 0-3 or 10-15; +1 for Hydrographics 0 or 9 or +2 for 10; +1
for Population 1-5 or 9 or +2 for Population 10; and the Government modifier
+1 at 0 or 5, +2 at 7, or -2 at 13-14.

Hostile environments impose minimums needed to sustain the recorded
population: TL4 for Hydrographics 0 or 10 with Population 6 or more; TL5 for
Atmosphere 4, 7, or 9; and TL7 for Atmosphere 0-3, Atmosphere 10-12, or an
Atmosphere 13-14 world with Hydrographics 10. Use the rolled result or the
applicable minimum, whichever is higher.

### Trade codes

Trade codes are classifications derived from the UWP, not additional random
traits. A world can have several. Cepheus Trader uses the following codes in
its commodity market:

| Code | Classification | UWP requirement |
| --- | --- | --- |
| Ag | Agricultural | Atmosphere 4-9, Hydrographics 4-8, Population 5-7 |
| As | Asteroid | Size 0, Atmosphere 0, Hydrographics 0 |
| Fl | Fluid Oceans | Atmosphere 10+, Hydrographics 1+ |
| Ga | Garden | Size 6-8, Atmosphere 5, 6, or 8, Hydrographics 5-7 |
| Hi | High Population | Population 9+ |
| Ht | High Technology | TL12+ |
| Ic | Ice-Capped | Atmosphere 0-1, Hydrographics 1+ |
| In | Industrial | Atmosphere 0-2, 4, 7, or 9 and Population 9+ |
| Na | Non-Agricultural | Atmosphere 0-3, Hydrographics 0-3, Population 6+ |
| Ni | Non-Industrial | Population 4-6 |
| Po | Poor | Atmosphere 2-5, Hydrographics 0-3 |
| Ri | Rich | Atmosphere 6 or 8, Population 6-8 |
| Va | Vacuum | Atmosphere 0 |

Each non-common commodity names up to two favorable purchase codes and two
favorable sale codes. If several listed codes match the world, only the
strongest applicable DM is added on each side of the negotiation. A code
therefore changes the chance of reaching a purchase or sale outcome; it does
not multiply the price by itself. Common Goods have no trade-code DM.

### Law, legality, and services

Law Level 0 means no general legal restrictions. Levels 1-3 are low law, 4-6
medium law, 7-9 high law, and 10 or more extreme law. The number is used
directly; the band is only a description. A higher Law Level increases the
ordinary enforcement and customs environment, while a captain's actual
authority still comes from government service, a commission, a warrant, a
lawful order, or immediate necessity as described later in these rules.

The commodity catalogue assigns a restriction threshold to seven controlled
goods. At the threshold the good is **restricted**. Three Law Levels above the
threshold it becomes **prohibited**.

| Commodity | Restricted at | Prohibited at |
| --- | ---: | ---: |
| Personal Weapons and Armor | Law 2-4 | Law 5+ |
| Military Supplies | Law 3-5 | Law 6+ |
| Liquor and Other Intoxicants | Law 5-7 | Law 8+ |
| Gambling Equipment | Law 6-8 | Law 9+ |
| Live Animals | Law 7-9 | Law 10+ |
| Pharmaceuticals | Law 8-10 | Law 11+ |
| Cybernetics | Law 9-11 | Law 12+ |

All other catalogued commodities are ordinarily legal regardless of Law
Level. Local events, a warrant, task terms, or an authority order can still
govern a particular lot. Law 8 or more also makes clearance a baseline
requirement at the primary port. The ordinary market tariff is 5% plus 0.5%
per Law Level.

During a compliant customs inspection, prohibited ordinary cargo is
confiscated and a fine of 10% of its base value is collected, limited by the
credits currently available. Restricted cargo remains aboard after a
compliant inspection unless another displayed rule or order says otherwise.
Any unpaid part of the assessment becomes a warrant with a bounty equal to
that balance. Refusing a lawful Inspection or Military encounter with any
posture except Comply or Surrender begins vessel combat and files one
Cr10,000, perfect-evidence refusal warrant in that system.

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

The captain's ordinary purchase and sale negotiations each roll:

**2D6 + Broker + CHA DM + strongest applicable trade-code DM - 2**

Compare that total with 8 and use its Effect:

| Effect | Purchase price before events and tariff | Sale price before events and tariff |
| ---: | ---: | ---: |
| 6 or more | 80% of base | 130% of base |
| 0 through 5 | 90% of base | 115% of base |
| -5 through -1 | 100% of base | 102% of base |
| -6 or less | 120% of base | 100% of base |

The local tariff is added to the purchase price and deducted from the sale
price, with fractions rounded against the trader: purchases round up and
sales round down. A shortage moves both price outcomes one tier higher; a
surplus moves both one tier lower; a shipping disruption moves only purchases
one tier higher; and a market recovery changes stock without shifting a price
tier. When the exchange both stocks and buys an item, its ordinary bid is
capped at least Cr1 below its ask, so immediate unloading cannot manufacture
profit. A separately reserved private buyer can cross that public spread.

| Event | Stock | Purchase tier | Sale tier | Duration |
| --- | ---: | ---: | ---: | ---: |
| Shortage | 50% | +1 | +1 | 7-21 days |
| Surplus | 150% | -1 | -1 | 7-21 days |
| Shipping disruption | 50% | +1 | 0 | 7-21 days |
| Market recovery | 125% | 0 | 0 | 7-21 days |

On each market day a system has a 1-in-37 chance to begin one such event for
one catalogued commodity. Its agency notice must still reach a remote captain
through ordinary mail.

Price landmarks show the absolute universe-wide span for that commodity, not
the odds of receiving a particular quote. Low purchase prices and high sale
prices are favorable. Market reports record their place, date, source, and
confidence; they do not update themselves while travelling.

Prohibited goods cannot be bought through the open exchange. Every cargo lot
has an owner, commodity, quantity, physical ship or facility, and provenance.
Speculative cargo belongs to the captain's estate; freight belongs to its
principal. Splitting a transaction cannot duplicate credits or cargo.

Quantities are retained to 0.001 ton. A purchase's final total rounds up to a
whole credit; a sale's final proceeds round down. Speculative cargo can be sold
in its origin system, but the bid/ask rule normally makes that an immediate
loss.

### Commodity reference

The following is the complete generic commodity catalogue. A lot expression
such as `2D x 5 tons` means roll two dice and multiply their sum by five tons.
The purchase and sale columns give the trade-code DMs used by the negotiation;
only the strongest matching DM on each side applies. Common Goods use the
starport-dependent lot rules given under Starport class and have no trade-code
DM.

| D66 | Commodity | Base Cr/ton | Trade lot | Purchase DMs | Sale DMs |
| --- | --- | ---: | ---: | --- | --- |
| Common | Basic Consumable Goods | 1,000 | Starport lot | -- | -- |
| Common | Basic Electronics | 25,000 | Starport lot | -- | -- |
| Common | Basic Machine Parts | 10,000 | Starport lot | -- | -- |
| Common | Basic Manufactured Goods | 20,000 | Starport lot | -- | -- |
| Common | Basic Raw Materials | 5,000 | Starport lot | -- | -- |
| Common | Basic Unrefined Ore | 2,000 | Starport lot | -- | -- |
| 11 | Electronics | 100,000 | 1D x 5 tons | Ht +1, In +2 | Ni +1, Po +1 |
| 12 | Sporting Equipment | 5,500 | 2D x 5 tons | In +2, Ri +2 | Hi +2, Ni +2 |
| 13 | Agricultural Equipment | 150,000 | 1D tons | In +2, Ri +1 | Ag +2, Ga +1 |
| 14 | Animal Products | 1,500 | 4D x 5 tons | Ag +1, Ga +2 | Hi +1, Ri +2 |
| 15 | Collectibles | 50,000 | 1D tons | In +1, Ri +2 | Hi +1, Ni +1 |
| 16 | Computers and Handcomps | 150,000 | 2D tons | Ht +2, In +1 | Na +1, Ni +1 |
| 21 | Crystals and Gems | 20,000 | 1D x 5 tons | Ni +2, Na +1 | In +1, Ri +1 |
| 22 | Cybernetics | 250,000 | 1D x 5 tons | Ht +2, Ri +1 | Na +1, Ni +1 |
| 23 | Food Service Equipment | 4,000 | 2D tons | In +2, Na +1 | Ag +1, Ni +1 |
| 24 | Furniture | 5,000 | 4D tons | Ag +1, Ga +2 | Hi +1, Ri +2 |
| 25 | Gambling Equipment | 4,000 | 1D tons | Hi +1, Ri +1 | Na +1, Ni +1 |
| 26 | Vehicles | 160,000 | 1D tons | Ht +2, Ri +1 | Ni +2, Po +1 |
| 31 | Grocery Products | 6,000 | 1D x 5 tons | Ag +3, Ga +2 | Hi +1, Ri +2 |
| 32 | Household Appliances | 12,000 | 4D tons | Hi +2, In +3 | Na +1, Ni +2 |
| 33 | Industrial Supplies | 75,000 | 2D tons | In +3, Ri +2 | Na +1, Ni +2 |
| 34 | Liquor and Other Intoxicants | 15,000 | 1D x 5 tons | Ag +2, Ga +1 | In +1, Ri +2 |
| 35 | Luxury Goods and Rarities | 150,000 | 1D tons | Ag +1, Ga +2 | In +1, Ri +2 |
| 36 | Manufacturing Equipment | 750,000 | 1D x 5 tons | In +2, Ri +2 | Na +1, Ni +2 |
| 41 | Medical Equipment | 50,000 | 1D x 5 tons | Ht +2, Ri +2 | Hi +1, In +2 |
| 42 | Petrochemicals | 10,000 | 2D x 5 tons | Na +2, Ni +2 | Ag +1, In +2 |
| 43 | Pharmaceuticals | 100,000 | 1D tons | Ht +3 | In +2, Ri +1 |
| 44 | Polymers | 7,000 | 4D x 5 tons | In +2, Ri +1 | Ni +2, Va +1 |
| 45 | Precious Metals | 50,000 | 1D tons | As +3, Ic +2 | In +1, Ri +2 |
| 46 | Radioactive Ore | 1,000,000 | 1D tons | As +2, Ni +3 | In +2, Ht +1 |
| 51 | Robots and Drones | 500,000 | 1D x 5 tons | Ht +3, In +2 | Ni +1, Ri +2 |
| 52 | Scientific Equipment | 50,000 | 1D x 5 tons | Ht +3, Ri +2 | Hi +2, Ni +1 |
| 53 | Survival Gear | 4,000 | 2D tons | Ga +2, Ri +2 | Fl +2, Va +1 |
| 54 | Textiles | 3,000 | 3D x 5 tons | Ag +3, Ni +2 | Na +1, Ri +2 |
| 55 | Construction Supplies | 20,000 | 2D x 5 tons | Ag +3, Ni +2 | In +2, Na +1 |
| 56 | Raw Materials | 20,000 | 2D x 5 tons | As +2, Va +1 | In +2, Na +1 |
| 61 | Live Animals | 25,000 | 5D x 5 tons | Ag +3, Ga +2 | Hi +1, In +2 |
| 62 | Children's Toys | 5,000 | 2D x 5 tons | In +2, Ri +2 | Hi +2, Ni +1 |
| 63 | Medical Laboratory Equipment | 50,000 | 1D x 5 tons | Ht +2, Ri +3 | In +2, Na +2 |
| 64 | Military Supplies | 150,000 | 2D tons | Ht +3, In +2 | Hi +2, Ni +2 |
| 65 | Personal Weapons and Armor | 30,000 | 2D tons | In +3, Ri +2 | Ni +2, Po +2 |

### Research and reservations

Physical canvassing and online research take one to six game minutes. A black-
market search takes six to 24 minutes and applies -1 to the task. A hired local
Broker costs Cr500, supplies Broker-2, and reports in one to three minutes; the
named crewmember remains liaison. Physical, black-market, and hired searches
use CHA; online research uses INT and Computer. All other searches use Broker.
The assigned crewmember must be fit and on watch.

A commodity-specific buyer search requires matching player-owned speculative
cargo aboard the commanded ship. Freight, contract cargo, and unique objects
do not qualify. This eligibility check occurs before any hired-broker
commission is charged. A completed buyer lead cannot cover more matching cargo
than remains aboard, and produces no lead if none remains.

Research is an Average task. Effect below -5 produces no reliable lead.
Otherwise a commodity-specific supplier or buyer search records confidence
**70% + three times Effect**, limited to 40%-100%. Its quoted price and
quantity range is the underlying result plus or minus **25 - Effect** percent,
limited to a spread of 5%-30%. The finite lead quantity is the observed
quantity multiplied by **100% + three times Effect**, with Effect limited to
-5 through +10.

A completed lead records a finite quantity, price range, source, observation
date, confidence, expiry, and revision. Reserving it places 10% of estimated
value in escrow. Release or expiry does not refund that opportunity payment.
An unreserved lead expires after seven game days. A reservation's displayed
expiry is three game days after it is made. Reservations are for a positive
whole-ton quantity no greater than the lead's finite quantity. The reservation
can therefore be lost, and old intelligence cannot be reused as infinite
stock.

### Carriage and tasks

Ordinary freight, passengers, and electronic mail use a standing declaration
for one destination. The captain states maximum freight, eligible passenger
capacity, and whether to accept mail. Departure previews a concrete manifest
and brokerage, then loads those exact offers if they remain valid. Freight is
a titled physical lot; passengers occupy actual eligible accommodation; mail
uses the declared route but never causes a voyage.

Standing carriage pays by billed route hop. The current offer generator treats
each edge in the operational route as one billed parsec even when its 3D
length differs, and the displayed signed offer controls. A regular passenger
needs one uncommitted passenger berth; low passage needs one low berth. High
and middle passengers require someone aboard with Etiquette, and low
passengers require someone aboard with Medicine.

| Carriage | Gross rate |
| --- | ---: |
| Freight | Cr3,500 per ton per billed route hop |
| High passage | Cr25,000 per passenger per billed route hop |
| Middle passage | Cr10,000 per passenger per billed route hop |
| Steerage | Cr5,000 per passenger per billed route hop |
| Low passage | Cr2,000 per passenger for the passage |

The ledger supports all four ordinary classes, but the current local offer
generator emits Middle passage only. Charters use the High rate and couriers
use their signed special rate; no ordinary generated offer presently emits
High, Steerage, or Low passage.

The departure preview is authoritative: it names the offers, occupied
capacity, gross revenue, brokerage, and revision that commit will use. If an
offer or its revision changes first, departure is rejected for a fresh
preview instead of loading a different manifest. Each selected freight offer
also charges a stable brokerage of Cr100-Cr300 per whole ton, derived from that
offer; passage has no additional brokerage.

Other offers become durable Tasks. Current types include freight, passage,
purchase orders, forward sales, supply commitments, charters, couriers, and
bounties. Each offer states closing time, origin, destination, performing
capacity, collateral, payment, deadline, failure penalty, and whether partial
performance is allowed.

Generated offers are local work routed through nearby settled systems. Their
normal delivery deadline is **14 days + 7 days per route hop** after issue.
Freight offers are 5-30 tons and pay Cr3,500 per ton per hop; middle-passage
offers carry 1-6 travelers at Cr10,000 each per hop. Purchase orders pay
Cr6,000 per ton per hop, forward sales Cr7,000, and four-performance supply
commitments Cr12,000. Charters pay high-passage rates, couriers Cr30,000 per
hop, and combat bounties Cr50,000 per hop. Route scarcity and active local
events can adjust the displayed payment, so the signed offer always controls.

During an eligible passenger grace period, late payment deductions equal 10%
of the offered payment for each started late day, never more than the payment.
Ordinary passengers have three days of grace; charters and couriers have one.
Other generated work has no late-delivery grace. Partial purchase,
forward-sale, and supply terms pay the delivered fraction. Freight and
passenger terms require the full consignment unless their signed offer
expressly says otherwise.

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

Before custody transfers, an unresolved claim may be withdrawn and an awarded
task may be cancelled without performance liability. After loading, entrusted
cargo or passengers can be returned only while docked at the origin. Default
forfeits collateral and assesses the displayed failure penalty plus capped
non-delivery liability, but never takes more liquid credits than are present.
A dispute requires written grounds and remains an obligation while its sealed
filing travels to the issuing office.

### Finance and title

An independent trader purchase begins with 20% equity and 80% secured debt.
Monthly principal is purchase price divided by 240, rounded up, and continues
each accounting month until the recorded principal reaches zero; mandatory
insurance is separate. A missed principal or insurance installment has one
accounting month of grace before default action.

For a non-naval command, annual mandatory insurance is **1.02% of purchase
price, rounded up, + Cr15,000**; its monthly escrow is that amount divided by
12, rounded up. Principal and
insurance are paid together every 30 game days. Restricted operating credit
may cover the insurance portion, but only liquid funds can reduce a trader's
secured principal.

Privateer vessels are sponsor-owned and naval vessels are institution-owned.
Restricted operating credit pays authorized vessel expenses before liquid
cash and can pay required insurance, but it does not pay secured principal,
private trade, collateral, private messages, fines, or other personal costs.
Command, possession, title, and debt are separate facts.

The command account journal distinguishes liquid, restricted operating,
reserved, and secured-principal postings. Every balance change and its exact
resulting balance are committed in the same authoritative transaction as the
purchase, payment, hold, release, income, transfer, or financing event. The
journal is retained indefinitely and can be filtered by transaction class or
vessel. An estate created before the journal receives a single carried-forward
opening entry; the game does not invent a historical reconstruction.

Task income does not enter liquid funds when delivery is merely certified.
Until the settlement filing reaches the issuing office and its remittance
reaches the captain, Accounts reports the payment and collateral release as
pending. The displayed resolution day is an estimate based on the known mail
route and may move as carrier service changes. Pending income is never included
in spendable or available cash.

| Recorded vessel title | Meaning |
| --- | --- |
| Owned With Lien | Privately registered ownership subject to secured principal |
| Owned Clear | Privately registered ownership with no secured principal |
| Sponsor Owned | A private sponsor owns the vessel and grants command under terms |
| Institution Owned | A navy or other public institution owns the issued command |
| Prize Custody | The captor possesses the vessel while title awaits settlement |
| Stolen Registry | Possession has no recognized lawful title |
| Court Impound | A court or lender controls the vessel pending disposition |

A naval captain can forge a positive ship-expense receipt no greater than the
available service balance, moving that amount to personal cash immediately.
All false receipts remain for the next accounting audit. Let **A** be their
total, **N** their count, and **E** the legitimate authorized expenses that
month. Detection percent is:

**ceil(((5 x bit-length(A)) + min(10 x (N - 1), 20)) x
(50 + floor(100 x A / (A + E))) / 100)**

The result is limited to 1%-90%. Here bit-length is 1 for Cr1, 2 for Cr2-3,
3 for Cr4-7, and so on. A detected audit files a forgery warrant at the
captain's origin with bounty **max(2 x A, Cr100)**; the warrant then propagates
normally by mail. Legitimate expense totals and pending receipt totals reset
at the accounting boundary.

When a grace period expires, the lender files a private impound order at the
captain's origin. It is enforceable elsewhere only after the signed message
arrives. Posting one overdue installment clears the default and sets the next
due date one accounting month later. An irrecoverable bankruptcy petition is
available only while docked and in default: the fleet is liquidated, a named
successor receives a replacement command in the original starter class under
a new lien, and the captain's career, legal, and estate history remains.

Ship and crew exchanges are finite daily port markets. Ship offers can draw
from every active Jump-capable catalog design that local TL can support; an
offer that another buyer claims disappears. A used ship retains its actual
age, use, damage, and any latent construction quirk, so advertised condition
is not a diagnosis. Purchase settles price, trade-in, title, lien, condition,
ordinary cargo, and assigned crew atomically. A trade-in is refused while the
old vessel holds active Tasks, entrusted cargo, or passengers. Keeping it
leaves its obligations and physical stores aboard that hull.

An operational construction yard also offers every admitted catalog design
whose displacement and component TL fit that facility. A new
commission takes a 20% deposit, finances the remaining 80%, and places the
named but undelivered hull in Fleet for the catalog construction time. It
cannot accept a captain, stores, or active command before delivery. Warranty,
maintenance, berth aging, and monthly finance dates begin when construction
completes, not when the contract is signed.

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

Fuel and cargo transactions use 0.001-ton units. Dockside fuel may be bought
in any positive quantity at that resolution, allowing an exact fill after
fractional consumption. The final charge rounds up to a whole credit, so
splitting a purchase cannot reduce its price. Refined fuel costs Cr500 per ton
and unrefined fuel Cr100 per ton where the port table permits it. A tank can
contain both; the unrefined portion is tracked. Every power-plant or Jump burn
draws refined and unrefined fuel proportionally to the tank mixture (with
deterministic integer rounding), so the Jump penalty follows whether that
specific burn consumes any unrefined fuel rather than assuming refined fuel is
always used first.

New ships begin with 30 person-days of provisions for every awake-accommodation
place and can store at most 180 per place. Away from a berth, one person-day is
consumed daily by each represented living crewmember and awake passenger.
Docked ordinary crew arrange their own meals. The captain consumes one ship
person-day or, if stores are empty, automatically buys an ashore meal from
liquid credits at twice the ship's package-average person-day price. A person
may go three days without food; subsequent days require the CE Routine (+2)
Endurance check with cumulative DM-1 per prior check, and failure causes 1D6
damage that cannot heal until the person is fed. Shipboard water is assumed
available. A dockside monthly package costs Cr2,000 per ordinary or compact stateroom, Cr3,000 per crew
berth or barracks allocation, Cr5,000 per high-class stateroom, Cr100 per low
berth, and Cr100 per emergency-berth place.

Ordinary berthing costs Cr100 for the first six game days, then Cr100 for each
additional started day. The accrued charge must be clear before departure or
yard work. Restricted operating credit is used first for authorized ship
services.

### Routine upkeep, wear, and warranty

Routine maintenance is continuing onboard work charged every 30 game days at
one twelfth of 0.1% of ship value. Paying it prevents neglect-related
degradation; it does not heal combat damage, reset age, conceal a known defect,
or replace a destroyed component. If the operating account cannot cover the
complete charge, no partial payment is taken, the cycle is recorded as missed,
and the normal neglect check applies.

The monthly charge is purchase price divided by 12,000, rounded up. On a
missed cycle roll **2D6 + consecutive missed cycles**. A total below 8 adds no
damage; on 8 or more a further 1D6 applies one hit on 1-3, two on 4-5, or
three on 6 to an eligible installation. Paying a later routine charge resets
the consecutive-miss count and prevents that month's check, but does not erase
an earlier missed cycle's neglect hits.

Installations separately record calendar age, operating time, Jump and
maneuver cycles, and stressful skimming cycles. Hidden construction quirks can
manifest only through relevant use. Once manifested, their symptoms are
reported; routine upkeep does not erase them.

A new ship carries ordinary warranty until either five years or 200 transits.
Gas-giant skimming voids it. A qualifying defect can be removed without charge
by a capable facility while coverage remains; warranty expiry does not itself
cause a failure. A newly replaced component records a five-year component
warranty date; a reconditioned replacement records one year.

At a paid monthly maintenance boundary while docked at an operational Class
A-C repair shop, each covered quirk is independently detected and removed on
4-in-6 if it has manifested or 1-in-6 if it is still latent. Warranty coverage
requires both the five-year date and 200-transit limit to remain unexpired and
must not have been voided.

Age and transit use take whichever is further through the five-year/200-
transit reference. The first 20% is a declining shakedown-risk period, the
ordinary useful-life risk is 20 chances per million per accounting month, and
wear-out rises after 180% of the reference. Each gas-skimming cycle also counts
as a transit for wear. These probabilities govern hidden defect attachment;
ordinary status checks reveal symptoms only after a relevant use manifests
them.

### Damage, repair, refit, and replacement

Battlefield repair, proper repair, refit, refurbishment, and replacement are
different operations.

- A battlefield repair temporarily covers a sustained combat result and
  expires when the encounter ends.
- A proper repair removes eligible underlying damage at a capable facility.
  It takes one game day per sustained hit, has no parts charge beyond accrued
  berth because routine upkeep covers ordinary repair items, and does not
  change age or service history. It remains tied to that berth: a successful
  departure cancels unfinished work and its completion event, leaving the
  underlying damage unchanged.
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

Proper repair requires an operational repair shop able to support the ship's
TL. Refit yard limits are Class C for hulls through 800 tons, Class B through
2,000 tons, and Class A above that, subject also to the port's published yard
capacity. Only one yard activity can be active at a time. A refit clears
repairable physical damage, temporary patches, neglect damage, and minor
latent quirks; it retains hull and component age, usage counters, destroyed
installations, and deeper component faults. New and reconditioned replacement
quotes use the actual catalog component and give only the replaced component
a new installation record and component warranty.

## Flight, fuel, and Jump

**Rules lineage:** Cepheus Engine maneuver, Jump, course plotting, fuel, and
misjump rules; compatible admitted third-party operational procedures; Cepheus
Trader flight plans and 3D geometry. Alternate interstellar-drive systems are
not used.

### Flight plans

A Flight Plan is the ship's executable route, separate from Tasks and Known
Universe. Each checkpoint has either **Hold** authority, which waits for the
captain's arrival watch, or **Through** authority, which may resolve contacts
under standing orders and continue while the captain is away. Separately, one
terminal marker is always attached to the last step and ends the plan after
that step completes. Filing commits only the reviewed revision. Completed
steps and accepted custody cannot be undone by editing later steps.

The door presents adjacent implementation steps as logical route items. A new
charted leg first selects the target system, then its first in-system
destination (port, world or moon, gas giant, or belt), and finally standard or
private/offset emergence. The Jump and its first normal-space maneuver form one
charted leg; further in-system stops or another Jump can follow. Any future item can have its checkpoint authority or settings changed,
be deleted, or have a charted leg inserted before or after it. The active
physical operation cannot simply disappear, but an active normal-space
destination can be redirected. Future items can be reordered and charted
destinations replaced; dependent charted loci are
rebuilt, while an ordering that would detach a body-specific or coordinate
operation from its actual locus is refused.

Revising a plan does not restart its current physical operation. A change only
to later items or encounter policy preserves the active maneuver, Jump, Belt
Cycle, frontier-fuel operation, or onboard refining work and its scheduled
completion. Changing the active destination in normal space instead computes a
new bounded-thrust intercept from the ship's actual position and velocity. Once
Jump begins, its emergence destination is physically fixed, but later items and
encounter policy remain editable. Completed steps remain visible but locked.

Generated task routes and ordinary routes default to Through at every
checkpoint, including the terminal step, so they can complete while the
captain is away. Player Preferences can change either route type to generate
Hold checkpoints instead. The setting affects future proposals, not filed
plans; Hold must therefore be chosen explicitly either as that preference or
in the editor.

A plan can include port purchases, wilderness water or ice collection,
gas-giant skimming, Jump loci, known systems, surveyed coordinates, imported
plotted courses, and mining-drone Belt Cycles at catalogued planetoid belts.
Preview calculates known time, fuel, purchases, and obligation warnings. When
proper repair is active, preview marks the first planned berth-clearing step and
requires acknowledgement that departure will cancel the work; refit and other
yard operations remain departure blockers. Warnings are numbered once below
the route and referenced beside every affected step. If a required service,
source, payment, or course is no longer valid when reached, the plan holds
instead of silently substituting another one.

Each ship also has default encounter standing orders, edited from Task
Management. A new plan begins with that default; changing the ship default does
not silently rewrite a filed plan. The Flight Plan editor can load the default,
keep different orders for this plan, or save its orders back as the ship
default. For every encounter type, the order names an ordinary response and a
separate Fight condition: Never, Always, or only when the estimated combat
outlook meets a percentage. Hostile orders additionally carry an ordered list
of permitted emergency fallbacks. Preview separately warns and requires
acknowledgement whenever filed orders may attack a contact that has not attacked
the ship.

### Belt prospecting and mining

A Belt Cycle leaves the primary berth, travels to the named belt, and exposes
prospecting, survey, mining, refining, field recovery, and egress as distinct
scheduled phases. It must include a later validated port egress. The ship
protects enough power-plant fuel for that filed route using **184 hours per
remaining Jump**, plus one additional day. Work repeats only while the lode,
cargo capacity, and that reserve remain. A failed recovery attempt aborts the
cycle immediately to the filed egress; it never leaves the ship holding for an
offline captain.

Prospecting uses an Average INT/Trade (Prospector) task in six-hour watches.
Discoveries create persistent shared lodes, but the observation belongs only
to the discovering captain. A discovery is not an exclusive claim: another
crew can independently find and work the same deposit. Ordinary lodes range
from ten to one million tons of feedstock. Their generated belt composition,
extent, grade, and remaining quantity persist after every extraction.

The game does not offer a blind sensor sweep for active miners across an
entire belt: its physical volume makes geometric coincidence an unhelpful
player action. Another operation can instead become contactable through a
traffic record, transponder or radio emission, shared observation, contract,
or an interception watch already established at that catalogued belt. System
population and local order can affect how many such leads exist without
pretending that every ship in the belt occupies the same sensor-local point.

One installed mining-drone set handles **1D6 x 10 tons of feedstock per day**.
A successful daily Prospector task produces the full result; Effect -1 halves
it; Effect -2 or worse produces nothing and starts a 1D6-hour Mechanic
recovery. Successful recovery resumes work and failed recovery begins egress.

Without a mineral refinery, recovered feedstock is stowed as Basic Unrefined
Ore. A refinery separates a grade-dependent output up to its daily capacity:
silicate and carbonaceous finds become Basic Raw Materials, metal-bearing ore
becomes Raw Materials, hydrocarbons become Petrochemicals, and exceptional
finds can become Crystals and Gems, Precious Metals, or Radioactive Ore.
Discarded tailings do not consume cargo space. Icy output can instead top up
unrefined fuel when the ship has fuel-processing equipment. Every mined cargo
lot retains its source system, body, and persistent lode identity.

Task delivery occurs when docking completes, not when an arrival checkpoint
becomes ready. Preview marks a task deadline in red when projected docking is
late. It also marks a timely Hold checkpoint when the captain must take arrival
watch before the deadline. If the captain waits, the task can default even
though the ship reached its checkpoint earlier.

Ordinary interplanetary transfer uses continuous acceleration to the midpoint
and continuous deceleration afterward. For a stationary endpoint estimate:

**travel time in seconds = 2 x square root of (distance in meters / acceleration
in meters per second squared)**

Flight time uses the orbital positions at departure and the vessel's actual
acceleration, so a moving destination or later departure can change the
answer.

An underway ship may replace its course during any in-system maneuver. The
new estimate starts from the ship's position and velocity relative to the
moving destination at the instant the replacement is filed; it does not
pretend that the ship has stopped. The navigation computer finds the shortest
whole-second two-burn intercept whose acceleration never exceeds the ship's
effective maneuver thrust and which reaches the destination with the
destination's orbital velocity. A velocity shared by the whole system cancels
out of this calculation. The captain may turn back to the primary port,
redirect to either conventional Jump traffic locus, or redirect to a selected
belt or lawful frontier fuel source. A ship already in Jump space cannot maneuver or replace that
physical Jump leg.

### Jump range and fuel

A Jump drive rating is the maximum parsecs in one Jump. Any positive
sub-parsec Jump counts as Jump-1, and every fractional distance rounds up to
its Jump number for fuel and tape price. Required fuel is:

**Jump fuel tons = 0.1 x hull displacement tons x Jump number**

Jump can begin or end only outside the union of the 100-diameter exclusion
zones of relevant massive bodies. By traffic convention, the published
departure locus is north of the system ecliptic and the published arrival locus
is south. Either is physically legal for departure, although using the inbound
locus is bad traffic practice. Maneuver flight between a port and either legal
locus uses the ship's real thrust, position, time, and fuel rules.
Away from a berth, the ship's ordinary power-plant allocation burns
continuously at the rate implied by its catalogued endurance. Fractional burn
is retained exactly between scheduled settlements; an older save establishes
a fresh timestamp without retroactive consumption.

An onboard plot is an EDU/Astrogation task against 8 with an equipment DM of
**4 - Jump number** and takes 1D6 kiloseconds. A failed plot normally holds the
plan for recalculation; the captain may instead give explicit authority to use
it, which makes the Jump an automatic misjump. A fresh commercial course tape
skips that task, costs Cr1,000 per Jump number, and is sold only at an
operational Class-D-or-better origin for a populated destination. Initiation
is an Average EDU/Engineer (Jump) task taking 1D6 x 10 seconds. Its Effect
contributes to the Jump success result. Drive damage and unrefined fuel apply
normally.

A fresh tape is purchased while clearing a berth. A Jump that follows a body
or traffic-locus stop without docking must use onboard plotting; returning to
port makes a new tape purchase possible.

The Jump success total is:

**2D6 + initiation Effect - 2 per Jump-drive hit - 2 if any Jump fuel burned
is unrefined**

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
avoid them. An inaccurate emergence or misjump that damages a subsystem also
creates a durable Engineering Casualty Report naming the route, timing
consequence, damaged installation, resulting hit total, and operational
effect. A connected captain receives it at the next safe prompt; a disconnected
captain receives it on the next sign-on. It remains until positively
acknowledged, while the game clock and any still-valid filed plan continue.

The same report and acknowledgement rule applies when an exceptional
fuel-processing failure damages a subsystem or unpaid routine upkeep causes
neglect damage. These are one report per damaging operation or upkeep check,
not one report per damage hit. Combat damage remains aggregated in the combat,
encounter-result, and Command Loss Report procedures instead of entering this
non-combat report queue.

A standard emergence uses a safe point outside every 100-diameter exclusion
zone and nearest the filed first in-system destination. The ship then performs
ordinary interplanetary travel to that port or named body. A private emergence
uses a seeded offset direction and additional standoff distance from the same
target. It avoids the ordinary published-arrival contact check but lengthens
the normal-space maneuver. It does not bypass port law: a later docking still
performs the port contact/inspection procedure, and high-law ports require
inspection there. Deliberately emerging at the primary world's published
departure locus remains possible, but conflicts with outbound traffic and
carries the same increased contact risk as using the wrong lane for departure.

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

A scoop permits collection without a processor. The captain explicitly chooses
whether an installed processor refines the collected batch; otherwise it enters
the tanks as unrefined fuel. Processing is Average (8+) Engineer (Power)/EDU.
Failure doubles processing time. Effect -6 or worse also causes one sustained
hit to the Jump drive, falling back to the maneuver drive and then the fuel
system when necessary. Unrefined fuel already aboard may be processed to
0.001-ton resolution as a stationary Flight Plan step while docked or safely holding. Selected feedstock
is protected from power burn, and an off-berth attempt requires enough other
fuel for the worst-case doubled duration.

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
departing and arriving vessels. A direct carried hop pays **Cr100 + Cr1 per
envelope** on delivery. Accepting a mailbag never chooses or changes the
vessel's route. News, offers, warrants,
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

Institutional affiliation follows the same causal boundary. A captain's home
BBS and polity are known locally at login. A League name, when present, is an
institutional affiliation rather than part of the polity's name. The Known
Universe shows a remote system's BBS, polity, and optional League only after
that system's mapping is `KnownPublic`; a private observation, withheld chart,
secret chart, direct filing, or still-travelling public dispatch does not leak
those fields. A League rename updates current dossiers and console identity,
but it does not rewrite already dispatched historical news.

The first valid private claim for a newly discovered settled system is judged
at Earth. Its award is Cr218,000. Competing filings are decided when they
physically reach the adjudicating office, and the public notice begins its own
outward carriage only after judgment.

A valid filing includes canonical 3D position and distinguishing stellar
observations, authenticated captain and discovering-ship identities,
observation and dispatch times with custody and route provenance, and survey
evidence of an established population rather than a transient ship, camp,
cache, or new player base. The first valid receipt committed at Earth's
repository wins; discovery time, dispatch time, arrival at another Federation
office, and the claimant's return do not reserve priority. A public report
starts at the source. An encrypted direct filing remains private in transit;
if it wins, Earth originates the authoritative public notice.

## Traffic and encounters

**Rules lineage:** Cepheus Engine encounter checks and opposed crew tasks;
admitted third-party Open Game Content for traffic, ports, naval operations,
and piracy; Cepheus Trader contact geography and causal response.

### Where traffic exists

Traffic concentrates at ports, inhabited worlds, gas giants, Jump arrival
loci, and Jump departure loci. Ordinary interplanetary space is sparse unless
a ship's actual trajectory makes contact or interception possible. Jump space
contains no ordinary encounters.

Scheduled route traffic is observable for one game hour before through one
hour after its route edge. Local persistent traffic is assigned 55% to ports,
30% to Jump loci, and 15% to other bodies; deep-space ships are visible only
when their real trajectories supply a contact solution. Generated traffic
never enters, exits, or passes through an unvisited frontier system. Plotting
or publishing a contact does not open it to background traffic. Initial
catalogue systems and BBS polity members are established visited systems; an
ordinary frontier system opens on the first player arrival or when it is an
actual stop on a later BBS founding contact route. Traffic volume follows
population, starport, connectivity, route demand, and vessels capable of the
route rather than a fixed encounter table.

At an arrival checkpoint with **N** eligible nearby traffic vessels, the
standard one-in-six candidate check is combined as:

**contact chance = 1 - (5/6)^N**

An encounter still requires sensor contact and a feasible relative-position
solution. Refreshing the arrival screen does not make another independent
traffic roll. Sparse traffic follows persistent route schedules rather than
appearing once per visitor.

For each local contact, sensors roll:

**2D6 + ship Electronics DM + target size DM - 2 per uncovered sensor hit**

Size DM is -1 below 100 tons, +0 from 100-999 tons, +1 from 1,000-4,999 tons,
and +2 at 5,000 tons or more. A total of 10+ identifies the vessel at
**80% + 5% per point above 10**, maximum 100%. A total of 6-9 gives an
approximate size class and tonnage at **45% + 10% per point above 6**,
maximum 75%. A lower total provides only its transponder at 25% confidence.
Approximate data does not reveal the hidden catalog design.

A cooperative vessel's identification transmission includes its declared ship
name, transponder, and registered class. Those are claims made by the contact,
not additional sensor resolution. The encounter view therefore presents them
separately from the sensor classification and confidence: an exact declared
class can accompany only a 45% approximate size-class return. A dark or
non-identifying contact has no declared class to display. Sensor classification
continues to expose only the generic size class, never the hidden catalog
design inferred from the declaration.

Each polity also has two public 0-100 orientations: **Trade-Combat** describes
commercial versus martial priorities, and **Chaos-Order** describes loose
versus controlled administration. Neither replaces the world's Law Level.
For traffic enforcement, local security is:

**(Chaos-Order + 10 x min(Law Level, 10)) / 2**

Strongly ordered systems more often challenge arrivals with customs or naval
pickets. Pirate pickets favor uncontrolled arrival and frontier-fuel loci and
retreat as traffic and enforcement make them untenable. A pirate compares the
observed target with its own capability and can abandon a plainly disastrous
intercept.

### Encounter posture and fallback

The arrival screen describes responses in terms of the actual contact. Routine
traffic exchanges identification and continues immediately. Traffic-control
orders may be followed or declined while maneuvering clear; an inspection may
be submitted to or refused; distress and derelict contacts may be assisted or
investigated instead of reported while continuing; debris may be avoided or
tracked while holding course; and a naval challenge may be answered or
refused. At a captain-held encounter, a physical inspection, assistance
attempt, sensor pass, or maneuver queues one kilosecond of authoritative
action time. The pending display reports that completion time; it is not a
wait for the other vessel's approval.

Every encounter also permits **Fight**, which immediately treats the displayed
contact as hostile and enters Vessel Combat. The immediate routine-identification
completion applies only when identification is exchanged, not when Fight is
chosen. Unless a locally accepted Naval Order or Privateer Commission names
that exact contact, fighting a contact that has not already attacked is an
unauthorized armed interception: it adds 10 public heat, files a 75%-evidence
warrant, and changes an Independent force-career record to Pirate.

Hostile encounters additionally use Flee, Meet Demand, Surrender, or Board
posture and may carry Surrender, Abandon, Jettison Cargo, and Break Off fallbacks.
Filed Flight Plan policy supplies responses at a Through checkpoint. The
default policy flees hostile contacts with Surrender fallback, complies with
inspections, reports distress, and does not divert to assist. Complying with an
Inspection applies the customs rule above. Meeting a hostile demand releases
the ship; Surrender immediately loses the command.

The displayed **Combat outlook** is the percentage used by conditional Fight
orders. It deliberately remains sensor-limited: Favorable, Comparable,
Dangerous, and Overwhelming assessments map to coarse estimates rather than an
omniscient combat simulation, and Unknown has no percentage and never satisfies
a threshold. It can change when the ship obtains better information and is not
a promise that combat will produce that result.

Other hostile postures resolve each kilosecond with:

**captain total = 2D6 + highest of Pilot (Spacecraft), Tactics (Naval), or
Gunner (Turrets) level + posture DM + intervention DM**

The posture DM is +1 for Fight or Board and +2 for Flee. The opponent rolls
2D6+2. Beginning on turn two in a Law-6-or-higher system, a 1-in-4 delayed law
intervention gives the captain +3 for that turn. The captain succeeds by
beating the opponent by at least 2: Flee escapes, Board secures the hostile as
a prize, and Fight disables it.

Otherwise the ship takes **1 + one hit per three full points by which the
opponent leads**, applied to Hull and then Structure, and another turn is
queued unless a fallback resolves it. Jettison Cargo drops half of every
cargo lot and ends the encounter. At the end of turn four, Surrender fallback
loses the command; without it, the hostile breaks away. Abandon and Break Off
can be ordered, but the current encounter revision gives them no outcome
distinct from that non-surrender fourth-turn breakaway.

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

An armed intercept is legally authorized only by a locally received warrant
associated with that vessel or by an accepted Naval Order or Privateer
Commission naming that exact traffic contact. An arrest purpose is unavailable
without the local warrant. A pirate lead or commission identifies prey and
terms but does not legalize the attack. Any other armed intercept or boarding
demand adds 10 public heat, files a 75%-evidence warrant, and changes an
Independent captain's force-career record to Pirate. A berthed or landed
background contact cannot be attacked directly; a player vessel can instead
be watched for departure at the shared locus.

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

Every conscious active crewmember assigned to at least one station has one
action budget for the turn. Where an action calls for a task, its target is 8.
The actor, characteristic, and skill assignments are:

| Action | Task |
| --- | --- |
| Coordinate; Increase Initiative | CHA/Leadership |
| Evasive Maneuvers; Line Up Shot; Break Pursuit | current DEX/Pilot (Spacecraft) |
| Close Range; Open Range; Prepare Jump | EDU/Astrogation |
| Sensor Targeting; Inspect Contact | EDU/Communications |
| Electronic Warfare | INT/Communications |
| Damage Control | EDU/Mechanic |
| Attack with turret or small mount | current DEX/Gunner (Turrets) |
| Attack with a bay | current DEX/Gunner (Capital) |
| Board | INT/Tactics (Military) assignment; the current boarding rounds do not apply this DM |

Close and Open Range move one band on success. Break Pursuit uses the same
open-range result. Inspect Contact creates the inspection evidence required by
an inspection order only on success. Board is legal only at Adjacent range.
Launch Escape Craft abandons the command; surrender completes only when the
opposing side accepts. **Electronic Warfare and Prepare Jump currently reserve
and record their station actions but apply no separate combat modifier or
Jump-progress benefit. Boarding begins normally, but its listed actor's
INT/Tactics DM likewise does not change the opposed boarding totals.**

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

Reaction tasks use current DEX: Dodge uses Pilot (Spacecraft), Point Defense
and Fire Sand use Gunner (Turrets), and damper or meson-screen triggers use
Gunner (Screens). In the current combat revision, only Point Defense has a
resolved trigger effect; the other four can be prioritized but do not yet
alter an attack. A reaction is spent only when its implemented trigger is
resolved.

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

Every terminal command loss atomically preserves a censored Command Loss
Report with the encounter. Its typed cause, captain fate, material and personnel
consequences, recovery date, and successor requirement govern recovery; outcome
prose is never parsed as rule state. The captain must acknowledge the current
report revision before recovery or succession. Reconnecting before that point
shows the report again. The detailed incident log is optional, and the report
is deleted with the terminal encounter only after recovery succeeds.

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
Effect -6 drives off the attackers. Any other result removes one Structure
from the defender, continues the boarding, and gives +2 on the next round to
the side that won that round. Destruction at zero Structure still applies
during boarding.

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

### Career status and naval service

The four recorded force careers are Independent, Navy, Privateer, and Pirate.
A change must be filed while docked against the current career revision. Navy
requires an institution-owned issued command and no locally enforceable
warrant. Privateer requires a privately registered, sponsor-owned, or financed
vessel, a non-independent polity with Law 4+, and no locally enforceable
warrant. Leaving Navy requires selecting a private vessel first. Entering
Pirate while commanding an institution ship changes its registry to stolen,
adds 50 public heat, and dispatches a mutiny-and-theft warrant.

Public heat is a nonnegative record of conspicuous unlawful attention;
underworld standing is a signed reputation record. They change only through
the explicit events stated in this chapter and currently supply no general
task DM of their own.

Naval captains command public property under orders. Service funds are
restricted, and rank or authority can be changed through service procedure.

Naval service points make a captain eligible, but promotion occurs only at a
board every 180 game days. Monthly salary follows the grade actually awarded,
not merely the eligible grade.

| Naval grade | Minimum points | Maximum issued command | Monthly salary |
| --- | ---: | ---: | ---: |
| Lieutenant | 0 | 400 tons | Cr6,000 |
| Lieutenant Commander | 10 | 1,000 tons | Cr8,000 |
| Commander | 25 | 3,000 tons | Cr10,000 |
| Captain | 45 | No limit | Cr12,000 |
| Commodore | 70 | No limit | Cr14,000 |

An order rated difficulty 0-2 awards 1 service point, 3-5 awards 3, 6-8
awards 6, and 9+ awards 10 after successful reporting.

### Orders, commissions, and evidence

Career opportunities refer to a traffic vessel already projected in the
system; accepting one never creates a target. An offer expires two game days
after that contact's scheduled traffic edge. Its objective is Patrol, Inspect,
Escort, Intercept, Capture, or Seize Cargo, and each requires its own evidence:
respectively a certified patrol log, signed sensor inspection, escort-release
receipt, engagement log showing the target driven off, capture papers with
custody, or seized inventory delivered to the issuing authority. Evidence for
one objective does not satisfy another.

The sealed operations report must physically reach the issuing authority
before success, points, or reward settles. An escort must start while the
protected contact is still at the named port locus and occupies the command
for three combat turns. A seize-cargo report requires the captured vessel and
some cargo to be physically present at the issuing authority.

Privateers act under a sponsor's commission and prize terms. The commission's
scope, expiry, targets, reporting duties, and exit terms bound its authority;
force outside them is not legalized by the ship's armament.

Privateer capture or cargo commissions pay a base Cr10,000 plus Cr50 per ton
of the named target. Pirate commissions use Cr25,000 plus Cr50 per ton; an
uncommissioned pirate lead has no base payment but uses the same Cr50-per-ton
scale. The signed instrument controls if its displayed terms differ.

Piracy can arise through free predation, accepted leads, or a pirate
commission. It uses the same physical interception, combat, boarding, cargo,
damage, prisoner, and evidence rules as every other career. A pirate result is
not made safe or profitable by selecting a career label.

A pirate cruise records a hunting system, end time, crew share, ship-fund
share, and prohibited targets. The two shares may total no more than 100% and
a cruise cannot begin at crew pressure 100 or more. Defaults are 50% to crew,
20% to the ship fund, with hospital, rescue, and surrendering vessels
prohibited. Each active month with a secured prize removes 20 pressure; a
month without prey adds 10. Expiry without prey adds another 15 and loses one
underworld standing. Fencing a pirate prize separately removes 25 pressure and
gains one standing.

### Prizes and warrants

A captured ship or cargo becomes a prize claim only when real custody and
required evidence exist. Adjudication, bounty settlement, impound, assessment,
appeal, and notice happen at their named offices and propagate through the
mail system. Paying an assessment or delivering a prisoner cannot instantly
erase enforcement at an office that has not received the signed instrument.

Prize realizable value is **gross catalog value x current condition percent**.
The deterministic settlement share is 10%, 20%, or 30% for a privateer and
5%, 10%, or 15% for Navy. A pirate fence pays 10%-30%, with lower Law able to
improve the result. A secured lawful prize may take an operating advance of at
most half its settlement; final payment deducts that advance. A privateer may
sell after adjudication or keep the vessel by taking a lien for realizable
value not covered by the awarded equity. A pirate may fence the vessel under
the cruise crew share or pay the gap between gross and realizable value to
launder its registry.

An unlawful armed interception files a warrant with severity
**1 + min(target value / Cr10,000,000, 9)**. Its bounty is **target value x
evidence percent / 1,000**, so perfect evidence produces 10% of target value.
The issuing system can act immediately; elsewhere the warrant is enforceable
only after its signed message arrives. A captain can satisfy a locally received
warrant by paying assessment and bond equal to half the bounty, minimum Cr1.
That creates a second resolution message. An office that has received the
warrant but not the resolution continues to enforce it. A hunter receives a
bounty only by holding the named person aboard the hunting ship and delivering
them while docked at an authority office.

## Ship and weapon reference

The [Ship Catalog](ships.html) gives each available vessel's hull, drives,
fuel, accommodation, equipment, weapons, ammunition, software, performance,
and price. Those listed statistics and fittings govern the vessel in play;
they are not sample configurations.

The weapon appendix below is the reference used in vessel combat. `--` means
that a weapon cannot attack at that range. Damage is expressed as dice plus a
fixed modifier. Traits identify rules such as beam eligibility, delayed missiles,
radiation, meson penetration, bay mounts, and physical ammunition.

| Weapon trait | Rule |
| --- | --- |
| Beam | Marks an energy attack as eligible for a Fire Sand reaction when that reaction gains a resolved effect |
| Missile | Launches now and rolls impact after the range-dependent delay; Point Defense can destroy it |
| Radiation | Adds one crew hit when damage penetrates armor |
| Meson | Ignores armor and starts on the internal hit column |
| Bay | Uses Gunner (Capital Weapons) rather than Gunner (Turrets) |
| Ammunition | Consumes one compatible physical round per weapon fired |

The appendix is generated from the same live catalogue used to resolve combat.
It is therefore the controlling source for weapon range DMs, damage, traits,
and ammunition names rather than an illustrative equipment list.

### Hidden information

Unrevealed contacts, hidden defects, undiscovered systems, unarrived messages,
and other hidden facts are not player knowledge. They become known only
through the observations, discoveries, messages, and other means described in
these rules.
