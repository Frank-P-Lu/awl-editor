//! THE DIAGONAL COMPOSITION's own laws: one logical→device boundary, two
//! mirrored orientations from one owner, and world identity living in theme
//! data rather than in the renderer.

use super::super::*;
use crate::render::chrome::diagonal::DiagonalComposition;

#[test]
fn logical_registry_scales_every_diagonal_quantity_once_with_dpi() {
    let one = DiagonalComposition::resolve(theme::DiagonalDirection::Descending, 1.0);
    let two = DiagonalComposition::resolve(theme::DiagonalDirection::Descending, 2.0);
    assert_eq!(two.row_step, one.row_step * 2.0);
    assert_eq!(two.spine_weight, one.spine_weight * 2.0);
    assert_eq!(two.spine_corner, one.spine_corner * 2.0);
    assert_eq!(two.attachment_inset, one.attachment_inset * 2.0);
    assert_eq!(two.selected_outward, one.selected_outward * 2.0);
    assert_eq!(two.selected_spine_weight, one.selected_spine_weight * 2.0);
    assert!(
        one.row_step > 0.0,
        "the diagonal law must not pass on an inert composition"
    );
}

#[test]
fn one_composition_owns_the_two_mirrored_orientations() {
    let down = DiagonalComposition::resolve(theme::DiagonalDirection::Descending, 1.0);
    let up = DiagonalComposition::resolve(theme::DiagonalDirection::Ascending, 1.0);
    assert_eq!(down.row_step, -up.row_step);
    assert_eq!(down.spine_weight, up.spine_weight);
    assert_eq!(down.spine_corner, up.spine_corner);
    assert_eq!(down.attachment_inset, up.attachment_inset);
    assert_eq!(down.selected_outward, up.selected_outward);
    assert_eq!(down.selected_spine_weight, up.selected_spine_weight);
    assert!(
        down.selected_spine_weight > down.spine_weight,
        "the selected local spine must visibly thicken over its resting stroke"
    );
}

#[test]
fn only_world_data_names_mangrove_and_magpie() {
    let _g = crate::testlock::serial();
    let renderer = include_str!("../chrome/diagonal.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap();
    assert!(!renderer.contains("Mangrove"));
    assert!(!renderer.contains("Magpie"));

    let mut diagonal = Vec::new();
    for world in theme::THEMES {
        match world.render_caps.list_style {
            theme::ListStyle::Diagonal(direction) => diagonal.push((world.name, direction)),
            theme::ListStyle::Pane | theme::ListStyle::Bars { .. } => {}
        }
    }
    assert_eq!(
        diagonal,
        vec![
            ("Mangrove", theme::DiagonalDirection::Descending),
            ("Magpie", theme::DiagonalDirection::Ascending),
        ]
    );
    for name in ["Mangrove", "Magpie"] {
        theme::set_active_by_name(name).unwrap();
        assert_ne!(
            theme::muted(),
            theme::primary(),
            "{name}: spine ink is never the accent"
        );
        assert_ne!(
            theme::muted(),
            theme::base_content(),
            "{name}: the selected local spine must brighten over the resting muted ink"
        );
    }
    theme::set_active(theme::DEFAULT_THEME);
}
