//! WHAT READ-ONLY PROSE A COMPARISON PANE SHOWS, AND WHERE IT CAME FROM.
//!
//! A `WorkspaceShape::TimelineOverComparison` workspace puts a list in its
//! primary column and read-only prose in its content region. That prose is not
//! drawn by a pane renderer of its own — awl has exactly ONE prose renderer, and
//! item 116b relocated it (`render/chrome/comparison.rs`). What was still missing
//! was a typed answer to *which* prose, asked in a way that is not about Version
//! History.
//!
//! # Why this is not `selected_history_id()`
//!
//! Before item 116d the comparison's content came from `App::history_preview_text`,
//! keyed on `OverlayState::selected_history_id()` and cached under that bare id.
//! Two things are wrong with that shape the moment a SECOND consumer exists:
//!
//!   * it is History-specific — an overlay that is not a timeline has no "history
//!     id" to be keyed on, so a second consumer needs a second mechanism, which
//!     is a second renderer by another name;
//!   * it is UNTYPED — one opaque string per subject, so a surface that shows
//!     several read-only views OF THE SAME SUBJECT (queue item 204's external-file
//!     conflict: *Differences* / *Your version* / *Version on disk*, one at a
//!     time) cannot express them without colliding in the cache.
//!
//! [`ComparisonRequest`] is both: a [`ComparisonView`] naming which read-only view
//! is wanted, and an opaque per-surface SUBJECT naming what of. Its
//! [`ComparisonRequest::cache_key`] folds both in, so two views of one subject are
//! two entries rather than one served twice.
//!
//! # The producer stays where the data is
//!
//! This module states the REQUEST, not the answer: resolving a request needs the
//! buffer, its path and the store, none of which the overlay content model may
//! reach. `App::comparison_transcript` is the one resolver, and the headless
//! capture folds through the same request so live and `--keys` replay cannot
//! disagree about what a comparison shows.

use super::{OverlayKind, OverlayState};

/// WHICH READ-ONLY VIEW of a subject the comparison pane shows.
///
/// The variants are named for what the READER sees, not for where the text came
/// from, because the same view is produced differently per surface: *Differences*
/// is a version diff on a timeline and a merge diff on a conflict.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComparisonView {
    /// The writer's DIFF — a marked-up manuscript of what changed between the
    /// subject and the current buffer ([`crate::prosediff`]). Version History's
    /// only view, and the one a conflict opens on.
    Differences,
    /// The user's OWN text for the subject, shown whole and unmarked. No consumer
    /// yet; queue item 204's "Your version".
    #[allow(dead_code)]
    Mine,
    /// The OTHER text for the subject, shown whole and unmarked. No consumer yet;
    /// queue item 204's "Version on disk".
    #[allow(dead_code)]
    Theirs,
}

impl ComparisonView {
    /// A stable, machine-readable tag — the cache key's first component and the
    /// sidecar's spelling. Not the user-facing label: that belongs to the surface
    /// that shows it, because "Differences" reads differently on a timeline and
    /// on a conflict.
    pub fn tag(self) -> &'static str {
        match self {
            ComparisonView::Differences => "diff",
            ComparisonView::Mine => "mine",
            ComparisonView::Theirs => "theirs",
        }
    }
}

/// ONE REQUEST for read-only comparison prose: which view, of what.
///
/// `subject` is opaque to everything but the surface that produced it and the
/// resolver that answers it — a history restore id today, a conflict's own handle
/// for item 204. Nothing between them parses it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ComparisonRequest {
    pub view: ComparisonView,
    pub subject: String,
}

impl ComparisonRequest {
    /// THE CACHE KEY — view AND subject, never the subject alone.
    ///
    /// This is the whole reason the type exists: a surface offering three views of
    /// one subject keyed on the bare subject would serve whichever view was
    /// rendered first for all three, which is the cache-key discipline's own
    /// failure mode (a key that does not name everything the value depends on) and
    /// exactly what blocked item 204 against the old shape.
    pub fn cache_key(&self) -> String {
        format!("{}:{}", self.view.tag(), self.subject)
    }
}

impl OverlayState {
    /// WHAT READ-ONLY PROSE THIS CARD'S COMPARISON REGION IS ASKING FOR, or `None`
    /// when it has nothing to show — a kind with no comparison at all, or a
    /// timeline standing on its empty-state row.
    ///
    /// `None` is a real product fact, not just an absence: it is what makes the
    /// focus transfer into a comparison DECLINE on an empty history rather than
    /// hand the keyboard to a blank region, and what makes `Enter` there fall
    /// through to the ordinary close.
    ///
    /// Wildcard-free: a new picker kind must say whether it shows read-only prose.
    pub fn comparison_request(&self) -> Option<ComparisonRequest> {
        match self.kind {
            OverlayKind::History => Some(ComparisonRequest {
                view: ComparisonView::Differences,
                subject: self.selected_history_id()?.to_string(),
            }),
            OverlayKind::Settings
            | OverlayKind::Goto
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
            | OverlayKind::KeepName
            | OverlayKind::Context => None,
        }
    }
}
