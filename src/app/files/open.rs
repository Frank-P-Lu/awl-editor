use crate::app::*;
impl App {
    /// Apply the result of a platform file chooser. Kept separate from the
    /// modal panel so Cancel and accepted-path behavior are testable without an
    /// OS surface. Returns whether a choice was committed.
    #[cfg(any(target_os = "macos", all(test, not(target_arch = "wasm32"))))]
    pub(in crate::app) fn apply_file_choice(&mut self, chosen: Option<PathBuf>) -> bool {
        let Some(path) = chosen else {
            return false;
        };
        self.load_path(path);
        true
    }

    /// Apply the result of a platform folder chooser through the same rescope,
    /// session, and recent-folder owner as a typed Go-to folder row.
    #[cfg(any(target_os = "macos", all(test, not(target_arch = "wasm32"))))]
    pub(in crate::app) fn apply_folder_choice(&mut self, chosen: Option<PathBuf>) -> bool {
        let Some(path) = chosen else {
            return false;
        };
        self.switch_project(path);
        true
    }

    pub(in crate::app) fn open_settings(&mut self) {
        let path = self.config.path.clone();
        if path.as_os_str().is_empty() {
            return; // no resolvable config path (no HOME); nothing to open
        }
        if !crate::fs::active().exists(&path)
            && let Err(e) = Config::write_default(&path)
        {
            eprintln!("could not create config {}: {e}", path.display());
            return;
        }
        self.load_path(path);
    }

    /// SWITCH the active folder to `new_root` — the ONE owner of a genuine
    /// switch-project (both the `Project` picker's accepted folder AND the
    /// Recent Projects picker route here). Re-scopes the root ([`Self::set_root`]),
    /// EAGERLY remembers it as the one active-folder-context (`session_flush`,
    /// native only — see `app/session.rs`'s module doc; the old
    /// separate "sticky project root" config key into the ONE session-owned
    /// context, so a crash/relaunch right after a switch still resumes here,
    /// not the pre-switch folder), AND pushes it to the front of the persisted
    /// RECENT list ([`Self::push_recent_project`]).
    pub(in crate::app) fn switch_project(&mut self, new_root: PathBuf) {
        // A cancelled MAS grant panel (see `set_root`'s doc) means the switch
        // never happened — never persist/MRU a root we didn't actually move
        // into.
        if !self.set_root(new_root) {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.session_flush();
        // Read the CANONICAL root `set_root` just stored, not the raw
        // argument this function was called with — an alias spelling of an
        // already-open project (a firmlink/symlink/case variant) must MOVE
        // its existing MRU entry to the front, never grow a second one
        // (`crate::recents::push`'s own dedupe compares by exact `PathBuf`
        // equality, so it only collapses aliases if both sides already agree
        // on one spelling).
        self.push_recent_project(self.project_location.root.clone());
    }

    /// Push `root` to the FRONT of the persisted RECENT PROJECT ROOTS (deduped +
    /// capped, [`crate::recents::push`]) and save the list ATOMICALLY. A save
    /// error is reported and swallowed (a lost MRU entry is never worth crashing
    /// a project switch). Native only, and App-only — but `--screenshot-app`
    /// drives a real App through the real effect interpreter, so a replayed Goto
    /// DOES reach here; what keeps the user's own list out of it is the seeded
    /// sandbox that door installs (`scenario::install_hermetic_fs`).
    pub(in crate::app) fn push_recent_project(&mut self, root: PathBuf) {
        let list = std::mem::take(&mut self.project_location.recent_projects);
        self.project_location.recent_projects =
            crate::recents::push(list, root, crate::recents::CAP);
        if let Err(e) = crate::recents::save(
            &crate::recents::recents_path(),
            &self.project_location.recent_projects,
        ) {
            eprintln!("recent-projects save failed: {e}");
        }
    }

    /// Push `file` to the FRONT of the persisted RECENTLY-OPENED FILES MRU (deduped
    /// + capped, [`crate::recent_files::push`]) and save it ATOMICALLY. A save error
    ///   is reported + swallowed (a lost MRU entry is never worth crashing a file
    ///   open). Same door split as [`Self::push_recent_project`], whose doc states
    ///   it: `--screenshot-app` reaches here too and writes into its hermetic
    ///   sandbox. The FILE sibling of that owner.
    pub(in crate::app) fn push_recent_file(&mut self, file: PathBuf) {
        // CANONICALIZED, mirroring `switch_project`'s own comment: `file` is
        // whatever spelling this open arrived under (a raw CLI argument, a
        // native file-chooser result), and `crate::recents::push`'s dedupe
        // compares by exact `PathBuf` equality — two spellings of one real
        // file (a symlink/firmlink alias) would otherwise both stay in the
        // MRU as separate entries, breaking its own documented "never
        // duplicates" contract.
        let file = crate::buffers::normalize_path(&file);
        let list = std::mem::take(&mut self.project_location.recent_files);
        self.project_location.recent_files = crate::recent_files::push(list, file);
        if let Err(e) = crate::recent_files::save(&self.project_location.recent_files) {
            eprintln!("recent-files save failed: {e}");
        }
    }

    /// Open a project-relative path: swap in a fresh Buffer, reset cursor/undo,
    /// keep `App.file` + window title in sync. The product model is open/switch
    /// only — no file ops — so we just re-read from disk. `rel` is a root-relative
    /// index entry. The recently-opened-files MRU is pushed inside [`Self::load_path`]
    /// (the ONE door every real-file open routes through), so this stays a thin
    /// resolve-and-load.
    pub(in crate::app) fn open_rel(&mut self, rel: &str) {
        let path = crate::index::resolve(&self.project_location.root, rel);
        self.load_path(path);
    }

    /// SEARCH-IN-FOLDER's own door: open `rel` (root-relative, `open_rel`'s
    /// own resolve) AND land the caret at `line`/`col` -- but ONLY if the open
    /// actually succeeds (unlike `open_rel`, which discards `load_path`'s
    /// bool; a refused open here -- classified unsupported, deleted since the
    /// search ran -- must never jump the caret inside whatever buffer was
    /// already active).
    pub(in crate::app) fn open_path_at_line(&mut self, rel: &str, line: usize, col: usize) {
        let path = crate::index::resolve(&self.project_location.root, rel);
        if self.load_path(path) {
            self.jump_to_line_col(line, col);
        }
    }

    pub(in crate::app) fn last_buffer_toggle(&mut self) {
        let Some(prev) = self.document.previous_key() else {
            return; // nothing opened before; toggle is a quiet no-op
        };
        self.activate_open_buffer(prev);
    }

    /// Swap in the buffer for `path`: remember the file we are LEAVING as
    /// `prev_file` (the 2-deep last-buffer history), then either SWITCH to its
    /// already-open live buffer (unsaved edits + cursor + scroll + undo + spell
    /// state all survive — the multi-buffer registry win) or read it fresh from
    /// disk for a first-time open. Shared by `open_rel` and the C-x b toggle so
    /// both keep the history honest.
    pub(in crate::app) fn load_path(&mut self, path: PathBuf) -> bool {
        // THE ONE CAPABILITY OWNER, first: a binary/unsupported
        // `path` is refused HERE, before the MAS grant probe below (never
        // powerbox a file we're about to refuse) or any other side effect —
        // the active buffer, `self.root`, and every remembered-context field
        // stay exactly as they were. This is the door EVERY picker selection
        // (open_rel), the C-x b toggle, AND the daemon's `open` handoff
        // (`App::handle_daemon_event`, which calls this same fn) all share —
        // see `crate::openable`'s module doc for the full door list.
        if let Some(msg) = crate::openable::classify(&path).refusal_message() {
            self.set_sticky_notice(msg);
            return false;
        }
        // MAS SANDBOX GRANT GATE (native macOS `mas` builds only — see
        // `src/mas.rs`'s module doc): `path` may live outside the container.
        // `ensure_access` is a no-op fast-path for anything inside it or
        // inside an already-granted root; otherwise it powerboxes the user via
        // the system folder panel BEFORE any read below is attempted. A
        // cancelled panel aborts the open outright — never let a doomed read
        // fail against a silent sandbox `EPERM` instead.
        #[cfg(all(feature = "mas", target_os = "macos"))]
        if !crate::mas::ensure_access(&path) {
            return false;
        }
        // LEAVING AN UNRESOLVED DOCUMENT IS NOT ALLOWED. The conflicted buffer
        // is the sole editable copy of one of the two versions; parking it
        // behind another document would leave the user's text reachable only
        // through the recovery record, and the conflict itself invisible. The
        // refusal names the two resolutions.
        if self.refuse_while_unresolved() {
            return false;
        }
        self.flush_note();
        self.autosave_flush();
        // …AND AGAIN, because the flush above is itself a write door: it may
        // have discovered the change and latched the conflict a moment ago,
        // after the gate at the top of this function had already passed. Missing
        // this leaves the conflicted buffer parked behind another document with
        // its notice showing and no way back to it — found by an existing
        // reopen law, not by this item's own tests.
        if self.refuse_while_unresolved() {
            return false;
        }
        // WRITING STREAKS: sample the LEAVING buffer's word-delta BEFORE it is
        // replaced below, so words written in it this session are recorded against
        // the right document; the anchor is reset after the swap so the arriving
        // buffer's existing words are never miscounted (native only; gated inside).
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_flush_if_document();
        // If the flush we just ran raised the clobber-guard notice (the file we
        // are LEAVING changed on disk outside awl, so its unsaved edit could
        // not be safely autosaved), that notice must survive the switch below
        // — otherwise the unconditional clear a few lines down would wipe it
        // in the very same call it was set, so the user never sees it at all
        // (code review nit: a real, if minor, live bug — the warning fires
        // and vanishes before a single frame renders it).
        let clobber_notice_just_raised = self.clobber_notice_active();
        // Already the active file: a no-op reopen preserves everything for free
        // (and avoids parking a buffer under its own key). Compared via the
        // SAME normalized identity the registry uses (`BufferKey::path`), not
        // raw path equality — a relative launch argument and its later
        // root-joined spelling (see `BufferKey::path`'s doc) must both be
        // recognized as "already here", or this falls through into an
        // unnecessary (if harmless, post-fix) park/take round trip.
        let opened = self.document.open_path(
            &path,
            crate::external::Seen::at(&path),
            &self.project_location.root,
        );
        if opened == document::OpenPath::AlreadyActive {
            return true;
        }
        self.finish_buffer_activation(Some(path), clobber_notice_just_raised);
        true
    }

    /// Finish activation of the whole slot already installed by the document
    /// owner. Fresh opens, explicit working-set switches, and the successor of
    /// a close all enter here, so arrival never depends on reloading the file.
    pub(in crate::app) fn finish_buffer_activation(
        &mut self,
        path: Option<PathBuf>,
        preserve_notice: bool,
    ) {
        // THE ARRIVING DOCUMENT BRINGS ITS PROJECT WITH IT. A buffer opened
        // under one root can be activated while another is current — Last file
        // across a Switch-project is the everyday route — and until this, the
        // root simply stayed where it was. The document and the bottom identity
        // then described two different directories, with Go to's corpus, New
        // document and every export destination scoped to the one the reader
        // was NOT looking at.
        //
        // Restored HERE rather than by the caller, and BEFORE `sync_view` below,
        // so no frame is ever composed from the disagreement. `resync_project_location`
        // is the one legal derivation door for everything the root implies.
        if let Some(root) = self.document.working_set().active_root()
            && root != self.project_location.root
        {
            self.project_location.root = root.to_path_buf();
            self.resync_project_location(self.config.location_policy());
        }
        // ALREADY OPEN elsewhere in this session: switch to its LIVE buffer
        // instead of re-reading disk — unsaved edits, cursor, scroll, undo,
        // and spell-cache state all survive the round trip (a whole-slot
        // activation — see `activate_from_registry`).
        // OPENING A DOCUMENT NEVER EDITS IT. There is deliberately no
        // language-tag stamp on the fresh-open branch: the render ladder
        // re-reads the frontmatter tag from the buffer on every reshape
        // (`TextPipeline::doc_lang`), so nothing has to be written into the
        // user's file for resolution to work, and an untagged document's Han
        // runs stay governed by the config `cjk_priority` tiebreak the user
        // set — which a stamped tag would outrank forever after. The one door
        // that writes the tag is EXPLICIT: `actions::edit::tag_document_language`
        // ("Tag document language" in the palette). Law:
        // `opening_an_untagged_cjk_document_never_mutates_the_buffer`.
        if !preserve_notice {
            self.clear_notice();
        }
        // THE ARRIVING BUFFER is a persistence boundary of its own, in two
        // ways. A record left by a previous run (or by a conflict raised on
        // this file before it was parked) is adopted here, so the unresolved
        // state is found by opening the file it belongs to rather than only on
        // the launch that happens to reopen it. And a REACTIVATED buffer — one
        // parked in the registry, carrying the baseline it had when it was
        // parked — is re-checked, because the disk has had the whole
        // intervening session to move.
        if let Some(path) = path.as_deref() {
            self.adopt_unresolved_for(path);
            if !self.change_unresolved() {
                self.settle_external_change();
            }
        }
        // `Buffer::path()` already carries this exact path — on the fresh-open
        // arm from `Buffer::from_file(&path)`, on the registry-hit arm from
        // the entry's own remembered path (it was parked under a key derived
        // from that same path) — so there is no separate `App.file` mirror to
        // write here any more (`Buffer::path()` is the sole source).
        // RECENTLY-OPENED FILES MRU: this file was just OPENED (either fresh from
        // disk or switched-to from the buffer registry — BOTH arrive here), so push
        // it to the front of the persisted MRU that feeds the go-to Recent lens +
        // recency tier. After the already-active early-return above, so re-selecting
        // the current file is a no-op that never re-orders the MRU.
        if let Some(path) = path {
            self.push_recent_file(path.clone());
            #[cfg(not(target_arch = "wasm32"))]
            self.stats_touch_file(path);
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_reset_caret_anchor();
        // WRITING STREAKS: the buffer just swapped — drop the word-delta anchor so
        // the arriving document's existing words re-anchor (never counted as
        // freshly written) on its first flush.
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_reset_baseline();
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_flush();
        self.workspace_state.close_search();
        self.input.clear_preedit();
        // The HISTORY TIMELINE preview cache is now buffer-scoped
        // (`BufferExtra::history_preview`): the ARRIVING buffer's own
        // slot already carries its own value (`None` on a fresh open, or
        // whatever it held when it was parked) — no manual clear needed here
        // any more (the whole-slot move made the old hand-written
        // `self.history_preview = None` structurally redundant, and actively
        // WRONG once folded in, since it would stomp the buffer we just
        // activated instead of the one we left).
        // STICKY PAGE WIDTH: re-apply the measure for the ARRIVING buffer's own
        // kind (prose vs code — see `Config::measure_for`) BEFORE `sync_view`, so
        // its cursor-follow scroll math reads freshly re-wrapped row geometry
        // rather than whatever the LEAVING buffer's kind left behind.
        self.sync_page_measure();
        self.update_title();
        self.sync_view(true);
        self.request_frame();
    }

    /// Activate one existing working-set entry by identity. Pathed entries
    /// continue through `load_path`, the single file-open door. A scratch entry
    /// has no path to feed that door, so it moves the complete parked slot here
    /// while preserving the same leave/arrive boundaries.
    pub(in crate::app) fn activate_open_buffer(&mut self, key: crate::buffers::BufferKey) {
        if let Some(path) = self
            .document
            .working_set()
            .path_for(&key)
            .map(std::path::Path::to_path_buf)
        {
            self.load_path(path);
            return;
        }
        if self.refuse_while_unresolved() {
            return;
        }
        self.flush_note();
        self.autosave_flush();
        if self.refuse_while_unresolved() {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_flush();
        if !self.document.activate_key(&key) {
            // A legacy/corrupt scratch row with no parked entry used to be a
            // dead end. Retire the ghost row and recover from the persistent
            // stash through the ordinary scratch-open door. Fresh identities
            // never share this fallback: each owns distinct user text.
            if key == crate::buffers::BufferKey::Scratch {
                self.document.discard(&key);
                self.open_scratch();
            }
            return;
        }
        self.finish_buffer_activation(None, false);
    }

    /// Summon the persistent scratch buffer as the active document — the ONE
    /// in-session door back after `Action::FinishBuffer` / a stack-row close
    /// discards it (`app/files/close.rs`'s decided fix: closing scratch just
    /// closes it, because the autosave engine's stash already holds the text
    /// — but a session that never relaunches needs a way back to that stash
    /// besides restarting).
    ///
    /// Scratch is a SINGLETON identity (`BufferKey::Scratch`): if it is still
    /// open anywhere this session — active, or merely parked behind another
    /// document — this is the ordinary reactivation door
    /// ([`Self::activate_open_buffer`]), never a fresh read that would
    /// silently discard whatever that live entry holds. Only when scratch
    /// exists NOWHERE in the working set does this read its stash back, via
    /// the exact door `App::new`'s own launch-time restore uses.
    pub(in crate::app) fn open_scratch(&mut self) {
        let key = crate::buffers::BufferKey::Scratch;
        if self.document.close_facts(&key).is_some() {
            self.activate_open_buffer(key);
            return;
        }
        if self.refuse_while_unresolved() {
            return;
        }
        self.flush_note();
        self.autosave_flush();
        if self.refuse_while_unresolved() {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_flush_if_document();
        let (buffer, baseline) = crate::app::startup::scratch_buffer_from_stash();
        self.document
            .open_scratch(buffer, baseline, self.project_location.root.clone());
        self.workspace_state.close_search();
        self.input.clear_preedit();
        self.sync_page_measure();
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_reset_caret_anchor();
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_reset_baseline();
        self.clear_notice();
        self.update_title();
        self.sync_view(true);
        self.request_frame();
    }

    pub(in crate::app) fn jump_to_line(&mut self, line: usize) {
        self.jump_to_line_col(line, 0);
    }

    /// `jump_to_line`'s own column-aware generalization -- search-in-folder's
    /// door lands the caret exactly on the match, not just the line start.
    pub(in crate::app) fn jump_to_line_col(&mut self, line: usize, col: usize) {
        let idx = self.document.buffer().line_col_to_char(line, col);
        self.document.clear_mark();
        self.document.set_cursor(idx);
        // REVEALED PLACEMENT (folds): a heading Go-to / margin-outline jump may target
        // a line hidden inside a collapsed section — route through the ONE placement
        // owner so the landing line is revealed, never left inside a fold. A cheap
        // no-op unless a section is folded.
        self.document.reveal_placement();
        self.document.set_shift_selecting(false);
        self.sync_view(true);
        self.request_frame();
    }

    /// Re-scan `self.root`'s file index through the `FileSystem` trait (git
    /// `ls-files` union `.env*`, or a recursive walk — see `index::build_index`)
    /// and replace the cached `file_index` with the fresh result. The ONE owner
    /// of "make the go-to corpus current": every trigger that can make the old
    /// index stale (a root switch, a note's first save, a rename, a move) calls
    /// this rather than re-deriving the same line; the Goto summon itself
    /// (`C-x f`, `app/apply.rs`) also calls it — RE-SCAN ON EVERY SUMMON (queue:
    /// "file picker freshness"), so a file created on disk after the app
    /// launched or last scanned is never missing. No cache TTL, no watcher: a
    /// summoned overlay is transient and the walk is disk-cheap for a real
    /// project tree (measured on this repo: see `index::tests::build_index_on_this_repo_is_fast`).
    ///
    /// Narrower than [`Self::resync_project_location`] on purpose: a rename/
    /// move/first-save re-scans the SAME root's index without touching
    /// `project` or `workspace_root`, which haven't changed.
    pub(in crate::app) fn rescan_file_index(&mut self) {
        self.project_location.rescan_file_index();
    }

    /// THE ONE OWNER of "what does `self.root` imply" (docs/app-domains.md's
    /// `ProjectLocation` section): `project`, `file_index`, and `workspace_root`
    /// are all pure functions of `(self.root, self.cli_workspace,
    /// self.config.workspace)`, and this is the only place that derives any of
    /// the three — every entry point that changes one of those inputs calls
    /// this rather than re-deriving a subset by hand.
    ///
    /// Before this fn existed, `set_root` re-derived `project` + `file_index`
    /// (via [`Self::rescan_file_index`]) but not `workspace_root`, while
    /// `reload_config` re-derived `workspace_root` but not the other two — two
    /// half-owners of one derived value, agreeing only by accident. Because
    /// [`crate::resolve_workspace`] falls back to `root.parent()` when neither
    /// the CLI flag nor `config.workspace` names one, a Switch-project into a
    /// tree whose parent differs from the old one used to leave
    /// `workspace_root` pointing at the OLD parent — and the Project picker
    /// (`C-x p`) browses `workspace_root` — until something unrelated called
    /// `reload_config`. Folding the derivation into one fn, called by both
    /// sites, makes that disagreement structurally impossible: there is no
    /// window where `project`/`file_index` reflect the new root while
    /// `workspace_root` still reflects the old one.
    pub(in crate::app) fn resync_project_location(&mut self, policy: location::LocationPolicy) {
        self.project_location.resync(policy);
    }

    /// Make `new_root` the ACTIVE project: re-derive everything `root` implies
    /// ([`Self::resync_project_location`]), reset the MRU, and re-sync the
    /// view. Shared by switch-project (C-x p) and the new-note jump (C-x n) so
    /// both re-scope the go-to list — AND the workspace the Project picker
    /// itself browses — the same way. No buffer is opened here (that is the
    /// caller's concern). Returns `false` ONLY when a MAS sandbox grant panel
    /// was cancelled (see the gate below) — every other path always switches
    /// and returns `true`; callers that persist a "switched to" fact (the
    /// sticky root, the recent-projects MRU) must check this before doing so.
    ///
    /// `new_root` is CANONICALIZED before it becomes `project_location.root`
    /// — the ONE root-identity owner: every later comparison (a working-set
    /// group's `OpenFile::root`, via [`crate::workingset::root_for`]'s
    /// `active_root` argument; the recent-projects MRU, via
    /// [`Self::push_recent_project`] reading this same field back after this
    /// call rather than trusting its own un-normalized argument) inherits one
    /// stable spelling instead of re-deriving its own. Reuses
    /// [`crate::buffers::normalize_path`] rather than a second
    /// canonicalizer — the same absolutize + symlink/firmlink-resolve +
    /// not-yet-existing-tail fallback [`crate::buffers::BufferKey`] already
    /// trusts for file identity, so a root and a file under it agree on what
    /// "the same path" means.
    pub(in crate::app) fn set_root(&mut self, new_root: PathBuf) -> bool {
        let new_root = crate::buffers::normalize_path(&new_root);
        // MAS SANDBOX GRANT GATE (native macOS `mas` builds only — see
        // `src/mas.rs`'s module doc): a project root reaches outside the
        // container far more often than a single file does, so this is the
        // OTHER real "touch outside the sandbox" door (Switch project…, the
        // C-x n notes-root jump). Same no-op-inside/first-touch-outside shape
        // as `load_path`'s gate.
        #[cfg(all(feature = "mas", target_os = "macos"))]
        if !crate::mas::ensure_access(&new_root) {
            return false;
        }
        // ROBUST AUTOSAVE: switching project re-scopes (and may precede a buffer
        // swap), so flush a pending note write first — never lose the open note.
        // The document autosave / scratch stash flushes on the same trigger.
        self.flush_note();
        self.autosave_flush();
        self.project_location.root = new_root;
        self.resync_project_location(self.config.location_policy());
        self.sync_view(false);
        self.request_frame();
        true
    }

    pub(in crate::app) fn new_document(&mut self) {
        let _ = crate::fs::active().create_dir_all(&self.project_location.root);
        self.start_fresh_document();
    }
}
