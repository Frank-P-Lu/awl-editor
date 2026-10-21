//! Outcome laws for structural rails, source reveal and measured nesting space.
use super::super::*;
use super::{headless_pipeline, view_md};
mod cells;

const DOC: &str = concat!(
    "- Parent\n",
    "  - Child with enough words to wrap repeatedly across a narrow writing column ",
    "and preserve the rails through its visual rows\n",
    "    - [ ] Task\n    - [ ] \n  1. Ordered\n     - Detail\n\nOutside\n",
);

fn rails(p: &mut TextPipeline) -> Vec<[f32; 4]> {
    p.row_geom.seal_frame(&p.buffer, &p.metrics);
    p.list_guide_geometry().0
}

#[test]
fn list_guides_persist_on_active_and_selected_source_in_every_world() {
    let _serial = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    let entry_world = theme::active().name;
    for dpi in [1.0, 2.0] {
        p.set_dpi(dpi);
        p.set_size(620.0 * dpi, 1500.0 * dpi);
        for world in theme::THEMES.iter() {
            theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            for selection in [None, Some(((2, 0), (3, 10))), Some(((3, 10), (2, 0)))] {
                let mut v = view_md(DOC, 2, 10);
                v.selection = selection;
                p.set_view(&v);
                let rects = rails(&mut p);
                let top = p.doc_top() + p.visual_rows(2)[0].line_top;
                let task_rails = rects
                    .iter()
                    .filter(|r| r[1] <= top + 0.1 && r[1] + r[3] > top + 0.1)
                    .collect::<Vec<_>>();
                assert_eq!(
                    task_rails.len(),
                    2,
                    "{} dpi {dpi} {selection:?}",
                    world.name
                );
                assert!(task_rails[1][0] > task_rails[0][0] + dpi);
                assert_eq!(p.buffer.lines[2].text(), "    - [ ] Task");
                assert_eq!(p.buffer.lines[2].hanging_inset(), 0.0);
                // The raw task prefix remains editable and has real advance.
                let row = &p.visual_rows(2)[0];
                assert!(row.xs[10] > row.xs[4]);
                let x = p.col_x_and_advance(2, 10).0;
                assert!((x - row.xs[10]).abs() < 0.51);
                let hit = p.hit_test_scroll(
                    p.text_left() + x + 0.1,
                    top + row.line_height * 0.5,
                    ScrollPos::default(),
                );
                assert_eq!(hit.0, 2);
                assert!((10..=11).contains(&hit.1), "{hit:?}");
            }
        }
    }
    theme::set_active_by_name(entry_world).unwrap();
}

#[test]
fn list_guides_cover_wrapped_preview_and_stop_before_unrelated_text() {
    let _serial = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    p.set_size(520.0, 1500.0);
    p.set_view(&view_md(DOC, 7, 0));
    let rects = rails(&mut p);
    let rows = p.visual_rows(1);
    assert!(rows.len() > 1);
    for row in rows {
        let top = p.doc_top() + row.line_top;
        assert!(
            rects
                .iter()
                .any(|r| r[1] <= top + 0.1 && r[1] + r[3] >= top + row.line_height - 0.1)
        );
    }
    let outside_top = p.doc_top() + p.visual_rows(7)[0].line_top;
    assert!(rects.iter().all(|r| r[1] + r[3] <= outside_top));
    p.set_view(&view_md("- Parent\n  - Child\nlazy prose\n", 2, 4));
    let rects = rails(&mut p);
    let lazy_top = p.doc_top() + p.visual_rows(2)[0].line_top;
    assert!(rects.iter().all(|r| r[1] < lazy_top));
    let mut plain = view_md(DOC, 7, 0);
    plain.is_markdown = false;
    p.set_view(&plain);
    assert!(rails(&mut p).is_empty());
    assert!(
        p.list_layout.guides.is_empty(),
        "buffer swaps clear structural ownership"
    );
}

#[test]
fn list_guide_work_is_bounded_by_visible_rows_and_clipped_to_viewport() {
    let _serial = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    p.set_size(620.0, 600.0);
    let text = "- Parent\n  - Child\n".repeat(2000);
    p.set_view(&view_md(&text, 0, 0));
    p.row_geom.seal_frame(&p.buffer, &p.metrics);
    let (rects, visited) = p.list_guide_geometry();
    assert!(
        visited > 0 && visited < 100,
        "visited {visited} of 4000 lines"
    );
    assert!(!rects.is_empty());
    assert!(rects.iter().all(|r| r[1] >= 0.0 && r[1] + r[3] <= 600.0));
    crate::markdown::set_wysiwyg_on(false);
    assert!(rails(&mut p).is_empty());
    crate::markdown::set_wysiwyg_on(true);
}

#[test]
fn ordered_guide_rails_do_not_jog_on_authored_continuations() {
    let _serial = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    p.set_size(900.0, 1000.0);
    let text = "- Parent\n  1. Ordered\n     continuation\n\nOutside\n";
    for cursor in [1, 2, 4] {
        p.set_view(&view_md(text, cursor, 0));
        let rects = rails(&mut p);
        let at = |line| {
            let top = p.doc_top() + p.visual_rows(line)[0].line_top;
            rects
                .iter()
                .find(|r| r[1] <= top + 0.1 && r[1] + r[3] > top + 0.1)
                .unwrap()[0]
        };
        assert_eq!(at(1), at(2));
    }
}

#[test]
fn retained_ordered_indent_matches_fresh_layout_through_fence_edits() {
    let _serial = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    let before = "1. Parent\n   1. Ordered\n\nOutside\n";
    let fenced = "```\n1. Parent\n   1. Ordered\n\nOutside\n";
    p.set_view(&view_md(before, 3, 0));
    let normal = p.visual_rows(1)[0].xs.clone();
    p.set_view(&view_md(fenced, 4, 0));
    let fenced_xs = p.visual_rows(2)[0].xs.clone();
    assert!(rails(&mut p).is_empty());
    p.set_view(&view_md(before, 3, 0));
    assert_eq!(p.visual_rows(1)[0].xs, normal);
    // A fresh buffer identity exercises the full-shape oracle.
    let Some(mut fresh) = headless_pipeline() else {
        return;
    };
    fresh.set_view(&view_md(fenced, 4, 0));
    assert_eq!(fresh.visual_rows(2)[0].xs, fenced_xs);
}

#[test]
fn guides_align_with_parent_bullet_and_checkbox_ink_in_every_world() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = super::headless_dqp(900.0, 700.0) else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    let text = concat!(
        "- Parent\n  - [ ] Open parent\n    - [x] Checked parent\n",
        "      - Child parent\n        - Leaf\n\nOutside\n",
    );
    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            let width = (900.0 * dpi) as u32;
            let height = (700.0 * dpi) as u32;
            p.set_dpi(dpi);
            p.set_size(width as f32, height as f32);
            p.set_view(&view_md(text, 6, 0));
            let marks = p.list_marks();
            assert_eq!(marks.len(), 5);
            let (masks, _) = super::task_markers::isolated_marker_masks(
                &mut p, &device, &queue, width, height, &marks,
            );
            let rects = rails(&mut p);
            let mut centers = Vec::new();
            for parent in 0..4 {
                let mask = &masks[parent];
                assert!(!mask.is_empty());
                let left = mask.iter().map(|(x, _)| *x).min().unwrap();
                let right = mask.iter().map(|(x, _)| *x).max().unwrap();
                let center = marks[parent].left.floor() + (left + right + 1) as f32 * 0.5;
                centers.push(center);
                let child_top = p.doc_top() + p.visual_rows(parent + 1)[0].line_top;
                let row = rects
                    .iter()
                    .filter(|r| r[1] <= child_top + 0.1 && r[1] + r[3] > child_top + 0.1)
                    .collect::<Vec<_>>();
                assert_eq!(row.len(), parent + 1, "{} dpi {dpi}", world.name);
                let rail_center = row[parent][0] + row[parent][2] * 0.5;
                assert!(
                    (rail_center - center).abs() <= 1.01,
                    "{} dpi {dpi} parent {parent}: rail {rail_center}, marker {center}",
                    world.name
                );
            }
            let step = p
                .list_layout
                .metrics
                .indent_advance(2.0, world.list_indent_scale);
            for pair in centers.windows(2) {
                assert!(
                    (pair[1] - pair[0] - step).abs() <= 1.51,
                    "{} dpi {dpi}: marker centres {centers:?}, expected step {step}",
                    world.name
                );
            }
        }
    }
}
