# Clement Zimm Drive to CE Jump Drive Conversion

*Rules audit: 2026-07-26*

Cepheus Trader uses the standard Cepheus Engine Jump drive. Clement Sector
ships use the Zimm drive. Their construction systems are deliberately similar,
so most non-capital Clement ships can be converted without redesigning their
hulls.

This note distinguishes preservation of a published ship from optimization as
a new CE design.

## Non-Capital Ships: 100–2,000 Tons

For ships from 100 through 2,000 tons designed under the current Anderson &
Felix rules, replace the prescribed Zimm-drive code with a CE Jump drive of
the same letter. Every letter in the current Anderson & Felix Zimm Drive
Letter Chart produces Jump-2 when read on the CE Drive Performance by Hull
Volume table.

| Hull tons | Clement Z-drive | Result as CE J-drive | Minimum CE J-2 drive |
| ---: | :---: | :---: | :---: |
| 100 | A | J-2 | A |
| 200 | B | J-2 | B |
| 300 | C | J-2 | C |
| 400 | D | J-2 | D |
| 500 | G | J-2 | E |
| 600 | H | J-2 | F |
| 700 | J | J-2 | G |
| 800 | J | J-2 | G |
| 900 | K | J-2 | H |
| 1,000 | K | J-2 | H |
| 1,200 | L | J-2 | J |
| 1,400 | M | J-2 | K |
| 1,600 | N | J-2 | L |
| 1,800 | P | J-2 | M |
| 2,000 | Q | J-2 | N |

The Anderson & Felix non-capital cost table prints the K Z-drive as 65 tons,
which breaks the otherwise regular sequence by making it larger than the
subsequent 60-ton L drive. Treat this as a source-table error and verify a
specific published K-drive ship before using its total. The corresponding CE
K drive is 55 tons and MCr100.

Through 400 tons, the current chart's Z-drive is also the minimum CE Jump-2
drive. From 500 through 2,000 tons, its selection is normally 10 tons and
MCr20 larger than the minimum CE Jump-2 drive. There are therefore two valid
conversion policies:

- **Preserve the published ship:** relabel the existing drive. The ship remains
  Jump-2 with unchanged displacement, base drive price, and layout. Its drive
  is simply larger than a CE-optimized installation.
- **Optimize as a new CE design:** install the minimum CE Jump-2 drive, recost
  the ship, and decide how to use the recovered displacement. This creates a
  variant and is not a purely editorial conversion.

Some published ship books predate or vary from the current construction chart.
For example, the 550-ton Raptor already uses a type-F Z-drive, which is also
the minimum practical CE Jump-2 installation, while the 600-ton Kiviat uses a
type-H drive and can recover ten tons by changing to type F. Some source
blocks also use improved or nonstandard component tonnages. The actual source
stat block must therefore be compared with the CE minimum; hull size alone
does not prove that ten tons are available.

Starter-catalog conversions use the optimized policy when the source
installation is oversized. The recovered displacement becomes
starter-configurable space. Any drive-cost reduction belongs to package/refit
accounting and is not unrestricted starting cash. Designs without an
oversized source drive need deliberately reserved configurable volume if
their starter package is intended to provide a similar fitting choice.

## Fuel and Computer Conversion

The interstellar fuel formula is identical:

`fuel tons = 0.1 × hull tons × distance in parsecs`

A Clement ship carrying fuel for one two-parsec transition already carries
the correct fuel for one CE Jump-2. Under CE, the same tank can instead support
two Jump-1 jumps if power-plant endurance permits. Cepheus Trader permits the
two legs to use an empty-space midpoint and requires a one-game-day turnaround
before the second firing; see
[`interstellar-jump-operations.md`](interstellar-jump-operations.md).

Clement Zimm Control and CE Jump Control/2 both require computer rating 10 and
cost MCr0.2. Rename Zimm Control to Jump Control/2. A `bis` computer
specialized for Zimm Control can likewise become a `bis` computer specialized
for Jump Control without changing its cost. Low-TL ships may rely on imported
higher-TL Jump Control software, as CE's own TL9 Courier does.

Rename the relevant Engineer specialty from Zimm Drive to Jump Drive.

## Zimm-Only Equipment and Cost

A military-grade Zimm drive costs an additional 50% and reduces the Zimm
recharge time from eight hours to five. Standard CE Jump drives have no
equivalent grade or recharge rule. Remove that surcharge when converting a
published ship unless Cepheus Trader separately defines and prices a useful
CE ruggedized-drive option. Recalculate the published ship total rather than
trusting a line item that may contain a source arithmetic error.

Zimm emitter-node and Z-bubble damage rules become CE Jump-grid, Jump-drive
hit, plotting, and misjump rules. They do not coexist.

## Capital Ships: Above 2,000 Tons

Direct relabeling stops being reliable above 2,000 tons. Anderson & Felix
switches to a capital Z-drive equal to 3% of hull displacement at MCr2 per
drive ton. CE continues using its lettered Jump-drive table. These produce
different sizes and prices:

| Hull tons | Clement capital Z-drive | Minimum core CE J-2 drive |
| ---: | ---: | ---: |
| 3,000 | 90 tons, MCr180 | T: 95 tons, MCr180 |
| 4,000 | 120 tons, MCr240 | Y: 120 tons, MCr230 |
| 5,000 | 150 tons, MCr300, but Z-bubble prohibited | Z: 125 tons, MCr240 |

Intermediate sizes require an explicit construction ruling because the core
CE performance table lists only selected hull volumes. All ships over 2,000
tons should therefore be recalculated as CE designs rather than relabeled.

Clement's additional size-failure rule begins above 2,000 tons, not above
3,000 tons. The prose requires a percentage roll for vessels larger than
2,000 tons; its table gives collapse chances from 2% at 2,000–2,500 tons
through 25% at 4,501–4,999 tons. A vessel of 5,000 tons or more cannot form a
Z-bubble. The inclusion of exactly 2,000 and 5,000 tons differs slightly
between the prose and tables, but that boundary ambiguity does not affect the
conversion: CE Jump uses its own failure rules and has no Z-bubble size roll.

## Operational Rules Do Not Convert by Relabeling

The hardware compatibility does not make Zimm and Jump travel operationally
equivalent:

| Property | Clement Zimm drive | CE Jump drive |
| --- | --- | --- |
| Interstellar route | Specific outgoing point linked to a specific incoming point | Any plotted destination in range from beyond the 100-diameter limit |
| Nominal range | At most two parsecs using available Zimm-point links | Rating of installed drive; converted source codes yield Jump-2 |
| Transit time | 3.5 days per parsec | Approximately one week regardless of distance |
| Sub-parsec/in-system use | Seconds or minutes with negligible distance-scaled fuel | Approximately one week and at least Jump-1 fuel |
| Reuse | Eight-hour recharge; five hours for military grade | No corresponding recharge rule |
| Failure | Entry/exit checks, emitter damage, bubble collapse, and large-hull risk | Plot, power-diversion, Jump-success, inaccurate-Jump, and misjump rules |

Consequently:

- do not import a Clement route map or Zimm Point;
- do not allow essentially free instantaneous in-system skip transits;
- use CE's 100-diameter limit, plotting, transit time, and failure rules;
- remove Zimm recharge assumptions from schedules; and
- recompute any narrative timetable that depended on a one-parsec trip taking
  only 3.5 days.

For a non-capital ship stat block, the physical conversion is normally a
relabel plus the Zimm-only cleanup above. For its behavior in the game, all
travel is governed by CE Jump rules.
