//! **THE KEY THAT OPENS A WORKSPACE'S CONTENT IS THE KEY THAT CLOSES IT.**
//!
//! Reported live on macOS: in the Settings workspace `→` moved focus from the
//! category rail into the rows, and `←` did not come back — it cycled the
//! category lens backwards instead, moving a highlight in the region the user
//! had just left while focus stayed put. `→` opened a door `←` could not close.
//!
//! A two-region keyboard owes its user that pair, so the law here is a
//! SYMMETRY: wherever `→` moves focus, `←` from the destination returns to the
//! origin — the same stage, the same rail position, the same query — or the
//! state NAMES why not, in the footer cell it devotes to that axis.
//!
//! Two things this file is deliberately built to survive:
//!
//!   * **A symmetry law is satisfiable by `→` never moving focus at all.** Every
//!     cell therefore carries a PRESENCE FLOOR: the sweep asserts `→` really did
//!     move focus before it asserts anything about `←`, counts the cells that
//!     did, and fails if none did.
//!   * **An enrolment pinned to a named member sweeps whatever that member
//!     happens to be.** The roster comes from
//!     [`crate::overlay::OverlayKind::workspace_shape`] — every kind DRAWN as a
//!     workspace — and the failure messages name what enrolled, so a shape whose
//!     `→` silently stopped being a door reads as a changed enrolment rather
//!     than as a quiet pass.
//!
//! The NAMED exception — a Range row's value rail, which owns `←/→` and says so
//! — is graded against the footer on every visible settings row by
//! `overlay_drive::the_foot_hint_names_what_left_right_actually_do_on_every_settings_row`.

use super::workspace_esc::card_for;
use super::*;
use crate::overlay::{Journey, OverlayKind, OverlayState, Surface};

/// Every kind DRAWN as a summoned workspace, derived from the shape owner
/// itself. Not `sustained()` (a near-neighbour that answers a different
/// question) and not a hand-list: a kind that starts or stops being drawn as a
/// workspace changes this roster the day it lands.
fn workspace_kinds() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| k.workspace_shape().is_some())
        .collect()
}

/// How many positions this card's PRIMARY list has — the rail's categories, or
/// the timeline's rows. Routed through `rows_are_primary`, the one fact every
/// consumer reduces to, so a third shape cannot arrive with a private answer.
fn primary_positions(card: &OverlayState) -> usize {
    let rows_primary = card
        .workspace_shape()
        .expect("only workspace kinds reach here")
        .rows_are_primary();
    match rows_primary {
        true => card.items.len(),
        false => card.lens_strip().len(),
    }
}

/// Where a journey is standing, as the three facts a return has to restore:
/// which stage holds focus, which rail category is showing, which row is
/// selected — plus the live query, which no focus move may disturb.
#[derive(Debug, PartialEq, Eq)]
struct Standing {
    surface: Option<Surface>,
    lens: usize,
    selected: usize,
    query: String,
}

fn standing(journey: &Journey) -> Standing {
    let card = journey.card().expect("the workspace is up");
    Standing {
        surface: match journey.state() {
            crate::overlay::State::Editor => None,
            crate::overlay::State::Summoned { surface, .. } => Some(surface),
        },
        lens: card.facet_lens,
        selected: card.selected,
        query: card.query.text().to_string(),
    }
}

/// A query built from the card's OWN first row, so every member gets a filter
/// that really narrows and really matches — rather than a literal that happens
/// to hit one kind's corpus and silently empties another's.
fn self_filtering_query(card: &OverlayState) -> String {
    card.item_strings()
        .first()
        .map(|s| s.chars().take(3).flat_map(char::to_lowercase).collect())
        .unwrap_or_default()
}

/// A journey standing on `kind`'s PRIMARY list with `query` typed and the
/// primary selection stepped `steps` times — all through the real seam. Typing
/// on a `RailOverRows` workspace hands focus to the rows (that is its own law),
/// so the helper walks back with the focus key before it reports.
fn on_the_primary_list(kind: OverlayKind, query: &str, steps: usize) -> Journey {
    let mut journey = Journey::seeded(Some(card_for(kind)));
    for c in query.chars() {
        settings_drive(&mut journey, &Action::InsertChar(c));
    }
    if journey.card().is_some_and(|o| o.detail_focus) {
        settings_drive(&mut journey, &Action::InsertTab);
    }
    assert!(
        journey.card().is_some_and(|o| !o.detail_focus),
        "{kind:?}: precondition — the sweep starts on the primary list"
    );
    for _ in 0..steps {
        settings_drive(&mut journey, &Action::NextLine);
    }
    journey
}

/// **THE HEADLINE.** Over every workspace kind × every primary-list position ×
/// query-filtered and not: if `→` moved focus into the content, `←` from there
/// puts it back exactly where it started.
#[test]
fn left_closes_every_door_right_opens_on_every_workspace_stage() {
    let _g = crate::testlock::serial();
    let kinds = workspace_kinds();
    assert!(
        !kinds.is_empty(),
        "the workspace roster is empty — `workspace_shape` enrolled nothing, so this law \
         swept nothing"
    );

    // Which kinds `→` is a door on, and which it is not. Both are reported,
    // because a kind silently leaving the first set is exactly the regression
    // this law is named for and it would otherwise read as a pass.
    let mut doors: Vec<OverlayKind> = Vec::new();
    let mut not_doors: Vec<OverlayKind> = Vec::new();
    let mut cells = 0usize;

    for kind in &kinds {
        let kind = *kind;
        let queries = ["".to_string(), self_filtering_query(&card_for(kind))];
        let mut moved_here = 0usize;
        let mut inert_here = 0usize;
        for query in &queries {
            let positions = primary_positions(
                on_the_primary_list(kind, query, 0)
                    .card()
                    .expect("the workspace is up"),
            );
            assert!(
                positions > 0,
                "{kind:?} (query {query:?}): a primary list with no positions sweeps nothing"
            );
            for step in 0..positions {
                cells += 1;
                let mut journey = on_the_primary_list(kind, query, step);
                let origin = standing(&journey);

                // PRESENCE FLOOR. `→` has to have really moved focus, or the
                // symmetry below is a claim about nothing.
                settings_drive(&mut journey, &Action::ForwardChar);
                let after_right = standing(&journey);
                if after_right.surface != Some(Surface::WorkspaceDetail) {
                    inert_here += 1;
                    continue;
                }
                moved_here += 1;

                // AND `←` CLOSES IT — back to the same stage, the same category,
                // the same row, the same query.
                settings_drive(&mut journey, &Action::BackwardChar);
                assert_eq!(
                    standing(&journey),
                    origin,
                    "{kind:?} (query {query:?}, primary position {step}): `→` moved focus into \
                     the content, so `←` must put it back where it came from. It did not — \
                     which is the reported defect: the door opens and will not close. \
                     After `→` it stood at {after_right:?}"
                );
            }
        }
        // The enrolment may not be POSITION-dependent: a kind whose `→` is a
        // door from some rows and not others is a third behaviour hiding inside
        // a two-way law.
        assert!(
            moved_here == 0 || inert_here == 0,
            "{kind:?}: `→` moved focus on {moved_here} primary positions and not on \
             {inert_here} others — the door has to be the same door everywhere, or the \
             footer has to say which rows have one"
        );
        match moved_here > 0 {
            true => doors.push(kind),
            false => not_doors.push(kind),
        }
    }

    assert!(
        !doors.is_empty(),
        "no workspace kind's `→` moved focus anywhere, so the symmetry above asserted \
         nothing at all. Roster {kinds:?}; `→` inert on {not_doors:?}"
    );
    assert!(
        cells >= kinds.len() * 2,
        "the sweep graded only {cells} cells across {} kinds x 2 query states — it is not \
         reaching the primary positions it claims to",
        kinds.len()
    );
}

/// **AND THE WAY BACK EXISTS EVEN WHERE `→` IS NOT THE DOOR.** Reached through
/// the focus key, `←` comes back from a workspace's detail stage on every member
/// — or the stage NAMES what owns the axis instead, in the footer cell it gives
/// it, and driving `←` does that named thing rather than nothing.
///
/// This is the half the symmetry law above cannot reach: a shape whose `→` never
/// moves focus enrols in nothing there, and would then be free to leave `←`
/// inert on a stage a reader is standing on.
#[test]
fn left_returns_from_every_workspace_detail_stage_or_the_footer_names_its_owner() {
    let _g = crate::testlock::serial();
    let mut graded = 0usize;
    let mut named_exceptions = 0usize;
    for kind in workspace_kinds() {
        let mut journey = Journey::seeded(Some(card_for(kind)));
        settings_drive(&mut journey, &Action::InsertTab);
        let card = journey.card().expect("the workspace is up");
        assert!(
            card.detail_focus,
            "{kind:?}: precondition — the focus key must reach the detail stage"
        );
        let hint = card.foot_hint();
        let owns_axis = !card.detail_left_returns();
        let advertised = hint
            .split(crate::overlay::HINT_SEP)
            .any(|cell| cell.starts_with(crate::overlay::ARROWS_LR));
        assert_eq!(
            owns_axis, advertised,
            "{kind:?}: the detail stage must carry a `←/→` footer cell EXACTLY when something \
             on it owns that axis instead of the region seam — owner={owns_axis}, \
             advertised={advertised}, line {hint:?}"
        );
        graded += 1;
        if owns_axis {
            named_exceptions += 1;
            continue;
        }
        settings_drive(&mut journey, &Action::BackwardChar);
        assert!(
            journey.card().is_some_and(|o| !o.detail_focus),
            "{kind:?}: `←` must come back from the detail stage — the footer names no other \
             owner for that key here, so an inert `←` is a key that does nothing on a stage \
             a user is standing on"
        );
        assert!(
            journey.card().is_some(),
            "{kind:?}: and coming back is not leaving — Esc is what leaves"
        );
    }
    assert_eq!(
        graded,
        workspace_kinds().len(),
        "every workspace member must be graded"
    );
    assert!(
        graded > named_exceptions,
        "every workspace member handed its `←/→` axis to a row control, so the return path \
         itself was never driven — {named_exceptions} of {graded} were exceptions"
    );
}

/// **`→` HAS NOTHING TO ITS RIGHT ON THE CONTENT, AND — WHERE THE PRIMARY LIST
/// IS A RAIL — `←` HAS NOTHING TO ITS LEFT.** The far halves of the same axis:
/// neither may fall through to the picker's lens cycle and step the rail
/// sideways behind the user's back, which is what made the reported defect worse
/// than an inert key — the category changed while focus stayed put.
///
/// The primary-list half is asked only of a workspace whose primary list is a
/// RAIL, through the one owner that says which those are: where the primary list
/// is a row list, `←/→` are the picker's own prev/next-or-lens grammar and this
/// law has no claim on them.
#[test]
fn the_far_side_of_each_region_leaves_the_rail_alone() {
    let _g = crate::testlock::serial();
    let mut rails = 0usize;
    let mut contents = 0usize;
    for kind in workspace_kinds() {
        // ON A RAIL PRIMARY LIST: `←` is inert.
        let rows_primary = kind
            .workspace_shape()
            .expect("enrolled from the shape owner")
            .rows_are_primary();
        if !rows_primary {
            rails += 1;
            let mut journey = on_the_primary_list(kind, "", 1);
            let before = standing(&journey);
            settings_drive(&mut journey, &Action::BackwardChar);
            assert_eq!(
                standing(&journey),
                before,
                "{kind:?}: there is nothing to the left of the rail, so `←` must not move it \
                 sideways as well as vertically"
            );
        }

        // ON THE DETAIL STAGE: `→` is inert, unless a row control owns the axis.
        let mut journey = Journey::seeded(Some(card_for(kind)));
        settings_drive(&mut journey, &Action::InsertTab);
        if !journey
            .card()
            .is_some_and(crate::overlay::OverlayState::detail_left_returns)
        {
            continue;
        }
        contents += 1;
        let before = standing(&journey);
        settings_drive(&mut journey, &Action::ForwardChar);
        assert_eq!(
            standing(&journey),
            before,
            "{kind:?}: there is nothing to the right of the content, so `→` must not cycle \
             the category rail from a region that does not hold focus"
        );
    }
    assert!(
        rails > 0 && contents > 0,
        "the sweep graded {rails} rail primary lists and {contents} content stages — both \
         halves of the axis have to be reached, or one of them is unasserted"
    );
}
