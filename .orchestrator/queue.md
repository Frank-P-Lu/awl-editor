# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 448 — the resting caret needs a more confident body (USER-REPORTED VISUAL DEFECT 2026-08-16)

🟡 IN PROGRESS — claude, branch claude/item-448-caret-body

The caret in ordinary body prose reads a touch too small beside the character
it occupies. In the reported shot it is unmistakably present, but the rounded
accent body feels tucked inside the lowercase letter rather than standing as
the clearest point of presence promised by DESIGN §4. This is a modest visual
size correction, not a request for a different caret style or more animation.

Premise-check the actual rendered form before tuning: reproduce the shot with
the settled caret over a lowercase ascender and identify whether the active
form is Block or Morph from the sidecar/config rather than guessing from the
colour. Measure its top, bottom, width and area against the row's real prose
ink. The current proportional-cell owner deliberately uses one stable height
per `(face, row)` so the body does not jump between `a`, `l`, punctuation and
space; preserve that law. Its real-pixel presence floor currently accepts a
caret at only half the row's rendered ink height, which is a regression guard,
not proof that the authored proportion feels substantial enough.

Tune the shared caret geometry owner so the resting cell body is modestly
larger and still feels like it occupies one character. Do not add a Kite-only,
font-only or glyph-only exception; do not size each anchor from its own raster
box; and do not change the I-beam merely to make an exhaustive enum sweep move.
Preserve baseline alignment, descender clearance, Morph's inhabited-glyph
treatment, mono-grid stability, CJK cells, heading scaling, zoom/DPI scaling,
motion settling and the single amber accent. Width, height, padding and corner
radius must be judged together—a taller narrow lozenge or an over-wide pill is
not the requested result.

Verify in rendered pixels across the world roster at 1x/2x DPI and representative
zoom, covering proportional and mono faces, lowercase x-height/ascender/
descender anchors, punctuation, space/end-of-line, CJK, headings, and Block plus
Morph. Strengthen the presence oracle with measured before/after proportions
while retaining the one-height-per-face/row and no-neighbouring-line-overlap
laws; mutating the tuned body back to its current dimensions must fail the new
visual-presence assertion. Land the modest shared tuning on main for judgement
(reverting is one small geometry commit), then show the user about five ordinary
body-prose crops across light/dark and proportional/mono worlds and ask only
whether the caret now has enough presence.

### 447 — Kite's page frame must meet the top of the canvas (USER-REPORTED REGRESSION 2026-08-16)

🟡 IN PROGRESS — claude, branch claude/item-447-kite-page-frame-top

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

🟡 IN PROGRESS — claude, branch claude/item-446-clipboard-buffer-switch

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

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16)

🟡 IN PROGRESS — claude, branch claude/item-444-margin-buffer-stack

Design session 2026-08-16. The user works between a couple of files and wants
tabs' affordance without tabs: ⌃Tab covers two files but not three, and a
purely keyboard-summoned working set was judged too invisible for the widened
audience — for a non-programmer, tabs are intuitive because they are visible
and clickable. A persistent tab strip stays out (PHILOSOPHY §1 names it). The
shape chosen instead lives inside DESIGN §5's existing margin grammar: the
bottom-left identity (filename + folder — "position in the filesystem") WIDENS
into a quiet stack of the open files — current file on a soft Arc-like selected
row, the others dimmer, click to switch. Not a new chrome region; an existing
orientation surface deepens by a line or two, and with one file open it is
byte-identical to today. The user chose bottom over above-the-outline, and chose
the visibly row-shaped Arc-like treatment over bare labels floating in the
margin.

The stack is PROJECT-SCOPED, not one global pile of arbitrary paths. The active
folder remains load-bearing: it owns Go to's recursive file corpus, New
document, Move and export destinations, git identity, and the session's folder
context. All files below that root may coexist in the stack, including files in
different subfolders; a nested file reads by its shortest unambiguous
ROOT-RELATIVE path (`journal/field-notes.md`), not by a leaf name that throws
away the location. Switching folders through Go to changes which stack is
visible, while each folder's working set stays parked and returns when that
folder becomes active again — the Arc-Space shape, without a permanent space
switcher. Opening or activating a buffer from another root must restore that
buffer's remembered project context in the same transition: the current
registry can preserve cross-root buffers while `load_path` leaves the old root
active, which would make the document and bottom folder identity disagree.
Close that seam here; never present it as intentional mixed-root behavior.

Nested files render as a FLAT working set, never a miniature tree. The root
heading names the active folder (`notes`); each row shows the filename and, when
it lives below the root, its root-relative parent in quieter ink
(`journal/ field-notes.md`). Preserve the leaf and the nearest useful location
when space is tight (`research/…/ final-draft.md`). No indentation, disclosure
arrows or expandable folders: those marks promise a file tree this surface does
not provide.

What keeps it tabs-but-not-tabs: no reordering and no drag; hides under space
pressure exactly as the outline does; inherits every §5 margin law (hug the
column, quiet label treatment, never change the prose column's geometry).
STABLE OPEN ORDER within a project, not MRU — reordering on every switch would
jitter the margin while the pointer is reaching for a row.

Closing is part of the feature, not an assumed affordance. Today ⌘W's
"Finish file" saves, notifies any daemon waiter, and switches to the previous
buffer but leaves the finished buffer parked; that is not close. Give the
working set one true removal owner: ⌘W closes the active file after the same
lossless save/conflict gate and still notifies a waiter, and a row exposes the
same action through a quiet hover-only close target (never a persistent column
of crosses). The close target appears when the pointer enters the row's RIGHT
close zone; the rest of the row remains the larger switch target, so merely
moving through the list does not fill it with controls. Closing an inactive row
closes that named buffer without first activating it; a dirty or conflicted
buffer is never discarded.

The stack makes the existing file verbs discoverable without becoming their
implementation. Secondary-click a row to open the SAME filename context menu
the bottom identity already owns: Rename file…, Move file…, Duplicate file,
Version history…, plus Close file. Pointer and keyboard routes dispatch the
same catalog actions; Cmd-P → Move file… opens the identical destination
navigator. A row action targets the named buffer rather than silently switching
documents merely to make an active-buffer-only function convenient.

Move stays deliberately bounded to the source file's owning root. Its summoned
folders-only navigator says `move <filename>`, shows the current root-relative
destination, descends/ascends through folders, offers an explicit `New folder…`
row and a `Move here` action at every level. A successful move keeps the stack
slot stable and updates its quiet parent path. No drag-to-move, bulk selection,
folder moves or cross-root moves in this item: those are file-manager machinery,
and a tiny contextual stack is the wrong place to imply them. Moving never
silently rewrites Markdown or incoming links; when the file contains relative
links/images, the completion feedback states that their paths may need review.

Closing the LAST file enters a real ZERO-DOCUMENT state, not a fake unnamed
buffer and not a closed application window. Today `DocumentSession` always owns
one active `Entry`, so this is product machinery rather than empty-state copy:
the renderer, actions, autosave, session, title, accessibility tree and sidecar
must all represent `no active document` honestly. The world remains; the page
surface disappears because drawing blank paper would imply an unsaved file. A
small calm start surface offers exactly `New document` and `Go to…`; the active
folder remains remembered, so either route has an unambiguous context. First
launch still opens the authored Welcome document — zero-document is reached by
an explicit close, never used as a replacement tour.

Visible overflow is bounded independently of the registry's safety cap.
PROTOTYPE five file rows plus a quiet `+ N more…` row. Accepting it EXPANDS the
same bottom-anchored stack UPWARD into a transient scrollable working-set view;
it does not detour through Go to, permanently lengthen the resting margin, or
turn the whole window into a sidebar. Wheel/trackpad motion over the expanded
list scrolls that list, Esc/click-away collapses it, and choosing a file returns
to the resting stack. The active file must always be represented in the visible
five; prototype the least-jittering stable-order window and put the >5 shots to
the user before fixing its exact windowing rule. Do not silently evict a sixth
file merely to make the drawing easy: the existing registry's
clean-LRU/never-dirty eviction is a memory safety bound, not a visible-stack
product rule.

PROTOTYPE the cross-project half in that SAME expanded view, taking the useful
part of Codex's sidebar grammar without adopting its permanent project-manager
shell: group retained open files under folder headings; mark the active file's
group clearly; keep only the active folder's group in the resting stack. The
ONE generic `+ N more…` row counts every hidden open buffer — same-root overflow
and other roots alike — and expands this grouped view; never add a parallel
`N files in other folders…` row. Clicking a file in another group atomically
restores its remembered project root AND activates its buffer, after which Go
to, New, Move, export and the resting stack all operate in that folder. This is
why the main folder remains meaningful: it is the ACTIVE group, not a claim that
no other folder may retain buffers. Do not show multiple groups persistently;
awl is not a project manager.

No new digit shortcuts. Once the working set can be grouped, partially hidden
and scrolled, “the third file” has no stable, obvious meaning across resting and
expanded states. ⌃Tab Last file stays exactly as shipped, `C-x b` may remain its
quiet Emacs alias, and Go to… remains the complete keyboard route.

Contract edits owned here: DESIGN §5's margin roster gains the stack as a
member with the outline's own license (may click-to-switch; orientation, not
management UI). PHILOSOPHY §1 is untouched — this is not a strip.

Capture-prototype FIRST, judgement before machinery: gallery shots across a
few worlds × {one root with nested files, only one file in the active root,
more than five files, zero documents}, bottom-left, including right-edge
hover-close, collapsed overflow, expanded scrolling and grouped cross-project
states, put to the user before laws or bindings land. Harness reach
(docs/harness-reach.md read for this clause):
`last_buffer` and the multi-file working set are App-owned — tier-1 captures
classify them Unsupported — so the prototype and every switching claim drive
`--screenshot-app`, which skips nothing and is hermetic (sandbox seeded from
the named CLI paths); that hermeticity also satisfies the path-leak rule,
because the margin photographs filenames — shots run against seeded roots
only, never the ambient ones. Verify: the sidecar gains the working set (open
files + active index) through the one redacting writer; a `--screenshot-app`
law opens A, opens B, accepts A's stack row and asserts the active buffer
changed AND the stack's drawn order did not. A pixel law asserts that the
active row is
distinguishable from its dimmed siblings, with a companion presence floor
(a stack faded to the page must fail, not pass happier); cross-root activate
restores the matching project/root before the frame and the gutter never names
the old root; nested same-root files render distinct relative labels; close
removes exactly its target and never loses a dirty/conflicted buffer; overflow
keeps the active file represented and its `+ N more…` count exact; expanded
scroll remains inside the working set; closing the last file produces no active
buffer, no page surface and exactly the two start actions without changing the
remembered root; the one-file case before close is byte-identical to today's
margin across the world roster; context-menu Move and palette Move dispatch the
same action; moving a nested file keeps its stack slot and updates its relative
label, and never crosses the source root. Generated reference rows are
spot-checked against the dispatch they claim.

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
