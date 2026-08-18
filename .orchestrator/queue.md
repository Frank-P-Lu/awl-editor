# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 459 — complete the ordinary-file vocabulary: Trash, Save a Copy, reveal/path, Go to line (USER DECISION 2026-08-18; ready to build in slices)

🟡 IN PROGRESS on slice 3 (Reveal/Copy Path) — claude, branch `claude/item-459-reveal-copy-path`
🟡 IN PROGRESS on slice 4 (Go to Line) — claude, branch `claude/item-459-goto-line`

awl is a complete home for ordinary Markdown files, and the current file verbs
stop one step short of that promise. Add the five agreed capabilities below as
one coherent filesystem-completeness pass. They are independently shippable;
do not hold the small platform/path and navigation slices behind the larger
destructive-file lifecycle work.

1. **Move file to Trash.** Add the missing recoverable sibling of Rename,
   Move and Duplicate. Use the OS Trash abstraction — never permanent deletion
   and never a home-grown trash folder. The command targets the named buffer
   when invoked from item 444's working-set context menu and the active document
   from the palette; both doors dispatch one action. Run the existing
   save/external-change/conflict gate first. A dirty or conflicted document is
   never discarded or overwritten. A successful trash closes/removes that
   buffer and selects the same successor the close owner would. **Dependency:**
   trashing the final open document waits for item 444 residual 2's honest
   zero-document state; until then, either land only the non-final case with a
   truthful refusal for the last document or sequence the full slice after that
   residual. Confirmation is owed only where the document is dirty/conflicted;
   an ordinary clean file goes directly to the recoverable OS Trash.

2. **Save a Copy…, not Save As….** Write the current Markdown bytes to a
   user-chosen destination while preserving the current document's path,
   working-set identity, history ownership, autosave target, cursor and undo
   timeline. This is the useful Markdown "export" behavior without inventing
   Export Markdown or ambiguously overlapping Rename/Move. No-clobber and
   overwrite confirmation follow the platform save-panel contract. A copy is a
   snapshot, not a newly active document and not a second live buffer unless the
   user opens it later.

3. **Reveal in File Manager + Copy File Path.** Add both to the palette and
   item 444's shared filename context menu. On macOS the visible label may read
   **Reveal in Finder**; elsewhere use the platform-neutral file-manager label.
   Reuse the existing live-only reveal owner currently exercised by exports,
   generalized to any document path. Copy File Path puts the absolute native
   path on the system clipboard. Both commands are gated off for an unnamed
   scratch document rather than fabricating a location. They perform no file
   mutation and never change the active folder.

4. **Go to Line….** Add a line-number route inside the existing unified **Go
   to…** task rather than a parallel top-level navigation system. Accept a
   one-based line number, clamp or clearly refuse out-of-range input, move the
   caret through the shared jump owner, reveal any enclosing fold, and scroll
   the destination into view. Preserve the existing Headings lens; this is its
   numeric companion for long prose and light code.

Product boundary: no permanent file tree, bulk deletion, folder deletion,
cross-root file management, Export Markdown command, or second document model.
The filesystem stays real and understandable. The context menu is a discovery
door onto the same catalog actions, never a second implementation.

Verify each slice at its purest reachable seam. Trash uses an injected fake
Trash backend and sweeps active/parked × clean/dirty/conflicted ×
last/non-last, proving a failed Trash leaves both disk and working set intact.
Save a Copy drives a live headless `App` over `InMemoryFs` and asserts the
destination bytes while every source identity/state field remains unchanged.
Reveal is live-only and must remain suppressed on a headless surface, matching
the export law; path-copy asserts exact native clipboard text and the unnamed
gate. Go to Line sweeps first/middle/last/out-of-range lines, wrapped text,
Unicode and a folded destination through `--keys`, with the sidecar proving
caret and scroll state. Read `docs/platform.md` and `docs/harness-reach.md`
before implementing or promising captures; render-touching slices receive the
standing vision-smoke and DPI/world audit required by policy.

### 458 — 🔴 Credits is BROKEN, not merely the wrong size: it renders as a one-row palette over a blurred page (USER-REPORTED 2026-08-18; ready to build)

🟡 IN PROGRESS — claude, branch `claude/item-458-workspace-predicate`

**Premise revised.** This item was carried as a taste divergence ("full
workspace, not the mini window asked for"). The user opened ⌘P → Credits and
saw neither: a `credits ›` search line, one row reading `credits`, the
`↑/↓ scroll ⌫ back` footer, and the CREDITS text only as the frosted BLUR
behind the card. Reproduced headlessly with a real `--screenshot-app`
capture (`s-p c r e d i t s Enter`, seeded root): the sidecar reports
`overlay.workspace: true`, `detail_focus: true`, `preview_view: "document"`,
`items: ["credits"]`, and the PNG shows the same one-row card over blurred
credits prose — the content region is never opened.

**Root cause (read, not fixed — the user asked for the diagnosis to be
queued):** the render side's `overlay_is_workspace()`
(`src/render/chrome/workspace.rs`) is `self.overlay_workspace &&
!self.overlay_lens.is_empty()` — a workspace is recognised by its LENS
STRIP, a gate written when Settings was the only workspace and History (a
faceting kind) was the second. `OverlayKind::Credits` is registered
lens-less in `facets.rs` (as is `Conflict`; "a lens over one/three fixed
rows would be a strip with nothing to narrow"). So on every Credits frame
`overlay_is_workspace()` is false: `workspace_primary_w` measures 0,
`comparison_viewport()` returns `None`, `transcript_parked()` is false, and
the pushed CREDITS transcript falls back to the ordinary page column UNDER
the card, where the overlay's blur frosts it — exactly the picture. The
sidecar's `workspace: true` comes from the App-side
`workspace_shape().is_some()` (`viewstate.rs`), so the two halves of the
product disagree about what a workspace is, and the sidecar oracle
reported the intended shape while the pixels drew a picker.

**Why every law stayed green:** the workspace laws seed
`v.overlay_lens = ov.lens_strip()` from History/Settings only — no render
law drives `new_credits()` or `new_conflict()` through the workspace
geometry — and 452's presence/legibility law asserted CREDITS ink was
visible somewhere on the canvas, which the frosted fallback satisfies. The
same defect must be checked on the Conflict workspace (also lens-less;
its capture reach is via `preview_id`/`preview_view`): if it draws the
same one-column card, it has been broken since it landed and its harness
photos never showed the two-region shape.

Decided fix shape: the renderer's workspace predicate follows the SHAPE,
not the lens — `overlay_workspace` alone (the App already derives it from
`workspace_shape()`), with the lens strip an optional header decoration a
`TimelineOverComparison` workspace may or may not have. Then
`measure_workspace_primary_w` measures the rows (it already handles
`rows_primary` from `overlay_items`), the content pane opens, and the
transcript relocates into it. Keep the deep-link (`toggle_detail` on
`OpenCredits`) so Credits still lands focused on the content.

Verify: a `--screenshot-app` law over `s-p c r e d i t s Enter` asserts
`comparison_viewport()` is `Some` (or, at the sidecar seam, that the
transcript is drawn INSIDE the card's content pane: sample the PNG for
CREDITS ink at unblurred contrast within the pane rect and NONE at the
page column outside the card) — the frosted-fallback frame must FAIL that,
so run it against `main` first and watch it go red. Add `new_credits()`
and `new_conflict()` to the render workspace-law fixtures so the
lens-less members sweep the same geometry laws History does (enrolment
derived from the roster: every kind whose `workspace_shape()` is `Some`,
no named list). Rust change: full gate. Then, and only then, the original
taste question — full workspace vs. smaller floating card — can be put to
the user, because today they cannot see either.

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16; STACK RENDERS, SWITCHES AND CLOSES; AFFORDANCE OWED NEXT)

**Landed on `main`** (full sha list in `git log --grep 'item 444'`): the
`--seed-tree` capture door; the `WorkingSet` module; the cross-root ownership
fix; the resting-stack render (N=1 byte-identical by construction, 40/40
proven non-vacuously); sidecar exposure + click-to-switch (`buffers` gains
`files[]`/`active_index`, `SCHEMA_VERSION` 203→204, through the same
`App::load_path` door every picker/daemon handoff shares); and now **⌘W as
the one true removal owner** (`9213f16d`). `App::close_buffer(key)`
(`src/app/files/close.rs`) is reached by both ⌘W and a stack row's close
zone, resolving to the active entry or a parked one without the caller
choosing. Three pieces that didn't exist before now do, all law-tested with
captured mutation panics: `DocumentSession::entry_unsaved` (save, generalized
off any entry, not just the active one), a parked-entry conflict gate that
**refuses** rather than latching into the single unresolved-conflict slot
(pointing that slot at a parked path would let a later "Save your version"
overwrite the wrong document), and `notify_close_waiters(key)` (daemon
notify, generalized off the active buffer). Closing the last file saves and
notifies but removes nothing — that's residual 1 below, not a bug.

**New gap this landing opens, sequencing decided 2026-08-18 (merge now,
affordance next):** the stack row's right-edge close zone is now WIRED to
`close_buffer` — always lossless (saves first, refuses on conflict) — but
still draws NOTHING. No chrome hover-tracking exists anywhere in the margin
outline (it's click-only too), so a click near a row's right edge now
silently closes that file instead of switching to it, with no visual cue
distinguishing the zone. Landed anyway on the user's explicit call, with the
hover-reveal × treatment owed as the next residual, not deferred indefinitely.

**Two smaller findings, still carried forward:**
- `gutter` in the sidecar (the single name/project fact) is stale
  *documentation* once the stack draws at N≥2 — its doc claims "exactly as
  drawn" but the pixels show a whole stack. Whoever next touches sidecar
  `buffers`/`gutter` reconciles the doc, or the field, deliberately.
- The scratch buffer is closeable (dirty scratch refuses like any other
  entry, the successor search skips it) but still has no *activation* door
  anywhere (`load_path` takes a path; `previous_path()` returns
  `Option<PathBuf>`) — a scratch row still silently swallows a switch click.
  Feeds residual 2 (zero-document), which needs the same "no path yet"
  reasoning anyway.

**Two facts worth knowing before the next residual touches this area:**
`Finish file` is now a misnomer for a command that CLOSES — renaming it
touches the palette, GUIDE, REFERENCE and the `finish_file` config key, so
only its *description* was corrected this round, not its name. And
`load_path` flushes autosave before parking, so under the default config a
parked entry is essentially never dirty — the parked-conflict path is mainly
reachable via `autosave = false`.

**Residual, in the order the landed work sets up:**

1. **Hover-reveal close affordance + folder-line distinction (USER-REPORTED
   2026-08-18 on the live app). (b) row cursor and (d) Wagtail legibility
   are LANDED** (`f8f3fb4c`): stack rows now earn the pointing hand via the
   same `CursorContext` roster/no-wildcard law the outline rows already use
   (`gutter_stack_hit`, no parallel hit-test), and the active row's label
   ink routes through `theme::selected_row_secondary_ink`/`surface_selected`
   — the same "ink over a filled plate inverts" mechanism `one_bit.rs`
   already proves for the picker/toast — so Wagtail's selected file is
   legible again, presence + legibility floors both mutation-proven
   independently. (a) and (c) remain open, below.

   (a) The × is invisible: the row's right close zone is wired to
   `close_buffer` but draws nothing — the user hovered and asked "I don't
   see the x mark?" Draw the × (or equivalent) when the pointer enters the
   zone; capture-prototype and put shots to the user before committing to a
   treatment. New live-only render axis (hover), so `cursor_shape.rs`-style
   unit laws + a live check. **Decided 2026-08-18: hover-only (never a
   persistent column), on the RIGHT — the ink is right-aligned so the right
   end is the one stable edge across filename lengths, and `close_zone` is
   already the rightmost row-height square.** The × occupies the plate's
   right pad / the space past the ink's right edge; it must not shift the
   label (no hover jitter). Prototype for the user: one-stage reveal
   (× on row-hover) vs two-stage (faint × on row-hover, full ink inside
   the close zone), and active-row-with-× vs siblings-only (⌘W already
   closes the active file).
   (c) The FOLDER line under the stack (`notes`) reads as a third file: it
   is drawn at the same LABEL size and the same `faint` ink as the inactive
   rows, so `awl-start.md / anxiety-2.md / notes` scan as three siblings.
   The user asked whether the type scale has a smaller step — the chrome
   scale has `LABEL` (the margin's one size) and the rotated location's
   `LOCATION_SCALE 0.92`; a smaller size for the folder line is a DESIGN
   §5 call (the identity's "position in the filesystem" was two lines of
   one size when it was one file). Prototype at least two treatments and
   put both to the user: a smaller/quieter step for the folder line, and a
   spacing/heading treatment (folder as heading ABOVE the rows, which the
   444 text already names: "the root heading names the active folder").
   Byte-identity for the one-file case must hold whichever wins.

2. **Zero-document state** — the largest remaining piece; `DocumentSession`
   would need an optional active slot, and every subsystem this item names
   (renderer, actions, autosave, session, title, accessibility tree,
   sidecar) needs an honest `no active document` representation.
3. **Overflow windowing, expanded/grouped cross-project view, Move
   navigator** — deliberately last: this item's own text asks for captures
   judged by the user before any of these get built. The resting stack now
   renders, switches and closes, so a >5-file / cross-project capture set can
   finally be produced — but the exact windowing/grouping rule still needs
   the user's eyes on that set before it's built, not guessed ahead of it.

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
