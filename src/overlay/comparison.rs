//! WHAT READ-ONLY PROSE A COMPARISON PANE SHOWS, AND WHERE IT CAME FROM.
//!
//! A `WorkspaceShape::TimelineOverComparison` workspace puts a list in its
//! primary column and read-only prose in its content region. That prose is not
//! drawn by a pane renderer of its own — awl has exactly ONE prose renderer, and
//! the one there is has been relocated instead (`render/chrome/comparison.rs`).
//! What that leaves is a typed answer to *which* prose, asked in a way that is not
//! about Version History.
//!
//! # Why this is not `selected_history_id()`
//!
//! The obvious shape is a History-shaped one: ask the card for its selected
//! history id and cache the transcript under that bare id. Two things go wrong
//! with it the moment a SECOND consumer exists:
//!
//!   * it is History-specific — an overlay that is not a timeline has no "history
//!     id" to be keyed on, so a second consumer needs a second mechanism, which
//!     is a second renderer by another name;
//!   * it is UNTYPED — one opaque string per subject, so a surface that shows
//!     several read-only views OF THE SAME SUBJECT — an external-file conflict
//!     offering *Differences* / *Your version* / *Version on disk* one at a time
//!     — cannot express them without colliding in the cache.
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
    /// yet; the "Your version" a file conflict would offer.
    #[allow(dead_code)]
    Mine,
    /// The OTHER text for the subject, shown whole and unmarked. No consumer yet;
    /// the "Version on disk" a file conflict would offer.
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
/// resolver that answers it — a history restore id today, a conflict's own
/// handle tomorrow. Nothing between them parses it.
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
    /// failure mode: a key that does not name everything the value depends on.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// THE ROSTER: exactly one kind asks for read-only prose today, and it asks
    /// for the DIFF. Wildcard-free at the source; this pins what the roster
    /// currently reads so a kind that quietly grows a comparison is noticed.
    #[test]
    fn exactly_the_timeline_asks_for_read_only_prose() {
        let _g = crate::testlock::serial();
        let mut asked = 0usize;
        for kind in OverlayKind::ALL {
            let card = OverlayState::new(kind, vec!["a row".into()], vec![], vec![]);
            match kind {
                OverlayKind::History => {
                    // A generic card carries no History row meta, so its subject
                    // is absent and it correctly asks for nothing — which is the
                    // empty-timeline case, and the reason the focus transfer can
                    // decline. The populated case is covered below.
                    assert!(card.comparison_request().is_none());
                    asked += 1;
                }
                _ => assert!(
                    card.comparison_request().is_none(),
                    "{kind:?} is not a comparison surface and must ask for nothing"
                ),
            }
        }
        assert_eq!(asked, 1, "the roster must contain the timeline");

        let rows = vec![crate::history::TimelineRow {
            when: "2 hr ago".into(),
            which: "edited \"Title\"".into(),
            counts: "+1 −1".into(),
            id: "1700000000000".into(),
            timestamp: 1_700_000_000_000,
            pinned: false,
            name: None,
        }];
        let card = OverlayState::new_history(rows, None, None);
        let req = card
            .comparison_request()
            .expect("a timeline standing on a version asks for its diff");
        assert_eq!(req.view, ComparisonView::Differences);
        assert_eq!(req.subject, "1700000000000");
    }

    /// THE CACHE KEY NAMES THE VIEW, NOT ONLY THE SUBJECT.
    ///
    /// This is the whole reason the request is typed, and it is the one claim a
    /// second consumer's correctness rests on: a surface offering several
    /// read-only views of ONE subject must get several cache entries. Keyed on the
    /// bare subject, all three views would be served whichever was rendered first
    /// — the cache-key discipline's own failure mode, and silent.
    #[test]
    fn two_views_of_one_subject_are_two_cache_entries() {
        let subject = "the-same-file".to_string();
        let keys: Vec<String> = [
            ComparisonView::Differences,
            ComparisonView::Mine,
            ComparisonView::Theirs,
        ]
        .into_iter()
        .map(|view| {
            ComparisonRequest {
                view,
                subject: subject.clone(),
            }
            .cache_key()
        })
        .collect();
        let unique: std::collections::BTreeSet<&String> = keys.iter().collect();
        assert_eq!(
            unique.len(),
            keys.len(),
            "three views of one subject collided in the cache: {keys:?}"
        );
        for key in &keys {
            assert!(
                key.ends_with(&subject),
                "the key must still carry the subject whole, so a producer can read it \
                 back: {key}"
            );
        }
        // …and two SUBJECTS under one view stay distinct too, which is the
        // property the old bare-id key already had and this must not lose.
        assert_ne!(
            ComparisonRequest {
                view: ComparisonView::Differences,
                subject: "a".into()
            }
            .cache_key(),
            ComparisonRequest {
                view: ComparisonView::Differences,
                subject: "b".into()
            }
            .cache_key()
        );
    }

    /// Every view's tag is distinct and stable — the cache key and the producer's
    /// own dispatch both read it, so two views sharing a tag would collide at
    /// both ends at once.
    #[test]
    fn every_view_tag_is_distinct() {
        let tags: Vec<&str> = [
            ComparisonView::Differences,
            ComparisonView::Mine,
            ComparisonView::Theirs,
        ]
        .into_iter()
        .map(ComparisonView::tag)
        .collect();
        let unique: std::collections::BTreeSet<&&str> = tags.iter().collect();
        assert_eq!(unique.len(), tags.len(), "view tags collided: {tags:?}");
        assert!(tags.iter().all(|t| !t.is_empty()));
    }
}
