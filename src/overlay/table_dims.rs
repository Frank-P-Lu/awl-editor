//! The INSERT-TABLE dimension-picker sub-state: `TableDimsEdit` and its
//! `OverlayState` verbs, split out of `overlay::capture` to keep that file
//! under its file-size mark -- same module, own file, no ownership change.
//!
//! Shaped after `RenameEdit`'s minibuffer: a small piece of state the
//! journey/card lifecycle owns while it is `Some`, so the picker is
//! `--keys`-drivable and sidecar-visible with zero new lifecycle plumbing.
//! It differs from every `*_edit` sibling in what it draws (a small 2-D grid
//! of cells, not a text field): [`super::OverlayKind::window_rows`] et al.
//! never see it -- the render side owns a dedicated geometry + a dedicated
//! quad draw, both keyed off [`ViewState::overlay_table_dims`]
//! (`render::viewstate_def`) exactly like the SPELL popup's own dedicated
//! arm keys off `overlay_spell`.

use super::{OverlayKind, OverlayState, nav};

/// The smallest table worth inserting.
pub const MIN_DIM: usize = 1;
/// The largest row/column count the drawn grid ever offers -- ONE pair of
/// bounds, read by the render geometry (what the grid draws and what a click
/// can pick), the arrow-key clamp, and the typed-digit clamp, so the three
/// can never disagree about how big a table this picker can make.
pub const MAX_ROWS: usize = 8;
pub const MAX_COLS: usize = 8;
/// The seeded default -- modest enough that a bare `↵` is already useful.
pub const DEFAULT_ROWS: usize = 3;
pub const DEFAULT_COLS: usize = 2;

/// The live INSERT-TABLE minibuffer sub-state: the sculpted `rows`/`cols`
/// (always held pre-clamped to `[MIN_DIM, MAX_ROWS]`/`[MIN_DIM, MAX_COLS]`)
/// plus the free-text buffer a forgiving typed parse (`3x4` / `3 4`) reads.
/// While it is `Some`, the picker OWNS every key at the intercept level --
/// see `actions::overlay_nav::table_dims_intercept`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDimsEdit {
    pub rows: usize,
    pub cols: usize,
    /// The raw digits/separator typed so far, re-parsed on every keystroke
    /// (`parse_dims`). An arrow key or a pointer pick clears it, so a stale
    /// partial entry from one input mode can never resurface once the other
    /// mode has overwritten `rows`/`cols` directly.
    typed: String,
}

impl Default for TableDimsEdit {
    fn default() -> Self {
        Self {
            rows: DEFAULT_ROWS,
            cols: DEFAULT_COLS,
            typed: String::new(),
        }
    }
}

impl TableDimsEdit {
    /// The dim PROMPT line the card shows while sculpting, surfaced to the
    /// sidecar's `overlay.hint` via [`OverlayState::foot_hint`] -- the exact
    /// seam [`super::RenameEdit::prompt`] rides, so the live readout is
    /// `--keys`-verifiable with zero new sidecar plumbing.
    pub fn prompt(&self) -> String {
        format!(
            "{} × {} table   ↵ insert   Esc cancel",
            self.rows, self.cols
        )
    }
}

/// Parse a forgiving `"RxC"` / `"R C"` typed buffer into `(rows, cols)`, or
/// `None` while it doesn't yet name two positive integers -- an in-progress
/// single number (`"3"`) is not yet an answer, so the caller keeps whatever
/// `rows`/`cols` it already had. Pure; the separator is the first run of
/// non-digit characters, which can only ever be `x`/`X`/space, the only
/// non-digit characters [`OverlayState::table_dims_push`] admits.
fn parse_dims(typed: &str) -> Option<(usize, usize)> {
    let sep = typed.find(|c: char| !c.is_ascii_digit())?;
    let (a, rest) = typed.split_at(sep);
    let b = rest.trim_start_matches(|c: char| !c.is_ascii_digit());
    if a.is_empty() || b.is_empty() || !b.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let rows: usize = a.parse().ok()?;
    let cols: usize = b.parse().ok()?;
    (rows > 0 && cols > 0).then_some((rows, cols))
}

fn clamp_dim(v: i32, max: usize) -> usize {
    v.clamp(MIN_DIM as i32, max as i32) as usize
}

impl OverlayState {
    /// Build the fresh DIMENSION PICKER, seeded at [`DEFAULT_ROWS`] ×
    /// [`DEFAULT_COLS`]. Carries no candidate rows at all (`corpus` empty) --
    /// unlike every `*_edit` sibling, this card's content is never a row
    /// list.
    pub fn new_table_dims() -> Self {
        let mut s = Self::new_marked(
            OverlayKind::TableDims,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        );
        s.table_dims = Some(TableDimsEdit::default());
        s
    }

    /// `↑`/`↓` SCULPT the row count by `delta`, clamped to `[MIN_DIM,
    /// MAX_ROWS]`. Clears the typed buffer -- see [`TableDimsEdit::typed`]'s
    /// own doc for why the two input modes cannot share one buffer. A no-op
    /// with no table-dims edit active.
    pub fn table_dims_row_delta(&mut self, delta: i32) {
        let Some(td) = self.table_dims.as_mut() else {
            return;
        };
        td.rows = clamp_dim(td.rows as i32 + delta, MAX_ROWS);
        td.typed.clear();
    }

    /// `←`/`→` SCULPT the column count. See
    /// [`Self::table_dims_row_delta`]'s doc for the shared shape.
    pub fn table_dims_col_delta(&mut self, delta: i32) {
        let Some(td) = self.table_dims.as_mut() else {
            return;
        };
        td.cols = clamp_dim(td.cols as i32 + delta, MAX_COLS);
        td.typed.clear();
    }

    /// A typed digit / `x`/`X`/space extends the typed buffer; every other
    /// character is ignored (a numeric field, not free text). Once the
    /// buffer names two positive integers, `rows`/`cols` adopt the parse
    /// (clamped) immediately -- the forgiving `3x4`/`3 4` parse applies AS
    /// YOU TYPE rather than only on commit, so the drawn grid always shows
    /// exactly what `↵` would insert. A no-op with no table-dims edit
    /// active.
    pub fn table_dims_push(&mut self, c: char) {
        let Some(td) = self.table_dims.as_mut() else {
            return;
        };
        if !(c.is_ascii_digit() || c == 'x' || c == 'X' || c == ' ') {
            return;
        }
        td.typed.push(c);
        if let Some((rows, cols)) = parse_dims(&td.typed) {
            td.rows = rows.clamp(MIN_DIM, MAX_ROWS);
            td.cols = cols.clamp(MIN_DIM, MAX_COLS);
        }
    }

    /// Backspace pops the last typed character and re-parses; `rows`/`cols`
    /// stay at their last valid reading while the buffer is incomplete (an
    /// in-progress edit never blanks the readout). A no-op once the buffer
    /// is already empty -- arrows/clicks never populate it, so there is
    /// nothing to pop back to after them.
    pub fn table_dims_pop(&mut self) {
        let Some(td) = self.table_dims.as_mut() else {
            return;
        };
        td.typed.pop();
        if let Some((rows, cols)) = parse_dims(&td.typed) {
            td.rows = rows.clamp(MIN_DIM, MAX_ROWS);
            td.cols = cols.clamp(MIN_DIM, MAX_COLS);
        }
    }

    /// A pointer PICK: set `rows`/`cols` to the clicked 0-based `(row, col)`
    /// cell outright -- the mouse's own route to the exact state an
    /// arrow-key sculpt reaches, never a shortcut around it (the picker's
    /// own accept/commit gesture, `↵`, stays the one insertion door). Clamped
    /// defensively even though every caller already bounds `row`/`col` to
    /// the drawn grid. A no-op with no table-dims edit active.
    ///
    /// This is ALSO the hover-preview's own write path
    /// ([`Self::table_dims_hover_at`]): a click commits (the mouse layer
    /// follows this call with `Action::Newline`) and a hover merely previews
    /// (the mouse layer does not), but both reach `rows`/`cols` through this
    /// one function -- there is no second, hover-only copy of this state to
    /// disagree with a keyboard sculpt or a click.
    pub fn table_dims_pick(&mut self, row: usize, col: usize) {
        let Some(td) = self.table_dims.as_mut() else {
            return;
        };
        td.rows = (row + 1).clamp(MIN_DIM, MAX_ROWS);
        td.cols = (col + 1).clamp(MIN_DIM, MAX_COLS);
        td.typed.clear();
    }

    /// THE DIMENSION PICKER'S OWN REAL-MOTION GATE — [`Self::table_dims_pick`]'s
    /// hover-preview door, sharing [`Self::last_hover_px`]/
    /// [`nav::HOVER_MOVE_SLOP_PX`] with the candidate-row list's own
    /// [`Self::hover_at`] rather than growing a second anchor: the two gates
    /// are mutually exclusive by `kind` (a `TableDims` card carries no
    /// candidate rows, an ordinary picker carries no table-dims edit), so one
    /// field safely serves both.
    ///
    /// Without this gate, a platform-synthesized duplicate `CursorMoved` at
    /// an UNMOVED pixel -- fired right after an arrow-key/typed-digit sculpt
    /// re-anchors the baseline via [`Self::arm_hover_baseline`] (called
    /// generically after every keyboard action on any open overlay, this
    /// card included) -- would re-hit-test the stationary pointer and
    /// silently REVERT the keyboard's own change back to whatever cell rests
    /// under it: the exact "hover steals a keyboard selection" hazard
    /// `hover_at` exists to close for the row list, now for a grid instead.
    /// `hit` is the cell the CALLER already resolved under `(px, py)` (a
    /// plain injected value, not a pipeline call -- keeps this
    /// pure/unit-testable, mirroring `hover_at`'s own shape).
    ///
    /// Returns whether the hover actually changed the previewed dims, so the
    /// caller knows whether a redraw is owed.
    pub fn table_dims_hover_at(&mut self, px: f32, py: f32, hit: Option<(usize, usize)>) -> bool {
        let moved = match self.last_hover_px {
            None => true,
            Some((lx, ly)) => {
                let dx = px - lx;
                let dy = py - ly;
                dx * dx + dy * dy > nav::HOVER_MOVE_SLOP_PX * nav::HOVER_MOVE_SLOP_PX
            }
        };
        if !moved {
            return false;
        }
        self.last_hover_px = Some((px, py));
        let Some((row, col)) = hit else {
            return false;
        };
        let before = self.table_dims_target();
        self.table_dims_pick(row, col);
        before != self.table_dims_target()
    }

    /// The commit target: `Some((rows, cols))` while a table-dims edit is
    /// active, `None` otherwise.
    pub fn table_dims_target(&self) -> Option<(usize, usize)> {
        if !self.kind.is_local_insertion_card() {
            return None;
        }
        self.table_dims.as_ref().map(|td| (td.rows, td.cols))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_the_modest_default() {
        let ov = OverlayState::new_table_dims();
        assert_eq!(ov.table_dims_target(), Some((DEFAULT_ROWS, DEFAULT_COLS)));
        assert_eq!(
            ov.rows.len(),
            0,
            "no candidate row list -- this card is not a list"
        );
    }

    #[test]
    fn typed_kind_owns_whether_table_dimensions_reach_the_renderer() {
        let mut ov = OverlayState::new_table_dims();
        assert_eq!(ov.table_dims_target(), Some((DEFAULT_ROWS, DEFAULT_COLS)));

        // Preserve the edit state but change the overlay identity. A stale
        // table-dimensions payload must not project the local-card reason into
        // an unrelated overlay's ViewState.
        ov.kind = OverlayKind::Command;
        assert_eq!(ov.table_dims_target(), None);
    }

    #[test]
    fn arrows_sculpt_and_clamp_at_both_bounds() {
        let mut ov = OverlayState::new_table_dims();
        ov.table_dims_row_delta(1);
        ov.table_dims_col_delta(1);
        assert_eq!(
            ov.table_dims_target(),
            Some((DEFAULT_ROWS + 1, DEFAULT_COLS + 1))
        );
        // Down past MIN_DIM floors at MIN_DIM, never underflows/panics.
        for _ in 0..20 {
            ov.table_dims_row_delta(-1);
            ov.table_dims_col_delta(-1);
        }
        assert_eq!(ov.table_dims_target(), Some((MIN_DIM, MIN_DIM)));
        // Up past MAX_* caps at MAX_*.
        for _ in 0..20 {
            ov.table_dims_row_delta(1);
            ov.table_dims_col_delta(1);
        }
        assert_eq!(ov.table_dims_target(), Some((MAX_ROWS, MAX_COLS)));
    }

    #[test]
    fn typed_digits_parse_forgivingly_with_x_or_space_separator() {
        for (typed, want) in [
            ("3x4", (3, 4)),
            ("3X4", (3, 4)),
            ("3 4", (3, 4)),
            ("7x1", (7, 1)),
        ] {
            let mut ov = OverlayState::new_table_dims();
            for c in typed.chars() {
                ov.table_dims_push(c);
            }
            assert_eq!(ov.table_dims_target(), Some(want), "typed {typed:?}");
        }
    }

    #[test]
    fn an_incomplete_typed_number_leaves_the_last_valid_reading_untouched() {
        let mut ov = OverlayState::new_table_dims();
        ov.table_dims_push('5');
        // "5" alone names no second number yet -- the seeded default holds.
        assert_eq!(ov.table_dims_target(), Some((DEFAULT_ROWS, DEFAULT_COLS)));
        ov.table_dims_push('x');
        ov.table_dims_push('2');
        assert_eq!(ov.table_dims_target(), Some((5, 2)));
    }

    #[test]
    fn typed_digits_clamp_past_the_grid_ceiling() {
        let mut ov = OverlayState::new_table_dims();
        for c in "99x99".chars() {
            ov.table_dims_push(c);
        }
        assert_eq!(ov.table_dims_target(), Some((MAX_ROWS, MAX_COLS)));
    }

    #[test]
    fn backspace_pops_and_reparses_but_never_touches_arrow_set_values() {
        let mut ov = OverlayState::new_table_dims();
        ov.table_dims_row_delta(2); // rows now DEFAULT_ROWS+2, typed buffer empty
        ov.table_dims_pop(); // nothing to pop -- no-op
        assert_eq!(
            ov.table_dims_target(),
            Some((DEFAULT_ROWS + 2, DEFAULT_COLS))
        );
        for c in "6x3".chars() {
            ov.table_dims_push(c);
        }
        assert_eq!(ov.table_dims_target(), Some((6, 3)));
        ov.table_dims_pop(); // "6x" -- incomplete again, last valid reading holds
        assert_eq!(ov.table_dims_target(), Some((6, 3)));
    }

    #[test]
    fn an_arrow_key_clears_a_stale_partial_typed_buffer() {
        let mut ov = OverlayState::new_table_dims();
        ov.table_dims_push('2'); // partial, unparsed
        ov.table_dims_row_delta(1);
        // The stray "2" must not resurface and combine with a later digit.
        ov.table_dims_push('4');
        // "4" alone parses to nothing (no separator) -- the arrow-set rows/cols hold.
        assert_eq!(
            ov.table_dims_target(),
            Some((DEFAULT_ROWS + 1, DEFAULT_COLS))
        );
    }

    #[test]
    fn pointer_pick_sets_one_based_dims_from_zero_based_cell_and_clamps() {
        let mut ov = OverlayState::new_table_dims();
        ov.table_dims_pick(2, 4);
        assert_eq!(ov.table_dims_target(), Some((3, 5)));
        ov.table_dims_pick(99, 99);
        assert_eq!(ov.table_dims_target(), Some((MAX_ROWS, MAX_COLS)));
    }

    #[test]
    fn non_numeric_input_is_ignored() {
        let mut ov = OverlayState::new_table_dims();
        for c in "abc!".chars() {
            ov.table_dims_push(c);
        }
        assert_eq!(ov.table_dims_target(), Some((DEFAULT_ROWS, DEFAULT_COLS)));
    }

    /// THE HOVER-PREVIEW's own hit-mapping/selection-update seam, swept over
    /// the grid's four CORNERS -- exactly where an off-by-one in a
    /// 0-based-cell-to-1-based-dims mapping hides.
    #[test]
    fn hover_at_a_cell_sets_selection_to_that_cell_swept_over_every_grid_corner() {
        for (row, col) in [
            (0, 0),
            (0, MAX_COLS - 1),
            (MAX_ROWS - 1, 0),
            (MAX_ROWS - 1, MAX_COLS - 1),
        ] {
            let mut ov = OverlayState::new_table_dims();
            assert!(
                ov.table_dims_hover_at(10.0, 10.0, Some((row, col))),
                "the first hover at a fresh position always re-hit-tests: cell ({row},{col})"
            );
            assert_eq!(
                ov.table_dims_target(),
                Some((row + 1, col + 1)),
                "hovering cell ({row},{col}) must select it 1-based, mirroring a click"
            );
        }
    }

    /// HOVER AND A CLICK CAN NEVER DISAGREE: both reach `rows`/`cols` through
    /// the exact same `table_dims_pick` write (see its own doc) -- there is
    /// no second, hover-only selection state for the two to drift out of
    /// sync with.
    #[test]
    fn hover_and_a_click_reach_the_identical_selection_state() {
        let (row, col) = (MAX_ROWS - 1, MAX_COLS - 1);
        let mut hovered = OverlayState::new_table_dims();
        hovered.table_dims_hover_at(1.0, 1.0, Some((row, col)));
        let mut clicked = OverlayState::new_table_dims();
        clicked.table_dims_pick(row, col);
        assert_eq!(hovered.table_dims_target(), clicked.table_dims_target());
    }

    #[test]
    fn hover_off_every_cell_leaves_the_prior_selection_untouched() {
        let mut ov = OverlayState::new_table_dims();
        ov.table_dims_pick(2, 3);
        assert!(!ov.table_dims_hover_at(500.0, 500.0, None));
        assert_eq!(ov.table_dims_target(), Some((3, 4)));
    }

    /// THE REAL-MOTION GATE LAW, for the grid instead of the row list: a
    /// platform-synthesized duplicate `CursorMoved` at an UNMOVED pixel,
    /// arriving right after `arm_hover_baseline` re-anchors from a keyboard
    /// sculpt, must NOT revert the keyboard's own change -- "the keyboard
    /// path stays authoritative, the two never fighting" is this law.
    /// NON-VACUOUS: proves the hazard is real first (an UNGATED
    /// `table_dims_pick` at the same stale pixel really would clobber the
    /// keyboard's selection), the same shape
    /// `hover_at_gates_on_real_pointer_motion_not_a_relayout_hit_test_change`
    /// (`overlay/tests/hover_keyboard_nav.rs`) uses for the row list.
    #[test]
    fn a_stationary_duplicate_cursor_moved_never_reverts_a_keyboard_sculpt() {
        let mut ov = OverlayState::new_table_dims();
        // The user sculpts to (DEFAULT+3, DEFAULT+4) with the keyboard while
        // the pointer rests at a stale pixel that a hit-test resolves to a
        // DIFFERENT cell, (2, 2). `App::apply` re-anchors the hover baseline
        // to the pointer's CURRENT position after every keyboard action.
        ov.table_dims_row_delta(3);
        ov.table_dims_col_delta(4);
        let sculpted = ov.table_dims_target();
        assert_eq!(sculpted, Some((DEFAULT_ROWS + 3, DEFAULT_COLS + 4)));
        ov.arm_hover_baseline(50.0, 50.0);

        // PROVE THE HAZARD IS REAL: an UNGATED write at the same stale pixel
        // really would clobber the keyboard's own selection.
        let mut naive = ov.clone();
        naive.table_dims_pick(2, 2);
        assert_ne!(
            naive.table_dims_target(),
            sculpted,
            "an ungated re-hit-test really would flip the selection -- the hazard is real"
        );

        // THE ACTUAL LAW: the SAME stationary pixel, through the gated
        // `table_dims_hover_at`, must not move the selection.
        assert!(
            !ov.table_dims_hover_at(50.0, 50.0, Some((2, 2))),
            "a redraw-duplicate CursorMoved at an unmoved pixel must not report a hover move"
        );
        assert_eq!(
            ov.table_dims_target(),
            sculpted,
            "the keyboard sculpt must survive a stationary duplicate CursorMoved"
        );

        // Real travel PAST the slop DOES take over, landing on whatever cell
        // is now under the pointer.
        assert!(ov.table_dims_hover_at(50.0 + 20.0, 50.0, Some((2, 2))));
        assert_eq!(ov.table_dims_target(), Some((3, 3)));
    }
}
