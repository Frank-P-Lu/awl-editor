#!/usr/bin/env bash
# Process survival oracle for unowned descendants in the gate's fixtures.

native_gate_probe_running() {
  kill -0 "$1" 2>/dev/null || return 1
  local state
  # A zombie retains its PID until its parent reaps it, but has exited and
  # closed its descriptors. Missing process-state evidence must not hide a
  # live orphan when kill -0 succeeded.
  state="$(ps -o stat= -p "$1" 2>/dev/null)" || return 0
  state="${state#"${state%%[![:space:]]*}"}"
  case "$state" in
    Z*) return 1 ;;
    *) return 0 ;;
  esac
}
