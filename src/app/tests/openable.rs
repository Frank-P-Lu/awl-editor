//! ITEM 77 — THE LAW: every open door (a picker selection / C-x b / the
//! daemon's `open` handoff, all routed through `App::load_path`; a CLI/OS-open
//! LAUNCH argument, `App::new`) asks the SAME `crate::openable::classify`
//! capability owner the SAME question, and Text-vs-All `file_visibility` only
//! changes what the Browse picker LISTS, never that verdict.
//!
//! NON-VACUOUS: before item 77, `Buffer::from_file` swallowed a decode
//! failure and returned an EMPTY buffer STILL BOUND to the binary path (`fs.rs`'s
//! `read_to_string` errors on invalid UTF-8, and `Buffer::from_file`'s `Err`
//! arm silently falls back to an empty rope) — so `load_path`/`App::new`
//! against `/proj/logo.png` used to leave `active.buffer.path()` == `Some("/proj/logo.png")`,
//! failing every assertion below that the refused path never becomes the
//! active document. Reverting the `load_path`/`App::new` gating added in this
//! round (while leaving `crate::openable` itself in place) reproduces exactly
//! that failure.

use super::*;

/// A binary PNG's real magic bytes (signature + an embedded NUL) — the same
/// shape `crate::openable::tests` uses, here exercised through the live App
/// doors instead of the pure classifier directly.
const PNG_BYTES: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00];

fn seeded_fs() -> crate::fs::InMemoryFs {
    use crate::fs::FileSystem;
    let mem = crate::fs::InMemoryFs::new();
    // A supported UNUSUAL-text file: an unfamiliar extension, real prose bytes
    // — NOT an extension allow-list, see `crate::openable`'s module doc.
    mem.write(
        std::path::Path::new("/proj/notes.xyzzy"),
        b"plain prose, odd extension\n",
    )
    .unwrap();
    mem.write(std::path::Path::new("/proj/logo.png"), PNG_BYTES)
        .unwrap();
    mem
}

/// DOOR 1 — the CLI/OS-open LAUNCH argument (`App::new`): a supported
/// unusual-text file opens as the active document; a binary file is refused
/// and the app falls back to the ordinary no-argument scratch buffer, with
/// the refusal named in a sticky notice — never a phantom buffer bound to
/// the binary path.
#[test]
fn cli_launch_door_opens_text_and_refuses_binary() {
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(seeded_fs()));

    let app = app_on(
        Some(PathBuf::from("/proj/notes.xyzzy")),
        "/proj",
        Config::empty(),
    );
    assert_eq!(
        app.document.buffer().path(),
        Some(std::path::Path::new("/proj/notes.xyzzy")),
        "a supported unusual-text CLI argument opens"
    );

    let refused = app_on(
        Some(PathBuf::from("/proj/logo.png")),
        "/proj",
        Config::empty(),
    );
    assert_eq!(
        refused.document.buffer().path(),
        None,
        "a refused CLI launch never binds the active buffer to the binary path"
    );
    assert_eq!(
        refused.frame.notice_text(),
        Some("PNG \u{b7} not editable in awl"),
        "the refusal names the type, calmly"
    );
}

/// DOOR 2 — `App::load_path` (picker selection / C-x b / the daemon's `open`
/// handoff all converge here — see `App::handle_daemon_event`): the SAME
/// verdict, and a refusal leaves the remembered context — the active folder
/// AND the active document — completely INTACT.
#[test]
fn load_path_door_opens_text_and_refuses_binary_leaving_context_intact() {
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(seeded_fs()));
    let mut app = app_on(
        Some(PathBuf::from("/proj/notes.xyzzy")),
        "/proj",
        Config::empty(),
    );
    let before_path = app.document.buffer().path().map(|p| p.to_path_buf());
    let before_root = app.project_location.root.clone();

    app.load_path(PathBuf::from("/proj/logo.png"));
    assert_eq!(
        app.document.buffer().path().map(|p| p.to_path_buf()),
        before_path,
        "a refused open leaves the active DOCUMENT intact"
    );
    assert_eq!(
        app.project_location.root, before_root,
        "a refused open leaves the active FOLDER intact"
    );
    assert_eq!(
        app.frame.notice_text(),
        Some("PNG \u{b7} not editable in awl")
    );

    // The SAME door opens a supported unusual-text file from a bare scratch
    // start too.
    let mut fresh = app_on(None, "/proj", Config::empty());
    fresh.load_path(PathBuf::from("/proj/notes.xyzzy"));
    assert_eq!(
        fresh.document.buffer().path(),
        Some(std::path::Path::new("/proj/notes.xyzzy")),
        "a supported unusual-text file opens via load_path (picker/C-x b/daemon)"
    );
}

// The daemon's `open` handoff (`App::handle_daemon_event`, `src/app/daemon.rs`)
// is NOT a separate verdict: its ENTIRE body opens with
// `self.load_path(path);` (the very first statement — see that file) before
// anything else runs, so `load_path_door_opens_text_and_refuses_binary_leaving_context_intact`
// above already exercises the exact call the daemon makes. A dedicated test
// double here would just re-assert the same fn with extra plumbing (a mock
// Unix socket) for zero additional coverage.

/// Text vs All `file_visibility` changes ONLY what the Browse picker LISTS —
/// never the openable verdict. Toggled both ways against the SAME two files,
/// both through the pure classifier and through the live `load_path` door.
#[test]
fn file_visibility_never_changes_the_openable_verdict() {
    let _g_fs = crate::fs::FsGuard::install(std::sync::Arc::new(seeded_fs()));
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();

    for all in [false, true] {
        crate::file_visibility::set_all_on(all);
        assert!(
            crate::openable::classify(std::path::Path::new("/proj/notes.xyzzy")).is_text(),
            "file_visibility={all}: the unusual-text verdict is unchanged"
        );
        assert!(
            !crate::openable::classify(std::path::Path::new("/proj/logo.png")).is_text(),
            "file_visibility={all}: the binary verdict is unchanged"
        );

        let mut app = app_on(
            Some(PathBuf::from("/proj/notes.xyzzy")),
            "/proj",
            Config::empty(),
        );
        app.load_path(PathBuf::from("/proj/logo.png"));
        assert_eq!(
            app.document.buffer().path(),
            Some(std::path::Path::new("/proj/notes.xyzzy")),
            "file_visibility={all}: still refused, still leaves the prior document active"
        );
    }
    crate::file_visibility::set_all_on(saved);
}
