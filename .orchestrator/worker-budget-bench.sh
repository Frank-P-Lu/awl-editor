#!/usr/bin/env bash
# Measure four truly clean disposable worktree builds. This is deliberately an
# orchestration tool: it must never run inside a product gate or change Cargo's
# global configuration. It records the configured Cargo cap, aggregate CPU
# samples, wall time, and a scheduler-heartbeat proxy; a human still judges the
# desktop's actual responsiveness while it runs.
set -euo pipefail

if (( $# != 1 )) || [[ "$1" != "baseline" && "$1" != "worker" ]]; then
  echo "usage: .orchestrator/worker-budget-bench.sh baseline|worker" >&2
  exit 2
fi

mode="$1"
root="$(git rev-parse --show-toplevel)"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/awl-169-worker-budget.XXXXXX")"
report="$scratch/report.tsv"
declare -a trees=() pids=()

cleanup() {
  local tree
  for tree in "${trees[@]:-}"; do
    [[ "$tree" == "$scratch"/* ]] || continue
    git -C "$root" worktree remove --force "$tree" >/dev/null 2>&1 || true
  done
  [[ "$scratch" == "${TMPDIR:-/tmp}/awl-169-worker-budget."* ]] && rm -rf "$scratch"
}
trap cleanup EXIT

for n in 1 2 3 4; do
  tree="$scratch/worktree-$n"
  git -C "$root" worktree add --detach --quiet "$tree" HEAD
  trees+=("$tree")
done

if [[ "$mode" == worker ]]; then
  jobs=2
  command=("$root/.orchestrator/worker-build.sh" cargo build --locked)
else
  jobs=auto
  command=(cargo build --locked)
fi

printf 'mode\tconfigured_cargo_jobs\tworktrees\n%s\t%s\t4\n' "$mode" "$jobs" >"$report"
start="$(date +%s)"
for tree in "${trees[@]}"; do
  (
    cd "$tree"
    /usr/bin/time -lp "${command[@]}" >"$tree/build.log" 2>&1
  ) &
  pids+=("$!")
done

# Sample all descendants of the four build shells. macOS `ps` reports a
# per-process %CPU; summing the tree produces an honest aggregate witness,
# while the pulse records whether the orchestrator itself can still schedule.
printf 'elapsed_s\taggregate_cpu_percent\tpulse_ms\n' >"$scratch/samples.tsv"
while :; do
  live=0
  for pid in "${pids[@]}"; do kill -0 "$pid" 2>/dev/null && ((live += 1)); done
  (( live > 0 )) || break
  tick="$(date +%s)"
  cpu="$(ps -axo pid=,ppid=,%cpu= | awk -v roots="${pids[*]}" '
    BEGIN { split(roots, r, " "); for (i in r) keep[r[i]]=1 }
    { pid=$1; parent[pid]=$2; cpu[pid]=$3 }
    END { changed=1; while (changed) { changed=0; for (pid in parent) if (!keep[pid] && keep[parent[pid]]) { keep[pid]=1; changed=1 } }
      for (pid in keep) sum+=cpu[pid]; printf "%.1f", sum }')"
  pulse_ms="$(python3 -c 'import time; a=time.monotonic_ns(); b=time.monotonic_ns(); print((b-a)//1_000_000)')"
  printf '%s\t%s\t%s\n' "$((tick - start))" "$cpu" "$pulse_ms" >>"$scratch/samples.tsv"
  sleep 1
done

status=0
for pid in "${pids[@]}"; do wait "$pid" || status=1; done
end="$(date +%s)"
{
  printf 'wall_seconds\t%s\n' "$((end - start))"
  printf 'aggregate_cpu_peak_percent\t%s\n' "$(awk 'NR > 1 && $2 > max { max=$2 } END { print max + 0 }' "$scratch/samples.tsv")"
  printf 'aggregate_cpu_mean_percent\t%s\n' "$(awk 'NR > 1 { sum += $2; n += 1 } END { print n ? sum / n : 0 }' "$scratch/samples.tsv")"
  printf 'scheduler_pulse_max_ms\t%s\n' "$(awk 'NR > 1 && $3 > max { max=$3 } END { print max + 0 }' "$scratch/samples.tsv")"
  printf 'configured_aggregate_cargo_jobs\t%s\n' "$([[ "$jobs" == auto ]] && printf auto || printf 8)"
  printf 'human_desktop_responsiveness\tconfirm live during run (not harness-verifiable)\n'
} >>"$report"
cat "$report"
exit "$status"
