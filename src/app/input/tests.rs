//! src/app/input/tests.rs — the click/drag-selection unit-test suite
//! (formerly `mod click_tests`), moved verbatim out of the former
//! `app/input.rs` monolith (2026-07 code-organization pass) and renamed
//! to the directory-split convention's plain `tests` — every test's
//! behavior is unchanged, only its module path
//! (`app::input::click_tests::foo` -> `app::input::tests::foo`, no
//! external caller named the old path).

use super::*;
use crate::app::*;
use crate::render::{Metrics, TEXT_LEFT, TEXT_TOP};

// Every `App` below is built via `App::new_hermetic` (see its doc on
// `App::new` in `app.rs`) — these tests only care about click/selection
// behavior over a `set_text` fixture, never real file content, so the
// hermetic constructor's injected `InMemoryFs` + disabled session-restore
// keep them from ever touching the developer's real
// `~/.local/share/awl/{session.toml,scratch.md}`.

/// Drive the selection-state seam after the separately-tested live pipeline has
/// resolved a pointer to document column `col`.
fn press_at_col(app: &mut App, col: usize, shift: bool) {
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    app.input.pointer.cursor_px = (TEXT_LEFT.0 + col as f32 * m.char_width, TEXT_TOP.0);
    app.press_at_char(col, shift);
}

#[test]
fn gutter_press_never_moves_or_selects_document_text() {
    // THE REPORTED BUG: margin/gutter coordinates used to enter the ordinary
    // hit-test, whose correct out-of-range clamp made the click land on the page's
    // first text column. The writing-column gate must leave the entire edit state
    // untouched — cursor, active selection, and drag arm alike.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    app.document.select_range(2, 7);
    let before_cursor = app.document.buffer().cursor_char();
    let before_selection = app.document.buffer().selection_range();

    app.input.pointer.cursor_px = (0.0, TEXT_TOP.0);
    app.on_press(false, false);

    assert_eq!(
        app.document.buffer().cursor_char(),
        before_cursor,
        "gutter press leaves the caret alone"
    );
    assert_eq!(
        app.document.buffer().selection_range(),
        before_selection,
        "gutter press neither creates nor clears a document selection"
    );
    assert!(
        !app.input.pointer.dragging,
        "gutter press cannot arm a text-selection drag"
    );
    assert!(
        !app.input.pointer.drag_armed,
        "gutter press cannot cross the drag-slop gate later"
    );

    // Moving well past the normal drag slop after that ignored press remains a
    // no-op: the gutter cannot become a delayed selection start.
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    move_by(&mut app, m.char_width * 4.0, 0.0);
    assert_eq!(app.document.buffer().cursor_char(), before_cursor);
    assert_eq!(app.document.buffer().selection_range(), before_selection);
}

#[test]
fn plain_click_clears_the_mark_and_places_the_cursor() {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    app.document.set_cursor(0);
    app.document.set_mark(); // an existing selection from a prior gesture
    press_at_col(&mut app, 6, false); // "w" of "world"
    assert!(
        !app.document.buffer().has_selection(),
        "a plain click drops any selection"
    );
    assert_eq!(app.document.buffer().cursor_char(), 6);
}

#[test]
fn shift_click_extends_from_the_cursors_prior_position() {
    // No existing mark: a shift-click must DROP the mark at wherever the
    // cursor already sat (char 0), then move ONLY the cursor to the hit
    // point — never `clear_mark`.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    app.document.set_cursor(0);
    assert!(app.document.buffer().anchor_char().is_none());
    press_at_col(&mut app, 6, true);
    assert_eq!(
        app.document.buffer().anchor_char(),
        Some(0),
        "mark drops at the prior cursor spot"
    );
    assert_eq!(
        app.document.buffer().cursor_char(),
        6,
        "cursor moves to the click"
    );
    assert_eq!(app.document.buffer().selection_range(), Some((0, 6)));
}

#[test]
fn shift_click_keeps_an_already_active_mark() {
    // A mark is already active (e.g. from C-Space or a prior shift-click):
    // a further shift-click must NOT move the mark, only the cursor.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    app.document.set_cursor(2);
    app.document.set_anchor(1); // mark pinned at char 1
    press_at_col(&mut app, 9, true);
    assert_eq!(
        app.document.buffer().anchor_char(),
        Some(1),
        "an active mark is never disturbed"
    );
    assert_eq!(app.document.buffer().cursor_char(), 9);
}

#[test]
fn double_and_triple_click_arms_ignore_shift() {
    // The word/line-select arms (click_count 2/3) are untouched by shift —
    // shift only modifies the single-click arm.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    // A first click at col 0 primes the multi-click detector; the SECOND
    // press at the same spot (inside `on_press`'s own `bump_click_count`
    // call) is recognized as the double-click, exactly as two real clicks
    // would be.
    press_at_col(&mut app, 0, false);
    press_at_col(&mut app, 0, true);
    // A double click at col 0 still selects the word "hello" wholesale,
    // exactly as an un-shifted double click would.
    assert_eq!(app.document.buffer().selection_range(), Some((0, 5)));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_double_click_and_word_drag_share_linguistic_units() {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("大幅に構成が変わっており");
    press_at_col(&mut app, 3, false);
    press_at_col(&mut app, 3, false);
    assert_eq!(
        app.document.buffer().selection_range(),
        Some((3, 5)),
        "double-click selects 構成"
    );

    app.drag_to_char(6);
    assert_eq!(
        app.document.buffer().selection_range(),
        Some((3, 9)),
        "the same word-granularity drag extends through 変わっ"
    );
}

// === THE PHANTOM-SELECTION-CLICK FIX ================================
// `PointerInput::drag_armed` / `PointerInput::exceeds_drag_slop`: a `CursorMoved` while
// `dragging` must only extend the selection once the pointer has genuinely
// traveled past `DRAG_ARM_SLOP_PX` from the press position — never merely
// because a WYSIWYG reveal reflow (concealed markup regaining its real glyph
// advance the instant the caret lands on that line) shifted what the SAME
// pixel position would now hit-test to. The pure `exceeds_drag_slop`
// geometry check below proves the arm decision reads pixel travel alone;
// the `App`-level tests prove the wiring end to end over the real
// `on_press` / `on_cursor_moved` seam.

#[test]
fn exceeds_drag_slop_is_false_for_a_perfectly_stationary_pointer() {
    // THE CORE OF THE FIX: zero pixel travel never arms a drag, no matter
    // what a hit-test at that same position would now resolve to (a reveal
    // reflow changes the hit-test RESULT, never the pointer's own pixel
    // position) — `exceeds_drag_slop` only ever looks at the two positions.
    assert!(!PointerInput::exceeds_drag_slop(
        (100.0, 200.0),
        (100.0, 200.0)
    ));
}

#[test]
fn exceeds_drag_slop_is_false_for_sub_slop_jitter() {
    // Real mice/trackpads report tiny (sub-pixel-rounded) motion even while
    // "held still" — e.g. the physical act of pressing the button. Anything
    // strictly under the slop must not arm.
    assert!(!PointerInput::exceeds_drag_slop(
        (100.0, 200.0),
        (102.0, 200.0)
    ));
    assert!(!PointerInput::exceeds_drag_slop(
        (100.0, 200.0),
        (100.0, 203.0)
    ));
    // Right at the threshold (distance == slop, not >) still does not arm —
    // the comparison is strict `>`.
    assert!(!PointerInput::exceeds_drag_slop(
        (0.0, 0.0),
        (DRAG_ARM_SLOP_PX, 0.0)
    ));
}

#[test]
fn exceeds_drag_slop_is_true_past_the_threshold() {
    assert!(PointerInput::exceeds_drag_slop(
        (100.0, 200.0),
        (105.0, 200.0)
    ));
    assert!(PointerInput::exceeds_drag_slop(
        (100.0, 200.0),
        (100.0, 205.0)
    ));
}

#[test]
fn exceeds_drag_slop_combines_both_axes_diagonally() {
    // Neither axis alone clears the slop, but the diagonal (Euclidean)
    // distance does — the squared-distance compare must sum both axes, not
    // check them independently.
    let (dx, dy): (f32, f32) = (3.0, 3.0);
    assert!(
        (dx * dx + dy * dy).sqrt() > DRAG_ARM_SLOP_PX,
        "test fixture sanity"
    );
    assert!(PointerInput::exceeds_drag_slop((0.0, 0.0), (dx, dy)));
}

#[test]
fn release_resets_drag_arm_and_next_press_snapshots_a_fresh_baseline() {
    let mut pointer = PointerInput {
        pointer_hide: crate::pointer_hide::PointerHide::Visible,
        cursor_px: (10.0, 20.0),
        dragging: false,
        drag_press_px: (0.0, 0.0),
        drag_armed: false,
        drag_scroll_last_tick: None,
        page_resizing: false,
        page_resize_edge: None,
        page_resize_anchor: None,
        image_resizing: None,
        range_drag: None,
        cursor_icon: winit::window::CursorIcon::Default,
        drag_granularity: DragGranularity::Char,
        last_click_time: None,
        last_click_px: (0.0, 0.0),
        click_count: 0,
        scroll_px_accum: 0.0,
        scroll_sensitivity: 1.0,
        resynced_geometry: None,
    };

    pointer.begin_text_drag();
    assert_eq!(pointer.drag_press_px, (10.0, 20.0));
    pointer.cursor_px = (20.0, 20.0);
    assert!(
        pointer.arm_text_drag_if_moved(),
        "first gesture crosses slop"
    );

    pointer.finish_text_drag();
    assert!(!pointer.dragging, "release ends the gesture");
    assert!(!pointer.drag_armed, "release retires the sticky drag arm");

    pointer.cursor_px = (80.0, 90.0);
    pointer.begin_text_drag();
    assert_eq!(
        pointer.drag_press_px,
        (80.0, 90.0),
        "the next press snapshots its own position, never the old baseline"
    );
    assert!(pointer.dragging);
    assert!(!pointer.drag_armed, "every press starts below drag slop");
}

/// Move a hermetic pointer through the drag-arm state seam, then supply the
/// document endpoint that production obtains from the live pipeline.
fn move_by(app: &mut App, dx: f32, dy: f32) {
    let (x, y) = app.input.pointer.cursor_px;
    app.on_cursor_moved(winit::dpi::PhysicalPosition::new(
        (x + dx) as f64,
        (y + dy) as f64,
    ));
    if app.input.pointer.drag_armed {
        let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
        let line = ((app.input.pointer.cursor_px.1 - TEXT_TOP.0).max(0.0) / m.line_height).floor()
            as usize;
        let col = ((app.input.pointer.cursor_px.0 - TEXT_LEFT.0).max(0.0) / m.char_width).round()
            as usize;
        app.drag_to_char(app.document.buffer().hit_char(line, col));
    }
}

#[test]
fn stationary_pointer_after_press_never_arms_a_selection() {
    // A press, then a `CursorMoved` reporting the EXACT press pixel again —
    // exactly what a reveal-reflow's redraw could look like if it ever
    // spuriously re-delivered the pointer position (or a genuinely idle
    // pointer between press and release) — must read as a plain click.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    press_at_col(&mut app, 6, false);
    assert_eq!(app.document.buffer().cursor_char(), 6);
    move_by(&mut app, 0.0, 0.0);
    assert!(
        !app.document.buffer().has_selection(),
        "no travel must never arm a selection"
    );
    assert_eq!(
        app.document.buffer().cursor_char(),
        6,
        "the caret stays at the press's own hit-test result"
    );
}

#[test]
fn sub_slop_jitter_does_not_arm_a_selection_even_across_a_column_boundary() {
    // Engineer the press to sit just BEFORE a column's rounding boundary, so
    // a jitter of less than `DRAG_ARM_SLOP_PX` is enough to make a fresh
    // hit-test resolve to the NEXT column over — standing in for a WYSIWYG
    // reveal reflow relocating the same document position by a few px under
    // an otherwise-still pointer. The fix must gate on the pointer's own
    // travel, not on whatever the hit-test now returns.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    // Half a cell short of column 6's boundary: rounds to column 6 today,
    // but a nudge of less than half a cell tips it to column 7.
    app.input.pointer.cursor_px = (TEXT_LEFT.0 + 6.0 * m.char_width - 0.5, TEXT_TOP.0);
    app.press_at_char(6, false);
    let pressed_at = app.document.buffer().cursor_char();
    assert!(
        DRAG_ARM_SLOP_PX < m.char_width / 2.0,
        "test fixture sanity: slop < half a cell"
    );
    move_by(&mut app, DRAG_ARM_SLOP_PX - 0.1, 0.0);
    assert!(
        !app.document.buffer().has_selection(),
        "sub-slop travel must never arm a selection"
    );
    assert_eq!(
        app.document.buffer().cursor_char(),
        pressed_at,
        "the caret must not drift under sub-slop jitter"
    );
}

#[test]
fn real_drag_past_the_slop_arms_and_extends_the_selection() {
    // A genuine drag — well past the slop — must still work exactly as
    // before: the selection extends live, char by char, as the pointer
    // moves.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    press_at_col(&mut app, 0, false);
    assert!(!app.document.buffer().has_selection());
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    move_by(&mut app, 6.0 * m.char_width, 0.0);
    assert!(
        app.document.buffer().has_selection(),
        "travel past the slop must arm a real drag"
    );
    assert_eq!(app.document.buffer().selection_range(), Some((0, 6)));
}

#[test]
fn once_armed_a_drag_stays_armed_through_further_sub_slop_moves() {
    // A real drag that then pauses/jitters mid-gesture must keep extending
    // (armed is sticky for the rest of the gesture) — only the FIRST move of
    // a fresh press is slop-gated.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    press_at_col(&mut app, 0, false);
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    move_by(&mut app, 6.0 * m.char_width, 0.0); // arms the drag
    assert_eq!(app.document.buffer().selection_range(), Some((0, 6)));
    // A tiny further nudge (well under the slop) still extends, because the
    // gesture is already armed.
    move_by(&mut app, 1.0, 0.0);
    assert!(
        app.document.buffer().has_selection(),
        "an already-armed drag keeps extending on any move"
    );
}

// --- Dragging past the page's left edge --------------------------------
//
// The PAINT-side half (selection wash / search-match / preedit /
// caret never spill past the active content clip) is proven over real
// pixel/geometry arithmetic in `render::tests::selection_clip_law` — that
// half needs a GPU pipeline. This half is the STATE seam: hit-testing a drag
// that has traveled into the margin, well left of the writing column, must
// clamp to the nearest valid document position (the row's own first
// column) — never panic, never leave a stale/partial selection — and this
// is a purely GPU-less `App`-level concern.

#[test]
fn a_drag_past_the_pages_left_edge_clamps_to_the_rows_first_column() {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello\nworld");
    // Press mid-word on line 1: "wor|ld" (char 9 = line-1 start(6) + col 3).
    press_at_row_col(&mut app, 1, 3, false);
    assert_eq!(app.document.buffer().cursor_char(), 9);
    assert!(!app.document.buffer().has_selection());

    // Drag the pointer far LEFT of the writing column's own origin — well past
    // the page's left edge, deep in the margin — but keep it on the SAME row.
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    let (x, y) = app.input.pointer.cursor_px;
    move_by(
        &mut app,
        TEXT_LEFT.0 - 500.0 - x,
        TEXT_TOP.0 + 1.5 * m.line_height - y,
    );

    // The clamp lands on the row's OWN first column (char 6, "world"'s own
    // start) — never a negative/OOB index, never the document's absolute
    // start (line 0) just because x went negative; y still picks the row.
    assert!(
        app.document.buffer().has_selection(),
        "travel far past the slop must still arm a real drag"
    );
    assert_eq!(
        app.document.buffer().selection_range(),
        Some((6, 9)),
        "the drag clamps to the row's nearest valid column, not a narrower selectable range"
    );

    // Dragging even FURTHER left changes nothing further — the clamp is
    // idempotent, not a crash-prone unbounded extrapolation.
    let (x, y) = app.input.pointer.cursor_px;
    move_by(
        &mut app,
        TEXT_LEFT.0 - 100_000.0 - x,
        TEXT_TOP.0 + 1.5 * m.line_height - y,
    );
    assert_eq!(app.document.buffer().selection_range(), Some((6, 9)));
}

#[test]
fn release_disarms_so_the_next_press_is_slop_gated_again() {
    // The armed flag must not leak across gestures: after a real drag then
    // release, a FRESH press elsewhere followed by a sub-slop move must not
    // arm — proves `drag_armed` resets per press (belt-and-braces with the
    // release-time reset).
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    press_at_col(&mut app, 0, false);
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    move_by(&mut app, 6.0 * m.char_width, 0.0);
    assert!(app.document.buffer().has_selection());
    // This is the one release transition, not a mirror of its statements:
    // removing `finish_text_drag`'s armed reset makes this law fail by name.
    app.input.finish_text_drag();
    assert!(
        !app.input.pointer.dragging && !app.input.pointer.drag_armed,
        "release retires both the active drag and its slop latch"
    );
    press_at_col(&mut app, 3, false);
    assert!(
        !app.document.buffer().has_selection(),
        "a fresh plain click drops the old selection"
    );
    move_by(&mut app, DRAG_ARM_SLOP_PX - 0.1, 0.0);
    assert!(
        !app.document.buffer().has_selection(),
        "the new gesture is slop-gated again, not still armed"
    );
}

// --- Folded-selection state ----------------------------------------------------

/// A small nested-free markdown doc (no soft-wrap): row 0 # A hides a1,a2 when
/// folded; # B / b1 stay visible.
const FOLD_DOC: &str = "# A\na1\na2\n# B\nb1";

/// Drive selection state at a filtered document row after a live hit test has
/// resolved the row and column.
fn press_at_row_col(app: &mut App, row: usize, col: usize, shift: bool) {
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    app.input.pointer.cursor_px = (
        TEXT_LEFT.0 + col as f32 * m.char_width,
        TEXT_TOP.0 + (row as f32 + 0.5) * m.line_height,
    );
    let full = app.document.buffer().visible_line_to_full(row);
    let char_idx = app.document.buffer().line_col_to_char(full, col);
    app.press_at_char(char_idx, shift);
}

fn folded_app() -> App {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text(FOLD_DOC);
    app.document.set_cursor(0); // on # A
    // Fold # A -> hide a1,a2 (filtered: 0 # A / 1 # B / 2 b1).
    app.document.toggle_fold_at_cursor();
    assert!(
        app.document.buffer().folds().contains(&0),
        "precondition: # A is folded"
    );
    app
}

#[test]
fn clicking_the_heading_text_places_the_caret_without_expanding() {
    let mut app = folded_app();
    // Press ON the heading text (col 1, inside "# A"): the caret lands, the fold STAYS
    // collapsed — the affordance is the tail region past the text, not the text itself.
    press_at_row_col(&mut app, 0, 1, false);
    assert!(
        app.document.buffer().folds().contains(&0),
        "clicking the heading text does not expand the fold"
    );
    assert_eq!(
        app.document.buffer().cursor_line_col().0,
        0,
        "caret is on the heading line"
    );
}

#[test]
fn a_heading_jump_onto_a_hidden_line_reveals_its_fold() {
    // REVEALED PLACEMENT (folds): a Go-to-heading / margin-outline jump targeting a
    // line hidden inside a collapsed section must reveal it — the caret can never be
    // left logically inside a fold. `jump_to_line` routes through the placement owner.
    let mut app = folded_app(); // # A folded, hiding a1 (line 1) and a2 (line 2)
    app.jump_to_line(1); // jump onto the hidden a1
    assert!(
        app.document.buffer().folds().is_empty(),
        "a jump onto a hidden line revealed the fold"
    );
    assert_eq!(
        app.document.buffer().cursor_line_col().0,
        1,
        "caret parked on the now-visible line"
    );
}

#[test]
fn outline_click_target_maps_a_fold_filtered_row_back_to_the_raw_heading_line() {
    // THE BUG: `TextPipeline::outline_hit_line` (what a real outline
    // click hit-tests to) reports a row's line in FOLD-FILTERED space — with # A
    // folded, `# B`'s row sits at FILTERED line 1 (the two hidden a1/a2 rows above
    // it collapse away), even though `# B` truly lives on RAW document line 3.
    // `App::outline_click` used to hand that filtered line straight to
    // `jump_to_line`, landing on the wrong line (raw line 1 = the hidden a1) any
    // time a fold sits before the clicked heading. `outline_row_target_line` is the
    // exact seam the fix routes the hit-tested row through before jumping — this
    // pins its law without needing a live GPU hit test.
    let app = folded_app(); // # A folded (line 0), hides a1 (raw 1) / a2 (raw 2); # B is raw line 3.
    assert_eq!(
        app.outline_row_target_line(1),
        3,
        "outline row 1 (# B, filtered) must map back to its TRUE raw document line 3"
    );
    // The folded heading's OWN row (filtered 0) is unaffected — nothing hides above
    // a heading's own line.
    assert_eq!(
        app.outline_row_target_line(0),
        0,
        "# A's own row needs no remap"
    );

    // NO-FOLD CASE UNCHANGED: the identical document with nothing folded maps the
    // identity — today's no-fold outline click resolves to the same target as
    // before this fix.
    let mut plain = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    plain.document.set_text(FOLD_DOC);
    assert_eq!(
        plain.outline_row_target_line(1),
        1,
        "unfolded: row 1 (# B) is already its own raw line — the identity"
    );
}

#[test]
fn a_shift_click_across_a_collapsed_section_reveals_the_fold_it_spans() {
    // THE DRAG/SHIFT-CLICK REVEAL (Wave-4 neighbourhood): with # A folded, the caret
    // parks on # A (char 0). A shift-click on the b1 row (filtered row 2 -> full line
    // 4) drops the mark at char 0 and moves the cursor to line 4 — a selection that
    // spans the hidden a1/a2. It must never span a fold invisibly: the placement
    // owner reveals # A before the selection is shown.
    let mut app = folded_app();
    assert_eq!(
        app.document.buffer().cursor_char(),
        0,
        "precondition: caret on # A"
    );
    press_at_row_col(&mut app, 2, 0, true);
    assert!(
        app.document.buffer().folds().is_empty(),
        "a shift-click whose selection spans the fold reveals it"
    );
    assert!(
        app.document.buffer().has_selection(),
        "the shift-click built a selection"
    );
    let (start, end) = app.document.buffer().selection_range().unwrap();
    assert_eq!(start, 0, "mark stayed at the prior caret");
    // The far endpoint is the START of b1 — full line 4, now that the fold is open.
    assert_eq!(
        end,
        app.document.buffer().line_col_to_char(4, 0),
        "selection reaches b1"
    );
}

#[test]
fn a_drag_across_a_collapsed_section_reveals_every_fold_it_crosses() {
    // A plain press on # A (its heading text, not the tail) places the caret and arms
    // a char drag; dragging DOWN past the hidden a1/a2 onto b1 must reveal # A so the
    // growing selection never crosses hidden lines. Drives the real
    // on_press -> on_cursor_moved(drag) seam.
    let mut app = folded_app();
    press_at_row_col(&mut app, 0, 0, false); // caret on # A, drag armed on next travel
    assert!(
        app.input.pointer.dragging,
        "a press on the heading text arms a text drag"
    );
    assert!(
        app.document.buffer().folds().contains(&0),
        "the press alone does not reveal"
    );
    let m = Metrics::with_dpi(app.frame.zoom(), app.frame.dpi());
    // Travel two visible rows down (well past the slop) onto the b1 row.
    move_by(&mut app, 0.0, 2.0 * m.line_height);
    assert!(
        app.document.buffer().folds().is_empty(),
        "the drag crossing the fold revealed it"
    );
    assert!(
        app.document.buffer().has_selection(),
        "the drag extended a real selection"
    );
    assert_eq!(
        app.document.buffer().selection_range().unwrap(),
        (0, app.document.buffer().line_col_to_char(4, 0)),
        "selection runs from # A to b1 with nothing hidden inside it"
    );
}

#[test]
fn a_click_below_a_collapsed_section_lands_on_the_right_full_document_line() {
    // THE FOLD HIT-TEST REMAP: with # A folded, the render shapes "# A\n# B\nb1", so a
    // click on the 2nd VISIBLE row hit-tests to filtered line 1 — which must resolve to
    // FULL-document line 3 (# B), not rope line 1 (a1, hidden inside the fold). Without
    // the visible→full remap the caret would land on the wrong (hidden) line.
    let mut app = folded_app();
    press_at_row_col(&mut app, 1, 0, false);
    assert_eq!(
        app.document.buffer().cursor_line_col().0,
        3,
        "click on filtered row 1 lands on full line 3 (# B), not the hidden a1"
    );
    assert!(
        app.document.buffer().folds().contains(&0),
        "clicking # B does not disturb the fold"
    );
}

/// `App::overlay_wheel` is a SECOND deliberate-crossing
/// input path that drives `OverlayState::move_sel` exactly like the keyboard
/// (the keyboard-baseline stamp was initially wired only into
/// `App::apply` / `ReplaySession::apply_chord`, missing this one — the wheel
/// is dispatched straight from `on_mouse_wheel`, never through `apply`). From
/// a COLD START (the overlay opened by keyboard, the pointer never having
/// hovered a row yet — `last_hover_px` still `None`), a wheel scroll used to
/// leave the gate armed with nothing: `hover_at`'s own cold-start rule reads
/// a `None` baseline as unconditional real motion, so the very next
/// `hover_at` call — even an exact repeat of the SAME resting pixel, the kind
/// of platform-synthesized duplicate a relayout/redraw can produce — would
/// silently steal the wheel-driven selection. Reproduces the hazard
/// ("a list window scrolling under a stationary cursor... yank the selection
/// out from under the user") through the wheel rather than the keyboard.
#[test]
fn wheel_scroll_from_cold_start_does_not_expose_selection_to_the_next_hover_check() {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    let corpus: Vec<String> = (0..40).map(|i| format!("row{i}")).collect();
    let ov = crate::overlay::OverlayState::new(
        crate::overlay::OverlayKind::Goto,
        corpus,
        vec![],
        vec![],
    );
    assert_eq!(
        ov.last_hover_px, None,
        "cold start: the pointer has never hovered a row"
    );
    app.workspace_state.install_overlay_for_test(ov);
    // The pointer is resting somewhere (its OS position is always something;
    // it just hasn't generated a hover check on this overlay yet).
    app.input.pointer.cursor_px = (123.0, 45.0);

    // A wheel scroll deep enough to move the window (Goto's `window_rows` is
    // 12; 22 notches lands `selected` at 22, well past the first page).
    app.overlay_wheel(-22.0);
    let ov = app
        .workspace_state
        .overlay_mut()
        .expect("overlay stays open across a wheel scroll");
    assert_eq!(
        ov.selected, 22,
        "the wheel drove the selection exactly like ↓ would"
    );

    // The exact same resting pixel — no travel at all — now hit-tests to a
    // DIFFERENT row (15) because the window scrolled under it, mirroring a
    // relayout/redraw's incidental re-check. Row 15 sits safely inside the
    // post-scroll visible band ([11, 23) for Goto's window), so a steal here
    // can only come from the missing cold-start stamp, never from
    // `hover_select`'s own separate visible-band rejection.
    let (px, py) = app.input.resting_pointer().px();
    let stolen = ov.hover_at(px, py, Some(15));
    assert!(
        !stolen,
        "a stationary pointer re-check after a wheel scroll must not steal the selection"
    );
    assert_eq!(
        ov.selected, 22,
        "the wheel-driven selection survives the stray re-check"
    );
}

/// `App::step_drag_scroll` (`app/input/mouse.rs`) is the drag-scroll owner
/// both `on_drag` and `App::schedule_drag_scroll` (the idle-timer re-arm)
/// drive, gated on `App::drag_scroll_primed`. A hermetic test `App` never has
/// a live GPU pipeline (only a real window builds one), so this pins the
/// SAME degrade-to-no-op gate `on_drag` itself already relies on (`let
/// Some(_) = self.frame.gpu() else { return };`) — the pixel geometry
/// (`drag_scroll_overshoot`, the composed `drag_scroll_step`) is exercised
/// at the pipeline level instead (`render::tests::drag_scroll`), where a
/// real device is available.
#[test]
fn step_drag_scroll_is_a_no_op_without_a_live_gpu() {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    app.input.pointer.dragging = true;
    app.input.pointer.drag_armed = true;
    app.input.pointer.cursor_px = (0.0, -500.0);
    assert!(
        !app.drag_scroll_primed(),
        "off a live GPU there is no viewport to read an overshoot against"
    );
    assert!(
        !app.step_drag_scroll(),
        "off a live GPU there is nothing to scroll"
    );
}

/// THE PHANTOM-SCROLL GUARD: a press that hasn't crossed the drag-arm slop
/// yet (`dragging` true, `drag_armed` still false — an ordinary stationary
/// click, or a drag whose first `CursorMoved` hasn't arrived) must never
/// scroll, no matter how far past the edge its press coordinates happen to
/// sit. Without gating on `drag_armed`, the idle-timer re-arm
/// (`App::schedule_drag_scroll`) would scroll — and extend a selection —
/// under a motionless pointer, exactly the phantom-selection class
/// `DRAG_ARM_SLOP_PX` exists to keep out of an ordinary click.
#[test]
fn drag_scroll_is_never_primed_before_the_drag_arm_slop_is_crossed() {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");
    app.input.pointer.dragging = true;
    app.input.pointer.drag_armed = false;
    app.input.pointer.cursor_px = (TEXT_LEFT.0, TEXT_TOP.0 - 500.0);
    assert!(
        !app.drag_scroll_primed(),
        "an unarmed drag must never be primed to scroll, however far past the edge the press sits"
    );
    assert!(
        !app.step_drag_scroll(),
        "an unarmed drag must not scroll even if a caller reaches step_drag_scroll directly"
    );
}

/// The drag-scroll clock (`PointerInput::drag_scroll_last_tick`) must not
/// survive past its drag: a NEW press priming a stale timestamp would hand
/// the very first post-press drag-scroll tick a huge, bogus `dt` computed
/// against the PREVIOUS drag's last tick.
#[test]
fn drag_scroll_tick_resets_on_release_and_does_not_survive_into_the_next_press() {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.document.set_text("hello world");

    app.input.pointer.drag_scroll_last_tick = Some(app.frame.now());
    app.input.finish_text_drag();
    assert!(
        app.input.pointer.drag_scroll_last_tick.is_none(),
        "releasing a drag must forget its drag-scroll clock"
    );

    app.input.pointer.drag_scroll_last_tick = Some(app.frame.now());
    app.input.pointer.cursor_px = (TEXT_LEFT.0, TEXT_TOP.0);
    app.press_at_char(0, false);
    assert!(
        app.input.pointer.drag_scroll_last_tick.is_none(),
        "a fresh press must not inherit a stale drag-scroll clock from an earlier drag"
    );
}

/// **A SUMMONED MODAL CARD SWALLOWS THE WHEEL WHOLE** — it neither dismisses
/// the card (a wheel is not the press-owns-the-next-input intent a click is)
/// nor reaches ANY of `on_mouse_wheel`'s branches below the guard, proved on
/// the one branch that is genuinely GPU-free and so headlessly observable:
/// ⌘-wheel ZOOM (`App::new_hermetic` builds no window, so plain vertical
/// document scroll is itself resolved through `gpu.pipeline.scroll_by_px` —
/// GPU-gated and a no-op on every headless `App` regardless of this guard;
/// zoom is a bare `Frame` field with no such dependency, and this guard fires
/// before the wheel is even inspected for its zoom modifier, so zoom is
/// exactly as swallowed as scroll would be). Swept across all three modal
/// cards — About, Lifetime, Streaks — since the guard is one predicate over
/// all three, not a per-card check.
///
/// NON-VACUITY: for each card, the IDENTICAL ⌘-wheel with the card closed
/// DOES move zoom immediately afterward — proving this fixture's wheel is
/// able to mutate state at all, so the "card open ⇒ zoom unchanged"
/// assertion above is not satisfied merely because nothing in this fixture
/// can ever move. Removing the guard in `on_mouse_wheel` turns the first
/// assertion of each iteration red.
#[test]
fn wheel_with_a_modal_card_open_is_swallowed_whole_not_dismissed_or_applied() {
    let _g = crate::testlock::serial();
    crate::about::set_open(false);
    crate::lifetime::set_open(false);
    crate::streaks::set_open(false);

    type CardGates = (&'static str, fn(bool), fn() -> bool);
    let cards: [CardGates; 3] = [
        ("about", crate::about::set_open, crate::about::about_open),
        (
            "lifetime",
            crate::lifetime::set_open,
            crate::lifetime::lifetime_open,
        ),
        (
            "streaks",
            crate::streaks::set_open,
            crate::streaks::streaks_open,
        ),
    ];

    for (name, set_open, is_open) in cards {
        let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
        app.document.set_text("hello world\n");
        app.input
            .set_modifiers(winit::event::Modifiers::from(ModifiersState::SUPER));

        set_open(true);
        let before = app.frame.zoom();
        app.on_mouse_wheel(MouseScrollDelta::LineDelta(0.0, -3.0));
        assert_eq!(
            app.frame.zoom(),
            before,
            "{name}: a modal card up must swallow the wheel outright — zoom must not move"
        );
        assert!(
            is_open(),
            "{name}: the wheel must not dismiss the card either — swallow, not dismiss"
        );
        set_open(false);

        // Non-vacuity precondition, checked AFTER (not instead of) the real
        // assertions above: the same wheel, same App, card now closed.
        let before2 = app.frame.zoom();
        app.on_mouse_wheel(MouseScrollDelta::LineDelta(0.0, -3.0));
        assert_ne!(
            app.frame.zoom(),
            before2,
            "{name}: precondition failed — this fixture's ⌘-wheel must be ABLE \
             to move zoom once no card is up, or the assertion above proves nothing"
        );
    }
}

/// **A PRESS ON THE STREAKS CARD'S OWN SURFACE PAGES; OFF IT, DISMISSES —**
/// the decision half of the click-routing split in `mouse_button.rs`
/// (`press_with_card_open`), driven with the real hit-test's ANSWER rather
/// than real geometry: the geometry lookup itself
/// (`streaks_card_rect`, read through `self.frame.gpu()`) is live-only, the
/// same as every other pixel-position hit-test in that file (`popover_hit`,
/// `start_action_at`) — `render::tests::streaks_card_ink` proves that
/// geometry matches the drawn card instead. This law proves the round trip:
/// paging twice returns to the starting page (mirroring the ←/→ key
/// intercept's own round-trip law), the card stays open the whole time an
/// on-card press lands, and an off-card press dismisses exactly like every
/// other summoned-card press.
#[test]
fn press_with_card_open_pages_streaks_on_card_and_dismisses_off_it() {
    let _g = crate::testlock::serial();
    crate::about::set_open(false);
    crate::lifetime::set_open(false);
    crate::streaks::set_open(true);
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    assert_eq!(
        crate::streaks::card_view(),
        crate::streaks::CardView::Heatmap,
        "precondition: a fresh summon opens on the habit view"
    );

    app.press_with_card_open(true);
    assert_eq!(
        crate::streaks::card_view(),
        crate::streaks::CardView::Cumulative,
        "an on-card press pages, the same flip the ←/→ key intercept performs"
    );
    assert!(
        crate::streaks::streaks_open(),
        "the card stays open across an on-card press"
    );

    app.press_with_card_open(true);
    assert_eq!(
        crate::streaks::card_view(),
        crate::streaks::CardView::Heatmap,
        "a second on-card press pages back — the round trip"
    );
    assert!(crate::streaks::streaks_open());

    app.press_with_card_open(false);
    assert!(
        !crate::streaks::streaks_open(),
        "a press OFF the card dismisses it, exactly like every other summoned-card press"
    );

    crate::streaks::set_open(false);
}
