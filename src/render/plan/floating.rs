//! Pure placement for transient chrome whose text or grid is already measured.
//!
//! Shaping stays with the surface that owns the glyph buffer. This module begins
//! at the measurement boundary and owns the per-frame boxes handed to paint,
//! hit testing, and reports.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) enum CornerAnchor {
    TopRight,
    BottomRight,
    TopCenter,
    AtPoint(f32, f32),
}

#[allow(clippy::too_many_arguments)]
pub(in crate::render) fn plan_corner_label(
    anchor: CornerAnchor,
    text_w: f32,
    line_height: f32,
    width: f32,
    height: f32,
    col_left: f32,
    col_width: f32,
    top_reserve: f32,
    canvas_inset: f32,
) -> [f32; 4] {
    let (left, top) = match anchor {
        CornerAnchor::TopRight => (
            (width - text_w - canvas_inset).max(canvas_inset),
            canvas_inset + top_reserve,
        ),
        CornerAnchor::BottomRight => (
            (col_left + col_width - text_w).max(col_left),
            height - line_height - canvas_inset,
        ),
        CornerAnchor::TopCenter => (
            (col_left + (col_width - text_w) * 0.5).max(col_left),
            top_reserve,
        ),
        CornerAnchor::AtPoint(px, py) => (
            (px + 14.0).min(width - text_w - 4.0).max(4.0),
            (py - line_height - 10.0).max(4.0),
        ),
    };
    [left, top, text_w, line_height]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct FloatCardPlan {
    pub card: [f32; 4],
    pub text: [f32; 2],
}

pub(in crate::render) fn plan_float_card(
    canvas: [f32; 2],
    measured: [f32; 2],
    pad: [f32; 2],
    min_top: f32,
) -> FloatCardPlan {
    let card_w = measured[0] + 2.0 * pad[0];
    let card_h = measured[1] + 2.0 * pad[1];
    let text_top = ((canvas[1] - measured[1]) * 0.5).max(min_top);
    let card_x = (canvas[0] - card_w) * 0.5;
    let card_y = text_top - pad[1];
    FloatCardPlan {
        card: [card_x, card_y, card_w, card_h],
        text: [card_x + pad[0], text_top],
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct WhichKeyCardPlan {
    pub card: [f32; 4],
    pub text: [f32; 2],
}

pub(in crate::render) fn plan_whichkey_card(
    canvas_h: f32,
    measured: [f32; 2],
    pad: [f32; 2],
    margin: f32,
) -> WhichKeyCardPlan {
    let card_w = measured[0] + 2.0 * pad[0];
    let card_h = measured[1] + 2.0 * pad[1];
    let card_x = margin;
    let card_y = (canvas_h - margin - card_h).max(margin);
    WhichKeyCardPlan {
        card: [card_x, card_y, card_w, card_h],
        text: [card_x + pad[0], card_y + pad[1]],
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct StreaksCardInput {
    pub canvas: [f32; 2],
    pub text: [f32; 2],
    pub grid: [f32; 2],
    pub pad: [f32; 2],
    pub dot: f32,
    pub gap_dots: f32,
    pub gap_between: f32,
    pub min_card_top: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct StreaksCardPlan {
    pub card: [f32; 4],
    pub grid: [f32; 4],
    pub dots_y: f32,
    pub text: [f32; 2],
}

pub(in crate::render) fn plan_streaks_card(input: StreaksCardInput) -> StreaksCardPlan {
    let content_w = input.grid[0].max(input.text[0]);
    let content_h = input.grid[1] + input.gap_dots + input.dot + input.gap_between + input.text[1];
    let card_w = content_w + 2.0 * input.pad[0];
    let card_h = content_h + 2.0 * input.pad[1];
    let card_x = ((input.canvas[0] - card_w) * 0.5).max(0.0);
    let card_y = ((input.canvas[1] - card_h) * 0.5).max(input.min_card_top);
    let grid_x = card_x + (card_w - input.grid[0]) * 0.5;
    let grid_y = card_y + input.pad[1];
    let dots_y = grid_y + input.grid[1] + input.gap_dots;
    let text_top = dots_y + input.dot + input.gap_between;
    StreaksCardPlan {
        card: [card_x, card_y, card_w, card_h],
        grid: [grid_x, grid_y, input.grid[0], input.grid[1]],
        dots_y,
        text: [grid_x, text_top],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_anchors_hold_extents_through_canvas_transitions() {
        for width in [90.0, 240.0, 1200.0] {
            for anchor in [
                CornerAnchor::TopRight,
                CornerAnchor::BottomRight,
                CornerAnchor::TopCenter,
                CornerAnchor::AtPoint(width - 1.0, 2.0),
            ] {
                let p = plan_corner_label(anchor, 80.0, 20.0, width, 180.0, 30.0, 120.0, 12.0, 8.0);
                assert_eq!(p[2..], [80.0, 20.0]);
                assert!(p[0].is_finite() && p[1].is_finite());
            }
        }
        assert_eq!(
            plan_corner_label(
                CornerAnchor::BottomRight,
                80.0,
                20.0,
                240.0,
                180.0,
                30.0,
                120.0,
                0.0,
                8.0
            ),
            [70.0, 152.0, 80.0, 20.0]
        );
    }

    #[test]
    fn float_and_whichkey_cards_follow_measured_extents() {
        let a = plan_float_card([1200.0, 800.0], [200.0, 80.0], [24.0, 16.0], 16.0);
        let b = plan_float_card([1200.0, 800.0], [260.0, 120.0], [24.0, 16.0], 16.0);
        assert_eq!(b.card[2] - a.card[2], 60.0);
        assert_eq!(b.card[3] - a.card[3], 40.0);
        assert_eq!(a.text[0], a.card[0] + 24.0);
        let top = plan_whichkey_card(800.0, [180.0, 100.0], [20.0, 12.0], 24.0);
        let clamped = plan_whichkey_card(120.0, [180.0, 100.0], [20.0, 12.0], 24.0);
        assert_eq!(top.card[1] + top.card[3] + 24.0, 800.0);
        assert_eq!(clamped.card[1], 24.0);
    }

    #[test]
    fn streaks_plan_tiles_grid_dots_and_text_without_overlap() {
        let p = plan_streaks_card(StreaksCardInput {
            canvas: [900.0, 700.0],
            text: [260.0, 90.0],
            grid: [300.0, 120.0],
            pad: [30.0, 20.0],
            dot: 8.0,
            gap_dots: 12.0,
            gap_between: 18.0,
            min_card_top: -4.0,
        });
        assert_eq!(p.card[2], 360.0);
        assert_eq!(p.grid[1] + p.grid[3] + 12.0, p.dots_y);
        assert_eq!(p.dots_y + 8.0 + 18.0, p.text[1]);
        assert!(p.text[1] + 90.0 <= p.card[1] + p.card[3] - 20.0 + 0.001);
    }
}
