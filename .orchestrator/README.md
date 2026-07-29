# Shared orchestration board

`queue.md` is the one source of truth for work, dependencies, and status.
`ROADMAP.md` holds product direction. Do not create a second writable board.

`.claude/orchestrator` and `.codex/orchestrator` are compatibility symlinks.
Always edit `.orchestrator`, and preserve active entries across tools and
worktrees.

## The default: one owner, end to end

One capable agent owns an item from diagnosis through implementation, tests,
mutation proof, and commit. Splitting an item across a planner, an
implementer, and a verifier costs three contexts and buys, in practice, one
handoff bug. Prefer more context in one head.

The orchestrator may implement directly when writing the brief would cost more
than the change. A bounded, already-diagnosed fix is not worth a dispatch.

Give every worker an explicit model and effort chosen for the role. Inheritance
is valid only when the brief records why the same model and effort fit.

## Mutation proof is part of the deliverable

**A law that was never watched failing is not evidence.** The author breaks the
product, watches the law go red by name, restores, and pastes the actual panic
text into the report. This is the single cheapest defect-catcher available and
it belongs to the author, not to a reviewer.

The axis a law sweeps must be the one the author did not think of. A law that
checks only imagined cases goes green over a real defect. Sweep the roster,
the whole geometry range, the empirical worst case.

An audit that finds something ends by writing the missing law.

## Independent verification: only where a gate cannot see

The suite, both conventions, wasm, and the health ratchet already answer "does
it behave". Do not pay an agent to re-ask that. Spend a second pair of eyes
only where a green test can still be wrong:

- **Is this the right owner?** Ownership and state migrations, cache identity.
- **Does this law actually fail on its bug?** Harness and oracle work.
- **Is this claim true?** Any assertion the gates do not mechanically check —
  measurements, "identical output", "no other call sites", perf numbers.
- Data loss, security, and subtle rendering geometry.

A byte-identity refactor still needs an outcome audit: identity preserves
pre-existing bugs.

When a verifier rejects, it gets one pass and the owner repairs. Three rounds
on a routine refactor means the brief was wrong, not the code.

## Claims and the board

1. **Claim before code.** Mark the item
   `🟡 IN PROGRESS — <owner> (codex|claude|human), branch <name>`, then dispatch.
2. **Fold board edits into the work commit.** A claim or a landing note is not
   its own commit — board-only commits otherwise outnumber the work they
   track. Batch board state with the code it describes, or with the next
   commit that touches anything.
3. **Keep items short.** A brief states the defect, the constraint, and how to
   prove it — target 150–250 words. Length is a symptom of handing off to a
   stranger; prefer keeping the owner.
4. **Landing notes are one or two sentences plus the sha.** Record a surprise if
   the item's premise was wrong — that is what a later reader needs. Do not
   restate the work; `git log -p` has it.
5. **Reread at HEAD before every board write, and diff before committing.**
   Two orchestrators share this file, and a scripted rewrite drops items
   without saying so. Confirm the diff touches the lines you intended and no
   others — `git diff` the `^\d+\.` lines specifically. Never rewrite
   `queue.md` wholesale; it will silently take the other tool's claims with
   it.
6. **Reclaim stale claims** older than about a day with no branch activity,
   with a takeover note.

Only the orchestrator writes the board, from the main working tree. Workers
report shas and outcomes.

## Gates and landing

Run the cheapest thing that can falsify the current claim; run the full set
once, at landing.

- **While working:** rustfmt, the narrowest affected clippy/health arm, the
  targeted tests. Expected to run often, so keep it small.
- **Before landing, on the exact combined-main candidate:** code health, the
  full native suite under **both** conventions, wasm smoke, and the item's
  required captures. Repair failures on the candidate, rerun the failed slice,
  then the full set once.
- **Push after two or three landed items.** Board- or docs-only changes ride
  the next train. Push immediately for a CI repair, a correctness or data-loss
  fix, or a requested checkpoint.
- **Check `main`'s CI before pushing and after** —
  `gh run list --branch main --limit 1`. A green local train says nothing about
  the remote. While `main` is red, the repair is the only thing that ships.
- **Keep the local toolchain level with CI's** — `rustup check`. CI tracks
  floating stable; a stale local clippy cannot see the lint it is pushing.

Integrate one branch at a time. Two branches each green alone can be red
together — a roster or ownership law is designed to cause exactly that. For
structs with per-call-site initializers, grep the construction sites before
declaring a merge done: git merges a missing field cleanly and fails to
compile later.

A failed `main` CI run becomes the top-priority `CI RED` item with the run URL
and first known bad commit, and blocks further integration.

**Tags and releases wait for the user's explicit word, every time.**

## Non-negotiable operational facts

- **Preserve gate truth.** Never pipe a gate through something that hides its
  exit status. Always run the wasm gate: a change can look native-only and
  still break it.
- **Never claim a tier that did not run**, and never hide a failure by
  selecting a smaller one. Formatting-, docs-, and board-only changes use only
  the applicable arms.
- **Durable worktrees, never under `/private/tmp`.** One worktree per
  concurrent writer; shared worktrees are reader-only. Commit before pausing,
  so a stalled agent never holds the only copy of its work.
- **Clean up merged worktrees, then run `scripts/sweep.sh`.** Cargo never
  collects superseded artifacts, so `target/` grows without bound into tens of
  gigabytes. `git worktree remove` removes the checkout, not the branch.
- **Classify suspicious failures before blaming code.** Retry incremental
  failures with `CARGO_INCREMENTAL=0`. For `SIGKILL` with no test failure,
  check memory and rerun the gate alone.
- **Terminate only owned processes.** Never kill `awl` by name; stop only the
  exact PID this run created. Identify them with `pgrep -f` plus `ps -ww`:
  macOS `ps -o command=` truncates before arguments like `--user-data-dir`, so
  it reports a confident "0 orphans" while orphans are running. Other agents
  work in this repo concurrently — target windows by unique title, never by
  process name.
- **Establish input modality before diagnosis.** If a report does not say
  keyboard, pointer, or wheel, disambiguate first.

## Design sessions

- Brainstorm read-only. Resolve ambiguity before turning a decision into work.
- Each decided outcome becomes a self-contained queue item, committed with an
  `orchestrator: decisions` subject. Git is the record; no decisions file.
- Actionable work goes in `queue.md`, standing technical constraints in
  `CLAUDE.md` or the matching doc, product and taste laws in the contract docs.
- Park a user-only decision in the queue with the exact question, the options,
  and a recommendation. Mark it blocked and keep unrelated work moving.
- Agents may read the user's private notes when directed; they never write
  there.

## Model routing

- **Production** — current Sonnet at medium, or `gpt-5.6-terra` at `medium`:
  implementation, structured research, routine diagnosis, merges, audits.
- **Deep** — current Opus, or `gpt-5.6-sol` at `high`: ownership and state
  maps, ambiguous high-value work, adversarial verification. `xhigh` when
  several plausible candidates must be eliminated. `xhigh` is the ceiling;
  do not dispatch workers at `max`.
- **Repeatable** — `gpt-5.6-luna` at `low`/`medium`: bounded extraction,
  classification, mechanical transformation, deterministic probes.
- **Visual judge** — Fable, or `gpt-5.6-sol` at `xhigh`: receives real
  captures and returns a verdict only; the implementer applies it and owns the
  gates. Taste work is judged at the deep tier, not below it.

Use full model IDs. Choose by failure cost, not task size. When weekly usage
runs well ahead of the reset cycle, cut concurrency and optional dispatch
before cutting the tier where failure is expensive.
