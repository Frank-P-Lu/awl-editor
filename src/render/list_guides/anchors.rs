//! Marker-cell ink envelopes shared by guide attachment and clearance.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct GuideAnchor {
    pub bounds: [f32; 4],
    pub indent: usize,
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
            });
        }
        self.list_layout.ordered.binary_search(&line).ok()?;
        let item = crate::markdown::list_item(self.buffer.lines.get(line)?.text())?;
        let layout = self.buffer.lines[line].layout_opt()?.first()?;
        let origin = (self.text_left(), self.line_ornament_baseline(line));
        if !origin.1.is_finite() {
            return None;
        }
        let keys = layout
            .glyphs
            .iter()
            .filter(|glyph| glyph.start >= item.indent && glyph.start < item.content - 1)
            .map(|glyph| {
                let physical = glyph.physical(origin, 1.0);
                (physical.cache_key, physical.x, physical.y)
            })
            .collect::<Vec<_>>();
        let bounds =
            self.glyph_ink_cache
                .bounds(&mut self.font_system, &mut self.swash_cache, &keys)?;
        Some(GuideAnchor {
            bounds,
            indent: item.indent,
        })
    }
}
