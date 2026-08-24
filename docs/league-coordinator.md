# League Coordinator Guide

A League Coordinator (LC) manages one named group of Cepheus Trader BBSs. It
is a separate authority from the global server administrator and from each BBS
sysop. The LC can rename its League, create member BBS enrollments, and disable
or re-enable those members. It cannot attach an existing BBS, remove or
transfer membership, configure a BBS polity, manage players, or act as another
League.

## Bootstrap

After the universe is initialized, the global administrator creates the
League:

```console
cepheus-trader-admin add-league
```

This prints a numeric League ID and its 32-byte PSK once. Transfer both through
a private channel. On the LC host, enter them interactively to create a
protected credential file; the PSK does not belong in a command argument,
environment variable, configuration file, or log:

```console
cepheus-trader-league init-credential league.credential
```

The file is owner-only on Unix and receives the protected credential ACL on
Windows. Back it up as a secret. The server's dedicated CT-League TLS-PSK
endpoint uses port 7326 by default and can be changed with
`--league-listen HOST:PORT`.

## League operations

Read the current League revision and member revisions before a mutation:

```console
cepheus-trader-league --credential league.credential status
```

Set or change the unique League name with the League revision reported by
status:

```console
cepheus-trader-league --credential league.credential \
  --expected-revision REV set-name "Spinward League"
```

Create a permanent member enrollment and write its normal BBS credential
directly to a protected file:

```console
cepheus-trader-league --credential league.credential \
  --bbs-credential new-bbs.credential add-bbs "Example BBS"
```

Transfer that BBS credential privately to its sysop, who completes ordinary
sysop configuration. Membership is established by the authenticated League
credential; there is no caller-supplied League ID and no later attach, remove,
or transfer command.

Disable or re-enable a member with that member's revision:

```console
cepheus-trader-league --credential league.credential \
  --expected-revision REV disable-bbs BBS_ID "maintenance hold"
cepheus-trader-league --credential league.credential \
  --expected-revision REV enable-bbs BBS_ID
```

Disablement preserves the BBS credential, polity, systems, players, and game
state, but immediately rejects new player and sysop authentication and closes
their active connections. Re-enabling restores access with the same BBS
credential. Disabled BBSs remain League placement anchors.

## Retry and display behavior

Mutations are exactly-once. If a connection fails after a command may have
reached the server, repeat it with the command ID printed by the client:

```console
cepheus-trader-league --credential league.credential \
  --command-id HEX ...
```

Revision mismatches return the current state without applying the mutation.
Refresh status and make a new decision instead of blindly replacing another
LC action.

Once a member BBS has materialized its polity, current player screens display
the affiliation as `Polity (League)`. Historical founding news keeps the name
it recorded when published. The first materialized BBS in a League uses normal
placement; later member BBSs prefer the closest valid placement to an existing
materialized League capital while retaining all universe geometry invariants.
