//! `overlay_selection.rs`'s own unit laws, carved out to a sibling file so that
//! module's size measures production code. A sibling `tests.rs` is exempt from the
//! production ceiling by `code-health.py`'s `production()` rule, whose own comment
//! notes that counting one as production would defeat the point of the carve.

use super::*;
use crate::render::plan::{OverlayRowPlanInput, plan_overlay_rows};

#[test]
fn unfocused_selection_stays_present_and_its_label_ink_stays_legible_in_every_world() {
    let _guard = crate::testlock::serial();
    for (index, world) in crate::theme::THEMES.iter().enumerate() {
        crate::theme::set_active(index);
        let band = super::super::overlay_selected_band_srgb();
        let full = row_focus_rgba(band, true);
        let quiet = row_focus_rgba(band, false);
        assert!(
            quiet[3] >= 80,
            "{}: focus cue disappeared: {quiet:?}",
            world.name
        );
        assert!(
            quiet[3] < full[3],
            "{}: focused and selected collapsed",
            world.name
        );
        let ink = super::super::overlay_selected_label_ink();
        assert_eq!(
            ink.a(),
            255,
            "{}: selected label ink lost opacity",
            world.name
        );
    }
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

#[test]
fn twoshape_echo_uses_its_own_nearest_planned_row_span() {
    let plan = plan_overlay_rows(&OverlayRowPlanInput {
        card_x: 100.0,
        card_w: 300.0,
        text_top: 40.0,
        lh: 20.0,
        header_gap: 0.0,
        header_rows: 0,
        billed_header_rows: 0,
        visible: 3,
        top_idx: 0,
        n_items: 3,
        selected: 2,
        empty_rows: 0,
        lines: None,
        dx_per_row: 10.0,
        cluster_span: None,
        selected_offset: None,
        selected_display: None,
        cue_above_rows: 0,
        cue_below_rows: 0,
    });
    // These are the leading, echo, and overlap bands from one TwoShape
    // crossing. Their centres deliberately belong to three different rows.
    let mut rects = [
        [100.0, 40.0, 300.0, 20.0],
        [100.0, 60.0, 300.0, 20.0],
        [100.0, 80.0, 300.0, 20.0],
    ];
    apply_living_row_spans(&plan, &mut rects);
    assert_eq!(rects[0], [100.0, 40.0, 300.0, 20.0]);
    assert_eq!(rects[1], [110.0, 60.0, 290.0, 20.0]);
    assert_eq!(rects[2], [120.0, 80.0, 280.0, 20.0]);
}
