use super::*;

#[test]
fn detects_lang_tag() {
    let fm = detect("---\nlang: ja\n---\n# Title\n").expect("valid frontmatter");
    assert_eq!(fm.lang, Some(Lang::Ja));
    assert_eq!(
        &"---\nlang: ja\n---\n# Title\n"[fm.range.clone()],
        "---\nlang: ja\n---\n"
    );
}

#[test]
fn all_five_bcp47_tags_parse() {
    for (tag, want) in [
        ("en", Lang::En),
        ("ja", Lang::Ja),
        ("zh-Hans", Lang::ZhHans),
        ("zh-Hant", Lang::ZhHant),
        ("ko", Lang::Ko),
    ] {
        let doc = format!("---\nlang: {tag}\n---\n");
        let fm = detect(&doc).unwrap_or_else(|| panic!("{tag} should parse: {doc:?}"));
        assert_eq!(fm.lang, Some(want), "tag {tag}");
        assert_eq!(Lang::parse(want.code()), Some(want), "code() round-trips");
    }
}

#[test]
fn tag_is_case_insensitive_on_the_key_and_value() {
    let fm = detect("---\nLang: JA\n---\n").expect("valid");
    assert_eq!(fm.lang, Some(Lang::Ja));
}

#[test]
fn unknown_lang_value_is_inert_not_a_crash() {
    let fm = detect("---\nlang: klingon\n---\nbody\n").expect("still valid frontmatter");
    assert_eq!(fm.lang, None, "unrecognized tag is simply not parsed");
}

#[test]
fn unknown_keys_are_inert() {
    let fm = detect("---\ntitle: My Doc\ndate: 2026-01-01\nlang: ko\n---\n").expect("valid");
    assert_eq!(
        fm.lang,
        Some(Lang::Ko),
        "lang still extracted alongside unknown keys"
    );
}

#[test]
fn no_lang_key_at_all_is_fine() {
    let fm = detect("---\ntitle: x\n---\n").expect("valid, no lang");
    assert_eq!(fm.lang, None);
}

#[test]
fn only_at_byte_zero_never_mid_document() {
    // A `---` block anywhere but the very start of the text is never a
    // frontmatter block (mid-doc it is only ever a thematic break / setext
    // underline, handled entirely by `markdown::spans`).
    assert!(
        detect("\n---\nlang: ja\n---\n").is_none(),
        "leading blank line disqualifies it"
    );
    assert!(detect("prelude\n---\nlang: ja\n---\n").is_none());
}

#[test]
fn no_closing_delimiter_is_not_frontmatter() {
    assert!(
        detect("---\nlang: ja\n").is_none(),
        "unterminated opener is not a block"
    );
    assert!(
        detect("---\n").is_none(),
        "a bare opener with nothing after it"
    );
    assert!(
        detect("---").is_none(),
        "a single dash-rule line, no newline at all"
    );
}

#[test]
fn plain_prose_starting_with_a_rule_stays_untouched() {
    // The classic false-positive risk: a document that legitimately opens
    // with a thematic break, has ordinary prose, and later has an unrelated
    // second break. None of that prose is "key: value" shaped, so `detect`
    // must bail entirely rather than swallowing the prose as metadata.
    let doc = "---\n\nSome opening prose about nothing in particular.\n\n---\n\nMore prose.\n";
    assert!(
        detect(doc).is_none(),
        "a non-kv line between the dashes bails: {doc:?}"
    );
}

#[test]
fn blank_lines_inside_the_block_are_tolerated() {
    let fm = detect("---\nlang: ja\n\ntitle: x\n---\n").expect("blank line inside is fine");
    assert_eq!(fm.lang, Some(Lang::Ja));
}

#[test]
fn crlf_line_endings_are_handled() {
    let fm = detect("---\r\nlang: ja\r\n---\r\nbody\r\n").expect("CRLF frontmatter parses");
    assert_eq!(fm.lang, Some(Lang::Ja));
}

#[test]
fn empty_frontmatter_block_is_valid_with_no_lang() {
    let fm = detect("---\n---\n").expect("an immediately-closed block is valid");
    assert_eq!(fm.lang, None);
    assert_eq!(fm.range, 0..8);
}

#[test]
fn all_langs_round_trip_through_parse_and_code() {
    // ALL_LANGS is the law-sweep list every future `Lang` variant must join
    // (a no-wildcard `match` on `Lang` elsewhere is the real compile-time
    // guard; this pins that the LIST itself stays exhaustive by eye).
    assert_eq!(ALL_LANGS.len(), 5);
    for l in ALL_LANGS {
        assert_eq!(Lang::parse(l.code()), Some(l), "{l:?} must round-trip");
    }
    assert_eq!(
        DEFAULT_CJK_PRIORITY,
        [Lang::Ja, Lang::ZhHans, Lang::ZhHant, Lang::Ko]
    );
}

#[test]
fn never_panics_on_garbage() {
    // A grab-bag of malformed/edge inputs must never panic.
    for doc in [
        "",
        "-",
        "--",
        "----",
        "---x",
        "---\n:::\n---\n",
        "---\n:\n---\n",
    ] {
        let _ = detect(doc);
    }
}

#[test]
fn every_lang_label_round_trips_through_from_label() {
    for l in ALL_LANGS {
        assert_eq!(
            Lang::from_label(l.label()),
            Some(l),
            "{l:?} must round-trip"
        );
    }
    // Case-insensitive, like DictVariant::from_label.
    assert_eq!(Lang::from_label("japanese"), Some(Lang::Ja));
    assert_eq!(Lang::from_label("nonsense"), None);
}

#[test]
fn pack_unpack_priority_round_trips() {
    for perm in [
        DEFAULT_CJK_PRIORITY,
        [Lang::Ko, Lang::ZhHant, Lang::ZhHans, Lang::Ja],
        [Lang::ZhHans, Lang::Ja, Lang::Ko, Lang::ZhHant],
    ] {
        assert_eq!(unpack_priority(pack_priority(perm)), perm.to_vec());
    }
}

#[test]
fn normalize_priority_drops_en_dedups_and_fills_gaps() {
    // A short, En-polluted, duplicated list still normalizes to a full,
    // well-formed 4-member CJK permutation — En dropped, first occurrence
    // of a dup wins, and the missing members fill in DEFAULT order.
    let n = normalize_priority(&[Lang::En, Lang::Ko, Lang::Ko, Lang::En]);
    assert_eq!(n, [Lang::Ko, Lang::Ja, Lang::ZhHans, Lang::ZhHant]);
    // An already-well-formed permutation is untouched (order preserved).
    let full = [Lang::ZhHant, Lang::Ko, Lang::Ja, Lang::ZhHans];
    assert_eq!(normalize_priority(&full), full);
    // A totally empty input falls back to the built-in default order.
    assert_eq!(normalize_priority(&[]), DEFAULT_CJK_PRIORITY);
}

#[test]
fn cjk_priority_global_defaults_seeds_sets_and_promotes() {
    let _g = crate::testlock::serial();
    // Reset to the built-in default so this test is order-independent.
    set_cjk_priority(&DEFAULT_CJK_PRIORITY);
    assert_eq!(cjk_priority(), DEFAULT_CJK_PRIORITY.to_vec());

    // Promoting the already-front language is a no-op (still the same order).
    assert_eq!(
        promote_cjk_priority(Lang::Ja),
        DEFAULT_CJK_PRIORITY.to_vec()
    );

    // Promoting Korean moves it to the front; the REST keep their relative
    // order (Ja, ZhHans, ZhHant) — the picker's whole point.
    let promoted = promote_cjk_priority(Lang::Ko);
    assert_eq!(
        promoted,
        vec![Lang::Ko, Lang::Ja, Lang::ZhHans, Lang::ZhHant]
    );
    set_cjk_priority(&promoted);
    assert_eq!(cjk_priority(), promoted);

    // Promoting zh-Hant (currently 3rd) keeps Ko/Ja's relative order.
    let promoted2 = promote_cjk_priority(Lang::ZhHant);
    assert_eq!(
        promoted2,
        vec![Lang::ZhHant, Lang::Ko, Lang::Ja, Lang::ZhHans]
    );

    // Cleanup: leave the global at the built-in default for other tests.
    set_cjk_priority(&DEFAULT_CJK_PRIORITY);
}

#[test]
fn set_cjk_priority_normalizes_a_malformed_input() {
    let _g = crate::testlock::serial();
    set_cjk_priority(&[Lang::En, Lang::Ko]);
    assert_eq!(
        cjk_priority(),
        vec![Lang::Ko, Lang::Ja, Lang::ZhHans, Lang::ZhHant]
    );
    set_cjk_priority(&DEFAULT_CJK_PRIORITY);
}
