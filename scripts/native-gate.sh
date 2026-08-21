#!/usr/bin/env bash
# The one native full-suite gate. A receipt from this script, on the commit it
# names, is the only evidence that both supported conventions ran every native
# Cargo test target. `cargo test --bin awl` is binary unit tests, not this gate.
set -euo pipefail

if (( $# != 0 )); then
  echo "native-gate: target selection and test-name arguments are forbidden; run targeted tests directly" >&2
  exit 2
fi

gate_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# ── The full-gate arbiter ─────────────────────────────────────────────────────
# Full gates contend for the same GPU and test processes; targeted tests do not
# enter here. An inherited kernel flock is the admission door, while the marker
# is the readable holder identity an arriving gate reports. Capacity is deliberately
# one: six shards shortened a unit wave from 231 s to 60 s, but no measurement
# establishes that two full-width gates are safe together.
# Worktree-local paths would give every lane a private queue. The common Git
# directory names the fleet, so every linked worktree resolves the main checkout
# and shares this one readable holder marker and kernel lock.
gate_common_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
gate_fleet_root="$(cd "$gate_common_dir/.." && pwd -P)"
gate_marker="${AWL_NATIVE_GATE_MARKER:-$gate_fleet_root/.orchestrator/native-gate.marker}"
gate_arbiter_lock="${AWL_NATIVE_GATE_ARBITER_LOCK:-${gate_marker}.lock}"

gate_holder_field() {
  local field="$1" holder="${2:-}"
  holder="${holder#*$field=}"
  printf '%s\n' "${holder%% *}"
}

gate_arbiter_acquire() {
  local holder holder_pid
  # The lock lives on fd 8. `flock` applies to its shared open-file description,
  # so the short Perl probe leaves it held by this shell; the kernel releases it
  # even after SIGKILL. That is safer than inferring stale ownership from a PID.
  exec 8>>"$gate_arbiter_lock"
  while ! perl -e 'use Fcntl qw(:flock); open my $lock, ">&=8" or exit 1; flock($lock, LOCK_EX | LOCK_NB) or exit 1;' 2>/dev/null; do
    holder="$(cat "$gate_marker" 2>/dev/null || true)"
    holder_pid="$(gate_holder_field pid "$holder")"
    if [[ "$holder_pid" =~ ^[0-9]+$ ]] && kill -0 "$holder_pid" 2>/dev/null; then
      printf 'native-gate: waiting for arbiter holder %s\n' "$holder" >&2
    elif [[ "$holder_pid" =~ ^[0-9]+$ ]]; then
      # The marker can outlive a killed holder, but the flock cannot. The next
      # successful acquisition overwrites this stale identity without stealing
      # a live slot or deleting a path another process might be publishing.
      printf 'native-gate: waiting for arbiter to recover stale holder %s\n' "$holder" >&2
    else
      # An owner creates the directory before publishing its identity. Do not
      # reclaim that tiny window: missing metadata is not stale evidence.
      printf 'native-gate: waiting for arbiter holder identity to publish\n' >&2
    fi
    sleep 1
  done
}

gate_arbiter_publish() {
  # These identify work that has actually been admitted. Capturing them before
  # queueing would spend a caller's budget in line and could certify a SHA that
  # changed while it waited.
  printf 'pid=%s start_commit=%s start_epoch=%s\n' "$$" "$start_commit" "$gate_started_epoch" \
    >"$gate_marker"
  printf 'native-gate-arbiter capacity=1 holder pid=%s start_commit=%s start_epoch=%s\n' \
    "$$" "$start_commit" "$gate_started_epoch"
}

gate_arbiter_release() {
  rm -f "$gate_marker"
  # Closing fd 8 returns the admission slot. The lock file itself carries no
  # authority and may persist, just like disk-preflight's lock file.
  exec 8>&-
}

gate_arbiter_acquire
start_commit="$(git rev-parse HEAD)"
gate_started_epoch="$(date +%s)"
gate_run_dir="$(mktemp -d "${TMPDIR:-/tmp}/awl-native-gate.XXXXXX")"
gate_arbiter_publish

# The vitals heartbeat launched below (`gate_vitals_loop`) is put in its own
# process group by `gate_launch`, same as every phase, so a signal aimed only
# at THIS script's pid never reaches it on its own. The three explicit
# `kill -TERM "$vitals_pid"` sites further down only cover the exits they sit
# on (a failed canary, an exhausted budget, a clean finish) — measured live:
# a direct SIGTERM to this script's own pid (a killed background job, a
# forwarded SIGINT, anything that ends the process outside those three
# branches) skips all of them, and the loop is left running at ppid=1 with
# its `sleep` child once its parent is gone — still holding this script's
# inherited stdout open, the exact failure `gate_sleep_then` below warns
# against. Killing it here, in the EXIT trap, makes retirement unconditional:
# this trap already fires on every one of those paths (proven by the marker
# it already removes on a killed run), so `vitals_pid` dying here too closes
# the gap without touching the three existing sites. `vitals_pid=""` is
# declared before the trap so a death before `gate_launch` assigns it still
# finds a bound empty variable under `set -u`, and the kill is a bare PID —
# not a group signal — because `gate_vitals_loop`'s own TERM trap already
# relays to its one sleeper child by exact pid, the same mechanism the three
# existing call sites already rely on.
vitals_pid=""
gate_kill_vitals() {
  [[ -n "$vitals_pid" ]] || return 0
  kill -TERM "$vitals_pid" 2>/dev/null || true
}

gate_teardown() {
  gate_kill_vitals
  # A signal to the top-level gate otherwise leaves phase leaders reparented to
  # init. End their groups before releasing the arbiter or removing diagnostics.
  if [[ -n "${gate_pgid_file:-}" && -f "$gate_pgid_file" ]]; then
    gate_kill_groups TERM
    sleep 1
    gate_kill_groups KILL
  fi
  rm -rf "$gate_run_dir"
  gate_arbiter_release
}

trap gate_teardown EXIT

# The preflight has no reason to retain gate admission after it returns.
( exec 8>&-; AWL_DISK_PREFLIGHT_CALLER=native-gate "$gate_root/.orchestrator/disk-preflight.sh" )

gate_elapsed() { printf '%s\n' $(( $(date +%s) - gate_started_epoch )); }

# Two conventions run at once below, so every bound here is per convention and
# the machine sees twice it.
readonly gate_conventions=2

gate_cpu_count() {
  if [[ -n "${AWL_NATIVE_GATE_CPUS:-}" ]]; then printf '%s\n' "$AWL_NATIVE_GATE_CPUS"; return; fi
  if sysctl -n hw.ncpu 2>/dev/null; then return; fi
  if command -v nproc >/dev/null 2>&1; then nproc; return; fi
  echo 1
}

gate_mem_bytes() {
  if [[ -n "${AWL_NATIVE_GATE_MEM_BYTES:-}" ]]; then printf '%s\n' "$AWL_NATIVE_GATE_MEM_BYTES"; return; fi
  if sysctl -n hw.memsize 2>/dev/null; then return; fi
  if [[ -r /proc/meminfo ]]; then awk '/^MemTotal:/ { print $2 * 1024; exit }' /proc/meminfo; return; fi
  echo 0
}

# Measured at HEAD on 2026-08-02 (`/usr/bin/time -l` over the unit-test binary,
# 3484 tests): peak RSS 448 MiB at one test thread, 486 MiB at three, 667 MiB at
# ten — about 24 MiB per added thread over a ~448 MiB process floor. Wall time
# across that same sweep was 125.1 s / 119.7 s / 126.2 s, i.e. FLAT, because
# `testlock::serial` already serialises every global-touching test. So a thread
# bound buys headroom for free; it is not a speed/safety trade.
readonly gate_thread_floor_bytes=$((512 * 1024 * 1024))
readonly gate_thread_bytes=$((32 * 1024 * 1024))

gate_test_threads() {
  local cpus="$1" mem_bytes="$2" cpu_share mem_share
  cpu_share=$(( cpus / gate_conventions ))
  (( cpu_share < 1 )) && cpu_share=1
  mem_share=$cpu_share
  if (( mem_bytes > 0 )); then
    mem_share=$(( (mem_bytes / gate_conventions - gate_thread_floor_bytes) / gate_thread_bytes ))
    (( mem_share < 1 )) && mem_share=1
  fi
  if (( mem_share < cpu_share )); then printf '%s\n' "$mem_share"; else printf '%s\n' "$cpu_share"; fi
}

gate_cpus="$(gate_cpu_count)"
gate_mem_bytes_value="$(gate_mem_bytes)"
# A caller that states a bound owns it; the gate only supplies the default. It
# is deliberately RUST_TEST_THREADS and not a `cargo test` argument, because the
# suite's SCOPE must stay literally unfiltered — this bounds how many tests run
# at once, never which ones run. It is equally deliberately not Cargo's own job
# budget: `.orchestrator/worker-build.sh` is that value's sole owner, and this
# gate must not compete with it.
if [[ -z "${RUST_TEST_THREADS:-}" ]]; then
  RUST_TEST_THREADS="$(gate_test_threads "$gate_cpus" "$gate_mem_bytes_value")"
fi
export RUST_TEST_THREADS

# ── The budget, and why it is two numbers ────────────────────────────────────
# The budget exists to convert an OUTCOME NOBODY CAN READ into one anybody can.
# Left unset it does nothing, so no local run inherits a new way to fail.
#
# A duration alone was not enough. `AWL_NATIVE_GATE_BUDGET_SECONDS` starts when
# THIS SCRIPT starts, but the thing racing it — a hosted macOS runner that stops
# talking to the server, upstream actions/runner-images#13882 — is on the
# RUNNER's clock, which starts at job step 1. This script can start at different
# points in that clock, so the same duration expires at different points on the
# clock that actually kills the job. `AWL_NATIVE_GATE_DEADLINE_EPOCH` is an
# ABSOLUTE unix time, set by the caller from the job's own start, and the gate
# takes whichever of the two comes first.
gate_budget_seconds=""
gate_budget_source="none"
if [[ -n "${AWL_NATIVE_GATE_BUDGET_SECONDS:-}" ]]; then
  gate_budget_seconds="$AWL_NATIVE_GATE_BUDGET_SECONDS"
  gate_budget_source="duration"
fi
if [[ -n "${AWL_NATIVE_GATE_DEADLINE_EPOCH:-}" ]]; then
  gate_deadline_remaining=$(( AWL_NATIVE_GATE_DEADLINE_EPOCH - gate_started_epoch ))
  (( gate_deadline_remaining < 1 )) && gate_deadline_remaining=1
  if [[ -z "$gate_budget_seconds" ]] || (( gate_deadline_remaining < gate_budget_seconds )); then
    gate_budget_seconds="$gate_deadline_remaining"
    gate_budget_source="deadline"
  fi
fi

# A hosted runner that is starved to death uploads NO log at all — the mac job's
# step-8 deaths on 2026-08-01/02 left an HTTP 404 where the log should be — so
# the gate states the machine it is about to load BEFORE it loads it, and keeps
# saying what that machine is doing while it runs. Both lines are unconditional:
# evidence that only appears on failure is evidence nobody has ever read.
printf 'native-gate-env cpus=%s mem_bytes=%s conventions=%s test_threads=%s budget_seconds=%s budget_source=%s deadline_epoch=%s\n' \
  "$gate_cpus" "$gate_mem_bytes_value" "$gate_conventions" "$RUST_TEST_THREADS" \
  "${gate_budget_seconds:-none}" "$gate_budget_source" \
  "${AWL_NATIVE_GATE_DEADLINE_EPOCH:-none}"

gate_free_bytes() {
  if [[ -r /proc/meminfo ]]; then
    awk '/^MemAvailable:/ { print $2 * 1024; exit }' /proc/meminfo
    return
  fi
  # The page size lives mid-header ("… (page size of 16384 bytes)"), not in the
  # last field: reading $NF there yields "bytes)" and every sample reports zero.
  vm_stat 2>/dev/null | awk '
    NR == 1 { size = $0; sub(/.*page size of /, "", size); sub(/[^0-9].*/, "", size); page = size + 0 }
    /^Pages free/ || /^Pages inactive/ || /^Pages speculative/ { gsub(/\./, "", $NF); pages += $NF }
    END { print pages * page }
  '
}

gate_swap_bytes() {
  if [[ -r /proc/meminfo ]]; then
    awk '/^SwapTotal:/ { t = $2 } /^SwapFree:/ { f = $2 } END { print (t - f) * 1024 }' /proc/meminfo
    return
  fi
  sysctl -n vm.swapusage 2>/dev/null | awk '{ gsub(/M/, "", $6); printf "%.0f\n", $6 * 1048576 }'
}

# ── Deadlock or livelock: the one number that separates them ─────────────────
# Steady memory and zero swap read identically for both. Processes blocked on a
# GPU fence and processes spinning on one both leave the memory graph flat, and
# the fixes have nothing in common. CPU is the discriminator.
#
# The system load average is the headline but it cannot answer the question on
# its own: it says the box is busy, never WHICH process is busy, and on a
# shared runner it is also the slowest thing on the machine to move. So the
# heartbeat reports both — the machine's one-minute load beside the core count
# that makes it readable, and the CPU seconds each process in the gate's own
# tracked groups actually consumed since the previous heartbeat, over the wall
# time between the two samples. 100 is one core pegged.
#
# It is a DELTA and not `ps -o pcpu` because pcpu does not mean the same thing
# on the two platforms this script runs on: a lifetime average on Linux, a
# decayed one on macOS. A suite that ran hot for 3.5 minutes and then hung for
# 35 reads as roughly 9% on one and roughly 0% on the other, and neither number
# is about the interval anybody is asking about. `-o time=` is POSIX and is
# cumulative on both.
gate_load1() {
  local raw="" pattern='^[0-9]+([.][0-9]+)?$'
  if [[ -r /proc/loadavg ]]; then
    raw="$(awk '{ print $1 }' /proc/loadavg)"
  else
    # `{ 5.70 12.72 16.79 }` — the braces are fields, so the first number is $2.
    raw="$(sysctl -n vm.loadavg 2>/dev/null | awk '{ print $2 }')"
    if [[ -z "$raw" ]]; then
      raw="$(uptime 2>/dev/null | awk '
        {
          for (i = 1; i < NF; i++) if ($i ~ /^averages?:$/) {
            value = $(i + 1); sub(/,$/, "", value); print value; exit
          }
        }
      ')"
    fi
  fi
  # An unparsed reading says so. Printing 0.00 for one is the exact failure this
  # heartbeat has already shipped once: a memory probe that read macOS's page
  # size out of the wrong field reported free_bytes=0 through a full green gate,
  # and only a human reading the output ever noticed.
  [[ "$raw" =~ $pattern ]] || { printf 'unavailable\n'; return 1; }
  printf '%s\n' "$raw"
}

gate_cpu_prev="$gate_run_dir/cpu-prev"

# One sample: the epoch it was taken at, then `pid cpu_seconds age_seconds name`
# for every process in a tracked group. macOS prints `mm:ss.ff` with hundredths
# and Linux `[[dd-]hh:]mm:ss` whole seconds; both are colon-scaled, so one
# parser reads both, and the Linux quantum is 1 s against a 60 s window.
gate_cpu_sample() {
  date +%s
  if ! ps -A -o pid=,pgid=,etime=,time=,comm= 2>/dev/null \
    | awk -v groups="$(tr '\n' ' ' <"$gate_pgid_file")" '
        function seconds(t,   days, parts, p, i, out) {
          days = 0
          if (t ~ /-/) { split(t, p, "-"); days = p[1] + 0; t = p[2] }
          parts = split(t, p, ":")
          out = 0
          for (i = 1; i <= parts; i++) out = out * 60 + (p[i] + 0)
          return out + days * 86400
        }
        BEGIN { n = split(groups, g, " "); for (i = 1; i <= n; i++) if (g[i] != "") want[g[i]] = 1 }
        !want[$2] { next }
        {
          name = $5
          for (i = 6; i <= NF; i++) name = name " " $i
          sub(/.*\//, "", name)
          if (name == "") name = "unnamed"
          print $1, seconds($4), seconds($3), name
        }
      '; then
    printf 'unavailable\n'
  fi
}

# A pid present in both samples is measured over the window between them. A pid
# that appeared inside the window is measured over its own age instead, and
# counted in `new_procs` so a reader knows which readings those are.
#
# Dropping the newcomers, as the first draft did, was the same confident zero
# this heartbeat exists to avoid, one level in: on the receipt run of
# 2026-08-02 the unit-test binary finished and the integration binaries started
# inside one window, and two heartbeats reported `tracked_procs=0` and `0.6%`
# while two test binaries were burning a core each. Crediting a newcomer the
# whole window it was not alive for is the opposite lie, so it gets its own age.
#
# `tracked_procs=0` still prints `tracked_cpu_pct=none` rather than `0.0`: no
# measurement and an idle machine are exactly the two answers this heartbeat
# exists to tell apart.
gate_cpu_report() {
  local now="$gate_run_dir/cpu-now"
  gate_cpu_sample >"$now"
  awk '
    BEGIN { best_pct = -1 }
    FNR == 1 {
      file++
      if (file == 1) { prev_epoch = $1 + 0 }
      else { window = ($1 + 0) - prev_epoch; if (window < 1) window = 1 }
      next
    }
    $1 == "unavailable" { unavailable = 1; next }
    file == 1 { was[$1] = $2 + 0; seen[$1] = 1; next }
    {
      if ($1 in seen) {
        span = window
        delta = ($2 + 0) - was[$1]
      } else {
        fresh++
        # Its whole CPU time over its whole age — a lifetime average, which is
        # the only honest reading for a process with no baseline. NOT over the
        # window: a process can be older than the window and still new to the
        # probe, because a phase group is registered as it launches, and
        # dividing a long lifetime by a short window reports several hundred
        # percent for something merely idle.
        span = ($3 + 0 < 1) ? 1 : $3 + 0
        delta = $2 + 0
      }
      if (delta < 0) delta = 0
      pct = delta * 100 / span
      matched++
      total += pct
      if (pct > best_pct) { best_pct = pct; best = $4 ":" $1 }
    }
    END {
      if (unavailable) {
        print "cpu_probe=unavailable window_seconds=0 tracked_procs=0 new_procs=0 tracked_cpu_pct=none busiest=[none]"
      } else if (file < 2) {
        print "cpu_probe=broken window_seconds=0 tracked_procs=0 new_procs=0 tracked_cpu_pct=none busiest=[none]"
      } else if (matched == 0) {
        printf "window_seconds=%d tracked_procs=0 new_procs=0 tracked_cpu_pct=none busiest=[none]\n", window
      } else {
        printf "window_seconds=%d tracked_procs=%d new_procs=%d tracked_cpu_pct=%.1f busiest=[%s=%.1f]\n", \
          window, matched, fresh + 0, total, best, best_pct
      }
    }
  ' "$gate_cpu_prev" "$now"
  mv "$now" "$gate_cpu_prev"
}

gate_vitals_interval="${AWL_NATIVE_GATE_VITALS_SECONDS:-60}"

# ── Per-phase timing, and naming the line a hang stopped on ──────────────────
# Both conventions write to one stdout, so without line labels
# "which convention got where" was not readable at all. Every convention line
# now carries its own label, and the phase boundaries Cargo already announces
# get a timestamped marker — which answers, without a second run, whether a
# 40-minute step is COMPILING test harnesses or RUNNING tests.
#
# The per-convention progress file is APPEND-only on purpose: a truncating
# writer races the heartbeat that reads it, and `tail -1` of an append-only file
# always yields a whole line.
gate_phase() {
  printf 'native-gate-phase label=%s event=%s elapsed_seconds=%s %s\n' \
    "$1" "$2" "$(gate_elapsed)" "${3:-}"
}

gate_progress_file() { printf '%s\n' "$gate_run_dir/progress-$1"; }

gate_last_progress() {
  local file
  file="$(gate_progress_file "$1")"
  [[ -s "$file" ]] || { printf 'none\n'; return; }
  tail -n 1 "$file"
}

gate_stamp_phases() {
  local label="$1" line target progress compiled=0 running=0
  progress="$(gate_progress_file "$label")"
  : >"$progress"
  # This filter deliberately IGNORES SIGTERM. The budget's TERM is aimed at
  # Cargo; when Cargo dies the pipe closes, and the filter's last act is to
  # flush the unterminated line libtest left behind — "test NAME ... " with no
  # result, which is the exact name of the test that never returned. Killing
  # the filter alongside Cargo would throw that line away, and it is the one
  # line worth the whole exercise. The escalation's follow-up KILL retires the
  # filter if it does not leave on its own.
  trap '' TERM
  # `|| [[ -n "$line" ]]` flushes a final partial line. libtest prints
  # "test NAME ... " BEFORE running the test and its result after, so on a clean
  # EOF that trailing fragment is the exact name of the test that never
  # returned — the single most valuable line in the whole log.
  while IFS= read -r line || [[ -n "$line" ]]; do
    printf '%s| %s\n' "$label" "$line"
    # "test result:" is matched BEFORE the bare "test " arm that swallows every
    # per-test line, and that arm exists so a test whose NAME contains
    # "Running (…)" cannot forge a phase marker.
    case "$line" in
      "test result:"*)
        gate_phase "$label" target-end "detail=${line#test result: }" ;;
      "test "*) : ;;
      "running "*)
        (( running )) || { gate_phase "$label" first-tests-running; running=1; } ;;
      *"Finished"*"target(s) in"*)
        (( compiled )) || { gate_phase "$label" compile-finished; compiled=1; } ;;
      *"Running"*"("*")"*)
        target="${line##*/}"
        gate_phase "$label" target-start "target=${target%%)*}" ;;
    esac
    case "$line" in
      "test "*|"running "*|"test result:"*|*"Compiling"*|*"Running"*|*"error"*|*"panicked"*)
        printf '%s\n' "${line:0:160}" >>"$progress" ;;
    esac
    line=""
  done
}

# Both helpers below outlive nothing: each sleeps in a child it can name, so the
# TERM that retires it also retires the sleep. An orphaned `sleep` would keep
# the gate's inherited stdout open, and a caller capturing this script's output
# would block on it long after the receipt was printed.
gate_sleep_then() {
  local seconds="$1" sleeper=""
  shift
  trap '[[ -n "$sleeper" ]] && kill "$sleeper" 2>/dev/null; exit 0' TERM
  sleep "$seconds" &
  sleeper=$!
  wait "$sleeper" 2>/dev/null || exit 0
  "$@"
}

gate_vitals_loop() {
  local elapsed sleeper=""
  trap '[[ -n "$sleeper" ]] && kill "$sleeper" 2>/dev/null; exit 0' TERM
  # The baseline is taken before the first sleep, so heartbeat one already
  # carries a delta instead of a hole. A CPU probe whose first reading is
  # meaningless is a probe that says nothing for the first minute, and the
  # canary phase is entirely inside that minute on a warm runner.
  gate_cpu_sample >"$gate_cpu_prev"
  while :; do
    sleep "$gate_vitals_interval" &
    sleeper=$!
    wait "$sleeper" 2>/dev/null || exit 0
    elapsed="$(gate_elapsed)"
    printf 'native-gate-vitals elapsed_seconds=%s free_bytes=%s swap_used_bytes=%s load1=%s cpu_count=%s %s mac_last=[%s] linux_last=[%s]\n' \
      "$elapsed" "$(gate_free_bytes)" "$(gate_swap_bytes)" \
      "$(gate_load1)" "$gate_cpus" "$(gate_cpu_report)" \
      "$(gate_last_progress mac)" "$(gate_last_progress linux)"
  done
}

# ── Ending a phase that will not end itself ──────────────────────────────────
# `kill $pid` retires `env … cargo test` and NOTHING BELOW IT. Those
# orphans inherit the step's stdout, and a GitHub step does not conclude while
# anything still holds that pipe.
#
# So every phase is launched under `set -m`, making it a process-group leader,
# and the budget kills the GROUP. The per-convention output filter lives inside
# that group too, which is what makes this safe: Cargo's descendants hold the
# filter's pipe, never the step's stdout, so even a descendant that refuses to
# die cannot keep the step open once the filter is gone.
#
# The group list lives in a FILE, not a variable. The watchdog is forked before
# the phases it has to be able to end, so a shell variable would hand it a
# snapshot that is empty exactly when it matters. The file also keeps the
# watchdog's OWN group off the list: reading its own pgid back would make its
# first `kill` a suicide, and the abort message would never be written.
gate_pgid_file="$gate_run_dir/phase-pgids"
: >"$gate_pgid_file"

gate_launch() {
  local var="$1" tracked="$2"
  shift 2
  set -m
  # fd 8 is the parent gate's admission lease. No phase, watchdog, test binary,
  # or orphan fixture may inherit it: killing the holder must release the queue
  # even while a descendant takes time to die.
  ( exec 8>&-; "$@" ) &
  local launched=$!
  set +m
  [[ "$tracked" == tracked ]] && printf '%s\n' "$launched" >>"$gate_pgid_file"
  eval "$var=$launched"
}

gate_kill_groups() {
  local signal="$1" pgid
  while read -r pgid; do
    [[ -n "$pgid" ]] || continue
    kill "-$signal" "-$pgid" 2>/dev/null || true
  done <"$gate_pgid_file"
}

gate_run_convention() {
  local label="$1"
  shift
  "$@" 2>&1 | gate_stamp_phases "$label"
}

gate_shard_count="${AWL_NATIVE_GATE_SHARDS:-6}"
if [[ "$gate_shard_count" != 1 && "$gate_shard_count" != 6 ]]; then
  echo "native-gate: AWL_NATIVE_GATE_SHARDS must be 1 or 6 (got $gate_shard_count)" >&2
  exit 2
fi

gate_prepare_tests() {
  local cargo_json="$gate_run_dir/cargo-tests.json"
  cargo test --no-run --message-format=json >"$cargo_json"
  python3 "$gate_root/scripts/native-test-shards.py" artifacts \
    "$cargo_json" "$gate_run_dir/artifacts"

  gate_binary="$(sed -n 's/^binary=//p' "$gate_run_dir/artifacts")"
  [[ -x "$gate_binary" ]] || {
    echo "native-gate: discovered binary is not executable: $gate_binary" >&2
    return 1
  }
  "$gate_binary" --list --format terse >"$gate_run_dir/full.list"
  python3 "$gate_root/scripts/native-test-shards.py" partition \
    "$gate_run_dir/full.list" "$gate_run_dir/shards" "$gate_shard_count"

  # Self-test seam: prove the completeness oracle rejects a missing generated
  # prefix. It is deliberately after partitioning and before the shard lists,
  # so the mutation attacks the subject the gate will actually execute.
  if [[ -n "${AWL_NATIVE_GATE_PROBE_DELETE_PREFIX:-}" ]]; then
    sed '1d' "$gate_run_dir/shards/shard-1.filters" \
      >"$gate_run_dir/shards/shard-1.filters.mutated"
    mv "$gate_run_dir/shards/shard-1.filters.mutated" \
      "$gate_run_dir/shards/shard-1.filters"
  fi

  local shard filter skip
  for (( shard = 1; shard <= gate_shard_count; shard++ )); do
    local args=()
    while IFS= read -r filter; do [[ -n "$filter" ]] && args+=("$filter"); done \
      <"$gate_run_dir/shards/shard-$shard.filters"
    while IFS= read -r skip; do [[ -n "$skip" ]] && args+=(--skip "$skip"); done \
      <"$gate_run_dir/shards/shard-$shard.skips"
    "$gate_binary" "${args[@]}" --list --format terse \
      >"$gate_run_dir/shards/shard-$shard.list"
  done
  local shard_lists=()
  for (( shard = 1; shard <= gate_shard_count; shard++ )); do
    shard_lists+=("$gate_run_dir/shards/shard-$shard.list")
  done
  python3 "$gate_root/scripts/native-test-shards.py" verify \
    "$gate_run_dir/full.list" "${shard_lists[@]}"
  printf 'native-gate-shards count=%s binary=%s integrations=%s\n' \
    "$gate_shard_count" "$gate_binary" "$(grep -c '^integration=' "$gate_run_dir/artifacts")"
}

# EVERY binary unit test, in `gate_shard_count` concurrent processes — the ONE
# owner of that wave. Both the convention arms and the menu-bar axis arm below
# run it, which is what makes "the forced arm sees the same tests a convention
# does" a structural fact rather than two filter lists that agree by hand.
gate_run_unit_shards() {
  local shard filter skip status=0 shard_status
  local shard_pids=()
  for (( shard = 1; shard <= gate_shard_count; shard++ )); do
    local args=()
    while IFS= read -r filter; do [[ -n "$filter" ]] && args+=("$filter"); done \
      <"$gate_run_dir/shards/shard-$shard.filters"
    while IFS= read -r skip; do [[ -n "$skip" ]] && args+=(--skip "$skip"); done \
      <"$gate_run_dir/shards/shard-$shard.skips"
    "$gate_binary" "${args[@]}" &
    shard_pids+=("$!")
  done
  set +e
  for shard in "${!shard_pids[@]}"; do
    wait "${shard_pids[$shard]}"
    shard_status=$?
    (( shard_status == 0 )) || status=$shard_status
  done
  set -e
  return "$status"
}

gate_run_native_suite() {
  local convention="$1" status=0
  export AWL_CONVENTION_FORCE="$convention"
  set +e
  gate_run_unit_shards
  status=$?
  set -e
  (( status == 0 )) || return "$status"

  local integration_args=()
  while IFS= read -r filter; do
    [[ "$filter" == integration=* ]] && integration_args+=(--test "${filter#integration=}")
  done <"$gate_run_dir/artifacts"
  cargo test "${integration_args[@]}"
}

# This is deliberately an integration target, outside the binary unit-test
# target. Its first position makes integration-test discovery disappear loudly.
canary_command=(cargo test --test native_gate_canary)

# ── THE MENU-BAR AXIS: THE BRANCH THIS HOST CANNOT SEE, OVER EVERY UNIT TEST ──
# `menubar::MENU_BAR_ON` is the ONE platform-forked sticky default in the tree —
# OFF on macOS, ON everywhere else. So a law or fixture about the drawn bar
# sweeps NOTHING on a macOS host while being live on every Linux one, and that
# asymmetry is not hypothetical: it took this repo a gating CI RED (a picker
# drawing zero candidate rows on Linux, because the bar's height comes off every
# card's height budget), it fired a global-leak audit on sixty CI tests and zero
# local ones, and it has since cost three more CI reds in two days.
#
# ⚠️ A NAME FILTER CANNOT FIND THIS POPULATION, AND THAT IS MEASURED, NOT ARGUED.
# These arms used to run `cargo test --bin awl -- menubar menu_bar`. A runtime
# census — `menubar::menu_bar_on()` recording its own test thread across a whole
# suite run — found that 1627 of 4043 binary unit tests READ the flag, that 172
# of those PIN it first with `set_menu_bar_on`, and that 1455 take whatever the
# host hands them. Of that exposed 1455, the old filter selected TWO. The three
# laws that shipped blind to this axis (`capture::tests::metric_scale`,
# `render::tests::caret_filled_knockout`, `render::tests::workspace_back_width`)
# were all in the missed remainder, and not one is identifiable by name: a test
# that observes the bar's reserve does not say so in its name, which is the whole
# lesson. (Reading the flag is an upper bound on caring about it — most of the
# 1455 reach it through `menubar_reserve()`, which is 0.0 when the bar is off and
# so changes nothing for them. Which ones actually care is not knowable by
# inspection; running them under the other branch is the only way to ask, and
# that is what this arm is.)
#
# So the arm is not filtered. It runs EVERY binary unit test — the same shards a
# convention runs, through the same `gate_run_unit_shards` — under the forcing
# for the branch THIS HOST'S ambient default is not. On macOS that is
# `AWL_MENU_BAR_FORCE=on`, which is exactly what every Linux host and CI's
# `linux` job run natively; on Linux it is `off`, which is what the mac hosts
# run. The opposite branch needs no arm because the two conventions already run
# the whole suite at the ambient default.
#
# ⚠️ THE AMBIENT IS DERIVED FROM THE HOST, AND THAT DERIVATION IS ITSELF PINNED.
# An arm that forces the branch the conventions already cover sweeps nothing
# while looking identical in the log — the enrolment failure, not an assertion
# failure. `menubar::tests::the_gate_forces_the_branch_this_host_lacks` reads the
# table below out of this file and requires it to agree with
# `menubar::platform_default`, so flipping either const fails a law instead of
# silently retargeting the arm.
#
# ⚠️ IN CI THIS ARM IS OFF BY DEFAULT, DELIBERATELY. CI already runs the full
# suite at BOTH ambients across jobs — `linux` at on, the two `mac` jobs at off —
# so a third full unit wave there buys nothing and spends a runner that has run
# its `timeout-minutes` ceiling out before (a timed-out job reports as
# `cancelled`, easy to misread as a supersede). The gap this arm closes is
# LOCAL: a pre-push gate on one machine only ever sees one ambient. CI therefore
# keeps the cheap name-filtered pair, purely so the axis is still visibly swept
# there, and `AWL_NATIVE_GATE_MENUBAR_FULL=1|0` overrides the choice either way.
# Both modes announce themselves on `native-gate-menubar` and in the receipt.
menubar_filters=(menubar menu_bar)
menubar_on_command=(env AWL_MENU_BAR_FORCE=on RUST_TEST_THREADS=1 cargo test --bin awl -- "${menubar_filters[@]}")
menubar_off_command=(env AWL_MENU_BAR_FORCE=off RUST_TEST_THREADS=1 cargo test --bin awl -- "${menubar_filters[@]}")

# THE TABLE `menubar::tests::the_gate_forces_the_branch_this_host_lacks` READS.
# Keep the two `uname -s` cases on their own lines and in this shape; the law
# parses them and fails naming this file if it cannot.
gate_menubar_uname="$(uname -s)"
case "$gate_menubar_uname" in
  Darwin) gate_menubar_ambient=off ;;  # menubar::MENU_BAR_DEFAULT_MACOS
  *) gate_menubar_ambient=on ;;        # menubar::MENU_BAR_DEFAULT_OTHER
esac
if [[ "$gate_menubar_ambient" == off ]]; then
  gate_menubar_forced=on
else
  gate_menubar_forced=off
fi

gate_menubar_full="${AWL_NATIVE_GATE_MENUBAR_FULL:-}"
if [[ -z "$gate_menubar_full" ]]; then
  if [[ -n "${CI:-}" ]]; then gate_menubar_full=0; else gate_menubar_full=1; fi
fi

gate_run_menubar_suite() {
  export AWL_MENU_BAR_FORCE="$1"
  gate_run_unit_shards
}

# WHAT THIS ARM DOES NOT COVER, printed rather than left to a comment nobody
# reads at 2am: integration targets are outside it (it runs the binary unit-test
# shards, so `tests/*.rs` sees only the ambient default), and so is the ambient
# branch itself, which is the conventions' job. A reader who needs to know what
# the gate swept on this axis reads this line, not the prose above it.
if (( gate_menubar_full )); then
  printf 'native-gate-menubar mode=full-suite host=%s ambient=%s forced=%s scope=binary-unit-tests-all-shards uncovered=integration-targets\n' \
    "$gate_menubar_uname" "$gate_menubar_ambient" "$gate_menubar_forced"
else
  printf 'native-gate-menubar mode=name-filtered host=%s ambient=%s arms=on,off filter=[%s] scope=binary-unit-tests-matching-the-filter uncovered=integration-targets,every-unit-test-whose-NAME-omits-the-filter\n' \
    "$gate_menubar_uname" "$gate_menubar_ambient" "${menubar_filters[*]}"
fi

gate_budget_marker="$gate_run_dir/budget-expired"

gate_budget_expired() {
  printf 'exceeded\n' >"$gate_budget_marker"
  printf 'native-gate: budget of %ss (%s) exceeded at %ss; free_bytes=%s swap_used_bytes=%s load1=%s cpu_count=%s; terminating every phase group\n' \
    "$gate_budget_seconds" "$gate_budget_source" "$(gate_elapsed)" \
    "$(gate_free_bytes)" "$(gate_swap_bytes)" "$(gate_load1)" "$gate_cpus" >&2
  printf 'native-gate-budget-last label=mac line=[%s]\n' "$(gate_last_progress mac)" >&2
  printf 'native-gate-budget-last label=linux line=[%s]\n' "$(gate_last_progress linux)" >&2
  # A hung suite is a hung PROCESS; name it and its age before killing it, so
  # the next reader gets the binary and its elapsed time rather than a silence.
  # `time=` beside `etime=` is the deadlock/livelock answer at the instant of
  # death and costs nothing: CPU time far below elapsed means blocked, CPU time
  # tracking elapsed means spinning. `stat=` says the same thing from the
  # kernel's side (R against S/U), and two independent readings of it are worth
  # having in a log that may be the only one this failure ever produces.
  ps -A -o pid=,ppid=,pgid=,etime=,time=,rss=,stat=,comm= 2>/dev/null \
    | awk -v groups="$(tr '\n' ' ' <"$gate_pgid_file")" '
        BEGIN { n = split(groups, g, " "); for (i = 1; i <= n; i++) want[g[i]] = 1 }
        want[$3] { print "native-gate-budget-proc " $0 }
      ' >&2 || true
  gate_kill_groups TERM
  sleep 5
  gate_kill_groups KILL
}

# The budget is armed HERE, before the canary, and covers every phase to the
# receipt. Arming it after the canary — as the first draft did — left the whole
# dependency-and-library compile, the slowest phase on a cold runner, with no
# watchdog at all: a canary that hung would have run to the job's ceiling and
# published nothing.
budget_pid=""
if [[ -n "$gate_budget_seconds" ]]; then
  gate_launch budget_pid untracked gate_sleep_then "$gate_budget_seconds" gate_budget_expired
fi

gate_launch vitals_pid untracked gate_vitals_loop
# The teardown fixture needs the exact child identity even on managed runners
# that deny `ps`/`pgrep`. Production leaves this unset and publishes nothing.
if [[ -n "${AWL_NATIVE_GATE_PROBE_VITALS_PID_FILE:-}" ]]; then
  printf '%s\n' "$vitals_pid" >"$AWL_NATIVE_GATE_PROBE_VITALS_PID_FILE"
fi

gate_aborted_on_budget() { [[ -f "$gate_budget_marker" ]]; }

gate_abort_report() {
  printf 'native-gate: ABORTED on its %ss budget with %s; no receipt issued\n' \
    "${gate_budget_seconds:-unset}" "$1" >&2
}

# The gate must not exit while its watchdog is still escalating. A convention
# subshell dies on the TERM, so the parent's `wait` returns immediately — five
# seconds BEFORE the follow-up KILL that retires anything which ignored the
# TERM. Exiting in that window is precisely the failure this whole change
# exists to remove: the gate would be gone and the survivor would still be
# holding the step's output open. So the parent joins the escalation, then
# re-runs it itself, and only then exits.
gate_finish_abort() {
  [[ -n "$budget_pid" ]] && { wait "$budget_pid" 2>/dev/null || true; }
  gate_kill_groups KILL
  kill -TERM "$vitals_pid" 2>/dev/null || true
  sleep 1
  gate_abort_report "$1"
  exit 1
}

echo "==> native integration canary"
gate_phase canary begin
# The canary runs in the BACKGROUND and is waited on, rather than in the
# foreground, for one reason: bash defers a trap and cannot be interrupted while
# a foreground child runs, so a foreground canary is a phase the budget cannot
# reach. Everything the gate spends time in must be a group it can end.
gate_launch canary_pid tracked gate_run_convention canary "${canary_command[@]}"
set +e
wait "$canary_pid"
canary_status=$?
set -e
gate_phase canary end "status=$canary_status"

if gate_aborted_on_budget; then
  gate_finish_abort "canary_status=$canary_status (budget expired during the canary phase)"
fi
if (( canary_status != 0 )); then
  printf 'native-gate: integration canary failed (status=%s); no receipt issued\n' "$canary_status" >&2
  kill -TERM "$vitals_pid" 2>/dev/null || true
  exit 1
fi

echo "==> discover and prove the binary unit-test shards"
gate_phase prepare begin
gate_launch prepare_pid tracked gate_run_convention prepare gate_prepare_tests
set +e
wait "$prepare_pid"
prepare_status=$?
set -e
gate_phase prepare end "status=$prepare_status"
if gate_aborted_on_budget; then
  gate_finish_abort "prepare_status=$prepare_status (budget expired during shard preparation)"
fi
if (( prepare_status != 0 )); then
  printf 'native-gate: shard preparation or completeness proof failed (status=%s); no receipt issued\n' \
    "$prepare_status" >&2
  kill -TERM "$vitals_pid" 2>/dev/null || true
  exit 1
fi
# The preparation runs in a child process, so publish its discovered path into
# this parent from the artifact manifest rather than relying on shell state.
gate_binary="$(sed -n 's/^binary=//p' "$gate_run_dir/artifacts")"
gate_unit_tests="$(grep -c ': test$' "$gate_run_dir/full.list")"
gate_integration_targets="$(grep -c '^integration=' "$gate_run_dir/artifacts")"

# The canary fronts dependency and library compilation. Cargo's shared-target
# lock prevents duplicate remaining compilation when these siblings start; in
# worker lanes both also inherit the orchestration-owned Cargo cap.
echo "==> native suites (mac and linux conventions, concurrent)"
gate_launch mac_pid tracked gate_run_convention mac gate_run_native_suite mac
gate_launch linux_pid tracked gate_run_convention linux gate_run_native_suite linux
# The menu-bar arm rides alongside the two conventions rather than after them:
# all three share the target dir and the already-built shard binary, so nothing
# here compiles and what is left is test execution overlapped with two suites
# that take longer. The full arm runs the same six shards a convention does.
menubar_on_pid=""
menubar_off_pid=""
menubar_full_pid=""
menubar_on_status=0
menubar_off_status=0
menubar_full_status=0
if (( gate_menubar_full )); then
  echo "==> menu-bar axis (AWL_MENU_BAR_FORCE=$gate_menubar_forced, EVERY unit test, concurrent)"
  gate_launch menubar_full_pid tracked gate_run_convention menubar-full \
    gate_run_menubar_suite "$gate_menubar_forced"
else
  echo "==> menu-bar axis (AWL_MENU_BAR_FORCE on and off, name-filtered, concurrent)"
  gate_launch menubar_on_pid tracked gate_run_convention menubar-on "${menubar_on_command[@]}"
  gate_launch menubar_off_pid tracked gate_run_convention menubar-off "${menubar_off_command[@]}"
fi

# `wait` is allowed to report failure without set -e ending the gate before the
# sibling has finished. Preserve both statuses; neither convention can hide the
# other or authorize a receipt on partial coverage.
set +e
wait "$mac_pid"
mac_status=$?
wait "$linux_pid"
linux_status=$?
if [[ -n "$menubar_full_pid" ]]; then
  wait "$menubar_full_pid"
  menubar_full_status=$?
fi
if [[ -n "$menubar_on_pid" ]]; then
  wait "$menubar_on_pid"
  menubar_on_status=$?
  wait "$menubar_off_pid"
  menubar_off_status=$?
fi
set -e
gate_phase mac suite-end "status=$mac_status"
gate_phase linux suite-end "status=$linux_status"
if (( gate_menubar_full )); then
  gate_phase menubar-full suite-end "status=$menubar_full_status"
else
  gate_phase menubar-on suite-end "status=$menubar_on_status"
  gate_phase menubar-off suite-end "status=$menubar_off_status"
fi

if [[ -f "$gate_budget_marker" ]]; then
  gate_finish_abort "mac_status=$mac_status linux_status=$linux_status"
fi

kill -TERM "$vitals_pid" 2>/dev/null || true
[[ -n "$budget_pid" ]] && { kill -TERM "$budget_pid" 2>/dev/null || true; }

if (( mac_status != 0 || linux_status != 0 )); then
  printf 'native-gate: suite failure mac_status=%s linux_status=%s; no receipt issued\n' \
    "$mac_status" "$linux_status" >&2
  exit 1
fi

# A menu-bar arm failing is a REAL failure of the same tree, and it suppresses
# the receipt exactly like a convention does. It is reported separately, and the
# full arm says the whole diagnosis in one line, because this failure arrives at
# a developer who has just watched the same tests pass: the tree is green at the
# ambient default and red one env var away, which reads as a flake until someone
# explains that it is the axis. Naming the arm, the branch and the remedy is the
# difference between a fixed law and a re-run.
if (( menubar_full_status != 0 )); then
  printf 'native-gate: menu-bar axis failure — the FULL unit suite is red under AWL_MENU_BAR_FORCE=%s while green at this host'"'"'s ambient %s (status=%s). A law that reads the menu-bar default without pinning it measures a different product here than on the other platform; find the named test above, capture the ambient with menubar::menu_bar_on() and restore it, or make the law hold on both branches. No receipt issued.\n' \
    "$gate_menubar_forced" "$gate_menubar_ambient" "$menubar_full_status" >&2
  exit 1
fi

if (( menubar_on_status != 0 || menubar_off_status != 0 )); then
  printf 'native-gate: menu-bar axis failure on_status=%s off_status=%s (filter=%s); no receipt issued\n' \
    "$menubar_on_status" "$menubar_off_status" "${menubar_filters[*]}" >&2
  exit 1
fi

end_commit="$(git rev-parse HEAD)"
if [[ "$start_commit" != "$end_commit" ]]; then
  echo "native-gate: HEAD changed while the suite ran (start=$start_commit end=$end_commit); no receipt issued" >&2
  exit 1
fi

# `menubar=` is the axis's own claim, and it is deliberately narrow: `full:on`
# says every binary unit test ran forced ON as well, `filtered:on,off` says only
# the name-matched ones did. A reader deciding whether a push is covered on this
# axis reads that field rather than inferring it from `scope=all-targets`, which
# speaks for the ambient default alone.
if (( gate_menubar_full )); then
  gate_menubar_claim="full:$gate_menubar_forced"
else
  gate_menubar_claim="filtered:on,off"
fi
printf 'native-gate-receipt commit=%s conventions=mac,linux scope=all-targets menubar=%s unit_tests=%s unit_shards=%s integration_targets=%s\n' \
  "$end_commit" "$gate_menubar_claim" "$gate_unit_tests" "$gate_shard_count" "$gate_integration_targets"
