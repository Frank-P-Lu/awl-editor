//! `card::figures` unit tests. A sibling of `mod.rs`, named `tests.rs` so
//! it stays exempt from the production line ceiling
//! (`scripts/code-health.py::production`) however large the suite grows.

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
    // Unicode whitespace beyond ASCII does still split: an ideographic
    // space is whitespace, so the two runs around it never fuse into one.
    // The COUNT is 3, not 2, because each side is now itself ideographic
    // (2 Han characters, then 1) — ideographs count individually on
    // either side of the split; what the space still guarantees is that
    // "本" and "語" are never glued together into a single run.
    assert_eq!(word_count("日本\u{3000}語"), 3);
}

/// CJK: Japanese and Chinese prose is written without inter-word spaces,
/// so an ideograph counts as a token of its own rather than riding along
/// inside a whitespace run — a run of unspaced CJK prose no longer
/// collapses to ONE token however long it runs on. Korean spaces its own
/// words and is untouched: the divergence is script-specific, not
/// "CJK"-wide, and that is exactly the assumption worth pinning.
#[test]
fn cjk_prose_counts_ideographs_as_tokens() {
    // 11 Japanese characters: 10 Han/Kana ideographs (each its own token)
    // plus the trailing `。`, buffered and flushed as its own token — 11.
    let ja = "今日はいい天気ですね。";
    assert_eq!(ja.chars().count(), 11);
    assert_eq!(word_count(ja), 11);
    assert_eq!(
        words_readout(ja, true),
        "11 characters · 1 min",
        "an unspaced-script-dominant document reads in CHARACTERS, not words"
    );
    // Simplified Chinese, same shape: 9 Han + 1 trailing `。` = 10.
    assert_eq!(word_count("我们今天去公园散步。"), 10);
    // Korean is spaced BETWEEN phrases and carries no ideographic
    // (Kana/Han/Bopomofo) characters at all — Hangul is deliberately
    // excluded from `is_unspaced` — so it tokenizes exactly as before.
    assert_eq!(word_count("오늘 날씨가 좋네요"), 3);
    assert_eq!(words_readout("오늘 날씨가 좋네요", true), "3 words · 1 min");
    // MIXED: the Japanese run no longer rides along as ONE token behind
    // its Latin neighbours; it contributes one token per ideograph (10)
    // plus the trailing `。` (1), on top of "The"/"title"/"is" (3) = 14,
    // not 4. The document's characters are still majority-Latin (13
    // ASCII incl. spaces vs 11 CJK), so the LABEL stays "words" — only
    // the number moves.
    let mixed = "The title is 今日はいい天気ですね。";
    assert_eq!(word_count(mixed), 14);
    assert_eq!(words_readout(mixed, true), "14 words · 1 min");
    // And the reading time now follows a count that actually grows with
    // the document: a long Japanese document no longer reads `1 min`
    // regardless of length — the exact defect this item measured.
    let long_ja = "今日はいい天気ですね。".repeat(500);
    assert_eq!(long_ja.chars().count(), 5_500);
    assert_eq!(word_count(&long_ja), 5_500, "5,500 characters, not 1 word");
    assert_eq!(
        readout_figures(&long_ja, true),
        Some((5_500, 28, CountUnit::Characters)),
        "ceil(5500 / 200) = 28, not the old flat 1 min"
    );
}

/// DOMINANCE FLIPS ACROSS A REAL EDIT, at an EXACT and PINNED crossing —
/// the axis named in the brief as the one worth sweeping deliberately: a
/// document authored in Latin, having Japanese sentences typed into it one
/// at a time, must change its readout's UNIT at the moment (and only the
/// moment) unspaced-script characters become a strict majority, never
/// before and never after.
///
/// The construction is exact arithmetic, not measured: `"word "` (5
/// ASCII characters, itself one whitespace-separated token) repeated 10
/// times is a fixed 50-character Latin floor; appending `M` bare Han
/// characters (each an ideograph — no separator needed, since an
/// ideograph counts as a token the instant it's seen) brings the body to
/// `50 + M` characters, `M` of them unspaced. `dominant_unit` reads
/// `Characters` iff `M*2 > 50+M`, i.e. `M > 50` — so `M=50` is the last
/// tie (reads `Words`, the documented tie rule) and `M=51` is the first
/// real majority.
#[test]
fn dominant_unit_flips_at_the_exact_character_majority_crossing() {
    let latin_floor = "word ".repeat(10); // 50 chars, 10 Latin tokens
    for m in 0..=50 {
        let body = format!("{latin_floor}{}", "字".repeat(m));
        assert_eq!(
            dominant_unit(&body),
            CountUnit::Words,
            "M={m}: a tie or a Latin majority must read Words"
        );
    }
    for m in 51..=60 {
        let body = format!("{latin_floor}{}", "字".repeat(m));
        assert_eq!(
            dominant_unit(&body),
            CountUnit::Characters,
            "M={m}: past the crossing, must read Characters"
        );
    }
    // The crossing is a single edit — appending ONE more ideograph at
    // M=50 → M=51 is the whole story, pinned as its own assertion so a
    // reader does not have to infer it from the sweep's endpoints.
    let at_tie = format!("{latin_floor}{}", "字".repeat(50));
    let just_past = format!("{latin_floor}{}", "字".repeat(51));
    assert_eq!(dominant_unit(&at_tie), CountUnit::Words);
    assert_eq!(dominant_unit(&just_past), CountUnit::Characters);
}

/// NO FLICKER: once a document is DECISIVELY dominant one way, a single
/// character typed at the OTHER script never flips the label back — the
/// crossing above is real (a genuine multi-character edit that actually
/// changes the majority); this is the axis the brief warns is easy to get
/// wrong by accident (an off-by-one in the tie math would make single
/// keystrokes near a decisive majority jitter the unit on every frame).
#[test]
fn dominant_unit_does_not_flicker_on_a_single_character_at_a_decisive_majority() {
    // Decisively CJK (90 ideographs vs 10 Latin chars): one trailing
    // Latin letter, then two, then three, must not tip it back to Words.
    let mut cjk_heavy = "字".repeat(90);
    assert_eq!(dominant_unit(&cjk_heavy), CountUnit::Characters);
    for _ in 0..3 {
        cjk_heavy.push('a');
        assert_eq!(
            dominant_unit(&cjk_heavy),
            CountUnit::Characters,
            "a single Latin character must not flip a decisive CJK majority"
        );
    }
    // Decisively Latin (90 Latin chars vs 10 ideographs): one trailing
    // ideograph must not tip it to Characters.
    let mut latin_heavy = "a".repeat(90);
    assert_eq!(dominant_unit(&latin_heavy), CountUnit::Words);
    for _ in 0..3 {
        latin_heavy.push('字');
        assert_eq!(
            dominant_unit(&latin_heavy),
            CountUnit::Words,
            "a single ideograph must not flip a decisive Latin majority"
        );
    }
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
