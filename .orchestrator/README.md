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

## Concurrent worker build budget

The orchestration layer owns the CPU policy for concurrent workers. On this
ten-core host, every concurrently dispatched worker runs build or gate commands
through `.orchestrator/worker-build.sh`; it sets and receipts
`CARGO_BUILD_JOBS=2`, so four workers schedule at most eight Cargo jobs in
aggregate and leave interactive headroom. The wrapper is the sole budget owner:
workers and repository gate scripts do not set a competing value.

The root's isolated merge-train gate, CI, and a developer's lone build do not
use the wrapper and remain hardware-adaptive. The wrapper is intentionally a
launch seam, not `.cargo/config.toml`; it passes its environment through to
native, wasm, and other scripts without changing what those gates run or claim.
When dispatching, state the wrapper command in the worker brief and report its
receipt with the gate outcome, for example:

```sh
.orchestrator/worker-build.sh scripts/native-gate.sh
.orchestrator/worker-build.sh scripts/web-smoke.sh
```

**Tell the worker to run gates in the foreground, with an explicit long
timeout — except `native-gate.sh`, which cannot fit.** A worker that launches a
multi-minute gate in the background has nothing left to do and ends its turn, so
the orchestrator must wake it once per gate; on 2026-07-31 several lanes each
burned two or three round trips this way, and some ended a turn holding
uncommitted work. The brief should name the timeout, because the default is
shorter than a cold gate.

**But `native-gate.sh` genuinely exceeds the tool's 600 s maximum in this
repo** — it runs the suite under both conventions, and `cargo test --bin awl`
alone measured 276 s at `cargo_jobs=2` on 2026-08-01. The harness auto-
backgrounds it; that is the tool, not the worker's choice. Briefs told lanes
"never background a gate" for a full night before item 131 measured it, which
made the instruction unfollowable for the one gate that issues the receipt.
Say instead: run code-health and web-smoke in the foreground; expect
`native-gate.sh` to be auto-backgrounded, and **wait on it rather than ending
the turn** — ending the turn is the actual failure, because nothing wakes a
worker but the orchestrator. Committing before any wait remains mandatory
regardless.

**A turn must never end on the word "holding".** On 2026-08-01 one lane ended
four consecutive turns saying it was waiting for a monitor to notify it that a
gate had finished. The gate had already finished — no build process was even
running — and its work sat uncommitted underneath the whole time. Two separate
mistakes hide in that pattern and both are worth naming. First, **an armed
background monitor is not a wake-up source**: nothing but the orchestrator
resumes a worker, so waiting on a notification is waiting forever. Second, a
turn that ends with no findings wastes the round trip entirely — the
orchestrator has to spend a message asking what happened before it can spend
one deciding anything.

So the rule is about what a turn *ends with*, not about what it waits on: if a
turn has to end, it ends with the findings so far written down, gaps named as
"unknown". A partial report costs one round trip. "Holding" costs one round
trip and buys nothing. When the orchestrator needs to know whether a lane's
gate is still alive, the honest check is the host itself — `ps aux | grep
cargo` — not the lane's own account of it.

**‼ STOP WRITING THAT RULE AND START WRITING THE ORDER INSTEAD. On 2026-08-04
SEVEN lanes in one wave ended a turn on a status line, every one of them
carrying the rule verbatim in its brief.** An instruction that loses seven times
out of seven is not being ignored; it is unfollowable as written, and the reason
is mechanical. A lane launches the final gate, the harness auto-backgrounds it,
and the lane now has *nothing left to do* — ending the turn is not a choice it
makes, it is what happens when there are no more tool calls. Telling it "do not
end the turn" asks for the impossible; what we actually want is that **the
findings already exist when that moment arrives.**

**So brief the ORDER, not the prohibition:**

1. Do the work, commit it.
2. **Write the full report FIRST** — everything except the receipt: what was
   built, the premise check, the captures and their arithmetic, the mutation
   panic text, what is owed to a human.
3. **Then** launch `native-gate.sh` / `web-smoke.sh`.
4. When woken, **append the receipt** to a report that is already written.

This also fixes the failure the older text describes, because a lane that has
already written its findings cannot end a turn without them. And it removes the
temptation to commit mid-gate — step 1 finishes before step 3 begins, which is
the other thing three separate actors got wrong on 2026-08-04, each throwing
away a clean full native run to a `HEAD changed while the suite ran` refusal.

‼ **THE ORDER REWRITE IS STILL NOT ENOUGH, MEASURED 2026-08-05: three of four
lanes carrying it verbatim still ended a turn on a status line.** But they
failed *differently* from the 2026-08-04 wave, and the difference names the
missing sentence. **Every one of them had already committed** — step 1 held. What
they lost was only the REPORT, and they lost it to a harness property no
amount of ordering can fix:

> **ONLY A LANE'S FINAL MESSAGE REACHES THE ORCHESTRATOR.** Everything written
> earlier in the same turn — however complete, however carefully assembled — is
> not delivered. One lane said in as many words: "the full report is written
> above (already delivered)". It was not. The orchestrator received that
> sentence and nothing else.

So "write the report first" is, read literally, the instruction that *causes*
the loss: a lane writes its findings, then launches a gate, and the launch
becomes the final message. **Say this instead, and say it in the brief rather
than here, because a lane reads the brief:**

- **Your report must BE your final message.** Not written earlier in the turn,
  not split across messages, not summarised at the end of a longer one.
- **If a gate is still running when you must stop, your last message is still
  the whole report**, with the gate named as outstanding — not a status line
  about waiting for it.
- **Never arm a Monitor or background watcher to wake yourself.** Nothing
  resumes a worker but the orchestrator; a self-armed watcher guarantees the
  turn ends on a status line, which is exactly how two lanes lost their reports
  on 2026-08-05.

**The orchestrator's half of this contract:** when a lane returns a status line,
do not ask "what happened" — check the host directly (`git log` on its branch,
`ps` for its gates), then wake it with a numbered list of exactly what is
missing. A lane that has done the work can restate it in one message; the round
trip is only wasted if the orchestrator spends it asking an open question.

## Disk-pressure preflight

`.orchestrator/disk-preflight.sh` is the one serialized disk-recovery door.
`worker-build.sh` invokes it before every concurrent worker command; the
canonical native gate invokes the same owner for the root merge train. Above
its 32 GiB healthy fleet floor it only reads filesystem capacity. Below that floor it
locks, rechecks, and asks the sole traversal/deletion owner, `scripts/sweep.sh
1`, for recovery. A post-sweep 24 GiB minimum is an early, truthful failure.
The serializer is a kernel advisory lock held through inherited file descriptor
9. The lock file may persist, but its contents carry no authority; the kernel
releases ownership when a process exits or is killed.

CI is intentionally capacity-only: it does not install or run `cargo-sweep`,
and uses an explicit 2 GiB capacity floor rather than the local four-lane
reserve, so an undersized hosted runner fails before Cargo instead of attempting
host cleanup. WebAssembly remains portable because its worker launch inherits the
same preflight, while standalone `scripts/web-smoke.sh` stays a normal
cross-platform build script with no macOS-specific disk policy.

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
7. **Never block on the user.** A decision only the user can make becomes a
   `🔵` board item carrying the exact question, the options with their real
   measured tradeoffs, and a recommendation — then unrelated work keeps
   moving. Do not stop the wave to ask interactively; the user may be away,
   and a taste pick is not a gate. Leave the current state landable so the
   answer costs one command rather than a rebuild. This is the same rule as
   §Design sessions' parked decision, repeated here because it is needed at
   integration time, not only in a design session — where it was filed, it
   read as scoped to brainstorming and got missed.

Only the orchestrator writes the board, from the main working tree. Workers
report shas and outcomes.

## Gates and landing

Run the cheapest thing that can falsify the current claim; run the full set
once, at landing.

- **While working:** rustfmt, the narrowest affected clippy/health arm, the
  targeted tests. Expected to run often, so keep it small.
- **Before landing, on the exact combined-main candidate:** code health,
  `scripts/native-gate.sh`, wasm smoke, and the item's required captures. Only
  that script's receipt authorizes “full native suite”; it names the exact
  commit and both conventions. **What it does not name is the GPU.** The gate
  runs on the host's own adapter — real Apple Silicon Metal here — so a receipt
  certifies “sound on the hardware the receipts run on, with virtualised-GPU
  behaviour untested by any local gate” (item 232). Item 231's wedge sat green
  under that receipt for ~140 commits while hosted macOS was red. No local arm
  covers the axis and a software adapter cannot supply one, so **the hosted-mac
  jobs are the only arm that sees it.** Since item 243 (user decision
  2026-08-03, resolving item 232's parked question), that arm is split:
  `mac (build + test, minus render::tests)` GATES `main` directly, and
  `mac (render::tests)` is tolerated red, pinned by name to item 231 in the
  workflow file itself. `cargo test --bin awl` is “binary unit tests”;
  every filtered Cargo invocation is “targeted tests.” Counts never prove
  scope. Repair failures on the candidate, rerun the failed slice, then the
  full set once.
- **Push after two or three landed items.** Board- or docs-only changes ride
  the next train. Push immediately for a CI repair, a correctness or data-loss
  fix, or a requested checkpoint.
- **Check `main`'s CI before pushing and after** —
  `gh run list --branch main --limit 1`. A green local train says nothing about
  the remote. While `main` is red, the repair is the only thing that ships.
  **A `cancelled` run is not a pass — it is no verification at all.** Check for
  the last **successful** sha, not the last run:
  `gh run list --branch main --limit 12 --json headSha,conclusion -q '.[] |
  select(.conclusion=="success")'`.
  **`cancelled` has two unrelated causes that read identically in the
  conclusion field, and the fix is to check duration, not to slow down
  pushing.** A genuine supersede (a newer push cancelled an in-flight run) is
  usually short — it dies within minutes of the next push landing. GitHub also
  reports a **timed-out** job as `cancelled`, with no separate status: item 196
  (2026-08-01) found the orchestrator had misdiagnosed exactly this, twice,
  attributing a string of cancellations to pushing faster than CI's cycle, when
  four of the last six `mac`/`linux` runs had actually run the clock out on
  `timeout-minutes: 30` (durations `30m21s`, `30m25s`, `30m18s`, `30m25s`
  cancelled vs. `29m59s`, `27m05s` success — one of the two passes cleared the
  wall by a single second). `gh run view <id> --json jobs` gives per-job
  `startedAt`/`completedAt`; a `cancelled` job that ran close to its
  `timeout-minutes` is a timeout, not a supersede, no matter how it was
  triggered. Waiting longer between pushes never fixes a timeout — only
  lowering the job's real cost or raising its ceiling does. When a wave is
  landing quickly, still let one run finish before pushing the next — CI's
  linux job is the only thing that tests on real Linux, which no local gate
  covers — but do not assume a `cancelled` streak means "pushed too fast"
  without checking the clock first.
- **Keep the local toolchain level with CI's** — `rustup check`. CI tracks
  floating stable; a stale local clippy cannot see the lint it is pushing.

**Do not commit while the merge train's own gate is running.** `native-gate.sh`
records HEAD at start and end and refuses to issue a receipt if they differ —
correctly, since a receipt naming a commit that moved underneath it certifies
nothing. On 2026-07-31 the orchestrator committed a board note during item 186's
gate and threw away a full native run: every test passed, no receipt. Write the
board note first or hold it until the receipt lands; the gate is the one thing
that cannot be redone cheaply.

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
