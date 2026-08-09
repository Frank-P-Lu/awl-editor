//! THE DOCUMENT'S FIRST-ROW TOP, THROUGH THE LIVE PIPELINE, AT EVERY DPI.
//!
//! The vertical twin of `column_left_dpi.rs`: `TEXT_TOP` was the
//! last untyped `f32` in the same family as `TEXT_LEFT`, read unscaled by `doc_top`,
//! `visible_lines_z` and `scroll.rs`. `render/geometry/tests.rs`'s
//! `visible_lines_z_is_dpi_invariant_at_matched_logical_geometry_with_a_presence_floor`
//! is the PURE half — the row-count arithmetic, swept without a device. These are the
//! claims that only the assembled pipeline can answer: `TextPipeline::text_origin_top`
//! is read by `doc_top`, the zoom anchor, `hit_test_scroll`, the margin Outline, the
//! corner notice and the capture sidecar, so a vertical origin that moves must move
//! all of them TOGETHER.
//!
//! The experiment is 307's / 314's: `--capture-dpi N` makes a `WxH` DEVICE canvas a
//! `(W/N)x(H/N)` LOGICAL window, so every tier here grows its physical canvas in
//! lockstep with `dpi` to hold the LOGICAL window fixed. Comparing two tiers at one
//! device size compares two different windows and proves nothing.
//!
//! Unlike `TEXT_LEFT`'s rail branch (gated on headings via `outline_wants_rail`),
//! `text_origin_top` sits on the document's UNCONDITIONAL default path (`doc_top`'s
//! `None` arm — the only arm a non-comparison buffer ever takes), so a headingless
//! fixture already reaches it. The axis this file exists to sweep instead is
//! `MENU_BAR_ON`: default OFF on macOS (the authoring/capture platform) and ON
//! everywhere else, so a law that never flips it has only ever seen half the sum
//! `text_origin_top` composes.

use super::super::*;
use super::{headless_pipeline, view_md};

/// The four tiers every claim below is graded at. 1.5 is a real macOS scale and is
/// here deliberately: a fix that multiplies by an integer-only factor passes at 2 and
/// 3 but not at a fractional tier.
const TIERS: [f32; 4] = [1.0, 1.5, 2.0, 3.0];

const HEADED: &str = "# Title\n\nsome prose that is long enough to shape a row\n\n\
                      ## Section A\n\nmore body text here\n\n### Deep\n\nlast paragraph\n";

/// CLAIM 1 — `text_origin_top()` IS A LOGICAL QUANTITY, MENU BAR OFF, WITH A PRESENCE
/// FLOOR.
///
/// TWO claims in one, because invariance ALONE is satisfiable by deleting the pad —
/// `0 * dpi` is beautifully dpi-invariant. So this also asserts the constant is
/// PRESENT and exactly `TEXT_TOP.0` at every tier once the DPI multiply is divided
/// back out — the presence half a Logical(0.0) mutation cannot satisfy.
#[test]
fn text_origin_top_is_dpi_invariant_at_matched_logical_geometry_menu_bar_off() {
    let _g = crate::testlock::serial();
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping text_origin_top_is_dpi_invariant menu_bar_off: no wgpu adapter");
        crate::menubar::set_menu_bar_on(ambient_menu_bar);
        return;
    };

    for &(lw, lh) in &[
        (1200.0f32, 800.0f32),
        (900.0f32, 700.0f32),
        (600.0f32, 600.0f32),
    ] {
        let mut logical: Vec<(f32, f32)> = Vec::new();
        for &dpi in &TIERS {
            p.set_size(lw * dpi, lh * dpi);
            p.set_dpi(dpi);
            p.set_view(&view_md(HEADED, 0, 0));
            let top = p.text_origin_top();
            logical.push((top / dpi, dpi));
            // PRESENCE, per tier, against the HARD-CODED historical value rather than
            // a re-read of `TEXT_TOP.0` — comparing against the same constant a
            // mutation also zeroes would make this assertion vacuous under exactly
            // the mutation it exists to catch (proved: mutating `TEXT_TOP` to
            // `Logical(0.0)` left a `TEXT_TOP.0`-relative version of this assertion
            // green). Bar off means the reserve is 0.0, so the whole answer is the
            // scaled TEXT_TOP alone, and 16.0 is the authored, un-scaled pad.
            assert!(
                (top / dpi - 16.0).abs() < 1e-2,
                "logical {lw}x{lh} dpi {dpi}: text_origin_top is {top} device px \
                 ({} logical) but the authored TEXT_TOP is 16.0 — the constant is not \
                 the whole answer with the bar off",
                top / dpi
            );
        }
        let (t0, _) = logical[0];
        for &(t, dpi) in &logical[1..] {
            assert!(
                (t - t0).abs() < 1e-2,
                "logical {lw}x{lh}: text_origin_top is {t0} logical px at dpi 1 but {t} \
                 at dpi {dpi} — the vertical origin is reading the display"
            );
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
}

/// CLAIM 2 — `text_origin_top()` STAYS DPI-INVARIANT WITH THE MENU BAR SHOWN, AND
/// CORRECTLY COMPOSES THE TWO TERMS.
///
/// This does NOT assume `menubar_reserve()` itself scales cleanly with DPI (a
/// SEPARATE, sibling constant family in `src/menubar.rs` — see the landing report) —
/// it only asserts that `text_origin_top` composes `metrics.px(TEXT_TOP)` and
/// `menubar_reserve()` by addition and nothing else, which is the part this item
/// owns. `w == metrics.px(TEXT_TOP) + menubar_reserve()` is checked directly rather
/// than re-deriving the reserve from a hand-rolled formula, so this law cannot drift
/// out of step with `menubar_reserve`'s own (separately evolving) definition.
#[test]
fn text_origin_top_composes_scaled_text_top_and_the_menubar_reserve_menu_bar_on() {
    let _g = crate::testlock::serial();
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping text_origin_top_composes... menu_bar_on: no wgpu adapter");
        crate::menubar::set_menu_bar_on(ambient_menu_bar);
        return;
    };

    let mut checked = 0usize;
    for &dpi in &TIERS {
        p.set_size(1200.0 * dpi, 800.0 * dpi);
        p.set_dpi(dpi);
        p.set_view(&view_md(HEADED, 0, 0));
        let reserve = p.menubar_reserve();
        assert!(
            reserve > 0.0,
            "dpi {dpi}: the bar must actually be reserving space"
        );
        let want = p.metrics.px(TEXT_TOP) + reserve;
        let got = p.text_origin_top();
        assert!(
            (got - want).abs() < 1e-3,
            "dpi {dpi}: text_origin_top={got} but scaled TEXT_TOP + menubar_reserve={want}"
        );
        checked += 1;
    }
    assert_eq!(checked, TIERS.len(), "every tier must be graded");
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
}

/// CLAIM 3 — A CLICK AT A DRAWN ROW RETURNS THAT ROW, AT EVERY DPI, ASSERTED ON BOTH
/// SIDES OF THE ROW BOUNDARY.
///
/// `text_origin_top()` is read by `doc_top` (the draw side, via `char_screen_top_scroll`)
/// and `hit_test_scroll` (the pointer side). This is the agreement that makes moving it
/// safe: a point just inside the row band the first line draws at must resolve to that
/// row, and a point just past its bottom edge must resolve to the next one — a
/// one-sided assertion would pass on a hit test that answered the same row for the
/// whole canvas.
#[test]
fn a_click_at_the_drawn_row_boundary_returns_the_correct_row_at_every_dpi() {
    let _g = crate::testlock::serial();
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping a_click_at_the_drawn_row_boundary: no wgpu adapter");
        crate::menubar::set_menu_bar_on(ambient_menu_bar);
        return;
    };

    let mut checked = 0usize;
    for &dpi in &TIERS {
        p.set_size(1200.0 * dpi, 800.0 * dpi);
        p.set_dpi(dpi);
        p.set_view(&view_md(HEADED, 2, 0));
        let scroll = ScrollPos::default();
        // The boundary between the title row (0) and the blank row under it (1) —
        // real geometry the pipeline actually drew, not a hand-computed offset.
        let row0_top = p.char_screen_top_scroll(0, 0, scroll);
        let row1_top = p.char_screen_top_scroll(1, 0, scroll);
        assert!(
            row1_top > row0_top + 1.0,
            "dpi {dpi}: row 1 must be measurably below row 0 ({row1_top} vs {row0_top})"
        );
        let x = p.text_left() + 4.0;
        // BOTH SIDES of the boundary: just above it must still read row 0, and just
        // below it must already read row 1 — a one-sided assertion would pass on a
        // hit test that answered one row for the whole canvas.
        assert_eq!(
            p.hit_test_scroll(x, row1_top - 1.0, scroll).0,
            0,
            "dpi {dpi}: a click one device px ABOVE row 1's drawn top must still hit \
             row 0 (row0_top={row0_top}, row1_top={row1_top})"
        );
        assert_eq!(
            p.hit_test_scroll(x, row1_top + 1.0, scroll).0,
            1,
            "dpi {dpi}: a click one device px BELOW row 1's drawn top must hit row 1 \
             (row0_top={row0_top}, row1_top={row1_top})"
        );
        checked += 1;
    }
    assert_eq!(checked, TIERS.len(), "every tier must be graded");
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
}

/// CLAIM 4 — BYTE-IDENTITY AT 1x, MENU BAR OFF: THE DEFAULT DOCUMENT ORIGIN IS
/// UNCHANGED BY THIS REFACTOR.
///
/// `Metrics::px(Logical(16.0))` at `scale == 1.0` is `16.0 * 1.0`, which is exactly
/// `16.0` in IEEE-754 (a whole power-of-two-friendly value with no rounding), so the
/// default capture geometry this item touches must be bit-for-bit what it was before
/// `TEXT_TOP` was retyped.
#[test]
fn text_origin_top_is_byte_identical_to_the_historical_16px_at_1x_menu_bar_off() {
    let _g = crate::testlock::serial();
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping text_origin_top_is_byte_identical: no wgpu adapter");
        crate::menubar::set_menu_bar_on(ambient_menu_bar);
        return;
    };
    p.set_size(1200.0, 800.0);
    p.set_dpi(1.0);
    p.set_view(&view_md(HEADED, 0, 0));
    assert_eq!(
        p.text_origin_top().to_bits(),
        16.0f32.to_bits(),
        "the default (1x, bar off) document top must stay bit-identical to the \
         historical 16.0 literal"
    );
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
}
