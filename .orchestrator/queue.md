# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 447 — Kite's page frame must meet the top of the canvas (USER-REPORTED REGRESSION 2026-08-16)

In page mode, Kite's dark writing-column frame starts at the document text
origin rather than at the top of the editor canvas. The screenshot shows a
plain horizontal strip between the native title bar and the frame's top edge;
the two vertical rails begin only where that inset top edge lands. A page frame
describes the writing SURFACE, so it must meet the canvas top (or the bottom of
awl's drawn menu bar on platforms where that persistent chrome exists), while
the prose keeps its existing top inset inside the frame.

Premise check `page_frame_vertical_bounds`: its unscrolled cases currently
choose `doc_top` (typically the text inset) as `top`, and the law explicitly
expects that gap. Separate the frame's surface boundary from document-row
geometry. Do not move `doc_top`, text, headings, outline entries, or hit-testing
upward to disguise the defect; only the page-frame rectangle owns this change.
Scrolled and short documents must resolve to the same canvas-owned frame.

Verify the pure bounds owner over no-menu and drawn-menu configurations, then
prove real pixels for every world whose `PageFrame` is `Line` (currently Kite
and Wagtail): the top horizontal edge and both vertical rails touch the first
legal canvas row, the frame never paints through a drawn menu bar, and no
unframed page-colour strip survives above it. Sweep page width, viewport height,
scroll position, DPI, and menu-bar state. Add a companion identity assertion
that the first text row remains at its pre-fix Y and a roster-wide absence law
for `PageFrame::None` worlds. Mutating the bound back to `doc_top` must fail on
the exact top-gap assertion.

### 446 — copy must survive a buffer switch (USER-REPORTED REGRESSION 2026-08-16)

Copy text in one file, switch to another open file, then Paste: nothing is
inserted. Copy/Paste are system-clipboard operations and must cross buffer
boundaries. The defect is in the live clipboard bridge, not the platform
clipboard: `sync_kill_to_clipboard` writes the copied text and records it in
App-global `clipboard_last_written`; after a switch the destination buffer has
its own empty kill ring, but `refresh_kill_from_clipboard` sees that the OS text
equals `clipboard_last_written` and returns early without hydrating the NEW
buffer. `YankText` then reads that buffer's empty kill ring.

Make the duplicate-read optimization conditional on the active buffer already
holding the same text, or move clipboard text ownership to the App so a buffer
swap cannot invalidate the premise. Keep the system clipboard authoritative:
external clipboard changes still replace the internal value; an empty or
non-text clipboard retains the documented graceful fallback; copy/kill
coalescing inside a buffer remains unchanged. Do not solve this by copying kill
rings between buffers during every switch—the clipboard bridge, not document
state, owns cross-document paste.

Verify with a live-App law using a controllable clipboard seam: select and Copy
in buffer A, activate buffer B, Paste, and assert B receives the exact text in
one undoable edit while A remains unchanged. Sweep both switch doors (Last file
and direct activation), multiline and multibyte text, a destination selection
that must be replaced, and an external clipboard overwrite between switch and
Paste. Add the non-vacuity case that reproduces today's state exactly: OS text
equals App `clipboard_last_written` while the active buffer's kill ring is
empty; Paste must still succeed. Preserve the existing same-buffer redundant
write/read suppression laws.

### 445 — inline Code must round-trip a multi-line selection (USER-REPORTED REGRESSION 2026-08-16)

The contextual formatting popover's **Code** button does not toggle cleanly
when the active selection spans two or more lines. First activation inserts
the inline-code backticks around the selected text; activating **Code** again
leaves those backticks in the document. The same action round-trips correctly
for a single-line selection. Premise check first: reproduce through the real
popover dispatch (`PopoverButton::Code` → `Action::InlineCode`) and compare it
with the keyboard/palette route, because every surface is meant to share
`actions::format` rather than own formatting behavior.

Fix the shared inline-format toggle, not the popover alone. A selection already
wrapped by the command must unwrap on the second invocation even when its byte
range contains newlines, with one undo step per invocation and the selection
remaining on the same logical content. Do not silently reinterpret the button
as Code Block: this report is about the existing inline Code action removing
the delimiters it inserted. Preserve the shipped single-line behavior and do
not disturb fenced-code toggling.

Verify at the pure formatting seam and through `apply_transition`: selections
spanning two complete lines, partial text on both boundary lines, an empty
middle line, multibyte text, and a selection ending at column zero of the next
line all wrap then unwrap to the exact original bytes. Add a real popover-route
law proving two presses dispatch the same action and round-trip the buffer, plus
an undo/redo assertion and the existing single-line and Code Block cases as
regression guards. The law must fail against the current behavior before the
fix lands.

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

## Needs specific hardware

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.

## Needs release authority

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
