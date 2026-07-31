#!/usr/bin/env bash
# Item 194 — THE MOTION SHEET for Kite's tunnel, and the surface the live human
# review is made on.
#
# ROUND 2 ADDS THE PRIMARY AXIS: PAGE WIDTH. Round 1's sheet shot every route
# pose at ONE column width, and that is precisely why its review could approve
# the composition and still fail the world — the defect was that the projection
# rescaled with the margin it landed in, which no single width can show. This
# sheet therefore has two halves:
#
#   1. THE WIDTH SWEEP, at the settled straight pose: the same world across
#      awl's own measure band, from the narrowest page (widest margins, the
#      composition the first review approved) to a page wide enough that the two
#      windows overlap. What to look for is one cylinder at ONE size, cropped
#      differently — not a tunnel squeezed into whatever room is left.
#   2. THE ROUTE, at every pose the knob names, plus two mid-ease frames, at a
#      geometry with real margins on both sides.
#
# The harness can prove the two margins are one camera and that the section
# never rescales (`render::tests::warp_tunnel_item194`). It cannot judge whether
# a BEND READS as one opening shifting off-centre, whether the travel feels calm,
# or whether the field is comfortable in the corner of the eye for an hour. That
# is the user's call, and this sheet is what it is made on.
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

# A wide canvas; the MEASURE is what this sheet varies.
W=1600
H=1000
MEASURE=66

# The width sweep. 20 is `page::MIN_MEASURE` — the narrowest page the "Narrow
# page" command reaches, and the widest margins the world is ever seen in; 92 is
# past the width where the two windows have slid inward far enough to overlap.
WIDTHS=(20 32 44 56 66 78 92)

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
  local name="$1" pose="$2" measure="$3"
  AWL_WARP_POSE="$pose" "$BIN" --screenshot "$CAPTURES/$name.png" \
    --capture-size "${W}x${H}" --measure "$measure" --theme Kite \
    --config "$NO_CONFIG" "$SPECIMEN" >/dev/null
  # The sidecar is the STATE oracle: prove the frame really is Kite at the pose
  # and the width it claims, so a sheet cannot silently show one frame twice.
  grep -q '"name": "Kite"' "$CAPTURES/$name.json" \
    || { echo "FAIL: $name did not render Kite"; exit 1; }
  grep -q '"kind":"warped-grid"' "$CAPTURES/$name.json" \
    || { echo "FAIL: $name did not render the warped grid"; exit 1; }
  grep -q "\"measure\": $measure" "$CAPTURES/$name.json" \
    || { echo "FAIL: $name did not render at measure $measure"; exit 1; }
}

for m in "${WIDTHS[@]}"; do
  echo "==> width m$m (straight)"
  shoot "width-$m" straight "$m"
done

for pose in "${POSES[@]}"; do
  echo "==> $pose"
  shoot "$pose" "$pose" "$MEASURE"
done
i=1
for phase in "${EASES[@]}"; do
  echo "==> mid-ease $phase s"
  shoot "ease-$i" "$phase" "$MEASURE"
  i=$(( i + 1 ))
done

# The bend, seen at the two ENDS of the width band — the comparison round 1's
# sheet could not make, and the one this round exists for.
for m in 20 92; do
  echo "==> left bend at m$m"
  shoot "bend-left-$m" left "$m"
done

# TWO sheets, not one: a composite is rendered by awl into a single texture,
# and one sheet holding every panel of this round exceeds the 8192px a GPU
# texture dimension allows. The split is along the round's own seam anyway —
# the width axis is the new claim, the route axis is the standing one.
head_of_sheet() {
  echo "# Item 194 round 2 — Kite's tunnel: $1"
  echo
  echo "One cylinder at one constant scale, seen through two windows. Every frame is"
  echo "the same ${W}x${H} canvas."
  echo
}

width_sheet="$RUN_DIR/width-sheet.md"
{
  head_of_sheet "across the page-width band"
  echo "## What to judge, in order"
  echo
  echo "1. Is it the SAME tunnel at every width, cropped differently? The"
  echo "   cross-section's size and its roundness must not change as the page widens."
  echo "   (Round 1's defect was exactly this: the section grew with the page and"
  echo "   flattened, and the world read squashed. That is why this sheet exists.)"
  echo "2. As the margins close, each window slides inward until the centre of the"
  echo "   cylinder appears in BOTH margins. That duplication is intended. Does it"
  echo "   read as one world seen twice, or as two worlds?"
  echo "3. The last two frames are the SAME left bend at both ends of the band: a"
  echo "   turn is legible at the narrowest page; is it still legible at the widest,"
  echo "   where each window holds less of the cylinder? That is this round's own"
  echo "   open question, and the harness cannot answer it."
  echo
  echo "## Straight pose, by page width"
  echo
  for m in "${WIDTHS[@]}"; do
    echo "### measure $m"
    echo
    echo "![Kite at measure $m|1000](captures/width-${m}.png)"
    echo
  done
  echo "## The same left bend at both ends of the band"
  echo
  echo "### measure 20 — widest margins, the windows tile"
  echo
  echo "![Kite left bend at measure 20|1000](captures/bend-left-20.png)"
  echo
  echo "### measure 92 — the windows overlap"
  echo
  echo "![Kite left bend at measure 92|1000](captures/bend-left-92.png)"
  echo
} > "$width_sheet"

route_sheet="$RUN_DIR/route-sheet.md"
{
  head_of_sheet "along the route, at measure ${MEASURE}"
  echo "## What to judge, in order"
  echo
  echo "1. **Straight** — do the two margins read as one cylinder, cropped by the"
  echo "   page, rather than two separately cropped circles?"
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
} > "$route_sheet"

render_sheet() {
  local md="$1" out="$2" panels="$3"
  local height=$(( 900 + panels * 700 ))
  "$BIN" --screenshot "$out" \
    --capture-size "1080x${height}" --page off --theme Saltpan --config "$NO_CONFIG" \
    "$md" >/dev/null
  echo "wrote $out ($panels panels, ${height}px)"
}

render_sheet "$width_sheet" "$RUN_DIR/warp-width-sheet.png" $(( ${#WIDTHS[@]} + 2 ))
render_sheet "$route_sheet" "$RUN_DIR/warp-route-sheet.png" $(( ${#POSES[@]} + ${#EASES[@]} ))

echo "individual frames: $CAPTURES/<name>.png (+ .json sidecars)"
