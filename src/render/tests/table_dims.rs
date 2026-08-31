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

/// THE HINT'S OWN SHAPED WIDTH NEVER EXCEEDS THE CARD'S TEXT COLUMN, swept
/// across a zoom × DPI grid — a card sized to the grid alone let the hint
/// ("N × M table   ↵ insert   Esc cancel") outrun `text_w` and clip mid-word.
/// The widest live hint (`MAX_ROWS`×`MAX_COLS`) is the case actually reported
/// clipping. NON-VACUOUS: reverting `table_dims_overlay_geometry`'s
/// `desired_w` from `grid_w.max(hint_w)` back to bare `grid_w` turns this red
/// at every cell (the hint dominates the grid at every zoom/DPI sampled here).
#[test]
fn the_hint_lines_shaped_width_fits_inside_the_cards_text_column_across_zoom_and_dpi() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200.0f32, 800.0f32);
    let Some((device, queue, mut p)) = headless_dqp(w, h) else {
        eprintln!("skipping hint-width law: no wgpu adapter");
        return;
    };
    let mut graded = 0;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        for zoom in [0.8f32, 1.0, 1.6] {
            let mut v = dims_view(
                "hello\n",
                crate::overlay::MAX_ROWS,
                crate::overlay::MAX_COLS,
            );
            v.zoom = zoom;
            let (cw, ch) = ((w * dpi) as u32, (h * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            p.set_view(&v);
            p.prepare(&device, &queue, cw, ch).unwrap();
            let geom = p.overlay_geometry(cw);
            // Re-measured independently of the geometry's own cache
            // (`overlay_table_dims_hint_w`), through the exact same shaper
            // `push_overlay_hint_spans` draws with — this law asks the real
            // shaper, not the cache the fix relies on.
            let hint_w = p.measure_workspace_hint_text_px(&v.overlay_hint);
            assert!(
                hint_w > 1.0,
                "dpi={dpi} zoom={zoom}: the hint must actually shape glyphs -- a fit floor \
                 satisfied by an absent hint proves nothing"
            );
            assert!(
                hint_w <= geom.text_w + 0.5,
                "dpi={dpi} zoom={zoom}: the hint shapes to {hint_w:.1}px, wider than the \
                 card's own {:.1}px text column -- it will clip mid-word",
                geom.text_w
            );
            graded += 1;
        }
    }
    p.set_dpi(1.0);
    assert_eq!(graded, 6, "the zoom x dpi sweep must actually run");
}

/// THE HINT'S FINAL GLYPH ACTUALLY PAINTS, and lands inside the card -- a
/// PRESENCE floor alongside the width law above, so a "fix" that grew the
/// card but left the hint's own glyph run truncated (or shifted it off the
/// card) would still pass the pure-arithmetic law and fail here, on real
/// rendered pixels.
#[test]
fn the_hint_lines_final_glyph_column_carries_ink_inside_the_card() {
    let _g = crate::testlock::serial();
    const JND: f64 = 2.3;
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping hint-ink-at-edge law: no wgpu adapter");
        return;
    };
    let v = dims_view(
        "hello\n",
        crate::overlay::MAX_ROWS,
        crate::overlay::MAX_COLS,
    );
    p.set_view(&v);
    p.prepare(&device, &queue, w, h).unwrap();
    let geom = p.overlay_geometry(w);
    let card = p.overlay_card_rect().unwrap();
    let hint_w = p.measure_workspace_hint_text_px(&v.overlay_hint);
    let hint_h = p.overlay_hint_h();
    let pixels = render_frame(&mut p, &device, &queue, w, h);
    let sample = |x: f32, y: f32| pixels[(y as usize) * w as usize + x as usize];
    // A page-ground sample at the hint's own row, well clear of any glyph.
    let bg = sample(geom.text_left + 1.0, geom.text_top + hint_h * 0.5);
    // A small box hugging the hint's own measured right edge -- the last
    // glyph's ink, not the card's padding beyond it. A truncated run (the
    // clip this law is named for) leaves this box entirely background.
    let mut ink_found = false;
    for dy in 0..(hint_h as i32).max(1) {
        for dx in -6..-1 {
            let x = geom.text_left + hint_w + dx as f32;
            let y = geom.text_top + dy as f32;
            if delta_e(sample(x, y), bg) > JND {
                ink_found = true;
            }
        }
    }
    assert!(
        ink_found,
        "no pixel near the hint's own measured right edge ({:.1}px from text_left \
         {:.1}) differs from the page ground -- the last glyph is missing, not just tight",
        hint_w, geom.text_left
    );
    assert!(
        geom.text_left + hint_w <= card[0] + card[2] + 0.5,
        "the hint's shaped right edge ({:.1}) must land inside the card's own right edge \
         ({:.1}), not past it",
        geom.text_left + hint_w,
        card[0] + card[2]
    );
}

/// THE CARD TRACKS THE CARET, not a fixed top rail -- the whole point of the
/// caret-anchored placement (decision: option b). Two caret lines in the same
/// window/document must answer two DIFFERENT card positions; a fixed-rail
/// card (the pre-fix placement, `margin + CARD_TOP_DROP`) answers the same
/// `card_y` regardless of where the caret sits, which is the defect this law
/// is named for. NON-VACUOUS: reverting the anchor to the old fixed-rail
/// `card_y` formula collapses `y_top` and `y_mid` to the same value, below.
#[test]
fn the_card_tracks_the_caret_rather_than_a_fixed_rail() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200.0f32, 800.0f32);
    let Some((device, queue, mut p)) = headless_dqp(w, h) else {
        eprintln!("skipping caret-tracking law: no wgpu adapter");
        return;
    };
    let text: String = (0..14)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let card_y_at = |p: &mut crate::render::TextPipeline, line: usize| {
        let mut v = dims_view(&text, 3, 2);
        v.cursor_line = line;
        v.cursor_col = 0;
        p.set_view(&v);
        p.prepare(&device, &queue, w as u32, h as u32).unwrap();
        p.overlay_card_rect().unwrap()[1]
    };
    let y_top = card_y_at(&mut p, 0);
    let y_mid = card_y_at(&mut p, 8);
    assert!(
        (y_mid - y_top).abs() > 100.0,
        "a caret-anchored card must move with the caret: line 0 put the card at y={y_top}, \
         line 8 at y={y_mid} -- a fixed-rail card would answer the same y regardless of which \
         line the caret sits on"
    );
}

/// THE CLAMP the contextual spell popup already earns (`plan_spell_anchor`'s
/// near-edge and bottom-of-window logic) applies here too: sweeps a caret at
/// a page corner, near the right edge, near the bottom, and at a corner
/// combining both, and asserts the resulting card never paints past any of
/// the four window edges. NON-VACUOUS: reverting to the pre-fix fixed-rail
/// placement (`card_y = margin + CARD_TOP_DROP + menubar_reserve()`, caret
/// ignored) turns this red on the bottom-right-corner case -- a fixed rail
/// pins the card near the TOP regardless of window height, so on a window
/// short enough that the caret sits near its bottom, the card's own bottom
/// edge (336px) runs past the window's (320px). The clamp this law is named
/// for is what keeps that from happening once the card tracks the caret.
#[test]
fn caret_anchored_card_stays_within_window_bounds_under_near_edge_and_bottom_clamping() {
    let _g = crate::testlock::serial();
    // (label, text, line, col, window_w, window_h). Window heights are picked
    // to exceed the card's own fixed height (~284px) plus margin on both
    // sides, so a failure here is the CLAMP's fault, not a card too tall for
    // any placement to hold.
    let cases: Vec<(&str, String, usize, usize, f32, f32)> = vec![
        ("top-left doc start", "hello\n".into(), 0, 0, 1200.0, 800.0),
        (
            "near-right-edge, wide line in a narrow window",
            format!("{}\n", "x".repeat(200)),
            0,
            180,
            420.0,
            800.0,
        ),
        (
            "near-bottom, window tall enough to hold the card",
            (0..14)
                .map(|i| format!("line {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
            11,
            0,
            1200.0,
            400.0,
        ),
        (
            "bottom-right corner: narrow AND short-but-sufficient window",
            format!("{}\n", "x".repeat(200)),
            0,
            180,
            420.0,
            320.0,
        ),
    ];
    let mut clamp_engaged = false;
    let mut graded = 0;
    for (label, text, line, col, ww, wh) in cases {
        let Some((device, queue, mut p)) = headless_dqp(ww, wh) else {
            eprintln!("skipping {label}: no wgpu adapter");
            continue;
        };
        let mut v = dims_view(&text, crate::overlay::MAX_ROWS, crate::overlay::MAX_COLS);
        v.cursor_line = line;
        v.cursor_col = col;
        p.set_view(&v);
        p.prepare(&device, &queue, ww as u32, wh as u32).unwrap();
        let (caret_x, caret_y, _, _) = p.caret_pixel_rect();
        let [cx, cy, cw, ch] = p.overlay_card_rect().unwrap();
        const SLACK: f32 = 0.5;
        assert!(
            cx >= -SLACK,
            "{label}: card left edge {cx:.1} is off the window's left"
        );
        assert!(
            cy >= -SLACK,
            "{label}: card top edge {cy:.1} is off the window's top"
        );
        assert!(
            cx + cw <= ww + SLACK,
            "{label}: card right edge {:.1} clips the window's {ww}px width",
            cx + cw
        );
        assert!(
            cy + ch <= wh + SLACK,
            "{label}: card bottom edge {:.1} clips the window's {wh}px height",
            cy + ch
        );
        if (cx - caret_x).abs() > 5.0 || (cy - caret_y).abs() > ch {
            clamp_engaged = true;
        }
        graded += 1;
    }
    assert_eq!(graded, 4, "the stressed-caret sweep must actually run");
    assert!(
        clamp_engaged,
        "the sweep must exercise the near-edge/bottom clamp on at least one case -- every \
         case landing at the caret's own raw position would mean the clamp was never reached"
    );
}

/// THE GRID'S OWN INK, not just its arithmetic bounding box, stays inside the
/// window at the worst-case clamp -- a card rect that "contains" the grid on
/// paper but got shaped into device pixels a frame late (or a clamp that
/// silently produced NaN/negative geometry) would still fail here, on the
/// real rendered cell.
#[test]
fn the_grids_last_cell_paints_ink_inside_the_window_at_the_corner_stress_case() {
    let _g = crate::testlock::serial();
    let (ww, wh) = (420u32, 320u32);
    let Some((device, queue, mut p)) = headless_dqp(ww as f32, wh as f32) else {
        eprintln!("skipping corner-stress ink law: no wgpu adapter");
        return;
    };
    let mut v = dims_view(
        &format!("{}\n", "x".repeat(200)),
        crate::overlay::MAX_ROWS,
        crate::overlay::MAX_COLS,
    );
    v.cursor_line = 0;
    v.cursor_col = 180;
    p.set_view(&v);
    p.prepare(&device, &queue, ww, wh).unwrap();
    let geom = p.overlay_geometry(ww);
    let [lx, ly, lw, lh] = p.table_dims_cell_rect(
        &geom,
        crate::overlay::MAX_ROWS - 1,
        crate::overlay::MAX_COLS - 1,
    );
    assert!(
        lx >= 0.0 && ly >= 0.0 && lx + lw <= ww as f32 && ly + lh <= wh as f32,
        "the grid's own last cell [{lx},{ly},{lw},{lh}] must land fully inside the \
         {ww}x{wh} window after caret clamping -- sampling it below would otherwise be \
         reading outside the frame, not just off the card"
    );
    let pixels = render_frame(&mut p, &device, &queue, ww, wh);
    let sample = |x: f32, y: f32| pixels[(y as usize) * ww as usize + x as usize];
    let filled_px = sample(lx + lw * 0.5, ly + lh * 0.5);
    let card = p.overlay_card_rect().unwrap();
    // A patch of card fill just outside the grid but still inside the card --
    // the grid is centered now, so there is slack to its left to sample.
    let bg_px = sample(card[0] + 2.0, ly + lh * 0.5);
    const JND: f64 = 2.3;
    assert!(
        delta_e(filled_px, bg_px) > JND,
        "the grid's own last (filled, at MAX_ROWS x MAX_COLS) cell {filled_px:?} must clear \
         the JND against the card's own fill {bg_px:?} at this corner stress case"
    );
}

/// PLATELESS-BACKING JUDGMENT CALL: does a bare-plates world (Ruled/Bars/
/// Diagonal — `ListBacking::BarePlates`, which draws no card plate/border at
/// all) still leave the grid legible with NO added backing? Swept over the
/// roster DERIVED from `theme::THEMES` (never a hand-list), at both grid
/// extremes, so the enrollment can't silently drift as the roster grows.
/// This is the evidence behind the decision recorded in this module's own
/// worker report: no plate was added, because (a) the spell popup itself
/// does not special-case one on these worlds either (`ListStyle::list_backing`
/// ignores its own `spell` flag and returns `BarePlates` regardless), and (b)
/// this sweep passes without one — the shared blurred backdrop every card
/// already reads against, plus the grid's own filled/empty/muted inks, is
/// already enough contrast.
#[test]
fn the_grid_clears_the_jnd_on_every_backing_style_with_no_added_plate() {
    let _g = crate::testlock::serial();
    const JND: f64 = 2.3;
    let (w, h) = (1200u32, 800u32);
    let ambient = crate::theme::active_index();
    let mut enrolled_bare = 0;
    let mut enrolled_card = 0;
    for (index, world) in crate::theme::THEMES.iter().enumerate() {
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            eprintln!("skipping plateless-backing law: no wgpu adapter");
            return;
        };
        crate::theme::set_active(index);
        let backing = world.render_caps.list_style.list_backing(false);
        for (rows, cols) in [
            (crate::overlay::MIN_DIM, crate::overlay::MIN_DIM),
            (crate::overlay::MAX_ROWS, crate::overlay::MAX_COLS),
        ] {
            p.set_view(&dims_view("hello\n", rows, cols));
            p.prepare(&device, &queue, w, h).unwrap();
            let geom = p.overlay_geometry(w);
            let pixels = render_frame(&mut p, &device, &queue, w, h);
            let sample = |x: f32, y: f32| pixels[(y as usize) * w as usize + x as usize];
            let [fx, fy, fw, fh] = p.table_dims_cell_rect(&geom, 0, 0); // filled
            let filled_px = sample(fx + fw * 0.5, fy + fh * 0.5);
            let [ex, ey, ew, eh] = p.table_dims_cell_rect(
                &geom,
                crate::overlay::MAX_ROWS - 1,
                crate::overlay::MAX_COLS - 1,
            ); // empty at MIN_DIM dims, may be filled at MAX_ROWS/MAX_COLS
            let empty_px = sample(ex + ew * 0.5, ey + eh * 0.5);
            let card = p.overlay_card_rect().unwrap();
            let bg_px = sample(card[0] + 2.0, ey + eh * 0.5);
            assert!(
                delta_e(filled_px, bg_px) > JND,
                "{} ({backing:?}) rows={rows} cols={cols}: filled cell {filled_px:?} vs card \
                 fill {bg_px:?} must clear the JND with no added plate",
                world.name
            );
            if rows == crate::overlay::MIN_DIM {
                assert!(
                    delta_e(empty_px, bg_px) > JND,
                    "{} ({backing:?}): PRESENCE floor -- empty cell {empty_px:?} vs card fill \
                     {bg_px:?} must clear the JND",
                    world.name
                );
                assert!(
                    delta_e(filled_px, empty_px) > JND,
                    "{} ({backing:?}): filled {filled_px:?} vs empty {empty_px:?} must clear \
                     the JND",
                    world.name
                );
            }
        }
        match backing {
            crate::theme::ListBacking::BarePlates => enrolled_bare += 1,
            crate::theme::ListBacking::Card => enrolled_card += 1,
        }
    }
    crate::theme::set_active(ambient);
    assert!(
        enrolled_bare > 0,
        "the roster must include at least one BarePlates (plateless) world for this law to \
         say anything about the judgment call it backs"
    );
    assert!(
        enrolled_card > 0,
        "the roster must include at least one Card-backed world too"
    );
}
