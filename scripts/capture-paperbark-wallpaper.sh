#!/usr/bin/env bash
# Real Awl drag-trajectory evidence for Paperbark's fixed Room
# wallpaper. Each frame is one settled headless capture at the next page-width
# position; the labeled sheet is rendered by Awl, not an external compositor.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
BIN="$ROOT/target/release/awl"
RUN_DIR="$ROOT/gallery/paperbark-wallpaper"
CAPTURES="$RUN_DIR/captures"
SPECIMEN="$ROOT/scripts/world-gallery-specimen.md"
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"

cargo build --release
rm -rf "$RUN_DIR"
mkdir -p "$CAPTURES"

# Same fixed viewport, stepped measures matching a page-edge drag trajectory:
# the page covers/reveals stable Room coordinates. A 2x canvas has the same
# logical composition at twice the physical sampling density.
for dpi in 1x 2x; do
  case "$dpi" in
    1x) canvas=1400x900 ;;
    2x) canvas=2800x1800 ;;
  esac
  for measure in 38 54 70 86; do
    "$BIN" --screenshot "$CAPTURES/${dpi}-m${measure}.png" \
      --capture-size "$canvas" --measure "$measure" --theme Paperbark \
      --config "$NO_CONFIG" --keys 's-Down' "$SPECIMEN" >/dev/null
  done
done

sheet="$RUN_DIR/trajectory.md"
{
  echo '# Paperbark — fixed Room wallpaper across page-width dragging'
  echo
  echo 'Each row is a settled Awl capture at the next page-width position in one fixed viewport.'
  echo 'The texture is viewport-anchored; the opaque page alone covers or reveals it.'
  echo
  for measure in 38 54 70 86; do
    echo "## Drag position — measure $measure"
    echo
    echo "![1× measure $measure|500](captures/1x-m${measure}.png)"
    echo
    echo "![2× measure $measure|500](captures/2x-m${measure}.png)"
    echo
    echo '---'
    echo
  done
} > "$sheet"

"$BIN" --screenshot "$RUN_DIR/paperbark-wallpaper-trajectory.png" \
  --capture-size 760x5200 --page off --theme Saltpan --config "$NO_CONFIG" \
  "$sheet" >/dev/null

for dpi in 1x 2x; do
  for measure in 38 54 70 86; do
    test -f "$CAPTURES/${dpi}-m${measure}.png"
  done
done
test -f "$RUN_DIR/paperbark-wallpaper-trajectory.png"
echo "wrote $RUN_DIR/paperbark-wallpaper-trajectory.png"
