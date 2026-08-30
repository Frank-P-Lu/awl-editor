//! The MULTI-BUFFER REGISTRY — the identity + eviction policy shared by the
//! live [`crate::app::App`] and the headless `--keys` replay
//! ([`crate::main::run::replay_keys`]), so "open a file that's already open
//! switches to its live buffer" behaves IDENTICALLY in both (same behavior ⇒
//! same code, per CLAUDE.md's engineering principles — no aligned copies).
//!
//! The ACTIVE buffer is never held here: it stays exactly where it always
//! lived (`App::buffer` / the replay's `buffer` local), so that seam's name +
//! type stay stable and the diff introducing this module is reviewable. This
//! module owns only the BACKGROUNDED buffers — the other N-1 open files —
//! keyed by a stable identity so re-opening one finds its live state instead
//! of re-reading disk.
//!
//! The registry is the state model, not chrome. Working-set rows, session
//! persistence, and daemon waiters consume the same `BufferKey` but keep their
//! own policy. The registry is generic over a small `extra` payload (`T`) so
//! the live App can carry per-buffer bookkeeping (scroll / spell cache /
//! autosave versions — see `app::files::BufferExtra`) while headless replay
//! carries none (`()`).

use std::path::{Path, PathBuf};

mod path;

use crate::buffer::Buffer;

/// A buffer's stable identity for registry lookups. A SAVED file is keyed by
/// its bound path (NORMALIZED — absolutized + canonicalized where possible,
/// see [`BufferKey::path`]); the ONE
/// pathless "scratch" writing surface (the launch buffer, or the persistent
/// stash it restores from) is keyed by the `Scratch` sentinel — there is only
/// ever one such identity, mirroring the one persistent scratch stash
/// (`fs::scratch_stash_path`). A pathless QUICK NOTE that hasn't been named
/// yet carries a session-unique `Fresh` identity until its naming save commits.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BufferKey {
    Path(NormPath),
    Scratch,
    Fresh(u64),
}

/// A normalized path (see [`normalize_path`]) — the identity payload of
/// `BufferKey::Path`.
/// Rust always makes an enum variant's fields as visible as the enum itself
/// (there is no way to mark `BufferKey::Path`'s field private while keeping
/// `BufferKey` public), so normalization is instead enforced by wrapping the
/// `PathBuf` in this newtype with a PRIVATE field: the only way to build one
/// is [`NormPath::of`] (private to this module), routed from
/// [`BufferKey::path`]. A sibling module (`app::files`, `main::run`, or any
/// future caller) structurally CANNOT construct `BufferKey::Path(..)` from a
/// raw, un-normalized path — the bypass this module's doc warns about doesn't
/// just rely on convention, it fails to compile.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NormPath(PathBuf);

impl NormPath {
    fn of(p: &Path) -> Self {
        NormPath(normalize_path(p))
    }
}

impl BufferKey {
    /// Build a PATH identity, routed through [`normalize_path`] so the SAME
    /// file is recognized as the same registry entry no matter which of its
    /// (possibly several) textual spellings produced the path — e.g. a CLI
    /// file argument typed with no directory component (`cd project && awl
    /// a.txt`, staying relative) versus that same file's later ROOT-JOINED
    /// spelling (`index::resolve`, always absolute — every Goto-picker
    /// candidate). Without this, the two spellings hash to different keys: the
    /// buffer opened under the first spelling gets parked under it, a later
    /// Goto to the second spelling never finds it, silently re-reads the file
    /// from disk (discarding the live edit), and leaves the first spelling's
    /// entry orphaned in the registry forever (never evictable once dirty).
    /// THE ONE constructor every `BufferKey::Path` site must go through
    /// (`BufferKey::of` below, plus the Goto-accept sites in
    /// `app::files::load_path` / `main::run::replay_keys`) — same behavior ⇒
    /// same code, per CLAUDE.md.
    pub fn path(p: &Path) -> Self {
        BufferKey::Path(NormPath::of(p))
    }

    /// The registry identity for `buffer`. Every live document has one: a
    /// path, the persistent scratch sentinel, or a session-unique provisional
    /// identity while a fresh document is waiting for its first successful
    /// naming save.
    pub fn of(buffer: &Buffer) -> Self {
        match buffer.path() {
            Some(p) => BufferKey::path(p),
            None => buffer
                .fresh_id()
                .map(BufferKey::Fresh)
                .unwrap_or(BufferKey::Scratch),
        }
    }

    pub(crate) fn sidecar_label(&self) -> String {
        match self {
            BufferKey::Path(path) => path.0.display().to_string(),
            BufferKey::Scratch => "scratch".to_string(),
            BufferKey::Fresh(_) => "untitled".to_string(),
        }
    }

    #[cfg(test)]
    pub(crate) fn path_buf(&self) -> Option<PathBuf> {
        match self {
            BufferKey::Path(path) => Some(path.0.clone()),
            BufferKey::Scratch | BufferKey::Fresh(_) => None,
        }
    }
}

/// Normalize `p` to a stable, comparable form: make it ABSOLUTE (joined
/// against the process's current directory when relative), then resolve it
/// through [`std::fs::canonicalize`] — which ALSO collapses `.`/`..` and
/// follows symlinks, so a symlinked directory in the path resolves to the
/// SAME identity as the real one (two spellings of one file must be one
/// registry entry, full stop; tracking the symlink's own name would defeat
/// the entire point of normalizing). `canonicalize` requires every component
/// to exist, which a freshly-typed CLI argument for a NOT-YET-CREATED file
/// never does — so on failure, [`canonicalize_lenient`] walks UP to the
/// deepest EXISTING ancestor, canonicalizes that instead, and re-joins the
/// remaining (lexically pre-collapsed) tail — so the new file's key
/// normalizes identically once it exists, matching whatever spelling of its
/// existing parent directory was used to reach it. See [`BufferKey::path`]
/// for why this matters.
///
/// `pub(crate)` (not just a private helper behind [`BufferKey::path`]) so the
/// DAEMON CLIENT (`crate::daemon::startup`) can canonicalize a launch-argument
/// path the SAME lenient way before handing it to an already-running instance
/// — the server can never recover the client's cwd on its own, so the client
/// must send an already-normalized, absolute path.
pub(crate) fn normalize_path(p: &Path) -> PathBuf {
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        crate::fs::current_dir()
            .map(|cwd| cwd.join(p))
            .unwrap_or_else(|_| p.to_path_buf())
    };
    let clean = path::lexically_collapse(&abs);
    std::fs::canonicalize(&clean).unwrap_or_else(|_| path::canonicalize_lenient(&clean))
}

/// Max simultaneously-open buffers (the active one + everything backgrounded
/// here), a modest cap so a long session doesn't grow memory unboundedly.
/// PRODUCT CALL (flagged, taking the calm default): past the cap, the
/// LEAST-RECENTLY-USED CLEAN (unedited-since-open) backgrounded buffer is
/// evicted; a DIRTY buffer is NEVER evicted — the cap is silently exceeded
/// rather than discarding unsaved work. A future UI could surface "N buffers
/// open, M unsaved" (out of scope here — no tab strip in v1).
pub const MAX_OPEN_BUFFERS: usize = 16;

/// One BACKGROUNDED buffer's saved state: the [`Buffer`] itself (cursor,
/// selection anchor, undo/redo history, and dirty flag already live ON
/// `Buffer` — nothing to duplicate there) plus an opaque `extra` payload the
/// caller attaches for its OWN per-buffer bookkeeping.
pub struct Entry<T> {
    pub buffer: Buffer,
    pub extra: T,
}

/// MRU-ordered registry of backgrounded buffers (index 0 = most recently
/// backgrounded = the eviction LAST-resort), keyed by [`BufferKey`]. Generic
/// over the caller's per-buffer payload `T`.
pub struct BufferRegistry<T> {
    entries: Vec<(BufferKey, Entry<T>)>,
    /// Latches once the over-cap-all-dirty notice (see `park`) has fired, so a
    /// user who keeps opening dirty files past the cap gets ONE calm stderr
    /// line instead of a re-print on every subsequent open (code review nit:
    /// the un-latched version was harmless but noisy). Clears the instant a
    /// clean eviction succeeds again — i.e. it tracks "are we CURRENTLY stuck
    /// over cap with nothing evictable", not "has this ever happened".
    over_cap_warned: bool,
}

impl<T> Default for BufferRegistry<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            over_cap_warned: false,
        }
    }
}

impl<T> BufferRegistry<T> {
    /// How many buffers are parked here (NOT counting the caller's active one).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when `key` names a currently-backgrounded buffer — a test-only
    /// companion of `park`/`take` (the live code never queries membership).
    #[cfg(test)]
    pub(crate) fn contains(&self, key: &BufferKey) -> bool {
        self.entries.iter().any(|(k, _)| k == key)
    }

    /// Test oracle for route laws that must prove a backgrounded identity is
    /// still paired with its exact user text, not merely count registry slots.
    #[cfg(test)]
    pub(crate) fn text_snapshots(&self) -> Vec<(BufferKey, String)> {
        self.entries
            .iter()
            .map(|(key, entry)| (key.clone(), entry.buffer.text()))
            .collect()
    }

    /// Park `entry` under `key` at the MRU front, evicting the LRU clean PATH
    /// entry (the only kind disk can reconstruct) while doing so would push the
    /// total open count (this registry + 1 active) past [`MAX_OPEN_BUFFERS`].
    /// Pathless Scratch/Fresh entries may exceed the soft cap. Replaces
    /// any existing entry under the same key (should not normally happen —
    /// the caller only parks the buffer it is LEAVING).
    pub fn park(&mut self, key: BufferKey, entry: Entry<T>) {
        self.entries.retain(|(k, _)| k != &key);
        self.entries.insert(0, (key, entry));
        while self.entries.len() + 1 > MAX_OPEN_BUFFERS {
            // Only a clean PATH is reversibly evictable: reopening it reloads
            // disk. Scratch and Fresh have no path to reconstruct from, so
            // evicting either would leave a dead working row and lose state.
            match self.entries.iter().rposition(|(key, entry)| {
                matches!(key, BufferKey::Path(_)) && !entry.buffer.is_dirty()
            }) {
                Some(pos) => {
                    self.entries.remove(pos);
                    self.over_cap_warned = false;
                }
                None => {
                    // No entry is reversibly reloadable: never discard pathless
                    // state or dirty work — exceed the cap instead. Fire
                    // the notice once per "stuck over cap" spell, not once per
                    // subsequent open (see `over_cap_warned`'s doc).
                    if !self.over_cap_warned {
                        eprintln!(
                            "awl: buffer registry over cap ({} open, no clean reloadable path) — \
                             keeping all",
                            self.entries.len() + 1
                        );
                        self.over_cap_warned = true;
                    }
                    break;
                }
            }
        }
    }

    /// BORROW the entry for `key` without disturbing MRU order — the read half
    /// of `take`, for a caller that must ask a backgrounded buffer a question
    /// (is it unsaved, what would it write) before deciding whether to remove
    /// it at all. Reading is not using: a close that inspects an entry and then
    /// refuses must leave it exactly where it was in the eviction order.
    pub fn get(&self, key: &BufferKey) -> Option<&Entry<T>> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, e)| e)
    }

    /// **DISCARD** `key`'s entry for good, reporting whether one was there.
    ///
    /// Deliberately NOT [`Self::take`], though the mechanism is nearly the
    /// same. `take` is the ACTIVATION half of the park/activate swap: what it
    /// removes from here is about to become the active slot, so the buffer
    /// survives the call. This ends the buffer. Spelling both as one method
    /// would make "bring this forward" and "drop this forever" indistinguishable
    /// at every call site, and the source audit that pins `take` to its single
    /// owner could no longer tell the two verbs apart.
    ///
    /// Note this is the ONLY removal that is not eviction. `park`'s clean-LRU
    /// drop is a memory-safety bound and refuses a dirty buffer; this is a
    /// product close, and the decision that the buffer is safe to end was made
    /// by its caller's own save-and-conflict gate.
    pub fn remove(&mut self, key: &BufferKey) -> bool {
        let before = self.entries.len();
        self.entries.retain(|(k, _)| k != key);
        self.entries.len() != before
    }

    /// Remove and return the entry for `key` (a buffer being brought back to
    /// the foreground), or `None` if it isn't backgrounded (first time open).
    pub fn take(&mut self, key: &BufferKey) -> Option<Entry<T>> {
        let pos = self.entries.iter().position(|(k, _)| k == key)?;
        Some(self.entries.remove(pos).1)
    }
}

#[cfg(test)]
mod tests;
