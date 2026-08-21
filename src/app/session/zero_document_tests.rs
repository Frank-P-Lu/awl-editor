use super::*;
use std::sync::Arc;

#[test]
fn intentional_zero_document_session_round_trips_without_changing_first_launch() {
    let _guard = crate::testlock::serial();
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/n")
            .with_file("/n/only.md", "one\n"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(
            Some(PathBuf::from("/n/only.md")),
            PathBuf::from("/n"),
            None,
            None,
            Config {
                session_restore: Some(true),
                ..Config::empty()
            },
        );
        let key = app.document.active_key().unwrap();
        let _ = app.close_buffer(key);
        assert!(!app.document.has_active());
        app.session_flush();
        let state = crate::session::load(&crate::session::session_path());
        assert_eq!(state.document_active, Some(false));
        assert_eq!(state.root, Some(PathBuf::from("/n")));

        let restored = App::new(
            None,
            PathBuf::from("/n"),
            None,
            None,
            Config {
                session_restore: Some(true),
                ..Config::empty()
            },
        );
        assert!(!restored.document.has_active());

        crate::fs::active()
            .remove_file(&crate::session::session_path())
            .unwrap();
        let first = App::new(
            None,
            PathBuf::from("/n"),
            None,
            None,
            Config {
                session_restore: Some(true),
                ..Config::empty()
            },
        );
        assert!(
            first.document.has_active(),
            "missing session keeps launch scratch"
        );
    });
}
