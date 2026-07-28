#!/usr/bin/env bash
# Build the disposable item-133 Paperbark A–E review from real Awl captures.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
OUT="$ROOT/gallery/review/paperbark-trial"
ASSETS="$OUT/assets"
BIN="$ROOT/target/release/awl"
PROFILES="$ROOT/scripts/paperbark-trial/profiles.tsv"
FIXTURE="$ROOT/scripts/world-gallery-specimen.md"

echo "==> release build"
cargo build --release

rm -rf "$OUT"
mkdir -p "$ASSETS"
NO_CONFIG="$OUT/.unseeded-config.toml"

echo "==> real-app Retina captures"
while IFS=$'\t' read -r id slug label description; do
  [[ -z "$id" || "$id" == \#* ]] && continue
  lower="$(printf '%s' "$id" | tr '[:upper:]' '[:lower:]')"
  echo "    $id · $label · wide"
  AWL_PAPERBARK_TRIAL="$id" "$BIN" \
    --screenshot "$ASSETS/$lower-$slug-wide.png" \
    --capture-size 3600x2000 --capture-dpi 2 --measure 74 --page on \
    --theme Saltpan --keys "s-Down" --config "$NO_CONFIG" "$FIXTURE" >/dev/null
  echo "    $id · $label · narrow"
  AWL_PAPERBARK_TRIAL="$id" "$BIN" \
    --screenshot "$ASSETS/$lower-$slug-narrow.png" \
    --capture-size 1800x1400 --capture-dpi 2 --measure 58 --page on \
    --theme Saltpan --keys "s-Down" --config "$NO_CONFIG" "$FIXTURE" >/dev/null
done < "$PROFILES"

commit="$(git rev-parse HEAD)"
dirty=false
if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then dirty=true; fi

echo "==> offline comparison sheet + artifact laws"
node scripts/paperbark-trial/build.mjs \
  --root "$ROOT" --out "$OUT" --commit "$commit" --dirty "$dirty"
python3 scripts/paperbark-trial/verify_pixels.py "$OUT"

echo "==> done"
echo "    $OUT/index.html"
