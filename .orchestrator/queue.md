# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 460 — context menu: omit unavailable commands instead of tagging them "unavailable" (USER DECISION 2026-08-19; 🟡 IN PROGRESS — item-460-context-menu (codex), branch codex/item-460-context-menu)

The right-click card shows state-gated commands as disabled rows carrying a
literal `unavailable` secondary label (`context_menu::overlay`,
`src/context_menu.rs`). A plain body right-click with no selection therefore
reads `Cut — unavailable / Copy — unavailable` above Paste. The user's call:
that makes no sense — a command that cannot apply here is simply not shown.
The menu contains only what works, now. (macOS native menus grey such items
out; this drawn card instead spells a word, and omission was chosen over
imitating the grey.)

The disabled roster today, read from `context_menu::rows`: the `Body`
target's Cut/Copy (`has_selection` is false by construction — a selection
routes to the `Selection` target, where both are always live), and the
`Filename` target's four file verbs when the document is unnamed
(`named_file` false). Under the decision: Body yields exactly Paste +
Select all; an unnamed document's filename target yields no rows.

Mechanism:
- `rows()` stays the one owner of the roster; it returns only applicable
  rows.
- Empty rows ⇒ no menu, owned in ONE place. The gutter summon branch already
  suppresses an empty card (`src/app/input/context_menu.rs`) while the
  document and heading branches summon unconditionally — move the suppression
  into the shared summon/overlay seam so no path can open an empty card.
- With no producer of disabled rows left, cut the dead machinery rather than
  leaving a dormant second path: `ContextRow.enabled`, the `None` slots the
  overlay stores in `context_actions`, and the `unavailable` secondary
  writer. If the lane finds a genuine near-term need for an informative
  disabled row, keep the field and say so in the landing note; the default is
  cut.
- Named consequence: the accessory-ink law
  (`src/render/tests/accessory_ink.rs`) proves per-row secondary ink on the
  one overlay kind that combines zero header rows with a non-empty secondary
  column — precisely the context card's `unavailable` tags. Removing the tags
  removes that law's production subject; retire or re-target it deliberately
  in the same landing, never incidentally.
- Boundary: availability remains state-gated capability (selection present,
  file named). No clipboard sniffing to gate Paste, no per-platform changes
  beyond the existing web omissions.

Verify at the purest reachable seam: unit laws over `rows()` sweeping
target × state × platform — no cell returns an inapplicable row, Body with no
selection is exactly Paste + Select all, unnamed Filename is empty — plus a
law at the summon seam proving an empty rows list opens no card on ANY
branch (the pre-fix document branch must fail it). Prove non-vacuity by
mutation. The card is pointer-summoned; read `docs/harness-reach.md` before
promising any capture of it. Rust change: full gate.

### 459 — complete the ordinary-file vocabulary: Trash, Save a Copy, reveal/path, Go to line (USER DECISION 2026-08-18; slice 2 🟡 IN PROGRESS — item-459-save-copy (codex), branch codex/item-459-save-copy)

🟢 LANDED — slice 3 (Reveal in File Manager + Copy File Path), `cdd5a1bc`
🟢 LANDED — slice 4 (Go to Line), `0d95045d`

**Slice 4 landed on `main` (`0d95045d`).** Go to Line is a numeric row inside
the existing unified Go to… overlay, not a parallel navigation system: it
carries the destination buffer's line count (`OverlayState::goto_line_count`
/ `attach_line_jump`), refreshes its single fixed row live from the typed
query on every keystroke (`refilter`'s `sync_goto_line_row`), and its own
accept-path gate (`selected_is_line_jump`) is the numeric sibling of the
existing `selected_is_heading` gate. Both resolve through the SAME shared
jump owner (`Effect::JumpToLine`) the Headings lens already uses, so caret
placement, fold reveal and follow-scroll are not reimplemented. First,
middle, last and out-of-range lines, wrapped text, Unicode and a folded
destination are swept through `--keys` with the sidecar proving caret and
scroll state (`src/capture/tests/goto_line_jump.rs`).

**A real lens-leak bug surfaced and got fixed along the way** (found by the
lane while writing the lens-scoping law, not briefed): the generic "Files"
bucket predicate (`!heading && !is_dir`) accidentally also claimed the
line-jump row, since it wasn't gated on anything but those two flags.
`filter.rs`'s `retain_visible_rows` now scopes the row to `facet_lens == 0`
explicitly, mirroring how `GotoHeading` is already scoped to its own lens —
watched the law go red on the exact bug, then green after the fix; the
accept-path wiring itself was separately mutation-tested by disabling it and
confirming both the unit and capture-level laws go red on that regression.
`overlay_nav.rs`, `app/apply.rs`, `overlay/nav.rs`, `overlay/state.rs` and
`render/benchsuite/scenarios.rs` crossed their frozen code-health high-water
marks with this slice's own additions; each raise verified against the
merged tree.

**Item 459 is now half-landed: slices 3 and 4 done, slices 1 (Trash) and 2
(Save a Copy) not yet dispatched.** Slice 1 has an explicit dependency
(item 444 residual 2's zero-document state) for its final-document case.

**Slice 3 landed on `main` (`cdd5a1bc`).** `CopyFilePath` and
`RevealInFileManager` join the palette and item 444's shared filename context
menu, both gated off (`PaletteGates::named_file`) for an unnamed scratch
document rather than fabricating a location. Reveal reuses the export
door's own `App::reveal_path` gate — no second live-only implementation —
with an explicit `#[cfg(not(target_arch = "wasm32"))]`/wasm-stub split so
the match stays exhaustive on the browser target, which never surfaces the
command at all (native-only in the catalog). `Effect::RevealInFileManager`
is recorded (intercepted, never performed) by `--keys` replay, the same
external-handoff shape `Export`/`FollowLink` already use. Copy File Path
puts the absolute native path on the system clipboard, asserted by exact
text.

Two real bugs the lane caught before landing: a non-hermetic clipboard test
that read the real host's scratch clipboard stash instead of an injected
fake, and a wasm build break in the `RevealInFileManager` effect arm (the
match needed its browser-side stub, not just the native gate). Both fixed
and reverified. `actions.rs`, `app/apply.rs`, `commands.rs` and `replay.rs`
crossed their frozen code-health high-water marks with this slice's own
additions; each raise verified against the merged tree, not the branch's
pre-rebase number.

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

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16; residual 1(a)+(c) 🟡 IN PROGRESS — item-444-affordance-prototypes (codex), branch codex/item-444-affordance-prototypes)

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
