//! Shared fold-chevron geometry, raster-ink placement, and hit-testing.

use super::*;

const GAP_CHARS: f32 = 0.3;
const WIDTH_CHARS: f32 = 1.0;
const FOLD_CHEVRON: &str = "\u{203A}";

/// One chevron's exact shaped-row box. Paint centres the visible mark within it;
/// hit-testing deliberately keeps the whole row height as a generous target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct FoldChevronGeom {
    pub(in crate::render) line: usize,
    pub(in crate::render) left: f32,
    pub(in crate::render) width: f32,
    pub(in crate::render) row_top: f32,
    pub(in crate::render) row_height: f32,
}

impl FoldChevronGeom {
    pub(in crate::render) fn row_center(self) -> f32 {
        self.row_top + self.row_height * 0.5
    }

    fn hit(self, px: f32, py: f32) -> bool {
        px >= self.left
            && px <= self.left + self.width
            && py >= self.row_top
            && py < self.row_top + self.row_height
    }
}

impl TextPipeline {
    /// Is there room in the writing column's own leading pad for the mark without
    /// spilling into the outline margin or overlapping the heading text?
    fn fold_chevron_has_room(&self) -> bool {
        let need = self.metrics.char_width * (GAP_CHARS + WIDTH_CHARS);
        self.text_left() - self.column_left() >= need
    }

    /// Hang the mark in the writing column's leading pad through the same shared
    /// pull-quote placement rule as the other left-margin-adjacent ornament.
    fn fold_chevron_left(&self) -> f32 {
        let gap = self.metrics.char_width * GAP_CHARS;
        let width = self.metrics.char_width * WIDTH_CHARS;
        super::super::geometry::pull_quote_left(self.column_left(), self.text_left(), gap, width)
    }

    /// One geometry owner for paint and hit-test: each currently summoned mark
    /// resolves to the exact first shaped-row box of its heading.
    pub(in crate::render) fn fold_chevron_geometries(&self) -> Vec<FoldChevronGeom> {
        if self.outline_headings.is_empty() || !self.fold_chevron_has_room() {
            return Vec::new();
        }
        let left = self.fold_chevron_left();
        let width = self.metrics.char_width * WIDTH_CHARS;
        self.outline_headings
            .iter()
            .filter(|h| {
                crate::fold::chevron_revealed(h.line, self.cursor_line, self.hover_line)
                    && self.line_ornament_visible(h.line)
            })
            .filter_map(|h| {
                let row = self.visual_rows(h.line).first()?.clone();
                Some(FoldChevronGeom {
                    line: h.line,
                    left,
                    width,
                    row_top: self.doc_top() + row.line_top,
                    row_height: row.line_height,
                })
            })
            .collect()
    }

    #[cfg(test)]
    pub(in crate::render) fn fold_chevron_marks(&self) -> Vec<(f32, f32, usize)> {
        self.fold_chevron_geometries()
            .into_iter()
            .map(|g| (g.row_center(), g.left, g.line))
            .collect()
    }

    /// Does `(px, py)` land on a currently painted mark, and which filtered
    /// heading line does it toggle? The same resolved geometry paint consumes
    /// drives the pointing-hand cursor and click action.
    pub fn fold_chevron_hit(&self, px: f32, py: f32) -> Option<usize> {
        self.fold_chevron_geometries()
            .into_iter()
            .find_map(|g| g.hit(px, py).then_some(g.line))
    }
}

pub(super) struct FoldChevrons {
    marks: Vec<FoldChevronGeom>,
    buffers: Vec<GlyphBuffer>,
    ink_centers: Vec<f32>,
    color: glyphon::Color,
    fallback_line_y: f32,
}

impl FoldChevrons {
    pub(super) fn shape(pipeline: &mut TextPipeline, metrics: Metrics, col_w: f32) -> Self {
        let marks = pipeline.fold_chevron_geometries();
        let color = theme::fold_afford_chevron_ink().to_glyphon();
        let mark_h = metrics.line_height * crate::markdown::type_scale::LABEL;
        let glyph_metrics = GlyphMetrics::new(
            metrics.font_size * crate::markdown::type_scale::LABEL,
            mark_h,
        );
        let attrs = panel_attrs().color(color);
        let buffers: Vec<GlyphBuffer> = marks
            .iter()
            .map(|_| {
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(&mut pipeline.font_system, Some(col_w), Some(mark_h));
                buffer.set_text(
                    &mut pipeline.font_system,
                    FOLD_CHEVRON,
                    &attrs,
                    Shaping::Advanced,
                    None,
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                buffer
            })
            .collect();
        let fallback_line_y = mark_h * 0.8;
        let ink_centers = buffers
            .iter()
            .map(|buffer| Self::ink_center(pipeline, buffer, fallback_line_y))
            .collect();
        Self {
            marks,
            buffers,
            ink_centers,
            color,
            fallback_line_y,
        }
    }

    /// The visible mark's centre in its shaped-buffer coordinates, measured from
    /// the actual nonzero swash-mask rows rather than the font baseline.
    fn ink_center(pipeline: &mut TextPipeline, buffer: &GlyphBuffer, fallback_line_y: f32) -> f32 {
        let glyphs: Vec<(CacheKey, f32)> = buffer
            .layout_runs()
            .flat_map(|run| {
                run.glyphs
                    .iter()
                    .map(move |g| (g.physical((0.0, 0.0), 1.0).cache_key, run.line_y))
            })
            .collect();
        let (mut top, mut bottom) = (f32::MAX, f32::MIN);
        let TextPipeline {
            swash_cache,
            font_system,
            ..
        } = pipeline;
        for (key, baseline) in glyphs {
            let Some(image) = swash_cache.get_image(font_system, key).as_ref() else {
                continue;
            };
            if image.content != SwashContent::Mask
                || image.placement.width == 0
                || image.placement.height == 0
            {
                continue;
            }
            let width = image.placement.width as usize;
            let mut first_ink_row = None;
            let mut last_ink_row = None;
            for (row, coverage) in image.data.chunks_exact(width).enumerate() {
                if coverage.iter().any(|&alpha| alpha != 0) {
                    first_ink_row.get_or_insert(row);
                    last_ink_row = Some(row);
                }
            }
            let (Some(first), Some(last)) = (first_ink_row, last_ink_row) else {
                continue;
            };
            let glyph_top = baseline - image.placement.top as f32 + first as f32;
            let glyph_bottom = baseline - image.placement.top as f32 + (last + 1) as f32;
            top = top.min(glyph_top);
            bottom = bottom.max(glyph_bottom);
        }
        if bottom > top {
            (top + bottom) * 0.5
        } else {
            fallback_line_y
        }
    }

    pub(super) fn len(&self) -> usize {
        self.marks.len()
    }

    pub(super) fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        for (i, &mark) in self.marks.iter().enumerate() {
            let buffer = &self.buffers[i];
            let ink_center = self
                .ink_centers
                .get(i)
                .copied()
                .unwrap_or(self.fallback_line_y);
            areas.push(TextArea {
                buffer,
                left: mark.left,
                top: mark.row_center() - ink_center,
                scale: 1.0,
                bounds,
                default_color: self.color,
                custom_glyphs: &[],
            });
        }
    }
}
