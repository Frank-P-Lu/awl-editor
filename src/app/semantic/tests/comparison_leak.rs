//! **THE ACCESSIBILITY TREE MUST DESCRIBE WHAT THE PIXELS SHOW** — the
//! substituted comparison prose, never the buffer behind it — for every
//! member of the read-only-prose family.
//!
//! # Why this is its own law rather than an assumption
//!
//! The render pipeline relocates the document layer into a comparison and
//! substitutes a transcript for the pixels
//! (`crate::comparison::prose_for`, `render/chrome/card.rs`'s
//! `figure_source`), but the accessibility tree is built by an entirely
//! separate fold (`app/semantic/*`) walking the buffer's own `RunTable`
//! (`SemanticProjection::build_run`/`seed`/`sync_runs`). Nothing forces the
//! two to agree — a screen reader (or a `--semantic-json` capture) focused on
//! the document node can only be trusted to describe what a sighted reader
//! sees if something asserts they cannot drift apart. This file is that
//! assertion, proved against a canary string that must never appear in any
//! published run while a family member is up.
//!
//! # Enrolment
//!
//! The family is derived from [`crate::overlay::OverlayState::shows_read_only_prose`]
//! over [`OverlayKind::ALL`], never named — the roster's own documented
//! failure mode (CLAUDE.md) is an enrolment pinned to named members that
//! silently stops matching. `family()` below is the same shape
//! `app::tests::read_only_surface::family` and
//! `render::tests::read_only_caret`'s own enrolments already use.

use super::*;
use crate::overlay::{ComparisonView, OverlayKind, OverlayState};
use std::sync::Arc;

/// The buffer's own text while a comparison is up — must never appear
/// verbatim in a published run for Credits or Conflict's "Version on disk"
/// (an entirely different document is substituted for both). History's
/// Differences view is the one legitimate exception: a diff of a document
/// against ITS OWN past necessarily quotes the current buffer as one side of
/// the comparison, so History's law asserts the TRANSCRIPT shape instead (see
/// `the_accessibility_tree_shows_historys_diff_transcript_not_the_raw_buffer`).
const SECRET_BUFFER_TEXT: &str = "PASSWORD-CANARY-42: nobody else may read this line";

const DISK_TEXT: &str = "Whatever is actually saved on disk, unrelated to the buffer";

/// **THE FAMILY, DERIVED.** Every kind whose representative card asks for
/// read-only prose — never a literal list.
fn family() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .into_iter()
        .filter(|k| representative(*k).shows_read_only_prose())
        .collect()
}

/// The most populated card this suite knows how to build for `kind` —
/// wildcard-free, mirroring `app::tests::read_only_surface::representative`,
/// so a fourth read-only kind must be given a subject here before the sweep
/// can enrol it (a bare card would silently enrol nothing).
fn representative(kind: OverlayKind) -> OverlayState {
    match kind {
        OverlayKind::History => OverlayState::new_history(
            vec![crate::history::TimelineRow {
                when: "2 hr ago".into(),
                which: "edited \"Title\"".into(),
                counts: "+1 -1".into(),
                id: "1700000000000".into(),
                timestamp: 1_700_000_000_000,
                pinned: false,
                name: None,
            }],
            None,
            None,
        ),
        OverlayKind::Conflict => {
            let mut ov =
                OverlayState::new_conflict(PathBuf::from("/proj/draft.md"), Some(DISK_TEXT.into()));
            // Land on "Version on disk" (`ComparisonView::ALL[2]`) — the row
            // whose subject is NOT the buffer, so the substitution has
            // somewhere unambiguous to prove itself against.
            ov.selected = ComparisonView::ALL
                .iter()
                .position(|v| *v == ComparisonView::Theirs)
                .unwrap();
            ov
        }
        OverlayKind::Credits => OverlayState::new_credits(),
        _ => OverlayState::new(kind, vec!["a row".into()], vec![], vec![]),
    }
}

/// One live App with a REAL buffer holding [`SECRET_BUFFER_TEXT`], so a
/// projection that leaks the buffer has something distinctive to leak.
fn app_with_secret_buffer() -> App {
    let mut app = App::new_hermetic(
        Some(PathBuf::from("/proj/draft.md")),
        PathBuf::from("/proj"),
        Config::empty(),
    );
    app.set_semantic_text_for_test(SECRET_BUFFER_TEXT);
    app
}

/// Every text run's `value`, joined the same way [`OverlayState`]'s consumers
/// join document lines — with `\n`, so a joined result is directly comparable
/// to a whole transcript string.
fn document_run_text(snapshot: &crate::semantic::SemanticSnapshot) -> String {
    snapshot
        .nodes
        .iter()
        .filter(|n| n.id.starts_with("document.run."))
        .map(|n| n.value.clone().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

/// **THE LAW.** For every family member: the published document text is never
/// the raw buffer, and — for Credits and Conflict's disk view, where the
/// substituted document has NOTHING to do with the buffer — the buffer's own
/// canary never appears in it at all.
///
/// The PRESENCE COMPANION is not optional (CLAUDE.md): a law satisfied by
/// deleting its own subject is worse than no law. So this also asserts, with
/// NO overlay installed, that the ordinary document tree DOES publish the
/// canary — otherwise "the canary never appears" could pass because nothing
/// here ever publishes canary text at all, family enrolled or not.
#[test]
fn the_accessibility_tree_never_leaks_the_buffer_while_a_wholly_different_document_is_shown() {
    let _g = crate::testlock::serial();

    let enrolled = family();
    assert!(
        !enrolled.is_empty(),
        "the read-only prose family enrolled NOTHING — derived from \
         `OverlayState::shows_read_only_prose` over `OverlayKind::ALL`"
    );

    // PRESENCE: with no overlay up, the ordinary document tree really does
    // publish the buffer's own text — otherwise the refusals below would be
    // vacuous.
    let plain = app_with_secret_buffer();
    let plain_text = document_run_text(&plain.semantic_snapshot());
    assert!(
        plain_text.contains(SECRET_BUFFER_TEXT),
        "the ordinary document tree must publish the buffer's own text, or the \
         absences below prove nothing (got {plain_text:?})"
    );

    for kind in [OverlayKind::Credits, OverlayKind::Conflict] {
        assert!(
            enrolled.contains(&kind),
            "{kind:?} must be a member of the derived family for this law to mean \
             anything (family: {enrolled:?})"
        );
        let mut app = app_with_secret_buffer();
        app.workspace_state
            .install_overlay_for_test(representative(kind));
        assert!(
            app.presents_read_only_prose(),
            "{kind:?}'s representative card must read as read-only prose"
        );
        let text = document_run_text(&app.semantic_snapshot());
        assert!(
            !text.is_empty(),
            "{kind:?}: the substituted document published no text at all"
        );
        assert!(
            !text.contains(SECRET_BUFFER_TEXT),
            "{kind:?}: the accessibility tree published the REAL buffer's text \
             ({SECRET_BUFFER_TEXT:?}) while a wholly different document was on \
             screen — got {text:?}"
        );
    }

    // Conflict's "Version on disk" must show the DISK text specifically, not
    // merely "not the buffer" — proving the substitution actually reached the
    // right producer, not just an empty or unrelated placeholder.
    let mut conflict_app = app_with_secret_buffer();
    conflict_app
        .workspace_state
        .install_overlay_for_test(representative(OverlayKind::Conflict));
    let conflict_text = document_run_text(&conflict_app.semantic_snapshot());
    assert!(
        conflict_text.contains(DISK_TEXT),
        "Conflict's \"Version on disk\" must publish the disk text — got {conflict_text:?}"
    );
}

/// **HISTORY'S OWN SHAPE.** A Differences view legitimately quotes the
/// current buffer — it is a diff of the document against its own past, not a
/// substitution of an unrelated document — so the law here is not "the buffer
/// never appears" but "what is published is the DIFF TRANSCRIPT
/// (`crate::comparison::prose_for`'s own dispatch), not the raw undiffed
/// buffer verbatim". Before this item's fix, the projection read
/// `Buffer::run_text` directly, so the published text was byte-identical to
/// the plain buffer and never carried the transcript's own title line.
#[test]
fn the_accessibility_tree_shows_historys_diff_transcript_not_the_raw_buffer() {
    let _g = crate::testlock::serial();
    let path = PathBuf::from("/proj/draft.md");
    let fs = crate::fs::InMemoryFs::new().with_file(&path, SECRET_BUFFER_TEXT);
    let _fs = crate::fs::FsGuard::install(Arc::new(fs));

    const OLD_VERSION: &str = "the version before the rewrite, gone from the buffer now";
    crate::history::record_at(
        &path,
        OLD_VERSION,
        &Config::empty(),
        1_700_000_000_000,
        false,
        None,
    );

    let mut app = App::new_hermetic(Some(path), PathBuf::from("/proj"), Config::empty());
    app.set_semantic_text_for_test(SECRET_BUFFER_TEXT);
    app.workspace_state
        .install_overlay_for_test(representative(OverlayKind::History));
    assert!(app.presents_read_only_prose());

    // The ground truth, resolved through the SAME dispatch the fold now uses
    // — no second literal of what a diff transcript looks like.
    let overlay = app.workspace_state.overlay().unwrap().clone();
    let request = overlay.comparison_request().unwrap();
    let (_, expected_transcript, _) = crate::comparison::prose_for(
        &overlay,
        &request,
        Some(std::path::Path::new("/proj/draft.md")),
        false,
        SECRET_BUFFER_TEXT,
    )
    .expect("History's Differences view must resolve against a seeded snapshot");

    let published = document_run_text(&app.semantic_snapshot());
    assert_eq!(
        published, expected_transcript,
        "the accessibility tree must publish exactly the diff transcript \
         `prose_for` produces, not a different rendering of the buffer"
    );
    assert_ne!(
        expected_transcript, SECRET_BUFFER_TEXT,
        "the transcript must differ from the raw buffer, or the equality above \
         would pass for a projection that never substituted anything"
    );
}

/// **THE SUBSTITUTION BOUNDARY ITSELF, CROSSED TWICE, ON ONE RETAINED
/// PROJECTION.** `semantic_snapshot()` always builds a fresh projection, which
/// cannot exercise `sync_transcript`/`built_from_transcript`'s reseed-on-
/// crossing — the incremental path a live App's `refresh_accessibility`
/// actually runs frame over frame. This drives the SAME retained
/// `SemanticProjection` through buffer -> Credits -> buffer, proving the
/// second crossing does not serve a stale transcript run, a stale RunId
/// clash, or the wrong selection source.
#[test]
fn a_retained_projection_reseeds_cleanly_across_the_substitution_boundary_both_ways() {
    let _g = crate::testlock::serial();
    let mut app = app_with_secret_buffer();

    let mut projection = super::super::projection::SemanticProjection::new();
    projection.refresh(&app.semantic_view());
    let before = document_run_text(projection.snapshot());
    assert!(
        before.contains(SECRET_BUFFER_TEXT),
        "the buffer must publish before any overlay opens, or the crossing below \
         proves nothing (got {before:?})"
    );

    app.workspace_state
        .install_overlay_for_test(representative(OverlayKind::Credits));
    projection.refresh(&app.semantic_view());
    let during = document_run_text(projection.snapshot());
    assert!(
        !during.contains(SECRET_BUFFER_TEXT),
        "crossing INTO Credits on a RETAINED projection must not leave the \
         buffer's text sitting in stale run nodes — got {during:?}"
    );

    app.workspace_state.dismiss_pickers();
    projection.refresh(&app.semantic_view());
    let after = document_run_text(projection.snapshot());
    assert!(
        after.contains(SECRET_BUFFER_TEXT),
        "crossing BACK OUT of Credits must republish the real buffer, not a \
         stale Credits transcript — got {after:?}"
    );
}
