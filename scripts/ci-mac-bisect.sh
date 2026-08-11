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
    "https://github.com/Frank-P-Lu/awl-editor/actions?query=branch%3A$BRANCH"
}

# Classify a finished run against the fixed oracle.
#
# The marker is the `native full suite` STEP, not the job's conclusion. Two
# facts force that, both measured rather than assumed:
#
#  1. `gh` encodes an unfinished step as `status:"in_progress"`,
#     `conclusion:""` — an EMPTY STRING, never `null`. A `.conclusion==null`
#     test matches nothing and calls every dead run GOOD. Verified against
#     30750073308 and 30706851397, whose step "native full suite" is
#     `in_progress`/`""` in a job that is long over.
#  2. The job's own conclusion does not discriminate. The same freeze reads
#     `failure` when GitHub reaps a runner that stopped answering
#     (30750073308, 30715372469) and `cancelled` when the job ceiling fires
#     first (30706851397, cancelled with step 8 still `in_progress`) —
#     CLAUDE.md's standing note that GitHub reports a timeout as `cancelled`
#     with no separate status. A gate step left `in_progress` in a finished
#     job is one phenomenon under two labels.
#
# So: the job is over and the gate step never completed => BAD.
#
# GATE SECONDS is printed on every GOOD reading and is not decoration. The
# window's own ci.yml capped the mac JOB at 35 minutes while the last green run
# took 26m51s end to end — an 8-minute margin. This probe workflow runs a
# 75-minute ceiling precisely so that "the suite got slower" completes and
# reads GOOD while only a real hang reads BAD. A GOOD probe whose gate took 40
# minutes and one whose gate took 17 tell different stories, and the bisect
# would otherwise throw that away.
cmd_verdict() {
  local run="${1:-}"
  if [ -z "$run" ]; then
    run="$(gh run list --branch "$BRANCH" --workflow bisect-mac --limit 1 \
             --json databaseId -q '.[0].databaseId')"
  fi
  [ -n "$run" ] || die "no run found on $BRANCH"

  local json status conclusion
  json="$(gh run view "$run" --json status,conclusion,headSha,jobs)"
  status="$(printf '%s' "$json" | jq -r '.status')"
  conclusion="$(printf '%s' "$json" | jq -r '.conclusion')"

  printf 'run %s  head=%s  status=%s  conclusion=%s\n' \
    "$run" "$(printf '%s' "$json" | jq -r '.headSha[0:8]')" "$status" "$conclusion"
  printf '%s' "$json" | jq -r '.jobs[] | "  job \(.databaseId) \(.name) \(.conclusion) \(.startedAt)..\(.completedAt)"'
  printf '%s' "$json" | jq -r '.jobs[].steps[] | "    step \(.number) \(.name): \(.status)/\(.conclusion)"'

  if [ "$status" != "completed" ]; then printf 'VERDICT: RUNNING\n'; return; fi

  local jobid gate_status gate_conc job_secs gate_secs loghttp
  jobid="$(printf '%s' "$json" | jq -r '.jobs[] | select(.name|startswith("mac (")) | .databaseId' | head -1)"
  gate_status="$(printf '%s' "$json" | jq -r '[.jobs[].steps[] | select(.name=="native full suite") | .status] | first // "absent"')"
  gate_conc="$(printf '%s' "$json" | jq -r '[.jobs[].steps[] | select(.name=="native full suite") | .conclusion] | first // ""')"
  job_secs="$(printf '%s' "$json" | jq -r '.jobs[] | select(.name|startswith("mac (")) | (( .completedAt|fromdateiso8601 ) - ( .startedAt|fromdateiso8601 ))' 2>/dev/null || echo '?')"
  gate_secs="$(gh api "repos/Frank-P-Lu/awl-editor/actions/jobs/$jobid" \
                 -q '[.steps[]|select(.name=="native full suite")][0] | if .completed_at then ((.completed_at|fromdateiso8601)-(.started_at|fromdateiso8601)) else "unfinished" end' \
                 2>/dev/null || echo '?')"
  loghttp="$(gh api "repos/Frank-P-Lu/awl-editor/actions/jobs/$jobid/logs" \
               --include --silent 2>&1 | head -1 || true)"

  printf 'gate step: %s/%s | gate seconds: %s | job seconds: %s | job log: %s\n' \
    "$gate_status" "$gate_conc" "$gate_secs" "$job_secs" "$loghttp"

  # THREE terminal shapes, not two. The hang reaches the API three different
  # ways depending on who kills it first, and only the third one looks like a
  # normal ending:
  #
  #   runner reaped mid-step   status=in_progress conclusion=""        log 404
  #     (30750073308, probes 1 and 2 — the VM stopped answering)
  #   job ceiling fires        status=completed   conclusion=cancelled log 200
  #     (probe 3, run 30756807172 — the runner SURVIVED, so post steps ran,
  #      the cache saved, and a log exists)
  #   gate exits on its own    status=completed   conclusion=success|failure
  #     (the only GOOD shape)
  #
  # The middle one is why `status != "completed"` is not sufficient: a step
  # GitHub kills at the ceiling is `completed`, and reading only status scores
  # a 64-minute hang as GOOD. Same class of bug as the `conclusion:""` trap —
  # an unfinished step wearing a finished step's field. So the test is on the
  # CONCLUSION, allow-listed: only success and failure mean the suite ran to
  # an answer. Anything else is the suite not finishing.
  case "$gate_status/$gate_conc" in
    absent/*)
      printf 'VERDICT: INVALID (the gate step never ran — re-run)\n' ;;
    completed/success|completed/failure)
      printf 'VERDICT: GOOD (gate concluded %s in %ss)\n' "$gate_conc" "$gate_secs" ;;
    completed/cancelled|completed/timed_out)
      printf 'VERDICT: BAD (gate step %s after %ss — killed by a ceiling, never finished)\n' \
        "$gate_conc" "$gate_secs" ;;
    completed/*)
      printf 'VERDICT: INVALID (unrecognised gate conclusion %s — score by hand)\n' "$gate_conc" ;;
    *)
      printf 'VERDICT: BAD (job over, gate step still %s — the runner was reaped mid-step)\n' \
        "$gate_status" ;;
  esac
}

# Next probe, from the boundaries established so far. The window is a DAG, not
# a line — main was fast-forwarded onto tmp/simd-search, so 46 commits sit on
# only 6 first-parent steps, with item 194's render work arriving on the SECOND
# parent of the merge 97cc62f0. `git rev-list --bisect` is the thing that knows
# how to halve that; hand-picking along --first-parent would skip the 40 commits
# where the render work actually lives.
#
#   scripts/ci-mac-bisect.sh next BAD_REF GOOD_REF [GOOD_REF...]
#
# e.g. after c5b8399e reads GOOD:
#   scripts/ci-mac-bisect.sh next edc89757 7bca59d6 c5b8399e
cmd_next() {
  local bad="${1:-}"; shift || true
  [ -n "$bad" ] || die "next needs a BAD ref and at least one GOOD ref"
  [ "$#" -gt 0 ] || die "next needs at least one GOOD ref"

  local excludes=()
  local g
  for g in "$@"; do excludes+=("^$g"); done

  local remaining
  remaining="$(git rev-list --count "$bad" "${excludes[@]}")"
  if [ "$remaining" -le 1 ]; then
    printf 'converged: first bad commit is %s\n  %s\n' \
      "$(git rev-parse --short=8 "$bad")" "$(git log -1 --format=%s "$bad")"
    return
  fi

  git rev-list --bisect-vars "$bad" "${excludes[@]}" | sed 's/^/  /'
  local rev
  rev="$(git rev-list --bisect "$bad" "${excludes[@]}")"
  printf 'candidates remaining: %s\nnext probe: %s  %s\n' \
    "$remaining" "$(git rev-parse --short=8 "$rev")" "$(git log -1 --format=%s "$rev")"
  printf 'run: scripts/ci-mac-bisect.sh probe %s\n' "$(git rev-parse --short=8 "$rev")"
}

cmd_cleanup() {
  git push origin --delete "$BRANCH" 2>/dev/null || true
  git branch -D "$BRANCH" 2>/dev/null || true
  printf 'ci-mac-bisect: %s deleted locally and on origin\n' "$BRANCH"
}

case "${1:-}" in
  probe)   shift; cmd_probe "$@" ;;
  verdict) shift; cmd_verdict "$@" ;;
  next)    shift; cmd_next "$@" ;;
  cleanup) shift; cmd_cleanup "$@" ;;
  *) die "usage: $0 {probe <commit-ish>|verdict [run-id]|next <bad> <good...>|cleanup}" ;;
esac
