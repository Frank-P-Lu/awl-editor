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
            subject_name: "September.md".to_string(),
        }),
        PickerInput::Settings(Default::default()),
        PickerInput::Assets(vec![orphan("unused.png", 12)]),
        PickerInput::UserWords(vec!["awlish".to_string()]),
        PickerInput::SearchFolder(SearchFolderInputs {
            root: std::path::PathBuf::from("/project"),
            corpus: vec![("a.md".to_string(), "needle".to_string())],
            incomplete: false,
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

/// The action boundary names one focused picker kind before live App or replay
/// gathers its distinct payload. Both callers must delegate that decision;
/// local action matches would let their summon rosters drift again.
#[test]
fn picker_action_roster_has_one_shared_owner() {
    let direct = [
        (crate::keymap::Action::OpenGoto, OverlayKind::Goto),
        (crate::keymap::Action::OpenProject, OverlayKind::Goto),
        (crate::keymap::Action::OpenRecentProjects, OverlayKind::Goto),
        (crate::keymap::Action::OpenOutline, OverlayKind::Goto),
        (crate::keymap::Action::OpenThemeMenu, OverlayKind::Theme),
        (crate::keymap::Action::OpenCaretMenu, OverlayKind::Caret),
        (
            crate::keymap::Action::OpenDictionaryMenu,
            OverlayKind::Dictionary,
        ),
        (crate::keymap::Action::OpenKeymapMenu, OverlayKind::Keymap),
        (
            crate::keymap::Action::OpenCommandPalette,
            OverlayKind::Command,
        ),
        (
            crate::keymap::Action::OpenKeybindings,
            OverlayKind::Keybindings,
        ),
        (crate::keymap::Action::OpenSpellSuggest, OverlayKind::Spell),
        (crate::keymap::Action::OpenHistory, OverlayKind::History),
        (crate::keymap::Action::CompareVersion, OverlayKind::History),
        (
            crate::keymap::Action::OpenSettingsMenu,
            OverlayKind::Settings,
        ),
        (crate::keymap::Action::OpenAssetClean, OverlayKind::Assets),
        (crate::keymap::Action::OpenUserWords, OverlayKind::UserWords),
        (
            crate::keymap::Action::OpenSearchFolder,
            OverlayKind::SearchFolder,
        ),
        (crate::keymap::Action::OpenCredits, OverlayKind::Credits),
    ];
    for (action, expected) in direct {
        assert_eq!(
            crate::overlay::picker_kind_for(&action, None, None),
            Some(expected),
            "{action:?} must choose its one focused picker kind"
        );
    }
    for action in [
        crate::keymap::Action::Cancel,
        crate::keymap::Action::Newline,
        crate::keymap::Action::AcceptAlternate,
    ] {
        assert_eq!(
            crate::overlay::picker_kind_for(&action, Some(OverlayKind::Command), None),
            Some(OverlayKind::Command),
            "{action:?} must rebuild the parked parent",
        );
    }
    let mut settings = build(&PickerInput::Settings(Default::default())).unwrap();
    assert!(settings.select_accept("Caret style"));
    for action in [
        crate::keymap::Action::Newline,
        crate::keymap::Action::AcceptAlternate,
    ] {
        assert_eq!(
            crate::overlay::picker_kind_for(&action, Some(OverlayKind::Command), Some(&settings),),
            Some(OverlayKind::Caret),
            "a Settings row's selected child wins over an older parked launcher",
        );
    }
    let retained =
        crate::overlay::picker_kind_for(&crate::keymap::Action::ToggleDebug, None, Some(&settings));
    assert_eq!(
        retained,
        Some(OverlayKind::Settings),
        "a Settings-owned child action must retain its parent input",
    );
    assert_eq!(
        crate::overlay::picker_kind_for(&crate::keymap::Action::ToggleDebug, None, None),
        None,
        "an ordinary action must not acquire a picker input",
    );

    let live_apply = include_str!("../../app/apply.rs");
    let live_inputs = include_str!("../../app/apply/overlay_inputs.rs");
    let replay = include_str!("../../main/run/chord.rs");
    assert!(
        live_apply.contains("gather_picker_input("),
        "the live transition must delegate environment-specific gathering",
    );
    for source in [live_inputs, replay] {
        assert!(
            source.contains("overlay::picker_kind_for("),
            "every live/replay construction site must delegate picker identity",
        );
    }
    for source in [live_apply, live_inputs, replay] {
        assert!(
            !source.contains("let picker_kind = match action"),
            "a caller-local action match would bypass the shared picker roster",
        );
    }
}
