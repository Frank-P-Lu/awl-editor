#!/usr/bin/env bash
# The cheap preflight: format/lint/source-audit checks, plus the
# benchmark-output and view-construction ownership audits, run BEFORE any
# GPU-touching test. See docs/verification.md "Order the work by cost" — this
# is step 2 (format, source audits, compiler/lint, targeted tests), run
# standalone so a violation is caught without waiting on step 3's GPU sweep.
#
# This composes existing check owners rather than restating their rules:
# scripts/code-health.sh already owns fmt/clippy/cargo-machete/the static
# source audits, and src/println_audit.rs / src/view_policy.rs already own
# the two ownership-audit assertions. This script adds no new rule of its
# own — only earlier, standalone ordering, so a lane or the orchestrator gets
# a fast answer instead of waiting on scripts/native-gate.sh's full sweep.
#
# NOT a full receipt. It proves nothing about GPU-touching behavior, the
# menu-bar axis, or wasm — see scripts/native-gate.sh and scripts/web-smoke.sh
# for those. A pass here is targeted evidence, never cited as "the full
# native suite ran."
set -euo pipefail

if (( $# != 0 )); then
  echo "preflight: no arguments are accepted; this always runs the same fixed steps" >&2
  exit 2
fi

preflight_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$preflight_root"

preflight_start=$(date +%s)

echo "== preflight 1/2: format, lint, and source audits (scripts/code-health.sh) =="
scripts/code-health.sh

echo
echo "== preflight 2/2: benchmark-output and view-construction ownership audits =="
# src/println_audit.rs (application diagnostics route through the notice
# seam; benchmark diagnostics enroll under BENCHMARK_MODULE_PATHS) and
# src/view_policy.rs (capture and the live App route through one shared
# policy owner) are pure, file-scanning unit tests: no GPU device, no
# crate::testlock hold, so they are safe and fast to isolate from the rest of
# the suite. The filter selects only these two modules — everything else in
# the binary, GPU tests included, is left untouched and unbuilt-for-execution.
cargo test --bin awl -- println_audit view_policy

preflight_elapsed=$(( $(date +%s) - preflight_start ))
echo
printf 'preflight: PASSED in %ss — targeted evidence only (format, lint, source\n' "$preflight_elapsed"
printf '  audits, benchmark-output + view-construction ownership). No GPU test ran,\n'
printf '  no menu-bar axis, no wasm. Run scripts/native-gate.sh (+ scripts/web-smoke.sh)\n'
printf '  for a full receipt before claiming the native/wasm suite passed.\n'
