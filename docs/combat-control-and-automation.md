# Combat Control and Automation

## CE Turn Boundary

Space combat follows the core CE turn structure. A combat turn is one
kilosecond. Vessels act in initiative order; when a vessel activates, its crew
resolve their actions together. Each crew member receives the CE minor and
significant actions appropriate to their assigned position, while Dodge, Fire
Sand, Point Defense, and Trigger Screens consume the vessel's limited reactions
when their triggers occur.

Cepheus Trader submits one authoritative **joint crew order** for a vessel
activation. It includes each modeled crew or station team's minor/significant
action and a prioritized reaction policy. This avoids mid-attack network
prompts while preserving the CE reaction limit. The server validates and
commits the complete order atomically against the current combat revision.

Large establishments retain the existing data-model boundary: named officers,
leaders, and senior specialists receive individual orders, while aggregate
supporting personnel act as station or formation teams. The game does not
create hundreds of indistinguishable UI prompts merely because a naval design
has hundreds of supporting crew.

## Two Automation Levels

Combat has two classic, rules-based controllers. Neither is an LLM, changes a
crew member's skills, grants a dice modifier, or bypasses the ordinary combat
rules.

### 1. Conservative crew defaults

When the commanding player is online, every activation opens with a complete
most-conservative legal order already selected for every crew assignment. The
player may change any of those actions and reaction priorities before
submitting the turn. Accepting the defaults is a valid complete turn.

The conservative controller prioritizes immediate preservation without making
irreversible strategic choices for an attentive player:

- the Captain coordinates defensive, withdrawal, or emergency work;
- the Pilot avoids collision, attempts Evasive Maneuvers, breaks pursuit, and
  preserves or opens an escape vector;
- the Navigator checks range or prepares a viable escape Jump when useful;
- Sensors maintains communications or distress traffic and uses Electronic
  Warfare defensively;
- gunners reserve reactions for Point Defense, screens, and sand, holding
  offensive fire unless defensive fire is necessary to protect withdrawal;
- engineers and damage-control teams preserve power, life support, maneuver,
  and escape capability;
- security and marines prepare to repel boarders rather than initiating a
  boarding action; and
- unassigned personnel shelter or prepare for emergency evacuation.

It does not choose pursuit, ramming, hostile docking, offensive boarding,
unnecessary missile expenditure, surrender, or abandonment as an ordinary
default. The exact action still depends on legal targets, current damage,
range, missiles in flight, pursuit state, collision hazards, available
reactions, and resources; “conservative” is not a static action name.

### 2. Risk-directed tactical controller

Each commanded ship has a player-selected minimum estimated probability of
winning its current encounter objective. Lower values are more aggressive;
higher values require stronger odds before continuing to fight. The objective
is scenario-specific: protecting a convoy, escaping with intelligence,
holding a position, forcing inspection, capturing a prize, or defeating an
opponent can each define victory differently.

The controller selects the best evaluated legal joint crew order consistent
with that threshold. “Optimal” means the best result found by a versioned,
bounded tactical search over the same information available to the captain.
It does not inspect hidden authoritative enemy state. Sensor uncertainty,
misidentification, concealed damage, unknown reserves, and opponent behavior
remain probability distributions, so the displayed likelihood is an estimate
and never a guarantee.

At each activation the controller reassesses:

1. If estimated objective success meets the configured threshold, pursue the
   objective using the best evaluated legal order.
2. Otherwise, attempt withdrawal. Running is a sequence of real CE actions:
   change speed and range, evade, break pursuit, jam targeting, prepare a Jump,
   and protect the escape systems. It is not an automatic state transition.
3. If viable escape falls away, surrender is a legal final strategy when the
   opponent is expected to accept it and it offers the best survivable result.
4. If surrender is unavailable, rejected, or assessed as worse than
   abandonment, assigning survivors to and launching available escape craft is
   a legal final strategy.

Surrender and abandonment have their real legal, ownership, capture, crew,
and survival consequences. The controller cannot assume that a pirate honors
surrender, that escape craft are safe, or that an opponent permits evacuation.

## Online, Offline, and Delegated Control

An actively connected commander receives conservative defaults and may edit
them. If the player is absent, disconnects, or fails to submit before the
activation deadline, the risk-directed controller acts using the ship's
persistent policy. The exact real-time deadline and notification/grace policy
remain to be selected.

As a Milestone 7 quality-of-life feature, provide a small HTTPS companion web app
for standards-based Web Push enrollment. A player pairs the browser with their
game identity, may revoke the subscription, and can receive at least
activation-soon and activation-ready notices without leaving the page or
browser open. The app is not an alternate gameplay client and accepts no game
orders. Push delivery is optional and best-effort: a missed, delayed, or
duplicate notice never changes the activation deadline or prevents the
risk-directed controller from acting. Lock-screen text must avoid disclosing
tactical details.

The risk-directed controller must also be invokable explicitly while online.
Otherwise disconnecting would give a player access to a supposedly stronger
controller that a connected player could not select. Manual play offers
control, explanation, and the ability to pursue unusual intentions; it is not
a penalty for remaining connected.

All controllers run server-side against a censored `CombatView` equivalent to
the player-visible snapshot. Their selected orders, input-view revision,
algorithm revision, estimated outcomes, and decisive policy branch are stored
for audit and replay. Search randomness, if used, is deterministic and domain-
separated; it cannot reroll because a screen is refreshed.

## Observation and Third-Party Intervention

A combat encounter does not instantly notify every ship in the system. Sensor
emissions, weapons fire, transponders, distress calls, and deliberate reports
become observations at another ship no earlier than:

```text
observation_time = emission_time + separation / c
```

Detection quality then determines what that observer can infer. The observer
may know only that high-energy activity exists, may identify the participants,
or may have enough evidence to estimate who initiated unlawful force. A
distress claim is evidence, not authoritative truth. Later emissions can
improve or contradict the first observation, each with its own light-speed
delay.

An observing captain may ignore the incident, monitor it, report it, alter
course to investigate, attempt rescue, or intervene for a side. Responding is
physical interplanetary movement, not a combat-screen transition. The server
solves an intercept using the observer's actual position, velocity, thrust,
fuel, and the participants' projected motion. A responder joins the encounter
only at its achieved arrival time and enters the initiative sequence from that
point forward. If combat has already ended, the response may instead become a
pursuit, rescue, arrest, salvage, or aftermath encounter. Nothing acts on or
learns information before its causal arrival.

The encounter frame remains isolated from unrelated universe activity, but a
ship whose causal observation and trajectory intersect the incident becomes a
participant at the relevant scheduled transaction. Conversely, the original
combatants do not instantly learn that help is coming: a responder's course
change, acknowledgement, or challenge reaches them only through ordinary
sensor and light-delay rules.

### Player intervention policy

An online player is prompted when their ship receives actionable evidence and
may choose among the legal responses. Every commanded ship also carries an
offline intervention policy so the decision does not depend on the player
being connected. At minimum the policy records:

- whether to ignore, observe/report, investigate, rescue, or consider armed
  intervention;
- protected allies, authorities, distress classes, and legal jurisdictions;
- the minimum confidence that an aggressor or lawful side has been identified;
- the minimum estimated success probability and maximum acceptable loss;
- maximum diversion time, distance, fuel, and reserve expenditure; and
- whether to announce, challenge, request assistance, or approach silently.

The policy is evaluated against the same delayed, uncertain evidence the
captain would see. It cannot use the server's hidden truth about who attacked
first. Committing a response suspends or replaces the ship's current
continuation plan through the normal ordered-transaction rules. Naval orders,
escort contracts, or convoy duties may constrain the available policy without
making a physically impossible response possible.

### Computer-controlled ships

Enforcement ships generally investigate credible combat or distress activity
within their authority and operational reach. They identify, challenge,
separate, arrest, escort, or engage according to their jurisdiction and rules
of engagement; uncertain identity or aggression may cause them to approach
without immediately choosing a side.

Other computer-controlled ships generally consider armed intervention when
they have a comfortable projected advantage over the combatants and the
available evidence clearly identifies an aggressor. Allegiance, law, standing
orders, crew disposition, reputation, reward, damage, ammunition, fuel,
mission commitments, and fear of reinforcements modify that choice. An armed
merchant may aid an obvious victim it can safely overwhelm, but is not a
universal volunteer police force. Ships that decline combat may still report
it, render later aid, carry witnesses, or update enforcement and news state.

## Offline Post-Combat Recovery

After an offline-controlled encounter, if the player's crew still controls the
ship, it immediately begins all feasible onboard recovery without waiting for
the player to reconnect. Temporary CE battlefield-repair coverage expires at
encounter end before recovery is assessed. The automatic work priority is:

1. Life support
2. Maneuver drive
3. Jump drive
4. Weapons

The priority is dependency-aware. Hull containment, power, bridge, fuel, or
control work needed to restore a higher-priority capability is part of that
goal rather than an excuse to repair a lower-priority discretionary system
first. “Weapons” includes the installed offensive and defensive combat systems
that the surviving crew can actually service.

The recovery planner assigns the best available qualified crew, repair drones,
tools, spares, and repair supplies while retaining the minimum safe watch and
respecting injury, fatigue, access, and environmental constraints. It
continues until no safe supported repair remains, required materials or skills
are exhausted, the ship reaches a facility, or the player reconnects and
changes the plan. Each work step is a scheduled authoritative transaction and
consumes its actual game time and supplies.

Automatic recovery does not convert battle patches into permanent repairs.
It may stabilize, jury-rig, or properly repair only to the extent permitted by
the eventual field-repair rules and onboard capability. Destroyed systems and
work requiring a shipyard remain damaged and become repair work orders. If the
crew surrendered, abandoned the ship, was captured, or is incapable of acting,
the player's recovery planner does not control the vessel.

## Persistent State

The combat model needs at least:

- `CombatAutomationPolicy`: ship/captain, minimum victory probability,
  revision, and effective time;
- `EncounterObjective`: the outcome whose success probability is being
  estimated;
- `CombatView`: the censored evidence and belief state available for a ship's
  activation;
- `CrewOrderSet`: actions for named crew and aggregate station teams;
- `ReactionPolicy`: ordered triggers, eligible operators, ammunition/resource
  limits, and reaction budget;
- `AutomationDecision`: controller and algorithm revision, estimated outcomes,
  selected branch, and resulting order-set ID; and
- `CombatEmission` and `CombatObservation`: time and position, signal or
  evidence type, causal arrival time, sensor result, provenance, and confidence;
- `InterventionPolicy` and `InterventionDecision`: protected interests,
  authority, evidence and risk thresholds, diversion limits, selected response,
  input revision, and explanation;
- `InterventionPlan`: intercept solution, projected arrival, participant side
  or neutral purpose, announcements, resource limits, and suspension/result;
  and
- `PostCombatRecoveryPlan`: retained control, ordered capability goals,
  assigned personnel, supplies, work steps, and suspension reason.

## Implemented Milestone 6 Boundary

The authoritative round is 1,000 game seconds and its shared order window is
about 35.7 real seconds at the fixed clock rate. Every active vessel submits
against the same combat revision; initiative orders the resolution after the
window closes. A missing player order invokes the persistent risk controller.
The default success threshold is 70 percent. Its versioned search considers at
most 64 candidates, evaluates the best eight with 256 deterministic three-round
rollouts each, and stores its selected branch and estimate.

The engine implements range changes, attacks, delayed missiles, point defense,
damage, surrender, abstract boarding, withdrawal, and escape-craft
abandonment. It accepts any number of vessel participants. A qualifying real
traffic contact may receive the incident after a deterministic causal delay
and join on a later activation boundary; it is not created by the combat.

Direct player-versus-player interception uses that same shared record and turn
boundary. Every player participant is its actual stored ship and crew. A
captain directly commanding a participant may submit one joint order per
revision; missing orders and separately commanded player vessels use their own
persistent risk-directed policies. One scheduled event resolves the round and
commits expenditure, damage, casualties, legal evidence, and each participant's
perspective atomically.

Surrender or successful boarding transfers the actual vessel, cargo, unique
objects, and prize title rather than materializing a duplicate. Real prisoners
remain attached to their original identities for parole and recovery. A
captain commanding a different ship is not pulled into the encounter and
learns the remote outcome only when a causal private report arrives.

At combat end all battlefield coverage expires. A retained-control ship
persists a recovery watch ordered by life support, dependencies, maneuver
drive, jump drive, and weapons. At a capable yard the ordinary proper-repair
transaction is used. Away from a yard, the best available conscious named crew
member attempts one shared-resolver field-recovery task using EDU, Mechanic or
Jack-of-All-Trades, injury/fatigue, and the field-equipment penalty. That work
consumes one full game day per attempt and removes an underlying sustained hit
only on success. Destroyed installations remain blocked for refurbishment or
replacement.

Terminal loss remains a state of the same player identity rather than a dead
end. A surviving captain returns after seven days for abandonment, fourteen
days for other destruction/loss, or thirty days after capture or surrender. A
dead captain requires a named successor. Irrecoverable bankruptcy is a docked,
default-only proceeding that liquidates the whole fleet and installs a named
successor in the original starter class under a new lien. Career and warrant
history survive both forms of succession. The unique apple pie follows the
physical result: captor custody, system wreckage, or destruction; destruction
therefore activates the already-settled universe-reset rule.

Detailed emission classification, player-owned third-party response policy,
pursuit/rescue aftermath, deception refinements, and the optional Web Push
notifier remain later Milestone 7 expansions. They extend this implementation
without changing the shared round, queue, or offline-control contract.
