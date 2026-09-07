//! WYSIWYG reveal/conceal: the zero-width markup-hiding mechanism, the
//! caret/selection reveal rule, and the per-line conceal-state predicates
//! it's built from.

use super::*;

mod bare_url;
mod cell;
mod footnotes;
mod smart_punct;
mod substitutes;
pub(in crate::render) use bare_url::is_bare_url_tail;
pub(in crate::render) use cell::cell_inline_attrs;
pub(in crate::render) use smart_punct::{shape_smart_punct_glyph, smart_punct_kind_for};
pub(in crate::render) use substitutes::{SubstituteAdvances, shape_footnote_number};

pub(in crate::render) const RULE_CONCEAL_COLOR: glyphon::Color = glyphon::Color::rgba(0, 0, 0, 0);

/// WYSIWYG v1.1 — TRUE ZERO-WIDTH conceal (the live-review headline fix). v1
/// hid a `ConcealMarkup` span with transparent ink alone, which kept the
/// marker glyphs' natural ADVANCE: a concealed `"## "` still indented the
/// heading off the column edge, and concealed `"**"`/`"*"` left a visible
/// word-gap ("almost  italics"). The cure rides the SAME per-span `AttrsList`
/// seam CJK/syntax/markdown already use: `Attrs::metrics` lets ONE byte-range
/// override its font size independent of the rest of the line
/// ([`scaled_base_attrs`] already proved this for headings, just per-LINE
/// instead of per-span). cosmic-text computes a glyph's pixel advance as
/// `metrics_opt.font_size * glyph.x_advance` at LAYOUT time — shaping itself
/// (kerning/ligatures/clustering) happens earlier and is UNAFFECTED, because
/// `Attrs::compatible` (the run-splitting test) checks family/stretch/style/
/// weight only, never `metrics_opt` — so a concealed run shapes seamlessly
/// alongside its visible neighbors and only its FINAL on-screen width
/// collapses. A near-zero (not exactly `0.0`, defensively) font size shrinks
/// the advance to sub-pixel — true zero-width — while glyphon already
/// tolerates a zero-size rasterized glyph bitmap (`width == 0 || height ==
/// 0`, `text_render.rs`), so nothing panics; the alpha-0 color means nothing
/// would draw regardless. The paired `Attrs::metrics` line-height half MUST
/// be set to the line's own (already heading-scaled) row height, never a
/// small value — cosmic-text keys a visual row's height off the MAX
/// `line_height_opt` across every glyph on the row, but only among glyphs
/// that carry an EXPLICIT override; a stray small value here would apply
/// even when every other glyph on the row has none, shrinking the WHOLE row
/// rather than "staying keyed to the surviving glyphs" (see
/// [`add_wysiwyg_conceal_spans`]'s caller in [`build_line_attrs`], which
/// threads the line's real scaled height through). Hit-testing / caret
/// placement need no new logic: `col_in_run`/`col_in_row` (`geometry.rs`)
/// already walk glyphs sequentially comparing midpoints, so several
/// near-coincident zero-width x boundaries just resolve to the nearest one
/// in sequence — no panic, no infinite loop, a valid (if visually ambiguous)
/// byte column.
const CONCEAL_ZERO_WIDTH_FONT_SIZE: f32 = 0.01;

#[cfg(not(target_arch = "wasm32"))]
pub(in crate::render) const IMAGE_MISSING_ROW_LINES: f32 = 3.0;

pub(in crate::render) const IMAGE_MAX_VIEWPORT_FRAC: f32 = 0.65;

#[cfg(not(target_arch = "wasm32"))]
pub(in crate::render) fn image_display_size(
    intrinsic_w: u32,
    intrinsic_h: u32,
    width_hint: Option<u32>,
    wrap_width: f32,
    max_h: f32,
) -> (f32, f32) {
    let iw = (intrinsic_w.max(1)) as f32;
    let ih = (intrinsic_h.max(1)) as f32;
    let desired = width_hint.map(|h| h as f32).unwrap_or(iw);
    let w = desired.min(wrap_width.max(1.0)).max(1.0);
    let h = (w * ih / iw).max(1.0);
    if max_h > 0.0 && h > max_h {
        let scale = max_h / h;
        ((w * scale).max(1.0), max_h)
    } else {
        (w, h)
    }
}

pub(in crate::render) fn add_rule_conceal_span(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
) {
    let line_end = line_doc_start + line_text.len();
    let hidden = base.clone().color(RULE_CONCEAL_COLOR);
    for (r, kind) in md_spans {
        if *kind != crate::markdown::MdKind::Rule {
            continue;
        }
        let lo = r.start.max(line_doc_start);
        let hi = r.end.min(line_end);
        if lo < hi {
            al.add_span((lo - line_doc_start)..(hi - line_doc_start), &hidden);
        }
    }
}

pub(in crate::render) fn add_bullet_conceal_span(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    base: &Attrs<'static>,
) {
    let Some(it) = crate::markdown::list_item(line_text) else {
        return;
    };
    if it.ordered {
        return;
    }
    let hidden = base.clone().color(RULE_CONCEAL_COLOR);
    al.add_span(it.indent..it.indent + 1, &hidden);
}

pub(in crate::render) fn add_list_indent_span(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    base: &Attrs<'static>,
    base_font_size: f32,
    row_lh: f32,
) {
    let Some(it) = crate::markdown::list_item(line_text) else {
        return;
    };
    if it.indent == 0 {
        return; // depth 0: no indent run to widen, byte-identical
    }
    let list_indent_scale = crate::theme::active().list_indent_scale;
    if (list_indent_scale - 1.0).abs() < 1e-3 {
        return; // the PLAIN tier leaves the renderer byte-identical
    }
    let wide = base.clone().metrics(GlyphMetrics::new(
        base_font_size * list_indent_scale,
        row_lh,
    ));
    al.add_span(0..it.indent, &wide);
}

/// The document BYTE RANGE covering EVERY LINE the active selection TOUCHES —
/// `l0..=l1` inclusive (the ordered selection's earlier-to-later LINE
/// endpoints, column-agnostic — a one-character selection on a line touches
/// that line's WHOLE markup, exactly like the caret's own line does), from
/// line `l0`'s first byte to line `l1`'s last byte (its text, excluding the
/// trailing `\n`). `None` when there is no active (non-empty) selection.
///
/// THE shared "selection reveal" extent: [`wysiwyg_reveals`] tests a
/// concealable span's overlap against it, [`build_line_attrs`]'s rule/bullet
/// conceal gate tests a line's own byte range against it, and
/// [`super::TextPipeline::prepare_table_xray`]/`prepare_table_grid` extend
/// their caret-only table reveal with the SAME line set — so a selection can
/// never reveal a different line set than its own highlight
/// ([`super::TextPipeline::selection_rects`]' own `l0..=l1` line loop)
/// touches. Pure: `line_start(i)` is line `i`'s first document byte,
/// `line_len(i)` its text length (excluding `\n`) — callers pass whichever
/// source is in hand (the freshly-diffed lines mid-reshape, or the live
/// `self.buffer.lines`), so this has no `&self` of its own.
pub(in crate::render) fn selection_touch_bytes(
    sel: Option<((usize, usize), (usize, usize))>,
    line_start: impl Fn(usize) -> usize,
    line_len: impl Fn(usize) -> usize,
) -> Option<std::ops::Range<usize>> {
    let ((l0, _), (l1, _)) = sel?;
    Some(line_start(l0)..(line_start(l1) + line_len(l1)))
}

/// True when the active selection's touched-line byte extent
/// (`selection_touch`, see [`selection_touch_bytes`]) overlaps `range` at
/// all — a plain half-open-interval overlap test. THE single "does the
/// selection reveal this span" predicate: [`wysiwyg_reveals`] uses it for its
/// own `selected` decision; the SELECTION REVEAL path makes
/// [`super::TextPipeline::compute_image_layout`]'s `revealed_now`
/// and [`super::TextPipeline::images_report`]'s `revealed` reuse the SAME
/// call rather than re-deriving the overlap arithmetic a second (or third)
/// time, so an inline-image line's LAYOUT/DRAW reveal state can never
/// disagree with its markdown-markup reveal state.
pub(in crate::render) fn selection_touches(
    selection_touch: Option<&std::ops::Range<usize>>,
    range: &std::ops::Range<usize>,
) -> bool {
    selection_touch.is_some_and(|st| st.start < range.end && range.start < st.end)
}

/// THE reveal decision for ONE `ConcealMarkup` span — the single rule shared by
/// [`add_wysiwyg_conceal_spans`] (the renderer) and
/// [`super::TextPipeline::wysiwyg_report`] (the capture sidecar), so the two can
/// never drift on what "concealed" means. `range` is the span's own document
/// byte range; `conceal_off_cursor` is true when the caret is on a DIFFERENT
/// line than the span's own (irrelevant for the BLOCK-scoped kinds);
/// `cursor_byte` is the document byte offset of the caret's own line's first
/// byte. `selection_touch` ([`selection_touch_bytes`]) is the byte extent of
/// every line the ACTIVE SELECTION touches, or `None` with no selection.
///
/// SELECTION REVEAL (user-decided 2026-07-22): a line shows raw markdown when
/// the caret OR an active selection touches it — today's caret-only rule
/// widens to "caret line's byte range OR any `selection_touch` overlap",
/// tested identically for the line-scoped and block-scoped kinds below via
/// one `range`-overlap check, so a selected table/heading/fence reveals its
/// real source exactly like the caret landing on it would (a selected table
/// then shows raw `|` source, giving correct select/copy for free). Threaded
/// alongside the caret line, never duplicated: every caller computes
/// `selection_touch` ONCE per reshape/refresh and passes the same reference
/// through every span/line decision.
pub(crate) fn wysiwyg_reveals(
    ck: crate::markdown::ConcealKind,
    conceal_off_cursor: bool,
    cursor_byte: usize,
    range: &std::ops::Range<usize>,
    selection_touch: Option<&std::ops::Range<usize>>,
) -> bool {
    use crate::markdown::ConcealKind;
    let selected = selection_touches(selection_touch, range);
    match ck {
        // BLOCK-scoped: reveal iff the caret's line sits anywhere in the block,
        // OR the selection touches ANY line inside it. A frontmatter block
        // reuses the exact `Fence` rule (it has no body sub-span to carve out,
        // so the whole range conceals/reveals as one).
        ConcealKind::Fence | ConcealKind::Frontmatter => range.contains(&cursor_byte) || selected,
        // A TABLE's source NEVER un-conceals IN PLACE — THE X-RAY (the user's
        // canonized metaphor): the drawn GRID stays put so the document never
        // reflows during a keyboard walk / selection drag, and each revealed
        // row (caret's own, or any row the selection touches) floats its raw
        // source as one non-wrapping line OVER the dimmed grid cells
        // (`prepare_table_xray`), never by growing the document rows. So the
        // source rows stay zero-width (concealed) at every caret/selection
        // state — the float(s) + the caret redirect (`col_x_and_advance`) do
        // the reveal, not this in-place un-conceal; `prepare_table_grid`
        // extends the SAME caret-or-selection test to decide which rows swap.
        ConcealKind::Table => false,
        // LINE-scoped: reveal iff the caret is on THIS line, or the selection
        // touches it. An IMAGE ref is one line, and follows the "heading
        // model" (source reveals for editing when the caret lands, the drawn
        // image parks) exactly like every other line-scoped kind — see
        // `ConcealKind::Image`.
        ConcealKind::Heading
        | ConcealKind::Emphasis
        | ConcealKind::Code
        | ConcealKind::Highlight
        | ConcealKind::Strikethrough
        | ConcealKind::Image
        | ConcealKind::Link
        | ConcealKind::Blockquote
        | ConcealKind::Footnote
        | ConcealKind::BareUrl
        | ConcealKind::SmartPunct => !conceal_off_cursor || selected,
    }
}

pub(in crate::render) fn line_has_image_span(
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    line_doc_start: usize,
    line_end: usize,
) -> bool {
    use crate::markdown::{ConcealKind, MdKind};
    md_spans.iter().any(|(cr, ck)| {
        matches!(ck, MdKind::ConcealMarkup(ConcealKind::Image))
            && cr.start < line_end
            && cr.end > line_doc_start
    })
}

pub(in crate::render) fn image_line_has_other_content(
    line_text: &str,
    image_local: std::ops::Range<usize>,
) -> bool {
    let content_start = crate::markdown::list_item(line_text)
        .map(|it| it.content)
        .unwrap_or(0);
    let b = line_text.as_bytes();
    (content_start.min(b.len())..b.len())
        .any(|i| !(b[i].is_ascii_whitespace() || i >= image_local.start && i < image_local.end))
}

/// True when the REAL parse (`md_spans`, ground truth) rules this line a
/// genuine `MdKind::Rule` — the thematic-break ornament layer's own source of
/// truth (`prepare_ornaments` reads `md_spans`, never the bare-text scan). The
/// single-line heuristic [`crate::markdown::is_thematic_break`] cannot see
/// CONTEXT the real parse excludes on purpose — e.g. `---` living inside a
/// fenced code block's body reads as a break here but is never `MdKind::Rule`
/// in `md_spans`; this is the corroborating check [`md_line_scale`] requires
/// before growing a `---` row, so a bare-scan false positive never reserves
/// space for an ornament that `prepare_ornaments` will never draw. (A dash
/// underline directly under a paragraph — a setext H2 to CommonMark — is NOT
/// such a case: awl has no setext headings, so a qualifying underline gets a
/// real `Rule` span from `spans`' `Tag::Heading` arm and this function agrees.)
pub(in crate::render) fn line_has_rule_span(
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    line_doc_start: usize,
    line_end: usize,
) -> bool {
    use crate::markdown::MdKind;
    md_spans
        .iter()
        .any(|(cr, ck)| *ck == MdKind::Rule && cr.start < line_end && cr.end > line_doc_start)
}

pub(in crate::render) fn line_has_code_span(
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    line_doc_start: usize,
    line_end: usize,
) -> bool {
    use crate::markdown::MdKind;
    md_spans.iter().any(|(cr, ck2)| {
        matches!(ck2, MdKind::Code { .. } | MdKind::CodeSyntax { .. })
            && cr.start < line_end
            && cr.end > line_doc_start
    })
}

/// REVEAL-ON-CURSOR concealment for the WYSIWYG amendment ("if the caret is on
/// that line, show the actual markdown; otherwise show the preview") —
/// GENERALIZES [`add_rule_conceal_span`]/[`add_bullet_conceal_span`] to the five
/// markup kinds [`crate::markdown::ConcealKind`] names, over the SAME transparent
/// [`RULE_CONCEAL_COLOR`] mechanism: the marker glyphs still SHAPE (the row keeps
/// its height and the bytes stay editable) but draw invisibly.
///
/// Every [`crate::markdown::MdKind::ConcealMarkup`] span in `md_spans` is
/// scoped by its [`crate::markdown::ConcealKind`]:
///  - `Heading`/`Emphasis`/`Code`/`Highlight` are LINE-scoped: `conceal_off_cursor`
///    (the caller's "caret is on a DIFFERENT line" gate — the same one
///    `add_rule_conceal_span`/`add_bullet_conceal_span` already use) decides all
///    four in lockstep with the hr/bullet conceal.
///  - `Fence` is BLOCK-scoped: a fenced code block's marker LINES (the fence open
///    line + the fence close line, including the info string) conceal unless
///    `cursor_byte` — the document byte offset of the CARET'S OWN line's first
///    byte — falls anywhere inside the whole span's byte range (`r.contains`),
///    i.e. the caret sits somewhere in the block, not just on this one line. A
///    BODY line inside the block (one carrying its own `Code`/`CodeSyntax` span)
///    is NEVER concealed by this arm regardless — only the marker lines hide;
///    the always-present PANEL (drawn from this same span, see
///    `super::TextPipeline::fence_panel_rects`) is the block's affordance.
///
/// Gated on [`crate::markdown::wysiwyg_on`]: OFF is a total no-op, so `wysiwyg =
/// false` reproduces the always-visible markup this round shipped without,
/// byte-identically (no `ConcealMarkup` span is ever concealed, only ever dimmed
/// like plain `Markup` — see `md_attrs`). No-op when no `ConcealMarkup` span
/// intersects the line, keeping non-WYSIWYG lines untouched.
///
/// `line_height` is the LINE's own effective row height (already
/// heading-scaled — i.e. `base_line_height * scale`, exactly what
/// [`scaled_base_attrs`] used to build `base`/`lb`) — see
/// [`CONCEAL_ZERO_WIDTH_FONT_SIZE`]'s doc comment for why the concealed span's
/// paired line-height override must match it exactly rather than shrinking.
///
/// `image_force` (`Some((dh, target_advance_px))` only for a
/// MIXED off-cursor image line — see [`super::TextPipeline::image_force`]'s
/// field doc for the full mechanism): when set, the line's OWN `Image` conceal
/// span is NOT collapsed uniformly like every other kind here. Instead its
/// SECOND byte (the `[` of `![alt](p)`) gets an ADDITIONAL
/// `letter_spacing(target_advance_px / CONCEAL_ZERO_WIDTH_FONT_SIZE)` —
/// cosmic-text divides `letter_spacing` by the SAME per-glyph font-scale factor
/// it divides `x_advance` by, so the raw value must be pre-multiplied by
/// `1 / CONCEAL_ZERO_WIDTH_FONT_SIZE` to land the glyph's ACTUAL on-screen
/// advance at `target_advance_px` — forcing `Wrap::WordOrGlyph` to push it (and
/// the rest of the markup right after, which trivially fits alongside it) onto
/// a genuine NEW visual row of this same logical line, both at
/// `line_height = dh` (so that row reports `dh`, never the caption's own
/// `line_height`). NEVER the leading `!` itself: Unicode UAX14 rule LB13
/// ("do not break before `!`/`;`/`/`/`]`") forbids a line break immediately
/// before `!`, so forcing on `!` glues the CAPTION'S OWN LAST WORD to it as one
/// unbreakable unit — dragging that real, visible word onto the `dh`-tall row
/// too and re-stranding it (confirmed empirically; this was the actual bug in
/// an earlier build of this same mechanism, distinct from the prior round's
/// row-inflation bug but with the identical stranded-word symptom). `[` carries
/// no such restriction, so `!` stays a plain, non-forcing zero-width glyph and
/// `[` becomes the forcing one. The caption's OWN row is UNTOUCHED — no metrics
/// override at all touches it — so it never centers/strands away from its own
/// list marker (the prior round's bug). `None` (every other image line — bare,
/// revealed, or feature-off) is byte-identical to the uniform zero-width
/// treatment every other kind gets.
#[allow(clippy::too_many_arguments)]
pub(in crate::render) fn add_wysiwyg_conceal_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    conceal_off_cursor: bool,
    cursor_byte: usize,
    line_height: f32,
    image_force: Option<(f32, f32)>,
    selection_touch: Option<&std::ops::Range<usize>>,
    substitute_advances: Option<SubstituteAdvances>,
) {
    if !crate::markdown::wysiwyg_on() {
        return;
    }
    use crate::markdown::{ConcealKind, MdKind};
    let line_end = line_doc_start + line_text.len();
    // TRUE ZERO-WIDTH: transparent ink (draws nothing) PLUS a near-zero font
    // size (collapses the advance to sub-pixel), paired with the line's own
    // real line-height so the row's height stays keyed to its surviving
    // (unconcealed) glyphs — see this fn's doc comment + `CONCEAL_ZERO_WIDTH_FONT_SIZE`.
    let hidden = base
        .clone()
        .color(RULE_CONCEAL_COLOR)
        .metrics(GlyphMetrics::new(CONCEAL_ZERO_WIDTH_FONT_SIZE, line_height));
    for (r, kind) in md_spans {
        let ck = match *kind {
            MdKind::ConcealMarkup(ck) => ck,
            _ => continue,
        };
        if wysiwyg_reveals(ck, conceal_off_cursor, cursor_byte, r, selection_touch) {
            continue;
        }
        if ck == ConcealKind::Fence && line_has_code_span(md_spans, line_doc_start, line_end) {
            continue;
        }
        let lo = r.start.max(line_doc_start);
        let hi = r.end.min(line_end);
        if lo >= hi {
            continue;
        }
        if ck == ConcealKind::Image
            && let Some((dh, target_advance)) = image_force
        {
            let mut chars = line_text[(lo - line_doc_start)..].char_indices();
            let bang_len = chars.next().map_or(1, |(_, c)| c.len_utf8());
            let second_len = chars.next().map_or(0, |(_, c)| c.len_utf8());
            let bang_end = (lo + bang_len).min(hi);
            let force_end = (bang_end + second_len).min(hi);
            if bang_end > lo {
                al.add_span((lo - line_doc_start)..(bang_end - line_doc_start), &hidden);
            }
            // `[` onward (the trailing row): every glyph here MUST pair
            // with `dh`, not `line_height` — they land on the FORCED row,
            // whose height a smaller `line_height` value could otherwise
            // win the MAX against if `dh` is unusually small (a tiny image).
            let dh_hidden = hidden
                .clone()
                .metrics(GlyphMetrics::new(CONCEAL_ZERO_WIDTH_FONT_SIZE, dh));
            if force_end > bang_end {
                let forcing = dh_hidden
                    .clone()
                    .letter_spacing(target_advance / CONCEAL_ZERO_WIDTH_FONT_SIZE);
                al.add_span(
                    (bang_end - line_doc_start)..(force_end - line_doc_start),
                    &forcing,
                );
            }
            if force_end < hi {
                al.add_span(
                    (force_end - line_doc_start)..(hi - line_doc_start),
                    &dh_hidden,
                );
            }
            continue;
        }
        if substitutes::add_substitute_conceal_spans(
            al,
            line_text,
            line_doc_start,
            md_spans,
            r,
            lo,
            hi,
            &hidden,
            ck,
            substitute_advances,
        ) {
            continue;
        }
        al.add_span((lo - line_doc_start)..(hi - line_doc_start), &hidden);
    }
}
