#!/usr/bin/env bash
# THE OFFLINE ICON EXPORT — the whole per-world app-icon pipeline, ahead of time.
#
#   scripts/export-icons.sh [--out DIR] [--check] [--only SUBSTR]
#
# Four steps, each one a separate program so each can be re-run alone:
#
#   1. `cargo run -- --icon-manifest`   the theme-derived manifest: every world's
#      icon palette + display face, straight out of `theme::THEMES`, and every
#      face's bundled font FILE + real weight, read from the .ttf name tables.
#      Nothing here is hand-maintained; retune a world and the icons follow.
#   2. `scripts/icons/build.mjs`        manifest + tuning -> self-contained HTML
#      with the fonts inlined as data: URLs.
#   3. `scripts/icons/render.mjs`       one pinned, offline Chromium renders
#      every tile and gallery sheet.
#   4. `cargo run -- --pack-icns`       cut each world's tiles, at the ONE
#      preset its world literal assigns, into a real `.icns`; write the
#      canonical bundle icon; regenerate `src/app_icon/embedded.rs`. Pure
#      Rust — no `iconutil`, so the container has one owner and `cargo test`
#      can re-pack a committed asset and demand identical bytes.
#
# NO BROWSER IN THE BUILD: this script is the ONLY thing that runs Chromium. An
# ordinary `cargo build`, `cargo test`, and the shipping app never invoke it —
# the app only ever reads the PNGs this produced and committed.
#
# ZERO NETWORK: the browser is a pinned local revision (never downloaded here),
# launched with networking disabled, loading file:// pages whose fonts are
# base64 data: URLs from `assets/fonts`. Run it in airplane mode; it will not
# notice.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT="assets/macos/candidates"
BUILD="target/icon-export"
CHECK=0
ONLY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --build) BUILD="$2"; shift 2 ;;
    --only) ONLY="$2"; shift 2 ;;
    --check) CHECK=1; shift ;;
    -h|--help) sed -n '2,25p' "$0"; exit 0 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done

command -v node >/dev/null || { echo "node is required (Node 22+ for global WebSocket)" >&2; exit 1; }

mkdir -p "$BUILD" "$OUT"

echo "==> manifest (from theme::THEMES + assets/fonts)"
cargo run --quiet -- --icon-manifest > "$BUILD/manifest.json"

echo "==> pages"
node scripts/icons/build.mjs --manifest "$BUILD/manifest.json" --out "$BUILD/html" --fonts assets/fonts

echo "==> render"
if [ -n "$ONLY" ]; then
  node scripts/icons/render.mjs --build "$BUILD/html" --out "$OUT" --only "$ONLY"
else
  node scripts/icons/render.mjs --build "$BUILD/html" --out "$OUT"
fi

echo "==> pixel checks"
python3 scripts/icons/verify.py \
  --manifest "$BUILD/manifest.json" \
  --tiles "$OUT/tiles" \
  --report "$OUT/legibility.txt" \
  --geometry-report "$OUT/geometry.txt"

# --- 4. PACK (Rust, no browser, no `iconutil`) ------------------------------
# Cut each SHIPPED world's tiles — at the ONE preset its world literal assigns
# (`Theme::icon_cursor`) — into a real multi-representation `.icns`, write the
# canonical bundle icon (the DEFAULT world's), and regenerate the embedded
# table `src/app_icon/embedded.rs`. Skipped when only a subset was rendered
# (`--only`), since the pack needs every size.
if [ -z "$ONLY" ]; then
  echo "==> pack (.icns per world + the canonical bundle icon)"
  cargo run --quiet -- --pack-icns "$OUT/tiles"
else
  echo "==> pack SKIPPED (--only renders a subset; the pack needs every size)"
fi

if [ "$CHECK" = "1" ]; then
  echo "==> determinism: second render into a scratch tree, hashes compared"
  SECOND="$BUILD/recheck"
  rm -rf "$SECOND"
  node scripts/icons/render.mjs --build "$BUILD/html" --out "$SECOND" ${ONLY:+--only "$ONLY"}
  ( cd "$OUT" && find . -name '*.png' | sort | xargs shasum -a 256 ) > "$BUILD/hashes-a.txt"
  ( cd "$SECOND" && find . -name '*.png' | sort | xargs shasum -a 256 ) > "$BUILD/hashes-b.txt"
  if diff -q "$BUILD/hashes-a.txt" "$BUILD/hashes-b.txt" >/dev/null; then
    echo "    identical: $(wc -l < "$BUILD/hashes-a.txt" | tr -d ' ') PNGs, sha256 of the set:"
    shasum -a 256 "$BUILD/hashes-a.txt" | awk '{print "    " $1}'
  else
    echo "NOT DETERMINISTIC — differing files:" >&2
    diff "$BUILD/hashes-a.txt" "$BUILD/hashes-b.txt" >&2 || true
    exit 1
  fi
fi

echo "done -> $OUT"
