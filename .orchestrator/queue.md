# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 513 — the spell popup's material is unjudged (residual of the 2026-08-29 design session)

Parts (a) (context menu keeps palette grammar) and (c) (float material
reserved for content-preview surfaces) are pure decisions with nothing to
build, and part (b) (teaching footer dropped from contextual menus) has
landed on `main`. The one open piece: the **spell popup** is the taxonomy's
stray — a float that is actually a command list, unjudged rather than
"well-loved." Its precondition is now satisfied (the de-footered context
menu shipped). Judge it side by side with the context menu on Kite and a
Pane world, and either re-home it onto the pocket-palette grammar or record
why it stays a float.

---
### 514 — two defects `range_rail.rs` work surfaced, neither caused by that work (found while building item 508, 2026-08-29)

(a) `settings_overlay_view` (`src/render/tests/mod.rs`) never sets
`overlay_workspace`, so any caller that doesn't override it afterward tests
a state production can't reach (the Settings card renders through the wrong,
faceted geometry family instead of the real `RailOverRows` workspace
family). Repro: add `v.overlay_workspace = ov.workspace_shape().is_some();`
after `v.overlay_lens = ov.lens_strip();`, run
`settings_row_reach_law::{every_editor_row_is_hoverable_at_its_own_y_center_across_the_world_roster,
the_zoom_rows_band_and_its_neighbours_never_bleed_into_one_another}` — both
go red. Fix shape: set `overlay_workspace` correctly in the shared fixture
(every caller wants the real state; `range_rail.rs` worked around it locally
rather than fixing the shared one, since that was out of its own scope).

(b) `range_rail::a_non_selected_rails_thumb_never_wears_the_selected_rails_ink`
is `#[ignore]`d with the full evidence chain in its own doc comment: on
`world=Potoroo`, `assert_selected_rail_shows_its_flip` hits a documented,
pre-existing oracle weakness (Potoroo's striped background is a known
pixel-search false-positive source for a sibling assertion, per that file's
own history). Judge and repair the oracle, then un-ignore; this parks the
law's differential non-selected-ink coverage until fixed.

---
### 515 — one plate, one meaning: the working-set panel plates the current project AND the active file at once (user-confirmed confusing, 2026-08-29)

Screenshot evidence on Kite: the expanded panel drew TWO purple plates —
the `notes/` group heading (plated because it is the current project,
`workingset/panel.rs` Group arm + `gutter_stack.rs::plate_rects`) and the
`scratch` file row (plated because it is the active buffer,
`workingset.rs::file_row`). The code itself names these as two different
questions ("which file" / "which project", `gutter_stack.rs` doc) but
answers both with the same treatment in the same column, and the gutter
block's own folder heading ALREADY names the current project directly above
the rows — so "you are in notes" is stated twice and the double plate reads
as two selections. User: "it's confusing as heck? like we already have
'notes' at the top no?"

DECIDED (user-confirmed 2026-08-29): **the plate means the active file,
and nothing else.** A group heading that is the current project keeps its
`active_ink` distinction but loses its plate; the project identity is the
gutter folder heading's job. The lane enumerates every `plate_rects`-family
consumer rather than patching the one arm the screenshot showed (the module
doc also names a "bottom identity" plate — same sweep, same one-meaning
rule judged against it). Cheap to revert (render-side row treatment; no
state change), so per the standing taste policy: land on main and await
feedback, revert cost stated in the commit.

Law shape: at most one plated row per frame across the resting stack and
the expanded panel, and when one exists it is the active FILE row — swept
across worlds (Wagtail's page-inverse plate arm included), with non-vacuity
proven by re-plating a heading and watching it go red. Coordination: group-
row presentation is also touched by 507/512's folder-identity work — same
surface, keep the label conventions theirs and the plate rule this item's.

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
