# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 457 — 🔵 BLOCKED (user decision): what the emacs flavor MEANS on Linux — clipboard chords and the Meta layer

Two coupled taste calls, parked with a recommendation; the exact question was
put to the user in-session 2026-08-17 and may resolve within the day.

Today `keymap = "emacs"` on Linux keeps EVERY displaced Ctrl-letter to its
emacs meaning (`src/config/model.rs:119-125` composes the preset unfiltered):
C-c and C-x become prefixes, C-v page-down — so Omarchy's Super+C→Ctrl+C
forwarding lands on `BeginPrefix` and "copy is broken." CLAUDE.md's
"C-c/C-x/C-v stay native" tripwire holds only under the NATIVE flavor; the
compensation is a manual `[keys]` carve-out (`src/config/write.rs:90-98`,
GUIDE.md). And the flavor seeds no Meta layer at all: the Option-letter
retirement is a macOS platform rule (Option types accents) that does not bind
Linux Alt.

**Question (a):** should the emacs preset leave C-c and C-v native by default —
clipboard survives compositor forwarding; emacs hands keep C-w cut and C-y
paste — while C-x stays the emacs prefix (it carries save/open; excluding it
guts the flavor)? Purists reclaim chords via `[keys]`, the same recipe the
generated config already prints, inverted.

**Question (b):** should the emacs flavor on Linux seed the classic Meta
layer — M-x command palette, M-w copy, M-f/M-b/M-d word ops, M-v/M-< /M-> —
given Alt IS emacs's Meta? This is "use Alt on Linux" done the emacs-authentic
way, and it completes item 456's Linux-emacs palette binding.

**Recommendation: yes to both.** On accept: update the CLAUDE.md tripwire
wording and the generated-config carve-out comment, and re-pin the flavor
laws (`keymap_flavor_emacs_preset_reverts_every_displaced_chord_to_emacs_meaning`
currently pins c/x/v to emacs meanings — it flips to pinning the new
composition, swept over the whole displaced roster).

### 456 — the command palette becomes a real command (USER DECISION 2026-08-17; ready to build)

`OpenCommandPalette` is an uncatalogued hand-written resolver arm
(`src/keymap/resolve.rs:126-138`). Confirmed consequences: absent from GUIDE's
generated key table (the user looked and reasonably concluded it has no
default binding); un-rebindable — `[keys]` resolves names through the catalog
only; no menu item possible — the routed roster maps ids by catalog command
name; and on Linux under the emacs flavor it has NO binding at all (C-p is
kept as previous-line, and the resolver consults seeded defaults before the
bespoke arm). Combined with item 454's silent toggle this stranded the user;
recovery was Ctrl-, → Settings alone.

Decided: catalog it. A real command ("Command palette", native slot Cmd-P,
emacs slot EMPTY pending item 457 — M-x is the natural candidate), keeping
Cmd-Shift-P → Open project intact, plus a menu item (lane picks the section).
Behavior must not change on macOS or Linux-native: ⌘P / Ctrl-P resolve exactly
as today. Check the web build: if the catalog path trips `webreserved` for
Cmd-P where the bespoke arm did not, add the `WEB_ALTERNATE` entry rather than
losing the web binding.

Verify: existing Cmd-P/Ctrl-P/Cmd-Shift-P keymap laws stay green; a new law
rebinds via `[keys]` and asserts dispatch; the GUIDE table gains its row,
spot-checked against the dispatch it claims (generated-docs rule); the menu
roster law covers the new item. Mutation: drop the catalog slot, watch the
rebind law go red.

### 455 — the drawn menu's chord column tells the truth (defect; ready to build)

The Linux drawn menu bar's chord column is config-blind:
`render/chrome/menubar/dropdown.rs` → `menu::item_chord_for_id` →
`resolved_native_label_truthful`, which by its own doc
(`src/commands/chords.rs:151-154`) applies only the unconditional builtin keep
tier and EXPECTS a caller that knows the user's config to layer the rest on
top. The menu never does — so under `keymap = "emacs"` it prints Ctrl+C beside
Copy while Ctrl+C actually begins a prefix, and a `[keys]` rebind never
updates any label. The palette already does this right
(`visible_effective_bindings`, `src/overlay/build.rs:190-201`).

Fix: thread the config's `[keys]` overrides and `effective_linux_keep()` into
the menu chord column through the same owner the palette uses — one owner,
never a second implementation — and re-read labels on flavor toggle / config
reload. A chord suppressed for this user's config shows an EMPTY cell, like
Insert-link's Linux cell, never a false chord.

Also correct the stale claims this exposed: `docs/platform.md:52`
("Linux/wasm have none") and Cargo.toml's muda scope comment both deny the
drawn bar exists; it defaults ON off-macOS (`src/menubar.rs:30-31`). Give the
drawn bar its own docs sentence while there.

Verify: unit law over the menu roster × both flavors × sampled displaced
letters: under emacs, Copy/Find/Select-all rows never print a Ctrl-letter the
resolver dispatches elsewhere; under native they print the real chord. Prove
non-vacuity by routing back through the config-blind path and watching it red.

### 454 — Keymap setting becomes a picker with a name people can read (USER DECISION 2026-08-17; ready to build)

The palette's "Keymap" row is a `Toggle`: Enter silently flips native↔emacs,
persists it, closes the palette, and shows nothing
(`dispatch_settings_row` → `SettingToggle "keymap"`; the refresh no-ops
because the palette already closed). The user hit exactly this — flipped to
emacs without knowing, then item 456's stranding. The repo already knows
silent state flips are wrong: `Action::ConvertLineEndings` carries the comment
and the notice ("which one am I on" is the question the user has).

Decided: Keymap presents like Caret style — a catalog command ("Keymap…"),
`COVERED_BY` suppressing the settings row in the palette, descending into a
sub-overlay picker: one row per flavor, current value pre-selected, Esc
resumes the palette. No audition; accept applies AND emits a notice naming
the layout now in effect.

Labels (recommendation — land it and await taste, per standing policy): plain
language, never "emacs" bare — e.g. "Standard — Ctrl+C copies, Ctrl+V pastes"
/ "Emacs — Ctrl navigates (C-p, C-n, C-a…)", descriptions in the secondary
column. On macOS the flavor is structurally inert (`linux_keeps` gates on
`Convention::Linux`) — hide the command/row there rather than shipping a
setting that does nothing.

Verify: unit laws at the overlay seam — picker rows, pre-selection,
Esc-resume, notice text naming the RESULTING flavor. Persistence is App-owned
(`persist_pref`), so that half drives `--screenshot-app` or the settings unit
seam per docs/harness-reach.md. Mutation: re-silence the accept, watch the
notice law go red.

### 453 — rip out the in-app Guide and Reference (USER DECISION 2026-08-17; ready to build)

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

### 452 — Credits becomes a summoned read-only viewer (USER DECISION 2026-08-17; ready to build)

Help ▸ Credits (and ⌘P → Credits) swaps the editor to a real editable buffer:
`open_credits` → `open_bundled_doc` (`src/app/files/open.rs:41-68`) writes the
embedded text to `data_root()/credits.md` and runs ordinary `load_path`. The
user reports it as disorienting — suddenly you are in another file, with no
reason to edit it (edits are silently clobbered by the refresh write on next
open).

Decided: Credits renders in a summoned, scrollable, read-only viewer — a mini
window over the document, lightly rendered markdown — never a buffer swap.
DESIGN's summoned-overlays-over-persistent-chrome. No read-only document
surface exists today: cards are span lists, not document renderers; the
nearest machinery is the History/Conflict comparison pane
(`src/overlay/comparison.rs`), read-only by overlay modality. Whether to grow
that or build a sibling is the lane's call.

Scope: Credits only — item 453 removes Guide/Reference. The on-disk refresh
copy and its autosave rationale retire with the buffer route. The About
card's "⌘P → Credits" span stays true; update `docs/licensing.md`'s
description. The user's forthcoming baked-in starting guide may later share
this viewer; do not design for it yet.

Verify: read docs/harness-reach.md before promising captures. Overlay
presence + scroll position land in the sidecar through the one redacting
writer; `--keys` drives open/scroll/dismiss; pixel arithmetic asserts
rendered credits text is present and legible across the world roster, with a
companion presence floor (a viewer faded to the page fails, never passes
happier). A law asserts the active buffer and its path did NOT change across
open/dismiss — the regression this item exists to prevent.

### 450 — the bottom identity names the ACTIVE FILE's own folder (USER DECISION 2026-08-17; ready to build)

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

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16; RESTING STACK RENDERS, INTERACTION NOT STARTED)

🟡 IN PROGRESS (residuals 1+2: sidecar exposure, click-to-switch,
hover-close) — claude, branch claude/item-444-working-set-interaction

**Landed on `main` (`f8558c41`, `f53ffa6c`, `7c442d2b`, `dcb86fbb`, `8adcd961`,
`05527cc1`, `49a76026`, `aa1972b0`, `809aad3d`, `d7024bbb`, `17090614`,
`53e82629`, `df79d816`, `749b8e7e`):** the `--seed-tree` capture door; the
`WorkingSet` module; the cross-root ownership fix (see item 450 for the one
related product question it surfaced); and now **the resting-stack render
itself**. The bottom-left identity widens into a stable-order stack when the
active project holds more than one open file — current file forward on a
plate, siblings dimmed, nested files showing their root-relative parent in
quieter ink. N=1 (today's single-file case) is BYTE-IDENTICAL by
construction: `WorkingSet::stack_rows` returns empty below two files, so
`GutterLayout::lines()` takes the exact old single-`Name`-line path — proven
40/40 across all 20 worlds × 2 DPI against a real base-vs-branch binary
comparison, non-vacuously (the same comparison DIFFERS at 3 files). This
also found and fixed a real capture-harness gap: `--screenshot-app` drives a
real `App` for STATE but rendered through a `ViewState` built from the
buffer alone, so a 3-file working set photographed as a 1-file margin — the
one door meant to witness this surface couldn't. `CaptureOpts::fold_gutter`
closes it at zero line cost (`src/capture/modes.rs` stays at its frozen
size). Mutation-proven throughout. Gallery shots exist now (worktree-local,
not yet re-captured against a durable path — the next lane's first task is
producing a reviewable set from this landed render, since the prototype
crops used to validate it lived in a now-removed worktree).

**Residual, in the order the landed work sets up:**
1. **Sidecar working set** — `buffers` gains `files[]` + `active_index`.
   Costs a `SCHEMA_VERSION` bump and a ledger row. `src/capture/modes.rs`
   is at its frozen size baseline already — new state can't grow it.
2. **Click-to-switch + hover-close** — the gutter has no left-click path
   today; `App::outline_click` is the template. `close_key` is already
   law-tested and waiting.
3. **⌘W as the true removal owner** (today it parks, not closes).
4. **Zero-document state** — the largest remaining piece; `DocumentSession`
   would need an optional active slot, and every subsystem this item names
   (renderer, actions, autosave, session, title, accessibility tree,
   sidecar) needs an honest `no active document` representation.
5. **Overflow windowing, expanded/grouped cross-project view, Move
   navigator** — deliberately last: this item's own text asks for captures
   judged by the user before any of these get built. The resting stack now
   renders, so a >5-file / cross-project capture set can finally be
   produced — but the exact windowing/grouping rule still needs the user's
   eyes on that set before it's built, not guessed ahead of it.

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
