#!/usr/bin/env bash
# Regenerate the public themes page and every image on it from the product.
# The ordered roster comes only from `awl --list-worlds`, a printer over
# `theme::THEMES`; neither this script nor the generated HTML names worlds.
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"
BIN="${AWL_BIN:-$ROOT/target/debug/awl}"
SPECIMEN="$SCRIPT_DIR/public-world-gallery-specimen.md"
OUT_DIR="$ROOT/site/img/worlds"
PAGE="$ROOT/site/themes.html"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/awl-public-worlds.XXXXXX")"
CAPTURE_DIR="$SCRATCH/captures"
trap 'rm -rf "$SCRATCH"' EXIT

if [[ ! -x "$BIN" ]]; then
  echo "error: build awl first, or set AWL_BIN to an executable" >&2
  exit 1
fi

mapfile_compat() {
  while IFS= read -r line; do
    [[ -n "$line" ]] && worlds+=("$line")
  done
}
worlds=()
mapfile_compat < <("$BIN" --list-worlds)
if (( ${#worlds[@]} == 0 )); then
  echo "error: --list-worlds returned an empty roster" >&2
  exit 1
fi
dupes="$(printf '%s\n' "${worlds[@]}" | sort | uniq -d)"
if [[ -n "$dupes" ]]; then
  echo "error: --list-worlds returned duplicate names: $dupes" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
mkdir -p "$CAPTURE_DIR"
find "$OUT_DIR" -type f -name '*.png' -delete

NO_CONFIG="$SCRATCH/empty-config.toml"
: > "$NO_CONFIG"
for world in "${worlds[@]}"; do
  echo "capture: $world"
  "$BIN" --screenshot "$CAPTURE_DIR/$world.png" \
    --capture-size 1200x900 --capture-dpi 1 --measure 58 --page on \
    --theme "$world" --config "$NO_CONFIG" --keys s-Down \
    "$SPECIMEN" >/dev/null

  json="$CAPTURE_DIR/$world.json"
  got="$(python3 - "$json" <<'PY'
import json
import pathlib
import sys

print(json.loads(pathlib.Path(sys.argv[1]).read_text())["theme"]["name"])
PY
)"
  if [[ "$got" != "$world" ]]; then
    echo "error: $world capture sidecar reports theme '$got'" >&2
    exit 1
  fi
  cp "$CAPTURE_DIR/$world.png" "$OUT_DIR/$world.png"
done

{
  cat <<'HTML'
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Worlds — awl</title>
  <meta name="description" content="The worlds of awl, rendered by the editor itself.">
  <link rel="icon" type="image/png" sizes="32x32" href="favicon.png">
  <meta property="og:title" content="Worlds — awl">
  <meta property="og:description" content="The worlds of awl, rendered by the editor itself.">
  <meta property="og:type" content="website">
  <meta property="og:url" content="https://awl-editor.fly.dev/themes.html">
  <meta property="og:image" content="https://awl-editor.fly.dev/img/social.png">
  <meta name="twitter:card" content="summary_large_image">
  <meta name="twitter:image" content="https://awl-editor.fly.dev/img/social.png">
  <link rel="stylesheet" href="style.css">
  <script data-goatcounter="https://fluflu.goatcounter.com/count" async src="//gc.zgo.at/count.js"></script>
</head>
<body>
  <header class="site-nav themes-nav">
    <a class="nav-mark" href="/" aria-label="awl home">awl</a>
    <nav aria-label="Site">
      <a href="/">Home</a>
      <a href="/editor/">Try</a>
      <a href="/guide.html">Guide</a>
      <a href="/reference.html">Reference</a>
    </nav>
  </header>
  <main class="worlds-page">
    <header class="worlds-intro">
      <p class="eyebrow">Worlds</p>
      <h1>Worlds in awl.</h1>
      <p>Each world changes the palette, type, margins, and chrome without changing the document. These are captures from awl itself.</p>
    </header>
    <div class="world-grid">
HTML
  for world in "${worlds[@]}"; do
    printf '      <figure class="world-card" data-world="%s">\n' "$world"
    printf '        <a href="img/worlds/%s.png"><img src="img/worlds/%s.png" alt="The canonical Markdown document rendered in the %s world" loading="lazy" width="1200" height="900"></a>\n' "$world" "$world" "$world"
    printf '        <figcaption>%s</figcaption>\n' "$world"
    printf '      </figure>\n'
  done
  cat <<'HTML'
    </div>
  </main>
</body>
</html>
HTML
} > "$PAGE"

echo "wrote ${#worlds[@]} ordered worlds to site/themes.html and site/img/worlds/"
