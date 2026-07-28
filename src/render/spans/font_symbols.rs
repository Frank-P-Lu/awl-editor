//! Script and symbol font-span helpers.

use super::*;

pub(crate) fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x3000..=0x303F   // CJK symbols & punctuation (、。「」…)
        | 0x3040..=0x309F // Hiragana
        | 0x30A0..=0x30FF // Katakana
        | 0x31F0..=0x31FF // Katakana phonetic extensions
        | 0x3400..=0x4DBF // CJK Unified Ideographs Extension A
        | 0x4E00..=0x9FFF // CJK Unified Ideographs
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
        | 0xFF00..=0xFFEF // Halfwidth & Fullwidth Forms
    )
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
/// generalizing what used to be a single ja-only CJK family span into an
/// independent [`theme::FontId`] resolution per run. Walks
/// [`crate::script::script_runs`] (kana / hangul / bopomofo / han, each
/// named) and resolves EACH run's [`theme::FontId`] via
/// [`crate::script::resolve_font_id`]'s ladder — (a) the document's own
/// frontmatter `lang:` tag, if compatible with the run's script; (b) else the
/// script's own unambiguous mapping; (c) else (a Han run with no compatible
/// tag) the `cjk_priority` tiebreak; (d) else no override at all (a
/// `FontId::Latin` result, or a script whose ladder resolved to nothing on
/// this machine — `fonts.get` returns `None` either way, so the base doc face
/// wins — the same degenerate fallback the old single-script version had).
/// `fonts` is [`super::text::ScriptFonts`], resolved ONCE per reshape by
/// [`TextPipeline::resolve_script_fonts`] — this function does no font-DB
/// work itself, just the per-run ladder + span laying.
///
/// WEIGHT + STYLE PIN (bold/italic-breaks-Japanese fix): each per-script span
/// PINS the run's weight AND style to the resolved face's REGISTERED values —
/// `.weight(wt)` (the concrete weight nearest 400 the font DB has for that
/// family) and `.style(Normal)`. Every bundled CJK face
/// ([`crate::render::FONT_CJK_FACES`] / [`FONT_ZH_KO_FACES`]) registers ONLY at
/// Regular/400/Normal — there is no bold or italic CJK cut in v1 — so pinning is
/// exactly "the resolved face's registered values", never a guess. This layer
/// runs LAST over the markdown layer in [`build_line_attrs`] (script spans UNDER
/// nothing that re-weights a CJK run), and `AttrsList::add_span` REPLACES the
/// whole run range, so a `**bold**` (Weight 700) / `*italic*` (Style::Italic)
/// markdown span sitting under a CJK run is overwritten on exactly those bytes:
/// Japanese inside emphasis keeps its correct per-world face instead of dropping
/// it (cosmic-text's fallback keeps only `weight_diff == 0` + style-matching
/// faces — a 700/italic request would drop the 400/Normal bundled JP face and
/// tofu/system-fall mid-sentence). The pin derives from `base` (the plain doc
/// attrs, already Normal), so even a styled base can never leak a synthetic
/// slant/weight onto a CJK run. The emphasis still reads — via the revealed
/// `**`/`*` markers on the caret's line and the surrounding Latin styling.
///
/// LOGGED TASTE CALL: NO synthetic bold/italic for CJK in v1 — a CJK run in a
/// `**bold**`/`*italic*` span renders at the bundled face's own Regular weight,
/// upright, rather than letting glyphon synthesize an oblique or drop to a
/// heavier fallback. A future real JP/zh/ko bold-or-italic bundled face would
/// lift this clamp (resolve the emphasis to that cut instead of pinning Normal).
pub(crate) fn add_script_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    text: &str,
    base: &Attrs,
    doc_lang: Option<crate::frontmatter::Lang>,
    cjk_priority: &[crate::frontmatter::Lang],
    fonts: &super::text::ScriptFonts,
) {
    for (run, script) in crate::script::script_runs(text) {
        let id = crate::script::resolve_font_id(doc_lang, Some(script), cjk_priority);
        let Some((fam, wt)) = fonts.get(id) else {
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

/// True for the SYMBOL / ORNAMENT codepoints the bundled mono + proportional
/// display faces lack — the macOS modifier glyphs (⌘ ⇧ ⌥ ⌃), the key-hint keycaps
/// (↵ Return, ⇥ Tab), the fine-press ornaments / fleurons (❧ ❦ ☙ ❡ ❥), the
/// asterism (⁂), and the reference marks (§ † ‡). These render as TOFU under the
/// global fallback (IBM Plex Mono Light), so the renderer overlays the bundled
/// [`SYMBOL_FAMILY`] face on their runs (see [`add_symbol_spans`]). Exactly the
/// glyph set bundled in `AwlSymbols.ttf`; keep the two in sync.
pub(crate) fn is_symbol(c: char) -> bool {
    matches!(
        c as u32,
        0x2318   // ⌘ Command
        | 0x21E7 // ⇧ Shift
        | 0x2325 // ⌥ Option
        | 0x2303 // ⌃ Control
        | 0x21B5 // ↵ Downwards arrow with corner leftwards (Return / Enter)
        | 0x21E5 // ⇥ Rightwards arrow to bar (Tab)
        | 0x2767 // ❧ Rotated floral heart (fleuron — the hr ornament)
        | 0x2766 // ❦ Floral heart (the `___` break ornament)
        | 0x2619 // ☙ Reversed rotated floral heart (fleuron variant)
        | 0x2761 // ❡ Curved stem paragraph sign ornament
        | 0x2765 // ❥ Rotated heavy black heart bullet (fleuron variant)
        | 0x2042 // ⁂ Asterism
        | 0x00A7 // § Section sign
        | 0x2020 // † Dagger
        | 0x2021 // ‡ Double dagger
    )
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
