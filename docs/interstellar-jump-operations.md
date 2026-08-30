# Interstellar Jump Operations

Cepheus Trader uses the standard Cepheus Engine Jump drive, not the Clement
Sector Zimm drive. A Jump drive's rating is the maximum distance of one Jump
in parsecs; every normal Jump takes approximately one week. See the CE
[Interstellar Travel](../cepodnew-markdown/06-off-world-travel.md#interstellar-travel)
and [Fuel](../cepodnew-markdown/08-ship-design-and-construction.md#fuel)
rules. Fuel for one leg is:

`jump fuel tons = 0.1 × hull displacement tons × leg distance in parsecs`

A sub-parsec leg still counts as Jump-1 for navigation and fuel. These are CE
rules. The staged empty-space operation below is a game adaptation resolving
details that CE does not state explicitly.

The current implementation resolves each filed leg with assigned watch crew.
An onboard plot is the CE Easy Education/Astrogation task, modified by Jump
number and taking `1D6` kiloseconds; a failed plot is recalculated unless the
captain explicitly authorized proceeding, in which case CE makes the result
an automatic misjump. A fresh commercial tape costs Cr1,000 per Jump number
and is available at Class-D-or-better ports for populated destinations. Jump
initiation uses the CE Average Education/Engineer (Jump) task, takes 10--60
seconds, and contributes its Effect to the Jump Success roll. Drive hits,
unrefined fuel, variable duration, inaccurate emergence, misjump, and the
discordant-transition critical hit are all committed server state.

## Safe Jump locus and minimum approach

A Jump may begin or end only outside the union of the 100-diameter exclusion
volumes of every relevant massive body. The server evaluates that union in
three dimensions from the stars' and bodies' complete orbital elements at the
actual game time; a scalar distance from the primary star is not sufficient
in a multiple-star system.

Cepheus Trader adds an operational safety clearance around an inhabited
primary-world port. Its radius is the distance the particular ship covers in
one-half game day under constant thrust with a midpoint turnover:

`clearance = thrust × (0.5 game day)² / 4`

The clearance therefore scales with acceleration. A higher-thrust ship clears
a larger radius in the same time; it does not compress the mandatory
port-to-safe-locus operation below half a game day. If a physical
100-diameter exclusion requires more distance, ordinary maneuver travel time
applies instead.

For BBS capitals and their two complementary J-1 systems, seed conditioning
also rejects primary-world geometry whose guarded 1G approach exceeds 3.5
game days. This prevents a starting core from acquiring an accidentally
pathological port-to-Jump delay.

## Empty-space staging

A Jump destination does not need to contain a star, planet, beacon, or other
mass. A ship may deliberately Jump to a calculated empty-space staging volume
and make a later Jump from there. This follows naturally from using the
standard Jump drive: there are no Zimm points or Zimm-link restrictions to
make empty space inaccessible.

A double-tanked Jump-1 ship can therefore cover up to two parsecs without
refueling by making two Jump-1 legs. The path must contain a suitable
geometric midpoint no farther than one parsec from either endpoint, but that
midpoint need not be an inhabited or generated stellar system. For example, a
Jump-1 ship can travel from Sol to Alpha Centauri through empty space even
though Sol has no directly reachable external stellar system within one
parsec.

An empty-space midpoint provides no fuel, port, market, mail exchange, traffic
control, rescue service, or ordinary encounter population. It is a navigation
and engineering staging point, not a hidden star system.

## Midpoint turnaround

A planned two-leg Jump-1 journey includes a mandatory one-game-day turnaround
between the two Jumps. It is not treated as two immediately chained drive
firings. During that day the crew:

1. determines the actual breakout position and time;
2. scans the staging volume and confirms that the ship is safe;
3. inspects the Jump drive, hull grid, power plant, and relevant damage;
4. resolves any Jump intrusion or engineering problem;
5. corrects the second plot from the actual position and time; and
6. proceeds, delays, aborts, or calculates a replacement plot as conditions
   require.

With CE's `148 + 6D6` hours per normal Jump, a double-Jump journey takes
`320 + 12D6` hours including the turnaround: approximately 13.8–16.3 days,
with a mean just over 15 days.

The midpoint day consumes ordinary power-plant endurance, life support, crew
time, and contract time. It does not consume another allocation of Jump fuel.

## Double-Jump course tapes

Starports and navigation services may sell a **double-Jump tape** for a
specific origin, empty-space staging volume, destination, and scheduled
departure window. It is a coordinated package containing:

- a complete first-leg Jump-1 plot;
- a time-indexed second-leg Jump-1 solution for the expected turnaround
  window; and
- correction data allowing the ship's computer and navigator to adapt that
  second solution to the actual first-leg breakout within the tape's stated
  staging envelope.

The tape does not turn a Jump-1 drive into a Jump-2 drive. It supplies two
separate Jump-1 plots and both legs are resolved separately. If the first
breakout falls outside the correction envelope, the second plot cannot be
used as supplied and the navigator must calculate a replacement. Drive,
grid, power, or other relevant trouble found at midpoint can likewise delay
or abort the second leg.

Double-Jump tapes are strongly departure-time-sensitive. Each leg's age and
other modifiers are evaluated at that leg's actual firing time against the
epoch for which its plot was prepared. The second solution is intentionally
prepared for use after the first Jump and the one-day turnaround; it is not
automatically considered eight days old merely because it was purchased at
the origin. Delaying outside the prepared window makes the tape stale.

Under the CE course-tape price of Cr1,000 per Jump number, two Jump-1 plots
have a baseline price of Cr2,000, the same nominal plot price as one Jump-2
plot. Whether the specialized correction package carries an additional
service premium remains open and must not be silently added to catalog or
game data.

## Reliability

The two legs have independent Jump initiation and success resolution. Apply
the relevant fuel, damage, plot-age, engineering, and other modifiers to each
leg at its own firing time.

If a single leg has probability `p` of clean success, both legs complete
cleanly with probability:

`p²`

The probability that at least one leg has a problem is:

`1 − p²`

The one-day turnaround reduces the danger of carrying an unresolved first-leg
problem into the second firing, but it does not remove the extra statistical
exposure inherent in making two Jumps. This is why a fresh double-Jump tape is
valuable and a stale one is normally unattractive.

## Ship-design and economic comparison

Two Jump-1 legs and one Jump-2 leg both consume 20% of hull displacement in
Jump fuel over the completed two-parsec journey. A ship carrying both
Jump-1 allocations therefore needs the same Jump-fuel tank volume as a
Jump-2 ship carrying one maximum-range allocation.

The Jump-1 installation is nevertheless cheaper and generally smaller. The
Jump-2 design may also require a larger power plant, more power-plant fuel,
and higher-capacity Jump Control software. The exact difference depends on
hull size and on whether the maneuver drive already requires the larger power
plant.

For a 300-ton ship whose maneuver drive already requires a C power plant:

| Arrangement | J-drive | J-drive volume | J-drive price | Jump fuel |
| --- | --- | ---: | ---: | ---: |
| Two Jump-1 legs | B | 15 tons | MCr20 | 60 tons |
| One Jump-2 leg | C | 20 tons | MCr30 | 60 tons |

In that case, double-tanked Jump-1 saves five displacement tons and MCr10 in
drive cost, plus the small Jump Control software difference. Jump-2 instead
buys roughly half the travel time, one Jump exposure rather than two, direct
course-tape operation, and greater route flexibility.

This is an intentional capacity/capital-versus-time/reliability trade-off.
Jump-1 remains a valid interstellar design even where the direct stellar
neighbor graph has no one-parsec edge. Jump-2 is faster and operationally
safer, but is not a universal minimum drive rating.

## Starting-offer boundary

The existence of staged Jump-1 travel removes “cannot leave the home system”
as a reason to prohibit Jump-1 starting ships. The current starting-offer
review nevertheless selected ships fitted and fueled for at least one Jump-2
transit: the Hudson and Crusoe were rebuilt as J-2 designs, while the other
25 offers already selected J-2 ships. A later Jump-1 starter remains possible,
but its drive rating, fuel endurance, route preview, contract timing, and
expected progression must be reviewed together.

The UI and server route planner must distinguish:

- direct range, based on the installed Jump rating;
- staged range, based on drive rating, carried fuel, power/life-support
  endurance, and permitted turnaround time; and
- serviced routes, where fuel, tapes, mail, markets, or rescue are actually
available.

A filed charted leg names both a destination system and its first local
destination. Standard emergence is the safe 100-diameter-clear point nearest
that selected port or body; private emergence adds a seeded offset and greater
standoff. Breakout is followed by an ordinary bounded-thrust interplanetary
maneuver. Subsequent named body or port stops remain separate Flight Plan
steps, and the route may then continue with another Jump.

The Known Universe course plotter exposes two ship-specific projections. A
fastest course minimizes elapsed game time; a cheapest course minimizes
modeled purchased-fuel credits and then time. Both honor installed Jump range,
current or assumed tank loads, primary-port refined- and unrefined-fuel
availability, and gas-giant skimming only when the ship has scoops and
processing capacity. Importing a course materializes its required port
purchases or frontier-fuel operations as executable Flight Plan steps. The
cost figure remains explicitly narrower than total operating cost until crew,
maintenance, fees, risk, and encounters have authoritative price models.

Passenger, freight, mail, and contract rewards apply to the promised journey,
not to an arbitrary count of empty-space legs. A captain cannot double a fare
merely by inserting an unnecessary midpoint.

## Remaining decisions

- exact departure windows and stale-tape thresholds;
- any price premium for a double-Jump correction package;
- whether a midpoint turnaround is normally automatic, interactive, or
  configurable by standing orders;
- the consequences of a first-leg inaccurate Jump that remains recoverable
  but misses the tape envelope; and
- the navigation boundary between Jump and maneuver travel for extremely
  close stellar companions.
