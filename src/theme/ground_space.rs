//! The coordinate space of an authored ground quantity.
//!
//! The vocabulary [`super::ground`]'s dials are authored in, kept in its own
//! file because it is the answer to a question, not another dial: every number
//! a procedural ground carries is either COMPOSITION or SAMPLING, and which one
//! it is decides whether a 2x display shows the user's composition or half of
//! it. `Background::authored_quantities` below is the table; a no-wildcard
//! match makes a new ground state its answer rather than inherit one.

use super::ground::{Arrangement, Background, Tunnel, Weave};

mod tables;
use tables::{
    BANDS, DECKLE_FIBRES, DECKLE_STRATA, DOTS, LAVA, ORGANIC_FINDS, ORGANIC_MASSES, PINSTRIPE,
    STRIPES, WARPED_GRID, WAVES, ZIGZAG,
};

/// Which coordinate space one authored ground quantity lives in.
///
/// The two classes are structurally different things and the distinction is the
/// point; a blanket conversion of one into the other is the failure mode:
///
/// * [`GroundSpace::Logical`] — COMPOSITION. A cell, a pitch, a mark size, a
///   wander, a reach. It describes what the user sees, so it must be
///   density-independent: matched LOGICAL canvases show the same world
///   composition at 1x and at 2x.
/// * [`GroundSpace::Physical`] — SAMPLING. An antialias feather, a dither cell.
///   It describes how the device's sample grid RESOLVES that composition, so it
///   belongs to the device pixel: a 2x display resolves the SAME composition
///   more finely, which is the whole benefit of the density. Converting it would make the same edge
///   blurrier on a better display.
///
/// The renderer honours this in one place — `shaders/background.wgsl` divides
/// the fragment position through the device ratio once (`to_logical`) and
/// converts each physical feather back through one owner (`sampling_feather`) —
/// and [`Background::authored_quantities`] is the table that says which number
/// is which and why. The precedent is `render::dither`'s
/// `WAGTAIL_HIGHLIGHT_STIPPLE_CELL_LOGICAL`, an authored logical cell already
/// multiplied by the live DPI at its one owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundSpace {
    /// COMPOSITION — logical pixels, so the composition is the user's, not the
    /// display's.
    Logical,
    /// SAMPLING — physical pixels, so a better display resolves the same
    /// composition more finely.
    Physical,
}

impl GroundSpace {
    pub fn as_str(self) -> &'static str {
        match self {
            GroundSpace::Logical => "logical",
            GroundSpace::Physical => "physical",
        }
    }
    /// The class name this space stands for, for a law's own failure text.
    pub fn class(self) -> &'static str {
        match self {
            GroundSpace::Logical => "composition",
            GroundSpace::Physical => "sampling",
        }
    }
}

/// One authored number a ground carries, the space it lives in, and why.
///
/// `name` is the identifier as it is written — a field of the world literal
/// (`period_px`) or the shader constant that shapes the field
/// (`FINDS_EDGE_AA_PX`). `why` is not decoration: a quantity that cannot state
/// its reason has not been classified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroundQuantity {
    pub name: &'static str,
    pub space: GroundSpace,
    pub why: &'static str,
}

pub(super) const fn logical(name: &'static str, why: &'static str) -> GroundQuantity {
    GroundQuantity {
        name,
        space: GroundSpace::Logical,
        why,
    }
}
pub(super) const fn physical(name: &'static str, why: &'static str) -> GroundQuantity {
    GroundQuantity {
        name,
        space: GroundSpace::Physical,
        why,
    }
}

// The two quantities EVERY ground shares, so no ground states them twice.
const SHARED: &[GroundQuantity] = &[
    logical(
        "EDGE_FALLOFF",
        "how far the page's presence radiates into the margin — a reach the eye \
         measures against the margin's own width, so it is composition",
    ),
    physical(
        "BAYER8 (the banding-kill dither cell)",
        "a threshold matrix whose job is to perturb each DEVICE pixel by half a \
         quantization step before the 8-bit target rounds it; tiling it \
         logically would put four device pixels on one threshold at 2x and hand \
         the banding straight back",
    ),
];

impl Background {
    /// Every authored quantity this ground carries, the space it lives in, and
    /// why — the item-186 classification table, and the authority
    /// `shaders/background.wgsl` mirrors.
    ///
    /// It names BOTH the dials a world writes in its own literal (`period_px`,
    /// `scale_px`) and the shader constants that shape the field
    /// (`FINDS_EDGE_AA_PX`, Dots' cell), because both are authored numbers and
    /// both were physical pixels before this item. The match carries NO wildcard
    /// arm: a new ground cannot inherit a classification it never made.
    pub fn authored_quantities(&self) -> &'static [GroundQuantity] {
        match self {
            Background::Gradient { .. } => &[],
            Background::Dots { .. } => DOTS,
            Background::Pinstripe { .. } => PINSTRIPE,
            Background::Stripes { .. } => STRIPES,
            Background::Lava { .. } => LAVA,
            Background::Bands { .. } => BANDS,
            Background::Waves { .. } => WAVES,
            Background::Zigzag { .. } => ZIGZAG,
            Background::Organic { arrangement, .. } => match arrangement {
                Arrangement::Masses => ORGANIC_MASSES,
                Arrangement::Finds => ORGANIC_FINDS,
            },
            Background::Deckle { weave, anchor, .. } => match (weave, anchor) {
                (Weave::Strata, _) => DECKLE_STRATA,
                (Weave::Fibres, _) => DECKLE_FIBRES,
            },
            // Every tunnel profile authors the same quantities in the same
            // spaces; the non-shipping arms change framing or travel direction.
            Background::WarpedGrid { tunnel, .. } => match tunnel {
                Tunnel::Fixed => WARPED_GRID,
                Tunnel::PageScaled => WARPED_GRID,
                Tunnel::MarginPlaced => WARPED_GRID,
                Tunnel::Reversed => WARPED_GRID,
            },
        }
    }

    /// Every quantity, this ground's own plus the two the whole family shares.
    pub fn all_authored_quantities(&self) -> Vec<GroundQuantity> {
        let mut all: Vec<GroundQuantity> = self.authored_quantities().to_vec();
        // A plain gradient draws no marks, so neither shared quantity reaches
        // it — the dither still runs, and it is the one thing it declares.
        if matches!(self, Background::Gradient { .. }) {
            all.push(SHARED[1]);
            return all;
        }
        all.extend_from_slice(SHARED);
        all
    }

    /// This ground's ordinal in the `Background` roster. Its ONLY purpose is to
    /// let a law prove its sweep is total: the match has no wildcard, so a new
    /// variant fails to compile here, and `ROSTER_LEN` then fails the sweep
    /// until a representative is enrolled. Never a shader discriminant —
    /// `shader_id` is that, and it deliberately maps two variants onto 0.
    ///
    /// This is a DENSE index into `[bool; ROSTER_LEN]`, which makes it the
    /// exact opposite kind of number from `shader_id`: a hole here would leave
    /// a slot no ground can ever set, and the sweep's own completeness check
    /// would fail forever on an index that stands for nothing. So retiring a
    /// ground CLOSES its gap and drops `ROSTER_LEN`, where retiring a
    /// `shader_id` vacates one and touches nothing else.
    pub fn roster_index(&self) -> usize {
        match self {
            Background::Gradient { .. } => 0,
            Background::Dots { .. } => 1,
            Background::Pinstripe { .. } => 2,
            Background::Stripes { .. } => 3,
            Background::Lava { .. } => 4,
            Background::Bands { .. } => 5,
            Background::Waves { .. } => 6,
            Background::Zigzag { .. } => 7,
            Background::Organic { .. } => 8,
            Background::Deckle { .. } => 9,
            Background::WarpedGrid { .. } => 10,
        }
    }

    /// How many members the `Background` roster has. Bumping this without
    /// enrolling a representative in the item-186 sweep fails that law by name.
    pub const ROSTER_LEN: usize = 11;
}
