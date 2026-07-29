#!/usr/bin/env bash
# The one disk-recovery door for build/gate launchers. It never traverses a
# target itself: scripts/sweep.sh 1 remains the only cleanup owner.
set -euo pipefail

# A current worktree target is about 6.3 GiB. Four worker lanes need headroom
# before they all begin compiling, so this is a fleet floor, not Cargo's own
# per-worktree estimate.
readonly HEALTHY_BYTES=$((32 * 1024 * 1024 * 1024))
readonly MINIMUM_BYTES=$((24 * 1024 * 1024 * 1024))
# CI gets one disposable checkout rather than a local four-lane build fleet.
# Keep its explicit capacity floor well above the tiny volumes that make Cargo
# error messages misleading, without demanding the local reserve.
readonly CI_MINIMUM_BYTES=$((2 * 1024 * 1024 * 1024))
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CALLER="${AWL_DISK_PREFLIGHT_CALLER:-build}"
LOCK="${AWL_DISK_PREFLIGHT_LOCK_DIR:-${TMPDIR:-/tmp}/awl-disk-preflight.lock}"

available_bytes() {
  if [[ -n "${AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND:-}" ]]; then
    "${AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND}"
    return
  fi
  # POSIX -P keeps the available-block field stable on macOS and Linux. The
  # orchestration scripts are Bash-only; wasm itself never runs this probe.
  df -Pk "$ROOT" | awk 'NR == 2 { printf "%.0f\n", $4 * 1024 }'
}

receipt() {
  printf 'disk-preflight caller=%s status=%s policy=%s free_bytes=%s healthy_bytes=%s minimum_bytes=%s stale_lock_reclaimed=%s\n' \
    "$CALLER" "$1" "$2" "$3" "$4" "$5" "$STALE_LOCK_RECLAIMED"
}

fail_insufficient() {
  local free_bytes="$1" recovery="$2" minimum_bytes="$3" healthy_bytes="$4" policy="$5"
  printf 'disk-preflight: insufficient space after %s; policy=%s free_bytes=%s minimum_bytes=%s healthy_bytes=%s\n' \
    "$recovery" "$policy" "$free_bytes" "$minimum_bytes" "$healthy_bytes" >&2
  exit 1
}

run_sweep() {
  if [[ -n "${AWL_DISK_PREFLIGHT_SWEEP_COMMAND:-}" ]]; then
    "${AWL_DISK_PREFLIGHT_SWEEP_COMMAND}"
  else
    "$ROOT/scripts/sweep.sh" 1
  fi
}

free_bytes="$(available_bytes)"
# CI runners normally have a single fresh checkout and deliberately do not
# install cargo-sweep. Do not make CI's capacity a host-cleanup policy.
if [[ -n "${CI:-}" ]]; then
  if (( free_bytes < CI_MINIMUM_BYTES )); then
    fail_insufficient "$free_bytes" "ci-no-sweep" "$CI_MINIMUM_BYTES" "$CI_MINIMUM_BYTES" ci
  fi
  STALE_LOCK_RECLAIMED=0
  receipt ci-capacity ci "$free_bytes" "$CI_MINIMUM_BYTES" "$CI_MINIMUM_BYTES"
  exit 0
fi

if (( free_bytes >= HEALTHY_BYTES )) && [[ -z "${AWL_DISK_PREFLIGHT_SERIALIZED:-}" ]]; then
  STALE_LOCK_RECLAIMED=0
  receipt healthy fleet "$free_bytes" "$HEALTHY_BYTES" "$MINIMUM_BYTES"
  exit 0
fi

STALE_LOCK_RECLAIMED=0
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
if [[ -z "${AWL_DISK_PREFLIGHT_SERIALIZED:-}" ]]; then
  AWL_DISK_PREFLIGHT_LOCK_PATH="$LOCK" \
    exec perl -e 'use Fcntl qw(:flock F_SETFD); use POSIX qw(dup2); open my $lock, ">>", $ENV{AWL_DISK_PREFLIGHT_LOCK_PATH} or die "disk-preflight: cannot open lock: $!\n"; flock($lock, LOCK_EX) or die "disk-preflight: cannot lock: $!\n"; dup2(fileno($lock), 9) >= 0 or die "disk-preflight: cannot preserve lock: $!\n"; open my $keep, ">&=9" or die "disk-preflight: cannot retain lock: $!\n"; fcntl($keep, F_SETFD, 0) or die "disk-preflight: cannot preserve lock: $!\n"; $ENV{AWL_DISK_PREFLIGHT_SERIALIZED} = 1; exec {$ARGV[0]} @ARGV or die "disk-preflight: cannot restart: $!\n";' \
    /usr/bin/env bash "${BASH_SOURCE[0]}"
fi

test_hook AWL_DISK_PREFLIGHT_AFTER_SERIALIZER_COMMAND "$$"

# A contender can have observed low space before the first owner recovered.
# The in-lock read is the concurrency boundary: without it every waiter sweeps.
# DISK_PREFLIGHT_RECHECK
free_bytes="$(available_bytes)"
if (( free_bytes >= HEALTHY_BYTES )); then
  receipt reused-recovery fleet "$free_bytes" "$HEALTHY_BYTES" "$MINIMUM_BYTES"
  exit 0
fi

run_sweep
free_bytes="$(available_bytes)"
if (( free_bytes < MINIMUM_BYTES )); then
  fail_insufficient "$free_bytes" "sweep-1d" "$MINIMUM_BYTES" "$HEALTHY_BYTES" fleet
fi
test_hook AWL_DISK_PREFLIGHT_BEFORE_CLEANUP_COMMAND "$LOCK"
receipt recovered fleet "$free_bytes" "$HEALTHY_BYTES" "$MINIMUM_BYTES"
