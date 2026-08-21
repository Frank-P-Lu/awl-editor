use super::*;
use std::path::Path;
use std::sync::Arc;

#[test]
fn vanished_session_file_is_silently_skipped() {
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_file("/n/keep.md", "x\n"));
    crate::fs::with_fs(fake, || {
        let state = crate::session::SessionState {
            root: None,
            document_active: None,
            active: Some(PathBuf::from("/n/gone.md")),
            buffers: [("/n/gone.md", 5, 5, 5), ("/n/keep.md", 0, 0, 0)]
                .into_iter()
                .map(|(path, line, col, scroll)| {
                    (
                        PathBuf::from(path),
                        crate::session::BufferPos {
                            line,
                            col,
                            scroll,
                            scroll_px_q: 0,
                        },
                    )
                })
                .collect(),
            window: None,
        };
        crate::session::save(&crate::session::session_path(), &state).unwrap();
        let app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
        assert_ne!(app.document.buffer().path(), Some(Path::new("/n/gone.md")));
        assert!(
            !app.document
                .contains_background(&crate::buffers::BufferKey::path(Path::new("/n/gone.md")))
        );
        assert!(
            app.document
                .contains_background(&crate::buffers::BufferKey::path(Path::new("/n/keep.md")))
        );
    });
}
