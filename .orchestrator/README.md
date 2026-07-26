# Shared orchestration board

This directory is awl's tool-neutral execution board. Codex, Claude Code, and a
human collaborator all read and update the same files here.

- `queue.md` is the one source of truth for concrete work, dependencies, and status.
- `ROADMAP.md` remains the product-direction document; do not duplicate it here.
- Tool-specific paths may point here for compatibility, but must not carry a
  second writable copy of the queue.
- Preserve active queue entries when changing tools or worktrees.

**Layout**
- `queue.md` — the canonical execution queue. Siblings support it; never carry a second writable copy.

**Compat symlinks** so every tool's path resolves to this one dir: `.claude/orchestrator` and `.codex/orchestrator` both → `../.orchestrator`. `CLAUDE.md` / `AGENTS.md` reference the shared board contract.

## Claiming protocol (multi-tool coordination)

The board only prevents double-work if claims are visible BEFORE work starts. Any tool (Codex, Claude Code, human) picking up an item:

1. **Claim first, work second.** Edit the item's status line in `queue.md` to `🟡 IN PROGRESS — <owner> (codex|claude|human), branch <name>` and COMMIT that board edit to main before writing any code. An uncommitted claim is invisible to the other tool; git already records when it happened.
2. **Work in a worktree, never the main tree.** Branch off local `main`, name the branch on the claim line. The main working tree belongs to merge gates and the human's live session.
3. **Re-read the board before firing.** A claim may have landed since you last looked. `git pull`-equivalent for us is just re-reading `queue.md` at HEAD.
4. **Land = suite-gated merge to local main** (full `cargo test`, both conventions for keymap-adjacent work) + flip the board line to `✅ LANDED @ <sha>` in the same session. Push per the push policy (public repo — push after green trains).
5. **Conflicts are normal, not a coordination failure.** If two branches collide on merge, reconcile via a merge pass (Claude dispatches a merge agent; Codex resolves inline) — never serialize the whole queue out of fear.
6. **Stale claims:** an IN PROGRESS line older than ~a day with no branch activity may be reclaimed — note the takeover on the line.

## Board writes are ORCHESTRATOR-ONLY (user rule)

Within each tool, the top-level ORCHESTRATOR session is the board's only
writer. Delegated subagents / workflow workers NEVER edit anything under
`.orchestrator/` — they return structured results, and the orchestrator
translates those into board edits:

- **Claims** are committed by the orchestrator BEFORE dispatching build work
  (claim-first still holds; it just isn't delegated).
- **Status flips** (`✅ LANDED @ sha`, defect notes, morning-review entries)
  happen when the orchestrator processes the workers' results — a worker only
  knows its own slice, so letting it flip status invites premature or
  wrong-altitude entries; the orchestrator holds the cross-workstream truth.
- **Why:** one writer serializes the shared file — no same-file races between
  concurrent workers and the live session (the exact race this prevents: a
  spec log held back because two merge-train agents had queue.md edits in
  flight); and the board keeps one consistent voice and altitude.
- **Corollary:** board edits happen only in the MAIN working tree, never in a
  worktree — a worktree's `queue.md` edit dumps a guaranteed conflict on the
  merge train.
- Worker briefs must therefore NOT include "edit the board / flip the claim"
  steps; they report shas + outcomes instead.

## Design sessions → decisions → the board (user rule)

How a brainstorm/interview session ("awl design"-type) turns talk into work:

1. **Brainstorm read-only.** During discussion the orchestrator changes nothing.
   **Interview ruthlessly (user rule):** when a note or an answer is
   ambiguous, the designer asks until the intent is unambiguous — a guess never
   gets built into a queue item.
2. **Decisions land as queue items.** Each crystallized decision becomes a
   self-contained item (or a `DECIDED` line folded into an existing
   item) — a worker must receive the decided thing, never the open question.
3. **The commit message is the session record.** Board-decision edits are
   committed to main with a subject starting `orchestrator: decisions` —
   `git log --grep=decisions` replays every design session in order. There is
   deliberately NO decisions log file: append-only logs rot into noise, and
   git already keeps the full history (the CLAUDE.md philosophy).
4. **Retention tiers (existing law, restated):** build decisions live in queue
   items → git history when archived. A standing constraint a future agent
   would re-litigate ("no locale sniffing") gets ONE line in CLAUDE.md's
   "Open decisions & known divergences". Taste/product-level decisions amend
   PHILOSOPHY/DESIGN/SCOPE/THEMES in the landing round.
5. **The user's notes (private, outside the repo) are the user's space.**
   Agents READ questionnaires and notes there when directed; they NEVER write
   there. The machine-side record lives in this repo.
6. **One-off reports do not become a second archive.** Harvest actionable work
   into `queue.md`, standing invariants into `CLAUDE.md` or the matching
   technical doc, and product/taste laws into the contract docs. The working
   report stays in its task output; git preserves any deliberately committed
   history. Do not accumulate a `reports/` directory beside the live board.

## Cooking: parallelize by clash, run unattended (user rule)

When the orchestrator is cooking a queue, the default is throughput within the
active quota budget, not raw fan-out:

1. **Parallelize by file-clash, not by fear.** Items whose file/module
   footprints are DISJOINT cook CONCURRENTLY — a few at a time (~3–4; enough to
   fill the machine without thrashing cargo), not one-at-a-time. Assess each
   item's footprint up front; only genuine same-file clashes are sequenced.
   Parallel builds run in ISOLATED worktrees (never the main tree), so
   concurrent builds can't clobber each other or a live session.
2. **Integration stays serial and gated.** Worktree branches merge to main ONE
   at a time through the suite gate (full `cargo test`, both conventions for
   keymap-adjacent work, wasm on the train — the existing merge train). A clash
   on merge is reconciled by a delegated MERGE agent, never by serializing the
   whole queue out of merge-fear and never by the orchestrator hand-editing
   conflict markers. On any red at integration, reset main clean and skip-flag
   the item (below) — main is never left broken.
3. **Aim to cook unattended; never idle with work queued.** While independent
   items remain, something is always cooking. A stuck item — can't reach green,
   or hits an ambiguity that would need a user decision — is REVERTED clean,
   left out of main, and FLAGGED for the user; it never blocks the rest. Only
   genuinely user-gated items (a permission grant, an approval, a taste call the
   user reserved) wait; everything else proceeds. "If you get stuck, do
   everything else before pausing to wait for my say" (user).
4. **Never interrupt the cook to interview the user (user rule, 2026-07-26:
   "you can just block it, and note it down in the queue… the cooking agent is
   just focused on cooking").** A fork only the user can settle does NOT become
   an interactive question — it BLOCKS its item, gets written into the queue as
   a stated question with the options and a recommendation, and the wave moves
   on to the next independent item. Do not answer it yourself either: a taste or
   feel call the user reserved is theirs, and an orchestrator-invented answer
   buried in a queue item is worse than an honest block, because it looks
   decided. Write the question so it can be answered cold, then keep cooking.
   Design SESSIONS are the opposite case — there the user opened the
   conversation, so interview ruthlessly (§Design sessions).

## Dynamic harnesses inside queue items (user rule)

**A queue item is the unit of outcome, not the unit of agency.** Keep each agent
on one bounded role with a fresh context, but choose the item's internal
workflow dynamically from its risk and the evidence it produces. The workflow
owns branching, loops, barriers, and synthesis; one long generalist agent is
not the default.

Use the smallest shape that can establish the promised outcome:

- **Routine, deterministic change:** one implementer, then an independent
  verifier.
- **Unknown root cause:** parallel competing hypotheses from disjoint evidence,
  an evidence-based selector, the fix, then the standing neighborhood audit.
- **State/ownership migration:** the qualifying Opus ownership map, focused
  invariant/call-site probes, Sonnet implementation, then adversarial
  verification against those invariants.
- **Visual/taste round:** bounded candidate generation, real awl captures,
  Fable selection against the decided rubric, implementer-applied verdict, then
  Sonnet pixel/perf verification. Fable still never implements.
- **Broad audit or open-ended hunt:** enumerate the cells, fan out bounded
  probes, synthesize/dedupe, and repeat only until a stated stop condition
  (green, no new findings, or no progress) is met.

Dynamic does not mean unbounded. Default to fewer than five agents inside one
item and retain the machine-wide ~3–4 active-workstream limit. Name a stop
condition and token/repair budget before any loop or tournament; routine work
does not earn speculative fan-out. Evidence may add a repair, re-verification,
or merge specialist, but never silently expand product scope.

The top-level orchestrator still exclusively owns `queue.md`, local-main
integration, push trains, remote CI, and user gates. A workflow returns
structured evidence and commits from its worktree; it never writes the board,
merges or pushes main, or replaces durable queue/worktree state. Save a workflow
script only after the same orchestration shape has proved genuinely reusable;
per-task scripts are disposable, not repository cruft.

## Push trains and remote CI (user rule)

Local gates protect a commit; remote CI protects `main`. Treat them as two
separate gates:

1. **Push small trains, not individual churn or giant batches.** Integrate 2–3
   locally green build items serially, then push that mini-train. Queue/docs-only
   commits ride the next train, or push at the end of a session when no build
   train is coming. Push immediately for a CI repair, serious correctness or
   data-loss fix, user-requested checkpoint, handoff, or release preparation.
2. **One remote train at a time.** After every push, wait for the resulting
   non-cancelled `main` CI run to finish before integrating the next train into
   local `main`. Independent workers keep cooking in their worktrees while CI
   runs; only the merge train waits. Never push over an in-flight run merely to
   cancel it.
3. **Red CI is live queue state.** A failing `main` run creates or updates the
   top-priority `CI RED` queue item with the run URL, failing job/test, and first
   known bad commit. Assign one Sonnet worker immediately. Other worktrees may
   continue, but nothing else integrates into `main` until the fixing push is
   remotely green. Cancelled superseded runs do not count.
4. **Reconcile at durable boundaries.** Before dispatch, after compaction or a
   tool/task handoff, and before answering a queue/status question, compare the
   board with actual ahead/dirty worktrees and the latest non-cancelled `main`
   CI result. The live orchestrator UI/scratch remains the detailed runtime
   view; the board records only enough owner/branch/phase state for another
   orchestrator to recover honestly. Do not copy transient agent counts,
   timers, token usage, or scratch narration into the board.

## Execution hygiene

These are orchestration rules, not live queue state:

- **Durable worktrees only.** Never create a worktree under `/private/tmp`;
  use `.claude/worktrees/` or another durable path. Commit work in progress
  before any pause.
- **Clean up after every landed wave.** Once a worktree is clean and its patch
  is merged (or patch-equivalent on main), remove the worktree and prune stale
  registrations. Leave dirty, unmerged, locked, or differently-owned
  worktrees alone unless their owner explicitly hands them back. Not deferrable
  bookkeeping: each worktree carries its own `target/`, and 27 of them reached
  132 GB before the first sweep. `git worktree remove` deletes the checkout, not
  the branch — removing a merged worktree loses nothing, so the only ones worth
  keeping are those holding uncommitted or untracked work.
- **Preserve gate truth.** Never pipe a build/test gate through `head`, `tail`,
  or anything else that can hide its exit status. Run the wasm gate on every
  train as required by `AGENTS.md`.
- **Treat suspicious incremental failures as suspect first.** Retry with
  `CARGO_INCREMENTAL=0` before diagnosing product code. A `signal: 9, SIGKILL`
  with NO test failure line is not a product failure at all — it is the OS
  reaping the test binary. Check memory before diagnosing: awl's suites are
  GPU-heavy, and 3–4 concurrent worktree suites plus the main tree's gate can
  exhaust swap on this machine (observed 2816 MB of 4096 MB used, gate killed,
  identical tree green on retry). The ~3–4 workstream ceiling is about the
  machine, not just about merge conflicts — when a gate is SIGKILLed, re-run it
  alone rather than bisecting the innocent merge.
- **Terminate only owned processes.** Never kill `awl` by a bare process name;
  stop only the exact PID created by the current run.
- Background model/effort routing follows the Brew skill and any narrower
  repository rule; record any launcher substitution.

## Blocked items PARK; never stall the queue on them (user rule)

When an item hits a blocker only the user can clear — a taste/product fork, a
permission grant, an approval — the orchestrator NOTES it blocked on the board
(a one-line status naming exactly what's needed) and IMMEDIATELY moves on:
every non-blocked item keeps cooking in parallel. The blocker note IS the
channel: WRITE the decision straight into the queue item — the specific fork,
the options, and your recommendation — mark it blocked, and move on. The user
resolves it by EDITING THE QUEUE on their own time (exactly how items 48–52 and
the frost rework landed). Do NOT reach for an interactive prompt that stalls the
turn for a decision that could just be a queue note, and do NOT idle other work
waiting on the answer. The blocked item resumes the moment the user's queue edit
lands. (User's word: "you can just write down the blocking decision in the queue
and move on"; "just note that it's blocked and churn through everything else.")
The failure this kills: asking a question and then idling the whole queue until
it's answered.

## Delegation: what the orchestrator does and does not touch (user rules)

Ported from the retired `cook` skill 2026-07-26, which is deleted — this file is
now the single owner of how cooking works, so the two cannot drift apart again.

- **The orchestrator never writes code — it only delegates.** Hands off the
  build: no `Edit`, no `Write`, no code the orchestrator authors, not in the
  session and not as "glue" inside a workflow script. The script is control flow
  (`await`, loops, fan-out) that ROUTES work to agents; every line that lands in
  the codebase is written by a dispatched worker. Reading, brainstorming, writing
  the spec, and keeping `queue.md` remain the orchestrator's. The urge to "just
  fix this one line" is exactly the urge to resist — dispatch for it. This
  applies to merges too: a conflict is reconciled by a delegated MERGE agent,
  never by hand-editing conflict markers (already stated under Cooking).
- **Never investigate inline either — delegate the diagnosis** (user rule,
  2026-07-18: "please don't do investigations yourself! you can spin up a
  workflow and delegate investigations there!"). Light reading to WRITE a spec
  stays the orchestrator's; a real investigation — reproduce, capture/measure,
  eliminate candidates, root-cause — is itself delegated. It burns the
  orchestrator's context with probe noise and is exactly the bounded, verifiable
  work a worker does well. Shape it `diagnose → fix → verify` in ONE workflow:
  the first phase returns the root cause with its evidence, the next builds, the
  last adversarially verifies. On catching yourself running a fifth
  `--screenshot` or grep to chase a cause, hand the whole investigation over.
- **A real build is a Workflow, not a background Agent.** "It can't be
  parallelized" is a reason for SEQUENTIAL PHASES inside a Workflow — `await` one
  agent, then the next, each building on the prior commit — not a reason to drop
  to a lone Agent. The Workflow keeps deterministic phases, live progress, and
  the verify stage; an Agent gives none of them. A genuinely minor one-off (a
  rename, a lookup) needn't be a Workflow at all; a single subagent is fine.

## Delegating to Codex (user rule, 2026-07-18: "you can now delegate code to codex")

The local Codex CLI is wired in (`codex:rescue` skill / `codex:codex-rescue`
agent). Hand Codex a burner exactly as you would a Claude worker — self-contained
brief, its own worktree, a diagnose/build/verify shape — and pick its
model+effort to match the ROLE the quota ladder above assigns to that stage, not
the stage's size:

- **`gpt-5.6-terra` at `high`/`xhigh`** — the PRODUCTION DEFAULT, Sonnet's role:
  implementation, structured research, routine diagnosis, merge reconciliation,
  spot-check audits.
- **`gpt-5.6-sol` at `low`** — Opus's role: the qualifying plan for high-risk
  ownership/state work, and targeted verification while a hidden-risk condition
  remains.
- **`gpt-5.6-sol` at `high`** — the hardest REASONING tier: subtle root cause
  where candidates must be eliminated with evidence. Note this is NOT Fable's
  role — Fable judges awl's real captures as images and Codex cannot, so taste
  calls never go to a Codex burner.

Use the FULL model ids; the account's CLI rejects a bare `gpt-5.6` with a 400
"model is not supported" (`gpt-5.6-sol` is the default in `~/.codex/config.toml`).
Pass via `codex:rescue` flags, e.g. `--model gpt-5.6-terra --effort high`.
Spreading burners across both runtimes is encouraged when many things are
cooking — it widens throughput and gives independent implementation and
diagnosis perspectives. Record which runtime and tier each burner got.

## Quota-aware model routing (user rule)

The shared subscription allowance is a weekly compute budget. Choose a model by
failure cost and task phase, not by a blanket "coding = strongest model" rule:

1. **Sonnet is the production default.** Use the current Sonnet at medium effort
   for implementation, structured research, routine diagnosis, merge
   reconciliation, and every standing spot-check audit. Raise effort only when
   the task's evidence demands it. A crisp brief + one known owner +
   deterministic tests/captures is a Sonnet job even when it touches several
   files.
2. **Opus plans high-risk ownership work; Sonnet executes the plan.** Spend the
   current Opus on the reasoning phase when a task migrates state/identity,
   discovers ownership across subsystems, or could produce a locally green but
   subtly corrupt result. The Opus output is a concrete file/owner/invariant
   plan handed to Sonnet for implementation (`opusplan` where available).
   Targeted Opus verification is allowed when the same hidden-risk condition
   remains. Do not use Opus merely because a task is large, and do not use fast
   mode for unattended work where latency has no value.
3. **Fable judges taste only.** The implementer generates real awl captures;
   Fable inspects those images and returns the visual call. Fable never writes
   implementation or edits files. The implementer applies the verdict and owns
   the gates.
4. **Escalate from evidence, never prestige.** Promote a Sonnet task only after
   its investigation exposes an ownership/invariant ambiguity or repeated
   attempts fail for reasons requiring deeper reasoning. Record the reason in
   the worker brief; model escalation is not a substitute for a sharper spec.
5. **Budget context as well as agents.** Give each worker one self-contained
   queue brief and a fresh bounded context. Do not keep feeding unrelated work
   through a conversation carrying old repository reads and diffs. Avoid
   duplicate scouts and speculative agents; parallelize independent build work
   in the existing ~3–4 slots, then drain and clean them before the next batch.
6. **Pace against the reset.** At dispatch, compare weekly percentage used with
   percentage of the reset cycle elapsed. If usage is more than about ten
   percentage points ahead, enter conservation mode by reducing compute per
   completed task, not throughput: keep the normal 3–4 independent workers
   cooking, use Sonnet at medium effort by default, reserve Opus for qualifying
   data/state-risk planning, and avoid duplicate scouts, repeated speculative
   attempts, and unbounded inherited context. Defer work only when intentionally
   moving an optional task beyond the reset. A separate, underused taste
   allowance does not justify using Fable outside its role.
