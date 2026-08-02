//! src/app/workspace/ — THE SUMMONED-UI LAYER OWNER (`WorkspaceState`, queue
//! item 172's first slice, extended by item 173; map: `docs/app-domains.md`).
//!
//! awl summons four surfaces over the editor, and they form a strict
//! PRECEDENCE LADDER:
//!
//! ```text
//!   Overlay   a modal picker (Cmd-P / go-to / browse / theme / keybindings /
//!             spell), or a CHILD AUDITION over a parked workspace — owns the
//!             keyboard and the pointer while it is up
//!     ▲
//!   Workspace a SUSTAINED summoned workspace (Settings, Version History): you
//!             stay in it and navigate within it
//!     ▲
//!   Search    the summoned find/replace panel — owns every key, but not the
//!             pointer outside its own card
//!     ▲
//!   Popover   the reveal-on-select format toolbar — a pure mouse affordance
//!     ▲
//!   Editor    nothing summoned
//! ```
//!
//! Before this module the ladder did not exist as a thing. It was three
//! independent `App` fields (`overlay: Option<_>`, `search: Option<_>`,
//! `popover_open: bool`) plus the SAME conjunction re-typed at five sites
//! (`app/viewstate.rs`'s popover gate, `app/input/mouse.rs`'s Cmd-click link
//! follow, its popover-button press, its popover summon-on-release, and the
//! overlay-before-search `else if` in its press dispatch) and five independent
//! writers spread over five files. Nothing stopped a sixth site from spelling
//! the rule differently, and nothing recorded that the two "impossible"
//! combinations (an overlay open on top of a summoned popover; a search panel
//! and an overlay at once) resolve the way they do.
//!
//! ONE OWNER, ONE RULE: [`WorkspaceState::layer`] is the sole description of
//! the ladder, and every ladder question in the tree derives from it
//! ([`WorkspaceState::overlay_open`], [`WorkspaceState::pickers_clear`],
//! [`WorkspaceState::popover_holds_attention`]). The fields are PRIVATE to this
//! module, so a consumer physically cannot re-derive the rule — the compile
//! error is the enforcement, and `app/tests/domains.rs` guards the fields from
//! being re-added to root `App`.
//!
//! Item 173 added the fourth rung. The LIFECYCLE behind it —  which surface is
//! up, what is parked beneath it, and where every Esc/Back/accept lands — is
//! [`crate::overlay::Journey`], in the core the live App and the headless
//! `--keys` replay share; `WorkspaceState` owns the one live instance and asks
//! it for a single closed fact ([`crate::overlay::Rung`]). The ladder therefore
//! reads the lifecycle instead of re-deriving it, which is what keeps
//! "one owner" true across two owners of two different rules.
//!
//! The ladder is an enum rather than a bool trio precisely so that adding that
//! rung was a compile error at every no-wildcard match over `Layer` until it
//! was placed.

use crate::overlay::{Journey, OverlayState, Rung};
use crate::search::SearchState;

/// Which summoned surface currently holds attention. Ordered LOW to HIGH so
/// `derive(PartialOrd)` reads as the ladder itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::app) enum Layer {
    /// Nothing summoned: the document owns the keyboard and the pointer.
    Editor,
    /// The reveal-on-select format toolbar is up (mouse affordance only).
    Popover,
    /// The find/replace panel is up and consuming every key.
    Search,
    /// A SUSTAINED summoned workspace is up and owns both keyboard and pointer.
    /// It sits BELOW `Overlay` because a child audition summoned out of a
    /// workspace is modal over it — the workspace is parked while the child
    /// holds attention, and "highest present rung wins" then names the child.
    Workspace,
    /// A modal picker is up and owns both keyboard and pointer.
    Overlay,
}

impl Layer {
    /// Every rung, low to high. Paired with the no-wildcard matches below: a
    /// new rung must be added here to be swept by the ladder law.
    #[cfg(test)]
    pub(in crate::app) const ROSTER: &'static [Layer] = &[
        Layer::Editor,
        Layer::Popover,
        Layer::Search,
        Layer::Workspace,
        Layer::Overlay,
    ];
}

/// THE SUMMONED-UI LAYER STATE. Fields are private on purpose: every write is
/// a named transition below, so the ladder cannot be violated by assignment.
#[derive(Default)]
pub(in crate::app) struct WorkspaceState {
    /// THE SUMMONED-UI JOURNEY: which card is up, what is parked beneath it,
    /// and the closed lifecycle that owns every Esc/Back/accept outcome.
    /// Content mutation goes through [`Self::overlay_mut`]; the journey itself
    /// is only ever advanced by the shared action core via [`Self::core_slots`].
    journey: Journey,
    /// The summoned find/replace panel, or `None`.
    search: Option<SearchState>,
    /// The format popover's SUMMON BIT. Not "is the popover visible" — the
    /// ladder decides that ([`Self::popover_holds_attention`]). Only
    /// [`Self::summon_popover`] may set it true, and it applies the ladder on
    /// the way in, so the bit can never be armed underneath a picker.
    popover_summoned: bool,
    tutorial_folder_intent: Option<super::files::TutorialFolderIntent>,
}

impl WorkspaceState {
    pub(in crate::app) fn set_tutorial_folder_intent(
        &mut self,
        intent: super::files::TutorialFolderIntent,
    ) {
        self.tutorial_folder_intent = Some(intent);
    }

    pub(in crate::app) fn take_tutorial_folder_intent(
        &mut self,
    ) -> Option<super::files::TutorialFolderIntent> {
        self.tutorial_folder_intent.take()
    }

    // ─── THE LADDER: ONE OWNER ───────────────────────────────────────────

    /// Which rung holds attention. **The sole description of the precedence
    /// rule** — every other ladder question in the tree derives from this.
    ///
    /// Wildcard-free by design: all eight combinations of the three underlying
    /// facts are spelled out, including the two that are unreachable in
    /// practice (a picker cannot be summoned while the find panel eats every
    /// key, and `summon_popover` refuses to arm under a picker). Writing them
    /// out is the point — the unreachable cells are exactly the ones a
    /// hand-written conjunction gets wrong when a future rung makes them
    /// reachable, and the law test below asserts every cell rather than the
    /// four the author happened to imagine.
    pub(in crate::app) fn layer(&self) -> Layer {
        match (
            self.journey.rung(),
            self.search.is_some(),
            self.popover_summoned,
        ) {
            (Rung::Modal, true, true) => Layer::Overlay,
            (Rung::Modal, true, false) => Layer::Overlay,
            (Rung::Modal, false, true) => Layer::Overlay,
            (Rung::Modal, false, false) => Layer::Overlay,
            (Rung::Sustained, true, true) => Layer::Workspace,
            (Rung::Sustained, true, false) => Layer::Workspace,
            (Rung::Sustained, false, true) => Layer::Workspace,
            (Rung::Sustained, false, false) => Layer::Workspace,
            (Rung::Nothing, true, true) => Layer::Search,
            (Rung::Nothing, true, false) => Layer::Search,
            (Rung::Nothing, false, true) => Layer::Popover,
            (Rung::Nothing, false, false) => Layer::Editor,
        }
    }

    /// Is a modal picker up? (The top rung.) Replaces every
    /// `self.overlay.card().is_some()`.
    pub(in crate::app) fn overlay_open(&self) -> bool {
        matches!(self.layer(), Layer::Overlay | Layer::Workspace)
    }

    /// Is NO picker up — i.e. does the document (possibly with its format
    /// popover) still own the pointer? Replaces the
    /// `overlay.card().is_none() && search.is_none()` conjunction.
    pub(in crate::app) fn pickers_clear(&self) -> bool {
        matches!(self.layer(), Layer::Editor | Layer::Popover)
    }

    /// Is the format popover the rung holding attention — i.e. summoned AND
    /// unshadowed? Replaces the
    /// `popover_open && overlay.card().is_none() && search.is_none()` conjunction.
    pub(in crate::app) fn popover_holds_attention(&self) -> bool {
        matches!(self.layer(), Layer::Popover)
    }

    /// Is the find/replace panel present, REGARDLESS of the ladder?
    ///
    /// Deliberately NOT `layer() == Layer::Search`: the live key path's search
    /// guard (`app/input/keys.rs`) runs before keymap resolution and consumes
    /// every key whenever the panel exists, which is a different question from
    /// "is the panel the top rung". Keeping the two distinct preserves today's
    /// behavior exactly; collapsing them would silently hand keys to an
    /// overlay in the combination the ladder calls `Overlay`.
    pub(in crate::app) fn search_active(&self) -> bool {
        self.search.is_some()
    }

    /// The popover's raw SUMMON BIT, before the ladder.
    ///
    /// THE ONE DELIBERATE BYPASS in this module, with exactly one call site:
    /// `sync_cursor_icon`'s popover-button hover test in
    /// `app/input/mouse.rs`. It stays ladder-free so the cursor-icon
    /// composition is byte-identical to the pre-item-172 code; the pipeline's
    /// own popover model is already cleared when a picker is up, so the
    /// combination cannot change the resulting icon. Every other consumer must
    /// ask [`Self::popover_holds_attention`].
    /// `app/tests/domains.rs` counts this method's call sites.
    pub(in crate::app) fn popover_summon_bit(&self) -> bool {
        self.popover_summoned
    }

    // ─── OVERLAY TRANSITIONS ─────────────────────────────────────────────

    /// Read the summoned picker's content.
    pub(in crate::app) fn overlay(&self) -> Option<&OverlayState> {
        self.journey.card()
    }

    /// Read the WHOLE summoned-overlay journey, for the sidecar fold (item 188):
    /// a parked parent is lifecycle state, not card content, so `overlay()`
    /// cannot answer it — the same reason `ReplaySession::journey` exists.
    /// Read-only; `core_slots` stays the only way to mutate it. Native/test only,
    /// like the `--screenshot-app` capture that is its consumer.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(in crate::app) fn journey(&self) -> &Journey {
        &self.journey
    }

    /// Mutate the summoned picker's CONTENT (selection, scroll, notice, query,
    /// captured binding, …). The picker's own type owns those rules; this
    /// module owns only whether a picker is up at all, which is why this hands
    /// out `&mut OverlayState` and never `&mut Option<OverlayState>`.
    pub(in crate::app) fn overlay_mut(&mut self) -> Option<&mut OverlayState> {
        self.journey.card_mut()
    }

    /// Dismiss EVERY picker at once, dropping straight to the editor (or to
    /// the popover, if one was summoned before the picker shadowed it — which
    /// is the pre-existing behavior, unchanged). The one caller is the awl menu
    /// bar's title press: opening a menu-bar dropdown must clear whatever was
    /// summoned underneath it.
    pub(in crate::app) fn dismiss_pickers(&mut self) {
        self.journey.dismiss();
        self.search = None;
    }

    /// Pointer-only entrance for the awl-rendered contextual card. The card's
    /// rows remain ordinary catalog Actions and acceptance returns through the
    /// shared action core; this method owns only the named summon transition.
    pub(in crate::app) fn summon_context(&mut self, card: OverlayState) {
        self.search = None;
        self.popover_summoned = false;
        self.journey.enter(Some(card));
    }

    // ─── SEARCH TRANSITIONS ──────────────────────────────────────────────

    /// Read the find/replace panel's state (matches, query, focus).
    pub(in crate::app) fn search(&self) -> Option<&SearchState> {
        self.search.as_ref()
    }

    /// Mutate the find/replace panel's CONTENT (field focus, case toggle).
    pub(in crate::app) fn search_mut(&mut self) -> Option<&mut SearchState> {
        self.search.as_mut()
    }

    /// FOCUS the summoned workspace's DETAIL stage — its content pane — leaving
    /// it there if it is already focused. The pointer's counterpart to `↵` on a
    /// navigation-rail entry (`app/input/mouse.rs::overlay_click`): a rail click
    /// means "show me this category, and put me in it" at every width, and a
    /// click that landed you somewhere different from the key that means the same
    /// thing would be two behaviours.
    ///
    /// A named transition rather than a `core_slots` borrow, because it is one:
    /// the ladder's rule is that every write here has a name.
    pub(in crate::app) fn focus_workspace_detail(&mut self) {
        if self.journey.card().is_some_and(|card| !card.detail_focus) {
            self.journey.toggle_detail();
        }
    }

    /// Close the find/replace panel. Called on every buffer swap (opening a
    /// file, starting a fresh document) — a panel's matches are indices into
    /// the buffer it was opened against, so it can never survive the swap.
    pub(in crate::app) fn close_search(&mut self) {
        self.search = None;
    }

    /// Lend BOTH slots to the shared action core (`actions::ActionCtx`, the
    /// item-171 typed-transition seam), which is the one place allowed to open
    /// or close a picker/panel, because opening one is an `Action` outcome and
    /// the headless `--keys` replay must reach the identical code.
    ///
    /// Returns both in one call so the caller cannot hold two overlapping
    /// mutable borrows of this type, and — unlike the `take()` … `= put_back`
    /// pair this replaced — there is nothing to forget to give back: the slots
    /// are borrowed in place, never moved out.
    pub(in crate::app) fn core_slots(&mut self) -> (&mut Option<SearchState>, &mut Journey) {
        (&mut self.search, &mut self.journey)
    }

    // ─── POPOVER TRANSITIONS ─────────────────────────────────────────────

    /// SUMMON (or refuse to summon) the format popover after a mouse gesture.
    /// `eligible` carries the caller's own non-ladder conditions (the feature
    /// is on, there IS a selection, the buffer is markdown); the LADDER
    /// condition is applied here, so no caller can arm the bit underneath a
    /// picker. Passing `eligible: false` dismisses, which is what makes this
    /// the whole of the mouse-release rule.
    pub(in crate::app) fn summon_popover(&mut self, eligible: bool) {
        self.popover_summoned = eligible && self.pickers_clear();
    }

    /// Dismiss the format popover. Any real key press does this (it is a
    /// mouse-only affordance), as does a press outside its card.
    pub(in crate::app) fn dismiss_popover(&mut self) {
        self.popover_summoned = false;
    }

    /// TEST-ONLY: install a picker directly, so a test can drive a picker's
    /// own behavior (rail scrub, history diff nav, asset trashing) without
    /// replaying the whole Action that summons it. Absent from a non-test
    /// build ON PURPOSE — production may only open a picker through
    /// [`Self::core_slots`], which is what keeps "who can summon a picker"
    /// answerable with a grep.
    #[cfg(test)]
    pub(in crate::app) fn install_overlay_for_test(&mut self, overlay: OverlayState) {
        self.journey.install_for_test(overlay);
    }

    /// TEST-ONLY: the find/replace panel's twin of
    /// [`Self::install_overlay_for_test`]. Production opens the panel only
    /// through [`Self::core_slots`], and the law over that seam counts its call
    /// sites — so a test that needs the panel standing takes this door rather
    /// than widening the one the law guards.
    #[cfg(test)]
    pub(in crate::app) fn install_search_for_test(&mut self, search: SearchState) {
        self.search = Some(search);
    }
}

#[cfg(test)]
mod tests;
