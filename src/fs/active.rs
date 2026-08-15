use super::{FileSystem, NativeFs};
use std::io;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

fn global() -> &'static RwLock<Arc<dyn FileSystem>> {
    use std::sync::OnceLock;
    static FS: OnceLock<RwLock<Arc<dyn FileSystem>>> = OnceLock::new();
    FS.get_or_init(|| RwLock::new(Arc::new(NativeFs)))
}

/// THE FS-BACKEND-SERIALIZATION LAW. The active backend is
/// process-GLOBAL and SWAPPABLE, and `cargo test` runs in parallel: while one
/// test has an [`crate::fs::InMemoryFs`] installed via [`FsGuard`], EVERY
/// other thread's
/// `fs::active()` returns that fake — so a sibling test reading the real disk
/// gets "file not found" and a sibling asserting on a fake gets the real disk.
/// The `RwLock` makes each individual read atomic; it does nothing about WHICH
/// backend is installed when the read lands.
///
/// The WRITER side was already disciplined ([`FsGuard`] / [`CwdGuard`] take
/// [`crate::testlock::serial`] internally, for their whole life). The missing
/// discipline was on the READER: `main::run::resolve_root` consulted
/// `active().is_dir(f)` with no guard, and its callers took none either, so
/// `run::tests::resolve_launch_context_dir_argument_awl_dot_is_explicit_not_remembered`
/// silently resolved a REAL temp dir against a sibling test's in-memory fake —
/// `is_dir` came back false, the dir argument decayed to its PARENT, and the
/// test failed under parallel load. An unguarded reader against a disciplined
/// writer is not a smaller bug than an unguarded writer; it is the same race
/// from the other end.
///
/// So the guard is required at THE one door every reader and writer passes
/// through — [`active`] and [`set_active`] — turning the convention into a LAW:
/// an unguarded fs touch in a test build panics IMMEDIATELY and by name on its
/// first run, instead of statistically failing someone else's merge. In a
/// release/live build it compiles to nothing (the live app installs its backend
/// once at startup and is single-threaded over this global).
///
/// Law-tested in [`crate::fs::serialization_law`] — both that the check rejects
/// an unguarded caller and that it is WIRED into the real `resolve_root` path.
#[inline]
pub(crate) fn assert_fs_is_serialized(door: &str) {
    #[cfg(test)]
    assert!(
        crate::testlock::currently_held(),
        "fs law: `fs::{door}` in a test build must be reached while holding \
         `crate::testlock::serial()` — the active backend is a process-global that \
         `FsGuard` swaps out from under every other thread, so an unguarded fs touch \
         races every fs-installing test and reads a backend it did not choose (queue \
         item 101). Add `let _tg = crate::testlock::serial();` as the first line of \
         the test (or use `fs::with_fs` / `fs::FsGuard`, which take it for you)."
    );
    #[cfg(not(test))]
    let _ = door;
}

/// The ACTIVE filesystem backend. Production code routes EVERY file op through this
/// (`fs::active().read_to_string(p)`), so swapping the global swaps the backend
/// everywhere. Returns an `Arc` clone (cheap) so the caller holds no lock across
/// the actual I/O.
///
/// In a TEST build this is the enforcement point of the fs-serialization law
/// ([`assert_fs_is_serialized`]): reading the global off-guard is a hard error.
pub fn active() -> Arc<dyn FileSystem> {
    assert_fs_is_serialized("active()");
    global().read().unwrap().clone()
}

/// Install `fs` as the active backend. Three callers: the wasm entrypoint
/// ([`install_web_fs`]), the HERMETIC SCENARIO door (`crate::scenario` — a
/// strict replay swaps in a seeded [`crate::fs::InMemoryFs`] once, at startup,
/// before any
/// other fs consumer runs), and tests (via [`with_fs`] / [`FsGuard`]).
///
/// The writer half of the fs-serialization law ([`assert_fs_is_serialized`]):
/// in a test build, installing a backend off-guard is a hard error too.
pub fn set_active(fs: Arc<dyn FileSystem>) {
    assert_fs_is_serialized("set_active()");
    *global().write().unwrap() = fs;
}

/// THE ONE READER of the process CWD — the SIBLING global this module owns
/// (its writer, [`CwdGuard`], has always taken [`crate::testlock::serial`];
/// `std::env::set_current_dir` has no other caller in the tree). Same shape,
/// same law as [`active`]: the cwd is process-global, a `CwdGuard` moves it out
/// from under every other thread, and a reader that does not hold the guard can
/// be answered from a directory it did not choose — or, worse, read it TWICE
/// and get two different answers (`buffers::tests::
/// buffer_key_path_normalizes_a_relative_path_against_the_cwd` compares a bare
/// `current_dir()` against the one `normalize_path` takes internally; a chdir
/// landing between them makes the two disagree).
///
/// Routing every read through here gives the cwd the same single guarded door
/// the fs backend has, and `fs::serialization_law::no_cwd_reader_outside_the_one_door`
/// keeps it the ONLY one: `std::env::current_dir()` appears nowhere else in
/// `src/`.
pub(crate) fn current_dir() -> io::Result<PathBuf> {
    assert_fs_is_serialized("current_dir()");
    std::env::current_dir()
}

/// Run `body` with `fs` installed as the active backend, restoring the previous
/// backend (normally [`NativeFs`]) afterwards — so an fs-touching test runs against
/// the fake without leaking it into sibling tests. Holds the shared
/// [`crate::testlock`] guard for the duration. Test-only.
#[cfg(test)]
pub(crate) fn with_fs<T>(fs: Arc<dyn FileSystem>, body: impl FnOnce() -> T) -> T {
    let _guard = FsGuard::install(fs);
    body()
}

/// An RAII alternative to [`with_fs`] for a MULTI-STATEMENT test that can't easily
/// wrap its whole body in a closure (e.g. a setup helper that returns a fake +
/// keeps it installed for the rest of the test). Holds the shared
/// [`crate::testlock`] guard and restores the previous backend when dropped.
/// Test-only.
#[cfg(test)]
pub(crate) struct FsGuard {
    _lock: crate::testlock::SerialGuard,
    prev: Arc<dyn FileSystem>,
}

#[cfg(test)]
impl FsGuard {
    /// Install `fs` as the active backend, returning a guard that restores the
    /// prior backend on drop. The shared [`crate::testlock`] guard is held for the guard's life.
    pub(crate) fn install(fs: Arc<dyn FileSystem>) -> Self {
        let lock = crate::testlock::serial();
        let prev = active();
        set_active(fs);
        FsGuard { _lock: lock, prev }
    }

    /// Hold the guard and RESTORE-ON-DROP whatever backend is installed during
    /// the window, without installing one now — for a test that then calls a
    /// PRODUCTION door that swaps the backend itself
    /// ([`crate::scenario::install_hermetic_fs`]), so the sandbox can never
    /// leak into a sibling test.
    ///
    /// This exists because the idiom it replaces —
    /// `FsGuard::install(fs::active())` — is a TORN read-modify-write of the
    /// global: Rust evaluates the argument BEFORE `install`
    /// takes the lock, so the "previous" backend it memorizes is whatever was
    /// installed a moment before the guard, not what is installed under it.
    /// Under a concurrent `FsGuard` that restores a backend that was never
    /// current. Reading `prev` INSIDE the locked window makes that
    /// unrepresentable: one acquisition, one read, one truth.
    pub(crate) fn capture() -> Self {
        let lock = crate::testlock::serial();
        let prev = active();
        FsGuard { _lock: lock, prev }
    }
}

#[cfg(test)]
impl Drop for FsGuard {
    fn drop(&mut self) {
        set_active(self.prev.clone());
    }
}

/// An RAII helper that chdirs the process into `dir` for the guard's life,
/// restoring the ORIGINAL cwd on drop — even if the test body panics or an
/// assertion fails, so one failing test never stably strands every sibling
/// test (including ones that just read `current_dir()`, like
/// `main::run::tests::resolve_root_absent_sticky_reproduces_todays_default`)
/// in the wrong directory. The process cwd is a global like the fs backend, so
/// this holds the shared [`crate::testlock`] guard (reentrant) for its whole
/// life — ONE owner for every process-global, no cross-lock order left to
/// invert. Test-only.
#[cfg(test)]
pub(crate) struct CwdGuard {
    _lock: crate::testlock::SerialGuard,
    prev: PathBuf,
}

#[cfg(test)]
impl CwdGuard {
    pub(crate) fn enter(dir: &Path) -> Self {
        let lock = crate::testlock::serial();
        let prev = std::env::current_dir().expect("current dir must be readable");
        std::env::set_current_dir(dir).expect("chdir into test dir");
        CwdGuard { _lock: lock, prev }
    }
}

#[cfg(test)]
impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.prev);
    }
}

/// A backend where every write fails and every read reports "not found" — the
/// fake for proving a save path SURFACES an error rather than losing the edit
/// silently. Shared, because two suites need the same failing disk and a
/// second copy would let them drift into testing different failures.
#[cfg(test)]
pub(crate) struct UnwritableFs;

#[cfg(test)]
impl FileSystem for UnwritableFs {
    fn read_to_string(&self, _path: &std::path::Path) -> std::io::Result<String> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "unwritable fake",
        ))
    }
    fn read(&self, _path: &std::path::Path) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "unwritable fake",
        ))
    }
    fn write(&self, _path: &std::path::Path, _data: &[u8]) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "folder unwritable",
        ))
    }
    fn create_dir_all(&self, _path: &std::path::Path) -> std::io::Result<()> {
        Ok(()) // "creating" the dir succeeds; the WRITE into it is what fails
    }
    fn rename(&self, _from: &std::path::Path, _to: &std::path::Path) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "folder unwritable",
        ))
    }
    fn exists(&self, _path: &std::path::Path) -> bool {
        false
    }
    fn is_dir(&self, _path: &std::path::Path) -> bool {
        false
    }
    fn read_dir(&self, _path: &std::path::Path) -> std::io::Result<Vec<crate::fs::DirEntry>> {
        Ok(vec![])
    }
    fn metadata(&self, _path: &std::path::Path) -> std::io::Result<crate::fs::Metadata> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "unwritable fake",
        ))
    }
    fn remove_file(&self, _path: &std::path::Path) -> std::io::Result<()> {
        Ok(())
    }
}
