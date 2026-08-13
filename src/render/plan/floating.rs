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
    Absolute(f32, f32),
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
        CornerAnchor::Absolute(px, py) => (px, py),
    };
    [left, top, text_w, line_height]
}

/// The resolved short-lived notice geometry. Text and plate are reported
/// together so paint cannot re-derive the padding around the placement the
/// collision planner chose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct ToastPlan {
    pub text: [f32; 4],
    pub plate: [f32; 4],
    pub resolved: crate::theme::ToastAnchor,
    pub fell_back: bool,
}

fn toast_plate_at(
    anchor: crate::theme::ToastAnchor,
    canvas: [f32; 2],
    plate_size: [f32; 2],
    safe: f32,
    top_reserve: f32,
) -> [f32; 4] {
    let [width, height] = canvas;
    let [plate_w, plate_h] = plate_size;
    let top = safe + top_reserve;
    let bottom = (height - safe - plate_h).max(top);
    let left = safe;
    let right = (width - safe - plate_w).max(left);
    let center = ((width - plate_w) * 0.5).clamp(left, right);
    let (x, y) = match anchor {
        crate::theme::ToastAnchor::TopLeft => (left, top),
        crate::theme::ToastAnchor::TopRight => (right, top),
        crate::theme::ToastAnchor::BottomCenter => (center, bottom),
    };
    [x, y, plate_w.min((width - 2.0 * safe).max(0.0)), plate_h]
}

fn rects_clear(a: [f32; 4], b: [f32; 4], gap: f32) -> bool {
    let [ax, ay, aw, ah] = a;
    let [bx, by, bw, bh] = b;
    ax + aw + gap <= bx || bx + bw + gap <= ax || ay + ah + gap <= by || by + bh + gap <= ay
}

/// One geometry owner for a world's authored toast anchor, canvas safety, and
/// active-chrome avoidance.
///
/// A lateral anchor stops being a meaningful authored choice once the toast is
/// wider than either side can remain distinct. That is the narrow fallback:
/// bottom-centre is attempted first. Otherwise the world's choice is attempted
/// first, followed by the other authored slots in a fixed shared order. The
/// final arm is deterministic even when active chrome occupies every candidate
/// slot; ordinary product surfaces are required by the roster law to leave one
/// slot clear.
#[allow(clippy::too_many_arguments)]
pub(in crate::render) fn plan_toast(
    authored: crate::theme::ToastAnchor,
    canvas: [f32; 2],
    text_size: [f32; 2],
    padding: [f32; 2],
    safe: f32,
    collision_gap: f32,
    top_reserve: f32,
    obstacles: &[[f32; 4]],
) -> ToastPlan {
    let plate_size = [
        text_size[0] + 2.0 * padding[0],
        text_size[1] + 2.0 * padding[1],
    ];
    let narrow = canvas[0] < plate_size[0] * 2.0 + safe * 6.0;
    let shared = [
        crate::theme::ToastAnchor::TopLeft,
        crate::theme::ToastAnchor::TopRight,
        crate::theme::ToastAnchor::BottomCenter,
    ];
    let mut order = [authored; 4];
    let mut n = 0usize;
    let first = if narrow {
        crate::theme::ToastAnchor::BottomCenter
    } else {
        authored
    };
    order[n] = first;
    n += 1;
    if authored != first {
        order[n] = authored;
        n += 1;
    }
    for candidate in shared {
        if !order[..n].contains(&candidate) {
            order[n] = candidate;
            n += 1;
        }
    }

    let mut chosen = toast_plate_at(order[0], canvas, plate_size, safe, top_reserve);
    let mut resolved = order[0];
    for candidate in order.into_iter().take(n) {
        let plate = toast_plate_at(candidate, canvas, plate_size, safe, top_reserve);
        if obstacles
            .iter()
            .copied()
            .all(|obstacle| rects_clear(plate, obstacle, collision_gap))
        {
            chosen = plate;
            resolved = candidate;
            break;
        }
    }
    ToastPlan {
        text: [
            chosen[0] + padding[0],
            chosen[1] + padding[1],
            text_size[0],
            text_size[1],
        ],
        plate: chosen,
        resolved,
        fell_back: resolved != authored,
    }
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
                CornerAnchor::Absolute(17.0, 23.0),
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

    /// The new authored axis is crossed with every world, every anchor, all
    /// three surface families, three logical window classes, and both display
    /// densities. Geometry is the pure seam: world face/palette do not alter the
    /// placement inputs, but naming each world here proves no roster member can
    /// miss the axis when it grows.
    #[test]
    fn toast_geometry_sweeps_world_anchor_surface_window_and_dpi() {
        #[derive(Clone, Copy, Debug)]
        enum Surface {
            Document,
            Picker,
            Workspace,
        }
        let independently_clear = |a: [f32; 4], b: [f32; 4], gap: f32| {
            let [ax, ay, aw, ah] = a;
            let [bx, by, bw, bh] = b;
            ax + aw + gap <= bx || bx + bw + gap <= ax || ay + ah + gap <= by || by + bh + gap <= ay
        };
        let logical_canvases = [(480.0, 360.0), (1200.0, 800.0), (1800.0, 1000.0)];
        let mut cells = 0usize;
        let mut fallback_cells = 0usize;
        for world in crate::theme::THEMES {
            for anchor in crate::theme::ToastAnchor::ALL {
                for surface in [Surface::Document, Surface::Picker, Surface::Workspace] {
                    for (logical_w, logical_h) in logical_canvases {
                        for dpi in [1.0, 2.0] {
                            let canvas = [logical_w * dpi, logical_h * dpi];
                            let safe = 8.0 * dpi;
                            let obstacles = match surface {
                                // A visible document outline owns the upper-left
                                // interactive margin; another slot stays clear.
                                Surface::Document => vec![[
                                    8.0 * dpi,
                                    24.0 * dpi,
                                    logical_w * 0.16 * dpi,
                                    90.0 * dpi,
                                ]],
                                // A room-summoned picker owns the upper-middle of
                                // the room; the bottom slot is the shared escape.
                                Surface::Picker => vec![[
                                    logical_w * 0.18 * dpi,
                                    36.0 * dpi,
                                    logical_w * 0.64 * dpi,
                                    logical_h * 0.58 * dpi,
                                ]],
                                // A workspace's occupied header/row band. Its
                                // lower plane remains free for transient status.
                                Surface::Workspace => vec![[
                                    16.0 * dpi,
                                    16.0 * dpi,
                                    (logical_w - 32.0) * dpi,
                                    logical_h * 0.68 * dpi,
                                ]],
                            };
                            let plan = plan_toast(
                                anchor,
                                canvas,
                                [132.0 * dpi, 18.0 * dpi],
                                [11.0 * dpi, 4.0 * dpi],
                                safe,
                                6.0 * dpi,
                                0.0,
                                &obstacles,
                            );
                            let [x, y, w, h] = plan.plate;
                            let label = format!(
                                "{} / {anchor:?} / {surface:?} / {logical_w}x{logical_h} / {dpi}x",
                                world.name
                            );
                            assert!(
                                x >= safe && y >= safe,
                                "{label}: plate {:?} crossed the safe top/left inset {safe}",
                                plan.plate
                            );
                            assert!(
                                x + w <= canvas[0] - safe + 0.01
                                    && y + h <= canvas[1] - safe + 0.01,
                                "{label}: plate {:?} left canvas {:?}",
                                plan.plate,
                                canvas
                            );
                            assert!(
                                obstacles.iter().all(|&obstacle| independently_clear(
                                    plan.plate,
                                    obstacle,
                                    6.0 * dpi
                                )),
                                "{label}: plate {:?} collided with active chrome {:?}",
                                plan.plate,
                                obstacles
                            );
                            assert!(w > 0.0 && h > 0.0, "{label}: toast presence is vacuous");
                            fallback_cells += usize::from(plan.fell_back);
                            cells += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(cells, 20 * 3 * 3 * 3 * 2);
        assert!(
            fallback_cells > 0,
            "NON-VACUITY: no active surface forced a shared fallback"
        );
        eprintln!("toast geometry roster: cells={cells} fallbacks={fallback_cells}");
    }

    #[test]
    fn a_narrow_toast_uses_the_shared_bottom_centre_fallback() {
        let plan = plan_toast(
            crate::theme::ToastAnchor::TopLeft,
            [260.0, 180.0],
            [112.0, 18.0],
            [10.0, 4.0],
            8.0,
            6.0,
            0.0,
            &[],
        );
        assert_eq!(plan.resolved, crate::theme::ToastAnchor::BottomCenter);
        assert!(plan.fell_back);
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

    #[test]
    fn floating_chrome_production_paths_route_through_the_planner() {
        let hud = include_str!("../chrome/hud.rs");
        let whichkey = include_str!("../chrome/whichkey.rs");
        assert_eq!(hud.matches("plan::plan_float_card(").count(), 1);
        assert_eq!(hud.matches("plan::plan_streaks_card(").count(), 1);
        assert_eq!(whichkey.matches("plan::plan_whichkey_card(").count(), 1);
        let notice = include_str!("../chrome/readout/toast.rs");
        assert_eq!(notice.matches("plan::plan_toast(").count(), 1);
        let placement = notice
            .split("pub(super) fn notice_toast_plan(")
            .nth(1)
            .expect("notice_toast_plan exists");
        for world in crate::theme::world_names() {
            assert!(
                !placement.contains(world),
                "toast placement must not branch on world {world}"
            );
        }
        assert!(
            !hud.contains("let card_w = block_w + pad_x * 2.0"),
            "HUD card placement must not regain its retired parallel formula"
        );
        assert!(
            !hud.contains("let content_h = grid_h + gap_dots + dot + gap_between + text_h"),
            "streaks must not regain its retired parallel formula"
        );
        assert!(
            !whichkey.contains("let card_y = (height as f32 - margin - card_h).max(margin)"),
            "which-key must not regain its retired parallel formula"
        );
    }
}
