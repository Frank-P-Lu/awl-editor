use super::*;

pub(super) type ListOwnership = std::collections::BTreeMap<usize, usize>;

pub(in crate::render) struct ListLayoutState {
    pub(in crate::render) metrics: ListLayoutMetrics,
    pub(in crate::render) hanging_lines: Vec<usize>,
    pub(in crate::render) continuations: Vec<crate::markdown::ListContinuation>,
}

impl ListLayoutState {
    pub(in crate::render) fn new(font_system: &mut FontSystem, metrics: Metrics) -> Self {
        Self {
            metrics: ListLayoutMetrics::shape(
                font_system,
                metrics,
                theme::active().font,
                theme::active().mono,
            ),
            hanging_lines: Vec::new(),
            continuations: Vec::new(),
        }
    }

    pub(in crate::render) fn preview_inset(&self, marker_col: usize, wrap_width: f32) -> f32 {
        self.metrics
            .hanging_inset(marker_col, theme::active().list_indent_scale)
            .min((wrap_width - 1.0).max(0.0))
    }
}

pub(super) fn parsed_list_ownership(
    lines: &[&str],
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    continuations: &[crate::markdown::ListContinuation],
) -> ListOwnership {
    let mut owners = super::super::rects::parsed_list_lines(lines, md_spans)
        .into_iter()
        .map(|item| (item.line, item.marker_col))
        .collect::<ListOwnership>();
    owners.extend(
        continuations
            .iter()
            .map(|item| (item.line, item.marker_col)),
    );
    owners
}

fn retained_ownership_changed(
    old: &ListOwnership,
    new: &ListOwnership,
    change: change::TextChange,
) -> bool {
    let translated_old = old.iter().filter_map(|(&line, &marker_col)| {
        if line < change.prefix {
            Some((line, marker_col))
        } else if line >= change.old_end {
            Some((change.new_end + line - change.old_end, marker_col))
        } else {
            None
        }
    });
    let retained_new = new.iter().filter_map(|(&line, &marker_col)| {
        (line < change.prefix || line >= change.new_end).then_some((line, marker_col))
    });
    translated_old.ne(retained_new)
}

impl TextPipeline {
    /// Rebuild retained line attributes when a document-scope Markdown change
    /// moves unchanged text into or out of parsed list ownership.
    pub(super) fn reconcile_retained_list_ownership(
        &mut self,
        old: &ListOwnership,
        new: &ListOwnership,
        change: &mut change::TextChange,
        old_line_count: usize,
        new_line_count: usize,
    ) {
        let ownership_changed = retained_ownership_changed(old, new, *change);
        if ownership_changed {
            change.prefix = 0;
            change.old_end = old_line_count;
            change.new_end = new_line_count;
            change.geometry_safe = false;
        }
        if old_line_count != new_line_count || ownership_changed {
            for line_i in std::mem::take(&mut self.list_layout.hanging_lines) {
                if let Some(line) = self.buffer.lines.get_mut(line_i) {
                    line.set_hanging_inset(0.0);
                }
            }
        }
    }

    /// Apply one measured hanging inset to every concealed list paragraph,
    /// then shape. Paint, caret, selection and hit testing all read this layout.
    pub(in crate::render) fn shape_document(&mut self) {
        let list_lines = self.list_lines_snapshot();
        let selection_touch = self.selection_touch();
        let wrap_width = self.text_wrap_width();
        let mut insets = list_lines
            .iter()
            .filter(|item| {
                self.md_enabled && !self.line_is_revealed(item.line, selection_touch.as_ref())
            })
            .map(|item| {
                (
                    item.line,
                    self.list_layout.preview_inset(item.marker_col, wrap_width),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        for continuation in &self.list_layout.continuations {
            if self.md_enabled
                && !self.line_is_revealed(continuation.line, selection_touch.as_ref())
            {
                insets.insert(
                    continuation.line,
                    self.list_layout
                        .preview_inset(continuation.marker_col, wrap_width),
                );
            }
        }
        let mut changed = false;
        for line_i in std::mem::take(&mut self.list_layout.hanging_lines) {
            if !insets.contains_key(&line_i)
                && let Some(line) = self.buffer.lines.get_mut(line_i)
            {
                changed |= line.set_hanging_inset(0.0);
            }
        }
        for (line_i, inset) in insets {
            if let Some(line) = self.buffer.lines.get_mut(line_i) {
                changed |= line.set_hanging_inset(inset);
                if inset > 0.0 {
                    self.list_layout.hanging_lines.push(line_i);
                }
            }
        }
        if changed {
            self.row_geom.invalidate();
        }
        self.buffer.shape_until_scroll(&mut self.font_system, false);
    }
}
