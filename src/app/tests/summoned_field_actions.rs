//! **A SUMMONED TEXT-ENTRY SURFACE OWNS THE EDIT VERBS — ON EVERY FIELD IN
//! THE ROSTER, THROUGH THE DOOR THAT HAS NO KEYMAP GUARD.**
//!
//! # The report this file answers
//!
//! "⌘A inside Find selects the underlying document instead of the query."
//! With the query `beta`, ⌘A then typing `alpha` left the query reading
//! `betaalpha` while the screenshot showed the whole document selected
//! behind the panel.
//!
//! # The mechanism, and why it was invisible
//!
//! The find/replace panel's KEY door (`search::keys::intercept`) consumes
//! every key, and both key drivers gate on it — so `apply_transition` was
//! documented as unreachable while the panel is up. That is a claim about
//! KEYS. AppKit answers a menu item's key equivalent from the main menu in
//! `performKeyEquivalent:` BEFORE the key window sees the event, so on macOS
//! ⌘A never becomes a winit key at all: it fires Edit ▸ Select all, which
//! `App::handle_menu_event` routes into `App::apply` as an `Action`. The
//! menu/context-menu CLICK and a palette row's `Effect::RunAction` arrive the
//! same way. The summoned CARD has always had an action-level gate
//! (`actions::intercept_action`); the summoned PANEL had none, so a document
//! verb ran against the document parked behind it.
//!
//! # Why this tier, and why a sweep
//!
//! No capture door can drive it: `--keys` (and `--screenshot-app`'s live
//! `App`) both enter through the key drivers, which the panel's key guard
//! already stops — so the defect is structurally unreachable from every
//! chord-replay door and this is the purest reachable seam
//! (`docs/harness-reach.md`). And it is a ROUTING class, not one keystroke:
//! both axes are derived rather than listed — the surfaces from
//! `TextField::ALL` (the roster `textbox.rs` already keeps wildcard-free), the
//! verbs from `menu::edit_verbs::edit_menu_actions()`, which is the Edit menu's own rows
//! resolved through the catalog, i.e. exactly the set macOS installs real key
//! equivalents for. An eighth field fails to COMPILE the installer below until
//! someone says how it is summoned; a seventh Edit row enrols on its own.

use super::*;
use crate::keymap::Action;
use crate::overlay::{LinkEditMode, OverlayKind, OverlayState};
use crate::search::{Direction, SearchState};
use crate::textbox::TextField;
use std::sync::Arc;

const DOC: &str = "alpha beta gamma\nbeta again\n";
/// The same fixture with an inline image, for the one census door that needs
/// something to resize. Kept separate so the Edit-verb sweep above keeps the
/// exact document its assertions were written against.
const DOC_WITH_IMAGE: &str = "alpha beta gamma\nbeta again\n![a cat](cat.png)\n";
const QUERY: &str = "beta";

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

/// **SUMMON THE SURFACE THAT HOSTS `field`, IN THE STATE WHERE THAT FIELD IS
/// THE ONE BEING EDITED** — the sweep's own subject.
///
/// Wildcard-free on purpose. A card handed the wrong sub-state carries no
/// field at all (a bare `OverlayKind::Rename` card has `rename_edit: None`),
/// so an eighth `TextField` given a generic representative would enrol a
/// surface with nothing to type into and the sweep would go on passing while
/// the field it was written for leaked. Making the author of that field name
/// its summon here is what keeps the subject alive.
fn summon(app: &mut App, field: TextField) {
    match field {
        TextField::PickerQuery => {
            let mut ov = OverlayState::new(
                OverlayKind::Goto,
                vec!["draft.md".to_string(), "notes.md".to_string()],
                vec![],
                vec![],
            );
            ov.push('d');
            app.workspace_state.install_overlay_for_test(ov);
        }
        TextField::Rename => {
            app.workspace_state
                .install_overlay_for_test(OverlayState::new_rename("draft.md".to_string()));
        }
        TextField::InsertLink => {
            app.workspace_state
                .install_overlay_for_test(OverlayState::new_link_edit(
                    "https://example.org".to_string(),
                    LinkEditMode::Empty { at: 0 },
                ));
        }
        TextField::KeepVersion => {
            app.workspace_state
                .install_overlay_for_test(OverlayState::new_keep_name());
        }
        TextField::SettingsValue => {
            let mut ov = OverlayState::new(
                OverlayKind::Settings,
                crate::settings::visible_names(),
                vec![],
                vec![],
            );
            ov.set_secondaries(crate::settings::visible_value_cells(&Default::default()));
            ov.start_value_edit("zoom".to_string(), "Zoom".to_string());
            app.workspace_state.install_overlay_for_test(ov);
        }
        TextField::FindQuery => {
            app.workspace_state
                .install_search_for_test(SearchState::start_with_query(
                    0,
                    Direction::Forward,
                    QUERY,
                    DOC,
                ));
        }
        TextField::ReplaceText => {
            let mut st = SearchState::start_with_query(0, Direction::Forward, QUERY, DOC);
            st.focus_replacement();
            for c in "old".chars() {
                st.push_replace_char(c);
            }
            app.workspace_state.install_search_for_test(st);
        }
    }
}

/// The FOCUSED field's own text, read off the very `TextBox` the surface
/// types into — `None` when the surface is no longer standing, or is standing
/// without its field. Wildcard-free like `summon`, and for the same reason:
/// a field nobody knows how to read back cannot be asserted about.
fn field_text(app: &App, field: TextField) -> Option<String> {
    let ov = || app.workspace_state.overlay();
    let st = || app.workspace_state.search();
    match field {
        TextField::FindQuery => st().map(|s| s.query().to_string()),
        TextField::ReplaceText => st().map(|s| s.replacement().to_string()),
        TextField::PickerQuery => ov().map(|o| o.query.text().to_string()),
        TextField::Rename => {
            ov().and_then(|o| o.rename_edit.as_ref().map(|r| r.input.text().to_string()))
        }
        TextField::InsertLink => {
            ov().and_then(|o| o.link_edit.as_ref().map(|l| l.input.text().to_string()))
        }
        TextField::KeepVersion => {
            ov().and_then(|o| o.keep_edit.as_ref().map(|k| k.input.text().to_string()))
        }
        TextField::SettingsValue => {
            ov().and_then(|o| o.value_edit.as_ref().map(|v| v.input.text().to_string()))
        }
    }
}

/// A document snapshot precise enough to catch every verb in the roster: the
/// bytes, the undo version (a refusal that still takes an undo step is an edit
/// that happened to be empty), the caret, and the parked SELECTION — which is
/// what ⌘A actually moved.
fn doc_state(app: &App) -> (String, u64, usize, Option<(usize, usize)>) {
    let b = app.document.buffer();
    (b.text(), b.version(), b.cursor_char(), b.selection_range())
}

/// Fire one action through the SAME `App::apply` seam `handle_menu_event`
/// routes a fired menu item into — the door with no keymap guard in front of
/// it, attributed to `Door::Menu` exactly as the real one is.
fn fire(app: &mut App, action: &Action) {
    let exit = crate::app::schedule::RecordingExit::default();
    app.apply(action.clone(), false, &exit, crate::stats::Door::Menu);
}

/// **THE LAW.** For every summoned field × every Edit-menu verb: the document
/// behind the surface is byte-for-byte, version-for-version, caret- and
/// selection-for-selection what it was.
///
/// The PRESENCE COMPANION is the second half and it is not optional. "The
/// document did not change" is satisfied by a door that stopped working, by a
/// fixture whose buffer was never active, and by a roster that enrolled
/// nobody — so both rosters are required to be non-empty and named in every
/// failure message, the same verbs are driven with NOTHING summoned and
/// required to reach the document, and each surface is required to still be
/// standing (with its field text intact) after the refusal.
#[test]
fn no_edit_menu_verb_reaches_the_document_behind_a_summoned_field() {
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let _g = crate::testlock::serial();

    let verbs = crate::menu::edit_verbs::edit_menu_actions();
    assert!(
        !verbs.is_empty(),
        "the Edit-menu verb roster enrolled NOTHING — the sweep below would pass \
         over an empty set. Enrolment comes from `menu::edit_verbs::edit_menu_actions()`."
    );
    assert!(
        !TextField::ALL.is_empty(),
        "the text-field roster enrolled NOTHING"
    );

    // PRESENCE: with nothing summoned these verbs really do reach the
    // document. Without this every refusal below could be a dead door.
    let mut open = app();
    fire(&mut open, &Action::SelectAll);
    let whole = open.document.buffer().text().chars().count();
    assert_eq!(
        open.document.buffer().selection_range(),
        Some((0, whole)),
        "with nothing summoned, Select all must still select the whole document — \
         otherwise the refusals below prove nothing"
    );
    fire(&mut open, &Action::KillRegion);
    assert!(
        open.document.buffer().text().is_empty(),
        "with nothing summoned, Cut must still remove the selection"
    );

    for field in TextField::ALL {
        for action in &verbs {
            let mut app = app();
            summon(&mut app, field);
            let before_field = field_text(&app, field);
            assert!(
                before_field.is_some(),
                "{field:?}: the fixture did not actually stand this field up — the \
                 whole sweep for it would be vacuous"
            );
            let before = doc_state(&app);
            fire(&mut app, action);
            assert_eq!(
                doc_state(&app),
                before,
                "{action:?} reached the document behind the {field:?} surface \
                 (fields swept: {:?}; verbs swept: {verbs:?})",
                TextField::ALL
            );
            assert_eq!(
                field_text(&app, field),
                before_field,
                "{action:?} on {field:?}: the surface's own field must survive the \
                 refusal — a field that emptied (or a surface that closed) would make \
                 the document assertion above vacuous"
            );
        }
    }
}

/// **SELECT ALL BELONGS TO THE FOCUSED FIELD, AND THE JOURNEY THAT FOLLOWS IT
/// BELONGS THERE TOO.**
///
/// The refusal above is only half the report: ⌘A has to DO something. Both
/// find fields are driven the way the bug was met — the verb through the
/// menu-shaped door, the typing through the REAL key door
/// (`press_spec_headless`, which goes through the keymap and the panel's own
/// key guard) — and the outcome is read off the field, not off the report:
/// the query becomes the typed text, not the concatenation the bug produced.
#[test]
fn select_all_then_typing_replaces_the_focused_find_field() {
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let _g = crate::testlock::serial();

    for (field, focused_is_replacement) in [
        (TextField::FindQuery, false),
        (TextField::ReplaceText, true),
    ] {
        let mut app = app();
        summon(&mut app, field);
        let before = doc_state(&app);
        let text_before = field_text(&app, field).expect("the fixture stood the field up");
        let len = text_before.chars().count();
        assert!(len > 0, "{field:?}: the fixture field must not be empty");

        fire(&mut app, &Action::SelectAll);

        let st = app.workspace_state.search().expect("the panel is still up");
        assert_eq!(
            st.focused_selection(),
            Some((0, len)),
            "{field:?}: Select all must arm the WHOLE focused field"
        );
        // The UNFOCUSED field is left alone — the verb is field-scoped, not
        // panel-scoped.
        let other = if focused_is_replacement {
            st.query_selection()
        } else {
            st.replacement_selection()
        };
        assert_eq!(
            other, None,
            "{field:?}: Select all reached the field that does NOT have focus"
        );
        assert_eq!(
            field_text(&app, field).as_deref(),
            Some(text_before.as_str()),
            "{field:?}: selecting is not editing — the text must be untouched"
        );
        assert_eq!(
            doc_state(&app),
            before,
            "{field:?}: the parked document (bytes, version, caret AND selection) \
             must survive a Select all aimed at the field"
        );

        // The reported journey, through the real keymap: type over the
        // selection. `betaalpha` was the bug; `alpha` is the repair.
        app.press_spec_headless("a l p h a")
            .expect("the spec parses");
        assert_eq!(
            field_text(&app, field).as_deref(),
            Some("alpha"),
            "{field:?}: typing after Select all must REPLACE the field, not append \
             to it"
        );
        assert_eq!(
            app.document.buffer().text(),
            before.0,
            "{field:?}: typing after Select all must not write into the document"
        );
    }
}

/// **THE MACOS MENU DOOR ITSELF**, not a stand-in for it.
///
/// Everything above fires `App::apply` directly, which is where
/// `handle_menu_event` lands — but "the id resolves to that action" is its own
/// link in the chain, and it is the link the whole defect travelled down. Drive
/// the real Edit ▸ Select all id through the real handler and read the document
/// back.
#[cfg(target_os = "macos")]
#[test]
fn the_real_select_all_menu_id_does_not_select_the_document_behind_the_panel() {
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let _g = crate::testlock::serial();

    assert_eq!(
        crate::menu::resolve("awl.select_all"),
        Some(Action::SelectAll),
        "the Edit ▸ Select all row must still route to the verb this law drives"
    );

    let exit = crate::app::schedule::RecordingExit::default();
    let mut app = app();
    summon(&mut app, TextField::FindQuery);
    let before = doc_state(&app);
    app.handle_menu_event("awl.select_all".to_string(), &exit);
    assert_eq!(
        doc_state(&app),
        before,
        "the fired Edit ▸ Select all menu item selected the document parked behind \
         the find panel"
    );
    assert_eq!(
        app.workspace_state
            .search()
            .and_then(|s| s.focused_selection()),
        Some((0, QUERY.chars().count())),
        "the fired menu item must select the QUERY instead"
    );
}

/// **THE WALL IS SCOPED TO ROUTED ACTIONS, AND THAT SCOPE IS A MEASURED
/// DECISION, NOT AN OVERSIGHT — ACROSS THE WHOLE INSERTION-DOOR ROSTER.**
///
/// The doors that bypass `App::apply` altogether still reach the document while
/// the find panel is up. That is not a second half of this item left undone: it
/// is EXACTLY the boundary `app::tests::read_only_surface` already pinned for
/// the summoned CARD ("a card that is not a reading surface leaves the doors
/// open"), and the panel inheriting the same answer is what keeps the two
/// summoned surfaces from disagreeing. Widening the wall from "a read-only
/// prose surface" to "any summoned surface" is one decision to take across
/// both, with this measurement in front of whoever takes it — and it is not a
/// pure widening, because a refusal is only the right answer if the commit does
/// not instead belong in the PANEL'S OWN query.
///
/// The subject is DERIVED — `read_only_surface::walled_doors()`, i.e. every
/// member of the census roster whose gate is the wall — rather than the one
/// door this pin was first written against. That matters: the census grew the
/// roster from three doors to five, and a pin that named `Ime` alone would have
/// gone on reporting a measurement two doors out of date.
#[test]
fn the_text_insertion_doors_are_outside_this_wall_and_that_is_pinned() {
    let _fs = crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC_WITH_IMAGE),
    ));
    let _g = crate::testlock::serial();

    let image = super::read_only_surface::image_span_in(DOC_WITH_IMAGE);
    let walled = super::read_only_surface::walled_doors();
    assert!(
        !walled.is_empty(),
        "the walled roster is empty — no subject"
    );

    for door in walled {
        let mut app = app();
        summon(&mut app, TextField::FindQuery);
        let before = app.document.buffer().text();
        assert!(
            super::read_only_surface::drive(&mut app, door, "字", image),
            "{door:?} is walled but this sweep does not know how to press it"
        );
        assert_ne!(
            app.document.buffer().text(),
            before,
            "{door:?} no longer reaches the document behind the find panel — if that \
             was deliberate, this pin is what should have been updated with it, and \
             the summoned CARD's identical boundary (app::tests::read_only_surface) \
             needs the same decision"
        );
        assert_eq!(
            app.workspace_state.search().map(|s| s.query().to_string()),
            Some(QUERY.to_string()),
            "{door:?}: the panel's own query is untouched by that door either way"
        );
    }
}
