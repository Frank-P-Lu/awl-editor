#!/usr/bin/env bash
# Laws for scripts/sweep.sh: WHAT IT IS ALLOWED TO DELETE.
#
# The defect these exist for is not hypothetical. `sweep.sh` traversed every
# registered worktree, `.orchestrator/disk-preflight.sh` fires it on every
# concurrent worker command, and `cargo sweep` takes no lock — so a lane that
# merely started a build deleted fingerprints out from under a SIBLING lane's
# live compile, and the victim died on `failed to write …/.fingerprint/
# <crate>/invoked.timestamp` as though its own build were broken.
#
# Wired into scripts/code-health.sh at birth. An unwired law rots and then
# cries wolf; the previous version of this file was deleted for exactly that.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-sweep-test.XXXXXX")"
WORK="$(cd "$WORK" && pwd -P)"
trap 'rm -rf "$WORK"' EXIT

fail() {
    echo "test-sweep: FAIL — $*" >&2
    exit 1
}

physical() { (cd "$1" && pwd -P); }

# A fixture repo with a lane worktree nested where this repo keeps them, each
# holding a target/ with one sentinel artifact. `cargo` is stubbed so the law
# reads the TRAVERSAL rather than cargo-sweep's own age arithmetic: the stub
# logs the root it was pointed at and prunes that root's sentinel, which is
# exactly the authority sweep.sh hands it.
make_fixture() {
    local dir="$1"
    mkdir -p "$dir/main/scripts" "$dir/bin" "$dir/home"
    cp "$ROOT/scripts/sweep.sh" "$dir/main/scripts/sweep.sh"
    chmod +x "$dir/main/scripts/sweep.sh"
    git -C "$dir/main" init -q .
    git -C "$dir/main" add scripts/sweep.sh
    git -C "$dir/main" \
        -c user.email=test@example.invalid -c user.name=test \
        -c commit.gpgsign=false commit -q -m fixture
    git -C "$dir/main" worktree add -q "$dir/main/.claude/worktrees/lane-b" -b lane-b

    mkdir -p "$dir/main/target" "$dir/main/.claude/worktrees/lane-b/target"
    echo stale > "$dir/main/target/artifact.stale"
    echo stale > "$dir/main/.claude/worktrees/lane-b/target/artifact.stale"

    cat > "$dir/bin/cargo" <<'STUB'
#!/usr/bin/env bash
# Stand-in for `cargo sweep`: record the root and prune that root's sentinel.
[[ "${1:-}" == "sweep" ]] || exit 0
for arg in "$@"; do target_root="$arg"; done
printf '%s\n' "$target_root" >> "$SWEEP_LOG"
rm -f "$target_root/target/artifact.stale"
STUB
    # sweep.sh gates on `command -v cargo-sweep`, which must answer yes.
    printf '#!/usr/bin/env bash\nexit 0\n' > "$dir/bin/cargo-sweep"
    chmod +x "$dir/bin/cargo" "$dir/bin/cargo-sweep"
}

# sweep.sh prepends $HOME/.cargo/bin to PATH, so the stub only wins with HOME
# pointed somewhere without a real toolchain. Faking HOME also keeps the
# fixture's git off the developer's global config.
run_sweep() {
    local dir="$1"
    shift
    HOME="$dir/home" PATH="$dir/bin:$PATH" SWEEP_LOG="$dir/sweep.log" \
        "$dir/main/.claude/worktrees/lane-b/scripts/sweep.sh" "$@" \
        >"$dir/sweep.out" 2>"$dir/sweep.err"
}

# ---------------------------------------------------------------------------
# LAW 1: a sweep launched from worktree B leaves worktree A's target/ alone.
# ---------------------------------------------------------------------------
A="$WORK/law1"
make_fixture "$A"
: > "$A/sweep.log"
run_sweep "$A" 1

lane_root="$(physical "$A/main/.claude/worktrees/lane-b")"
main_root="$(physical "$A/main")"

# Presence floor first: a sweep that pruned NOTHING would satisfy the law about
# the sibling for free, and would report the same green as a correct one.
if [[ -e "$A/main/.claude/worktrees/lane-b/target/artifact.stale" ]]; then
    fail "the caller's own target/ was not swept, so this law proves nothing"
fi
if [[ ! -e "$A/main/target/artifact.stale" ]]; then
    fail "a sweep launched from $lane_root deleted inside $main_root"
fi

swept_count="$(wc -l < "$A/sweep.log" | tr -d ' ')"
if [[ "$swept_count" != "1" ]]; then
    fail "expected exactly 1 swept root, got $swept_count: $(tr '\n' ' ' < "$A/sweep.log")"
fi
if [[ "$(physical "$(head -n1 "$A/sweep.log")")" != "$lane_root" ]]; then
    fail "swept root was $(head -n1 "$A/sweep.log"), expected the caller's own $lane_root"
fi

# ---------------------------------------------------------------------------
# LAW 2: --all-worktrees still reaches every worktree.
#
# This is the other half of law 1's non-vacuity: it proves the stub and the
# fixture CAN delete across worktrees, so law 1's survivor is the scoping rule
# and not an inert harness.
# ---------------------------------------------------------------------------
B="$WORK/law2"
make_fixture "$B"
: > "$B/sweep.log"
run_sweep "$B" --all-worktrees 1

if [[ -e "$B/main/target/artifact.stale" \
    || -e "$B/main/.claude/worktrees/lane-b/target/artifact.stale" ]]; then
    fail "--all-worktrees left a worktree unswept; the opt-in path is broken"
fi
all_count="$(wc -l < "$B/sweep.log" | tr -d ' ')"
if [[ "$all_count" != "2" ]]; then
    fail "--all-worktrees swept $all_count roots, expected 2"
fi

# ---------------------------------------------------------------------------
# LAW 3: no automatic caller opts in.
#
# Law 1 is worth nothing if the preflight — which fires on every concurrent
# worker command — passes --all-worktrees. Enrolment is derived rather than
# assumed: the preflight must still be a sweep caller at all, or the scan below
# is vacuously clean.
# ---------------------------------------------------------------------------
if ! grep -q 'scripts/sweep\.sh' "$ROOT/.orchestrator/disk-preflight.sh"; then
    fail ".orchestrator/disk-preflight.sh no longer invokes sweep.sh — law 3 would be vacuous"
fi

offenders=""
while IFS= read -r tracked; do
    case "$tracked" in
        scripts/sweep.sh | scripts/test-sweep.sh) continue ;;
        *.sh | .github/workflows/*.yml) ;;
        *) continue ;;
    esac
    if grep -q -- '--all-worktrees' "$ROOT/$tracked" 2>/dev/null; then
        offenders="$offenders $tracked"
    fi
done < <(git -C "$ROOT" ls-files)
if [[ -n "$offenders" ]]; then
    fail "these scripts pass --all-worktrees, which is manual-only:$offenders"
fi

# ---------------------------------------------------------------------------
# LAW 4: cargo-sweep without --recursive does not descend into a nested
# worktree.
#
# sweep.sh's scoping is only as narrow as the tool it delegates to, and this
# repo keeps its lane worktrees UNDER the main checkout. `--hidden` (which
# sweep.sh passes) is what makes a recursive walk enter `.claude/`, so if
# cargo-sweep ever recursed by default the narrow fix would be void while every
# other law here stayed green. Pinned against the real binary, on any host that
# has one.
# ---------------------------------------------------------------------------
if ! command -v cargo-sweep >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
    echo "test-sweep: SKIPPED law 4 (cargo-sweep not installed on this host)." >&2
    echo "  cargo-sweep's non-recursive default is UNVERIFIED by this run; sweep.sh's" >&2
    echo "  scoping assumes it. CI deliberately does not install cargo-sweep, so the" >&2
    echo "  developer gate is the only place this law runs." >&2
else
    C="$WORK/law4"
    mkdir -p "$C/main/src"
    printf '[package]\nname = "sweeplaw"\nversion = "0.0.0"\nedition = "2021"\n' \
        > "$C/main/Cargo.toml"
    echo 'fn main() {}' > "$C/main/src/main.rs"
    git -C "$C/main" init -q .
    git -C "$C/main" add Cargo.toml src/main.rs
    git -C "$C/main" \
        -c user.email=test@example.invalid -c user.name=test \
        -c commit.gpgsign=false commit -q -m fixture
    git -C "$C/main" worktree add -q "$C/main/.claude/worktrees/lane-b" -b lane-b

    (cd "$C/main" && RUSTC_WRAPPER= CARGO_INCREMENTAL=0 env -u CARGO_TARGET_DIR cargo build -q)
    (cd "$C/main/.claude/worktrees/lane-b" \
        && RUSTC_WRAPPER= CARGO_INCREMENTAL=0 env -u CARGO_TARGET_DIR cargo build -q)
    find "$C/main/target" "$C/main/.claude/worktrees/lane-b/target" \
        -exec touch -t 202001010000 {} +

    nested_before="$(find "$C/main/.claude/worktrees/lane-b/target" -type f | wc -l | tr -d ' ')"
    own_before="$(find "$C/main/target" -type f | wc -l | tr -d ' ')"
    cargo sweep --hidden --time 1 "$C/main" >/dev/null
    nested_after="$(find "$C/main/.claude/worktrees/lane-b/target" -type f | wc -l | tr -d ' ')"
    own_after="$(find "$C/main/target" -type f | wc -l | tr -d ' ')"

    if [[ "$own_after" -ge "$own_before" ]]; then
        fail "cargo sweep pruned nothing in its own root ($own_before -> $own_after); law 4 proves nothing"
    fi
    if [[ "$nested_after" != "$nested_before" ]]; then
        fail "cargo sweep --hidden (no --recursive) reached the nested worktree: $nested_before -> $nested_after files"
    fi
    echo "test-sweep: law 4 ran against $(cargo-sweep sweep --version 2>/dev/null || echo cargo-sweep)"
fi

echo "test-sweep: sweep.sh deletes only inside its caller's worktree"
