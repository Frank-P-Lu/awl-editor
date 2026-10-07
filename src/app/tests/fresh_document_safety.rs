//! Fresh-document doors must obey the same save and conflict boundary as open.
use super::*;
use crate::fs::{FileSystem, InMemoryFs};
use std::path::Path;
const MINE: &str = "my unflushed manuscript\n";
fn config() -> Config {
    Config {
        session_restore: Some(false),
        ..Config::empty()
    }
}
fn new_document(app: &mut App, browsed: bool) {
    if browsed {
        app.workspace_state
            .install_overlay_for_test(crate::overlay::OverlayState::new_files(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Some("drafts".into()),
            ));
    }
    let exit = crate::app::schedule::RecordingExit::new();
    app.apply(Action::NewDocument, false, &exit, crate::stats::Door::Menu);
}
#[test]
fn fresh_document_cannot_park_a_conflict_or_save_another_buffer_over_it() {
    let mut failed = Vec::new();
    for browsed in [false, true] {
        let path = Path::new("/probe/draft.md");
        let mem = InMemoryFs::new()
            .with_file(path, "old text\n")
            .with_dir("/probe/drafts");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let mut app = app_on(Some(path.into()), "/probe", config());
            app.document.set_text(MINE);
            mem.write(path, b"other editor text\n").unwrap();
            app.on_focus_gained();
            assert!(app.change_unresolved());
            new_document(&mut app, browsed);
            let active = app.document.buffer().path().map(Path::to_path_buf);
            let text = app.document.buffer().text();
            app.resolve_keep_mine();
            let disk = mem.read(path).unwrap();
            eprintln!("fresh conflict {browsed} active={active:?} text={text:?} disk={disk:?}");
            if active.as_deref() != Some(path) || text != MINE || disk != MINE.as_bytes() {
                failed.push(browsed);
            }
        });
    }
    assert!(
        failed.is_empty(),
        "fresh-document conflict protection failed: {failed:?}"
    );
}
#[test]
fn fresh_document_flushes_departing_edits_before_they_can_be_lost_at_exit() {
    let mut failed = Vec::new();
    for browsed in [false, true] {
        let path = Path::new("/probe/draft.md");
        let mem = InMemoryFs::new()
            .with_file(path, "old text\n")
            .with_dir("/probe/drafts");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let mut app = app_on(Some(path.into()), "/probe", config());
            app.document.set_text(MINE);
            new_document(&mut app, browsed);
            app.flush_documents_for_shutdown();
            drop(app);
            let disk = mem.read(path).unwrap();
            eprintln!("FRESH_DOCUMENT_PROBE exit browsed={browsed} after_exit={disk:?}");
            if disk != MINE.as_bytes() {
                failed.push(browsed);
            }
        });
    }
    assert!(
        failed.is_empty(),
        "departing edits were not saved: {failed:?}"
    );
}

#[test]
fn fresh_document_then_shutdown_cannot_replace_the_original_recovery_manuscript() {
    let mut failed = Vec::new();
    for browsed in [false, true] {
        let path = Path::new("/probe/draft.md");
        let mem = InMemoryFs::new()
            .with_file(path, "old text\n")
            .with_dir("/probe/drafts");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let mut app = app_on(Some(path.into()), "/probe", config());
            app.document.set_text(MINE);
            mem.write(path, b"other editor text\n").unwrap();
            app.on_focus_gained();
            assert_eq!(crate::recovery::read_for(path).unwrap().text, MINE);
            new_document(&mut app, browsed);
            // Exercise the actual owner called by the event-loop callback.
            app.flush_documents_for_shutdown();
            let recovered = crate::recovery::read_for(path).unwrap().text;
            eprintln!("FRESH_DOCUMENT_PROBE shutdown browsed={browsed} recovered={recovered:?}");
            if recovered != MINE {
                failed.push(browsed);
            }
        });
    }
    assert!(
        failed.is_empty(),
        "shutdown replaced recovery text: {failed:?}"
    );
}
#[test]
fn fresh_document_then_take_theirs_resolves_the_original_active_file() {
    let mut failed = Vec::new();
    for browsed in [false, true] {
        let path = Path::new("/probe/draft.md");
        let mem = InMemoryFs::new()
            .with_file(path, "old text\n")
            .with_dir("/probe/drafts");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let mut app = app_on(Some(path.into()), "/probe", config());
            app.document.set_text(MINE);
            mem.write(path, b"other editor text\n").unwrap();
            app.on_focus_gained();
            new_document(&mut app, browsed);
            app.resolve_take_theirs();
            let active = app.document.buffer().path().map(Path::to_path_buf);
            let text = app.document.buffer().text();
            eprintln!(
                "FRESH_DOCUMENT_PROBE take_theirs browsed={browsed} active={active:?} text={text:?}"
            );
            if active.as_deref() != Some(path)
                || text != "other editor text\n"
                || app.change_unresolved()
            {
                failed.push(browsed);
            }
            assert_eq!(mem.read(path).unwrap(), b"other editor text\n");
        });
    }
    assert!(
        failed.is_empty(),
        "resolution targeted the wrong slot: {failed:?}"
    );
}

#[test]
fn conflict_resolutions_and_recovery_refuse_a_mismatched_active_identity() {
    let path = Path::new("/probe/draft.md");
    let mem = InMemoryFs::new().with_file(path, "old text\n");
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut app = app_on(Some(path.into()), "/probe", config());
        app.document.set_text(MINE);
        mem.write(path, b"other editor text\n").unwrap();
        app.on_focus_gained();
        // Deliberately bypass the App transition to exercise each independent
        // identity guard, even if a future caller violates the parking contract.
        app.document.start_fresh_document(PathBuf::from("/probe"));
        app.resolve_keep_mine();
        app.resolve_take_theirs();
        app.write_recovery_record(path);
        assert_eq!(mem.read(path).unwrap(), b"other editor text\n");
        assert_eq!(crate::recovery::read_for(path).unwrap().text, MINE);
        assert_eq!(app.document.buffer().text(), "");
        assert_eq!(app.document.buffer().path(), None);
        assert!(app.change_unresolved());
    });
}

#[test]
fn fresh_document_refuses_unsaved_text_when_autosave_is_disabled_or_fails() {
    use crate::fs::{ScriptedFailure, ScriptedFs, ScriptedOperation};
    for browsed in [false, true] {
        for disabled in [false, true] {
            let path = Path::new("/probe/draft.md");
            let mem = InMemoryFs::new()
                .with_file(path, "old text\n")
                .with_dir("/probe/drafts");
            let fs = ScriptedFs::new(
                mem.clone(),
                ScriptedFailure {
                    operation: ScriptedOperation::Write,
                    ordinal: 1,
                    kind: std::io::ErrorKind::PermissionDenied,
                    reason: "synthetic save refusal",
                },
            );
            crate::fs::with_fs(Arc::new(fs), || {
                let mut config = config();
                config.autosave = Some(!disabled);
                let mut app = app_on(Some(path.into()), "/probe", config);
                app.document.set_text(MINE);
                new_document(&mut app, browsed);
                assert_eq!(app.document.buffer().path(), Some(path));
                assert_eq!(app.document.buffer().text(), MINE);
                assert!(app.is_document_dirty());
                assert_eq!(mem.read(path).unwrap(), b"old text\n");
                assert!(app.frame.notice().text().unwrap().contains("still unsaved"));
            });
        }
    }
}

#[test]
fn repeated_new_document_attempts_cannot_park_an_unsaved_held_scratch() {
    for browsed in [false, true] {
        let mem = InMemoryFs::new().with_dir("/probe/drafts");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let stash = crate::fs::scratch_stash_path();
            mem.write(&stash, b"original scratch\n").unwrap();
            let mut app = app_on(None, "/probe", config());
            app.document.set_text(MINE);
            mem.write(&stash, b"other window scratch\n").unwrap();
            app.autosave_flush();
            for _ in 0..2 {
                new_document(&mut app, browsed);
                assert_eq!(app.document.buffer().text(), MINE);
                assert!(!app.document.buffer().is_unnamed_fresh());
                assert!(app.is_document_dirty());
                assert_eq!(mem.read(&stash).unwrap(), b"other window scratch\n");
            }
        });
    }
}
