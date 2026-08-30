//! Pointer-driven horizontal table panning.

use crate::render::{ScrollPos, TextPipeline};

impl TextPipeline {
    pub fn try_table_pan(&mut self, px: f32, py: f32, scroll: ScrollPos, dx: f32) -> bool {
        if !(crate::markdown::wysiwyg_on() && self.md_enabled) {
            return false;
        }
        let (line, _) = self.hit_test_scroll(px, py, scroll);
        let line_byte = self.line_doc_byte_start(line);
        let Some((start, _range)) = self
            .table_blocks()
            .into_iter()
            .find(|(_, range)| range.start <= line_byte && line_byte < range.end)
            .map(|(_, range)| (range.start, range))
        else {
            return false;
        };
        let report = self.table_report.borrow();
        let Some(table) = report.iter().find(|table| table.range.0 == start) else {
            return false;
        };
        let columns = table.col_widths.len();
        if columns == 0 {
            return false;
        }
        let gap = self.metrics.px(crate::render::TABLE_COL_GAP);
        let content_w =
            table.col_widths.iter().sum::<f32>() + gap * columns.saturating_sub(1) as f32;
        drop(report);
        let view_w = self.text_wrap_width().max(1.0);
        if content_w <= view_w + 1e-3 {
            return false;
        }
        let current = self
            .table_pan
            .filter(|(table_start, _)| *table_start == start)
            .map(|(_, offset)| offset)
            .unwrap_or(0.0);
        let next = crate::markdown::table_pan_clamp(current - dx, content_w, view_w);
        self.table_pan = Some((start, next));
        true
    }
}
