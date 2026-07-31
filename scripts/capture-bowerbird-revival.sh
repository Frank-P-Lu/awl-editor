#!/usr/bin/env bash
# Item 176 — ONE focused comparison between Bowerbird's shipped ORGANIC control
# (`Arrangement::Masses`, the rounded cut-paper masses) and the crisp
# three-object COLLECTED-TREASURE revival (`Arrangement::Finds`), at a wide and
# a narrow page width and at 1x and 2x, plus the identity check that flipping
# that one word changes no other world.
#
# HOW THE REVIVAL IS REACHED, and why there is no knob for it. The arrangement
# is theme DATA, exactly like Deckle's `Weave`: a world adopts it by writing one
# word in its own literal. No world ships `Finds` yet — the user's verdict on
# this sheet is what decides that — so this script briefly rewrites that ONE
# token in `src/theme/worlds.rs`, builds a second binary, captures, and restores
# the file (an EXIT trap restores it even on interrupt). That patch IS the whole
# ship diff, which is the point: nothing here is a trial selector, an env knob,
# or a rejected arm left behind in the product.
#
# Everything is rendered by awl itself, including the contact sheet — no
# external image utility, no network, no OS automation (scripts/capture-worlds.sh
# owns that convention).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

RUN_DIR="$ROOT/gallery/bowerbird-revival"
CAPTURES="$RUN_DIR/captures"
WORLDS="$ROOT/src/theme/worlds.rs"
SPECIMEN="$ROOT/scripts/world-gallery-specimen.md"
BIN="$ROOT/target/release/awl"
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"
KEEP="$(mktemp -t awl-bowerbird-worlds)"

cp "$WORLDS" "$KEEP"
restore() { cp "$KEEP" "$WORLDS"; rm -f "$KEEP"; }
trap restore EXIT

rm -rf "$RUN_DIR"
mkdir -p "$CAPTURES"

# One arm's four panels plus its whole-roster identity set.
capture_arm() {
  local arm="$1"
  "$BIN" --screenshot "$CAPTURES/$arm-wide-1x.png" --capture-size 1440x900 \
    --measure 74 --theme Bowerbird --config "$NO_CONFIG" --keys 's-Down' \
    "$SPECIMEN" >/dev/null
  "$BIN" --screenshot "$CAPTURES/$arm-narrow-1x.png" --capture-size 900x900 \
    --measure 38 --theme Bowerbird --config "$NO_CONFIG" --keys 's-Down' \
    "$SPECIMEN" >/dev/null
  "$BIN" --screenshot "$CAPTURES/$arm-wide-2x.png" --capture-size 2880x1800 \
    --capture-dpi 2.0 --measure 74 --theme Bowerbird --config "$NO_CONFIG" \
    --keys 's-Down' "$SPECIMEN" >/dev/null
  "$BIN" --screenshot "$CAPTURES/$arm-narrow-2x.png" --capture-size 1800x1800 \
    --capture-dpi 2.0 --measure 38 --theme Bowerbird --config "$NO_CONFIG" \
    --keys 's-Down' "$SPECIMEN" >/dev/null
  mkdir -p "$RUN_DIR/roster-$arm"
  for world in $("$BIN" --list-worlds); do
    "$BIN" --screenshot "$RUN_DIR/roster-$arm/$world.png" --capture-size 1200x800 \
      --measure 44 --theme "$world" --config "$NO_CONFIG" --keys 's-Down' \
      "$SPECIMEN" >/dev/null
  done
}

echo "building the shipped control..."
cargo build --release >/dev/null
capture_arm control

echo "building the revival (one token: Arrangement::Masses -> Arrangement::Finds)..."
python3 - "$WORLDS" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); s = p.read_text()
old, new = "arrangement: Arrangement::Masses,", "arrangement: Arrangement::Finds,"
assert s.count(old) == 1, "expected exactly one Organic world to re-arm"
p.write_text(s.replace(old, new))
PY
cargo build --release >/dev/null
capture_arm revival

restore
trap - EXIT
echo "restoring the shipped control build..."
cargo build --release >/dev/null

# The sidecar is the state oracle for WHICH arm rendered; the PNG is the
# appearance. Both are asserted, so the sheet cannot silently show one arm twice.
for arm in control revival; do
  want=$([ "$arm" = control ] && echo masses || echo finds)
  grep -q "\"arrangement\":\"$want\"" "$CAPTURES/$arm-wide-1x.json" \
    || { echo "FAIL: $arm did not render the $want arrangement"; exit 1; }
done

# Identity: flipping Bowerbird's own word must change Bowerbird and NOTHING else.
changed=0
for world in $("$BIN" --list-worlds); do
  if cmp -s "$RUN_DIR/roster-control/$world.png" "$RUN_DIR/roster-revival/$world.png"; then
    [ "$world" = Bowerbird ] && { echo "FAIL: Bowerbird did not change"; exit 1; }
  else
    changed=$((changed + 1))
    [ "$world" = Bowerbird ] || { echo "FAIL: $world changed and must not have"; exit 1; }
  fi
done
[ "$changed" = 1 ] || { echo "FAIL: expected exactly one world to change, saw $changed"; exit 1; }

sheet="$RUN_DIR/sheet.md"
{
  echo "# Bowerbird — organic control vs. crisp three-object revival"
  echo
  echo "Each pair is the shipped ground (rounded masses) followed by the revival"
  echo "(one anchor, one companion, one cut-out per collection). The narrow-page"
  echo "pair is shown at 1:1 — look there for the grammar. Nothing is shipped:"
  echo "Bowerbird still carries the control."
  echo
  echo "## Narrow page, 1:1 — measure 38, 900x900"
  echo
  echo "### Control — rounded masses"
  echo
  echo "![control narrow|900](captures/control-narrow-1x.png)"
  echo
  echo "### Revival — collected objects"
  echo
  echo "![revival narrow|900](captures/revival-narrow-1x.png)"
  echo
  echo "## Wide page — measure 74, 1440x900"
  echo
  echo "### Control — rounded masses"
  echo
  echo "![control wide|960](captures/control-wide-1x.png)"
  echo
  echo "### Revival — collected objects"
  echo
  echo "![revival wide|960](captures/revival-wide-1x.png)"
  echo
  echo "## Retina — 2x device ratio, narrow page (shown at its logical size)"
  echo
  echo "### Control"
  echo
  echo "![control narrow 2x|900](captures/control-narrow-2x.png)"
  echo
  echo "### Revival"
  echo
  echo "![revival narrow 2x|900](captures/revival-narrow-2x.png)"
} > "$sheet"

"$BIN" --screenshot "$RUN_DIR/bowerbird-revival-comparison.png" \
  --capture-size 1040x5400 --page off --theme Saltpan --config "$NO_CONFIG" \
  "$sheet" >/dev/null

echo "wrote $RUN_DIR/bowerbird-revival-comparison.png"
