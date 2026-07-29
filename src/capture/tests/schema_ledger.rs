//! THE SCHEMA LEDGER — `SCHEMA_VERSION` and its `/N` history table must move
//! together. Carved out of `capture.rs` (item 158) so a test module stops
//! counting against a production file's own size mark, exactly as the rest of
//! `capture::tests` already was.

use super::super::SCHEMA_VERSION;
fn history_rows() -> Vec<u32> {
    let src = include_str!("../../capture.rs");
    let mut rows = Vec::new();
    for line in src.lines() {
        let Some(rest) = line.trim_start().strip_prefix("//") else {
            continue;
        };
        let rest = rest.trim_start_matches('/').trim_start();
        let Some(after) = rest.strip_prefix("`/") else {
            continue;
        };
        let Some(end) = after.find('`') else { continue };
        if let Ok(n) = after[..end].parse::<u32>() {
            rows.push(n);
        }
    }
    rows
}

#[test]
fn schema_version_matches_latest_history_row() {
    let rows = history_rows();
    assert!(
        !rows.is_empty(),
        "no `/N` history rows parsed — has the table's row format changed? \
         (see the CLAIM CONVENTION doc above SCHEMA_VERSION)"
    );
    for w in rows.windows(2) {
        assert!(
            w[1] > w[0],
            "schema history rows not strictly increasing (/{} then /{}) — a \
             duplicate or out-of-order row, almost certainly an unreconciled \
             merge collision; renumber the later row (see CLAIM CONVENTION).",
            w[0],
            w[1]
        );
    }
    let last = *rows.last().unwrap();
    assert_eq!(
        last, SCHEMA_VERSION,
        "SCHEMA_VERSION ({SCHEMA_VERSION}) must equal the LAST history row \
         (/{last}). Bump the const AND append a matching `/N` row together \
         (see the CLAIM CONVENTION doc above SCHEMA_VERSION)."
    );
}
