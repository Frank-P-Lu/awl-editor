//! Document chrome: overlays, gutter, and corner readouts.

use super::*;
// ITEM 174 — the scene planner owns the candidate-row geometry every overlay
// consumer here reads (its forward/inverse row<->y arithmetic stays private to
// `crate::render::plan`); item 181 adds the shared item-row HEIGHT clamp.
pub(super) use crate::render::plan::{
    ClusterExtent, OverlayRowPlan, OverlayRowPlanInput, PlanLine, PlannedRow, RowSpan,
    fit_item_rows, plan_overlay_rows,
};

const PREFIX_HEADER: &str = "C-x";

pub(in crate::render) const MARGIN_COLUMN_GAP_CHARS: f32 = 1.5;

#[derive(Clone, Copy)]
pub(in crate::render) struct CardHalftone {
    pub density: f32,
    pub angle_rad: f32,
    pub cell_px: f32,
    pub ink: [u8; 4],
}

pub(in crate::render) fn narrowed_chamfer_px(cut_px: f32, card_w: f32, card_h: f32) -> f32 {
    let cap = card_w.min(card_h).max(0.0) * 0.40;
    cut_px.min(cap).max(0.0)
}

fn awl_card_caps_force() -> &'static Option<String> {
    static ONCE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| std::env::var("AWL_CARD_CAPS_FORCE").ok())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::render) enum FloatElevation {
    Rimmed,
    Flat,
}

/// The one per-frame decision for the shared floating-panel GPU trio. Claimants
/// describe their panel while preparing their own text/geometry; the chrome layer
/// uploads this model once, after every claimant has had a chance to contribute.
/// `None` is the parked state. This prevents an inactive surface's old "park"
/// write from erasing an active surface later in the same frame.
#[derive(Clone, Copy)]
pub(in crate::render) struct FloatPanelModel {
    rect: [f32; 4],
    elevation: FloatElevation,
    chamfer_px: f32,
    texture: Option<CardHalftone>,
}

#[allow(clippy::too_many_arguments)]
fn set_float_quads(
    shadow: &mut SelectionPipeline,
    border: &mut SelectionPipeline,
    card: &mut SelectionPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    rect: Option<[f32; 4]>,
    elevation: FloatElevation,
    chamfer_px: f32,
    texture: Option<CardHalftone>,
) {
    let one = rect.map(|r| [r]);
    set_float_quads_rects(
        shadow,
        border,
        card,
        device,
        queue,
        width,
        height,
        one.as_ref().map(|r| &r[..]).unwrap_or(&[]),
        elevation,
        chamfer_px,
        texture,
    );
}

#[allow(clippy::too_many_arguments)]
fn set_float_quads_rects(
    shadow: &mut SelectionPipeline,
    border: &mut SelectionPipeline,
    card: &mut SelectionPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    rects: &[[f32; 4]],
    elevation: FloatElevation,
    chamfer_px: f32,
    texture: Option<CardHalftone>,
) {
    shadow.set_chamfer(chamfer_px);
    border.set_chamfer(chamfer_px);
    card.set_chamfer(chamfer_px);
    match texture {
        Some(t) => card.set_halftone(t.density, t.angle_rad, t.cell_px, t.ink),
        None => card.set_halftone(0.0, 0.0, 1.0, [0; 4]),
    }
    shadow.prepare(device, queue, width, height, &[]);
    let borders: Vec<[f32; 4]> = if elevation != FloatElevation::Flat {
        rects
            .iter()
            .map(|&[x, y, w, h]| [x - 1.0, y - 1.0, w + 2.0, h + 2.0])
            .collect()
    } else {
        Vec::new()
    };
    border.prepare(device, queue, width, height, &borders);
    card.prepare(device, queue, width, height, rects);
}

/// The page-mode GUTTER's fully decided layout for one frame — see
/// [`TextPipeline::gutter_layout`]. `name` AND `project` are ALREADY fit to one
/// line each (through the single shared elision door, [`rowlayout::fit_primary`]);
/// `avail` never lays raw text into a wrapping box, so neither line can ever
/// word-wrap mid-word. `project` is `""` only when there is genuinely no project
/// to show (never as a width-pressure yield — see `gutter_layout`'s doc).
struct GutterLayout {
    avail: f32,
    name: String,
    project: String,
    /// The `changed elsewhere` line, fit to the same budget, or empty when the
    /// active document holds no unresolved external change. Drawn ABOVE the
    /// filename in the strongest of the block's three inks — the ladder is
    /// value, not colour (DESIGN.md §4: the accent belongs to the caret alone).
    changed: String,
}

pub(in crate::render) struct PanelShape {
    no_match: bool,
    ink: glyphon::Color,
    red: glyphon::Color,
    pub(in crate::render) caret_byte: usize,
    pub(in crate::render) caret_fallback_chars: usize,
    pub(in crate::render) caret_row: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelHit {
    CaseToggle,
    Find,
    Replace,
    Elsewhere,
}

/// Resolved geometry for the summoned overlay card: the row WINDOW (`visible` rows
/// from `top_idx`, `n_items` total, plus the foot `hint`/`hint_rows`), the card
/// rectangle (`card_x/y/w/h`), and the inner text origin + width
/// (`text_left/top/w`). Computed BEFORE the rows so the binding column can
/// right-align to the text width.
/// The gap between adjacent lens labels in the theme picker's strip. Kept modest so
/// the whole strip fits one line on a wide mono world face. The `All` home (strip
/// index 0) is not drawn as a label, so the strip is just the facets, gap-separated.
const STRIP_GAP: &str = "  ";

const CHIP_STRIP_GAP: &str = "    ";

pub(super) fn strip_gap() -> &'static str {
    match crate::render::effective_facet_style() {
        theme::FacetStyle::Chips(_) => CHIP_STRIP_GAP,
        theme::FacetStyle::Text | theme::FacetStyle::Band => STRIP_GAP,
    }
}

#[derive(Clone)]
pub(super) struct OverlayGeom {
    visible: usize,
    top_idx: usize,
    n_items: usize,
    hint: String,
    hint_rows: usize,
    footer: Vec<String>,
    /// Display rows the footer occupies: `0` when empty, else `footer.len() + 1` (a blank
    /// separator line between the hint and the band). The card grows by exactly this, so
    /// the hit-test / selected-row band (which only span the candidate rows above) are
    /// untouched.
    footer_rows: usize,
    theme: bool,
    strip: Vec<(String, bool)>,
    /// The GROUPED family's candidate DISPLAY-LINE sequence — section headers
    /// interleaved with item rows, built by [`TextPipeline::theme_plan`] from the
    /// parallel `overlay_sections` and windowed to what the card shows. Handed
    /// verbatim to the scene planner, so the shaper's line `k` and the planner's
    /// row `k` are the same line by construction.
    plan: Vec<PlanLine>,
    /// Rows occupied ABOVE the candidate list: `1` for the query line the flat/nav
    /// pickers show at the top (`› query`), `0` for the contextual SPELL panel (no
    /// query line — just suggestion rows). Candidate row 0 therefore begins at
    /// the scene planner's first planned row, whose slot both the selected-row band
    /// and the pointer hit-test read, so they can't drift from the shaped rows.
    pub(super) header_rows: usize,
    /// PALETTE-COMPOSITION round: extra VERTICAL negative space (device px)
    /// inserted AFTER the header rows (the `› query` line, plus the lens strip on
    /// a faceted card) and BEFORE the candidate list — the calm "divider" that
    /// separates chrome from the list without a drawn rule. `0.0` for the
    /// contextual spell popup (no header to divide from). The candidate band, the
    /// selected-row highlight, the pointer hit-test, and the card height all fold
    /// it in through the scene planner, so they can't drift; the shaper realizes it
    /// either by inflating the last header line's height by exactly this, or — when
    /// that line is the query field itself — as its own glyph-free line
    /// (`plan::beat_stands_alone`, and [`OverlayGeom::shaped_first_row_line`]).
    pub(super) header_gap: f32,
    empty: Option<String>,
    card_x: f32,
    pub(super) card_y: f32,
    card_w: f32,
    pub(super) card_h: f32,
    pub(super) text_left: f32,
    pub(super) text_top: f32,
    pub(super) text_w: f32,
    card_narrow: bool,
    /// ITEM 114 — this card is drawn as a SUMMONED WORKSPACE. `false` for every
    /// contextual card, which keeps every arm reading it byte-identical there.
    /// The family's own doc is `render/chrome/workspace.rs`.
    pub(super) workspace: bool,
    /// The navigation RAIL's COLUMN (`[x, w]`), or `None` when no LABEL rail is
    /// drawn — off a workspace, on the narrow detail stage, or (item 116a) on
    /// a shape whose primary column carries rows instead of labels. Only the
    /// column: its vertical grid comes from the ROW PLAN's band origin
    /// (`workspace_rail_box`), so a rail entry and the row beside it share a line.
    pub(super) rail: Option<[f32; 2]>,
    /// The CONTENT BAND's horizontal extent — read through `band_x`/`band_w`,
    /// never directly, so a contextual card gets its card and a workspace its
    /// pane from one owner.
    pub(super) pane_x: f32,
    pub(super) pane_w: f32,
    /// Does the workspace's CONTENT pane hold focus (rather than its rail)? The
    /// one input to the focus cue. Always `false` off a workspace.
    pub(super) rows_focused: bool,
}

impl OverlayGeom {
    fn base() -> Self {
        OverlayGeom {
            visible: 0,
            top_idx: 0,
            n_items: 0,
            hint: String::new(),
            hint_rows: 0,
            footer: Vec::new(),
            footer_rows: 0,
            theme: false,
            strip: Vec::new(),
            plan: Vec::new(),
            header_rows: 0,
            header_gap: 0.0,
            empty: None,
            card_x: 0.0,
            card_y: 0.0,
            card_w: 0.0,
            card_h: 0.0,
            text_left: 0.0,
            text_top: 0.0,
            text_w: 0.0,
            card_narrow: false,
            workspace: false,
            rail: None,
            pane_x: 0.0,
            pane_w: 0.0,
            rows_focused: false,
        }
    }

    /// WHICH `panel_buffer` LINE CARRIES CANDIDATE ROW 0 — the one owner every
    /// reader of a shaped candidate line goes through.
    ///
    /// It is NOT always `header_rows`: when the query BEAT stands on its own
    /// (`plan::beat_stands_alone`) the shaper emits a glyph-free line for it
    /// between the header and the candidates, because cosmic-text centres a
    /// line's glyphs in its box and a beat folded into the query field's own line
    /// would drop the field's glyphs half a beat below its bar. A reader that
    /// re-derived `header_rows` for itself would then measure the spacer as a
    /// candidate row and every row width would land one line off.
    /// TEST PROBE — the footer band's own row counts, for a law reconstructing
    /// a retired plate rule inline.
    #[cfg(test)]
    pub(in crate::render) fn hint_rows_probe(&self) -> usize {
        self.hint_rows
    }
    #[cfg(test)]
    pub(in crate::render) fn footer_rows_probe(&self) -> usize {
        self.footer_rows
    }

    /// TEST PROBE — the hand-written header-row count a surface-contract law
    /// cross-checks its own table against.
    #[cfg(test)]
    pub(in crate::render) fn header_rows_probe(&self) -> usize {
        self.header_rows
    }

    pub(super) fn shaped_first_row_line(&self) -> usize {
        self.header_rows
            + usize::from(crate::render::plan::beat_stands_alone(
                self.header_rows,
                self.header_gap,
            ))
    }
}

// The chrome cluster is decomposed into cohesive per-subsystem submodules; each
// carries its own `impl TextPipeline { .. }` block (Rust merges the inherent impls
// across the module tree). This file keeps the SHARED items every submodule needs —
// the panel/overlay geometry structs, the float-quad primitive, the overlay row<->Y
// owner and the sidecar report structs.
pub(in crate::render) mod diagonal;
mod overlay;
mod overlay_clamp;
mod panel;
// ITEM 114 — the SUMMONED WORKSPACE family: geometry, navigation rail, hit-test.
// ITEM 116b — its two regions' shared box arithmetic, and the RELOCATED
// DOCUMENT VIEWPORT one of them can become (`comparison_viewport`).
mod comparison;
mod workspace;
mod workspace_column;
pub(in crate::render) use overlay::OVERLAY_UI_SCALE;
/// TEST PROBE — the workspace's own interior pad, which the retired footer-plate
/// rule added on top of a full row (item 225's law reconstructs that expression).
#[cfg(test)]
pub(in crate::render) const WORKSPACE_PAD_PROBE: f32 = workspace::WORKSPACE_PAD;
#[cfg(test)]
pub(in crate::render) use overlay::{
    CARD_EDGE_INSET_FLOOR, CARD_MAX_W, CARD_MAX_W_FACETED, overlay_card_box_policy,
    overlay_card_fill_regime, overlay_rail_inset,
};
// The card-DRAW half of the summoned overlay (shape + upload + composite): the
// geometry/hit-test owner is `overlay`, this turns that settled geometry into GPU
// work. A cohesive physical carve, byte-identical pixels — see the file's own doc.
mod overlay_draw;
mod overlay_rows;
mod overlay_selection;
#[cfg(test)]
mod overlay_selection_probe;
mod overlay_shape;
// ITEM 164 — the ONE visual-selection transaction every selected visual reads.
mod overlay_visual_sel;
#[cfg(test)]
pub(in crate::render) use overlay_shape::snap_placard_size;
pub(in crate::render) use overlay_visual_sel::{
    VisualSelection, overlay_selected_primary_ink, overlay_selected_secondary_ink,
    overlay_selected_secondary_srgb,
};
mod gutter;
mod menubar;
mod outline;
mod theme_picker;
#[cfg(test)]
pub(in crate::render) use outline::OutlineRow;
#[cfg(test)]
pub(in crate::render) use outline::OutlineRung;
mod card;
mod debug_text;
mod hud;
mod popover;
mod preview;
mod readout;
mod whichkey;
#[cfg(test)]
pub(crate) use popover::VPAD as POPOVER_VPAD;
#[allow(unused_imports)] // PopoverButtonGeom named only inside the popover module
pub(in crate::render) use popover::{PopoverButtonGeom, PopoverGeom};

impl TextPipeline {
    /// Claim a small, summoned, transient FLOATING PANEL. The shared trio is
    /// uploaded by [`Self::flush_float_panel`] once at the end of chrome
    /// preparation; callers never park it directly.
    /// bordered box with CARD ELEVATION (a crisp raised BORDER edge + the opaque
    /// CARD — no drop shadow, see [`FloatElevation`]'s doc), and crucially NO
    /// scrim — so it floats over the live document without dimming it, distinct
    /// from the full-width takeover overlay. `rect = Some([x, y, w, h])` summons
    /// it; `None` parks both elevation quads empty (nothing drawn). `elevation`
    /// picks the dressing ([`FloatElevation`]) — the caret-style preview panel,
    /// the spell popup, AND the format popover all ride the same RIMMED style
    /// (border + card, no shadow slab — see [`Self::prepare_popover`]'s "fat
    /// chin" note, the decision this round's `Shadowed`/`Rimmed` merge generalized).
    ///
    /// THE ONE FLOAT-SURFACE OWNER (overlay/chrome polish round): every summoned
    /// micro-panel that wants this "small floating card, no scrim" language routes
    /// through here — the caret-style preview panel, the search panel, the
    /// contextual SPELL popup, AND the format popover — onto the SAME
    /// `float_shadow`/`float_border`/`float_card` quads, never a per-feature
    /// duplicate trio. `set_float_quads` (the underlying quad math) stays a
    /// private fn of this module — this is its ONLY door (law-tested,
    /// `float_surface_primitive_has_no_bypass_among_the_unified_family`), so a
    /// future micro-panel in this family can't accidentally reinvent the call
    /// inline (the popover used to).
    ///
    /// SAFE to share one buffer set because the four callers are STRUCTURALLY
    /// mutually exclusive (`viewstate.rs` gates the popover on
    /// `WorkspaceState::pickers_clear`; the preview panel and the spell popup
    /// are two different `OverlayKind`s; the search panel requires
    /// `search_active`, itself exclusive with any overlay) — each call site
    /// parks (`rect: None`) on every frame it isn't the active one. That alone
    /// is NOT sufficient, though: with four INDEPENDENTLY gated calls each
    /// unconditionally touching the buffer every frame, whichever call happens
    /// to run LAST always wins — including a "closed" park call from a feature
    /// that just isn't active this frame, which would erase a genuinely real
    /// one prepared earlier (the bug a first draft of this round's popover
    /// merge shipped, caught by
    /// `caret_preview_panel_appears_below_picker_and_stops_on_close`). The fix
    /// is NOT calling order — it's [`Self::prepare_popover`]'s own GUARD (see
    /// its doc): it only ever touches these quads when `!overlay_active &&
    /// !search_active`, i.e. only in the one frame-state where popover is
    /// structurally allowed to be the real owner, so its park call can never
    /// race the caret-preview panel / spell popup / search panel. "Summoned,
    /// not furniture" (DESIGN §5).
    ///
    /// `chamfer_px`/`texture` (item 70): `0.0`/`None` for every non-card
    /// caller (the caret-style preview panel, the search panel, the format
    /// popover — byte-identical); the SPELL POPUP arm of `overlay_draw_card`
    /// is the one caller that ever passes a real chamfer/texture (Quokka's
    /// "small card popup").
    #[allow(clippy::too_many_arguments)]
    pub(super) fn claim_float_panel(
        &mut self,
        rect: [f32; 4],
        elevation: FloatElevation,
        chamfer_px: f32,
        texture: Option<CardHalftone>,
    ) {
        debug_assert!(
            self.float_panel_model.is_none(),
            "two float-panel claimants must be structurally exclusive"
        );
        self.float_panel_model = Some(FloatPanelModel {
            rect,
            elevation,
            chamfer_px,
            texture,
        });
    }

    pub(super) fn begin_float_panel_frame(&mut self) {
        self.float_panel_model = None;
    }

    pub(super) fn flush_float_panel(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let model = self.float_panel_model;
        set_float_quads(
            &mut self.float_shadow,
            &mut self.float_border,
            &mut self.float_card,
            device,
            queue,
            width,
            height,
            model.map(|m| m.rect),
            model.map(|m| m.elevation).unwrap_or(FloatElevation::Rimmed),
            model.map(|m| m.chamfer_px).unwrap_or(0.0),
            model.and_then(|m| m.texture),
        );
    }

    /// **THE DOCUMENT CONTENT CLIP's VERTICAL HALF (item 84, re-owned by item
    /// 116b).** The band (`(top, bottom)` in px) document content may paint
    /// into, or `None` on an ordinary frame (the whole canvas, no clipping).
    ///
    /// Item 84 derived it from the diff-preview panel's own card rect — the one
    /// place a document ever drew somewhere other than the canvas. Item 116b
    /// replaced that composition with a real relocated viewport, so the band is
    /// now [`Self::comparison_viewport`]'s own vertical extent and the LAW is
    /// unchanged: applied at every content emitter — the text layer's
    /// `TextBounds`, the wash / pill / fence-panel quads, the strike / squiggle
    /// lines, the ornament glyphs, and the caret quad — so a scrolled
    /// comparison clips AT the region's edge instead of sliding over the
    /// workspace around it.
    pub(in crate::render) fn doc_clip_band(&self) -> Option<(f32, f32)> {
        self.comparison_viewport().map(|[_, y, _, h]| (y, y + h))
    }

    pub(super) fn prepare_panel_card_elevation(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rects: &[[f32; 4]],
    ) {
        let card_elevation = crate::render::effective_card_elevation();
        self.panel_card
            .set_color(theme::pane_surface(card_elevation).rgba_bytes());
        let elevation = if !rects.is_empty() && card_elevation == theme::Elevation::Bordered {
            FloatElevation::Rimmed
        } else {
            FloatElevation::Flat
        };
        let (chamfer_px, texture) = self.card_shape_texture(rects);
        set_float_quads_rects(
            &mut self.panel_shadow,
            &mut self.panel_border,
            &mut self.panel_card,
            device,
            queue,
            width,
            height,
            rects,
            elevation,
            chamfer_px,
            texture,
        );
    }

    pub(super) fn card_shape_texture(&self, rects: &[[f32; 4]]) -> (f32, Option<CardHalftone>) {
        let mut caps = theme::active().render_caps;
        // DEV-ONLY GALLERY PROBE (mirrors `AWL_CJK_FORCE`'s "total no-op unless
        // set" contract — no config key, no CLI flag): `AWL_CARD_CAPS_FORCE`
        // stages Quokka's own printed-card caps down for the round's
        // "current / type-only / type+halftone / full-chamfered" capture
        // sequence, so each stage is a REAL render of the shipped mechanism
        // rather than a synthetic mockup. `"flat"` forces `Flat`/`Rectangular`
        // (the font-only stage); `"halftone"` keeps the world's own texture but
        // forces `Rectangular` (texture, no chamfer yet). Unset (every normal
        // run) is a no-op — the active world's own data renders untouched.
        if let Some(force) = awl_card_caps_force() {
            match force.as_str() {
                "flat" => {
                    caps.card_texture = theme::CardTexture::DEFAULT;
                    caps.card_shape = theme::CardShape::DEFAULT;
                }
                "halftone" => {
                    caps.card_shape = theme::CardShape::DEFAULT;
                }
                _ => {}
            }
        }
        let chamfer_px = match caps.card_shape {
            theme::CardShape::Rectangular => 0.0,
            theme::CardShape::Chamfered { cut_px } => {
                let physical_cut = cut_px * self.dpi.max(1.0);
                rects
                    .iter()
                    .map(|&[_, _, w, h]| narrowed_chamfer_px(physical_cut, w, h))
                    .fold(f32::INFINITY, f32::min)
                    .min(physical_cut)
                    .max(0.0)
            }
        };
        let texture = match caps.card_texture {
            theme::CardTexture::Flat => None,
            theme::CardTexture::HalftoneDots {
                angle_deg,
                cell_px,
                density,
            } => Some(CardHalftone {
                density,
                angle_rad: angle_deg.to_radians(),
                cell_px: cell_px * self.dpi.max(1.0),
                ink: theme::card_texture_ink().rgba_bytes(),
            }),
        };
        (chamfer_px, texture)
    }
}

fn preview_glyph_key_at(buf: &GlyphBuffer, text: &str, idx: usize) -> Option<CacheKey> {
    let byte = text
        .char_indices()
        .nth(idx)
        .map(|(b, _)| b)
        .unwrap_or(text.len());
    if byte >= text.len() {
        return None;
    }
    for run in buf.layout_runs() {
        for g in run.glyphs.iter() {
            if byte >= g.start && byte < g.end {
                return Some(g.physical((0.0, 0.0), 1.0).cache_key);
            }
        }
    }
    None
}

pub(super) const BAR_SIDE_INSET: f32 = 8.0;

pub(super) const BAR_TEXT_PAD: f32 = 13.0;

pub(super) const INLINE_SHORTCUT_GAP: &str = "   ";

pub(super) fn bars_inline_shortcut() -> bool {
    matches!(
        crate::render::effective_list_style(),
        theme::ListStyle::Bars { extent, .. } if extent.inline_shortcut()
    )
}

pub(super) fn bar_full_span(card_x: f32, card_w: f32) -> (f32, f32) {
    (
        card_x + BAR_SIDE_INSET,
        (card_w - 2.0 * BAR_SIDE_INSET).max(1.0),
    )
}

pub(super) fn bar_hug_span(
    card_x: f32,
    card_w: f32,
    text_left: f32,
    primary_px: f32,
) -> (f32, f32) {
    let (x, full_w) = bar_full_span(card_x, card_w);
    let full_right = x + full_w;
    let right = (text_left + primary_px + BAR_TEXT_PAD).min(full_right);
    (x, (right - x).max(1.0))
}

pub(super) fn grow_span(x: f32, w: f32, grow: f32, mirror: bool) -> (f32, f32) {
    let g = grow.max(0.0);
    if mirror {
        let left = (x - g).max(0.0);
        (left, x + w - left)
    } else {
        (x, w + g)
    }
}

/// PURE geometry (SLANT-ON-BARS) — a bar's `(x, w)` extended by its display
/// row's two-sided extent `(dx, dw)` (item 131a). A `hug` plate (never at the
/// card's right edge) simply translates by `dx` — `dw` cannot mean anything to
/// it, since a hug plate's own width already comes from its measured content,
/// not from the band's width, and a right-hugging mirror plate is a
/// COMPOSITION concern (item 131c/d), not this primitive's. A FULL-WIDTH plate
/// keeps whichever edge is untouched flush and sheds the other: `dx` steps the
/// left edge in with the right held at `x + w` (mirroring the Pane band's
/// `[card_x + dx, w - dx]`), `dw` steps the right edge in with the left held at
/// `x` — so a slanted full-width bar, mirrored either direction, can never
/// paint past the card. `dx == 0.0 && dw == 0.0` (the unslanted default, or a
/// fan-in at rest) → the input span verbatim (byte-identical). The ONE owner
/// both the unselected and selected slanted plates read, so the two extents
/// cascade identically.
pub(super) fn slant_bar_span(x: f32, w: f32, hug: bool, dx: f32, dw: f32) -> (f32, f32) {
    if dx == 0.0 && dw == 0.0 {
        return (x, w);
    }
    if hug {
        (x + dx, w)
    } else {
        (x + dx, (w + dw - dx).max(1.0))
    }
}

#[cfg(test)]
pub(super) fn bar_rect_unselected(card_x: f32, card_w: f32, top: f32, bar_h: f32) -> [f32; 4] {
    let (x, w) = bar_full_span(card_x, card_w);
    [x, top, w, bar_h]
}

#[cfg(test)]
pub(super) fn bar_rect_selected(
    card_x: f32,
    card_w: f32,
    top: f32,
    bar_h: f32,
    grow_px: f32,
    mirror: bool,
) -> [f32; 4] {
    let (bx, bw) = bar_full_span(card_x, card_w);
    let (x, w) = grow_span(bx, bw, grow_px, mirror);
    [x, top, w.max(1.0), bar_h]
}

/// The `Bars` footer PLATE, seated at the planned footer top
/// ([`crate::render::plan::OverlayRowPlan::footer_top`]) — never at a row index
/// this function re-derives, so the plate and the footer glyphs it backs cannot
/// land on different rows.
pub(super) fn footer_plate_rect(
    hint_top: f32,
    card_x: f32,
    card_w: f32,
    card_bottom: f32,
    hug: Option<(f32, f32)>,
) -> [f32; 4] {
    let (x, w) = match hug {
        Some((text_left, content_px)) => bar_hug_span(card_x, card_w, text_left, content_px),
        None => bar_full_span(card_x, card_w),
    };
    [x, hint_top, w, (card_bottom - hint_top).max(1.0)]
}

pub(super) fn field_caret_byte(text: &str, caret_char: usize) -> usize {
    if caret_char == 0 {
        return 0;
    }
    text.char_indices()
        .nth(caret_char)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

/// ITEM 80 — THE ONE FIXED-WIDTH FIELD RULE: given a panel VALUE field's FULL
/// `text` and its CHAR-index `caret_char`, returns `(view, view_caret)` — a
/// WINDOW of EXACTLY `cap` chars that always contains the caret. Typing or
/// pasting past `cap` chars into the find query / replace text used to widen
/// the panel (`panel_layout` sized the card off the panel_buffer's ACTUAL
/// shaped width, which grew unbounded with the query); every consumer that
/// promises a FIXED exterior geometry (today: [`TextField::FindQuery`],
/// [`TextField::ReplaceText`] — see `textbox.rs`'s `TextField::ALL` law) must
/// shape THIS window, never the raw field text, so the reserved column is
/// always exactly `cap` cells regardless of what's typed.
///
/// `text` shorter than `cap` is RIGHT-PADDED with spaces to the full `cap` —
/// not left as its own (shorter) length — so a three-letter query reserves the
/// SAME width as an empty one; only a `text` longer than `cap` SCROLLS,
/// windowed to the `cap` chars ending at the caret once the caret has advanced
/// past the cap (so typing/pasting at the end — the common case — keeps
/// revealing the newest chars), or the leading `cap` chars while the caret
/// hasn't reached the cap yet (Home, or a fresh short field). The caret is
/// ALWAYS inside `[0, cap]` in the returned view — never lost off either edge,
/// regardless of typing, Backspace, word-delete, Home/End, or a mouse-placed
/// caret; the SAME rule for both the find query and the replace text (there is
/// no second, field-specific version of this scroll).
///
/// Callers MUST shape `view` in a MONOSPACE family ([`Family::Monospace`]):
/// the fixed-CHAR-COUNT contract above only yields a fixed PIXEL width when
/// every cell advances the same amount — a proportional face would let "cat"
/// and "WWW" occupy the same `cap` chars but different pixels, reopening this
/// exact bug one level down.
pub(super) fn field_view_window(text: &str, caret_char: usize, cap: usize) -> (String, usize) {
    if cap == 0 {
        return (String::new(), 0);
    }
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let caret_char = caret_char.min(len);
    if len <= cap {
        let mut view: String = chars.into_iter().collect();
        view.extend(std::iter::repeat_n(' ', cap - len));
        return (view, caret_char);
    }
    let start = caret_char.saturating_sub(cap);
    let view: String = chars[start..start + cap].iter().collect();
    (view, caret_char - start)
}

pub(super) fn scroll_window(
    len: usize,
    sel: usize,
    scroll_hint: usize,
    max: usize,
) -> (usize, usize) {
    let count = len.min(max);
    if count == 0 {
        return (0, 0);
    }
    let mut top = scroll_hint;
    if sel < top {
        top = sel;
    } else if sel >= top + count {
        top = sel + 1 - count;
    }
    // Clamp so the window never runs past the end (`len >= count`, so this can't wrap).
    top = top.min(len - count);
    (top, count)
}

pub struct HudReport {
    pub held: bool,
    pub words: Option<(usize, usize)>,
    pub percent: u32,
    pub lang: Option<crate::frontmatter::Lang>,
    pub eol: crate::buffer::Eol,
    pub saved: String,
}

pub struct LifetimeReport {
    pub open: bool,
    pub chars: String,
    pub writing: String,
    pub files: String,
    pub caret_travel: String,
    pub world: String,
}

pub struct PeekReport {
    pub open: bool,
    pub rows: Vec<crate::peek::PeekRow>,
}

pub struct StreaksReport {
    pub open: bool,
    pub view: &'static str,
    pub streak: u64,
    pub today_words: u64,
    pub total_words: u64,
    pub cells: Vec<u8>,
}

pub struct DebugPerfReport {
    pub frame_ms: Option<f32>,
    pub worst_ms: Option<f32>,
    pub budget_ms: Option<f32>,
    pub key_px_ms: Option<f32>,
    pub redraws: Option<u64>,
    pub still: bool,
    pub autosave: Option<crate::debug::AutosaveState>,
}

#[cfg(test)]
mod window_tests {
    use super::scroll_window;

    #[test]
    fn caps_the_window_at_max_and_shows_all_when_it_fits() {
        assert_eq!(scroll_window(5, 0, 0, 12), (0, 5));
        assert_eq!(scroll_window(12, 3, 0, 12), (0, 12));
        assert_eq!(scroll_window(100, 0, 0, 12), (0, 12));
        assert_eq!(scroll_window(100, 5, 5, 12).1, 12);
    }

    #[test]
    fn slides_the_minimum_to_keep_the_selection_visible() {
        assert_eq!(scroll_window(100, 2, 20, 12), (2, 12));
        assert_eq!(scroll_window(100, 40, 0, 12), (40 + 1 - 12, 12));
        assert_eq!(scroll_window(100, 25, 20, 12), (20, 12));
    }

    #[test]
    fn selection_is_always_within_the_returned_window() {
        for len in [1usize, 3, 12, 13, 50, 200] {
            for sel in [0usize, 1, len / 2, len.saturating_sub(1)] {
                if sel >= len {
                    continue;
                }
                for hint in [0usize, sel, len, len / 3, sel.saturating_sub(3)] {
                    let (top, count) = scroll_window(len, sel, hint, 12);
                    assert!(count <= 12 && count <= len, "count bounded (len {len})");
                    assert!(
                        sel >= top && sel < top + count,
                        "sel {sel} in [{top}, {}), len {len} hint {hint}",
                        top + count
                    );
                    assert!(top + count <= len, "window in range (len {len})");
                }
            }
        }
    }

    #[test]
    fn matches_the_prior_inline_flat_math_when_the_hint_already_keeps_sel_visible() {
        for n in [0usize, 4, 12, 30] {
            for max in [8usize, 12] {
                let visible = n.min(max);
                for sel in 0..n {
                    let hint = sel.saturating_sub(max - 1).min(n.saturating_sub(visible));
                    let expected = (hint.min(n.saturating_sub(visible)), visible);
                    assert_eq!(
                        scroll_window(n, sel, hint, max),
                        expected,
                        "n {n} max {max} sel {sel}"
                    );
                }
            }
        }
    }

    #[test]
    fn empty_list_yields_an_empty_window() {
        assert_eq!(scroll_window(0, 0, 0, 12), (0, 0));
    }
}

#[cfg(test)]
mod field_view_window_tests {
    use super::field_view_window;

    #[test]
    fn short_text_is_right_padded_to_the_full_cap() {
        let (view, caret) = field_view_window("cat", 3, 8);
        assert_eq!(view, "cat     "); // 3 + 5 spaces = 8
        assert_eq!(view.chars().count(), 8);
        assert_eq!(caret, 3);
    }

    #[test]
    fn empty_text_is_all_padding_caret_at_zero() {
        let (view, caret) = field_view_window("", 0, 5);
        assert_eq!(view, "     ");
        assert_eq!(caret, 0);
    }

    #[test]
    fn caret_past_the_text_end_is_clamped_before_windowing() {
        let (view, caret) = field_view_window("ab", 99, 5);
        assert_eq!(view, "ab   ");
        assert_eq!(caret, 2, "clamped to the text's own length");
    }

    #[test]
    fn text_exactly_at_the_cap_needs_no_padding_or_scroll() {
        let (view, caret) = field_view_window("abcdefgh", 8, 8);
        assert_eq!(view, "abcdefgh");
        assert_eq!(caret, 8, "one past the last char — a valid landing cell");
    }

    #[test]
    fn long_text_caret_at_the_end_scrolls_to_the_trailing_cap_chars() {
        let text = "abcdefghijklmnopqrstuvwxyz01"; // 28 chars
        let (view, caret) = field_view_window(text, 28, 8);
        assert_eq!(view, "uvwxyz01", "the last 8 chars");
        assert_eq!(caret, 8, "pinned at the window's own trailing edge");
    }

    #[test]
    fn long_text_caret_at_the_start_shows_the_leading_cap_chars_unscrolled() {
        let text = "abcdefghijklmnopqrstuvwxyz01";
        let (view, caret) = field_view_window(text, 0, 8);
        assert_eq!(view, "abcdefgh");
        assert_eq!(caret, 0);
    }

    #[test]
    fn scrolling_by_one_char_slides_the_window_by_one_char() {
        let text = "0123456789"; // 10 chars, cap 4 -> scrolls once caret > 4
        assert_eq!(field_view_window(text, 4, 4), ("0123".to_string(), 4));
        assert_eq!(field_view_window(text, 5, 4), ("1234".to_string(), 4));
        assert_eq!(field_view_window(text, 6, 4), ("2345".to_string(), 4));
        assert_eq!(field_view_window(text, 10, 4), ("6789".to_string(), 4));
    }

    #[test]
    fn multibyte_text_windows_by_char_not_byte() {
        let text = "日本語のテキスト検索ですよ"; // 13 chars
        let (view, caret) = field_view_window(text, 13, 6);
        assert_eq!(view.chars().count(), 6);
        assert_eq!(caret, 6);
        assert_eq!(
            view, "ト検索ですよ",
            "the trailing 6 chars, caret at the end"
        );
    }

    #[test]
    fn the_view_is_always_exactly_cap_chars_and_the_caret_is_always_inside_it() {
        let texts: Vec<String> = vec![
            String::new(),
            "a".to_string(),
            "a".repeat(7),
            "a".repeat(8),
            "a".repeat(9),
            "a".repeat(50),
        ];
        for text in &texts {
            let len = text.chars().count();
            for caret in [0, len / 2, len, len + 5] {
                for cap in [1usize, 4, 8, 16] {
                    let (view, view_caret) = field_view_window(text, caret, cap);
                    assert_eq!(
                        view.chars().count(),
                        cap,
                        "len {len} caret {caret} cap {cap}: view must be exactly cap chars"
                    );
                    assert!(
                        view_caret <= cap,
                        "len {len} caret {caret} cap {cap}: caret {view_caret} must be inside [0, cap]"
                    );
                }
            }
        }
    }

    #[test]
    fn cap_zero_is_an_inert_empty_field() {
        assert_eq!(field_view_window("anything", 3, 0), (String::new(), 0));
    }

    #[test]
    fn every_fixed_geometry_textfield_routes_through_the_one_clipping_rule() {
        use crate::textbox::TextField;
        for f in TextField::ALL {
            let promises_fixed_geometry = match f {
                TextField::FindQuery => true,
                TextField::ReplaceText => true,
                TextField::PickerQuery => false,
                TextField::Rename => false,
                TextField::InsertLink => false,
                TextField::KeepVersion => false,
                TextField::SettingsValue => false,
            };
            if !promises_fixed_geometry {
                continue;
            }
            for cap in [1usize, 8, 28] {
                for (text, caret) in [
                    ("", 0usize),
                    ("hi", 2),
                    (
                        "a very long value that overflows any realistic field width by a lot",
                        71,
                    ),
                    ("mid caret text", 4),
                ] {
                    let (view, view_caret) = field_view_window(text, caret, cap);
                    assert_eq!(
                        view.chars().count(),
                        cap,
                        "{f:?} cap {cap} text {text:?}: the fixed field must always be exactly `cap` chars"
                    );
                    assert!(
                        view_caret <= cap,
                        "{f:?} cap {cap} text {text:?}: the caret must always land inside the fixed field"
                    );
                }
            }
        }
    }
}
