//! Marker-cell ink envelopes shared by guide attachment and clearance.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct GuideAnchor {
    pub bounds: [f32; 4],
    pub indent: usize,
    pub continuous_gutter: bool,
}

impl GuideAnchor {
    pub fn center_x(self) -> f32 {
        self.bounds[0] + self.bounds[2] * 0.5
    }
    pub fn bottom(self) -> f32 {
        self.bounds[1] + self.bounds[3]
    }
}

impl TextPipeline {
    pub(super) fn list_guide_anchor(&mut self, line: usize) -> Option<GuideAnchor> {
        if let Some(item) = self.parsed_list_line(line) {
            let metrics = self.list_layout.metrics;
            let [width, height] = metrics.marker_size(item.depth, item.task());
            let center_x = self.text_left()
                + metrics.indent_advance(item.marker_col as f32, theme::active().list_indent_scale)
                + metrics.marker_center();
            let center_y =
                metrics.marker_center_y(self.list_marker_baseline(line, item.body_col()));
            return Some(GuideAnchor {
                bounds: [
                    center_x - width * 0.5,
                    center_y - height * 0.5,
                    width,
                    height,
                ],
                indent: item.marker_col,
                continuous_gutter: true,
            });
        }
        self.list_layout.ordered.binary_search(&line).ok()?;
        let item = crate::markdown::list_item(self.buffer.lines.get(line)?.text())?;
        let bounds = self.raw_list_prefix_bounds(line, item.indent, item.content - 1)?;
        Some(GuideAnchor {
            bounds,
            indent: item.indent,
            // Rich paragraphs reserve a hanging gutter. Raw ordered wraps
            // return to the text origin, so the bridge would cross their prose.
            continuous_gutter: self.buffer.lines[line].layout_opt()?.len() == 1,
        })
    }

    fn raw_list_prefix_bounds(
        &mut self,
        line: usize,
        start: usize,
        end: usize,
    ) -> Option<[f32; 4]> {
        let layout = self.buffer.lines.get(line)?.layout_opt()?.first()?;
        let origin = (self.text_left(), self.doc_top());
        let baseline = self.line_ornament_baseline(line) - origin.1;
        if !baseline.is_finite() {
            return None;
        }
        // Glyphon rounds the shaped baseline separately from the TextArea origin.
        // Folding it into physical() changes the subpixel mask and its ink bounds.
        let keys = layout
            .glyphs
            .iter()
            .filter(|glyph| glyph.start >= start && glyph.start < end)
            .map(|glyph| {
                let physical = glyph.physical(origin, 1.0);
                (
                    physical.cache_key,
                    physical.x,
                    physical.y + baseline.round() as i32,
                )
            })
            .collect::<Vec<_>>();
        self.glyph_ink_cache
            .bounds(&mut self.font_system, &mut self.swash_cache, &keys)
    }

    pub(super) fn painted_list_marker_bounds(&mut self, line: usize) -> Option<[f32; 4]> {
        if let Some(item) = self.parsed_list_line(line)
            && self.line_is_revealed(line, self.selection_touch().as_ref())
        {
            return self.raw_list_prefix_bounds(line, item.marker_col, item.body_col());
        }
        self.list_guide_anchor(line).map(|anchor| anchor.bounds)
    }
}
