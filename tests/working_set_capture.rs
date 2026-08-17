//! tests/working_set_capture.rs — THE VISIBLE WORKING SET at the one capture
//! door that can see it, on the real binary.
//!
//! The margin's stack is App-owned, so tier 1 classifies its switching
//! Unsupported (`docs/harness-reach.md`) and only `--screenshot-app` can drive
//! it; and a live-App capture is hermetic, so its sandbox holds nothing to open
//! a second file FROM unless `--seed-tree` carries a project in. Both halves
//! have to be true at once for the surface to be witnessable at all, which is
//! why this runs the whole real chain — `parse_args` → sandbox install → config
//! load → `App::new` → real chords through the real keymap → the capture
//! pipeline — rather than asserting against a hand-built `CaptureOpts`.
//!
//! What it pins is the surface's ONE contract: switching files moves which row
//! is current and LEAVES THE DRAWN ORDER ALONE. A stable-order stack and an MRU
//! list are indistinguishable from any single frame, so the law is a sequence.

use std::path::{Path, PathBuf};

mod common;
use common::ScratchDir;

/// The fixture project: two files directly under the root and one NESTED, so a
/// row that reported its leaf instead of its root-relative path is
/// distinguishable — and opened in an order that is not alphabetical, so is a
/// row list that had been sorted.
fn arrange(dir: &Path) -> PathBuf {
    let notes = dir.join("notes");
    std::fs::create_dir_all(notes.join("journal")).unwrap();
    std::fs::write(notes.join("opening.md"), "# Opening\n\nthe first file.\n").unwrap();
    std::fs::write(notes.join("ledger.md"), "# Ledger\n\nthe second file.\n").unwrap();
    std::fs::write(
        notes.join("journal").join("field-notes.md"),
        "# Field notes\n\nnested under journal.\n",
    )
    .unwrap();
    std::fs::write(dir.join("awl.toml"), "theme = \"Alabaster\"\n").unwrap();
    notes
}

/// Drive one live-`App` capture over the seeded project and hand back its
/// `buffers` block. The config is the SEEDED one and the root is the SEEDED
/// one, never the ambient pair — the margin photographs filenames, so a shot
/// pointed at a real directory would put a real path in a PNG.
fn buffers(dir: &Path, tag: &str, keys: &str) -> serde_json::Value {
    let notes = dir.join("notes");
    let out = dir.join(format!("{tag}.png"));
    // Through the shared spawn owner, which pins the config ladder inside the
    // sandbox: a direct spawn inherits it and can read the developer's own
    // config. The explicit `--config` below is the seeded fixture's, so both
    // rungs land inside this test's directory.
    let mut cmd = common::awl(dir);
    cmd.env("AWL_CONVENTION_FORCE", "mac")
        .arg("--screenshot-app")
        .arg(&out)
        .arg("--seed-tree")
        .arg(dir)
        .arg("--config")
        .arg(dir.join("awl.toml"))
        .arg("--root")
        .arg(&notes)
        .arg(notes.join("opening.md"));
    if !keys.is_empty() {
        cmd.arg("--keys").arg(keys);
    }
    let run = cmd.output().expect("failed to spawn CARGO_BIN_EXE_awl");
    assert!(
        run.status.success(),
        "{tag}: awl exited {}\n{}",
        run.status,
        String::from_utf8_lossy(&run.stderr)
    );
    let json = std::fs::read_to_string(out.with_extension("json")).expect("sidecar exists");
    let v: serde_json::Value = serde_json::from_str(&json).expect("sidecar parses");
    assert_eq!(
        v["driver"],
        serde_json::json!("live-app"),
        "{tag}: only a live-App capture can see the working set"
    );
    v["buffers"].clone()
}

/// Go to (`Cmd-O`), filtered by a prefix unique to each fixture file, accepted.
const TO_LEDGER: &str = "Cmd-o l e d Enter";
const TO_FIELD: &str = "Cmd-o f i e Enter";
const TO_OPENING: &str = "Cmd-o o p e n Enter";

#[test]
fn switching_files_moves_the_active_row_and_never_reorders_the_stack() {
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-working-set-capture-{}", std::process::id())),
    );
    arrange(&dir);

    // ONE FILE: no stack at all. The anti-vacuity half — without it every
    // assertion below is satisfiable by a block that always lists the registry.
    let lone = buffers(&dir, "one-file", "");
    assert_eq!(
        lone["files"],
        serde_json::json!([]),
        "a single open file draws no stack, so no rows are reported"
    );
    assert_eq!(
        lone["active_index"],
        serde_json::Value::Null,
        "no stack means no active row to name"
    );

    // THREE FILES, in the order they were opened, the nested one reading by its
    // root-relative path.
    let opened = buffers(&dir, "three-files", &format!("{TO_LEDGER} {TO_FIELD}"));
    let order = serde_json::json!(["opening.md", "ledger.md", "journal/field-notes.md"]);
    assert_eq!(
        opened["files"], order,
        "the stack draws the project's open files in stable open order"
    );
    assert_eq!(
        opened["active_index"],
        serde_json::json!(2),
        "the last-opened file is the active row"
    );

    // SWITCH BACK TO THE FIRST — the switch an MRU list answers by moving the
    // row. The drawn order must be byte-for-byte what it was.
    let switched = buffers(
        &dir,
        "switched",
        &format!("{TO_LEDGER} {TO_FIELD} {TO_OPENING}"),
    );
    assert_eq!(
        switched["files"], order,
        "switching reordered the stack: {} -> {}",
        opened["files"], switched["files"]
    );
    assert_eq!(
        switched["active_index"],
        serde_json::json!(0),
        "the active row followed the switch to the first file"
    );
    assert_ne!(
        switched["active_index"], opened["active_index"],
        "the switch was non-vacuous — the active row genuinely moved"
    );

    // NO ROW LEAKS A PATH. The margin photographs filenames; a row label is
    // root-relative by construction, and this is the assertion that keeps it so.
    for arm in [&opened, &switched] {
        for label in arm["files"].as_array().expect("files is an array") {
            let label = label.as_str().expect("a row label is a string");
            assert!(
                !label.starts_with('/') && !label.contains(&*dir.to_string_lossy()),
                "row label {label:?} carries an absolute path"
            );
        }
    }
}
