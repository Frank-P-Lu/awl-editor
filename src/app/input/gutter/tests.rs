//! The GPU-free half of click-to-switch: a drawn stack row resolves to the file
//! it names, against a real `App`'s real working set.

use super::*;
use crate::config::Config;
use std::path::Path;
use std::sync::Arc;

/// The rows the MARGIN would draw for `app`, as labels — read through the same
/// owner the renderer reads (`ViewState`'s `gutter_files` comes from
/// `stack_rows(active_root())`), so the law's row indices are the drawn ones.
fn drawn_labels(app: &App) -> Vec<String> {
    let working = app.document.working_set();
    working
        .active_root()
        .map(|root| working.stack_rows(root))
        .unwrap_or_default()
        .iter()
        .map(|row| format!("{}{}", row.parent, row.leaf))
        .collect()
}

/// EVERY DRAWN ROW RESOLVES TO ITS OWN FILE, and the mapping is asserted over
/// the WHOLE stack rather than one hand-picked row — an off-by-one, or a lookup
/// that ignored the group filter, agrees with the right answer at index 0.
///
/// The tree is deliberately nested and opened out of alphabetical order, so a
/// resolution that leaned on sorting, on the leaf name, or on the registry's own
/// MRU order gives a different answer than this one.
#[test]
fn every_working_set_row_resolves_to_the_file_it_names() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_dir("/ws/notes/journal")
            .with_dir("/ws/archive")
            .with_file("/ws/notes/index.md", "index\n")
            .with_file("/ws/archive/log.md", "log\n")
            .with_file("/ws/notes/journal/field.md", "field\n")
            .with_file("/ws/notes/alpha.md", "alpha\n"),
    );
    crate::fs::with_fs(mem, || {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/ws"));
        let mut app = App::new_hermetic(
            Some(PathBuf::from("/ws/notes/index.md")),
            PathBuf::from("/ws/notes"),
            config,
        );
        // A file under ANOTHER root, opened BETWEEN two of this root's, so the
        // drawn group and the whole open set diverge IN THE MIDDLE — the one
        // arrangement where a resolution that skipped the group filter still
        // agrees at index 0 and is wrong everywhere after it.
        app.load_path(PathBuf::from("/ws/archive/log.md"));
        app.load_path(PathBuf::from("/ws/notes/alpha.md"));
        app.load_path(PathBuf::from("/ws/notes/journal/field.md"));
        assert_eq!(
            app.document.working_set().len(),
            4,
            "the foreign-root file is open too, and must not be a drawn row here"
        );
        assert_eq!(
            app.document.working_set().files()[1].leaf(),
            "log.md",
            "the foreign file sits mid-list, so group() and files() disagree from index 1 on"
        );

        let labels = drawn_labels(&app);
        assert_eq!(
            labels,
            vec!["index.md", "alpha.md", "journal/field.md"],
            "the margin draws the files in stable OPEN order"
        );

        let resolved: Vec<Option<PathBuf>> = (0..labels.len())
            .map(|row| app.gutter_stack_row_path(row))
            .collect();
        assert_eq!(
            resolved,
            vec![
                Some(PathBuf::from("/ws/notes/index.md")),
                Some(PathBuf::from("/ws/notes/alpha.md")),
                Some(PathBuf::from("/ws/notes/journal/field.md")),
            ],
            "each drawn row resolves to its own file, in the drawn order"
        );
        assert_eq!(
            app.gutter_stack_row_path(labels.len()),
            None,
            "a row past the end of the stack names nothing"
        );
    });
}

/// SWITCHING THROUGH THE ROW'S OWN ROUTE CHANGES THE FILE AND NOT THE ORDER.
///
/// The order half is the whole point of the surface — a tab strip would move
/// the switched-to file, and this must not — so it is asserted after EVERY
/// switch in a round trip, not once. Drives `load_path` with the row's own
/// resolved path, which is exactly what `gutter_stack_click` does once its
/// hit-test has answered; the hit-test half needs a live renderer and is
/// live-only (`docs/harness-reach.md` — no capture door drives a pointer).
#[test]
fn accepting_a_row_switches_the_buffer_and_never_reorders_the_stack() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_file("/ws/notes/a.md", "a\n")
            .with_file("/ws/notes/b.md", "b\n")
            .with_file("/ws/notes/c.md", "c\n"),
    );
    crate::fs::with_fs(mem, || {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/ws"));
        let mut app = App::new_hermetic(
            Some(PathBuf::from("/ws/notes/a.md")),
            PathBuf::from("/ws/notes"),
            config,
        );
        app.load_path(PathBuf::from("/ws/notes/b.md"));
        app.load_path(PathBuf::from("/ws/notes/c.md"));
        let order = drawn_labels(&app);
        assert_eq!(order, vec!["a.md", "b.md", "c.md"], "the opened order");

        // Every row in turn, ending back where a plain MRU list would have left
        // the most disturbed order.
        for (row, expected) in [
            (0, "/ws/notes/a.md"),
            (2, "/ws/notes/c.md"),
            (1, "/ws/notes/b.md"),
        ] {
            let path = app
                .gutter_stack_row_path(row)
                .unwrap_or_else(|| panic!("row {row} names a file"));
            app.load_path(path);
            assert_eq!(
                app.document.buffer().path(),
                Some(Path::new(expected)),
                "accepting row {row} activated the file it names"
            );
            assert_eq!(
                drawn_labels(&app),
                order,
                "accepting row {row} reordered the stack"
            );
            let active = app
                .document
                .working_set()
                .active_index()
                .expect("a file is active");
            assert_eq!(
                app.document
                    .working_set()
                    .group(Path::new("/ws/notes"))
                    .iter()
                    .position(|&at| at == active),
                Some(row),
                "the drawn active row followed the switch to row {row}"
            );
        }
    });
}
