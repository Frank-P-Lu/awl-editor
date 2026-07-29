#!/usr/bin/env bash
# The one disk-recovery door for build/gate launchers. It never traverses a
# target itself: scripts/sweep.sh 1 remains the only cleanup owner.
set -euo pipefail

readonly HEALTHY_BYTES=$((8 * 1024 * 1024 * 1024))
readonly MINIMUM_BYTES=$((2 * 1024 * 1024 * 1024))
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
  df -Pk "$ROOT" | awk 'NR == 2 { print $4 * 1024 }'
}

receipt() {
  printf 'disk-preflight caller=%s status=%s free_bytes=%s healthy_bytes=%s minimum_bytes=%s\n' \
    "$CALLER" "$1" "$2" "$HEALTHY_BYTES" "$MINIMUM_BYTES"
}

fail_insufficient() {
  local free_bytes="$1" recovery="$2"
  printf 'disk-preflight: insufficient space after %s; free_bytes=%s minimum_bytes=%s healthy_bytes=%s\n' \
    "$recovery" "$free_bytes" "$MINIMUM_BYTES" "$HEALTHY_BYTES" >&2
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
if (( free_bytes >= HEALTHY_BYTES )); then
  receipt healthy "$free_bytes"
  exit 0
fi

# CI runners normally have a single fresh checkout and deliberately do not
# install cargo-sweep. Do not make CI's capacity a host-cleanup policy.
if [[ -n "${CI:-}" ]]; then
  fail_insufficient "$free_bytes" "ci-no-sweep"
fi

while ! mkdir "$LOCK" 2>/dev/null; do
  sleep 0.1
done
trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT

# A contender can have observed low space before the first owner recovered.
# The in-lock read is the concurrency boundary: without it every waiter sweeps.
# DISK_PREFLIGHT_RECHECK
free_bytes="$(available_bytes)"
if (( free_bytes >= HEALTHY_BYTES )); then
  receipt reused-recovery "$free_bytes"
  exit 0
fi

run_sweep
free_bytes="$(available_bytes)"
if (( free_bytes < MINIMUM_BYTES )); then
  fail_insufficient "$free_bytes" "sweep-1d"
fi
receipt recovered "$free_bytes"
