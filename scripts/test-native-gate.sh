#!/usr/bin/env bash
# Exercise the gate's orchestration and failure semantics without compiling.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PYTHONDONTWRITEBYTECODE=1 python3 "$ROOT/scripts/test-native-test-shards.py"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-native-gate-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
# Every fixture gate inherits its own arbiter state. Besides keeping this test
# hermetic, that lets the concurrent-holder law choose a fresh marker without
# ever touching a real orchestrator's queue.
export AWL_NATIVE_GATE_MARKER="$WORK/default-marker"
export AWL_NATIVE_GATE_ARBITER_LOCK="$WORK/default-arbiter.lock"
# THE MENU-BAR AXIS MODE IS PINNED FOR EVERY LAW THAT IS NOT ABOUT IT. The gate
# chooses between a full-suite forced arm and the cheap name-filtered pair from
# the environment (`CI`), so leaving it ambient would make phase counts, shard
# counts and event tallies below differ between a developer's run of this file
# and CI's — a check whose configuration is itself untested. The two laws that
# ARE about the axis override this explicitly, including one that unsets it to
# prove the derivation.
export AWL_NATIVE_GATE_MENUBAR_FULL=0

PYTHONDONTWRITEBYTECODE=1 python3 - "$ROOT/scripts/native-test-shards.py" \
  >"$WORK/awl-test-list" <<'PY'
import importlib.util
import sys

spec = importlib.util.spec_from_file_location("shards", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
for hints in module.HINTS:
    for prefix in hints:
        print(prefix + "probe: test")
print("firstrun::tests::probe: test")
print("other::tests::remainder: test")
PY

cat >"$WORK/awl-test-bin" <<'EOF'
#!/usr/bin/env bash
set -eo pipefail
tests=()
while IFS= read -r test; do tests+=("${test%: test}"); done \
  <"$AWL_NATIVE_GATE_PROBE_TEST_LIST"
filters=()
skips=()
listing=0
while (( $# )); do
  case "$1" in
    --list) listing=1 ;;
    --format) shift ;;
    --skip) shift; skips+=("$1") ;;
    --*) : ;;
    *) filters+=("$1") ;;
  esac
  shift
done
selected=()
for test in "${tests[@]}"; do
  match=0
  (( ${#filters[@]} == 0 )) && match=1
  for filter in "${filters[@]}"; do [[ "$test" == *"$filter"* ]] && match=1; done
  for skip in "${skips[@]}"; do [[ "$test" == *"$skip"* ]] && match=0; done
  (( match )) && selected+=("$test")
done
if (( listing )); then
  for test in "${selected[@]}"; do printf '%s: test\n' "$test"; done
  exit 0
fi
printf 'shard %s %s\n' "${AWL_CONVENTION_FORCE:-unset}" "$$" >>"$AWL_NATIVE_GATE_PROBE_LOG"
# The MENU-BAR forcing as the SHARD saw it. The full-suite arm runs the shard
# binary directly rather than through Cargo, so the `menubar` lines the fake
# `cargo` writes cannot see it at all — and a law reading only those would report
# an unswept axis as swept the moment the arm stopped going through Cargo.
printf 'shardbar %s\n' "${AWL_MENU_BAR_FORCE:-unset}" >>"$AWL_NATIVE_GATE_PROBE_LOG"
printf '\nrunning %s tests\n' "${#selected[@]}"
printf 'test result: ok. %s passed; 0 failed; 0 ignored; 0 measured\n' "${#selected[@]}"
# A shard that fails under a forcing — the full-suite arm's own red, which no
# fixture keyed on Cargo can produce.
if [[ -n "${AWL_MENU_BAR_FORCE:-}" \
  && "$AWL_MENU_BAR_FORCE" == "${AWL_NATIVE_GATE_FAIL_MENU_BAR:-}" ]]; then
  exit "${AWL_NATIVE_GATE_FAIL_STATUS:-1}"
fi
EOF
chmod +x "$WORK/awl-test-bin"

cat >"$WORK/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
convention="${AWL_CONVENTION_FORCE:-canary}"
if [[ " $* " == *" --no-run "* ]]; then
  printf '{"reason":"compiler-artifact","target":{"kind":["bin"],"name":"awl"},"profile":{"test":true},"executable":"%s"}\n' "$AWL_NATIVE_GATE_PROBE_TEST_BINARY"
  printf '{"reason":"compiler-artifact","target":{"kind":["test"],"name":"native_gate_canary"},"profile":{"test":true},"executable":"%s"}\n' "$AWL_NATIVE_GATE_PROBE_TEST_BINARY"
  exit 0
fi
printf 'start %s\n' "$convention" >>"$AWL_NATIVE_GATE_PROBE_LOG"
printf 'threads %s %s\n' "$convention" "${RUST_TEST_THREADS:-unset}" >>"$AWL_NATIVE_GATE_PROBE_LOG"
# The MENU-BAR axis, recorded per invocation. `unset` is a real answer and the
# one the canary and both conventions must give: if a convention inherited a
# forcing, the axis would be a property of the convention arms rather than its
# own two, and the pair would be measuring one branch twice.
printf 'menubar %s\n' "${AWL_MENU_BAR_FORCE:-unset}" >>"$AWL_NATIVE_GATE_PROBE_LOG"
# Cargo and libtest emit these lines with SGR colour on a GitHub runner (the
# 2026-08-02 mac log is full of them), so the fixture emits them coloured too:
# a phase matcher anchored to a bare "Finished" would pass here and see nothing
# in CI. The deliberately hostile test NAME carries the words a phase marker
# keys on — a per-test line must never be able to forge one.
if [[ -n "${AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT:-}" ]]; then
  printf '\033[1m\033[92m   Compiling\033[0m awl v0.1.0\n'
  printf '\033[1m\033[92m    Finished\033[0m `test` profile [optimized + debuginfo] target(s) in 1m 22s\n'
  printf '\033[1m\033[92m     Running\033[0m unittests src/main.rs (target/debug/deps/awl-a623f1caab4)\n'
  printf '\nrunning 3484 tests\n'
  printf 'test render::Running_tests::a_name_with_(parens) ... ok\n'
  printf 'test render::a_name_with_Finished_and_target(s) in_it ... ok\n'
  printf 'test result: ok. 3484 passed; 0 failed; 0 ignored; 0 measured\n'
  printf '\033[1m\033[92m     Running\033[0m tests/harness.rs (target/debug/deps/harness-0f0f0f0f)\n'
  printf 'running 2 tests\n'
  printf 'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured\n'
fi
# libtest prints "test NAME ... " BEFORE running the test and its result after,
# so a test that never returns leaves an UNTERMINATED line naming itself. That
# fragment is the single most valuable line in a hung run's log, and it is only
# ever flushed when the pipe closes.
if [[ -n "${AWL_NATIVE_GATE_PROBE_HANG_LINE:-}" ]]; then
  printf 'test the_test_that_never_returned ... '
fi
# A grandchild that a bare `kill $cargo_pid` cannot reach. On a real runner this
# is the test binary itself, and it is what the 2026-08-02 job cleanup had to
# reap by hand after the gate had already exited.
if [[ -n "${AWL_NATIVE_GATE_PROBE_ORPHAN_FILE:-}" ]]; then
  ( trap '' TERM; sleep 600 ) &
  printf '%s\n' "$!" >>"$AWL_NATIVE_GATE_PROBE_ORPHAN_FILE"
fi
if [[ "$convention" == canary ]]; then
  sleep "${AWL_NATIVE_GATE_PROBE_CANARY_SLEEP:-0}"
elif [[ -n "${AWL_NATIVE_GATE_PROBE_SPIN_SECONDS:-}" ]]; then
  # A LIVELOCK, as opposed to the sleeping fixture's deadlock. Both leave memory
  # flat and both stop producing output; only CPU tells them apart, so the
  # fixture has to be able to be either. The spin is bash's own `SECONDS`
  # builtin and an empty loop body: no fork per iteration, so the CPU it burns
  # is attributed to THIS pid, which is the pid the heartbeat has to name.
  # The spinner is a CHILD born after a delay, so it is deterministically
  # absent from the heartbeat sample before it and present in the next — the
  # NEWCOMER case, which the probe must measure over the process's own age
  # rather than drop. Letting the conventions themselves be the newcomers was
  # flaky: whether they beat the gate's baseline sample was a fork race.
  sleep "${AWL_NATIVE_GATE_PROBE_SPIN_DELAY:-0}"
  bash -c 'printf "%s\n" "$$" >>"$1"; SECONDS=0; while (( SECONDS < $2 )); do :; done' \
    _ "$AWL_NATIVE_GATE_PROBE_SPIN_PID_FILE" "$AWL_NATIVE_GATE_PROBE_SPIN_SECONDS"
else
  sleep "${AWL_NATIVE_GATE_PROBE_SLEEP:-0.2}"
fi
printf 'finish %s\n' "$convention" >>"$AWL_NATIVE_GATE_PROBE_LOG"
if [[ "$convention" == "${AWL_NATIVE_GATE_FAIL_CONVENTION:-}" ]]; then
  exit "${AWL_NATIVE_GATE_FAIL_STATUS:-1}"
fi
# A MENU-BAR arm failing, keyed on the forcing rather than the convention. Both
# guards read the same `AWL_NATIVE_GATE_FAIL_STATUS`, and the `-n` test keeps
# this inert for the canary and the two conventions, which carry no forcing.
if [[ -n "${AWL_MENU_BAR_FORCE:-}" \
  && "$AWL_MENU_BAR_FORCE" == "${AWL_NATIVE_GATE_FAIL_MENU_BAR:-}" ]]; then
  exit "${AWL_NATIVE_GATE_FAIL_STATUS:-1}"
fi
EOF
chmod +x "$WORK/cargo"

cat >"$WORK/free-oracle" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' $((40 * 1024 * 1024 * 1024))
EOF
chmod +x "$WORK/free-oracle"

cat >"$WORK/git" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == 'rev-parse --path-format=absolute --git-common-dir' \
  && -n "${AWL_NATIVE_GATE_PROBE_COMMON_GIT_DIR:-}" ]]; then
  printf '%s\n' "$AWL_NATIVE_GATE_PROBE_COMMON_GIT_DIR"
else
  /usr/bin/git "$@"
fi
EOF
chmod +x "$WORK/git"

run_probe() {
  local failing="${1:-}" status="${2:-0}" output="$WORK/output-${1:-success}"
  : >"$WORK/events"
  set +e
  PATH="$WORK:$PATH" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
    AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
    AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
    AWL_NATIVE_GATE_FAIL_CONVENTION="$failing" \
    AWL_NATIVE_GATE_FAIL_STATUS="$status" \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-${1:-success}" \
    "$ROOT/scripts/native-gate.sh" >"$output" 2>&1
  probe_status=$?
  set -e
}

# The bound is a launch seam, so it is asserted where it lands: in the child's
# environment. The machine axis swept here is the one the gate exists for — a
# hosted runner far smaller than the host the gate was written on — plus the
# case a core count alone cannot see, a wide box with a starved RAM budget.
probe_bound() {
  local cpus="$1" mem_bytes="$2" caller="${3:-}" label="bound-$1-$2-${3:-default}"
  : >"$WORK/events"
  set +e
  env -u RUST_TEST_THREADS \
    PATH="$WORK:$PATH" \
    ${caller:+RUST_TEST_THREADS="$caller"} \
    AWL_NATIVE_GATE_CPUS="$cpus" \
    AWL_NATIVE_GATE_MEM_BYTES="$mem_bytes" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
    AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
    AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-$label" \
    "$ROOT/scripts/native-gate.sh" >"$WORK/output-$label" 2>&1
  probe_status=$?
  set -e
  bound_output="$WORK/output-$label"
}

assert_bound() {
  local cpus="$1" mem_bytes="$2" caller="$3" expected="$4" convention seen
  probe_bound "$cpus" "$mem_bytes" "$caller"
  (( probe_status == 0 )) || {
    echo "test-native-gate: bound probe cpus=$cpus mem=$mem_bytes failed ($probe_status)" >&2
    tail -n 20 "$bound_output" >&2
    exit 1
  }
  for convention in mac linux; do
    seen="$(awk -v c="$convention" '$1 == "threads" && $2 == c { print $3 }' "$WORK/events")"
    [[ "$seen" == "$expected" ]] || {
      echo "test-native-gate: cpus=$cpus mem=$mem_bytes caller=${caller:-none} gave the $convention convention RUST_TEST_THREADS=$seen, expected $expected" >&2
      exit 1
    }
  done
  grep -Fq "native-gate-env cpus=$cpus mem_bytes=$mem_bytes conventions=2 test_threads=$expected" "$bound_output" || {
    echo "test-native-gate: machine receipt did not name cpus=$cpus mem=$mem_bytes test_threads=$expected" >&2
    exit 1
  }
}

# A three-vCPU hosted runner gets one thread per convention (two on three
# cores); the ten-core dev host keeps five apiece, which is its core count in
# total rather than twice it; a wide box with 2 GiB is bounded by RAM, not
# cores; and a caller that states a value owns it.
assert_bound 3 $((7 * 1024 * 1024 * 1024)) "" 1
assert_bound 10 $((64 * 1024 * 1024 * 1024)) "" 5
assert_bound 64 $((2 * 1024 * 1024 * 1024)) "" 16
assert_bound 10 $((64 * 1024 * 1024 * 1024)) 3 3

echo "test-native-gate: the thread bound tracks cores and RAM, defers to a caller, and is receipted"

# A gate that outruns its budget must fail LOUDLY and IN BAND. The whole point
# is a starved CI runner that dies mid-step and uploads no log at all: a gate
# that ends itself first leaves a log behind that names what happened.
: >"$WORK/events"
set +e
env -u RUST_TEST_THREADS \
  PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_BUDGET_SECONDS=1 \
  AWL_NATIVE_GATE_PROBE_SLEEP=60 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-budget" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-budget" 2>&1
budget_status=$?
set -e
(( budget_status == 1 )) || {
  echo "test-native-gate: an exhausted budget returned $budget_status, expected gate status 1" >&2
  exit 1
}
grep -Fq 'native-gate: ABORTED on its 1s budget' "$WORK/output-budget" || {
  echo "test-native-gate: an exhausted budget did not name itself in the output" >&2
  exit 1
}
if grep -Fq 'native-gate-receipt' "$WORK/output-budget"; then
  echo "test-native-gate: an exhausted budget leaked a receipt" >&2
  exit 1
fi

echo "test-native-gate: an exhausted budget ends the run in band, by name, with no receipt"

# A general probe runner for the phase/budget laws below. Every one of them
# asserts against the gate's real output, on this host, with a fixture that
# emits the exact line shapes the 2026-08-02 mac runner emitted.
probe() {
  local label="$1" probe_started
  shift
  : >"$WORK/events"
  probe_started="$(date +%s)"
  set +e
  env -u RUST_TEST_THREADS -u AWL_NATIVE_GATE_BUDGET_SECONDS -u AWL_NATIVE_GATE_DEADLINE_EPOCH \
    PATH="$WORK:$PATH" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
    AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
    AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-$label" \
    "$@" \
    "$ROOT/scripts/native-gate.sh" >"$WORK/output-$label" 2>&1
  probe_status=$?
  set -e
  probe_elapsed=$(( $(date +%s) - probe_started ))
  probe_output="$WORK/output-$label"
}

require() {
  grep -Fq "$2" "$probe_output" || {
    echo "test-native-gate: $1 — missing from the gate's output: $2" >&2
    exit 1
  }
}

refuse() {
  if grep -Fq "$2" "$probe_output"; then
    echo "test-native-gate: $1 — the gate emitted what it must not: $2" >&2
    exit 1
  fi
}

# The direct child of $1 whose own child is a `sleep` process. No budget is
# armed in the callers below, so gate_vitals_loop is the only child that ever
# spawns one — this is the same evidence a live host's `ps -ww` showed for
# the leaked orphans this item was opened against: a pid whose one child is
# sitting in `sleep`. Bash-3.2-safe (no `mapfile`), matching this repo's
# other portability notes.
#
# The search POLLS, for the same reason the marker waits are loops: the marker
# and the vitals loop are two separate spawns, so a caller that reads the pid
# the instant the marker lands is racing the fork — and the loser reports
# "could not find the vitals-loop child", aborting a law that was about to
# pass. It is a probe of the machine's scheduling, not of the gate.
find_vitals_pid() {
  local gate_pid="$1" child grandchild attempt
  for attempt in $(seq 1 50); do
    for child in $(pgrep -P "$gate_pid" 2>/dev/null || true); do
      for grandchild in $(pgrep -P "$child" 2>/dev/null || true); do
        if ps -ww -o command= -p "$grandchild" 2>/dev/null | grep -q '^sleep '; then
          printf '%s\n' "$child"
          return 0
        fi
      done
    done
    kill -0 "$gate_pid" 2>/dev/null || return 1
    sleep 0.1
  done
  return 1
}

# ── Per-phase timing ─────────────────────────────────────────────────────────
# The question this exists to answer is whether a 40-minute step is COMPILING
# test harnesses or RUNNING tests. Cargo already announces both boundaries; the
# gate must stamp them, per convention, and it must not be possible for a per-
# test line to forge one — the fixture's test is named
# `Running_tests::a_name_with_(parens)_and_Finished_target(s) in`, which
# contains every token the matchers key on.
probe phases AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT=1
(( probe_status == 0 )) || { echo "test-native-gate: phase probe failed ($probe_status)" >&2; exit 1; }
for convention in mac linux; do
  require "phase timing" "native-gate-phase label=$convention event=compile-finished elapsed_seconds="
  require "phase timing" "native-gate-phase label=$convention event=first-tests-running elapsed_seconds="
  require "phase timing" "native-gate-phase label=$convention event=suite-end elapsed_seconds="
  require "phase timing" "target=awl-a623f1caab4"
  require "phase timing" "target=harness-0f0f0f0f"
  require "phase timing" "detail=ok. 3484 passed"
  # Both conventions write one stdout. Without a label, "which convention got
  # where" is not recoverable from the log at all — which is what made the one
  # surviving mac log take line-by-line archaeology to read.
  require "convention labelling" "$convention| running 3484 tests"
done
require "phase timing" "native-gate-phase label=canary event=begin"
require "phase timing" "native-gate-phase label=canary event=end elapsed_seconds="
# The fixture runs two real targets and prints two hostile test names: one
# carrying "Running_tests" and "(parens)", one carrying "Finished" and
# "target(s) in". Counting is the assertion — a marker that exists is not the
# same as a marker that is right, and an over-count is exactly how a per-test
# line quietly becomes a phase boundary.
for convention in mac linux; do
  for event in target-start:2 compile-finished:1 first-tests-running:1; do
    seen="$(grep -c "native-gate-phase label=$convention event=${event%%:*} " "$probe_output" || true)"
    [[ "$seen" == "${event##*:}" ]] || {
      echo "test-native-gate: $convention stamped $seen ${event%%:*} markers over ${event##*:} real ones — a test NAME forged a phase boundary" >&2
      exit 1
    }
  done
done

echo "test-native-gate: every phase boundary is stamped per convention, and a test name cannot forge one"

# ── The line a hang leaves behind ────────────────────────────────────────────
# The only thing that identified 2026-08-02's hang was libtest's unterminated
# "test NAME ... " fragment, and it survived by luck: an interleaved heartbeat
# write happened to flush it. The gate must produce it on purpose.
probe hang-line AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=30 \
  AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT=1 AWL_NATIVE_GATE_PROBE_HANG_LINE=1
(( probe_status == 1 )) || { echo "test-native-gate: hang-line probe returned $probe_status" >&2; exit 1; }
for convention in mac linux; do
  require "hang line" "$convention| test the_test_that_never_returned ... "
done

echo "test-native-gate: a killed convention still flushes the unterminated line naming the test that hung"

# ── The budget covers the canary ─────────────────────────────────────────────
# The first draft armed the budget only after the canary had returned, leaving
# the whole dependency-and-library compile — the slowest phase on a cold hosted
# runner — with no watchdog at all.
probe canary-budget AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_CANARY_SLEEP=30
(( probe_status == 1 )) || {
  echo "test-native-gate: a canary that outran the budget returned $probe_status, expected 1" >&2
  exit 1
}
require "canary budget" "budget expired during the canary phase"
refuse "canary budget" "native-gate-receipt"
grep -Fq 'finish canary' "$WORK/events" && {
  echo "test-native-gate: the budget did not actually end the canary — it ran to completion" >&2
  exit 1
}
# The fixture hangs for 30 s. A gate whose watchdog does not reach this phase
# does not fail — it WAITS, which is the whole defect, so the law is stated in
# wall-clock: a 2 s budget plus its 5 s escalation must land nowhere near 30.
(( probe_elapsed < 20 )) || {
  echo "test-native-gate: the gate took ${probe_elapsed}s to end a 2s budget — it waited out the hang instead of ending it" >&2
  exit 1
}

echo "test-native-gate: the budget reaches the canary phase, not only the concurrent suites"

# ── The budget ends the whole process GROUP ──────────────────────────────────
# `kill $cargo_pid` retires `env … cargo test` and nothing below it. On
# A gate cleanup can leave survivors after it exits; a survivor holds the step's stdout, and a GitHub
# step does not conclude while that pipe is open. The fixture's grandchild
# IGNORES SIGTERM on purpose, so only a group-directed KILL can retire it — a
# gate that merely signalled its direct children would leave it running.
: >"$WORK/orphans"
probe group-kill AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=30 \
  AWL_NATIVE_GATE_PROBE_ORPHAN_FILE="$WORK/orphans"
(( probe_status == 1 )) || {
  echo "test-native-gate: the group-kill probe returned $probe_status, expected 1" >&2
  exit 1
}
[[ -s "$WORK/orphans" ]] || {
  echo "test-native-gate: the fixture spawned no grandchild, so this law proved nothing" >&2
  exit 1
}
survivors=""
while read -r orphan; do
  [[ -n "$orphan" ]] || continue
  if kill -0 "$orphan" 2>/dev/null; then
    survivors="$survivors $orphan"
    kill -KILL "$orphan" 2>/dev/null || true
  fi
done <"$WORK/orphans"
[[ -z "$survivors" ]] || {
  echo "test-native-gate: the budget left grandchildren alive ($survivors) — they hold the CI step's stdout open" >&2
  exit 1
}
require "group kill" "native-gate-budget-proc"

echo "test-native-gate: an exhausted budget retires every descendant, not just the process it launched"

# ── The budget is anchored to the caller's clock, not only to the gate's ─────
# The runner's death clock starts at job step 1; this script's starts whenever
# the earlier steps happen to have finished. On 2026-08-02 the same 2400 s
# duration meant job-minute 41 on a cold cache and job-minute 42 on a hot one —
# and the runner was lost at 53. An absolute deadline pins the end of the gate
# to the clock that is actually killing it.
now="$(date +%s)"
probe deadline-only AWL_NATIVE_GATE_DEADLINE_EPOCH=$(( now + 2 )) AWL_NATIVE_GATE_PROBE_SLEEP=30
(( probe_status == 1 )) || {
  echo "test-native-gate: an absolute deadline alone did not end the gate (status $probe_status)" >&2
  exit 1
}
require "deadline" "budget_source=deadline"
refuse "deadline" "native-gate-receipt"

now="$(date +%s)"
probe deadline-wins AWL_NATIVE_GATE_BUDGET_SECONDS=3600 \
  AWL_NATIVE_GATE_DEADLINE_EPOCH=$(( now + 2 )) AWL_NATIVE_GATE_PROBE_SLEEP=30
(( probe_status == 1 )) || {
  echo "test-native-gate: a near deadline lost to a distant duration (status $probe_status)" >&2
  exit 1
}
require "deadline" "budget_source=deadline"

probe duration-wins AWL_NATIVE_GATE_BUDGET_SECONDS=2 \
  AWL_NATIVE_GATE_DEADLINE_EPOCH=$(( now + 3600 )) AWL_NATIVE_GATE_PROBE_SLEEP=30
(( probe_status == 1 )) || {
  echo "test-native-gate: a near duration lost to a distant deadline (status $probe_status)" >&2
  exit 1
}
require "deadline" "budget_seconds=2 budget_source=duration"

echo "test-native-gate: the budget takes whichever of its duration and its absolute deadline comes first"

# ── The heartbeat and the abort both name where each convention got to ───────
# Reconstructing the one surviving mac log meant reading 6500 interleaved lines
# to find the test that never returned. The gate must say it outright, while it
# is still running and again when it gives up.
probe last-progress AWL_NATIVE_GATE_VITALS_SECONDS=1 AWL_NATIVE_GATE_PROBE_SLEEP=4 \
  AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT=1
(( probe_status == 0 )) || { echo "test-native-gate: last-progress probe failed ($probe_status)" >&2; exit 1; }
require "heartbeat progress" "mac_last=[test result: ok. 2 passed"
require "heartbeat progress" "linux_last=[test result: ok. 2 passed"

probe abort-progress AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=30 \
  AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT=1
(( probe_status == 1 )) || { echo "test-native-gate: abort-progress probe returned $probe_status" >&2; exit 1; }
require "abort progress" "native-gate-budget-last label=mac line=[test result: ok. 2 passed"
require "abort progress" "native-gate-budget-last label=linux line=[test result: ok. 2 passed"

echo "test-native-gate: the heartbeat and the abort both name the last line each convention reached"

# The heartbeat is the only thing that will describe the machine while a slow
# gate is still alive, so a heartbeat carrying a placeholder is worse than
# none — it reads as real. This asserts the probe against THIS host's real
# kernel counters: the first draft parsed macOS's page size out of the wrong
# field and every sample reported free_bytes=0 through a full green gate.
: >"$WORK/events"
set +e
env -u RUST_TEST_THREADS \
  PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_VITALS_SECONDS=1 \
  AWL_NATIVE_GATE_PROBE_SLEEP=3 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-vitals" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-vitals" 2>&1
vitals_status=$?
set -e
(( vitals_status == 0 )) || {
  echo "test-native-gate: vitals probe failed ($vitals_status)" >&2
  exit 1
}
vitals_free="$(awk '/^native-gate-vitals/ { for (i = 1; i <= NF; i++) if ($i ~ /^free_bytes=/) { sub(/free_bytes=/, "", $i); print $i; exit } }' "$WORK/output-vitals")"
[[ -n "$vitals_free" ]] || {
  echo "test-native-gate: a running suite emitted no vitals heartbeat at all" >&2
  exit 1
}
[[ "$vitals_free" =~ ^[0-9]+$ ]] && (( vitals_free > 0 )) || {
  echo "test-native-gate: the vitals heartbeat reported free_bytes=$vitals_free — the memory probe does not work on this host" >&2
  exit 1
}

echo "test-native-gate: the vitals heartbeat reaches this host's real memory counters while the suite runs"

# ── Deadlock or livelock ─────────────────────────────────────────────────────
# Flat memory and zero swap are equally consistent with processes blocked on a
# fence and processes spinning on one, and the fixes have nothing in common.
# The laws below are stated as a PAIR on purpose: each one alone is satisfiable
# by a probe that always answers the same thing, and the pair is not. A probe
# hardwired to 0 fails the spinning direction; one hardwired to 100 fails the
# sleeping direction; a probe that finds no processes at all fails both.

# `busiest=[name:pid=pct]` is one whitespace-free field, so the highest reading
# across every heartbeat of a run is one awk pass. The MAX is the statistic that
# matters: a spin only has to show up in one heartbeat to be diagnosed.
busiest_peak() {
  awk 'BEGIN { peak = -1 }
    /^native-gate-vitals/ {
      for (i = 1; i <= NF; i++) if (index($i, "busiest=[") == 1) {
        value = $i; sub(/.*=/, "", value); sub(/\]$/, "", value)
        if (value + 0 > peak) { peak = value + 0; who = $i }
      }
    } END { printf "%.1f %s\n", peak, (who == "" ? "busiest=[absent]" : who) }' "$probe_output"
}

vitals_peak() {
  awk -v key="$1=" 'BEGIN { peak = -1 }
    /^native-gate-vitals/ {
      for (i = 1; i <= NF; i++) if (index($i, key) == 1) {
        value = substr($i, length(key) + 1) + 0
        if (value > peak) peak = value
      }
    } END { printf "%g\n", peak }' "$probe_output"
}

# The system load average is the heartbeat's headline and it is the field most
# likely to come back as a brace or an empty string: macOS hands it over as
# `{ 5.70 12.72 16.79 }` and Linux as the first field of /proc/loadavg. A probe
# that could not read it must SAY so rather than report a confident 0.00, so
# this asserts a real number from this host's real kernel — the same shape of
# assertion the memory law above makes, for the same reason.
probe load-average AWL_NATIVE_GATE_VITALS_SECONDS=1 AWL_NATIVE_GATE_PROBE_SLEEP=3
(( probe_status == 0 )) || { echo "test-native-gate: load-average probe failed ($probe_status)" >&2; exit 1; }
load_seen="$(awk '/^native-gate-vitals/ { for (i = 1; i <= NF; i++) if (index($i, "load1=") == 1) { sub(/load1=/, "", $i); print $i; exit } }' "$probe_output")"
[[ "$load_seen" =~ ^[0-9]+\.?[0-9]*$ ]] || {
  echo "test-native-gate: the heartbeat reported load1=$load_seen — the load-average probe does not work on this host" >&2
  exit 1
}
require "load average" "cpu_count=$(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 1)"

echo "test-native-gate: the heartbeat reports this host's real load average beside the core count that makes it readable"

# Direction 1 — LIVELOCK. The fixture's conventions burn CPU in-process for 9 s
# while producing no output, which is exactly what a spinning test binary looks
# like from outside. The heartbeat must report a pegged core AND name the pid
# doing it: a bare load average would rise here too, and would not say which of
# the gate's own processes to attach a debugger to.
: >"$WORK/spinners"
probe cpu-spin AWL_NATIVE_GATE_VITALS_SECONDS=3 AWL_NATIVE_GATE_PROBE_SPIN_SECONDS=9 \
  AWL_NATIVE_GATE_PROBE_SPIN_DELAY=4 AWL_NATIVE_GATE_PROBE_SPIN_PID_FILE="$WORK/spinners"
(( probe_status == 0 )) || { echo "test-native-gate: cpu-spin probe failed ($probe_status)" >&2; exit 1; }
[[ -s "$WORK/spinners" ]] || {
  echo "test-native-gate: the fixture never entered its spin, so this law proved nothing" >&2
  exit 1
}
read -r spin_peak spin_who <<<"$(busiest_peak)"
# One core fully pegged is 100. The floor is 50 because `ps -o time=` quantises
# to whole seconds on Linux, so a 3 s window can under-read a pegged process by
# a third; macOS reports hundredths and measures nearer 100.
awk -v peak="$spin_peak" 'BEGIN { exit !(peak >= 50) }' || {
  echo "test-native-gate: two conventions spun for 9s and the busiest tracked process peaked at ${spin_peak}% ($spin_who) — the CPU probe cannot see a livelock" >&2
  exit 1
}
# A process that appeared INSIDE the window must be measured over its own age,
# not dropped. The first draft dropped it, and the receipt run of 2026-08-02
# shows what that cost: two heartbeats reporting `tracked_procs=0` and `0.6%`
# while two test binaries were burning a core each, because Cargo had moved
# from one test target to the next inside the window. A probe that can only see
# processes older than a heartbeat is blind for the first minute of every run
# and blind again at every phase change — including the one the hang is near.
#
# The fixture's spinner is born mid-run for exactly this, so the heartbeat that
# first sees it is deterministically a newcomer reading.
read -r new_peak new_who <<<"$(awk 'BEGIN { peak = -1 }
    /^native-gate-vitals/ {
      fresh = -1; value = -1; who = ""
      for (i = 1; i <= NF; i++) {
        if (index($i, "new_procs=") == 1) fresh = substr($i, 11) + 0
        else if (index($i, "busiest=[") == 1) {
          who = $i; value = $i; sub(/.*=/, "", value); sub(/\]$/, "", value); value += 0
        }
      }
      if (fresh >= 1 && value > peak) { peak = value; bestwho = who; found = 1 }
    } END { if (found) printf "%.1f %s\n", peak, bestwho; else print "-1 no-heartbeat-reported-a-newcomer" }' "$probe_output")"
awk -v peak="$new_peak" 'BEGIN { exit !(peak >= 50) }' || {
  echo "test-native-gate: the busiest NEW process across every heartbeat read ${new_peak}% ($new_who) — a process that appeared inside the window is dropped instead of measured over its own age" >&2
  exit 1
}
new_pid="${new_who##*:}"; new_pid="${new_pid%%=*}"
grep -Fxq "$new_pid" "$WORK/spinners" || {
  echo "test-native-gate: the newcomer heartbeat blamed pid $new_pid ($new_who), which is not one of the fixture's spinners ($(tr '\n' ' ' <"$WORK/spinners"))" >&2
  exit 1
}
spin_pid="${spin_who##*:}"; spin_pid="${spin_pid%%=*}"
grep -Fxq "$spin_pid" "$WORK/spinners" || {
  echo "test-native-gate: the heartbeat blamed pid $spin_pid ($spin_who) but the processes actually spinning were $(tr '\n' ' ' <"$WORK/spinners")— a load number nobody can attribute is not a diagnosis" >&2
  exit 1
}

echo "test-native-gate: a spinning convention is reported as a pegged core and named by pid, not merely as a busy machine"

# Direction 2 — DEADLOCK. Same silence, same flat memory, zero CPU. This is the
# half that makes the pair non-vacuous, and `tracked_procs` is asserted
# separately: without it a probe that found NOTHING would pass this law while
# failing to measure anything at all.
probe cpu-idle AWL_NATIVE_GATE_VITALS_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=7
(( probe_status == 0 )) || { echo "test-native-gate: cpu-idle probe failed ($probe_status)" >&2; exit 1; }
idle_procs="$(vitals_peak tracked_procs)"
awk -v procs="$idle_procs" 'BEGIN { exit !(procs >= 1) }' || {
  echo "test-native-gate: the heartbeat tracked $idle_procs processes while two conventions were running — it measured nothing, so its zero means nothing" >&2
  exit 1
}
read -r idle_peak idle_who <<<"$(busiest_peak)"
awk -v peak="$idle_peak" 'BEGIN { exit !(peak < 25) }' || {
  echo "test-native-gate: two conventions were blocked in sleep(1) and the busiest tracked process read ${idle_peak}% ($idle_who) — the CPU probe cannot tell a deadlock from a livelock" >&2
  exit 1
}
refuse "cpu idle" "cpu_probe=broken"

echo "test-native-gate: a blocked convention reads as idle over a heartbeat that still tracked its processes — the two failures are distinguishable"

assert_concurrent_and_complete() {
  local first_two
  first_two="$(grep -E '^(start|finish) (mac|linux)$' "$WORK/events" | sed -n '1,2p' | sort)"
  [[ "$first_two" == $'start linux\nstart mac' ]] || {
    echo "test-native-gate: convention suites did not overlap: $first_two" >&2
    exit 1
  }
  [[ "$(grep -Ec '^finish (mac|linux)$' "$WORK/events")" == 2 ]] || {
    echo "test-native-gate: gate did not await both suites" >&2
    exit 1
  }
}

run_probe
(( probe_status == 0 )) || { echo "test-native-gate: success probe failed" >&2; exit 1; }
assert_concurrent_and_complete
grep -Fq 'native-gate-receipt' "$WORK/output-success" || {
  echo "test-native-gate: successful sibling suites emitted no receipt" >&2
  exit 1
}
grep -Fq "unit_tests=$(awk 'END { print NR }' "$WORK/awl-test-list") unit_shards=6 integration_targets=1" \
  "$WORK/output-success" || {
    echo "test-native-gate: receipt did not state its proved unit/shard/integration scope" >&2
    exit 1
  }
for convention in mac linux; do
  [[ "$(awk -v c="$convention" '$1 == "shard" && $2 == c { count++ } END { print count + 0 }' "$WORK/events")" == 6 ]] || {
    echo "test-native-gate: $convention did not execute all six proved binary shards" >&2
    exit 1
  }
done
grep -Fq 'native-test-shards verified full=' "$WORK/output-success" || {
  echo "test-native-gate: the successful receipt carried no binary completeness proof" >&2
  exit 1
}

probe one-shard AWL_NATIVE_GATE_SHARDS=1
(( probe_status == 0 )) || { echo "test-native-gate: one-shard probe failed ($probe_status)" >&2; exit 1; }
for convention in mac linux; do
  [[ "$(awk -v c="$convention" '$1 == "shard" && $2 == c { count++ } END { print count + 0 }' "$WORK/events")" == 1 ]] || {
    echo "test-native-gate: AWL_NATIVE_GATE_SHARDS=1 did not run one binary process for $convention" >&2
    exit 1
  }
done

probe shard-mutation AWL_NATIVE_GATE_PROBE_DELETE_PREFIX=1
(( probe_status == 1 )) || {
  echo "test-native-gate: deleting a generated prefix returned $probe_status, expected refusal status 1" >&2
  exit 1
}
require "shard mutation" "native-test-shards: completeness refusal"
require "shard mutation" "missing="
refuse "shard mutation" "native-gate-receipt"

echo "test-native-gate: six shards are complete, the one-shard wave knob is live, and deleting one generated prefix refuses by missing test name"

for failing in mac linux; do
  run_probe "$failing" 23
  (( probe_status == 1 )) || {
    echo "test-native-gate: $failing failure returned $probe_status, expected gate status 1" >&2
    exit 1
  }
  if grep -Fq 'native-gate-receipt' "$WORK/output-$failing"; then
    echo "test-native-gate: $failing failure leaked a receipt" >&2
    exit 1
  fi
  grep -Fq "mac_status=$([[ $failing == mac ]] && echo 23 || echo 0) linux_status=$([[ $failing == linux ]] && echo 23 || echo 0)" \
    "$WORK/output-$failing" || {
      echo "test-native-gate: $failing failure did not preserve both statuses" >&2
      exit 1
    }
  assert_concurrent_and_complete
done

echo "test-native-gate: both conventions overlap, both statuses survive, and either failure suppresses the receipt"

# ── THE MENU-BAR AXIS IS ACTUALLY SWEPT, IN BOTH ARMS ────────────────────────
# The point of the arms is that the axis stops depending on who remembers to
# edit a source file, and the way that promise dies quietly is the arms simply
# not being there — a deleted `gate_launch` line reads as a smaller diff, not as
# lost coverage. So this law reads the FORCING each cargo invocation actually
# received, not the gate's prose about it.
#
# Both directions matter and the second is the one that makes the pair
# non-vacuous: `on` and `off` must each appear exactly once (an arm that ran
# twice with the same value would sweep one branch and report two), and the
# canary plus both conventions must appear with the forcing UNSET (a convention
# that inherited one would make the axis a property of the convention arms).
run_probe
(( probe_status == 0 )) || { echo "test-native-gate: menu-bar axis probe failed" >&2; exit 1; }
for arm in on off; do
  [[ "$(grep -Fxc "menubar $arm" "$WORK/events")" == 1 ]] || {
    echo "test-native-gate: the gate ran the AWL_MENU_BAR_FORCE=$arm arm $(grep -Fxc "menubar $arm" "$WORK/events") times, expected exactly 1 — the axis is not swept" >&2
    exit 1
  }
done
[[ "$(grep -Fxc 'menubar unset' "$WORK/events")" == 3 ]] || {
  echo "test-native-gate: expected the canary and both conventions (3 invocations) to carry NO menu-bar forcing, saw $(grep -Fxc 'menubar unset' "$WORK/events")" >&2
  exit 1
}

# Either arm failing suppresses the receipt, and BOTH statuses are preserved in
# the message — the same contract the conventions have, for the same reason: a
# report that collapsed the two could not say which branch of the axis is red.
for arm in on off; do
  probe "menubar-fail-$arm" AWL_NATIVE_GATE_FAIL_MENU_BAR="$arm" AWL_NATIVE_GATE_FAIL_STATUS=29
  (( probe_status == 1 )) || {
    echo "test-native-gate: a failing menu-bar $arm arm returned $probe_status, expected gate status 1" >&2
    exit 1
  }
  require "menu-bar $arm failure" "native-gate: menu-bar axis failure"
  require "menu-bar $arm failure" \
    "on_status=$([[ $arm == on ]] && echo 29 || echo 0) off_status=$([[ $arm == off ]] && echo 29 || echo 0)"
  refuse "menu-bar $arm failure" "native-gate-receipt"
done

echo "test-native-gate: the menu-bar axis runs both arms with the conventions unforced, and either arm's failure suppresses the receipt"

# ── THE FULL-SUITE ARM REACHES EVERY SHARD, NOT EVERY NAME ───────────────────
# The filtered pair above is the CI shape. The local shape runs one arm over the
# WHOLE binary unit-test suite, and the only thing that makes that worth a third
# suite is that it reaches the tests a name filter cannot find. So the law counts
# the forcing at the SHARD, and requires it on exactly as many shard processes as
# a convention gets: a full arm that quietly re-acquired a filter would run one
# process and still print `mode=full-suite`.
probe menubar-full AWL_NATIVE_GATE_MENUBAR_FULL=1
(( probe_status == 0 )) || {
  echo "test-native-gate: full-suite menu-bar probe failed ($probe_status)" >&2
  tail -n 20 "$probe_output" >&2
  exit 1
}
menubar_line="$(grep -F 'native-gate-menubar ' "$probe_output" || true)"
[[ "$menubar_line" == *"mode=full-suite"* ]] || {
  echo "test-native-gate: AWL_NATIVE_GATE_MENUBAR_FULL=1 did not announce a full-suite arm: [$menubar_line]" >&2
  exit 1
}
menubar_field() { sed -n "s/.* $1=\([^ ]*\).*/\1/p" <<<"$menubar_line"; }
menubar_ambient="$(menubar_field ambient)"
menubar_forced="$(menubar_field forced)"
# The arm is only worth a suite if it forces the branch this host does NOT run
# ambiently; forcing the ambient one would sweep what the conventions swept.
# (`menubar::tests::the_gate_forces_the_branch_this_host_lacks` pins the ambient
# itself against `platform_default`; this end pins the opposition.)
[[ "$menubar_ambient" != "$menubar_forced" && -n "$menubar_forced" ]] || {
  echo "test-native-gate: the full arm forces ambient=$menubar_ambient forced=$menubar_forced — it sweeps the branch the conventions already ran" >&2
  exit 1
}
shard_forced="$(grep -Fxc "shardbar $menubar_forced" "$WORK/events")"
convention_shards="$(grep -Fxc 'shardbar unset' "$WORK/events")"
[[ "$shard_forced" == 6 ]] || {
  echo "test-native-gate: the full menu-bar arm forced $shard_forced shard processes, expected 6 — it is not running the whole suite" >&2
  exit 1
}
[[ "$convention_shards" == 12 ]] || {
  echo "test-native-gate: expected both conventions' 12 shards to carry NO forcing, saw $convention_shards" >&2
  exit 1
}
require "full menu-bar arm" "native-gate-receipt"
require "full menu-bar arm" "menubar=full:$menubar_forced"
# The cheap pair must be GONE in this mode, not merely joined: leaving it would
# spend two more Cargo invocations sweeping a subset of what just ran.
[[ "$(grep -c '^menubar ' "$WORK/events")" == 3 ]] || {
  echo "test-native-gate: the full-suite mode still ran name-filtered arms" >&2
  exit 1
}

probe menubar-full-fail AWL_NATIVE_GATE_MENUBAR_FULL=1 \
  AWL_NATIVE_GATE_FAIL_MENU_BAR="$menubar_forced" AWL_NATIVE_GATE_FAIL_STATUS=29
(( probe_status == 1 )) || {
  echo "test-native-gate: a failing full menu-bar arm returned $probe_status, expected gate status 1" >&2
  exit 1
}
require "full menu-bar failure" "native-gate: menu-bar axis failure"
require "full menu-bar failure" "AWL_MENU_BAR_FORCE=$menubar_forced"
require "full menu-bar failure" "ambient $menubar_ambient"
refuse "full menu-bar failure" "native-gate-receipt"

# WHICH MODE A HOST GETS IS DERIVED, AND THE DERIVATION IS THE PART THAT ROTS.
# Unset the pin and ask the gate twice. `CI` present means the fleet already runs
# both ambients across jobs, so the cheap pair is enough there; absent, this host
# is the only host and the full arm runs. A default that silently became
# "filtered everywhere" would leave every local gate exactly as blind as it was.
probe_menubar_mode() {
  local label="$1" expected="$2"
  shift 2
  : >"$WORK/events"
  set +e
  env -u RUST_TEST_THREADS -u AWL_NATIVE_GATE_BUDGET_SECONDS -u AWL_NATIVE_GATE_MENUBAR_FULL "$@" \
    PATH="$WORK:$PATH" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
    AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
    AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-$label" \
    "$ROOT/scripts/native-gate.sh" >"$WORK/output-$label" 2>&1
  local status=$?
  set -e
  (( status == 0 )) || {
    echo "test-native-gate: menu-bar mode probe $label failed ($status)" >&2
    tail -n 20 "$WORK/output-$label" >&2
    exit 1
  }
  grep -Fq "native-gate-menubar mode=$expected" "$WORK/output-$label" || {
    echo "test-native-gate: $label expected mode=$expected, got [$(grep -F 'native-gate-menubar ' "$WORK/output-$label" || true)]" >&2
    exit 1
  }
}
probe_menubar_mode menubar-mode-local full-suite -u CI
probe_menubar_mode menubar-mode-ci name-filtered CI=true

echo "test-native-gate: the full menu-bar arm forces every shard on the branch this host lacks, names itself on failure, and is derived from CI rather than remembered"

# ── The full-gate arbiter: one holder, a visible queue, safe stale recovery ──
# Two full gates must not recreate the contention that sharding removed. The
# second probe has its own event log, so an empty log while it reports the first
# holder is evidence that it has not begun a canary or a suite behind our back.
arbiter_marker="$WORK/arbiter-marker"
arbiter_lock="$WORK/arbiter-lock"
: >"$WORK/events-arbiter-first"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$arbiter_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_lock" \
  AWL_NATIVE_GATE_PROBE_SLEEP=4 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-arbiter-first" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-arbiter-first" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-arbiter-first" 2>&1 &
arbiter_first_pid=$!

for _ in $(seq 1 50); do
  [[ -s "$arbiter_marker" ]] && break
  sleep 0.1
done
[[ -s "$arbiter_marker" ]] || {
  echo "test-native-gate: the first arbiter probe never published a holder" >&2
  kill -TERM "$arbiter_first_pid" 2>/dev/null || true
  exit 1
}
arbiter_holder="$(cat "$arbiter_marker")"
arbiter_holder_pid="${arbiter_holder#pid=}"; arbiter_holder_pid="${arbiter_holder_pid%% *}"
kill -0 "$arbiter_holder_pid" 2>/dev/null || {
  echo "test-native-gate: arbiter holder pid=$arbiter_holder_pid was not alive" >&2
  exit 1
}

: >"$WORK/events-arbiter-second"
arbiter_second_arrival="$(date +%s)"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$arbiter_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_lock" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-arbiter-second" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-arbiter-second" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-arbiter-second" 2>&1 &
arbiter_second_pid=$!

for _ in $(seq 1 50); do
  grep -Fq "native-gate: waiting for arbiter holder $arbiter_holder" "$WORK/output-arbiter-second" && break
  sleep 0.1
done
grep -Fq "native-gate: waiting for arbiter holder $arbiter_holder" "$WORK/output-arbiter-second" || {
  echo "test-native-gate: the second full gate neither queued nor named its holder" >&2
  kill -TERM "$arbiter_first_pid" "$arbiter_second_pid" 2>/dev/null || true
  exit 1
}
[[ ! -s "$WORK/events-arbiter-second" ]] || {
  echo "test-native-gate: the queued second gate began work before the holder released" >&2
  kill -TERM "$arbiter_first_pid" "$arbiter_second_pid" 2>/dev/null || true
  exit 1
}
wait "$arbiter_first_pid"
wait "$arbiter_second_pid"
[[ ! -e "$arbiter_marker" ]] || {
  echo "test-native-gate: clean arbiter probes left admission state behind" >&2
  exit 1
}
grep -Fq 'native-gate-arbiter capacity=1 holder ' "$WORK/output-arbiter-second" || {
  echo "test-native-gate: the queued gate never acquired the arbiter" >&2
  exit 1
}
arbiter_second_epoch="$(awk '/^native-gate-arbiter capacity=1 holder / { for (i = 1; i <= NF; i++) if ($i ~ /^start_epoch=/) { sub(/^start_epoch=/, "", $i); print $i; exit } }' "$WORK/output-arbiter-second")"
[[ "$arbiter_second_epoch" =~ ^[0-9]+$ && "$arbiter_second_epoch" -gt "$arbiter_second_arrival" ]] || {
  echo "test-native-gate: queued gate published start_epoch=$arbiter_second_epoch from arrival=$arbiter_second_arrival — it measured queue time instead of admitted work" >&2
  exit 1
}

# Mutation: the holder is killed while its test descendant ignores TERM. The
# waiter must acquire before that owned orphan is reaped; otherwise fd 8 leaked
# through a child and the claimed kernel-release guarantee is false.
arbiter_orphan_marker="$WORK/arbiter-orphan-marker"
arbiter_orphan_lock="$WORK/arbiter-orphan.lock"
: >"$WORK/arbiter-orphans"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$arbiter_orphan_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_orphan_lock" \
  AWL_NATIVE_GATE_PROBE_SLEEP=30 \
  AWL_NATIVE_GATE_PROBE_ORPHAN_FILE="$WORK/arbiter-orphans" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-arbiter-orphan-first" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-arbiter-orphan-first" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-arbiter-orphan-first" 2>&1 &
arbiter_orphan_first_pid=$!
for _ in $(seq 1 50); do
  [[ -s "$arbiter_orphan_marker" && -s "$WORK/arbiter-orphans" ]] && break
  sleep 0.1
done
[[ -s "$WORK/arbiter-orphans" ]] || {
  echo "test-native-gate: orphan-holder fixture never created its surviving descendant" >&2
  kill -TERM "$arbiter_orphan_first_pid" 2>/dev/null || true
  exit 1
}
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$arbiter_orphan_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_orphan_lock" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-arbiter-orphan-second" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-arbiter-orphan-second" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-arbiter-orphan-second" 2>&1 &
arbiter_orphan_second_pid=$!
kill -TERM "$arbiter_orphan_first_pid"
set +e
wait "$arbiter_orphan_first_pid"
set -e
for _ in $(seq 1 50); do
  grep -Fq 'native-gate-arbiter capacity=1 holder ' "$WORK/output-arbiter-orphan-second" && break
  sleep 0.1
done
grep -Fq 'native-gate-arbiter capacity=1 holder ' "$WORK/output-arbiter-orphan-second" || {
  echo "test-native-gate: waiter stayed behind a killed holder's surviving descendant — fd 8 leaked" >&2
  kill -TERM "$arbiter_orphan_second_pid" 2>/dev/null || true
  exit 1
}
while read -r orphan; do kill -KILL "$orphan" 2>/dev/null || true; done <"$WORK/arbiter-orphans"
wait "$arbiter_orphan_second_pid"

# This is the failure-shaped mutation: a killed holder leaves stale marker
# text. The kernel releases its flock, so the next holder overwrites the text
# without PID-reuse guessing or deleting a path another gate may be publishing.
printf 'pid=99999999 start_commit=stale start_epoch=1\n' >"$arbiter_marker"
probe arbiter-stale AWL_NATIVE_GATE_MARKER="$arbiter_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_lock"
(( probe_status == 0 )) || {
  echo "test-native-gate: stale arbiter recovery probe failed ($probe_status)" >&2
  exit 1
}
require "arbiter stale recovery" "native-gate-arbiter capacity=1 holder"
[[ ! -e "$arbiter_marker" ]] || {
  echo "test-native-gate: a recovered stale holder still blocked later gates" >&2
  exit 1
}

echo "test-native-gate: full gates queue behind a named live holder, then proceed; a stale marker cannot wedge the kernel arbiter"

# A durable worktree's own .orchestrator directory is private. This hostile
# git seam reports a distinct common Git directory and proves the default path
# follows that fleet root instead, without an explicit marker/lock override.
fleet_root="$WORK/fleet-root"
fleet_common="$fleet_root/.git"
fleet_marker="$fleet_root/.orchestrator/native-gate.marker"
mkdir -p "$fleet_common" "$fleet_root/.orchestrator"
: >"$WORK/events-fleet-default"
set +e
env -u AWL_NATIVE_GATE_MARKER -u AWL_NATIVE_GATE_ARBITER_LOCK \
  PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_PROBE_COMMON_GIT_DIR="$fleet_common" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-fleet-default" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-fleet-default" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-fleet-default" 2>&1
fleet_default_status=$?
set -e
(( fleet_default_status == 0 )) || {
  echo "test-native-gate: fleet-default path probe failed ($fleet_default_status)" >&2
  exit 1
}
[[ ! -e "$fleet_marker" ]] || {
  echo "test-native-gate: fleet-default probe left its common marker behind" >&2
  exit 1
}
grep -Fq 'native-gate-arbiter capacity=1 holder ' "$WORK/output-fleet-default" || {
  echo "test-native-gate: fleet-default probe never entered its derived arbiter" >&2
  exit 1
}

echo "test-native-gate: the default arbiter path follows Git's common directory, not a worktree-local marker"

# ── The in-flight marker: holder identity and signal cleanup ─────────────────
marker="$WORK/marker"
: >"$WORK/events"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$marker" \
  AWL_NATIVE_GATE_PROBE_SLEEP=4 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-marker-live" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-marker-live" 2>&1 &
marker_gate_pid=$!

# Polled rather than a fixed sleep, so this is not flaky on a loaded runner —
# the marker is written before the canary even starts, so it should appear
# almost immediately.
marker_seen=0
for _ in $(seq 1 50); do
  [[ -s "$marker" ]] && { marker_seen=1; break; }
  sleep 0.1
done
(( marker_seen == 1 )) || {
  echo "test-native-gate: no marker appeared while the gate was running" >&2
  kill -TERM "$marker_gate_pid" 2>/dev/null || true
  exit 1
}
marker_line="$(cat "$marker")"
[[ "$marker_line" == pid=*' start_commit='*' start_epoch='* ]] || {
  echo "test-native-gate: marker did not carry pid/start_commit/start_epoch: $marker_line" >&2
  exit 1
}
marker_pid="${marker_line#pid=}"; marker_pid="${marker_pid%% *}"
# Mutation proof: a reader checking the marker mid-run must actually see a
# live process, not merely a file. `kill -0` is the exact check the README
# tells an orchestrator to run.
kill -0 "$marker_pid" 2>/dev/null || {
  echo "test-native-gate: marker named pid=$marker_pid but that pid is not alive — kill -0 would wrongly call this run dead" >&2
  exit 1
}
marker_commit="${marker_line#*start_commit=}"; marker_commit="${marker_commit%% *}"
[[ "$marker_commit" == "$(git -C "$ROOT" rev-parse HEAD)" ]] || {
  echo "test-native-gate: marker start_commit=$marker_commit did not match the actual HEAD" >&2
  exit 1
}

echo "test-native-gate: a reader checking the marker mid-run sees a live pid (kill -0 succeeds) and the correct start commit"

# Captured BEFORE the kill below, while the loop is still alive to be found.
# This is the process the item's live-host diagnosis kept finding at
# ppid=1 with a `sleep` child: the vitals heartbeat, orphaned by a SIGTERM
# that only ever reached the top-level script.
vitals_pid="$(find_vitals_pid "$marker_gate_pid")" || {
  echo "test-native-gate: could not find the running gate's vitals-loop child, so the leak law below would prove nothing" >&2
  kill -TERM "$marker_gate_pid" 2>/dev/null || true
  exit 1
}

# ── The kill path: the case that matters most ────────────────────────────────
# A marker that outlives its process silently wedges every later session's
# advisory check — worse than the defect this item exists to fix. SIGTERM is
# the realistic case: a human or an agent ending a gate deliberately.
kill -TERM "$marker_gate_pid"
set +e
wait "$marker_gate_pid" 2>/dev/null
marker_kill_status=$?
set -e
(( marker_kill_status != 0 )) || {
  echo "test-native-gate: a SIGTERM'd gate reported success ($marker_kill_status)" >&2
  exit 1
}
[[ ! -e "$marker" ]] || {
  echo "test-native-gate: the marker survived SIGTERM — a killed gate would silently wedge a later commit's advisory check" >&2
  exit 1
}

# The marker disappearing proves the EXIT trap ran; it does not by itself
# prove the trap retired vitals_pid too. Give the signal a moment to land —
# gate_vitals_loop's own TERM trap does the actual dying — then check the
# pid directly, the same evidence a live host's `ps -ww` supplied.
sleep 1
kill -0 "$vitals_pid" 2>/dev/null && {
  echo "test-native-gate: the vitals heartbeat (pid=$vitals_pid) survived a SIGTERM to the gate's own pid — orphaned at ppid=1, still holding a sleep child and this script's inherited stdout open" >&2
  kill -TERM "$vitals_pid" 2>/dev/null || true
  exit 1
}

echo "test-native-gate: a SIGTERM to the gate's own pid retires the vitals heartbeat too, not only the marker"

echo "test-native-gate: killing the gate removes the marker — a killed run cannot wedge a later session"

# ── SIGINT reaches the same unconditional trap ────────────────────────────
# Ctrl-C forwarded to a foregrounded gate is the other realistic teardown
# shape, and it is worth proving separately rather than assumed to behave
# like SIGTERM: bash's default disposition differs per signal, and the EXIT
# trap is only guaranteed to fire on the ones actually exercised here.
sigint_marker="$WORK/marker-sigint"
: >"$WORK/events"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$sigint_marker" \
  AWL_NATIVE_GATE_PROBE_SLEEP=4 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-sigint" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-sigint" 2>&1 &
sigint_gate_pid=$!

sigint_seen=0
for _ in $(seq 1 50); do
  [[ -s "$sigint_marker" ]] && { sigint_seen=1; break; }
  sleep 0.1
done
(( sigint_seen == 1 )) || {
  echo "test-native-gate: no marker appeared for the SIGINT probe" >&2
  kill -TERM "$sigint_gate_pid" 2>/dev/null || true
  exit 1
}
sigint_vitals_pid="$(find_vitals_pid "$sigint_gate_pid")" || {
  echo "test-native-gate: could not find the SIGINT probe's vitals-loop child, so this law would prove nothing" >&2
  kill -TERM "$sigint_gate_pid" 2>/dev/null || true
  exit 1
}

kill -INT "$sigint_gate_pid"
set +e
wait "$sigint_gate_pid" 2>/dev/null
set -e
sleep 1
kill -0 "$sigint_vitals_pid" 2>/dev/null && {
  echo "test-native-gate: the vitals heartbeat (pid=$sigint_vitals_pid) survived a SIGINT to the gate's own pid" >&2
  kill -TERM "$sigint_vitals_pid" 2>/dev/null || true
  exit 1
}

echo "test-native-gate: a SIGINT to the gate's own pid retires the vitals heartbeat too"

# ── A clean run leaves nothing behind ─────────────────────────────────────────
marker_clean="$WORK/marker-clean"
: >"$WORK/events"
set +e
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$marker_clean" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-marker-clean" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-marker-clean" 2>&1
marker_clean_status=$?
set -e
(( marker_clean_status == 0 )) || {
  echo "test-native-gate: the marker-cleanliness probe's own gate run failed ($marker_clean_status)" >&2
  exit 1
}
[[ ! -e "$marker_clean" ]] || {
  echo "test-native-gate: a normal completion left the marker behind" >&2
  exit 1
}

echo "test-native-gate: a normal completion leaves no marker behind"
