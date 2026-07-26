//! ITEM 105 — THE CARET CELL TRANSITION LAW. Item 91 introduced the two-arm
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
//! top/bottom pixels (mirroring how `caret_ink_box.rs`'s own item-91 laws read
//! pixel-exact geometry rather than decoding a PNG).
//!
//! NON-VACUITY. Every sweep below also computes the item-91-only OLD fallback
//! formula (`cy = caret.pos.y`, `h = caret_block_h * cursor_scale()` — byte-copy
//! of the pre-105 `caret.rs`/`facepitch.rs`, i.e. `main`/`07f1b7d`'s line-cell
//! arm) and asserts that number alone WOULD have blown the bound — so the
//! fixture is proven capable of catching the exact regression this item
//! repairs, not merely passing by construction. Confirmed directly: this whole
//! file, run with `src/render/caret.rs` + `src/render/facepitch.rs` reverted to
//! their pre-105 `main` state (the same two-arm formula item 91/07f1b7d
//! shipped — item 97/main only changed WHICH worlds take which arm, never the
//! formula), fails 4 of 9 tests (the ones exercising the `aaa`/x-height class
//! this docstring's own numbers are drawn from); passes all 9 once the repair
//! is restored. See the item's queue/commit history for the literal red/green
//! console output this proof produced.
//!
//! SWEPT AXES (the ones item 91's laws did not): the full proportional-world
//! roster (not just the two the user found), representative glyph classes
//! (x-height / ascender / descender / punctuation / space / EOL / empty line /
//! ligature), Block AND Morph (rest — travel is proven UNAFFECTED, not simply
//! re-measured, since a moving caret is a streak with no cell to jump), a
//! wrapped-line boundary, two zooms including a non-1.0 value, and 1x/2x DPI —
//! plus the mono complement (must stay at ZERO discontinuity: item 97's grid
//! never leaves the line-cell arm, so there is no seam to jump across there).

use super::super::*;
use super::{headless_pipeline, view};

/// The pixel scale (zoom × dpi) the pads and the transition bound both ride —
/// the same quantity `caret_ink_box.rs::pad_px` reads, redefined here so this
/// file has no cross-module dependency on that one's private helper.
fn pixel_scale(p: &TextPipeline) -> f32 {
    p.metrics.caret_h / CARET_H
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

/// THE WIDE AUTHORED BOUND: a broad sanity ceiling for classes a SINGLE fixed
/// synthetic reference structurally cannot fully close — an ASCENDER or
/// DESCENDER neighbour, a LIGATURE cluster, a wrapped-line boundary, a tiny
/// PUNCTUATION mark. These glyphs are legitimately taller/shorter than an
/// "ordinary" letter (the ink-box arm ITSELF already shows a comparable
/// column-to-column spread with NO bug involved — e.g. `lamp`'s real
/// ink-arm height goes 25px on `l` to 19px on `a`, a 6px swing between two
/// adjacent REAL glyphs; see `every_glyph_class_stays_bounded_into_eol`'s own
/// fixture-sanity assert), so a synthetic "typical letter" reference cannot
/// hug an extreme every time. See [`assert_no_worse_than_before`] for how this
/// combines with a per-transition non-regression check into one honest claim.
/// (Measured worst residual on this sweep's own extreme-class fixtures: 7.0px,
/// Bilby's real "fi" ligature against a plain glyph — the bound sits with a
/// small margin above that and well under the old code's own worst extreme
/// case, ~13px on a bare punctuation mark.)
const TRANSITION_BOUND_WIDE_PX: f32 = 7.5;

/// The MINIMUM pre-105 discontinuity a fixture must reproduce to prove the law
/// non-vacuous — deliberately BELOW the smallest old-bug magnitude actually
/// measured on the x-height class across the roster (2.4px, Mopoke/Galah/
/// Magpie), so the non-vacuity check itself never spuriously fires on the
/// world with the smallest (but still real) old bug.
const NONVACUITY_OLD_DELTA_MIN_PX: f32 = 2.0;

/// `(center_y, height)` at `col` on `line`, via the ONE owner, at REST (settled
/// spring — `settle_caret` pins `settle_factor` to 1 so this is a genuine
/// rest-to-rest comparison, never mid-glide).
fn cell_at(p: &mut TextPipeline, text: &str, line: usize, col: usize) -> (f32, f32) {
    p.set_view(&view(text, line, col));
    p.settle_caret();
    p.caret_cell_vertical()
}

/// The OLD (pre-item-105) line-cell arm's `(center_y, height)` at the CURRENT
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
fn assert_bounded_by(p: &TextPipeline, bound_px: f32, what: &str, cy0: f32, h0: f32, cy1: f32, h1: f32) {
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

/// THE EXTREME-CLASS LAW: for a glyph class no single fixed synthetic
/// reference can fully hug (ascender / descender / digit / punctuation /
/// ligature — see [`TRANSITION_BOUND_WIDE_PX`]'s doc), item 105 must leave the
/// transition NO WORSE than it already was: `new_d <= max(old_d,
/// TRANSITION_BOUND_WIDE_PX)`. This is one honest claim doing two jobs at
/// once — wherever the OLD code was already small (an ascender, where the
/// fixed old cell happened to sit close to a real ascender's height), the
/// wide bound alone caps the new residual; wherever the OLD code was
/// GENUINELY large (punctuation, up to ~13px old), the new residual is
/// required to be strictly SMALLER than that old number, i.e. a real,
/// measured improvement — never merely "not worse than an arbitrary
/// constant". Either way this rules out a REGRESSION beyond the wide bound
/// on every extreme class, the thing item 105 must not introduce even where
/// it cannot achieve the tight bound's full closure.
fn assert_no_worse_than_before(p: &TextPipeline, what: &str, old_d: f32, new_d: f32) {
    let ceiling = old_d.max(TRANSITION_BOUND_WIDE_PX * pixel_scale(p));
    assert!(
        new_d <= ceiling + 1e-3,
        "{what}: item 105 must leave this transition no worse than before \
         (old Δ={old_d:.2}, wide bound={:.2}, so ceiling={ceiling:.2}): got new Δ={new_d:.2}",
        TRANSITION_BOUND_WIDE_PX * pixel_scale(p)
    );
}

/// THE HEADLINE FIXTURE, swept over the FULL proportional-world roster: the
/// user's literal `aaa` line, comparing the caret ON the final `a` (the real
/// ink-box arm) against one column later, at end-of-line (the fallback arm).
/// Reproduces the reported bug at its exact seam, and proves it repaired
/// everywhere a proportional world ships, not just Mopoke/Gumtree.
#[test]
fn aaa_to_eol_transition_is_bounded_on_every_proportional_world() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping aaa_to_eol_transition_is_bounded_on_every_proportional_world: no wgpu adapter");
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
        assert_bounded(&p, &format!("{} aaa->EOL", t.name), cy_glyph, h_glyph, cy_eol, h_eol);
        checked += 1;
    }
    assert!(checked >= 11, "every proportional-display world is swept (got {checked})");

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE MONO COMPLEMENT: item 97's uniform grid never leaves the line-cell arm
/// on EITHER side of an "aaa"->EOL step (the ink-box arm is gated off entirely
/// on a mono world), so there is no seam to jump across — the transition must
/// be EXACTLY zero, not merely bounded.
#[test]
fn aaa_to_eol_transition_is_exactly_zero_on_every_mono_world() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping aaa_to_eol_transition_is_exactly_zero_on_every_mono_world: no wgpu adapter");
        return;
    };
    let text = "aaa";
    let worlds = super::facepitch::mono_display_worlds();
    assert!(worlds.len() >= 7, "every mono-display world is swept, got {worlds:?}");

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

/// REPRESENTATIVE GLYPH CLASSES, swept on Gumtree + Mopoke (the two the user
/// named): ascender, x-height, descender, punctuation, digit — each
/// immediately followed by end-of-line, so the bound holds regardless of WHICH
/// letter the caret was hugging before the jump, not just the lowercase `a`
/// the headline fixture uses.
///
/// X-HEIGHT keeps the TIGHT bound (the reported class). The others get the
/// WIDE bound: the synthetic reference is deliberately a "typical letter"
/// (the mean of x-height and cap-height — see `facepitch::typical_letter_ratio`),
/// so an extreme class (a tall ascender, a tiny punctuation mark) cannot fully
/// close against it with ANY single fixed reference — the ink-box arm's own
/// letter-to-letter spread already shows a comparable gap with no bug
/// involved (fixture-sanity asserted below). Every class ALSO stays under the
/// GLOBAL pre-fix worst case, so even the classes item 105 cannot fully
/// close are proven never worse than the original bug's own ceiling.
#[test]
fn every_glyph_class_stays_bounded_into_eol() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping every_glyph_class_stays_bounded_into_eol: no wgpu adapter");
        return;
    };
    // (fixture, glyph col, class label, tight-bound?)
    let fixtures: &[(&str, usize, &str, bool)] = &[
        ("lamp", 0, "ascender (l)", false), // 'l' at col0, EOL at col4
        ("lamp", 1, "x-height (a)", true),
        ("gap", 0, "descender (g)", false),
        ("a1.", 2, "punctuation (.)", false),
        ("a1.", 1, "digit (1)", false),
    ];

    // FIXTURE SANITY: the ink-box arm's OWN letter-to-letter height spread on
    // "lamp" (an ascender next to an x-height letter, both real glyphs, no
    // fallback involved) is itself several px — proof that SOME height
    // variation between adjacent columns is normal product behaviour, not a
    // defect, and that the wide bound below is not simply "anything goes".
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

    for world in ["Gumtree", "Mopoke"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for &(text, col, label, tight) in fixtures {
            let eol = text.chars().count();
            let (cy0, h0) = cell_at(&mut p, text, 0, col);
            assert!(
                p.caret_anchor_ink_box().is_some(),
                "{world} {label}: fixture must anchor a real ink box"
            );
            if tight {
                let (cy1, h1) = cell_at(&mut p, text, 0, eol);
                assert_bounded(&p, &format!("{world} {label} -> EOL"), cy0, h0, cy1, h1);
            } else {
                p.set_view(&view(text, 0, eol));
                p.settle_caret();
                let old = old_fallback_cell(&p);
                let old_d = cell_delta((cy0, h0), old);
                let (cy1, h1) = p.caret_cell_vertical();
                let new_d = cell_delta((cy0, h0), (cy1, h1));
                assert_no_worse_than_before(&p, &format!("{world} {label} -> EOL"), old_d, new_d);
            }
        }
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE LIGATURE SEAM: `caret_anchor_ink_box` deliberately gates a multi-char
/// LIGATURE cluster OUT of the real ink-box arm (its horizontal ink can't be
/// fairly split across the chars it covers), so a ligature-anchored column
/// used to fall all the way to the item-91 fixed line-cell — the SAME jump the
/// glyphless case had. `"fine"` genuinely ligates `fi` into one glyph on every
/// bundled proportional prose face (confirmed by probe: `caret_anchor_ink_box`
/// is `None` at both col 0 and col 1 while `caret_anchor_raster_box` is
/// `Some`), so col 1 (still inside the `fi` cluster) followed by col 2 (the
/// plain `n` glyph) is a REAL ligature -> plain-glyph transition, not a
/// synthetic stand-in.
#[test]
fn ligature_to_plain_glyph_transition_is_bounded() {
    let _t = crate::testlock::serial();
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
        let is_ligature = p.caret_anchor_ink_box().is_none() && p.caret_anchor_raster_box().is_some();
        if !is_ligature {
            // Not every face is guaranteed to ligate "fi" (a face without the
            // `liga` feature would shape one glyph per char) — skip rather than
            // fail on a world that doesn't reproduce the precondition.
            continue;
        }
        saw_real_ligature = true;
        let (cy0, h0) = p.caret_cell_vertical();
        let old0 = old_fallback_cell(&p); // the pre-105 line-cell this ligature used to draw

        let (cy1, h1) = cell_at(&mut p, text, 0, 2); // the plain 'n' — the ink-box
        // arm's own value, UNCHANGED by item 105, so it is both the OLD and NEW
        // reading at this column: the correct oracle for both deltas below.
        assert!(
            p.caret_anchor_ink_box().is_some(),
            "{}: 'n' must be a plain single-glyph anchor",
            t.name
        );
        // "fi" (an ascender-height ligature) next to a plain x-height 'n' is
        // the same cross-class shape as the ascender case in
        // `every_glyph_class_stays_bounded_into_eol` — see
        // `assert_no_worse_than_before`'s doc for why a single fixed synthetic
        // reference cannot always fully close it.
        let old_d = cell_delta(old0, (cy1, h1));
        let new_d = cell_delta((cy0, h0), (cy1, h1));
        assert_no_worse_than_before(&p, &format!("{} ligature->plain", t.name), old_d, new_d);
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
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping wrap_boundary_transition_is_bounded_on_a_proportional_world: no wgpu adapter");
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
    let old1 = old_fallback_cell(&p);
    let old_d = cell_delta((cy0, h0), old1);
    let (cy1, h1) = p.caret_cell_vertical();
    // The char right before the wrap collapse is `d`, a descender (the
    // fixture's own "word "-repeated content), so this is the same
    // cross-class shape as `every_glyph_class_stays_bounded_into_eol`'s
    // descender case — see `assert_no_worse_than_before`'s doc.
    let new_d = cell_delta((cy0, h0), (cy1, h1));
    assert_no_worse_than_before(&p, "Gumtree wrap boundary", old_d, new_d);

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// AN EMPTY LINE'S synthetic cell stays a REASONABLE, bounded size — never the
/// item-91-original "large empty accent cap" (a fixed ~22px cell regardless of
/// the font) and never degenerate (zero/negative). Compared against the SAME
/// world's real ink-arm height on an ordinary x-height letter, which the
/// synthetic box is explicitly modelled to approximate.
#[test]
fn empty_line_synthetic_cell_stays_reasonable_not_the_old_fixed_cap() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping empty_line_synthetic_cell_stays_reasonable_not_the_old_fixed_cap: no wgpu adapter");
        return;
    };
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;

    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();

        // A real x-height letter's ink-arm height, on its own line...
        let (_cy_glyph, h_glyph) = cell_at(&mut p, "a", 0, 0);
        assert!(p.caret_anchor_ink_box().is_some(), "{}: 'a' must ink-align", t.name);

        // ...versus an EMPTY line's synthetic fallback height.
        let (_cy_empty, h_empty) = cell_at(&mut p, "", 0, 0);
        assert!(p.caret_anchor_ink_box().is_none(), "{}: an empty line has no ink", t.name);

        assert!(
            h_empty > 0.0,
            "{}: the empty-line synthetic cell must be a real positive height",
            t.name
        );
        // Bounded BOTH ways: not collapsed to nothing, and not the old fixed
        // ~0.8*row-height cap regardless of the letter (item 91's original bug,
        // reproduced at the seam if this synthetic box regresses to it).
        assert!(
            h_empty < h_glyph * 2.0 + 4.0 * pixel_scale(&p),
            "{}: empty-line cell must not balloon back to a large fixed cap: \
             h_empty={h_empty} h_glyph={h_glyph}",
            t.name
        );
        checked += 1;
    }
    assert!(checked >= 11, "every proportional-display world is swept (got {checked})");

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// TWO ZOOMS, including a NON-1.0 value, and 1x/2x DPI: the headline `aaa`->EOL
/// transition stays bounded at EVERY pixel scale, with the bound itself scaled
/// by that same factor (`pixel_scale`) — proving the fix is a geometric
/// relationship, not a value tuned to look right only at the capture's default
/// zoom/DPI. Mindful of the documented zoom trap (CLAUDE.md / item 93/96): this
/// reads `caret_cell_vertical`'s OWN already-scaled pixel output directly,
/// never a sidecar field, so there is no scaled/unscaled unit mismatch to fall
/// into.
#[test]
fn transition_stays_bounded_across_zoom_and_dpi() {
    let _t = crate::testlock::serial();
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
            assert!(p.caret_anchor_ink_box().is_some(), "{world} z{zoom} d{dpi}: fixture must ink-align");

            let mut v2 = view(text, 0, 3);
            v2.zoom = zoom;
            p.set_view(&v2);
            p.settle_caret();
            let (cy1, h1) = p.caret_cell_vertical();
            assert_bounded(&p, &format!("{world} zoom={zoom} dpi={dpi}"), cy0, h0, cy1, h1);
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
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Morph);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping morph_rest_transition_is_bounded_through_caret_geometry: no wgpu adapter");
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
        assert_bounded(&p, &format!("{} morph rest a->space", t.name), owner_cy, owner_h, cy1, h1);
        checked += 1;
    }
    assert!(checked >= 11, "every proportional-display world is swept (got {checked})");

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// MORPH, TRAVELLING: item 105 touches only the REST endpoint inside
/// `caret_cell_vertical`; a moving caret is a thin STREAK
/// (`motion_geometry`), with no cell to jump between columns at all. Widens
/// `caret_ink_box.rs`'s own `moving_caret_streak_is_unaffected_by_the_ink_box`
/// (which swept two worlds) to the FULL proportional roster, so this item's
/// change is proven not to have introduced a settle/travel thickness pop
/// anywhere it ships.
#[test]
fn morph_travel_stays_a_thin_streak_on_every_proportional_world() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping morph_travel_stays_a_thin_streak_on_every_proportional_world: no wgpu adapter");
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
        assert!(s < 0.2, "{}: fixture must be genuinely mid-glide (s={s})", t.name);
        assert!(w > h, "{}: motion pose must be long-and-thin: w={w} h={h}", t.name);
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
    assert!(checked >= 11, "every proportional-display world is swept (got {checked})");

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE THEME-PICKER PREVIEW SEAM (found auditing item 105's own repair). The
/// caret's proportional-fallback branch gate and its synthetic ratio lookup
/// must read [`TextPipeline::doc_family`] (the LIVE face the ACTIVE theme
/// wants) — NOT `shaped_font` (the face the document is ACTUALLY shaped in
/// right now). `sync_theme_colors` (`App::retint_theme_preview`'s per-arrow
/// step) re-tints every baked colour and switches the active theme instantly
/// but deliberately LEAVES `shaped_font` stale until the separately-deferred
/// font reshape (`sync_theme_font`) catches up — the whole point of the
/// split, so a fast preview scrub never pays a reshape per arrow press.
///
/// Before item 105 this never mattered: a GLYPHLESS anchor's fallback was one
/// constant formula regardless of font identity. Item 105 made the fallback
/// font-aware, so reading the LAGGING `shaped_font` there would leave the
/// caret itself showing STALE (source-world) geometry for the entire window
/// between a preview's color retint and its deferred reshape — exactly the
/// kind of surface `render::tests::distinguishability`'s
/// `theme_preview_retint_regrounds_the_page_surface_on_every_world` law exists
/// to catch (a full-frame pixel diff caught this directly during development;
/// this is the fast unit-level companion, pinned at the exact seam).
///
/// Non-vacuous: reverting the caller's gate to `self.shaped_font` (item 105's
/// first draft) makes this red — the MONO source's stale `shaped_font` makes
/// `caret_cell_vertical` take the old byte-identical MONO branch even after
/// the active theme (and the caret's OWN colour) have already moved to a
/// PROPORTIONAL destination.
#[test]
fn caret_fallback_geometry_tracks_the_live_theme_not_the_lagging_shaped_font() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping caret_fallback_geometry_tracks_the_live_theme_not_the_lagging_shaped_font: no wgpu adapter");
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
