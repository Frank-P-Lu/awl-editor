//! tests/scratch_dir_law.rs — THE SCRATCH-DIRECTORY-CLEANUP LAW.
//!
//! THE LEAK THIS LOCKS OUT. Every fixture that needed a real on-disk directory
//! used to write its own copy of a three-line idiom: wipe a stale leftover,
//! `create_dir_all` a fresh one, do the test, `remove_dir_all` once more at the
//! end of the function. That closing call is an explicit statement on the
//! HAPPY PATH ONLY — it never runs when the test panics, returns early, or is
//! killed, and several fixtures never even wrote a closing call at all. The
//! result: `$TMPDIR` accumulated 2,768 `awl-*` directories (9.2 GB) before
//! `common::ScratchDir` (a guard whose `Drop` removes its directory on every
//! exit path) replaced the idiom everywhere in this test crate.
//!
//! TWO LAWS, so a new fixture cannot silently reintroduce either half of the
//! old idiom:
//!
//!   1. **No manual removal.** The literal call `remove_dir_all` may appear in
//!      exactly ONE file, `tests/common/mod.rs` — `ScratchDir`'s own `Drop`
//!      impl. A test that writes its own `std::fs::remove_dir_all(&dir)` has
//!      reintroduced the happy-path-only cleanup this item retired.
//!   2. **No unwrapped root.** Every `std::env::temp_dir()` call that seeds a
//!      NEW scratch directory must be a direct argument to
//!      `ScratchDir::new` or `ScratchDir::claim` — checked by requiring at
//!      least as many `ScratchDir::new(`/`ScratchDir::claim(` sites as
//!      `std::env::temp_dir()` sites in a file, so a bare, never-wrapped
//!      `let dir = std::env::temp_dir()...` cannot slip in uncounted. A small
//!      NAMED allowlist covers the legitimate exceptions: single loose FILES
//!      (not directories — e.g. `tests/world_gallery_roster.rs`'s one fixed
//!      PNG path) and reads of the bare system temp root that create nothing.
//!
//! `tests/fault_kill9.rs` is NOT exempt from either law: its child process
//! cannot clean up the pre-rename file it is deliberately `SIGKILL`ed inside,
//! but the PARENT test still owns and wraps its directory in a `ScratchDir`
//! like every other fixture (see `src/testscratch.rs`'s module doc and
//! `tests/fault_kill9.rs`'s own `tmp_dir` doc comment for the exception,
//! spelled out where the kill actually happens).

use std::path::Path;

/// A source file's CODE, with every `//`-comment cut away (mirrors
/// `tests/spawn_config_law.rs`'s `code_only` — both laws need the same
/// "a raw text scan would convict this very file's prose" guard). String-aware:
/// a `//` inside a string literal is not a comment.
fn code_only(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let bytes = line.as_bytes();
        let mut in_str = false;
        let mut i = 0;
        let cut = loop {
            if i >= bytes.len() {
                break bytes.len();
            }
            match bytes[i] {
                b'\\' if in_str => i += 1,
                b'"' => in_str = !in_str,
                b'/' if !in_str && bytes.get(i + 1) == Some(&b'/') => break i,
                _ => {}
            }
            i += 1;
        };
        out.push_str(&line[..cut]);
        out.push('\n');
    }
    out
}

/// Every `tests/*.rs` source, plus the shared module, as `(name, code)` —
/// comments stripped by [`code_only`].
fn test_sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut out: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(&dir)
        .expect("tests/ is readable")
        .flatten()
    {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "rs") {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            let src = std::fs::read_to_string(&p).expect("test source reads");
            out.push((name, code_only(&src)));
        }
    }
    let owner = dir.join("common").join("mod.rs");
    let owner_src = std::fs::read_to_string(&owner).expect("the shared spawn module exists");
    out.push(("common/mod.rs".to_string(), code_only(&owner_src)));
    out.sort();
    out
}

/// LAW 1: `remove_dir_all` names ownership; only the guard's own `Drop` impl
/// may call it. Assembled in two pieces so this file's own doc prose above
/// (which names the call) cannot trip its own law.
fn banned_removal() -> String {
    format!("remove_dir{}", "_all")
}

#[test]
fn only_the_guard_calls_remove_dir_all() {
    let banned = banned_removal();
    let offenders: Vec<String> = test_sources()
        .into_iter()
        .filter(|(name, _)| name != "common/mod.rs" && name != "scratch_dir_law.rs")
        .filter(|(_, text)| text.contains(&banned))
        .map(|(name, _)| name)
        .collect();
    assert!(
        offenders.is_empty(),
        "these tests call remove_dir_all directly instead of letting a \
         `common::ScratchDir` guard's Drop own the cleanup: {offenders:?}. An \
         explicit end-of-function remove is a happy-path-only idiom — it never \
         runs on a panic or an early return. \
         Wrap the directory in `common::ScratchDir::new`/`::claim` instead."
    );
}

/// LAW 2: a bare, never-wrapped `std::env::temp_dir()` root. `world_gallery_
/// roster.rs` names one loose FILE (not a directory) directly under the OS
/// temp root — nothing is ever created there for `ScratchDir` to own, so it
/// is the one named allowlist entry.
const FILE_NOT_DIR_ALLOWLIST: &[&str] = &["world_gallery_roster.rs"];

#[test]
fn every_new_scratch_root_is_wrapped_in_scratch_dir() {
    let needle = "std::env::temp_dir()";
    let mut offenders: Vec<String> = Vec::new();
    for (name, text) in test_sources() {
        if name == "common/mod.rs"
            || name == "scratch_dir_law.rs"
            || FILE_NOT_DIR_ALLOWLIST.contains(&name.as_str())
        {
            continue;
        }
        let calls = text.matches(needle).count();
        let wraps =
            text.matches("ScratchDir::new(").count() + text.matches("ScratchDir::claim(").count();
        if calls > wraps {
            offenders.push(format!(
                "{name} ({calls} temp_dir() call(s), {wraps} ScratchDir wrap(s))"
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "these tests build a scratch root off std::env::temp_dir() without routing it \
         through common::ScratchDir::new/::claim: {offenders:?}. An unwrapped root is \
         either never cleaned up at all, or cleaned up only on the happy path — \
         add it to FILE_NOT_DIR_ALLOWLIST here ONLY if it names a \
         single loose file, never a directory."
    );
}
