#!/usr/bin/env bash
# Exercise scripts/preflight.sh's orchestration and failure semantics without
# compiling clippy or the test binary: the real steps ARE scripts/code-health.sh
# (several minutes) and a `cargo test` invocation, and this file is itself
# wired into code-health.sh, so an unstubbed run here would recompile clippy
# from scratch on every code-health.sh pass. Stub both steps instead, the same
# shape as scripts/test-native-gate.sh's disposable health command.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-preflight-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

pass_stub() {
  # $1: path to write. $2: marker file to touch when invoked (proves the
  # step ran), $3: message to print on stdout.
  cat >"$1" <<STUBEOF
#!/usr/bin/env bash
touch "$2"
echo "$3"
STUBEOF
  chmod +x "$1"
}

fail_stub() {
  cat >"$1" <<STUBEOF
#!/usr/bin/env bash
touch "$2"
echo "$3" >&2
exit 1
STUBEOF
  chmod +x "$1"
}

run_preflight() {
  # Runs with both steps stubbed via the probe env vars; captures stdout+stderr
  # and exit status without letting a failure trip this script's own set -e.
  local out status
  out=$(AWL_PREFLIGHT_PROBE_HEALTH_COMMAND="$1" AWL_PREFLIGHT_PROBE_AUDIT_COMMAND="$2" \
    "$ROOT/scripts/preflight.sh" 2>&1) && status=0 || status=$?
  printf '%s' "$out" >"$WORK/last-output"
  return $status
}

# 1. Argument rejection: no side effects, exits 2, never reaches either step.
if AWL_PREFLIGHT_PROBE_HEALTH_COMMAND=/bin/true AWL_PREFLIGHT_PROBE_AUDIT_COMMAND=/bin/true \
  "$ROOT/scripts/preflight.sh" extra-arg >"$WORK/argtest-output" 2>&1; then
  echo "test-preflight: expected an argument to be refused" >&2
  cat "$WORK/argtest-output" >&2
  exit 1
fi
if ! grep -q "no arguments are accepted" "$WORK/argtest-output"; then
  echo "test-preflight: argument-rejection message missing" >&2
  cat "$WORK/argtest-output" >&2
  exit 1
fi
echo "test-preflight: an argument is refused before either step runs"

# 2. A failing health step fails the whole run and the audit step never runs.
pass_stub "$WORK/health-ok" "$WORK/health-ran" "health: ok"
fail_stub "$WORK/health-fail" "$WORK/health-ran" "code-health: policy check failed"
pass_stub "$WORK/audit-ok" "$WORK/audit-ran" "audit: ok"

rm -f "$WORK/health-ran" "$WORK/audit-ran"
if run_preflight "$WORK/health-fail" "$WORK/audit-ok"; then
  echo "test-preflight: expected failure when the health step fails" >&2
  cat "$WORK/last-output" >&2
  exit 1
fi
if [[ ! -e "$WORK/health-ran" ]]; then
  echo "test-preflight: the health step should have run" >&2
  exit 1
fi
if [[ -e "$WORK/audit-ran" ]]; then
  echo "test-preflight: the audit step ran despite a failed health step — ordering is broken" >&2
  exit 1
fi
if grep -q "PASSED" "$WORK/last-output"; then
  echo "test-preflight: a failed health step must never print PASSED" >&2
  exit 1
fi
echo "test-preflight: a failing health step fails the run and the audit step never runs"

# 3. A passing health step but a failing audit step still fails the whole run.
fail_stub "$WORK/audit-fail" "$WORK/audit-ran" "a println!/eprintln! call appeared somewhere unaccounted for"
rm -f "$WORK/health-ran" "$WORK/audit-ran"
if run_preflight "$WORK/health-ok" "$WORK/audit-fail"; then
  echo "test-preflight: expected failure when the audit step fails" >&2
  cat "$WORK/last-output" >&2
  exit 1
fi
if [[ ! -e "$WORK/health-ran" || ! -e "$WORK/audit-ran" ]]; then
  echo "test-preflight: both steps should have run before the audit step's failure surfaced" >&2
  exit 1
fi
if grep -q "PASSED" "$WORK/last-output"; then
  echo "test-preflight: a failed audit step must never print PASSED" >&2
  exit 1
fi
echo "test-preflight: a failing audit step (health clean) still fails the run"

# 4. Both steps passing: the run succeeds and labels itself targeted evidence,
# never a full receipt.
rm -f "$WORK/health-ran" "$WORK/audit-ran"
if ! run_preflight "$WORK/health-ok" "$WORK/audit-ok"; then
  echo "test-preflight: expected success when both steps pass" >&2
  cat "$WORK/last-output" >&2
  exit 1
fi
if [[ ! -e "$WORK/health-ran" || ! -e "$WORK/audit-ran" ]]; then
  echo "test-preflight: both steps should have run on success" >&2
  exit 1
fi
if ! grep -q "PASSED" "$WORK/last-output"; then
  echo "test-preflight: a successful run should print PASSED" >&2
  cat "$WORK/last-output" >&2
  exit 1
fi
if ! grep -qi "targeted evidence" "$WORK/last-output"; then
  echo "test-preflight: a successful run must label itself targeted evidence, not a full receipt" >&2
  cat "$WORK/last-output" >&2
  exit 1
fi
if ! grep -qi "NOT a full receipt" "$WORK/last-output"; then
  echo "test-preflight: success output must state it is NOT a full receipt" >&2
  cat "$WORK/last-output" >&2
  exit 1
fi
echo "test-preflight: a fully passing run labels itself targeted evidence, not a full receipt"

echo "test-preflight: ordering, failure propagation, and self-labeling all proved"
