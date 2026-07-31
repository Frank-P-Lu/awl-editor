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

// ── ITEM 187 — CAPTURE.md's reservation header must equal the const ────────
//
// CAPTURE.md's "## The sidecar JSON" header is the RESERVATION TABLE: the
// artefact a worker reads to answer "which schema number do I take next".
// Nothing else reads it, so it drifted a full round behind `SCHEMA_VERSION`
// (item 187) with no law noticing. The three numbers it prints are not an
// independent design — `schema_plain`/`schema_timeline`/`schema_held` above
// are `SCHEMA_VERSION`, `+1`, `+2` — so the header is DERIVED from the same
// const the history rows are, and must be checked against it directly rather
// than only against itself.

/// The exact header line CAPTURE.md must carry for the current const.
fn expected_header() -> String {
    format!(
        "## The sidecar JSON — schema `awl-capture/{SCHEMA_VERSION}` \
         (`/{}` timeline, `/{}` held)",
        SCHEMA_VERSION + 1,
        SCHEMA_VERSION + 2
    )
}

#[test]
fn capture_md_header_matches_schema_version() {
    let doc = crate::embedded_docs::CAPTURE_MD;
    let found = doc
        .lines()
        .find(|line| line.starts_with("## The sidecar JSON — schema `awl-capture/"))
        .unwrap_or_else(|| {
            panic!(
                "CAPTURE.md has no \"## The sidecar JSON — schema `awl-capture/…`\" \
                 header — has it been renamed or removed? The reservation table \
                 workers read to pick the next schema number must still exist."
            )
        });
    let expected = expected_header();
    assert_eq!(
        found, expected,
        "CAPTURE.md's schema reservation header has drifted from \
         `capture::SCHEMA_VERSION` ({SCHEMA_VERSION}) — this is the table a \
         worker reads to learn which schema number is free, so a stale header \
         hands out an already-taken number. Replace the header line with \
         exactly:\n{expected}"
    );
}
