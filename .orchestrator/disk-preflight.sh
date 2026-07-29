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

if (( free_bytes >= HEALTHY_BYTES )); then
  STALE_LOCK_RECLAIMED=0
  receipt healthy fleet "$free_bytes" "$HEALTHY_BYTES" "$MINIMUM_BYTES"
  exit 0
fi

STALE_LOCK_RECLAIMED=0
while ! mkdir "$LOCK" 2>/dev/null; do
  owner="$(sed -n 's/^pid=\([1-9][0-9]*\) caller=.*/\1/p' "$LOCK/owner" 2>/dev/null || true)"
  if [[ "$owner" =~ ^[1-9][0-9]*$ ]] && ! kill -0 "$owner" 2>/dev/null; then
    # A SIGKILL can skip the EXIT trap. Only a recorded, dead PID is safe to
    # reclaim; an unparseable or live lock remains another owner's boundary.
    rm -f "$LOCK/owner"
    rmdir "$LOCK" 2>/dev/null || true
    STALE_LOCK_RECLAIMED=1
    continue
  fi
  sleep 0.1
done
printf 'pid=%s caller=%s\n' "$$" "$CALLER" >"$LOCK/owner"
trap 'rm -f "$LOCK/owner"; rmdir "$LOCK" 2>/dev/null || true' EXIT

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
receipt recovered fleet "$free_bytes" "$HEALTHY_BYTES" "$MINIMUM_BYTES"
