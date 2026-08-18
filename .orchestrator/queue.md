# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 453 — rip out the in-app Guide and Reference (USER DECISION 2026-08-17; ready to build)

🟡 IN PROGRESS — claude, branch `claude/item-453-remove-guide-reference`

Decided: the in-app Guide and Reference doors go. Remove the catalog commands
(`src/commands/catalog/navigation.rs` Guide/Reference entries), their
Help-menu items, embedded copies (`embedded_docs.rs`, `guide.rs`), and their
`open_bundled_doc` callers — after item 452 lands, Credits is the only
bundled-doc consumer and the helper shrinks to fit. Rationale: Reference
belongs on the site, not in the editor; the user is writing a new starting
guide to be baked in, which arrives as its own future item. Welcome/first-run
seeding (`firstrun.rs`) is untouched.

GUIDE.md and REFERENCE.md remain in the repo as site/source documents — this
item removes the in-app doors only, not the files or their generators.
Reconcile the laws that reference these doors: the keytoken starting-docs law
drives chords the welcome/tour/GUIDE teach — keep whatever half still has a
subject; GUIDE's generated key table keeps its generation laws while the file
stays. If a law's whole subject is deleted, delete the law and say so in the
landing note.

Verify: the palette no longer offers Guide/Reference; the Help menu roster
law passes with the shrunk roster; grep for dangling references (docs, About
card spans, generated config comments, welcome/tour cross-links). Rust +
docs change: full gate.

### 458 — 🔵 Credits landed as a full workspace, not the "mini window" the user asked for (TASTE DIVERGENCE, found during item 452)

Item 452 (Credits summoned read-only viewer, landed `19339cd8`) asked for
"a mini window over the document"; what shipped reuses
`WorkspaceShape::TimelineOverComparison` — the same full-viewport
presentation History/Conflict use, with the primary column degenerated to
one fixed row. The lane's own call: DESIGN.md §5 reads sustained document
reading as workspace territory, not a brief contextual choice, and no
smaller read-only surface exists yet to reuse (cards are span lists, not
document renderers). **Named honestly, not silently landed as
cheap-to-revert**: a genuine floating mini-window would need new overlay
geometry, not a one-line revert of this decision.

Open the app, ⌘P → Credits, and look: does the full-workspace presentation
read as intended, or does a smaller floating card belong here instead? If
the latter, that is new geometry work, not a revert — say so and it goes
back on the board as its own item.

### 450 — the bottom identity names the ACTIVE FILE's own folder (USER DECISION 2026-08-17; ready to build)

🟡 IN PROGRESS — claude, branch `claude/item-450-folder-identity`

Item 444's lane closed the cross-root ownership bug where OPENING or
ACTIVATING a buffer from another root left `load_path` and the buffer
registry disagreeing about which project was active (`7c442d2b`, landed on
`main` — fixed, mutation-proven). A second, narrower case remains, and it was
a product call rather than a bug: invoking **Switch project** alone (Go to's
project picker, `s-o <project> Enter`), with NO document opened or activated
afterward, changes `project.root` while `buffers.active` stays the document
that was open before the switch — which may not live under the new root at
all. Reproduced on the real binary: after `s-o archive Enter` with nothing
else, the gutter's bottom identity prints `{ name: "index.md", project:
"archive" }` for a file that actually lives under `notes/`, and Go to / New /
Move / export now default into `archive` while the visibly open document sits
elsewhere.

**Decided: the bottom identity always names the active file's own folder,
never the nominally "active project."** It is DESIGN §5's "position in the
filesystem," so it describes where the open document actually is and never
shows false information. The data is already there — the file's own remembered
root, restored correctly by `7c442d2b`.

The DISPATCH root deliberately does NOT follow the label. Switch project keeps
doing exactly what it says: Go to, New, Move and export continue to default
into the newly chosen root. Only the identity string stops claiming the open
document lives there. Both halves are the decision; a later change that
re-syncs them by reverting the root switch is a different product, not a fix.

Accepted cost, recorded so it is not re-discovered as a defect: Switch project
gives no immediate visible confirmation while a foreign-root document stays
open. Do NOT add a toast here to compensate. The confirmation's home is item
444's expanded cross-project grouped view, where "which folder is active" is
already the thing being drawn; until that exists, the surface is quietly
silent on purpose.

Scope: the identity formatter's folder label only. This item does not touch
the registry, `load_path`, the project picker, or any destination default.

Verify: `s-o` is App-owned, so every claim here drives `--screenshot-app`
against seeded roots (never ambient ones — the margin photographs filenames).
A law seeds roots A and B, opens a file under A, invokes Switch project to B
with nothing else, and asserts BOTH halves in one frame: the gutter identity
still names A, AND a destination default (New document's target) still resolves
under B. Non-vacuity is provable by construction — before the change the
identity printed B — so break it and watch it go red. Companion presence floor:
the ordinary same-root case must still NAME its folder, so a formatter that
prints no folder at all fails rather than passes; assert the label's content,
not merely that it differs from B. Sweep the world roster at both DPI, since
the label is a rendered margin string.

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

1. **Hover-reveal close affordance** — draw the × (or equivalent) when the
   pointer enters a row's right close zone, so the now-live close action has
   a visual cue before more of the working set builds on top of it. New
   live-only render axis; capture-prototype and put shots to the user before
   committing to a treatment, same discipline as residual 3 below.
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
