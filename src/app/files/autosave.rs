//! src/app/files/autosave.rs — the DOCUMENT AUTOSAVE ENGINE (config-gated,
//! default ON: atomic write on idle/blur/switch/quit with a clobber guard,
//! plus the persistent scratch stash), the note's own debounced auto-name
//! save, the save-feedback dirty/title/HUD-saved sync, and the local-history
//! save-hook. Split out of the former `app/files.rs` monolith.

use super::{SCRATCH_CHANGED_NOTICE, WritePermission};
use super::{window_title, window_title_no_document};
use crate::app::*;

impl App {
    /// Commit every key-indexed owner after a naming write changes a provisional
    /// identity into a path. Nothing calls this before the write succeeds.
    pub(in crate::app) fn commit_naming_identity(&mut self, old: crate::buffers::BufferKey) {
        let new = crate::buffers::BufferKey::of(self.document.buffer());
        if old == new {
            return;
        }
        self.document.rekey_active_after_naming();
        self.persistence.retire_note_ledger(&old);
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
        if let Some(waiters) = self.wait_conns.remove(&old) {
            self.wait_conns.entry(new).or_default().extend(waiters);
        }
    }

    /// Set the window title from the active file + theme (kept in one place so
    /// open/switch/theme-cycle all agree). Wraps the pure [`window_title`] — the
    /// ONE owner of the actual string, also used by the initial window
    /// construction in `resumed()` (before a `gpu`/window exists to `set_title`
    /// on), so a fresh launch's very first title and every later update agree.
    /// SAVE-FEEDBACK round: "is the active buffer UNSAVED", by the SAME
    /// version-vs-saved-version bookkeeping the autosave engine already
    /// tracks (`sync_view`'s own arm-check, mirrored here as a read) — NOT
    /// the raw `Buffer::is_dirty()` edit-tracked bit, which autosave (a
    /// direct `fs::write_atomic`, never routed through `Buffer::save`)
    /// deliberately never clears. Using the raw bit would leave the title's
    /// edited marker (and the native titlebar dot) stuck on indefinitely on
    /// an actively-autosaved document, even though its content is already
    /// safely on disk — misleading, and the opposite of what autosave is
    /// FOR. This reads true "unsaved" — cleared the instant ANY successful
    /// write lands, manual save or autosave alike — matching every
    /// conventional editor's own dirty-dot behavior.
    pub(in crate::app) fn is_document_dirty(&self) -> bool {
        let Some(buffer) = self.document.buffer_opt() else {
            return false;
        };
        if buffer.is_unnamed_fresh() {
            self.persistence.note_write_owed(
                &self.document.active_key().expect("active document key"),
                buffer.version(),
            )
        } else {
            // THE ONE UNSAVED RULE (`DocumentSession::entry_unsaved`), which the
            // removal owner asks of parked entries too. Spelled inline here, it
            // was an active-buffer-only question by construction — and a close
            // that judged "is there unsaved text" by a second copy of this
            // arithmetic would eventually discard a buffer this one calls dirty.
            self.document.active_unsaved()
        }
    }

    /// NOTES VERBS round: push the held HUD's SAVED stat state into the pipeline —
    /// `Dirty` while the buffer has unsaved changes RIGHT NOW (`is_document_dirty`,
    /// the SAME check the window title's dirty-dot uses), else `Saved(secs)` from
    /// `persistence.last_save_at()` (the last successful write of ANY kind — manual save, the
    /// scratch→note conversion, a note's own autosave, or the document autosave
    /// engine), else `None` when nothing has ever saved yet this session (renders
    /// the fixed placeholder). Called every `sync_view`, mirroring `stats_sync_hud`
    /// exactly — LIVE-ONLY (a real clock read), so a headless capture never calls
    /// this and the pipeline field stays `None`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn sync_hud_saved(&mut self) {
        let state = if self.is_document_dirty() {
            Some(crate::hud::HudSaved::Dirty)
        } else {
            self.persistence
                .last_save_at()
                .map(|t| crate::hud::HudSaved::Saved(self.frame.now().duration_since(t).as_secs()))
        };
        let Some(gpu) = self.frame.gpu_mut() else {
            return;
        };
        gpu.pipeline.set_hud_saved(state);
    }

    /// CHECK FOR UPDATES round: push the About card's "checked … ago" figure —
    /// reads the LOCAL "last checked" marker (`updates::update_checked_state`,
    /// `Never` if no marker exists yet, `CheckedAgo(secs)` otherwise) against a
    /// real clock. Called every `sync_view`, mirroring `sync_hud_saved` exactly
    /// — LIVE-ONLY (a real clock + fs read), so a headless capture never calls
    /// this and the pipeline field stays `None` (the About card's determinism
    /// boundary — `updates::checked_line(None)` renders the fixed placeholder).
    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn sync_update_checked(&mut self) {
        let dir = crate::fs::data_root();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let state = crate::updates::update_checked_state(&dir, now);
        let Some(gpu) = self.frame.gpu_mut() else {
            return;
        };
        gpu.pipeline.set_update_checked(Some(state));
        gpu.pipeline.set_pending_crash(self.pending_crash.is_some());
    }

    pub(in crate::app) fn update_title(&mut self) {
        // SAVE-FEEDBACK round: keep the title dirty-cache (which `sync_view`
        // compares against for its "only re-title on a real flip" gate — see
        // its own doc) in step with whatever this call actually renders, no
        // matter which caller reached here.
        let dirty = self.is_document_dirty();
        self.persistence.record_title(dirty);
        if let Some(gpu) = self.frame.gpu() {
            let title = self.document.buffer_opt().map_or_else(
                || window_title_no_document(crate::theme::active().name),
                |buffer| {
                    window_title(
                        buffer.path(),
                        buffer.is_unnamed_fresh(),
                        crate::theme::active().name,
                        dirty,
                    )
                },
            );
            gpu.window.set_title(&title);
            // NATIVE macOS TITLEBAR DIRTY-DOT: winit exposes this directly
            // (`WindowExtMacOS::set_document_edited` — the grey dot in the
            // titlebar's close button, the same convention every native Mac
            // document app uses), so no bespoke `mac_chrome.rs` plumbing is
            // needed for this one. LIVE-ONLY (needs human confirmation) — the
            // headless capture never constructs a `gpu`/window, so this is
            // unreachable from `--screenshot`/`--keys` and adds no sidecar
            // field, mirroring `cursor_shape`'s `set_cursor` precedent.
            #[cfg(target_os = "macos")]
            {
                use winit::platform::macos::WindowExtMacOS;
                gpu.window.set_document_edited(dirty);
            }
        }
    }

    /// ROBUST-AUTOSAVE flush: write a pending note save IMMEDIATELY, bypassing the
    /// debounce, so nothing typed in the last quiet window is lost when we switch
    /// away from / close the note. Called before opening another file (`load_path`),
    /// switching project / starting a new note (`set_root`), on focus-out, and on
    /// quit. A truly empty note still writes nothing (no litter); a non-note buffer
    /// or an already-saved version is a no-op.
    pub(in crate::app) fn flush_note(&mut self) {
        if !self.document.has_active() {
            return;
        }
        if self.document.buffer().is_unnamed_fresh()
            && self.persistence.note_write_owed(
                &self.document.active_key().expect("active document key"),
                self.document.buffer().version(),
            )
        {
            self.persistence
                .disarm_note_debounce(&self.document.active_key().expect("active document key"));
            self.autosave_note();
        }
    }

    /// Auto-save the active UNNAMED FRESH DOCUMENT (live only, debounced). The
    /// buffer derives its filename from the first non-empty line on this — its
    /// FIRST — save (an empty document writes nothing — no litter); the moment
    /// that happens, `Buffer::save` binds the path AND clears the fresh-document
    /// marker in the SAME step (the one-shot naming law), so
    /// `is_unnamed_fresh()` is false by the time this function returns and every
    /// LATER save (including a later edit to the first line) simply routes
    /// through the ordinary document autosave engine instead — never a second
    /// call here, never a live rename.
    ///
    /// **Hand-off bookkeeping:** since the buffer transitions from "fresh" to
    /// "ordinary pathed document" in THIS call, also stamp the ordinary
    /// document autosave engine's own baselines (`doc_saved_version`,
    /// `disk_mtime`, `caret_synced_version`) here — mirroring
    /// `App::finish_manual_save`/`convert_scratch_and_save`'s post-save
    /// bookkeeping — so `is_document_dirty`'s `path().is_some()` branch (which
    /// this buffer reads from the very next `sync_view`, `is_unnamed_fresh()`
    /// now being false) sees a freshly-saved, clean baseline rather than a
    /// stale/absent one that would misreport dirty.
    pub(in crate::app) fn autosave_note(&mut self) {
        if !self.document.has_active() {
            return;
        }
        let naming_key = self.document.active_key().expect("active document key");
        if !self.document.buffer().is_unnamed_fresh() {
            return;
        }
        self.persistence
            .record_note_write(naming_key.clone(), self.document.buffer().version());
        match self.document.save_owned(crate::durable::Owner::Autosave) {
            Ok(()) => {
                self.persistence.clear_note_failure(&naming_key);
                self.commit_naming_identity(naming_key.clone());
                // `Buffer::save` only returns `Ok` here having derived + bound a
                // path (an empty document, the ONLY other `Ok`-less case, bails
                // into the `Err` arm below instead) — so `path()` is always
                // `Some` on this arm.
                let p = self.document.buffer().path().map(|p| p.to_path_buf());
                let version = self.document.buffer().version();
                let seen = p
                    .as_deref()
                    .map(crate::external::Seen::at)
                    .unwrap_or_default();
                self.document.record_document_saved(version, seen);
                // SAVE-FEEDBACK round: no terminal echo — a background
                // autosave naming a fresh document is silent chatter (the
                // window title already renders the new name). `Buffer::save`
                // already stamped the derived path onto the buffer itself
                // (the sole authoritative path).
                self.update_title();
                // Re-scope the go-to index so the new document is jump-able.
                self.rescan_file_index();
                // AUTOMATIC LOCAL SNAPSHOT: a loose document just hit the disk, so
                // capture a history point (git-managed files + history-off are
                // skipped inside).
                self.snapshot_after_save();
                // NOTES VERBS round: the held HUD's SAVED stat.
                let now = self.frame.now();
                self.persistence.record_save(now);
            }
            Err(error) => {
                if self.persistence.first_note_failure(naming_key) {
                    self.set_sticky_notice(format!(
                        "autosave failed: {error} — changes remain in editor"
                    ));
                    self.request_frame();
                }
            }
        }
    }

    /// PASTE-IMAGE'S NO-PATH PRE-SAVE (`App::paste_image_reference`, `app/apply.rs`): a
    /// path-less buffer — the bare scratch surface, or an unnamed fresh
    /// document — has no directory to hang an `assets/` folder off of. Give it
    /// one FIRST by reusing the EXISTING fresh-document auto-name save
    /// (`Self::autosave_note` → `Buffer::save`'s first-line-derived filename),
    /// rather than inventing a parallel naming rule. A plain scratch buffer
    /// (never summoned via Cmd-N — `note_dir` unset) is first PROMOTED into an
    /// unnamed fresh document rooted at `self.root` (the ACTIVE folder — the
    /// same home Cmd-N uses), via `Buffer::set_note_dir` (content-preserving —
    /// unlike `start_fresh_doc`, nothing is reset) — so it now follows the
    /// one-shot naming model exactly as if Cmd-N had started it. An
    /// already-in-progress fresh document (`note_dir` already set) is left
    /// pointed at its own dir. An EMPTY buffer has no first line to derive a name
    /// from yet — `autosave_note` (via `Buffer::save`) errs quietly and the
    /// buffer stays path-less; the caller (`paste_image_reference`) falls back to its
    /// pre-existing absolute data-root location rather than blocking the paste.
    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn ensure_note_named_before_paste(&mut self) {
        if !self.document.has_active() {
            return;
        }
        if !self.document.buffer().is_unnamed_fresh() {
            let _ = crate::fs::active().create_dir_all(&self.project_location.root);
            self.document
                .set_note_dir(self.project_location.root.clone());
        }
        self.autosave_note();
    }

    /// SAVE-HOOK for AUTOMATIC LOCAL HISTORY: after a successful save (manual OR
    /// autosave — every save records), record a snapshot of the current buffer to
    /// the local history store (see [`crate::history::record`]). The store itself
    /// decides whether to keep it — a GIT-MANAGED file (git owns its versioning,
    /// unconditionally) or `history = false` writes nothing; a loose note/draft
    /// (or any file on the web) is snapshotted, keyed by its path + a timestamp,
    /// and pruned by the aged retention ladder. A no-op for a scratch buffer that
    /// has no bound path yet (the scratch stash records under its own stash
    /// path). Best-effort: any store error is swallowed inside `record`, so a
    /// failed history write never disrupts the save.
    ///
    /// CONSCIOUS MARK (banked, not built): a deliberate pin-this-version-before-
    /// major-surgery flag would be minted here and carried into the store,
    /// exempt from the ladder. See `history::prune_ladder`.
    pub(in crate::app) fn snapshot_after_save(&self) {
        if let Some(buffer) = self.document.buffer_opt()
            && let Some(path) = buffer.path()
        {
            crate::history::record(path, &buffer.text(), &self.config);
        }
    }

    /// The AUTOSAVE ENGINE's flush — the one door every trigger goes through
    /// (idle, window blur, file switch, quit). Config-gated (`autosave`, default
    /// ON). Routes by buffer kind: a NOTE keeps its own 400ms flow (untouched); a
    /// pathed document writes atomically via [`Self::autosave_doc_now`]; a true
    /// scratch (no path, not a note) stashes via [`Self::stash_scratch_now`].
    /// Lives only on the live `App`, so the headless capture is structurally
    /// autosave-free (determinism law).
    pub(in crate::app) fn autosave_flush(&mut self) {
        self.document.disarm_doc_autosave();
        if !self.config.autosave_on() || !self.document.has_active() {
            return;
        }
        if self.document.buffer().is_unnamed_fresh() {
            return; // notes have their own debounced autosave (flush_note)
        }
        if self.document.buffer().path().is_some() {
            self.autosave_doc_now();
        } else {
            self.stash_scratch_now();
        }
    }

    /// Quietly SAVE the open document NOW (the autosave engine's pathed-buffer
    /// arm): skip when the buffer version is already on disk; route through the
    /// one external-change guard ([`Self::settle_external_change`]), which may
    /// reload a clean buffer or latch a conflict instead of letting the write
    /// through; otherwise write atomically, take a fresh baseline from the bytes
    /// just written, clear the notice, and record a history snapshot (the
    /// store's git gate + dedup + ladder decide what's kept). Errors go to
    /// stderr, never disrupt.
    ///
    /// While a conflict is latched the engine keeps running — it just writes to
    /// the RECOVERY RECORD instead of to the user's file, which is what makes
    /// carrying on editing an unresolved document safe.
    fn autosave_doc_now(&mut self) {
        let Some(path) = self.document.buffer().path().map(|p| p.to_path_buf()) else {
            return;
        };
        let version = self.document.buffer().version();
        if self.document.doc_saved_version() == Some(version) {
            return; // nothing new to write
        }
        match self.settle_external_change() {
            WritePermission::Clear => {}
            // The buffer was clean and has already been replaced by the disk's
            // own text; there is nothing of the user's left to write.
            WritePermission::Reloaded => return,
            WritePermission::Held => {
                self.write_recovery_record(&path);
                // Mark the version handled so the idle timer doesn't spin on the
                // same content; the next edit re-arms (and the record refreshes).
                self.document.acknowledge_document_version(version);
                return;
            }
        }
        // Restore the buffer's remembered line ending on the way out (CRLF files
        // round-trip byte-for-byte; LF is byte-identical to `text().as_bytes()`).
        let bytes = self.document.buffer().disk_bytes();
        match crate::durable::write(crate::durable::Owner::Autosave, &path, &bytes) {
            Ok(()) => {
                self.document.record_document_saved(
                    version,
                    crate::external::Seen::after_write(&path, &bytes),
                );
                if self.clobber_notice_active() {
                    self.clear_notice();
                }
                // DEBUG PANEL + the held HUD's SAVED stat, in ONE transition:
                // the engine's own "last wrote successfully" clock and the
                // any-kind save clock moved in lockstep at both engine sites,
                // so `record_engine_write` is now the only spelling.
                let now = self.frame.now();
                self.persistence.record_engine_write(now);
                // Every save records a snapshot (dedup + the git gate live inside).
                self.snapshot_after_save();
            }
            Err(e) => {
                self.set_sticky_notice(format!("autosave held: {e} — changes remain in editor"));
                eprintln!("autosave failed ({}): {e}", path.display());
            }
        }
    }

    /// STASH the persistent SCRATCH buffer NOW (the autosave engine's no-path
    /// arm): write the whole text — EVEN empty, so an emptied scratch clears a
    /// stale stash — atomically to [`crate::fs::scratch_stash_path`], guarded by
    /// the same clobber truth-table (two awl instances sharing one stash), then
    /// grow the stash's own ladder timeline via [`crate::history::record`]. The
    /// restore half lives in `App::new` (a no-argument launch).
    ///
    /// Returns whether the stash now genuinely holds this buffer's content —
    /// `false` on a held clobber (another window's copy wins; nothing was
    /// written) or a write error. The idle/blur/switch/quit engine ignores
    /// this; the ACTIVE-scratch close arm (`App::try_save_finished_buffer`)
    /// does not — a close about to discard the in-memory buffer must know the
    /// stash is not simply an accepted best-effort loss.
    pub(in crate::app) fn stash_scratch_now(&mut self) -> bool {
        let version = self.document.buffer().version();
        if self.document.scratch_saved_version() == Some(version) {
            return true; // stash already holds this content
        }
        let path = crate::fs::scratch_stash_path();
        let (change, seen) = crate::external::look(&path, &self.document.scratch_baseline());
        if change.is_change() {
            self.set_sticky_notice(SCRATCH_CHANGED_NOTICE);
            self.document
                .record_scratch_saved(version, self.document.scratch_baseline());
            return false;
        }
        let _ = seen;
        let text = self.document.buffer().text();
        let fs = crate::fs::active();
        if let Some(parent) = path.parent() {
            let _ = fs.create_dir_all(parent);
        }
        // A true scratch buffer is always Lf, but route the write through the ONE
        // encoder for uniformity; the history snapshot stays the internal pure-`\n`
        // `text` (awl's own store — see the "Line endings" note in CLAUDE.md).
        let bytes = self.document.buffer().disk_bytes();
        match crate::durable::write(crate::durable::Owner::Scratch, &path, &bytes) {
            Ok(()) => {
                self.document.record_scratch_saved(
                    version,
                    crate::external::Seen::after_write(&path, &bytes),
                );
                if self.clobber_notice_active() {
                    self.clear_notice();
                }
                // DEBUG PANEL + the held HUD's SAVED stat, in ONE transition:
                // the engine's own "last wrote successfully" clock and the
                // any-kind save clock moved in lockstep at both engine sites,
                // so `record_engine_write` is now the only spelling.
                let now = self.frame.now();
                self.persistence.record_engine_write(now);
                // The persistent scratch grows a timeline of its own.
                crate::history::record(&path, &text, &self.config);
                true
            }
            Err(e) => {
                self.set_sticky_notice(format!(
                    "scratch save held: {e} — changes remain in editor"
                ));
                eprintln!("scratch stash failed ({}): {e}", path.display());
                false
            }
        }
    }
}
