#!/usr/bin/env bash
# Exercise the gate's orchestration and failure semantics without compiling.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-native-gate-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

cat >"$WORK/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
convention="${AWL_CONVENTION_FORCE:-canary}"
printf 'start %s\n' "$convention" >>"$AWL_NATIVE_GATE_PROBE_LOG"
printf 'threads %s %s\n' "$convention" "${RUST_TEST_THREADS:-unset}" >>"$AWL_NATIVE_GATE_PROBE_LOG"
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
  printf '%s\n' "$$" >>"$AWL_NATIVE_GATE_PROBE_SPIN_PID_FILE"
  SECONDS=0
  while (( SECONDS < AWL_NATIVE_GATE_PROBE_SPIN_SECONDS )); do :; done
else
  sleep "${AWL_NATIVE_GATE_PROBE_SLEEP:-0.2}"
fi
printf 'finish %s\n' "$convention" >>"$AWL_NATIVE_GATE_PROBE_LOG"
if [[ "$convention" == "${AWL_NATIVE_GATE_FAIL_CONVENTION:-}" ]]; then
  exit "${AWL_NATIVE_GATE_FAIL_STATUS:-1}"
fi
EOF
chmod +x "$WORK/cargo"

cat >"$WORK/free-oracle" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' $((40 * 1024 * 1024 * 1024))
EOF
chmod +x "$WORK/free-oracle"

run_probe() {
  local failing="${1:-}" status="${2:-0}" output="$WORK/output-${1:-success}"
  : >"$WORK/events"
  set +e
  PATH="$WORK:$PATH" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
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
# 2026-08-02 run 30732589551 the job's own cleanup had to reap two survivors by
# hand AFTER this gate exited; a survivor holds the step's stdout, and a GitHub
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
  awk '/^native-gate-vitals/ {
      for (i = 1; i <= NF; i++) if (index($i, "busiest=[") == 1) {
        value = $i; sub(/.*=/, "", value); sub(/\]$/, "", value)
        if (value + 0 > peak) { peak = value + 0; who = $i }
      }
    } END { printf "%.1f %s\n", peak, (who == "" ? "busiest=[absent]" : who) }' "$probe_output"
}

vitals_peak() {
  awk -v key="$1=" '/^native-gate-vitals/ {
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
  AWL_NATIVE_GATE_PROBE_SPIN_PID_FILE="$WORK/spinners"
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
# The FIRST heartbeat is the one that has to work, and it is the hardest: its
# baseline was taken before the conventions existed, so every tracked process is
# a NEWCOMER to it. The first draft dropped newcomers outright, and the receipt
# run of 2026-08-02 shows what that cost — two heartbeats reporting
# `tracked_procs=0 tracked_cpu_pct=none` and `0.6%` while two test binaries were
# burning a core each, because Cargo had moved from one test target to the next
# inside the window. A probe that only measures processes older than a minute is
# blind to exactly the minute anybody is asking about.
first_new="$(awk '/^native-gate-vitals/ { for (i = 1; i <= NF; i++) if (index($i, "new_procs=") == 1) { print substr($i, 11) + 0; exit } }' "$probe_output")"
first_busy="$(awk '/^native-gate-vitals/ { for (i = 1; i <= NF; i++) if (index($i, "busiest=[") == 1) { v = $i; sub(/.*=/, "", v); sub(/\]$/, "", v); print v + 0; exit } }' "$probe_output")"
awk -v new="$first_new" -v busy="$first_busy" 'BEGIN { exit !(new >= 1 && busy >= 50) }' || {
  echo "test-native-gate: the first heartbeat of a spinning run reported new_procs=$first_new busiest=${first_busy}% — a process that appeared inside the window is dropped instead of measured over its own age" >&2
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
