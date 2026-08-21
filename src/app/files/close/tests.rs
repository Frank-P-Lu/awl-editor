//! Laws for THE REMOVAL OWNER.
//!
//! These run against a REAL on-disk `ScratchDir` rather than `InMemoryFs`,
//! because the thing under test is the conflict gate, and the gate's whole job
//! is to notice that a file MOVED between two observations. Simulating that
//! needs a filesystem a test can write behind the App's back —
//! `App::new_hermetic`'s injected in-memory backend is exactly the thing that
//! makes such a write unreachable. So: the real disk, the shared `testlock`
//! guard (a concurrent `fs::with_fs` would otherwise steal these saves into
//! someone else's backend), and session restore explicitly off so no developer's
//! real open files are parked into this test's registry.

use super::*;
use crate::config::Config;
#[cfg(not(target_arch = "wasm32"))]
use crate::fs::FileSystem;
use crate::testscratch::ScratchDir;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

/// A two-file session on real disk: `a.txt` and `b.txt` under one root, both
/// open, `b` active and `a` parked. The everyday shape every law below starts
/// from.
struct Session {
    dir: ScratchDir,
    app: App,
}

impl Session {
    fn new(name: &str) -> Self {
        Self::with_autosave(name, true)
    }

    /// The same session with the AUTOSAVE ENGINE OFF.
    ///
    /// Needed to reach a dirty PARKED entry at all, and the reason is worth
    /// stating: `load_path` flushes autosave before it parks, so under the
    /// default config a buffer is written to disk on its way out and a parked
    /// entry is essentially never unsaved. The unsaved-parked state is real —
    /// `autosave = false` is a supported config — but a law that tried to build
    /// it with autosave on would silently test the CLEAN path while claiming to
    /// test the dirty one.
    fn without_autosave(name: &str) -> Self {
        Self::with_autosave(name, false)
    }

    fn with_autosave(name: &str, autosave: bool) -> Self {
        let dir = ScratchDir::new(
            std::env::temp_dir().join(format!("awl-close-{name}-{}", std::process::id())),
        );
        std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
        std::fs::write(dir.join("b.txt"), "beta\n").unwrap();
        let cfg = Config {
            session_restore: Some(false),
            autosave: Some(autosave),
            ..Config::empty()
        };
        let mut app = App::new(Some(dir.join("a.txt")), dir.to_path_buf(), None, None, cfg);
        app.load_path(dir.join("b.txt"));
        Self { dir, app }
    }

    fn a(&self) -> PathBuf {
        self.dir.join("a.txt")
    }

    fn b(&self) -> PathBuf {
        self.dir.join("b.txt")
    }

    /// The working set's paths, in the order the margin draws them.
    fn open_files(&self) -> Vec<PathBuf> {
        self.app
            .document
            .working_set()
            .files()
            .iter()
            .filter_map(|f| f.path.clone())
            .collect()
    }
}

/// Drive ⌘W through THE REAL EFFECT SEQUENCE the chord produces, rather than
/// calling the owner directly — so a law about "⌘W closes" cannot pass while the
/// chord's own wiring points somewhere else.
fn drive_finish_file(app: &mut App) {
    let transition = app.transition_for_test(&crate::keymap::Action::FinishBuffer, false);
    assert_eq!(
        &transition.effects()[..3],
        &[
            crate::actions::Effect::Persistence(crate::actions::PersistenceEffect::Save(
                crate::actions::SaveKind::Finish,
            )),
            crate::actions::Effect::Daemon(crate::actions::DaemonEffect::NotifyFinished),
            crate::actions::Effect::Buffer(crate::actions::BufferEffect::CloseActive),
        ],
        "Finish file's effects are save, notify, then CLOSE — not a park-and-switch"
    );
    app.save_finished_buffer();
    app.notify_finished_buffer();
    app.close_active_buffer();
}

/// ⌘W REMOVES THE ENTRY, it does not merely park it.
///
/// This is the whole item in one assertion, and the reason it needs stating: the
/// pre-existing behaviour SWITCHED AWAY correctly, so every observable fact about
/// the newly-active document was already right. The only thing that was wrong was
/// that the finished file stayed in the working set — which is exactly what the
/// margin draws.
#[test]
fn finish_file_removes_the_active_entry_rather_than_parking_it() {
    let _guard = crate::testlock::serial();
    let mut s = Session::new("finish-removes");
    assert_eq!(s.open_files(), vec![s.a(), s.b()], "both files start open");

    drive_finish_file(&mut s.app);

    assert_eq!(
        s.open_files(),
        vec![s.a()],
        "the finished file is GONE from the working set, not parked in it"
    );
    assert_eq!(
        s.app.document.buffer().path(),
        Some(s.a().as_path()),
        "the surviving file is active"
    );
    assert!(
        s.app
            .document
            .close_facts(&crate::buffers::BufferKey::path(&s.b()))
            .is_none(),
        "and its registry entry is gone too — both halves, or the margin draws a ghost"
    );
}

/// ⌘W still SAVES before it removes. The removal must not have quietly replaced
/// the lossless half of Finish with a faster one.
#[test]
fn finish_file_still_saves_the_buffer_it_closes() {
    let _guard = crate::testlock::serial();
    let mut s = Session::new("finish-saves");
    s.app.document.set_text("beta\nedited\n");

    drive_finish_file(&mut s.app);

    assert_eq!(
        std::fs::read_to_string(s.b()).unwrap(),
        "beta\nedited\n",
        "the edit landed on disk before the entry was removed"
    );
    assert_eq!(s.open_files(), vec![s.a()], "and then it closed");
}

/// Trash owns the full active/parked × clean/dirty/external-change matrix. A
/// fake records the one OS handoff without mutating the fixture file, so the
/// assertions can separate "we asked the OS" from "we released the buffer".
///
/// MUTATION TARGET: removing the `facts.unsaved` or external-baseline gates in
/// `App::trash_buffer` sends a dirty/conflicted path to the fake and this law
/// fails by name before any working-set assertion can hide the loss.
#[test]
fn trash_refuses_dirty_or_changed_buffers_and_releases_only_clean_targets() {
    let _guard = crate::testlock::serial();
    for parked in [false, true] {
        for state in ["clean", "dirty", "changed"] {
            let mut s = Session::without_autosave(&format!("trash-{parked}-{state}"));
            let target = if parked { s.a() } else { s.b() };
            if state == "dirty" {
                if parked {
                    s.app.load_path(target.clone());
                    s.app.document.set_text("alpha\nunsaved\n");
                    s.app.load_path(s.b());
                } else {
                    s.app.document.set_text("beta\nunsaved\n");
                }
            }
            if state == "changed" {
                std::fs::write(&target, "changed elsewhere\n").unwrap();
            }
            let before = s.open_files();
            let fake = Arc::new(crate::assets::FakeTrash::default());
            let recorder = fake.clone();
            let outcome = crate::assets::with_trash(fake, || {
                s.app.trash_buffer(crate::buffers::BufferKey::path(&target))
            });
            let asked = recorder.trashed.lock().unwrap().clone();
            match state {
                "clean" => {
                    assert_eq!(outcome, CloseOutcome::Closed, "parked={parked}");
                    assert_eq!(asked, vec![target.clone()], "parked={parked}");
                    assert!(
                        !s.open_files().contains(&target),
                        "parked={parked}: successful Trash removes its named working-set row"
                    );
                }
                "changed" if !parked => {
                    assert_eq!(outcome, CloseOutcome::Closed, "parked={parked}");
                    assert_eq!(asked, vec![target.clone()], "parked={parked}");
                    assert!(
                        !s.open_files().contains(&target),
                        "parked={parked}: successful Trash removes its named working-set row"
                    );
                }
                "dirty" | "changed" => {
                    assert_eq!(outcome, CloseOutcome::Refused, "parked={parked} {state}");
                    assert!(
                        asked.is_empty(),
                        "parked={parked} {state}: Trash was never called"
                    );
                    assert_eq!(
                        s.open_files(),
                        before,
                        "parked={parked} {state}: rows stay intact"
                    );
                    assert!(
                        target.exists(),
                        "parked={parked} {state}: disk file stays intact"
                    );
                }
                _ => unreachable!(),
            }
        }
    }
}

/// The final clean document leaves the honest no-document state only after the
/// fake has accepted its OS handoff; this is the Trash analogue of close's own
/// successor rule, rather than a new scratch-buffer invention.
#[test]
fn trashing_the_last_clean_document_enters_zero_document_state() {
    let _guard = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-trash-final-{}", std::process::id())),
    );
    let path = dir.join("only.md");
    std::fs::write(&path, "only\n").unwrap();
    let cfg = Config {
        session_restore: Some(false),
        autosave: Some(false),
        ..Config::empty()
    };
    let mut app = App::new(Some(path.clone()), dir.to_path_buf(), None, None, cfg);
    let fake = Arc::new(crate::assets::FakeTrash::default());
    let recorder = fake.clone();
    let outcome = crate::assets::with_trash(fake, || {
        app.trash_buffer(crate::buffers::BufferKey::path(&path))
    });
    assert_eq!(outcome, CloseOutcome::Closed);
    assert_eq!(recorder.trashed.lock().unwrap().as_slice(), [path]);
    assert!(
        !app.document.has_active(),
        "no scratch replacement is invented"
    );
    assert!(app.document.working_set().files().is_empty());
}

#[test]
fn backend_failure_attempts_once_and_preserves_final_nonfinal_and_parked_state() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Fail(Arc<AtomicUsize>);
    impl crate::assets::TrashCan for Fail {
        fn trash(&self, _: &Path) -> Result<(), String> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err("denied by OS".into())
        }
    }
    let _guard = crate::testlock::serial();
    for shape in ["active-nonfinal", "parked-nonfinal", "active-final"] {
        let mut s = Session::without_autosave(&format!("trash-fail-{shape}"));
        if shape == "active-final" {
            assert_eq!(
                s.app.close_buffer(crate::buffers::BufferKey::path(&s.a())),
                CloseOutcome::Closed
            );
        }
        let path = if shape == "parked-nonfinal" {
            s.a()
        } else {
            s.b()
        };
        let key = crate::buffers::BufferKey::path(&path);
        let open_before = s.open_files();
        let facts_before = s.app.document.close_facts(&key).unwrap();
        let active_text = s.app.document.buffer().text();
        let active_version = s.app.document.buffer().version();
        let disk = std::fs::read(&path).unwrap();
        let attempts = Arc::new(AtomicUsize::new(0));
        assert_eq!(
            crate::assets::with_trash(Arc::new(Fail(attempts.clone())), || {
                s.app.trash_buffer(key.clone())
            }),
            CloseOutcome::Refused
        );
        assert_eq!(attempts.load(Ordering::SeqCst), 1, "{shape}");
        assert_eq!(s.open_files(), open_before, "{shape}");
        let facts_after = s.app.document.close_facts(&key).unwrap();
        assert_eq!(facts_after.path, facts_before.path, "{shape}");
        assert_eq!(facts_after.unsaved, facts_before.unsaved, "{shape}");
        assert_eq!(facts_after.baseline, facts_before.baseline, "{shape}");
        assert_eq!(s.app.document.buffer().text(), active_text, "{shape}");
        assert_eq!(s.app.document.buffer().version(), active_version, "{shape}");
        assert_eq!(std::fs::read(&path).unwrap(), disk, "{shape}");
        assert_eq!(
            s.app.frame.notice().text(),
            Some("couldn't move to Trash: denied by OS")
        );
    }
}

#[test]
fn true_conflict_refusal_preserves_active_parked_and_final_shapes_without_backend_attempt() {
    let _guard = crate::testlock::serial();
    for shape in ["active-nonfinal", "parked-nonfinal", "active-final"] {
        let mut s = Session::without_autosave(&format!("trash-conflict-{shape}"));
        if shape == "active-final" {
            assert_eq!(
                s.app.close_buffer(crate::buffers::BufferKey::path(&s.a())),
                CloseOutcome::Closed
            );
        }
        let path = if shape == "parked-nonfinal" {
            s.a()
        } else {
            s.b()
        };
        let key = crate::buffers::BufferKey::path(&path);
        s.app
            .persistence
            .set_unresolved(crate::app::persistence::UnresolvedChange {
                path: path.clone(),
                theirs: Some("disk version\n".into()),
            });
        let before = s.open_files();
        let disk = std::fs::read(&path).unwrap();
        let fake = Arc::new(crate::assets::FakeTrash::default());
        let recorder = fake.clone();
        assert_eq!(
            crate::assets::with_trash(fake, || s.app.trash_buffer(key.clone())),
            CloseOutcome::Refused
        );
        assert!(recorder.trashed.lock().unwrap().is_empty(), "{shape}");
        assert_eq!(s.open_files(), before, "{shape}");
        assert_eq!(std::fs::read(&path).unwrap(), disk, "{shape}");
        assert!(s.app.persistence.unresolved_for(&path), "{shape}");
    }
}

/// The contextual filename card carries its parked row identity into the
/// accept effect. Without that payload, Enter would re-dispatch `TrashFile`
/// against the active document and this exact inactive-row case would remove
/// `b.txt` instead of `a.txt`.
#[test]
fn working_set_context_trash_targets_the_parked_row_not_the_active_document() {
    let _guard = crate::testlock::serial();
    let mut s = Session::without_autosave("trash-context-parked");
    let key = crate::buffers::BufferKey::path(&s.a());
    let state = crate::context_menu::ContextState {
        has_selection: false,
        link: false,
        heading: false,
        heading_folded: false,
        misspelled: false,
        named_file: true,
    };
    let mut card = crate::context_menu::overlay(
        crate::context_menu::rows(
            crate::context_menu::ContextTarget::Filename,
            state,
            crate::commands::Platform::Native,
        ),
        (0.0, 0.0),
    );
    card.selected = card
        .context_actions
        .iter()
        .position(|action| *action == crate::keymap::Action::TrashFile)
        .expect("premise: filename menu exposes Trash");
    s.app.workspace_state.summon_context_for_buffer(card, key);
    let fake = Arc::new(crate::assets::FakeTrash::default());
    let recorder = fake.clone();
    crate::assets::with_trash(fake, || {
        s.app
            .press_spec_headless("Enter")
            .expect("context Enter parses");
    });
    assert_eq!(recorder.trashed.lock().unwrap().as_slice(), [s.a()]);
    assert_eq!(s.open_files(), vec![s.b()]);
    assert_eq!(s.app.document.buffer().path(), Some(s.b().as_path()));
}

/// CLOSING AN INACTIVE ROW LEAVES THE ACTIVE DOCUMENT UNTOUCHED — asserted by
/// byte-identity of its text AND by its version, because a close that reached
/// through `self.document.buffer()` (the only save path that existed before this
/// module) would have saved the ACTIVE buffer's bytes to the parked file and
/// left the active text looking perfectly correct on screen.
#[test]
fn closing_an_inactive_row_never_touches_the_active_document() {
    let _guard = crate::testlock::serial();
    let mut s = Session::new("inactive-clean");
    s.app.document.set_text("beta\nlive edit\n");
    let text_before = s.app.document.buffer().text();
    let version_before = s.app.document.buffer().version();

    let outcome = s.app.close_buffer(crate::buffers::BufferKey::path(&s.a()));

    assert_eq!(outcome, CloseOutcome::Closed);
    assert_eq!(s.open_files(), vec![s.b()], "only the named row closed");
    assert_eq!(
        s.app.document.buffer().path(),
        Some(s.b().as_path()),
        "the active document was never activated away from"
    );
    assert_eq!(
        s.app.document.buffer().text(),
        text_before,
        "the active buffer's text is byte-identical"
    );
    assert_eq!(
        s.app.document.buffer().version(),
        version_before,
        "and nothing edited it"
    );
    assert_eq!(
        std::fs::read_to_string(s.a()).unwrap(),
        "alpha\n",
        "the closed file kept its own content — the active buffer's bytes went nowhere near it"
    );
}

/// A DIRTY INACTIVE ENTRY IS SAVED BEFORE IT IS CLOSED.
///
/// The companion to the clean case above, and the one that is not vacuous: a
/// close that simply dropped the entry passes every assertion of the clean law
/// and loses the user's text here.
#[test]
fn a_dirty_inactive_entry_is_saved_before_it_is_closed() {
    let _guard = crate::testlock::serial();
    let mut s = Session::without_autosave("inactive-dirty");
    // Edit A, then leave it — so A is parked AND unsaved.
    s.app.load_path(s.a());
    s.app.document.set_text("alpha\nunsaved work\n");
    s.app.load_path(s.b());
    assert_eq!(
        std::fs::read_to_string(s.a()).unwrap(),
        "alpha\n",
        "precondition: the edit is only in the parked buffer, not on disk"
    );

    let outcome = s.app.close_buffer(crate::buffers::BufferKey::path(&s.a()));

    assert_eq!(outcome, CloseOutcome::Closed);
    assert_eq!(
        std::fs::read_to_string(s.a()).unwrap(),
        "alpha\nunsaved work\n",
        "the parked buffer's own unsaved text was written before its entry was dropped"
    );
    assert_eq!(s.open_files(), vec![s.b()]);
}

/// A CONFLICTED INACTIVE ENTRY IS REFUSED, NOT DISCARDED — and refused without
/// latching, because the unresolved slot and the recovery record are
/// active-scoped: a conflict latched for a parked path would let the next
/// "Save your version" write the ACTIVE document's bytes over this file.
#[test]
fn a_conflicted_inactive_entry_is_refused_and_nothing_is_lost() {
    let _guard = crate::testlock::serial();
    let mut s = Session::without_autosave("inactive-conflict");
    s.app.load_path(s.a());
    s.app.document.set_text("alpha\nmy version\n");
    s.app.load_path(s.b());
    // SOMEONE ELSE writes A while it is parked.
    std::fs::write(s.a(), "alpha\ntheir version\n").unwrap();

    let outcome = s.app.close_buffer(crate::buffers::BufferKey::path(&s.a()));

    assert_eq!(
        outcome,
        CloseOutcome::Refused,
        "a parked buffer whose file moved under it is never closed"
    );
    assert_eq!(
        s.open_files(),
        vec![s.a(), s.b()],
        "the entry is still open, in its own slot"
    );
    assert_eq!(
        std::fs::read_to_string(s.a()).unwrap(),
        "alpha\ntheir version\n",
        "and the other version on disk was not overwritten"
    );
    assert!(
        !s.app.change_unresolved(),
        "NOTHING WAS LATCHED: a parked conflict must not seize the active \
         document's single unresolved slot"
    );
    let notice = s.app.frame.notice().owned().unwrap_or_default();
    assert!(
        notice.contains("a.txt") && notice.contains("open it to resolve"),
        "the refusal names the file and the way out, got {notice:?}"
    );
    // The refusal is IDEMPOTENT: the baseline was not adopted, so a second
    // attempt re-looks at the disk and refuses on its own evidence.
    assert_eq!(
        s.app.close_buffer(crate::buffers::BufferKey::path(&s.a())),
        CloseOutcome::Refused,
        "re-asking gets the same answer, not a close licensed by the first look"
    );
    assert_eq!(s.open_files(), vec![s.a(), s.b()]);
}

/// A PATH-LESS SCRATCH IS A REAL SUCCESSOR.
///
/// The activation key, rather than a path, is the identity carried across the
/// close. This is the fixture that previously exposed the dead seam:
/// `[scratch, a.txt]`, closing `a.txt`, with only scratch left to activate.
#[test]
fn the_successor_can_activate_the_pathless_scratch_row() {
    let _guard = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-close-scratch-{}", std::process::id())),
    );
    std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
    std::fs::write(dir.join("b.txt"), "beta\n").unwrap();
    let cfg = Config {
        session_restore: Some(false),
        ..Config::empty()
    };
    // Launch with NO file: the active buffer is the path-less scratch surface,
    // and it takes working-set slot 0.
    let mut app = App::new(None, dir.to_path_buf(), None, None, cfg);
    app.load_path(dir.join("a.txt"));
    let working = app.document.working_set();
    assert_eq!(working.len(), 2, "scratch plus the one file");
    assert!(
        working.files()[0].path.is_none(),
        "precondition: the scratch row is slot 0, immediately behind the only file"
    );

    let scratch = working.files()[0].key.clone();
    let only = crate::buffers::BufferKey::path(&dir.join("a.txt"));
    assert_eq!(
        app.document.successor_key(&only),
        Some(scratch),
        "the path-less row remains activatable by registry identity"
    );

    assert!(app.remove_active_entry());
    assert_eq!(
        app.document.working_set().len(),
        1,
        "the file closes while the scratch survives"
    );
    assert!(app.document.buffer().path().is_none());
}

/// CLOSING THE LAST FILE SAVES, NOTIFIES, AND LEAVES NO DOCUMENT.
///
/// A single-file session is the DAEMON's primary shape (`EDITOR=awl` opens one
/// file and waits), so the save and the notification are the load-bearing half
/// here; only the removal is withheld.
#[test]
fn closing_the_last_file_enters_the_honest_zero_document_state() {
    let _guard = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-close-last-{}", std::process::id())),
    );
    std::fs::write(dir.join("only.txt"), "one\n").unwrap();
    let cfg = Config {
        session_restore: Some(false),
        ..Config::empty()
    };
    let mut app = App::new(
        Some(dir.join("only.txt")),
        dir.to_path_buf(),
        None,
        None,
        cfg,
    );
    assert_eq!(app.document.working_set().len(), 1);
    app.document.set_text("one\nedited\n");

    drive_finish_file(&mut app);

    assert_eq!(
        std::fs::read_to_string(dir.join("only.txt")).unwrap(),
        "one\nedited\n",
        "the save half of Finish still runs with nowhere to go afterwards"
    );
    assert_eq!(
        app.document.working_set().len(),
        0,
        "the working set has no invented replacement row"
    );
    assert!(!app.document.has_active());
    assert!(app.document.buffer_opt().is_none());
    assert_eq!(app.project_location.root, dir.to_path_buf());

    let exit = crate::app::schedule::RecordingExit::new();
    app.apply(Action::OpenGoto, false, &exit, crate::stats::Door::Chord);
    assert!(
        app.workspace_state.overlay_open(),
        "Go to remains usable with no document"
    );
    app.apply(Action::Cancel, false, &exit, crate::stats::Door::Chord);
    app.project_location.root = crate::fs::data_root();
    assert_eq!(
        app.prepare_tutorial_action(Action::NewDocument),
        Action::NewDocument,
        "with no document, New remains the exact start action even at the tutorial root"
    );
    assert_eq!(
        app.workspace_state.take_tutorial_folder_intent(),
        None,
        "the zero-document start action leaves no deferred folder intent armed"
    );
    app.apply(Action::NewDocument, false, &exit, crate::stats::Door::Chord);
    assert!(
        app.document.has_active(),
        "New document leaves the empty state"
    );
    assert!(app.document.buffer().is_unnamed_fresh());
}

#[test]
fn zero_document_can_load_a_goto_choice_without_a_departing_buffer() {
    let _guard = crate::testlock::serial();
    let mut s = Session::new("zero-goto-accept");
    let next = s.dir.join("next.txt");
    std::fs::write(&next, "arrived\n").unwrap();
    drive_finish_file(&mut s.app);
    drive_finish_file(&mut s.app);
    assert!(
        !s.app.document.has_active(),
        "precondition: the canvas is empty"
    );

    s.app.load_path(next.clone());

    assert_eq!(s.app.document.buffer().path(), Some(next.as_path()));
    assert_eq!(s.app.document.buffer().text(), "arrived\n");
}

#[test]
fn close_successor_runs_the_same_external_arrival_boundary_as_an_explicit_switch() {
    let _guard = crate::testlock::serial();
    let mut s = Session::new("successor-arrival");
    std::fs::write(s.a(), "changed while parked\n").unwrap();

    drive_finish_file(&mut s.app);

    assert_eq!(s.app.document.buffer().path(), Some(s.a().as_path()));
    assert_eq!(
        s.app.document.buffer().text(),
        "changed while parked\n",
        "the close-selected successor must be rechecked and adopt a clean external change"
    );
}

#[test]
fn pointer_close_and_finish_chord_both_close_after_adopting_a_clean_external_rewrite() {
    let _guard = crate::testlock::serial();
    for door in ["pointer", "chord"] {
        let mut s = Session::new(&format!("external-close-{door}"));
        std::fs::write(s.b(), "disk won cleanly\n").unwrap();
        match door {
            "pointer" => {
                let key = s.app.document.active_key().unwrap();
                assert_eq!(s.app.close_buffer(key), CloseOutcome::Closed);
            }
            "chord" => drive_finish_file(&mut s.app),
            _ => unreachable!(),
        }
        assert_eq!(
            s.app.document.buffer().path(),
            Some(s.a().as_path()),
            "{door} must converge on the same closed successor"
        );
        assert_eq!(
            std::fs::read_to_string(s.b()).unwrap(),
            "disk won cleanly\n",
            "{door} preserves the adopted external bytes"
        );
    }
}

#[test]
fn closing_final_clean_external_rewrite_clears_its_reload_notice() {
    let _guard = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-close-final-reload-{}", std::process::id())),
    );
    let path = dir.join("only.txt");
    std::fs::write(&path, "first\n").unwrap();
    let cfg = Config {
        session_restore: Some(false),
        ..Config::empty()
    };
    let mut app = App::new(Some(path.clone()), dir.to_path_buf(), None, None, cfg);
    std::fs::write(&path, "disk changed cleanly\n").unwrap();
    let key = app.document.active_key().unwrap();

    assert_eq!(app.close_buffer(key), CloseOutcome::Closed);

    assert!(!app.document.has_active());
    assert_eq!(
        app.frame.notice().text(),
        None,
        "the empty canvas must not retain a reload toast for a removed document"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn failed_finish_save_keeps_dirty_document_open_and_never_claims_completion() {
    let _guard = crate::testlock::serial();
    let path = PathBuf::from("/finish-failure/note.txt");
    let inner = crate::fs::InMemoryFs::new()
        .with_dir("/finish-failure")
        .with_file(&path, "old complete bytes\n");
    let _memory = crate::fs::FsGuard::install(Arc::new(inner.clone()));
    let cfg = Config {
        session_restore: Some(false),
        autosave: Some(false),
        ..Config::empty()
    };
    let mut app = App::new(
        Some(path.clone()),
        PathBuf::from("/finish-failure"),
        None,
        None,
        cfg,
    );
    app.document.set_text("dirty text that must survive\n");
    let scripted = Arc::new(crate::fs::ScriptedFs::new(
        inner.clone(),
        crate::fs::ScriptedFailure {
            operation: crate::fs::ScriptedOperation::Write,
            ordinal: 1,
            kind: std::io::ErrorKind::PermissionDenied,
            reason: "finish save denied",
        },
    ));
    let _failure = crate::fs::FsGuard::install(scripted.clone());

    assert_eq!(app.close_active_now(), CloseOutcome::Refused);

    assert_eq!(app.document.buffer().path(), Some(path.as_path()));
    assert!(
        app.document.active_unsaved(),
        "the edit remains dirty and owned"
    );
    assert_eq!(inner.read_to_string(&path).unwrap(), "old complete bytes\n");
    assert!(
        app.frame
            .notice()
            .text()
            .is_some_and(|text| text.starts_with("save failed:")),
        "the refusal is visible rather than reported as completion"
    );
    assert!(
        scripted
            .trace()
            .iter()
            .any(|entry| entry.starts_with("write#1")),
        "the law must enroll the named save failure"
    );
}

/// ⌃TAB DOES NOT RESURRECT A FILE THE READER JUST CLOSED.
#[test]
fn closing_a_file_clears_it_as_the_last_file_target() {
    let _guard = crate::testlock::serial();
    let mut s = Session::new("previous-cleared");
    assert_eq!(
        s.app.document.previous_path(),
        Some(s.a()),
        "precondition: A is the last-file target"
    );

    let outcome = s.app.close_buffer(crate::buffers::BufferKey::path(&s.a()));

    assert_eq!(outcome, CloseOutcome::Closed);
    assert_eq!(
        s.app.document.previous_path(),
        None,
        "the closed file is no longer the Last file target"
    );
}

/// The DAEMON half, which needs a real socket pair and therefore the same cfg
/// gate `crate::daemon` itself carries: under `mas` the daemon module compiles
/// out entirely, and wasm has no unix sockets.
#[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
mod waiters {
    use super::*;
    use std::io::{BufRead, BufReader, Read};
    use std::os::unix::net::UnixStream;

    /// Register a mocked `--wait` client on `key`: a real connected pair, no
    /// listener and no socket file. Returns OUR end of it.
    fn park_waiter(
        app: &mut App,
        path: &std::path::Path,
        key: crate::buffers::BufferKey,
    ) -> UnixStream {
        let (mine, theirs) = UnixStream::pair().expect("unix socketpair");
        app.wait_conns
            .entry(key)
            .or_default()
            .push(crate::daemon::Waiter::new(path.to_path_buf(), theirs));
        mine
    }

    fn assert_done(mut mine: UnixStream, path: &Path) {
        let mut line = String::new();
        BufReader::new(&mut mine).read_line(&mut line).unwrap();
        assert_eq!(line, crate::daemon::format_done(path));
        let mut byte = [0u8; 1];
        assert_eq!(mine.read(&mut byte).unwrap(), 0, "done is followed by EOF");
    }

    fn assert_waiting(mut mine: UnixStream) {
        mine.set_nonblocking(true).unwrap();
        let mut byte = [0u8; 1];
        assert!(
            matches!(mine.read(&mut byte), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock)
        );
    }

    #[test]
    fn trash_notifies_exact_active_and_parked_waiters_only_after_backend_success() {
        let _guard = crate::testlock::serial();
        for parked in [false, true] {
            let mut s = Session::without_autosave(&format!("trash-waiter-{parked}"));
            let path = if parked { s.a() } else { s.b() };
            let key = crate::buffers::BufferKey::path(&path);
            let mine = park_waiter(&mut s.app, &path, key.clone());
            let fake = Arc::new(crate::assets::FakeTrash::default());
            assert_eq!(
                crate::assets::with_trash(fake, || s.app.trash_buffer(key.clone())),
                CloseOutcome::Closed
            );
            assert_done(mine, &path);
            assert!(!s.app.wait_conns.contains_key(&key));
        }
    }

    #[test]
    fn refused_and_backend_failed_trash_leave_the_waiter_connected() {
        struct Fail;
        impl crate::assets::TrashCan for Fail {
            fn trash(&self, _: &Path) -> Result<(), String> {
                Err("denied".into())
            }
        }
        let _guard = crate::testlock::serial();
        for failed_backend in [false, true] {
            let mut s = Session::without_autosave(&format!("trash-waiter-refuse-{failed_backend}"));
            let path = s.b();
            let key = crate::buffers::BufferKey::path(&path);
            if !failed_backend {
                s.app.document.set_text("dirty\n");
            }
            let mine = park_waiter(&mut s.app, &path, key.clone());
            let backend: Arc<dyn crate::assets::TrashCan> = if failed_backend {
                Arc::new(Fail)
            } else {
                Arc::new(crate::assets::FakeTrash::default())
            };
            assert_eq!(
                crate::assets::with_trash(backend, || s.app.trash_buffer(key.clone())),
                CloseOutcome::Refused
            );
            assert_waiting(mine);
            assert!(s.app.wait_conns.contains_key(&key));
            assert_eq!(s.open_files(), vec![s.a(), s.b()]);
            assert!(path.exists());
        }
    }

    /// A `--wait` CLIENT BLOCKED ON A FILE THAT IS NOT ACTIVE is still notified
    /// when that file closes.
    ///
    /// The third of the three missing pieces. The notification used to be
    /// derived from `self.document.buffer()`, so a client blocked on a file the
    /// reader had since switched away from could only be released by switching
    /// back to it first. Keying it makes the question "who is waiting on THIS
    /// file", which is what the client actually asked.
    #[test]
    fn a_daemon_waiter_on_a_parked_file_is_notified_when_that_file_closes() {
        let _guard = crate::testlock::serial();
        let mut s = Session::new("waiter-parked");
        let a = s.a();
        let key = crate::buffers::BufferKey::path(&a);
        let mine = park_waiter(&mut s.app, &a, key.clone());
        assert_eq!(
            s.app.document.buffer().path(),
            Some(s.b().as_path()),
            "precondition: the waited-on file is PARKED, not the active one"
        );

        assert_eq!(s.app.close_buffer(key), CloseOutcome::Closed);

        let mut reader = BufReader::new(mine);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(
            line,
            crate::daemon::format_done(&a),
            "the client blocked on the PARKED file was told about that file"
        );
        let mut rest = String::new();
        assert_eq!(
            reader.read_line(&mut rest).unwrap(),
            0,
            "and its connection closed right after"
        );
        assert!(
            !s.app
                .wait_conns
                .contains_key(&crate::buffers::BufferKey::path(&a)),
            "the notified entry is drained"
        );
    }

    /// A REFUSED ⌘W LEAVES THE WAITER CONNECTED.
    ///
    /// The conflict gate declines to write, so `done` would be a false claim
    /// about the file: the client would proceed with the version on disk, which
    /// is the one the user has not chosen. Asserted by a NON-BLOCKING read that
    /// must report `WouldBlock` — not EOF, which is itself a valid "done" signal
    /// to a real client (dropping a `Waiter` closes its socket), and not data.
    /// Reading for silence is the only assertion that separates all three.
    #[test]
    fn a_held_conflict_leaves_the_daemon_waiter_connected() {
        let _guard = crate::testlock::serial();
        let mut s = Session::without_autosave("waiter-conflict");
        // Make the ACTIVE file conflicted: edited here, moved on disk.
        s.app.document.set_text("beta\nmy version\n");
        std::fs::write(s.b(), "beta\ntheir version\n").unwrap();
        let b = s.b();
        let key = crate::buffers::BufferKey::path(&b);
        let mine = park_waiter(&mut s.app, &b, key);

        drive_finish_file(&mut s.app);

        assert!(
            s.app.change_unresolved(),
            "precondition: the gate latched a conflict rather than writing"
        );
        assert_eq!(
            s.open_files(),
            vec![s.a(), s.b()],
            "the conflicted buffer was not closed"
        );
        assert_eq!(
            std::fs::read_to_string(s.b()).unwrap(),
            "beta\ntheir version\n",
            "and nothing was written over the other version"
        );

        mine.set_nonblocking(true).unwrap();
        let mut buf = [0u8; 64];
        match (&mine).read(&mut buf) {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Ok(0) => {
                panic!("the waiter's connection was CLOSED — a real client reads that as done")
            }
            Ok(n) => panic!(
                "the waiter was told {:?} for a file awl refused to save",
                String::from_utf8_lossy(&buf[..n])
            ),
            Err(e) => panic!("unexpected socket error: {e}"),
        }
    }
}
