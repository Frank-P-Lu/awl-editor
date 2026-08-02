//! Laws for the unresolved-change record. The axis these sweep is the DOCUMENT
//! CONTENT, not the header: a record is a manuscript with two lines on top, so
//! every case below round-trips text that would break a naive parser — text
//! containing the magic line, text that is empty, text with no trailing
//! newline, text that looks like TOML.

use super::*;
use crate::fs::{FileSystem, InMemoryFs};
use std::sync::Arc;

fn rec(path: &str, text: &str) -> Record {
    Record {
        path: PathBuf::from(path),
        text: text.to_string(),
    }
}

/// **THE ROUND TRIP, over documents chosen to break a parser.** The record's
/// whole value is that the bytes come back exactly; a manuscript that returns
/// with one byte different is data loss with extra steps.
#[test]
fn every_document_shape_round_trips_byte_for_byte() {
    let documents = [
        ("empty", ""),
        ("no trailing newline", "one line"),
        ("trailing newline", "one line\n"),
        ("blank lines", "\n\n\n"),
        ("windows-ish content", "a\r\nb\r\n"),
        (
            "the magic line as content",
            "awl-unresolved-change 1\nnot a header\n",
        ),
        (
            "a path-looking second line",
            "/etc/passwd\nreally just prose\n",
        ),
        ("toml-looking", "[section]\nkey = \"value\"\n"),
        ("unicode", "héllo — 日本語 — 🙂\n"),
        ("markdown frontmatter", "---\ntitle: x\n---\n\nbody\n"),
        ("a lone dash rule", "---\n"),
        ("very long single line", "x"),
    ];
    for (what, text) in documents {
        let original = rec("/notes/draft.md", text);
        let wire = encode(&original).unwrap_or_else(|| panic!("{what}: refused to encode"));
        let back = decode(&wire).unwrap_or_else(|| panic!("{what}: refused to decode"));
        assert_eq!(
            back, original,
            "{what}: the record did not survive the trip"
        );
        assert_eq!(back.text, text, "{what}: the manuscript changed");
    }
}

/// A record's path is a real path, and a record for one file must never be read
/// as a record for another. This is the guard that makes a stale record safe to
/// find at startup.
#[test]
fn a_record_belongs_to_exactly_one_path() {
    let r = rec("/notes/draft.md", "mine\n");
    assert!(matches_path(&r, Path::new("/notes/draft.md")));
    assert!(!matches_path(&r, Path::new("/notes/other.md")));
    assert!(!matches_path(&r, Path::new("/notes/draft.md.bak")));
    assert!(!matches_path(&r, Path::new("/draft.md")));
    // The decoded record carries the same identity as the encoded one.
    let back = decode(&encode(&r).expect("encodes")).expect("decodes");
    assert!(matches_path(&back, Path::new("/notes/draft.md")));
}

/// ANYTHING THAT IS NOT EXACTLY THIS FORMAT DECODES TO NOTHING. A half-understood
/// record would be restored over the user's real document, which is the failure
/// this whole file exists to prevent — so the parser is required to be brittle,
/// and here is the sweep that keeps it brittle.
#[test]
fn a_malformed_record_is_refused_rather_than_guessed_at() {
    let bad = [
        ("empty file", ""),
        ("magic only", "awl-unresolved-change 1"),
        ("magic and newline only", "awl-unresolved-change 1\n"),
        ("wrong magic", "awl-unresolved-change 2\n/p\ntext"),
        ("no magic", "/notes/draft.md\nsome text\n"),
        ("leading blank line", "\nawl-unresolved-change 1\n/p\ntext"),
        ("empty path line", "awl-unresolved-change 1\n\ntext"),
        ("a plain markdown file", "# A heading\n\nsome prose\n"),
        ("truncated mid-magic", "awl-unresolv"),
        ("case-shifted magic", "AWL-UNRESOLVED-CHANGE 1\n/p\ntext"),
        (
            "trailing space on magic",
            "awl-unresolved-change 1 \n/p\ntext",
        ),
    ];
    for (what, raw) in bad {
        assert!(decode(raw).is_none(), "{what}: must decode to nothing");
    }
    // The one shape that IS valid and looks marginal: a header with an empty
    // document. That is a real state (the user emptied the buffer) and must
    // survive.
    assert_eq!(
        decode("awl-unresolved-change 1\n/p\n"),
        Some(rec("/p", "")),
        "an empty document is a legitimate record"
    );
}

/// A path that cannot be represented is REFUSED, never written in a form that
/// would decode as something else.
#[test]
fn an_unrepresentable_path_is_refused_at_encode() {
    assert!(encode(&rec("/notes/od\nd.md", "text")).is_none());
    assert!(encode(&rec("", "text")).is_none());
    assert!(encode(&rec("/fine.md", "text")).is_some());
}

/// The record lives beside the other machine-owned state, not among the user's
/// documents — and there is exactly ONE of it, which is what "one recovery
/// record" means operationally.
#[test]
fn the_record_sits_beside_the_scratch_stash_and_there_is_one() {
    let _g = crate::testlock::serial();
    assert_eq!(
        record_path().parent(),
        crate::fs::scratch_stash_path().parent(),
        "the record belongs with the machine state, not with the documents"
    );
    assert_eq!(
        record_path().parent(),
        Some(crate::fs::data_root().as_path())
    );
    assert_ne!(record_path(), crate::fs::scratch_stash_path());
    // `record_path` takes no argument: there is no per-file record, by
    // construction rather than by discipline.
    assert_eq!(record_path(), record_path());
}

/// WRITE, READ, CLEAR — over the real backend, including the property that
/// matters most: a second conflict REPLACES the record rather than accumulating
/// beside it.
#[test]
fn writing_twice_maintains_one_record_and_clearing_removes_it() {
    let _g = crate::testlock::serial();
    let mem = InMemoryFs::new();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));

    assert_eq!(read(), None, "nothing recorded yet");

    assert!(write(&rec("/notes/a.md", "first\n")));
    assert_eq!(read(), Some(rec("/notes/a.md", "first\n")));

    assert!(write(&rec("/notes/b.md", "second\n")));
    assert_eq!(
        read(),
        Some(rec("/notes/b.md", "second\n")),
        "the newer conflict replaced the older record"
    );
    // …and left exactly one file behind in the data root, not two.
    let root = crate::fs::data_root();
    let records: Vec<_> = mem
        .read_dir(&root)
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.path == record_path())
        .collect();
    assert_eq!(records.len(), 1, "there must be exactly one record");

    clear();
    assert_eq!(read(), None, "resolving removes the record");
    // Clearing again is harmless — resolution can be reached twice.
    clear();
    assert_eq!(read(), None);
}

/// An unparseable record's bytes are PRESERVED before being ignored, so the very
/// next write cannot destroy a manuscript awl merely failed to understand. Same
/// treatment the scratch stash gets.
#[test]
fn an_unreadable_record_is_preserved_before_it_is_ignored() {
    let _g = crate::testlock::serial();
    let mem = InMemoryFs::new();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let path = record_path();
    mem.create_dir_all(path.parent().expect("has a parent"))
        .unwrap();
    mem.write(&path, b"this is not a record, but it IS somebody's prose\n")
        .unwrap();

    assert_eq!(read(), None, "an unparseable record is not restored");

    let siblings: Vec<String> = mem
        .read_dir(&crate::fs::data_root())
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.name)
        .filter(|n| n.contains(".corrupt-"))
        .collect();
    assert_eq!(
        siblings.len(),
        1,
        "the bytes must be copied aside, not discarded: {siblings:?}"
    );
    let kept = mem
        .read_to_string(&crate::fs::data_root().join(&siblings[0]))
        .expect("the preserved sibling is readable");
    assert_eq!(kept, "this is not a record, but it IS somebody's prose\n");
}

/// The whole point, end to end: text that exists nowhere else survives being
/// written, and comes back the same. Driven over the record's real write/read
/// doors rather than encode/decode, so the atomic-write path is in the loop.
#[test]
fn the_only_copy_of_the_users_text_survives_a_round_trip_through_disk() {
    let _g = crate::testlock::serial();
    let mem = InMemoryFs::new();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let manuscript =
        "The rain in Spain\n\n---\n\nfalls mainly on the plain.\n\nawl-unresolved-change 1\n";
    assert!(write(&rec("/notes/spain.md", manuscript)));
    let back = read().expect("a record was written");
    assert_eq!(back.text, manuscript);
    assert_eq!(back.path, PathBuf::from("/notes/spain.md"));
}
