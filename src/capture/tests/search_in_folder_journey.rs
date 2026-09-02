//! **"SEARCH IN FOLDER…", DRIVEN ENTIRELY THROUGH `--keys`.** Real chords
//! open the Command Palette, filter to "Search in folder…" (no default
//! chord, palette-only per the brief), type a query, and accept — the exact
//! route a live user takes (`crate::run::ReplaySession::apply_chord`, the
//! seam `--keys` itself runs). The sidecar is read back for the LANDED FILE
//! (`buffers.active`) AND the caret (`cursor.line`/`cursor.col`), proving
//! `Effect::OpenPathAtLine` genuinely does both halves of its job: switch to
//! a DIFFERENT file than the one open at summon time, and land the caret
//! exactly on the match rather than merely the line start.
//!
//! Two files on real disk under a `ScratchDir` (search-folder's own corpus
//! load reads through `crate::fs::active()`, the same seam every buffer
//! open uses): only `notes/todo.md` contains the query, proving the OTHER
//! candidate file's non-matching lines never leak a spurious row.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::config::Config;
use crate::overlay::OverlayKind;
use crate::testscratch::ScratchDir;

fn type_chars(session: &mut crate::run::ReplaySession, text: &str) {
    let spec: String = text
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let chords = crate::keyspec::parse_chords(&spec).expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

fn press_enter(session: &mut crate::run::ReplaySession) {
    let chords = crate::keyspec::parse_chords("Enter").expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

/// Open the palette (Cmd-P) and filter down to "Search in folder…" — a
/// palette-only command, exactly like Go to…'s own `open_goto` helper
/// (`goto_line_jump.rs`) filters to a different row. `searchin` is a
/// subsequence of "Search in folder…" that skips only the space, and is NOT
/// a subsequence of "Search forward"/"Search backward" (no `i`/`n` survive
/// after "search " in either) — asserted below rather than merely assumed.
fn open_search_in_folder(session: &mut crate::run::ReplaySession) {
    let chords = crate::keyspec::parse_chords("s-p").expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
    type_chars(session, "searchin");
    press_enter(session);
    assert_eq!(
        session.journey().card().map(|o| o.kind),
        Some(OverlayKind::SearchFolder),
        "the palette filter must resolve to the Search-in-folder picker alone"
    );
}

#[test]
fn summon_type_enter_lands_the_caret_on_the_match_in_the_matched_file() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!(
            "skipping summon_type_enter_lands_the_caret_on_the_match_in_the_matched_file: no wgpu adapter"
        );
        return;
    }
    let dir = ScratchDir::new(std::env::temp_dir().join(format!(
        "awl_search_in_folder_journey_{}",
        std::process::id()
    )));
    std::fs::create_dir_all(dir.join("notes")).expect("notes dir");
    let todo_text = "line one\nremember the todo item\nline three\n";
    std::fs::write(dir.join("notes/todo.md"), todo_text).expect("write todo.md");
    std::fs::write(dir.join("notes/other.md"), "nothing to see here\n").expect("write other.md");

    // The document open at summon time is a DIFFERENT, unnamed buffer -- the
    // journey below must genuinely SWITCH files, not merely jump within one.
    let mut buffer = Buffer::from_str("(scratch document)\n");
    let corpus: Vec<String> = vec!["notes/todo.md".to_string(), "notes/other.md".to_string()];
    let root = dir.to_path_buf();
    let config = Config::empty();
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    let mut session = crate::run::ReplaySession::new(
        crate::run::ReplayPolicy::ordinary(),
        &mut buffer,
        &corpus,
        &root,
        Some(root.as_path()),
        &config,
        None,
        &mut km,
    );

    open_search_in_folder(&mut session);
    type_chars(&mut session, "todo");
    assert!(
        session
            .journey()
            .card()
            .is_some_and(|o| o.item_strings().len() == 1),
        "only notes/todo.md contains the query -- exactly one hit row: {:?}",
        session.journey().card().map(|o| o.item_strings())
    );
    press_enter(&mut session);
    assert!(
        session.journey().card().is_none(),
        "accepting a hit must close the overlay back to the document"
    );

    let project = crate::run::project_info(&root, &None, None, &config);
    let opts = crate::run::fold_capture_state(&session, project);
    let out = dir.join("landed.png");
    capture_with(&out, session.buffer(), &opts).expect("capture succeeds");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.with_extension("json")).unwrap())
            .expect("sidecar parses");
    drop(session);

    // Canonicalize before joining: macOS aliases the OS temp root
    // (`/tmp` -> `/private/tmp`), the same symlink class
    // `app/apply/overlay_inputs.rs`'s `root_relative` doc names -- the app
    // resolves and reports the canonical spelling, so the fixture's own
    // expectation must match it rather than the raw `temp_dir()` spelling.
    let canonical_root = dir.canonicalize().expect("scratch dir canonicalizes");
    let expected_path = canonical_root.join("notes/todo.md").display().to_string();
    assert_eq!(
        json["buffers"]["active"].as_str(),
        Some(expected_path.as_str()),
        "the sidecar's landed file: {json}"
    );
    let expected_line = 1u64; // 0-based: "remember the todo item"
    let expected_col = "remember the ".chars().count() as u64; // the match's own char column
    assert_eq!(
        json["cursor"]["line"].as_u64(),
        Some(expected_line),
        "landed line: {json}"
    );
    assert_eq!(
        json["cursor"]["col"].as_u64(),
        Some(expected_col),
        "landed col (on the match, not line start): {json}"
    );
}
