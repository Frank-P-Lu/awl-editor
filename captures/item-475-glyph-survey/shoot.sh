#!/bin/sh
# Reproduce this item's gallery sheets from the repo root:
#   sh captures/item-475-glyph-survey/shoot.sh
#
# Deviates from captures/item-444's own convention (which drives the real
# `awl --screenshot-app` binary): this survey has NO production wiring to
# drive — the whole point is that no shipped code path draws a candidate
# glyph. Instead it runs the real `rotated_label` mechanism (the same
# `LabelMask` + `RotatedLabelPipeline` code production will draw through once
# a candidate is picked) through an `#[ignore]`d unit test, gated a second way
# on AWL_FOLD_MARK_GALLERY_OUT so an ordinary `cargo test` — filtered or not —
# never writes gallery files.
set -eu
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
OUT="$PWD/captures/item-475-glyph-survey"
export AWL_FOLD_MARK_GALLERY_OUT="$OUT"

cargo test --bin awl fold_mark_candidate_gallery -- --ignored --nocapture

echo "--- direction-at-rest law (non-vacuous proof the turn actually rotates each candidate)"
cargo test --bin awl fold_mark_candidates_settle -- --nocapture
