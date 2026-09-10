#!/usr/bin/env bash
# The cheap preflight: format/lint/source-audit checks, plus the
# benchmark-output, document-substitution, and view-construction ownership
# audits, run BEFORE any
# GPU-touching test. See docs/verification.md "Order the work by cost" — this
# is step 2 (format, source audits, compiler/lint, targeted tests), run
# standalone so a violation is caught without waiting on step 3's GPU sweep.
#
# This composes existing check owners rather than restating their rules:
# scripts/code-health.sh already owns fmt/clippy/cargo-machete/the static
# source audits, while src/println_audit.rs, src/card/figures/tests.rs, and
# src/view_policy.rs already own the three ownership-audit assertions. This
# script adds no new rule of its
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

# Test-only escape hatches, same shape as native-gate.sh's
# AWL_NATIVE_GATE_PROBE_HEALTH_COMMAND: scripts/test-preflight.sh stubs both
# steps so it can exercise ordering and failure-propagation without compiling
# clippy or the test binary (which would also recurse into this very script,
# since the real health command IS scripts/code-health.sh).
preflight_health_command=("$preflight_root/scripts/code-health.sh")
if [[ -n "${AWL_PREFLIGHT_PROBE_HEALTH_COMMAND:-}" ]]; then
  read -r -a preflight_health_command <<<"$AWL_PREFLIGHT_PROBE_HEALTH_COMMAND"
fi
# The filters are deliberately separate: Cargo accepts one substring filter,
# and the document-substitution law has no common selector with println_audit.
# Keep the view-policy audit too: its exhaustive live/capture consumer roster
# is the remaining view-construction ownership coverage, distinct from which
# path may replace a ViewState's document text.
preflight_print_audit_command=(cargo test --bin awl -- println_audit)
preflight_substitution_audit_command=(cargo test --bin awl -- card::figures::tests::only_the_substitution_door_replaces_a_view_states_text)
preflight_view_policy_command=(cargo test --bin awl -- view_policy)
if [[ -n "${AWL_PREFLIGHT_PROBE_AUDIT_COMMAND:-}" ]]; then
  read -r -a preflight_print_audit_command <<<"$AWL_PREFLIGHT_PROBE_AUDIT_COMMAND"
  preflight_substitution_audit_command=("${preflight_print_audit_command[@]}")
  preflight_view_policy_command=("${preflight_print_audit_command[@]}")
fi

preflight_start=$(date +%s)

echo "== preflight 1/4: format, lint, and source audits (scripts/code-health.sh) =="
"${preflight_health_command[@]}"

echo
echo "== preflight 2/4: benchmark-output ownership audit =="
# src/println_audit.rs checks that application diagnostics route through the
# notice seam and benchmark diagnostics enroll under BENCHMARK_MODULE_PATHS.
"${preflight_print_audit_command[@]}"

echo
echo "== preflight 3/4: document-substitution ownership audit =="
# card::figures::tests owns the exhaustive production roster for replacement
# of ViewState text. It must run here, rather than an adjacent view-policy
# test, so a substitute that bypasses ViewState::substitute_text is stopped
# before a GPU test can run.
"${preflight_substitution_audit_command[@]}"

echo
echo "== preflight 4/4: live/capture view-policy ownership audit =="
# src/view_policy.rs owns the shared policy API and its exhaustive consumer
# roster. It is related view-construction coverage, but not a replacement for
# the document-substitution law above. All three audits are pure source/unit
# tests: no GPU device and no crate::testlock hold; filtered execution leaves
# the GPU tests untouched.
"${preflight_view_policy_command[@]}"

preflight_elapsed=$(( $(date +%s) - preflight_start ))
echo
printf 'preflight: PASSED in %ss — targeted evidence only, NOT a full receipt (format,\n' "$preflight_elapsed"
printf '  lint, source audits, benchmark-output + document-substitution + view-policy\n'
printf '  ownership). No GPU test ran, no menu-bar axis, no wasm. Run scripts/native-gate.sh (+\n'
printf '  scripts/web-smoke.sh) for a full receipt before claiming the native/wasm\n'
printf '  suite passed.\n'
