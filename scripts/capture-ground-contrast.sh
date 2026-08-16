#!/usr/bin/env bash
#
# capture-ground-contrast.sh — the idle ground-contrast audit sweep.
#
# The world dashboard (scripts/capture-worlds.sh) captures every world's Room
# and Frame at ONE fixed wide canvas. This audit asks a different
# question — how strongly does a world ask for attention while the user is
# simply WRITING — and that answer moves with page width: the ground only
# exists in the page-mode margins, so how much territory it owns, and how
# many repeats of its pattern land in that territory, are both functions of
# (canvas width, column measure). A world that is quiet at a laptop width can
# be busy at a wide one, and the audit has to see both.
#
# So this sweeps every world's ROOM across the widths a user actually gets:
#
#   narrow  1440x900   measure 70   a laptop window, prose column
#   laptop  1600x1000  measure 70   the dashboard's canvas, prose column
#   wide    2000x1250  measure 70   a wide window, prose column
#   code    2000x1250  measure 100  a wide window at the CODE measure
#                                   (docs/config.md: page_width_code=100) — the
#                                   narrowest margins a default-config user
#                                   ever writes in
#
# 70/100 are the product's own defaults (`page_width_prose` / `page_width_code`),
# not invented numbers. Frames are deliberately NOT captured: the audit
# definition counts palette, typography, margin pattern and ambient motion, and
# says summoned overlays do not.
#
# WHY THE NARROW ARM IS 1440 AND NOT SOMETHING SMALLER. The adaptive column
# (docs/render.md) shifts the writing column RIGHT under width pressure to
# grant the margin outline a rail, taking the space out of the right margin.
# Measured at measure=70: 2000px canvas -> 496/496 symmetric; 1600 -> 296/296
# symmetric; 1440 -> 244 left / 188 right; 1300 -> 244/48; 1200 -> 176/16.
# So below ~1600 the ground's right margin collapses fast, and by 1280 there
# is essentially no right ground left to look at. That is intended policy, not
# a defect, but it means a canvas under ~1400 measures the rail rather than the
# world. 1440 is the narrowest arm where both margins still carry real ground.
#
# THE SAME PRESSURE IS WHY THE CODE ARM IS 2000 WIDE. At measure=100 the column
# resolves to 1440px, so a 1600 window leaves 144 left / 16 right — the ground
# is effectively gone. That is a real product fact worth stating plainly: in a
# CODE buffer at an ordinary window size, every world's margin pattern all but
# disappears and the roster's ground-contrast spread collapses toward its palette
# alone. The code arm here is therefore a wide window, which is the only place
# a code buffer shows its ground at all.
#
# The roster comes from the BINARY (`--list-worlds`), never a shell list, for
# the same reason capture-worlds.sh does it: enrolling or retiring a world
# changes this sweep with nothing here to edit.
#
# Output — a REPLACEABLE gitignored run dir, wiped and rebuilt every run:
#   gallery/ground-contrast/<arm>/<World>.png + .json
#
# Measurement over these captures is scripts/ground-contrast-measure.py. It reports
# territory and contrast only; pixel arithmetic never claims the
# taste score.
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  for p in "$HOME/.cargo/bin" \
           "$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin"; do
    if [[ -x "$p/cargo" ]]; then export PATH="$p:$PATH"; break; fi
  done
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found on PATH." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

BIN="$ROOT/target/release/awl"
echo "==> building awl (release) — a ground-contrast reading is only honest on the release build"
cargo build --release

SPECIMEN="$SCRIPT_DIR/world-gallery-specimen.md"
if [[ ! -f "$SPECIMEN" ]]; then
  echo "error: missing specimen fixture $SPECIMEN" >&2
  exit 1
fi

RUN_DIR="$ROOT/gallery/ground-contrast"
rm -rf "$RUN_DIR"
mkdir -p "$RUN_DIR"

# A config path that deliberately does not exist, so every capture gets pure
# built-in defaults regardless of the operator's own config.toml (CAPTURE.md).
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"

# Caret parked at buffer end, so every heading sits off-caret and renders
# fully WYSIWYG-concealed — the idle writing view, not a raw-markdown line.
ROOM_KEYS="s-Down"

# arm:canvas:measure
ARMS=(
  "narrow:1440x900:70"
  "laptop:1600x1000:70"
  "wide:2000x1250:70"
  "code:2000x1250:100"
)

echo "==> roster: querying $BIN --list-worlds"
worlds_raw="$("$BIN" --list-worlds)"
if [[ -z "$worlds_raw" ]]; then
  echo "error: --list-worlds returned no worlds" >&2
  exit 1
fi
# shellcheck disable=SC2206
worlds=($worlds_raw)
echo "==> ${#worlds[@]} worlds x ${#ARMS[@]} arms"

for spec in "${ARMS[@]}"; do
  arm="${spec%%:*}"; rest="${spec#*:}"
  canvas="${rest%%:*}"; measure="${rest##*:}"
  out="$RUN_DIR/$arm"
  mkdir -p "$out"
  echo "==> arm $arm — canvas $canvas, measure $measure"
  for world in "${worlds[@]}"; do
    png="$out/$world.png"
    json="$out/$world.json"
    if ! "$BIN" --screenshot "$png" \
         --capture-size "$canvas" --measure "$measure" --page on \
         --theme "$world" --config "$NO_CONFIG" --keys "$ROOM_KEYS" \
         "$SPECIMEN" >/dev/null; then
      echo "error: Room capture failed for '$world' in arm '$arm'" >&2
      exit 1
    fi
    got="$(grep -m1 '^  "theme":' "$json" | grep -Eo '"name": "[^"]*"' | head -1 | sed -E 's/.*"([^"]*)"$/\1/')"
    if [[ "$got" != "$world" ]]; then
      echo "error: sidecar for '$world' ($arm) reports theme '$got'" >&2
      exit 1
    fi
    page_on="$(grep -m1 '^  "page":' "$json" | grep -Eo '"on": (true|false)' | head -1 | awk '{print $2}')"
    if [[ "$page_on" != "true" ]]; then
      echo "error: Room for '$world' ($arm) has page mode OFF — no margins, no ground" >&2
      exit 1
    fi
  done
done

echo
echo "==> done. $(( ${#worlds[@]} * ${#ARMS[@]} )) Room captures under:"
echo "    $RUN_DIR"
