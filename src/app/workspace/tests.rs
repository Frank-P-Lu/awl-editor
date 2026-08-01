//! The ladder laws for [`super::WorkspaceState`] — swept over the WHOLE
//! `rung × search × popover` space, not the reachable corner of it.

use super::*;
use crate::overlay::OverlayKind;

/// A journey standing on `kind`, or on the editor.
fn journey_on(kind: Option<OverlayKind>) -> Journey {
    Journey::seeded(
        kind.map(|kind| OverlayState::new(kind, vec!["README.md".to_string()], vec![], vec![])),
    )
}

/// Build a `WorkspaceState` at an arbitrary cell of the three-fact space.
/// Constructs the private fields directly — the whole point is to reach
/// combinations the public transitions REFUSE to produce, so the ladder is
/// pinned over the entire space rather than over the reachable corner of it.
fn cell(rung: Rung, search: bool, popover: bool) -> WorkspaceState {
    let kind = match rung {
        Rung::Nothing => None,
        Rung::Modal => Some(OverlayKind::Goto),
        Rung::Sustained => Some(OverlayKind::Settings),
    };
    WorkspaceState {
        journey: journey_on(kind),
        search: search.then(|| SearchState::start(0, crate::search::Direction::Forward)),
        popover_summoned: popover,
        tutorial_folder_intent: None,
    }
}

/// Every rung the journey can report. Paired with `layer`'s no-wildcard match:
/// a new `Rung` member fails to compile there and is missing here.
const RUNGS: &[Rung] = &[Rung::Nothing, Rung::Modal, Rung::Sustained];

/// THE LADDER LAW, swept over ALL TWELVE cells of the (journey rung, search,
/// popover) space — not the four single-surface cases anyone would think to
/// write.
///
/// The expectation column is the ladder stated independently of
/// `WorkspaceState::layer`'s own match (highest present rung wins), so this is
/// a real second opinion rather than the implementation restated.
#[test]
fn the_summoned_layer_ladder_resolves_every_combination() {
    let mut seen = std::collections::BTreeSet::new();
    for &rung in RUNGS {
        for search in [false, true] {
            for popover in [false, true] {
                // The ladder, independently derived: take the HIGHEST rung
                // whose fact is present.
                let expect = match rung {
                    Rung::Modal => Layer::Overlay,
                    Rung::Sustained => Layer::Workspace,
                    Rung::Nothing if search => Layer::Search,
                    Rung::Nothing if popover => Layer::Popover,
                    Rung::Nothing => Layer::Editor,
                };
                let ws = cell(rung, search, popover);
                let got = ws.layer();
                assert_eq!(
                    got, expect,
                    "layer() disagrees with the ladder at \
                     (rung={rung:?}, search={search}, popover={popover})"
                );
                seen.insert(got);

                // Every derived predicate must equal the conjunction it
                // replaced, at EVERY cell. A card of EITHER rung owns the
                // keyboard, which is what keeps these byte-identical to the
                // pre-item-173 `overlay.is_some()` answers.
                let card_up = !matches!(rung, Rung::Nothing);
                assert_eq!(
                    ws.overlay_open(),
                    card_up,
                    "overlay_open must equal `a card is up` at ({rung:?}, {search}, {popover})"
                );
                assert_eq!(
                    ws.pickers_clear(),
                    !card_up && !search,
                    "pickers_clear must equal `no card && no search` at \
                     ({rung:?}, {search}, {popover})"
                );
                assert_eq!(
                    ws.popover_holds_attention(),
                    popover && !card_up && !search,
                    "popover_holds_attention must equal the old \
                     `popover_open && overlay.is_none() && search.is_none()` at \
                     ({rung:?}, {search}, {popover})"
                );
                assert_eq!(
                    ws.search_active(),
                    search,
                    "search_active must equal `search.is_some()` at \
                     ({rung:?}, {search}, {popover})"
                );
            }
        }
    }
    // Non-vacuity: the sweep must actually have visited every rung. A ladder
    // law that only ever observed `Editor` would pass every assertion above and
    // guard nothing.
    assert_eq!(
        seen.iter().copied().collect::<Vec<_>>(),
        Layer::ROSTER.to_vec(),
        "the sweep must reach every rung of the ladder"
    );
}

/// The ladder is a total ORDER, and `layer()` always reports the maximum
/// present rung. Stated separately because it is the property item 173's fourth
/// rung had to preserve: inserting a rung between `Overlay` and the editor must
/// not require re-deriving anything.
#[test]
fn the_ladder_is_ordered_lowest_to_highest() {
    assert!(Layer::Editor < Layer::Popover);
    assert!(Layer::Popover < Layer::Search);
    assert!(Layer::Search < Layer::Workspace);
    assert!(Layer::Workspace < Layer::Overlay);
    // And the roster is in that same order, so a variant inserted in the wrong
    // place is caught here rather than silently reordering the ladder.
    let mut sorted = Layer::ROSTER.to_vec();
    sorted.sort();
    assert_eq!(sorted, Layer::ROSTER.to_vec());
}

/// THE FOURTH RUNG IS INHABITED, and by exactly the kinds the lifecycle calls
/// sustained. Swept over the WHOLE `OverlayKind` roster rather than the two the
/// author had in mind, so a kind promoted to a workspace without a conscious
/// decision here fails loudly.
#[test]
fn every_overlay_kind_lands_on_the_rung_its_lifecycle_claims() {
    let mut sustained = Vec::new();
    for kind in OverlayKind::ALL {
        let ws = WorkspaceState {
            journey: journey_on(Some(kind)),
            search: None,
            popover_summoned: false,
            tutorial_folder_intent: None,
        };
        let expect = if kind.sustained() {
            sustained.push(kind);
            Layer::Workspace
        } else {
            Layer::Overlay
        };
        assert_eq!(ws.layer(), expect, "{kind:?} landed on the wrong rung");
    }
    assert_eq!(
        sustained,
        vec![OverlayKind::History, OverlayKind::Settings],
        "the shared workspace is scoped to Version History and Settings — \
         adding a third is a product decision, not a refactor"
    );
}

/// THE SUMMON GATE: the popover bit can never be armed under a picker, no
/// matter what the caller claims about eligibility — and that now includes a
/// SUSTAINED workspace, the rung that did not exist when the gate was written.
#[test]
fn the_popover_cannot_be_summoned_underneath_a_picker() {
    for &rung in RUNGS {
        for search in [false, true] {
            let mut ws = cell(rung, search, false);
            ws.summon_popover(true);
            assert_eq!(
                ws.popover_summon_bit(),
                matches!(rung, Rung::Nothing) && !search,
                "summon_popover(true) armed the bit with (rung={rung:?}, search={search})"
            );
            // An ineligible gesture always dismisses, at every cell.
            ws.summon_popover(false);
            assert!(!ws.popover_summon_bit());
        }
    }
}

/// Dismissing the pickers drops to the editor — or back to a popover that was
/// summoned before a picker shadowed it. Pins the pre-item-172 behaviour (the
/// menu-bar title press cleared overlay + search and left `popover_open` alone)
/// so a future tidy-up cannot change it silently, at BOTH card rungs.
#[test]
fn dismissing_pickers_reveals_whatever_was_underneath() {
    for &rung in &[Rung::Modal, Rung::Sustained] {
        let mut ws = cell(rung, true, false);
        ws.dismiss_pickers();
        assert_eq!(ws.layer(), Layer::Editor, "from {rung:?}");

        let mut ws = cell(rung, true, true);
        ws.dismiss_pickers();
        assert_eq!(
            ws.layer(),
            Layer::Popover,
            "dismiss_pickers must not touch the popover bit (from {rung:?})"
        );
    }
}

/// A buffer swap closes the panel and leaves the card alone — the two call
/// sites (`load_path`, `start_fresh_document`) used to write `self.search = None`
/// directly.
#[test]
fn closing_the_search_panel_leaves_the_picker_alone() {
    let mut ws = cell(Rung::Modal, true, false);
    ws.close_search();
    assert!(!ws.search_active());
    assert!(ws.overlay_open());
}
