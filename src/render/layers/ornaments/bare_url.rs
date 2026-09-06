//! Painted bare-URL ellipsis affordance: one quiet "…" per concealed tail.

use super::*;

/// The document source owns each horizontal slot (forced via
/// `render::spans::conceal::bare_url_ellipsis_slot`); this batch only supplies
/// the visible ink. Unlike [`super::footnotes::FootnoteNumbers`] there is no
/// payload to distinguish marks by — every mark paints the SAME glyph, so one
/// shaped buffer serves every mark (no per-value dedup loop needed).
pub(super) struct BareUrlEllipses {
    marks: Vec<(f32, f32, f32)>,
    glyph: GlyphBuffer,
    color: glyphon::Color,
}

impl BareUrlEllipses {
    pub(super) fn shape(pipeline: &mut TextPipeline, metrics: Metrics) -> Self {
        let marks = pipeline.bare_url_marks();
        let color = theme::muted().to_glyphon();
        let family = pipeline.shaped_font;
        // The tamed tail's "…" IS the smart-punctuation ellipsis — one
        // codepoint, painted for one reason — so it comes through that
        // roster's own shaping door rather than a second opinion about the
        // face and size. The reserved slot reads the SAME measurement
        // (`SubstituteAdvances::ellipsis_slot`).
        let (buffer, _) = super::super::spans::shape_smart_punct_glyph(
            &mut pipeline.font_system,
            metrics,
            family,
            crate::markdown::SmartPunctKind::Ellipsis,
            color,
        );
        Self {
            marks,
            glyph: buffer,
            color,
        }
    }

    pub(super) fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        let width = self
            .glyph
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0f32, f32::max);
        for (top, left, slot) in &self.marks {
            debug_assert!(
                width <= *slot + 0.5,
                "bare-URL ellipsis width {width} exceeds reserved slot {slot}"
            );
            areas.push(TextArea {
                buffer: &self.glyph,
                left: *left,
                top: *top,
                scale: 1.0,
                bounds,
                default_color: self.color,
                custom_glyphs: &[],
            });
        }
    }

    pub(super) fn len(&self) -> usize {
        self.marks.len()
    }
}
