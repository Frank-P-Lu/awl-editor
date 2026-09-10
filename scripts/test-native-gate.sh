#!/usr/bin/env bash
# Exercise the gate's orchestration and failure semantics without compiling.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PYTHONDONTWRITEBYTECODE=1 python3 "$ROOT/scripts/test-native-test-shards.py"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-native-gate-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
# Every fixture gate inherits its own arbiter state. Besides keeping this test
# hermetic, that lets the concurrent-holder law choose a fresh marker without
# ever touching a real orchestrator's queue.
export AWL_NATIVE_GATE_MARKER="$WORK/default-marker"
export AWL_NATIVE_GATE_ARBITER_LOCK="$WORK/default-arbiter.lock"
# THE MENU-BAR AXIS MODE IS PINNED FOR EVERY LAW THAT IS NOT ABOUT IT. The gate
# chooses between a full-suite forced arm and the cheap name-filtered pair from
# the environment (`CI`), so leaving it ambient would make phase counts, shard
# counts and event tallies below differ between a developer's run of this file
# and CI's — a check whose configuration is itself untested. The two laws that
# ARE about the axis override this explicitly, including one that unsets it to
# prove the derivation.
export AWL_NATIVE_GATE_MENUBAR_FULL=0

# THE HEALTH ARM IS PINNED FOR EVERY LAW THAT IS NOT ABOUT IT, same reason as
# the menu-bar pin above: the real scripts/code-health.sh runs full clippy and
# recurses into this very file, so every probe below that did not override
# this would compile clippy from scratch and then re-run the whole suite you
# are reading right now, once per probe. The three laws that ARE about the
# health arm override this explicitly.
cat >"$WORK/health-stub" <<'HEALTHEOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${AWL_NATIVE_GATE_PROBE_HEALTH_SLEEP:-}" ]]; then
  sleep "$AWL_NATIVE_GATE_PROBE_HEALTH_SLEEP"
fi
# These two bodies are code-health.py's OWN message shapes, copied verbatim
# (check_structural's mark-shrink message and its ungrandfathered production-
# limit message) rather than invented text — a wording law over an invented
# signal would prove nothing about the real distinction it is pinning.
if [[ -n "${AWL_NATIVE_GATE_PROBE_HEALTH_RAISABLE:-}" ]]; then
  echo "code-health: policy check failed" >&2
  echo "src/large.rs: stored file-size mark is 500 lines but current file is 480; marks may only decrease" >&2
  echo "  paste into scripts/code-health.toml, replacing the existing block for this file (the size genuinely shrank; no reason needed for a lower mark):" >&2
  exit 1
fi
if [[ -n "${AWL_NATIVE_GATE_PROBE_HEALTH_CEILING:-}" ]]; then
  echo "code-health: policy check failed" >&2
  echo "src/new.rs: 501 lines (production limit is 500)" >&2
  exit 1
fi
echo "code-health: structural and Clippy ratchets clean (baseline deadbeef; 0 Clippy exceptions)"
HEALTHEOF
chmod +x "$WORK/health-stub"
export AWL_NATIVE_GATE_PROBE_HEALTH_COMMAND="$WORK/health-stub"

PYTHONDONTWRITEBYTECODE=1 python3 - "$ROOT/scripts/native-test-shards.py" \
  >"$WORK/awl-test-list" <<'PY'
import importlib.util
import sys

spec = importlib.util.spec_from_file_location("shards", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
for hints in module.HINTS:
    for prefix in hints:
        print(prefix + "probe: test")
print("firstrun::tests::probe: test")
print("other::tests::remainder: test")
PY

cat >"$WORK/awl-test-bin" <<'EOF'
#!/usr/bin/env bash
set -eo pipefail
tests=()
while IFS= read -r test; do tests+=("${test%: test}"); done \
  <"$AWL_NATIVE_GATE_PROBE_TEST_LIST"
filters=()
skips=()
listing=0
while (( $# )); do
  case "$1" in
    --list) listing=1 ;;
    --format) shift ;;
    --skip) shift; skips+=("$1") ;;
    --*) : ;;
    *) filters+=("$1") ;;
  esac
  shift
done
selected=()
for test in "${tests[@]}"; do
  match=0
  (( ${#filters[@]} == 0 )) && match=1
  for filter in "${filters[@]}"; do [[ "$test" == *"$filter"* ]] && match=1; done
  for skip in "${skips[@]}"; do [[ "$test" == *"$skip"* ]] && match=0; done
  (( match )) && selected+=("$test")
done
if (( listing )); then
  for test in "${selected[@]}"; do printf '%s: test\n' "$test"; done
  exit 0
fi
printf 'shard %s %s\n' "${AWL_CONVENTION_FORCE:-unset}" "$$" >>"$AWL_NATIVE_GATE_PROBE_LOG"
# The MENU-BAR forcing as the SHARD saw it. The full-suite arm runs the shard
# binary directly rather than through Cargo, so the `menubar` lines the fake
# `cargo` writes cannot see it at all — and a law reading only those would report
# an unswept axis as swept the moment the arm stopped going through Cargo.
printf 'shardbar %s\n' "${AWL_MENU_BAR_FORCE:-unset}" >>"$AWL_NATIVE_GATE_PROBE_LOG"
# THE TWO-WRITE SHAPE, ON PURPOSE AND WITHOUT A RACE. libtest writes a test's
# NAME and its RESULT as separate writes with the test running in between; six
# shards sharing one stdout can therefore complete each other's dangling names.
# Exactly one shard wins the `mkdir` election and leaves its name unterminated
# until another shard has written a whole line of its own, so the splice either
# happens or the gate's per-shard reader prevented it — never "it depends".
# Inert unless the probe asks for it, so every other probe's output is
# byte-identical to what it was.
if [[ -n "${AWL_NATIVE_GATE_PROBE_SPLIT_LINES:-}" ]]; then
  splice_dir="$(dirname "$AWL_NATIVE_GATE_PROBE_LOG")/splice"
  mkdir -p "$splice_dir"
  if mkdir "$splice_dir/leader" 2>/dev/null; then
    printf 'test probe::the_leaders_own_test ... '
    : >"$splice_dir/leader-armed"
    for _ in $(seq 1 200); do
      [[ -e "$splice_dir/other-done" ]] && break
      sleep 0.05
    done
    printf 'ok\n'
  else
    for _ in $(seq 1 200); do
      [[ -e "$splice_dir/leader-armed" ]] && break
      sleep 0.05
    done
    printf 'test probe::a_follower_shards_test ... ok\n'
    : >"$splice_dir/other-done"
  fi
fi
printf '\nrunning %s tests\n' "${#selected[@]}"
printf 'test result: ok. %s passed; 0 failed; 0 ignored; 0 measured\n' "${#selected[@]}"
# A shard that fails under a forcing — the full-suite arm's own red, which no
# fixture keyed on Cargo can produce.
if [[ -n "${AWL_MENU_BAR_FORCE:-}" \
  && "$AWL_MENU_BAR_FORCE" == "${AWL_NATIVE_GATE_FAIL_MENU_BAR:-}" ]]; then
  exit "${AWL_NATIVE_GATE_FAIL_STATUS:-1}"
fi
EOF
chmod +x "$WORK/awl-test-bin"

cat >"$WORK/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
convention="${AWL_CONVENTION_FORCE:-canary}"
if [[ " $* " == *" --no-run "* ]]; then
  printf '{"reason":"compiler-artifact","target":{"kind":["bin"],"name":"awl"},"profile":{"test":true},"executable":"%s"}\n' "$AWL_NATIVE_GATE_PROBE_TEST_BINARY"
  printf '{"reason":"compiler-artifact","target":{"kind":["test"],"name":"native_gate_canary"},"profile":{"test":true},"executable":"%s"}\n' "$AWL_NATIVE_GATE_PROBE_TEST_BINARY"
  exit 0
fi
printf 'start %s\n' "$convention" >>"$AWL_NATIVE_GATE_PROBE_LOG"
printf 'threads %s %s\n' "$convention" "${RUST_TEST_THREADS:-unset}" >>"$AWL_NATIVE_GATE_PROBE_LOG"
# The MENU-BAR axis, recorded per invocation. `unset` is a real answer and the
# one the canary and both conventions must give: if a convention inherited a
# forcing, the axis would be a property of the convention arms rather than its
# own two, and the pair would be measuring one branch twice.
printf 'menubar %s\n' "${AWL_MENU_BAR_FORCE:-unset}" >>"$AWL_NATIVE_GATE_PROBE_LOG"
# Cargo and libtest emit these lines with SGR colour on a GitHub runner (the
# 2026-08-02 mac log is full of them), so the fixture emits them coloured too:
# a phase matcher anchored to a bare "Finished" would pass here and see nothing
# in CI. The deliberately hostile test NAME carries the words a phase marker
# keys on — a per-test line must never be able to forge one.
if [[ -n "${AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT:-}" ]]; then
  printf '\033[1m\033[92m   Compiling\033[0m awl v0.1.0\n'
  printf '\033[1m\033[92m    Finished\033[0m `test` profile [optimized + debuginfo] target(s) in 1m 22s\n'
  printf '\033[1m\033[92m     Running\033[0m unittests src/main.rs (target/debug/deps/awl-a623f1caab4)\n'
  printf '\nrunning 3484 tests\n'
  printf 'test render::Running_tests::a_name_with_(parens) ... ok\n'
  printf 'test render::a_name_with_Finished_and_target(s) in_it ... ok\n'
  printf 'test result: ok. 3484 passed; 0 failed; 0 ignored; 0 measured\n'
  printf '\033[1m\033[92m     Running\033[0m tests/harness.rs (target/debug/deps/harness-0f0f0f0f)\n'
  printf 'running 2 tests\n'
  printf 'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured\n'
fi
# libtest prints "test NAME ... " BEFORE running the test and its result after,
# so a test that never returns leaves an UNTERMINATED line naming itself. That
# fragment is the single most valuable line in a hung run's log, and it is only
# ever flushed when the pipe closes.
if [[ -n "${AWL_NATIVE_GATE_PROBE_HANG_LINE:-}" ]]; then
  printf 'test the_test_that_never_returned ... '
fi
# A grandchild that a bare `kill $cargo_pid` cannot reach. On a real runner this
# is the test binary itself, and it is what the 2026-08-02 job cleanup had to
# reap by hand after the gate had already exited.
if [[ -n "${AWL_NATIVE_GATE_PROBE_ORPHAN_FILE:-}" ]]; then
  ( trap '' TERM; sleep 600 ) &
  printf '%s\n' "$!" >>"$AWL_NATIVE_GATE_PROBE_ORPHAN_FILE"
fi
if [[ "$convention" == canary ]]; then
  sleep "${AWL_NATIVE_GATE_PROBE_CANARY_SLEEP:-0}"
elif [[ -n "${AWL_NATIVE_GATE_PROBE_SPIN_SECONDS:-}" ]]; then
  # A LIVELOCK, as opposed to the sleeping fixture's deadlock. Both leave memory
  # flat and both stop producing output; only CPU tells them apart, so the
  # fixture has to be able to be either. The spin is bash's own `SECONDS`
  # builtin and an empty loop body: no fork per iteration, so the CPU it burns
  # is attributed to THIS pid, which is the pid the heartbeat has to name.
  # The spinner is a CHILD born after a delay, so it is deterministically
  # absent from the heartbeat sample before it and present in the next — the
  # NEWCOMER case, which the probe must measure over the process's own age
  # rather than drop. Letting the conventions themselves be the newcomers was
  # flaky: whether they beat the gate's baseline sample was a fork race.
  sleep "${AWL_NATIVE_GATE_PROBE_SPIN_DELAY:-0}"
  spin_wall_start="$SECONDS"
  bash -c 'printf "%s\n" "$$" >>"$1"; SECONDS=0; while (( SECONDS < $2 )); do :; done' \
    _ "$AWL_NATIVE_GATE_PROBE_SPIN_PID_FILE" "$AWL_NATIVE_GATE_PROBE_SPIN_SECONDS"
  # GROUND TRUTH, independent of the heartbeat's own `ps -A` sampling: `times`
  # reads the kernel's rusage accounting for the child that just exited, so it
  # reports what the spinner ACTUALLY got — on an idle host or a loaded one —
  # without going anywhere near the `ps`-based mechanism under test.
  if [[ -n "${AWL_NATIVE_GATE_PROBE_SPIN_TRUTH_FILE:-}" ]]; then
    spin_wall_seconds=$(( SECONDS - spin_wall_start ))
    (( spin_wall_seconds < 1 )) && spin_wall_seconds=1
    # `times` must run un-subshelled: `$(times)` forks a command-substitution
    # subshell, and a fresh subshell has no children of its own yet, so it
    # silently reports zero for the very child this line exists to measure.
    # `{ times; } >file` redirects the current shell's own builtin instead.
    times_tmp="$AWL_NATIVE_GATE_PROBE_SPIN_TRUTH_FILE.times.$$"
    { times; } >"$times_tmp"
    children_line="$(sed -n '2p' "$times_tmp")"
    rm -f "$times_tmp"
    cpu_seconds="$(awk '{
        total = 0
        for (i = 1; i <= NF; i++) {
          t = $i; sub(/s$/, "", t); split(t, mm, "m")
          total += mm[1] * 60 + mm[2]
        }
        printf "%.3f", total
      }' <<<"$children_line")"
    printf 'cpu_seconds=%s wall_seconds=%s\n' "$cpu_seconds" "$spin_wall_seconds" \
      >>"$AWL_NATIVE_GATE_PROBE_SPIN_TRUTH_FILE"
  fi
else
  sleep "${AWL_NATIVE_GATE_PROBE_SLEEP:-0.2}"
fi
printf 'finish %s\n' "$convention" >>"$AWL_NATIVE_GATE_PROBE_LOG"
if [[ "$convention" == "${AWL_NATIVE_GATE_FAIL_CONVENTION:-}" ]]; then
  exit "${AWL_NATIVE_GATE_FAIL_STATUS:-1}"
fi
# A MENU-BAR arm failing, keyed on the forcing rather than the convention. Both
# guards read the same `AWL_NATIVE_GATE_FAIL_STATUS`, and the `-n` test keeps
# this inert for the canary and the two conventions, which carry no forcing.
if [[ -n "${AWL_MENU_BAR_FORCE:-}" \
  && "$AWL_MENU_BAR_FORCE" == "${AWL_NATIVE_GATE_FAIL_MENU_BAR:-}" ]]; then
  exit "${AWL_NATIVE_GATE_FAIL_STATUS:-1}"
fi
EOF
chmod +x "$WORK/cargo"

# THE FREE-ORACLE IS DERIVED FROM DISK-PREFLIGHT'S OWN FLOOR, NOT A REMEMBERED
# NUMBER. A hardcoded 40 GiB sat above the healthy floor (27 GiB) only by
# coincidence: raise HEALTHY_BYTES past it and every probe below silently stops
# exercising the no-recovery path and starts taking the preflight's real lock
# and sweep — a policy change with no test going red. Read the real floor off
# disk-preflight's own receipt (the same technique test-disk-preflight.sh's
# band_field uses) rather than parsing its source arithmetic, so a change to
# HOW the floor is computed cannot desync the two files — only a huge
# probe value is needed here, so the healthy branch is reached regardless of
# what the real floor currently is.
disk_preflight_floor_oracle="$WORK/disk-preflight-floor-oracle"
cat >"$disk_preflight_floor_oracle" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' $((1024 * 1024 * 1024 * 1024))
EOF
chmod +x "$disk_preflight_floor_oracle"
disk_preflight_floor_receipt="$(
  env -u CI \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$disk_preflight_floor_oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-preflight-floor-lock" \
    "$ROOT/.orchestrator/disk-preflight.sh"
)"
disk_preflight_healthy_bytes="${disk_preflight_floor_receipt#*healthy_bytes=}"
disk_preflight_healthy_bytes="${disk_preflight_healthy_bytes%% *}"
[[ "$disk_preflight_healthy_bytes" =~ ^[0-9]+$ ]] || {
  echo "test-native-gate: could not read disk-preflight's healthy_bytes floor from its own receipt: $disk_preflight_floor_receipt" >&2
  exit 1
}
# A STATED margin, not a coincidence: every probe's oracle sits exactly this
# far above the real floor, so the gap is legible on its own rather than only
# discoverable by reading two files side by side.
readonly free_oracle_margin_bytes=$((1 * 1024 * 1024 * 1024))
free_oracle_bytes=$((disk_preflight_healthy_bytes + free_oracle_margin_bytes))

cat >"$WORK/free-oracle" <<EOF
#!/usr/bin/env bash
printf '%s\n' $free_oracle_bytes
EOF
chmod +x "$WORK/free-oracle"

# LAW: the oracle exceeds the real floor by exactly the stated margin — a
# presence floor first, since a zero-or-negative margin would satisfy a bare
# ">" comparison for free (the shape where a law is satisfiable by deleting
# its own subject).
(( free_oracle_margin_bytes > 0 )) || {
  echo "test-native-gate: free_oracle_margin_bytes=$free_oracle_margin_bytes; a non-positive margin makes the coupling law vacuous" >&2
  exit 1
}
(( free_oracle_bytes - disk_preflight_healthy_bytes == free_oracle_margin_bytes )) || {
  echo "test-native-gate: free-oracle is $free_oracle_bytes but the real healthy floor is $disk_preflight_healthy_bytes (want exactly +$free_oracle_margin_bytes)" >&2
  exit 1
}

# MUTATION PROOF: raise the floor the way the item warns about — past the OLD
# hardcoded 40 GiB — and show (a) the scenario is real, (b) this file's DERIVED
# oracle keeps its margin with no code change, and (c) the retired hardcode
# would have gone silently wrong under exactly this mutation, which is the bug
# this fix retires.
legacy_hardcoded_oracle_bytes=$((40 * 1024 * 1024 * 1024))
mutated_disk_preflight="$WORK/disk-preflight-with-raised-floor.sh"
cp "$ROOT/.orchestrator/disk-preflight.sh" "$mutated_disk_preflight"
perl -pi -e 's/readonly MINIMUM_BYTES=\$\(\(24 \* 1024 \* 1024 \* 1024\)\)/readonly MINIMUM_BYTES=\$((60 * 1024 * 1024 * 1024))/' \
  "$mutated_disk_preflight"
chmod +x "$mutated_disk_preflight"
mutated_floor_receipt="$(
  env -u CI \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$disk_preflight_floor_oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-preflight-mutated-floor-lock" \
    "$mutated_disk_preflight"
)"
mutated_healthy_bytes="${mutated_floor_receipt#*healthy_bytes=}"
mutated_healthy_bytes="${mutated_healthy_bytes%% *}"
[[ "$mutated_healthy_bytes" =~ ^[0-9]+$ ]] || {
  echo "test-native-gate: mutation fixture did not raise a readable floor: $mutated_floor_receipt" >&2
  exit 1
}
(( mutated_healthy_bytes > legacy_hardcoded_oracle_bytes )) || {
  echo "test-native-gate: mutation fixture raised the floor to $mutated_healthy_bytes, which is not past the legacy hardcode $legacy_hardcoded_oracle_bytes — the scenario this law guards against was not reproduced" >&2
  exit 1
}
mutated_derived_oracle_bytes=$((mutated_healthy_bytes + free_oracle_margin_bytes))
(( mutated_derived_oracle_bytes - mutated_healthy_bytes == free_oracle_margin_bytes )) || {
  echo "test-native-gate: derivation did not keep its margin under a raised floor: floor=$mutated_healthy_bytes derived=$mutated_derived_oracle_bytes" >&2
  exit 1
}
(( legacy_hardcoded_oracle_bytes <= mutated_healthy_bytes )) || {
  echo "test-native-gate: mutation did not exceed the legacy hardcode; the regression this law names was not reproduced" >&2
  exit 1
}
echo "test-native-gate: free-oracle margin law — real floor $disk_preflight_healthy_bytes, oracle $free_oracle_bytes (margin $free_oracle_margin_bytes); a raised floor of $mutated_healthy_bytes keeps its margin under the derived value and would have silently passed the retired $legacy_hardcoded_oracle_bytes hardcode"

cat >"$WORK/git" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == 'rev-parse --path-format=absolute --git-common-dir' \
  && -n "${AWL_NATIVE_GATE_PROBE_COMMON_GIT_DIR:-}" ]]; then
  printf '%s\n' "$AWL_NATIVE_GATE_PROBE_COMMON_GIT_DIR"
elif [[ "$*" == 'status --short' ]]; then
  # Intercepted UNCONDITIONALLY, not only when a probe asks for dirt: this
  # repo's own working tree is ambient state a probe does not control, and
  # CLAUDE.md's own workflow runs code-health.sh (which reaches this file)
  # from a deliberately dirty tree (after `git add`, before commit). Default
  # clean; the one law that is actually about this axis sets the override.
  if [[ -n "${AWL_NATIVE_GATE_PROBE_DIRTY_STATUS:-}" ]]; then
    printf '%s\n' "$AWL_NATIVE_GATE_PROBE_DIRTY_STATUS"
  fi
else
  /usr/bin/git "$@"
fi
EOF
chmod +x "$WORK/git"

run_probe() {
  local failing="${1:-}" status="${2:-0}" output="$WORK/output-${1:-success}"
  : >"$WORK/events"
  set +e
  PATH="$WORK:$PATH" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
    AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
    AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
    AWL_NATIVE_GATE_FAIL_CONVENTION="$failing" \
    AWL_NATIVE_GATE_FAIL_STATUS="$status" \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-${1:-success}" \
    "$ROOT/scripts/native-gate.sh" >"$output" 2>&1
  probe_status=$?
  set -e
}

# The bound is a launch seam, so it is asserted where it lands: in the child's
# environment. The machine axis swept here is the one the gate exists for — a
# hosted runner far smaller than the host the gate was written on — plus the
# case a core count alone cannot see, a wide box with a starved RAM budget.
probe_bound() {
  local cpus="$1" mem_bytes="$2" caller="${3:-}" label="bound-$1-$2-${3:-default}"
  : >"$WORK/events"
  set +e
  env -u RUST_TEST_THREADS \
    PATH="$WORK:$PATH" \
    ${caller:+RUST_TEST_THREADS="$caller"} \
    AWL_NATIVE_GATE_CPUS="$cpus" \
    AWL_NATIVE_GATE_MEM_BYTES="$mem_bytes" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
    AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
    AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-$label" \
    "$ROOT/scripts/native-gate.sh" >"$WORK/output-$label" 2>&1
  probe_status=$?
  set -e
  bound_output="$WORK/output-$label"
}

assert_bound() {
  local cpus="$1" mem_bytes="$2" caller="$3" expected="$4" convention seen
  probe_bound "$cpus" "$mem_bytes" "$caller"
  (( probe_status == 0 )) || {
    echo "test-native-gate: bound probe cpus=$cpus mem=$mem_bytes failed ($probe_status)" >&2
    tail -n 20 "$bound_output" >&2
    exit 1
  }
  for convention in mac linux; do
    seen="$(awk -v c="$convention" '$1 == "threads" && $2 == c { print $3 }' "$WORK/events")"
    [[ "$seen" == "$expected" ]] || {
      echo "test-native-gate: cpus=$cpus mem=$mem_bytes caller=${caller:-none} gave the $convention convention RUST_TEST_THREADS=$seen, expected $expected" >&2
      exit 1
    }
  done
  grep -Fq "native-gate-env cpus=$cpus mem_bytes=$mem_bytes conventions=2 test_threads=$expected" "$bound_output" || {
    echo "test-native-gate: machine receipt did not name cpus=$cpus mem=$mem_bytes test_threads=$expected" >&2
    exit 1
  }
}

# A three-vCPU hosted runner gets one thread per convention (two on three
# cores); the ten-core dev host keeps five apiece, which is its core count in
# total rather than twice it; a wide box with 2 GiB is bounded by RAM, not
# cores; and a caller that states a value owns it.
assert_bound 3 $((7 * 1024 * 1024 * 1024)) "" 1
assert_bound 10 $((64 * 1024 * 1024 * 1024)) "" 5
assert_bound 64 $((2 * 1024 * 1024 * 1024)) "" 16
assert_bound 10 $((64 * 1024 * 1024 * 1024)) 3 3

echo "test-native-gate: the thread bound tracks cores and RAM, defers to a caller, and is receipted"

# A gate that outruns its budget must fail LOUDLY and IN BAND. The whole point
# is a starved CI runner that dies mid-step and uploads no log at all: a gate
# that ends itself first leaves a log behind that names what happened.
: >"$WORK/events"
set +e
env -u RUST_TEST_THREADS \
  PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_BUDGET_SECONDS=1 \
  AWL_NATIVE_GATE_PROBE_SLEEP=60 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-budget" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-budget" 2>&1
budget_status=$?
set -e
(( budget_status == 1 )) || {
  echo "test-native-gate: an exhausted budget returned $budget_status, expected gate status 1" >&2
  exit 1
}
grep -Fq 'native-gate: ABORTED on its 1s budget' "$WORK/output-budget" || {
  echo "test-native-gate: an exhausted budget did not name itself in the output" >&2
  exit 1
}
if grep -Fq 'native-gate-receipt' "$WORK/output-budget"; then
  echo "test-native-gate: an exhausted budget leaked a receipt" >&2
  exit 1
fi

echo "test-native-gate: an exhausted budget ends the run in band, by name, with no receipt"

# A general probe runner for the phase/budget laws below. Every one of them
# asserts against the gate's real output, on this host, with a fixture that
# emits the exact line shapes the 2026-08-02 mac runner emitted.
probe() {
  local label="$1" probe_started
  shift
  : >"$WORK/events"
  probe_started="$(date +%s)"
  set +e
  env -u RUST_TEST_THREADS -u AWL_NATIVE_GATE_BUDGET_SECONDS -u AWL_NATIVE_GATE_DEADLINE_EPOCH \
    PATH="$WORK:$PATH" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
    AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
    AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-$label" \
    "$@" \
    "$ROOT/scripts/native-gate.sh" >"$WORK/output-$label" 2>&1
  probe_status=$?
  set -e
  probe_elapsed=$(( $(date +%s) - probe_started ))
  probe_output="$WORK/output-$label"
}

require() {
  grep -Fq "$2" "$probe_output" || {
    echo "test-native-gate: $1 — missing from the gate's output: $2" >&2
    exit 1
  }
}

refuse() {
  if grep -Fq "$2" "$probe_output"; then
    echo "test-native-gate: $1 — the gate emitted what it must not: $2" >&2
    exit 1
  fi
}

# The direct child of $1 whose own child is a `sleep` process. No budget is
# armed in the callers below, so gate_vitals_loop is the only child that ever
# spawns one — this is the same evidence a live host's `ps -ww` showed for
# the leaked orphans this item was opened against: a pid whose one child is
# sitting in `sleep`. Bash-3.2-safe (no `mapfile`), matching this repo's
# other portability notes.
#
# The search POLLS, for the same reason the marker waits are loops: the marker
# and the vitals loop are two separate spawns. A fixture pid file avoids a
# process-table dependency on managed runners; the live census remains the
# fallback for an ordinary host.
find_vitals_pid() {
  local gate_pid="$1" pid_file="${2:-}" child grandchild attempt
  for attempt in $(seq 1 50); do
    if [[ -n "$pid_file" && -s "$pid_file" ]]; then
      child="$(cat "$pid_file")"
      if kill -0 "$child" 2>/dev/null; then
        printf '%s\n' "$child"
        return 0
      fi
    fi
    for child in $(pgrep -P "$gate_pid" 2>/dev/null || true); do
      for grandchild in $(pgrep -P "$child" 2>/dev/null || true); do
        if ps -ww -o command= -p "$grandchild" 2>/dev/null | grep -q '^sleep '; then
          printf '%s\n' "$child"
          return 0
        fi
      done
    done
    kill -0 "$gate_pid" 2>/dev/null || return 1
    sleep 0.1
  done
  return 1
}

# ── Per-phase timing ─────────────────────────────────────────────────────────
# The question this exists to answer is whether a 40-minute step is COMPILING
# test harnesses or RUNNING tests. Cargo already announces both boundaries; the
# gate must stamp them, per convention, and it must not be possible for a per-
# test line to forge one — the fixture's test is named
# `Running_tests::a_name_with_(parens)_and_Finished_target(s) in`, which
# contains every token the matchers key on.
probe phases AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT=1
(( probe_status == 0 )) || { echo "test-native-gate: phase probe failed ($probe_status)" >&2; exit 1; }
for convention in mac linux; do
  require "phase timing" "native-gate-phase label=$convention event=compile-finished elapsed_seconds="
  require "phase timing" "native-gate-phase label=$convention event=first-tests-running elapsed_seconds="
  require "phase timing" "native-gate-phase label=$convention event=suite-end elapsed_seconds="
  require "phase timing" "target=awl-a623f1caab4"
  require "phase timing" "target=harness-0f0f0f0f"
  require "phase timing" "detail=ok. 3484 passed"
  # Both conventions write one stdout. Without a label, "which convention got
  # where" is not recoverable from the log at all — which is what made the one
  # surviving mac log take line-by-line archaeology to read.
  require "convention labelling" "$convention| running 3484 tests"
done
require "phase timing" "native-gate-phase label=canary event=begin"
require "phase timing" "native-gate-phase label=canary event=end elapsed_seconds="
# The fixture runs two real targets and prints two hostile test names: one
# carrying "Running_tests" and "(parens)", one carrying "Finished" and
# "target(s) in". Counting is the assertion — a marker that exists is not the
# same as a marker that is right, and an over-count is exactly how a per-test
# line quietly becomes a phase boundary.
for convention in mac linux; do
  for event in target-start:2 compile-finished:1 first-tests-running:1; do
    seen="$(grep -c "native-gate-phase label=$convention event=${event%%:*} " "$probe_output" || true)"
    [[ "$seen" == "${event##*:}" ]] || {
      echo "test-native-gate: $convention stamped $seen ${event%%:*} markers over ${event##*:} real ones — a test NAME forged a phase boundary" >&2
      exit 1
    }
  done
done

echo "test-native-gate: every phase boundary is stamped per convention, and a test name cannot forge one"

# ── The line a hang leaves behind ────────────────────────────────────────────
# The only thing that identified 2026-08-02's hang was libtest's unterminated
# "test NAME ... " fragment, and it survived by luck: an interleaved heartbeat
# write happened to flush it. The gate must produce it on purpose.
probe hang-line AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=30 \
  AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT=1 AWL_NATIVE_GATE_PROBE_HANG_LINE=1
(( probe_status == 1 )) || { echo "test-native-gate: hang-line probe returned $probe_status" >&2; exit 1; }
for convention in mac linux; do
  require "hang line" "$convention| test the_test_that_never_returned ... "
done

echo "test-native-gate: a killed convention still flushes the unterminated line naming the test that hung"

# ── The budget covers the canary ─────────────────────────────────────────────
# The first draft armed the budget only after the canary had returned, leaving
# the whole dependency-and-library compile — the slowest phase on a cold hosted
# runner — with no watchdog at all.
probe canary-budget AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_CANARY_SLEEP=30
(( probe_status == 1 )) || {
  echo "test-native-gate: a canary that outran the budget returned $probe_status, expected 1" >&2
  exit 1
}
require "canary budget" "budget expired during the canary phase"
refuse "canary budget" "native-gate-receipt"
grep -Fq 'finish canary' "$WORK/events" && {
  echo "test-native-gate: the budget did not actually end the canary — it ran to completion" >&2
  exit 1
}
# The fixture hangs for 30 s. A gate whose watchdog does not reach this phase
# does not fail — it WAITS, which is the whole defect, so the law is stated in
# wall-clock: a 2 s budget plus its 5 s escalation must land nowhere near 30.
(( probe_elapsed < 20 )) || {
  echo "test-native-gate: the gate took ${probe_elapsed}s to end a 2s budget — it waited out the hang instead of ending it" >&2
  exit 1
}

echo "test-native-gate: the budget reaches the canary phase, not only the concurrent suites"

# ── The budget ends the whole process GROUP ──────────────────────────────────
# `kill $cargo_pid` retires `env … cargo test` and nothing below it. On
# A gate cleanup can leave survivors after it exits; a survivor holds the step's stdout, and a GitHub
# step does not conclude while that pipe is open. The fixture's grandchild
# IGNORES SIGTERM on purpose, so only a group-directed KILL can retire it — a
# gate that merely signalled its direct children would leave it running.
: >"$WORK/orphans"
probe group-kill AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=30 \
  AWL_NATIVE_GATE_PROBE_ORPHAN_FILE="$WORK/orphans"
(( probe_status == 1 )) || {
  echo "test-native-gate: the group-kill probe returned $probe_status, expected 1" >&2
  exit 1
}
[[ -s "$WORK/orphans" ]] || {
  echo "test-native-gate: the fixture spawned no grandchild, so this law proved nothing" >&2
  exit 1
}
survivors=""
while read -r orphan; do
  [[ -n "$orphan" ]] || continue
  if kill -0 "$orphan" 2>/dev/null; then
    survivors="$survivors $orphan"
    kill -KILL "$orphan" 2>/dev/null || true
  fi
done <"$WORK/orphans"
[[ -z "$survivors" ]] || {
  echo "test-native-gate: the budget left grandchildren alive ($survivors) — they hold the CI step's stdout open" >&2
  exit 1
}
if ps -A -o pid=,ppid=,pgid=,etime=,time=,rss=,stat=,comm= >/dev/null 2>&1; then
  require "group kill" "native-gate-budget-proc"
else
  echo "test-native-gate: SKIPPED process diagnostic: process table unavailable"
fi

echo "test-native-gate: an exhausted budget retires every descendant, not just the process it launched"

# ── …and STOPS at the caller's process group ─────────────────────────────────
# code-health.sh runs this file, and both inherit the process group of whatever
# shell launched them — an agent's tool shell, with every other command that
# shell is running. The law above requires the budget to reach the gate's own
# descendants. This one requires it to reach no further, because the failure at
# that end leaves nothing to read: the shell dies mid-run, its turn ends with no
# output and no error, and the gate it launched keeps going with nobody waiting
# on it. A widened `gate_kill_groups` is the one edit that can cause it.
#
# THREE LAYERS, and the middle one is what makes the law reportable at all. A
# bystander in the group under test cannot report its own death, and neither
# can a script sharing that group. So the stand-in caller is a subshell with a
# process group of its OWN: it plants the bystander, runs the probe as a plain
# child exactly as a shell does, and leaves a verdict this script reads from
# outside that group. A widened kill takes the stand-in and its bystander
# together; the reader survives to name it.
caller_group_verdict="$WORK/caller-group-verdict"
: >"$caller_group_verdict"
set -m
(
  set +m
  sleep 900 &
  bystander=$!
  printf 'planted=%s\n' "$bystander" >>"$caller_group_verdict"
  probe caller-group AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=30
  printf 'probe_status=%s\n' "$probe_status" >>"$caller_group_verdict"
  if kill -0 "$bystander" 2>/dev/null; then
    printf 'bystander=survived\n' >>"$caller_group_verdict"
  else
    printf 'bystander=reaped\n' >>"$caller_group_verdict"
  fi
  kill -KILL "$bystander" 2>/dev/null || true
) &
caller_group_stand_in=$!
set +m
# ENROLMENT, read off the live process table while the stand-in is still
# running rather than assumed from `set -m`: a stand-in sharing THIS script's
# group could not have survived a widened kill, so the law would be
# structurally unable to report the defect it names. Read from HERE, because
# `$$` inside a subshell is still the parent's pid and Bash 3.2 — what macOS
# ships, and what this file runs under — has no `BASHPID` to ask instead.
own_group="$(ps -o pgid= -p $$ | tr -d ' ')"
stand_in_group="$(ps -o pgid= -p "$caller_group_stand_in" | tr -d ' ')"
set +e
wait "$caller_group_stand_in"
caller_group_status=$?
set -e

[[ -n "$stand_in_group" && "$stand_in_group" != "$own_group" ]] || {
  echo "test-native-gate: the caller-group stand-in ran in this script's own group (stand_in=${stand_in_group:-unreadable}, self=$own_group) — it could not have outlived a widened kill, so this law would prove nothing" >&2
  exit 1
}
# PRESENCE, twice: a bystander that was never planted survives for free, and so
# does one whose probe never armed a budget and therefore killed no group at all.
grep -q '^planted=' "$caller_group_verdict" || {
  echo "test-native-gate: no bystander was planted beside the gate, so this law proves nothing" >&2
  exit 1
}
# The stand-in's own fate is read FIRST, because a widened kill takes it before
# it can write any later verdict line — and every downstream assertion would
# then blame a missing line rather than the reap that removed it.
(( caller_group_status == 0 )) || {
  echo "test-native-gate: a group kill reached the CALLER's process group — the stand-in caller did not survive the gate it launched (exit $caller_group_status). This is how a lane's shell dies mid-run with nothing to read: $(tr '\n' ' ' <"$caller_group_verdict")" >&2
  exit 1
}
grep -Fxq 'probe_status=1' "$caller_group_verdict" || {
  echo "test-native-gate: the caller-group probe did not end on its budget ($(sed -n 's/^probe_status=//p' "$caller_group_verdict")) — no group kill ran, so a survivor means nothing" >&2
  exit 1
}
grep -Fxq 'bystander=survived' "$caller_group_verdict" || {
  echo "test-native-gate: a group kill reached the CALLER's process group — the bystander planted beside the gate was reaped: $(tr '\n' ' ' <"$caller_group_verdict")" >&2
  exit 1
}

echo "test-native-gate: the group kill stops at the gate's own phases — a shell that launches code-health keeps its other children"

# ── The budget is anchored to the caller's clock, not only to the gate's ─────
# The runner's death clock starts at job step 1; this script's starts whenever
# the earlier steps happen to have finished. On 2026-08-02 the same 2400 s
# duration meant job-minute 41 on a cold cache and job-minute 42 on a hot one —
# and the runner was lost at 53. An absolute deadline pins the end of the gate
# to the clock that is actually killing it.
now="$(date +%s)"
probe deadline-only AWL_NATIVE_GATE_DEADLINE_EPOCH=$(( now + 2 )) AWL_NATIVE_GATE_PROBE_SLEEP=30
(( probe_status == 1 )) || {
  echo "test-native-gate: an absolute deadline alone did not end the gate (status $probe_status)" >&2
  exit 1
}
require "deadline" "budget_source=deadline"
refuse "deadline" "native-gate-receipt"

now="$(date +%s)"
probe deadline-wins AWL_NATIVE_GATE_BUDGET_SECONDS=3600 \
  AWL_NATIVE_GATE_DEADLINE_EPOCH=$(( now + 2 )) AWL_NATIVE_GATE_PROBE_SLEEP=30
(( probe_status == 1 )) || {
  echo "test-native-gate: a near deadline lost to a distant duration (status $probe_status)" >&2
  exit 1
}
require "deadline" "budget_source=deadline"

probe duration-wins AWL_NATIVE_GATE_BUDGET_SECONDS=2 \
  AWL_NATIVE_GATE_DEADLINE_EPOCH=$(( now + 3600 )) AWL_NATIVE_GATE_PROBE_SLEEP=30
(( probe_status == 1 )) || {
  echo "test-native-gate: a near duration lost to a distant deadline (status $probe_status)" >&2
  exit 1
}
require "deadline" "budget_seconds=2 budget_source=duration"

echo "test-native-gate: the budget takes whichever of its duration and its absolute deadline comes first"

# ── The heartbeat and the abort both name where each convention got to ───────
# Reconstructing the one surviving mac log meant reading 6500 interleaved lines
# to find the test that never returned. The gate must say it outright, while it
# is still running and again when it gives up.
probe last-progress AWL_NATIVE_GATE_VITALS_SECONDS=1 AWL_NATIVE_GATE_PROBE_SLEEP=4 \
  AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT=1
(( probe_status == 0 )) || { echo "test-native-gate: last-progress probe failed ($probe_status)" >&2; exit 1; }
require "heartbeat progress" "mac_last=[test result: ok. 2 passed"
require "heartbeat progress" "linux_last=[test result: ok. 2 passed"

probe abort-progress AWL_NATIVE_GATE_BUDGET_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=30 \
  AWL_NATIVE_GATE_PROBE_CARGO_OUTPUT=1
(( probe_status == 1 )) || { echo "test-native-gate: abort-progress probe returned $probe_status" >&2; exit 1; }
require "abort progress" "native-gate-budget-last label=mac line=[test result: ok. 2 passed"
require "abort progress" "native-gate-budget-last label=linux line=[test result: ok. 2 passed"

echo "test-native-gate: the heartbeat and the abort both name the last line each convention reached"

# The heartbeat is the only thing that will describe the machine while a slow
# gate is still alive, so a heartbeat carrying a placeholder is worse than
# none — it reads as real. This asserts the probe against THIS host's real
# kernel counters: the first draft parsed macOS's page size out of the wrong
# field and every sample reported free_bytes=0 through a full green gate.
: >"$WORK/events"
set +e
env -u RUST_TEST_THREADS \
  PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_VITALS_SECONDS=1 \
  AWL_NATIVE_GATE_PROBE_SLEEP=3 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-vitals" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-vitals" 2>&1
vitals_status=$?
set -e
(( vitals_status == 0 )) || {
  echo "test-native-gate: vitals probe failed ($vitals_status)" >&2
  exit 1
}
vitals_free="$(awk '/^native-gate-vitals/ { for (i = 1; i <= NF; i++) if ($i ~ /^free_bytes=/) { sub(/free_bytes=/, "", $i); print $i; exit } }' "$WORK/output-vitals")"
[[ -n "$vitals_free" ]] || {
  echo "test-native-gate: a running suite emitted no vitals heartbeat at all" >&2
  exit 1
}
[[ "$vitals_free" =~ ^[0-9]+$ ]] && (( vitals_free > 0 )) || {
  echo "test-native-gate: the vitals heartbeat reported free_bytes=$vitals_free — the memory probe does not work on this host" >&2
  exit 1
}

echo "test-native-gate: the vitals heartbeat reaches this host's real memory counters while the suite runs"

# ── Deadlock or livelock ─────────────────────────────────────────────────────
# Flat memory and zero swap are equally consistent with processes blocked on a
# fence and processes spinning on one, and the fixes have nothing in common.
# The laws below are stated as a PAIR on purpose: each one alone is satisfiable
# by a probe that always answers the same thing, and the pair is not. A probe
# hardwired to 0 fails the spinning direction; one hardwired to 100 fails the
# sleeping direction; a probe that finds no processes at all fails both.

# `busiest=[name:pid=pct]` is one whitespace-free field, so the highest reading
# across every heartbeat of a run is one awk pass. The MAX is the statistic that
# matters: a spin only has to show up in one heartbeat to be diagnosed.
busiest_peak() {
  awk 'BEGIN { peak = -1 }
    /^native-gate-vitals/ {
      for (i = 1; i <= NF; i++) if (index($i, "busiest=[") == 1) {
        value = $i; sub(/.*=/, "", value); sub(/\]$/, "", value)
        if (value + 0 > peak) { peak = value + 0; who = $i }
      }
    } END { printf "%.1f %s\n", peak, (who == "" ? "busiest=[absent]" : who) }' "$probe_output"
}

vitals_peak() {
  awk -v key="$1=" 'BEGIN { peak = -1 }
    /^native-gate-vitals/ {
      for (i = 1; i <= NF; i++) if (index($i, key) == 1) {
        value = substr($i, length(key) + 1) + 0
        if (value > peak) peak = value
      }
    } END { printf "%g\n", peak }' "$probe_output"
}

# The last heartbeat's reading of a scalar field — used for load1/cpu_count,
# which describe the HOST rather than a tracked process, so the most recent
# sample (closest to when a livelock law actually asserted) is the one worth
# printing beside that assertion.
vitals_field_last() {
  awk -v key="$1=" '
    /^native-gate-vitals/ {
      for (i = 1; i <= NF; i++) if (index($i, key) == 1) value = substr($i, length(key) + 1)
    }
    END { print (value == "" ? "unknown" : value) }
  ' "$probe_output"
}

# Ground truth for the livelock direction below, one line per spinner
# (`cpu_seconds=.. wall_seconds=..`), written by the fixture itself from
# bash's `times` builtin — the kernel's own rusage for the child that just
# exited, reached by a path that shares nothing with the heartbeat's `ps -A`
# sampling under test. The MAX across spinners is the fixture's own answer to
# "what did the busiest one actually get", on an idle host or a loaded one.
ground_truth_peak_pct() {
  awk -F'[= ]' '
    { if ($4 > 0) { pct = $2 * 100 / $4; if (pct > peak) peak = pct } }
    END { printf "%.1f", peak + 0 }
  ' "$1"
}

# The relationship, not the absolute: an absolute floor assumes the fixture
# always gets a whole core, which this fleet's whole design makes false. What
# the law actually needs is agreement between the heartbeat's own delta-over-
# `ps` reading and the ground truth above, over the same stretch — true on an
# idle host, true on a loaded one, and still false if the heartbeat loses
# sight of the process altogether, which is the defect this law exists for.
#
# A presence floor comes first, in the shape CLAUDE.md names for a law that
# grades a treatment: a ground truth near zero means the fixture itself never
# got meaningful CPU — the host is starved past what any comparison here can
# speak to — and a bare ratio against noise would be satisfiable by silence.
# That is reported as a loud, named skip, not a pass and not a hard failure:
# the whole point of this item is that a busy fleet must still get a receipt.
assert_pct_tracks_ground_truth() {
  local label="$1" heartbeat_pct="$2" heartbeat_who="$3" truth_pct="$4" load1="$5" cpu_count="$6"
  awk -v t="$truth_pct" 'BEGIN { exit !(t >= 5) }' || {
    echo "test-native-gate: SKIPPED $label: the spinner's own ground truth was only ${truth_pct}% of a core (load1=$load1 cpu_count=$cpu_count) — this host is too starved right now for the probe to say anything" >&2
    return 2
  }
  awk -v h="$heartbeat_pct" -v t="$truth_pct" 'BEGIN { exit !(h >= t * 0.5) }' || {
    echo "test-native-gate: $label read ${heartbeat_pct}% ($heartbeat_who) while the spinner's own ground truth was ${truth_pct}% of a core (load1=$load1 cpu_count=$cpu_count) — the heartbeat is not tracking the fixture's real CPU use" >&2
    exit 1
  }
}

# The system load average is the heartbeat's headline and it is the field most
# likely to come back as a brace or an empty string: macOS hands it over as
# `{ 5.70 12.72 16.79 }` and Linux as the first field of /proc/loadavg. A probe
# that could not read it must SAY so rather than report a confident 0.00, so
# this asserts a real number from this host's real kernel — the same shape of
# assertion the memory law above makes, for the same reason.
probe load-average AWL_NATIVE_GATE_VITALS_SECONDS=1 AWL_NATIVE_GATE_PROBE_SLEEP=3
(( probe_status == 0 )) || { echo "test-native-gate: load-average probe failed ($probe_status)" >&2; exit 1; }
load_seen="$(awk '/^native-gate-vitals/ { for (i = 1; i <= NF; i++) if (index($i, "load1=") == 1) { sub(/load1=/, "", $i); print $i; exit } }' "$probe_output")"
[[ "$load_seen" =~ ^[0-9]+\.?[0-9]*$ ]] || {
  echo "test-native-gate: the heartbeat reported load1=$load_seen — the load-average probe does not work on this host" >&2
  exit 1
}
require "load average" "cpu_count=$(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 1)"

echo "test-native-gate: the heartbeat reports this host's real load average beside the core count that makes it readable"

# Per-process CPU is diagnostic only. Keep every non-process heartbeat law live
# when a managed runner denies the exact process-table query used in production.
process_cpu_table_available=0
if ps -A -o pid=,pgid=,etime=,time=,comm= >/dev/null 2>&1; then
  process_cpu_table_available=1
fi

if (( process_cpu_table_available )); then
# Direction 1 — LIVELOCK. The fixture's conventions burn CPU in-process for 9 s
# while producing no output, which is exactly what a spinning test binary looks
# like from outside. The heartbeat must report a pegged core AND name the pid
# doing it: a bare load average would rise here too, and would not say which of
# the gate's own processes to attach a debugger to.
: >"$WORK/spinners"
: >"$WORK/spin-truth"
probe cpu-spin AWL_NATIVE_GATE_VITALS_SECONDS=3 AWL_NATIVE_GATE_PROBE_SPIN_SECONDS=9 \
  AWL_NATIVE_GATE_PROBE_SPIN_DELAY=4 AWL_NATIVE_GATE_PROBE_SPIN_PID_FILE="$WORK/spinners" \
  AWL_NATIVE_GATE_PROBE_SPIN_TRUTH_FILE="$WORK/spin-truth"
(( probe_status == 0 )) || { echo "test-native-gate: cpu-spin probe failed ($probe_status)" >&2; exit 1; }
[[ -s "$WORK/spinners" ]] || {
  echo "test-native-gate: the fixture never entered its spin, so this law proved nothing" >&2
  exit 1
}
[[ -s "$WORK/spin-truth" ]] || {
  echo "test-native-gate: the fixture recorded no ground truth for its own spin, so this law has no oracle to compare against" >&2
  exit 1
}
# THE CONFIGURATION THIS RAN IN, printed unconditionally — a reader sees, on a
# green run or a red one, whether this was an idle box or a busy afternoon
# without re-running anything.
spin_load1="$(vitals_field_last load1)"
spin_cpu_count="$(vitals_field_last cpu_count)"
spin_truth_peak="$(ground_truth_peak_pct "$WORK/spin-truth")"
read -r spin_peak spin_who <<<"$(busiest_peak)"
echo "test-native-gate: cpu-spin probe ran at load1=$spin_load1 cpu_count=$spin_cpu_count — the spinners' own ground truth peaked at ${spin_truth_peak}% of a core, the heartbeat's busiest reading was ${spin_peak}% ($spin_who)"

# The relationship, not the absolute: 0.5 preserves the exact historical
# guarantee on an idle host (ground truth ~100, so the floor is still ~50 —
# where `ps -o time=`'s whole-second quantisation on Linux can under-read a
# pegged process by about a third), and it scales down honestly on a loaded
# one instead of assuming a core this fleet does not promise to have free.
spin_direction_status=0
if assert_pct_tracks_ground_truth "the busiest tracked process" "$spin_peak" "$spin_who" \
  "$spin_truth_peak" "$spin_load1" "$spin_cpu_count"; then
  :
else
  spin_direction_status=$?
fi

# A process that appeared INSIDE the window must be measured over its own age,
# not dropped. The first draft dropped it, and the receipt run of 2026-08-02
# shows what that cost: two heartbeats reporting `tracked_procs=0` and `0.6%`
# while two test binaries were burning a core each, because Cargo had moved
# from one test target to the next inside the window. A probe that can only see
# processes older than a heartbeat is blind for the first minute of every run
# and blind again at every phase change — including the one the hang is near.
#
# The fixture's spinner is born mid-run for exactly this, so the heartbeat that
# first sees it is deterministically a newcomer reading.
read -r new_peak new_who <<<"$(awk 'BEGIN { peak = -1 }
    /^native-gate-vitals/ {
      fresh = -1; value = -1; who = ""
      for (i = 1; i <= NF; i++) {
        if (index($i, "new_procs=") == 1) fresh = substr($i, 11) + 0
        else if (index($i, "busiest=[") == 1) {
          who = $i; value = $i; sub(/.*=/, "", value); sub(/\]$/, "", value); value += 0
        }
      }
      if (fresh >= 1 && value > peak) { peak = value; bestwho = who; found = 1 }
    } END { if (found) printf "%.1f %s\n", peak, bestwho; else print "-1 no-heartbeat-reported-a-newcomer" }' "$probe_output")"
new_direction_status=0
if assert_pct_tracks_ground_truth "the busiest NEW process" "$new_peak" "$new_who" \
  "$spin_truth_peak" "$spin_load1" "$spin_cpu_count"; then
  :
else
  new_direction_status=$?
fi

if (( new_direction_status == 0 )); then
  new_pid="${new_who##*:}"; new_pid="${new_pid%%=*}"
  grep -Fxq "$new_pid" "$WORK/spinners" || {
    echo "test-native-gate: the newcomer heartbeat blamed pid $new_pid ($new_who), which is not one of the fixture's spinners ($(tr '\n' ' ' <"$WORK/spinners"))" >&2
    exit 1
  }
fi
if (( spin_direction_status == 0 )); then
  spin_pid="${spin_who##*:}"; spin_pid="${spin_pid%%=*}"
  grep -Fxq "$spin_pid" "$WORK/spinners" || {
    echo "test-native-gate: the heartbeat blamed pid $spin_pid ($spin_who) but the processes actually spinning were $(tr '\n' ' ' <"$WORK/spinners")— a load number nobody can attribute is not a diagnosis" >&2
    exit 1
  }
fi

if (( spin_direction_status == 0 && new_direction_status == 0 )); then
  echo "test-native-gate: a spinning convention is reported as a pegged core and named by pid, not merely as a busy machine"
else
  echo "test-native-gate: cpu-spin livelock law SKIPPED (see above) — load1=$spin_load1 cpu_count=$spin_cpu_count, ground truth ${spin_truth_peak}%"
fi

# Direction 2 — DEADLOCK. Same silence, same flat memory, zero CPU. This is the
# half that makes the pair non-vacuous, and `tracked_procs` is asserted
# separately: without it a probe that found NOTHING would pass this law while
# failing to measure anything at all.
probe cpu-idle AWL_NATIVE_GATE_VITALS_SECONDS=2 AWL_NATIVE_GATE_PROBE_SLEEP=7
(( probe_status == 0 )) || { echo "test-native-gate: cpu-idle probe failed ($probe_status)" >&2; exit 1; }
idle_procs="$(vitals_peak tracked_procs)"
awk -v procs="$idle_procs" 'BEGIN { exit !(procs >= 1) }' || {
  echo "test-native-gate: the heartbeat tracked $idle_procs processes while two conventions were running — it measured nothing, so its zero means nothing" >&2
  exit 1
}
read -r idle_peak idle_who <<<"$(busiest_peak)"
awk -v peak="$idle_peak" 'BEGIN { exit !(peak < 25) }' || {
  echo "test-native-gate: two conventions were blocked in sleep(1) and the busiest tracked process read ${idle_peak}% ($idle_who) — the CPU probe cannot tell a deadlock from a livelock" >&2
  exit 1
}
refuse "cpu idle" "cpu_probe=broken"

echo "test-native-gate: a blocked convention reads as idle over a heartbeat that still tracked its processes — the two failures are distinguishable"
else
  require "restricted CPU diagnostics" "cpu_probe=unavailable"
  echo "test-native-gate: SKIPPED per-process CPU diagnostics: process table unavailable"
fi

assert_concurrent_and_complete() {
  local first_two
  first_two="$(grep -E '^(start|finish) (mac|linux)$' "$WORK/events" | sed -n '1,2p' | sort)"
  [[ "$first_two" == $'start linux\nstart mac' ]] || {
    echo "test-native-gate: convention suites did not overlap: $first_two" >&2
    exit 1
  }
  [[ "$(grep -Ec '^finish (mac|linux)$' "$WORK/events")" == 2 ]] || {
    echo "test-native-gate: gate did not await both suites" >&2
    exit 1
  }
}

run_probe
(( probe_status == 0 )) || { echo "test-native-gate: success probe failed" >&2; exit 1; }
assert_concurrent_and_complete
grep -Fq 'native-gate-receipt' "$WORK/output-success" || {
  echo "test-native-gate: successful sibling suites emitted no receipt" >&2
  exit 1
}
grep -Fq "unit_tests=$(awk 'END { print NR }' "$WORK/awl-test-list") unit_shards=6 integration_targets=1" \
  "$WORK/output-success" || {
    echo "test-native-gate: receipt did not state its proved unit/shard/integration scope" >&2
    exit 1
  }
for convention in mac linux; do
  [[ "$(awk -v c="$convention" '$1 == "shard" && $2 == c { count++ } END { print count + 0 }' "$WORK/events")" == 6 ]] || {
    echo "test-native-gate: $convention did not execute all six proved binary shards" >&2
    exit 1
  }
done
grep -Fq 'native-test-shards verified full=' "$WORK/output-success" || {
  echo "test-native-gate: the successful receipt carried no binary completeness proof" >&2
  exit 1
}

probe one-shard AWL_NATIVE_GATE_SHARDS=1
(( probe_status == 0 )) || { echo "test-native-gate: one-shard probe failed ($probe_status)" >&2; exit 1; }
for convention in mac linux; do
  [[ "$(awk -v c="$convention" '$1 == "shard" && $2 == c { count++ } END { print count + 0 }' "$WORK/events")" == 1 ]] || {
    echo "test-native-gate: AWL_NATIVE_GATE_SHARDS=1 did not run one binary process for $convention" >&2
    exit 1
  }
done

probe shard-mutation AWL_NATIVE_GATE_PROBE_DELETE_PREFIX=1
(( probe_status == 1 )) || {
  echo "test-native-gate: deleting a generated prefix returned $probe_status, expected refusal status 1" >&2
  exit 1
}
require "shard mutation" "native-test-shards: completeness refusal"
require "shard mutation" "missing="
refuse "shard mutation" "native-gate-receipt"

echo "test-native-gate: six shards are complete, the one-shard wave knob is live, and deleting one generated prefix refuses by missing test name"

# ── NO SHARD MAY FINISH ANOTHER SHARD'S LINE ──
# The transcript is the only account anyone reads of a wave, and six shards
# share one stdout. libtest leaves "test NAME ... " unterminated while the test
# runs, so a neighbour's whole line can land inside it and the reader is handed
# a line that names one test and carries another's verdict — a green suite
# reading as a named test failing, with that name in no `failures:` block at
# all. The stub above stages exactly that collision without a race, so this
# probe answers the same way every time.
rm -rf "$WORK/splice"
probe shard-lines AWL_NATIVE_GATE_PROBE_SPLIT_LINES=1
(( probe_status == 0 )) || {
  echo "test-native-gate: the shard-lines probe failed ($probe_status)" >&2
  exit 1
}
# The negative first: it is the subject, and it names the exact shape.
refuse "shard lines" \
  "test probe::the_leaders_own_test ... test probe::a_follower_shards_test ... ok"
require "shard lines" "test probe::the_leaders_own_test ... ok"
require "shard lines" "test probe::a_follower_shards_test ... ok"
echo "test-native-gate: a shard's dangling test name is completed by its own shard, never by a neighbour's verdict"

for failing in mac linux; do
  run_probe "$failing" 23
  (( probe_status == 1 )) || {
    echo "test-native-gate: $failing failure returned $probe_status, expected gate status 1" >&2
    exit 1
  }
  if grep -Fq 'native-gate-receipt' "$WORK/output-$failing"; then
    echo "test-native-gate: $failing failure leaked a receipt" >&2
    exit 1
  fi
  grep -Fq "mac_status=$([[ $failing == mac ]] && echo 23 || echo 0) linux_status=$([[ $failing == linux ]] && echo 23 || echo 0)" \
    "$WORK/output-$failing" || {
      echo "test-native-gate: $failing failure did not preserve both statuses" >&2
      exit 1
    }
  assert_concurrent_and_complete
done

echo "test-native-gate: both conventions overlap, both statuses survive, and either failure suppresses the receipt"

# ── THE MENU-BAR AXIS IS ACTUALLY SWEPT, IN BOTH ARMS ────────────────────────
# The point of the arms is that the axis stops depending on who remembers to
# edit a source file, and the way that promise dies quietly is the arms simply
# not being there — a deleted `gate_launch` line reads as a smaller diff, not as
# lost coverage. So this law reads the FORCING each cargo invocation actually
# received, not the gate's prose about it.
#
# Both directions matter and the second is the one that makes the pair
# non-vacuous: `on` and `off` must each appear exactly once (an arm that ran
# twice with the same value would sweep one branch and report two), and the
# canary plus both conventions must appear with the forcing UNSET (a convention
# that inherited one would make the axis a property of the convention arms).
run_probe
(( probe_status == 0 )) || { echo "test-native-gate: menu-bar axis probe failed" >&2; exit 1; }
for arm in on off; do
  [[ "$(grep -Fxc "menubar $arm" "$WORK/events")" == 1 ]] || {
    echo "test-native-gate: the gate ran the AWL_MENU_BAR_FORCE=$arm arm $(grep -Fxc "menubar $arm" "$WORK/events") times, expected exactly 1 — the axis is not swept" >&2
    exit 1
  }
done
[[ "$(grep -Fxc 'menubar unset' "$WORK/events")" == 3 ]] || {
  echo "test-native-gate: expected the canary and both conventions (3 invocations) to carry NO menu-bar forcing, saw $(grep -Fxc 'menubar unset' "$WORK/events")" >&2
  exit 1
}

# Either arm failing suppresses the receipt, and BOTH statuses are preserved in
# the message — the same contract the conventions have, for the same reason: a
# report that collapsed the two could not say which branch of the axis is red.
for arm in on off; do
  probe "menubar-fail-$arm" AWL_NATIVE_GATE_FAIL_MENU_BAR="$arm" AWL_NATIVE_GATE_FAIL_STATUS=29
  (( probe_status == 1 )) || {
    echo "test-native-gate: a failing menu-bar $arm arm returned $probe_status, expected gate status 1" >&2
    exit 1
  }
  require "menu-bar $arm failure" "native-gate: menu-bar axis failure"
  require "menu-bar $arm failure" \
    "on_status=$([[ $arm == on ]] && echo 29 || echo 0) off_status=$([[ $arm == off ]] && echo 29 || echo 0)"
  refuse "menu-bar $arm failure" "native-gate-receipt"
done

echo "test-native-gate: the menu-bar axis runs both arms with the conventions unforced, and either arm's failure suppresses the receipt"

# ── THE FULL-SUITE ARM REACHES EVERY SHARD, NOT EVERY NAME ───────────────────
# The filtered pair above is the CI shape. The local shape runs one arm over the
# WHOLE binary unit-test suite, and the only thing that makes that worth a third
# suite is that it reaches the tests a name filter cannot find. So the law counts
# the forcing at the SHARD, and requires it on exactly as many shard processes as
# a convention gets: a full arm that quietly re-acquired a filter would run one
# process and still print `mode=full-suite`.
probe menubar-full AWL_NATIVE_GATE_MENUBAR_FULL=1
(( probe_status == 0 )) || {
  echo "test-native-gate: full-suite menu-bar probe failed ($probe_status)" >&2
  tail -n 20 "$probe_output" >&2
  exit 1
}
menubar_line="$(grep -F 'native-gate-menubar ' "$probe_output" || true)"
[[ "$menubar_line" == *"mode=full-suite"* ]] || {
  echo "test-native-gate: AWL_NATIVE_GATE_MENUBAR_FULL=1 did not announce a full-suite arm: [$menubar_line]" >&2
  exit 1
}
menubar_field() { sed -n "s/.* $1=\([^ ]*\).*/\1/p" <<<"$menubar_line"; }
menubar_ambient="$(menubar_field ambient)"
menubar_forced="$(menubar_field forced)"
# The arm is only worth a suite if it forces the branch this host does NOT run
# ambiently; forcing the ambient one would sweep what the conventions swept.
# (`menubar::tests::the_gate_forces_the_branch_this_host_lacks` pins the ambient
# itself against `platform_default`; this end pins the opposition.)
[[ "$menubar_ambient" != "$menubar_forced" && -n "$menubar_forced" ]] || {
  echo "test-native-gate: the full arm forces ambient=$menubar_ambient forced=$menubar_forced — it sweeps the branch the conventions already ran" >&2
  exit 1
}
shard_forced="$(grep -Fxc "shardbar $menubar_forced" "$WORK/events")"
convention_shards="$(grep -Fxc 'shardbar unset' "$WORK/events")"
[[ "$shard_forced" == 6 ]] || {
  echo "test-native-gate: the full menu-bar arm forced $shard_forced shard processes, expected 6 — it is not running the whole suite" >&2
  exit 1
}
[[ "$convention_shards" == 12 ]] || {
  echo "test-native-gate: expected both conventions' 12 shards to carry NO forcing, saw $convention_shards" >&2
  exit 1
}
require "full menu-bar arm" "native-gate-receipt"
require "full menu-bar arm" "menubar=full:$menubar_forced"
# The cheap pair must be GONE in this mode, not merely joined: leaving it would
# spend two more Cargo invocations sweeping a subset of what just ran.
[[ "$(grep -c '^menubar ' "$WORK/events")" == 3 ]] || {
  echo "test-native-gate: the full-suite mode still ran name-filtered arms" >&2
  exit 1
}

probe menubar-full-fail AWL_NATIVE_GATE_MENUBAR_FULL=1 \
  AWL_NATIVE_GATE_FAIL_MENU_BAR="$menubar_forced" AWL_NATIVE_GATE_FAIL_STATUS=29
(( probe_status == 1 )) || {
  echo "test-native-gate: a failing full menu-bar arm returned $probe_status, expected gate status 1" >&2
  exit 1
}
require "full menu-bar failure" "native-gate: menu-bar axis failure"
require "full menu-bar failure" "AWL_MENU_BAR_FORCE=$menubar_forced"
require "full menu-bar failure" "ambient $menubar_ambient"
refuse "full menu-bar failure" "native-gate-receipt"

# WHICH MODE A HOST GETS IS DERIVED, AND THE DERIVATION IS THE PART THAT ROTS.
# Unset the pin and ask the gate twice. `CI` present means the fleet already runs
# both ambients across jobs, so the cheap pair is enough there; absent, this host
# is the only host and the full arm runs. A default that silently became
# "filtered everywhere" would leave every local gate exactly as blind as it was.
probe_menubar_mode() {
  local label="$1" expected="$2"
  shift 2
  : >"$WORK/events"
  set +e
  env -u RUST_TEST_THREADS -u AWL_NATIVE_GATE_BUDGET_SECONDS -u AWL_NATIVE_GATE_MENUBAR_FULL "$@" \
    PATH="$WORK:$PATH" \
    AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
    AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
    AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
    AWL_DISK_PREFLIGHT_TEST_MODE=1 \
    AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
    AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-$label" \
    "$ROOT/scripts/native-gate.sh" >"$WORK/output-$label" 2>&1
  local status=$?
  set -e
  (( status == 0 )) || {
    echo "test-native-gate: menu-bar mode probe $label failed ($status)" >&2
    tail -n 20 "$WORK/output-$label" >&2
    exit 1
  }
  grep -Fq "native-gate-menubar mode=$expected" "$WORK/output-$label" || {
    echo "test-native-gate: $label expected mode=$expected, got [$(grep -F 'native-gate-menubar ' "$WORK/output-$label" || true)]" >&2
    exit 1
  }
}
probe_menubar_mode menubar-mode-local full-suite -u CI
probe_menubar_mode menubar-mode-ci name-filtered CI=true

echo "test-native-gate: the full menu-bar arm forces every shard on the branch this host lacks, names itself on failure, and is derived from CI rather than remembered"

# ── The full-gate arbiter: one holder, a visible queue, safe stale recovery ──
# Two full gates must not recreate the contention that sharding removed. The
# second probe has its own event log, so an empty log while it reports the first
# holder is evidence that it has not begun a canary or a suite behind our back.
arbiter_marker="$WORK/arbiter-marker"
arbiter_lock="$WORK/arbiter-lock"
: >"$WORK/events-arbiter-first"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$arbiter_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_lock" \
  AWL_NATIVE_GATE_PROBE_SLEEP=4 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-arbiter-first" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-arbiter-first" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-arbiter-first" 2>&1 &
arbiter_first_pid=$!

for _ in $(seq 1 50); do
  [[ -s "$arbiter_marker" ]] && break
  sleep 0.1
done
[[ -s "$arbiter_marker" ]] || {
  echo "test-native-gate: the first arbiter probe never published a holder" >&2
  kill -TERM "$arbiter_first_pid" 2>/dev/null || true
  exit 1
}
arbiter_holder="$(cat "$arbiter_marker")"
arbiter_holder_pid="${arbiter_holder#pid=}"; arbiter_holder_pid="${arbiter_holder_pid%% *}"
kill -0 "$arbiter_holder_pid" 2>/dev/null || {
  echo "test-native-gate: arbiter holder pid=$arbiter_holder_pid was not alive" >&2
  exit 1
}

: >"$WORK/events-arbiter-second"
arbiter_second_arrival="$(date +%s)"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$arbiter_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_lock" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-arbiter-second" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-arbiter-second" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-arbiter-second" 2>&1 &
arbiter_second_pid=$!

for _ in $(seq 1 50); do
  grep -Fq "native-gate: waiting for arbiter holder $arbiter_holder" "$WORK/output-arbiter-second" && break
  sleep 0.1
done
grep -Fq "native-gate: waiting for arbiter holder $arbiter_holder" "$WORK/output-arbiter-second" || {
  echo "test-native-gate: the second full gate neither queued nor named its holder" >&2
  kill -TERM "$arbiter_first_pid" "$arbiter_second_pid" 2>/dev/null || true
  exit 1
}
[[ ! -s "$WORK/events-arbiter-second" ]] || {
  echo "test-native-gate: the queued second gate began work before the holder released" >&2
  kill -TERM "$arbiter_first_pid" "$arbiter_second_pid" 2>/dev/null || true
  exit 1
}
wait "$arbiter_first_pid"
wait "$arbiter_second_pid"
[[ ! -e "$arbiter_marker" ]] || {
  echo "test-native-gate: clean arbiter probes left admission state behind" >&2
  exit 1
}
grep -Fq 'native-gate-arbiter capacity=1 holder ' "$WORK/output-arbiter-second" || {
  echo "test-native-gate: the queued gate never acquired the arbiter" >&2
  exit 1
}
arbiter_second_epoch="$(awk '/^native-gate-arbiter capacity=1 holder / { for (i = 1; i <= NF; i++) if ($i ~ /^start_epoch=/) { sub(/^start_epoch=/, "", $i); print $i; exit } }' "$WORK/output-arbiter-second")"
[[ "$arbiter_second_epoch" =~ ^[0-9]+$ && "$arbiter_second_epoch" -gt "$arbiter_second_arrival" ]] || {
  echo "test-native-gate: queued gate published start_epoch=$arbiter_second_epoch from arrival=$arbiter_second_arrival — it measured queue time instead of admitted work" >&2
  exit 1
}

# Mutation: the holder is killed while its test descendant ignores TERM. The
# waiter must acquire before that owned orphan is reaped; otherwise fd 8 leaked
# through a child and the claimed kernel-release guarantee is false.
arbiter_orphan_marker="$WORK/arbiter-orphan-marker"
arbiter_orphan_lock="$WORK/arbiter-orphan.lock"
: >"$WORK/arbiter-orphans"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$arbiter_orphan_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_orphan_lock" \
  AWL_NATIVE_GATE_PROBE_SLEEP=30 \
  AWL_NATIVE_GATE_PROBE_ORPHAN_FILE="$WORK/arbiter-orphans" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-arbiter-orphan-first" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-arbiter-orphan-first" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-arbiter-orphan-first" 2>&1 &
arbiter_orphan_first_pid=$!
for _ in $(seq 1 50); do
  [[ -s "$arbiter_orphan_marker" && -s "$WORK/arbiter-orphans" ]] && break
  sleep 0.1
done
[[ -s "$WORK/arbiter-orphans" ]] || {
  echo "test-native-gate: orphan-holder fixture never created its surviving descendant" >&2
  kill -TERM "$arbiter_orphan_first_pid" 2>/dev/null || true
  exit 1
}
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$arbiter_orphan_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_orphan_lock" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-arbiter-orphan-second" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-arbiter-orphan-second" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-arbiter-orphan-second" 2>&1 &
arbiter_orphan_second_pid=$!
kill -TERM "$arbiter_orphan_first_pid"
set +e
wait "$arbiter_orphan_first_pid"
set -e
for _ in $(seq 1 50); do
  grep -Fq 'native-gate-arbiter capacity=1 holder ' "$WORK/output-arbiter-orphan-second" && break
  sleep 0.1
done
grep -Fq 'native-gate-arbiter capacity=1 holder ' "$WORK/output-arbiter-orphan-second" || {
  echo "test-native-gate: waiter stayed behind a killed holder's surviving descendant — fd 8 leaked" >&2
  kill -TERM "$arbiter_orphan_second_pid" 2>/dev/null || true
  exit 1
}
while read -r orphan; do kill -KILL "$orphan" 2>/dev/null || true; done <"$WORK/arbiter-orphans"
wait "$arbiter_orphan_second_pid"

# This is the failure-shaped mutation: a killed holder leaves stale marker
# text. The kernel releases its flock, so the next holder overwrites the text
# without PID-reuse guessing or deleting a path another gate may be publishing.
printf 'pid=99999999 start_commit=stale start_epoch=1\n' >"$arbiter_marker"
probe arbiter-stale AWL_NATIVE_GATE_MARKER="$arbiter_marker" \
  AWL_NATIVE_GATE_ARBITER_LOCK="$arbiter_lock"
(( probe_status == 0 )) || {
  echo "test-native-gate: stale arbiter recovery probe failed ($probe_status)" >&2
  exit 1
}
require "arbiter stale recovery" "native-gate-arbiter capacity=1 holder"
[[ ! -e "$arbiter_marker" ]] || {
  echo "test-native-gate: a recovered stale holder still blocked later gates" >&2
  exit 1
}

echo "test-native-gate: full gates queue behind a named live holder, then proceed; a stale marker cannot wedge the kernel arbiter"

# A durable worktree's own .orchestrator directory is private. This hostile
# git seam reports a distinct common Git directory and proves the default path
# follows that fleet root instead, without an explicit marker/lock override.
fleet_root="$WORK/fleet-root"
fleet_common="$fleet_root/.git"
fleet_marker="$fleet_root/.orchestrator/native-gate.marker"
mkdir -p "$fleet_common" "$fleet_root/.orchestrator"
: >"$WORK/events-fleet-default"
set +e
env -u AWL_NATIVE_GATE_MARKER -u AWL_NATIVE_GATE_ARBITER_LOCK \
  PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_PROBE_COMMON_GIT_DIR="$fleet_common" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events-fleet-default" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-fleet-default" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-fleet-default" 2>&1
fleet_default_status=$?
set -e
(( fleet_default_status == 0 )) || {
  echo "test-native-gate: fleet-default path probe failed ($fleet_default_status)" >&2
  exit 1
}
[[ ! -e "$fleet_marker" ]] || {
  echo "test-native-gate: fleet-default probe left its common marker behind" >&2
  exit 1
}
grep -Fq 'native-gate-arbiter capacity=1 holder ' "$WORK/output-fleet-default" || {
  echo "test-native-gate: fleet-default probe never entered its derived arbiter" >&2
  exit 1
}

echo "test-native-gate: the default arbiter path follows Git's common directory, not a worktree-local marker"

# ── The in-flight marker: holder identity and signal cleanup ─────────────────
marker="$WORK/marker"
: >"$WORK/events"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$marker" \
  AWL_NATIVE_GATE_PROBE_VITALS_PID_FILE="$WORK/vitals-marker-live" \
  AWL_NATIVE_GATE_PROBE_SLEEP=4 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-marker-live" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-marker-live" 2>&1 &
marker_gate_pid=$!

# Polled rather than a fixed sleep, so this is not flaky on a loaded runner —
# the marker is written before the canary even starts, so it should appear
# almost immediately.
marker_seen=0
for _ in $(seq 1 50); do
  [[ -s "$marker" ]] && { marker_seen=1; break; }
  sleep 0.1
done
(( marker_seen == 1 )) || {
  echo "test-native-gate: no marker appeared while the gate was running" >&2
  kill -TERM "$marker_gate_pid" 2>/dev/null || true
  exit 1
}
marker_line="$(cat "$marker")"
[[ "$marker_line" == pid=*' start_commit='*' start_epoch='* ]] || {
  echo "test-native-gate: marker did not carry pid/start_commit/start_epoch: $marker_line" >&2
  exit 1
}
marker_pid="${marker_line#pid=}"; marker_pid="${marker_pid%% *}"
# Mutation proof: a reader checking the marker mid-run must actually see a
# live process, not merely a file. `kill -0` is the exact check the README
# tells an orchestrator to run.
kill -0 "$marker_pid" 2>/dev/null || {
  echo "test-native-gate: marker named pid=$marker_pid but that pid is not alive — kill -0 would wrongly call this run dead" >&2
  exit 1
}
marker_commit="${marker_line#*start_commit=}"; marker_commit="${marker_commit%% *}"
[[ "$marker_commit" == "$(git -C "$ROOT" rev-parse HEAD)" ]] || {
  echo "test-native-gate: marker start_commit=$marker_commit did not match the actual HEAD" >&2
  exit 1
}

echo "test-native-gate: a reader checking the marker mid-run sees a live pid (kill -0 succeeds) and the correct start commit"

# Captured BEFORE the kill below, while the loop is still alive to be found.
# This is the process the item's live-host diagnosis kept finding at
# ppid=1 with a `sleep` child: the vitals heartbeat, orphaned by a SIGTERM
# that only ever reached the top-level script.
vitals_pid="$(find_vitals_pid "$marker_gate_pid" "$WORK/vitals-marker-live")" || {
  echo "test-native-gate: could not find the running gate's vitals-loop child, so the leak law below would prove nothing" >&2
  kill -TERM "$marker_gate_pid" 2>/dev/null || true
  exit 1
}

# ── The kill path: the case that matters most ────────────────────────────────
# A marker that outlives its process silently wedges every later session's
# advisory check — worse than the defect this item exists to fix. SIGTERM is
# the realistic case: a human or an agent ending a gate deliberately.
kill -TERM "$marker_gate_pid"
set +e
wait "$marker_gate_pid" 2>/dev/null
marker_kill_status=$?
set -e
(( marker_kill_status != 0 )) || {
  echo "test-native-gate: a SIGTERM'd gate reported success ($marker_kill_status)" >&2
  exit 1
}
[[ ! -e "$marker" ]] || {
  echo "test-native-gate: the marker survived SIGTERM — a killed gate would silently wedge a later commit's advisory check" >&2
  exit 1
}

# The marker disappearing proves the EXIT trap ran; it does not by itself
# prove the trap retired vitals_pid too. Give the signal a moment to land —
# gate_vitals_loop's own TERM trap does the actual dying — then check the
# pid directly, the same evidence a live host's `ps -ww` supplied.
sleep 1
kill -0 "$vitals_pid" 2>/dev/null && {
  echo "test-native-gate: the vitals heartbeat (pid=$vitals_pid) survived a SIGTERM to the gate's own pid — orphaned at ppid=1, still holding a sleep child and this script's inherited stdout open" >&2
  kill -TERM "$vitals_pid" 2>/dev/null || true
  exit 1
}

echo "test-native-gate: a SIGTERM to the gate's own pid retires the vitals heartbeat too, not only the marker"

echo "test-native-gate: killing the gate removes the marker — a killed run cannot wedge a later session"

# ── SIGINT reaches the same unconditional trap ────────────────────────────
# Ctrl-C forwarded to a foregrounded gate is the other realistic teardown
# shape, and it is worth proving separately rather than assumed to behave
# like SIGTERM: bash's default disposition differs per signal, and the EXIT
# trap is only guaranteed to fire on the ones actually exercised here.
sigint_marker="$WORK/marker-sigint"
: >"$WORK/events"
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$sigint_marker" \
  AWL_NATIVE_GATE_PROBE_VITALS_PID_FILE="$WORK/vitals-sigint" \
  AWL_NATIVE_GATE_PROBE_SLEEP=4 \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-sigint" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-sigint" 2>&1 &
sigint_gate_pid=$!

sigint_seen=0
for _ in $(seq 1 50); do
  [[ -s "$sigint_marker" ]] && { sigint_seen=1; break; }
  sleep 0.1
done
(( sigint_seen == 1 )) || {
  echo "test-native-gate: no marker appeared for the SIGINT probe" >&2
  kill -TERM "$sigint_gate_pid" 2>/dev/null || true
  exit 1
}
sigint_vitals_pid="$(find_vitals_pid "$sigint_gate_pid" "$WORK/vitals-sigint")" || {
  echo "test-native-gate: could not find the SIGINT probe's vitals-loop child, so this law would prove nothing" >&2
  kill -TERM "$sigint_gate_pid" 2>/dev/null || true
  exit 1
}

kill -INT "$sigint_gate_pid"
set +e
wait "$sigint_gate_pid" 2>/dev/null
set -e
sleep 1
kill -0 "$sigint_vitals_pid" 2>/dev/null && {
  echo "test-native-gate: the vitals heartbeat (pid=$sigint_vitals_pid) survived a SIGINT to the gate's own pid" >&2
  kill -TERM "$sigint_vitals_pid" 2>/dev/null || true
  exit 1
}

echo "test-native-gate: a SIGINT to the gate's own pid retires the vitals heartbeat too"

# ── A clean run leaves nothing behind ─────────────────────────────────────────
marker_clean="$WORK/marker-clean"
: >"$WORK/events"
set +e
PATH="$WORK:$PATH" \
  AWL_NATIVE_GATE_MARKER="$marker_clean" \
  AWL_NATIVE_GATE_PROBE_LOG="$WORK/events" \
  AWL_NATIVE_GATE_PROBE_TEST_BINARY="$WORK/awl-test-bin" \
  AWL_NATIVE_GATE_PROBE_TEST_LIST="$WORK/awl-test-list" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$WORK/free-oracle" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock-marker-clean" \
  "$ROOT/scripts/native-gate.sh" >"$WORK/output-marker-clean" 2>&1
marker_clean_status=$?
set -e
(( marker_clean_status == 0 )) || {
  echo "test-native-gate: the marker-cleanliness probe's own gate run failed ($marker_clean_status)" >&2
  exit 1
}
[[ ! -e "$marker_clean" ]] || {
  echo "test-native-gate: a normal completion left the marker behind" >&2
  exit 1
}

echo "test-native-gate: a normal completion leaves no marker behind"

# ── THE HEALTH ARM: NAMED ON THE RECEIPT, BEFORE THE ARBITER, TWO FAILURE TIERS ──
# The stub is deliberately not the real code-health.sh (see the export at the
# top of this file for why); a stubbed pass must not read like a real one, so
# the gate labels it `mode=override` rather than a fabricated elapsed time.
probe health-pass
(( probe_status == 0 )) || { echo "test-native-gate: health-pass probe failed ($probe_status)" >&2; exit 1; }
require "health pass" "native-gate-health status=ok"
require "health pass" "mode=override"
require "health pass" "native-gate-receipt"
grep -Eq 'health=override:[0-9]+s' "$probe_output" || {
  echo "test-native-gate: a passing health arm did not name itself override:<seconds>s on the receipt" >&2
  exit 1
}

echo "test-native-gate: a passing health arm names itself on the receipt rather than reading as a real pass"

# A raisable mark failure — missing/must-decrease/raised-without-reason — is
# code-health.py's own text always naming the toml paste site. The stub emits
# that exact message shape (copied from check_structural, not invented), so
# this law pins the gate's label against the real distinguishing signal.
probe health-raisable AWL_NATIVE_GATE_PROBE_HEALTH_RAISABLE=1
(( probe_status == 1 )) || {
  echo "test-native-gate: a raisable health failure returned $probe_status, expected gate status 1" >&2
  exit 1
}
require "raisable health failure" "native-gate: code-health failed"
require "raisable health failure" "native-gate: RAISABLE"
refuse "raisable health failure" "native-gate: HARD CEILING"
refuse "raisable health failure" "native-gate-receipt"

echo "test-native-gate: a raisable mark failure is labelled RAISABLE, not HARD CEILING, and issues no receipt"

# The flat 500-line production ceiling never carries a toml paste site — no
# edit to code-health.toml can move it — so the gate's other label applies.
probe health-ceiling AWL_NATIVE_GATE_PROBE_HEALTH_CEILING=1
(( probe_status == 1 )) || {
  echo "test-native-gate: a hard-ceiling health failure returned $probe_status, expected gate status 1" >&2
  exit 1
}
require "hard-ceiling health failure" "native-gate: code-health failed"
require "hard-ceiling health failure" "native-gate: HARD CEILING"
refuse "hard-ceiling health failure" "native-gate: RAISABLE"
refuse "hard-ceiling health failure" "native-gate-receipt"

echo "test-native-gate: a hard-ceiling failure is labelled HARD CEILING, distinctly from a raisable mark, and issues no receipt"

# The dirty-tree refusal is asserted BEFORE health or the arbiter, so a dirty
# tree must never show either one having run at all.
probe health-dirty-tree AWL_NATIVE_GATE_PROBE_DIRTY_STATUS=' M scripts/native-gate.sh'
(( probe_status == 1 )) || {
  echo "test-native-gate: a dirty-tree probe returned $probe_status, expected gate status 1" >&2
  exit 1
}
require "dirty tree refusal" "native-gate: working tree is dirty"
refuse "dirty tree refusal" "native-gate-health"
refuse "dirty tree refusal" "native-gate-arbiter"
refuse "dirty tree refusal" "native-gate-receipt"

echo "test-native-gate: a dirty working tree is refused before health or the arbiter ever run"

# ── The arbiter's held window excludes the health arm, MEASURED ─────────────
# Health runs before gate_arbiter_acquire, so `gate_started_epoch` — the
# reference point every `elapsed_seconds` in this file is relative to — is
# captured strictly after it. The law below is the actual measurement that
# fact implies, not an assumption from reading the source: wall time grows by
# the stub's own sleep (health really ran), the arbiter's own published
# `start_epoch` is delayed by roughly that same sleep (health really precedes
# acquisition), and the suite's `elapsed_seconds` — what a queued sibling
# gate actually waits behind — stays near the fast test phase's own duration,
# not the health sleep added on top of it.
health_measure_launch="$(date +%s)"
probe health-cpu-only AWL_NATIVE_GATE_PROBE_HEALTH_SLEEP=3
(( probe_status == 0 )) || { echo "test-native-gate: health-cpu-only probe failed ($probe_status)" >&2; exit 1; }
(( probe_elapsed >= 2 )) || {
  echo "test-native-gate: a 3s health sleep barely moved the gate's wall time (${probe_elapsed}s) — the arm may not actually be running" >&2
  exit 1
}
health_arbiter_start_epoch="$(awk '/^native-gate-arbiter capacity=1 holder / { for (i = 1; i <= NF; i++) if ($i ~ /^start_epoch=/) { sub(/^start_epoch=/, "", $i); print $i; exit } }' "$probe_output")"
[[ "$health_arbiter_start_epoch" =~ ^[0-9]+$ ]] || {
  echo "test-native-gate: the health-cpu-only probe published no arbiter start_epoch" >&2
  exit 1
}
(( health_arbiter_start_epoch - health_measure_launch >= 2 )) || {
  echo "test-native-gate: the arbiter acquired only $((health_arbiter_start_epoch - health_measure_launch))s after launch against a 3s health sleep — health is not actually running before acquisition" >&2
  exit 1
}
health_suite_elapsed="$(sed -n 's/.*label=mac event=suite-end elapsed_seconds=\([0-9]*\).*/\1/p' "$probe_output" | head -n1)"
[[ -n "$health_suite_elapsed" ]] || {
  echo "test-native-gate: no mac suite-end phase marker in the health-cpu-only probe" >&2
  exit 1
}
(( health_suite_elapsed < 3 )) || {
  echo "test-native-gate: the mac suite reported elapsed_seconds=$health_suite_elapsed against a 3s health sleep — the arbiter-held window grew by the health arm's own time instead of excluding it" >&2
  exit 1
}

echo "test-native-gate: a 3s health arm grows the gate's wall time but not the arbiter-held window — measured, not assumed"
