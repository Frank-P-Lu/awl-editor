#!/usr/bin/env bash
# The orchestration-owned build launch seam for concurrently dispatched workers.
# Root merge-train gates, CI, and a developer's lone build deliberately do not
# use this wrapper and retain Cargo's hardware-adaptive default.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if (( $# == 0 )); then
  echo "usage: .orchestrator/worker-build.sh <build-or-gate-command> [args…]" >&2
  exit 2
fi

readonly WORKER_CARGO_JOBS=2
# CARGO_BUILD_JOBS bounds COMPILATION parallelism only. native-gate.sh runs
# both keymap conventions concurrently, and each `cargo test` defaults its
# harness thread count to the core count — so four workers at the gate phase,
# each left to that default, schedule 4 workers x 2 conventions x (core count)
# runnable test threads in aggregate, independent of the build budget above.
# One thread per worker's convention keeps four workers' aggregate test
# threads (4 x 2 x 1 = 8) matching the build budget's own aggregate (4 x 2 =
# 8) instead of several times the host's core count.
readonly WORKER_TEST_THREADS=1
export CARGO_BUILD_JOBS="$WORKER_CARGO_JOBS"
export RUST_TEST_THREADS="$WORKER_TEST_THREADS"
AWL_DISK_PREFLIGHT_CALLER=worker-build "$ROOT/.orchestrator/disk-preflight.sh"
printf 'orchestrator-worker-budget cargo_jobs=%s test_threads=%s command=' \
  "$WORKER_CARGO_JOBS" "$WORKER_TEST_THREADS"
printf '%q ' "$@"
printf '\n'
exec "$@"
