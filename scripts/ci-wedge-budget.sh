#!/usr/bin/env bash
# scripts/ci-wedge-budget.sh — run item 231's wedge under a budget that FAILS
# rather than lets the runner CANCEL.
#
# WHY THIS EXISTS, and it is not the same thing as `timeout-minutes`.
# GitHub's `continue-on-error: true` tolerates a job that FAILS. It does not
# tolerate a job that is CANCELLED, and a step or job that exceeds its
# `timeout-minutes` is cancelled, not failed. Cancellation then propagates to
# the workflow's conclusion. So the first run after item 243's split
# (30825396088) had every gating job green — `mac (build + test, minus
# render::tests)`, `linux`, `web`, `mac live-probe` — and still concluded
# `cancelled`, purely because the tolerated wedge job hit its step timeout.
# That defeats the split's whole purpose, which is that `main` is not blocked
# by a known hang.
#
# Converting the hang into an ordinary non-zero exit is what makes
# `continue-on-error` apply. This is the same move `native-gate.sh` makes with
# its own budget — "convert an OUTCOME NOBODY CAN READ into one anybody can" —
# reduced to the one thing this job needs, because that script's phase and
# heartbeat reporting is coupled to a script this job deliberately never calls.
#
# `timeout-minutes` stays on the step as a BACKSTOP for the different failure
# it was written for: the hosted runner losing communication with the server
# (upstream actions/runner-images#13882), which no in-process watchdog can
# survive. It is set ABOVE this budget so this budget fires first in the
# ordinary hang, and the runner's cancellation is reached only when the
# watchdog itself is gone.
set -uo pipefail

# The test FILTER is an argument rather than baked in, so the workflow step
# still names `render::tests::` in the file itself. That keeps item 243's
# promise — a red job is attributable from the workflow file alone — and keeps
# `code-health.py`'s `mac-split-audit` reading the real scope instead of
# following an indirection into this script.
convention="${1:?usage: ci-wedge-budget.sh <mac|linux> <test_filter> [budget_seconds]}"
filter="${2:?usage: ci-wedge-budget.sh <mac|linux> <test_filter> [budget_seconds]}"
budget="${3:-1500}"

echo "ci-wedge-budget convention=${convention} filter=${filter} budget_seconds=${budget}"

env "AWL_CONVENTION_FORCE=${convention}" cargo test "$filter" &
test_pid=$!

( sleep "$budget"; kill -9 "$test_pid" 2>/dev/null ) &
watchdog_pid=$!

wait "$test_pid"
status=$?

kill "$watchdog_pid" 2>/dev/null
wait "$watchdog_pid" 2>/dev/null

if (( status == 0 )); then
  echo "ci-wedge-budget convention=${convention} status=0 — the wedge did NOT hang."
  echo "Do not read this as item 231 fixed until it stays green; see item 231."
  exit 0
fi

# 137 is SIGKILL: the watchdog fired, which is the hang. Anything else is an
# ordinary test failure. Both are reported as a FAILED step so that the job's
# `continue-on-error` can tolerate them; neither may cancel the workflow.
if (( status == 137 )); then
  echo "ci-wedge-budget convention=${convention} status=137 — HUNG past ${budget}s."
  echo "This is item 231's signature. The job is allowed to fail; the workflow is not cancelled."
else
  echo "ci-wedge-budget convention=${convention} status=${status} — failed without hanging."
  echo "A wedge job that FAILS rather than hangs is NOT item 231's signature; check it."
fi
exit "$status"
