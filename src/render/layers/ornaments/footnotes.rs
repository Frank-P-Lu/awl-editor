//! Painted first-appearance footnote numbers.

use super::*;

/// The document source owns each horizontal slot; this batch only supplies the
/// visible superscript ink.
pub(super) struct FootnoteNumbers {
    marks: Vec<(f32, f32, usize, f32)>,
    glyphs: Vec<(usize, GlyphBuffer, f32)>,
    color: glyphon::Color,
    rise: f32,
}

impl FootnoteNumbers {
    pub(super) fn shape(pipeline: &mut TextPipeline, metrics: Metrics) -> Self {
        let marks = pipeline.footnote_marks();
        let color = theme::muted().to_glyphon();
        let glyph_metrics = GlyphMetrics::new(metrics.font_size * 0.68, metrics.line_height);
        let attrs = Attrs::new()
            .family(Family::Name(theme::active().font))
            .color(color);
        let mut distinct = Vec::new();
        for (_, _, number, _) in &marks {
            if !distinct.contains(number) {
                distinct.push(*number);
            }
        }
        let glyphs = distinct
            .into_iter()
            .map(|number| {
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(
                    &mut pipeline.font_system,
                    Some(metrics.line_height * 2.0),
                    Some(metrics.line_height),
                );
                buffer.set_text(
                    &mut pipeline.font_system,
                    &number.to_string(),
                    &attrs,
                    Shaping::Advanced,
                    None,
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                let width = buffer
                    .layout_runs()
                    .map(|run| run.line_w)
                    .fold(0.0f32, f32::max);
                (number, buffer, width)
            })
            .collect();
        Self {
            marks,
            glyphs,
            color,
            rise: metrics.line_height * 0.20,
        }
    }

    pub(super) fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        for (top, left, number, slot) in &self.marks {
            let (_, buffer, width) = self
                .glyphs
                .iter()
                .find(|(candidate, _, _)| candidate == number)
                .expect("footnote number was deduped in");
            debug_assert!(
                *width <= *slot + 0.5,
                "footnote number {number} width {width} exceeds reserved slot {slot}"
            );
            areas.push(TextArea {
                buffer,
                left: *left,
                top: *top - self.rise,
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
