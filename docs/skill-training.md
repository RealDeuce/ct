# Skill Training and Advancement

**Status:** initial assignments, weekly calendar accrual, and course completion
implemented; advanced training resources and interruption rules deferred.

This document records how skill improvement works at player creation and what
remains to be designed for later progress.

## Rules Authority

Cepheus Trader uses the core Cepheus Engine elapsed-training rule as its
baseline:

- a person's **Skill Total** is the sum of all positive skill levels;
- level-0 skills add zero to the Skill Total;
- training a skill to a desired level requires a number of game weeks equal
  to the person's current Skill Total plus the desired level;
- a person may train only one skill in a given game week; and
- Jack of All Trades cannot be learned through training.

Skill levels are gained in sequence. A person first becomes trained at level
0, then advances to level 1, then level 2, and so forth. Training never skips
an intervening level.

The phrase "game week" is important. Training is measured in authoritative
simulation time, not wall-clock time and not the number or duration of BBS
logins.

The CE passage does **not** say that training consumes a watch, requires full
rest, or stops whenever a character performs ordinary work. It does not ask
the table to account for each hour of study. “A character may only train one
skill in a given week” limits parallel subjects; it is not a full-time labor
allocation rule.

## Examples

The revision-6 default Starting Captain has three level-2 skills and six
level-1 skills. Its Skill Total is therefore 12; the three level-0 skills do
not increase it.

- Learning a previously untrained skill at level 0 takes 12 game weeks.
- Raising a level-1 skill to level 2 takes 14 game weeks.
- Raising a level-2 skill to level 3 takes 15 game weeks.

Each initial crew role template has two level-2 skills and four level-1
skills, giving a Skill Total of 8.

- Learning a previously untrained skill at level 0 takes 8 game weeks.
- Raising a level-1 skill to level 2 takes 10 game weeks.
- Raising a level-2 skill to level 3 takes 11 game weeks.

Jack of All Trades contributes its positive level to Skill Total even though
it cannot itself be learned or improved through ordinary training. The
default captain's Jack of All Trades 2 therefore makes every later course two
weeks longer than it would otherwise be.

## Implemented Initial Assignment

Every newly created captain and named initial crew member begins with one
training target. The target must be one of that person's existing skills and
cannot be Jack of All Trades. This creation-time restriction avoids resolving
the still-open zero-level/unskilled case.

The default is the primary skill for the role. The Starting Captain defaults
to Leadership; each fixed crew template defaults to the first, principal
skill in its role package. The door permits choosing another eligible skill.

Each persisted person carries:

- the targeted skill;
- `needed_weeks`, derived from the CE formula; and
- `current_weeks`, initialized to zero.

For the default level-2 targets, these are 15 weeks for the captain and 11
weeks for qualified crew. The server recalculates and validates the captain's
duration. Crew creation submits only the target skill; the server reconstructs
the fixed package and calculates the duration itself.

## Special Cases

### Jack of All Trades

Jack of All Trades cannot be acquired or increased through ordinary
post-creation training. Its creation-time restriction remains separate: it
may occupy only a level-1 or level-2 Starting Captain slot. No fixed initial
crew template contains Jack of All Trades.

### Cascade Skills

Cepheus Trader currently represents the mechanically relevant cascade
specialties as distinct identifiers. Training applies to the selected
specialty, not to every member of its source cascade. Each positive
specialization level contributes separately to Skill Total.

### Training from Unskilled to Level 0

Applying the CE formula literally makes the time for level 0 equal to the
current Skill Total. For a person whose Skill Total is zero, this produces a
zero-week result. The source does not state a minimum duration. Cepheus
Trader must choose a minimum before permitting an unskilled target; no minimum
is adopted by this document. Initial creation therefore limits targets to
skills already present on the person.

### Maximum Skill Level

The CE training passage does not state a general maximum skill level.
Cepheus Trader has not yet adopted a cap, a diminishing-return rule beyond
the growing Skill Total, or eligibility requirements for high levels.
Starting characters no longer receive Skill-3, but that creation decision is
not itself a post-creation cap.

## Clement Advancement Rules Not Adopted

The older *Clement Sector Third Edition* character rules provide a different
advancement economy:

- Adventure Points are awarded at the end of an adventure and spent on skill
  levels or characteristics;
- Success Points are earned from exceptional task successes and apply to the
  skill used; and
- Instruction can accelerate advancement through checks by a teacher and
  student.

Adventure Points do not map cleanly to a persistent game without discrete
RPG adventures. Success Points would reward repeatedly manufacturing task
checks and would make anti-grinding rules part of every simulator action.
Neither point economy is adopted for Cepheus Trader.

The current *Clement Sector Core Rulebook* audit did not locate a replacement
general game-play advancement procedure. Instruction remains a possible later
addition, but the Third Edition procedure
produces points from its own advancement system and therefore cannot be
copied directly onto the CE elapsed-week formula. Any instructor, school, or
training-facility bonus must be designed explicitly before it is used.

## Calendar Accrual Boundary

Training is deliberately coarse. A person assigned a valid training target
accrues at most one training week per completed authoritative game week.
Normal watches, Jump, interplanetary travel, port calls, routine work, and
brief encounters do not require subtractions from that week. Training is not
conditioned on the Crew manager's off-watch flag.

Only a state that makes training genuinely impossible for a material period
should interrupt it—for example death, prolonged unconsciousness, cold sleep,
or an explicitly modeled full-time course/activity that says it is mutually
exclusive. Injury and combat do not retroactively erase an otherwise valid
week merely because they occupied part of it. The implementation should use
weekly boundaries, not accumulate minute-level “study time.”

Crew Management's off-watch state remains relevant to full rest, watch
coverage, and natural healing. It is not the source of training eligibility.

The scheduler uses completed seven-day boundaries measured from the course's
start or last target change. A valid course advances by one week regardless
of watch assignment. At `needed_weeks`, the server atomically increments the
selected rating and stops that course pending a new target. Changing the
target resets progress and schedules a fresh first week. On database open,
missing weekly events are reconstructed from the person records.

The following policy questions remain for later training expansion:

- how a partial week is treated after a genuinely modeled interruption;
- whether self-study requires software, course material, facilities,
  tuition, an instructor, or a minimum EDU;
- whether some technical, medical, military, or high-level skills require
  certification or institutional access;
- whether several people aboard one ship may train concurrently and what
  shared resource limits apply;
- whether training orders can be queued and whether a player must confirm
  advancement when a course completes;
- whether captains may direct the training of crew they employ but do not
  own as game assets;
- whether a maximum level is required for long-running universes; and
- whether the nominal CE durations produce acceptable real-world progression
  under Cepheus Trader's compressed Jump-time calendar.

The last item is especially important. At four game weeks per real day, an
uninterrupted 12-week course finishes in roughly three real days. That result
comes from the chosen universe time rate and the CE duration, not from watches.
If it is too fast, the training economy or global pacing needs an explicit
balance rule; a hidden daily-activity ledger is not an appropriate throttle.

## Persistence and Future Transaction Boundary

The authoritative person record now carries the target skill, required weeks,
and current weeks. The starting and desired levels are derivable from the
person's current rating because initial assignments always improve the target
by exactly one level.

Course progress and completion are ordered engine transactions. Each event
revalidates the person and target, commits the new counter or rating, and
schedules at most one successor. A completed course has no successor until
the captain selects another target.
