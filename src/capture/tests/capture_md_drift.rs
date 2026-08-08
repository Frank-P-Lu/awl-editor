//! THE CAPTURE.md ENUMERATION LAWS. `CAPTURE.md` is the harness contract, and
//! two of its sidecar-field rows enumerate a set the CODE owns:
//!
//! * `overlay.mode`'s value set — owned by [`crate::overlay::OverlayKind::as_str`]
//!   over `OverlayKind::ALL`. The doc had 12 values against a 21-variant enum,
//!   and one of the 12 (`outline`) was not a mode at all: it is a settings
//!   toggle key and a separate sidecar object, so a reader filtering captures on
//!   `mode == "outline"` was waiting for a value nothing can emit.
//! * `project`'s field set — owned by the sidecar writer itself. The doc named
//!   four of the seven keys it emits, so three verifiable facts
//!   (`default_folder`, `workspace`, `keymap_flavor` — each added precisely so a
//!   `--config` launch could be checked from the sidecar with no flags) were
//!   undiscoverable from the document that exists to list them.
//!
//! THE DOC CONVENTION both laws read: a closed value or field set is written as
//! a slash-joined run of code-ticked tokens — `` `a`/`b`/`c` `` — and the LONGEST
//! such run in the row is that row's enumeration. It is the spelling the `mode`
//! row already used, it is unambiguous against the surrounding prose (which
//! ticks single tokens, never joins them with slashes), and it means the doc
//! stays readable rather than growing a machine-only marker.

use super::super::*;
use crate::overlay::OverlayKind;

/// The row of `CAPTURE.md`'s sidecar-field table whose first cell is
/// `` `name` ``. Panics rather than returning `None`: a missing row is drift of
/// exactly the kind these laws exist to catch, and skipping it silently is how a
/// law goes vacuous.
fn field_row(name: &str) -> &'static str {
    let want = format!("| `{name}`");
    crate::embedded_docs::CAPTURE_MD
        .lines()
        .find(|l| l.starts_with(&want))
        .unwrap_or_else(|| {
            panic!(
                "CAPTURE.md's sidecar-field table has no `{name}` row — the \
                 table's row spelling changed, or the field was renamed \
                 without the document following"
            )
        })
}

/// The LONGEST slash-joined run of code-ticked tokens in `row`, as the tokens
/// themselves. `` `a`/`b`/`c` `` yields `[a, b, c]`; a lone `` `x` `` in prose is
/// a run of one and loses to any real enumeration.
fn ticked_run(row: &str) -> Vec<String> {
    let mut best: Vec<String> = Vec::new();
    let bytes = row.as_bytes();
    let mut i = 0usize;
    while i < row.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        // Walk a maximal `tok`(/`tok`)* chain starting here.
        let mut run: Vec<String> = Vec::new();
        let mut j = i;
        loop {
            if bytes.get(j) != Some(&b'`') {
                break;
            }
            let Some(close) = row[j + 1..].find('`') else {
                break;
            };
            run.push(row[j + 1..j + 1 + close].to_string());
            j = j + 1 + close + 1;
            if bytes.get(j) == Some(&b'/') {
                j += 1;
            } else {
                break;
            }
        }
        if run.len() > best.len() {
            best = run;
        }
        i = j.max(i + 1);
    }
    best
}

/// THE MODE LAW. `overlay.mode`'s documented value set is exactly what
/// `OverlayKind::as_str` can emit — no missing mode a reader cannot discover,
/// and no phantom value nothing produces.
///
/// Enrolled from `OverlayKind::ALL`, so a new picker kind fails this law the day
/// it is added rather than the day someone re-reads the document.
#[test]
fn capture_md_documents_every_overlay_mode() {
    let _g = crate::testlock::serial();
    let documented: std::collections::BTreeSet<String> =
        ticked_run(field_row("overlay")).into_iter().collect();
    let real: std::collections::BTreeSet<String> = OverlayKind::ALL
        .iter()
        .map(|k| k.as_str().to_string())
        .collect();
    let missing: Vec<&String> = real.difference(&documented).collect();
    let phantom: Vec<&String> = documented.difference(&real).collect();
    assert!(
        missing.is_empty() && phantom.is_empty(),
        "CAPTURE.md's `overlay.mode` value set has drifted from \
         `OverlayKind::as_str`.\n  undocumented modes: {missing:?}\n  \
         documented values no kind emits: {phantom:?}\n  (the row's enumeration \
         is the longest `a`/`b`/`c` run of ticked tokens in it)"
    );
}

/// THE PROJECT LAW. The `project` row names exactly the keys the sidecar writer
/// emits for that object — read out of the writer's own JSON rather than out of
/// a second list someone has to remember to grow. (`sidecar::project_json` is
/// `pub(super)` for this one reader and no other.)
///
/// The oracle is the emitted TEXT, not `ProjectInfo`'s fields: what the document
/// promises a reader is what the file contains, and the two could part company
/// (a field held back from the sidecar, or a key composed from more than one
/// field) without either being a bug.
#[test]
fn capture_md_documents_every_project_field() {
    let _g = crate::testlock::serial();
    let opts = CaptureOpts {
        project: Some(crate::capture::opts::ProjectInfo {
            root: std::path::PathBuf::from("/tmp/awl-project-field-law"),
            name: "awl-project-field-law".to_string(),
            branch: Some("main".to_string()),
            dirty: true,
            default_folder: Some(std::path::PathBuf::from("/tmp/awl-project-field-law/notes")),
            workspace: Some(std::path::PathBuf::from("/tmp")),
            keymap_flavor: "native",
        }),
        ..CaptureOpts::default()
    };
    let json = crate::capture::sidecar::project_json(&opts);
    let value: serde_json::Value = serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("the project block is not valid JSON: {e}\n{json}"));
    let emitted: std::collections::BTreeSet<String> = value
        .as_object()
        .expect("a fully-populated project block is a JSON object")
        .keys()
        .cloned()
        .collect();
    assert!(
        emitted.len() > 1,
        "the project block emitted {} key(s) — the fixture above is supposed to \
         populate every one, so this law would be checking nothing",
        emitted.len()
    );
    let documented: std::collections::BTreeSet<String> =
        ticked_run(field_row("project")).into_iter().collect();
    assert_eq!(
        documented, emitted,
        "CAPTURE.md's `project` row and the sidecar writer disagree about that \
         object's keys (the row's enumeration is the longest `a`/`b`/`c` run of \
         ticked tokens in it)"
    );
}
