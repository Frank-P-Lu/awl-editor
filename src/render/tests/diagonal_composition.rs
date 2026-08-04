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

/// A `spine_segment` triple's two ENDPOINTS, recovered from `(center, half, axis)`.
fn ends(seg: ([f32; 2], [f32; 2], [f32; 2])) -> ([f32; 2], [f32; 2]) {
    let (center, half, axis) = seg;
    let (dx, dy) = (axis[0] * half[0], axis[1] * half[0]);
    (
        [center[0] - dx, center[1] - dy],
        [center[0] + dx, center[1] + dy],
    )
}

/// ITEM 247 — THE SELECTED MARK IS A CHEVRON, graded by SHAPE rather than by
/// instance count.
///
/// ⚠️ The three shipping laws that already touch this mark (`list_surfaces`,
/// `settings_row_reach_law`, `row_offset_item131`) each assert
/// `overlay_spine_selected.instance_count() == 2`. **A chevron is ALSO two
/// segments, and it deliberately inscribes the SAME bounding box** as the
/// tick-plus-connector pair it replaced — so all three stay green across a total
/// change of shape, and neither a count nor an extent can tell the two marks
/// apart. The discriminating property is ANGLE: a chevron's arms are both
/// OFF-AXIS and mirror one another, where the old pair was exactly one vertical
/// segment and one horizontal one.
///
/// Swept across row heights, both reach directions and both row origins, because
/// the mirror and the off-axis property must hold for every row the planner can
/// produce — not for one hand-picked geometry. Non-vacuity is proved in
/// [`the_replaced_tick_and_connector_pair_fails_the_chevron_law`] below, which
/// builds the OLD shape and watches the same predicate reject it.
#[test]
fn the_selected_mark_is_an_off_axis_mirrored_chevron_at_every_row_and_reach() {
    const EPS: f32 = 1e-4;
    let mut cases = 0;
    for top in [0.0_f32, 137.5, 1024.0] {
        // Heights stay above the 4px degenerate floor: the mark takes a 2px
        // inset at BOTH ends, so a 4px row collapses both arms onto one
        // horizontal line. No planner produces a row that cannot seat text.
        for height in [12.0_f32, 27.5, 44.0, 88.0] {
            // BOTH signs: a Descending world reaches right, an Ascending one left.
            for reach in [-40.0_f32, -10.0, -3.0, 3.0, 10.0, 40.0] {
                let spine_x = 64.0_f32;
                let (t, b) = (top + 2.0, top + height - 2.0);
                let arms = crate::render::chrome::diagonal::selected_chevron(
                    spine_x,
                    spine_x + reach,
                    t,
                    b,
                    3.0,
                );
                let ctx = format!("top {top} height {height} reach {reach}");
                let mid = (t + b) * 0.5;

                let (upper_start, upper_end) = ends(arms[0]);
                let (lower_start, lower_end) = ends(arms[1]);

                // Both arms START at the vertex, and the vertex sits ON the spine
                // at the row's middle — this is what keeps the mark attached to
                // the line the composition says carries focus.
                for (name, start) in [("upper", upper_start), ("lower", lower_start)] {
                    assert!(
                        (start[0] - spine_x).abs() < EPS && (start[1] - mid).abs() < EPS,
                        "{ctx}: the {name} arm must start at the vertex \
                         ({spine_x}, {mid}), got ({}, {})",
                        start[0],
                        start[1]
                    );
                }

                // The arms reach the SAME x and the row's two inset ends — the
                // bounding box the reservation terms still describe.
                assert!(
                    (upper_end[0] - (spine_x + reach)).abs() < EPS
                        && (lower_end[0] - (spine_x + reach)).abs() < EPS,
                    "{ctx}: both arms must reach x {}",
                    spine_x + reach
                );
                assert!(
                    (upper_end[1] - t).abs() < EPS && (lower_end[1] - b).abs() < EPS,
                    "{ctx}: the arms must land on the row's inset top and bottom"
                );

                // THE DISCRIMINATOR: neither arm is axis-aligned.
                for (name, seg) in [("upper", arms[0]), ("lower", arms[1])] {
                    let axis = seg.2;
                    assert!(
                        axis[0].abs() > EPS && axis[1].abs() > EPS,
                        "{ctx}: the {name} arm is AXIS-ALIGNED ({}, {}) — that is the \
                         tick-plus-connector shape item 247 replaced, not a chevron",
                        axis[0],
                        axis[1]
                    );
                }

                // And the two arms mirror about the horizontal through the vertex.
                let (up_axis, low_axis) = (arms[0].2, arms[1].2);
                assert!(
                    (up_axis[0] - low_axis[0]).abs() < EPS
                        && (up_axis[1] + low_axis[1]).abs() < EPS,
                    "{ctx}: the arms must mirror — got ({}, {}) and ({}, {})",
                    up_axis[0],
                    up_axis[1],
                    low_axis[0],
                    low_axis[1]
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 72, "the sweep must not silently shrink");
}

/// NON-VACUITY for the law above: reconstruct the tick-plus-connector pair that
/// shipped before item 247 and confirm the chevron predicate REJECTS it. Without
/// this, "neither arm is axis-aligned" could be satisfied by an accident of the
/// sweep rather than by the shape actually changing.
#[test]
fn the_replaced_tick_and_connector_pair_fails_the_chevron_law() {
    const EPS: f32 = 1e-4;
    let (spine_x, top, height, reach) = (64.0_f32, 0.0_f32, 27.5_f32, 10.0_f32);
    let (t, b) = (top + 2.0, top + height - 2.0);
    let mid = top + height * 0.5;

    // Exactly what `prepare_diagonal_spine` drew before this item: one VERTICAL
    // tick spanning the row, one HORIZONTAL connector reaching the cluster.
    let tick = crate::selection::spine_segment([spine_x, t], [spine_x, b], 3.0);
    let connector = crate::selection::spine_segment([spine_x, mid], [spine_x + reach, mid], 3.0);

    let axis_aligned =
        |seg: ([f32; 2], [f32; 2], [f32; 2])| seg.2[0].abs() < EPS || seg.2[1].abs() < EPS;
    assert!(
        axis_aligned(tick) && axis_aligned(connector),
        "the pre-247 mark must be two AXIS-ALIGNED segments — if this ever stops \
         being true the chevron law above has lost its discriminating power"
    );

    // And it spans the same bounding box the chevron does, which is precisely why
    // an extent-based or count-based law cannot separate them.
    let chevron =
        crate::render::chrome::diagonal::selected_chevron(spine_x, spine_x + reach, t, b, 3.0);
    let xs = |segs: &[([f32; 2], [f32; 2], [f32; 2])]| {
        segs.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &s| {
            let (a, z) = ends(s);
            (lo.min(a[0]).min(z[0]), hi.max(a[0]).max(z[0]))
        })
    };
    assert_eq!(
        xs(&[tick, connector]),
        xs(&chevron),
        "the chevron must inscribe the old mark's horizontal extent, so the \
         reservation terms need no compensating adjustment"
    );
}
