//! Hermetic journey checks for the shared location-navigation contract.

use super::super::*;
use super::{browse_level, drive};
use crate::overlay::{LocationConsumer, OverlayKind, OverlayState};

fn drive_effect(journey: &mut crate::overlay::Journey, action: &Action) -> Effect {
    let mut buffer = Buffer::scratch();
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut make = |_kind: OverlayKind| None;
    let mut levels = |kind, rel| browse_level(kind, rel);
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey,
        make_overlay: &mut make,
        browse_to: &mut levels,
        oracle: None,
    };
    apply_transition(&mut ctx, action, false).primary()
}

/// A folder row is navigation, not a persistence shortcut.  This drives the
/// actual action core through search, descend, Back, cancel, invalid commit,
/// and each operation's valid explicit commit using the deterministic level
/// supplier shared by the other picker tests.
#[test]
fn location_navigation_search_back_cancel_and_commit_keep_their_operation_boundaries() {
    let cases = [
        (
            LocationConsumer::Browse,
            OverlayKind::Browse,
            Effect::OverlayAccept(OverlayKind::Goto, "README.md".to_string()),
        ),
        (
            LocationConsumer::MoveDestination,
            OverlayKind::MoveDest,
            Effect::OverlayAccept(OverlayKind::MoveDest, String::new()),
        ),
        (
            LocationConsumer::ExportDestination,
            OverlayKind::ExportDest,
            Effect::Export(crate::export::Format::Html, Some("docs/api".to_string())),
        ),
        (
            LocationConsumer::ProjectBrowse,
            OverlayKind::ProjectBrowse,
            Effect::OverlayAccept(OverlayKind::Project, "docs/api".to_string()),
        ),
    ];

    for (consumer, kind, expected_commit) in cases {
        let mut card = browse_level(kind, None).expect("hermetic level");
        if kind == OverlayKind::ExportDest {
            card.export_format = Some(crate::export::Format::Html);
        }
        let mut journey = crate::overlay::Journey::seeded(Some(card));
        let mut accepted = None;

        // Search changes only the card.  A navigation key opens the matched
        // folder but emits no effect, which is the no-premature-commit claim.
        drive(&mut journey, &mut accepted, &Action::InsertChar('d'));
        assert_eq!(journey.card().unwrap().selected_value(), Some("docs"));
        let navigation = if kind == OverlayKind::Browse {
            Action::Newline
        } else {
            Action::ForwardChar
        };
        let effect = drive_effect(&mut journey, &navigation);
        assert_eq!(journey.card().unwrap().browse_dir.as_deref(), Some("docs"));
        assert!(
            accepted.is_none(),
            "{consumer:?}: opening a folder must not commit"
        );
        assert_eq!(
            effect,
            Effect::None,
            "{consumer:?}: folder navigation is not a side effect"
        );

        // Back returns to the original scope, then cancel closes without an
        // operation effect.  This is the core-only half of "cancel writes no
        // file"; the App's byte-level copy/export laws cover the interpreter.
        drive(&mut journey, &mut accepted, &Action::DeleteBackward);
        drive(&mut journey, &mut accepted, &Action::DeleteBackward);
        assert!(
            journey
                .card()
                .unwrap()
                .browse_dir
                .as_deref()
                .is_none_or(str::is_empty),
            "{consumer:?}: Back returns to its supplied root"
        );
        drive(&mut journey, &mut accepted, &Action::Cancel);
        assert!(
            journey.card().is_none(),
            "{consumer:?}: cancel closes the card"
        );
        assert!(accepted.is_none(), "{consumer:?}: cancel must not commit");

        // An explicit accept is the sole commitment.  Browse needs its file
        // row; destinations name the folder after a deliberate descent, while
        // Move accepts its dedicated `Move here` row at the root.
        let mut card = browse_level(kind, None).expect("fresh hermetic level");
        if kind == OverlayKind::ExportDest {
            card.export_format = Some(crate::export::Format::Html);
        }
        let mut journey = crate::overlay::Journey::seeded(Some(card));
        if kind == OverlayKind::Browse {
            drive(&mut journey, &mut accepted, &Action::NextLine);
        } else if kind != OverlayKind::MoveDest {
            drive(&mut journey, &mut accepted, &Action::ForwardChar);
        }
        let effect = drive_effect(&mut journey, &Action::Newline);
        match expected_commit {
            Effect::OverlayAccept(expected_kind, expected_value) => assert_eq!(
                effect,
                Effect::OverlayAccept(expected_kind, expected_value),
                "{consumer:?}: only explicit acceptance produces its typed operation"
            ),
            Effect::Export(format, dest) => assert_eq!(effect, Effect::Export(format, dest)),
            _ => unreachable!("location roster has only typed commits"),
        }
    }

    // No format is an invalid export destination: even Enter is inert, rather
    // than guessing a write.  This must exercise the real accept seam.
    let mut invalid = crate::overlay::Journey::seeded(Some(OverlayState::new_marked(
        OverlayKind::ExportDest,
        vec!["out".into()],
        vec![false],
        vec![true],
        Vec::new(),
        Vec::new(),
        None,
    )));
    assert_eq!(drive_effect(&mut invalid, &Action::Newline), Effect::None);
    assert!(
        invalid.card().is_none(),
        "invalid explicit accept still closes the request"
    );
}
