#!/usr/bin/env bash
# ci-atspi-probe.sh — item 252: does the AT-SPI2 bridge come up on real Linux
# and expose the tree SemanticSnapshot intends?
#
# ‼ THIS IS NOT ITEM 251 AND DOES NOT CLOSE IT. It proves bridge LIVENESS AND
# STRUCTURE ONLY — that AccessKit's Unix adapter registers with the
# accessibility bus and publishes the document, item 218's stable line runs,
# focus and a live selection. It says nothing about what a screen reader user
# would hear or how navigation feels; that needs Orca on a real Linux desktop
# with a person listening (item 251), which no hosted runner has.
#
# CI-ONLY: stands up a virtual X display (Xvfb) plus a minimal window manager
# (fluxbox — xdotool's keyboard injection needs real X11 input focus, which a
# bare Xvfb with no WM does not reliably grant) and a private D-Bus session
# bus with service activation (`dbus-run-session`, which is what lets
# `org.a11y.Bus.GetAddress` activate at-spi-bus-launcher/at-spi2-registryd the
# first time anything asks for the accessibility bus). It then launches the
# awl binary and hands off to ci-atspi-probe.py, the actual AT-SPI client and
# oracle — this script only stands up the stack around it.
#
# NEVER RUN THIS ON A DEVELOPER MACHINE: it starts Xvfb/fluxbox and drives
# real keyboard events. It is meant only for the CI runner (mirrors the
# `ci-live-probe.sh` warning for the mac live-probe).
#
# Usage: scripts/ci-atspi-probe.sh <path-to-awl-binary>

set -uo pipefail

BIN="${1:?usage: ci-atspi-probe.sh <path-to-awl-binary>}"
DISPLAY_NUM=":99"
XVFB_LOG="$(mktemp -t awl-atspi-xvfb.XXXXXX)"
FLUXBOX_LOG="$(mktemp -t awl-atspi-fluxbox.XXXXXX)"

XVFB_PID=""
FLUXBOX_PID=""
cleanup() {
  # Terminate only what this run started, by PID (CLAUDE.md's "terminate only
  # owned processes" discipline, applied to this job's own helper daemons).
  [ -n "$FLUXBOX_PID" ] && kill "$FLUXBOX_PID" 2>/dev/null
  [ -n "$XVFB_PID" ] && kill "$XVFB_PID" 2>/dev/null
  rm -f "$XVFB_LOG" "$FLUXBOX_LOG"
}
trap cleanup EXIT

echo "ATSPI-PROBE: starting Xvfb on $DISPLAY_NUM"
Xvfb "$DISPLAY_NUM" -screen 0 1280x800x24 >"$XVFB_LOG" 2>&1 &
XVFB_PID=$!
ready=0
for _ in $(seq 1 20); do
  if [ -e "/tmp/.X11-unix/X${DISPLAY_NUM#:}" ]; then
    ready=1
    break
  fi
  if ! kill -0 "$XVFB_PID" 2>/dev/null; then
    break
  fi
  sleep 0.5
done
if [ "$ready" -ne 1 ]; then
  echo "ATSPI-PROBE FAIL: Xvfb did not start on $DISPLAY_NUM" >&2
  cat "$XVFB_LOG" >&2
  exit 1
fi

export DISPLAY="$DISPLAY_NUM"

echo "ATSPI-PROBE: starting fluxbox (xdotool needs a real window manager for input focus)"
fluxbox >"$FLUXBOX_LOG" 2>&1 &
FLUXBOX_PID=$!
sleep 2
if ! kill -0 "$FLUXBOX_PID" 2>/dev/null; then
  echo "ATSPI-PROBE FAIL: fluxbox did not start" >&2
  cat "$FLUXBOX_LOG" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "ATSPI-PROBE: launching under dbus-run-session (activates org.a11y.Bus on first ask)"
dbus-run-session -- python3 "$SCRIPT_DIR/ci-atspi-probe.py" "$BIN"
rc=$?

echo "----- Xvfb log (tail) -----"
tail -n 30 "$XVFB_LOG" 2>/dev/null || true
echo "----- fluxbox log (tail) -----"
tail -n 30 "$FLUXBOX_LOG" 2>/dev/null || true
echo "-------------------------------"

exit "$rc"
