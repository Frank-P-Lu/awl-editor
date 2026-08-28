//! Painted bare-URL ellipsis affordance: one quiet "…" per concealed tail.

use super::*;

const ELLIPSIS: char = '…';

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
        let glyph_metrics = GlyphMetrics::new(metrics.font_size, metrics.line_height);
        let attrs = Attrs::new()
            .family(Family::Name(theme::active().font))
            .color(color);
        let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
        buffer.set_size(
            &mut pipeline.font_system,
            Some(metrics.line_height * 2.0),
            Some(metrics.line_height),
        );
        buffer.set_text(
            &mut pipeline.font_system,
            &ELLIPSIS.to_string(),
            &attrs,
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut pipeline.font_system, false);
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
