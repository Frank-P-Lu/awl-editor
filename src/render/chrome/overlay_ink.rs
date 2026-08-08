//! WHAT THE SUMMONED CARD ACTUALLY DRAWS, AND HOW WIDE IT REACHES.
//!
//! The card's LAYOUT box (`overlay_card_rect`) is a placement policy: a desired width
//! clamped to the window, with no relation to how wide the shaped rows turned out. On a
//! composition that draws a panel behind everything that is invisible — the box IS the
//! surface. On one that draws no panel and no plate it is a claim about territory that
//! nothing occupies, and a treatment scoped to it treats air.
//!
//! [`TextPipeline::overlay_drawn_surfaces`] is the other question: the boxes the frame's
//! own surfaces occupy. Its consumer is the footprint frost, which is a treatment of the
//! page BEHIND the card and therefore owes exactly the drawn surfaces a backdrop and
//! nothing beside them.
//!
//! # THE ENUMERATION IS NOT WHAT THE GUARANTEE RESTS ON
//!
//! A list of surfaces goes stale the moment a composition grows a new one, and a
//! too-short list here would leave real chrome over sharp document. Two laws bound it
//! from both ends, and neither is a reading of this file:
//!
//! * `frost_parallelogram_item318`'s coverage floor requires the card's upright chrome to
//!   be frosted, over the box its own production owner declares.
//! * `frost_width_item343`'s coverage law renders the picker with the frost SUPPRESSED
//!   over an empty document and differences it against the same frame with the picker
//!   closed — the two are identical but for the card's own drawing, so the residue IS that
//!   drawing — then requires every pixel of it to have the frost's shipping mask at or
//!   above a floor beneath it. A surface nobody remembered is card ink with no frost under
//!   it and fails there by existing, which is also how every EXCLUSION from the list below
//!   is earned: "this composition draws nothing else" is a measurement rather than a name.
//!   Measured on the enrolled roster, dropping the rules term alone leaves 9700 of
//!   Paperbark's 23319 drawn pixels over sharp document, and dropping the selected row's
//!   mark leaves 98 of Mangrove's.
//!
//! Every seat here is READ from the owner the draw path reads, never re-derived: the
//! panel buffer's per-band seats are [`TextPipeline::overlay_panel_bands`], which the
//! EMITTER itself now loops over, so the band a frost measures and the band glyphon was
//! handed are one object.

use super::*;

/// ONE CLIP BAND OF THE PANEL BUFFER — where the shared `panel_buffer` is seated for the
/// slice of the card between `clip_top` and `clip_bottom`.
///
/// A leaning composition draws the ONE buffer through several areas, each with its own
/// `left`: the head band at the card's text edge, one band per row seated off the
/// cluster, and the whole foot as a third. `clip_bottom` may be infinite — the emitter
/// clamps it to the surface, which is the only place a canvas height belongs.
pub(in crate::render) struct PanelBand {
    pub left: f32,
    pub clip_top: f32,
    pub clip_bottom: f32,
}

/// A box's horizontal union with `(lo, hi)`, or the box alone when there is none yet.
fn widen(span: &mut Option<[f32; 4]>, add: [f32; 4]) {
    match span {
        Some(b) => {
            b[0] = b[0].min(add[0]);
            b[1] = b[1].min(add[1]);
            b[2] = b[2].max(add[2]);
            b[3] = b[3].max(add[3]);
        }
        None => *span = Some(add),
    }
}

/// `[left, top, right, bottom]` from a `[x, y, w, h]` rect, both orders normalised so a
/// negative extent cannot invert the box.
fn ltrb(rect: [f32; 4]) -> [f32; 4] {
    let [x, y, w, h] = rect;
    [x.min(x + w), y.min(y + h), x.max(x + w), y.max(y + h)]
}

impl TextPipeline {
    /// WHERE THE PANEL BUFFER IS SEATED THIS FRAME, band by band — `None` when the whole
    /// buffer rides ONE area at the card's text edge (every upright composition, where
    /// there is nothing to band).
    ///
    /// THE EMITTER LOOPS OVER THIS. It is not a description of what the emitter does; it
    /// is the thing the emitter does, so a consumer measuring the drawn ink cannot part
    /// company with the seat glyphon was handed. Both readings used to exist and the
    /// second was a copy.
    pub(in crate::render) fn overlay_panel_bands(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<Vec<PanelBand>> {
        let slant = crate::render::overlay_slant();
        let cluster = self.diagonal_cluster;
        if slant.is_none() && cluster.is_none() {
            return None;
        }
        let mut bands = vec![PanelBand {
            left: geom.text_left,
            clip_top: 0.0,
            clip_bottom: plan.first_top(),
        }];
        // A mirrored cluster hangs its name on the SPINE end, so an ascending world's
        // name is right-aligned and its origin is a function of the ink it measures —
        // read from the shaped buffer this frame drew, never re-measured.
        let primary = cluster.map(|_| self.overlay_row_primary_px(geom));
        for row in plan.rows() {
            let left = match (cluster, &primary) {
                (Some(cluster), Some(primary)) => cluster.label_origin(
                    row.display,
                    primary.get(&row.display).copied().unwrap_or(0.0),
                ),
                _ => geom.text_left + row.dx,
            };
            bands.push(PanelBand {
                left,
                clip_top: row.top,
                clip_bottom: row.bottom(),
            });
        }
        bands.push(PanelBand {
            left: self.overlay_foot_left(geom, plan),
            clip_top: plan.band_bottom(),
            clip_bottom: f32::INFINITY,
        });
        Some(bands)
    }

    /// THE CARD'S TEXT COLUMN as the emitter's own `TextBounds` clips to it — ink outside
    /// it is not drawn, so it is not the frost's business either.
    fn overlay_text_clip(&self, geom: &OverlayGeom) -> (f32, f32) {
        let left = geom.text_left.max(0.0);
        (left, geom.text_left + geom.text_w)
    }

    /// ONE BUFFER'S DRAWN INK inside one clip band, in canvas coordinates — the glyph
    /// CELLS, clamped to the column the emitter clips to, or `None` when the band shapes
    /// nothing.
    ///
    /// The cells, not the line box: a line's advance width includes trailing space the
    /// frame draws nothing for, and the whole point of this file is to stop treating
    /// unoccupied width as occupied.
    fn buffer_band_ink(
        &self,
        buffer: &glyphon::Buffer,
        band: &PanelBand,
        top: f32,
        clip: (f32, f32),
    ) -> Option<[f32; 4]> {
        let mut out: Option<[f32; 4]> = None;
        for run in buffer.layout_runs() {
            let (y0, y1) = (top + run.line_top, top + run.line_top + run.line_height);
            if y1 <= band.clip_top || y0 >= band.clip_bottom || run.glyphs.is_empty() {
                continue;
            }
            let x0 = run.glyphs.iter().map(|g| g.x).fold(f32::INFINITY, f32::min);
            let x1 = run
                .glyphs
                .iter()
                .map(|g| g.x + g.w)
                .fold(f32::NEG_INFINITY, f32::max);
            if !(x0.is_finite() && x1 > x0) {
                continue;
            }
            let (l, r) = (
                (band.left + x0).clamp(clip.0, clip.1),
                (band.left + x1).clamp(clip.0, clip.1),
            );
            if r <= l {
                continue;
            }
            widen(
                &mut out,
                [l, y0.max(band.clip_top), r, y1.min(band.clip_bottom)],
            );
        }
        out
    }

    /// EVERY SURFACE THE SUMMONED CARD DRAWS, as `[left, top, right, bottom]` canvas
    /// boxes — the union's terms rather than the union, so a caller can shear each one
    /// about its own row before combining them.
    ///
    /// Read this module's header first: the list's completeness is a law's claim, not
    /// this comment's.
    pub(in crate::render) fn overlay_drawn_surfaces(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Vec<[f32; 4]> {
        let mut out: Vec<[f32; 4]> = Vec::new();
        let clip = self.overlay_text_clip(geom);
        // (1) THE SHAPED TEXT, band by band, off the seats the emitter used.
        let bands = self.overlay_panel_bands(geom, plan).unwrap_or_else(|| {
            vec![PanelBand {
                left: geom.text_left,
                clip_top: 0.0,
                clip_bottom: f32::INFINITY,
            }]
        });
        for band in &bands {
            out.extend(self.buffer_band_ink(&self.panel_buffer, band, geom.text_top, clip));
        }
        // (2) THE ACCESSORY COLUMN — its own buffer, hung on the cluster's outer end (or
        // the text column's far edge on an upright card), leading with the header's own
        // empty lines. `overlay_right_shown` is the emitter's own gate.
        if self.overlay_right_shown {
            let flow = super::diagonal::accessory_flow(self);
            let secondary = self.overlay_row_secondary_px(geom);
            for row in plan.rows() {
                let w = secondary.get(&row.display).copied().unwrap_or(0.0);
                if w <= 0.0 {
                    continue;
                }
                let anchor = match self.diagonal_cluster {
                    Some(cluster) => cluster.accessory_anchor(row.display),
                    None => geom.text_left + geom.text_w,
                };
                let (l, r) = flow.span(anchor, w);
                out.push([
                    l.clamp(clip.0, clip.1),
                    row.top,
                    r.clamp(clip.0, clip.1),
                    row.bottom(),
                ]);
            }
        }
        // (3) THE QUERY CARET — a quad, past the head band's last glyph by its own width.
        out.extend(self.overlay_query_caret_box(geom, plan).map(ltrb));
        // (4) THE RULED COMPOSITION'S RULES, at the reach the one rule owner can emit
        // over any row set and either mark. Rules run the BAND, which is the card's full
        // width, so this is the term that keeps a ruled world's frost the box it was.
        if let Some(spans) = self.overlay_rule_spans(geom) {
            let (l, r) = spans.x_reach();
            let (top, bottom) = match (plan.rows().first(), plan.rows().last()) {
                (Some(f), Some(t)) => (f.top - spans.heavy, t.bottom() + spans.heavy),
                _ => (plan.first_top(), plan.band_bottom()),
            };
            out.push([l, top, r, bottom]);
        }
        // (5) THE DIAGONAL SPINE and the selected row's mark, off the same rail the draw
        // read them from.
        if let (Some(cluster), Some(composition)) =
            (self.diagonal_cluster, super::diagonal::active(self))
        {
            // ⚠️ THE SPINE'S TWO ENDS, NOT ITS BOUNDING BOX. The spine is a diagonal
            // segment, and a consumer that shears its bbox about the card's centre reads
            // the two corners the segment never touches — widening the answer by
            // `|shear| × spine height`, which on this roster is 78 physical px and larger
            // than the whole narrowing. The shape is convex and the un-shear affine, so
            // containing both END caps contains the segment exactly.
            let half = composition.spine_weight * 0.5;
            for row in [plan.rows().first(), plan.rows().last()]
                .into_iter()
                .flatten()
            {
                let x = cluster.spine_x(row.display);
                let cy = row.top + row.height * 0.5;
                out.push([x - half, cy - half, x + half, cy + half]);
            }
            // THE MARK'S LANE, ASKED OF EVERY ROW — never of "which row is selected".
            //
            // Only one row draws a mark, and asking the plan which one would read the LOGICAL
            // selected row rather than the visual-selection transaction's answer, which is a
            // second answer on the card during every selection move. `mark_span` needs no
            // such question: the rail carries its own selected row and applies the outward
            // shift only there, so the union over all rows already contains the shifted mark
            // the frame drew, and every unshifted sibling collapses inside it. The frost's
            // box comes out INDEPENDENT of the selection, which also keeps a selection move
            // from invalidating the cached backdrop.
            for row in plan.rows() {
                let (vertex, arm) = cluster.mark_span(row.display);
                let (top, bottom) = composition.mark_span_y(row.top, row.height);
                let half = composition.mark_weight * 0.5;
                out.push([
                    vertex.min(arm) - half,
                    top - half,
                    vertex.max(arm) + half,
                    bottom + half,
                ]);
            }
        }
        // (6) THE RANGE ROWS' RAILS, off the one rail owner the pointer hit-test reads.
        for (_, rail) in self.overlay_rails(geom, plan) {
            out.push(ltrb(rail.track));
            out.push(ltrb(rail.fill));
            out.push(ltrb(rail.thumb));
        }
        // (7) THE FACETED STRIP'S ACTIVE-LENS MARK, as the shaper recorded it.
        out.extend(self.overlay_theme_underline.map(ltrb));
        out.retain(|b| b[0].is_finite() && b[1].is_finite() && b[2] > b[0] && b[3] > b[1]);
        out
    }
}
