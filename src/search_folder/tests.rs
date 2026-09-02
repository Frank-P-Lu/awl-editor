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

// ── load_corpus: the bounded, injectable file loader ───────────────────────

#[test]
fn load_corpus_stops_at_max_files() {
    let files: Vec<String> = (0..10).map(|i| format!("f{i}.md")).collect();
    let budget = SearchBudget {
        max_files: 3,
        ..tight_budget()
    };
    let corpus = load_corpus(&files, &budget, |p| Some(format!("content of {p}")));
    assert_eq!(corpus.len(), 3);
}

#[test]
fn load_corpus_skips_a_file_over_the_per_file_cap() {
    let files = vec!["small.md".to_string(), "huge.md".to_string()];
    let budget = SearchBudget {
        max_file_bytes: 10,
        ..tight_budget()
    };
    let corpus = load_corpus(&files, &budget, |p| {
        if p == "huge.md" {
            Some("x".repeat(1000))
        } else {
            Some("tiny".to_string())
        }
    });
    assert_eq!(corpus.len(), 1);
    assert_eq!(corpus[0].0, "small.md");
}

#[test]
fn load_corpus_stops_once_the_total_byte_budget_is_spent() {
    let files: Vec<String> = (0..10).map(|i| format!("f{i}.md")).collect();
    let budget = SearchBudget {
        max_total_bytes: 25,
        max_file_bytes: 1_000_000,
        ..tight_budget()
    };
    let corpus = load_corpus(&files, &budget, |_| Some("a".repeat(10)));
    assert!(
        corpus.len() < 10,
        "the loader must stop before exhausting every candidate once the byte budget is spent"
    );
    assert!(!corpus.is_empty());
}

#[test]
fn load_corpus_skips_an_unreadable_file_rather_than_aborting() {
    let files = vec![
        "ok.md".to_string(),
        "binary.png".to_string(),
        "ok2.md".to_string(),
    ];
    let corpus = load_corpus(&files, &tight_budget(), |p| {
        if p == "binary.png" {
            None
        } else {
            Some("text".to_string())
        }
    });
    assert_eq!(corpus.len(), 2);
    assert!(corpus.iter().all(|(p, _)| p != "binary.png"));
}
