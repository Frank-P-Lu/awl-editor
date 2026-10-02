//! Shipping Japanese emphasis: real shaped cells, cache transitions and pixels.

use super::super::*;
use super::{headless_dqp, pixeldiff, view_md};

const MIXED: &str = "*「日本語」、。 quiet* **日本語** ***かな***\nplain";

#[test]
fn dots_follow_emphasis_roles_scripts_and_caret_reveal() {
    let _g = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _world = theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(1000.0, 700.0) else {
        eprintln!("skipping Japanese emphasis role law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    for cursor in [1, 0, 1] {
        p.set_view(&view_md(MIXED, cursor, 0));
        p.prepare(&device, &queue, 1000, 700).unwrap();
        let dots = p.japanese_emphasis_dot_rects();
        assert_eq!(
            dots.len(),
            5,
            "three italic kanji and two bold-italic kana, cursor={cursor}"
        );
        // Content styling survives caret reveal; only the markup advances change.
        let row = p.visual_rows(0).remove(0);
        // Locate from source, so the raw/hidden markers cannot change enrollment.
        let chars: Vec<_> = MIXED.lines().next().unwrap().chars().collect();
        let cols: Vec<_> = chars
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c, '日' | '本' | '語' | 'か' | 'な'))
            .map(|(col, _)| col)
            .collect();
        let wanted = [cols[0], cols[1], cols[2], cols[6], cols[7]];
        for (dot, col) in dots.iter().zip(wanted) {
            let center = p.text_left() + (row.xs[col] + row.xs[col + 1]) * 0.5;
            assert!(
                (dot[0] + dot[2] * 0.5 - center).abs() < 0.01,
                "cursor={cursor} col={col}"
            );
        }
    }
    // Switching buffers must not retain the prior geometry, even at matching versions.
    for (text, count) in [
        ("*quiet*\nplain", 0),
        ("*かな*\nplain", 2),
        ("**かな**\nplain", 0),
    ] {
        p.set_view(&view_md(text, 1, 0));
        p.prepare(&device, &queue, 1000, 700).unwrap();
        assert_eq!(p.japanese_emphasis_dot_rects().len(), count, "{text:?}");
    }
    let nested =
        "# *かな*\n> *日本語*\n*[かな](https://example.com/日本語)*\n*かな `日本語`*\nplain";
    p.set_view(&view_md(nested, 4, 0));
    p.prepare(&device, &queue, 1000, 700).unwrap();
    assert_eq!(
        p.japanese_emphasis_dot_rects().len(),
        9,
        "heading/quote/link text retain emphasis; URL and code do not receive dots"
    );
    let mut v = view_md("---\nlang: zh-Hans\n---\n*日本語*\nplain", 4, 0);
    p.set_view(&v);
    p.prepare(&device, &queue, 1000, 700).unwrap();
    assert!(
        p.japanese_emphasis_dot_rects().is_empty(),
        "Chinese-routed Han must not acquire Japanese dots"
    );
    v.is_markdown = false;
    v.text = MIXED.into();
    p.set_view(&v);
    p.prepare(&device, &queue, 1000, 700).unwrap();
    assert!(
        p.japanese_emphasis_dot_rects().is_empty(),
        "plain text never receives Markdown dots"
    );
}

#[test]
fn dots_are_visible_and_clear_text_across_worlds_dpi_wraps_and_headings() {
    let _g = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _world = theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping Japanese emphasis pixel sweep: no wgpu adapter");
        return;
    }
    crate::page::set_page_on(false);
    crate::markdown::set_wysiwyg_on(true);
    let text = "# *日本語かな*\n\n*日本語かな日本語かな日本語かな日本語かな日本語かな日本語かな日本語かな*\nplain";
    let mut checked = 0;
    for world in theme::THEMES {
        theme::set_active_by_name(world.name).unwrap();
        for dpi in [1.0, 2.0] {
            let width = (420.0 * dpi) as u32;
            let height = (800.0 * dpi) as u32;
            let Some((device, queue, mut p)) = headless_dqp(width as f32, height as f32) else {
                panic!("{} dpi={dpi}: adapter lost during sweep", world.name);
            };
            p.set_dpi(dpi);
            p.set_size(width as f32, height as f32);
            p.set_view(&view_md(text, 3, 0));
            p.prepare(&device, &queue, width, height).unwrap();
            let dots = p.japanese_emphasis_dot_rects();
            assert_eq!(
                dots.len(),
                40,
                "{} dpi={dpi}: heading 5 + body 35",
                world.name
            );
            assert!(
                p.visual_rows(2).len() > 1,
                "fixture must wrap on {} dpi={dpi}",
                world.name
            );
            let with = pixeldiff::render_frame(&mut p, &device, &queue, width, height);
            p.japanese_emphasis_dot_pipeline
                .prepare(&device, &queue, width, height, &[]);
            let without = pixeldiff::render_frame(&mut p, &device, &queue, width, height);
            for dot in dots {
                assert!(
                    (dot[2] / dpi - 2.4).abs() < 0.01,
                    "{}: fixed logical dot size",
                    world.name
                );
                let region =
                    pixeldiff::Region::new(dot[0] - 1.0, dot[1] - 1.0, dot[2] + 3.0, dot[3] + 3.0);
                let report =
                    pixeldiff::diff_region(&with, &without, width as i64, height as i64, region);
                assert!(
                    report.differing >= (2.0 * dpi * dpi) as usize
                        && report.max_channel_delta >= 12,
                    "{} dpi={dpi}: missing/indistinct dot {dot:?}: {report:?}",
                    world.name
                );
                // Measure real raster ink, rather than assuming line metrics contain it.
                let origin = (p.text_left(), p.doc_top());
                for run in p.buffer.layout_runs() {
                    for glyph in run.glyphs {
                        let physical = glyph.physical(origin, 1.0);
                        let Some(image) = p
                            .swash_cache
                            .get_image(&mut p.font_system, physical.cache_key)
                            .as_ref()
                        else {
                            continue;
                        };
                        let left = physical.x as f32 + image.placement.left as f32;
                        let right = left + image.placement.width as f32;
                        let top = run.line_y + physical.y as f32 - image.placement.top as f32;
                        let bottom = top + image.placement.height as f32;
                        assert!(
                            dot[0] + dot[2] <= left
                                || dot[0] >= right
                                || dot[1] + dot[3] <= top
                                || dot[1] >= bottom,
                            concat!(
                                "{} dpi={}: dot {:?} collides with raster ink ",
                                "[{}, {}, {}, {}]"
                            ),
                            world.name,
                            dpi,
                            dot,
                            left,
                            top,
                            right,
                            bottom
                        );
                    }
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, theme::THEMES.len() * 2);
}

#[test]
fn dots_track_zoom_and_resize_without_changing_source() {
    let _g = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _world = theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(700.0, 700.0) else {
        eprintln!("skipping Japanese emphasis zoom/resize law: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(false);
    crate::markdown::set_wysiwyg_on(true);
    let text = "# *日本語かな*\n\n*日本語かな日本語かな日本語かな日本語かな*\nplain";
    for zoom in [0.8, 1.0, 1.5] {
        for width in [360, 700] {
            let mut v = view_md(text, 3, 0);
            v.zoom = zoom;
            p.set_size(width as f32, 700.0);
            p.set_view(&v);
            p.prepare(&device, &queue, width, 700).unwrap();
            let dots = p.japanese_emphasis_dot_rects();
            assert_eq!(dots.len(), 25, "zoom={zoom} width={width}");
            for dot in dots {
                assert!(
                    (dot[2] / zoom - 2.4).abs() < 0.01,
                    "zoom={zoom} width={width}: {dot:?}"
                );
            }
            assert_eq!(
                p.buffer
                    .lines
                    .iter()
                    .map(|line| line.text())
                    .collect::<Vec<_>>()
                    .join("\n"),
                text
            );
        }
    }
}
