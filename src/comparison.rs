//! src/comparison.rs — **THE ONE DISPATCH** from a typed
//! [`crate::overlay::ComparisonRequest`] to the producer that answers it.
//!
//! [`crate::overlay::ComparisonRequest`] names a VIEW and an opaque SUBJECT, so
//! a surface offering several read-only views of one thing can ask for them
//! without colliding in the cache. That generality is only half a seam: both
//! resolvers — the live App's (`App::comparison_transcript`) and the headless
//! capture fold's (`main/run/capture_fold.rs`) — need somewhere kind-neutral to
//! send a request, or the answer is hard-wired to whichever surface came first.
//!
//! This module is that dispatch and nothing else. It owns no prose of its own:
//! History's producer stays in `history/picker.rs` where the version store is,
//! the conflict's stays below it where the two texts are, and the two resolvers
//! above both route through [`prose_for`]. **One resolver, two producers** — no
//! second renderer and no parallel cache, which is the whole point of the
//! generalisation.
//!
//! # Why the dispatch is a match and not a trait
//!
//! A trait object would let a kind grow a comparison without anyone noticing.
//! The wildcard-free match below means a new [`crate::overlay::OverlayKind`] must
//! say whether it produces read-only prose before it compiles — the same
//! discipline `comparison_request` applies at the asking end, now applied at the
//! answering one.

use crate::overlay::{CONFLICT_ROWS, ComparisonRequest, ComparisonView, OverlayKind, OverlayState};
use crate::prosediff::DiffCounts;
use std::path::Path;

/// Resolve one read-only comparison request to `(subject, transcript, counts)`.
///
/// `None` for an overlay that shows no comparison, an empty-state row, or a
/// subject this build cannot resolve — the document then simply shows the
/// buffer, which is the calm degrade every caller already handles.
///
/// Called by exactly two consumers, deliberately: the live App's per-frame
/// resolver (which caches by [`ComparisonRequest::cache_key`]) and the headless
/// capture fold (which does not need to). Both hand it the same request, so a
/// `--keys` replay and the running editor cannot disagree about what a
/// comparison shows.
pub fn prose_for(
    ov: &OverlayState,
    request: &ComparisonRequest,
    buffer_path: Option<&Path>,
    is_unnamed_fresh: bool,
    current: &str,
) -> Option<(String, String, DiffCounts)> {
    match ov.kind {
        OverlayKind::History => {
            crate::history::comparison_prose(ov, request, buffer_path, is_unnamed_fresh, current)
        }
        OverlayKind::Conflict => conflict_prose(ov, request, current),
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

/// What the "Version on disk" view says when there is no disk version, because
/// the file was deleted. A whole-text view of nothing would render as an empty
/// document, which reads as "the file is empty" — a different and much more
/// alarming fact than the true one.
pub const DELETED_ON_DISK: &str = "The file was deleted. There is no version \
     on disk to take — Save your version writes it back.";

/// THE CONFLICT'S PRODUCER: three read-only views of the same two manuscripts.
///
/// * `Differences` is the writer's diff of the DISK text against the buffer —
///   the same marked-up-manuscript transcript a timeline shows, with the disk
///   version standing in for "the earlier version". That is not an analogy: in
///   both cases the reader is asking "what would I be replacing".
/// * `Mine` and `Theirs` are each version whole and unmarked. They exist because
///   a diff answers "what changed" and cannot answer "is this one still the one
///   I want" — a rewritten paragraph reads as noise beside its own replacement,
///   and a reader deciding between two manuscripts needs to see each of them.
///
/// The title is the ROW'S OWN LABEL, read out of [`CONFLICT_ROWS`] at the view's
/// position in [`ComparisonView::ALL`] rather than spelled again here — so the
/// prose names itself with the same words the list does, structurally, and a
/// reader at a narrow width with the list off-screen still knows which version
/// they are reading. Reads only — the buffer is NEVER touched, which is what
/// makes `Esc` from here "back to editing, unresolved".
fn conflict_prose(
    ov: &OverlayState,
    request: &ComparisonRequest,
    current: &str,
) -> Option<(String, String, DiffCounts)> {
    let subject = ov.conflict.as_ref()?;
    let name = subject
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| subject.path.to_string_lossy().to_string());
    let label = ComparisonView::ALL
        .iter()
        .position(|v| *v == request.view)
        .and_then(|i| CONFLICT_ROWS.get(i))?;
    let title = format!("{label} — {name}");
    let (transcript, counts) = match request.view {
        ComparisonView::Differences => {
            // A DELETED file diffs against nothing, which renders as "the whole
            // document was inserted" — true, and the honest reading of a
            // deletion the user is being asked to consent to.
            let theirs = subject.theirs.as_deref().unwrap_or("");
            crate::prosediff::diff_and_render(
                theirs,
                current,
                crate::prosediff::Params::shipping(),
                &title,
            )
        }
        ComparisonView::Mine => (whole(&title, current), DiffCounts::default()),
        ComparisonView::Theirs => (
            whole(&title, subject.theirs.as_deref().unwrap_or(DELETED_ON_DISK)),
            DiffCounts::default(),
        ),
    };
    Some((request.subject.clone(), transcript, counts))
}

/// One WHOLE, UNMARKED version under its own title — the same `# title` header
/// [`crate::prosediff::render_markdown_blocks`] gives a transcript, so the two
/// kinds of view compose into one surface with one typographic grammar rather
/// than one of them arriving bare.
fn whole(title: &str, text: &str) -> String {
    format!("# {title}\n\n{text}")
}

#[cfg(test)]
mod tests;
