use super::FileSystem;
#[cfg(target_arch = "wasm32")]
use super::{DirEntry, Metadata, set_active};
use std::path::Path;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

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
mod backend {
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

        fn rename_no_replace(&self, from: &Path, to: &Path) -> io::Result<()> {
            if self.exists(to) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "destination exists",
                ));
            }
            self.rename(from, to)
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
    let webfs = backend::WebFs;
    webfs.seed_samples();
    set_active(Arc::new(webfs));
}
