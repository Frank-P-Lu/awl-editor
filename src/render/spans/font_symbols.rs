//! Script and symbol font-span helpers.

use super::*;

pub(crate) fn is_cjk(c: char) -> bool {
    crate::script::classify_char(c).is_some()
}

pub(crate) fn cjk_runs(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut runs = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in text.char_indices() {
        if is_cjk(c) {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            runs.push(s..i);
        }
    }
    if let Some(s) = start.take() {
        runs.push(s..text.len());
    }
    runs
}

/// i18n: lay PER-SCRIPT family spans over `al` for every CJK-family run in
/// `text` — the render wiring for `crate::script`'s classifier + ladder,
/// resolving an independent [`theme::FontId`] per run rather than one
/// CJK-wide family span. Walks
/// [`crate::script::script_runs`] (kana / hangul / bopomofo / han / CJK-common,
/// each named) and resolves EACH run's [`theme::FontId`] via
/// [`crate::script::resolve_font_id`]'s ladder — (a) the document's own
/// frontmatter `lang:` tag, if compatible with the run's script; (b) else the
/// script's own unambiguous mapping; (c) else (a Han/Common run with no
/// compatible tag) the `cjk_priority` tiebreak; (d) else no override at all (a
/// `FontId::Latin` result, or a script whose ladder resolved to nothing on
/// this machine — `fonts.get` returns `None` either way, so the base doc face
/// wins — the same degenerate fallback the old single-script version had).
/// `cjk_priority` here is already the CALLER's effective ladder
/// ([`crate::script::effective_cjk_priority`]) — the document's own Han
/// evidence promoted to the front when decisive — so this function's own
/// ladder never changed shape. CJK-common punctuation first inherits immediate
/// strong-script context. With none, an authored/detected document language
/// owns it; a Latin-only document leaves it in the base display face.
/// `fonts` is [`super::text::ScriptFonts`], resolved ONCE per reshape by
/// [`TextPipeline::resolve_script_fonts`] — this function does no font-DB
/// work itself, just the per-run ladder + span laying.
///
/// WEIGHT + STYLE RESOLUTION: each per-script span pins upright style and a REAL
/// registered face. Japanese has a bundled heavy companion for every family, so
/// a Markdown/heading request of weight >= 600 selects that companion (including
/// Fontworks' authentic Klee One SemiBold at 600). Other scripts, and ordinary
/// Japanese, keep the resolved Regular weight. This layer
/// runs LAST over the markdown layer in [`build_line_attrs`] (script spans UNDER
/// nothing that re-weights a CJK run), and `AttrsList::add_span` REPLACES the
/// whole run range, so a `**bold**` (Weight 700) / `*italic*` (Style::Italic)
/// markdown span sitting under a CJK run is overwritten on exactly those bytes:
/// Japanese inside emphasis keeps its correct per-world face instead of dropping
/// it (cosmic-text's fallback keeps only `weight_diff == 0` + style-matching
/// faces — a 700/italic request would drop the 400/Normal bundled JP face and
/// tofu/system-fall mid-sentence). The pin derives from `base` (the plain doc
/// attrs, already Normal), so even a styled base can never leak a synthetic
/// slant onto a CJK run. Japanese emphasis draws dots in the content decoration
/// layer; bold keeps its authentic companion and Latin keeps its italic face.
///
/// There is still no synthetic slant: Japanese italic remains upright, and
/// zh/ko continue to pin their real Regular faces until genuine companions are
/// bundled.
pub(crate) fn add_script_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    text: &str,
    base: &Attrs,
    doc_lang: Option<crate::frontmatter::Lang>,
    cjk_evidence: Option<crate::frontmatter::Lang>,
    cjk_priority: &[crate::frontmatter::Lang],
    fonts: &super::text::ScriptFonts,
) {
    for (run, classified) in crate::script::script_runs(text) {
        let script = crate::script::contextual_script_at(text, run.start).unwrap_or(classified);
        // An isolated CJK-common mark in Latin-only text stays in the display
        // face. A real document language (authored tag or decisive text
        // evidence) owns it; otherwise only adjacent CJK context can enrol it.
        let resolution_lang = if script == crate::script::Script::Common {
            let Some(lang) = doc_lang.or(cjk_evidence) else {
                continue;
            };
            Some(lang)
        } else {
            doc_lang
        };
        let id = crate::script::resolve_font_id(resolution_lang, Some(script), cjk_priority);
        let requested_weight = al.get_span(run.start).weight;
        let resolved = if id == theme::FontId::Ja && requested_weight.0 >= 600 {
            fonts.ja_bold.or(fonts.ja)
        } else {
            fonts.get(id)
        };
        let Some((fam, wt)) = resolved else {
            continue;
        };
        let a = base
            .clone()
            .family(Family::Name(fam))
            .weight(wt)
            .style(glyphon::Style::Normal);
        al.add_span(run, &a);
    }
}

/// True for the SYMBOL / ORNAMENT codepoints that chrome routes explicitly
/// through the bundled [`SYMBOL_FAMILY`] face. Enrolment comes from the
/// `symbol-span` role in `AwlMarks.roster.tsv`, so adopting or retiring a mark
/// has one owner rather than a parallel match arm here. PUA ornament-only marks
/// deliberately carry no such role: they remain explicit awl chrome, never an
/// ambient document fallback.
pub(crate) fn is_symbol(c: char) -> bool {
    super::super::marks::has_role(c, "symbol-span")
}

pub(crate) fn symbol_runs(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut runs = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in text.char_indices() {
        if is_symbol(c) {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            runs.push(s..i);
        }
    }
    if let Some(s) = start.take() {
        runs.push(s..text.len());
    }
    runs
}

/// THE ONE OWNER of the chrome symbol-split PUSH loop. Append `text` onto `spans`
/// as alternating non-symbol / [`is_symbol`] runs (via [`symbol_runs`]): every
/// symbol run takes `sym`'s attrs (the bundled [`SYMBOL_FAMILY`] face — real,
/// finite advances for the macOS modifier glyphs ⌘ ⇧ ⌥ ⌃ and keycap ornaments
/// ↵ ⇥ …, which the display/mono faces render as tofu), every other run takes
/// `plain`'s. The overlay foot hint, the keybindings-tips footer, the inline
/// trailing shortcut, and the right-aligned chord column all shared this loop
/// verbatim (the C2 footer round's `push_overlay_hint_spans` was the first copy);
/// they now route through here so a symbol-split can never drift between them.
/// A symbol-free `text` pushes exactly ONE `plain` span — byte-identical to a bare
/// `spans.push((text, plain()))`. The attrs come from CLOSURES so each caller keeps
/// its own color / metrics without this owner knowing them. Does NOT emit any line
/// break: a caller that wants the run on its own line pushes the `"\n"` itself.
pub(crate) fn push_symbol_split<'a>(
    spans: &mut Vec<(&'a str, Attrs<'a>)>,
    text: &'a str,
    plain: impl Fn() -> Attrs<'a>,
    sym: impl Fn() -> Attrs<'a>,
) {
    let mut last = 0usize;
    for run in symbol_runs(text) {
        if run.start > last {
            spans.push((&text[last..run.start], plain()));
        }
        let end = run.end;
        spans.push((&text[run], sym()));
        last = end;
    }
    if last < text.len() {
        spans.push((&text[last..], plain()));
    }
}

pub(crate) fn add_symbol_spans(al: &mut glyphon::cosmic_text::AttrsList, text: &str, base: &Attrs) {
    let runs = symbol_runs(text);
    if runs.is_empty() {
        return;
    }
    let a = base.clone().family(Family::Name(SYMBOL_FAMILY));
    for run in runs {
        al.add_span(run, &a);
    }
}
