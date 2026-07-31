#!/usr/bin/env bash
# Item 194 — THE MOTION SHEET for Kite's tunnel, and the surface the live human
# review is made on.
#
# The harness can prove the two margins are one camera (`render::tests::
# warp_tunnel_item194`) and that the page hides a third of the cross-section. It
# cannot judge the thing item 132's own Verify asks for: whether a BEND READS as
# one opening shifting off-centre, whether the travel feels calm, and whether the
# field is comfortable in the corner of the eye for an hour. That is the user's
# call, and this sheet is what it is made on — every route pose, both margins,
# at a geometry with real margins on both sides.
#
# `AWL_WARP_POSE` is the existing mid-motion knob (a headless capture never ticks
# the clock, so without it no pose but the settled one is reachable). This script
# invents nothing: the named poses are the knob's own vocabulary.
#
# Everything — including the sheet itself — is rendered by awl; no external image
# utility, no network (scripts/capture-worlds.sh owns that convention).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

RUN_DIR="$ROOT/gallery/item-194-warp-motion"
CAPTURES="$RUN_DIR/captures"
SPECIMEN="$ROOT/scripts/world-gallery-specimen.md"
BIN="$ROOT/target/release/awl"
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"

# A wide canvas at a measure that leaves a REAL margin on both sides — the
# geometry the composition is designed for, and the one the review is about.
W=1600
H=1000
MEASURE=66

# The knob's own named poses, plus two mid-EASE phases in raw seconds: the review
# is about a TURN, and a turn is a transition, so the sheet must show the field
# part-way into one rather than only at the steady holds.
POSES=(straight left climb right descent wrap)
EASES=(103.0 277.0)

rm -rf "$RUN_DIR"
mkdir -p "$CAPTURES"

echo "==> building awl (release)"
cargo build --release >/dev/null

shoot() {
  local name="$1" pose="$2"
  AWL_WARP_POSE="$pose" "$BIN" --screenshot "$CAPTURES/$name.png" \
    --capture-size "${W}x${H}" --measure "$MEASURE" --theme Kite \
    --config "$NO_CONFIG" "$SPECIMEN" >/dev/null
  # The sidecar is the STATE oracle: prove the frame really is Kite at the pose
  # it claims, so a sheet cannot silently show the settled still six times.
  grep -q '"name": "Kite"' "$CAPTURES/$name.json" \
    || { echo "FAIL: $name did not render Kite"; exit 1; }
  grep -q '"kind": "warped-grid"' "$CAPTURES/$name.json" \
    || { echo "FAIL: $name did not render the warped grid"; exit 1; }
}

for pose in "${POSES[@]}"; do
  echo "==> $pose"
  shoot "$pose" "$pose"
done
i=1
for phase in "${EASES[@]}"; do
  echo "==> mid-ease $phase s"
  shoot "ease-$i" "$phase"
  i=$(( i + 1 ))
done

sheet="$RUN_DIR/motion-sheet.md"
{
  echo "# Item 194 — Kite's tunnel at every route pose"
  echo
  echo "One camera, one projected cylinder, cropped at the page. Each frame is the"
  echo "same ${W}x${H} canvas at measure ${MEASURE}; only the route's pose differs."
  echo
  echo "What to judge, in order:"
  echo
  echo "1. **Straight** — do the two margins read as one cylinder continuing behind"
  echo "   the page, rather than two separately cropped circles?"
  echo "2. **Left / right** — does the WHOLE opening shift off-centre into the bend,"
  echo "   with the near wall broadening around the outside and the far wall"
  echo "   compressing around the inside? Nothing should pinch or steer per margin."
  echo "3. **Climb / descent** — do both margins lift or drop together?"
  echo "4. **The two mid-ease frames** — is the world part-way through a turn, and"
  echo "   still one coherent opening?"
  echo "5. **Wrap** — indistinguishable from straight, which is the point."
  echo
  for name in "${POSES[@]}"; do
    echo "## $name"
    echo
    echo "![Kite $name|1000](captures/${name}.png)"
    echo
  done
  echo "## mid-ease, 103 s (straight into the left bend)"
  echo
  echo "![Kite ease 1|1000](captures/ease-1.png)"
  echo
  echo "## mid-ease, 277 s (the right bend into the descent)"
  echo
  echo "![Kite ease 2|1000](captures/ease-2.png)"
  echo
} > "$sheet"

panels=$(( ${#POSES[@]} + ${#EASES[@]} ))
height=$(( 700 + panels * 700 ))
"$BIN" --screenshot "$RUN_DIR/warp-motion-sheet.png" \
  --capture-size "1080x${height}" --page off --theme Saltpan --config "$NO_CONFIG" \
  "$sheet" >/dev/null

echo "wrote $RUN_DIR/warp-motion-sheet.png ($panels poses, ${height}px)"
echo "individual frames: $CAPTURES/<pose>.png (+ .json sidecars)"
