#!/usr/bin/env bash
# The one native full-suite gate. A receipt from this script, on the commit it
# names, is the only evidence that both supported conventions ran every native
# Cargo test target. `cargo test --bin awl` is binary unit tests, not this gate.
set -euo pipefail

if (( $# != 0 )); then
  echo "native-gate: target selection and test-name arguments are forbidden; run targeted tests directly" >&2
  exit 2
fi

gate_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AWL_DISK_PREFLIGHT_CALLER=native-gate "$gate_root/.orchestrator/disk-preflight.sh"

start_commit="$(git rev-parse HEAD)"

# This is deliberately an integration target, outside the binary unit-test
# target. Its first position makes integration-test discovery disappear loudly.
canary_command=(cargo test --test native_gate_canary)
mac_command=(env AWL_CONVENTION_FORCE=mac cargo test)
linux_command=(env AWL_CONVENTION_FORCE=linux cargo test)

echo "==> native integration canary"
"${canary_command[@]}"

# The canary fronts dependency and library compilation. Cargo's shared-target
# lock prevents duplicate remaining compilation when these siblings start; in
# worker lanes both also inherit the orchestration-owned Cargo cap.
echo "==> native suites (mac and linux conventions, concurrent)"
"${mac_command[@]}" &
mac_pid=$!
"${linux_command[@]}" &
linux_pid=$!

# `wait` is allowed to report failure without set -e ending the gate before the
# sibling has finished. Preserve both statuses; neither convention can hide the
# other or authorize a receipt on partial coverage.
set +e
wait "$mac_pid"
mac_status=$?
wait "$linux_pid"
linux_status=$?
set -e

if (( mac_status != 0 || linux_status != 0 )); then
  printf 'native-gate: suite failure mac_status=%s linux_status=%s; no receipt issued\n' \
    "$mac_status" "$linux_status" >&2
  exit 1
fi

end_commit="$(git rev-parse HEAD)"
if [[ "$start_commit" != "$end_commit" ]]; then
  echo "native-gate: HEAD changed while the suite ran (start=$start_commit end=$end_commit); no receipt issued" >&2
  exit 1
fi

printf 'native-gate-receipt commit=%s conventions=mac,linux scope=all-targets\n' "$end_commit"
