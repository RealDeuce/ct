# Player-Creation Transaction and Character Records

*Status: first protocol, door, validation, and persistent-record slice
implemented; offer financial/authority/refit terms remain pending,
2026-07-28*

## Creation Flow

New-player setup has three ordered customization steps:

1. customize the captain using the **Starting Captain** pools;
2. select one of the three starter offers available in the home polity cell
   and customize the ship within that offer; and
3. review the ship's fixed role templates and name each initial individual
   crew member.

The ordering matters. The selected ship and package determine the required
billets, allowed crew establishment, equipment allowance, and employment
terms shown in the third step. The UI may keep drafts between steps, but none
of them changes authoritative state.

Final confirmation submits one `CreatePlayer` command. The server validates
the complete proposal against the persisted setup options and either commits
the entire starting estate or commits a rule-rejection response without
creating any part of it.

## New-User RPC Walkthrough

The target protocol flow is:

1. `ClientHello` authenticates the attested player identity.
2. `ServerHello` reports phase `newUser`.
3. `GetCaptainCreationOptions` returns the current setup revision, Starting
   Captain characteristic and skill pools, permitted identifiers,
   constraints, and a legal default captain.
4. The client customizes the captain locally.
5. `GetStartingShipOffers` returns the three comparable offer summaries in
   the player's home-polity matrix cell. Their canonical design selections
   come from `catalog/starting-offers.toml`.
6. `GetStartingShipOptions(setupRevision, offerId)` returns the selected
   offer's complete default ship, option groups, legal selections,
   constraints, obligations, and customization envelopes.
7. The client customizes the ship locally.
8. `GetStartingCrewPlan(setupRevision, offerId, captainDraft,
   shipCustomization)` validates the preview and returns the resulting ship
   characteristics, required and optional crew slots, fixed role-specific
   characteristics and skills, employment terms, coverage requirements, and
   legal default crew.
9. The client reviews each crew template and supplies its name. Changing the
   ship fit or the captain's proposed billets requires another crew-plan
   query.
10. `CreatePlayer` resubmits the complete captain, offer, ship, and crew
    proposal and creates the player atomically.

The separate ship-options round trip is intentional. `GetStartingShipOffers`
stays compact and comparison-oriented, while detailed component choices and
constraints are fetched only for ships the player inspects. There are only
three offers and setup is not latency-sensitive.

The target query commands are read-only and valid only in `newUser`. They do
not reserve a ship, consume a crew slot, or persist a client draft. They
return data from one committed setup revision. Every response includes that
revision; every dependent query supplies it. A stale revision receives a
normal rule rejection with the current phase and revision so the client can
restart or refresh the affected portion of the wizard.

`GetStartingCrewPlan` is also the authoritative preview of a customized ship.
It removes the need for a separate “apply option” RPC after every local edit:
ship option groups enumerate their constraints, and the crew-plan query
performs full server validation and returns the derived fit before crew
editing begins. The final `CreatePlayer` never trusts the preview and repeats
all validation against the submitted revision.

Conceptually, the query payloads are:

```text
GetCaptainCreationOptions

GetStartingShipOffers

GetStartingShipOptions
  setup_revision: UInt64
  starting_offer_id: UInt32

GetStartingCrewPlan
  setup_revision: UInt64
  starting_offer_id: UInt32
  captain: CaptainDraft
  ship_customization: List<OfferOptionChoice>

CaptainCreationOptions
  setup_revision: UInt64
  characteristic_point_buy: CharacteristicPointBuy
  skill_pools: List<SkillPool>
  permitted_skills: List<SkillDefinition>
  default_captain: CaptainDraft

StartingShipOffers
  setup_revision: UInt64
  offers: List<StartingShipOfferSummary>

StartingShipOptions
  setup_revision: UInt64
  offer: StartingShipOfferDetail
  option_groups: List<OfferOptionGroup>

StartingCrewPlan
  setup_revision: UInt64
  ship_preview: StartingShipPreview
  slots: List<StartingCrewSlot>
  coverage: List<BilletCoverageRequirement>

CharacteristicPointBuy
  minimum: UInt8
  maximum: UInt8
  neutral: UInt8
  budget: Int16
```

The captain's characteristic cost is `score - neutral` for each of STR, DEX,
END, INT, EDU, and CHA. The submitted costs must total `budget` exactly.
Setup revision 1 sets the range to 2 through 12, neutral to 7, and budget to
12. This is equivalently a total of 54 characteristic points, centered on an
all-9 captain. Scores below 7 refund points at the same linear rate that
scores above 7 consume them.

A `StartingCrewSlot` has a stable slot ID within the offer revision, required
or optional status, allowed billets, its fixed role template, authoritative
employment terms, and a legal default crew member. The final request
identifies the slot and supplies the member's name and selected training
skill. The server derives the characteristics, skills, and required training
weeks again from the slot when it materializes the person, so client-supplied
character mechanics cannot alter the template or shorten the course.

## Creation Request Shape

The request contains selections, not client-calculated game state:

```text
CreatePlayer
  setup_revision: UInt64
  starting_offer_id: UInt32
  captain: PersonDraft
  ship_name: Text
  crew: List<InitialCrewDraft>

PersonDraft
  name: Text
  characteristics: Characteristics
  skills: List<SkillRating>
  training: SkillTraining

InitialCrewDraft
  slot_id: UInt16
  name: Text
  training_skill: SkillId

Characteristics
  strength: UInt8
  dexterity: UInt8
  endurance: UInt8
  intelligence: UInt8
  education: UInt8
  charisma: UInt8

SkillRating
  skill: SkillId
  level: Int8

SkillTraining
  skill: SkillId
  needed_weeks: UInt16
  current_weeks: UInt16

```

`SkillId` is a closed schema enum rather than player-supplied text. The client
does not submit component prices, derived ship characteristics, cash, salary,
loyalty, certifications, IDs, ownership shares, or institutional authority.
Those values are derived from authoritative offer and polity state.

The offer's ship is a specific immutable design revision from
[`ship-catalog-records.md`](ship-catalog-records.md). Later catalog corrections
never mutate a ship that has already been created.

The 27 base-design selections are adopted in
`catalog/starting-offers.toml`. That registry does not contain or imply title,
equity, debt, authority, operating reserves, staffing, or refit budgets; those
remain versioned offer terms returned by the setup queries.

`setup_revision` identifies the persisted options used to build the draft. It
prevents a reconnect, rules change, or authorized polity change from
silently applying a stale client draft to different offers. It is first
delivered by `GetCaptainCreationOptions` and accompanies every later setup
response. The read-only walkthrough does not split final creation into
multiple mutations. This is a domain revision for that player's setup
options, not the envelope's global committed-state revision; unrelated
universe transactions therefore do not invalidate an in-progress wizard.

## Persistent Character and Crew Shape

The captain and every individually modeled crew member share a persistent
`Person` record:

```text
Person
  person_id: UInt64
  origin_bbs_id: UInt32
  origin_system_id: SystemId
  name: Text
  characteristics: Characteristics
  skills: List<SkillRating>
  training: SkillTraining
  health: CharacterHealth
  life_status: LifeStatus
  advancement: CharacterAdvancement
  revision: UInt64
```

The server assigns `person_id`. Health, injury, aging if retained, death, and
training belong to the person rather than to a ship assignment. Scoped
facts such as naval rank, title, citizenship, warrants, reputation, and
faction standing are separate records keyed by the person and relevant
institution; there is no universal SOC field.

A crew member is a person plus a current service relationship:

```text
CrewService
  person_id: UInt64
  ship_id: ShipId
  command_player: PlayerIdentity
  service_kind: ServiceKind
  service_appointment: BilletAssignment
  active_watch_billets: List<BilletAssignment>
  compensation: CompensationTerms
  service_start: GameTime
  service_end: Optional<GameTime>
  morale: MoraleState
  loyalty: LoyaltyState
  risk_tolerance: RiskTolerance
  availability: CrewAvailability
  revision: UInt64
```

`ServiceKind` distinguishes at least owner/captain, salaried hire, profit- or
prize-share crew, and institutionally assigned naval personnel.
`CompensationTerms` stores the authoritative salary, signing payment, term,
and any profit or prize share. Institutional restrictions and legal or
faction consequences remain scoped records rather than flattened universal
numbers.

The service appointment is the person's durable organizational post and helps
define the ship's role catalog. It is not overwritten when watch duties
change. `active_watch_billets` is a separate zero-or-more list: empty is off
watch, while several entries represent legal role doubling. Command authority
is also separate, so an off-watch captain remains captain. Only one person may
hold the active Pilot role; other roles may be shared.

Separating `Person` from `CrewService` allows a character to change ships,
leave employment, enter naval assignment, become a captain, or survive the
loss of a vessel without copying or replacing their characteristics and
skills. A convenient API response may present a joined `CrewMember` view, but
that view is not the authoritative storage boundary.

The player's captain is represented by a `Person` plus the player-to-captain
and command relationships. Captaincy is not a different character type. The
captain may also occupy one or more ship billets, subject to the ordinary
simultaneous-action rules.

Large late-game crews may use aggregate department records as already
described in the character audit. This individual record is required for the
captain, starting crew, officers, and other mechanically significant people;
it does not require materializing every rating aboard a dreadnought.

## Authoritative Validation

Before creation commits, the server checks:

- the authenticated player identity still has no player record;
- `setup_revision` and `starting_offer_id` identify that player's persisted
  home-polity setup;
- captain characteristics consume exactly the advertised point-buy budget,
  captain skills consume exactly the permitted Starting Captain rating
  slots, and all ranges, levels, identifiers, and duplicate constraints
  hold;
- every crew name is valid and every crew draft fills a slot returned for the
  same offer revision, with no duplicate required slots and no invented
  optional slots;
- the selected ship permits the submitted crew count and the captain/crew
  billet assignments cover its required initial establishment;
- no person is assigned to incompatible simultaneous stations merely because
  they possess both skills;
- all ship options are offered, mutually compatible, and within the separate
  displacement, equipment/refit, staffing, and reserve envelopes;
- names and list sizes meet protocol limits; and
- all catalog, polity, title, financing, employment, and derived-stat rules
  still hold at the submitted revision.

The Starting Captain skill pool contains three level-2, six level-1, and
three level-0 selections. Each initial crew role template contains two fixed
level-2, four fixed level-1, and three fixed level-0 skills. The crew ratings
calibrate template competence; they are not player-selectable slots. Changing
a role package requires an explicit setup revision.

The Starting Captain characteristic pool permits scores from 2 through 12,
prices each score as `score - 7`, and requires an exact budget of 12. Thus
every legal captain has a total characteristic score of 54, but that total
may be distributed freely; the legal all-9 default is not a required
multiset. Every initial crew role uses the standard characteristic multiset
`[10,9,8,8,7,6]`, assigned by role: pilots emphasize DEX and INT, engineers
EDU and INT, marines STR and END, and so forth. The server selects both that
assignment and the role's standard skill package. Better personnel are
acquired later through normal hiring.

Jack of All Trades is a special case rather than an unrestricted rating-slot
selection. It may occupy a level-1 or level-2 slot, but never a level-0 or
level-3 slot. Level 0 has no mechanical effect, while level 3 would erase the
entire ordinary untrained penalty across every skill and is not comparable in
value to a conventional level-3 specialty. Jack of All Trades still does not
provide trained status or permit a task that forbids untrained attempts.
The default captain receives Jack of All Trades 2 so the captain can cover an
otherwise-vacant shipboard station at a meaningful disadvantage to a trained
specialist.

## Atomic Instantiation

A successful transaction creates or links, at minimum:

- the `Player` record for the attested identity;
- the captain's `Person` record and player/captain relationship;
- the selected `Ship` record, title/command authority, installed fit,
  liabilities, and operating reserves;
- each initial crew `Person` and `CrewService` record;
- initial institutional rank, reputation, citizenship, or charter records
  granted by the offer;
- the initial concrete capital-world starport location from which the docked
  phase is derived; and
- a consumed marker for the persisted setup offer.

Server-assigned IDs and all derived values are returned in a creation result.
If validation rejects the proposal, none of these records exists. An
unexpected storage failure follows the engine's fatal rollback policy rather
than continuing with an uncertain partial estate.

The required location and facility shape is specified in
[`docked-operations.md`](docked-operations.md). Creation must not persist an
independent `Docked` boolean that can disagree with ship location.

## Current Implementation Boundary

The shared schema and both language bindings implement the four read-only
queries and complete `PlayerCreation` proposal. The server validates the BBS
matrix cell, setup
revision, offer, captain point-buy budget, exact captain skill-rating pools,
crew slot identities, and names. One transaction
creates the player link, captain and crew people,
catalog-ID-and-revision-backed ship at the BBS home, and crew-service
records.

Setup revision 1 uses core-cascade skill identifiers, the adopted skill-slot
counts, and the 2-through-12/neutral-7/budget-12 Starting Captain point buy.
The server publishes those parameters and a legal all-9 default, and the door
shows score modifiers, individual costs or refunds, and the remaining budget
while editing. Initial crew mechanics are server-derived fixed role templates
using the `[10,9,8,8,7,6]` standard array and fixed 2/2/1/1/1/1/0/0/0 skill
packages. The door first displays the complete role/name roster with
role-specific default callsigns. Enter accepts every default; selecting an
entry displays that person's template and permits renaming or changing the
training target. Every person starts with zero completed weeks. The default
target is the first, primary skill in the role package: Leadership for the
captain and the role's principal operational skill for crew.

The catalog does not yet define offer-specific financing, title, command
authority, reserves, refit option groups, compensation, or billet-coverage
terms. The implemented ship screen therefore names and displays the canonical
ready-to-depart fit but offers no invented refit choices. One named officer,
leader, or senior specialist represents each catalog crew role. Its
crew-service record also carries the total assigned-position count. Additional
positions are aggregate supporting personnel: they do not share the named
person's name, and the creation flow does not require individual names for
them.
