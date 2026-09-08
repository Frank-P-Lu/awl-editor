//! `DocumentSession`'s search/replace delegates — split out of `document.rs`
//! to keep it under its production ceiling. Every arm here is a THIN
//! forwarder onto `crate::search::keys`, the one renderer-independent
//! interception seam the live keyboard door and the headless `--keys` replay
//! already share; a summoned find/replace panel CLICK routes through the
//! exact same functions its matching keyboard chord already drives, so a
//! click can never diverge into a second, click-only reimplementation.

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

    /// A CLICK on the panel's `Match case` checkbox: the same
    /// `search::keys::toggle_case_and_jump` the keyboard's ⌘⌥C/M-c door
    /// already calls, so a click can never diverge from the chord it mirrors
    /// (the click driver used to reimplement the recompute + cursor-follow
    /// inline, and that copy was missing the keyboard door's
    /// `reveal_placement` call — a case toggle on a match inside a folded
    /// section left the real cursor logically inside a hidden row).
    pub(in crate::app) fn search_toggle_case(
        &mut self,
        search: &mut Option<crate::search::SearchState>,
    ) {
        crate::search::keys::toggle_case_and_jump(search, &mut self.active_entry_mut().buffer);
    }

    /// A CLICK on a nav prev/next button: the same step the keyboard's
    /// arrows / Cmd-F family already drive. The live-only recoil feedback is
    /// intentionally dropped here (a click has no failing-I-search bump to
    /// animate), mirroring how the headless `--keys` replay already ignores it.
    pub(in crate::app) fn search_step(
        &mut self,
        search: &mut Option<crate::search::SearchState>,
        dir: crate::search::Direction,
    ) {
        crate::search::keys::step(search, &mut self.active_entry_mut().buffer, dir);
    }

    /// A CLICK on the `Replace` button: the same replace-current-and-advance
    /// the keyboard's Enter (with the replace row up) already drives.
    pub(in crate::app) fn search_replace_current(
        &mut self,
        search: &mut Option<crate::search::SearchState>,
    ) {
        crate::search::keys::replace_current(search, &mut self.active_entry_mut().buffer);
    }

    /// A CLICK on the `Replace all` button: the same atomic replace-every-match
    /// the keyboard's Cmd/Super-Enter already drives.
    pub(in crate::app) fn search_replace_all(
        &mut self,
        search: &mut Option<crate::search::SearchState>,
    ) {
        crate::search::keys::replace_all(search, &mut self.active_entry_mut().buffer);
    }
}
