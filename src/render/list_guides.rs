//! Structural list rails, painted independently of caret/selection concealment.
use super::*;

impl TextPipeline {
    pub(super) fn list_guide_geometry(&self) -> (Vec<[f32; 4]>, usize) {
        if !self.md_enabled || !crate::markdown::wysiwyg_on() || !self.document_active {
            return (Vec::new(), 0);
        }
        let guides = &self.list_layout.guides;
        let (x0, clip_top, x1, clip_bottom) = self.content_clip();
        let y0 = clip_top.max(0.0);
        let y1 = clip_bottom.min(self.window_h);
        let doc_top = self.doc_top();
        let stroke = self.dpi.max(1.0);
        self.row_geom
            .with_report_rows(|rows| {
                let first = rows.partition_point(|entry| {
                    doc_top + entry.row.line_top + entry.row.line_height <= y0
                });
                let past = rows.partition_point(|entry| doc_top + entry.row.line_top < y1);
                let visible = &rows[first..past.max(first)];
                let mut rects = Vec::new();
                for entry in visible {
                    let Ok(index) = guides.binary_search_by_key(&entry.logical_line, |g| g.line)
                    else {
                        continue;
                    };
                    let guide = &guides[index];
                    let row = &entry.row;
                    let scale = theme::active().list_indent_scale;
                    let body_start = if guide.marker {
                        self.parsed_list_line(entry.logical_line)
                            .map(|item| item.body_col())
                            .or_else(|| {
                                crate::markdown::list_item(
                                    self.buffer.lines[entry.logical_line].text(),
                                )
                                .map(|item| item.content)
                            })
                            .unwrap_or(guide.source_indent)
                    } else {
                        guide.source_indent
                    };
                    let first_ink = (row.start_col.max(body_start)..row.end_col)
                        .find(|&col| row.xs[col + 1] - row.xs[col] > 0.01)
                        .map_or(f32::INFINITY, |col| row.xs[col]);
                    let top = (doc_top + row.line_top).max(y0);
                    let bottom = (doc_top + row.line_top + row.line_height).min(y1);
                    for (&column, &parent) in guide.columns.iter().zip(&guide.parents) {
                        let metrics = self.list_layout.metrics;
                        let advance = self.parsed_list_line(parent).map_or_else(
                            || metrics.indent_advance(column, scale),
                            |item| {
                                metrics.indent_advance(item.marker_col as f32, scale)
                                    + metrics.marker_center(item.depth, item.task())
                            },
                        );
                        // Lazy/revealed continuation rows may start at column zero.
                        // Raw marker punctuation stays above the rail; prose must stay clear.
                        if advance + stroke >= first_ink {
                            continue;
                        }
                        let left = (self.text_left() + advance - stroke * 0.5).round();
                        if left >= x0 && left + stroke <= x1 && bottom > top {
                            rects.push([left, top, stroke, bottom - top]);
                        }
                    }
                }
                (rects, visible.len())
            })
            .unwrap_or_default()
    }

    pub(super) fn prepare_list_guides(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let (rects, _) = self.list_guide_geometry();
        self.list_guide_pipeline
            .prepare(device, queue, width, height, &rects);
    }
}
