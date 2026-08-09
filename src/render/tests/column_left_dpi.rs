//! ITEM 314 — THE WRITING COLUMN'S LEFT EDGE, THROUGH THE LIVE PIPELINE, AT EVERY DPI.
//!
//! `render::geometry::tests::adaptive_column_left_is_dpi_invariant_at_matched_logical_geometry`
//! is the PURE half — the policy arithmetic, swept exhaustively without a device. These
//! are the three claims that only the assembled pipeline can answer, and the reason the
//! defect needed a deep-tier item rather than a units patch: `geometry.rs` is read by the
//! caret, the selection, the hit test and the drag handle, so a column that moves must
//! move all of them TOGETHER.
//!
//! The experiment is item 307's, and getting it wrong proves nothing: `--capture-dpi N`
//! makes a `WxH` DEVICE canvas a `(W/N)x(H/N)` LOGICAL window, so every tier here grows
//! its physical canvas in lockstep with `dpi` to hold the LOGICAL window fixed.
//!
//! Why 307's own gutter law did not already catch this: its fixture is `"hello world\n"`,
//! a buffer with NO headings, so `outline_wants_rail()` is false and the adaptive column
//! is a pure passthrough. The unscaled pad only enters through the RAIL branch. Every
//! fixture here is HEADED for exactly that reason.

use super::super::*;
use super::{headless_pipeline, view_md};

/// The three tiers every claim below is graded at. 1.5 is a real macOS scale and is
/// here deliberately: a fix that multiplies by an integer-only factor passes at 2 and 3.
const TIERS: [f32; 4] = [1.0, 1.5, 2.0, 3.0];

const HEADED: &str = "# Title\n\nsome prose that is long enough to shape a row\n\n\
                      ## Section A\n\nmore body text here\n\n### Deep\n\nlast paragraph\n";

/// CLAIM 1 — THE COLUMN'S LEFT EDGE AND THE TEXT ORIGIN ARE LOGICAL QUANTITIES.
///
/// Measured before the fix, at logical 1200x800 with this fixture: the rail plateau sat
/// at 244.96 / 236.96 / 234.29 logical px at dpi 1 / 2 / 3 (`228.96 + 16/dpi`) and a
/// collapsed page pinned at 16 / 8 / 5.33. The residual allowed here is one whole
/// PHYSICAL pixel — the subpixel-shimmer floor `adaptive_column_left` ends with, which is
/// under `1/dpi` logical px and is the ONLY legitimate cross-tier difference.
#[test]
fn column_left_and_text_origin_are_dpi_invariant_at_matched_logical_geometry() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping column_left_and_text_origin_are_dpi_invariant: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(true);
    crate::outline::set_outline_on(true);

    let mut saw_rail_shift = false;
    let mut saw_collapsed = false;

    for &(lw, lh) in &[
        (1200.0f32, 800.0f32),
        (900.0f32, 700.0f32),
        (600.0f32, 600.0f32),
    ] {
        for measure in 20..=110usize {
            let mut seen: Vec<(f32, f32, f32)> = Vec::new();
            for &dpi in &TIERS {
                crate::page::set_measure(measure);
                p.set_size(lw * dpi, lh * dpi);
                p.set_dpi(dpi);
                p.set_view(&view_md(HEADED, 0, 0));
                let col_left = p.column_left();
                let text_left = p.text_left();
                // The authored pad, resolved at this tier — the collapse floor's value.
                let pad = p.edge_pad();
                if (col_left - pad).abs() < 0.51 {
                    saw_collapsed = true;
                }
                let symmetric =
                    column_left_for(lw * dpi, CHAR_WIDTH * dpi, true, measure, dpi).floor();
                if col_left > symmetric + 0.5 {
                    saw_rail_shift = true;
                }
                seen.push((col_left / dpi, text_left / dpi, dpi));
            }
            let (c0, t0, _) = seen[0];
            for &(c, t, dpi) in &seen[1..] {
                assert!(
                    (c - c0).abs() <= 1.0 + 1e-3,
                    "logical {lw}x{lh} measure={measure}: column_left is {c0} logical px at \
                     dpi 1 but {c} at dpi {dpi} — more than the one whole physical pixel the \
                     shimmer snap may cost, so the placement is reading the display"
                );
                assert!(
                    (t - t0).abs() <= 1.0 + 1e-3,
                    "logical {lw}x{lh} measure={measure}: text_left is {t0} logical px at dpi \
                     1 but {t} at dpi {dpi} — the GLYPH ORIGIN moved with the display"
                );
            }
        }
    }
    assert!(
        saw_rail_shift,
        "the sweep must actually enter the RAIL-SHIFT branch (the only branch the \
         unscaled pad reached) — otherwise this law is grading a passthrough"
    );
    assert!(
        saw_collapsed,
        "the sweep must also reach the COLLAPSE floor, where the pad is the whole answer"
    );
    crate::outline::set_outline_on(false);
}

/// CLAIM 2 — A CLICK LANDS ON THE GLYPH IT IS DRAWN UNDER, AT EVERY DPI.
///
/// `column_left()` is read by the caret, the selection, the hit test and the drag
/// handle. This is the agreement that makes moving it safe: the DRAWN x of a glyph cell
/// (`text_left() + col_x_and_advance`, the same pair the caret quad and the selection
/// band are built from) is fed back through `hit_test_scroll` — the pointer path the live
/// app uses — and must return the very column it was drawn for. Graded at all four tiers
/// on the SAME logical window.
///
/// ⚠️ WHAT THIS LAW CAN AND CANNOT FAIL ON, measured rather than assumed. It survives
/// EVERY pad-unit mutation this round was proved against, and that is not a weakness in
/// the assertion — it is the single-owner design holding: both the drawn x and the hit
/// test compose `text_left()`, so a mispriced pad moves them TOGETHER and they still
/// agree. What it does fail on is the bug it is actually named for — a reader deriving
/// its own text origin instead of composing the owner. Proved red by replacing
/// `hit_test_scroll`'s `self.text_left()` with an independently rebuilt
/// `column_left() + <pad>`: a parallel geometry, which is the only way drawn and hit can
/// ever disagree here. Read it as the guard on that seam, and read the invariance claims
/// above as the guard on the pads.
#[test]
fn a_click_at_a_drawn_glyph_returns_that_glyph_at_every_dpi() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping a_click_at_a_drawn_glyph_returns_that_glyph: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(true);
    crate::outline::set_outline_on(true);

    let mut checked = 0usize;
    // Two measures: one wide (symmetric passthrough) and one inside the rail-shift band
    // measured above, so the agreement is graded on BOTH sides of the branch that moves.
    for &measure in &[40usize, 60, 75] {
        for &dpi in &TIERS {
            crate::page::set_measure(measure);
            p.set_size(1200.0 * dpi, 800.0 * dpi);
            p.set_dpi(dpi);
            p.set_view(&view_md(HEADED, 2, 0));
            let line = 2usize; // the prose row, wide enough to hold many cells
            let row_y = p.char_screen_top_scroll(line, 0, ScrollPos::default())
                + p.metrics.line_height * 0.5;
            for col in [0usize, 1, 5, 11, 20] {
                let (x, adv) = p.col_x_and_advance(line, col);
                if adv <= 0.0 {
                    continue;
                }
                // Caret placement snaps to the NEAREST cell boundary, so the cell's own
                // two halves are what pin drawn to hit: the NEAR half must resolve to
                // this column and the FAR half to the next. Asserting only the near half
                // would pass on a hit test that answered `col` for the whole row.
                let near = p.text_left() + x + adv * 0.25;
                let far = p.text_left() + x + adv * 0.75;
                assert_eq!(
                    p.hit_test_scroll(near, row_y, ScrollPos::default()),
                    (line, col),
                    "dpi {dpi} measure={measure}: the glyph drawn for col {col} spans \
                     x=[{}, {}] from text_left={}, but a click in its NEAR half ({near}) \
                     hit-tested elsewhere — drawn and hit disagree",
                    p.text_left() + x,
                    p.text_left() + x + adv,
                    p.text_left()
                );
                assert_eq!(
                    p.hit_test_scroll(far, row_y, ScrollPos::default()),
                    (line, col + 1),
                    "dpi {dpi} measure={measure}: a click in the FAR half ({far}) of col \
                     {col}'s drawn cell must resolve to the NEXT boundary — otherwise the \
                     near-half assertion above proves nothing"
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked >= 3 * TIERS.len() * 4,
        "the agreement must be graded on real cells at every tier, checked={checked}"
    );
    crate::outline::set_outline_on(false);
}

/// CLAIM 3 — THE MARGIN OUTLINE'S CLICK BAND STARTS AT THE SAME LOGICAL X AT EVERY DPI,
/// AND IS ASSERTED ON BOTH SIDES.
///
/// `outline_hit_line`'s x band is `[edge_pad, right_edge]`, and the rail BLOCK is drawn
/// from the same pad. Asserting only "a point inside hits" would pass on a band that
/// swallowed the whole window, so the point just OUTSIDE must miss — and the pair must
/// sit at one logical x across the tiers. This is the affordance half of the blast
/// radius: the rail is clickable exactly where a reader can see it.
#[test]
fn the_outline_click_band_starts_at_one_logical_x_at_every_dpi() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping the_outline_click_band_starts_at_one_logical_x: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(true);
    crate::outline::set_outline_on(true);
    crate::page::set_measure(40);

    let mut edges: Vec<(f32, f32)> = Vec::new();
    for &dpi in &TIERS {
        let (lw, lh) = (1200.0f32, 800.0f32);
        p.set_size(lw * dpi, lh * dpi);
        p.set_dpi(dpi);
        p.set_view(&view_md(HEADED, 0, 0));
        let height = (lh * dpi) as u32;
        assert!(
            p.outline_visible(height),
            "dpi {dpi}: the fixture must actually DRAW the rail, or this law grades nothing"
        );
        let pad = p.edge_pad();
        // Scan down the drawn band for a y that lands on a row, then walk x inward from
        // the pad. `+ 0.5` is inside the first physical pixel of the band at every tier.
        let mut inside = None;
        let mut y = 0.0f32;
        while y < lh * dpi {
            if let Some(hit) = p.outline_hit_line(pad + 0.5, y, height) {
                inside = Some((y, hit));
                break;
            }
            y += 1.0;
        }
        let (y, _) = inside.unwrap_or_else(|| {
            panic!("dpi {dpi}: no y in the drawn band hit-tests at the pad's own left edge")
        });
        // BOTH SIDES: one physical pixel further out must miss.
        assert!(
            p.outline_hit_line(pad - 1.0, y, height).is_none(),
            "dpi {dpi}: x={} is OUTSIDE the rail's left inset ({pad}) and must not hit — a \
             one-sided band assertion passes on a gate that never turns off",
            pad - 1.0
        );
        edges.push((pad / dpi, y / dpi));
    }
    let (e0, _) = edges[0];
    for (i, &(e, _)) in edges.iter().enumerate().skip(1) {
        assert!(
            (e - e0).abs() < 1e-2,
            "the outline's click band begins at {e0} logical px at dpi 1 but {e} at dpi {} \
             — the rail's inset is reading the display, so the band and the drawn block \
             drift off the reader's eye together",
            TIERS[i]
        );
    }
    crate::outline::set_outline_on(false);
}

/// CLAIM 4 — THE GUTTER'S VISIBILITY BOUNDARY IS DPI-INVARIANT *WITH A RAIL PRESENT*.
///
/// This is the reported defect in its original form. Item 307 proved the boundary
/// invariant for a HEADINGLESS buffer; with headings the rail branch runs, the unscaled
/// pad enters `column_left()`, and the boundary moved a whole `--measure` step — measured
/// at logical 1200x800: visible through 75 and hidden from 76 at dpi 1, but visible
/// through 76 and hidden from 77 at dpi 2 and 3. Asserted on BOTH sides: the sweep must
/// contain a visible cell and a hidden one, or an always-off gate would pass.
#[test]
fn gutter_visibility_boundary_with_a_rail_is_dpi_invariant() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping gutter_visibility_boundary_with_a_rail_is_dpi_invariant: no adapter");
        return;
    };
    crate::page::set_page_on(true);
    crate::outline::set_outline_on(true);

    for &(lw, lh) in &[(1200.0f32, 800.0f32), (1000.0f32, 750.0f32)] {
        let mut saw_visible = false;
        let mut saw_hidden = false;
        let mut boundaries: Vec<(f32, Option<usize>)> = Vec::new();
        for &dpi in &TIERS {
            let mut last_visible: Option<usize> = None;
            for measure in 20..=110usize {
                crate::page::set_measure(measure);
                p.set_size(lw * dpi, lh * dpi);
                p.set_dpi(dpi);
                let mut v = view_md(HEADED, 0, 0);
                v.gutter_name = "notes.md".to_string();
                v.gutter_project = "awl".to_string();
                p.set_view(&v);
                let visible = p.gutter_report().is_some();
                if dpi == 1.0 {
                    saw_visible |= visible;
                    saw_hidden |= !visible;
                }
                if visible {
                    last_visible = Some(measure);
                }
            }
            boundaries.push((dpi, last_visible));
        }
        assert!(
            saw_visible && saw_hidden,
            "logical {lw}x{lh}: the measure sweep must CROSS the gutter's gate \
             (visible={saw_visible} hidden={saw_hidden})"
        );
        let (_, b0) = boundaries[0];
        for &(dpi, b) in &boundaries[1..] {
            assert_eq!(
                b, b0,
                "logical {lw}x{lh}: the widest measure still showing the gutter is {b0:?} at \
                 dpi 1 but {b:?} at dpi {dpi} — the SAME logical page, so the boundary moved \
                 with the display (the reported defect: 75/76 at 1x, 76/77 at 2x and 3x)"
            );
        }
    }
    crate::outline::set_outline_on(false);
}
