use super::*;

#[test]
fn bulk_dimensions_keep_the_same_last_valid_prefix_as_scalar_typing() {
    let _guard = crate::testlock::serial();
    for input in [
        "4x5 6",
        "4x9999999999999999999999999999999",
        "8x00",
        "invalid4X7",
        "0x8",
    ] {
        let mut bulk = OverlayState::new_table_dims();
        let mut scalar = bulk.clone();
        bulk.table_dims_insert(input);
        for c in input.chars() {
            scalar.table_dims_push(c);
        }
        assert_eq!(bulk.table_dims, scalar.table_dims, "{input}");
    }
}

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
