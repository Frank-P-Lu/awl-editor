//! Parsed list-marker concealment for rich bullets and task states.

use super::*;

pub(in crate::render) fn add_bullet_conceal_span(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
) {
    let Some(item) =
        crate::markdown::rich_unordered_list_item(line_text, line_doc_start, md_spans.iter())
    else {
        return;
    };
    let hidden = base.clone().color(RULE_CONCEAL_COLOR);
    al.add_span(item.marker_col..item.marker_col + 1, &hidden);
}

/// Collapse a parsed task marker's `[ ]` / `[x]` source while the shared
/// list-marker painter occupies the ordinary `- ` marker slot. Its parser-owned
/// trailing separator stays at full advance, keeping marker ink clear of the
/// body without inventing layout space. Invalid `[]` text has no `MdKind::Task`
/// span and therefore remains visible content.
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
        let mut end = range.end - line_doc_start;
        if matches!(
            line_text.as_bytes().get(end.saturating_sub(1)),
            Some(b' ' | b'\t')
        ) {
            end -= 1;
        }
        al.add_span(start..end, &hidden);
    }
}
