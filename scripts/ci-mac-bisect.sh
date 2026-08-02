#!/usr/bin/env bash
# Bisect the hosted-macOS `mac (build + test)` VM freeze.
#
# Three rounds of in-VM instrumentation are proven futile: the runner agent
# itself dies, so nothing inside the VM can report. This script stops observing
# and searches for the commit instead.
#
# It takes a window commit, grafts ONE byte-identical mac-only workflow on top
# of it (via plumbing — no checkout, so the caller's worktree never moves), and
# force-pushes it to a single throwaway branch. Every probe therefore differs
# from every other probe in exactly one thing: the tree under test.
#
# Why the probe workflow is a NEW file rather than an edit to ci.yml:
#   * ci.yml is byte-identical across the whole 7bca59d6..edc89757 window
#     (verified: `git diff 7bca59d6 edc89757 -- .github/` is empty), and so are
#     scripts/native-gate.sh, scripts/code-health.sh and install-sccache.sh. A
#     separate file keeps that true and keeps the diff to one added path.
#   * ci.yml only fires on push-to-main / pull_request / workflow_dispatch, so
#     it stays silent on a `ci-probe/**` branch. Only this workflow runs, and
#     only its `mac` job — the linux, web and mac-live-probe jobs would cost
#     minutes and buy the bisect nothing.
#
# The probe job deliberately reproduces the LAST GREEN configuration
# (run 30686231377, mac step 8 = 16m54s) rather than today's ci.yml:
#   * no `Runner death clock` / AWL_NATIVE_GATE_DEADLINE_EPOCH,
#   * no AWL_NATIVE_GATE_BUDGET_SECONDS,
#   * no step-level `timeout-minutes: 40`.
# All three are established NOT to fire on a dying runner (runs 30746762499 and
# 30750073308), so they add no signal — and a budget that DID fire would abort
# a slow-but-healthy job and forge a BAD reading. The `Rust code health` step is
# dropped too: scripts/code-health.toml is one of the few files that DOES move
# inside the window, and a clippy failure would end the job before the suite
# ever ran, i.e. an invalid probe wearing a GOOD costume.
#
# THE ORACLE (fixed before the first probe, not adjusted afterwards):
#   BAD     — the mac job dies with a `null` step conclusion and no job log.
#   GOOD    — the mac job completes: pass, or an ordinary test/build failure.
#   INVALID — cancelled, or the `native full suite` step never started.
#             Re-run; it is neither reading.
#
# Usage:
#   scripts/ci-mac-bisect.sh probe <commit-ish>   # push a probe, print run URL
#   scripts/ci-mac-bisect.sh verdict [run-id]     # classify a finished run
#   scripts/ci-mac-bisect.sh cleanup              # delete the throwaway branch
set -euo pipefail

BRANCH="ci-probe/mac"
WORKFLOW=".github/workflows/bisect-mac.yml"

die() { printf 'ci-mac-bisect: %s\n' "$*" >&2; exit 1; }

workflow_body() {
  cat <<'YAML'
name: bisect-mac

# Throwaway workflow for the hosted-macOS freeze bisect. Fires only on the
# probe branch, so it can never run on main. Delete the branch and this file
# goes with it.
on:
  push:
    branches: ['ci-probe/**']

concurrency:
  group: bisect-mac-${{ github.ref }}
  cancel-in-progress: true

jobs:
  mac:
    name: mac (build + test)
    runs-on: macos-latest
    # ci.yml's own ceiling. Not a bound we expect to fire: the runner agent
    # enforces it and the runner agent is what dies. The four observed losses
    # came at job-minute 53, 55, 56 and 62, reaped server-side.
    timeout-minutes: 75
    steps:
      - uses: actions/checkout@v7

      - uses: dtolnay/rust-toolchain@stable

      # Same branch for every probe, so probe N+1 restores probe N's cache and
      # a cold compile is paid once rather than six times. Cache state changes
      # how long the gate takes; it cannot turn a VM freeze on or off.
      - uses: Swatinem/rust-cache@v2
        with:
          cache-on-failure: true

      - name: Install sccache
        run: scripts/install-sccache.sh

      - name: cargo build
        run: cargo build

      - name: native full suite
        run: scripts/native-gate.sh
YAML
}

cmd_probe() {
  local target="${1:-}"
  [ -n "$target" ] || die "probe needs a commit-ish"

  local sha
  sha="$(git rev-parse --verify "${target}^{commit}")" || die "no such commit: $target"

  git merge-base --is-ancestor "$sha" edc89757 2>/dev/null \
    || printf 'ci-mac-bisect: warning: %s is not inside the failure window\n' "$target" >&2

  local blob tmpidx tree probe
  blob="$(workflow_body | git hash-object -w --stdin)"
  tmpidx="$(mktemp -t ci-mac-bisect-index.XXXXXX)"
  trap 'rm -f "$tmpidx"' RETURN

  GIT_INDEX_FILE="$tmpidx" git read-tree "$sha"
  GIT_INDEX_FILE="$tmpidx" git update-index --add \
    --cacheinfo "100644,$blob,$WORKFLOW"
  tree="$(GIT_INDEX_FILE="$tmpidx" git write-tree)"

  probe="$(git commit-tree "$tree" -p "$sha" -m "ci probe: mac-only bisect workflow over $(git rev-parse --short=8 "$sha")

$(git log -1 --format=%s "$sha")

Not for main. Adds only $WORKFLOW; the tree under test is untouched.")"

  git branch -f "$BRANCH" "$probe"
  git push --force origin "$BRANCH:refs/heads/$BRANCH"

  printf '\nprobe pushed: %s (%s)\n  %s\n' \
    "$(git rev-parse --short=8 "$sha")" "$(git log -1 --format=%s "$sha")" \
    "https://github.com/Frank-P-Lu/awl-next/actions?query=branch%3A$BRANCH"
}

# Classify a finished run against the fixed oracle. Prints one of
# BAD / GOOD / INVALID / RUNNING plus the evidence it used.
cmd_verdict() {
  local run="${1:-}"
  if [ -z "$run" ]; then
    run="$(gh run list --branch "$BRANCH" --workflow bisect-mac --limit 1 \
             --json databaseId -q '.[0].databaseId')"
  fi
  [ -n "$run" ] || die "no run found on $BRANCH"

  local json status conclusion
  json="$(gh run view "$run" --json status,conclusion,headSha,createdAt,updatedAt,jobs)"
  status="$(printf '%s' "$json" | jq -r '.status')"
  conclusion="$(printf '%s' "$json" | jq -r '.conclusion')"

  printf 'run %s  head=%s  status=%s  conclusion=%s\n' \
    "$run" "$(printf '%s' "$json" | jq -r '.headSha[0:8]')" "$status" "$conclusion"
  printf '%s' "$json" | jq -r '.jobs[] | "  job \(.databaseId) \(.name) \(.status)/\(.conclusion) \(.startedAt)..\(.completedAt)"'
  printf '%s' "$json" | jq -r '.jobs[].steps[] | "    step \(.number) \(.name): \(.status)/\(.conclusion)"'

  if [ "$status" != "completed" ]; then printf 'VERDICT: RUNNING\n'; return; fi

  local jobid nullsteps gatestarted loghttp
  jobid="$(printf '%s' "$json" | jq -r '.jobs[] | select(.name|test("mac")) | .databaseId' | head -1)"
  nullsteps="$(printf '%s' "$json" | jq -r '[.jobs[].steps[] | select(.conclusion==null)] | length')"
  gatestarted="$(printf '%s' "$json" | jq -r '[.jobs[].steps[] | select(.name=="native full suite")] | length')"
  loghttp="$(gh api "repos/Frank-P-Lu/awl-next/actions/jobs/$jobid/logs" \
               --include --silent 2>&1 | head -1 || true)"

  printf 'null-conclusion steps: %s | gate step present: %s | job log: %s\n' \
    "$nullsteps" "$gatestarted" "$loghttp"

  if [ "$conclusion" = "cancelled" ]; then
    printf 'VERDICT: INVALID (cancelled is not a reading — re-run)\n'
  elif [ "$nullsteps" -gt 0 ]; then
    printf 'VERDICT: BAD (step conclusion null — the VM froze)\n'
  elif [ "$gatestarted" -eq 0 ]; then
    printf 'VERDICT: INVALID (the gate step never ran)\n'
  else
    printf 'VERDICT: GOOD (the job completed: %s)\n' "$conclusion"
  fi
}

cmd_cleanup() {
  git push origin --delete "$BRANCH" 2>/dev/null || true
  git branch -D "$BRANCH" 2>/dev/null || true
  printf 'ci-mac-bisect: %s deleted locally and on origin\n' "$BRANCH"
}

case "${1:-}" in
  probe)   shift; cmd_probe "$@" ;;
  verdict) shift; cmd_verdict "$@" ;;
  cleanup) shift; cmd_cleanup "$@" ;;
  *) die "usage: $0 {probe <commit-ish>|verdict [run-id]|cleanup}" ;;
esac
