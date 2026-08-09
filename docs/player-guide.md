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
- `?` opens help for the screen or decision in front of you. Help pages like
  other long output; press Enter at the resume prompt to return to the same
  menu or input prompt.
- `<` and `>` move between pages where shown. Some packet screens also accept
  the corresponding arrow keys and display letter alternatives.
- A full page pauses at an `[Enter/Space] Continue  [C] Continuous` prompt.
  Press Enter or Space to continue one page, or `C` to suppress further page
  pauses until the next keyboard input. The prompt is erased and output resumes
  beneath it.
- A menu may omit an action when the present port, ship, or situation cannot
  support it.

Cepheus Trader is server-authoritative. If a command is rejected, read the
reason and refresh the relevant screen rather than assuming that the displayed
proposal took effect.

## Create your first command

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

## A good first session

Your new ship begins docked and ready to operate. The following route gives a
useful tour without requiring a particular career or speculative purchase.

1. Open `U`, then inspect **Crew Management** and **Ship Management**. Note
   duty coverage, fuel, cargo capacity, provisions, damage, and maintenance.
2. Return to the docked menu and open **Jobs and Passage**. Read local offers,
   but do not accept an obligation whose destination, deadline, or custody you
   do not understand.
3. Open the **Cargo Exchange**. Compare local stock with your carried market
   reports and available hold space. You do not have to buy cargo merely
   because it is offered.
4. Check **Fuel and Supplies**. Leave enough fuel for the proposed route and
   enough provisions for everyone aboard.
5. From `U`, open **Known Universe**. Inspect nearby systems and use the course
   plotter before committing the ship to a destination.
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

- `C` — **Cargo Exchange:** inspect the hold, buy finite local stock, sell
  cargo carried from another system, and research suppliers or buyers.
- `J` — **Jobs and Passage:** inspect and accept freight, passenger, mail,
  charter, courier, service, or other local offers.
- `F` — **Fuel and Supplies:** buy available fuel, provisions, and ammunition,
  or arrange a supported wilderness-fuel expedition.
- `Y` — **Shipyard:** inspect ships for sale and buy one by trading in the
  current vessel where the title and finances permit it.
- `P` — **Personnel:** hire available crew and open roster actions such as
  assignment, leave, recall, treatment, or discharge.
- `B` — **Banking and Accounts:** inspect debt, insurance, assistance, and
  other available financial actions.
- `A` — **Authorities:** handle local career, warrant, prize, traffic, and
  official business.
- `D` — **Depart:** construct, preview, and file a flight plan.
- `U` — open the universal command console.

Banking, personnel, authorities, refined fuel, repairs, and other services are
not guaranteed at every facility. A missing menu entry is useful information,
not a display error.

### Naval service accounts

A naval captain's liquid balance is personal money. The separately displayed
naval service account pays authorized vessel expenses before personal cash:
berthing and navigation charges, fuel, provisions, ammunition, necessary crew
medical treatment, repairs, replacement work, and refits. It cannot ordinarily
pay for cargo trading, collateral, private messages, personal insurance, fines,
or other private obligations.

The Banking and Accounts screen permits a naval captain to file a false
ship-expense receipt and move service credit into the personal balance. The
confirmation is deliberate: the receipt is retained for the next accounts
audit. Detection is not certain; its probability rises with the total amount
and the number of forged receipts in that accounting cycle, and with the share
of the month's ship expenses represented by false claims. A small false claim
can therefore hide more easily among substantial legitimate operating costs.
A detected forgery produces a naval warrant at the auditing office. That
finding reaches the captain and other authorities by mail; it is not
instantaneously known across systems.

### Cargo and local markets

Market inventory is shared, finite, and persistent. Quotes depend on the
captain, cargo, world, and current market state. Buying consumes credits and
hold capacity; selling removes the selected quantity from the ship. Splitting
or repeating a request does not create duplicate cargo or money.

Speculative cargo must be carried away from its origin before it can be sold.
Market reports are observations made at a particular place and time, not a
promise that stock or prices will be unchanged when you arrive.

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

After successful delivery, the task remains awaiting settlement while the
delivery filing travels back to the issuing office. Payment and release of the
posted collateral take effect when the remittance reaches you.

The Task ledger records accepted work. The Flight Plan controls where the ship
will actually travel; changing a route does not rewrite an obligation.

### Fuel, provisions, and port costs

Refined and unrefined fuel are distinct, and availability depends on the port
or selected collection method. Provisions are consumed by crew and awake
passengers; passengers travelling in low berths do not consume them while
frozen. Berth charges and other immediate obligations may be settled when the
ship departs. Keep a reserve instead of committing every credit to cargo.

## Universal command console

Press `U` from docked operations, or Enter from the voyage screen, to open six
managers that remain inspectable throughout normal play:

- `C` — **Crew Management**
- `S` — **Ship Management**
- `T` — **Task Management**
- `M` — **Message Management**
- `K` — **Known Universe**
- `O` — **Operations Ledger**

Enter refreshes the command console. `X` returns to the previous operational
screen. `Q` offers to return to the BBS and requires confirmation.

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

Battlefield patches, proper repairs, scheduled maintenance, refits, and
component replacement are different kinds of work. A temporary patch does not
make underlying damage or overdue maintenance disappear.

### Task Management

Task Management holds accepted obligations, local offers, and
automatic-carriage declarations. Available offers are shown by default; `V`
switches between them and the otherwise-hidden offers that the current ship or
crew cannot reasonably perform. Either view permits inspection and acceptance.
Inspect an offer before claiming it. Actions
such as cancellation, default, dispute, claim withdrawal, or custody return are
available only when supported by the task's current state.

### Message Management

Messages include news, public-service reports, offers, traffic notices, and
private correspondence. You can inspect, ignore, mark for later, action, or
archive delivered records. Classification changes how you organize a message;
it does not erase the underlying event or obligation.

Arrival-packet filters set the minimum importance shown for each service
class. Material filtered out of the arrival review remains available in the
message archive. Some messages carry direct links to the relevant Task,
Finance, Mapping, or Operations record.

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

When adding a charted leg, the destination list reports distance from that
leg's origin, primary-world port, population and tech codes, and the number of
charted gas giants. Its dossier adds chart age, source, and coordinates.

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

Player vessels appear in these same traffic pictures. `[PLAYER]` identifies a
player-owned ship operating under standing orders; `[ONLINE]` identifies the
player-owned ship currently under a connected captain's direct control. These
markers describe control, not the quality of the ship's sensor identification.

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
stops, or coordinates where those choices are available. Importing a plotted
course also adds its required port purchases or frontier-fuel operations to
the plan. A Flight Plan preview reports estimated time, fuel, and warnings
before the plan is filed. Keep enough operating cash available: a purchase
step that cannot be paid for or executed when reached holds the plan for the
captain's attention. The preview marks a deadline warning in red when the
planned route would deliver an accepted task late, or would run past its
deadline without reaching its destination.

Course knowledge and course execution are separate. A carried or purchased
plot can still be risky, and warnings about a known bad plot require a
deliberate choice. Filing a plan commits the route; revising it later does not
undo travel or obligations already completed.

The voyage screen shows the present stage, origin, destination, ship time,
next scheduled event, and fuel. You may revise the plan or enter the command
console while the voyage continues.

Travel can stop for a checkpoint, validation problem, traffic contact, or
encounter. An arrival packet may contain news, mail, market observations, and
offers accumulated in transit. Review or classify it, then take the arrival
watch when required. Uneventful authorized stages can complete without an
extra prompt.

## Encounters and combat

An encounter presents the actions legal in that situation, such as fighting,
running, complying, surrendering, or boarding. Vessel combat supplies a
conservative default order, an optional tactical controller, and detailed
joint-order editing for players who want direct control.

Standing combat policy matters when you are absent or do not submit an order
before the decision window closes. Review its objective and risk threshold
before relying on it. Disconnecting is not a way to freeze an encounter or
obtain a better combat controller.

Damage, ammunition use, injury, capture, surrender, and loss persist. A crew
that retains the ship attempts feasible emergency recovery, but supplies,
skills, time, and underlying damage still matter. If command is lost, the
recovery screen explains the available continuation or succession action.

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
