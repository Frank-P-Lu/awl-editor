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
if [[ "$convention" != canary ]]; then
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
