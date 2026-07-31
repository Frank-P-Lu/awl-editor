//! THE SUMMONED WORKSPACE'S CONTENT MODEL (queue item 114).
//!
//! Item 173 put the summoned-UI LIFECYCLE in [`super::Journey`] — entry, focus
//! transfer, child suspend/return, Back, exit and the parked-parent position.
//! This module is the other half: what a sustained workspace SHOWS, as data the
//! renderer and the sidecar both read, with no second copy of the navigation
//! rules.
//!
//! # Two surfaces, one attention rule (DESIGN.md §5)
//!
//! An OVERLAY splits attention for a brief contextual choice: it keeps the
//! document visible because the user still needs it. A summoned WORKSPACE
//! RELOCATES attention for sustained work: it takes the viewport, leaves the
//! document as a quiet backdrop, and returns to the exact editor state on exit.
//!
//! [`OverlayKind::workspace_shell`] is the one owner of which side of that line
//! a kind falls on. It is deliberately NOT the same predicate as
//! [`OverlayKind::sustained`]: `sustained` says a kind has workspace LIFECYCLE
//! (a place you stay in, with a detail stage and a Back), and both Settings and
//! Version History have had that since item 173. `workspace_shell` says a kind
//! is PRESENTED as a relocated workspace, which today is Settings alone —
//! History's own migration is item 116, and giving it the shell here would land
//! that item's presentation without its content.
//!
//! # The two regions
//!
//! A Settings workspace is one task in two coordinated regions:
//!
//!   * the CATEGORY RAIL — the primary list, `All` plus the six authored
//!     categories, which is exactly the picker's existing faceting strip
//!     ([`crate::settings::SETTINGS_FACETS`]) stood on its end. There is no
//!     second category model: the rail's selection IS `facet_lens`, so
//!     `refilter` already narrows the rows and a `←/→` lens step and a rail move
//!     are one state change.
//!   * the ROWS pane — the search field over the settings rows for that
//!     category, with their current values and their existing controls (range
//!     rails, toggles, sub-pickers).
//!
//! Which region holds focus is [`super::OverlayState::detail_focus`], written
//! only by the lifecycle: the rail is the workspace's PRIMARY list
//! ([`super::Surface::Workspace`]) and the rows are its DETAIL stage
//! ([`super::Surface::WorkspaceDetail`]). That is what makes `Esc` on the rows a
//! BACK to the rail and `Esc` on the rail an exit to the editor, at every width,
//! without any arm of the transition table being able to see the width.
//!
//! WIDTH IS PRESENTATION, NOT LIFECYCLE (item 173's premise correction 3). Wide
//! draws both regions side by side and focus moves between them; narrow draws
//! one at a time and the same focus fact becomes the stage you are on. The
//! renderer decides that from the canvas; nothing here knows the width.

use super::{OverlayKind, OverlayState};

impl OverlayKind {
    /// Is this kind PRESENTED as a summoned workspace — relocating attention to
    /// the viewport with a navigation rail beside its content — rather than as a
    /// contextual card floating over a still-readable document?
    ///
    /// Wildcard-free: a new picker kind must decide which surface grammar it
    /// belongs to before it compiles. See the module doc for why this is a
    /// different question from [`OverlayKind::sustained`].
    pub fn workspace_shell(self) -> bool {
        match self {
            OverlayKind::Settings => true,
            // Item 116 moves Version History onto this shell together with its
            // timeline/comparison content. It keeps its card presentation until
            // then, because a shell without the content it was designed for is
            // exactly the empty workspace item 114 forbids.
            OverlayKind::History => false,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::MoveDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName => false,
        }
    }
}

impl OverlayState {
    /// Is this card drawn as a summoned workspace?
    pub fn workspace_shell(&self) -> bool {
        self.kind.workspace_shell()
    }

    /// Move the workspace's RAIL selection by `delta` categories. The one door
    /// for `↑`/`↓` while the rail holds focus; a plain alias of the lens cycle,
    /// so the rail and `←/→` from the rows pane can never disagree about what
    /// "the current category" means.
    pub fn rail_move(&mut self, delta: isize) {
        self.cycle_lens(delta);
    }

    /// Point the rail at the category a given settings row lives under — the
    /// deep-link door (`Cmd-P` → a `§ setting` row → this workspace, standing on
    /// that row under its own category). A no-op when the row's category names
    /// no lens, so an unknown category degrades to the `All` home rather than
    /// landing nowhere.
    pub fn rail_focus_category(&mut self, category: &str) {
        let Some(sc) = self.facet_scheme() else { return };
        let Some(idx) = sc
            .strip
            .iter()
            .position(|f| f.sections.first().copied() == Some(category))
        else {
            return;
        };
        self.set_facet_lens(idx);
    }

    /// Select the row whose corpus accept-string is `accept`, if the current
    /// filter shows it. Returns whether it landed. The row half of the deep
    /// link; the category half is [`Self::rail_focus_category`].
    pub fn select_accept(&mut self, accept: &str) -> bool {
        let Some(pos) = self
            .items
            .iter()
            .position(|&ci| self.rows[ci].accept == accept)
        else {
            return false;
        };
        self.selected = pos;
        self.scroll_to_selected();
        true
    }
}
