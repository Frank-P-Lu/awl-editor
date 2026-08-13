//! Device-free row geometry for persistent margin chrome.

#[derive(Clone, Debug, PartialEq)]
pub(in crate::render) struct GutterStackPlan {
    pub top: f32,
    pub carve: [f32; 4],
    pub rows: Vec<[f32; 4]>,
}

pub(in crate::render) fn plan_gutter_stack(
    canvas_h: f32,
    avail: f32,
    row_h: f32,
    row_count: usize,
    bottom_inset: f32,
    carve_breath_rows: f32,
) -> GutterStackPlan {
    let block_h = row_h * row_count as f32;
    let top = canvas_h - block_h - bottom_inset;
    let rows = (0..row_count)
        .map(|row| [0.0, top + row as f32 * row_h, avail, row_h])
        .collect();
    GutterStackPlan {
        top,
        carve: [
            0.0,
            (top - row_h * carve_breath_rows).max(0.0),
            avail,
            canvas_h,
        ],
        rows,
    }
}

impl GutterStackPlan {
    pub(in crate::render) fn hit_row(&self, px: f32, py: f32) -> Option<usize> {
        self.rows.iter().position(|rect| {
            px >= rect[0] && px <= rect[0] + rect[2] && py >= rect[1] && py < rect[1] + rect[3]
        })
    }
}

pub(in crate::render) fn plan_outline_left(right_edge: f32, block_w: f32, min_left: f32) -> f32 {
    (right_edge - block_w).max(min_left)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct OutlineSlot {
    pub line: usize,
    pub y: f32,
}

pub(in crate::render) fn plan_outline_slots(
    top: f32,
    row_h: f32,
    gap_rows: f32,
    rows: impl IntoIterator<Item = (usize, bool)>,
) -> Vec<OutlineSlot> {
    let mut y = top;
    rows.into_iter()
        .map(|(line, gap_before)| {
            if gap_before {
                y += row_h * gap_rows;
            }
            let slot = OutlineSlot { line, y };
            y += row_h;
            slot
        })
        .collect()
}

pub(in crate::render) fn hit_outline_slot(
    slots: &[OutlineSlot],
    px: f32,
    py: f32,
    x_band: [f32; 2],
    row_h: f32,
) -> Option<usize> {
    if px < x_band[0] || px > x_band[1] || row_h <= 0.0 {
        return None;
    }
    slots
        .iter()
        .find(|slot| py >= slot.y && py < slot.y + row_h)
        .map(|slot| slot.line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gutter_stack_keeps_draw_carve_and_hit_on_one_bottom_anchor() {
        let two = plan_gutter_stack(300.0, 90.0, 12.0, 2, 8.0, 0.5);
        let three = plan_gutter_stack(300.0, 90.0, 12.0, 3, 8.0, 0.5);
        assert_eq!(two.top, 268.0);
        assert_eq!(three.top, 256.0);
        assert_eq!(three.carve, [0.0, 250.0, 90.0, 300.0]);
        for (row, rect) in three.rows.iter().enumerate() {
            assert_eq!(three.hit_row(45.0, rect[1] + 6.0), Some(row));
        }
        assert_eq!(three.hit_row(91.0, 270.0), None);
    }

    #[test]
    fn outline_slots_preserve_group_gaps_and_hit_only_row_bands() {
        let slots = plan_outline_slots(20.0, 12.0, 0.5, [(4, false), (9, true), (15, false)]);
        assert_eq!(slots[0], OutlineSlot { line: 4, y: 20.0 });
        assert_eq!(slots[1], OutlineSlot { line: 9, y: 38.0 });
        assert_eq!(slots[2], OutlineSlot { line: 15, y: 50.0 });
        assert_eq!(
            hit_outline_slot(&slots, 30.0, 39.0, [16.0, 80.0], 12.0),
            Some(9)
        );
        assert_eq!(
            hit_outline_slot(&slots, 30.0, 35.0, [16.0, 80.0], 12.0),
            None
        );
        assert_eq!(plan_outline_left(300.0, 120.0, 16.0), 180.0);
        assert_eq!(plan_outline_left(300.0, 400.0, 16.0), 16.0);
    }

    #[test]
    fn margin_chrome_production_paths_route_through_the_planner() {
        let gutter = include_str!("../chrome/gutter.rs");
        let outline = include_str!("../chrome/outline.rs");
        assert_eq!(gutter.matches("plan::plan_gutter_stack(").count(), 4);
        assert_eq!(outline.matches("plan::plan_outline_slots(").count(), 3);
        for (owner, next_owner) in [
            ("fn outline_keepout_rect(", "fn lava_frost_pill_rects("),
            ("fn outline_ink_bands(", "fn outline_frost_seeds("),
            ("fn outline_hit_line(", "fn prepare_outline("),
        ] {
            let body = outline
                .split_once(owner)
                .unwrap_or_else(|| panic!("missing exact outline geometry owner {owner}"))
                .1
                .split_once(next_owner)
                .unwrap_or_else(|| panic!("missing boundary {next_owner} after {owner}"))
                .0;
            assert_eq!(
                body.matches("plan::plan_outline_slots(").count(),
                1,
                "{owner} must route exactly once through plan_outline_slots"
            );
        }
        assert_eq!(outline.matches("plan::plan_outline_left(").count(), 1);
        assert!(!gutter.contains("let block_top = height as f32 - row_h * lines"));
        assert!(!outline.contains("let mut y = layout.top"));
        assert!(!outline.contains("(right_edge - block_w).max(min_left)"));
    }
}
