//! THE FS-SERIALIZATION LAWS (queue item 101) — the structural rules that
//! retired the `resolve_launch_context_dir_argument_awl_dot_is_explicit_not_remembered`
//! flake, each tested at the purest seam it reaches.
//!
//! THE FLAKE. `run::resolve_root` decides "is this launch argument a
//! directory?" by asking the process-GLOBAL filesystem backend —
//! `crate::fs::active().is_dir(f)` — and took no
//! [`crate::testlock::serial`] guard doing it; neither did its callers. The
//! WRITER side was already disciplined: [`crate::fs::FsGuard`] /
//! [`crate::fs::CwdGuard`] take the guard internally, for their whole life. So
//! under parallel `cargo test` the arrangement was an UNGUARDED READER against
//! a DISCIPLINED WRITER — which is not a lesser bug, it is the same race
//! observed from the other end. A sibling test holding an
//! [`crate::fs::InMemoryFs`] answered the probe: the fake knows nothing of a
//! real temp dir, `is_dir` came back `false`, the dir argument decayed to its
//! PARENT (`/tmp`), and the assertion failed. Measured on the unfixed tree with
//! a flipper thread cycling `FsGuard::install`: **389/400 = 97.2%** of probes
//! read the wrong backend. It reached the merge train as roughly one failure in
//! five doubled-load suites — rare enough to teach everyone to re-run until
//! green, which is what makes a flaky gate corrosive.
//!
//! THE THREE LAYERS, none of them a retry, a tolerance, or an `#[ignore]`:
//!
//!   1. **THE CAUSE** — `let _tg = crate::testlock::serial();` on the victim.
//!   2. **THE CLASS** — [`crate::fs::assert_fs_is_serialized`] fires inside
//!      `fs::active()` / `fs::set_active()`, the ONE door every fs read and
//!      write in the tree passes through. The convention became a LAW: an
//!      unguarded fs touch in a test build panics immediately and by name
//!      instead of statistically failing someone else's merge. Turning it on
//!      immediately found THIRTEEN more unguarded tests across five modules
//!      (`updates` ×5, `run` ×5, `buffers`, `config`, `scenario`) — the class
//!      was far wider than the symptom.
//!   3. **THE SIBLING GLOBAL** — `src/fs.rs` owns a second process-global with
//!      exactly this shape: the process CWD, whose writer `CwdGuard` was
//!      likewise already disciplined while its readers were not. It now has the
//!      same single guarded door, [`crate::fs::current_dir`], and
//!      [`no_cwd_reader_outside_the_one_door`] keeps it the only one. That
//!      routing guarded three more tests — two of which read the cwd TWICE and
//!      compared the answers — and put `capture_mode_bare_invocation_…`'s
//!      hand-rolled save/chdir/restore back under `CwdGuard`, where a failing
//!      assertion no longer strands every sibling test in a temp dir.
//!
//! And one atomicity repair alongside them: `FsGuard::install(fs::active())`,
//! the idiom three tests entered the hermetic sandbox through, is a torn
//! read-modify-write of the global — see [`fs_guard_capture_reads_prev_inside_its_own_lock`].
//!
//! In a release/live build every check compiles to nothing: the live app
//! installs its backend once at startup and is single-threaded over these
//! globals.

use crate::fs::{self, FileSystem, InMemoryFs};
use crate::testscratch::ScratchDir;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// The process-globals `src/fs.rs` OWNS — the sweep axis of this module. Each
/// has the same shape (a swappable process-global, an RAII writer guard that
/// takes [`crate::testlock::serial`], and readers scattered across the tree),
/// so each needs the same guarded reader door.
///
/// A new global added to `fs.rs` must be added here: [`Self::read`]'s
/// match is deliberately WILDCARD-FREE, so a new variant fails to COMPILE
/// rather than silently skipping the sweep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FsGlobal {
    /// The active [`FileSystem`] backend — `fs::active()` / `fs::set_active()`,
    /// written by `fs::FsGuard`.
    Backend,
    /// The process working directory — `fs::current_dir()`, written by
    /// `fs::CwdGuard` (via `std::env::set_current_dir`).
    Cwd,
}

/// Every [`FsGlobal`]. The sweep below runs each one; a new variant that is not
/// added here still fails [`FsGlobal::read`]'s wildcard-free match at compile
/// time, so the two cannot silently drift apart.
pub(crate) const ALL_FS_GLOBALS: [FsGlobal; 2] = [FsGlobal::Backend, FsGlobal::Cwd];

impl FsGlobal {
    /// Perform this global's ordinary READ through its production door. The
    /// match is WILDCARD-FREE on purpose (see [`FsGlobal`]).
    fn read(self) {
        match self {
            FsGlobal::Backend => {
                let _ = fs::active().exists(std::path::Path::new("/"));
            }
            FsGlobal::Cwd => {
                let _ = fs::current_dir();
            }
        }
    }

    /// The door name the law's panic message must carry, so a reader who hits
    /// it learns WHICH global they touched off-guard.
    fn door(self) -> &'static str {
        match self {
            FsGlobal::Backend => "active()",
            FsGlobal::Cwd => "current_dir()",
        }
    }
}

/// LAW 1, THE SWEEP: EVERY process-global `fs.rs` owns rejects an unguarded
/// read and accepts a guarded one — the whole axis, not just the backend the
/// flake happened to expose.
///
/// `testlock`'s hold flag is THREAD-LOCAL, so "a caller without the guard" is
/// modelled exactly by a spawned thread — which is also the real shape of the
/// race (the fs-installing test and the resolving test were two `cargo test`
/// worker threads). We hold the guard for the whole window, so the spawned
/// thread cannot acquire it behind our back either.
#[test]
fn every_fs_global_rejects_an_unguarded_reader() {
    let _tg = crate::testlock::serial();
    for g in ALL_FS_GLOBALS {
        // Guarded (this thread): the door is an ordinary read.
        g.read();

        // Unguarded (another thread): a hard error, naming the door.
        let outcome = std::thread::spawn(move || g.read()).join();
        let err = outcome.unwrap_err();
        let msg = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&'static str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(
            msg.contains("fs law:") && msg.contains(g.door()),
            "{g:?} must panic naming its own door `{}` — got: {msg}",
            g.door()
        );
    }
}

/// LAW 1, WIRED: the check is actually ON the real production path, not merely
/// defined beside it. `run::resolve_root` — the function the flake lived in —
/// driven off-guard with a DIR argument must panic, proving it cannot reach the
/// global without passing the door. Both of its reads are covered: the
/// `fs::active().is_dir(f)` probe (dir/file argument) and the `fs::current_dir()`
/// fallback (bare argument).
#[test]
fn the_real_resolve_root_enforces_the_law() {
    let _tg = crate::testlock::serial();
    let dir = std::env::temp_dir();

    let probe = std::thread::spawn(move || {
        // The exact call the victim test makes: a DIR argument, resolved
        // through `fs::active().is_dir`.
        crate::run::resolve_root(&None, &Some(dir))
    });
    assert!(
        probe.join().is_err(),
        "`resolve_root` with a dir argument reads `fs::active()`; off-guard that must panic"
    );

    let bare = std::thread::spawn(|| crate::run::resolve_root(&None, &None));
    assert!(
        bare.join().is_err(),
        "`resolve_root`'s bare fallback reads the process CWD; off-guard that must panic too"
    );
}

/// LAW 2: the guard actually EXCLUDES the writer — the property the whole fix
/// rests on, measured the same way the bug was.
///
/// This is the permanent form of the reproduction harness. On the unfixed tree
/// the identical loop, run WITHOUT the guard against this same flipper, read
/// the wrong backend 389 times in 400 (97.2%). Holding the guard, the flipper
/// cannot install anything for the whole window, so the answer is stable: 0
/// mismatches. The counter proves the flipper genuinely TRIED (it is not a
/// thread that quietly did nothing) and that every one of its installs lands
/// only after we release.
#[test]
fn a_guarded_reader_never_sees_a_concurrent_backend_swap() {
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-fs-law-{}", std::process::id())));

    let attempts = Arc::new(AtomicUsize::new(0));
    let installs = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));

    let guard = crate::testlock::serial();

    let flipper = {
        let (attempts, installs, stop) = (attempts.clone(), installs.clone(), stop.clone());
        std::thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) {
                attempts.fetch_add(1, Ordering::SeqCst);
                // The WRITER side, exactly as an fs-touching test drives it.
                let g = fs::FsGuard::install(Arc::new(InMemoryFs::new()));
                installs.fetch_add(1, Ordering::SeqCst);
                drop(g);
            }
        })
    };

    // Make the contention REAL before measuring: wait until the flipper has
    // actually reached its acquire, so this law can never pass by racing ahead
    // of a thread that never started.
    let spun_up = std::time::Instant::now();
    while attempts.load(Ordering::SeqCst) == 0 {
        std::thread::yield_now();
        assert!(
            spun_up.elapsed() < std::time::Duration::from_secs(5),
            "the flipper thread never reached its acquire"
        );
    }

    let rounds = 400;
    let mut mismatches = 0;
    for _ in 0..rounds {
        // The victim's exact question: is this REAL directory a directory?
        if !fs::active().is_dir(&dir) {
            mismatches += 1;
        }
        std::hint::spin_loop();
    }
    assert_eq!(
        mismatches, 0,
        "a reader holding the guard must never be answered by a concurrently installed \
         backend — 0 of {rounds}, against 389 of 400 on the unfixed tree (queue item 101)"
    );
    // The flipper reached its acquire (so it was really contending) and got
    // NOWHERE past it while we held the guard.
    assert!(
        attempts.load(Ordering::SeqCst) >= 1,
        "the flipper must have tried, or this law proves nothing"
    );
    assert_eq!(
        installs.load(Ordering::SeqCst),
        0,
        "no install may complete inside the guarded window"
    );

    stop.store(true, Ordering::SeqCst);
    drop(guard);
    flipper.join().unwrap();
}

/// LAW 2's other end: an `FsGuard` restores THE backend that was installed
/// under its own lock, never one it read a moment before acquiring.
///
/// `FsGuard::install(fs::active())` — the idiom three tests used to enter the
/// hermetic sandbox — evaluates its argument BEFORE `install` takes the guard,
/// so the "previous" backend it memorizes is whatever happened to be installed
/// in that unlocked instant. [`fs::FsGuard::capture`] reads it INSIDE the
/// locked window instead: one acquisition, one read, one truth.
#[test]
fn fs_guard_capture_reads_prev_inside_its_own_lock() {
    let _tg = crate::testlock::serial();
    let sentinel: Arc<dyn FileSystem> = Arc::new(InMemoryFs::new().with_file("/sentinel", "x"));
    let outer = fs::FsGuard::install(sentinel);
    assert!(fs::active().exists(std::path::Path::new("/sentinel")));
    {
        // A production door (`scenario::install_hermetic_fs`) swaps the backend
        // under us; `capture()` must put back the SENTINEL, i.e. what was
        // installed when it locked.
        let _restore = fs::FsGuard::capture();
        fs::set_active(Arc::new(InMemoryFs::new()));
        assert!(
            !fs::active().exists(std::path::Path::new("/sentinel")),
            "the inner swap really took effect"
        );
    }
    assert!(
        fs::active().exists(std::path::Path::new("/sentinel")),
        "capture() restored the backend live under its own lock"
    );
    drop(outer);
}

/// LAW 3 (the sibling global, source-audited): `fs::current_dir` is the ONE
/// reader of the process CWD. A raw `std::env::current_dir()` anywhere else in
/// `src/` is exactly the unguarded reader this item retired — invisible to the
/// runtime check precisely because it bypasses the door — so it is pinned here
/// instead, in the shape `println_audit` already uses for its own bypass class.
///
/// `src/fs.rs` is the sole allowed file: it holds the door itself, `CwdGuard`'s
/// save/restore (the WRITER, which must use `std::env` directly), and the doc
/// prose naming the bypass. This law file assembles its needle from FRAGMENTS
/// so the scanner does not match its own source (`durable.rs`'s trick).
#[test]
fn no_cwd_reader_outside_the_one_door() {
    let needle = concat!("std::env::", "current_dir");
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();
    scan(&root, &root, needle, &mut offenders);
    offenders.sort();
    assert_eq!(
        offenders,
        vec!["fs.rs".to_string()],
        "the process CWD has exactly ONE reader door, `crate::fs::current_dir()` — it \
         carries the serialization law's check, and a raw `std::env` read bypasses it \
         silently (queue item 101). Route the call through `fs::current_dir()`; if the \
         site genuinely owns the global (a new writer beside `CwdGuard`), it belongs in \
         `src/fs.rs` with the rest of them."
    );
}

/// Recursive `.rs` walk collecting repo-relative paths whose text contains
/// `needle`. Skips this file (its own doc + `concat!` fragments describe the
/// needle) — the self-match `println_audit` solves the same way.
fn scan(base: &std::path::Path, dir: &std::path::Path, needle: &str, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan(base, &path, needle, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.ends_with("fs/serialization_law.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.contains(needle) {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy();
            out.push(rel.replace('\\', "/"));
        }
    }
}
