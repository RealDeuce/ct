---
name: triage
description: Review and triage Cepheus Trader GitHub issue batches or imported player-feedback reports against the current repository. Use when asked to inspect recent, unlabeled, or specified CT issues; distinguish bugs, enhancements, documentation gaps, questions, and duplicates; apply requested labels; inspect attachments and code; or post concise evidence-backed maintainer follow-ups.
---

# Cepheus Trader Issue Triage

Triage reported behavior against the actual CT implementation before
classifying it. Preserve useful player observations while turning each issue
into an accurate, actionable public record.

## Respect the request boundary

- Treat review-only requests as read-only.
- Apply labels or post comments only when the user requests those mutations.
- Do not edit issue titles or bodies, assign people, add milestones or project
  fields, close issues, change code, or open pull requests unless explicitly
  requested.
- Label and cross-reference exact duplicates, but leave them open unless the
  user authorizes closing them.
- Stop before publicly discussing a suspected security vulnerability, secret,
  credential, private identity, or exploitable operational detail. Report the
  need for private handling to the user.

## Establish the batch

1. Confirm that the repository is `RealDeuce/ct` from the current Git remote,
   unless the user names another fork.
2. Prefer authenticated GitHub MCP or API access. Use `gh api` when no suitable
   GitHub connector is available.
3. Fetch current repository labels and their descriptions before classifying.
   Reuse the existing taxonomy; do not invent component or priority labels
   without authorization.
4. Use an explicit issue list or range when supplied. Otherwise identify the
   smallest credible batch, normally the newest open, unlabeled issues created
   together by the requesting maintainer. Exclude pull requests.
5. Read every issue body, existing label, status, and comment before acting.
   Do not repeat a conclusion or question already posted.

Do not silently sweep unrelated older issues merely because they are open or
unlabeled. State the inferred batch in the final report.

## Build evidence

For each issue:

1. Identify the reported screen, operation, stored state, and desired outcome.
2. Inspect every supplied image or other attachment. Download transient files
   to `/tmp` and view them when the API does not expose their contents.
3. Search the repository with `rg` for exact copy, screen renderer,
   transaction, protocol fields, rules, tests, and design documentation.
4. Trace behavior across client and server boundaries when relevant. Check
   whether an existing observation contains the information needed or whether
   CT-RPC, persistence, or an authoritative server rule must change.
5. Compare implemented behavior with documented intent. Separate an intended
   but poorly explained rule from an implementation defect.
6. Look across the whole batch for shared root causes and duplicates. One
   defect can explain one report while neighboring reports remain valid UI or
   documentation improvements.

Do not modify code while triaging unless the user separately asks for a fix.
Read-only diagnostics and targeted test inspection are appropriate.

## Use the CT evidence map

- Read root `LLM_INSTRUCTIONS.md` and the directly relevant design document
  before asserting intended behavior.
- Door presentation and player flows normally live in `client/src/door_main.cpp`
  and `client/src/door_help.cpp`; typed observations and transactions live in
  `client/include/ct/protocol.hpp` and `client/src/protocol.cpp`.
- Authoritative rules and persistence normally live under `server/src/`, with
  transaction and snapshot construction concentrated in `server/src/store.rs`
  and wire conversion in `server/src/wire.rs`.
- Treat server state as authoritative. Do not infer that missing door output or
  missing server log lines prove a connection did or did not occur.
- Check player-facing layout at supported terminal widths, especially 40x24.
  A screenshot showing merged columns, clipped choices, or an inaccessible
  return path is evidence of a presentation bug, not merely aesthetics.
- Distinguish game timestamps from scaled wall-time waits and identify whether
  the relevant snapshot carries current game time and clock-rate fields before
  proposing a client-only countdown.
- Trace liquid, restricted operating, reserved, secured, and institutional
  funds separately. Authorized ship expenses normally use the operating-account
  charging path; a direct liquid-credit deduction can be a career-specific bug.
- Keep routine upkeep, proper repair, refit, and refurbishment distinct.
  Routine upkeep is scheduled ship-wide accounting; a refit is a separate
  weeks-long overhaul and does not automatically replace destroyed systems.
- Check career authority explicitly. Trader, privateer, and navy commands do
  not necessarily have the same access to cargo, passengers, commercial Tasks,
  operating funds, orders, or ship ownership.
- Respect CT information delay. Do not propose omniscient remote knowledge or
  describe stale carried information as current server truth.
- For Windows crashes, use WER exception/module/offset data, build identity,
  symbols, and dumps when supplied. Do not assume the OpenDoors door prints a
  useful stderr stream or that the remote server logged the client failure.

## Classify conservatively

Use the current repository label definitions. With the conventional labels:

- **bug**: implementation contradicts its intended rule, loses or corrupts
  state, crashes, miscalculates, or renders incorrectly. Establish a concrete
  mismatch; player surprise alone is not proof.
- **enhancement**: behavior works as designed, but the requested presentation,
  workflow, information, or capability would improve it.
- **documentation**: correcting or adding documentation is the primary remedy.
  Combine with another label only when the repository normally does so.
- **question**: essential reproduction facts or intended behavior remain
  genuinely unresolved after inspecting code, attachments, and comments.
- **duplicate**: another issue asks for substantively the same outcome or
  tracks the same actionable defect. Choose the clearer or earlier canonical
  issue and name it in a comment.
- **invalid** or **wontfix**: use only when repository policy or an explicit
  maintainer decision supports it, never as a substitute for uncertainty.

Do not stack `bug` and `enhancement` merely to hedge. When a confusing report
reveals both a defect and a usability gap, identify the defect explicitly and
keep distinct actionable issues distinct.

## Plan mutations before applying them

Create an internal matrix with one row per issue:

```text
issue | classification | evidence | relation | label change | follow-up
```

Check that:

- every proposed label exists;
- unrelated existing labels will be preserved;
- each duplicate names one canonical issue;
- public assertions are supported by inspected code or documented behavior;
- comments add information instead of acknowledging an already complete
  report; and
- no comment promises a schedule, assignee, release, or fix not committed.

## Write useful public follow-ups

Post a comment only when it contributes at least one of these:

- a confirmed mechanism or root cause;
- a distinction between expected behavior and a defect;
- the exact current rule when the interface obscures it;
- a client, server, protocol, or data boundary that affects implementation;
- a canonical duplicate reference;
- a precise remaining question that cannot be answered locally; or
- a narrowed acceptance criterion.

Keep comments concise and player-respectful. Lead with the conclusion, explain
the evidence in plain language, and state the issue scope. Avoid internal local
paths, speculative fixes, canned thanks, blame, and unnecessary implementation
jargon. Do not claim reproduction or confirmation when only a hypothesis was
established.

Useful patterns:

```text
Confirmed as a bug in the accounting path. [Observed mismatch and impact].
This issue should cover [bounded remedy or acceptance behavior].
```

```text
This is the same requested outcome as #N. Consolidating the work there and
marking this report as a duplicate.
```

```text
The current rule is [behavior]. The interface exposes [partial fact] but does
not explain [missing contract]. Treating this as an enhancement to [surface].
```

## Apply and verify

1. Apply requested labels without removing unrelated labels.
2. Post each planned comment once. Re-read immediately before posting if the
   issue could have changed during analysis.
3. Read the batch back from GitHub and verify labels, comment counts or URLs,
   status, and duplicate references.
4. Report:
   - the issue range reviewed;
   - classifications grouped by label with links;
   - which issues received follow-ups;
   - the most important confirmed finding;
   - duplicates left open or closed;
   - any issue intentionally skipped or still requiring user input; and
   - whether local files were changed.

If a GitHub write fails, report the exact issues affected and preserve the
verified successful actions. Do not imply that a partial batch completed.
