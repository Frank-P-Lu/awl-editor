//! ITEM 186 — THE CLASSIFICATION TABLES THEMSELVES.
//!
//! One `const` per ground (per PROFILE, where a ground's profiles author
//! different numbers), naming every authored quantity, its coordinate space and
//! why. Split out of the parent when Kite's warped grid pushed the file past its
//! ceiling: the parent is the vocabulary and the wildcard-free dispatch, this is
//! the answers. Nothing here is code — it is the round's deliverable in the form
//! a law can sweep.

use super::{GroundQuantity, logical, physical};

pub(super) const DOTS: &[GroundQuantity] = &[
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

pub(super) const STARFIELD: &[GroundQuantity] = &[
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

pub(super) const PINSTRIPE: &[GroundQuantity] = &[
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

pub(super) const STRIPES: &[GroundQuantity] = &[
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

pub(super) const LAVA: &[GroundQuantity] = &[
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

pub(super) const BANDS: &[GroundQuantity] = &[
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

pub(super) const WAVES: &[GroundQuantity] = &[
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

pub(super) const ZIGZAG: &[GroundQuantity] = &[
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

pub(super) const ORGANIC_MASSES: &[GroundQuantity] = &[
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

pub(super) const ORGANIC_FINDS: &[GroundQuantity] = &[
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

pub(super) const DECKLE_STRATA: &[GroundQuantity] = &[
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

pub(super) const WARPED_GRID: &[GroundQuantity] = &[
    logical(
        "spacing_px (the ring pitch at WARP_RING_PITCH_AT of the anchor)",
        "the projected PITCH of the cross-rings at one fixed place on a section \
         that itself never rescales — so it is a length the reader measures \
         against the marks beside it. It is what `WARP_RPO_MIN..MAX` bounds and \
         what sets how many rings a margin holds, which is the composition \
         itself",
    ),
    logical(
        "curvature (the bend gain)",
        "a dimensionless gain multiplying `anchor^2`, a composition quantity, so \
         it inherits its space: it decides HOW FAR the shared opening shifts \
         off-centre in a turn, and the eye measures that shift against the \
         section it moves, never against the sample grid",
    ),
    logical(
        "density (the coverage multiplier)",
        "dimensionless contrast, spaceless by construction, and named here so its \
         absence from both classes is a recorded decision rather than an \
         omission. `0.0` collapses the field to its flat ground EXACTLY, which \
         is what gives the family item 86's `mark_field` differential oracle",
    ),
    logical(
        "the route's pose triple (yaw, pitch, forward_cells)",
        "yaw and pitch are dimensionless STEERING and forward travel is a COUNT \
         of ring cells, so a 2x display travels the same journey at the same \
         speed through the same lattice. Resolved on the host by \
         `crate::warpgrid::route_pose`; the shader carries no route arithmetic",
    ),
    logical(
        "WARP_SECTION_ROOM_FRAC (the anchor ring's radius, in room heights)",
        "the whole size and shape of the cross-section, and after item 194 round \
         2 the whole of its SCALE: a ratio of a quantity the host measured (the \
         room), so it is density-independent by construction — the same \
         mechanism `Bands` and `Lava` are named here for. Round 1 authored it \
         against the PAGE COLUMN instead, which is what let a wider page \
         rescale and squash the world; that geometry survives only inside \
         `Tunnel::PageScaled`, as the mutation arm",
    ),
    logical(
        "WARP_WINDOW_FULL / WARP_WINDOW_TIGHT / WARP_WINDOW_STRADDLE (where each \
         margin's window sits on the one projection)",
        "the first two are margin widths measured in ANCHORS and the third a \
         fraction of a margin's own width — all three dimensionless ratios of \
         composition quantities, so they carry no pixel to convert. They move \
         the WINDOW, never the world, which is exactly why the projection's \
         aspect ratio survives the whole adaptive-column range",
    ),
    logical(
        "the ring/rail half-widths (0.45px minor, 1.00px major) and \
         WARP_CORE_FRAC's radius floor",
        "the drawn WEIGHT of a line, which the eye reads against its neighbours \
         (the Pinstripe hairline rule), and a floor on a COMPOSITION quantity — \
         which is itself composition, or the clamp hands the far end's pitch \
         back to the display exactly where it binds (the FINDS_MIN_SCALE_PX \
         argument)",
    ),
    logical(
        "WARP_EDGE_QUIET_PX / WARP_EDGE_FADE_MAX_PX and \
         WARP_NARROW_LO_PX / WARP_NARROW_HI_PX",
        "a reach into the margin and the margin WIDTHS at which the minor \
         lattice retires — both measured by the eye against the margin they \
         describe, the same argument as the shared EDGE_FALLOFF",
    ),
    physical(
        "WARP_AA_PX (the 1.0px line skirt)",
        "the skirt that resolves a line's edge on the sample grid, and this \
         ground's ONLY sampling quantity. A 2x display must draw the SAME \
         hairline more crisply, not a fatter blur. It is the one number here \
         already in the device space `fwidth` measures in, so unlike the drawn \
         line WEIGHT beside it, it needs no conversion at all",
    ),
    logical(
        "WARP_ALIAS_FADE_LO_PX / WARP_ALIAS_FADE_HI_PX (the 4.5..9.0px moire \
         bound)",
        "its MOTIVATION is a sampling one — a converging lattice aliases when its \
         projected pitch approaches a device pixel — but what it DECIDES is how \
         deep the tunnel is drawn, i.e. how many cross-rings the composition \
         holds. In physical px the display would choose that, which is exactly \
         the tension item 186 settled the same way for FINDS_MIN_SCALE_PX and \
         DECKLE_MIN_PITCH_PX. Logical is also the conservative reading: at 2x \
         these logical pixels carry twice the device samples, so the moire the \
         bound exists to prevent is further away, not nearer",
    ),
];

pub(super) const DECKLE_FIBRES: &[GroundQuantity] = &[
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
