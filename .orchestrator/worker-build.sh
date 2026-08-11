#!/usr/bin/env bash
# The orchestration-owned build launch seam for concurrently dispatched workers.
# Root merge-train gates, CI, and a developer's lone build deliberately do not
# use this wrapper and retain Cargo's hardware-adaptive default.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if (( $# == 0 )); then
  echo "usage: .orchestrator/worker-build.sh <build-or-gate-command> [args…]" >&2
  exit 2
fi

readonly WORKER_CARGO_JOBS=2
# CARGO_BUILD_JOBS bounds COMPILATION parallelism only. native-gate.sh runs
# both keymap conventions concurrently, and each `cargo test` defaults its
# harness thread count to the core count — so four workers at the gate phase,
# each left to that default, schedule 4 workers x 2 conventions x (core count)
# runnable test threads in aggregate, independent of the build budget above.
# One thread per worker's convention keeps four workers' aggregate test
# threads (4 x 2 x 1 = 8) matching the build budget's own aggregate (4 x 2 =
# 8) instead of several times the host's core count.
readonly WORKER_TEST_THREADS=1
export CARGO_BUILD_JOBS="$WORKER_CARGO_JOBS"
export RUST_TEST_THREADS="$WORKER_TEST_THREADS"

# A launch seam owes its callers a usable toolchain, not just a CPU budget.
# `cargo fmt --all` reaches rustup's shim before the real binary under some
# ambient PATHs and dies on `unexpected argument '--all'` — a rustup usage
# error wearing a Cargo command's clothes, which reads as a broken gate rather
# than a broken PATH. Prepend the ACTIVE toolchain's own bin so every command
# launched here sees the same binaries, and derive it from rustup rather than
# naming a host triple: this seam runs on Apple Silicon and on Linux CI images.
# Probing `cargo fmt --version` first keeps the fix inert wherever the ambient
# PATH already works, so a developer's own toolchain choice is never overridden.
if ! cargo fmt --version >/dev/null 2>&1; then
  if toolchain="$(rustup show active-toolchain 2>/dev/null | head -n1 | cut -d' ' -f1)" \
     && [ -n "$toolchain" ] \
     && [ -d "${RUSTUP_HOME:-$HOME/.rustup}/toolchains/$toolchain/bin" ]; then
    PATH="${RUSTUP_HOME:-$HOME/.rustup}/toolchains/$toolchain/bin:$PATH"
    export PATH
    printf 'orchestrator-worker-budget toolchain_bin=%s\n' "$toolchain"
  fi
fi
AWL_DISK_PREFLIGHT_CALLER=worker-build "$ROOT/.orchestrator/disk-preflight.sh"
printf 'orchestrator-worker-budget cargo_jobs=%s test_threads=%s command=' \
  "$WORKER_CARGO_JOBS" "$WORKER_TEST_THREADS"
printf '%q ' "$@"
printf '\n'
exec "$@"
