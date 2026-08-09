use super::super::*;
use super::{keyspec, replay_keys};

#[test]
fn headless_replay_never_arms_autosave_or_stashes_scratch() {
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i RET t h e r e").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert_eq!(buffer.text(), "hi\nthere", "the edits themselves landed");
        assert!(
            crate::fs::active()
                .read(&crate::fs::scratch_stash_path())
                .is_err(),
            "no scratch stash is ever written headlessly"
        );
        let hist = crate::fs::data_root().join("history");
        assert!(
            crate::fs::active()
                .read_dir(&hist)
                .map(|v| v.is_empty())
                .unwrap_or(true),
            "no history log is ever written headlessly"
        );
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn headless_screenshot_never_installs_the_crash_hook() {
    // The CRASH-VISIBILITY CAPTURE GATE as the same tripwire shape:
    // `crashlog::install_hook` is called from exactly ONE door,
    // `crate::app::run`'s native branch — never reached by any headless
    // `--screenshot`/`--keys`/`--bench-*` mode, every one of which drives a
    // bare `Buffer` straight through `replay_keys` (this file's own shared
    // seam) and never constructs a live `App` or calls `crate::app::run`.
    // The witness global stays false across a whole replay.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(
            !crate::crashlog::hook_installed_for_test(),
            "a headless replay must never install the panic hook"
        );
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn headless_replay_never_touches_the_session_file() {
    // The SESSION RESTORE determinism law as the same tripwire shape:
    // `session_flush`/`apply_session_restore` live only on the live App
    // (`app/session.rs`), which `replay_keys` never constructs — so a
    // `--keys` replay against a bare `Buffer` must never create
    // `session.toml`, even after edits + a save.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i s-s").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(
            crate::fs::active()
                .read(&crate::session::session_path())
                .is_err(),
            "no session file is ever written headlessly"
        );
    });
}

#[test]
fn headless_replay_never_touches_the_recent_files_store() {
    // The RECENTLY-OPENED FILES determinism law as the same tripwire shape:
    // `push_recent_file` (and the `recent_files` load) live only on the live
    // `App` (`app/files/`), which `replay_keys` never constructs — so a
    // `--keys` replay against a bare `Buffer` must never create
    // `recent-files.toml`, even after edits + a save.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i s-s").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(
            crate::fs::active()
                .read(&crate::recent_files::recent_files_path())
                .is_err(),
            "no recent-files store is ever written headlessly"
        );
    });
}

#[test]
fn headless_replay_never_touches_reduced_motion() {
    // ACCESSIBILITY TIER 1's determinism law: `motion::apply_at_startup` (the
    // ONLY function that ever consults OS/browser detection OR reads
    // `Config::reduce_motion`) lives exclusively on the live App's own
    // startup path (`App::new`), which `replay_keys` never constructs — so a
    // `--keys` replay must leave `motion::reduced()` at its default `false`
    // EVEN WHEN the passed config explicitly names `reduce_motion: true`
    // (proving the config value itself is never read here, not merely that
    // the OS call is skipped).
    let _g = crate::testlock::serial();
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("h i s-s").unwrap();
    let root = PathBuf::from("/tmp");
    let cfg = Config {
        reduce_motion: Some(true),
        ..Config::empty()
    };
    let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &cfg, None);
    assert!(
        !crate::motion::reduced(),
        "a headless --keys replay must never apply the config's reduce_motion pref"
    );
    crate::motion::set_reduced(saved);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn headless_replay_never_touches_the_stats_file() {
    // The LIFETIME STATS determinism law as the same tripwire shape: every
    // stats tracking hook + `stats_flush` lives only on the live `App`
    // (`app/stats.rs`), which `replay_keys` never constructs — so a `--keys`
    // replay against a bare `Buffer` must never create `stats.toml`, even
    // after edits + a save. The SILENT USAGE LEDGER (`command_usage`) rides
    // the SAME `stats.toml`, recorded only in `App::apply` (never the headless
    // core), so this one tripwire covers it too — no capture can attribute a
    // command dispatch to any door.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i s-s").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(
            crate::fs::active()
                .read(&crate::stats::stats_path())
                .is_err(),
            "no stats file is ever written headlessly"
        );
    });
}

#[test]
fn headless_load_buffer_never_writes_back_frontmatter() {
    // The i18n round's DETERMINISM LAW as a tripwire (mirrors the autosave
    // one above): `load_buffer` is the headless capture's ONLY file-load
    // door, and the write-back-once tagger lives exclusively on the live
    // `App` (`App::new` / `App::load_path`), never here — so an untagged
    // Japanese fixture loads byte-identically, with NO frontmatter block
    // ever appearing headlessly.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let p = PathBuf::from("/notes/japanese.md");
        let original = "これは日本語の文章です。\n";
        crate::fs::active().write(&p, original.as_bytes()).unwrap();
        let buffer = load_buffer(&Some(p));
        assert_eq!(
            buffer.text(),
            original,
            "no frontmatter ever appears headlessly"
        );
    });
}

/// DOOR 3: `load_buffer` (the `--screenshot` /
/// `--screenshot-motion[-v|-d]` / timeline / held / frames / storyboard
/// headless capture door — the LAUNCH-ARGUMENT analog of `App::new`,
/// `src/app/tests/openable.rs`'s DOOR 1) asks the SAME
/// `crate::openable::classify` capability owner before it ever reaches
/// `Buffer::from_file`.
///
/// NON-VACUOUS — the exact real-world repro (`cargo run --
/// --screenshot out.png --keys "s-s" logo.png` truncating a real PNG to
/// zero bytes) reproduced headlessly: revert `load_buffer`'s gating back
/// to `Some(p) => Buffer::from_file(p)` (leaving `crate::openable` itself
/// untouched) and this test FAILS at the FIRST assertion —
/// `buffer.path()` comes back `Some("/proj/logo.png")` instead of `None`,
/// because `Buffer::from_file`'s UTF-8-decode-error fallback returns an
/// EMPTY buffer STILL BOUND to the binary path (see `crate::openable`'s
/// module doc) — and the end-to-end save assertion at the bottom fails
/// too: the replayed `s-s` truncates `logo.png` to `b""`.
#[test]
fn headless_capture_door_refuses_binary_and_never_lets_save_truncate_it() {
    use crate::fs::FileSystem;
    use std::sync::Arc;

    let png = PathBuf::from("/proj/logo.png");
    let png_bytes: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00];
    let xyzzy = PathBuf::from("/proj/notes.xyzzy");

    let mem = crate::fs::InMemoryFs::new();
    mem.write(&png, png_bytes).unwrap();
    mem.write(&xyzzy, b"plain prose, odd extension\n").unwrap();

    crate::fs::with_fs(Arc::new(mem), || {
        let buffer = load_buffer(&Some(png.clone()));
        assert_eq!(
            buffer.path(),
            None,
            "a refused binary file never produces a buffer bound to its path"
        );
        assert_eq!(
            buffer.text(),
            "",
            "the refusal degrades to an ordinary empty scratch buffer"
        );

        let text_buffer = load_buffer(&Some(xyzzy.clone()));
        assert_eq!(
            text_buffer.path(),
            Some(xyzzy.as_path()),
            "a supported unusual-extension text file still opens headlessly"
        );
        assert_eq!(text_buffer.text(), "plain prose, odd extension\n");

        let mut buffer = load_buffer(&Some(png.clone()));
        let keys = keyspec::parse_keys("s-s").unwrap();
        let root = PathBuf::from("/proj");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        let after = crate::fs::active().read(&png).unwrap();
        assert_eq!(
            after.as_slice(),
            png_bytes,
            "a replayed save can never truncate a file the capture refused to open"
        );
    });
}
