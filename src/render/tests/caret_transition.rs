//! Adaptive endpoints retain insertion anchoring and deliberate stable fallbacks.
use super::super::*;
use super::{headless_pipeline, view};

#[test]
fn previous_character_option_anchors_whole_graphemes_without_crossing_a_row_start() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    let text = "A\u{30a}漢 ";
    crate::caret::set_highlight_previous_character(false);
    p.set_view(&view(text, 0, 2));
    assert_eq!(p.caret_anchor_col(), 2);
    crate::caret::set_highlight_previous_character(true);
    for (cursor, anchor) in [(0, 0), (1, 0), (2, 0), (3, 2), (4, 3)] {
        p.set_view(&view(text, 0, cursor));
        p.settle_caret();
        assert_eq!(p.caret_anchor_col(), anchor, "cursor={cursor}");
        assert_eq!(p.caret_is_bar_form(), cursor == 0);
    }
    let mut drag = view(text, 0, 2);
    drag.selecting_drag = true;
    p.set_view(&drag);
    assert_eq!(p.caret_anchor_col(), 2);
    assert!(p.caret_is_bar_form());
}

#[test]
fn glyphless_ligature_and_empty_endpoints_stay_visible_and_finite() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    for dpi in [1.0, 2.0] {
        p.set_dpi(dpi);
        for world in theme::THEMES {
            theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            for (text, col) in [
                ("", 0),
                (" ", 0),
                ("x", 1),
                ("fi", 0),
                ("fi", 1),
                ("   ", 2),
            ] {
                p.set_view(&view(text, 0, col));
                p.settle_caret();
                let (x, y, w, h, r, ax, ay) = p.caret_geometry();
                assert!(
                    [x, y, w, h, r, ax, ay].iter().all(|v| v.is_finite()),
                    "{} {text:?}",
                    world.name
                );
                assert!(w > 0.0 && h > 0.0 && r >= 0.0);
            }
            let mut sizes = Vec::new();
            for col in 0..3 {
                p.set_view(&view("   ", 0, col));
                p.settle_caret();
                sizes.push(p.caret_geometry().3);
            }
            assert!(
                sizes.windows(2).all(|v| (v[0] - v[1]).abs() < 0.01),
                "spaces have one fallback height"
            );
        }
    }
}

#[test]
fn previous_character_wrap_boundary_stays_on_the_cursor_visual_row() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(true);
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    crate::page::set_page_on(true);
    crate::page::set_measure(20);
    let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda";
    p.set_view(&view(text, 0, 0));
    let rows = p.visual_rows(0);
    assert!(rows.len() > 1, "fixture must wrap");
    let starts: Vec<_> = rows.iter().skip(1).map(|r| r.start_col).collect();
    for col in starts {
        p.set_view(&view(text, 0, col));
        p.settle_caret();
        assert_eq!(p.caret_anchor_col(), col);
        assert!(p.caret_is_bar_form());
    }
}

#[test]
fn reduced_motion_and_live_world_switch_keep_the_adaptive_endpoint() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
    crate::motion::set_reduced(true);
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    for world in ["Paperbark", "Tawny", "Cassowary", "Wagtail"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        p.set_view(&view("Å", 0, 0));
        p.settle_caret();
        let ink = p.caret_anchor_ink_box().unwrap();
        let (_, cy, _, h, ..) = p.caret_geometry();
        let baseline = p.caret_baseline_y();
        assert!(
            cy - h * 0.5 <= baseline - ink.top && cy + h * 0.5 >= baseline - ink.top + ink.height
        );
        assert!(!p.caret.is_animating());
    }
}
