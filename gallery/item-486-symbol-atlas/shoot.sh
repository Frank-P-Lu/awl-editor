#!/bin/sh
# Regenerate the symbol atlas from the repo root:
#   sh gallery/item-486-symbol-atlas/shoot.sh
#
# Writes gallery/item-486-symbol-atlas/symbol-atlas.html — SCRATCH, never
# committed (~50MB: every bundled face's raw .ttf bytes, base64-encoded, for
# the faces that carry at least one in-scope glyph). Open it directly in a
# browser; there is no server step.
#
# Deviates from captures/item-475's own convention (which drives the real
# `FontSystem`/GPU rendering path): this is a BROWSE-only inventory, so the
# generator embeds each bundled face as a browser `@font-face` `data:` URI
# instead — acceptable for shopping, never a substitute for the real
# pipeline. See README.md's "Known limitation" section before trusting any
# one face's rendered shape here.
set -eu
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
OUT="$PWD/gallery/item-486-symbol-atlas"
export AWL_SYMBOL_ATLAS_OUT="$OUT"

cargo test --bin awl symbol_atlas_gallery -- --ignored --nocapture
