# Cepheus Trader player guide

Cepheus Trader is a persistent multiplayer space-trading game played through
participating BBSs. You command a captain, crew, and starship in the same
universe as players from other boards. Trade, mail, traffic, jobs, authorities,
and combat continue to develop while you are away.

The game is in field testing. Feedback, bug reports, and attaboys should be
filed as [GitHub issues](https://github.com/RealDeuce/ct/issues). Include the
BBS you called, the client version if known, and enough of the screen or error
text to identify what happened.

## Using the door

The choices shown at the bottom of each screen are authoritative for that
screen. Letter choices are case-insensitive.

- `Enter` normally accepts a displayed default, continues, returns to the
  previous menu, or refreshes the current screen.
- `Q` normally cancels the current action or backs out. At a command console
  or the docked menu it offers to return to the BBS; you must confirm with
  `Y` before the game exits.
- `?` opens help for the screen or decision in front of you. **Beginner** help
  explains why the screen is useful, every field, column, state, warning, and
  indicator it can show, why those details matter, every available action and
  consequence, why conditional actions may be absent, and a sensible first
  use. **Expert** help is a shorter, accurate operational reference. Each BBS
  player has a durable default. Help may span as many pages as it needs; it is
  not shortened to fit one terminal screen.
- During help paging, Enter or Space advances, `C` continues without more page
  pauses, `B` restarts the topic in Beginner mode, `X` restarts it in Expert
  mode, and `Q` stops help. The B/X choice lasts for that help visit only.
- At the end of a help topic, `H` opens the Help Browser. Its tree contains
  Getting Started, Menus and Screens, Concepts, and an alphabetical Glossary.
  `Q` moves toward the root and then returns to the game. The browser's `D`
  action changes the durable default.
- `<` and `>` move between pages where shown. Some packet screens also accept
  the corresponding arrow keys and display letter alternatives.
- A full page pauses at a contrasting `-- MORE -- Enter/Sp  C=All  Q=Menu` prompt.
  Press Enter or Space to continue one page, `C` to suppress further page
  pauses until the next keyboard input, or `Q` to skip the remaining ordinary
  output and reach the screen's menu. The prompt is erased and output resumes
  beneath it. Player Preferences can durably disable these automatic pauses;
  menus and confirmations still wait for input.
- A menu may omit an action when the present port, ship, or situation cannot
  support it.
- `B` in the universal menu opens **Browser Alerts**. When the BBS operator has
  enabled the service, the door displays a ten-minute link and, where it fits,
  a QR code with the complete fallback address immediately below it for the
  captain's portable communicator. The browser can warn you
  before the ship reaches a waypoint that will wait for orders, optionally
  warn before standing orders carry the ship through a waypoint, and call you
  when a checkpoint or detected encounter needs attention. Hold warnings,
  Through warnings, and immediate calls can be selected independently. Alerts
  successfully delivered to that browser remain in its temporary Received
  Transmissions list until they expire, so dismissing a notification does not
  discard its detail. Push alerts are a convenience, not a guarantee; game
  time and standing orders continue if one is delayed or missed.

Cepheus Trader is server-authoritative. If a command is rejected, read the
reason and refresh the relevant screen rather than assuming that the displayed
proposal took effect.

## Create your first command

The first time an unregistered local player reaches captain creation, the door
opens a short orientation. It introduces the setting and the shape of the
decisions ahead; it is not required reading. Press `Q` at any of its pager
prompts to skip the rest. The orientation is shown only once for that local
BBS identity.

New players create a captain and choose one of three starting commands. Every
screen begins with workable defaults, so you can safely accept them on a first
visit and learn the systems in play.

### Customize the captain

You may rename the captain, redistribute the characteristic point budget,
choose the offered skills, and select a training target. Characteristics must
use the complete displayed budget before you finish. A training target is a
long-term activity rather than an immediate skill increase.

The default captain is valid. Changing the captain is a choice, not a puzzle
with one required solution.

### Choose a starting offer

The three offers place you at different points between commerce and public
service:

- **Trader** starts with an independent commercial command. Ownership, debt,
  reserves, and commercial capacity are central concerns.
- **Privateer** starts with an armed charter. The command carries legal
  authority and prize rights, but also restrictions and obligations.
- **Navy** starts in public service with an issued command. Rank, orders, pay,
  and service obligations matter; commanding the vessel is not the same as
  owning it. The naval service account is restricted institutional credit,
  separate from the captain's personal funds.

Inspect each offer rather than comparing only ship size or weapons. The detail
screen explains the ship, starting package, reserves, authority, and exit
terms. A powerful or specialized ship may have less free cargo space, higher
costs, or less freedom of action.

Each offer also identifies its home BBS and polity. If that BBS belongs to a
League, the current League name follows the polity in parentheses.

### Name and fit the ship

After choosing an offer, name the ship and select any starting-fit choices.
The first option in each fit group is a usable default. Fit choices change
what the ship is prepared to do; they are not extra cash paid to the captain.

### Review the crew

The roster names officers, leaders, and senior specialists. Supporting crew
may be represented as part of a position rather than requiring an individual
name for every person aboard. You may inspect or rename the named crew and
select their training targets. Crew Management reports the effective headcount
against the ship's established complement. A grouped appointment shows its
current and established strength; combat casualties reduce that strength one
supporting position at a time, so a short-handed ship remains visibly short
until replacement personnel are assigned.

The final confirmation creates the captain, ship, crew, and starting estate
together. Nothing is registered until you answer `Y` on that screen.

After registration, the command library offers **Guided First Watch**. It is
an optional tour of your real crew, ship, accounts, messages, duty, charts,
and Flight Plan procedure in the shared universe. The route follows the
command: traders inspect commercial Tasks, privateers review their commission
before any optional eligible work, and naval captains follow issued orders
without being sent through cargo, passenger, or commercial-offer lessons. It
does not stop time, reserve an offer, accept a Task, award a bonus, or relax a
rule. Other captains and markets continue normally.

Enter begins the watch; `S` skips one briefing, `Q` returns to the Console
with progress available later, and `H` hides it. The Command Console's `W`
choice and Player Preferences can begin or resume it. Preferences can also
restart only the explanations without rewinding the captain or universe.
First Watch completes only when a real Flight Plan is successfully filed;
previewing and backing out does not launch the ship.

## A good first session

Your new ship begins docked and ready to operate. Guided First Watch can lead
you through the following route, or you can follow it yourself. The middle of
the route depends on the command's career and authority.

1. Open `U`, then inspect **Crew Management** and **Ship Management**. Note
   duty coverage, fuel, provisions, damage, endurance, and maintenance.
2. Inspect universal **Accounts**. Distinguish liquid cash, available cash,
   reserved funds, restricted operating credit, and principal; review any
   pending income by its estimated settlement date.
3. Follow the command's duty path:
   - Traders inspect **Jobs and Passage** and, when useful, the **Cargo
     Exchange**. Do not accept an obligation whose route or terms you do not
     understand, and do not buy cargo merely because it is offered.
   - Privateers inspect **Operations** and the commission first. Ordinary Tasks
     or cargo are supplementary choices only when the command is eligible.
   - Naval captains inspect issued orders in **Operations**. They need not use
     commercial cargo, passenger, or offer screens.
4. Read **Messages**, paying attention to origin and age, then check **Fuel and
   Supplies**. Leave enough fuel for the proposed route and
   enough provisions for everyone aboard.
5. From `U`, open **Known Universe**. Inspect nearby systems and use the course
   plotter to relate the destination to the chosen work or issued duty.
6. Choose **Depart**, build a flight plan, review its preview, and file it only
   when the destination, fuel use, and warnings make sense.

Once filed, a valid flight plan continues through its authorized stages even
if you return to the BBS. Re-entering the door reconnects you to the current
authoritative state; it does not rewind the voyage.

## Docked operations

The docked screen shows the ship, facility, system, port conditions, cash,
restricted ship credit, debt, berth account, fuel, and cargo use. Its services
are local: another port may have different stock, facilities, law, or
institutional offices.

### Technology levels and local capability

Tech Level, abbreviated **TL**, measures a world's scientific and production
capability and the complexity it can manufacture locally. It is separate from
population, wealth, law, starport class, and the capability of a particular
facility. Imported equipment may exceed local TL; conversely, a poor port,
closed yard, or missing parts can prevent work that the world's TL could
otherwise support.

| TL | Broad capability |
| --- | --- |
| 0 | No technological base; uninhabited worlds also have TL0. |
| 1–3 | Primitive technology through early mass production and steam power. |
| 4–6 | Industrial technology through radio, electrification, combustion, fission power, and increasingly capable computers. |
| 7–8 | Reliable orbital flight followed by practical travel within the star system. |
| 9 | Gravity technology and the first steps toward Jump. This is the lowest TL represented in the current Cepheus Trader vessel catalog. |
| 10–11 | Early interstellar technology and increasingly autonomous computers. |
| 12–14 | Mature interstellar technology, including advanced planetary engineering, armor, computers, and weapons. |
| 15+ | High stellar technology. |

The examples explain the scale rather than promising that every named
technology is implemented as a game system. For an actual purchase, repair,
or refit, compare the world's TL with its port and the displayed facility. A
world below TL9 can have a working port but no locally eligible ships for sale.

- `C` — **Cargo Exchange:** inspect the hold, buy finite local stock, sell
  cargo carried from another system, research suppliers, and locate buyers for
  player-owned speculative cargo aboard.
- `J` — **Jobs and Passage:** inspect and accept freight, passenger, mail,
  charter, courier, service, or other local offers.
- `F` — **Fuel and Supplies:** buy available fuel, provisions, and ammunition,
  or arrange a supported wilderness-fuel expedition.
- `Y` — **Shipyard:** inspect ships for sale, trade in the current vessel where
  title and finances permit it, or commission an admitted catalog design that
  the local yard can construct. A commission takes a deposit and appears in
  Fleet until its real construction time ends.
- `P` — **Personnel:** hire available crew and open roster actions such as
  assignment, leave, recall, treatment, or discharge.
- `B` — **Banking Services:** inspect the active vessel's finance terms and
  perform locally available insurance, arrears, or insolvency actions.
- `A` — **Authorities:** handle local career, warrant, prize, traffic, and
  official business.
- `D` — **Depart:** construct, preview, and file a flight plan.
- `U` — open the universal command console.

Banking, personnel, authorities, refined fuel, repairs, and other services are
not guaranteed at every facility. A missing menu entry is useful information,
not a display error.

### Accounts and pending settlement

`F` on the universal Command Console opens **Accounts** in every operational
phase. Liquid balance is the money currently posted to the captain's account;
available cash subtracts reservations already committed to tasks or escrow.
Restricted vessel credit pays authorized operating expenses, and secured
principal is debt rather than spending capacity.

Certified task income appears under Pending income until the issuing office's
remittance physically reaches the captain. It is grouped by estimated game
day; an amount marked as released is existing collateral expected to become
available again, not additional revenue. A missing estimate means the current
mail network offers no defensible delivery date. Do not budget pending income
as cash.

`T` opens the durable Transaction Journal. `F` cycles income, expense,
transfer, hold, financing, and opening entries; `V` filters by vessel; `N` and
`P` page through older and newer entries. Select a numbered entry to see each
affected account, exact increase or decrease, and resulting balance. An estate
that predates the journal begins with one carried-forward opening entry.

### Naval service accounts

A naval captain's liquid balance is personal money. The separately displayed
naval service account pays authorized vessel expenses before personal cash:
berthing and navigation charges, fuel, provisions, ammunition, necessary crew
medical treatment, repairs, replacement work, and refits. It cannot ordinarily
pay for cargo trading, collateral, private messages, personal insurance, fines,
or other private obligations.

Banking Services permits a naval captain to file a false
ship-expense receipt and move service credit into the personal balance. The
confirmation is deliberate: the receipt is retained for the next accounts
audit. Detection is not certain; its probability rises with the total amount
and the number of forged receipts in that accounting cycle, and with the share
of the month's ship expenses represented by false claims. A small false claim
can therefore hide more easily among substantial legitimate operating costs.
A detected forgery produces a naval warrant at the auditing office. That
finding reaches the captain and other authorities by mail; it is not
instantaneously known across systems.

### Sponsor-owned ships and arrears

A privateer may command a sponsor-owned ship without owing secured purchase
principal. The sponsor can still require monthly insurance. Restricted
operating credit pays that insurance before the captain's liquid balance, but
it cannot pay secured principal or private expenses.

If an installment defaults, Accounts shows the changed balance and the
financial notice remains available in Messages. At a port with Banking
Services, `P` posts the displayed overdue installment and withdraws its
impound order. The universal Accounts manager remains readable in Jump, but
the correspondent-bank action does not travel with the ship.

### Cargo and local markets

Market inventory is shared, finite, and persistent. Local offers show the
price to load stock from the exchange. Cargo aboard shows the current local
bid for each player-owned lot; that is the price an ordinary sale will use.
Buying consumes credits and hold capacity, while selling removes the selected
quantity from the ship. The ordinary bid remains below the local ask, but a
reserved private buyer may offer more. Splitting or repeating a request does
not create duplicate cargo or money.

Each listed price has a compact range plot. Its minimum, Q1, median, Q3, and
maximum are universe-wide market-value landmarks for that commodity, not a
forecast tailored to your captain. The current-price marker is your actual
negotiated quote, which can change with the captain's Broker skill and Charisma
as well as local trade codes, events, and tariffs. Low purchase prices are
favorable; high sale prices are favorable. The green/yellow/red judgment is
also written as `low-price`, `mid-price`, or `high-price` so it remains clear
without color.

Speculative cargo may be sold in its origin system, normally at an immediate
loss. Market reports are observations made at a particular place and time,
not a promise that stock or prices will be unchanged when you arrive.

### Jobs, custody, and deadlines

Claiming an offer at its issuing office produces an immediate award when the
offer is still available. Claiming a copy anywhere else files a sealed claim:
capacity and collateral are reserved while it travels, but the claim is not an
award until the issuing office's confirmation reaches you. If another claim
arrives first, the decline releases those reserves. A captain travelling to the
issuing office carries their own signed filing, so the ship cannot arrive there
before its claim does even when it declines ordinary mailbags.

An award closes the offer at its origin, but remote copies remain visible until
the closure notice reaches them. Cargo or passenger custody transfers only
after the award is known and the performing ship is at the origin. Once aboard,
freight, passengers, and mail remain assigned to that ship; changing command or
owning another vessel does not teleport them. Review the task's destination,
deadline, payment, failure terms, and required capacity before filing a claim.
The local-offer list shows the pickup slack remaining after the fastest
executable course to the issuing system. Green means more than six hours,
yellow means 30 minutes through six hours, and red means less than 30 minutes,
already late, or no executable course.

For a remote pickup, availability is based on one continuous projected voyage:
current system to pickup and then pickup to delivery. Fuel remaining after the
first leg carries into the second, the pickup must occur before the offer
closes, and final arrival is compared with the delivery deadline. The task
ledger receives this assessment with the offer so its classification cannot
drift while the client separately plots each endpoint.

The ledger initially shows only offers the commanded ship and its present crew
can reasonably perform. It reports how many unavailable offers are hidden;
press `V` to view those offers and the reasons they are unavailable. The check
includes hold or passenger capacity, required steward or medical staffing,
whether posting the collateral would leave enough cash to clear the current
berth, an executable course to pickup, and deadlines that can already be proven
impossible. The unavailable classification is advisory: you may still inspect
and claim an offer from that view, accepting the risk that you must correct the
listed problems before performance. The classification is refreshed with the
ledger, so selling cargo, hiring crew, or otherwise correcting a stated problem
can make an offer appear in the normal list.

The issuing market screens destinations against traffic vessels that could
actually perform the route. Work needing extended tankage or other specialist
Jump capability is progressively less common and carries a progressively
higher payment than ordinary refuel-each-stop J-2 work.

After successful delivery, the task remains awaiting settlement while the
delivery filing travels back to the issuing office. Payment and release of the
posted collateral take effect when the remittance reaches you.

The Task ledger records accepted work. The Flight Plan controls where the ship
will actually travel; changing a route does not rewrite an obligation.

### Fuel, provisions, and port costs

Refined and unrefined fuel are distinct, and availability depends on the port
or selected collection method. The source list identifies planets, moons, gas
giants, and icy belts; ordinary wilderness entries are unoccupied routine-
access sources, and unavailable entries are not numbered. Remaining tank room
appears once above the choices. Each frontier source gives its own round-trip
distance and travel time; those values, rather than the body type alone,
distinguish the operational cost of the detour.

A ship with scoops may collect without a processor. When a processor is fitted,
Flight Plan defaults to refining the selected batch, but you may keep it
unrefined. Preview shows travel, collection, processing, normal total, and the
longer failed-check total. Refining is Engineer (Power)/EDU 8+; failure doubles
processing time and exceptional failure can damage the Jump drive. Purchased or
collected unrefined fuel may be refined later while docked or safely holding.
An unrefined purchase already embedded in a Flight Plan defaults to processing
that batch before the next step when the ship has a processor.

Mixed tanks are consumed proportionally. Ordinary power use has no separate
unrefined-fuel roll, but a Jump that actually burns any unrefined fuel takes the
normal -2 success DM. Preview warns on that Jump; if a prior refining step
removes all unrefined fuel, the warning is omitted.

Away from a berth, provisions are consumed by
crew and awake passengers; passengers travelling in low berths do not consume
them while frozen. Docked ordinary crew arrange their own food. The captain
uses one ship person-day or, when stores are empty, automatically buys a meal
from liquid credits at twice the package-average daily price. Three days
without food cause discomfort but no damage; later days bring increasingly
difficult Endurance checks, and failed checks cause starvation damage that
cannot heal until the character eats. Berth charges and other immediate obligations may be settled when the
ship departs. After a port fuel purchase, the fueling receipt lists the amount
loaded, tank state, total charge, the restricted and liquid amounts used, and
both remaining balances. A chandlery receipt likewise lists the monthly packages
and person-days loaded, resulting life-support stores, charge, payment split, and
remaining balances. Keep a reserve instead of committing every credit to cargo.

## Universal command console

Press `U` from docked operations, or Enter from the voyage screen, to open the
universal command console. Its eight managers are grouped into four areas:

- `1` — **Vessel and Crew:** `C` Crew Management and `S` Ship Management.
- `2` — **Duty and Accounts:** `T` Task Management, `O` Operations Ledger,
  and `F` Accounts.
- `3` — **Information and Communications:** `K` Known Universe, `M` Message
  Management, and `R` System Common Radio.
- `4` — **Captain and Interface:** browser alerts, help, preferences, Guided
  First Watch, license notices, and the isolated abandonment action.

The established manager letters also work as direct shortcuts from the console
root. Enter refreshes the current console screen. `Q` returns from an area to
the root; at the root it offers to return to the BBS and requires confirmation.
`X` returns directly to the previous operational screen.

The console identifies the captain's home BBS and polity. A parenthesized name
after the polity is the BBS's current League affiliation.

`A` permanently abandons the current captain and starts over. It discards every
ship and crew member, cargo and stores, cash and financing, tasks and contracts,
career and service history, prizes and warrants, private messages, and personal
knowledge. Nothing carries over. The door requires the exact phrase `ABANDON
EVERYTHING` and a separate final confirmation, then returns the same BBS account
to new-captain registration. The command-recovery screen also offers this action
when a command has been lost. Abandonment is temporarily unavailable while an
encounter or shared combat is being resolved.

### Crew Management

Crew Management shows named personnel, service positions, watches, injuries,
fatigue, morale, pay, and training. You can inspect it anywhere, but hiring,
discharge, shore leave, medical care, and some reassignments require a suitable
place and situation.

An officer's permanent appointment and current watch duties are different.
Removing someone from watch does not erase their employment or automatically
transfer command. Vacancies, doubled duties, and unavailable crew can reduce
what the ship can safely accomplish.

### Ship Management

Ship Management shows vessels under the captain's control and the selected
ship's hull, drives, systems, weapons, stores, damage, and service state. It
also provides transfers and command changes where legal.

Routine upkeep is continuous work by the crew. Its cost is charged
automatically every 30 game days to the operating account, using restricted
operating credit before liquid credits, so there is no monthly yard order to
place. If the account cannot cover the full charge, the shortfall becomes
arrears and the missed cycle can damage a subsystem.

Battlefield patches, proper repairs, routine upkeep, refits, and component
replacement are different kinds of work. A temporary patch does not remove
underlying damage. Ship Management shows an active operation's completion in
game time and the corresponding real-world wait. Proper repair remains tied
to the berth: Flight Plan preview warns about a departure, and actually
clearing the berth cancels unfinished repair without removing the damage.
Refits and other yard work still prevent departure.

Choosing **Begin refit** opens a quotation before any money is spent. It shows
the operating-account charge, expected yard time, damage the yard will repair,
and any destroyed installations that will remain destroyed. Authorizing the
quotation begins the yard stay. Completion repairs non-destroyed damage,
removes temporary battlefield patches, and may correct minor faults found in
the overhaul. It does not replace a destroyed installation or reset an
installation's age or use. Routine upkeep and its automatic charges continue
during the refit.

### Task Management

Task Management holds accepted obligations, local offers, and
automatic-carriage declarations. Available offers are shown by default; `V`
switches between them and the otherwise-hidden offers that the current ship or
crew cannot reasonably perform. Either view permits inspection and acceptance.
Inspect an offer before claiming it. Actions
such as cancellation, default, dispute, claim withdrawal, or custody return are
available only when supported by the task's current state.

### Message Management

Messages include meaningful news, public-service reports, traffic notices,
operational and financial notices, and private correspondence. Commercial
offers appear only in Task Management, alongside their current availability
and complete terms. You can inspect, ignore, mark for later, action, or archive
delivered correspondence. Classification changes how you organize a message;
it does not erase the underlying event or obligation.

Communication Filters set the minimum importance shown for News, Public
service, Traffic, and Private copy. The same thresholds apply to the Message
Management list and arrival review. Raising a threshold hides lower-priority
retained copy; lowering it reveals that copy again. Some messages carry direct
links to the relevant Task, Finance, Mapping, or Operations record.

Information and private correspondence travel through the universe. A report
may be old when it reaches you, and a message cannot reveal facts that have not
yet arrived at the ship.

### System Common radio

System Common is a single public radio channel for player broadcasts within
the current system. Open it with **R** from Universal Managers. Radio waves
travel at light speed: a ship at the port, gas giant, or Jump locus may receive
a transmission later than a ship near its source. The channel is unavailable
in Jump space and depends on functioning ship communications.

The inbox shows unread reception metadata. Opening a reception displays its
body once and then removes the receiving ship's copy, so save anything you
want to retain outside the game. Unread receptions expire after 196 game days
and stay with the receiving ship if that vessel changes hands. Player
broadcasts may contain up to 500 printable ASCII characters and are limited to
one every 15 real seconds. You may persistently mute another captain's normal
broadcasts.

Customs inspections, naval boarding instructions, and hostile surrender
demands also arrive as structured System Common hails. They cannot be muted.
Nearby ships may overhear them, but an encounter action belongs only to the
ship being hailed. The door reports newly arrived unread radio at safe input
boundaries instead of interrupting another screen with message bodies.

### Known Universe

Known Universe is the current ship's carried navigation and intelligence
library. It contains known systems, source and age information, market
observations, course plots, and mapping-disclosure choices.

Knowledge may be incomplete or stale. It is carried with its repository and
does not become omniscient merely because the server knows something. The
course plotter can compare fastest and cheapest known routes, including fuel
carried aboard, purchases of refined or unrefined fuel at charted ports, and
frontier skimming where the ship and charts support it. The plot reports its
modeled fuel-purchase cost and travel time. The captain still chooses and
imports a course into the executable Flight Plan.

Flight Plan can also request a route through every active accepted-task stop
assigned to the commanded ship. The server uses a bounded beam-search heuristic
rather than enumerating all stop permutations: pickups remain before their
deliveries, deadlines influence the order, shared stops are consolidated when
useful, and a system may be revisited when an urgent delivery requires it. This
returns quickly but is not a proof of the optimal route. Preview the imported
plan to see authoritative deadline warnings before filing it.

When adding a charted leg, the destination list reports distance from that
leg's origin, primary-world port, population and tech codes, and the number of
charted gas giants. Its dossier adds chart age, source, and coordinates. A
publicly mapped system also identifies its home BBS and current polity and
League affiliation when known. Private, withheld, secret, and merely
dispatched mapping states do not disclose that affiliation.

When the ship reaches an unmapped system, you may publish it, file it directly,
withhold it, or mark it secret. Read the prompt: disclosure choices have
different privacy and cost consequences.

### Operations Ledger

Operations Ledger covers naval rank and orders, privateer commissions and
prize claims, pirate opportunities and cruises, warrants, system traffic,
local contacts, and combat-career standing. Available actions depend on local
authority, delivered information, the ship, and the captain's current service.

The **Local contacts** section is a sensor picture of the ship's current traffic
locus. A port, Jump locus, gas giant, or other frontier-fuel body can contain
contacts; ordinary interplanetary transit and Jump space normally do not. A
civilian contact's name, operator, role, and transponder come from its broadcast
registry. Class and tonnage are separate sensor estimates, and may be
approximate or unresolved when the ship has modest or damaged electronics.
Only contacts still sharing the locus can be intercepted.

`[BERTHED]` and `[LANDED]` mean that a vessel shares the traffic locus but is
still attached to a facility or surface. Selecting one does not start combat
inside the berth or on the ground. Your ship clears its own berth, pays the
charges then due, and waits at the locus; the intercept occurs only if the
selected vessel departs. Gas-giant skimming is spaceborne and remains exposed
to immediate interception, while a wilderness water/ice expedition is landed
until it lifts.

**Standing interception order** places the ship on picket at its current port,
Jump point, gas giant, or other modeled body locus. It can target all modeled
craft or one catalogued craft class observed in the current traffic picture.
It also specifies whether matching traffic is attacked or ordered to submit to
boarding and inspection. An armed attack starts combat at once. A boarding
order starts combat only if the target refuses; offline vessels apply their
stored inspection response and then their normal tactical automation. An
unauthorized boarding demand is still a crime even if the target submits.
Matching background movements and player arrivals or departures trigger the
chosen intercept. The Operations Ledger shows the active watch and can remove it. A
named departure watch ends when that vessel departs; an all-craft or class
watch continues after an engagement while the ship remains capable and at the
same locus. Removing a port watch returns the ship to a berth with a new
arrival time.

Customs and naval pickets are more common at incoming Jump loci in strongly
enforced systems. Refusing a lawful inspection causes the local picket to call
for help and try to withdraw rather than duel for honor; capable enforcement
traffic may join on its real movement time, and the warrant can meet the ship
again at its next controlled destination. Pirate pickets favor Jump-arrival
loci in chaotic, lightly travelled systems, retreat toward gas giants as
security grows, and avoid systems or targets that are too dangerous.

Player vessels appear in these same traffic pictures. `[PLAYER]` identifies a
player-owned ship operating under standing orders; `[ONLINE]` identifies the
player-owned ship currently under a connected captain's direct control. These
markers describe control, not the quality of the ship's sensor identification.

Warrants arrive through the store-and-forward electronic mail network and are entered into the Operations
Ledger automatically; there is no bounty to accept. A `[WARRANT]` marker means
that a locally received warrant names a person associated with that vessel.
The association is dated evidence, not proof that the person is still aboard.
Select **Arrest named subject** to issue a lawful surrender demand. A vessel
may surrender the person, deny the person is aboard, or refuse boarding. A
denial permits a search: Investigate, Recon, and Streetwise skill and the size
of the boarding party oppose the subject's concealment. A failed search leaves
the warrant active. A successful search places the person in custody aboard
the hunter's ship. Deliver a held prisoner through **Warrant court** at a
docked port with an authority office to collect the stated bounty.

Generated traffic also carries persistent generated people with warrants, so
bounty hunting does not depend on finding another player. Those targets move
through the same systems and appear in the same traffic pictures as other
ships. A warrant's last report does not track them magically; current presence
is known only when their transponder or a local sensor contact is actually
observed. When a player ship answers an arrest demand offline, its saved
inspection and surrender policies govern the response. At a port, a delivered
warrant may cause an automatic enforcement boarding. Whether a search occurs
and how thoroughly it is conducted scales with local law level.
For an offline player vessel, a lawful demand for someone not actually aboard
uses the saved inspection-compliance setting. If the person is aboard, the
saved surrender permission and minimum acceptable victory estimate decide
whether the crew turns the subject over or resists. Ordinary combat automation
then governs any refusal.

A local player vessel can be intercepted under the same physical and legal
rules as other traffic. If its captain is actively commanding it, both
captains receive the shared combat turn and may seal orders before the window
closes. A disconnected captain, a missed deadline, or a vessel commanded
elsewhere invokes that ship's stored combat policy and actual crew. The server
does not pause combat for a disconnected player.

PvP acts on the real vessel rather than a disposable encounter copy. Damage,
ammunition, casualties, cargo loss, surrender, boarding, destruction, warrants,
and prize custody persist for both sides. Capturing a ship transfers the ship
and everything physically aboard it; surviving personnel remain real people
and enter parole or recovery rather than becoming captor-owned crew.

This is distinct from the system traffic-control picture. Traffic control can
report transponder movements elsewhere in the system, showing that the system
is active without making those vessels local sensor contacts or viable
interception targets.

Changing service, intercepting traffic, mutiny, piracy, and similar actions
can be irreversible or unlawful. The door supplies an additional confirmation
for the most serious choices; treat that warning literally.

## Flight plans and travel

The Flight Plan is a sequence of destinations and authorized actions. It can
use carried charts, imported plotted courses, task destinations, frontier-fuel
stops, coordinates, or the bounded all-assigned-tasks suggestion where those
choices are available. Importing a plotted
course also adds its required port purchases or frontier-fuel operations to
the plan. A Flight Plan preview reports estimated time, fuel, and warnings
before the plan is filed. Each warning appears once as a numbered footnote,
with its number repeated beside every affected step. A provision-shortage
warning is repeated beside every step where cumulative projected consumption
has exceeded the person-days aboard. An active proper-repair warning marks the
first departure step that will cancel the work and leave the damage in place. Keep
enough operating cash available: a purchase step that cannot be paid for or
executed when reached holds the plan for the
captain's attention. The preview marks a deadline warning in red when the
planned route would dock after an accepted task's deadline, would run past its
deadline without reaching its destination, or reaches a Hold checkpoint in
time but still requires arrival watch before docking. Checkpoint readiness is
not delivery.

Checkpoint authority and plan completion are separate. **Through** permits the
crew to use standing orders and continue while you are away; **Hold** waits for
arrival watch. One terminal marker is always on the last step and ends the plan
after the selected Hold or Through behavior completes. Generated task and
ordinary routes initially use Through throughout. Player Preferences can make
either type generate Hold checkpoints instead without changing a filed plan.

The editor groups related physical steps into logical route items. Select any
future item to change its authority or settings, delete or reorder it, insert a
charted leg beside it, or replace its charted destination. Completed items stay
visible and locked. Task Management's **Standing orders** edits the active
ship's default encounter policy; a new plan starts with that default, while a
filed plan keeps its own copy. The Flight Plan can load the ship default, keep
different orders for that voyage, or save its edited orders as the new default.
Each encounter type has an ordinary response plus Never, Always, or a minimum
sensor-estimated combat-outlook percentage for Fight. Unknown outlook never
passes a percentage threshold, and automatic attack against non-hostile traffic
requires an explicit risk acknowledgement.

Course knowledge and course execution are separate. A carried or purchased
plot can still be risky, and warnings about a known bad plot require a
deliberate choice. Filing a plan commits the route; revising it later does not
undo travel or obligations already completed. While the ship remains in normal
space, revising the first unprocessed waypoint calculates a new maneuver from
the ship's current motion. You may turn back to the primary port, choose a new
Jump destination, or replace the route with a belt cycle or lawful frontier
fuel stop. The displayed estimate includes the time needed to cancel the old
relative motion and intercept the moving destination. A Jump already in
progress cannot be redirected before breakout, but later route items and
standing orders remain editable. Future-only changes preserve the active leg or
shipboard operation and its scheduled completion rather than restarting it.

The voyage screen shows the present stage, origin, destination, ship time,
next scheduled event, and fuel aboard compared with effective total tank
capacity. You may revise the plan or enter the command console while the voyage
continues.

Travel can stop for a checkpoint, validation problem, traffic contact, or
encounter. An arrival packet may contain news, mail, market observations, and
offers accumulated in transit. Review or classify it, then take the arrival
watch when required. Uneventful authorized stages can complete without an
extra prompt.

If a discordant Jump transition, exceptional fuel-processing failure, or
unpaid-upkeep check damages a subsystem, an **Engineering Casualty Report**
interrupts the next safe prompt or appears before ordinary command work on the
next sign-on. It identifies the operation, ship, subsystem, new and resulting
hits, operational effect, and any inaccurate-Jump approach delay. Reports are
shown oldest first and remain until Enter positively acknowledges them. Q
leaves the current report pending. Acknowledgement records that the captain saw
the consequence; it neither repairs the ship nor stops the game clock or a
still-valid plan. Combat consequences remain consolidated in their encounter,
combat, and Command Loss Report screens rather than producing a report for
every hit.

## Encounters and combat

An encounter presents actions named for that situation. Routine traffic uses
Enter to exchange identification and continue immediately. Traffic-control,
inspection, distress, derelict, hazard, and military contacts instead describe
the specific order, assistance, investigation, maneuver, or refusal being
chosen. Physical work may take authoritative time; the pending screen shows
when it completes, and does not mean the other vessel is deciding whether to
approve your response. Hostile contacts retain fighting, running, meeting a
demand, surrendering, and boarding, while a departing contact can be pursued
or allowed to go. Every displayed contact can instead be fought immediately.
Doing so against a contact that has not attacked is an armed interception;
without an accepted Naval Order or Privateer Commission naming that exact
contact, it creates public heat and a propagating warrant and changes an
Independent force-career record to Pirate. When a vessel identifies itself,
its **Declared identity**, **Transponder**, and **Declared class** are transmitted
claims. They are displayed separately from **Sensor resolution** and **Sensor
classification**, so an exact registered-class claim may appear beside a 45%
approximate size-class estimate. A dark or non-identifying contact instead has
no declared class. The contact screen also shows the apparent authority, range,
confidence, and a coarse threat assessment; it does not silently substitute a
claimed exact hull class for an uncertain sensor return.

Pirate demands state the owned cargo and entrusted freight actually exposed.
The demand does not imply that pirates already possess your manifest: it states
what their boarding search will take, while the displayed exposure comes from
your own cargo ledger. They demand all entrusted freight and a stated,
milliton-rounded share of each owned lot, but cannot take more than fits in
their own cargo hold.
They pack the most valuable cargo per ton first; an indivisible unique object is
skipped if it cannot fit. A documented loss of entrusted freight can be filed
from the Task ledger. The authenticated encounter, custody record, response,
and assessed threat affect adjudication; only cargo physically taken is claimed.

Vessel combat supplies a
conservative default order, an optional tactical controller, and detailed
joint-order editing for players who want direct control.

Running from an intercept and chasing a departing contact use the same pursuit
rules in opposite directions. At Short or Close range and matched speed, the
pursuer uses a significant Pilot action to establish pursuit. Maintaining it
uses the same action and improves attacks after the first maintained turn, to a
maximum +4. The target can oppose with Break Pursuit. Opening to Medium range,
becoming seven speed points faster, leaving the fight, or winning the opposed
break ends pursuit. Speed may change up or down by no more than currently
effective maneuver thrust each combat turn.

Standing combat policy matters when you are absent or do not submit an order
before the decision window closes. Review its objective and risk threshold
before relying on it. Disconnecting is not a way to freeze an encounter or
obtain a better combat controller.

Damage, ammunition use, injury, capture, surrender, and loss persist. A crew
that retains the ship attempts feasible emergency recovery, but supplies,
skills, time, and underlying damage still matter. If command is lost, the next
login opens a mandatory **Command Loss Report** before recovery or succession.
It identifies the censored contact evidence, whether a Through plan supplied
the response, the posture and fallbacks used, tactical-controller involvement,
the final ship and captain disposition, crew casualties, and cargo, fuel,
passenger, and damage consequences. `L` opens the optional chronological
incident log; Enter acknowledges the summary. Recovery remains locked until
the current report is acknowledged. After acknowledgement, `V` reviews it
again until recovery or succession succeeds, at which point the report is
removed.

## Practical advice

- Read the complete prompt before pressing Enter; its meaning is contextual.
- Keep cash available for fuel, provisions, fees, repairs, and failure costs.
- Check cargo space and physical custody before accepting another task.
- Check the ship's actual fuel and provisions, not only whether a route exists.
- Treat charts, prices, reports, warrants, and news as dated information.
- Use the universal managers before and after a voyage to catch changed crew,
  ship, task, message, radio, and career state.
- Conservative defaults are valid, but they cannot make an impossible route,
  neglected ship, or unsuitable obligation safe.
- Returning to the BBS leaves the persistent universe running. File only the
  continuation plan and standing policies you actually want followed.
