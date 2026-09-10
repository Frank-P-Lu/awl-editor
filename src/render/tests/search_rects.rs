//! Search geometry equivalence and work bounds.

use super::super::*;
use super::{headless_pipeline, view};

#[test]
fn intersecting_rows_matches_the_exhaustive_column_filter() {
    let rows: Vec<VisualRow> = [(0, 4), (5, 9), (10, 14)]
        .into_iter()
        .map(|(start_col, end_col)| VisualRow {
            line_top: 0.0,
            line_height: 1.0,
            start_col,
            end_col,
            xs: Vec::new(),
        })
        .collect();
    for start in 0..18 {
        for end in 0..18 {
            let got = super::super::rects::intersecting_rows(&rows, start, end);
            let want: Vec<_> = rows
                .iter()
                .enumerate()
                .filter_map(|(i, row)| (row.end_col >= start && row.start_col <= end).then_some(i))
                .collect();
            assert_eq!(got.collect::<Vec<_>>(), want, "{start}..{end}");
        }
    }
}

/// SEARCH WORK-COUNT LAW: a dense match roster may emit one wash per match,
/// but it gets ONE shaped-row gather for the frame. The source check names the
/// owner and counts its actual gather call, so putting a batched helper beside
/// a restored per-match `range_rects` loop cannot satisfy the law.
#[test]
fn search_matches_share_one_visible_row_gather() {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/render/rects/ranges.rs"),
    )
    .expect("search rectangle owner must be readable");
    let start = source
        .find("fn search_match_rects(")
        .expect("search rectangle owner must exist");
    let body = &source[start
        ..source[start..]
            .find("fn search_no_matches(")
            .expect("search rectangle owner must end at search_no_matches")
            + start];
    assert!(
        body.contains("visible_lines_for_ranges(&self.search_matches)"),
        "search must gather the visible line set from its complete match roster"
    );
    assert_eq!(
        body.matches("visual_rows_for_lines(&lines)").count(),
        1,
        "search must gather shaped rows once per frame, never once per match"
    );
    assert!(
        body.contains("range_rects_from_rows((a, b), &rows_by_line)"),
        "each match must resolve against the shared gathered rows"
    );
}

#[test]
fn search_match_geometry_is_differentially_exact_and_work_is_bounded() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping bounded search geometry law: no wgpu adapter");
        return;
    };
    let text = "é the river moves calmly. ".repeat(180);
    let needle = "the";
    let matches: Vec<_> = text
        .match_indices(needle)
        .map(|(byte, _)| {
            let start = text[..byte].chars().count();
            ((0, start), (0, start + needle.chars().count()))
        })
        .collect();
    for width in [360.0, 600.0] {
        for scroll in [ScrollPos::default(), ScrollPos::at_row(9)] {
            for count in [0, 1, matches.len()] {
                p.set_size(width, 800.0);
                let mut v = view(&text, 0, 0);
                v.search_active = true;
                v.search_matches = matches[..count].to_vec();
                v.scroll = scroll;
                p.set_view(&v);
                p.visible_row_gathers.set(0);
                p.reset_search_rect_work();
                let actual = p.search_match_rects();
                let gathers = p.visible_row_gathers.get();
                let visits = p.search_rect_work();
                let reference: Vec<_> = v
                    .search_matches
                    .iter()
                    .flat_map(|&(a, b)| p.range_rects(a, b))
                    .collect();
                assert_eq!(
                    actual, reference,
                    "width={width} scroll={scroll:?} count={count}"
                );
                assert_eq!(
                    gathers,
                    usize::from(count > 0),
                    "width={width} scroll={scroll:?} count={count}"
                );
                if count == 0 {
                    assert_eq!(visits, 0, "empty search must not visit rows");
                } else if count > 1 {
                    assert!(
                        !actual.is_empty(),
                        "dense search must paint visible matches"
                    );
                    assert!(
                        visits <= count * 2,
                        "width={width} scroll={scroll:?}: {visits} row visits for {count} matches"
                    );
                }
            }
        }
    }
}

#[test]
fn offscreen_search_matches_do_not_gather_rows() {
    let _g = crate::testlock::serial();
    let mut p = headless_pipeline().expect("offscreen search law requires a GPU");
    p.set_size(600.0, 400.0);
    let text = "the river moves calmly.\n".repeat(120);
    let mut v = view(&text, 0, 0);
    v.search_active = true;
    v.search_matches = vec![((110, 0), (110, 3)), ((115, 0), (115, 3))];
    p.set_view(&v);
    p.visible_row_gathers.set(0);
    p.reset_search_rect_work();
    assert!(p.search_match_rects().is_empty());
    assert_eq!(p.visible_row_gathers.get(), 0);
    assert_eq!(p.search_rect_work(), 0);
}
