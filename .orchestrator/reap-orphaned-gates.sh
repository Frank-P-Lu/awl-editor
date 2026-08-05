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
# The root checkout's own gate (the merge train's) needs a DIFFERENT
# discriminator than a worktree's: the repo root always exists, is always a
# known worktree, and its branch is never "already merged into main" in the
# way a lane's is, so none of the three worktree signals below can ever fire
# there. Item 270's marker (.orchestrator/native-gate.marker) is what closes
# that gap — it names the pid, start commit, and start time of whichever
# root-tree gate is actually live, and is removed on every one of that
# script's own trappable exits. So for a root-tree candidate: marker absent,
# or present but naming a pid that is not alive (a killed run whose exit
# outran the trap, or a stale leftover), means no root gate is live and any
# native-gate.sh-tagged process found there — including a leaked vitals-loop
# child, which is what "verified by hand" against ppid=1 orphans holding a
# `sleep` child meant in practice — is an orphan by construction, exactly
# like a worktree one. A marker naming a live pid is the one case this
# script still declines to touch, for the same reason as before: it cannot
# tell "the train's gate, running long" from "an orphan" once one IS
# genuinely in flight.
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

# Overridable for tests, same reasoning as AWL_REAP_GATES_ROOT: a test must
# never let this read the real repo's live marker.
gate_marker="${AWL_REAP_GATES_MARKER:-$ROOT/.orchestrator/native-gate.marker}"

# Item 270's marker line is `pid=%s start_commit=%s start_epoch=%s`. Prints
# nothing (and fails) when the marker is absent or unparseable — both read
# as "no evidence of a live root gate" to the caller below, same as a marker
# naming a pid that `kill -0` says is dead.
marker_live_pid() {
  local marker="$1" line pid
  [[ -f "$marker" ]] || return 1
  line="$(<"$marker")"
  [[ "$line" == pid=* ]] || return 1
  pid="${line#pid=}"; pid="${pid%% *}"
  [[ -n "$pid" ]] || return 1
  kill -0 "$pid" 2>/dev/null || return 1
  printf '%s\n' "$pid"
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

  reason=""
  if [[ -n "$main_worktree" && "$cwd" == "$main_worktree" ]]; then
    # The repo root always exists, is always a known worktree, and its
    # branch is never "merged into main" the way a lane's is — none of the
    # three worktree signals below can fire here. Item 270's marker is the
    # only evidence available: absent or stale (naming a dead pid) means no
    # root gate is live, so this candidate — main script or a leaked vitals-
    # loop child, both tagged native-gate.sh — is an orphan by construction.
    if marker_pid="$(marker_live_pid "$gate_marker")"; then
      : # a live root gate is named; this candidate is left alone
    else
      reason="root gate marker absent or stale (item 270) — no live root gate"
    fi
  elif [[ ! -d "$cwd" ]]; then
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
