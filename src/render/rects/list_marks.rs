//! One placement owner for ordinary bullets and task-state markers.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) enum ListLineKind {
    Bullet,
    Task(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ListLine {
    pub(super) line: usize,
    pub(super) marker_col: usize,
    pub(super) depth: usize,
    pub(super) kind: ListLineKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct ListMark {
    pub(in crate::render) top: f32,
    pub(in crate::render) left: f32,
    pub(in crate::render) kind: ListLineKind,
    pub(in crate::render) glyphs: crate::theme::TaskMarkerGlyphs,
    pub(in crate::render) scale: f32,
    /// The raw unordered marker's retained advance, used to seat the glyph.
    pub(in crate::render) slot_width: f32,
    /// Source-derived room before body text: marker advance plus any retained
    /// task separator. This is the visual containment boundary, not new layout.
    pub(in crate::render) paint_width: f32,
    pub(in crate::render) ink: [u8; 4],
}

impl TextPipeline {
    /// Visible unordered-list markers in document order. Text-derived membership
    /// is reshape-cached; each frame filters to visible, unrevealed lines and
    /// resolves every indented marker through one batched row-geometry walk.
    pub(in crate::render) fn list_marks(&self) -> Vec<ListMark> {
        self.list_marks_with_work().0
    }

    /// The production marker pass plus its candidate-visit witness. The sorted
    /// cache is binary-searched by the exact screen-space cull band before the
    /// loop, so the per-frame walk is bounded by paintable list rows rather than
    /// by the document's total list count.
    pub(in crate::render) fn list_marks_with_work(&self) -> (Vec<ListMark>, usize) {
        if !self.md_enabled {
            return (Vec::new(), 0);
        }
        self.ensure_ornament_lists();
        let text_left = self.text_left();
        let selection_touch = self.selection_touch();
        let mut items = Vec::new();
        let mut geometry_lines = std::collections::BTreeSet::new();
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
            if item.marker_col > 0 || matches!(item.kind, ListLineKind::Task(_)) {
                geometry_lines.insert(li);
            }
            items.push((item, self.line_ornament_top(li)));
        }
        let rows_by_line = if geometry_lines.is_empty() {
            std::collections::HashMap::new()
        } else {
            self.visual_rows_for_lines(&geometry_lines)
        };
        let active = crate::theme::active();
        let marks = items
            .into_iter()
            .map(|(item, top)| {
                let x = if item.marker_col == 0 {
                    0.0
                } else {
                    rows_by_line
                        .get(&item.line)
                        .and_then(|rows| rows.first())
                        .and_then(|row| row.xs.get(item.marker_col).copied())
                        .unwrap_or(0.0)
                };
                let slot_width = match item.kind {
                    ListLineKind::Task(_) => rows_by_line
                        .get(&item.line)
                        .and_then(|rows| rows.first())
                        .and_then(|row| {
                            let left = row.xs.get(item.marker_col)?;
                            let right = row.xs.get(item.marker_col + 2)?;
                            Some((right - left).max(1.0))
                        })
                        .unwrap_or((self.metrics.char_width * 2.0).max(1.0)),
                    ListLineKind::Bullet => (self.metrics.char_width * 2.0).max(1.0),
                };
                let paint_width = match item.kind {
                    ListLineKind::Task(_) => rows_by_line
                        .get(&item.line)
                        .and_then(|rows| rows.first())
                        .and_then(|row| {
                            let left = row.xs.get(item.marker_col)?;
                            let body = row.xs.get(item.marker_col + 6)?;
                            Some((body - left).max(slot_width))
                        })
                        .unwrap_or(slot_width),
                    ListLineKind::Bullet => slot_width,
                };
                let (glyphs, scale, ink) = match item.kind {
                    ListLineKind::Bullet => {
                        let ch = active.bullet_for_depth(item.depth);
                        (
                            crate::theme::TaskMarkerGlyphs::Single(ch),
                            active.bullet_scale_for(ch),
                            active.bullet_ink_for(ch).rgba_bytes(),
                        )
                    }
                    ListLineKind::Task(checked) => (
                        active.task_marker.glyphs(checked),
                        crate::theme::TASK_MARKER_SCALE,
                        active.muted.rgba_bytes(),
                    ),
                };
                ListMark {
                    top,
                    left: text_left + x,
                    kind: item.kind,
                    glyphs,
                    scale,
                    slot_width,
                    paint_width,
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
            .filter_map(|mark| match (mark.kind, mark.glyphs) {
                (ListLineKind::Bullet, crate::theme::TaskMarkerGlyphs::Single(ch)) => {
                    Some((mark.top, mark.left, ch))
                }
                (ListLineKind::Bullet, crate::theme::TaskMarkerGlyphs::Overlay { .. }) => {
                    unreachable!("ordinary bullets are single glyphs")
                }
                (ListLineKind::Task(_), _) => None,
            })
            .collect()
    }
}
