//! THE CARET'S ONE HEIGHT (VERTICAL). Laws for the rule that gives the
//! CELL-form caret ONE height per (face, row) on a proportional world, taken
//! from the row's TYPICAL LETTER and never from the anchored glyph.
//!
//! **TWO SHAPES ARE WRONG AND THE SHIPPED ONE SITS BETWEEN THEM. The history
//! is the whole point of this file, because each of the two failures was a
//! deliberate fix for the other.**
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
//!   * SHIPPED: the row's typical letter (`facepitch::typical_letter_ratio` —
//!     the shipped face's own measured mean of x-height and cap-height, times
//!     the row's own real `max_ascent`), padded. One height for every anchor on
//!     the row, close enough to the letters actually being typed that the
//!     row-fraction's dead space never comes back.
//!
//! `TextPipeline::caret_cell_vertical` is the one owner and now has TWO arms,
//! split by `crate::caret::font_is_mono` alone: the typical-letter box on a
//! proportional world, the row-scaled line cell (descender extension folded in)
//! on a mono one. These laws pin, with glyph-mask arithmetic against the real
//! raster placement:
//!
//!   * settled: the caret's top and bottom are the SAME two numbers for an
//!     ascender, an x-height letter, a capital and two descenders — while those
//!     letters' own ink boxes provably still differ (the axis the caret no
//!     longer follows must stay LIVE in the fixture, or "one height" is a claim
//!     about the fixture rather than about the caret);
//!   * that the one height is neither of the two wrong shapes: strictly taller
//!     than the shortest letter's ink cell, strictly shorter than the row cell,
//!     with the pre-91 dead space above an `a` measured and reported;
//!   * moving: the travelling streak is untouched;
//!   * mono: the uniform grid is byte-identical (a fixed top, a bottom that drops
//!     only for a real dipper);
//!   * every caret FORM is swept by a no-wildcard match, so a new `CaretMode`
//!     cannot dodge the vertical policy;
//!   * the glyphless space / end-of-line / bar cases read the SAME box as the
//!     letters (`layers.rs` holds NO vertical caret geometry of its own — the
//!     grep-law that bans a second rule from growing back).
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

/// THE CORE LAW. On a PROPORTIONAL world the settled cell caret's TOP and
/// BOTTOM are ONE PAIR OF NUMBERS for the whole row: the row's typical-letter
/// box grown by one [`CARET_INK_PAD`], identical on an ascender, an x-height
/// letter, a capital and a descender. The formula is RE-DERIVED here from
/// `facepitch::typical_letter_ratio` and the row's own metrics rather than read
/// back out of the owner, so this is a law about the rule and not a restatement
/// of the code.
///
/// NON-VACUOUS THREE WAYS, and the first is the one that matters: an equality
/// law over a set is satisfiable by the SET being uniform, so this asserts the
/// letters' OWN raster ink boxes still differ across the fixture by several px
/// — the axis the caret deliberately ignores has to be live, or "one height" is
/// a fact about `lamgy` rather than about the caret. Then both discarded shapes
/// are measured on the same fixture and shown to be genuinely different
/// numbers: the pre-91 row cell (whose dead space above an `a` is the reported
/// 8–9px, asserted as a fixture witness) and the per-glyph ink cell (whose
/// spread between `a` and `l` is what the user called distracting).
#[test]
fn cell_caret_takes_one_typical_letter_height_across_every_letter_class() {
    // Ink-box lookup folds the theme font AND the page wrap globals; the anchor is
    // mode-keyed. Hold theme -> page -> caret (the suite-wide order), pin BLOCK.
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping the one-typical-letter-height law: no wgpu adapter");
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
        // THE RULE, re-derived from the face's own measured ratio rather than
        // read back from `caret_cell_vertical`.
        let typical = row_ascent * super::super::facepitch::typical_letter_ratio(font);
        let want_top = baseline - typical - pad;
        let want_bottom = baseline + pad;

        let (top, bottom) = caret_top_bottom(&mut p);
        assert!(
            (top - want_top).abs() < 1e-2,
            "'{ch}': caret top must be the row's TYPICAL-letter top minus one pad, \
             whatever letter is anchored: top={top} want={want_top} pad={pad}"
        );
        assert!(
            (bottom - want_bottom).abs() < 1e-2,
            "'{ch}': caret bottom must be the baseline plus one pad: \
             bottom={bottom} want={want_bottom} pad={pad}"
        );

        // THE DESCENDER DECISION, asserted rather than left to a comment: a
        // dipping letter's ink deliberately passes BELOW the caret's bottom pad,
        // because extending for it would be a per-glyph rule on the bottom edge
        // — the same jump the top edge was just relieved of. A non-dipper stays
        // inside. (No knockout world is affected: every world that punches the
        // letter out of its caret is mono and keeps the row cell.)
        let dips = ink.descent() > 2.0;
        assert_eq!(
            dips,
            ink_bottom > bottom,
            "'{ch}': only a real dipper may pass below the caret's bottom \
             (ink_bottom={ink_bottom:.2} caret_bottom={bottom:.2} \
             descent={:.2})",
            ink.descent()
        );

        drawn.push((ch, top, bottom));
        ink_tops.push((ch, ink_top));
        // What the PER-GLYPH rule would have drawn for this letter: its own ink
        // box, padded — the shape the user reported as distracting.
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

    assert_between_the_two_discarded_shapes(
        &mut p,
        text,
        (first_top, first_bottom),
        &ink_cell_heights,
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// NON-VACUITY 2 AND 3 for the core law: the one height is NEITHER of the two
/// shapes it replaced. Against the ROW-FRACTION cell, the dead accent it hangs
/// above an `a`'s ink must be genuinely relieved; against the PER-GLYPH ink
/// cell, the shipped height must sit strictly between the shortest and tallest
/// letter's own cell — which is what "somewhere in between" has to mean if it
/// is to be a law rather than a preference.
fn assert_between_the_two_discarded_shapes(
    p: &mut TextPipeline,
    text: &str,
    drawn: (f32, f32),
    ink_cell_heights: &[(char, f32)],
) {
    let (first_top, first_bottom) = drawn;
    // AGAINST THE ROW FRACTION. The row-fraction cell centred on
    // the spring anchor hangs the reported 8-9px of dead accent above an `a`'s
    // ink; the shipped top must clear that gap by a real margin, or the revert
    // has simply reinstated the row-fraction shape.
    p.set_view(&view(text, 0, 1)); // the 'a'
    p.settle_caret();
    let ink_a = p.caret_anchor_ink_box().expect("'a' ink");
    let baseline_a = p.caret_row_metrics().0;
    let ink_top_a = baseline_a - ink_a.top;
    let row_cell_top = p.caret.pos.y - (p.metrics.caret_block_h * p.cursor_scale()) * 0.5;
    let row_dead = ink_top_a - row_cell_top;
    let shipped_dead = ink_top_a - first_top;
    assert!(
        row_dead > 5.0,
        "fixture must reproduce the pre-91 dead space above 'a' (row cell sat \
         only {row_dead:.2}px above the ink)"
    );
    assert!(
        shipped_dead < row_dead - 2.0,
        "the one height must sit well INSIDE the pre-91 row cell above an 'a': \
         shipped gap={shipped_dead:.2}px vs row-cell gap={row_dead:.2}px"
    );

    // AGAINST THE PER-GLYPH INK CELL: strictly taller than
    // the shortest letter's own cell, strictly shorter than the tallest's. The
    // "somewhere in between" the reversal actually asked for.
    let shipped_h = first_bottom - first_top;
    let ink_h_min = ink_cell_heights
        .iter()
        .map(|&(_, h)| h)
        .fold(f32::MAX, f32::min);
    let ink_h_max = ink_cell_heights
        .iter()
        .map(|&(_, h)| h)
        .fold(f32::MIN, f32::max);
    let row_h = p.metrics.caret_block_h * p.cursor_scale();
    assert!(
        ink_h_min < shipped_h && shipped_h < ink_h_max && shipped_h < row_h,
        "the one height must sit between the shortest and tallest per-glyph ink \
         cells and under the row cell: shipped={shipped_h:.2} ink=[{ink_h_min:.2}, \
         {ink_h_max:.2}] row={row_h:.2}"
    );
    eprintln!(
        "one caret height (Gumtree/Literata, zoom 1, dpi 1): shipped={shipped_h:.2}px \
         [per-glyph ink cells {ink_h_min:.2}..{ink_h_max:.2}px, pre-91 row cell \
         {row_h:.2}px]; dead space above 'a''s ink: shipped={shipped_dead:.2}px vs \
         pre-91 {row_dead:.2}px"
    );
}

/// THE FORM SWEEP (no-wildcard). Every caret LOOK is enumerated through
/// `CaretMode::ALL` and matched EXHAUSTIVELY — a new look added to the enum fails
/// to compile here, so it cannot silently pick its own vertical rule:
///
///   * `Block` / `Morph` draw the CELL form (Morph's fast-travel deferral and its
///     ink-caret-world fold both land on the very same quad), so their vertical
///     bounds are the row's one typical-letter envelope.
///   * `Ibeam` is the BAR form — an insertion bar marks the boundary BETWEEN
///     glyphs, so it deliberately spans the LINE BOX (`ibeam_bar_dims`) and must
///     be provably taller than the cell.
#[test]
fn cell_caret_vertical_has_one_owner_across_every_caret_form() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
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
        let (want_top, want_bottom) = (baseline - typical - pad, baseline + pad);
        let (ink_top, ink_bottom) = (baseline - ink.top, baseline + ink.descent());

        match mode {
            CaretMode::Block | CaretMode::Morph => {
                let (cy, h) = p.caret_cell_vertical();
                assert!(
                    (cy - h * 0.5 - want_top).abs() < 1e-2
                        && (cy + h * 0.5 - want_bottom).abs() < 1e-2,
                    "{mode:?}: the CELL form must take its vertical from the row's \
                     typical letter: got {}..{} want {want_top}..{want_bottom}",
                    cy - h * 0.5,
                    cy + h * 0.5,
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
                    tall > (want_bottom - want_top) + 1.0,
                    "the I-beam bar must be provably TALLER than the cell form's own \
                     height (so this arm is non-vacuous): tall={tall} cell={} \
                     (the anchored ink it does not read: {:.2})",
                    want_bottom - want_top,
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
        eprintln!("skipping item-205 punctuation height law: no wgpu adapter");
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
/// BOTTOM for a real dipper — byte-identical to the pre-item-91 geometry, which
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
    let mut worst_dead = 0.0f32;
    let mut tightest_relief = f32::MAX;
    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
            theme::set_active_by_name(t.name).unwrap();
            p.sync_theme();
            let mut tops: Vec<f32> = Vec::new();
            let mut ink_tops: Vec<f32> = Vec::new();
            // The pre-91 shape's own dead space above the x-height `a`, and the
            // shipped one's, on this world at this DPI.
            let (mut row_dead, mut shipped_dead) = (0.0f32, 0.0f32);
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
                let ink_top = p.caret_row_metrics().0 - ink.top;
                tops.push(cy - h * 0.5);
                ink_tops.push(ink_top);
                if ch == 'a' {
                    row_dead = ink_top
                        - (p.caret.pos.y - (p.metrics.caret_block_h * p.cursor_scale()) * 0.5);
                    shipped_dead = ink_top - (cy - h * 0.5);
                }
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
            // NOT A BARE REVERT, on every world and both DPIs: the row
            // fraction's own measured defect was its dead accent above an
            // `a`'s ink. The shipped top must sit well inside it — a fixed
            // height that reproduced that gap would be the discarded shape.
            assert!(
                row_dead > 4.0 * dpi,
                "{} ({}) d{dpi}: fixture must reproduce the pre-91 dead space \
                 above 'a' (row cell sat {row_dead:.2}px above the ink)",
                t.name,
                t.font
            );
            assert!(
                shipped_dead < row_dead - 0.5 * dpi,
                "{} ({}) d{dpi}: the one height must clear the pre-91 dead space \
                 above an 'a': shipped={shipped_dead:.2}px row cell={row_dead:.2}px",
                t.name,
                t.font
            );
            worst_dead = worst_dead.max(shipped_dead / dpi);
            tightest_relief = tightest_relief.min((row_dead - shipped_dead) / dpi);
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
            checked += 1;
        }
    }
    p.set_dpi(1.0);
    assert!(
        checked >= 22,
        "every proportional-display world is swept at both DPIs (got {checked})"
    );
    eprintln!(
        "one caret top across the proportional roster × both DPIs: worst dead space \
         above an 'a''s ink = {worst_dead:.2}px at scale 1; tightest relief against \
         the pre-91 row cell = {tightest_relief:.2}px"
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
