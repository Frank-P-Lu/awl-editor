//! The faceted card's own half of the location cue: which plan line it is,
//! and — per the active world's `LocationStyle` — where that cue is composed.
//! `Raked` composes against the card's own row band and diagonal spine;
//! `RotatedRail` composes against the ROOM: the wordmark placard's margin, its
//! type size, its ink. The mask compose, the shrink-to-fit solver, and the
//! shared preparation call are `TextPipeline::prepare_rotated_location_label`
//! (`render/rotated_location.rs`), which knows nothing about facets, plans,
//! placards or diagonals — only a string and a
//! `RotatedLabelPlacement`. `OverlayGeom`/`OverlayRowPlan` are
//! `chrome`-private, so this is the one place that reads them for the cue.

use super::*;
use crate::render::rotated_location::{
    FlushEdge, Overflow, ROTATED_RAIL_PLACARD_GAP_EM, RotatedLabelPlacement, format_location_text,
    placard_font_size, raked_along_budget,
};

fn location_ink(style: theme::LocationLabelStyle) -> ([f32; 3], [f32; 3]) {
    let active = theme::active();
    let (a, b) = match style.ink {
        theme::LocationInk::Flat(role) => (role.resolve(&active), role.resolve(&active)),
        theme::LocationInk::Gradient(a, b) => (a.resolve(&active), b.resolve(&active)),
    };
    (
        srgb_u8_to_linear3(a.rgba_bytes()),
        srgb_u8_to_linear3(b.rgba_bytes()),
    )
}

impl TextPipeline {
    /// Read THIS frame's location line (the shared row planner's
    /// `PlanLine::Location`, still the row-plan's own single slot) and, on any
    /// world whose style paints it itself (`draws_inline() == false`), hand
    /// its text and a composed placement to the rotated-label capability.
    pub(super) fn prepare_overlay_rotated_location(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) {
        let style = theme::active().render_caps.location_style;
        let cue = (!style.draws_inline())
            .then(|| {
                geom.plan
                    .iter()
                    .enumerate()
                    .find_map(|(display, line)| match line {
                        PlanLine::Location(l) => Some((display, l.clone())),
                        _ => None,
                    })
            })
            .flatten()
            .and_then(|(display, label)| plan.rows().get(display).map(|row| (label, *row)));
        let Some((label, row)) = cue else {
            self.rotated_label_pipeline.clear();
            return;
        };

        let active_index =
            crate::render::rotated_location::active_location_index(&self.overlay_lens);
        let cluster = self.diagonal_cluster;
        let (label_style, placement) = match style {
            theme::LocationStyle::Inline => return, // excluded by `draws_inline()` above
            theme::LocationStyle::RotatedRail(label_style) => {
                (label_style, self.rotated_rail_placement(geom, label_style))
            }
            theme::LocationStyle::Raked(label_style) => (
                label_style,
                cluster.map(|cluster| {
                    // THE MEASURED step, not `DiagonalComposition::row_step` — see
                    // `location_axis_deg`'s own doc for why reading the narrow-card
                    // yield here is what keeps the cue and the spine beside it
                    // from disagreeing on a card too tight for the authored step.
                    let axis_deg =
                        super::diagonal::location_axis_deg(cluster.spine_step(), row.height);
                    let m = self.metrics;
                    let ui = crate::render::effective_overlay_scale();
                    let (color_a, color_b) = location_ink(label_style);
                    RotatedLabelPlacement {
                        flush: FlushEdge::Left(geom.text_left + row.dx),
                        bottom: row.bottom(),
                        // Unbounded ACROSS the reading axis: the rake's own
                        // thickness sits in the card's text column, which the row
                        // beside it is already sized to hold.
                        fit: [
                            f32::INFINITY,
                            raked_along_budget(row.height, geom.header_gap),
                        ],
                        natural_size: m.font_size * ui * label_style.scale,
                        face: label_style.face,
                        tracking_em: label_style.tracking_em,
                        axis_deg,
                        color_a,
                        color_b,
                        overflow: Overflow::Shrink,
                    }
                }),
            ),
        };

        let Some(label) = format_location_text(label_style, &label, active_index) else {
            self.rotated_label_pipeline.clear();
            return;
        };

        match placement {
            Some(placement) => self
                .prepare_rotated_location_label(device, queue, width, height, &label, &placement),
            // `Raked` with no measured diagonal cluster cannot happen on a
            // shipping world (only a `ListStyle::Diagonal` world is ever
            // assigned `Raked`, and `resolve_diagonal_cluster` always runs
            // before this), but a probe/force path could disagree with the
            // theme's own data — park rather than paint from stale data.
            // `RotatedRail` with no wordmark on this frame parks for a
            // designed reason, in `rotated_rail_placement`'s own doc.
            None => self.rotated_label_pipeline.clear(),
        }
    }

    /// **`RotatedRail`'s COMPOSITION: the wordmark's vertical companion.** The
    /// cue takes the placard's own outer MARGIN, sits just ABOVE it, and
    /// carries the theme-authored fraction of its type size in theme-authored
    /// ink. The placard geometry is read from its own owner
    /// ([`Self::overlay_shape_placard`]) on this frame, never re-derived here.
    ///
    /// `None` — the cue parks — in three cases, and each is the composition
    /// being honest rather than a quiet downgrade to a smaller treatment:
    ///
    /// - **No wordmark on this frame.** A card in the fill regime
    ///   (`card_narrow`: it already spans the room, leaving no margin to be a
    ///   companion in) or a kind that announces no title draws no placard, so
    ///   there is nothing to be the companion OF.
    /// - **A wordmark that hugs the room's TOP.** This composition is defined
    ///   against a bottom-anchored wordmark, which is what every corner the
    ///   `Auto` rule can derive gives (`render::derived_placard_corner`) and
    ///   what the roster's only `RotatedRail` world resolves to. A hand-pinned
    ///   top corner would need the mirrored vertical anchor; the law
    ///   `every_rotated_rail_world_anchors_its_wordmark_to_the_rooms_floor`
    ///   fails by name if a future world asks for one.
    /// - **A margin the authored run does not fit.** Between the card's own drawn
    ///   left edge and the room's, past roughly 1.7× zoom on the widest card,
    ///   the margin the cue lives in closes; the placard bleeds behind the card
    ///   there, and a cue seated on it would too. Parking rather than shrinking is
    ///   [`Overflow::Park`]'s own doc: the scale IS the composition, so the cue is
    ///   either exactly that or absent.
    ///
    /// THE FIT BOX IS THE REAL RISK and it is measured, never assumed: ALONG
    /// the reading axis the run may rise from the placard to the room's own top
    /// margin, and ACROSS it may not reach the card. Both bounds come from real
    /// rects on this frame (the placard's, the card's), so the longest name a
    /// faceted picker can carry is bounded by the same arithmetic as the
    /// shortest.
    fn rotated_rail_placement(
        &mut self,
        geom: &OverlayGeom,
        label_style: theme::LocationLabelStyle,
    ) -> Option<RotatedLabelPlacement> {
        let theme::TitleStyle::Placard { .. } = crate::render::effective_title_style() else {
            return None;
        };
        // THE PLACARD'S OWN OWNER, re-asked rather than remembered: it is a
        // pure function of this frame's geometry and window, and asking it
        // again is how this file avoids a second copy of the wordmark's sizing
        // ladder. It re-shapes `placard_buffer` with identical inputs AFTER
        // that buffer's own upload has already been taken, which the
        // location-differential pixel laws prove costs the placard no pixel.
        let (px, py, pw, ph) = self.overlay_shape_placard(geom)?;
        let room_top = self.menubar_reserve();
        let below = (self.window_h - (py + ph)).max(0.0);
        let above = (py - room_top).max(0.0);
        if below > above {
            return None; // a wordmark hugging the room's ceiling — see the doc
        }
        // THE ROOM'S OWN FRAME MARGIN, read off the placard's rect rather than
        // re-stated: whatever inset the wordmark keeps from the canvas is the
        // inset the cue keeps from the canvas's other three sides.
        let frame_inset = below;
        let natural_size = placard_font_size(ph) * label_style.scale;
        let bottom = py - natural_size * ROTATED_RAIL_PLACARD_GAP_EM;
        let along = bottom - room_top - frame_inset;

        // WHICH MARGIN: the one the wordmark itself hugs. The placard's corner
        // is derived to be the card's OPPOSITE (`derived_placard_corner`), so
        // following it is also what keeps the cue off the card.
        let hugs_left = px <= self.window_w - (px + pw);
        let (card_left, card_right) = self.overlay_card_drawn_span(geom);
        let (flush, across) = match hugs_left {
            true => (FlushEdge::Left(px), card_left - px - frame_inset),
            false => (
                FlushEdge::Right(px + pw),
                (px + pw) - card_right - frame_inset,
            ),
        };
        if along <= 0.0 || across <= 0.0 {
            return None;
        }
        let (color_a, color_b) = location_ink(label_style);
        Some(RotatedLabelPlacement {
            flush,
            bottom,
            fit: [across, along],
            natural_size,
            face: label_style.face,
            tracking_em: label_style.tracking_em,
            axis_deg: 90.0,
            color_a,
            color_b,
            overflow: Overflow::Park,
        })
    }

    /// **THE CARD'S TRUE DRAWN SPAN `(left, right)`, which is WIDER THAN ITS
    /// BOX.** `geom.card_x` is where the card's own surface starts; under
    /// `ListStyle::Bars` the SELECTED row's plate grows OUTWARD past it
    /// (`grow_span`, mirrored on a right-anchored card) and the plate scrim
    /// then pads that by [`BAR_SCRIM_PAD`] again — at 1.8× zoom on a
    /// right-anchored card that is 32 device px of card LEFT of `card_x`, and
    /// a cue bounded by `card_x` alone cleared the box while sitting 2 px from
    /// the plate. Asked through the plate's OWN span owners rather than
    /// re-derived, and at the growth animation's SETTLED maximum (progress
    /// `1.0`, not this frame's eased value), because a bound that holds only
    /// mid-transition is not a bound.
    fn overlay_card_drawn_span(&self, geom: &OverlayGeom) -> (f32, f32) {
        let (mut left, mut right) = (geom.card_x, geom.card_x + geom.card_w);
        if crate::render::effective_list_style().draws_row_plates() {
            let scale = self.metrics.scale;
            let (bx, bw) = super::bar_full_span(geom.card_x, geom.card_w, scale);
            let grow = self
                .metrics
                .px(Logical(crate::render::effective_bar_config().grow_px));
            let mirror = crate::render::resolve_overlay_anchor(self.overlay_align).mirrors_growth();
            let (gx, gw) = super::grow_span(bx, bw, grow, mirror);
            let pad = self.metrics.px(super::BAR_SCRIM_PAD);
            left = left.min(gx - pad);
            right = right.max(gx + gw + pad);
        }
        (left, right)
    }

    /// TEST-ONLY readers for the cue laws, which have to compare the cue's
    /// own decided placement and the card's own drawn span against real pixels
    /// without re-deriving either.
    #[cfg(test)]
    pub(in crate::render) fn rotated_rail_probe(
        &mut self,
        geom: &OverlayGeom,
    ) -> Option<(f32, [f32; 2], f32, f32)> {
        let theme::LocationStyle::RotatedRail(style) = theme::active().render_caps.location_style
        else {
            return None;
        };
        self.rotated_rail_placement(geom, style).map(|p| {
            let flush_x = match p.flush {
                FlushEdge::Left(x) | FlushEdge::Right(x) => x,
            };
            (p.natural_size, p.fit, p.bottom, flush_x)
        })
    }

    #[cfg(test)]
    pub(in crate::render) fn overlay_card_drawn_span_probe(
        &self,
        geom: &OverlayGeom,
    ) -> (f32, f32) {
        self.overlay_card_drawn_span(geom)
    }
}
