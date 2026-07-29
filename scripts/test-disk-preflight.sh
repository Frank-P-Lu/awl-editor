#!/usr/bin/env bash
# External law for the orchestration-owned disk preflight. Every oracle and
# sweep is a disposable fixture; no real target directory is traversed.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFLIGHT="$ROOT/.orchestrator/disk-preflight.sh"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-disk-preflight-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

oracle="$WORK/oracle.sh"
sweep="$WORK/sweep.sh"
cat >"$oracle" <<'EOF'
#!/usr/bin/env bash
cat "$AWL_TEST_FREE_FILE"
EOF
cat >"$sweep" <<'EOF'
#!/usr/bin/env bash
printf 'sweep\n' >>"$AWL_TEST_SWEEP_LOG"
sleep "${AWL_TEST_SWEEP_DELAY:-0}"
printf '%s\n' "$AWL_TEST_SWEEP_RESULT" >"$AWL_TEST_FREE_FILE"
EOF
chmod +x "$oracle" "$sweep"

run() {
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$oracle" \
  AWL_DISK_PREFLIGHT_SWEEP_COMMAND="$sweep" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/lock" \
  AWL_TEST_FREE_FILE="$WORK/free" \
  AWL_TEST_SWEEP_LOG="$WORK/sweeps" \
  AWL_TEST_SWEEP_RESULT="${1:-9663676416}" \
  "${2:-$PREFLIGHT}"
}

grep -Fq 'AWL_DISK_PREFLIGHT_CALLER=worker-build "$ROOT/.orchestrator/disk-preflight.sh"' \
  "$ROOT/.orchestrator/worker-build.sh" || {
  echo "test-disk-preflight: worker launch seam bypasses the canonical preflight" >&2; exit 1;
}
grep -Fq 'AWL_DISK_PREFLIGHT_CALLER=native-gate "$gate_root/.orchestrator/disk-preflight.sh"' \
  "$ROOT/scripts/native-gate.sh" || {
  echo "test-disk-preflight: canonical native gate bypasses the preflight" >&2; exit 1;
}

# Healthy: no recovery owner executes.
printf '%s\n' 9663676416 >"$WORK/free"
: >"$WORK/sweeps"
healthy="$(run)"
[[ "$healthy" == *'status=healthy'* && ! -s "$WORK/sweeps" ]] || {
  echo "test-disk-preflight: healthy disk must not sweep" >&2; exit 1;
}

# Low space recovers through exactly the supplied sweep owner.
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
recovered="$(run 9663676416)"
[[ "$recovered" == *'status=recovered'* && "$(wc -l <"$WORK/sweeps")" -eq 1 ]] || {
  echo "test-disk-preflight: low disk did not recover through one sweep" >&2; exit 1;
}

# An unsuccessful recovery is a truthful early failure, before any Cargo call.
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
if run 1073741824 >"$WORK/insufficient.out" 2>"$WORK/insufficient.err"; then
  echo "test-disk-preflight: insufficient recovery unexpectedly passed" >&2; exit 1
fi
grep -Fq 'insufficient space after sweep-1d' "$WORK/insufficient.err" || {
  echo "test-disk-preflight: insufficiency did not name the attempted recovery" >&2; exit 1;
}

# CI is portable capacity checking, not a second cleanup owner.
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
if CI=1 run 9663676416 >"$WORK/ci.out" 2>"$WORK/ci.err"; then
  echo "test-disk-preflight: undersized CI unexpectedly passed" >&2; exit 1
fi
grep -Fq 'ci-no-sweep' "$WORK/ci.err" && [[ ! -s "$WORK/sweeps" ]] || {
  echo "test-disk-preflight: CI attempted a host sweep" >&2; exit 1;
}

# Four contenders share one lock. The delayed sweep lets every process observe
# the low initial value; only the in-lock recheck stops later contenders.
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
for n in 1 2 3 4; do
  AWL_TEST_SWEEP_DELAY=0.3 run 9663676416 >"$WORK/contender-$n.out" &
done
wait
[[ "$(wc -l <"$WORK/sweeps")" -eq 1 ]] || {
  echo "test-disk-preflight: contention launched more than one sweep" >&2; exit 1;
}
grep -l 'status=reused-recovery' "$WORK"/contender-*.out >/dev/null || {
  echo "test-disk-preflight: contenders did not reuse the in-lock recovery" >&2; exit 1;
}

# MUTATION PROOF: deleting the in-lock recheck makes all stale contenders sweep.
mutated="$WORK/disk-preflight-without-recheck.sh"
cp "$PREFLIGHT" "$mutated"
perl -0pi -e 's/# DISK_PREFLIGHT_RECHECK\nfree_bytes="\$\(available_bytes\)"/# DISK_PREFLIGHT_RECHECK REMOVED\nfree_bytes=0/' "$mutated"
chmod +x "$mutated"
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
for n in 1 2 3 4; do
  AWL_TEST_SWEEP_DELAY=0.3 run 9663676416 "$mutated" >"$WORK/mutated-$n.out" &
done
wait
[[ "$(wc -l <"$WORK/sweeps")" -gt 1 ]] || {
  echo "test-disk-preflight: mutation removing recheck did not fail contention law" >&2; exit 1;
}

echo "test-disk-preflight: healthy, recovery, insufficiency, contention, and recheck mutation proved"
