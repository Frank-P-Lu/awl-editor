//! tests/table_verbs_journey.rs — the structural table verbs driven the way a
//! WRITER reaches them: real binary, real keymap, the command palette, Enter.
//!
//! The splices have exhaustive unit laws (`markdown::table_edit`) and the
//! action seam has its own (`actions::tests::table_verbs`). Neither one crosses
//! the dispatch path this file does: chord parsing → the palette's own overlay →
//! command activation → `apply_transition` → the sidecar. A verb that worked at
//! every inner seam and was unreachable from the palette would pass all of them
//! and fail here.
//!
//! The capture is pointed at a SEEDED root with the config ladder pinned inside
//! the sandbox (`common::awl`), never the ambient pair — the shot is taken with
//! the palette already dismissed, so no directory is photographed either way.

use std::path::Path;

mod common;
use common::ScratchDir;

/// The fixture's table rows, in source order. Already aligned, so any change in
/// the sidecar's `text` is the verb's own work and not a re-pad.
const FIXTURE_ROWS: &[&str] = &[
    "| Name | V   |",
    "| ---- | --- |",
    "| a    | 100 |",
    "| b    | 2   |",
];

/// `rows` wrapped in the fixture's surrounding prose — the whole document, as
/// the sidecar reports it. The prose on both sides is what proves a verb edited
/// the table and nothing else.
fn document(rows: &[&str]) -> String {
    format!("intro\n\n{}\n\ntail\n", rows.join("\n"))
}

/// Drive one palette journey over the fixture and hand back its sidecar.
///
/// `keys` is a chord spec exactly as `--keys` takes it: the caret is walked to
/// the row under test, `s-p` summons the palette, the command's name is typed
/// and `Enter` activates it.
fn journey(dir: &Path, tag: &str, keys: &str) -> serde_json::Value {
    let note = dir.join("note.md");
    std::fs::write(&note, document(FIXTURE_ROWS)).unwrap();
    let out = dir.join(format!("{tag}.png"));
    let mut cmd = common::awl(dir);
    cmd.env("AWL_CONVENTION_FORCE", "mac")
        .arg("--screenshot")
        .arg(&out)
        .arg("--root")
        .arg(dir)
        .arg("--keys")
        .arg(keys)
        .arg(&note);
    let run = cmd.output().expect("failed to spawn CARGO_BIN_EXE_awl");
    assert!(
        run.status.success(),
        "{tag}: awl exited {}\n{}",
        run.status,
        String::from_utf8_lossy(&run.stderr)
    );
    let json = std::fs::read_to_string(out.with_extension("json")).expect("sidecar exists");
    serde_json::from_str(&json).expect("sidecar parses")
}

/// Walk the caret down `n` lines from the document start.
fn down(n: usize) -> String {
    std::iter::repeat_n("Down", n).collect::<Vec<_>>().join(" ")
}

/// One palette pick per verb, each asserted on the sidecar's own `text`. The
/// caret sits on the first body row (`| a    | 100 |`, line 4) in every case,
/// so the six outcomes differ only by which verb ran.
#[test]
fn every_verb_is_reachable_from_the_palette_and_rewrites_the_source() {
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-table-verbs-journey-{}", std::process::id())),
    );
    let cases: [(&str, &str, &[&str]); 6] = [
        (
            "insert-row-above",
            "I n s e r t space r o w space a b o v e",
            &[
                "| Name | V   |",
                "| ---- | --- |",
                "|      |     |",
                "| a    | 100 |",
                "| b    | 2   |",
            ],
        ),
        (
            "insert-row-below",
            "I n s e r t space r o w space b e l o w",
            &[
                "| Name | V   |",
                "| ---- | --- |",
                "| a    | 100 |",
                "|      |     |",
                "| b    | 2   |",
            ],
        ),
        (
            "insert-column-left",
            "I n s e r t space c o l u m n space l e f t",
            &[
                "|   | Name | V   |",
                "| - | ---- | --- |",
                "|   | a    | 100 |",
                "|   | b    | 2   |",
            ],
        ),
        (
            "insert-column-right",
            "I n s e r t space c o l u m n space r i g h t",
            &[
                "| Name |   | V   |",
                "| ---- | - | --- |",
                "| a    |   | 100 |",
                "| b    |   | 2   |",
            ],
        ),
        (
            "delete-row",
            "D e l e t e space r o w",
            // Column `V` narrows to one character once `100` leaves with its
            // row — the padder's own widths, re-derived from what is left.
            &["| Name | V |", "| ---- | - |", "| b    | 2 |"],
        ),
        (
            "delete-column",
            "D e l e t e space c o l u m n",
            &["| V   |", "| --- |", "| 100 |", "| 2   |"],
        ),
    ];
    for (tag, query, want) in cases {
        // Line 4 is the first body row; `Right` puts the caret inside the
        // first cell's own content rather than on the opening pipe.
        let keys = format!("{} Right Right s-p {query} Enter", down(4));
        let sidecar = journey(&dir, tag, &keys);
        assert_eq!(
            sidecar["text"].as_str(),
            Some(document(want).as_str()),
            "{tag}: the palette pick did not rewrite the table as expected"
        );
        assert!(
            !sidecar["overlay"]["active"].as_bool().unwrap_or(true),
            "{tag}: the palette should have closed on Enter"
        );
        assert_eq!(
            sidecar["notice"],
            serde_json::Value::Null,
            "{tag}: a verb that succeeded raises no notice"
        );
    }
}

/// A verb DECLINED by the table's shape reaches the writer through the sidecar's
/// notice channel — the refusal is a published fact, not a silent no-op, and the
/// document is untouched. Driven from the header row, where Delete row refuses.
#[test]
fn a_refused_verb_publishes_its_reason_and_edits_nothing() {
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-table-verbs-refusal-{}", std::process::id())),
    );
    // Line 2 is the header row.
    let keys = format!("{} Right Right s-p D e l e t e space r o w Enter", down(2));
    let sidecar = journey(&dir, "refusal", &keys);
    assert_eq!(
        sidecar["notice"]["text"].as_str(),
        Some("a table's header row can't be deleted")
    );
    assert_eq!(sidecar["notice"]["kind"].as_str(), Some("sticky"));
    assert_eq!(
        sidecar["text"].as_str(),
        Some(document(FIXTURE_ROWS).as_str()),
        "a refusal leaves the document byte-identical"
    );
}

/// Off a table entirely, the palette pick still answers — the same notice
/// channel, naming what the command needs. (The palette lists these rows on
/// every buffer, so this is the answer a writer actually meets.)
#[test]
fn a_verb_picked_off_a_table_says_what_it_needs() {
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-table-verbs-off-table-{}", std::process::id())),
    );
    // Line 0 is the prose `intro`.
    let sidecar = journey(
        &dir,
        "off-table",
        "s-p I n s e r t space r o w space b e l o w Enter",
    );
    assert_eq!(
        sidecar["notice"]["text"].as_str(),
        Some("put the caret in a table first")
    );
    assert_eq!(
        sidecar["text"].as_str(),
        Some(document(FIXTURE_ROWS).as_str())
    );
}
