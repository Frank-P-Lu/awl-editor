# Shared orchestration board

Read this guide before each dispatch wave. [queue.md](queue.md) is the only writable
source of work, dependencies, and status; [ROADMAP.md](../ROADMAP.md) holds product
direction. Edit `.orchestrator`, preserving active entries across tools and worktrees.
`.claude/orchestrator` and `.codex/orchestrator` are compatibility symlinks.

## Choose the work

Read the whole item and its landing notes, inspect the named symbols, and check
`git log --grep` for its number before dispatch. Reconcile stale or contradictory
notes against the tree. Verify context claims as well as the reported defect;
mark unverified claims explicitly. Correct false premises in place and record
“premise false, oracle repaired” when the product needed no fix.

Prefer one owner from diagnosis through implementation, tests, mutation proof,
and commit. Implement directly when a brief would cost more than the change.
Partition parallel work by mechanism: shared files do not justify duplicate
behavior owners. Hub files are integration points, not automatic serialization
boundaries. Keep necessary shared-owner changes reviewable in separate commits.

A brief states the defect, constraints, and proof in roughly 150–250 words. Check
that its permitted scope can achieve its deliverable. If constraints conflict,
resolve them against the task's purpose and current repository policy; identify
any necessary incursion and isolate it for review. Genuine product conflicts go
back to the user.

## Claims and the board

1. Re-read the current board immediately before assigning a number or editing it.
   Preserve the earlier committed number if concurrent assignments collide;
   renumber the later item and its cross-references.
2. Claim before implementation with
   `🟡 IN PROGRESS — <owner> (codex|claude|human), branch <name>`.
   Commit the claim before code, as required by [CLAUDE.md](../CLAUDE.md).
   Workers use the named durable worktree; only orchestrators write the board,
   from the main working tree.
3. Edit anchored blocks, never rewrite the board wholesale. Inspect the complete
   diff, item lines, and every `##` heading after each write. Check both heading
   names and count; a restored region must match its original bytes, not merely
   contain its heading.
4. Reclaim claims older than about a day only after checking branch activity;
   leave a takeover note. Confirm an agent and its children have stopped before
   replacing it or deleting its worktree.
5. Landing notes are one or two sentences plus the sha, including any false
   premise or material surprise. Batch routine landing/status notes with related
   work when possible; do not turn the board into a second changelog.

## Dispatch and reports

Read [worker operations](operations.md) before concurrent builds, process recovery,
or cleanup. Every dispatched worker receives an explicit model and effort from
[model routing](models.md); inheritance requires a deliberate rationale in the brief.
State the build wrapper, check scope, and a timeout appropriate to the tool.

Workers normally run targeted checks. [Verification policy](../docs/verification.md)
owns scope; a worker full gate needs an identified risk reason. Give independent
review a concrete subject the gate cannot settle: ownership, mutation validity,
unsupported claims, data loss/security, or subtle geometry. An identity-preserving
refactor still needs an outcome audit. The reviewer reports findings; the owner
repairs. Repeated review rounds on routine work call for revisiting the brief.

A worker's final message contains the complete report: commit, premise check,
changes, checks and scope, capture arithmetic, mutation failure evidence, gaps,
and anything owed to the user. Commentary is not a substitute. Commit before
pausing or launching a final gate. Prepare findings before that launch; if a gate
remains running, still deliver the full report with its receipt marked outstanding.
Wait through the tool's supported mechanism, then append the result. Do not rely
on a self-armed monitor to resume the worker.

Before saying nothing remains, check the session against every required audit
trigger in the verification policy, including visual smoke. An empty board does
not discharge standing obligations.

## Gates and landing

Read [landing and CI](landing.md) before integrating branches, checking receipts,
pushing, or diagnosing CI. Integrate and inspect branches sequentially, then commit
and freeze one combined candidate for the full required checks. Never mutate that
candidate or its HEAD during the gate. Push in a separate tool call after reading
the authorizing results. Tags and releases always require the user's explicit word.

## Taste calls

The user's default is to land measured, cheap-to-revert changes on main for feedback.
State in the commit that the change is for judgment and give the concrete revert
cost. Keep the `🔵 OWED` entry: landing does not settle the taste question.

This does not authorize moving user data, rewriting documents, outward publishing,
tags/releases, unmeasured changes, or changes whose cost lies in late discovery
(such as a consumed schema or a default that silently rewrites files).

## Design sessions

Brainstorm read-only. Each resolved outcome becomes a self-contained queue item
committed with an `orchestrator: decisions` subject; git is the record, not a
separate decisions file. Technical constraints belong in their feature docs;
product and taste laws belong in the contract docs.

For a user-only decision, record the exact question, options with measured
tradeoffs, and a recommendation in a blocked `🔵` item. Keep unrelated work moving
and leave the candidate ready for the answer. Agents may read the user's private
notes when directed; they never write there.
