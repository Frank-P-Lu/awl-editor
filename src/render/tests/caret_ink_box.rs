//! ITEM 91 — THE CARET INK BOX (VERTICAL). Laws for the one rule that sizes the
//! CELL-form caret's TOP and BOTTOM to the anchored glyph's own full raster ink box
//! instead of the generic line cell.
//!
//! The reported bug: the settled Block quad's height was `CARET_BLOCK_H` (a fixed
//! 0.8 fraction of the row) centred on the line box, so its TOP sat at the same y
//! for every letter while the real ink top moves with the letter — measured on
//! Gumtree/Literata at zoom 1, the caret hung **8–9px of empty accent above** the
//! ink of `a`/`m`/`g`/`y` and ~3px above `i`. Only the BOTTOM had a glyph-aware
//! rule (the descender extension), which is exactly why the two edges disagreed.
//!
//! The fix is one owner — `TextPipeline::caret_cell_vertical` — with arms
//! behind the SAME ink funnel (`caret_anchor_ink_box`) the horizontal ink
//! alignment already rode: the padded ink box on a proportional world, else
//! (item 105 below) a REAL ligature raster box or a SYNTHETIC typical-letter
//! box on a proportional glyphless anchor, else (mono only) the row-scaled
//! line cell (descender extension folded in). These laws pin, with glyph-mask
//! arithmetic against the real raster placement:
//!
//!   * settled: the caret's top/bottom ARE the ink box ± one letter-INDEPENDENT
//!     pad, across an ascender, an x-height letter and two descenders;
//!   * moving: the travelling streak is untouched by the ink box;
//!   * mono: the uniform grid is byte-identical (a fixed top, a bottom that drops
//!     only for a real dipper);
//!   * every caret FORM is swept by a no-wildcard match, so a new `CaretMode`
//!     cannot dodge the vertical policy;
//!   * the glyphless space / end-of-line / bar fallbacks read the SAME
//!     baseline-relative formula as the ink-box arm on a proportional world
//!     (`layers.rs` holds NO vertical caret geometry of its own — the grep-law
//!     that bans a second rule from growing back).
//!
//! **ITEM 105 — THE ADJACENT-COLUMN TRANSITION.** Item 91's laws above prove
//! each arm correct IN ISOLATION; none of them construct the SEAM between an
//! on-glyph column and an adjacent glyphless one on the SAME row. That seam is
//! exactly what a user's paired release screenshots caught: on `aaa`, the
//! Block caret on the final `a` versus one column later at end-of-line has a
//! visibly discontinuous outer cell, on every proportional world in the
//! roster (the pre-105 fallback centred on `caret.pos.y` — a row-box-geometric
//! centre — while the ink-box arm centres on the font BASELINE; the two
//! conventions had no reason to agree, and measurably did not: 2–13px at zoom
//! 1 depending on world/glyph class). The fix gives the fallback the SAME
//! baseline-relative formula the ink-box arm uses, fed a real ligature raster
//! box when one exists or else a SYNTHETIC typical-letter box
//! (`facepitch::typical_letter_ratio` — the shipped face's own measured mean
//! of x-height and cap-height, scaled by the row's own real `max_ascent`) —
//! see `TextPipeline::caret_synthetic_ink_box` and `caret_row_metrics`'s doc.
//! The TRANSITION law itself — swept over the full proportional roster, every
//! representative glyph class, Block/Morph rest, a wrapped-line boundary, two
//! zooms and 1x/2x DPI, with an explicit non-vacuity proof against the pre-105
//! formula — lives in `render/tests/caret_transition_item105.rs`, a sibling
//! file rather than an addition here, because it is a different KIND of law
//! (adjacent-column diffs, not single-column measurements).

use super::super::*;
use super::{headless_pipeline, view};

/// The pixel scale the pads ride (zoom × dpi), read the same way the geometry does.
fn pad_px(p: &TextPipeline) -> f32 {
    p.metrics.caret_h / CARET_H
}

/// The settled caret's drawn vertical bounds `(top, bottom)` — straight off the
/// geometry the renderer draws from, so a law here is a law about pixels.
fn caret_top_bottom(p: &mut TextPipeline) -> (f32, f32) {
    let (_cx, cy, _w, h, _corner, _ax, _ay) = p.caret_geometry();
    (cy - h * 0.5, cy + h * 0.5)
}

/// THE CORE LAW (item 91). On a PROPORTIONAL world the settled cell caret's TOP
/// and BOTTOM are the anchored glyph's own full raster ink box grown by exactly
/// `CARET_INK_PAD` — for an ASCENDER (`l`), an X-HEIGHT letter (`a`, `m`) and a
/// DESCENDER (`g`, `y`) alike, through the SAME box (no separate descender rule,
/// no per-letter list, no per-world offset).
///
/// Non-vacuous twice over: it asserts the pad is the same small number for every
/// letter class (the exact property the old line-cell geometry failed — its top
/// was letter-independent while the ink's was not), AND that the pre-fix line-cell
/// top really did hang far above the ink on a non-ascender, so the fixture
/// reproduces the reported bug rather than passing either way.
#[test]
fn cell_caret_hugs_the_full_ink_box_on_ascenders_x_height_and_descenders() {
    // Ink-box lookup folds the theme font AND the page wrap globals; the anchor is
    // mode-keyed. Hold theme -> page -> caret (the suite-wide order), pin BLOCK.
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping cell_caret_hugs_the_full_ink_box_on_ascenders_x_height_and_descenders: no wgpu adapter"
        );
        return;
    };
    theme::set_active_by_name("Gumtree").unwrap(); // proportional (Literata)
    p.sync_theme();

    // One fixture line spanning all three vertical classes. `l` is the ascender,
    // `a`/`m` sit on the x-height, `g`/`y` dip below the baseline.
    let text = "lamgy";
    let pad = CARET_INK_PAD * pad_px(&p);
    let mut top_gaps: Vec<(char, f32)> = Vec::new();
    let mut saw_ascender_ink = false;
    let mut saw_descender_ink = false;

    for (col, ch) in text.chars().enumerate() {
        p.set_view(&view(text, 0, col));
        p.settle_caret();

        // GLYPH-MASK ARITHMETIC: the real rasterized placement of the very glyph
        // the caret sits on — the same swash box glyphon blits the letter from.
        let ink = p
            .caret_anchor_ink_box()
            .unwrap_or_else(|| panic!("'{ch}' must yield a real ink box on Gumtree"));
        let baseline = p.caret_baseline_y();
        let ink_top = baseline - ink.top;
        let ink_bottom = baseline + ink.descent();

        let (top, bottom) = caret_top_bottom(&mut p);
        assert!(
            (top - (ink_top - pad)).abs() < 1e-2,
            "'{ch}': caret top must be the ink top minus one pad: top={top} ink_top={ink_top} pad={pad}"
        );
        assert!(
            (bottom - (ink_bottom + pad)).abs() < 1e-2,
            "'{ch}': caret bottom must be the ink bottom plus one pad (descenders \
             covered through the SAME box): bottom={bottom} ink_bottom={ink_bottom} pad={pad}"
        );
        // The caret always fully CONTAINS the letter's ink, with room to spare.
        assert!(
            top < ink_top && bottom > ink_bottom,
            "'{ch}': the ink must sit strictly inside the caret: caret={top}..{bottom} ink={ink_top}..{ink_bottom}"
        );

        top_gaps.push((ch, ink_top - top));

        // Fixture witnesses: the line really does hold both an ascender-tall ink
        // box and a below-baseline one, so the letter classes are genuinely covered.
        if ch == 'l' {
            saw_ascender_ink = true;
        }
        if ink.descent() > 2.0 {
            saw_descender_ink = true;
        }

        // NON-VACUITY: the PRE-FIX geometry — the generic row-scaled line cell
        // centred on the spring anchor — put the top far above this ink on a
        // non-ascender. That gap is the reported bug (8–9px measured).
        let old_top = p.caret.pos.y - (p.metrics.caret_block_h * p.cursor_scale()) * 0.5;
        if ch == 'a' || ch == 'm' {
            assert!(
                ink_top - old_top > 5.0,
                "'{ch}': fixture must reproduce the reported gap — the old line-cell \
                 top sat only {} px above the ink",
                ink_top - old_top
            );
        }
    }

    assert!(saw_ascender_ink, "fixture must include an ascender");
    assert!(saw_descender_ink, "fixture must include a real descender");

    // THE LETTER-INDEPENDENCE LAW: the margin above the ink is ONE small constant
    // for every letter — that is what makes this a mechanism and not a table of
    // offsets. (The old geometry's top gap ranged 3px on `i` to 9px on `a`.)
    let (_c0, first) = top_gaps[0];
    for &(ch, gap) in &top_gaps {
        assert!(
            (gap - first).abs() < 1e-2,
            "'{ch}': the top margin must be letter-INDEPENDENT: {gap} vs {first} \
             (all gaps: {top_gaps:?})"
        );
        assert!(
            gap > 0.0 && gap < 5.0 * pad_px(&p),
            "'{ch}': the top margin must stay SMALL and bounded: {gap}"
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE FORM SWEEP (no-wildcard). Every caret LOOK is enumerated through
/// `CaretMode::ALL` and matched EXHAUSTIVELY — a new look added to the enum fails
/// to compile here, so it cannot silently pick its own vertical rule:
///
///   * `Block` / `Morph` draw the CELL form (Morph's fast-travel deferral and its
///     ink-caret-world fold both land on the very same quad), so their vertical
///     bounds come from the ink box.
///   * `Ibeam` is the BAR form — an insertion bar marks the boundary BETWEEN
///     glyphs, so it deliberately spans the LINE BOX (`ibeam_bar_dims`) and must
///     be provably NOT ink-sized.
#[test]
fn cell_caret_vertical_has_one_owner_across_every_caret_form() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping cell_caret_vertical_has_one_owner_across_every_caret_form: no wgpu adapter"
        );
        return;
    };
    theme::set_active_by_name("Gumtree").unwrap();
    p.sync_theme();
    // Both `m`s are the same glyph, so Block (anchoring the cursor cell) and Morph
    // (anchoring one char BACK) measure the identical ink box — the comparison is
    // about the RULE, not about which letter each look happens to sit on.
    let text = "mm";
    let col = 1;
    let pad = CARET_INK_PAD * pad_px(&p);

    for mode in CaretMode::ALL {
        crate::caret::set_mode(mode);
        p.set_view(&view(text, 0, col));
        p.settle_caret();
        let ink = p
            .caret_anchor_ink_box()
            .expect("'m' must yield an ink box on Gumtree");
        let baseline = p.caret_baseline_y();
        let (ink_top, ink_bottom) = (baseline - ink.top, baseline + ink.descent());

        match mode {
            CaretMode::Block | CaretMode::Morph => {
                let (cy, h) = p.caret_cell_vertical();
                assert!(
                    (cy - h * 0.5 - (ink_top - pad)).abs() < 1e-2
                        && (cy + h * 0.5 - (ink_bottom + pad)).abs() < 1e-2,
                    "{mode:?}: the CELL form must take its vertical from the ink box: \
                     got {}..{} want {}..{}",
                    cy - h * 0.5,
                    cy + h * 0.5,
                    ink_top - pad,
                    ink_bottom + pad
                );
                assert!(
                    !p.caret_is_bar_form(),
                    "{mode:?}: fixture must be the cell form here"
                );
            }
            CaretMode::Ibeam => {
                assert!(p.caret_is_bar_form(), "Ibeam must be the bar form");
                // The bar AS DRAWN at rest (settle 1) — its own line-box geometry.
                let (_bx, _by, _bw, tall, _bc) = p.caret_ibeam_geometry();
                assert!(
                    (tall - p.metrics.caret_h * p.cursor_scale()).abs() < 1e-3,
                    "Ibeam must span the LINE BOX, not the ink box: tall={tall}"
                );
                assert!(
                    tall > (ink_bottom - ink_top) + 2.0 * pad + 1.0,
                    "the I-beam bar must be provably TALLER than the ink cell would be \
                     (so this arm is non-vacuous): tall={tall} ink_h={}",
                    ink_bottom - ink_top
                );
            }
        }
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE MONO GRID IS UNTOUCHED. A monospace display exists to look perfectly
/// uniform, so `caret_anchor_ink_box` returns `None` there and the cell caret keeps
/// the historical row-scaled `caret_block_h` line cell: the TOP is the SAME y at
/// every column (no per-glyph wobble), the height is exactly the line cell on a
/// non-dipper, and the ONLY variation is the descender extension dropping the
/// BOTTOM for a real dipper — byte-identical to the pre-item-91 geometry, which
/// applied that extension at the draw site.
///
/// ITEM 97 widened this from the hand-listed pair `["Tawny", "Mangrove"]` to a
/// SWEEP over every mono-display world in the roster
/// (`super::facepitch::mono_display_worlds`, derived from each face's own
/// measured advance widths). The old pair is precisely how the bug hid: Currawong
/// and Cassowary shape in Iosevka — a real fixed pitch the retired
/// `font_is_mono` name list did not know — so they took the proportional ink-box
/// arm and this law never looked. It now fails for any mono-faced world that
/// loses the grid.
#[test]
fn mono_world_caret_grid_stays_uniform_and_line_box_sized() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping mono_world_caret_grid_stays_uniform_and_line_box_sized: no wgpu adapter"
        );
        return;
    };
    let text = "lamgy";

    let worlds = super::facepitch::mono_display_worlds();
    assert!(
        worlds.len() >= 7,
        "every mono-display world is swept, got {worlds:?}"
    );
    for world in worlds {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();

        let mut tops: Vec<f32> = Vec::new();
        let mut dippers = 0usize;
        for (col, ch) in text.chars().enumerate() {
            p.set_view(&view(text, 0, col));
            p.settle_caret();
            assert!(
                p.caret_anchor_ink_box().is_none(),
                "{world}: a mono world must never ink-align ('{ch}')"
            );
            let (cy, h) = p.caret_cell_vertical();
            let cell_h = p.metrics.caret_block_h * p.cursor_scale();
            tops.push(cy - h * 0.5);

            let descender = p
                .caret_anchor_raster_box()
                .map(|b| b.descent())
                .unwrap_or(0.0);
            if descender > 2.0 {
                dippers += 1;
                assert!(
                    h >= cell_h - 1e-3,
                    "{world}: a dipper ('{ch}') may only GROW the cell: h={h} cell={cell_h}"
                );
            } else {
                assert!(
                    (h - cell_h).abs() < 1e-3,
                    "{world}: a non-dipper ('{ch}') must be exactly the line cell: \
                     h={h} cell={cell_h}"
                );
            }
        }
        assert!(dippers >= 2, "{world}: fixture must include real dippers");

        // THE UNIFORM GRID: one TOP y for every column, dippers included.
        let first = tops[0];
        for (i, t) in tops.iter().enumerate() {
            assert!(
                (t - first).abs() < 1e-3,
                "{world}: the mono caret top must be identical at every column \
                 (col {i}: {t} vs {first}) — the grid is the whole point"
            );
        }
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE OTHER HALF OF ITEM 97'S SWEEP: every PROPORTIONAL-display world still
/// takes the ink-box arm, with a per-letter top — unchanged by the pitch
/// predicate becoming a measurement.
///
/// The risk a widened `font_is_mono` carries is over-reach: a predicate that
/// mistook a near-gridded face (iA Writer Quattro S is bundled and duospaced) for
/// a mono would flip that world's caret from ink-hugging to a fixed cell, a
/// silent look change in the opposite direction. This is the complement of
/// `mono_world_caret_grid_stays_uniform_and_line_box_sized` over the SAME roster
/// split, so between them every world in `theme::THEMES` is asserted to be in
/// exactly the arm its face's own advance widths put it in.
#[test]
fn proportional_worlds_still_ink_align_with_a_per_letter_top() {
    let _t = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping proportional_worlds_still_ink_align_with_a_per_letter_top: no wgpu adapter"
        );
        return;
    };
    let text = "lamgy";
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;
    for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let mut tops: Vec<f32> = Vec::new();
        for (col, ch) in text.chars().enumerate() {
            p.set_view(&view(text, 0, col));
            p.settle_caret();
            assert!(
                p.caret_anchor_ink_box().is_some(),
                "{} ({}): a proportional world must ink-align ('{ch}')",
                t.name,
                t.font
            );
            let (cy, h) = p.caret_cell_vertical();
            tops.push(cy - h * 0.5);
        }
        // An ASCENDER's ink starts higher than an x-height letter's, so the tops
        // must genuinely differ — the property the mono arm deliberately lacks.
        let spread = tops.iter().cloned().fold(f32::MIN, f32::max)
            - tops.iter().cloned().fold(f32::MAX, f32::min);
        assert!(
            spread > 1.0,
            "{} ({}): the ink-box top must move with the letter (spread {spread})",
            t.name,
            t.font
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

/// THE MOVING CARET IS UNTOUCHED. The ink box corrects the RESTING pose only: both
/// the height and the vertical re-centring are scaled by the settle factor, so a
/// caret mid-glide is still the thin streak running through the TEXT optical centre
/// (`pos.y + caret_trail_drop`) with the streak's own thickness — identical on a
/// proportional world (where the ink box applies at rest) and a mono world (where
/// it never does. Covers the MOVING half of the settled/moving pair; the settled
/// half is `cell_caret_hugs_the_full_ink_box_on_ascenders_x_height_and_descenders`.
#[test]
fn moving_caret_streak_is_unaffected_by_the_ink_box() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping moving_caret_streak_is_unaffected_by_the_ink_box: no wgpu adapter");
        return;
    };
    let text = "alpha\nbeta\ngamma\ndelta\nepsilon\nzeta\neta\ntheta\niota";

    for world in ["Gumtree", "Tawny"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        p.set_view(&view(text, 0, 0));

        // HORIZONTAL fast glide (settle ≈ 0): the deterministic mid-motion pose the
        // `--screenshot-motion` capture renders.
        p.inject_motion_demo();
        let (_cx, cy, w, h, _c, _ax, _ay) = p.caret_geometry();
        let s = p.caret.settle_factor();
        assert!(
            s < 0.2,
            "{world}: fixture must be genuinely mid-glide (s={s})"
        );
        assert!(
            w > h,
            "{world}: the motion pose must be long-and-thin: w={w} h={h}"
        );
        assert!(
            h < p.metrics.caret_block_h * 0.5,
            "{world}: the streak must stay thin — the ink box must not thicken it: h={h}"
        );
        let want_cy = p.caret.pos.y + p.metrics.caret_trail_drop;
        assert!(
            (cy - want_cy).abs() < 0.5,
            "{world}: the streak must run through the TEXT centre, NOT be pulled onto \
             the ink box: cy={cy} want={want_cy}"
        );

        // VERTICAL fast glide: same rule, other axis.
        p.inject_motion_demo_vertical();
        let (_cx, _cy, w_v, h_v, ..) = p.caret_geometry();
        assert!(
            w_v > h_v,
            "{world}: the vertical motion pose must stay long-and-thin: w={w_v} h={h_v}"
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE GLYPHLESS FALLBACKS SURVIVE — UPDATED BY ITEM 105. There is no ink to
/// hug at a SPACE, at END-OF-LINE, or on an EMPTY line, so the ONE ink funnel
/// returns `None`. On a MONO world the cell owner still falls back to the
/// historical row-scaled `caret_block_h` centred on the spring anchor
/// (item 97's uniform grid, byte-identical). On a PROPORTIONAL world it no
/// longer does: item 105 found that THIS exact invariant — the fallback
/// pinned to `caret.pos.y` (a row-box-geometric-centre convention) — was the
/// root cause of a visible cell jump the instant a proportional caret left a
/// real glyph for an adjacent glyphless column (the user's `aaa`->EOL report;
/// see `render/tests/caret_transition_item105.rs`). The fallback now reads a
/// SYNTHETIC typical-letter box through the SAME baseline-relative formula the
/// ink-box arm above uses, so this test pins the NEW formula directly rather
/// than re-asserting the convention that caused the bug. The space bar and
/// Morph's line-start bar-form mechanics (not their SIZE — see below) are
/// otherwise unchanged.
#[test]
fn glyphless_fallbacks_use_the_synthetic_baseline_box_on_proportional_worlds() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping glyphless_fallbacks_use_the_synthetic_baseline_box_on_proportional_worlds: no wgpu adapter"
        );
        return;
    };
    theme::set_active_by_name("Gumtree").unwrap();
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
    let text = "am am"; // col 2 = the space; col 5 = end of line
    let pad = CARET_INK_PAD * pad_px(&p);

    for (col, what) in [(2usize, "a space"), (5usize, "end of line")] {
        p.set_view(&view(text, 0, col));
        p.settle_caret();
        assert!(
            p.caret_anchor_ink_box().is_none(),
            "{what}: a glyphless anchor must yield no ink box"
        );
        let (cy, h) = p.caret_cell_vertical();
        // NEW INVARIANT: the fallback is no longer the row-box-centred line
        // cell — it must have MOVED OFF `caret.pos.y`/`caret_block_h`, proving
        // item 105 actually changed this seam rather than leaving it inert.
        let old_cy = p.caret.pos.y;
        let old_h = p.metrics.caret_block_h * p.cursor_scale();
        assert!(
            (cy - old_cy).abs() > 0.5 || (h - old_h).abs() > 0.5,
            "{what}: the proportional fallback must no longer equal the old \
             row-box-centred cell (cy={cy} h={h} old_cy={old_cy} old_h={old_h})"
        );
        // The height must still be a small, positive, letter-plausible cell —
        // never collapsed, never the item-91-original oversized fixed cap.
        assert!(
            h > pad && h < old_h * 1.5,
            "{what}: the synthetic cell must stay a plausible letter-sized box: h={h}"
        );
    }

    // The SPACE BAR routes through the same owner, so it inherits whatever
    // `caret_cell_vertical` returns at this glyphless anchor — no longer
    // pinned to the old fixed line-box height on a proportional world.
    p.set_view(&view(text, 0, 2));
    p.settle_caret();
    let (owner_cy, owner_h) = p.caret_cell_vertical();
    let (_bx, by, bw, bh, _bc) = p.caret_space_bar_geometry();
    assert!(
        (bh - owner_h).abs() < 1e-3 && (by - owner_cy).abs() < 1e-3,
        "the glyphless space bar must read the SAME owner value: by={by} bh={bh} \
         want cy={owner_cy} h={owner_h}"
    );
    assert!(
        (bw - CARET_SPACE_BAR_W * p.metrics.zoom).abs() < 1e-3,
        "the space bar stays the slim bar: bw={bw}"
    );

    // MORPH's LINE-START degrade: the I-beam's own bar, line-box tall — still
    // untouched by the ink-box mechanism (a bar has no glyph of its own to
    // hug), unchanged by item 105 exactly as it was by item 91.
    crate::caret::set_mode(CaretMode::Morph);
    p.set_view(&view(text, 0, 0));
    p.settle_caret();
    assert!(p.caret_is_bar_form(), "col 0 in Morph must be the bar form");
    let (_lx, _ly, _lw, lh, _lc) = p.caret_linestart_bar_geometry();
    assert!(
        (lh - p.metrics.caret_h * p.cursor_scale()).abs() < 1e-3,
        "the line-start bar must span the LINE BOX: lh={lh}"
    );

    // THE MONO COMPLEMENT: on a mono world the fallback is BYTE-IDENTICAL to
    // the pre-105 line-cell — item 97's uniform grid never reads any ink box,
    // real or synthetic (see `caret_cell_vertical`'s mono arm).
    theme::set_active_by_name("Tawny").unwrap();
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
    let cell = |p: &TextPipeline| p.metrics.caret_block_h * p.cursor_scale();
    for (col, what) in [(2usize, "a space"), (5usize, "end of line")] {
        p.set_view(&view(text, 0, col));
        p.settle_caret();
        let (cy, h) = p.caret_cell_vertical();
        assert!(
            (h - cell(&p)).abs() < 1e-3 && (cy - p.caret.pos.y).abs() < 1e-3,
            "{what} (Tawny, mono): must keep the OLD line-box cell on the spring \
             anchor: cy={cy} h={h} want cy={} h={}",
            p.caret.pos.y,
            cell(&p)
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE PAD IS BOUNDED AND OUTRUNS THE SILHOUETTE'S OWN DILATION. Two structural
/// facts the mechanism rests on, asserted as data so a future tune cannot quietly
/// break either:
///
///   * `CARET_INK_PAD` is SMALL relative to the line cell (a pad, not a second
///     cell height) — otherwise "hug the ink" would drift back into "cover the row".
///   * it is strictly GREATER than `CARET_MORPH_DILATE_PX`. On a world whose block
///     KNOCKS THE GLYPH OUT of the lit cell (`CaretBlockStyle::Filled` — the CRT
///     phosphor cursor), the knockout is the morph silhouette dilated by that
///     constant; a pad at or below it would let the knockout eat the whole cell and
///     the caret would vanish exactly where the caret IS the cursor.
#[test]
fn caret_ink_pad_is_bounded_and_exceeds_the_morph_dilation() {
    assert!(
        CARET_INK_PAD > CARET_MORPH_DILATE_PX,
        "the ink pad must outrun the knockout/silhouette dilation: pad={CARET_INK_PAD} \
         dilate={CARET_MORPH_DILATE_PX}"
    );
    assert!(
        CARET_INK_PAD > 0.0 && CARET_INK_PAD < CARET_BLOCK_H * 0.25,
        "the ink pad must stay a small margin, not a second cell height: {CARET_INK_PAD}"
    );
}

/// GREP-LAW: the caret's vertical geometry has ONE owner, and the DRAW SITE holds
/// none of it. The bug this item fixed was structural — `layers.rs` re-derived a
/// descender-aware BOTTOM off the already motion-blended rect while nothing pulled
/// the TOP down, so the two edges could not agree. Ban the ingredients of a second
/// vertical rule from that file: a raster/ink box read, a descender depth, and the
/// line-cell height constants. `caret_geometry` (the one owner's own consumer) is
/// all `prepare_caret_block` may call.
#[test]
fn layers_holds_no_caret_vertical_geometry_of_its_own() {
    let src = include_str!("../layers.rs");
    // CALL/FIELD-shaped tokens (leading `.`) for the seams, so the file may still
    // NAME the owner in a doc comment while being unable to invoke a second rule.
    for banned in [
        ".caret_anchor_raster_box(",
        ".caret_anchor_ink_box(",
        ".caret_cell_vertical(",
        ".caret_baseline_y() +",
        ".descent()",
        ".caret_block_h",
        "CARET_DESCENDER_PAD",
        "CARET_INK_PAD",
    ] {
        assert!(
            !src.contains(banned),
            "render/layers.rs must hold NO caret vertical geometry of its own — found \
             `{banned}`. The cell caret's top/bottom belong to \
             `TextPipeline::caret_cell_vertical`, reached through `caret_geometry`."
        );
    }
    // ...and the owner really is where the rule lives (so the ban above is not
    // passing merely because the mechanism moved somewhere else again).
    let owner = include_str!("../caret.rs");
    assert!(
        owner.contains("fn caret_cell_vertical")
            && owner.contains("CARET_INK_PAD")
            && owner.contains("CARET_DESCENDER_PAD"),
        "render/caret.rs must own the cell caret's vertical rule (both pads included)"
    );
}
