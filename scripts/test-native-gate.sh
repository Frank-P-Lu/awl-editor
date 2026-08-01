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
if [[ "$convention" != canary ]]; then
  sleep 0.2
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
