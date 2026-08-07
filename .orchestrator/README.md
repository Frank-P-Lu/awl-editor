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
`CARGO_BUILD_JOBS=2`. The wrapper is the sole budget owner: workers and
repository gate scripts do not set a competing value.

⚠️ **THE AGGREGATE IS 16 BUILD JOBS FOR FOUR LANES, NOT EIGHT — this paragraph
claimed eight for as long as it existed, and the arithmetic omitted the same
×2 the paragraph below applies correctly to test threads.** `CARGO_BUILD_JOBS`
is per **Cargo invocation**, and `native-gate.sh` says so in its own source:
*"Two conventions run at once below, so every bound here is per convention"*
(`gate_conventions=2`), launching both concurrently near the end of the script.
So one lane at the gate phase schedules **2 conventions × 2 jobs = 4**, and four
lanes schedule **16 on ten cores** — over-subscribed, not headroom-leaving.
**Measured 2026-08-06 with four lanes at the gate phase: load average 69.79**,
35 live `cargo`/`rustc` processes (item 277's earlier reading of 49.6 was taken
the same way and is consistent). Derived from the gate's source and confirmed
against `ps`, because the old figure was asserted rather than computed.

**Consequences for a dispatching orchestrator, in order of usefulness:**
- **Four lanes is the practical ceiling on this host, and only because gates
  stagger.** Do not read "eight jobs, interactive headroom" as spare capacity —
  there is none once two or more lanes reach a gate together.
- **Never run the root merge-train gate while lanes are gating.** It is
  deliberately hardware-adaptive and unbounded, so it lands on top of an already
  over-subscribed host. Check `sysctl -n vm.loadavg` and
  `ps aux | grep -cE "[c]argo|[r]ustc"` first; wait for the wave to quiesce.
- This is the load that makes `test-native-gate.sh`'s CPU-heartbeat self-test
  flake (see below) — the two facts are the same fact.

**`CARGO_BUILD_JOBS` bounds compilation only, not test-execution parallelism**
(item 277, measured 2026-08-05: load average 49.6 on this ten-core host with
four dispatched lanes each running `native-gate.sh`, one gate 48 minutes into
a run that normally takes ~4). `native-gate.sh` runs both keymap conventions
concurrently, and each `cargo test` defaults its harness thread count to the
core count, so four lanes at the gate phase — the build budget fully honoured
— still schedule `4 workers x 2 conventions x (core count)` runnable test
threads. The wrapper also exports `RUST_TEST_THREADS=1`, so four workers'
gates schedule at most eight test threads in aggregate (`4 x 2 x 1`), matching
the build budget's own aggregate of eight instead of several times the host's
core count. `native-gate.sh` already deferred to a caller-supplied
`RUST_TEST_THREADS` before this item — it only computes its own core/memory-
derived default when the variable is unset — so this needed no change inside
the gate script itself; the wrapper remains the sole owner of both bounds.

⚠️ **ONE CHECK IN THIS REPO FAILS FROM ORCHESTRATION LOAD ALONE, AND READS AS A
CODE DEFECT.** `scripts/test-native-gate.sh`'s CPU-heartbeat self-test — *"the
busiest NEW process … dropped instead of measured over its own age"* — went red
for a lane at load average 33–42 on this ten-core host with concurrent lanes at
the gate phase, on a diff that could not touch it, and passed on a rerun once
contention eased. **The wrapper's bounds do not prevent this**: they cap Cargo's
build jobs and test threads, not the number of short-lived processes a heartbeat
probe has to measure against its own age. So a red heartbeat self-test during a
multi-lane wave is classified as contention **before** it is attributed to a
diff — check the load average first, then rerun alone. It is the same
classify-before-blaming rule as an incremental failure or a `SIGKILL`, applied
to a check whose own configuration is the thing that failed.

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

**‼ INSTANCE EIGHT, 2026-08-06, under a brief carrying the order verbatim —
and it failed by a mechanism the order does not address.** The lane launched
**two** gates back to back (`code-health.sh`, then `native-gate.sh`), both
auto-backgrounded, and only then had nothing left to do. Its work was committed
and its gates were genuinely alive, so the status line was *honest*; what it
cost was the report. **The reason a lane defers the report is that it wants the
receipts in it** — writing first feels like writing an incomplete report. So say
the thing that dissolves that motive:

> **Treat your first gate launch as the LAST tool call you will ever make.
> Everything you intend to say must already be written when you make it.** The
> report is not a summary of finished work — it is a document you write with the
> receipt lines reading `outstanding`, and edit only those lines if you are
> woken.

The orchestrator's recovery is cheap and should be assumed rather than feared:
checking the branch and `ps`, then sending a numbered list, costs one round trip
and always works. **A lane that ends on a status line has not lost its work — it
has lost one turn.** Do not let the fear of that make a lane sit on a finished
gate.

‼ **BUT CHECK FOR UNCOMMITTED WORK FIRST, BECAUSE THE WORSE VARIANT LOOKS
IDENTICAL FROM HERE.** Measured 2026-08-06, instance nine: a lane returned the
usual status line — *"waiting for the background test run"* — and its branch had
**ZERO commits** with **24 modified files plus one untracked** in its worktree, 115
insertions across `actions.rs`, `app/apply.rs`, `keymap.rs`, `replay_effects.rs` and
a new module. It had inverted the order completely: launched a suite, then ended the
turn, having never committed. The lane was the **sole copy** of everything.

A status line with committed work costs a turn. A status line with *uncommitted*
work is one crashed process away from costing the whole item, which is why
"commit before pausing" is the first rule and not a tidiness preference. **So the
orchestrator's first probe is not `ps`, it is:**

```sh
cd <worktree> && git log --oneline main..HEAD && git status --short
```

An empty log with a dirty status means the reply is **"commit now, then report"**
and nothing else — no gates, no questions — because every further instruction is
worthless until the work exists somewhere other than one process's working tree.
Tell the lane explicitly to `git add` **named paths**: a shared repo has already
lost a sibling's source to `git add -u`.

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

‼ **TWO MUTATIONS AT ONCE CAN NEUTRALISE EACH OTHER, AND THE SURVIVOR READS GREEN.** Measured
2026-08-07: a lane broke a hardcoded constant *and* forced the quantity that constant should have
differed from to zero — which made the two coincide, so the first mutation had nothing left to
change and its law passed. Run separately, both fired. **One mutation at a time, and if a
combination is genuinely needed, say what each one's subject is and check they are disjoint.**

‼ **AND A MAGNITUDE PROBE IS BLIND TO ANY LAW THAT HAS OPTED ITSELF OUT.** In the same census, two
laws could not see a tripled reserve because an **earlier arm of the same roster** ended with a
hardcoded `set_menu_bar_on(false)` — so their configuration was a property of **iteration order**,
not of the platform. **A sweep that forces a global finds only the tests that read it; enumerate
the population that pins it off and read those separately.** An arm that pins a global restores
the **ambient** value, never a `cfg!` — inside a test that reflects the host that compiled it.

‼ **A MUTATION CAN BE A SILENT NO-OP, AND THEN A GREEN LAW LOOKS LIKE A PROVEN ONE.** Measured
2026-08-07: a lane's mutation script targeted lines that **rustfmt had since joined**, so the
edit applied to nothing and the law reported green — indistinguishable from a law that survived
its own mutation. It was caught only because the patch script **asserted its replacement
applied**.

**So a scripted mutation asserts it changed something**, and the cheapest form is what this
repo already uses elsewhere: `assert t.count(old) == 1` before replacing, or check the file's
hash moved. A mutation you did not prove landed is not evidence, and it reads exactly like the
strongest possible result.

**A law that was never watched failing is not evidence.** The author breaks the
product, watches the law go red by name, restores, and pastes the actual panic
text into the report. This is the single cheapest defect-catcher available and
it belongs to the author, not to a reviewer.

The axis a law sweeps must be the one the author did not think of. A law that
checks only imagined cases goes green over a real defect. Sweep the roster,
the whole geometry range, the empirical worst case.

**‼ THE REAL PAYOFF IS NOT "THE LAW CAN GO RED" — IT IS CATCHING A LAW THAT
CANNOT.** Worked instance, item 291, 2026-08-06. The lane wrote the obvious test
for a burst-counting bug: mark, present, mark, present. It passed. Then it ran the
mutation — reinstated the exact bug — and **the test stayed green**, because an
immediate present always drains the single slot before the next mark arrives, so
the alternating shape can never observe the collapse. The lane rewrote the test to
the **zero-gap arrival order** (every mark fires before any present closes one
out, which is what a fast burst actually produces when input outruns the frame
loop), and only then did the mutation go red — at **n=2**, the smallest length
where a single-slot bug is distinguishable at all.

Two lessons, and the second is the one that generalises:

1. A hand-picked burst length would have hidden it. **Sweep the parameter**
   (this law sweeps n ∈ {1, 2, 3, 8, 9}); an off-by-one in a queue passes at one
   length and fails at another.
2. **A green test plus a green mutation is not two pieces of good news — it is
   one piece of bad news.** If breaking the product does not break the law, the
   law is measuring something else. This is the cheapest way to discover that,
   and it only works if the mutation is run *before* the law is believed.

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

## Shell hazards that silently corrupt what you write

⚠️ **BACKTICKS INSIDE `git commit -m "..."` RUN AS COMMANDS AND EAT THE TEXT.** Bitten
twice on 2026-08-06. A message containing `` `use super::*;` `` or `` `height: u32` ``
lost exactly that span — zsh substituted it, printed `command not found`, and committed
the sentence with a hole in it. The commit still succeeds, so nothing fails; the record
is just quietly wrong, which is worse. **Always write a commit or merge message through
a quoted heredoc**, and note that `-F -` reads stdin only for `commit`, not for
`merge` — `git merge -F -` errors with `could not read file '-'`, so a merge message
needs a real file:

```sh
git commit -q -F - <<'EOF'
...message with `backticks` and $vars safely...
EOF

cat > "$TMPDIR/msg.txt" <<'EOF'
...merge message...
EOF
git merge --no-ff <branch> -F "$TMPDIR/msg.txt"
```

**And never pipe a gate through `grep`.** A grep matching *either* outcome exits 0 and
hides the failure exactly as thoroughly as `|| true`; that is how a red tree reached
`origin` today. Assert the exit status, or count failure lines and report from the count.

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
5a. ⚠️ **THE "DIFF BEFORE COMMITTING" CHECK BELOW HAS A BLIND SPOT THAT HAS NOW
   EATEN THE BOARD A THIRD TIME: it looks at ITEM lines, and what gets lost is
   SECTIONS.** Measured 2026-08-06: a script retiring three closed item bodies
   deleted the TRIPWIRE, Decided against, Parked, Monitoring and Release-blockers
   sections in one go, because the last closed item was the last numbered item on
   the board and the loop that skips a body "until the next numbered line" ran to
   EOF. The diff was checked — for `^\d+\.` lines, exactly as §5 says — and that
   filter is structurally incapable of showing a dropped `## ` heading.

   **So the census is of HEADINGS, not items, and it runs after every board write:**

   ```sh
   grep -c '^## ' .orchestrator/queue.md      # expect the same count as before
   grep '^## ' .orchestrator/queue.md         # and the same names
   ```

   And when restoring, **verify byte-identity of the restored region rather than its
   mere presence** — a heading can come back with its body truncated. This is the
   same failure as the BLOCKED heading's two silent deletions, from a third script;
   prefer an anchored replacement of one known block over any loop that scans
   forward for a terminator it might never find.

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

8a. ⚠️ **`scripts/code-health.toml` CANNOT BE PARTITIONED, AND AN ORCHESTRATOR THAT
   ASSIGNS IT TO ONE LANE WILL BLOCK THE OTHERS.** Measured the hard way
   2026-08-06: five of six lanes in one wave needed it, because **any** file that
   grows past its `file_size_mark` trips an EXACT-count ratchet whose raise
   requires a stated reason. Granting it to one lane made a second lane
   code-complete-but-blocked, correctly refusing to route around the lock, and cost
   a round trip to clear.

   **So it is ORCHESTRATOR-OWNED at merge time, never granted to a lane.** Tell
   every lane: *if your change trips a size mark or a clippy exception, report the
   number and leave the file alone — the orchestrator raises the mark with your
   reason when it merges you.*

   ‼ **STATE THE MARK RULE EXACTLY, BECAUSE THE PLAUSIBLE-SOUNDING VERSION IS WRONG
   AND AN ORCHESTRATOR HAS NOW BRIEFED IT WRONG TO THREE LANES IN ONE WAVE.** It is
   **two tiers, not one ceiling**, and `scripts/code-health.py` is the authority
   (`check_mark_raises` + `check_structural`, `BASELINE = "f12d04a"`):
   1. **The hard ceiling is an ABSOLUTE 500-line production limit**, and separately a
      file may not exceed its own size at the frozen baseline `f12d04a`. Whichever
      binds first, binds — for `src/menubar.rs` the baseline is 510, so the 500 limit is
      the real cap, and the failure message states which it used
      (*"production limit is 500; high-water mark is 228"*). A file over the hard
      ceiling cannot be rescued by any `reason` — the remedy is carving `mod tests`
      into a sibling `tests.rs`, which has been the answer repeatedly.
   2. **Below that ceiling a mark MAY RISE, and the only requirement is a recorded
      `reason` for the raise.** *"A mark may rise only with a reason recorded for that
      raise"* is the function's own docstring.

   **"Marks can only tighten" is NOT the rule.** That formulation collapses the two
   tiers into one and refuses raises the tool allows: `src/menubar.rs` sat at a mark of
   228 against a **510**-line frozen baseline, so its raise to 391 was legitimate, and a
   lane that had been briefed "can only tighten" reported it as a violation it had
   worked around. **Look the two numbers up before answering a lane** — the mark from
   the toml, the baseline from `git show f12d04a:<path> | wc -l`. A remembered rule
   about this file has been wrong more than once. The stanzas are per-file, so concurrent edits
   usually merge cleanly; the cost of a lane touching it is not conflict, it is a
   lane blocked behind another lane's grant. **Read the count off the tree at merge
   time (`wc -l`), not out of the lane's report** — one lane reported 904 and a
   later citation-rewording pass left the file at 900.

   ‼ **A GREEN `code-health.sh` CAN CERTIFY A RED TREE, AND THE WINDOW IS AN UNSTAGED
   `code-health.toml`.** The ratchet config is read **off disk**; the file-size scan
   reads **tracked** files. So mark edits left unstaged make a refusing tree report
   clean, and a receipt taken there certifies nothing. Measured 2026-08-06: this cost a
   push of `main` whose committed toml disagreed with its own sources in two places
   (`render.rs` marked 2643 against an actual 2632). CLAUDE.md's "run code-health after
   `git add`" is not an ordering preference — it is the whole guard.
   **`git add scripts/code-health.toml` before the gate, and treat a clean
   `git status --short` immediately before a push as part of the receipt.**

   ‼ **AND TWO LANES LANDING IN ONE FILE BOTH REPORT A NUMBER THAT IS WRONG FOR THE
   MERGE.** `render.rs` came out at 2665 where one lane said 2632 and the other 2676;
   `chrome/mod.rs` at 1035 against 1031 and 1041. Applying reports serially cannot get
   there. **Audit every mark against the tree in one pass:**

   ```sh
   python3 - <<'EOF'
   import pathlib, re
   t = pathlib.Path("scripts/code-health.toml").read_text()
   for m in re.finditer(r'file = "([^"]+)"\nlines = (\d+)', t):
       f, mark = m.group(1), int(m.group(2)); p = pathlib.Path(f)
       cur = len(p.read_text().split("\n")) - 1 if p.exists() else None
       if cur != mark: print(f"{f}: mark={mark} actual={cur}")
   EOF
   ```

   Target a `clippy_exception` by **file + function**, never by message text — two
   blocks can carry identical messages, and a `file` that no longer exists is the
   stale-exemption class item 288 was written to kill.

8. **Partition parallel lanes by FILE, but treat a hold as a DEBT, not a
   boundary.** The partition is what lets several lanes run at once without
   collision, and its bill is duplicated shapes: one lane duplicated a chrome
   geometry because another held the file it would otherwise have called into,
   and said so in its own module doc. A later lane could then not wire its
   headline at all, because the call site sat in a held file — and it correctly
   **reported the block rather than inventing a second owner.** A route around a
   lock is a worse design adopted only to dodge the lock. **When a lane says it
   needs a held file, sequence the two. Two lanes that both need one file are one
   lane.**

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
- ‼ **AND THE ORCHESTRATOR CAN CANCEL ITS OWN ARBITER, WHICH IS THE WORST VERSION OF
  THIS.** Measured 2026-08-07: `main` went red, the run on the *next* sha was the only thing
  that could say whether a later commit had already fixed it — and pushing the CI-RED **board
  note** killed that run, because `concurrency.cancel-in-progress: true` does not care that
  the push was documentation. **The arbiter became the run on the board commit, one full CI
  cycle later.**
  **So while a specific run is the arbiter of something you need, DO NOT PUSH — not even a
  markdown-only commit.** Write the board note, hold it, and push after the run reports. The
  rule "push after two or three landed items" already permits holding; this is the case where
  holding is mandatory rather than tidy.
- ⚠️ **A THIRD CAUSE OF A RED- OR CANCELLED-LOOKING RUN: GITHUB ACTIONS ITSELF IS
  DOWN. CHECK THAT BEFORE DIAGNOSING ANYTHING.** Measured 2026-08-06 on `e2e40445`:
  `linux`, `web` and `atspi` all reported `cancelled` **at the identical second**,
  15 minutes after queueing, while three sibling jobs sat `queued` — and **no push
  had superseded them.** `curl -s https://www.githubstatus.com/api/v2/status.json`
  returned `{"indicator":"major","description":"Partial System Outage"}` with
  `Actions: major_outage`. The run immediately before it had died on
  `##[error]Service Unavailable` / `Failed to resolve action download info`, the
  same outage one stage earlier.

  **The tells, in order of cheapness:** several jobs completing at the *same
  second*; `cancelled` with no push behind it; a duration nowhere near
  `timeout-minutes`; and the status API. **Rerunning during an outage is futile** —
  wait for the API to clear, then rerun.

  **And the state this leaves `main` in must be named honestly: not green, not red,
  but UNVERIFIED REMOTELY.** A local `native-gate.sh` receipt still certifies only
  "sound on the hardware the receipts run on"; CI's `linux` job is the only real
  Linux coverage and the hosted-mac jobs are the only virtualised-GPU coverage, so
  an outage means those axes are simply uncovered. Hold further pushes until a run
  completes, and never let `cancelled` be recorded as a pass.
- ⚠️ **A TOLERATED-RED JOB HAS TWO FAILURE CAUSES, AND ONLY ONE OF THEM FLIPS THE
  RUN'S CONCLUSION.** `continue-on-error: true` is set at the JOB level for
  `mac (render::tests)` and `atspi`, and it tolerates their **steps** failing —
  which is why a run whose only failures are those two reads `success`. It does
  **not** cover a failure in `Set up job`, before any step runs. Measured
  2026-08-06 on `9c9d7da6`: all four gating jobs green, `atspi` red exactly as in
  the last known-good run, and `mac (render::tests)` red with
  `##[error]Service Unavailable` / `Failed to resolve action download info.` — a
  GitHub Actions infrastructure failure that never reached a test. The run's
  conclusion was `failure`; nothing in the tree was wrong. **So a red run
  conclusion is classified by reading the FAILED STEP NAMES, not the job names:**
  `gh run view <id> --json jobs -q '.jobs[] | select(.conclusion=="failure") |
  "\(.name)" + (.steps[] | select(.conclusion=="failure") | "  step: \(.name)")'`.
  A failed `Set up job` on a tolerated job is infrastructure — retry it; it is not
  a `CI RED` item and it does not block integration. This is the same shape as the
  `cancelled`-means-two-things rule above: the conclusion field is not the
  diagnosis.
- **Keep the local toolchain level with CI's** — `rustup check`. CI tracks
  floating stable; a stale local clippy cannot see the lint it is pushing.

**Do not commit while the merge train's own gate is running.** `native-gate.sh`
records HEAD at start and end and refuses to issue a receipt if they differ —
correctly, since a receipt naming a commit that moved underneath it certifies
nothing. On 2026-07-31 the orchestrator committed a board note during item 186's
gate and threw away a full native run: every test passed, no receipt. Write the
board note first or hold it until the receipt lands; the gate is the one thing
that cannot be redone cheaply.

**That rule only binds the session that started the gate — a second, concurrent
orchestrator session cannot obey a fact it has no way to observe**, and this
board explicitly supports two of them at once, so check before every commit to
`main`: `.orchestrator/native-gate.marker` (gitignored) names the PID, start
sha, and start time of any gate the root's `native-gate.sh` currently has in
flight, written while it runs and removed on every exit path including
failure and interrupt. Read it and test liveness with `kill -0`, since a
marker can outlive a killed run and its mere existence carries no authority:

```sh
cat .orchestrator/native-gate.marker 2>/dev/null   # pid=… start_commit=… start_epoch=…
kill -0 <pid> 2>/dev/null && echo "gate PID <pid> is alive" || echo "marker is stale"
```

This is advisory, not a lock — a gate is not entitled to freeze the
repository. A live marker on the sha you're about to move means: wait, or
commit deliberately and accept that the in-flight run's receipt will refuse
itself (`HEAD changed while the suite ran`) and need a rerun. No marker, or a
dead PID, means commit freely.

Integrate one branch at a time. Two branches each green alone can be red
together — a roster or ownership law is designed to cause exactly that. For
structs with per-call-site initializers, grep the construction sites before
declaring a merge done: git merges a missing field cleanly and fails to
compile later.

A failed `main` CI run becomes the top-priority `CI RED` item with the run URL
and first known bad commit, and blocks further integration.

‼ **THE RUN'S CONCLUSION IS NOT THE GATE'S VERDICT, AND A TOLERATED-RED JOB CAN
MAKE A FULLY GREEN TRAIN READ `cancelled`.** Measured 2026-08-07 on `ba292f75`:
the run's conclusion was `cancelled` while **all four gating jobs succeeded**
(`linux (build + test)`, `web`, `mac live-probe`, `mac (build + test, minus
render::tests)`). The cause was `atspi` — a job pinned tolerated-red — running
**30m20s** into its `timeout-minutes: 30` ceiling: `continue-on-error` keeps a
job's *failure* from failing the run, but it does not keep a **timed-out** job
from cancelling the run's conclusion. So the roll-up field cannot distinguish
"a gating job died" from "an allowed-failure job ran the clock out." **Read the
per-job conclusions — `gh run view <id> --json jobs` — and check them against
the workflow's own list of gating jobs. Never authorize a push, or open a `CI
RED` item, off the run-level conclusion alone.** (The same reading also
surfaces a real cost: `atspi` used to fail fast and now times out, which is
worth its own note on the item that owns it.)

‼ **A PUSH MUST NEVER SHARE A SHELL CHAIN WITH THE CHECK THAT AUTHORIZES IT.**
Measured 2026-08-07: three gates and a `git push` went into one command block as
separate statements, so each printed its own `EXIT=`, the health arm printed
`EXIT=1`, and the push ran anyway — the red result was *in the output I was
reading* and had already shipped by the time I read it. This is the same family
as grepping a gate's text instead of its status, and neither is fixed by being
more careful: **the push goes in its own tool call, made after the check's exit
status has been read.** (Here the health failure was a flake and the rerun on
the identical sha was clean, so nothing bad shipped. That is luck, not process
— a chain like that will ship a genuine failure the first time it meets one.)

**And a green rerun does not make the first failure noise — read what it said.**
The arm that failed was one of `test-native-gate.sh`'s own laws, reporting that
it *"could not find the SIGINT probe's vitals-loop child, so this law would
prove nothing"*: a probe that polled for the gate's marker in a loop and then
read the vitals pid **once, immediately**, racing a separate fork. It lost that
race because a worker lane was compiling on the same host — so the flake
appears exactly when the orchestrator is busiest, and it aborts a law that was
about to pass. `find_vitals_pid` now polls (bounded, with an early-out on a
dead gate). **A self-test that says it cannot prove its law is a defect report
about the test, and it is worth the ten minutes then rather than the next four
times it fires.**

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
