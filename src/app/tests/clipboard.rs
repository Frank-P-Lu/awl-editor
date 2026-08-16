//! **COPY MUST SURVIVE A BUFFER SWITCH.**
//!
//! # The bug
//!
//! `clipboard_last_written` (`App`-global) mirrors whatever text
//! `sync_kill_to_clipboard` last wrote to the OS clipboard.
//! `refresh_kill_from_clipboard` used it to skip re-hydrating the ACTIVE
//! buffer's kill ring whenever the OS text still matched that stamp — sound
//! within one buffer, but the kill ring the stamp is meant to describe is
//! PER-BUFFER. After a switch, the new buffer's own kill ring is whatever it
//! was last (commonly empty), yet the stamp still equals the OS text left by
//! the buffer just departed — so the skip fired, the new buffer was never
//! hydrated, and `YankText` read an empty kill ring: Paste silently inserted
//! nothing.
//!
//! # Why this tier
//!
//! Both switch doors (`load_path`, `last_buffer_toggle`) and the clipboard
//! bridge (`sync_kill_to_clipboard`/`refresh_kill_from_clipboard`) are live-
//! `App`-only — headless `--keys` replay treats `BufferEffect::Previous` as a
//! no-op and never builds a real `App` at all (`docs/harness-reach.md`).
//! [`FakeClipboard`] (`src/app.rs`) is the controllable seam: a fake OS
//! pasteboard installed in place of the real `arboard::Clipboard`, so every
//! law here is hermetic and deterministic instead of racing the developer's
//! real system clipboard or silently no-opping on a headless CI runner with
//! no clipboard service.
//!
//! Every test drives `Action::CopyRegion` / `Action::Yank` through the real
//! `App::apply` (via [`dispatch`]) — the same interpreter a keypress reaches
//! — rather than calling the clipboard-bridge functions directly, so a
//! regression anywhere in that chain (not just in the one line this item
//! fixes) would show up here too.

use super::*;
use crate::fs::{FileSystem, InMemoryFs};

/// Install a hermetic [`FakeClipboard`] on `app`, replacing whatever
/// `App::new`'s own `arboard::Clipboard::new()` attempt produced (or failed
/// to produce — a headless CI runner has no clipboard service at all). No law
/// in this file touches the real OS pasteboard.
fn install_fake_clipboard(app: &mut App) -> FakeClipboard {
    let fake = FakeClipboard::new();
    app.clipboard = Some(Box::new(fake.clone()));
    app.clipboard_last_written = None;
    fake
}

/// Drive one `Action` through the REAL live interpreter (`App::apply` →
/// `apply_live_effect` → the clipboard bridge), exactly as a keypress or the
/// palette would — never a hand-rolled stand-in for it.
fn dispatch(app: &mut App, action: Action) {
    let exit = crate::app::schedule::RecordingExit::new();
    app.apply(action, false, &exit, crate::stats::Door::Chord);
}

/// Select `start..end` (char offsets) in the active buffer and Copy it — the
/// real path: `Action::CopyRegion` → `Effect::Clipboard(WriteKillRing)` →
/// `sync_kill_to_clipboard`.
fn copy_selection(app: &mut App, start: usize, end: usize) {
    app.document.select_range(start, end);
    dispatch(app, Action::CopyRegion);
}

/// Paste — the real path: `Action::Yank` → `Effect::Clipboard(PasteImage)` →
/// `paste_image_reference` (no image on the fake clipboard, so it falls
/// through) → `refresh_kill_from_clipboard` → `Action::YankText`.
fn paste(app: &mut App) {
    dispatch(app, Action::Yank);
}

// ── THE MINIMAL, DIRECT NON-VACUITY CASE ─────────────────────────────────

#[test]
fn refresh_kill_from_clipboard_hydrates_a_buffer_whose_kill_ring_never_held_the_stamped_text() {
    // Manufacture TODAY'S BUG PRECONDITION exactly, without going through a
    // real switch: the OS clipboard equals the App-global stamp, but the
    // ACTIVE buffer's own kill ring is empty (it never produced that stamp).
    let mut app = app_on(None, "/proj", Config::empty());
    let fake = install_fake_clipboard(&mut app);
    fake.set_external("alpha");
    app.clipboard_last_written = Some("alpha".to_string());
    assert_eq!(
        app.document.buffer().kill_buffer(),
        "",
        "premise: a fresh buffer's kill ring starts empty"
    );

    app.refresh_kill_from_clipboard();

    assert_eq!(
        app.document.buffer().kill_buffer(),
        "alpha",
        "the stamp matching the OS text must not skip hydrating a buffer \
         that never actually held it"
    );
}

// ── BOTH SWITCH DOORS, END TO END THROUGH REAL COPY/PASTE ────────────────

#[test]
fn copy_in_a_survives_a_direct_activation_switch_to_b_then_pastes() {
    let a = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new().with_file(&a, "alpha\n").with_file(&b, "");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(
        Some(a.clone()),
        "/proj",
        Config {
            autosave: Some(false),
            ..Config::empty()
        },
    );
    let fake = install_fake_clipboard(&mut app);

    copy_selection(&mut app, 0, 5); // "alpha"
    assert_eq!(fake.current().as_deref(), Some("alpha"));
    assert_eq!(app.clipboard_last_written.as_deref(), Some("alpha"));

    // Direct activation: the Goto-picker door (`open_rel` → `load_path`).
    app.load_path(b.clone());
    assert_eq!(app.document.buffer().path(), Some(b.as_path()));

    // THE EXACT BUG PRECONDITION, reached naturally by the switch: the stamp
    // still reads "alpha" (nothing clears it on a switch) while B's own kill
    // ring — a fresh buffer's — is empty.
    assert_eq!(app.clipboard_last_written.as_deref(), Some("alpha"));
    assert_eq!(app.document.buffer().kill_buffer(), "");

    paste(&mut app);

    assert_eq!(
        app.document.buffer().text(),
        "alpha",
        "Paste in B must insert the text copied in A"
    );
    // ONE undoable edit.
    dispatch(&mut app, Action::Undo);
    assert_eq!(app.document.buffer().text(), "");

    // A itself is untouched by any of this.
    app.load_path(a.clone());
    assert_eq!(app.document.buffer().text(), "alpha\n");
}

#[test]
fn copy_in_a_survives_a_last_file_toggle_switch_to_b_then_pastes() {
    let a = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new().with_file(&a, "alpha\n").with_file(&b, "");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    // Open B first, then A, so B becomes `previous_path()` — the toggle then
    // swaps A -> B, exactly the door `last_buffer_toggle` (`C-x b` / Finish)
    // owns, distinct from a picker's direct activation.
    let mut app = app_on(
        Some(b.clone()),
        "/proj",
        Config {
            autosave: Some(false),
            ..Config::empty()
        },
    );
    app.load_path(a.clone());
    assert_eq!(app.document.previous_path(), Some(b.clone()));
    let fake = install_fake_clipboard(&mut app);

    copy_selection(&mut app, 0, 5); // "alpha"
    assert_eq!(fake.current().as_deref(), Some("alpha"));

    app.last_buffer_toggle();
    assert_eq!(app.document.buffer().path(), Some(b.as_path()));
    assert_eq!(app.document.buffer().kill_buffer(), "");

    paste(&mut app);

    assert_eq!(app.document.buffer().text(), "alpha");
}

// ── MULTILINE + MULTIBYTE SWEEP (direct activation) ──────────────────────

#[test]
fn paste_after_switch_sweeps_multiline_and_multibyte_text() {
    for fixture in [
        "single line",
        "line one\nline two\n\nline four after a blank line",
        "日本語のテキスト\n複数行\n",
        "emoji 🎉 mid-sentence and a final line",
    ] {
        let a = PathBuf::from("/proj/a.txt");
        let b = PathBuf::from("/proj/b.txt");
        let mem = InMemoryFs::new().with_file(&a, fixture).with_file(&b, "");
        let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
        let mut app = app_on(
            Some(a.clone()),
            "/proj",
            Config {
                autosave: Some(false),
                ..Config::empty()
            },
        );
        let _fake = install_fake_clipboard(&mut app);

        let char_len = fixture.chars().count();
        copy_selection(&mut app, 0, char_len);
        app.load_path(b.clone());
        assert_eq!(
            app.document.buffer().kill_buffer(),
            "",
            "fixture {fixture:?}: B's kill ring starts empty"
        );

        paste(&mut app);

        assert_eq!(
            app.document.buffer().text(),
            fixture,
            "fixture {fixture:?}: paste after switch must reproduce it exactly"
        );
    }
}

// ── A DESTINATION SELECTION MUST BE REPLACED, NOT AUGMENTED ──────────────

#[test]
fn paste_after_switch_replaces_an_existing_destination_selection() {
    let a = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new()
        .with_file(&a, "alpha\n")
        .with_file(&b, "XXXreplaceMEYYY");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(
        Some(a.clone()),
        "/proj",
        Config {
            autosave: Some(false),
            ..Config::empty()
        },
    );
    let _fake = install_fake_clipboard(&mut app);

    copy_selection(&mut app, 0, 5); // "alpha"
    app.load_path(b.clone());

    // "XXX" (0..3) + "replaceME" (3..12) + "YYY" (12..15)
    app.document.select_range(3, 12);
    paste(&mut app);

    assert_eq!(app.document.buffer().text(), "XXXalphaYYY");
}

// ── AN EXTERNAL OVERWRITE BETWEEN SWITCH AND PASTE WINS ───────────────────

#[test]
fn an_external_clipboard_overwrite_between_switch_and_paste_wins_over_the_stale_stamp() {
    let a = PathBuf::from("/proj/a.txt");
    let b = PathBuf::from("/proj/b.txt");
    let mem = InMemoryFs::new().with_file(&a, "alpha\n").with_file(&b, "");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(
        Some(a.clone()),
        "/proj",
        Config {
            autosave: Some(false),
            ..Config::empty()
        },
    );
    let fake = install_fake_clipboard(&mut app);

    copy_selection(&mut app, 0, 5); // "alpha"
    app.load_path(b.clone());

    // Another application writes the OS clipboard directly — awl never sees
    // this write, so `clipboard_last_written` still reads "alpha".
    fake.set_external("from another app");
    assert_eq!(app.clipboard_last_written.as_deref(), Some("alpha"));

    paste(&mut app);

    assert_eq!(
        app.document.buffer().text(),
        "from another app",
        "the OS clipboard is authoritative: a real external change must win \
         over awl's own stale stamp"
    );
    assert_eq!(
        app.clipboard_last_written.as_deref(),
        Some("from another app")
    );
}

// ── SAME-BUFFER SUPPRESSION LAWS STAY INTACT ──────────────────────────────

#[test]
fn same_buffer_copy_then_paste_twice_keeps_the_redundant_write_read_suppression() {
    let a = PathBuf::from("/proj/a.txt");
    let mem = InMemoryFs::new().with_file(&a, "hi\n");
    let _g = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(
        Some(a.clone()),
        "/proj",
        Config {
            autosave: Some(false),
            ..Config::empty()
        },
    );
    let fake = install_fake_clipboard(&mut app);

    copy_selection(&mut app, 0, 2); // "hi"
    assert_eq!(fake.current().as_deref(), Some("hi"));
    assert_eq!(app.clipboard_last_written.as_deref(), Some("hi"));

    // First paste: real hydrate (buffer's kill already equals "hi" from the
    // copy above, so THIS call already exercises the same-buffer skip).
    paste(&mut app);
    assert_eq!(app.document.buffer().kill_buffer(), "hi");
    assert_eq!(app.clipboard_last_written.as_deref(), Some("hi"));

    // Second paste: still the same-buffer skip path, still correct.
    paste(&mut app);
    assert_eq!(app.document.buffer().text(), "hihihi\n");
    assert_eq!(app.clipboard_last_written.as_deref(), Some("hi"));
}
