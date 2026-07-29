//! The MARGIN GROUND data model — `Background` and its dials.
//!
//! Carved out of `theme/model.rs` when item 158's Deckle family pushed that
//! file past its own size mark: the ground is a self-contained vocabulary (one
//! enum, its two dial enums, the shader mirrors the laws read) with exactly one
//! consumer shape — `render::background_desc` flattens it into a `BgDesc`, and
//! the WGSL in `shaders/background.wgsl` draws whatever `shader_id` names.
//! Nothing here knows a world's name; a world picks a ground in its own literal.

use super::color::Srgb;

#[cfg(test)]
pub const ZIGZAG_STROKE_FRAC: f32 = 0.10;
#[cfg(test)]
pub const ZIGZAG_MIN_STROKE_PX: f32 = 1.2;
#[cfg(test)]
pub const ZIGZAG_MAX_ROW_PITCH_PX: f32 = 160.0;

// DECKLE's shader mirrors (item 158). `cfg(test)` for the same reason the
// ZIGZAG mirrors are: the GPU is the only runtime consumer, and the host reads
// these ONLY to state the field's laws. A grep-law holds them in lockstep with
// `shaders/background.wgsl`'s own copies.
//
// The two PERIOD bounds are AUTHORED guards on every future assignee, and each
// names a real failure:
//   * below `DECKLE_MIN_PERIOD_PX` the deckle edge — a FRACTION of a lane —
//     falls under a pixel and the field aliases into moire. The shader ALSO
//     clamps to this floor, so coverage is a property of the shader rather
//     than of a dial pair (item 89's abutment lesson).
//   * above `DECKLE_MAX_PERIOD_PX` a real page margin cannot hold one whole
//     lane, so it renders as a single flat tone — the "collapses toward a
//     quiet gradient" failure Paperbark's own brief forbids.
#[cfg(test)]
pub const DECKLE_MIN_PERIOD_PX: f32 = 40.0;
#[cfg(test)]
pub const DECKLE_MAX_PERIOD_PX: f32 = 160.0;
/// The lane value `density == 0.0` flattens every Deckle world to — the anchor
/// that makes `density: 0.0` an EXACT flat differential reference.
#[cfg(test)]
pub const DECKLE_MID: f32 = 0.46;
/// The lane-value half-range per unit `density`.
#[cfg(test)]
pub const DECKLE_SPREAD_GAIN: f32 = 1.2;

#[derive(Clone, Copy, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Background {
    Gradient { from: Srgb, to: Srgb, dir: (f32, f32) },
    Dots { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb, edge: bool },
    Starfield { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb },
    Pinstripe { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb },
    Stripes { from: Srgb, to: Srgb, band: Srgb, angle: f32 },
    Lava {
        ground: Srgb,
        blob_lo: Srgb,
        blob_hi: Srgb,
        edge: LavaEdge,
        dithered: bool,
    },
    Bands { tones: [Srgb; 3], angle: f32 },
    Waves { tones: [Srgb; 3] },
    Zigzag { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb,
        period_px: f32, amplitude_px: f32, angle: f32, density: f32, banded: bool },
    Organic { tones: [Srgb; 3], scale_px: f32, density: f32 },
    Deckle { ground: Srgb, layer: Srgb, deckle: Srgb, weave: Weave,
        period_px: f32, wander_px: f32, density: f32 },
}

/// DECKLE's one theme-owned profile dial (item 158). The handmade-paper field
/// draws quasi-random contour lanes; `Weave` picks WHICH lanes, and nothing
/// else in the renderer ever asks which world is active — a second world adopts
/// a profile by writing this word in its own literal, exactly as `LavaEdge`
/// and `CardTexture` are chosen.
///
/// * [`Weave::Strata`] — lanes indexed on DISTANCE FROM THE PAGE COLUMN, so the
///   contours gather around the writing page and mirror across it; each lane is
///   filled at its own seeded value and its boundary carries a torn deckle
///   tint. Paperbark's material.
/// * [`Weave::Fibres`] — lanes indexed on screen `y`, drawn as thin translucent
///   strokes with seeded dropouts plus a sparser diagonal vein family. Reusable
///   infrastructure, currently carried by no world (the `Bands` /
///   `Dots { edge: true }` "ships until one wants it" shape).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weave {
    Strata,
    Fibres,
}

impl Weave {
    /// The scalar the WGSL `deckle_rgb` branches on (`params.w`). MUST match
    /// `shaders/background.wgsl`'s own `DECKLE_WEAVE_FIBRES` threshold.
    pub fn mode(self) -> f32 {
        match self {
            Weave::Strata => 0.0,
            Weave::Fibres => 1.0,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Weave::Strata => "strata",
            Weave::Fibres => "fibres",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LavaEdge {
    Hard,
    Glow,
}

impl LavaEdge {
    pub fn mask_mode(self) -> f32 {
        match self {
            LavaEdge::Hard => 1.0,
            LavaEdge::Glow => 2.0,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            LavaEdge::Hard => "hard",
            LavaEdge::Glow => "glow",
        }
    }
}

impl Background {
    pub fn shader_id(&self) -> u32 {
        match self {
            Background::Gradient { .. } => 0,
            Background::Dots { .. } => 1,
            Background::Starfield { .. } => 2,
            Background::Pinstripe { .. } => 3,
            Background::Stripes { .. } => 4,
            Background::Lava { .. } => 0,
            Background::Bands { .. } => 5,
            Background::Waves { .. } => 6,
            Background::Zigzag { .. } => 7,
            Background::Organic { .. } => 8,
            Background::Deckle { .. } => 9,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Background::Gradient { .. } => "gradient",
            Background::Dots { .. } => "dots",
            Background::Starfield { .. } => "starfield",
            Background::Pinstripe { .. } => "pinstripe",
            Background::Stripes { .. } => "stripes",
            Background::Lava { .. } => "lava",
            Background::Bands { .. } => "bands",
            Background::Waves { .. } => "waves",
            Background::Zigzag { .. } => "zigzag",
            Background::Organic { .. } => "organic",
            Background::Deckle { .. } => "deckle",
        }
    }
    pub fn from(&self) -> Srgb {
        match self {
            Background::Gradient { from, .. }
            | Background::Dots { from, .. }
            | Background::Starfield { from, .. }
            | Background::Pinstripe { from, .. }
            | Background::Stripes { from, .. }
            | Background::Zigzag { from, .. } => *from,
            Background::Lava { ground, .. } | Background::Deckle { ground, .. } => *ground,
            Background::Bands { tones, .. }
            | Background::Waves { tones }
            | Background::Organic { tones, .. } => tones[0],
        }
    }
    pub fn to(&self) -> Srgb {
        match self {
            Background::Gradient { to, .. }
            | Background::Dots { to, .. }
            | Background::Starfield { to, .. }
            | Background::Pinstripe { to, .. }
            | Background::Stripes { to, .. }
            | Background::Zigzag { to, .. } => *to,
            Background::Lava { ground, .. } => *ground,
            Background::Deckle { layer, .. } => *layer,
            Background::Bands { tones, .. }
            | Background::Waves { tones }
            | Background::Organic { tones, .. } => tones[2],
        }
    }
    pub fn dir(&self) -> (f32, f32) {
        match self {
            Background::Gradient { dir, .. }
            | Background::Dots { dir, .. }
            | Background::Starfield { dir, .. }
            | Background::Pinstripe { dir, .. }
            | Background::Zigzag { dir, .. } => *dir,
            Background::Stripes { angle, .. } | Background::Bands { angle, .. } => {
                (angle.cos(), angle.sin())
            }
            Background::Lava { .. }
            | Background::Waves { .. }
            | Background::Organic { .. }
            | Background::Deckle { .. } => (0.0, 1.0),
        }
    }
    pub fn tint(&self) -> Srgb {
        match self {
            Background::Dots { tint, .. }
            | Background::Starfield { tint, .. }
            | Background::Pinstripe { tint, .. }
            | Background::Zigzag { tint, .. } => *tint,
            Background::Stripes { band, .. } => *band,
            Background::Gradient { from, .. } => *from,
            Background::Lava { ground, .. } => *ground,
            Background::Deckle { deckle, .. } => *deckle,
            Background::Bands { tones, .. }
            | Background::Waves { tones }
            | Background::Organic { tones, .. } => tones[1],
        }
    }
    pub fn edge(&self) -> bool {
        matches!(self, Background::Dots { edge: true, .. })
    }
    pub fn angle(&self) -> f32 {
        match self {
            Background::Stripes { angle, .. }
            | Background::Bands { angle, .. }
            | Background::Zigzag { angle, .. } => *angle,
            _ => 0.0,
        }
    }
    pub fn period_px(&self) -> f32 {
        match self {
            Background::Zigzag { period_px, .. } => *period_px,
            Background::Organic { scale_px, .. } => *scale_px,
            Background::Deckle { period_px, .. } => *period_px,
            _ => 0.0,
        }
    }
    pub fn amplitude_px(&self) -> f32 {
        match self {
            Background::Zigzag { amplitude_px, .. } => *amplitude_px,
            Background::Deckle { wander_px, .. } => *wander_px,
            _ => 0.0,
        }
    }
    /// DECKLE's weave, as the scalar the shader branches on — `0.0` (the INERT
    /// default) for every ground that has no weave, so nothing else in the
    /// pipeline changes shape. See [`Weave`].
    pub fn weave_mode(&self) -> f32 {
        match self {
            Background::Deckle { weave, .. } => weave.mode(),
            _ => 0.0,
        }
    }
    pub fn is_deckle(&self) -> bool {
        matches!(self, Background::Deckle { .. })
    }
    #[cfg(test)]
    pub fn zigzag_stroke_px(&self) -> f32 {
        (self.amplitude_px() * ZIGZAG_STROKE_FRAC).max(ZIGZAG_MIN_STROKE_PX)
    }
    #[cfg(test)]
    pub fn zigzag_row_pitch_px(&self) -> f32 {
        2.0 * self.amplitude_px() + self.zigzag_stroke_px()
    }
    pub fn density(&self) -> f32 {
        match self {
            Background::Zigzag { density, .. } => *density,
            Background::Organic { density, .. } => *density,
            Background::Deckle { density, .. } => *density,
            _ => 0.0,
        }
    }
    pub fn zigzag_banded(&self) -> bool {
        matches!(self, Background::Zigzag { banded: true, .. })
    }
    pub fn is_lava(&self) -> bool {
        matches!(self, Background::Lava { .. })
    }
    pub fn is_waves(&self) -> bool {
        matches!(self, Background::Waves { .. })
    }
    pub fn is_organic(&self) -> bool {
        matches!(self, Background::Organic { .. })
    }
    pub fn lava_params(&self) -> Option<(Srgb, Srgb, Srgb, LavaEdge, bool)> {
        match self {
            Background::Lava {
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
