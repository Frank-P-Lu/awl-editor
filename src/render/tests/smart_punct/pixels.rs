//! Real-pixel acceptance law for smart-punctuation ornaments.

use super::super::super::*;
use super::super::{headless_dqp, pixeldiff, view};

const W: u32 = 1200;
const H: u32 = 320;

#[derive(Clone, Copy, Debug)]
struct InkGeometry {
    count: usize,
    width: i64,
    height: i64,
    dominant: [u8; 4],
}

fn content_ink_geometry(
    pixels: &[[u8; 4]],
    canvas_w: i64,
    canvas_h: i64,
    region: pixeldiff::Region,
    expected: [u8; 4],
) -> Option<InkGeometry> {
    use std::collections::HashMap;
    let x0 = region.x.max(0);
    let y0 = region.y.max(0);
    let x1 = (region.x + region.w).min(canvas_w);
    let y1 = (region.y + region.h).min(canvas_h);
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (x1, x0, y1, y0);
    let mut count = 0usize;
    let mut colors = HashMap::new();
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
                *colors.entry(pixel).or_insert(0usize) += 1;
            }
        }
    }
    let dominant = colors.into_iter().max_by_key(|(_, n)| *n)?.0;
    Some(InkGeometry {
        count,
        width: max_x - min_x + 1,
        height: max_y - min_y + 1,
        dominant,
    })
}

struct RenderedPair {
    source: Vec<[u8; 4]>,
    control: Vec<[u8; 4]>,
    cell: pixeldiff::Region,
    ground: [u8; 4],
}

fn render_pair(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    world: &str,
    kind: crate::markdown::SmartPunctKind,
) -> RenderedPair {
    let source_doc = format!("A {}\npark\n", kind.literal());
    let mut source_view = view(&source_doc, 1, 0);
    source_view.is_markdown = true;
    p.set_view(&source_view);
    let marks = p.smart_punct_marks();
    assert_eq!(marks.len(), 1, "{world} {kind:?}: one ornament enrolls");
    assert_eq!(marks[0].2, kind, "{world}: enrolled kind");
    p.prepare(device, queue, W, H).unwrap();
    let source = pixeldiff::render_frame(p, device, queue, W, H);

    let control_doc = format!("A {}\npark\n", kind.glyph());
    let mut control_view = view(&control_doc, 1, 0);
    control_view.is_markdown = true;
    p.set_view(&control_view);
    assert!(
        p.smart_punct_marks().is_empty(),
        "{world} {kind:?}: Unicode control has no ornament"
    );
    let row = p.visual_rows(0)[0].clone();
    assert_eq!(
        row.end_col - row.start_col,
        3,
        "{world} {kind:?}: prefix plus one Unicode glyph"
    );
    p.prepare(device, queue, W, H).unwrap();
    let control = pixeldiff::render_frame(p, device, queue, W, H);
    let x0 = p.text_left() + row.xs[2] - 3.0;
    let x1 = p.text_left() + row.xs[3] + 3.0;
    let y0 = p.line_ornament_top(0);
    let cell = pixeldiff::Region::new(
        x0,
        y0 - 3.0,
        (x1 - x0).max(1.0),
        row.line_height.max(1.0) + 6.0,
    );
    let bg_x = (p.text_left() + row.xs[3] + 24.0) as usize;
    let bg_y = (y0 + row.line_height * 0.5) as usize;
    let ground = control[bg_y * W as usize + bg_x];
    RenderedPair {
        source,
        control,
        cell,
        ground,
    }
}

fn grade_pair(pair: &RenderedPair, world: &str, kind: crate::markdown::SmartPunctKind) {
    let body_ink =
        pixeldiff::dominant_ink_color(&pair.control, W as i64, H as i64, pair.cell, pair.ground, 8)
            .unwrap_or_else(|| panic!("{world} {kind:?}: body control absent in {:?}", pair.cell));
    let control = content_ink_geometry(&pair.control, W as i64, H as i64, pair.cell, body_ink)
        .unwrap_or_else(|| panic!("{world} {kind:?}: body control has no ink geometry"));
    let source = content_ink_geometry(&pair.source, W as i64, H as i64, pair.cell, body_ink)
        .unwrap_or_else(|| {
            panic!(
                "{world} {kind:?}: ornament has no pixels in body ink {body_ink:?} within {:?}",
                pair.cell
            )
        });
    assert!(
        source.count >= 3 && control.count >= 3,
        "{world} {kind:?}: both subjects need pixel presence; source={source:?} control={control:?}"
    );
    let width_tolerance = (control.width / 5).max(2);
    let height_tolerance = (control.height / 5).max(1);
    assert!(
        (source.width - control.width).abs() <= width_tolerance
            && (source.height - control.height).abs() <= height_tolerance,
        "{world} {kind:?}: ornament must occupy the body's full ink box; \
         tolerance={width_tolerance}×{height_tolerance}px source={source:?} control={control:?}"
    );
    let ink_delta = source
        .dominant
        .into_iter()
        .zip(body_ink)
        .map(|(a, b)| a.abs_diff(b))
        .max()
        .unwrap_or(0);
    assert!(
        ink_delta <= 1,
        "{world} {kind:?}: ornament must use body ink; delta={ink_delta} \
         source={source:?} body={body_ink:?} ground={:?}",
        pair.ground
    );
    assert_eq!(
        control.dominant, body_ink,
        "{world} {kind:?}: presence control carries body ink"
    );
}

/// Every off-caret ASCII run renders like the corresponding ordinary Unicode
/// punctuation glyph at full body size and in body ink. Both rosters own
/// enrolment; the per-cell pixel-presence floor keeps equality non-vacuous.
#[test]
fn smart_punct_ornament_is_full_body_glyph_in_content_ink_every_world() {
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
