#!/usr/bin/env bash
# Mutation probes for scripts/check-web-site-assembly.sh's generated-asset law.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECK="$SCRIPT_DIR/check-web-site-assembly.sh"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/awl-site-assembly-test.XXXXXX")"
trap 'rm -rf "$SCRATCH"' EXIT

mkdir -p "$SCRATCH/editor"
printf '%s\n' '<script data-goatcounter="https://fluflu.goatcounter.com/count"></script><script src="/editor/awl-0123456789abcdef.js"></script><script src="/editor/awl-0123456789abcdef_bg.wasm"></script>' > "$SCRATCH/editor/index.html"
: > "$SCRATCH/editor/awl-0123456789abcdef.js"
: > "$SCRATCH/editor/awl-0123456789abcdef_bg.wasm"
"$CHECK" "$SCRATCH"

# The files exist, so this must fail specifically because `g` is not hexadecimal.
mv "$SCRATCH/editor/awl-0123456789abcdef.js" "$SCRATCH/editor/awl-0123456789abcdeg.js"
sed -i.bak 's/0123456789abcdef\.js/0123456789abcdeg.js/' "$SCRATCH/editor/index.html"
rm "$SCRATCH/editor/index.html.bak"
if "$CHECK" "$SCRATCH"; then
  echo "web-site-assembly test: accepted a non-hex generated JavaScript hash" >&2
  exit 1
fi
echo "web-site-assembly test: non-hex generated JavaScript hash rejected."
