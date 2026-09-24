use super::*;

fn tight_budget() -> SearchBudget {
    SearchBudget {
        max_files: 300,
        max_total_bytes: 20_000_000,
        max_file_bytes: 1_000_000,
        max_hits: 200,
        max_hits_per_file: 20,
        snippet_chars: 80,
    }
}

/// **THE CASE-FOLDING / UNICODE DECISION, RECORDED.** `search` matches through
/// `crate::search::find_all(.., case_sensitive: false)` — Unicode-aware
/// casefold via `char::to_lowercase`, the SAME matcher and the SAME default
/// Cmd-F's in-buffer isearch already ships. An upper-cased accented query
/// finds a lower-cased accented candidate: this is not an ASCII-only
/// `to_ascii_lowercase` fold (which would miss it).
#[test]
fn case_insensitive_unicode_query_matches_via_the_shared_matcher() {
    let corpus = vec![("notes/menu.md".to_string(), "the café is open".to_string())];
    let hits = search(&corpus, "CAFÉ", &tight_budget());
    assert_eq!(
        hits.len(),
        1,
        "an upper-cased accented query must fold onto the lower-cased candidate"
    );
    assert_eq!(&hits[0].snippet[hits[0].hl_start..hits[0].hl_end], "café");
}

#[test]
fn ordinary_ascii_case_insensitivity_holds_too() {
    let corpus = vec![("a.md".to_string(), "Hello World".to_string())];
    let hits = search(&corpus, "world", &tight_budget());
    assert_eq!(hits.len(), 1);
    assert_eq!(&hits[0].snippet[hits[0].hl_start..hits[0].hl_end], "World");
}

#[test]
fn empty_query_scans_nothing() {
    let corpus = vec![("a.md".to_string(), "anything at all".to_string())];
    assert!(search(&corpus, "", &tight_budget()).is_empty());
}

#[test]
fn hits_arrive_grouped_by_file_in_corpus_order() {
    let corpus = vec![
        ("b.md".to_string(), "needle one".to_string()),
        ("a.md".to_string(), "needle two\nneedle three".to_string()),
    ];
    let hits = search(&corpus, "needle", &tight_budget());
    let paths: Vec<&str> = hits.iter().map(|h| h.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["b.md", "a.md", "a.md"],
        "hits must stay contiguous per file, in CORPUS order (never re-sorted alphabetically), \
         so adjacent rows read as one group"
    );
}

/// Line/col land the caret through `Effect::OpenPathAtLine` — both are
/// CHAR-indexed, zero-based, matching `line_col_to_char`'s own unit.
#[test]
fn line_and_col_are_zero_based_char_indices() {
    let corpus = vec![(
        "a.md".to_string(),
        "first line\nsecond line has needle here".to_string(),
    )];
    let hits = search(&corpus, "needle", &tight_budget());
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].line, 1, "the match is on the SECOND (index 1) line");
    let expected_col = "second line has ".chars().count();
    assert_eq!(hits[0].col, expected_col);
}

#[test]
fn max_hits_caps_the_total_across_every_file() {
    let corpus: Vec<(String, String)> = (0..10)
        .map(|i| (format!("f{i}.md"), "needle needle needle".to_string()))
        .collect();
    let budget = SearchBudget {
        max_hits: 5,
        ..tight_budget()
    };
    let hits = search(&corpus, "needle", &budget);
    assert_eq!(
        hits.len(),
        5,
        "the total must stop exactly at the budget, mid-file if needed"
    );
    assert!(search_report(&corpus, "needle", &budget).limited);
}

#[test]
fn max_hits_per_file_caps_one_files_share_without_starving_the_rest() {
    let corpus = vec![
        ("dense.md".to_string(), "needle ".repeat(20)),
        ("other.md".to_string(), "one needle here".to_string()),
    ];
    let budget = SearchBudget {
        max_hits_per_file: 3,
        max_hits: 200,
        ..tight_budget()
    };
    let hits = search(&corpus, "needle", &budget);
    let dense = hits.iter().filter(|h| h.path == "dense.md").count();
    let other = hits.iter().filter(|h| h.path == "other.md").count();
    assert_eq!(dense, 3, "one dense file must be capped per-file");
    assert_eq!(
        other, 1,
        "capping the dense file must not crowd out the other file's own hit"
    );
    assert!(search_report(&corpus, "needle", &budget).limited);
}

#[test]
fn a_query_below_both_result_caps_does_not_claim_to_be_limited() {
    let corpus = vec![("a.md".to_string(), "one needle here".to_string())];
    assert!(!search_report(&corpus, "needle", &tight_budget()).limited);
}

/// A short line (within the snippet width) is returned UNCHANGED — no
/// spurious ellipsis on ordinary prose, the common case.
#[test]
fn short_line_is_not_windowed() {
    let corpus = vec![(
        "a.md".to_string(),
        "a short line with needle in it".to_string(),
    )];
    let hits = search(&corpus, "needle", &tight_budget());
    assert_eq!(hits[0].snippet, "a short line with needle in it");
    assert_eq!(&hits[0].snippet[hits[0].hl_start..hits[0].hl_end], "needle");
}

/// **THE MATCH IS NEVER ELIDED AWAY.** A long line's match survives windowing
/// even when the match sits far from either end, unlike `rowlayout::fit_primary`'s
/// generic trailing-ellipsis elision (which would happily cut the match if it
/// fell past the budget).
#[test]
fn long_line_windows_around_the_match_and_keeps_it_intact() {
    let padding_before = "x".repeat(200);
    let padding_after = "y".repeat(200);
    let line = format!("{padding_before} needle {padding_after}");
    let corpus = vec![("a.md".to_string(), line)];
    let budget = SearchBudget {
        snippet_chars: 30,
        ..tight_budget()
    };
    let hits = search(&corpus, "needle", &budget);
    assert_eq!(hits.len(), 1);
    let hit = &hits[0];
    assert!(
        hit.snippet.chars().count() <= budget.snippet_chars + 2,
        "windowed snippet ({} chars: {:?}) must stay near the budget (plus up to two ellipses)",
        hit.snippet.chars().count(),
        hit.snippet
    );
    assert_eq!(
        &hit.snippet[hit.hl_start..hit.hl_end],
        "needle",
        "the match itself must survive windowing byte-for-byte"
    );
    assert!(
        hit.snippet.starts_with('\u{2026}'),
        "context was cut on both sides"
    );
    assert!(hit.snippet.ends_with('\u{2026}'));
}

/// A match wider than the whole snippet budget (a pathological query) is
/// still shown whole rather than truncated — the budget bounds ROWS, never
/// the text a row exists to show.
#[test]
fn a_match_wider_than_the_budget_is_shown_whole() {
    let long_needle = "n".repeat(50);
    let corpus = vec![("a.md".to_string(), format!("before {long_needle} after"))];
    let budget = SearchBudget {
        snippet_chars: 10,
        ..tight_budget()
    };
    let hits = search(&corpus, &long_needle, &budget);
    assert_eq!(hits.len(), 1);
    assert_eq!(
        &hits[0].snippet[hits[0].hl_start..hits[0].hl_end],
        long_needle
    );
}

/// A match at the very START of a long line still fills the window from the
/// available text on the far side, rather than showing fewer chars than the
/// budget allows.
#[test]
fn match_near_line_start_still_fills_the_window() {
    let line = format!("needle {}", "z".repeat(200));
    let corpus = vec![("a.md".to_string(), line)];
    let budget = SearchBudget {
        snippet_chars: 30,
        ..tight_budget()
    };
    let hits = search(&corpus, "needle", &budget);
    assert!(
        !hits[0].snippet.starts_with('\u{2026}'),
        "nothing precedes the match at line start"
    );
    assert!(hits[0].snippet.ends_with('\u{2026}'));
    assert_eq!(hits[0].snippet.chars().count(), budget.snippet_chars + 1);
}

// ── load_corpus: attempted-I/O and actual-byte laws ────────────────────────

fn complete(text: &str) -> crate::fs::BoundedRead {
    crate::fs::BoundedRead::Complete(text.as_bytes().to_vec())
}

#[test]
fn max_files_counts_attempts_including_all_rejected_candidates() {
    let files: Vec<String> = (0..1_000).map(|i| format!("huge-{i}.md")).collect();
    let budget = SearchBudget {
        max_files: 3,
        max_file_bytes: 10,
        ..tight_budget()
    };
    let mut metadata_calls = 0;
    let mut read_calls = 0;
    let load = load_corpus_with(
        &files,
        &budget,
        |_| {
            metadata_calls += 1;
            Some(100_000)
        },
        |_, _| {
            read_calls += 1;
            unreachable!("known-oversized files must be rejected before content I/O")
        },
    );
    assert_eq!(load.attempted_files, 3);
    assert_eq!(metadata_calls, 3);
    assert_eq!(read_calls, 0);
    assert_eq!(load.bytes_read, 0);
    assert!(load.corpus.is_empty());
    assert!(load.incomplete);
}

#[test]
fn exact_attempt_limit_admits_exactly_that_many_files_and_marks_the_rest_incomplete() {
    let files: Vec<String> = (0..4).map(|i| format!("f{i}.md")).collect();
    let budget = SearchBudget {
        max_files: 3,
        ..tight_budget()
    };
    let mut reads = 0;
    let load = load_corpus_with(
        &files,
        &budget,
        |_| Some(1),
        |_, cap| {
            reads += 1;
            assert!(cap >= 1);
            complete("x")
        },
    );
    assert_eq!(reads, 3);
    assert_eq!(load.attempted_files, 3);
    assert_eq!(load.corpus.len(), 3);
    assert_eq!(load.bytes_read, 3);
    assert!(load.incomplete);
}

#[test]
fn exact_per_file_and_total_limits_are_admitted_but_cap_plus_one_is_not_read() {
    let files = vec!["ten.md".into(), "fifteen.md".into(), "one-more.md".into()];
    let budget = SearchBudget {
        max_total_bytes: 25,
        max_file_bytes: 15,
        ..tight_budget()
    };
    let mut reads = Vec::new();
    let load = load_corpus_with(
        &files,
        &budget,
        |path| match path {
            "ten.md" => Some(10),
            "fifteen.md" => Some(15),
            "one-more.md" => Some(1),
            _ => unreachable!(),
        },
        |path, cap| {
            reads.push((path.to_string(), cap));
            match path {
                "ten.md" => complete(&"a".repeat(10)),
                "fifteen.md" => complete(&"b".repeat(15)),
                _ => unreachable!("the total cap must stop before another read"),
            }
        },
    );
    assert_eq!(
        reads,
        vec![("ten.md".into(), 15), ("fifteen.md".into(), 15)]
    );
    assert_eq!(load.bytes_read, 25);
    assert_eq!(load.corpus.len(), 2);
    assert_eq!(load.corpus.iter().map(|(_, s)| s.len()).sum::<usize>(), 25);
    assert!(load.incomplete);
}

#[test]
fn known_file_over_per_file_cap_is_rejected_without_reading_it() {
    let files = vec!["exact.md".into(), "cap-plus-one.md".into()];
    let budget = SearchBudget {
        max_file_bytes: 10,
        ..tight_budget()
    };
    let mut reads = Vec::new();
    let load = load_corpus_with(
        &files,
        &budget,
        |path| Some(if path == "exact.md" { 10 } else { 11 }),
        |path, cap| {
            reads.push((path.to_string(), cap));
            complete(&"x".repeat(10))
        },
    );
    assert_eq!(reads, vec![("exact.md".into(), 10)]);
    assert_eq!(load.bytes_read, 10);
    assert_eq!(load.corpus.len(), 1);
    assert!(load.incomplete);
}

#[test]
fn unknown_or_growing_size_is_capped_and_never_admitted_as_a_prefix() {
    let files = vec!["growing.md".into(), "never-attempted.md".into()];
    let budget = SearchBudget {
        max_total_bytes: 10,
        max_file_bytes: 10,
        ..tight_budget()
    };
    let mut reads = 0;
    let load = load_corpus_with(
        &files,
        &budget,
        |_| None,
        |_, cap| {
            reads += 1;
            assert_eq!(cap, 10);
            crate::fs::BoundedRead::LimitReached(vec![b'x'; cap])
        },
    );
    assert_eq!(reads, 1);
    assert_eq!(load.attempted_files, 1);
    assert_eq!(load.bytes_read, 10);
    assert!(load.corpus.is_empty());
    assert!(load.incomplete);
}

#[test]
fn invalid_text_and_failed_reads_charge_actual_bytes_and_do_not_abort_later_text() {
    let files = vec!["binary.md".into(), "failed.md".into(), "ok.md".into()];
    let budget = SearchBudget {
        max_total_bytes: 10,
        max_file_bytes: 10,
        ..tight_budget()
    };
    let mut calls = Vec::new();
    let load = load_corpus_with(
        &files,
        &budget,
        |_| None,
        |path, cap| {
            calls.push((path.to_string(), cap));
            match path {
                "binary.md" => crate::fs::BoundedRead::Complete(vec![0xff]),
                "failed.md" => crate::fs::BoundedRead::Failed {
                    bytes: vec![b'x', b'y'],
                    _error: std::io::Error::other("late failure"),
                },
                "ok.md" => complete("needle"),
                _ => unreachable!(),
            }
        },
    );
    assert_eq!(
        calls,
        vec![
            ("binary.md".into(), 10),
            ("failed.md".into(), 9),
            ("ok.md".into(), 7)
        ]
    );
    assert_eq!(
        load.bytes_read, 9,
        "invalid and partial-failure bytes are charged"
    );
    assert_eq!(load.corpus, vec![("ok.md".into(), "needle".into())]);
    assert!(load.incomplete);
}

#[test]
fn incomplete_coverage_is_visible_in_the_picker_footer_without_hiding_open() {
    let complete = crate::overlay::OverlayState::new_search_folder(
        std::path::PathBuf::from("/notes"),
        vec![("a.md".into(), "needle".into())],
        false,
    );
    let incomplete = crate::overlay::OverlayState::new_search_folder(
        std::path::PathBuf::from("/notes"),
        vec![("a.md".into(), "needle".into())],
        true,
    );
    assert_eq!(complete.foot_hint(), "type to filter   ↵ open   esc close");
    assert_eq!(
        incomplete.foot_hint(),
        "some files not searched   ↵ open   esc close"
    );
}

#[test]
fn nul_bearing_utf8_is_not_a_searchable_document() {
    let files = vec!["binary.md".into()];
    let load = load_corpus_with(
        &files,
        &tight_budget(),
        |_| None,
        |_, _| complete("needle\0binary"),
    );
    assert_eq!(load.attempted_files, 1);
    assert_eq!(load.bytes_read, "needle\0binary".len());
    assert!(load.corpus.is_empty());
    assert!(load.incomplete);
}

#[test]
fn a_result_cap_is_visible_in_the_picker_footer() {
    let mut picker = crate::overlay::OverlayState::new_search_folder(
        std::path::PathBuf::from("/notes"),
        vec![("a.md".into(), "needle\n".repeat(21))],
        false,
    );
    for ch in "needle".chars() {
        picker.push(ch);
    }
    assert_eq!(picker.rows.len(), SearchBudget::default().max_hits_per_file);
    assert!(picker.search_limited);
    assert_eq!(picker.foot_hint(), "results limited   ↵ open   esc close");
}
