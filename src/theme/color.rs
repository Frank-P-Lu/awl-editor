//! src/theme/color.rs — the [`Srgb`] color primitive, the authoritative
//! representation every other theme file + render call site builds on.

/// An sRGB color stored as raw 8-bit channels. This is the authoritative
/// representation; converter methods project it into whatever the GPU /
/// glyphon call site wants. Storing bytes (not floats) keeps every existing
/// output byte-identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Srgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// One 8-bit sRGB channel as LINEAR light — the sRGB EOTF, IEC 61966-2-1.
///
/// The same transfer function the shader-side converters apply
/// (`selection::srgba_u8_to_linear` and its siblings in `background`, `lava`,
/// `caret` and `render`); agreement with one of them is pinned by this module's
/// tests, since a second definition of a transfer function is a second answer
/// waiting to disagree.
pub(crate) fn srgb_channel_to_linear(c: u8) -> f64 {
    let s = c as f64 / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

impl Srgb {
    /// Opaque sRGB color (alpha = 0xFF).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }
    /// sRGB color with explicit alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// wgpu CLEAR color — this token DECODED TO LINEAR LIGHT.
    ///
    /// `LoadOp::Clear`'s value skips the fragment stage, so an sRGB-format
    /// attachment never applies the linear→sRGB encode to it and instead
    /// consumes it as linear light already. Hand such a target raw sRGB bytes
    /// and the surface stores their sRGB *ENCODE*: a dark ground draws far
    /// lighter than it is authored, and the error grows toward black where the
    /// curve is steepest. Decoding here is what makes the drawn pixel equal the
    /// authored token.
    ///
    /// Every awl render target is an sRGB view — the capture texture, native's
    /// `Bgra8UnormSrgb` surface, and the web canvas's sRGB view over a non-sRGB
    /// config format (`app::gpu`) — so there is no call site that wants the raw
    /// bytes, and the name says CLEAR because that is the only thing this
    /// semantic is right for. Alpha is not gamma-encoded and passes straight
    /// through. `f64` because `wgpu::Color` is.
    pub fn to_wgpu_clear(self) -> wgpu::Color {
        wgpu::Color {
            r: srgb_channel_to_linear(self.r),
            g: srgb_channel_to_linear(self.g),
            b: srgb_channel_to_linear(self.b),
            a: self.a as f64 / 255.0,
        }
    }
    /// glyphon text color (drops alpha; glyphon::Color::rgb is opaque, matching
    /// the old FG which was Color::rgb).
    pub fn to_glyphon(self) -> glyphon::Color {
        glyphon::Color::rgb(self.r, self.g, self.b)
    }
    /// Raw sRGB bytes for the caret pipeline (which converts to linear itself).
    pub fn rgb_bytes(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
    /// Raw sRGBA bytes for the selection / spell pipelines (which convert to
    /// linear themselves).
    pub fn rgba_bytes(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
    /// Lowercase 6-digit `#rrggbb` hex (alpha dropped). Used by the sidecar.
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// This color's `(hue°, saturation, lightness)` in HSL space — hue in
    /// `[0, 360)`, saturation + lightness in `[0, 1]`. Alpha is ignored. f32 math
    /// INTERNALLY only: the u8 channels stay the authoritative store (this pair of
    /// converters exists for the syntax ROLE-STYLE derivation in `render/spans.rs`,
    /// which mixes each world's own ink lightness with a fixed role hue anchor).
    pub fn to_hsl(self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) * 0.5;
        let d = max - min;
        if d <= f32::EPSILON {
            return (0.0, 0.0, l); // achromatic: hue is undefined, report 0
        }
        let s = d / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == r {
            60.0 * ((g - b) / d).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        (h, s, l)
    }

    /// Linear per-channel blend toward `other` by `t` (`0.0` = self, `1.0` =
    /// other; clamped). Alpha blends too. The one arithmetic primitive the
    /// PLACARD-INK derivation (`theme::derive::placard_ink`) uses to step a
    /// rung below [`Theme::faint`] WITHOUT inventing a free color — the result
    /// is always a mix of two tokens already on the world's own ink ladder.
    pub fn lerp(self, other: Srgb, t: f32) -> Srgb {
        let t = t.clamp(0.0, 1.0);
        let ch = |a: u8, b: u8| {
            (a as f32 + (b as f32 - a as f32) * t)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        Srgb::rgba(
            ch(self.r, other.r),
            ch(self.g, other.g),
            ch(self.b, other.b),
            ch(self.a, other.a),
        )
    }

    /// An OPAQUE [`Srgb`] from `(hue°, saturation, lightness)` — the inverse of
    /// [`Srgb::to_hsl`] (up to u8 rounding, which stays authoritative). Hue wraps;
    /// saturation / lightness clamp to `[0, 1]`.
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let hp = h / 60.0;
        let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
        let (r1, g1, b1) = match hp as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = l - c * 0.5;
        let to = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        Srgb::rgb(to(r1), to(g1), to(b1))
    }
}
