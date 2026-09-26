# File navigation and presentation

Read before changing Files navigation, hover, input timing, or preview presentation.

[Guide](../render.md) · Paths in code examples are relative to the repo root.

## File navigation: hierarchy, focus and commitment

Files / Recent is one contextual overlay for the **folder → file → writing**
journey. It does not absorb the working set, Outline, Commands, Settings or
History: the working set switches files already open; Files / Recent chooses what
to open; Outline moves within the current document; Commands runs actions;
Settings changes behaviour; History examines changes to the named document.

The visible order is context, query/views, choices, actions:

1. The title and current root name the scope. Files adds breadcrumbs and an Up
   affordance for the browse location; Recent stays scoped to the same root.
2. One bounded query and the separate Files / Recent choices say how the roster
   is being viewed. A filename/path query searches the named root; it is not a
   document-content search.
3. Rows lead with names. Root-relative paths appear when they disambiguate a
   result, including duplicate names and search results below the current level.
4. Change folder and New document are explicit actions. New document names the
   destination: the displayed directory while browsing, otherwise the root.

The Files overlay opens with the query focused and the Files view active. Its
forward Tab route is `query → Files → Recent → Up → choices → Change
folder → New document → query`; Shift-Tab walks the same route backwards.
Up is skipped at the root and in Recent, and an absent or disabled action is
skipped. Narrow presentation may stage or move regions, but it does not alter
this order. A child system/fallback folder chooser parks the parent; Esc in the
child returns to the same parent control, while Esc in Files / Recent dismisses
the overlay and restores editor focus.

| Focus | Text and arrows | Enter / accept | Pointer |
| --- | --- | --- | --- |
| Query | Text edits the query; Left/Right moves its caret; Up/Down transfers to the choices and selects the nearest surviving row. | Accepts the selected row when one exists. | Clicking the field focuses it without committing a row. |
| Files or Recent | Left/Right moves only between these two view choices; typing transfers to the query and filters. | Activates that view and restores its useful selection and scroll. | Clicking activates that view. |
| Up | Left or Enter ascends one directory; typing transfers to the query. Right is inert. | Ascends without changing the writing root. | Clicking ascends. |
| Choices | Up/Down changes selection; typing transfers to the query and filters. With an empty Files query, Left ascends and Right enters a selected folder; Left/Right are inert for files, Recent, and flat search results. | A folder enters that folder in place. A file opens or activates it and dismisses the overlay. | Clicking selects; activating uses the same folder/file action as Enter. |
| Change folder | Arrows do not mutate the document or browsing root. | Opens the platform or fallback root chooser. Success returns to Files showing the chosen root; cancel preserves the prior context. | Clicking invokes the same action. |
| New document | Arrows do not mutate the document or browsing root. | Creates at the destination named by the control, through the existing New document action. | Clicking invokes the same action. |

Selection, focus, hover, preview and commitment are separate facts. One row may
remain selected while keyboard focus moves to another region. Focus marks the
single region receiving keys. Hover identifies the pointer target without
changing selection. Files / Recent has no document preview: moving selection,
changing views, searching and browsing leave the background buffer and its root
untouched. Commitment occurs only when a file, root choice, or New document is
accepted. Accessibility publishes the same separation as focus, selection,
active view, expanded-folder state and action roles rather than collapsing them
into one highlight.

Direct menu, context-menu and palette Actions cross the same action-level surface
gate as their key bindings. In particular, New document names and uses the
current Files destination whether it arrived through a key, menu item or row;
a menu key equivalent must not bypass the summoned surface and act on the
background document.

One concrete interaction proves the route is connected rather than a collection
of labels: summon Files through its actual native or Emacs binding, type to
filter, press Down to focus a surviving folder, Right to enter it, Tab forward
through Change folder and New document, Shift-Tab back to the choices, then Enter
on a file. The overlay closes only on that final commitment; the original buffer
is untouched until then. Footers name the current verb (`enter folder`, `open
file`, `change folder`, or `new document`) and read the effective rebound/platform
binding from the command owner rather than printing a default literal.


## Picker hover/click stability + movement latency (`app/input/mouse/overlay.rs`, `overlay/nav.rs`, `probe.rs`)

- **THE REAL-MOTION HOVER GATE.** Passive hover previews only the two surfaces whose choice is itself a live document audition: Theme and Caret (`OverlayKind::previews_live_document`). `OverlayState::preview_hover_at(px, py, hit)` routes those two through `hover_at`, so real motion may select and preview; every ordinary picker records the same movement anchor but preserves the keyboard selection, with the pointing-hand cursor as its acknowledgement. Click is independent and hit-tests the row under the press. For a previewing surface, a world jump may re-layout rail/size/type/material immediately, but that re-layout must never itself read as another pointer gesture: `hover_at` compares `(px, py)` against `last_hover_px` and only re-hit-tests when the pointer's physical position genuinely changed. Pure and GPU-free (`hit` is caller-supplied), so `overlay::tests::hover_keyboard_nav::passive_hover_changes_selection_only_for_live_preview_kinds` sweeps the complete kind roster, beside the movement/slop unit laws and `render::tests::overlay_hover_stability_law` against real geometry. Keyboard navigation remains one visible row per press.
- **THE MOVEMENT-SLOP WIDENING + KEYBOARD BASELINE.** The exact-motion gate closed EXACT-pixel duplicates, but left a hazard the exact-equality check couldn't see: a list WINDOW SCROLLING under a genuinely resting pointer (a real keyboard `move_sel` deep enough to shift the candidate window) changes which row a stationary pixel hits, and any real `CursorMoved` past that point — even a single physical pixel of a resting hand's ordinary hardware jitter — used to steal the keyboard's own selection outright. `hover_at` now gates on SQUARED DISTANCE against `HOVER_MOVE_SLOP_PX` (private to `overlay::nav`, one owner; 4.0px, matching `app::DRAG_ARM_SLOP_PX`'s identical "content relocating under a stationary pointer" precedent) rather than bare inequality — real travel PAST the slop still takes over on the very first such event, no added latency, no dead zone; the anchor is STICKY below the slop (stamped forward only on a reported move) so a genuinely slow drag still crosses the threshold from its original anchor rather than re-basing itself every sub-slop increment. The OTHER half: `OverlayState::arm_hover_baseline(px, py)` re-anchors the gate to the pointer's CURRENT resting position, called by the shared keyboard-dispatch seam (`App::apply` live, `ReplaySession::apply_chord` headless) after every keyboard-driven action, AND by `App::overlay_wheel` — the mouse wheel drives `move_sel` exactly like ↑/↓ (a second "deliberate crossing" input class, same as the keyboard) but is dispatched straight from `on_mouse_wheel`, never through `apply`, so it stamps the baseline itself rather than relying on `apply`'s stamp — without either stamp, a session with no prior hover (`last_hover_px` still `None`) would let the pointer's first incidental `CursorMoved`, however small, read as unconditional motion under `hover_at`'s own cold-start rule and hand the selection to wherever a motionless hand happens to rest. `HOVER_MOVE_SLOP_PX` is now a re-export of `app::DRAG_ARM_SLOP_PX` itself (not a second `4.0` literal), so the two gates cannot drift apart under a future retune of either. Laws: `overlay::tests::hover_at_movement_slop_boundary_law` (the threshold in both directions, pure distance), `overlay::tests::keyboard_baseline_stamp_protects_a_pointer_that_was_never_explicitly_hovered` (non-vacuous: proves the unstamped scenario steals first), `overlay::tests::keyboard_nav_survives_a_pointer_parked_over_any_row_relative_to_the_destination` (row above/below/far/landing), `overlay::tests::hover_movement_slop_gate_holds_across_every_overlay_kind_no_wildcard` (the full `OverlayKind` roster via a compile-time-exhaustive match — the sweep still iterates the hand-kept `OverlayKind::ALL` roster, not the match itself; a variant added to the enum and the match but forgotten in `ALL` still compiles clean and silently skips the sweep, a limitation `OverlayKind::ALL`'s own doc discloses), `render::tests::hover_slop_law` (the same hazard against real pipeline geometry, swept across `Pane`/`Bars` and 1×/2× DPI), `app::input::tests::wheel_scroll_from_cold_start_does_not_expose_selection_to_the_next_hover_check` (the wheel path specifically, non-vacuous: proves the unstamped `overlay_wheel` steals first). **The pointer-replay seam:** `TextPipeline::resolve_overlay_hover` is the ONE hit-test-then-`hover_at` seam both `App::overlay_hover` (live) and `ReplaySession::apply_move` (headless, via `capture::OraclePipeline::resolve_overlay_hover` + `sync_overlay`, which pushes the CURRENT overlay's row/window geometry onto the otherwise buffer-only motion oracle) route through — proven end-to-end through the real `--keys` replay engine by `run::tests::pointer_replay_seam_reproduces_a_keyboard_scroll_stealing_a_stationary_pointer_check`. `apply_move`/`sync_overlay` are NOT yet wired to the `--keys` CLI STRING grammar (no `move X Y` token) — only reachable by constructing `ReplaySession` directly, as that law does — and cover the flat/list picker geometry path only (not the Theme picker's faceted `overlay_lens`/section-header layout).
- **THE ONE CLICK-ACTIVATION RULE.** `App::overlay_click` fires ONLY on `ElementState::Pressed` (never `Released`) and hit-tests `self.cursor_px` at that same instant — so "the row a click activates" is unconditionally the row under the PRESS, never a release position or whatever an earlier hover last computed. No separate release-time re-evaluation exists.
- **MOVEMENT-LATENCY, the live companion to offscreen `--bench-theme-burst`.** `App::retint_theme_preview` — the ONE owner every input kind (keyboard nav, mouse hover, mouse wheel) funnels a theme-picker world change through — arms `probe::mark_movement_input()` right before the real relayout work; `Gpu::redraw` closes it out against the frame it actually PRESENTS via `probe::note_presented_frame()`, at the exact point the existing `"present"` trace fires. `probe::latency_distribution()` reports count/min/p50/p95/max in ms. Reads through the SAME `probe::recording()` gate (both the automated `--live-script` probe and the user's `AWL_FLIGHT_RECORDER`) every other diagnostic trace point uses — no new shared live/headless driver. Surfaced two ways: the script grammar's `latency` step (prints one `LIVE-PROBE latency …` stdout line, the `shot` line's protocol twin) and, for the flight recorder, one closing `movement-latency distribution: …` trace line at app teardown. Native-only; a no-op on a plain launch (the clock is never even read). Laws: `probe::tests::movement_latency_mark_and_present_produce_a_sample_and_distribution`, `probe::tests::movement_latency_is_a_no_op_outside_recording`, `probe::tests::parse_covers_the_latency_step`, and `probe::tests::movement_latency_burst_of_n_reports_n_not_one` — **one pending mark per step, QUEUED, never a single slot that the next mark evicts.** An overwriting slot collapses an N-step burst to `n=1` whatever N is, and did: it once reported `n=1` for an 8-input burst and put a figure two orders of magnitude low into a design decision. The law sweeps N = 1, 2, 3, 8, 9 under the **zero-gap arrival order** — every mark fires before any present closes one out, which is what a burst produces when input outruns the frame loop, and is the ONLY shape that can observe the collapse; a tidy alternating mark/present loop passes against the bug, because the immediate present drains the slot before the next mark arrives. The distribution's TIMING/feel over real time is live-only by nature (CLAUDE.md) — the user remains the oracle for felt lag; this instruments the same real end-to-end path (dispatch → visible-prefix reshape → encode/submit → compositor present) `--bench-theme-burst` cannot reach offscreen. The off-screen tail is latest-selection-wins: a new preview supersedes it and the shared crossing quiet-settle finishes only the final highlighted world. Commit/revert force that settle synchronously.
- **THE THEME BAND'S 110 MS STARTS AT INPUT, NOT PREPARE.** The same `retint_theme_preview` owner stamps the live pipeline before synchronous preview shaping. The redraw supplies its injected `now` before `advance`/`prepare`; when selected-row geometry is finally known, both Pane's living morph and the ordinary sliding-band override derive phase from `now - movement_at`. Work therefore consumes the authored budget (40 ms of shaping means phase `40/110`, and 110 ms means settled) instead of adding a fresh tail. A rapid retarget samples the old pose at the input instant before preserving the hybrid latest-selection-wins snap; first-open and Reduce Motion stay settled. The stamp is absent from every unarmed capture, so ordinary settled frames read no clock and remain byte-identical. Flight lines keep the existing input/apply/prepare/present chain and add the prepared `phase` plus one `theme-band input-to-settled` endpoint; `theme latest/worst` still ends at first present, not the decorative tail.
- **LIVE MOTION SHARES ONE PRESENTATION SAMPLE.** Caret position and preview,
  copy pulse, overlay entrance and band, fold chevrons, and the travelling
  ground keep their authored curves but advance from one `FrameSample` (`now`
  plus elapsed since the prior successful present). `now` is the clock's
  visible-time axis: blur, occlusion, and failed presentation retain its last
  accepted value, so an absolute input epoch cannot fast-forward while pixels
  are parked. The post-prepare activity
  set is exhaustive, so geometry that arms the band during prepare is visible
  to scheduling in the same frame. Reduce Motion still settles each owner at
  its existing final pose. Capture's explicit delta seam stays clock-free; it
  never starts the live presentation clock.
