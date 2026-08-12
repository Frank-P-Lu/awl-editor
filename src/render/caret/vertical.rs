//! The PROPORTIONAL caret's one vertical envelope, and the row metrics a
//! GLYPHLESS row does not carry but must still report.

use super::*;

impl TextPipeline {
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
    /// EVERY proportional anchor — a letter, a ligature, a space, end-of-line,
    /// an empty row — which is what makes those anchors one height rather than
    /// six ([`Self::caret_cell_vertical_typical`] holds the reasoning).
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
}
