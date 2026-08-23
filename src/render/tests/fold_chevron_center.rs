//! Real-pixel fold-chevron centring laws.

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
        "{} folded section\nbody kept inside the section\n{} sibling\nsibling body\n",
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
        crate::fold::apply_to_view(&mut v, &hidden, &tails, &folds);
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
        eprintln!("skipping full-world fold-chevron pixel law: no GPU adapter");
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
fn wagtail_h2_chevron_centres_across_state_viewport_dpi_and_zoom() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Wagtail axis-sweep fold-chevron pixel law: no GPU adapter");
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

/// THE MARK RIDES ITS HEADING'S OWN LADDER STEP. An H1's chevron and its gap
/// are `heading_scale(1)` times the base mark, an H3's `heading_scale(3)` —
/// asserted twice, at the two seams that can independently regress: the GEOM
/// arithmetic (box width and gap proportional to the geom's own pad-clamped
/// `scale`, which on a wide canvas is exactly the ladder step) and the REAL
/// PAINTED INK (differential bbox extents strictly ordered H3 < H2 < H1, with
/// the H1:H3 extent ratio in the ladder's own neighbourhood — this arm is what
/// fails on a paint path that sizes every mark at the base char width, the
/// exact pre-scaling renderer). The pad CLAMP is graded at the pure seam below.
#[test]
fn fold_chevron_ink_and_box_ride_the_heading_ladder() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping fold-chevron ladder-scale pixel law: no GPU adapter");
        return;
    };
    crate::theme::set_active_by_name("Gumtree").unwrap();
    p.sync_theme();

    let mut per_level: Vec<(u8, crate::render::layers::fold_chevron::FoldChevronGeom, PixelBox)> =
        Vec::new();
    for level in 1..=3u8 {
        let bbox = assert_chevron_pixel_center(
            &mut p,
            &device,
            &queue,
            1200,
            800,
            level as usize,
            false,
            ChevronSummon::Hover,
            &format!("ladder-scale H{level}"),
        );
        let geom = p.fold_chevron_geometries()[0];
        let ladder = crate::markdown::heading_scale(level);
        assert!(
            (geom.scale - ladder).abs() < 1e-5,
            "H{level}: on a wide canvas the geom's scale must be the heading's own \
             ladder step {ladder}, got {}",
            geom.scale
        );
        per_level.push((level, geom, bbox));
    }

    // Geometry proportionality: width/scale and gap/scale are one shared base
    // value across all levels — the scaled box is the base box times the step.
    let base_w = per_level[0].1.width / per_level[0].1.scale;
    let base_gap = (p.text_left() - (per_level[0].1.left + per_level[0].1.width))
        / per_level[0].1.scale;
    for (level, geom, _) in &per_level {
        let w = geom.width / geom.scale;
        let gap = (p.text_left() - (geom.left + geom.width)) / geom.scale;
        assert!(
            (w - base_w).abs() < 0.01 && (gap - base_gap).abs() < 0.01,
            "H{level}: box width {w:.3} and gap {gap:.3} (per unit scale) must match \
             the shared base ({base_w:.3}, {base_gap:.3})"
        );
    }

    // Painted-ink ordering and magnitude: extents shrink strictly down the
    // ladder, and H1's ink exceeds H3's by an amount only the ladder explains
    // (ratio graded with slack for anti-aliased edges at small sizes).
    let ink_w = |b: &PixelBox| (b.right - b.left + 1) as f32;
    let ink_h = |b: &PixelBox| (b.bottom - b.top + 1) as f32;
    let (h1, h2, h3) = (&per_level[0].2, &per_level[1].2, &per_level[2].2);
    assert!(
        ink_w(h1) > ink_w(h2) && ink_w(h2) > ink_w(h3) && ink_h(h1) > ink_h(h3),
        "painted ink extents must be strictly ordered down the ladder: \
         H1 {h1:?}, H2 {h2:?}, H3 {h3:?}"
    );
    let ratio = ink_w(h1) / ink_w(h3);
    let ladder_ratio = crate::markdown::heading_scale(1) / crate::markdown::heading_scale(3);
    assert!(
        (ratio - ladder_ratio).abs() <= 0.25,
        "H1:H3 painted ink width ratio {ratio:.3} must sit in the ladder's own \
         neighbourhood {ladder_ratio:.3} ± 0.25 — equal-size ink (ratio 1.0) is \
         the pre-scaling renderer this law exists to catch"
    );
}

/// The pad CLAMP at its pure seam: the scale is the ladder step while the pad
/// affords it, degrades to exactly what fits when the pad is tighter than the
/// step, and floors at the base size (the gate `fold_chevron_has_room` owns
/// the decision to draw at all below that).
#[test]
fn fold_chevron_scale_clamps_to_the_pad_it_hangs_in() {
    use crate::render::layers::fold_chevron::fold_chevron_scale;
    let cw = 8.0f32;
    let generous = cw * 100.0;
    for level in 1..=6u8 {
        let ladder = crate::markdown::heading_scale(level);
        assert!(
            (fold_chevron_scale(level, cw, generous) - ladder).abs() < 1e-6,
            "H{level}: a generous pad yields the full ladder step"
        );
    }
    // H4+ shares H3's step (the ladder's `_ => SUBHEAD` arm).
    assert_eq!(
        fold_chevron_scale(6, cw, generous),
        fold_chevron_scale(3, cw, generous),
        "a deep heading rides the subhead step, not an invented deeper one"
    );
    assert!(
        (fold_chevron_scale(1, cw, 0.0) - 1.0).abs() < 1e-6,
        "with no pad at all the scale floors at 1.0 — whether to draw at all is \
         the room gate's call, never this fn's"
    );
    // The degrade region is LINEAR in the pad. Recover the fn's OWN base need
    // from the first in-between sample rather than restating the authored
    // gap/width fractions here, then require every sample to sit on that line.
    let ladder1 = crate::markdown::heading_scale(1);
    let samples: Vec<(f32, f32)> = (1..400)
        .map(|i| {
            let pad = cw * 0.01 * i as f32;
            (pad, fold_chevron_scale(1, cw, pad))
        })
        .collect();
    let inbetween: Vec<(f32, f32)> = samples
        .iter()
        .copied()
        .filter(|(_, s)| *s > 1.0 + 1e-4 && *s < ladder1 - 1e-4)
        .collect();
    assert!(
        inbetween.len() >= 3,
        "the pad sweep must cross the degrade region between base and the H1 step"
    );
    let need = inbetween[0].0 / inbetween[0].1;
    for (pad, s) in &samples {
        let expect = (pad / need).clamp(1.0, ladder1);
        assert!(
            (s - expect).abs() < 1e-4,
            "pad {pad}: scale {s} must sit on the clamp line (expected {expect}) — \
             ladder while it fits, exactly-what-fits while it doesn't, floored at base"
        );
    }
}

/// TWO CHEVRONS OF DIFFERENT LEVELS IN ONE BATCH — the caret summons an H1's
/// mark while a hover summons an H3's, the one way the product paints two sizes
/// through the single shared `SelectionPipeline` upload at once. Two properties
/// only this shape can grade:
///
/// (1) EACH MARK KEEPS ITS OWN SIZE IN COMPANY. A paint path that resolved the
/// batch's metrics once (from the first geom, or the base char width) would
/// size both marks identically — so the H1 mark's ink, measured inside its own
/// box in the mixed frame, must exceed the H3 mark's.
///
/// (2) A MARK'S PIXELS ARE UNCHANGED BY WHO SHARES ITS BATCH. Asserted as
/// byte-identity: the H3 box's pixels with the H1 summoned alongside equal the
/// H3-alone frame's, exactly — red under any batch-membership leak (proven on
/// the shared-metrics regression above, which resizes the H3 the moment the H1
/// joins). The one per-batch uniform that legitimately varies with membership,
/// the corner radius (seeded from the smallest mark's half-stroke), is INERT at
/// shipped metrics — a wrong max-seed rasterizes identically here, which is the
/// same inertness the batch-corner unit law pins — so this arm neither guards
/// nor is fooled by it.
#[test]
fn two_chevrons_of_different_levels_share_one_batch_each_at_its_own_size() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping mixed-level fold-chevron batch law: no GPU adapter");
        return;
    };
    crate::theme::set_active_by_name("Gumtree").unwrap();
    p.sync_theme();

    let text = "# title\nbody one\n### deep\nbody two\n";
    let (width, height) = (1200u32, 800u32);
    let h1_line = 0usize;
    let h3_line = 2usize;

    // Base: caret parked on a body line, no hover — zero chevrons.
    p.set_view(&view_md(text, 3, 0));
    p.set_hover_line(None);
    p.prepare(&device, &queue, width, height).unwrap();
    assert!(p.fold_chevron_geometries().is_empty(), "base frame: no marks");
    let base = pixeldiff::render_frame(&mut p, &device, &queue, width, height);

    // H3 alone: hover summons it, caret stays on the body line.
    p.set_hover_line(Some(h3_line));
    p.prepare(&device, &queue, width, height).unwrap();
    let solo_geoms = p.fold_chevron_geometries();
    assert_eq!(solo_geoms.len(), 1, "hover alone summons exactly the H3 mark");
    let h3_solo = solo_geoms[0];
    let h3_frame = pixeldiff::render_frame(&mut p, &device, &queue, width, height);

    // Mixed: the caret moves onto the H1 while the H3 hover holds.
    p.set_view(&view_md(text, h1_line, 0));
    p.set_hover_line(Some(h3_line));
    p.prepare(&device, &queue, width, height).unwrap();
    let geoms = p.fold_chevron_geometries();
    assert_eq!(
        geoms.len(),
        2,
        "caret on the H1 plus hover on the H3 summons both marks at once"
    );
    let h1 = geoms.iter().find(|g| g.line == h1_line).expect("H1 geom");
    let h3 = geoms.iter().find(|g| g.line == h3_line).expect("H3 geom");
    assert!(
        (h1.scale - crate::markdown::heading_scale(1)).abs() < 1e-5
            && (h3.scale - crate::markdown::heading_scale(3)).abs() < 1e-5,
        "each geom carries its OWN ladder step in the mixed batch: H1 {} H3 {}",
        h1.scale,
        h3.scale
    );
    assert_eq!(*h3, h3_solo, "the H3 geom is unchanged by the H1 joining");
    let mixed = pixeldiff::render_frame(&mut p, &device, &queue, width, height);

    // A geom's pixel box, padded a row either side so escaping ink is seen.
    let boxed = |g: &crate::render::layers::fold_chevron::FoldChevronGeom| {
        (
            g.left.floor() as i32,
            (g.row_top - g.row_height).floor() as i32,
            (g.left + g.width).ceil() as i32,
            (g.row_top + g.row_height * 2.0).ceil() as i32,
        )
    };

    // (2) Byte-identity of the H3 mark's box between solo and mixed frames.
    let (x0, y0, x1, y1) = boxed(h3);
    for y in y0.max(0) as u32..(y1.min(height as i32)).max(0) as u32 {
        for x in x0.max(0) as u32..(x1.min(width as i32)).max(0) as u32 {
            let i = (y * width + x) as usize;
            assert_eq!(
                h3_frame[i], mixed[i],
                "H3 mark pixel ({x},{y}) changed when the H1 mark joined its \
                 batch — some per-batch state (shared metrics resolution, a \
                 membership-dependent uniform) is leaking across marks"
            );
        }
    }

    // (1) Each mark's own ink, differenced against the zero-chevron base,
    // measured inside its own box in the frame where both are live. The caret
    // also reveals the H1's raw markdown, but that ink lives at text_left and
    // rightward — each geom box ends a gap short of it, so the diff inside the
    // box is the chevron alone. EXACT row bounds here, not the padded box: the
    // tall H1 row's padded box reaches down into the H3 mark's own rows, and a
    // measure that swept both would blame one mark for the other's ink (the
    // solo centring laws already own the escaped-ink sweep).
    let rowbox = |g: &crate::render::layers::fold_chevron::FoldChevronGeom| {
        (
            g.left.floor() as i32,
            g.row_top.floor() as i32,
            (g.left + g.width).ceil() as i32,
            (g.row_top + g.row_height).ceil() as i32,
        )
    };
    let (bx, by, bx1, by1) = rowbox(h1);
    let (h1_ink, _) = changed_pixels_in(&base, &mixed, width, height, bx, by, bx1, by1);
    let (cx, cy, cx1, cy1) = rowbox(h3);
    let (h3_ink, _) = changed_pixels_in(&base, &mixed, width, height, cx, cy, cx1, cy1);
    let w = |b: &PixelBox| (b.right - b.left + 1) as f32;
    let h = |b: &PixelBox| (b.bottom - b.top + 1) as f32;
    assert!(
        w(&h1_ink) > w(&h3_ink) && h(&h1_ink) > h(&h3_ink),
        "in the mixed batch the H1 mark's ink must exceed the H3's on both \
         axes — one shared metrics resolution for the whole batch sizes them \
         equally: H1 {h1_ink:?}, H3 {h3_ink:?}"
    );
    for (g, ink) in [(h1, &h1_ink), (h3, &h3_ink)] {
        let delta = (ink.center_y() - g.row_center()).abs();
        assert!(
            delta <= 1.5,
            "line {}: mixed-batch mark ink centres {delta:.2}px off its own row",
            g.line
        );
    }
}
