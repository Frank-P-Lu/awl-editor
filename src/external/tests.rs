//! Truth tables for the external-change owner. The axis these sweep is CONTENT
//! against STAT — the one an mtime-and-length guard gets wrong — so every arm
//! below is stated twice where it can be: once with the stat moving and once
//! with the stat held perfectly still.

use super::*;
use crate::fs::{FileSystem, InMemoryFs};
use std::sync::Arc;

fn present(modified_ms: u64, len: u64, bytes: &[u8]) -> Seen {
    Seen::Present {
        stat: Metadata {
            modified: Some(
                crate::clock::SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_millis(modified_ms),
            ),
            len: Some(len),
        },
        digest: Some(digest(bytes)),
    }
}

/// The stat-only pair for the degraded arm: present, but the bytes could not be
/// read.
fn opaque(modified_ms: u64, len: u64) -> Seen {
    Seen::Present {
        stat: Metadata {
            modified: Some(
                crate::clock::SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_millis(modified_ms),
            ),
            len: Some(len),
        },
        digest: None,
    }
}

/// **THE HEADLINE.** A rewrite that lands in the same timestamp tick AND keeps
/// the same byte length is a real external edit and must read as one.
///
/// This is the case the item's premise correction names, and it is the case an
/// mtime-plus-length guard answers `Unchanged` to — which is a silent overwrite
/// of someone's work. Swept across the shapes that actually produce it: a
/// one-character correction, an equal-width search-and-replace, and a
/// byte-permutation (same multiset, same length, same everything but order).
#[test]
fn a_same_tick_same_length_rewrite_is_a_change() {
    let cases: &[(&str, &[u8], &[u8])] = &[
        ("one character corrected", b"the cat sat", b"the bat sat"),
        (
            "equal-width replace",
            b"colour is colour",
            b"colours i colour",
        ),
        ("permuted bytes", b"abcd\n", b"dcba\n"),
        ("a single flipped bit", b"\x00", b"\x01"),
    ];
    for (what, before, after) in cases {
        assert_eq!(
            before.len(),
            after.len(),
            "{what}: fixture is not same-length"
        );
        // The SAME stat on both sides — identical mtime, identical length.
        let last = present(1_700_000_000_000, before.len() as u64, before);
        let now = present(1_700_000_000_000, after.len() as u64, after);
        assert_eq!(
            compare(&last, &now),
            Change::Modified,
            "{what}: a same-tick, same-length rewrite must be seen"
        );
    }
}

/// The empirical half of the headline, through the REAL backend rather than
/// hand-built values: read the rewritten file for real, then substitute the
/// PRE-rewrite stat into the observation. Whatever the filesystem did with the
/// timestamp, the verdict must survive having it taken away — which is what
/// proves the digest, not the stat, is doing the work.
#[test]
fn the_verdict_survives_having_the_stat_taken_away() {
    let _g = crate::testlock::serial();
    let mem = InMemoryFs::new();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let p = std::path::Path::new("/d/f.md");
    mem.write(p, b"one line of prose\n").unwrap();
    let baseline = Seen::at(p);
    mem.write(p, b"ONE LINE OF PROSE\n").unwrap(); // same length, upper-cased
    let fresh = Seen::at(p);

    // Forge the observation an mtime-and-length guard would have compared: the
    // real new content, wearing the old stat.
    let Seen::Present { stat: old_stat, .. } = baseline else {
        panic!("the fixture file exists");
    };
    let Seen::Present {
        digest: new_digest, ..
    } = fresh
    else {
        panic!("the rewritten file exists");
    };
    let stat_disguised = Seen::Present {
        stat: old_stat,
        digest: new_digest,
    };
    assert_eq!(
        compare(&baseline, &stat_disguised),
        Change::Modified,
        "content, not the stat, decides"
    );
    // And the same pair with the digests removed — the guard awl used to have —
    // reports nothing at all. This is the defect, pinned, so the law above can
    // never be read as belt-and-braces.
    let last_blind = opaque(0, old_stat.len.unwrap_or(0));
    let now_blind = opaque(0, old_stat.len.unwrap_or(0));
    assert_eq!(
        compare(&last_blind, &now_blind),
        Change::Unchanged,
        "a stat-only compare cannot see this, which is why the digest exists"
    );
}

/// Every cell of `(last, now)`, with the stat swept independently of content.
#[test]
fn the_full_truth_table() {
    let a = b"alpha\n";
    let b = b"beta beta\n";

    // Absent x Absent — nothing was there and nothing is; a write CREATES.
    assert_eq!(compare(&Seen::Absent, &Seen::Absent), Change::Unchanged);
    // Absent x Present — the file APPEARED; a write would destroy it.
    assert_eq!(compare(&Seen::Absent, &present(10, 6, a)), Change::Appeared);
    // Present x Absent — DELETED under us.
    assert_eq!(compare(&present(10, 6, a), &Seen::Absent), Change::Deleted);

    // Present x Present, content equal, stat MOVED — a touch, a same-content
    // rewrite, a restored identical revision. Not a change: nothing can be lost.
    assert_eq!(
        compare(&present(10, 6, a), &present(999, 6, a)),
        Change::Unchanged,
        "identical bytes are never a conflict, however far the timestamp moved"
    );
    // Present x Present, content differs, stat MOVED — the ordinary case.
    assert_eq!(
        compare(&present(10, 6, a), &present(999, 10, b)),
        Change::Modified
    );
    // Present x Present, content differs, stat STILL — the headline, again in
    // the table so the table itself is complete.
    assert_eq!(
        compare(&present(10, 6, a), &present(10, 6, b"omega\n")),
        Change::Modified
    );
}

/// The DEGRADED arm: an unreadable file has no digest, so the compare falls back
/// to the stat — and the fallback is pessimistic. "We could not check" must
/// never render as "safe to overwrite".
#[test]
fn an_unreadable_file_degrades_to_the_stat_and_degrades_pessimistically() {
    let a = b"alpha\n";
    // Unknown on the NOW side, stat moved → Modified.
    assert_eq!(
        compare(&present(10, 6, a), &opaque(999, 6)),
        Change::Modified
    );
    // Unknown on the LAST side, stat moved → Modified.
    assert_eq!(
        compare(&opaque(10, 6), &present(999, 6, a)),
        Change::Modified
    );
    // Unknown on both sides, length differs at the same mtime → Modified (the
    // old size guard's one genuine catch, kept for exactly this case).
    assert_eq!(compare(&opaque(10, 6), &opaque(10, 7)), Change::Modified);
    // Unknown, stat identical → Unchanged is all that can honestly be said.
    assert_eq!(compare(&opaque(10, 6), &opaque(10, 6)), Change::Unchanged);
}

/// REPEATED external writes: each look is judged against the baseline awl
/// actually holds, and adopting a fresh observation as the baseline is what
/// makes the next write detectable rather than the same one reported forever.
#[test]
fn repeated_external_writes_are_each_seen_once() {
    let _g = crate::testlock::serial();
    let mem = InMemoryFs::new();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let p = std::path::Path::new("/d/f.md");
    mem.write(p, b"v1\n").unwrap();
    let mut baseline = Seen::at(p);

    for n in 2..=5u32 {
        // Same length every time, so only the digest can tell them apart.
        mem.write(p, format!("v{n}\n").as_bytes()).unwrap();
        let (change, fresh) = look(p, &baseline);
        assert_eq!(change, Change::Modified, "write {n} must be seen");
        // Looking again WITHOUT adopting still reports the same one change —
        // the verdict is a function of the baseline, not a consumed event.
        assert_eq!(look(p, &baseline).0, Change::Modified);
        baseline = fresh;
        // Adopted: quiet again until someone else writes.
        assert_eq!(look(p, &baseline).0, Change::Unchanged);
    }
}

/// DELETION, then reappearance — the two arms that are easy to collapse into
/// "changed" and then get wrong at the call site, since a deleted file and a
/// modified one need different treatment (there is nothing to compare against).
#[test]
fn deletion_and_reappearance_are_distinct_arms() {
    let _g = crate::testlock::serial();
    let mem = InMemoryFs::new();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let p = std::path::Path::new("/d/f.md");
    mem.write(p, b"here\n").unwrap();
    let baseline = Seen::at(p);
    mem.remove_file(p).unwrap();
    let (change, gone) = look(p, &baseline);
    assert_eq!(change, Change::Deleted);
    assert_eq!(gone, Seen::Absent);
    // Someone puts a file back at that path: from the Absent baseline that is
    // an APPEARANCE, not a modification — awl has no bytes there to have lost.
    mem.write(p, b"different file entirely\n").unwrap();
    assert_eq!(look(p, &gone).0, Change::Appeared);
}

/// `after_write` must produce a baseline that immediately reads clean against
/// the disk — otherwise every successful save would raise a conflict with
/// itself, which is the bookkeeping bug this constructor exists to make
/// impossible.
#[test]
fn a_baseline_taken_from_our_own_write_reads_clean() {
    let _g = crate::testlock::serial();
    let mem = InMemoryFs::new();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let p = std::path::Path::new("/d/f.md");
    let bytes = b"what awl wrote\n";
    crate::fs::write_atomic(p, bytes).unwrap();
    let baseline = Seen::after_write(p, bytes);
    assert_eq!(look(p, &baseline).0, Change::Unchanged);
    assert_eq!(baseline, Seen::at(p), "the adopted baseline IS the disk");
}

/// The digest is a function of the bytes and nothing else, and it separates the
/// inputs a length-based guard cannot. Includes the empty file, which must have
/// a digest rather than being confused with an absent one.
#[test]
fn the_digest_separates_what_length_cannot() {
    let samples: &[&[u8]] = &[
        b"",
        b"\n",
        b"a",
        b"b",
        b"ab",
        b"ba",
        b"the cat sat",
        b"the bat sat",
        "héllo".as_bytes(),
        "hello".as_bytes(),
    ];
    let mut seen = std::collections::BTreeMap::new();
    for s in samples {
        let d = digest(s);
        if let Some(prev) = seen.insert(d, *s) {
            panic!("digest collision between {prev:?} and {s:?}");
        }
    }
    // Deterministic across calls, and the empty file has a real value.
    assert_eq!(digest(b""), digest(b""));
    assert_eq!(digest(b"abc"), digest(b"abc"));
    assert_ne!(digest(b""), 0, "the empty file must still fingerprint");
    // And `Seen::Absent` is not the empty file — the distinction the
    // Appeared/Deleted arms rest on. An absent file and an empty one must not
    // compare equal, or a deletion would read as an emptying and vice versa.
    let empty = Seen::Present {
        stat: crate::fs::Metadata {
            modified: None,
            len: Some(0),
        },
        digest: Some(digest(b"")),
    };
    assert_ne!(Seen::Absent, empty);
    assert_eq!(compare(&Seen::Absent, &empty), Change::Appeared);
    assert_eq!(compare(&empty, &Seen::Absent), Change::Deleted);
}

/// `Change::is_change` is exhaustive by construction; this pins that every arm
/// but `Unchanged` counts, so a new arm defaulting to "harmless" is noticed.
#[test]
fn every_change_arm_but_unchanged_counts() {
    let all = [
        Change::Unchanged,
        Change::Modified,
        Change::Appeared,
        Change::Deleted,
    ];
    assert!(!Change::Unchanged.is_change());
    for c in all {
        if c != Change::Unchanged {
            assert!(c.is_change(), "{c:?} must count as a change");
        }
    }
}
