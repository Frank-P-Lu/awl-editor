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

Fix shape (not yet decided): a breadcrumb rendered beside the title, or the
current folder folded into the row label — one owner all three navigators
route through, matching the pattern the mechanism already uses elsewhere.
Small enough to scope as its own item; needs a taste call on presentation
before building.

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
### 506 — the same folder can be opened twice (user-reported, unreproduced, 2026-08-27)

The user's words: "you can open the same folder twice??" — no steps captured.
First task is the premise check: find the open path that admits a duplicate
(Go-to's Folders lens, "Choose another folder…", recent projects, the
working-set rail?) and reproduce before touching code. If real: dedup at the
one owner of folder-open, plus a law; if the premise dies, close as "premise
false, oracle repaired" with the measurement that killed it.

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
are... when you click the show more, it needs to show a scroll bar." Needs a
taste call first — a literal scrollbar cuts against the summoned-overlay
personality (DESIGN.md), so weigh a position cue (e.g. "12–19 of 64" or a
minimal thumb) against the user's explicit ask, then build it as one owner
across the folded surfaces.

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
kinda weird." Fix shape: seed the query with the current name. Taste call on
the seed's ergonomics — caret at end, name selected for overtype, or stem
selected with the extension left alone (the common file-manager convention) —
then a law that the field opens seeded and a `--keys` journey that edits the
seed rather than typing from scratch.

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
