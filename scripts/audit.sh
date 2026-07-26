#!/usr/bin/env bash
#
# audit.sh — awl's one SUPPLY-CHAIN dependency-policy wrapper. It runs the
# checked-in cargo-deny policy against Cargo.lock: RustSec advisories/yanks,
# licenses, duplicate versions, and registries/git sources all have one owner.
#
# This is a convenience wrapper, not a framework: it pins the toolchain PATH,
# resolves the repo root from its own location, and runs `cargo deny check`.
# The advisory and registry checks fetch their build-time databases over the
# network and are otherwise read-only (they never mutate Cargo.lock).
#
# Usage:
#   scripts/audit.sh          # check Cargo.lock; non-zero exit = findings
#
# The intended routine (see docs/licensing.md): run each merge-train day, and
# for each finding either apply the MINIMAL semver-compatible bump (`cargo
# update -p <crate>`, never a major/risky bump for a chore) or add one narrow,
# reasoned exception to deny.toml. cargo-deny rejects stale advisory ignores.
#
# One-time install: cargo install cargo-deny --version 0.20.2 --locked
#
set -euo pipefail

# Pin this Mac's toolchain so cargo/cargo-audit are findable regardless of cwd
# or a bare shell. Prefer a cargo already on PATH; only fall back otherwise.
if ! command -v cargo >/dev/null 2>&1; then
  export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found on PATH. Install Rust (https://rustup.rs) or add cargo to PATH." >&2
  exit 1
fi
DENY_VERSION="0.20.2"
if [[ "${1:-}" == "--cargo-deny-version" ]]; then
  printf '%s\n' "$DENY_VERSION"
  exit 0
fi

if ! cargo deny --version 2>/dev/null | grep -Fq "$DENY_VERSION"; then
  echo "error: cargo-deny $DENY_VERSION is required. Run:" >&2
  echo "  cargo install cargo-deny --version $DENY_VERSION --locked" >&2
  exit 1
fi

# Resolve repo root from this script's location so it works from any cwd.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

echo "==> cargo deny $DENY_VERSION (Cargo.lock policy: advisories, licenses, bans, sources)"
# Promote cargo-deny's otherwise-warning-level stale configuration diagnostics:
# an exception that no longer matches must make the shared gate fail.
exec cargo deny --config deny.toml check \
  --deny advisory-not-detected \
  --deny license-not-encountered \
  --deny unmatched-skip
