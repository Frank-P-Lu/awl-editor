#!/usr/bin/env bash
# Exercises the orchestration wrapper without compiling or touching a real
# worktree. The probe is a disposable temp file and verifies a child command —
# including gate scripts — receives the one effective Cargo budget.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-worker-build-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

probe="$WORK/probe.sh"
free_oracle="$WORK/free-oracle.sh"
never_sweep="$WORK/never-sweep.sh"
cat >"$probe" <<'EOF'
#!/usr/bin/env bash
printf 'child-cargo-jobs=%s\n' "${CARGO_BUILD_JOBS:-unset}"
EOF
cat >"$free_oracle" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' $((40 * 1024 * 1024 * 1024))
EOF
cat >"$never_sweep" <<'EOF'
#!/usr/bin/env bash
echo "test-worker-build: healthy preflight attempted a sweep" >&2
exit 1
EOF
chmod +x "$probe"
chmod +x "$free_oracle" "$never_sweep"

output="$(CARGO_BUILD_JOBS=99 \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$free_oracle" \
  AWL_DISK_PREFLIGHT_SWEEP_COMMAND="$never_sweep" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/disk-lock" \
  "$ROOT/.orchestrator/worker-build.sh" "$probe")"
[[ "$output" == *"orchestrator-worker-budget cargo_jobs=2 command="* ]] \
  || { echo "test-worker-build: wrapper did not issue its budget receipt" >&2; exit 1; }
[[ "$output" == *"child-cargo-jobs=2"* ]] \
  || { echo "test-worker-build: child did not inherit CARGO_BUILD_JOBS=2" >&2; exit 1; }

# The canonical gate is intentionally not a worker-budget owner. Its argument
# rejection is enough to execute only its pre-Cargo path while proving an
# isolated caller's unset environment is left untouched.
unset CARGO_BUILD_JOBS
if [[ -n "${CARGO_BUILD_JOBS:-}" ]]; then
  echo "test-worker-build: isolated root gate caller retained a worker cap" >&2
  exit 1
fi
if ! AWL_DISK_PREFLIGHT_FREE_BYTES_COMMAND="$free_oracle" \
  AWL_DISK_PREFLIGHT_TEST_MODE=1 \
  AWL_DISK_PREFLIGHT_SWEEP_COMMAND="$never_sweep" \
  AWL_DISK_PREFLIGHT_LOCK_DIR="$WORK/native-disk-lock" \
  "$ROOT/scripts/native-gate.sh" --bin >/dev/null 2>"$WORK/native-gate.err"; then
  if ! grep -Fq 'target selection and test-name arguments are forbidden' "$WORK/native-gate.err"; then
    echo "test-worker-build: canonical native gate did not reject its probe argument" >&2
    exit 1
  fi
else
  echo "test-worker-build: canonical native gate unexpectedly accepted --bin" >&2
  exit 1
fi

echo "test-worker-build: wrapper caps children at 2; isolated native gate caller stays uncapped"

owners="$(rg --hidden -l 'CARGO_BUILD_JOBS' "$ROOT" \
  -g '!target' -g '!Cargo.lock' -g '!*.json' -g '!queue.md' | sort)"
expected="$ROOT/.orchestrator/README.md
$ROOT/.orchestrator/worker-build.sh
$ROOT/scripts/test-worker-build.sh"
[[ "$owners" == "$expected" ]] || {
  echo "test-worker-build: competing CARGO_BUILD_JOBS owner(s): $owners" >&2
  exit 1
}

echo "test-worker-build: no repository script or Cargo config competes for the budget"
