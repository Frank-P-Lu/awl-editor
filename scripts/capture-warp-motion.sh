#!/usr/bin/env bash
# Kite evidence: fixed framing across page widths and linear forward travel.
#
# The harness proves direction, rate, exact wrap, and page-width invariance.
# These captures are for the remaining human judgement: whether the movement is
# calm and whether the fixed room framing stays comfortable over time.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

RUN_DIR="$ROOT/gallery/warp-motion"
CAPTURES="$RUN_DIR/captures"
SPECIMEN="$ROOT/scripts/world-gallery-specimen.md"
BIN="$ROOT/target/release/awl"
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"

W=1600
H=1000
MEASURE=66
WIDTHS=(20 44 66 92 120 140)
PHASES=(0 101.5 203 304.5 406)

rm -rf "$RUN_DIR"
mkdir -p "$CAPTURES"

echo "==> building awl (release)"
cargo build --release >/dev/null

shoot() {
  local name="$1" phase="$2" measure="$3"
  AWL_WARP_PHASE="$phase" "$BIN" --screenshot "$CAPTURES/$name.png"     --capture-size "${W}x${H}" --measure "$measure" --theme Kite     --config "$NO_CONFIG" "$SPECIMEN" >/dev/null
  grep -q '"name": "Kite"' "$CAPTURES/$name.json"     || { echo "FAIL: $name did not render Kite"; exit 1; }
  grep -q '"kind":"warped-grid"' "$CAPTURES/$name.json"     || { echo "FAIL: $name did not render the warped grid"; exit 1; }
  grep -q "\"measure\": $measure" "$CAPTURES/$name.json"     || { echo "FAIL: $name did not render at measure $measure"; exit 1; }
}

for measure in "${WIDTHS[@]}"; do
  echo "==> width m$measure"
  shoot "width-$measure" 0 "$measure"
done

index=0
for phase in "${PHASES[@]}"; do
  echo "==> phase $phase s"
  shoot "phase-$index" "$phase" "$MEASURE"
  index=$(( index + 1 ))
done

width_sheet="$RUN_DIR/width-sheet.md"
{
  echo "# Kite — fixed framing across page widths"
  echo
  echo "The room field stays at one scale and position. Only the opaque page crop changes."
  echo
  for measure in "${WIDTHS[@]}"; do
    echo "## measure $measure"
    echo
    echo "![Kite at measure $measure|1000](captures/width-$measure.png)"
    echo
  done
} > "$width_sheet"

travel_sheet="$RUN_DIR/travel-sheet.md"
{
  echo "# Kite — one forward loop"
  echo
  echo "The same straight tube at quarter-loop intervals. Rings should travel outward."
  echo "The final frame must match the first."
  echo
  index=0
  for phase in "${PHASES[@]}"; do
    echo "## $phase seconds"
    echo
    echo "![Kite at $phase seconds|1000](captures/phase-$index.png)"
    echo
    index=$(( index + 1 ))
  done
} > "$travel_sheet"

render_sheet() {
  local md="$1" out="$2" panels="$3"
  local height=$(( 700 + panels * 700 ))
  "$BIN" --screenshot "$out" --capture-size "1080x${height}"     --page off --theme Saltpan --config "$NO_CONFIG" "$md" >/dev/null
  echo "wrote $out"
}

render_sheet "$width_sheet" "$RUN_DIR/warp-width-sheet.png" "${#WIDTHS[@]}"
render_sheet "$travel_sheet" "$RUN_DIR/warp-travel-sheet.png" "${#PHASES[@]}"

echo "individual frames: $CAPTURES/<name>.png (+ .json sidecars)"
