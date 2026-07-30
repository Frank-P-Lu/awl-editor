#!/usr/bin/env bash
#
# hero-image.sh — item 157: the site's dedicated social-sharing image, produced
# through the real product exactly like every other captured asset (see
# CAPTURE.md, capture-worlds.sh). No HTML mockup, no image editor, no network.
#
# It renders `scripts/hero-specimen.md` — the product's own thesis sentence
# (PHILOSOPHY.md's opening line, minus the blockquote marker) under its own
# "# awl" wordmark — through THREE candidate worlds, each a genuinely different
# taste answer (see the queue-item report for the argument):
#
#   Saltpan  — the app's actual DEFAULT world (`theme::DEFAULT_THEME`): the
#              most "earned" answer, since it's what a first-time user
#              literally sees, unmodified.
#   Firetail — the roster's other STATEMENT world (THEMES.md): lava ground,
#              ember caret, the boldest/most "intriguing" answer.
#   Wagtail  — the true 1-bit world: maximum contrast, structurally immune to
#              getting muddy under heavy downscale/crop.
#
# Usage:
#   scripts/hero-image.sh                  # release build, render all 3 + comparison sheet
#   scripts/hero-image.sh --debug          # same, using the debug build
#   scripts/hero-image.sh --install World  # ALSO copy that one world's render to
#                                           # site/img/social.png (the wired asset) —
#                                           # the one-command way to change the pick
#
# Output (gallery/ is gitignored — replaceable, regenerate any time):
#   gallery/hero/candidates/<World>.png + .json   — one capture per candidate
#   gallery/hero/comparison.png + .json            — labeled side-by-side sheet,
#                                                      itself an awl capture of
#                                                      a small markdown doc
#                                                      embedding the three PNGs
#                                                      (same technique as
#                                                      capture-worlds.sh's
#                                                      contact sheets — no new
#                                                      external image tool).
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  for p in "$HOME/.cargo/bin" \
           "$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin"; do
    if [[ -x "$p/cargo" ]]; then export PATH="$p:$PATH"; break; fi
  done
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found on PATH. Install Rust (https://rustup.rs) or add cargo to PATH." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

PROFILE_FLAG="--release"
BIN="$ROOT/target/release/awl"
INSTALL_WORLD=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug)
      PROFILE_FLAG=""
      BIN="$ROOT/target/debug/awl"
      shift
      ;;
    --install)
      INSTALL_WORLD="${2:-}"
      if [[ -z "$INSTALL_WORLD" ]]; then
        echo "error: --install requires a world name" >&2
        exit 1
      fi
      shift 2
      ;;
    *)
      echo "error: unknown argument '$1'" >&2
      exit 1
      ;;
  esac
done

echo "==> building awl ($([[ -n "$PROFILE_FLAG" ]] && echo release || echo debug)) — first build can take several minutes"
# shellcheck disable=SC2086
cargo build $PROFILE_FLAG

SPECIMEN="$SCRIPT_DIR/hero-specimen.md"
if [[ ! -f "$SPECIMEN" ]]; then
  echo "error: missing specimen fixture $SPECIMEN" >&2
  exit 1
fi

RUN_DIR="$ROOT/gallery/hero"
CAND_DIR="$RUN_DIR/candidates"
rm -rf "$RUN_DIR"
mkdir -p "$CAND_DIR"

# Pure built-in defaults regardless of the operator's own config (CAPTURE.md).
NO_CONFIG="$RUN_DIR/.unseeded-config.toml"

# The OG-canonical aspect ratio (1200x630, ~1.905:1 — "roughly 1.91:1"), at a
# narrow measure so the world's own ground pattern shows generously on both
# sides (the roster's most distinctive asset, per DESIGN.md/THEMES.md), and a
# zoom that makes the wordmark + one sentence + caret read at real size rather
# than swimming in the 630px-tall frame. The caret parks at buffer end
# (`s-Down`, the same convention capture-worlds.sh uses for its Room shot) so
# the "# awl" heading line is off-caret and renders fully WYSIWYG-styled, not
# raw markdown.
CANVAS="1200x630"
MEASURE=40
ZOOM=1.5
KEYS="s-Down"

CANDIDATES=(Saltpan Firetail Wagtail)

for world in "${CANDIDATES[@]}"; do
  out_png="$CAND_DIR/$world.png"
  echo "==> candidate — $world"
  if ! "$BIN" --screenshot "$out_png" \
       --capture-size "$CANVAS" --measure "$MEASURE" --zoom "$ZOOM" --page on \
       --theme "$world" --config "$NO_CONFIG" --keys "$KEYS" \
       "$SPECIMEN" >/dev/null; then
    echo "error: capture failed for candidate world '$world'" >&2
    exit 1
  fi
  got="$(grep -m1 '^  "theme":' "$CAND_DIR/$world.json" | grep -Eo '"name": "[^"]*"' | head -1 | sed -E 's/.*"([^"]*)"$/\1/')"
  if [[ "$got" != "$world" ]]; then
    echo "error: candidate '$world' sidecar reports theme '$got'" >&2
    exit 1
  fi
done

echo "==> building comparison sheet"
THUMB_W=560
COMPARE_MD="$RUN_DIR/comparison.md"
{
  echo "# Hero image candidates — item 157"
  echo
  for world in "${CANDIDATES[@]}"; do
    echo "## $world"
    echo
    echo "![$world hero candidate|$THUMB_W](candidates/$world.png)"
    echo
    echo "---"
    echo
  done
} > "$COMPARE_MD"

COMPARE_W=$((THUMB_W + 200))
# Per-candidate block: heading + rule + the thumbnail at THUMB_W (native
# 1200x630 aspect, so height = THUMB_W*630/1200) + gaps, padded generously
# (measured against a real 3-candidate sheet, then rounded up).
BLOCK_H=$(( (THUMB_W * 630 / 1200) + 170 ))
COMPARE_H=$((160 + BLOCK_H * ${#CANDIDATES[@]} + 80))
"$BIN" --screenshot "$RUN_DIR/comparison.png" \
  --capture-size "${COMPARE_W}x${COMPARE_H}" --page off \
  --theme Saltpan --config "$NO_CONFIG" \
  "$COMPARE_MD" >/dev/null

echo
echo "==> done. candidates + comparison sheet under: $RUN_DIR"
find "$RUN_DIR" -maxdepth 2 -name "*.png" | sort

if [[ -n "$INSTALL_WORLD" ]]; then
  src="$CAND_DIR/$INSTALL_WORLD.png"
  if [[ ! -f "$src" ]]; then
    echo "error: --install '$INSTALL_WORLD' has no rendered candidate at $src (typo, or not in the CANDIDATES roster above?)" >&2
    exit 1
  fi
  dest="$ROOT/site/img/social.png"
  mkdir -p "$(dirname "$dest")"
  cp "$src" "$dest"
  echo "==> installed $INSTALL_WORLD -> $dest (site metadata is unchanged by this script; it always points at site/img/social.png)"
fi
