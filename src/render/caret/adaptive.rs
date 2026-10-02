//! Resting block bounds come from the resolved shaped grapheme's complete ink.
use super::*;
use unicode_segmentation::UnicodeSegmentation;

pub(in crate::render) type ShapedCaretGlyph = (usize, usize, CacheKey, i32, i32);

struct RasterCoverage {
    key: CacheKey,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl TextPipeline {
    pub(in crate::render) fn previous_grapheme_col(&self) -> usize {
        let row_start = self.caret_row_start_col();
        if self.cursor_col <= row_start {
            return self.cursor_col;
        }
        let Some(line) = self.buffer.lines.get(self.cursor_line) else {
            return self.cursor_col;
        };
        let text = line.text();
        let cursor_byte = text
            .char_indices()
            .nth(self.cursor_col)
            .map_or(text.len(), |(i, _)| i);
        text.grapheme_indices(true)
            .take_while(|(i, _)| *i < cursor_byte)
            .last()
            .map_or(self.cursor_col, |(byte, _)| {
                text[..byte].chars().count().max(row_start)
            })
    }

    pub(in crate::render) fn adaptive_anchor_glyphs(
        &mut self,
        include_ligature: bool,
    ) -> Option<Vec<ShapedCaretGlyph>> {
        let line = self.cursor_line;
        let col = self.caret_anchor_col();
        let text = self.buffer.lines.get(line)?.text().to_string();
        let byte = text.char_indices().nth(col).map_or(text.len(), |(i, _)| i);
        let (start, grapheme) = text
            .grapheme_indices(true)
            .find(|(i, g)| byte >= *i && byte < i + g.len())?;
        let end = start + grapheme.len();
        self.ensure_caret_line_glyphs(line);
        let glyphs: Vec<_> = self
            .caret_line_glyphs
            .borrow()
            .as_ref()?
            .clusters
            .iter()
            .filter(|(s, e, ..)| *s < end && *e > start)
            .copied()
            .collect();
        // One ligature spanning independent graphemes has no honest single-letter ink box.
        if !include_ligature && glyphs.iter().any(|(s, e, ..)| *s < start || *e > end) {
            return None;
        }
        Some(glyphs)
    }

    pub(in crate::render) fn adaptive_anchor_ink_box(&mut self) -> Option<InkBox> {
        let glyphs = self.adaptive_anchor_glyphs(false)?;
        let (pen_x, _) = self.col_x_and_advance_aff(
            self.cursor_line,
            self.caret_anchor_col(),
            self.caret_affinity,
        );
        shaped_ink_box(&glyphs, pen_x, &mut self.swash_cache, &mut self.font_system)
    }
    /// Filled blocks knock out every glyph of the anchored grapheme, including
    /// separately positioned combining marks. The signature avoids GPU uploads
    /// while the same shaped grapheme remains under the caret.
    pub(in crate::render) fn prepare_adaptive_knockout(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> bool {
        let Some(glyphs) = self.adaptive_anchor_glyphs(true) else {
            self.caret_mask_to = None;
            self.caret_mask_glyphs.clear();
            return false;
        };
        let (pen_x, _) = self.col_x_and_advance_aff(
            self.cursor_line,
            self.caret_anchor_col(),
            self.caret_affinity,
        );
        let signature: Vec<_> = glyphs
            .iter()
            .map(|(_, _, key, x, y)| (*key, *x - pen_x.round() as i32, *y))
            .collect();
        self.caret_mask_from = None;
        if signature == self.caret_mask_glyphs {
            return self
                .caret_mask_to
                .as_ref()
                .is_some_and(|mask| Some(mask.key) == signature.first().map(|g| g.0));
        }
        self.caret_mask_glyphs = signature.clone();
        self.caret_mask_to = None;
        let clip = if self.adaptive_anchor_ink_box().is_none() {
            let (cx, cy, w, h, corner, ..) = self.caret_geometry();
            Some((
                cx - pen_x - self.text_left(),
                cy - self.caret_baseline_y(),
                w * 0.5,
                h * 0.5,
                corner,
            ))
        } else {
            None
        };
        let mut images = Vec::new();
        for &(key, x, y) in &signature {
            let Some(img) = self
                .swash_cache
                .get_image(&mut self.font_system, key)
                .as_ref()
            else {
                continue;
            };
            if img.content != SwashContent::Mask
                || img.placement.width == 0
                || img.placement.height == 0
            {
                continue;
            }
            images.push(RasterCoverage {
                key,
                x: x + img.placement.left,
                y: y - img.placement.top,
                width: img.placement.width,
                height: img.placement.height,
                data: img.data.clone(),
            });
        }
        let Some(mask) = composite_coverage(images, clip) else {
            return false;
        };
        self.caret_mask_to = Some(GlyphMask::from_coverage(
            device,
            queue,
            mask.key,
            mask.x,
            -mask.y,
            mask.width,
            mask.height,
            &mask.data,
        ));
        true
    }
}

/// Shared raster bounds for document text and the picker sample.
pub(in crate::render) fn shaped_ink_box(
    glyphs: &[ShapedCaretGlyph],
    pen_x: f32,
    swash_cache: &mut SwashCache,
    font_system: &mut FontSystem,
) -> Option<InkBox> {
    let mut bounds: Option<(f32, f32, f32, f32)> = None;
    for &(_, _, key, x, y) in glyphs {
        let image = swash_cache.get_image(font_system, key).as_ref()?;
        let p = &image.placement;
        if p.width == 0 || p.height == 0 {
            continue;
        }
        let left = x as f32 + p.left as f32 - pen_x;
        let top = y as f32 - p.top as f32;
        let right = left + p.width as f32;
        let bottom = top + p.height as f32;
        bounds = Some(match bounds {
            Some((l, t, r, b)) => (l.min(left), t.min(top), r.max(right), b.max(bottom)),
            None => (left, top, right, bottom),
        });
    }
    bounds.map(|(l, t, r, b)| InkBox {
        left: l,
        top: -t,
        width: r - l,
        height: b - t,
    })
}

/// Alpha-union positioned source rasters; clipped ligatures keep neighboring ink.
fn composite_coverage(
    images: Vec<RasterCoverage>,
    clip: Option<(f32, f32, f32, f32, f32)>,
) -> Option<RasterCoverage> {
    let key = images.first()?.key;
    let left = images.iter().map(|g| g.x).min()?;
    let top = images.iter().map(|g| g.y).min()?;
    let right = images.iter().map(|g| g.x + g.width as i32).max()?;
    let bottom = images.iter().map(|g| g.y + g.height as i32).max()?;
    let width = (right - left) as u32;
    let height = (bottom - top) as u32;
    let mut data = vec![0u8; (width * height) as usize];
    for glyph in images {
        for iy in 0..glyph.height {
            for ix in 0..glyph.width {
                let dst = ((glyph.y - top) as u32 + iy) * width + (glyph.x - left) as u32 + ix;
                let mut alpha = glyph.data[(iy * glyph.width + ix) as usize] as u32;
                if let Some((cx, cy, hw, hh, corner)) = clip {
                    let dx = (glyph.x as f32 + ix as f32 + 0.5 - cx).abs() - hw + corner;
                    let dy = (glyph.y as f32 + iy as f32 + 0.5 - cy).abs() - hh + corner;
                    let distance = dx.max(0.0).hypot(dy.max(0.0)) + dx.max(dy).min(0.0) - corner;
                    alpha = (alpha as f32 * (0.5 - distance).clamp(0.0, 1.0)).round() as u32;
                }
                let old = data[dst as usize] as u32;
                data[dst as usize] = (255 - ((255 - old) * (255 - alpha) + 127) / 255) as u8;
            }
        }
    }
    Some(RasterCoverage {
        key,
        x: left,
        y: top,
        width,
        height,
        data,
    })
}
