//! ITEM 116d — **ONE ESC ALWAYS LEAVES**, and the footer says what the Back is.
//!
//! The user settled this once for BOTH workspace members on 2026-08-02, exactly
//! as item 114 asked: `Esc` dismisses a summoned workspace from anywhere inside
//! it, and focus moves between its two regions on `Tab` / `Shift-Tab` alone. The
//! rejected arm — Esc unwinds one rung, so leaving History from the comparison
//! takes two presses — was rejected because the comparison is exactly where a
//! reader spends their time, and Esc would then mean two different things
//! depending on where focus sits.
//!
//! That decision costs the footer something, and this file is where the bill is
//! paid. Esc was the taught way back; with Esc leaving, a workspace that does not
//! NAME its Back has no way back a user can find. awl's footer is its only
//! statement of what a key does and there is no accessibility tree behind it
//! (ACCESSIBILITY.md), so "the surface advertises its Back" is an accessibility
//! requirement, not polish.
//!
//! The law below therefore does not check the footer against a string an author
//! typed: it DRIVES the advertised key through the real `apply_transition` seam
//! and asserts the advertised sentence describes what happened — the same shape
//! as `overlay_drive`'s `the_foot_hint_names_what_left_right_actually_do_on_every
//! _settings_row`. It sweeps the sustained roster from `OverlayKind::ALL`, so a
//! third workspace member cannot join without answering it.

use super::*;
use crate::overlay::{Journey, Landing, OverlayKind, OverlayState, TAB_GLYPH};

/// Every kind that is a SUSTAINED surface — the members this decision binds.
/// Derived from the roster rather than hand-listed, and floored below, because
/// the whole point is that the two members answer the same way.
fn sustained_kinds() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| k.sustained())
        .collect()
}

fn card_for(kind: OverlayKind) -> OverlayState {
    match kind {
        OverlayKind::Settings => settings_overlay(),
        OverlayKind::History => {
            let row = |when: &str, id: &str, ts: u64| crate::history::TimelineRow {
                when: when.into(),
                which: String::new(),
                counts: "+1 \u{2212}2".into(),
                id: id.into(),
                timestamp: ts,
                pinned: false,
                name: None,
            };
            OverlayState::new_history(
                vec![
                    row("2 hr ago", "200", 1_700_000_000_000),
                    row("yesterday", "100", 1_699_900_000_000),
                ],
                None,
                None,
            )
        }
        other => panic!(
            "{other:?} is sustained but this law does not know how to build its card — a new \
             workspace member must state what its two regions advertise before it ships"
        ),
    }
}

/// A `Journey` standing on `kind`'s DETAIL stage, reached the way a user reaches
/// it: through the real focus-transfer key, never by writing `detail_focus`
/// behind the lifecycle's back.
fn on_the_detail_stage(kind: OverlayKind) -> Journey {
    let mut journey = Journey::seeded(Some(card_for(kind)));
    settings_drive(&mut journey, &Action::InsertTab);
    assert!(
        journey.card().is_some_and(|o| o.detail_focus),
        "{kind:?}: precondition — Tab must reach the detail stage"
    );
    journey
}

/// Does `hint` advertise a cell reading `glyph label`?
fn advertises(hint: &str, glyph: &str, label: &str) -> bool {
    hint.split(crate::overlay::HINT_SEP)
        .any(|cell| cell == format!("{glyph} {label}"))
}

/// THE DECISION, DRIVEN. On every sustained member's DETAIL stage: one `Esc`
/// leaves outright, and `Tab` / `Shift-Tab` are the Back.
#[test]
fn one_esc_leaves_a_workspace_from_its_detail_stage_on_every_sustained_kind() {
    let _g = crate::testlock::serial();
    let kinds = sustained_kinds();
    assert_eq!(
        kinds.len(),
        2,
        "the sustained roster is Settings + History; a third member must answer this law \
         rather than silently skip it — got {kinds:?}"
    );
    for kind in kinds {
        // ESC LEAVES.
        let mut journey = on_the_detail_stage(kind);
        settings_drive(&mut journey, &Action::Cancel);
        assert!(
            journey.card().is_none(),
            "{kind:?}: one Esc from the detail stage must leave the workspace — a second \
             press would make Esc mean two different things depending on where focus sits"
        );

        // AND SO DOES ESC FROM THE PRIMARY LIST — the invariance is the decision.
        let mut journey = Journey::seeded(Some(card_for(kind)));
        settings_drive(&mut journey, &Action::Cancel);
        assert!(
            journey.card().is_none(),
            "{kind:?}: Esc from the primary list must leave too"
        );

        // TAB AND SHIFT-TAB ARE THE BACK, both ways, without closing.
        for back in [Action::InsertTab, Action::Outdent] {
            let mut journey = on_the_detail_stage(kind);
            settings_drive(&mut journey, &back);
            assert!(
                journey.card().is_some(),
                "{kind:?}: {back:?} must not close the workspace"
            );
            assert!(
                !journey.card().unwrap().detail_focus,
                "{kind:?}: {back:?} must return focus to the primary list — it is the only \
                 way back now that Esc leaves"
            );
        }
    }
}

/// THE FOOTER NAMES THE BACK, AND DOES NOT NAME ESC AS ONE.
///
/// Two halves, both driven rather than pattern-matched against a literal:
///
///   * the detail stage's line carries a `tab back` cell, and pressing that key
///     really does come back;
///   * no line on either stage advertises `esc back`, and pressing `esc` really
///     does leave. A surface that kept the old sentence would be teaching a
///     gesture the lifecycle no longer performs — the exact drift the
///     `range_row_hint` law exists to stop on the other axis.
#[test]
fn the_footer_names_the_back_it_actually_has_on_every_sustained_kind() {
    let _g = crate::testlock::serial();
    let mut graded = 0usize;
    for kind in sustained_kinds() {
        // THE DETAIL STAGE — advertises the Back, and the Back works.
        let journey = on_the_detail_stage(kind);
        let hint = journey.card().unwrap().foot_hint();
        assert!(
            advertises(&hint, TAB_GLYPH, "back"),
            "{kind:?}: the detail stage must NAME its Back — Esc no longer performs one, and \
             the footer is awl's only statement of what a key does. got {hint:?}"
        );
        let mut journey = on_the_detail_stage(kind);
        settings_drive(&mut journey, &Action::InsertTab);
        assert!(
            journey.card().is_some_and(|o| !o.detail_focus),
            "{kind:?}: the advertised `{TAB_GLYPH} back` must be the key that actually goes \
             back"
        );

        // NEITHER STAGE MAY STILL CALL ESC A BACK.
        for (stage, journey) in [
            ("primary", Journey::seeded(Some(card_for(kind)))),
            ("detail", on_the_detail_stage(kind)),
        ] {
            let hint = journey.card().unwrap().foot_hint();
            assert!(
                !advertises(&hint, "esc", "back"),
                "{kind:?} ({stage}): the footer still calls Esc a BACK, but one Esc now \
                 leaves the workspace from either region. got {hint:?}"
            );
            graded += 1;
        }
    }
    assert_eq!(graded, 4, "two members x two stages must each be graded");
}

/// AND THE TABLE AGREES WITH THE KEYBOARD. The lifecycle's own statement of the
/// decision is `landing_of`; this pins the driven outcome to it for both stages
/// of both members, so a future edit cannot fix one and leave the other.
#[test]
fn the_driven_esc_lands_where_the_table_says_for_both_stages() {
    let _g = crate::testlock::serial();
    for kind in sustained_kinds() {
        for detail in [false, true] {
            let mut journey = match detail {
                true => on_the_detail_stage(kind),
                false => Journey::seeded(Some(card_for(kind))),
            };
            let before = journey.state();
            let expected = crate::overlay::landing_of(before, crate::overlay::Event::Cancel);
            assert_eq!(
                expected,
                Landing::Editor,
                "{kind:?} (detail={detail}): the table itself must say a cancel leaves"
            );
            settings_drive(&mut journey, &Action::Cancel);
            assert!(
                journey.card().is_none(),
                "{kind:?} (detail={detail}): and the real keyboard must land there too"
            );
        }
    }
}
