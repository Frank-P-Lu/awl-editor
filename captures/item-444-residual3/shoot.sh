#!/bin/sh
# Reproduce this item's capture set from the repo root:  sh captures/item-444-residual3/shoot.sh
#
# THROWAWAY PROTOTYPE EVIDENCE, not the shipped feature. Item 444 residual 3
# ("the working set becomes visible") needs a user pick on the overflow
# windowing rule and the cross-project grouped view before either ships. These
# shots audition the ALREADY-BUILT capture-only prototype door
# (`src/workingset/prototype.rs`, gated by `AWL_WORKING_SET_PROTOTYPE` and
# read only under `--screenshot-app`) against a multi-root working set. No
# production render or hit-test code changed to produce this gallery — see
# README.md's "What this gallery does and does not demonstrate".
#
# Hermetic: the sandbox is seeded from `fixture/` alone, through --seed-tree,
# with an explicit --config and --root -- never the ambient project or the
# ambient config -- so nothing here photographs a real directory. The PNGs
# and their sidecars are scratch and are not committed; the fixture and this
# script are, so the set survives the worktree that produced it (mirrors
# captures/item-444/README.md's own note).
set -eu
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
export AWL_CONVENTION_FORCE=mac
OUT="captures/item-444-residual3"
FIX="$PWD/$OUT/fixture"
AWL=./target/debug/awl

# Ten files under the "notebook" root, opened in stable order, then three
# under the sibling "atlas" root reached through the real Switch-project
# picker (Cmd-Shift-P) -- so the working set the prototype projects is the
# one a reader would actually accumulate, not a hand-built fixture struct.
# Every Cmd-o (`s-o`) steps to the Files lens (`Right`) first: with
# `file_visibility = true` (load-bearing -- see README) the All lens also
# ranks folder rows, and this worktree's own absolute path fuzzy-matches
# short queries because it is nested under a dot-directory
# (.claude/worktrees/...) whose ancestor chain pollutes corpus-order ranking.
OPEN10="s-o Right l e d g Enter s-o Right i d e a Enter s-o Right t o d o Enter s-o Right d r a f Enter s-o Right p l a n Enter s-o Right r e v i Enter s-o Right a r c h Enter s-o Right i n d e Enter s-o Right e n t r Enter "
TO_ATLAS="s-S-p Down Enter s-o Right r e a d Enter s-o Right n o t e Enter s-o Right s e t u Enter "
BACK_OPENING="s-S-p Down Down Enter s-o Right o p e n Enter"
BACK_ENTRY="s-S-p Down Down Enter s-o Right e n t r Enter"
# entry.md, then plan.md (already open, already visible in entry.md's own
# window) -- probes whether the candidate rule re-anchors a file that never
# left the resting five.
BACK_ENTRY_THEN_PLAN="s-S-p Down Down Enter s-o Right e n t r Enter s-o Right p l a n Enter"

shot() {
  name="$1"
  mode="$2"
  scroll="$3"
  theme="$4"
  keys="$5"
  if [ -n "$theme" ]; then
    theme_args="--theme $theme"
  else
    theme_args=""
  fi
  AWL_WORKING_SET_PROTOTYPE="$mode" AWL_WORKING_SET_PROTOTYPE_SCROLL="$scroll" \
    "$AWL" --screenshot-app "$OUT/$name.png" \
    --seed-tree "$FIX" --config "$FIX/awl.toml" --root "$FIX/workspace/notebook" \
    $theme_args "$FIX/workspace/notebook/opening.md" --keys "$keys"
  printf '%s\n' "--- $name"
  python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print("driver:", d["driver"]); print("prototype:", json.dumps(d["buffers"]["prototype"]))' "$OUT/$name.json"
}

# Resting stack: five files plus the one "+N more..." row. `total_open` is 13
# (10 notebook + 3 atlas); the "+N more" count is EVERY hidden open buffer,
# same-root overflow and other roots alike, by the queue item's own spec --
# not a bug if it reads larger than "10 minus 5".
shot collapsed-opening-active collapsed 0 "" "$OPEN10$TO_ATLAS$BACK_OPENING"

# Same window rule, but the ACTIVE file is the group's LAST slot (entry.md) --
# the resting five slide to keep it visible instead of always showing the
# first five.
shot collapsed-entry-active collapsed 0 "" "$OPEN10$TO_ATLAS$BACK_ENTRY"

# Then activate plan.md, which was ALREADY visible (top row) in the previous
# shot's window. The candidate rule is stateless -- it re-derives the window
# from the active index alone -- so it re-anchors plan.md to the BOTTOM of a
# window that jumps four slots, even though nothing forced it off screen.
shot collapsed-jitter collapsed 0 "" "$OPEN10$TO_ATLAS$BACK_ENTRY_THEN_PLAN"

# Switch away and stop: the active root is now atlas (3 files, all visible,
# no overflow row needed within THIS root) -- demonstrating "only the active
# folder's group shows at rest" independent of the windowing rule above.
shot collapsed-atlas-active collapsed 0 "" "$OPEN10$TO_ATLAS"

# Expanded: an 8-row scrollable window over notebook's 10 files, unscrolled
# and scrolled to its far end.
shot expanded-scroll0 expanded 0 "" "$OPEN10$TO_ATLAS$BACK_OPENING"
shot expanded-scroll2 expanded 2 "" "$OPEN10$TO_ATLAS$BACK_OPENING"

# Grouped cross-project view: every open file under BOTH roots, headed by
# each root's own name, the active root's heading legible over the other's.
shot grouped-saltpan grouped 0 "" "$OPEN10$TO_ATLAS$BACK_OPENING"

# The same grouped view under a second world, for contrast (Magpie: a light
# horizontal-band ground -- the "bars" personality).
shot grouped-magpie grouped 0 Magpie "$OPEN10$TO_ATLAS$BACK_OPENING"

# The resting stack under a third, structurally different world (Gumtree: a
# diagonal zigzag ground -- the "diagonal/ruled" personality), so the reader
# judges the rows against more than one ground treatment.
shot collapsed-gumtree collapsed 0 Gumtree "$OPEN10$TO_ATLAS$BACK_OPENING"
