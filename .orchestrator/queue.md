# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16)

Design session 2026-08-16. The user works between a couple of files and wants
tabs' affordance without tabs: ⌃Tab covers two files but not three, and a
purely keyboard-summoned working set was judged too invisible for the widened
audience — for a non-programmer, tabs are intuitive because they are visible
and clickable. A persistent tab strip stays out (PHILOSOPHY §1 names it). The
shape chosen instead lives inside DESIGN §5's existing margin grammar: the
bottom-left identity (filename + folder — "position in the filesystem") WIDENS
into a quiet stack of the open files — current file in normal ink, the others
dimmer, click to switch. Not a new chrome region; an existing orientation
surface deepens by a line or two, and with one file open it is byte-identical
to today. The user chose bottom over above-the-outline.

What keeps it tabs-but-not-tabs: filenames only; no close affordance, no
reordering, no drag; capped at a handful (propose 5 — past the cap the answer
is ⌘O, and awl does not want twelve buffers); hides under space pressure
exactly as the outline does; inherits every §5 margin law (hug the column,
quiet label treatment, never change the prose column's geometry). STABLE OPEN
ORDER, not MRU — reordering on every switch would jitter the margin and make
the digit chords mean a different file each press; stable order makes ⌘2 a
NAME for the session.

Shortcuts (decided): ⌘1–⌘9 jump to the nth stack entry (slot 1; Linux
resolves Ctrl+1–9 per the normal rule — only ⌘0 is spent today, on reset
zoom). ⌃Tab Last-file stays EXACTLY as shipped — the alternate gesture,
untouched; no stack-cycling mode is added (cycling a stable list and toggling
MRU would fight, and digits + toggle cover every path in fewer keys). The
Emacs double is `C-x b`, aliased to Last file (faithful to `C-x b RET`), and
lands in which-key with the rest of the prefix for free. Optional flourish to
PROTOTYPE, not committed: while ⌘ is held for a beat, faint 1..n ordinals
beside the stack entries — the shortcut teaches itself at the moment of the
reach; cut it if it reads as noise in the shots.

Contract edits owned here: DESIGN §5's margin roster gains the stack as a
member with the outline's own license (may click-to-switch; orientation, not
management UI). PHILOSOPHY §1 is untouched — this is not a strip.

Capture-prototype FIRST, judgement before machinery: gallery shots across a
few worlds × {1, 2, 3 files open}, bottom-left, put to the user before laws
or bindings land. Harness reach (docs/harness-reach.md read for this clause):
`last_buffer` and the multi-file working set are App-owned — tier-1 captures
classify them Unsupported — so the prototype and every switching claim drive
`--screenshot-app`, which skips nothing and is hermetic (sandbox seeded from
the named CLI paths); that hermeticity also satisfies the path-leak rule,
because the margin photographs filenames — shots run against seeded roots
only, never the ambient ones. Verify: the sidecar gains the working set (open
files + active index) through the one redacting writer; a `--screenshot-app`
law drives open, open, ⌘2 and asserts the active buffer changed AND the
stack's drawn order did not; a pixel law that the active row is
distinguishable from its dimmed siblings, with a companion presence floor
(a stack faded to the page must fail, not pass happier); the one-file case
byte-identical to today's margin across the world roster; the digit chords in
both keymap flavors' tables so the generated reference picks them up — with
the generated rows spot-checked against the dispatch they claim.

## Needs a person, hardware, or release authority

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
2. **AT-SPI journey (251)** — on a real Linux desktop with Orca, exercise
   document reading, caret/selection, overlays, and an editing burst.
3. **Linux drawn-menu Export click** — use a real window/compositor and confirm
   the rendered menu's Export action reaches its destination.
4. **Linux v0.10.0 artifacts** — launch both the tarball and AppImage on a real
   desktop; check launcher name/icon and the AppImage FUSE fallback.
5. **macOS Export as PDF panel (301)** — confirm initial folder/name and that
   Cancel leaves the document untouched.
6. **Live glide (284)** — judge the 20° travel tilt and whether wrapping needs a
   distinct flourish.
