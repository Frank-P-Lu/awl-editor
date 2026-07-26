//! src/app/files/open.rs — opening folder-relative files, the last-buffer
//! (C-x b) toggle, project/folder switching + the recent-projects/recent-files
//! MRU pushes, Cmd-N's own re-scan half, and the i18n write-back-once +
//! fold-reveal jump helpers. Split out of the former `app/files.rs` monolith
//! (item 56); see `files/active.rs` for the owned slot this module swaps
//! through `park_active_buffer`/`activate_from_registry`, and
//! `files/document.rs` for the fresh-document buffer swap built on top of
//! these.

use super::active::BufferExtra;
use crate::app::*;

impl App {
    /// Settings command: open the config file into the buffer for editing AS TEXT,
    /// creating the commented default first if it does not exist. The palette runs
    /// this; you then edit + Cmd-S to save, which live-reloads (see `reload_config`).
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

    /// Credits command: open the embedded `CREDITS.md` into the buffer, exactly
    /// like Settings opens the config file. UNLIKE Settings, the source of truth
    /// is the BINARY (`credits::CREDITS_MD`), not a user-owned disk file — so this
    /// always REFRESHES the on-disk view to the embedded text before opening it
    /// (never a create-if-missing; the doc must never drift from what shipped).
    /// Routed through a real path (under `fs::data_root()`) rather than left
    /// path-less: a path-less buffer reads as SCRATCH to the autosave engine
    /// (`autosave_flush`'s `buffer.path().is_none()` arm), which would silently
    /// overwrite the user's real scratch stash the next time autosave flushes —
    /// see `credits.rs`'s module doc for the full reasoning.
    pub(in crate::app) fn open_credits(&mut self) {
        let path = crate::fs::data_root().join("credits.md");
        let fs = crate::fs::active();
        if let Some(parent) = path.parent() {
            let _ = fs.create_dir_all(parent);
        }
        if let Err(e) = crate::fs::write_atomic(&path, crate::credits::CREDITS_MD.as_bytes()) {
            eprintln!("could not write credits view {}: {e}", path.display());
            return;
        }
        self.load_path(path);
    }

    /// Guide command: open the embedded `GUIDE.md` into the buffer, exactly like
    /// Credits opens `CREDITS.md` (same on-disk-refresh-then-load pattern, same
    /// reasoning for why it is NOT left path-less — see `open_credits`'s doc
    /// above and `guide.rs`'s module doc). Rendered through `guide::render`
    /// (the CONVENTION-TRUTHFUL SURFACES round's chord-token substitution) at
    /// OPEN TIME for the live convention/platform, so the doc always names the
    /// chord that actually fires under THIS session.
    pub(in crate::app) fn open_guide(&mut self) {
        let path = crate::fs::data_root().join("guide.md");
        let fs = crate::fs::active();
        if let Some(parent) = path.parent() {
            let _ = fs.create_dir_all(parent);
        }
        let rendered = crate::guide::render(
            crate::convention::Convention::current(),
            crate::commands::Platform::current(),
        );
        if let Err(e) = crate::fs::write_atomic(&path, rendered.as_bytes()) {
            eprintln!("could not write guide view {}: {e}", path.display());
            return;
        }
        self.load_path(path);
    }

    /// SWITCH the active folder to `new_root` — the ONE owner of a genuine
    /// switch-project (both the `Project` picker's accepted folder AND the
    /// Recent Projects picker route here). Re-scopes the root ([`Self::set_root`]),
    /// EAGERLY remembers it as the one active-folder-context (`session_flush`,
    /// native only — see `app/session.rs`'s module doc; item 76 unified the old
    /// separate "sticky project root" config key into the ONE session-owned
    /// context, so a crash/relaunch right after a switch still resumes here,
    /// not the pre-switch folder), AND pushes it to the front of the persisted
    /// RECENT list ([`Self::push_recent_project`]).
    pub(in crate::app) fn switch_project(&mut self, new_root: PathBuf) {
        // A cancelled MAS grant panel (see `set_root`'s doc) means the switch
        // never happened — never persist/MRU a root we didn't actually move
        // into.
        if !self.set_root(new_root.clone()) {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.session_flush();
        self.push_recent_project(new_root);
    }

    /// Push `root` to the FRONT of the persisted RECENT PROJECT ROOTS (deduped +
    /// capped, [`crate::recents::push`]) and save the list ATOMICALLY. A save
    /// error is reported and swallowed (a lost MRU entry is never worth crashing
    /// a project switch). Native/live only — the headless capture never
    /// constructs an `App`, so this file is never touched from a capture.
    pub(in crate::app) fn push_recent_project(&mut self, root: PathBuf) {
        let list = std::mem::take(&mut self.recent_projects);
        self.recent_projects = crate::recents::push(list, root, crate::recents::CAP);
        if let Err(e) = crate::recents::save(&crate::recents::recents_path(), &self.recent_projects)
        {
            eprintln!("recent-projects save failed: {e}");
        }
    }

    /// Push `file` to the FRONT of the persisted RECENTLY-OPENED FILES MRU (deduped
    /// + capped, [`crate::recent_files::push`]) and save it ATOMICALLY. A save error
    ///   is reported + swallowed (a lost MRU entry is never worth crashing a file
    ///   open). Native/live only — the headless capture never constructs an `App`, so
    ///   `recent-files.toml` is never touched from a capture. The FILE sibling of
    ///   [`Self::push_recent_project`].
    pub(in crate::app) fn push_recent_file(&mut self, file: PathBuf) {
        let list = std::mem::take(&mut self.recent_files);
        self.recent_files = crate::recent_files::push(list, file);
        if let Err(e) = crate::recent_files::save(&self.recent_files) {
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
        let path = crate::index::resolve(&self.root, rel);
        self.load_path(path);
    }

    /// C-x b last-buffer toggle: flip between the current and previously-opened
    /// file (a tiny 2-deep history). No-op until a second file has been opened.
    /// The two paths simply swap, so repeated C-x b ping-pongs between them.
    pub(in crate::app) fn last_buffer_toggle(&mut self) {
        let Some(prev) = self.prev_file.clone() else {
            return; // nothing opened before; toggle is a quiet no-op
        };
        self.load_path(prev);
    }

    /// Swap in the buffer for `path`: remember the file we are LEAVING as
    /// `prev_file` (the 2-deep last-buffer history), then either SWITCH to its
    /// already-open live buffer (unsaved edits + cursor + scroll + undo + spell
    /// state all survive — the multi-buffer registry win) or read it fresh from
    /// disk for a first-time open. Shared by `open_rel` and the C-x b toggle so
    /// both keep the history honest.
    pub(in crate::app) fn load_path(&mut self, path: PathBuf) {
        // ITEM 77 — THE ONE CAPABILITY OWNER, first: a binary/unsupported
        // `path` is refused HERE, before the MAS grant probe below (never
        // powerbox a file we're about to refuse) or any other side effect —
        // the active buffer, `self.root`, and every remembered-context field
        // stay exactly as they were. This is the door EVERY picker selection
        // (open_rel), the C-x b toggle, AND the daemon's `open` handoff
        // (`App::handle_daemon_event`, which calls this same fn) all share —
        // see `crate::openable`'s module doc for the full door list.
        if let Some(msg) = crate::openable::classify(&path).refusal_message() {
            self.set_sticky_notice(msg);
            return;
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
            return;
        }
        // ROBUST AUTOSAVE: before we drop the current buffer, flush any pending
        // note write so nothing typed in the last debounce window is lost — and
        // flush the LEAVING document / scratch through the autosave engine
        // (locked decision: save on file switch).
        self.flush_note();
        self.autosave_flush();
        // WRITING STREAKS: sample the LEAVING buffer's word-delta BEFORE it is
        // replaced below, so words written in it this session are recorded against
        // the right document; the anchor is reset after the swap so the arriving
        // buffer's existing words are never miscounted (native only; gated inside).
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_flush();
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
        if self
            .active
            .buffer
            .path()
            .map(crate::buffers::BufferKey::path)
            == Some(crate::buffers::BufferKey::path(&path))
        {
            return;
        }
        // The file we are leaving becomes the last-buffer target — captured
        // BEFORE `park_active_buffer` below moves the slot away.
        self.prev_file = self.active.buffer.path().map(|p| p.to_path_buf());
        self.park_active_buffer();
        let key = crate::buffers::BufferKey::path(&path);
        // ALREADY OPEN elsewhere in this session: switch to its LIVE buffer
        // instead of re-reading disk — unsaved edits, cursor, scroll, undo,
        // and spell-cache state all survive the round trip (a whole-slot
        // activation — see `activate_from_registry`).
        if !self.activate_from_registry(&key) {
            // First time open this session: read fresh from disk — build a
            // COMPLETE entry and install it in ONE move (item 56: no
            // half-moved slot).
            self.active = crate::buffers::Entry {
                buffer: Buffer::from_file(&path),
                extra: BufferExtra::default(),
            };
            // AUTOSAVE bookkeeping for the ARRIVING file: its buffer IS the
            // on-disk content, so it starts saved; the current mtime is the
            // clobber guard's baseline. Stamped BEFORE the i18n write-back
            // below, so a stamped tag correctly reads as a PENDING edit
            // (buffer.version() past doc_saved_version) rather than being
            // mistaken for already-on-disk content — autosave picks it up
            // on the next idle/blur/switch/quit exactly like any other edit.
            self.active.extra.disk_mtime = Self::disk_mtime_of(&path);
            self.active.extra.doc_saved_version = Some(self.active.buffer.version());
            // A brand-new buffer starts at version 0; match the synced
            // version so the next sync_view doesn't read the delta as an
            // edit and streak the caret.
            self.active.extra.caret_synced_version = self.active.buffer.version();
            // i18n WRITE-BACK-ONCE: an untagged CJK document gets a `lang:`
            // frontmatter tag stamped in as one normal undoable edit (never
            // for a pure-Latin doc, never a second time on a doc that
            // already carries a frontmatter block). Live-App-only by
            // construction (called only from this fresh-open branch) — the
            // headless `load_buffer` never reaches this function at all.
            self.write_back_lang_tag_once();
        }
        if !clobber_notice_just_raised {
            self.clear_notice();
        }
        // `Buffer::path()` already carries this exact path — on the fresh-open
        // arm from `Buffer::from_file(&path)`, on the registry-hit arm from
        // the entry's own remembered path (it was parked under a key derived
        // from that same path) — so there is no separate `App.file` mirror to
        // write here any more (item 56: `Buffer::path()` is the sole source).
        // RECENTLY-OPENED FILES MRU: this file was just OPENED (either fresh from
        // disk or switched-to from the buffer registry — BOTH arrive here), so push
        // it to the front of the persisted MRU that feeds the go-to Recent lens +
        // recency tier. After the already-active early-return above, so re-selecting
        // the current file is a no-op that never re-orders the MRU.
        self.push_recent_file(path.clone());
        // LIFETIME STATS: record this open into the distinct-files set (deduped),
        // beside the recent-files MRU push — the same door. Native-only + config-
        // gated inside; a re-open of an already-seen path is inert.
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_touch_file(path);
        // LIFETIME STATS: the buffer just swapped — drop the caret-travel anchor
        // so the new document's first caret sample re-anchors rather than counting
        // the cross-document coordinate jump as travel.
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_reset_caret_anchor();
        // WRITING STREAKS: the buffer just swapped — drop the word-delta anchor so
        // the arriving document's existing words re-anchor (never counted as
        // freshly written) on its first flush.
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_reset_baseline();
        // LIFETIME STATS: flush on the file-SWITCH trigger (the same door the
        // autosave flush above rides), so the just-recorded touch + any pending
        // keystroke/caret increments survive the switch (native only; gated).
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_flush();
        self.search = None;
        self.preedit.clear();
        // The HISTORY TIMELINE preview cache is now buffer-scoped
        // (`BufferExtra::history_preview`, item 56): the ARRIVING buffer's own
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
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }

    /// i18n WRITE-BACK-ONCE: on a fresh (first-time-this-session) open of an
    /// UNTAGGED markdown document that contains CJK, stamp a `lang:`
    /// frontmatter tag in as ONE normal undoable buffer edit — never a silent
    /// disk write (the version bump is picked up by the ordinary autosave
    /// engine on the next idle/blur/switch/quit, exactly like any other edit;
    /// Cmd-Z removes it cleanly, restoring the pre-tag text and cursor). Called
    /// ONLY from [`Self::load_path`]'s fresh-disk-read branch, so:
    ///  - a PURE-LATIN document ([`crate::script::dominant_cjk`] returns `None`)
    ///    is NEVER touched — no frontmatter block, no version bump, no undo
    ///    entry;
    ///  - a document that ALREADY carries a frontmatter block (tagged or not)
    ///    is NEVER re-tagged — [`crate::frontmatter::detect`] finds it and this
    ///    returns immediately, so write-back happens AT MOST ONCE in a
    ///    document's life (a later reopen this session hits the buffer-
    ///    registry SWITCH branch instead, which never calls this at all; a
    ///    reopen in a FRESH session sees the tag already on disk from the
    ///    first pass and detects it, so it still never re-fires);
    ///  - a NON-markdown buffer (a `.rs`/`.txt`/`.env` path) is never touched —
    ///    frontmatter is a markdown/notes convention, and stamping literal
    ///    `---`/`lang:` text into a code file would corrupt it.
    ///    A Han-only (ambiguous) document resolves via the config `cjk_priority`
    ///    ladder (default ja-first); an unambiguous script (kana/hangul/bopomofo)
    ///    always wins regardless of the ladder — see `crate::script::dominant_cjk`
    ///    / `doc_lang_for`.
    pub(in crate::app) fn write_back_lang_tag_once(&mut self) {
        if !self.active.buffer.is_markdown() {
            return;
        }
        let text = self.active.buffer.text();
        if crate::frontmatter::detect(&text).is_some() {
            return; // already carries a frontmatter block — never re-tag
        }
        let Some(script) = crate::script::dominant_cjk(&text) else {
            return; // pure Latin — never touched
        };
        let lang = crate::script::doc_lang_for(script, &self.config.cjk_priority_or_default());
        let block = format!("---\nlang: {}\n---\n", lang.code());
        self.active.buffer.replace_char_range(0, 0, &block);
    }

    /// Jump the cursor to the START of the 0-based `line`. Clears any selection,
    /// then re-syncs the view so the target scrolls into view. Callers pass the
    /// line directly: Go-to's HEADINGS lens (`Effect::JumpToLine`) and a click on
    /// a persistent margin-outline row.
    pub(in crate::app) fn jump_to_line(&mut self, line: usize) {
        let idx = self.active.buffer.line_col_to_char(line, 0);
        self.active.buffer.clear_mark();
        self.active.buffer.set_cursor(idx);
        // REVEALED PLACEMENT (folds): a heading Go-to / margin-outline jump may target
        // a line hidden inside a collapsed section — route through the ONE placement
        // owner so the landing line is revealed, never left inside a fold. A cheap
        // no-op unless a section is folded.
        self.active.buffer.reveal_placement();
        self.active.extra.shift_selecting = false;
        self.sync_view(true);
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
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
    pub(in crate::app) fn rescan_file_index(&mut self) {
        self.file_index = crate::index::build_index(&self.root);
    }

    /// Make `new_root` the ACTIVE project: re-resolve the project, rebuild the
    /// file index, reset the MRU, and re-sync the view. Shared by switch-project
    /// (C-x p) and the new-note jump (C-x n) so both re-scope the go-to list the
    /// same way. No buffer is opened here (that is the caller's concern).
    /// Returns `false` ONLY when a MAS sandbox grant panel was cancelled (see
    /// the gate below) — every other path always switches and returns `true`;
    /// callers that persist a "switched to" fact (the sticky root, the recent-
    /// projects MRU) must check this before doing so.
    pub(in crate::app) fn set_root(&mut self, new_root: PathBuf) -> bool {
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
        self.root = new_root;
        self.project = crate::project::Project::resolve(&self.root);
        self.rescan_file_index();
        self.sync_view(false);
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
        true
    }

    /// Cmd-N: a fresh, unnamed DOCUMENT in the CURRENT active folder (item 76 —
    /// no root jump: New document used to jump to a separate "notes root"; now
    /// it lands wherever you already are). The user starts typing immediately;
    /// the filename is derived (slugified first line) ONCE, on the first
    /// material save — see [`Self::autosave_note`] / `Buffer::save`. The file
    /// we are leaving becomes the last-buffer (C-x b) target.
    pub(in crate::app) fn new_document(&mut self) {
        // The active folder may not exist yet on a fresh machine; create it
        // lazily so the buffer's first save has somewhere to land (best-effort
        // — a MAS sandbox build against an ungranted external root simply has
        // this attempt fail silently; the folder is already granted by
        // construction, since it's the folder we are already working in).
        let _ = crate::fs::active().create_dir_all(&self.root);
        self.start_fresh_document();
    }
}
