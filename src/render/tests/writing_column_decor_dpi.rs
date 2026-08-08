//! THE WRITING COLUMN'S DECORATIONS, AT EVERY PANEL DENSITY.
//!
//! The vertical/horizontal origin laws (`text_top_dpi_item315.rs`,
//! `column_left_dpi_item314.rs`) grade where the text column STARTS. This grades
//! the decorations drawn INSIDE it — the inline-code pill's inset, the fence
//! panel's overhang, and the spell squiggle's amplitude, period and stroke.
//! Every one of them was multiplied by `metrics.zoom` alone for its whole life,
//! so it held its DEVICE size as the display got denser and rendered at half its
//! tuned size beside doubled text on every retina panel. Measured before the
//! repair, at matched logical geometry: the pill was 92.70px wide at dpi 1 and
//! 179.39px at dpi 2, where twice the first is 185.39 — short by exactly twice
//! the pill's own inset — and the fence panel short by exactly twice its own.
//!
//! **WHY A DECLARATION SWEEP COULD NOT CLOSE THIS AND THIS CAN.** A type sweep
//! grades the constant; only a geometry law grades the FACTOR the read site
//! hands it. Both halves are asserted per tier, because invariance alone is
//! satisfiable by deleting the decoration — `0 * dpi` is beautifully
//! dpi-invariant — so each claim also pins the authored quantity, with the
//! display factor divided back out, against a HARD-CODED number rather than a
//! re-read of the constant. Re-reading it makes the pin vacuous under exactly
//! the mutation it exists to catch: a `Logical(0.0)` satisfies a
//! `CONST.0`-relative assertion perfectly.
//!
//! The experiment is the origin laws': `--capture-dpi N` makes a `WxH` DEVICE
//! canvas a `(W/N)x(H/N)` LOGICAL window, so the physical canvas grows in
//! lockstep with `dpi` to hold the logical window fixed. Comparing two tiers at
//! one device size compares two different windows.
//!
//! **AND EVERY TIER IS A FRESH PIPELINE, DPI SET BEFORE THE DOCUMENT, DRIVEN
//! THROUGH A REAL `prepare()`.** That is the order the capture path and app
//! startup both use, and the order matters: raising the DPI of a pipeline that
//! ALREADY holds a shaped document leaves `visual_rows`'s own `line_height` at the
//! value it shaped at (32.0 at dpi 1, 1.5, 2 and 3 alike), so a decoration whose
//! band is derived from the row height measures against a stale row while its
//! x-extents track the new metrics. A sweep that re-used one pipeline across tiers
//! therefore reported a pill band that SHRANK as the display got denser — a
//! property of the fixture's ordering, not of the constants this file grades.

use super::super::*;
use super::{headless_dqp, view};

/// The tiers every claim is graded at. 1.5 is a real macOS scale and is here
/// deliberately: a repair that multiplies by an integer-only factor passes at 2
/// and 3 and fails at a fractional tier.
const TIERS: [f32; 4] = [1.0, 1.5, 2.0, 3.0];

/// The AUTHORED values, hard-coded. See the module doc: a pin that re-reads the
/// constant it is pinning cannot fail on a mutation of that constant.
const AUTHORED_PILL_INSET_X: f32 = 3.0;
const AUTHORED_PILL_INSET_Y: f32 = 1.0;
const AUTHORED_FENCE_INSET_X: f32 = 8.0;
const AUTHORED_SPELL_AMP: f32 = 3.2;
const AUTHORED_SPELL_PERIOD: f32 = 12.0;
const AUTHORED_SPELL_THICKNESS: f32 = 3.6;

/// A FRESH pipeline at `dpi`, sized so the LOGICAL window is 1200x800 at every
/// tier. The DPI is set before any document is shaped — the capture path's own
/// order, and the only order in which a decoration's row band is the row band the
/// user gets (see the module doc).
fn tier_pipeline(dpi: f32) -> Option<(wgpu::Device, wgpu::Queue, TextPipeline)> {
    let (device, queue, mut p) = headless_dqp(1200.0 * dpi, 800.0 * dpi)?;
    p.set_dpi(dpi);
    p.set_size(1200.0 * dpi, 800.0 * dpi);
    Some((device, queue, p))
}

/// PAGE MODE OFF is the zero-slack configuration: `text_pad()` is hard-zeroed,
/// so `text_left() == column_left()` and the decorative overhangs have no page
/// margin to hide in. It is also the configuration the pill/panel overhang laws
/// in `washes.rs` establish as the worst case for the content clip, so a
/// measurement here is of the same geometry those laws already pin at dpi 1.
fn zero_slack(p: &mut TextPipeline) {
    assert_eq!(
        p.text_pad(),
        0.0,
        "precondition: page mode off => text_pad is hard-zeroed"
    );
    assert!(
        (p.text_left() - p.column_left()).abs() < 1e-4,
        "precondition: text_left == column_left with zero slack: {} vs {}",
        p.text_left(),
        p.column_left()
    );
}

/// CLAIM 1 — THE INLINE-CODE PILL'S INSET IS A LOGICAL LENGTH, BOTH AXES.
///
/// The pill's own overhang past the glyph column, X and Y, divided by the
/// display factor, is the authored inset at every tier. The X half is the
/// quantity whose absence measured exactly `2 * CODE_PILL_INSET_X` of missing
/// pill width at dpi 2.
#[test]
fn the_code_pills_inset_holds_its_logical_size_at_every_panel_density() {
    let _g = crate::testlock::serial();
    let prev_page = crate::page::page_on();
    crate::page::set_page_on(false);
    crate::markdown::set_wysiwyg_on(true);
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the code-pill density law: no wgpu adapter");
        crate::page::set_page_on(prev_page);
        return;
    }
    // The span starts at column 0, so its un-inset left edge lands exactly on
    // `text_left()` and the whole overhang is the inset and nothing else.
    let text = "`code` at line start\n";
    let mut graded = 0usize;
    let mut widths: Vec<(f32, f32)> = Vec::new();
    for &dpi in &TIERS {
        let Some((device, queue, mut p)) = tier_pipeline(dpi) else {
            continue;
        };
        let mut v = view(text, 1, 0);
        v.is_markdown = true;
        p.set_view(&v);
        p.prepare(&device, &queue, (1200.0 * dpi) as u32, (800.0 * dpi) as u32)
            .expect("a headless frame prepares");
        p.atlas.trim();
        zero_slack(&mut p);
        let rects = p.code_pill_rects();
        assert_eq!(rects.len(), 1, "dpi {dpi}: one span => one pill: {rects:?}");
        let [rx, ry, rw, rh] = rects[0];
        let inset_x = p.text_left() - rx;
        assert!(
            (inset_x / dpi - AUTHORED_PILL_INSET_X).abs() < 1e-2,
            "dpi {dpi}: the pill's left overhang is {inset_x} device px ({} logical), \
             and the authored inset is {AUTHORED_PILL_INSET_X} — a pill that holds its \
             device inset renders at 1/dpi of its tuned size beside doubled text",
            inset_x / dpi
        );
        // The Y inset is not reachable as an edge difference (the pill's band top
        // is derived from the row), so it is read off the pill's HEIGHT against
        // the row's own cell height, which the metrics owner already scales.
        let inset_y = (rh - p.metrics.caret_h) * 0.5;
        assert!(
            (inset_y / dpi - AUTHORED_PILL_INSET_Y).abs() < 1e-2,
            "dpi {dpi}: the pill's vertical overhang is {inset_y} device px ({} logical) \
             against an authored {AUTHORED_PILL_INSET_Y}",
            inset_y / dpi
        );
        assert!(
            rw > 20.0 * dpi && rh > 4.0 * dpi && ry > 0.0,
            "dpi {dpi}: the pill must actually be drawn — an absent quad satisfies \
             every ratio above (w={rw} h={rh} y={ry})"
        );
        widths.push((rw / dpi, dpi));
        graded += 1;
    }
    // THE WHOLE QUAD, not only its inset: the pill's logical width is one number
    // across the roster of tiers. This is the claim the original measurement made
    // (92.70 at dpi 1 against 179.39 at dpi 2, where 185.39 was owed).
    let (w0, _) = widths[0];
    for &(w, dpi) in &widths[1..] {
        assert!(
            (w - w0).abs() < 2e-2,
            "the pill is {w0} logical px wide at dpi 1 and {w} at dpi {dpi} — the quad \
             is reading the display"
        );
    }
    assert_eq!(graded, TIERS.len(), "every tier must be graded");
    crate::page::set_page_on(prev_page);
}

/// CLAIM 2 — THE FENCE PANEL'S OVERHANG IS A LOGICAL LENGTH, BOTH EDGES.
///
/// The panel hangs its own inset past the writing column on each side, so a
/// device-pixel inset costs twice itself in width — the second exact figure the
/// original measurement produced.
#[test]
fn the_fence_panels_overhang_holds_its_logical_size_at_every_panel_density() {
    let _g = crate::testlock::serial();
    let prev_page = crate::page::page_on();
    crate::page::set_page_on(false);
    crate::markdown::set_wysiwyg_on(true);
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the fence-panel density law: no wgpu adapter");
        crate::page::set_page_on(prev_page);
        return;
    }
    let text = "```rust\nlet x = 1;\n```\n";
    let mut graded = 0usize;
    for &dpi in &TIERS {
        let Some((device, queue, mut p)) = tier_pipeline(dpi) else {
            continue;
        };
        let mut v = view(text, 0, 0);
        v.is_markdown = true;
        p.set_view(&v);
        p.prepare(&device, &queue, (1200.0 * dpi) as u32, (800.0 * dpi) as u32)
            .expect("a headless frame prepares");
        p.atlas.trim();
        zero_slack(&mut p);
        let rects = p.fence_panel_rects();
        assert_eq!(
            rects.len(),
            1,
            "dpi {dpi}: one fenced block => one merged panel: {rects:?}"
        );
        let [rx, _ry, rw, rh] = rects[0];
        let left = p.text_left() - rx;
        let right = (rx + rw) - (p.text_left() + p.text_wrap_width());
        for (edge, got) in [("left", left), ("right", right)] {
            assert!(
                (got / dpi - AUTHORED_FENCE_INSET_X).abs() < 1e-2,
                "dpi {dpi}: the panel's {edge} overhang is {got} device px ({} logical) \
                 against an authored {AUTHORED_FENCE_INSET_X}",
                got / dpi
            );
        }
        assert!(
            rh > 4.0 * dpi,
            "dpi {dpi}: the panel must actually be drawn (h={rh})"
        );
        graded += 1;
    }
    assert_eq!(graded, TIERS.len(), "every tier must be graded");
    crate::page::set_page_on(prev_page);
}

/// CLAIM 3 — THE SPELL SQUIGGLE'S THREE SHAPE TERMS ARE LOGICAL LENGTHS.
///
/// Amplitude, period and stroke thickness are separately tuned taste
/// quantities, and all three rode `zoom` alone: on a retina panel the wave kept
/// its device amplitude and its device wavelength while the word beneath it
/// doubled, so the squiggle read as a tighter, thinner ripple under bigger text.
/// Graded as the authored logical value per tier, and each with its own presence
/// floor — a wave of zero amplitude is perfectly dpi-invariant.
#[test]
fn the_spell_squiggles_shape_holds_its_logical_size_at_every_panel_density() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the spell-squiggle density law: no wgpu adapter");
        return;
    }
    let text = "helo wrld\n";
    let mis = vec![crate::spell::Misspelling {
        line: 0,
        start_col: 0,
        end_col: 4,
    }];
    let mut graded = 0usize;
    for &dpi in &TIERS {
        let Some((device, queue, mut p)) = tier_pipeline(dpi) else {
            continue;
        };
        // The caret parked off the word: the reveal-on-cursor rule yields the
        // squiggle under active editing, and a yielded word draws nothing.
        let mut v = view(text, 0, 100);
        v.misspelled = mis.clone();
        p.set_view(&v);
        p.prepare(&device, &queue, (1200.0 * dpi) as u32, (800.0 * dpi) as u32)
            .expect("a headless frame prepares");
        p.atlas.trim();
        let squiggles = p.spell_squiggles();
        assert_eq!(
            squiggles.len(),
            1,
            "dpi {dpi}: one misspelling away from the caret => one squiggle"
        );
        let s = &squiggles[0];
        for (name, got, authored) in [
            ("amplitude", s.amp, AUTHORED_SPELL_AMP),
            ("period", s.period, AUTHORED_SPELL_PERIOD),
            ("thickness", s.thickness, AUTHORED_SPELL_THICKNESS),
        ] {
            assert!(
                (got / dpi - authored).abs() < 1e-2,
                "dpi {dpi}: the squiggle's {name} is {got} device px ({} logical) against \
                 an authored {authored} — the wave is reading the display, not the text \
                 it underlines",
                got / dpi
            );
            assert!(
                got > 0.5,
                "dpi {dpi}: the squiggle's {name} must be present ({got}) — a zero term \
                 satisfies every invariance claim above"
            );
        }
        assert!(
            s.w > 10.0 * dpi,
            "dpi {dpi}: the squiggle must span the word it underlines (w={})",
            s.w
        );
        graded += 1;
    }
    assert_eq!(graded, TIERS.len(), "every tier must be graded");
}
