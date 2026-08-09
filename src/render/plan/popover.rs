//! Pure post-measure planning for the contextual formatting popover.

use crate::popover::PopoverButton;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::render) struct MeasuredPopoverButton {
    pub button: PopoverButton,
    pub span: [f32; 2],
    pub active: bool,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::render) struct PopoverButtonGeom {
    pub button: PopoverButton,
    pub x0: f32,
    pub x1: f32,
    pub active: bool,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::render) struct PopoverGeom {
    pub card: [f32; 4],
    pub text_top: f32,
    pub band_top: f32,
    pub band_h: f32,
    pub buttons: Vec<PopoverButtonGeom>,
}

pub(in crate::render) struct PopoverPlanInput {
    pub canvas: [f32; 2],
    pub measured_width: f32,
    pub band_top_rel: f32,
    pub band_h: f32,
    pub pad: [f32; 2],
    pub anchor_gap: f32,
    pub edge_pad: f32,
    /// Selection start x, row top, and row height.
    pub anchor: [f32; 3],
    pub buttons: Vec<MeasuredPopoverButton>,
}

pub(in crate::render) fn plan_popover(input: PopoverPlanInput) -> PopoverGeom {
    let card_w = input.measured_width + 2.0 * input.pad[0];
    let card_h = input.band_h + 2.0 * input.pad[1];
    let [sel_x, sel_top, sel_row_h] = input.anchor;

    let mut card_y = sel_top - input.anchor_gap - card_h;
    if card_y < input.anchor_gap {
        card_y = sel_top + sel_row_h + input.anchor_gap;
    }
    card_y = card_y
        .min(input.canvas[1] - card_h - input.anchor_gap)
        .max(input.anchor_gap);
    let card_x = (sel_x - card_w * 0.5)
        .min(input.canvas[0] - card_w - input.edge_pad)
        .max(input.edge_pad);

    let text_left = card_x + input.pad[0];
    let band_top = card_y + input.pad[1];
    let text_top = band_top - input.band_top_rel;
    let buttons = input
        .buttons
        .into_iter()
        .map(|button| {
            let [rx0, rx1] = if button.span[0] <= button.span[1] {
                button.span
            } else {
                [0.0, input.measured_width]
            };
            PopoverButtonGeom {
                button: button.button,
                x0: text_left + rx0,
                x1: text_left + rx1,
                active: button.active,
                label: button.label,
            }
        })
        .collect();
    PopoverGeom {
        card: [card_x, card_y, card_w, card_h],
        text_top,
        band_top,
        band_h: input.band_h,
        buttons,
    }
}

/// Place a measured spell card against its measured word rectangle.
pub(in crate::render) fn plan_spell_anchor(
    canvas: [f32; 2],
    word: [f32; 4],
    card: [f32; 2],
    margin: f32,
    gap: f32,
) -> [f32; 2] {
    let mut card_x = word[0];
    if card_x + card[0] > canvas[0] - margin {
        card_x = (canvas[0] - margin - card[0]).max(margin);
    }
    card_x = card_x.max(margin);
    let below_y = word[1] + word[3] + gap;
    let card_y = if below_y + card[1] <= canvas[1] - margin {
        below_y
    } else {
        (word[1] - gap - card[1]).max(margin)
    };
    [card_x, card_y]
}

impl PopoverGeom {
    pub(in crate::render) fn contains(&self, px: f32, py: f32) -> bool {
        let [x, y, w, h] = self.card;
        px >= x && px <= x + w && py >= y && py <= y + h
    }

    pub(in crate::render) fn hit(&self, px: f32, py: f32) -> Option<PopoverButton> {
        if !self.contains(px, py) {
            return None;
        }
        let [card_x, _, card_w, _] = self.card;
        for (index, button) in self.buttons.iter().enumerate() {
            let lo = if index == 0 {
                card_x
            } else {
                (self.buttons[index - 1].x1 + button.x0) * 0.5
            };
            let hi = if index + 1 == self.buttons.len() {
                card_x + card_w
            } else {
                (button.x1 + self.buttons[index + 1].x0) * 0.5
            };
            if px >= lo && px <= hi {
                return Some(button.button);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(anchor: [f32; 3]) -> PopoverPlanInput {
        PopoverPlanInput {
            canvas: [500.0, 300.0],
            measured_width: 120.0,
            band_top_rel: 4.0,
            band_h: 18.0,
            pad: [12.0, 7.0],
            anchor_gap: 8.0,
            edge_pad: 6.0,
            anchor,
            buttons: vec![
                MeasuredPopoverButton {
                    button: PopoverButton::Bold,
                    span: [0.0, 20.0],
                    active: false,
                    label: "B".into(),
                },
                MeasuredPopoverButton {
                    button: PopoverButton::Italic,
                    span: [40.0, 60.0],
                    active: true,
                    label: "I".into(),
                },
            ],
        }
    }

    #[test]
    fn popover_prefers_above_then_flips_below_at_the_top_transition() {
        let above = plan_popover(inputs([250.0, 160.0, 22.0]));
        let below = plan_popover(inputs([250.0, 20.0, 22.0]));
        assert_eq!(above.card, [178.0, 120.0, 144.0, 32.0]);
        assert_eq!(below.card, [178.0, 50.0, 144.0, 32.0]);
        assert_eq!(above.band_top - above.card[1], 7.0);
        assert_eq!(above.text_top, above.band_top - 4.0);
    }

    #[test]
    fn popover_clamps_to_canvas_and_hit_regions_tile_the_card() {
        let left = plan_popover(inputs([0.0, 160.0, 22.0]));
        let right = plan_popover(inputs([500.0, 160.0, 22.0]));
        assert_eq!(left.card[0], 6.0);
        assert_eq!(right.card[0] + right.card[2], 494.0);
        let y = left.card[1] + 1.0;
        assert_eq!(left.hit(left.card[0], y), Some(PopoverButton::Bold));
        let midpoint = (left.buttons[0].x1 + left.buttons[1].x0) * 0.5;
        assert_eq!(left.hit(midpoint - 0.01, y), Some(PopoverButton::Bold));
        assert_eq!(left.hit(midpoint + 0.01, y), Some(PopoverButton::Italic));
        assert_eq!(
            left.hit(left.card[0] + left.card[2], y),
            Some(PopoverButton::Italic)
        );
        assert_eq!(left.hit(left.card[0] - 0.01, y), None);
    }

    #[test]
    fn popover_planner_owns_the_degenerate_measured_span_fallback() {
        let mut input = inputs([250.0, 160.0, 22.0]);
        input.buttons[0].span = [f32::MAX, f32::MIN];
        let plan = plan_popover(input);
        let text_left = plan.card[0] + 12.0;
        assert_eq!(plan.buttons[0].x0, text_left);
        assert_eq!(plan.buttons[0].x1, text_left + 120.0);
    }

    #[test]
    fn spell_anchor_flips_and_clamps_at_canvas_extents() {
        let below = plan_spell_anchor(
            [500.0, 300.0],
            [40.0, 80.0, 30.0, 20.0],
            [180.0, 90.0],
            12.0,
            8.0,
        );
        let above_right = plan_spell_anchor(
            [500.0, 300.0],
            [480.0, 250.0, 18.0, 20.0],
            [180.0, 90.0],
            12.0,
            8.0,
        );
        assert_eq!(below, [40.0, 108.0]);
        assert_eq!(above_right, [308.0, 152.0]);
        assert_eq!(
            plan_spell_anchor(
                [500.0, 100.0],
                [-20.0, 4.0, 18.0, 20.0],
                [180.0, 90.0],
                12.0,
                8.0,
            ),
            [12.0, 12.0]
        );
    }

    #[test]
    fn popover_and_spell_production_paths_route_through_the_planner() {
        let popover = include_str!("../chrome/popover.rs");
        let overlay = include_str!("../chrome/overlay.rs");
        assert_eq!(popover.matches("plan::plan_popover(").count(), 1);
        assert_eq!(overlay.matches("plan::plan_spell_anchor(").count(), 1);
        assert!(
            !popover.contains("let mut card_y = sel_top - gap - card_h"),
            "popover must not regain its retired placement formula"
        );
        assert!(
            !popover.contains("if rx0 <= rx1"),
            "popover measurement must pass raw spans to the planner-owned fallback"
        );
        assert!(
            !overlay.contains("let below_y = word_top + word_h + gap"),
            "spell popup must not regain its retired anchor formula"
        );
    }
}
