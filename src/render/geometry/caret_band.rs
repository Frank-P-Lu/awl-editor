//! THE CARET-HEIGHT BAND — the one scale every caret-adjacent treatment on a
//! document row is drawn from, and the one place a row's DECORATIVE height is
//! told apart from the height its own glyphs asked for.
//!
//! Three row-inflation mechanisms grow a document row past `line_height`, and a
//! band that took the raw quotient inherited whatever the row was grown FOR
//! rather than what is drawn in it. An inline image's absolute row and an
//! x-rayed table row are handled by carving out to a body-height band (both
//! below); a heading is the subtle one, because its row carries the size rung
//! its glyphs really take AND `heading_row_lead`'s decoupled breathing room,
//! which no glyph occupies. On the `###` rung — most lead, least size — that
//! second factor stood a selection band 43.15px tall over type shaped at
//! 27.6px, and did the same for the search wash, the code pill, the strike and
//! spell bands, the mono line cell and the insertion bar. The proportional
//! Block caret's ink envelope was already on the right quantity; this is what
//! puts the rest of the roster on it too.
//!
//! Laws: `render/tests/decor_geometry_vs_caret.rs`.

use super::*;

impl TextPipeline {
    pub(in crate::render) fn cursor_scale(&self) -> f32 {
        self.caret_band_scale(self.cursor_line, self.cursor_row_height())
            .max(1.0)
    }

    /// THE ONE OWNER of "how tall is the caret-height BAND on line `li`, as a
    /// multiple of the base line height" — shared by the resting caret
    /// ([`Self::cursor_scale`]) AND the selection / squiggle / nit row-band
    /// builders ([`super::TextPipeline::row_band_for`]), so the highlight over a
    /// character is always the SAME height the caret would draw there.
    ///
    /// `1.0` on body text; the heading's own SIZE rung (e.g. 1.6 on an `#`) so a
    /// heading's selection is as tall as its glyphs. IMAGE LINE (the caption
    /// model, WYSIWYG on): `1.0` — a BODY-height band, NOT the tall reserved
    /// row; the revealed source is body-size and the caret sizes to it
    /// ([`Self::cursor_row_height`]'s doc), and a row-scaled band would balloon
    /// into a char-wide × whole-image-height PILLAR (the reported selection
    /// bug). The band's vertical CENTRING still uses the full (tall)
    /// `row_height` at the call site, exactly where cosmic-text centres the
    /// source glyphs, so the body-height band lands ON the caption.
    ///
    /// **A HEADING ROW'S HEIGHT IS NOT ITS GLYPH SCALE:** it also carries
    /// [`crate::markdown::heading_row_lead`]'s decoupled breathing room, which
    /// no glyph occupies and a raw quotient would stand up to 34% over. That
    /// lead divides back out through [`super::spans::md_line_row_lead`], the
    /// owner `build_line_attrs` multiplied it in with. Laws:
    /// `render/tests/decor_geometry_vs_caret.rs`.
    pub(in crate::render) fn caret_band_scale(&self, li: usize, row_height: f32) -> f32 {
        if crate::markdown::wysiwyg_on() && self.line_is_inline_image(li) {
            return 1.0;
        }
        // THE X-RAY table row: the caret (or an active selection) rides the
        // FLOATED body-size source, not the (possibly tall, wrapped-cell) grid
        // row — so the band sizes to the source line, exactly like the image
        // caption model above.
        if self.xray.iter().any(|x| x.line == li) {
            return 1.0;
        }
        let lh = self.metrics.line_height;
        let text = self.buffer.lines.get(li).map(|l| l.text()).unwrap_or("");
        let lead = super::spans::md_line_row_lead(text, self.md_enabled);
        if lh > 0.0 {
            row_height / lh / lead
        } else {
            1.0
        }
    }
}
