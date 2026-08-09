//! **ONE ESC ALWAYS LEAVES**, and the footer says what the Back is.
//!
//! The user settled this once for BOTH workspace members on 2026-08-02: `Esc`
//! dismisses a summoned workspace from anywhere inside it, and focus moves
//! between its two regions on `Tab` / `Shift-Tab` alone. The
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
use crate::overlay::workspace::TAB_GLYPH;
use crate::overlay::{Journey, Landing, OverlayKind, OverlayState};

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
        // The conflict workspace: its primary column is the three read-only
        // views and its content region is the one they name, so it answers this
        // law from exactly the same two stages the other two do.
        OverlayKind::Conflict => OverlayState::new_conflict(
            std::path::PathBuf::from("/notes/heron.md"),
            Some("what the disk says\n".to_string()),
        ),
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
    // The number is spelled out so a new member cannot arrive by silently
    // widening a filter. The DECISION each time is whether that member really is
    // a place you stay in — the conflict workspace is one: you read two
    // manuscripts there to choose between them.
    assert_eq!(
        kinds.len(),
        3,
        "the sustained roster is Settings + History + Conflict; a fourth member must answer \
         this law rather than silently skip it — got {kinds:?}"
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
    assert_eq!(graded, 6, "three members x two stages must each be graded");
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

/// THE TWO DEEP LINKS ENTER THE SAME WORKSPACE AT DIFFERENT FOCUS.
///
/// "Version history…" asks WHICH version and lands on the timeline; "Compare with
/// version…" asks WHAT CHANGED and lands in the comparison. One surface, two
/// doors, and the difference between them is the focus — which is what makes the
/// second door worth keeping rather than an alias of the first.
///
/// Driven through `apply_transition`, so this is the real dispatch and not a
/// hand-built card: a deep link that wrote `detail_focus` behind the lifecycle's
/// back would leave the journey's own stage disagreeing with the card's bit.
#[test]
fn version_history_lands_on_the_timeline_and_compare_lands_in_the_comparison() {
    for (action, want_detail, what) in [
        (Action::OpenHistory, false, "Version history…"),
        (Action::CompareVersion, true, "Compare with version…"),
    ] {
        let mut journey = Journey::seeded(None);
        drive_with_history(&mut journey, &action, card_for(OverlayKind::History));
        let card = journey
            .card()
            .unwrap_or_else(|| panic!("{what} must open the History workspace"));
        assert_eq!(card.kind, OverlayKind::History, "{what} opens History");
        assert_eq!(
            card.detail_focus,
            want_detail,
            "{what} must land {}",
            match want_detail {
                true => "in the COMPARISON — it is a question about what changed",
                false => "on the TIMELINE — it is a question about which version",
            }
        );
        // The LIFECYCLE agrees, not just the bit: a deep link that wrote the
        // focus flag directly would pass the assertion above and still leave the
        // journey standing on the wrong surface, so `Esc` and `Tab` would answer
        // for a stage nothing is on.
        assert_eq!(
            journey.state(),
            crate::overlay::State::Summoned {
                surface: match want_detail {
                    true => crate::overlay::Surface::WorkspaceDetail,
                    false => crate::overlay::Surface::Workspace,
                },
                beneath: crate::overlay::Beneath::Editor,
            },
            "{what}: the journey's own stage must be the focus it claims"
        );
    }

    // AN EMPTY HISTORY DEGRADES TO THE TIMELINE rather than handing the keyboard
    // to a blank region — the same `comparison_request()` fact the intercept
    // reads, exercised from the other end.
    let mut journey = Journey::seeded(None);
    drive_with_history(
        &mut journey,
        &Action::CompareVersion,
        OverlayState::new_history(Vec::new(), None, None),
    );
    assert!(
        journey
            .card()
            .is_some_and(|o| o.kind == OverlayKind::History && !o.detail_focus),
        "with nothing to compare, the Compare deep link must land on the timeline"
    );
}

/// Dispatch `action` with `card` available to `make_overlay`, exactly as the live
/// `App` supplies one from the gathered timeline.
fn drive_with_history(journey: &mut Journey, action: &Action, card: OverlayState) {
    let mut buffer = crate::buffer::Buffer::scratch();
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut make_overlay = |k: OverlayKind| match k {
        OverlayKind::History => Some(card.clone()),
        _ => None,
    };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| None;
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    let _ = apply_transition(&mut ctx, action, false).primary();
}
