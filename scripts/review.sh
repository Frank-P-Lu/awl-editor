#!/usr/bin/env bash
# Build the offline visual-review artifact from real awl captures.
#
#   scripts/review.sh           # release build
#   scripts/review.sh --debug   # faster iteration
#
# Output: gallery/review/index.html + relative PNG/JSON assets.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROFILE="release"
BIN="$ROOT/target/release/awl"
if [[ "${1:-}" == "--debug" ]]; then
  PROFILE="debug"
  BIN="$ROOT/target/debug/awl"
elif [[ $# -gt 0 ]]; then
  echo "usage: scripts/review.sh [--debug]" >&2
  exit 2
fi

OUT="$ROOT/gallery/review"
SCRATCH="$ROOT/target/review-build"
echo "==> world Rooms + Frames ($PROFILE)"
if [[ "$PROFILE" == "debug" ]]; then
  scripts/capture-worlds.sh --debug
else
  scripts/capture-worlds.sh
fi

rm -rf "$OUT"
mkdir -p "$OUT/assets/worlds" "$OUT/assets/scenes" "$OUT/assets/icons" "$SCRATCH"
cp -R "$ROOT/gallery/worlds/room" "$OUT/assets/worlds/room"
cp -R "$ROOT/gallery/worlds/frame" "$OUT/assets/worlds/frame"
cp "$ROOT/gallery/worlds/contact-light.png" "$OUT/assets/worlds/contact-light.png"
cp "$ROOT/gallery/worlds/contact-dark.png" "$OUT/assets/worlds/contact-dark.png"

NO_CONFIG="$OUT/.unseeded-config.toml"
echo "==> canonical important screens"
while IFS=$'\t' read -r id label theme canvas measure keys fixture capture_mode expect description; do
  [[ -z "$id" || "$id" == \#* ]] && continue
  png="$OUT/assets/scenes/$id.png"
  echo "    $label"
  args=(--screenshot "$png" --capture-size "$canvas" --measure "$measure" --page on
    --theme "$theme" --config "$NO_CONFIG")
  [[ "$keys" != "—" ]] && args+=(--keys "$keys")
  case "$capture_mode" in
    normal) "$BIN" "${args[@]}" "$fixture" >/dev/null ;;
    popover) AWL_POPOVER=1 "$BIN" "${args[@]}" "$fixture" >/dev/null ;;
    diff)
      AWL_DIFF_OLD="$ROOT/scripts/review/diff-old.md" \
      AWL_DIFF_NEW="$ROOT/scripts/review/diff-new.md" \
      AWL_DIFF_TITLE="Comparing dashboard copy" \
        "$BIN" "${args[@]}" "$fixture" >/dev/null
      ;;
    *) echo "unknown capture mode '$capture_mode' for $id" >&2; exit 2 ;;
  esac
done < "$ROOT/scripts/review/scenes.tsv"

echo "==> current shipped icon sheets (no pack, no committed-asset writes)"
"$BIN" --icon-manifest > "$SCRATCH/icon-manifest.json"
cp "$SCRATCH/icon-manifest.json" "$OUT/assets/icons/manifest.json"
node scripts/icons/build.mjs \
  --manifest "$SCRATCH/icon-manifest.json" \
  --out "$SCRATCH/icon-html" \
  --fonts assets/fonts
node scripts/icons/render.mjs \
  --build "$SCRATCH/icon-html" \
  --out "$OUT/assets/icons" \
  --only shipped

commit="$(git rev-parse HEAD)"
dirty=false
if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then dirty=true; fi

echo "==> offline HTML index"
node scripts/review/build.mjs \
  --root "$ROOT" \
  --out "$OUT" \
  --commit "$commit" \
  --dirty "$dirty"

echo "==> done"
echo "    $OUT/index.html"
