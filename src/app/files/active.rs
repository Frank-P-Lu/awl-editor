//! THE OWNED ACTIVE BUFFER SLOT (item 56) — the SOLE module that constructs
//! or destructures `App::active` (`crate::buffers::Entry<BufferExtra>`) or
//! touches the park/activate swap. Every other module reads
//! `self.active.buffer` / `self.active.extra.<field>` directly; the bypass
//! this module's doc warns about (a hand-rolled snapshot/restore pair, or a
//! raw `self.active = ...` outside `park_active_buffer`/
//! `activate_from_registry`) is a source-audit LAW
//! (`app::tests::source_audit`), not just a convention.
//!
//! `BufferExtra` is the App-level per-buffer bookkeeping that must travel
//! WITH a buffer across a park/activate swap — everything `App` tracks about
//! the ACTIVE buffer beyond the `Buffer` itself (whose cursor/selection/undo/
//! dirty/folds are already its own business). A swap is always a WHOLE-SLOT
//! `mem::replace`/assignment — never a field-by-field snapshot/restore — so a
//! future field added here travels correctly by construction with no
//! matching edit needed in `park_active_buffer`/`activate_from_registry`
//! (the Wagtail/version-0 class of bug this round retires — see
//! `CLAUDE.md`'s cache-key-discipline tripwire).
//!
//! NOT carried here (deliberately): the fresh-document debounce fields
//! (`autosave_dirty_at` / `autosave_saved_version`) stay App-global — they only
//! ever matter while `buffer.is_unnamed_fresh()`, and a fresh document only
//! becomes registry-keyable once it has been named (given a real path), at
//! which point it is an ordinary pathed buffer for every OTHER purpose here; a
//! stale value simply re-triggers one redundant (harmless) autosave on
//! reactivation.

use crate::app::*;

#[derive(Default)]
pub(in crate::app) struct BufferExtra {
    /// Whether the buffer's active selection (if any) was begun with Shift —
    /// TRANSIENT, but tied to THIS buffer's `anchor`, so it travels with it
    /// rather than leaking whatever the LAST-active buffer happened to leave it
    /// at (a plain unshifted motion in the reactivated buffer resets it anyway;
    /// this only matters for the one motion right after a switch).
    pub shift_selecting: bool,
    pub scroll_lines: usize,
    pub spell_cache: Vec<crate::spell::SpellVerdict>,
    pub spell_checked_version: Option<u64>,
    pub sync_text_cache: Option<(u64, String)>,
    pub caret_synced_version: u64,
    pub doc_saved_version: Option<u64>,
    pub scratch_saved_version: Option<u64>,
    pub disk_mtime: Option<crate::fs::Metadata>,
    pub scratch_mtime: Option<crate::fs::Metadata>,
    pub doc_autosave_at: Option<Instant>,
    /// DIFF-AS-PREVIEW cache (folded in from the old hand-cleared App field —
    /// item 56): the History overlay is never open across a buffer swap in
    /// practice, so this always reaches a park at its default `None`, which
    /// reproduces the old manual `self.active.extra.history_preview = None` clears
    /// automatically (whole-slot move, not a field-by-field snapshot).
    pub history_preview: Option<(String, String)>,
    /// The document scroll captured when the History timeline opened (folded
    /// in alongside `history_preview` for the same reason).
    pub history_scroll_before: Option<usize>,
}

impl App {
    /// PARK the active buffer into `buffer_registry` under its stable identity
    /// (a no-op for an ephemeral, still-empty pathless note — see
    /// `crate::buffers::BufferKey::of`), leaving `self.active` a throwaway
    /// scratch-buffer-with-default-extra placeholder for the caller to
    /// immediately overwrite. The ONE door every "the active buffer is about
    /// to be replaced" site goes through (`load_path`, `new_document`), so
    /// backgrounding a buffer always preserves the same state.
    ///
    /// WHOLE-SLOT MOVE (item 56): `mem::replace` swaps the ENTIRE
    /// `Entry<BufferExtra>` in one move — no field is enumerated here, so a
    /// future buffer-scoped field added to `BufferExtra` travels correctly by
    /// construction, with no matching edit needed in this function (the
    /// Wagtail/version-0 class of bug this round retires: see this module's
    /// doc / `CLAUDE.md`'s cache-key-discipline tripwire).
    pub(super) fn park_active_buffer(&mut self) {
        let Some(key) = crate::buffers::BufferKey::of(&self.active.buffer) else {
            return;
        };
        let outgoing = std::mem::replace(
            &mut self.active,
            crate::buffers::Entry { buffer: Buffer::scratch(), extra: BufferExtra::default() },
        );
        self.buffer_registry.park(key, outgoing);
    }

    /// ACTIVATE a backgrounded entry from the registry as the new active slot
    /// (a whole-slot move, the inverse half of `park_active_buffer`): `true`
    /// if `key` was resident and is now active, `false` if there was nothing
    /// to activate (first-time open this session) — the caller then builds a
    /// fresh `Entry` itself. THE ONE place that reads out of
    /// `buffer_registry` into `self.active`, so every switch site installs a
    /// COMPLETE entry in one assignment, never a half-moved slot.
    pub(super) fn activate_from_registry(&mut self, key: &crate::buffers::BufferKey) -> bool {
        match self.buffer_registry.take(key) {
            Some(entry) => {
                self.active = entry;
                true
            }
            None => false,
        }
    }

    /// How many buffers are open right now (the active one + everything
    /// backgrounded) — feeds the sidecar-analog debug line / future chrome.
    /// Not yet surfaced live (no chrome in v1); kept here as the one place
    /// that knows the count.
    #[allow(dead_code)]
    pub(in crate::app) fn open_buffer_count(&self) -> usize {
        self.buffer_registry.len() + 1
    }
}
