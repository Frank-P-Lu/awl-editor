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
//! The footprint is scoped with a SCISSOR RECT (`BlurBackdrop::draw_backdrop`), not
//! a rect uniform: the composite is a fullscreen triangle whose fragments outside the
//! card must not be written AT ALL, which is precisely a scissor's job — one API
//! call, no shader branch, and the pixels inside the rect stay bit-for-bit the
//! fullscreen composite's. A rect uniform would need a `discard` branch in `fs_comp`,
//! the rect plumbed through `U`, and would produce the same hard edge anyway.
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
    /// The summoned card's own box `[x, y, w, h]` in PHYSICAL px (the same canvas
    /// coordinates the card is drawn at), and nothing outside it.
    Footprint([f32; 4]),
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

/// Does a CRISP picker over this list composition need its OWN FOOTPRINT frosted?
///
/// True exactly when the composition puts NOTHING OPAQUE between its rows and the
/// document: no card panel under the whole box, and no plate under the rows. Asked of
/// the roster's own two backing owners — `ListStyle::list_backing` and
/// `ListStyle::draws_row_plates` — rather than of a named world, so a new
/// composition enrols (or doesn't) by what it actually draws, and a world that changes
/// its list style changes its answer here with it. A `Pane` world is excluded by its
/// panel and a `Bars` world by its plates; both stay byte-identical.
///
/// `spell` is `false` because a crisp picker is never the contextual spell popup (that
/// one is not `overlay_crisp` and recedes nothing).
pub fn footprint_frost_applies(style: crate::theme::ListStyle) -> bool {
    !matches!(style.list_backing(false), crate::theme::ListBacking::Card)
        && !style.draws_row_plates()
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
