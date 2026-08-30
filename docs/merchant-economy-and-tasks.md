# Merchant Economy and Tasks

*Implemented redux boundary: 2026-08-03*

## Market

The authoritative commodity catalogue contains the six common goods and all
35 generic results from the revised *Bounded Fortune* trade table. Optional
named examples and other upstream Product Identity are not data. Commodity
identifiers preserve their source-table meaning and are not assumed to form a
dense integer range.

Each system/day derives finite stock from the system seed, starport, primary
world, and trade codes. Player purchases persist shared consumption; reads do
not reroll or reserve stock. Ordinary quotes are a separate negotiation:
arrival at a market begins a seven-game-day price window, while sale terms for
a lot acquired during that stay begin at acquisition. Daily stock changes do
not reroll those terms. A full week permits another negotiation, a different
system supplies a different market immediately, and named events may still
shift the applicable price tier. Quotes also depend on the captain's Broker
skill and Charisma. Purchase outcomes use 80%, 90%, 100%, or 120% of base price;
sale outcomes use 30%, 15%, 2%, or 0% markup over base price. Generated local
tariffs are shown in basis points and included in both sides of the quote. The
ordinary local bid is capped below the local ask. At the lot's origin it is
also capped below the actual purchase price, so waiting in the same berth can
never turn an ordinary resale profitable. A named private buyer is a separate
negotiated transaction and may cross the public spread.

Cargo Exchange price plots compare the captain's current quote against an
absolute, universe-wide market-value span for the commodity, not against a
distribution tailored to that captain. Purchase spans include the rules'
80%-to-120% negotiation outcomes and the generated universe's 5%-to-12.5%
import-tariff bounds. Sale spans include the 100%-to-130% outcomes, export
tariffs, and the ordinary bid/ask spread cap. The lower quartile, median, and
upper quartile divide that absolute credit span into four equal intervals;
they are market-value landmarks, not probabilities for the current captain.
The current quote still reflects Broker skill, Charisma, local trade codes,
events, tariffs, and (for ordinary sales) the local ask.

For purchases, a quote below Q1 is favorable/green, Q1 through below the
median is middling/yellow, and the median or higher is unfavorable/red. Sales
reverse that judgment: above Q3 is favorable/green, above the median through
Q3 is middling/yellow, and the median or lower is unfavorable/red. Every row
also prints `low-price`, `mid-price`, or `high-price` buy/sale wording so color
is never the only signal. The plot uses minimum, Q1, median, Q3, maximum, and a
current-price marker; it intentionally omits a mean.

Prohibited goods have no open-exchange buy or sale quote. A private-introduction
search uses Streetwise and Charisma to find a prohibited-goods supplier or
buyer; Broker and Charisma determine the resulting price. Restricted goods
remain identifiable for deeper permit enforcement.

Supplier and buyer research is concrete work. Physical canvassing, a private
introduction, or a hired local search takes 1D6 game days; an online search
takes 1D6 game hours. Each previous attempt in the same system and 30-day
market month applies DM-1. A hired local broker charges Cr500 when engaged and
supplies Broker-2; the named crewmember remains the ship's liaison. Completion
is admitted as `MerchantWork` to the single engine-input queue and records a
dated, sourced confidence range in Known Universe. Cancelling work changes the
assignment state; a later already-indexed completion safely becomes a no-op.

A buyer search requires a positive quantity of matching player-owned
speculative cargo aboard the commanded ship. Freight, contract cargo, and
unique objects cannot be used to begin one. The server applies this check
before charging a hired broker's commission. When the work finishes, its lead
cannot cover more matching cargo than remains aboard; no lead is produced if
none remains.

Completed research produces a finite lead rather than an endlessly reusable
hint. The lead records its observed and acquired dates, confidence, quantity,
price range, source, expiry, and revision. An available lead can be performed
immediately. Reservation is optional: it places ten percent of estimated value
in escrow to hold the scarce opportunity. The reservation expires through
scheduled system work, and expired or voluntarily released reservations do
not refund that opportunity payment. The port can therefore expose a real
scarce source or buyer without pretending that old market intelligence is
current stock.

Named market events have exact persisted effects on stock and price tiers and
originate matching agency news in the same system-day transaction. Ordinary
daily consumption and player purchases change the shared market independently
of observations, so a dated report remains evidence rather than omniscient
state.

A ship that complies with a customs inspection has its manifest tested
against the destination world's law level. Prohibited ordinary cargo is
physically confiscated and customs collects a fine of ten percent of its base
value, limited by the credits presently in the operating account. Restricted
cargo remains aboard at this boundary: permits, declarations, bribery, and
the legal consequences of an unpaid balance remain deeper law-enforcement
work beyond the current warrant boundary.

## Carriage and Tasks

Ordinary freight, passengers, and electronic mail use a standing carriage
declaration tied to one destination. The captain declares maximum freight and
passenger accommodation and whether to accept ordinary mail. Departure loads
only eligible offers for the committed destination. It never changes the
Flight Plan, and a later route edit never rewrites an obligation.

Flight-plan preview selects a concrete manifest, shows gross revenue and
brokerage, and hashes the offer revisions into the proposal. Commit accepts
those exact offers atomically or rejects the changed manifest. Freight becomes
a physical titled lot at the origin; passage reserves an actual eligible
berth; mail uses the same declared hop without inducing service. Steward and
accommodation requirements are validated rather than silently waived.

Generated offers include freight, passage, purchase orders, forward sales,
supply commitments, charters, couriers, and bounties. Acceptance creates a
durable Task with collateral, reserved capacity, deadline, payment, and
failure penalty. Freight is a titled cargo lot belonging to its principal;
speculative cargo remains player-owned. Tasks move independently through
accepted, sourcing/loading, in-transit, settlement, completed, and defaulted
states. A deadline is a concrete scheduled work item admitted through the
same ordered queue as player commands and every other simulator action.
Delivery is completed by docking at the destination, not by reaching an
arrival checkpoint. A Hold checkpoint therefore requires arrival watch before
the task deadline; a Through checkpoint may dock under standing orders while
the captain is away. Flight-plan preview identifies that distinction.

Task-ledger offer listings carry contextual availability reasons. The player
client hides unavailable offers by default, reports the hidden count, and can
show them with their reasons on request. Server-derived reasons cover current
hold or berth capacity, required passenger staff, and whether posting the
collateral would leave enough cash to clear the current berth. The ledger also
carries an authoritative route assessment for each offer. One continuous
search preserves time and fuel from the captain's current system through the
required pickup and onward to delivery. It enforces the offer closing time
during the search and supplies the final arrival used to classify the delivery
deadline. The client displays that assessment rather than issuing independent
per-offer course plots. These are refreshed observations, not
permanent properties of the offer, and they do not prohibit a captain from
accepting the associated risk.

The accepted record names one performing ship. Freight loading creates a
task-titled cargo lot; passenger, charter, and passenger-courier work creates
a manifest with class, head count, origin, destination, and embarkation time.
Purchase orders, forward sales, and supply commitments consume only matching
player-owned goods actually aboard. Partial terms pay the delivered fraction,
and recurring supply performances reset only after that performance settles.
Delivery removes physical custody exactly once and dispatches a settlement
filing to the issuing office. The office then mails the payment and collateral
release to the captain. The task remains awaiting settlement, and the credits
remain reserved, until that remittance reaches the captain. A restart does not
reconstruct manifests or infer delivery from location.

An offer claimed at its issuing office is awarded or declined there. A claim
filed anywhere else dispatches a sealed institutional message. Competing
claims are awarded by physical arrival order at the issuing office, and the
private award or decline reply must reach the captain before cargo or passenger
custody can transfer. An award also dispatches a closure notice over the
offer's original regional scope. Each remote copy remains actionable until
that notice arrives, so another captain may file against a stale local copy and
later receive a decline. The captain retains their own signed filing, so
reaching the issuing office makes the claim available there no later than the
ship's arrival; a separately carried copy may arrive first. Uploading the
captain-retained copy on arrival does not depend on accepting an ordinary
beacon mailbag. Institutional filings
are not misclassified as mail addressed to every captain at their destination.

The Task manager has revision-checked transactions to withdraw an unresolved
claim, cancel before custody, return custody at the origin, declare default,
or file a sealed dispute with the issuing office. Default forfeits reserved
collateral and assesses the stated failure penalty and non-delivery liability,
capped by available credits. A dispute keeps the obligation and its reserves
in place while its signed electronic filing is propagating. Recurring supply commitments
schedule and settle each performance independently. No path may duplicate
payment, cargo, capacity, or collateral after restart.

Private correspondence may target a fixed system along its exact known route
or a mobile captain across the reachable TTL sphere. It costs Cr1 per started
KiB per charged hop per started TTL week, uses a one-to-52-week elapsed-time
TTL, and is retained as physical encrypted carriage. Destination assistance is an optional Cr350,000
annual policy bound or cancelled while docked; cancellation has no refund.

Combat bounties may be accepted and retained, but they settle only from an
explicit qualifying combat or authority result. No placeholder success is
awarded merely because a deadline or target identifier exists.

## Finance, Ships, and Crew

Trader starts use 20% equity and 80% secured debt. The monthly principal
payment is purchase price divided by 240 and the full schedule spans 480
months; mandatory insurance is separately escrowed. Privateer ships are
sponsor-owned and navy ships institution-owned. Institutional funds are
restricted rather than interchangeable with personal credits. Restricted
operating credit pays mandatory insurance and other authorized vessel costs
before liquid cash, but never pays secured principal.

All 27 polity/career starting cells carry nonzero construction value and a
role-correct title/finance package. Refit choices are versioned and unique to
the offer. A cargo-to-fuel alternative is shown only when the selected hull
actually has the ten tons it consumes; every presented alternative is
server-validated as part of the one player-creation transaction.

A missed secured payment or mandatory insurance installment has one standard
accounting month of grace. Default creates a private impound order at the
captain's origin. The order is an ordinary mail object: a remote authority
cannot know or enforce it until that message physically arrives. The finance
display distinguishes the captain's knowledge of default from locally received
enforcement authority. Posting one overdue installment clears default,
withdraws the order, and schedules the next installment one accounting month
later; this transaction is available in flight through the impound message's
Finance action.

Ship and crew exchanges are finite, seed/day-derived port markets. Ship offers
draw from all 135 active Jump-capable starships in the validated catalog, not
merely the 27 starting offers, and respect the local world's TL. Claimed
offers disappear. Ship purchase settles the old lien and trade-in, creates
the new title and financing, moves ordinary cargo and crew, and schedules
condition accounting atomically. A trade-in is rejected while the vessel has
active Tasks, entrusted cargo, or passengers; retaining the old vessel leaves
its obligations and stores aboard it. Used ships retain actual age and a hidden latent quirk;
the advertised condition is not a diagnostic guarantee. Hired crew become
individual people with salary, service record, training, and off-watch status.
Personnel names use the setting-neutral OGC pools in
`catalog/person-names.toml`; given and family draws are independent,
deterministic for the market day, and unique within each displayed hiring
slate.

## Unique Ordinary Cargo

The universe contains exactly one one-kilogram cargo object named `apple pie
(made from scratch)`, priced at Cr1. It is bought, sold, carried, lost in
wreckage, and moved by background ships through ordinary cargo mechanisms.
It receives no exceptional price, traffic behavior, notice, or protection.

Its destruction—not sale, theft, jettison, loss, or change of custody—sets a
durable reset condition. The engine responds through the normal full-universe
initialization boundary while preserving BBS enrollment, credentials, and
sysop configuration. This paragraph is developer-only information and must
never be reflected or hinted at by player-facing text or behavior.

## Persistence

Merchant state is split into markets, cargo lots, Tasks, carriage
declarations, work assignments and their due indexes, finance, dated market
observations, finite-market claims, crew service, and unique cargo. Intent,
physical objects, money, and future eligibility are therefore independently
auditable. No compatibility reader exists while the game remains undeployed.
