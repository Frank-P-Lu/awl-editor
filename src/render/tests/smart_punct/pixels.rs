//! Real-pixel acceptance law for smart-punctuation ornaments.

use super::super::super::*;
use super::super::{headless_dqp, pixeldiff, view};

const W: u32 = 1200;
const H: u32 = 320;
// The shipped roster reads 1.000..=1.286 on Metal. The band is deliberately
// broad around the Unicode control; its lower edge separates the 0.5-scale
// regression, while its upper edge admits raster-phase growth without making
// a doubled ornament acceptable.
const WIDTH_RATIO_BAND: std::ops::RangeInclusive<f64> = 0.75..=1.35;

#[derive(Clone, Copy, Debug)]
struct InkGeometry {
    count: usize,
    left: i64,
    width: i64,
    height: i64,
}

fn core_ink_geometry(
    pixels: &[[u8; 4]],
    canvas_w: i64,
    canvas_h: i64,
    region: pixeldiff::Region,
    expected: [u8; 4],
) -> Option<InkGeometry> {
    let x0 = region.x.max(0);
    let y0 = region.y.max(0);
    let x1 = (region.x + region.w).min(canvas_w);
    let y1 = (region.y + region.h).min(canvas_h);
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (x1, x0, y1, y0);
    let mut count = 0usize;
    for y in y0..y1 {
        for x in x0..x1 {
            let pixel = pixels[(y * canvas_w + x) as usize];
            let delta = pixel
                .into_iter()
                .zip(expected)
                .map(|(a, b)| a.abs_diff(b))
                .max()
                .unwrap_or(0);
            if delta <= 2 {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                count += 1;
            }
        }
    }
    if count == 0 {
        return None;
    }
    Some(InkGeometry {
        count,
        left: min_x,
        width: max_x - min_x + 1,
        height: max_y - min_y + 1,
    })
}

fn modal_color(
    pixels: &[[u8; 4]],
    canvas_w: i64,
    canvas_h: i64,
    region: pixeldiff::Region,
) -> Option<[u8; 4]> {
    use std::collections::HashMap;
    let x0 = region.x.max(0);
    let y0 = region.y.max(0);
    let x1 = (region.x + region.w).min(canvas_w);
    let y1 = (region.y + region.h).min(canvas_h);
    let mut colors = HashMap::new();
    for y in y0..y1 {
        for x in x0..x1 {
            *colors
                .entry(pixels[(y * canvas_w + x) as usize])
                .or_insert(0usize) += 1;
        }
    }
    colors.into_iter().max_by_key(|(_, n)| *n).map(|(p, _)| p)
}

fn strongest_ink_color(
    pixels: &[[u8; 4]],
    canvas_w: i64,
    canvas_h: i64,
    region: pixeldiff::Region,
    ground: [u8; 4],
) -> Option<[u8; 4]> {
    let x0 = region.x.max(0);
    let y0 = region.y.max(0);
    let x1 = (region.x + region.w).min(canvas_w);
    let y1 = (region.y + region.h).min(canvas_h);
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| pixels[(y * canvas_w + x) as usize]))
        .max_by(|a, b| {
            pixeldiff::delta_e(*a, ground)
                .partial_cmp(&pixeldiff::delta_e(*b, ground))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

struct RenderedPair {
    pixels: Vec<[u8; 4]>,
    source_cell: pixeldiff::Region,
    control_cell: pixeldiff::Region,
    source_suffix_cell: pixeldiff::Region,
    control_suffix_cell: pixeldiff::Region,
}

#[derive(Clone, Copy)]
struct CellInk {
    ground: [u8; 4],
    strongest: [u8; 4],
    geometry: InkGeometry,
}

fn cell_ink(
    pair: &RenderedPair,
    region: pixeldiff::Region,
    world: &str,
    kind: crate::markdown::SmartPunctKind,
    subject: &str,
) -> CellInk {
    let ground = modal_color(&pair.pixels, W as i64, H as i64, region)
        .unwrap_or_else(|| panic!("{world} {kind:?}: {subject} cell is empty"));
    let strongest = strongest_ink_color(&pair.pixels, W as i64, H as i64, region, ground)
        .unwrap_or_else(|| panic!("{world} {kind:?}: {subject} is absent in {region:?}"));
    let geometry = core_ink_geometry(&pair.pixels, W as i64, H as i64, region, strongest)
        .unwrap_or_else(|| panic!("{world} {kind:?}: {subject} has no core ink"));
    CellInk {
        ground,
        strongest,
        geometry,
    }
}

fn render_pair(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    world: &str,
    kind: crate::markdown::SmartPunctKind,
) -> RenderedPair {
    let doc = format!(
        "A {}     tail\n\nA {}     tail\n\npark\n",
        kind.literal(),
        kind.glyph()
    );
    let mut v = view(&doc, 4, 0);
    v.is_markdown = true;
    p.set_view(&v);
    let marks = p.smart_punct_marks();
    assert_eq!(marks.len(), 1, "{world} {kind:?}: one ornament enrolls");
    assert_eq!(marks[0].2, kind, "{world}: enrolled kind");
    let source_row = p.visual_rows(0)[0].clone();
    let control_row = p.visual_rows(2)[0].clone();
    assert_eq!(control_row.end_col - control_row.start_col, 12);
    p.prepare(device, queue, W, H).unwrap();
    let pixels = pixeldiff::render_frame(p, device, queue, W, H);

    let source_x0 = marks[0].1 - 3.0;
    let source_x1 = marks[0].1 + marks[0].3 + 3.0;
    let source_y0 = marks[0].0;
    let source_cell = pixeldiff::Region::new(
        source_x0,
        source_y0 - 3.0,
        (source_x1 - source_x0).max(1.0),
        source_row.line_height.max(1.0) + 6.0,
    );
    let control_x0 = p.text_left() + control_row.xs[2] - 3.0;
    let control_x1 = control_x0 + marks[0].3 + 6.0;
    let control_y0 = p.line_ornament_top(2);
    let control_cell = pixeldiff::Region::new(
        control_x0,
        control_y0 - 3.0,
        (control_x1 - control_x0).max(1.0),
        control_row.line_height.max(1.0) + 6.0,
    );
    let source_suffix_col = 7 + kind.literal().chars().count();
    let source_suffix_x = p.text_left() + source_row.xs[source_suffix_col];
    let control_suffix_x = p.text_left() + control_row.xs[8];
    let suffix_x0 = source_suffix_x.min(control_suffix_x) - 4.0;
    let suffix_x1 = p.text_left()
        + source_row
            .xs
            .last()
            .copied()
            .unwrap_or(0.0)
            .max(control_row.xs.last().copied().unwrap_or(0.0))
        + 3.0;
    let source_suffix_cell = pixeldiff::Region::new(
        suffix_x0,
        source_y0 - 3.0,
        suffix_x1 - suffix_x0,
        source_row.line_height.max(1.0) + 6.0,
    );
    let control_suffix_cell = pixeldiff::Region::new(
        suffix_x0,
        control_y0 - 3.0,
        suffix_x1 - suffix_x0,
        control_row.line_height.max(1.0) + 6.0,
    );
    assert!(
        source_cell.x + source_cell.w <= source_suffix_cell.x
            && control_cell.x + control_cell.w <= control_suffix_cell.x,
        "{world} {kind:?}: punctuation cells must not admit suffix ink"
    );
    assert!(
        source_cell.y + source_cell.h <= control_cell.y
            && source_suffix_cell.y + source_suffix_cell.h <= control_suffix_cell.y,
        "{world} {kind:?}: source and control cells must be vertically disjoint"
    );
    RenderedPair {
        pixels,
        source_cell,
        control_cell,
        source_suffix_cell,
        control_suffix_cell,
    }
}

fn grade_pair(pair: &RenderedPair, world: &str, kind: crate::markdown::SmartPunctKind) {
    let source = cell_ink(pair, pair.source_cell, world, kind, "ornament");
    let control = cell_ink(pair, pair.control_cell, world, kind, "body control");
    let source_suffix = cell_ink(pair, pair.source_suffix_cell, world, kind, "source suffix");
    let control_suffix = cell_ink(
        pair,
        pair.control_suffix_cell,
        world,
        kind,
        "control suffix",
    );
    assert!(
        source.geometry.count >= 3 && control.geometry.count >= 3,
        "{world} {kind:?}: both subjects need pixel presence; source={:?} control={:?}",
        source.geometry,
        control.geometry,
    );
    let width_ratio = source.geometry.width as f64 / control.geometry.width.max(1) as f64;
    let suffix_alignment_tolerance = (control_suffix.geometry.width / 10).max(1);
    assert!(
        (source_suffix.geometry.left - control_suffix.geometry.left).abs()
            <= suffix_alignment_tolerance
            && WIDTH_RATIO_BAND.contains(&width_ratio),
        "{world} {kind:?}: ornament must keep the body's shaped advance and full horizontal \
         ink extent; suffix left source={} control={} tolerance={suffix_alignment_tolerance}px; \
         width ratio={width_ratio:.3} band={WIDTH_RATIO_BAND:?} \
         source={:?} control={:?}; \
         vertical core is diagnostic only: {}px vs {}px",
        source_suffix.geometry.left,
        control_suffix.geometry.left,
        source.geometry,
        control.geometry,
        source.geometry.height,
        control.geometry.height,
    );
    let body_distance = pixeldiff::delta_e(control.strongest, control.ground);
    let source_distance = pixeldiff::delta_e(source.strongest, source.ground);
    let ink_travel = source_distance / body_distance.max(1e-9);
    let body_drift = pixeldiff::delta_e(source.strongest, control.strongest);
    assert!(
        ink_travel >= 0.75 && body_drift < source_distance,
        "{world} {kind:?}: ornament must use body ink from the same frame; \
         travel={ink_travel:.3} (floor 0.750) body_drift={body_drift:.3} \
         source_distance={source_distance:.3} source={:?} control={:?} \
         body={:?} source_ground={:?} control_ground={:?}",
        source.geometry,
        control.geometry,
        control.strongest,
        source.ground,
        control.ground,
    );
}

/// Every off-caret ASCII run matches the corresponding ordinary Unicode
/// punctuation glyph's shaped advance and horizontal ink extent, in body ink.
/// Punctuation has no shared vertical-extent requirement: an ellipsis can have
/// a one-raster-row core. Both rosters own enrolment; the per-cell pixel-
/// presence floor keeps the comparisons non-vacuous.
#[test]
fn smart_punct_ornament_matches_body_advance_horizontal_extent_and_ink_every_world() {
    let _t = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    crate::page::set_measure(80);
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping smart-punctuation pixel law: no wgpu adapter");
        return;
    };
    let mut enrolled = Vec::new();
    for world in theme::THEMES {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for kind in crate::markdown::SmartPunctKind::ALL {
            let pair = render_pair(&mut p, &device, &queue, world.name, kind);
            grade_pair(&pair, world.name, kind);
            enrolled.push((world.name, kind));
        }
    }
    assert_eq!(
        enrolled.len(),
        theme::THEMES.len() * crate::markdown::SmartPunctKind::ALL.len(),
        "full world × punctuation roster enrolled: {enrolled:?}"
    );
}
