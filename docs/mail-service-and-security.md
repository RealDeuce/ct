# Electronic Mail Service and Key Security

## Store-and-Forward Service

Electronic mail is digital. Within a system, mail beacons broadcast or relay
it at light speed. To cross a Jump boundary, a beacon downloads an electronic
mailbag to a departing ship and the destination beacon accepts the upload on
arrival. The bundle consumes negligible cargo capacity; physical letters and
parcels are cargo instead. There is no live interstellar directory lookup,
cancellation, delivery receipt, key
revocation, or balance update across a Jump boundary. Beacons and relays use
the persistent due-time event queue and ordinary carrier capacity described in
the simulation design.

Messages have stable IDs, origin and dispatch times, a service class, one of
four importance bands, an immutable subject/article body, addresses, payload
metadata, propagation plan, and an absolute game-time
`expires_at` where the service requires a TTL. The immutable archive is
retained after expiry; TTL ends routing, holding, acceptance, or delivery. It
is never a hop counter.

The initial fixed-system simulation substrate now implements immutable
messages, route-specific envelopes, beacon queues, ordinary simulated carrier
ships, bounded mailbags, named custody legs, one-week Jump arrivals,
multi-hop forwarding, final delivery, absolute-time expiry, deterministic
recovery, and a custody audit. Player ships use the same exact-hop queues: a
departure can seal one eligible bag into the ship record, arrival hands it off
and pays the recorded stipend atomically, and replay cannot deliver or pay it
twice. Arrival receipts and per-captain readership/classification are separate
from institutional availability.

Each captain's arrival packet applies a durable minimum importance per service
class. The default suppresses routine public-service and traffic copy while
retaining notable news and all commercial offers. Filtering does not delete a
message, prevent receipt classification, or suppress structured Known Universe
updates; Message Management always permits the retained copy to be found and
the door can revise all five thresholds.

The initial mapping path also dispatches authenticated public notices through
normal public-service envelopes and sealed direct filings through one private
route to Earth. Withholding and secret classification create no mail, and a
committed dispatch cannot be retracted. News/public-service fan-out and initial
message rates are calibration constants pending the full dispatch/tariff model. See
[`non-interactive-universe-tour.md`](non-interactive-universe-tour.md).

The initial player carrier tariff is deliberately provisional: every nonempty
exact-hop bag pays the same Cr100 handling amount plus Cr1 per envelope. Route
activity, scarcity, urgency, and danger never alter that amount. Mail does not
cause, redirect, or economically justify a voyage; the carrier is already
making the transit for some other reason. This is an implemented token payment,
not the final general sender-charge or carrier-accounting model.

## Service Classes and Sender Charges

The initial electronic-data tariff has four classes:

1. **Agency news:** an actual news agency accepts and distributes an item at
   no charge to its source. Editorial acceptance and the agency's scope keep a
   player from labeling arbitrary private traffic as free news.
2. **Broadcast public service:** authenticated mapping announcements, safety
   warnings, key revocations, and other admitted public-service schemas are
   propagated without a sender charge. The server validates the schema and
   authority; “public service” is not a user-selected free-mail flag.
3. **Public-key distribution:** public encryption/signature keys, bindings,
   rotations, and revocations use a constrained free distribution format.
4. **Private or other non-public-service mail:** the sender pays a small
   dispatch charge based on payload class, the purchased TTL, and the actual
   delivery plan. Contents are end-to-end encrypted.

The first three services are free to the sender, not costless to the
simulation. Carrier stipends, institutional subsidies, and relay capacity are
still accounted for by the background mail system.

## Universal Broadcast Completion

Messages explicitly admitted to a universal broadcast service have a
monotonic `universally_seen` state. Here, **seen** means available in every
applicable system or institutional public repository; it does not mean that
every player has read, reviewed, or even displayed the message.

While the message is propagating, the mail system retains its ordinary sparse
route frontier, carrier custody, and exceptional systems still awaiting
delivery. Once every currently applicable repository has received it, one
authoritative transaction sets `universally_seen = true` and deletes the
completed per-system propagation rows. The bit never returns to false.

Later discovery does not reopen old propagation. Public mail follows system
discovery automatically: the discovery/bootstrap package establishes the new
repository at the current completed universal-broadcast checkpoint, including
access to the immutable universal archive. A message already marked universal
is therefore implicitly available in a system discovered later. Messages that
are still propagating when a new system joins the known mail graph add that
system to their live frontier normally.

This rule applies only to messages whose validated propagation policy is
universal. Regional news, polity-scoped notices, contracts, private mail,
mobile hold-sphere replicas, and local messages retain their actual scoped
delivery state. Expiry may stop an incomplete broadcast from ever becoming
universal, but it does not erase its immutable archive or completed custody
history.

The exact credit tariff remains a balance decision. A dispatch quote must be
deterministic from the sender's current delayed route knowledge and disclose
the absolute expiry, covered routes or systems, number of paid branch copies,
and maximum charge before submission. Later delivery failure does not turn a
free quote into a debt or silently expand the purchased scope.

## Fixed-System Addressing

A private message addressed to a system follows one exact known route selected
at dispatch. The sender pays the per-hop amount only for that route, adjusted
for the chosen TTL and payload class. Relays may batch it into ordinary
mailbags, but may not branch the logical message into unpaid alternate routes.

If the selected route cannot complete before expiry, the quote must warn or
reject it. A later disruption may stall or expire the message. Rerouting beyond
the purchased plan requires an authorized contingency in the original quote
or a new dispatch; it is not silently billed after the fact.

## Mobile Addressees

A captain, ship, or other mobile identity has no instantaneous location. Paid
mobile mail therefore purchases an encrypted **hold sphere** rather than an
exact route to a current position.

The dispatch plan fans out over the known mail graph to every system in the
purchased jump sphere. The sender pays for the whole planned fan-out and the
chosen TTL, not merely the branch on which delivery eventually occurs. Each
covered system retains the encrypted message until:

- the intended identity authenticates locally and accepts it;
- an authenticated delivery/cancellation receipt later reaches that system;
  or
- the message expires.

There is no instantaneous cancellation of other replicas. A recipient may
therefore encounter another copy before its first delivery receipt propagates.
Stable message IDs make receipt idempotent, and only the first valid delivery
can trigger a contract, payment, or other authoritative side effect.

The initial service uses the purchased TTL weeks as the maximum known route
hop count for the hold sphere. Its quote sums the charged hops to every covered
system and applies the ordinary per-KiB, per-hop, per-TTL-week tariff. TTL
always remains elapsed game time. The jump sphere describes spatial
replication, not a redefinition of the first “T”.

## Dispatch and Persistent Records

Sending is one authoritative transaction. The server quotes against the
sender's available route knowledge and a versioned tariff, then atomically
debits the quoted charge and creates the immutable message and delivery plan.
Insufficient funds or a stale quote rejects the whole dispatch. The initial
policy charges for purchased service rather than successful delivery; any
refund or carrier-fault rule must be an explicit later tariff term.

The persistent shape needs at least:

- `Message`: immutable content or ciphertext, service class, addresses,
  signatures, key IDs, dispatch time, and `expires_at`;
- `DispatchCharge`: tariff revision, quoted scope, charged credits, payer,
  and payment transaction;
- `DeliveryPlan`: exact fixed-system route or mobile hold-sphere fan-out;
- `CarrierLeg`: custody, carrier, origin, destination, due time, and receipt;
- `HoldReplica`: system, availability time, expiry, and local delivery state;
- `DeliveryReceipt`: message ID, recipient identity, accepting system and
  time, used to suppress later duplicate effects; and
- for universal broadcasts, the monotonic `universally_seen` bit plus sparse
  live propagation state only until that bit is set; and
- public key, private credential custody, compromise, replacement, and
  revocation records described below.

Ciphertext remains immutable across replicas. Routing and receipt records
refer to it rather than copying authoritative message state into every row.

## Encryption and Identity Keys

Private and non-public-service payloads are encrypted to the addressee's
published encryption key. Senders sign with their own signing credential when
authentication matters. Relays need routing headers, message IDs, expiry, and
key identifiers but do not receive plaintext merely to carry the message.

Public keys are intentionally public and their distribution is free. The
capturable gameplay assets are the corresponding **private** signing and
decryption keys, key stores, tokens, recovery material, and unlocked sessions
carried by a captain or ship. Capturing a public key alone grants no ability to
decrypt or impersonate anyone.

Encryption and signing uses should have distinct key IDs even if an initial
implementation stores them in one protected credential bundle. A public key
record contains at least:

- subject identity and key ID;
- encryption or signing purpose;
- public key and algorithm suite;
- issuer or identity-binding signatures;
- issue, validity, and optional supersession times; and
- references to any revocation.

Private credential state records its physical custodian and compromise status.
It must never be included in a public message, general Known Universe merge,
command line, environment variable, or ordinary log.

## Capture, Rotation, and Revocation

Boarding, piracy, theft, surrender, or capture may expose a ship or captain's
private credential material if it was not destroyed, locked, or otherwise
protected. A captured decryption key can reveal addressed ciphertext still
available to the captor and any archived ciphertext obtained with it. A
captured signing key can impersonate the subject until recipients learn of its
revocation. Encryption does not erase this operational risk.

When the compromise becomes known to a competent issuer or law-enforcement
authority, it emits a signed public-key revocation as free public-key/public-
service traffic. Revocation propagates through the same store-and-forward mail. Each system,
bank, beacon, or authority stops trusting the key only when it receives a
valid revocation under its local policy; frontier systems may accept the
compromised credential for weeks or months longer.

The legitimate subject may establish a replacement key locally with an
appropriate issuer. Its new public binding propagates the same way. Relays
cannot decrypt an old message and re-encrypt it to the replacement key, so
undelivered ciphertext for a revoked key normally expires or must be resent.
Revocation cannot retract plaintext already read or signatures already relied
upon before the receiving institution learned of the compromise.

High-value banking, naval, and administrator identities may use separate
credentials, threshold authorization, offline recovery keys, or tamper-
responsive stores. Those protections reduce capture probability; they do not
create instantaneous revocation.

## Discovery Notifications

A mapping-disclosure prompt offers two send operations:

- **Send Public Notification:** emit a free public-service mapping
  announcement from the ship's current system. Its structured observations
  begin propagating into Known Universe repositories from that point. The
  authenticated package also serves as a Federation bounty filing when it
  eventually reaches Earth.
- **Send Direct Notification:** send an encrypted private filing addressed to
  the Federation discovery office on Earth. It follows one exact paid route
  and requires a TTL sufficient for that route. Intermediate relays do not
  merge its encrypted observations into their Known Universe repositories.

If a direct filing is the first valid notification committed on Earth, Earth
pays the award and originates a free authoritative public-service mapping
announcement. Public propagation therefore begins at Earth at award time, not
at the ship's dispatch point. A rejected, expired, duplicate, or otherwise
non-winning direct filing remains non-public unless its sender separately
authorizes publication.

The other disclosure choices remain **Do Not Send** and **Do Not Send and Mark
Secret**. Neither creates a mail message.

## Settled initial private-message tariff

Private point-to-point mail is charged at Cr1 per started KiB of payload, per
route hop, per started TTL week. Public news, public-service broadcasts, and
public-key distribution remain free. The route and retention period are
quoted before the filing commits; mail traffic never induces a dedicated
carrier voyage.

## Open Decisions

- Select encryption, signature, key-binding, and at-rest key-store algorithms.
- Define capture/destruction checks and the capabilities required to extract a
  protected private key.
- Define issuer hierarchies and local policy for banks, navies, polities,
  ships, captains, and pseudonymous or criminal identities.
- Decide whether a non-winning direct discovery filing may offer an explicit
  “publish anyway” instruction.
