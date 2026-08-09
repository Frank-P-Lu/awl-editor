#!/usr/bin/env bash
#
# The CHROME PIXEL-SPACE capture matrix.
#
# Shoots every world x a surface roster covering all three overlay geometry
# families (flat picker, grouped/faceted picker, contextual spell popup,
# summoned workspace) at three canvas/DPI cells, with a given binary, into a
# given directory.
#
# TWO CLAIMS, ONE MATRIX. At `dpi 1` and the default zoom the multiply is the
# IDENTITY, so those cells must be byte-identical — PNG and sidecar — against a
# base build. At `dpi 2` the deltas are the point, and the report says which
# cells changed and why, per cell.
#
# COMPARE IN PLACE. Run this against the branch build, `git stash`, rebuild,
# run it again into a second directory, then `git stash pop`. Comparing two
# WORKTREES instead reports a spurious sidecar diff, because the gutter renders
# the project name and a different basename changes pixels.
#
# Usage: scripts/item242-chrome-pixel-space-identity.sh <binary> <outdir>
set -euo pipefail

BIN="${1:?usage: item242-chrome-pixel-space-identity.sh <binary> <outdir>}"
OUT="${2:?usage: item242-chrome-pixel-space-identity.sh <binary> <outdir>}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

rm -rf "$OUT"
mkdir -p "$OUT"

SAMPLE="$ROOT/samples/prose.md"
[[ -f "$SAMPLE" ]] || SAMPLE="$(ls "$ROOT"/samples/*.md | head -1)"

# A config path that deliberately does not exist, so every capture gets pure
# built-in defaults regardless of the operator's own config.toml (CAPTURE.md).
NO_CONFIG="$OUT/.unseeded-config.toml"

# name|keys — every distinct chrome composition the migrated families reach.
SURFACES=(
  'palette|s-p'                               # grouped card: query + lens strip
  'palette-typed|s-p c a'                     # grouped, filtered rows
  'palette-empty|s-p z q x j'                 # grouped, the empty-state notice
  'goto|s-o'                                  # grouped file picker
  'switch|s-S-p'                              # grouped project picker
  'browse|s-p b r o w s e Enter'              # grouped browser
  'theme|s-t'                                 # FLAT, whole-corpus window
  'keybindings|s-p k e y b i n d Enter'       # FLAT, secondary chord column
  'caret|s-p c a r e t Enter'                 # FLAT, short corpus
  'rename|s-p r e n a m e Enter'              # FLAT, no drawn title prefix
  'settings|s-,'                              # the summoned workspace
  'spell|M-> Enter t e h x z q Left Left s-;' # contextual popup: no header line
)

# canvas|dpi — two 1x cells (the identity path) and one retina cell (the delta).
CANVASES=('1200x800|1' '900x520|1' '2400x1600|2')

worlds=$("$BIN" --list-worlds)
[[ -n "$worlds" ]] || { echo "error: empty world roster" >&2; exit 1; }

# Shoot the matrix in parallel: a capture is dominated by process + GPU device
# start-up, not by the frame, so a serial sweep of this size costs half an hour
# for no reason. Each probe is an independent process writing its own stem.
JOBS="${AWL_CAPTURE_JOBS:-5}"
plan="$OUT/.plan"
: >"$plan"
n=0
for world in $worlds; do
  for cell in "${CANVASES[@]}"; do
    canvas="${cell%%|*}"
    dpi="${cell#*|}"
    for entry in "${SURFACES[@]}"; do
      name="${entry%%|*}"
      keys="${entry#*|}"
      printf '%s|%s|%s|%s|%s\n' "$world" "$canvas" "$dpi" "$name" "$keys" >>"$plan"
      n=$((n + 1))
    done
  done
done

export BIN OUT SAMPLE NO_CONFIG
shoot() {
  IFS='|' read -r world canvas dpi name keys <<<"$1"
  stem="$OUT/${world}__${canvas}@${dpi}__${name}"
  if ! "$BIN" --screenshot "$stem.png" --theme "$world" --capture-size "$canvas" \
       --capture-dpi "$dpi" --config "$NO_CONFIG" --keys "$keys" \
       "$SAMPLE" >/dev/null 2>"$stem.err"; then
    echo "CAPTURE FAILED: $world $canvas@$dpi $name" >&2
    cat "$stem.err" >&2
    exit 1
  fi
  rm -f "$stem.err"
}
export -f shoot
xargs -P "$JOBS" -I{} bash -c 'shoot "$@"' _ {} <"$plan"
rm -f "$plan" "$NO_CONFIG"
echo "captured $n probes into $OUT"
