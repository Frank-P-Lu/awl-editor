//! Exit requests must preserve every owned manuscript, including parked ones.
use super::*;
use crate::fs::{FileSystem, InMemoryFs, ScriptedFailure, ScriptedFs, ScriptedOperation};
use std::path::Path;

const FIRST: &str = "earlier manuscript\n";
const LATEST: &str = "latest manuscript 日本語\n";
const OTHER: &str = "another editor's text\n";

#[derive(Clone, Copy, Debug)]
enum Door {
    Quit,
    WindowClose,
}
const DOORS: [Door; 2] = [Door::Quit, Door::WindowClose];

fn request(app: &mut App, door: Door) -> bool {
    let exit = crate::app::schedule::RecordingExit::new();
    let accepted = match door {
        Door::Quit => app.apply(Action::Quit, false, &exit, crate::stats::Door::Menu),
        Door::WindowClose => app.request_document_exit(&exit),
    };
    assert_eq!(accepted, exit.exit_requested(), "{door:?}");
    accepted
}

fn config(autosave: bool) -> Config {
    Config {
        autosave: Some(autosave),
        session_restore: Some(false),
        ..Config::empty()
    }
}

fn snapshot(
    app: &App,
) -> (
    Option<crate::buffers::BufferKey>,
    Vec<(crate::buffers::BufferKey, String)>,
) {
    let active = app.document.active_key();
    let text = app
        .document
        .open_entry_keys()
        .into_iter()
        .map(|key| {
            let text = if active.as_ref() == Some(&key) {
                app.document.buffer().text()
            } else {
                app.document.parked_text(&key).unwrap()
            };
            (key, text)
        })
        .collect();
    (active, text)
}

fn fault(mem: InMemoryFs, operation: ScriptedOperation, repeating: bool) -> ScriptedFs {
    let fs = ScriptedFs::new(
        mem,
        ScriptedFailure {
            operation,
            ordinal: 1,
            kind: std::io::ErrorKind::PermissionDenied,
            reason: "synthetic preservation refusal",
        },
    );
    if repeating { fs.repeating() } else { fs }
}

fn conflict(app: &mut App, mem: &InMemoryFs, path: &Path, deleted: bool) {
    app.document.set_text(FIRST);
    if deleted {
        mem.remove_file(path).unwrap();
    } else {
        mem.write(path, OTHER.as_bytes()).unwrap();
    }
    app.manual_save();
    assert!(app.change_unresolved());
    assert_eq!(crate::recovery::read_for(path).unwrap().text, FIRST);
    app.document.set_text(LATEST);
}

#[test]
fn quit_safety_autosave_off_preserves_active_and_parked_text_on_every_request() {
    let _guard = crate::testlock::serial();
    for door in DOORS {
        for parked in [false, true] {
            let a = Path::new("/probe/a.md");
            let b = Path::new("/probe/b.md");
            let mem = InMemoryFs::new()
                .with_file(a, "original a\n")
                .with_file(b, "original b\n");
            let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
            let mut app = app_on(Some(a.into()), "/probe", config(false));
            app.document.set_text(LATEST);
            if parked {
                assert!(app.load_path(b.into()));
            }
            let before = snapshot(&app);
            for _ in 0..3 {
                assert!(!request(&mut app, door));
                assert_eq!(snapshot(&app), before);
                assert!(
                    app.frame
                        .notice()
                        .text()
                        .unwrap()
                        .contains("Unsaved changes")
                );
                assert_eq!(mem.read(a).unwrap(), b"original a\n");
                assert_eq!(mem.read(b).unwrap(), b"original b\n");
            }
            if parked {
                assert!(app.load_path(a.into()));
            }
            app.manual_save();
            assert!(
                request(&mut app, door),
                "explicit successful Save permits exit"
            );
            assert_eq!(mem.read(a).unwrap(), LATEST.as_bytes());
        }
    }
}

#[test]
fn quit_safety_persistent_original_write_failure_keeps_every_buffer_open() {
    let _guard = crate::testlock::serial();
    for door in DOORS {
        for parked in [false, true] {
            let a = Path::new("/probe/a.md");
            let b = Path::new("/probe/b.md");
            let mem = InMemoryFs::new()
                .with_file(a, "original a\n")
                .with_file(b, "original b\n");
            let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
            let mut app = app_on(Some(a.into()), "/probe", config(false));
            app.document.set_text(LATEST);
            if parked {
                assert!(app.load_path(b.into()));
                app.document.set_text(FIRST);
            }
            app.config.autosave = Some(true);
            let before = snapshot(&app);
            let _fault = crate::fs::FsGuard::install(Arc::new(fault(
                mem.clone(),
                ScriptedOperation::Write,
                true,
            )));
            for _ in 0..3 {
                assert!(!request(&mut app, door));
                assert_eq!(snapshot(&app), before);
                assert!(app.is_document_dirty());
                assert_eq!(mem.read(a).unwrap(), b"original a\n");
                assert_eq!(mem.read(b).unwrap(), b"original b\n");
            }
        }
    }
}

#[test]
fn quit_safety_clean_saved_and_autosaved_controls_can_exit() {
    let _guard = crate::testlock::serial();
    for door in DOORS {
        for autosave in [false, true] {
            let path = Path::new("/probe/a.md");
            let mem = InMemoryFs::new().with_file(path, FIRST);
            let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
            let mut app = app_on(Some(path.into()), "/probe", config(autosave));
            assert!(request(&mut app, door));
            app.document.set_text(LATEST);
            if !autosave {
                app.manual_save();
            }
            assert!(request(&mut app, door));
            assert_eq!(mem.read(path).unwrap(), LATEST.as_bytes());
            assert!(app.document.has_active());
        }
    }
}

#[test]
fn quit_safety_stale_recovery_never_authorizes_persistent_failed_writes() {
    let _guard = crate::testlock::serial();
    for door in DOORS {
        for autosave in [false, true] {
            let path = Path::new("/probe/a.md");
            let mem = InMemoryFs::new().with_file(path, "original\n");
            let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
            let mut app = app_on(Some(path.into()), "/probe", config(autosave));
            conflict(&mut app, &mem, path, false);
            let before = snapshot(&app);
            let _fault = crate::fs::FsGuard::install(Arc::new(fault(
                mem.clone(),
                ScriptedOperation::Write,
                true,
            )));
            for _ in 0..3 {
                assert!(!request(&mut app, door));
                assert_eq!(snapshot(&app), before);
                assert!(app.is_document_dirty());
                assert!(
                    app.frame
                        .notice()
                        .text()
                        .unwrap()
                        .contains("Recovery could not be saved")
                );
                assert_eq!(crate::recovery::read_for(path).unwrap().text, FIRST);
                assert_eq!(mem.read(path).unwrap(), OTHER.as_bytes());
            }
        }
    }
}

#[test]
fn quit_safety_partial_recovery_publication_refuses_exit_then_retries_current_text() {
    let _guard = crate::testlock::serial();
    for door in DOORS {
        let path = Path::new("/probe/a.md");
        let mem = InMemoryFs::new().with_file(path, "original\n");
        let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
        let mut app = app_on(Some(path.into()), "/probe", config(false));
        conflict(&mut app, &mem, path, false);
        let before = snapshot(&app);
        let _fault = crate::fs::FsGuard::install(Arc::new(fault(
            mem.clone(),
            ScriptedOperation::Rename,
            false,
        )));
        assert!(!request(&mut app, door));
        assert_eq!(snapshot(&app), before);
        assert_eq!(crate::recovery::read_for(path).unwrap().text, FIRST);
        assert!(
            request(&mut app, door),
            "current successful recovery permits exit"
        );
        assert_eq!(snapshot(&app), before);
        assert_eq!(crate::recovery::read_for(path).unwrap().text, LATEST);
        assert_eq!(mem.read(path).unwrap(), OTHER.as_bytes());
        assert!(
            app.is_document_dirty(),
            "recovery never marks the original saved"
        );
    }
}

#[test]
fn quit_safety_partial_preservation_cannot_hide_another_dirty_parked_entry() {
    let _guard = crate::testlock::serial();
    for door in DOORS {
        let a = Path::new("/probe/a.md");
        let b = Path::new("/probe/b.md");
        let mem = InMemoryFs::new()
            .with_file(a, "original a\n")
            .with_file(b, "original b\n");
        let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
        let mut app = app_on(Some(a.into()), "/probe", config(false));
        app.document.set_text("parked unsaved text\n");
        assert!(app.load_path(b.into()));
        conflict(&mut app, &mem, b, false);
        let before = snapshot(&app);
        for _ in 0..3 {
            assert!(!request(&mut app, door));
            assert_eq!(snapshot(&app), before);
            assert_eq!(crate::recovery::read_for(b).unwrap().text, LATEST);
            assert_eq!(mem.read(a).unwrap(), b"original a\n");
            assert_eq!(mem.read(b).unwrap(), OTHER.as_bytes());
        }
    }
}

#[test]
fn quit_safety_failed_recovery_does_not_mark_saved_or_bypass_deleted_close() {
    let _guard = crate::testlock::serial();
    let path = Path::new("/probe/a.md");
    let mem = InMemoryFs::new().with_file(path, "original\n");
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(path.into()), "/probe", config(true));
    conflict(&mut app, &mem, path, true);
    let before = snapshot(&app);
    let _fault =
        crate::fs::FsGuard::install(Arc::new(fault(mem.clone(), ScriptedOperation::Write, true)));
    app.autosave_flush();
    assert!(app.is_document_dirty());
    assert_ne!(
        app.document.doc_saved_version(),
        Some(app.document.buffer().version())
    );
    for _ in 0..3 {
        app.close_deleted_buffer();
        assert_eq!(snapshot(&app), before);
        assert!(app.change_unresolved());
        assert_eq!(crate::recovery::read_for(path).unwrap().text, FIRST);
        assert!(!mem.exists(path));
    }
}

#[test]
fn quit_safety_failed_recovery_retries_without_edit_and_deduplicates_only_current_bytes() {
    let _guard = crate::testlock::serial();
    let path = Path::new("/probe/a.md");
    let mem = InMemoryFs::new().with_file(path, "original\n");
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(path.into()), "/probe", config(true));
    conflict(&mut app, &mem, path, false);
    let key = app.document.active_key().unwrap();
    let record_path = crate::recovery::record_path();
    let temporary = record_path.with_file_name(".unresolved-change.md.awl-tmp");
    let scripted = fault(mem.clone(), ScriptedOperation::Write, false);
    let _fault = crate::fs::FsGuard::install(Arc::new(scripted.clone()));
    app.autosave_flush();
    assert_eq!(
        scripted
            .trace()
            .iter()
            .filter(|call| call.starts_with("write#"))
            .cloned()
            .collect::<Vec<_>>(),
        vec![format!("write#1 {}", temporary.display())]
    );
    assert_eq!(app.document.buffer().text(), LATEST);
    assert_eq!(mem.read(path).unwrap(), OTHER.as_bytes());
    assert!(app.change_unresolved());
    assert!(app.is_document_dirty());
    assert!(app.document.close_facts(&key).unwrap().unsaved);
    assert_eq!(crate::recovery::read_for(path).unwrap().text, FIRST);
    // The one-shot refusal has expired. Retry the unchanged editor text.
    app.autosave_flush();
    let writes = |trace: Vec<String>| {
        trace
            .into_iter()
            .filter(|call| call.starts_with("write#") || call.starts_with("rename#"))
            .collect::<Vec<_>>()
    };
    let published = writes(scripted.trace());
    assert_eq!(
        published,
        vec![
            format!("write#1 {}", temporary.display()),
            format!("write#2 {}", temporary.display()),
            format!(
                "rename#1 {} -> {}",
                temporary.display(),
                record_path.display()
            )
        ]
    );
    assert_eq!(crate::recovery::read_for(path).unwrap().text, LATEST);
    assert!(app.document.close_facts(&key).unwrap().unsaved);
    app.autosave_flush();
    assert_eq!(
        writes(scripted.trace()),
        published,
        "unchanged durable recovery needs no redundant publication"
    );
    app.document.set_text("newest manuscript\n");
    app.autosave_flush();
    assert_eq!(writes(scripted.trace()).len(), published.len() + 2);
    assert_eq!(
        crate::recovery::read_for(path).unwrap().text,
        "newest manuscript\n"
    );
    assert!(app.document.close_facts(&key).unwrap().unsaved);
    assert_eq!(mem.read(path).unwrap(), OTHER.as_bytes());
}

#[test]
fn quit_safety_successful_recovery_keeps_deleted_close_confirmation_and_latest_text() {
    let _guard = crate::testlock::serial();
    let path = Path::new("/probe/a.md");
    let mem = InMemoryFs::new().with_file(path, "original\n");
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(path.into()), "/probe", config(true));
    conflict(&mut app, &mem, path, true);
    let before = snapshot(&app);
    let key = app.document.active_key().unwrap();
    app.autosave_flush();
    assert_eq!(crate::recovery::read_for(path).unwrap().text, LATEST);
    assert!(
        app.is_document_dirty(),
        "recovery does not save the deleted original"
    );
    assert_ne!(
        app.document.doc_saved_version(),
        Some(app.document.buffer().version())
    );
    app.close_deleted_buffer();
    assert_eq!(
        snapshot(&app),
        before,
        "first close still requires confirmation"
    );
    assert!(app.change_unresolved());
    assert_eq!(crate::recovery::read_for(path).unwrap().text, LATEST);
    app.close_deleted_buffer();
    assert!(!app.document.open_entry_keys().contains(&key));
    assert!(!app.change_unresolved());
    assert!(
        crate::recovery::read().is_none(),
        "active recovery cleared only after retaining text"
    );
    assert_eq!(crate::recovery::read_for(path).unwrap().text, LATEST);
    assert!(
        !mem.exists(path),
        "deleted original is not recreated silently"
    );
}

#[test]
fn quit_safety_unpreserved_fresh_and_scratch_text_stays_open() {
    let _guard = crate::testlock::serial();
    for door in DOORS {
        for fresh in [false, true] {
            let mem = InMemoryFs::new().with_file("/probe/a.md", FIRST);
            let _fs = crate::fs::FsGuard::install(Arc::new(mem));
            let mut app = app_on(
                if fresh {
                    Some("/probe/a.md".into())
                } else {
                    None
                },
                "/probe",
                config(false),
            );
            if fresh {
                let exit = crate::app::schedule::RecordingExit::new();
                app.apply(Action::NewDocument, false, &exit, crate::stats::Door::Menu);
            }
            assert!(request(&mut app, door), "empty fresh/scratch is safe");
            app.document.set_text(LATEST);
            let before = snapshot(&app);
            for _ in 0..3 {
                assert!(!request(&mut app, door));
                assert_eq!(snapshot(&app), before);
            }
        }
    }
}

#[test]
fn quit_safety_window_close_uses_the_shared_gate_before_gpu_readiness() {
    let _guard = crate::testlock::serial();
    let source = include_str!("../lifecycle.rs");
    let window = source.split("fn window_event(").nth(1).unwrap();
    assert!(
        window
            .find("self.request_document_exit(event_loop)")
            .unwrap()
            < window.find("if self.frame.gpu().is_none()").unwrap()
    );
    assert!(!window.contains("WindowEvent::CloseRequested => event_loop.exit()"));
}
