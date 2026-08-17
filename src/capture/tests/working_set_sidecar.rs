//! `buffers.files` / `buffers.active_index` — the VISIBLE WORKING SET as the
//! sidecar reports it, through the one redacting writer.
//!
//! The margin's stack was readable only from the PNG, so no oracle could state
//! the surface's central contract: switching files moves which row is current
//! and LEAVES THE DRAWN ORDER ALONE (stable open order, not MRU). These laws
//! assert both halves against a real [`crate::workingset::WorkingSet`] rather
//! than against literals the serializer could also satisfy by accident.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::testscratch::ScratchDir;
use crate::workingset::WorkingSet;
use std::path::{Path, PathBuf};

const ROOT: &str = "/proj";

/// Three files in a fixed OPEN order, one of them nested — so a serializer that
/// dropped the parent, or re-ordered by anything, is distinguishable. `journal/
/// field.md` is opened SECOND and activated LAST, which makes "stable order"
/// and "most recently used" disagree about the whole list.
fn seeded() -> WorkingSet {
    let mut ws = WorkingSet::default();
    for rel in ["a.md", "journal/field.md", "b.md"] {
        let path = PathBuf::from(ROOT).join(rel);
        ws.open(
            crate::buffers::BufferKey::path(&path),
            Some(path),
            PathBuf::from(ROOT),
        );
    }
    ws
}

/// The sidecar's own answer for `ws`, captured through the REAL writer.
fn capture_buffers(dir: &Path, tag: &str, ws: &WorkingSet) -> serde_json::Value {
    let out = dir.join(format!("{tag}.png"));
    let opts = CaptureOpts {
        working_set: ws.stack_rows(Path::new(ROOT)),
        ..CaptureOpts::default()
    };
    capture_with(&out, &Buffer::from_str("hello\n"), &opts).expect("capture");
    let text = std::fs::read_to_string(out.with_extension("json")).expect("sidecar");
    let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    v["buffers"].clone()
}

/// WHICH row the working set itself says is current, expressed the way the
/// sidecar expresses it: a position within the DRAWN GROUP, not within every
/// open file. Derived here rather than asserted as a literal, because the naive
/// serialization — reporting `WorkingSet::active_index` verbatim — agrees with
/// the right answer whenever the group happens to be the whole set, and this is
/// the oracle that has to disagree with it.
fn group_active(ws: &WorkingSet) -> Option<usize> {
    let active = ws.active_index()?;
    ws.group(Path::new(ROOT))
        .iter()
        .position(|&at| at == active)
}

#[test]
fn the_sidecar_working_set_is_the_real_one_in_stable_open_order() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!("skipping working-set sidecar law: no wgpu adapter");
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_working_set_sidecar_{}", std::process::id())),
    );
    let mut ws = seeded();

    let opened = capture_buffers(&dir, "opened", &ws);
    let labels: Vec<String> = ws
        .stack_rows(Path::new(ROOT))
        .iter()
        .map(|row| format!("{}{}", row.parent, row.leaf))
        .collect();
    assert_eq!(
        opened["files"],
        serde_json::json!(labels),
        "the reported rows are the working set's own, root-relative and in open order"
    );
    assert_eq!(
        opened["files"],
        serde_json::json!(["a.md", "journal/field.md", "b.md"]),
        "a nested file reads by its root-relative path, not its leaf"
    );
    assert_eq!(
        opened["active_index"],
        serde_json::json!(group_active(&ws).expect("a file is active")),
        "the active row is the one the working set says it is"
    );
    assert_eq!(
        opened["active_index"],
        serde_json::json!(2),
        "the LAST-opened file is the active one before any switch"
    );

    // PATHS NEVER LEAK. A row label is root-relative by construction, so no
    // absolute path — the ambient machine's or the fixture's — can ride out on
    // this block even before `redact` sees it.
    for label in opened["files"].as_array().expect("files is an array") {
        let label = label.as_str().expect("a row label is a string");
        assert!(
            !label.starts_with('/') && !label.contains(ROOT),
            "row label {label:?} carries an absolute path"
        );
    }

    // THE ORDER CONTRACT. Switch to the FIRST-opened file — the switch an MRU
    // list would answer by moving it to an end — and require the drawn order to
    // be untouched while the active row moves.
    assert!(ws.set_active(0), "slot 0 exists");
    let switched = capture_buffers(&dir, "switched", &ws);
    assert_eq!(
        switched["files"], opened["files"],
        "switching files must not reorder the stack: {} -> {}",
        opened["files"], switched["files"]
    );
    assert_eq!(
        switched["active_index"],
        serde_json::json!(group_active(&ws).expect("a file is active")),
        "the active row followed the switch"
    );
    assert_ne!(
        switched["active_index"], opened["active_index"],
        "the switch was non-vacuous — the active row genuinely moved"
    );
}

/// THE SINGLE-FILE ARM, which is the reason the whole surface can be additive:
/// with one file open there is no stack, so the block reports no rows and no
/// active row. Without this the first law above is satisfiable by a serializer
/// that always emits the whole registry.
#[test]
fn one_open_file_reports_no_working_set_rows_at_all() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!("skipping single-file working-set sidecar law: no wgpu adapter");
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_working_set_one_{}", std::process::id())),
    );
    let mut ws = WorkingSet::default();
    let path = PathBuf::from(ROOT).join("only.md");
    ws.open(
        crate::buffers::BufferKey::path(&path),
        Some(path),
        PathBuf::from(ROOT),
    );
    let lone = capture_buffers(&dir, "lone", &ws);
    assert_eq!(
        lone["files"],
        serde_json::json!([]),
        "one open file draws no stack, so the sidecar reports no rows"
    );
    assert_eq!(
        lone["active_index"],
        serde_json::Value::Null,
        "no stack means no active row to name"
    );
}
