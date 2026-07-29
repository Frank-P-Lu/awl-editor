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

healthy_bytes=$((40 * 1024 * 1024 * 1024))
insufficient_bytes=$((20 * 1024 * 1024 * 1024))
ci_capacity_bytes=$((3 * 1024 * 1024 * 1024))
stale_pid=$(( $$ + 100000000 ))
while kill -0 "$stale_pid" 2>/dev/null; do
  stale_pid=$((stale_pid + 100000000))
done

run() {
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$oracle" \
  AWL_DISK_PREFLIGHT_SWEEP_COMMAND="$sweep" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/lock" \
  AWL_TEST_FREE_FILE="$WORK/free" \
  AWL_TEST_SWEEP_LOG="$WORK/sweeps" \
  AWL_TEST_SWEEP_RESULT="${1:-$healthy_bytes}" \
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
printf '%s\n' "$healthy_bytes" >"$WORK/free"
: >"$WORK/sweeps"
healthy="$(run)"
[[ "$healthy" == *'status=healthy'* && ! -s "$WORK/sweeps" ]] || {
  echo "test-disk-preflight: healthy disk must not sweep" >&2; exit 1;
}

# Low space recovers through exactly the supplied sweep owner.
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
recovered="$(run "$healthy_bytes")"
[[ "$recovered" == *'status=recovered'* && "$(wc -l <"$WORK/sweeps")" -eq 1 ]] || {
  echo "test-disk-preflight: low disk did not recover through one sweep" >&2; exit 1;
}

# An unsuccessful recovery is a truthful early failure, before any Cargo call.
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
if run "$insufficient_bytes" >"$WORK/insufficient.out" 2>"$WORK/insufficient.err"; then
  echo "test-disk-preflight: insufficient recovery unexpectedly passed" >&2; exit 1
fi
grep -Fq 'insufficient space after sweep-1d' "$WORK/insufficient.err" || {
  echo "test-disk-preflight: insufficiency did not name the attempted recovery" >&2; exit 1;
}

# CI is portable capacity checking, not a second cleanup owner.
printf '%s\n' "$ci_capacity_bytes" >"$WORK/free"
: >"$WORK/sweeps"
ci_capacity="$(CI=1 run "$healthy_bytes")"
[[ "$ci_capacity" == *'status=ci-capacity'* && "$ci_capacity" == *'policy=ci'* && ! -s "$WORK/sweeps" ]] || {
  echo "test-disk-preflight: ordinary CI capacity required the local reserve or swept" >&2; exit 1;
}
printf '%s\n' 1073741824 >"$WORK/free"
if CI=1 run "$healthy_bytes" >"$WORK/ci.out" 2>"$WORK/ci.err"; then
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
  AWL_TEST_SWEEP_DELAY=0.3 run "$healthy_bytes" >"$WORK/contender-$n.out" &
done
wait
[[ "$(wc -l <"$WORK/sweeps")" -eq 1 ]] || {
  echo "test-disk-preflight: contention launched more than one sweep" >&2; exit 1;
}
grep -l 'status=reused-recovery' "$WORK"/contender-*.out >/dev/null || {
  echo "test-disk-preflight: contenders did not reuse the in-lock recovery" >&2; exit 1;
}

# Deterministic A/B/C stale handoff: every contender sees A's dead lock; the
# elected reclaimer must publish C before the others can acquire the path.
printf 'pid=%s caller=dead-owner\n' "$stale_pid" >"$WORK/lock"
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
for n in 1 2 3 4; do
  AWL_TEST_SWEEP_DELAY=0.3 run "$healthy_bytes" >"$WORK/stale-contender-$n.out" &
done
wait
[[ "$(wc -l <"$WORK/sweeps")" -eq 1 ]] || {
  echo "test-disk-preflight: stale A/B/C handoff launched more than one sweep" >&2; exit 1;
}
grep -l 'status=reused-recovery' "$WORK"/stale-contender-*.out >/dev/null || {
  echo "test-disk-preflight: stale A/B/C contenders did not preserve C's lock" >&2; exit 1;
}

# A process killed after metadata is written but before the hard-link publish
# leaves no lock path. The next owner proceeds instead of inheriting an empty
# or unparseable acquisition artifact.
kill_owner="$WORK/kill-owner.sh"
cat >"$kill_owner" <<'EOF'
#!/usr/bin/env bash
kill -KILL "$1"
EOF
chmod +x "$kill_owner"
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
if env AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$oracle" \
  AWL_DISK_PREFLIGHT_SWEEP_COMMAND="$sweep" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/lock" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_AFTER_METADATA_COMMAND="$kill_owner" \
  AWL_TEST_FREE_FILE="$WORK/free" \
  AWL_TEST_SWEEP_LOG="$WORK/sweeps" \
  AWL_TEST_SWEEP_RESULT="$healthy_bytes" \
  perl -e '$pid = fork; if (!$pid) { exec @ARGV } waitpid $pid, 0; exit (($? & 127) == 9 ? 42 : 1)' \
  "$PREFLIGHT" >/dev/null 2>&1; then
  killed_owner=0
else
  killed_owner=$?
fi
if [[ "$killed_owner" -ne 42 ]]; then
  echo "test-disk-preflight: death-before-publish fixture unexpectedly returned" >&2; exit 1
fi
[[ ! -e "$WORK/lock" ]] || {
  echo "test-disk-preflight: death-before-publish left an incomplete lock" >&2; exit 1;
}
run "$healthy_bytes" >/dev/null

# MUTATION PROOF: deleting the in-lock recheck makes all stale contenders sweep.
mutated="$WORK/disk-preflight-without-recheck.sh"
cp "$PREFLIGHT" "$mutated"
perl -0pi -e 's/# DISK_PREFLIGHT_RECHECK\nfree_bytes="\$\(available_bytes\)"/# DISK_PREFLIGHT_RECHECK REMOVED\nfree_bytes=0/' "$mutated"
chmod +x "$mutated"
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
for n in 1 2 3 4; do
  AWL_TEST_SWEEP_DELAY=0.3 run "$healthy_bytes" "$mutated" >"$WORK/mutated-$n.out" &
done
wait
[[ "$(wc -l <"$WORK/sweeps")" -gt 1 ]] || {
  echo "test-disk-preflight: mutation removing recheck did not fail contention law" >&2; exit 1;
}

# A SIGKILL can leave only a fully published lock file: there is no empty
# directory/owner-file window. A dead owner is reclaimed observably.
printf 'pid=%s caller=interrupted-worker\n' "$stale_pid" >"$WORK/lock"
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
stale_recovered="$(run "$healthy_bytes")"
[[ "$stale_recovered" == *'status=recovered'* && "$stale_recovered" == *'stale_lock_reclaimed=1'* && ! -e "$WORK/lock" ]] || {
  echo "test-disk-preflight: dead-owner lock was not safely reclaimed" >&2; exit 1;
}

# MUTATION PROOF: treating a dead owner as live must leave this fixture stuck.
stale_mutated="$WORK/disk-preflight-without-stale-recovery.sh"
cp "$PREFLIGHT" "$stale_mutated"
perl -0pi -e 's/&& ! kill -0 "\$owner"/&& kill -0 "\$owner"/' "$stale_mutated"
chmod +x "$stale_mutated"
printf 'pid=%s caller=interrupted-worker\n' "$stale_pid" >"$WORK/lock"
printf '%s\n' 1073741824 >"$WORK/free"
if AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$oracle" \
  AWL_DISK_PREFLIGHT_SWEEP_COMMAND="$sweep" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/lock" \
  AWL_TEST_FREE_FILE="$WORK/free" \
  AWL_TEST_SWEEP_LOG="$WORK/sweeps" \
  AWL_TEST_SWEEP_RESULT="$healthy_bytes" \
  perl -e '$pid = fork; if (!$pid) { exec @ARGV } $SIG{ALRM} = sub { kill "TERM", $pid; waitpid $pid, 0; exit 124 }; alarm 2; waitpid $pid, 0; exit 1' \
  "$stale_mutated" >"$WORK/stale-mutated.out" 2>"$WORK/stale-mutated.err"; then
  echo "test-disk-preflight: stale-owner mutation unexpectedly escaped the lock" >&2; exit 1
fi
rm -f "$WORK/lock"

# Cleanup is identity-safe. Replacing the published inode just before EXIT
# must leave the replacement behind for its new owner.
replacement="$WORK/replacement.sh"
cat >"$replacement" <<'EOF'
#!/usr/bin/env bash
rm -f "$1"
printf 'pid=%s caller=replacement-owner\n' "$$" >"$1"
EOF
chmod +x "$replacement"
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_BEFORE_CLEANUP_COMMAND="$replacement" run "$healthy_bytes" >/dev/null
grep -Fq 'caller=replacement-owner' "$WORK/lock" || {
  echo "test-disk-preflight: old cleanup removed a replacement lock inode" >&2; exit 1;
}
rm -f "$WORK/lock"

# The real df path consumes POSIX -P's 1 KiB Available field. This fixture is
# the common macOS/Linux shape and proves Bash receives an integer byte count.
fake_df="$WORK/df"
cat >"$fake_df" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' 'Filesystem 1024-blocks Used Available Capacity Mounted on'
printf '%s\n' '/dev/fake 100000000 0 41943040 0% /'
EOF
chmod +x "$fake_df"
df_receipt="$(PATH="$WORK:$PATH" CI=1 AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/df-lock" "$PREFLIGHT")"
[[ "$df_receipt" == *'free_bytes=42949672960'* && "$df_receipt" == *'status=ci-capacity'* ]] || {
  echo "test-disk-preflight: POSIX df fixture did not yield an integer byte count" >&2; exit 1;
}

echo "test-disk-preflight: healthy, recovery, CI, contention, stale-lock, and mutations proved"
