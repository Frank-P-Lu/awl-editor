use super::Srgb;
use super::model::{Background, LavaEdge};

impl Background {
    pub fn shader_id(&self) -> u32 {
        match self {
            Self::Gradient { .. } => 0,
            Self::Dots { .. } => 1,
            Self::Starfield { .. } => 2,
            Self::Pinstripe { .. } => 3,
            Self::Stripes { .. } => 4,
            Self::Lava { .. } => 0,
            Self::Bands { .. } => 5,
            Self::Waves { .. } => 6,
            Self::Zigzag { .. } => 7,
            Self::Organic { .. } => 8,
            Self::WarpedGrid { .. } => 9,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gradient { .. } => "gradient",
            Self::Dots { .. } => "dots",
            Self::Starfield { .. } => "starfield",
            Self::Pinstripe { .. } => "pinstripe",
            Self::Stripes { .. } => "stripes",
            Self::Lava { .. } => "lava",
            Self::Bands { .. } => "bands",
            Self::Waves { .. } => "waves",
            Self::Zigzag { .. } => "zigzag",
            Self::Organic { .. } => "organic",
            Self::WarpedGrid { .. } => "warped-grid",
        }
    }

    pub fn from(&self) -> Srgb {
        match self {
            Self::Gradient { from, .. }
            | Self::Dots { from, .. }
            | Self::Starfield { from, .. }
            | Self::Pinstripe { from, .. }
            | Self::Stripes { from, .. }
            | Self::Zigzag { from, .. } => *from,
            Self::Lava { ground, .. } => *ground,
            Self::Bands { tones, .. }
            | Self::Waves { tones }
            | Self::Organic { tones, .. }
            | Self::WarpedGrid { tones, .. } => tones[0],
        }
    }

    pub fn to(&self) -> Srgb {
        match self {
            Self::Gradient { to, .. }
            | Self::Dots { to, .. }
            | Self::Starfield { to, .. }
            | Self::Pinstripe { to, .. }
            | Self::Stripes { to, .. }
            | Self::Zigzag { to, .. } => *to,
            Self::Lava { ground, .. } => *ground,
            Self::Bands { tones, .. }
            | Self::Waves { tones }
            | Self::Organic { tones, .. }
            | Self::WarpedGrid { tones, .. } => tones[2],
        }
    }

    pub fn dir(&self) -> (f32, f32) {
        match self {
            Self::Gradient { dir, .. }
            | Self::Dots { dir, .. }
            | Self::Starfield { dir, .. }
            | Self::Pinstripe { dir, .. }
            | Self::Zigzag { dir, .. } => *dir,
            Self::Stripes { angle, .. } | Self::Bands { angle, .. } => (angle.cos(), angle.sin()),
            Self::Lava { .. }
            | Self::Waves { .. }
            | Self::Organic { .. }
            | Self::WarpedGrid { .. } => (0.0, 1.0),
        }
    }

    pub fn tint(&self) -> Srgb {
        match self {
            Self::Dots { tint, .. }
            | Self::Starfield { tint, .. }
            | Self::Pinstripe { tint, .. }
            | Self::Zigzag { tint, .. } => *tint,
            Self::Stripes { band, .. } => *band,
            Self::Gradient { from, .. } => *from,
            Self::Lava { ground, .. } => *ground,
            Self::Bands { tones, .. }
            | Self::Waves { tones }
            | Self::Organic { tones, .. }
            | Self::WarpedGrid { tones, .. } => tones[1],
        }
    }

    pub fn edge(&self) -> bool {
        matches!(self, Self::Dots { edge: true, .. })
    }

    pub fn angle(&self) -> f32 {
        match self {
            Self::Stripes { angle, .. }
            | Self::Bands { angle, .. }
            | Self::Zigzag { angle, .. } => *angle,
            _ => 0.0,
        }
    }

    pub fn period_px(&self) -> f32 {
        match self {
            Self::Zigzag { period_px, .. } => *period_px,
            Self::Organic { scale_px, .. } => *scale_px,
            Self::WarpedGrid { spacing_px, .. } => *spacing_px,
            _ => 0.0,
        }
    }

    pub fn amplitude_px(&self) -> f32 {
        match self {
            Self::Zigzag { amplitude_px, .. } => *amplitude_px,
            Self::WarpedGrid { curvature, .. } => *curvature,
            _ => 0.0,
        }
    }

    #[cfg(test)]
    pub fn zigzag_stroke_px(&self) -> f32 {
        use super::model::{ZIGZAG_MIN_STROKE_PX, ZIGZAG_STROKE_FRAC};
        (self.amplitude_px() * ZIGZAG_STROKE_FRAC).max(ZIGZAG_MIN_STROKE_PX)
    }

    #[cfg(test)]
    pub fn zigzag_row_pitch_px(&self) -> f32 {
        2.0 * self.amplitude_px() + self.zigzag_stroke_px()
    }

    pub fn density(&self) -> f32 {
        match self {
            Self::Zigzag { density, .. }
            | Self::Organic { density, .. }
            | Self::WarpedGrid { density, .. } => *density,
            _ => 0.0,
        }
    }

    pub fn zigzag_banded(&self) -> bool {
        matches!(self, Self::Zigzag { banded: true, .. })
    }

    pub fn is_lava(&self) -> bool {
        matches!(self, Self::Lava { .. })
    }

    pub fn is_waves(&self) -> bool {
        matches!(self, Self::Waves { .. })
    }

    pub fn is_organic(&self) -> bool {
        matches!(self, Self::Organic { .. })
    }

    pub fn is_warped_grid(&self) -> bool {
        matches!(self, Self::WarpedGrid { .. })
    }

    pub fn lava_params(&self) -> Option<(Srgb, Srgb, Srgb, LavaEdge, bool)> {
        match self {
            Self::Lava {
                ground,
                blob_lo,
                blob_hi,
                edge,
                dithered,
            } => Some((*ground, *blob_lo, *blob_hi, *edge, *dithered)),
            _ => None,
        }
    }
}
