//! The DOCUMENT figures a summoned card shows, derived from the document and
//! nothing else.
//!
//! Three of a card's lines — the word-count readout, the frontmatter language,
//! and the through-doc percent — used to be computable only inside the render
//! pipeline, because the pipeline was the only holder of the shaped lines they
//! were summed over. That made the pipeline the one description of them, so the
//! semantic fold could not derive a card at all: it had to be HANDED one, and a
//! `--screenshot-app` capture (which has no pipeline of its own) announced no
//! card for a card its PNG plainly drew.
//!
//! Everything here is pure over `&str` plus the two buffer facts a caller
//! already has — is the buffer markdown, and where is the caret. So the
//! renderer and the semantic fold read ONE owner of each figure rather than two
//! descriptions that can drift.

use crate::frontmatter::Lang;

/// The three document figures, gathered together so a caller cannot fill one
/// from this owner and another by hand.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocFigures {
    /// The word-count readout line, empty when there is nothing to count.
    pub words: String,
    /// The document's frontmatter language, when it declares one.
    pub lang: Option<Lang>,
    /// How far through the document the caret sits, in whole percent.
    pub percent: u32,
}

impl DocFigures {
    /// THE derivation. `text` is the document as it stands, `is_markdown` the
    /// buffer's own kind, and `cursor_line`/`cursor_col` the caret's logical
    /// position in CHARACTERS (the same pair `Buffer::cursor_line_col` returns).
    pub fn of(text: &str, is_markdown: bool, cursor_line: usize, cursor_col: usize) -> Self {
        Self {
            words: words_readout(text, is_markdown),
            lang: frontmatter_lang(text),
            percent: through_doc_percent(text, cursor_line, cursor_col),
        }
    }
}

/// The document's declared language tag, or `None` when it carries no
/// frontmatter block or the block declares no recognized `lang:`.
pub fn frontmatter_lang(text: &str) -> Option<Lang> {
    crate::frontmatter::detect(text).and_then(|fm| fm.lang)
}

/// The MANUSCRIPT body of `text`: everything past a leading frontmatter block,
/// which is metadata rather than prose and so never counts toward a word count
/// or a reading time.
fn manuscript(text: &str) -> &str {
    match crate::frontmatter::detect(text) {
        Some(fm) => &text[fm.range.end..],
        None => text,
    }
}

/// The document's word count — whitespace-separated tokens over the manuscript
/// body.
///
/// The tokenizer is [`crate::markdown::word_count`], the single owner of what
/// awl calls a word. It splits on Unicode whitespace, which means a run of
/// unspaced CJK prose counts as ONE word however long it is: Japanese, Chinese
/// and Korean do not put spaces between words, so there is nothing for the
/// splitter to split on. That is a real limitation of the figure rather than a
/// property of this module — it is pinned by
/// [`tests::cjk_prose_counts_whitespace_runs_not_characters`] so it can never
/// change silently, and it is stated here because a reader of a Japanese
/// document's WORD COUNT line deserves to know what the number means.
pub fn word_count(text: &str) -> usize {
    crate::markdown::word_count(manuscript(text))
}

/// `Some((words, reading_minutes))` when the buffer is markdown and has at
/// least one word, else `None` — nothing is drawn and the sidecar reports null.
pub fn readout_figures(text: &str, is_markdown: bool) -> Option<(usize, usize)> {
    if !is_markdown {
        return None;
    }
    let words = word_count(text);
    if words == 0 {
        return None;
    }
    Some((words, crate::markdown::reading_time_min(words)))
}

/// The readout LINE, e.g. `"240 words · 2 min"`. Empty when there is nothing to
/// show (a non-markdown or wordless buffer).
pub fn words_readout(text: &str, is_markdown: bool) -> String {
    match readout_figures(text, is_markdown) {
        Some((w, m)) => {
            let unit = if w == 1 { "word" } else { "words" };
            format!("{w} {unit} · {m} min")
        }
        None => String::new(),
    }
}

/// How far through `text` the caret at `cursor_line`/`cursor_col` sits, in
/// whole percent of the document's CHARACTER length (inter-line newlines
/// included, exactly as they occupy a caret position). An empty document is 0.
pub fn through_doc_percent(text: &str, cursor_line: usize, cursor_col: usize) -> u32 {
    let denom = text.chars().count();
    if denom == 0 {
        return 0;
    }
    let mut offset = 0usize;
    for line in text.split('\n').take(cursor_line) {
        offset += line.chars().count() + 1; // + the line's trailing newline
    }
    offset += cursor_col;
    (((offset.min(denom) as f32) / denom as f32) * 100.0).round() as u32
}

/// THE fixture the drawn⇔announced figure laws share: one document, one caret,
/// and the figures HAND-DERIVED from them — never by calling the code those laws
/// test.
///
/// It lives here, beside the owner, so the pure `fold` law, the GPU pipeline law
/// and the capture-level History-preview law cannot drift apart on what the
/// document is or what it should read. Every number below is written out with
/// its arithmetic, because an oracle that recomputes the figure through
/// `DocFigures::of` would agree with any bug the derivation has.
#[cfg(test)]
pub(crate) mod fixture {
    /// A markdown document with a frontmatter language and two sibling
    /// sections. Nine logical lines (the trailing newline makes an empty ninth),
    /// 77 characters.
    pub const DOC: &str =
        "---\nlang: ja\n---\n# Alpha\nalpha one two\nalpha three four\n# Beta\nbeta five six\n";

    /// The logical line of `# Alpha`, whose section is lines 4–5.
    pub const FOLD_HEADING: usize = 3;

    /// [`DOC`] with `# Alpha` collapsed: lines 4 and 5 are gone, so the caret's
    /// line 7 sits at filtered line 5. 46 characters.
    pub const FOLDED: &str = "---\nlang: ja\n---\n# Alpha\n# Beta\nbeta five six\n";

    /// The caret, in `DOC`'s own line/column space — the start of `beta five
    /// six`, 63 characters in: 4 + 9 + 4 + 8 + 14 + 17 + 7 for the seven lines
    /// before it, each counted with its own newline.
    pub const CARET: (usize, usize) = (7, 0);

    /// The caret's line once `# Alpha` is folded: 7 minus the 2 hidden lines.
    pub const FOLDED_CARET_LINE: usize = 5;

    /// The DOCUMENT's readout. The frontmatter block is metadata, so the
    /// manuscript is `# Alpha / alpha one two / alpha three four / # Beta /
    /// beta five six` — 2 + 3 + 3 + 2 + 3 = 13 whitespace-separated tokens (the
    /// `#`s are tokens; that is what awl's tokenizer counts), and 13 words at
    /// 200 wpm rounds up to 1 minute.
    pub const WORDS: &str = "13 words · 1 min";
    /// [`WORDS`] as the `(words, reading_minutes)` pair the sidecar reports.
    pub const WORDS_PAIR: (usize, usize) = (13, 1);

    /// The DOCUMENT's through-doc percent: 63 characters into 77 is 81.81%,
    /// which rounds to 82.
    pub const PERCENT: u32 = 82;

    /// The VISIBLE-only readout, if the figures were derived from the folded
    /// text: the manuscript falls to `# Alpha / # Beta / beta five six`, which
    /// is 2 plus 2 plus 3 = 7 tokens. Asserted so the laws prove the two
    /// readings really differ; a fixture where they agreed would go green over
    /// the bug.
    pub const FOLDED_WORDS: &str = "7 words · 1 min";

    /// The VISIBLE-only percent: 32 characters into 46 is 69.56%, rounding to
    /// 70.
    pub const FOLDED_PERCENT: u32 = 70;

    /// A History preview's diff transcript — what the renderer is asked to shape
    /// while the picker previews an older version. It is not the user's
    /// document: no frontmatter, and its own six tokens have nothing to do with
    /// the manuscript's thirteen.
    pub const TRANSCRIPT: &str = "# beta five six\n\n~~alpha one~~ ==alpha two==\n";
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The frontmatter block is metadata, never manuscript: a `lang:`/`title:`
    /// line must not inflate the readout, and the language it declares must be
    /// the language the card shows.
    #[test]
    fn frontmatter_is_read_for_its_language_and_excluded_from_the_count() {
        let doc = "---\nlang: ja\ntitle: A Long Ceremonial Title\n---\none two three\n";
        assert_eq!(frontmatter_lang(doc), Some(Lang::Ja));
        assert_eq!(word_count(doc), 3, "the metadata block never counts");
        assert_eq!(words_readout(doc, true), "3 words · 1 min");
        // A document that merely OPENS with a thematic break is not frontmatter.
        let bare = "---\nprose here\n";
        assert_eq!(frontmatter_lang(bare), None);
        assert_eq!(word_count(bare), 3);
    }

    /// A non-markdown buffer draws no readout at all, and neither does an empty
    /// or whitespace-only one.
    #[test]
    fn a_wordless_or_non_markdown_buffer_shows_no_readout() {
        assert_eq!(words_readout("fn main() {}\n", false), "");
        assert_eq!(readout_figures("fn main() {}\n", false), None);
        assert_eq!(words_readout("", true), "");
        assert_eq!(words_readout("   \n\n \t\n", true), "");
        assert_eq!(words_readout("solo", true), "1 word · 1 min", "singular");
    }

    /// GRAPHEMES: a word is a run between whitespace, so a combining mark, a
    /// ZWJ emoji family and a regional-indicator flag each stay ONE word — they
    /// must never be split into their scalars, and a decomposed `é` must count
    /// the same as its precomposed twin.
    #[test]
    fn grapheme_clusters_count_as_one_word_each() {
        let decomposed = "e\u{301}te\u{301} 👨\u{200d}👩\u{200d}👧\u{200d}👦 🇯🇵";
        assert_eq!(word_count(decomposed), 3);
        assert_eq!(word_count("été 👨\u{200d}👩\u{200d}👧\u{200d}👦 🇯🇵"), 3);
        // A grapheme cluster spanning many scalars is still one token.
        assert_eq!(word_count("👨\u{200d}👩\u{200d}👧\u{200d}👦"), 1);
        // A zero-width joiner is NOT whitespace and must not split a word.
        assert_eq!(word_count("a\u{200d}b"), 1);
        // Unicode whitespace beyond ASCII does split: an ideographic space is
        // whitespace, so the two runs around it are two words.
        assert_eq!(word_count("日本\u{3000}語"), 2);
    }

    /// CJK, PINNED HONESTLY: Japanese, Chinese and Korean prose is written
    /// without inter-word spaces, so the whitespace tokenizer sees ONE token
    /// however many words a reader would count. This is what awl's WORD COUNT
    /// line actually reports today. The law exists so the figure cannot change
    /// under anyone silently — not to bless the number.
    #[test]
    fn cjk_prose_counts_whitespace_runs_not_characters() {
        // 11 Japanese characters, no spaces: one whitespace-run.
        let ja = "今日はいい天気ですね。";
        assert_eq!(ja.chars().count(), 11);
        assert_eq!(word_count(ja), 1);
        // Simplified Chinese, same shape.
        assert_eq!(word_count("我们今天去公园散步。"), 1);
        // Korean is spaced BETWEEN phrases, so it tokenizes the way the
        // whitespace rule expects — the divergence is script-specific, not
        // "CJK"-wide, and that is exactly the assumption worth pinning.
        assert_eq!(word_count("오늘 날씨가 좋네요"), 3);
        // MIXED: the Japanese run rides along with its neighbours as one token
        // each, so a bilingual paragraph is undercounted by the Japanese half.
        assert_eq!(word_count("The title is 今日はいい天気ですね。"), 4);
        // And the reading time follows the count, so a long Japanese document
        // reads as `1 min` no matter its length.
        let long_ja = "今日はいい天気ですね。".repeat(500);
        assert_eq!(word_count(&long_ja), 1);
        assert_eq!(readout_figures(&long_ja, true), Some((1, 1)));
    }

    /// The percent is over the document's CHARACTER length, newlines included,
    /// so the caret at the very end reads 100 and at the very start reads 0 —
    /// on a multi-byte document exactly as on an ASCII one.
    #[test]
    fn through_doc_percent_walks_zero_to_one_hundred_in_characters() {
        let doc = "abcd\nefgh\nijkl"; // 14 characters
        assert_eq!(through_doc_percent(doc, 0, 0), 0);
        assert_eq!(through_doc_percent(doc, 2, 4), 100);
        assert_eq!(through_doc_percent(doc, 1, 2), 50, "7 of 14");
        assert_eq!(through_doc_percent("", 0, 0), 0, "an empty document is 0");
        // Multi-BYTE, single-character glyphs count once each, not per byte:
        // the same shape in Japanese lands on the same percentages.
        let ja = "あいうえ\nかきくけ\nさしすせ";
        assert_eq!(through_doc_percent(ja, 0, 0), 0);
        assert_eq!(through_doc_percent(ja, 1, 2), 50);
        assert_eq!(through_doc_percent(ja, 2, 4), 100);
        // A caret past the end is clamped rather than overflowing the figure.
        assert_eq!(through_doc_percent(doc, 99, 99), 100);
    }

    /// The shared [`fixture`] is arithmetic done BY HAND, and the
    /// fold / preview laws lean on it as their oracle. This is where that
    /// arithmetic meets the owner: if the hand-derivation in `fixture`'s doc
    /// comments and this module's code ever disagree, one of them is wrong and
    /// every law built on the fixture is worth less than it looks.
    #[test]
    fn the_shared_fixture_arithmetic_matches_the_owner() {
        assert_eq!(fixture::DOC.chars().count(), 77);
        assert_eq!(fixture::FOLDED.chars().count(), 46);
        let doc = DocFigures::of(fixture::DOC, true, fixture::CARET.0, fixture::CARET.1);
        assert_eq!(doc.words, fixture::WORDS);
        assert_eq!(doc.percent, fixture::PERCENT);
        assert_eq!(doc.lang, Some(Lang::Ja));
        assert_eq!(
            readout_figures(fixture::DOC, true),
            Some(fixture::WORDS_PAIR)
        );
        let folded = DocFigures::of(fixture::FOLDED, true, fixture::FOLDED_CARET_LINE, 0);
        assert_eq!(folded.words, fixture::FOLDED_WORDS);
        assert_eq!(folded.percent, fixture::FOLDED_PERCENT);
        // The transcript is a THIRD reading again, so a preview law that reads it
        // by mistake cannot accidentally land on either of the other two.
        let transcript = DocFigures::of(fixture::TRANSCRIPT, true, 0, 0);
        assert_eq!(
            transcript.lang, None,
            "a diff transcript has no frontmatter"
        );
        assert_ne!(transcript.words, fixture::WORDS);
        assert_ne!(transcript.words, fixture::FOLDED_WORDS);
    }

    /// THE OWNER'S INPUT IS THE DOCUMENT, and only one door may say otherwise.
    ///
    /// Two production paths replace what the renderer shapes with something that
    /// is NOT the user's document: a fold drops the hidden lines, a History
    /// preview substitutes a diff transcript. Both now go through
    /// [`crate::render::ViewState::substitute_text`], which records the document
    /// behind them so these figures stay over it. A THIRD such path added later
    /// would have no test of its own yet, and would silently reintroduce exactly
    /// this bug — so the assignment itself is enumerated.
    ///
    /// Counted exactly per file (the `app/tests/source_audit.rs` shape) rather
    /// than filtered down to things that look like a `ViewState` — a site that
    /// never spells the type would dodge that filter, and the whole crate has
    /// only two such assignments in production, so the curated roster costs
    /// almost nothing to keep honest. The needle is assembled at runtime so this
    /// law's own source cannot match itself.
    #[test]
    fn only_the_substitution_door_replaces_a_view_states_text() {
        let needle = format!("{}{}", ".text", " = ");
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("src is readable") {
                let path = entry.expect("dir entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let rel = path
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if !rel.ends_with(".rs") || rel.ends_with("tests.rs") || rel.contains("/tests/") {
                    continue;
                }
                let source = std::fs::read_to_string(&path).expect("source is utf-8");
                // Test code inside a production file is free to build whatever
                // document it likes; the rule is about frames.
                let production = source
                    .split_once("#[cfg(test)]")
                    .map_or(source.as_str(), |(before, _)| before);
                let hits = production
                    .lines()
                    .filter(|line| !line.trim_start().starts_with("//"))
                    .filter(|line| line.contains(&needle))
                    .count();
                if hits > 0 {
                    counts.insert(rel, hits);
                }
            }
        }
        let expected: &[(&str, usize)] = &[
            // The frame NOTICE's own text — a different type entirely, and never
            // anything the renderer shapes as the document.
            ("app/frame/poll.rs", 2),
            // The bench scenarios BUILD a document into a fresh view (empty, then
            // the scenario's own text). Nothing is substituted for anything, so
            // the shaped text is the document and the figures come off it — the
            // `None` reading `substitute_text` leaves alone.
            ("render/benchsuite/scenarios.rs", 2),
        ];
        let expected: std::collections::BTreeMap<String, usize> = expected
            .iter()
            .map(|(f, n)| ((*f).to_string(), *n))
            .collect();
        assert_eq!(
            counts, expected,
            "a production site replaced a ViewState's text outside \
             `ViewState::substitute_text`. If it is a SUBSTITUTION (a fold, a \
             preview, anything that shapes less or other than the user's \
             document), route it through that door so the card's document \
             figures keep reading the document; if it is really CONSTRUCTING a \
             document, account for it here.",
        );
    }

    /// The gathered struct is exactly its three owners, so nothing can fill one
    /// figure from here and another by hand.
    #[test]
    fn the_gathered_figures_are_the_three_owners_verbatim() {
        let doc = "---\nlang: zh-Hans\n---\nsome prose here\n";
        let figures = DocFigures::of(doc, true, 3, 0);
        assert_eq!(figures.words, words_readout(doc, true));
        assert_eq!(figures.lang, frontmatter_lang(doc));
        assert_eq!(figures.percent, through_doc_percent(doc, 3, 0));
        assert_eq!(figures.lang, Some(Lang::ZhHans));
    }
}
