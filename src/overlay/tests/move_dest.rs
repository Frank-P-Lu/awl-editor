use super::*;

fn move_dest(folders: &[&str]) -> OverlayState {
    OverlayState::new_move_dest(
        None,
        folders.iter().map(|f| (f.to_string(), false)).collect(),
    )
}

#[test]
fn move_here_leads_at_rest_and_new_folder_stays_hidden() {
    let ov = move_dest(&["docs", "assets"]);
    assert_eq!(ov.selected_value(), Some("Move here"));
    assert!(
        !ov.item_strings()
            .iter()
            .any(|s| s.starts_with("New folder")),
        "nothing typed yet: {:?}",
        ov.item_strings()
    );
}

#[test]
fn new_folder_target_is_none_for_an_empty_or_matching_query() {
    let mut ov = move_dest(&["docs"]);
    assert_eq!(ov.move_dest_new_folder_target(), None, "empty query");
    for c in "docs".chars() {
        ov.push(c);
    }
    assert_eq!(
        ov.move_dest_new_folder_target(),
        None,
        "an exact (case-insensitive) match names no NEW folder"
    );
}

#[test]
fn new_folder_target_is_case_insensitive_against_existing_folders() {
    let mut ov = move_dest(&["Docs"]);
    for c in "docs".chars() {
        ov.push(c);
    }
    assert_eq!(
        ov.move_dest_new_folder_target(),
        None,
        "\"docs\" reads as the same folder as \"Docs\""
    );
}

#[test]
fn new_folder_target_names_a_genuinely_unmatched_query() {
    let mut ov = move_dest(&["docs"]);
    for c in "ideas".chars() {
        ov.push(c);
    }
    assert_eq!(ov.move_dest_new_folder_target(), Some("ideas".to_string()));
}

/// Move stays bounded to the source file's owning root: a query naming more
/// than one path segment can never ride the create-a-folder door, however it
/// is typed.
#[test]
fn new_folder_target_refuses_anything_that_is_not_one_path_segment() {
    for bad in ["..", ".", "a/b", "../escape", "a\\b", "/abs"] {
        let mut ov = move_dest(&["docs"]);
        for c in bad.chars() {
            ov.push(c);
        }
        assert_eq!(
            ov.move_dest_new_folder_target(),
            None,
            "{bad:?} must not be offered as a new-folder name"
        );
        assert!(
            !ov.item_strings()
                .iter()
                .any(|s| s.starts_with("New folder")),
            "{bad:?}: the row itself must stay hidden: {:?}",
            ov.item_strings()
        );
    }
}

#[test]
fn move_here_is_reachable_but_no_longer_first_once_a_query_is_typed() {
    let mut ov = move_dest(&["docs"]);
    ov.push('d');
    assert_ne!(
        ov.selected_value(),
        Some("Move here"),
        "a real match leads instead"
    );
    assert!(
        ov.item_strings().iter().any(|s| s == "Move here"),
        "still reachable: {:?}",
        ov.item_strings()
    );
}

#[test]
fn only_declared_row_meta_tags_appear_and_both_new_ones_are_produced() {
    let mut ov = move_dest(&["docs"]);
    ov.push('z'); // matches nothing -> New folder becomes visible too
    let tags: Vec<_> = ov.rows.iter().map(|r| r.meta.tag()).collect();
    assert!(tags.contains(&crate::overlay::RowMetaTag::MoveHere));
    assert!(tags.contains(&crate::overlay::RowMetaTag::NewFolder));
    assert!(tags.contains(&crate::overlay::RowMetaTag::Plain));
    for tag in tags {
        assert!(
            OverlayKind::MoveDest.row_meta_roster().contains(&tag),
            "{tag:?} not in MoveDest's declared roster"
        );
    }
}
