//! The GPU-free half of click-to-switch: a drawn stack row resolves to the file
//! it names, against a real `App`'s real working set.

use super::*;
use crate::config::Config;
use std::path::Path;
use std::sync::Arc;

/// The FILE rows the MARGIN would draw for `app`, as labels — read through the
/// same owner the renderer reads (`ViewState`'s `gutter_files` comes from
/// `stack_rows(active_root())`), so the law's row indices are the drawn ones.
/// Filtered to `StackRowKind::File`: every law below asserts row→file
/// resolution, and the trailing `+ N more…` row (residual 3's overflow
/// affordance, present once a hidden buffer exists anywhere) names no file at
/// all — including it here would assert a leaf/parent pair for a row that is
/// not one.
fn drawn_labels(app: &App) -> Vec<String> {
    let working = app.document.working_set();
    working
        .active_root()
        .map(|root| working.stack_rows(root))
        .unwrap_or_default()
        .iter()
        .filter(|row| matches!(row.kind, crate::workingset::StackRowKind::File))
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

/// THE CLOSE ROUTE AND THE SWITCH ROUTE NAME THE SAME FILE FOR THE SAME ROW.
///
/// Two resolutions of one row index is the shape that goes wrong quietly: a
/// close that was off by one against the switch would still close a real,
/// open file — just not the one under the pointer — and every assertion about
/// "a file closed" would pass.
///
/// Swept over the WHOLE stack against a cross-root file parked mid-list, so a
/// resolution that dropped the `group(root)` filter agrees at index 0 and is
/// wrong from index 1 on. The scratch row is deliberately absent here (it has
/// no path, so the pair cannot be compared on it) and is covered by the
/// removal owner's own successor law.
#[test]
fn the_close_route_resolves_every_row_to_the_same_file_the_switch_route_does() {
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
        app.load_path(PathBuf::from("/ws/archive/log.md"));
        app.load_path(PathBuf::from("/ws/notes/alpha.md"));
        app.load_path(PathBuf::from("/ws/notes/journal/field.md"));
        let labels = drawn_labels(&app);
        assert_eq!(
            labels,
            vec!["index.md", "alpha.md", "journal/field.md"],
            "the drawn group excludes the foreign-root file"
        );

        for (row, label) in labels.iter().enumerate() {
            let path = app
                .gutter_stack_row_path(row)
                .unwrap_or_else(|| panic!("row {row} names a file"));
            let key = app
                .gutter_stack_row_key(row)
                .unwrap_or_else(|| panic!("row {row} names a buffer"));
            assert_eq!(
                key,
                crate::buffers::BufferKey::path(&path),
                "row {row} ({label}) resolves to two different buffers depending on which \
                 half of the row the pointer landed on"
            );
        }
        assert_eq!(
            app.gutter_stack_row_key(labels.len()),
            None,
            "a row past the end of the stack names no buffer either"
        );
    });
}

/// **THE SINGLE-FILE ROW'S CLOSE ZONE REACHES THE SAME ZERO-DOCUMENT START
/// SURFACE ⌘W DOES.**
///
/// The identity line now enrols in row 0 of this exact row→file door
/// (`crate::render::chrome::gutter_hit::stack_hit_from_plan` answers
/// `row: 0` for `GutterLine::Name`, `gutter_hit::tests`' own law), and its
/// close zone dispatches through [`App::close_buffer`] — the SAME owner ⌘W's
/// `Action::FinishBuffer` calls (`app/files/close.rs`'s doc: "same behavior ⇒
/// same code"). So this asserts the SAME outcome
/// `app::files::close::tests::closing_the_last_file_enters_the_honest_zero_document_state`
/// pins for ⌘W, reached through the pointer's own row index instead — proving
/// the single-file margin is not a second, undertested door to "no document
/// left".
#[test]
fn closing_the_lone_row_reaches_the_same_zero_document_state_cmd_w_does() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_file("/ws/notes/only.md", "one\n"),
    );
    crate::fs::with_fs(mem, || {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/ws"));
        let mut app = App::new_hermetic(
            Some(PathBuf::from("/ws/notes/only.md")),
            PathBuf::from("/ws/notes"),
            config,
        );
        assert_eq!(app.document.working_set().len(), 1, "precondition: N=1");

        // Row 0 is the ONLY row the single-file margin's close zone can ever
        // enrol in (`gutter_hit::tests::the_lone_identity_row_resolves_the_
        // same_close_zone_a_stack_row_would`), so this resolves through the
        // exact index that hit-test answers rather than a hand-picked one.
        let key = app
            .gutter_stack_row_key(0)
            .expect("row 0 names the lone open file");
        assert_eq!(
            key,
            crate::buffers::BufferKey::path(Path::new("/ws/notes/only.md")),
            "row 0 must name the one open file"
        );

        app.close_buffer(key);

        assert_eq!(
            app.document.working_set().len(),
            0,
            "the working set has no invented replacement row"
        );
        assert!(
            !app.document.has_active(),
            "closing the lone row must leave no active document"
        );
        assert!(app.document.buffer_opt().is_none());
    });
}

/// **THE EXPANDED PANEL'S ROWS RESOLVE TO THEIR OWN FILES, across TWO roots
/// at once** — the residual-3 counterpart of
/// `every_working_set_row_resolves_to_the_file_it_names`, in the panel's own
/// multi-root, headed index space rather than `group(root)`. Swept over the
/// whole drawn panel so an off-by-one at a root boundary is not hidden by
/// checking only row 0.
#[test]
fn every_expanded_panel_row_resolves_to_the_file_it_names() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_dir("/ws/archive")
            .with_file("/ws/notes/index.md", "index\n")
            .with_file("/ws/notes/alpha.md", "alpha\n")
            .with_file("/ws/archive/log.md", "log\n"),
    );
    crate::fs::with_fs(mem, || {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/ws"));
        let mut app = App::new_hermetic(
            Some(PathBuf::from("/ws/notes/index.md")),
            PathBuf::from("/ws/notes"),
            config,
        );
        app.load_path(PathBuf::from("/ws/notes/alpha.md"));
        app.load_path(PathBuf::from("/ws/archive/log.md"));
        app.document.working_set_mut().expand();
        assert!(app.document.working_set().is_expanded());

        let rows = app.document.working_set().expanded_rows();
        for (row, drawn) in rows.iter().enumerate() {
            match drawn.kind {
                crate::workingset::StackRowKind::File => {
                    let path = app
                        .gutter_stack_row_path(row)
                        .unwrap_or_else(|| panic!("row {row} names a file"));
                    let key = app
                        .gutter_stack_row_key(row)
                        .unwrap_or_else(|| panic!("row {row} names a buffer"));
                    assert_eq!(
                        path.file_name().and_then(|n| n.to_str()),
                        Some(drawn.leaf.as_str()),
                        "row {row} resolves to a different file than the one drawn"
                    );
                    assert_eq!(
                        key,
                        crate::buffers::BufferKey::path(&path),
                        "row {row}'s path and key routes disagree about which file it names"
                    );
                }
                crate::workingset::StackRowKind::Group { .. } => {
                    assert_eq!(
                        app.gutter_stack_row_path(row),
                        None,
                        "row {row} is a heading and must name no file"
                    );
                }
                crate::workingset::StackRowKind::Overflow { .. } => {
                    assert_eq!(
                        app.gutter_stack_row_path(row),
                        None,
                        "row {row} is a passive overflow cue and must name no file"
                    );
                }
                crate::workingset::StackRowKind::More { .. } => {
                    unreachable!("the expanded panel draws no More row")
                }
            }
        }
    });
}

/// **A GROUP HEADING'S CLOSE ROUTE CLOSES ONLY ITS OWN ROOT'S FILES** — the
/// end-to-end proof behind clicking a heading's own close zone. Resolves the
/// heading row through the exact GPU-free door the pixel hit-test hands off
/// to ([`App::gutter_stack_row_group_root`]), then folds through
/// [`App::close_group`], and checks the SIBLING root's files never moved —
/// the failure this exists to catch is a group-close that reads `root` from
/// the wrong row, or that closes across the whole working set rather than one
/// group of it.
#[test]
fn a_group_headings_close_route_closes_only_its_own_root_never_a_sibling_groups_files() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_dir("/ws/archive")
            .with_file("/ws/notes/index.md", "index\n")
            .with_file("/ws/notes/alpha.md", "alpha\n")
            .with_file("/ws/archive/log.md", "log\n"),
    );
    crate::fs::with_fs(mem, || {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/ws"));
        let mut app = App::new_hermetic(
            Some(PathBuf::from("/ws/notes/index.md")),
            PathBuf::from("/ws/notes"),
            config,
        );
        app.load_path(PathBuf::from("/ws/notes/alpha.md"));
        app.load_path(PathBuf::from("/ws/archive/log.md"));
        app.document.working_set_mut().expand();

        let rows = app.document.working_set().expanded_rows();
        let heading_row = rows
            .iter()
            .position(|r| r.leaf == "notes/")
            .expect("the notes group draws its own heading");
        let root = app
            .gutter_stack_row_group_root(heading_row)
            .expect("a heading row names its own root");

        app.close_group(root);

        let remaining: Vec<_> = app
            .document
            .working_set()
            .files()
            .iter()
            .filter_map(|f| f.path.clone())
            .collect();
        assert_eq!(
            remaining,
            vec![PathBuf::from("/ws/archive/log.md")],
            "closing the notes heading must close every notes file and \
             leave archive's own file untouched"
        );
    });
}

/// **CROSS-ROOT ACTIVATION RESTORES THE MATCHING PROJECT ROOT** — the
/// end-to-end proof behind clicking a file in another group of the expanded
/// panel. The click plumbing itself resolves a row to a [`crate::buffers::BufferKey`]
/// and hands it to [`App::activate_open_buffer`] exactly as the resting
/// stack's own switch already does (the sibling law just above proving that,
/// `accepting_a_row_switches_the_buffer_and_never_reorders_the_stack`); this
/// proves the DOOR that call lands on already restores the root, so the panel
/// needed no second restoration mechanism of its own — only the click wiring,
/// which is what this dispatch built.
#[test]
fn activating_an_open_file_from_another_root_restores_its_remembered_project_root() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_dir("/ws/archive")
            .with_file("/ws/notes/index.md", "index\n")
            .with_file("/ws/notes/alpha.md", "alpha\n")
            .with_file("/ws/archive/log.md", "log\n"),
    );
    crate::fs::with_fs(mem, || {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/ws"));
        let mut app = App::new_hermetic(
            Some(PathBuf::from("/ws/notes/index.md")),
            PathBuf::from("/ws/notes"),
            config,
        );
        app.load_path(PathBuf::from("/ws/notes/alpha.md"));
        app.load_path(PathBuf::from("/ws/archive/log.md"));
        // Back to `notes` — the active root when the cross-root activation
        // below fires, so the restore is genuinely exercised rather than a
        // no-op against the root already active.
        app.load_path(PathBuf::from("/ws/notes/index.md"));
        assert_eq!(app.project_location.root, PathBuf::from("/ws/notes"));
        assert_eq!(
            app.document.working_set().active_root(),
            Some(Path::new("/ws/notes"))
        );

        let key = crate::buffers::BufferKey::path(Path::new("/ws/archive/log.md"));
        app.activate_open_buffer(key);

        assert_eq!(
            app.document.buffer().path(),
            Some(Path::new("/ws/archive/log.md")),
            "the cross-root file is now the active document"
        );
        assert_eq!(
            app.project_location.root,
            PathBuf::from("/ws/archive"),
            "the project root must follow the arriving document's own remembered root"
        );
        assert_eq!(
            app.document.working_set().active_root(),
            Some(Path::new("/ws/archive")),
            "the working set's own active-root readout agrees"
        );
    });
}

/// The FILE leaves `capture_opts().working_set` reports, in drawn order — the
/// SAME fold `--screenshot-app` writes into its sidecar
/// (`app/capture_state.rs`), so a law reading through this proves the
/// SIDECAR-FACING order agrees, not just an internal field. Filtered to
/// `StackRowKind::File` for the same reason `drawn_labels` is. Native-only:
/// `capture_opts` lives on `app/capture_state.rs`, gated the same way
/// (`--screenshot-app` is a native-only CLI mode).
#[cfg(not(target_arch = "wasm32"))]
fn sidecar_labels(app: &App) -> Vec<String> {
    app.capture_opts()
        .working_set
        .iter()
        .filter(|row| matches!(row.kind, crate::workingset::StackRowKind::File))
        .map(|row| format!("{}{}", row.parent, row.leaf))
        .collect()
}

/// **A DRAG-AND-DROP REORDERS THE GROUP, and the sidecar fold agrees** — the
/// GPU-free seam `gutter_stack_row_drop` gives the live pointer machinery,
/// driven directly with two row indices (as if a recognized drag had already
/// resolved them) rather than through pixel geometry. Native-only: see
/// `sidecar_labels`'s own doc.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn dragging_a_file_row_reorders_its_own_group_and_the_sidecar_fold_agrees() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_file("/ws/notes/a.md", "a\n")
            .with_file("/ws/notes/b.md", "b\n")
            .with_file("/ws/notes/c.md", "c\n")
            .with_file("/ws/notes/d.md", "d\n"),
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
        app.load_path(PathBuf::from("/ws/notes/d.md"));
        assert_eq!(
            drawn_labels(&app),
            vec!["a.md", "b.md", "c.md", "d.md"],
            "opened order"
        );

        // Drag row 0 (a.md) to row 2's slot — the standard "move to this
        // position in the final order" convention `reorder_in_group` documents.
        assert!(app.gutter_stack_row_drop(0, 2));

        let want = vec!["b.md", "c.md", "a.md", "d.md"];
        assert_eq!(drawn_labels(&app), want, "the internal stack order moved");
        assert_eq!(
            sidecar_labels(&app),
            want,
            "the sidecar-facing fold (capture_opts) must report the same order"
        );
    });
}

/// **A DRAG NEVER ACTIVATES THE ROW IT DROPS** — it moves a row without
/// disturbing what the reader is looking at, even when the dragged file
/// itself is not the active one.
#[test]
fn dropping_a_row_never_changes_which_file_is_active() {
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
        // c.md is active. Drag a.md (row 0, background) to the end.
        assert_eq!(
            app.document.buffer().path(),
            Some(Path::new("/ws/notes/c.md"))
        );
        assert!(app.gutter_stack_row_drop(0, 2));
        assert_eq!(
            app.document.buffer().path(),
            Some(Path::new("/ws/notes/c.md")),
            "dragging a background row must not switch the active document"
        );
        assert_eq!(drawn_labels(&app), vec!["b.md", "c.md", "a.md"]);
    });
}

/// **THE WINDOW-OFFSET REGRESSION, END TO END**: with more than `RESTING_FILES`
/// files open under one root and the hold-still window slid away from the
/// top, dragging the drawn row 0 must move the file the window ACTUALLY shows
/// there — the exact bug a naive `group(root)[row]` resolution carried,
/// dormant until a group grew past the resting cap.
#[test]
fn dragging_a_row_in_a_slid_resting_window_moves_the_file_actually_drawn_there() {
    let _guard = crate::testlock::serial();
    let mut fs = crate::fs::InMemoryFs::new().with_dir("/ws/notes");
    for i in 0..8 {
        fs = fs.with_file(format!("/ws/notes/f{i}.md"), "x\n");
    }
    let mem = Arc::new(fs);
    crate::fs::with_fs(mem, || {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/ws"));
        let mut app = App::new_hermetic(
            Some(PathBuf::from("/ws/notes/f0.md")),
            PathBuf::from("/ws/notes"),
            config,
        );
        for i in 1..8 {
            app.load_path(PathBuf::from(format!("/ws/notes/f{i}.md")));
        }
        // Eight files, RESTING_FILES=5: the window has slid to show the last
        // five (f3..f7), with f0 the reader was just looking at now hidden.
        assert_eq!(
            drawn_labels(&app),
            vec!["f3.md", "f4.md", "f5.md", "f6.md", "f7.md"],
            "precondition: the window has slid"
        );

        // Drawn row 0 names f3.md, NOT f0.md — dragging it to the drawn row-2
        // slot must move f3.md, not the group's absolute index 0.
        assert!(app.gutter_stack_row_drop(0, 2));
        assert_eq!(
            drawn_labels(&app),
            vec!["f4.md", "f5.md", "f3.md", "f6.md", "f7.md"],
            "the window-aware resolver moved the file actually drawn at row 0"
        );
        // f0/f1/f2 (never touched by the drag — outside the visible window)
        // keep their original relative order ahead of the moved group.
        assert_eq!(
            app.document
                .working_set()
                .files()
                .iter()
                .map(|f| f.leaf())
                .collect::<Vec<_>>(),
            vec![
                "f0.md", "f1.md", "f2.md", "f4.md", "f5.md", "f3.md", "f6.md", "f7.md"
            ]
        );
    });
}

/// **IN-GROUP ONLY: A DRAG IN THE EXPANDED PANEL CANNOT CROSS A GROUP
/// HEADING.** Dragging a `notes` file onto a row belonging to `archive` (or
/// its own heading) clamps the drop to the nearest edge of `notes`' own
/// block — `archive`'s own group is untouched.
#[test]
fn a_drag_in_the_expanded_panel_never_crosses_into_another_roots_group() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_dir("/ws/archive")
            .with_file("/ws/notes/a.md", "a\n")
            .with_file("/ws/notes/b.md", "b\n")
            .with_file("/ws/notes/c.md", "c\n")
            .with_file("/ws/archive/x.md", "x\n")
            .with_file("/ws/archive/y.md", "y\n"),
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
        app.load_path(PathBuf::from("/ws/archive/x.md"));
        app.load_path(PathBuf::from("/ws/archive/y.md"));
        app.document.working_set_mut().expand();
        assert!(app.document.working_set().is_expanded());

        // Drawn rows: [Group(notes), a, b, c, Group(archive), x, y] (7 rows).
        let rows = app.document.working_set().expanded_rows();
        assert_eq!(rows.len(), 7);

        // Drag `notes/a.md` (row 1) onto `archive`'s OWN heading (row 4): must
        // clamp to the BOTTOM of notes' own group, never move into archive's.
        assert!(app.gutter_stack_row_drop(1, 4));
        let notes_group: Vec<String> = app
            .document
            .working_set()
            .group(Path::new("/ws/notes"))
            .iter()
            .map(|&at| app.document.working_set().files()[at].leaf())
            .collect();
        assert_eq!(
            notes_group,
            vec!["b.md", "c.md", "a.md"],
            "a.md landed at the bottom of its OWN group, never crossing into archive's"
        );
        let archive_group: Vec<String> = app
            .document
            .working_set()
            .group(Path::new("/ws/archive"))
            .iter()
            .map(|&at| app.document.working_set().files()[at].leaf())
            .collect();
        assert_eq!(
            archive_group,
            vec!["x.md", "y.md"],
            "archive's own group is untouched by a drag that never belonged to it"
        );
    });
}

/// **A ROW THAT NAMES NO FILE REFUSES THE DROP**, leaving the working set
/// exactly as it was — the close-zone press is not this door's concern (it
/// never reaches here), but a stale/past-the-end `from_row` must still be
/// refused rather than panicking or moving an arbitrary row.
#[test]
fn gutter_stack_row_drop_refuses_a_row_that_names_no_file() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_file("/ws/notes/a.md", "a\n")
            .with_file("/ws/notes/b.md", "b\n"),
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
        let before = drawn_labels(&app);
        assert!(
            !app.gutter_stack_row_drop(99, 0),
            "a row past the end of the stack names no file"
        );
        assert_eq!(drawn_labels(&app), before, "a refused drop changes nothing");
    });
}

/// **ESC COLLAPSES AN OPEN EXPANDED PANEL** — driven through the real
/// `Action::Cancel` dispatch (`app/apply.rs`), the same door a live Escape
/// keypress resolves to (`keymap/resolve.rs`), rather than calling
/// `WorkingSet::collapse` directly.
#[test]
fn escape_collapses_an_open_expanded_panel() {
    let _guard = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/notes")
            .with_file("/ws/notes/a.md", "a\n")
            .with_file("/ws/notes/b.md", "b\n"),
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
        app.document.working_set_mut().expand();
        assert!(app.document.working_set().is_expanded());

        let exit = crate::app::schedule::RecordingExit::new();
        app.apply(Action::Cancel, false, &exit, crate::stats::Door::Chord);

        assert!(
            !app.document.working_set().is_expanded(),
            "Escape must collapse the open panel"
        );
    });
}
