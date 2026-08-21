//! src/app/files/close.rs — **THE ONE REMOVAL OWNER for the working set.**
//!
//! Before this module, awl could not close a file. ⌘W *finished* one — it saved
//! the active buffer through the external-change guard, notified any daemon
//! waiter, and switched away — but the buffer it left stayed parked in the
//! registry and stayed a row in the margin. The only path that ever removed an
//! entry was [`crate::buffers::BufferRegistry`]'s clean-LRU eviction, which
//! refuses a dirty buffer because it is a MEMORY-SAFETY bound, not a product
//! verb: it exists so a long session does not grow without limit, and it is
//! deliberately incapable of being told "the user meant this".
//!
//! So closing needed three things that existed nowhere: a save of an entry that
//! is not `self.document.buffer()`, a conflict gate for that entry, and a
//! daemon notification for its key rather than for whatever happens to be
//! active. All three live here, and both routes — ⌘W and a stack row's close
//! zone — go through [`App::close_buffer`].
//!
//! # Why the parked arm cannot simply reuse the active one
//!
//! [`App::settle_external_change`] is the one write guard, and two of its three
//! outcomes are structurally active-only:
//!
//! * `Reloaded` replaces the buffer the reader is looking at with the disk's
//!   text. For an entry about to be discarded that is pure waste, and it would
//!   raise a `reloaded — changed elsewhere` toast about a document that is gone
//!   by the time the frame draws.
//! * `Held` latches the conflict into `persistence`'s SINGLE unresolved slot and
//!   writes THE recovery record. Both are active-scoped, and pointing them at a
//!   parked path is not a lesser version of the guard — it is a data-loss bug:
//!   `resolve_keep_mine` writes `self.document.buffer()`'s bytes to
//!   `unresolved.path`, so a conflict latched for a parked file would let the
//!   user's next "Save your version" write the WRONG DOCUMENT over it.
//!
//! The parked arm therefore asks the same question — has the disk moved since
//! awl last looked — and answers it by REFUSING, without latching anything. The
//! entry stays exactly where it was, unsaved text intact, and the notice names
//! both the file and the way out, because the conflict machinery only works on
//! the active buffer and a notice describing a state with no exit is a dead end.
//!
//! Closing the last file unseats the active entry after the same save/conflict
//! gates and leaves `DocumentSession::active == None`. That absence is carried
//! end-to-end; this owner never manufactures an unnamed replacement buffer.

use super::WritePermission;
use crate::app::*;
use std::path::Path;

/// Whether a close actually happened. `Refused` is a first-class outcome, not
/// an error: refusing to discard unsaved or conflicted text is the guarantee
/// this module exists to make.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum CloseOutcome {
    Closed,
    /// Nothing was removed and nothing was lost. The entry is still open.
    Refused,
}

impl App {
    /// Move ONE named document to the operating system's recoverable Trash,
    /// then release its working-set entry through this module's existing
    /// successor/removal ownership. This deliberately does not reuse `close`:
    /// close is authorized to save a dirty document, while Trash must refuse a
    /// dirty or externally changed document before it asks the OS to move the
    /// only on-disk copy.
    pub(in crate::app) fn trash_buffer(&mut self, key: crate::buffers::BufferKey) -> CloseOutcome {
        let Some(facts) = self.document.close_facts(&key) else {
            return CloseOutcome::Refused;
        };
        let Some(path) = facts.path else {
            return CloseOutcome::Refused;
        };
        if facts.unsaved {
            self.refuse_parked(&path, "has unsaved text — save it before moving to Trash");
            return CloseOutcome::Refused;
        }
        if self.document.active_key().as_ref() == Some(&key) {
            // The active guard can safely reload a clean document before the
            // irreversible-in-this-process handoff. A held conflict stays
            // resolved through its existing workspace; never trash through it.
            if !matches!(
                self.settle_external_change(),
                WritePermission::Clear | WritePermission::Reloaded
            ) || self.change_unresolved()
            {
                return CloseOutcome::Refused;
            }
        } else if self.persistence.unresolved_for(&path)
            || crate::external::look(&path, &facts.baseline).0 != crate::external::Change::Unchanged
        {
            self.refuse_parked(&path, "changed elsewhere — open it to resolve");
            return CloseOutcome::Refused;
        }
        if let Err(message) = crate::assets::active_trash().trash(&path) {
            self.set_sticky_notice(format!("couldn't move to Trash: {message}"));
            self.request_frame();
            return CloseOutcome::Refused;
        }
        // A successful Trash ends this exact buffer just as surely as Finish
        // file. Drain the keyed EDITOR waiter only after the OS accepted the
        // recoverable move; every refusal above leaves it connected.
        self.notify_close_waiters(&key);
        if self.document.active_key().as_ref() == Some(&key) {
            if self.remove_active_entry() {
                CloseOutcome::Closed
            } else {
                CloseOutcome::Refused
            }
        } else if self.document.discard(&key) {
            self.document.forget_previous(&path);
            self.sync_view(false);
            self.request_frame();
            CloseOutcome::Closed
        } else {
            CloseOutcome::Refused
        }
    }

    /// **CLOSE THE BUFFER NAMED BY `key`**, active or not.
    ///
    /// One door for both routes, so "what does closing a file mean" has one
    /// answer. The pointer does not know whether the row it was aimed at is the
    /// file on screen, and it must not have to: a row action targets the named
    /// buffer rather than silently switching documents to make an
    /// active-buffer-only function convenient.
    pub(in crate::app) fn close_buffer(&mut self, key: crate::buffers::BufferKey) -> CloseOutcome {
        if self.document.active_key().as_ref() == Some(&key) {
            self.close_active_now()
        } else {
            self.close_parked(key)
        }
    }

    /// The ACTIVE arm, spelled as the exact sequence ⌘W's three effects run in
    /// (`Save(Finish)`, `NotifyFinished`, `CloseActive`) and calling the very
    /// same methods — same behavior ⇒ same code. A row's close zone over the
    /// active file is ⌘W, not a second implementation of it.
    fn close_active_now(&mut self) -> CloseOutcome {
        if !self.try_save_finished_buffer() {
            return CloseOutcome::Refused;
        }
        self.notify_finished_buffer();
        if self.remove_active_entry() {
            CloseOutcome::Closed
        } else {
            CloseOutcome::Refused
        }
    }

    /// **The `CloseActive` effect's leg**: remove the active entry, having
    /// already been saved and notified by the two effects before it.
    ///
    /// The gate ran in [`Self::save_finished_buffer`]; a latched conflict means
    /// it REFUSED the write, and a buffer awl would not write is a buffer awl
    /// must not drop. That check is the whole difference between this and the
    /// `last_buffer_toggle` it replaced.
    ///
    /// The document state owner returns the gated entry explicitly, then this
    /// close owner releases it. A successor moves by registry identity, which
    /// includes the path-less scratch; no successor means honest absence.
    pub(in crate::app) fn close_active_buffer(&mut self) {
        self.remove_active_entry();
    }

    /// The removal itself, reporting whether anything was removed so the
    /// pointer route can tell a refusal from a close.
    fn remove_active_entry(&mut self) -> bool {
        if self.change_unresolved() {
            return false;
        }
        let Some(key) = self.document.active_key() else {
            return false; // an unnamed fresh note has no identity to close
        };
        let closing_path = self.document.buffer().path().map(|p| p.to_path_buf());
        // The save/conflict gate above is what authorizes releasing the entry.
        // Assert its postcondition before ownership moves; `unseat_active`
        // returns the entry to this owner so the state layer never discards it.
        if self
            .document
            .close_facts(&key)
            .is_some_and(|facts| facts.unsaved)
        {
            return false;
        }
        let successor = self.document.successor_key(&key);
        let Some(closed) = self.document.unseat_active(&key, successor.as_ref()) else {
            return false;
        };
        // `closed` is released only here, after the lossless gate above.
        closed.release();
        let activated_path = self
            .document
            .buffer_opt()
            .and_then(|buffer| buffer.path())
            .map(Path::to_path_buf);
        if let Some(path) = closing_path {
            self.document.forget_previous(&path);
        }
        self.workspace_state.dismiss_pickers();
        if successor.is_some() {
            self.finish_buffer_activation(activated_path, false);
        } else {
            // A reload notice describes the document that just left. With no
            // successor there is no subject for any document notice, so the
            // calm start surface begins without stale arrival chrome.
            self.clear_notice();
            self.input.clear_preedit();
            self.update_title();
            self.sync_view(false);
            self.request_frame();
        }
        true
    }

    /// The PARKED arm: close a named entry that is not the active document,
    /// without activating it first.
    fn close_parked(&mut self, key: crate::buffers::BufferKey) -> CloseOutcome {
        let Some(facts) = self.document.close_facts(&key) else {
            return CloseOutcome::Refused; // no such entry
        };
        if facts.unsaved && !self.save_parked(&key, facts.path.as_deref(), facts.baseline) {
            return CloseOutcome::Refused;
        }
        self.notify_close_waiters(&key);
        if !self.document.discard(&key) {
            return CloseOutcome::Refused;
        }
        if let Some(path) = facts.path {
            self.document.forget_previous(&path);
        }
        self.sync_view(false);
        self.request_frame();
        CloseOutcome::Closed
    }

    /// SAVE A PARKED ENTRY through the generalized conflict gate. Returns
    /// whether the close may proceed.
    ///
    /// Every refusal below leaves the entry byte-identical — including its
    /// `disk_baseline`, which is deliberately NOT adopted on the refusing path,
    /// so a second attempt re-looks at the disk rather than deciding from a
    /// baseline this call moved.
    fn save_parked(
        &mut self,
        key: &crate::buffers::BufferKey,
        path: Option<&Path>,
        baseline: crate::external::Seen,
    ) -> bool {
        let Some(path) = path else {
            // A path-less parked scratch must be activated before its stash
            // conflict guard can run; its stack row is now a real activation
            // door, so this refusal tells the reader the available route.
            self.set_sticky_notice("scratch has unsaved text — open it before closing");
            self.request_frame();
            return false;
        };
        // A conflict already latched for this path belongs to a document the
        // user is being asked to resolve. Never write past it.
        if self.persistence.unresolved_for(path) {
            self.refuse_parked(path, "changed elsewhere — open it to resolve");
            return false;
        }
        let (change, seen) = crate::external::look(path, &baseline);
        if change != crate::external::Change::Unchanged {
            self.refuse_parked(path, "changed elsewhere — open it to resolve");
            return false;
        }
        let Some(bytes) = self.document.parked_disk_bytes(key) else {
            return false;
        };
        if let Err(e) = crate::durable::write(crate::durable::Owner::ManualSave, path, &bytes) {
            // A FAILED WRITE IS A REFUSAL, not a detail to log past: the entry
            // still holds the only copy of that text.
            self.set_sticky_notice(format!("save failed: {e}"));
            self.request_frame();
            return false;
        }
        // The local-history snapshot every other save takes, read from the
        // ENTRY's own text rather than from the active buffer, which is a
        // different document. `snapshot_after_save` cannot be reused for
        // exactly that reason.
        if let Some(text) = self.document.parked_text(key) {
            crate::history::record(path, &text, &self.config);
        }
        // No saved-version bookkeeping: a successful parked save is always
        // followed by discarding the entry, so there is no later reader.
        let _ = seen;
        true
    }

    /// Say WHICH file could not be closed and what to do about it. The leaf
    /// alone, because the row the pointer was on shows the leaf — and because a
    /// full path in a one-line notice is the part that gets elided away.
    fn refuse_parked(&mut self, path: &Path, tail: &str) {
        let leaf = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        self.set_sticky_notice(format!("{leaf} {tail}"));
        self.request_frame();
    }

    /// Notify and drop every daemon connection waiting on `key`.
    ///
    /// Keyed rather than derived from the active buffer, which is the piece
    /// that did not exist: a `--wait` client blocked on a file the reader has
    /// since switched away from is still owed its answer when that file closes.
    /// A no-op on wasm and under `mas`, where the daemon compiles out entirely.
    pub(in crate::app) fn notify_close_waiters(&mut self, key: &crate::buffers::BufferKey) {
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
        if let Some(waiters) = self.wait_conns.remove(key) {
            for w in waiters {
                w.notify_done();
            }
        }
        #[cfg(any(target_arch = "wasm32", feature = "mas"))]
        let _ = key;
    }
}

#[cfg(test)]
mod tests;
