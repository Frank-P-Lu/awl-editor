//! Bare-URL source concealment and its quiet ellipsis affordance.

use super::*;

/// True when a concealed [`crate::markdown::ConcealKind::BareUrl`] span's OWN
/// byte range (`local_start`, its start offset within `line_text`) is the URL's
/// TAIL half rather than its SCHEME half — the two spans
/// `crate::markdown::spans::markers::push_bare_url_spans` pushes per bare URL. A
/// TAIL always opens on the first `/` or `?` after the authority (the exact
/// boundary `crate::markdown::spans::detect::bare_url_split` cuts on); a SCHEME
/// span always opens on `h` (`http://`/`https://`). Re-derived from the raw byte
/// rather than carried on the span itself, mirroring
/// [`crate::markdown::fence_line_lang`]'s "render re-derives from source"
/// precedent — one predicate, shared by the reserved-slot forcing here
/// ([`add_bare_url_conceal_spans`]) and the ellipsis-mark collection in
/// `render::rects::ensure_ornament_lists`, so the two can never disagree about
/// which half of a bare URL paints the ellipsis.
pub(in crate::render) fn is_bare_url_tail(line_text: &str, local_start: usize) -> bool {
    matches!(
        line_text.as_bytes().get(local_start),
        Some(b'/') | Some(b'?')
    )
}

/// Force a concealed bare-URL TAIL span's leading scalar to the reserved
/// ellipsis slot ([`SubstituteAdvances::ellipsis_slot`] — the substitute's own
/// shaped advance, never a row-metric fraction) and zero-width the rest; a SCHEME
/// span (or anything [`is_bare_url_tail`] doesn't recognize) is left for the
/// generic uniform collapse in [`super::add_wysiwyg_conceal_spans`] — returning
/// `false` so the caller falls through to it. Mirrors
/// [`super::footnotes::add_footnote_conceal_spans`]'s forced-first-scalar shape
/// exactly, minus the "which number" lookup: an ellipsis has no payload, only a
/// position.
pub(super) fn add_bare_url_conceal_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    lo: usize,
    hi: usize,
    hidden: &Attrs<'static>,
    advances: SubstituteAdvances,
) -> bool {
    if !is_bare_url_tail(line_text, lo - line_doc_start) {
        return false;
    }
    let first_len = line_text[(lo - line_doc_start)..]
        .chars()
        .next()
        .map_or(0, char::len_utf8);
    let first_end = (lo + first_len).min(hi);
    if first_end > lo {
        let slot = advances.ellipsis_slot();
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
    true
}
