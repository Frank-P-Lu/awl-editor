//! Structural rails attach to marker cells, with one ink-clearance rule at endpoints.
use super::*;
mod anchors;
mod clipping;

struct GuideRow {
    line: usize,
    first: bool,
    last: bool,
    top: f32,
    bottom: f32,
    first_ink: f32,
    parents: Vec<usize>,
}

impl TextPipeline {
    fn visible_guide_rows(&self, y0: f32, y1: f32) -> (Vec<GuideRow>, usize) {
        let doc_top = self.doc_top();
        self.row_geom
            .with_report_rows(|rows| {
                let first = rows.partition_point(|entry| {
                    doc_top + entry.row.line_top + entry.row.line_height <= y0
                });
                let past = rows.partition_point(|entry| doc_top + entry.row.line_top < y1);
                let mut out = Vec::new();
                for index in first..past.max(first) {
                    let entry = &rows[index];
                    let line = entry.logical_line;
                    let Ok(guide_index) = self
                        .list_layout
                        .guides
                        .binary_search_by_key(&line, |g| g.line)
                    else {
                        continue;
                    };
                    let guide = &self.list_layout.guides[guide_index];
                    debug_assert_eq!(guide.parents.len(), guide.columns.len());
                    let row = &entry.row;
                    let body_start = if guide.marker {
                        self.parsed_list_line(line)
                            .map(|item| item.body_col())
                            .or_else(|| {
                                crate::markdown::list_item(self.buffer.lines[line].text())
                                    .map(|item| item.content)
                            })
                            .unwrap_or(guide.source_indent)
                    } else {
                        guide.source_indent
                    };
                    let first_ink = (row.start_col.max(body_start)..row.end_col)
                        .find(|&col| row.xs[col + 1] - row.xs[col] > 0.01)
                        .map_or(f32::INFINITY, |col| self.text_left() + row.xs[col]);
                    out.push(GuideRow {
                        line,
                        first: row.start_col == 0,
                        last: rows
                            .get(index + 1)
                            .is_none_or(|next| next.logical_line != line),
                        top: doc_top + row.line_top,
                        bottom: doc_top + row.line_top + row.line_height,
                        first_ink,
                        parents: guide.parents.clone(),
                    });
                }
                (out, past.max(first) - first)
            })
            .unwrap_or_default()
    }

    pub(super) fn list_guide_geometry(&mut self) -> (Vec<[f32; 4]>, usize) {
        if !self.md_enabled || !crate::markdown::wysiwyg_on() || !self.document_active {
            return (Vec::new(), 0);
        }
        let (x0, clip_top, x1, clip_bottom) = self.content_clip();
        let (y0, y1) = (clip_top.max(0.0), clip_bottom.min(self.window_h));
        let (rows, visited) = self.visible_guide_rows(y0, y1);
        let stroke = self.dpi.max(1.0);
        let clearance = self.metrics.font_size * 0.125;
        let touch = self.selection_touch();
        let mut cache = std::collections::BTreeMap::new();
        let mut markers = std::collections::BTreeMap::new();
        let mut rects = Vec::new();
        for row in rows {
            for parent in row.parents {
                let Some(anchor) = *cache
                    .entry(parent)
                    .or_insert_with(|| self.list_guide_anchor(parent))
                else {
                    continue;
                };
                let center = anchor.center_x();
                if center + stroke >= row.first_ink {
                    continue;
                }
                let mut top = row.top;
                let mut bottom = row.bottom;
                if row.first
                    && parent + 1 == row.line
                    && anchor.continuous_gutter
                    && !self.line_is_revealed(parent, touch.as_ref())
                    && anchor.bottom().is_finite()
                {
                    top = top.min(anchor.bottom() + clearance);
                }
                if row.last && !self.line_is_revealed(row.line + 1, touch.as_ref()) {
                    let next = *cache
                        .entry(row.line + 1)
                        .or_insert_with(|| self.list_guide_anchor(row.line + 1));
                    if let Some(next) = next
                        && next.indent == anchor.indent
                        && center + stroke > next.bounds[0] - clearance
                        && center - stroke < next.bounds[0] + next.bounds[2] + clearance
                    {
                        bottom = bottom.max(next.bounds[1] - clearance);
                    }
                }
                let left = (center - stroke * 0.5).round();
                let (top, bottom) = (top.max(y0), bottom.min(y1));
                if left >= x0 && left + stroke <= x1 && bottom > top {
                    // A neighbouring marker may extend beyond its row box. Protect
                    // all of its painted ink, including the guide shader's 1px fringe.
                    let mut segments = vec![(top, bottom)];
                    for line in row.line.saturating_sub(1)..=row.line.saturating_add(1) {
                        if let Some(bounds) = *markers
                            .entry(line)
                            .or_insert_with(|| self.painted_list_marker_bounds(line))
                        {
                            clipping::exclude_marker(
                                &mut segments,
                                left,
                                stroke,
                                bounds,
                                clearance + 1.0,
                            );
                        }
                    }
                    rects.extend(
                        segments
                            .into_iter()
                            .map(|(top, bottom)| [left, top, stroke, bottom - top]),
                    );
                }
            }
        }
        (rects, visited)
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
