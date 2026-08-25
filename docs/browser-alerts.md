# Browser alerts and the portable communicator

Cepheus Trader can pair up to five browsers with a captain and send Web Push
alerts while the captain is away. The companion page is deliberately narrow:
in-world it is the captain's portable communicator; operationally it enrolls
a browser, edits receiving preferences, displays authenticated alert detail,
lists unexpired alerts successfully delivered to the current receiver, and
unlinks that receiver. It never accepts game orders.

The alert service currently produces three independently selectable classes:

- an advance bridge-watch reminder for a scheduled leg that will finish at a
  `Hold` waypoint and wait for the captain;
- an advance standing-orders reminder for a scheduled leg that will reach a
  `Through` waypoint and act for the captain;
- an immediate call when an unacknowledged checkpoint becomes visible or the
  captain detects an encounter. Hidden engine knowledge does not originate an
  encounter alert.

Alert text may contain the same game information that the captain can see.
Opening the companion is optional: the push title and body carry the useful
headline, such as “Captain to the bridge! An armed ship is moving to
intercept!” The detail link is for the fuller player-visible record.

## Components

The authoritative server owns a dedicated `ct-web-push` thread and a bounded
queue. It stores alerts and per-subscription delivery attempts in a shared
SQLite database, encrypts standards-based Web Push payloads, and retries
temporary push-service failures. Network delivery never runs on the
authoritative engine thread and never changes a game deadline or outcome.

The Synchronet SSJS application under `synchronet-web/ct-alerts/` uses the same
SQLite database for enrollment and preferences. It is written for
Synchronet's SpiderMonkey 1.8.5 runtime and must be checked with Synchronet's
`jsexec`, not Node.js. The Synchronet binaries must be built with the `SQLite`
JavaScript binding and `sha256_calc()` global enabled; verify both with the same
build used by the web server:

```console
SBBSCTRL=/srv/sbbs/ctrl /srv/sbbs/exec/jsexec -C -r \
  'writeln(js.version); writeln(typeof SQLite); writeln(typeof sha256_calc);'
```

The last two lines must both be `function`. No semaphore event or separately
scheduled Synchronet worker is needed.

## Server setup

Generate the stable VAPID private key once. Protect it like an application
credential and include it in backups:

```console
cepheus-trader-server \
  --init-web-push-key /srv/cepheus-trader/secrets/web-push-vapid.key
```

The command prints the corresponding public key and exits. Then start the
normal server with all four Web Push settings:

```console
cepheus-trader-server \
  --web-push-url https://bbs.example/ct-alerts/ \
  --web-push-database /srv/cepheus-trader/data/web-push.sqlite3 \
  --web-push-vapid-key /srv/cepheus-trader/secrets/web-push-vapid.key \
  --web-push-vapid-subject mailto:sysop@example.com
```

The public URL must be HTTPS, must not contain user information, query, or
fragment, and must end in `/`. All four options are required together;
omitting all four disables the feature. The server creates the database and
records the public URL and VAPID public key for the companion.

## Synchronet web setup

Copy the contents of `synchronet-web/ct-alerts/` to the directory served as
`https://bbs.example/ct-alerts/`. Keep the helper and `.ssjs` endpoints beside
`index.ssjs`; `service-worker.js` must remain at this path so its scope covers
the communicator.

Copy `synchronet-web/cepheus-trader-web-push.ini.example` to Synchronet's
control directory as `cepheus-trader-web-push.ini` and set:

```ini
database=/srv/cepheus-trader/data/web-push.sqlite3
```

This must be the same absolute path supplied to `--web-push-database`. The
Synchronet web-server account and Cepheus Trader server account both need read
and write access to the database directory, database, WAL, and shared-memory
files. The INI file belongs outside the web root and contains no VAPID private
key.

Web Push and service workers require a secure browser context. Serve the whole
communicator over HTTPS and do not redirect it to another origin. If a reverse
proxy is used, preserve the configured origin exactly.

## Pairing and use

The player opens the universal menu and chooses `Browser Alerts`. The door
requests a ten-minute, single-use pairing address and prints the complete URL
as its OSC 8 link text. It also renders a QR code when the terminal is wide
enough:

- CP437 at exactly 40 columns uses square full cells;
- CP437 above 40 columns packs two vertical modules into upper/lower
  half-blocks;
- ISO 646 at exactly 40 columns uses one `M` per square module;
- ISO 646 above 40 columns uses two horizontal cells per square module.

The URL fragment carries the pairing token, so it is not sent in the HTTP
request target or ordinary access logs. The browser requests notification
permission only after the player presses **Link this communicator**. Once
linked, the companion exposes independent toggles for advance Hold warnings,
advance Through/standing-orders warnings, and attention-now calls, plus one
1–1440 minute lead time shared by both advance-warning classes. This lets a
captain disable routine Through notices while retaining every warning that
means the ship will wait for orders.

The communicator's Received Transmissions section is a temporary receiver
inbox. It lists only alerts that the push service accepted for that linked
browser, and each entry opens the same authenticated detail as its system
notification. Entries disappear under the alert's existing expiry rule; this
is recovery from a dismissed notification, not a permanent message archive.

## Security and failure behavior

- Pairing tokens are stored only as SHA-256 hashes, expire after ten minutes,
  and are consumed transactionally.
- The browser creates a 256-bit device credential. Only its hash is stored;
  the credential is returned as a host-only `Secure`, `HttpOnly`,
  `SameSite=Strict` cookie.
- State-changing SSJS endpoints require a same-origin POST, bounded JSON, bound
  SQLite parameters, and the device credential where applicable.
- Alert detail references are random. The detail endpoint checks the linked
  browser session, captain identity, and expiry before returning data. The
  service worker opens the reference in a URL fragment.
- The temporary inbox requires the receiver credential and joins delivery rows
  to that browser session, captain identity, successful state, and alert
  expiry. Revoking the receiver invalidates that credential; a push-service
  revocation does not prematurely hide alerts it had already accepted. The
  inbox does not expose pending, failed, or other-receiver alerts.
- Push subscription endpoints must be plain HTTPS URLs. Delivery disables
  redirects, resolves and pins a public address, and rejects loopback, private,
  link-local, multicast, and other non-public ranges.
- A `404` or `410` from a push service revokes that subscription. `429` and
  server errors receive bounded exponential retries. Push is advisory:
  delivery failure never delays the game or suppresses standing orders.

The door can revoke every receiver for the captain. The web companion can
unlink only its own receiver. Enrollment stops at five active receivers.
