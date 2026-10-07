//! Explicit folder choices preserve documents while replacing overlapping scope.
use super::*;
use crate::fs::FileSystem;
use std::sync::Arc;

#[test]
fn explicit_parent_child_switch_preserves_open_buffers_and_replaces_inside_roots() {
    let _guard = crate::testlock::serial();
    for opener in ["/workspace/parent.md", "/workspace/notes/a.md"] {
        for destination in ["/workspace/notes", "/workspace"] {
            let mem = Arc::new(
                crate::fs::InMemoryFs::new()
                    .with_dir("/workspace/notes")
                    .with_dir("/other")
                    .with_file("/workspace/parent.md", "parent disk\n")
                    .with_file("/workspace/notes/a.md", "child disk\n")
                    .with_file("/other/outside.md", "outside disk\n"),
            );
            crate::fs::with_fs(mem.clone(), || {
                let config = Config {
                    autosave: Some(false),
                    session_restore: Some(false),
                    ..Config::empty()
                };
                // Keep the seeded caller-owned memory FS through construction and switching.
                let mut app = App::new(
                    Some(PathBuf::from(opener)),
                    PathBuf::from("/workspace"),
                    None,
                    None,
                    config,
                );
                app.document.set_text("unsaved opener\n");
                app.document.set_cursor(4);
                assert!(app.load_path(PathBuf::from("/workspace/notes/a.md")));
                app.document.set_text("unsaved child\n");
                assert!(app.load_path(PathBuf::from("/other/outside.md")));
                app.document.set_text("unsaved outside\n");
                assert!(app.load_path(PathBuf::from(opener)));
                let before = app.document.buffer().text().to_string();
                let cursor = app.document.buffer().cursor_char();
                let count = app.document.open_count();
                let keys: Vec<_> = app
                    .document
                    .working_set()
                    .files()
                    .iter()
                    .map(|f| f.key.clone())
                    .collect();
                app.setting_path_pick("project_root", destination);
                assert_eq!(app.project_location.root, Path::new(destination));
                assert_eq!(app.document.open_count(), count);
                assert_eq!(app.document.buffer().text(), before);
                assert_eq!(app.document.buffer().cursor_char(), cursor);
                assert_eq!(
                    app.document
                        .working_set()
                        .files()
                        .iter()
                        .map(|f| f.key.clone())
                        .collect::<Vec<_>>(),
                    keys
                );
                for file in app.document.working_set().files() {
                    if file
                        .path
                        .as_deref()
                        .is_some_and(|path| path.starts_with(destination))
                    {
                        assert_eq!(
                            file.root,
                            Path::new(destination),
                            "inside files adopt the explicit folder"
                        );
                    }
                }
                assert!(app.load_path(PathBuf::from("/workspace/notes/a.md")));
                assert_eq!(
                    app.project_location.root,
                    Path::new(destination),
                    "reopening an inside file must not undo the choice"
                );
                assert_eq!(app.document.buffer().text(), "unsaved child\n");
                assert!(app.load_path(PathBuf::from("/other/outside.md")));
                assert_eq!(app.project_location.root, Path::new("/other"));
                assert_eq!(app.document.buffer().text(), "unsaved outside\n");
                assert_eq!(
                    mem.read(Path::new("/other/outside.md")).unwrap(),
                    b"outside disk\n"
                );
                assert_eq!(
                    mem.read(Path::new("/workspace/notes/a.md")).unwrap(),
                    b"child disk\n"
                );
            });
        }
    }
}
