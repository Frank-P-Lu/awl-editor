//! The summoned workspace's content model.
//!
//! [`super::Journey`] owns entry, focus transfer, child suspend/return, Back,
//! exit and the parked-parent position. This module owns what a sustained
//! workspace shows, as data the
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
//! [`OverlayKind::workspace_shape`] is the one owner of which side of that line
//! a kind falls on, and — for a kind that IS a workspace — which of the two
//! shapes DESIGN.md §5 sanctions it draws as. It is deliberately NOT the same
//! predicate as [`OverlayKind::sustained`]: `sustained` says a kind has
//! workspace LIFECYCLE (a place you stay in, with a detail stage and a Back),
//! and both Settings and Version History are sustained surfaces.
//! `workspace_shape` says a kind is PRESENTED as a relocated workspace and
//! which shape. WHICH kinds those are is that match's own answer, never a list
//! here: it is wildcard-free, so the set cannot change without a decision.
//!
//! # One shape is not enough
//!
//! A rail of category labels beside a pane of rows (`RailOverRows`) is the
//! shape Settings needs.
//! Version History needs the opposite arrangement: a narrow TIMELINE whose
//! rows ARE the primary list, beside a large read-only comparison. Flipping a
//! bare bool would put History's rows in the wide pane and leave the rail
//! showing nothing — the wrong composition, not merely an incomplete one. Both
//! shapes are named in [`WorkspaceShape`], and [`WorkspaceShape::rows_are_primary`]
//! is the one fact every consumer — geometry, keyboard handling, the footer
//! hint — reduces to, rather than each re-deriving which region is a workspace's
//! row list. Its match over the two variants is the only one anywhere in the
//! crate (a grep-law enforces this); every other reader calls the method.
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
//! ([`super::Surface::WorkspaceDetail`]). `Esc` leaves outright from EITHER of
//! them — the settled decision the transition table spells out cell by cell —
//! so the way back to the rail is a key of its own, and [`BackKey`] below is the
//! one owner of which key that is.
//!
//! Width is presentation, not lifecycle. Wide
//! draws both regions side by side and focus moves between them; narrow draws
//! one at a time and the same focus fact becomes the stage you are on. The
//! renderer decides that from the canvas; nothing here knows the width.

use super::{HintAction, OverlayKind, OverlayState};

use super::ARROWS_UD;

/// The FOCUS-TRANSFER key, as the footer spells it: `Tab`/`Shift-Tab` move
/// attention between a workspace's two regions.
pub(crate) const TAB_GLYPH: &str = "tab";

/// The ERASE key, as the footer spells it — the same glyph the folder
/// navigators already teach as `⌫ up`, because on a workspace it is the same
/// gesture aimed one rung differently: erase nothing, so go up a level.
pub(crate) const ERASE_GLYPH: &str = "\u{232B}";

/// **WHAT TAKES YOU BACK from a summoned workspace's DETAIL stage, right now.**
///
/// A workspace has to NAME its Back: `Esc` leaves outright from either region,
/// and awl's footer is its only statement of what a key does. But the key that
/// performs it is not constant, because the detail stage's OTHER keys are not
/// constant either — so this is a derived answer rather than a per-kind
/// literal, and [`OverlayState::detail_back`] is the one place that derives it.
/// Both consumers read that one owner: the action seam
/// (`crate::actions::workspace_nav`) to decide what the key does, and
/// [`OverlayState::foot_hint`] to decide what the footer says. They cannot
/// drift, because neither of them holds an opinion of its own.
///
/// Why `Erase` is the preferred answer, and `Tab` only the fallback: `Tab` is a
/// FOCUS key, and a focus key reads as a Back only while both regions are on
/// screen at once. Below `workspace_is_wide` the stage you left is not merely
/// unfocused, it is GONE — and "tab back" then teaches a gesture whose ordinary
/// meaning the surface no longer displays. `⌫` is the gesture awl's own folder
/// navigators already teach for "up one level" (`⌫ up` on Browse / Switch
/// project / Move to… / Export to…), under the same rule: it belongs to the
/// search field until the field is empty, and it goes up the moment it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackKey {
    /// `⌫` — free on this stage, so it goes back.
    Erase,
    /// `tab` — the erase key is busy editing this stage's live query, so the
    /// focus-transfer key is the honest answer for as long as that lasts.
    Focus,
}

impl BackKey {
    /// How the footer spells this key.
    pub(crate) fn glyph(self) -> &'static str {
        match self {
            BackKey::Erase => ERASE_GLYPH,
            BackKey::Focus => TAB_GLYPH,
        }
    }

    /// The footer cell this key earns: `⌫ back` / `tab back`. One spelling of
    /// the LABEL too, so the sentence a user reads on a settings pane and on a
    /// comparison is the same sentence.
    pub(crate) fn hint(self) -> HintAction {
        HintAction {
            glyph: self.glyph(),
            label: "back",
        }
    }
}

/// Which of a summoned workspace's two coordinated regions is its PRIMARY
/// list — DESIGN.md §5's "categories beside controls, or a timeline beside a
/// comparison" — named rather than left as a bool because the two shapes
/// place the SAME kind of content (a workspace's own row list) on opposite
/// sides.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WorkspaceShape {
    /// Settings, today: the primary (narrow) column is a rail of category
    /// LABELS; the workspace's own rows live in the wide content pane.
    RailOverRows,
    /// Version History: the primary (narrow) column IS the workspace's row list
    /// — a timeline of versions — and the wide region is a read-only COMPARISON
    /// the document layer itself relocates into
    /// (`TextPipeline::comparison_viewport`), composited onto the workspace surface.
    ///
    /// The LENS has nowhere to live in the primary column here, because that
    /// column carries the rows; it moves into the header instead, on the same
    /// strip line the grouped card family already owns.
    TimelineOverComparison,
}

impl WorkspaceShape {
    /// THE single fact geometry, keyboard handling and the footer hint all
    /// reduce to, rather than each re-deriving which region holds a
    /// workspace's own rows: `true` when the PRIMARY (narrow) column carries
    /// them, `false` when they live in the content pane instead and the
    /// primary column carries labels.
    ///
    /// This is the only match over [`WorkspaceShape`]'s variants anywhere in
    /// the crate — a grep-law (`render::tests::workspace_shape`)
    /// keeps it that way, so a third shape cannot silently carry a different
    /// answer in two places.
    pub(crate) fn rows_are_primary(self) -> bool {
        match self {
            WorkspaceShape::RailOverRows => false,
            WorkspaceShape::TimelineOverComparison => true,
        }
    }
}

impl OverlayKind {
    /// Is this kind PRESENTED as a summoned workspace — relocating attention to
    /// the viewport with two coordinated regions — rather than as a contextual
    /// card floating over a still-readable document? `Some` names which
    /// [`WorkspaceShape`] it draws as; `None` keeps the kind on its existing
    /// contextual-card or grouped-card presentation.
    ///
    /// Wildcard-free: a new picker kind must decide which surface grammar it
    /// belongs to before it compiles. See the module doc for why this is a
    /// different question from [`OverlayKind::sustained`].
    pub fn workspace_shape(self) -> Option<WorkspaceShape> {
        match self {
            OverlayKind::Settings => Some(WorkspaceShape::RailOverRows),
            OverlayKind::History => Some(WorkspaceShape::TimelineOverComparison),
            // THE SAME SHAPE, a different subject: a short list of views beside
            // the read-only prose one of them names. It is a workspace and not a
            // card because reading two manuscripts to decide between them is
            // sustained work, and because the document behind it is the very
            // thing being compared — leaving it visible would show a third text.
            OverlayKind::Conflict => Some(WorkspaceShape::TimelineOverComparison),
            // THE SAME SHAPE AGAIN, degenerated to its simplest case: ONE fixed
            // row standing in for the primary list (there is nothing to
            // navigate), beside the same relocated read-only prose Conflict and
            // History already draw into. A workspace because Credits is
            // sustained reading of a whole document, not a brief contextual
            // choice — the document behind it recedes as a quiet backdrop while
            // you read, same as a comparison's.
            OverlayKind::Credits => Some(WorkspaceShape::TimelineOverComparison),
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::TableDims
            | OverlayKind::SearchFolder => None,
        }
    }

    /// The foot hint while a summoned WORKSPACE's PRIMARY list — its navigation
    /// rail — holds focus. The rows pane's own hint is
    /// [`Self::hint_actions`]; this is the other stage's, and the two differ in
    /// exactly the keys that differ: on the rail `↑/↓` steps categories and `esc`
    /// leaves for the editor, while on the rows `↑/↓` steps rows and `esc` comes
    /// back here.
    ///
    /// Wildcard-free, like every other per-kind statement here: a kind that is
    /// not drawn as a workspace still has to say what its rail would advertise,
    /// which is nothing, and it can never be reached because
    /// [`crate::overlay::OverlayState::foot_hint`] gates on
    /// [`Self::workspace_shape`].
    pub fn rail_hint_actions(self) -> Vec<HintAction> {
        let enter = |label| HintAction {
            glyph: "\u{21B5}",
            label,
        };
        let key = |glyph, label| HintAction { glyph, label };
        match self {
            OverlayKind::Settings => vec![
                key(ARROWS_UD, "category"),
                enter("settings"),
                key("esc", "close"),
            ],
            // THE TIMELINE STAGE. `↑/↓` steps versions and the comparison follows
            // immediately; `⇧↵` is the deliberate restore, the one key here that
            // changes the document; `esc` leaves.
            //
            // THREE CELLS, and the omissions are deliberate rather than an
            // oversight. This column is NARROW by design (a timeline beside a large
            // comparison), and its footer rides the column — so every cell is paid
            // for in width the comparison does not get. `←/→ lens` is dropped
            // because the lens strip is drawn directly above with its active label
            // marked, and `←/→` is the grammar every faceted picker already
            // teaches; `tab` is dropped because it is taught on the COMPARISON's
            // own line, which is where you need to know how to come back from.
            OverlayKind::History => vec![
                key(ARROWS_UD, "version"),
                key("\u{21E7}\u{21B5}", "restore"),
                key("esc", "close"),
            ],
            // THE VIEWS STAGE. Stepping the rows changes the prose beside them,
            // so there is nothing to commit and no `↵` cell. `esc keep editing`
            // states the outcome rather than the key's usual meaning, because
            // leaving here does NOT resolve anything — the two resolutions are
            // named by the affordance and run from the palette.
            OverlayKind::Conflict => vec![
                key(ARROWS_UD, "view"),
                HintAction {
                    glyph: "\u{21B5}",
                    label: "read",
                },
                key("esc", "keep editing"),
            ],
            // THE ONE-ROW RAIL. Nothing to step — Credits opens with the
            // content stage already focused, so a reader only lands here by
            // pressing `tab` on purpose — and nothing to commit, so no `↵`
            // cell either.
            OverlayKind::Credits => vec![key("esc", "close")],
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::TableDims
            | OverlayKind::SearchFolder => Vec::new(),
        }
    }
    /// THE DETAIL STAGE'S OWN LINE — what the footer teaches while a summoned
    /// workspace's CONTENT region holds focus. Its sibling is
    /// [`super::workspace::OverlayKind::rail_hint_actions`], the PRIMARY list's
    /// line; between them a workspace advertises the stage you are standing on.
    ///
    /// This is a WHOLE line, not [`Self::hint_actions`] with cells
    /// swapped, because the two workspace members' detail stages are not the same
    /// kind of thing. Settings' rows ARE the picker: you type to filter them and
    /// `↵` edits one, so its detail line is exactly its picker line. A COMPARISON
    /// is read-only prose: typing does not filter it, `↵` does nothing (restore
    /// is deliberately behind `⇧↵`), and `↑/↓` scrolls the transcript rather
    /// than moving a selection. Reusing the picker line there would
    /// advertise three keys that do not do what it says.
    ///
    /// Wildcard-free like its neighbours; unreachable for a kind that is not
    /// drawn as a workspace, because [`super::OverlayState::foot_hint`] gates on
    /// [`Self::workspace_shape`].
    ///
    /// NO ARM HERE SPELLS A BACK CELL. Which key goes back is a fact about the
    /// STATE (is the erase key free right now?), not about the kind, so it is
    /// appended by `foot_hint` from [`BackKey`] — the same owner the action seam
    /// reads.
    pub fn detail_hint_actions(self) -> Vec<HintAction> {
        let key = |glyph, label| HintAction { glyph, label };
        match self {
            OverlayKind::History => {
                vec![key(ARROWS_UD, "scroll"), key("\u{21E7}\u{21B5}", "restore")]
            }
            // A comparison here is read-only prose exactly as it is on a
            // timeline, minus the one key that changes the document.
            OverlayKind::Conflict => vec![key(ARROWS_UD, "scroll")],
            // Credits is the same read-only prose with nothing to restore —
            // there is no earlier version, so not even Conflict's bare
            // scroll cell needs a sibling.
            OverlayKind::Credits => vec![key(ARROWS_UD, "scroll")],
            OverlayKind::Settings
            | OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::TableDims
            | OverlayKind::SearchFolder => self.hint_actions(),
        }
    }
}

impl OverlayState {
    /// Which [`WorkspaceShape`] this card draws as, or `None` off a workspace.
    pub fn workspace_shape(&self) -> Option<WorkspaceShape> {
        self.kind.workspace_shape()
    }

    /// **THE ONE OWNER OF "WHAT GOES BACK FROM HERE"** — see [`BackKey`].
    ///
    /// `None` off a workspace, on its primary list (there is nothing behind the
    /// primary list but the editor, and `Esc` is what leaves), and during an
    /// IN-PLACE EDIT, which claims both candidate keys at once: a Range row's
    /// value field takes `⌫` for its digits and swallows `Tab` whole
    /// (`actions::overlay_nav::value_edit_intercept` runs before every other
    /// arm). A stage with nothing that goes back advertises nothing that goes
    /// back — the footer follows this answer rather than restating it.
    ///
    /// Otherwise the erase key wins unless this stage's own SEARCH FIELD is
    /// mid-word. Which stage owns the field is the shape's one fact: a
    /// `RailOverRows` workspace puts its rows — and therefore their query — in
    /// the detail stage, so `⌫` is the query's until the query is empty; a
    /// `TimelineOverComparison` workspace's query rides the primary column, so
    /// the erase key is never busy on the comparison at all.
    pub(crate) fn detail_back(&self) -> Option<BackKey> {
        let shape = self.workspace_shape()?;
        if !self.detail_focus || self.value_edit.is_some() {
            return None;
        }
        Some(match !shape.rows_are_primary() && !self.query.is_empty() {
            true => BackKey::Focus,
            false => BackKey::Erase,
        })
    }

    /// **DOES `←` COME BACK FROM THIS DETAIL STAGE — and `→` therefore mean
    /// nothing here?**
    ///
    /// The horizontal keys are the REGION SEAM's own axis, and they are a pair.
    /// On the primary list `→` goes into the content and `←` has nothing to its
    /// left; on the detail stage the mirror holds — `←` comes back to the
    /// primary list and `→` has nothing to its right. Without this, `→` opens a
    /// door that `←` cannot close, which is the one gesture a two-region
    /// keyboard owes its user.
    ///
    /// Derived from [`Self::detail_back`] rather than re-deciding: `←` comes
    /// back exactly when there IS something to come back to (a workspace's
    /// detail stage, not mid in-place-edit), and nothing on this stage already
    /// owns the axis. Exactly one thing does — a Range row, whose value rail IS
    /// `←/→` — and it is the state that NAMES why not, in the same footer cell
    /// ([`OverlayKind::range_row_actions`]) that says so.
    ///
    /// Why the categories are not that second owner: a `RailOverRows`
    /// workspace's facet strip is stood on its END as a vertical rail, so it is
    /// walked by `↑/↓` where it lives. `←/→` cycling it from the rows pane is
    /// the horizontal strip's leftover grammar — it moved a highlight in a
    /// region that did not have focus, and it took the key a user reaches for
    /// to come back.
    pub(crate) fn detail_left_returns(&self) -> bool {
        self.detail_back().is_some() && self.selected_range().is_none()
    }

    /// Move the workspace's RAIL selection by `delta` categories. The one door
    /// for `↑`/`↓` while the rail holds focus; a plain alias of the lens cycle,
    /// so the rail and `←/→` from the rows pane can never disagree about what
    /// "the current category" means.
    pub fn rail_move(&mut self, delta: isize) {
        self.cycle_lens(delta);
    }

    /// Point the rail at the category a given settings row lives under — the
    /// deep-link door (`Cmd-P` → a Settings-category row → this workspace, standing on
    /// that row under its own category). A no-op when the row's category names
    /// no lens, so an unknown category degrades to the `All` home rather than
    /// landing nowhere.
    pub fn rail_focus_category(&mut self, category: &str) {
        let Some(sc) = self.facet_scheme() else {
            return;
        };
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
