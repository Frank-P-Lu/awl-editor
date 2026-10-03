use super::*;
use crate::app::{App, Config};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[test]
fn every_isolated_route_never_calls_the_system_factory_and_live_calls_once() {
    let calls = Cell::new(0);
    for route in [
        Route::default_app(),
        Route::Capture,
        Route::Scheduler,
        Route::Persistence,
    ] {
        let answer = select(
            route,
            || {
                calls.set(calls.get() + 1);
                panic!("system factory");
            },
            || 7,
        );
        assert_eq!(answer, 7);
        assert_eq!(calls.get(), 0, "{route:?}");
        assert!(for_route(route).is_some());
    }
    assert_eq!(
        select(
            Route::Live,
            || {
                calls.set(calls.get() + 1);
                9
            },
            || panic!("memory factory")
        ),
        9
    );
    assert_eq!(calls.get(), 1);
}

#[test]
fn the_factory_guard_rejects_each_isolated_route_mutated_to_live() {
    for intended in [
        Route::default_app(),
        Route::Capture,
        Route::Scheduler,
        Route::Persistence,
    ] {
        let result = std::panic::catch_unwind(|| {
            // The same guard as the preceding law sees the wrong-backend mutant.
            select(
                Route::Live,
                || panic!("system factory for {intended:?}"),
                || 7,
            )
        });
        assert!(
            result.is_err(),
            "{intended:?}: wrong-backend mutant escaped"
        );
    }
}

#[test]
fn actual_unit_capture_and_scheduler_constructors_start_with_memory() {
    let _serial = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(crate::fs::InMemoryFs::new()));
    let constructors: [fn() -> App; 3] = [
        || App::new_hermetic(None, PathBuf::from("/unit"), Config::empty()),
        || App::new_headless_capture(None, PathBuf::from("/capture"), None, Config::empty()),
        || App::new_headless_scheduler(PathBuf::from("/scheduler"), Config::empty()),
    ];
    for construct in constructors {
        let mut app = construct();
        let clip = app
            .clipboard
            .as_mut()
            .expect("memory handle installed at construction");
        assert!(clip.get_text().is_err());
        clip.set_text("isolated".into()).unwrap();
        assert_eq!(clip.get_text().unwrap(), "isolated");
    }
}

#[test]
fn a_shared_backend_is_installed_before_the_real_app_interpreter_copies_and_pastes() {
    let _serial = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file("/n/a.md", "alpha"),
    ));
    let memory = MemoryClipboard::new();
    let mut app = App::new_with_clipboard(
        Some(PathBuf::from("/n/a.md")),
        PathBuf::from("/n"),
        None,
        None,
        Config {
            session_restore: Some(false),
            reduce_motion: Some(false),
            ..Config::empty()
        },
        Some(Box::new(memory.clone())),
    );
    let exit = crate::app::schedule::RecordingExit::new();
    app.document.select_range(0, 5);
    app.apply(
        crate::keymap::Action::CopyRegion,
        false,
        &exit,
        crate::stats::Door::Chord,
    );
    assert_eq!(memory.current().as_deref(), Some("alpha"));
    memory.set_external("external");
    app.document.select_range(0, 5);
    app.apply(
        crate::keymap::Action::Yank,
        false,
        &exit,
        crate::stats::Door::Chord,
    );
    assert_eq!(app.document.buffer().text(), "external");
}

#[test]
fn system_initialization_has_one_live_owner_and_no_common_constructor_fallback() {
    let backend = include_str!("../clipboard_backend.rs");
    let app = include_str!("../../app.rs");
    let needle = ["arboard", "::", "Clipboard", "::new()"].concat();
    assert_eq!(backend.matches(&needle).count(), 1);
    assert!(backend.contains("#[cfg(not(test))]\nfn system()"));
    assert!(!app.contains(&needle));
    let body = app
        .split("fn new_with_clipboard(")
        .nth(1)
        .unwrap()
        .split("fn set_sticky_notice(")
        .next()
        .unwrap();
    assert!(body.contains("            clipboard,"));
    assert!(!body.contains("default_clipboard()"));
    assert!(!body.contains("for_route("));
    for (source, expected) in [
        (include_str!("../capture_state.rs"), "Route::Capture"),
        (
            app.split("fn new_headless_scheduler(").nth(1).unwrap(),
            "Route::Scheduler",
        ),
        (
            include_str!("../persistence/fault_probe.rs"),
            "Route::Persistence",
        ),
    ] {
        assert!(source.contains(expected));
        assert!(source.contains("new_with_clipboard("));
    }
    assert!(app.contains("for_route(clipboard_backend::Route::default_app())"));
    // Scan the whole source tree so a new acquisition site cannot hide elsewhere.
    fn scan(dir: &Path, needle: &str, hits: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                scan(&path, needle, hits);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                if text.contains(needle) {
                    hits.push(path);
                }
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    scan(&root, &needle, &mut hits);
    assert_eq!(hits, vec![root.join("app/clipboard_backend.rs")]);
}

#[test]
fn every_injected_constructor_is_accounted_for_without_weakening_real_fs_audits() {
    let app_needle = ["App", "::", "new_with_clipboard", "("].concat();
    let self_needle = ["Self", "::", "new_with_clipboard", "("].concat();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    fn census(
        dir: &Path,
        root: &Path,
        needles: &[&str],
        counts: &mut std::collections::BTreeMap<String, usize>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                census(&path, root, needles, counts);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                let n = needles
                    .iter()
                    .map(|needle| source.matches(needle).count())
                    .sum();
                if n > 0 {
                    counts.insert(
                        path.strip_prefix(root)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned(),
                        n,
                    );
                }
            }
        }
    }
    let mut counts = std::collections::BTreeMap::new();
    census(&root, &root, &[&app_needle, &self_needle], &mut counts);
    // App's default selection and scheduler; seeded capture; real-FS persistence;
    // the one retained-memory interpreter law. Every new caller needs review.
    let expected = [
        ("app.rs", 2),
        ("app/capture_state.rs", 1),
        ("app/persistence/fault_probe.rs", 1),
        ("app/clipboard_backend/tests.rs", 1),
    ]
    .into_iter()
    .map(|(file, n)| (file.to_owned(), n))
    .collect();
    assert_eq!(counts, expected);
}
