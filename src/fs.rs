use crate::clock::SystemTime;
use std::io;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Arc;
mod active;
mod fault;
#[cfg(any(test, not(target_arch = "wasm32")))]
mod memory;
mod native;
mod paths;
#[cfg(any(test, target_arch = "wasm32"))]
mod web;
#[cfg(test)]
pub(crate) use active::{CwdGuard, FsGuard, UnwritableFs, with_fs};
pub use active::{active, set_active};
#[allow(unused_imports)]
pub(crate) use active::{assert_fs_is_serialized, current_dir};
#[cfg(any(test, not(target_arch = "wasm32")))]
pub use memory::InMemoryFs;
pub use native::NativeFs;
pub(crate) use paths::home_dir;
#[cfg(target_arch = "wasm32")]
pub use paths::web_config_path;
#[cfg(not(target_arch = "wasm32"))]
pub use paths::write_atomic_new;
pub use paths::{data_root, scratch_stash_path, write_atomic};
#[cfg(target_arch = "wasm32")]
pub use web::install_web_fs;
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(unused_imports)]
pub(crate) use web::{SEED_SAMPLES, SEED_SENTINEL_KEY, seed_write_if_absent};
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

    /// Atomically publish `from` at an absent `to`, refusing to replace an
    /// existing destination. This is the no-clobber half of a durable write.
    #[cfg(not(target_arch = "wasm32"))]
    fn rename_no_replace(&self, from: &Path, to: &Path) -> io::Result<()>;

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

#[cfg(test)]
mod serialization_law;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod scripted;
#[cfg(all(test, not(target_arch = "wasm32")))]
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
