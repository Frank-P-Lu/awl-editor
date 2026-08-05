const AFF_US: &str = include_str!("../assets/dict/en_US.aff");
const DIC_US: &str = include_str!("../assets/dict/en_US.dic");
const AFF_GB: &str = include_str!("../assets/dict/en_GB.aff");
const DIC_GB: &str = include_str!("../assets/dict/en_GB.dic");
const AFF_AU: &str = include_str!("../assets/dict/en_AU.aff");
const DIC_AU: &str = include_str!("../assets/dict/en_AU.dic");

enum_with_all! {
    /// Active bundled Hunspell variant, shared by live mode and capture.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[allow(clippy::enum_variant_names)]
    pub enum DictVariant {
        EnUs,
        EnGb,
        EnAu,
    }
}

impl DictVariant {
    /// The variant a fresh install spells with. The ONE owner of that fact —
    /// `ACTIVE_VARIANT` below is initialised from it, and the generated
    /// reference reads it rather than restating `en_us`.
    pub const DEFAULT: DictVariant = DictVariant::EnUs;

    const fn as_u8(self) -> u8 {
        match self {
            DictVariant::EnUs => 0,
            DictVariant::EnGb => 1,
            DictVariant::EnAu => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DictVariant::EnUs => "English (US)",
            DictVariant::EnGb => "English (UK)",
            DictVariant::EnAu => "English (Australia)",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            DictVariant::EnUs => "Hunspell en_US — American spelling",
            DictVariant::EnGb => "Hunspell en_GB — British spelling",
            DictVariant::EnAu => "Hunspell en_AU — Australian spelling",
        }
    }

    pub fn from_label(s: &str) -> Option<DictVariant> {
        Self::ALL
            .into_iter()
            .find(|v| v.label().eq_ignore_ascii_case(s))
    }

    fn files(self) -> (&'static str, &'static str) {
        match self {
            DictVariant::EnUs => (AFF_US, DIC_US),
            DictVariant::EnGb => (AFF_GB, DIC_GB),
            DictVariant::EnAu => (AFF_AU, DIC_AU),
        }
    }
}

static ACTIVE_VARIANT: std::sync::atomic::AtomicU8 =
    std::sync::atomic::AtomicU8::new(DictVariant::DEFAULT.as_u8());

pub fn active_variant() -> DictVariant {
    match ACTIVE_VARIANT.load(std::sync::atomic::Ordering::Relaxed) {
        1 => DictVariant::EnGb,
        2 => DictVariant::EnAu,
        _ => DictVariant::EnUs,
    }
}

pub fn set_active_variant(v: DictVariant) {
    ACTIVE_VARIANT.store(v.as_u8(), std::sync::atomic::Ordering::Relaxed);
}

/// Whether spell-check is active AT ALL — the GLOBAL escape hatch (default ON):
/// a process-global read by the ONE owner seam
/// ([`SpellChecker::misspellings_for`] and [`SpellChecker::suggest_at`]) so OFF
/// silences every squiggle — prose comments and the scoped
/// code-string/comment check alike — and turns the spell-suggest picker into a
/// calm no-op, with zero duplicated gating at any call site (render, capture,
/// the right-click seam).
/// The value this flag carries on a fresh install, before any config or
/// settings write — the ONE owner of that fact, read both by the static
/// below and by the generated reference (`settings::toggle_default`).
pub(crate) const SPELLCHECK_DEFAULT: bool = true;
static SPELLCHECK_ON: crate::toggle::Toggle = crate::toggle::Toggle::new(SPELLCHECK_DEFAULT);

pub fn spellcheck_on() -> bool {
    SPELLCHECK_ON.on()
}

pub fn set_spellcheck_on(on: bool) {
    SPELLCHECK_ON.set(on);
}

pub fn toggle() -> bool {
    SPELLCHECK_ON.toggle()
}

/// A misspelled word's location in the document, in CHAR columns on a logical
/// line. `[start_col, end_col)` is a half-open char range; the renderer maps it
/// to pixels with the SAME advance-aware layout used for selection rects, so the
/// squiggle lands exactly under the word's glyphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Misspelling {
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

/// A text's logical lines, sliced ONCE — THE ONE owner of the span-to-text
/// rule every span read routes through.
///
/// Resolving a span needs the line it sits on, and finding that line by walking
/// from the document start costs O(line). The readers here are BATCH readers —
/// [`keyed`] over a whole fresh scan, [`visible`] over the whole verdict cache
/// on every `sync_view` — so a per-span walk costs O(spans x lines): on a
/// novel-length manuscript carrying an ordinary typo rate that dominates the
/// per-edit cost, and it grows quadratically with the document. Slicing once
/// makes the same batch O(lines + spans).
///
/// Cheap to build for a ONE-SHOT read too ([`word_at`]): the slice walk it
/// replaces was already O(text), and this adds only a pointer per line.
pub struct LineIndex<'a> {
    lines: Vec<&'a str>,
}

impl<'a> LineIndex<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lines: text.split('\n').collect(),
        }
    }

    /// The exact word text at `m`'s span — the same char-column extraction
    /// [`SpellChecker::suggest_at`] uses to read the word it is about to offer
    /// corrections for. `""` when the span no longer resolves (a vanished line,
    /// or `end_col <= start_col`) — never a panic on a stale span read against
    /// edited-out text.
    pub fn word_at(&self, m: &Misspelling) -> String {
        self.lines
            .get(m.line)
            .copied()
            .unwrap_or("")
            .chars()
            .skip(m.start_col)
            .take(m.end_col.saturating_sub(m.start_col))
            .collect()
    }

    /// True iff this index's text still holds `v`'s EXACT word at its span. A
    /// `false` means the text under this span changed since the verdict was
    /// computed — painting it now would show a squiggle under the WRONG word
    /// (or a stale MISSPELLED squiggle for a word that's since been fixed):
    /// the just-completed-word flash [`SpellVerdict`] exists to make
    /// structurally impossible. THE ONE check every consumer routes through.
    pub fn still_valid(&self, v: &SpellVerdict) -> bool {
        self.word_at(&v.span) == v.word
    }
}

/// One-shot [`LineIndex::word_at`] for a caller holding a SINGLE span. Every
/// production reader resolves spans in batches and builds the index once, so
/// this convenience exists for tests alone.
#[cfg(test)]
pub fn word_at(text: &str, m: &Misspelling) -> String {
    LineIndex::new(text).word_at(m)
}

/// A spell verdict KEYED to the exact word text it judged (the COMPLETED-WORD-
/// LAG fix's "keyed" half): [`Misspelling`] alone says WHERE a word was
/// flagged; this additionally freezes WHAT text was there at judgment time, so
/// a caller holding a verdict across an edit can tell "still genuinely
/// misspelled" apart from "this span's text has since changed underneath it"
/// (an edit can shift/alter a span's covered text without moving its columns —
/// e.g. correcting "helo" to "hell" keeps the SAME `0..4` span but a DIFFERENT
/// word). [`LineIndex::still_valid`] is the one check every consumer routes
/// through, so a stale verdict can never paint a squiggle under text it never
/// actually judged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellVerdict {
    pub span: Misspelling,
    pub word: String,
}

impl SpellVerdict {
    /// One-shot [`LineIndex::still_valid`] for a caller holding a SINGLE
    /// verdict. Production filters the whole cache at once through
    /// [`visible`], so — like [`word_at`] — this convenience is for tests.
    #[cfg(test)]
    pub fn still_valid(&self, text: &str) -> bool {
        LineIndex::new(text).still_valid(self)
    }
}

pub fn keyed(text: &str, misspellings: Vec<Misspelling>) -> Vec<SpellVerdict> {
    let index = LineIndex::new(text);
    misspellings
        .into_iter()
        .map(|span| {
            let word = index.word_at(&span);
            SpellVerdict { span, word }
        })
        .collect()
}

/// Filter a KEYED verdict cache down to the render-facing spans still valid
/// against `text` — plain [`Misspelling`] positions, word text dropped (the
/// renderer never needs it). THE ONE reader every "what does the view push as
/// `misspelled`" call site routes through ([`crate::app::App::sync_view`]), so
/// the keying discipline can't be bypassed by a future caller reading `.span`
/// directly off a possibly-stale verdict — a verdict whose text has changed
/// since it was computed is silently dropped here, never painted.
pub fn visible(cache: &[SpellVerdict], text: &str) -> Vec<Misspelling> {
    let index = LineIndex::new(text);
    cache
        .iter()
        .filter(|v| index.still_valid(v))
        .map(|v| v.span)
        .collect()
}

/// Loaded-once spell checker. Holds the parsed Hunspell dictionary; `check` is a
/// pure lookup. Construction is the only fallible part (dictionary parse).
///
/// The USER (personal) DICTIONARY — the words "Add to dictionary" (Cmd-`;`)
/// accepts — rides alongside as a lowercased [`std::collections::HashSet`]: a
/// second, always-correct predicate on top of the bundled Hunspell lookup. It is
/// loaded from a plain-text word list beside `config.toml` (one word per line,
/// hand-editable) at launch — ZERO-NETWORK, a file, never a fetch — and grows in
/// memory + on disk when the user adds a word (`crate::app::App::add_to_dictionary`
/// owns the file write; this struct owns only the in-memory set).
pub struct SpellChecker {
    dict: spellbook::Dictionary,
    user_words: std::collections::HashSet<String>,
}

impl SpellChecker {
    /// Parse the bundled Hunspell dictionary for `variant`. Returns an error
    /// string if the real-world dictionary fails to parse (so the caller can
    /// REPORT it rather than silently disabling spell-check). This is the ONE
    /// real per-switch cost the dictionary picker pays (see `spell::tests`'s
    /// timed parse test) — never called on a mere navigation move. The user
    /// dictionary starts EMPTY; the caller loads it via [`Self::set_user_words`].
    pub fn new(variant: DictVariant) -> Result<Self, String> {
        let (aff, dic) = variant.files();
        let dict = spellbook::Dictionary::new(aff, dic).map_err(|e| {
            format!(
                "failed to parse bundled {} dictionary: {e}",
                variant.label()
            )
        })?;
        Ok(Self {
            dict,
            user_words: std::collections::HashSet::new(),
        })
    }

    pub fn check(&self, word: &str) -> bool {
        if self.dict.check(word) {
            return true;
        }
        let lower = word.to_lowercase();
        if lower != word && self.dict.check(&lower) {
            return true;
        }
        if self.user_words.contains(&lower) {
            return true;
        }
        false
    }

    /// REPLACE the user (personal) dictionary with `words` — the launch-time load
    /// from the on-disk word list (and the re-load after a dictionary-variant
    /// switch reconstructs the checker). Each word is trimmed + lowercased +
    /// blanks dropped, so a hand-edited file with stray casing / whitespace still
    /// matches. The one owner of the in-memory set's population.
    pub fn set_user_words<I: IntoIterator<Item = String>>(&mut self, words: I) {
        self.user_words = words
            .into_iter()
            .map(|w| w.trim().to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();
    }

    pub fn add_user_word(&mut self, word: &str) -> bool {
        let w = word.trim().to_lowercase();
        if w.is_empty() {
            return false;
        }
        self.user_words.insert(w)
    }

    #[cfg(test)]
    pub fn user_word_count(&self) -> usize {
        self.user_words.len()
    }

    pub fn misspellings(&self, text: &str) -> Vec<Misspelling> {
        misspelled_spans(text, |w| self.check(w))
    }

    /// THE ONE OWNER of the spell scope: detect misspellings honoring the
    /// buffer's language. GATED FIRST on the GLOBAL [`spellcheck_on`] toggle — OFF
    /// returns empty unconditionally, so no squiggle survives anywhere (prose or
    /// code) once the user has switched it off. `lang == None` (prose / markdown /
    /// scratch) is [`SpellChecker::misspellings`] VERBATIM over the text PAST any
    /// leading frontmatter block ([`crate::frontmatter::detect`] — metadata, not
    /// manuscript, so a `lang: ja` key is never itself squiggled), with the
    /// result's `line` numbers shifted back up by the block's line count —
    /// otherwise byte-identical, keeping the existing markdown fence /
    /// inline-code / URL skips. `Some(lang)` (a recognized CODE buffer) spell-
    /// checks ONLY the prose regions the lexer already delimits: the PROSE-tier
    /// [`crate::syntax::SynKind::Comment`] spans VERBATIM, and the
    /// [`crate::syntax::SynKind::Str`] spans FURTHER GATED on
    /// [`looks_like_prose_string`] — a STRING squiggles only when its content
    /// reads as prose (multiple space-separated words); a single CODE-VOCABULARY
    /// token (`"struct"`, `"en_AU"`, a format specifier, a CSS selector) never
    /// does. Commented-out code (`CommentCode`), identifiers, keywords, and
    /// everything else can never squiggle. Every spell call site routes through
    /// here (app debounce, capture, framebench), so live + headless can't drift.
    pub fn misspellings_for(
        &self,
        text: &str,
        lang: Option<crate::syntax::Lang>,
    ) -> Vec<Misspelling> {
        if !spellcheck_on() {
            return Vec::new();
        }
        match lang {
            None => match crate::frontmatter::detect(text) {
                Some(fm) => {
                    let line_offset = text[..fm.range.end].matches('\n').count();
                    self.misspellings(&text[fm.range.end..])
                        .into_iter()
                        .map(|m| Misspelling {
                            line: m.line + line_offset,
                            ..m
                        })
                        .collect()
                }
                None => self.misspellings(text),
            },
            Some(l) => {
                let mut ranges: Vec<std::ops::Range<usize>> = crate::syntax::spans(l, text)
                    .into_iter()
                    .filter(|(r, k)| match k {
                        crate::syntax::SynKind::Comment => true,
                        crate::syntax::SynKind::Str => {
                            text.get(r.clone()).is_some_and(looks_like_prose_string)
                        }
                        _ => false,
                    })
                    .map(|(r, _)| r)
                    .collect();
                ranges.sort_by_key(|r| r.start);
                misspelled_spans_scoped(text, |w| self.check(w), &ranges)
            }
        }
    }

    pub fn suggest(&self, word: &str) -> Vec<String> {
        let mut out = Vec::new();
        self.dict.suggest(word, &mut out);
        out
    }

    pub fn suggest_at(
        &self,
        text: &str,
        line: usize,
        col: usize,
        lang: Option<crate::syntax::Lang>,
    ) -> Option<SuggestionTarget> {
        if !spellcheck_on() {
            return None;
        }
        // Route through THE ONE OWNER of the spell scope ([`Self::misspellings_for`])
        // rather than a parallel UNSCOPED scan, so the suggest target and the DRAWN
        // squiggle can never disagree: in a CODE buffer an identifier/keyword the
        // scoped scan excludes is never offered a "correction" here either. The
        // spans arrive in document order, so the left-most one wins a column tie —
        // the same rule [`misspelling_at`] applies for prose.
        let m = self
            .misspellings_for(text, lang)
            .into_iter()
            .find(|m| m.line == line && col >= m.start_col && col <= m.end_col)?;
        let word: String = text
            .split('\n')
            .nth(m.line)
            .unwrap_or("")
            .chars()
            .skip(m.start_col)
            .take(m.end_col - m.start_col)
            .collect();
        let suggestions = self.suggest(&word);
        Some(SuggestionTarget {
            misspelling: m,
            word,
            suggestions,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SuggestionTarget {
    pub misspelling: Misspelling,
    /// The current (misspelled) word text. Carried for callers/tests that want to
    /// echo it; the live/headless pickers replace by SPAN, so the binary itself
    /// reads only `misspelling` + `suggestions`.
    #[allow(dead_code)]
    pub word: String,
    pub suggestions: Vec<String>,
}

pub fn parse_dictionary(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}

/// The misspelled word the cursor at `(line, col)` is ON or ADJACENT to, if any.
/// "Adjacent" means the cursor sits anywhere in `[start_col, end_col]` INCLUSIVE
/// of both ends, so a caret just before the first letter or just after the last
/// letter still targets the word (typical when you finish typing a word). Pure
/// (the dictionary arrives via `check`) so it's unit-testable with a stub. When
/// two spans somehow touch the same column, the earlier (left-most) one wins.
///
/// Retained as the pure UNSCOPED targeting primitive (the `[start,end]`-inclusive
/// column rule, unit-tested directly); [`SpellChecker::suggest_at`] no longer calls
/// it — it targets via THE ONE OWNER [`SpellChecker::misspellings_for`] so suggest
/// and the drawn squiggle share one scope in a code buffer.
#[allow(dead_code)]
pub fn misspelling_at<F: Fn(&str) -> bool>(
    text: &str,
    line: usize,
    col: usize,
    check: F,
) -> Option<Misspelling> {
    misspelled_spans(text, check)
        .into_iter()
        .find(|m| m.line == line && col >= m.start_col && col <= m.end_col)
}

/// Is `c` a letter we spell-check? We only check Latin-script words for v1, so
/// CJK / other-script letters are treated as non-word here (a CJK run is skipped
/// entirely, never flagged). ASCII fast-path first.
fn is_latin_letter(c: char) -> bool {
    if c.is_ascii_alphabetic() {
        return true;
    }
    if !c.is_alphabetic() {
        return false;
    }
    matches!(c as u32,
        0x00C0..=0x024F   // Latin-1 Supplement + Latin Extended-A/B
        | 0x1E00..=0x1EFF // Latin Extended Additional
    )
}

fn is_intraword_apostrophe(c: char) -> bool {
    c == '\'' || c == '\u{2019}'
}

pub fn misspelled_spans<F: Fn(&str) -> bool>(text: &str, check: F) -> Vec<Misspelling> {
    let mut out = Vec::new();
    let mut in_fence = false;

    for (line_no, line) in text.split('\n').enumerate() {
        // Fenced code block toggle: a line that is just ``` (optionally with an
        // info string / indentation) flips the state. The fence line itself is
        // never spell-checked.
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        scan_line(line, line_no, &check, &mut out);
    }
    out
}

fn scan_line<F: Fn(&str) -> bool>(
    line: &str,
    line_no: usize,
    check: &F,
    out: &mut Vec<Misspelling>,
) {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0usize;

    while i < n {
        let c = chars[i];

        if c == '`' {
            i += 1;
            while i < n && chars[i] != '`' {
                i += 1;
            }
            if i < n {
                i += 1;
            }
            continue;
        }

        if c.is_ascii_alphabetic() && url_at(&chars, i) {
            while i < n && !chars[i].is_whitespace() {
                i += 1;
            }
            continue;
        }

        if is_latin_letter(c) {
            let start = i;
            let mut skip = false;
            while i < n {
                let ch = chars[i];
                if is_latin_letter(ch) {
                    i += 1;
                } else if is_intraword_apostrophe(ch) && i + 1 < n && is_latin_letter(chars[i + 1])
                {
                    // Keep the apostrophe in the word; the following letter advances next.
                    i += 1;
                } else if ch.is_alphanumeric() {
                    skip = true;
                    i += 1;
                } else {
                    break;
                }
            }
            if skip {
                continue;
            }
            let word: String = chars[start..i].iter().collect();
            let trimmed = word.trim_end_matches(is_intraword_apostrophe);
            if trimmed.is_empty() {
                continue;
            }
            if !check(trimmed) {
                out.push(Misspelling {
                    line: line_no,
                    start_col: start,
                    end_col: start + trimmed.chars().count(),
                });
            }
            continue;
        }

        i += 1;
    }
}

/// SCOPED detection for CODE buffers: run the SAME tokenizer as
/// [`misspelled_spans`], then keep only the words whose DOCUMENT BYTE range
/// falls FULLY inside one of `prose_ranges` (the lexer-delimited prose regions —
/// prose comments + strings; ranges must be sorted by start, non-overlapping is
/// not required but typical). Scoped mode additionally drops IDENTIFIER-SHAPED
/// words ([`identifier_shaped`]) so `SelInstance` / `WGSL` / `px` never squiggle
/// even inside a comment or string. Pure (dictionary via `check`); prose buffers
/// never take this path, so their output is untouched. Line byte offsets come
/// from ONE running `split('\n')` walk and words arrive in document order, so
/// the range merge is a two-pointer O(doc) pass — fine for a debounced scan.
pub fn misspelled_spans_scoped<F: Fn(&str) -> bool>(
    text: &str,
    check: F,
    prose_ranges: &[std::ops::Range<usize>],
) -> Vec<Misspelling> {
    let all = misspelled_spans(text, check);
    if all.is_empty() || prose_ranges.is_empty() {
        return Vec::new();
    }
    debug_assert!(
        prose_ranges.windows(2).all(|w| w[0].start <= w[1].start),
        "prose_ranges must be sorted by start"
    );
    let lines: Vec<&str> = text.split('\n').collect();
    let mut line_starts: Vec<usize> = Vec::with_capacity(lines.len());
    let mut acc = 0usize;
    for l in &lines {
        line_starts.push(acc);
        acc += l.len() + 1;
    }
    let mut out = Vec::new();
    let mut ri = 0usize;
    for m in all {
        let Some(line) = lines.get(m.line) else {
            continue;
        };
        let byte_at = |col: usize| {
            line.char_indices()
                .nth(col)
                .map(|(b, _)| b)
                .unwrap_or(line.len())
        };
        let lo = line_starts[m.line] + byte_at(m.start_col);
        let hi = line_starts[m.line] + byte_at(m.end_col);
        while ri < prose_ranges.len() && prose_ranges[ri].end < hi {
            ri += 1;
        }
        let inside =
            ri < prose_ranges.len() && prose_ranges[ri].start <= lo && hi <= prose_ranges[ri].end;
        if !inside {
            continue;
        }
        let word: String = line
            .chars()
            .skip(m.start_col)
            .take(m.end_col - m.start_col)
            .collect();
        if identifier_shaped(&word) {
            continue; // SelInstance / WGSL / px — code vocabulary, never a typo
        }
        out.push(m);
    }
    out
}

/// Does a STRING LITERAL's content read as PROSE rather than a single code
/// token? Mirrors [`crate::syntax::looks_like_code`]'s shape — a small, pure,
/// DEFAULT-TO-SKIP heuristic: PROSE iff the trimmed body holds AT LEAST TWO
/// space-separated tokens that each carry a Latin letter ("hello world", "Item
/// not found" — an ordinary English phrase, incl. one with a `{placeholder}`
/// mixed in, still reads as prose and gets checked word-by-word). A SINGLE
/// token — `"struct"`, `"en_AU"`, a bare format specifier (`"{}"`, `"%d"`), a
/// CSS selector (`".foo-bar"`) — is CODE VOCABULARY, not prose, and the WHOLE
/// string is skipped (no word inside it is even considered, so a bare
/// non-English identifier never gets a chance to look like a typo). An empty
/// string, or one with fewer than two word-shaped tokens, is not prose either
/// (DEFAULT-TO-SKIP, same posture as `looks_like_code`'s DEFAULT-TO-PROSE:
/// when unsure, this heuristic prefers silence over a false-positive squiggle
/// on code vocabulary).
fn looks_like_prose_string(body: &str) -> bool {
    body.split_whitespace()
        .filter(|tok| tok.chars().any(|c| c.is_alphabetic()))
        .count()
        >= 2
}

/// True for a word that reads as CODE VOCABULARY rather than prose — the scoped
/// mode's post-filter (prose buffers never see this): ALL-CAPS of length ≥ 2
/// (`WGSL`), an INTERIOR uppercase (CamelCase — `SelInstance`; a plain
/// sentence-initial capital stays checkable), an underscore, or anything
/// shorter than 3 chars (`px`, `en`-style fragments).
fn identifier_shaped(word: &str) -> bool {
    let n = word.chars().count();
    if n < 3 || word.contains('_') {
        return true;
    }
    if n >= 2 && word.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
        return true;
    }
    word.chars().skip(1).any(|c| c.is_uppercase())
}

fn url_at(chars: &[char], i: usize) -> bool {
    const PREFIXES: &[&str] = &["https://", "http://", "www."];
    for p in PREFIXES {
        let pc: Vec<char> = p.chars().collect();
        if i + pc.len() <= chars.len() {
            let mut ok = true;
            for (k, &want) in pc.iter().enumerate() {
                if !chars[i + k].eq_ignore_ascii_case(&want) {
                    ok = false;
                    break;
                }
            }
            if ok {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests;

/// Adversarial corpus probes exercise the bundled dictionary rather than a
/// stub predicate, so they remain separate from the fast pure tests.
#[cfg(test)]
mod verifier_probe {
    use super::*;

    /// Real-dictionary probe: typos flag only in prose comments and strings,
    /// never identifiers, commented-out code, or code-vocabulary strings.
    #[test]
    fn verifier_scoped_spell_real_dict() {
        let _g = crate::testlock::serial();
        let sc = SpellChecker::new(DictVariant::EnUs).expect("bundled dictionary");
        let text = "\
// The SelInstance atlas uses WGSL px offsets and must recieve the update.\n\
// let recieve = definately_broken(px);\n\
fn recieve_stuff(definately: &str) -> &str {\n\
    let msg = \"definately a mispeled string\";\n\
    definately\n\
}\n";
        let ms = sc.misspellings_for(text, Some(crate::syntax::Lang::Rust));
        let words: Vec<(usize, String)> = ms
            .iter()
            .map(|m| {
                let l = text.split('\n').nth(m.line).unwrap();
                (
                    m.line,
                    l.chars()
                        .skip(m.start_col)
                        .take(m.end_col - m.start_col)
                        .collect(),
                )
            })
            .collect();
        assert_eq!(
            words,
            vec![
                (0, "recieve".to_string()),
                (3, "definately".to_string()),
                (3, "mispeled".to_string()),
            ],
            "scoped spell must flag exactly the prose-comment + string typos, \
             never the commented-out-code line (1), real source identifiers \
             (2, 4), or the shape-filtered `SelInstance`/`WGSL`/`px` tokens: {words:?}"
        );
        // lang=None equality: byte-identical prose behavior (the scoped path is
        // a strict NARROWING of the unscoped one, never a parallel rewrite of it).
        let prose = "Plain prose with a definately real typo.\n\
                      And SelInstance stays flagged here (prose keeps the old behavior).\n";
        assert_eq!(sc.misspellings_for(prose, None), sc.misspellings(prose));
    }

    #[test]
    fn verifier_scoped_boundaries() {
        fn one_range(range: std::ops::Range<usize>) -> Vec<std::ops::Range<usize>> {
            vec![range]
        }

        let none = |_: &str| false;
        // A word STRADDLING the prose-range boundary must not flag; one that
        // exactly fills a range must.
        let text = "abcde fghij";
        assert!(
            misspelled_spans_scoped(text, none, &one_range(0..8))
                .iter()
                .all(|m| m.start_col == 0),
            "straddling word (fghij over byte 8) must not flag"
        );
        let ms = misspelled_spans_scoped(text, none, &one_range(6..11));
        assert_eq!(ms.len(), 1);
        assert_eq!((ms[0].start_col, ms[0].end_col), (6, 11));
        // Identifier shapes inside a fully-kept range never flag.
        // NOTE: the tokenizer splits snake_case at `_`, so its halves are plain
        // 3+-char lowercase runs and DO reach the dictionary (the `_` arm of
        // `identifier_shaped` is unreachable post-tokenization).
        let t2 = "SelInstance WGSL px foo_bar someword";
        let ms2 = misspelled_spans_scoped(t2, none, &one_range(0..t2.len()));
        let l: Vec<String> = ms2
            .iter()
            .map(|m| {
                t2.chars()
                    .skip(m.start_col)
                    .take(m.end_col - m.start_col)
                    .collect()
            })
            .collect();
        assert_eq!(
            l,
            vec!["foo".to_string(), "bar".to_string(), "someword".to_string()]
        );
        // Multi-byte safety: a kept range after a multi-byte char still maps.
        let t3 = "caf\u{e9} recieve";
        let ms3 = misspelled_spans_scoped(t3, none, &one_range(0..t3.len()));
        assert_eq!(ms3.len(), 2);
    }

    #[test]
    fn verifier_real_dict_code_corpus() {
        let _g = crate::testlock::serial();
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        for f in [
            "src/render/rects.rs",
            "src/render/spans.rs",
            "src/theme/model.rs",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(f);
            let text = std::fs::read_to_string(&path).unwrap();
            let ms = sc.misspellings_for(&text, Some(crate::syntax::Lang::Rust));
            let mut words: Vec<String> = ms
                .iter()
                .map(|m| {
                    let l = text.split('\n').nth(m.line).unwrap_or("");
                    l.chars()
                        .skip(m.start_col)
                        .take(m.end_col - m.start_col)
                        .collect()
                })
                .collect();
            words.sort();
            words.dedup();
            println!(
                "{f}: {} squiggles, {} unique: {:?}",
                ms.len(),
                words.len(),
                words
            );
            // The scope must strictly shrink the flag set vs the unscoped scan
            // (identifiers/keywords outside prose spans no longer squiggle) and
            // no shape-filtered word may leak through.
            assert!(
                ms.len() < sc.misspellings(&text).len(),
                "{f}: scoped scan must flag fewer words than unscoped ({} vs {})",
                ms.len(),
                sc.misspellings(&text).len()
            );
            for w in &words {
                assert!(
                    w.chars().count() >= 3 && !w.contains('_'),
                    "shape filter leaked: {w}"
                );
                assert!(
                    !w.chars().skip(1).any(|c| c.is_uppercase())
                        || !w.chars().next().unwrap().is_lowercase(),
                    "interior-uppercase word leaked: {w}"
                );
            }
        }
    }

    /// The naive per-span line walk [`LineIndex`] replaces, kept as the
    /// correctness ORACLE: the index must agree with it on every span shape,
    /// including the ones that resolve to `""`.
    fn word_at_reference(text: &str, m: &Misspelling) -> String {
        text.split('\n')
            .nth(m.line)
            .unwrap_or("")
            .chars()
            .skip(m.start_col)
            .take(m.end_col.saturating_sub(m.start_col))
            .collect()
    }

    #[test]
    fn line_index_agrees_with_the_naive_walk_on_every_span_shape() {
        let _g = crate::testlock::serial();
        // Deliberately awkward: an empty line, a trailing empty line, CJK and
        // combining-mark text (char columns are NOT byte offsets), a lone CR
        // that is CONTENT not a break, and no trailing newline on the last line.
        let texts = [
            "hello wrld\n\nsecond line\n日本語のテスト\ncafe\u{301} x\nlast",
            "",
            "\n",
            "single",
            "a\rb\nc",
            "trailing\n",
        ];
        for text in texts {
            let index = LineIndex::new(text);
            for line in 0..8 {
                for start_col in 0..8 {
                    // end_col deliberately sweeps BELOW start_col (degenerate,
                    // must yield "") and far past the line end (must clamp).
                    for end_col in 0..12 {
                        let m = Misspelling {
                            line,
                            start_col,
                            end_col,
                        };
                        assert_eq!(
                            index.word_at(&m),
                            word_at_reference(text, &m),
                            "LineIndex disagreed with the naive walk at \
                             line={line} cols={start_col}..{end_col} in {text:?}"
                        );
                        // the free function must route through the same owner
                        assert_eq!(word_at(text, &m), word_at_reference(text, &m));
                    }
                }
            }
        }
    }

    /// Build a document plus spans for EVERY word on the LAST `tail_lines`
    /// lines. Placing spans in the TAIL is the whole point: a span on line 0
    /// costs the naive walk nothing, so a law built from early spans passes
    /// with the quadratic still in place.
    fn tail_span_doc(
        lines: usize,
        words_per_line: usize,
        tail_lines: usize,
    ) -> (String, Vec<Misspelling>) {
        let mut text = String::new();
        for l in 0..lines {
            for w in 0..words_per_line {
                if w > 0 {
                    text.push(' ');
                }
                text.push_str(&format!("wrd{l:05}x{w:02}"));
            }
            if l + 1 < lines {
                text.push('\n');
            }
        }
        let mut spans = Vec::new();
        for l in lines.saturating_sub(tail_lines)..lines {
            for w in 0..words_per_line {
                let start_col = w * 12;
                spans.push(Misspelling {
                    line: l,
                    start_col,
                    end_col: start_col + 11,
                });
            }
        }
        (text, spans)
    }

    /// LAW — resolving a BATCH of spans must not re-walk the document per span.
    ///
    /// `keyed` (every edit) and `visible` (every `sync_view`, so every scroll
    /// and drag too) each resolve the whole span set at once. With a per-span
    /// line walk that is O(spans x lines) and a novel-length manuscript stalls
    /// for hundreds of ms per keystroke; with [`LineIndex`] it is O(lines +
    /// spans). The ceiling below sits ~10x above the linear cost and well under
    /// the quadratic one, so it fails on the bug it names rather than on a slow
    /// machine.
    #[test]
    fn batch_span_resolution_is_not_quadratic_in_document_length() {
        let _g = crate::testlock::serial();
        let (text, spans) = tail_span_doc(4000, 12, 800);
        assert_eq!(spans.len(), 9600, "the law needs a large tail span set");
        assert!(text.len() > 250_000, "and a long document: {}", text.len());

        let t = std::time::Instant::now();
        let cache = keyed(&text, spans.clone());
        let keyed_ms = t.elapsed().as_secs_f64() * 1000.0;

        let t = std::time::Instant::now();
        let vis = visible(&cache, &text);
        let visible_ms = t.elapsed().as_secs_f64() * 1000.0;

        // Non-vacuity: the spans must actually RESOLVE. A bug that returned ""
        // for everything would be fast and useless, so pin the payload too.
        assert_eq!(vis.len(), spans.len(), "every tail span must stay valid");
        assert_eq!(
            cache[0].word, "wrd03200x00",
            "spans must resolve to real words"
        );

        // The linear cost here is ~2ms and the per-span walk it replaced ~590ms,
        // so this ceiling clears the fix by ~50x and trips the bug by ~6x.
        const CEILING_MS: f64 = 100.0;
        assert!(
            keyed_ms < CEILING_MS && visible_ms < CEILING_MS,
            "batch span resolution went quadratic: keyed {keyed_ms:.0}ms, \
             visible {visible_ms:.0}ms (ceiling {CEILING_MS:.0}ms each)"
        );
    }
}
