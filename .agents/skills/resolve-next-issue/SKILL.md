---
name: resolve-next-issue
description: Select the most impactful actionable open Cepheus Trader GitHub issue, investigate it against the repository, and produce an implementation plan for approval. Use when asked to choose what CT issue to work on next, plan and implement the highest-impact issue, continue an approved issue plan, revise the resulting uncommitted implementation from review feedback, or commit and push an explicitly accepted resolution.
---

# Resolve the Next Cepheus Trader Issue

Run one issue through a gated lifecycle:

```text
select and investigate -> propose plan -> wait for approval
-> implement and test -> leave uncommitted -> wait for review
-> revise or, on explicit acceptance, commit and push
```

Never collapse the approval gates. Optimize for player impact and correctness,
not for closing the easiest issue.

## Preserve the workspace

1. Read root `LLM_INSTRUCTIONS.md` and inspect `git status --short` before
   selecting work.
2. Preserve all existing changes. If the worktree contains unrelated changes
   that make isolation or final review unsafe, stop and ask the user to resolve
   them. Do not reset, stash, overwrite, or absorb them.
3. Confirm the `RealDeuce/ct` remote and current branch. Do not fetch, pull,
   switch branches, create a branch, or mutate GitHub during selection unless
   requested.
4. Treat issue selection and planning as read-only except for the ignored
   issue-evaluation cache described below. Do not edit tracked repository files
   before plan approval.

## Reuse local issue evaluations

Use `.agents/cache/resolve-next-issue.json` as a disposable, Git-ignored cache
of issue-content analysis. Never stage or commit it.

1. Fetch the current open-issue index with number, title, `updated_at`, labels,
   and assignees. GitHub remains authoritative for whether an issue is open.
2. Load the cache only when `schema_version` is `1` and `repository` is
   `RealDeuce/ct`. Ignore a missing or malformed cache without blocking work.
3. Reuse a cached issue summary and judgment only when its `updated_at` exactly
   matches GitHub. Fetch and fully evaluate every new or changed issue.
4. Remove entries absent from the current open-issue index. Record, for each
   remaining issue, its metadata, concise summary, classification/disposition,
   impact judgment, dependencies or duplicates, known mechanism, code evidence,
   and the commit inspected during the last code investigation.
5. Refresh the cache after evaluating changed issues and after an accepted
   resolution is pushed. Mark an implemented-but-still-open issue with its
   resolution commit so it is not selected again accidentally.
6. Treat cached code findings as leads, not current truth. Reinspect the current
   repository for every shortlisted issue before proposing a plan, even when
   its GitHub content is unchanged.

Cache maintenance is the only write allowed during selection. It must not
alter `git status`, substitute for fetching the current open-issue index, or
weaken the approval gates.

## Phase 1: Select the issue

Fetch all open issues, excluding pull requests. For new or changed candidates,
read the body, labels, comments, linked issues, and supplied attachments. For
unchanged candidates, use the matching cached issue-content analysis. Do not
rely on the title or label alone.

Exclude issues that are:

- duplicates, invalid, or wontfix;
- already assigned to active work or superseded by a newer canonical issue;
- blocked on unavailable user evidence or a prerequisite not in scope;
- primarily an unanswered question; or
- too ambiguous to plan without first obtaining clarification.

Rank the remaining issues with this priority order:

1. **Severity of player harm:** crash, data loss/corruption, inability to enter
   or continue the game, or a broken authoritative rule outranks inconvenience.
2. **Breadth and frequency:** a core-loop or all-player failure outranks a rare
   career, platform, facility, or presentation edge case.
3. **Economic and authority integrity:** incorrect balances, permissions,
   contracts, persistence, or simulation outcomes outrank missing explanatory
   copy.
4. **Field-alpha urgency:** failures preventing useful real-world testing or
   making the game misleading outrank cosmetic polish.
5. **Confidence and actionability:** prefer a code-grounded, testable issue over
   an equally severe report whose mechanism remains unknown.
6. **Dependency value:** prefer work that safely unblocks several legitimate
   reports, but do not bundle those reports without approval.

Use effort only as a tie-breaker between similarly impactful issues. Do not
select a small visual fix over a confirmed accounting defect merely because it
is faster.

## Investigate before planning

Shortlist the strongest candidates, then inspect enough code and documentation
to validate their relative impact and feasibility:

- locate the exact client surface, server rule, protocol field, persistence
  path, tests, and design text with `rg`;
- inspect attachments rather than asking the user to restate visible evidence;
- trace CT-RPC and client/server ownership when applicable;
- distinguish intended but unclear behavior from a defect;
- identify compatibility, migration, release, economy, information-delay,
  career, and supported-terminal consequences; and
- verify that the proposed fix can be tested without requiring unavailable
  production state.

Do not implement a probe or edit code during this phase. Read-only commands are
allowed. If no issue is sufficiently actionable, report why and wait rather
than manufacturing a plan.

## Produce the approval plan

Present one selected issue with a direct GitHub link. Include:

- why it is the most impactful choice;
- the next one or two contenders and why they rank lower;
- the confirmed or best-supported mechanism;
- the intended player-visible outcome;
- implementation steps with likely files and ownership boundaries;
- focused and regression validation;
- compatibility, persistence, CT-RPC, release, or deployment implications;
- risks and explicit non-goals; and
- any assumption the implementation depends on.

Use a concrete step list suitable for tracking during implementation. End the
turn by asking for approval. Make it explicit that no files were changed and
that implementation will begin only after approval.

If the user changes the plan, revise it and wait for approval again. Do not
treat discussion, partial agreement, or a request for explanation as approval.

## Phase 2: Implement only the approved plan

On clear approval:

1. Re-read the selected issue and check for new comments, closure, or changed
   requirements.
2. Recheck the worktree and ensure the approved issue can still be isolated.
3. Track the approved steps with the task plan mechanism when available.
4. Implement the smallest complete resolution. Follow authoritative ownership:
   keep rules and persistence on the server, presentation in the client, and
   use typed protocol fields across the boundary.
5. Add regression coverage that fails for the reported behavior and proves the
   intended outcome. Cover neighboring invariants likely to regress.
6. Update player, sysop, design, or protocol documentation only when the fix
   changes or clarifies its contract.

Do not opportunistically fix other issues. If investigation reveals that the
approved mechanism or scope is materially wrong, stop, explain the discovery,
and request approval for a revised plan. Minor implementation choices that
preserve the approved outcome do not need a new gate.

## Validate proportionately

Always run focused tests, repository checks, and `git diff --check`. Add the
relevant broader checks:

- **Server rules or persistence:** Rust unit/integration tests, formatting, and
  lint checks appropriate to the changed targets.
- **Door behavior or presentation:** native client build and CTests, supported
  terminal-width coverage, and the real TLS/OpenDoors harness when the flow
  crosses the live protocol.
- **Windows behavior or ABI:** relevant 32-bit and 64-bit cross-builds, PE/ABI
  checks, and Wine smoke or unit tests when available.
- **CT-RPC/schema:** coordinated client/server tests, compatibility/version
  review, and explicit release implications.
- **Player documentation/help:** build the documentation site and verify help
  topic generation.

Do not claim a check passed unless it ran successfully. Explain environmental
or unrelated failures and preserve their output.

## Phase 3: Hand off an uncommitted implementation

After implementation and validation:

- leave every change unstaged and uncommitted;
- do not push, close the issue, or post a GitHub resolution comment;
- inspect the final diff for unrelated edits and accidental generated files;
- summarize the behavior change and important design choices;
- link the principal changed files;
- list tests and their results;
- call out compatibility or release implications and known limitations; and
- state the exact worktree status and that no commit was created.

End by inviting review. State that the user may provide feedback or reply
`Accept` to authorize staging the reviewed files, committing them, and pushing
the current branch. This statement makes a subsequent unqualified `Accept` an
explicit commit-and-push authorization for that reviewed diff.

## Apply review feedback

Treat feedback about the approved issue as continuation of the implementation:

1. inspect the concern against the current diff;
2. make the requested in-scope revision;
3. rerun affected tests and checks;
4. leave the result unstaged and uncommitted; and
5. return another complete review handoff.

If feedback materially expands the issue, changes authoritative behavior, or
adds another issue, propose a plan amendment and wait for approval before
editing that expanded scope.

## Phase 4: Commit and push only after acceptance

Proceed only when the user replies `Accept` after the review handoff or gives
an equally explicit commit-and-push instruction.

1. Confirm that the worktree diff still matches the reviewed implementation
   and contains no unrelated files.
2. Re-run any cheap final integrity check that may have become stale.
3. Stage only the reviewed issue files.
4. Create a concise commit naming the resolved behavior. When the accepted
   implementation completely resolves the selected issue, include `Fixes #N`
   in the commit message so GitHub closes it on the target branch. Use a
   non-closing `Refs #N` only for an explicitly accepted partial resolution or
   when the user asks to keep the issue open.
5. Before pushing, inspect the committed message and verify both the selected
   issue number and the appropriate closing or non-closing keyword. If either
   is wrong and the commit is still local, amend only the message. Never
   rewrite already-pushed history merely to repair issue linkage.
6. Push the current branch.
7. Verify the resulting commit, upstream push, and clean worktree.

Report the commit hash, subject, branch, push result, resulting issue state,
and any remaining local changes. Do not manually close, relabel, or comment on
the GitHub issue unless the user separately authorizes that mutation. If commit
succeeds but push fails, report the local commit and failure accurately rather
than retrying destructively.
