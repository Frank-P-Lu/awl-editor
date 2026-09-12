#!/usr/bin/env bash
#
# Assemble the static landing site and a fresh Trunk bundle into an untracked
# scratch directory.  Deploy and local preview both use this owner so /editor/
# is exercised exactly as it will be served, without generated files entering
# site/ or Git.
#
# Usage: scripts/assemble-web-site.sh <new-output-directory>
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT="${1:-}"

if [ -z "$OUTPUT" ] || [ "$#" -ne 1 ]; then
  echo "usage: scripts/assemble-web-site.sh <new-output-directory>" >&2
  exit 2
fi
if [ -e "$OUTPUT" ]; then
  echo "error: output directory already exists: $OUTPUT" >&2
  exit 2
fi
if [ ! -f "$ROOT/dist/index.html" ]; then
  echo "error: missing dist/index.html; run scripts/with-remap.sh trunk build --release --public-url /editor/ first" >&2
  exit 2
fi
if [ -n "$(find "$ROOT/site/editor" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null || true)" ]; then
  echo "error: site/editor must remain absent; generated editor files belong only in the scratch assembly" >&2
  exit 2
fi

mkdir -p "$(dirname "$OUTPUT")"
mkdir "$OUTPUT"
cp -R "$ROOT/site/." "$OUTPUT"
rmdir "$OUTPUT/editor" 2>/dev/null || true
mkdir "$OUTPUT/editor"
cp -R "$ROOT/dist/." "$OUTPUT/editor"

echo "assembled web site: $OUTPUT"
