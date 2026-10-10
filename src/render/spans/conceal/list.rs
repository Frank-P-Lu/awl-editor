//! Parsed list-marker concealment for rich bullets and task states.

use super::*;

/// Extra tracking on the normalized monospace separator, measured in ems. The
/// result carries the complete Nishiki checked drawing plus a clear body gap.
pub(in crate::render) const LIST_MARKER_GAP_TRACKING: f32 = 0.75;

pub(in crate::render) fn add_bullet_conceal_span(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    row_lh: f32,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
) {
    let Some(item) =
        crate::markdown::rich_unordered_list_item(line_text, line_doc_start, md_spans.iter())
    else {
        return;
    };
    // Collapse the complete authored prefix. The paragraph's measured hanging
    // inset now owns both the marker rail and the prose gap on every visual row.
    let hidden = base
        .clone()
        .metrics(GlyphMetrics::new(CONCEAL_ZERO_WIDTH_FONT_SIZE, row_lh))
        .color(RULE_CONCEAL_COLOR);
    al.add_span(item.marker_col..item.marker_col + 2, &hidden);
}

/// Collapse a parsed task marker's `[ ] ` / `[x] ` source while the shared
/// list-marker painter occupies the ordinary normalized list-prefix slot.
/// Invalid `[]` text has no `MdKind::Task` span and remains visible content.
pub(in crate::render) fn add_task_conceal_span(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    row_lh: f32,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
) {
    let Some(item) =
        crate::markdown::rich_unordered_list_item(line_text, line_doc_start, md_spans.iter())
    else {
        return;
    };
    let Some(checked) = item.task else {
        return;
    };
    let hidden = base
        .clone()
        .color(RULE_CONCEAL_COLOR)
        .metrics(GlyphMetrics::new(CONCEAL_ZERO_WIDTH_FONT_SIZE, row_lh));
    let prefix_end = crate::markdown::list_item(line_text)
        .map(|list| line_doc_start + list.content)
        .expect("rich list item has a raw list prefix");
    if let Some((range, _)) = md_spans.iter().find(|(range, kind)| {
        *kind == crate::markdown::MdKind::Task(checked) && range.start == prefix_end
    }) {
        let start = range.start - line_doc_start;
        let end = range.end - line_doc_start;
        al.add_span(start..end, &hidden);
    }
}
