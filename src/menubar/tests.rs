//! `menubar`'s own unit tests — the pure bar-height/DPI arithmetic, the sliver-fix
//! bleed math, the global toggle/dropdown state, and the title/drop hit-testing.
//! Carved out of `menubar.rs` as a sibling module so the production file stays
//! inside its size mark — `code-health.py`'s `production()` exempts a file named
//! `tests.rs` precisely so carving an inline `mod tests` out of an oversized module
//! is a real remedy (the same move `readout.rs` / `readout/tests.rs` already use).

use super::*;

/// `bar_height` IS DPI-INVARIANT AT MATCHED LOGICAL GEOMETRY, WITH A PRESENCE
/// FLOOR. This is the PURE half — the row-count-style arithmetic, swept without a
/// device. The live-pipeline half (whether `TextPipeline::menubar_reserve` / the
/// card's height budget move together with this fn) needs a `TextPipeline` from
/// `src/render/tests/mod.rs::headless_pipeline`, which this module does not reach.
///
/// `line_height` here stands in for the caller's already-scaled
/// `Metrics::line_height`; sweeping `scale` while holding the LOGICAL line height
/// fixed and dividing the pad term back out proves `BAR_PAD_Y` moves WITH it
/// rather than getting added in raw. BOTH SIDES: (1) the device-px answer must
/// actually MOVE as scale changes — ruling out a scale-blind fn that would
/// trivially look invariant — and (2) the recovered LOGICAL answer must be
/// identical at every tier AND equal to the authored `line_height +
/// 2*BAR_PAD_Y.0` — a presence floor, since `Logical(0.0)` is perfectly
/// DPI-invariant too and would pass side (2) alone.
#[test]
#[allow(clippy::assertions_on_constants)] // the constant IS the subject under test
fn bar_height_is_dpi_invariant_at_matched_logical_geometry_with_a_presence_floor() {
    assert!(
        BAR_PAD_Y.0 > 0.0,
        "presence floor: BAR_PAD_Y must not be zeroed"
    );
    const TIERS: [f32; 4] = [1.0, 1.5, 2.0, 3.0];
    let logical_line_height = 20.0f32;
    let baseline = bar_height(logical_line_height, 1.0);

    let mut logical_answers = Vec::with_capacity(TIERS.len());
    for &scale in &TIERS {
        let physical_line_height = logical_line_height * scale;
        let h = bar_height(physical_line_height, scale);
        // SIDE ONE: the fn is not scale-blind — its device-px answer actually
        // moves once scale departs from 1.0.
        if scale != 1.0 {
            assert_ne!(
                h, baseline,
                "scale {scale}: bar_height must not be scale-blind"
            );
        }
        logical_answers.push(h / scale);
    }
    // SIDE TWO: every tier recovers the SAME logical answer, and it is the
    // authored constant — not just "some" invariant value (the trap a deleted
    // pad would also satisfy).
    let want_logical = logical_line_height + 2.0 * BAR_PAD_Y.0;
    for (&scale, &got) in TIERS.iter().zip(logical_answers.iter()) {
        assert!(
            (got - want_logical).abs() < 1e-4,
            "scale {scale}: recovered logical bar height {got} != authored \
             line_height + 2*BAR_PAD_Y ({want_logical}) — BAR_PAD_Y is not \
             scaling with line_height"
        );
    }
}

/// THE SLIVER FIX, pure: a rect flush on all three canvas-touching sides (the
/// bar's own ground strip) bleeds top/left/right by `EDGE_BLEED_PX`, and its
/// BOTTOM (never a canvas edge for the bar) is untouched.
#[test]
fn bleed_extends_every_flush_edge_and_leaves_the_bottom_alone() {
    let rect = [0.0, 0.0, 1200.0, 32.0];
    let bled = bleed_to_canvas_edges(rect, 1200.0);
    assert_eq!(bled[0], -EDGE_BLEED_PX, "left bleeds past x=0");
    assert_eq!(bled[1], -EDGE_BLEED_PX, "top bleeds past y=0");
    assert_eq!(
        bled[2],
        1200.0 + 2.0 * EDGE_BLEED_PX,
        "width bleeds on both flush sides"
    );
    assert_eq!(
        bled[3],
        32.0 + EDGE_BLEED_PX,
        "height bleeds on the top side only"
    );
    // The bottom edge (y + h) moves by exactly the top bleed, i.e. the BOTTOM
    // itself (a non-flush edge) never moved: bled_y + bled_h == rect_y + rect_h.
    assert_eq!(
        bled[1] + bled[3],
        rect[1] + rect[3],
        "the bottom edge itself is unmoved"
    );
}

#[test]
fn bleed_leaves_interior_left_and_right_edges_untouched() {
    let rect = [400.0, 0.0, 80.0, 32.0]; // nowhere near x=0 or x=1200
    let bled = bleed_to_canvas_edges(rect, 1200.0);
    assert_eq!(bled[0], 400.0, "left edge is interior, untouched");
    assert_eq!(bled[2], 80.0, "width is untouched (no side bled)");
    assert_eq!(
        bled[1], -EDGE_BLEED_PX,
        "top still bleeds — it's always flush for the bar"
    );
    assert_eq!(bled[3], 32.0 + EDGE_BLEED_PX);
}

#[test]
fn bleed_is_independent_per_side() {
    let rect = [1100.0, 0.0, 100.0, 32.0]; // right edge exactly at canvas_w=1200
    let bled = bleed_to_canvas_edges(rect, 1200.0);
    assert_eq!(bled[0], 1100.0, "left edge is interior, untouched");
    assert_eq!(
        bled[2],
        100.0 + EDGE_BLEED_PX,
        "right bleeds (flush to canvas_w)"
    );
    assert_eq!(bled[1], -EDGE_BLEED_PX);
}

/// A rect NOT touching the canvas top at all (hypothetical future caller) is
/// left exactly alone on every side — the fix only ever touches an edge that is
/// ACTUALLY flush with the canvas boundary, never a rect drawn purely elsewhere.
#[test]
fn bleed_is_a_total_no_op_off_every_canvas_edge() {
    let rect = [200.0, 50.0, 300.0, 40.0];
    assert_eq!(bleed_to_canvas_edges(rect, 1200.0), rect);
}

#[test]
fn globals_toggle_and_open_close() {
    let _g = crate::testlock::serial();
    let ambient = menu_bar_on(); // not `cfg!`: that reflects the host, not the initializer
    set_menu_bar_on(true);
    assert!(menu_bar_on());
    assert_eq!(toggle_open(2), Some(2));
    assert_eq!(open_menu(), Some(2));
    assert_eq!(toggle_open(2), None);
    assert_eq!(open_menu(), None);
    set_open(Some(1));
    assert_eq!(toggle_open(3), Some(3));
    set_open(Some(0));
    set_menu_bar_on(false);
    assert!(!menu_bar_on());
    assert_eq!(open_menu(), None, "a hidden bar holds no open dropdown");
    set_open(Some(0));
    assert!(toggle(), "toggle from off -> on");
    set_open(Some(0));
    assert!(!toggle(), "toggle from on -> off closes the dropdown");
    assert_eq!(open_menu(), None);
    set_menu_bar_on(ambient);
}

#[test]
fn boxes_from_extents_abut_at_midpoints() {
    let boxes = boxes_from_extents(&[(20.0, 50.0), (70.0, 96.0), (110.0, 146.0)]);
    assert_eq!(boxes.len(), 3);
    assert_eq!(boxes[0].band_left, 20.0 - TITLE_PAD_X);
    assert_eq!(boxes[0].text_left, 20.0);
    assert_eq!(boxes[0].text_right, 50.0);
    assert_eq!(boxes[0].band_right, (50.0 + 70.0) / 2.0);
    assert_eq!(boxes[1].band_left, boxes[0].band_right, "bands abut");
    assert_eq!(boxes[1].band_right, (96.0 + 110.0) / 2.0);
    assert_eq!(boxes[2].band_left, boxes[1].band_right);
    assert_eq!(boxes[2].band_right, 146.0 + TITLE_PAD_X);
}

#[test]
fn title_at_maps_x_across_the_whole_bar() {
    let boxes = boxes_from_extents(&[(20.0, 50.0), (70.0, 96.0), (110.0, 146.0)]);
    let bar_h = bar_height(20.0, 1.0);
    assert_eq!(
        title_at(&boxes, bar_h, boxes[0].text_left + 1.0, 4.0),
        Some(0)
    );
    assert_eq!(
        title_at(&boxes, bar_h, boxes[1].text_left + 1.0, 4.0),
        Some(1)
    );
    assert_eq!(
        title_at(&boxes, bar_h, boxes[2].band_right - 1.0, 4.0),
        Some(2)
    );
    assert_eq!(
        title_at(&boxes, bar_h, boxes[0].text_left, bar_h + 1.0),
        None
    );
    assert_eq!(title_at(&boxes, bar_h, 0.0, 4.0), None);
    assert_eq!(
        title_at(&boxes, bar_h, boxes[2].band_right + 5.0, 4.0),
        None
    );
}

#[test]
fn drop_rows_stack_uniform_slots_marking_separators() {
    let (rows, total) = drop_rows(&[false, false, true, false], 22.0);
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].top, 0.0);
    assert_eq!(rows[1].top, 22.0);
    assert_eq!(rows[2].top, 44.0);
    assert!(rows[2].separator, "the third row is the separator");
    assert_eq!(rows[3].top, 66.0);
    assert_eq!(total, 4.0 * 22.0);
}

#[test]
fn drop_item_at_hits_clickable_rows_only() {
    let anchor = TitleBox {
        band_left: 40.0,
        text_left: 52.0,
        text_right: 84.0,
        band_right: 90.0,
    };
    let bar_h = bar_height(20.0, 1.0);
    let (rows, total) = drop_rows(&[false, true, false], 22.0);
    let rect = drop_rect(&anchor, bar_h, 120.0, total);
    assert_eq!(rect[0], 40.0, "the dropdown left-aligns under its title");
    assert_eq!(rect[1], bar_h, "it hangs just below the bar");
    assert_eq!(rect[2], 120.0 + 2.0 * DROP_PAD_X);
    let (x, y) = (rect[0] + 5.0, rect[1] + DROP_PAD_Y + 1.0);
    assert_eq!(drop_item_at(rect, &rows, x, y), Some(0));
    // The separator row (index 1) is never a hit.
    let sep_y = rect[1] + DROP_PAD_Y + rows[1].top + 1.0;
    assert_eq!(drop_item_at(rect, &rows, x, sep_y), None);
    let third_y = rect[1] + DROP_PAD_Y + rows[2].top + 1.0;
    assert_eq!(drop_item_at(rect, &rows, x, third_y), Some(2));
    assert_eq!(drop_item_at(rect, &rows, rect[0] - 1.0, y), None);
    assert_eq!(drop_item_at(rect, &rows, x, rect[1] + 1.0), None);
}
