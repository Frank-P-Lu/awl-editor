use super::*;

fn m(start: usize, end: usize) -> Match {
    Match { start, end }
}

#[test]
fn find_all_basic() {
    assert_eq!(find_all("hello world", "world", false), vec![m(6, 11)]);
    assert_eq!(
        find_all("hello world", "o", false),
        vec![m(4, 4 + 1), m(7, 8)]
    );
}

#[test]
fn find_all_multiple_hits() {
    let hay = "line one\nline two\nline three";
    let got = find_all(hay, "line", false);
    assert_eq!(got.len(), 3);
    assert_eq!(got[0], m(0, 4));
}

#[test]
fn find_all_non_overlapping() {
    assert_eq!(find_all("aaaa", "aa", false), vec![m(0, 2), m(2, 4)]);
}

#[test]
fn find_all_case_insensitive_default_vs_sensitive() {
    assert_eq!(find_all("Hello HELLO hello", "hello", false).len(), 3);
    assert_eq!(
        find_all("Hello HELLO hello", "hello", true),
        vec![m(12, 17)]
    );
}

#[test]
fn find_all_empty_needle() {
    assert!(find_all("anything", "", false).is_empty());
    assert!(find_all("", "x", false).is_empty());
}

#[test]
fn find_all_multibyte_char_offsets() {
    // "naïve café" — the 'ï' (U+00EF) and 'é' (U+00E9) are multibyte in
    // UTF-8, so byte offsets would differ from char offsets. We assert CHAR
    // offsets.
    let hay = "naïve café";
    let got = find_all(hay, "café", false);
    assert_eq!(got, vec![m(6, 10)]);
    let chars: Vec<char> = hay.chars().collect();
    let matched: String = chars[got[0].start..got[0].end].iter().collect();
    assert_eq!(matched, "café");

    let cjk = "日本語日本語";
    let g = find_all(cjk, "日本", false);
    assert_eq!(g, vec![m(0, 2), m(3, 5)]);
}

#[test]
fn current_pick_forward_at_or_after_origin_then_wrap() {
    let hay = "..x...x...x";
    let mut s = SearchState::start(4, Direction::Forward);
    s.push_char('x', hay);
    assert_eq!(s.hit_count(), 3);
    assert_eq!(s.current_match(), Some(m(6, 7)));
    let mut s2 = SearchState::start(100, Direction::Forward);
    s2.push_char('x', hay);
    assert_eq!(s2.current_match(), Some(m(2, 3)));
}

#[test]
fn current_pick_backward_at_or_before_origin_then_wrap() {
    let hay = "..x...x...x";
    let mut s = SearchState::start(8, Direction::Backward);
    s.push_char('x', hay);
    assert_eq!(s.current_match(), Some(m(6, 7)));
    let mut s2 = SearchState::start(0, Direction::Backward);
    s2.push_char('x', hay);
    assert_eq!(s2.current_match(), Some(m(10, 11)));
}

#[test]
fn step_forward_and_backward_wrap() {
    let hay = "x.x.x";
    let mut s = SearchState::start(0, Direction::Forward);
    s.push_char('x', hay); // matches at 0,2,4; current 0
    assert_eq!(s.current_match(), Some(m(0, 1)));
    assert_eq!(s.step(Direction::Forward), StepOutcome::Moved);
    assert_eq!(s.current_match(), Some(m(2, 3)));
    assert_eq!(s.step(Direction::Forward), StepOutcome::Moved);
    assert_eq!(s.current_match(), Some(m(4, 5)));
    assert_eq!(
        s.step(Direction::Forward),
        StepOutcome::RecoiledAtBoundary(Direction::Forward)
    );
    assert_eq!(s.current_match(), Some(m(4, 5)), "recoil does not advance");
    assert_eq!(s.step(Direction::Forward), StepOutcome::Wrapped);
    assert_eq!(s.current_match(), Some(m(0, 1)));
    assert_eq!(
        s.step(Direction::Backward),
        StepOutcome::RecoiledAtBoundary(Direction::Backward)
    );
    assert_eq!(s.current_match(), Some(m(0, 1)), "recoil does not advance");
    assert_eq!(s.step(Direction::Backward), StepOutcome::Wrapped);
    assert_eq!(s.current_match(), Some(m(4, 5)));
}

#[test]
fn wrap_arm_set_and_cleared_by_other_actions() {
    let hay = "x.x.x";
    let mut s = SearchState::start(0, Direction::Forward);
    s.push_char('x', hay); // 0,2,4; current 0
    s.step(Direction::Forward); // ->2
    s.step(Direction::Forward); // ->4 (last)
    assert_eq!(s.wrap_armed, None);
    assert_eq!(
        s.step(Direction::Forward),
        StepOutcome::RecoiledAtBoundary(Direction::Forward)
    );
    assert_eq!(s.wrap_armed, Some(Direction::Forward));
    assert_eq!(s.step(Direction::Backward), StepOutcome::Moved);
    assert_eq!(s.wrap_armed, None);
    assert_eq!(s.current_match(), Some(m(2, 3)));

    let mut s2 = SearchState::start(0, Direction::Forward);
    s2.push_char('x', hay);
    s2.step(Direction::Forward); // ->2
    s2.step(Direction::Forward); // ->4
    s2.step(Direction::Forward); // arm
    assert_eq!(s2.wrap_armed, Some(Direction::Forward));
    s2.push_char('.', hay); // query "x." now matches at 0,2; recompute disarms
    assert_eq!(s2.wrap_armed, None);
    let mut s3 = SearchState::start(0, Direction::Forward);
    s3.push_char('x', hay);
    s3.step(Direction::Forward);
    s3.step(Direction::Forward);
    s3.step(Direction::Forward); // arm
    s3.pop_char(hay); // also disarms via recompute
    assert_eq!(s3.wrap_armed, None);
}

#[test]
fn wrap_arm_single_match_alternates_recoil_and_wrap() {
    // With ONE match there is no "next": a forward step recoils (arms), the next
    // wraps to itself, and the pattern alternates. No panic on the len-1 edge.
    let hay = "..x..";
    let mut s = SearchState::start(0, Direction::Forward);
    s.push_char('x', hay); // one match at 2
    assert_eq!(s.current_match(), Some(m(2, 3)));
    assert_eq!(
        s.step(Direction::Forward),
        StepOutcome::RecoiledAtBoundary(Direction::Forward)
    );
    assert_eq!(s.step(Direction::Forward), StepOutcome::Wrapped);
    assert_eq!(s.current_match(), Some(m(2, 3)));
}

#[test]
fn step_noop_when_empty() {
    let mut s = SearchState::start(0, Direction::Forward);
    s.push_char('z', "abc"); // no matches
    assert_eq!(s.current_match(), None);
    assert_eq!(s.step(Direction::Forward), StepOutcome::NoMatches); // must not panic
    assert_eq!(s.current_match(), None);
}

#[test]
fn push_then_pop_restores_match_set() {
    let hay = "abc abd";
    let mut s = SearchState::start(0, Direction::Forward);
    s.push_char('a', hay); // matches at 0,4
    s.push_char('b', hay); // matches at 0,4 (ab, ab)
    let two = s.hit_count();
    assert_eq!(two, 2);
    s.push_char('c', hay); // only "abc" => 1 match
    assert_eq!(s.hit_count(), 1);
    s.pop_char(hay); // back to "ab" => 2 matches
    assert_eq!(s.hit_count(), 2);
    assert_eq!(s.query(), "ab");
}

#[test]
fn toggle_case_changes_hit_count() {
    let hay = "Hello HELLO hello";
    let mut s = SearchState::start(0, Direction::Forward);
    for c in "hello".chars() {
        s.push_char(c, hay);
    }
    assert_eq!(s.hit_count(), 3); // insensitive default
    s.toggle_case(hay);
    assert!(s.is_case_sensitive());
    assert_eq!(s.hit_count(), 1); // only exact "hello"
    s.toggle_case(hay);
    assert_eq!(s.hit_count(), 3);
}

#[test]
fn origin_preserved_across_edits() {
    let mut s = SearchState::start(42, Direction::Forward);
    s.push_char('a', "aaa");
    s.push_char('a', "aaa");
    s.pop_char("aaa");
    s.toggle_case("aaa");
    assert_eq!(s.origin(), 42);
}

#[test]
fn replace_mode_reveal_and_focus_toggle() {
    let mut s = SearchState::start(0, Direction::Forward);
    // Off by default: a plain isearch never reveals the replace field.
    assert!(!s.is_replace_active());
    assert!(!s.is_editing_replacement());
    s.toggle_replace();
    assert!(s.is_replace_active());
    assert!(s.is_editing_replacement());
    s.toggle_replace();
    assert!(s.is_replace_active());
    assert!(!s.is_editing_replacement());
    s.toggle_replace();
    assert!(s.is_editing_replacement());
    for c in "X".chars() {
        s.push_replace_char(c);
    }
    s.push_replace_char('Y');
    assert_eq!(s.replacement(), "XY");
    s.pop_replace_char();
    assert_eq!(s.replacement(), "X");
}

#[test]
fn reveal_replace_keeps_find_focus_then_focus_replacement_moves_it() {
    let mut s = SearchState::start(0, Direction::Forward);
    s.reveal_replace();
    assert!(s.is_replace_active(), "replace row is revealed");
    assert!(!s.is_editing_replacement(), "focus stays on the find field");
    // Idempotent: a second reveal never steals focus back once you've moved on.
    s.toggle_replace(); // Tab -> switch to replace
    assert!(s.is_editing_replacement());
    s.reveal_replace();
    assert!(
        s.is_editing_replacement(),
        "reveal_replace never yanks focus"
    );
    let mut s2 = SearchState::start(0, Direction::Forward);
    s2.reveal_replace();
    assert!(!s2.is_editing_replacement());
    s2.focus_replacement();
    assert!(s2.is_replace_active() && s2.is_editing_replacement());
}

/// CLICK-TO-SWITCH-FIELD's pure state change: a press on the REPLACE row
/// (`focus_replacement`) edits the replacement; a press on the FIND row
/// (`focus_query`) returns to the query — and `focus_query` leaves the replace
/// row revealed (a click never hides it). These are the two doors
/// `App::panel_click` drives off `TextPipeline::panel_hit`.
#[test]
fn click_focus_doors_switch_the_edited_field() {
    let mut s = SearchState::start(0, Direction::Forward);
    s.focus_replacement(); // click the replace row
    assert!(s.is_replace_active());
    assert!(s.is_editing_replacement());
    s.focus_query(); // click the find row
    assert!(!s.is_editing_replacement(), "focus returns to the query");
    assert!(s.is_replace_active(), "the replace row stays revealed");
    s.focus_query();
    assert!(!s.is_editing_replacement());
}

#[test]
fn replace_all_text_swaps_every_match() {
    let hay = "line one\nline two\nline three";
    let mut s = SearchState::start(0, Direction::Forward);
    for c in "line".chars() {
        s.push_char(c, hay);
    }
    s.toggle_replace();
    for c in "row".chars() {
        s.push_replace_char(c);
    }
    assert_eq!(s.hit_count(), 3);
    let out = s.replace_all_text(hay);
    assert_eq!(out, "row one\nrow two\nrow three");
    let mut z = SearchState::start(0, Direction::Forward);
    z.push_char('z', hay);
    assert_eq!(z.replace_all_text(hay), hay);
}

#[test]
fn replace_current_text_replaces_one_then_advances() {
    let hay = "x.x.x";
    let mut s = SearchState::start(0, Direction::Forward);
    s.push_char('x', hay); // matches at 0,2,4; current = 0
    s.toggle_replace();
    s.push_replace_char('Y'); // single-char replacement keeps offsets simple
    assert_eq!(s.current_match(), Some(m(0, 1)));
    let t1 = s.replace_current_text(hay).unwrap();
    assert_eq!(t1, "Y.x.x");
    assert_eq!(s.current_match(), Some(m(2, 3)));
    let t2 = s.replace_current_text(&t1).unwrap();
    assert_eq!(t2, "Y.Y.x");
    assert_eq!(s.current_match(), Some(m(4, 5)));
}

#[test]
fn replace_current_text_handles_multibyte() {
    let hay = "café au lait, café noir";
    let mut s = SearchState::start(0, Direction::Forward);
    for c in "café".chars() {
        s.push_char(c, hay);
    }
    s.toggle_replace();
    for c in "thé".chars() {
        s.push_replace_char(c);
    }
    let out = s.replace_current_text(hay).unwrap();
    assert_eq!(out, "thé au lait, café noir");
}

#[test]
fn replace_writeback_roundtrips_buffer_and_lands_cursor() {
    use crate::buffer::Buffer;

    let mut buf = Buffer::from_str("line one\nline two\nline three");
    let mut st = SearchState::start(0, Direction::Forward);
    let q_hay = buf.text();
    for c in "line".chars() {
        st.push_char(c, &q_hay);
    }
    st.toggle_replace();
    for c in "row".chars() {
        st.push_replace_char(c);
    }
    let hay = buf.text();
    let new_text = st.replace_all_text(&hay);
    let origin = st.origin();
    assert_ne!(new_text, hay, "replace-all must change the text");
    buf.set_text(&new_text);
    let new_hay = buf.text();
    st.refind(origin, &new_hay);
    if let Some(mm) = st.current_match() {
        buf.set_cursor(mm.start);
    }
    assert_eq!(buf.text(), "row one\nrow two\nrow three");
    assert_eq!(
        st.current_match(),
        None,
        "no needle remains after replace-all"
    );

    let mut buf = Buffer::from_str("x.x.x");
    let mut st = SearchState::start(0, Direction::Forward);
    st.push_char('x', &buf.text());
    st.toggle_replace();
    st.push_replace_char('Y');
    let replace_current_once = |buf: &mut Buffer, st: &mut SearchState| {
        let hay = buf.text();
        if let Some(t) = st.replace_current_text(&hay) {
            buf.set_text(&t);
            if let Some(mm) = st.current_match() {
                buf.set_cursor(mm.start);
            }
        }
    };
    replace_current_once(&mut buf, &mut st);
    assert_eq!(buf.text(), "Y.x.x");
    assert_eq!(st.current_match(), Some(m(2, 3)));
    assert_eq!(buf.cursor_char(), 2, "cursor lands on the next match");
    replace_current_once(&mut buf, &mut st);
    assert_eq!(buf.text(), "Y.Y.x");
    assert_eq!(buf.cursor_char(), 4);

    let mut buf = Buffer::from_str("café au lait, café noir");
    let mut st = SearchState::start(0, Direction::Forward);
    for c in "café".chars() {
        st.push_char(c, &buf.text());
    }
    st.toggle_replace();
    for c in "thé".chars() {
        st.push_replace_char(c);
    }
    replace_current_once(&mut buf, &mut st);
    assert_eq!(buf.text(), "thé au lait, café noir");
    assert_eq!(buf.cursor_char(), 13, "next 'café' starts at char 13");
}

#[test]
fn start_with_query_prefills_and_matches_immediately() {
    let hay = "alpha beta alpha gamma alpha";
    let s = SearchState::start_with_query(0, Direction::Forward, "alpha", hay);
    assert_eq!(s.query(), "alpha");
    assert_eq!(s.hit_count(), 3);
    assert!(
        s.current_match().is_some(),
        "the prefilled query is matched, not blank"
    );
    let blank = SearchState::start_with_query(0, Direction::Forward, "", hay);
    assert_eq!(blank.query(), "");
    assert_eq!(blank.hit_count(), 0);
}

#[test]
fn last_query_remembers_and_is_reset_by_clear() {
    let _g = crate::testlock::serial();
    clear_last_query();
    assert_eq!(
        last_query(),
        "",
        "a fresh/cleared process remembers nothing"
    );
    set_last_query("needle");
    assert_eq!(last_query(), "needle");
    // A LATER empty close never overwrites a still-useful remembered query
    // (an abandoned blank search shouldn't erase the last real one).
    set_last_query("");
    assert_eq!(last_query(), "needle");
    set_last_query("second");
    assert_eq!(last_query(), "second");
    clear_last_query(); // leave no residue for other tests reading the global
}

/// LAW — the vectorized byte path and the exhaustive char scan must return
/// the IDENTICAL span list for every text awl can hold.
///
/// The axis that matters is not "does search still work" but WHICH path a
/// given (haystack, needle, case) triple lands on, so the corpus sweeps the
/// gate itself: pure ASCII (byte path both ways), non-ASCII haystack with an
/// ASCII needle (byte path both ways unless it contains Kelvin Sign), a
/// non-ASCII needle, and folds Unicode does NOT do per-char (`İ`, `ß`).
/// Multibyte text sits BEFORE the matches on purpose — a byte offset used
/// as a char index is the exact bug this remap exists to prevent, and it is
/// invisible unless something multibyte precedes a hit.
#[test]
fn byte_path_and_char_path_agree_across_the_corpus() {
    let _g = crate::testlock::serial();
    let corpus = [
        "",
        "the quick brown fox",
        "The Quick BROWN fox the THE tHe",
        "aaaaa",
        "abababab",
        "line one\nline two\r\nline three\rlone cr",
        // multibyte BEFORE the hits: catches a byte offset leaking through
        // as a char index.
        "日本語のテスト the fox 日本語 the end",
        "café the naïve the résumé",
        "e\u{301}crit the combining the mark",
        "İstanbul the İ fold",
        "straße the ß fold STRASSE",
        "🎨🎭 the emoji the 🎪 tail",
        "\u{2028}\u{2029} the separators the",
        "bake a \u{212A}elvin the cake",
        "\u{212A}\u{212A} the double kelvin the",
    ];
    let needles = [
        "the",
        "The",
        "THE",
        "tHe",
        "a",
        "aa",
        "ab",
        "fox",
        "日本語",
        "テスト",
        "é",
        "e",
        "İ",
        "ß",
        "ss",
        "🎨",
        "k",
        "K",
        "\u{212A}",
        "\n",
        "\r\n",
        "\r",
        "zzz",
        "the quick brown fox and more",
        " ",
        "  ",
    ];
    let mut byte_path_hits = 0usize;
    for haystack in corpus {
        for needle in needles {
            for case_sensitive in [true, false] {
                let got = find_all(haystack, needle, case_sensitive);
                let want = find_all_by_char(haystack, needle, case_sensitive);
                assert_eq!(
                    got, want,
                    "path divergence: haystack={haystack:?} needle={needle:?} \
                         case_sensitive={case_sensitive}"
                );
                // Every reported span must be a REAL char slice of the
                // haystack, so an off-by-one in the remap cannot hide behind
                // both paths agreeing on a wrong answer.
                let chars: Vec<char> = haystack.chars().collect();
                for m in &got {
                    assert!(m.end <= chars.len(), "span past end: {m:?}");
                    let slice: String = chars[m.start..m.end].iter().collect();
                    let equal = if case_sensitive {
                        slice == needle
                    } else {
                        slice.to_lowercase() == needle.to_lowercase()
                    };
                    assert!(
                        equal,
                        "span {m:?} reads {slice:?}, not {needle:?} in {haystack:?}"
                    );
                }
                if haystack.is_ascii() && needle.is_ascii() && !got.is_empty() {
                    byte_path_hits += 1;
                }
            }
        }
    }
    // Non-vacuity: the fast path must actually be producing hits, not
    // silently returning empty while the oracle also returns empty.
    assert!(
        byte_path_hits > 20,
        "the byte path barely ran ({byte_path_hits} hit cases) — the law would \
             pass even if it were broken"
    );
}

#[test]
fn mixed_utf8_prose_with_an_ascii_needle_uses_the_byte_path() {
    let _g = crate::testlock::serial();
    let haystack = "日本語の前置き — then the ASCII needle";
    assert!(can_byte_search(haystack, "ascii", false));
    assert_eq!(
        find_all(haystack, "ascii", false),
        vec![Match { start: 19, end: 24 }]
    );
    assert!(!can_byte_search("a Kelvin exception", "k", false));
}

/// LAW — the ASCII byte fold is exact because exactly ONE non-ASCII scalar
/// folds to an ASCII char, and [`find_all`] guards on it.
///
/// Swept over EVERY Unicode scalar rather than the handful anyone would
/// think to name: if a future Unicode table adds a second such scalar, this
/// goes red and the guard has to grow with it.
#[test]
fn kelvin_sign_is_the_only_scalar_folding_to_ascii() {
    let _g = crate::testlock::serial();
    let mut found: Vec<char> = Vec::new();
    for cp in 0u32..=0x10FFFF {
        let Some(c) = char::from_u32(cp) else {
            continue;
        };
        if c.is_ascii() {
            continue;
        }
        let lower: Vec<char> = c.to_lowercase().collect();
        if lower.len() == 1 && lower[0].is_ascii() {
            found.push(c);
        }
    }
    assert_eq!(
        found,
        vec![KELVIN_SIGN.chars().next().unwrap()],
        "the set of non-ASCII scalars folding to ASCII changed; find_all's \
             byte-fold guard must cover exactly this set"
    );
    // And the guard must actually SEE it, wherever it sits.
    assert!(holds_kelvin_sign(KELVIN_SIGN));
    assert!(holds_kelvin_sign(&format!(
        "a long stretch of prose {KELVIN_SIGN} tail"
    )));
    assert!(!holds_kelvin_sign("no kelvin here, just a plain k and K"));
    // The exotic fold still resolves correctly, via the char fallback.
    let text = format!("bake a {KELVIN_SIGN}elvin cake");
    assert_eq!(
        find_all(&text, "k", false),
        find_all_by_char(&text, "k", false),
        "U+212A must still fold-match 'k' through the fallback"
    );
}

/// LAW — line-ending detection is unchanged by vectorizing its two counts.
#[test]
fn eol_detect_matches_the_scalar_count_on_every_mix() {
    let _g = crate::testlock::serial();
    fn scalar(s: &str) -> crate::buffer::Eol {
        let total_lf = s.bytes().filter(|&b| b == b'\n').count();
        let crlf = s.match_indices("\r\n").count();
        let lone_lf = total_lf - crlf;
        if crlf > lone_lf {
            crate::buffer::Eol::Crlf
        } else {
            crate::buffer::Eol::Lf
        }
    }
    let cases = [
        "",
        "no breaks at all",
        "a\nb\nc",
        "a\r\nb\r\nc",
        // exact ties must resolve the SAME way (Lf wins on `>`)
        "a\r\nb\nc",
        "a\nb\r\nc\r\nd\ne",
        // a lone CR is CONTENT, never a break — and `\r\r\n` must count ONE
        // pair, not two, which is where a naive overlapping scan diverges.
        "a\rb\rc",
        "a\r\r\nb",
        "\r\n",
        "\n",
        "\r",
        "日本語\r\nテスト\n",
        "trailing\r\n\r\n",
    ];
    for s in cases {
        assert_eq!(
            crate::buffer::Eol::detect(s),
            scalar(s),
            "eol detect diverged on {s:?}"
        );
    }
}
