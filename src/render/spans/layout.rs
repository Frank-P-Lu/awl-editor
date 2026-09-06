//! Per-line size scale (heading ladder, thematic-break ornament room) and
//! the final `AttrsList` assembly, `build_line_attrs`, that layers every
//! styling pass in the canonical order.

use super::*;

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
pub(in crate::render) fn md_line_scale(line_text: &str, md: bool, confirmed_rule: bool) -> f32 {
    let level = md_line_heading_level(line_text, md);
    if level > 0 {
        return crate::markdown::heading_scale(level);
    }
    if md && confirmed_rule && crate::markdown::is_thematic_break(line_text) {
        return crate::theme::active().ornament_scale;
    }
    1.0
}

/// The factor by which one line's ROW is taller than its own glyphs ask for:
/// [`crate::markdown::heading_row_lead`] of the line's heading level, `1.0` for
/// every other line (body prose, thematic breaks, image rows, list items). THE
/// divide-out seam — [`crate::render::TextPipeline::caret_band_scale`] reads it
/// to strip the decoupled lead back off a caret-adjacent band, and it asks the
/// same [`md_line_heading_level`] that [`build_line_attrs`] asks when it
/// multiplies the lead IN, so the two can never answer about different levels.
pub(in crate::render) fn md_line_row_lead(line_text: &str, md: bool) -> f32 {
    crate::markdown::heading_row_lead(md_line_heading_level(line_text, md))
}

/// Delegates to [`crate::fold::heading_level`], the one owner — see its own
/// doc for the exact rule. Kept as a distinct name in this module because
/// every call site here reads as "the render SIZE half"; the fold half is
/// [`crate::fold::heading_level`] itself.
pub(in crate::render) fn md_line_heading_level(line_text: &str, md: bool) -> u8 {
    crate::fold::heading_level(line_text, md)
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
pub(in crate::render) fn scaled_base_attrs(
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

/// The document-wide inputs for [`build_line_attrs`]. Each text update path builds
/// this once, so adding another shared input requires every path to supply it.
pub(in crate::render) struct LineAttrsCtx<'a> {
    pub(in crate::render) base: &'a Attrs<'static>,
    pub(in crate::render) base_font_size: f32,
    pub(in crate::render) base_line_height: f32,
    pub(in crate::render) md: bool,
    pub(in crate::render) md_spans: &'a [(std::ops::Range<usize>, crate::markdown::MdKind)],
    pub(in crate::render) syn_spans: &'a [(std::ops::Range<usize>, crate::syntax::SynKind)],
    pub(in crate::render) doc_lang: Option<crate::frontmatter::Lang>,
    pub(in crate::render) cjk_priority: &'a [crate::frontmatter::Lang],
    pub(in crate::render) fonts: &'a super::text::ScriptFonts,
    pub(in crate::render) cursor_byte: usize,
    pub(in crate::render) selection_touch: Option<&'a std::ops::Range<usize>>,
    pub(in crate::render) substitute_advances: SubstituteAdvances,
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
pub(in crate::render) fn build_line_attrs(
    ctx: &LineAttrsCtx<'_>,
    line_text: &str,
    line_doc_start: usize,
    conceal_off_cursor: bool,
    image_row_height: Option<f32>,
    image_force: Option<(f32, f32)>,
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
    // REVEALED: caret on this line OR the selection touches it — the ONE
    // `wysiwyg_reveals` rule, read here BEFORE the ornament-scale decision so a
    // revealed thematic break drops the ornament's reserved room entirely
    // (raw markup at body size, row at body height) rather than keeping a
    // whole-line scale the caret-entry reveal never needed.
    // `prepare_ornaments`/`rule_lines` mirrors this exact caret-or-selection
    // test for its own draw gate (never the caret line alone), so the two
    // layers can't disagree about which state a rule line is in.
    let line_end = line_doc_start + line_text.len();
    let line_selected = ctx
        .selection_touch
        .is_some_and(|st| st.start < line_end && line_doc_start < st.end);
    let revealed = !conceal_off_cursor || line_selected;
    let confirmed_rule =
        ctx.md && !revealed && line_has_rule_span(ctx.md_spans, line_doc_start, line_end);
    let scale = md_line_scale(line_text, ctx.md, confirmed_rule);
    // ROW-HEIGHT LEAD (theme-QA round): a heading's row grows a further,
    // DECOUPLED amount beyond `scale` alone gives its font — vertical
    // breathing room, a second axis (besides size/weight) for the hierarchy
    // to read on. `heading_row_lead` is `1.0` for every non-heading line
    // (body text AND thematic breaks alike — `md_line_heading_level` is `0`
    // for both), so `row_scale == scale` there, unchanged from before this
    // round.
    let heading_level = md_line_heading_level(line_text, ctx.md);
    let row_scale = scale * crate::markdown::heading_row_lead(heading_level);
    let (lb, row_lh) = match image_row_height {
        Some(h) => (
            ctx.base
                .clone()
                .metrics(GlyphMetrics::new(ctx.base_font_size, h)),
            h,
        ),
        None => (
            scaled_base_attrs(
                ctx.base,
                ctx.base_font_size,
                ctx.base_line_height,
                scale,
                row_scale,
            ),
            ctx.base_line_height * row_scale,
        ),
    };
    let mut al = glyphon::cosmic_text::AttrsList::new(&lb);
    add_md_line_spans(&mut al, line_text, line_doc_start, &lb, ctx.md_spans);
    add_syn_line_spans(&mut al, line_text, line_doc_start, &lb, ctx.syn_spans);
    add_script_spans(
        &mut al,
        line_text,
        &lb,
        ctx.doc_lang,
        ctx.cjk_priority,
        ctx.fonts,
    );
    add_symbol_spans(&mut al, line_text, &lb);
    // SELECTION REVEAL: `line_selected` (computed above, alongside `revealed`)
    // is the same overlap test `wysiwyg_reveals` uses for a concealable span's
    // own byte range, applied here to the whole line's range so the LEGACY
    // (pre-`ConcealKind`) rule/bullet conceal widens identically — a selected
    // bulleted list reveals its raw `-`/`*`/`+` exactly like a selected
    // heading reveals its raw `#`, never a mixed state on the same line.
    // REVEAL-ON-CURSOR: when the caret is off this line AND the selection
    // doesn't touch it, conceal a thematic break's raw `---` (leaving the
    // fleuron) AND a bullet's raw `-` (leaving the depth glyph). Both are
    // drawn as ornaments on the SAME rows; on the caret's own line — or any
    // selected line — the raw markup reveals for editing and no ornament is
    // drawn.
    if conceal_off_cursor && !line_selected {
        add_rule_conceal_span(&mut al, line_text, line_doc_start, &lb, ctx.md_spans);
        add_bullet_conceal_span(&mut al, line_text, &lb);
    }
    add_wysiwyg_conceal_spans(
        &mut al,
        line_text,
        line_doc_start,
        &lb,
        ctx.md_spans,
        conceal_off_cursor,
        ctx.cursor_byte,
        row_lh,
        image_force,
        ctx.selection_touch,
        Some(ctx.substitute_advances),
    );
    add_list_indent_span(&mut al, line_text, &lb, ctx.base_font_size, row_lh);
    al
}
