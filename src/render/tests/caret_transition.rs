//! THE CARET CELL TRANSITION LAW. The caret has a two-arm
//! `caret_cell_vertical` (a real INK BOX on a single proportional glyph, a
//! LINE CELL everywhere else) and proved each arm correct IN ISOLATION. Nothing
//! it wrote ever constructed the SEAM between them: two adjacent caret columns
//! on the SAME row, one glyph-anchored and one not. That seam is exactly what
//! the user's paired release screenshots caught — on `aaa`, the Block caret on
//! the final `a` versus one step later at end-of-line has a visibly
//! DISCONTINUOUS outer cell, on Mopoke and Gumtree alike, and (this file's own
//! sweep below proves) on every proportional world in the roster.
//!
//! THE LAW. For adjacent caret columns on one row, the outer cell's
//! `(center_y, height)` — read straight from [`TextPipeline::caret_cell_vertical`],
//! the ONE owner every cell-form caret draws from — may not jump by more than a
//! small, explicitly authored, pixel-scaled bound. This is a REAL-PIXEL claim:
//! `caret_cell_vertical`'s numbers are not an approximation of what gets drawn,
//! they are literally the values [`TextPipeline::caret_geometry`] feeds the GPU
//! quad, so a law here about `(cy, h)` is a law about the rendered rectangle's
//! top/bottom pixels (mirroring how `caret_ink_box.rs`'s laws read
//! pixel-exact geometry rather than decoding a PNG).
//!
//! NON-VACUITY. Every sweep below also computes the OLD fallback
//! formula (`cy = caret.pos.y`, `h = caret_block_h * cursor_scale()` — byte-copy
//! of the former `caret.rs`/`facepitch.rs` line-cell
//! arm) and asserts that number alone WOULD have blown the bound — so the
//! fixture is proven capable of catching the exact regression, not merely
//! passing by construction. Confirmed directly: this whole
//! file, run with `src/render/caret.rs` + `src/render/facepitch.rs` reverted to
//! their former `main` state (the same two-arm formula; the pitch classifier
//! only changed WHICH worlds take which arm, never the
//! formula), fails every test exercising the `aaa`/x-height class this
//! docstring's own numbers are drawn from, and passes them all once the repair
//! is restored — stated as the CLASS that fails, because the fractions that
//! stood here counted a file since grown. See the commit history for the
//! console output this proof produced.
//!
//! SWEPT AXES: the full proportional-world
//! roster (not just the two the user found), representative glyph classes
//! (x-height / ascender / descender / punctuation / digit / capital / space /
//! EOL / empty line / ligature), Block AND Morph (rest — travel is proven
//! UNAFFECTED, not simply re-measured, since a moving caret is a streak with
//! no cell to jump), a wrapped-line boundary, two zooms including a non-1.0
//! value, and 1x/2x DPI — plus the mono complement (must stay at ZERO
//! discontinuity: the mono grid never leaves the line-cell arm, so there is
//! no seam to jump across there).
//!
//! THE NEIGHBOR-BORROW REPAIR.
//! The first fix's fallback synthesized a "typical letter" box from a per-font
//! MEAN of x-height/cap-height ratios — a single fixed reference that closed
//! the reported x-height class but, unswept, silently REGRESSED every other
//! class it never tested: a CAPITAL letter (absent from the class roster
//! entirely) landed WORSE than the former code on 11/11 proportional worlds
//! (new Δ 2.3–4.4px vs old Δ 0.4–2.9px), and the WIDE-bound "no worse than
//! before" law meant to catch exactly that kind of regression could not,
//! because its ceiling FLOORED at the wide bound regardless of how small
//! `old_d` was — permitting up to a 7.5px regression on a class that started
//! at 0.6px and calling it "no worse than before".
//!
//! THE FIX: [`TextPipeline::caret_cell_vertical`]'s fallback arm now BORROWS
//! the immediately PRECEDING column's own real ink (when one exists) instead
//! of synthesizing an approximation — a glyphless cell beside a real letter
//! now draws with THAT letter's exact ink, closing the literal adjacent seam
//! to (near-)zero for EVERY class, not merely x-height, because it no longer
//! guesses. The synthetic per-font ratio survives only for the case NO real
//! neighbor exists (an empty line, or a glyphless anchor at column 0) — see
//! `caret::TextPipeline::caret_synthetic_ink_box`'s doc. The "no worse than
//! before" (`old_d`-relative) framing was RETIRED entirely, not just
//! tightened: once neighbor-borrow makes an adjacent transition read genuine
//! glyph ink instead of a guess, comparing that real ink to the OLD crude
//! approximation is no longer a meaningful regression signal (real ink can
//! legitimately differ from a guess by any amount without either being
//! wrong) — see [`TRANSITION_BOUND_WIDE_PX`]'s doc for the concrete case
//! (the ligature fixture) where the tightened `old_d`-relative version
//! produced a false positive. The remaining real-ink-to-real-ink fixtures
//! (ligature, wrap-boundary) use a plain ABSOLUTE sanity bound instead; the
//! class that genuinely needs a regression guard (a SYNTHETIC approximation
//! with nothing real to fall back on) is now tested at literal adjacency,
//! where neighbor-borrow closes it exactly — see
//! `every_glyph_class_closes_exactly_at_the_literal_eol_seam`.

use super::super::*;
use super::{headless_pipeline, view};

/// The pixel scale (zoom × dpi) the pads and the transition bound both ride —
/// the same quantity `caret_ink_box.rs::pad_px` reads, redefined here so this
/// file has no cross-module dependency on that one's private helper. Reads the
/// stored [`render::Metrics::scale`] field directly rather than recovering it
/// by division.
fn pixel_scale(p: &TextPipeline) -> f32 {
    p.metrics.scale
}

/// THE TIGHT AUTHORED BOUND (px at zoom×dpi 1.0): how far the outer cell's
/// centre or height may move between an X-HEIGHT-class on-glyph column and an
/// adjacent glyphless one on the same row — the literal `aaa`->EOL shape of
/// the user's report, and the single most common transition in ordinary prose
/// (x-height letters, without an ascender/descender, are the majority of
/// English lowercase text). Measured residual after the fix: 0.98–2.60px
/// across the full proportional roster; the bound sits with real margin above
/// that and well below the pre-fix bug's magnitude on this SAME class
/// (2.4–6.4px, 6.4/22.4 ≈ 29% of the old fixed cell on Bombora).
const TRANSITION_BOUND_PX: f32 = 3.0;

/// THE WIDE AUTHORED BOUND: an ABSOLUTE sanity ceiling for a REAL-ink-to-REAL-
/// ink transition the repair round's neighbor-borrow does not collapse to
/// literal adjacency on its own — a LIGATURE cluster next to a plain glyph
/// (both read straight off [`TextPipeline::caret_anchor_raster_box`], never
/// the synthetic path at all) or the wrap-boundary sibling. Deliberately
/// ABSOLUTE, not "no worse than the pre-105 formula": the repair round found
/// that framing was the WRONG invariant here — pre-105's number was a crude
/// row-centred guess, post-105's is the glyph's OWN real ink (a genuine
/// deliberate improvement), so the two can legitimately differ
/// by any natural amount without either being a regression; a "no worse than
/// old" check on a fixture where "new" is now simply CORRECT produced a false
/// positive (Mopoke's real `fi`→`n` ligature step is 6.0px, comfortably sane,
/// but exceeded a tightened `old_d`-relative margin). See
/// `every_glyph_class_closes_exactly_at_the_literal_eol_seam` for the class
/// that DOES deserve a `old_d`-relative regression guard (a SYNTHETIC
/// approximation with no real ink to fall back on) — that class is now tested
/// at literal adjacency instead, where neighbor-borrow closes it exactly.
const TRANSITION_BOUND_WIDE_PX: f32 = 7.5;

/// The MINIMUM pre-105 discontinuity a fixture must reproduce to prove the law
/// non-vacuous — deliberately BELOW the smallest old-bug magnitude actually
/// measured on the x-height class across the roster (2.4px, Mopoke/Galah/
/// Magpie), so the non-vacuity check itself never spuriously fires on the
/// world with the smallest (but still real) old bug.
const NONVACUITY_OLD_DELTA_MIN_PX: f32 = 2.0;

/// A LOOSER non-vacuity floor for classes whose pre-105 delta was already
/// small by coincidence (a digit or a capital sitting close to the OLD
/// fixed-cell height on some worlds — Bombora's digit old Δ is 0.51px) — still
/// enough to prove the pre-105 code was not ALREADY perfectly continuous
/// there, without demanding every class reproduce the x-height class's own
/// (larger) bug magnitude.
const NONVACUITY_ANY_DELTA_MIN_PX: f32 = 0.1;

/// `(center_y, height)` at `col` on `line`, via the ONE owner, at REST (settled
/// spring — `settle_caret` pins `settle_factor` to 1 so this is a genuine
/// rest-to-rest comparison, never mid-glide).
fn cell_at(p: &mut TextPipeline, text: &str, line: usize, col: usize) -> (f32, f32) {
    p.set_view(&view(text, line, col));
    p.settle_caret();
    p.caret_cell_vertical()
}

/// The OLD line-cell arm's `(center_y, height)` at the CURRENT
/// view, byte-identical to `caret_cell_vertical`'s fallback arm before this
/// item — the non-vacuity oracle every sweep below checks itself against.
fn old_fallback_cell(p: &TextPipeline) -> (f32, f32) {
    (p.caret.pos.y, p.metrics.caret_block_h * p.cursor_scale())
}

/// The max-norm distance between two `(cy, h)` cells — the same quantity every
/// bound/non-vacuity check below compares against a threshold.
fn cell_delta(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// Assert the step from `(cy0, h0)` to `(cy1, h1)` stays inside `bound_px`
/// (scaled by this pipeline's own pixel scale).
fn assert_bounded_by(
    p: &TextPipeline,
    bound_px: f32,
    what: &str,
    cy0: f32,
    h0: f32,
    cy1: f32,
    h1: f32,
) {
    let bound = bound_px * pixel_scale(p);
    let d_cy = (cy1 - cy0).abs();
    let d_h = (h1 - h0).abs();
    assert!(
        d_cy <= bound && d_h <= bound,
        "{what}: adjacent-column cell jumped beyond the authored bound \
         ({bound:.2}px): Δcy={d_cy:.2} Δh={d_h:.2} \
         ({cy0:.2},{h0:.2}) -> ({cy1:.2},{h1:.2})"
    );
}

/// [`assert_bounded_by`] at the TIGHT (x-height-class) bound.
fn assert_bounded(p: &TextPipeline, what: &str, cy0: f32, h0: f32, cy1: f32, h1: f32) {
    assert_bounded_by(p, TRANSITION_BOUND_PX, what, cy0, h0, cy1, h1);
}

/// [`assert_bounded_by`] at the WIDE (absolute-sanity) bound.
fn assert_bounded_wide(p: &TextPipeline, what: &str, cy0: f32, h0: f32, cy1: f32, h1: f32) {
    assert_bounded_by(p, TRANSITION_BOUND_WIDE_PX, what, cy0, h0, cy1, h1);
}

/// THE HEADLINE FIXTURE, swept over the FULL proportional-world roster: the
/// user's literal `aaa` line, comparing the caret ON the final `a` (the real
/// ink-box arm) against one column later, at end-of-line (the fallback arm).
/// Reproduces the reported bug at its exact seam, and proves it repaired
/// everywhere a proportional world ships, not just Mopoke/Gumtree.
#[test]
fn aaa_to_eol_transition_is_bounded_on_every_proportional_world() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping aaa_to_eol_transition_is_bounded_on_every_proportional_world: no wgpu adapter"
        );
        return;
    };
    let text = "aaa";
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();

        let (cy_glyph, h_glyph) = cell_at(&mut p, text, 0, 2); // on the final 'a'
        assert!(
            p.caret_anchor_ink_box().is_some(),
            "{}: fixture must anchor a real ink box on the final 'a'",
            t.name
        );

        // NON-VACUITY: at the EOL column, what the OLD fallback arm would have
        // drawn must itself clear a floor well below the authored bound against
        // the glyph column — proving this fixture really does reproduce the
        // reported jump, on every world, not merely the two worst-case ones.
        p.set_view(&view(text, 0, 3));
        p.settle_caret();
        let (old_cy, old_h) = old_fallback_cell(&p);
        let old_d = cell_delta((cy_glyph, h_glyph), (old_cy, old_h));
        let floor = NONVACUITY_OLD_DELTA_MIN_PX * pixel_scale(&p);
        assert!(
            old_d > floor,
            "{}: fixture must reproduce the pre-105 jump (old Δ={old_d:.2} vs floor {floor:.2}) \
             or this law is vacuous",
            t.name
        );

        // THE LAW: the ACTUAL (repaired) fallback cell must stay bounded.
        let (cy_eol, h_eol) = p.caret_cell_vertical();
        assert_bounded(
            &p,
            &format!("{} aaa->EOL", t.name),
            cy_glyph,
            h_glyph,
            cy_eol,
            h_eol,
        );
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE MONO COMPLEMENT: the uniform grid never leaves the line-cell arm
/// on EITHER side of an "aaa"->EOL step (the ink-box arm is gated off entirely
/// on a mono world), so there is no seam to jump across — the transition must
/// be EXACTLY zero, not merely bounded.
#[test]
fn aaa_to_eol_transition_is_exactly_zero_on_every_mono_world() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping aaa_to_eol_transition_is_exactly_zero_on_every_mono_world: no wgpu adapter"
        );
        return;
    };
    let text = "aaa";
    let worlds = super::facepitch::mono_display_worlds();
    assert!(
        worlds.len() >= 7,
        "every mono-display world is swept, got {worlds:?}"
    );

    for world in worlds {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let (cy0, h0) = cell_at(&mut p, text, 0, 2);
        assert!(
            p.caret_anchor_ink_box().is_none(),
            "{world}: a mono world must never take the ink-box arm"
        );
        let (cy1, h1) = cell_at(&mut p, text, 0, 3);
        assert!(
            (cy1 - cy0).abs() < 1e-3 && (h1 - h0).abs() < 1e-3,
            "{world}: the mono grid must show NO transition at all (Δcy={} Δh={})",
            (cy1 - cy0).abs(),
            (h1 - h0).abs()
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// EVERY REPRESENTATIVE GLYPH CLASS, swept over the FULL proportional-world
/// roster, at the LITERAL adjacent seam: ascender, x-height, descender,
/// punctuation, digit, and CAPITAL — each as the very last character before
/// end-of-line, immediately followed by it, the exact shape the headline
/// `aaa` fixture uses for `a`. This is the axis the first repair did
/// not sweep: CAPITAL was entirely absent from its class roster, and on that
/// exact absence the first landing regressed 11/11 proportional worlds
/// against pre-105 (new Δ 2.3–4.4px vs old Δ 0.4–2.9px) without any test
/// noticing — see this file's module doc.
///
/// EVERY class now gets the TIGHT bound, not just x-height: the repair
/// round's neighbor-borrow (`caret_cell_vertical`'s fallback arm) makes a
/// literal adjacent transition BORROW the real letter's own ink rather than
/// approximate it, so the residual is not merely bounded, it is (up to float
/// rounding) exactly zero for every class this sweeps — a strictly stronger
/// claim than the first landing's per-class "close enough" bound, proven
/// per-fixture non-vacuous against the pre-105 code below.
#[test]
fn every_glyph_class_closes_exactly_at_the_literal_eol_seam() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping every_glyph_class_closes_exactly_at_the_literal_eol_seam: no wgpu adapter"
        );
        return;
    };
    // (fixture, class label) — the class char is always the LAST char, col
    // `eol - 1`, immediately followed by EOL at `eol`. A leading filler char
    // keeps every fixture a genuine two-column transition, never a bare
    // single-char line (already covered by the headline/empty-line fixtures).
    let fixtures: &[(&str, &str)] = &[
        ("xl", "ascender (l)"),
        ("xa", "x-height (a)"),
        ("xg", "descender (g)"),
        ("x.", "punctuation (.)"),
        ("x1", "digit (1)"),
        ("xA", "capital (A)"),
    ];

    // FIXTURE SANITY: the ink-box arm's OWN letter-to-letter height spread on
    // "lamp" (an ascender next to an x-height letter, both real glyphs, no
    // fallback involved) is itself several px — proof that SOME height
    // variation between adjacent columns is normal product behaviour, and
    // that the exact-closure claim below comes from borrowing the real
    // neighbor's ink, not from every class secretly measuring the same.
    {
        theme::set_active_by_name("Gumtree").unwrap();
        p.sync_theme();
        let (_cy_l, h_l) = cell_at(&mut p, "lamp", 0, 0);
        let (_cy_a, h_a) = cell_at(&mut p, "lamp", 0, 1);
        assert!(
            (h_l - h_a).abs() > 2.0,
            "fixture sanity: adjacent real glyphs of different classes must \
             already show a real height spread (l={h_l} a={h_a})"
        );
    }

    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;
    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        for &(text, label) in fixtures {
            let eol = text.chars().count();
            let anchor = eol - 1;
            let (cy0, h0) = cell_at(&mut p, text, 0, anchor);
            assert!(
                p.caret_anchor_ink_box().is_some(),
                "{} {label}: fixture must anchor a real ink box",
                t.name
            );

            // NON-VACUITY: the OLD (pre-105) fallback must genuinely have
            // differed here, or this fixture proves nothing.
            p.set_view(&view(text, 0, eol));
            p.settle_caret();
            let old_d = cell_delta((cy0, h0), old_fallback_cell(&p));
            let floor = NONVACUITY_ANY_DELTA_MIN_PX * pixel_scale(&p);
            assert!(
                old_d > floor,
                "{} {label}: fixture must reproduce SOME pre-105 discontinuity \
                 (old Δ={old_d:.2} vs floor {floor:.2}) or this law is vacuous",
                t.name
            );

            let (cy1, h1) = p.caret_cell_vertical();
            assert_bounded(&p, &format!("{} {label} -> EOL", t.name), cy0, h0, cy1, h1);
        }
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE LIGATURE SEAM: `caret_anchor_ink_box` deliberately gates a multi-char
/// LIGATURE cluster OUT of the real ink-box arm (its horizontal ink can't be
/// fairly split across the chars it covers), so a ligature-anchored column
/// used to fall all the way to the fixed line-cell — the SAME jump the
/// glyphless case had. `"fine"` genuinely ligates `fi` into one glyph on every
/// bundled proportional prose face (confirmed by probe: `caret_anchor_ink_box`
/// is `None` at both col 0 and col 1 while `caret_anchor_raster_box` is
/// `Some`), so col 1 (still inside the `fi` cluster) followed by col 2 (the
/// plain `n` glyph) is a REAL ligature -> plain-glyph transition, not a
/// synthetic stand-in.
#[test]
fn ligature_to_plain_glyph_transition_is_bounded() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping ligature_to_plain_glyph_transition_is_bounded: no wgpu adapter");
        return;
    };
    let text = "fine";
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;
    let mut saw_real_ligature = false;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();

        p.set_view(&view(text, 0, 1)); // still inside the "fi" cluster
        p.settle_caret();
        let is_ligature =
            p.caret_anchor_ink_box().is_none() && p.caret_anchor_raster_box().is_some();
        if !is_ligature {
            // Not every face is guaranteed to ligate "fi" (a face without the
            // `liga` feature would shape one glyph per char) — skip rather than
            // fail on a world that doesn't reproduce the precondition.
            continue;
        }
        saw_real_ligature = true;
        let (cy0, h0) = p.caret_cell_vertical();

        let (cy1, h1) = cell_at(&mut p, text, 0, 2); // the plain 'n' — the ink-box
        // arm's own value, unchanged by the transition repair.
        assert!(
            p.caret_anchor_ink_box().is_some(),
            "{}: 'n' must be a plain single-glyph anchor",
            t.name
        );
        // "fi" (an ascender-height ligature, REAL ink) next to a plain
        // x-height 'n' (also REAL ink) — an absolute sanity bound, not a
        // "no worse than the pre-105 formula" comparison: both sides are now
        // genuine glyph ink, so they may legitimately differ by a natural
        // amount (see `TRANSITION_BOUND_WIDE_PX`'s doc for why comparing this
        // to the old crude fallback was the wrong invariant).
        assert_bounded_wide(&p, &format!("{} ligature->plain", t.name), cy0, h0, cy1, h1);
        checked += 1;
    }
    assert!(
        saw_real_ligature && checked >= 1,
        "fixture must reproduce a real ligature cluster on at least one proportional world"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE WRAPPED-LINE BOUNDARY: the space where a long proportional line
/// soft-wraps collapses to a near-zero raw cell (the same fixture
/// `block_caret_full_cell_on_wrap_boundary_space` uses), and both the real
/// glyph just before it and the collapsed space itself sit on the SAME visual
/// row (row 0's half-open span). This is the horizontal-adjacency sibling of
/// the EOL case at a genuinely different geometric boundary.
#[test]
fn wrap_boundary_transition_is_bounded_on_a_proportional_world() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping wrap_boundary_transition_is_bounded_on_a_proportional_world: no wgpu adapter"
        );
        return;
    };
    let long = "word ".repeat(80);

    theme::set_active_by_name("Gumtree").unwrap();
    p.sync_theme();
    p.set_view(&view(&long, 0, 0));
    let rows = p.visual_rows(0);
    assert!(rows.len() >= 2, "fixture should wrap ({} rows)", rows.len());
    let space_col = rows[1].start_col - 1; // the collapsed wrap-boundary space
    let last_glyph_col = space_col - 1; // the real letter right before it

    let (cy0, h0) = cell_at(&mut p, &long, 0, last_glyph_col);
    assert!(
        p.caret_anchor_ink_box().is_some(),
        "the char before the wrap boundary must anchor a real ink box"
    );
    p.set_view(&view(&long, 0, space_col));
    p.settle_caret();
    let (cy1, h1) = p.caret_cell_vertical();
    // The collapsed space's anchor column is `last_glyph_col + 1`, so the
    // repair round's neighbor-borrow reads `last_glyph_col`'s own real ink
    // directly — the SAME box (cy0, h0) already came from — closing this to
    // (near-)exact rather than merely bounded.
    assert_bounded(&p, "Gumtree wrap boundary", cy0, h0, cy1, h1);

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// LEADING WHITESPACE BEFORE A CAPITAL. A one-sided neighbor borrow
/// tried only `raster_box_at(col - 1)`, a single BACKWARD hop, so a glyphless
/// anchor at COLUMN 0 (no `col - 1` to borrow from at all) fell straight to
/// the synthetic guess even though a real letter sits one column FORWARD.
/// `" A"` is the literal mirror-direction shape of every fixture the first
/// round shipped (all real-glyph -> glyphless, never glyphless -> real-glyph):
/// any line beginning with a leading space or indentation before a
/// capitalized word reproduces this directly. The current
/// `TextPipeline::nearest_row_raster_box` searches OUTWARD in both
/// directions), so column 0 now finds the capital ONE column forward and the
/// seam closes.
#[test]
fn leading_glyphless_column_at_col_zero_closes_against_the_next_real_glyph() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping leading_glyphless_column_at_col_zero_closes_against_the_next_real_glyph: no wgpu adapter"
        );
        return;
    };
    let text = " A";
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();

        // col 0: the leading space — glyphless, and (the point of this
        // fixture) has no `col - 1` at all to borrow from.
        let (cy0, h0) = cell_at(&mut p, text, 0, 0);
        assert!(
            p.caret_anchor_ink_box().is_none(),
            "{}: a leading space must not itself anchor a real ink box",
            t.name
        );

        let (cy_a, h_a) = cell_at(&mut p, text, 0, 1); // col 1: the capital 'A'
        assert!(
            p.caret_anchor_ink_box().is_some(),
            "{}: 'A' must anchor a real ink box",
            t.name
        );

        // NON-VACUITY: the OLD (pre-105) fallback must genuinely have differed
        // from the real 'A' ink, or this fixture proves nothing.
        p.set_view(&view(text, 0, 0));
        p.settle_caret();
        let old_d = cell_delta((cy_a, h_a), old_fallback_cell(&p));
        let floor = NONVACUITY_ANY_DELTA_MIN_PX * pixel_scale(&p);
        assert!(
            old_d > floor,
            "{}: fixture must reproduce SOME pre-105 discontinuity (old Δ={old_d:.2} \
             vs floor {floor:.2}) or this law is vacuous",
            t.name
        );

        // THE LAW: col 0 (leading space) and col 1 ('A') stay bounded — the
        // outward search finds col 1's own real ink from col 0, so this
        // closes to (near-)zero exactly like the mirror (real-glyph ->
        // glyphless) direction already does.
        assert_bounded(
            &p,
            &format!("{} leading-space->A", t.name),
            cy0,
            h0,
            cy_a,
            h_a,
        );
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// A RUN OF 2+ CONSECUTIVE GLYPHLESS COLUMNS — the SECOND repair round (found
/// auditing the first one). The first round's neighbor-borrow was a SINGLE
/// backward hop: the second glyphless column in any run has a `col - 1` that
/// is ITSELF glyphless, so the hop fails and that column falls straight to
/// the synthetic guess — jumping against its own immediate neighbor, which
/// DID borrow real ink one column earlier. `"A  "` (capital, two trailing
/// spaces, then EOL) is the canonical shape: a markdown hard-break's own two
/// trailing spaces, or an ordinary mid-paragraph double space, reproduce this
/// directly. Every adjacent pair across the run — 'A'->space1,
/// space1->space2, space2->EOL — must stay bounded; and since the second
/// round's fix searches OUTWARD rather than stopping at one hop, space1,
/// space2, and EOL all resolve to the exact SAME borrowed 'A' ink, so they
/// read identically to each other (near-zero, not merely bounded) — proof
/// the fix reaches ACROSS the whole run instead of degrading one column in.
#[test]
fn run_of_glyphless_columns_stays_bounded_end_to_end() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping run_of_glyphless_columns_stays_bounded_end_to_end: no wgpu adapter");
        return;
    };
    let text = "A  "; // capital, two trailing spaces, EOL at col 3
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();

        let (cy_a, h_a) = cell_at(&mut p, text, 0, 0); // 'A' — real ink
        assert!(
            p.caret_anchor_ink_box().is_some(),
            "{}: 'A' must anchor a real ink box",
            t.name
        );

        let (cy_s1, h_s1) = cell_at(&mut p, text, 0, 1); // first trailing space
        assert!(
            p.caret_anchor_ink_box().is_none(),
            "{}: space 1 must be glyphless",
            t.name
        );

        let (cy_s2, h_s2) = cell_at(&mut p, text, 0, 2); // second trailing space
        assert!(
            p.caret_anchor_ink_box().is_none(),
            "{}: space 2 must be glyphless",
            t.name
        );

        let (cy_eol, h_eol) = cell_at(&mut p, text, 0, 3); // literal EOL

        // NON-VACUITY: the OLD (pre-105) fallback must genuinely have differed
        // from 'A''s real ink at the FAR end of the run (space 2, two hops
        // deep — the exact column the first repair round's single hop could
        // never reach), or this fixture proves nothing about the run's far
        // column.
        p.set_view(&view(text, 0, 2));
        p.settle_caret();
        let old_d = cell_delta((cy_a, h_a), old_fallback_cell(&p));
        let floor = NONVACUITY_ANY_DELTA_MIN_PX * pixel_scale(&p);
        assert!(
            old_d > floor,
            "{}: fixture must reproduce SOME pre-105 discontinuity at the far \
             end of the run (old Δ={old_d:.2} vs floor {floor:.2}) or this law is vacuous",
            t.name
        );

        // THE LAW: every adjacent pair across the run stays bounded.
        assert_bounded(&p, &format!("{} A->space1", t.name), cy_a, h_a, cy_s1, h_s1);
        assert_bounded(
            &p,
            &format!("{} space1->space2", t.name),
            cy_s1,
            h_s1,
            cy_s2,
            h_s2,
        );
        assert_bounded(
            &p,
            &format!("{} space2->EOL", t.name),
            cy_s2,
            h_s2,
            cy_eol,
            h_eol,
        );

        // THE MECHANISM CLAIM: space1, space2, and EOL all borrow the SAME
        // real 'A' ink via the outward search (same baseline, same box), so
        // they read (near-)identically to one another — not merely "within
        // bound".
        let tight = 0.05 * pixel_scale(&p);
        assert!(
            (h_s1 - h_s2).abs() < tight && (h_s2 - h_eol).abs() < tight,
            "{}: every glyphless column in the run must borrow the SAME real ink \
             (h_s1={h_s1:.3} h_s2={h_s2:.3} h_eol={h_eol:.3})",
            t.name
        );
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// How far ABOVE the same world's real x-height letter cell an empty line's
/// synthetic cell may sit (px at zoom×DPI 1). The synthetic box is a TYPICAL
/// letter — the mean of the face's x-height and cap-height — so it is
/// legitimately a little taller than an `a`; measured residual across the
/// proportional roster after the empty-row repair: +0.40..+2.60px.
const EMPTY_LINE_OVER_LETTER_PX: f32 = 3.0;

/// How far BELOW that same letter cell it may sit. This is the SIZE FLOOR the
/// user-reported empty-line defect broke: with cosmic-text handing an empty row
/// a `max_ascent` of ZERO, the synthetic box clamped to its 1px guard and the
/// whole arm collapsed onto `caret_visual_body_dims`'s minimum body — 13.31px
/// on EVERY world (the identical-everywhere value is the tell), against letter
/// cells of 16.00–20.00px. The repaired arm never lands below the letter cell
/// at all, so this is pure slack.
const EMPTY_LINE_UNDER_LETTER_PX: f32 = 0.5;

/// AN EMPTY LINE'S synthetic cell TRACKS THE TYPE ON THE SAME WORLD — bounded
/// on BOTH sides against that world's own real ink-arm height for an ordinary
/// x-height letter, which the synthetic box is explicitly modelled to
/// approximate.
///
/// * ABOVE: never the "large empty accent cap" (the fixed ~22.4px line-box
///   cell regardless of the font).
/// * BELOW: never smaller than the type beside it — the reported defect, where
///   an empty row's zero `max_ascent` floored the cell at the minimum visible
///   body.
///
/// The previous bound here was ONE-SIDED (`h_glyph * 1.05`, an upper limit
/// only) and was calibrated against the DEFECT's own number: `h_empty` was
/// 7.00px then, identical on every world, and a value that sits far under every
/// ceiling reads as comfortably passing while being the bug. A ceiling alone is
/// satisfiable by the caret shrinking to nothing, so both bounds ship together
/// and both carry their own non-vacuity oracle.
///
/// SWEPT: the full proportional roster × 1x/2x DPI × the empty line in four
/// POSITIONS — its own empty document, the first line above text, the last line
/// below text, and one between two text lines. All four are the same arm, and
/// a law that only ever asks about a lone empty buffer cannot see a row lookup
/// that picks the wrong row.
#[test]
fn empty_line_cell_tracks_the_letter_cell_on_the_same_world() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping empty_line_cell_tracks_the_letter_cell_on_the_same_world: no wgpu adapter");
        return;
    };
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;
    // The empty line, in every position it can hold in a document.
    let positions: &[(&str, &str, usize)] = &[
        ("alone", "", 0),
        ("first line", "\nabc", 0),
        ("last line", "abc\n", 1),
        ("between", "abc\n\ndef", 1),
    ];
    // How many worlds the CEILING's non-vacuity oracle actually bites on: the
    // old fixed cap only exceeds `h_glyph + EMPTY_LINE_OVER_LETTER_PX` where
    // the letter cell is small enough. Derived by measurement below, never
    // pinned to named worlds.
    let mut ceiling_nonvacuous = 0usize;

    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
            theme::set_active_by_name(t.name).unwrap();
            p.sync_theme();
            let ps = pixel_scale(&p);

            // A real x-height letter's ink-arm height, on its own line...
            let (_cy_glyph, h_glyph) = cell_at(&mut p, "a", 0, 0);
            assert!(
                p.caret_anchor_ink_box().is_some(),
                "{} d{dpi}: 'a' must ink-align",
                t.name
            );
            let ceiling = h_glyph + EMPTY_LINE_OVER_LETTER_PX * ps;
            let floor = h_glyph - EMPTY_LINE_UNDER_LETTER_PX * ps;

            for &(what, text, line) in positions {
                let (_cy_empty, h_empty) = cell_at(&mut p, text, line, 0);
                assert!(
                    p.caret_anchor_ink_box().is_none(),
                    "{} d{dpi} {what}: an empty line has no ink",
                    t.name
                );
                assert!(
                    h_empty <= ceiling,
                    "{} d{dpi} {what}: empty-line cell must not balloon toward the \
                     fixed line-box cap: h_empty={h_empty:.2} h_glyph={h_glyph:.2} \
                     ceiling={ceiling:.2}",
                    t.name
                );
                assert!(
                    h_empty >= floor,
                    "{} d{dpi} {what}: empty-line cell draws SMALLER than the type \
                     beside it: h_empty={h_empty:.2} h_glyph={h_glyph:.2} \
                     floor={floor:.2}",
                    t.name
                );
            }

            // NON-VACUITY, FLOOR: the DEFECT's own cell — what this arm produces
            // when the row hands it a zero ascent, i.e. `caret_synthetic_ink_box`
            // clamped to its 1px guard and fed through the shared body floor —
            // must FAIL the floor above, or the floor proves nothing about the
            // bug it names.
            let degenerate = super::super::caret_body::caret_visual_body_dims(
                super::super::caret_body::InkBox {
                    left: 0.0,
                    top: 1.0,
                    width: 0.0,
                    height: 1.0,
                },
                ps,
            )
            .1;
            assert!(
                degenerate < floor,
                "{} d{dpi}: fixture must reproduce the zero-ascent collapse \
                 (degenerate={degenerate:.2} floor={floor:.2}) or the floor is vacuous",
                t.name
            );
            // NON-VACUITY, CEILING: the OLD fixed cap must fail the ceiling
            // wherever the letter cell is small enough for it to bite.
            if old_fallback_cell(&p).1 > ceiling {
                ceiling_nonvacuous += 1;
            }
            checked += 1;
        }
    }
    p.set_dpi(1.0);
    assert!(
        checked >= 22,
        "every proportional-display world is swept at both DPIs (got {checked})"
    );
    assert!(
        ceiling_nonvacuous >= 10,
        "the fixed line-box cap must exceed the ceiling on a real share of the \
         roster or the ceiling is vacuous (got {ceiling_nonvacuous} of {checked})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE MECHANISM UNDER THE EMPTY-LINE CELL: an empty row must report the SAME
/// `(baseline, max_ascent)` a really-shaped row on that same line reports.
///
/// cosmic-text emits an empty line's `LayoutLine` with `max_ascent: 0.0,
/// max_descent: 0.0` — a real row, with a real layout, carrying no metrics at
/// all (`shape.rs`'s "visual line for empty lines" arm). Read literally those
/// zeros put the baseline at the row's MIDPOINT instead of on the type
/// baseline, and hand the caret's synthetic ink box an ascent of nothing to
/// scale. `TextPipeline::glyphless_row_vertical` rebuilds the pair from the
/// face's own per-em ascent/descent; this pins that reconstruction against the
/// ground truth — the same line index, one letter in it — so the claim is
/// EQUALITY with a really-shaped row, not a tolerance around a guess.
///
/// It is also a CROSS-FONT-STACK claim, which is why it is a law and not a
/// comment: the reconstruction reads the face through **skrifa**
/// (`facepitch::vertical_em_metrics`), while the row it must agree with was
/// shaped through **swash**. Two stacks, two readings of the same tables. This
/// sweeps every world in the roster — mono included, since row metrics belong
/// to the row and not to the caret's arm — at two DPIs and a non-1.0 zoom, and
/// goes red the day either stack reads a bundled face differently.
///
/// NON-VACUITY: each iteration also asserts the RAW cosmic-text row really does
/// carry a zero ascent, so the equality above is measuring a reconstruction
/// rather than a value that was already there.
#[test]
fn an_empty_row_carries_the_metrics_a_shaped_row_would_have_given_it() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping an_empty_row_carries_the_metrics_a_shaped_row_would_have_given_it: no wgpu adapter"
        );
        return;
    };
    let mut checked = 0usize;
    let mut worst_ascent = 0.0f32;
    let mut worst_baseline = 0.0f32;

    for &(zoom, dpi) in &[(1.0f32, 1.0f32), (1.0, 2.0), (1.7, 1.0)] {
        p.set_dpi(dpi);
        for t in theme::THEMES.iter() {
            theme::set_active_by_name(t.name).unwrap();
            p.sync_theme();
            let ps = pixel_scale(&p);

            // GROUND TRUTH: line 0 holds one letter, so cosmic-text shapes it
            // and reports the row's real metrics.
            let mut v = view("a", 0, 0);
            v.zoom = zoom;
            p.set_view(&v);
            p.settle_caret();
            let (base_shaped, ascent_shaped, _) = p.caret_row_metrics();

            // THE SUBJECT: the same line 0, now empty.
            let mut v = view("", 0, 0);
            v.zoom = zoom;
            p.set_view(&v);
            p.settle_caret();
            let (base_empty, ascent_empty, font_empty) = p.caret_row_metrics();

            // NON-VACUITY: cosmic-text's own row really is metric-less here.
            let raw = p
                .buffer
                .lines
                .first()
                .and_then(|l| l.layout_opt())
                .and_then(|l| l.first())
                .map(|ll| (ll.max_ascent, ll.max_descent, ll.glyphs.len()));
            assert_eq!(
                raw,
                Some((0.0, 0.0, 0)),
                "{} z{zoom} d{dpi}: the empty row must really be the glyphless \
                 zero-metric row this law reconstructs (got {raw:?}) or the \
                 equality below measures nothing",
                t.name
            );
            assert_eq!(
                font_empty,
                p.doc_family(),
                "{} z{zoom} d{dpi}: a glyphless row's ascent is a property of the \
                 LIVE doc family, and must be reported as such",
                t.name
            );

            let d_ascent = (ascent_empty - ascent_shaped).abs();
            let d_baseline = (base_empty - base_shaped).abs();
            let tol = 0.05 * ps;
            assert!(
                d_ascent <= tol && d_baseline <= tol,
                "{} z{zoom} d{dpi} ({}): an empty row must carry the metrics a \
                 shaped row on the same line carries: ascent {ascent_empty:.3} vs \
                 {ascent_shaped:.3} (Δ{d_ascent:.3}), baseline {base_empty:.3} vs \
                 {base_shaped:.3} (Δ{d_baseline:.3}), tolerance {tol:.3}",
                t.name,
                p.doc_family()
            );
            worst_ascent = worst_ascent.max(d_ascent / ps);
            worst_baseline = worst_baseline.max(d_baseline / ps);
            checked += 1;
        }
    }
    p.set_dpi(1.0);
    assert!(
        checked >= 60,
        "every world is swept at every scale (got {checked})"
    );
    eprintln!(
        "an_empty_row_carries_the_metrics_a_shaped_row_would_have_given_it: \
         {checked} world×scale cells, worst ascent Δ={worst_ascent:.4}px, \
         worst baseline Δ={worst_baseline:.4}px (both at zoom×DPI 1)"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// TWO ZOOMS, including a NON-1.0 value, and 1x/2x DPI: the headline `aaa`->EOL
/// transition stays bounded at EVERY pixel scale, with the bound itself scaled
/// by that same factor (`pixel_scale`) — proving the fix is a geometric
/// relationship, not a value tuned to look right only at the capture's default
/// zoom/DPI. Mindful of the documented zoom trap: this
/// reads `caret_cell_vertical`'s OWN already-scaled pixel output directly,
/// never a sidecar field, so there is no scaled/unscaled unit mismatch to fall
/// into.
#[test]
fn transition_stays_bounded_across_zoom_and_dpi() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping transition_stays_bounded_across_zoom_and_dpi: no wgpu adapter");
        return;
    };
    let text = "aaa";

    // (zoom, dpi) pairs: the capture default, a genuinely non-1.0 zoom, and a
    // HiDPI (2x) monitor at the default zoom.
    let cases: &[(f32, f32)] = &[(1.0, 1.0), (1.7, 1.0), (1.0, 2.0)];

    for world in ["Gumtree", "Mopoke", "Bombora"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for &(zoom, dpi) in cases {
            p.set_dpi(dpi);
            let mut v = view(text, 0, 2);
            v.zoom = zoom;
            p.set_view(&v);
            p.settle_caret();
            assert!(
                (p.metrics.zoom - zoom).abs() < 1e-3,
                "{world}: zoom must actually apply (got {})",
                p.metrics.zoom
            );
            let (cy0, h0) = p.caret_cell_vertical();
            assert!(
                p.caret_anchor_ink_box().is_some(),
                "{world} z{zoom} d{dpi}: fixture must ink-align"
            );

            let mut v2 = view(text, 0, 3);
            v2.zoom = zoom;
            p.set_view(&v2);
            p.settle_caret();
            let (cy1, h1) = p.caret_cell_vertical();
            assert_bounded(
                &p,
                &format!("{world} zoom={zoom} dpi={dpi}"),
                cy0,
                h0,
                cy1,
                h1,
            );
        }
        // Restore DPI to the capture default before the next world.
        p.set_dpi(1.0);
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// MORPH, RESTING: Morph's fast-travel deferral and its ink-caret-world fold
/// both land on the very same cell quad `caret_geometry` builds from
/// `caret_cell_vertical` — at rest (`settle_factor == 1`) the ink/rise
/// corrections apply in FULL, so the drawn geometry's top/bottom must equal
/// `caret_cell_vertical`'s own numbers exactly, and the same bounded-transition
/// law must hold read through THAT path too (catching a regression introduced
/// by the blend math in `caret_geometry`, not just in the owner function).
#[test]
fn morph_rest_transition_is_bounded_through_caret_geometry() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Morph);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping morph_rest_transition_is_bounded_through_caret_geometry: no wgpu adapter"
        );
        return;
    };
    // Morph anchors one char BACK, so "aaaa" col 3 anchors the 3rd 'a' (ink
    // arm) and col 4 (EOL) anchors the 4th 'a' — still glyph-anchored, so a
    // FIFTH column is needed to reach the true glyphless EOL anchor Morph
    // shows once past the last character.
    let text = "aaaa";
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();

        p.set_view(&view(text, 0, 4)); // anchors the 4th 'a' (Morph: col-1)
        p.settle_caret();
        let (owner_cy, owner_h) = p.caret_cell_vertical();
        let (_cx, geo_cy, _w, geo_h, ..) = p.caret_geometry();
        assert!(
            (owner_cy - geo_cy).abs() < 1e-2 && (owner_h - geo_h).abs() < 1e-2,
            "{}: at rest, caret_geometry must equal caret_cell_vertical exactly \
             (owner=({owner_cy},{owner_h}) geometry=({geo_cy},{geo_h}))",
            t.name
        );

        p.set_view(&view(text, 0, 5)); // EOL: Morph anchors the 4th 'a' again...
        p.settle_caret();
        // ...so instead compare against the true glyphless case: an anchor past
        // a TRAILING SPACE, which Morph's space-bar geometry reads through the
        // SAME owner.
        let spaced = "aaa ";
        p.set_view(&view(spaced, 0, 4)); // Morph anchors col 3, the space itself
        p.settle_caret();
        assert!(
            p.caret_anchor_ink_box().is_none(),
            "{}: a trailing space must be a glyphless Morph anchor",
            t.name
        );
        let (cy1, h1) = p.caret_cell_vertical();
        assert_bounded(
            &p,
            &format!("{} morph rest a->space", t.name),
            owner_cy,
            owner_h,
            cy1,
            h1,
        );
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// MORPH, TRAVELLING: the transition repair touches only the REST endpoint inside
/// `caret_cell_vertical`; a moving caret is a thin STREAK
/// (`motion_geometry`), with no cell to jump between columns at all. Widens
/// `caret_ink_box.rs`'s own `moving_caret_streak_is_unaffected_by_the_ink_box`
/// (which swept two worlds) to the FULL proportional roster, so this item's
/// change is proven not to have introduced a settle/travel thickness pop
/// anywhere it ships.
#[test]
fn morph_travel_stays_a_thin_streak_on_every_proportional_world() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping morph_travel_stays_a_thin_streak_on_every_proportional_world: no wgpu adapter"
        );
        return;
    };
    let text = "alpha\nbeta\ngamma\ndelta\nepsilon\nzeta\neta\ntheta\niota";
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        p.set_view(&view(text, 0, 0));

        p.inject_motion_demo();
        let (_cx, cy, w, h, ..) = p.caret_geometry();
        let s = p.caret.settle_factor();
        assert!(
            s < 0.2,
            "{}: fixture must be genuinely mid-glide (s={s})",
            t.name
        );
        assert!(
            w > h,
            "{}: motion pose must be long-and-thin: w={w} h={h}",
            t.name
        );
        assert!(
            h < p.metrics.caret_block_h * 0.5,
            "{}: the streak must stay thin — the ink/synthetic box must not thicken it: h={h}",
            t.name
        );
        let want_cy = p.caret.pos.y + p.metrics.caret_trail_drop;
        assert!(
            (cy - want_cy).abs() < 0.5,
            "{}: the streak must run through the TEXT centre, not the ink box: cy={cy} want={want_cy}",
            t.name
        );
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE THEME-PICKER PREVIEW SEAM. The
/// caret's proportional-fallback branch gate and its synthetic ratio lookup
/// must read [`TextPipeline::doc_family`] (the LIVE face the ACTIVE theme
/// wants) — NOT `shaped_font` (the face the document is ACTUALLY shaped in
/// right now). `sync_theme_colors` (`App::retint_theme_preview`'s per-arrow
/// step) re-tints every baked colour and switches the active theme instantly
/// but deliberately LEAVES `shaped_font` stale until the separately-deferred
/// font reshape (`sync_theme_font`) catches up — the whole point of the
/// split, so a fast preview scrub never pays a reshape per arrow press.
///
/// This matters because a GLYPHLESS anchor's fallback is now
/// font-aware, so reading the LAGGING `shaped_font` there would leave the
/// caret itself showing STALE (source-world) geometry for the entire window
/// between a preview's color retint and its deferred reshape — exactly the
/// kind of surface `render::tests::distinguishability`'s
/// `theme_preview_retint_regrounds_the_page_surface_on_every_world` law exists
/// to catch (a full-frame pixel diff caught this directly during development;
/// this is the fast unit-level companion, pinned at the exact seam).
///
/// Non-vacuous: reverting the caller's gate to `self.shaped_font` makes this
/// red — the MONO source's stale `shaped_font` makes
/// `caret_cell_vertical` take the old byte-identical MONO branch even after
/// the active theme (and the caret's OWN colour) have already moved to a
/// PROPORTIONAL destination.
#[test]
fn caret_fallback_geometry_tracks_the_live_theme_not_the_lagging_shaped_font() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping caret_fallback_geometry_tracks_the_live_theme_not_the_lagging_shaped_font: no wgpu adapter"
        );
        return;
    };
    // An EMPTY buffer: no real glyph anywhere, so `caret_cell_vertical` is
    // ALWAYS in the fallback (glyphless) case — the exact seam under test.
    p.set_view(&view("", 0, 0));

    // Fully settle on a MONO source world (Tawny) — a real render, not just a
    // theme swap, so any state only touched at draw time is genuinely present.
    theme::set_active_by_name("Tawny").unwrap();
    p.sync_theme();
    p.settle_caret();
    assert!(
        crate::caret::font_is_mono(p.shaped_font),
        "fixture must start on a genuinely mono-shaped buffer"
    );

    // The COLD reference: a full settle directly on the PROPORTIONAL
    // destination (Magpie) — the ground truth the preview step below must
    // reproduce byte-for-byte.
    theme::set_active_by_name("Magpie").unwrap();
    p.sync_theme();
    p.settle_caret();
    let cold = p.caret_cell_vertical();

    // Back to the MONO source, fully settled again...
    theme::set_active_by_name("Tawny").unwrap();
    p.sync_theme();
    p.settle_caret();

    // ...then the PREVIEW step: switch active to Magpie but apply ONLY the
    // color half — `shaped_font` stays "IBM Plex Mono" (Tawny's), exactly the
    // state a picker arrow leaves before the deferred reshape.
    theme::set_active_by_name("Magpie").unwrap();
    p.sync_theme_colors();
    assert_eq!(
        p.shaped_font, "IBM Plex Mono",
        "fixture must reproduce the lag: shaped_font stays the SOURCE's face \
         after a color-only retint"
    );
    assert!(
        !crate::caret::font_is_mono(p.doc_family()),
        "the LIVE active theme (Magpie) must already read as proportional"
    );
    let preview = p.caret_cell_vertical();

    assert!(
        (preview.0 - cold.0).abs() < 1e-3 && (preview.1 - cold.1).abs() < 1e-3,
        "the color-only preview's caret fallback must already match the cold \
         destination's geometry, not the lagging shaped_font's: preview={preview:?} \
         cold={cold:?}"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE THEME-PREVIEW SEAM ON REAL TEXT (found auditing the repair round's own
/// first fix to [`TextPipeline::caret_fallback_geometry_tracks_the_live_theme_not_the_lagging_shaped_font`]).
/// That law's ONLY fixture is an EMPTY buffer, where `caret_row_metrics`'s
/// ascent approximation is `self.metrics.font_size * 0.8` — theme-INDEPENDENT
/// by construction (`Metrics::with_dpi` never reads the active theme) — so
/// keying the ratio on the LIVE `doc_family()` there costs nothing and the law
/// holds exactly. On any document with REAL text, `caret_row_metrics` instead
/// reads a genuinely shaped [`cosmic_text::LayoutLine`]'s `max_ascent`, a
/// property of `shaped_font` (the face still ACTUALLY on screen mid-preview,
/// stale until the deferred reshape). Multiplying THAT ascent by a DIFFERENT
/// font's ratio (`doc_family()`, live) produces a mixed-font number neither
/// factor alone would — confirmed empirically (throwaway probe, reverted):
/// worst case 5.19px at (Tawny → Bilby), the SAME magnitude as the original
/// transition bug this whole file exists to close.
///
/// THE FIX: `caret_synthetic_ink_box`'s ratio now reads `caret_row_metrics`'s
/// own THIRD element — whichever font actually produced the ascent it is
/// paired with — never an independently-chosen font. This does NOT (and
/// cannot, without paying for the very reshape the debounce exists to defer)
/// make the preview match the COLD destination exactly on real text — the
/// row's actual on-screen geometry genuinely IS still the source font's until
/// the reshape catches up, so some residual is inherent to the design, not a
/// bug. What the fix buys is INTERNAL consistency (one font's ascent times
/// THAT SAME font's ratio, never a cross-font product) and a real, measured
/// drop in the worst-case residual, swept over every mono-source ×
/// proportional-destination pair on a genuinely shaped row.
///
/// FIXTURE: a real letter ANYWHERE on the caret's row is no longer safe here —
/// the nearest-raster repair widened the
/// neighbor-borrow from one fixed hop to an OUTWARD search across the WHOLE
/// row (`TextPipeline::nearest_row_raster_box`), so a row containing any real
/// ink at all (the original `"a  "`) now legitimately borrows it instead of
/// taking the synthetic path this law means to isolate. An ALL-WHITESPACE row
/// (`"   "`) has a genuinely shaped `LayoutLine` (unlike a truly empty line,
/// which takes a different, already-covered fallback) but literally zero
/// rasterizable ink at ANY column, so the outward search finds nothing no
/// matter how far it reaches — the fixture that stays synthetic-only under
/// BOTH the first AND second repair rounds' neighbor-borrow reach.
#[test]
fn caret_synthetic_ratio_reads_the_same_font_as_its_paired_ascent() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping caret_synthetic_ratio_reads_the_same_font_as_its_paired_ascent: no wgpu adapter"
        );
        return;
    };
    // Bound: comfortably above the fixed formula's measured worst, comfortably
    // below the mixed-font formula's measured worst (5.19px) — proven
    // non-vacuous by the assert below over the SAME sweep.
    const WORST_CASE_BOUND_PX: f32 = 3.5;

    let text = "   "; // all whitespace: a shaped row with NO real ink anywhere,
    // so the outward neighbor-borrow search (however far it reaches) always
    // comes up empty and every column takes the synthetic path.
    let mono = super::facepitch::mono_display_worlds();
    let prop: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|t| !mono.contains(&t.name))
        .map(|t| t.name)
        .collect();
    assert!(
        mono.len() >= 7 && prop.len() >= 11,
        "full roster on both sides"
    );

    let mut worst = 0.0f32;
    let mut worst_pair = ("", "");
    for &src in &mono {
        for &dst in &prop {
            // COLD: settle directly on the destination — ground truth.
            theme::set_active_by_name(src).unwrap();
            p.sync_theme();
            p.set_view(&view(text, 0, 2));
            p.settle_caret();
            theme::set_active_by_name(dst).unwrap();
            p.sync_theme();
            p.settle_caret();
            let cold = p.caret_cell_vertical();

            // PREVIEW: back to the mono source, settle, then a COLOR-ONLY
            // retint to the destination — `shaped_font` stays the source's.
            theme::set_active_by_name(src).unwrap();
            p.sync_theme();
            p.set_view(&view(text, 0, 2));
            p.settle_caret();
            assert!(
                p.caret_anchor_ink_box().is_none(),
                "fixture must be glyphless at the anchor"
            );
            let src_family = p.shaped_font; // the FONT family, not the world name
            theme::set_active_by_name(dst).unwrap();
            p.sync_theme_colors();
            assert_eq!(p.shaped_font, src_family, "fixture must reproduce the lag");
            let preview = p.caret_cell_vertical();

            let d = cell_delta(cold, preview);
            let bound = WORST_CASE_BOUND_PX * pixel_scale(&p);
            assert!(
                d <= bound,
                "{src} -> {dst}: preview/cold delta {d:.2} exceeds the bound {bound:.2} \
                 (cold={cold:?} preview={preview:?})"
            );
            if d > worst {
                worst = d;
                worst_pair = (src, dst);
            }
        }
    }
    // NON-VACUITY: the sweep must exercise a genuinely nonzero worst case —
    // otherwise every pair happening to read the same ratio trivially passes.
    assert!(
        worst > 0.5,
        "fixture must reproduce a real nonzero lag residual somewhere in the \
         roster (worst={worst:.2} at {worst_pair:?}) or this law is vacuous"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE INTERPOLATION REPAIR. `nearest_row_raster_box` previously
/// searched outward and hard-PICKED whichever side won (backward on a tie).
/// That borrows exactly ONE side's ink, so a glyphless column tied between
/// two DIFFERENT real letters (a table's `"| 1"` — pipe one side, digit the
/// other, tied at distance 1) read as pure pipe ink: the backward seam
/// (pipe->space) closed to a clean 0.00px while the untouched FORWARD seam
/// (space->digit) inherited the WHOLE pipe-to-digit gap. Flipping the
/// tiebreak to prefer forward is NOT a fix — it only rotates the same whole
/// gap onto the other seam, still directional, still one seam absorbing
/// everything.
///
/// THE BAR. Rather than assume an arbitrary ceiling, this law MEASURES what
/// the product already ships and accepts: the real GLYPH-TO-GLYPH adjacent
/// cell delta between genuinely different letter classes (x-height,
/// ascender, descender, capital, digit, punctuation, ligature) — transitions
/// nobody has ever filed as a bug, because two adjacent DIFFERENT real
/// letters legitimately draw different cells. That empirical worst case
/// becomes the ceiling a glyphless seam must clear: "no worse than the
/// glyph-to-glyph seams the product already accepts" (not an assumed
/// constant — see the round's own report for the literal numbers: median
/// ~5.5-6px, worst 14.0px, `digit->punctuation` on Bombora/Mopoke/Galah).
///
/// THE FIX PROVEN HERE: blending the two sides by relative distance makes
/// the borrow non-directional — the tied `"| 1"` space now reads as the
/// MIDPOINT of pipe and digit, so EACH seam absorbs roughly half the gap
/// instead of one absorbing all of it. Both seams measure well inside the
/// empirical bar on every world (worst observed 4.0px, a third of the
/// bar's 14.0px), and — the mechanism claim — the two seams are now
/// (near-)SYMMETRIC, which a hard pick structurally cannot
/// produce (a hard pick always drove one side to 0 and the other to the
/// full gap).
#[test]
fn glyphless_seams_stay_within_the_products_own_accepted_glyph_to_glyph_bar() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping glyphless_seams_stay_within_the_products_own_accepted_glyph_to_glyph_bar: no wgpu adapter"
        );
        return;
    };
    let mono = super::facepitch::mono_display_worlds();
    let prop: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|t| !mono.contains(&t.name))
        .map(|t| t.name)
        .collect();
    assert!(
        prop.len() >= 11,
        "full proportional roster (got {})",
        prop.len()
    );

    // ---- STEP 1: measure the bar. Real glyph pairs, adjacent columns, both
    // sides genuine ink (never the fallback arm) — the transitions the
    // product ships today with nobody calling them a bug.
    let glyph_pairs: &[&str] = &["al", "ag", "a1", "a.", "aA", "lg", "1.", "A1", "Ag", ".A"];
    let mut bar = 0.0f32;
    for &world in &prop {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for &text in glyph_pairs {
            p.set_view(&view(text, 0, 0));
            p.settle_caret();
            if p.caret_anchor_ink_box().is_none() {
                continue;
            }
            let c0 = p.caret_cell_vertical();
            p.set_view(&view(text, 0, 1));
            p.settle_caret();
            if p.caret_anchor_ink_box().is_none() {
                continue;
            }
            let c1 = p.caret_cell_vertical();
            bar = bar.max(cell_delta(c0, c1) / pixel_scale(&p));
        }
    }
    // NON-VACUITY: the bar itself must be a real, non-trivial number, or
    // "hold glyphless seams to it" is a vacuous claim.
    assert!(
        bar > 5.0,
        "the measured glyph-to-glyph bar must be a real, non-trivial ceiling \
         (got {bar:.2}px) or this law tests nothing"
    );

    // ---- STEP 2: every glyphless seam this round's fix targets, measured
    // against that SAME bar, on every proportional world.
    let mut worst_glyphless = 0.0f32;
    let mut sym_worst = 0.0f32;
    for &world in &prop {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let ps = {
            p.set_view(&view("a", 0, 0));
            pixel_scale(&p)
        };
        let bound = bar * ps;

        // THE OPEN DEFECT ITSELF: "| 1" (pipe, space, digit) — BOTH seams.
        let c_pipe = cell_at(&mut p, "| 1", 0, 0);
        let c_space = cell_at(&mut p, "| 1", 0, 1);
        let c_digit = cell_at(&mut p, "| 1", 0, 2);
        let bwd = cell_delta(c_pipe, c_space);
        let fwd = cell_delta(c_space, c_digit);
        assert!(
            bwd <= bound && fwd <= bound,
            "{world}: '| 1' seam(s) exceed the product's own accepted bar \
             ({bar:.2}px): bwd={:.2} fwd={:.2}",
            bwd / ps,
            fwd / ps
        );
        worst_glyphless = worst_glyphless.max(bwd / ps).max(fwd / ps);
        // THE MECHANISM CLAIM: a tied single space between two different
        // letters must now be (near-)SYMMETRIC — the hard pick could
        // never produce this (one side was always exactly 0, the other the
        // whole gap).
        sym_worst = sym_worst.max((bwd - fwd).abs() / ps);

        // The rest of the round's fixture list, each within the same bar.
        let fixtures: &[(&str, usize, usize)] = &[
            ("aaa", 2, 3),            // the headline case
            (" A", 0, 1),             // leading glyphless, column 0
            ("A  ", 0, 1),            // run of 2+, first seam
            ("A  ", 1, 2),            // run of 2+, interior seam
            ("A  ", 2, 3),            // run of 2+, tail -> EOL
            ("| 1  Capital |", 2, 3), // real table row
            ("| 1  Capital |", 4, 5),
            ("hi ", 1, 2), // line ending in a space
            ("hi ", 2, 3),
            ("xg", 1, 2), // descender adjacent to EOL
        ];
        for &(text, a, b) in fixtures {
            let ca = cell_at(&mut p, text, 0, a);
            let cb = cell_at(&mut p, text, 0, b);
            let d = cell_delta(ca, cb);
            assert!(
                d <= bound,
                "{world}: {text:?} col{a}->{b} exceeds the product's own \
                 accepted bar ({bar:.2}px): Δ={:.2}",
                d / ps
            );
            worst_glyphless = worst_glyphless.max(d / ps);
        }

        // A completely empty line, versus a fresh 'a' line's real ink height
        // — the synthetic arm, not neighbor-borrow, but still held to the
        // same bar since it is also a glyphless-anchor fallback value.
        let (_cy_a, h_a) = cell_at(&mut p, "a", 0, 0);
        let (_cy_e, h_e) = cell_at(&mut p, "", 0, 0);
        assert!(
            (h_a - h_e).abs() <= bound,
            "{world}: empty-line synthetic height exceeds the bar ({bar:.2}px): \
             a={h_a:.2} empty={h_e:.2}",
        );
    }

    assert!(
        sym_worst < bar * 0.5,
        "the '| 1' backward/forward seams must be substantially more symmetric \
         than a hard directional pick (worst asymmetry {sym_worst:.2}px vs bar {bar:.2}px)"
    );

    eprintln!(
        "glyph-to-glyph bar={bar:.2}px; worst glyphless seam={worst_glyphless:.2}px; \
         worst '| 1' bwd/fwd asymmetry={sym_worst:.2}px"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE INTERIOR-FLIP DEFECT: a run of 4+ glyphless columns flanked by two
/// DIFFERENT real letters. The nearest-distance-wins pick made
/// exactly ONE interior column switch from "nearest is the left letter" to
/// "nearest is the right letter" — a hard, single-column STEP from one
/// letter's full ink to the other's, with nothing between to soften it
/// (measured 6.5-7.0px on Bombora against this same fixture shape).
///
/// The blend fix makes every step in the run small: `t` changes by a fixed
/// increment column-to-column, so the interpolated box moves smoothly, and
/// NO single adjacent pair in the run may jump by more than any other pair —
/// there is no longer a privileged "flip" column at all.
#[test]
fn interior_run_between_two_different_letters_has_no_flip_step() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping interior_run_between_two_different_letters_has_no_flip_step: no wgpu adapter"
        );
        return;
    };
    // 'A' (capital), 5 glyphless spaces, 'y' (descender) — a run of 5
    // interior glyphless columns flanked by two different real letter
    // classes, the exact shape the interior-flip defect needs.
    let text = "A     y";
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        p.set_view(&view(text, 0, 0));
        p.settle_caret();
        let ps = pixel_scale(&p);

        let cols: Vec<(f32, f32)> = (0..=6).map(|c| cell_at(&mut p, text, 0, c)).collect();
        let steps: Vec<f32> = (0..cols.len() - 1)
            .map(|i| cell_delta(cols[i], cols[i + 1]) / ps)
            .collect();
        let max_step = steps.iter().cloned().fold(0.0f32, f32::max);
        let min_step = steps.iter().cloned().fold(f32::MAX, f32::min);

        // NO FLIP: every step in the run is small on its own terms — well
        // under the ligature-class wide bound, and critically no single step
        // dominates the others (a "flip" is exactly one step being many
        // times larger than its neighbors).
        assert!(
            max_step <= TRANSITION_BOUND_WIDE_PX,
            "{}: a step in the 'A     y' run exceeds the wide sanity bound \
             ({TRANSITION_BOUND_WIDE_PX}px): steps={steps:?}",
            t.name
        );
        assert!(
            max_step <= min_step * 4.0 + 0.5,
            "{}: one step in the run dominates the others (a flip) — \
             steps={steps:?} (max={max_step:.2} min={min_step:.2})",
            t.name
        );
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE GLYPH-CLASS ROSTER this item's own diagnosis names, as a closed enum.
/// [`class_role`] is the ONE no-wildcard match that assigns every variant a
/// role in the sweep below — a class added here without a role in that match
/// fails to compile, so a new class cannot silently dodge the law the way
/// hand-picked fixture lists could (one tested a single direction; another
/// tested a handful of ad hoc pairs; neither swept the
/// cross-product).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GlyphClass {
    XHeight,
    Ascender,
    Descender,
    Capital,
    Digit,
    Punctuation,
    Ligature,
    Space,
    Eol,
    EmptyLine,
}

const ALL_GLYPH_CLASSES: [GlyphClass; 10] = [
    GlyphClass::XHeight,
    GlyphClass::Ascender,
    GlyphClass::Descender,
    GlyphClass::Capital,
    GlyphClass::Digit,
    GlyphClass::Punctuation,
    GlyphClass::Ligature,
    GlyphClass::Space,
    GlyphClass::Eol,
    GlyphClass::EmptyLine,
];

/// Seven classes anchor REAL ink at a literal token; the other three are
/// STRUCTURAL (a glyphless separator, the end-of-line position, and the
/// whole-empty-line case) and get their own dedicated sweep phases instead of
/// pairwise concatenation. NO WILDCARD: every one of the 10 variants above
/// must appear on the left of an arm here, or the match fails to compile.
enum ClassRole {
    Endpoint(&'static str),
    Space,
    Eol,
    EmptyLine,
}

fn class_role(c: GlyphClass) -> ClassRole {
    match c {
        GlyphClass::XHeight => ClassRole::Endpoint("a"),
        GlyphClass::Ascender => ClassRole::Endpoint("l"),
        GlyphClass::Descender => ClassRole::Endpoint("g"),
        GlyphClass::Capital => ClassRole::Endpoint("A"),
        GlyphClass::Digit => ClassRole::Endpoint("1"),
        GlyphClass::Punctuation => ClassRole::Endpoint("."),
        GlyphClass::Ligature => ClassRole::Endpoint("fi"),
        GlyphClass::Space => ClassRole::Space,
        GlyphClass::Eol => ClassRole::Eol,
        GlyphClass::EmptyLine => ClassRole::EmptyLine,
    }
}

/// The 7 [`ClassRole::Endpoint`] classes, derived from [`class_role`] rather
/// than hand-listed — adding an 8th endpoint class to the enum grows this
/// automatically; the compile-time exhaustiveness lives in `class_role`
/// itself.
fn endpoint_classes() -> Vec<(GlyphClass, &'static str)> {
    ALL_GLYPH_CLASSES
        .iter()
        .filter_map(|&c| match class_role(c) {
            ClassRole::Endpoint(tok) => Some((c, tok)),
            ClassRole::Space | ClassRole::Eol | ClassRole::EmptyLine => None,
        })
        .collect()
}

/// THE MISSING SWEEP LAW. Every fixture rounds 1–3 wrote had a real glyph on
/// at most ONE side of the transition it measured (a literal-adjacency
/// closure, or a run flanked by copies of the SAME letter) — a shape that is
/// structurally INCAPABLE of exposing a directional asymmetry, because there
/// is only one direction to have an asymmetry IN. The open
/// defect needed a glyphless column with TWO DIFFERENT real letters, one on
/// each side, while earlier laws swept at most one hand-picked instance of
/// that shape (the "| 1" fixture: pipe and
/// digit, and nothing else).
///
/// This law sweeps the ORDERED PAIR cross-product of the full 10-class
/// roster this item's diagnosis names — every one of the 7 [`ClassRole::Endpoint`]
/// classes (x-height, ascender, descender, capital, digit, punctuation,
/// ligature) against every OTHER one, tied through a single glyphless SPACE,
/// on EVERY proportional world, in BOTH directions: `"{A} {B}"` and
/// `"{B} {A}"` are different fixtures, because the space's BACKWARD seam
/// borrows from whichever letter sits behind it and its FORWARD seam borrows
/// from whichever sits ahead, and the hard "nearest wins" pick made
/// those two seams structurally UNEQUAL (one always ~0, the other the whole
/// gap) — a claim about DIRECTION, not magnitude, which a bound-only check
/// cannot catch even at a generous bound (the hard pick's worst-case single-seam
/// magnitude, ~11.85px, still clears a ~14px bar) — see the SYMMETRY assert
/// below, which is the actual load-bearing check this law adds.
///
/// THE BAR is measured fresh per world (not assumed), from each of the 7
/// endpoint classes' own ISOLATED real-ink cell (no fallback, no neighbor —
/// a plain [`TextPipeline::caret_anchor_raster_box`] read), pairwise-diffed —
/// the same "glyph-to-glyph transitions nobody has filed as a bug" measurement
/// interpolation testing introduced, generalized from one hand-picked pair to the full
/// 7-class roster and computed once per world instead of re-shaped per pair.
///
/// Phases 2–4 place the three STRUCTURAL classes (`Eol`, `EmptyLine`,
/// and `Space` as an endpoint in its own right — leading and trailing) in the
/// same sweep, so all 10 roster classes are actually exercised, not merely
/// declared. The MONO complement's uniform grid closes the loop:
/// every one of the same ordered pairs must read EXACTLY zero there, since a
/// mono world never leaves the line-cell arm at all.
///
/// NON-VACUITY (see the orchestrator's own required proof, reproduced in the
/// commit message / task report): this law is RED on the hard-pick implementation
/// hard-pick code) via the symmetry assert — at least one ordered pair's
/// backward/forward seams differ by more than half the bar. It is RED on
/// the former baseline via the per-seam bound assert on the
/// `"{X-height} {Eol-adjacent...}"`-style fixtures reproducing the original
/// `aaa`→EOL class of jump. It is GREEN on this round's landed blend.
#[test]
fn ordered_class_pair_transitions_stay_within_the_measured_bar_both_directions() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping ordered_class_pair_transitions_stay_within_the_measured_bar_both_directions: no wgpu adapter"
        );
        return;
    };

    let mono = super::facepitch::mono_display_worlds();
    let prop: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|t| !mono.contains(&t.name))
        .map(|t| t.name)
        .collect();
    assert!(
        prop.len() >= 11,
        "full proportional roster is swept (got {})",
        prop.len()
    );
    assert!(
        mono.len() >= 7,
        "full mono roster is swept (got {})",
        mono.len()
    );

    let endpoints = endpoint_classes();
    assert_eq!(
        endpoints.len(),
        7,
        "7 endpoint classes expected (10-class roster minus space/eol/empty-line)"
    );

    let mut global_worst = 0.0f32;
    let mut global_worst_desc = String::new();
    let mut global_worst_asym = 0.0f32;
    let mut global_worst_asym_desc = String::new();

    for &world in &prop {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let ps = {
            p.set_view(&view("a", 0, 0));
            pixel_scale(&p)
        };

        let bar = endpoint_bar(&mut p, world, &endpoints, ps);
        let (worst, worst_desc, asym, asym_desc) =
            assert_ordered_endpoint_pairs(&mut p, world, &endpoints, ps, bar);
        if worst > global_worst {
            (global_worst, global_worst_desc) = (worst, worst_desc);
        }
        if asym > global_worst_asym {
            (global_worst_asym, global_worst_asym_desc) = (asym, asym_desc);
        }
        assert_structural_classes(&mut p, world, &endpoints, ps, bar);
    }

    eprintln!(
        "ordered_class_pair sweep: worst seam={global_worst:.2}px ({global_worst_desc}); \
         worst bwd/fwd asymmetry={global_worst_asym:.2}px ({global_worst_asym_desc})"
    );

    assert_mono_glyphless_spaces(&mut p, &mono, &endpoints);

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

fn endpoint_bar(
    p: &mut TextPipeline,
    world: &str,
    endpoints: &[(GlyphClass, &'static str)],
    ps: f32,
) -> f32 {
    let solo: Vec<(GlyphClass, (f32, f32))> = endpoints
        .iter()
        .map(|&(class, token)| {
            let cell = cell_at(p, token, 0, token.chars().count() - 1);
            assert!(
                p.caret_anchor_raster_box().is_some(),
                "{world}: {class:?} ({token:?}) must anchor real raster ink"
            );
            (class, cell)
        })
        .collect();
    let bar = solo
        .iter()
        .flat_map(|a| solo.iter().map(move |b| cell_delta(a.1, b.1) / ps))
        .fold(0.0, f32::max);
    assert!(bar > 5.0, "{world}: glyph bar must be real ({bar:.2}px)");
    bar
}

fn assert_ordered_endpoint_pairs(
    p: &mut TextPipeline,
    world: &str,
    endpoints: &[(GlyphClass, &'static str)],
    ps: f32,
    bar: f32,
) -> (f32, String, f32, String) {
    let mut worst = (0.0, String::new());
    let mut asym_worst = (0.0, String::new());
    for &(left_class, left) in endpoints {
        for &(right_class, right) in endpoints {
            if left_class == right_class {
                continue;
            }
            let text = format!("{left} {right}");
            let middle = left.chars().count();
            let before = cell_at(p, &text, 0, middle - 1);
            let middle_cell = cell_at(p, &text, 0, middle);
            assert!(
                p.caret_anchor_ink_box().is_none(),
                "{world}: {text:?} middle must be glyphless"
            );
            let after = cell_at(p, &text, 0, middle + right.chars().count());
            let back = cell_delta(before, middle_cell) / ps;
            let forward = cell_delta(middle_cell, after) / ps;
            assert!(
                back <= bar && forward <= bar,
                "{world}: {text:?} exceeds {bar:.2}px: {back:.2}/{forward:.2}"
            );
            let desc = format!("{world} {left_class:?}<->{right_class:?}");
            if back.max(forward) > worst.0 {
                worst = (back.max(forward), desc.clone());
            }
            let asym = (back - forward).abs();
            assert!(
                asym <= bar * 0.5,
                "{desc} tied space is directional: {back:.2}/{forward:.2}"
            );
            if asym > asym_worst.0 {
                asym_worst = (asym, desc);
            }
        }
    }
    (worst.0, worst.1, asym_worst.0, asym_worst.1)
}

fn assert_structural_classes(
    p: &mut TextPipeline,
    world: &str,
    endpoints: &[(GlyphClass, &'static str)],
    ps: f32,
    bar: f32,
) {
    let context = TransitionContext {
        world,
        class: GlyphClass::XHeight,
        ps,
        bar,
    };
    for &(class, token) in endpoints {
        let end = token.chars().count();
        assert_transition_at(p, context.with_class(class), token, end - 1, end, "Eol");
        let leading = format!(" {token}");
        assert_transition_at(
            p,
            context.with_class(class),
            &leading,
            0,
            1,
            "leading Space",
        );
        let trailing = format!("{token} ");
        assert_transition_at(
            p,
            context.with_class(class),
            &trailing,
            end - 1,
            end,
            "trailing Space",
        );
    }
    let (_, real_h) = cell_at(p, "a", 0, 0);
    let (_, empty_h) = cell_at(p, "", 0, 0);
    let delta = (real_h - empty_h).abs() / ps;
    let bound = bar.max(TRANSITION_BOUND_WIDE_PX);
    assert!(
        delta <= bound,
        "{world}: EmptyLine exceeds {bound:.2}px: {delta:.2}"
    );
}

#[derive(Clone, Copy)]
struct TransitionContext<'a> {
    world: &'a str,
    class: GlyphClass,
    ps: f32,
    bar: f32,
}

impl TransitionContext<'_> {
    fn with_class(self, class: GlyphClass) -> Self {
        Self { class, ..self }
    }
}

fn assert_transition_at(
    p: &mut TextPipeline,
    context: TransitionContext<'_>,
    text: &str,
    from: usize,
    to: usize,
    shape: &str,
) {
    let delta = cell_delta(cell_at(p, text, 0, from), cell_at(p, text, 0, to)) / context.ps;
    assert!(
        delta <= context.bar,
        "{}: {:?} {shape} exceeds {:.2}px: {delta:.2}",
        context.world,
        context.class,
        context.bar
    );
}

/// The mono complement of the proportional neighbor-borrow sweep: a
/// glyphless cell cannot read either adjacent glyph on a uniform grid.
fn assert_mono_glyphless_spaces(
    p: &mut TextPipeline,
    mono: &[&str],
    endpoints: &[(GlyphClass, &'static str)],
) {
    for &world in mono {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let reference = cell_at(p, " a", 0, 0);
        for &(ca, ta) in endpoints {
            for &(cb, tb) in endpoints {
                if ca == cb {
                    continue;
                }
                let text = format!("{ta} {tb}");
                let c_mid = cell_at(p, &text, 0, ta.chars().count());
                assert!(
                    cell_delta(c_mid, reference) < 1e-3,
                    "{world}: mono glyphless column tied between {ca:?} and {cb:?} ({text:?}) \
                     must match the reference space cell exactly: got {c_mid:?} want {reference:?}"
                );
            }
        }
    }
}

/// THE ALL-BLANK WRAPPED ROW (adjudicated 2026-07-26). Two independent audit
/// passes each measured the ABSOLUTE caret position stepping across a run of
/// spaces long enough to fill an entire wrapped visual row by itself —
/// flanked above and below by real-ink rows of the SAME logical line — and
/// reported a ~30px jump as a HIGH-severity defect. It is not one: this law
/// is the standing, non-vacuous refutation, so this exact shape cannot be
/// re-reported a fifth time without checking here first.
///
/// THE MECHANISM (real, and BY DESIGN, not a bug). `nearest_row_raster_box`
/// is deliberately bounded to the caret's OWN visual row — an explicit
/// O(row-width)-not-O(doc) perf tradeoff, see that function's own doc. On a
/// row that is 100% blank, both the backward and forward search come up
/// empty, so `caret_cell_vertical`'s fallback arm drops to the SYNTHETIC
/// tier-3 box (`caret_synthetic_ink_box`), anchored to THAT row's own
/// baseline — a genuinely different baseline than the row immediately before
/// or after it.
///
/// THE WRAP-BOUNDARY TRAP — why this law is ROW-RELATIVE, never absolute.
/// Crossing a soft-wrap boundary moves the caret down exactly one row's
/// `line_height` and back toward the left margin: correct, INTENDED
/// behaviour (the caret is supposed to drop a visual row), not a defect. An
/// ABSOLUTE position delta cannot tell that apart from a genuine
/// cell-geometry discontinuity, because it is measuring two different rows
/// against a single shared origin that neither row is drawn relative to.
/// Measured directly below, on the reported fixture, over the full
/// proportional roster: the absolute Δcy at the worst row boundary is a flat
/// 32.00px on EVERY world (exactly this fixture's `line_height` — the row
/// pitch, not a defect magnitude), while the ROW-RELATIVE residual (`cy`
/// minus THAT column's own [`TextPipeline::caret_baseline_y`]) — the only
/// quantity two adjacently-drawn rows can actually be judged against, since
/// they are never drawn at the same y — never exceeds 5.85px anywhere in the
/// sweep (Bilby, worst; Gumtree measures 5.50px, matching the two audit
/// reports' own literal fixture almost exactly). The absolute figure the two
/// reports quoted (~30px) is ~84% pure row-pitch; the genuine tier-2/tier-3
/// boundary residual is the 5.85px figure, comfortably inside
/// [`TRANSITION_BOUND_WIDE_PX`] (7.5px) and well under the file's own
/// empirically-measured 14px glyph-to-glyph worst case
/// (`glyphless_seams_stay_within_the_products_own_accepted_glyph_to_glyph_bar`'s
/// bar).
///
/// SWEEPS every adjacent column pair across the WHOLE fixture line (not one
/// hand-picked seam), on every proportional world — a superset of the exact
/// enter/exit seams (real-ink row → all-blank row, and back) the two reports
/// named, found automatically rather than hand-indexed, so a future
/// unrelated tier-boundary regression anywhere on this line's wrap also
/// trips this law.
///
/// NON-VACUITY, both halves in one sweep, per world: the ABSOLUTE delta
/// really is large (≥15px, well above `TRANSITION_BOUND_WIDE_PX` — proving
/// the fixture genuinely reproduces the reported wrap-pitch jump, not some
/// unrelated small motion) AND the ROW-RELATIVE residual clears comfortably
/// under the bound at the SAME column — so a regression that made the
/// row-relative geometry itself jump (not just the wrap pitch) would turn
/// this law red without touching the floor.
#[test]
fn glyphless_row_transition_is_bounded_row_relatively_across_a_wrap() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping glyphless_row_transition_is_bounded_row_relatively_across_a_wrap: no wgpu adapter"
        );
        return;
    };
    // 20 real 'a's, a run of 30 spaces long enough to fill an entire wrapped
    // row by itself at this window width, 20 real 'b's — one logical line;
    // the two audit reports' own literal fixture shape.
    let a_run = 20usize;
    let space_run = 30usize;
    let b_run = 20usize;
    let text = format!(
        "{}{}{}",
        "a".repeat(a_run),
        " ".repeat(space_run),
        "b".repeat(b_run)
    );
    let blank_start = a_run;
    let blank_end = a_run + space_run;
    let n = text.chars().count();

    // Narrow enough that the space run cannot share a row with the 'a's or
    // the 'b's on any bundled proportional face — forces at least one row to
    // be ENTIRELY blank (verified below per world, never assumed).
    p.set_size(260.0, 800.0);

    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;
    let mut worst_rowrel = 0.0f32;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        p.set_view(&view(&text, 0, 0));

        // FIXTURE PRECONDITION: the fixture must actually reproduce the
        // reported shape — a wrapped visual row entirely inside the space
        // run, on THIS world's own real glyph metrics.
        let rows = p.visual_rows(0);
        assert!(
            rows.iter()
                .any(|r| r.start_col >= blank_start && r.end_col <= blank_end),
            "{}: fixture must wrap a fully-blank row at this width ({} rows: {:?})",
            t.name,
            rows.len(),
            rows.iter()
                .map(|r| (r.start_col, r.end_col))
                .collect::<Vec<_>>()
        );

        // Sweep every adjacent column pair across the whole line: the
        // absolute cell position, and the ROW-RELATIVE residual (this
        // column's cy minus ITS OWN row's baseline — the quantity that
        // strips out the wrap's row-pitch component and isolates the
        // tier-2/tier-3 boundary's own claim).
        let ps = pixel_scale(&p);
        let mut cy = Vec::with_capacity(n + 1);
        let mut h = Vec::with_capacity(n + 1);
        let mut residual = Vec::with_capacity(n + 1);
        for col in 0..=n {
            p.set_view(&view(&text, 0, col));
            p.settle_caret();
            let (c, ht) = p.caret_cell_vertical();
            let baseline = p.caret_baseline_y();
            cy.push(c);
            h.push(ht);
            residual.push(c - baseline);
        }

        let mut max_abs = 0.0f32;
        let mut max_rowrel = 0.0f32;
        for i in 0..n {
            let d_abs = (cy[i + 1] - cy[i]).abs() / ps;
            let d_rowrel = (residual[i + 1] - residual[i])
                .abs()
                .max((h[i + 1] - h[i]).abs())
                / ps;
            max_abs = max_abs.max(d_abs);
            max_rowrel = max_rowrel.max(d_rowrel);
        }

        // NON-VACUITY: the fixture must genuinely reproduce a LARGE absolute
        // jump somewhere (the wrap's row-pitch component the two audit
        // reports measured) — or the row-relative claim below proves nothing
        // about the trap this law names.
        const ABS_JUMP_FLOOR_PX: f32 = 15.0;
        assert!(
            max_abs >= ABS_JUMP_FLOOR_PX,
            "{}: fixture must reproduce a large absolute row-boundary jump \
             (got {max_abs:.2}px, floor {ABS_JUMP_FLOOR_PX}px) or this law's \
             non-vacuity claim is empty",
            t.name
        );

        // THE LAW: the ROW-RELATIVE residual — the tier-2/tier-3 boundary's
        // actual geometric claim, with the wrap's row-pitch subtracted out —
        // stays inside the file's own wide sanity bound, even though the
        // ABSOLUTE delta at the same seam is far larger.
        assert!(
            max_rowrel <= TRANSITION_BOUND_WIDE_PX,
            "{}: row-relative cell discontinuity across the all-blank wrapped \
             row exceeds the wide bound ({TRANSITION_BOUND_WIDE_PX}px): \
             max_rowrel={max_rowrel:.2}px (absolute max at the same seam was \
             {max_abs:.2}px — a wrap-pitch artifact, not part of this claim)",
            t.name
        );

        worst_rowrel = worst_rowrel.max(max_rowrel);
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );
    eprintln!(
        "glyphless_row_transition_is_bounded_row_relatively_across_a_wrap: \
         worst row-relative residual={worst_rowrel:.2}px (bound {TRANSITION_BOUND_WIDE_PX}px)"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}
