# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 504 — destination navigators show no current-folder indication while browsing (found by item 444's Move-navigator verification, 2026-08-26)

Move, Export, and ProjectBrowse all navigate `browse_dir` internally but never
render it: the card title stays static (e.g. `move welcome.md`) with no
breadcrumb after descending into a subfolder, confirmed by real
`--screenshot-app` pixels on Move. Item 444's decided UX said the Move
navigator "shows the current root-relative destination" — this is the gap
against that line, left unbuilt there because a Move-only fix would violate
"same behavior ⇒ same code" (this is shared, pre-existing infrastructure, not
something Move's build introduced).

DECIDED (orchestrator, cheap to revert — one function): fold the current
root-relative destination into the card title itself rather than adding a
separate breadcrumb widget, extending `OverlayState::title()`
(`src/overlay/state.rs`) — the one existing owner that already builds
`"move welcome.md"` — to append the browse-relative folder when
`browse_dir` is `Some` and non-root (e.g. `"move welcome.md to notes/drafts/"`),
unchanged when at root. All three navigators (Move/Export/ProjectBrowse)
route through this one function already, so no forking. Revert cost: one
function in `overlay/state.rs`.

Item 507 (folder identity + current-item indication elsewhere) follows the
same convention once its surface is identified: the existing trailing-`/`
rule in `row_display` (`row.is_dir` / `RowMeta::GotoFolder`) for identity,
and the existing `active: bool` marker already carried by `StackRow` /
panel rows for current-item indication — reuse both rather than inventing a
third.

---
### 505 — active-lens mark draws at the wrong x on banded compositions (user-reported, reproduced headlessly, 2026-08-27)

Verified by real pixels:
`cargo run -- --screenshot OUT.png --theme Magpie --keys "s-o Right Right f i l e" README.md`
— the Go-to strip shows Headings active in full ink, while the underline
draws left of "All", under empty card. Reproduces identically with the theme
pinned, so the law can be hermetic.

Mechanism (hypothesis; the lane confirms before fixing): `overlay_shape_theme`
(`render/chrome/theme_picker.rs`) computes every mark rect — underline, pill,
tab, brackets, and the ghost/tab-plate collections — as `geom.text_left +
shaped glyph x`. But the emitter seats the head band (query + strip lines) at
`overlay_head_left(geom, plan)` (`overlay_ink.rs::overlay_panel_bands`), which
differs from `text_left` on any banded composition (right-anchored faceted
card, split row lane, diagonal cluster — Magpie). The mark misses by exactly
the seat delta. The strip hit-test reads the same raw spans ("the skin can
never disagree with where a label is clicked" — true, but both can disagree
with where the label is *drawn*), so clicking a drawn label likely selects the
wrong lens on those worlds — probe that too.

Fix shape: the mark and hit-test ride the same seat owner the emitter uses —
one owner, no second reading. Law: the active mark's x-span sits under the
ACTIVE label's drawn glyphs, swept across facet-style × composition (upright
AND banded worlds — the green law that misses this sweeps only upright), with
the enrolled world named in the failure message. Prove non-vacuity by
reintroducing the `text_left` seat and watching it go red.

---
### 507 — an opened folder's row reads as a file and gets no current highlight (user-reported, 2026-08-27)

The user opened the `syntax` folder and, in the right-anchored list, its row
shows no folder identity (no trailing `/`, unlike Go-to's Folders-lens rows
which carry one by law) and no indication it is the current/open one: "it's
kinda confusing how its not highlighted? and it looks like a file when it's a
folder? (i guess cuz this isn't the root folder?)". Identify which surface
that list is (working set rail vs. a picker), verify the non-root guess, and
fix folder identity + current-item indication there. Same missing-orientation
family as item 504 — decide the presentation together so the two land as one
pattern, not two.

---
### 508 — truncated lists give no scroll or position indication (user-reported UX gap, 2026-08-27)

Long lists fold behind `+ N more…` (`workingset.rs`) and pickers window their
rows, but nothing tells the user where they are in the list or how much is
below: "there is no scroll bar so like how do i even know where my files
are... when you click the show more, it needs to show a scroll bar."
DECIDED (user-confirmed 2026-08-27): no literal scrollbar — a faint
positional count cue at the window's edges ("↑ 3 more" / "↓ 41 more"),
extending the existing `+ N more…` idiom; text-only, so it fits the
summoned-card personality and adds no interactive machinery. Scrolling
already works (picker wheel accumulation in `app/input/wheel.rs`, plus
arrow-key window sliding via `scroll_window`); the cue is orientation on top.
If direct manipulation is ever wanted, match the app's one existing
scrollbar-like object — the transient, thumb-proportioned table pan bar
(`markdown/tables.rs::table_pan_bar`) — rather than standing up persistent
chrome.

Scope (user-confirmed): STRUCTURAL, not per-surface. Derive the cue at the
one windowing owner (`scroll_window`'s `item_top`/`item_visible`/`n_items`),
so every windowed list — Go-to, command palette, theme picker, the
destination navigators, the expanded working-set panel — enrols for free and
a fitting list draws nothing. Two traps: sectioned cards (theme picker)
window DISPLAY LINES, but the cue counts hidden ITEMS, which the
plan/window split already distinguishes; and the resting stack's `+ N more…`
is an expand affordance, not a count — it stays, the cue lives in the
scrolling views only. Law: one sweep over the picker roster, no-wildcard
match, so a new picker cannot ship windowed rows without the cue.

Acceptance case (the user's own screenshot, 2026-08-27): the COMMAND
PALETTE's unfiltered All lens — a dozen-ish rows drawn from a much longer
command roster, ending at an ordinary row with nothing saying the list
continues. After the fix, that exact frame shows the below-window count; a
capture of the palette at the default window with the cue present and
arithmetic-correct (hidden = roster − visible) is the item's verify.

Second acceptance case, and the law's geometry axis (user's screenshot,
2026-08-27): the THEME PICKER in a SHORT window — eight world rows visible
of the full roster, nothing below the last row saying more exist. The cue
fires whenever the window clips the list, so the law sweeps window
geometries (tall-fits → no cue; short-clips → cue, arithmetic-correct) —
one geometry is the classic way this law would go green while blind.

---
### 509 — right-click menu summons the full-page scrim instead of a localized panel (user decision, 2026-08-27)

Right-clicking a heading opens the context actions (Fold section / Collapse
other sections / Go to heading…) positioned near the pointer, but with the
whole page frosted/blurred — the theme-picker's summoned-card treatment. The
user's direction is explicit: "we want like a localised right click panel" —
a compact panel at the click, the page around it staying legible, no
full-page scrim. Scope: give the context menu its own presentation (or a
scrim-free arm of the card machinery) without forking the row/hit-test
mechanics; sweep worlds so every composition draws it localized.

---
### 510 — Rename opens with an empty field instead of the current name (user-reported, 2026-08-27)

The Rename prompt shows a bare caret with the existing name only in the faint
hint ("rename to: fukushima-trip.md"); the user expected the field
pre-populated for editing: "it doesn't populate the existing file name? it's
kinda weird." Fix shape: seed the query with the current name. DECIDED
(user-confirmed 2026-08-27): stem selected, extension left untouched — the
file-manager convention. Law: the field opens seeded with that selection, plus
a `--keys` journey that edits the seed rather than typing from scratch.

---
### 511 — long bare URLs render as a raw multi-line wall (user-reported, 2026-08-27)

A pasted tracking-heavy URL wraps across seven lines of raw query string:
"isn't this kinda ugly?" On the Live-Preview model this is a conceal
candidate: a bare URL could display tamed while the caret is off its line
(e.g. domain + a quiet ellipsis, the full text returning on caret entry —
the same reveal mechanics `[text](url)` links already use, docs/markdown.md).
The file stays plain text throughout, per the product boundary.

DECIDED (orchestrator, cheap to revert — one new `ConcealKind` arm): tamed
form is scheme-stripped domain plus a quiet ellipsis, dropping path/query
entirely when concealed (`https://example.com/track?x=1&y=2` →
`example.com…`); full raw text reveals on caret entry or selection touch,
same as existing link conceal. Detect a bare URL as its own new
`ConcealKind::BareUrl` variant (`src/markdown/spans/kind.rs` +
`src/markdown/spans/detect.rs`) and route its reveal decision through the
existing line-scoped branch of `wysiwyg_reveals`
(`src/render/spans/conceal.rs`) rather than adding a second reveal rule.
Revert cost: remove the new `ConcealKind` variant, its detector arm, and its
`wysiwyg_reveals` match arm.

---
### 512 — working-set groups and folder history dedup by exact path spelling, and group labels carry leaf-only identity (measured, supersedes withdrawn 506, 2026-08-27)

The user reported "you can open the same folder twice??" (no steps captured).
Measured against the code rather than reproduced live: opening the same
folder twice through the SAME spelling cannot duplicate anything —
`recents::push` retains-out prior occurrences, `WorkingSet::open` matches by
key, `workingset/panel.rs::expanded_full` contains-checks roots. Two adjacent
gaps are certain from the code, either of which reads as "the same folder
twice":

(a) Group headings print `project::folder_name(root)` — the leaf alone — so
two different projects both named `notes` draw two identical,
indistinguishable headings. Any two same-named folders opened as projects in
one session hit this.

(b) Root identity is exact `PathBuf` equality everywhere (groups, recents
MRU), so alias spellings of ONE folder — macOS `/System/Volumes/Data`
firmlink vs `/Users/...`, symlinks, case-variant paths on case-insensitive
APFS — become two groups / two recent rows.

Fix shape: (a) disambiguate same-leaf group labels (quiet parent, like file
rows' `parent` label); (b) canonicalize at the one root-identity owner before
compare/store. Presentation of group rows lands with item 507's
folder-identity work. Laws at the unit seam with injected paths (the Recent
lens is tier-1-only, docs/harness-reach.md).

---
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
