# Worker operations

Read before concurrent builds, recovering a worker, or cleaning worktrees.
[Orchestration guide](README.md) owns dispatch; paths below are repo-relative.

## Build budget and gate lifetime

Every concurrent worker build/check runs through `.orchestrator/worker-build.sh`.
It owns `CARGO_BUILD_JOBS=2` and `RUST_TEST_THREADS=1`; workers and gates must not
set competing values. These bounds are per Cargo invocation. Two concurrent
conventions multiply a gate's possible compilation demand to four jobs and test
execution to two threads; build jobs alone do not bound test threads.

The practical host ceiling is six to eight lanes, subject to measured capacity,
not a target. Full native gates enter one shared arbiter, so they do not run
concurrently. Targeted checks stay outside that queue. Do not launch duplicate full
gates or wait for a whole wave to quiesce before using the canonical gate door.
The isolated merge-train gate, CI, and a developer's lone build use the normal
hardware-adaptive scripts without the worker wrapper.

```sh
.orchestrator/worker-build.sh scripts/native-gate.sh
.orchestrator/worker-build.sh scripts/web-smoke.sh
```

Include the wrapper receipt in the report. Give long commands explicit adequate
timeouts. If the tool yields a running command, retain its handle and wait for its
exit status; auto-backgrounding is not completion. Do not launch another overlapping
gate to compensate. Do not pipe a gate through `grep` or another command that hides
its exit status.

`native-gate.marker` names the arbiter holder's PID, start commit, and time. The
kernel admission lock carries ownership; stale marker text does not. EXIT/TERM/INT
cleanup removes the marker. The arbiter serializes gates, not commits: check whether
a live holder is testing the candidate you would change, and wait before changing it.

## Recover a worker

A completed turn does not establish that its background children have stopped.
Inspect the agent roster and explicitly stop the original before assigning a
replacement to the same worktree. Tell the replacement that inherited work is
unreviewed. One concurrent writer owns each durable worktree; shared trees are
reader-only. Do not put durable worktrees under `/private/tmp`.

When a worker returns only a status line, first inspect its branch and working tree:

```sh
git -C <worktree> log --oneline main..HEAD
git -C <worktree> status --short
```

Check its gate processes on the host as well. Request the specific missing report
fields. If its only work is uncommitted, stop or finish its gate before committing;
never rescue the work by changing HEAD underneath a live gate. Stage named paths.
A full final report with named unknowns is useful even if a receipt is outstanding.

Classify failures from evidence. Retry suspected incremental failures with
`CARGO_INCREMENTAL=0`; for SIGKILL without a test failure, check memory and run the
gate alone. A tool-shell exit 144 can indicate the harness killed its shell:
inspect timeout and process evidence before blaming repository group cleanup.
`code-health.sh`'s caller-survival law checks that its probes leave the caller's
process group alive. Give it a sufficient timeout rather than detaching a subshell
and losing the exit status. A heartbeat/self-test failure is investigated, not
waived because the host is busy; a green rerun does not explain the first failure.

Terminate only owned processes or groups. Never kill awl by name. Use full command
lines (`pgrep -f` plus `ps -ww`) and exact PIDs; truncated command output cannot
establish that no orphan exists. Target test windows by unique title. TERM on an
owned orphaned gate group allows the arbiter's cleanup trap to release its slot.
`.orchestrator/reap-orphaned-gates.sh` reaps gates on merged branches; it does not
prove the producing agent is retired and must not trigger worktree deletion.

## Disk pressure and cleanup

`.orchestrator/disk-preflight.sh` is the serialized recovery owner, called by the
worker wrapper and the canonical native gate. It reads capacity above the 27 GiB
healthy fleet floor. Below it, it locks, rechecks, and invokes `scripts/sweep.sh 1`
for the caller's own worktree only. The minimum is 24 GiB; the 3 GiB recovery budget
is a measured ceiling, not promised reclaimed space. Read `sweep_yield_bytes` and
`reclaimed_bytes` in receipts. Do not bypass a refusal to reach the lane ceiling.

Active artifacts often reclaim nothing. A durable worktree is roughly 5 GiB, so
eight lanes need about 40 GiB plus shared artifacts and working room. Disk state,
not these estimates, controls admission. The lock uses inherited file descriptor
9; the kernel releases it when the holder exits or dies.

CI uses capacity-only preflight with a 2 GiB floor and does not install or run
cargo-sweep. Standalone wasm smoke remains portable; a worker launch inherits the
same preflight through its wrapper.

Remove merged worktrees only after their agents and builds have stopped, then
sweep. Removing a worktree does not remove its branch. `scripts/sweep.sh` prunes
only its current worktree, including stale debug/release incremental sessions.
Fleet-wide `--all-worktrees` cleanup is manual and requires all affected builds to
be idle. Never sweep a sibling lane's live compilation.

## Worktree locations and retirement

See [worktree lifecycle](../docs/worktree-lifecycle.md) for creation, review holds,
inventory and retirement.

## Safe shell and fixture writes

Write commit/merge messages with quoted heredocs so backticks and dollar signs
remain literal. `git commit -F -` reads stdin; `git merge -F` needs a real file.
Keep push in a separate tool call after inspecting check results.

Use a parser when modifying TOML and parse after each write. An escaped quote is
not the end of a string. A path restore from the index may restore already-staged
corruption; inspect the intended source revision and preserve unrelated changes
before restoring anything.

Place capture outputs outside the corpus a file picker enumerates. Use seeded
roots and explicit config; a changing fixture directory can produce false geometry
differences. Establish whether a reported interaction used keyboard, pointer, or
wheel before choosing its reproducer.
