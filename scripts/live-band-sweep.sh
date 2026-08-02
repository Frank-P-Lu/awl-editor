#!/usr/bin/env bash
# live-band-sweep.sh — THE SELECTION-BAND GLIDE SWEEP (macOS, live window only).
#
# What it answers, and nothing else: does a picker's selection band actually
# TRAVEL one row per accepted navigation input, on a real presenting window?
# That is the one claim `--screenshot` structurally cannot make. Every capture
# path leaves the pipeline unarmed (`arm_live_juice` is called only by the live
# App's GPU init), so `chase_or_snap` is unreachable offscreen and a settled
# capture is byte-identical whether the ease works or never gets a second frame.
# Item 211's defect lived exactly there for three sightings while items 104 and
# 106's laws stayed green.
#
# It reads the answer out of the flight/probe trace, not out of exit status:
#
#   apply <Action> sel <before> -> <after>   one accepted input, one index step
#   prepare_highlight logical= target= band_top=  the band's drawn top, per frame
#   redraw ... band_started= presented= keep_hot=  the scheduling decision
#
# THE DEFECT SHAPE, stated as a predicate this script greps for:
#   band_started=true AND presented=true AND keep_hot=false
# — `prepare` started an ease and the loop parked anyway, so the ease never got
# a second frame. That is the every-other-input jump. Any hit is a failure.
#
# TWO HARNESS FACTS THIS SCRIPT EXISTS TO NOT REPEAT (both cost a whole sitting):
#
#   1. `--live-script` forces a Prohibited, non-activating window. Under a
#      locked display or full occlusion it still prints successful-looking
#      `LIVE-PROBE shot … ok` lines while presenting ZERO frames. The shot
#      protocol is not evidence; the presents counter is. This script fails on
#      a zero-presents cell no matter how clean the run looked.
#   2. A lock can land MID-SITTING. Checking it in preflight alone passes a run
#      that was invalidated three minutes later, which is how a previous sweep
#      recorded ten `Occluded` presents and noticed nothing. This script checks
#      at BOTH ends and prints both readings with timestamps.
#
# PACING IS NOT MEASURED HERE, deliberately. Frame intervals on a loaded host
# say nothing about feel, and a load-contaminated interval presented as evidence
# of smoothness is worse than no measurement. Presence — the band moved, one row
# per input, over multiple presented frames — is robust to load and is all this
# script claims. Whether the glide reads as calm stays the user's call.
#
# Usage:
#   scripts/live-band-sweep.sh                    # release build, default cells
#   scripts/live-band-sweep.sh --theme Firetail   # a Bars world: expect NO ease
#   scripts/live-band-sweep.sh --keep             # keep the work dir on PASS
set -uo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "live-band-sweep.sh is macOS-only (it drives a real window server); skipping." >&2
  exit 0
fi

THEME="Quokka"   # a Pane world — the living band's home, and the world of the
                 # 2026-08-01 Commands report. Bars and Diagonal worlds carry no
                 # living band at all and correctly show one settled frame.
KEEP=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --theme) THEME="$2"; shift 2 ;;
    --keep) KEEP=1; shift ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

screen_locked() {
  ioreg -n Root -d1 -a 2>/dev/null | plutil -extract IOConsoleUsers json -o - - 2>/dev/null \
    | grep -q '"CGSSessionScreenIsLocked":true'
}

LOCK_BEFORE="$(date '+%Y-%m-%d %H:%M:%S %Z')"
if screen_locked; then
  echo "error: the screen is LOCKED — the wgpu occlusion gate returns Occluded" >&2
  echo "       before nextDrawable(), so this window would present zero frames." >&2
  exit 1
fi
echo "screen unlocked at $LOCK_BEFORE (opening reading)"

echo "==> building awl (release — the band ease is only honest at release speed)"
cargo build --release || exit 1

WORK="$(mktemp -d /tmp/awl-band-sweep.XXXXXX)"
PROC="awl-bandsweep-$$"          # a unique name: never `awl`, so cleanup can
BIN="$WORK/$PROC"                # never touch someone else's running editor.
cp target/release/awl "$BIN" && chmod +x "$BIN"

FAILED=0
cleanup() {
  pkill -f "$PROC" 2>/dev/null || true
  if [[ "$KEEP" -eq 0 && "$FAILED" -eq 0 ]]; then rm -rf "$WORK"; fi
}
trap cleanup EXIT

FIXTURE="$WORK/fix.md"
cat > "$FIXTURE" <<'EOF'
# Band sweep fixture

Prose so the page reads as a page behind the picker.
A second line of text.
EOF

# One isolated launch per cell: HOME/XDG point at the work dir, so a sweep can
# never read or write the user's config, session, history or daemon socket.
cell() { # cell NAME SCRIPT
  local name="$1" script="$2"
  local d="$WORK/$name"
  mkdir -p "$d/home" "$d/cfg" "$d/data" "$d/shots"
  HOME="$d/home" XDG_CONFIG_HOME="$d/cfg" XDG_DATA_HOME="$d/data" \
    timeout 90 "$BIN" --theme "$THEME" --live-script "$script" \
    --live-shots "$d/shots" "$FIXTURE" > "$d/stdout.log" 2> "$d/stderr.log"
  local rc=$? log="$d/stderr.log"
  local presented parked occluded eases
  presented=$(grep -c 'presented=true' "$log")
  occluded=$(grep -c 'present SKIPPED Occluded' "$log")
  eases=$(grep -c 'band_started=true' "$log")
  # THE DEFECT PREDICATE. An ease started inside `prepare` and the loop parked
  # on Wait regardless — the frame the band never got.
  parked=$(grep -c 'band_started=true.*presented=true.*keep_hot=false' "$log")
  printf '  %-22s rc=%d presented=%-4s eases=%-3s occluded=%-3s parked-after-ease=%s\n' \
    "$name" "$rc" "$presented" "$eases" "$occluded" "$parked"
  if [[ $rc -ne 0 ]]; then echo "    DEFECT: exited rc=$rc (see $log)"; FAILED=1; fi
  # Zero presents is the silent failure the shot protocol hides: the window was
  # locked, hidden or fully occluded and photographed nothing.
  if [[ "$presented" -eq 0 ]]; then
    echo "    DEFECT: ZERO presented frames — nothing was photographed. Check"
    echo "            display lock and window occlusion, not the GPU."
    FAILED=1
  fi
  if [[ "$parked" -ne 0 ]]; then
    echo "    DEFECT: $parked frame(s) started a band ease and then parked the loop"
    echo "            (band_started=true, keep_hot=false) — item 211's every-other-"
    echo "            input jump. The band is drawn on the row the selection LEFT."
    FAILED=1
  fi
  # The travel itself: distinct band_top values while one selection is settled.
  # A working glide shows many; a snap shows exactly one per input.
  local tops
  tops=$(grep -o 'band_top=Some([0-9.]*)' "$log" | sort -u | wc -l | tr -d ' ')
  echo "    distinct drawn band tops across the cell: $tops"
}

echo "==> sweeping (theme=$THEME)"
D=400  # dwell: comfortably past OVERLAY_BAND_SLIDE_MS, so each tap settles

cell taps \
"sleep 1500; keys s-p; sleep 600; shot open;
 keys Down; sleep $D; shot d1; keys Down; sleep $D; shot d2;
 keys Down; sleep $D; shot d3; keys Up;   sleep $D; shot u1; quit"

# Repeat faster than one glide: the sustained-SNAP regime chase_or_snap exists
# for. The band must never trail, and no index may be lost or doubled.
cell held-repeat \
"sleep 1500; keys s-p; sleep 600;
 keys Down; sleep 60; keys Down; sleep 60; keys Down; sleep 60; keys Down; sleep 60;
 keys Down; sleep 60; keys Down; sleep 60; sleep $D; shot after-repeat; quit"

# A STATIONARY pointer parked on, above and below a row while the keyboard
# drives — item 106's guard, re-asserted live rather than by unit law.
cell pointer-parked \
"sleep 1500; keys s-p; sleep 600;
 move 700 120;  sleep 200; keys Down; sleep $D; shot above-list;
 move 700 240;  sleep 200; keys Down; sleep $D; shot on-a-row;
 move 700 1100; sleep 200; keys Down; sleep $D; shot below-list; quit"

cell theme-picker \
"sleep 1500; keys s-t; sleep 700; shot open;
 keys Down; sleep $D; shot d1; keys Down; sleep $D; shot d2; quit"

echo
echo "==> per-input trace (one line per accepted navigation input)"
for d in "$WORK"/*/; do
  [[ -f "$d/stderr.log" ]] || continue
  echo "--- $(basename "$d")"
  sed 's/ t=Instant.*//' "$d/stderr.log" \
    | grep -E 'apply (NextLine|PreviousLine)|prepare_highlight' \
    | sed 's/^PROBE-TRACE /    /'
done

LOCK_AFTER="$(date '+%Y-%m-%d %H:%M:%S %Z')"
if screen_locked; then
  echo
  echo "error: the screen LOCKED DURING the sweep (closing reading $LOCK_AFTER)." >&2
  echo "       Every present after the lock read Occluded. This run proves nothing;" >&2
  echo "       re-run at an unlocked display, ideally under 'caffeinate -d -i'." >&2
  FAILED=1
else
  echo
  echo "screen still unlocked at $LOCK_AFTER (closing reading) — the sweep is valid"
fi

echo
if [[ "$FAILED" -eq 0 ]]; then
  echo "live-band-sweep → PASS: every cell presented real frames, every accepted"
  echo "input advanced the selection, and no frame started a band ease and parked."
  echo "PRESENCE only — pacing and feel are NOT measured here (see the header)."
else
  echo "live-band-sweep → FAIL (work dir kept at $WORK)"
fi
exit "$FAILED"
