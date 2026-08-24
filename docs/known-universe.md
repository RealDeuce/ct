# Known Universe Database

## Separate Player Surface

The Known Universe database and Message Management both deal with delayed,
incomplete, sourced information, but they are separate stores and interfaces.

Message Management is a chronological collection of discrete delivered
objects: news, mail, offers, orders, reports, warnings, and correspondence.
Its principal operations are reading, classifying, filtering, searching, and
accepting an offered obligation.

Known Universe is the shipboard operational model assembled from observations
and received data. Its principal operations are browsing places, comparing
routes and facilities, inspecting data age and confidence, and supporting
navigation, trade, and risk decisions. It must not be implemented as a
message-folder convention.

The Milestone-2 implementation persists the first narrow slice of this model:
the starting published navigation set, systems learned from physically
delivered public-origin messages, per-system acquisition time/source text, and
mapping disclosure state. The door's existing subject browser and course
plotter read that carried set. Confidence conflicts, detailed observation
records, market history, repository transfer, and editing old Secret Systems
entries remain later expansions; the arrival prompt implements the initial
public/direct/withheld/secret choice.

## Contents

The database can hold knowledge about:

- systems, coordinates, stellar and planetary architecture, and survey
  completeness;
- known jump links, empty-space staging routes, travel estimates, and
  navigation-data freshness;
- worlds, polities, borders, allegiances, law, warrants, and diplomatic
  conditions;
- starports, bases, facilities, fuel, repair capability, damage, congestion,
  fees, and closures;
- historical market prices and availability observations, with projected
  probabilities and acquisition timelines;
- traffic levels, patrol patterns, piracy and other hazards;
- contacts, institutions, registries, and authentication keys; and
- provenance needed to decide whether any of the above should be trusted.

Contracts and orders remain messages until accepted and then become Tasks.
They may refer to Known Universe subjects without becoming knowledge records
themselves.

## Observation and Provenance

An observation is immutable evidence:

```text
KnowledgeObservation
  observation_id
  repository_id
  subject_kind
  subject_id
  attribute
  observed_value
  observed_at
  acquired_at
  source_system_id
  source_message_id: Optional<MessageId>
  source_identity
  authentication
  confidence
  effective_until: Optional<GameTime>
```

`observed_at` is when the source measured or asserted the fact.
`acquired_at` is when this repository received it. Their difference matters:
a freshly delivered report may already describe an old market.

Conflicting observations are retained. An older report arriving later never
blindly overwrites newer evidence. A deterministic projection selects or
combines applicable observations into a cached current estimate, preserving
the evidence and reasoning needed to explain it.

Static facts such as coordinates may converge toward certainty. Dynamic facts
such as prices, traffic, facility damage, borders, and warrants decay in
utility with age. The UI must always be able to expose age, source, confidence,
and conflict rather than presenting a stale estimate as omniscient truth.

## Physical Information Boundaries

There is no ansible. A resolved system in the server database is not
automatically known to a player.

Knowledge belongs to a physical or institutional repository, not inherently
to every ship owned by an account. The active ship has an onboard repository;
ports, navies, banks, BBS polities, and other institutions may have their own.
Repositories merge observations only through physical contact, carried data,
or ordinary mail propagation. This prevents two widely separated player-owned
ships from becoming an accidental FTL communications channel.

The player-facing Known Universe manager normally opens the current ship's
repository. Access to another repository requires an in-world connection or a
previously synchronized local copy.

## Relationship to Messages

Messages and knowledge observations may share:

- origin and observation times;
- source identity and authentication;
- mail route and delivery provenance;
- expiry and confidence concepts; and
- indexes for delayed arrival.

A delivered data packet can atomically retain its message and add one or more
knowledge observations. Deleting, archiving, or classifying the message does
not delete those observations. Conversely, direct sensor measurements and
manual captain notes can update Known Universe without manufacturing a
message.

The propagation system routes messages and data packages. Known Universe is a
materialized recipient-side result, not a global bitmask saying which current
facts every system or player knows.

Message visibility is independent of knowledge ingestion. A delivery may be
hidden by the recipient's default Message Management filters while its
structured observations are still merged into that repository's Known
Universe database. Filters control presentation and notification; they never
prevent an otherwise addressed knowledge update from being received.

## Newly Materialized Systems

Server materialization and in-world discovery are separate events. Resolving a
new system for simulation creates no instantaneous remote knowledge.

An in-world discovery always creates local observations in the discovering
repository. It does not automatically create outbound mail. After the first
scan, the captain may send a free public mapping announcement, send an
encrypted direct filing to Earth, withhold it for now, or add the system to the
captain-private Secret Systems list.

Discovery does bootstrap inbound public mail. The discovering contact or the
first repository package carries the completed universal-broadcast checkpoint
and access to that immutable archive. A previously completed universal message
is therefore available in a repository established later without reopening
per-system propagation state. This is still physical carriage, not an ansible;
it neither publishes the discovery outbound nor teaches remote repositories
that the system exists.

A public announcement enters the ordinary mail network at the ship's current
system and propagates to reachable Known Universe repositories. A direct
filing follows one paid private route to Earth; intermediate repositories
cannot merge its encrypted observations. If that direct filing wins the
Federation award, Earth originates the free authoritative public mapping
announcement. Remote repositories learn only when a public package or an
independent report physically arrives. Full mail-class and encryption rules
are in [`mail-service-and-security.md`](mail-service-and-security.md).

A newly added BBS polity remains headline-level news at any distance, but
“headline-level” controls significance and rebroadcast priority rather than
delivery speed. Its first contact is one polity-originated packet carried by
the arriving ship. That dossier lists the polity's complete registered-system
catalogue, so receiving it adds every polity member to the local Known Universe
together; the unaligned stub and surrounding frontier are not members and are
not disclosed by the packet. It still reaches distant systems on the mail timeline. A
routine discovery of an uninhabited system is normally low-significance
rather than public headline news. If its captain sends a public system-
discovery package, that package still propagates fully. A direct filing
remains private until a winning Earth award creates the public announcement.
Default message filters suppress routine public notices so ordinary players
are not forced to read a stream of cartographic updates.

Once the founding mapping is public, current dossiers expose its BBS and
optional League affiliation as typed institutional data and render `Polity
(League)`; an independent BBS renders only the polity. Renaming a League
changes current dossiers after the current mapping is consulted, but never
rewrites the text of an already dispatched founding article. Non-public
mapping states disclose no institutional affiliation.

Public mail can carry both human-readable news and structured system
observations in one data package. Delivery atomically retains the public
message and merges its observations into the recipient repository even when
the message is hidden by the active filters. An encrypted direct filing is
merged only by its addressed Earth repository after decryption and validation;
relays retain ciphertext and routing metadata. Later public reports may refine
or contradict the first report without rewriting its historical evidence.

For a settled system, the same package can carry an authenticated Federation
discovery filing. The first valid filing committed to the Earth repository
wins the standing discovery award; dispatch time or arrival elsewhere does not
establish priority. Award value, validation, duplicate handling, and
Earth-based payment are defined in
[`settlement-and-system-survey.md`](settlement-and-system-survey.md).

### Disclosure state

The current repository distinguishes at least:

- locally observed, with no mapping notification sent;
- public mapping notification dispatched, with its message and custody
  provenance;
- direct Earth filing dispatched, with route, TTL, encryption-key, and custody
  provenance;
- known by received evidence to be publicly mapped; and
- suppressed by the current captain's Secret Systems list.

When the ship arrives somewhere it does not know to be publicly mapped, the
arrival flow prompts to send publicly, send directly to Earth, withhold, or
withhold and mark secret. The secret list is editable from Known Universe at
any time. It is captain-private and physically scoped, so viewing it aboard a
distant ship cannot become an accidental information channel. Withholding
does not prevent another observer from reporting the system, and dispatch
cannot be retracted later.

## Interface

The universal command-console key is tentatively `K`.

The interface is subject-oriented rather than chronological:

- system and world directory;
- route and reachable-neighbor view;
- ship-specific course plotting from the present system to a known primary or
  between any two known primaries, comparing fastest and lowest-fuel-cost
  plans;
- port and facility comparison;
- market observation history and projections;
- polity, law, warrant, and hazard overlays;
- data-age, confidence, conflict, and provenance inspection; and
- filters for route planning and ship-computer resource searches;
- disclosure state and an explicit send-mapping action; and
- the editable captain-private Secret Systems list.

Message Management may provide “open referenced system” and Known Universe
may provide “show source message,” but each retains its own navigation model.

The initial operational database is implemented. Player creation copies the IDs
of the home polity, every system in its locally plotted six-parsec frontier,
and every universally published system, including the initial Federation
frontier shell, into the captain record with one acquisition/observation game
second. `GetKnownDestinations`
looks up only those IDs, computes distance from the ship's carried current
system, marks systems within its catalogued jump rating, and returns primary-
world starport, population, TL, and a starting-packet provenance label. The
door displays this listing, and Flight Plan preview rejects destinations absent from
it. This is intentionally not a query of every system the server knows.

The read-only course plotter uses only systems in that carried listing. It
accounts for the commanded ship's Jump rating, tank endurance, fuel already
aboard for a present-location plot, and refueling feasibility at intermediate
systems. The fastest plan minimizes elapsed time and then fuel purchases. The
cheapest plan minimizes purchased-fuel credits and then elapsed time, using a
nearest known gas giant when the ship actually carries fuel scoops and enough
processing machinery. Each route ends docked at the destination primary.
Planning time includes normal Jump time, primary-world Jump-locus travel, the
mean CE skimming duration, processor throughput, and the seed-derived orbital
geometry of the gas-giant detour at the observation epoch. The displayed cost
is deliberately labelled fuel purchases: payroll, maintenance, port fees,
damage risk, encounter delay, and other operating costs remain outside the
estimate until those systems become authoritative. Actual skimming will roll
duration and failures rather than inheriting the planner's mean.

Hydrographic data never implies that a water source is available to the
captain. A future water/ice waypoint is routable only when carried knowledge
identifies both a usable source and a legal or practical access basis. The
ordinary free sources are unoccupied bodies and ice-bearing belts. Inhabited
world water requires known permission or institutional authority; a proposed
forcible extraction is a hostile operation and is not offered as an ordinary
“cheapest route” fueling stop.

The current bootstrap vector records the systems known at creation. New
knowledge subsequently arrives through store-and-forward mail and mapping disclosure;
future per-attribute observations and market intelligence must live in their
own knowledge records rather than expanding the player row.

The browser exposes exact Galactic coordinates, newly resolved survey
contacts, editable pre-dispatch Secret Systems entries, and course plotting.
Fastest or cheapest course results can be copied into Flight Plan without
making Known Universe the owner of the active route. Every Jump breakout at a
stellar system or at plotted deep-space coordinates materializes the
six-parsec arrival volume and adds its resolved stellar contacts as private
carried observations; a later Jump can depart a deep-space hold for one of
those candidates.

On request, the server also suggests a single fast-to-compute course through
all active accepted-task stops assigned to the commanded ship. Its bounded beam
search respects pickup-before-delivery precedence and ranks alternatives by
deadline risk and estimated travel time. It may consolidate common stops or
revisit a system, but it is deliberately heuristic rather than an exhaustive
travelling-salesman search. The resulting course carries fuel between segments,
uses only directly importable carried or port fuel, and must still pass Flight
Plan preview before filing.
