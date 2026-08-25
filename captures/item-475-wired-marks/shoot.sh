#!/bin/sh
# Real-pipeline pixel evidence for the WIRED (shipped) fold-mark glyphs —
# reproduce with:
#   sh captures/item-475-wired-marks/shoot.sh
#
# Drives the actual `--screenshot` binary (the real TextPipeline, not a
# synthetic gallery harness) with real key chords against a real markdown
# fixture: caret navigation (Down) to reach each heading level, and
# `s-S-e` (Cmd-Shift-E, ToggleFold's native chord) to fold/unfold it. One
# world per ornament register, one light and one dark member each, so every
# shipped mark is proven on both a light and a dark ground.
set -eu
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
DIR="$(cd "$(dirname "$0")" && pwd)"
OUT="$DIR"
FILE="$DIR/heading.md"

shot() {
  world="$1"; level="$2"; state="$3"; keys="$4"
  out="$OUT/${world}-${level}-${state}.png"
  cargo run --bin awl -- --screenshot "$out" "$FILE" --theme "$world" --keys "$keys" >/dev/null
  echo "wrote $out"
}

# world, register
worlds="Bilby:garamond Bombora:garamond Gumtree:junicode Mopoke:junicode Galah:marks Wagtail:marks"

for pair in $worlds; do
  world="${pair%%:*}"
  for level_keys in "h1: " "h3:Down Down Down Down"; do
    level="${level_keys%%:*}"
    nav="${level_keys#*:}"
    shot "$world" "$level" "expanded" "$nav"
    shot "$world" "$level" "collapsed" "$nav s-S-e"
  done
done

echo "--- all wired-mark shots written to $OUT (scratch, untracked)"
