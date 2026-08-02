//! src/external.rs — DID THIS FILE CHANGE UNDERNEATH US? The one truth, and why
//! a stat alone can never tell it.
//!
//! awl holds a document in memory and writes it back atomically. Between the
//! read and the write, anything may touch the file: another editor, `git
//! checkout`, a sync client, a second awl. The clobber guard's whole job is to
//! notice, and the thing it must never do is answer "unchanged" about a file
//! whose bytes moved.
//!
//! # Why the stat is not the answer
//!
//! The obvious baseline is mtime, and mtime plus byte length is the obvious
//! repair for it. Both are wrong in the same direction — they answer *probably*
//! when the question is *certainly*:
//!
//!   * **mtime alone** misses any write that lands inside the filesystem's own
//!     timestamp resolution. HFS+ and many network filesystems stamp whole
//!     seconds; a `git checkout` in the same second as awl's last look is
//!     invisible.
//!   * **mtime plus length** misses the SAME-TIME, SAME-LENGTH rewrite, which is
//!     not exotic: a one-character correction, a search-and-replace of equal-width
//!     words, a `sed -i` over a line, or a checkout of a sibling revision that
//!     happens to be the same size. That combination is precisely a silent
//!     overwrite of someone's work, and it is the failure this module exists for.
//!
//! So the baseline carries a DIGEST of the exact bytes awl last saw, and content
//! is what decides. The stat is kept — but only as a cheap way to skip the read,
//! never as the verdict.
//!
//! # The pleasant consequence
//!
//! Because content decides, a write that does not change the bytes is not a
//! change. `touch`, a checkout that restores the identical revision, or a backup
//! tool rewriting a file byte-for-byte all now read as [`Change::Unchanged`],
//! where the old stat compare raised a conflict over nothing. A guard that cries
//! wolf is a guard users learn to dismiss.
//!
//! Everything here is PURE over its inputs except [`Seen::at`], the one read.

use crate::fs::Metadata;
use std::path::Path;

/// FNV-1a, 64-bit, over a file's exact bytes.
///
/// Chosen for being three lines of arithmetic with no dependency and no
/// endianness or version drift — the value is compared only against another
/// value this same function produced, in this same process or a later one, so
/// what matters is that it is deterministic and spelled out here rather than
/// inherited from a hasher whose implementation may change under us
/// (`std::hash::DefaultHasher` explicitly reserves that right).
///
/// This is NOT a security primitive: a hostile party could construct a
/// collision. Nothing here defends against a hostile party — the adversary is a
/// text editor in another window.
pub fn digest(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// WHAT AWL SAW AT A PATH, AT ONE MOMENT.
///
/// `Absent` is a real observation, not a missing one: "there was no file here"
/// is exactly what makes a later appearance detectable. A buffer that has never
/// looked at its path holds `Absent`, which is the conservative reading — the
/// first real look then reports [`Change::Appeared`] rather than nothing.
///
/// `digest: None` inside `Present` means the file was there but its bytes could
/// not be read (permissions, a device error, a race with a rename). The compare
/// degrades to the stat for that pair and says so in its own doc; it never
/// pretends the content matched.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Seen {
    #[default]
    Absent,
    Present {
        stat: Metadata,
        digest: Option<u64>,
    },
}

impl Seen {
    /// LOOK AT THE DISK NOW — the module's one impure function, through the
    /// injectable [`crate::fs`] backend so a test drives it over an
    /// `InMemoryFs`.
    ///
    /// The read is unconditional rather than stat-gated. Gating it on "the stat
    /// moved" would reintroduce the exact hole this module closes: a same-time,
    /// same-length rewrite leaves the stat identical, so the read that would
    /// have caught it is the one that gets skipped. awl's documents are prose —
    /// the read is a few tens of microseconds and it is paid at persistence and
    /// identity boundaries, never per frame.
    pub fn at(path: &Path) -> Seen {
        let fs = crate::fs::active();
        let Ok(stat) = fs.metadata(path) else {
            return Seen::Absent;
        };
        Seen::Present {
            stat,
            digest: fs.read(path).ok().map(|bytes| digest(&bytes)),
        }
    }

    /// The observation implied by bytes awl ITSELF just wrote to `path`: re-stat
    /// for the timestamp, but take the digest from the bytes in hand rather than
    /// reading them back. Same value, one less read — and, more importantly, no
    /// window in which someone else's write lands between our rename and our
    /// re-read and silently becomes our baseline.
    pub fn after_write(path: &Path, bytes: &[u8]) -> Seen {
        match crate::fs::active().metadata(path) {
            Ok(stat) => Seen::Present {
                stat,
                digest: Some(digest(bytes)),
            },
            // The write reported success and the file is already unreadable.
            // Record what we know — the content — with no stat, so the next
            // compare still has a digest to work from.
            Err(_) => Seen::Present {
                stat: Metadata {
                    modified: None,
                    len: Some(bytes.len() as u64),
                },
                digest: Some(digest(bytes)),
            },
        }
    }
}

/// HOW THE DISK MOVED between two observations. Wildcard-free at every consumer
/// so a new arm cannot be silently swallowed by an `_`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// The bytes awl last saw are still the bytes on disk — or there is still
    /// nothing there. A write is safe.
    Unchanged,
    /// The file is still there and its content is not what awl last saw.
    Modified,
    /// There was nothing there and now there is. Our write would destroy it.
    Appeared,
    /// There was a file and now there is not.
    Deleted,
}

impl Change {
    /// Is this a change a user could lose work to? Every arm but `Unchanged`,
    /// spelled out rather than negated, so a new arm has to choose.
    pub fn is_change(self) -> bool {
        match self {
            Change::Unchanged => false,
            Change::Modified | Change::Appeared | Change::Deleted => true,
        }
    }
}

/// THE TRUTH TABLE — pure over two observations.
///
/// The `(Present, Present)` arm is the whole point. When both digests are known,
/// **content alone decides**: equal bytes are `Unchanged` however far the stat
/// moved, and different bytes are `Modified` however still the stat sat. Only
/// when a digest is missing — an unreadable file at one end or the other — does
/// this fall back to comparing the stat, and that fallback is deliberately
/// PESSIMISTIC: an unknown digest on either side with any stat difference reads
/// as `Modified`, because "we could not check" must never render as "safe to
/// overwrite".
pub fn compare(last: &Seen, now: &Seen) -> Change {
    match (last, now) {
        (Seen::Absent, Seen::Absent) => Change::Unchanged,
        (Seen::Absent, Seen::Present { .. }) => Change::Appeared,
        (Seen::Present { .. }, Seen::Absent) => Change::Deleted,
        (
            Seen::Present {
                stat: ls,
                digest: ld,
            },
            Seen::Present {
                stat: ns,
                digest: nd,
            },
        ) => match (ld, nd) {
            (Some(l), Some(n)) if l == n => Change::Unchanged,
            (Some(_), Some(_)) => Change::Modified,
            _ if stat_moved(ls, ns) => Change::Modified,
            _ => Change::Unchanged,
        },
    }
}

/// Did the stat move? The pre-digest guard's own rule, kept for the one case
/// that still needs it (an unreadable file), and never consulted when content
/// is knowable.
fn stat_moved(last: &Metadata, now: &Metadata) -> bool {
    if last.modified != now.modified {
        return true;
    }
    match (last.len, now.len) {
        (Some(l), Some(n)) => l != n,
        _ => false,
    }
}

/// LOOK, AND SAY WHAT MOVED — the shape every persistence and identity boundary
/// calls. Returns the fresh observation alongside the verdict so a caller that
/// decides to proceed can adopt it as the new baseline without a second read
/// (and, more to the point, without a second read that could see a DIFFERENT
/// file than the one it just judged).
pub fn look(path: &Path, last: &Seen) -> (Change, Seen) {
    let now = Seen::at(path);
    (compare(last, &now), now)
}

#[cfg(test)]
mod tests;
