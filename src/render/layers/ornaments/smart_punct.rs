//! Painted smart-punctuation substitute glyphs: the single en dash, em dash,
//! or ellipsis character that replaces a concealed `--`/`---`/`...` run.

use super::*;

/// The document source owns each horizontal slot (forced via
/// [`crate::render::spans::SubstituteAdvances`]); this batch supplies the
/// visible ink from that same shaping owner. Distinct GLYPHS — at most three,
/// the whole [`crate::markdown::SmartPunctKind`] roster — are shaped once and
/// shared by every mark of that kind, the
/// [`super::footnotes::FootnoteNumbers`] per-value dedup precedent, bounded to
/// a fixed roster instead of an open digit range.
pub(super) struct SmartPunctGlyphs {
    marks: Vec<(f32, f32, crate::markdown::SmartPunctKind, f32, u8)>,
    glyphs: Vec<(crate::markdown::SmartPunctKind, u8, GlyphBuffer, f32)>,
    color: glyphon::Color,
}

impl SmartPunctGlyphs {
    pub(super) fn shape(pipeline: &mut TextPipeline, metrics: Metrics) -> Self {
        let marks = pipeline.smart_punct_marks();
        let color = theme::base_content().to_glyphon();
        let family = pipeline.shaped_font;
        let mut distinct = Vec::new();
        for (_, _, kind, _, level) in &marks {
            if !distinct.contains(&(*kind, *level)) {
                distinct.push((*kind, *level));
            }
        }
        let glyphs = distinct
            .into_iter()
            .map(|(kind, level)| {
                let (buffer, width) = shape_smart_punct_glyph(
                    &mut pipeline.font_system,
                    super::super::spans::smart_punct_metrics(metrics, level),
                    family,
                    kind,
                    color,
                );
                (kind, level, buffer, width)
            })
            .collect();
        Self {
            marks,
            glyphs,
            color,
        }
    }

    pub(super) fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        for (top, left, kind, slot, level) in &self.marks {
            let (_, _, buffer, width) = self
                .glyphs
                .iter()
                .find(|(candidate, candidate_level, _, _)| {
                    candidate == kind && candidate_level == level
                })
                .expect("smart-punct glyph was deduped in");
            debug_assert!(
                (*width - *slot).abs() <= 0.5,
                "smart-punct glyph {kind:?} width {width} differs from its measured advance {slot}"
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
