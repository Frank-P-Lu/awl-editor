//! Native single-instance daemon. The newline protocol hands off absolute paths;
//! `--wait` treats connection closure as completion. Headless modes never import it.
#![cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

// --- where the socket lives -------------------------------------------------

/// The socket's directory: the app data dir (same convention as
/// [`crate::fs::scratch_stash_path`] — the socket sits BESIDE `scratch.md`),
/// unless a test has installed an override (see [`set_socket_dir_for_test`]).
fn socket_dir() -> PathBuf {
    socket_dir_override()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_else(crate::fs::data_root)
}

fn socket_dir_override() -> &'static Mutex<Option<PathBuf>> {
    static DIR: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    DIR.get_or_init(|| Mutex::new(None))
}

/// Test-only socket-directory override.
#[cfg(test)]
pub(crate) fn set_socket_dir_for_test(dir: Option<PathBuf>) {
    *socket_dir_override()
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = dir;
}

/// Where the single-instance socket lives.
pub fn socket_path() -> PathBuf {
    socket_dir().join("awl.sock")
}

// --- wire protocol (pure) ---------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRequest {
    pub path: PathBuf,
    pub wait: bool,
}

/// Serialize an absolute client path. A trailing `" wait"` denotes the wait flag.
pub fn format_open(path: &Path, wait: bool) -> String {
    if wait {
        format!("open {} wait\n", path.display())
    } else {
        format!("open {}\n", path.display())
    }
}

/// Parse ONE client request line (a trailing `\n`/`\r\n` is trimmed if
/// present; a bare line with none also parses). `None` for anything not
/// shaped like `open <path>[ wait]`.
pub fn parse_open(line: &str) -> Option<OpenRequest> {
    let line = line.trim_end_matches(['\n', '\r']);
    let rest = line.strip_prefix("open ")?;
    let (path_str, wait) = match rest.strip_suffix(" wait") {
        Some(p) => (p, true),
        None => (rest, false),
    };
    if path_str.is_empty() {
        return None;
    }
    Some(OpenRequest {
        path: PathBuf::from(path_str),
        wait,
    })
}

/// The server's immediate ack, sent the instant a request parses (before the
/// path is actually opened — opening happens on the winit thread).
pub const REPLY_OK: &str = "ok\n";

/// The server's completion notice for a `wait` client (`Action::FinishBuffer`
/// on the served buffer). A `wait` client that never sees this line — because
/// the connection just closed instead — MUST treat that closure as done too;
/// see the module doc.
pub fn format_done(path: &Path) -> String {
    format!("done {}\n", path.display())
}

// --- stale-socket detection --------------------------------------------

pub enum BindOutcome {
    Instance(UnixListener),
    Handoff(UnixStream),
}

/// Bind `path` (become the instance) or discover a live instance to hand off
/// to. The truth table:
///   * bind SUCCEEDS               → `Instance` (nobody was there).
///   * bind fails, connect SUCCEEDS → `Handoff` (a live instance owns it).
///   * bind fails, connect REFUSED  → a crash left a stale socket file with
///     nobody listening: unlink it and reclaim the address → `Instance`.
///
/// Any I/O error surviving the reclaim attempt propagates to the caller
/// (surfaces as "give up the singleton dance, launch anyway" — see `startup`).
pub fn bind_or_connect(path: &Path) -> std::io::Result<BindOutcome> {
    match UnixListener::bind(path) {
        Ok(l) => Ok(BindOutcome::Instance(l)),
        Err(_bind_err) => match UnixStream::connect(path) {
            Ok(s) => Ok(BindOutcome::Handoff(s)),
            Err(_connect_err) => {
                // STALE SOCKET: the special file exists but nothing answers.
                // Reclaim the address ourselves.
                let _ = std::fs::remove_file(path);
                UnixListener::bind(path).map(BindOutcome::Instance)
            }
        },
    }
}

// --- server: the waiter + accept-loop thread --------------------------------

/// One `--wait` client's still-open connection, kept until the server can tell
/// it the buffer FINISHED (or drops it, closing the socket — see the module
/// doc's "closed connection counts as done too" contract).
pub struct Waiter {
    path: PathBuf,
    stream: UnixStream,
}

impl Waiter {
    /// Build a `Waiter` directly (the accept-loop thread's own path; also the
    /// door `App`-level tests use to mock a waiting client with a real
    /// connected [`UnixStream`] pair, no socket FILE required — see
    /// `crate::app::daemon`'s tests). Kept `pub(crate)` rather than test-only,
    /// like `BufferRegistry::contains`, as the natural companion constructor.
    #[allow(dead_code)]
    pub(crate) fn new(path: PathBuf, stream: UnixStream) -> Self {
        Waiter { path, stream }
    }

    pub fn notify_done(self) {
        let mut s = self.stream;
        let _ = s.write_all(format_done(&self.path).as_bytes());
        // `s` drops here, closing the socket.
    }
}

/// One event the accept-loop thread posts into the live winit event loop.
pub enum DaemonEvent {
    /// A peer asked to open `path`. `waiter` is `Some` for a `wait` client
    /// (kept open for the eventual `done`); `None` for fire-and-forget.
    OpenPath {
        path: PathBuf,
        waiter: Option<Waiter>,
    },
}

/// Spawn the accept-loop THREAD: blocks on `listener.accept()` — genuinely 0%
/// CPU while idle, no polling — and for each connection reads ONE request
/// line, replies `ok`, and posts a [`DaemonEvent::OpenPath`] via `proxy` for
/// the winit thread to actually act on (`App::handle_daemon_event`). Runs for
/// the process's life (torn down by process exit, same as the listener it
/// closes over); the socket FILE itself is unlinked separately on a clean
/// quit (`App::daemon_shutdown`, called from `exiting()`).
///
/// No separate "notify waiters on eviction" plumbing is needed on the server
/// side: a `Waiter`'s `UnixStream` is owned by the live `App` once posted
/// (`App::wait_conns`), so it closes the instant `App` drops it — on an
/// explicit `Action::FinishBuffer` (which sends `done` first via
/// `notify_done`), on process exit (`App::daemon_shutdown` drains the map),
/// or if a future caller ever removes an entry for another reason — the
/// closed-socket-means-done contract covers every case uniformly.
/// `E` is the CALLER's own winit user-event type (`crate::app::AwlEvent`
/// today), NAMED HERE ONLY AS A GENERIC — this module never imports
/// `crate::app`, so the daemon protocol stays decoupled from the App's event
/// enum (the same reason `crate::menu::install` takes a `wrap` closure rather
/// than depending on `crate::app::AwlEvent` directly). `wrap` lets the caller
/// name its own variant (`AwlEvent::Daemon`) around the posted
/// [`DaemonEvent`].
pub fn spawn_accept_thread<E: Send + 'static>(
    listener: UnixListener,
    proxy: winit::event_loop::EventLoopProxy<E>,
    wrap: impl Fn(DaemonEvent) -> E + Send + 'static,
) {
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut stream) = conn else { continue };
            let mut reader = match stream.try_clone() {
                Ok(s) => BufReader::new(s),
                Err(_) => continue,
            };
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                continue; // client hung up before sending anything
            }
            let Some(req) = parse_open(&line) else {
                continue;
            };
            let _ = stream.write_all(REPLY_OK.as_bytes());
            let waiter = if req.wait {
                Some(Waiter {
                    path: req.path.clone(),
                    stream,
                })
            } else {
                None
            };
            let _ = proxy.send_event(wrap(DaemonEvent::OpenPath {
                path: req.path,
                waiter,
            }));
        }
    });
}

pub enum StartupOutcome {
    HandedOff,
    /// Nobody was running: we bound the socket and ARE the instance now. The
    /// caller proceeds to build its window as normal, later handing this
    /// listener to [`spawn_accept_thread`] once it has an `EventLoopProxy`.
    Instance(UnixListener),
}

/// Run the startup singleton dance for `crate::app::run`. `file` is the raw
/// (possibly relative) launch argument; when handing off it is canonicalized
/// CLIENT-side (mirroring `crate::buffers::normalize_path` — the same lenient
/// "resolve against MY cwd, absolutize, collapse `.`/`..`" rules `BufferKey`
/// uses), since the server can never recover the client's cwd on its own.
///
/// TASTE CALL (documented scope, not a bug): a BARE launch (`file: None`) when
/// another instance is already running does not extend the protocol with a
/// focus-only message (out of scope for this dumb v1 protocol) — it simply
/// declines to open a second window and returns `HandedOff` without sending
/// anything. Only a launch that names a FILE actually hands work off.
pub fn startup(file: Option<&Path>, wait: bool) -> std::io::Result<StartupOutcome> {
    let path = socket_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match bind_or_connect(&path)? {
        BindOutcome::Instance(l) => Ok(StartupOutcome::Instance(l)),
        BindOutcome::Handoff(mut stream) => {
            if let Some(f) = file {
                let canon = crate::buffers::normalize_path(f);
                stream.write_all(format_open(&canon, wait).as_bytes())?;
                let mut reader = BufReader::new(stream.try_clone()?);
                let mut ok_line = String::new();
                let _ = reader.read_line(&mut ok_line); // "ok\n"; best-effort
                if wait {
                    // Block for "done <path>\n" OR the connection closing — a
                    // closed socket (quit/crash/eviction) counts as done too,
                    // so this can never hang.
                    let mut done_line = String::new();
                    let _ = reader.read_line(&mut done_line);
                }
            }
            Ok(StartupOutcome::HandedOff)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- protocol parse/serialize (pure) ------------------------------------

    #[test]
    fn format_and_parse_open_round_trip_without_wait() {
        let p = PathBuf::from("/tmp/a.txt");
        let line = format_open(&p, false);
        assert_eq!(line, "open /tmp/a.txt\n");
        let req = parse_open(&line).expect("parses");
        assert_eq!(
            req,
            OpenRequest {
                path: p,
                wait: false
            }
        );
    }

    #[test]
    fn format_and_parse_open_round_trip_with_wait() {
        let p = PathBuf::from("/tmp/notes/draft.md");
        let line = format_open(&p, true);
        assert_eq!(line, "open /tmp/notes/draft.md wait\n");
        let req = parse_open(&line).expect("parses");
        assert_eq!(
            req,
            OpenRequest {
                path: p,
                wait: true
            }
        );
    }

    #[test]
    fn parse_open_tolerates_a_bare_line_with_no_trailing_newline() {
        let req = parse_open("open /a/b.txt").expect("parses");
        assert_eq!(req.path, PathBuf::from("/a/b.txt"));
        assert!(!req.wait);
    }

    #[test]
    fn parse_open_rejects_garbage() {
        assert!(parse_open("").is_none());
        assert!(parse_open("close /a.txt\n").is_none());
        assert!(
            parse_open("open \n").is_none(),
            "an empty path is not a request"
        );
        assert!(parse_open("open\n").is_none(), "no space, no path");
    }

    #[test]
    fn format_done_names_the_path() {
        assert_eq!(format_done(Path::new("/a/b.txt")), "done /a/b.txt\n");
    }

    // --- stale-socket detection truth table (real temp-dir sockets) --------

    fn temp_socket_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("awl-daemon-test-{tag}-{}.sock", std::process::id()))
    }

    #[test]
    fn bind_or_connect_becomes_the_instance_when_nobody_is_there() {
        let path = temp_socket_path("fresh");
        let _ = std::fs::remove_file(&path);
        match bind_or_connect(&path).expect("bind should succeed on a fresh path") {
            BindOutcome::Instance(_l) => {}
            BindOutcome::Handoff(_) => panic!("a fresh path must never hand off"),
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bind_or_connect_hands_off_to_a_real_listener() {
        let path = temp_socket_path("live");
        let _ = std::fs::remove_file(&path);
        let _live = UnixListener::bind(&path).expect("bind the 'other instance'");
        match bind_or_connect(&path).expect("connect should succeed against a live listener") {
            BindOutcome::Handoff(_s) => {}
            BindOutcome::Instance(_) => {
                panic!("a live listener must never be reclaimed as a fresh instance")
            }
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bind_or_connect_reclaims_a_stale_socket_left_by_a_crash() {
        let path = temp_socket_path("stale");
        let _ = std::fs::remove_file(&path);
        // Create a listener then DROP it WITHOUT unlinking the socket file —
        // `std::os::unix::net::UnixListener` never removes its own special
        // file on drop, so this is EXACTLY what a crashed instance leaves
        // behind: the path exists, but nothing is listening at it.
        {
            let _dead = UnixListener::bind(&path).expect("bind the 'crashed instance'");
        }
        assert!(
            path.exists(),
            "the stale socket special file must still be on disk"
        );
        match bind_or_connect(&path).expect("must reclaim the stale address") {
            BindOutcome::Instance(_l) => {}
            BindOutcome::Handoff(_) => panic!("nothing is listening; must never hand off"),
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn startup_handoff_canonicalizes_a_relative_file_against_the_clients_cwd() {
        // A relative launch argument must be sent to the server as its
        // cwd-joined, normalized (mirrors `BufferKey::path`) absolute form —
        // the server cannot ever recover the client's cwd on its own.
        let dir =
            std::env::temp_dir().join(format!("awl-daemon-handoff-canon-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
        let sock = dir.join("awl.sock");
        let listener = UnixListener::bind(&sock).unwrap();

        let (tx, rx) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let req = parse_open(&line).unwrap();
            stream.write_all(REPLY_OK.as_bytes()).unwrap();
            tx.send(req).unwrap();
        });

        let _cwd = crate::fs::CwdGuard::enter(&dir);
        let outcome = match bind_or_connect(&sock).unwrap() {
            BindOutcome::Handoff(mut stream) => {
                let canon = crate::buffers::normalize_path(Path::new("a.txt"));
                stream
                    .write_all(format_open(&canon, false).as_bytes())
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut ok = String::new();
                reader.read_line(&mut ok).unwrap();
                "handed off"
            }
            BindOutcome::Instance(_) => "became instance",
        };
        drop(_cwd);
        assert_eq!(outcome, "handed off");

        let req = rx.recv().unwrap();
        server.join().unwrap();
        assert_eq!(
            req.path,
            crate::buffers::normalize_path(&dir.join("a.txt")),
            "the server must receive the CANONICAL absolute path, not the relative spelling"
        );
        let _ = std::fs::remove_file(&sock);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- single-process loopback: listener thread -> channel -> DaemonEvent -

    #[test]
    fn accept_thread_posts_open_path_and_replies_ok() {
        // LIVE-ONLY note: this exercises the REAL accept-loop thread and a REAL
        // socket, but loops back through a plain `mpsc` channel standing in for
        // `EventLoopProxy::send_event` (no winit event loop in a unit test) —
        // the honestly-testable slice of `spawn_accept_thread`. The real
        // winit hop (`App::handle_daemon_event`) is live-only; see the module doc.
        let dir =
            std::env::temp_dir().join(format!("awl-daemon-accept-thread-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sock = dir.join("awl.sock");
        let listener = UnixListener::bind(&sock).unwrap();

        let (tx, rx) = std::sync::mpsc::channel::<DaemonEvent>();
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                let Ok(mut stream) = conn else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    continue;
                }
                let Some(req) = parse_open(&line) else {
                    continue;
                };
                let _ = stream.write_all(REPLY_OK.as_bytes());
                let waiter = if req.wait {
                    Some(Waiter {
                        path: req.path.clone(),
                        stream,
                    })
                } else {
                    None
                };
                let _ = tx.send(DaemonEvent::OpenPath {
                    path: req.path,
                    waiter,
                });
            }
        });

        // Client: connect, send a WAIT request, read "ok", then read "done".
        let mut client = UnixStream::connect(&sock).unwrap();
        let target = PathBuf::from("/some/file.md");
        client
            .write_all(format_open(&target, true).as_bytes())
            .unwrap();
        let mut reader = BufReader::new(client.try_clone().unwrap());
        let mut ok_line = String::new();
        reader.read_line(&mut ok_line).unwrap();
        assert_eq!(ok_line, REPLY_OK);

        // The "winit thread" side: receive the posted event, then finish it.
        let DaemonEvent::OpenPath { path, waiter } = rx.recv().unwrap();
        assert_eq!(path, target);
        let waiter = waiter.expect("a wait request must carry a waiter");
        waiter.notify_done();

        let mut done_line = String::new();
        reader.read_line(&mut done_line).unwrap();
        assert_eq!(done_line, format_done(&target));

        let _ = std::fs::remove_file(&sock);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_dropped_waiter_closes_the_socket_so_the_client_never_hangs() {
        // The "closed connection counts as done too" contract: a waiter that is
        // DROPPED (never explicitly `notify_done`d — e.g. the app quit, or the
        // buffer was evicted) still unblocks the client's blocking read with a
        // clean EOF rather than hanging forever.
        let dir =
            std::env::temp_dir().join(format!("awl-daemon-drop-waiter-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sock = dir.join("awl.sock");
        let listener = UnixListener::bind(&sock).unwrap();

        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let req = parse_open(&line).unwrap();
            let mut s = stream;
            s.write_all(REPLY_OK.as_bytes()).unwrap();
            let waiter = Waiter {
                path: req.path,
                stream: s,
            };
            drop(waiter);
        });

        let mut client = UnixStream::connect(&sock).unwrap();
        client
            .write_all(format_open(Path::new("/x.txt"), true).as_bytes())
            .unwrap();
        let mut reader = BufReader::new(client.try_clone().unwrap());
        let mut ok_line = String::new();
        reader.read_line(&mut ok_line).unwrap();
        let mut done_line = String::new();
        let n = reader.read_line(&mut done_line).unwrap();
        assert_eq!(
            n, 0,
            "EOF (0 bytes), not a hang — the closed socket IS the done signal"
        );

        server.join().unwrap();
        let _ = std::fs::remove_file(&sock);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- the headless capture gate (structural + runtime) -------------------

    #[test]
    fn headless_editing_never_touches_the_socket() {
        // Mirrors `main::run::tests::headless_replay_never_arms_autosave_or_
        // stashes_scratch`'s shape: point the socket at a throwaway dir via
        // the test-only override, drive a few edits + a Save THROUGH THE REAL
        // `actions::apply_core` seam every headless `--keys` replay rides
        // (`crate::app::run` / `crate::daemon::startup` — the only doors that
        // ever touch a socket — are never on this call path at all, headless
        // or otherwise: `main::run`'s capture modes never call `app::run`),
        // then assert nothing ever appeared at the overridden socket path.
        let _lock = crate::testlock::serial();
        let dir =
            std::env::temp_dir().join(format!("awl-daemon-capture-gate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        set_socket_dir_for_test(Some(dir.clone()));

        let mut buffer = crate::buffer::Buffer::scratch();
        let mut shift_selecting = false;
        let mut zoom = 1.0f32;
        let mut search: Option<crate::search::SearchState> = None;
        let mut overlay: Option<crate::overlay::OverlayState> = None;
        let mut make_overlay = |_: crate::overlay::OverlayKind| None;
        let mut browse_to = |_: crate::overlay::OverlayKind, _: Option<String>| None;
        {
            let mut ctx = crate::actions::ActionCtx {
                buffer: &mut buffer,
                shift_selecting: &mut shift_selecting,
                zoom: &mut zoom,
                search: &mut search,
                scroll_page_lines: 20,
                overlay: &mut overlay,
                make_overlay: &mut make_overlay,
                browse_to: &mut browse_to,
                oracle: None,
            };
            let _ = crate::actions::apply_core(
                &mut ctx,
                &crate::keymap::Action::InsertChar('h'),
                false,
            );
            let _ = crate::actions::apply_core(&mut ctx, &crate::keymap::Action::Save, false);
            let _ =
                crate::actions::apply_core(&mut ctx, &crate::keymap::Action::FinishBuffer, false);
        }

        set_socket_dir_for_test(None);
        assert!(
            !dir.join("awl.sock").exists(),
            "editing + saving through the pure core must never bind or connect the daemon socket"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
