//! The PROPORTIONAL caret's one vertical envelope, and the row metrics a
//! GLYPHLESS row does not carry but must still report.

use super::*;

impl TextPipeline {
    /// A CJK anchor's stable resolved-face cell. No glyph identity or raster
    /// box enters the result, so adjacent kanji on one script face cannot jitter.
    pub(in crate::render) fn caret_anchor_ideographic_cell(&self) -> Option<(f32, (f32, f32))> {
        let col = self.caret_anchor_col();
        let ch = self
            .buffer
            .lines
            .get(self.cursor_line)?
            .text()
            .chars()
            .nth(col)?;
        let script = crate::script::classify_char(ch)?;
        let id = crate::script::resolve_font_id(self.doc_lang, Some(script), &self.cjk_priority);
        let (family, _) = self.script_fonts.get(id)?;
        let em = facepitch::ideographic_cell_em(family)?;
        let key = self.cursor_glyph_key_at(self.cursor_line, col)?;
        Some((f32::from_bits(key.font_size_bits), em))
    }

    /// A resolved CJK face's stable one-em placement box. Pure geometry: the
    /// face supplies the baseline split, the shaped run supplies only its size.
    pub(in crate::render) fn ideographic_cell_box(
        font_size: f32,
        (ascent_em, descent_em): (f32, f32),
    ) -> InkBox {
        let top = (font_size * ascent_em).max(1.0);
        let descent = (font_size * descent_em).max(0.0);
        InkBox {
            left: 0.0,
            top,
            width: font_size,
            height: top + descent,
        }
    }

    /// THE ONE OWNER of "what ascent and descent does a row with NO GLYPHS
    /// have" — `(max_ascent, max_descent, font)`, in the same absolute pixels
    /// and the same sign convention a shaped [`cosmic_text::LayoutLine`]
    /// carries, so [`Self::caret_row_metrics`] can feed either through one
    /// centring formula.
    ///
    /// cosmic-text derives a shaped row's pair by multiplying the face's own
    /// per-em ascent/descent by the row's font size; with no glyph to read them
    /// off, this does the identical multiply one factor earlier
    /// ([`facepitch::vertical_em_metrics`] — measured off the shipped face
    /// file, never a hand-tuned fraction). The retired approximation here was a
    /// flat `font_size * 0.8`, which is ~30% low against Literata's real row
    /// ascent and carried no descent at all.
    ///
    /// Keyed on `doc_family()` (the LIVE effective face the ACTIVE theme wants),
    /// deliberately NOT `shaped_font`: nothing was shaped on this row, so there
    /// is no on-screen glyph whose face this must agree with, and the value
    /// should track the theme-picker preview's instant colour retint rather
    /// than wait on the separately-deferred reshape — the same gate
    /// [`Self::caret_cell_vertical`] picks its fallback arm with. `font_size` is
    /// the row's size: an empty row can carry no heading scale (a heading needs
    /// its `#`), so the base metrics ARE its metrics, already zoom/DPI-folded.
    pub(super) fn glyphless_row_vertical(&self) -> (f32, f32, &'static str) {
        let font = self.doc_family();
        let (ascent_em, descent_em) = facepitch::vertical_em_metrics(font);
        let size = self.metrics.font_size;
        (size * ascent_em, size * descent_em, font)
    }

    /// THE PROPORTIONAL CARET'S ONE BOX: a typical letter's placement on this
    /// row, expressed in the SAME `top`/`height`-above-baseline convention a
    /// real raster [`InkBox`] uses. [`Self::caret_cell_vertical`] feeds it for
    /// every non-CJK proportional anchor — a letter, a ligature, a space,
    /// end-of-line, an empty row — which is what makes those anchors one height
    /// rather than six ([`Self::caret_cell_vertical_typical`] holds the reasoning).
    ///
    /// `top == height` (zero descent) by construction: a typical non-descending
    /// letter's ink sits ON the baseline with nothing below it. There is
    /// deliberately no synthetic descender, and no real one either — the box
    /// describes the row's ordinary letter, never the anchored glyph, so a
    /// dipping `g` extends nothing.
    ///
    /// `row_max_ascent` is the SAME per-row value [`Self::caret_row_metrics`]
    /// pairs with the baseline this box is fed against — already reshaped for a
    /// heading / zoom / DPI row, so this needs no separate font-size lookup —
    /// scaled by `ratio_font`'s OWN measured typical-letter/ascent ratio
    /// ([`facepitch::typical_letter_ratio`] — the MEAN of the face's x-height and
    /// cap-height, not bare x-height: a pure x-height reference reproduces the
    /// vertical misalignment in miniature against an ASCENDER neighbour, so the mean is the
    /// balance point between the two glyph classes the ink-box arm already
    /// treats as different heights): a real per-font quantity read off the
    /// shipped face file, not a hand-tuned per-world offset.
    ///
    /// `ratio_font` is [`Self::caret_row_metrics`]'s own third element —
    /// WHICHEVER font actually produced `row_max_ascent` — never independently
    /// re-derived here. A real shaped row's `max_ascent` is a property of
    /// `shaped_font` (the face the row is ACTUALLY laid out in this frame); a
    /// glyphless row's is rebuilt from `doc_family()`'s own per-em ascent and
    /// pairs with `doc_family()` (see the caller's doc). Reading anything
    /// else here (e.g. unconditionally `doc_family()`) would multiply one
    /// font's ascent by a DIFFERENT font's ratio whenever the two diverge — the
    /// theme-picker preview lag (`sync_theme_colors` without a reshape yet) is
    /// the one live case that does, and the mixed number it produces pops a few
    /// px on ordinary text mid-scrub even though neither factor alone is wrong.
    pub(super) fn caret_synthetic_ink_box(&self, row_max_ascent: f32, ratio_font: &str) -> InkBox {
        let ratio = facepitch::typical_letter_ratio(ratio_font);
        let top = (row_max_ascent * ratio).max(1.0);
        InkBox {
            left: 0.0,
            top,
            width: 0.0,
            height: top,
        }
    }

    /// The proportional cell's `(center_y, height)`: the row's typical-letter
    /// box, padded, floored by the shared minimum visible body. Horizontal
    /// support-body dimensions stay ink-derived ([`super::caret_visual_body_dims`]
    /// in [`Self::caret_geometry`]); only this centre and height are the row's.
    ///
    /// ⚠️ The body floor is measured against the SYNTHETIC box, never against
    /// the anchored glyph's real ink, and that is load-bearing rather than
    /// convenient: [`super::caret_visual_body_dims`] grows a small body to an
    /// AREA, so feeding it a real `m`'s wide ink and a real `i`'s narrow ink
    /// would hand back two different heights at small sizes — the per-glyph
    /// jump this arm exists to remove, re-entering through the floor.
    ///
    /// This is the shared-CELL formula — [`Self::caret_cell_vertical`]'s
    /// non-Block arm (Morph's support-body decision, the glyphless space bar).
    /// The literal Block caret's own, TALLER envelope is
    /// [`Self::caret_cell_vertical_block`], a sibling policy rather than a
    /// parameter here, so Morph and the space bar cannot inherit it merely
    /// because their geometry sits nearby.
    pub(in crate::render) fn caret_cell_vertical_typical(
        &self,
        baseline: f32,
        ascent: f32,
        font: &str,
        px: f32,
    ) -> (f32, f32) {
        let box_ = self.caret_synthetic_ink_box(ascent, font);
        let (_, floor_h) = caret_visual_body_dims(box_, px);
        let h = floor_h.max(box_.height + 2.0 * CARET_INK_PAD.px(px));
        (baseline - box_.top + box_.height * 0.5, h)
    }

    /// THE LITERAL BLOCK CARET'S proportional-face vertical box: the row's
    /// real ASCENDER-to-DESCENDER ink envelope
    /// ([`facepitch::ink_envelope_em`]), in the same baseline-relative
    /// convention [`Self::caret_synthetic_ink_box`] uses. Item 451's user
    /// verdict on the typical-letter box: a real ascender (`d`, `l`, `b`,
    /// `h`, `k`) visibly pokes its ink above that box's top, because the
    /// typical-letter ratio is tuned to the MEAN letter, not the tallest one
    /// that can occupy the cell. This box is tall enough for the roster's
    /// worst case instead.
    ///
    /// `ratio_font`'s own measured `hhea` ascent fraction
    /// ([`facepitch::vertical_em_metrics`].0) recovers the row's FONT SIZE
    /// from `row_max_ascent` (`row_max_ascent == font_size * hhea_ascent_em`,
    /// the same identity [`Self::caret_synthetic_ink_box`]'s caller already
    /// relies on) — one factor earlier than reading a font size directly, so
    /// this needs no second row lookup. The real ink fractions then scale
    /// that font size, never `row_max_ascent` itself: `hhea` ascent is a
    /// generous LINE-SPACING metric (on some bundled faces its sum with
    /// descent alone exceeds the row height), so scaling directly off it
    /// would reproduce the very overshoot this box exists to avoid.
    ///
    /// A FACE-LEVEL fraction of a real per-row font size, not a per-glyph
    /// raster read — the top and bottom stay fixed for every anchor on the
    /// same face/row, exactly the stability item 448 established and this
    /// item's verdict only widens, never removes.
    pub(super) fn caret_block_ink_box(&self, row_max_ascent: f32, ratio_font: &str) -> InkBox {
        let (hhea_ascent_em, _) = facepitch::vertical_em_metrics(ratio_font);
        let (ink_ascent_em, ink_descent_em) = facepitch::ink_envelope_em(ratio_font);
        let font_size = if hhea_ascent_em > 0.0 {
            row_max_ascent / hhea_ascent_em
        } else {
            0.0
        };
        let top = (font_size * ink_ascent_em).max(1.0);
        let bottom = (font_size * ink_descent_em).max(0.0);
        InkBox {
            left: 0.0,
            top,
            width: 0.0,
            height: top + bottom,
        }
    }

    /// The literal Block caret's `(center_y, height)` on a proportional
    /// Latin row — [`Self::caret_cell_vertical_block`]'s ONLY caller besides
    /// its own law tests wants exactly this pair, padded and floored the
    /// same way [`Self::caret_cell_vertical_typical`] is, so a reader
    /// comparing the two sees the one difference that matters: which box
    /// backs it.
    ///
    /// ⚠️ NEVER TOUCHES THE ADJACENT ROW — measured, not assumed: on the
    /// roster's TIGHTEST bundled face (Bitter — Mopoke/Magpie), the ink
    /// envelope plus both full [`CARET_INK_PAD`]s already overshoots the
    /// row's own line height by a fraction of a px (the app renders every
    /// face at one FIXED line height, `render::LINE_HEIGHT`, independent of
    /// that face's own metrics — see [`super::super::facepitch`]'s module
    /// doc on why `hhea` ascent/descent cannot supply this box either). The
    /// PAD is the sacrificial quantity, shrunk symmetrically toward zero
    /// until the padded box fits the row, never the ink coverage
    /// (`box_.height`) itself — giving up ink coverage under pressure is
    /// exactly the bug this box exists to fix, so the floor below refuses to
    /// go under it even if that means the tightest face's caret still sits a
    /// hair from the row edge. The CENTRE is unaffected by how much pad
    /// survives: it depends only on `box_.top`/`box_.height`, so shrinking
    /// the pad narrows the box symmetrically around the same ink midpoint
    /// rather than sliding it.
    pub(in crate::render) fn caret_cell_vertical_block(
        &self,
        baseline: f32,
        ascent: f32,
        font: &str,
        px: f32,
    ) -> (f32, f32) {
        let box_ = self.caret_block_ink_box(ascent, font);
        let (_, floor_h) = caret_visual_body_dims(box_, px);
        let ideal_h = floor_h.max(box_.height + 2.0 * CARET_INK_PAD.px(px));
        let row_h = self.cursor_row_height();
        let clearance = Logical(1.0).px(px);
        let max_h = (row_h - clearance).max(box_.height);
        let h = ideal_h.min(max_h);
        (baseline - box_.top + box_.height * 0.5, h)
    }

    /// The CJK cell-form caret: one resolved-face em square, padded through the
    /// same authored body rule as the Latin typical-letter box.
    pub(in crate::render) fn caret_cell_vertical_ideographic(
        &self,
        baseline: f32,
        font_size: f32,
        em: (f32, f32),
        px: f32,
    ) -> (f32, f32) {
        let box_ = Self::ideographic_cell_box(font_size, em);
        let (_, floor_h) = caret_visual_body_dims(box_, px);
        let h = floor_h.max(box_.height + 2.0 * CARET_INK_PAD.px(px));
        (baseline - box_.top + box_.height * 0.5, h)
    }
}
