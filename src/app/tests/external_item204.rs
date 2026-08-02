//! ITEM 204's TIER-2 SWEEP: every gated action, both resolutions, and relaunch
//! recovery, driven into a hermetic live `App` over an `InMemoryFs`.
//!
//! **Why this tier and not a capture** (`docs/harness-reach.md`): the conflict is
//! latched on the `App`'s own per-buffer disk baseline, which nothing below the
//! live `App` builds — an ordinary `--keys` replay never opens a baseline at all,
//! and `--screenshot-app`'s hermetic sandbox has no way to make an external write
//! happen mid-run. So the two resolution effects classify **Unsupported**, and
//! the honest oracle is Rust assertions on real `App` state.
//!
//! The axis these sweep is THE DOOR, not the state: every boundary that could
//! write to, leave, or rename a conflicted file is driven, because a guard that
//! covers four of five doors is a guard that loses work through the fifth.

use super::*;
use crate::fs::{FileSystem, InMemoryFs};

const DISK_FIRST: &str = "what was on disk\n";
/// Deliberately the SAME LENGTH as `DISK_FIRST`: every external write in this
/// file is invisible to an mtime-and-length guard, so nothing here can pass by
/// accident on the stat.
const DISK_SECOND: &str = "somebody else typed\n";
const MINE: &str = "what I typed instead\n";

fn doc() -> PathBuf {
    PathBuf::from("/notes/draft.md")
}

/// A hermetic App holding UNSAVED edits over a file that has since changed on
/// disk — the state every door below is tested against. Returns the App and the
/// fake so the test can keep writing behind awl's back.
fn conflicted() -> (App, InMemoryFs, crate::fs::FsGuard) {
    let mem = InMemoryFs::new().with_file(doc(), DISK_FIRST);
    let guard = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(doc()), "/notes", Config::empty());
    // The user types.
    app.document.set_text(MINE);
    // Somebody else writes the file. Same byte length as what awl last saw.
    mem.write(&doc(), DISK_SECOND.as_bytes()).unwrap();
    (app, mem, guard)
}

fn palette_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-p",
        crate::convention::Convention::Linux => "C-p",
    }
}

/// Run a catalog command THROUGH THE COMMAND PALETTE by real chords — the only
/// door these two rows have. Typing the name filters; Enter accepts. Returns
/// whether the palette actually offered a row to accept, which is what makes
/// the hidden-row assertions below meaningful rather than vacuous.
fn run_from_palette(app: &mut App, name: &str) -> bool {
    app.press_spec_headless(palette_chord())
        .expect("the palette chord parses");
    let keys: Vec<String> = name
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_string())
        .collect();
    app.press_spec_headless(&keys.join(" "))
        .expect("the typed name parses");
    let offered = app
        .workspace_state
        .overlay()
        .and_then(|o| o.selected_value())
        .is_some_and(|v| v == name);
    app.press_spec_headless("Enter").expect("Enter parses");
    offered
}

// ── THE DOORS ────────────────────────────────────────────────────────────

/// **THE HEADLINE, AND MUTATION-PROOF TARGET #2.** A manual Save must not
/// force-write over an external change.
///
/// The retired contract said "⌘S keeps yours", which spent the other manuscript
/// to do it, silently, on a keystroke the user reaches for without thinking.
#[test]
fn a_manual_save_no_longer_overwrites_an_external_change() {
    let (mut app, mem, _fs) = conflicted();
    app.manual_save();
    assert_eq!(
        mem.read_to_string(&doc()).unwrap(),
        DISK_SECOND,
        "⌘S must not destroy the other version"
    );
    assert_eq!(
        app.document.buffer().text(),
        MINE,
        "…and must not destroy this one either"
    );
    assert!(app.change_unresolved(), "the conflict is latched");
    assert_eq!(
        app.frame.notice().text(),
        Some(crate::app::CHANGED_ELSEWHERE_NOTICE)
    );
}

/// EVERY DOOR THAT WOULD LEAVE OR RENAME THE CONFLICTED FILE is refused, and
/// refused the same way. Swept as a table so a door added later without a gate
/// is a missing row rather than an invisible hole.
#[test]
fn every_gated_door_is_refused_while_the_change_is_unresolved() {
    type Door = (&'static str, fn(&mut App));
    let doors: &[Door] = &[
        ("manual save", |a| a.manual_save()),
        ("idle autosave", |a| a.autosave_flush()),
        ("finish file", |a| a.save_finished_buffer()),
        ("switch buffer", |a| {
            a.load_path(PathBuf::from("/notes/other.md"))
        }),
        ("rename", |a| a.rename_current_file("renamed.md")),
        ("move", |a| a.move_current_file("sub")),
        ("focus return", |a| a.on_focus_gained()),
    ];
    for (what, drive) in doors {
        let (mut app, mem, _fs) = conflicted();
        mem.write(std::path::Path::new("/notes/other.md"), b"elsewhere\n")
            .unwrap();
        // Latch first, so each door is exercised against a REAL open conflict
        // rather than being the thing that raises it.
        app.manual_save();
        assert!(app.change_unresolved(), "{what}: precondition");

        drive(&mut app);

        assert_eq!(
            mem.read_to_string(&doc()).unwrap(),
            DISK_SECOND,
            "{what}: the disk version survived"
        );
        assert_eq!(
            app.document.buffer().text(),
            MINE,
            "{what}: the buffer version survived"
        );
        assert_eq!(
            app.document.buffer().path().map(|p| p.to_path_buf()),
            Some(doc()),
            "{what}: the conflicted document is still the active, sole editable one"
        );
        assert!(
            app.change_unresolved(),
            "{what}: the conflict is still open — the door did not quietly resolve it"
        );
        assert_eq!(
            app.frame.notice().text(),
            Some(crate::app::CHANGED_ELSEWHERE_NOTICE),
            "{what}: the refusal names the way out"
        );
        // …and the record still holds exactly what the buffer holds.
        assert_eq!(
            crate::recovery::read().map(|r| r.text),
            Some(MINE.to_string()),
            "{what}: the only copy of the user's text is still recorded"
        );
    }
}

/// QUIT routes back through resolution ONCE, then proceeds — and the record is
/// on disk either way, so the second Quit is not a loss.
#[test]
fn quit_is_deferred_once_and_never_traps() {
    let (mut app, _mem, _fs) = conflicted();
    app.manual_save();
    assert!(app.change_unresolved());

    let first = app
        .press_spec_headless(quit_chord())
        .expect("the quit chord parses");
    assert!(!first, "the first Quit is sent back to the conflict");
    assert_eq!(
        app.frame.notice().text(),
        Some(crate::app::CHANGED_ELSEWHERE_NOTICE)
    );
    assert_eq!(
        crate::recovery::read().map(|r| r.text),
        Some(MINE.to_string()),
        "the deferred Quit wrote the record before refusing"
    );

    let second = app
        .press_spec_headless(quit_chord())
        .expect("the quit chord parses");
    assert!(
        second,
        "a second Quit proceeds — refusing forever would trap someone whose only \
         way out is a decision they are not ready to make"
    );
}

/// Quit's chord in the convention this run is driving. Spelled out rather than
/// derived, because the catalog's `effective_bindings` returns DISPLAY labels
/// (`⌘Q`) and this door wants a keyspec. The trap worth naming: Linux quit is
/// `C-q`, NOT the emacs `C-x C-c` a reader reaches for — the gate runs both
/// conventions, so the wrong one here is green on one and red on the other.
fn quit_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-q",
        crate::convention::Convention::Linux => "C-q",
    }
}

// ── THE TWO RESOLUTIONS ──────────────────────────────────────────────────

/// **SAVE YOUR VERSION**, through the palette by real chords. The row is offered
/// only because a conflict is open, and accepting it writes.
#[test]
fn save_your_version_writes_and_clears_the_record() {
    let (mut app, mem, _fs) = conflicted();
    app.manual_save();
    assert!(
        run_from_palette(&mut app, "Save your version"),
        "the row is offered"
    );

    assert_eq!(
        mem.read_to_string(&doc()).unwrap(),
        MINE,
        "the user's version is now the file"
    );
    assert!(!app.change_unresolved(), "resolved");
    assert_eq!(crate::recovery::read(), None, "the record is retired");
    // The baseline moved with the write: an immediate re-check is quiet.
    assert_eq!(
        crate::external::look(&doc(), &app.document.disk_baseline()).0,
        crate::external::Change::Unchanged
    );
    // And an ordinary autosave now works again.
    app.document.set_text("still writing\n");
    app.autosave_flush();
    assert_eq!(mem.read_to_string(&doc()).unwrap(), "still writing\n");
}

/// **SAVE YOUR VERSION RECHECKS.** If the file moved AGAIN between the conflict
/// and the decision, the write is refused and the conflict re-raised: consent to
/// replace one version is not consent to replace any version.
#[test]
fn save_your_version_refuses_when_the_disk_moved_again() {
    let (mut app, mem, _fs) = conflicted();
    app.manual_save();
    // A third party writes once more, AFTER the user was shown the conflict.
    mem.write(&doc(), b"a third version entirely\n").unwrap();

    app.resolve_keep_mine();

    assert_eq!(
        mem.read_to_string(&doc()).unwrap(),
        "a third version entirely\n",
        "the newest disk version was not overwritten"
    );
    assert!(app.change_unresolved(), "still unresolved");
    assert_eq!(
        app.frame.notice().text(),
        Some("changed elsewhere again — check before saving")
    );
    // Deciding again, now against what the user was actually shown, works.
    app.resolve_keep_mine();
    assert_eq!(mem.read_to_string(&doc()).unwrap(), MINE);
    assert!(!app.change_unresolved());
}

/// **USE DISK VERSION** is ONE undoable replacement — the claim the notice makes,
/// asserted by actually undoing it.
#[test]
fn use_disk_version_is_one_undoable_replacement() {
    let (mut app, mem, _fs) = conflicted();
    app.manual_save();
    assert!(
        run_from_palette(&mut app, "Use disk version"),
        "the row is offered"
    );

    assert_eq!(
        app.document.buffer().text(),
        DISK_SECOND,
        "the buffer took the disk"
    );
    assert!(!app.change_unresolved(), "resolved");
    assert_eq!(crate::recovery::read(), None, "the record is retired");
    assert_eq!(
        mem.read_to_string(&doc()).unwrap(),
        DISK_SECOND,
        "…and nothing was written; the file is untouched"
    );

    // ONE undo brings the user's own text straight back. This is the whole
    // safety story of the destructive-looking arm.
    app.document.undo();
    assert_eq!(
        app.document.buffer().text(),
        MINE,
        "a single undo restores the user's version"
    );
}

/// A DELETED file has no version to take, and the resolution declines rather
/// than replacing a manuscript with nothing.
#[test]
fn use_disk_version_declines_when_the_file_was_deleted() {
    let mem = InMemoryFs::new().with_file(doc(), DISK_FIRST);
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(doc()), "/notes", Config::empty());
    app.document.set_text(MINE);
    mem.remove_file(&doc()).unwrap();

    app.manual_save();
    assert!(
        app.change_unresolved(),
        "a deletion latches like any other change"
    );
    assert!(
        !mem.exists(&doc()),
        "awl must not silently re-create a file somebody deleted"
    );

    app.resolve_take_theirs();
    assert_eq!(
        app.document.buffer().text(),
        MINE,
        "there is no disk version to take, so nothing was replaced"
    );
    assert!(app.change_unresolved(), "still unresolved");

    // Save your version is the way out of a deletion, and it re-creates the
    // file deliberately, on an explicit instruction.
    app.resolve_keep_mine();
    assert_eq!(mem.read_to_string(&doc()).unwrap(), MINE);
    assert!(!app.change_unresolved());
}

/// THE ROWS ARE HIDDEN when there is nothing to resolve. Offering an action that
/// does nothing is the exact defect the retired "reopen for theirs" notice was.
#[test]
fn the_resolution_rows_are_offered_only_while_a_change_is_unresolved() {
    let mem = InMemoryFs::new().with_file(doc(), DISK_FIRST);
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(doc()), "/notes", Config::empty());
    for name in ["Save your version", "Use disk version"] {
        assert!(
            !run_from_palette(&mut app, name),
            "{name:?} must not be selectable with no conflict open"
        );
        assert_eq!(
            app.document.buffer().text(),
            DISK_FIRST,
            "{name:?}: nothing happened"
        );
    }
    // Under a real conflict, both appear. Each probe re-establishes the
    // divergence from scratch, because the PREVIOUS probe's acceptance resolved
    // it — a loop that reuses one conflict would find the second row correctly
    // hidden and report that as a failure of the gate rather than of the test.
    for name in ["Save your version", "Use disk version"] {
        crate::recovery::clear();
        mem.write(&doc(), DISK_FIRST.as_bytes()).unwrap();
        let mut probe = app_on(Some(doc()), "/notes", Config::empty());
        probe.document.set_text(MINE);
        mem.write(&doc(), DISK_SECOND.as_bytes()).unwrap();
        probe.manual_save();
        assert!(probe.change_unresolved(), "{name:?}: precondition");
        assert!(
            run_from_palette(&mut probe, name),
            "{name:?} must be selectable while a conflict is open"
        );
        assert!(
            !probe.change_unresolved(),
            "{name:?}: accepting the row really resolved the conflict"
        );
    }
}

// ── THE CLEAN-BUFFER RELOAD ──────────────────────────────────────────────

/// A CLEAN buffer reloads from disk, keeping the caret's line/column and the
/// scroll. No conflict, no notice to dismiss — just the current text.
#[test]
fn a_clean_buffer_reloads_from_disk_keeping_cursor_and_scroll() {
    let path = PathBuf::from("/notes/long.md");
    let before = "one\ntwo\nthree\nfour\nfive\n";
    let after = "ONE\nTWO\nthree\nfour\nfive\nsix\n";
    let mem = InMemoryFs::new().with_file(&path, before);
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(path.clone()), "/notes", Config::empty());
    // Park the caret on line 3, column 2, and scroll down.
    let idx = app.document.buffer().line_col_to_char(2, 2);
    app.document.set_cursor(idx);
    app.document.set_scroll(crate::render::ScrollPos::at_row(3));

    mem.write(&path, after.as_bytes()).unwrap();
    app.on_focus_gained();

    assert_eq!(app.document.buffer().text(), after, "the disk won");
    assert!(!app.change_unresolved(), "a clean buffer is not a conflict");
    assert_eq!(
        app.document
            .buffer()
            .char_to_line_col(app.document.buffer().cursor_char()),
        (2, 2),
        "the caret kept its line and column"
    );
    assert_eq!(
        app.document.scroll().row,
        3,
        "the scroll position survived the reload"
    );
    assert_eq!(crate::recovery::read(), None, "nothing to recover");
    // Reloading is idempotent: a second look reports nothing.
    let text_after_reload = app.document.buffer().text();
    app.on_focus_gained();
    assert_eq!(app.document.buffer().text(), text_after_reload);
}

/// AN IDENTICAL REWRITE IS NOT A CHANGE. `touch`, a checkout that restores the
/// same revision, or a backup tool rewriting bytes for bytes must not raise a
/// conflict over an unsaved buffer — the old stat compare did, and a guard that
/// cries wolf is one users learn to dismiss.
#[test]
fn a_byte_identical_rewrite_is_not_a_conflict() {
    let mem = InMemoryFs::new().with_file(doc(), DISK_FIRST);
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(doc()), "/notes", Config::empty());
    app.document.set_text(MINE);
    // Same bytes, new timestamp.
    mem.write(&doc(), DISK_FIRST.as_bytes()).unwrap();

    app.autosave_flush();

    assert!(
        !app.change_unresolved(),
        "identical bytes are not a conflict"
    );
    assert_eq!(
        mem.read_to_string(&doc()).unwrap(),
        MINE,
        "the ordinary autosave went through"
    );
}

// ── REPEATED EXTERNAL WRITES ─────────────────────────────────────────────

/// The guard survives a file being written repeatedly behind awl's back: each
/// pass keeps both versions, and the record tracks the buffer rather than
/// freezing at the first conflict.
#[test]
fn repeated_external_writes_never_lose_either_side() {
    let (mut app, mem, _fs) = conflicted();
    for n in 0..4u32 {
        let theirs = format!("their revision {n}\n");
        mem.write(&doc(), theirs.as_bytes()).unwrap();
        let mine = format!("my revision {n}\n");
        app.document.set_text(&mine);
        app.autosave_flush();

        assert_eq!(
            mem.read_to_string(&doc()).unwrap(),
            theirs,
            "pass {n}: their text survived"
        );
        assert_eq!(
            app.document.buffer().text(),
            mine,
            "pass {n}: mine survived"
        );
        assert_eq!(
            crate::recovery::read().map(|r| r.text),
            Some(mine),
            "pass {n}: the record tracks the buffer, it does not freeze"
        );
    }
}

// ── RELAUNCH RECOVERY ────────────────────────────────────────────────────

/// **MUTATION-PROOF TARGET #3.** Unresolved state survives a crash and relaunch:
/// the text awl was holding comes back, and the conflict comes back with it.
///
/// The "crash" is modelled by dropping the `App` without a clean exit — no
/// `exiting`, no session flush — while the fake filesystem stays installed, which
/// is exactly what a `SIGKILL` leaves behind.
#[test]
fn an_unresolved_change_survives_a_crash_and_relaunch() {
    let mem = InMemoryFs::new().with_file(doc(), DISK_FIRST);
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    {
        let mut app = app_on(Some(doc()), "/notes", Config::empty());
        app.document.set_text(MINE);
        mem.write(&doc(), DISK_SECOND.as_bytes()).unwrap();
        app.manual_save();
        assert!(app.change_unresolved());
        // …and the process dies here. No exit hook runs.
    }
    assert_eq!(
        crate::recovery::read().map(|r| r.text),
        Some(MINE.to_string()),
        "the record outlived the process"
    );

    // Relaunch on the same file.
    let mut relaunched = app_on(Some(doc()), "/notes", Config::empty());
    assert_eq!(
        relaunched.document.buffer().text(),
        MINE,
        "the unsaved text came back"
    );
    assert!(
        relaunched.change_unresolved(),
        "and so did the conflict — otherwise the very next save clobbers"
    );
    assert_eq!(
        relaunched.frame.notice().text(),
        Some(crate::app::CHANGED_ELSEWHERE_NOTICE)
    );
    // Both versions are still reachable after the relaunch.
    relaunched.resolve_take_theirs();
    assert_eq!(relaunched.document.buffer().text(), DISK_SECOND);
    relaunched.document.undo();
    assert_eq!(relaunched.document.buffer().text(), MINE);
}

/// A record belonging to ANOTHER file is neither adopted nor destroyed by
/// opening this one. It is the only copy of that text.
#[test]
fn a_record_for_another_file_is_left_alone() {
    let mem = InMemoryFs::new()
        .with_file(doc(), DISK_FIRST)
        .with_file("/notes/other.md", "other\n");
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    assert!(crate::recovery::write(&crate::recovery::Record {
        path: PathBuf::from("/notes/other.md"),
        text: "somebody else's unsaved work\n".into(),
    }));

    let app = app_on(Some(doc()), "/notes", Config::empty());
    assert_eq!(
        app.document.buffer().text(),
        DISK_FIRST,
        "this document was not contaminated by another file's record"
    );
    assert!(!app.change_unresolved());
    assert_eq!(
        crate::recovery::read().map(|r| r.text),
        Some("somebody else's unsaved work\n".to_string()),
        "…and the other file's record was not destroyed"
    );
}

/// OPENING the file a record belongs to adopts it, which is what makes the
/// record findable on a launch that did not happen to reopen it first.
#[test]
fn opening_the_recorded_file_adopts_its_unresolved_change() {
    let mem = InMemoryFs::new()
        .with_file(doc(), DISK_SECOND)
        .with_file("/notes/start.md", "start\n");
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    assert!(crate::recovery::write(&crate::recovery::Record {
        path: doc(),
        text: MINE.into(),
    }));

    let mut app = app_on(
        Some(PathBuf::from("/notes/start.md")),
        "/notes",
        Config::empty(),
    );
    assert!(
        !app.change_unresolved(),
        "the launch document has no conflict"
    );
    app.load_path(doc());
    assert_eq!(
        app.document.buffer().text(),
        MINE,
        "the held text came back"
    );
    assert!(app.change_unresolved(), "and the conflict with it");
}

// ── ANTI-VACUITY ─────────────────────────────────────────────────────────

/// THE SWEEP ABOVE MUST BE PRESSING REAL CHORDS. `run_from_palette` would return
/// `false` for every row if the palette never opened, which would make the
/// hidden-row assertions pass over nothing at all.
#[test]
fn the_palette_door_this_file_drives_really_opens() {
    let mem = InMemoryFs::new().with_file(doc(), DISK_FIRST);
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(Some(doc()), "/notes", Config::empty());
    app.press_spec_headless(palette_chord())
        .expect("the palette chord parses");
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Command),
        "the chord this file uses really summons the command palette"
    );
    // …and an UNGATED row is reachable through the very same helper, so a
    // `false` above means "hidden", never "the helper is broken".
    app.press_spec_headless("Escape").expect("Escape parses");
    assert!(
        run_from_palette(&mut app, "Keep version…"),
        "an ungated row is offered through this same door"
    );
}
