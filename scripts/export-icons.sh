#!/usr/bin/env bash
# THE OFFLINE ICON EXPORT — the whole per-world app-icon pipeline, ahead of time.
#
#   scripts/export-icons.sh [--out DIR] [--check] [--only SUBSTR] [--timeout-ms N]
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
#      every tile and gallery sheet. Every wait in it is named and bounded;
#      `scripts/icons/render-laws.mjs` runs first (below) and holds it to that.
#      This step once stopped answering mid-capture and said nothing at all.
#   4. `cargo run -- --pack-icns`       cut each world's tiles, at the ONE
#      preset its world literal assigns, into a real `.icns`; write the
#      canonical bundle icon; regenerate `src/app_icon/embedded.rs`. Pure
#      Rust — no `iconutil`, so the container has one owner and `cargo test`
#      can re-pack a committed asset and demand identical bytes.
#   5. install the DEFAULT world's 32px favicon into the static site. Every
#      world's paired favicon remains available under candidates/favicons.
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
# Per-stage budget for the renderer's named waits; raise it on a loaded machine
# rather than going back to a run that can hang without saying where.
TIMEOUT_MS=""
while [ $# -gt 0 ]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --build) BUILD="$2"; shift 2 ;;
    --only) ONLY="$2"; shift 2 ;;
    --timeout-ms) TIMEOUT_MS="$2"; shift 2 ;;
    --check) CHECK=1; shift ;;
    -h|--help) sed -n '2,27p' "$0"; exit 0 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done

command -v node >/dev/null || { echo "node is required (Node 22+ for global WebSocket)" >&2; exit 1; }

mkdir -p "$BUILD" "$OUT"

# The renderer's guards, proven before a minutes-long export leans on them:
# a drained browser stderr, a named stall on the call that hangs, a teardown
# that leaves no process and no scratch profile. Twenty seconds.
echo "==> renderer laws"
node scripts/icons/render-laws.mjs

echo "==> manifest (from theme::THEMES + assets/fonts)"
cargo run --quiet -- --icon-manifest > "$BUILD/manifest.json"

echo "==> pages"
node scripts/icons/build.mjs --manifest "$BUILD/manifest.json" --out "$BUILD/html" --fonts assets/fonts

echo "==> render"
if [ -n "$ONLY" ]; then
  node scripts/icons/render.mjs --build "$BUILD/html" --out "$OUT" --only "$ONLY" ${TIMEOUT_MS:+--timeout-ms "$TIMEOUT_MS"}
else
  node scripts/icons/render.mjs --build "$BUILD/html" --out "$OUT" ${TIMEOUT_MS:+--timeout-ms "$TIMEOUT_MS"}
fi

echo "==> pixel checks"
python3 scripts/icons/verify.py \
  --manifest "$BUILD/manifest.json" \
  --tiles "$OUT/tiles" \
  --favicons "$OUT/favicons" \
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
  # The pack writes its table one entry per line; rustfmt wraps the longer ones.
  # Without this the export is not idempotent — the .icns bytes come back
  # identical while `embedded.rs` shows an 18-line diff that is pure formatting,
  # and a re-export looks like a change nobody made.
  rustfmt --edition 2024 src/app_icon/embedded.rs
  DEFAULT_WORLD="$(sed -n 's/.*DEFAULT_THEME: usize = world_index("\([^"]*\)").*/\1/p' src/theme/worlds.rs)"
  test -n "$DEFAULT_WORLD"
  cp "$OUT/favicons/$DEFAULT_WORLD-32.png" site/favicon.png
else
  echo "==> pack SKIPPED (--only renders a subset; the pack needs every size)"
fi

if [ "$CHECK" = "1" ]; then
  echo "==> determinism: second render into a scratch tree, hashes compared"
  SECOND="$BUILD/recheck"
  rm -rf "$SECOND"
  node scripts/icons/render.mjs --build "$BUILD/html" --out "$SECOND" ${ONLY:+--only "$ONLY"} ${TIMEOUT_MS:+--timeout-ms "$TIMEOUT_MS"}
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
