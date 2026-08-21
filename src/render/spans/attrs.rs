//! Building `Attrs` from a markdown/syntax span kind, and laying spans
//! onto an `AttrsList` — the shared core both the markdown and syntax
//! styling passes ride.

use super::*;

/// Build the concrete `Attrs` for one markdown span kind, transforming `base`
/// (the doc attrs — family, ligature features, etc.):
/// - `Markup`/`ConcealMarkup`/`ListMarker`/`Rule` → recede to the DIM ink (syntax +
///   quiet text); a `Rule` row also gets a thin centered quad drawn over it. `Quote`
///   (blockquote body) dims too BY DEFAULT — a taste flag, see [`quote_text_dim`].
///   `ConcealMarkup` additionally hides off the caret's line/block — see
///   [`add_wysiwyg_conceal_spans`], applied as a later layer over this one.
/// - `Heading` → no transform; reads by SIZE alone (set per-line upstream).
/// - `Task(true)`/`TaskDone` → DIM (a completed todo recedes as one); `Task(false)`
///   (an OPEN checkbox) rides the full default ink so the box stays present.
/// - `Bold`/`Italic`/`BoldItalic` → weight / style; NO color, so they ride the
///   buffer's default ink (full when focus off, dim when focus dims the region).
/// - `Code` → the registered monospace family + a subtle tint toward MUTED ink.
/// - `LinkText` → the buffer's full CONTENT ink (it lifts off the dim `Markup`
///   span; DESIGN §3 keeps `primary`/amber for the caret alone).
pub(in crate::render) fn md_attrs(
    base: &Attrs<'static>,
    kind: crate::markdown::MdKind,
) -> Attrs<'static> {
    use crate::markdown::MdKind;
    let th = theme::active();
    let dim = th.muted.to_glyphon();
    let mut a = base.clone();
    let mut natural: Option<glyphon::Color> = None;
    match kind {
        MdKind::Markup
        | MdKind::ConcealMarkup(_)
        | MdKind::ListMarker
        | MdKind::Rule
        | MdKind::Task(true)
        | MdKind::TaskDone
        | MdKind::TablePipe
        | MdKind::TableSep => {
            natural = Some(dim);
        }
        MdKind::TableHeader => {}
        MdKind::FootnoteReference(_) | MdKind::FootnoteDefinition(_) => {
            natural = Some(dim);
        }
        MdKind::FootnoteText => {}
        MdKind::Task(false) => {}
        MdKind::Quote => {
            if quote_text_dim() {
                natural = Some(dim);
            }
        }
        MdKind::Heading(level) => {
            if crate::markdown::heading_weight_bold(th.heading_bold, level) {
                a = a.weight(glyphon::Weight::BOLD);
            }
        }
        MdKind::Bold => {
            // Resolves to the world's real bundled BOLD (700) face — for EVERY world,
            // proportional AND mono. Each display family ships a 700 companion under
            // the SAME family name as its Regular (`FONT_THEME_BOLD_FACES`), so this
            // plain `Weight::BOLD` request matches `weight_diff == 0` and lands on the
            // bold FILE. The five mono-display worlds (Tawny = IBM Plex Mono, Mangrove
            // = JetBrains Mono, Firetail/Potoroo = Monaspace Xenon, Currawong =
            // Iosevka) used to have no 700, so this request tripped the trap and fell
            // into a FOREIGN proportional sans (the "weird fi-ligature" bug); the mono
            // bolds keep the fixed grid AND give true emphasis.
            a = a.weight(glyphon::Weight::BOLD);
        }
        MdKind::Italic => {
            a = a.style(glyphon::Style::Italic);
        }
        MdKind::BoldItalic => {
            // Same as `Bold` above (real bundled 700 on every world, proportional AND
            // mono) plus glyphon's synthesized slant (no bundled italic face).
            a = a
                .weight(glyphon::Weight::BOLD)
                .style(glyphon::Style::Italic);
        }
        MdKind::Code { .. } => {
            a = a.family(Family::Monospace);
            // A subtle tint toward the MUTED ink so inline/fenced code reads as a
            // distinct surface even where mono ≈ the body face (the mono worlds).
            // Never amber — this rides the same base_content→muted ramp as the
            // Alabaster syntax roles (DESIGN §3: `primary` is the caret's alone).
            natural = Some(lerp_srgb(th.base_content, th.muted, 0.28).to_glyphon());
        }
        MdKind::CodeSyntax { role, .. } => {
            // A highlighted byte of a recognized fenced block: KEEP the mono family
            // (like `Code`) but take the syntax ROLE COLOR instead of the flat tint,
            // so the fence body reads as Alabaster-highlighted code in mono. The role
            // color comes from the SAME single derivation the code-buffer pass uses
            // ([`role_style_for`], THE role style provider), so a fence and a `.rs`
            // file highlight identically — prose comments prominent, commented-out
            // code muted, tints per world — and the syntax role WINS the flat Code
            // tint for these bytes because this span is laid AFTER the body `Code`
            // span (last-wins on overlap). The role's wash (if any) rides the wash
            // pipelines via the same md-span source (see `rects.rs::wash_rects`).
            a = a.family(Family::Monospace);
            natural = Some(role_style_for(&theme::active(), role).fg.to_glyphon());
        }
        MdKind::LinkText => {
            natural = Some(th.base_content.to_glyphon());
        }
        MdKind::Highlight => {
            // No-op transform, like `Heading`: `==marked==` text rides the buffer's
            // full default ink (it may sit OVER a dimmer context span — e.g. inside a
            // blockquote — and, like `LinkText`, is pushed AFTER that span so it lifts
            // back to full ink). The highlighter identity is carried entirely by the
            // WASH quad drawn behind it (`rects.rs::ensure_wash_protos`'s dedicated
            // `Highlight` bucket → its own per-world [`highlight_wash`] tint/pipeline,
            // a split-complementary of the world's accent, DECOUPLED from the warm
            // comment wash so it POPS), never a text color change. Never amber (DESIGN §3).
        }
        MdKind::Strikethrough => {
            natural = Some(strike_ink(&th).to_glyphon());
        }
    }
    if let Some(c) = natural {
        a = a.color(c);
    }
    a
}

/// Lay the markdown styling spans that intersect ONE buffer line over `al`. Maps
/// each document-byte span in `md_spans` into this line's local byte range
/// (`line_doc_start` is the line's first byte in the document) and adds it with
/// [`md_attrs`]. Spans are applied in their stored order so the intentional
/// link/code-block overlaps (whole-range dim, then inner content) resolve
/// correctly. No-op when `md_spans` is empty (non-markdown buffers), keeping their
/// render byte-identical.
pub(in crate::render) fn add_md_line_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
) {
    add_line_spans(al, line_text, line_doc_start, base, md_spans, md_attrs);
}

/// Shared body of [`add_md_line_spans`] / [`add_syn_line_spans`]: lay the document-
/// byte spans that intersect ONE buffer line over `al`, clamping each into the
/// line's local byte range (`line_doc_start` is the line's first byte) and adding
/// it with `attrs_fn`. Spans are applied in their stored order so intentional
/// overlaps (whole-range dim, then inner content) resolve correctly. No-op when
/// `spans` is empty, keeping non-styled buffers byte-identical.
pub(in crate::render) fn add_line_spans<K: Copy>(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    spans: &[(std::ops::Range<usize>, K)],
    attrs_fn: impl Fn(&Attrs<'static>, K) -> Attrs<'static>,
) {
    if spans.is_empty() {
        return;
    }
    let line_end = line_doc_start + line_text.len();
    for (r, kind) in spans {
        let lo = r.start.max(line_doc_start);
        let hi = r.end.min(line_end);
        if lo < hi {
            let local = (lo - line_doc_start)..(hi - line_doc_start);
            al.add_span(local, &attrs_fn(base, *kind));
        }
    }
}

pub(in crate::render) fn syn_attrs(
    base: &Attrs<'static>,
    kind: crate::syntax::SynKind,
) -> Attrs<'static> {
    base.clone()
        .color(role_style_for(&theme::active(), kind).fg.to_glyphon())
}

pub(in crate::render) fn add_syn_line_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    syn_spans: &[(std::ops::Range<usize>, crate::syntax::SynKind)],
) {
    add_line_spans(al, line_text, line_doc_start, base, syn_spans, syn_attrs);
}
