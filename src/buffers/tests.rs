//! Registry laws: path normalization and identity, the park/take round
//! trip, and the eviction policy — clean-LRU only, and never a dirty
//! buffer. Carved out of `buffers.rs` to keep that file under its frozen
//! size baseline when the close route's read accessor landed.

use super::*;

fn keyed(path: &str) -> BufferKey {
    BufferKey::path(Path::new(path))
}

#[test]
fn park_then_take_round_trips_the_same_buffer() {
    let mut reg: BufferRegistry<()> = BufferRegistry::default();
    let mut b = Buffer::scratch();
    b.set_text("hello");
    reg.park(
        keyed("/a.txt"),
        Entry {
            buffer: b,
            extra: (),
        },
    );
    assert_eq!(reg.len(), 1);
    assert!(reg.contains(&keyed("/a.txt")));
    let entry = reg.take(&keyed("/a.txt")).expect("parked entry");
    assert_eq!(entry.buffer.text(), "hello");
    assert_eq!(reg.len(), 0);
    assert!(!reg.contains(&keyed("/a.txt")));
}

#[test]
fn take_of_unknown_key_is_none() {
    let mut reg: BufferRegistry<()> = BufferRegistry::default();
    assert!(reg.take(&keyed("/nope.txt")).is_none());
}

#[test]
fn buffer_key_path_normalizes_a_relative_path_against_the_cwd() {
    // REGRESSION (code review): a relative path (e.g. an un-directoried CLI
    // file argument) must key IDENTICALLY to its cwd-joined absolute form —
    // the same file reached two different ways must be the same registry
    // entry.
    // TWO reads of the process-CWD global: this one and the one
    // `normalize_path` takes inside `BufferKey::path`. A `CwdGuard` landing
    // between them made `rel` and `abs` describe two different directories
    // — so the guard is load-bearing, not ceremony.
    let _tg = crate::testlock::serial();
    let cwd = crate::fs::current_dir().unwrap();
    let rel = BufferKey::path(Path::new("some_never_created_test_file.rs"));
    let abs = BufferKey::path(&cwd.join("some_never_created_test_file.rs"));
    assert_eq!(
        rel, abs,
        "relative and cwd-joined-absolute must key the same"
    );
}

#[test]
fn buffer_key_path_collapses_dot_and_dotdot_components() {
    let messy = PathBuf::from("/a/b/x/../c/./file.rs");
    let clean = PathBuf::from("/a/b/c/file.rs");
    assert_eq!(BufferKey::path(&messy), BufferKey::path(&clean));
}

#[test]
#[cfg(unix)]
fn buffer_key_path_resolves_a_symlinked_directory_to_the_real_path() {
    // REGRESSION (code review, scenario c): a path reached THROUGH a
    // symlinked directory must key IDENTICALLY to the path reached via
    // the real directory it points at — a symlink is just another
    // spelling of the same file, and `normalize_path` now resolves it
    // (real `std::fs::canonicalize`, not just lexical `.`/`..` collapse)
    // rather than tracking the symlink's own name.
    let base = crate::testscratch::ScratchDir::new(
        std::env::temp_dir().join(format!("awl-buffers-symlink-{}", std::process::id())),
    );
    let real_dir = base.join("real");
    let link_dir = base.join("link");
    std::fs::create_dir_all(&real_dir).unwrap();
    std::fs::write(real_dir.join("a.txt"), "alpha\n").unwrap();
    std::os::unix::fs::symlink(&real_dir, &link_dir).unwrap();

    let via_real = BufferKey::path(&real_dir.join("a.txt"));
    let via_link = BufferKey::path(&link_dir.join("a.txt"));
    assert_eq!(
        via_real, via_link,
        "the symlinked spelling must key identically to the real path"
    );
}

#[test]
#[cfg(unix)]
fn buffer_key_path_resolves_a_not_yet_existing_file_under_a_symlinked_directory() {
    // The ancestor-canonicalize fallback (`canonicalize_lenient`) must
    // ALSO resolve a symlinked ancestor directory for a file that doesn't
    // exist yet — a new file's key must match its real (not symlink)
    // parent identically whether reached via the link or the target, so
    // it normalizes the same before and after the file is created.
    let base = crate::testscratch::ScratchDir::new(
        std::env::temp_dir().join(format!("awl-buffers-symlink-new-{}", std::process::id())),
    );
    let real_dir = base.join("real");
    let link_dir = base.join("link");
    std::fs::create_dir_all(&real_dir).unwrap();
    std::os::unix::fs::symlink(&real_dir, &link_dir).unwrap();

    let via_real = BufferKey::path(&real_dir.join("new.txt"));
    let via_link = BufferKey::path(&link_dir.join("new.txt"));
    assert_eq!(
        via_real, via_link,
        "a not-yet-existing file under a symlinked ancestor still keys identically"
    );
}

#[test]
fn buffer_key_of_scratch_and_path_and_unnamed_note() {
    // `Buffer::from_file` reads through the swappable fs global — off-guard
    // it can be answered by a sibling test's
    // `InMemoryFs`.
    let _tg = crate::testlock::serial();
    let scratch = Buffer::scratch();
    assert_eq!(BufferKey::of(&scratch), BufferKey::Scratch);

    let file = Buffer::from_file(std::path::Path::new("/does/not/exist/x.rs"));
    assert_eq!(
        BufferKey::of(&file),
        BufferKey::path(Path::new("/does/not/exist/x.rs"))
    );

    let mut note = Buffer::scratch();
    note.start_fresh_doc(PathBuf::from("/notes"));
    assert!(matches!(BufferKey::of(&note), BufferKey::Fresh(_)));
}

#[test]
fn park_evicts_lru_clean_entry_over_cap() {
    let mut reg: BufferRegistry<()> = BufferRegistry::default();
    // Fill to exactly the cap (MAX_OPEN_BUFFERS - 1 backgrounded + 1 active).
    for i in 0..(MAX_OPEN_BUFFERS - 1) {
        reg.park(
            keyed(&format!("/f{i}.txt")),
            Entry {
                buffer: Buffer::scratch(),
                extra: (),
            },
        );
    }
    assert_eq!(reg.len(), MAX_OPEN_BUFFERS - 1);
    // One more push would exceed the cap: the LRU (last-in, i.e. `/f0.txt`,
    // parked first and never re-touched) is evicted.
    reg.park(
        keyed("/new.txt"),
        Entry {
            buffer: Buffer::scratch(),
            extra: (),
        },
    );
    assert_eq!(reg.len(), MAX_OPEN_BUFFERS - 1, "cap holds steady");
    assert!(
        !reg.contains(&keyed("/f0.txt")),
        "the LRU clean entry was evicted"
    );
    assert!(reg.contains(&keyed("/new.txt")));
}

#[test]
fn park_never_evicts_a_dirty_buffer() {
    let mut reg: BufferRegistry<()> = BufferRegistry::default();
    // Fill to the cap with DIRTY buffers (an edit marks dirty).
    for i in 0..(MAX_OPEN_BUFFERS - 1) {
        let mut b = Buffer::scratch();
        b.set_text("x");
        reg.park(
            keyed(&format!("/f{i}.txt")),
            Entry {
                buffer: b,
                extra: (),
            },
        );
    }
    assert_eq!(reg.len(), MAX_OPEN_BUFFERS - 1);
    // The newly-parked buffer is ALSO dirty, so there is truly no clean
    // victim anywhere in the registry.
    let mut newest = Buffer::scratch();
    newest.set_text("y");
    reg.park(
        keyed("/new.txt"),
        Entry {
            buffer: newest,
            extra: (),
        },
    );
    // Nothing dirty could be evicted, so the registry is left OVER cap
    // rather than discarding unsaved work.
    assert_eq!(
        reg.len(),
        MAX_OPEN_BUFFERS,
        "over cap: no dirty buffer was evicted"
    );
    for i in 0..(MAX_OPEN_BUFFERS - 1) {
        assert!(
            reg.contains(&keyed(&format!("/f{i}.txt"))),
            "dirty entry {i} survives"
        );
    }
    assert!(
        reg.contains(&keyed("/new.txt")),
        "the new dirty entry survives too"
    );
}

#[test]
fn park_evicts_the_newest_clean_entry_when_it_is_the_only_clean_one() {
    // A subtler shape of the same law: eviction picks ANY clean victim over
    // NO eviction, even if the only clean buffer happens to be the one just
    // parked (the incoming buffer is not specially protected — only DIRTY
    // buffers are).
    let mut reg: BufferRegistry<()> = BufferRegistry::default();
    for i in 0..(MAX_OPEN_BUFFERS - 1) {
        let mut b = Buffer::scratch();
        b.set_text("x");
        reg.park(
            keyed(&format!("/f{i}.txt")),
            Entry {
                buffer: b,
                extra: (),
            },
        );
    }
    reg.park(
        keyed("/clean.txt"),
        Entry {
            buffer: Buffer::scratch(),
            extra: (),
        },
    );
    assert_eq!(
        reg.len(),
        MAX_OPEN_BUFFERS - 1,
        "cap holds: the one clean entry was evicted"
    );
    assert!(!reg.contains(&keyed("/clean.txt")));
    for i in 0..(MAX_OPEN_BUFFERS - 1) {
        assert!(reg.contains(&keyed(&format!("/f{i}.txt"))));
    }
}
