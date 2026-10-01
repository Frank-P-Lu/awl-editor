#!/usr/bin/env bash
# Three verification layers; existing commands retain ownership of their checks.
set -euo pipefail
verify_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$verify_root"
if [[ "${AWL_PROJECT_RUST_ACTIVE:-}" != "$verify_root" ]]; then
  exec "$verify_root/scripts/project-rust.sh" "$verify_root/scripts/verify.sh" "$@"
fi

usage() {
  cat <<'USAGE'
Usage: scripts/verify.sh fast [--lint] UNIT_FILTER...
       scripts/verify.sh full
       scripts/verify.sh extended [pretag-journeys.py options]

fast: format, compiler, ownership audits and explicitly selected unit tests.
      --lint also checks all targets/features with Clippy. Never a full receipt.
full: clean frozen commit; full native health/tests, wasm runtime, profile parity.
extended: release journeys across worlds/DPI; subsets are targeted evidence only.
See docs/verification.md for integration targets, mutations and visual review.
USAGE
}
mode="${1:-}"
[[ $# -eq 0 ]] || shift
case "$mode" in
  --help|-h) usage; exit 0 ;;
  fast)
    lint=0
    if [[ "${1:-}" == --lint ]]; then lint=1; shift; fi
    if (( $# == 0 )); then
      echo "verify fast: name the unit-test filters relevant to your diff" >&2
      exit 2
    fi
    for filter in "$@"; do
      if [[ -z "$filter" || "$filter" == -* ]]; then
        echo "verify fast: filters must be nonempty test names, not runner flags" >&2
        exit 2
      fi
    done
    cargo fmt --all -- --check
    cargo check --all-targets
    if (( lint )); then scripts/code-health.sh --clippy-only; fi
    cargo test --bin awl -- println_audit \
      card::figures::tests::only_the_substitution_door_replaces_a_view_states_text view_policy
    fast_log="$(mktemp "${TMPDIR:-/tmp}/awl-verify-fast.XXXXXX")"
    trap 'rm -f "$fast_log"' EXIT
    # Keep the command's failure visible, and reject an unmatched selector.
    if cargo test --bin awl -- "$@" >"$fast_log" 2>&1; then
      cat "$fast_log"
    else
      status=$?
      cat "$fast_log"
      exit "$status"
    fi
    if ! grep -Eq 'test result: ok\. [1-9][0-9]* passed;' "$fast_log"; then
      echo "verify fast: no selected test passed; check filters and ignored tests" >&2
      exit 1
    fi
    echo "verify fast: targeted checks passed; NOT a full native/web receipt"
    ;;
  full)
    if (( $# != 0 )); then usage >&2; exit 2; fi
    # Fail before the expensive gate if wasm execution would be skipped.
    command -v wasm-bindgen-test-runner >/dev/null || {
      echo "verify full: wasm-bindgen-test-runner is required; install the pinned wasm-bindgen-cli" >&2
      exit 1
    }
    full_commit="$(git rev-parse HEAD)"
    scripts/native-gate.sh
    scripts/web-smoke.sh
    scripts/release-profile-gate.sh
    if [[ "$(git rev-parse HEAD)" != "$full_commit" || -n "$(git status --short)" ]]; then
      echo "verify full: candidate changed; no combined verification result issued" >&2
      exit 1
    fi
    echo "verify full: passed commit=$full_commit native+wasm-runtime+debug-release-parity"
    ;;
  extended)
    python3 scripts/pretag-journeys.py "$@"
    echo "verify extended: selected journey sweep passed; human/platform journeys remain separate"
    ;;
  *) usage >&2; exit 2 ;;
esac
