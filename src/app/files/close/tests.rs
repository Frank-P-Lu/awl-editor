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
use crate::testscratch::ScratchDir;

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

/// THE SUCCESSOR NEVER LANDS ON THE PATH-LESS SCRATCH ROW.
///
/// The scratch surface IS a working-set member — launching with no file enrols
/// it — so the obvious "take the nearest neighbour" rule can hand the close a
/// slot the one file-open door cannot activate.
///
/// ⚠️ THE ARRANGEMENT IS THE WHOLE LAW, and the obvious one proves nothing.
/// `enrol_active` runs at startup, so the scratch row is ALWAYS slot 0 — which
/// means in any session with two or more files there is a real file between the
/// closing slot and the scratch, and the search finds it without ever reaching
/// the row it is supposed to skip. A mutation that dropped the path requirement
/// entirely stayed green under exactly that fixture. The state that reaches the
/// scratch is the ONE-file session: `[scratch, a.txt]`, closing `a.txt`, where
/// the scratch is the only thing left to search.
#[test]
fn the_successor_skips_the_pathless_scratch_row() {
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

    // THE LOAD-BEARING ARM. Backward from slot 1 reaches ONLY the scratch row,
    // so a search that did not require a path answers with it here and nowhere
    // else in any fixture this app can build.
    let only = crate::buffers::BufferKey::path(&dir.join("a.txt"));
    assert_eq!(
        app.document.successor_path(&only),
        None,
        "the scratch row is not a place the reader can be sent"
    );

    // And end to end: with no successor, the close withholds the removal rather
    // than stranding the reader or dropping the buffer.
    assert!(
        !app.close_active_buffer(),
        "no activatable successor means no removal"
    );
    assert_eq!(
        app.document.working_set().len(),
        2,
        "both rows survive — the file was not dropped to reach the scratch"
    );

    // With a second file open the forward search finds it, so the skip above is
    // a refusal to use the scratch and not a refusal to find anything.
    app.load_path(dir.join("b.txt"));
    assert_eq!(
        app.document.successor_path(&only),
        Some(dir.join("b.txt")),
        "forward search finds the next real file"
    );
}

/// CLOSING THE LAST FILE SAVES AND NOTIFIES BUT REMOVES NOTHING — the
/// zero-document bound, stated as a law so it cannot be crossed by accident.
///
/// A single-file session is the DAEMON's primary shape (`EDITOR=awl` opens one
/// file and waits), so the save and the notification are the load-bearing half
/// here; only the removal is withheld.
#[test]
fn closing_the_last_file_still_saves_and_notifies_but_removes_nothing() {
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
        1,
        "and the entry stays: there is no honest zero-document state yet"
    );
    assert_eq!(
        app.document.buffer().path(),
        Some(dir.join("only.txt").as_path()),
        "the reader is left on the document they had, never on a fake empty buffer"
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
