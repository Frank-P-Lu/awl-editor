//! Tests for the per-line CJK evidence scan and its retained aggregate.

use super::*;
use crate::frontmatter::DEFAULT_CJK_PRIORITY;

// --- the generated tables: invariants + a spot-check against hand-known
// characters, per CLAUDE.md's "spot-check a sample of generated entries
// against the code" tripwire (a generator can't be trusted blind). ------

#[test]
fn tables_are_the_expected_size_and_never_overlap() {
    assert_eq!(
        SIMPLIFIED_ONLY.len(),
        6368,
        "6368 bytes covers 0x3400..=0xFAFF"
    );
    assert_eq!(TRADITIONAL_ONLY.len(), 6368);
    for byte in 0..SIMPLIFIED_ONLY.len() {
        assert_eq!(
            SIMPLIFIED_ONLY[byte] & TRADITIONAL_ONLY[byte],
            0,
            "byte {byte}: no codepoint is both simplified-only and traditional-only"
        );
    }
}

#[test]
fn tables_match_their_frozen_generation_counts() {
    // Locks the regenerated tables to the exact counts `scripts/
    // regenerate-han-evidence.py` reported when they were last cut — a
    // silent corruption or a codec-table change on a future Python would
    // move this number, which is the point.
    let popcount = |t: &[u8]| t.iter().map(|b| b.count_ones()).sum::<u32>();
    assert_eq!(popcount(SIMPLIFIED_ONLY), 2191);
    assert_eq!(popcount(TRADITIONAL_ONLY), 6503);
}

#[test]
fn spot_check_known_simplified_and_traditional_characters() {
    // The item's own headline sentence characters, verified against the
    // codec tables directly (not just through `cjk_evidence`, so a bug in
    // the scan logic can't hide a bug in the table or vice versa).
    for c in "这简测试头开关门说话车".chars() {
        assert!(is_simplified_only(c), "{c:?} should be simplified-only");
        assert!(
            !is_traditional_only(c),
            "{c:?} is not also traditional-only"
        );
    }
    // Characters shared across all three legacy charsets (present in the
    // JP face too) must be neither.
    for c in "是体中文的一段文字骨直角与其内外站".chars() {
        assert!(
            !is_simplified_only(c),
            "{c:?} is shared, not simplified-only"
        );
        assert!(
            !is_traditional_only(c),
            "{c:?} is shared, not traditional-only"
        );
    }
    // A genuine traditional-only character (not shared with JIS X 0208).
    assert!(is_traditional_only('說'));
    assert!(!is_simplified_only('說'));
}

// --- cjk_evidence: the 5 rules, in priority order -----------------------

#[test]
fn kana_anywhere_wins_even_with_simplified_only_han_present() {
    // Rule 1 over rule 2 — the documented known miss's mechanism: kana
    // decides the WHOLE document even when Chinese-only evidence is also
    // present.
    assert_eq!(cjk_evidence("これは中文です这"), Some(Lang::Ja));
}

#[test]
fn a_single_trailing_kana_after_a_long_shared_han_run_still_decides() {
    // A document whose FIRST several CJK characters are unrepresentative
    // (shared, decide nothing on their own) must still resolve on the
    // ONE kana character that shows up only at the very end — a scan
    // that samples only the leading characters would miss it.
    let text = "是体中文的一段文字骨直角与其内外站の"; // trailing hiragana の
    assert_eq!(cjk_evidence(text), Some(Lang::Ja));
}

#[test]
fn simplified_only_character_resolves_zh_hans() {
    assert_eq!(cjk_evidence("这是测试"), Some(Lang::ZhHans));
}

#[test]
fn traditional_only_character_resolves_zh_hant() {
    assert_eq!(cjk_evidence("說話"), Some(Lang::ZhHant));
}

#[test]
fn simplified_only_wins_over_traditional_only_when_both_present() {
    // Rule 2 over rule 3 — an unusual document, but the priority order is
    // specified, not accidental.
    assert_eq!(cjk_evidence("这說"), Some(Lang::ZhHans));
}

#[test]
fn hangul_resolves_ko_only_when_nothing_stronger_is_present() {
    assert_eq!(cjk_evidence("한국어"), Some(Lang::Ko));
}

#[test]
fn a_single_trailing_hangul_after_a_long_shared_han_run_still_decides() {
    let text = "是体中文的一段文字骨直角与其内外站한"; // trailing hangul syllable
    assert_eq!(cjk_evidence(text), Some(Lang::Ko));
}

#[test]
fn simplified_only_wins_over_hangul_when_both_present() {
    // Rule 2 over rule 4.
    assert_eq!(cjk_evidence("한국어这"), Some(Lang::ZhHans));
}

#[test]
fn shared_characters_only_is_undecided() {
    // Every character here is shared across gb2312/big5/shift_jis (see
    // the spot-check above) — no kana, no hangul, no exclusive character.
    assert_eq!(cjk_evidence("是体中文的一段文字骨直角与其内外站"), None);
}

#[test]
fn pure_latin_and_empty_are_undecided() {
    assert_eq!(cjk_evidence("nothing but english here"), None);
    assert_eq!(cjk_evidence(""), None);
}

#[test]
fn bopomofo_alone_is_undecided_by_this_tier() {
    // Bopomofo already has its own unambiguous natural mapping
    // (`Script::natural_font_id`) that never reaches this tier at all in
    // the render ladder; this tier itself deliberately does not cover it
    // (module doc), so a Bopomofo-only document reports no evidence here.
    assert_eq!(cjk_evidence("ㄍㄨㄛˊ"), None);
}

/// THE HEADLINE REGRESSION: the item's own reported sentence — untagged,
/// no kana — must resolve `ZhHans` from the text alone.
#[test]
fn the_reported_untagged_chinese_sentence_resolves_zh_hans() {
    let sentence = "这是简体中文的一段测试文字骨头直角与其内外开关门说话车站";
    assert_eq!(cjk_evidence(sentence), Some(Lang::ZhHans));
}

/// A UNREPRESENTATIVE FIRST CJK CHARACTER must not short-circuit the
/// scan: many shared characters lead, and the one decisive character
/// sits at the very END. A scan that samples only the first CJK run (or
/// stops at the first Han character found) would report `None` here.
#[test]
fn a_decisive_character_late_in_a_long_shared_run_still_decides() {
    let text = "是体中文的一段文字骨直角与其内外站这"; // shared… shared… 这 (simplified-only) LAST
    assert_eq!(cjk_evidence(text), Some(Lang::ZhHans));

    // The traditional mirror, same shape.
    let text2 = "是体中文的一段文字骨直角与其内外站說"; // shared…說 (traditional-only) LAST
    assert_eq!(cjk_evidence(text2), Some(Lang::ZhHant));
}

// --- effective_cjk_priority + resolve_font_id, end to end ---------------

#[test]
fn effective_priority_passes_through_unchanged_with_no_evidence() {
    let priority = [Lang::ZhHant, Lang::Ja, Lang::ZhHans, Lang::Ko];
    assert_eq!(effective_cjk_priority(None, &priority), priority.to_vec());
}

#[test]
fn effective_priority_promotes_decisive_evidence_to_front() {
    let priority = [Lang::Ja, Lang::ZhHans, Lang::ZhHant, Lang::Ko];
    assert_eq!(
        effective_cjk_priority(Some(Lang::ZhHant), &priority),
        vec![Lang::ZhHant, Lang::Ja, Lang::ZhHans, Lang::Ko]
    );
}

#[test]
fn explicit_ja_first_ladder_does_not_override_a_simplified_only_hit() {
    // The item's own headline law, at the pure-function seam: even the
    // DEFAULT ladder (ja-first — the exact config the reporter had) must
    // not win once the text itself carries simplified-only evidence.
    let sentence = "这是简体中文的一段测试文字骨头直角与其内外开关门说话车站";
    let evidence = cjk_evidence(sentence);
    let effective = effective_cjk_priority(evidence, &DEFAULT_CJK_PRIORITY);
    let id = super::super::resolve_font_id(None, Some(Script::Han), &effective);
    assert_eq!(
        id,
        crate::theme::FontId::ZhHans,
        "an untagged, ja-first-configured Han run with simplified-only evidence \
         must resolve the Simplified-Chinese face"
    );

    // NON-VACUITY: restoring the OLD behavior (reading `cjk_priority.
    // first()` directly, with no evidence folded in at all) must go red —
    // proving this law actually exercises the fix rather than passing on
    // its own account. `doc_lang_for` is exactly that old, un-evidenced
    // path.
    let old_lang = super::super::doc_lang_for(Script::Han, &DEFAULT_CJK_PRIORITY);
    assert_eq!(
        old_lang,
        Lang::Ja,
        "sanity: the un-evidenced path really did default to Japanese"
    );
    assert_ne!(
        old_lang,
        Lang::ZhHans,
        "mutation proof: without the evidence tier this reads Japanese, \
         not Simplified Chinese — the exact reported bug"
    );
}

#[test]
fn a_kanji_only_japanese_title_stays_japanese() {
    // The item's own worked non-miss: no simplified-only character in a
    // plain kanji title, so evidence is undecided and the (default,
    // ja-first) setting still wins — unaffected by this tier.
    let title = "日本語学校"; // pure Han, all characters shared/JP-common
    assert_eq!(cjk_evidence(title), None);
    let effective = effective_cjk_priority(cjk_evidence(title), &DEFAULT_CJK_PRIORITY);
    assert_eq!(
        super::super::resolve_font_id(None, Some(Script::Han), &effective),
        crate::theme::FontId::Ja
    );
}

/// THE DOCUMENTED KNOWN MISS: a note mixing a Japanese paragraph and a
/// Chinese paragraph resolves Japanese AS A WHOLE (kana anywhere wins the
/// document), leaving the Chinese paragraph in the same patchwork the
/// item reports for a pure-Chinese note. This is not a regression this
/// round introduces — it is the documented boundary of a document-scoped
/// (not per-paragraph) evidence tier, asserted here so it can never be
/// silently "fixed" by accident without a conscious decision (the item
/// explicitly defers per-paragraph scoping).
#[test]
fn known_miss_mixed_japanese_and_chinese_paragraphs_read_as_japanese() {
    let mixed =
        "これは日本語の段落です。\n\n这是简体中文的一段测试文字骨头直角与其内外开关门说话车站\n";
    assert_eq!(
        cjk_evidence(mixed),
        Some(Lang::Ja),
        "kana anywhere wins the whole document, per rule 1 — the Chinese \
         paragraph does not get its own answer"
    );
}

#[test]
fn frontmatter_tag_still_wins_over_decisive_evidence() {
    // "A tagged document is unchanged: the tag still wins." Ladder step
    // (a) is evaluated by `resolve_font_id` BEFORE evidence is ever
    // consulted, so a doc tagged `ja` with simplified-only text in it
    // still resolves `Ja` — evidence only ever fills the gap a missing/
    // incompatible tag leaves.
    let sentence = "这是简体中文的一段测试文字骨头直角与其内外开关门说话车站";
    let effective = effective_cjk_priority(cjk_evidence(sentence), &DEFAULT_CJK_PRIORITY);
    assert_eq!(
        super::super::resolve_font_id(Some(Lang::Ja), Some(Script::Han), &effective),
        crate::theme::FontId::Ja,
        "the doc tag wins outright; the ZhHans evidence is never reached"
    );
}

// --- `line_evidence` / `EvidenceCounts`: the retained-projection half ---

/// WORK-COUNT EQUIVALENCE: for every line of a multi-line document, folding
/// each line's own [`line_evidence`] into one [`EvidenceCounts`] and
/// resolving it must equal a fresh whole-document [`cjk_evidence`] scan —
/// the exact property [`crate::render::rects::HanEvidenceProjection`]
/// relies on to answer the document question from per-line data alone.
fn counts_for(lines: &[&str]) -> EvidenceCounts {
    let mut counts = EvidenceCounts::default();
    for &line in lines {
        counts.add(line_evidence(line));
    }
    counts
}

#[test]
fn per_line_aggregate_matches_a_fresh_whole_document_scan() {
    let cases: &[&str] = &[
        "nothing but english here",
        "这是简体中文的一段测试文字骨头直角与其内外开关门说话车站",
        "說話",
        "한국어",
        "これは中文です这",
        "是体中文的一段文字骨直角与其内外站",
        "是体中文的一段文字骨直角与其内外站の",
        "是体中文的一段文字骨直角与其内外站한",
        "一\n二\n三这\n四",   // decisive character on a LATER line only
        "한\n中\n说\n한国어", // multiple lines, multiple signals
    ];
    for text in cases {
        let lines: Vec<&str> = text.split('\n').collect();
        let whole_doc = cjk_evidence(text);
        let retained = counts_for(&lines).resolve();
        assert_eq!(
            retained, whole_doc,
            "per-line aggregate must match a fresh whole-document scan for {text:?}"
        );
    }
}

/// THE ITEM'S OWN RETRACTION LAW: deleting a document's last decisive
/// line must retract that line's evidence from the aggregate, not leave
/// a stale positive count behind. Mirrors editing away the document's
/// only simplified-only character and checking `cjk_evidence` on the
/// EDITED text resolves `None` again.
#[test]
fn removing_the_only_decisive_line_retracts_its_evidence() {
    let mut counts = EvidenceCounts::default();
    let decisive = line_evidence("这"); // simplified-only
    let plain = line_evidence("shared prose, no CJK at all");
    counts.add(decisive);
    counts.add(plain);
    assert_eq!(counts.resolve(), Some(Lang::ZhHans));

    // The line is edited away (or deleted) — its OLD contribution is
    // removed before the new (empty-of-evidence) text is added.
    counts.remove(decisive);
    assert_eq!(
        counts.resolve(),
        None,
        "the aggregate must fall back to None once the only decisive line is gone, \
         exactly like a fresh cjk_evidence scan of the edited text"
    );

    // NON-VACUITY: a design that only ever ADDED (never retracted) would
    // leave this `Some(ZhHans)` forever — this is the exact regression
    // the retraction call guards against.
    let mut leaky = EvidenceCounts::default();
    leaky.add(decisive);
    leaky.add(plain);
    assert_ne!(
        leaky.resolve(),
        None,
        "sanity: the counts really do carry the decisive line before it's removed"
    );
}

/// A document with TWO lines carrying the SAME signal must keep resolving
/// that signal after only ONE of them is retracted (a reference count,
/// not a boolean latch).
#[test]
fn one_of_two_decisive_lines_can_retract_while_the_other_still_decides() {
    let mut counts = EvidenceCounts::default();
    let a = line_evidence("这"); // simplified-only
    let b = line_evidence("测"); // also simplified-only
    counts.add(a);
    counts.add(b);
    assert_eq!(counts.resolve(), Some(Lang::ZhHans));
    counts.remove(a);
    assert_eq!(
        counts.resolve(),
        Some(Lang::ZhHans),
        "the second decisive line still stands after the first is retracted"
    );
    counts.remove(b);
    assert_eq!(counts.resolve(), None);
}

#[test]
fn line_evidence_does_not_short_circuit_on_kana_unlike_cjk_evidence() {
    // A single line mixing kana AND a simplified-only character must
    // report BOTH flags (needed for correct retraction bookkeeping),
    // even though `cjk_evidence` itself would answer `Ja` and never learn
    // about the simplified-only character at all.
    let ev = line_evidence("これは中文です这");
    assert!(ev.kana, "kana must be recorded");
    assert!(
        ev.simplified_only,
        "the simplified-only character must ALSO be recorded, unlike the \
         short-circuiting whole-document scan"
    );
}
