use super::*;

// ── The HISTORY TIMELINE live preview (App-level, InMemoryFs seam) ───────
//
// The preview is DERIVED at ViewState-build time — these tests pin the
// resolver (`history_preview_text`) and the close contract
// (`history_overlay_closed`) directly, buffer untouched throughout.

/// Seed two history versions for `p` and open the History overlay on `app`,
/// exactly as the OpenHistory gather builds it (timeline_rows → new_history).
fn open_history_overlay(app: &mut App, p: &std::path::Path) {
    let rows =
        crate::history::timeline_rows(p, &app.active.buffer.text(), crate::history::now_millis());
    app.workspace_state
        .install_overlay_for_test(crate::overlay::OverlayState::new_history(rows, None, None));
}

#[test]
fn history_preview_resolves_without_touching_buffer() {
    use crate::fs::{FileSystem, InMemoryFs};
    let p = PathBuf::from("/notes/draft.md");
    let mem = InMemoryFs::new().with_file(&p, "the second draft entirely\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    crate::history::record(&p, "the first draft wording\n", &Config::empty());
    crate::history::record(&p, "the second draft entirely\n", &Config::empty());
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());
    let version_before = app.active.buffer.version();
    open_history_overlay(&mut app, &p);
    // DIFF-AS-PREVIEW: row 0 (newest, identical to the buffer) previews a
    // folds-only transcript; row 1 (older) previews a transcript CARRYING the
    // change marks (the reworded paragraph shows surgery / a rewrite).
    let newest = app.history_preview_text().expect("row 0 previews");
    assert!(
        newest.starts_with("# Comparing with "),
        "a titled transcript: {newest}"
    );
    assert!(
        !newest.contains("~~") && !newest.contains("=="),
        "identical content diffs to no marks: {newest}"
    );
    app.workspace_state.overlay_mut().unwrap().move_sel(1);
    let older = app.history_preview_text().expect("row 1 previews");
    assert!(
        older.contains("~~") || older.contains("=="),
        "arrowing to the older version previews ITS diff (marks present): {older}"
    );
    // The BUFFER was never touched: content, version, and undo all intact.
    assert_eq!(app.active.buffer.text(), "the second draft entirely\n");
    assert_eq!(
        app.active.buffer.version(),
        version_before,
        "no version bump"
    );
    // The per-id CACHE serves a repeat without re-reading the store: blow the
    // store away and the highlighted row still previews from the cache.
    let hist_dir = crate::fs::data_root().join("history");
    for entry in mem.read_dir(&hist_dir).unwrap_or_default() {
        let _ = mem.rename(&entry.path, std::path::Path::new("/gone"));
    }
    assert_eq!(
        app.history_preview_text().as_deref(),
        Some(older.as_str()),
        "a repeat on the same id is a cache hit"
    );
}

#[test]
fn preview_cache_invalidates_on_selection_move() {
    use crate::fs::InMemoryFs;
    let p = PathBuf::from("/notes/draft.md");
    let mem = InMemoryFs::new().with_file(&p, "v2\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    crate::history::record(&p, "v1\n", &Config::empty());
    crate::history::record(&p, "v2\n", &Config::empty());
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());
    open_history_overlay(&mut app, &p);
    assert!(app.history_preview_text().is_some());
    let cached_id = app
        .active
        .extra
        .history_preview
        .as_ref()
        .map(|(id, _)| id.clone());
    // Moving the selection to another row (a different id) re-renders: the
    // cache is keyed by id, never by "an overlay is open". (The selection
    // move also resets the diff panel scroll — the transcript changed.)
    app.workspace_state.overlay_mut().unwrap().diff_scroll = 7;
    app.workspace_state.overlay_mut().unwrap().move_sel(1);
    assert_eq!(
        app.workspace_state.overlay().unwrap().diff_scroll,
        0,
        "a new version tops the diff out"
    );
    assert!(app.history_preview_text().is_some());
    assert_ne!(
        app.active
            .extra
            .history_preview
            .as_ref()
            .map(|(id, _)| id.clone()),
        cached_id,
        "the cache now holds the newly highlighted id"
    );
}

#[test]
fn history_close_without_accept_restores_scroll_and_drops_preview() {
    use crate::fs::InMemoryFs;
    let _g = crate::fs::FsGuard::install(Arc::new(InMemoryFs::new()));
    let mut app = app_on(None, "/proj", Config::empty());
    // A shorter previewed version clamped the scroll while the picker was
    // open; the close-without-accept restores the saved scroll EXACTLY
    // ("Esc = back to now") and puts the preview down.
    let before = crate::render::ScrollPos { row: 42, px_q: 17 };
    app.active.extra.history_scroll_before = Some(before);
    app.active.extra.scroll = crate::render::ScrollPos { row: 3, px_q: 29 };
    app.active.extra.history_preview = Some(("100".into(), "old\n".into()));
    app.history_overlay_closed(false);
    assert_eq!(
        app.active.extra.scroll, before,
        "Esc restores the pre-open scroll"
    );
    assert!(app.active.extra.history_scroll_before.is_none());
    assert!(
        app.active.extra.history_preview.is_none(),
        "the preview is dropped"
    );
    // A real ACCEPT keeps the current viewport (the restored version owns
    // it) — the saved scroll is discarded, the preview still dropped.
    app.active.extra.history_scroll_before = Some(before);
    let accepted = crate::render::ScrollPos { row: 3, px_q: 29 };
    app.active.extra.scroll = accepted;
    app.active.extra.history_preview = Some(("100".into(), "old\n".into()));
    app.history_overlay_closed(true);
    assert_eq!(
        app.active.extra.scroll, accepted,
        "an accept never yanks the viewport"
    );
    assert!(app.active.extra.history_scroll_before.is_none());
    assert!(app.active.extra.history_preview.is_none());
}

// ── DIFF-AS-PREVIEW — the History picker's writer's-diff preview ────────
//
// The diff IS the picker's live preview now (the takeover Compare view is
// retired). These pin the transcript's shape and the read-only invariants on
// the PREVIEW path (buffer / version / undo untouched — the successor of
// the old diff_view_gate suite). The render is SYNCHRONOUS: the round's
// release perf probe measured ~1-2 ms per diff at contract-document scale (the diff
// folds unchanged regions, so the transcript stays tiny), so no per-arrow
// debounce is warranted; the old settle machinery was cut.

#[test]
fn diff_preview_renders_marked_up_transcript_without_touching_buffer() {
    use crate::fs::InMemoryFs;
    let p = PathBuf::from("/notes/draft.md");
    // Current buffer keeps the first paragraph, drops the second, adds a third.
    let now = "Keep this opening paragraph exactly as it was.\n\nAn entirely fresh third paragraph appears here now.\n";
    let mem = InMemoryFs::new().with_file(&p, now);
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    // Seed an older version (the one the highlighted row compares against).
    let old = "Keep this opening paragraph exactly as it was.\n\nDrop this whole second paragraph entirely please.\n";
    crate::history::record(&p, old, &Config::empty());
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());
    let version_before = app.active.buffer.version();
    let text_before = app.active.buffer.text();
    open_history_overlay(&mut app, &p);
    let transcript = app
        .history_preview_text()
        .expect("the diff preview is live");
    // The transcript speaks awl's diff vocabulary: a struck deletion (REAL
    // `~~` markdown) AND a highlight-washed insertion (`==`), under a title
    // heading naming the compared row.
    assert!(transcript.starts_with("# Comparing with "), "{transcript}");
    assert!(transcript.contains("~~"), "a struck deletion: {transcript}");
    assert!(
        transcript.contains("=="),
        "a washed insertion: {transcript}"
    );
    // The BUFFER was never touched — content, version, undo all intact.
    assert_eq!(
        app.active.buffer.text(),
        text_before,
        "preview never mutates the buffer"
    );
    assert_eq!(
        app.active.buffer.version(),
        version_before,
        "no version bump"
    );
    app.active.buffer.undo();
    assert_eq!(
        app.active.buffer.text(),
        text_before,
        "undo after preview is inert"
    );
}

#[test]
fn diff_preview_read_only_law_typing_edits_the_query_never_the_buffer() {
    // THE READ-ONLY LAW on the preview path (the successor of the retired
    // diff_view_gate suite): while the History picker's diff preview is up,
    // the overlay's MODALITY is the law — every key routes through
    // `overlay_intercept`, so typing filters the query, Tab shifts focus,
    // PgUp/PgDn scroll the panel, and NOTHING reaches the rope.
    use crate::fs::InMemoryFs;
    let p = PathBuf::from("/notes/draft.md");
    let mem = InMemoryFs::new().with_file(&p, "current words\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    crate::history::record(&p, "older words\n", &Config::empty());
    crate::history::record(&p, "current words\n", &Config::empty());
    let mut app = app_on(Some(p.clone()), "/notes", Config::empty());
    let version_before = app.active.buffer.version();
    open_history_overlay(&mut app, &p);
    assert!(app.history_preview_text().is_some(), "preview live");
    // Drive the modal intercept exactly as a keypress would (the core seam).
    for act in [
        Action::InsertChar('z'),
        Action::InsertTab,
        Action::PageScrollDown,
        Action::NextLine,
        Action::DeleteBackward,
    ] {
        app.apply_transition_for_test(&act);
    }
    assert_eq!(
        app.active.buffer.text(),
        "current words\n",
        "the rope never changed"
    );
    assert_eq!(
        app.active.buffer.version(),
        version_before,
        "no version bump"
    );
    // Esc from panel focus returns to the LIST; a second Esc closes — two
    // Escs total from panel to home, and the buffer text is back untouched.
    app.apply_transition_for_test(&Action::InsertTab); // focus the panel
    assert!(app.workspace_state.overlay().unwrap().detail_focus);
    app.apply_transition_for_test(&Action::Cancel);
    assert!(
        app.workspace_state.overlay_open(),
        "first Esc: back to LIST focus, not home"
    );
    assert!(!app.workspace_state.overlay().unwrap().detail_focus);
    app.apply_transition_for_test(&Action::Cancel);
    assert!(
        !app.workspace_state.overlay_open(),
        "second Esc closes the picker"
    );
    assert_eq!(
        app.active.buffer.version(),
        version_before,
        "back to now exactly"
    );
}

#[test]
fn scratch_buffer_lists_its_stash_history() {
    use crate::fs::InMemoryFs;
    let _g = crate::fs::FsGuard::install(Arc::new(InMemoryFs::new()));
    // The persistent scratch stashes (autosave engine) — recording history
    // under its stash path — and the timeline gather's shared source_path
    // fallback finds it, so the no-path scratch has a summonable timeline.
    let mut app = app_on(None, "/proj", Config::empty());
    app.active.buffer.set_text("scratch thoughts\n");
    app.autosave_flush();
    let key = crate::history::source_path(
        app.active.buffer.path(),
        app.active.buffer.is_unnamed_fresh(),
    )
    .expect("the true scratch keys under its stash");
    assert_eq!(key, crate::fs::scratch_stash_path());
    let rows = crate::history::timeline_rows(
        &key,
        &app.active.buffer.text(),
        crate::history::now_millis(),
    );
    assert!(!rows.is_empty(), "the scratch stash has a timeline");
    // And the preview resolver rides the same key: the newest row previews
    // the stashed content.
    app.workspace_state
        .install_overlay_for_test(crate::overlay::OverlayState::new_history(rows, None, None));
    // DIFF-AS-PREVIEW: the stash's newest snapshot is identical to the
    // buffer, so the preview is a titled folds-only transcript.
    let transcript = app.history_preview_text().expect("the stash previews");
    assert!(transcript.starts_with("# Comparing with "), "{transcript}");
}

#[test]
fn notes_keep_their_own_autosave() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(None, "/proj", Config::empty());
    app.active.buffer.start_fresh_doc(PathBuf::from("/mynotes"));
    app.active.buffer.set_text("a note in flight\n");
    app.autosave_flush();
    // The DOC engine leaves notes to their own 400ms flow (flush_note): no
    // scratch stash, no note file written by this door.
    assert!(
        mem.read(&crate::fs::scratch_stash_path()).is_err(),
        "a note is never stashed as scratch"
    );
    assert!(
        mem.read_dir(std::path::Path::new("/mynotes"))
            .map(|v| v.is_empty())
            .unwrap_or(true),
        "autosave_flush does not write note files"
    );
}
