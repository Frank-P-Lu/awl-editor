use super::*;

#[test]
fn assets_picker_shows_leaf_names_and_size_parent_secondary() {
    let ov = OverlayState::new_assets(vec![
        orphan("assets/photo.png", 12_600),
        orphan("notes/assets/old.png", 5),
    ]);
    // PRIMARY cell is the leaf file name, not the full path.
    assert_eq!(ov.item_strings(), vec!["photo.png", "old.png"]);
    // SECONDARY cell (bindings column) is "size · parent dir".
    assert_eq!(
        ov.item_bindings(),
        vec!["12.3 KB · assets", "5 B · notes/assets"]
    );
    // The ACCEPT value stays the full root-relative path (the trash key).
    assert_eq!(ov.selected_value(), Some("assets/photo.png"));
    // Fuzzy still matches over the full path, so typing a folder narrows.
    let mut ov2 = OverlayState::new_assets(vec![
        orphan("assets/photo.png", 1),
        orphan("notes/assets/old.png", 1),
    ]);
    ov2.push('n');
    ov2.push('o');
    ov2.push('t');
    assert_eq!(ov2.selected_value(), Some("notes/assets/old.png"));
}

#[test]
fn assets_remove_asset_row_shrinks_the_list_and_keeps_the_picker_open() {
    let mut ov = OverlayState::new_assets(vec![
        orphan("assets/a.png", 1),
        orphan("assets/b.png", 2),
        orphan("assets/c.png", 3),
    ]);
    assert_eq!(ov.items.len(), 3);
    // Remove the MIDDLE row by value: the other two remain, in order.
    assert!(ov.remove_asset_row("assets/b.png"));
    assert_eq!(ov.item_strings(), vec!["a.png", "c.png"]);
    // The secondary column stays index-aligned (b's row is gone, not misaligned).
    assert_eq!(ov.item_bindings(), vec!["1 B · assets", "3 B · assets"]);
    // PRESERVATION LAW (removal preserves identity): with ONE typed
    // `OverlayRow`, a `Vec::remove` carries a row's own accept + secondary
    // together as a single element — there is no second parallel array that
    // could drift out of step with `rows` after the shift. Assert it directly
    // against the underlying `rows`, not just the filtered `items` view.
    assert_eq!(
        ov.rows.len(),
        2,
        "b's row is actually gone, not just hidden"
    );
    let want_secondary = |rel: &str| {
        let size = if rel == "assets/a.png" { 1 } else { 3 };
        crate::assets::secondary_label(&orphan(rel, size))
    };
    for row in &ov.rows {
        assert_eq!(
            row.secondary,
            want_secondary(&row.accept),
            "row {:?}'s own secondary traveled with it, never a neighbor's",
            row.accept
        );
    }
    // Removing a value not present is a calm no-op.
    assert!(!ov.remove_asset_row("assets/zzz.png"));
    assert_eq!(ov.items.len(), 2);
}

#[test]
fn assets_emptying_the_list_shows_the_calm_empty_state() {
    let mut ov = OverlayState::new_assets(vec![orphan("assets/only.png", 1)]);
    assert!(ov.empty_notice().is_none(), "one row → no empty state");
    assert!(ov.remove_asset_row("assets/only.png"));
    // Now empty: the picker stays valid and shows the calm per-kind message.
    assert_eq!(ov.items.len(), 0);
    assert_eq!(ov.empty_notice().as_deref(), Some("no unused assets"));
    // Enter on the empty state is a no-op (nothing selected).
    assert_eq!(ov.selected_value(), None);
}

#[test]
fn assets_empty_corpus_always_summons_with_the_calm_message() {
    let ov = OverlayState::new_assets(vec![]);
    assert_eq!(ov.kind, OverlayKind::Assets);
    assert_eq!(ov.empty_notice().as_deref(), Some("no unused assets"));
}

/// THE LIVE PREVIEW's ONE INPUT: `selected_asset_path` follows the highlight
/// across every row, keyed on each row's own [`crate::overlay::RowMeta::Asset`]
/// — never a `self.kind` check (a bare kind check would pass vacuously if a
/// non-Assets kind's row ever carried the same tag by accident). Swept over
/// first/middle/last so a bug that only shows up mid-list (an off-by-one on
/// `selected_corpus_index`) cannot hide behind a single-row fixture.
#[test]
fn selected_asset_path_follows_the_highlight_first_middle_last() {
    let ov0 = orphan("assets/a.png", 1);
    let ov1 = orphan("assets/b.png", 2);
    let ov2 = orphan("assets/c.png", 3);
    let mut ov = OverlayState::new_assets(vec![ov0.clone(), ov1.clone(), ov2.clone()]);
    assert_eq!(ov.selected, 0, "starts on the first row");
    assert_eq!(ov.selected_asset_path(), Some(ov0.abs.as_path()));
    ov.move_sel(1);
    assert_eq!(ov.selected, 1, "moved to the middle row");
    assert_eq!(ov.selected_asset_path(), Some(ov1.abs.as_path()));
    ov.move_sel(1);
    assert_eq!(ov.selected, 2, "moved to the last row");
    assert_eq!(ov.selected_asset_path(), Some(ov2.abs.as_path()));
    // MUTATION-PROOF: breaking the selection -> preview link (reading the
    // FIRST row's path regardless of `self.selected`, the bug this law
    // exists to catch) makes the last assertion above fail —
    // `ov.selected_asset_path()` would read `Some(ov0.abs.as_path())`
    // (`assets/a.png`) instead of `ov2.abs.as_path()` (`assets/c.png`).

    // Emptying the list (every row removed) leaves nothing selected.
    assert!(ov.remove_asset_row("assets/a.png"));
    assert!(ov.remove_asset_row("assets/b.png"));
    assert!(ov.remove_asset_row("assets/c.png"));
    assert_eq!(
        ov.selected_asset_path(),
        None,
        "no rows -> no preview input"
    );
}
