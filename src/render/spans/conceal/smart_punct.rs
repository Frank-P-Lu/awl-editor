//! Smart-punctuation source concealment: unlike every other `ConcealKind`
//! this hides real prose bytes (never syntax markup) behind a PAINTED
//! substitute glyph rather than nothing — the bare-URL ellipsis's precedent,
//! generalized from one fixed glyph to a small closed roster of three.

use super::*;

/// Which display glyph a concealed smart-punctuation span's own literal bytes
/// map to — re-derived from the raw source rather than carried on the span
/// itself (the `is_bare_url_tail` / `fence_line_lang` "render re-derives from
/// source" precedent), so the concealed byte range and the painted glyph can
/// never disagree. `None` only for a byte range that isn't actually one of
/// the three recognized runs — defensive, not a real case: every span this is
/// called on came from `push_smart_punct_spans`, which only ever emits an
/// exact `--`/`---`/`...` match.
pub(in crate::render) fn smart_punct_kind_for(
    line_text: &str,
    local_range: std::ops::Range<usize>,
) -> Option<crate::markdown::SmartPunctKind> {
    use crate::markdown::SmartPunctKind::*;
    match line_text.get(local_range)? {
        "--" => Some(EnDash),
        "---" => Some(EmDash),
        "..." => Some(Ellipsis),
        _ => None,
    }
}

/// Width reserved for the single painted smart-punctuation substitute glyph
/// (en dash, em dash, or ellipsis) that replaces a concealed run — the exact
/// `bare_url_ellipsis_slot` shape and value, generous enough for the widest of
/// the three (the ellipsis) so one constant covers all three kinds without a
/// per-kind branch, exactly like the bare-URL tail's ONE reserved slot covers
/// every URL's own "…" regardless of its actual path length.
pub(in crate::render) fn smart_punct_slot(line_height: f32) -> f32 {
    line_height * 0.9
}

/// Force a concealed smart-punctuation span's leading scalar to the reserved
/// substitute slot ([`smart_punct_slot`]) and zero-width the rest — mirrors
/// [`super::footnotes::add_footnote_conceal_spans`] / [`super::bare_url::add_bare_url_conceal_spans`]'s
/// forced-first-scalar shape exactly. Unlike those two this has no
/// "not actually mine" fallthrough: every `ConcealKind::SmartPunct` span
/// always gets this treatment (there is no SCHEME/TAIL-style second case), so
/// this is called unconditionally rather than returning a dispatch bool.
pub(super) fn add_smart_punct_conceal_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    lo: usize,
    hi: usize,
    hidden: &Attrs<'static>,
    line_height: f32,
) {
    let first_len = line_text[(lo - line_doc_start)..]
        .chars()
        .next()
        .map_or(0, char::len_utf8);
    let first_end = (lo + first_len).min(hi);
    if first_end > lo {
        let slot = smart_punct_slot(line_height);
        let forcing = hidden
            .clone()
            .letter_spacing(slot / CONCEAL_ZERO_WIDTH_FONT_SIZE);
        al.add_span(
            (lo - line_doc_start)..(first_end - line_doc_start),
            &forcing,
        );
    }
    if first_end < hi {
        al.add_span((first_end - line_doc_start)..(hi - line_doc_start), hidden);
    }
}
