//! Hit-testing law for the SINGLE-FILE identity line — it now enrols in the
//! same close/switch geometry a working-set row does
//! ([`GutterLine::Name`] in [`stack_hit_from_plan`]), so this proves the
//! resolved row/zone split matches a real working-set row's for the one shape
//! `gutter_stack::tests`' own fixtures never draw: a block with no stack at
//! all.

use super::*;

fn one_file_layout(avail: f32, project: bool, changed: bool) -> GutterLayout {
    GutterLayout {
        avail,
        name: "opening.md".to_string(),
        project: if project {
            "notes".to_string()
        } else {
            String::new()
        },
        changed: if changed {
            "changed elsewhere".to_string()
        } else {
            String::new()
        },
        files: Vec::new(),
    }
}

/// **THE LONE IDENTITY ROW RESOLVES THE SAME CLOSE/SWITCH SPLIT A WORKING-SET
/// ROW WOULD.**
///
/// Swept over the margin widths a real window produces, mirroring
/// `gutter_stack::tests::close_zone_hugs_the_rows_own_leading_ink_edge_and_the_rest_switches`'s
/// own sweep — the geometry underneath is the SAME function
/// ([`gutter_stack::row_intent`]), so a width-dependent bug in one is a
/// width-dependent bug in the other, and this proves the single-file margin
/// was actually wired to it rather than merely documented as such. ALSO swept
/// over the `changed elsewhere` affordance, which shifts the identity line's
/// own index in the drawn block (`hover_close_keeps_label_geometry_fixed_
/// and_enrols_every_truthful_row`'s own axis) — a resolution keyed to a fixed
/// line number rather than found by kind would pass every case here except
/// the one with a conflict raised.
#[test]
fn the_lone_identity_row_resolves_the_same_close_zone_a_stack_row_would() {
    // Small enough that the fixture's own fixed "opening.md" name (never
    // re-fit to `avail` here, unlike production's `gutter_layout`) keeps a
    // positive ink leading edge even at the narrowest `avail` this sweeps —
    // only internal consistency with `close_zone`'s own call below matters,
    // not a realistic pixel width.
    let label_char_w = 1.0;
    for avail in [14.0_f32, 20.0, 48.0, 96.0, 120.0, 300.0] {
        for changed in [false, true] {
            let layout = one_file_layout(avail, true, changed);
            let plan = crate::render::plan::plan_gutter_stack(
                300.0,
                avail,
                12.0,
                layout.lines().len(),
                8.0,
                0.5,
            );
            let row_line = layout
                .lines()
                .iter()
                .position(|(_, k)| matches!(k, GutterLine::Name))
                .expect("the one-file block always draws a Name line");
            let band = plan.rows[row_line];
            let mark_chars = gutter_stack::CLOSE_MARK_TEXT.chars().count();
            let text_w = (layout.name.chars().count() + mark_chars) as f32 * label_char_w;
            let mark_w = mark_chars as f32 * label_char_w;
            let zone = gutter_stack::close_zone(band, text_w, mark_w);
            let label = format!("avail={avail} changed={changed}");
            let switch = stack_hit_from_plan(
                &layout,
                &plan,
                label_char_w,
                zone[0] - 1.0,
                band[1] + band[3] * 0.5,
            )
            .unwrap_or_else(|| panic!("{label}: switch point does not enrol"));
            let close = stack_hit_from_plan(
                &layout,
                &plan,
                label_char_w,
                zone[0] + 1.0,
                band[1] + band[3] * 0.5,
            )
            .unwrap_or_else(|| panic!("{label}: close point does not enrol"));
            assert_eq!(switch.row, 0, "{label}: the lone open file is always row 0");
            assert!(
                !switch.is_close(),
                "{label}: left of the close zone must switch"
            );
            assert_eq!(close.row, 0, "{label}: the lone open file is always row 0");
            assert!(
                close.is_close(),
                "{label}: inside the close zone must close"
            );
        }
    }
}

/// **ONLY THE IDENTITY LINE NAMES A FILE.** The `changed elsewhere`
/// affordance and the project heading enrol in the block's own click-through
/// (`gutter_context_target`'s `Filename`/`Folder` targets), never in the
/// row/zone geometry a close or switch reads — a hit-test that resolved the
/// heading to a row would let a click on the folder name close the document
/// it never named.
#[test]
fn only_the_identity_line_enrols_the_lone_margin_in_row_click_geometry() {
    let layout = one_file_layout(120.0, true, false);
    let plan = crate::render::plan::plan_gutter_stack(
        300.0,
        layout.avail,
        12.0,
        layout.lines().len(),
        8.0,
        0.5,
    );
    let heading_line = layout
        .lines()
        .iter()
        .position(|(_, k)| matches!(k, GutterLine::Project))
        .expect("a project line is drawn");
    let band = plan.rows[heading_line];
    assert!(
        stack_hit_from_plan(
            &layout,
            &plan,
            6.0,
            band[0] + band[2] - 1.0,
            band[1] + band[3] * 0.5
        )
        .is_none(),
        "the project heading must not resolve to a clickable row"
    );
}
