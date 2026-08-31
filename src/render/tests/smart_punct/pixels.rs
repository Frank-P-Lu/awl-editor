//! Real-pixel acceptance law for smart-punctuation ornaments.

use super::super::super::*;
use super::super::{headless_dqp, pixeldiff, view};

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
    let mut min_x = x1;
    let mut max_x = x0;
    let mut min_y = y1;
    let mut max_y = y0;
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

/// Every off-caret ASCII source run must render exactly like the corresponding
/// ordinary Unicode punctuation character at full body size and in body ink.
///
/// The whole world roster and the closed punctuation roster are both the
/// enrolment source. The source ornament's real-pixel ink box must match the
/// Unicode body's within a two-pixel/20% raster-phase tolerance, and both must
/// carry the body's rendered content ink. The isolated cells must contain
/// multiple ink pixels, so deleting both subjects cannot make the comparison
/// vacuously green.
#[test]
fn smart_punct_ornament_is_full_body_glyph_in_content_ink_every_world() {
    let _t = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    crate::page::set_measure(80);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 320.0) else {
        eprintln!(
            "skipping smart_punct_ornament_is_full_body_glyph_in_content_ink_every_world: \
             no wgpu adapter"
        );
        return;
    };
    let (w, h) = (1200u32, 320u32);
    let mut enrolled = Vec::new();

    for world in theme::THEMES {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for kind in crate::markdown::SmartPunctKind::ALL {
            // A prose prefix keeps `---` out of the thematic-break grammar;
            // the substitute remains isolated at the row's tail for pixel QA.
            let source_doc = format!("A {}\npark\n", kind.literal());
            let mut source_view = view(&source_doc, 1, 0);
            source_view.is_markdown = true;
            p.set_view(&source_view);
            let marks = p.smart_punct_marks();
            assert_eq!(
                marks.len(),
                1,
                "{} {kind:?}: exactly one ornament must enroll",
                world.name
            );
            assert_eq!(marks[0].2, kind, "{}: enrolled kind", world.name);
            p.prepare(&device, &queue, w, h).unwrap();
            let source_pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

            let control_doc = format!("A {}\npark\n", kind.glyph());
            let mut control_view = view(&control_doc, 1, 0);
            control_view.is_markdown = true;
            p.set_view(&control_view);
            assert!(
                p.smart_punct_marks().is_empty(),
                "{} {kind:?}: Unicode control must not create an ornament",
                world.name
            );
            let row = p.visual_rows(0)[0].clone();
            assert_eq!(
                row.end_col - row.start_col,
                3,
                "{} {kind:?}: control is the two-character prefix plus one Unicode glyph",
                world.name
            );
            p.prepare(&device, &queue, w, h).unwrap();
            let control_pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

            let x0 = p.text_left() + row.xs[2] - 3.0;
            let x1 = p.text_left() + row.xs[3] + 3.0;
            let y0 = p.line_ornament_top(0);
            let glyph_cell = pixeldiff::Region::new(
                x0,
                y0 - 3.0,
                (x1 - x0).max(1.0),
                row.line_height.max(1.0) + 6.0,
            );
            let bg_x = (p.text_left() + row.xs[3] + 24.0) as usize;
            let bg_y = (y0 + row.line_height * 0.5) as usize;
            let bg = control_pixels[bg_y * w as usize + bg_x];
            let body_ink = pixeldiff::dominant_ink_color(
                &control_pixels,
                w as i64,
                h as i64,
                glyph_cell,
                bg,
                8,
            )
            .unwrap_or_else(|| {
                panic!(
                    "{} {kind:?}: Unicode body control has no dominant ink in {glyph_cell:?}",
                    world.name
                )
            });
            let control_ink = content_ink_geometry(
                &control_pixels,
                w as i64,
                h as i64,
                glyph_cell,
                body_ink,
            )
            .unwrap_or_else(|| {
                panic!(
                    "{} {kind:?}: Unicode control glyph has no pixel presence in {glyph_cell:?}",
                    world.name
                )
            });
            let source_ink =
                content_ink_geometry(&source_pixels, w as i64, h as i64, glyph_cell, body_ink)
                    .unwrap_or_else(|| {
                        panic!(
                            "{} {kind:?}: source ornament has no pixels in the Unicode body's \
                     content ink {body_ink:?} within {glyph_cell:?}",
                            world.name
                        )
                    });
            assert!(
                source_ink.count >= 3 && control_ink.count >= 3,
                "{} {kind:?}: both subjects need real pixel presence; \
                 source={source_ink:?} control={control_ink:?}",
                world.name
            );
            let width_tolerance = (control_ink.width / 5).max(2);
            let height_tolerance = (control_ink.height / 5).max(1);
            assert!(
                (source_ink.width - control_ink.width).abs() <= width_tolerance
                    && (source_ink.height - control_ink.height).abs() <= height_tolerance,
                "{} {kind:?}: ornament must occupy the Unicode body's full ink box; \
                 tolerance={width_tolerance}×{height_tolerance}px \
                 source={source_ink:?} control={control_ink:?}",
                world.name
            );
            let source_ink_delta = source_ink
                .dominant
                .into_iter()
                .zip(body_ink)
                .map(|(a, b)| a.abs_diff(b))
                .max()
                .unwrap_or(0);
            assert!(
                source_ink_delta <= 1,
                "{} {kind:?}: dominant ornament ink must be base content (allowing one byte \
                 of render-target rounding); delta={source_ink_delta} source={source_ink:?} \
                 body_control={body_ink:?} glyph_cell={glyph_cell:?} bg={bg:?}",
                world.name
            );
            assert_eq!(
                control_ink.dominant, body_ink,
                "{} {kind:?}: Unicode presence control must itself carry base-content ink; \
                 control={control_ink:?}",
                world.name
            );
            enrolled.push((world.name, kind));
        }
    }
    assert_eq!(
        enrolled.len(),
        theme::THEMES.len() * crate::markdown::SmartPunctKind::ALL.len(),
        "full world × punctuation roster enrolled: {enrolled:?}"
    );
}
