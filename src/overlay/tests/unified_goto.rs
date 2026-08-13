use super::super::*;

fn unified() -> OverlayState {
    let mut ov = OverlayState::new(
        OverlayKind::Goto,
        vec!["notes/alpha.md".into(), "zebra.txt".into()],
        vec![],
        vec![1],
    );
    ov.attach_headings(vec![("Alpha heading".into(), 7)]);
    ov.attach_folders(
        vec![
            ("/work/notes".into(), true),
            ("/work/archive".into(), false),
        ],
        &["/work/archive".into()],
    );
    ov
}

#[test]
fn goto_lenses_are_the_exact_typed_destination_roster() {
    let mut ov = unified();
    assert_eq!(
        ov.lens_strip()
            .into_iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>(),
        ["All", "Files", "Headings", "Folders", "Recent"]
    );

    let cells = [
        ("all", 5usize),
        ("files", 2),
        ("headings", 1),
        // two destinations plus the direct chooser fallback
        ("folders", 3),
        // one recent file + one recent folder, preserving source MRU order
        ("recent", 2),
    ];
    for (lens, expected) in cells {
        ov.focus_facet_id(lens);
        assert_eq!(ov.items.len(), expected, "{lens} must enrol its real rows");
    }
    ov.focus_facet_id("folders");
    assert_eq!(ov.item_times(), ["folder", "folder", ""]);
    assert!(
        ov.item_strings()
            .iter()
            .take(2)
            .all(|row| row.ends_with('/')),
        "folder rows carry visible path identity: {:?}",
        ov.item_strings()
    );
    assert_eq!(ov.item_strings().last().unwrap(), "Choose another folder…");
}

#[test]
fn goto_all_fuzzy_ranks_across_types_and_empty_states_are_specific() {
    let mut ov = unified();
    for c in "archive".chars() {
        ov.push(c);
    }
    assert_eq!(ov.selected_value(), Some("/work/archive"));
    assert!(ov.selected_is_goto_folder());

    ov.focus_facet_id("headings");
    assert_eq!(ov.empty_message(), "no matches");
    while !ov.query.is_empty() {
        ov.pop();
    }
    assert_eq!(
        ov.item_strings(),
        [format!(
            "{}Alpha heading",
            OverlayKind::HEADING_MARKER_PREFIX
        )]
    );

    let mut empty = OverlayState::new(OverlayKind::Goto, Vec::new(), vec![], vec![]);
    empty.focus_facet_id("files");
    assert_eq!(empty.empty_message(), "no files here");
    empty.focus_facet_id("headings");
    assert_eq!(empty.empty_message(), "no headings yet");
    empty.focus_facet_id("recent");
    assert_eq!(empty.empty_message(), "no recent destinations");
}

#[test]
fn typed_goto_rows_emit_file_heading_and_folder_effects() {
    use crate::actions::{ActionCtx, Effect, apply_transition};
    use crate::keymap::Action;

    let run = |mut ov: OverlayState, query: &str| {
        for c in query.chars() {
            ov.push(c);
        }
        let mut journey = Journey::seeded(Some(ov));
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
        apply_transition(&mut ctx, &Action::Newline, false).primary()
    };

    assert_eq!(
        run(unified(), "zebra"),
        Effect::OverlayAccept(OverlayKind::Goto, "zebra.txt".into())
    );
    assert_eq!(run(unified(), "heading"), Effect::JumpToLine(7));
    assert_eq!(
        run(unified(), "archive"),
        Effect::OverlayAccept(OverlayKind::Project, "/work/archive".into())
    );

    let mut fallback = unified();
    fallback.focus_facet_id("folders");
    fallback.select_last();
    assert_eq!(
        run(fallback, ""),
        Effect::Surface(crate::actions::SurfaceEffect::OpenFolderChooser)
    );
}
