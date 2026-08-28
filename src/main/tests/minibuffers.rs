use super::super::*;
use super::{keyspec, replay_keys};

#[test]
fn replay_keys_drives_the_rename_minibuffer_prompt_and_sidecar_reflects_typing() {
    // Cmd-P → "rename" → Enter opens the Rename overlay pre-filled with the
    // current filename, its STEM pre-selected (item 510's file-manager
    // convention: the first keystroke would REPLACE "old", not extend it —
    // see the dedicated type-over-selection law below). Right COLLAPSES that
    // selection to ITS OWN end — right before the extension, not the very
    // end of the text (a plain caret motion, not a replace) — and typing MORE
    // characters from there extends the STEM live — all through the shared
    // core, so both the overlay STATE and its sidecar-facing `foot_hint()`
    // (the same seam the Keybindings capture prompt rides) reflect the
    // in-progress edit with zero live App involved.
    let mut buffer = Buffer::scratch();
    buffer.set_path(PathBuf::from("/proj/old.md"));
    let keys = keyspec::parse_keys("s-p r e n a m e RET Right 2").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .journey
        .card()
        .expect("Rename note… opens the minibuffer overlay");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Rename);
    assert_eq!(
        ov.accepts(),
        vec!["old2.md"],
        "collapsing the seeded selection first lands the caret right after \
         the stem, before the extension — typing there extends the stem, \
         never the extension"
    );
    assert_eq!(
        ov.foot_hint(),
        "rename to: old2.md   Enter commit   Esc cancel",
        "the live prompt is sidecar-visible via the same foot_hint seam Keybindings uses"
    );
}

/// REQUIRED LAW (2/2): typing right after Rename opens REPLACES the seeded
/// STEM selection outright — proving the selection this module arms is not
/// decorative. `report.md` -> stem `"report"` selected -> typing "q3-summary"
/// replaces it, and the untouched extension survives verbatim.
#[test]
fn replay_keys_typing_over_the_seeded_rename_selection_replaces_the_stem() {
    let mut buffer = Buffer::scratch();
    buffer.set_path(PathBuf::from("/proj/report.md"));
    let keys = keyspec::parse_keys("s-p r e n a m e RET q 3 - s u m m a r y").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .journey
        .card()
        .expect("Rename note… opens the minibuffer overlay");
    assert_eq!(
        ov.accepts(),
        vec!["q3-summary.md"],
        "the FIRST keystroke deletes the selected stem \"report\" rather than \
         appending after it — the typed text plus the ORIGINAL extension, \
         untouched"
    );
}

#[test]
fn replay_keys_rename_minibuffer_esc_cancels_with_no_overlay_left() {
    let mut buffer = Buffer::scratch();
    buffer.set_path(PathBuf::from("/proj/old.md"));
    let keys = keyspec::parse_keys("s-p r e n a m e RET x Esc").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "Esc closes the minibuffer outright, no breadcrumb pop"
    );
    assert_eq!(
        buffer.path(),
        Some(std::path::Path::new("/proj/old.md")),
        "no disk rename happened"
    );
}

#[test]
fn replay_keys_rename_minibuffer_does_not_open_on_a_pathless_buffer() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p r e n a m e RET").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "nothing to rename on a pathless buffer"
    );
}

#[test]
fn replay_keys_drives_the_keep_version_minibuffer_prompt_and_sidecar_reflects_typing() {
    // Cmd-P → "keep" → Enter opens the naming minibuffer (empty — a fresh
    // point has no old name); typing builds the optional name live — all
    // through the shared core, so both the overlay STATE and its
    // sidecar-facing `foot_hint()` (the same seam Rename/InsertLink ride)
    // reflect the in-progress edit with zero live App involved.
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e e p RET d r a f t").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .journey
        .card()
        .expect("Keep version… opens the naming minibuffer");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::KeepName);
    assert_eq!(
        ov.accepts(),
        vec!["draft"],
        "typing builds the name from empty"
    );
    assert_eq!(
        ov.foot_hint(),
        "name this version: draft   Enter keep   Esc cancel",
        "the live prompt is sidecar-visible via the same foot_hint seam Rename uses"
    );
}

#[test]
fn replay_keys_keep_version_minibuffer_esc_pops_back_to_the_palette() {
    // The minibuffer's Cancel arm routes through the ONE lifecycle
    // door (`Journey::cancel`) instead of a hand-rolled `dismiss()` that always
    // dropped to the editor regardless of what was parked. Reached via the
    // palette (like `replay_keys_palette_theme_esc_pops_back_to_palette`), Esc
    // now pops back to the Command palette exactly like every other
    // palette-launched picker — nothing is kept.
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e e p RET x Esc").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .journey
        .card()
        .expect("Esc pops back to the palette, not the buffer");
    assert_eq!(
        ov.kind,
        crate::overlay::OverlayKind::Command,
        "back at the command palette, consistent with every other palette-launched picker"
    );
    assert_eq!(
        res.journey.parked_kind(),
        None,
        "single-level: the resumed palette parks nothing itself"
    );
}

// The OTHER branch — `Action::KeepVersion` reached with NO card open at all
// (no palette, nothing parked) still `enter`s rather than `descend`s, so Esc
// still lands with nothing left, exactly as before the fix (only the PARKED
// case above changes): `actions::tests::overlay_drive::
// keep_version_blank_enter_is_the_plain_keep_and_esc_cancels` pins it at the
// pure `apply_transition` seam this replay test builds on.

#[test]
fn replay_keys_keep_version_commit_closes_and_defers_the_store_write() {
    // Enter commits through the REAL keymap: the overlay closes and the
    // deferred Effect::KeepVersion { name } is the documented headless no-op
    // (the history determinism gate — a capture never touches the store), so
    // the buffer and fs stay untouched.
    use crate::fs::InMemoryFs;
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(InMemoryFs::new()));
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("h i s-p k e e p RET d r a f t RET").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.journey.card().is_none(), "commit closes the minibuffer");
    assert_eq!(buffer.text(), "hi", "the keep never edits the buffer");
}
