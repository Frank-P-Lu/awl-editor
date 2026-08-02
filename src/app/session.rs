//! SESSION RESTORE's App-side wiring (native only — `cfg(not(target_arch =
//! "wasm32"))`, mirroring the single-instance daemon's own gate): the CAPTURE
//! half (`session_flush`, called from the same blur+quit doors the autosave
//! engine's own flush uses, AND eagerly from `switch_project` on every folder
//! switch) and the RESTORE half (`apply_session_restore`, called once from
//! `App::new`). `crate::session` owns the pure data model + (de)serializer +
//! window-frame clamp math; this file is the seam that folds it into the live
//! `App` — the buffer registry, the active buffer, and (on `resumed()`, in
//! `app.rs`) the window frame.
//!
//! **Item 76 — the ONE active-folder-context owner:** `SessionState.root` is
//! now written by `session_flush` ALONGSIDE `active`/`buffers` in the exact
//! same atomic write, so the folder and the active document restored from
//! them can never disagree (the old, retired `config.project_root` was a
//! SEPARATE store that could drift from the session's own `active`). The
//! FOLDER side of the one launch-precedence law is read by
//! `main/run.rs::resolve_launch_context` (via `crate::session::remembered_root`,
//! gated on `Config::session_restore_on()`) BEFORE `App::new` is even called;
//! this file's `apply_session_restore` owns only the DOCUMENT/buffer-registry
//! side, reading the SAME underlying state — both single-read from one file,
//! so they cannot disagree by construction.
//!
//! **Determinism:** headless capture never constructs the live `App` (`main::run::replay_keys`
//! / `load_buffer` build a bare `Buffer` directly), so a capture is
//! STRUCTURALLY incapable of touching the session file — see
//! `main::run::tests::headless_replay_never_touches_the_session_file`.

use super::*;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

impl App {
    /// SESSION FLUSH — the CAPTURE half's one door (mirrors the autosave
    /// engine's `autosave_flush`): snapshot every open PATHED buffer's path +
    /// cursor + scroll (the active one, via `self.document.buffer()` — `Buffer::path()`
    /// is the sole authoritative path, item 56 — plus every backgrounded one
    /// still in the registry), which one is active, and the native window
    /// frame, then write it atomically beside the scratch stash. Config-gated
    /// (`session_restore`, default ON — the SAME flag also gates the restore
    /// half, so turning it off makes the feature vanish both ways); a no-op
    /// when off.
    ///
    /// Called from the SAME two triggers the autosave engine's blur/quit
    /// flushes use (window blur + `exiting()`) — deliberately NOT idle or
    /// file-switch (a TASTE CALL, logged): the open-file SET changes rarely
    /// enough that the coarser two triggers are plenty, and capturing the
    /// window frame on every idle tick / file switch would mean writing it on
    /// every resize-drag frame too. The no-path SCRATCH buffer is never a
    /// member of `buffers` (it keeps its own persistent stash — composing,
    /// not duplicating, per the module doc).
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn session_flush(&mut self) {
        if !self.config.session_restore_on() {
            return;
        }
        let active_path = self.document.buffer().path().map(Path::to_path_buf);
        let buffers = self.document.session_buffers();
        // Best-effort: `outer_position` can fail (e.g. some Wayland compositors
        // refuse it) — degrade to no window frame rather than skip the whole
        // flush, mirroring every other "never let a live-only quirk disrupt the
        // rest of the save" pattern in this codebase.
        let window = self.gpu.as_ref().and_then(|gpu| {
            let pos = gpu.window.outer_position().ok()?;
            let size = gpu.window.inner_size();
            Some(crate::session::WindowFrame {
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
            })
        });
        let state = crate::session::SessionState {
            root: Some(self.project_location.root.clone()),
            active: active_path,
            buffers,
            window,
        };
        if let Err(e) = crate::session::save(&crate::session::session_path(), &state) {
            eprintln!("session save failed: {e}");
        }
    }

    /// SESSION RESTORE's apply half, called ONCE from `App::new` (after the
    /// scratch-stash restore has already picked `self.document.buffer()`). `file_arg_given`
    /// is whether THIS launch named an explicit file:
    ///
    ///  - a BARE launch (`false`): the session's own remembered `active`
    ///    file (if it SURVIVES — still exists on disk) becomes the active
    ///    buffer, its cursor/scroll restored; every OTHER surviving file is
    ///    parked into the buffer registry (backgrounded, cursor/scroll
    ///    restored too). Composes with — never replaces — the scratch-stash
    ///    outcome: a session with no `active` (or an `active` that vanished)
    ///    leaves `self.document.buffer()` exactly as the stash restore left it,
    ///    and its OTHER survivors still get parked.
    ///  - a launch WITH a file argument (`true`, TASTE CALL — logged in
    ///    CLAUDE.md): that file STAYS active (never overridden), but the
    ///    rest of the session still restores BEHIND it into the registry —
    ///    the daemon hands a `--wait`/plain launch off into a long-lived
    ///    instance, so the session belongs to the INSTANCE, not to any one
    ///    launch's argument.
    ///
    /// A vanished file (deleted/moved since the last session) is silently
    /// skipped (`crate::session::existing_buffers`); the kill-switch
    /// (`config.session_restore_on()`) makes this whole function a no-op,
    /// including the window-frame stash into `self.restored_window`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn apply_session_restore(&mut self, file_arg_given: bool) {
        if !self.config.session_restore_on() {
            return;
        }
        let state = crate::session::load(&crate::session::session_path());
        self.restored_window = state.window;
        let survivors = crate::session::existing_buffers(&state);
        if survivors.is_empty() {
            return;
        }
        // A BARE launch may adopt the session's own `active` file (if it
        // survived); a launch WITH a file argument keeps that file active no
        // matter what the session says.
        let active_path = if file_arg_given {
            None
        } else {
            state
                .active
                .as_ref()
                .and_then(|p| survivors.iter().find(|(sp, _)| sp == p).cloned())
        };
        if let Some((path, pos)) = &active_path {
            self.document
                .restore_active(path, *pos, Self::disk_mtime_of(path));
        }
        for (path, pos) in &survivors {
            if active_path.as_ref().map(|(p, _)| p) == Some(path) {
                continue; // just became the active buffer above
            }
            if self.document.buffer().path() == Some(path.as_path()) {
                continue; // already this launch's CLI-argument file
            }
            self.document
                .restore_background(path, *pos, Self::disk_mtime_of(path));
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::Arc;

    /// Build a `SessionState` the way `session_flush` would, sparing tests the
    /// need to fully replicate its exact shape by hand.
    fn state(
        active: Option<&str>,
        buffers: &[(&str, usize, usize, usize)],
    ) -> crate::session::SessionState {
        crate::session::SessionState {
            root: None,
            active: active.map(PathBuf::from),
            buffers: buffers
                .iter()
                .map(|(p, line, col, scroll)| {
                    (
                        PathBuf::from(p),
                        crate::session::BufferPos {
                            line: *line,
                            col: *col,
                            scroll: *scroll,
                            scroll_px_q: 0,
                        },
                    )
                })
                .collect(),
            window: None,
        }
    }

    #[test]
    fn bare_launch_restores_active_and_parks_the_rest() {
        let fake = Arc::new(
            crate::fs::InMemoryFs::new()
                .with_file("/n/a.md", "one\ntwo\nthree\n")
                .with_file("/n/b.md", "alpha\nbeta\n"),
        );
        crate::fs::with_fs(fake, || {
            let session_path = crate::session::session_path();
            let s = state(
                Some("/n/a.md"),
                &[("/n/a.md", 1, 2, 3), ("/n/b.md", 0, 1, 0)],
            );
            crate::session::save(&session_path, &s).unwrap();

            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());

            assert_eq!(
                app.document.buffer().path(),
                Some(Path::new("/n/a.md")),
                "session active file wins"
            );
            assert_eq!(
                app.document.buffer().cursor_line_col(),
                (1, 2),
                "cursor restored"
            );
            assert_eq!(app.document.scroll().row, 3, "scroll restored");
            assert_eq!(
                app.document.open_count() - 1,
                1,
                "the OTHER survivor is parked"
            );
            assert!(
                app.document
                    .contains_background(&crate::buffers::BufferKey::path(Path::new("/n/b.md")))
            );

            // Switching to it finds the restored cursor/scroll, not a fresh 0,0.
            app.load_path(PathBuf::from("/n/b.md"));
            assert_eq!(app.document.buffer().cursor_line_col(), (0, 1));
            assert_eq!(app.document.scroll().row, 0);
        });
    }

    #[test]
    fn file_argument_launch_stays_active_but_restores_the_rest_behind_it() {
        let fake = Arc::new(
            crate::fs::InMemoryFs::new()
                .with_file("/n/a.md", "one\ntwo\n")
                .with_file("/n/b.md", "alpha\nbeta\n"),
        );
        crate::fs::with_fs(fake, || {
            let session_path = crate::session::session_path();
            // The session's own "active" was b.md, but THIS launch names a.md.
            let s = state(Some("/n/b.md"), &[("/n/b.md", 1, 0, 2)]);
            crate::session::save(&session_path, &s).unwrap();

            let app = App::new(
                Some(PathBuf::from("/n/a.md")),
                PathBuf::from("/n"),
                None,
                None,
                Config::empty(),
            );

            assert_eq!(
                app.document.buffer().path(),
                Some(Path::new("/n/a.md")),
                "the CLI file argument wins"
            );
            assert_eq!(
                app.document.buffer().cursor_line_col(),
                (0, 0),
                "the CLI-argument file opens at its own start, not the session's remembered cursor"
            );
            assert_eq!(
                app.document.open_count() - 1,
                1,
                "b.md still restores BEHIND the active file"
            );
            assert!(
                app.document
                    .contains_background(&crate::buffers::BufferKey::path(Path::new("/n/b.md")))
            );
        });
    }

    #[test]
    fn vanished_session_file_is_silently_skipped() {
        let fake = Arc::new(crate::fs::InMemoryFs::new().with_file("/n/keep.md", "x\n"));
        crate::fs::with_fs(fake, || {
            let session_path = crate::session::session_path();
            let s = state(
                Some("/n/gone.md"),
                &[("/n/gone.md", 5, 5, 5), ("/n/keep.md", 0, 0, 0)],
            );
            crate::session::save(&session_path, &s).unwrap();

            let app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());

            // "/n/gone.md" never existed: it must never become active, and must
            // never appear in the registry.
            assert_ne!(app.document.buffer().path(), Some(Path::new("/n/gone.md")));
            assert!(
                !app.document
                    .contains_background(&crate::buffers::BufferKey::path(Path::new("/n/gone.md")))
            );
            // "keep.md" survives and gets parked (it wasn't the session's
            // `active`, which vanished, so it's just a background survivor —
            // and since the session named no SURVIVING active file, the
            // scratch-stash outcome for `self.document.buffer()` stands).
            assert!(
                app.document
                    .contains_background(&crate::buffers::BufferKey::path(Path::new("/n/keep.md")))
            );
        });
    }

    #[test]
    fn kill_switch_off_restores_nothing_and_leaves_no_registry_entries() {
        let fake = Arc::new(
            crate::fs::InMemoryFs::new()
                .with_file("/n/a.md", "one\ntwo\n")
                .with_file("/n/b.md", "alpha\n"),
        );
        crate::fs::with_fs(fake, || {
            let session_path = crate::session::session_path();
            let s = state(
                Some("/n/a.md"),
                &[("/n/a.md", 1, 0, 0), ("/n/b.md", 0, 0, 0)],
            );
            crate::session::save(&session_path, &s).unwrap();

            let cfg = Config {
                session_restore: Some(false),
                ..Config::empty()
            };
            let app = App::new(None, PathBuf::from("/n"), None, None, cfg);

            assert_eq!(
                app.document.buffer().path(),
                None,
                "the kill-switch leaves the plain scratch buffer active"
            );
            assert_eq!(
                app.document.open_count() - 1,
                0,
                "nothing is parked when the switch is off"
            );
            assert_eq!(
                app.restored_window, None,
                "no window frame is restored either"
            );
        });
    }

    #[test]
    fn session_flush_writes_the_active_and_backgrounded_buffers_then_round_trips() {
        let fake = Arc::new(
            crate::fs::InMemoryFs::new()
                .with_file("/n/a.md", "one\ntwo\nthree\n")
                .with_file("/n/b.md", "alpha\nbeta\n"),
        );
        crate::fs::with_fs(fake, || {
            let mut app = App::new(
                Some(PathBuf::from("/n/a.md")),
                PathBuf::from("/n"),
                None,
                None,
                Config::empty(),
            );
            app.document
                .set_cursor(app.document.buffer().line_col_to_char(2, 1));
            app.document
                .set_scroll(crate::render::ScrollPos { row: 7, px_q: 23 });
            app.load_path(PathBuf::from("/n/b.md")); // a.md is now backgrounded

            app.session_flush();

            let saved = crate::session::load(&crate::session::session_path());
            assert_eq!(
                saved.root,
                Some(PathBuf::from("/n")),
                "the ONE owner also carries the folder"
            );
            assert_eq!(saved.active, Some(PathBuf::from("/n/b.md")));
            let a_pos = saved
                .buffers
                .iter()
                .find(|(p, _)| p == Path::new("/n/a.md"))
                .map(|(_, pos)| *pos)
                .expect("a.md was flushed as a backgrounded buffer");
            assert_eq!(
                a_pos,
                crate::session::BufferPos {
                    line: 2,
                    col: 1,
                    scroll: 7,
                    scroll_px_q: 23,
                }
            );
        });
    }

    #[test]
    fn switch_project_eagerly_flushes_the_new_root_not_only_on_blur_or_quit() {
        // item 76: a crash/relaunch right after a folder switch — before any
        // blur/quit ever fires — must still resume the NEW folder, not the
        // pre-switch one. `switch_project` calls `session_flush` itself.
        let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/a").with_dir("/b"));
        crate::fs::with_fs(fake, || {
            let mut app = App::new(None, PathBuf::from("/a"), None, None, Config::empty());
            assert_eq!(
                crate::session::load(&crate::session::session_path()).root,
                None,
                "nothing flushed yet"
            );
            app.switch_project(PathBuf::from("/b"));
            assert_eq!(app.project_location.root, PathBuf::from("/b"));
            assert_eq!(
                crate::session::load(&crate::session::session_path()).root,
                Some(PathBuf::from("/b")),
                "the switch itself flushed the new root — no blur/quit needed"
            );
        });
    }

    #[test]
    fn a_to_b_to_a_restores_folder_and_the_active_documents_view() {
        // The launch-precedence law's document/view half: open A (folder FA),
        // switch to folder FB and open a file there, flush (mirrors blur), then
        // a FRESH App with nothing but the remembered session lands back in FA
        // with FA's file + cursor/scroll intact (the buffer registry round-trip
        // — `switch_project`/`load_path` never discard a buffer, only park it).
        let fake = Arc::new(
            crate::fs::InMemoryFs::new()
                .with_file("/fa/one.md", "aaa\nbbb\nccc\n")
                .with_file("/fb/two.md", "xxx\nyyy\n"),
        );
        crate::fs::with_fs(fake, || {
            let mut app = App::new(
                Some(PathBuf::from("/fa/one.md")),
                PathBuf::from("/fa"),
                None,
                None,
                Config::empty(),
            );
            app.document
                .set_cursor(app.document.buffer().line_col_to_char(2, 1));
            app.document.set_scroll(crate::render::ScrollPos::at_row(5));

            app.switch_project(PathBuf::from("/fb"));
            app.load_path(PathBuf::from("/fb/two.md"));
            app.session_flush();

            // A brand-new App, bare launch: `App::new`'s `root` here stands in
            // for what `resolve_launch_context` would have handed it after
            // reading `crate::session::remembered_root()` (tested standalone in
            // `main::run::tests`) — this test's job is the DOCUMENT/view half.
            let remembered_root = crate::session::remembered_root().unwrap();
            assert_eq!(
                remembered_root,
                PathBuf::from("/fb"),
                "resumes the LAST active folder (FB)"
            );
            let mut app2 = App::new(None, remembered_root, None, None, Config::empty());
            assert_eq!(app2.document.buffer().path(), Some(Path::new("/fb/two.md")));
            assert!(
                app2.document
                    .contains_background(&crate::buffers::BufferKey::path(Path::new("/fa/one.md")))
            );

            // Now actually flip BACK to FA (mirrors a Last-file / Goto back to
            // it) and confirm the FULL previous location — folder + buffer +
            // cursor/scroll — round-trips through the registry, not a fresh re-read.
            app2.load_path(PathBuf::from("/fa/one.md"));
            assert_eq!(
                app2.document.buffer().cursor_line_col(),
                (2, 1),
                "A's cursor survived"
            );
            assert_eq!(app2.document.scroll().row, 5, "A's scroll survived");
        });
    }
}
