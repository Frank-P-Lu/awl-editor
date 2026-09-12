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
    /// The paint-only vertical seat. This can carry a glyph-specific optical
    /// correction without changing marker layout or its retained prefix slot.
    pub(in crate::render) paint_top: f32,
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

impl TextPipeline {
    /// Visible unordered-list markers in document order. Text-derived membership
    /// is reshape-cached; each frame filters to visible, unrevealed lines and
    /// resolves every measured marker slot through one batched row-geometry walk.
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
            geometry_lines.insert(li);
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
                let row = rows_by_line.get(&item.line).and_then(|rows| rows.first());
                let x = row
                    .and_then(|row| row.xs.get(item.marker_col).copied())
                    .unwrap_or(0.0);
                let body_col = item.marker_col
                    + match item.kind {
                        ListLineKind::Bullet => 2,
                        ListLineKind::Task(_) => 6,
                    };
                let slot_width = row
                    .and_then(|row| {
                        let left = row.xs.get(item.marker_col)?;
                        let body = row.xs.get(body_col)?;
                        Some((body - left).max(1.0))
                    })
                    .unwrap_or((self.metrics.char_width * 2.0).max(1.0));
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
                    paint_top: top
                        + crate::theme::bullet_optical_drop(glyph, self.metrics.font_size),
                    left: text_left + x,
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
