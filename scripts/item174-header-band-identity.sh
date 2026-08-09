#!/usr/bin/env bash
#
# CAPTURE IDENTITY MATRIX for the overlay header band.
#
# Shoots the same overlay probes with a given binary into a given directory:
# every world x a surface roster that covers all three header layouts (flat
# picker, grouped/faceted picker, contextual spell popup, workspace), at two
# canvases. Run it once against a base build and once against the branch build,
# then diff the two trees byte for byte — the migrated surfaces must be EXACT.
#
# Usage: scripts/item174-header-band-identity.sh <binary> <outdir>
set -euo pipefail

BIN="${1:?usage: item174-header-band-identity.sh <binary> <outdir>}"
OUT="${2:?usage: item174-header-band-identity.sh <binary> <outdir>}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

rm -rf "$OUT"
mkdir -p "$OUT"

SAMPLE="$ROOT/samples/prose.md"
[[ -f "$SAMPLE" ]] || SAMPLE="$(ls "$ROOT"/samples/*.md | head -1)"

# name|keys — every distinct header layout the family can reach.
# name|keys — every distinct header layout the family can reach: the GROUPED
# card (query line + lens strip), the FLAT card (query line carrying the beat —
# where the retired pointer band missed), the WORKSPACE, and the CONTEXTUAL
# spell popup (no header line at all).
SURFACES=(
  'palette|s-p'                              # grouped, default lens
  'palette-lens|s-p Right'                   # grouped, a non-default lens active
  'palette-typed|s-p c a'                    # grouped, typed query + filtered rows
  'palette-empty|s-p z q x j'                # grouped, zero matches (notice row)
  'goto|s-o'                                 # grouped file picker
  'switch|s-S-p'                             # grouped project picker
  'browse|s-p b r o w s e Enter'             # grouped browser
  'theme|s-t'                                # FLAT, whole-corpus window
  'keybindings|s-p k e y b i n d Enter'      # FLAT, long corpus + secondary column
  'caret|s-p c a r e t Enter'                # FLAT, short corpus
  'rename|s-p r e n a m e Enter'             # FLAT, no drawn title prefix
  'dictionary|s-p d i c t Enter'             # FLAT
  'settings|s-,'                             # the workspace family
  'spell|M-> Enter t e h x z q Left Left s-;' # contextual popup: no header line
)

# canvas|dpi — including a retina cell, where the retired band's miss doubled.
CANVASES=('1200x800|1' '900x520|1' '2400x1600|2')

worlds=$("$BIN" --list-worlds)
[[ -n "$worlds" ]] || { echo "error: empty world roster" >&2; exit 1; }

n=0
for world in $worlds; do
  for cell in "${CANVASES[@]}"; do
    canvas="${cell%%|*}"
    dpi="${cell#*|}"
    for entry in "${SURFACES[@]}"; do
      name="${entry%%|*}"
      keys="${entry#*|}"
      stem="$OUT/${world}__${canvas}@${dpi}__${name}"
      "$BIN" --screenshot "$stem.png" --theme "$world" --capture-size "$canvas" \
             --capture-dpi "$dpi" --keys "$keys" "$SAMPLE" >/dev/null 2>"$stem.err" || {
        echo "CAPTURE FAILED: $world $canvas@$dpi $name" >&2
        cat "$stem.err" >&2
        exit 1
      }
      rm -f "$stem.err"
      n=$((n + 1))
    done
  done
done
echo "captured $n probes into $OUT"
