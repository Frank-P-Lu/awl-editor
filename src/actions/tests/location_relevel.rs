//! Law matrix for the one action-level folder relevel owner.

use super::super::*;
use crate::overlay::{OverlayKind, OverlayState};

fn drive(
    journey: &mut crate::overlay::Journey,
    action: &Action,
    browse_to: &mut dyn FnMut(OverlayKind, Option<String>) -> Option<OverlayState>,
) -> Effect {
    let mut buffer = Buffer::scratch();
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut make = |_kind| None;
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey,
        make_overlay: &mut make,
        browse_to,
        oracle: None,
    };
    apply_transition(&mut ctx, action, false).primary()
}

#[derive(Clone, Copy)]
enum Payload {
    Plain,
    Export,
    SaveCopy,
    SettingPath,
    ProjectBrowse,
}

fn card(kind: OverlayKind, dir: Option<&str>, child: &str, payload: Payload) -> OverlayState {
    let mut card = match kind {
        OverlayKind::MoveDest => {
            let mut card = OverlayState::new_move_dest(
                dir.map(str::to_string),
                vec![(child.to_string(), false)],
            );
            assert!(card.hover_select(1), "Move's directory row must enroll");
            card
        }
        OverlayKind::Project => OverlayState::new_project(
            dir.expect("Project walks an absolute path").to_string(),
            vec![(child.to_string(), false)],
            &[],
        ),
        _ => OverlayState::new_marked(
            kind,
            vec![child.to_string()],
            vec![false],
            vec![true],
            Vec::new(),
            Vec::new(),
            dir.map(str::to_string),
        ),
    };
    match payload {
        Payload::Export => card.export_format = Some(crate::export::Format::Html),
        Payload::SaveCopy => card.save_copy = true,
        Payload::Plain | Payload::SettingPath | Payload::ProjectBrowse => {}
    }
    card
}

fn journey(
    kind: OverlayKind,
    dir: Option<&str>,
    child: &str,
    payload: Payload,
) -> crate::overlay::Journey {
    let card = card(kind, dir, child, payload);
    let (parent, bind) = match payload {
        Payload::SettingPath => (
            OverlayState::new(
                OverlayKind::Settings,
                vec!["Project root".to_string()],
                Vec::new(),
                Vec::new(),
            ),
            crate::overlay::Bind::Path {
                key: "default_folder".to_string(),
            },
        ),
        Payload::ProjectBrowse => (
            OverlayState::new(
                OverlayKind::Goto,
                vec!["Files".to_string()],
                Vec::new(),
                Vec::new(),
            ),
            crate::overlay::Bind::Value,
        ),
        Payload::Plain | Payload::Export | Payload::SaveCopy => {
            return crate::overlay::Journey::seeded(Some(card));
        }
    };
    let mut journey = crate::overlay::Journey::seeded(Some(parent));
    journey.descend(card, bind);
    journey
}

type Case = (
    &'static str,
    OverlayKind,
    Option<&'static str>,
    &'static str,
    Action,
    Option<&'static str>,
    Payload,
);

macro_rules! case {
    ($name:literal, $kind:ident, $dir:expr, $child:literal, $action:ident, $target:expr, $payload:ident) => {
        (
            $name,
            OverlayKind::$kind,
            $dir,
            $child,
            Action::$action,
            $target,
            Payload::$payload,
        )
    };
}

#[rustfmt::skip]
const CASES: [Case; 19] = [
    case!("Browse Enter", Browse, None, "docs", Newline, Some("docs"), Plain),
    case!("Browse Backspace", Browse, Some("docs/api"), "v1", DeleteBackward, Some("docs"), Plain),
    case!("Move Enter", MoveDest, None, "docs", Newline, Some("docs"), Plain),
    case!("Move Right", MoveDest, None, "docs", ForwardChar, Some("docs"), Plain),
    case!("Move Left", MoveDest, Some("docs/api"), "v1", BackwardChar, Some("docs"), Plain),
    case!("Move Backspace", MoveDest, Some("docs/api"), "v1", DeleteBackward, Some("docs"), Plain),
    case!("Export Right", ExportDest, None, "docs", ForwardChar, Some("docs"), Export),
    case!("Export Left", ExportDest, Some("docs/api"), "v1", BackwardChar, Some("docs"), Export),
    case!("Export Backspace", ExportDest, Some("docs/api"), "v1", DeleteBackward, Some("docs"), Export),
    case!("Save Copy Right", ExportDest, None, "docs", ForwardChar, Some("docs"), SaveCopy),
    case!("Save Copy Left", ExportDest, Some("docs/api"), "v1", BackwardChar, Some("docs"), SaveCopy),
    case!("Save Copy Backspace", ExportDest, Some("docs/api"), "v1", DeleteBackward, Some("docs"), SaveCopy),
    case!("Setting Enter", Project, Some("/ws"), "docs", Newline, Some("/ws/docs"), SettingPath),
    case!("Setting Right", Project, Some("/ws"), "docs", ForwardChar, Some("/ws/docs"), SettingPath),
    case!("Setting Left", Project, Some("/ws/docs"), "api", BackwardChar, Some("/ws"), SettingPath),
    case!("Setting Backspace", Project, Some("/ws/docs"), "api", DeleteBackward, Some("/ws"), SettingPath),
    case!("ProjectBrowse Right", ProjectBrowse, Some("/ws"), "docs", ForwardChar, Some("/ws/docs"), ProjectBrowse),
    case!("ProjectBrowse Left", ProjectBrowse, Some("/ws/docs"), "api", BackwardChar, Some("/ws"), ProjectBrowse),
    case!("ProjectBrowse Backspace", ProjectBrowse, Some("/ws/docs"), "api", DeleteBackward, Some("/ws"), ProjectBrowse),
];

/// Every action that means "change directory level" reaches one action owner.
/// The builder call is the non-vacuity witness; no operation may be emitted.
#[test]
fn every_shared_folder_step_relevels_once_without_committing() {
    for (name, kind, dir, child, action, target, payload) in CASES {
        let mut journey = journey(kind, dir, child, payload);
        assert_eq!(
            journey.card().and_then(OverlayState::selected_value),
            Some(child),
            "{name}: real folder row"
        );
        assert!(
            journey.card().is_some_and(OverlayState::selected_is_dir),
            "{name}: subject enrollment"
        );
        let expected = target.map(str::to_string);
        assert_ne!(
            dir.map(str::to_string),
            expected,
            "{name}: step changes level"
        );
        let mut calls = Vec::new();
        let mut build = |kind: OverlayKind, target: Option<String>| {
            calls.push((kind, target.clone()));
            Some(card(kind, target.as_deref(), "next", Payload::Plain))
        };
        assert_eq!(
            drive(&mut journey, &action, &mut build),
            Effect::None,
            "{name}: navigation has no operation"
        );
        assert_eq!(
            calls,
            vec![(kind, expected.clone())],
            "{name}: exactly one shared rebuild"
        );
        assert_eq!(
            journey.card().and_then(|card| card.browse_dir.clone()),
            expected,
            "{name}: rebuilt level installed"
        );
        match payload {
            Payload::Export => assert_eq!(
                journey.card().and_then(|card| card.export_format),
                Some(crate::export::Format::Html),
                "{name}: format survives"
            ),
            Payload::SaveCopy => assert!(
                journey.card().is_some_and(|card| card.save_copy),
                "{name}: Save Copy survives"
            ),
            Payload::SettingPath => assert!(
                matches!(journey.bind(), Some(crate::overlay::Bind::Path { key }) if key == "default_folder"),
                "{name}: Settings parent and key survive"
            ),
            Payload::ProjectBrowse => assert!(
                matches!(journey.bind(), Some(crate::overlay::Bind::Value)),
                "{name}: Files parent survives"
            ),
            Payload::Plain => {}
        }
    }
}

fn assert_no_rebuild(journey: &mut crate::overlay::Journey, expected: Effect, name: &str) {
    let mut calls = Vec::new();
    let mut reject = |kind, target| {
        calls.push((kind, target));
        None
    };
    assert_eq!(
        drive(journey, &Action::Newline, &mut reject),
        expected,
        "{name}: typed effect"
    );
    assert!(
        calls.is_empty(),
        "{name}: accept must not browse: {calls:?}"
    );
}

/// Enter remains each consumer's typed operation wherever it is not a folder
/// step. These cases are the negative boundary around the shared helper.
#[test]
fn typed_location_accepts_never_rebuild_a_directory_level() {
    let mut flat = crate::overlay::Journey::seeded(Some(card(
        OverlayKind::Project,
        Some("/ws"),
        "docs",
        Payload::Plain,
    )));
    assert_no_rebuild(
        &mut flat,
        Effect::OverlayAccept(OverlayKind::Project, "/ws/docs".to_string()),
        "flat Project",
    );

    let mut browse = journey(
        OverlayKind::ProjectBrowse,
        Some("/ws"),
        "docs",
        Payload::ProjectBrowse,
    );
    assert_no_rebuild(
        &mut browse,
        Effect::OverlayAccept(OverlayKind::Project, "/ws/docs".to_string()),
        "ProjectBrowse",
    );

    let mut export = crate::overlay::Journey::seeded(Some(card(
        OverlayKind::ExportDest,
        None,
        "docs",
        Payload::Export,
    )));
    assert_no_rebuild(
        &mut export,
        Effect::Export(crate::export::Format::Html, Some("docs".to_string())),
        "Export",
    );

    let mut save = crate::overlay::Journey::seeded(Some(card(
        OverlayKind::ExportDest,
        None,
        "docs",
        Payload::SaveCopy,
    )));
    assert_no_rebuild(&mut save, Effect::None, "Save Copy destination");
    assert!(
        save.card().is_some_and(|card| card.rename_edit.is_some()),
        "Save Copy must enter its filename prompt"
    );

    let move_here = OverlayState::new_move_dest(None, vec![("docs".to_string(), false)]);
    assert_eq!(move_here.selected_value(), Some("Move here"));
    let mut move_here = crate::overlay::Journey::seeded(Some(move_here));
    assert_no_rebuild(
        &mut move_here,
        Effect::OverlayAccept(OverlayKind::MoveDest, String::new()),
        "Move here",
    );
}

/// Query editing and a builder's scope refusal leave the route intact.
#[test]
fn folder_steps_respect_query_ownership_and_builder_refusal() {
    let calls = std::cell::RefCell::new(Vec::new());
    let mut reject = |kind, target| {
        calls.borrow_mut().push((kind, target));
        None
    };

    let mut filtered = card(OverlayKind::MoveDest, Some("docs"), "api", Payload::Plain);
    filtered.push('x');
    assert!(!filtered.query.is_empty(), "query-edit case must enroll");
    let mut filtered = crate::overlay::Journey::seeded(Some(filtered));
    assert_eq!(
        drive(&mut filtered, &Action::DeleteBackward, &mut reject),
        Effect::None
    );
    assert!(
        calls.borrow().is_empty(),
        "nonempty Backspace edits instead of ascending"
    );
    assert_eq!(
        filtered.card().and_then(|card| card.browse_dir.as_deref()),
        Some("docs")
    );

    let mut mid = card(
        OverlayKind::ExportDest,
        Some("docs"),
        "api",
        Payload::Export,
    );
    for c in "api".chars() {
        mid.push(c);
    }
    mid.query_home();
    assert!(!mid.query_at_rest(), "mid-query motion case must enroll");
    let mut mid = crate::overlay::Journey::seeded(Some(mid));
    assert_eq!(
        drive(&mut mid, &Action::ForwardChar, &mut reject),
        Effect::None
    );
    assert!(
        calls.borrow().is_empty(),
        "mid-query Right moves the caret instead of descending"
    );
    assert_eq!(
        mid.card().and_then(|card| card.browse_dir.as_deref()),
        Some("docs")
    );

    let mut floored = journey(
        OverlayKind::ProjectBrowse,
        Some("/ws/docs"),
        "api",
        Payload::ProjectBrowse,
    );
    let snapshot = |journey: &crate::overlay::Journey| {
        journey.card().map(|card| {
            (
                card.kind,
                card.browse_dir.clone(),
                card.selected_value().map(str::to_string),
            )
        })
    };
    let before = snapshot(&floored);
    assert_eq!(
        drive(&mut floored, &Action::BackwardChar, &mut reject),
        Effect::None
    );
    assert_eq!(
        *calls.borrow(),
        vec![(OverlayKind::ProjectBrowse, Some("/ws".to_string()))]
    );
    assert_eq!(
        snapshot(&floored),
        before,
        "a refused rebuild leaves the exact visible level"
    );
    assert!(matches!(floored.bind(), Some(crate::overlay::Bind::Value)));
}
