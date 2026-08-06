use super::*;

mod font_symbols;
pub(super) use font_symbols::*;

const QUOTE_TEXT_DIM: bool = true;

fn quote_text_dim() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| QUOTE_TEXT_DIM && std::env::var_os("AWL_QUOTE_FULL_INK").is_none())
}

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
pub(super) fn md_attrs(base: &Attrs<'static>, kind: crate::markdown::MdKind) -> Attrs<'static> {
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
pub(super) fn add_md_line_spans(
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
pub(super) fn add_line_spans<K: Copy>(
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

pub(super) const RULE_CONCEAL_COLOR: glyphon::Color = glyphon::Color::rgba(0, 0, 0, 0);

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
pub(super) const IMAGE_MISSING_ROW_LINES: f32 = 3.0;

pub(super) const IMAGE_MAX_VIEWPORT_FRAC: f32 = 0.65;

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn image_display_size(
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

pub(super) fn add_rule_conceal_span(
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

pub(super) fn add_bullet_conceal_span(
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

pub(super) fn add_list_indent_span(
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
        return; // the PLAIN tier: byte-identical to the pre-item-15 renderer
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
pub(super) fn selection_touch_bytes(
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
/// own `selected` decision, and — SELECTION REVEAL regression fix, item 16
/// follow-up — [`super::TextPipeline::compute_image_layout`]'s `revealed_now`
/// and [`super::TextPipeline::images_report`]'s `revealed` reuse the SAME
/// call rather than re-deriving the overlap arithmetic a second (or third)
/// time, so an inline-image line's LAYOUT/DRAW reveal state can never
/// disagree with its markdown-markup reveal state.
pub(super) fn selection_touches(
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
        | ConcealKind::Blockquote => !conceal_off_cursor || selected,
    }
}

pub(super) fn line_has_image_span(
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

pub(super) fn image_line_has_other_content(
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
pub(super) fn line_has_rule_span(
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    line_doc_start: usize,
    line_end: usize,
) -> bool {
    use crate::markdown::MdKind;
    md_spans
        .iter()
        .any(|(cr, ck)| *ck == MdKind::Rule && cr.start < line_end && cr.end > line_doc_start)
}

pub(super) fn line_has_code_span(
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
/// `image_force` (item 5 rework, `Some((dh, target_advance_px))` only for a
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
pub(super) fn add_wysiwyg_conceal_spans(
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
        al.add_span((lo - line_doc_start)..(hi - line_doc_start), &hidden);
    }
}

/// Build the per-cell `AttrsList` for a GFM table GRID cell (the tables-v1 styled,
/// off-cursor render — `layers::prepare_table_grid`). A cell is a SMALL INLINE
/// markdown context: it may carry `**bold**` / `*italic*` / `` `code` `` /
/// `==highlight==`, but no block construct. This reuses the EXACT inline-styling
/// seam prose uses — [`crate::markdown::spans`] on the cell substring, then
/// [`add_md_line_spans`] (content styling: real bundled Bold weight, italic,
/// mono+tint inline code) followed by [`add_wysiwyg_conceal_spans`] (the emphasis
/// / code / highlight DELIMITERS collapse to true zero-width) — so a cell styles
/// identically to the same run in body prose, with the raw markers gone from both
/// the pixels AND the shaped WIDTH (the concealed advance is sub-pixel, so a
/// caller measuring `run.line_w` after shaping sizes the column to the styled
/// content, not the raw source). `line_height` is the cell row's own height,
/// threaded into the zero-width conceal so the concealed markers never shrink the
/// row (the same contract [`build_line_attrs`] honors). A grid cell is ALWAYS the
/// off-cursor styled form — the caret's OWN table parks the grid and reveals raw
/// source a level up (`prepare_table_grid`), so `conceal_off_cursor = true` and
/// `cursor_byte` is irrelevant (a cell has no fenced block). A cell with NO inline
/// markup yields an empty span set → the returned list is `base` alone, so a
/// plain cell shapes BYTE-IDENTICALLY to the pre-styling `set_text(cell, base)`.
/// Gated implicitly on `wysiwyg_on()` (the caller only builds a grid when it is
/// on; `add_wysiwyg_conceal_spans` also self-gates), so nothing conceals off.
pub(super) fn cell_inline_attrs(
    base: &Attrs<'static>,
    line_height: f32,
    cell: &str,
) -> glyphon::cosmic_text::AttrsList {
    let md_spans = crate::markdown::spans(cell);
    let mut al = glyphon::cosmic_text::AttrsList::new(base);
    add_md_line_spans(&mut al, cell, 0, base, &md_spans);
    add_wysiwyg_conceal_spans(
        &mut al,
        cell,
        0,
        base,
        &md_spans,
        true,
        0,
        line_height,
        None,
        None,
    );
    al
}

pub(super) fn syn_attrs(base: &Attrs<'static>, kind: crate::syntax::SynKind) -> Attrs<'static> {
    base.clone()
        .color(role_style_for(&theme::active(), kind).fg.to_glyphon())
}

const HUE_STR: f32 = 140.0;
const HUE_DEF: f32 = 220.0;
const HUE_CONST: f32 = 290.0;
const HUE_COMMENT_WASH: f32 = 50.0;

const S_FG_DARK: f32 = 0.46;
const S_FG_LIGHT: f32 = 0.18;

const T_DARK: [f32; 3] = [0.26, 0.28, 0.44];
const T_LIGHT: [f32; 3] = [0.76, 0.78, 0.80];

const WASH_S_DARK: f32 = 0.62;
const WASH_L_DARK: f32 = 0.66;
const WASH_ALPHA_DARK: u8 = 0x2A;
const WASH_S_LIGHT: f32 = 0.55;
const WASH_L_LIGHT: f32 = 0.50;
const WASH_ALPHA_LIGHT: u8 = 0x2E;

const HIGHLIGHT_HUE_OFFSET_FROM_PRIMARY: f32 = 165.0;
const HIGHLIGHT_S_DARK: f32 = 0.58;
const HIGHLIGHT_L_DARK: f32 = 0.64;
const HIGHLIGHT_ALPHA_DARK: u8 = 0x3A;
const HIGHLIGHT_S_LIGHT: f32 = 0.50;
const HIGHLIGHT_L_LIGHT: f32 = 0.58;
const HIGHLIGHT_ALPHA_LIGHT: u8 = 0x4D;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RoleStyle {
    pub fg: theme::Srgb,
    pub wash: Option<theme::Srgb>,
}

pub(super) fn role_style_for(th: &theme::Theme, kind: crate::syntax::SynKind) -> RoleStyle {
    use crate::syntax::SynKind;
    let ov = th.role_overrides;
    let (_, _, l_full) = th.base_content.to_hsl();
    let (_, _, l_dim) = th.muted.to_hsl();
    let (t, s_fg) = if th.dark {
        (T_DARK, S_FG_DARK)
    } else {
        (T_LIGHT, S_FG_LIGHT)
    };
    let fg_at =
        |anchor: f32, ti: f32| theme::Srgb::from_hsl(anchor, s_fg, l_full + (l_dim - l_full) * ti);
    let derived_wash = |anchor: f32| {
        if th.dark {
            let c = theme::Srgb::from_hsl(anchor, WASH_S_DARK, WASH_L_DARK);
            theme::Srgb::rgba(c.r, c.g, c.b, WASH_ALPHA_DARK)
        } else {
            let c = theme::Srgb::from_hsl(HUE_COMMENT_WASH, WASH_S_LIGHT, WASH_L_LIGHT);
            theme::Srgb::rgba(c.r, c.g, c.b, WASH_ALPHA_LIGHT)
        }
    };
    let with_override = |derived: Option<theme::Srgb>, ov: theme::WashOverride| match ov {
        theme::WashOverride::Default => derived,
        theme::WashOverride::Off => None,
        theme::WashOverride::Pin(c) => Some(c),
    };
    match kind {
        // PROSE comments are PROMINENT (decision: comments are the prose in the
        // code): FULL content ink + the warm wash carrying the comment identity.
        SynKind::Comment => RoleStyle {
            fg: th.base_content,
            wash: with_override(Some(derived_wash(HUE_COMMENT_WASH)), ov.comment_wash),
        },
        SynKind::CommentCode => RoleStyle {
            fg: th.muted,
            wash: None,
        },
        SynKind::Definition => RoleStyle {
            fg: ov.def_fg.unwrap_or_else(|| fg_at(HUE_DEF, t[0])),
            wash: None,
        },
        SynKind::Constant => RoleStyle {
            fg: ov.const_fg.unwrap_or_else(|| fg_at(HUE_CONST, t[1])),
            wash: None,
        },
        // Strings: green fg tint everywhere; the green wash only on DARK worlds
        // (light worlds carry string identity in the fg tint alone).
        SynKind::Str => RoleStyle {
            fg: ov.str_fg.unwrap_or_else(|| fg_at(HUE_STR, t[2])),
            wash: with_override(
                if th.dark {
                    Some(derived_wash(HUE_STR))
                } else {
                    None
                },
                ov.str_wash,
            ),
        },
    }
}

pub(super) fn wash_rgba_bytes(kind: crate::syntax::SynKind) -> [u8; 4] {
    role_style_for(&theme::active(), kind)
        .wash
        .unwrap_or(theme::Srgb::rgba(0, 0, 0, 0))
        .rgba_bytes()
}

/// The DEDICATED markdown `==highlight==` wash quad color for a world — its hue
/// DERIVED from the world's OWN accent (`hue(primary) +
/// HIGHLIGHT_HUE_OFFSET_FROM_PRIMARY`, a split-complementary), with the presence
/// (saturation / lightness / alpha) split per light/dark class. Decoupled from
/// the warm comment wash so a highlighter POPS while comments stay a subtle prose
/// whisper, but — unlike the retired fixed violet — now reads as NATIVE to each
/// world (see the `HIGHLIGHT_HUE_OFFSET_FROM_PRIMARY` doc above for the "why" and
/// the sweep that picked 165°). A PURE function of the passed theme (its
/// `primary` hue + `dark` flag), so the law test can sweep every world lock-free.
/// Every world carries it (no override hatch in v1 — unlike the syntax washes, a
/// highlight is never opted out).
///
/// **MONOCHROME WORLDS (`Theme::is_monochrome`, THEMES.md's logged DESIGN.md
/// §3 "no warm thing" amendment):** an achromatic `primary` has NO hue to
/// rotate — `hue(primary)` is a meaningless `0.0` for a plain grey (see
/// `Srgb::to_hsl`'s achromatic case), so deriving a highlight hue from it would
/// silently produce a color the world otherwise renders none of. Forced to
/// saturation `0.0` instead: the highlight becomes a pure VALUE-STEP wash — the
/// same "no hue, only lightness" idiom the WYSIWYG panel/pill already use — at
/// the SAME per-mode `l`/`alpha` every other world's highlight uses, so it still
/// pops exactly as loud, just without a hue to pop WITH.
///
/// **TRUE 1-BIT WORLDS (`Theme::is_monochrome` is the general case;
/// `Theme::is_one_bit` — Wagtail's 2026-07 rework — is the stricter one):**
/// the monochrome branch above still leaves a MID-LIGHTNESS grey wash
/// (`HIGHLIGHT_L_DARK`/`_LIGHT` sit well short of 0.0/1.0), which is exactly
/// the kind of authored grey a 1-bit world forbids outright.
///
/// **THE DITHER ROUND (supersedes the old "fully OFF" answer):** a 1-bit
/// world no longer drops the highlight wash to `alpha = 0` — it routes
/// through **THE ONE WAGTAIL HIGHLIGHT TEXTURE** instead (the user's razor:
/// one kind of emphasis, one texture — see THEMES.md's 1-bit section), a
/// deterministic Bayer-ordered dither stipple (`shaders/selection.wgsl`'s
/// `fs_main` dither branch, density `render::dither::
/// WAGTAIL_HIGHLIGHT_DITHER_DENSITY`) that is EVERY pixel either pure quad
/// color at full opacity or fully transparent — never a fractional alpha, so
/// it never composites a forbidden grey the way the old flat-alpha wash
/// would have. This function's job for a one-bit world simplifies to naming
/// the dither's ONE color: pure opaque white (the token
/// [`highlight_wash_rgba_bytes`] feeds the pipeline; the DENSITY that turns
/// dither mode on is a separate call, [`wagtail_dither_density`], applied at
/// the same construction/re-tint call sites). `==highlight==` still reads
/// structurally either way (the `==` delimiters still conceal/reveal, the
/// marked text still keeps full ink) — now it ALSO carries the dither band,
/// exactly like search matches do on a one-bit world (see
/// `wagtail_dither_density`'s doc for the "one texture, two consumers" wiring).
pub(super) fn highlight_wash(th: &theme::Theme) -> theme::Srgb {
    if let theme::HighlightTexture::Stipple { color, .. } = th.render_caps.highlight_texture {
        return theme::Srgb::rgba(color.r, color.g, color.b, 0xFF);
    }
    let (s, l, alpha) = if th.dark {
        (HIGHLIGHT_S_DARK, HIGHLIGHT_L_DARK, HIGHLIGHT_ALPHA_DARK)
    } else {
        (HIGHLIGHT_S_LIGHT, HIGHLIGHT_L_LIGHT, HIGHLIGHT_ALPHA_LIGHT)
    };
    let s = if th.is_monochrome() { 0.0 } else { s };
    let (primary_hue, _, _) = th.primary.to_hsl();
    let hue = (primary_hue + HIGHLIGHT_HUE_OFFSET_FROM_PRIMARY).rem_euclid(360.0);
    let c = theme::Srgb::from_hsl(hue, s, l);
    theme::Srgb::rgba(c.r, c.g, c.b, alpha)
}

pub(super) fn highlight_wash_rgba_bytes() -> [u8; 4] {
    highlight_wash(&theme::active()).rgba_bytes()
}

/// THE ONE WAGTAIL HIGHLIGHT TEXTURE's density switch — `0.0` (dither mode
/// OFF, every non-one-bit world) or [`dither::WAGTAIL_HIGHLIGHT_DITHER_DENSITY`]
/// (one-bit worlds). Fed into `SelectionPipeline::set_dither` at the SAME two
/// call sites [`highlight_wash_rgba_bytes`] feeds `set_color` — construction
/// AND every `sync_theme_colors` re-tint (a switch AWAY from a one-bit world
/// must reset this back to `0.0`, never merely leave it stale). The two
/// consumers this drives — `wash_highlight_pipeline` (`==highlight==` spans)
/// and `match_pipeline` (search matches) — deliberately share this ONE
/// function + density: the razor is ONE texture for ONE meaning ("something
/// here is marked"), not a per-consumer ladder.
pub(super) fn wagtail_dither_density() -> f32 {
    match theme::active().render_caps.highlight_texture {
        theme::HighlightTexture::Stipple { density, .. } => density,
        theme::HighlightTexture::Wash => 0.0,
    }
}

pub(super) fn wagtail_stipple_cell_px(dpi: f32) -> f32 {
    if wagtail_dither_density() <= 0.0 {
        return 1.0;
    }
    let logical = stipple_cell_logical_override()
        .unwrap_or(crate::render::dither::WAGTAIL_HIGHLIGHT_STIPPLE_CELL_LOGICAL);
    (logical * dpi.max(1.0)).round().max(1.0)
}

fn stipple_cell_logical_override() -> Option<f32> {
    static ONCE: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    *ONCE.get_or_init(|| {
        std::env::var("AWL_STIPPLE_CELL")
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|c| *c >= 1.0 && c.is_finite())
    })
}

/// The ACTIVE world's SEARCH-MATCH quad rgba — `theme::selection_document()` on every
/// ordinary world (unchanged), but on a one-bit world this NO LONGER shares
/// the (now true-inverse-video) document-selection token: it instead reads
/// pure opaque white, the SAME single color [`highlight_wash_rgba_bytes`]
/// feeds the dither pipeline, since a one-bit search match renders through
/// THE ONE WAGTAIL HIGHLIGHT TEXTURE too (paired with [`wagtail_dither_density`]
/// on `match_pipeline`) rather than the old solid-white/punch-outline
/// mechanism document selection used to share with it.
pub(super) fn search_match_rgba_bytes() -> [u8; 4] {
    match theme::active().render_caps.highlight_texture {
        theme::HighlightTexture::Stipple { color, .. } => {
            theme::Srgb::rgba(color.r, color.g, color.b, 0xFF).rgba_bytes()
        }
        theme::HighlightTexture::Wash => theme::selection_document().rgba_bytes(),
    }
}

pub(in crate::render) const STRIKE_THICKNESS: f32 = 1.3;

pub(in crate::render) const STRIKE_V_FRAC: f32 = 0.5;

fn line_band(top: f32, height: f32, zoom: f32, v_frac: f32, thickness: f32) -> (f32, f32, f32) {
    let stroke = thickness * zoom;
    let band_h = stroke + 2.0;
    let center = top + height * v_frac;
    (center - band_h * 0.5, band_h, stroke)
}

pub(in crate::render) fn strike_line_band(top: f32, height: f32, zoom: f32) -> (f32, f32, f32) {
    line_band(top, height, zoom, STRIKE_V_FRAC, STRIKE_THICKNESS)
}

pub(in crate::render) const LINK_UNDERLINE_THICKNESS: f32 = STRIKE_THICKNESS;

pub(in crate::render) const LINK_UNDERLINE_V_FRAC: f32 = 0.92;

pub(in crate::render) fn link_underline_band(top: f32, height: f32, zoom: f32) -> (f32, f32, f32) {
    line_band(
        top,
        height,
        zoom,
        LINK_UNDERLINE_V_FRAC,
        LINK_UNDERLINE_THICKNESS,
    )
}

pub(in crate::render) fn strike_ink(th: &theme::Theme) -> theme::Srgb {
    th.muted
}

pub(in crate::render) fn strike_srgba_bytes() -> [u8; 4] {
    strike_ink(&theme::active()).rgba_bytes()
}

pub(in crate::render) fn link_underline_ink(th: &theme::Theme) -> theme::Srgb {
    strike_ink(th)
}

pub(in crate::render) fn link_underline_srgba_bytes() -> [u8; 4] {
    link_underline_ink(&theme::active()).rgba_bytes()
}

pub(super) fn add_syn_line_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    base: &Attrs<'static>,
    syn_spans: &[(std::ops::Range<usize>, crate::syntax::SynKind)],
) {
    add_line_spans(al, line_text, line_doc_start, base, syn_spans, syn_attrs);
}

/// The font / line-height SCALE for ONE buffer line, driven by its LEADING `#`
/// run: `# ` → h1, `## ` → h2, `###`+ → h3 (see [`crate::markdown::heading_scale`]).
/// Keyed off the raw hash COUNT, NOT a fully-valid ATX heading, so a line grows the
/// instant you type `#` — before the space and title (and even for `#foo`). Only
/// the LEADING run counts (after optional indent), so a `#` mid-prose is ignored.
/// A THEMATIC-BREAK line (`---`/`***`/`___`, see [`crate::markdown::is_thematic_break`])
/// grows to the ACTIVE world's [`crate::theme::Theme::ornament_scale`] (per-world by
/// the ornament's character — see that field) so its row fits the bigger centered
/// break fleuron (drawn separately by `prepare_ornaments`, which reads the SAME
/// per-world scale for its glyph line-box, so the two stay in lockstep). `md` gates it:
/// a non-markdown buffer (and any plain line) returns the byte-identical `1.0`. The
/// DIM-markup + bold-weight styling still comes from the pulldown spans in
/// [`md_attrs`]; this governs SIZE alone, so an in-progress `#foo` is big but not yet
/// bold until it becomes a real heading.
///
/// `confirmed_rule` gates the thematic-break growth alone (the heading branch above
/// is untouched — still the eager raw-hash-count heuristic on purpose). Callers that
/// can afford to check the real parse (`build_line_attrs`, via [`line_has_rule_span`])
/// pass the ground truth: `is_thematic_break` is a single-line scan that cannot see
/// CONTEXT the real parse excludes on purpose — e.g. a `---` line living inside a
/// fenced code block's body reads as a break here but the real spans never rule it —
/// so without this second gate a bare-scan false positive would reserve space for an
/// ornament `prepare_ornaments`, which reads the real spans, never draws. A caller
/// with no cheap access to `md_spans` (the zoom-restyle fast path in
/// `has_heading_lines`) may pass `true` unconditionally: that's a harmless
/// over-approximation there (it only widens when a restyle runs), never the applied
/// row geometry.
pub(super) fn md_line_scale(line_text: &str, md: bool, confirmed_rule: bool) -> f32 {
    let level = md_line_heading_level(line_text, md);
    if level > 0 {
        return crate::markdown::heading_scale(level);
    }
    if md && confirmed_rule && crate::markdown::is_thematic_break(line_text) {
        return crate::theme::active().ornament_scale;
    }
    1.0
}

pub(super) fn md_line_heading_level(line_text: &str, md: bool) -> u8 {
    if !md {
        return 0;
    }
    let b = line_text.as_bytes();
    let mut i = 0;
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    let mut hashes = 0u8;
    while i < b.len() && b[i] == b'#' {
        hashes = hashes.saturating_add(1);
        i += 1;
    }
    hashes
}

/// `base` with a per-line metrics override applied (heading lines render LARGER,
/// and — theme-QA round — grow their ROW a further, DECOUPLED amount beyond
/// what their font size alone needs, the same "absolute line-height, unlinked
/// from font size" shape an inline image's row already uses). At
/// `font_scale == row_scale == 1.0` this returns a plain clone with NO
/// `metrics_opt`, so a non-heading line shapes byte-identically to the
/// pre-heading-size renderer. Otherwise it sets
/// `Attrs::metrics(base_font * font_scale, base_line * row_scale)`; cosmic-text
/// derives a row's height from the MAX of its glyphs' per-span line heights
/// (`shape.rs`), so applying this to the line's default attrs AND to every span
/// built from it makes the whole heading row taller (by `row_scale`) and its
/// glyphs bigger (by `font_scale`) — INDEPENDENTLY, so a heading's SIZE ladder
/// (Ladder J) and its row's SPACING lead (`crate::markdown::heading_row_lead`)
/// can each be tuned without moving the other. `font_scale == row_scale` for
/// every pre-existing caller shape (plain body lines and thematic breaks both
/// still pass the SAME value twice), so this is a pure signature widening, not
/// a behavior change for either. The values are ABSOLUTE pixels (already
/// zoom/DPI-folded), so any zoom/DPI change must rebuild these (see
/// [`TextPipeline::restyle_all_lines`]).
pub(super) fn scaled_base_attrs(
    base: &Attrs<'static>,
    base_font_size: f32,
    base_line_height: f32,
    font_scale: f32,
    row_scale: f32,
) -> Attrs<'static> {
    if (font_scale - 1.0).abs() < 1e-3 && (row_scale - 1.0).abs() < 1e-3 {
        return base.clone();
    }
    base.clone().metrics(GlyphMetrics::new(
        base_font_size * font_scale,
        base_line_height * row_scale,
    ))
}

/// Assemble ONE buffer line's complete `AttrsList` from the base doc attrs plus
/// every styling layer, in the canonical order: heading SIZE scale
/// ([`scaled_base_attrs`]) → markdown spans → syntax spans → CJK family spans →
/// SYMBOL family spans → (optional) RULE + BULLET concealment (symbol family wins on
/// symbol runs, CJK family on CJK runs; markdown/syntax weight/color/style win
/// elsewhere; the concealed markup's transparent ink wins LAST over its own glyphs).
/// `line_doc_start` is the line's first document byte (so the whole-document span
/// lists map into this line's local range). `conceal_off_cursor` is the reveal-on-
/// cursor gate: when set (the caret is on a DIFFERENT line) a markdown horizontal-rule
/// line's literal `---` are hidden via [`add_rule_conceal_span`] (leaving the centered
/// fleuron) AND a bullet's raw `-`/`*`/`+` via [`add_bullet_conceal_span`] (leaving its
/// depth glyph); when clear (the caret is on the line) the raw markup stays dim +
/// editable and no ornament is drawn. `cursor_byte` (the caret line's first document
/// byte) additionally drives the WYSIWYG conceal ([`add_wysiwyg_conceal_spans`]) for
/// its one BLOCK-scoped kind (a fenced code block's marker lines). `selection_touch`
/// ([`selection_touch_bytes`], `None` with no active selection) extends the SAME
/// reveal rule: a line the selection touches drops BOTH the rule/bullet conceal gate
/// here (`conceal_off_cursor && !line_selected`) AND (threaded straight through) every
/// `add_wysiwyg_conceal_spans` kind, so a selected line shows raw markdown exactly
/// like the caret's own line does. This is the SINGLE recipe shared by
/// [`TextPipeline::set_text_incremental`], [`TextPipeline::restyle_all_lines`], and
/// [`TextPipeline::refresh_rule_conceal`], so the three paths can never drift on layer
/// ordering, membership, or the caret-vs-selection reveal decision.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_line_attrs(
    base: &Attrs<'static>,
    base_font_size: f32,
    base_line_height: f32,
    md: bool,
    line_text: &str,
    line_doc_start: usize,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    syn_spans: &[(std::ops::Range<usize>, crate::syntax::SynKind)],
    doc_lang: Option<crate::frontmatter::Lang>,
    cjk_priority: &[crate::frontmatter::Lang],
    fonts: &super::text::ScriptFonts,
    conceal_off_cursor: bool,
    cursor_byte: usize,
    image_row_height: Option<f32>,
    image_force: Option<(f32, f32)>,
    selection_touch: Option<&std::ops::Range<usize>>,
) -> glyphon::cosmic_text::AttrsList {
    // A BARE image line (`- ![alt](p)`, no other content) or a wrapped-table row
    // reserves a TALL row at its display height — NORMAL font size (so the
    // revealed `![alt](path)` source stays readable when the caret lands) over a
    // tall LINE-HEIGHT (the row cosmic-text derives from the row's max glyph
    // line-height). This is the "per-line metric override" the headings use,
    // but with an ABSOLUTE line-height rather than a font-size scale, so it stays
    // decoupled from the font size. `row_lh` also feeds the zero-width conceal
    // below so the off-cursor (fully concealed) source keeps the row tall.
    //
    // A MIXED image line (`- caption text ![alt](p)`) does NOT use this at all —
    // `image_row_height` is always `None` there; instead `image_force` (below)
    // drives a forced-trailing-ROW mechanism on the SAME (untouched) line so the
    // caption's own row never inflates (see `add_wysiwyg_conceal_spans`'s doc).
    //
    // CAPTION-STYLE REVEAL (re-decided 2026-07-09, supersedes the reveal-GROW
    // model): a BARE image row is ALWAYS exactly the image height `h` — the caret
    // landing on / leaving the line causes ZERO row-height change and ZERO reflow
    // (the headline win). Off the caret's line the source CONCEALS (zero-width) and
    // the image fills the row. ON the caret's line the source REVEALS at body size
    // and cosmic-text centres it VERTICALLY within the same `h`-tall row — i.e.
    // OVER the still-drawn, DIMMED image (a deliberate caption; a scrim band behind
    // the text lifts legibility — see `layers::prepare_images`). Growing the row to
    // stack source-above-image is geometrically impossible with one layout line per
    // row (cosmic-text gives each line ONE vertically-centred baseline), so we lean
    // into the centering rather than fight it.
    let confirmed_rule =
        md && line_has_rule_span(md_spans, line_doc_start, line_doc_start + line_text.len());
    let scale = md_line_scale(line_text, md, confirmed_rule);
    // ROW-HEIGHT LEAD (theme-QA round): a heading's row grows a further,
    // DECOUPLED amount beyond `scale` alone gives its font — vertical
    // breathing room, a second axis (besides size/weight) for the hierarchy
    // to read on. `heading_row_lead` is `1.0` for every non-heading line
    // (body text AND thematic breaks alike — `md_line_heading_level` is `0`
    // for both), so `row_scale == scale` there, unchanged from before this
    // round.
    let heading_level = md_line_heading_level(line_text, md);
    let row_scale = scale * crate::markdown::heading_row_lead(heading_level);
    let (lb, row_lh) = match image_row_height {
        Some(h) => (
            base.clone().metrics(GlyphMetrics::new(base_font_size, h)),
            h,
        ),
        None => (
            scaled_base_attrs(base, base_font_size, base_line_height, scale, row_scale),
            base_line_height * row_scale,
        ),
    };
    let mut al = glyphon::cosmic_text::AttrsList::new(&lb);
    add_md_line_spans(&mut al, line_text, line_doc_start, &lb, md_spans);
    add_syn_line_spans(&mut al, line_text, line_doc_start, &lb, syn_spans);
    add_script_spans(&mut al, line_text, &lb, doc_lang, cjk_priority, fonts);
    add_symbol_spans(&mut al, line_text, &lb);
    // SELECTION REVEAL: does the active selection touch THIS line? Same
    // overlap test `wysiwyg_reveals` uses for a concealable span's own byte
    // range, applied here to the whole line's range so the LEGACY (pre-
    // `ConcealKind`) rule/bullet conceal widens identically — a selected
    // bulleted list reveals its raw `-`/`*`/`+` exactly like a selected
    // heading reveals its raw `#`, never a mixed state on the same line.
    let line_end = line_doc_start + line_text.len();
    let line_selected =
        selection_touch.is_some_and(|st| st.start < line_end && line_doc_start < st.end);
    // REVEAL-ON-CURSOR: when the caret is off this line AND the selection
    // doesn't touch it, conceal a thematic break's raw `---` (leaving the
    // fleuron) AND a bullet's raw `-` (leaving the depth glyph). Both are
    // drawn as ornaments on the SAME rows; on the caret's own line — or any
    // selected line — the raw markup reveals for editing and no ornament is
    // drawn.
    if conceal_off_cursor && !line_selected {
        add_rule_conceal_span(&mut al, line_text, line_doc_start, &lb, md_spans);
        add_bullet_conceal_span(&mut al, line_text, &lb);
    }
    add_wysiwyg_conceal_spans(
        &mut al,
        line_text,
        line_doc_start,
        &lb,
        md_spans,
        conceal_off_cursor,
        cursor_byte,
        row_lh,
        image_force,
        selection_touch,
    );
    add_list_indent_span(&mut al, line_text, &lb, base_font_size, row_lh);
    al
}
