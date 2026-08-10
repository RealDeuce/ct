# Pirate Gameplay

*Status: Milestone 6 operational loop implemented, 2026-08-02*

Pirates are autonomous. They do not receive a navy-style stream of mandatory
orders, but completely unstructured predation is not enough to make targets
discoverable or produce a useful 15–30 minute play session. Pirate gameplay
therefore combines unrestricted attacks with optional intelligence,
commissions, and a crew-defined cruise.

## Source audit

*Skull and Crossbones: Piracy in Clement Sector, Third Edition* does not
provide a formal pirate mission generator or rank progression comparable to a
navy. Its useful mechanical structure is instead:

- four distinct operations: cargo raiding, ship theft, commerce raiding, and
  marauding against a base, station, or settlement;
- forced docking, hull breaching, computer intrusion, planted crew or
  passengers, cargo negotiation and jettison, boarding, and prize crews;
- sympathetic havens, fences, repairs, recruiting, intelligence, and rules of
  local conduct;
- articles of agreement, crew shares, a ship fund, and no-prey/no-pay
  participation incentives;
- fencing that realizes only a fraction of stolen cargo or hull value;
- paid or deniable attacks by governments, corporations, and other patrons;
- letters of marque and prize courts that turn some of the same actions into
  lawful privateering; and
- adventure premises involving a valuable shipment, ransom, paid commerce
  raiding, pirate-community enforcement, false-flag work, and treasure.

The relevant source sections are Pirate Strategy and Tactics on pages 28–31,
Pirate Havens on pages 32–34, Pirate Life on pages 43–49, Anti-Piracy Efforts
on pages 72–73, Adventure Seeds on pages 81–82, and Random Encounter Tables on
pages 83–86.

The book's encounter tables distinguish ports, gas giants, and Jump loci,
which supports Cepheus Trader's encounter geography. Their rolled ship counts
are not imported: this game derives actual traffic from population, TL, trade,
routes, schedules, and current location as specified in
[`system-traffic-and-encounters.md`](system-traffic-and-encounters.md).

The source also contains named setting material and extreme criminal conduct.
Named people, organizations, places, ships, plots, and prose remain excluded
Product Identity. The source's inclusion of a subject is not itself a decision
to put that subject into Cepheus Trader; only explicitly adopted mechanics and
original game content belong here.

## Three sources of pirate action

### Free predation

A pirate may attack any ship, cargo movement, station, facility, or settlement
that the player can actually locate and intercept. No lead or commission is
required. The ordinary combat, evidence, witness, damage, legal, news, warrant,
bounty, fence, and political systems determine the outcome.

Free predation must not create a fresh target or generic random victim. It acts
on the same persistent or statistically materialized traffic that every other
role sees. Repeatedly attacking one route can reduce traffic, change security,
raise insurance and freight prices, attract patrols and hunters, close willing
ports, or destroy the route the pirate depended upon.

### Pirate leads

A lead is information about a possible existing opportunity. Sources include
underworld contacts, corrupt officials, brokers, passenger or crew moles,
stolen manifests, intercepted communications, traffic observation, haven
gossip, criminal buyers, and compromised schedules.

Examples include:

- a valuable cargo manifest and expected departure;
- a lightly escorted freighter or damaged ship;
- a vessel expected to skim at a particular gas giant;
- a predictable convoy or mail departure;
- a buyer seeking a particular stolen commodity or ship;
- a patrol gap or useful false identity; and
- a vulnerable depot, beacon, refinery, or frontier facility.

A lead references an existing `TrafficCall`, ship, cargo lot, passenger,
facility, scheduled event, or other authoritative target. It never spawns a
victim merely because the player reads or accepts it. A lead has a source,
observation and receipt time, confidence, known facts, price or obligation,
expiry or expected window, and possible competition, misinformation, or trap
risk. Mail delay can make it stale, and another actor may reach the target
first.

Players can also create their own leads through scanning, surveillance,
following a ship, analyzing schedules, bribery, interrogation, or other
information-gathering actions. Leads surface opportunities; they do not grant
exclusive rights or guarantee success.

### Pirate commissions

A commission is an optional bargain with a patron who wants an outcome in
addition to whatever prize the pirates can realize. Possible objectives
include:

- steal specified cargo or capture a particular vessel;
- destroy, delay, divert, or intimidate selected traffic;
- obtain or recover a passenger;
- raid or sabotage a depot, beacon, refinery, base, or settlement;
- protect a haven, fence, smuggler, or pirate convoy;
- rescue captured crew or recover an impounded ship;
- act against a pirate group sanctioned by a haven or criminal community; and
- support a rebellion, corporate conflict, or deniable false-flag operation.

The commission identifies its patron, target or target constraints, desired
result, deadline, evidence of performance, advertised compensation, payment
terms, and consequences for failure or betrayal. The patron may be unreliable,
unable to pay, attempting a sting, or planning to deny the relationship.
Pirate commissions consequently do not carry the authority or payment
certainty of naval orders.

## The pirate cruise

The pirate analogue to a naval deployment is a **cruise** defined by the
captain and crew before leaving a haven or other safe anchorage. Its articles
record:

- the intended hunting region, duration, or return condition;
- target classes and any prohibited targets or conduct;
- accepted commissions and particularly important leads;
- division of prizes among captain, officers, specialists, boarders, and
  other crew;
- the share reserved for fuel, repair, ammunition, losses, and the ship fund;
- participation requirements and no-prey/no-pay treatment; and
- any crew approval, amendment, discipline, or early-termination procedure.

The captain remains in command, but ignoring the agreed cruise, concealing
prizes, taking unacceptable risks, producing no prey, or refusing shares can
affect crew loyalty, performance, desertion, and mutiny risk. Exact crew
politics remain to be designed; the articles must not become repetitive
role-playing chores.

## Operational loop

The pirate loop is:

```text
obtain intelligence or a commission
→ choose a hunting region and cruise terms
→ locate and intercept real traffic
→ intimidate, disable, board, capture, strip, or destroy
→ escape or manage the response
→ reach a suitable haven, buyer, fence, or patron
→ realize prizes and divide proceeds
→ absorb heat, reputation, crew, and political consequences
```

The attack itself may produce no prize. A target can escape, prove worthless,
be too damaged to move, carry evidence or tracking hardware, or attract a
response that makes recovery impossible. A successful capture still requires
crew, fuel, repairs, registry or identity work, and a buyer capable of
settling it.

## Progression and standing

Pirates have no universal rank ladder. Their persistent progression comes
from:

- realized wealth, ships, cargo, equipment, and independent assets;
- crew competence, loyalty, cohesion, and willingness to follow ambitious
  cruises;
- underworld reputation distinct from public notoriety and legal heat;
- trusted contacts, better intelligence, and access to higher-value leads;
- haven access, fence capacity, repair support, corrupt protection, and
  favorable settlement terms; and
- eventually a flotilla, anchorage, base, syndicate, or regional political
  relationship.

Atrocity, treachery, attacking other pirates, violating haven rules, or failing
patrons can close criminal opportunities even when it increases public
infamy. Conversely, being feared may improve intimidation while making targets
fight harder, travel in convoy, or summon more capable hunters.

## Navy, privateer, and pirate distinction

The three combat-oriented structures are deliberately different:

| Role | Work source | Authority | Target access | Reward |
|---|---|---|---|---|
| Navy | orders | institutional command | assigned patrols and lawful objectives | pay, rank, authority, mission credit |
| Privateer | legally scoped commission or letter | limited issuing-polity authority | named enemies, pirates, bounties, and adjudicated prizes | contract pay and prize-court award |
| Pirate | self-selected prey, leads, and deniable bargains | none beyond crew and local criminal relationships | anything actually found and interceptable | fenced prizes, patron payment, and underworld standing |

A pirate may ignore every lead and commission and run amok. The structure
exists to reveal actionable opportunities, establish crew expectations, and
create reasons to travel; it is not a gate on criminal action.

## Daily simulation integration

`SystemDay` processing generates no player-specific pirate quest. It advances
real traffic, offers, criminal demand, security, piracy, and news. That
authoritative state may then produce a lead or commission referencing the
existing target.

When a player acts on it:

- the target is removed, diverted, delayed, damaged, captured, or destroyed in
  the same state used by background traffic;
- cargo and prizes cannot be duplicated by statistical simulation;
- witnesses, evidence, delayed mail, warrants, and political reaction follow
  normal propagation;
- competing pirates or patrons observe only what their information can
  actually reveal; and
- a missed or obsolete opportunity resolves naturally rather than respawning.

## Implemented boundary and later depth

The server projects leads and optional commissions from current seed-derived
traffic; every target ID must still be present when the captain attempts the
intercept. Free predation uses the same local-contact list and immediately
creates delayed electronic evidence and warrants when not authorized. A captured
ship is valued from its catalog price and surviving condition, then realizes a
deterministic 10–30 percent at a fence. Successful commissions and fenced
prizes improve underworld standing and relieve crew pressure.

Cruise articles persist hunting system, duration, crew share, ship-fund share,
and prohibited targets. Each queued monthly account increases crew pressure
when an active cruise takes no prey and relieves it when a prize is secured; a
fruitless expired cruise also harms underworld standing. At maximum pressure
the crew refuses a new cruise. This is the intentionally coarse first crew-
politics mechanic. Regional haven standings, misinformation/stings, prize
crew logistics, desertion and mutiny events, and balance calibration remain
later depth rather than missing Milestone 6 persistence.
