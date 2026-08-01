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
    let text = include_str!("../syntax/rust.rs");
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
