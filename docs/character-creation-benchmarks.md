# Published Character-Creation Benchmarks

**Status:** source analysis, not an adopted point-buy system.

## Sources and Cohorts

This benchmark uses the local copies of:

- *21 Characters: Clement Sector Third Edition*;
- *21 Villains OGL*;
- the nine merchant-crew pregens in *Cascadia Adventures Second Edition*;
- the nine CCA ship-crew pregens in *Hell's Paradise Second Edition*; and
- qualitative checks against the merchant crew in *Long Road to Redemption*,
  the pirate crew in *The Slide*, and the expedition characters in *Grand
  Safari*.

The older books use SOC and older skill names. They are useful measures of
character development, not authority for Cepheus Trader's adopted CHA or for
a particular specialization model. The 21-character collections include
older veterans and exceptional antagonists; the two adventure crews are
closer to immediately playable ordinary ship crews.

## Measured Profiles

“Positive levels” is the sum of levels above zero; level-zero listings do not
add to it. Characteristic totals are the sum of the six listed
characteristics, before translating SOC to CHA.

| Cohort | N | Characteristic total, median | Positive levels, median | Level-zero listings, median | Positive-level range |
| --- | ---: | ---: | ---: | ---: | ---: |
| *Cascadia Adventures* merchant crew | 9 | 47 | 8 | 3 | 6–12 |
| *Hell's Paradise* CCA crew | 9 | 49 | 10 | 2 | 2–14 |
| *21 Characters CS3* | 21 | 54 | 14 | 4 | 5–30 |
| *21 Villains OGL* | 21 | 53 | 16 | 2 | 13–24 |

The very low *Hell's Paradise* endpoint is a new private with two positive
levels and five level-zero competencies. Its captain has 14 positive levels
and three level-zero families. The cohort is therefore a useful example of
rank and experience variation within one functioning crew.

Representative published captains are:

| Character | Context | Positive levels | Level-zero listings |
| --- | --- | ---: | ---: |
| Captain Crawford Zha | Ordinary independent merchant captain | 11 | 3 |
| Captain Ludomir Stanca | Experienced small-service-ship captain | 14 | 3 |
| Hammer Allen | Eight-term successful free trader | 14 | 3 |
| Danko Divac | Very old, highly developed pirate captain | about 31 | 7 |

Danko is an endpoint or major NPC benchmark, not a reasonable new-player
target.

## What Published Development Looks Like

Across the ordinary adventure crews:

- level 2 normally defines the person's principal professional competency;
- level 1 supplies secondary duties and cross-training;
- two or three level-zero competencies provide emergency breadth;
- level 3 marks a notably experienced specialist; and
- a working ship is covered by complementary crew profiles rather than by
  making every person independently capable of every station.

The *21 Characters* collection is broader because its median subject is older
and more developed. Its median of 14 positive levels and four zero-level
listings is a useful experienced-character target, not automatically the
starting-crew budget.

## Effect of the Two Specialization Models

For comparison, define an “acquisition unit” as one career-style skill gain:

- Under Clement base-plus-specialization, acquiring family level 0 costs one
  unit and each positive specialization level costs another.
- Under standard cascade, each separately trained specialty first needs its
  own level 0 and then its positive levels.

Using the published profiles as written gives these approximate medians:

| Cohort | Base-plus-specialization units | Independent-specialty units |
| --- | ---: | ---: |
| *Cascadia Adventures* crew | 18 | 19 |
| *Hell's Paradise* crew | 20 | 21 |
| *21 Characters CS3* | 29 | 29 |

The median cost difference is small because published characters rarely
develop several positive specialties in the same family. The important
difference is instead the scope of family level 0: Electronics 0, for example,
removes the untrained penalty from several electronic tasks at once.

The CS3 characters visibly use that family-familiarity concept. Profiles may
list Electronics 0 alongside Electronics (Sensors) 1, or Gunner 0 alongside
Gunner (Turrets) 1. These are evidence that the broader zero is intentional
in that rules generation, not merely an abbreviation for an unspecified
specialty.

## Point-Buy Implication

If Cepheus Trader adopts base-plus-specialization, a single unrestricted pool
would make broad family-level zero purchases unusually attractive. A cleaner
creation structure would separate:

1. **Familiarity selections**, producing the source-like two to four level-zero
   families; and
2. **Expertise points**, purchasing positive levels in specific
   specializations.

Role packages could provide default familiarity selections while still
allowing customization. This preserves the published pattern—narrow expertise
with modest emergency breadth—without encouraging players to spend every
point buying family-level zero before acquiring a real profession.

If standard cascade is retained instead, comparable breadth must be supplied
through specifically enumerated level-zero specialties, role packages, or
Jack of All Trades. The data does not make that model invalid; it shows that
the starting budget must deliberately replace the broad familiarities present
in the Clement profiles.

Creation uses the customizable **Starting Captain** pools and separate fixed
initial-crew role templates. The captain independently satisfies the
Starting Captain pools; initial crew mechanics are derived from their slots,
not submitted as player selections.

The adopted skill-rating pools follow the measured ordinary profiles:

| Pool | Level +3 | Level +2 | Level +1 | Level +0 | Positive-level total |
| --- | ---: | ---: | ---: | ---: | ---: |
| Starting Captain | 0 | 3 | 6 | 3 | 12 |
| Initial crew role template | 0 | 2 | 4 | 3 | 8 |

These are separate rating slots, not fungible points. Thus a Starting Captain
selects three skills at level 2, six at level 1, and three at level 0. A
fixed initial crew template has two role skills at level 2, four at level 1, and
three at level 0. Published adventure captains are normally broad rather than
Skill-3 specialists; Skill-3 is therefore reserved for progression. The
captain lands within the ordinary-captain range, while a qualified crew
member lands at the lower of the two ordinary adventure-crew medians.

Starting Captain characteristics use a separate point-buy definition:
scores range from 2 through 12, each score costs `score - 7`, and all six
costs must consume an exact budget of 12. This fixes the total at 54
(all 9s by default) while permitting deliberately uneven 6/9/12-style
captains. Initial crew use the fixed `[10,9,8,8,7,6]` multiset, with its
assignment and skill package chosen by role. Better crew are a later hiring
upgrade rather than another starting-character optimization.
