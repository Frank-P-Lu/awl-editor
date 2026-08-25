//! src/app/files/verbs.rs — the GENERIC FILE VERBS: manual save finish +
//! scratch-to-document conversion, rename, move, duplicate, the inline-image
//! drag-resize write-back, the asset-cleaner trash verb, and the two
//! local-history bridges (Keep version / Restore). Split out of the former
//! `app/files.rs` monolith. The old LIVE
//! rename-to-title behavior — a document's name is now derived exactly once
//! (its first material save), and Rename is the one, explicit, generic verb
//! thereafter.

use super::WritePermission;
use crate::app::*;
use std::path::Path;

impl App {
    /// Live interpreter for `PersistenceEffect::Save(Manual)`. The shared
    /// transition describes the write; only this live owner may perform it.
    /// **A manual Save no longer force-writes.** It routes through the SAME
    /// external-change guard the autosave engine does
    /// ([`Self::settle_external_change`]), because the contract it used to
    /// honour — "⌘S keeps yours" — spent one of the two manuscripts to do it,
    /// and the user pressing ⌘S was never told that is what it meant. A file
    /// that moved under an unsaved buffer now raises the conflict; the explicit
    /// **Save your version** resolution is where "keep mine" lives, and it says
    /// so before it writes.
    pub(in crate::app) fn manual_save(&mut self) {
        match self.settle_external_change() {
            WritePermission::Clear => {}
            // The buffer was clean, so the reload IS the save's whole outcome:
            // the document now matches the file it would have been written to.
            WritePermission::Reloaded => return,
            WritePermission::Held => {
                if let Some(path) = self.document.buffer().path().map(|p| p.to_path_buf()) {
                    self.write_recovery_record(&path);
                }
                return;
            }
        }
        if self.document.buffer().path().is_none() && !self.document.buffer().is_unnamed_fresh() {
            self.convert_scratch_and_save();
        } else {
            let result = self.document.save();
            let (ok, message) = match result {
                Ok(()) => (true, "saved".to_string()),
                Err(error) => (false, format!("save failed: {error}")),
            };
            self.finish_manual_save(ok, message);
        }
        if self.document.buffer().path().is_some_and(|path| {
            !self.config.path.as_os_str().is_empty() && path == self.config.path
        }) {
            self.reload_config();
        }
    }

    /// INLINE-IMAGE DRAG-RESIZE (v2) WRITE-BACK: stamp the settled `|NNN` width hint
    /// into the image's ALT text as ONE undoable edit — templated on
    /// `actions::edit::tag_document_language`'s single-`replace_char_range` shape. `range`
    /// is the `![alt](path)` span's DOCUMENT BYTE range (from the drag), `width_px` the
    /// final display width (rounded to the int hint). The pure
    /// [`crate::markdown::image_width_hint_edit`] computes the alt sub-range +
    /// replacement (Obsidian `![alt|NNN](path)`); we convert its byte offsets to buffer
    /// CHAR indices and apply one sealed replace. A non-empty replace never coalesces,
    /// so the whole drag is a single Cmd-Z (restoring the pre-drag size + text).
    ///
    /// NUANCE (from the lang-tag precedent): `replace_char_range` moves the caret to
    /// the edit end — but a MOUSE drag must NOT move the text caret. So snapshot the
    /// cursor, apply, then restore it (shifted by the edit's length delta only when it
    /// sat past the edit), so the caret stays exactly where it was.
    pub(in crate::app) fn write_back_image_width(&mut self, range: (usize, usize), width_px: f32) {
        if !self.document.buffer().is_markdown() {
            return;
        }
        let width = width_px.round().max(1.0) as u32;
        let text = self.document.buffer().text();
        let (bstart, bend) = range;
        let Some(src) = text.get(bstart..bend) else {
            return;
        };
        let Some((alt_b0, alt_b1, new_alt)) = crate::markdown::image_width_hint_edit(src, width)
        else {
            return;
        };
        // src-relative byte offsets -> absolute document byte offsets -> char indices.
        let abs_b0 = bstart + alt_b0;
        let abs_b1 = bstart + alt_b1;
        let c0 = text[..abs_b0].chars().count();
        let c1 = text[..abs_b1].chars().count();
        let new_len = new_alt.chars().count();
        // No-op guard: the alt already reads exactly the target — keep the timeline
        // meaningful (mirrors `apply_format`'s equal-text short-circuit).
        if text.get(abs_b0..abs_b1) == Some(new_alt.as_str()) {
            return;
        }
        // Snapshot the caret so the mouse drag never moves it (see the doc nuance).
        let saved = self.document.buffer().cursor_char();
        let delta = new_len as isize - (c1 - c0) as isize;
        self.document.seal_undo_group();
        self.document.replace_char_range(c0, c1, &new_alt);
        self.document.seal_undo_group();
        let restored = if saved <= c0 {
            saved
        } else if saved >= c1 {
            (saved as isize + delta).max(0) as usize
        } else {
            c0
        };
        self.document.set_cursor(restored);
    }

    /// Finish the live interpreter's explicit manual save. The typed
    /// persistence request carried no write; `manual_save` ran the one
    /// `Buffer::save` call and passes its outcome here. On SUCCESS, capture a
    /// local-history point (the store's git
    /// gate / history-off / dedup all decide what's kept) and follow the
    /// AUTOSAVE ENGINE's own bookkeeping (the buffer version is now on disk —
    /// no redundant idle write; the fresh mtime is the clobber guard's new
    /// baseline — a manual save legitimately force-writes over an external
    /// change).
    ///
    /// A successful explicit save is a short live toast. Only a genuine FAILURE
    /// stays sticky (an
    /// unnamed empty document's "save failed: empty note: nothing to save yet"
    /// included) — errors must never go silent (the round's own bug was that
    /// both fates once reached only a terminal `eprintln!`, invisible on a GUI
    /// launch). Autosave stays SILENT too — only a failed explicit user action
    /// is acknowledged.
    ///
    /// This runs only after a successful `Buffer::save`, which — one-shot
    /// naming — always leaves
    /// `is_unnamed_fresh()` false by the time this runs: either the buffer was
    /// already ordinary, or it just got named and its marker cleared in the
    /// same step. So there is no longer a distinct "note" dirty-marker branch
    /// to stamp here — the ordinary `doc_saved_version` stamp below always
    /// covers it. `autosave_note` stamps that same baseline for the
    /// DEBOUNCED naming save, so both save doors agree.)
    pub(in crate::app) fn finish_manual_save(&mut self, ok: bool, message: String) {
        if ok {
            self.snapshot_after_save();
            if let Some(p) = self.document.buffer().path().map(|p| p.to_path_buf()) {
                self.document.record_document_saved(
                    self.document.buffer().version(),
                    crate::external::Seen::at(&p),
                );
            }
            // NOTES VERBS round: the held HUD's SAVED stat.
            let now = self.frame.now();
            self.persistence.record_save(now);
            self.emit_notice(crate::actions::NoticeEffect::Toast("saved".to_string()));
        } else {
            self.emit_notice(crate::actions::NoticeEffect::Sticky(message));
        }
    }

    /// `Cmd-S` / `C-x C-s` on the TRUE scratch surface: interpret the typed
    /// manual-save request by converting the pathless buffer into a real note,
    /// reusing the exact auto-name machinery
    /// [`Self::ensure_note_named_before_paste`] already established for the
    /// paste-image door ([`crate::buffer::Buffer::save_into_folder`]: `set_note_dir`
    /// then `Buffer::save`, which derives the filename from the first line via
    /// the same `note_stem` a `C-x n` note uses), then finish the bookkeeping a
    /// normal manual save would (title, go-to index, the fresh note's own
    /// sticky page measure — a brand-new note is always PROSE, mirroring
    /// `new_document`'s resync) and RETIRE the persistent SCRATCH STASH: the
    /// content just became a real, named file, so a later bare relaunch must
    /// never resurrect a ghost copy of it from the old stash (best-effort —
    /// a failed remove never disrupts the save that already succeeded).
    /// Raises the SAME calm "saved" / "save failed: …" notice a plain manual
    /// save does — never a terminal print. An active folder that doesn't exist
    /// or isn't writable surfaces here as the failure notice, never a crash.
    ///
    /// USER-FLIPPABLE (logged, not hidden): this round settled on "scratch
    /// Save promotes to a note" as the fix for the reported bug (silent save
    /// failure on Linux) — a future preference could instead make this
    /// notice-only ("nothing to save yet — start a note first"), leaving the
    /// scratch buffer untouched. Both are one function to swap here.
    pub(in crate::app) fn convert_scratch_and_save(&mut self) {
        match self.document.save_into_folder(&self.project_location.root) {
            Ok(()) => {
                // `Buffer::save_into_folder` already stamped the derived path onto
                // the buffer itself (the sole authoritative path).
                self.update_title();
                self.rescan_file_index();
                self.sync_page_measure();
                // RETIRE THE STASH: best-effort, mirroring every other
                // fallible bookkeeping call in this file — a failed remove
                // never disrupts the save that already succeeded.
                let _ = crate::fs::active().remove_file(&crate::fs::scratch_stash_path());
                self.document.clear_scratch_saved();
                // The note's own debounced autosave now owns this buffer;
                // mark the version we just wrote as already-saved so the
                // next idle tick doesn't immediately rewrite it (mirrors
                // `autosave_note`'s own post-save bookkeeping).
                self.persistence
                    .record_note_write(self.document.buffer().version());
                self.snapshot_after_save();
                if let Some(p) = self.document.buffer().path().map(|p| p.to_path_buf()) {
                    self.document.record_document_saved(
                        self.document.buffer().version(),
                        crate::external::Seen::at(&p),
                    );
                }
                self.emit_notice(crate::actions::NoticeEffect::Toast("saved".to_string()));
                // NOTES VERBS round: the held HUD's SAVED stat.
                let now = self.frame.now();
                self.persistence.record_save(now);
            }
            Err(e) => {
                self.emit_notice(crate::actions::NoticeEffect::Sticky(format!(
                    "save failed: {e}"
                )));
            }
        }
        self.request_frame();
    }

    /// THE CONSCIOUS MARK ("Keep version…"): record the CURRENT buffer state as a
    /// PINNED, prune-EXEMPT local-history snapshot ([`crate::history::record_pinned`]),
    /// optionally NAMED (`name` = the naming minibuffer's typed text, `None` for a
    /// blank Enter — the plain keep; a NAMED SAVE POINT renders its name as the
    /// timeline's primary cell and is prune-exempt like any pin). Keyed on the SAME
    /// path the snapshot store records/restores under
    /// ([`crate::history::source_path`]: the buffer's own path, else the
    /// persistent scratch's stash path — so the scratch can be pinned too). A
    /// no-op for an unnamed note (no history key yet), a git-managed file (git owns
    /// its versioning — awl pins nothing there, named or not: the pre-name story,
    /// unchanged), or `history = false`; the store itself enforces those gates.
    /// Best-effort: any store error is swallowed inside `record_pinned`, so a failed
    /// pin never disrupts the buffer.
    pub(in crate::app) fn keep_version(&self, name: Option<&str>) {
        let path = crate::history::source_path(
            self.document.buffer().path(),
            self.document.buffer().is_unnamed_fresh(),
        );
        if let Some(path) = path {
            crate::history::record_pinned(
                &path,
                &self.document.buffer().text(),
                &self.config,
                name,
            );
        }
    }

    /// RESTORE a local-history VERSION into the buffer (the summoned timeline's Enter).
    /// Resolves `id` to its captured content via [`crate::history::load`] — the awl log
    /// for a loose file, `git show` for a git-managed one — and replaces the whole
    /// buffer with it via [`crate::buffer::Buffer::set_text`], which is ONE atomic,
    /// undoable edit (so C-/ undoes the restore, exactly like any other edit). Keyed on
    /// the SAME path the snapshot store records under ([`crate::history::source_path`]:
    /// `buffer.path()`, else the persistent scratch's stash path — so the scratch
    /// timeline restores too). A no-op for an unnamed note, or an unknown /
    /// unresolvable id (best-effort — a failed restore must never disrupt the buffer).
    /// AND ONE CALM NOTICE, naming both the version and the way back.
    /// A restore replaces the whole document silently; DESIGN.md's calm bias makes
    /// that the one place a toast earns its keep, because the alternative is a
    /// user who cannot tell whether the workspace did anything and does not know
    /// that `⌘Z` covers it. `notice_readout_text` returns the moment the workspace
    /// closes, so it lands on exactly the frame the document changed.
    ///
    /// `Esc` deliberately emits NOTHING: it undoes a view substitution and the
    /// document never changed, and a toast confirming a no-op is the nagging the
    /// same bias forbids.
    pub(in crate::app) fn restore_history(&mut self, id: &str) {
        let path = crate::history::source_path(
            self.document.buffer().path(),
            self.document.buffer().is_unnamed_fresh(),
        );
        if let Some(path) = path
            && let Some(content) = crate::history::load(&path, id)
        {
            self.document.set_text(&content);
            let label = crate::history::version_label(&path, id, crate::history::now_millis());
            if let Some(label) = label {
                self.set_toast_notice(format!(
                    "restored \"{label}\" · {} to undo",
                    crate::keyspec::undo_chord_label()
                ));
            }
        }
    }

    /// ASSET CLEANER: move the orphan at root-relative `rel` to the OS Trash
    /// (recoverable — never `rm`), then — ONLY on success — remove its row from the
    /// still-open picker (`OverlayState::remove_asset_row`), so the list shrinks as you
    /// clean and the picker stays up. A failure (a missing file, a non-macOS platform,
    /// an OS refusal) LEAVES the row and shows a calm dim notice. The trash goes through
    /// the injectable [`crate::assets::TrashCan`] seam, so a test drives it with a fake
    /// (the REAL macOS `NSFileManager` call is live-only, flagged).
    pub(in crate::app) fn trash_asset(&mut self, rel: String) {
        let abs = self.project_location.root.join(&rel);
        match crate::assets::active_trash().trash(&abs) {
            Ok(()) => {
                if let Some(ov) = self.workspace_state.overlay_mut() {
                    ov.remove_asset_row(&rel);
                    ov.notice.clear();
                }
            }
            Err(msg) => {
                if let Some(ov) = self.workspace_state.overlay_mut() {
                    ov.notice = format!("couldn't move to Trash: {msg}");
                }
            }
        }
    }

    /// C-x m accept: MOVE the current file into `dest_rel` (a directory relative
    /// to the ACTIVE folder — the same folder Cmd-N creates a document
    /// in, not a separate notes root; `""` = the active folder itself), keeping
    /// the filename. Creates the destination folder if needed, refuses to
    /// clobber (numeric suffix), then re-points the buffer so editing/auto-save
    /// continue at the new path. A true `std::fs::rename` move — never a copy.
    ///
    /// The navigator only ever composes `dest_rel` from real folder names or
    /// the single-segment create-a-folder gate, but a `..` segment is refused
    /// here too, belt-and-braces.
    pub(in crate::app) fn move_current_file(&mut self, dest_rel: &str) {
        // A move is an IDENTITY boundary: doing it while one version of the
        // file is unwritten would strand the conflict on a dead path.
        if self.refuse_while_unresolved() {
            return;
        }
        if dest_rel.split('/').any(|seg| seg == "..") {
            self.set_sticky_notice("can't move above the current folder".to_string());
            return;
        }
        let Some(old) = self.document.buffer().path().map(|p| p.to_path_buf()) else {
            return; // no current file to move
        };
        let dest_dir = if dest_rel.is_empty() {
            self.project_location.root.clone()
        } else {
            self.project_location.root.join(dest_rel)
        };
        // The actual mkdir + no-clobber + rename lives in `buffer::move_file`.
        let new_path = match crate::buffer::move_file(&old, &dest_dir) {
            Ok(p) => p,
            Err(e) => {
                // A failure gets the SAME calm bottom-center notice a failed save does.
                self.set_sticky_notice(format!("move failed: {e}"));
                return;
            }
        };
        if new_path == old {
            return; // already there: nothing changed
        }
        self.document.set_path(new_path.clone());
        // The guard's baseline is keyed to a PATH; re-take it at the new one
        // or a later check compares against a file it no longer lives in.
        self.document
            .adopt_disk_baseline(crate::external::Seen::at(&new_path));
        // KEEP THE STACK SLOT: `rekey_active` re-points the working set's own
        // row in place (its doc explains why `open()` cannot).
        self.document.working_set_mut().rekey_active(
            crate::buffers::BufferKey::path(&new_path),
            Some(new_path.clone()),
        );
        // An UNNAMED fresh document being moved before its first save (rare —
        // the picker only opens for a pathed file) keeps auto-saving into its
        // new home.
        if self.document.buffer().is_unnamed_fresh() {
            self.document.set_note_dir(dest_dir);
        }
        self.update_title();
        self.rescan_file_index();
        // The notice names the full new path, root-relative (never
        // absolute). Moving never rewrites links, so it flags a relative one.
        let rel_display = new_path
            .strip_prefix(&self.project_location.root)
            .unwrap_or(&new_path)
            .to_string_lossy()
            .replace('\\', "/");
        let needs_link_review = self.document.buffer().is_markdown()
            && crate::markdown::has_relative_references(&self.document.buffer().text());
        if needs_link_review {
            self.set_sticky_notice(format!(
                "moved to {rel_display} — relative links/images may need review"
            ));
        } else {
            self.set_toast_notice(format!("moved to {rel_display}"));
        }
        self.request_frame();
    }

    /// NOTES VERBS round: RENAME the current file to `new_name` (a bare filename,
    /// same directory — the minibuffer never lets a typed name cross directories,
    /// see `RenameEdit::push`'s `/`-rejection). THE ONE OWNER of every path-keyed
    /// store that must follow a rename: the buffer's own path and the
    /// local-history log ([`crate::history::rename`]) — the multi-buffer REGISTRY
    /// never needs touching here because it only ever holds BACKGROUNDED buffers,
    /// and a rename only ever acts on the ACTIVE one; the recent-files MRU and the
    /// session file are left untouched, mirroring `move_current_file`'s own
    /// established scope (a soft MRU / a machine-state snapshot, not a hard
    /// identity — both self-heal on the next open/quit). REFUSES calmly (a notice,
    /// no write) rather than clobbering: a NAME COLLISION with an existing file, or
    /// a GIT-MANAGED source (git owns naming there — `git mv` is the honest tool).
    /// A blank or UNCHANGED typed name is a quiet no-op (nothing to rename to).
    pub(in crate::app) fn rename_current_file(&mut self, new_name: &str) {
        // Same identity boundary as a move — see `move_current_file`.
        if self.refuse_while_unresolved() {
            return;
        }
        let Some(old) = self.document.buffer().path().map(|p| p.to_path_buf()) else {
            return; // nothing to rename (the prompt shouldn't have opened either)
        };
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return;
        }
        let old_name = old.file_name().map(|s| s.to_string_lossy().to_string());
        if old_name.as_deref() == Some(trimmed) {
            return; // unchanged — nothing to do
        }
        if crate::history::is_git_managed(&old) {
            self.set_sticky_notice("can't rename a file git already tracks");
            return;
        }
        let dest = match old.parent() {
            Some(p) => p.join(trimmed),
            None => PathBuf::from(trimmed),
        };
        if crate::fs::active().exists(&dest) {
            self.set_sticky_notice(format!("already a file named \"{trimmed}\" here"));
            return;
        }
        if let Err(e) = crate::fs::active().rename(&old, &dest) {
            self.set_sticky_notice(format!("rename failed: {e}"));
            return;
        }
        // Best-effort: the history log follows the file; a failed carry-over never
        // disrupts the rename that already succeeded on disk.
        let _ = crate::history::rename(&old, &dest);
        self.document.set_path(dest.clone());
        // Re-key the guard's baseline at the new path (see `move_current_file`).
        self.document
            .adopt_disk_baseline(crate::external::Seen::at(&dest));
        if self.document.buffer().is_unnamed_fresh()
            && let Some(dir) = dest.parent()
        {
            self.document.set_note_dir(dir.to_path_buf());
        }
        self.update_title();
        self.rescan_file_index();
        self.set_toast_notice(format!("renamed to {trimmed}"));
        self.request_frame();
    }

    /// NOTES VERBS round: DUPLICATE the current file — copy the CURRENT buffer
    /// content (including any unsaved edits — a duplicate captures what you're
    /// actually looking at, not necessarily what's on disk) to an auto-named
    /// sibling, then open the copy as the active buffer via the ordinary
    /// [`Self::load_path`] door — which PARKS the original first (so ITS live
    /// edits are never lost) and gives the copy a genuinely FRESH history timeline
    /// (a brand-new `Buffer::from_file`, a brand-new local-history log — nothing
    /// carries over, since the copy is a new file). The sibling name is chosen by
    /// the SAME no-clobber dedup [`crate::buffer::unique_path`] uses elsewhere
    /// (`move_current_file`) — `name-2.md`, `name-3.md`, … — never a
    /// space-separated `"name 2.md"`, matching the codebase's own established
    /// convention. A pathless buffer (scratch / an unnamed fresh document) is a calm no-op —
    /// there is nothing to duplicate yet. Flushes any pending debounced write
    /// FIRST so the ORIGINAL reliably exists on disk under its own name before the
    /// dedup scan runs (otherwise a not-yet-flushed `old` would look "free" to
    /// `unique_path` and the copy could collide with it).
    pub(in crate::app) fn duplicate_current_file(&mut self) {
        let Some(old) = self.document.buffer().path().map(|p| p.to_path_buf()) else {
            return; // scratch: nothing to duplicate
        };
        self.flush_note();
        self.autosave_flush();
        // DUPLICATE IS AN IDENTITY BOUNDARY TOO, and it is gated BEFORE the copy
        // is written rather than after. The flushes above are what raise a
        // conflict, and `load_path` below refuses to leave a conflicted
        // document — so writing first left a real sibling file on disk, no
        // switch, and a "duplicated" toast sitting on top of the refusal's own
        // line, which reads as "it worked" for something that did not.
        if self.refuse_while_unresolved() {
            return;
        }
        let bytes = self.document.buffer().disk_bytes();
        let dir = old.parent().map(Path::to_path_buf).unwrap_or_default();
        let stem = old
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = old
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let new_path = crate::buffer::unique_path(&dir, &stem, &ext);
        match crate::fs::write_atomic(&new_path, &bytes) {
            Ok(()) => {
                self.load_path(new_path);
                self.set_toast_notice("duplicated");
            }
            Err(e) => {
                self.set_sticky_notice(format!("duplicate failed: {e}"));
            }
        }
        self.request_frame();
    }
}
