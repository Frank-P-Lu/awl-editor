//! THE SUMMONED-UI LIFECYCLE (`Journey`) — one closed set of
//! states, one closed set of events, and one table ([`table::landing_of`])
//! saying where every pair lands.
//!
//! The summoned surfaces have a precedence LADDER
//! (`app::workspace::Layer`). A ladder answers "who holds attention right now".
//! It cannot answer the questions a SUSTAINED surface asks:
//!
//!   * Esc in a picker closes it — but Esc in a workspace's DETAIL stage must
//!     hand focus back to the list, not close the workspace.
//!   * A workspace row that opens a child must come BACK to the exact row,
//!     query and lens it left from.
//!   * A child that AUDITIONS something live (a world, a caret look) must
//!     revert on cancel and keep on commit — and whether a commit returns to
//!     the parent or lands in the editor depends on what the parent WAS.
//!
//! Before this module those rules were `OverlayState::return_to` (a bare
//! `Option<OverlayKind>` breadcrumb that re-summoned the parent FRESH, so the
//! position was silently lost), `setting_path_key` (a `String` carried forward
//! by hand at exactly the two rebuild seams somebody remembered), `detail_focus`
//! (a bool with its own exceptional `Esc` arm), and `original_theme` /
//! `original_caret` / `original_caret_was_auto` (three loose fields whose
//! inconsistent combinations were representable). Six fields, four hand-written
//! rules, and no single place that said what Esc does.
//!
//! # The four rungs
//!
//! ```text
//!   Audition     a CHILD over a PARKED parent — the child owns the keyboard,
//!                the parent's exact return position is held
//!     ▲
//!   Workspace    a SUSTAINED summoned workspace (Settings, Version History):
//!                you stay in it and navigate WITHIN it
//!     ▲
//!   Contextual   a BRIEF contextual overlay — summoned, used, gone on pick
//!     ▲
//!   Editor       nothing summoned
//! ```
//!
//! # Why the lifecycle lives in the shared core
//!
//! `WorkspaceState` is
//! `pub(in crate::app)`, and the headless `--keys` replay
//! (`main/run.rs::ReplaySession`) cannot see it — yet every transition here is
//! reached through `actions::apply_transition`, which both flows share. A
//! lifecycle owned inside `crate::app` would have forced the replay to keep a
//! second copy: the exact defect this item exists to close. So the lifecycle
//! type lives in the core both flows share, and `WorkspaceState` owns the one
//! live instance and derives its ladder from [`Journey::rung`]. One owner of
//! the lifecycle, one owner of the ladder, and the ladder READS the lifecycle
//! rather than re-deriving it.
//!
//! # Scope
//!
//! Shared lifecycle machinery for Settings and Version History — not a route
//! stack: the depth is exactly one by construction,
//! the payloads are typed ([`Bind`], [`Audition`]) rather than a string map,
//! and a content supplier hands over ROWS while navigation preservation stays
//! here.

use super::{AcceptDisposition, OverlayKind, OverlayState};

mod parked;
mod table;
#[cfg(test)]
mod tests;

pub use parked::{Audition, Bind, Parked, Resume};
pub use table::{Beneath, Event, Landing, State, Surface, landing_of};

/// Which LADDER RUNG a journey occupies — the one fact
/// `app::workspace::WorkspaceState::layer` reads out of the lifecycle, so the
/// ladder never re-derives the phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rung {
    /// No card is summoned.
    Nothing,
    /// A MODAL card owns the keyboard: a brief overlay, or a child audition.
    Modal,
    /// A SUSTAINED workspace owns the keyboard.
    Sustained,
}

impl State {
    /// Project this state onto the ladder. Wildcard-free: a new surface must be
    /// placed on a rung to compile.
    pub fn rung(self) -> Rung {
        match self {
            State::Editor => Rung::Nothing,
            State::Summoned {
                surface: Surface::Contextual,
                ..
            } => Rung::Modal,
            State::Summoned {
                surface: Surface::Workspace,
                ..
            } => Rung::Sustained,
            State::Summoned {
                surface: Surface::WorkspaceDetail,
                ..
            } => Rung::Sustained,
        }
    }
}

impl OverlayKind {
    /// Is this kind a SUSTAINED summoned workspace — a place you stay in and
    /// navigate within — rather than a brief contextual overlay?
    ///
    /// Deliberately scoped to the two surfaces the shared workspace exists for:
    /// Settings and Version History. It lives beside the
    /// lifecycle it drives rather than in `overlay/kind.rs`, so the
    /// classification and the table that consumes it are read together.
    ///
    /// It also subsumes the retired `retains_value_pick_child()`: a value-pick
    /// child returns to its parent exactly when the parent is a place. One
    /// predicate, not two that can drift.
    pub fn sustained(self) -> bool {
        match self {
            OverlayKind::Settings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Credits => true,
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
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::TableDims => false,
        }
    }
}

/// The journey's one private fact. Every invalid combination the six loose
/// fields could express is gone by construction: a parked parent cannot exist
/// without a child, a child cannot exist without its parent, and a [`Bind`]
/// cannot sit on a surface with nothing to write back to.
#[derive(Default)]
enum Stage {
    #[default]
    Editor,
    Card(OverlayState),
    Suspended {
        child: OverlayState,
        parent: Parked,
        bind: Bind,
    },
}

/// THE SUMMONED-UI LIFECYCLE. The field is private: every write below is a
/// named transition that consults [`landing_of`], so no caller can put the
/// journey somewhere the table does not allow.
#[derive(Default)]
pub struct Journey {
    stage: Stage,
}

impl Journey {
    /// The card holding attention — the summoned surface's CONTENT, whose own
    /// rules (rows, selection, query, notice) belong to [`OverlayState`].
    pub fn card(&self) -> Option<&OverlayState> {
        match &self.stage {
            Stage::Editor => None,
            Stage::Card(card) => Some(card),
            Stage::Suspended { child, .. } => Some(child),
        }
    }

    /// Mutate the card's CONTENT. Never the lifecycle: the rung is a function
    /// of the stage and the card's kind, neither of which this hands out.
    pub fn card_mut(&mut self) -> Option<&mut OverlayState> {
        match &mut self.stage {
            Stage::Editor => None,
            Stage::Card(card) => Some(card),
            Stage::Suspended { child, .. } => Some(child),
        }
    }

    /// The lifecycle state — the table's row.
    pub fn state(&self) -> State {
        let (card, beneath) = match &self.stage {
            Stage::Editor => return State::Editor,
            Stage::Card(card) => (card, Beneath::Editor),
            Stage::Suspended { child, parent, .. } => (
                child,
                match parent.kind.sustained() {
                    true => Beneath::Workspace,
                    false => Beneath::Launcher,
                },
            ),
        };
        let surface = match (card.kind.sustained(), card.detail_focus) {
            (true, true) => Surface::WorkspaceDetail,
            (true, false) => Surface::Workspace,
            // A brief overlay has no detail stage, so the focus bit cannot
            // mean anything there — spelled out rather than defaulted.
            (false, true) => Surface::Contextual,
            (false, false) => Surface::Contextual,
        };
        State::Summoned { surface, beneath }
    }

    /// The ladder rung, for `WorkspaceState::layer`.
    pub fn rung(&self) -> Rung {
        self.state().rung()
    }

    /// The parked parent, if a child is up.
    pub fn parked(&self) -> Option<&Parked> {
        match &self.stage {
            Stage::Editor | Stage::Card(_) => None,
            Stage::Suspended { parent, .. } => Some(parent),
        }
    }

    /// What the child writes back, if a child is up.
    pub fn bind(&self) -> Option<&Bind> {
        match &self.stage {
            Stage::Editor | Stage::Card(_) => None,
            Stage::Suspended { bind, .. } => Some(bind),
        }
    }

    /// **THE ONE OWNER of the footer sentence**, for every caller that has a
    /// `Journey` in hand rather than a bare card. [`OverlayState::foot_hint`]
    /// cannot answer this alone: [`OverlayKind::Project`] draws as two
    /// different features from the SAME card shape — the flat switch-project
    /// picker (no bind, or `Bind::Value`) and the Settings folder-VALUE picker
    /// (`Bind::Path`) — and only the fact parked here, on the journey, says
    /// which one a given card is. A caller that reaches the card directly
    /// (most of the test suite, which builds a standalone `OverlayState` with
    /// no `Bind` at all) gets the flat answer, which is the correct default:
    /// a plainly-summoned card is never mid-descend from Settings.
    pub fn foot_hint(&self) -> String {
        match self.card() {
            Some(card) => card.foot_hint_scoped(self.bind()),
            None => String::new(),
        }
    }

    // ─── TRANSITIONS ─────────────────────────────────────────────────────

    /// SUMMON a surface from the editor. Whatever was up is replaced outright
    /// and nothing is parked — a summon is not a descend. `None` (a builder
    /// that found nothing to show) leaves the editor, which is what the old
    /// `*ctx.overlay = make_overlay(kind)` always did.
    pub fn enter(&mut self, card: Option<OverlayState>) {
        self.stage = match card {
            Some(card) => Stage::Card(card),
            None => Stage::Editor,
        };
    }

    /// REPLACE the card at the SAME rung, keeping any parked parent and bind —
    /// a folder navigator descending or ascending a level. This retires the
    /// hand-carried breadcrumb snapshot/re-apply pair, which had to be wired at
    /// each rebuild seam and silently dropped the config key at any seam
    /// nobody remembered.
    ///
    /// A LEVEL IS REBUILT FROM A DIRECTORY, so the level supplier can only know
    /// what is on disk — anything the SUMMON decided (which export format this
    /// navigator is finding a folder for) would be lost at every descend if this
    /// did not carry it forward. [`OverlayState::carry_level_payload_from`] is
    /// the one owner of that carry, which is what keeps a `Bind`-shaped fact off
    /// each caller's memory.
    pub fn relevel(&mut self, mut next: OverlayState) {
        self.stage = match std::mem::take(&mut self.stage) {
            Stage::Editor => Stage::Card(next),
            Stage::Card(prev) => {
                next.carry_level_payload_from(&prev);
                Stage::Card(next)
            }
            Stage::Suspended {
                child,
                parent,
                bind,
            } => {
                next.carry_level_payload_from(&child);
                Stage::Suspended {
                    child: next,
                    parent,
                    bind,
                }
            }
        };
    }

    /// OPEN A CHILD over this surface, parking it at its exact position.
    /// Consults the table, so a descend from the editor is a calm no-op.
    ///
    /// SINGLE LEVEL: a suspend replaces whatever was already parked, so the
    /// depth is one no matter how deep the chain of rows — the pre-existing
    /// rule (no N-deep stack, no A→B→A loop), now enforced by the type rather
    /// than by every caller overwriting one breadcrumb field.
    pub fn descend(&mut self, child: OverlayState, bind: Bind) -> Landing {
        let landing = landing_of(self.state(), Event::Descend);
        if landing != Landing::Suspend {
            return landing;
        }
        self.stage = match std::mem::take(&mut self.stage) {
            // Unreachable: `Suspend` requires a card. Total without a panic.
            Stage::Editor => Stage::Editor,
            Stage::Card(parent) | Stage::Suspended { child: parent, .. } => Stage::Suspended {
                parent: Parked {
                    kind: parent.kind,
                    resume: Resume::of(&parent),
                },
                child,
                bind,
            },
        };
        landing
    }

    /// ATTRIBUTE a launch to a surface that has ALREADY closed — the command
    /// palette's re-dispatch seam, where the palette runs a command, closes,
    /// and only then does the command open its own picker. There is no position
    /// left to snapshot, so the parent parks [`Resume::fresh`]; a real
    /// [`Self::descend`] from a still-open surface preserves its position. A
    /// no-op unless a card is up with nothing already parked — a child's own
    /// parent wins.
    pub fn attribute_launch(&mut self, parent: Option<OverlayKind>) {
        let Some(parent) = parent else { return };
        self.stage = match std::mem::take(&mut self.stage) {
            Stage::Card(child) => Stage::Suspended {
                child,
                parent: Parked {
                    kind: parent,
                    resume: Resume::fresh(),
                },
                bind: Bind::Value,
            },
            unchanged => unchanged,
        };
    }

    /// ADVANCE the lifecycle by one event — the single door. It reads
    /// [`landing_of`], reverts the audition on exactly [`Event::Cancel`], and
    /// performs the landing. `rebuild` supplies a fresh parent on a resume: the
    /// core cannot read the filesystem or the config, so the caller injects it,
    /// exactly as `ActionCtx::make_overlay` already does.
    pub fn advance(
        &mut self,
        event: Event,
        rebuild: &mut dyn FnMut(OverlayKind) -> Option<OverlayState>,
    ) -> Landing {
        let landing = landing_of(self.state(), event);
        if event == Event::Cancel
            && let Some(card) = self.card()
        {
            card.audition.revert();
        }
        match landing {
            Landing::Stay => {}
            Landing::Editor => self.stage = Stage::Editor,
            Landing::Primary => {
                if let Some(card) = self.card_mut() {
                    card.detail_focus = false;
                }
            }
            Landing::Detail => {
                if let Some(card) = self.card_mut() {
                    card.detail_focus = true;
                }
            }
            Landing::Resume => self.perform_resume(rebuild),
            // Only `descend` can suspend, because only it carries the child.
            Landing::Suspend => {}
        }
        landing
    }

    /// Esc / C-g / Back — the ONE door for every cancel gesture.
    pub fn cancel(
        &mut self,
        rebuild: &mut dyn FnMut(OverlayKind) -> Option<OverlayState>,
    ) -> Landing {
        self.advance(Event::Cancel, rebuild)
    }

    /// An ACCEPT, dispatched by the highlighted kind's declared disposition —
    /// the one place the accept classification meets the lifecycle.
    pub fn accept(
        &mut self,
        disposition: AcceptDisposition,
        rebuild: &mut dyn FnMut(OverlayKind) -> Option<OverlayState>,
    ) -> Landing {
        let event = match disposition {
            AcceptDisposition::Navigate => Event::AcceptNavigate,
            AcceptDisposition::ValuePick => Event::AcceptValue,
            AcceptDisposition::StayOpen => Event::AcceptStayOpen,
        };
        self.advance(event, rebuild)
    }

    /// A row that flips a setting IN PLACE: a launcher's errand is done, a
    /// workspace keeps configuring.
    pub fn toggled(
        &mut self,
        rebuild: &mut dyn FnMut(OverlayKind) -> Option<OverlayState>,
    ) -> Landing {
        self.advance(Event::Toggle, rebuild)
    }

    /// Move focus between a workspace's primary list and its detail stage.
    pub fn toggle_detail(&mut self) -> Landing {
        self.advance(Event::ToggleDetail, &mut |_| None)
    }

    /// GO SOMEWHERE: the whole journey ends, parked parent included. You asked
    /// to land in the document, so you land there.
    pub fn navigate_away(&mut self) -> Landing {
        self.advance(Event::AcceptNavigate, &mut |_| None)
    }

    /// EVERYTHING SUMMONED GOES — the menu bar took over, or the buffer swapped
    /// underneath and a picker's row indices no longer mean anything.
    pub fn dismiss(&mut self) -> Landing {
        self.advance(Event::Dismiss, &mut |_| None)
    }

    fn perform_resume(&mut self, rebuild: &mut dyn FnMut(OverlayKind) -> Option<OverlayState>) {
        let Stage::Suspended { parent, .. } = std::mem::take(&mut self.stage) else {
            return;
        };
        self.stage = match rebuild(parent.kind) {
            Some(mut card) => {
                parent.resume.restore_into(&mut card);
                Stage::Card(card)
            }
            // A parent that cannot be rebuilt is no reason to trap the
            // keyboard: fall to the editor, as the breadcrumb pop's `None`
            // arm always did.
            None => Stage::Editor,
        };
    }

    /// TEST-ONLY: install a card directly, so a test can drive a picker's own
    /// behaviour without replaying the Action that summons it.
    #[cfg(test)]
    pub fn install_for_test(&mut self, card: OverlayState) {
        self.stage = Stage::Card(card);
    }

    /// TEST-ONLY: a journey already standing on `card` (or on the editor).
    #[cfg(test)]
    pub fn seeded(card: Option<OverlayState>) -> Self {
        let mut journey = Self::default();
        journey.enter(card);
        journey
    }

    /// TEST-ONLY: the parked parent's recorded position, for the
    /// position-restoration law.
    #[cfg(test)]
    pub fn parked_resume(&self) -> Option<&Resume> {
        self.parked().map(|p| &p.resume)
    }

    /// TEST-ONLY: which surface is parked beneath the card (the sidecar's
    /// `overlay.return_to`, as a kind).
    #[cfg(test)]
    pub fn parked_kind(&self) -> Option<OverlayKind> {
        self.parked().map(|p| p.kind)
    }

    /// TEST-ONLY: the config key a folder-picking child is filling in.
    #[cfg(test)]
    pub fn path_key(&self) -> Option<&str> {
        match self.bind() {
            Some(Bind::Path { key }) => Some(key.as_str()),
            Some(Bind::Value) | None => None,
        }
    }
}
