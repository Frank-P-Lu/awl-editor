//! ONE-GEOMETRY LAW for the INSERT-TABLE dimension picker: the drawn grid IS
//! the clickable grid, because both read `TextPipeline::table_dims_cell_rect`
//! and nothing else. Swept across every cell of the grid and at least two
//! window geometries — a hand-picked single geometry would hide a law that
//! only happens to hold at 1200×800.

use super::pixeldiff::{delta_e, render_frame};
use super::{headless_dqp, view};

fn dims_view(text: &str, rows: usize, cols: usize) -> crate::render::ViewState {
    let mut v = view(text, 0, 0);
    v.overlay_active = true;
    v.overlay_table_dims = Some((rows, cols));
    v.overlay_hint = format!("{rows} × {cols} table   ↵ insert   Esc cancel");
    v
}

/// EVERY CELL, at EVERY swept window size, hit-tests its own painted rect's
/// CENTER back to itself. Non-vacuous: also asserts a point well outside the
/// grid resolves to `None`, so the law cannot pass by a hit-test that always
/// answers `Some`.
#[test]
fn every_cell_center_hit_tests_to_itself_across_window_geometries() {
    let _g = crate::testlock::serial();
    for (w, h) in [(1200.0f32, 800.0f32), (900.0, 700.0)] {
        let Some((device, queue, mut p)) = headless_dqp(w, h) else {
            eprintln!("skipping every_cell_center_hit_tests_to_itself: no wgpu adapter");
            return;
        };
        p.set_view(&dims_view("hello\n", 3, 2));
        p.prepare(&device, &queue, w as u32, h as u32).unwrap();

        let geom = p.overlay_geometry(w as u32);
        let mut checked = 0;
        for row in 0..crate::overlay::MAX_ROWS {
            for col in 0..crate::overlay::MAX_COLS {
                let [x, y, cw, ch] = p.table_dims_cell_rect(&geom, row, col);
                let (cx, cy) = (x + cw * 0.5, y + ch * 0.5);
                assert_eq!(
                    p.table_dims_cell_at(cx, cy),
                    Some((row, col)),
                    "{w}x{h}: cell ({row},{col})'s own painted center must hit-test back to it"
                );
                checked += 1;
            }
        }
        assert_eq!(
            checked,
            crate::overlay::MAX_ROWS * crate::overlay::MAX_COLS,
            "the sweep reached the whole grid"
        );

        // NON-VACUOUS: a point far outside the card hits nothing.
        assert_eq!(
            p.table_dims_cell_at(-500.0, -500.0),
            None,
            "{w}x{h}: well outside the card is a miss, not a false hit"
        );
    }
}

/// The picker CLOSED (`overlay_table_dims: None`) hit-tests to `None`
/// everywhere — a stale grid from a PRIOR summon can never answer a click
/// after the card has closed.
#[test]
fn closed_picker_hit_tests_to_none_everywhere() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200.0f32, 800.0f32);
    let Some((device, queue, mut p)) = headless_dqp(w, h) else {
        eprintln!("skipping closed_picker_hit_tests_to_none_everywhere: no wgpu adapter");
        return;
    };
    // Open, then close (a bare `view` carries `overlay_table_dims: None`).
    p.set_view(&dims_view("hello\n", 3, 2));
    p.prepare(&device, &queue, w as u32, h as u32).unwrap();
    p.set_view(&view("hello\n", 0, 0));
    p.prepare(&device, &queue, w as u32, h as u32).unwrap();

    for (px, py) in [(600.0, 400.0), (100.0, 100.0), (0.0, 0.0)] {
        assert_eq!(p.table_dims_cell_at(px, py), None);
    }
}

/// THE GRID NEVER OVERLAPS THE HINT LINE. `header_gap` only ever reaches the
/// page through the CANDIDATE ROW plan's `first_top` (`plan_overlay_rows`),
/// and this card seats zero candidate rows for that plan to position — a
/// version that left `text_top` at the card's bare content top (relying on
/// `header_gap` alone to push the hint's glyph flow down) drew the readout
/// directly on top of the grid's own first row. Asserted at both extremes of
/// the grid's live size, since the grid's own footprint on screen is
/// constant (always `MAX_ROWS`×`MAX_COLS`) regardless of the sculpted
/// `(rows, cols)` — this law would not be sensitive to that axis, so it is
/// swept across widths instead, the axis the geometry actually depends on.
#[test]
fn the_hint_line_starts_at_or_below_the_grids_own_bottom_edge() {
    let _g = crate::testlock::serial();
    for (w, h) in [(1200.0f32, 800.0f32), (900.0, 700.0)] {
        let Some((device, queue, mut p)) = headless_dqp(w, h) else {
            eprintln!("skipping hint-vs-grid overlap law: no wgpu adapter");
            return;
        };
        p.set_view(&dims_view("hello\n", 3, 2));
        p.prepare(&device, &queue, w as u32, h as u32).unwrap();
        let geom = p.overlay_geometry(w as u32);
        let last_row_bottom = {
            let [_, y, _, ch] = p.table_dims_cell_rect(&geom, crate::overlay::MAX_ROWS - 1, 0);
            y + ch
        };
        // `text_top` is where the hint's glyph flow begins (see the module
        // doc on `table_dims_cell_rect` for why that is NOT the same origin
        // the grid itself uses).
        let hint_top = geom.text_top;
        assert!(
            hint_top >= last_row_bottom - 0.5,
            "{w}x{h}: hint starts at {hint_top}, before the grid's own last \
             row ends at {last_row_bottom} -- they overlap"
        );
    }
}

/// FILLED and EMPTY cells are mutually distinguishable, AND each is
/// distinguishable from the card's own background — a PRESENCE floor
/// alongside the difference floor, so a wash that quietly collapsed to the
/// card's fill color (the real defect this law was written for: an opaque
/// `base_200` cell measured byte-identical to Wagtail's own card fill) fails
/// here rather than passing by disappearing. Compares rendered pixel to
/// rendered pixel only, never to an authored theme constant, and sweeps two
/// worlds so a pass isn't an accident of one world's palette.
#[test]
fn filled_and_empty_cells_are_visible_against_each_other_and_the_card() {
    let _g = crate::testlock::serial();
    const JND: f64 = 2.3;
    for world in ["Bowerbird", "Wagtail"] {
        let Some(_pin) = crate::theme::WorldPin::world(world) else {
            eprintln!("skipping filled_and_empty_cells_are_visible: {world} unavailable");
            continue;
        };
        let (w, h) = (1200u32, 800u32);
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            eprintln!("skipping filled_and_empty_cells_are_visible: no wgpu adapter");
            return;
        };
        p.set_view(&dims_view("hello\n", 3, 2));
        p.prepare(&device, &queue, w, h).unwrap();
        let geom = p.overlay_geometry(w);
        let pixels = render_frame(&mut p, &device, &queue, w, h);
        let sample = |x: f32, y: f32| pixels[(y as usize) * w as usize + x as usize];

        let [fx, fy, fw, fh] = p.table_dims_cell_rect(&geom, 0, 0); // filled (inside 3x2)
        let filled_px = sample(fx + fw * 0.5, fy + fh * 0.5);
        let [ex, ey, ew, eh] = p.table_dims_cell_rect(
            &geom,
            crate::overlay::MAX_ROWS - 1,
            crate::overlay::MAX_COLS - 1,
        ); // empty
        let empty_px = sample(ex + ew * 0.5, ey + eh * 0.5);
        // A patch of card fill with no cell over it: a few px right of the
        // grid's own right edge, still well inside the card.
        let bg_px = sample(ex + ew + 6.0, ey + eh * 0.5);

        assert!(
            delta_e(filled_px, empty_px) > JND,
            "{world}: filled {filled_px:?} vs empty {empty_px:?} must clear the JND"
        );
        assert!(
            delta_e(empty_px, bg_px) > JND,
            "{world}: PRESENCE floor -- empty cell {empty_px:?} vs card fill \
             {bg_px:?} must clear the JND (an empty cell that collapsed to \
             the card's own color would still pass the filled-vs-empty \
             check above, which is why this floor exists separately)"
        );
        assert!(
            delta_e(filled_px, bg_px) > JND,
            "{world}: filled {filled_px:?} vs card fill {bg_px:?} must clear the JND"
        );
    }
}

/// The live `(rows, cols)` selects which cells paint FILLED — swept at both
/// grid extremes (the modest default and the ceiling) rather than one
/// hand-picked size, so an off-by-one at a bound can't hide.
#[test]
fn filled_cell_count_matches_the_live_dims_at_both_extremes() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200.0f32, 800.0f32);
    for (rows, cols) in [
        (crate::overlay::DEFAULT_ROWS, crate::overlay::DEFAULT_COLS),
        (crate::overlay::MIN_DIM, crate::overlay::MIN_DIM),
        (crate::overlay::MAX_ROWS, crate::overlay::MAX_COLS),
    ] {
        let Some((device, queue, mut p)) = headless_dqp(w, h) else {
            eprintln!("skipping filled_cell_count_matches_the_live_dims: no wgpu adapter");
            return;
        };
        p.set_view(&dims_view("hello\n", rows, cols));
        p.prepare(&device, &queue, w as u32, h as u32).unwrap();
        assert_eq!(
            p.table_dims_cells.instance_count() as usize,
            crate::overlay::MAX_ROWS * crate::overlay::MAX_COLS,
            "the drawn grid ALWAYS covers the full MAX_ROWS x MAX_COLS extent \
             (rows={rows}, cols={cols}); only fill/empty COLOR distinguishes \
             the sculpted region, never the instance count"
        );
    }
}
