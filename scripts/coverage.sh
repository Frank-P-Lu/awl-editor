#!/usr/bin/env bash
# Periodic missing-law detector.  This is intentionally not a CI percentage
# gate: it makes observable seams easy to inspect before risky work and release
# preparation, then asks a person to decide whether a gap is a real contract.
set -euo pipefail

TOOL_VERSION="0.8.3"
TOOLCHAIN="nightly-2026-07-24"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="${AWL_COVERAGE_OUT:-$ROOT/coverage}"

if [[ "${1:-}" == "--cargo-llvm-cov-version" ]]; then
  printf '%s\n' "$TOOL_VERSION"
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found on PATH" >&2
  exit 1
fi
TOOLCHAIN_BIN="$(dirname "$(rustup which --toolchain "$TOOLCHAIN" cargo 2>/dev/null || true)")"
if [[ ! -x "$TOOLCHAIN_BIN/cargo" || ! -x "$TOOLCHAIN_BIN/rustc" ]]; then
  echo "error: $TOOLCHAIN is required; run: rustup toolchain install $TOOLCHAIN --profile minimal --component llvm-tools-preview" >&2
  exit 1
fi
export PATH="$TOOLCHAIN_BIN:$PATH"
if ! cargo llvm-cov --version 2>/dev/null | grep -Fxq "cargo-llvm-cov $TOOL_VERSION"; then
  cat >&2 <<EOF
error: cargo-llvm-cov $TOOL_VERSION is required. Run:
  cargo install cargo-llvm-cov --version $TOOL_VERSION --locked
  rustup toolchain install $TOOLCHAIN --profile minimal --component llvm-tools-preview
EOF
  exit 1
fi
if ! rustup component list --toolchain "$TOOLCHAIN" --installed 2>/dev/null | grep -q '^llvm-tools-'; then
  echo "error: llvm-tools-preview is required; run: rustup toolchain install $TOOLCHAIN --profile minimal --component llvm-tools-preview" >&2
  exit 1
fi

mkdir -p "$OUT"
rm -rf "$OUT/html"

# These are deliberately narrow. `mac_chrome.rs` is direct AppKit/window-server
# glue, and the native entry point cannot execute under the test harness. Their
# pure decisions belong in their callers' laws; this detector must not imply
# that a line in an uncallable OS boundary is a missing behavior law.
EXCLUDE='(^|/)(mac_chrome|main)\.rs$'

cd "$ROOT"
{
  printf 'commit=%s\n' "$(git rev-parse HEAD)"
  printf 'branch=%s\n' "$(git branch --show-current)"
  printf 'cargo_llvm_cov=%s\n' "$(cargo llvm-cov --version)"
  printf 'rustc=%s\n' "$(rustup run "$TOOLCHAIN" rustc -Vv | tr '\n' ' ')"
  printf 'command=PATH=%s:$PATH cargo llvm-cov --all-targets --all-features --branch --ignore-filename-regex %q\n' "$TOOLCHAIN_BIN" "$EXCLUDE"
  printf 'exclusions=src/mac_chrome.rs: direct AppKit/window-server glue; src/main.rs: native process entry point\n'
} >"$OUT/provenance.txt"

# Run once, then export both views from the same profiles. `--branch` is
# requested deliberately: the triage must show an uncalled decision, not only
# an uncalled statement. If the pinned toolchain cannot provide it, fail rather
# than silently reporting line-only coverage as branch evidence.
cargo llvm-cov --all-targets --all-features --branch --no-report
cargo llvm-cov report --json --branch --output-path "$OUT/coverage.json" \
  --ignore-filename-regex "$EXCLUDE"
cargo llvm-cov report --html --branch --output-dir "$OUT/html" \
  --ignore-filename-regex "$EXCLUDE"
python3 "$SCRIPT_DIR/coverage-triage.py" --self-test
python3 "$SCRIPT_DIR/coverage-triage.py" "$ROOT" "$OUT/coverage.json" "$OUT/triage.md"

printf 'coverage reports: %s\n' "$OUT"
