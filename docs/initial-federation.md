# Initial Federation Universe

The first universe is created explicitly by the global administrator:

```console
client/build/cepheus-trader-admin initialize-universe
```

The utility warns that this is destructive and requires the exact phrase
`INITIALIZE FEDERATION` on standard input before it connects to the server.
The command atomically removes players, sessions, queued player commands,
outboxes, systems, worlds, ships, markets, messages, and other game/simulation
state. It preserves BBS enrollment, BBS credentials, and sysop-selected BBS
configuration because those are installation and control-plane state.
Connected player sessions are disconnected after the reset commits.

Initialization creates the Federation polity, the 35 stellar-component
systems from Sol through Tau Ceti, and Earth at TL13. Every initial system is
a Federation member and receives an independently generated cryptographic
seed for later planetary-system generation. Only Earth is materialized as a
world by this slice.

## Astrometric model

The source positions are from the [Fifth Catalogue of Nearby Stars
(CNS5)](https://dc.g-vo.org/cns5/q/cone/info), described by [Golovin et al.,
2023](https://arxiv.org/abs/2211.01449). Right ascension, declination, and
parallax are transformed into heliocentric Galactic Cartesian coordinates:

- positive X is coreward;
- positive Y is spinward;
- positive Z is Galactic north; and
- all three coordinates and distances are in parsecs.

A game system represents one stellar component because each component may
have its own generated planetary system. This gives 35 systems rather than
22 gravitational groupings. When CNS5 supplies only a combined position for
a close multiple, its components initially share that interstellar-map
position. Local orbital separation belongs to the later generated
planetary-system model. Component positions with separate CNS5 astrometry use
the primary component's parallax to avoid turning small measurement
differences into false interstellar separation.

The component split is:

| Gravitational grouping | Game systems |
| --- | --- |
| Alpha Centauri | Alpha Centauri A, Alpha Centauri B, Proxima Centauri |
| Luhman 16 | Luhman 16 A, Luhman 16 B |
| Sirius | Sirius A, Sirius B |
| Luyten 726-8 | Luyten 726-8 A, Luyten 726-8 B |
| EZ Aquarii | EZ Aquarii A, EZ Aquarii B, EZ Aquarii C |
| 61 Cygni | 61 Cygni A, 61 Cygni B |
| Procyon | Procyon A, Procyon B |
| Struve 2398 | Struve 2398 A, Struve 2398 B |
| Groombridge 34 | Groombridge 34 A, Groombridge 34 B |
| Epsilon Indi | Epsilon Indi A, Epsilon Indi Ba, Epsilon Indi Bb |

Luhman 16 A/B and Epsilon Indi Ba/Bb are brown dwarfs.
WISE 0855-0714 is a sub-brown-dwarf object and is retained as a system for
the same gameplay reason: substellar systems may still provide useful local
bodies and skimming opportunities.

## Jump-2 neighborhood

The table lists every other initial system no farther than 2.000 parsecs.
Distances are Euclidean distances in the stored Galactic frame. Values shown
as `<0.001` are close companions whose separation is below the displayed
precision; they are not intended to occupy literally the same local orbital
position.

| ID | System | Other systems within Jump-2 range (parsecs) |
| ---: | --- | --- |
| 1 | Sol | Alpha Centauri A (1.325), Alpha Centauri B (1.325), Proxima Centauri (1.302), Barnard's Star (1.828), Luhman 16 A (1.998), Luhman 16 B (1.998) |
| 2 | Alpha Centauri A | Sol (1.325), Alpha Centauri B (<0.001), Proxima Centauri (0.055), Barnard's Star (1.980), Luhman 16 A (1.102), Luhman 16 B (1.102) |
| 3 | Alpha Centauri B | Sol (1.325), Alpha Centauri A (<0.001), Proxima Centauri (0.055), Barnard's Star (1.980), Luhman 16 A (1.102), Luhman 16 B (1.102) |
| 4 | Proxima Centauri | Sol (1.302), Alpha Centauri A (0.055), Alpha Centauri B (0.055), Luhman 16 A (1.079), Luhman 16 B (1.079) |
| 5 | Barnard's Star | Sol (1.828), Alpha Centauri A (1.980), Alpha Centauri B (1.980), Ross 154 (1.702) |
| 6 | Luhman 16 A | Sol (1.998), Alpha Centauri A (1.102), Alpha Centauri B (1.102), Proxima Centauri (1.079), Luhman 16 B (<0.001), WISE 0855-0714 (1.876) |
| 7 | Luhman 16 B | Sol (1.998), Alpha Centauri A (1.102), Alpha Centauri B (1.102), Proxima Centauri (1.079), Luhman 16 A (<0.001), WISE 0855-0714 (1.876) |
| 8 | WISE 0855-0714 | Luhman 16 A (1.876), Luhman 16 B (1.876), Wolf 359 (1.353), Sirius A (1.443), Sirius B (1.443), Procyon A (1.663), Procyon B (1.663) |
| 9 | Wolf 359 | WISE 0855-0714 (1.353), Lalande 21185 (1.247), Ross 128 (1.196) |
| 10 | Lalande 21185 | Wolf 359 (1.247) |
| 11 | Sirius A | WISE 0855-0714 (1.443), Sirius B (<0.001), Procyon A (1.613), Procyon B (1.613) |
| 12 | Sirius B | WISE 0855-0714 (1.443), Sirius A (<0.001), Procyon A (1.614), Procyon B (1.614) |
| 13 | Luyten 726-8 A | Luyten 726-8 B (<0.001), Epsilon Eridani (1.559), Tau Ceti (0.956) |
| 14 | Luyten 726-8 B | Luyten 726-8 A (<0.001), Epsilon Eridani (1.559), Tau Ceti (0.956) |
| 15 | Ross 154 | Barnard's Star (1.702) |
| 16 | Ross 248 | 61 Cygni A (1.712), 61 Cygni B (1.711), Groombridge 34 A (0.556), Groombridge 34 B (0.557) |
| 17 | Epsilon Eridani | Luyten 726-8 A (1.559), Luyten 726-8 B (1.559), Tau Ceti (1.675) |
| 18 | Lacaille 9352 | EZ Aquarii A (1.251), EZ Aquarii B (1.251), EZ Aquarii C (1.251), Epsilon Indi A (1.448), Epsilon Indi Ba (1.445), Epsilon Indi Bb (1.445) |
| 19 | Ross 128 | Wolf 359 (1.196) |
| 20 | EZ Aquarii A | Lacaille 9352 (1.251), EZ Aquarii B (<0.001), EZ Aquarii C (<0.001) |
| 21 | EZ Aquarii B | Lacaille 9352 (1.251), EZ Aquarii A (<0.001), EZ Aquarii C (<0.001) |
| 22 | EZ Aquarii C | Lacaille 9352 (1.251), EZ Aquarii A (<0.001), EZ Aquarii B (<0.001) |
| 23 | 61 Cygni A | Ross 248 (1.712), 61 Cygni B (0.001), Struve 2398 A (1.865), Struve 2398 B (1.865) |
| 24 | 61 Cygni B | Ross 248 (1.711), 61 Cygni A (0.001), Struve 2398 A (1.866), Struve 2398 B (1.865) |
| 25 | Procyon A | WISE 0855-0714 (1.663), Sirius A (1.613), Sirius B (1.614), Procyon B (<0.001), DX Cancri (1.518) |
| 26 | Procyon B | WISE 0855-0714 (1.663), Sirius A (1.613), Sirius B (1.614), Procyon A (<0.001), DX Cancri (1.518) |
| 27 | Struve 2398 A | 61 Cygni A (1.865), 61 Cygni B (1.866), Struve 2398 B (<0.001) |
| 28 | Struve 2398 B | 61 Cygni A (1.865), 61 Cygni B (1.865), Struve 2398 A (<0.001) |
| 29 | Groombridge 34 A | Ross 248 (0.556), Groombridge 34 B (0.001) |
| 30 | Groombridge 34 B | Ross 248 (0.557), Groombridge 34 A (0.001) |
| 31 | DX Cancri | Procyon A (1.518), Procyon B (1.518) |
| 32 | Epsilon Indi A | Lacaille 9352 (1.448), Epsilon Indi Ba (0.007), Epsilon Indi Bb (0.007) |
| 33 | Epsilon Indi Ba | Lacaille 9352 (1.445), Epsilon Indi A (0.007), Epsilon Indi Bb (<0.001) |
| 34 | Epsilon Indi Bb | Lacaille 9352 (1.445), Epsilon Indi A (0.007), Epsilon Indi Ba (<0.001) |
| 35 | Tau Ceti | Luyten 726-8 A (0.956), Luyten 726-8 B (0.956), Epsilon Eridani (1.675) |

There are 65 undirected component-to-component pairs within Jump-2 range.
This is a geometric range audit, not yet a rule that requires or permits a
Jump transition between extremely close companions; that boundary belongs to
the later navigation and local-orbital model.

Direct-neighbor geometry is not a hard limit on staged travel. Standard Jump
drives may target empty-space volumes, so a double-tanked Jump-1 ship can
cross a two-parsec gap in two legs with a required one-day midpoint
turnaround. The operational, tape, risk, and economic rules are in
[`interstellar-jump-operations.md`](interstellar-jump-operations.md).
