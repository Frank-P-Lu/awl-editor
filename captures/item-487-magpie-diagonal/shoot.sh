#!/bin/sh
# RESOLVED — historical audition artifact. The picks were built: frost-top0
# and chevron-short are the unconditional production default, query-right
# ships direction-derived, frost-full was rejected and deleted. The
# AWL_DIAGONAL_GALLERY_* vars this script sets NO LONGER EXIST, so every
# "candidate" shot below now renders byte-identical to its own "current"
# shot — see README.md's own note before trusting a re-run's diff.
#
# Reproduce this item's capture gallery from the repo root:
#   sh captures/item-487-magpie-diagonal/shoot.sh
#
# THROWAWAY PROTOTYPE EVIDENCE, not the shipped feature. Item 487's three
# symptoms (query-to-first-item distance, frost boundaries crossing legible
# content, a stranded selected-row chevron) all live in the Diagonal picker
# composition (`src/render/chrome/diagonal.rs` + `src/render/pipeline_prepare.rs`).
# The candidates below run through the SAME production draw path the shipped
# composition already uses — the only new code is a handful of
# `AWL_DIAGONAL_GALLERY_*`-gated branches in `src/render/chrome/diagonal/gallery.rs`
# that are `None`/`false` (byte-identical to today) on every ordinary run,
# including every run of this script's own "current" shots (which set none of
# these vars). No shipped default changed to produce this gallery — see
# README.md's "What this gallery does and does not demonstrate".
#
# Hermetic: the sandbox is seeded from `fixture/` alone, through an explicit
# --config and --root -- never the ambient project or the ambient config --
# so nothing here photographs a real directory. The PNGs and their sidecars
# are scratch and are not committed; the fixture, this script and measure.py
# are, so the set survives the worktree that produced it (mirrors
# captures/item-444-residual3/README.md's own note).
set -eu
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
export AWL_CONVENTION_FORCE=mac
OUT="captures/item-487-magpie-diagonal"
FIX="$PWD/$OUT/fixture"
AWL=./target/debug/awl
DOC="$FIX/workspace/doc.md"
KEYS="Cmd-p t h e m e Ret"

shot() {
  name="$1"
  world="$2"
  dpi="$3"
  size="$4"
  gallery_var="$5"
  gallery_val="$6"
  if [ -n "$gallery_var" ]; then
    env "$gallery_var=$gallery_val" "$AWL" --screenshot "$OUT/$name.png" \
      --theme "$world" --capture-dpi "$dpi" --capture-size "$size" \
      --config "$FIX/awl.toml" --root "$FIX/workspace" "$DOC" --keys "$KEYS"
  else
    "$AWL" --screenshot "$OUT/$name.png" \
      --theme "$world" --capture-dpi "$dpi" --capture-size "$size" \
      --config "$FIX/awl.toml" --root "$FIX/workspace" "$DOC" --keys "$KEYS"
  fi
  printf '%s\n' "--- $name"
  python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print("driver:", d["driver"], " replay_skips:", d["replay_skips"], " mode:", d["overlay"]["mode"], " selected_index:", d["overlay"]["selected_index"])' "$OUT/$name.json"
}

for world_pair in "Magpie:magpie" "Mangrove:mangrove"; do
  world="${world_pair%%:*}"
  slug="${world_pair##*:}"
  for dpi_pair in "1:1x" "2:2x"; do
    dpi="${dpi_pair%%:*}"
    tag="${dpi_pair##*:}"

    # CURRENT (broken) state — every gallery var unset, byte-identical to a
    # live app run over this same fixture.
    shot "$slug-current-$tag" "$world" "$dpi" 1200x800 "" ""

    # Candidate 1a: full-canvas frost instead of the card's own footprint.
    shot "$slug-frost-full-$tag" "$world" "$dpi" 1200x800 \
      AWL_DIAGONAL_GALLERY_FROST full

    # Candidate 2: the footprint's top face seated above the first document
    # line (pivot-compensated — see gallery.rs's own doc on why a naive
    # rect edit would move the side faces too).
    shot "$slug-frost-top0-$tag" "$world" "$dpi" 1200x800 \
      AWL_DIAGONAL_GALLERY_FROST top0

    # Candidate 3: the selected row's chevron reach shortened to the row's
    # own measured name ink instead of the row's whole reserved cluster width.
    shot "$slug-chevron-short-$tag" "$world" "$dpi" 1200x800 \
      AWL_DIAGONAL_GALLERY_CHEVRON short

    # Candidate 1b: the query header right-aligned against the card's own
    # text column instead of seated at its left text edge.
    shot "$slug-query-right-$tag" "$world" "$dpi" 1200x800 \
      AWL_DIAGONAL_GALLERY_QUERY right
  done
done

# One WIDE-window pair (Magpie only, 1x) — the item's own motivation for the
# full-frost candidate ("a wide window leaves most of the document crisply
# readable beside frosted fragments"), which a 1200-wide capture cannot show.
shot "wide-current-1x" Magpie 1 1600x900 "" ""
shot "wide-frost-full-1x" Magpie 1 1600x900 AWL_DIAGONAL_GALLERY_FROST full

echo "--- pixel-arithmetic checks (non-vacuity — see measure.py's own module doc)"
python3 "$OUT/measure.py" "$OUT"
