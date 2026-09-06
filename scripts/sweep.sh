#!/usr/bin/env bash
# Reclaim stale Cargo build artifacts in THIS worktree.
#
# Cargo hashes each build's output and never garbage-collects the old copies, so
# a target/ dir grows without bound under repeated rebuilds — one reached 68 GB
# holding 180k files for 344 crates. `cargo sweep --time N` removes artifacts
# untouched for N days and keeps the current ones, so an active worktree keeps
# what it is using and only dead output goes.
#
# THE SCOPE IS THE CALLER'S OWN WORKTREE, AND THAT IS A CORRECTNESS RULE RATHER
# THAN A CONVENIENCE. `cargo sweep` decides what is dead from file mtimes and
# takes no lock, so it happily deletes fingerprints an incremental build is
# about to reuse. Pointed at a SIBLING worktree it kills that lane's live
# compile — the victim dies on `failed to write …/.fingerprint/<crate>/
# invoked.timestamp`, which reads as its own broken build rather than as
# another process's deletion. The disk preflight fires this script on every
# concurrent worker command, so a fleet-wide traversal was reachable from any
# lane at any moment the disk sat under its floor. One root per invocation also
# keeps the receipt attributable.
#
# `--all-worktrees` restores the fleet traversal for hand-driven maintenance,
# where the operator knows nothing is building. No automatic caller passes it,
# and scripts/test-sweep.sh holds both halves of that line.
#
# `cargo sweep` is deliberately invoked WITHOUT `--recursive`, so it touches
# only `<root>/target` and never descends into the worktrees kept under
# `.claude/worktrees/`. That is a property of the tool, not of this script, so
# scripts/test-sweep.sh pins it against the real binary.
#
#   scripts/sweep.sh                    # this worktree: artifacts unused for 7+ days
#   scripts/sweep.sh 3                  # ...for 3+ days
#   scripts/sweep.sh --all-worktrees 1  # every registered worktree (manual only)
#   DRY_RUN=1 scripts/sweep.sh          # report only
set -euo pipefail

ALL_WORKTREES=0
if [[ "${1:-}" == "--all-worktrees" ]]; then
    ALL_WORKTREES=1
    shift
fi

DAYS="${1:-7}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
export PATH="$HOME/.cargo/bin:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

if ! command -v cargo-sweep >/dev/null 2>&1; then
    echo "sweep: cargo-sweep not installed — cargo install cargo-sweep" >&2
    exit 1
fi

# macOS still ships Bash 3, so do not use an associative array here.
unique_roots=()
if [[ "$ALL_WORKTREES" -eq 1 ]]; then
    echo "sweep: --all-worktrees — pruning EVERY registered worktree. A build live in any" >&2
    echo "  of them can die on a fingerprint deleted underneath it; run this only when the" >&2
    echo "  fleet is idle." >&2

    # Worktrees are discovered from Git rather than guessed from where this
    # checkout happens to keep them. That includes registered worktrees outside
    # this repo and dotted locations such as .claude/worktrees/.
    roots=()
    while IFS= read -r line; do
        case "$line" in
            "worktree "*)
                root="${line#worktree }"
                if [[ -d "$root" ]]; then
                    root="$(cd "$root" && pwd -P)"
                fi
                roots+=("$root")
                ;;
        esac
    done < <(git -C "$ROOT" worktree list --porcelain)

    for root in "${roots[@]}"; do
        known=0
        for registered in "${unique_roots[@]:-}"; do
            if [[ "$registered" == "$root" ]]; then
                known=1
                break
            fi
        done
        if [[ "$known" -eq 0 ]]; then
            unique_roots+=("$root")
        fi
    done
    echo "sweep: scope=all-worktrees roots=${#unique_roots[@]} days=$DAYS"
else
    unique_roots=("$ROOT")
    echo "sweep: scope=self root=$ROOT days=$DAYS"
fi

for root in "${unique_roots[@]}"; do
    if [[ ! -d "$root" ]]; then
        echo "sweep: $root (registered worktree unavailable)" >&2
        continue
    fi

    before_kib="$(du -sk "$root" 2>/dev/null | awk '{print $1}')"
    if [[ -n "${DRY_RUN:-}" ]]; then
        cargo sweep --dry-run --hidden --time "$DAYS" "$root"
    else
        cargo sweep --hidden --time "$DAYS" "$root"
    fi
    after_kib="$(du -sk "$root" 2>/dev/null | awk '{print $1}')"
    reclaimed_kib=$((before_kib - after_kib))
    echo "sweep: $root: ${before_kib}KiB -> ${after_kib}KiB (reclaimed ${reclaimed_kib}KiB; kept artifacts used within ${DAYS}d)"
done
