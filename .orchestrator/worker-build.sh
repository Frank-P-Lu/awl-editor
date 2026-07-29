#!/usr/bin/env bash
# The orchestration-owned build launch seam for concurrently dispatched workers.
# Root merge-train gates, CI, and a developer's lone build deliberately do not
# use this wrapper and retain Cargo's hardware-adaptive default.
set -euo pipefail

if (( $# == 0 )); then
  echo "usage: .orchestrator/worker-build.sh <build-or-gate-command> [args…]" >&2
  exit 2
fi

readonly WORKER_CARGO_JOBS=2
export CARGO_BUILD_JOBS="$WORKER_CARGO_JOBS"
printf 'orchestrator-worker-budget cargo_jobs=%s command=' "$WORKER_CARGO_JOBS"
printf '%q ' "$@"
printf '\n'
exec "$@"
