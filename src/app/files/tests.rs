//! src/app/files/tests.rs — the former `app/files.rs` monolith's own test
//! module, moved verbatim (item 56's directory split) — window_title, the
//! DOCUMENT AUTOSAVE ENGINE, save-feedback, rename/move/duplicate verbs, the
//! dictionary + CJK-priority persistence, and the recent-projects/
//! recent-files MRU. Item 76 retired the two-desk "Notes" flip tests that
//! used to live here. Module path: `app::files::tests`, since `files/mod.rs`
//! declares `mod tests;`.

use super::*;
use std::sync::Arc;

// --- window_title (ACCESSIBILITY TIER 1: the window names the document) ---

#[test]
fn window_title_names_a_pathed_file_and_the_active_world() {
    let t = window_title(
        Some(Path::new("/tmp/notes/draft.md")),
        false,
        "Quokka",
        false,
    );
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

// --- SAVE-FEEDBACK round: the dirty edited-marker, dirty × scratch/note/file ---

#[test]
fn window_title_dirty_pathed_file_gets_the_leading_marker() {
    let t = window_title(
        Some(Path::new("/tmp/notes/draft.md")),
        false,
        "Quokka",
        true,
    );
    assert_eq!(t, "awl - \u{2022} /tmp/notes/draft.md [Quokka]");
}

#[test]
fn window_title_clean_pathed_file_has_no_marker() {
    let t = window_title(
        Some(Path::new("/tmp/notes/draft.md")),
        false,
        "Quokka",
        false,
    );
    assert!(
        !t.contains('\u{2022}'),
        "a clean buffer's title carries no edited marker"
    );
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
    assert!(
        app.active.buffer.is_markdown(),
        "a no-path scratch buffer is markdown"
    );
    // Caret parked on the SECOND line (past the image span) — a mouse drag must
    // never move it.
    let cursor = app.active.buffer.text().chars().count() - 1;
    app.active.buffer.set_cursor(cursor);

    // INSERT: a 300px drag stamps `|300` into the hint-less alt (round from 300.4).
    app.write_back_image_width((0, 17), 300.4);
    assert_eq!(app.active.buffer.text(), "![a cat|300](cat.png)\ntail\n");
    // The caret shifted by the +4-char insertion (stayed on its glyph), never
    // jumped to the edit end.
    assert_eq!(
        app.active.buffer.cursor_char(),
        cursor + 4,
        "caret past the edit shifts by the delta"
    );

    // ONE undoable edit: a single Cmd-Z restores the pre-drag text exactly.
    app.active.buffer.undo();
    assert_eq!(
        app.active.buffer.text(),
        "![a cat](cat.png)\ntail\n",
        "one Cmd-Z restores the size"
    );

    // REPLACE: an existing `|NNN` is swapped in place (still one edit); a caret
    // BEFORE the edit never moves.
    app.active.buffer.set_text("![a cat|300](cat.png)\n");
    app.active.buffer.set_cursor(0);
    app.write_back_image_width((0, 21), 128.0);
    assert_eq!(app.active.buffer.text(), "![a cat|128](cat.png)\n");
    assert_eq!(
        app.active.buffer.cursor_char(),
        0,
        "a caret before the edit stays put"
    );
    app.active.buffer.undo();
    assert_eq!(
        app.active.buffer.text(),
        "![a cat|300](cat.png)\n",
        "one Cmd-Z restores the prior hint"
    );

    // No-op guard: re-committing the SAME width records nothing (keeps the timeline
    // meaningful) — the text is unchanged and a following undo reaches PAST it.
    app.active.buffer.set_text("![a cat|200](cat.png)\n");
    app.active.buffer.set_cursor(3);
    app.write_back_image_width((0, 21), 200.0);
    assert_eq!(
        app.active.buffer.text(),
        "![a cat|200](cat.png)\n",
        "same width is a no-op"
    );
    assert_eq!(
        app.active.buffer.cursor_char(),
        3,
        "a no-op never disturbs the caret"
    );
}

#[test]
fn trash_asset_moves_the_file_and_removes_the_row_via_the_fake_seam() {
    let mut app = App::new_hermetic(None, PathBuf::from("/proj"), Config::empty());
    // Arm the ASSET CLEANER picker with two orphans (the scan is unit-tested in
    // `assets.rs`; here we drive the App's trash + row-removal wiring).
    let mk = |rel: &str| crate::assets::Orphan {
        rel: rel.to_string(),
        name: rel.rsplit('/').next().unwrap().to_string(),
        parent: rel
            .rsplit_once('/')
            .map(|(d, _)| d.to_string())
            .unwrap_or_default(),
        size: Some(42),
    };
    app.workspace_state
        .install_overlay_for_test(crate::overlay::OverlayState::new_assets(vec![
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
        &[app.project_location.root.join("assets/drop.png")],
    );
    // The picker STAYS OPEN and the trashed row LEAVES the list.
    let ov = app
        .workspace_state
        .overlay()
        .expect("picker stays open after a trash");
    assert_eq!(ov.item_strings(), vec!["keep.png"]);
    assert!(
        ov.notice.is_empty(),
        "a successful trash shows no error notice"
    );
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
    app.workspace_state
        .install_overlay_for_test(crate::overlay::OverlayState::new_assets(vec![
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
    let ov = app.workspace_state.overlay().unwrap();
    assert_eq!(
        ov.item_strings(),
        vec!["x.png"],
        "a failed trash keeps the row"
    );
    assert!(
        ov.notice.contains("Trash"),
        "a calm notice explains the failure"
    );
}

// ── NO-PATH PASTE SAVES FIRST (the paste-image seam, `app/apply.rs::
// paste_image_reference`) ────────────────────────────────────────────────

/// A bare SCRATCH buffer (never summoned via Cmd-N) with real text in it: the
/// pre-paste save promotes it into an unnamed fresh document rooted at the
/// ACTIVE folder (item 76 — `self.root`, NOT the `default_folder` fallback)
/// and derives a path from its first line — the SAME name/derivation a real
/// fresh document's first autosave would produce. Proves the "gains a path
/// under the active folder" half of the paste-image contract.
#[test]
fn ensure_note_named_before_paste_promotes_a_scratch_buffer_and_saves_under_the_active_folder() {
    use crate::fs::{FileSystem, InMemoryFs};
    let fake = Arc::new(InMemoryFs::new());
    crate::fs::with_fs(fake.clone(), || {
        let mut app = App::new(
            None,
            PathBuf::from("/proj"),
            None,
            Some(PathBuf::from("/notes")), // default_folder: irrelevant once running
            Config::empty(),
        );
        assert!(
            !app.active.buffer.is_unnamed_fresh(),
            "a bare launch buffer starts as plain scratch"
        );
        assert!(app.active.buffer.path().is_none());
        app.active
            .buffer
            .set_text("My Pasted Screenshot\n\nsome body text\n");

        app.ensure_note_named_before_paste();

        // ONE-SHOT NAMING (item 76): the promotion AND the derive-a-name save
        // happen in this one call, so by the time it returns the buffer is
        // already an ORDINARY pathed document, not a lasting note identity.
        assert!(
            !app.active.buffer.is_unnamed_fresh(),
            "named once — an ordinary file now"
        );
        let path = app
            .active
            .buffer
            .path()
            .expect("gained a path")
            .to_path_buf();
        assert!(
            path.starts_with("/proj"),
            "the derived path lives under the ACTIVE folder, not default_folder: {}",
            path.display()
        );
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("md"));
        // The slug came from the first non-empty line, matching the
        // fresh-document system's own derivation (`buffer::note_stem`).
        assert!(
            path.file_stem()
                .unwrap()
                .to_string_lossy()
                .contains("pasted-screenshot"),
            "filename derives from the first line: {}",
            path.display()
        );
        // The save actually landed on disk (not just an in-memory path stamp).
        assert_eq!(
            fake.read_to_string(&path).unwrap(),
            "My Pasted Screenshot\n\nsome body text\n"
        );
        // `App.file` + the title track the freshly-named document, exactly like
        // a real fresh document's first autosave.
        assert_eq!(app.active.buffer.path(), Some(path.as_path()));
    });
}

/// An ALREADY-STARTED fresh document (`note_dir` set, still unnamed) is left
/// pointed at its own dir — never re-promoted/re-rooted a second time.
#[test]
fn ensure_note_named_before_paste_leaves_an_in_progress_note_dir_alone() {
    use crate::fs::InMemoryFs;
    let fake = Arc::new(InMemoryFs::new());
    crate::fs::with_fs(fake.clone(), || {
        let mut app = App::new(
            None,
            PathBuf::from("/proj"),
            None,
            Some(PathBuf::from("/notes")),
            Config::empty(),
        );
        app.active
            .buffer
            .start_fresh_doc(PathBuf::from("/elsewhere"));
        app.active.buffer.set_text("Elsewhere Note\n");

        app.ensure_note_named_before_paste();

        let path = app.active.buffer.path().expect("gained a path");
        assert!(
            path.starts_with("/elsewhere"),
            "an in-progress fresh document's own dir is respected, not overridden: {}",
            path.display()
        );
    });
}

/// An EMPTY buffer (no first line to derive a name from) fails the save
/// quietly and stays path-less — the caller (`paste_image_reference`) falls back to
/// its pre-existing absolute data-root location rather than blocking the
/// paste. Also proves the promotion side effect (now a fresh document)
/// survives the failed save, matching what typing-then-pausing would do from
/// here.
#[test]
fn ensure_note_named_before_paste_on_an_empty_buffer_stays_path_less() {
    use crate::fs::InMemoryFs;
    let fake = Arc::new(InMemoryFs::new());
    crate::fs::with_fs(fake, || {
        let mut app = App::new(
            None,
            PathBuf::from("/proj"),
            None,
            Some(PathBuf::from("/notes")),
            Config::empty(),
        );
        assert_eq!(
            app.active.buffer.text(),
            "",
            "a fresh scratch buffer starts empty"
        );

        app.ensure_note_named_before_paste();

        assert!(
            app.active.buffer.path().is_none(),
            "no first line to derive a name from"
        );
        assert!(
            app.active.buffer.is_unnamed_fresh(),
            "promoted regardless — matches typing-then-pausing"
        );
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
        assert_eq!(
            app.config.cjk_priority,
            Some(want.clone()),
            "mirrored in-memory"
        );
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
        assert_eq!(
            Config::load(cfg_path.clone()).spellcheck,
            None,
            "disk still absent"
        );

        app.reload_config();

        assert!(
            !crate::spell::spellcheck_on(),
            "an absent disk key must leave the global AS-IS, not force it back ON"
        );

        // The other direction too: ON stays ON across the same reload.
        crate::spell::set_spellcheck_on(true);
        app.reload_config();
        assert!(
            crate::spell::spellcheck_on(),
            "and leaves an ON global alone too"
        );
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
        assert_eq!(
            app.config.spellcheck,
            Some(false),
            "persist mirrors self.config"
        );

        crate::spell::set_spellcheck_on(true); // simulate reload starting from a stale global
        app.reload_config();
        assert!(
            !crate::spell::spellcheck_on(),
            "the persisted OFF value re-applies on reload"
        );
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
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/w/proj")
            .with_dir("/cfg"),
    );
    crate::fs::with_fs(fake, || {
        let mut config = Config::empty();
        config.path = PathBuf::from("/cfg/config.toml");
        let mut app = App::new(None, PathBuf::from("/w/proj"), None, None, config);
        // Precondition: a made-up word squiggles.
        assert!(!app.spell.as_ref().unwrap().check("wrold"));

        app.add_to_dictionary("wrold");
        // Live: the checker now accepts it.
        assert!(
            app.spell.as_ref().unwrap().check("wrold"),
            "silenced in the live checker"
        );
        // Persisted: the file beside config.toml holds exactly one line.
        let dict = PathBuf::from("/cfg/dictionary.txt");
        let text = crate::fs::active()
            .read_to_string(&dict)
            .expect("the file was written");
        assert_eq!(crate::spell::parse_dictionary(&text), vec!["wrold"]);

        // Re-adding it is a no-op on disk (no duplicate line) and still silent.
        app.add_to_dictionary("wrold");
        let text2 = crate::fs::active().read_to_string(&dict).unwrap();
        assert_eq!(
            crate::spell::parse_dictionary(&text2),
            vec!["wrold"],
            "no duplicate line"
        );
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
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/w/proj")
            .with_dir("/cfg"),
    );
    crate::fs::with_fs(fake, || {
        // A prior session already wrote the word list (hand-edited shape: a
        // header comment + one word per line).
        crate::fs::write_atomic(Path::new("/cfg/dictionary.txt"), b"# my words\nwrold\n").unwrap();
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
        let mut app = App::new(
            None,
            PathBuf::from("/w/proj-a"),
            None,
            None,
            Config::empty(),
        );
        // Fresh launch: no recents yet (missing store).
        assert!(app.project_location.recent_projects.is_empty());

        // Switching to two projects pushes each to the FRONT, newest-first.
        app.switch_project(PathBuf::from("/w/proj-a"));
        app.switch_project(PathBuf::from("/w/proj-b"));
        assert_eq!(
            app.project_location.recent_projects,
            vec![PathBuf::from("/w/proj-b"), PathBuf::from("/w/proj-a")],
        );
        assert_eq!(
            app.project_location.root,
            PathBuf::from("/w/proj-b"),
            "root followed the switch"
        );

        // Re-switching to proj-a moves it to the front (dedup, never a dupe).
        app.switch_project(PathBuf::from("/w/proj-a"));
        assert_eq!(
            app.project_location.recent_projects,
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
        assert!(
            app.project_location.recent_files.is_empty(),
            "fresh launch: empty MRU"
        );

        // Opening three files pushes each to the FRONT (most-recent first). Both
        // load_path branches route here; a fresh disk read is the None branch.
        app.load_path(PathBuf::from("/w/proj/a.md"));
        app.load_path(PathBuf::from("/w/proj/b.md"));
        app.load_path(PathBuf::from("/w/proj/c.md"));
        assert_eq!(
            app.project_location.recent_files,
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
            app.project_location.recent_files,
            vec![
                PathBuf::from("/w/proj/a.md"),
                PathBuf::from("/w/proj/c.md"),
                PathBuf::from("/w/proj/b.md"),
            ],
        );

        // Re-selecting the ALREADY-ACTIVE file is a no-op (load_path's early
        // return), so the MRU is untouched — no re-order, no dupe.
        app.load_path(PathBuf::from("/w/proj/a.md"));
        assert_eq!(
            app.project_location.recent_files.len(),
            3,
            "no-op reopen never re-orders / dupes"
        );
        assert_eq!(
            app.project_location.recent_files[0],
            PathBuf::from("/w/proj/a.md")
        );

        // PERSISTED: a second launch reads the MRU back through the store.
        assert_eq!(
            crate::recent_files::load(),
            app.project_location.recent_files
        );
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
        let app = App::new(
            None,
            PathBuf::from("/w/proj-a"),
            None,
            None,
            Config::empty(),
        );
        assert_eq!(
            app.project_location.recent_projects,
            vec![PathBuf::from("/w/proj-a"), PathBuf::from("/w/proj-b")],
        );
    });
}

// ── ITEM 180 — `ProjectLocation`'s ONE DERIVATION: `resync_project_location` ──
//
// Before this round, `set_root` re-derived `project` + `file_index` but not
// `workspace_root`, and `reload_config` re-derived `workspace_root` but not
// the other two — two half-owners of one derived value. Since
// `resolve_workspace` falls back to `root.parent()` whenever nothing
// overrides it, a Switch-project into a tree whose parent differs from the
// old one left `workspace_root` — and so the Project picker (`C-x p`), which
// browses it directly (`app/apply.rs`'s `browse_to` closure) — pointed at the
// OLD workspace until something unrelated happened to call `reload_config`.
//
// These tests read the SAME oracle the live picker does:
// `crate::overlay::browse_level(OverlayKind::Project, ..)`'s resulting rows —
// a state oracle, never a pixel (docs/platform.md's sidecar tripwire: a
// picker's LISTING is state, not appearance). Calling `App::switch_project` /
// `set_root` directly is the PUREST reachable seam for the derivation itself
// (CLAUDE.md's unit > sidecar > capture ladder).
//
// ITEM 183 CORRECTION: item 180 recorded that the whole live-`App` transition
// class was structurally unreachable from any headless entry point, because
// `App::apply` demanded an `&ActiveEventLoop` no headless caller can produce.
// That was true then and is no longer. The last test in this block drives the
// same switch END TO END from real chords through the real live `App`; the
// exact boundary that remains is mapped in `docs/harness-reach.md`.

/// The names the Project picker (`C-x p`) would list for `app`'s CURRENT
/// `workspace_root`, in the exact shape `app/apply.rs`'s live `browse_to`
/// closure builds them (`self.root.clone()`, `self.workspace_root.clone()`,
/// the persisted recents) — minus the leading "." self-entry every level
/// carries, which is never one of the project siblings this bug is about.
fn project_picker_rows(app: &App) -> Vec<String> {
    let ov = crate::overlay::browse_level(
        crate::overlay::OverlayKind::Project,
        None,
        &app.project_location.root,
        app.project_location.workspace_root.as_deref(),
        &[],
    )
    .expect("workspace_root is always Some on a live App, so the picker always builds");
    ov.rows
        .into_iter()
        .map(|r| r.accept)
        .filter(|name| name != ".")
        .collect()
}

/// THE REPORTED REPRO: switching into a tree whose PARENT DIFFERS from the
/// old one must re-point the picker at the NEW workspace immediately — no
/// dependency on a later config reload. `switch_project` is the ONE owner
/// every real Switch-project door (an accepted Project-picker row, the
/// Recent Projects picker) routes through, so driving it directly here is the
/// same transition either door takes.
#[test]
fn switch_project_into_a_different_parent_repoints_the_picker_at_the_new_workspace() {
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/old-ws/proj-a")
            .with_dir("/old-ws/sibling")
            .with_dir("/new-ws/proj-b")
            .with_dir("/new-ws/other"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(
            None,
            PathBuf::from("/old-ws/proj-a"),
            None,
            None,
            Config::empty(),
        );
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/old-ws"))
        );
        assert_eq!(project_picker_rows(&app), vec!["proj-a", "sibling"]);

        app.switch_project(PathBuf::from("/new-ws/proj-b"));

        assert_eq!(
            app.project_location.project.root,
            PathBuf::from("/new-ws/proj-b"),
            "project agrees with root"
        );
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/new-ws")),
            "workspace_root must follow the switch, never stay pinned to the OLD parent"
        );
        assert_eq!(
            project_picker_rows(&app),
            vec!["other", "proj-b"],
            "the picker must list the NEW workspace's siblings, not the old \
             workspace's leftover ['proj-a', 'sibling'] — that would be the \
             stale-sibling bug back"
        );
    });
}

/// Axis value: the SAME-parent switch. Two projects share a parent, so the
/// derivation must be a no-op on `workspace_root` — proving the fix doesn't
/// merely always re-run `root.parent()` blindly without landing on the right
/// value.
#[test]
fn switch_project_within_the_same_parent_leaves_the_picker_on_the_shared_workspace() {
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/ws/proj-a")
            .with_dir("/ws/proj-b"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(
            None,
            PathBuf::from("/ws/proj-a"),
            None,
            None,
            Config::empty(),
        );
        app.switch_project(PathBuf::from("/ws/proj-b"));
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/ws"))
        );
        assert_eq!(project_picker_rows(&app), vec!["proj-a", "proj-b"]);
    });
}

/// Axis value: the filesystem-root edge. Switching INTO a root with no
/// parent (`/`) must fall back to the root itself
/// ([`crate::resolve_workspace`]'s own no-parent arm), not panic or leave a
/// stale value from before the switch.
#[test]
fn switch_project_into_the_filesystem_root_falls_back_to_the_root_itself() {
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/somewhere/proj"));
    crate::fs::with_fs(fake, || {
        let mut app = App::new(
            None,
            PathBuf::from("/somewhere/proj"),
            None,
            None,
            Config::empty(),
        );
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/somewhere"))
        );

        app.switch_project(PathBuf::from("/"));

        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/")),
            "no parent to fall back to — the root itself is the workspace"
        );
    });
}

/// Axis value: a workspace EXPLICITLY named in config must keep winning over
/// the `root.parent()` fallback across a switch into a different-parent tree
/// — the derivation folding two call sites into one must never silently drop
/// the CLI/config precedence `resolve_workspace` itself already encodes.
#[test]
fn switch_project_never_overrides_an_explicitly_configured_workspace() {
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/a/proj")
            .with_dir("/x/y/proj2"),
    );
    crate::fs::with_fs(fake, || {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/explicit-ws"));
        let mut app = App::new(None, PathBuf::from("/a/proj"), None, None, config);
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/explicit-ws"))
        );

        app.switch_project(PathBuf::from("/x/y/proj2"));

        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/explicit-ws")),
            "an explicit config workspace must survive a switch into a \
             different-parent tree, never fall back to root.parent()"
        );
    });
}

/// Axis value: A → B → A. A stale value from the FIRST switch must not
/// survive the ROUND TRIP back — the exact shape a half-derivation that only
/// updates "sometimes" would get wrong on a second switch.
#[test]
fn switch_project_a_to_b_and_back_never_leaves_a_stale_workspace() {
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/aa/proj")
            .with_dir("/bb/cc/proj2"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(None, PathBuf::from("/aa/proj"), None, None, Config::empty());
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/aa"))
        );

        app.switch_project(PathBuf::from("/bb/cc/proj2"));
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/bb/cc"))
        );

        app.switch_project(PathBuf::from("/aa/proj"));
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/aa")),
            "the round trip back to A must not leave B's workspace behind"
        );
    });
}

/// ITEM 183 — THE LIVE/HEADLESS PARITY LAW for the project location.
///
/// The harness carries its OWN derivation of "what does this root imply": a
/// capture reports the location in its sidecar, once from the launch root and
/// again after a Project-picker accept. That second site re-derived
/// `name`/`branch`/`dirty` from the accepted root while carrying the LAUNCH
/// root's `workspace` forward — item 180's defect, alive in the harness's copy
/// of the rule long after item 180 fixed the App's. Reproduced before the fix
/// on a real capture: `--keys "s-S-p Backspace Enter Enter Enter"` into
/// `/new-ws/proj-b` reported `root: /new-ws/proj-b` beside
/// `workspace: /old-ws`. The oracle lied about the exact transition it was
/// asked to witness.
///
/// One builder now serves both capture sites (`run::project_info`), and this
/// law pins it to the live `App`'s own derivation across item 180's whole axis
/// — different parent, same parent, the filesystem-root edge, an explicitly
/// configured workspace that must beat the `root.parent()` fallback, and a
/// switch away and back. Neither side is allowed to be "close": the sidecar
/// must report exactly what a live editor at that root would hold.
#[test]
fn the_capture_sidecars_project_location_equals_the_live_apps() {
    // The axis is item 180's own, value for value.
    let axis: &[(&str, &[&str], &str, &str)] = &[
        (
            "different parent",
            &["/old-ws/proj-a", "/new-ws/proj-b"],
            "/old-ws/proj-a",
            "/new-ws/proj-b",
        ),
        (
            "same parent",
            &["/ws/proj-a", "/ws/proj-b"],
            "/ws/proj-a",
            "/ws/proj-b",
        ),
        (
            "filesystem-root edge",
            &["/somewhere/proj"],
            "/somewhere/proj",
            "/",
        ),
        (
            "A -> B -> A round trip returns to A's own workspace",
            &["/aa/proj", "/bb/cc/proj2"],
            "/bb/cc/proj2",
            "/aa/proj",
        ),
    ];
    let mut checked = 0usize;
    for (label, dirs, launch, switch_to) in axis {
        assert_live_and_capture_locations_agree(label, dirs, launch, switch_to, None, None);
        checked += 1;
    }
    // The two precedence values, where the fallback must NOT win.
    let ws_dirs: &[&str] = &["/x/y/proj1", "/x/y/proj2"];
    assert_live_and_capture_locations_agree(
        "configured workspace beats root.parent()",
        ws_dirs,
        "/x/y/proj1",
        "/x/y/proj2",
        None,
        Some("/x"),
    );
    assert_live_and_capture_locations_agree(
        "cli workspace beats config",
        ws_dirs,
        "/x/y/proj1",
        "/x/y/proj2",
        Some("/x/y"),
        Some("/x"),
    );
    checked += 2;
    assert_eq!(checked, 6, "every axis value must be graded, not skipped");
}

/// One axis value: build the tree, launch a live `App` at `launch`, switch it to
/// `switch_to`, and demand that the capture's own builder reports exactly what
/// that live `App` now holds.
fn assert_live_and_capture_locations_agree(
    label: &str,
    dirs: &[&str],
    launch: &str,
    switch_to: &str,
    cli_ws: Option<&str>,
    cfg_ws: Option<&str>,
) {
    let mut fake = crate::fs::InMemoryFs::new();
    for d in dirs {
        fake = fake.with_dir(d);
    }
    crate::fs::with_fs(Arc::new(fake), || {
        // `Config` is not `Clone`; both sides get their own identical one.
        let workspace_of = || Config {
            workspace: cfg_ws.map(PathBuf::from),
            ..Config::empty()
        };
        let config = workspace_of();
        let cli = cli_ws.map(PathBuf::from);
        let mut app = App::new(
            None,
            PathBuf::from(launch),
            cli.clone(),
            None,
            workspace_of(),
        );
        app.switch_project(PathBuf::from(switch_to));

        // The capture's builder, called exactly as `capture_screenshot` calls it
        // on an accepted Project row: the already-folded flag-over-config
        // workspace, and the accepted root.
        let folded = cli.or_else(|| config.workspace.clone());
        let info =
            crate::run::project_info(std::path::Path::new(switch_to), &folded, None, &config);

        assert_eq!(
            info.root, app.project_location.root,
            "{label}: sidecar root == live root"
        );
        assert_eq!(
            info.name, app.project_location.project.name,
            "{label}: sidecar project name == live project name"
        );
        assert_eq!(
            info.workspace, app.project_location.workspace_root,
            "{label}: the sidecar's workspace must be the live workspace — a \
             capture that reports a workspace the running editor does not have \
             is item 180's bug living on in the harness"
        );
    });
}

/// ITEM 183 — THE SAME BUG, DRIVEN FROM REAL KEYS THROUGH THE LIVE `App`.
///
/// Every test above calls `App::switch_project` directly, because until this
/// round nothing else could: `App::apply` — the ONE seam a keypress, a menu
/// item, a palette command and an overlay click all funnel through — demanded
/// an `&ActiveEventLoop`, which exists only inside a running winit loop and
/// cannot be constructed. So item 180's Verify clause ("drive Switch-project
/// through the real keymap and assert the picker's contents") named a capture
/// that structurally could not exist. Narrowing that borrow to the ONE
/// capability `apply` actually used (`app::Exit` — `event_loop.exit()`, nothing
/// else) is what made this test possible.
///
/// Nothing here stands in for the live path; it IS the live path minus the
/// window. The chords take `App::press_chord_headless` →
/// `dispatch_pressed_key` (the same owner `WindowEvent::KeyboardInput` and the
/// `--live-script` probe call) → keymap resolve → `App::apply` →
/// `Effect::OverlayAccept(Project, ..)` → `switch_project` → `set_root` →
/// `resync_project_location`. The picker is a REAL summoned overlay built by
/// the real `browse_to` closure, navigated by real Backspace/Down/Enter.
///
/// The spec branches on the running convention rather than hardcoding one, so
/// `native-gate.sh`'s mac AND linux passes each drive their own real chord for
/// "Switch project…" — the axis a hardcoded `s-S-p` would have skipped.
#[test]
fn switch_project_driven_by_real_chords_through_apply_repoints_the_workspace() {
    let fake = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/old-ws/proj-a")
            .with_dir("/old-ws/sibling")
            .with_dir("/new-ws/other")
            .with_dir("/new-ws/proj-b"),
    );
    crate::fs::with_fs(fake, || {
        let mut app = App::new(
            None,
            PathBuf::from("/old-ws/proj-a"),
            None,
            None,
            Config::empty(),
        );
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/old-ws"))
        );
        assert_eq!(project_picker_rows(&app), vec!["proj-a", "sibling"]);

        // "Switch project…" — the real binding of the convention this pass runs.
        let open_project = match crate::convention::Convention::current() {
            crate::convention::Convention::Mac => "s-S-p",
            crate::convention::Convention::Linux => "C-S-p",
        };
        app.press_spec_headless(open_project)
            .expect("the switch-project chord parses");
        assert_eq!(
            app.workspace_state
                .overlay()
                .map(|o| o.kind)
                .expect("the chord summoned a real Project picker"),
            crate::overlay::OverlayKind::Project,
        );

        // Backspace ascends above the old workspace to `/`; Enter descends into
        // `new-ws`; Down moves off `other` onto `proj-b`; Enter descends into
        // it; the last Enter accepts the drilled-in directory as the new root.
        app.press_spec_headless("Backspace Enter Down Enter Enter")
            .expect("the navigation chords parse");

        assert!(
            !app.workspace_state.overlay_open(),
            "accepting the row closes the picker, exactly as live"
        );
        assert_eq!(app.project_location.root, PathBuf::from("/new-ws/proj-b"));
        assert_eq!(
            app.project_location.workspace_root,
            Some(PathBuf::from("/new-ws")),
            "the workspace must follow a switch driven by real keys, not only \
             one driven by a direct call to switch_project"
        );
        assert_eq!(
            project_picker_rows(&app),
            vec!["other", "proj-b"],
            "the picker must now list the NEW workspace's siblings — the \
             stale ['proj-a', 'sibling'] here is item 180's reported bug"
        );
    });
}
