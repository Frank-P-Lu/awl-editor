#!/usr/bin/env bash
# Item 191 — BEFORE/AFTER sheet for Bowerbird's `Finds` spacing/size tuning.
#
# "Before" is item 176's dormant `Finds` preview exactly as it landed (cell
# pitch 156, the original anchor radius band, the original unconstrained
# per-cell dropout) — not the old `Masses` control, which is a separate,
# already-approved decision (item 176) that this round does not revisit.
# "After" is the current tree: the anchor/companion/cut-out composition grown
# ~15% as one move, the cell pitch opened separately to 195, and the dropout
# gate replaced with the decorrelated, void-bounded one
# (`finds_is_local_min` in shaders/background.wgsl).
#
# Mirrors `scripts/capture-bowerbird-revival.sh`'s mechanism: patch the
# handful of literals that differ, build, capture, restore (an EXIT trap
# restores even on interrupt) — nothing here is a trial selector or an env
# knob left behind in the product.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

RUN_DIR="$ROOT/gallery/bowerbird-spacing-191"
CAPTURES="$RUN_DIR/captures"
WORLDS="$ROOT/src/theme/worlds.rs"
SHADER="$ROOT/shaders/background.wgsl"
SPECIMEN="$ROOT/scripts/world-gallery-specimen.md"
BIN="$ROOT/target/release/awl"
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"

KEEP_WORLDS="$(mktemp -t awl-item191-worlds)"
KEEP_SHADER="$(mktemp -t awl-item191-shader)"
cp "$WORLDS" "$KEEP_WORLDS"
cp "$SHADER" "$KEEP_SHADER"
restore() {
  cp "$KEEP_WORLDS" "$WORLDS"
  cp "$KEEP_SHADER" "$SHADER"
  rm -f "$KEEP_WORLDS" "$KEEP_SHADER"
}
trap restore EXIT

rm -rf "$RUN_DIR"
mkdir -p "$CAPTURES"

capture_arm() {
  local arm="$1"
  "$BIN" --screenshot "$CAPTURES/$arm-wide-1x.png" --capture-size 1440x900 \
    --measure 74 --theme Bowerbird --config "$NO_CONFIG" --keys 's-Down' \
    "$SPECIMEN" >/dev/null
  "$BIN" --screenshot "$CAPTURES/$arm-narrow-1x.png" --capture-size 900x900 \
    --measure 38 --theme Bowerbird --config "$NO_CONFIG" --keys 's-Down' \
    "$SPECIMEN" >/dev/null
  "$BIN" --screenshot "$CAPTURES/$arm-wide-2x.png" --capture-size 2880x1800 \
    --capture-dpi 2.0 --measure 74 --theme Bowerbird --config "$NO_CONFIG" \
    --keys 's-Down' "$SPECIMEN" >/dev/null
  "$BIN" --screenshot "$CAPTURES/$arm-narrow-2x.png" --capture-size 1800x1800 \
    --capture-dpi 2.0 --measure 38 --theme Bowerbird --config "$NO_CONFIG" \
    --keys 's-Down' "$SPECIMEN" >/dev/null
}

echo "building AFTER (the current tree — item 191's tuning)..."
cargo build --release >/dev/null
capture_arm after

echo "building BEFORE (item 176's dormant Finds preview: pitch 156, the original anchor band, the original unconstrained dropout)..."
python3 - "$WORLDS" "$SHADER" <<'PY'
import pathlib, sys
worlds_path, shader_path = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])

w = worlds_path.read_text()
old, new = "scale_px: 195.0,", "scale_px: 156.0,"
assert w.count(old) == 1, "expected exactly one Bowerbird scale_px to revert"
worlds_path.write_text(w.replace(old, new))

s = shader_path.read_text()
subs = [
    ("const FINDS_ANCHOR_LO: f32 = 0.1725;", "const FINDS_ANCHOR_LO: f32 = 0.150;"),
    ("const FINDS_ANCHOR_HI: f32 = 0.22425;", "const FINDS_ANCHOR_HI: f32 = 0.195;"),
    ("const FINDS_DROPOUT: f32 = 0.226;", "const FINDS_DROPOUT: f32 = 0.10;"),
    (
        "    if (h0 < FINDS_DROPOUT && finds_is_local_min(cell, h0)) {\n        return g.c_from.rgb;\n    }\n",
        "    if (h0 < FINDS_DROPOUT) {\n        return g.c_from.rgb;\n    }\n",
    ),
]
for old, new in subs:
    assert s.count(old) == 1, f"expected exactly one occurrence of: {old!r}"
    s = s.replace(old, new)
shader_path.write_text(s)
PY
cargo build --release >/dev/null
capture_arm before

restore
trap - EXIT
echo "restoring the AFTER (shipped) build..."
cargo build --release >/dev/null

sheet="$RUN_DIR/sheet.md"
{
  echo "# Bowerbird — Finds spacing/size, before item 191's tuning vs after"
  echo
  echo "BEFORE is item 176's dormant \`Finds\` preview exactly as landed (156px"
  echo "cell, the original anchor band, the original unconstrained per-cell"
  echo "dropout). AFTER is the current tree (composition ~15% larger as one"
  echo "hierarchy-preserving move, cell pitch opened separately to 195px, and"
  echo "the dropout gate replaced with the decorrelated, void-bounded one)."
  echo
  echo "## Narrow page, 1:1 — measure 38, 900x900"
  echo
  echo "### Before"
  echo
  echo "![before narrow|900](captures/before-narrow-1x.png)"
  echo
  echo "### After"
  echo
  echo "![after narrow|900](captures/after-narrow-1x.png)"
  echo
  echo "## Wide page — measure 74, 1440x900"
  echo
  echo "### Before"
  echo
  echo "![before wide|960](captures/before-wide-1x.png)"
  echo
  echo "### After"
  echo
  echo "![after wide|960](captures/after-wide-1x.png)"
  echo
  echo "## Retina — 2x device ratio, narrow page (shown at its logical size)"
  echo
  echo "### Before"
  echo
  echo "![before narrow 2x|900](captures/before-narrow-2x.png)"
  echo
  echo "### After"
  echo
  echo "![after narrow 2x|900](captures/after-narrow-2x.png)"
  echo
  echo "## Retina — 2x device ratio, wide page (shown at its logical size)"
  echo
  echo "### Before"
  echo
  echo "![before wide 2x|960](captures/before-wide-2x.png)"
  echo
  echo "### After"
  echo
  echo "![after wide 2x|960](captures/after-wide-2x.png)"
} > "$sheet"

"$BIN" --screenshot "$RUN_DIR/bowerbird-spacing-191-comparison.png" \
  --capture-size 1040x6600 --page off --theme Saltpan --config "$NO_CONFIG" \
  "$sheet" >/dev/null

echo "wrote $RUN_DIR/bowerbird-spacing-191-comparison.png"
