//! **A READING SURFACE ACCEPTS NO TEXT — THROUGH ANY DOOR, ON EVERY MEMBER OF
//! THE READ-ONLY PROSE FAMILY.**
//!
//! # The report this file answers
//!
//! "you can type in the credits screen?" The Credits viewer relocates the one
//! document layer into a workspace pane and substitutes CREDITS.md for the
//! user's own text. The buffer behind it is untouched and INVISIBLE — so a
//! character that reaches it edits a document nobody can see.
//!
//! # Why this tier, and why a sweep over doors rather than one test
//!
//! The keymap door was never the leak: with any card up,
//! `actions::intercept_action` consumes every action before a buffer verb can
//! run, so no chord reaches the rope — proven here too, at the bottom, because a
//! premise that is only *believed* is the premise that turns out to be wrong.
//! What leaked were the doors that bypass `App::apply` altogether and asked only
//! "is there an active document": the platform IME's committed composition, and
//! the two assistive-technology requests that write document text. None of the
//! three has a `--keys` chord — an IME commit never touches the keymap at all —
//! so no capture door can drive them and this is the purest reachable seam
//! (`docs/harness-reach.md`).
//!
//! The sweep is over `TextDoor::ALL` × the family, and BOTH axes are derived
//! rather than listed: the doors from the production roster the wall itself
//! matches on, the family from `OverlayState::shows_read_only_prose`, which is
//! `comparison_request`'s wildcard-free roster asked as a predicate. A fourth
//! read-only surface therefore enrols the day it compiles, and a fourth door
//! fails to compile here until it is actually driven.

use super::*;
use crate::overlay::{OverlayKind, OverlayState};
use crate::semantic::{DOCUMENT_ID, SemanticRequest};
use std::sync::Arc;

const DOC: &str = "# My Notes\n\nSome real prose the user is editing.\n";

fn seeded() -> crate::fs::InMemoryFs {
    crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC)
}

fn app() -> App {
    app_on(
        Some(PathBuf::from("/proj/draft.md")),
        "/proj",
        Config {
            autosave: Some(false),
            session_restore: Some(false),
            ..Config::empty()
        },
    )
}

/// **THE MOST POPULATED CARD THIS SUITE KNOWS HOW TO BUILD FOR `kind`** — the
/// enrolment's own subject.
///
/// Wildcard-free on purpose, and it is the enrolment that this protects rather
/// than the assertion. `OverlayState::new` of a History or Conflict kind carries
/// no row meta and no conflict payload, so it asks for no comparison and would
/// answer `false` to the family predicate — a fourth read-only kind handed a
/// bare card would silently enrol NOTHING and this file would go on passing
/// while the surface it was written for leaked. Forcing the author of that kind
/// to name its representative here is what keeps the sweep's subject alive.
fn representative(kind: OverlayKind) -> OverlayState {
    let generic = || OverlayState::new(kind, vec!["a row".into()], vec![], vec![]);
    match kind {
        OverlayKind::History => OverlayState::new_history(
            vec![crate::history::TimelineRow {
                when: "2 hr ago".into(),
                which: "edited \"Title\"".into(),
                counts: "+1 −1".into(),
                id: "1700000000000".into(),
                timestamp: 1_700_000_000_000,
                pinned: false,
                name: None,
            }],
            None,
            None,
        ),
        OverlayKind::Conflict => {
            OverlayState::new_conflict(PathBuf::from("/proj/draft.md"), Some("disk text".into()))
        }
        OverlayKind::Credits => OverlayState::new_credits(),
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
        | OverlayKind::SearchFolder
        | OverlayKind::Spell
        | OverlayKind::Keybindings
        | OverlayKind::Assets
        | OverlayKind::UserWords
        | OverlayKind::Rename
        | OverlayKind::InsertLink
        | OverlayKind::KeepName
        | OverlayKind::Context
        | OverlayKind::TableDims
        | OverlayKind::Settings => generic(),
    }
}

/// THE FAMILY, derived — every kind whose representative card asks for read-only
/// prose. Never a literal list: the point of `shows_read_only_prose` is that the
/// roster answers this question, so a law that re-spelled the answer here would
/// be two literals agreeing with each other.
fn family() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .into_iter()
        .filter(|k| representative(*k).shows_read_only_prose())
        .collect()
}

/// Drive one door for real. Wildcard-free, so a door added to the production
/// roster cannot ride this sweep without someone saying how it is pressed —
/// which is the difference between a sweep and a list of three tests.
fn drive(app: &mut App, door: TextDoor, text: &str) {
    match door {
        // The SAME `winit` event `lifecycle.rs`'s `WindowEvent::Ime` arm hands
        // to the same `App::on_ime` — the harness's IME injection seam, not a
        // stand-in for it (`App::commit_ime_headless`).
        TextDoor::Ime => app.commit_ime_headless(text),
        TextDoor::AssistiveReplaceSelection => {
            app.apply_semantic_request(SemanticRequest::ReplaceSelectedText {
                id: DOCUMENT_ID.to_string(),
                value: text.to_string(),
            });
        }
        TextDoor::AssistiveSetValue => {
            app.apply_semantic_request(SemanticRequest::SetValue {
                id: DOCUMENT_ID.to_string(),
                value: text.to_string(),
            });
        }
    }
}

/// **THE LAW.** For every door × every family member: the buffer's bytes are
/// what they were.
///
/// The PRESENCE COMPANION is the second half and it is not optional. "The bytes
/// did not change" is satisfied by a door that no longer works at all, by a
/// fixture whose buffer was never active, and by a family that enrolled nobody —
/// so each door is ALSO driven with no card up and required to change the same
/// buffer, and the enrolled family is required to be non-empty and is named in
/// every failure message.
#[test]
fn no_door_writes_text_into_a_read_only_prose_surface() {
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let _g = crate::testlock::serial();

    let enrolled = family();
    assert!(
        !enrolled.is_empty(),
        "the read-only prose family enrolled NOTHING — the sweep below would pass \
         over an empty set. Enrolment comes from `OverlayState::shows_read_only_prose` \
         over `OverlayKind::ALL`."
    );

    for door in TextDoor::ALL {
        // PRESENCE: with nothing summoned, this door really does write. Without
        // this, every refusal below could be a door that silently stopped
        // working.
        let mut open = app();
        let before = open.document.buffer().text();
        drive(&mut open, door, "字");
        assert_ne!(
            open.document.buffer().text(),
            before,
            "{door:?}: with no card up this door must still reach the document — \
             otherwise the refusals below prove nothing"
        );

        for kind in &enrolled {
            let mut app = app();
            app.workspace_state
                .install_overlay_for_test(representative(*kind));
            assert!(
                app.presents_read_only_prose(),
                "{kind:?} enrolled in the family but the App does not read it as one"
            );
            let before = app.document.buffer().text();
            let version = app.document.buffer().version();
            drive(&mut app, door, "字");
            assert_eq!(
                app.document.buffer().text(),
                before,
                "{door:?} wrote into the buffer behind the {kind:?} reading surface \
                 (family enrolled: {enrolled:?})"
            );
            assert_eq!(
                app.document.buffer().version(),
                version,
                "{door:?} on {kind:?}: the buffer must not even take an undo step — a \
                 refusal that bumps the version is an edit that happened to be empty"
            );
        }
    }
}

/// **THE WALL IS SCOPED TO THE FAMILY, AND THAT SCOPE IS A DECISION.**
///
/// A card that is NOT a reading surface — the command palette over your
/// document, say — leaves these doors open exactly as before. That is the
/// deliberate boundary of this item, not an oversight: the question of whether
/// an IME commit should reach the document while any picker is up is a wider one
/// about every summoned surface. Pinning it here means widening the wall later
/// is a decision someone has to make on purpose, with this law in front of them,
/// rather than a silent drift.
#[test]
fn a_card_that_is_not_a_reading_surface_leaves_the_doors_open() {
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let _g = crate::testlock::serial();

    let enrolled = family();
    let outside: Vec<OverlayKind> = OverlayKind::ALL
        .into_iter()
        .filter(|k| !enrolled.contains(k))
        .collect();
    assert!(
        !outside.is_empty(),
        "every kind enrolled in the read-only family — this law has no subject"
    );

    for kind in &outside {
        let mut app = app();
        app.workspace_state
            .install_overlay_for_test(representative(*kind));
        assert!(
            !app.presents_read_only_prose(),
            "{kind:?} is outside the family but the App reads it as a reading surface"
        );
        let before = app.document.buffer().text();
        drive(&mut app, TextDoor::Ime, "字");
        assert_ne!(
            app.document.buffer().text(),
            before,
            "{kind:?}: the wall is scoped to the read-only prose family, so a card \
             outside it must not silently acquire the refusal"
        );
    }
}

/// **THE KEYMAP DOOR, MEASURED RATHER THAN ASSUMED.**
///
/// The wall deliberately does NOT duplicate the overlay intercept, on the
/// grounds that a chord cannot reach the rope while a card is up. That is a
/// claim about production code, and a claim is a hypothesis: if it were ever
/// false the wall would have a hole exactly where it was reasoned to be
/// unnecessary. So drive real chords through the real keymap into the real
/// `App::apply` on every family member and read the bytes back.
#[test]
fn real_chords_reach_no_buffer_behind_a_reading_surface() {
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let _g = crate::testlock::serial();

    // PRESENCE: this spec really does type, so the refusals mean something.
    let mut open = app();
    let before = open.document.buffer().text();
    open.press_spec_headless("a b c Backspace Enter")
        .expect("the spec parses");
    assert_ne!(
        open.document.buffer().text(),
        before,
        "with no card up these chords must edit the document"
    );

    let enrolled = family();
    for kind in &enrolled {
        let mut app = app();
        app.workspace_state
            .install_overlay_for_test(representative(*kind));
        let before = app.document.buffer().text();
        app.press_spec_headless("a b c Backspace Enter")
            .expect("the spec parses");
        assert_eq!(
            app.document.buffer().text(),
            before,
            "{kind:?}: a real chord stream reached the buffer behind the reading \
             surface (family enrolled: {enrolled:?})"
        );
    }
}

/// **CREDITS HAS NOTHING TO SEARCH, SO TYPING ON ITS RAIL DOES NOTHING.**
///
/// Its primary "list" is one fixed row that NAMES the document beside it, so a
/// query can only ever hide that row and leave the reader on `no matches` with
/// the prose it named still on screen. `OverlayKind::offers_query` is the fact,
/// and `OverlayState::push` — the one door the query grows through — reads it.
///
/// The companion is the whole roster: every OTHER kind must still accept the
/// same character, or "the query did not grow" would be true of a query field
/// that stopped working everywhere.
#[test]
fn only_a_card_with_nothing_to_search_refuses_its_query() {
    let _g = crate::testlock::serial();
    let mut refused = Vec::new();
    for kind in OverlayKind::ALL {
        let mut card = representative(kind);
        let rows_before = card.item_strings();
        card.push('z');
        if card.query.text().is_empty() {
            assert!(
                !kind.offers_query(),
                "{kind:?} swallowed a typed character but claims to offer a query"
            );
            assert_eq!(
                card.item_strings(),
                rows_before,
                "{kind:?} refused the character but its rows moved anyway — the refusal \
                 must leave the card exactly as it was"
            );
            refused.push(kind);
        } else {
            assert!(
                kind.offers_query(),
                "{kind:?} grew its query but claims to offer none"
            );
        }
    }
    assert_eq!(
        refused,
        vec![OverlayKind::Credits],
        "exactly the kinds with nothing to search refuse a query"
    );
}

/// **AND AN ASSISTIVE TECHNOLOGY IS TOLD, RATHER THAN LEFT TO DISCOVER IT.**
///
/// A node that advertises an action nothing routes is worse than a node that
/// advertises nothing — `app/semantic/requests.rs`'s own rule, and the reason
/// the wall above is not the whole fix. While a reading surface is up the
/// document node drops both text-WRITING actions and its `editable` flag, so a
/// screen reader offers reading and selecting and never an edit that silently
/// does nothing.
///
/// The presence companion is the same App with the card closed: every action
/// comes back. The read-only fact is per-FRAME, not per-revision — a card opens
/// without touching the buffer's content or shape — so this drives the SAME
/// projection through open and closed rather than two fresh ones, which is
/// exactly the staleness the `sync_document` re-ask exists to prevent.
#[test]
fn the_document_node_stops_advertising_writes_on_a_reading_surface() {
    use crate::semantic::SemanticAction;
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let _g = crate::testlock::serial();

    let writes = [
        SemanticAction::ReplaceSelectedText,
        SemanticAction::SetValue,
    ];
    let document = |app: &App| {
        app.semantic_snapshot()
            .nodes
            .into_iter()
            .find(|n| n.id == DOCUMENT_ID)
            .expect("the tree always carries a document node")
    };

    let enrolled = family();
    assert!(!enrolled.is_empty(), "nothing enrolled — no subject");

    for kind in &enrolled {
        let mut app = app();
        // PRESENCE, on this very App: with nothing summoned the writes are
        // advertised and the document is editable.
        let open = document(&app);
        assert!(open.editable, "an ordinary document must read as editable");
        for w in writes {
            assert!(
                open.actions.contains(&w),
                "an ordinary document must advertise {w:?}, or its absence below \
                 says nothing"
            );
        }

        app.workspace_state
            .install_overlay_for_test(representative(*kind));
        let reading = document(&app);
        assert!(
            !reading.editable,
            "{kind:?}: a reading surface must not report the document as editable"
        );
        for w in writes {
            assert!(
                !reading.actions.contains(&w),
                "{kind:?}: the document node still advertises {w:?}, which the one wall \
                 refuses — an advertised action nothing performs is worse than none \
                 (family enrolled: {enrolled:?})"
            );
        }
        assert!(
            reading.actions.contains(&SemanticAction::SetTextSelection),
            "{kind:?}: reading and selecting stay — the surface is FOR reading"
        );
    }
}
