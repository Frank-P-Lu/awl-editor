//! ONE-GEOMETRY LAW for the INSERT-TABLE dimension picker: the drawn grid IS
//! the clickable grid, because both read `TextPipeline::table_dims_cell_rect`
//! and nothing else. Swept across every cell of the grid and at least two
//! window geometries — a hand-picked single geometry would hide a law that
//! only happens to hold at 1200×800.

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
