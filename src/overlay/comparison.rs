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
//! reach. [`crate::comparison::prose_for`] is the one DISPATCH — one resolver,
//! one producer per surface — and both consumers (the live App's
//! `App::comparison_transcript` and the headless capture fold) go through it, so
//! live and `--keys` replay cannot disagree about what a comparison shows.

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
    /// The user's OWN text for the subject, shown whole and unmarked — the
    /// conflict workspace's "Your version".
    Mine,
    /// The OTHER text for the subject, shown whole and unmarked — the conflict
    /// workspace's "Version on disk".
    Theirs,
    /// A STANDALONE embedded document, shown whole and unmarked, belonging to
    /// no roster of alternatives — Credits' one and only view. Deliberately
    /// OUT of [`Self::ALL`]: that array is Conflict's row-index roster, and a
    /// fourth entry there would silently offer Conflict a view it has no row
    /// for.
    Document,
}

impl ComparisonView {
    /// The three CONFLICT views, in the order a reader wants them: what
    /// changed, then each whole version. [`CONFLICT_ROWS`] is this roster's
    /// user-facing spelling and a law pins the two in lockstep. `Document` is
    /// deliberately absent — it names no row of Conflict's.
    pub const ALL: [ComparisonView; 3] = [
        ComparisonView::Differences,
        ComparisonView::Mine,
        ComparisonView::Theirs,
    ];

    /// A stable, machine-readable tag — the cache key's first component and the
    /// sidecar's spelling. Not the user-facing label: that belongs to the surface
    /// that shows it, because "Differences" reads differently on a timeline and
    /// on a conflict.
    pub fn tag(self) -> &'static str {
        match self {
            ComparisonView::Differences => "diff",
            ComparisonView::Mine => "mine",
            ComparisonView::Theirs => "theirs",
            ComparisonView::Document => "document",
        }
    }
}

/// THE CONFLICT WORKSPACE'S ROW LABELS, parallel to [`ComparisonView::ALL`].
///
/// The user-facing names, which are deliberately NOT the machine tags: a tag is
/// a cache key and a sidecar spelling, a label is what a reader is asked to
/// choose between. "Your version" and "Version on disk" name the two
/// manuscripts by whose they are, because that is the only distinction that
/// matters while deciding.
pub const CONFLICT_ROWS: [&str; 3] = ["Differences", "Your version", "Version on disk"];

/// THE CONFLICT WORKSPACE'S SUBJECT: which file, and what the disk said.
///
/// It rides the CARD because the producer that reads it is reached from both the
/// live App and the headless capture fold, and the fold is handed only the
/// overlay and the buffer — the App's own latch is not in its reach. Putting the
/// payload on the card is what lets live and capture resolve one request through
/// one producer.
///
/// `theirs` is `None` for a DELETED file — there is no disk version to read, and
/// the "Version on disk" view says so rather than showing an empty document that
/// would read as "the file is empty".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictSubject {
    pub path: std::path::PathBuf,
    pub theirs: Option<String>,
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
    /// THE CONFLICT WORKSPACE: three read-only views of ONE subject, shown one
    /// at a time, in the order a reader wants them — what changed, then each
    /// whole version.
    ///
    /// The row order IS the [`ComparisonView`] order
    /// ([`Self::comparison_request`] reads the selected CORPUS index, never the
    /// filtered position), so the two cannot drift; the row labels are the
    /// user-facing names and the view tags are the machine ones, deliberately
    /// separate because "Differences" reads differently on a timeline.
    pub fn new_conflict(path: std::path::PathBuf, theirs: Option<String>) -> Self {
        let mut s = Self::new(
            OverlayKind::Conflict,
            CONFLICT_ROWS.iter().map(|r| r.to_string()).collect(),
            Vec::new(),
            Vec::new(),
        );
        s.conflict = Some(ConflictSubject { path, theirs });
        s
    }

    /// THE CREDITS VIEWER: one fixed row naming the document, standing in for
    /// the primary list a `TimelineOverComparison` workspace otherwise needs —
    /// there is nothing to navigate, so the "list" is the title.
    ///
    /// Opens on the PRIMARY list, `detail_focus: false`, exactly like
    /// History/Conflict — `detail_focus` is lifecycle state
    /// (`overlay/journey/`'s own law: it may be written only there, never by a
    /// constructor), so landing straight on the content stage is the CALLER's
    /// job, the same `toggle_detail()` deep link `Action::CompareVersion`
    /// already uses to skip History's primary list.
    pub fn new_credits() -> Self {
        Self::new(
            OverlayKind::Credits,
            vec![OverlayKind::Credits.title().to_string()],
            Vec::new(),
            Vec::new(),
        )
    }

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
            // THE SECOND CONSUMER the typed request was built for: three views of
            // ONE subject. The view comes from the selected CORPUS index — the
            // row's own identity — never from its filtered position, so a typed
            // query that hides a row cannot silently re-point the remaining ones
            // at the wrong view. The subject is the file's path: stable across
            // all three views, so `cache_key`'s view component is what keeps them
            // apart, which is exactly the collision that type exists to prevent.
            OverlayKind::Conflict => {
                let subject = self.conflict.as_ref()?.path.to_string_lossy().to_string();
                let view = *ComparisonView::ALL.get(self.selected_corpus_index()?)?;
                Some(ComparisonRequest { view, subject })
            }
            // CREDITS: always Some — a THIRD consumer, and the simplest one.
            // One fixed subject, one fixed view, no selection to read: unlike
            // History/Conflict there is no empty state and no row that could
            // fail to resolve, so this never returns `None`.
            OverlayKind::Credits => Some(ComparisonRequest {
                view: ComparisonView::Document,
                subject: "credits".to_string(),
            }),
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
            | OverlayKind::SearchFolder
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::TableDims => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// THE ROSTER: exactly THREE kinds ask for read-only prose — the timeline,
    /// the external-change conflict, and Credits. Wildcard-free at the
    /// source; this pins what the roster currently reads so a kind that
    /// quietly grows a comparison is noticed.
    ///
    /// The number moved from one to two (116d, a second consumer) and now to
    /// three (Credits, the first STATIC one). What has NOT moved is the shape
    /// of the membership test — each member is named, and a bare card of
    /// every other kind must still ask for nothing.
    #[test]
    fn exactly_the_timeline_the_conflict_and_credits_ask_for_read_only_prose() {
        let _g = crate::testlock::serial();
        let mut asked = Vec::new();
        for kind in OverlayKind::ALL {
            let card = OverlayState::new(kind, vec!["a row".into()], vec![], vec![]);
            match kind {
                // A generic card carries no History row meta and no conflict
                // payload, so its subject is absent in both cases and it
                // correctly asks for nothing — the empty-timeline case, and the
                // reason the focus transfer can decline. Both populated cases are
                // covered below.
                OverlayKind::History | OverlayKind::Conflict => {
                    assert!(card.comparison_request().is_none());
                    asked.push(kind);
                }
                // CREDITS asks unconditionally — a bare generic card is already
                // the whole story, unlike History/Conflict's payload-gated
                // empty state.
                OverlayKind::Credits => {
                    assert!(card.comparison_request().is_some());
                    asked.push(kind);
                }
                _ => assert!(
                    card.comparison_request().is_none(),
                    "{kind:?} is not a comparison surface and must ask for nothing"
                ),
            }
        }
        assert_eq!(
            asked,
            vec![
                OverlayKind::History,
                OverlayKind::Conflict,
                OverlayKind::Credits
            ],
            "the comparison roster is the timeline, the conflict and credits"
        );

        // THE CONFLICT, POPULATED: it asks, on its own subject, for the view its
        // selected row names.
        let conflict = OverlayState::new_conflict(
            std::path::PathBuf::from("/notes/heron.md"),
            Some("disk".into()),
        );
        let req = conflict
            .comparison_request()
            .expect("a conflict standing on its first row asks for the differences");
        assert_eq!(req.view, ComparisonView::Differences);
        assert_eq!(req.subject, "/notes/heron.md");

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

        // CREDITS: the real constructor asks for exactly the fixed view,
        // unconditionally — the deep link to the content stage is the
        // ACTION's job (`Action::OpenCredits`'s own `toggle_detail`), not
        // this constructor's.
        let credits = OverlayState::new_credits();
        let req = credits
            .comparison_request()
            .expect("credits always has something to show");
        assert_eq!(req.view, ComparisonView::Document);
        assert_eq!(req.subject, "credits");
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
