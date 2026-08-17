//! THE CARET'S ONE HEIGHT (VERTICAL). Laws for the rule that gives the
//! CELL-form caret ONE height per (face, row) on a proportional world, and for
//! WHICH box supplies it.
//!
//! **THREE SHAPES HAVE BEEN TRIED FOR THE ROW'S ONE HEIGHT, AND THE HISTORY IS
//! THE WHOLE POINT OF THIS FILE — each failure was a deliberate fix for the
//! one before it, and the SHIPPED shape now differs by caret FORM, not just by
//! magnitude.**
//!
//!   * A fixed fraction of the ROW (`CARET_BLOCK_H`, 0.8 of the line box, still
//!     the mono arm's rule) is perfectly stable and hangs **8–9px of empty
//!     accent above** an `a`/`m` on Gumtree/Literata at zoom 1 while clearing
//!     an `l` by ~3px, because a row box is not a letter.
//!   * Sizing to the ANCHORED GLYPH'S own raster ink box closes that gap to a
//!     letter-independent pad — and makes the caret's own top and bottom move
//!     with every letter typed, which the user reported as distracting in
//!     ordinary prose. It is the row-fraction failure's exact mirror: perfect
//!     per-letter fit, no stability.
//!   * The TYPICAL-LETTER box (`facepitch::typical_letter_ratio` — the shipped
//!     face's own measured mean of x-height and cap-height, times the row's
//!     own real `max_ascent`), padded, closed that gap for ordinary prose —
//!     but a real ASCENDER (`d`, `l`, `b`, `h`, `k`) still visibly pokes its
//!     ink above it, because "typical" is tuned to the mean letter, not the
//!     tallest one that can occupy the cell. The user's verdict on that
//!     residual (item 451) is the fourth shape below, and it does NOT retire
//!     the typical box — it narrows where that box still applies.
//!   * SHIPPED: `caret_cell_vertical` now reads
//!     [`super::super::caret::CaretMode`]. The literal **Block** caret (and a
//!     Morph preference folded to Block on an ink-caret world) takes the row's
//!     real ASCENDER-to-DESCENDER ink envelope
//!     (`facepitch::ink_envelope_em`, real glyph outline bounds over a
//!     representative roster — never `hhea` ascent/descent, whose sum exceeds
//!     the app's own row height on several bundled faces). Every other
//!     consumer of the same shared cell owner — Morph's support-body floor
//!     decision, Morph's fast-travel deferral, the glyphless space bar — keeps
//!     the typical-letter box, because handing Morph the taller envelope would
//!     make its floor decision trip on nearly every ordinary letter and draw a
//!     full body behind text Morph exists to leave mostly uncovered.
//!
//! `TextPipeline::caret_cell_vertical` is still the one owner. It has TWO arms
//! split by `crate::caret::font_is_mono` (proportional vs. the row-scaled
//! mono line cell, descender extension folded in) — and, inside the
//! proportional arm, a second gate on `effective_caret_look() ==
//! CaretMode::Block` that picks the ink envelope over the typical box. These
//! laws pin, with glyph-mask arithmetic against the real raster placement:
//!
//!   * settled, Block: the caret's top and bottom are the SAME two numbers for
//!     an ascender, an x-height letter, a capital and two descenders — while
//!     those letters' own ink boxes provably still differ (the axis the caret
//!     no longer follows must stay LIVE in the fixture, or "one height" is a
//!     claim about the fixture rather than about the caret) — and CONTAINS
//!     every one of those letters' own ink, top and bottom alike;
//!   * that the Block height is genuinely taller than the retired
//!     typical-letter box (or item 451's fix has not landed) and reaches at
//!     least the tallest per-glyph ink cell in the fixture;
//!   * settled, Morph: the typical-letter box is UNCHANGED from before item
//!     451 — Morph does not inherit Block's taller envelope;
//!   * moving: the travelling streak is untouched;
//!   * mono: the uniform grid is byte-identical (a fixed top, a bottom that drops
//!     only for a real dipper) — untouched by any of the above, on every form;
//!   * every caret FORM is swept by a no-wildcard match, so a new `CaretMode`
//!     cannot dodge the vertical policy;
//!   * the glyphless space / end-of-line / bar cases read the SAME (typical)
//!     box the letters do on Morph (`layers.rs` holds NO vertical caret
//!     geometry of its own — the grep-law that bans a second rule from growing
//!     back).
//!
//! **THE ADJACENT-COLUMN SEAM**, once the discontinuity a user's paired release
//! screenshots caught between an on-glyph column and the glyphless one beside
//! it, is now zero by construction — one arm has no seam. The sweep that proves
//! it (full proportional roster × every representative glyph class × Block and
//! Morph rest × a wrapped-line boundary × two zooms × 1x/2x DPI) lives in
//! `render/tests/caret_transition.rs`, a sibling file rather than an addition
//! here, because it is a different KIND of law (adjacent-column diffs, not
//! single-column measurements).

use super::super::*;
use super::{headless_pipeline, view};

/// The pixel scale the pads ride (zoom × dpi), read the same way the geometry does:
/// the stored [`render::Metrics::scale`] field, not a division that recovers it.
fn pad_px(p: &TextPipeline) -> f32 {
    p.metrics.scale
}

/// The settled caret's drawn vertical bounds `(top, bottom)` — straight off the
/// geometry the renderer draws from, so a law here is a law about pixels.
fn caret_top_bottom(p: &mut TextPipeline) -> (f32, f32) {
    let (_cx, cy, _w, h, _corner, _ax, _ay) = p.caret_geometry();
    (cy - h * 0.5, cy + h * 0.5)
}

/// Re-derive the literal Block caret's `(want_top, want_bottom)` from
/// `facepitch::ink_envelope_em`/`vertical_em_metrics` and the row's own
/// metrics — INCLUDING the pad-shrinking floor `caret_cell_vertical_block`
/// applies when the padded envelope would overshoot the row's own line
/// height (`block_envelope_never_touches_the_adjacent_row`'s own law). A law
/// calling this checks the RULE, both its ink source and its row-fit floor,
/// rather than a restatement of the owner that skips the floor and fails on
/// the roster's tightest bundled face at body size.
fn want_block_top_bottom(baseline: f32, row_ascent: f32, font: &str, row_h: f32, pad: f32) -> (f32, f32) {
    let (hhea_ascent, _) = super::super::facepitch::vertical_em_metrics(font);
    let (ink_ascent_em, ink_descent_em) = super::super::facepitch::ink_envelope_em(font);
    let font_size = row_ascent / hhea_ascent;
    let top = (font_size * ink_ascent_em).max(1.0);
    let bottom = (font_size * ink_descent_em).max(0.0);
    let ink_h = top + bottom;
    let ideal_h = ink_h + 2.0 * pad;
    let clearance = pad / CARET_INK_PAD.0; // 1 logical px, scaled the same way `pad` was
    let max_h = (row_h - clearance).max(ink_h);
    let h = ideal_h.min(max_h);
    let center = baseline - top + ink_h * 0.5;
    (center - h * 0.5, center + h * 0.5)
}

/// THE CORE LAW. On a PROPORTIONAL world the settled cell caret's TOP and
/// BOTTOM are ONE PAIR OF NUMBERS for the whole row: the row's typical-letter
/// box grown by one [`CARET_INK_PAD`], identical on an ascender, an x-height
/// letter, a capital and a descender. The formula is RE-DERIVED here from
/// `facepitch::ink_envelope_em` and the row's own metrics rather than read
/// back out of the owner, so this is a law about the rule and not a restatement
/// of the code.
///
/// NON-VACUOUS THREE WAYS, and the first is the one that matters: an equality
/// law over a set is satisfiable by the SET being uniform, so this asserts the
/// letters' OWN raster ink boxes still differ across the fixture by several px
/// — the axis the caret deliberately ignores has to be live, or "one height" is
/// a fact about `lamgy` rather than about the caret. Then the retired
/// typical-letter box is measured on the same fixture and shown to be a
/// genuinely SHORTER number (the shape item 451 replaced for Block), and
/// CONTAINMENT is asserted directly, per letter, both edges: no anchored
/// glyph's own ink — ascender or descender — may fall outside the drawn box.
#[test]
fn cell_caret_takes_the_block_ink_envelope_across_every_letter_class() {
    // Ink-box lookup folds the theme font AND the page wrap globals; the anchor is
    // mode-keyed. Hold theme -> page -> caret (the suite-wide order), pin BLOCK.
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping the one-block-ink-envelope-height law: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Gumtree").unwrap(); // proportional (Literata)
    p.sync_theme();

    // One fixture line spanning every vertical class. `l` is the ascender,
    // `a`/`m` sit on the x-height, `g`/`y` dip below the baseline, `A` is a
    // capital — the class whose absence once hid a regression for a whole
    // round (see `caret_transition.rs`'s module doc).
    let text = "lamgyA";
    let pad = CARET_INK_PAD.px(pad_px(&p));
    let mut drawn: Vec<(char, f32, f32)> = Vec::new();
    let mut ink_tops: Vec<(char, f32)> = Vec::new();
    let mut ink_cell_heights: Vec<(char, f32)> = Vec::new();
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
        let (baseline, row_ascent, font) = p.caret_row_metrics();
        let ink_top = baseline - ink.top;
        let ink_bottom = baseline + ink.descent();
        // THE RULE, re-derived from the face's own measured ink extremes rather
        // than read back from `caret_cell_vertical` — including the row-fit
        // floor, since Gumtree/Literata's own margin is tight enough at body
        // size to engage it (`block_envelope_never_touches_the_adjacent_row`).
        let row_h = p.cursor_row_height();
        let (want_top, want_bottom) = want_block_top_bottom(baseline, row_ascent, font, row_h, pad);

        let (top, bottom) = caret_top_bottom(&mut p);
        assert!(
            (top - want_top).abs() < 1e-2,
            "'{ch}': caret top must be the row's BLOCK ink-envelope top minus one \
             pad, whatever letter is anchored: top={top} want={want_top} pad={pad}"
        );
        assert!(
            (bottom - want_bottom).abs() < 1e-2,
            "'{ch}': caret bottom must be the row's BLOCK ink-envelope bottom plus \
             one pad: bottom={bottom} want={want_bottom} pad={pad}"
        );

        // CONTAINMENT — item 451's actual law, per letter, both edges: no
        // anchored glyph's own real raster ink may fall outside the drawn box.
        // (The retired typical box let an ascender's ink pass above the top;
        // this is that assertion's flip, and the mutation proof below
        // reinstates the retired formula to watch it fail here by name.)
        assert!(
            ink_top >= top - 1e-2,
            "'{ch}': the glyph's own ink top ({ink_top:.2}) must not rise above \
             the caret box top ({top:.2}) — an ascender must be fully covered"
        );
        assert!(
            ink_bottom <= bottom + 1e-2,
            "'{ch}': the glyph's own ink bottom ({ink_bottom:.2}) must not sink \
             below the caret box bottom ({bottom:.2}) — a descender must be \
             fully covered"
        );

        drawn.push((ch, top, bottom));
        ink_tops.push((ch, ink_top));
        // What the PER-GLYPH rule would have drawn for this letter: its own ink
        // box, padded — the shape the user reported as distracting, and the
        // floor the shipped ONE height must still reach for every letter.
        ink_cell_heights.push((ch, ink.top.max(1.0) + ink.descent() + 2.0 * pad));

        // Fixture witnesses: the line really does hold both an ascender-tall ink
        // box and a below-baseline one, so the letter classes are genuinely covered.
        if ch == 'l' {
            saw_ascender_ink = true;
        }
        if ink.descent() > 2.0 {
            saw_descender_ink = true;
        }
    }

    assert!(saw_ascender_ink, "fixture must include an ascender");
    assert!(saw_descender_ink, "fixture must include a real descender");

    // THE ONE-HEIGHT LAW: every letter draws the identical top AND bottom.
    let (_c0, first_top, first_bottom) = drawn[0];
    for &(ch, top, bottom) in &drawn {
        assert!(
            (top - first_top).abs() < 1e-2 && (bottom - first_bottom).abs() < 1e-2,
            "'{ch}': the caret's edges must be letter-INDEPENDENT: \
             {top:.2}..{bottom:.2} vs {first_top:.2}..{first_bottom:.2} (all: {drawn:?})"
        );
    }

    // NON-VACUITY 1 — THE AXIS IS LIVE. The letters' own ink tops still differ
    // by several px on this very fixture, so the equality above is a property
    // of the caret and not of `lamgyA`.
    let ink_spread = ink_tops.iter().map(|&(_, t)| t).fold(f32::MIN, f32::max)
        - ink_tops.iter().map(|&(_, t)| t).fold(f32::MAX, f32::min);
    assert!(
        ink_spread > 5.0,
        "the per-glyph ink axis must still be live on this fixture or the \
         equality above proves nothing: spread={ink_spread:.2}px {ink_tops:?}"
    );

    assert_taller_than_the_retired_typical_box_and_the_tallest_ink_cell(
        &mut p,
        text,
        (first_top, first_bottom),
        &ink_cell_heights,
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// NON-VACUITY 2 AND 3 for the core law: the shipped height is provably NOT
/// the shape it replaced (the typical-letter box, for Block), and it reaches
/// every letter's own padded ink cell — CONTAINMENT restated as a single
/// height bound, so a future revert to the typical-letter box fails here BY
/// NAME (the mutation this file's law is proven against) rather than by drift.
fn assert_taller_than_the_retired_typical_box_and_the_tallest_ink_cell(
    p: &mut TextPipeline,
    text: &str,
    drawn: (f32, f32),
    ink_cell_heights: &[(char, f32)],
) {
    let (first_top, first_bottom) = drawn;
    let shipped_h = first_bottom - first_top;

    // AGAINST THE RETIRED TYPICAL-LETTER BOX — the shape item 451 replaced
    // for the literal Block caret. Re-derived the same way
    // `caret_cell_vertical_typical` builds it, from
    // `facepitch::typical_letter_ratio` rather than the ink envelope, so a
    // MUTATION that reinstates the typical box for Block (undoing this item)
    // makes `shipped_h` collapse onto `typical_h` and this assertion fails.
    p.set_view(&view(text, 0, 1)); // the 'a'
    p.settle_caret();
    let (_baseline, row_ascent, font) = p.caret_row_metrics();
    let pad = CARET_INK_PAD.px(pad_px(p));
    let typical_top = row_ascent * super::super::facepitch::typical_letter_ratio(font);
    let typical_h = typical_top + 2.0 * pad;
    assert!(
        shipped_h > typical_h + 2.0,
        "the Block envelope must be genuinely taller than the retired \
         typical-letter box, or item 451's fix has not landed: \
         shipped={shipped_h:.2} retired-typical={typical_h:.2}"
    );

    // AGAINST THE TALLEST PER-GLYPH INK CELL: the shipped ONE height must
    // reach at least the tallest letter's own tightly-fit padded ink cell in
    // the fixture — the per-letter containment asserted above, restated as a
    // single height bound.
    let ink_h_max = ink_cell_heights
        .iter()
        .map(|&(_, h)| h)
        .fold(f32::MIN, f32::max);
    assert!(
        shipped_h + 1e-2 >= ink_h_max,
        "the shipped Block height must reach the tallest per-glyph ink cell in \
         the fixture: shipped={shipped_h:.2} tallest-ink-cell={ink_h_max:.2}"
    );
    eprintln!(
        "Block ink-envelope height (Gumtree/Literata, zoom 1, dpi 1): \
         shipped={shipped_h:.2}px [retired typical-letter box {typical_h:.2}px, \
         tallest per-glyph ink cell {ink_h_max:.2}px]"
    );
}

/// THE FORM SWEEP (no-wildcard). Every caret LOOK is enumerated through
/// `CaretMode::ALL` and matched EXHAUSTIVELY — a new look added to the enum fails
/// to compile here, so it cannot silently pick its own vertical rule:
///
///   * `Block` draws the CELL form sized to the row's BLOCK INK ENVELOPE
///     (item 451) — the ascender/descender-covering box, never the typical
///     letter.
///   * `Morph` also draws the CELL form (its fast-travel deferral and its
///     ink-caret-world fold both land on the very same quad type as Block),
///     but keeps the OLD row's typical-letter envelope — item 451's explicit
///     scope: Morph does not inherit Block's taller body merely because the
///     geometry sits next to it. This arm is THE PIN for that scope clause: it
///     asserts Block and Morph draw genuinely DIFFERENT heights on the
///     identical anchor.
///   * `Ibeam` is the BAR form — an insertion bar marks the boundary BETWEEN
///     glyphs, so it deliberately spans the LINE BOX (`ibeam_bar_dims`) and must
///     be provably taller than even Block's now-larger cell.
#[test]
fn cell_caret_vertical_diverges_by_form_block_gets_the_envelope_morph_keeps_typical() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping cell_caret_vertical_diverges_by_form_block_gets_the_envelope_morph_keeps_typical: no wgpu adapter"
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
    let pad = CARET_INK_PAD.px(pad_px(&p));

    for mode in CaretMode::ALL {
        crate::caret::set_mode(mode);
        p.set_view(&view(text, 0, col));
        p.settle_caret();
        let ink = p
            .caret_anchor_ink_box()
            .expect("'m' must yield an ink box on Gumtree");
        let (baseline, row_ascent, font) = p.caret_row_metrics();
        let typical = row_ascent * super::super::facepitch::typical_letter_ratio(font);
        let (want_top_typical, want_bottom_typical) = (baseline - typical - pad, baseline + pad);

        let row_h = p.cursor_row_height();
        let (want_top_block, want_bottom_block) =
            want_block_top_bottom(baseline, row_ascent, font, row_h, pad);

        let (ink_top, ink_bottom) = (baseline - ink.top, baseline + ink.descent());

        match mode {
            CaretMode::Block => {
                let (cy, h) = p.caret_cell_vertical();
                assert!(
                    (cy - h * 0.5 - want_top_block).abs() < 1e-2
                        && (cy + h * 0.5 - want_bottom_block).abs() < 1e-2,
                    "Block: the CELL form must take its vertical from the row's \
                     BLOCK INK ENVELOPE: got {}..{} want {want_top_block}..{want_bottom_block}",
                    cy - h * 0.5,
                    cy + h * 0.5,
                );
                assert!(!p.caret_is_bar_form(), "Block: fixture must be the cell form here");
                assert!(
                    (want_bottom_block - want_top_block)
                        > (want_bottom_typical - want_top_typical) + 1.0,
                    "SCOPE NON-VACUITY: Block's envelope must be provably taller than \
                     the typical box Morph still uses below, or the two arms have \
                     nothing to diverge over: block={:.2} typical={:.2}",
                    want_bottom_block - want_top_block,
                    want_bottom_typical - want_top_typical,
                );
            }
            CaretMode::Morph => {
                let (cy, h) = p.caret_cell_vertical();
                assert!(
                    (cy - h * 0.5 - want_top_typical).abs() < 1e-2
                        && (cy + h * 0.5 - want_bottom_typical).abs() < 1e-2,
                    "Morph: the CELL form must KEEP the row's typical-letter box — \
                     item 451's scope is Block-only: got {}..{} want \
                     {want_top_typical}..{want_bottom_typical}",
                    cy - h * 0.5,
                    cy + h * 0.5,
                );
                assert!(!p.caret_is_bar_form(), "Morph: fixture must be the cell form here");
            }
            CaretMode::Ibeam => {
                assert!(p.caret_is_bar_form(), "Ibeam must be the bar form");
                // The bar AS DRAWN at rest (settle 1) — its own line-box geometry.
                let (_bx, _by, _bw, tall, _bc) = p.caret_ibeam_geometry();
                assert!(
                    (tall - p.metrics.caret_h * p.cursor_scale()).abs() < 1e-3,
                    "Ibeam must span the LINE BOX, not the ink box: tall={tall}"
                );
                // NON-VACUITY: I-beam's own LINE-BOX height must be a
                // genuinely DIFFERENT number from Block's cell height, proving
                // it is not silently reading the same formula — NOT
                // necessarily taller, since item 451's Block envelope can now
                // exceed the I-beam's fixed line-box constant on a tight face
                // at body size (`block_envelope_never_touches_the_adjacent_row`
                // measures that margin directly; this arm only needs the two
                // numbers to differ).
                assert!(
                    (tall - (want_bottom_block - want_top_block)).abs() > 0.5,
                    "the I-beam bar must draw a genuinely DIFFERENT height from \
                     Block's own cell, whichever is taller (so this arm stays \
                     non-vacuous): ibeam={tall} block-cell={} \
                     (the anchored ink it does not read: {:.2})",
                    want_bottom_block - want_top_block,
                    ink_bottom - ink_top
                );
            }
        }
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// Punctuation is horizontally ink-hugging but vertically an
/// insertion point in the same ordinary-letter band as its row.  This is the
/// reported comparison the earlier floor law omitted: a period is not judged
/// against its own tiny ink, but against a letter caret on the SAME row.
///
/// Every world, form, DPI, and the two anchoring shapes are covered.  Block
/// and Morph each put their anchor on the period by their real conventions;
/// I-beam deliberately remains a full line bar, and therefore must be equal
/// on both columns too. The bound is the measured adjacent
/// glyph-to-glyph seam budget, with genuinely short ink held tighter.
fn assert_punctuation_cell(
    p: &mut TextPipeline,
    world: &str,
    is_mono: bool,
    dpi: f32,
    mode: CaretMode,
    ch: char,
) -> bool {
    let text = format!("a{ch}a");
    let (letter_col, punctuation_col) = match mode {
        CaretMode::Block | CaretMode::Ibeam => (0, 1),
        CaretMode::Morph => (1, 2),
    };
    p.set_view(&view(&text, 0, letter_col));
    p.settle_caret();
    let (_, _, _, letter_h, ..) = p.caret_geometry();
    p.set_view(&view(&text, 0, punctuation_col));
    p.settle_caret();
    let (_, _, _, punctuation_h, ..) = p.caret_geometry();
    let bound = 14.0 * p.metrics.scale;
    assert!(
        (punctuation_h - letter_h).abs() <= bound,
        "{world} {mode:?} {ch:?} dpi={dpi}: punctuation height {punctuation_h:.2} \
         differs from its row letter {letter_h:.2} beyond {bound:.2}px"
    );

    let short_ink = if is_mono {
        false
    } else {
        let ink = p
            .caret_anchor_ink_box()
            .expect("proportional punctuation ink");
        let (_, ascent, font) = p.caret_row_metrics();
        let short = ink.height < ascent * super::super::facepitch::typical_letter_ratio(font);
        if short {
            let tight = 6.0 * p.metrics.scale;
            assert!(
                (punctuation_h - letter_h).abs() <= tight,
                "{world} {mode:?} {ch:?} dpi={dpi}: short punctuation height {punctuation_h:.2} \
                 differs from its row x-height letter {letter_h:.2} beyond {tight:.2}px"
            );
        }
        short
    };

    if mode == CaretMode::Block {
        let eol = format!("a{ch}");
        p.set_view(&view(&eol, 0, 1));
        p.settle_caret();
        let (_, _, _, punctuation_h, ..) = p.caret_geometry();
        p.set_view(&view(&eol, 0, 2));
        p.settle_caret();
        let (_, _, _, eol_h, ..) = p.caret_geometry();
        assert!(
            (eol_h - punctuation_h).abs() <= bound,
            "{world} {mode:?} {ch:?} dpi={dpi}: punctuation->EOL height seam \
             {punctuation_h:.2}->{eol_h:.2} exceeds {bound:.2}px"
        );
    }
    short_ink
}

#[test]
fn punctuation_uses_the_rows_letter_height_across_forms_worlds_dpi_and_anchors() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping punctuation height law: no wgpu adapter");
        return;
    };
    let punct = [',', '.', ':', '-', '(', '[', '。'];
    let mut proportional = false;
    let mut mono = false;
    let mut raw_shorter_than_band = false;

    for world in theme::THEMES {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        let is_mono = crate::caret::font_is_mono(p.shaped_font);
        // dpi 1.5 is a real fractional-scaling factor no capture ever runs at
        // (`caret_scale_law.rs`'s own blind spot) — enrolled here alongside the
        // two exact factors so this oracle exercises the axis the fix is for.
        for dpi in [1.0, 1.5, 2.0] {
            p.set_dpi(dpi);
            for mode in CaretMode::ALL {
                crate::caret::set_mode(mode);
                for ch in punct {
                    if is_mono {
                        mono = true;
                    } else {
                        proportional = true;
                    }
                    raw_shorter_than_band |=
                        assert_punctuation_cell(&mut p, world.name, is_mono, dpi, mode, ch);
                }
            }
        }
    }
    assert!(proportional, "the proportional roster must be exercised");
    assert!(mono, "the mono roster must be exercised");
    assert!(
        raw_shorter_than_band,
        "non-vacuity: no punctuation ink was shorter than its row x-height band"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// THE MONO GRID IS UNTOUCHED. A monospace display exists to look perfectly
/// uniform, so `caret_anchor_ink_box` returns `None` there and the cell caret keeps
/// the historical row-scaled `caret_block_h` line cell: the TOP is the SAME y at
/// every column (no per-glyph wobble), the height is exactly the line cell on a
/// non-dipper, and the ONLY variation is the descender extension dropping the
/// BOTTOM for a real dipper — byte-identical to the compact geometry, which
/// applied that extension at the draw site.
///
/// The roster sweep widened this from the hand-listed pair `["Tawny", "Mangrove"]` to a
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
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
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

/// THE OTHER HALF OF THE SWEEP, over the WHOLE proportional roster and BOTH
/// DPIs: the caret's top is the SAME y at every column there too, exactly as
/// the mono grid's is — the two arms differ in what sets the height, never in
/// whether it holds still.
///
/// This is the complement of `mono_world_caret_grid_stays_uniform_and_line_box_sized`
/// over the SAME roster split, so between them every world in `theme::THEMES` is
/// asserted to be in exactly the arm its face's own advance widths put it in;
/// a widened `font_is_mono` that mistook a near-gridded face (iA Writer Quattro
/// S is bundled and duospaced) for a mono still fails the arm assertion here.
///
/// NON-VACUITY, per world and per DPI: the letters' own ink tops must still
/// spread by several px. A uniformity law over a set that happens to be uniform
/// tests nothing, and this fixture's uniformity is exactly what the caret is
/// being asked NOT to have.
///
/// Also asserts item 451's CONTAINMENT law directly, over the same sweep: the
/// caret's one top/bottom must sit AT OR BEYOND every letter's own real ink,
/// on every proportional world and both DPIs — an ascender's ink may never
/// rise above the drawn top, a descender's may never sink below the drawn
/// bottom.
#[test]
fn proportional_worlds_take_one_caret_top_at_every_letter() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping proportional_worlds_take_one_caret_top_at_every_letter: no wgpu adapter"
        );
        return;
    };
    let text = "lamgyA";
    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;
    let mut tightest_top_margin = f32::MAX;
    let mut tightest_bottom_margin = f32::MAX;
    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
            theme::set_active_by_name(t.name).unwrap();
            p.sync_theme();
            let mut tops: Vec<f32> = Vec::new();
            let mut ink_tops: Vec<f32> = Vec::new();
            let mut ink_bottoms: Vec<f32> = Vec::new();
            let (mut caret_top, mut caret_bottom) = (0.0f32, 0.0f32);
            for (col, ch) in text.chars().enumerate() {
                p.set_view(&view(text, 0, col));
                p.settle_caret();
                let ink = p.caret_anchor_ink_box().unwrap_or_else(|| {
                    panic!(
                        "{} ({}) d{dpi}: a proportional world must read real ink for '{ch}'",
                        t.name, t.font
                    )
                });
                let (cy, h) = p.caret_cell_vertical();
                let baseline = p.caret_row_metrics().0;
                let ink_top = baseline - ink.top;
                let ink_bottom = baseline + ink.descent();
                caret_top = cy - h * 0.5;
                caret_bottom = cy + h * 0.5;
                tops.push(caret_top);
                ink_tops.push(ink_top);
                ink_bottoms.push(ink_bottom);
                // CONTAINMENT, per letter: item 451's actual law. An ascender's
                // ink top must not rise above the caret's own top, a
                // descender's ink bottom must not sink below the caret's own
                // bottom.
                assert!(
                    ink_top >= caret_top - 1e-2,
                    "{} ({}) d{dpi} '{ch}': ink top {ink_top:.2} rises above the \
                     caret's own top {caret_top:.2} — an ascender is escaping the \
                     accent body",
                    t.name,
                    t.font
                );
                assert!(
                    ink_bottom <= caret_bottom + 1e-2,
                    "{} ({}) d{dpi} '{ch}': ink bottom {ink_bottom:.2} sinks below \
                     the caret's own bottom {caret_bottom:.2} — a descender is \
                     escaping the accent body",
                    t.name,
                    t.font
                );
            }
            let spread = |v: &[f32]| {
                v.iter().cloned().fold(f32::MIN, f32::max)
                    - v.iter().cloned().fold(f32::MAX, f32::min)
            };
            // NON-VACUITY: the axis the caret ignores is live on this world.
            // The floor sits under the roster's TIGHTEST real value rather than
            // at a round number — Quokka (Sour Gummy) measures exactly 4.00px
            // at DPI 1, the smallest ascender-to-x-height ink spread any
            // bundled display face shows.
            let ink_spread = spread(&ink_tops);
            assert!(
                ink_spread >= 3.0 * dpi,
                "{} ({}) d{dpi}: the per-glyph ink tops must genuinely differ or \
                 the uniformity below proves nothing (spread {ink_spread:.2})",
                t.name,
                t.font
            );
            // THE LAW: one top y for every column, ink spread notwithstanding.
            let top_spread = spread(&tops);
            assert!(
                top_spread < 1e-2,
                "{} ({}) d{dpi}: the caret top must be identical at every column \
                 (spread {top_spread:.3} over {tops:?}, against an ink spread of \
                 {ink_spread:.2})",
                t.name,
                t.font
            );
            // Slack, in device-independent px: how far the caret's own edge sits
            // BEYOND the worst-case letter's ink — non-negative when containment
            // holds, and this is where the CONTAINMENT assertions above would
            // have fired had it not.
            let top_margin =
                (ink_tops.iter().cloned().fold(f32::MAX, f32::min) - caret_top) / dpi;
            let bottom_margin =
                (caret_bottom - ink_bottoms.iter().cloned().fold(f32::MIN, f32::max)) / dpi;
            tightest_top_margin = tightest_top_margin.min(top_margin);
            tightest_bottom_margin = tightest_bottom_margin.min(bottom_margin);
            checked += 1;
        }
    }
    p.set_dpi(1.0);
    assert!(
        checked >= 22,
        "every proportional-display world is swept at both DPIs (got {checked})"
    );
    eprintln!(
        "containment across the proportional roster × both DPIs: tightest top margin \
         {tightest_top_margin:.2}px, tightest bottom margin {tightest_bottom_margin:.2}px \
         (both must be >= 0)"
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
/// half is `cell_caret_takes_one_typical_letter_height_across_every_letter_class`.
#[test]
fn moving_caret_streak_is_unaffected_by_the_ink_box() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
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

/// THE GLYPHLESS FALLBACKS SURVIVE. There is no ink to
/// hug at a SPACE, at END-OF-LINE, or on an EMPTY line, so the ONE ink funnel
/// returns `None`. On a MONO world the cell owner still falls back to the
/// historical row-scaled `caret_block_h` centred on the spring anchor
/// (the uniform grid, byte-identical). On a PROPORTIONAL world it no
/// longer does: THIS exact invariant — the fallback
/// pinned to `caret.pos.y` (a row-box-geometric-centre convention) — was the
/// root cause of a visible cell jump the instant a proportional caret left a
/// real glyph for an adjacent glyphless column (the user's `aaa`->EOL report;
/// see `render/tests/caret_transition.rs`). The fallback now reads a
/// SYNTHETIC typical-letter box through the SAME baseline-relative formula the
/// ink-box arm above uses, so this test pins the NEW formula directly rather
/// than re-asserting the convention that caused the bug. The space bar and
/// Morph's line-start bar-form mechanics (not their SIZE — see below) are
/// otherwise unchanged.
#[test]
fn glyphless_fallbacks_use_the_synthetic_baseline_box_on_proportional_worlds() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
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
    let pad = CARET_INK_PAD.px(pad_px(&p));

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
        // the adjacent-column transition actually changed this seam rather than leaving it inert.
        let old_cy = p.caret.pos.y;
        let old_h = p.metrics.caret_block_h * p.cursor_scale();
        assert!(
            (cy - old_cy).abs() > 0.5 || (h - old_h).abs() > 0.5,
            "{what}: the proportional fallback must no longer equal the old \
             row-box-centred cell (cy={cy} h={h} old_cy={old_cy} old_h={old_h})"
        );
        // The height must still be a small, positive, letter-plausible cell —
        // never collapsed, never the former oversized fixed cap.
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
        (bw - p.metrics.px(CARET_SPACE_BAR_W)).abs() < 1e-3,
        "the space bar stays the slim bar: bw={bw}"
    );

    // MORPH's LINE-START degrade: the I-beam's own bar, line-box tall — still
    // untouched by the ink-box mechanism (a bar has no glyph of its own to
    // hug), unchanged by the adjacent-column transition.
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
    // the prior line-cell — the uniform grid never reads any ink box,
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

/// THE PAD IS BOUNDED. The ink box needs room around the true-weight Filled
/// knockout and the ordinary Morph silhouette alike, but its own job is simply to
/// remain a small, letter-independent margin rather than a second line cell.
///
/// `CARET_INK_PAD` is SMALL relative to the line cell (a pad, not a second
/// cell height) — otherwise "hug the ink" would drift back into "cover the row".
#[test]
fn caret_ink_pad_is_bounded() {
    assert!(
        std::hint::black_box(CARET_INK_PAD.0) > 0.0 && CARET_INK_PAD.0 < CARET_BLOCK_H * 0.25,
        "the ink pad must stay a small margin, not a second cell height: {}",
        CARET_INK_PAD.0
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

/// THE "CARET IS TOO SHORT ON TALL ROWS" CLAIM, CONFIRMED FALSE BY CAPTURE.
///
/// User-reported with a screenshot; not the same axis as the OTHER punctuation
/// ink-box fix above (glyph shape, not row height). A source read of
/// `render.rs`'s `caret_h: CARET_H * s` (zoom/DPI only, no row term) matched the
/// reported shape exactly, but `caret_h` alone is never the FINAL drawn height:
/// every consumer corrects it before a pixel lands — the CELL form through
/// `caret_cell_vertical`'s ink-box arm (this file's laws, above), the
/// BAR/mono-cell forms through `cursor_scale()`'s `row_height / line_height`
/// multiply (`ibeam_bar_dims`, and the mono arm of `caret_cell_vertical` itself).
///
/// A capture sweep (screenshots + pixel arithmetic) covered every heading
/// level, mono AND proportional worlds, Block/Morph/Ibeam, 1x/2x DPI, zoom
/// 1.0/1.5, a wrapped heading's continuation row, a titleless heading's
/// fully-glyphless synthetic fallback, a freshly-`Enter`ed blank line, list
/// items/fences/blockquotes/thematic breaks — and found no case where the
/// caret undershoots. The ink-box CELL form tracks the row's FONT SIZE (the
/// quantity a caret should track — the row's cap/ascender box, not its full
/// height including leading); the bar/mono-cell forms track the row's fuller
/// height (font size × the heading ladder's own row-height lead), if anything
/// OVERSHOOTING. This law and the two below it pin that confirmed-correct
/// outcome — nothing previously measured the CARET's own height against its
/// row across heading levels (only the ROW's height was pinned, by
/// `heading_rows_are_taller_and_gated_to_markdown` in `markdown_headings.rs`)
/// — so a real future regression toward the bare constant fails here, by name,
/// instead of shipping unnoticed a second time.
///
/// Swept at 1x/2x DPI directly here (not just in the manual capture sweep):
/// a bare `CARET_H * zoom * dpi` constant is exactly the shape that ships
/// correct at one tier and wrong at another, and every OTHER capture in this
/// repo runs at DPI 1 by default. The per-DPI thresholds are a floor, not an
/// exact-ratio pin — glyph rasterization is inherently integer-pixel
/// quantized (confirmed separately: caret-height/ink-height holds within
/// 0.2% across DPI 1x/2x at a large zoom, where quantization noise is
/// diluted, so the residual few-percent drift in caret/body-caret ratios at
/// small sizes is glyph-raster rounding, not a scaling defect) — an exact
/// match would make the law flaky on that rounding, not more correct.
#[test]
fn heading_cell_caret_grows_with_the_headings_own_font_size_not_the_bare_row_constant() {
    // Ink-box lookup folds the theme font AND the page wrap globals; the anchor
    // is mode-keyed. Hold theme -> page -> caret (the suite-wide order), pin
    // MORPH so a real letter anchors the CELL/ink-box arm.
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Morph);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping heading cell caret font-size law: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Gumtree").unwrap(); // proportional (Literata)
    p.sync_theme();

    for dpi in [1.0_f32, 2.0] {
        p.set_dpi(dpi);
        // One real lowercase letter `x` per line, at increasing heading depth.
        // Morph anchors ONE COLUMN BACK, so parking the cursor right after `x`
        // anchors the caret ON it — the ink-box CELL form, never a glyphless
        // fallback.
        let mut heights = Vec::new();
        for level in 0..=3u8 {
            let text = format!("{} x", "#".repeat(level as usize));
            let mut v = view(&text, 0, text.len());
            v.is_markdown = true;
            p.set_view(&v);
            p.settle_caret();
            let (_cx, _cy, _w, h, ..) = p.caret_geometry();
            heights.push(h);
        }
        let body = heights[0];
        let (h1, h2, h3) = (heights[1], heights[2], heights[3]);
        assert!(
            h1 > h2 && h2 > h3 && h3 > body,
            "dpi={dpi}: heading depth must grow the cell caret monotonically: \
             body={body} h3={h3} h2={h2} h1={h1}"
        );
        // Thresholds sit strictly between "stuck at body" (ratio 1.0 — the bug)
        // and the ladder's own SIZE rung (`heading_scale`), so raster rounding
        // cannot make the law flaky; a regression to the bare constant fails
        // all three, at both DPI tiers.
        for (level, h, min_ratio) in [(1u8, h1, 1.30_f32), (2, h2, 1.15), (3, h3, 1.05)] {
            let ratio = h / body;
            assert!(
                ratio > min_ratio,
                "dpi={dpi}: h{level} cell caret must clear {min_ratio}x the body \
                 caret's height (tracking heading_scale({level})={}), got {ratio} \
                 ({h}/{body})",
                crate::markdown::heading_scale(level)
            );
        }
    }

    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// The MONO-WORLD complement: a mono world's uniform glyph grid never reads an
/// ink box (every column shares one row-scaled line cell), so the whole caret
/// depends on `cursor_scale()` alone to track the row. Proves that arm ALSO
/// clears the bug — a regression here would be invisible to the proportional
/// law above. Swept at 1x/2x DPI like its sibling; unlike the ink-box form
/// this one reads no glyph raster at all (`caret_block_h * cursor_scale()` is
/// pure float arithmetic over row geometry), so its ratio holds tight at
/// every DPI with no rounding slack needed.
#[test]
fn heading_line_cell_caret_on_a_mono_world_also_tracks_the_row_not_the_bare_constant() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping heading mono line-cell caret law: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Tawny").unwrap(); // mono (IBM Plex Mono)
    p.sync_theme();

    for dpi in [1.0_f32, 2.0] {
        p.set_dpi(dpi);
        let mut heights = Vec::new();
        for level in 0..=3u8 {
            let text = format!("{} x", "#".repeat(level as usize));
            let mut v = view(&text, 0, text.len());
            v.is_markdown = true;
            p.set_view(&v);
            p.settle_caret();
            let (_cx, _cy, _w, h, ..) = p.caret_geometry();
            heights.push(h);
        }
        let body = heights[0];
        let (h1, h2, h3) = (heights[1], heights[2], heights[3]);
        assert!(
            h1 > h2 && h2 > h3 && h3 > body,
            "dpi={dpi}: the mono LINE-CELL caret (a uniform grid, no ink box) must \
             still grow with cursor_scale()'s row_height/line_height ratio: \
             body={body} h3={h3} h2={h2} h1={h1}"
        );
        // No additive pad complicates the mono arm (`caret_block_h * cursor_scale()`,
        // `x` has no descender to extend), so this ratio can be pinned tightly
        // against the SAME formula `cursor_scale` reads from, at both DPI tiers.
        for (level, h) in [(1u8, h1), (2, h2), (3, h3)] {
            let scale = crate::markdown::heading_scale(level);
            let lead = crate::markdown::heading_row_lead(level);
            let want = scale * lead;
            let ratio = h / body;
            assert!(
                (ratio - want).abs() < 0.05,
                "dpi={dpi}: h{level} mono cell caret should be ~{want}x the body \
                 caret (cursor_scale tracks the FULL row incl. the ladder's own \
                 lead), got {ratio}"
            );
        }
    }

    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// A tempting (but false) alternate diagnosis for "the caret is too short on
/// tall rows": a blank row sitting directly under a heading, inflated by the
/// heading's own "space above" bleeding onto its neighbor. That doesn't happen
/// on `main` — `heading_row_lead` and `md_line_scale` key off a line's OWN
/// leading `#` run (see their doc comments in `markdown/headings.rs`), so a
/// plain blank line next to a heading is untouched; a decoupled-space-above
/// experiment that would have produced exactly that inflation never landed.
/// Pinned directly so a future "fix" for a caret that never undershot cannot
/// reintroduce the inflation it would actually need to guard against.
#[test]
fn blank_row_directly_below_a_heading_stays_body_height() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping blank row under heading law: no wgpu adapter");
        return;
    };
    let text = "# Heading\n\nBody one\nBody two\n";
    let mut v = view(text, 0, 0);
    v.is_markdown = true;
    p.set_view(&v);
    let heading_row = p.row_height_px(0);
    let blank_row_below_heading = p.row_height_px(1);
    let ordinary_body_row = p.row_height_px(2);
    assert!(
        heading_row > blank_row_below_heading,
        "sanity: the heading row must actually be taller than its neighbor \
         (else this fixture proves nothing): heading={heading_row} blank={blank_row_below_heading}"
    );
    assert!(
        (blank_row_below_heading - ordinary_body_row).abs() < 0.01,
        "a blank line directly under a heading must be exactly body height, not \
         inflated by the heading's own row-height lead: blank={blank_row_below_heading} \
         body={ordinary_body_row}"
    );
}

/// THE BLOCK ENVELOPE NEVER TOUCHES THE ADJACENT ROW — item 451's other
/// explicit requirement, checked GEOMETRICALLY (against the row's own real
/// `cursor_row_height()`, which folds the heading ladder in) across the whole
/// proportional roster, both DPIs, and every heading level.
///
/// This is the axis that made the ink envelope's first cut wrong: the roster's
/// tightest bundled face (Bitter — Mopoke/Magpie) has real ink extremes tall
/// enough that the envelope plus both full [`CARET_INK_PAD`]s overshot the
/// row's own fixed line height by a fraction of a px at body size, DPI 1 —
/// `caret_cell_vertical_block`'s pad-shrinking floor exists to close exactly
/// that gap. NON-VACUITY: the worst margin across the whole sweep is asserted
/// to land close to the floor's own 1-logical-px clearance (proving the floor
/// is genuinely the binding constraint on the tightest face, not slack
/// nobody needed) while never going negative (the actual law).
#[test]
fn block_envelope_never_touches_the_adjacent_row() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping block_envelope_never_touches_the_adjacent_row: no wgpu adapter");
        return;
    };
    let mono = super::facepitch::mono_display_worlds();
    let mut worst_margin = f32::MAX;
    let mut worst_world = "";
    let mut checked = 0usize;
    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
            theme::set_active_by_name(t.name).unwrap();
            p.sync_theme();
            for level in 0..=3u8 {
                let text = format!("{} m", "#".repeat(level as usize));
                let mut v = view(&text, 0, text.len());
                v.is_markdown = true;
                p.set_view(&v);
                p.settle_caret();
                let (_cy, h) = p.caret_cell_vertical();
                let row_h = p.cursor_row_height();
                let margin = row_h - h;
                assert!(
                    margin >= -1e-2,
                    "{} ({}) d{dpi} h{level}: the Block envelope ({h:.2}px) must \
                     never exceed the row's own line height ({row_h:.2}px) — it \
                     is reaching into the row above or below (margin={margin:.2})",
                    t.name,
                    t.font
                );
                if margin < worst_margin {
                    worst_margin = margin;
                    worst_world = t.name;
                }
                checked += 1;
            }
        }
    }
    assert!(
        checked >= 88,
        "every proportional-display world is swept at both DPIs and every \
         heading level (got {checked})"
    );
    // NON-VACUITY: the tightest margin must sit CLOSE to the floor's own
    // clearance (a couple of device px, not the wide heading-row margins the
    // sweep otherwise shows) — proving the pad-shrinking floor is actually
    // load-bearing on the roster's tightest face, not decoration nobody needed.
    assert!(
        worst_margin < 4.0,
        "the tightest (world, dpi, heading-level) margin must land close to the \
         floor's own clearance or the floor is not the binding constraint \
         anywhere in the roster: worst={worst_margin:.2}px at {worst_world}"
    );
    eprintln!(
        "Block envelope vs. row height across the proportional roster × both \
         DPIs × every heading level: tightest margin {worst_margin:.2}px at \
         {worst_world}"
    );

    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}
