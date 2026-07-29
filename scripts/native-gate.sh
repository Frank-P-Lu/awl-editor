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
echo "==> native suite (mac convention)"
"${mac_command[@]}"
echo "==> native suite (linux convention)"
"${linux_command[@]}"

end_commit="$(git rev-parse HEAD)"
if [[ "$start_commit" != "$end_commit" ]]; then
  echo "native-gate: HEAD changed while the suite ran (start=$start_commit end=$end_commit); no receipt issued" >&2
  exit 1
fi

printf 'native-gate-receipt commit=%s conventions=mac,linux scope=all-targets\n' "$end_commit"
