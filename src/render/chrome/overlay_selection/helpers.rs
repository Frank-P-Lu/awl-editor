//! Small geometry and color transforms shared by selection emitters and laws.

use super::*;

pub(super) fn apply_living_row_spans(plan: &OverlayRowPlan, rects: &mut [[f32; 4]]) {
    for rect in rects {
        if let Some(row) = plan.display_nearest(rect[1] + rect[3] * 0.5) {
            let dx = plan.row_dx(row);
            let dw = plan.row_dw(row);
            rect[0] += dx;
            rect[2] += dw - dx;
        }
    }
}

pub(super) fn row_focus_rgba(band: crate::theme::Srgb, focused: bool) -> [u8; 4] {
    if focused {
        band.rgba_bytes()
    } else {
        super::super::workspace::dimmed(band, super::super::workspace::UNFOCUSED_MARK_ALPHA)
    }
}
