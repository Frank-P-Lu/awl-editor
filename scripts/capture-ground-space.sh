#!/usr/bin/env bash
# THE 1x/2x GROUND SHEET. One panel pair per world: the same LOGICAL
# canvas rendered at a device ratio of 1.0 and of 2.0. The two must show the
# SAME composition; the 2x panel simply resolves it more finely.
#
# The arithmetic is already covered by a roster-wide ground-space law.
# This sheet exists for the thing arithmetic cannot judge: a mathematically
# consistent conversion can still be UGLY — a mark that reads as a whisper at 1x
# can read as a lump when it doubles in size, or a hairline can go wan when its
# feather stays put. Look at the pairs, not the numbers.
#
# The roster comes from the BINARY (`awl --list-worlds`), never a hand-copied
# shell list, so a new world joins the sheet with nothing here to edit.
# Everything — including the sheet itself — is rendered by awl; no external
# image utility, no network (scripts/capture-worlds.sh owns that convention).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

RUN_DIR="$ROOT/gallery/ground-space"
CAPTURES="$RUN_DIR/captures"
SPECIMEN="$ROOT/scripts/world-gallery-specimen.md"
BIN="$ROOT/target/release/awl"
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"

# One logical rectangle, generous margins so a ground is a real field rather
# than a sliver. The 2x arm is the SAME logical size at twice the resolution.
LOGICAL_W=1100
LOGICAL_H=680
MEASURE=44

rm -rf "$RUN_DIR"
mkdir -p "$CAPTURES"

echo "==> building awl (release)"
cargo build --release >/dev/null

worlds=()
while IFS= read -r w; do [[ -n "$w" ]] && worlds+=("$w"); done < <("$BIN" --list-worlds)
if (( ${#worlds[@]} == 0 )); then
  echo "error: empty world roster from --list-worlds" >&2
  exit 1
fi
echo "==> capturing ${#worlds[@]} worlds at 1x and 2x"

for world in "${worlds[@]}"; do
  "$BIN" --screenshot "$CAPTURES/$world-1x.png" \
    --capture-size "${LOGICAL_W}x${LOGICAL_H}" \
    --measure "$MEASURE" --theme "$world" --config "$NO_CONFIG" \
    "$SPECIMEN" >/dev/null
  "$BIN" --screenshot "$CAPTURES/$world-2x.png" \
    --capture-size "$((LOGICAL_W * 2))x$((LOGICAL_H * 2))" --capture-dpi 2.0 \
    --measure "$MEASURE" --theme "$world" --config "$NO_CONFIG" \
    "$SPECIMEN" >/dev/null
  # The sidecar is the STATE oracle: prove each panel really rendered the world
  # and the density it claims, so the sheet cannot silently show one twice.
  grep -q "\"dpi\": 2" "$CAPTURES/$world-2x.json" \
    || { echo "FAIL: $world 2x panel did not record dpi 2"; exit 1; }
  grep -q "\"name\": \"$world\"" "$CAPTURES/$world-1x.json" \
    || { echo "FAIL: $world 1x panel rendered another world"; exit 1; }
done

# The sheet is rendered by awl itself, so it is bounded by the GPU's own max
# texture dimension (8192). Chunk the roster rather than shrink the panels: the
# whole point is looking at marks at something near their real size.
PER_SHEET=6
build_sheet() {
  local idx="$1"; shift
  local group=("$@")
  local sheet="$RUN_DIR/sheet-$idx.md"
  {
    echo "# Every ground at 1x and 2x, same logical canvas (sheet $idx)"
    echo
    echo "Each pair is the SAME ${LOGICAL_W}x${LOGICAL_H} logical canvas: first at a"
    echo "device ratio of 1.0, then at 2.0 (shown at its logical size). The"
    echo "composition must be the same picture — same number of marks, same size —"
    echo "with the 2x panel merely resolving it more finely."
    echo
    echo "The arithmetic is law-tested. This sheet is the catch-the-ugly pass: a"
    echo "consistent conversion can still read badly."
    echo
    for world in "${group[@]}"; do
      echo "## $world"
      echo
      echo "![${world} 1x|960](captures/${world}-1x.png)"
      echo
      echo "![${world} 2x|960](captures/${world}-2x.png)"
      echo
    done
  } > "$sheet"
  local height=$(( 320 + ${#group[@]} * 1290 ))
  "$BIN" --screenshot "$RUN_DIR/ground-space-sheet-$idx.png" \
    --capture-size "1040x${height}" --page off --theme Saltpan --config "$NO_CONFIG" \
    "$sheet" >/dev/null
  echo "wrote $RUN_DIR/ground-space-sheet-$idx.png (${#group[@]} worlds, ${height}px)"
}

idx=1
group=()
for world in "${worlds[@]}"; do
  group+=("$world")
  if (( ${#group[@]} == PER_SHEET )); then
    build_sheet "$idx" "${group[@]}"
    group=()
    idx=$(( idx + 1 ))
  fi
done
(( ${#group[@]} > 0 )) && build_sheet "$idx" "${group[@]}"

echo "individual panels: $CAPTURES/<World>-{1x,2x}.png"
