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

run() {
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$oracle" \
  AWL_DISK_PREFLIGHT_SWEEP_COMMAND="$sweep" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/lock" \
  AWL_TEST_FREE_FILE="$WORK/free" \
  AWL_TEST_SWEEP_LOG="$WORK/sweeps" \
  AWL_TEST_SWEEP_RESULT="${1:-$healthy_bytes}" \
  "${2:-$PREFLIGHT}"
}

wait_for_file() {
  local path="$1" attempts=0
  until [[ -e "$path" ]]; do
    attempts=$((attempts + 1))
    if (( attempts > 200 )); then
      echo "test-disk-preflight: timed out waiting for $path" >&2
      exit 1
    fi
    sleep 0.01
  done
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

# Perl clears close-on-exec on FD 9 before it execs Bash. The hook runs from
# that restarted Bash and proves the descriptor, rather than the lock pathname,
# is the live serialization authority.
fd_probe="$WORK/fd-probe.sh"
cat >"$fd_probe" <<'EOF'
#!/usr/bin/env bash
test -e /dev/fd/9
printf 'fd9-survived\n' >"$AWL_TEST_FD_PROBE"
EOF
chmod +x "$fd_probe"
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_AFTER_SERIALIZER_COMMAND="$fd_probe" \
  AWL_TEST_FD_PROBE="$WORK/fd-probe.out" run "$healthy_bytes" >/dev/null
[[ "$(cat "$WORK/fd-probe.out")" == fd9-survived ]] || {
  echo "test-disk-preflight: serializer descriptor did not survive exec" >&2; exit 1;
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

# Old marker variables and an unrelated inherited FD 9 are forgeable inputs,
# not capabilities. All four contenders must still serialize to one sweep.
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
: >"$WORK/not-the-lock"
for n in 1 2 3 4; do
  AWL_DISK_PREFLIGHT_SERIALIZED=1 \
    AWL_TEST_SWEEP_DELAY=0.3 run "$healthy_bytes" >"$WORK/forged-$n.out" \
    9>"$WORK/not-the-lock" &
done
wait
[[ "$(wc -l <"$WORK/sweeps")" -eq 1 ]] || {
  echo "test-disk-preflight: forgeable environment or FD bypassed flock" >&2; exit 1;
}
grep -l 'status=reused-recovery' "$WORK"/forged-*.out >/dev/null || {
  echo "test-disk-preflight: forged contenders did not reuse serialized recovery" >&2; exit 1;
}

# MUTATION PROOF: omitting both blocking and capability-probe flock calls makes
# the restarted Bashes race; the one-sweep contention law must turn red.
fd_mutated="$WORK/disk-preflight-without-fd9.sh"
cp "$PREFLIGHT" "$fd_mutated"
perl -0pi -e 's/flock\(\$lock, LOCK_EX \| LOCK_NB\) or exit 1;/1;/; s/flock\(\$lock, LOCK_EX\) or die "disk-preflight: cannot lock: \$!\\n";/1;/' "$fd_mutated"
chmod +x "$fd_mutated"
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
for n in 1 2 3 4; do
  AWL_TEST_SWEEP_DELAY=0.3 run "$healthy_bytes" "$fd_mutated" >"$WORK/fd-mutant-$n.out" &
done
wait
[[ "$(wc -l <"$WORK/sweeps")" -gt 1 ]] || {
  echo "test-disk-preflight: omitting flock mutation did not break contention" >&2; exit 1;
}

# A waiter is already blocked behind an independently held kernel lock when the
# owner is killed. The kernel releases it and that exact waiter performs the
# sole sweep; no PID or pathname reclamation participates.
printf '%s\n' 1073741824 >"$WORK/free"
: >"$WORK/sweeps"
perl -e 'use Fcntl qw(:flock); open my $lock, ">>", $ARGV[0] or die $!; flock($lock, LOCK_EX) or die $!; open my $ready, ">", $ARGV[1] or die $!; print {$ready} "$$\n"; close $ready; sleep 60' \
  "$WORK/lock" "$WORK/owner-ready" &
killed_owner_job=$!
wait_for_file "$WORK/owner-ready"
killed_owner="$(cat "$WORK/owner-ready")"
run "$healthy_bytes" >"$WORK/takeover-waiter.out" &
takeover_waiter=$!
sleep 0.1
[[ ! -s "$WORK/sweeps" ]] || {
  echo "test-disk-preflight: waiter swept before owning flock" >&2; exit 1;
}
kill -KILL "$killed_owner"
if wait "$killed_owner_job"; then
  echo "test-disk-preflight: serializer SIGKILL fixture unexpectedly returned" >&2; exit 1
else
  killed_owner_status=$?
fi
[[ "$killed_owner_status" -eq 137 ]] || {
  echo "test-disk-preflight: serializer owner did not die by SIGKILL" >&2; exit 1;
}
wait "$takeover_waiter"
[[ "$(wc -l <"$WORK/sweeps")" -eq 1 ]] || {
  echo "test-disk-preflight: SIGKILL waiter takeover did not run one sweep" >&2; exit 1;
}

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

# The real df path consumes POSIX -P's 1 KiB Available field. This fixture is
# the common macOS/Linux shape and proves Bash receives an integer byte count.
fake_df="$WORK/df"
cat >"$fake_df" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' 'Filesystem 1024-blocks Used Available Capacity Mounted on'
printf '%s\n' '/dev/fake 100000000 0 41943040 0% /'
EOF
chmod +x "$fake_df"
df_receipt="$(PATH="$WORK:$PATH" CI=1 AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/df-lock" "$PREFLIGHT")"
[[ "$df_receipt" == *'free_bytes=42949672960'* && "$df_receipt" == *'status=ci-capacity'* ]] || {
  echo "test-disk-preflight: POSIX df fixture did not yield an integer byte count" >&2; exit 1;
}

echo "test-disk-preflight: healthy, recovery, CI, flock contention, SIGKILL takeover, and mutations proved"
