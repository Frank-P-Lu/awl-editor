#!/usr/bin/env bash
#
# hero-image.sh — item 157: the site's dedicated social-sharing image, produced
# through the real product exactly like every other captured asset (see
# CAPTURE.md, capture-worlds.sh). No HTML mockup, no image editor, no network.
#
# It renders `scripts/hero-specimen.md` — the user-picked "write Markdown / now
# with lava lamps" composition under the `# awl` wordmark — through FOUR
# candidate worlds, each a genuinely different taste answer (see the queue-item
# report for the argument):
#
#   Saltpan  — the app's actual DEFAULT world (`theme::DEFAULT_THEME`): the
#              most "earned" answer, since it's what a first-time user
#              literally sees, unmodified.
#   Firetail — the roster's other STATEMENT world (THEMES.md): lava ground,
#              ember caret, the boldest/most "intriguing" answer.
#   Wagtail  — the true 1-bit world: maximum contrast, structurally immune to
#              getting muddy under heavy downscale/crop.
#   Bombora  — the roster's own "dark hero" reference (`site/style.css`'s
#              "dark is the default room (the Bombora hero is dark)" comment):
#              wave-tier ground, book serif, a genuine fourth contender, not a
#              substitute for the three the user named.
#
# item 157 ROUND 2 (this pass) fixed a real defect the first round shipped:
# the image was an EDITOR SCREENSHOT, not a composed image — CAPTURE.md's own
# page-mode chrome leaked into a "marketing" asset. Two leaks, one composition
# fix, all from REAL product settings (never a hand-picked filename or a
# pixel hack):
#
#   1. The bottom-left ORIENTATION GUTTER (render/chrome/gutter.rs) drew the
#      fixture's own filename ("hero-specimen.md") over its directory
#      ("scripts") — a scratch file's name has no business in a public asset.
#      `Buffer::display_name()` never actually returns an empty string
#      (verified, not assumed — CAPTURE.md's "an unnamed buffer" hint doesn't
#      hold up over the real CLI path), so the honest lever is the SAME hard
#      floor the gutter and the persistent OUTLINE share on purpose
#      (`GUTTER_MIN_NAME_CHARS`/`OUTLINE_MIN_CHARS`, `render/rowlayout.rs`):
#      below it, the margin is too narrow to bother, and the whole gutter
#      hides rather than draw a stub. Widening the writing column (MEASURE)
#      shrinks the margin below that floor — see the mutation-proof
#      calibration in `scripts/hero-verify.py --calibrate`.
#   2. A small second "awl" sat top-of-margin, next to the column — the
#      persistent margin OUTLINE (`src/outline.rs`, DEFAULT ON), echoing the
#      specimen's own "# awl" H1 as a nav label. Orientation chrome for
#      finding your place in a document has no job in a one-screen marketing
#      image, so it's turned off through its own real, documented config
#      sticky-pref (`outline = false`, `$HERO_CONFIG` below) — never a
#      rendering hack. Turning it off has a second, free effect: the
#      persistent-outline RAIL PUSH (`render/geometry.rs`'s
#      `adaptive_column_left`) stops shifting the column off-centre to grant
#      the outline its own rail, so the page recentres for free.
#   3. Composition: MEASURE 61 / ZOOM 1.9 clears the gutter/outline hide floor
#      across all four candidate fonts while preserving the approved large type
#      and four-row pause before the gag. `scripts/hero-verify.py` re-measures
#      the resulting crop and thumbnail contrast on every invocation, including
#      where it is still weak.
#
# Usage:
#   scripts/hero-image.sh                  # release build, render all 4 + comparison sheet
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
#                                                      embedding the four PNGs
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

# Pure built-in defaults regardless of the operator's own config (CAPTURE.md),
# EXCEPT the one deliberate override this item needs: the persistent margin
# OUTLINE off, through its own real sticky-pref config key (`src/outline.rs`,
# `Config::apply_sticky_globals`) — never a rendering hack. This is what
# removes the leaked second "awl" label AND recentres the column (the
# outline's own rail-push in `adaptive_column_left` stops firing once it's
# off) for free.
HERO_CONFIG="$RUN_DIR/.hero-config.toml"
printf 'outline = false\n' > "$HERO_CONFIG"

# The OG-canonical aspect ratio (1200x630, ~1.905:1 — "roughly 1.91:1").
# MEASURE 61 / ZOOM 1.9 shrinks the margin below the gutter/outline's shared
# hide floor on every candidate world's own font and preserves the approved
# composition. `End Left` parks the caret on `awl`'s `l`; Awl therefore
# reveals the heading marker on the active line, demonstrating its live-preview
# model in the image rather than explaining it in copy.
CANVAS="1200x630"
MEASURE=61
ZOOM=1.9
KEYS="End Left"

CANDIDATES=(Saltpan Firetail Wagtail Bombora)

for world in "${CANDIDATES[@]}"; do
  out_png="$CAND_DIR/$world.png"
  echo "==> candidate — $world"
  if ! "$BIN" --screenshot "$out_png" \
       --capture-size "$CANVAS" --measure "$MEASURE" --zoom "$ZOOM" --page on \
       --theme "$world" --config "$HERO_CONFIG" --keys "$KEYS" \
       "$SPECIMEN" >/dev/null; then
    echo "error: capture failed for candidate world '$world'" >&2
    exit 1
  fi
  got="$(grep -m1 '^  "theme":' "$CAND_DIR/$world.json" | grep -Eo '"name": "[^"]*"' | head -1 | sed -E 's/.*"([^"]*)"$/\1/')"
  if [[ "$got" != "$world" ]]; then
    echo "error: candidate '$world' sidecar reports theme '$got'" >&2
    exit 1
  fi

  # STATE + PIXEL verification (item 157 round 2): the sidecar's gutter.visible
  # is the state oracle, but the sidecar is never trusted alone for appearance
  # (CAPTURE.md's own tripwire — `selected_index: 2` once rendered on a fully
  # invisible row) — hero-verify.py backs it with real pixel arithmetic over
  # the exact region that leaked, calibrated non-vacuously against the actual
  # pre-fix candidates (`--calibrate`), plus reports (never gates — taste
  # stays the user's call) the OG/square-safe-area/thumbnail-contrast figures.
  if ! python3 "$SCRIPT_DIR/hero-verify.py" "$out_png" "$CAND_DIR/$world.json" "$world"; then
    echo "error: hero-verify.py failed for candidate '$world' — see the report above" >&2
    exit 1
  fi

  # Two-run byte-identical determinism: the same replay through the same real
  # keymap seam must be byte-for-byte reproducible (CAPTURE.md's determinism
  # contract) — asserted here, not assumed.
  rerun_png="$CAND_DIR/.$world.rerun.png"
  "$BIN" --screenshot "$rerun_png" \
       --capture-size "$CANVAS" --measure "$MEASURE" --zoom "$ZOOM" --page on \
       --theme "$world" --config "$HERO_CONFIG" --keys "$KEYS" \
       "$SPECIMEN" >/dev/null
  if ! cmp -s "$out_png" "$rerun_png"; then
    echo "error: candidate '$world' is NOT byte-identical across two runs" >&2
    exit 1
  fi
  rm -f "$rerun_png" "${rerun_png%.png}.json"
  echo "    two-run determinism: byte-identical (PASS)"
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
  --theme Saltpan --config "$HERO_CONFIG" \
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
