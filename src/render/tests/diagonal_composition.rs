//! THE DIAGONAL COMPOSITION's own laws: one logical→device boundary, two
//! mirrored orientations from one owner, and world identity living in theme
//! data rather than in the renderer.

use super::super::*;
use crate::render::chrome::diagonal::DiagonalComposition;

/// A spine authored the way a world authors one, so every law below resolves
/// through the same door `chrome::diagonal::active` uses. The MARK is a
/// parameter rather than a fixture constant: a law that pinned one mark could
/// not see a composition that ignored the world's authorship.
fn spine(direction: theme::DiagonalDirection, mark: theme::DiagonalMark) -> theme::DiagonalSpine {
    match direction {
        theme::DiagonalDirection::Descending => theme::DiagonalSpine::descending(mark),
        theme::DiagonalDirection::Ascending => theme::DiagonalSpine::ascending(mark),
    }
}

/// Every diagonal world's own authored mark, read off the ROSTER rather than
/// named — a third diagonal world joins these laws by shipping, not by being
/// added to a list here.
fn authored_marks() -> Vec<(&'static str, theme::DiagonalSpine)> {
    let mut out = Vec::new();
    for world in theme::THEMES {
        match world.render_caps.list_style {
            theme::ListStyle::Diagonal(spine) => out.push((world.name, spine)),
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Rules(_) => {}
        }
    }
    out
}

#[test]
fn logical_registry_scales_every_diagonal_quantity_once_with_dpi() {
    let authored = spine(
        theme::DiagonalDirection::Descending,
        theme::DiagonalMark::CRISP,
    );
    let one = DiagonalComposition::resolve(authored, 1.0);
    let two = DiagonalComposition::resolve(authored, 2.0);
    assert_eq!(two.row_step, one.row_step * 2.0);
    assert_eq!(two.spine_weight, one.spine_weight * 2.0);
    assert_eq!(two.spine_corner, one.spine_corner * 2.0);
    assert_eq!(two.attachment_inset, one.attachment_inset * 2.0);
    // The connector was the one member the registry law never graded.
    assert_eq!(two.connector, one.connector * 2.0);
    assert_eq!(two.selected_outward, one.selected_outward * 2.0);
    // The world-authored mark passes the SAME boundary as the composition's own
    // lengths — it is authored in logical pixels beside the world's face, not in
    // device pixels.
    assert_eq!(two.mark_weight, one.mark_weight * 2.0);
    assert_eq!(two.mark_gap, one.mark_gap * 2.0);
    assert_eq!(two.mark_reach, one.mark_reach * 2.0);
    assert_eq!(two.mark_row_inset, one.mark_row_inset * 2.0);
    assert_eq!(two.mark_lane(), one.mark_lane() * 2.0);
    // And its APERTURE does not: a fraction of a row is not a length.
    assert_eq!(two.mark_aperture, one.mark_aperture);
    assert!(
        one.row_step > 0.0,
        "the diagonal law must not pass on an inert composition"
    );
}

#[test]
fn one_composition_owns_the_two_mirrored_orientations() {
    let mark = theme::DiagonalMark::CRISP;
    let down = DiagonalComposition::resolve(spine(theme::DiagonalDirection::Descending, mark), 1.0);
    let up = DiagonalComposition::resolve(spine(theme::DiagonalDirection::Ascending, mark), 1.0);
    assert_eq!(down.row_step, -up.row_step);
    assert_eq!(down.spine_weight, up.spine_weight);
    assert_eq!(down.spine_corner, up.spine_corner);
    assert_eq!(down.attachment_inset, up.attachment_inset);
    assert_eq!(down.connector, up.connector);
    assert_eq!(down.selected_outward, up.selected_outward);
    // The MARK is world data, so the two orientations given the SAME mark must
    // resolve it identically: the mirror lives in the sign of `row_step` and in
    // the cluster's own outward dial, never in the mark's dimensions.
    assert_eq!(down.mark_weight, up.mark_weight);
    assert_eq!(down.mark_gap, up.mark_gap);
    assert_eq!(down.mark_reach, up.mark_reach);
    assert_eq!(down.mark_aperture, up.mark_aperture);
}

/// A compact mark spends less empty lane. This is composition, not a Magpie
/// placement exception: full-aperture marks retain the former shared gap while
/// every smaller aperture moves its vertex inward by the same proportion.
#[test]
fn marker_gap_tracks_the_authored_mark_aperture() {
    let full = DiagonalComposition::resolve(
        spine(theme::DiagonalDirection::Descending, theme::DiagonalMark::CRISP),
        1.0,
    );
    let compact = DiagonalComposition::resolve(
        spine(theme::DiagonalDirection::Ascending, theme::DiagonalMark::HAIRLINE),
        1.0,
    );
    assert_eq!(full.mark_gap, 7.0);
    assert_eq!(compact.mark_gap, full.mark_gap * compact.mark_aperture);
    assert!(compact.mark_gap < full.mark_gap);
}

/// THE PRESENCE FLOOR the mark's own dimensions answer to — the companion a
/// contrast or thickness claim needs, because "thinner" is satisfiable all the
/// way down to nothing.
///
/// Enrolment is the ROSTER's, at the tightest real value it ships: every world
/// that authors a diagonal spine must author a mark whose stroke still covers a
/// device pixel at `scale == 1.0`, whose aperture still spans a real fraction of
/// its row, and whose lane leaves the cluster's outer end genuinely clear. A
/// mark tuned to a hairline for one world's face cannot fade to a mark nobody
/// can see.
#[test]
fn every_authored_diagonal_mark_clears_the_presence_floor() {
    let marks = authored_marks();
    assert!(
        marks.len() >= 2,
        "the roster sweep found {} diagonal worlds — it is not reading the \
         roster it thinks it is",
        marks.len()
    );
    for (name, authored) in marks {
        let c = DiagonalComposition::resolve(authored, 1.0);
        assert!(
            c.mark_weight >= 1.0,
            "{name}: an authored mark stroke of {} logical px cannot cover a \
             device pixel at 1x — a mark that thin is not thinner, it is absent",
            c.mark_weight
        );
        assert!(
            c.mark_aperture > 0.25 && c.mark_aperture <= 1.0,
            "{name}: aperture {} is outside the readable band (0.25, 1.0]",
            c.mark_aperture
        );
        assert!(
            c.mark_reach > 1.0 && c.mark_gap > 1.0,
            "{name}: reach {} / gap {} — the mark must be a shape standing clear \
             of the cluster, not a dot against it",
            c.mark_reach,
            c.mark_gap
        );
        assert!(
            c.mark_lane() > c.mark_reach * 2.0,
            "{name}: the reserved lane must exceed the mark's own extent, or the \
             gap it stands off by is not reserved at all"
        );
    }
}

/// THE SPLIT THIS ROUND EXISTS FOR — one mark cannot serve two display faces,
/// so the two shipping diagonal worlds must author DIFFERENT marks.
///
/// Derived from the roster and from each world's own `Theme::font`, never from a
/// name: the claim is "worlds with different display faces carry different
/// marks", and a shared renderer constant — the exact state this replaced —
/// fails it by making every entry identical.
#[test]
fn diagonal_worlds_with_different_display_faces_author_different_marks() {
    let mut by_face: Vec<(&str, &str, theme::DiagonalMark)> = Vec::new();
    for world in theme::THEMES {
        if let theme::ListStyle::Diagonal(spine) = world.render_caps.list_style {
            by_face.push((world.name, world.font, spine.mark));
        }
    }
    assert!(
        by_face.len() >= 2,
        "fewer than two diagonal worlds enrolled: {by_face:?}"
    );
    for (i, (a_name, a_face, a_mark)) in by_face.iter().enumerate() {
        for (b_name, b_face, b_mark) in by_face.iter().skip(i + 1) {
            if a_face == b_face {
                continue;
            }
            assert_ne!(
                a_mark, b_mark,
                "{a_name} ({a_face}) and {b_name} ({b_face}) draw different \
                 display faces and yet author the IDENTICAL selected-row mark. \
                 A geometric mark right for a technical face contradicts an \
                 editorial one; the weight and form are world data for exactly \
                 this reason."
            );
        }
    }
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
            theme::ListStyle::Diagonal(s) => diagonal.push((world.name, s.direction)),
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Rules(_) => {}
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

/// THE SELECTED MARK IS A CHEVRON, graded by SHAPE rather than by
/// instance count.
///
/// ⚠️ The three shipping laws that already touch this mark (`list_surfaces`,
/// `settings_row_reach_law`, `row_offset`) each assert
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
                let vertex_x = 64.0_f32;
                let (t, b) = (top + 2.0, top + height - 2.0);
                let arms = crate::render::chrome::diagonal::selected_chevron(
                    vertex_x,
                    vertex_x + reach,
                    t,
                    b,
                    3.0,
                );
                let ctx = format!("top {top} height {height} reach {reach}");
                let mid = (t + b) * 0.5;

                let (upper_start, upper_end) = ends(arms[0]);
                let (lower_start, lower_end) = ends(arms[1]);

                // Both arms START at the vertex, at the row's middle — this is
                // what keeps the mark pointing back into the row it marks
                // rather than reading as loose ink in the margin.
                for (name, start) in [("upper", upper_start), ("lower", lower_start)] {
                    assert!(
                        (start[0] - vertex_x).abs() < EPS && (start[1] - mid).abs() < EPS,
                        "{ctx}: the {name} arm must start at the vertex \
                         ({vertex_x}, {mid}), got ({}, {})",
                        start[0],
                        start[1]
                    );
                }

                // The arms reach the SAME x and the row's two inset ends — the
                // bounding box the reservation terms still describe.
                assert!(
                    (upper_end[0] - (vertex_x + reach)).abs() < EPS
                        && (lower_end[0] - (vertex_x + reach)).abs() < EPS,
                    "{ctx}: both arms must reach x {}",
                    vertex_x + reach
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
/// preceded the chevron and confirm the chevron predicate REJECTS it. Without
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
