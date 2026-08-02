#!/usr/bin/env bash
# The one native full-suite gate. A receipt from this script, on the commit it
# names, is the only evidence that both supported conventions ran every native
# Cargo test target. `cargo test --bin awl` is binary unit tests, not this gate.
set -euo pipefail

if (( $# != 0 )); then
  echo "native-gate: target selection and test-name arguments are forbidden; run targeted tests directly" >&2
  exit 2
fi

gate_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AWL_DISK_PREFLIGHT_CALLER=native-gate "$gate_root/.orchestrator/disk-preflight.sh"

start_commit="$(git rev-parse HEAD)"

# Two conventions run at once below, so every bound here is per convention and
# the machine sees twice it.
readonly gate_conventions=2

gate_cpu_count() {
  if [[ -n "${AWL_NATIVE_GATE_CPUS:-}" ]]; then printf '%s\n' "$AWL_NATIVE_GATE_CPUS"; return; fi
  if sysctl -n hw.ncpu 2>/dev/null; then return; fi
  if command -v nproc >/dev/null 2>&1; then nproc; return; fi
  echo 1
}

gate_mem_bytes() {
  if [[ -n "${AWL_NATIVE_GATE_MEM_BYTES:-}" ]]; then printf '%s\n' "$AWL_NATIVE_GATE_MEM_BYTES"; return; fi
  if sysctl -n hw.memsize 2>/dev/null; then return; fi
  if [[ -r /proc/meminfo ]]; then awk '/^MemTotal:/ { print $2 * 1024; exit }' /proc/meminfo; return; fi
  echo 0
}

# Measured at HEAD on 2026-08-02 (`/usr/bin/time -l` over the unit-test binary,
# 3484 tests): peak RSS 448 MiB at one test thread, 486 MiB at three, 667 MiB at
# ten — about 24 MiB per added thread over a ~448 MiB process floor. Wall time
# across that same sweep was 125.1 s / 119.7 s / 126.2 s, i.e. FLAT, because
# `testlock::serial` already serialises every global-touching test. So a thread
# bound buys headroom for free; it is not a speed/safety trade.
readonly gate_thread_floor_bytes=$((512 * 1024 * 1024))
readonly gate_thread_bytes=$((32 * 1024 * 1024))

gate_test_threads() {
  local cpus="$1" mem_bytes="$2" cpu_share mem_share
  cpu_share=$(( cpus / gate_conventions ))
  (( cpu_share < 1 )) && cpu_share=1
  mem_share=$cpu_share
  if (( mem_bytes > 0 )); then
    mem_share=$(( (mem_bytes / gate_conventions - gate_thread_floor_bytes) / gate_thread_bytes ))
    (( mem_share < 1 )) && mem_share=1
  fi
  if (( mem_share < cpu_share )); then printf '%s\n' "$mem_share"; else printf '%s\n' "$cpu_share"; fi
}

gate_cpus="$(gate_cpu_count)"
gate_mem_bytes_value="$(gate_mem_bytes)"
# A caller that states a bound owns it; the gate only supplies the default. It
# is deliberately RUST_TEST_THREADS and not a `cargo test` argument, because the
# suite's SCOPE must stay literally unfiltered — this bounds how many tests run
# at once, never which ones run. It is equally deliberately not Cargo's own job
# budget: `.orchestrator/worker-build.sh` is that value's sole owner, and this
# gate must not compete with it.
if [[ -z "${RUST_TEST_THREADS:-}" ]]; then
  RUST_TEST_THREADS="$(gate_test_threads "$gate_cpus" "$gate_mem_bytes_value")"
fi
export RUST_TEST_THREADS

# A hosted runner that is starved to death uploads NO log at all — the mac job's
# step-8 deaths on 2026-08-01/02 left an HTTP 404 where the log should be — so
# the gate states the machine it is about to load BEFORE it loads it, and keeps
# saying what that machine is doing while it runs. Both lines are unconditional:
# evidence that only appears on failure is evidence nobody has ever read.
printf 'native-gate-env cpus=%s mem_bytes=%s conventions=%s test_threads=%s budget_seconds=%s\n' \
  "$gate_cpus" "$gate_mem_bytes_value" "$gate_conventions" "$RUST_TEST_THREADS" \
  "${AWL_NATIVE_GATE_BUDGET_SECONDS:-none}"

gate_free_bytes() {
  if [[ -r /proc/meminfo ]]; then
    awk '/^MemAvailable:/ { print $2 * 1024; exit }' /proc/meminfo
    return
  fi
  vm_stat 2>/dev/null | awk '
    NR == 1 { gsub(/[^0-9]/, "", $NF); page = $NF }
    /^Pages free/ || /^Pages inactive/ || /^Pages speculative/ { gsub(/\./, "", $NF); pages += $NF }
    END { print pages * page }
  '
}

gate_swap_bytes() {
  if [[ -r /proc/meminfo ]]; then
    awk '/^SwapTotal:/ { t = $2 } /^SwapFree:/ { f = $2 } END { print (t - f) * 1024 }' /proc/meminfo
    return
  fi
  sysctl -n vm.swapusage 2>/dev/null | awk '{ gsub(/M/, "", $6); printf "%.0f\n", $6 * 1048576 }'
}

gate_vitals_interval="${AWL_NATIVE_GATE_VITALS_SECONDS:-60}"

# Both helpers below outlive nothing: each sleeps in a child it can name, so the
# TERM that retires it also retires the sleep. An orphaned `sleep` would keep
# the gate's inherited stdout open, and a caller capturing this script's output
# would block on it long after the receipt was printed.
gate_sleep_then() {
  local seconds="$1" sleeper=""
  shift
  trap '[[ -n "$sleeper" ]] && kill "$sleeper" 2>/dev/null; exit 0' TERM
  sleep "$seconds" &
  sleeper=$!
  wait "$sleeper" 2>/dev/null || exit 0
  "$@"
}

gate_vitals_loop() {
  local started elapsed sleeper=""
  started="$(date +%s)"
  trap '[[ -n "$sleeper" ]] && kill "$sleeper" 2>/dev/null; exit 0' TERM
  while :; do
    sleep "$gate_vitals_interval" &
    sleeper=$!
    wait "$sleeper" 2>/dev/null || exit 0
    elapsed=$(( $(date +%s) - started ))
    printf 'native-gate-vitals elapsed_seconds=%s free_bytes=%s swap_used_bytes=%s\n' \
      "$elapsed" "$(gate_free_bytes)" "$(gate_swap_bytes)"
  done
}

# This is deliberately an integration target, outside the binary unit-test
# target. Its first position makes integration-test discovery disappear loudly.
canary_command=(cargo test --test native_gate_canary)
mac_command=(env AWL_CONVENTION_FORCE=mac cargo test)
linux_command=(env AWL_CONVENTION_FORCE=linux cargo test)

echo "==> native integration canary"
"${canary_command[@]}"

# The canary fronts dependency and library compilation. Cargo's shared-target
# lock prevents duplicate remaining compilation when these siblings start; in
# worker lanes both also inherit the orchestration-owned Cargo cap.
echo "==> native suites (mac and linux conventions, concurrent)"
gate_vitals_loop &
vitals_pid=$!
"${mac_command[@]}" &
mac_pid=$!
"${linux_command[@]}" &
linux_pid=$!

# The budget exists to convert an OUTCOME NOBODY CAN READ into one anybody can.
# Left unset it does nothing, so no local run inherits a new way to fail; CI
# sets it under the job's own `timeout-minutes`, because a job that trips its
# GitHub ceiling — or gets its runner killed — publishes no log, while a gate
# that trips its own budget exits normally and its log survives to say why.
gate_budget_marker="$(mktemp "${TMPDIR:-/tmp}/awl-native-gate-budget.XXXXXX")"
rm -f "$gate_budget_marker"

gate_budget_expired() {
  printf 'exceeded\n' >"$gate_budget_marker"
  printf 'native-gate: budget of %ss exceeded; free_bytes=%s swap_used_bytes=%s; terminating both conventions\n' \
    "$AWL_NATIVE_GATE_BUDGET_SECONDS" "$(gate_free_bytes)" "$(gate_swap_bytes)" >&2
  kill -TERM "$mac_pid" "$linux_pid" 2>/dev/null || true
  sleep 5
  kill -KILL "$mac_pid" "$linux_pid" 2>/dev/null || true
}

budget_pid=""
if [[ -n "${AWL_NATIVE_GATE_BUDGET_SECONDS:-}" ]]; then
  gate_sleep_then "$AWL_NATIVE_GATE_BUDGET_SECONDS" gate_budget_expired &
  budget_pid=$!
fi

# `wait` is allowed to report failure without set -e ending the gate before the
# sibling has finished. Preserve both statuses; neither convention can hide the
# other or authorize a receipt on partial coverage.
set +e
wait "$mac_pid"
mac_status=$?
wait "$linux_pid"
linux_status=$?
set -e

kill -TERM "$vitals_pid" 2>/dev/null || true
[[ -n "$budget_pid" ]] && { kill -TERM "$budget_pid" 2>/dev/null || true; }

if [[ -f "$gate_budget_marker" ]]; then
  rm -f "$gate_budget_marker"
  printf 'native-gate: ABORTED on its %ss budget with mac_status=%s linux_status=%s; no receipt issued\n' \
    "${AWL_NATIVE_GATE_BUDGET_SECONDS:-unset}" "$mac_status" "$linux_status" >&2
  exit 1
fi
rm -f "$gate_budget_marker"

if (( mac_status != 0 || linux_status != 0 )); then
  printf 'native-gate: suite failure mac_status=%s linux_status=%s; no receipt issued\n' \
    "$mac_status" "$linux_status" >&2
  exit 1
fi

end_commit="$(git rev-parse HEAD)"
if [[ "$start_commit" != "$end_commit" ]]; then
  echo "native-gate: HEAD changed while the suite ran (start=$start_commit end=$end_commit); no receipt issued" >&2
  exit 1
fi

printf 'native-gate-receipt commit=%s conventions=mac,linux scope=all-targets\n' "$end_commit"
