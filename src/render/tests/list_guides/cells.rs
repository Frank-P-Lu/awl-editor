//! Mixed marker cells use one measured grid and ink-aware stem clearance.
use super::*;
use crate::render::tests::{pixeldiff, task_markers};

#[test]
fn guide_endpoints_clear_mixed_empty_markers_without_changing_row_pitch() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = super::super::headless_dqp(900.0, 700.0) else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            let (width, height) = ((900.0 * dpi) as u32, (700.0 * dpi) as u32);
            p.set_dpi(dpi);
            p.set_size(width as f32, height as f32);
            for (parent, empty) in [("-", "- [ ]"), ("- [ ]", "- [x]"), ("- [x]", "-")] {
                let text = format!(
                    "- Root\n  {parent} Parent\n    - Child\n  {empty} \n  - Next\n\nOutside\n"
                );
                p.set_view(&view_md(&text, 6, 0));
                let marks = p.list_marks();
                let (masks, _) = task_markers::isolated_marker_masks(
                    &mut p, &device, &queue, width, height, &marks,
                );
                let rects = rails(&mut p);
                let left = masks[1].iter().map(|(x, _)| *x).min().unwrap();
                let right = masks[1].iter().map(|(x, _)| *x).max().unwrap();
                let center = marks[1].left.floor() + (left + right + 1) as f32 * 0.5;
                let child_top = p.doc_top() + p.visual_rows(2)[0].line_top;
                let stem = rects
                    .iter()
                    .find(|r| {
                        (r[0] + r[2] * 0.5 - center).abs() <= 1.01
                            && r[1] <= child_top
                            && r[1] + r[3] > child_top
                    })
                    .unwrap();
                let ink_top =
                    marks[3].top.floor() + masks[3].iter().map(|(_, y)| *y).min().unwrap() as f32;
                let gap = ink_top - (stem[1] + stem[3]);
                let expected = p.metrics.font_size * 0.125;
                assert!(
                    (gap - expected).abs() <= 1.51,
                    "{} dpi {dpi} {parent}/{empty}: empty marker gap {gap}, expected {expected}",
                    world.name
                );
                let parent_bottom = marks[1].top.floor()
                    + masks[1].iter().map(|(_, y)| *y).max().unwrap() as f32
                    + 1.0;
                assert!((stem[1] - parent_bottom - expected).abs() <= 1.51);
                let tops = (1..=4)
                    .map(|line| p.visual_rows(line)[0].line_top)
                    .collect::<Vec<_>>();
                for pair in tops.windows(2) {
                    assert!((pair[1] - pair[0] - p.metrics.line_height).abs() < 0.1);
                }
                // Revealing the empty source prefix must keep the stem out of it.
                p.set_view(&view_md(&text, 3, 2));
                let raw_top = p.doc_top() + p.visual_rows(3)[0].line_top;
                let rects = rails(&mut p);
                assert!(
                    rects
                        .iter()
                        .filter(|r| (r[0] + r[2] * 0.5 - center).abs() <= 1.01)
                        .all(|r| r[1] + r[3] <= raw_top + 0.1)
                );
            }
        }
    }
}

#[test]
fn ordered_parent_guides_use_rendered_number_ink_in_every_world() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = super::super::headless_dqp(900.0, 700.0) else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            let (width, height) = ((900.0 * dpi) as u32, (700.0 * dpi) as u32);
            p.set_dpi(dpi);
            p.set_size(width as f32, height as f32);
            for number in ["1.", "10.", "9)"] {
                let text = format!("{number} Parent\n    - Detail\n\nOutside\n");
                p.set_view(&view_md(&text, 3, 0));
                p.prepare(&device, &queue, width, height).unwrap();
                let with = pixeldiff::render_frame(&mut p, &device, &queue, width, height);
                p.renderer
                    .prepare(
                        &device,
                        &queue,
                        &mut p.font_system,
                        &mut p.atlas,
                        &p.viewport,
                        std::iter::empty(),
                        &mut p.swash_cache,
                    )
                    .unwrap();
                let without = pixeldiff::render_frame(&mut p, &device, &queue, width, height);
                let row = p.visual_rows(0)[0].clone();
                let x0 = p.text_left().floor() as u32;
                let x1 = (p.text_left() + row.xs[number.len()]).ceil() as u32;
                let y0 = (p.doc_top() + row.line_top).floor() as u32;
                let y1 = (p.doc_top() + row.line_top + row.line_height).ceil() as u32;
                let mut xs = Vec::new();
                for y in y0..y1 {
                    for x in x0..x1 {
                        let index = (y * width + x) as usize;
                        if with[index] != without[index] {
                            xs.push(x);
                        }
                    }
                }
                let center =
                    (*xs.iter().min().unwrap() + *xs.iter().max().unwrap() + 1) as f32 * 0.5;
                let rects = rails(&mut p);
                assert!(!rects.is_empty());
                for r in rects {
                    assert!(
                        (r[0] + r[2] * 0.5 - center).abs() <= 1.01,
                        "{} dpi {dpi} {number}: rail {}, actual number centre {center}",
                        world.name,
                        r[0] + r[2] * 0.5
                    );
                }
            }
        }
    }
}

#[test]
fn guide_bridges_require_a_clear_gutter_through_parent_wraps() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    let body =
        "A parent paragraph with enough prose to wrap through several visual rows. ".repeat(4);
    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            p.set_dpi(dpi);
            p.set_size(660.0 * dpi, 1800.0 * dpi);
            for (prefix, indent, hanging) in [("1.", 3, false), ("10.", 4, false), ("-", 2, true)] {
                let text = format!(
                    "{prefix} {body}\n{}- Child\n\nOutside\n",
                    " ".repeat(indent)
                );
                p.set_view(&view_md(&text, 3, 0));
                let rows = p.visual_rows(0);
                assert!(rows.len() > 1, "{} dpi {dpi} {prefix}", world.name);
                let last = rows.last().unwrap();
                let parent_bottom = p.doc_top() + last.line_top + last.line_height;
                let rects = rails(&mut p);
                let first_top = rects.iter().map(|r| r[1]).reduce(f32::min).unwrap();
                if hanging {
                    assert!(
                        first_top < parent_bottom - p.metrics.line_height,
                        "{} dpi {dpi}: concealed rich parent keeps its clear hanging gutter",
                        world.name
                    );
                } else {
                    assert!(
                        first_top >= parent_bottom - 0.1,
                        "{} dpi {dpi} {prefix}: guide starts at {first_top}, crossing raw wrapped prose \
                         above {parent_bottom}",
                        world.name
                    );
                }
            }
        }
    }
}
