//! Close-time access to active and parked entries.
//!
//! Read unsaved state, disk baseline, and save bytes before removal. Discarding
//! an entry removes its registry entry and working-set row together.

use super::*;

/// WHAT THE REMOVAL OWNER NEEDS TO KNOW about one entry before it may close it.
///
/// Deliberately a snapshot rather than a borrow of the entry: the gate that
/// reads it goes on to touch the filesystem and the notice slot, and a live
/// borrow of the registry across that would either fight the borrow checker or
/// invite a re-read that silently disagrees with the decision already made.
pub(in crate::app) struct CloseFacts {
    /// The entry's own path. `None` for the path-less scratch surface.
    pub(in crate::app) path: Option<PathBuf>,
    /// Is this entry holding text that is not on disk?
    pub(in crate::app) unsaved: bool,
    /// What awl last SAW at `path` — stat AND digest, the conflict gate's own
    /// input. Meaningless (and unread) when `path` is `None`.
    pub(in crate::app) baseline: crate::external::Seen,
}

/// Ownership of an entry that has been unseated by the close owner. Its
/// private payload prevents any caller from reading or mutating a half-closed
/// document; only the explicit release below can finish the already-gated
/// close.
pub(in crate::app) struct CloseRelease(crate::buffers::Entry<BufferExtra>);

impl CloseRelease {
    pub(in crate::app) fn release(self) {
        drop(self.0);
    }
}

impl DocumentSession {
    /// Shared unsaved rule for active and parked entries. Active Fresh dirty
    /// display also consults `PersistenceRuntime`; parked Fresh entries remain
    /// unsaved until naming establishes a baseline, unless discardably empty.
    fn entry_unsaved(entry: &crate::buffers::Entry<BufferExtra>) -> bool {
        if entry.buffer.is_discardable_empty_fresh() {
            return false;
        }
        let version = entry.buffer.version();
        if entry.buffer.path().is_some() {
            entry.extra.doc_saved_version != Some(version)
        } else {
            entry.extra.scratch_saved_version != Some(version)
        }
    }

    /// Is the ACTIVE buffer holding unsaved text? The same rule every parked
    /// entry is judged by, so the two can never drift.
    pub(in crate::app) fn active_unsaved(&self) -> bool {
        self.active.as_ref().is_some_and(Self::entry_unsaved)
    }

    /// The active buffer's registry identity, or `None` only when the document
    /// session itself is empty.
    pub(in crate::app) fn active_key(&self) -> Option<crate::buffers::BufferKey> {
        self.active
            .as_ref()
            .map(|active| crate::buffers::BufferKey::of(&active.buffer))
    }

    /// The facts for `key`, whether it is the active entry or a parked one.
    ///
    /// Answering for BOTH is what lets the removal owner have one shape: the
    /// pointer route does not know, and must not have to know, whether the row
    /// it was aimed at happens to be the file on screen.
    pub(in crate::app) fn close_facts(
        &self,
        key: &crate::buffers::BufferKey,
    ) -> Option<CloseFacts> {
        let entry = if self.active_key().as_ref() == Some(key) {
            self.active.as_ref()?
        } else {
            self.registry.get(key)?
        };
        Some(CloseFacts {
            path: entry.buffer.path().map(Path::to_path_buf),
            unsaved: Self::entry_unsaved(entry),
            baseline: entry.extra.disk_baseline,
        })
    }

    /// The bytes a parked entry would write, with its own remembered line
    /// ending restored — [`crate::buffer::Buffer::disk_bytes`], never
    /// `text().as_bytes()`, so closing a CRLF file does not silently rewrite
    /// every line ending in it.
    pub(in crate::app) fn parked_disk_bytes(
        &self,
        key: &crate::buffers::BufferKey,
    ) -> Option<Vec<u8>> {
        Some(self.registry.get(key)?.buffer.disk_bytes())
    }

    /// A parked entry's text, for the local-history snapshot that accompanies
    /// its save. Read from the entry itself rather than from the active buffer,
    /// which is a different document.
    pub(in crate::app) fn parked_text(&self, key: &crate::buffers::BufferKey) -> Option<String> {
        Some(self.registry.get(key)?.buffer.text())
    }

    /// What a PARKED entry's own stash flush last saw at the persistent
    /// scratch path — the parked counterpart of [`Self::scratch_baseline`],
    /// needed to close a parked true scratch through the same clobber check
    /// the active arm uses, keyed off the entry that is about to be discarded
    /// rather than whatever happens to be active.
    pub(in crate::app) fn parked_scratch_baseline(
        &self,
        key: &crate::buffers::BufferKey,
    ) -> crate::external::Seen {
        self.registry
            .get(key)
            .map(|entry| entry.extra.scratch_baseline)
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(in crate::app) fn parked_version(&self, key: &crate::buffers::BufferKey) -> Option<u64> {
        Some(self.registry.get(key)?.buffer.version())
    }

    /// **THE ONE REMOVAL DOOR.** Drop `key`'s parked entry and its working-set
    /// row together, returning whether anything was there.
    ///
    /// Both halves or neither. `registry` decides what may be evicted for
    /// memory; `working` decides what the margin draws; and an entry removed
    /// from one but not the other is either a row naming a buffer that no
    /// longer exists or a buffer no surface can reach again.
    pub(in crate::app) fn discard(&mut self, key: &crate::buffers::BufferKey) -> bool {
        let parked = self.registry.remove(key);
        let row = self.working.close_key(key).is_some();
        parked || row
    }

    /// WHICH FILE SHOULD BECOME ACTIVE when `closing` is the active entry.
    ///
    /// The working set's own neighbour rule ([`crate::workingset::WorkingSet::close`]
    /// keeps the reader near the row they closed). The path-less scratch row is
    /// a real working-set member and a real successor: `unseat_active` moves
    /// the registry entry by KEY, not through the path-taking file-open door,
    /// so activating scratch this way needs no path at all (see
    /// `the_successor_can_activate_the_pathless_scratch_row`).
    ///
    /// Searches FORWARD from the closing slot first, then backward, then gives
    /// up. `None` means "nothing else to show" and is the zero-document bound,
    /// not an error.
    pub(in crate::app) fn successor_key(
        &self,
        closing: &crate::buffers::BufferKey,
    ) -> Option<crate::buffers::BufferKey> {
        let files = self.working.files();
        let at = self.working.index_of(closing)?;
        let forward = (at + 1)..files.len();
        let backward = (0..at).rev();
        forward
            .chain(backward)
            .find_map(|i| files.get(i).map(|f| f.key.clone()))
    }

    /// Move the active entry out after the close owner has completed its save
    /// and conflict gates. The removed entry is returned to that owner instead
    /// of being discarded here, so this state transition itself cannot lose
    /// document data. `successor` may name the path-less scratch entry.
    pub(in crate::app) fn unseat_active(
        &mut self,
        closing: &crate::buffers::BufferKey,
        successor: Option<&crate::buffers::BufferKey>,
    ) -> Option<CloseRelease> {
        if self.active_key().as_ref() != Some(closing) {
            return None;
        }
        let outgoing = self.take_active()?;
        if let Some(next) = successor {
            let Some(mut incoming) = self.take_parked(next) else {
                self.active = Some(outgoing);
                return None;
            };
            incoming.buffer.take_list_continuation_generated();
            self.active = Some(incoming);
        }
        self.working.close_key(closing);
        if let Some(next) = successor
            && let Some(at) = self.working.index_of(next)
        {
            self.working.set_active(at);
        }
        Some(CloseRelease(outgoing))
    }

    /// Activate an already-open entry, including the path-less scratch slot.
    pub(in crate::app) fn activate_key(&mut self, key: &crate::buffers::BufferKey) -> bool {
        if self.active_key().as_ref() == Some(key) {
            return true;
        }
        self.previous = self
            .active
            .as_ref()
            .map(|active| crate::buffers::BufferKey::of(&active.buffer));
        let outgoing = self.active_key();
        self.park_active();
        if !self.activate(key) {
            if let Some(outgoing) = outgoing {
                let _ = self.activate(&outgoing);
            }
            return false;
        }
        if let Some(at) = self.working.index_of(key) {
            self.working.set_active(at);
        }
        true
    }

    /// Forget the 2-deep last-file target when it names `path`.
    ///
    /// Without this, ⌃Tab right after a close re-reads the closed file from
    /// disk — resurrecting, as the "previous" buffer, the one document the
    /// reader just said they were done with.
    pub(in crate::app) fn forget_previous(&mut self, path: &Path) {
        let gone = crate::buffers::BufferKey::path(path);
        if self.previous.as_ref() == Some(&gone) {
            self.previous = None;
        }
    }
}
