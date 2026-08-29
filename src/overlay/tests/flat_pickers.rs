use super::*;

#[test]
fn theme_picker_is_flat_and_lists_every_world_with_active_selected() {
    // The theme picker's runtime lens strip was RETIRED (2026-07-15): it is now a
    // FLAT browsable list of every world in THEMES order, no faceting, no sections.
    let names: Vec<String> = crate::theme::THEMES
        .iter()
        .map(|t| t.name.to_string())
        .collect();
    let gum = names.iter().position(|n| n == "Gumtree").unwrap();
    let mut ov = OverlayState::new_theme(names.clone(), gum);
    assert_eq!(ov.kind.as_str(), "theme");
    assert_eq!(
        ov.audition,
        crate::overlay::Audition::Theme { original: gum },
        "the opening world is remembered for revert"
    );
    // FLAT: no facet scheme, no lens strip, no section labels.
    assert!(!ov.is_faceting(), "the theme picker does not facet");
    assert!(ov.active_facet_id().is_none(), "no active lens");
    assert!(ov.lens_strip().is_empty(), "no lens strip");
    assert!(
        ov.item_sections().iter().all(|s| s.is_empty()),
        "no section grouping"
    );
    // Every world is listed, in THEMES declaration order, and the active world opens
    // selected (so it is highlighted + previewable with no move).
    assert_eq!(
        ov.item_strings(),
        names,
        "flat list = every world in THEMES order"
    );
    assert_eq!(
        ov.selected_value(),
        Some("Gumtree"),
        "active world opens selected"
    );
    // No git / dir markers on the theme rows.
    assert!(
        ov.item_strings()
            .iter()
            .all(|s| !s.contains('•') && !s.ends_with('/'))
    );
    // cycle_lens is inert on a non-faceting picker (it grew no strip to cycle).
    ov.cycle_lens(1);
    assert_eq!(ov.facet_lens, 0);
    assert!(ov.active_facet_id().is_none());
    assert_eq!(
        ov.item_strings(),
        names,
        "cycle_lens did not regroup the flat list"
    );
}

/// The CLICKABLE lens strip's pointing counterpart to a no-op LEFT/RIGHT at an
/// end: clicking the ALREADY-ACTIVE facet is a calm no-op (documented on
/// `set_facet_lens` itself) — `facet_lens`, the selected item, and the scroll
/// position all stay byte-identical, unlike a real switch (which regroups the
/// list and can move `selected`/`scroll`).
#[test]
fn clicking_the_current_facet_is_a_calm_no_op() {
    // Driven over a still-faceting picker (the Command palette) — the theme picker
    // retired its lens strip, so this generic law now rides a surviving faceter.
    let names = crate::commands::names();
    let hidden = vec![false; names.len()];
    let mut ov = OverlayState::new_command(
        names,
        crate::commands::effective_bindings(&[], &[], crate::keymap::KeymapFlavor::Native),
        hidden,
    );
    ov.set_facet_lens(2); // switch to Navigate once, a real change
    assert_eq!(ov.active_facet_id(), Some("navigate"));
    let (before_lens, before_selected, before_scroll, before_items) =
        (ov.facet_lens, ov.selected, ov.scroll, ov.item_strings());
    ov.set_facet_lens(2); // click the SAME facet again — a calm no-op
    assert_eq!(ov.facet_lens, before_lens);
    assert_eq!(ov.selected, before_selected);
    assert_eq!(ov.scroll, before_scroll);
    assert_eq!(ov.item_strings(), before_items);
}

#[test]
fn flat_pickers_have_no_lens_strip() {
    // A non-faceting picker never grows a lens strip or section labels, and has no
    // facet scheme (so `active_facet_id` is None). Both the Caret picker and — since
    // 2026-07-15 — the THEME picker are flat, non-faceting examples (Goto / Browse /
    // Project / Command / History / Settings still facet — see `facets::scheme`).
    let names: Vec<String> = crate::theme::THEMES
        .iter()
        .map(|t| t.name.to_string())
        .collect();
    let theme = OverlayState::new_theme(names, 0);
    for mut ov in [
        OverlayState::new(OverlayKind::Caret, corpus(), vec![], vec![]),
        theme,
    ] {
        assert!(!ov.is_faceting(), "{:?} must not facet", ov.kind);
        assert!(
            ov.lens_strip().is_empty(),
            "{:?} has no lens strip",
            ov.kind
        );
        assert!(
            ov.active_facet_id().is_none(),
            "{:?} has no active lens",
            ov.kind
        );
        assert!(
            ov.item_sections().iter().all(|s| s.is_empty()),
            "{:?} no sections",
            ov.kind
        );
        // cycle_lens on a non-faceting picker is inert (facet_lens stays 0).
        ov.cycle_lens(1);
        assert_eq!(ov.facet_lens, 0, "{:?} cycle_lens is inert", ov.kind);
    }
}
