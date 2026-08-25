//! THE SUMMONED-UI LIFECYCLE AT THE REAL `apply_transition` SEAM.
//!
//! `overlay::journey::tests` pins the table and the transitions in isolation.
//! These drive the same lifecycle through the one seam the live App and the
//! headless `--keys` replay share, and assert the thing the isolated laws
//! cannot see: that the DOCUMENT is untouched by the whole journey.

use super::super::*;
use super::{settings_drive, settings_overlay};
use crate::buffer::Buffer;
use crate::overlay::{Journey, OverlayKind};

/// Everything about the editor that a summoned journey must not disturb. The
/// buffer's BYTES (`disk_bytes`, the exact thing a save would write — not the
/// rope's normalized text, which would hide an EOL change) plus the caret,
/// the selection and the undo depth.
#[derive(Debug, PartialEq)]
struct EditorState {
    bytes: Vec<u8>,
    cursor: usize,
    selection: Option<(usize, usize)>,
    eol: crate::buffer::Eol,
    can_undo: bool,
}

impl EditorState {
    fn of(buffer: &Buffer) -> Self {
        Self {
            bytes: buffer.disk_bytes(),
            cursor: buffer.cursor_char(),
            selection: buffer.selection_range(),
            eol: buffer.eol(),
            can_undo: buffer.can_undo(),
        }
    }
}

/// A document with a caret parked mid-word and a live selection, so a byte
/// comparison has something to lose.
fn seeded_buffer() -> Buffer {
    let mut buffer = Buffer::from_str("# Notes\n\nthe quick brown fox\nover the lazy dog\n");
    buffer.set_cursor(14);
    buffer.select_range(10, 14);
    buffer
}

/// Drive one action against a real buffer through `apply_transition`, with the
/// Settings/Caret/Theme rebuild hooks the live App supplies.
fn drive_on(journey: &mut Journey, buffer: &mut Buffer, action: &Action) -> Effect {
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut make_overlay = |k: OverlayKind| match k {
        OverlayKind::Settings => Some(settings_overlay()),
        OverlayKind::Caret => Some(OverlayState::new_caret(crate::caret::mode())),
        OverlayKind::Theme => Some(OverlayState::new_theme(
            crate::theme::THEMES.iter().map(|t| t.name.into()).collect(),
            crate::theme::active_index(),
        )),
        _ => None,
    };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| None;
    let mut ctx = ActionCtx {
        buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    apply_transition(&mut ctx, action, false).primary()
}

/// ENTER/EXIT BYTE-IDENTITY. The whole journey — summon a workspace, filter it,
/// move within it, suspend into a child audition, audition another look, revert,
/// and leave — must return the DOCUMENT exactly as it was found, down to the
/// bytes a save would write.
///
/// A Settings workspace may own its rows and
/// nothing else. It sweeps the WHOLE journey rather than a bare open/close,
/// because the open/close pair is the case anyone would think to test and the
/// suspend/resume pair is the one that touches state.
#[test]
fn the_whole_journey_leaves_the_document_byte_identical() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::caret::clear_override();

    let mut buffer = seeded_buffer();
    let before = EditorState::of(&buffer);
    let mut journey = Journey::default();

    // Enter the workspace, filter, and move within it.
    drive_on(&mut journey, &mut buffer, &Action::OpenSettingsMenu);
    assert_eq!(journey.card().unwrap().kind, OverlayKind::Settings);
    for c in "caret".chars() {
        drive_on(&mut journey, &mut buffer, &Action::InsertChar(c));
    }
    // Suspend into the child audition and audition a different look.
    drive_on(&mut journey, &mut buffer, &Action::Newline);
    assert_eq!(journey.card().unwrap().kind, OverlayKind::Caret);
    assert_eq!(journey.parked_kind(), Some(OverlayKind::Settings));
    drive_on(&mut journey, &mut buffer, &Action::NextLine);
    // Revert the audition; the workspace resumes ON ITS CONTENT PANE — a child
    // is opened from a row, so returning to the rail would lose the place the
    // whole suspend/resume pair exists to keep.
    drive_on(&mut journey, &mut buffer, &Action::Cancel);
    assert_eq!(journey.card().unwrap().kind, OverlayKind::Settings);
    assert!(
        journey.card().unwrap().detail_focus,
        "resumed in the content pane, where the row it descended from lives"
    );
    // And leave: ONE Esc, from the content pane (user decision 2026-08-02 — one
    // Esc always leaves a workspace, from either of its regions; `Tab` is the
    // Back, and the footer names it).
    drive_on(&mut journey, &mut buffer, &Action::Cancel);
    assert!(
        journey.card().is_none(),
        "one Esc off the content pane ended the journey in the editor"
    );

    assert_eq!(
        EditorState::of(&buffer),
        before,
        "a summoned journey must not touch the document"
    );
    crate::caret::clear_override();
}

/// THE TYPED CHARS GO TO THE CARD, NEVER THE ROPE — the half of byte-identity a
/// closed buffer comparison could pass by accident if the filter never reached
/// the card at all. Asserts BOTH halves: the rope is untouched AND the query
/// actually took the characters.
#[test]
fn filtering_a_workspace_types_into_the_card_not_the_rope() {
    let _g = crate::testlock::serial();
    let mut buffer = seeded_buffer();
    let before = EditorState::of(&buffer);
    let mut journey = Journey::default();
    drive_on(&mut journey, &mut buffer, &Action::OpenSettingsMenu);
    for c in "page".chars() {
        drive_on(&mut journey, &mut buffer, &Action::InsertChar(c));
    }
    assert_eq!(journey.card().unwrap().query.text(), "page");
    assert_eq!(EditorState::of(&buffer), before);
}

/// POSITION RESTORATION THROUGH THE REAL SEAM. The isolated law drives
/// `Journey` directly; this one goes through `apply_transition` with the real
/// Settings corpus, so a regression in the ACTION plumbing (not just the
/// lifecycle) is caught too.
#[test]
fn a_settings_child_returns_to_the_settings_row_it_was_opened_from() {
    let _g = crate::testlock::serial();
    crate::caret::clear_override();
    let mut journey = Journey::seeded(Some(settings_overlay()));
    for c in "caret".chars() {
        settings_drive(&mut journey, &Action::InsertChar(c));
    }
    let row = journey
        .card()
        .unwrap()
        .selected_value()
        .unwrap()
        .to_string();
    assert_eq!(row, "Caret style");

    settings_drive(&mut journey, &Action::Newline); // descend into the Caret picker
    assert_eq!(journey.card().unwrap().kind, OverlayKind::Caret);
    settings_drive(&mut journey, &Action::Cancel); // revert + resume

    let back = journey.card().expect("the workspace resumed");
    assert_eq!(back.kind, OverlayKind::Settings);
    assert_eq!(
        back.selected_value(),
        Some(row.as_str()),
        "resumed on the row the child was opened from, not on row 0"
    );
    assert_eq!(
        back.query.text(),
        "caret",
        "and with the filter that found it"
    );
    crate::caret::clear_override();
}

/// THE SAME LAW FOR THE KEYMAP PICKER — unlike Caret (which auditions live
/// on move, so Esc must REVERT the process-global) the Keymap picker never
/// touches anything until Enter, so Esc is a pure descend/resume with
/// nothing to undo. Only meaningful on `Convention::Linux` (the row is
/// hidden on `Convention::Mac` — `settings::row_available_on`);
/// `Convention` is process-frozen, so this branches on the ambient value
/// rather than forcing one — the same reason the picker's live-App proof
/// does (`run::live_app::tests::
/// a_live_app_capture_photographs_a_keymap_pick_an_ordinary_capture_cannot_see`).
#[test]
fn a_keymap_child_esc_resumes_the_settings_row_it_was_opened_from() {
    if crate::convention::Convention::current() != crate::convention::Convention::Linux {
        return;
    }
    let _g = crate::testlock::serial();
    let mut journey = Journey::seeded(Some(settings_overlay()));
    for c in "keym".chars() {
        settings_drive(&mut journey, &Action::InsertChar(c));
    }
    let row = journey
        .card()
        .unwrap()
        .selected_value()
        .unwrap()
        .to_string();
    assert_eq!(row, "Keymap");

    settings_drive(&mut journey, &Action::Newline); // descend into the Keymap picker
    assert_eq!(journey.card().unwrap().kind, OverlayKind::Keymap);
    settings_drive(&mut journey, &Action::Cancel); // resume, nothing to revert

    let back = journey.card().expect("the workspace resumed");
    assert_eq!(back.kind, OverlayKind::Settings);
    assert_eq!(
        back.selected_value(),
        Some(row.as_str()),
        "resumed on the row the child was opened from, not on row 0"
    );
    assert_eq!(
        back.query.text(),
        "keym",
        "and with the filter that found it"
    );
}

/// A SETTINGS TOGGLE KEEPS THE WORKSPACE OPEN; the same row reached from the
/// COMMAND PALETTE closes it. One dispatcher, one table, two outcomes — never
/// a `close_on_toggle` boolean the caller has to pass correctly.
#[test]
fn a_toggle_keeps_a_workspace_and_completes_a_launcher() {
    let _g = crate::testlock::serial();
    let mut journey = Journey::seeded(Some(settings_overlay()));
    for c in "typewriter".chars() {
        settings_drive(&mut journey, &Action::InsertChar(c));
    }
    let eff = settings_drive(&mut journey, &Action::Newline);
    assert!(
        matches!(eff, Effect::SettingToggle { .. }),
        "the toggle fired: {eff:?}"
    );
    assert_eq!(
        journey.card().map(|o| o.kind),
        Some(OverlayKind::Settings),
        "a workspace keeps you configuring"
    );
}
