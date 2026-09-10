use super::super::*;

fn files_level(dir: Option<&str>) -> OverlayState {
    OverlayState::new_files(
        vec![
            "alpha.md".into(),
            "notes/draft.md".into(),
            "notes/deep/draft.md".into(),
            "notes/deep/plan.md".into(),
        ],
        vec![0],
        vec![3, 1],
        dir.map(str::to_string),
    )
}

#[test]
fn files_and_recent_are_the_exact_visible_views() {
    let mut ov = files_level(None);
    assert_eq!(
        ov.lens_strip()
            .into_iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>(),
        ["Files", "Recent"]
    );
    assert_eq!(
        ov.item_strings(),
        [
            "alpha.md",
            "notes/  ›",
            "Change folder…",
            "New document — root/"
        ]
    );

    ov.focus_facet_id("recent");
    assert_eq!(
        ov.item_strings(),
        [
            "notes/deep/plan.md",
            "notes/draft.md",
            "Change folder…",
            "New document — root/"
        ]
    );
}

#[test]
fn files_searches_the_whole_root_and_clear_restores_the_browse_level() {
    let mut ov = files_level(Some("notes"));
    assert_eq!(
        ov.item_strings(),
        [
            "draft.md",
            "deep/  ›",
            "Change folder…",
            "New document — notes/"
        ]
    );

    for c in "draft".chars() {
        ov.push(c);
    }
    assert_eq!(
        ov.item_strings(),
        [
            "notes/draft.md",
            "notes/deep/draft.md",
            "Change folder…",
            "New document — notes/"
        ],
        "duplicate names must retain root-relative path identity"
    );
    while !ov.query.is_empty() {
        ov.pop();
    }
    assert_eq!(ov.item_strings()[..2], ["draft.md", "deep/  ›"]);
}

#[test]
fn folder_accept_descends_without_emitting_a_root_switch() {
    use crate::actions::{ActionCtx, Effect, apply_transition};
    use crate::keymap::Action;

    let mut journey = Journey::seeded(Some(files_level(None)));
    journey.card_mut().unwrap().move_sel(1);
    let mut buffer = crate::buffer::Buffer::scratch();
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut make_overlay = |_| None;
    let mut browse_to = |kind, rel: Option<String>| {
        assert_eq!(kind, OverlayKind::Goto);
        Some(files_level(rel.as_deref()))
    };
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    let effect = apply_transition(&mut ctx, &Action::Newline, false).primary();
    assert_eq!(effect, Effect::None);
    assert_eq!(
        ctx.journey.card().unwrap().browse_dir.as_deref(),
        Some("notes")
    );
    assert_eq!(
        ctx.journey.card().unwrap().item_strings()[..2],
        ["draft.md", "deep/  ›"]
    );
}

#[test]
fn files_intercepts_direct_new_document_actions_at_the_browse_destination() {
    use crate::actions::{ActionCtx, Effect, apply_transition};
    use crate::keymap::Action;

    let mut journey = Journey::seeded(Some(files_level(Some("notes/deep"))));
    let mut buffer = crate::buffer::Buffer::scratch();
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut make_overlay = |_| None;
    let mut browse_to = |_, _| None;
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    assert_eq!(
        apply_transition(&mut ctx, &Action::NewDocument, false).primary(),
        Effect::NewDocumentAt("notes/deep".into())
    );
    assert!(ctx.journey.card().is_none());
}

#[test]
fn heading_and_line_routes_remain_separate_from_files() {
    let mut ov = files_level(None);
    ov.attach_headings(vec![("Chapter".into(), 7)]);
    ov.attach_line_jump(20);
    assert!(!ov.item_strings().iter().any(|row| row.contains("Chapter")));
    ov.focus_headings();
    assert_eq!(
        ov.item_strings(),
        [format!("{}Chapter", OverlayKind::HEADING_MARKER_PREFIX)]
    );
    ov.push('9');
    assert!(ov.item_strings().iter().any(|row| row == "Go to line 9"));
}

#[test]
fn files_tab_route_is_complete_reversible_and_keeps_selection_separate() {
    let mut ov = files_level(Some("notes"));
    let selected = ov.selected_value().map(str::to_string);
    let forward = [
        FilesFocus::Files,
        FilesFocus::Recent,
        FilesFocus::Up,
        FilesFocus::Choices,
        FilesFocus::ChangeFolder,
        FilesFocus::NewDocument,
        FilesFocus::Query,
    ];
    for expected in forward {
        ov.files_focus_step(1);
        assert_eq!(ov.files_focus, expected);
    }
    let backward = [
        FilesFocus::NewDocument,
        FilesFocus::ChangeFolder,
        FilesFocus::Choices,
        FilesFocus::Up,
        FilesFocus::Recent,
        FilesFocus::Files,
        FilesFocus::Query,
    ];
    for expected in backward {
        ov.files_focus_step(-1);
        assert_eq!(ov.files_focus, expected);
    }
    assert_eq!(ov.selected_value(), selected.as_deref());

    ov.focus_facet_id("recent");
    ov.files_focus = FilesFocus::Recent;
    ov.files_focus_step(1);
    assert_eq!(ov.files_focus, FilesFocus::Choices, "Recent skips Up");
}

#[test]
fn files_keeps_empty_directories_and_names_three_level_outcomes() {
    let mut ov = OverlayState::new_files(Vec::new(), Vec::new(), Vec::new(), None);
    ov.attach_file_directories(vec!["empty".into()]);
    assert_eq!(ov.item_strings()[0], "empty/  ›");

    let mut empty = OverlayState::new_files(Vec::new(), Vec::new(), Vec::new(), None);
    empty.set_files_level_state(Some(&[]));
    assert_eq!(empty.notice, "this folder is empty");

    let unsupported = [crate::index::DirEntry {
        name: "movie.bin".into(),
        is_dir: false,
        is_git: false,
    }];
    empty.set_files_level_state(Some(&unsupported));
    assert_eq!(empty.notice, "no supported files in this folder");
    empty.set_files_level_state(None);
    assert_eq!(
        empty.notice,
        "folder unavailable — check access and try again"
    );
}

#[test]
fn files_names_a_real_unsupported_only_directory_instead_of_offering_its_binary() {
    // This is the production Files shape: the root-wide corpus contains the
    // path, while the current level reports that same unsupported leaf.
    // A binary-only directory needs its own calm outcome, not an openable row.
    let mut files = OverlayState::new_files(
        vec!["logo.icns".into()],
        Vec::new(),
        Vec::new(),
        None,
    );
    let entries = [crate::index::DirEntry {
        name: "logo.icns".into(),
        is_dir: false,
        is_git: false,
    }];
    files.set_files_level_state(Some(&entries));
    assert_eq!(files.notice, "no supported files in this folder");
    assert!(
        !files.item_strings().iter().any(|row| row == "logo.icns"),
        "an unsupported-only Files level must not present a binary as a file choice"
    );
}
