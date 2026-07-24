//! src/app/files/tests.rs — the former `app/files.rs` monolith's own test
//! module, moved verbatim (item 56's directory split) — window_title, the
//! Notes-flip pure/impure pair, the DOCUMENT AUTOSAVE ENGINE, save-feedback,
//! rename/move/duplicate verbs, the dictionary + CJK-priority persistence,
//! and the recent-projects/recent-files MRU. Behavior unchanged; only the
//! module path moved (`app::files::tests::foo` unchanged — this file still
//! surfaces as `app::files::tests`, since `files/mod.rs` declares
//! `mod tests;`).

use super::*;
use std::sync::Arc;

// --- window_title (ACCESSIBILITY TIER 1: the window names the document) ---

#[test]
fn window_title_names_a_pathed_file_and_the_active_world() {
    let t = window_title(Some(Path::new("/tmp/notes/draft.md")), false, "Quokka", false);
    assert_eq!(t, "awl - /tmp/notes/draft.md [Quokka]");
}

#[test]
fn window_title_untitled_note_reads_scratch() {
    let t = window_title(None, true, "Tawny", false);
    assert_eq!(t, "awl - scratch [Tawny]");
}

#[test]
fn window_title_bare_launch_scratch_reads_star_scratch_star() {
    let t = window_title(None, false, "Tawny", false);
    assert_eq!(t, "awl - *scratch* [Tawny]");
}

#[test]
fn window_title_untitled_note_and_bare_scratch_are_distinct() {
    assert_ne!(
        window_title(None, true, "Tawny", false),
        window_title(None, false, "Tawny", false)
    );
}

// --- NOTES FLIP round (2026-07-22): the pure toggle-target resolution ---

#[test]
fn notes_flip_target_enters_notes_root_and_remembers_the_leaving_project() {
    let current = Path::new("/w/proj-a");
    let notes = Path::new("/home/me/notes");
    assert_eq!(
        notes_flip_target(current, notes, None),
        NotesFlipTarget::Enter { target: notes.to_path_buf(), remember: current.to_path_buf() },
        "not in notes_root yet: flip there, remembering the project being left"
    );
    // A previously-remembered root is IGNORED while entering (only consulted
    // once we are actually standing IN notes_root) — a stray leftover from an
    // unrelated earlier flip never leaks into a fresh Enter's target.
    assert_eq!(
        notes_flip_target(current, notes, Some(Path::new("/w/stale"))),
        NotesFlipTarget::Enter { target: notes.to_path_buf(), remember: current.to_path_buf() }
    );
}

#[test]
fn notes_flip_target_round_trips_back_to_the_remembered_project() {
    let notes = Path::new("/home/me/notes");
    let prev = Path::new("/w/proj-a");
    // Standing IN notes_root with a remembered previous root: flip BACK.
    assert_eq!(
        notes_flip_target(notes, notes, Some(prev)),
        NotesFlipTarget::Back { target: prev.to_path_buf() }
    );
}

#[test]
fn notes_flip_target_already_home_with_nothing_remembered_is_inert() {
    // Standing IN notes_root but NOTHING was remembered (e.g. a bare launch
    // landed here directly, never having flipped) — no "back" to go to.
    let notes = Path::new("/home/me/notes");
    assert_eq!(notes_flip_target(notes, notes, None), NotesFlipTarget::AlreadyHome);
}

#[test]
fn notes_flip_target_missing_notes_root_is_inert() {
    // The empty-path sentinel (mirrors `persist_page_reset`'s own "no usable
    // config path" idiom): a caller with nowhere to flip TO never touches
    // the filesystem, whether or not a previous root happens to be remembered.
    let current = Path::new("/w/proj-a");
    let empty = Path::new("");
    assert_eq!(notes_flip_target(current, empty, None), NotesFlipTarget::Inert);
    assert_eq!(notes_flip_target(current, empty, Some(Path::new("/w/proj-b"))), NotesFlipTarget::Inert);
    // Even standing "in" the empty path itself, still inert (never a Back).
    assert_eq!(notes_flip_target(empty, empty, Some(current)), NotesFlipTarget::Inert);
}

#[test]
fn notes_flip_round_trips_the_live_app_project_root() {
    // The impure APPLY half, exercised end-to-end against a fake fs (mirrors
    // `switch_project_pushes_and_persists_the_recent_root`'s own shape).
    let fake = Arc::new(
        crate::fs::InMemoryFs::new().with_dir("/w/proj-a").with_dir("/home/me/notes"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(None, PathBuf::from("/w/proj-a"), None, None, Config::empty());
        app.notes_root = PathBuf::from("/home/me/notes");

        // FIRST invocation: enters notes_root, remembering the home desk
        // (proj-a, no open file). With nothing remembered from a prior visit
        // a FRESH untitled note opens (item 59) rather than an arbitrary file.
        app.notes_flip();
        assert_eq!(app.root, PathBuf::from("/home/me/notes"));
        assert_eq!(
            app.notes_return.as_ref().map(|d| d.root.clone()),
            Some(PathBuf::from("/w/proj-a"))
        );
        assert!(app.active.buffer.is_note(), "a first-ever visit opens a fresh untitled quick-note");
        assert!(app.active.buffer.path().is_none(), "the fresh note is unnamed until it gains content");

        // SECOND invocation: flips straight back, consuming the memory.
        app.notes_flip();
        assert_eq!(app.root, PathBuf::from("/w/proj-a"));
        assert!(app.notes_return.is_none());

        // A Notes flip is a VISIT, not a switch: it never touches the sticky
        // project_root pref nor the recent-projects MRU (both stay exactly
        // as a fresh launch left them — mirrors `new_note`'s own C-x n jump).
        assert_eq!(app.config.project_root, None, "the flip never persists a sticky root");
        assert!(app.recent_projects.is_empty(), "the flip never counts as a recent project");
    });
}

#[test]
fn notes_flip_is_a_two_desk_swap_of_root_and_active_buffer() {
    // ITEM 59: "Notes" flips the WHOLE writing context — the active BUFFER
    // travels with the root, not only the folder. Project A's file → the
    // remembered notes file → back to EXACTLY A's file, and re-entering
    // restores the last notes file (with its live buffer), never a fresh one.
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_file("/w/proj-a/a.md", "alpha body")
            .with_file("/home/me/notes/n.md", "note body"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(None, PathBuf::from("/w/proj-a"), None, None, Config::empty());
        app.notes_root = PathBuf::from("/home/me/notes");
        // Home desk: an open project file, with content.
        app.load_path(PathBuf::from("/w/proj-a/a.md"));
        assert_eq!(app.active.buffer.path(), Some(Path::new("/w/proj-a/a.md")));
        assert!(app.active.buffer.text().contains("alpha"));

        // ENTER Notes (first visit): root re-scoped, home file remembered,
        // a fresh untitled note opened. The home buffer is parked, not lost.
        app.notes_flip();
        assert_eq!(app.root, PathBuf::from("/home/me/notes"));
        assert_eq!(
            app.notes_return.as_ref().and_then(|d| d.file.clone()),
            Some(PathBuf::from("/w/proj-a/a.md")),
            "the home desk's active file is the remembered return target"
        );
        assert!(app.active.buffer.is_note() && app.active.buffer.path().is_none());

        // Work IN the notes desk: open a real notes file.
        app.load_path(PathBuf::from("/home/me/notes/n.md"));
        assert!(app.active.buffer.text().contains("note body"));

        // FLIP BACK: exact prior project AND its active buffer/view restored.
        app.notes_flip();
        assert_eq!(app.root, PathBuf::from("/w/proj-a"));
        assert_eq!(app.active.buffer.path(), Some(Path::new("/w/proj-a/a.md")));
        assert!(app.active.buffer.text().contains("alpha"), "A's buffer came back, not re-read stale");
        assert!(app.notes_return.is_none(), "the return memory is consumed");
        assert_eq!(
            app.notes_last_file,
            Some(PathBuf::from("/home/me/notes/n.md")),
            "the notes desk's last active file is remembered for re-entry"
        );

        // RE-ENTER: the last notes file returns (its live buffer), NOT a
        // fresh untitled note and NOT an arbitrary pick.
        app.notes_flip();
        assert_eq!(app.root, PathBuf::from("/home/me/notes"));
        assert_eq!(app.active.buffer.path(), Some(Path::new("/home/me/notes/n.md")));
        assert!(app.active.buffer.text().contains("note body"));

        // Still transient the whole way: no sticky root, no recent-project MRU.
        assert_eq!(app.config.project_root, None);
        assert!(app.recent_projects.is_empty());
    });
}

#[test]
fn notes_flip_parks_a_dirty_home_buffer_and_restores_it_unsaved() {
    // ITEM 59: a DIRTY home buffer is PARKED across the visit — never
    // discarded — and comes back with its unsaved edit intact. Autosave is
    // turned OFF here to isolate the PARKING guarantee from awl's standard
    // autosave-on-switch (which set_root fires for every project switch, so
    // the flip never SPURIOUSLY saves — it rides the one existing door).
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_file("/w/proj-a/a.md", "alpha body")
            .with_dir("/home/me/notes"),
    );
    crate::fs::with_fs(fake, || {
        let mut cfg = Config::empty();
        cfg.autosave = Some(false);
        let mut app = App::new(None, PathBuf::from("/w/proj-a"), None, None, cfg);
        app.notes_root = PathBuf::from("/home/me/notes");
        app.load_path(PathBuf::from("/w/proj-a/a.md"));
        // Dirty the home buffer WITHOUT saving.
        app.active.buffer.set_text("alpha body — UNSAVED EDIT");
        assert!(app.active.buffer.is_dirty());

        app.notes_flip(); // enter notes (parks the dirty home buffer)
        // Disk untouched: with autosave off nothing was written on the flip.
        assert_eq!(
            crate::fs::active().read_to_string(Path::new("/w/proj-a/a.md")).unwrap(),
            "alpha body",
            "with autosave off the flip writes nothing to disk"
        );

        app.notes_flip(); // back home — the unsaved edit survives
        assert_eq!(app.active.buffer.path(), Some(Path::new("/w/proj-a/a.md")));
        assert!(app.active.buffer.text().contains("UNSAVED EDIT"), "the dirty edit came back intact");
        assert!(app.active.buffer.is_dirty(), "still dirty — the parked buffer was never discarded");
    });
}

#[test]
fn notes_flip_denied_root_change_leaves_both_desks_untouched() {
    // ITEM 59: root change + buffer activation are ONE transaction. If the
    // root change is refused, BOTH desks and the remembered return target
    // stay exactly as they were. Modeled with an EMPTY notes_root (the "no
    // usable folder" sentinel → `Inert`): the command is a clean no-op.
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_file("/w/proj-a/a.md", "alpha"));
    crate::fs::with_fs(fake, || {
        let mut app = App::new(None, PathBuf::from("/w/proj-a"), None, None, Config::empty());
        app.notes_root = PathBuf::new(); // no usable notes root
        app.load_path(PathBuf::from("/w/proj-a/a.md"));

        app.notes_flip();
        assert_eq!(app.root, PathBuf::from("/w/proj-a"), "root unchanged");
        assert_eq!(app.active.buffer.path(), Some(Path::new("/w/proj-a/a.md")), "active buffer unchanged");
        assert!(app.notes_return.is_none(), "no return target remembered");
    });
}

// --- SAVE-FEEDBACK round: the dirty edited-marker, dirty × scratch/note/file ---

#[test]
fn window_title_dirty_pathed_file_gets_the_leading_marker() {
    let t = window_title(Some(Path::new("/tmp/notes/draft.md")), false, "Quokka", true);
    assert_eq!(t, "awl - \u{2022} /tmp/notes/draft.md [Quokka]");
}

#[test]
fn window_title_clean_pathed_file_has_no_marker() {
    let t = window_title(Some(Path::new("/tmp/notes/draft.md")), false, "Quokka", false);
    assert!(!t.contains('\u{2022}'), "a clean buffer's title carries no edited marker");
}

#[test]
fn window_title_dirty_untitled_note_gets_the_marker_too() {
    let t = window_title(None, true, "Tawny", true);
    assert_eq!(t, "awl - \u{2022} scratch [Tawny]");
}

#[test]
fn window_title_dirty_bare_scratch_gets_the_marker_too() {
    let t = window_title(None, false, "Tawny", true);
    assert_eq!(t, "awl - \u{2022} *scratch* [Tawny]");
}

#[test]
fn window_title_dirty_is_only_ever_a_leading_marker_insertion() {
    // Every other field held fixed — the dirty flip is EXACTLY inserting
    // "• " right after "awl - " and nothing else in the string moves.
    let clean = window_title(Some(Path::new("a.md")), false, "Bilby", false);
    let dirty = window_title(Some(Path::new("a.md")), false, "Bilby", true);
    assert_ne!(clean, dirty);
    assert_eq!(dirty, format!("awl - \u{2022} a.md [Bilby]"));
    assert_eq!(clean, format!("awl - a.md [Bilby]"));
    assert_eq!(dirty, clean.replacen("awl - ", "awl - \u{2022} ", 1));
}

#[test]
fn update_title_uses_the_same_pure_window_title() {
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.active.buffer.set_text("hello");
    // No live `gpu`/window in a hermetic App (see `App::update_title`'s gate) —
    // this proves the call is a harmless no-op off a real window and exercises
    // the same code path `resumed()`/`load_path`/theme-switch drive.
    app.update_title();
}

#[test]
fn image_width_hint_write_back_is_one_undoable_edit_that_keeps_the_cursor() {
    // The v2 drag-resize WRITE-BACK over a real Buffer (the buffer/markdown seam):
    // insert/replace `|NNN` in the alt as ONE undoable edit, restoring the mouse
    // caret rather than moving it to the edit end.
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.active.buffer.set_text("![a cat](cat.png)\ntail\n");
    assert!(app.active.buffer.is_markdown(), "a no-path scratch buffer is markdown");
    // Caret parked on the SECOND line (past the image span) — a mouse drag must
    // never move it.
    let cursor = app.active.buffer.text().chars().count() - 1;
    app.active.buffer.set_cursor(cursor);

    // INSERT: a 300px drag stamps `|300` into the hint-less alt (round from 300.4).
    app.write_back_image_width((0, 17), 300.4);
    assert_eq!(app.active.buffer.text(), "![a cat|300](cat.png)\ntail\n");
    // The caret shifted by the +4-char insertion (stayed on its glyph), never
    // jumped to the edit end.
    assert_eq!(app.active.buffer.cursor_char(), cursor + 4, "caret past the edit shifts by the delta");

    // ONE undoable edit: a single Cmd-Z restores the pre-drag text exactly.
    app.active.buffer.undo();
    assert_eq!(app.active.buffer.text(), "![a cat](cat.png)\ntail\n", "one Cmd-Z restores the size");

    // REPLACE: an existing `|NNN` is swapped in place (still one edit); a caret
    // BEFORE the edit never moves.
    app.active.buffer.set_text("![a cat|300](cat.png)\n");
    app.active.buffer.set_cursor(0);
    app.write_back_image_width((0, 21), 128.0);
    assert_eq!(app.active.buffer.text(), "![a cat|128](cat.png)\n");
    assert_eq!(app.active.buffer.cursor_char(), 0, "a caret before the edit stays put");
    app.active.buffer.undo();
    assert_eq!(app.active.buffer.text(), "![a cat|300](cat.png)\n", "one Cmd-Z restores the prior hint");

    // No-op guard: re-committing the SAME width records nothing (keeps the timeline
    // meaningful) — the text is unchanged and a following undo reaches PAST it.
    app.active.buffer.set_text("![a cat|200](cat.png)\n");
    app.active.buffer.set_cursor(3);
    app.write_back_image_width((0, 21), 200.0);
    assert_eq!(app.active.buffer.text(), "![a cat|200](cat.png)\n", "same width is a no-op");
    assert_eq!(app.active.buffer.cursor_char(), 3, "a no-op never disturbs the caret");
}

#[test]
fn trash_asset_moves_the_file_and_removes_the_row_via_the_fake_seam() {
    let mut app = App::new_hermetic(None, PathBuf::from("/proj"), Config::empty());
    // Arm the ASSET CLEANER picker with two orphans (the scan is unit-tested in
    // `assets.rs`; here we drive the App's trash + row-removal wiring).
    let mk = |rel: &str| crate::assets::Orphan {
        rel: rel.to_string(),
        name: rel.rsplit('/').next().unwrap().to_string(),
        parent: rel.rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_default(),
        size: Some(42),
    };
    app.overlay = Some(crate::overlay::OverlayState::new_assets(vec![
        mk("assets/keep.png"),
        mk("assets/drop.png"),
    ]));

    let fake = Arc::new(crate::assets::FakeTrash::default());
    let recorder = fake.clone();
    crate::assets::with_trash(fake, || {
        app.trash_asset("assets/drop.png".to_string());
    });

    // The file was sent to the (fake) Trash at the ROOT-joined absolute path.
    assert_eq!(
        recorder.trashed.lock().unwrap().as_slice(),
        &[app.root.join("assets/drop.png")],
    );
    // The picker STAYS OPEN and the trashed row LEAVES the list.
    let ov = app.overlay.as_ref().expect("picker stays open after a trash");
    assert_eq!(ov.item_strings(), vec!["keep.png"]);
    assert!(ov.notice.is_empty(), "a successful trash shows no error notice");
}

/// A trash FAILURE (a backend that errors) LEAVES the row + shows a calm notice —
/// the list never shrinks unless the file actually went to the Trash.
#[test]
fn trash_asset_failure_keeps_the_row_and_notes_the_error() {
    use std::path::Path;
    struct FailTrash;
    impl crate::assets::TrashCan for FailTrash {
        fn trash(&self, _p: &Path) -> Result<(), String> {
            Err("nope".to_string())
        }
    }
    let mut app = App::new_hermetic(None, PathBuf::from("/proj"), Config::empty());
    app.overlay = Some(crate::overlay::OverlayState::new_assets(vec![
        crate::assets::Orphan {
            rel: "assets/x.png".into(),
            name: "x.png".into(),
            parent: "assets".into(),
            size: Some(1),
        },
    ]));
    crate::assets::with_trash(Arc::new(FailTrash), || {
        app.trash_asset("assets/x.png".to_string());
    });
    let ov = app.overlay.as_ref().unwrap();
    assert_eq!(ov.item_strings(), vec!["x.png"], "a failed trash keeps the row");
    assert!(ov.notice.contains("Trash"), "a calm notice explains the failure");
}

// ── NO-PATH PASTE SAVES FIRST (the paste-image seam, `app/apply.rs::
// try_paste_image`) ──────────────────────────────────────────────────────

/// A bare SCRATCH buffer (never summoned via C-x n) with real text in it: the
/// pre-paste save promotes it into a note rooted at `notes_root` and derives
/// a path from its first line — the SAME name/derivation a real quick note's
/// first autosave would produce. Proves the "gains a path under notes_root"
/// half of the paste-image contract.
#[test]
fn ensure_note_named_before_paste_promotes_a_scratch_buffer_and_saves_under_notes_root() {
    use crate::fs::{FileSystem, InMemoryFs};
    let fake = Arc::new(InMemoryFs::new());
    crate::fs::with_fs(fake.clone(), || {
        let mut app = App::new(
            None,
            PathBuf::from("/proj"),
            None,
            Some(PathBuf::from("/notes")),
            Config::empty(),
        );
        assert!(!app.active.buffer.is_note(), "a bare launch buffer starts as plain scratch");
        assert!(app.active.buffer.path().is_none());
        app.active.buffer.set_text("My Pasted Screenshot\n\nsome body text\n");

        app.ensure_note_named_before_paste();

        assert!(app.active.buffer.is_note(), "promoted into a note living under notes_root");
        let path = app.active.buffer.path().expect("gained a path").to_path_buf();
        assert!(
            path.starts_with("/notes"),
            "the derived path lives under notes_root: {}",
            path.display()
        );
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("md"));
        // The slug came from the first non-empty line, matching the notes
        // system's own derivation (`buffer::note_stem`).
        assert!(
            path.file_stem().unwrap().to_string_lossy().contains("pasted-screenshot"),
            "filename derives from the first line: {}",
            path.display()
        );
        // The save actually landed on disk (not just an in-memory path stamp).
        assert_eq!(
            fake.read_to_string(&path).unwrap(),
            "My Pasted Screenshot\n\nsome body text\n"
        );
        // `App.file` + the title track the freshly-named note, exactly like a
        // real quick note's first autosave.
        assert_eq!(app.active.buffer.path(), Some(path.as_path()));
    });
}

/// An ALREADY-STARTED note (`note_dir` set, still unnamed) is left pointed at
/// its own dir — never re-promoted/re-rooted at `notes_root` a second time.
#[test]
fn ensure_note_named_before_paste_leaves_an_in_progress_note_dir_alone() {
    use crate::fs::InMemoryFs;
    let fake = Arc::new(InMemoryFs::new());
    crate::fs::with_fs(fake.clone(), || {
        let mut app = App::new(None, PathBuf::from("/proj"), None, Some(PathBuf::from("/notes")), Config::empty());
        app.active.buffer.start_note(PathBuf::from("/elsewhere"));
        app.active.buffer.set_text("Elsewhere Note\n");

        app.ensure_note_named_before_paste();

        let path = app.active.buffer.path().expect("gained a path");
        assert!(
            path.starts_with("/elsewhere"),
            "an in-progress note's own dir is respected, not overridden: {}",
            path.display()
        );
    });
}

/// An EMPTY buffer (no first line to derive a name from) fails the save
/// quietly and stays path-less — the caller (`try_paste_image`) falls back to
/// its pre-existing absolute data-root location rather than blocking the
/// paste. Also proves the promotion side effect (now a note) survives the
/// failed save, matching what typing-then-pausing would do from here.
#[test]
fn ensure_note_named_before_paste_on_an_empty_buffer_stays_path_less() {
    use crate::fs::InMemoryFs;
    let fake = Arc::new(InMemoryFs::new());
    crate::fs::with_fs(fake, || {
        let mut app = App::new(None, PathBuf::from("/proj"), None, Some(PathBuf::from("/notes")), Config::empty());
        assert_eq!(app.active.buffer.text(), "", "a fresh scratch buffer starts empty");

        app.ensure_note_named_before_paste();

        assert!(app.active.buffer.path().is_none(), "no first line to derive a name from");
        assert!(app.active.buffer.is_note(), "promoted regardless — matches typing-then-pausing");
    });
}

#[test]
fn persist_cjk_priority_writes_the_whole_ordered_ladder_to_config() {
    // App::persist_cjk_priority (fired by Effect::OverlayAccept(CjkLang, ..)
    // after the core promotes + sets the live global) writes the WHOLE
    // ordered ladder as a TOML array and mirrors it into `self.config`.
    let _g = crate::testlock::serial();
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/w/proj"));
    crate::fs::with_fs(fake, || {
        let mut config = Config::empty();
        config.path = PathBuf::from("/cfg/config.toml");
        let mut app = App::new(None, PathBuf::from("/w/proj"), None, None, config);

        // The core already promoted Korean to the front (mirrors what
        // `actions::overlay_nav`'s CjkLang accept branch does).
        crate::frontmatter::set_cjk_priority(&crate::frontmatter::promote_cjk_priority(
            crate::frontmatter::Lang::Ko,
        ));
        app.persist_cjk_priority();

        let want = vec![
            crate::frontmatter::Lang::Ko,
            crate::frontmatter::Lang::Ja,
            crate::frontmatter::Lang::ZhHans,
            crate::frontmatter::Lang::ZhHant,
        ];
        assert_eq!(app.config.cjk_priority, Some(want.clone()), "mirrored in-memory");
        let reloaded = Config::load(PathBuf::from("/cfg/config.toml"));
        assert_eq!(reloaded.cjk_priority, Some(want), "persisted to disk");

        crate::frontmatter::set_cjk_priority(&crate::frontmatter::DEFAULT_CJK_PRIORITY);
    });
}

// ── SPELL-CHECK TOGGLE x CONFIG RELOAD (the spell-toggle-x-theme report):
// `App::reload_config` re-applies `spellcheck` LIVE (unlike theme/caret/
// dictionary, which only apply once at launch) so a hand-edited
// `spellcheck = false` in the Settings buffer takes effect on save. That
// special-casing must still obey `Config::apply_sticky_globals`'s law: an
// ABSENT key leaves the global AS-IS, never forces a default. ──────────

/// THE FIX this round makes: an absent `spellcheck` key on disk must NOT
/// force the global to the built-in ON default — it must leave whatever is
/// currently running untouched, exactly like `apply_sticky_globals`
/// (`apply_sticky_globals_restores_spellcheck` in `config/tests.rs` pins
/// the same law at the launch seam). Reachable if an earlier
/// `persist_spellcheck`/`setting_toggle` write never reached disk (I/O
/// error, unresolvable config path) while the runtime toggle sat OFF: the
/// very next config-buffer save or Keybindings rebind — both call
/// `reload_config` — used to silently flip a still-intended-OFF toggle
/// back ON.
#[test]
fn reload_config_absent_spellcheck_key_leaves_global_untouched() {
    let _sp = crate::testlock::serial();
    let saved = crate::spell::spellcheck_on();
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/w/proj"));
    crate::fs::with_fs(fake, || {
        let cfg_path = PathBuf::from("/cfg/config.toml");
        // A config file that has never recorded a spellcheck preference —
        // the common case for anyone who has never touched the toggle.
        let mut config = Config::empty();
        config.path = cfg_path.clone();
        assert_eq!(config.spellcheck, None);
        let mut app = App::new(None, PathBuf::from("/w/proj"), None, None, config);

        // The runtime global sits OFF (e.g. a toggle whose persist never
        // landed), but the disk file still has no `spellcheck` key.
        crate::spell::set_spellcheck_on(false);
        assert_eq!(Config::load(cfg_path.clone()).spellcheck, None, "disk still absent");

        app.reload_config();

        assert!(
            !crate::spell::spellcheck_on(),
            "an absent disk key must leave the global AS-IS, not force it back ON"
        );

        // The other direction too: ON stays ON across the same reload.
        crate::spell::set_spellcheck_on(true);
        app.reload_config();
        assert!(crate::spell::spellcheck_on(), "and leaves an ON global alone too");
    });
    crate::spell::set_spellcheck_on(saved);
}

/// The POSITIVE half of the same seam: a hand-edited `spellcheck = false`
/// saved into the config file DOES take effect immediately on
/// `reload_config` — the documented "takes effect immediately, exactly
/// like the Toggle Spellcheck command" contract (`reload_config`'s own
/// doc comment). Round-trips through the real `persist_pref`/`Config::load`
/// pair, not a hand-built `Config`.
#[test]
fn reload_config_reapplies_a_persisted_spellcheck_value_immediately() {
    let _sp = crate::testlock::serial();
    let saved = crate::spell::spellcheck_on();
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/w/proj"));
    crate::fs::with_fs(fake, || {
        let cfg_path = PathBuf::from("/cfg/config.toml");
        let mut config = Config::empty();
        config.path = cfg_path.clone();
        let mut app = App::new(None, PathBuf::from("/w/proj"), None, None, config);
        crate::spell::set_spellcheck_on(true);

        // Toggle OFF through the real seam (mirrors `Action::ToggleSpellcheck`
        // + `App::persist_spellcheck`), then reload — mirrors saving the
        // Settings buffer right after a hand-edit.
        crate::spell::toggle();
        app.persist_spellcheck();
        assert_eq!(app.config.spellcheck, Some(false), "persist mirrors self.config");

        crate::spell::set_spellcheck_on(true); // simulate reload starting from a stale global
        app.reload_config();
        assert!(!crate::spell::spellcheck_on(), "the persisted OFF value re-applies on reload");
    });
    crate::spell::set_spellcheck_on(saved);
}

// ── ADD TO DICTIONARY (item 39): a plain-text word list beside config.toml,
// GLOBAL, hand-editable, ZERO-NETWORK. "Add to dictionary" both silences the
// word live AND appends it to the file; startup loads the file so an added
// word never squiggles again, across a restart. ─────────────────────────

/// `App::add_to_dictionary` silences the word in the LIVE checker AND appends
/// it (one word per line) to `dictionary.txt` beside `config.toml`. Re-adding
/// the same word never duplicates the line.
#[test]
fn add_to_dictionary_persists_the_word_and_silences_it_live() {
    let _sp = crate::testlock::serial();
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/w/proj").with_dir("/cfg"));
    crate::fs::with_fs(fake, || {
        let mut config = Config::empty();
        config.path = PathBuf::from("/cfg/config.toml");
        let mut app = App::new(None, PathBuf::from("/w/proj"), None, None, config);
        // Precondition: a made-up word squiggles.
        assert!(!app.spell.as_ref().unwrap().check("wrold"));

        app.add_to_dictionary("wrold");
        // Live: the checker now accepts it.
        assert!(app.spell.as_ref().unwrap().check("wrold"), "silenced in the live checker");
        // Persisted: the file beside config.toml holds exactly one line.
        let dict = PathBuf::from("/cfg/dictionary.txt");
        let text = crate::fs::active().read_to_string(&dict).expect("the file was written");
        assert_eq!(crate::spell::parse_dictionary(&text), vec!["wrold"]);

        // Re-adding it is a no-op on disk (no duplicate line) and still silent.
        app.add_to_dictionary("wrold");
        let text2 = crate::fs::active().read_to_string(&dict).unwrap();
        assert_eq!(crate::spell::parse_dictionary(&text2), vec!["wrold"], "no duplicate line");
    });
}

/// THE RESTART GUARANTEE: a word already in the on-disk personal dictionary is
/// loaded AT STARTUP (`App::new` → `load_user_dictionary`), so a fresh App
/// never squiggles it — the "never squiggles again, including across a restart"
/// contract. Seeds the file first, THEN constructs, and asserts the just-built
/// App's checker already accepts the word (proving construction did the load).
#[test]
fn startup_loads_the_personal_dictionary_so_an_added_word_never_squiggles_across_a_restart() {
    let _sp = crate::testlock::serial();
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/w/proj").with_dir("/cfg"));
    crate::fs::with_fs(fake, || {
        // A prior session already wrote the word list (hand-edited shape: a
        // header comment + one word per line).
        crate::fs::write_atomic(
            Path::new("/cfg/dictionary.txt"),
            b"# my words\nwrold\n",
        )
        .unwrap();
        let mut config = Config::empty();
        config.path = PathBuf::from("/cfg/config.toml");
        // Fresh App (the "restart"): its startup load reads the file.
        let app = App::new(None, PathBuf::from("/w/proj"), None, None, config);
        assert!(
            app.spell.as_ref().unwrap().check("wrold"),
            "startup loaded the personal dictionary — the word never squiggles across a restart"
        );
        // A word NOT in the file still squiggles (the load is scoped to the file).
        assert!(!app.spell.as_ref().unwrap().check("teh"));
    });
}

#[test]
fn switch_project_pushes_and_persists_the_recent_root() {
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/w/proj-a")
            .with_dir("/w/proj-b"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(None, PathBuf::from("/w/proj-a"), None, None, Config::empty());
        // Fresh launch: no recents yet (missing store).
        assert!(app.recent_projects.is_empty());

        // Switching to two projects pushes each to the FRONT, newest-first.
        app.switch_project(PathBuf::from("/w/proj-a"));
        app.switch_project(PathBuf::from("/w/proj-b"));
        assert_eq!(
            app.recent_projects,
            vec![PathBuf::from("/w/proj-b"), PathBuf::from("/w/proj-a")],
        );
        assert_eq!(app.root, PathBuf::from("/w/proj-b"), "root followed the switch");

        // Re-switching to proj-a moves it to the front (dedup, never a dupe).
        app.switch_project(PathBuf::from("/w/proj-a"));
        assert_eq!(
            app.recent_projects,
            vec![PathBuf::from("/w/proj-a"), PathBuf::from("/w/proj-b")],
        );

        // The list is PERSISTED: a second launch reads it back (via the store).
        let reloaded = crate::recents::load(&crate::recents::recents_path());
        assert_eq!(
            reloaded,
            vec![PathBuf::from("/w/proj-a"), PathBuf::from("/w/proj-b")],
        );
    });
}

#[test]
fn opening_files_pushes_them_onto_the_recent_files_mru_and_persists() {
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_file("/w/proj/a.md", "a")
            .with_file("/w/proj/b.md", "b")
            .with_file("/w/proj/c.md", "c"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(None, PathBuf::from("/w/proj"), None, None, Config::empty());
        assert!(app.recent_files.is_empty(), "fresh launch: empty MRU");

        // Opening three files pushes each to the FRONT (most-recent first). Both
        // load_path branches route here; a fresh disk read is the None branch.
        app.load_path(PathBuf::from("/w/proj/a.md"));
        app.load_path(PathBuf::from("/w/proj/b.md"));
        app.load_path(PathBuf::from("/w/proj/c.md"));
        assert_eq!(
            app.recent_files,
            vec![
                PathBuf::from("/w/proj/c.md"),
                PathBuf::from("/w/proj/b.md"),
                PathBuf::from("/w/proj/a.md"),
            ],
        );

        // Re-opening a.md (the buffer-registry SWITCH branch) moves it to the
        // front — dedup, never a dupe.
        app.load_path(PathBuf::from("/w/proj/a.md"));
        assert_eq!(
            app.recent_files,
            vec![
                PathBuf::from("/w/proj/a.md"),
                PathBuf::from("/w/proj/c.md"),
                PathBuf::from("/w/proj/b.md"),
            ],
        );

        // Re-selecting the ALREADY-ACTIVE file is a no-op (load_path's early
        // return), so the MRU is untouched — no re-order, no dupe.
        app.load_path(PathBuf::from("/w/proj/a.md"));
        assert_eq!(app.recent_files.len(), 3, "no-op reopen never re-orders / dupes");
        assert_eq!(app.recent_files[0], PathBuf::from("/w/proj/a.md"));

        // PERSISTED: a second launch reads the MRU back through the store.
        assert_eq!(crate::recent_files::load(), app.recent_files);
    });
}

#[test]
fn app_new_loads_the_persisted_recent_projects() {
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/w/proj-a"));
    crate::fs::with_fs(fake, || {
        // Pre-seed the store, then launch: App::new loads it into the field.
        crate::recents::save(
            &crate::recents::recents_path(),
            &[PathBuf::from("/w/proj-a"), PathBuf::from("/w/proj-b")],
        )
        .unwrap();
        let app = App::new(None, PathBuf::from("/w/proj-a"), None, None, Config::empty());
        assert_eq!(
            app.recent_projects,
            vec![PathBuf::from("/w/proj-a"), PathBuf::from("/w/proj-b")],
        );
    });
}
