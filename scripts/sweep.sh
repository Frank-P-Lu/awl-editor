#!/usr/bin/env bash
# Reclaim stale Cargo build artifacts across every registered worktree.
#
# Cargo hashes each build's output and never garbage-collects the old copies, so
# a target/ dir grows without bound under repeated rebuilds — one reached 68 GB
# holding 180k files for 344 crates. `cargo sweep --time N` removes artifacts
# untouched for N days and keeps the current ones, so an active worktree is
# unaffected and only dead output goes.
#
# Worktrees are discovered from Git rather than guessed from where this checkout
# happens to keep them. That includes registered worktrees outside this repo and
# dotted locations such as .claude/worktrees/.
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

# macOS still ships Bash 3, so do not use an associative array here. Cargo is
# invoked once per worktree, which both makes the receipt attributable and
# avoids one root recursively sweeping another worktree nested below it.
unique_roots=()
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
