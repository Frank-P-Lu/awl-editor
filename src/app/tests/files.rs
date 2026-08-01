use super::*;

// ── GOTO FILE-INDEX FRESHNESS (queue: "file picker freshness") ──────────
//
// The go-to overlay (`C-x f`) corpus comes from `App.file_index`, a CACHED
// field only ever rebuilt on specific triggers (root switch, a note's first
// save, a rename, a move) — never simply because the picker summoned. A file
// dropped into the root by another process, or a shell command, while awl
// sits open would never appear until one of those triggers happened to also
// fire. The fix: RE-SCAN ON EVERY SUMMON via `App::rescan_file_index` (called
// from `App::apply`'s `Action::OpenGoto` arm, over the `FileSystem` trait) —
// no watcher, no TTL, just re-walk right as the overlay opens.

#[test]
fn rescan_file_index_picks_up_a_file_created_after_the_last_scan() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new().with_file("/proj/a.txt", "a\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(None, "/proj", Config::empty());
    // The initial scan (at App::new) sees only the file that existed then.
    assert_eq!(app.project_location.file_index, vec!["a.txt".to_string()]);
    // SUMMON #1 (simulated: `rescan_file_index` is exactly what `C-x f`
    // triggers): still just the one file — nothing has changed yet.
    app.rescan_file_index();
    assert_eq!(app.project_location.file_index, vec!["a.txt".to_string()]);
    // A file appears on disk WITHOUT going through awl at all (another
    // process, a git checkout, a plain `touch`) — the picker is CLOSED at
    // this point, so nothing in awl has any reason to know yet.
    mem.write(std::path::Path::new("/proj/b.txt"), b"b\n")
        .unwrap();
    assert_eq!(
        app.project_location.file_index,
        vec!["a.txt".to_string()],
        "the cached index does not spontaneously update"
    );
    // SUMMON #2 (`C-x f` again): the fresh scan MUST find it.
    app.rescan_file_index();
    assert_eq!(
        app.project_location.file_index,
        vec!["a.txt".to_string(), "b.txt".to_string()],
        "re-summoning must re-scan and pick up the new file"
    );
    // Build the ACTUAL overlay the way `App::apply`'s Goto arm does, to prove
    // the fresh index really reaches the summoned picker's corpus (the same
    // `overlay::build` the live App and headless replay both call).
    let effective_keep = app.config.effective_linux_keep();
    let build_ctx = crate::overlay::BuildCtx {
        goto_corpus: app.project_location.file_index.clone(),
        goto_open: Vec::new(),
        goto_recent: Vec::new(),
        goto_times: Vec::new(),
        config_keys: &app.config.keys,
        config_linux_keep: &effective_keep,
        goto_headings: Vec::new(),
        spell_target: None,
        history_entries: Vec::new(),
        history_now: None,
        history_session_start: None,
        settings_values: Default::default(),
        assets: Vec::new(),
        has_waiter: false,
    };
    let ov = crate::overlay::build(crate::overlay::OverlayKind::Goto, &build_ctx)
        .expect("Goto always summons");
    assert!(ov.accepts().contains(&"b.txt"), "the new file is listed");
}

// ── THE KEYMAP FLAVOR ROUND — the Settings "Keymap" toggle round-trip ────

/// Enter on the "Keymap" settings row (`App::toggle_keymap_flavor`, the
/// special-cased door `App::setting_toggle` routes "keymap" through):
/// flips native <-> emacs, PERSISTS the flip format-preservingly (the same
/// `persist_pref` owner every other sticky pref rides), and re-applies the
/// keymap LIVE from the updated in-memory config — proven here by feeding
/// the SAME `app.config.effective_linux_keep()` a fresh `KeymapState`
/// would consume (the exact composition `toggle_keymap_flavor` rebuilds
/// `self.keymap` from) into a `Convention::Linux`-pinned keymap and
/// confirming it now carries the full emacs preset.
#[test]
fn settings_keymap_toggle_flips_persists_and_live_reapplies() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let mut app = app_on(None, "/proj", cfg);
    assert_eq!(
        app.config.keymap_flavor(),
        crate::keymap::KeymapFlavor::Native,
        "starts native"
    );

    // Enter #1: native -> emacs.
    app.toggle_keymap_flavor();
    assert_eq!(
        app.config.keymap_flavor(),
        crate::keymap::KeymapFlavor::Emacs,
        "in-memory mirror flips"
    );
    let written = mem
        .read_to_string(std::path::Path::new("/cfg/config.toml"))
        .unwrap();
    assert!(
        written.contains("keymap = \"emacs\""),
        "persisted format-preservingly: {written:?}"
    );

    // LIVE RE-APPLY: the same composed keep-list the toggle rebuilt
    // `self.keymap` from now carries the WHOLE emacs preset — build a
    // fresh convention-pinned keymap from exactly that composition (the
    // private `KeymapState.linux_keep` field can't be introspected from
    // here, so this proves the INPUT the live rebuild consumed, which
    // `keymap::tests::keymap_flavor_emacs_preset_reverts_every_displaced_chord_to_emacs_meaning`
    // already proves is sufficient to flip dispatch).
    let effective = app.config.effective_linux_keep();
    let preset = crate::keymap::linux_emacs_preset_keep();
    // The insert-link-yields-to-kill-line round's built-in floor
    // (`keymap::linux_builtin_keep()`) rides ALONG with the preset — it is
    // NOT flavor-gated, so it's present under emacs too, just not part of
    // `preset` itself (see `linux_builtin_keep()`'s own doc).
    assert_eq!(
        effective.len(),
        preset.len() + crate::keymap::linux_builtin_keep().len(),
        "the live rebuild's keep-list is the whole preset plus the built-in floor"
    );
    for chord in &preset {
        assert!(
            effective.contains(chord),
            "{chord:?} missing from the live rebuild's keep-list"
        );
    }
    for chord in crate::keymap::linux_builtin_keep() {
        assert!(
            effective.iter().any(|c| c == chord),
            "{chord:?} missing from the live rebuild's keep-list"
        );
    }

    // Enter #2: emacs -> native (round-trips cleanly, doesn't accumulate).
    app.toggle_keymap_flavor();
    assert_eq!(
        app.config.keymap_flavor(),
        crate::keymap::KeymapFlavor::Native,
        "flips back"
    );
    let written2 = mem
        .read_to_string(std::path::Path::new("/cfg/config.toml"))
        .unwrap();
    assert!(
        written2.contains("keymap = \"native\""),
        "the second toggle persists too: {written2:?}"
    );
    // Native flavor: no preset widening, but the built-in floor is still
    // there (it's unconditional, not flavor-gated) — never truly empty.
    assert_eq!(
        app.config.effective_linux_keep().len(),
        crate::keymap::linux_builtin_keep().len(),
        "native flavor: no preset widening, just the built-in floor"
    );
}

/// LAW TEST (the "settings toggle rows dispatch live" round): EVERY row
/// the corpus marks `SettingKind::Toggle` — enumerated straight off
/// `settings::visible_rows()`, never hand-copied — round-trips through
/// the REAL live door, `App::setting_toggle(key)` (exactly what
/// `Effect::SettingToggle` resolves to at the `app/apply.rs` seam, see
/// `App::apply`'s `Effect::SettingToggle { key } => self.setting_toggle(&key)`
/// arm): the value readout VISIBLY CHANGES after one toggle, and
/// round-trips back to its exact starting value after a second — so a
/// toggle that silently no-ops (the Keymap-row bug: wired in
/// `settings::toggle_key` and in `settings_accept`, but never driven
/// through `App::setting_toggle` itself by any prior test — the prior
/// `settings_keymap_toggle_flips_persists_and_live_reapplies` test called
/// `app.toggle_keymap_flavor()` directly, skipping the string-keyed
/// dispatch a live Enter/click actually goes through) fails here instead
/// of shipping quietly. Companion:
/// `actions::tests::overlay_drive::every_settings_toggle_row_signals_its_own_setting_toggle_key`
/// (the pure `apply_transition`-level half: Enter on the row signals the RIGHT
/// key in the first place). Each toggle is undone immediately after
/// asserting it, so every process-global this sweep touches (page /
/// typewriter / wysiwyg / inline images / ligatures / spellcheck /
/// writing nits / outline / menu bar / reduce motion) is back to its
/// pre-test value by the time the lock releases — no leak into a sibling
/// test, mirroring the `page::measure()` save/restore convention used
/// elsewhere in this file. (16 toggles as of item 77's "File visibility" row.)
#[test]
fn every_settings_toggle_row_dispatches_live_and_flips_its_value() {
    use crate::fs::InMemoryFs;
    let _g2 = crate::fs::FsGuard::install(Arc::new(InMemoryFs::new()));
    let _g = crate::testlock::serial();

    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let mut app = app_on(None, "/proj", cfg);

    let toggle_rows: Vec<crate::settings::SettingRow> = crate::settings::visible_rows()
        .into_iter()
        .filter(|r| r.kind == crate::settings::SettingKind::Toggle)
        .copied()
        .collect();
    assert_eq!(
        toggle_rows.len(),
        16,
        "the toggle roster changed size — update this sweep deliberately"
    );

    let gather = |app: &App| {
        crate::settings::SettingsValues::gather(
            &app.config,
            &app.project_location.root,
            app.zoom,
            crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
        )
    };
    for row in &toggle_rows {
        let key = crate::settings::toggle_key(row.id).expect("a Toggle row always has a key");
        let values0 = gather(&app);
        let before = crate::settings::value_for(row, &values0);

        // DATE FORMAT is a genuine FIVE-way CYCLE (unlike every other Toggle
        // row here, which is a plain bool OR "Keymap"'s own 2-state cycle) —
        // snapshot the process-global directly so restoration below doesn't
        // assume "two toggles returns to start" (true for a 2-state row,
        // false for a 5-state one).
        let date_format_before = (row.name == "Date format").then(crate::dateformat::active_format);

        app.setting_toggle(key);
        let values1 = gather(&app);
        let after = crate::settings::value_for(row, &values1);
        assert_ne!(
            before, after,
            "row {:?} (key {:?}) did not visibly flip its value readout — the live dispatch is a silent no-op",
            row.name, key
        );

        if let Some(saved) = date_format_before {
            crate::dateformat::set_active_format(saved); // restore, no leak
            continue;
        }

        // Toggle back — restores the global/config AND proves the flip is
        // a clean round-trip, not a one-way ratchet.
        app.setting_toggle(key);
        let values2 = gather(&app);
        let restored = crate::settings::value_for(row, &values2);
        assert_eq!(
            restored, before,
            "row {:?} (key {:?}) did not round-trip back to its starting value",
            row.name, key
        );
    }
}

/// The corpus GREW to carry the row: "Keymap" is a real, visible settings
/// row (mirrors `settings::tests::settings_table_names_are_unique`'s own
/// count law, exercised here through the App's own config/root — a
/// belt-and-suspenders confirmation that the live overlay build would
/// actually list it).
#[test]
fn settings_corpus_includes_the_keymap_row() {
    assert!(crate::settings::visible_names().contains(&"Keymap".to_string()));
    assert_eq!(
        crate::settings::toggle_key(crate::settings::SettingId::Keymap),
        Some("keymap")
    );
}

#[test]
fn disk_changed_truth_table() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let p = std::path::Path::new("/d/f.md");
    // (None, None): the file never existed — our write CREATES it, no clobber.
    assert!(!App::disk_changed(p, None));
    mem.write(p, b"v1").unwrap();
    let t1 = App::disk_mtime_of(p);
    assert!(t1.is_some(), "the fake records mtimes");
    // (Some, Some) equal → unchanged.
    assert!(!App::disk_changed(p, t1));
    // (Some, None): the file APPEARED externally since we looked.
    assert!(App::disk_changed(p, None));
    // (Some, Some) differing → a real external change.
    std::thread::sleep(Duration::from_millis(2)); // ensure a distinct mtime
    mem.write(p, b"v2").unwrap();
    assert!(App::disk_changed(p, t1));
    // (Some, Some) with the SAME mtime but a DIFFERENT size → a same-tick
    // external edit (equal mtime, changed content) must still be caught by the
    // size guard, or we'd silently overwrite it.
    let cur = App::disk_mtime_of(p).expect("v2 exists");
    let same_tick_other_size = Some(crate::fs::Metadata {
        modified: cur.modified,
        len: cur.len.map(|n| n + 1),
    });
    assert!(App::disk_changed(p, same_tick_other_size));
    // (None, Some): the file was DELETED externally (renamed away here — the
    // trait has no remove op, and a rename models the same disappearance).
    let last = App::disk_mtime_of(p);
    mem.rename(p, std::path::Path::new("/d/elsewhere.md"))
        .unwrap();
    assert!(App::disk_changed(p, last));
}

#[test]
fn autosave_flush_writes_doc_and_snapshots_loose_file() {
    use crate::fs::{FileSystem, InMemoryFs};
    let p = PathBuf::from("/notes/draft.md");
    let mem = InMemoryFs::new().with_file(&p, "v1\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());
    assert!(
        app.persistence.engine_last_write_at().is_none(),
        "the debug panel's autosave clock is untouched before any write"
    );
    app.active.buffer.set_text("v2\n");
    app.autosave_flush();
    assert_eq!(
        mem.read_to_string(&p).unwrap(),
        "v2\n",
        "the edit hit the disk"
    );
    assert_eq!(
        app.active.extra.doc_saved_version,
        Some(app.active.buffer.version()),
        "the flushed version is bookkept"
    );
    assert!(app.notice.is_none(), "a clean write raises no notice");
    assert!(
        app.persistence.engine_last_write_at().is_some(),
        "a real engine write stamps the debug panel's autosave clock"
    );
    // The debug panel's pure composer agrees: enabled + not held + a stamped
    // write => Saved (never Off/Held after a clean autosave).
    assert!(matches!(
        crate::debug::autosave_state(app.config.autosave_on(), app.notice.is_some(), Some(0)),
        crate::debug::AutosaveState::Saved(Some(0))
    ));
    // Every save records: the loose file grew a history snapshot.
    assert!(
        !crate::history::list(&p).is_empty(),
        "autosave records a local-history snapshot for a loose file"
    );
    // An unchanged buffer is not re-written (version bookkeeping short-circuits).
    let t = App::disk_mtime_of(&p);
    app.autosave_flush();
    assert_eq!(
        App::disk_mtime_of(&p),
        t,
        "no redundant write for a clean buffer"
    );
}

#[test]
fn autosave_flush_skips_and_notices_when_disk_changed_externally() {
    use crate::fs::{FileSystem, InMemoryFs};
    let p = PathBuf::from("/notes/draft.md");
    let mem = InMemoryFs::new().with_file(&p, "disk v1\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());
    // Someone ELSE writes the file behind awl's back.
    std::thread::sleep(Duration::from_millis(2)); // distinct mtime
    mem.write(&p, b"external edit\n").unwrap();
    app.active.buffer.set_text("mine\n");
    app.autosave_flush();
    // The CLOBBER GUARD held the write: the external edit survives on disk.
    assert_eq!(
        mem.read_to_string(&p).unwrap(),
        "external edit\n",
        "autosave never overwrites external edits"
    );
    assert_eq!(
        app.notice.as_deref(),
        Some(CLOBBER_NOTICE),
        "a calm notice is raised"
    );
    assert!(
        app.persistence.engine_last_write_at().is_none(),
        "a HELD write must never stamp the debug panel's autosave clock — no write happened"
    );
    // The debug panel's pure composer agrees: held wins over "nothing written yet".
    assert_eq!(
        crate::debug::autosave_state(app.config.autosave_on(), app.notice.is_some(), None),
        crate::debug::AutosaveState::Held
    );
    // The version is marked handled so the idle timer doesn't spin; the NEXT
    // edit re-arms the engine (and the notice would recur calmly).
    assert_eq!(
        app.active.extra.doc_saved_version,
        Some(app.active.buffer.version())
    );
}

#[test]
fn autosave_off_disables_flush() {
    use crate::fs::{FileSystem, InMemoryFs};
    let p = PathBuf::from("/notes/draft.md");
    let mem = InMemoryFs::new().with_file(&p, "v1\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let cfg = Config {
        autosave: Some(false),
        ..Config::empty()
    };
    let mut app = app_on(Some(p.clone()), "/notes", cfg);
    app.active.buffer.set_text("v2\n");
    app.autosave_flush();
    assert_eq!(
        mem.read_to_string(&p).unwrap(),
        "v1\n",
        "autosave = false leaves the disk untouched"
    );
    assert!(app.notice.is_none());
    assert!(
        app.persistence.engine_last_write_at().is_none(),
        "a disabled engine never stamps the debug panel's autosave clock"
    );
    // The debug panel's pure composer agrees: disabled wins over everything.
    assert_eq!(
        crate::debug::autosave_state(app.config.autosave_on(), app.notice.is_some(), None),
        crate::debug::AutosaveState::Off
    );
}

#[test]
fn load_path_flushes_the_leaving_buffer() {
    use crate::fs::{FileSystem, InMemoryFs};
    let a = PathBuf::from("/notes/a.md");
    let b = PathBuf::from("/notes/b.md");
    let mem = InMemoryFs::new().with_file(&a, "A\n").with_file(&b, "B\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/notes", Config::empty());
    app.active.buffer.set_text("A edited\n");
    app.load_path(b.clone());
    assert_eq!(
        mem.read_to_string(&a).unwrap(),
        "A edited\n",
        "switching files flushes the buffer being left"
    );
    assert_eq!(app.active.buffer.text(), "B\n", "the new file is open");
    assert_eq!(
        app.active.extra.doc_saved_version,
        Some(app.active.buffer.version()),
        "the arriving buffer starts saved"
    );
}

// ── i18n WRITE-BACK-ONCE (App::new launch arg + App::load_path switch) ───

#[test]
fn launching_on_an_untagged_japanese_file_tags_it_once() {
    use crate::fs::{FileSystem, InMemoryFs};
    let p = PathBuf::from("/notes/nihongo.md");
    let original = "これは日本語の文章です。\n";
    let mem = InMemoryFs::new().with_file(&p, original);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let app = app_on(Some(p.clone()), "/notes", Config::empty());
    assert_eq!(
        app.active.buffer.text(),
        format!("---\nlang: ja\n---\n{original}"),
        "an untagged kana-bearing doc is tagged ja on first open"
    );
    // NEVER a silent disk write: the file on disk is untouched, and the
    // buffer reads as DIRTY (past doc_saved_version) so the ordinary
    // autosave engine picks the tag up on the next idle/blur/switch/quit.
    assert_eq!(
        mem.read_to_string(&p).unwrap(),
        original,
        "disk is untouched"
    );
    assert!(
        app.active.extra.doc_saved_version.unwrap() < app.active.buffer.version(),
        "the stamped tag is a PENDING edit, not already-saved"
    );
}

#[test]
fn write_back_never_touches_a_pure_latin_document() {
    use crate::fs::InMemoryFs;
    let p = PathBuf::from("/notes/english.md");
    let original = "Just some ordinary English prose.\n";
    let mem = InMemoryFs::new().with_file(&p, original);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let app = app_on(Some(p.clone()), "/notes", Config::empty());
    assert_eq!(
        app.active.buffer.text(),
        original,
        "a pure-Latin doc is never touched"
    );
    assert_eq!(
        app.active.extra.doc_saved_version,
        Some(app.active.buffer.version()),
        "no edit landed -> still reads as saved"
    );
}

#[test]
fn write_back_never_fires_on_a_non_markdown_file() {
    use crate::fs::InMemoryFs;
    // A `.rs` file with a Japanese string literal: frontmatter is a
    // markdown/notes convention, and stamping `---`/`lang:` text into a
    // code file would corrupt it, so this must stay untouched.
    let p = PathBuf::from("/proj/main.rs");
    let original = "fn main() {\n    println!(\"こんにちは\");\n}\n";
    let mem = InMemoryFs::new().with_file(&p, original);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let app = app_on(Some(p.clone()), "/proj", Config::empty());
    assert_eq!(
        app.active.buffer.text(),
        original,
        "a non-markdown file is never tagged"
    );
}

#[test]
fn write_back_uses_the_configured_cjk_priority_for_ambiguous_han() {
    use crate::fs::InMemoryFs;
    let p = PathBuf::from("/notes/hanzi.md");
    let original = "汉字漢字\n"; // Han only, no kana/hangul/bopomofo -> ambiguous
    let mem = InMemoryFs::new().with_file(&p, original);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let cfg = Config {
        cjk_priority: Some(vec![
            crate::frontmatter::Lang::ZhHans,
            crate::frontmatter::Lang::Ja,
        ]),
        ..Config::empty()
    };
    let app = app_on(Some(p.clone()), "/notes", cfg);
    assert_eq!(
        app.active.buffer.text(),
        format!("---\nlang: zh-Hans\n---\n{original}")
    );
}

#[test]
fn write_back_is_undoable_with_cmd_z() {
    use crate::fs::InMemoryFs;
    let p = PathBuf::from("/notes/nihongo.md");
    let original = "こんにちは\n";
    let mem = InMemoryFs::new().with_file(&p, original);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());
    assert_ne!(app.active.buffer.text(), original, "the tag landed");
    app.active.buffer.undo();
    assert_eq!(
        app.active.buffer.text(),
        original,
        "Cmd-Z removes the stamped tag cleanly"
    );
}

#[test]
fn write_back_never_re_tags_a_document_already_carrying_frontmatter() {
    use crate::fs::InMemoryFs;
    let p = PathBuf::from("/notes/tagged.md");
    // Already tagged (as if a previous session's write-back had already
    // fired and been saved) — must never gain a SECOND block.
    let already = "---\nlang: ja\n---\nこんにちは\n";
    let mem = InMemoryFs::new().with_file(&p, already);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let app = app_on(Some(p.clone()), "/notes", Config::empty());
    assert_eq!(
        app.active.buffer.text(),
        already,
        "an already-tagged doc is untouched"
    );
    assert_eq!(
        app.active.extra.doc_saved_version,
        Some(app.active.buffer.version()),
        "no edit landed -> still reads as saved"
    );
}

#[test]
fn write_back_never_fires_twice_across_a_reopen() {
    use crate::fs::{FileSystem, InMemoryFs};
    let a = PathBuf::from("/notes/a.md");
    let b = PathBuf::from("/notes/nihongo.md");
    let original = "こんにちは\n";
    let mem = InMemoryFs::new()
        .with_file(&a, "hello\n")
        .with_file(&b, original);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/notes", Config::empty());
    // First open of `b`: tags it (still only in-memory — disk untouched).
    app.load_path(b.clone());
    let tagged = app.active.buffer.text();
    assert_eq!(tagged, format!("---\nlang: ja\n---\n{original}"));
    // Simulate a save (autosave/Cmd-S would write exactly this).
    mem.write(&b, tagged.as_bytes()).unwrap();
    // Switch away, then back: `load_path`'s SWITCH branch (already open in
    // the registry) restores the live buffer untouched — no second call.
    app.load_path(a.clone());
    app.load_path(b.clone());
    assert_eq!(
        app.active.buffer.text(),
        tagged,
        "no second frontmatter block, live round trip"
    );
    // And a FRESH session reopening the now-tagged file also never re-tags
    // (the write-back gate is `frontmatter::detect`, not a one-shot flag).
    let app2 = app_on(Some(b.clone()), "/notes", Config::empty());
    assert_eq!(
        app2.active.buffer.text(),
        tagged,
        "a fresh session sees the tag and never re-fires"
    );
}

#[test]
fn load_path_preserves_a_clobber_notice_the_leaving_flush_just_raised() {
    // REGRESSION (code review nit): if the flush `load_path` runs on the
    // buffer being LEFT hits the autosave clobber guard (the file changed
    // on disk outside awl), the notice it raises must survive the switch
    // — the unconditional `self.notice = None` a few lines later used to
    // wipe it in the very same call, before a single frame ever rendered
    // it, so the user never learned their unsaved edit was held.
    use crate::fs::{FileSystem, InMemoryFs};
    let a = PathBuf::from("/notes/a.md");
    let b = PathBuf::from("/notes/b.md");
    let mem = InMemoryFs::new().with_file(&a, "A\n").with_file(&b, "B\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(a.clone()), "/notes", Config::empty());
    app.active.buffer.set_text("A edited\n");
    // Someone ELSE writes A behind awl's back before we switch away from it.
    std::thread::sleep(Duration::from_millis(2)); // distinct mtime
    mem.write(&a, b"external edit\n").unwrap();

    app.load_path(b.clone());

    assert_eq!(
        app.active.buffer.text(),
        "B\n",
        "the switch to B still happens"
    );
    assert_eq!(
        mem.read_to_string(&a).unwrap(),
        "external edit\n",
        "the clobber guard held A's write — the external edit is intact"
    );
    assert_eq!(
        app.notice.as_deref(),
        Some(CLOBBER_NOTICE),
        "the notice raised while leaving A must survive into the switch, not vanish unseen"
    );
}

#[test]
fn scratch_stash_and_restore_round_trip() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let stash = crate::fs::scratch_stash_path();
    // A no-file launch, some typing, then a flush (idle/blur/quit all route here).
    let mut app = app_on(None, "/proj", Config::empty());
    app.active.buffer.set_text("brain dump\n");
    app.autosave_flush();
    assert_eq!(
        mem.read_to_string(&stash).unwrap(),
        "brain dump\n",
        "the scratch stashed"
    );
    assert!(
        !crate::history::list(&stash).is_empty(),
        "the persistent scratch grows its own timeline"
    );
    // A fresh no-argument launch RESTORES it: still path-less, still the
    // markdown-first scratch surface, not a note.
    let mut app2 = app_on(None, "/proj", Config::empty());
    assert_eq!(
        app2.active.buffer.text(),
        "brain dump\n",
        "the stash restores"
    );
    assert!(
        app2.active.buffer.path().is_none(),
        "restored scratch stays path-less"
    );
    assert!(app2.active.buffer.is_markdown() && !app2.active.buffer.is_unnamed_fresh());
    // The restore stamped the stash mtime, so a follow-up edit + flush is not
    // mistaken for a two-instance clobber.
    app2.active.buffer.set_text("brain dump\nmore\n");
    app2.autosave_flush();
    assert_eq!(mem.read_to_string(&stash).unwrap(), "brain dump\nmore\n");
    assert!(
        app2.notice.is_none(),
        "no false clobber notice after a restore"
    );
}

// ── SAVE-FEEDBACK round: scratch Save -> note, notice, dirty title marker ──

#[test]
fn convert_scratch_and_save_promotes_the_buffer_and_retires_the_stash() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    // Stash an OLD scratch content first, exactly like a real prior session
    // would have — the very ghost-copy risk the round's own doc names.
    let stash = crate::fs::scratch_stash_path();
    mem.write(&stash, b"yesterday's dump\n").unwrap();

    let mut app = app_on(None, "/proj", Config::empty());
    assert_eq!(
        app.active.buffer.text(),
        "yesterday's dump\n",
        "restored from the stash first"
    );
    assert!(
        app.active.buffer.path().is_none() && !app.active.buffer.is_unnamed_fresh(),
        "still a true scratch"
    );

    app.convert_scratch_and_save();

    // ONE-SHOT NAMING (item 76): the derive-a-name save ALSO clears the
    // fresh-document marker in the same step — by the time this call
    // returns, the buffer reads as an ORDINARY pathed file, not a lasting
    // "note" identity.
    assert!(
        !app.active.buffer.is_unnamed_fresh(),
        "Cmd-S named the document once — it's an ordinary file now, not a lasting note identity"
    );
    let p = app.active.buffer.path().unwrap().to_path_buf();
    assert!(
        p.starts_with("/proj"),
        "item 76: the document lands under the ACTIVE folder, not a separate notes-root concept: {p:?}"
    );
    assert_eq!(mem.read_to_string(&p).unwrap(), "yesterday's dump\n");
    assert_eq!(
        app.active.buffer.path(),
        Some(p.as_path()),
        "Buffer::path is the new path"
    );
    assert_eq!(app.notice.as_deref(), Some("saved"));
    // THE STASH IS RETIRED: a later bare relaunch must never resurrect a
    // ghost copy of content that is now a real, named file.
    assert!(
        mem.read_to_string(&stash).is_err(),
        "the stash file was removed"
    );
}

#[test]
fn convert_scratch_and_save_second_save_is_a_plain_save() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(None, "/proj", Config::empty());
    app.active.buffer.set_text("first entry\n");
    app.convert_scratch_and_save();
    let named = app.active.buffer.path().unwrap().to_path_buf();

    // A SECOND explicit save (the buffer is now an ordinary note) must
    // NOT re-run the scratch-conversion machinery — same path, same file,
    // just the updated content. This spells out the two live-interpreter
    // halves: `Buffer::save()`, then `finish_manual_save` bookkeeping.
    app.active.buffer.set_text("first entry\nmore\n");
    app.active.buffer.save().unwrap();
    app.finish_manual_save(true, "saved".to_string());
    assert_eq!(
        app.active.buffer.path().unwrap(),
        named,
        "no re-homing on the second save"
    );
    assert_eq!(mem.read_to_string(&named).unwrap(), "first entry\nmore\n");
}

#[test]
fn convert_scratch_and_save_unwritable_active_folder_raises_a_calm_notice_never_a_panic() {
    // An active folder that can't be written to (a full disk, a permissions
    // error, …) must surface as the SAME calm notice a failed manual save
    // gets — never a terminal print, never a crash, and the scratch stash
    // is left untouched (nothing succeeded to retire it over).
    let _g = crate::fs::FsGuard::install(Arc::new(crate::fs::UnwritableFs));
    let mut app = app_on(None, "/proj", Config::empty());
    app.active.buffer.set_text("won't land\n");

    app.convert_scratch_and_save();

    assert!(
        app.notice
            .as_deref()
            .is_some_and(|n| n.starts_with("save failed:")),
        "a calm failure notice, not a panic: {:?}",
        app.notice
    );
}

// ── NOTES VERBS round: Rename note… / Duplicate note ──

#[test]
fn rename_current_file_happy_path_renames_disk_buffer_and_history() {
    use crate::fs::{FileSystem, InMemoryFs};
    let old = PathBuf::from("/notes/old.md");
    let mem = InMemoryFs::new().with_file(&old, "hi\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    // A prior snapshot exists under the OLD path — the ONE-OWNER rename must
    // carry it over so the timeline survives the rename.
    crate::history::record(&old, "hi\n", &Config::empty());
    assert!(
        !crate::history::list(&old).is_empty(),
        "arranged: a snapshot exists"
    );

    let mut app = app_on(Some(old.clone()), "/notes", Config::empty());
    assert_eq!(app.active.buffer.path(), Some(old.as_path()));

    app.rename_current_file("new.md");

    let new = PathBuf::from("/notes/new.md");
    assert_eq!(
        app.active.buffer.path(),
        Some(new.as_path()),
        "buffer follows the rename — the sole authoritative path (item 56)"
    );
    assert_eq!(mem.read_to_string(&new).unwrap(), "hi\n", "content moved");
    assert!(mem.read_to_string(&old).is_err(), "the old path is gone");
    assert_eq!(app.notice.as_deref(), Some("renamed to new.md"));
    // THE ONE-OWNER LAW: the history log followed too.
    assert!(
        !crate::history::list(&new).is_empty(),
        "history followed to the new path"
    );
    assert!(
        crate::history::list(&old).is_empty(),
        "nothing stranded under the old path"
    );
}

#[test]
fn rename_current_file_refuses_to_clobber_an_existing_name() {
    use crate::fs::{FileSystem, InMemoryFs};
    let old = PathBuf::from("/notes/old.md");
    let taken = PathBuf::from("/notes/taken.md");
    let mem = InMemoryFs::new()
        .with_file(&old, "old body\n")
        .with_file(&taken, "taken body\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(old.clone()), "/notes", Config::empty());

    app.rename_current_file("taken.md");

    assert_eq!(
        app.active.buffer.path(),
        Some(old.as_path()),
        "buffer stays put — refused, not clobbered"
    );
    assert_eq!(
        mem.read_to_string(&old).unwrap(),
        "old body\n",
        "old untouched"
    );
    assert_eq!(
        mem.read_to_string(&taken).unwrap(),
        "taken body\n",
        "never overwritten"
    );
    assert!(
        app.notice
            .as_deref()
            .is_some_and(|n| n.contains("already a file named")),
        "a calm refusal notice: {:?}",
        app.notice
    );
}

#[test]
fn rename_current_file_refuses_a_git_managed_file() {
    use crate::fs::{FileSystem, InMemoryFs};
    let old = PathBuf::from("/proj/tracked.md");
    let mem = InMemoryFs::new()
        .with_file(&old, "body\n")
        .with_dir("/proj/.git");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(old.clone()), "/proj", Config::empty());

    app.rename_current_file("renamed.md");

    assert_eq!(
        app.active.buffer.path(),
        Some(old.as_path()),
        "a git-managed file never renames here"
    );
    assert!(mem.exists(&old), "old path untouched");
    assert!(
        !mem.exists(&PathBuf::from("/proj/renamed.md")),
        "no new file created"
    );
    assert!(
        app.notice
            .as_deref()
            .is_some_and(|n| n.contains("git already tracks")),
        "a calm git-managed refusal notice: {:?}",
        app.notice
    );
}

#[test]
fn rename_current_file_unchanged_or_blank_name_is_a_quiet_no_op() {
    use crate::fs::InMemoryFs;
    let old = PathBuf::from("/notes/old.md");
    let mem = InMemoryFs::new().with_file(&old, "hi\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(old.clone()), "/notes", Config::empty());

    app.rename_current_file("old.md");
    assert_eq!(
        app.active.buffer.path(),
        Some(old.as_path()),
        "unchanged name: no-op"
    );
    assert!(app.notice.is_none(), "no notice for a no-op");

    app.rename_current_file("   ");
    assert_eq!(
        app.active.buffer.path(),
        Some(old.as_path()),
        "blank name: no-op"
    );
    assert!(app.notice.is_none(), "no notice for a no-op");
}

#[test]
fn duplicate_current_file_dedups_the_name_and_starts_a_fresh_history_timeline() {
    use crate::fs::{FileSystem, InMemoryFs};
    let old = PathBuf::from("/notes/old.md");
    // A prior "old-2.md" already exists, so the dedup must land on "old-3.md".
    let taken2 = PathBuf::from("/notes/old-2.md");
    let mem = InMemoryFs::new()
        .with_file(&old, "on disk\n")
        .with_file(&taken2, "someone else's\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    // The old file has its own history timeline.
    crate::history::record(&old, "on disk\n", &Config::empty());
    assert!(
        !crate::history::list(&old).is_empty(),
        "arranged: old has history"
    );

    let mut app = app_on(Some(old.clone()), "/notes", Config::empty());
    // Simulate an UNSAVED edit: the duplicate must carry the LIVE buffer
    // content, not necessarily what's on disk.
    app.active.buffer.set_text("live edit, not yet flushed\n");

    app.duplicate_current_file();

    let dup = PathBuf::from("/notes/old-3.md");
    assert_eq!(
        app.active.buffer.path(),
        Some(dup.as_path()),
        "switched to the deduped sibling"
    );
    assert_eq!(
        mem.read_to_string(&dup).unwrap(),
        "live edit, not yet flushed\n",
        "the copy captures the buffer's LIVE content"
    );
    assert!(
        mem.exists(&old),
        "the original file is untouched, still present"
    );
    assert!(
        mem.exists(&taken2),
        "the pre-existing -2 sibling is never clobbered"
    );
    // FRESH HISTORY: the duplicate is a brand-new file, so its own timeline
    // starts empty, even though the SOURCE had history.
    assert!(
        crate::history::list(&dup).is_empty(),
        "the copy starts a fresh history timeline"
    );
    // The ORIGINAL buffer was PARKED (backgrounded), never discarded — its
    // pending edit is still flushed to disk (autosave_flush runs before the
    // dedup scan) and its live state survives in the registry.
    let key = crate::buffers::BufferKey::path(&old);
    assert!(
        app.buffer_registry.contains(&key),
        "the original was parked, not dropped"
    );
}

#[test]
fn duplicate_current_file_on_a_pathless_buffer_is_a_quiet_no_op() {
    // HERMETIC: install an InMemoryFs before `App::new` so this test never
    // touches the machine's real `session.toml`/stash (`app_on(None, ..)`
    // runs the full App startup). FsGuard also holds `testlock::serial()`
    // for the test's life, so the pass no longer rides ordering luck.
    use crate::fs::InMemoryFs;
    let mem = InMemoryFs::new().with_dir("/proj");
    let _g = crate::fs::FsGuard::install(Arc::new(mem));
    let mut app = app_on(None, "/proj", Config::empty());
    assert!(app.active.buffer.path().is_none());
    app.duplicate_current_file();
    assert!(
        app.active.buffer.path().is_none(),
        "nothing to duplicate yet"
    );
    assert!(app.notice.is_none());
}

#[test]
fn finish_manual_save_ok_is_silent_failure_notices_the_error() {
    // SAVE-UX round: a SUCCESSFUL manual save raises NO bottom-center notice
    // (autosave is already silent; a lone non-fading "saved" is just noise).
    // A FAILURE still surfaces its error — errors must never go silent.
    use crate::fs::InMemoryFs;
    let _l = crate::testlock::serial();
    let p = PathBuf::from("/notes/draft.md");
    let mem = InMemoryFs::new().with_file(&p, "v1\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());

    app.finish_manual_save(true, "saved".to_string());
    assert_eq!(app.notice.as_deref(), Some("saved"));
    assert_eq!(app.notice_kind, NoticeKind::Toast);
    assert!(
        app.notice_expires_at.is_none(),
        "a headless test never arms a live timer"
    );

    app.finish_manual_save(false, "save failed: disk full".to_string());
    assert_eq!(app.notice.as_deref(), Some("save failed: disk full"));
}

#[test]
fn finish_manual_save_clears_a_freshly_named_documents_dirty_marker_immediately() {
    // BUG LOCK-DOWN (adapted for item 76's one-shot naming): `Buffer::save`
    // derives the filename AND clears the fresh-document marker in the SAME
    // step, so a document named by THIS save reads `is_unnamed_fresh() ==
    // false` + `doc_saved_version`-tracked immediately — `finish_manual_save`
    // must stamp `doc_saved_version` right here (the general, un-gated path
    // below) or the title `•` + native titlebar dot would linger until the
    // ordinary document autosave engine's next idle write incidentally
    // stamped it.
    use crate::fs::InMemoryFs;
    let _l = crate::testlock::serial();
    let notes = PathBuf::from("/notes");
    let mem = InMemoryFs::new().with_dir(&notes);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(None, "/notes", Config::empty());

    // Make the active buffer an UNNAMED FRESH DOCUMENT with content, then
    // write it to disk the way `apply_transition`'s `Action::Save` arm does before
    // signalling SaveDone.
    app.active.buffer.start_fresh_doc(notes.clone());
    app.active.buffer.set_text("note body\n");
    app.active.buffer.save().unwrap();
    assert!(
        !app.active.buffer.is_unnamed_fresh() && app.active.buffer.path().is_some(),
        "arranged: the save named it once — an ordinary pathed file now"
    );
    // Pre-bookkeeping the document reads DIRTY: `doc_saved_version` is still
    // stale (None) against the just-written version.
    assert!(
        app.is_document_dirty(),
        "arranged: the document reads dirty pre-bookkeeping"
    );

    app.finish_manual_save(true, "saved".to_string());

    assert!(
        !app.is_document_dirty(),
        "clean IMMEDIATELY after ⌘S, not ~400ms later"
    );
}

#[test]
fn finish_manual_save_clears_a_regular_files_dirty_marker_immediately() {
    // REGRESSION GUARD: a path-backed file reads `doc_saved_version` in
    // `is_document_dirty` — it was always fine, and must stay fine.
    use crate::fs::InMemoryFs;
    let _l = crate::testlock::serial();
    let p = PathBuf::from("/proj/doc.md");
    let mem = InMemoryFs::new().with_file(&p, "v1\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(p.clone()), "/proj", Config::empty());

    app.active.buffer.set_text("edited body\n");
    app.active.buffer.save().unwrap();
    assert!(
        !app.active.buffer.is_unnamed_fresh() && app.active.buffer.path().is_some(),
        "arranged: a saved file"
    );
    assert!(
        app.is_document_dirty(),
        "arranged: the file reads dirty pre-bookkeeping"
    );

    app.finish_manual_save(true, "saved".to_string());

    assert!(
        !app.is_document_dirty(),
        "a regular file is clean immediately after ⌘S"
    );
}

// ── SAVE-FEEDBACK round: the ambient dirty title marker ──

#[test]
fn sync_view_retitles_only_on_an_actual_dirty_flip() {
    // HERMETIC: install an InMemoryFs before `App::new` so this test never
    // touches the machine's real `session.toml`/stash (`app_on(None, ..)` runs
    // the full App startup, including the scratch-stash restore). FsGuard also
    // holds `testlock::serial()` for the test's life — without it this
    // uninstalled `app_on(None, ..)` read the PROCESS-GLOBAL active fs, so under
    // CI parallelism it raced INTO a concurrent test's installed InMemoryFs and,
    // finding that test's deliberately-corrupt scratch stash, preserved a second
    // `.corrupt-*` sibling into it — the deterministic CI-only failure of
    // `scratch_stash_invalid_utf8_preserves_a_corrupt_sibling_then_starts_a_blank_scratch`
    // (mirrors `duplicate_current_file_on_a_pathless_buffer_is_a_quiet_no_op`).
    use crate::fs::InMemoryFs;
    let mem = InMemoryFs::new().with_dir("/proj");
    let _g = crate::fs::FsGuard::install(Arc::new(mem));
    let mut app = app_on(None, "/proj", Config::empty());
    assert!(
        !app.persistence.title_cache_stale(false),
        "a fresh scratch buffer starts clean"
    );
    // No gpu/window in a hermetic App: `sync_view` bails before the title
    // comparison (its own gpu-present gate) — this proves the flip-tracking
    // logic itself is reachable + correct via `is_document_dirty` directly,
    // mirroring `update_title_uses_the_same_pure_window_title`'s own
    // "no live window, still exercised" shape.
    assert!(!app.is_document_dirty(), "just-loaded content starts saved");
    app.active.buffer.set_text("edited\n");
    assert!(
        app.is_document_dirty(),
        "an edit past the saved version is dirty"
    );
}

#[test]
fn is_document_dirty_clears_on_autosave_not_just_manual_save() {
    // The definition this round settled on for the title's dirty marker:
    // "unsaved" by the SAME version-vs-saved-version bookkeeping the
    // autosave engine tracks — so an AUTOSAVED (not manually Cmd-S'd)
    // document reads as clean too, never stuck showing the edited marker
    // on content that's already safely on disk.
    use crate::fs::{FileSystem, InMemoryFs};
    let p = PathBuf::from("/notes/draft.md");
    let mem = InMemoryFs::new().with_file(&p, "v1\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());
    assert!(!app.is_document_dirty());
    app.active.buffer.set_text("v2\n");
    assert!(app.is_document_dirty(), "an unsaved edit reads dirty");
    app.autosave_flush(); // NOT a manual save — the background engine
    assert_eq!(mem.read_to_string(&p).unwrap(), "v2\n");
    assert!(
        !app.is_document_dirty(),
        "autosave clears the dirty marker too"
    );
}

#[test]
fn scratch_stash_invalid_utf8_preserves_a_corrupt_sibling_then_starts_a_blank_scratch() {
    // DATA-SAFETY HARDENING: the scratch stash IS a manuscript, so a
    // stash file that's PRESENT but fails to decode as UTF-8 text (real
    // disk corruption, never a bug write_atomic itself can produce) must
    // never be silently discarded — a `.corrupt-*` sibling preserves the
    // raw bytes BEFORE `App::new` falls back to a blank scratch buffer.
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let stash = crate::fs::scratch_stash_path();
    // Invalid UTF-8: a lone continuation byte can never decode.
    mem.write(&stash, &[0x2E, 0x62, 0xFF, 0xFE, 0x0A]).unwrap();

    let app = app_on(None, "/proj", Config::empty());
    assert_eq!(
        app.active.buffer.text(),
        "",
        "an undecodable stash falls back to a blank scratch"
    );
    assert!(app.active.buffer.path().is_none());

    let dir = stash.parent().unwrap();
    let names: Vec<String> = mem
        .read_dir(dir)
        .unwrap()
        .into_iter()
        .map(|e| e.name)
        .collect();
    let stash_name = stash.file_name().unwrap().to_string_lossy().into_owned();
    let backup_prefix = format!("{stash_name}.corrupt-");
    let backups: Vec<&String> = names
        .iter()
        .filter(|n| n.starts_with(&backup_prefix))
        .collect();
    assert_eq!(
        backups.len(),
        1,
        "exactly one corrupt sibling preserved: {names:?}"
    );
    let backup_bytes = mem.read(&dir.join(backups[0])).unwrap();
    assert_eq!(
        backup_bytes,
        vec![0x2E, 0x62, 0xFF, 0xFE, 0x0A],
        "the sibling holds the ORIGINAL undecodable bytes verbatim"
    );
}

#[test]
fn blur_flush_never_reloads_buffer_or_resets_cursor() {
    // WEB STRESS-TEST HYPOTHESIS (characterized, not reproduced): a Playwright
    // run typing "AAA" then, in a LATER dispatch batch, "BBB" observed BBB
    // landing at buffer position 0 instead of after "AAA", as if a blur/
    // visibility flap between the two batches made the web build RE-LOAD the
    // scratch from its localStorage stash mid-session (which would restore
    // the STASHED content and reset the cursor to 0 — restoring a buffer
    // always starts a fresh Buffer at cursor 0, see `App::new`).
    //
    // `WindowEvent::Focused(false)` is the one live door a blur reaches —
    // and it calls exactly `App::autosave_flush` (`app.rs`'s `Focused(false)`
    // arm), which fans out to `stash_scratch_now` for a no-path scratch. That
    // function is a pure WRITE: it reads `self.active.buffer.text()` and writes it
    // OUT to the stash path; it never calls `crate::fs::active().read_*` or
    // reconstructs `self.active.buffer`. The ONLY place a stash is ever read back
    // INTO a buffer is `App::new` (a true process/page (re)launch) — never a
    // blur, never any other live-App path. This test pins that down: typing
    // "AAA", flushing (the blur trigger) as many times as a stress test's
    // spurious focus flapping might, then typing "BBB" must land the cursor
    // right after "AAA", not at 0.
    use crate::fs::InMemoryFs;
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(None, "/proj", Config::empty());
    for c in "AAA".chars() {
        app.active.buffer.insert_char(c);
    }
    assert_eq!(
        app.active.buffer.cursor_char(),
        3,
        "cursor sits after the typed AAA"
    );
    // Simulate the exact call the live `Focused(false)` arm makes — as many
    // times as a flappy test harness might re-fire it between dispatches.
    app.autosave_flush();
    app.autosave_flush();
    app.autosave_flush();
    assert_eq!(
        app.active.buffer.text(),
        "AAA",
        "a blur-driven flush never reloads content"
    );
    assert_eq!(
        app.active.buffer.cursor_char(),
        3,
        "a blur-driven flush never resets the cursor — only App::new restores"
    );
    // A later "dispatch batch" continues typing from exactly where it left off.
    for c in "BBB".chars() {
        app.active.buffer.insert_char(c);
    }
    assert_eq!(
        app.active.buffer.text(),
        "AAABBB",
        "BBB lands after AAA, not at position 0"
    );
    assert_eq!(app.active.buffer.cursor_char(), 6);
}

#[test]
fn scratch_restore_skips_empty_stash() {
    use crate::fs::{FileSystem, InMemoryFs};
    // An EMPTY stash restores nothing (plain scratch)… (each half owns its
    // FsGuard — the guard holds the process-wide FS lock, so they must not
    // overlap on one thread.)
    {
        let mem = InMemoryFs::new();
        let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
        mem.write(&crate::fs::scratch_stash_path(), b"").unwrap();
        let app = app_on(None, "/proj", Config::empty());
        assert!(
            app.active.buffer.text().is_empty(),
            "empty stash → plain scratch"
        );
    }
    // …and so does a MISSING one (fresh fake).
    {
        let _g = crate::fs::FsGuard::install(Arc::new(InMemoryFs::new()));
        let app = app_on(None, "/proj", Config::empty());
        assert!(
            app.active.buffer.text().is_empty(),
            "missing stash → plain scratch"
        );
    }
}

#[test]
fn autosave_writes_git_files_but_never_snapshots_them() {
    // LOCKED DECISION 4, both halves at the App seam: autosave still WRITES
    // a git-managed file (writing is not version-meddling), but records NO
    // awl snapshot for it — its timeline stays git log alone.
    use crate::fs::{FileSystem, InMemoryFs};
    let p = PathBuf::from("/repo/doc.md");
    let mem = InMemoryFs::new()
        .with_dir("/repo/.git")
        .with_file(&p, "v1\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(p.clone()), "/repo", Config::empty());
    app.active.buffer.set_text("v2\n");
    app.autosave_flush();
    assert_eq!(
        mem.read_to_string(&p).unwrap(),
        "v2\n",
        "autosave still WRITES a git-managed file"
    );
    assert!(app.notice.is_none(), "a clean write raises no notice");
    // The snapshot store never grew a log dir — the record gate held.
    let store = crate::fs::data_root().join("history");
    assert!(
        mem.read_dir(&store).map(|v| v.is_empty()).unwrap_or(true),
        "no awl snapshot log for a git-managed file"
    );
}

#[test]
fn scratch_stash_clobber_guard_holds_two_instance_writes() {
    // TWO-INSTANCE SAFETY: another awl (or anything) writes the stash after
    // this instance launched — the flush HOLDS (the external stash content
    // survives) and raises the same calm notice as the document guard.
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let stash = crate::fs::scratch_stash_path();
    let mut app = app_on(None, "/proj", Config::empty());
    mem.write(&stash, b"the other instance's dump\n").unwrap();
    app.active.buffer.set_text("mine\n");
    app.autosave_flush();
    assert_eq!(
        mem.read_to_string(&stash).unwrap(),
        "the other instance's dump\n",
        "the stash write is held — external content survives"
    );
    assert_eq!(
        app.notice.as_deref(),
        Some(CLOBBER_NOTICE),
        "the calm notice names the hold"
    );
}

#[test]
fn emptied_scratch_clears_the_stale_stash() {
    // The stash writes EVEN EMPTY text: emptying the restored scratch and
    // flushing must clear yesterday's dump, or a deliberately-emptied
    // scratch would resurrect on the next launch.
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let stash = crate::fs::scratch_stash_path();
    mem.write(&stash, b"yesterday's dump\n").unwrap();
    let mut app = app_on(None, "/proj", Config::empty());
    assert_eq!(
        app.active.buffer.text(),
        "yesterday's dump\n",
        "the stash restored"
    );
    app.active.buffer.set_text("");
    app.autosave_flush();
    assert_eq!(
        mem.read_to_string(&stash).unwrap(),
        "",
        "an emptied scratch clears the stale stash"
    );
    assert!(
        app.notice.is_none(),
        "our own restore is not an external edit"
    );
}

// ── ITEM 56 PHASE B: `Buffer::path()` IS THE SOLE, AUTHORITATIVE PATH ──
//
// `App.file` is gone entirely (there is no second field left to disagree
// with `Buffer::path()` — the type system now makes the "single truth" law
// structural, not just observed). These tests walk `self.active.buffer.
// path()` through every path-changing verb, proving it always reflects
// reality rather than merely compiling.

#[test]
fn path_law_across_a_plain_file_lifecycle_open_rename_duplicate_close_toggle() {
    use crate::fs::InMemoryFs;
    let a = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new()
        .with_file(&a, "alpha\n")
        .with_file(&b, "beta\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));

    // OPEN: the launch argument becomes the buffer's own path.
    let mut app = app_on(Some(a.clone()), "/proj", Config::empty());
    assert_eq!(app.active.buffer.path(), Some(a.as_path()), "open");

    // RENAME: the buffer follows the on-disk rename.
    app.rename_current_file("renamed.txt");
    let renamed = PathBuf::from("/proj/renamed.txt");
    assert_eq!(app.active.buffer.path(), Some(renamed.as_path()), "rename");

    // DUPLICATE: `load_path` switches the ACTIVE buffer to the new sibling.
    app.duplicate_current_file();
    let dup = PathBuf::from("/proj/renamed-2.txt");
    assert_eq!(app.active.buffer.path(), Some(dup.as_path()), "duplicate");

    // CLOSE-TOGGLE (C-x b): the 2-deep last-buffer history swaps back to the
    // pre-duplicate path, itself sourced from `Buffer::path()` (captured
    // before each park — see `load_path`/`duplicate_current_file`).
    app.last_buffer_toggle();
    assert_eq!(
        app.active.buffer.path(),
        Some(renamed.as_path()),
        "close-toggle restores the pre-duplicate path"
    );

    // OPEN a second, previously-untouched file: still tracks exactly.
    app.load_path(b.clone());
    assert_eq!(
        app.active.buffer.path(),
        Some(b.as_path()),
        "open a second file"
    );
}

#[test]
fn path_law_across_a_document_lifecycle_new_document_one_shot_name_no_rename_move() {
    // item 76: New document lands in the ACTIVE folder (never a separate
    // separate-notes-root jump); the first material save derives the filename EXACTLY
    // ONCE; a LATER edit to the first line never re-derives/renames it; Move
    // is relative to the active folder.
    use crate::fs::InMemoryFs;
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(None, "/proj", Config::empty());

    // NEW DOCUMENT: no path until it has content, and it lands in the
    // CURRENT active folder — no root jump.
    app.new_document();
    assert_eq!(
        app.active.buffer.path(),
        None,
        "a fresh document is unnamed"
    );
    assert_eq!(
        app.project_location.root,
        PathBuf::from("/proj"),
        "no root jump on Cmd-N"
    );

    // FIRST (MATERIAL) SAVE (auto-name from the first line): the buffer gains
    // a derived path, under the ACTIVE folder, with no separate field to keep
    // in step.
    app.active.buffer.insert_text("first draft\n");
    app.autosave_note();
    let named = app.active.buffer.path().map(|p| p.to_path_buf());
    assert!(named.is_some(), "auto-name gave the document a path");
    assert!(
        named.as_deref().unwrap().starts_with("/proj"),
        "named under the ACTIVE folder, not a separate notes-root concept: {named:?}"
    );
    assert!(
        !app.active.buffer.is_unnamed_fresh(),
        "one-shot: named once, ordinary now"
    );

    // A LATER title edit NEVER renames (the old live-rename-to-title behavior
    // is retired): editing the FIRST LINE again leaves the path untouched.
    let end_of_first_line = app.active.buffer.line_col_to_char(0, usize::MAX);
    app.active.buffer.set_cursor(end_of_first_line);
    app.active.buffer.insert_text(" retitled");
    app.autosave_note(); // is_unnamed_fresh() is false: this is now a no-op
    let after_title_edit = app.active.buffer.path().map(|p| p.to_path_buf());
    assert_eq!(
        after_title_edit, named,
        "later title edits never rename (one-shot naming)"
    );

    // MOVE (C-x m): re-points the buffer to the destination folder (relative
    // to the ACTIVE folder), keeping the filename.
    let before_move = after_title_edit.clone().unwrap();
    app.move_current_file("sub");
    let moved = app.active.buffer.path().map(|p| p.to_path_buf()).unwrap();
    assert_eq!(
        moved.file_name(),
        before_move.file_name(),
        "the filename survives the move"
    );
    assert!(
        moved.starts_with("/proj/sub"),
        "moved under the active folder's dest: {moved:?}"
    );
}

// ── ITEM 94 — RANGE ROWS at the LIVE APP seam (apply / persist accounting) ────

/// Build a Settings overlay with its rail column, selected on the Zoom row, and
/// return it plus that row's `items` index — exactly `overlay::build`'s shape.
fn settings_overlay_with_rail(app: &App) -> (crate::overlay::OverlayState, usize) {
    let vals = crate::settings::SettingsValues::gather(
        &app.config,
        &app.project_location.root,
        app.zoom,
        crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    );
    let mut ov = crate::overlay::OverlayState::new(
        crate::overlay::OverlayKind::Settings,
        crate::settings::visible_names(),
        vec![],
        vec![],
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    let zi = ov
        .items
        .iter()
        .position(|&i| ov.rows[i].accept == "Zoom")
        .unwrap();
    ov.selected = zi;
    (ov, zi)
}

/// ONE PERSIST PER DRAG — the item's own accounting rule, proved at the live App
/// seam: a pointer scrub applies on EVERY resolved step (the value and the row's
/// own cell move each move) but writes config ZERO times, and the RELEASE writes
/// exactly once, with the settled value. The pending debounced write the live
/// apply armed is cancelled by that same release, so the gesture cannot leave a
/// second write trailing behind it.
#[test]
fn a_pointer_scrub_applies_every_step_and_persists_exactly_once_on_release() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let _g = crate::testlock::serial();
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let mut app = app_on(None, "/proj", cfg);
    let spec = crate::settings::range_spec(crate::settings::SettingId::Zoom).unwrap();
    app.zoom = spec.default;
    let (ov, zi) = settings_overlay_with_rail(&app);
    app.workspace_state.install_overlay_for_test(ov);

    // Arm the scrub exactly as a rail press does (the press-time track scale is
    // snapshotted; here it is supplied directly, since the hit-test needs a GPU).
    let (x0, x1) = (100.0f32, 300.0f32);
    app.range_drag = Some(crate::app::input::RangeDrag {
        id: crate::settings::SettingId::Zoom,
        item: zi,
        x0,
        x1,
    });

    let config_text = |mem: &InMemoryFs| -> String {
        mem.read_to_string(std::path::Path::new("/cfg/config.toml"))
            .unwrap_or_default()
    };
    // EVERY resolved step of the scrub applies live…
    let mut applied = Vec::new();
    for i in 0..=40u32 {
        let px = x0 + (x1 - x0) * (i as f32 / 40.0);
        app.cursor_px = (px, 0.0);
        app.on_range_drag();
        let want = spec.value_at_frac(crate::render::rail_frac_at(px, x0, x1));
        assert_eq!(
            app.zoom.to_bits(),
            want.to_bits(),
            "step {i} applied a parallel value"
        );
        applied.push(app.zoom);
        // …and the row's own readout + thumb track it in the same move.
        let ov = app.workspace_state.overlay().unwrap();
        assert_eq!(ov.item_bindings()[zi], spec.format(app.zoom));
        assert_eq!(ov.range_of_item(zi).unwrap().step, spec.step_of(app.zoom));
        // …while NOTHING is written to disk mid-gesture.
        assert!(
            !config_text(&mem).contains("zoom"),
            "step {i}: a drag must not persist (config: {:?})",
            config_text(&mem)
        );
    }
    assert!(
        applied.iter().any(|v| *v != spec.default),
        "the scrub genuinely moved the value"
    );
    let settled = app.zoom;

    // THE RELEASE: exactly one write, of the settled value, and the drag is over.
    app.end_range_drag();
    let written = config_text(&mem);
    assert_eq!(
        written
            .lines()
            .filter(|l| l.trim_start().starts_with("zoom"))
            .count(),
        1,
        "the whole gesture writes ONE zoom line, not one per step: {written:?}"
    );
    assert!(
        written.contains(&format!("zoom = {}", spec.persist_value(settled))),
        "the persisted value is the settled one: {written:?}"
    );
    assert!(app.range_drag.is_none(), "the release ends the gesture");
    assert!(
        app.zoom_persist_at.is_none(),
        "the release supersedes (and cancels) the debounced write the live apply armed"
    );
    assert_eq!(
        app.config.zoom,
        Some(settled),
        "the in-memory config mirrors the write"
    );
}

/// THE PAUSED SCRUB — the same accounting rule, but with REAL TIME PASSING inside
/// the gesture. The test above never advances the clock, so it only ever proved the
/// UNINTERRUPTED case; this one drives the actual `about_to_wait` scheduling body
/// under a [`crate::clock::VirtualClock`] (the which-key frame-loop pattern) while
/// the pointer is held STILL for several times the sticky-zoom debounce window —
/// an entirely ordinary "pause to look at the number" gesture.
///
/// The class it closes: the live apply goes through `set_zoom` -> `mark_zoom_dirty`,
/// which arms the SAME quiet-window debounce ⌘±/⌘-wheel use, and that debounce infers
/// "the gesture ended" from silence. A held-still drag IS silence, so before
/// `App::zoom_persist_held` gated it, half a second of pausing wrote the mid-gesture
/// value to config — a value the user never released on. Nothing may be written until
/// the button comes up, and then exactly once, with the released value.
#[test]
fn a_paused_mid_drag_persists_nothing_until_the_release() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let _g = crate::testlock::serial();
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let mut app = app_on(None, "/proj", cfg);
    // VIRTUAL TIME: the App's own clock, so `about_to_wait_impl` sees the pause as a
    // live idle loop would — deterministically, with no sleeping.
    let clock = crate::clock::VirtualClock::new();
    app.set_clock(Box::new(clock.clone()));

    let spec = crate::settings::range_spec(crate::settings::SettingId::Zoom).unwrap();
    app.zoom = spec.default;
    let (ov, zi) = settings_overlay_with_rail(&app);
    app.workspace_state.install_overlay_for_test(ov);
    let (x0, x1) = (100.0f32, 300.0f32);
    app.range_drag = Some(crate::app::input::RangeDrag {
        id: crate::settings::SettingId::Zoom,
        item: zi,
        x0,
        x1,
    });
    let config_text = |mem: &InMemoryFs| -> String {
        mem.read_to_string(std::path::Path::new("/cfg/config.toml"))
            .unwrap_or_default()
    };
    // The LIVE `zoom = …` lines actually in the file (the commented template lines the
    // config writer emits are not settings) — the compact oracle every step below reads.
    let zoom_lines = |mem: &InMemoryFs| -> Vec<String> {
        config_text(mem)
            .lines()
            .filter(|l| l.trim_start().starts_with("zoom"))
            .map(|l| l.trim().to_string())
            .collect()
    };

    // PRESS + a first move to a low value the user will NOT release on.
    app.cursor_px = (x0 + (x1 - x0) * 0.15, 0.0);
    app.on_range_drag();
    let paused = app.zoom;
    assert_ne!(
        paused, spec.default,
        "the scrub genuinely moved off the default"
    );
    assert!(
        app.zoom_persist_at.is_some(),
        "the live apply still ARMS the in-flight stamp (the zoom gesture is live — the \
         hold-⌘ peek suppression reads exactly this); it is the WRITE that is held"
    );

    // NOW THE USER PAUSES, pointer down, for 4x the debounce window — stepping the
    // REAL scheduling body once per 100ms frame, exactly as an idle winit loop does.
    let win_ms = ZOOM_PERSIST_DEBOUNCE.as_millis() as u64;
    let sched = RecordingScheduler::new();
    let mut wakes = Vec::new();
    for frame in 1..=(win_ms * 4 / 100) {
        clock.advance_ms(100);
        sched.begin_step();
        app.step_scheduling(&sched);
        let t = frame * 100;
        assert_eq!(
            zoom_lines(&mem),
            Vec::<String>::new(),
            "t={t}ms into a held pause (debounce window {win_ms}ms): a mid-gesture value \
             must NEVER reach the config — only the release writes"
        );
        assert_eq!(
            app.zoom.to_bits(),
            paused.to_bits(),
            "t={t}ms: the value itself is untouched"
        );
        assert!(
            app.range_drag.is_some(),
            "t={t}ms: the gesture is still in flight"
        );
        wakes.push((t, sched.scheduled_this_step()));
    }
    // HELD MEANS INERT: no write AND no wake armed — the loop still falls quiet
    // (DESIGN §6), rather than spinning on a deadline it will never honour.
    assert!(
        wakes.iter().all(|(_, cf)| cf.is_none()),
        "a held debounce must arm no WaitUntil: {wakes:?}"
    );

    // The pause ends: the user drags on to a genuinely different value and releases.
    app.cursor_px = (x0 + (x1 - x0) * 0.95, 0.0);
    app.on_range_drag();
    let settled = app.zoom;
    assert_ne!(
        settled.to_bits(),
        paused.to_bits(),
        "the release value differs from the paused one"
    );
    assert_eq!(
        zoom_lines(&mem),
        Vec::<String>::new(),
        "still nothing written before the button comes up"
    );
    app.end_range_drag();

    // EXACTLY ONE write, of the RELEASED value — never the paused one.
    assert_eq!(
        zoom_lines(&mem),
        vec![format!("zoom = {}", spec.persist_value(settled))],
        "the whole paused gesture writes ONE line, and it is the RELEASED value \
         (the paused one would have been `zoom = {}`)",
        spec.persist_value(paused)
    );
    let written = config_text(&mem);

    // …and nothing trails behind it: quiet time after the release adds no second write.
    for _ in 0..(win_ms * 2 / 100) {
        clock.advance_ms(100);
        sched.begin_step();
        app.step_scheduling(&sched);
    }
    assert_eq!(
        config_text(&mem),
        written,
        "the settled release leaves no debounced write trailing behind it"
    );
}

/// THE KEYBOARD'S OWN PERSISTENCE PATH is the DISCRETE one: each authored step
/// writes once, immediately (the same "write on every discrete commit" rule a
/// Toggle follows) — no debounce left pending afterwards.
#[test]
fn a_keyboard_range_step_persists_discretely_through_the_live_door() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let _g = crate::testlock::serial();
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let mut app = app_on(None, "/proj", cfg);
    let spec = crate::settings::range_spec(crate::settings::SettingId::Zoom).unwrap();
    let (ov, zi) = settings_overlay_with_rail(&app);
    app.workspace_state.install_overlay_for_test(ov);

    // The core already stepped the value (see the `actions` half of this pair);
    // the App owns the live tail. Drive the EXACT door `Effect::SettingRangeStep`
    // resolves to at the `app/apply.rs` seam.
    app.zoom = spec.stepped(spec.default, 1);
    app.setting_range_step("zoom");
    let written = mem
        .read_to_string(std::path::Path::new("/cfg/config.toml"))
        .unwrap();
    assert!(
        written.contains(&format!("zoom = {}", spec.persist_value(app.zoom))),
        "a discrete step persists at once: {written:?}"
    );
    assert!(
        app.zoom_persist_at.is_none(),
        "nothing left pending after a discrete write"
    );
    // The still-open menu was refreshed from the LIVE values through the one
    // owner, so its cell and its thumb both show the stepped value.
    let ov = app.workspace_state.overlay().unwrap();
    assert_eq!(ov.item_bindings()[zi], spec.format(app.zoom));
    assert_eq!(ov.range_of_item(zi).unwrap().step, spec.step_of(app.zoom));
}

/// THE LIVE-APPLY DOOR IS THE SETTING'S OWN OWNER: `range_apply_live` moves the
/// live value through `set_zoom` (which re-clamps through the same spec), so a
/// pointer can never install an off-grid or out-of-band value even if handed one.
#[test]
fn the_live_range_apply_door_clamps_through_the_settings_own_owner() {
    use crate::fs::InMemoryFs;
    let _g2 = crate::fs::FsGuard::install(Arc::new(InMemoryFs::new()));
    let _g = crate::testlock::serial();
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let mut app = app_on(None, "/proj", cfg);
    let spec = crate::settings::range_spec(crate::settings::SettingId::Zoom).unwrap();
    for raw in [-4.0f32, 0.0, 0.53, 1.0, 2.97, 99.0] {
        app.range_apply_live(crate::settings::SettingId::Zoom, raw);
        assert_eq!(
            app.zoom.to_bits(),
            spec.quantize(raw).to_bits(),
            "raw {raw}"
        );
        assert!((spec.min..=spec.max).contains(&app.zoom));
    }
}

/// THE APP-SIDE PORT SWEEP: EVERY row the corpus marks `SettingKind::Range` —
/// enumerated off `settings::visible_rows()`, never hand-copied — moves its own
/// LIVE value through `App::range_apply_live` (the pointer path's door) and
/// persists it through `App::range_persist` (the release/keyboard door). A future
/// range setting wired into the spec map but not into these two doors would draw
/// a rail that scrubs nothing, or scrub something that never sticks; it fails here.
#[test]
fn every_range_row_applies_and_persists_through_the_app_side_doors() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g2 = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let _g = crate::testlock::serial();
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let mut app = app_on(None, "/proj", cfg);
    let initial_measure = crate::page::measure();

    let range_rows: Vec<crate::settings::SettingRow> = crate::settings::visible_rows()
        .into_iter()
        .filter(|r| r.kind == crate::settings::SettingKind::Range)
        .copied()
        .collect();
    assert!(!range_rows.is_empty(), "the range roster must not be empty");
    for row in range_rows {
        let spec = crate::settings::range_spec(row.id).unwrap();
        let key = crate::settings::value_key(row.id).unwrap();
        // A value the setting is definitely NOT sitting on.
        let target = spec.stepped(spec.default, 3);
        assert_ne!(
            target, spec.default,
            "{}: pick a genuinely different value",
            row.name
        );

        app.range_apply_live(row.id, target);
        let values = crate::settings::SettingsValues::gather(
            &app.config,
            &app.project_location.root,
            app.zoom,
            crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
        );
        assert_eq!(
            crate::settings::range_value(row.id, &values),
            Some(target),
            "{}: the live-apply door must move the value the readout reads",
            row.name
        );
        app.range_persist(key);
        let written = mem
            .read_to_string(std::path::Path::new("/cfg/config.toml"))
            .unwrap_or_default();
        assert!(
            written.contains(&format!("{key} = {}", spec.persist_value(target))),
            "{}: the persist door must write {key}: {written:?}",
            row.name
        );
    }
    crate::page::set_measure(initial_measure);
}
