//! Shared indentation attributes; ordered markers retain their raw syntax.
use super::*;

pub(in crate::render) struct ListIndentSpanCtx<'a> {
    pub(in crate::render) base: &'a Attrs<'static>,
    pub(in crate::render) base_font_size: f32,
    pub(in crate::render) row_lh: f32,
    pub(in crate::render) hanging_preview: bool,
    pub(in crate::render) md_spans: &'a [(std::ops::Range<usize>, crate::markdown::MdKind)],
    pub(in crate::render) list_continuations: &'a [crate::markdown::ListContinuation],
}

pub(in crate::render) fn add_list_indent_span(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    ctx: ListIndentSpanCtx<'_>,
) {
    // Use the same parser-owned enrollment as marker concealment, painting and
    // hanging layout. A list-shaped line inside a code block must retain its
    // literal indentation and can never receive only one piece of list layout.
    let rich =
        crate::markdown::rich_unordered_list_item(line_text, line_doc_start, ctx.md_spans.iter());
    let continuation = ctx
        .list_continuations
        .binary_search_by_key(&line_doc_start, |item| item.line_doc_start)
        .ok()
        .map(|index| ctx.list_continuations[index]);
    let ordered = crate::markdown::list_item(line_text).filter(|item| {
        item.ordered
            && ctx.md_spans.iter().any(|(range, kind)| {
                *kind == crate::markdown::MdKind::ListMarker
                    && range.start == line_doc_start
                    && range.end >= line_doc_start + item.content
            })
    });
    let (source_indent, marker_col) = match (rich, continuation) {
        (Some(item), _) => (item.marker_col, item.marker_col),
        (None, Some(item)) => (item.source_indent, item.marker_col),
        (None, None) => match ordered {
            Some(item) => (item.indent, item.indent),
            None => return,
        },
    };
    if ctx.hanging_preview && ordered.is_none() {
        if source_indent > 0 {
            let hidden = ctx
                .base
                .clone()
                .metrics(GlyphMetrics::new(CONCEAL_ZERO_WIDTH_FONT_SIZE, ctx.row_lh));
            al.add_span(0..source_indent, &hidden);
        }
        return;
    }
    // A revealed continuation paragraph shows its literal source indentation.
    // Only an explicit marker line participates in the world's editing-time
    // list-indent widening.
    if (rich.is_none() && ordered.is_none()) || marker_col == 0 {
        return;
    }
    let list_indent_scale = crate::theme::active().list_indent_scale;
    if (list_indent_scale - 1.0).abs() < 1e-3 {
        return; // an authored unity scale needs no span
    }
    let wide = ctx.base.clone().metrics(GlyphMetrics::new(
        ctx.base_font_size * list_indent_scale,
        ctx.row_lh,
    ));
    al.add_span(0..marker_col, &wide);
}
