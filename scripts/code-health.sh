#!/usr/bin/env bash
# One owner for every blocking Rust code-health policy. Keep CI deliberately
# boring: it invokes this command and does not restate any of these checks.
set -euo pipefail

CARGO_MACHETE_VERSION="0.9.1"

if [[ "${1:-}" == "--cargo-machete-version" ]]; then
  printf '%s\n' "$CARGO_MACHETE_VERSION"
  exit 0
fi

if ! cargo machete --version 2>/dev/null | grep -Fxq "$CARGO_MACHETE_VERSION"; then
  echo "error: cargo-machete $CARGO_MACHETE_VERSION is required; install with:" >&2
  echo "  cargo install cargo-machete --version $CARGO_MACHETE_VERSION --locked" >&2
  exit 1
fi

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
python3 scripts/code-health.py
# The scan covers every tracked Rust source file, including native/macOS/wasm/
# feature-gated paths. Never let a target directory's generated output make a
# dependency look live. awl has no renamed dependency packages, so the
# metadata mode adds no correctness here and would let cargo-machete rewrite
# Cargo.lock as an implementation detail of its probe.
cargo machete --skip-target-dir
