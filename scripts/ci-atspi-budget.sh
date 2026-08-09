#!/usr/bin/env bash
# scripts/ci-atspi-budget.sh — run the AT-SPI probe under a budget that
# FAILS rather than lets the runner CANCEL.
#
# Reuses scripts/ci-wedge-budget.sh's exact pattern, not a fresh
# design: `continue-on-error: true` tolerates a job that FAILS. It does NOT
# tolerate a job that is CANCELLED, and a step or job that exceeds
# `timeout-minutes` is cancelled, not failed — cancellation propagates to the
# workflow's conclusion regardless of `continue-on-error`. The atspi job
# carries `continue-on-error: true`, but its probe can hang until
# `timeout-minutes: 20` cancelled the JOB, and that cancellation would have
# propagated to the whole workflow's conclusion had this job's failure mode
# not already been isolated to its own job. A tolerated
# arm that can still take the run down with it has not actually been made
# safe to fail.
#
# Converting the hang into an ordinary non-zero exit is what makes
# `continue-on-error` apply. `timeout-minutes` stays on the step as a
# BACKSTOP for the different failure it was written for — the hosted runner
# losing communication with the server — set ABOVE this budget so this
# budget fires first in the ordinary hang.
set -uo pipefail

binary="${1:?usage: ci-atspi-budget.sh <path-to-awl-binary> [budget_seconds]}"
budget="${2:-180}"

echo "ci-atspi-budget binary=${binary} budget_seconds=${budget}"

scripts/ci-atspi-probe.sh "$binary" &
probe_pid=$!

( sleep "$budget"; kill -9 "$probe_pid" 2>/dev/null ) &
watchdog_pid=$!

wait "$probe_pid"
status=$?

kill "$watchdog_pid" 2>/dev/null
wait "$watchdog_pid" 2>/dev/null

if (( status == 0 )); then
  echo "ci-atspi-budget status=0 — the probe passed within budget."
  exit 0
fi

# 137 is SIGKILL: the watchdog fired, which is the hang. Anything else is an
# ordinary probe failure (a fail() call, an assertion mismatch). Both are
# reported as a FAILED step so the job's `continue-on-error` can tolerate
# them; neither may cancel the workflow.
if (( status == 137 )); then
  echo "ci-atspi-budget status=137 — the probe HUNG past ${budget}s."
  echo "This is a wedge in the probe or the AT-SPI stack under it, not a clean bridge-liveness verdict; check the probe's own bounds before trusting this as 'no bridge'."
else
  echo "ci-atspi-budget status=${status} — the probe failed without hanging."
fi
exit "$status"
