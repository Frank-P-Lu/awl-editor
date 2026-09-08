//! **SELECT WITHIN THE TRANSCRIPT.**
//!
//! While History, Conflict or Credits substitutes a transcript for the pixels,
//! `sync_document` already answers every other reader-facing question about
//! that transcript rather than the hidden buffer. One door was left open
//! on purpose: `SemanticRequest::SetTextSelection` still mapped its grapheme
//! offsets against the REAL buffer unconditionally, so an assistive technology
//! "selecting" a line of a transcript silently moved the invisible document's
//! caret instead — reachable through no `--keys` chord (this is a
//! `performKeyEquivalent:`-shaped action request, CLAUDE.md's own note on why a
//! guard here needs to sit at the action level), so no capture door can see the
//! gap either way.
//!
//! The decision is to select WITHIN the transcript: the action stays
//! advertised, and it stays a real transition — it just answers about the text
//! a reader can actually see, exactly the same substitution
//! `SemanticProjection::sync_document`'s doc already states for the run text
//! itself.
//!
//! # Enrolment
//!
//! Derived from [`crate::overlay::OverlayState::shows_read_only_prose`] over
//! [`OverlayKind::ALL`], the same shape `comparison_leak`'s family uses —
//! never a named list, so a fourth reading surface enrols the day it compiles.

use super::*;
use crate::overlay::{ComparisonView, OverlayKind, OverlayState};
use crate::semantic::{DOCUMENT_ID, SemanticRequest, SemanticSelection};
use std::sync::Arc;

const BUFFER_TEXT: &str = "line one of the real document\nline two, still real and hidden\n";
const DISK_TEXT: &str = "the disk version, unrelated to the buffer or to any transcript";
const HISTORY_TIMESTAMP: u64 = 1_700_000_000_000;
const HISTORY_OLD_VERSION: &str = "the version before the rewrite, gone from the buffer now";

/// The most populated card this suite knows how to build for `kind` —
/// wildcard-free, mirroring `comparison_leak::representative`, so a fourth
/// read-only kind must be given a subject here before the sweep can enrol it.
fn representative(kind: OverlayKind) -> OverlayState {
    match kind {
        OverlayKind::History => OverlayState::new_history(
            vec![crate::history::TimelineRow {
                when: "2 hr ago".into(),
                which: "edited \"Title\"".into(),
                counts: "+1 -1".into(),
                id: HISTORY_TIMESTAMP.to_string(),
                timestamp: HISTORY_TIMESTAMP,
                pinned: false,
                name: None,
            }],
            None,
            None,
        ),
        OverlayKind::Conflict => {
            let mut ov =
                OverlayState::new_conflict(PathBuf::from("/proj/draft.md"), Some(DISK_TEXT.into()));
            // "Version on disk" — the view whose subject is NOT the buffer, so
            // a leaked buffer selection has somewhere unambiguous to show up.
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

/// **THE FAMILY, DERIVED.** Never a literal list — the roster's own documented
/// failure mode (CLAUDE.md) is an enrolment pinned to named members that
/// silently stops matching.
fn family() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .into_iter()
        .filter(|k| representative(*k).shows_read_only_prose())
        .collect()
}

fn app_with_real_buffer() -> App {
    let mut app = App::new_hermetic(
        Some(PathBuf::from("/proj/draft.md")),
        PathBuf::from("/proj"),
        Config::empty(),
    );
    app.set_semantic_text_for_test(BUFFER_TEXT);
    app
}

/// History's own setup: a REAL recorded snapshot at `HISTORY_TIMESTAMP`, so its
/// Differences view has something to diff against. `App::new_hermetic` swaps in
/// its own EMPTY filesystem only for the duration of construction
/// (`crate::fs::with_fs`), so the record has to land on the AMBIENT one this
/// returns holding the guard for — an unseeded row resolves to nothing and the
/// comparison silently degrades to showing the buffer instead, which would
/// make the whole law vacuous rather than a real test of the transcript path
/// (mirrors `comparison_leak`'s own History fixture).
fn app_with_real_buffer_and_history() -> (App, crate::fs::FsGuard) {
    let path = PathBuf::from("/proj/draft.md");
    let fs = crate::fs::InMemoryFs::new().with_file(&path, BUFFER_TEXT);
    let guard = crate::fs::FsGuard::install(Arc::new(fs));
    crate::history::record_at(
        &path,
        HISTORY_OLD_VERSION,
        &Config::empty(),
        HISTORY_TIMESTAMP,
        false,
        None,
    );
    let mut app = App::new_hermetic(Some(path), PathBuf::from("/proj"), Config::empty());
    app.set_semantic_text_for_test(BUFFER_TEXT);
    (app, guard)
}

/// The published DOCUMENT node's selection, read back through the same
/// projection a real platform adapter holds — never re-derived.
fn document_selection(app: &App) -> Option<SemanticSelection> {
    app.frame.accessibility_projection().and_then(|projection| {
        projection
            .snapshot()
            .nodes
            .iter()
            .find(|node| node.id == DOCUMENT_ID)
            .and_then(|node| node.selection)
    })
}

/// **THE PRESENCE COMPANION** (CLAUDE.md): with no overlay up, the very same
/// request really does move the real buffer's cursor and anchor — otherwise
/// every refusal the family sweep below asserts could be a request that had
/// stopped moving anything at all, on any subject.
#[test]
fn set_text_selection_moves_the_real_buffer_with_no_card_up() {
    let _g = crate::testlock::serial();

    let mut plain = app_with_real_buffer();
    plain.attach_assistive_technology_for_test();
    let text_before = plain.document.buffer().text();
    let version_before = plain.document.buffer().version();
    assert!(
        plain.apply_semantic_request(SemanticRequest::SetTextSelection {
            id: DOCUMENT_ID.to_string(),
            anchor: 2,
            focus: 5,
        }),
        "with no card up, SetTextSelection on the document must still be handled"
    );
    assert_eq!(plain.document.buffer().text(), text_before);
    assert_eq!(
        plain.document.buffer().version(),
        version_before,
        "a selection move must not itself take an undo step"
    );
    assert_eq!(
        plain.document.buffer().anchor_char(),
        Some(2),
        "with no card up the request must move the REAL buffer's anchor — \
         otherwise the family sweep proves nothing"
    );
    assert_eq!(
        plain.document.buffer().cursor_char(),
        5,
        "with no card up the request must move the REAL buffer's cursor"
    );
}

/// One member of the family: a `SetTextSelection` request over `kind` moves
/// the published TRANSCRIPT selection to exactly what was asked, and the
/// hidden buffer's own text, version, cursor and anchor are exactly what they
/// were.
fn assert_selection_moves_only_the_transcript(kind: OverlayKind) {
    // History's Differences view needs a REAL recorded snapshot to diff
    // against; every other kind is fine with the plain hermetic buffer. The
    // guard has to outlive the app's later `refresh_accessibility` calls
    // below, which is why it is bound here rather than dropped immediately.
    let (mut app, _history_guard) = if kind == OverlayKind::History {
        let (app, guard) = app_with_real_buffer_and_history();
        (app, Some(guard))
    } else {
        (app_with_real_buffer(), None)
    };
    app.workspace_state
        .install_overlay_for_test(representative(kind));
    assert!(
        app.presents_read_only_prose(),
        "{kind:?} enrolled in the family but the App does not read it as one"
    );
    app.attach_assistive_technology_for_test();
    assert!(
        app.frame
            .accessibility_projection()
            .expect("a real attach sequence must park a projection")
            .showing_transcript(),
        "{kind:?}: the seeded tree must be built from the transcript, or this \
         law's own gate never engages and every assertion below is vacuous"
    );

    let buffer_text_before = app.document.buffer().text();
    let buffer_version_before = app.document.buffer().version();
    let buffer_cursor_before = app.document.buffer().cursor_char();
    let buffer_anchor_before = app.document.buffer().anchor_char();

    assert!(
        app.apply_semantic_request(SemanticRequest::SetTextSelection {
            id: DOCUMENT_ID.to_string(),
            anchor: 0,
            focus: 1,
        }),
        "{kind:?}: SetTextSelection is advertised on a read-only document but \
         was refused — an advertised action nothing routes is worse than one \
         that is not advertised at all"
    );

    assert_eq!(
        app.document.buffer().text(),
        buffer_text_before,
        "{kind:?}: a selection into the transcript moved the hidden buffer's text"
    );
    assert_eq!(
        app.document.buffer().version(),
        buffer_version_before,
        "{kind:?}: the hidden buffer took an undo step for a transcript selection"
    );
    assert_eq!(
        app.document.buffer().cursor_char(),
        buffer_cursor_before,
        "{kind:?}: the hidden buffer's cursor moved — a reader-facing \
         question answered about invisible text, one field over from the \
         run text itself"
    );
    assert_eq!(
        app.document.buffer().anchor_char(),
        buffer_anchor_before,
        "{kind:?}: the hidden buffer's anchor moved"
    );

    app.refresh_accessibility();
    assert_eq!(
        document_selection(&app),
        Some(SemanticSelection {
            anchor: 0,
            focus: 1
        }),
        "{kind:?}: the published selection must be the transcript's own \
         offsets, exactly what was requested — \"it should select what you \
         selected\""
    );
}

/// **THE LAW.** For every member of the derived read-only-prose family, a
/// `SetTextSelection` request on the document node moves the published
/// TRANSCRIPT selection to exactly what was asked, and the hidden buffer's own
/// text, version, cursor and anchor are exactly what they were.
#[test]
fn set_text_selection_moves_the_transcript_never_the_hidden_buffer() {
    let _g = crate::testlock::serial();

    let enrolled = family();
    assert!(
        !enrolled.is_empty(),
        "the read-only prose family enrolled NOTHING — derived from \
         `OverlayState::shows_read_only_prose` over `OverlayKind::ALL`, the sweep \
         below would pass over an empty set"
    );

    for kind in enrolled {
        assert_selection_moves_only_the_transcript(kind);
    }
}

/// **CLAMPED, NOT TRUSTED.** An assistive technology's own bookkeeping about a
/// tree it no longer holds (a stale grapheme count from a previous, longer
/// transcript) must not be read past what is actually published now.
#[test]
fn set_text_selection_clamps_to_the_transcripts_own_length() {
    let _g = crate::testlock::serial();

    let mut app = app_with_real_buffer();
    app.workspace_state
        .install_overlay_for_test(representative(OverlayKind::Credits));
    app.attach_assistive_technology_for_test();
    let total: usize = app
        .frame
        .accessibility_projection()
        .unwrap()
        .snapshot()
        .nodes
        .iter()
        .filter(|node| node.id.starts_with("document.run."))
        .map(|node| node.character_lengths.len())
        .sum();
    assert!(total > 0, "Credits must publish some transcript text");

    assert!(
        app.apply_semantic_request(SemanticRequest::SetTextSelection {
            id: DOCUMENT_ID.to_string(),
            anchor: total + 500,
            focus: total + 900,
        })
    );
    app.refresh_accessibility();
    assert_eq!(
        document_selection(&app),
        Some(SemanticSelection {
            anchor: total,
            focus: total,
        }),
        "an out-of-range request must clamp to the transcript's real total \
         length, never panic and never silently read as the buffer's own"
    );
}

/// **A NEW SUBJECT, A FRESH SELECTION.** Moving from one transcript to a
/// DIFFERENT one — a different History row, say — without ever leaving
/// transcript mode (`built_from_transcript` never toggles, so
/// `SemanticProjection::invalidate` never runs) must not carry the old
/// subject's offsets onto the new one, which could name a wildly different
/// position or simply not exist in shorter prose.
#[test]
fn a_new_transcript_subject_starts_the_selection_over() {
    let _g = crate::testlock::serial();

    let mut app = app_with_real_buffer();
    let mut conflict = representative(OverlayKind::Conflict);
    // Land on "Mine" first, a real short subject distinct from "Theirs".
    conflict.selected = ComparisonView::ALL
        .iter()
        .position(|v| *v == ComparisonView::Mine)
        .unwrap();
    app.workspace_state.install_overlay_for_test(conflict);
    app.attach_assistive_technology_for_test();

    assert!(
        app.apply_semantic_request(SemanticRequest::SetTextSelection {
            id: DOCUMENT_ID.to_string(),
            anchor: 1,
            focus: 1,
        })
    );
    app.refresh_accessibility();
    assert_eq!(
        document_selection(&app),
        Some(SemanticSelection {
            anchor: 1,
            focus: 1
        }),
        "the selection must move on the subject it was asked to move on"
    );

    // Switch the row selection to a DIFFERENT view of the same conflict —
    // `built_from_transcript` stays `true` across this, so only
    // `sync_transcript`'s own reset is what can catch the stale offset.
    let overlay = app.workspace_state.overlay_mut().unwrap();
    overlay.selected = ComparisonView::ALL
        .iter()
        .position(|v| *v == ComparisonView::Theirs)
        .unwrap();
    app.refresh_accessibility();

    assert_eq!(
        document_selection(&app),
        Some(SemanticSelection {
            anchor: 0,
            focus: 0
        }),
        "a new transcript subject must start the reader's selection over, not \
         carry the previous subject's offsets onto text they never named"
    );
}
