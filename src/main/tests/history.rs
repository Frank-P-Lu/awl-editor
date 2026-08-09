use super::super::*;
use super::{keyspec, replay_keys};

fn with_seeded_history(body: impl FnOnce(PathBuf)) {
    use std::sync::Arc;
    let p = PathBuf::from("/notes/draft.md");
    let mem = crate::fs::InMemoryFs::new().with_file(&p, "v2\n");
    crate::fs::with_fs(Arc::new(mem), || {
        crate::history::record(&p, "v1\n", &Config::empty());
        crate::history::record(&p, "v2\n", &Config::empty());
        body(p.clone());
    });
}

#[test]
fn comparison_preview_for_resolves_selected_row() {
    // DIFF-AS-PREVIEW: the capture-side preview resolver — the still-open
    // History overlay's highlighted row resolves to (id, TRANSCRIPT, counts):
    // the writer's diff of the current buffer vs that version, exactly what
    // the live App renders (the shared `history::diff_preview` owner). The
    // buffer here is "v2\n", so row 0 (v2, identical) is a titled folds-only
    // transcript with NO change marks; row 1 (v1, older) carries them.
    with_seeded_history(|p| {
        let buffer = Buffer::from_file(&p);
        let rows = crate::history::timeline_rows(&p, &buffer.text(), crate::history::now_millis());
        assert_eq!(rows.len(), 2, "two seeded versions");
        let mut ov = crate::overlay::OverlayState::new_history(rows, None, None);
        let (id, transcript, _counts) =
            comparison_preview_for(&ov, &buffer).expect("the newest row resolves");
        assert!(
            transcript.starts_with("# Comparing with "),
            "a titled diff transcript: {transcript}"
        );
        assert!(
            !transcript.contains("~~") && !transcript.contains("=="),
            "row 0 is identical to the buffer → no change marks: {transcript}"
        );
        assert_eq!(Some(id.as_str()), ov.selected_history_id());
        ov.move_sel(1);
        let (_, older, _) = comparison_preview_for(&ov, &buffer).expect("row 1 resolves");
        assert!(
            older.contains("~~") || older.contains("=="),
            "the highlighted row's diff carries change marks: {older}"
        );
        // A non-history overlay never previews.
        let goto = crate::overlay::OverlayState::new(
            crate::overlay::OverlayKind::Goto,
            vec!["a.md".into()],
            Vec::new(),
            Vec::new(),
        );
        assert!(comparison_preview_for(&goto, &buffer).is_none());
    });
}

#[test]
fn replay_history_esc_leaves_buffer_text_exact() {
    // The Esc-reverts-exactly proof at the replay seam: summon the timeline
    // (Cmd-S-h), arrow to the older version (C-n), Esc — the overlay is gone,
    // NOTHING was accepted, and the buffer text is byte-for-byte what it was
    // (the preview is a ViewState-level derivation; the buffer never moved).
    with_seeded_history(|p| {
        let mut buffer = Buffer::from_file(&p);
        let before = buffer.text();
        let keys = keyspec::parse_keys("Cmd-S-h C-n Esc").unwrap();
        let root = PathBuf::from("/notes");
        let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(res.journey.card().is_none(), "Esc closed the timeline");
        assert!(res.accept.is_none(), "nothing was accepted");
        assert_eq!(buffer.text(), before, "Esc leaves the buffer text exact");
    });
}

#[test]
fn replay_history_enter_restores_undoably() {
    // Restore lives behind the ALTERNATE ACCEPT (⇧↵) — bare `RET`
    // only opens the comparison now, so the restoring chord is `S-RET`.
    with_seeded_history(|p| {
        let mut buffer = Buffer::from_file(&p);
        let keys = keyspec::parse_keys("Cmd-S-h C-n S-RET").unwrap();
        let root = PathBuf::from("/notes");
        let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        let (kind, id) = res
            .accept
            .expect("Shift+Enter accepts the highlighted version");
        assert_eq!(kind, crate::overlay::OverlayKind::History);
        // The capture arm's restore (same shared source_path + load + set_text).
        let path = crate::history::source_path(buffer.path(), buffer.is_unnamed_fresh())
            .expect("a pathed buffer keys under itself");
        let content = crate::history::load(&path, &id).expect("the id round-trips");
        buffer.set_text(&content);
        assert_eq!(buffer.text(), "v1\n", "the older version restored");
        // UNDO through the real keymap: one sealed edit, back to now.
        let undo = keyspec::parse_keys("s-z").unwrap();
        replay_keys(&mut buffer, &undo, &[], &root, None, &Config::empty(), None);
        assert_eq!(buffer.text(), "v2\n", "the restore is one undoable edit");
    });
}
