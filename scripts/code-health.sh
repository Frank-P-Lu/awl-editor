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
# Clippy drives rustc through clippy-driver rather than producing reusable
# compiler artifacts. Keep it outside the shared rustc cache: sccache's
# jobserver handshake with clippy-driver is not portable across local runners.
RUSTC_WRAPPER= cargo clippy --all-targets --all-features -- -D warnings
# BLIND-SPOT ARM (macOS hosts only): the pass above lints whatever the ambient
# host target actually compiles. On a macOS dev machine that's an
# aarch64/x86_64-apple-darwin triple, so any `#[cfg(not(target_os = "macos"))]`
# source (the non-mac trash-can arm, GPU backend split, dock-icon stub, …) is
# never compiled here, and a lint living only in that arm is invisible no
# matter how thorough `--all-targets --all-features` looks (item 178: proven
# by mutation — a `needless_return` planted in one such arm passed the plain
# pass above at exit 0 and was only caught cross-compiled below). CI's `linux`
# job is the real backstop (it compiles that code for real); this arm exists to
# catch the same class BEFORE a push, on machines that already carry the
# `x86_64-unknown-linux-gnu` target — clippy only checks (no link step), so no
# cross-linker is required. It never installs a target itself: absent one, it
# names the gap out loud so a clean run is never mistaken for full parity with
# CI's blind spot.
if [[ "$(uname -s)" == "Darwin" ]]; then
  if command -v rustup >/dev/null 2>&1 \
    && rustup target list --installed 2>/dev/null | grep -qx x86_64-unknown-linux-gnu; then
    RUSTC_WRAPPER= cargo clippy --target x86_64-unknown-linux-gnu --all-targets --all-features -- -D warnings
  else
    echo "code-health: SKIPPED the linux-target clippy arm (x86_64-unknown-linux-gnu not installed)." >&2
    echo "  cfg(not(target_os = \"macos\")) code is NOT linted by this run; CI's linux job is the only" >&2
    echo "  thing that will catch a lint living there. Install with 'rustup target add" >&2
    echo "  x86_64-unknown-linux-gnu' to close the gap locally." >&2
  fi
fi
RUSTC_WRAPPER= python3 scripts/code-health.py
# The scan covers every tracked Rust source file, including native/macOS/wasm/
# feature-gated paths. Never let a target directory's generated output make a
# dependency look live. awl has no renamed dependency packages, so the
# metadata mode adds no correctness here and would let cargo-machete rewrite
# Cargo.lock as an implementation detail of its probe.
cargo machete --skip-target-dir
