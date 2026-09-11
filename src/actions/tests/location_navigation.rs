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

fn probe_shared_location_routes() -> Vec<LocationConsumer> {
    let mut covered = Vec::new();
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
        covered.push(consumer);
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
    covered
}

fn probe_save_copy() -> LocationConsumer {
    // Save Copy shares ExportDest's navigation, but its explicit destination
    // accept must continue into the filename prompt and only that prompt may
    // emit the copy operation.
    let mut save_copy = browse_level(OverlayKind::ExportDest, None).expect("save-copy level");
    save_copy.save_copy = true;
    let mut journey = crate::overlay::Journey::seeded(Some(save_copy));
    assert_eq!(
        drive_effect(&mut journey, &Action::ForwardChar),
        Effect::None
    );
    assert_eq!(journey.card().unwrap().browse_dir.as_deref(), Some("docs"));
    assert_eq!(drive_effect(&mut journey, &Action::Newline), Effect::None);
    assert!(journey.card().unwrap().rename_edit.is_some());
    let save_effect = drive_effect(&mut journey, &Action::Newline);
    assert!(
        matches!(save_effect, Effect::SaveCopyName { ref dest, .. } if dest == "docs/api"),
        "Save Copy's filename commit must retain its chosen destination: {save_effect:?}"
    );
    LocationConsumer::SaveCopyDestination
}

fn probe_project_routes() -> [LocationConsumer; 2] {
    // The flat Project card commits a switch immediately, while the same card
    // under Bind::Path walks folders and emits a setting-path value instead.
    let project = browse_level(OverlayKind::Project, None).expect("project level");
    let mut journey = crate::overlay::Journey::seeded(Some(project));
    assert_eq!(
        drive_effect(&mut journey, &Action::DeleteBackward),
        Effect::None
    );
    assert_eq!(
        journey.card().unwrap().browse_dir,
        None,
        "the flat project picker cannot ascend out of its workspace landing"
    );
    assert_eq!(
        drive_effect(&mut journey, &Action::Newline),
        Effect::OverlayAccept(OverlayKind::Project, "docs".to_string())
    );
    let flat = LocationConsumer::ProjectSwitch;

    let parent = OverlayState::new(OverlayKind::Settings, vec!["folder".into()], vec![], vec![]);
    let project = browse_level(OverlayKind::Project, None).expect("setting path level");
    let mut journey = crate::overlay::Journey::seeded(Some(parent));
    journey.descend(
        project,
        crate::overlay::Bind::Path {
            key: "default_folder".to_string(),
        },
    );
    assert_eq!(
        drive_effect(&mut journey, &Action::ForwardChar),
        Effect::None,
        "a folder row navigates rather than writing the setting"
    );
    assert_eq!(journey.card().unwrap().browse_dir.as_deref(), Some("docs"));
    drive_effect(&mut journey, &Action::NextLine);
    assert_eq!(
        drive_effect(&mut journey, &Action::Newline),
        Effect::SettingPathPick {
            key: "default_folder".to_string(),
            path: "docs".to_string(),
        }
    );
    [flat, LocationConsumer::SettingPath]
}

fn probe_invalid_export() {
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

/// A folder row is navigation, not a persistence shortcut. This drives every
/// in-app consumer through its real action seam and fails enrollment when a
/// new route joins the roster without an outcome probe.
#[test]
fn location_navigation_search_back_cancel_and_commit_keep_their_operation_boundaries() {
    let mut covered = probe_shared_location_routes();
    covered.push(probe_save_copy());
    covered.extend(probe_project_routes());
    probe_invalid_export();

    let declared: Vec<_> = LocationConsumer::ALL
        .into_iter()
        .filter(|consumer| consumer.overlay_kind().is_some())
        .collect();
    assert_eq!(
        covered.len(),
        declared.len(),
        "duplicate or missing route case"
    );
    for consumer in declared {
        assert!(
            covered.contains(&consumer),
            "{consumer:?}: every in-app location consumer needs an outcome case"
        );
    }
}
