//! CPU preview of the already-built PDF display list. This deliberately takes
//! [`Layout`] rather than markdown: page switching paints the same glyph IDs,
//! baselines, rectangles, images, and pagination the writer consumes.

use glyphon::cosmic_text::CacheKeyFlags;
use glyphon::{
    Attrs, Buffer, CacheKey, Family, Metrics, Shaping, SwashCache, SwashContent, Weight, Wrap,
};
use image::{Rgba, RgbaImage};

use super::fonts::{FontRole, Fonts};
use super::layout::{Layout, Op, PdfColor};

const MAX_PREVIEW_EDGE: u32 = 8192;

pub(super) fn render(layout: &Layout, page: usize, scale: f32) -> Result<RgbaImage, String> {
    let page = layout
        .pages
        .get(page)
        .ok_or_else(|| format!("PDF page {} is outside 0..{}", page + 1, layout.pages.len()))?;
    if !scale.is_finite() || scale <= 0.0 {
        return Err("PDF preview scale must be finite and greater than zero".into());
    }
    let width = scaled_dimension(layout.geometry.width, scale)?;
    let height = scaled_dimension(layout.geometry.height, scale)?;
    if width > MAX_PREVIEW_EDGE || height > MAX_PREVIEW_EDGE {
        return Err(format!(
            "PDF preview {width}x{height} exceeds the {MAX_PREVIEW_EDGE}px edge limit"
        ));
    }

    let mut canvas = RgbaImage::from_pixel(width, height, Rgba([255, 255, 255, 255]));
    let mut fonts = Fonts::new(layout.fonts.clone());
    let mut swash = SwashCache::new();
    for op in &page.ops {
        match op {
            Op::Rect { x, y, w, h, gray } => {
                fill_rect(
                    &mut canvas,
                    *x,
                    *y,
                    *w,
                    *h,
                    themed_fill(layout, *gray).unwrap_or_else(|| gray_color(*gray)),
                    scale,
                );
            }
            Op::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                gray,
            } => draw_line(
                &mut canvas,
                (*x1, *y1),
                (*x2, *y2),
                *width,
                themed_stroke(layout, *gray).unwrap_or_else(|| gray_color(*gray)),
                scale,
            ),
            Op::Glyph(glyph) => {
                let color = layout
                    .appearance
                    .palette
                    .map_or(PdfColor([0, 0, 0]), |palette| palette.ink);
                draw_glyph(&mut canvas, &mut fonts, &mut swash, glyph, color, scale);
            }
            Op::Image { index, x, y, w, h } => {
                let image = &layout.images[*index];
                if image.jpeg {
                    draw_jpeg_plate(
                        &mut canvas,
                        &mut fonts,
                        &mut swash,
                        (*x, *y, *w, *h),
                        layout,
                        scale,
                    );
                } else {
                    draw_png(&mut canvas, image, (*x, *y, *w, *h), scale);
                }
            }
            Op::ColorRect { x, y, w, h, color } => {
                fill_rect(&mut canvas, *x, *y, *w, *h, *color, scale)
            }
            Op::ColorLine {
                x1,
                y1,
                x2,
                y2,
                width,
                color,
            } => draw_line(&mut canvas, (*x1, *y1), (*x2, *y2), *width, *color, scale),
        }
    }
    Ok(canvas)
}

fn scaled_dimension(points: f32, scale: f32) -> Result<u32, String> {
    let pixels = (points * scale).ceil();
    if !pixels.is_finite() || pixels < 1.0 || pixels > u32::MAX as f32 {
        Err("PDF preview dimensions are outside the supported range".into())
    } else {
        Ok(pixels as u32)
    }
}

fn gray_color(gray: f32) -> PdfColor {
    let byte = (gray.clamp(0.0, 1.0) * 255.0).round() as u8;
    PdfColor([byte; 3])
}

fn themed_fill(layout: &Layout, gray: f32) -> Option<PdfColor> {
    let palette = layout.appearance.palette?;
    Some(if gray <= 0.91 {
        palette.surface_strong
    } else if gray <= 0.96 {
        palette.surface
    } else {
        palette.paper
    })
}

fn themed_stroke(layout: &Layout, gray: f32) -> Option<PdfColor> {
    let palette = layout.appearance.palette?;
    Some(if gray <= 0.30 {
        palette.accent
    } else if gray <= 0.68 {
        palette.muted
    } else {
        palette.faint
    })
}

fn fill_rect(canvas: &mut RgbaImage, x: f32, y: f32, w: f32, h: f32, color: PdfColor, scale: f32) {
    let x0 = (x * scale).floor().max(0.0) as u32;
    let y0 = (y * scale).floor().max(0.0) as u32;
    let x1 = ((x + w) * scale).ceil().max(0.0) as u32;
    let y1 = ((y + h) * scale).ceil().max(0.0) as u32;
    for py in y0.min(canvas.height())..y1.min(canvas.height()) {
        for px in x0.min(canvas.width())..x1.min(canvas.width()) {
            canvas.put_pixel(px, py, Rgba([color.0[0], color.0[1], color.0[2], 255]));
        }
    }
}

fn draw_line(
    canvas: &mut RgbaImage,
    from: (f32, f32),
    to: (f32, f32),
    width: f32,
    color: PdfColor,
    scale: f32,
) {
    let (x0, y0) = (from.0 * scale, from.1 * scale);
    let (x1, y1) = (to.0 * scale, to.1 * scale);
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as u32;
    let radius = (width * scale * 0.5).ceil().max(0.5) as i32;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let cx = x0 + (x1 - x0) * t;
        let cy = y0 + (y1 - y0) * t;
        for oy in -radius..=radius {
            for ox in -radius..=radius {
                let px = cx.round() as i32 + ox;
                let py = cy.round() as i32 + oy;
                if px >= 0
                    && py >= 0
                    && (px as u32) < canvas.width()
                    && (py as u32) < canvas.height()
                {
                    canvas.put_pixel(
                        px as u32,
                        py as u32,
                        Rgba([color.0[0], color.0[1], color.0[2], 255]),
                    );
                }
            }
        }
    }
}

fn draw_glyph(
    canvas: &mut RgbaImage,
    fonts: &mut Fonts,
    swash: &mut SwashCache,
    glyph: &super::layout::GlyphOp,
    color: PdfColor,
    scale: f32,
) {
    let asset = fonts.roster.asset(glyph.role);
    let flags = if glyph.italic {
        CacheKeyFlags::FAKE_ITALIC
    } else {
        CacheKeyFlags::empty()
    };
    let (key, pen_x, pen_y) = CacheKey::new(
        fonts.id(glyph.role),
        glyph.glyph_id,
        glyph.size * scale,
        (glyph.x * scale, glyph.y * scale),
        glyphon::fontdb::Weight(asset.weight),
        flags,
    );
    let Some(mask) = swash.get_image(&mut fonts.system, key).as_ref() else {
        return;
    };
    if mask.content != SwashContent::Mask {
        return;
    }
    let x = pen_x + mask.placement.left;
    let y = pen_y - mask.placement.top;
    paint_mask(
        canvas,
        (x, y),
        (mask.placement.width, mask.placement.height),
        &mask.data,
        color,
        glyph.embolden,
    );
}

fn paint_mask(
    canvas: &mut RgbaImage,
    position: (i32, i32),
    size: (u32, u32),
    data: &[u8],
    color: PdfColor,
    embolden: bool,
) {
    let ((x, y), (width, height)) = (position, size);
    let offsets: &[(i32, i32)] = if embolden {
        &[(0, 0), (1, 0)]
    } else {
        &[(0, 0)]
    };
    for &(ox, oy) in offsets {
        for row in 0..height {
            for col in 0..width {
                let alpha = data[(row * width + col) as usize];
                if alpha == 0 {
                    continue;
                }
                let px = x + ox + col as i32;
                let py = y + oy + row as i32;
                if px < 0 || py < 0 || px as u32 >= canvas.width() || py as u32 >= canvas.height() {
                    continue;
                }
                blend(canvas.get_pixel_mut(px as u32, py as u32), color, alpha);
            }
        }
    }
}

fn blend(dst: &mut Rgba<u8>, color: PdfColor, alpha: u8) {
    let a = u16::from(alpha);
    let inv = 255 - a;
    for channel in 0..3 {
        dst.0[channel] =
            ((u16::from(color.0[channel]) * a + u16::from(dst.0[channel]) * inv + 127) / 255) as u8;
    }
    dst.0[3] = 255;
}

fn draw_png(
    canvas: &mut RgbaImage,
    image: &super::images::PdfImage,
    rect: (f32, f32, f32, f32),
    scale: f32,
) {
    let (x, y, w, h) = rect;
    let x0 = (x * scale).floor() as i32;
    let y0 = (y * scale).floor() as i32;
    let x1 = ((x + w) * scale).ceil() as i32;
    let y1 = ((y + h) * scale).ceil() as i32;
    for py in y0.max(0)..y1.min(canvas.height() as i32) {
        for px in x0.max(0)..x1.min(canvas.width() as i32) {
            let u = ((px - x0) as f32 / (x1 - x0).max(1) as f32 * image.width as f32)
                .floor()
                .min((image.width - 1) as f32) as u32;
            let v = ((py - y0) as f32 / (y1 - y0).max(1) as f32 * image.height as f32)
                .floor()
                .min((image.height - 1) as f32) as u32;
            let index = (v * image.width + u) as usize;
            let color = PdfColor([
                image.data[index * 3],
                image.data[index * 3 + 1],
                image.data[index * 3 + 2],
            ]);
            let alpha = image.alpha.as_ref().map_or(255, |alpha| alpha[index]);
            blend(canvas.get_pixel_mut(px as u32, py as u32), color, alpha);
        }
    }
}

fn draw_jpeg_plate(
    canvas: &mut RgbaImage,
    fonts: &mut Fonts,
    swash: &mut SwashCache,
    rect: (f32, f32, f32, f32),
    layout: &Layout,
    scale: f32,
) {
    let (x, y, w, h) = rect;
    let (plate, ink) = layout.appearance.palette.map_or(
        (PdfColor([238, 238, 238]), PdfColor([55, 55, 55])),
        |palette| (palette.surface, palette.ink),
    );
    fill_rect(canvas, x, y, w, h, plate, scale);
    draw_line(canvas, (x, y), (x + w, y), 0.75, ink, scale);
    draw_line(canvas, (x, y + h), (x + w, y + h), 0.75, ink, scale);
    draw_line(canvas, (x, y), (x, y + h), 0.75, ink, scale);
    draw_line(canvas, (x + w, y), (x + w, y + h), 0.75, ink, scale);
    draw_preview_label(
        canvas,
        fonts,
        swash,
        "JPEG IMAGE - INCLUDED IN PDF",
        (x + 6.0, y + (h - 11.0) * 0.5, (w - 12.0).max(1.0)),
        ink,
        scale,
    );
}

fn draw_preview_label(
    canvas: &mut RgbaImage,
    fonts: &mut Fonts,
    swash: &mut SwashCache,
    text: &str,
    rect: (f32, f32, f32),
    color: PdfColor,
    scale: f32,
) {
    let asset = fonts.roster.asset(FontRole::Serif);
    let attrs = Attrs::new()
        .family(Family::Name(asset.family))
        .weight(Weight(asset.weight));
    let mut buffer = Buffer::new(&mut fonts.system, Metrics::new(8.0, 11.0));
    buffer.set_size(&mut fonts.system, Some(rect.2), Some(11.0));
    buffer.set_wrap(&mut fonts.system, Wrap::None);
    buffer.set_text(&mut fonts.system, text, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut fonts.system, false);
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let physical = glyph.physical((rect.0 * scale, rect.1 * scale), scale);
            let Some(mask) = swash
                .get_image(&mut fonts.system, physical.cache_key)
                .as_ref()
            else {
                continue;
            };
            if mask.content != SwashContent::Mask {
                continue;
            }
            paint_mask(
                canvas,
                (
                    physical.x + mask.placement.left,
                    physical.y - mask.placement.top,
                ),
                (mask.placement.width, mask.placement.height),
                &mask.data,
                color,
                false,
            );
        }
    }
}
