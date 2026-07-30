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
    fn as_u8(self) -> u8 {
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

static ACTIVE_VARIANT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

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
static SPELLCHECK_ON: crate::toggle::Toggle = crate::toggle::Toggle::new(true);

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

/// The exact word text at `m`'s span in `text` — the same char-column
/// extraction [`SpellChecker::suggest_at`] uses to read the word it's about to
/// offer corrections for. `""` when the span no longer resolves (a vanished
/// line, or `end_col <= start_col`) — never a panic on a stale span read
/// against edited-out text.
pub fn word_at(text: &str, m: &Misspelling) -> String {
    text.split('\n')
        .nth(m.line)
        .unwrap_or("")
        .chars()
        .skip(m.start_col)
        .take(m.end_col.saturating_sub(m.start_col))
        .collect()
}

/// A spell verdict KEYED to the exact word text it judged (the COMPLETED-WORD-
/// LAG fix's "keyed" half): [`Misspelling`] alone says WHERE a word was
/// flagged; this additionally freezes WHAT text was there at judgment time, so
/// a caller holding a verdict across an edit can tell "still genuinely
/// misspelled" apart from "this span's text has since changed underneath it"
/// (an edit can shift/alter a span's covered text without moving its columns —
/// e.g. correcting "helo" to "hell" keeps the SAME `0..4` span but a DIFFERENT
/// word). [`Self::still_valid`] is the one check every consumer routes
/// through, so a stale verdict can never paint a squiggle under text it never
/// actually judged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellVerdict {
    pub span: Misspelling,
    pub word: String,
}

impl SpellVerdict {
    /// True iff `text` still holds this verdict's EXACT word at its span. A
    /// `false` means the text under this span changed since the verdict was
    /// computed — painting it now would show a squiggle under the WRONG word
    /// (or a stale MISSPELLED squiggle for a word that's since been fixed):
    /// the just-completed-word flash this type exists to make structurally
    /// impossible.
    pub fn still_valid(&self, text: &str) -> bool {
        word_at(text, &self.span) == self.word
    }
}

pub fn keyed(text: &str, misspellings: Vec<Misspelling>) -> Vec<SpellVerdict> {
    misspellings
        .into_iter()
        .map(|span| {
            let word = word_at(text, &span);
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
    cache
        .iter()
        .filter(|v| v.still_valid(text))
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
mod tests {
    use super::*;

    fn one_range(range: std::ops::Range<usize>) -> Vec<std::ops::Range<usize>> {
        vec![range]
    }

    fn stub<'a>(correct: &'a [&'a str]) -> impl Fn(&str) -> bool + 'a {
        move |w: &str| correct.iter().any(|c| c.eq_ignore_ascii_case(w))
    }

    fn cols(m: &Misspelling) -> (usize, usize, usize) {
        (m.line, m.start_col, m.end_col)
    }

    #[test]
    fn flags_a_single_bad_word() {
        let good = stub(&["hello", "world"]);
        let ms = misspelled_spans("hello wrld", &good);
        assert_eq!(ms.len(), 1);
        assert_eq!(cols(&ms[0]), (0, 6, 10)); // "wrld" at cols 6..10
    }

    #[test]
    fn correct_words_not_flagged() {
        let good = stub(&["the", "quick", "brown", "fox"]);
        assert!(misspelled_spans("the quick brown fox", &good).is_empty());
    }

    #[test]
    fn columns_are_char_indices_after_punctuation() {
        let good = stub(&["a", "test"]);
        let ms = misspelled_spans("a, tset.", &good);
        assert_eq!(ms.len(), 1);
        assert_eq!(cols(&ms[0]), (0, 3, 7));
    }

    #[test]
    fn intraword_apostrophe_kept_as_one_word() {
        let good = stub(&["don't", "it's"]);
        assert!(misspelled_spans("don't it's", &good).is_empty());
        let bad = stub(&["it's"]);
        let ms = misspelled_spans("dont", &bad);
        assert_eq!(ms.len(), 1);
        assert_eq!(cols(&ms[0]), (0, 0, 4));
    }

    #[test]
    fn trailing_apostrophe_trimmed() {
        let good = stub(&["dogs"]);
        let ms = misspelled_spans("dogs' bones", &good);
        assert_eq!(ms.iter().filter(|m| m.start_col == 0).count(), 0);
    }

    #[test]
    fn digits_make_a_token_unchecked() {
        let none = stub(&[]); // nothing is correct
        assert!(misspelled_spans("abc123 x2 v8", &none).is_empty());
    }

    #[test]
    fn cjk_run_is_skipped() {
        let none = stub(&[]);
        // Japanese should never be flagged (non-Latin script).
        assert!(misspelled_spans("日本語のテスト", &none).is_empty());
        let ms = misspelled_spans("日本 bad", &none);
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].start_col, 3);
    }

    #[test]
    fn inline_code_is_skipped() {
        let none = stub(&[]);
        // The word inside backticks must NOT be flagged.
        let ms = misspelled_spans("use `wgpu` here", &none);
        assert!(ms.iter().all(|m| {
            let w_start = m.start_col;
            w_start != 5 // wgpu would start at col 5
        }));
        assert_eq!(ms.len(), 2);
    }

    #[test]
    fn fenced_code_block_is_skipped() {
        let none = stub(&[]);
        let text = "before\n```\nnonsenseword\n```\nafter";
        let ms = misspelled_spans(text, &none);
        let lines: Vec<usize> = ms.iter().map(|m| m.line).collect();
        assert!(lines.contains(&0));
        assert!(lines.contains(&4));
        assert!(!lines.contains(&2), "fenced word must be skipped");
    }

    #[test]
    fn url_is_skipped() {
        let none = stub(&[]);
        // The misspelling embedded in the URL ("teh") must NOT be flagged.
        let ms = misspelled_spans("see https://example.com/teh ok", &none);
        assert_eq!(ms.len(), 2);
        let words: Vec<usize> = ms.iter().map(|m| m.start_col).collect();
        assert_eq!(words, vec![0, 28]); // "see"@0, "ok"@28
    }

    #[test]
    fn www_url_is_skipped() {
        let none = stub(&["go", "to"]);
        let ms = misspelled_spans("go to www.bad-spelll.com", &none);
        assert!(ms.is_empty(), "www. URL must be skipped");
    }

    #[test]
    fn parse_dictionary_takes_one_word_per_line_dropping_blanks_and_comments() {
        let text = "# my words\nwrold\n\n  spacey  \n# another comment\nzorp\n";
        assert_eq!(parse_dictionary(text), vec!["wrold", "spacey", "zorp"]);
        assert!(
            parse_dictionary("").is_empty(),
            "an empty file is an empty list"
        );
        assert!(parse_dictionary("# only a comment\n\n").is_empty());
    }

    #[test]
    fn user_dictionary_word_is_never_flagged_case_insensitively() {
        let mut sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        assert!(
            !sc.check("wrold"),
            "precondition: 'wrold' is misspelled by the base dict"
        );
        sc.set_user_words(parse_dictionary("wrold\n"));
        assert_eq!(sc.user_word_count(), 1);
        for w in ["wrold", "Wrold", "WROLD"] {
            assert!(
                sc.check(w),
                "{w:?} is correct once added (case-insensitive)"
            );
        }
        assert!(!sc.check("teh"), "an unrelated typo still flags");
    }

    #[test]
    fn add_user_word_reports_novelty_and_normalizes() {
        let mut sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        assert!(
            sc.add_user_word("  Zorp  "),
            "a new word is newly added (trimmed)"
        );
        assert!(
            !sc.add_user_word("zorp"),
            "the same word (any casing) is NOT re-added"
        );
        assert!(!sc.add_user_word("   "), "a blank word is never added");
        assert_eq!(sc.user_word_count(), 1);
        assert!(sc.check("zorp") && sc.check("ZORP"));
    }

    #[test]
    fn misspellings_for_honors_the_user_dictionary() {
        let _g = crate::testlock::serial();
        // The spell-scope owner must remove the drawn squiggle too.
        let mut sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "wrold peace\n";
        assert!(
            sc.misspellings_for(text, None)
                .iter()
                .any(|m| m.start_col == 0),
            "precondition: 'wrold' squiggles before it is added"
        );
        sc.add_user_word("wrold");
        assert!(
            !sc.misspellings_for(text, None)
                .iter()
                .any(|m| m.start_col == 0),
            "after Add to dictionary, 'wrold' no longer squiggles"
        );
    }

    #[test]
    fn real_dictionary_parses_and_checks_known_words() {
        let sc = SpellChecker::new(DictVariant::EnUs).expect("bundled en_US dictionary must parse");
        for w in [
            "sentence",
            "misspelled",
            "typo",
            "definitely",
            "receive",
            "the",
            "quick",
            "brown",
            "fox",
            "hello",
        ] {
            assert!(sc.check(w), "{w:?} should be correct");
        }
        for w in ["sentance", "mispelled", "tpyo", "definately", "recieve"] {
            assert!(!sc.check(w), "{w:?} should be flagged");
        }
    }

    #[test]
    fn real_dictionary_handles_capitalization() {
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        assert!(sc.check("Hello"));
        assert!(sc.check("The"));
        assert!(!sc.check("Definately"));
    }

    #[test]
    fn real_dictionary_on_fixture_finds_exactly_the_five() {
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "This sentance has a few mispelled words in it.\n\
                    Inline code like `wgpu` and `cosmic_text` must NOT be flagged.\n\
                    ```\nfn main() { let zzz = nonsenseword; }\n```\n\
                    A link https://example.com/teh should be skipped too.\n\
                    Another tpyo here, definately and recieve.";
        let ms = sc.misspellings(text);
        let words: Vec<String> = ms
            .iter()
            .map(|m| {
                let line = text.split('\n').nth(m.line).unwrap();
                line.chars()
                    .skip(m.start_col)
                    .take(m.end_col - m.start_col)
                    .collect()
            })
            .collect();
        assert_eq!(
            words,
            vec!["sentance", "mispelled", "tpyo", "definately", "recieve"],
            "exactly the five deliberate misspellings, nothing from code/URL"
        );
    }

    // --- JAPANESE PINNING (real dictionary): the scanner is ASCII/Latin-word-
    // based ([`is_latin_letter`]), not a language detector — it never even LOOKS
    // at a CJK run, so genuine Japanese prose can never squiggle no matter how
    // "wrong" it might read to a Latin dictionary. Pinned against the REAL
    // bundled en_US dictionary (not a stub), both for prose (`lang == None`) and
    // for a CODE buffer's scoped comment/string scan (`misspellings_for`), so a
    // future change to either path can't quietly start flagging kanji/kana. ---

    #[test]
    fn real_dictionary_never_squiggles_pure_japanese_prose() {
        let _g = crate::testlock::serial();
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        // The Latin-word scanner never considers Japanese text.
        let text = "今日は天気がいいですね。散歩に行きましょう。";
        assert!(
            sc.misspellings(text).is_empty(),
            "pure JP prose must never squiggle"
        );
        // Pin the buffer-aware entry point used by render and capture.
        assert!(sc.misspellings_for(text, None).is_empty());
        // Japanese code comments stay silent through the scoped path too.
        let code = format!("// {text}\nfn f() {{}}\n");
        assert!(
            sc.misspellings_for(&code, Some(crate::syntax::Lang::Rust))
                .is_empty()
        );
    }

    #[test]
    fn real_dictionary_mixed_japanese_and_english_only_flags_the_english_word() {
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "今日は良い天気です recieve 頑張りましょう。";
        let ms = sc.misspellings(text);
        let words: Vec<String> = ms
            .iter()
            .map(|m| {
                text.chars()
                    .skip(m.start_col)
                    .take(m.end_col - m.start_col)
                    .collect()
            })
            .collect();
        assert_eq!(
            words,
            vec!["recieve"],
            "only the embedded English typo flags: {words:?}"
        );
        let clean = "今日は良い天気です hello 頑張りましょう。";
        assert!(
            sc.misspellings(clean).is_empty(),
            "a correct embedded English word is silent"
        );
    }

    /// All three bundled dictionaries parse and answer a shared known-good word.
    /// "Never fabricate dictionary content" is enforced upstream (the files are
    /// the real LibreOffice downloads); this is the in-repo guarantee that they
    /// stay parseable as spellbook (or the bundled files) evolve.
    #[test]
    fn all_three_bundled_dictionaries_parse() {
        for v in DictVariant::ALL {
            let sc = SpellChecker::new(v).unwrap_or_else(|e| panic!("{}: {e}", v.label()));
            assert!(
                sc.check("hello"),
                "{}: a universally-shared word must check",
                v.label()
            );
        }
    }

    #[test]
    fn variants_disagree_on_british_spelling() {
        let us = SpellChecker::new(DictVariant::EnUs).unwrap();
        let gb = SpellChecker::new(DictVariant::EnGb).unwrap();
        let au = SpellChecker::new(DictVariant::EnAu).unwrap();
        assert!(
            !us.check("colour"),
            "en_US should reject the British spelling"
        );
        assert!(gb.check("colour"), "en_GB should accept it");
        assert!(au.check("colour"), "en_AU should accept it");
        assert!(us.check("color"), "en_US should accept its own spelling");
    }

    #[test]
    fn parse_cost_per_dictionary_variant() {
        for v in DictVariant::ALL {
            let t0 = std::time::Instant::now();
            let sc = SpellChecker::new(v).unwrap();
            let elapsed = t0.elapsed();
            eprintln!(
                "spell dictionary parse {}: {:.2}ms",
                v.label(),
                elapsed.as_secs_f64() * 1000.0
            );
            assert!(
                sc.check("the"),
                "a parsed dictionary must still answer lookups"
            );
        }
    }

    #[test]
    fn dict_variant_label_round_trips() {
        for v in DictVariant::ALL {
            assert_eq!(DictVariant::from_label(v.label()), Some(v));
        }
        assert_eq!(
            DictVariant::from_label("english (us)"),
            Some(DictVariant::EnUs)
        );
        assert_eq!(DictVariant::from_label("nonsense"), None);
    }

    #[test]
    fn active_variant_defaults_to_en_us_and_round_trips_through_the_global() {
        let _g = crate::testlock::serial();
        let saved = active_variant();
        set_active_variant(DictVariant::EnUs);
        assert_eq!(
            active_variant(),
            DictVariant::EnUs,
            "absent override defaults to en_US"
        );
        set_active_variant(DictVariant::EnGb);
        assert_eq!(active_variant(), DictVariant::EnGb);
        set_active_variant(DictVariant::EnAu);
        assert_eq!(active_variant(), DictVariant::EnAu);
        set_active_variant(saved);
    }

    #[test]
    fn scoped_keeps_only_words_fully_inside_prose_ranges() {
        let none = stub(&[]); // empty dict: every word flags — the SCOPE decides
        let text = "alpha \"beta\" gamma";
        let ms = misspelled_spans_scoped(text, &none, &one_range(6..12));
        assert_eq!(ms.len(), 1, "only the in-range word survives: {ms:?}");
        assert_eq!(cols(&ms[0]), (0, 7, 11)); // "beta"
        let ms = misspelled_spans_scoped(text, &none, &one_range(6..9));
        assert!(ms.is_empty(), "a straddling word must not squiggle");
        assert!(misspelled_spans_scoped(text, &none, &[]).is_empty());
    }

    #[test]
    fn scoped_drops_identifier_shaped_words() {
        let none = stub(&[]);
        let text = "\"SelInstance WGSL px some_var word\"";
        let ms = misspelled_spans_scoped(text, &none, &one_range(0..text.len()));
        let words: Vec<String> = ms
            .iter()
            .map(|m| {
                text.chars()
                    .skip(m.start_col)
                    .take(m.end_col - m.start_col)
                    .collect()
            })
            .collect();
        assert!(
            !words.iter().any(|w| w == "SelInstance"),
            "CamelCase never squiggles"
        );
        assert!(
            !words.iter().any(|w| w == "WGSL"),
            "ALL-CAPS never squiggles"
        );
        assert!(
            !words.iter().any(|w| w == "px"),
            "short fragments never squiggle"
        );
        assert!(
            words.iter().any(|w| w == "word"),
            "a plain prose word still checks: {words:?}"
        );
    }

    #[test]
    fn misspellings_for_none_is_exactly_the_unscoped_scan() {
        let _g = crate::testlock::serial();
        // Prose remains byte-identical to the unscoped scanner.
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "This sentance has a typo.\n```\nfenced zzz\n```\nsee `wgpu` and www.x.com ok";
        assert_eq!(sc.misspellings_for(text, None), sc.misspellings(text));
    }

    #[test]
    fn misspellings_for_excludes_a_leading_frontmatter_block() {
        let _g = crate::testlock::serial();
        // Frontmatter is metadata; body results retain whole-document lines.
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        // Only the body's line-3 "sentance" may appear.
        let text = "---\nlang: notalang\n---\nThis sentance has a typo.\n";
        let ms = sc.misspellings_for(text, None);
        assert!(
            ms.iter().all(|m| m.line >= 3),
            "no misspelling may fall inside the frontmatter block: {ms:?}"
        );
        assert!(
            ms.iter().any(|m| m.line == 3),
            "the body's own misspelling still lands at its correct (shifted) line: {ms:?}"
        );
        let plain = "This sentance has a typo.\n";
        assert_eq!(sc.misspellings_for(plain, None), sc.misspellings(plain));
    }

    #[test]
    fn misspellings_for_scopes_code_buffers_to_comments_and_strings() {
        let _g = crate::testlock::serial();
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        // Only prose comments and prose-shaped strings are in scope.
        let text = "// This sentance explains the plan.\n\
                    // SelInstance WGSL px sizes here.\n\
                    fn zzxqv() { let s = \"definately a typo\"; }\n\
                    // let recieve = 1;\n";
        let ms = sc.misspellings_for(text, Some(crate::syntax::Lang::Rust));
        let words: Vec<String> = ms
            .iter()
            .map(|m| {
                let line = text.split('\n').nth(m.line).unwrap();
                line.chars()
                    .skip(m.start_col)
                    .take(m.end_col - m.start_col)
                    .collect()
            })
            .collect();
        assert_eq!(
            words,
            vec!["sentance", "definately"],
            "comment + string typos flag; identifiers / code vocabulary / \
             commented-out code never do"
        );
    }

    #[test]
    fn suggest_at_honors_code_scope_matching_the_drawn_squiggle() {
        let _g = crate::testlock::serial();
        let saved = spellcheck_on();
        set_spellcheck_on(true);
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        // A rust buffer: a misspelled DEFINITION identifier (bare code — the
        // scoped squiggle skips it), a prose typo inside a STRING literal, and a
        // prose typo inside a COMMENT. suggest must agree with the squiggle.
        let text = "fn zzxqv() { let s = \"definately a typo\"; }\n// This sentance explains.\n";
        let line0 = text.split('\n').next().unwrap();
        let ident_col = line0.find("zzxqv").unwrap() + 1; // inside the identifier (ASCII)
        assert!(
            sc.suggest_at(text, 0, ident_col, Some(crate::syntax::Lang::Rust))
                .is_none(),
            "a bare code identifier has no squiggle, so suggest is a no-op there"
        );
        assert!(
            sc.suggest_at(text, 0, ident_col, None).is_some(),
            "unscoped, the same identifier is a normal misspelling"
        );
        let str_col = line0.find("definately").unwrap() + 1;
        let t = sc
            .suggest_at(text, 0, str_col, Some(crate::syntax::Lang::Rust))
            .expect("a prose typo in a string still suggests");
        assert!(t.suggestions.iter().any(|w| w == "definitely"));
        let line1 = text.split('\n').nth(1).unwrap();
        let com_col = line1.find("sentance").unwrap() + 1;
        let t = sc
            .suggest_at(text, 1, com_col, Some(crate::syntax::Lang::Rust))
            .expect("a prose typo in a comment still suggests");
        assert!(t.suggestions.iter().any(|w| w == "sentence"));
        set_spellcheck_on(saved);
    }

    #[test]
    fn suggest_at_excludes_a_frontmatter_block_matching_the_squiggle() {
        // A misspelled VALUE inside a `---` frontmatter block draws no squiggle
        // (`misspellings_for` strips the block), so suggest — routed through that
        // SAME one owner — must offer no target there either, while the BODY's own
        // typo still resolves. This is the suggest/squiggle-agree contract for
        // metadata (the code-scope analog of `suggest_at_honors_code_scope`).
        let _g = crate::testlock::serial();
        let saved = spellcheck_on();
        set_spellcheck_on(true);
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "---\nlang: notalang\n---\nThis sentance has a typo.\n";
        let fm_col = "lang: ".chars().count() + 1; // inside "notalang" on line 1
        assert!(
            sc.suggest_at(text, 1, fm_col, None).is_none(),
            "a typo inside a frontmatter block has no squiggle, so suggest is silent"
        );
        let body_col = "This ".chars().count() + 1; // inside "sentance" on line 3
        let t = sc
            .suggest_at(text, 3, body_col, None)
            .expect("the body's own typo still suggests");
        assert!(t.suggestions.iter().any(|w| w == "sentence"));
        set_spellcheck_on(saved);
    }

    #[test]
    fn looks_like_prose_string_needs_two_word_shaped_tokens() {
        assert!(!looks_like_prose_string(""));
        assert!(!looks_like_prose_string("struct"));
        assert!(!looks_like_prose_string("en_AU"));
        assert!(
            !looks_like_prose_string("{}"),
            "a bare format placeholder is one token"
        );
        assert!(
            !looks_like_prose_string("%d"),
            "a bare format specifier is one token"
        );
        assert!(
            !looks_like_prose_string(".foo-bar"),
            "a CSS selector is one token"
        );
        assert!(looks_like_prose_string("hello world"));
        assert!(
            looks_like_prose_string("Item {name} not found"),
            "a sentence with a placeholder is still prose"
        );
    }

    #[test]
    fn string_prose_gate_silences_single_token_code_strings() {
        let _g = crate::testlock::serial();
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "const DEF_KEYWORDS: &[&str] = &[\n    \
                    \"fn\", \"struct\", \"enum\", \"trait\", \"type\", \"union\", \
                    \"const\", \"static\", \"mod\",\n];\n\
                    const CONST_WORDS: &[&str] = &[\"true\", \"false\", \"None\"];\n";
        let ms = sc.misspellings_for(text, Some(crate::syntax::Lang::Rust));
        assert!(
            ms.is_empty(),
            "single-token code-vocabulary strings must never squiggle: {ms:?}"
        );
    }

    #[test]
    fn string_prose_gate_keeps_the_real_rust_lexer_silent() {
        let _g = crate::testlock::serial();
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = include_str!("syntax/rust.rs");
        let ms = sc.misspellings_for(text, Some(crate::syntax::Lang::Rust));
        let flagged: Vec<String> = ms
            .iter()
            .map(|m| {
                let line = text.split('\n').nth(m.line).unwrap();
                line.chars()
                    .skip(m.start_col)
                    .take(m.end_col - m.start_col)
                    .collect()
            })
            .collect();
        for kw in [
            "fn", "struct", "enum", "trait", "type", "union", "const", "static", "mod", "true",
            "false", "None",
        ] {
            assert!(
                !flagged.iter().any(|w| w == kw),
                "{kw:?} must never squiggle as a bare code-vocabulary string: {flagged:?}"
            );
        }
    }

    #[test]
    fn misspellings_for_still_checks_multi_word_prose_strings_in_code() {
        let _g = crate::testlock::serial();
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "fn f() { let msg = \"this has a typo teh\"; }\n";
        let ms = sc.misspellings_for(text, Some(crate::syntax::Lang::Rust));
        let words: Vec<String> = ms
            .iter()
            .map(|m| {
                let line = text.split('\n').nth(m.line).unwrap();
                line.chars()
                    .skip(m.start_col)
                    .take(m.end_col - m.start_col)
                    .collect()
            })
            .collect();
        assert_eq!(
            words,
            vec!["teh"],
            "a genuine multi-word prose string still checks: {words:?}"
        );
    }

    #[test]
    fn spellcheck_defaults_on_and_toggle_flips_it() {
        let _g = crate::testlock::serial();
        let saved = spellcheck_on();
        set_spellcheck_on(true);
        assert!(spellcheck_on(), "absent override defaults ON");
        assert!(
            !toggle(),
            "toggle flips ON -> off and returns the new state"
        );
        assert!(!spellcheck_on());
        assert!(toggle(), "toggle flips off -> ON and returns the new state");
        assert!(spellcheck_on());
        set_spellcheck_on(saved);
    }

    #[test]
    fn spellcheck_off_silences_misspellings_for_everywhere_prose_and_code() {
        let _g = crate::testlock::serial();
        let saved = spellcheck_on();
        set_spellcheck_on(true);
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let prose = "This sentance has a typo.";
        let code = "// This sentance explains the plan.\nfn f() { let s = \"a typo teh here\"; }\n";
        assert!(
            !sc.misspellings_for(prose, None).is_empty(),
            "on: prose still detects"
        );
        assert!(
            !sc.misspellings_for(code, Some(crate::syntax::Lang::Rust))
                .is_empty(),
            "on: scoped code still detects"
        );
        set_spellcheck_on(false);
        assert!(
            sc.misspellings_for(prose, None).is_empty(),
            "off: prose is silent too"
        );
        assert!(
            sc.misspellings_for(code, Some(crate::syntax::Lang::Rust))
                .is_empty(),
            "off: scoped code is silent too"
        );
        set_spellcheck_on(saved);
    }

    #[test]
    fn spellcheck_off_makes_suggest_at_a_calm_no_op() {
        let _g = crate::testlock::serial();
        let saved = spellcheck_on();
        set_spellcheck_on(true);
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "Please recieve this.";
        assert!(
            sc.suggest_at(text, 0, 9, None).is_some(),
            "on: a misspelling still resolves"
        );
        set_spellcheck_on(false);
        assert!(
            sc.suggest_at(text, 0, 9, None).is_none(),
            "off: the same cursor is now a calm no-op"
        );
        set_spellcheck_on(saved);
    }

    #[test]
    fn misspelling_at_targets_word_under_or_adjacent_to_cursor() {
        let good = stub(&["the", "quick"]);
        let text = "the wrld here";
        let m = misspelling_at(text, 0, 5, &good).expect("cursor in word");
        assert_eq!((m.start_col, m.end_col), (4, 8));
        assert!(misspelling_at(text, 0, 4, &good).is_some());
        assert!(misspelling_at(text, 0, 8, &good).is_some());
        assert!(misspelling_at(text, 0, 1, &good).is_none());
        assert!(misspelling_at("the quick", 0, 2, &good).is_none());
    }

    #[test]
    fn real_dictionary_suggests_corrections() {
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let s = sc.suggest("teh");
        assert!(!s.is_empty(), "engine should offer a correction for 'teh'");
        assert!(
            s.iter().any(|w| w == "the"),
            "'the' should be among the suggestions for 'teh': {s:?}"
        );
        let s = sc.suggest("recieve");
        assert!(
            s.iter().any(|w| w == "receive"),
            "'receive' should be suggested for 'recieve': {s:?}"
        );
    }

    #[test]
    fn suggest_at_resolves_word_and_suggestions() {
        let _g = crate::testlock::serial();
        let sc = SpellChecker::new(DictVariant::EnUs).unwrap();
        let text = "Please recieve this.";
        let t = sc
            .suggest_at(text, 0, 9, None)
            .expect("cursor on a misspelling");
        assert_eq!(t.word, "recieve");
        assert_eq!((t.misspelling.start_col, t.misspelling.end_col), (7, 14));
        assert!(t.suggestions.iter().any(|w| w == "receive"));
        assert!(
            sc.suggest_at(text, 0, 2, None).is_none(),
            "'Please' is correct"
        );
    }

    // ── COMPLETED-WORD-LAG FIX: keyed spell verdicts ────────────────────────
    // `word_at` / `SpellVerdict::still_valid` / `keyed` — the "a stale verdict
    // can never paint on changed text" half of the eager+keyed mechanism.

    #[test]
    fn word_at_extracts_the_exact_span_text() {
        let m = Misspelling {
            line: 1,
            start_col: 3,
            end_col: 7,
        };
        assert_eq!(word_at("first\nhi helo there\nlast", &m), "helo");
    }

    #[test]
    fn word_at_is_char_index_aware_not_byte_index() {
        // "café " is 5 chars but 6 bytes (é is 2 bytes) — start_col/end_col are
        // CHAR columns, so word_at must walk chars, not bytes, or it would slice
        // into the middle of the multi-byte é and panic / mis-extract.
        let m = Misspelling {
            line: 0,
            start_col: 5,
            end_col: 9,
        };
        assert_eq!(word_at("café helo", &m), "helo");
    }

    #[test]
    fn word_at_degrades_to_empty_on_a_vanished_span() {
        // A line that no longer exists (buffer shrank) — never panics.
        let m = Misspelling {
            line: 5,
            start_col: 0,
            end_col: 4,
        };
        assert_eq!(word_at("only one line", &m), "");
    }

    #[test]
    fn keyed_verdicts_start_out_valid_against_their_own_text() {
        let text = "helo wrld";
        let spans = vec![
            Misspelling {
                line: 0,
                start_col: 0,
                end_col: 4,
            },
            Misspelling {
                line: 0,
                start_col: 5,
                end_col: 9,
            },
        ];
        let verdicts = keyed(text, spans);
        assert_eq!(verdicts.len(), 2);
        assert_eq!(verdicts[0].word, "helo");
        assert_eq!(verdicts[1].word, "wrld");
        assert!(verdicts.iter().all(|v| v.still_valid(text)));
    }

    /// THE CORE BUG FIX, at the pure-function seam: a verdict keyed to "helo"
    /// must go INVALID the instant the SAME span's text changes underneath it
    /// — even though the span's (line, start_col, end_col) columns are
    /// unchanged (an in-place correction, "helo" -> "hell", keeps the same
    /// 0..4 char range). This is exactly the completed-word-flash scenario: a
    /// verdict computed BEFORE an edit must never be read as still describing
    /// the text AFTER it.
    #[test]
    fn a_verdict_keyed_to_old_text_never_validates_against_new_text_at_the_same_span() {
        let old_text = "helo wrld";
        let verdict = keyed(
            old_text,
            vec![Misspelling {
                line: 0,
                start_col: 0,
                end_col: 4,
            }],
        )
        .into_iter()
        .next()
        .unwrap();
        assert!(verdict.still_valid(old_text));

        let new_text = "hell wrld";
        assert_ne!(word_at(new_text, &verdict.span), verdict.word);
        assert!(
            !verdict.still_valid(new_text),
            "a verdict keyed to the OLD word must not validate against the NEW text at the same span"
        );

        let new_text2 = "hello wrld";
        assert!(!verdict.still_valid(new_text2));
    }

    /// SEQUENCE test mirroring the live flow: scan text A, key against A,
    /// then re-key a DIFFERENT scan against text B — a verdict from the FIRST
    /// (stale) keying can never validate against B, but a verdict freshly
    /// keyed against B does. Models "edit completes a word, caret leaves it":
    /// the old cache must never paint; the fresh rescan must.
    #[test]
    fn a_fresh_rescan_validates_where_the_stale_one_does_not() {
        let good = stub(&["hello", "world"]);
        let text_a = "helo world"; // "helo" misspelled
        let stale = keyed(text_a, misspelled_spans(text_a, &good));
        assert_eq!(stale.len(), 1);
        assert!(stale[0].still_valid(text_a));

        let text_b = "hello world"; // now fully correct
        assert!(
            !stale[0].still_valid(text_b),
            "the STALE verdict (keyed to the old misspelling) must not paint on the corrected text"
        );

        let fresh = keyed(text_b, misspelled_spans(text_b, &good));
        assert!(
            fresh.is_empty(),
            "a fresh rescan of the corrected text must not flag anything: {fresh:?}"
        );
    }
}

/// RESCUE ROUND (2026-07): three adversarial/corpus probes reimplemented from a
/// stale `verify` branch (forked ~316 commits behind, never landed) against
/// main's CURRENT scoped-spell API — `SpellChecker::new` now takes a
/// [`DictVariant`], and `misspellings_for`'s `Str` arm additionally gates on
/// [`looks_like_prose_string`] (a string only squiggles when its content reads
/// as multi-word prose, not a single code-vocabulary token) — neither existed
/// on the branch's fork point. These probes exercise the REAL bundled
/// dictionary (not a stub predicate), so they're slower than the pure unit
/// tests above; kept as their own module for that reason.
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
}
