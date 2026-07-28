#!/usr/bin/env bash
# Reclaim stale Cargo build artifacts across the repo AND every worktree.
#
# Cargo hashes each build's output and never garbage-collects the old copies, so
# a target/ dir grows without bound under repeated rebuilds — one reached 68 GB
# holding 180k files for 344 crates. `cargo sweep --time N` removes artifacts
# untouched for N days and keeps the current ones, so an active worktree is
# unaffected and only dead output goes.
#
# --hidden is REQUIRED, not optional: worktrees live under .claude/worktrees/
# and .agents/worktrees/, and cargo-sweep skips dotted directories by default.
# Without it this script finds nothing and still exits 0.
#
#   scripts/sweep.sh            # remove artifacts unused for 7+ days
#   scripts/sweep.sh 3          # ...for 3+ days
#   DRY_RUN=1 scripts/sweep.sh  # report only
set -euo pipefail

DAYS="${1:-7}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

if ! command -v cargo-sweep >/dev/null 2>&1; then
    echo "sweep: cargo-sweep not installed — cargo install cargo-sweep" >&2
    exit 1
fi

before=$(du -sh "$ROOT" 2>/dev/null | cut -f1)
cargo sweep ${DRY_RUN:+--dry-run} --hidden --recursive --time "$DAYS" "$ROOT"
echo "sweep: $before -> $(du -sh "$ROOT" 2>/dev/null | cut -f1) (kept artifacts used within ${DAYS}d)"
