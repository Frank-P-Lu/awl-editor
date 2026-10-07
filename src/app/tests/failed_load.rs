//! Real App transitions over synthetic files; failed reads must never authorize writes.
use super::*;
use crate::fs::{FileSystem, InMemoryFs};
use std::path::Path;

const EDIT: &str = "new user edit\n";
fn config(restore: bool) -> Config {
    Config {
        session_restore: Some(restore),
        ..Config::empty()
    }
}
fn samples() -> [(&'static str, &'static [u8]); 3] {
    [
        ("latin1", &[0xe9, 0xe9]),
        ("truncated_utf8", &[0xe2, 0x82]),
        ("nul", b"text\0binary"),
    ]
}

#[test]
fn clean_external_reload_preserves_text_bytes_baseline_and_recovers_when_text_returns() {
    for (label, bytes) in samples() {
        let path = Path::new("/probe/draft.md");
        let mem = InMemoryFs::new().with_file(path, "hello\n");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let mut app = app_on(Some(path.into()), "/probe", config(false));
            let baseline = app.document.disk_baseline();
            mem.write(path, bytes).unwrap();
            app.on_focus_gained();
            assert_eq!(app.document.buffer().text(), "hello\n", "{label}");
            assert_eq!(app.document.disk_baseline(), baseline, "{label}");
            assert!(app.change_unresolved(), "{label}");
            assert!(
                app.frame
                    .notice()
                    .text()
                    .unwrap()
                    .contains("cannot be read")
            );
            app.document.set_text(EDIT);
            app.autosave_flush();
            app.manual_save();
            assert_eq!(mem.read(path).unwrap(), bytes, "{label}");
            assert_eq!(app.document.buffer().text(), EDIT);
            mem.write(path, b"recovered disk text\n").unwrap();
            app.on_focus_gained();
            assert_eq!(
                app.document.buffer().text(),
                EDIT,
                "recovery requires a choice"
            );
            app.resolve_take_theirs();
            assert!(!app.change_unresolved());
            assert_eq!(app.document.buffer().text(), "recovered disk text\n");
            app.document.set_text(EDIT);
            app.autosave_flush();
            assert_eq!(mem.read(path).unwrap(), EDIT.as_bytes());
        });
    }
}

fn restore_case(background: bool) {
    for (label, bytes) in samples() {
        let path = Path::new("/probe/unsupported.md");
        let good = Path::new("/probe/good.md");
        let mem = InMemoryFs::new().with_file(good, "valid text\n");
        mem.write(path, bytes).unwrap();
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let stash = crate::fs::scratch_stash_path();
            mem.write(&stash, b"previous scratch text\n").unwrap();
            let state = crate::session::SessionState {
                active: Some(if background { good } else { path }.into()),
                buffers: vec![
                    (good.into(), Default::default()),
                    (path.into(), Default::default()),
                ],
                ..Default::default()
            };
            crate::session::save(&crate::session::session_path(), &state).unwrap();
            let mut app = app_on(None, "/probe", config(true));
            assert!(app.frame.notice().text().unwrap().contains("left unopened"));
            assert!(
                !app.document
                    .working_set()
                    .files()
                    .iter()
                    .any(|f| f.path.as_deref() == Some(path)),
                "{label}"
            );
            if background {
                assert_eq!(app.document.buffer().path(), Some(good));
                app.close_active_buffer();
                assert!(!app.document.has_active(), "failed successor {label}");
            } else {
                assert_eq!(app.document.buffer().path(), None);
                assert_eq!(app.document.buffer().text(), "previous scratch text\n");
                app.document.set_text(EDIT);
                app.autosave_flush();
            }
            assert_eq!(mem.read(path).unwrap(), bytes, "{label}");
            mem.write(path, b"recovered disk text\n").unwrap();
            assert!(app.load_path(path.into()), "{label}");
            assert_eq!(app.document.buffer().text(), "recovered disk text\n");
            app.document.set_text(EDIT);
            app.autosave_flush();
            assert_eq!(mem.read(path).unwrap(), EDIT.as_bytes());
        });
    }
}
#[test]
fn active_session_restore_skips_failed_load_without_discarding_scratch() {
    restore_case(false);
}
#[test]
fn background_session_restore_cannot_promote_failed_load_on_close() {
    restore_case(true);
}
#[test]
fn valid_external_text_control_reloads_and_saves_normally() {
    let path = Path::new("/probe/valid.md");
    let mem = InMemoryFs::new().with_file(path, "hello\n");
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut app = app_on(Some(path.into()), "/probe", config(false));
        mem.write(path, "valid external é text\n".as_bytes())
            .unwrap();
        app.on_focus_gained();
        assert_eq!(app.document.buffer().text(), "valid external é text\n");
        app.document.set_text(EDIT);
        app.autosave_flush();
        assert_eq!(mem.read(path).unwrap(), EDIT.as_bytes());
    });
}
#[test]
fn corrupt_scratch_bytes_remain_protected_after_startup_edit() {
    for (_, bytes) in samples() {
        let mem = InMemoryFs::new();
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let stash = crate::fs::scratch_stash_path();
            mem.write(&stash, bytes).unwrap();
            let mut app = app_on(None, "/probe", config(false));
            app.document.set_text(EDIT);
            app.autosave_flush();
            assert_eq!(mem.read(&stash).unwrap(), bytes);
            assert_eq!(app.document.buffer().text(), EDIT);
        });
    }
}

#[test]
fn closing_into_a_restored_slot_holds_a_later_failed_reload_without_losing_its_text() {
    for (label, bytes) in samples() {
        let good = Path::new("/probe/good.md");
        let later = Path::new("/probe/later.md");
        let mem = InMemoryFs::new()
            .with_file(good, "good\n")
            .with_file(later, "remembered text\n");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let state = crate::session::SessionState {
                active: Some(good.into()),
                buffers: vec![
                    (good.into(), Default::default()),
                    (later.into(), Default::default()),
                ],
                ..Default::default()
            };
            crate::session::save(&crate::session::session_path(), &state).unwrap();
            let mut app = app_on(None, "/probe", config(true));
            mem.write(later, bytes).unwrap();
            app.close_active_buffer();
            assert_eq!(app.document.buffer().path(), Some(later), "{label}");
            assert_eq!(app.document.buffer().text(), "remembered text\n", "{label}");
            assert!(app.change_unresolved(), "{label}");
            app.document.set_text(EDIT);
            app.autosave_flush();
            assert_eq!(mem.read(later).unwrap(), bytes, "{label}");
            assert_eq!(app.document.buffer().text(), EDIT);
        });
    }
}
