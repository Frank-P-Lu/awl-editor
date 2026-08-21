//! Durable-store laws: the TOML load/flush round trip, corrupt-sibling
//! preservation and pruning, and THE ATOMIC-WRITE AUDIT — the per-file
//! census of bare writes that bypass `crate::fs::write_atomic`.
//!
//! Carved out of `durable.rs`, which sits exactly at its frozen size
//! baseline: the audit's accounting rows are prose, they grow every time a
//! new fixture earns one, and a file with no headroom cannot host a census
//! that is designed to accrete justifications.

use super::*;
use crate::fs::FileSystem;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn corrupt_backup_name_is_stem_dot_corrupt_dash_padded_millis_dash_padded_seq() {
    assert_eq!(
        corrupt_backup_name("session.toml", 42, 7),
        "session.toml.corrupt-00000000000000000042-0000000007"
    );
}

#[test]
fn corrupt_backup_name_disambiguates_the_same_millisecond_via_seq() {
    // Two backups landing in the exact same wall-clock millisecond (a
    // real scenario under a tight burst) must still get DISTINCT names —
    // `seq` alone is the uniqueness guarantee, not `now_ms`.
    let a = corrupt_backup_name("x.toml", 1000, 0);
    let b = corrupt_backup_name("x.toml", 1000, 1);
    assert_ne!(a, b);
}

#[test]
fn corrupt_siblings_to_prune_keeps_newest_and_ignores_unrelated_names() {
    let names: Vec<String> = vec![
        "session.toml".to_string(),
        "session.toml.corrupt-00000000000000000001".to_string(),
        "session.toml.corrupt-00000000000000000002".to_string(),
        "session.toml.corrupt-00000000000000000003".to_string(),
        "stats.toml.corrupt-00000000000000000099".to_string(), // a DIFFERENT store's sibling
    ];
    let pruned = corrupt_siblings_to_prune(&names, "session.toml", 2);
    assert_eq!(
        pruned,
        vec!["session.toml.corrupt-00000000000000000001".to_string()]
    );
    // Never touches the live file or another store's sibling.
    assert!(!pruned.iter().any(|n| n == "session.toml"));
    assert!(!pruned.iter().any(|n| n.starts_with("stats.toml")));
}

#[test]
fn corrupt_siblings_to_prune_is_a_no_op_under_the_keep_count() {
    let names: Vec<String> = vec![
        "a.toml.corrupt-1".to_string(),
        "a.toml.corrupt-2".to_string(),
    ];
    assert!(corrupt_siblings_to_prune(&names, "a.toml", 5).is_empty());
    assert!(corrupt_siblings_to_prune(&[], "a.toml", 5).is_empty());
}

#[test]
fn preserve_corrupt_writes_a_sibling_and_prunes_down_to_the_keep_count() {
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_dir("/data"));
    crate::fs::with_fs(fake.clone(), || {
        let path = PathBuf::from("/data/session.toml");
        for i in 0..(CORRUPT_BACKUP_KEEP + 3) {
            preserve_corrupt(&path, format!("garbage {i}").as_bytes());
        }
        let names: Vec<String> = fake
            .read_dir(Path::new("/data"))
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        let siblings: Vec<&String> = names
            .iter()
            .filter(|n| n.starts_with("session.toml.corrupt-"))
            .collect();
        assert_eq!(
            siblings.len(),
            CORRUPT_BACKUP_KEEP,
            "pruned down to the keep count: {names:?}"
        );
    });
}

#[test]
fn preserve_corrupt_never_touches_the_live_store_file() {
    let fake =
        Arc::new(crate::fs::InMemoryFs::new().with_file("/data/session.toml", "active = 1\n"));
    crate::fs::with_fs(fake.clone(), || {
        preserve_corrupt(&PathBuf::from("/data/session.toml"), b"garbage");
        assert_eq!(
            fake.read_to_string(Path::new("/data/session.toml"))
                .unwrap(),
            "active = 1\n",
            "the live file is untouched — only a NEW sibling is written"
        );
    });
}

#[derive(Debug, Default, PartialEq)]
struct Toy {
    n: i64,
}
fn parse_toy(src: &str) -> Toy {
    let n = src
        .parse::<toml::Table>()
        .ok()
        .and_then(|t| t.get("n").and_then(|v| v.as_integer()))
        .unwrap_or(0);
    Toy { n }
}

#[test]
fn load_toml_store_missing_file_is_default_and_preserves_nothing() {
    let fake = Arc::new(crate::fs::InMemoryFs::new());
    crate::fs::with_fs(fake.clone(), || {
        let path = PathBuf::from("/data/toy.toml");
        assert_eq!(load_toml_store(&path, parse_toy), Toy::default());
        fake.read_dir(Path::new("/data")).unwrap_err();
    });
}

#[test]
fn load_toml_store_valid_toml_missing_field_is_lenient_default_no_backup() {
    let fake = Arc::new(crate::fs::InMemoryFs::new().with_file("/data/toy.toml", "other = 3\n"));
    crate::fs::with_fs(fake.clone(), || {
        let path = PathBuf::from("/data/toy.toml");
        assert_eq!(load_toml_store(&path, parse_toy), Toy { n: 0 });
        let names: Vec<String> = fake
            .read_dir(Path::new("/data"))
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert!(
            !names.iter().any(|n| n.contains(".corrupt-")),
            "a valid-but-incomplete TOML table must never back up: {names:?}"
        );
    });
}

#[test]
fn load_toml_store_garbled_toml_syntax_preserves_a_sibling_then_defaults() {
    let fake =
        Arc::new(crate::fs::InMemoryFs::new().with_file("/data/toy.toml", "not valid toml {{{"));
    crate::fs::with_fs(fake.clone(), || {
        let path = PathBuf::from("/data/toy.toml");
        assert_eq!(load_toml_store(&path, parse_toy), Toy::default());
        let names: Vec<String> = fake
            .read_dir(Path::new("/data"))
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        let siblings: Vec<&String> = names
            .iter()
            .filter(|n| n.starts_with("toy.toml.corrupt-"))
            .collect();
        assert_eq!(
            siblings.len(),
            1,
            "the garbled original is preserved: {names:?}"
        );
        let backup = fake
            .read_to_string(Path::new("/data").join(siblings[0]).as_path())
            .unwrap();
        assert_eq!(
            backup, "not valid toml {{{",
            "the sibling holds the ORIGINAL bytes verbatim"
        );
    });
}

#[test]
fn load_toml_store_next_flush_does_not_destroy_the_preserved_sibling() {
    // The exact bug this round exists to close: a corrupt load followed by
    // a normal save must not wipe out the backup it just made.
    let fake =
        Arc::new(crate::fs::InMemoryFs::new().with_file("/data/toy.toml", "not valid toml {{{"));
    crate::fs::with_fs(fake.clone(), || {
        let path = PathBuf::from("/data/toy.toml");
        let toy = load_toml_store(&path, parse_toy);
        assert_eq!(toy, Toy::default());
        // A normal "save the (now-default) state back" — mirrors every
        // store's own `save()`, which always goes through `write_atomic`
        // on the STORE's own path, never touching a `.corrupt-*` sibling.
        crate::fs::write_atomic(&path, b"n = 0\n").unwrap();
        let names: Vec<String> = fake
            .read_dir(Path::new("/data"))
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert!(
            names.iter().any(|n| n.starts_with("toy.toml.corrupt-")),
            "the sibling backup survives the next flush: {names:?}"
        );
        assert_eq!(fake.read_to_string(&path).unwrap(), "n = 0\n");
    });
}

// --- THE ATOMIC-WRITE AUDIT LAW: no bare, non-atomic durable write ----
//
// A "bare write" here means any of THREE call shapes that bypass
// `crate::fs::write_atomic`'s temp-sibling-then-rename dance: a plain
// `write` call chained straight off `active()`, a local `fs` handle's
// own bare `write` call, or the raw `std::fs` free-function `write`,
// unmediated by the `FileSystem` trait. The needles are assembled so this
// source scanner cannot catch its own literal patterns (as in `app.rs`).
// Every occurrence of these three needles across the whole crate is
// counted per file below (source-scan pattern, mirroring `app.rs`'s own
// law test). Adding a NEW bare write anywhere — including a new file —
// changes some file's count and fails this test until the table is
// consciously updated, which forces the same "route it through
// write_atomic, or justify why not" choice every existing site already
// made:
//
//   src/durable.rs (1)   — `preserve_corrupt`'s OWN backup writer: writing
//     a brand-new, uniquely-timestamped sibling file that never existed
//     before, so there is no pre-existing content a tear could destroy —
//     the one narrow case where "always a new file" makes bare-write
//     safe by construction (documented in `preserve_corrupt`'s own doc).
//   src/crashlog.rs (1)  — the mid-panic `write_log` writer, DELIBERATELY
//     primitive per this round's own instructions ("crashlog's mid-panic
//     writer stays deliberately primitive") — a panicking thread must
//     not risk taking a lock or doing a fancier multi-step write.
//   src/fs.rs + src/fs/{native,paths,web}.rs (1 each, except paths' two) — an in-memory test
//     seed, `NativeFs`'s primitive, `write_atomic` and `write_atomic_new`'s tmp-sibling writes,
//     and `seed_write_if_absent`. The latter never overwrites existing
//     content; a tear cannot corrupt a returning visitor's data. The three
//     production primitives cannot recursively route through themselves.
//   src/app/tests/{buffers,lifecycle}.rs, src/app/daemon.rs,
//   src/app/files/close/tests.rs, src/buffers/tests.rs,
//   src/daemon.rs, src/history/tests.rs, src/index.rs, src/main/tests/*.rs
//     — every one of these is INSIDE a `#[cfg(test)]` module, seeding a
//     real temp-dir fixture file directly (never a durable app store) or
//     (in `history/tests.rs`) deliberately planting garbage to exercise
//     THIS round's own corrupt-recovery test.
#[test]
fn no_bare_durable_write_bypasses_write_atomic_outside_the_accounted_for_sites() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    scan_dir_for_bare_writes(&root, &root, &mut counts);

    let expected: &[(&str, usize)] = &[
        ("app/tests/buffers.rs", 1),
        ("app/tests/lifecycle.rs", 3),
        ("app/daemon.rs", 3),
        // THE ICON PACK STEP (`app_icon::pack_all`, `awl --pack-icns`): a
        // build-time tool that writes REPO ARTIFACTS — the per-world
        // `.icns` files, the canonical bundle icon, and the generated
        // `src/app_icon/embedded.rs`. Not a durable user store: a torn
        // write means re-running `scripts/export-icons.sh`, and the result
        // is reviewed in a diff before it is committed. Routing it through
        // `write_atomic`/`fs::active()` would also send it into a hermetic
        // sandbox under test, swallowing the very files the caller asked
        // for — the same reason the capture PNG and the storyboard trace
        // write with plain `std::fs` (see `main/story.rs` below). The
        // fourth site is `app_icon::export_linux_icon` (`awl
        // --export-linux-icon`): same class of write, one committed
        // artifact cut into one file a packaging script asked for.
        ("app_icon/mod.rs", 4),
        // The REMOVAL OWNER's own laws (`app::files::close`). They seed and
        // then externally rewrite real fixture files on a `ScratchDir`,
        // because the subject under test is the conflict gate — whose whole
        // job is to notice that a file moved between two observations.
        // Routing these through `write_atomic`, or through `InMemoryFs`,
        // would make the "somebody else wrote it" half unreachable: the
        // fixture has to write BEHIND the App's back for the gate to have
        // anything to detect.
        ("app/files/close/tests.rs", 12),
        // The registry's own laws, carved out of `buffers.rs` into a
        // sibling to keep that file under its frozen size baseline. Same
        // single temp-dir fixture seed it always was, one directory down.
        ("buffers/tests.rs", 1),
        ("crashlog.rs", 1),
        ("daemon.rs", 1),
        ("durable.rs", 1),
        // The export golden-file BLESS helpers (`export::tests::golden` and
        // `export::pdf::tests`, gated on `AWL_BLESS`): write committed test
        // fixtures under `src/export/testdata/`, never a durable user store
        // — a torn write just means re-blessing, and either golden is
        ("export/pdf/tests.rs", 1),
        ("export/tests.rs", 1),
        ("firstrun/tests.rs", 1),
        ("fs.rs", 1),
        ("fs/native.rs", 1),
        ("fs/paths.rs", 2),
        ("fs/web.rs", 1),
        ("history/tests.rs", 1),
        // Four are pre-existing index fixtures; four more seed
        // `go_to_index_does_not_descend_a_symlinked_dir`'s real scratch
        // tree — a symlink cannot be faked in `InMemoryFs`, so that law
        // needs files on a real disk (an ordinary child, an ordinary
        // grandchild, and the two the linked directory would wrongly
        // contribute). Scratch fixtures under a `ScratchDir`, never a
        // durable store.
        ("index.rs", 8),
        // Two of these are the fresh-oracle Goto regression's own fixture
        // seeds (`goto_switch_mid_replay_reshapes_the_oracle_to_the_
        // arriving_buffer`) — temp-dir test files, never a durable store;
        // two more are the hermetic-scenario tests' real-disk inputs
        // (seeded precisely to prove the sandbox never writes them back).
        // The launch-context law and strict-replay no-artifact law
        // add two temp input fixtures of the same shape. The
        // Two marker files per root for the real `capture_screenshot`
        // fixture tree — the switch-project law's, and the DOOR journey's
        // for the nested tree. Scratch nothing reads back, so atomicity
        // buys them nothing.
        // `main/tests.rs` split seven ways; same 21 sites, unmoved.
        ("main/tests/buffer_switching.rs", 8),
        // The seventh site seeds the language-toast screenshot law's
        // markdown input under its `ScratchDir`; like the other six, it is
        // disposable harness input rather than a durable user store.
        ("main/tests/capture_scenarios.rs", 7),
        ("main/tests/credits_capture.rs", 1), // disposable ScratchDir fixture
        ("main/tests/headless_safety.rs", 1),
        ("main/tests/launch_context.rs", 2),
        ("main/tests/page_measure.rs", 2),
        ("main/tests/replay_warnings.rs", 2),
        ("main/tests/visual_motion.rs", 2),
        // The storyboard runner's `trace.json` write (`write_trace`): a
        // HARNESS DELIVERABLE, not app state — a storyboard run's active
        // backend IS the hermetic sandbox, so routing this through
        // `write_atomic`/`fs::active()` would swallow the artifact the
        // caller asked for (same reason the capture PNG + film frames
        // write with `std::fs`/`image` directly). Overwritten whole per
        // run; a torn write costs one re-run of a deterministic scenario,
        // never user data.
        ("main/story.rs", 1),
        // The link-target file `project_roster_includes_a_symlinked_child_
        // folder` points a symlink at, so the roster law can prove a link
        // to a FILE classifies as a file. Same reason as `index.rs` above:
        // `InMemoryFs` has no links, so the fixture lives on a real disk
        // under a `ScratchDir`.
        ("overlay/tests/project.rs", 1),
        ("render/overrides/tests.rs", 1), // render_overrides_env_read_law's own fixture.
        // The seeding boundary itself (`cli_seeds`/`data_root_seeds`/
        // `tree_seeds`) READs the real disk before the sandbox exists;
        // neither is a durable store.
        ("scenario.rs", 1),
        // The module's own tests WRITE real-disk fixtures the same way,
        // carved into their own file by the size ceiling.
        ("scenario/tests.rs", 14),
        ("testscratch.rs", 3), // ScratchDir's own fixtures, not a store.
    ];
    let expected_map: std::collections::BTreeMap<String, usize> =
        expected.iter().map(|(f, n)| (f.to_string(), *n)).collect();
    assert_eq!(
        counts, expected_map,
        "a bare (non-write_atomic) durable write appeared somewhere unaccounted for — \
         route it through crate::fs::write_atomic, or add it to this table with a \
         comment justifying why not (mirrors app.rs's own \
         real_fs_app_new_calls_are_all_accounted_for)"
    );
}

/// The three bare-write call shapes this law test hunts for, ASSEMBLED
/// from fragments (never written as one literal string in this file) so
/// the scanner — which walks this very file too — can't match its own
/// needle definitions. See the law test's doc comment above for why.
#[cfg(test)]
fn bare_write_needles() -> [String; 3] {
    [
        ["active()", ".", "write", "("].concat(),
        ["fs", ".", "write", "("].concat(),
        ["std", "::", "fs", "::", "write", "("].concat(),
    ]
}

#[cfg(test)]
fn scan_dir_for_bare_writes(
    base: &std::path::Path,
    dir: &std::path::Path,
    counts: &mut std::collections::BTreeMap<String, usize>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let needles = bare_write_needles();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_bare_writes(base, &path, counts);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        // The history corrupt-recovery fixture deliberately has this
        // method call rustfmt-split across lines: formatting must not hide
        // a bare durable write from the audit.
        let compact: String = text.split_whitespace().collect();
        let n: usize = needles
            .iter()
            .map(|needle| compact.matches(needle.as_str()).count())
            .sum();
        if n == 0 {
            continue;
        }
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        counts.insert(rel, n);
    }
}
