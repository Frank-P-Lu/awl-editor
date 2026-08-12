//! Proportional caret vertical envelopes, and everything a GLYPHLESS anchor
//! stands in with when there is no ink to read: a neighbour's borrowed box, the
//! synthetic typical-letter box, and the row metrics an empty row does not
//! carry.

use super::*;

impl TextPipeline {
    /// Search within the visual row and blend lifted boxes from both sides by
    /// distance. This stays O(row width), never O(document).
    pub(super) fn nearest_row_raster_box(&mut self, line: usize, col: usize) -> Option<InkBox> {
        let rows = self.visual_rows(line);
        let row = rows.get(pick_row_index_aff(&rows, col, self.caret_affinity))?;
        let (start, end) = (row.start_col, row.end_col);
        let max_back = col.saturating_sub(start);
        let max_fwd = end.saturating_sub(col + 1);
        let back = (1..=max_back).find_map(|d| self.raster_box_at(line, col - d).map(|b| (d, b)));
        let fwd = (1..=max_fwd).find_map(|d| self.raster_box_at(line, col + d).map(|b| (d, b)));
        let (_, ascent, font) = self.caret_row_metrics();
        let lift = |(d, ink)| (d, self.caret_cell_vertical_ink_box(ink, ascent, font));

        match (back.map(lift), fwd.map(lift)) {
            (Some((db, bb)), Some((df, fb))) => {
                let t = db as f32 / (db + df) as f32;
                Some(InkBox {
                    left: bb.left + (fb.left - bb.left) * t,
                    top: bb.top + (fb.top - bb.top) * t,
                    width: bb.width + (fb.width - bb.width) * t,
                    height: bb.height + (fb.height - bb.height) * t,
                })
            }
            (Some((_, box_)), None) | (None, Some((_, box_))) => Some(box_),
            (None, None) => None,
        }
    }

    /// Lift short ink into its row's measured x-height band, preserving any
    /// real ascender or descender without a punctuation table.
    fn caret_cell_vertical_ink_box(&self, ink: InkBox, ascent: f32, font: &str) -> InkBox {
        let top = ink
            .top
            .max((ascent * facepitch::x_height_ratio(font)).max(1.0));
        let descent = ink.descent();
        InkBox {
            height: top + descent,
            top,
            ..ink
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

    /// The FALLBACK arm's SYNTHETIC ink box for a truly GLYPHLESS
    /// PROPORTIONAL anchor (space / end-of-line / an empty line — nothing
    /// [`Self::caret_anchor_raster_box`] can measure): a typical lowercase
    /// letter's placement, expressed in the SAME `top`/`height`-above-baseline
    /// convention a real [`InkBox`] uses, so [`Self::caret_cell_vertical`] can
    /// feed it through the identical formula the real ink-box arm reads.
    ///
    /// `top == height` (zero descent) by construction: a typical non-descending
    /// letter's ink sits ON the baseline with nothing below it, exactly like a
    /// real non-dipping glyph's box. There is deliberately no synthetic
    /// descender — a glyphless anchor has no letter to dip, so nothing to extend
    /// for; a REAL dipping ligature already carries its own descent inside its
    /// raster box in the caller above, untouched by this function.
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

    /// The sole proportional-cell vertical owner. Horizontal support-body
    /// dimensions remain ink-derived; only this centre and height use the
    /// row's insertion envelope.
    pub(in crate::render) fn caret_cell_vertical_from_ink(
        &self,
        ink: InkBox,
        baseline: f32,
        ascent: f32,
        font: &str,
        px: f32,
    ) -> (f32, f32) {
        let vertical = self.caret_cell_vertical_ink_box(ink, ascent, font);
        let (_, old_h) = caret_visual_body_dims(ink, px);
        let h = old_h.max(vertical.height + 2.0 * CARET_INK_PAD.px(px));
        (baseline - vertical.top + vertical.height * 0.5, h)
    }
}
