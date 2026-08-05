#!/usr/bin/env bash
# Finds native-gate.sh runs whose lane is gone — the worktree that launched
# them was removed, or was never registered, or its branch is already merged
# into main — and, with --kill, retires them and every descendant process.
#
# native-gate.sh's own wall clock exceeds this tool's 600s cap in this repo,
# so every dispatched worker's gate runs auto-backgrounded and detached from
# the session that launched it. If that session ends (or the orchestrator
# merges the item and moves on) before the gate finishes on its own, nothing
# else ever stops it: it is reparented to init and keeps consuming cores.
# This script is the read-mostly detector for that state; it is deliberately
# NOT wired into any automatic teardown, because killing a process this
# script did not launch is only safe once the evidence below is unambiguous.
#
# Usage:
#   .orchestrator/reap-orphaned-gates.sh          # report only
#   .orchestrator/reap-orphaned-gates.sh --kill    # report and retire
#
# The root checkout's own gate (the merge train's) is never a candidate here,
# on purpose: it is the one gate allowed to run unbounded and outside worker
# policy, and this script has no way to tell "the train's gate, running long"
# from "an orphan" by evidence alone.
set -euo pipefail

# Overridable for tests: a test must never let `ps -A` sweep in this HOST's
# real, unrelated native-gate.sh processes — this repo runs concurrent worker
# sessions for real, and a fixture that scanned the live process table could
# misclassify (and with --kill, retire) somebody else's live lane. Test mode
# substitutes its own bounded process listing and points the git root at a
# disposable repo instead.
ROOT="${AWL_REAP_GATES_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
KILL=0
if [[ "${1:-}" == "--kill" ]]; then
  KILL=1
elif [[ $# -gt 0 ]]; then
  echo "usage: .orchestrator/reap-orphaned-gates.sh [--kill]" >&2
  exit 2
fi

# ps -o command= truncates before a script's own trailing arguments on macOS
# — CLAUDE.md's own documented tripwire, which fired once already during this
# item's diagnosis — so this greps the untruncated -ww form. `mapfile` is
# bash4+; macOS still ships bash 3.2 (sweep.sh notes the same constraint), so
# this reads the loop the portable way instead.
ps_snapshot() {
  if [[ -n "${AWL_REAP_GATES_PS_COMMAND:-}" ]]; then
    eval "$AWL_REAP_GATES_PS_COMMAND"
  else
    ps -ww -A -o pid=,command= 2>/dev/null
  fi
}

gate_lines=()
while IFS= read -r line; do
  [[ -n "$line" ]] && gate_lines+=("$line")
done < <(ps_snapshot | grep -F 'native-gate.sh' | grep -v grep || true)

if (( ${#gate_lines[@]} == 0 )); then
  echo "reap-orphaned-gates: no native-gate.sh process is running"
  exit 0
fi

known_worktrees=()
while IFS= read -r line; do
  case "$line" in
    "worktree "*) known_worktrees+=("${line#worktree }") ;;
  esac
done < <(git -C "$ROOT" worktree list --porcelain)
# `git worktree list` always names the main (non-linked) checkout first,
# regardless of which worktree this script itself runs from — so this is the
# repo's actual root, not merely "wherever I happen to be running." Only the
# main checkout's gate is the merge train's; every other entry is a lane.
main_worktree="${known_worktrees[0]:-}"

is_known_worktree() {
  local candidate="$1" known
  for known in "${known_worktrees[@]}"; do
    [[ "$known" == "$candidate" ]] && return 0
  done
  return 1
}

process_cwd() {
  lsof -p "$1" 2>/dev/null | awk '$4 == "cwd" { print $NF; exit }'
}

# Every PID whose ancestry runs through $1, $1 included. Cargo's own children
# (rustc, the compiled test binaries) are what actually hold the cores, and a
# process-group signal would miss them: native-gate.sh deliberately launches
# each convention with `set -m` as its OWN group leader, so the top-level
# script's group does not cover them. Walking real ancestry does.
collect_descendants() {
  local pid="$1" child
  printf '%s\n' "$pid"
  for child in $(pgrep -P "$pid" 2>/dev/null || true); do
    collect_descendants "$child"
  done
}

orphan_pids=()
orphan_reasons=()
for line in "${gate_lines[@]}"; do
  # `ps -o pid=` right-pads with leading spaces; `read` word-splits on IFS and
  # discards them, which a substring strip on the raw line does not.
  read -r pid _ <<<"$line"
  [[ -n "$pid" ]] || continue
  cwd="$(process_cwd "$pid")"
  [[ -n "$cwd" ]] || continue
  [[ -n "$main_worktree" && "$cwd" == "$main_worktree" ]] && continue

  reason=""
  if [[ ! -d "$cwd" ]]; then
    reason="worktree directory no longer exists"
  elif ! is_known_worktree "$cwd"; then
    reason="worktree no longer registered with git"
  else
    branch="$(git -C "$cwd" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")"
    if [[ -n "$branch" && "$branch" != "HEAD" ]] \
      && git -C "$ROOT" merge-base --is-ancestor "$branch" main 2>/dev/null; then
      reason="branch $branch is already merged into main"
    fi
  fi

  if [[ -n "$reason" ]]; then
    orphan_pids+=("$pid")
    orphan_reasons+=("$reason")
    echo "reap-orphaned-gates: pid=$pid cwd=$cwd — $reason"
  fi
done

if (( ${#orphan_pids[@]} == 0 )); then
  echo "reap-orphaned-gates: no orphan found among ${#gate_lines[@]} running native-gate.sh process(es)"
  exit 0
fi

if (( KILL == 0 )); then
  echo "reap-orphaned-gates: ${#orphan_pids[@]} orphan(s) found; rerun with --kill to retire them and their descendants"
  exit 0
fi

all_targets=()
for pid in "${orphan_pids[@]}"; do
  while IFS= read -r target; do
    all_targets+=("$target")
  done < <(collect_descendants "$pid")
done

for pid in "${all_targets[@]}"; do
  kill -TERM "$pid" 2>/dev/null || true
done
sleep 5
survivors=0
for pid in "${all_targets[@]}"; do
  if kill -0 "$pid" 2>/dev/null; then
    kill -KILL "$pid" 2>/dev/null || true
    survivors=$((survivors + 1))
  fi
done
echo "reap-orphaned-gates: retired ${#orphan_pids[@]} orphaned gate(s), ${#all_targets[@]} process(es) total ($survivors needed SIGKILL)"
