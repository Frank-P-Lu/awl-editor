//! The MARGIN GROUND data model — `Background` and its dials.
//!
//! Carved out of `theme/model.rs` when item 158's Deckle family pushed that
//! file past its own size mark: the ground is a self-contained vocabulary (one
//! enum, its dial enums, the shader mirrors the laws read) with exactly one
//! consumer shape — `render::background_desc` flattens it into a `BgDesc`, and
//! the WGSL in `shaders/background.wgsl` draws whatever `shader_id` names.
//! Nothing here knows a world's name; a world picks a ground in its own literal.
//!
//! A DIAL EARNS ITS ENUM BY CARRYING MORE THAN ONE ARM. The moment a profile
//! dial's roster collapses to a single value, the enum, its shader-facing
//! scalar, its shader branch and its `ground_space` table entry are machinery
//! serving one answer — so the whole column goes, not merely the unused arm.
//! [`Weave`] and [`Tunnel`] are the dials that still carry a real choice.
//! Organic's arrangement, Lava's edge treatment and Deckle's coordinate owner
//! were each such a dial and are now properties of the ground itself, spelled
//! once in the shader where a reader can see the only behaviour that ships.

use super::color::Srgb;

// ITEM 186 — the coordinate SPACE each authored quantity below lives in
// (composition/logical vs sampling/physical) is declared next door, in
// `theme::ground_space`: one enum, one per-quantity table, and the no-wildcard
// sweep that makes a new ground declare itself. Read it before authoring a dial.
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

// ORGANIC/FINDS's shader mirrors (`shaders/background.wgsl`'s `FINDS_*`).
// `cfg(test)` for the same reason the ZIGZAG and DECKLE mirrors are: the GPU
// is the only runtime consumer, and the host reads these ONLY to state the
// field's laws. A grep-law holds them in lockstep with the WGSL.
//
// `ORGANIC_FINDS_MIN_SCALE_PX` is the cell FLOOR the shader clamps to on that
// arrangement — the cut-out is a fraction of the anchor, so below it the
// smallest of the three roles falls under a pixel and a collection aliases
// into speckle. Enforced in the shader (a property of the field, not of a dial
// pair — item 89's abutment lesson, item 158's pitch floor), so no future
// Organic world can author its way under it.
#[cfg(test)]
pub const ORGANIC_FINDS_MIN_SCALE_PX: f32 = 96.0;
/// The anchor's nominal radius, in cell units. ITEM 191: 1.15x the item-176
/// values (0.150/0.195) — see `shaders/background.wgsl`'s own comment on
/// `FINDS_ANCHOR_LO`/`FINDS_ANCHOR_HI` for why bumping the anchor alone
/// carries the whole three-role composition up by the same factor.
#[cfg(test)]
pub const ORGANIC_FINDS_ANCHOR_LO: f32 = 0.1725;
#[cfg(test)]
pub const ORGANIC_FINDS_ANCHOR_HI: f32 = 0.22425;
/// The companion's radius, as a fraction of the anchor's.
#[cfg(test)]
pub const ORGANIC_FINDS_COMPANION_LO: f32 = 0.46;
#[cfg(test)]
pub const ORGANIC_FINDS_COMPANION_HI: f32 = 0.56;
/// The cut-out's radius, as a fraction of the anchor's.
#[cfg(test)]
pub const ORGANIC_FINDS_ACCENT_HI: f32 = 0.26;
/// The threshold on the WINNING hash of a cell's 3x3 neighbourhood (item
/// 191's void-bound dropout, `finds_is_local_min` in the shader) — not a
/// per-cell rate any more. See `FINDS_DROPOUT`'s own comment in
/// `shaders/background.wgsl` for the order-statistics derivation that keeps
/// the item-176 ~10%-of-cells breathing-room density.
#[cfg(test)]
pub const ORGANIC_FINDS_DROPOUT: f32 = 0.226;

#[derive(Clone, Copy, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Background {
    Gradient { from: Srgb, to: Srgb, dir: (f32, f32) },
    Dots { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb, edge: bool },
    Pinstripe { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb },
    Stripes { from: Srgb, to: Srgb, band: Srgb, angle: f32 },
    Lava {
        ground: Srgb,
        blob_lo: Srgb,
        blob_hi: Srgb,
        dithered: bool,
    },
    Bands { tones: [Srgb; 3], angle: f32 },
    Waves { tones: [Srgb; 3] },
    Zigzag { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb,
        period_px: f32, amplitude_px: f32, angle: f32, density: f32, banded: bool },
    Organic { tones: [Srgb; 3], scale_px: f32, density: f32 },
    Deckle { ground: Srgb, layer: Srgb, deckle: Srgb, weave: Weave,
        period_px: f32, wander_px: f32, density: f32 },
    WarpedGrid { ground: Srgb, minor: Srgb, major: Srgb, tunnel: Tunnel,
        spacing_px: f32, density: f32 },
}

/// WARPED GRID's framing profile. `Fixed` is the shipped room-owned projection;
/// the other arms mutation-prove page-independent placement and forward travel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tunnel {
    Fixed,
    PageScaled,
    MarginPlaced,
    Reversed,
}

impl Tunnel {
    /// The scalar the WGSL `warped_grid_rgb` branches on (`params.w`). Each arm
    /// occupies a unit-wide band bracketed by `background.wgsl`'s own
    /// `WARP_TUNNEL_*` thresholds, so a new arm cannot silently alias an old one.
    pub fn mode(self) -> f32 {
        match self {
            Tunnel::Fixed => 0.0,
            Tunnel::PageScaled => 1.0,
            Tunnel::MarginPlaced => 2.0,
            Tunnel::Reversed => 3.0,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Tunnel::Fixed => "fixed",
            Tunnel::PageScaled => "page-scaled",
            Tunnel::MarginPlaced => "margin-placed",
            Tunnel::Reversed => "reversed",
        }
    }
}

/// DECKLE's one theme-owned profile dial (item 158). The handmade-paper field
/// draws quasi-random contour lanes; `Weave` picks WHICH lanes, and nothing
/// else in the renderer ever asks which world is active — a second world adopts
/// a profile by writing this word in its own literal, exactly as `Tunnel` and
/// `CardTexture` are chosen.
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

impl Background {
    /// The WIRE value `shaders/background.wgsl` branches on. Every number here
    /// is matched by a literal `g.shader == Nu` test in that file, so an id is
    /// a protocol constant, not an ordinal: **renumbering repaints worlds.**
    ///
    /// `2` IS DELIBERATELY VACANT — it belonged to the retired scattered-star
    /// ground. The hole costs nothing (the shader simply carries no `== 2u`
    /// branch, and an unissued id falls through to the plain gradient), while
    /// closing it would have to renumber Pinstripe(3) through WarpedGrid(10),
    /// every one of them a live wire value. It also keeps a piece of evidence
    /// standing: `render::tests::backgrounds_item69` pins that Bombora carries
    /// its OWN id `6` rather than recycling the star ground's `2`, and that
    /// claim only means anything while `2` stays unissued. **Retiring a ground
    /// vacates its id; it never renumbers its neighbours.**
    ///
    /// Contrast `ground_space::roster_index`, which is the opposite kind of
    /// number: a DENSE array index bounded by `ROSTER_LEN`, where a hole is
    /// what costs.
    pub fn shader_id(&self) -> u32 {
        match self {
            Background::Gradient { .. } => 0,
            Background::Dots { .. } => 1,
            // 2 — vacant, see above.
            Background::Pinstripe { .. } => 3,
            Background::Stripes { .. } => 4,
            Background::Lava { .. } => 0,
            Background::Bands { .. } => 5,
            Background::Waves { .. } => 6,
            Background::Zigzag { .. } => 7,
            Background::Organic { .. } => 8,
            Background::Deckle { .. } => 9,
            Background::WarpedGrid { .. } => 10,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Background::Gradient { .. } => "gradient",
            Background::Dots { .. } => "dots",
            Background::Pinstripe { .. } => "pinstripe",
            Background::Stripes { .. } => "stripes",
            Background::Lava { .. } => "lava",
            Background::Bands { .. } => "bands",
            Background::Waves { .. } => "waves",
            Background::Zigzag { .. } => "zigzag",
            Background::Organic { .. } => "organic",
            Background::Deckle { .. } => "deckle",
            Background::WarpedGrid { .. } => "warped-grid",
        }
    }
    pub fn from(&self) -> Srgb {
        match self {
            Background::Gradient { from, .. }
            | Background::Dots { from, .. }
            | Background::Pinstripe { from, .. }
            | Background::Stripes { from, .. }
            | Background::Zigzag { from, .. } => *from,
            Background::Lava { ground, .. }
            | Background::Deckle { ground, .. }
            | Background::WarpedGrid { ground, .. } => *ground,
            Background::Bands { tones, .. }
            | Background::Waves { tones }
            | Background::Organic { tones, .. } => tones[0],
        }
    }
    pub fn to(&self) -> Srgb {
        match self {
            Background::Gradient { to, .. }
            | Background::Dots { to, .. }
            | Background::Pinstripe { to, .. }
            | Background::Stripes { to, .. }
            | Background::Zigzag { to, .. } => *to,
            Background::Lava { ground, .. } => *ground,
            Background::Deckle { layer, .. } => *layer,
            Background::WarpedGrid { major, .. } => *major,
            Background::Bands { tones, .. }
            | Background::Waves { tones }
            | Background::Organic { tones, .. } => tones[2],
        }
    }
    pub fn dir(&self) -> (f32, f32) {
        match self {
            Background::Gradient { dir, .. }
            | Background::Dots { dir, .. }
            | Background::Pinstripe { dir, .. }
            | Background::Zigzag { dir, .. } => *dir,
            Background::Stripes { angle, .. } | Background::Bands { angle, .. } => {
                (angle.cos(), angle.sin())
            }
            Background::Lava { .. }
            | Background::Waves { .. }
            | Background::Organic { .. }
            | Background::Deckle { .. }
            | Background::WarpedGrid { .. } => (0.0, 1.0),
        }
    }
    pub fn tint(&self) -> Srgb {
        match self {
            Background::Dots { tint, .. }
            | Background::Pinstripe { tint, .. }
            | Background::Zigzag { tint, .. } => *tint,
            Background::Stripes { band, .. } => *band,
            Background::Gradient { from, .. } => *from,
            Background::Lava { ground, .. } => *ground,
            Background::Deckle { deckle, .. } => *deckle,
            Background::WarpedGrid { minor, .. } => *minor,
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
            Background::WarpedGrid { spacing_px, .. } => *spacing_px,
            _ => 0.0,
        }
    }
    /// DECKLE's wander amplitude, by its own name (the shared slot is
    /// `amplitude_px`, which Zigzag's chevron profile also rides).
    pub fn wander_px(&self) -> f32 {
        self.amplitude_px()
    }
    pub fn amplitude_px(&self) -> f32 {
        match self {
            Background::Zigzag { amplitude_px, .. } => *amplitude_px,
            Background::Deckle { wander_px, .. } => *wander_px,
            _ => 0.0,
        }
    }
    /// The ground's own theme-owned PROFILE dial, as the scalar its shader
    /// branches on — Deckle's [`Weave`] is the only one left. `0.0` (the INERT
    /// default) for every ground that has no profile AND for the profile's own
    /// default member, so nothing else in the pipeline changes shape. One
    /// slot: exactly one ground is ever active at a time, and each reads it
    /// through its own `params` position.
    pub fn profile_mode(&self) -> f32 {
        match self {
            Background::Deckle { weave, .. } => weave.mode(),
            _ => 0.0,
        }
    }
    pub fn is_deckle(&self) -> bool {
        matches!(self, Background::Deckle { .. })
    }
    /// Warped grid's vanishing-region placement, as the scalar the shader
    /// branches on — inert `0.0` for every ground that has no tunnel, so no
    /// other world's upload changes shape. See [`Tunnel`].
    pub fn tunnel_mode(&self) -> f32 {
        match self {
            Background::WarpedGrid { tunnel, .. } => tunnel.mode(),
            _ => 0.0,
        }
    }
    pub fn is_warped_grid(&self) -> bool {
        matches!(self, Background::WarpedGrid { .. })
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
            Background::WarpedGrid { density, .. } => *density,
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
    pub fn lava_params(&self) -> Option<(Srgb, Srgb, Srgb, bool)> {
        match self {
            Background::Lava {
                ground,
                blob_lo,
                blob_hi,
                dithered,
            } => Some((*ground, *blob_lo, *blob_hi, *dithered)),
            _ => None,
        }
    }
}
