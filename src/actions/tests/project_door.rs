//! THE SWITCH-PROJECT DOOR — the flat picker's one reach past the workspace's
//! direct children.
//!
//! The flat picker is flat on purpose: `↵` on a folder switches to it, so a
//! grandchild can never enrol and a Backspace can never walk out of the
//! workspace. That leaves a real project one level down structurally
//! unreachable, and the door is what answers it: a terminal `Browse for
//! folder…` row that DESCENDS into a folder navigator
//! ([`OverlayKind::ProjectBrowse`]) with the destination-navigator grammar the
//! move/export pickers already ship, whose accept switches the project.
//!
//! Every law here drives the REAL `apply_transition` seam and the REAL
//! `overlay::browse_level` builder over a filesystem seam, so what they assert
//! is what `--keys` does.

use super::super::*;
use super::{drive_bt, proj_tree};
use crate::overlay::OverlayKind;

/// The production level builder, wired the way the live App and the headless
/// replay wire it: `ws` is BOTH the active root and the configured workspace.
fn ws_browse_to(
    ws: &std::path::Path,
) -> impl FnMut(OverlayKind, Option<String>) -> Option<crate::overlay::OverlayState> + '_ {
    move |kind, rel| crate::overlay::browse_level(kind, rel, ws, Some(ws), &[])
}

/// The flat picker as a real summon produces it — `Action::OpenProject` through
/// `apply_transition`, never a hand-built card, so the door is only ever here
/// because the shipped seam put it here.
fn opened_flat_picker(
    ws: &std::path::Path,
    browse_to: &mut dyn FnMut(OverlayKind, Option<String>) -> Option<crate::overlay::OverlayState>,
) -> crate::overlay::Journey {
    let _ = ws;
    let mut journey = crate::overlay::Journey::default();
    let mut accept = None;
    drive_bt(&mut journey, &mut accept, browse_to, &Action::OpenProject);
    assert_eq!(
        journey.card().map(|c| c.kind),
        Some(OverlayKind::Project),
        "OpenProject summons the flat switch-project picker"
    );
    journey
}

/// Move the highlight onto the door row through the real selection action.
fn select_door(journey: &mut crate::overlay::Journey) {
    let mut accept = None;
    let mut none = |_k: OverlayKind, _r: Option<String>| None;
    drive_bt(journey, &mut accept, &mut none, &Action::LineEnd);
    assert!(
        journey.card().unwrap().selected_is_browse_door(),
        "the door is the LAST row, so End lands on it: {:?}",
        journey.card().unwrap().item_strings()
    );
}

/// THE ELLIPSIS IS A PROMISE (`menu::ellipsis_law`'s rule, read on a picker
/// row): `Browse for folder…` must open a further surface rather than act on
/// the spot. Both halves are asserted from one drive — the label promises, and
/// the real `↵` dispatch delivers a card and NO accept.
///
/// MUTATION TARGET: make the door row switch to the workspace instead of
/// descending (drop the `selected_is_browse_door` arm in
/// `accept_path_overlay`) and this fails by name.
#[test]
fn the_browse_door_row_ends_in_an_ellipsis_and_really_opens_a_surface() {
    let (ws, _fs) = proj_tree();
    let mut browse_to = ws_browse_to(&ws);
    let mut journey = opened_flat_picker(&ws, &mut browse_to);

    let items = journey.card().unwrap().item_strings();
    assert_eq!(
        items.last().map(String::as_str),
        Some(OverlayKind::BROWSE_DOOR_LABEL),
        "the door is the picker's terminal row: {items:?}"
    );
    assert!(
        OverlayKind::BROWSE_DOOR_LABEL.ends_with('…'),
        "the label promises a surface"
    );

    select_door(&mut journey);
    let mut accept = None;
    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Newline);
    assert_eq!(
        journey.card().map(|c| c.kind),
        Some(OverlayKind::ProjectBrowse),
        "the ellipsis promised a surface, so ↵ must OPEN one"
    );
    assert_eq!(
        accept, None,
        "and it must not have switched anything on the way"
    );
    assert_eq!(
        journey.parked_kind(),
        Some(OverlayKind::Project),
        "the flat picker is PARKED beneath, not replaced — that is what makes \
         Esc come back to it"
    );
}

/// **THE DEFECT ITEM 411 EXISTS FOR.** With the workspace at `ws`, a project at
/// `ws/child-a/sub` is one level too deep for the flat roster and cannot be
/// reached by any number of `↵`s in it. Through the door it is three keys away,
/// and the switch names the GRANDCHILD's own absolute path.
///
/// MUTATION TARGET: make `browse_level`'s `ProjectBrowse` arm refuse to build a
/// level below the workspace, or drop `ProjectBrowse` from
/// `is_folder_destination` (so `→` stops descending), and this fails by name.
#[test]
fn the_browse_door_reaches_a_nested_project_and_switches_to_it() {
    let (ws, _fs) = proj_tree();
    let mut browse_to = ws_browse_to(&ws);
    let mut journey = opened_flat_picker(&ws, &mut browse_to);
    let mut accept = None;

    // The nested project is structurally absent from the flat roster — the
    // premise this door answers, asserted rather than assumed.
    let flat = journey.card().unwrap().item_strings();
    assert!(
        !flat.iter().any(|s| s.contains("sub")),
        "the flat picker cannot see a grandchild: {flat:?}"
    );

    select_door(&mut journey);
    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Newline);
    assert_eq!(
        journey.card().unwrap().selected_value(),
        Some("child-a"),
        "the navigator opens at the workspace"
    );
    // `→` descends, exactly as it does in the move/export destination pickers.
    drive_bt(
        &mut journey,
        &mut accept,
        &mut browse_to,
        &Action::ForwardChar,
    );
    assert_eq!(
        journey.card().unwrap().browse_dir.as_deref(),
        Some(ws.join("child-a").to_string_lossy().as_ref()),
        "one level down"
    );
    assert_eq!(journey.card().unwrap().selected_value(), Some("sub"));

    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Newline);
    assert_eq!(
        accept,
        Some((
            OverlayKind::Project,
            ws.join("child-a/sub").to_string_lossy().to_string()
        )),
        "the nested project is switched to — and it arrives under Project's own \
         accept, the one owner of the switch whichever door reached it"
    );
    assert!(
        journey.card().is_none(),
        "going somewhere ends the whole journey, parked parent included"
    );
}

/// AND BACK AGAIN. Esc in the navigator resumes the flat picker at the exact row
/// it was left on — the door — with the door still on it. The resume rebuilds
/// the parent from the disk (`resume_rebuild`), which is precisely where a
/// feature-owned row can go missing.
///
/// MUTATION TARGET: hand `Journey::cancel` the plain `make_overlay` again (which
/// answers `None` for every kind built from a directory level) and this fails by
/// name: the card is gone entirely.
#[test]
fn esc_in_the_navigator_returns_to_the_flat_picker_on_its_door_row() {
    let (ws, _fs) = proj_tree();
    let mut browse_to = ws_browse_to(&ws);
    let mut journey = opened_flat_picker(&ws, &mut browse_to);
    let mut accept = None;

    select_door(&mut journey);
    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Newline);
    drive_bt(
        &mut journey,
        &mut accept,
        &mut browse_to,
        &Action::ForwardChar,
    );
    assert_eq!(
        journey.card().map(|c| c.kind),
        Some(OverlayKind::ProjectBrowse)
    );

    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Cancel);
    let card = journey.card().expect("the flat picker comes back");
    assert_eq!(card.kind, OverlayKind::Project, "back on the picker");
    assert!(
        card.selected_is_browse_door(),
        "at the exact row it was left on: {:?} selected {}",
        card.item_strings(),
        card.selected
    );
    assert_eq!(
        card.item_strings().last().map(String::as_str),
        Some(OverlayKind::BROWSE_DOOR_LABEL),
        "and the rebuilt level still carries the door — it belongs to the \
         feature, not to the directory listing"
    );
    assert_eq!(accept, None, "coming back commits nothing");
}

/// THE DOOR'S FLOOR IS THE WORKSPACE. Both ascend gestures the navigator honours
/// stand still at the top level, and they do so because `browse_level` refuses
/// to BUILD a level outside the workspace — one gate, not a boundary test per
/// gesture.
///
/// MUTATION TARGET: delete the `starts_with(ws)` refusal in `browse_level`'s
/// `ProjectBrowse` arm and this fails by name, reporting the directory it
/// escaped to.
#[test]
fn the_browse_navigator_cannot_walk_above_the_workspace() {
    let (ws, _fs) = proj_tree();
    let mut browse_to = ws_browse_to(&ws);
    let mut journey = opened_flat_picker(&ws, &mut browse_to);
    let mut accept = None;
    select_door(&mut journey);
    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Newline);

    for action in [Action::BackwardChar, Action::DeleteBackward] {
        drive_bt(&mut journey, &mut accept, &mut browse_to, &action);
        assert_eq!(
            journey.card().unwrap().browse_dir.as_deref(),
            Some(ws.to_string_lossy().as_ref()),
            "{action:?} at the top level must stand still, not leave the workspace"
        );
    }
    // …while the same gesture one level DOWN really does ascend, so the law
    // above is about the floor and not about a navigator that never moves.
    drive_bt(
        &mut journey,
        &mut accept,
        &mut browse_to,
        &Action::ForwardChar,
    );
    assert_eq!(
        journey.card().unwrap().browse_dir.as_deref(),
        Some(ws.join("child-a").to_string_lossy().as_ref()),
    );
    drive_bt(
        &mut journey,
        &mut accept,
        &mut browse_to,
        &Action::BackwardChar,
    );
    assert_eq!(
        journey.card().unwrap().browse_dir.as_deref(),
        Some(ws.to_string_lossy().as_ref()),
        "← below the top ascends for real"
    );
}

/// A TYPED NAME IS A FILTER HERE, NEVER A NEW FOLDER. The move/export
/// destinations accept a name that does not exist yet — they create it. There is
/// no project to switch to in a folder that isn't there, so this navigator's
/// accept falls back to the level you are standing in
/// (`overlay_nav::dest_value`'s `allow_new`).
///
/// MUTATION TARGET: pass `true` for `allow_new` at the `ProjectBrowse` accept
/// and this fails by name, reporting the invented path.
#[test]
fn a_typed_query_in_the_browse_navigator_cannot_invent_a_project() {
    let (ws, _fs) = proj_tree();
    let mut browse_to = ws_browse_to(&ws);
    let mut journey = opened_flat_picker(&ws, &mut browse_to);
    let mut accept = None;
    select_door(&mut journey);
    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Newline);

    for c in "zzzz".chars() {
        drive_bt(
            &mut journey,
            &mut accept,
            &mut browse_to,
            &Action::InsertChar(c),
        );
    }
    assert!(
        journey.card().unwrap().item_strings().is_empty(),
        "the query matches no folder here"
    );
    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Newline);
    assert_eq!(
        accept,
        Some((OverlayKind::Project, ws.to_string_lossy().to_string())),
        "the folder you are standing in is the honest answer; a switch to \
         `<ws>/zzzz` would name a project that does not exist"
    );
}

/// THE PARTITION. The door belongs to the FLAT switch-project picker, and the
/// Settings folder-VALUE picker draws from the same `OverlayKind::Project` card
/// shape — it already walks the whole tree with `→`/`⌫`, and a descend from it
/// would un-park the Settings surface whose config key it is filling in. So the
/// LEVEL BUILDER never produces a door and only the flat picker's summon seams
/// attach one.
///
/// MUTATION TARGET: move `attach_browse_door` into `OverlayState::new_project`
/// (the tempting simplification) and this fails by name.
#[test]
fn only_the_flat_pickers_summon_attaches_the_door_never_the_level_builder() {
    let (ws, _fs) = proj_tree();
    let bare = crate::overlay::browse_level(OverlayKind::Project, None, &ws, Some(&ws), &[])
        .expect("a level at the workspace");
    assert!(
        !bare
            .item_strings()
            .iter()
            .any(|s| s == OverlayKind::BROWSE_DOOR_LABEL),
        "the level a Settings folder-VALUE descend builds carries no door: {:?}",
        bare.item_strings()
    );

    let mut browse_to = ws_browse_to(&ws);
    let opened = opened_flat_picker(&ws, &mut browse_to);
    assert!(
        opened
            .card()
            .unwrap()
            .item_strings()
            .iter()
            .any(|s| s == OverlayKind::BROWSE_DOOR_LABEL),
        "…and the flat picker's own summon does"
    );
}

/// THE FOOTER COMES FROM THE ONE OWNER. Whether a picker ascends is a property
/// of the JOURNEY, not of the kind — `Journey::foot_hint` is where that was
/// settled, and this door is a third journey through the same machinery. Its
/// line must be the navigator's real grammar, and standing on the door row must
/// not change the flat picker's own line.
///
/// MUTATION TARGET: give `OverlayKind::ProjectBrowse` the flat picker's
/// `kind_actions` arm (an easy copy/paste when a kind is added) and this fails by
/// name.
#[test]
fn the_browse_navigators_footer_routes_through_the_journeys_owner() {
    let (ws, _fs) = proj_tree();
    let mut browse_to = ws_browse_to(&ws);
    let mut journey = opened_flat_picker(&ws, &mut browse_to);
    let mut accept = None;

    select_door(&mut journey);
    let flat_hint = journey.foot_hint();
    assert!(
        flat_hint.contains("\u{21B5} select") && !flat_hint.contains('\u{232B}'),
        "standing on the door does not change the flat picker's own line: {flat_hint}"
    );

    drive_bt(&mut journey, &mut accept, &mut browse_to, &Action::Newline);
    let hint = journey.foot_hint();
    assert_eq!(
        hint,
        journey.card().unwrap().foot_hint(),
        "one owner: the journey's answer IS the card's, this kind needing no \
         bind to disambiguate it"
    );
    for cell in ["\u{21B5} switch here", "\u{2192} open", "\u{2190} up"] {
        assert!(
            hint.contains(cell),
            "the navigator's line teaches {cell}: {hint}"
        );
    }
}
