use crate::clock::SystemTime;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

mod fault;

/// Cross-backend directory entry: leaf name, full path, and kind are all the
/// walk/browse code consumes.
///
/// `is_dir`/`is_file` describe what the entry BEHAVES as, which for a symlink
/// is the type of its TARGET — so every door that lists a level shows a link
/// to a folder as a folder and a link to a file as a file, the same way every
/// other door in the tree already follows links (`is_dir` is `path.is_dir()`,
/// open follows, the `.git` probe follows). A link whose target cannot be
/// stat'd — broken, looping, or behind a permission wall — is neither, which
/// is how a level reader omits it.
///
/// `is_symlink` carries the orthogonal fact that the entry is REACHED through
/// a link, for the one consumer that must treat it differently: the recursive
/// go-to indexer, which shows a symlinked directory but never descends it
/// (`crate::index`'s `walk_collect` — it has no cycle guard).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metadata {
    pub modified: Option<SystemTime>,
    pub len: Option<u64>,
}

pub trait FileSystem: Send + Sync {
    /// Read the whole file at `path` as a UTF-8 string (config load, buffer open).
    fn read_to_string(&self, path: &Path) -> io::Result<String>;

    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()>;

    fn create_dir_all(&self, path: &Path) -> io::Result<()>;

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn exists(&self, path: &Path) -> bool;

    fn is_dir(&self, path: &Path) -> bool;

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirEntry>>;

    fn metadata(&self, path: &Path) -> io::Result<Metadata>;

    /// Remove a single file at `path`. Used by the corrupt-backup pruner
    /// ([`crate::durable::preserve_corrupt`]) to cap how many `.corrupt-*`
    /// siblings a store keeps. Best-effort at every call site (a failed prune
    /// just means one extra sibling lingers, never fatal).
    fn remove_file(&self, path: &Path) -> io::Result<()>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeFs;

impl FileSystem for NativeFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        std::fs::write(path, data)
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    /// `DirEntry::file_type()` is free — the readdir already carried it — but
    /// it reports the LINK's own type, never the target's, so classifying by
    /// it alone makes a symlinked folder neither dir nor file and every level
    /// reader drops it silently. A symlink (and ONLY a symlink) therefore
    /// costs one following `metadata` on its own path:
    ///
    /// * target is a directory or file → the entry behaves as that, and the
    ///   picker shows it as what it points to;
    /// * the stat FAILS — a broken link, a link loop (`ELOOP`, which the
    ///   kernel reports rather than chasing), a permission wall, a dead
    ///   network mount — → neither dir nor file, so the entry is omitted.
    ///   There is nothing to open and nothing to descend, and a name that
    ///   errors on Enter is worse than an absent one.
    ///
    /// The extra stat is bounded to entries the user themself linked; an
    /// ordinary child costs nothing new, and a dead mount holding a plain
    /// directory already blocks in `read_dir` above this line.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let Ok(entry) = entry else { continue };
            let Ok(ft) = entry.file_type() else { continue };
            let path = entry.path();
            let is_symlink = ft.is_symlink();
            let (is_dir, is_file) = if is_symlink {
                match std::fs::metadata(&path) {
                    Ok(md) => (md.is_dir(), md.is_file()),
                    Err(_) => (false, false),
                }
            } else {
                (ft.is_dir(), ft.is_file())
            };
            out.push(DirEntry {
                path,
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir,
                is_file,
                is_symlink,
            });
        }
        Ok(out)
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let md = std::fs::metadata(path)?;
        Ok(Metadata {
            modified: md.modified().ok(),
            len: Some(md.len()),
        })
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }
}

// --- In-memory backend (tests + the hermetic scenario sandbox) -------------
//
// Two consumers: fs-touching unit tests (no real disk, no temp-dir litter) and
// — since the hermetic-scenario round — the PRODUCTION strict-replay sandbox
// (`crate::scenario`), which seeds one of these from the CLI-named storyboard
// inputs and installs it so a scenario run never touches the user's real
// files. Gated `any(test, not(wasm32))`: native builds carry it for the
// scenario door; a wasm release build (whose sandbox is `WebFs`) doesn't.

#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Debug, Clone, Default)]
pub struct InMemoryFs {
    inner: Arc<RwLock<MemState>>,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Debug, Default)]
struct MemState {
    files: std::collections::BTreeMap<PathBuf, MemFile>,
    dirs: std::collections::BTreeSet<PathBuf>,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
struct MemFile {
    bytes: Vec<u8>,
    modified: SystemTime,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl InMemoryFs {
    pub fn new() -> Self {
        let fs = InMemoryFs::default();
        fs.inner.write().unwrap().dirs.insert(PathBuf::from("/"));
        fs
    }

    #[cfg(test)]
    pub fn with_file(self, path: impl AsRef<Path>, contents: &str) -> Self {
        self.write(path.as_ref(), contents.as_bytes()).unwrap();
        self
    }

    #[cfg(test)]
    pub fn with_dir(self, path: impl AsRef<Path>) -> Self {
        self.create_dir_all(path.as_ref()).unwrap();
        self
    }

    fn insert_dirs(state: &mut MemState, path: &Path) {
        let mut cur = Some(path);
        while let Some(p) = cur {
            state.dirs.insert(p.to_path_buf());
            cur = p.parent();
        }
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl FileSystem for InMemoryFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner
            .read()
            .unwrap()
            .files
            .get(path)
            .map(|f| f.bytes.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        let now = crate::clock::system_now();
        let mut state = self.inner.write().unwrap();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            InMemoryFs::insert_dirs(&mut state, parent);
        }
        state.files.insert(
            path.to_path_buf(),
            MemFile {
                bytes: data.to_vec(),
                modified: now,
            },
        );
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        let mut state = self.inner.write().unwrap();
        InMemoryFs::insert_dirs(&mut state, path);
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut state = self.inner.write().unwrap();
        let file = state
            .files
            .remove(from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))?;
        if let Some(parent) = to.parent()
            && !parent.as_os_str().is_empty()
        {
            InMemoryFs::insert_dirs(&mut state, parent);
        }
        state.files.insert(to.to_path_buf(), file);
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        let state = self.inner.read().unwrap();
        state.files.contains_key(path) || state.dirs.contains(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.inner.read().unwrap().dirs.contains(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
        let state = self.inner.read().unwrap();
        if !state.dirs.contains(path) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "no such directory"));
        }
        let mut out = Vec::new();
        let is_child = |p: &Path| p.parent() == Some(path);
        for f in state.files.keys().filter(|p| is_child(p)) {
            out.push(DirEntry {
                path: f.clone(),
                name: leaf_name(f),
                is_dir: false,
                is_file: true,
                // This backend has no symlink concept — a fixture that needs
                // one uses a real scratch directory instead.
                is_symlink: false,
            });
        }
        for d in state
            .dirs
            .iter()
            .filter(|p| p.as_path() != path && is_child(p))
        {
            out.push(DirEntry {
                path: d.clone(),
                name: leaf_name(d),
                is_dir: true,
                is_file: false,
                is_symlink: false,
            });
        }
        Ok(out)
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let state = self.inner.read().unwrap();
        if let Some(f) = state.files.get(path) {
            Ok(Metadata {
                modified: Some(f.modified),
                len: Some(f.bytes.len() as u64),
            })
        } else if state.dirs.contains(path) {
            Ok(Metadata {
                modified: None,
                len: None,
            })
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "no such file"))
        }
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        let mut state = self.inner.write().unwrap();
        state
            .files
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
fn leaf_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

// --- First-load seed samples (shared, platform-agnostic) -------------------
//
// The web build's FIRST-LOAD seed set: a small, CURATED welcome for a
// first-time visitor, not a dumping ground for every dev fixture that has
// ever lived under `samples/`. Kept here (unconditional — NOT `cfg(wasm32)`)
// so the list + its write-if-absent LAW are unit-testable on native via
// [`InMemoryFs`], never only exercised inside a browser sandbox.
//
// Curation note: `samples/longwrap.md` (soft-wrap stress fixture) and
// `samples/spellcheck.md` (squiggle-demo fixture) are deliberately EXCLUDED
// here — real files, still used by the capture harness and docs, just not
// what should greet a first-time visitor. `samples/tour.md` is the new
// markdown showcase; `samples/prose.md` and `samples/japanese.md` (the
// bundled-JP-face beauty moment) carry over unchanged in shape.
//
// `cfg(any(test, wasm32))`: the only consumers are `mod web` (wasm-only) and
// this file's own native unit tests — a plain native `cargo build` has no use
// for any of the three items below, so they'd otherwise warn `dead_code`.
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) const SEED_SAMPLES: &[(&str, &str)] = &[
    // The `include_str!` paths live in the ONE owner, `crate::embedded_docs`.
    ("/welcome.md", crate::embedded_docs::WELCOME_MD),
    ("/tour.md", crate::embedded_docs::TOUR_MD),
    ("/prose.md", crate::embedded_docs::PROSE_MD),
    ("/japanese.md", crate::embedded_docs::JAPANESE_MD),
];

/// The seed-generation sentinel key. Bumped `awlfs:seeded` -> `awlfs:seeded:v2`
/// alongside the curated seed-list change above, so an already-seeded browser
/// (which only ever wrote the OLD key) re-runs seeding exactly once more under
/// the new key — picking up `/tour.md` and dropping `/longwrap.md`+
/// `/spellcheck.md` from the seed set — while [`seed_write_if_absent`]'s own
/// per-file law means it can NEVER clobber bytes the visitor already has. The
/// old `awlfs:seeded` key is simply left inert in `localStorage` for a
/// returning visitor — not read, not cleaned up; a stray unused key costs
/// nothing and a migration pass isn't worth the complexity for one flag.
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) const SEED_SENTINEL_KEY: &str = "awlfs:seeded:v2";

/// Seed [`SEED_SAMPLES`] into `fs`, WRITE-IF-ABSENT per file: a path that
/// already exists is left completely untouched (never overwritten), so a
/// returning visitor who has edited `/welcome.md` — or still has an old
/// `/longwrap.md` / `/spellcheck.md` from a prior seed generation — keeps
/// every byte; they only ever GAIN newly-seeded paths. Generic over
/// `&dyn FileSystem` (not `WebFs`-specific) so this is unit-testable on
/// native with an [`InMemoryFs`] — the sentinel-gating (localStorage-
/// specific, "have I seeded THIS generation yet") stays the caller's job.
///
/// CONVENTION-TRUTHFUL SURFACES ROUND: `SEED_SAMPLES`' text carries
/// `{{key:slug}}` chord tokens (see `keytoken.rs`) — each file's content is
/// rendered through [`crate::keytoken::render_key_tokens`] for `convention`/
/// `platform` BEFORE it's written, so a Linux-web visitor's seeded welcome
/// note says `Ctrl+P` (or the web-alternate chord, where the native one is
/// browser-reserved) and a Mac-web visitor's says `⌘P` — never a hand-typed
/// literal that could drift from what actually fires. `convention`/`platform`
/// are EXPLICIT parameters (the same testability pattern
/// `resolved_native_label_truthful` uses); the one real call site
/// ([`web::WebFs::seed_samples`]) passes both `::current()`.
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn seed_write_if_absent(
    fs: &dyn FileSystem,
    convention: crate::convention::Convention,
    platform: crate::commands::Platform,
) {
    let _ = fs.create_dir_all(Path::new("/"));
    for (p, content) in SEED_SAMPLES {
        let path = Path::new(p);
        if fs.exists(path) {
            continue; // never clobber a visitor's own edits (or an old fixture)
        }
        let rendered = crate::keytoken::render_key_tokens(content, convention, platform);
        let _ = fs.write(path, rendered.as_bytes());
    }
}

// --- Web backend (browser localStorage) -----------------------------------
//
// The SANDBOXED browser backing the seam doc promised. There is no `std::fs` on
// `wasm32-unknown-unknown`, so awl's file ops route through the browser's
// `localStorage` — a synchronous, origin-scoped, reload-persistent key→string
// store, which fits this SYNC trait exactly (no OPFS worker / async handles
// needed for a single-user notes editor). Gated to wasm so the native build is
// byte-identical (the whole module vanishes on a native compile).
//
// MAPPING. localStorage is a FLAT string map, so a tiny virtual filesystem is
// laid over it with TYPE-PREFIXED keys (all under the `awlfs:` namespace so a
// host page's own keys never collide):
//   * `awlfs:F:<path>` → a file's UTF-8 contents.
//   * `awlfs:D:<path>` → a directory MARKER (value unused) so empty dirs exist.
//   * `awlfs:M:<path>` → a file's modified millis (best-effort time; the browser
//     has no inode, so it is recorded on write rather than read from a real stat).
//   * `awlfs:seeded:v2` → the SEED-generation sentinel (see `seed_samples`,
//     [`super::SEED_SENTINEL_KEY`]) — bumped from the v1 `awlfs:seeded` key
//     when the curated seed set changed; the old key is left inert, unread.
// `read_dir` enumerates the `F:`/`D:` keys and keeps the ones whose PARENT is the
// queried dir — the same parent-match `InMemoryFs` uses — so the index walk and
// the go-to / browse pickers see the seeded notes. Binary `read`/`write` round-
// trip through `String::from_utf8_lossy`: awl only ever writes UTF-8 rope text,
// and the only byte reader (the `AWL_FONT` face load) never runs on the web.
#[cfg(target_arch = "wasm32")]
mod web {
    use super::{DirEntry, FileSystem, Metadata};
    use crate::clock::SystemTime;
    use std::io;
    use std::path::Path;
    use std::time::Duration;

    const FILE_PREFIX: &str = "awlfs:F:";
    const DIR_PREFIX: &str = "awlfs:D:";
    const MTIME_PREFIX: &str = "awlfs:M:";

    #[derive(Debug, Default, Clone, Copy)]
    pub struct WebFs;

    /// The origin's `localStorage`, or `None` if the page has no window / the API
    /// is blocked (private-mode lockdowns). Callers degrade gracefully (a read
    /// becomes `NotFound`, a write a benign error) exactly like a headless native
    /// run with no disk.
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    fn js_err(what: &str) -> io::Error {
        io::Error::other(format!("localStorage {what} failed"))
    }

    /// Now, as whole milliseconds since the Unix epoch, via `crate::clock` (the JS
    /// clock on wasm — std's `SystemTime::now()` PANICS on `wasm32-unknown-unknown`,
    /// no platform clock).
    fn now_millis() -> u64 {
        crate::clock::system_now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// A stored-millis stamp back as a `SystemTime`, built by ADDING to the const
    /// `UNIX_EPOCH` (no clock read) so it never trips the wasm panic. The
    /// `Metadata` times cross module boundaries as `crate::clock::SystemTime`.
    fn millis_to_system_time(ms: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_millis(ms)
    }

    impl WebFs {
        fn key(prefix: &str, path: &Path) -> String {
            format!("{prefix}{}", path.to_string_lossy())
        }

        fn insert_dirs(s: &web_sys::Storage, path: &Path) {
            let mut cur = Some(path);
            while let Some(p) = cur {
                let _ = s.set_item(&Self::key(DIR_PREFIX, p), "");
                cur = p.parent();
            }
        }

        /// SEED the sample docs on FIRST load (sentinel-gated on
        /// [`super::SEED_SENTINEL_KEY`], so a reload of an already-seeded
        /// generation is a no-op). Called once at startup by
        /// [`super::install_web_fs`]; the bundled samples are embedded via
        /// `include_str!` (see [`super::SEED_SAMPLES`]), so seeding needs no
        /// network. The actual per-file write-if-absent law lives in the
        /// shared, platform-agnostic [`super::seed_write_if_absent`] — this
        /// method only owns the localStorage-specific sentinel check.
        pub fn seed_samples(&self) {
            let Some(s) = storage() else { return };
            if s.get_item(super::SEED_SENTINEL_KEY)
                .ok()
                .flatten()
                .is_some()
            {
                return; // already seeded this generation — preserve existing notes
            }
            // The UA-detected convention MUST already be set by the time this
            // runs — `main::wasm_start` calls `set_web_convention_from_ua`
            // BEFORE `fs::install_web_fs()` for exactly this reason (see that
            // ordering note there).
            super::seed_write_if_absent(
                self,
                crate::convention::Convention::current(),
                crate::commands::Platform::current(),
            );
            let _ = s.set_item(super::SEED_SENTINEL_KEY, "1");
        }
    }

    impl FileSystem for WebFs {
        fn read_to_string(&self, path: &Path) -> io::Result<String> {
            storage()
                .and_then(|s| s.get_item(&Self::key(FILE_PREFIX, path)).ok().flatten())
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
        }

        fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
            self.read_to_string(path).map(String::into_bytes)
        }

        fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
            let s = storage().ok_or_else(|| js_err("unavailable"))?;
            let text = String::from_utf8_lossy(data);
            s.set_item(&Self::key(FILE_PREFIX, path), &text)
                .map_err(|_| js_err("write"))?;
            let now = now_millis().to_string();
            let _ = s.set_item(&Self::key(MTIME_PREFIX, path), &now);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    Self::insert_dirs(&s, parent);
                }
            }
            Ok(())
        }

        fn create_dir_all(&self, path: &Path) -> io::Result<()> {
            let s = storage().ok_or_else(|| js_err("unavailable"))?;
            Self::insert_dirs(&s, path);
            Ok(())
        }

        fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
            let s = storage().ok_or_else(|| js_err("unavailable"))?;
            let content = s
                .get_item(&Self::key(FILE_PREFIX, from))
                .ok()
                .flatten()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))?;
            let _ = s.remove_item(&Self::key(FILE_PREFIX, from));
            let _ = s.remove_item(&Self::key(MTIME_PREFIX, from));
            s.set_item(&Self::key(FILE_PREFIX, to), &content)
                .map_err(|_| js_err("rename"))?;
            let _ = s.set_item(&Self::key(MTIME_PREFIX, to), &now_millis().to_string());
            if let Some(parent) = to.parent() {
                if !parent.as_os_str().is_empty() {
                    Self::insert_dirs(&s, parent);
                }
            }
            Ok(())
        }

        fn exists(&self, path: &Path) -> bool {
            storage()
                .map(|s| {
                    s.get_item(&Self::key(FILE_PREFIX, path))
                        .ok()
                        .flatten()
                        .is_some()
                        || s.get_item(&Self::key(DIR_PREFIX, path))
                            .ok()
                            .flatten()
                            .is_some()
                })
                .unwrap_or(false)
        }

        fn is_dir(&self, path: &Path) -> bool {
            storage()
                .and_then(|s| s.get_item(&Self::key(DIR_PREFIX, path)).ok().flatten())
                .is_some()
        }

        fn read_dir(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
            let s =
                storage().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no storage"))?;
            let len = s.length().map_err(|_| js_err("length"))?;
            let mut out = Vec::new();
            for i in 0..len {
                let Ok(Some(k)) = s.key(i) else { continue };
                let (rest, is_dir) = if let Some(r) = k.strip_prefix(FILE_PREFIX) {
                    (r, false)
                } else if let Some(r) = k.strip_prefix(DIR_PREFIX) {
                    (r, true)
                } else {
                    continue;
                };
                let child = Path::new(rest);
                if child.parent() != Some(path) || child == path {
                    continue;
                }
                out.push(DirEntry {
                    path: child.to_path_buf(),
                    name: child
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    is_dir,
                    is_file: !is_dir,
                    // localStorage has no links: every key is its own entry.
                    is_symlink: false,
                });
            }
            Ok(out)
        }

        fn metadata(&self, path: &Path) -> io::Result<Metadata> {
            let s =
                storage().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no storage"))?;
            let read_ms = |prefix: &str| -> Option<SystemTime> {
                s.get_item(&Self::key(prefix, path))
                    .ok()
                    .flatten()
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(millis_to_system_time)
            };
            // A file the store knows (it has content) reports its recorded times +
            // byte length (the stored UTF-8 string's length); a bare directory has
            // none; an unknown path errors like a native stat.
            let content = s.get_item(&Self::key(FILE_PREFIX, path)).ok().flatten();
            let is_dir = s
                .get_item(&Self::key(DIR_PREFIX, path))
                .ok()
                .flatten()
                .is_some();
            if let Some(content) = content {
                Ok(Metadata {
                    modified: read_ms(MTIME_PREFIX),
                    len: Some(content.len() as u64),
                })
            } else if is_dir {
                Ok(Metadata {
                    modified: None,
                    len: None,
                })
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "no such file"))
            }
        }

        fn remove_file(&self, path: &Path) -> io::Result<()> {
            let s = storage().ok_or_else(|| js_err("unavailable"))?;
            let key = Self::key(FILE_PREFIX, path);
            if s.get_item(&key).ok().flatten().is_none() {
                return Err(io::Error::new(io::ErrorKind::NotFound, "no such file"));
            }
            let _ = s.remove_item(&key);
            let _ = s.remove_item(&Self::key(MTIME_PREFIX, path));
            Ok(())
        }
    }
}

/// Install the browser [`web::WebFs`] (localStorage) as the active backend and
/// SEED the bundled sample docs on first load. The wasm entrypoint
/// (`main.rs::wasm_start`) calls this once before `app::run`, so the editor opens
/// on a seeded, reload-persistent virtual filesystem instead of the default
/// `NativeFs` (which has no real disk to reach in the sandbox).
#[cfg(target_arch = "wasm32")]
pub fn install_web_fs() {
    let webfs = web::WebFs;
    webfs.seed_samples();
    set_active(Arc::new(webfs));
}

// --- Shared write / path helpers (both backends) ---------------------------

/// ATOMIC WRITE through the active backend: write `data` to a hidden temp
/// sibling (`.<name>.awl-tmp`, same directory so the rename never crosses a
/// filesystem), then `rename` it over `path`. On the native backend a same-dir
/// rename is POSIX-atomic, so a crash mid-save leaves either the OLD file or the
/// NEW one — never a truncated half-write. Uses ONLY the trait's `write` +
/// `rename`, so `InMemoryFs` and `WebFs` model it too (wasm keeps compiling).
/// Used by every buffer save (manual and autosave), the scratch stash, and —
/// after this round's audit — every other durable app-owned store.
///
/// **`AWL_FAULT_DELAY_MS` (DEV-ONLY, native-only, no CLI flag — mirrors
/// `AWL_CJK_FORCE`'s "total no-op unless set" contract):** when set to a
/// valid integer, sleeps that many milliseconds AFTER the tmp write and
/// BEFORE the rename — artificially widening the pre-rename window so the
/// kill-9 fault harness (`tests/fault_kill9.rs`) can reliably land a SIGKILL
/// INSIDE it and assert the target file still holds its OLD content (the
/// rename never happened, so nothing was torn). Unset in every normal run —
/// including every other test in this suite — so this is a genuine zero-cost
/// no-op the rest of the time; reading the env var on every call is cheap
/// enough that a `#[cfg(test)]` gate isn't worth the code-path divergence
/// between test and release builds this primitive most needs to stay honest.
pub fn write_atomic(path: &Path, data: &[u8]) -> io::Result<()> {
    let fs = active();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed".to_string());
    let tmp_name = format!(".{name}.awl-tmp");
    let tmp = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join(tmp_name),
        _ => PathBuf::from(tmp_name),
    };
    fs.write(&tmp, data)?;
    fault::after_tmp_write(&tmp);
    fs.rename(&tmp, path)
}

/// THE ONE HOME-DIRECTORY LOOKUP — `$HOME`, read live rather than cached,
/// because it is not a constant of the process: the MAS sandbox rewrites it to
/// the container's own home at startup (`mas.rs`) and every path awl derives
/// must follow that redirect. `None` when unset or empty, and on wasm.
/// [`data_root`] and `args::resolve_default_folder` BUILD paths under it;
/// `capture::redact` STRIPS it back out of a capture artifact. One lookup, so
/// the stripper can never disagree with the builders about where home is.
pub(crate) fn home_dir() -> Option<PathBuf> {
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var_os("HOME")
            .filter(|h| !h.is_empty())
            .map(PathBuf::from)
    }
}

pub fn data_root() -> PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from("/awl")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(x) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(x).join("awl");
        }
        if let Some(home) = home_dir() {
            return home.join(".local").join("share").join("awl");
        }
        PathBuf::from("awl-data")
    }
}

/// Where the PERSISTENT SCRATCH BUFFER stashes across quits: the no-path launch
/// buffer is written here (atomic, on the same autosave triggers + quit) and
/// restored on the next no-argument launch. Web-safe via the trait (WebFs).
pub fn scratch_stash_path() -> PathBuf {
    data_root().join("scratch.md")
}

/// THE WEB CONFIG PATH — where `config.toml` lives inside the virtual `WebFs`
/// root (a `localStorage` key, `awlfs:F:/awl/config.toml`), closing WEB.md's
/// former "no config file on the web" gap. Beside the scratch stash under
/// [`data_root`] (the SAME `/awl` virtual-root convention `scratch.md` already
/// uses for machine-owned state), deliberately NOT under the seeded content
/// root `/` (which holds the user's own documents). `main::wasm_start` is the
/// ONE caller (native's `config::config_path` resolves an OS path instead and
/// is never reached on wasm); every `Config` write door (`write_pref`/
/// `write_binding`/`write_default`) already routes through
/// `crate::fs::active()` + [`write_atomic`], so a `Config` loaded from THIS
/// path just works over `WebFs` with zero further plumbing.
#[cfg(target_arch = "wasm32")]
pub fn web_config_path() -> PathBuf {
    data_root().join("config.toml")
}

fn global() -> &'static RwLock<Arc<dyn FileSystem>> {
    use std::sync::OnceLock;
    static FS: OnceLock<RwLock<Arc<dyn FileSystem>>> = OnceLock::new();
    FS.get_or_init(|| RwLock::new(Arc::new(NativeFs)))
}

/// THE FS-BACKEND-SERIALIZATION LAW. The active backend is
/// process-GLOBAL and SWAPPABLE, and `cargo test` runs in parallel: while one
/// test has an [`InMemoryFs`] installed via [`FsGuard`], EVERY other thread's
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
/// strict replay swaps in a seeded [`InMemoryFs`] once, at startup, before any
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

#[cfg(test)]
mod serialization_law;

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

#[cfg(test)]
mod scripted;
#[cfg(test)]
pub(crate) use scripted::{ScriptedFailure, ScriptedFs, ScriptedOperation};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_is_the_default_backend() {
        let _g = crate::testlock::serial();
        let err = active()
            .read_to_string(Path::new("/awl/definitely/not/here.toml"))
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn in_memory_round_trips_text() {
        let fs = InMemoryFs::new();
        fs.write(Path::new("/n/a.md"), b"hello").unwrap();
        assert_eq!(fs.read_to_string(Path::new("/n/a.md")).unwrap(), "hello");
        assert!(fs.exists(Path::new("/n")));
        assert!(fs.exists(Path::new("/n/a.md")));
        assert!(!fs.exists(Path::new("/n/b.md")));
    }

    #[test]
    fn in_memory_read_dir_levels_and_types() {
        let fs = InMemoryFs::new()
            .with_file("/r/readme.md", "r")
            .with_dir("/r/src")
            .with_file("/r/src/main.rs", "m");
        let mut names: Vec<String> = fs
            .read_dir(Path::new("/r"))
            .unwrap()
            .into_iter()
            .map(|e| format!("{}:{}", e.name, if e.is_dir { "d" } else { "f" }))
            .collect();
        names.sort();
        assert_eq!(names, vec!["readme.md:f".to_string(), "src:d".to_string()]);
        let sub = fs.read_dir(Path::new("/r/src")).unwrap();
        assert_eq!(sub.len(), 1);
        assert_eq!(sub[0].name, "main.rs");
        assert!(sub[0].is_file);
    }

    #[test]
    fn in_memory_rename_moves_bytes() {
        let fs = InMemoryFs::new().with_file("/a.md", "body");
        fs.rename(Path::new("/a.md"), Path::new("/sub/b.md"))
            .unwrap();
        assert!(!fs.exists(Path::new("/a.md")));
        assert_eq!(fs.read_to_string(Path::new("/sub/b.md")).unwrap(), "body");
    }

    #[test]
    fn in_memory_remove_file_deletes_and_errors_on_a_missing_path() {
        let fs = InMemoryFs::new()
            .with_file("/a.md", "body")
            .with_file("/b.md", "other");
        fs.remove_file(Path::new("/a.md")).unwrap();
        assert!(!fs.exists(Path::new("/a.md")), "removed file is gone");
        assert!(fs.exists(Path::new("/b.md")), "a sibling file is untouched");
        assert_eq!(
            fs.remove_file(Path::new("/a.md")).unwrap_err().kind(),
            io::ErrorKind::NotFound,
            "removing an already-gone (or never-existed) file errors NotFound, never panics"
        );
    }

    #[test]
    fn in_memory_metadata_has_times() {
        let fs = InMemoryFs::new().with_file("/a.md", "x");
        let md = fs.metadata(Path::new("/a.md")).unwrap();
        assert!(md.modified.is_some());
        fs.metadata(Path::new("/nope")).unwrap_err();
    }

    #[test]
    fn write_atomic_replaces_content_and_leaves_no_tmp() {
        // The atomic write lands the exact bytes AND leaves no `.awl-tmp` sibling
        // behind (the temp file is renamed over the target, not copied). Both a
        // fresh create and an overwrite go through the same tmp+rename dance.
        let fake = Arc::new(InMemoryFs::new().with_dir("/docs"));
        with_fs(fake.clone(), || {
            write_atomic(Path::new("/docs/a.md"), b"first").unwrap();
            assert_eq!(
                fake.read_to_string(Path::new("/docs/a.md")).unwrap(),
                "first"
            );
            write_atomic(Path::new("/docs/a.md"), b"second").unwrap();
            assert_eq!(
                fake.read_to_string(Path::new("/docs/a.md")).unwrap(),
                "second"
            );
            let names: Vec<String> = fake
                .read_dir(Path::new("/docs"))
                .unwrap()
                .into_iter()
                .map(|e| e.name)
                .collect();
            assert_eq!(names, vec!["a.md".to_string()], "no tmp residue: {names:?}");
        });
    }

    #[test]
    fn data_root_and_scratch_path_shapes() {
        // Pure SUFFIX asserts (no env mutation, so this can't race the config
        // env tests): whatever XDG/HOME arm resolves, the data root's leaf is
        // `awl` (or the total-function fallback `awl-data`), and the scratch
        // stash is `scratch.md` directly under it.
        let root = data_root();
        let leaf = root.file_name().map(|n| n.to_string_lossy().into_owned());
        assert!(
            leaf.as_deref() == Some("awl") || leaf.as_deref() == Some("awl-data"),
            "data root leaf is awl[-data]: {root:?}"
        );
        let stash = scratch_stash_path();
        assert_eq!(
            stash
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .as_deref(),
            Some("scratch.md")
        );
    }

    #[test]
    fn with_fs_installs_and_restores() {
        let fake = Arc::new(InMemoryFs::new().with_file("/cfg.toml", "zoom = 1.0"));
        with_fs(fake, || {
            assert_eq!(
                active().read_to_string(Path::new("/cfg.toml")).unwrap(),
                "zoom = 1.0"
            );
        });
        let _g = crate::testlock::serial();
        active().read_to_string(Path::new("/cfg.toml")).unwrap_err();
    }

    /// THE CURATED SEED LIST, pinned exactly — the four paths a first-time
    /// web visitor sees, in seed order, and nothing else. `/longwrap.md` and
    /// `/spellcheck.md` (dev fixtures — soft-wrap + squiggle stress tests)
    /// are deliberately NOT in the seed set anymore (the files themselves
    /// still live under `samples/` for the capture harness); a regression
    /// that re-adds either — or drops `/tour.md` — fails this test.
    #[test]
    fn seed_sample_list_is_exactly_the_curated_four() {
        let paths: Vec<&str> = SEED_SAMPLES.iter().map(|(p, _)| *p).collect();
        assert_eq!(
            paths,
            vec!["/welcome.md", "/tour.md", "/prose.md", "/japanese.md"]
        );
        assert!(
            !paths.contains(&"/longwrap.md") && !paths.contains(&"/spellcheck.md"),
            "dev fixtures must never re-enter the first-load seed set: {paths:?}"
        );
        for (p, content) in SEED_SAMPLES {
            assert!(!content.trim().is_empty(), "{p} seeds non-empty content");
        }
    }

    #[test]
    fn seed_sentinel_is_bumped_to_v2() {
        assert_eq!(SEED_SENTINEL_KEY, "awlfs:seeded:v2");
        assert_ne!(
            SEED_SENTINEL_KEY, "awlfs:seeded",
            "must differ from the v1 key"
        );
    }

    /// THE WRITE-IF-ABSENT LAW: seeding a fresh filesystem writes exactly the
    /// curated four paths, TOKEN-RENDERED for the pinned convention/platform
    /// (see `keytoken.rs`) — never the raw `{{key:..}}`-bearing source text.
    #[test]
    fn seed_write_if_absent_seeds_the_curated_set_on_a_fresh_fs() {
        let fs = InMemoryFs::new();
        seed_write_if_absent(
            &fs,
            crate::convention::Convention::Mac,
            crate::commands::Platform::Web,
        );
        for (p, content) in SEED_SAMPLES {
            let rendered = crate::keytoken::render_key_tokens(
                content,
                crate::convention::Convention::Mac,
                crate::commands::Platform::Web,
            );
            assert_eq!(fs.read_to_string(Path::new(p)).unwrap(), rendered);
            assert!(
                !fs.read_to_string(Path::new(p)).unwrap().contains("{{key:"),
                "{p} still carries a raw token"
            );
        }
    }

    #[test]
    fn seed_write_if_absent_never_clobbers_an_existing_path() {
        let fs = InMemoryFs::new()
            .with_file("/welcome.md", "my own edited welcome, thanks")
            .with_file("/longwrap.md", "an old dev-fixture leftover, untouched")
            .with_file("/spellcheck.md", "another old leftover, untouched");
        seed_write_if_absent(
            &fs,
            crate::convention::Convention::Mac,
            crate::commands::Platform::Web,
        );

        assert_eq!(
            fs.read_to_string(Path::new("/welcome.md")).unwrap(),
            "my own edited welcome, thanks"
        );
        // The two dropped-from-seeding dev fixtures are left alone too, not
        // deleted and not overwritten — seeding never touches a path it
        // didn't itself write.
        assert_eq!(
            fs.read_to_string(Path::new("/longwrap.md")).unwrap(),
            "an old dev-fixture leftover, untouched"
        );
        assert_eq!(
            fs.read_to_string(Path::new("/spellcheck.md")).unwrap(),
            "another old leftover, untouched"
        );
        for (p, content) in SEED_SAMPLES {
            let content = &crate::keytoken::render_key_tokens(
                content,
                crate::convention::Convention::Mac,
                crate::commands::Platform::Web,
            );
            if *p == "/welcome.md" {
                continue;
            }
            assert_eq!(fs.read_to_string(Path::new(p)).unwrap(), *content);
        }
    }
}
