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
use crate::script::{Script, classify_char};

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

/// The unit the readout counts in — the label a script's own reader would
/// expect. `Words` for a script that spaces its words (Latin, Korean); on a
/// script with no spaces to split on (Japanese, Chinese) the readout switches
/// to `Characters` rather than claim a word count that script doesn't have.
/// See [`dominant_unit`] for how a document picks one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountUnit {
    Words,
    Characters,
}

/// Characters-per-minute silent-reading pace for a script this readout counts
/// in [`CountUnit::Characters`] (Kana/Han/Bopomofo — see [`is_unspaced`]).
/// Deliberately not [`crate::markdown::READING_WPM`]: that figure is a
/// WORDS-per-minute rate, and applying it to a raw character count reads two
/// to three times too slow, because one CJK "word" of meaning is one to a
/// few characters while an English word averages ~5 — the same document
/// would read as slower prose purely because its tokens got smaller.
/// Published silent-reading-rate studies for Japanese
/// cluster roughly 400-600 characters/minute; 500 is this figure's own round
/// midpoint of that span, picked the same way 200 is the round conventional
/// figure for English silent prose (itself a midpoint of a cited 200-250 wpm
/// range, not a single study's number). Lives here, beside [`CountUnit`],
/// rather than in `markdown::spans` next to `READING_WPM` — a second
/// constant beside that one would leave the pace decision outside the unit
/// it belongs to, so a future caller could compute a words-count reading
/// time with the CJK pace (or vice versa) by picking the wrong constant by
/// hand. [`CountUnit::pace_per_minute`] is the one door.
const CJK_CHARS_PER_MINUTE: usize = 500;

impl CountUnit {
    /// The singular/plural label for `n`, e.g. `"word"` / `"words"`.
    fn label(self, n: usize) -> &'static str {
        match (self, n == 1) {
            (CountUnit::Words, true) => "word",
            (CountUnit::Words, false) => "words",
            (CountUnit::Characters, true) => "character",
            (CountUnit::Characters, false) => "characters",
        }
    }

    /// The sidecar's own tag for this unit — `"words"` / `"characters"`,
    /// always plural regardless of `n` (the JSON reports a fact about the
    /// document, not a sentence to read aloud).
    pub fn tag(self) -> &'static str {
        match self {
            CountUnit::Words => "words",
            CountUnit::Characters => "characters",
        }
    }

    /// The silent-reading pace, in units-per-minute, this unit's own count
    /// should be measured against — a property OF the unit rather than a
    /// second constant a caller must remember to pick to match it. Whichever
    /// `CountUnit` [`dominant_unit`] resolves for a document supplies that
    /// same document's pace, so the label on screen and the minutes beside it
    /// can never come from two different rates.
    pub fn pace_per_minute(self) -> usize {
        match self {
            CountUnit::Words => crate::markdown::READING_WPM,
            CountUnit::Characters => CJK_CHARS_PER_MINUTE,
        }
    }
}

/// Kana, Han and Bopomofo carry no inter-word spaces; Hangul does (Korean
/// spaces its words), so it is deliberately excluded — the divergence this
/// figure exists to fix is script-specific, not "CJK"-wide.
fn is_unspaced(script: Script) -> bool {
    matches!(script, Script::Kana | Script::Han | Script::Bopomofo)
}

/// The manuscript's dominant writing system, decided by a STRICT MAJORITY of
/// its own characters that carry an unspaced script (Kana/Han/Bopomofo) — more
/// than half the body's characters, a tie (including an empty document) reads
/// as `Words`, the same Latin/spaced floor [`crate::script::resolve_font_id`]
/// falls back to.
///
/// Deliberately NOT the frontmatter `lang:` tag, and not a majority of
/// COUNTED TOKENS either — both were considered and rejected:
///   - **Tokens** over-weights a short CJK insertion, because
///     [`count_tokens`] below turns each ideograph into its OWN token: a
///     three-word English sentence carrying one ten-character Japanese phrase
///     already has more ideograph-tokens than word-tokens, so a token
///     majority would call the whole document "characters" over one
///     inserted phrase — the opposite of "dominant".
///   - **Frontmatter `lang:`** is a DECLARED intent, not a report of what is
///     actually written — `card::figures::fixture::DOC` is the proof: it
///     tags `lang: ja` on a body that is plain, space-separated English (a
///     scaffold that predates this figure), and a reader of its WORD COUNT
///     line wants a count of the words on the page, not a label chosen by a
///     tag that has nothing to do with the visible prose.
///
/// A character majority answers the only question this figure actually
/// asks — "does this document's prose have words to count, or not" — from
/// the prose itself.
fn dominant_unit(body: &str) -> CountUnit {
    let mut unspaced = 0usize;
    let mut total = 0usize;
    for c in body.chars() {
        total += 1;
        if matches!(classify_char(c), Some(s) if is_unspaced(s)) {
            unspaced += 1;
        }
    }
    if unspaced * 2 > total {
        CountUnit::Characters
    } else {
        CountUnit::Words
    }
}

/// The document's token count over the manuscript body: a whitespace-
/// separated run counts as one token exactly as [`crate::markdown::word_count`]
/// always has, EXCEPT that an unspaced-script character (Kana/Han/Bopomofo —
/// see [`is_unspaced`]) is never folded into that run; it counts as a token of
/// its own, because it is one — an ideograph carries a full word or morpheme's
/// worth of meaning with no space to mark where it ends. Two consequences
/// worth naming: a run of unspaced CJK prose no longer collapses to ONE token
/// however long it runs on (the defect this figure exists to fix), and a
/// mixed run like `今日は…` no longer rides along as a single token behind its
/// Latin neighbours — it contributes one token per ideograph, so a bilingual
/// paragraph is no longer undercounted by its CJK half.
///
/// Everything this module already held still holds: a grapheme cluster (ZWJ
/// family, regional-indicator flag, decomposed `é`) contains no
/// Kana/Han/Bopomofo scalar, so it never gets split — it rides through as one
/// buffered non-ideographic token exactly as before. An ideographic space
/// (`U+3000`) is still Unicode whitespace, so `split_whitespace` still splits
/// on it before this function ever sees an ideograph on either side.
fn count_tokens(body: &str) -> usize {
    let mut count = 0usize;
    for run in body.split_whitespace() {
        let mut buffering = false;
        for c in run.chars() {
            if matches!(classify_char(c), Some(s) if is_unspaced(s)) {
                if buffering {
                    count += 1; // flush the non-ideographic token in progress
                    buffering = false;
                }
                count += 1; // the ideograph, its own token
            } else {
                buffering = true;
            }
        }
        if buffering {
            count += 1;
        }
    }
    count
}

/// The document's word/character count over the manuscript body — see
/// [`count_tokens`] for what counts as a token. Pinned regression floor:
/// [`tests::cjk_prose_counts_ideographs_as_tokens`].
pub fn word_count(text: &str) -> usize {
    count_tokens(manuscript(text))
}

/// `Some((count, reading_minutes, unit))` when the buffer is markdown and has
/// at least one token, else `None` — nothing is drawn and the sidecar reports
/// null. `unit` is [`dominant_unit`]'s call over the manuscript body — the ONE
/// owner both the drawn readout ([`words_readout`]) and the sidecar
/// (`capture::sidecar::readout_json`/`hud_json`) take their label from, so
/// neither can independently mislabel the other's number. `reading_minutes`
/// is measured at THAT SAME unit's own pace
/// ([`CountUnit::pace_per_minute`]) — a mixed document takes its dominant
/// script's pace outright rather than interpolating a blend of two rates,
/// exactly matching how the unit label already resolves (one majority
/// decides both the noun and the pace, so one rule explains both and the
/// same no-flicker guarantee `dominant_unit` already gives the label covers
/// the pace for free).
pub fn readout_figures(text: &str, is_markdown: bool) -> Option<(usize, usize, CountUnit)> {
    if !is_markdown {
        return None;
    }
    let words = word_count(text);
    if words == 0 {
        return None;
    }
    let unit = dominant_unit(manuscript(text));
    let minutes = crate::markdown::reading_time_min(words, unit.pace_per_minute());
    Some((words, minutes, unit))
}

/// The readout LINE, e.g. `"240 words · 2 min"` or `"5500 characters · 28
/// min"`. Empty when there is nothing to show (a non-markdown or wordless
/// buffer).
pub fn words_readout(text: &str, is_markdown: bool) -> String {
    match readout_figures(text, is_markdown) {
        Some((w, m, unit)) => {
            format!("{w} {} · {m} min", unit.label(w))
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

/// THE fixture the drawn⇔announced figure laws share (`fold::tests`,
/// `render::tests::folds`, `capture::tests::pickers_faceted`) — split into
/// its own file so this one stays under the production line ceiling; the
/// module path (`crate::card::figures::fixture`) is unchanged.
#[cfg(test)]
pub(crate) mod fixture;

#[cfg(test)]
mod tests;
