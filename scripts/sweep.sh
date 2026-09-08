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
# `target/{debug,release}/incremental` HAS NO OTHER OWNER. `cargo sweep` never
# touches it at any threshold — measured directly across this fleet, a sweep
# that emptied `deps` and `.fingerprint` left it byte-for-byte unchanged — and
# it is 46-86% of every `target/` sampled here. rustc's own incremental cache
# is safe to delete at any time (a missing session just costs one non-
# incremental recompile of that crate, never a wrong build), so this script is
# the one owner of both pools and applies the SAME age rule to both: a session
# directory untouched for DAYS+ is exactly as dead as a fingerprint untouched
# for DAYS+, and is deleted by the same call. This reaches only
# `<root>/target/{debug,release}/incremental` — two fixed paths under the root
# `cargo sweep` was already given, never a traversal from `<root>` downward —
# so it inherits the same non-crossing guarantee as the cargo-sweep call above
# without depending on find(1) or any tool's own scoping. A LIVE build keeps
# touching its own session directories, so this reclaims nothing from a lane
# that used its artifacts within the window — same shape as cargo-sweep's own
# near-zero yield on an active lane, and re-measured rather than assumed
# (scripts/test-sweep.sh pins both the deletion and the survival cases).
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

# The owner cargo-sweep does not have. Fixed subpaths of the SAME root
# cargo-sweep was just given, never a traversal from that root downward, so a
# sibling worktree nested elsewhere under `.claude/worktrees/` is unreachable
# by construction rather than by find(1) obeying a boundary.
prune_stale_incremental() {
    local root="$1" days="$2" profile dir pruned_dirs=0 pruned_kib=0
    for profile in debug release; do
        dir="$root/target/$profile/incremental"
        [[ -d "$dir" ]] || continue
        while IFS= read -r -d '' stale; do
            local kib
            kib="$(du -sk "$stale" 2>/dev/null | awk '{print $1}')"
            kib="${kib:-0}"
            if [[ -n "${DRY_RUN:-}" ]]; then
                echo "sweep: would prune $stale (${kib}KiB, unused ${days}+d)"
            else
                rm -rf "$stale"
            fi
            pruned_dirs=$((pruned_dirs + 1))
            pruned_kib=$((pruned_kib + kib))
        done < <(find "$dir" -mindepth 1 -maxdepth 1 -type d -mtime "+$days" -print0 2>/dev/null)
    done
    echo "sweep: $root: incremental: ${pruned_dirs} stale session dir(s), ${pruned_kib}KiB (cargo sweep cannot see this pool at any threshold)"
}

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
    prune_stale_incremental "$root" "$DAYS"
    after_kib="$(du -sk "$root" 2>/dev/null | awk '{print $1}')"
    reclaimed_kib=$((before_kib - after_kib))
    echo "sweep: $root: ${before_kib}KiB -> ${after_kib}KiB (reclaimed ${reclaimed_kib}KiB; kept artifacts used within ${DAYS}d)"
done
