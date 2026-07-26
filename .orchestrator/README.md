# Shared orchestration board

`queue.md` is the one source of truth for work, dependencies, and status.
`ROADMAP.md` holds product direction. Do not create a second writable board.

`.claude/orchestrator` and `.codex/orchestrator` are compatibility symlinks.
Always edit `.orchestrator`, and preserve active entries across tools and
worktrees.

## Ownership and claims

Only the top-level orchestrator edits `.orchestrator`, and only from the main
working tree. Workers return commits and outcomes; their briefs never ask them
to edit the board, merge or push main, or resolve user gates.

1. **Claim before code.** Mark the item
   `🟡 IN PROGRESS — <owner> (codex|claude|human), branch <name>` and commit the
   claim to main before dispatch.
2. **Use a worktree.** Branch from local `main`; keep the main tree for the
   human session and merge gates.
3. **Reread before dispatch.** Check `queue.md` at HEAD immediately before
   starting work.
4. **Land through gates.** Merge to local main only after the required suite,
   then mark `✅ LANDED @ <sha>` in the same session.
5. **Record routing.** The landing note gets one compact phase line, for example
   `Routing: plan=Sol high; build=Terra medium; verify=Terra medium`. Git and the
   item narrative remain the outcome and repair record; do not add a scorecard.
6. **Delegate conflicts.** A merge worker reconciles conflicts. The orchestrator
   does not hand-edit conflict markers.
7. **Reclaim stale work.** A claim older than about a day with no branch
   activity may be reclaimed with a takeover note.

## Design sessions

- Brainstorm read-only. Resolve ambiguity before turning a decision into work.
- Put each decided outcome in a self-contained queue item.
- Commit decision updates with a subject beginning `orchestrator: decisions`.
  Git history is the session record; do not add a decisions log.
- Put actionable work in `queue.md`, standing technical constraints in
  `CLAUDE.md` or the matching doc, and product or taste laws in the contract
  docs.
- Agents may read the user's private notes when directed; they never write
  there.
- Do not create a parallel reports archive.

## Queue execution

- Run disjoint work concurrently in isolated worktrees. Keep roughly three to
  four active workstreams; sequence genuine file clashes.
- Integrate branches into main one at a time through the full native and wasm
  gates, plus both keymap conventions when relevant.
- If integration fails, remove only that integration from main, preserve
  unrelated work, flag the item, and continue independent work.
- Park user-only decisions in the queue with the exact question, options, and a
  recommendation. Mark the item blocked, do not prompt during a cook, and do
  not idle unrelated work.

## Workflows

Use the smallest bounded workflow that proves the outcome:

- **Routine change:** implement, then independently verify.
- **Unknown cause:** test competing hypotheses, select from evidence, fix, then
  audit the neighborhood.
- **State or ownership migration:** map owners and invariants, implement, then
  verify adversarially.
- **Visual or taste work:** generate real captures, have a judge choose against
  the decided rubric, then let an implementer apply and verify the verdict.
- **Broad audit:** enumerate cells, run bounded probes, synthesize, and stop at
  green, no new findings, or no progress.

Give each agent one role and a fresh context. State the stop condition and
repair or token budget. Normally use fewer than five agents inside an item and
keep the machine-wide workstream cap. Evidence may add repair, re-verification,
or merge roles, but never silently expand product scope. Do not save one-off
workflow scripts.

The orchestrator owns `queue.md`, local-main integration, pushes, remote CI,
and user gates. Workers commit and report from their worktrees.

## Push trains and remote CI

1. Push after two or three locally green build items. Queue- or docs-only
   commits ride the next train or the end of the session. Push immediately for
   a CI repair, serious correctness or data-loss fix, requested checkpoint,
   handoff, or release preparation.
2. Run one remote train at a time. Wait for the resulting non-cancelled `main`
   CI run before integrating the next train; workers may keep cooking.
3. A failed `main` run creates or updates the top-priority `CI RED` queue item
   with the run URL, failure, and first known bad commit. Assign a production
   worker immediately and block further main integration until the fix is
   remotely green.
4. Before dispatch, after compaction or handoff, and before a status answer,
   reconcile the board with worktrees, local ahead/dirty state, and the latest
   non-cancelled main CI result. Keep transient counts, timings, and usage out
   of the board.

## Execution hygiene

- **Use durable worktrees.** Never put them under `/private/tmp`. Commit work in
  progress before a pause.
- **One worktree per concurrent writer.** Shared worktrees are reader-only.
- **Clean up landed worktrees promptly.** Remove clean, merged worktrees and
  prune registrations. Do not touch dirty, locked, or differently owned
  worktrees without handoff. `git worktree remove` removes the checkout, not
  the branch.
- **Establish input modality before diagnosis.** If a report does not identify
  keyboard, pointer, or wheel input, disambiguate that first.
- **Preserve gate truth.** Do not pipe gates through commands that hide their
  exit status. Run the required wasm gate.
- **Classify suspicious failures before blaming code.** Retry incremental
  failures with `CARGO_INCREMENTAL=0`. For `SIGKILL` without a test failure,
  check memory and rerun the gate alone.
- **Terminate only owned processes.** Never kill `awl` by process name; stop
  only the exact PID created by the current run.

## Delegation boundary

- **The orchestrator does not write production code.** It delegates
  implementation and merge reconciliation; it retains reading, planning,
  specs, and board updates.
- **Delegate real diagnosis.** Reading needed to write a brief is orchestration.
  Reproduction, measurement, candidate elimination, and root-cause work belong
  in a `diagnose → fix → verify` workflow.
- **Real builds use workflows.** Sequential work is sequential phases in one
  workflow. A minor rename, lookup, or other bounded one-off may use one agent.

## Model routing and budget

Set an explicit model and effort for every worker. Inheritance is valid only
when the brief records why the same model and effort fit the role. Record the
runtime, tier, and launcher substitutions in the landing routing line.

- **Production:** current Sonnet at medium, or `gpt-5.6-terra` at `medium`, for
  implementation, structured research, routine diagnosis, merges, and standing
  audits. Raise effort only when evidence exposes deeper ambiguity.
- **Deep planning:** current Opus, or `gpt-5.6-sol` at `high`, for ownership or
  state maps, ambiguous high-value work, and targeted adversarial verification
  while hidden-risk conditions remain. Use `xhigh` when several plausible
  candidates must be eliminated; reserve `max` for rare indivisible problems.
- **Repeatable work:** `gpt-5.6-luna` at `low` or `medium` only for bounded
  extraction, classification, mechanical transformation, log triage, roster
  enumeration, or deterministic probes with an explicit oracle.
- **Visual judge:** Fable, or provisionally `gpt-5.6-sol` at `high` or `xhigh`,
  receives real captures and returns a verdict only. The implementer applies it
  and owns gates. Keep the OpenAI route provisional until a representative
  gallery comparison validates it.

Use full model IDs. Do not use `ultra` inside an orchestrated workflow because
it adds its own delegation; use `max` for deeper single-agent reasoning.

Choose tiers by failure cost and evidence, not task size or prestige. Give each
worker one bounded brief; avoid duplicate scouts and inherited unrelated
context. When weekly usage is more than about ten percentage points ahead of
the reset cycle, conserve compute per task without reducing independent
throughput: keep the production default, reserve deep tiers for qualifying
risk, and route clear repeatable work cheaply. Defer only optional work
deliberately moved beyond the reset.
