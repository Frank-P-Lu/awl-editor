//! The hanging blockquote pull-quote PAIR, split out of `ornaments.rs` to keep
//! that file under its structural ceiling. See the module doc there for the
//! five-family ornament-frame context this is one member of.

use super::*;

/// The hanging pull-quote PAIR. Both ends are shaped from ONE `attrs`/`GlyphMetrics`
/// pair — one face, one scale, one [`theme::faint`] value — so open and close can
/// only ever differ in glyph, x and y. The OPEN mark keeps the original hanging
/// placement: [`super::super::geometry::pull_quote_left`], a page-geometry constant
/// (independent of which block), landed on its row's own top the way a drop-cap
/// hangs from a paragraph. The CLOSE mark instead FOLLOWS the text (decided
/// 2026-09): its x is that block's own last-row ink-right plus a gap
/// ([`super::QUOTE_CLOSE_GAP_EM`], clamped inside the text column so a wide wrap
/// can never push it past the page edge), and its y anchors to that row's own
/// baseline (with [`super::QUOTE_CLOSE_BASELINE_DROP_FRAC`]'s taste offset),
/// floored so it can never rise above the row's own top.
pub(super) struct QuoteOrnaments {
    /// `(top, left, side)` — the fully resolved paint position per mark. Unlike
    /// the OPEN end (one x for the whole document), the CLOSE end's x and y vary
    /// PER BLOCK, so each mark carries its own rather than sharing one baked
    /// per-side value.
    marks: Vec<(f32, f32, QuoteSide)>,
    /// Indexed by [`Self::slot`]: the shaped glyph alone (no baked x — see `marks`).
    ends: [GlyphBuffer; 2],
    color: glyphon::Color,
}

impl QuoteOrnaments {
    /// Mark count, for the ornament frame's pre-sized `Vec::with_capacity` —
    /// `marks` itself stays private now that this struct lives in its own
    /// module, unlike its siblings which still share `ornaments.rs` with
    /// `OrnamentFrame`.
    pub(super) fn len(&self) -> usize {
        self.marks.len()
    }

    fn slot(side: QuoteSide) -> usize {
        match side {
            QuoteSide::Open => 0,
            QuoteSide::Close => 1,
        }
    }

    pub(super) fn shape(pipeline: &mut TextPipeline, metrics: Metrics) -> Self {
        let raw_marks = pipeline.quote_marks();
        let color = theme::faint().to_glyphon();
        let glyph_metrics =
            GlyphMetrics::new(metrics.font_size * QUOTE_MARK_SCALE, metrics.line_height);
        let attrs = Attrs::new()
            .family(Family::Name(theme::active().font))
            .color(color);
        let box_w = (metrics.font_size * QUOTE_MARK_SCALE * 2.0).max(1.0);
        let open_gap = metrics.char_width * 0.3;
        let close_gap = metrics.font_size * QUOTE_CLOSE_GAP_EM;
        let column_left = pipeline.column_left();
        let text_left = pipeline.text_left();
        let text_right = text_left + pipeline.text_wrap_width();

        // Shape one end's glyph and report its own shaped advance (for x) and its
        // own single-line baseline offset from its box's top (`line_y`, the same
        // real-shaped-baseline read `FoldTails` anchors its tail to) — both are
        // properties of the GLYPH, invariant across every block it hangs from.
        let shape_end = |pipeline: &mut TextPipeline, glyph: char| -> (GlyphBuffer, f32, f32) {
            let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
            if raw_marks.is_empty() {
                return (buffer, 0.0, 0.0);
            }
            buffer.set_size(
                &mut pipeline.font_system,
                Some(box_w),
                Some(metrics.line_height),
            );
            buffer.set_text(
                &mut pipeline.font_system,
                &glyph.to_string(),
                &attrs,
                Shaping::Advanced,
                None,
            );
            buffer.shape_until_scroll(&mut pipeline.font_system, false);
            let mark_w = buffer
                .layout_runs()
                .map(|run| run.line_w)
                .fold(0.0f32, f32::max);
            let line_y = buffer
                .layout_runs()
                .next()
                .map(|run| run.line_y)
                .unwrap_or(0.0);
            (buffer, mark_w, line_y)
        };

        let (open_buf, open_w, _) = shape_end(pipeline, QUOTE_MARK_GLYPH);
        let (close_buf, close_w, close_line_y) = shape_end(pipeline, QUOTE_MARK_CLOSE_GLYPH);
        let open_x =
            super::super::geometry::pull_quote_left(column_left, text_left, open_gap, open_w);

        let marks = raw_marks
            .into_iter()
            .map(|(top, side, line)| match side {
                QuoteSide::Open => (top, open_x, side),
                QuoteSide::Close => {
                    // x: one gap past THIS block's own last-row ink, yielding
                    // inside the text column rather than escaping past its right
                    // edge at the widest wrap.
                    let ink_right = pipeline.quote_close_row_end_x(line);
                    let x = super::super::geometry::pull_quote_close_x(
                        ink_right, text_left, text_right, close_gap, close_w,
                    );
                    // y: the mark's OWN baseline lands `DROP` below the row's
                    // real baseline (closer to the line, not floating above it),
                    // floored so it can never rise above the row's own top —
                    // `top` here is that row's `line_ornament_last_top`, read
                    // straight from `raw_marks`.
                    let baseline = pipeline.line_ornament_last_baseline(line)
                        + metrics.line_height * QUOTE_CLOSE_BASELINE_DROP_FRAC;
                    let box_top = (baseline - close_line_y).max(top);
                    (box_top, x, side)
                }
            })
            .collect();

        Self {
            marks,
            ends: [open_buf, close_buf],
            color,
        }
    }

    pub(super) fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        for &(top, left, side) in &self.marks {
            let buffer = &self.ends[Self::slot(side)];
            areas.push(TextArea {
                buffer,
                left,
                top,
                scale: 1.0,
                bounds,
                default_color: self.color,
                custom_glyphs: &[],
            });
        }
    }
}

#[cfg(test)]
impl TextPipeline {
    /// TEST-ONLY: the exact resolved `(top, left, width, side)` of every
    /// pull-quote mark this frame would paint, without shaping the rest of the
    /// ornament frame or reaching a GPU. Lets a unit test assert the CLOSE
    /// mark's real x/y/clamp behavior directly, at the purest reachable seam,
    /// rather than only through rendered pixels.
    pub(in crate::render) fn quote_mark_geometry_for_test(
        &mut self,
    ) -> Vec<(f32, f32, f32, QuoteSide)> {
        let metrics = self.metrics;
        let q = QuoteOrnaments::shape(self, metrics);
        q.marks
            .iter()
            .map(|&(top, left, side)| {
                let width = q.ends[QuoteOrnaments::slot(side)]
                    .layout_runs()
                    .map(|run| run.line_w)
                    .fold(0.0f32, f32::max);
                (top, left, width, side)
            })
            .collect()
    }
}
