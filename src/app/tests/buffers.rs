use super::*;

// ── MULTI-BUFFER REGISTRY (App-level: open/switch preserves everything) ──

#[test]
fn load_path_switches_to_already_open_buffer_preserving_edits_and_cursor() {
    // THE v1 OBSERVABLE WIN: re-opening a file already open in this session
    // restores its LIVE buffer (unsaved edits, cursor) instead of re-reading
    // disk. Proven by mutating A's on-disk bytes BEHIND awl's back while B is
    // active, then asserting the restored A shows the in-memory edit, not the
    // disk write.
    use crate::fs::{FileSystem, InMemoryFs};
    let a = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new()
        .with_file(&a, "alpha\n")
        .with_file(&b, "beta\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    app.active.buffer.set_text("ALPHA EDITED\n");
    app.active.buffer.set_cursor(3);
    assert_eq!(app.open_buffer_count(), 1, "only A is open so far");

    app.load_path(b.clone());
    assert_eq!(
        app.active.buffer.text(),
        "beta\n",
        "B loads fresh from disk (first open)"
    );
    assert_eq!(
        app.open_buffer_count(),
        2,
        "A is now backgrounded, not closed"
    );
    app.active.buffer.set_text("BETA EDITED\n");

    mem.write(&a, b"ALPHA CHANGED ON DISK\n").unwrap();

    app.load_path(a.clone());
    assert_eq!(
        app.active.buffer.text(),
        "ALPHA EDITED\n",
        "the LIVE unsaved edit survived the round trip, not a re-read from disk"
    );
    assert_eq!(
        app.active.buffer.cursor_char(),
        3,
        "the cursor position survived too"
    );
    assert!(
        app.active.buffer.is_dirty(),
        "the unsaved edit is still unsaved"
    );
    assert_eq!(app.open_buffer_count(), 2, "A active again, B backgrounded");

    // And B's OWN edit is preserved too (not silently dropped when we left it).
    app.load_path(b.clone());
    assert_eq!(
        app.active.buffer.text(),
        "BETA EDITED\n",
        "B's edit also survived"
    );
}

#[test]
fn buffer_switch_clears_the_list_continuation_provenance_flag_item_78() {
    // Item 78: the short-lived "awl just generated this empty list continuation"
    // flag must NOT survive a park/activate round trip (`load_path`'s switch) —
    // even though the buffer's cursor/text/undo history all correctly DO survive
    // it (the test above). Simulate the flag as smart_newline would leave it
    // (`mark_list_continuation_generated`, pub(crate) — same crate, no need to
    // drive a real Enter here) rather than the flag's OWN clear masking a park/
    // activate bug behind an edit that would have cleared it anyway.
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.md");
    let b = PathBuf::from("/proj/b.md");
    let mem = InMemoryFs::new()
        .with_file(&a, "- \n")
        .with_file(&b, "beta\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    app.active.buffer.mark_list_continuation_generated();

    // Switch away (parks A) and back (activates A): both halves of the swap.
    app.load_path(b.clone());
    app.load_path(a.clone());

    assert!(
        !app.active.buffer.take_list_continuation_generated(),
        "a buffer switch clears the provenance flag — it must not survive backgrounding"
    );
}

// ── PROSE/CODE PAGE-WIDTH SPLIT (App-level buffer-switch resync) ────────

#[test]
fn load_path_switch_reapplies_default_measure_per_kind() {
    // WIRING (1): a buffer SWITCH re-applies the right measure through the
    // existing `set_measure` seam (`App::sync_page_measure`, called from
    // `load_path`). A.md (prose) -> B.rs (code) -> back to A.md, with NO
    // config override, must land on each class's own BUILT-IN default.
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.md");
    let b = PathBuf::from("/proj/b.rs");
    let mem = InMemoryFs::new()
        .with_file(&a, "# hello\n")
        .with_file(&b, "fn main() {}\n");
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem));
    // LOCK ORDER: fs seam first, page lock LAST (see page::test_lock()'s doc)
    // — the reverse order deadlocks against every fs-holding test whose
    // load_path transitively writes the measure.
    let _g = crate::testlock::serial();
    let measure0 = crate::page::measure();
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());

    // Deliberately wrong, so the switches below can't coincidentally "already"
    // hold the right value.
    crate::page::set_measure(12345);
    app.load_path(b.clone());
    assert_eq!(
        crate::page::measure(),
        crate::page::DEFAULT_MEASURE_CODE,
        "switching to B.rs (code) applies the code default"
    );
    app.load_path(a.clone());
    assert_eq!(
        crate::page::measure(),
        crate::page::DEFAULT_MEASURE,
        "switching back to A.md (prose) applies the prose default"
    );

    crate::page::set_measure(measure0);
}

#[test]
fn load_path_switch_reapplies_custom_measure_overrides() {
    // The SAME A.md/B.rs round trip, but with configured overrides for BOTH
    // classes — the switch must read `Config::measure_for`, not just the
    // built-in defaults.
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.md");
    let b = PathBuf::from("/proj/b.rs");
    let mem = InMemoryFs::new()
        .with_file(&a, "hello\n")
        .with_file(&b, "fn main() {}\n");
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial(); // fs first, page LAST (see page::test_lock())
    let measure0 = crate::page::measure();
    let cfg = Config {
        page_width_prose: Some(55),
        page_width_code: Some(120),
        ..Config::empty()
    };
    let mut app = app_on(Some(a.clone()), "/proj", cfg);

    crate::page::set_measure(1);
    app.load_path(b.clone());
    assert_eq!(
        crate::page::measure(),
        120,
        "B.rs picks up the configured code override"
    );
    app.load_path(a.clone());
    assert_eq!(
        crate::page::measure(),
        55,
        "back to A.md picks up the configured prose override"
    );

    crate::page::set_measure(measure0);
}

#[test]
fn new_note_always_reapplies_the_prose_measure() {
    // A fresh quick note is always markdown (PROSE), regardless of what kind
    // of buffer was active before it — `new_document` calls the same
    // `sync_page_measure` resync `load_path` does.
    use crate::fs::InMemoryFs;
    let b = PathBuf::from("/proj/b.rs");
    let mem = InMemoryFs::new().with_file(&b, "fn main() {}\n");
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial(); // fs first, page LAST (see page::test_lock())
    let measure0 = crate::page::measure();
    let mut app = app_on(Some(b.clone()), "/proj", Config::empty());

    crate::page::set_measure(crate::page::DEFAULT_MEASURE_CODE);
    app.new_document();
    assert_eq!(
        crate::page::measure(),
        crate::page::DEFAULT_MEASURE,
        "a new note is prose, so it gets the prose default even leaving a code buffer"
    );

    crate::page::set_measure(measure0);
}

#[test]
fn persist_page_width_writes_the_key_matching_the_active_buffer_kind() {
    // The STICKY WRITE half (drag-resize / C-x { / C-x }): `persist_page_width`
    // must target `page_width_prose` while a prose buffer is active and
    // `page_width_code` while a code buffer is active — never the other key.
    use crate::fs::InMemoryFs;
    let cfg_path = PathBuf::from("/home/.config/awl/config.toml");
    let a = PathBuf::from("/proj/a.md");
    let b = PathBuf::from("/proj/b.rs");
    let mem = InMemoryFs::new()
        .with_file(&a, "hello\n")
        .with_file(&b, "fn main() {}\n");
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial(); // fs first, page LAST (see page::test_lock())
    let measure0 = crate::page::measure();
    let cfg = Config {
        path: cfg_path.clone(),
        ..Config::empty()
    };
    let mut app = app_on(Some(a.clone()), "/proj", cfg);

    crate::page::set_measure(55);
    app.persist_page_width();
    let reloaded = Config::load(cfg_path.clone());
    assert_eq!(
        reloaded.page_width_prose,
        Some(55),
        "a PROSE buffer persists to page_width_prose"
    );
    assert_eq!(reloaded.page_width_code, None, "the code key is untouched");

    app.load_path(b.clone());
    crate::page::set_measure(130);
    app.persist_page_width();
    let reloaded2 = Config::load(cfg_path.clone());
    assert_eq!(
        reloaded2.page_width_code,
        Some(130),
        "a CODE buffer persists to page_width_code"
    );
    assert_eq!(
        reloaded2.page_width_prose,
        Some(55),
        "the prose key from before survives untouched"
    );

    crate::page::set_measure(measure0);
}

#[test]
fn persist_page_reset_clears_the_key_matching_the_active_buffer_kind() {
    // The RESET half: `persist_page_reset` must clear ONLY the override
    // matching the active buffer's kind, leaving the other class's override
    // (and every other pref) untouched.
    use crate::fs::InMemoryFs;
    let cfg_path = PathBuf::from("/home/.config/awl/config.toml");
    let a = PathBuf::from("/proj/a.md");
    let b = PathBuf::from("/proj/b.rs");
    let mem = InMemoryFs::new()
        .with_file(&a, "hello\n")
        .with_file(&b, "fn main() {}\n");
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial(); // fs first, page LAST (see page::test_lock())
    let measure0 = crate::page::measure();
    Config::write_pref(&cfg_path, "page_width_prose", "55").unwrap();
    Config::write_pref(&cfg_path, "page_width_code", "130").unwrap();
    let cfg = Config::load(cfg_path.clone());
    let mut app = app_on(Some(b.clone()), "/proj", cfg); // start on the CODE file

    app.persist_page_reset();
    let reloaded = Config::load(cfg_path.clone());
    assert_eq!(
        reloaded.page_width_code, None,
        "the code override is cleared"
    );
    assert_eq!(
        reloaded.page_width_prose,
        Some(55),
        "the prose override survives untouched"
    );

    app.load_path(a.clone());
    app.persist_page_reset();
    let reloaded2 = Config::load(cfg_path.clone());
    assert_eq!(
        reloaded2.page_width_prose, None,
        "the prose override is now also cleared"
    );

    crate::page::set_measure(measure0);
}

#[test]
fn setting_value_commit_clamps_persists_and_applies_measure_and_zoom() {
    // SETTINGS v2 inline VALUE edit (App half): parse + clamp the typed value,
    // apply it LIVE (page::measure / zoom), and persist the NAMED key.
    use crate::fs::InMemoryFs;
    let cfg_path = PathBuf::from("/home/.config/awl/config.toml");
    let a = PathBuf::from("/proj/a.md"); // a PROSE (.md) buffer
    let mem = InMemoryFs::new().with_file(&a, "hello\n");
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial(); // fs first, page LAST (see page::test_lock())
    let measure0 = crate::page::measure();
    let cfg = Config {
        path: cfg_path.clone(),
        ..Config::empty()
    };
    let mut app = app_on(Some(a.clone()), "/proj", cfg);

    // In-range prose width: applied LIVE (the active buffer is prose) + persisted.
    app.setting_value_commit("page_width_prose", "45");
    assert_eq!(
        crate::page::measure(),
        45,
        "a prose-width edit re-wraps live"
    );
    assert_eq!(Config::load(cfg_path.clone()).page_width_prose, Some(45));

    // Out of range: CLAMPED to PAGE_WIDTH_MAX, both live + on disk.
    app.setting_value_commit("page_width_prose", "5000");
    assert_eq!(crate::page::measure(), crate::settings::PAGE_WIDTH_MAX);
    assert_eq!(
        Config::load(cfg_path.clone()).page_width_prose,
        Some(crate::settings::PAGE_WIDTH_MAX)
    );

    // Unparseable: a calm no-op (measure + config unchanged).
    app.setting_value_commit("page_width_prose", "oops");
    assert_eq!(crate::page::measure(), crate::settings::PAGE_WIDTH_MAX);

    // Editing the CODE width while a PROSE buffer is active persists to its own key
    // but does NOT change the visible measure (sync_page_measure reads the active
    // class), so the prose/code split never bleeds.
    app.setting_value_commit("page_width_code", "88");
    assert_eq!(
        crate::page::measure(),
        crate::settings::PAGE_WIDTH_MAX,
        "the code-width edit leaves the prose measure alone"
    );
    assert_eq!(Config::load(cfg_path.clone()).page_width_code, Some(88));

    // ZOOM: the percent readout form parses + clamps through the shared set_zoom
    // owner + persists.
    app.setting_value_commit("zoom", "150%");
    assert!((app.zoom - 1.5).abs() < 1e-4, "150% -> factor 1.5");
    assert_eq!(Config::load(cfg_path.clone()).zoom, Some(1.5));

    crate::page::set_measure(measure0);
}

#[test]
fn load_path_reopening_the_active_file_is_a_noop() {
    // Re-"opening" the file that is already active must not disturb anything
    // (no park/restore round trip, no fresh disk read either).
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.txt");
    let mem = InMemoryFs::new().with_file(&a, "alpha\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    app.active.buffer.set_text("EDITED IN PLACE\n");
    app.active.buffer.set_cursor(2);
    app.load_path(a.clone());
    assert_eq!(app.active.buffer.text(), "EDITED IN PLACE\n");
    assert_eq!(app.active.buffer.cursor_char(), 2);
    assert_eq!(app.open_buffer_count(), 1, "no phantom second entry");
}

#[test]
fn load_path_recognizes_the_same_file_under_a_differently_spelled_path() {
    // REGRESSION (code review): the registry's identity must be blind to
    // which textual spelling of the same file produced the path — e.g. a
    // CLI file argument typed with no directory component (`cd project &&
    // awl a.txt`, staying relative) vs. that same file's later ROOT-JOINED
    // spelling (`index::resolve`, always absolute — every Goto candidate).
    // Reproduced here with a `.` path component (lexically different, same
    // file) so the fix is proven at the live-App layer, not just headless.
    use crate::fs::InMemoryFs;
    let messy = PathBuf::from("/proj/./a.txt");
    let clean = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new().with_file(&b, "beta\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(messy.clone()), "/proj", Config::empty());
    app.active.buffer.set_text("ALPHA EDITED\n");
    assert_eq!(app.open_buffer_count(), 1);

    app.load_path(b.clone());
    assert_eq!(
        app.open_buffer_count(),
        2,
        "the messy-spelled A is backgrounded"
    );

    app.load_path(clean.clone());
    assert_eq!(
        app.active.buffer.text(),
        "ALPHA EDITED\n",
        "the CLEAN path found A's live entry (parked under the MESSY spelling) instead of \
         opening a fresh, orphaned copy"
    );
    assert_eq!(
        app.open_buffer_count(),
        2,
        "no orphaned duplicate entry left behind for the messy spelling"
    );
}

#[test]
fn load_path_opens_a_relative_launch_path_then_finds_it_again_via_absolute_path() {
    // REGRESSION (code review, scenario a — the report's EXACT live shape):
    // `cd project && awl a.txt` leaves the launch file argument RELATIVE;
    // reopening the SAME file via its absolute spelling (what a Go-to-file
    // picker candidate always is — `index::resolve` root-joins) must find
    // the SAME live buffer, not silently re-read disk and orphan the
    // relative spelling's dirty entry forever. Needs a REAL chdir (not
    // InMemoryFs, which has no cwd concept) against a real temp dir — hold
    // both the fs TEST_LOCK (real-disk reads race a sibling's InMemoryFs
    // swap) and the CWD_LOCK (chdir is process-global too).
    let _fs = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl-relabs-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
    let _cwd = crate::fs::CwdGuard::enter(&dir);

    // This test's `App::new` runs against the REAL native FS (it can't use
    // InMemoryFs, per the chdir note above) — so it must explicitly kill
    // SESSION RESTORE, or `apply_session_restore` reads the developer's
    // ACTUAL `~/.local/share/awl/session.toml` and parks every real
    // buffer it names into the registry, inflating `open_buffer_count()`
    // by however many files happen to be open in a live awl session on
    // this machine right now (an environment-coupled failure, not a
    // random flake — see the investigation note in git history).
    let cfg = Config {
        session_restore: Some(false),
        ..Config::empty()
    };
    // The launch argument stays exactly as typed: relative, no directory.
    let mut app = App::new(Some(PathBuf::from("a.txt")), dir.clone(), None, None, cfg);
    app.active.buffer.set_text("ALPHA EDITED\n");
    app.active.buffer.set_cursor(3);
    assert_eq!(
        app.open_buffer_count(),
        1,
        "only the relative-spelled A is open so far"
    );

    // Reopen via the ABSOLUTE spelling.
    app.load_path(dir.join("a.txt"));
    assert_eq!(
        app.active.buffer.text(),
        "ALPHA EDITED\n",
        "the live edit survived — the absolute spelling found the SAME buffer, not a fresh \
         disk read"
    );
    assert_eq!(
        app.active.buffer.cursor_char(),
        3,
        "the cursor position survived too"
    );
    assert_eq!(
        app.open_buffer_count(),
        1,
        "one entry, not two — the relative and absolute spellings key identically"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn switching_buffers_isolates_the_view_text_cache() {
    // THE CACHE-KEY-DISCIPLINE bug class (CLAUDE.md): every swapped-in buffer
    // restarts its edit version at 0, so `view_text`'s version-keyed
    // rope-clone cache MUST travel with its own buffer (not collide with
    // another buffer sitting at the same version) across a three-way swap.
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new()
        .with_file(&a, "aaa\n")
        .with_file(&b, "bbb\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    assert_eq!(app.view_text(), "aaa\n");
    app.load_path(b.clone());
    assert_eq!(
        app.view_text(),
        "bbb\n",
        "B's text must not collide with A's stale version-0 cache"
    );
    app.load_path(a.clone());
    assert_eq!(
        app.view_text(),
        "aaa\n",
        "A's OWN cache is restored, not B's"
    );
}

#[test]
fn switching_away_from_a_dirty_file_still_autosaves() {
    // Item 4 of the spec: the existing autosave flush-on-FILE-SWITCH hook
    // (`App::autosave_flush`, the one door) must still fire on a registry
    // switch, exactly as it did on the old single-buffer swap.
    use crate::fs::{FileSystem, InMemoryFs};
    let a = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new()
        .with_file(&a, "aaa\n")
        .with_file(&b, "bbb\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    app.active.buffer.set_text("aaa EDITED\n");
    app.load_path(b.clone());
    assert_eq!(
        mem.read_to_string(&a).unwrap(),
        "aaa EDITED\n",
        "leaving a dirty pathed buffer autosaves it on switch"
    );
}

#[test]
fn new_note_parks_the_previous_buffer_for_a_later_reopen() {
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.txt");
    let mem = InMemoryFs::new().with_file(&a, "aaa\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    app.active.buffer.set_text("aaa EDITED\n");
    assert_eq!(app.open_buffer_count(), 1);
    app.new_document();
    assert_eq!(
        app.open_buffer_count(),
        2,
        "A is parked; the fresh note is active"
    );
    assert_eq!(app.active.buffer.text(), "", "the new note starts blank");
    app.load_path(a.clone());
    assert_eq!(
        app.active.buffer.text(),
        "aaa EDITED\n",
        "A's edit survived being backgrounded by C-x n"
    );
}

#[test]
fn registry_cap_evicts_the_lru_clean_buffer_not_a_dirty_one() {
    // Integration proof that `App` is really wired to
    // `crate::buffers::MAX_OPEN_BUFFERS` (the algorithm itself is exhaustively
    // unit-tested in `buffers.rs`): opening one more CLEAN file than the cap
    // allows evicts the oldest clean background entry, so re-opening THAT one
    // reads fresh from disk (its edits, if any, would be gone — here it has
    // none, so we assert the fresh disk content lands, and via the "clean" law
    // that a DIRTY one earlier in the queue is never touched).
    use crate::fs::InMemoryFs;
    let mut mem = InMemoryFs::new();
    for i in 0..crate::buffers::MAX_OPEN_BUFFERS {
        mem = mem.with_file(format!("/proj/f{i}.txt"), "clean\n");
    }
    mem = mem.with_file("/proj/dirty.txt", "will-be-edited\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(
        Some(PathBuf::from("/proj/dirty.txt")),
        "/proj",
        Config::empty(),
    );
    app.active.buffer.set_text("EDITED, NEVER EVICT ME\n");
    // Open every clean file in turn (backgrounding the dirty one first, then
    // each clean one), pushing the registry to (and one past) the cap.
    for i in 0..crate::buffers::MAX_OPEN_BUFFERS {
        app.load_path(PathBuf::from(format!("/proj/f{i}.txt")));
    }
    // The registry now holds MAX_OPEN_BUFFERS backgrounded entries (dirty.txt
    // + f0..f(N-2)) capped by evicting the LRU CLEAN one (f0) — never dirty.txt.
    assert_eq!(
        app.open_buffer_count(),
        crate::buffers::MAX_OPEN_BUFFERS,
        "capped, not unbounded"
    );
    app.load_path(PathBuf::from("/proj/dirty.txt"));
    assert_eq!(
        app.active.buffer.text(),
        "EDITED, NEVER EVICT ME\n",
        "the dirty buffer survived the whole cap-pressure run"
    );
}

#[test]
fn right_click_word_summons_spell_suggestions() {
    // The right-click path = place the cursor at the clicked word (the GPU
    // hit-test, untestable headlessly), then run the EXISTING OpenSpellSuggest
    // seam at that cursor. This locks the REUSED contract WITHOUT a window: a
    // cursor on a misspelling yields a target with corrections (so the picker
    // summons + builds a Spell overlay), while a correct word yields None — the
    // calm no-op the binding promises. Skipped if the bundled dictionary is absent.
    let Ok(sc) = crate::spell::SpellChecker::new(crate::spell::DictVariant::EnUs) else {
        return;
    };
    let mut buffer = Buffer::from_str("Please recieve this.\n");
    // Simulate the click landing inside the misspelling "recieve".
    let idx = buffer.line_col_to_char(0, 9);
    buffer.set_cursor(idx);
    let (line, col) = buffer.cursor_line_col();
    let t = sc
        .suggest_at(&buffer.text(), line, col, buffer.syntax_lang())
        .expect("a misspelled word under the right-click yields a target");
    assert!(t.suggestions.iter().any(|w| w == "receive"));
    // What `apply(OpenSpellSuggest)` builds from that target: a Spell picker.
    let ov = crate::overlay::OverlayState::new_spell(
        t.suggestions.clone(),
        (
            t.misspelling.line,
            t.misspelling.start_col,
            t.misspelling.end_col,
        ),
        t.word.clone(),
    );
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Spell);
    // A right-click on a CORRECTLY-spelled word ("Please") is a calm no-op.
    let ok_idx = buffer.line_col_to_char(0, 2);
    buffer.set_cursor(ok_idx);
    let (l, c) = buffer.cursor_line_col();
    assert!(
        sc.suggest_at(&buffer.text(), l, c, buffer.syntax_lang())
            .is_none(),
        "correct word: no summon"
    );
}

// ── ITEM 56: THE OWNED SLOT — every App-level per-buffer field travels ──
// with its buffer because the WHOLE `Entry<BufferExtra>` moves atomically on
// park/activate, never a field enumerated by hand (`files/active.rs`'s
// `park_active_buffer`/`activate_from_registry`). These tests witness that
// law directly, at the seam most likely to regress it: a NEW buffer-scoped
// field added to `App` (instead of `BufferExtra`) rather than a forgotten
// entry in a hand-written snapshot/restore pair.

#[test]
fn buffer_scoped_fields_round_trip_a_to_b_to_a_then_a_to_b_to_c_to_a() {
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.md");
    let b = PathBuf::from("/proj/b.md");
    let c = PathBuf::from("/proj/c.md");
    let mem = InMemoryFs::new()
        .with_file(&a, "# a\nhelo world\n")
        .with_file(&b, "# b\n")
        .with_file(&c, "# c\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    // Autosave OFF: this test's whole point is the SWAP mechanics, isolated
    // from the (unrelated) autosave engine's own clobber-guard bookkeeping,
    // which would otherwise treat the hand-set `doc_saved_version`/
    // `disk_mtime` sentinels below as a genuine external change and quietly
    // re-stamp them the moment the switch's own `autosave_flush` runs.
    let cfg = Config {
        autosave: Some(false),
        ..Config::empty()
    };
    let mut app = app_on(Some(a.clone()), "/proj", cfg);

    // Mutate EVERY BufferExtra field on A to a value distinguishable from
    // both `BufferExtra::default()` and whatever B/C will carry.
    app.active.extra.shift_selecting = true;
    app.active.extra.scroll_lines = 11;
    app.run_spellcheck_now(); // populates spell_cache + spell_checked_version
    assert!(
        !app.active.extra.spell_cache.is_empty(),
        "fixture text must actually misspell"
    );
    let a_spell_len = app.active.extra.spell_cache.len();
    let _ = app.view_text(); // populates sync_text_cache
    assert!(app.active.extra.sync_text_cache.is_some());
    app.active.extra.caret_synced_version = 999;
    app.active.extra.doc_saved_version = Some(777);
    app.active.extra.scratch_saved_version = Some(888);
    app.active.extra.disk_mtime = Some(crate::fs::Metadata {
        modified: Some(std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(111)),
        len: Some(222),
    });
    app.active.extra.scratch_mtime = Some(crate::fs::Metadata {
        modified: Some(std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(333)),
        len: Some(444),
    });
    // NOTE: `doc_autosave_at` is deliberately EXCLUDED from the round-trip set
    // below — `autosave_flush` (called on every switch-OUT, at the very top of
    // `load_path`, regardless of whether autosave itself is on) unconditionally
    // clears the OUTGOING buffer's own pending-idle-write marker as part of
    // treating the switch itself as a flush trigger; that is pre-existing,
    // deliberate behavior this round left untouched, not a slot-ownership leak
    // (a switch away from a buffer always retires its own pending autosave).
    app.active.extra.history_preview = Some(("42".to_string(), "old text".to_string()));
    app.active.extra.history_scroll_before = Some(crate::render::ScrollPos::at_row(55));

    app.load_path(b.clone()); // A parks whole-slot; B activates FRESH
    assert_eq!(
        app.active.extra.scroll_lines, 0,
        "B never sees A's sentinel"
    );
    assert!(!app.active.extra.shift_selecting);
    assert!(app.active.extra.spell_cache.is_empty());
    assert_eq!(
        app.active.extra.caret_synced_version,
        app.active.buffer.version()
    );
    assert_eq!(
        app.active.extra.doc_saved_version,
        Some(app.active.buffer.version())
    );
    assert_eq!(app.active.extra.history_preview, None);
    assert_eq!(app.active.extra.history_scroll_before, None);

    app.load_path(a.clone()); // A reactivates: every sentinel round-trips.
    assert!(app.active.extra.shift_selecting);
    assert_eq!(app.active.extra.scroll_lines, 11);
    assert_eq!(app.active.extra.spell_cache.len(), a_spell_len);
    assert_eq!(
        app.active.extra.spell_checked_version,
        Some(app.active.buffer.version())
    );
    assert!(app.active.extra.sync_text_cache.is_some());
    assert_eq!(app.active.extra.caret_synced_version, 999);
    assert_eq!(app.active.extra.doc_saved_version, Some(777));
    assert_eq!(app.active.extra.scratch_saved_version, Some(888));
    assert!(app.active.extra.disk_mtime.is_some());
    assert!(app.active.extra.scratch_mtime.is_some());
    assert_eq!(
        app.active.extra.history_preview,
        Some(("42".to_string(), "old text".to_string()))
    );
    assert_eq!(
        app.active.extra.history_scroll_before,
        Some(crate::render::ScrollPos::at_row(55))
    );

    // A -> B -> C -> A: not merely a 2-slot toggle artifact.
    app.load_path(b.clone());
    app.load_path(c.clone());
    app.load_path(a.clone());
    assert_eq!(
        app.active.extra.scroll_lines, 11,
        "A's state survives a 3-way swap too"
    );
    assert_eq!(app.active.extra.caret_synced_version, 999);
    assert_eq!(
        app.active.extra.history_preview,
        Some(("42".to_string(), "old text".to_string()))
    );
}

#[test]
fn switching_buffers_isolates_the_spell_cache() {
    // The SAME version-0 isolation law `switching_buffers_isolates_the_view_
    // text_cache` proves for `sync_text_cache`, extended to `spell_cache` /
    // `spell_checked_version`: a fresh buffer restarts its edit version at 0,
    // so a KEYED spell verdict computed for one buffer must never paint over
    // another buffer sitting at the same version (CLAUDE.md's cache-key-
    // discipline tripwire).
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.md");
    let b = PathBuf::from("/proj/b.md");
    let mem = InMemoryFs::new()
        .with_file(&a, "helo world\n") // misspelled: "helo"
        .with_file(&b, "hello world\n"); // correctly spelled
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    app.run_spellcheck_now();
    assert!(!app.active.extra.spell_cache.is_empty(), "A misspells");
    assert_eq!(
        app.active.extra.spell_checked_version,
        Some(app.active.buffer.version())
    );

    app.load_path(b.clone());
    // B is FRESH: no stale spell cache carried over from A, and its own
    // (empty) verdict must not be confused with A's checked version — both
    // buffers can sit at version 0 simultaneously.
    assert!(
        app.active.extra.spell_cache.is_empty(),
        "B must not inherit A's verdicts"
    );
    assert_eq!(
        app.active.extra.spell_checked_version, None,
        "B has not been checked yet"
    );
    app.run_spellcheck_now();
    assert!(
        app.active.extra.spell_cache.is_empty(),
        "B is correctly spelled"
    );
    assert_eq!(
        app.active.extra.spell_checked_version,
        Some(app.active.buffer.version())
    );

    app.load_path(a.clone());
    assert!(
        !app.active.extra.spell_cache.is_empty(),
        "A's OWN verdicts are restored, not B's empty one"
    );
}

#[test]
fn fresh_buffer_starts_with_default_buffer_extra() {
    // A newly-created buffer (never before backgrounded) gets `BufferExtra::
    // default()`, not a leaked carry-over from whatever was active before —
    // proven at the seam that installs a FRESH entry (`new_document`'s
    // `start_fresh_note`, distinct from the registry-hit branch).
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.md");
    let mem = InMemoryFs::new().with_file(&a, "# a\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    app.active.extra.scroll_lines = 40; // dirty the CURRENT buffer's extra
    app.active.extra.history_preview = Some(("1".to_string(), "x".to_string()));

    app.new_document(); // a brand-new, never-before-seen buffer

    assert_eq!(
        app.active.extra.scroll_lines, 0,
        "fresh-defaults, not carried over"
    );
    assert!(!app.active.extra.shift_selecting);
    assert!(app.active.extra.spell_cache.is_empty());
    assert_eq!(app.active.extra.spell_checked_version, None);
    assert_eq!(app.active.extra.sync_text_cache, None);
    assert_eq!(
        app.active.extra.caret_synced_version,
        app.active.buffer.version(),
        "the synced version matches the fresh buffer's own version"
    );
    assert_eq!(app.active.extra.history_preview, None);
    assert_eq!(app.active.extra.history_scroll_before, None);
    assert!(
        app.active.buffer.folds().is_empty(),
        "a fresh note has no folds"
    );
}
