//! Item 127 real-pixel fold-chevron centring laws.

use super::super::*;
use super::{headless_dqp, pixeldiff, view_md};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
enum ChevronSummon {
    Caret,
    Hover,
}

#[derive(Clone, Copy, Debug)]
struct PixelBox {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    count: usize,
}

impl PixelBox {
    fn center_y(self) -> f32 {
        (self.top + self.bottom + 1) as f32 * 0.5
    }
}

#[allow(clippy::too_many_arguments)]
fn changed_pixels_in(
    before: &[[u8; 4]],
    after: &[[u8; 4]],
    width: u32,
    height: u32,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> (PixelBox, Vec<(u32, u32)>) {
    let mut points = Vec::new();
    for y in y0.max(0) as u32..y1.min(height as i32).max(0) as u32 {
        for x in x0.max(0) as u32..x1.min(width as i32).max(0) as u32 {
            let i = (y * width + x) as usize;
            if before[i] != after[i] {
                points.push((x, y));
            }
        }
    }
    assert!(
        !points.is_empty(),
        "the summoned fold chevron must change real pixels in its margin lane"
    );
    let left = points.iter().map(|&(x, _)| x).min().unwrap();
    let top = points.iter().map(|&(_, y)| y).min().unwrap();
    let right = points.iter().map(|&(x, _)| x).max().unwrap();
    let bottom = points.iter().map(|&(_, y)| y).max().unwrap();
    (
        PixelBox {
            left,
            top,
            right,
            bottom,
            count: points.len(),
        },
        points,
    )
}

fn heading_fixture(level: usize) -> String {
    format!(
        "{} wave 7\nbody kept inside the section\n{} sibling\nsibling body\n",
        "#".repeat(level),
        "#".repeat(level)
    )
}

fn fold_view_for(text: &str, collapsed: bool, cursor_on_heading: bool) -> crate::render::ViewState {
    let cursor_line = if cursor_on_heading { 0 } else { 3 };
    let mut v = view_md(text, cursor_line, 0);
    if collapsed {
        let levels = crate::fold::heading_levels(text, true);
        let folds: BTreeSet<usize> = [0].into_iter().collect();
        let hidden = crate::fold::hidden_lines(&levels, &folds);
        let tails = crate::fold::fold_tails(&levels, &folds);
        crate::fold::apply_to_view(&mut v, &hidden, &tails);
    }
    v
}

#[allow(clippy::too_many_arguments)]
fn assert_chevron_pixel_center(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    level: usize,
    collapsed: bool,
    summon: ChevronSummon,
    label: &str,
) -> PixelBox {
    let text = heading_fixture(level);
    let rest = fold_view_for(&text, collapsed, false);
    p.set_view(&rest);
    p.set_hover_line(None);
    p.prepare(device, queue, width, height).unwrap();
    let before = pixeldiff::render_frame(p, device, queue, width, height);

    match summon {
        ChevronSummon::Hover => {
            p.set_hover_line(Some(0));
        }
        ChevronSummon::Caret => {
            let on = fold_view_for(&text, collapsed, true);
            p.set_view(&on);
            p.set_hover_line(None);
        }
    }
    p.prepare(device, queue, width, height).unwrap();
    let geoms = p.fold_chevron_geometries();
    assert_eq!(
        geoms.len(),
        1,
        "{label}: exactly one heading chevron is summoned"
    );
    let geom = geoms[0];
    let after = pixeldiff::render_frame(p, device, queue, width, height);

    // Differential pixels isolate the chevron from every world's page ground:
    // the paired frames differ only by the summon state in the leading pad.
    // Search one full row above/below too, so ink escaping the hit box is found
    // rather than silently cropped out of the oracle.
    let (bbox, changed) = changed_pixels_in(
        &before,
        &after,
        width,
        height,
        p.column_left().floor() as i32,
        (geom.row_top - geom.row_height).floor() as i32,
        p.text_left().ceil() as i32,
        (geom.row_top + geom.row_height * 2.0).ceil() as i32,
    );
    let center_delta = (bbox.center_y() - geom.row_center()).abs();
    assert!(
        center_delta <= 1.5,
        "{label}: fold-chevron real-pixel ink bbox {bbox:?} centres at {:.2}px, \
         shaped heading-row centre is {:.2}px (delta {center_delta:.2}px, ceiling 1.5px)",
        bbox.center_y(),
        geom.row_center(),
    );
    assert!(
        bbox.count >= 3 && bbox.right >= bbox.left && bbox.bottom >= bbox.top,
        "{label}: the chevron has a non-vacuous real-pixel silhouette: {bbox:?}"
    );
    for (x, y) in changed {
        assert_eq!(
            p.fold_chevron_hit(x as f32 + 0.5, y as f32 + 0.5),
            Some(geom.line),
            "{label}: painted chevron pixel ({x},{y}) must sit inside the shared \
             generous hit box and resolve to the same heading"
        );
    }
    bbox
}

/// The appearance law baseline alignment cannot express. Every world's real
/// U+203A mask is differenced against its identical no-hover frame, then its
/// visible-pixel bbox is centred on the actual shaped H1/H2/H3 row. This is the
/// full authored world roster, and the click owner encloses every painted pixel.
#[test]
fn fold_chevron_real_pixel_ink_centres_on_h1_h2_h3_across_every_world() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping item-127 full-world fold-chevron pixel law: no GPU adapter");
        return;
    };
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for level in 1..=3 {
            assert_chevron_pixel_center(
                &mut p,
                &device,
                &queue,
                1200,
                800,
                level,
                false,
                ChevronSummon::Hover,
                &format!("world={} H{level}", world.name),
            );
        }
    }
}

/// The exact reported Wagtail H2 fixture, crossed over the remaining axes:
/// expanded/collapsed, caret/hover summon, narrow/wide canvases, 1x/2x device
/// scale, and the supported zoom range's low/default/high representatives.
#[test]
fn wagtail_wave7_h2_chevron_centres_across_state_viewport_dpi_and_zoom() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping item-127 Wagtail axis-sweep pixel law: no GPU adapter");
        return;
    };
    crate::theme::set_active_by_name("Wagtail").unwrap();
    p.sync_theme();

    for (logical_w, logical_h) in [(720u32, 640u32), (1200, 800)] {
        for dpi in [1.0f32, 2.0] {
            let width = (logical_w as f32 * dpi) as u32;
            let height = (logical_h as f32 * dpi) as u32;
            p.set_size(width as f32, height as f32);
            p.set_dpi(dpi);
            for zoom in [0.5f32, 1.0, 3.0] {
                for collapsed in [false, true] {
                    for summon in [ChevronSummon::Caret, ChevronSummon::Hover] {
                        let label = format!(
                            "Wagtail H2 {} {summon:?} {logical_w}x{logical_h}@{dpi} zoom={zoom}",
                            if collapsed { "collapsed" } else { "expanded" }
                        );
                        let text = heading_fixture(2);
                        let mut rest = fold_view_for(&text, collapsed, false);
                        rest.zoom = zoom;
                        p.set_view(&rest);
                        p.set_hover_line(None);
                        p.prepare(&device, &queue, width, height).unwrap();
                        let before =
                            pixeldiff::render_frame(&mut p, &device, &queue, width, height);

                        match summon {
                            ChevronSummon::Hover => {
                                p.set_hover_line(Some(0));
                            }
                            ChevronSummon::Caret => {
                                let mut on = fold_view_for(&text, collapsed, true);
                                on.zoom = zoom;
                                p.set_view(&on);
                                p.set_hover_line(None);
                            }
                        }
                        p.prepare(&device, &queue, width, height).unwrap();
                        let geom = p.fold_chevron_geometries()[0];
                        let after = pixeldiff::render_frame(&mut p, &device, &queue, width, height);
                        // Caret summon also reveals raw markdown and moves the
                        // caret, so isolate the shared hit box in that arm. Hover
                        // changes only the chevron; its wider search additionally
                        // proves no painted pixel escapes the box.
                        let (x0, y0, x1, y1) = match summon {
                            ChevronSummon::Caret => (
                                geom.left.floor() as i32,
                                geom.row_top.floor() as i32,
                                (geom.left + geom.width).ceil() as i32,
                                (geom.row_top + geom.row_height).ceil() as i32,
                            ),
                            ChevronSummon::Hover => (
                                p.column_left().floor() as i32,
                                (geom.row_top - geom.row_height).floor() as i32,
                                p.text_left().ceil() as i32,
                                (geom.row_top + geom.row_height * 2.0).ceil() as i32,
                            ),
                        };
                        let (bbox, changed) =
                            changed_pixels_in(&before, &after, width, height, x0, y0, x1, y1);
                        let center_delta = (bbox.center_y() - geom.row_center()).abs();
                        assert!(
                            center_delta <= 1.5,
                            "{label}: fold-chevron real-pixel ink bbox {bbox:?} centre {:.2}, \
                             shaped row centre {:.2}, delta {center_delta:.2}",
                            bbox.center_y(),
                            geom.row_center(),
                        );
                        for (x, y) in changed {
                            assert_eq!(
                                p.fold_chevron_hit(x as f32 + 0.5, y as f32 + 0.5),
                                Some(geom.line),
                                "{label}: painted pixel ({x},{y}) escapes the shared hit box"
                            );
                        }
                    }
                }
            }
        }
    }
    crate::page::set_page_on(true);
}
