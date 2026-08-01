#!/usr/bin/env bash
#
# capture-ambient-118.sh — the AMBIENT-MOTION arm of item 118's loudness audit.
#
# Five shipping worlds move while the user is idle, and item 118 counts ambient
# motion as loudness. `Theme::has_ambient_tick()` is the roster's own answer for
# which they are — lava (Firetail, Mangrove), animated stars (Currawong), waves
# drift (Bombora), organic drift (Bowerbird) — and this script asks the binary
# rather than hard-coding that list.
#
# WHAT THIS CAN AND CANNOT PROVE, STATED UP FRONT. An ordinary headless capture
# freezes every ambient phase at t=0 by design, so it can say nothing about
# motion at all. What it CAN do is render the SAME world at a series of
# EXPLICIT phases through the shipped dev knobs — `AWL_LAVA=<palette>:<phase>`,
# `AWL_STARS_PHASE`, `AWL_WAVES_PHASE` (which drives Bombora's waves AND
# Bowerbird's organic drift; one shared clock) — and measure how far the field
# actually travels between them. That is a deterministic single-frame
# trajectory, which CAPTURE.md says the harness genuinely verifies.
#
# The phase-to-seconds conversion is the product's own: `lava::LAVA_SPEED` is
# 0.03 cycles per second and `lava::LAVA_LOOP_CYCLES` is 2.0, so a full loop is
# 66.7 real seconds and `phase = seconds * 0.03`. The sampled seconds below are
# therefore real seconds, not invented units.
#
# STILL LIVE-ONLY, AND NOT CLAIMED HERE: whether those frames actually arrive
# at 60fps, whether the tick holds its cadence, and whether the result FEELS
# calm. Item 118 asks for `--release` observation for exactly that, and this
# script is the preparation for that judgement, not a substitute for it.
#
# Output — a REPLACEABLE gitignored run dir:
#   gallery/item-118-ambient/<World>/t<seconds>.png + .json
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  for p in "$HOME/.cargo/bin" \
           "$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin"; do
    if [[ -x "$p/cargo" ]]; then export PATH="$p:$PATH"; break; fi
  done
fi
command -v cargo >/dev/null 2>&1 || { echo "error: cargo not found on PATH" >&2; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

BIN="$ROOT/target/release/awl"
echo "==> building awl (release)"
cargo build --release

SPECIMEN="$SCRIPT_DIR/world-gallery-specimen.md"
RUN_DIR="$ROOT/gallery/item-118-ambient"
rm -rf "$RUN_DIR"
mkdir -p "$RUN_DIR"
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"

CANVAS="1600x1000"
MEASURE=70
KEYS="s-Down"
# Real seconds since the world was entered. Chosen to answer three separate
# questions: does anything move in the first blink (1s, 2s), is it noticeable
# over a glance (5s, 10s), and where is it after a paragraph of writing
# (30s, 60s)?
SECONDS_LIST=(0 1 2 5 10 30 60)
SPEED=0.03

# The ambient roster, from the binary. `--list-worlds` prints every world; the
# per-world sidecar then says whether it carries an ambient mechanism, so the
# selection below is the code's answer, not this script's opinion.
worlds_raw="$("$BIN" --list-worlds)"
# shellcheck disable=SC2206
worlds=($worlds_raw)

phase_for() { awk -v s="$1" -v k="$SPEED" 'BEGIN{printf "%.6f", s*k}'; }

ambient_env() { # ambient_env WORLD PHASE -> echoes "VAR=VALUE" or empty
  case "$1" in
    Firetail)  echo "AWL_LAVA=warm:$2:glow" ;;
    Mangrove)  echo "AWL_LAVA=deepsea:$2:glow:dither" ;;
    Currawong) echo "AWL_STARS_PHASE=$2" ;;
    Bombora|Bowerbird) echo "AWL_WAVES_PHASE=$2" ;;
    *) echo "" ;;
  esac
}

found=0
for world in "${worlds[@]}"; do
  probe="$(ambient_env "$world" 0)"
  [[ -z "$probe" ]] && continue
  found=$((found + 1))
  out="$RUN_DIR/$world"
  mkdir -p "$out"
  echo "==> $world — ${#SECONDS_LIST[@]} phases"
  for s in "${SECONDS_LIST[@]}"; do
    ph="$(phase_for "$s")"
    env_kv="$(ambient_env "$world" "$ph")"
    if ! env "$env_kv" "$BIN" --screenshot "$out/t$s.png" \
         --capture-size "$CANVAS" --measure "$MEASURE" --page on \
         --theme "$world" --config "$NO_CONFIG" --keys "$KEYS" \
         "$SPECIMEN" >/dev/null; then
      echo "error: ambient capture failed for $world at t=${s}s ($env_kv)" >&2
      exit 1
    fi
  done
done

if [[ "$found" -eq 0 ]]; then
  echo "error: no ambient world matched — the roster changed and this script's env map did not" >&2
  exit 1
fi

echo
echo "==> $found ambient worlds captured under $RUN_DIR"
echo "    measure with: scripts/ambient-travel.py"
