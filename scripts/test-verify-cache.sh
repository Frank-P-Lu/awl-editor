#!/usr/bin/env bash
# Law suite for scripts/verify_cache.py's reuse mechanism, entirely against a
# throwaway fixture repo and fake fast commands — never the real cargo tests,
# so this runs in well under a second and can sweep every invalidation axis
# docs/verification.md names (source, tests, config, toolchain, hardware,
# environment branches) plus missing/corrupt records, cancellation, failure,
# concurrency, and unrecognised inputs.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VC="$ROOT/scripts/verify_cache.py"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-verify-cache-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

REPO="$WORK/repo"
CACHE="$WORK/cache"
mkdir -p "$REPO/src" "$REPO/tests"
git -C "$REPO" init -q
git -C "$REPO" config user.email test@example.com
git -C "$REPO" config user.name "Test"
echo "fn main() {}" >"$REPO/src/foo.rs"
echo "// fixture" >"$REPO/tests/fake_test.rs"
echo "[package]" >"$REPO/Cargo.toml"
git -C "$REPO" add -A
git -C "$REPO" commit -q -m "fixture"

COUNTER="$WORK/run-count"
: >"$COUNTER"
SHOULD_FAIL="$WORK/should-fail"
CHECK_CMD="$WORK/checkcmd.sh"
cat >"$CHECK_CMD" <<EOF
#!/usr/bin/env bash
echo x >>"$COUNTER"
if [[ -e "$SHOULD_FAIL" ]]; then
  exit 1
fi
exit 0
EOF
chmod +x "$CHECK_CMD"

MANIFEST="$WORK/manifest.toml"
cat >"$MANIFEST" <<EOF
[[check]]
name = "fake"
command = ["$CHECK_CMD"]
source_globs = [":(glob)src/**/*.rs"]
tests = ["tests/fake_test.rs"]
config = ["Cargo.toml"]
env_branches = ["AWL_VERIFY_CACHE_TEST_BRANCH"]
EOF

vc() {
  python3 "$VC" --manifest "$MANIFEST" --root "$REPO" --cache-dir "$CACHE" "$@"
}

count() { wc -l <"$COUNTER" | tr -d ' '; }

expect_count() {
  local want="$1" got
  got="$(count)"
  if [[ "$got" != "$want" ]]; then
    echo "test-verify-cache: expected $want real run(s) so far, got $got" >&2
    exit 1
  fi
}

# ── 1. Cold run executes; unchanged inputs reuse ────────────────────────────
vc run fake >/dev/null
expect_count 1
if ! vc run fake | grep -q "REUSED"; then
  echo "test-verify-cache: unchanged inputs should reuse the cached PASS" >&2
  exit 1
fi
expect_count 1
echo "test-verify-cache: unchanged inputs reuse; a changed check never reruns without cause"

# ── 2. A source change invalidates ──────────────────────────────────────────
echo "// changed" >>"$REPO/src/foo.rs"
vc run fake >/dev/null
expect_count 2
echo "test-verify-cache: a source_globs file change forces a rerun"

# ── 3. A test-file change invalidates ───────────────────────────────────────
vc run fake >/dev/null; expect_count 2  # settle onto a fresh cached PASS first
echo "// changed" >>"$REPO/tests/fake_test.rs"
vc run fake >/dev/null
expect_count 3
echo "test-verify-cache: a tests file change forces a rerun"

# ── 4. A config change invalidates ──────────────────────────────────────────
vc run fake >/dev/null; expect_count 3
echo "edition = \"2021\"" >>"$REPO/Cargo.toml"
vc run fake >/dev/null
expect_count 4
echo "test-verify-cache: a config file change forces a rerun"

# ── 5. A toolchain change invalidates ────────────────────────────────────────
vc run fake >/dev/null; expect_count 4
export AWL_VERIFY_CACHE_TOOLCHAIN_OVERRIDE=toolchain-b
vc run fake >/dev/null
expect_count 5
echo "test-verify-cache: a toolchain change forces a rerun"
# ...and reusing again under that SAME toolchain works, proving the toolchain
# is part of the signature rather than merely disabling caching outright.
if ! vc run fake | grep -q REUSED; then
  echo "test-verify-cache: same overridden toolchain should still reuse" >&2
  exit 1
fi
expect_count 5

# ── 6. A hardware-class change invalidates ──────────────────────────────────
export AWL_VERIFY_CACHE_HARDWARE_OVERRIDE="fake-arch"
vc run fake >/dev/null
expect_count 6
echo "test-verify-cache: a hardware-class change forces a rerun"

# ── 7. A declared environment-branch value change invalidates ──────────────
vc run fake >/dev/null; expect_count 6
export AWL_VERIFY_CACHE_TEST_BRANCH=on
vc run fake >/dev/null
expect_count 7
echo "test-verify-cache: a declared environment-branch value change forces a rerun"
# The same branch value reuses; an UNDECLARED variable changing must not.
vc run fake >/dev/null; expect_count 7
SOME_UNDECLARED_VAR=whatever vc run fake >/dev/null
expect_count 7
echo "test-verify-cache: an undeclared environment variable has no power to invalidate"

# ── 8. A missing record forces a rerun ──────────────────────────────────────
vc run fake >/dev/null; expect_count 7
rm -f "$CACHE/fake.json"
vc run fake >/dev/null
expect_count 8
echo "test-verify-cache: a missing record forces a rerun"

# ── 9. A corrupt record forces a rerun and is replaced with a valid one ────
echo "{ not valid json" >"$CACHE/fake.json"
vc run fake >/dev/null
expect_count 9
if ! python3 -c "import json,sys; json.load(open('$CACHE/fake.json'))"; then
  echo "test-verify-cache: the record must be replaced with valid JSON after a corrupt one" >&2
  exit 1
fi
vc run fake >/dev/null  # now reuses the freshly-repaired record
expect_count 9
echo "test-verify-cache: a corrupt record forces a rerun and self-heals to a valid one"

# ── 10. Failure: a failing check is recorded fail, never reused as green ───
# A fresh signature first (the SHOULD_FAIL marker itself is invisible to the
# cache — it is not a declared input — so failing on an UNCHANGED signature
# would just correctly reuse the prior PASS; that is not this law's subject).
echo "// force a fresh signature for the failure law" >>"$REPO/src/foo.rs"
touch "$SHOULD_FAIL"
if vc run fake >"$WORK/fail-output" 2>&1; then
  echo "test-verify-cache: a failing check must propagate a nonzero exit" >&2
  cat "$WORK/fail-output" >&2
  exit 1
fi
expect_count 10
# Same (still-failing) inputs: must rerun, NEVER reuse a cached failure as if
# it were a pass — "a failed run cannot become green through bookkeeping."
if vc run fake 2>&1 | grep -q REUSED; then
  echo "test-verify-cache: a cached FAILURE must never be reused" >&2
  exit 1
fi
expect_count 11
rm -f "$SHOULD_FAIL"
# Now genuinely passing again: reruns for real (the fix must be exercised,
# not bookkept in), then the resulting fresh PASS reuses normally.
vc run fake >/dev/null
expect_count 12
if ! vc run fake | grep -q REUSED; then
  echo "test-verify-cache: a genuine fresh PASS should reuse on the next call" >&2
  exit 1
fi
expect_count 12
echo "test-verify-cache: a failure is never reused as green; only a real, later PASS reuses"

# ── 11. Unrecognised check: hard error, no cache file, nothing runs ────────
before="$(count)"
if vc run does-not-exist >"$WORK/unknown-output" 2>&1; then
  echo "test-verify-cache: an unregistered check name must be refused" >&2
  exit 1
fi
if [[ -e "$CACHE/does-not-exist.json" ]]; then
  echo "test-verify-cache: an unregistered check must never create a cache record" >&2
  exit 1
fi
expect_count "$before"
echo "test-verify-cache: an unregistered check name is refused and caches nothing"

# ── 12. Cancellation: a killed run leaves no fabricated record ─────────────
SLOW_CMD="$WORK/slowcmd.sh"
SLOW_MARK="$WORK/slow-ran"
cat >"$SLOW_CMD" <<EOF
#!/usr/bin/env bash
sleep 5
touch "$SLOW_MARK"
exit 0
EOF
chmod +x "$SLOW_CMD"
SLOW_MANIFEST="$WORK/slow-manifest.toml"
cat >"$SLOW_MANIFEST" <<EOF
[[check]]
name = "slow"
command = ["$SLOW_CMD"]
source_globs = [":(glob)src/**/*.rs"]
EOF
rm -f "$CACHE/slow.json"
# Job control (`set -m`) puts the background pipeline in its OWN process
# group, so killing the group (not just verify_cache.py's own pid) also
# reaches the `sleep`/`slowcmd.sh` children it spawned — killing only the
# parent would leave an orphaned `sleep` running for the rest of this
# script, later writing garbage into a file this script has since reused.
set -m
python3 "$VC" --manifest "$SLOW_MANIFEST" --root "$REPO" --cache-dir "$CACHE" run slow &
slow_pid=$!
sleep 0.5
kill -TERM -- "-$slow_pid" 2>/dev/null || kill -TERM "$slow_pid" 2>/dev/null || true
wait "$slow_pid" 2>/dev/null || true
set +m
# Belt and braces: reap any straggler by exact command line match, so a
# platform where the process-group kill above did not reach every child
# cannot leave a background `sleep` mutating shared fixture state later.
pkill -f "$SLOW_CMD" 2>/dev/null || true
if [[ -e "$SLOW_MARK" ]]; then
  echo "test-verify-cache: the slow command should never have completed" >&2
  exit 1
fi
if [[ -e "$CACHE/slow.json" ]]; then
  echo "test-verify-cache: a killed run must leave no record at all" >&2
  exit 1
fi
if compgen -G "$CACHE/slow.json.tmp*" >/dev/null; then
  echo "test-verify-cache: a killed run must leave no dangling temp file either" >&2
  exit 1
fi
echo "test-verify-cache: a cancelled run leaves no fabricated or partial record"

# Now run it for real (fast this time — patch the command) and confirm it
# actually executes rather than treating the cancelled attempt as done.
cat >"$SLOW_CMD" <<EOF
#!/usr/bin/env bash
touch "$SLOW_MARK"
exit 0
EOF
python3 "$VC" --manifest "$SLOW_MANIFEST" --root "$REPO" --cache-dir "$CACHE" run slow >/dev/null
if [[ ! -e "$SLOW_MARK" ]]; then
  echo "test-verify-cache: after a cancellation, the next invocation must run for real" >&2
  exit 1
fi
echo "test-verify-cache: after a cancellation, the next invocation runs for real"

# ── 13. Concurrency: two simultaneous callers only run the command once ────
rm -f "$CACHE/fake.json"
: >"$COUNTER"
vc run fake >"$WORK/conc-a" 2>&1 &
pid_a=$!
vc run fake >"$WORK/conc-b" 2>&1 &
pid_b=$!
status_a=0; status_b=0
wait "$pid_a" || status_a=$?
wait "$pid_b" || status_b=$?
if [[ "$status_a" -ne 0 || "$status_b" -ne 0 ]]; then
  echo "test-verify-cache: both concurrent callers should succeed" >&2
  cat "$WORK/conc-a" "$WORK/conc-b" >&2
  exit 1
fi
if [[ "$(count)" != "1" ]]; then
  echo "test-verify-cache: two concurrent callers on identical inputs must run the check exactly once, ran $(count)" >&2
  cat "$WORK/conc-a" "$WORK/conc-b" >&2
  exit 1
fi
echo "test-verify-cache: two concurrent callers on identical inputs run the check exactly once"

echo "test-verify-cache: all reuse-invalidation, failure, cancellation, concurrency, and unrecognised-input laws hold"
