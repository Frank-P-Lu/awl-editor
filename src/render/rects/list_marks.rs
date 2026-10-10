//! One placement owner for ordinary bullets and task-state markers.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) enum ListLineKind {
    Bullet,
    Task(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) struct ListLine {
    pub(in crate::render) line: usize,
    pub(in crate::render) marker_col: usize,
    pub(in crate::render) depth: usize,
    pub(in crate::render) kind: ListLineKind,
}

impl ListLine {
    fn body_col(self) -> usize {
        self.marker_col
            + match self.kind {
                ListLineKind::Bullet => 2,
                ListLineKind::Task(_) => 6,
            }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct ListMark {
    pub(in crate::render) top: f32,
    /// The first visual row's real shaped prose baseline.
    pub(in crate::render) baseline: f32,
    pub(in crate::render) left: f32,
    pub(in crate::render) kind: ListLineKind,
    pub(in crate::render) glyph: char,
    pub(in crate::render) scale: f32,
    /// The normalized preview prefix's measured advance, used to seat the glyph.
    pub(in crate::render) slot_width: f32,
    /// The shared visual containment boundary before body text.
    pub(in crate::render) paint_width: f32,
    pub(in crate::render) ink: [u8; 4],
}

/// Parsed list rows for one immutable document snapshot. The sorted span sweep
/// is shared by the retained ornament cache and incremental-text ownership
/// comparison, so neither path falls back to a line-by-span rescan.
pub(in crate::render) fn parsed_list_lines(
    lines: &[&str],
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
) -> Vec<ListLine> {
    let mut sorted_spans: Vec<_> = md_spans.iter().collect();
    sorted_spans.sort_by_key(|(range, _)| range.start);
    let mut next_span = 0usize;
    let mut active = Vec::new();
    let mut start = 0usize;
    let mut list_lines = Vec::new();
    for (line_i, text) in lines.iter().copied().enumerate() {
        let end = start + text.len();
        while next_span < sorted_spans.len() && sorted_spans[next_span].0.start < end + 1 {
            active.push(sorted_spans[next_span]);
            next_span += 1;
        }
        active.retain(|(range, _)| range.end > start);
        if let Some(item) =
            crate::markdown::rich_unordered_list_item(text, start, active.iter().copied())
        {
            list_lines.push(ListLine {
                line: line_i,
                marker_col: item.marker_col,
                depth: item.depth,
                kind: item.task.map_or(ListLineKind::Bullet, ListLineKind::Task),
            });
        }
        start = end + 1;
    }
    list_lines
}

impl TextPipeline {
    /// Populate parsed unordered-list membership independently of shaped
    /// geometry, then reuse it for layout and marker paint.
    pub(in crate::render) fn ensure_list_lines(&self) {
        if self.ornament_cache.list_version.get() == Some(self.reshape_count) {
            return;
        }
        let line_texts: Vec<_> = self.buffer.lines.iter().map(|line| line.text()).collect();
        *self.ornament_cache.list_lines.borrow_mut() =
            parsed_list_lines(&line_texts, &self.md_spans);
        self.ornament_cache
            .list_version
            .set(Some(self.reshape_count));
    }

    pub(in crate::render) fn list_lines_snapshot(&self) -> Vec<ListLine> {
        self.ensure_list_lines();
        self.ornament_cache.list_lines.borrow().clone()
    }

    /// Map the normalized preview rail back to its collapsed source prefix.
    ///
    /// The hanging inset creates real pointer space before the prose while the
    /// authored indent, marker and task checkbox have near-zero shaped advance.
    /// Paint and hit testing therefore share the parsed list row, its measured
    /// marker slot and the first row's actual body boundary. Revealed rows have
    /// no inset and keep ordinary shaped hit testing.
    pub(in crate::render) fn concealed_list_prefix_hit_col(
        &self,
        line: usize,
        first_visual_row: bool,
        ltr: bool,
        target_x: f32,
    ) -> Option<usize> {
        if !first_visual_row || !ltr {
            return None;
        }
        self.ensure_list_lines();
        let item = {
            let lines = self.ornament_cache.list_lines.borrow();
            lines
                .binary_search_by_key(&line, |item| item.line)
                .ok()
                .map(|index| lines[index])?
        };
        let hanging_inset = self.buffer.lines.get(line)?.hanging_inset();
        if hanging_inset <= 0.0 {
            return None;
        }
        let first_row = self.line_rows_local_shaped(line)?.into_iter().next()?;
        let body_col = item.body_col();
        let body_x = *first_row.row.xs.get(body_col)?;
        if target_x > body_x {
            return None;
        }
        let slot_width = self
            .list_layout
            .metrics
            .marker_slot
            .min(hanging_inset.max(1.0));
        let marker_left = (body_x - slot_width).max(0.0);
        if item.marker_col > 0 && target_x < marker_left {
            let fraction = (target_x / marker_left).clamp(0.0, 1.0);
            return Some((fraction * item.marker_col as f32).round() as usize);
        }
        let marker_span = body_col - item.marker_col;
        let fraction = if slot_width > 0.0 {
            ((target_x - marker_left) / slot_width).clamp(0.0, 1.0)
        } else {
            1.0
        };
        Some(item.marker_col + (fraction * marker_span as f32).round() as usize)
    }

    /// Visible unordered-list markers in document order. Text-derived membership
    /// is reshape-cached; each frame filters to visible, unrevealed lines and
    /// resolves every measured marker slot through one batched row-geometry walk.
    pub(in crate::render) fn list_marks(&self) -> Vec<ListMark> {
        self.list_marks_with_work().0
    }

    /// The production marker pass plus its candidate-visit witness. The sorted
    /// cache is binary-searched by the exact screen-space cull band before the
    /// loop. Each paintable line then reads its own retained first layout row;
    /// no full-document visual-row partition is scanned per frame.
    pub(in crate::render) fn list_marks_with_work(&self) -> (Vec<ListMark>, usize) {
        if !self.md_enabled {
            return (Vec::new(), 0);
        }
        self.ensure_ornament_lists();
        let text_left = self.text_left();
        let selection_touch = self.selection_touch();
        let mut items = Vec::new();
        let margin = self.metrics.line_height * OFFSCREEN_CULL_MARGIN_ROWS.0;
        let lines = self.ornament_cache.list_lines.borrow();
        let first = lines.partition_point(|item| self.line_ornament_top(item.line) <= -margin);
        let past = lines
            .partition_point(|item| self.line_ornament_top(item.line) < self.window_h + margin);
        let mut visits = 0;
        for item in lines[first..past].iter().copied() {
            visits += 1;
            let li = item.line;
            if self.line_is_revealed(li, selection_touch.as_ref())
                || !self.line_ornament_visible(li)
            {
                continue;
            }
            items.push((item, self.line_ornament_top(li)));
        }
        let active = crate::theme::active();
        let marks = items
            .into_iter()
            .map(|(item, top)| {
                let row = self
                    .line_rows_local_shaped(item.line)
                    .and_then(|rows| rows.into_iter().next());
                let body_col = item.body_col();
                let hanging_inset = self
                    .buffer
                    .lines
                    .get(item.line)
                    .map(glyphon::cosmic_text::BufferLine::hanging_inset)
                    .unwrap_or(0.0);
                let body_x = row
                    .and_then(|row| row.row.xs.get(body_col).copied())
                    .unwrap_or(hanging_inset);
                let slot_width = self
                    .list_layout
                    .metrics
                    .marker_slot
                    .min(hanging_inset.max(1.0));
                let (glyph, scale, ink) = match item.kind {
                    ListLineKind::Bullet => {
                        let ch = active.bullet_for_depth(item.depth);
                        (
                            ch,
                            active.bullet_scale_for(ch),
                            active.bullet_ink_for(ch).rgba_bytes(),
                        )
                    }
                    ListLineKind::Task(checked) => (
                        crate::theme::task_marker(checked),
                        crate::theme::TASK_MARKER_SCALE,
                        active.muted.rgba_bytes(),
                    ),
                };
                ListMark {
                    top,
                    baseline: self.line_ornament_baseline(item.line),
                    left: text_left + body_x - slot_width,
                    kind: item.kind,
                    glyph,
                    scale,
                    slot_width,
                    paint_width: slot_width,
                    ink,
                }
            })
            .collect();
        (marks, visits)
    }

    #[cfg(test)]
    pub(in crate::render) fn cached_list_line_count(&self) -> usize {
        self.ensure_ornament_lists();
        self.ornament_cache.list_lines.borrow().len()
    }

    /// Ordinary-bullet compatibility view used by the existing bullet-specific
    /// geometry laws. Production drawing consumes [`Self::list_marks`] directly.
    pub(in crate::render) fn bullet_marks(&self) -> Vec<(f32, f32, char)> {
        self.list_marks()
            .into_iter()
            .filter_map(|mark| match mark.kind {
                ListLineKind::Bullet => Some((mark.top, mark.left, mark.glyph)),
                ListLineKind::Task(_) => None,
            })
            .collect()
    }
}
