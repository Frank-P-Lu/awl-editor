#!/usr/bin/env bash
# External behavioral law for sweep.sh. It uses a disposable Git repository and
# a fake cargo-sweep, so no real Cargo artifact or user worktree is touched.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-sweep-test.XXXXXX")"
REPO="$WORK/repo"
OUTSIDE="$WORK/registered-outside-repo"
LOG="$WORK/cargo-sweep.log"

cleanup() {
    if [[ -d "$REPO/.git" && -d "$OUTSIDE" ]]; then
        git -C "$REPO" worktree remove --force "$OUTSIDE" >/dev/null 2>&1 || true
    fi
    rm -rf "$WORK"
}
trap cleanup EXIT

fail() {
    echo "test-sweep: $*" >&2
    exit 1
}

assert_file() {
    [[ -e "$1" ]] || fail "expected $1 to exist"
}

assert_missing() {
    [[ ! -e "$1" ]] || fail "expected $1 to be removed"
}

assert_once() {
    local needle="$1"
    local count
    count="$(grep -Fxc -- "$needle" "$LOG" || true)"
    [[ "$count" == 1 ]] || fail "expected one sweep of $needle, found $count"
}

mkdir -p "$REPO/scripts" "$WORK/bin" "$WORK/home"
REPO="$(cd "$REPO" && pwd -P)"
cp "$ROOT/scripts/sweep.sh" "$REPO/scripts/sweep.sh"
chmod +x "$REPO/scripts/sweep.sh"
git -C "$REPO" init -q
git -C "$REPO" config user.email test@example.invalid
git -C "$REPO" config user.name "Sweep test"
touch "$REPO/README"
git -C "$REPO" add README scripts/sweep.sh
git -C "$REPO" commit -qm fixture
git -C "$REPO" worktree add -qb registered-outside "$OUTSIDE"
OUTSIDE="$(cd "$OUTSIDE" && pwd -P)"

cat >"$WORK/bin/cargo-sweep" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
root="${!#}"
printf '%s\n' "$root" >>"$SWEEP_TEST_LOG"
if [[ " $* " != *" --dry-run "* ]]; then
    find "$root/target" -type f -name stale -delete 2>/dev/null || true
fi
EOF
chmod +x "$WORK/bin/cargo-sweep"

for root in "$REPO" "$OUTSIDE"; do
    mkdir -p "$root/target"
    : >"$root/target/stale"
    : >"$root/target/fresh"
    touch -t 202001010000 "$root/target/stale"
done

# The structural half prevents a future rewrite from silently returning to a
# directory guess. The behavioral half below proves the external registration.
grep -Fq 'git -C "$ROOT" worktree list --porcelain' "$ROOT/scripts/sweep.sh" \
    || fail "registered-worktree discovery is missing"

SWEEP_TEST_LOG="$LOG" HOME="$WORK/home" PATH="$WORK/bin:$PATH" \
    "$REPO/scripts/sweep.sh" 7 >"$WORK/output"

assert_missing "$REPO/target/stale"
assert_missing "$OUTSIDE/target/stale"
assert_file "$REPO/target/fresh"
assert_file "$OUTSIDE/target/fresh"
assert_once "$REPO"
assert_once "$OUTSIDE"
grep -Fq "sweep: $REPO:" "$WORK/output" || fail "main worktree receipt missing"
grep -Fq "sweep: $OUTSIDE:" "$WORK/output" || fail "external worktree receipt missing"

for root in "$REPO" "$OUTSIDE"; do
    : >"$root/target/stale"
    touch -t 202001010000 "$root/target/stale"
done
: >"$LOG"
DRY_RUN=1 SWEEP_TEST_LOG="$LOG" HOME="$WORK/home" PATH="$WORK/bin:$PATH" \
    "$REPO/scripts/sweep.sh" 3 >"$WORK/dry-output"
assert_file "$REPO/target/stale"
assert_file "$OUTSIDE/target/stale"
assert_once "$REPO"
assert_once "$OUTSIDE"
grep -Fq 'kept artifacts used within 3d' "$WORK/dry-output" \
    || fail "requested retention period was not reported"

# Non-vacuity: mutate discovery back to the former root-only behavior. The
# external stale artifact must survive and the receipt must omit that root.
cp "$REPO/scripts/sweep.sh" "$REPO/scripts/sweep-root-only.sh"
perl -0pi -e 's#done < <\(git -C "\$ROOT" worktree list --porcelain\)#done < <(printf '\''worktree %s\\n'\'' "\$ROOT")#' \
    "$REPO/scripts/sweep-root-only.sh"
: >"$LOG"
SWEEP_TEST_LOG="$LOG" HOME="$WORK/home" PATH="$WORK/bin:$PATH" \
    "$REPO/scripts/sweep-root-only.sh" 7 >"$WORK/mutated-output"
assert_file "$OUTSIDE/target/stale"
if grep -Fq "sweep: $OUTSIDE:" "$WORK/mutated-output"; then
    fail "root-only mutation still reported the external worktree"
fi

echo "test-sweep: external registered worktree reclaimed; root-only mutation left it stale"
