//! `readout`'s own unit tests — the corner-anchor placement math and the
//! notice's plate padding. Carved out of `readout.rs` as a sibling module so the
//! production file stays inside its size mark — `code-health.py`'s
//! `production()` exempts a file named `tests.rs` precisely so carving an
//! inline `mod tests` out of an oversized module is a real remedy.

use super::corner_origin;
use crate::render::CornerAnchor;

/// THE DEBUG PANEL is TOP-RIGHT: right-aligned to the CANVAS edge (8px inset),
/// top row — clear of the top-left margin the persistent outline now owns.
/// `menubar_reserve = 0.0` throughout (bar off — the pre-existing, byte-identical
/// placement); the bar-shown case is [`debug_panel_yields_to_shown_menu_bar`].
#[test]
fn debug_panel_anchors_top_right() {
    // Canvas 1000 wide, a 200px-wide block: its right edge sits 8px in from the
    // canvas edge (left = 1000 − 200 − 8 = 792), top row 8px down.
    let (left, top) = corner_origin(
        CornerAnchor::TopRight,
        200.0,
        18.0,
        1000.0,
        800.0,
        0.0,
        0.0,
        0.0,
    );
    assert!(
        (left - 792.0).abs() < 1e-3,
        "right edge hugs the canvas edge, got left={left}"
    );
    assert_eq!(top, 8.0, "the top row sits 8px down");
    // The block's right edge is a fixed 8px inset regardless of its width.
    let (l2, _) = corner_origin(
        CornerAnchor::TopRight,
        350.0,
        18.0,
        1000.0,
        800.0,
        0.0,
        0.0,
        0.0,
    );
    assert!(
        (l2 + 350.0 - (1000.0 - 8.0)).abs() < 1e-3,
        "right edge is width−8 for any block width"
    );
    // On a canvas too narrow for the block it never runs off the LEFT edge.
    let (l3, _) = corner_origin(
        CornerAnchor::TopRight,
        500.0,
        18.0,
        300.0,
        800.0,
        0.0,
        0.0,
        0.0,
    );
    assert_eq!(l3, 8.0, "clamps to the left inset on a tiny canvas");
}

/// THE MENUBAR-YIELD LAW: a shown bar pushes a top-anchored corner straight down
/// by its own reserve, never touching its horizontal placement — the yield is
/// corner-AGNOSTIC (decoupled from the horizontal math), witnessed here by two
/// TopRight placements at different label widths yielding the IDENTICAL top.
/// TopRight is the debug panel's own anchor (the sole top anchor today). The SAME
/// `menubar_reserve` accessor the document/outline/search-panel already fold in,
/// so the debug panel can never disagree with its siblings about where the bar's
/// bottom edge sits. `top ≥ bar_height` holds by construction (`8.0 + reserve`).
#[test]
fn top_anchors_yield_to_the_menu_bar_bottom_anchors_do_not() {
    let reserve = 32.0; // a representative shown-bar height
    let (_, top_right) = corner_origin(
        CornerAnchor::TopRight,
        200.0,
        18.0,
        1000.0,
        800.0,
        0.0,
        0.0,
        reserve,
    );
    assert_eq!(
        top_right,
        8.0 + reserve,
        "TopRight (the debug panel) yields by exactly the reserve"
    );
    assert!(
        top_right >= reserve,
        "the debug panel's top never sits above the bar's own bottom edge"
    );

    // The yield is purely VERTICAL and decoupled from horizontal placement: a
    // different label width lands the panel at a different left, yet the top is
    // unchanged — exactly the corner-AGNOSTIC property the law asserts (the
    // reserve pushes any top-anchored corner down by the same amount, whatever its
    // own horizontal math).
    let (left_wide, top_wide) = corner_origin(
        CornerAnchor::TopRight,
        500.0,
        18.0,
        1000.0,
        800.0,
        0.0,
        0.0,
        reserve,
    );
    let (left_narrow, top_narrow) = corner_origin(
        CornerAnchor::TopRight,
        100.0,
        18.0,
        1000.0,
        800.0,
        0.0,
        0.0,
        reserve,
    );
    assert_ne!(
        left_wide, left_narrow,
        "a wider label moves the panel horizontally"
    );
    assert_eq!(
        top_wide, top_narrow,
        "…but both yield the IDENTICAL vertical top"
    );
    assert_eq!(
        top_wide,
        8.0 + reserve,
        "which is exactly the reserve push (same accessor, same law)"
    );

    // Bottom / pointer anchors are UNTOUCHED by a nonzero reserve — a strip at the
    // TOP of the canvas never reaches them.
    let (_, bottom_right) = corner_origin(
        CornerAnchor::BottomRight,
        120.0,
        18.0,
        1000.0,
        800.0,
        100.0,
        600.0,
        reserve,
    );
    assert_eq!(
        bottom_right,
        800.0 - 18.0 - 8.0,
        "BottomRight ignores the bar reserve"
    );
    // THE NOTICE's own anchor is TOP-anchored, so it MUST take the reserve —
    // the arm a "bottom anchors ignore it" law could never have covered.
    let (_, top_center) = corner_origin(
        CornerAnchor::TopCenter,
        120.0,
        18.0,
        1000.0,
        800.0,
        100.0,
        600.0,
        reserve,
    );
    assert_eq!(
        top_center,
        crate::render::TEXT_TOP + reserve,
        "TopCenter seats the notice on the document's own first-row origin, \
         bar reserve included"
    );
    let (_, at_point) = corner_origin(
        CornerAnchor::AtPoint(50.0, 60.0),
        40.0,
        18.0,
        1000.0,
        800.0,
        0.0,
        0.0,
        reserve,
    );
    assert_eq!(
        at_point,
        (60.0_f32 - 18.0 - 10.0).max(4.0),
        "AtPoint ignores the bar reserve"
    );

    // `reserve = 0.0` (bar off) is byte-identical to the pre-round placement.
    let (_, top_right_off) = corner_origin(
        CornerAnchor::TopRight,
        200.0,
        18.0,
        1000.0,
        800.0,
        0.0,
        0.0,
        0.0,
    );
    assert_eq!(
        top_right_off, 8.0,
        "bar off: the panel keeps its plain 8px top inset"
    );
}

/// The docked corners keep their historical placement (TopRight is the only new
/// arm; the others are byte-identical to the pre-extraction inline math).
#[test]
fn docked_corners_keep_their_placement() {
    // Bottom-right: right-aligned to the writing COLUMN (col_left + col_width − w).
    let (l, t) = corner_origin(
        CornerAnchor::BottomRight,
        120.0,
        18.0,
        1000.0,
        800.0,
        100.0,
        600.0,
        0.0,
    );
    assert!((l - (100.0 + 600.0 - 120.0)).abs() < 1e-3);
    assert_eq!(t, 800.0 - 18.0 - 8.0);
}
