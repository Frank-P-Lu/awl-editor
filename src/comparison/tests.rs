//! Laws for THE ONE DISPATCH and the conflict's producer.
//!
//! The claim this file is here to make non-vacuous: the dispatch is general, and
//! the three views of one subject are three DIFFERENT texts.
//! `overlay::comparison`'s `two_views_of_one_subject_are_two_cache_entries`
//! proves the KEY cannot collide; these prove the ANSWERS do not.

use super::*;
use crate::overlay::{CONFLICT_ROWS, ComparisonRequest, ComparisonView, OverlayState};
use std::path::PathBuf;

const MINE: &str = "The heron stood in the shallows.\n\nIt did not move.\n";
const THEIRS: &str = "The heron stood in the reeds.\n\nIt did not move.\n";

fn conflict_card() -> OverlayState {
    OverlayState::new_conflict(PathBuf::from("/notes/heron.md"), Some(THEIRS.to_string()))
}

fn ask(card: &OverlayState, view: ComparisonView) -> String {
    let request = ComparisonRequest {
        view,
        subject: "/notes/heron.md".to_string(),
    };
    let (subject, transcript, _) = prose_for(
        card,
        &request,
        Some(std::path::Path::new("/notes/heron.md")),
        false,
        MINE,
    )
    .unwrap_or_else(|| panic!("{view:?} must resolve on a conflict workspace"));
    assert_eq!(
        subject, "/notes/heron.md",
        "the subject is echoed back whole"
    );
    transcript
}

/// **THE SEAM.** Every view the conflict offers resolves to prose, and the three
/// are pairwise DIFFERENT — the property a shared cache key would have destroyed
/// silently, and the one a `None`-returning producer would have destroyed
/// loudly. Swept over `ComparisonView::ALL` rather than the three the author had
/// in mind, so a fourth view cannot ship unanswered.
#[test]
fn every_view_of_a_conflict_resolves_to_its_own_prose() {
    let _g = crate::testlock::serial();
    let card = conflict_card();
    let texts: Vec<String> = ComparisonView::ALL
        .into_iter()
        .map(|v| ask(&card, v))
        .collect();
    let unique: std::collections::BTreeSet<&String> = texts.iter().collect();
    assert_eq!(
        unique.len(),
        texts.len(),
        "two views of one conflict produced the SAME prose: {texts:#?}"
    );
    for t in &texts {
        assert!(t.starts_with("# "), "every view titles itself: {t:?}");
    }
}

/// The whole-text views are WHOLE and UNMARKED: the reader is looking at a
/// manuscript, not at a diff of one. A producer that quietly diffed here would
/// still pass the distinctness law above.
#[test]
fn the_two_whole_views_carry_each_version_verbatim() {
    let _g = crate::testlock::serial();
    let card = conflict_card();
    let mine = ask(&card, ComparisonView::Mine);
    let theirs = ask(&card, ComparisonView::Theirs);
    assert!(
        mine.contains(MINE),
        "Your version must carry the buffer whole"
    );
    assert!(
        theirs.contains(THEIRS),
        "Version on disk must carry the disk text whole"
    );
    // …and neither carries the other's, which is what "one at a time" means.
    assert!(
        !mine.contains("reeds"),
        "Your version showed the disk text too"
    );
    assert!(
        !theirs.contains("shallows"),
        "Version on disk showed the buffer text too"
    );
    // No diff marks anywhere: `prosediff` spells insertions `==…==` and
    // deletions `~~…~~`, so their absence is the unmarked claim.
    for (label, t) in [("mine", &mine), ("theirs", &theirs)] {
        assert!(
            !t.contains("==") && !t.contains("~~"),
            "{label} is a whole version, not a marked-up one: {t:?}"
        );
    }
}

/// The `Differences` view IS a diff — it reports changed blocks — and it reads
/// the DISK text as the old side, so what it shows is "what my version would
/// replace".
#[test]
fn the_differences_view_diffs_the_disk_against_the_buffer() {
    let _g = crate::testlock::serial();
    let card = conflict_card();
    let request = ComparisonRequest {
        view: ComparisonView::Differences,
        subject: "/notes/heron.md".to_string(),
    };
    let (_, transcript, counts) = prose_for(&card, &request, None, false, MINE).expect("resolves");
    assert!(
        counts.modified + counts.struck + counts.washed > 0,
        "a real divergence must produce marks: {counts:?}"
    );
    assert!(
        transcript.contains("shallows") && transcript.contains("reeds"),
        "both sides appear in the diff: {transcript}"
    );
}

/// A DELETED file has no disk version. The whole-text view says so in words
/// rather than rendering an empty document, which would read as "the file is
/// empty" — a different and much calmer-looking fact than the true one.
#[test]
fn a_deleted_file_says_so_instead_of_showing_an_empty_version() {
    let _g = crate::testlock::serial();
    let card = OverlayState::new_conflict(PathBuf::from("/notes/gone.md"), None);
    let request = ComparisonRequest {
        view: ComparisonView::Theirs,
        subject: "/notes/gone.md".to_string(),
    };
    let (_, transcript, _) = prose_for(&card, &request, None, false, MINE).expect("resolves");
    assert!(
        transcript.contains(DELETED_ON_DISK),
        "a deleted file's disk view must name the deletion: {transcript}"
    );
}

/// **THE DISPATCH IS WILDCARD-FREE AND HONEST.** Every kind that ASKS for prose
/// gets prose; every kind that does not ask gets `None` even when handed a
/// request anyway. The second half is the one that matters: it is what stops a
/// producer from answering for a surface that never asked.
#[test]
fn only_the_comparison_surfaces_produce_prose() {
    let _g = crate::testlock::serial();
    let request = ComparisonRequest {
        view: ComparisonView::Differences,
        subject: "anything".into(),
    };
    let mut answered = Vec::new();
    for kind in crate::overlay::OverlayKind::ALL {
        let card = OverlayState::new(kind, vec!["a row".into()], vec![], vec![]);
        if prose_for(&card, &request, None, false, MINE).is_some() {
            answered.push(kind);
        }
    }
    // A generic card carries no History row meta and no conflict payload, so
    // neither comparison surface can answer from one — which is exactly the
    // "unresolvable subject" degrade, asserted rather than assumed. CREDITS is
    // the one deliberate exception: its prose is a compiled-in constant, not a
    // caller-supplied payload, so it answers from a bare card regardless of
    // the request's view/subject — there is nothing else for it to gate on.
    assert_eq!(
        answered,
        vec![crate::overlay::OverlayKind::Credits],
        "only Credits should produce prose from a bare card with no payload: {answered:?}"
    );
    // …and the conflict card, which DOES carry its payload, answers.
    assert!(prose_for(&conflict_card(), &request, None, false, MINE).is_some());
}

/// The row the user is standing on decides the view, and the row LABELS are
/// pinned to `ComparisonView::ALL` in lockstep. Two parallel rosters that can
/// reorder independently is how a list ends up naming one view and showing
/// another.
#[test]
fn the_row_the_reader_stands_on_is_the_view_they_get() {
    let _g = crate::testlock::serial();
    assert_eq!(CONFLICT_ROWS.len(), ComparisonView::ALL.len());
    let mut card = conflict_card();
    for (i, expected) in ComparisonView::ALL.into_iter().enumerate() {
        card.selected = i;
        let request = card
            .comparison_request()
            .unwrap_or_else(|| panic!("row {i} must ask for a view"));
        assert_eq!(
            request.view, expected,
            "row {i} ({}) asked for {:?}",
            CONFLICT_ROWS[i], request.view
        );
        assert_eq!(request.subject, "/notes/heron.md");
        // The prose the row resolves to names ITSELF by that row's label, so a
        // reader at a narrow width, with the list off-screen, still knows which
        // version they are reading.
        let (_, transcript, _) = prose_for(&card, &request, None, false, MINE).expect("resolves");
        assert!(
            transcript.starts_with(&format!("# {}", CONFLICT_ROWS[i])),
            "row {i} resolved to prose titled {:?}",
            transcript.lines().next()
        );
    }
}

/// **THE CACHE-KEY HAZARD, CLOSED END TO END.** 116d proved the three keys
/// differ; this proves the three ANSWERS differ under those keys, which is the
/// claim a consumer actually depends on. Written as a real cache round-trip so
/// it fails on a producer that ignores the view, not merely on a key that drops
/// it.
#[test]
fn three_views_of_one_subject_never_serve_each_other_from_the_cache() {
    let _g = crate::testlock::serial();
    let mut card = conflict_card();
    let mut cache: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for i in 0..CONFLICT_ROWS.len() {
        card.selected = i;
        let request = card.comparison_request().expect("asks");
        let key = request.cache_key();
        let (_, transcript, _) = prose_for(&card, &request, None, false, MINE).expect("resolves");
        if let Some(previous) = cache.insert(key.clone(), transcript.clone()) {
            assert_eq!(previous, transcript, "the same key served two texts: {key}");
        }
    }
    assert_eq!(
        cache.len(),
        CONFLICT_ROWS.len(),
        "three views collapsed into {} cache entries",
        cache.len()
    );
    let distinct: std::collections::BTreeSet<&String> = cache.values().collect();
    assert_eq!(
        distinct.len(),
        cache.len(),
        "two entries hold the same text"
    );
}
