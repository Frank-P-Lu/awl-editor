#!/usr/bin/env bash
# External behavioral law for reap-orphaned-gates.sh, against a disposable Git
# repository and real (but disposable) sleeper processes standing in for
# native-gate.sh. AWL_REAP_GATES_PS_COMMAND replaces `ps -A`, so this test
# never lets the real host's own native-gate.sh processes — which this repo
# routinely has running for real, from concurrent worker lanes — leak into
# the classification. Without that substitution the tool would have no way
# to tell a fixture PID from a live lane's, and this test would be unsafe to
# run at all.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-reap-gates-test.XXXXXX")"
REPO="$WORK/repo"

declare -a spawned_pids=()
cleanup() {
  local pid
  for pid in "${spawned_pids[@]:-}"; do
    [[ -n "$pid" ]] && kill -KILL "$pid" 2>/dev/null || true
  done
  if [[ -d "$REPO/.git" ]]; then
    git -C "$REPO" worktree list --porcelain 2>/dev/null \
      | awk '/^worktree /{print $2}' | while read -r wt; do
        [[ "$wt" == "$REPO" ]] && continue
        git -C "$REPO" worktree remove --force "$wt" >/dev/null 2>&1 || true
      done
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT

fail() { echo "test-reap-orphaned-gates: $1" >&2; exit 1; }

# ── Disposable repo: main + three lanes ──────────────────────────────────
mkdir -p "$REPO"
git -C "$REPO" init --quiet --initial-branch=main
git -C "$REPO" config user.email test@example.com
git -C "$REPO" config user.name "Test"
echo hello >"$REPO/f.txt"
git -C "$REPO" add f.txt
git -C "$REPO" commit --quiet -m "initial"

# Lane A: merged into main, worktree still present on disk — an orphan by
# the "branch already merged" signal.
git -C "$REPO" branch lane-merged
worktree_merged="$WORK/lane-merged"
git -C "$REPO" worktree add --quiet "$worktree_merged" lane-merged >/dev/null

# Lane B: unmerged, worktree present — a live lane, never a candidate.
git -C "$REPO" checkout --quiet -b lane-active
echo change >"$REPO/f.txt"
git -C "$REPO" add f.txt
git -C "$REPO" commit --quiet -m "lane work in progress"
git -C "$REPO" checkout --quiet main
worktree_active="$WORK/lane-active"
git -C "$REPO" worktree add --quiet "$worktree_active" lane-active >/dev/null

# Lane C: worktree removed outright (the exact case this item's diagnosis
# found: a directory that no longer exists but whose process still names it
# as cwd). Simulated with a plain directory never registered with git —
# lsof still reports it as a real cwd for a real process, same as the
# genuinely-removed worktrees found on the live host.
worktree_gone="$WORK/lane-gone"
mkdir -p "$worktree_gone"

# ── Spawn real sleeper processes claiming to be native-gate.sh ──────────
spawn_fixture() {
  local dir="$1" var="$2"
  ( cd "$dir" && exec -a "bash /whatever/scripts/native-gate.sh" sleep 300 ) &
  local pid=$!
  spawned_pids+=("$pid")
  eval "$var=$pid"
}
spawn_fixture "$worktree_merged" pid_merged
spawn_fixture "$worktree_active" pid_active
spawn_fixture "$worktree_gone" pid_gone

# `exec -a` sets argv[0] but macOS `ps -ww -o command=` reports the real
# executable path, not the spoofed argv[0] — so the fixture's ps stand-in
# fabricates the command text the real tool would see, keyed off real pids.
ps_stub="$WORK/ps-stub.sh"
cat >"$ps_stub" <<EOF
#!/usr/bin/env bash
printf '%s bash /whatever/scripts/native-gate.sh\n' "$pid_merged"
printf '%s bash /whatever/scripts/native-gate.sh\n' "$pid_active"
printf '%s bash /whatever/scripts/native-gate.sh\n' "$pid_gone"
EOF
chmod +x "$ps_stub"

# Let the sleepers actually start before lsof is asked for their cwd.
sleep 0.3

run_tool() {
  AWL_REAP_GATES_ROOT="$REPO" \
    AWL_REAP_GATES_PS_COMMAND="$ps_stub" \
    "$ROOT/.orchestrator/reap-orphaned-gates.sh" "$@"
}

# ── Dry run: classification only, nothing killed ─────────────────────────
set +e
output="$(run_tool 2>&1)"
status=$?
set -e
(( status == 0 )) || fail "dry run exited $status: $output"

echo "$output" | grep -Fq "pid=$pid_merged" \
  || fail "merged-branch lane was not reported as an orphan: $output"
echo "$output" | grep -Fq "branch lane-merged is already merged into main" \
  || fail "merged-branch lane's reason did not name the merged branch: $output"
echo "$output" | grep -Fq "pid=$pid_gone" \
  || fail "removed-worktree lane was not reported as an orphan: $output"
echo "$output" | grep -Fq "worktree no longer registered with git" \
  || fail "removed-worktree lane's reason did not name deregistration: $output"
if echo "$output" | grep -Fq "pid=$pid_active"; then
  fail "an unmerged, still-registered lane was flagged as an orphan: $output"
fi
echo "$output" | grep -Fq "2 orphan(s) found; rerun with --kill" \
  || fail "dry run did not report exactly 2 orphans or did not name --kill: $output"

for pid in "$pid_merged" "$pid_active" "$pid_gone"; do
  kill -0 "$pid" 2>/dev/null || fail "a dry run killed pid=$pid — it must only report"
done

echo "test-reap-orphaned-gates: a merged-and-present lane and a removed-worktree lane are both flagged; an active lane is not"

# ── --kill retires only the orphans, never the active lane ───────────────
set +e
output="$(run_tool --kill 2>&1)"
status=$?
set -e
(( status == 0 )) || fail "--kill run exited $status: $output"
echo "$output" | grep -Fq "retired 2 orphaned gate(s)" \
  || fail "--kill did not report retiring exactly 2 gates: $output"

sleep 1
kill -0 "$pid_merged" 2>/dev/null && fail "merged-branch lane survived --kill"
kill -0 "$pid_gone" 2>/dev/null && fail "removed-worktree lane survived --kill"
kill -0 "$pid_active" 2>/dev/null || fail "the active lane was killed — --kill must never touch a live lane"

echo "test-reap-orphaned-gates: --kill retires exactly the two orphans and leaves the active lane's process alive"

# ── Descendants are retired too, not just the top-level script ───────────
# native-gate.sh launches each convention as its OWN process-group leader
# (`set -m`), so a group-directed signal at the top-level script would miss
# them; only walking real ancestry (as the tool does) reaches a child like
# this. The fixture's "top" process is a real parent of a real "child" sleep.
descendant_worktree="$WORK/lane-descendant"
mkdir -p "$descendant_worktree"
( cd "$descendant_worktree" && exec -a "bash /whatever/scripts/native-gate.sh" bash -c 'sleep 300 & wait' ) &
top_pid=$!
spawned_pids+=("$top_pid")
sleep 0.3
child_pid="$(pgrep -P "$top_pid" | head -n1)"
[[ -n "$child_pid" ]] || fail "fixture's own child never started, so this law proves nothing"
spawned_pids+=("$child_pid")

ps_stub2="$WORK/ps-stub2.sh"
cat >"$ps_stub2" <<EOF
#!/usr/bin/env bash
printf '%s bash /whatever/scripts/native-gate.sh\n' "$top_pid"
EOF
chmod +x "$ps_stub2"

set +e
output="$(AWL_REAP_GATES_ROOT="$REPO" AWL_REAP_GATES_PS_COMMAND="$ps_stub2" \
  "$ROOT/.orchestrator/reap-orphaned-gates.sh" --kill 2>&1)"
status=$?
set -e
(( status == 0 )) || fail "descendant --kill run exited $status: $output"

sleep 1
kill -0 "$top_pid" 2>/dev/null && fail "orphan top-level script survived --kill"
kill -0 "$child_pid" 2>/dev/null && fail "orphan's child process survived --kill — descendants must be retired too"

echo "test-reap-orphaned-gates: --kill retires an orphan's descendant process, not only the top-level script"

# ── The root checkout: item 270's marker is the only discriminator ───────
# None of the three worktree signals above can ever fire against the repo
# root itself — it always exists, is always registered with git, and its
# branch is never "already merged into main" the way a lane's is. This item
# taught the tool to read .orchestrator/native-gate.marker instead: absent,
# or present but naming a pid that is not alive, means no root gate is live,
# so a native-gate.sh-tagged process found there (the marker's own writer or
# a leaked vitals-loop child, indistinguishable by evidence alone) is an
# orphan by construction; a marker naming a genuinely live pid still
# protects it, same as the merge train always has been.
spawn_root_fixture() {
  local var="$1"
  ( cd "$REPO" && exec -a "bash /whatever/scripts/native-gate.sh" sleep 300 ) &
  local pid=$!
  spawned_pids+=("$pid")
  eval "$var=$pid"
}

root_ps_stub_for() {
  local pid="$1" stub="$2"
  cat >"$stub" <<EOF
#!/usr/bin/env bash
printf '%s bash /whatever/scripts/native-gate.sh\n' "$pid"
EOF
  chmod +x "$stub"
}

# Case 1: no marker file at all.
spawn_root_fixture pid_root_absent
sleep 0.2
ps_stub_root_absent="$WORK/ps-stub-root-absent.sh"
root_ps_stub_for "$pid_root_absent" "$ps_stub_root_absent"
marker_absent="$WORK/marker-absent-never-created"

set +e
output="$(AWL_REAP_GATES_ROOT="$REPO" AWL_REAP_GATES_PS_COMMAND="$ps_stub_root_absent" \
  AWL_REAP_GATES_MARKER="$marker_absent" \
  "$ROOT/.orchestrator/reap-orphaned-gates.sh" 2>&1)"
status=$?
set -e
(( status == 0 )) || fail "root/no-marker dry run exited $status: $output"
echo "$output" | grep -Fq "pid=$pid_root_absent" \
  || fail "a root-tree gate with no marker was not reported as an orphan: $output"
echo "$output" | grep -Fq "root gate marker absent or stale" \
  || fail "a root-tree orphan's reason did not name the marker rule: $output"
kill -0 "$pid_root_absent" 2>/dev/null || fail "a dry run killed pid=$pid_root_absent — it must only report"

set +e
output="$(AWL_REAP_GATES_ROOT="$REPO" AWL_REAP_GATES_PS_COMMAND="$ps_stub_root_absent" \
  AWL_REAP_GATES_MARKER="$marker_absent" \
  "$ROOT/.orchestrator/reap-orphaned-gates.sh" --kill 2>&1)"
status=$?
set -e
(( status == 0 )) || fail "root/no-marker --kill run exited $status: $output"
sleep 1
kill -0 "$pid_root_absent" 2>/dev/null && fail "a root-tree gate with no live marker survived --kill"

echo "test-reap-orphaned-gates: a root-tree native-gate.sh with no marker file is an orphan, reported then retired"

# Case 2: marker present and names a genuinely live pid — protected, exactly
# like the merge train always was, even under --kill.
spawn_root_fixture pid_root_live
sleep 0.2
ps_stub_root_live="$WORK/ps-stub-root-live.sh"
root_ps_stub_for "$pid_root_live" "$ps_stub_root_live"
marker_live="$WORK/marker-live"
printf 'pid=%s start_commit=deadbeefdeadbeefdeadbeefdeadbeefdeadbeef start_epoch=0\n' "$$" >"$marker_live"

set +e
output="$(AWL_REAP_GATES_ROOT="$REPO" AWL_REAP_GATES_PS_COMMAND="$ps_stub_root_live" \
  AWL_REAP_GATES_MARKER="$marker_live" \
  "$ROOT/.orchestrator/reap-orphaned-gates.sh" --kill 2>&1)"
status=$?
set -e
(( status == 0 )) || fail "root/live-marker --kill run exited $status: $output"
if echo "$output" | grep -Fq "pid=$pid_root_live"; then
  fail "a root-tree gate whose marker names a live pid was flagged as an orphan: $output"
fi
sleep 1
kill -0 "$pid_root_live" 2>/dev/null \
  || fail "a root-tree gate protected by a live marker was killed anyway"
kill -TERM "$pid_root_live" 2>/dev/null || true

echo "test-reap-orphaned-gates: a root-tree marker naming a live pid protects the gate, even under --kill"

# Case 3: marker present but stale — names a pid that is provably dead.
spawn_root_fixture pid_root_stale
sleep 0.2
ps_stub_root_stale="$WORK/ps-stub-root-stale.sh"
root_ps_stub_for "$pid_root_stale" "$ps_stub_root_stale"
marker_stale="$WORK/marker-stale"
( exit 0 ) &
dead_pid=$!
wait "$dead_pid" 2>/dev/null || true
printf 'pid=%s start_commit=deadbeefdeadbeefdeadbeefdeadbeefdeadbeef start_epoch=0\n' "$dead_pid" >"$marker_stale"

set +e
output="$(AWL_REAP_GATES_ROOT="$REPO" AWL_REAP_GATES_PS_COMMAND="$ps_stub_root_stale" \
  AWL_REAP_GATES_MARKER="$marker_stale" \
  "$ROOT/.orchestrator/reap-orphaned-gates.sh" --kill 2>&1)"
status=$?
set -e
(( status == 0 )) || fail "root/stale-marker --kill run exited $status: $output"
echo "$output" | grep -Fq "pid=$pid_root_stale" \
  || fail "a root-tree gate with a stale marker was not reported as an orphan: $output"
sleep 1
kill -0 "$pid_root_stale" 2>/dev/null && fail "a root-tree gate with a stale marker survived --kill"

echo "test-reap-orphaned-gates: a root-tree marker naming a dead pid is stale — the gate it once described is retired like any other orphan"
