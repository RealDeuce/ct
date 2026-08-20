# Perry Visual Manifest

*Status: catalog production family 204*

## Identity and architecture

Perry is an independent 300-ton / approximately 4,200 m³ TL11 standard patrol
frigate, 64 × 21 × 15 m. Its Jump-C drive provides Jump-2, while maneuver-F
and power-F machinery provide four-gravity acceleration and four weeks of
endurance. Advanced electronics, Model/3 fib, eight-point crystaliron armor,
and stealth treatment make it a compact naval interceptor rather than a trader.

A broad six-pane armored command prow leads into a deep faceted waist and a
square engineering block. Two long parallel closed shutters and their shared
center rail mark one 20-ton internal hangar, divided into two cells for exactly
two 10-ton Thermopylae fighters. Each cell is derived from the approved
11.8 × 6.4 × 3.2 m Thermopylae master, but both craft remain internal and
invisible in the catalog plate. The hangar is one installed component, not two
external craft clamps. All launch, personnel, service, and cargo doors remain
closed.

Perry must not inherit Decatur's narrower fast-escort proportions. Perry is
broader, centers its fighter hangar in the upper hull, and exposes four rather
than six maneuver apertures. Four power grilles, one standard-hull fuel scoop,
five processor housings, and paired Jump-coil service shoulders distinguish
its machinery without an external glowing ring.

## Hardpoint and firing-arc map

Three trainable hardpoints cover complementary axes rather than forming a
dorsal row:

- the visible forward port/upper-shoulder ring carries one triple beam-laser
  turret with exactly three short optical emitters;
- the visible aft port/lower-flank ring carries one outward-canted triple
  missile turret with exactly three closed rectangular launcher shutters;
- the hidden ventral/starboard ring carries the second triple missile turret.

The standard port three-quarter view therefore shows two of three turrets. The
forward shoulder and lower flank stations overlap across the port, forward,
dorsal, ventral, and aft approaches, while the hidden partner closes the
starboard blind region. No fourth ring, decorative point-defense node, fixed
gun, open missile tube, or visible missile may appear.

Deep Admiralty navy `#203B69`, warm white `#E8E5D4`, signal red `#C43C35`, and
bright aluminum identify Shipyard Path 7. Signal red is restricted to weapon
rings, hangar safety edges, access boundaries, and machinery warnings; bright
metal picks out the bridge frames and structural datum edges.

## Invariants and prompt

Preserve the 64 × 21 × 15 m faceted hull; six bridge panes; broad command
prow; twin closed hangar shutters and center rail; boarding waist; service and
cargo doors; scoop; five processor housings; Jump-service shoulders; four
maneuver apertures; four power grilles; two visible and one hidden hardpoint;
camera; crop; background; and lighting.

```text
Generate the neutral Perry patrol-frigate chassis with the fixed geometry and
three-station coverage map above. Derive the production plate without moving a
coordinate. Fit the visible shoulder triple beam turret and lower-flank triple
missile turret, retain the second missile turret on the hidden ventral/starboard
face, keep both Thermopylae fighters internal behind closed parallel shutters,
and apply Admiralty navy, warm white, signal red, and bright aluminum. No extra
ring, turret, PD node, exposed craft, open door, text, insignia, or drive glow.
```

The neutral anchor initially acquired an accidental third visible ring; it was
removed before production. Production then received focused edits to remove a
reintroduced aft dorsal ring and replace the lower-flank placeholder with an
exactly three-cell missile turret. Artwork was generated for Cepheus Trader on
2026-08-20 with OpenAI image generation in built-in mode, production derived
from the corrected neutral anchor.

| Asset | Purpose | Review status |
| --- | --- | --- |
| `site/ship-art/anchors/family-204-perry.png` | Neutral Perry patrol-frigate chassis | Approved |
| `site/ship-art/masters/ship-204-perry.png` | Archival Perry master | Approved |
| `site/assets/ships/ship-204-perry.webp` | Published Perry plate | Approved |

The artwork is Open Game Content under `LICENSE.md` and
`OPEN_GAME_LICENSE.md`.
