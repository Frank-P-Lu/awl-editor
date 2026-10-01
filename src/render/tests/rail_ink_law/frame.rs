//! Paired real frames and an independent raster support mask for the label.
//! Pixels inside that support are still graded from the rendered image.

use super::*;
use crate::render::tests::pixeldiff::render_frame;
use crate::render::tests::workspace::{workspace_card, workspace_view};
use std::collections::BTreeSet;

pub(super) struct RailFrame {
    pub pixels: Vec<[u8; 4]>,
    pub rect: Option<[f32; 4]>,
    pub label: Option<String>,
    pub label_w: Option<f32>,
    pub glyph_pixels: BTreeSet<(i64, i64)>,
}

pub(super) fn frame_with_lens(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    lens: usize,
    detail: bool,
    (w, h, dpi): (u32, u32, f32),
) -> RailFrame {
    let ov = workspace_card(lens, detail);
    p.set_dpi(dpi);
    p.set_size(w as f32, h as f32);
    p.set_view(&workspace_view(&ov));
    p.prepare(device, queue, w, h).unwrap();
    let rect = p.workspace_rail_probe(w).rows.first().copied().flatten();
    let label = p
        .workspace_rail_buffer
        .lines
        .first()
        .map(|l| l.text().to_owned());
    let label_w = p
        .workspace_rail_buffer
        .layout_runs()
        .next()
        .map(|r| r.line_w);
    let glyph_pixels = label_support(p);
    RailFrame {
        pixels: render_frame(p, device, queue, w, h),
        rect,
        label,
        label_w,
        glyph_pixels,
    }
}

/// Read Swash directly rather than the production optical-bounds cache. A
/// support mask identifies the label's footprint, never its rendered color.
fn label_support(p: &mut TextPipeline) -> BTreeSet<(i64, i64)> {
    let mut pixels = BTreeSet::new();
    let Some(origin) = p.workspace_rail_placement else {
        return pixels;
    };
    let mut swash = SwashCache::new();
    for run in p
        .workspace_rail_buffer
        .layout_runs()
        .filter(|r| r.line_i == 0)
    {
        for glyph in run.glyphs {
            let physical = glyph.physical(origin, 1.0);
            let Some(image) = swash.get_image(&mut p.font_system, physical.cache_key) else {
                continue;
            };
            assert_eq!(
                image.content,
                SwashContent::Mask,
                "category uses a text mask"
            );
            let x = physical.x + image.placement.left;
            let y = run.line_y.round() as i32 + physical.y - image.placement.top;
            for (i, &coverage) in image.data.iter().enumerate() {
                if coverage != 0 {
                    pixels.insert((
                        x as i64 + (i % image.placement.width as usize) as i64,
                        y as i64 + (i / image.placement.width as usize) as i64,
                    ));
                }
            }
        }
    }
    pixels
}

/// The paired capture contract belongs with capture and raster-support facts.
pub(super) fn paired_label_lane(
    at: &str,
    marked: &RailFrame,
    bare: &RailFrame,
) -> Option<([f32; 4], f32)> {
    let (rect, rect_bare) = (marked.rect?, bare.rect?);
    let (label_a, label_b) = (&marked.label, &bare.label);
    assert_eq!(
        (label_a.as_deref(), label_b.as_deref()),
        (Some("All"), Some("All")),
        "{at}: the first rail row must keep its `All` label in both selected states; \
        another workspace measurement overwrote it ({label_a:?} vs {label_b:?})"
    );
    let label_w = marked
        .label_w
        .expect("the active rail entry must have a shaped label width");
    let label_w_bare = bare
        .label_w
        .expect("the bare rail entry must have a shaped label width");
    assert!(
        (label_w - label_w_bare).abs() < 0.5,
        "{at}: the same `All` label shaped to different widths ({label_w:.1}px vs \
         {label_w_bare:.1}px) between paired frames"
    );
    assert!(
        rect.iter()
            .zip(rect_bare.iter())
            .all(|(a, b)| (a - b).abs() < 0.5),
        "{at}: the rail's first entry moved between the two frames ({rect:?} vs \
         {rect_bare:?}) — this law's whole oracle is that they are the same rect"
    );
    assert_eq!(
        marked.glyph_pixels, bare.glyph_pixels,
        "{at}: paired label raster moved"
    );
    assert!(
        !marked.glyph_pixels.is_empty(),
        "{at}: label raster must be nonempty"
    );
    Some((rect, label_w))
}
