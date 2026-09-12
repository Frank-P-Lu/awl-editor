#!/usr/bin/env bash
#
# Audit the no-generated-editor-files rule. With an assembly directory argument,
# also prove that its fresh /editor/ bundle has the assets and public paths the
# static host needs.
#
# Usage: scripts/check-web-site-assembly.sh [assembled-site-directory]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ASSEMBLED="${1:-}"

if [ "$#" -gt 1 ]; then
  echo "usage: scripts/check-web-site-assembly.sh [assembled-site-directory]" >&2
  exit 2
fi

tracked="$(git -C "$ROOT" ls-files site/editor)"
if [ -n "$tracked" ]; then
  echo "generated editor bundle is tracked; site/editor must be empty:" >&2
  printf '%s\n' "$tracked" >&2
  exit 1
fi
echo "web-site-assembly: no generated editor files are tracked."

if [ -z "$ASSEMBLED" ]; then
  exit 0
fi

INDEX="$ASSEMBLED/editor/index.html"
if [ ! -f "$INDEX" ]; then
  echo "web-site-assembly: missing assembled editor/index.html" >&2
  exit 1
fi

is_hashed_js() {
  printf '%s\n' "$1" | grep -Eq '^/editor/awl-[[:xdigit:]]{16}\.js$'
}

is_hashed_wasm() {
  printf '%s\n' "$1" | grep -Eq '^/editor/awl-[[:xdigit:]]{16}_bg\.wasm$'
}

# The generated loader writes JS and wasm URLs as quoted /editor/ paths. Gather
# both forms, then require every referenced asset to exist under the same mount.
ASSETS="$(grep -Eo "['\"]/editor/[^'\"[:space:]]+\\.(js|wasm)" "$INDEX" | sed "s/^['\"]//" | sort -u || true)"
if [ -z "$ASSETS" ]; then
  echo "web-site-assembly: editor/index.html contains no /editor/ JS or wasm references" >&2
  exit 1
fi
js=0
wasm=0
hashed_js=0
hashed_wasm=0
while IFS= read -r asset; do
  case "$asset" in
    /editor/*.js) js=$((js + 1)) ;;
    /editor/*.wasm) wasm=$((wasm + 1)) ;;
  esac
  if is_hashed_js "$asset"; then hashed_js=$((hashed_js + 1)); fi
  if is_hashed_wasm "$asset"; then hashed_wasm=$((hashed_wasm + 1)); fi
  if [ ! -f "$ASSEMBLED${asset}" ]; then
    echo "web-site-assembly: missing referenced asset: $asset" >&2
    exit 1
  fi
done <<EOF
$ASSETS
EOF
if [ "$js" -eq 0 ] || [ "$wasm" -eq 0 ] || [ "$hashed_js" -eq 0 ] || [ "$hashed_wasm" -eq 0 ]; then
  echo "web-site-assembly: editor/index.html must reference both hashed JS and wasm under /editor/" >&2
  exit 1
fi
if ! grep -Fq 'data-goatcounter="https://fluflu.goatcounter.com/count"' "$INDEX"; then
  echo "web-site-assembly: editor/index.html lost the analytics beacon" >&2
  exit 1
fi
if grep -R -a -F -q "$HOME" "$ASSEMBLED/editor"; then
  echo "web-site-assembly: assembled editor bundle contains the builder home path" >&2
  exit 1
fi
echo "web-site-assembly: fresh /editor/ bundle has rooted JS/wasm assets, analytics, and no builder-home path."
