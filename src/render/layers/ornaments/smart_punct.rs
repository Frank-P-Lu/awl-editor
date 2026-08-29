//! Painted smart-punctuation substitute glyphs: the single en dash, em dash,
//! or ellipsis character that replaces a concealed `--`/`---`/`...` run.

use super::*;

/// The document source owns each horizontal slot (forced via
/// `render::spans::conceal::smart_punct_slot`); this batch only supplies the
/// visible ink. Distinct GLYPHS — at most three, the whole
/// [`crate::markdown::SmartPunctKind`] roster — are shaped once and shared by
/// every mark of that kind, the [`super::footnotes::FootnoteNumbers`]
/// per-value dedup precedent, bounded to a fixed roster instead of an open
/// digit range.
pub(super) struct SmartPunctGlyphs {
    marks: Vec<(f32, f32, crate::markdown::SmartPunctKind, f32)>,
    glyphs: Vec<(crate::markdown::SmartPunctKind, GlyphBuffer, f32)>,
    color: glyphon::Color,
}

impl SmartPunctGlyphs {
    pub(super) fn shape(pipeline: &mut TextPipeline, metrics: Metrics) -> Self {
        let marks = pipeline.smart_punct_marks();
        let color = theme::muted().to_glyphon();
        let glyph_metrics = GlyphMetrics::new(metrics.font_size, metrics.line_height);
        let attrs = Attrs::new()
            .family(Family::Name(theme::active().font))
            .color(color);
        let mut distinct = Vec::new();
        for (_, _, kind, _) in &marks {
            if !distinct.contains(kind) {
                distinct.push(*kind);
            }
        }
        let glyphs = distinct
            .into_iter()
            .map(|kind| {
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(
                    &mut pipeline.font_system,
                    Some(metrics.line_height * 2.0),
                    Some(metrics.line_height),
                );
                buffer.set_text(
                    &mut pipeline.font_system,
                    &kind.glyph().to_string(),
                    &attrs,
                    Shaping::Advanced,
                    None,
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                let width = buffer
                    .layout_runs()
                    .map(|run| run.line_w)
                    .fold(0.0f32, f32::max);
                (kind, buffer, width)
            })
            .collect();
        Self {
            marks,
            glyphs,
            color,
        }
    }

    pub(super) fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        for (top, left, kind, slot) in &self.marks {
            let (_, buffer, width) = self
                .glyphs
                .iter()
                .find(|(candidate, _, _)| candidate == kind)
                .expect("smart-punct glyph was deduped in");
            debug_assert!(
                *width <= *slot + 0.5,
                "smart-punct glyph {kind:?} width {width} exceeds reserved slot {slot}"
            );
            areas.push(TextArea {
                buffer,
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
