//! THE PAYLOADS a suspended journey carries: what a cancel must undo
//! ([`Audition`]), what a child writes back ([`Bind`]), and the exact position
//! a parked parent comes back to ([`Resume`], held by [`Parked`]). Split out of
//! [`super`] to keep both files inside the module ceiling; the lifecycle that
//! reads them is next door.

use super::{OverlayKind, OverlayState};
use crate::textbox::TextBox;

/// WHAT A CANCEL MUST UNDO — the live audition's original value, captured when
/// the picker opened.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Audition {
    /// Nothing is auditioning; a cancel just leaves.
    #[default]
    None,
    /// A world is live-previewing as the selection moves; a cancel restores
    /// this world index.
    Theme { original: usize },
    /// A caret look is live-previewing; a cancel restores this look — or clears
    /// the override entirely, if the look had been automatic.
    Caret {
        original: crate::caret::CaretMode,
        was_auto: bool,
    },
}

impl Audition {
    /// THE ONE OWNER of the revert, called on exactly the [`Event::Cancel`]
    /// event for whatever card is up: a directly-summoned picker and a
    /// suspended child revert identically.
    pub fn revert(self) {
        match self {
            Audition::None => {}
            Audition::Theme { original } => {
                crate::theme::set_active(original);
            }
            Audition::Caret { original, was_auto } => {
                if was_auto {
                    crate::caret::clear_override();
                } else {
                    crate::caret::set_mode(original);
                }
            }
        }
    }
}

/// WHAT A CHILD WRITES BACK. Typed, so "which config key is this folder picker
/// filling in" is a payload rather than an `Option<String>` every rebuild has
/// to remember to carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bind {
    /// The child commits a global value (a world, a caret look, a dictionary, a
    /// CJK language, a date format) or nothing at all.
    Value,
    /// The child picks a FOLDER for one config key — a Settings PATH row. The
    /// key survives every descend/ascend of the navigator because
    /// [`Journey::relevel`] replaces only the card.
    Path { key: String },
}

/// THE EXACT POSITION A PARKED SURFACE COMES BACK TO. The parent is REBUILT on
/// resume (so its value cells show what the child just committed) and then
/// re-aimed at this position — the combination the old breadcrumb pop could not
/// express, which is why picking a caret look from the eleventh Settings row
/// used to drop you back on the first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resume {
    query: String,
    /// The CORPUS index of the highlighted row, not its filtered position: the
    /// rebuilt parent's row ORDER can differ, and the row is what you were on.
    selected_corpus: Option<usize>,
    scroll: usize,
    lens: usize,
    detail_focus: bool,
    detail_scroll: usize,
}

impl Resume {
    pub(super) fn of(card: &OverlayState) -> Self {
        Self {
            query: card.query.text().to_string(),
            selected_corpus: card.selected_corpus_index(),
            scroll: card.scroll,
            lens: card.facet_lens,
            detail_focus: card.detail_focus,
            detail_scroll: card.diff_scroll,
        }
    }

    /// Nothing to come back to. The one caller is [`Journey::attribute_launch`],
    /// which learns about a launcher AFTER it closed (the palette's re-dispatch
    /// seam), so there was never a position to snapshot.
    pub(super) fn fresh() -> Self {
        Self::default()
    }

    /// Re-aim a freshly rebuilt surface at this position. Lens first (it
    /// regroups), then the query (it refilters), then the row, then the window,
    /// then the detail stage — each step reading the state the previous left.
    pub(super) fn restore_into(&self, card: &mut OverlayState) {
        card.set_facet_lens(self.lens);
        card.query = TextBox::seeded(&self.query);
        card.refilter();
        if let Some(ci) = self.selected_corpus
            && let Some(pos) = card.items.iter().position(|&i| i == ci)
        {
            card.selected = pos;
        }
        card.scroll = self.scroll;
        card.scroll_to_selected();
        card.detail_focus = self.detail_focus;
        card.diff_scroll = self.detail_scroll;
    }

    /// TEST-ONLY: the recorded corpus row, for the restoration law.
    #[cfg(test)]
    pub fn selected_corpus(&self) -> Option<usize> {
        self.selected_corpus
    }

    /// TEST-ONLY: the recorded query text.
    #[cfg(test)]
    pub fn query(&self) -> &str {
        &self.query
    }
}

/// A PARKED PARENT: which surface to come back to, and where in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parked {
    pub(super) kind: OverlayKind,
    pub(super) resume: Resume,
}

impl Parked {
    /// Which surface a resume rebuilds — the sidecar's `overlay.return_to`.
    pub fn kind(&self) -> OverlayKind {
        self.kind
    }
}
