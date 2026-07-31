//! The coordinate space of an authored ground quantity.
//!
//! The vocabulary [`super::ground`]'s dials are authored in, kept in its own
//! file because it is the answer to a question, not another dial: every number
//! a procedural ground carries is either COMPOSITION or SAMPLING, and which one
//! it is decides whether a 2x display shows the user's composition or half of
//! it. `Background::authored_quantities` below is the table; a no-wildcard
//! match makes a new ground state its answer rather than inherit one.

use super::ground::{Arrangement, Background, Weave};

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

const fn logical(name: &'static str, why: &'static str) -> GroundQuantity {
    GroundQuantity {
        name,
        space: GroundSpace::Logical,
        why,
    }
}
const fn physical(name: &'static str, why: &'static str) -> GroundQuantity {
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
            Background::Starfield { .. } => STARFIELD,
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
    pub fn roster_index(&self) -> usize {
        match self {
            Background::Gradient { .. } => 0,
            Background::Dots { .. } => 1,
            Background::Starfield { .. } => 2,
            Background::Pinstripe { .. } => 3,
            Background::Stripes { .. } => 4,
            Background::Lava { .. } => 5,
            Background::Bands { .. } => 6,
            Background::Waves { .. } => 7,
            Background::Zigzag { .. } => 8,
            Background::Organic { .. } => 9,
            Background::Deckle { .. } => 10,
        }
    }

    /// How many members the `Background` roster has. Bumping this without
    /// enrolling a representative in the item-186 sweep fails that law by name.
    pub const ROSTER_LEN: usize = 11;
}

const DOTS: &[GroundQuantity] = &[
    logical(
        "the 24px dot cell",
        "the lattice PITCH — how many dots a margin holds is the composition \
         itself, and the number the user counts",
    ),
    logical(
        "the dot radius (1.4px uniform; 0.85..3.0px proximity-scaled)",
        "a mark's SIZE is what the eye reads it at, not how many samples drew \
         it; the proximity ramp is a size gradient across the whole margin",
    ),
    physical(
        "the dot rim feather (1.0px uniform, 0.9px proximity-scaled)",
        "the skirt that resolves a round edge on the sample grid — a 2x display \
         should draw the SAME dot with a crisper rim, not a fatter blur",
    ),
];

const STARFIELD: &[GroundQuantity] = &[
    logical(
        "the 34px star cell",
        "the lattice PITCH — how sparse or crowded the night reads, which is \
         the whole character of the field",
    ),
    logical(
        "the star radius (0.7px) and the sparkle's arm half-width (0.4px) and \
         length taper (2.5..4.5px)",
        "the drawn SHAPE of a star. The long taper is a profile, not a rim: it \
         is visible as a gradient, so it scales with the star rather than with \
         the sample grid",
    ),
    physical(
        "the star rim feather (1.0px) and the sparkle rim feather (0.6px)",
        "the two skirts that resolve those shapes' edges",
    ),
];

const PINSTRIPE: &[GroundQuantity] = &[
    logical(
        "the 9px rule period",
        "the ledger PITCH — how many rules a margin holds, and the tightest \
         pitch in the whole family",
    ),
    logical(
        "the rule half-width (0.5px)",
        "the drawn weight of a printed rule — a ledger's hairline should read \
         the same on any display",
    ),
    physical(
        "the rule edge feather (0.7px)",
        "the skirt that keeps a hairline from stair-stepping; a finer grid \
         should draw the same hairline more exactly",
    ),
];

const STRIPES: &[GroundQuantity] = &[
    logical(
        "the 13px stripe period",
        "the diagonal band's PITCH — how many stripes cross the bright band \
         hugging the page edge",
    ),
    logical(
        "the stripe half-width (2.0px)",
        "the drawn weight of one stripe, which the eye reads against its \
         neighbours rather than against the sample grid",
    ),
    physical(
        "the stripe edge feather (1.5px)",
        "the skirt that resolves a diagonal edge, where a sample grid needs it \
         most",
    ),
];

const LAVA: &[GroundQuantity] = &[
    logical(
        "the blob centres and radii",
        "already density-independent by a DIFFERENT mechanism, and left alone: \
         they are authored as FRACTIONS of the viewport (`shaders/lava.wgsl`'s \
         `field_viewport`), so the composition scales with the canvas at any \
         device ratio. The item's premise does not reach this ground",
    ),
    logical(
        "the frost skirt (`lava::FROST_FEATHER_PX`) and blur radius",
        "already logical before this item — `lava::frost_px(logical, zoom, dpi)` \
         converts them at the one owner. Its size decides whether neighbouring \
         margin words MERGE into one island, so it is composition even though it \
         is spelled like a feather",
    ),
    physical(
        "the print-grain Bayer cell (16px effective)",
        "the same quantization argument as the shared dither, doubled for a \
         coarser grain",
    ),
];

const BANDS: &[GroundQuantity] = &[
    logical(
        "the two tier boundaries (1/3, 2/3 of the projected extent)",
        "already density-independent: they are FRACTIONS of the viewport, so \
         three bands span any canvas at any ratio. Named here so the absence of \
         a pixel pitch is a recorded decision, not an omission",
    ),
    physical(
        "the boundary feather (1.5px)",
        "the skirt across a tone-on-tone boundary, where banding is most visible",
    ),
];

const WAVES: &[GroundQuantity] = &[
    logical(
        "the scallop amplitude (22px) and wavelength (260px)",
        "how tall and how wide a swell READS — the composition of the sea",
    ),
    logical(
        "the tier boundaries (1/3, 2/3 of viewport height)",
        "viewport FRACTIONS, so they were density-independent already; named \
         here so the absence of a pixel quantity is a decision on the record",
    ),
    physical(
        "the boundary feather (1.5px)",
        "as Bands: the skirt on a tone-on-tone crest line",
    ),
];

const ZIGZAG: &[GroundQuantity] = &[
    logical(
        "period_px (the tooth wavelength)",
        "the chevron's repeat PITCH along travel — the SCALE dial a world \
         authors, and half of what a margin's rhythm reads as",
    ),
    logical(
        "amplitude_px (the profile)",
        "the peak excursion across travel; through item 89's abutment rule it \
         also DERIVES the row pitch, so a physical amplitude would make the \
         field's own pitch density-dependent",
    ),
    logical(
        "the stroke thickness (max(0.10*amplitude, 1.2px)) and its 1.2px floor",
        "the drawn weight of the ribbon, and it feeds the row pitch through the \
         same abutment rule; a floor on a composition quantity is itself a \
         composition quantity, or the clamp hands the pitch back to the display",
    ),
    logical(
        "the ribbon's soft edge (0.6*thickness .. thickness)",
        "a PROPORTION of the stroke, not a fixed skirt — it is part of the drawn \
         profile and widens with a bolder ribbon, so it is not a sampling \
         feather at all",
    ),
];

const ORGANIC_MASSES: &[GroundQuantity] = &[
    logical(
        "scale_px (the cell)",
        "the collage's cell SIZE — how many masses a margin holds and how large \
         each reads. The quantity item 176 made countable",
    ),
    logical(
        "the drift floors (12px x, 9px y)",
        "a displacement the eye measures against the collage it moves; in \
         physical px a 2x display would slide the field half as far, reopening \
         item 163's \"could not see it move\" defect on the best displays",
    ),
    logical(
        "the mass/island/hole radii and their soft edges",
        "authored as FRACTIONS of the cell, so they follow it for free — the \
         blobs' softness is their drawn character, not a resolve of the grid",
    ),
];

const ORGANIC_FINDS: &[GroundQuantity] = &[
    logical(
        "scale_px (the collection cell)",
        "how many three-object collections are visible and how large each \
         reads — the exact quantity that turned this from texture into \
         composition (item 176)",
    ),
    logical(
        "FINDS_MIN_SCALE_PX (the 96px cell floor)",
        "its MOTIVATION is a sampling one (below it the cut-out falls under a \
         pixel), but it is a floor on a COMPOSITION quantity: in physical px it \
         would clamp the same authored cell differently at 1x and 2x, putting \
         the composition back under the display exactly where the floor binds. \
         Logical is also the conservative reading — at 2x those 96 logical px \
         carry 192 device pixels of detail",
    ),
    logical(
        "the anchor/companion/cut-out radii, offsets, jitter and lattice angle",
        "authored in CELL units, so the role hierarchy item 176 proves holds at \
         any scale and any density",
    ),
    logical(
        "the drift floors (12px x, 9px y)",
        "as Masses — both arrangements share one whole-field translation, and a \
         displacement is measured against the collage it moves",
    ),
    physical(
        "FINDS_EDGE_AA_PX (the 0.75px crisp edge)",
        "THE canonical sampling quantity, and it did not move. Crispness is the \
         point of this arrangement: the transition band measures 0.75px on the \
         GLASS at every device ratio, so a 2x display draws the same hard edge \
         more exactly. Converting it would make a better display blurrier",
    ),
];

const DECKLE_STRATA: &[GroundQuantity] = &[
    logical(
        "period_px (the lane pitch)",
        "how many paper contours a margin holds — the material's grain",
    ),
    logical(
        "DECKLE_MIN_PITCH_PX (the 40px pitch floor)",
        "the same argument as FINDS_MIN_SCALE_PX: a floor on a composition \
         quantity is a composition quantity",
    ),
    logical(
        "wander_px and the two wander frequencies (per logical px)",
        "how far and how often a lane tears off true — the tear's own scale",
    ),
    logical(
        "the deckle edge (0.015..0.075 of a lane)",
        "already a FRACTION of the composition, so it widens with the lane. \
         Deckle carries NO sampling feather: a torn paper edge is a drawn thing, \
         not a resolve of the sample grid",
    ),
];

const DECKLE_FIBRES: &[GroundQuantity] = &[
    logical(
        "period_px (the fibre pitch) and DECKLE_MIN_PITCH_PX",
        "how many fibres a margin holds, and the floor under it",
    ),
    logical(
        "wander_px and the fibre/vein frequencies",
        "how far and how often a fibre strays off true — the meander's own \
         scale, read against the margin it crosses",
    ),
    logical(
        "the fibre (0.7..2.2px) and vein (0.5..1.45px) half-width ramps",
        "a fibre IS a soft translucent stroke — that ramp is its drawn profile, \
         visible as a gradient, so it scales with the fibre rather than with the \
         sample grid",
    ),
];
