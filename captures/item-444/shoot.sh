#!/bin/sh
# Reproduce this item's capture set from the repo root:  sh captures/item-444/shoot.sh
# Hermetic: the sandbox is seeded from the named fixture tree only, never the
# ambient project or the ambient config, so nothing photographs a real path.
set -eu
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
export AWL_CONVENTION_FORCE=mac
OUT="captures/item-444"
FIX="$PWD/$OUT/fixture"
AWL=./target/debug/awl

shot() {
  name="$1"
  shift
  "$AWL" --screenshot-app "$OUT/$name.png" \
    --seed-tree "$FIX" --config "$FIX/awl.toml" --root "$FIX/notes" \
    "$FIX/notes/opening.md" "$@"
  printf '%s\n' "--- $name"
  python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print("driver:", d["driver"]); print("buffers:", json.dumps(d["buffers"])); print("gutter:", json.dumps(d["gutter"]))' "$OUT/$name.png".json 2>/dev/null \
    || python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print("driver:", d["driver"]); print("buffers:", json.dumps(d["buffers"])); print("gutter:", json.dumps(d["gutter"]))' "$OUT/$name.json"
}

# One file open: no stack, and the sidecar says so.
shot one-file

# Three files open, the last-opened active.
shot three-files --keys "Cmd-o l e d Enter Cmd-o f i e Enter"

# The same three, switched back to the FIRST — the drawn order must not move.
shot three-files-switched --keys "Cmd-o l e d Enter Cmd-o f i e Enter Cmd-o o p e n Enter"
