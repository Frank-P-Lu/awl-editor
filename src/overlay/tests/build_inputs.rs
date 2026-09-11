use super::super::*;
use super::{history_rows, orphan};

fn bindings() -> BindingInputs<'static> {
    BindingInputs {
        keys: &[],
        linux_keep: &[],
        keymap_flavor: crate::keymap::KeymapFlavor::Native,
    }
}

/// Every request variant constructs its declared picker, and no navigable or
/// direct-edit kind acquires a misleading catch-all construction payload.
#[test]
fn focused_picker_inputs_cover_every_shared_builder_door() {
    let requests = vec![
        PickerInput::Goto(GotoInputs {
            corpus: vec!["a.md".to_string()],
            open: vec![0],
            recent: vec![0],
            times: vec!["just now".to_string()],
            headings: vec![("Heading".to_string(), 0)],
            line_count: 2,
        }),
        PickerInput::Theme,
        PickerInput::Caret,
        PickerInput::Dictionary,
        PickerInput::CjkLang,
        PickerInput::Date {
            today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
        },
        PickerInput::Keymap {
            configured: "native".to_string(),
        },
        PickerInput::Command(CommandInputs {
            bindings: bindings(),
            settings_values: Default::default(),
            row_gates: Default::default(),
        }),
        PickerInput::Keybindings(bindings()),
        PickerInput::Spell(Some((
            vec!["spelling".to_string()],
            (0, 0, 4),
            "speling".to_string(),
        ))),
        PickerInput::History(HistoryInputs {
            entries: history_rows(),
            now: Some(300),
            session_start: Some(0),
        }),
        PickerInput::Settings(Default::default()),
        PickerInput::Assets(vec![orphan("unused.png", 12)]),
        PickerInput::UserWords(vec!["awlish".to_string()]),
        PickerInput::SearchFolder(SearchFolderInputs {
            root: std::path::PathBuf::from("/project"),
            corpus: vec![("a.md".to_string(), "needle".to_string())],
        }),
        PickerInput::Credits,
    ];

    for input in &requests {
        let expected = input.kind();
        let overlay = build(input).unwrap_or_else(|| panic!("{expected:?} must summon"));
        assert_eq!(
            overlay.kind, expected,
            "focused request changed picker identity"
        );
        match input {
            PickerInput::Goto(_) => assert!(
                overlay.accepts().contains(&"a.md"),
                "Go-to's focused corpus must remain visible"
            ),
            PickerInput::Spell(_) => assert!(
                overlay.accepts().contains(&"spelling"),
                "Spell's focused target must remain visible"
            ),
            PickerInput::History(_) => assert!(
                overlay.accepts().contains(&"just now · fix: the engine"),
                "History's focused timeline rows must remain visible"
            ),
            PickerInput::Assets(_) => assert!(
                overlay.accepts().contains(&"unused.png"),
                "Assets' focused orphan list must remain visible"
            ),
            PickerInput::UserWords(_) => assert!(
                overlay.accepts().contains(&"awlish"),
                "Personal dictionary's focused words must remain visible"
            ),
            PickerInput::SearchFolder(_) => assert!(
                overlay
                    .search_corpus
                    .contains(&("a.md".to_string(), "needle".to_string())),
                "Folder search's focused corpus must remain visible"
            ),
            _ => {}
        }
    }

    assert!(
        build(&PickerInput::Spell(None)).is_none(),
        "only an absent spell target leaves its picker unopened"
    );
}

/// Settings owns its own complete readout and passes only its date/keymap slices
/// to child construction. This is the one intentional parent/child reuse; it
/// must not turn into a second generic request path.
#[test]
fn settings_children_keep_their_parent_readout_and_exact_picker_identity() {
    let values = crate::settings::SettingsValues {
        keymap: "emacs".to_string(),
        today_ymd: (2030, 4, 5),
        ..Default::default()
    };
    let input = PickerInput::Settings(values);
    for kind in [
        OverlayKind::Settings,
        OverlayKind::Theme,
        OverlayKind::Caret,
        OverlayKind::Dictionary,
        OverlayKind::CjkLang,
        OverlayKind::Date,
        OverlayKind::Keymap,
    ] {
        assert_eq!(
            build_for(kind, &input).map(|overlay| overlay.kind),
            Some(kind),
            "Settings must rebuild its {kind:?} child from the one parent readout"
        );
    }
    assert!(
        build_for(OverlayKind::Goto, &input).is_none(),
        "Settings values must never become an unrelated picker bag"
    );
}

/// The enum is the source-level ownership boundary: construction requests name
/// only real shared-builder doors. The exhaustive match makes a newly added
/// request decide its picker identity before it can ship.
#[test]
fn picker_request_identity_roster_is_exact() {
    let request_kinds = [
        OverlayKind::Goto,
        OverlayKind::Theme,
        OverlayKind::Caret,
        OverlayKind::Dictionary,
        OverlayKind::CjkLang,
        OverlayKind::Date,
        OverlayKind::Keymap,
        OverlayKind::Command,
        OverlayKind::Keybindings,
        OverlayKind::Spell,
        OverlayKind::History,
        OverlayKind::Settings,
        OverlayKind::Assets,
        OverlayKind::UserWords,
        OverlayKind::SearchFolder,
        OverlayKind::Credits,
    ];
    for kind in OverlayKind::ALL {
        assert_eq!(
            request_kinds.contains(&kind),
            !matches!(
                kind,
                OverlayKind::Browse
                    | OverlayKind::MoveDest
                    | OverlayKind::ExportDest
                    | OverlayKind::Project
                    | OverlayKind::ProjectBrowse
                    | OverlayKind::Rename
                    | OverlayKind::InsertLink
                    | OverlayKind::KeepName
                    | OverlayKind::Context
                    | OverlayKind::TableDims
                    | OverlayKind::Conflict
            ),
            "{kind:?} must either have one focused request or stay at its direct/navigable owner"
        );
    }
}
