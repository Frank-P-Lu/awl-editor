//! src/app/files/external.rs — WHAT AWL DOES WHEN A FILE CHANGES UNDERNEATH IT.
//!
//! [`crate::external`] answers *did it change*. This answers *and then what*,
//! and it is the one owner of that answer: every persistence and identity
//! boundary — idle autosave, blur, focus return, buffer activation, manual
//! Save, Finish File, rename, move, Quit — asks [`App::settle_external_change`]
//! and obeys what it returns.
//!
//! # The three outcomes, and why there are exactly three
//!
//! * **Nothing moved.** Write.
//! * **The disk moved and awl holds nothing unsaved.** There is no conflict —
//!   only a stale view. Reload, keeping the caret's line/column and the scroll,
//!   so returning to the window after a `git pull` shows the new text where you
//!   were rather than an old text you might then save over it.
//! * **Both moved.** Two real manuscripts exist and awl may destroy neither.
//!   The buffer stays the one editable document; awl stops writing to the file;
//!   the unsaved text goes to the recovery record so a kill cannot take it; and
//!   the user picks.
//!
//! # What awl deliberately does not do
//!
//! No filesystem watcher — the checks happen at boundaries the user already
//! caused, so there is no thread, no inotify budget, and nothing to leak. No
//! second editable buffer, no merge, and no third file: a merge awl performed
//! would be a third version nobody wrote, and a `file.conflict.md` sibling is
//! litter in the user's own folder that they then have to clean up. The
//! recovery record lives under awl's data root and is deleted on resolution.
//!
//! # Deletion is a change too
//!
//! A file that vanished is not a licence to recreate it. Someone deleted it on
//! purpose, or moved it, or a checkout dropped it — and awl writing its buffer
//! back would silently undo that. So a deletion latches like any other change,
//! with no disk version to take (see [`crate::app::persistence::UnresolvedChange::theirs`]).

use crate::app::*;
use std::path::{Path, PathBuf};

/// THE STICKY LINE the conflict raises, and the vocabulary every user-facing
/// document is pinned to. It names the two resolutions by the exact words that
/// run them in Commands, because a notice that describes a state without naming
/// a way out is a dead end — which is precisely what its predecessor
/// ("reopen for theirs") was, since no reopen path existed.
pub(crate) const CHANGED_ELSEWHERE_NOTICE: &str =
    "This file changed outside Awl — Use disk version, or Keep my version";
pub(in crate::app) const DELETED_ELSEWHERE_NOTICE: &str =
    "This file was deleted outside Awl — Save to restore, or Close without saving";
const UNREADABLE_NOTICE: &str =
    "This file cannot be read — changes are held; check access and try again";

fn read_disk(path: &Path) -> (persistence::ExternalDiskState, Option<String>) {
    use persistence::ExternalDiskState;
    let fs = crate::fs::active();
    if let Err(error) = fs.metadata(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        return (ExternalDiskState::Unreadable, None);
    }
    match crate::openable::read_text(path) {
        Ok(text) => (ExternalDiskState::Modified, Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            (ExternalDiskState::Deleted, None)
        }
        Err(_) => (ExternalDiskState::Unreadable, None),
    }
}

/// The scratch stash's own version of the same trouble: two awl windows sharing
/// one stash. Deliberately a different sentence, because the resolutions above
/// are about a file the user named and this is not.
pub(in crate::app) const SCRATCH_CHANGED_NOTICE: &str =
    "scratch changed elsewhere — this window's copy is held";

/// WHAT A WRITE DOOR MAY DO, once the disk has been looked at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum WritePermission {
    /// The disk still holds what awl last saw. Proceed.
    Clear,
    /// The buffer was clean and the disk had moved, so the buffer has ALREADY
    /// been reloaded from disk by the time this returns. There is nothing left
    /// to write; a caller that was about to save must simply stop.
    Reloaded,
    /// Both sides changed. The write is refused and a conflict is latched.
    Held,
}

impl App {
    pub(in crate::app) fn settle_external_change_if_document(&mut self) {
        if self.document.has_active() {
            self.settle_external_change();
        }
    }

    /// **THE ONE GUARD.** Look at the active document's file and settle what may
    /// happen next; see the module doc for the three outcomes.
    ///
    /// An existing conflict holds writes while refreshing the observed disk
    /// state, so reappearance and access failures update the available choices.
    /// An unchanged file adopts its fresh observation as the next baseline.
    ///
    /// A path-less buffer (the scratch surface, an unnamed fresh document) has
    /// no file to be raced for and returns `Clear` — the stash has its own arm
    /// in the autosave engine.
    pub(in crate::app) fn settle_external_change(&mut self) -> WritePermission {
        let Some(path) = self.document.buffer().path().map(Path::to_path_buf) else {
            return WritePermission::Clear;
        };
        if self.persistence.unresolved_for(&path) {
            let (disk_state, theirs) = read_disk(&path);
            if self
                .persistence
                .unresolved()
                .is_some_and(|held| held.disk_state != disk_state || held.theirs != theirs)
            {
                self.latch_unresolved(path, disk_state, theirs);
            }
            return WritePermission::Held;
        }
        let (change, seen) = crate::external::look(&path, &self.document.disk_baseline());
        match change {
            crate::external::Change::Unchanged => {
                self.document.adopt_disk_baseline(seen);
                WritePermission::Clear
            }
            // A file that appeared where awl believed there was none is somebody
            // else's file now. Treated exactly like a modification: awl's write
            // would destroy it just as thoroughly.
            crate::external::Change::Modified | crate::external::Change::Appeared => {
                if self.is_document_dirty() {
                    let (disk_state, theirs) = read_disk(&path);
                    self.latch_unresolved(path, disk_state, theirs);
                    WritePermission::Held
                } else {
                    if self.reload_clean_document() {
                        WritePermission::Reloaded
                    } else {
                        let (disk_state, theirs) = read_disk(&path);
                        self.latch_unresolved(path, disk_state, theirs);
                        WritePermission::Held
                    }
                }
            }
            // A deletion always latches, clean buffer or not: there is nothing
            // to reload TO, and re-creating the file would undo the deletion.
            crate::external::Change::Deleted | crate::external::Change::Unreadable => {
                let (disk_state, theirs) = read_disk(&path);
                self.latch_unresolved(path, disk_state, theirs);
                WritePermission::Held
            }
        }
    }

    /// Is a conflict open right now? The predicate every gated door reads.
    pub(in crate::app) fn change_unresolved(&self) -> bool {
        self.persistence.unresolved().is_some()
    }

    /// SEND THE USER BACK TO THE CONFLICT: raise the sticky line again and
    /// refuse whatever was asked. Returns `true` when it refused, so a caller
    /// reads as `if self.refuse_while_unresolved() { return; }`.
    pub(in crate::app) fn refuse_while_unresolved(&mut self) -> bool {
        if !self.change_unresolved() {
            return false;
        }
        self.set_sticky_notice(self.external_change_notice());
        self.request_frame();
        true
    }

    /// The clean-buffer case: the file moved and awl was holding nothing of its
    /// own, so adopt the disk's text and keep the reader where they were.
    ///
    /// The caret is carried as LINE and COLUMN, not as a character offset,
    /// because the offset means something different in a document whose earlier
    /// lines just changed length; line/column at least lands on the same
    /// sentence. Both are clamped by the buffer, so a file that shrank does not
    /// leave the caret past the end.
    fn reload_clean_document(&mut self) -> bool {
        if self.document.reload_active_from_disk() {
            self.set_toast_notice("reloaded — changed elsewhere");
            self.sync_page_measure();
            self.update_title();
            self.request_frame();
            return true;
        }
        false
    }

    fn external_change_notice(&self) -> &'static str {
        match self.persistence.unresolved().map(|held| held.disk_state) {
            Some(persistence::ExternalDiskState::Deleted) => DELETED_ELSEWHERE_NOTICE,
            Some(persistence::ExternalDiskState::Unreadable) => UNREADABLE_NOTICE,
            Some(persistence::ExternalDiskState::Modified) | None => CHANGED_ELSEWHERE_NOTICE,
        }
    }

    pub(in crate::app) fn cancel_deleted_close(&mut self) {
        if let Some(mut held) = self.persistence.unresolved().cloned()
            && held.close_requested_version.take().is_some()
        {
            self.persistence.set_unresolved(held);
            self.set_sticky_notice(self.external_change_notice());
            self.request_frame();
        }
    }

    /// LATCH: stop writing to `path`, put the unsaved text somewhere a kill
    /// cannot reach it, and say so.
    ///
    /// The record is written BEFORE the latch is set, so there is no window in
    /// which awl believes it is holding text for the user without that text
    /// being on disk.
    fn latch_unresolved(
        &mut self,
        path: PathBuf,
        disk_state: persistence::ExternalDiskState,
        theirs: Option<String>,
    ) {
        self.write_recovery_record(&path);
        self.persistence
            .set_unresolved(persistence::UnresolvedChange {
                path,
                theirs,
                disk_state,
                close_requested_version: None,
            });
        self.set_sticky_notice(self.external_change_notice());
        self.request_frame();
    }

    /// Refresh the record with the buffer's CURRENT text. Called every time the
    /// autosave engine would have written the file — so while a conflict is
    /// open, autosave keeps doing its job, it just writes to the record instead
    /// of to the user's file. That is what makes "Esc and keep editing" safe.
    pub(in crate::app) fn write_recovery_record(&self, path: &Path) {
        if !self.active_file_is(path) {
            return;
        }
        crate::recovery::write(&crate::recovery::Record {
            path: path.to_path_buf(),
            text: self.document.buffer().text(),
        });
    }

    fn active_file_is(&self, path: &Path) -> bool {
        self.document.active_key() == Some(crate::buffers::BufferKey::path(path))
    }

    fn active_unresolved(&self) -> Option<&persistence::UnresolvedChange> {
        self.persistence
            .unresolved()
            .filter(|held| self.active_file_is(&held.path))
    }

    /// **REVIEW THE CHANGE** — summon the conflict workspace over the latched
    /// conflict: a list of the three read-only views beside the one it names.
    ///
    /// It reads and shows; it resolves nothing. That separation is the product
    /// decision, not an omission: the two resolutions each destroy a version, so
    /// each is reached by its own named palette row rather than by pressing `↵`
    /// on a page of prose. `Esc` from here lands back in the editor with the
    /// conflict exactly as it was, which is what the persistent affordance is for.
    ///
    /// A no-op with nothing latched. Its palette row is hidden then anyway
    /// (`commands::RowGates`), so this is the belt to that row's braces — the
    /// same pairing the two resolutions already have, and the reason a rebound
    /// chord cannot open an empty workspace.
    pub(in crate::app) fn review_external_change(&mut self) {
        let Some(unresolved) = self.persistence.unresolved() else {
            return;
        };
        if unresolved.disk_state == persistence::ExternalDiskState::Unreadable {
            self.set_sticky_notice(UNREADABLE_NOTICE);
            self.request_frame();
            return;
        }
        let card = crate::overlay::OverlayState::new_conflict(
            unresolved.path.clone(),
            unresolved.theirs.clone(),
        );
        self.workspace_state.summon_conflict(card);
        self.request_frame();
    }

    /// **SAVE YOUR VERSION** — write the buffer over the file, resolving the
    /// conflict.
    ///
    /// It RE-READS the disk first, and that recheck has teeth: between raising
    /// the conflict and the user deciding, the file may have moved AGAIN (a sync
    /// client is not patient). If what is there now is not what the user was
    /// shown, the conflict is re-raised against the new text rather than
    /// overwritten — the user consented to replacing one version, not any
    /// version.
    pub(in crate::app) fn resolve_keep_mine(&mut self) {
        let Some(unresolved) = self.active_unresolved().cloned() else {
            return;
        };
        let path = unresolved.path.clone();
        let (disk_state, now_on_disk) = read_disk(&path);
        if disk_state == persistence::ExternalDiskState::Unreadable
            || disk_state != unresolved.disk_state
            || now_on_disk != unresolved.theirs
        {
            self.latch_unresolved(path, disk_state, now_on_disk);
            if disk_state != persistence::ExternalDiskState::Unreadable {
                self.set_sticky_notice("changed elsewhere again — check before saving");
            }
            return;
        }
        let bytes = self.document.buffer().disk_bytes();
        #[cfg(not(target_arch = "wasm32"))]
        let saved = if disk_state == persistence::ExternalDiskState::Deleted {
            crate::durable::write_new(crate::durable::Owner::ManualSave, &path, &bytes)
        } else {
            crate::durable::write(crate::durable::Owner::ManualSave, &path, &bytes)
        };
        // The browser backend is synchronous and has no interleaving publisher.
        #[cfg(target_arch = "wasm32")]
        let saved = crate::durable::write(crate::durable::Owner::ManualSave, &path, &bytes);
        match saved {
            Ok(()) => {
                let version = self.document.buffer().version();
                self.document.record_document_saved(
                    version,
                    crate::external::Seen::after_write(&path, &bytes),
                );
                self.persistence.take_unresolved();
                crate::recovery::clear_for(&path);
                self.snapshot_after_save();
                let now = self.frame.now();
                self.persistence.record_save(now);
                self.set_toast_notice(if disk_state == persistence::ExternalDiskState::Deleted {
                    "restored the file"
                } else {
                    "kept your version"
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let (disk_state, theirs) = read_disk(&path);
                self.latch_unresolved(path.clone(), disk_state, theirs);
            }
            Err(error) => self.set_sticky_notice(format!("save failed: {error}")),
        }
        self.update_title();
        self.request_frame();
    }

    /// **USE DISK VERSION** — replace the buffer with what is on the file, as
    /// ONE undoable edit.
    ///
    /// `Buffer::set_text` is a single sealed edit, so `⌘Z` brings the user's own
    /// text straight back and the conflict does not need to have been kept
    /// alive to make that possible. Undoing it makes the buffer dirty again
    /// against a baseline that now matches the disk, which is correct: the
    /// user has re-chosen their version and an ordinary save will write it.
    ///
    /// A DELETED file has no version to take, and this declines rather than
    /// replacing a manuscript with nothing.
    pub(in crate::app) fn resolve_take_theirs(&mut self) {
        let Some(unresolved) = self.active_unresolved().cloned() else {
            return;
        };
        let path = unresolved.path.clone();
        let (disk_state, theirs) = read_disk(&path);
        if disk_state != unresolved.disk_state
            || disk_state == persistence::ExternalDiskState::Unreadable
        {
            self.latch_unresolved(path, disk_state, theirs);
            return;
        }
        if disk_state == persistence::ExternalDiskState::Deleted {
            self.close_deleted_buffer();
            return;
        }
        let Some(theirs) = theirs else {
            return;
        };
        // A NAMED EXEMPTION from the census's wall
        // (`app/input/text_door.rs`): the conflict card pressing this verb IS a
        // read-only prose surface, and this is one of the two resolutions it
        // exists to offer.
        self.write_document_text(TextDoor::ConflictTakeTheirs, TextEdit::Whole(&theirs));
        let version = self.document.buffer().version();
        self.document
            .record_document_saved(version, crate::external::Seen::at(&path));
        self.persistence.take_unresolved();
        crate::recovery::clear_for(&path);
        self.sync_page_measure();
        self.update_title();
        self.set_toast_notice(format!(
            "using the disk version · {} to undo",
            crate::keyspec::undo_chord_label()
        ));
        self.request_frame();
    }

    /// Is the sticky line one this guard raised? Both spellings count — the
    /// document's and the scratch stash's — because the one thing every caller
    /// wants to know is "may I clear this without swallowing the guard's own
    /// message".
    pub(in crate::app) fn clobber_notice_active(&self) -> bool {
        self.frame.notice().kind() == crate::app::NoticeKind::Sticky
            && matches!(
                self.frame.notice().text(),
                Some(CHANGED_ELSEWHERE_NOTICE)
                    | Some(SCRATCH_CHANGED_NOTICE)
                    | Some(DELETED_ELSEWHERE_NOTICE)
                    | Some(UNREADABLE_NOTICE)
            )
    }

    /// QUIT ROUTES BACK THROUGH RESOLUTION — once. The first attempt is
    /// deferred so the conflict is seen; every attempt after it proceeds,
    /// because by then the recovery record has already made quitting lossless,
    /// and refusing forever would trap someone whose only way out is a decision
    /// they are not ready to make. Returns whether the quit was deferred.
    pub(in crate::app) fn defer_quit_once_for_conflict(&mut self) -> bool {
        if !self.persistence.defer_quit_for_conflict() {
            return false;
        }
        self.set_sticky_notice(self.external_change_notice());
        if let Some(path) = self
            .document
            .buffer_opt()
            .and_then(|buffer| buffer.path())
            .map(|p| p.to_path_buf())
        {
            self.write_recovery_record(&path);
        }
        self.request_frame();
        true
    }

    /// RELAUNCH RECOVERY at startup: if the one record belongs to the document
    /// that ended up active, take it. Separate from [`Self::adopt_unresolved_for`]
    /// only so the construction site stays one line.
    pub(in crate::app) fn adopt_unresolved_after_startup(&mut self) {
        if let Some(path) = self
            .document
            .buffer_opt()
            .and_then(|buffer| buffer.path())
            .map(|p| p.to_path_buf())
        {
            self.adopt_unresolved_for(&path);
        }
    }

    /// RELAUNCH RECOVERY: if the one record belongs to `path`, put its text back
    /// into the buffer and re-raise the conflict against whatever the disk says
    /// now.
    ///
    /// Called wherever a document becomes active — startup and every open — so
    /// the record is found by opening the file it belongs to, rather than only
    /// on the launch that happens to reopen it.
    ///
    /// The record is NOT cleared when it belongs to a different file: it is the
    /// only copy of that text, and it is not this document's business.
    pub(in crate::app) fn adopt_unresolved_for(&mut self, path: &Path) {
        let Some(record) = crate::recovery::read_for(path) else {
            return;
        };
        if !crate::recovery::matches_path(&record, path) {
            return;
        }
        // The user's text is what awl was holding; the disk's text may have
        // moved again while awl was closed, so it is re-read rather than
        // remembered.
        // A NAMED EXEMPTION from the census's wall
        // (`app/input/text_door.rs`): this runs as a document BECOMES active,
        // before any card can be up, and puts back text the user already had.
        self.write_document_text(
            TextDoor::RelaunchRecoveryAdopt,
            TextEdit::Whole(&record.text),
        );
        let (disk_state, theirs) = read_disk(path);
        self.persistence
            .set_unresolved(persistence::UnresolvedChange {
                path: path.to_path_buf(),
                theirs,
                disk_state,
                close_requested_version: None,
            });
        self.set_sticky_notice(self.external_change_notice());
    }
}
