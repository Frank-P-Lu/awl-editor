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
