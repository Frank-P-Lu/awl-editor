//! Shared overlay sizing, anchoring, and narrow-hint policy.

use super::*;

pub(in crate::render) const OVERLAY_UI_SCALE: f32 = 0.85;
pub(in crate::render) const CARD_EDGE_INSET_FLOOR: Logical = Logical(10.0);
pub(in crate::render) const CARD_MAX_W: LogicalGrowOnly = LogicalGrowOnly(545.0);
pub(in crate::render) const CARD_MAX_W_FACETED: LogicalGrowOnly = LogicalGrowOnly(600.0);
pub(in crate::render) const CARD_CONTENT_MIN_W: LogicalGrowOnly = LogicalGrowOnly(160.0);
pub(in crate::render) const OVERLAY_QUERY_BEAT: Rows = Rows(1.55);
/// TASTE CALL, unified-pane worlds only (`ListStyle::Pane` + `PaneSplit::
/// Unified` — one continuous card surface, no seam splitting the query field
/// from the list, `Cassowary` the only shipping member today). On a SPLIT
/// pane or a plated world (Bars/Diagonal/Ruled) the beat sits inside a seam
/// or between occupied rows and reads as a considered divider; inside one
/// unbroken plate the same `OVERLAY_QUERY_BEAT` reads as an unoccupied strip
/// — measured on Cassowary's own command palette after the docked-strip and
/// planner fixes landed (`render/chrome/theme_picker.rs`,
/// `render/chrome/rotated_location.rs`) freed the row those fixes were
/// charging alongside it. Held at a full row rather than cut further: a full
/// row still reads as a deliberate beat (the `query_input_beat_reads_as_
/// more_than_a_full_row_flat_and_faceted` law's own floor, which this constant
/// does not have to clear itself — it is graded only where it applies).
/// REVERT COST: one line — delete this constant and the `unified_pane` arm in
/// `overlay_header_gap` reading it, leaving every world back on the plain
/// `OVERLAY_QUERY_BEAT`.
pub(in crate::render) const OVERLAY_QUERY_BEAT_UNIFIED_PANE: Rows = Rows(1.0);
pub(in crate::render) const OVERLAY_HINT_ROW: Rows = Rows(0.70);
pub(in crate::render) const OVERLAY_HINT_GAP_ROW: Rows = Rows(0.65);
pub(super) const OVERLAY_FOOTER_PAD: Logical = Logical(2.0);
pub(super) const CARD_PAD: Logical = Logical(12.0);
pub(super) const CARD_MARGIN: Logical = Logical(12.0);
pub(super) const CARD_TOP_DROP: Logical = Logical(40.0);
pub(super) const CONTEXT_ANCHOR_DROP: Logical = Logical(4.0);
pub(super) const PANE_TEXT_HPAD: Logical = Logical(12.0);
pub(super) const SPELL_PAD: Logical = Logical(10.0);
pub(super) const SPELL_MARGIN: Logical = Logical(8.0);
pub(super) const SPELL_WORD_GAP: Logical = Logical(6.0);
pub(super) const SPELL_MIN_W: LogicalGrowOnly = LogicalGrowOnly(140.0);
pub(super) const SPELL_MAX_W: LogicalGrowOnly = LogicalGrowOnly(520.0);

/// The overlay's three text inks bundled together: passed as bare adjacent
/// `glyphon::Color`/`Option<glyphon::Color>` params, `ink` and `muted` are the
/// same type in the same position — a swap compiles clean and recolors a row
/// wrong with no type error to catch it. `pub(in crate::render)` (not
/// `pub(super)`) because `render::tests` needs it too, to call the shapers
/// this bundles directly.
#[derive(Clone, Copy)]
pub(in crate::render) struct OverlaySpanInks {
    pub(in crate::render) ink: glyphon::Color,
    pub(in crate::render) muted: glyphon::Color,
    pub(in crate::render) selected: Option<glyphon::Color>,
}

/// The overlay card family's shared per-frame surface: the wgpu upload
/// target plus the geometry/plan pair every quad-prep step in that family
/// reads. `width`/`height` are same-typed adjacent `u32`s (a real transpose
/// risk on their own), and this bundles them with the `device`/`queue`/
/// `geom`/`plan` quartet that travels with them at every call in the family
/// rather than repeating all six as bare positional params.
#[derive(Clone, Copy)]
pub(super) struct OverlayCardSurface<'a> {
    pub(super) device: &'a wgpu::Device,
    pub(super) queue: &'a wgpu::Queue,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) geom: &'a OverlayGeom,
    pub(super) plan: &'a OverlayRowPlan,
}

const HINT_EXPLANATION: &str = "type to filter   ";

pub(in crate::render) fn overlay_rail_inset(ww: f32, scale: f32, dpi: f32) -> f32 {
    (ww / 3.0 - CARD_MAX_W.px(scale, dpi) * 0.5).max(0.0)
}

pub(in crate::render) fn hint_yielding_explanation(hint: &str, logical_window_w: f32) -> String {
    if logical_window_w < 800.0
        && let Some(actions) = hint.strip_prefix(HINT_EXPLANATION)
    {
        return actions.to_string();
    }
    hint.to_string()
}

pub(in crate::render) fn hint_matches_authored(authored: &str, shaped: &str) -> bool {
    shaped == authored
        || authored.strip_prefix(HINT_EXPLANATION) == Some(shaped)
        || (!shaped.is_empty()
            && authored.ends_with(shaped)
            && authored[..authored.len() - shaped.len()].ends_with(crate::overlay::HINT_SEP))
}

pub(in crate::render) fn overlay_card_box_policy(
    anchor: theme::CardAnchor,
    ww: f32,
    desired_w: f32,
    scale: f32,
    dpi: f32,
) -> (f32, f32) {
    let floor = CARD_EDGE_INSET_FLOOR.px(scale);
    let full = overlay_rail_inset(ww, scale, dpi);
    let cw = desired_w.min((ww - 2.0 * floor).max(0.0));
    let free = (ww - cw).max(0.0);
    let anchored_max = (ww - floor - cw).max(floor);
    let left = match anchor {
        theme::CardAnchor::TopCenter => free * 0.5,
        theme::CardAnchor::TopLeft => full.min(anchored_max).max(floor).min(free),
        theme::CardAnchor::Inset { x_frac } => {
            let span = (ww - cw - 2.0 * full).max(0.0);
            (full + x_frac.clamp(0.0, 1.0) * span)
                .min(anchored_max)
                .max(floor)
                .min(free)
        }
        theme::CardAnchor::TopRight => {
            let span = (ww - cw - 2.0 * full).max(0.0);
            (full + span).min(anchored_max).max(floor).min(free)
        }
    };
    (left, cw)
}

pub(in crate::render) fn overlay_card_fill_regime(ww: f32, desired_w: f32, scale: f32) -> bool {
    desired_w > (ww - 2.0 * CARD_EDGE_INSET_FLOOR.px(scale)).max(0.0)
}

/// Whether the active `FacetStyle` docks the lens strip's line ABOVE the card
/// (`docked_facet_band`) instead of leaving it drawn in the panel's own header
/// band. Read at both geometry owners that need it — `theme_overlay_geometry`
/// (how much row budget the strip's own box bills) and `overlay_row_plan`
/// (the SAME fact, fed into the row planner as a plain scalar so the pure
/// `render::plan` module never reads theme state itself) — from this one
/// place, so a future style added to the `DockedTab` family is picked up by
/// both without either call site re-deriving the match.
pub(in crate::render) fn facet_strip_is_docked() -> bool {
    matches!(
        crate::render::effective_facet_style(),
        theme::FacetStyle::DockedTab
    )
}

/// Whether a GROUPED card's lens strip sits inside a `Split` composition's
/// LOWER surface — the one gate every consumer of that fact reads (the mark's
/// plate floor, the relocated-strip seat, the shaping gate), so a future
/// composition axis is picked up everywhere at once rather than three
/// independently-drifting inline copies. `false` for a flat card (`geom.theme`),
/// a workspace (its own rail is its facet strip, no seam to speak of), a
/// `DockedTab` world (the strip already lives entirely off the card, above
/// it — a different relocation with its own seat), and any world whose list
/// style is not `Card`-backed (`Bars`/`Diagonal`/`Ruled` draw no plate to seam)
/// or whose pane composition is `Unified` (no seam at all).
pub(in crate::render) fn split_seam_active(geom: &OverlayGeom) -> bool {
    geom.theme
        && !geom.workspace
        && !facet_strip_is_docked()
        && crate::render::effective_list_style().list_backing(false) == theme::ListBacking::Card
        && matches!(
            crate::render::effective_pane_split(),
            theme::PaneSplit::Split
        )
}
