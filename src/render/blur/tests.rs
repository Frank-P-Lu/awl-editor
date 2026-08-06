//! `blur.rs`'s own unit laws, carved out to a sibling file so the module stays
//! under its production size ratchet (`scripts/code-health.toml` exempts a
//! `tests.rs`). Every test's NAME is unchanged; only which file it lives in moved.

use super::extent::*;
use super::*;

#[test]
fn doc_capture_cap_is_a_noop_at_or_below_the_cap() {
    // A normal surface, a 2× retina surface, and exactly the cap all pass through
    // UNCHANGED — so the capture (and thus the blurred backdrop) is byte-identical.
    assert_eq!(capped_doc_size(1200, 800, DOWNSAMPLE), (1200, 800));
    assert_eq!(
        capped_doc_size(2400, 1600, downsample_for(2.0)),
        (2400, 1600)
    );
    assert_eq!(
        capped_doc_size(DOC_CAPTURE_MAX, 1000, DOWNSAMPLE),
        (DOC_CAPTURE_MAX, 1000)
    );
    assert_eq!(
        capped_doc_size(1000, DOC_CAPTURE_MAX, DOWNSAMPLE),
        (1000, DOC_CAPTURE_MAX)
    );
}

#[test]
fn doc_capture_cap_scales_a_genuinely_large_surface_and_preserves_aspect() {
    // A 5K surface: the longest side is clamped to the cap, the short side scaled
    // by the same factor (aspect preserved), and the result stays at least the
    // quarter-res blur working size so the downsample is still a downsample.
    let (cw, ch) = capped_doc_size(5120, 2880, DOWNSAMPLE);
    assert_eq!(cw, DOC_CAPTURE_MAX);
    let scale = DOC_CAPTURE_MAX as f32 / 5120.0;
    assert_eq!(ch, (2880.0 * scale).round() as u32);
    assert!(cw >= 5120 / DOWNSAMPLE && ch >= 2880 / DOWNSAMPLE);
    // Portrait orientation clamps on height instead.
    let (pw, ph) = capped_doc_size(2880, 5120, DOWNSAMPLE);
    assert_eq!(ph, DOC_CAPTURE_MAX);
    assert_eq!(pw, (2880.0 * scale).round() as u32);
}

/// THE FROST'S REACH IS AUTHORED IN LOGICAL PX AND MULTIPLIED BY DPI ONCE.
///
/// The Gaussian's reach is a fixed count of quarter-res texels, so the reach in
/// PHYSICAL px is `taps × downsample` and the reach a reader perceives is that
/// over `dpi`. A fixed downsample therefore halves the perceived defocus at 2× —
/// the exact class of defect a capture cannot see, because every capture runs at
/// `--capture-dpi 1`. This law sweeps the DPI axis and requires the LOGICAL reach
/// to be constant, which is what a fixed downsample fails.
#[test]
fn the_frosts_logical_reach_is_constant_across_dpi() {
    // 1× is the historical value exactly — so every capture, and every 1× frame,
    // is byte-identical to before the DPI scaling existed.
    assert_eq!(
        downsample_for(1.0),
        DOWNSAMPLE,
        "1x must return the authored constant untouched (capture byte-identity)"
    );
    for dpi in [1.0f32, 1.25, 1.5, 2.0, 3.0] {
        let ds = downsample_for(dpi);
        // The Gaussian's ±4-tap reach, in LOGICAL px.
        let logical_reach = 4.0 * ds as f32 / dpi;
        let authored = 4.0 * DOWNSAMPLE as f32;
        assert!(
            (logical_reach - authored).abs() <= 1.0,
            "dpi {dpi}: the frost reaches {logical_reach:.2} logical px, \
                 authored {authored:.2} — a reach that changes with DPI is a \
                 device-pixel length"
        );
    }
    // Degenerate DPI can never produce a zero factor (a division by it follows).
    for dpi in [0.0f32, -1.0, f32::NAN] {
        assert!(downsample_for(dpi) >= 1, "dpi {dpi} must floor at 1");
    }
}

/// THE FOOTPRINT SCISSOR: outward rounding, a clamp, and an empty answer for a
/// footprint that lands off the target.
///
/// Outward rounding is the load-bearing half. A card box lands on fractional
/// physical px at any non-integer scale (`Metrics::px` multiplies a logical pad by
/// the scale), and rounding IN would leave a sliver of sharp document along the
/// card's own edge — a one-pixel version of the defect the frost exists to remove.
#[test]
fn the_footprint_scissor_rounds_outward_clamps_and_rejects_the_off_canvas() {
    // A fractional box grows to cover every pixel it touches.
    assert_eq!(
        scissor_px([10.4, 20.6, 100.3, 50.1], 1200, 800),
        Some((10, 20, 101, 51)),
        "near edges floor, far edges ceil — the frost covers the whole box"
    );
    // An integral box is exact — no free growth.
    assert_eq!(
        scissor_px([10.0, 20.0, 100.0, 50.0], 1200, 800),
        Some((10, 20, 100, 50))
    );
    // A 2x card box (the same logical box at scale 2) stays a doubled rect: the
    // rect arrives PHYSICAL, so this fn applies no scale of its own.
    assert_eq!(
        scissor_px([20.0, 40.0, 200.0, 100.0], 2400, 1600),
        Some((20, 40, 200, 100))
    );
    // Clamped to the target on both ends, never past it (wgpu validates this).
    assert_eq!(
        scissor_px([-30.0, -10.0, 200.0, 100.0], 1200, 800),
        Some((0, 0, 170, 90))
    );
    let (x, y, w, h) = scissor_px([1100.0, 700.0, 400.0, 400.0], 1200, 800).unwrap();
    assert!(
        x + w <= 1200 && y + h <= 800,
        "the scissor stays inside the target: {x},{y} {w}x{h}"
    );
    // Entirely off the target, degenerate, or non-finite: no scissor, and the
    // caller must draw NOTHING rather than fall back to the fullscreen triangle.
    assert_eq!(scissor_px([1300.0, 20.0, 100.0, 50.0], 1200, 800), None);
    assert_eq!(scissor_px([10.0, 20.0, 0.0, 50.0], 1200, 800), None);
    assert_eq!(scissor_px([f32::NAN, 20.0, 100.0, 50.0], 1200, 800), None);
    assert_eq!(scissor_px([10.0, 20.0, 100.0, 50.0], 0, 0), None);
}

/// The FOOTPRINT arm dims by nothing and the FULL arm keeps its authored recede —
/// the two extents carry their own dim, so a hue claim inside a footprint is a
/// claim about the blur alone.
#[test]
fn the_footprint_arm_carries_no_dim_and_the_full_arm_keeps_its_own() {
    assert_eq!(Frost::Footprint([0.0, 0.0, 10.0, 10.0]).dim(), 0.0);
    assert_eq!(Frost::Full.dim(), DIM);
    assert!(DIM > 0.0, "the full takeover still recedes a value");
}

/// ENROLMENT, DERIVED FROM THE ROSTER. Every world's own list composition decides
/// whether a crisp picker over it frosts its footprint — nothing here names a
/// world, and the answer follows a world that changes its list style.
///
/// Non-vacuity is asserted in both directions: the enrolled set and the excluded
/// set are both non-empty, and both are NAMED in the failure message. A predicate
/// that quietly stopped matching anything (the shape that swept nothing for the
/// life of a law once already) fails here rather than passing green.
#[test]
fn footprint_enrolment_follows_the_rosters_own_backing_owners() {
    use crate::theme::{ListBacking, ListStyle};
    let mut enrolled: Vec<&str> = Vec::new();
    let mut excluded: Vec<&str> = Vec::new();
    for t in crate::theme::THEMES.iter() {
        let style = t.render_caps.list_style;
        if footprint_frost_applies(style) {
            enrolled.push(t.name);
            assert!(
                !matches!(style.list_backing(false), ListBacking::Card),
                "{}: enrolled but its card is a filled panel",
                t.name
            );
            assert!(
                !style.draws_row_plates(),
                "{}: enrolled but it plates its own rows",
                t.name
            );
        } else {
            excluded.push(t.name);
            assert!(
                matches!(style.list_backing(false), ListBacking::Card) || style.draws_row_plates(),
                "{}: excluded while drawing neither a panel nor plates — \
                     the document shows straight through its rows",
                t.name
            );
        }
    }
    assert!(
        !enrolled.is_empty(),
        "no world enrols — the mechanism has no subject (enrolled={enrolled:?})"
    );
    assert!(
        !excluded.is_empty(),
        "every world enrols — the byte-identical arm has no subject \
         (excluded={excluded:?})"
    );
    // The style axis itself, exhaustively: one member per shape of backing, so a
    // new `ListStyle` cannot slip past with an unconsidered answer.
    assert!(footprint_frost_applies(ListStyle::Rules(
        crate::theme::RuleSelection::Weight
    )));
    assert!(footprint_frost_applies(ListStyle::Diagonal(
        crate::theme::DiagonalSpine::descending(crate::theme::DiagonalMark::CRISP)
    )));
    assert!(
        !footprint_frost_applies(ListStyle::Bars),
        "Bars plates its rows: the plate is the frost's job already"
    );
    assert!(
        !footprint_frost_applies(ListStyle::Pane),
        "Pane's panel covers its whole footprint"
    );
}
