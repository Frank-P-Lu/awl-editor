#!/usr/bin/env bash
# The one disk-recovery door for build/gate launchers. It never traverses a
# target itself: scripts/sweep.sh 1 remains the only cleanup owner.
set -euo pipefail

# MINIMUM_BYTES is a CAPACITY floor and owes nothing to the recovery below it:
# four worker lanes need room before they all begin compiling, and a lane's
# target/ measured a mean of 4.8 GiB and a worst case of 10.0 GiB across this
# fleet. It is unchanged.
#
# SWEEP_YIELD_BYTES is what the recovery arm can actually give back, MEASURED
# rather than assumed, and it is small for two reasons no tuning changes.
# sweep.sh prunes the CALLING worktree and nothing else, and `cargo sweep
# --time` keeps every artifact whose fingerprint was used inside the window —
# so the one worktree this script may prune is the one worktree guaranteed to
# have used its artifacts minutes ago. And cargo-sweep never touches
# target/debug/incremental, which measured 61-69% of every target/ sampled
# here. Sampled across worktrees spanning 1.2-25.4 GiB and one to fourteen days
# old, `--time 1` reclaimed nothing at all, and neither did any threshold up to
# 60 days; the largest sweepable pool anywhere on the fleet was 2.8 GiB of deps
# and fingerprints in a lane that had just rebuilt. 3 GiB is that ceiling, not
# an expectation.
#
# So the band is DERIVED: begin recovering exactly one sweep's worth of
# headroom above the floor this script refuses at. The previous 8 GiB band was
# tuned against a sweep that traversed the whole fleet, and a band wider than
# the yield buys only a serialized lock and two du(1) traversals per worker
# command in exchange for a recovery that cannot arrive. scripts/
# test-disk-preflight.sh pins the derivation.
readonly MINIMUM_BYTES=$((24 * 1024 * 1024 * 1024))
readonly SWEEP_YIELD_BYTES=$((3 * 1024 * 1024 * 1024))
readonly HEALTHY_BYTES=$((MINIMUM_BYTES + SWEEP_YIELD_BYTES))
# CI gets one disposable checkout rather than a local four-lane build fleet.
# Keep its explicit capacity floor well above the tiny volumes that make Cargo
# error messages misleading, without demanding the local reserve.
readonly CI_MINIMUM_BYTES=$((2 * 1024 * 1024 * 1024))
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CALLER="${AWL_DISK_PREFLIGHT_CALLER:-build}"
LOCK="${TMPDIR:-/tmp}/awl-disk-preflight.lock"
if [[ "${AWL_DISK_PREFLIGHT_TEST_MODE:-}" == 1 && -n "${AWL_DISK_PREFLIGHT_LOCK_DIR:-}" ]]; then
  LOCK="$AWL_DISK_PREFLIGHT_LOCK_DIR"
fi

available_bytes() {
  if [[ "${AWL_DISK_PREFLIGHT_TEST_MODE:-}" == 1 \
    && -n "${AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND:-}" ]]; then
    "${AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND}"
    return
  fi
  # POSIX -P keeps the available-block field stable on macOS and Linux. The
  # orchestration scripts are Bash-only; wasm itself never runs this probe.
  df -Pk "$ROOT" | awk 'NR == 2 { printf "%.0f\n", $4 * 1024 }'
}

# Every receipt states the policy it ran under, including what the recovery arm
# was expected to yield and what it actually did. A floor tuned from memory is
# how the 8 GiB band outlived the fleet-wide sweep it was sized for; the next
# person to tune these reads the numbers off a run instead.
receipt() {
  printf 'disk-preflight caller=%s status=%s policy=%s serializer=flock free_bytes=%s healthy_bytes=%s minimum_bytes=%s sweep_yield_bytes=%s reclaimed_bytes=%s\n' \
    "$CALLER" "$1" "$2" "$3" "$4" "$5" "$6" "$7"
}

fail_insufficient() {
  local free_bytes="$1" recovery="$2" minimum_bytes="$3" healthy_bytes="$4" policy="$5" reclaimed_bytes="$6"
  printf 'disk-preflight: insufficient space after %s; policy=%s free_bytes=%s minimum_bytes=%s healthy_bytes=%s reclaimed_bytes=%s\n' \
    "$recovery" "$policy" "$free_bytes" "$minimum_bytes" "$healthy_bytes" "$reclaimed_bytes" >&2
  exit 1
}

run_sweep() {
  if [[ "${AWL_DISK_PREFLIGHT_TEST_MODE:-}" == 1 \
    && -n "${AWL_DISK_PREFLIGHT_SWEEP_COMMAND:-}" ]]; then
    "${AWL_DISK_PREFLIGHT_SWEEP_COMMAND}"
  else
    "$ROOT/scripts/sweep.sh" 1
  fi
}

serializer_fd_is_authoritative() {
  AWL_DISK_PREFLIGHT_LOCK_PATH="$LOCK" perl -e '
    use Fcntl qw(:flock);
    open my $lock, ">&=9" or exit 1;
    my @held = stat($lock);
    my @path = stat($ENV{AWL_DISK_PREFLIGHT_LOCK_PATH});
    exit 1 unless @held && @path && $held[0] == $path[0] && $held[1] == $path[1];
    flock($lock, LOCK_EX | LOCK_NB) or exit 1;
  ' 2>/dev/null
}

SERIALIZER_HELD=0
if serializer_fd_is_authoritative; then
  SERIALIZER_HELD=1
fi

free_bytes="$(available_bytes)"
# CI runners normally have a single fresh checkout and deliberately do not
# install cargo-sweep. Do not make CI's capacity a host-cleanup policy.
if [[ -n "${CI:-}" ]]; then
  if (( free_bytes < CI_MINIMUM_BYTES )); then
    fail_insufficient "$free_bytes" "ci-no-sweep" "$CI_MINIMUM_BYTES" "$CI_MINIMUM_BYTES" ci none
  fi
  receipt ci-capacity ci "$free_bytes" "$CI_MINIMUM_BYTES" "$CI_MINIMUM_BYTES" 0 none
  exit 0
fi

if (( free_bytes >= HEALTHY_BYTES )) && (( SERIALIZER_HELD == 0 )); then
  receipt healthy fleet "$free_bytes" "$HEALTHY_BYTES" "$MINIMUM_BYTES" "$SWEEP_YIELD_BYTES" none
  exit 0
fi

test_hook() {
  local command_var="$1"
  shift
  if [[ "${AWL_DISK_PREFLIGHT_TEST_MODE:-}" == 1 && -n "${!command_var:-}" ]]; then
    "${!command_var}" "$@"
  fi
}

# Advisory flock is released by the kernel when its owner dies. The Perl core
# wrapper keeps the locked descriptor across exec, so Bash never has to infer
# or reclaim stale ownership from a PID or pathname.
if (( SERIALIZER_HELD == 0 )); then
  AWL_DISK_PREFLIGHT_LOCK_PATH="$LOCK" \
    exec perl -e 'use Fcntl qw(:flock F_SETFD); use POSIX qw(dup2); open my $lock, ">>", $ENV{AWL_DISK_PREFLIGHT_LOCK_PATH} or die "disk-preflight: cannot open lock: $!\n"; flock($lock, LOCK_EX) or die "disk-preflight: cannot lock: $!\n"; dup2(fileno($lock), 9) >= 0 or die "disk-preflight: cannot preserve lock: $!\n"; open my $keep, ">&=9" or die "disk-preflight: cannot retain lock: $!\n"; fcntl($keep, F_SETFD, 0) or die "disk-preflight: cannot preserve lock: $!\n"; exec {$ARGV[0]} @ARGV or die "disk-preflight: cannot restart: $!\n";' \
    /usr/bin/env bash "${BASH_SOURCE[0]}"
fi

test_hook AWL_DISK_PREFLIGHT_AFTER_SERIALIZER_COMMAND "$$"

# A contender can have observed low space before the first owner recovered.
# The in-lock read is the concurrency boundary: without it every waiter sweeps.
# DISK_PREFLIGHT_RECHECK
free_bytes="$(available_bytes)"
if (( free_bytes >= HEALTHY_BYTES )); then
  receipt reused-recovery fleet "$free_bytes" "$HEALTHY_BYTES" "$MINIMUM_BYTES" "$SWEEP_YIELD_BYTES" none
  exit 0
fi

free_before_sweep="$free_bytes"
run_sweep
free_bytes="$(available_bytes)"
reclaimed_bytes=$(( free_bytes - free_before_sweep ))
if (( free_bytes < MINIMUM_BYTES )); then
  fail_insufficient "$free_bytes" "sweep-1d" "$MINIMUM_BYTES" "$HEALTHY_BYTES" fleet "$reclaimed_bytes"
fi
receipt recovered fleet "$free_bytes" "$HEALTHY_BYTES" "$MINIMUM_BYTES" "$SWEEP_YIELD_BYTES" "$reclaimed_bytes"
