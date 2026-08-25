//! THE FROST'S EXTENT POLICY — pure, device-free, and the whole of what the two frost
//! arms disagree about. No wgpu here: this is the arithmetic and the roster question,
//! so both are unit-testable without a GPU (the purest reachable seam), and
//! `super`'s pipeline plumbing stays one concern.
//!
//! TWO EXTENTS, ONE EFFECT ([`Frost`]). A full-takeover overlay frosts the WHOLE
//! canvas. The CRISP pickers — the theme picker and the caret-style picker — frost
//! only THEIR OWN FOOTPRINT: the card's box and nothing outside it, so the
//! surrounding page keeps the world's live colours (the preview those two pickers
//! exist for) while the document under the card stops competing with the rows. The
//! footprint arm exists because frost was a property of the PLATE, and a composition
//! that draws no plate at all left the document and the list interleaving
//! glyph-for-glyph; a composition that DOES back its own rows already covers what it
//! sits on and stays crisp, byte-identical. Which compositions enrol is
//! [`footprint_frost_applies`] — asked of the ROSTER's own two backing owners, never
//! of a named world. The search SPLIT panel keeps the doc bright either way.
//!
//! THE FOOTPRINT IS A MASK IN `fs_comp` ([`footprint_mask`]), CARRIED IN THE
//! COMPOSITE'S ALPHA, with a SCISSOR kept only as a conservative bound. An earlier
//! reading of this module preferred the scissor alone and dismissed the rect uniform
//! because it "would produce the same hard edge anyway" — true of a rect uniform, and
//! FALSE of a feathered one. A scissor can only answer yes or no per pixel, so the
//! frost's boundary was a knife edge that sliced words mid-glyph, and no value
//! anywhere in the path could soften it: `blend: None` on the composite target plus an
//! alpha of `1.0` out of `fs_comp` is a hard rectangle by CONSTRUCTION.
//!
//! So the extent now arrives in [`U`] as a shape — box, shear and feather width — and
//! the composite target blends. Three properties hold it together:
//!
//! * [`Frost::Full`]'s mask is exactly `1.0` at every pixel, and an alpha of exactly
//!   1.0 under `BlendState::ALPHA_BLENDING` is `src * 1 + dst * 0` — a replace. So the
//!   full-takeover arm needs no second pipeline and is byte-identical. That is an
//!   assertion the laws MAKE (`the_full_frosts_composite_is_destination_independent`
//!   renders the same frame over two different destinations and requires the results
//!   to match bit-for-bit), not one this comment asks a reader to take on trust.
//! * The scissor survives as the [`footprint_bound`] of the whole feathered shape, so
//!   nothing outside it is written at all and the byte-identity of the page beyond the
//!   frost holds on every backend rather than resting on an sRGB blend round-trip.
//!   Its correctness condition is that the mask is ZERO at and beyond that bound,
//!   which is its own law.
//! * The mask is 1 on and INSIDE the shape's faces and ramps to 0 across the feather
//!   OUTSIDE them — the same direction as [`scissor_px`]'s outward rounding, and the
//!   same shape as `lava::lava_mask_2d`'s gutter carve, so everything the card covers
//!   stays fully frosted and only the skirt is new.
//!
//! EVERY LENGTH HERE IS PHYSICAL, AND THE ONE AUTHORED IN LOGICAL UNITS
//! ([`DOWNSAMPLE`], the frost's reach) IS MULTIPLIED BY DPI EXACTLY ONCE
//! ([`downsample_for`]). A fixed physical downsample made the frost's reach constant
//! in DEVICE px, so its reach in the units a reader sees HALVED on a 2× display —
//! the class of defect every capture (which runs at `--capture-dpi 1`) is blind to.

/// Downsample factor AT 1×: the blur runs at 1/Nth resolution on each axis (N×N fewer
/// pixels), which both speeds the passes and widens the effective blur radius for
/// free. Quarter-res (4) is the sweet spot — clearly frosted, still cheap. Never used
/// raw against a surface: [`downsample_for`] scales it by DPI so the frost's reach is
/// constant in LOGICAL px.
pub(super) const DOWNSAMPLE: u32 = 4;

/// The downsample factor for a surface at `dpi` — [`DOWNSAMPLE`] logical px worth of
/// device pixels, floored at 1. The Gaussian's reach is a fixed number of QUARTER-RES
/// TEXELS (±4 taps, [`BLUR_ROUNDS`] rounds), so its reach in physical px is
/// proportional to this factor: scaling the factor with DPI is what keeps the frost
/// the same DEFOCUS at 1× and 2× rather than half as strong on retina. `dpi == 1`
/// returns [`DOWNSAMPLE`] exactly, so every capture (all of which run at
/// `--capture-dpi 1`) is byte-identical.
pub(super) fn downsample_for(dpi: f32) -> u32 {
    if !dpi.is_finite() || dpi <= 0.0 {
        return DOWNSAMPLE;
    }
    ((DOWNSAMPLE as f32 * dpi).round() as u32).max(1)
}

/// How far the frosted backdrop dims toward the theme's OWN `base_100` (0 = pure
/// blur, no recede; 1 = the flat base). Small — the doc should still read through the
/// frost, just a value back. Never toward neutral grey (it is the theme's own base).
pub(super) const DIM: f32 = 0.16;

/// The FOOTPRINT arm's dim: ZERO. A full takeover recedes the whole document a value
/// ([`DIM`]) because the document is no longer the subject; a crisp picker's footprint
/// is a hole in a page whose live colours are STILL the subject a hair outside the
/// card's edge, and any recede would put a step of value at that edge on top of the
/// defocus. Pure blur, so the frosted patch is provably the SAME hue as the page it
/// sits in — the claim the laws assert.
pub(super) const FOOTPRINT_DIM: f32 = 0.0;

/// THE FOOTPRINT'S FEATHER (logical px): how far OUTSIDE the shape's faces the frost
/// ramps from full strength to nothing.
///
/// AUTHORED HERE rather than borrowed from `lava::MARGIN_GAP_PX`, which is the same
/// number today. What IS borrowed from the lava lamp is the SHAPE — `lava_mask_2d`'s
/// gutter carve is a bounded rect whose faces `smoothstep` over a gap, which is
/// exactly this — but that constant is the width of a ramp across a page MARGIN, a
/// property of a ground field's relationship to the writing column. Coupling a
/// picker's frost edge to it would mean retuning the lamp's margins silently retunes
/// this, and the two have no subject in common.
///
/// The FLOOR under the number is not taste, and it is law-tested
/// (`the_footprint_feather_is_at_least_the_blur_it_edges`): a feather NARROWER than
/// the blur's own reach reads as a hard edge regardless, because the interior it bounds
/// is soft over a wider distance than the boundary is. The Gaussian reaches ±4 taps of
/// [`DOWNSAMPLE`] logical px, so that reach is 16 logical px and this must clear it.
/// (`lava::FROST_FEATHER_PX` = 7.0 is the other candidate quantity in the tree and is
/// the closer *kind* — a frost's own skirt — but it sits UNDER that floor, so it would
/// have shipped the reported defect at a softer edge.)
///
/// Above the floor the width is TASTE and flagged for live review.
pub(crate) const FOOTPRINT_FEATHER_PX: f32 = 28.0;

/// The feather in PHYSICAL px at `dpi` — the authored logical width multiplied by DPI
/// EXACTLY ONCE, the same discipline (and for the same reason) as [`downsample_for`].
/// A feather held in device px would halve the reader's edge softness on a 2× display,
/// which no `--capture-dpi 1` capture can see.
pub(super) fn footprint_feather_px(dpi: f32) -> f32 {
    if !dpi.is_finite() || dpi <= 0.0 {
        return FOOTPRINT_FEATHER_PX;
    }
    FOOTPRINT_FEATHER_PX * dpi
}

/// THE CRISP PICKER'S FOOTPRINT: the card's box, and how far the shape LEANS with the
/// composition drawn inside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footprint {
    /// The summoned card's own box `[x, y, w, h]` in PHYSICAL px (the same canvas
    /// coordinates the card is drawn at).
    pub rect: [f32; 4],
    /// The SHEAR: physical px of horizontal displacement per physical px DOWN from
    /// the box's vertical centre. `0.0` on an upright composition.
    ///
    /// READ from the composition the frame actually drew — the spine's own resolved
    /// per-row step over the row pitch it steps across — never re-authored from
    /// `ROW_STEP`. The drawn step yields on a cramped card
    /// (`TRAVEL_MAX_BAND_FRACTION`), so a second reading of the constant would lean
    /// the frost more than the spine beside it actually rakes.
    pub shear: f32,
}

/// WHERE a frame's frost lands.
///
/// Both arms run the identical capture → downsample → Gaussian → composite; they
/// differ in the composite's EXTENT and its dim. Carried as one value (rather than a
/// bool plus a maybe-rect) so the extent and the dim cannot disagree, and so the
/// caller's decision arrives at [`BlurBackdrop::ensure`] in ONE argument.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Frost {
    /// The whole canvas — a full-takeover overlay, the held HUD, the lifetime card,
    /// the hold-⌘ peek. The historical behaviour, unchanged.
    Full,
    /// The summoned card's own footprint, and nothing more than a feather outside it.
    Footprint(Footprint),
}

impl Frost {
    /// How far this arm's composite dims toward `base_100`.
    pub(super) fn dim(self) -> f32 {
        match self {
            Frost::Full => DIM,
            Frost::Footprint(_) => FOOTPRINT_DIM,
        }
    }
}

/// The footprint's signed OUTSIDE distance at physical pixel `(px, py)`: `<= 0` inside
/// the shape, positive out beyond it, its magnitude just outside a face the per-axis
/// distance to that face. The same construction as `lava::gutter_corner_dist_outside`
/// (per-axis outside distances combined by `max`, so the result is negative iff both
/// axes are inside), asked ONCE — of the box SHEARED about its own vertical centre.
///
/// THE SHAPE IS A PARALLELOGRAM, AND THE SILHOUETTE IS THE DELIVERABLE. It has four
/// sides: two horizontal, two raking with the drawn spine. At any row both its left and
/// its right face translate by the same `shear × (py − cy)`, which is what makes it read
/// as leaning rather than as a leaning thing inside an upright thing.
///
/// AN EARLIER READING OF THIS MODULE UNIONED THE LEANING TERM WITH THE UPRIGHT BOX and
/// called that union a coverage floor. The floor was real — the card's query line is
/// upright and flush to its text edge, and frosting the lean alone left it over sharp
/// document on a mirrored composition — but the union CANNOT read as a parallelogram at
/// any shear on any world, because the box is one of its terms and therefore always
/// wholly inside the result. The shear could only add two overhang corners to a
/// rectangle. So the coverage duty moved OFF the mask and INTO the box: [`footprint_box`]
/// widens the rect until the parallelogram contains the card's upright chrome, and
/// widening a parallelogram leaves a parallelogram. The floor did not go away; it stopped
/// being a second shape.
///
/// MUST match `shaders/blur.wgsl`'s `footprint_dist_outside`.
// The SHIP path evaluates this on the GPU; this is the pure mirror a law grades and a
// reader checks the shader against (the same arrangement `lava`'s mask helpers use).
#[allow(dead_code)]
pub(super) fn footprint_dist_outside(foot: Footprint, px: f32, py: f32) -> f32 {
    let [x, y, w, h] = foot.rect;
    let gy = (y - py).max(py - (y + h));
    // The un-shear: where this pixel sits in the leaning shape's own frame.
    let sx = px - foot.shear * (py - (y + h * 0.5));
    (x - sx).max(sx - (x + w)).max(gy)
}

/// THE FOOTPRINT'S BOX: the card's own box, WIDENED until the parallelogram that box
/// shears into contains `upright` — the card's own chrome that the rake does not carry
/// (`[left, top, right, bottom]` in the same physical canvas coordinates).
///
/// THIS IS WHERE THE COVERAGE FLOOR LIVES NOW, and it is a floor over the shape's WIDTH
/// rather than a second shape beside it. A parallelogram widened horizontally is still a
/// parallelogram, so paying for coverage here costs the silhouette nothing; paying for it
/// in the mask cost the silhouette everything (see [`footprint_dist_outside`]).
///
/// The arithmetic is exact rather than padded. The shape's left face at row `py` sits at
/// `x + shear × (py − cy)` and its right at `x + w + shear × (py − cy)`, with `cy` a
/// function of `y` and `h` ALONE — so widening in x cannot move the pivot, and the two
/// bounds decouple. Over the chrome's own row range that displacement is linear, so its
/// extremes are at the range's ends, and the box must satisfy `x ≤ left − max` and
/// `x + w ≥ right − min`.
///
/// It only ever GROWS, and a chrome box already inside the shape grows it by nothing —
/// so an upright composition (`shear == 0`, where the parallelogram IS the box) and every
/// card whose chrome already rakes with its rows keep the rect they always had, bit for
/// bit.
pub fn footprint_box(card: [f32; 4], shear: f32, upright: Option<[f32; 4]>) -> [f32; 4] {
    let [x, y, w, h] = card;
    let Some(chrome) = upright else {
        return card;
    };
    if !(shear.is_finite() && chrome.iter().all(|v| v.is_finite())) {
        return card;
    }
    // The chrome's span in the shape's OWN un-sheared frame — the same frame, and the
    // same owner, the narrowing reads from the other side (`super::narrow`).
    let (lo, hi) = super::narrow::unsheared_x_span(chrome, shear, y + h * 0.5);
    let x0 = x.min(lo);
    let x1 = (x + w).max(hi);
    [x0, y, (x1 - x0).max(w), h]
}

/// THE FOOTPRINT'S TOP FACE, SEATED AT THE CANVAS TOP — extending a card-anchored box
/// upward to `y = 0`, which sits above any document line drawn beneath a card seated
/// near the window's top. A composition whose top face sits mid-way through the
/// document's own opening heading straddles it: the ink above the face reads sharp,
/// the ink below reads through the frost, and the seam falls inside a single glyph
/// row. Seating the face at the canvas top puts the whole heading on one side of it.
///
/// The naive edit (`rect[1] = 0.0; rect[3] = old_bottom`) SILENTLY MOVES THE RAKING
/// SIDE FACES: [`footprint_dist_outside`] un-shears every point about the box's own
/// vertical centre (`cy = y + h/2`), so changing `y`/`h` moves `cy` and therefore
/// slides the sheared side faces sideways at every row. Compensating `x` by
/// `shear * (new_cy - old_cy)` keeps every side face at the exact canvas x it already
/// had; only the top boundary moves. [`super::narrow::footprint_narrow`]'s own bottom
/// face compensates the same way, for the same reason, when it shrinks `h` from the
/// other end.
///
/// A non-finite shear or rect returns the rect unchanged — the inert answer, since a
/// caller already checked finiteness upstream and this is not the place to invent a
/// box.
pub fn footprint_seat_top(rect: [f32; 4], shear: f32) -> [f32; 4] {
    let [x, y, w, h] = rect;
    if !(x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite() && shear.is_finite()) {
        return rect;
    }
    let bottom = y + h;
    let new_y = y.min(0.0);
    let new_h = (bottom - new_y).max(0.0);
    let old_cy = y + h * 0.5;
    let new_cy = new_y + new_h * 0.5;
    let new_x = x + shear * (new_cy - old_cy);
    [new_x, new_y, w, new_h]
}

/// THE FOOTPRINT'S COVERAGE at physical pixel `(px, py)`: `1.0` on and inside the
/// shape's faces, ramping to `0.0` across `feather` px OUTSIDE them.
///
/// The ramp is entirely outside, in the same direction [`scissor_px`] rounds: whatever
/// the card covers must be COVERED by the frost, never left as a sliver of sharp
/// document under the card's own edge. A ramp centred on the face, or reaching inward,
/// would put partly-sharp document under the card's outermost rows — a softer version
/// of the defect rather than the end of it.
///
/// MUST match `shaders/blur.wgsl`'s `footprint_mask`.
#[allow(dead_code)] // shader-mirror (see `footprint_dist_outside`).
pub(super) fn footprint_mask(foot: Footprint, feather: f32, px: f32, py: f32) -> f32 {
    let d = footprint_dist_outside(foot, px, py);
    if !d.is_finite() {
        return 0.0;
    }
    1.0 - crate::lava::smoothstep(0.0, feather.max(1.0), d)
}

/// THE SHIPPING MASK AT A CANVAS PIXEL, asked of a whole [`Frost`] — the ONE door a
/// render-tier law reads the frost's coverage through.
///
/// It exists so a law measuring pixels cannot carry its own copy of the shape, which is how
/// a law comes to grade a shape the frame stopped drawing: the retired box-union lived in
/// the mirror, the shader AND two laws' own region predicates, and both of those laws were
/// satisfied BY the defect rather than by the product. Everything here — the shear, the
/// DPI-resolved feather, the [`Frost::Full`] arm's exact `1.0` — is the value the composite
/// pass was handed.
#[cfg(test)]
pub(crate) fn footprint_mask_for(frost: Frost, dpi: f32, px: f32, py: f32) -> f32 {
    match frost {
        Frost::Full => 1.0,
        Frost::Footprint(f) => footprint_mask(f, footprint_feather_px(dpi), px, py),
    }
}

/// THE SHAPE'S RAKING FACE at row `py`, in physical px — its LEFT face for `side < 0` and
/// its RIGHT face for `side > 0`, both displaced by the same `shear × (py − cy)`.
///
/// It exists for the same reason [`footprint_mask_for`] does, and it was earned the same
/// way. A render-tier law that profiles the frost's edge has to know where that edge IS at
/// each row, and the one that does carried its OWN copy of the answer — spelled
/// `min(0, shear × (py − cy))` on the left face and `max(0, …)` on the right, which is the
/// retired box-UNION's boundary, where each face moved on only the half of the card the rake
/// reached toward. It stayed spelled that way after the shape stopped being a union, so the
/// law was profiling a face up to `|shear| × h/2` from the drawn one on half of every leaning
/// card and reporting a soft edge where there might have been a knife. One owner, so the face
/// a law measures across cannot part company with the face the shader drew.
#[cfg(test)]
pub(crate) fn footprint_face_x(foot: Footprint, py: f32, side: f32) -> f32 {
    let [x, y, w, h] = foot.rect;
    let lean = foot.shear * (py - (y + h * 0.5));
    if side < 0.0 { x + lean } else { x + w + lean }
}

/// THE BOX THAT ENCLOSES THE WHOLE FEATHERED SHAPE, `[x, y, w, h]` in physical px —
/// what the composite's scissor is set to, and the one place the shear's and the
/// feather's own reach beyond the card box is arithmetic rather than a guess.
///
/// The feather adds its own width on every face, and the leaning term displaces
/// horizontally by the shear times the distance from the box's centre. ⚠️ **That
/// distance is `h / 2` PLUS THE FEATHER, not `h / 2`** — the skirt above and below the
/// box is part of the shape, and it is displaced by the shear too, so the shear's reach
/// compounds with the feather's rather than sitting beside it. A bound derived from
/// `h / 2` alone is tight by `|shear| * feather` and clips the skirt's two far corners
/// into a hard edge at the scissor's own boundary — which is what
/// `the_footprint_bound_encloses_the_whole_feathered_shape` caught, on the first run,
/// having been written to sample the bound itself rather than a point comfortably past it.
pub(super) fn footprint_bound(foot: Footprint, feather: f32) -> [f32; 4] {
    let [x, y, w, h] = foot.rect;
    let f = feather.max(1.0);
    let g = if foot.shear.is_finite() {
        (foot.shear * (h * 0.5 + f)).abs()
    } else {
        0.0
    };
    [x - g - f, y - f, w + 2.0 * (g + f), h + 2.0 * f]
}

/// Does a CRISP picker over this list composition need its OWN FOOTPRINT frosted?
///
/// True exactly when the composition puts no CARD PANEL under the whole box — asked of
/// the roster's own backing owner, `ListStyle::list_backing`, rather than of a named
/// world, so a new composition enrols (or doesn't) by what it actually draws, and a
/// world that changes its list style changes its answer here with it. A `Pane` world is
/// excluded by its panel and stays byte-identical.
///
/// `Bars` DOES draw an opaque plate under each row — but the plate covers only that
/// row's own footprint, never the GAPS between plates or the margin around them, and
/// an unfrosted document bleeds through exactly those gaps. Each row's own plate still
/// draws OPAQUE, on top of the frost, at its own
/// true colour — the "crisp live-colour preview" promise every enrolled world keeps is
/// about what a ROW wears, never about what the space between rows shows, so enrolling
/// `Bars` costs that promise nothing.
///
/// `spell` is `false` because a crisp picker is never the contextual spell popup (that
/// one is not `overlay_crisp` and recedes nothing).
pub fn footprint_frost_applies(style: crate::theme::ListStyle) -> bool {
    !matches!(style.list_backing(false), crate::theme::ListBacking::Card)
}

/// The SCISSOR rect for a footprint, in physical px, clamped to a `width`×`height`
/// target — or `None` when the footprint lands entirely off the target (nothing to
/// composite).
///
/// Rounds OUTWARD (floor the near edges, ceil the far ones): a footprint whose edge
/// falls mid-pixel must be COVERED by the frost, never left as a one-pixel sliver of
/// sharp document under the card's own edge. The input is already physical (the card
/// box is built through `Metrics::px`), so there is no scale factor to apply here —
/// this fn's whole job is float-to-integer with the right rounding and clamp.
pub(super) fn scissor_px(rect: [f32; 4], width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let [x, y, w, h] = rect;
    if !(x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite()) {
        return None;
    }
    let x0 = x.floor().max(0.0).min(width as f32) as u32;
    let y0 = y.floor().max(0.0).min(height as f32) as u32;
    let x1 = (x + w).ceil().max(0.0).min(width as f32) as u32;
    let y1 = (y + h).ceil().max(0.0).min(height as f32) as u32;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some((x0, y0, x1 - x0, y1 - y0))
}

/// Cap for the doc-capture texture's LARGEST dimension (physical px). The full-res
/// capture is the single biggest transient the blur allocates, yet it only ever feeds
/// the quarter-res downsample + Gaussian — so on a genuinely-large / high-DPI surface
/// (4K/5K) the full resolution is wasted VRAM. Clamping the capture's longest side to
/// this cap sheds that waste with NO visible change (it is blurred + quarter-
/// downsampled either way). Chosen well ABOVE any normal or 2× retina surface, so it
/// only bites when the surface is truly large — every capture at or below the cap is
/// byte-identical.
pub(super) const DOC_CAPTURE_MAX: u32 = 3200;

/// The doc-capture texture size for a `width`×`height` surface. UNCHANGED at or below
/// [`DOC_CAPTURE_MAX`] (so any normal / retina surface captures full-res and stays
/// byte-identical); above it, scaled DOWN proportionally so the longest side is the
/// cap. Never below the quarter-res blur working size (so the downsample stays a
/// downsample), and never zero. The document is drawn into this texture via the shared
/// glyphon viewport (still sized to the full surface), so a smaller target simply scales
/// the whole document down to fill it — a reduced-scale capture, not a cropped one.
pub(super) fn capped_doc_size(width: u32, height: u32, ds: u32) -> (u32, u32) {
    let maxd = width.max(height);
    if maxd <= DOC_CAPTURE_MAX || maxd == 0 {
        return (width, height);
    }
    let ds = ds.max(1);
    let scale = DOC_CAPTURE_MAX as f32 / maxd as f32;
    let cw = ((width as f32 * scale).round() as u32)
        .max(width / ds)
        .max(1);
    let ch = ((height as f32 * scale).round() as u32)
        .max(height / ds)
        .max(1);
    (cw, ch)
}

/// HOW FAR PAST THE CARD'S OWN BOX this frame's frost can reach, in physical px: the
/// feather, plus the shear's own displacement on a leaning composition. Zero for
/// [`Frost::Full`] and for a frame with no frost.
///
/// One scalar, taken as the larger of the two axes' reach, because its consumer is a
/// uniform collar drawn around the card — and DERIVED from [`footprint_bound`] rather
/// than re-authored, so a law that wants to measure the page OUTSIDE the frost follows a
/// retuned feather instead of quietly becoming a reading of the skirt.
#[cfg(test)]
pub(crate) fn footprint_skirt_px(frost: Option<Frost>, dpi: f32) -> f32 {
    match frost {
        Some(Frost::Footprint(foot)) => {
            let [bx, by, ..] = footprint_bound(foot, footprint_feather_px(dpi));
            (foot.rect[0] - bx).max(foot.rect[1] - by).max(0.0)
        }
        Some(Frost::Full) | None => 0.0,
    }
}

/// THE PER-PASS UNIFORM. MUST match `U` in `shaders/blur.wgsl`.
///
/// It lives here, beside the extent policy, rather than in `super`'s GPU plumbing:
/// two of its four vectors ARE the extent — the shape's box and its shear/feather —
/// so what the composite pass is told about where it lands has one owner, in a file
/// with no wgpu in it, unit-testable without a device.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct U {
    /// Sample step in UV space: the source texel for the downsample, the quarter
    /// texel times the pass direction for each Gaussian axis. Unused by the composite.
    pub step: [f32; 4],
    /// Composite tint: `rgb` = the theme's `base_100` (LINEAR), `a` = the dim amount.
    /// Unused by the downsample / Gaussian passes.
    pub tint: [f32; 4],
    /// The footprint's box `[x, y, w, h]` in PHYSICAL px. All zero under
    /// [`Frost::Full`], which masks nothing.
    pub foot: [f32; 4],
    /// `[shear, feather_px, footprint_enabled, 0]`. `footprint_enabled == 0.0` is the
    /// full-canvas arm and returns a mask of exactly `1.0` at every pixel.
    pub mask: [f32; 4],
}

impl U {
    /// A downsample / Gaussian pass's uniform: a sample step, and nothing else it
    /// reads. The tint and the footprint are inert here (those passes never sample
    /// them), so they are zero rather than a dummy value with a name.
    pub(super) fn pass(step: [f32; 4]) -> Self {
        Self {
            step,
            tint: [0.0; 4],
            foot: [0.0; 4],
            mask: [0.0; 4],
        }
    }

    /// THE COMPOSITE PASS'S UNIFORM — the tint this arm dims toward and the whole
    /// extent it lands on, derived from one [`Frost`] so the two cannot disagree.
    pub(super) fn comp(base100_linear: [f32; 3], frost: Frost, dpi: f32) -> Self {
        let (foot, mask) = match frost {
            Frost::Full => ([0.0; 4], [0.0; 4]),
            Frost::Footprint(f) => (f.rect, [f.shear, footprint_feather_px(dpi), 1.0, 0.0]),
        };
        Self {
            step: [0.0; 4],
            tint: [
                base100_linear[0],
                base100_linear[1],
                base100_linear[2],
                frost.dim(),
            ],
            foot,
            mask,
        }
    }
}

/// Reinterpret a `#[repr(C)]` POD as bytes for an upload (the same minimal shim the
/// other pipelines use; the type is a small f32 array struct with no padding).
pub(super) fn bytes_of(u: &U) -> &[u8] {
    unsafe { core::slice::from_raw_parts((u as *const U) as *const u8, core::mem::size_of::<U>()) }
}

/// The surface the backdrop is built for: its device size and the scale factor that
/// size is expressed in. Bundled because the three travel together and mean nothing
/// apart — the DPI is what turns the blur's authored LOGICAL reach into texels, so a
/// width and height without it cannot say how far the Gaussian should carry.
///
/// This exists so `BlurBackdrop::ensure` stays under clippy's argument ceiling without
/// the first `too_many_arguments` waiver in a tree that has 103 exceptions and none of
/// that class — the ceiling was pointing at a real bundle rather than at a limit worth
/// suppressing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlurSurface {
    pub width: u32,
    pub height: u32,
    pub dpi: f32,
}
