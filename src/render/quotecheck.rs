//! CURLY-QUOTE ORIENTATION — a GPU-free geometric check that a bundled display
//! face's OPENING curly quotes (`'` U+2018, `"` U+201C) actually draw the
//! rotated "6" shape (ink concentrated toward the BOTTOM of the glyph's own
//! bounding box) rather than the raised-comma "9" shape a CLOSING quote draws
//! (ink toward the TOP). Sour Gummy shipped both raised-quote pairs with
//! their outlines TRANSPOSED — `cmap[U+2018]`/`cmap[U+201C]` pointed at the
//! glyph that draws the closing 9-shape, and vice versa — identically across
//! Regular/Bold/Black, so Quokka's blockquote pull-quote mark and every
//! apostrophe in Quokka prose read backwards. `cmap` was the only thing wrong;
//! the outlines themselves were fine, just filed under the other glyph's name.
//!
//! THE MEASUREMENT. The original diagnosis rasterised each face's `U+201C`
//! and compared ink in the top vs bottom QUARTER of the glyph's bounding box
//! (24 of 25 bundled Regular-weight faces came out heavy-bottom; Sour Gummy
//! alone was heavy-top). This module answers the same question geometrically
//! instead of by rendering pixels: it reads the glyph's own outline through
//! **skrifa** (the same font stack `facepitch` measures advances with),
//! flattens every curve to a dense polyline, and computes the exact polygon
//! area inside the top and bottom quarter-height bands via Sutherland-Hodgman
//! clipping — no rasteriser, no antialiasing noise, and it runs on any box.
//! `is_heavy_bottom` cross-checked against a real PIL raster of the same 15
//! faces during the fix: the two methods agree, and the correct-face margin
//! (smallest observed: Zilla Slab's `quotedblleft`, bottom quarter beating top
//! by ~1%) sits nowhere near Sour Gummy's transposed ~20% deficit the other
//! way, so this is not a coin-flip threshold.
//!
//! THE ROSTER. `render::tests::quote_orientation_item253` sweeps
//! [`crate::render::bundled_display_faces`] — the SAME "every face a
//! `Theme::font` can name" roster `facepitch`'s own laws sweep — never a
//! hand-kept list, so the next face that ships a font-file bug like this one
//! fails on arrival instead of waiting for a screenshot to notice.

use glyphon::cosmic_text::skrifa::{
    FontRef, MetadataProvider,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
};

/// Straight-line samples per curve segment when flattening a glyph outline
/// for area measurement. Fixed and generous — a one-shot geometry check over
/// a handful of glyphs per face, not a per-frame budget; no adaptive flatness
/// logic is worth the complexity at this scale.
const CURVE_SAMPLES: usize = 24;

/// Collects a glyph outline as flattened polygon contours, in font design
/// units (`Size::unscaled()` — no ppem, so equality/comparisons carry no
/// per-instance scale to argue about).
#[derive(Default)]
struct ContourPen {
    contours: Vec<Vec<(f32, f32)>>,
    current: Vec<(f32, f32)>,
}

impl ContourPen {
    fn finish_current(&mut self) {
        if !self.current.is_empty() {
            self.contours.push(std::mem::take(&mut self.current));
        }
    }

    fn last(&self, fallback: (f32, f32)) -> (f32, f32) {
        *self.current.last().unwrap_or(&fallback)
    }
}

impl OutlinePen for ContourPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish_current();
        self.current.push((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.current.push((x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let p0 = self.last((cx0, cy0));
        for i in 1..=CURVE_SAMPLES {
            let t = i as f32 / CURVE_SAMPLES as f32;
            let mt = 1.0 - t;
            let px = mt * mt * p0.0 + 2.0 * mt * t * cx0 + t * t * x;
            let py = mt * mt * p0.1 + 2.0 * mt * t * cy0 + t * t * y;
            self.current.push((px, py));
        }
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let p0 = self.last((cx0, cy0));
        for i in 1..=CURVE_SAMPLES {
            let t = i as f32 / CURVE_SAMPLES as f32;
            let mt = 1.0 - t;
            let px = mt * mt * mt * p0.0
                + 3.0 * mt * mt * t * cx0
                + 3.0 * mt * t * t * cx1
                + t * t * t * x;
            let py = mt * mt * mt * p0.1
                + 3.0 * mt * mt * t * cy0
                + 3.0 * mt * t * t * cy1
                + t * t * t * y;
            self.current.push((px, py));
        }
    }

    fn close(&mut self) {
        self.finish_current();
    }
}

/// The flattened contours of the glyph `ch` maps to in `bytes`, or `None`
/// when the face has no `cmap` entry for it, the outline fails to draw, or it
/// draws nothing (a glyphless codepoint — never treated as a verdict either
/// way, only as "this axis does not apply here").
fn glyph_contours(bytes: &[u8], ch: char) -> Option<Vec<Vec<(f32, f32)>>> {
    let font = FontRef::new(bytes).ok()?;
    let gid = font.charmap().map(ch)?;
    let outline = font.outline_glyphs().get(gid)?;
    let settings = DrawSettings::unhinted(Size::unscaled(), LocationRef::default());
    let mut pen = ContourPen::default();
    outline.draw(settings, &mut pen).ok()?;
    pen.finish_current();
    if pen.contours.is_empty() {
        None
    } else {
        Some(pen.contours)
    }
}

/// Signed polygon area (shoelace), matching the outline's own winding — a
/// hole contour (opposite winding to its outer contour) contributes a
/// negative area, so summing every contour of a multi-contour glyph yields
/// the NET filled area, holes correctly subtracted.
fn signed_area(poly: &[(f32, f32)]) -> f32 {
    if poly.len() < 3 {
        return 0.0;
    }
    let mut a = 0.0;
    for i in 0..poly.len() {
        let (x0, y0) = poly[i];
        let (x1, y1) = poly[(i + 1) % poly.len()];
        a += x0 * y1 - x1 * y0;
    }
    a * 0.5
}

/// Sutherland-Hodgman clip of a (possibly non-convex) polygon against the
/// horizontal band `lo..=hi`, keeping the portion inside. The clip is applied
/// as two half-plane passes (`y >= lo` then `y <= hi`); each preserves vertex
/// order, so the shoelace formula on the result still reads a consistent
/// winding.
fn clip_to_band(poly: &[(f32, f32)], lo: f32, hi: f32) -> Vec<(f32, f32)> {
    fn half_plane(
        poly: &[(f32, f32)],
        keep: impl Fn(f32) -> bool,
        boundary: f32,
    ) -> Vec<(f32, f32)> {
        let mut out = Vec::with_capacity(poly.len());
        let n = poly.len();
        if n == 0 {
            return out;
        }
        for i in 0..n {
            let cur = poly[i];
            let next = poly[(i + 1) % n];
            let cur_in = keep(cur.1);
            let next_in = keep(next.1);
            let intersect = || {
                let t = (boundary - cur.1) / (next.1 - cur.1);
                (cur.0 + t * (next.0 - cur.0), boundary)
            };
            if cur_in {
                out.push(cur);
                if !next_in {
                    out.push(intersect());
                }
            } else if next_in {
                out.push(intersect());
            }
        }
        out
    }
    let above_lo = half_plane(poly, |y| y >= lo, lo);
    half_plane(&above_lo, |y| y <= hi, hi)
}

/// The exact polygon area of `contours` lying within the horizontal band
/// `lo..=hi`, in font design units. Signed areas of every contour's clipped
/// piece are summed BEFORE taking the absolute value, so holes stay
/// subtracted through the clip exactly as they were in the unclipped glyph.
fn band_area(contours: &[Vec<(f32, f32)>], lo: f32, hi: f32) -> f32 {
    let total: f32 = contours
        .iter()
        .map(|c| signed_area(&clip_to_band(c, lo, hi)))
        .sum();
    total.abs()
}

/// The top-quarter and bottom-quarter ink area of the glyph `ch` maps to in
/// `bytes` — the geometric twin of the item's own pixel roster raster. `None`
/// when the face has no glyph for `ch` (never a false verdict).
pub fn glyph_quarter_band_areas(bytes: &[u8], ch: char) -> Option<(f32, f32)> {
    let contours = glyph_contours(bytes, ch)?;
    let ys = contours.iter().flatten().map(|(_, y)| *y);
    let (mut ymin, mut ymax) = (f32::INFINITY, f32::NEG_INFINITY);
    for y in ys {
        ymin = ymin.min(y);
        ymax = ymax.max(y);
    }
    if ymax <= ymin {
        return None;
    }
    let q = (ymax - ymin) * 0.25;
    let top = band_area(&contours, ymax - q, ymax);
    let bottom = band_area(&contours, ymin, ymin + q);
    Some((top, bottom))
}

/// `true` when `ch`'s glyph in `bytes` draws heavier in the BOTTOM quarter of
/// its own bounding box than the top — the rotated "6" shape a correctly
/// mapped OPENING curly quote draws. `false` is the raised-comma "9" shape a
/// CLOSING quote (or a mis-mapped opening one, sharing the closing glyph's
/// outline) draws. `None` when the face has no glyph for `ch`.
pub fn is_heavy_bottom(bytes: &[u8], ch: char) -> Option<bool> {
    glyph_quarter_band_areas(bytes, ch).map(|(top, bottom)| bottom > top)
}
