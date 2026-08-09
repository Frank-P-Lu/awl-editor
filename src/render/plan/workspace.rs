//! Pure scalar geometry for workspace regions and their attached preview card.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct WorkspaceRegions {
    pub card: [f32; 4],
    pub primary: [f32; 2],
    pub pane: [f32; 2],
    pub wide: bool,
    pub content_focused: bool,
}

impl WorkspaceRegions {
    pub(in crate::render) fn primary_visible(&self) -> bool {
        self.wide || !self.content_focused
    }

    pub(in crate::render) fn content_visible(&self) -> bool {
        self.wide || self.content_focused
    }
}

pub(in crate::render) struct WorkspaceRegionsInput {
    pub canvas_w: f32,
    pub canvas_h: f32,
    pub margin: f32,
    pub top_reserve: f32,
    pub min_height: f32,
    pub hpad: f32,
    pub primary_w: f32,
    pub gap: f32,
    pub wide: bool,
    pub content_focused: bool,
}

pub(in crate::render) fn plan_workspace_regions(input: WorkspaceRegionsInput) -> WorkspaceRegions {
    let card_x = input.margin;
    let card_w = (input.canvas_w - 2.0 * input.margin).max(0.0);
    let card_y = input.margin + input.top_reserve;
    let card_h = (input.canvas_h - card_y - input.margin).max(input.min_height);
    let interior = (card_w - 2.0 * input.hpad).max(0.0);
    let (primary, pane) = if input.wide {
        (
            [card_x + input.hpad, input.primary_w],
            [
                card_x + input.hpad + input.primary_w + input.gap,
                (card_w - 2.0 * input.hpad - input.primary_w - input.gap).max(0.0),
            ],
        )
    } else {
        (
            [card_x + input.hpad, interior],
            [card_x + input.hpad, interior],
        )
    };
    WorkspaceRegions {
        card: [card_x, card_y, card_w, card_h],
        primary,
        pane,
        wide: input.wide,
        content_focused: input.content_focused,
    }
}

pub(in crate::render) fn plan_comparison_viewport(
    regions: WorkspaceRegions,
    eligible: bool,
    header_band: f32,
    pad: f32,
) -> Option<[f32; 4]> {
    if !eligible || !regions.content_visible() {
        return None;
    }
    let top = regions.card[1] + pad + header_band;
    let bottom = regions.card[1] + regions.card[3] - pad;
    let height = bottom - top;
    (regions.pane[1] > 0.0 && height > 0.0).then_some([
        regions.pane[0],
        top,
        regions.pane[1],
        height,
    ])
}

pub(in crate::render) fn plan_caret_preview_panel(
    picker_card: [f32; 4],
    line_height: f32,
    pad: f32,
    gap: f32,
) -> ([f32; 4], f32, f32) {
    let box_h = 2.0 * line_height + 2.0 * pad;
    let y = picker_card[1] + picker_card[3] + gap;
    let rect = [picker_card[0], y, picker_card[2], box_h];
    (rect, picker_card[0] + pad, y + box_h * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regions(wide: bool, focused: bool) -> WorkspaceRegions {
        plan_workspace_regions(WorkspaceRegionsInput {
            canvas_w: if wide { 1200.0 } else { 500.0 },
            canvas_h: 800.0,
            margin: 20.0,
            top_reserve: 12.0,
            min_height: 30.0,
            hpad: 16.0,
            primary_w: 180.0,
            gap: 24.0,
            wide,
            content_focused: focused,
        })
    }

    #[test]
    fn workspace_regions_switch_between_split_and_staged_extents() {
        let wide = regions(true, false);
        assert_eq!(wide.card, [20.0, 32.0, 1160.0, 748.0]);
        assert_eq!(wide.primary, [36.0, 180.0]);
        assert_eq!(wide.pane, [240.0, 924.0]);
        assert!(wide.primary_visible() && wide.content_visible());
        let primary = regions(false, false);
        let content = regions(false, true);
        assert_eq!(primary.primary, primary.pane);
        assert!(primary.primary_visible() && !primary.content_visible());
        assert!(!content.primary_visible() && content.content_visible());
    }

    #[test]
    fn comparison_and_preview_derive_from_their_parent_extents() {
        let r = regions(true, false);
        assert_eq!(
            plan_comparison_viewport(r, true, 70.0, 16.0),
            Some([240.0, 118.0, 924.0, 646.0])
        );
        assert_eq!(plan_comparison_viewport(r, false, 70.0, 16.0), None);
        assert_eq!(
            plan_caret_preview_panel([30.0, 40.0, 300.0, 200.0], 24.0, 12.0, 10.0),
            ([30.0, 250.0, 300.0, 72.0], 42.0, 286.0)
        );
    }

    #[test]
    fn workspace_and_preview_production_paths_route_through_the_planner() {
        let comparison = include_str!("../chrome/comparison.rs");
        let preview = include_str!("../chrome/preview.rs");
        assert_eq!(
            comparison.matches("plan::plan_workspace_regions(").count(),
            1
        );
        assert_eq!(
            comparison
                .matches("plan::plan_comparison_viewport(")
                .count(),
            1
        );
        assert_eq!(
            preview.matches("plan::plan_caret_preview_panel(").count(),
            1
        );
        assert!(!comparison.contains("let card_w = (width as f32 - 2.0 * margin).max(0.0)"));
        assert!(!preview.contains("let box_h = 2.0 * m.line_height + 2.0 * pad"));
        let viewport = comparison
            .split("fn comparison_viewport")
            .nth(1)
            .expect("comparison viewport production body");
        let early_return = viewport
            .find("return None")
            .expect("ordinary-frame early return");
        let regions = viewport
            .find("self.workspace_regions")
            .expect("workspace region planning");
        assert!(
            early_return < regions,
            "ordinary frames must return before the hot path plans workspace regions"
        );
    }
}
