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
        let family = pipeline.shaped_font;
        let mut distinct = Vec::new();
        for (_, _, number, _) in &marks {
            if !distinct.contains(number) {
                distinct.push(*number);
            }
        }
        // ONE shaping door with the reserved slot's own measurement
        // (`render::spans::shape_footnote_number`), so the ink and the room
        // made for it cannot be shaped at different sizes or in different
        // faces.
        let glyphs = distinct
            .into_iter()
            .map(|number| {
                let (buffer, width) = super::super::spans::shape_footnote_number(
                    &mut pipeline.font_system,
                    metrics,
                    family,
                    number,
                    color,
                );
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
