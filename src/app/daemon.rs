//! The live App's half of the SINGLE-INSTANCE DAEMON (`crate::daemon` owns the
//! protocol + socket plumbing; this file is the App-specific wiring — reacting
//! to a posted [`crate::daemon::DaemonEvent`], finishing a buffer for
//! `Action::FinishBuffer` (C-x #), and tearing down cleanly on quit). Native
//! only (`cfg(not(target_arch = "wasm32"))`); see `crate::daemon`'s module doc
//! for the full doors-and-capture-gate picture. The daemon-specific doors
//! below are additionally gated `not(feature = "mas")` — under `mas` the
//! daemon module itself compiles out (see `src/mas.rs`'s module doc), so
//! there is no `DaemonEvent`/`Waiter` type left to react to; `finish_buffer`
//! (C-x #'s save+switch, unrelated to the daemon notify half) stays available
//! on every native flavor.

use super::*;

impl App {
    /// A [`crate::daemon::DaemonEvent`] arrived from the accept-loop thread
    /// (posted via `EventLoopProxy::send_event`, so this always runs on the
    /// normal winit thread — no cross-thread `App` access anywhere). Opens the
    /// path exactly like any other file-open (`load_path`, so the multi-buffer
    /// registry does the rest), raises the window, and — for a `wait` client —
    /// registers the connection to be notified when the opened buffer FINISHES.
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
    pub(super) fn handle_daemon_event(&mut self, event: crate::daemon::DaemonEvent) {
        match event {
            crate::daemon::DaemonEvent::OpenPath { path, waiter } => {
                self.load_path(path);
                if let Some(gpu) = self.frame.gpu() {
                    // RAISE the window: bring it to front, and ask the OS for
                    // urgent attention (a mac dock bounce) in case it wasn't
                    // focused — harmless (a no-op-ish nudge) if it already was.
                    gpu.window.focus_window();
                    gpu.window.request_user_attention(Some(
                        winit::window::UserAttentionType::Informational,
                    ));
                    self.request_frame();
                }
                if let Some(w) = waiter {
                    match crate::buffers::BufferKey::of(self.document.buffer()) {
                        Some(key) => {
                            self.wait_conns.entry(key).or_default().push(w);
                        }
                        // A real file-path open always yields a stable key
                        // (`BufferKey::of` only returns `None` for an unnamed,
                        // still-empty note) — but never strand a waiter on the
                        // impossible case: notify it immediately instead of
                        // silently losing it.
                        None => w.notify_done(),
                    }
                }
            }
        }
    }

    /// Live persistence interpreter for Finish file. It runs before the
    /// separately typed daemon-notify and previous-buffer effects.
    ///
    /// Finish is a save AND a switch away, so it is gated twice over by the one
    /// external-change guard: an unresolved file must neither be written nor
    /// left behind. The refusal keeps the buffer and the waiting client exactly
    /// where they are — a `--wait` client parked on a conflicted file waits a
    /// little longer, which is the correct trade against finishing a file with
    /// the wrong version in it.
    pub(super) fn save_finished_buffer(&mut self) {
        match self.settle_external_change() {
            crate::app::files::WritePermission::Clear => {}
            crate::app::files::WritePermission::Reloaded => return,
            crate::app::files::WritePermission::Held => {
                if let Some(path) = self.document.buffer().path().map(|p| p.to_path_buf()) {
                    self.write_recovery_record(&path);
                }
                return;
            }
        }
        let _ = self.document.save();
        self.snapshot_after_save();
        if let Some(p) = self.document.buffer().path().map(|p| p.to_path_buf()) {
            self.document.record_document_saved(
                self.document.buffer().version(),
                crate::external::Seen::at(&p),
            );
            self.emit_notice(crate::actions::NoticeEffect::Clear);
        }
    }

    /// Live daemon interpreter. Headless replay classifies and records this
    /// request but has no socket capability.
    ///
    /// Notifies the waiters on the ACTIVE buffer's key, through the same keyed
    /// owner ([`Self::notify_close_waiters`]) the parked-close route uses — so
    /// there is one answer to "tell whoever is waiting on this file that it is
    /// done", and the old ordering constraint (run me before the buffer swap,
    /// while the finished buffer is still active) dissolves: the key is read
    /// here and the notification cannot address the wrong file.
    pub(super) fn notify_finished_buffer(&mut self) {
        let Some(key) = crate::buffers::BufferKey::of(self.document.buffer()) else {
            return;
        };
        self.notify_close_waiters(&key);
    }

    /// Clean-shutdown teardown (called from `exiting()`): flush every
    /// OUTSTANDING wait connection — dropping a `Waiter` closes its socket,
    /// and a closed connection is exactly as valid a "done" signal to the
    /// client as an explicit one (see `crate::daemon`'s module doc) — then
    /// unlink the socket special file so the NEXT launch binds cleanly rather
    /// than going through the stale-socket reclaim path.
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
    pub(super) fn daemon_shutdown(&mut self) {
        self.wait_conns.clear();
        if let Some(p) = self.daemon_socket_path.take() {
            let _ = std::fs::remove_file(&p);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32"), not(feature = "mas")))]
mod tests {
    use super::*;
    use crate::testscratch::ScratchDir;
    use std::io::{BufRead, BufReader};
    use std::os::unix::net::UnixStream;

    /// Drive `Action::FinishBuffer` through the REAL `actions::apply_transition` seam
    /// against `app.document.buffer()` (mirroring exactly how `App::apply` wires
    /// `ActionCtx`, minus the `ActiveEventLoop` a live keypress carries — no
    /// window/GPU/quit path is exercised by this action), returning the
    /// resulting `Effect`.
    fn drive_finish_buffer(app: &mut App) -> actions::Transition {
        app.transition_for_test(&Action::FinishBuffer, false)
    }

    #[test]
    fn finish_buffer_saves_notifies_the_waiter_and_switches_to_the_previous_buffer() {
        // The full C-x # story at the apply seam: A open, switch to B (so
        // `prev_file == Some(A)`), register a MOCKED daemon waiter on B (a real
        // connected `UnixStream` pair — no socket file needed), edit B, then
        // drive FinishBuffer. Expect: B is saved to disk, the waiter receives
        // exactly `done <B>` and its end of the pair sees EOF right after
        // (the `Waiter` is dropped post-notify), and the active buffer swaps
        // back to A (the "previously-open OTHER buffer").
        // This exercises the REAL native filesystem (no `InMemoryFs`), so hold
        // the shared `fs::TEST_LOCK` — otherwise a concurrently-running test
        // that swaps in a fake FS via `fs::with_fs` could steal our `save()`
        // write into its in-memory backend instead of the real disk. Can't
        // build via `App::new_hermetic` (its injected InMemoryFs would make
        // `Buffer::from_file` find neither real fixture below) — disable
        // session restore explicitly instead, so `apply_session_restore`
        // never reads the developer's real `~/.local/share/awl/session.toml`
        // and parks his real open files into this test's registry.
        let _fs = crate::testlock::serial();
        let dir = ScratchDir::new(
            std::env::temp_dir().join(format!("awl-finish-buffer-test-{}", std::process::id())),
        );
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        std::fs::write(&a, "alpha\n").unwrap();
        std::fs::write(&b, "beta\n").unwrap();

        let cfg = Config {
            session_restore: Some(false),
            ..Config::empty()
        };
        let mut app = App::new(Some(a.clone()), dir.to_path_buf(), None, None, cfg);
        app.load_path(b.clone());
        assert_eq!(
            app.document.buffer().path(),
            Some(b.as_path()),
            "B is active"
        );
        assert_eq!(
            app.document.previous_path(),
            Some(a.clone()),
            "A is the last-buffer target"
        );

        // Mock the waiter: a real connected pair, no listener/socket file.
        let (mine, theirs) = UnixStream::pair().expect("unix socketpair");
        let key = crate::buffers::BufferKey::path(&b);
        app.wait_conns
            .entry(key)
            .or_default()
            .push(crate::daemon::Waiter::new(b.clone(), theirs));

        app.document.set_text("beta\nedited\n");
        let transition = drive_finish_buffer(&mut app);
        assert_eq!(
            &transition.effects()[..3],
            &[
                actions::Effect::Persistence(actions::PersistenceEffect::Save(
                    actions::SaveKind::Finish,
                )),
                actions::Effect::Daemon(actions::DaemonEffect::NotifyFinished),
                actions::Effect::Buffer(actions::BufferEffect::CloseActive),
            ],
            "the core orders finish-save, daemon notification, then buffer switch"
        );
        app.save_finished_buffer();
        app.notify_finished_buffer();
        app.last_buffer_toggle();

        // SAVED: the edit landed on disk.
        assert_eq!(
            std::fs::read_to_string(&b).unwrap(),
            "beta\nedited\n",
            "FinishBuffer must save the buffer, exactly like Action::Save"
        );

        // NOTIFIED: the waiter's peer receives exactly `done <b>` then EOF (the
        // `Waiter` closes its end right after writing).
        let mut reader = BufReader::new(mine);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line, crate::daemon::format_done(&b));
        let mut rest = String::new();
        let n = reader.read_line(&mut rest).unwrap();
        assert_eq!(n, 0, "the waiter closes its end right after notifying");

        // SWITCHED: the active buffer is A again (the previously-open other buffer).
        assert_eq!(
            app.document.buffer().path(),
            Some(a.as_path()),
            "FinishBuffer switches to the previous buffer"
        );
        assert!(
            !app.wait_conns
                .contains_key(&crate::buffers::BufferKey::path(&b)),
            "the notified waiter entry is drained"
        );
    }

    #[test]
    fn daemon_shutdown_drops_every_waiter_and_unlinks_the_socket() {
        // The quit-path teardown: outstanding waiters are dropped (closing
        // their sockets — the client-side "closed counts as done too"
        // contract, proved directly on `Waiter` in `crate::daemon`'s own
        // tests) and the socket special file is unlinked.
        // No real file content is needed from the App itself (only the raw
        // `sock` file below, via plain `std::fs` — untouched by whichever
        // backend `crate::fs::active()` points at), so build hermetically:
        // `App::new_hermetic` closes both the session-restore AND
        // scratch-stash doors (it takes `fs::TEST_LOCK` internally for the
        // scope of construction, so don't ALSO hold it here — a plain
        // `Mutex` isn't reentrant).
        let dir = ScratchDir::new(
            std::env::temp_dir().join(format!("awl-daemon-shutdown-test-{}", std::process::id())),
        );
        let sock = dir.join("awl.sock");
        std::fs::write(&sock, b"").unwrap(); // stand-in for a bound socket file

        let mut app = App::new_hermetic(None, dir.to_path_buf(), Config::empty());
        app.daemon_socket_path = Some(sock.clone());
        let (_mine, theirs) = UnixStream::pair().unwrap();
        app.wait_conns.insert(
            crate::buffers::BufferKey::Scratch,
            vec![crate::daemon::Waiter::new(PathBuf::from("/x"), theirs)],
        );

        app.daemon_shutdown();

        assert!(app.wait_conns.is_empty(), "every waiter is dropped");
        assert!(
            !sock.exists(),
            "the socket file is unlinked on clean shutdown"
        );
        assert!(app.daemon_socket_path.is_none());
    }
}
