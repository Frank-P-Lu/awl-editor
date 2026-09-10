//! `DocumentSession`'s search key delegate — split out of `document.rs` to
//! keep it under its production ceiling. A thin forwarder onto
//! `crate::search::keys`, the one renderer-independent interception seam the
//! live keyboard door and the headless `--keys` replay already share. A
//! summoned find/replace panel CLICK does not live here: it resolves to an
//! `Action::SearchPanel` and reaches the same surface through
//! `actions::apply_transition` (`search::keys::intercept_action`) instead of
//! a second, click-only door into the document.

use super::*;

impl DocumentSession {
    /// Route a key to the active search surface (only called while
    /// `workspace_state.search_active()`). A thin delegate to the ONE
    /// renderer-independent interception seam
    /// — [`crate::search::keys::intercept`], shared verbatim with the headless
    /// `--keys` replay's search guard (`main/run.rs`), so the live panel and a
    /// replayed capture cannot drift. The seam consumes EVERY key
    /// (query/replacement typing, Backspace, C-s/C-r/arrow steps, M-c case
    /// toggle, Tab/Cmd-R field moves, Enter accept/replace, Cmd-Enter
    /// replace-all, Esc/C-g abort) and moves the REAL buffer cursor onto the
    /// current match, so the existing amber caret shows it for free. The
    /// returned recoil is the one LIVE-only consequence — a boundary step's
    /// failing-I-search bump — armed by the caller on the visual caret.
    pub(in crate::app) fn intercept_search_key(
        &mut self,
        search: &mut Option<crate::search::SearchState>,
        logical: &winit::keyboard::Key,
        mods: winit::keyboard::ModifiersState,
    ) -> Option<crate::caret::RecoilDir> {
        crate::search::keys::intercept(search, &mut self.active_entry_mut().buffer, logical, mods)
    }
}
