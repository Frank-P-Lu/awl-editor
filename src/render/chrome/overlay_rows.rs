//! Selected overlay rows and facets: bands, bars, motion, marks, and draw probes.
//!
//! Carved out of [`super::overlay`] verbatim, no behaviour change. `TextPipeline`
//! lives in [`crate::render`], of which this is a descendant module, so these
//! methods keep full access to its private GPU fields; Rust merges the inherent
//! `impl TextPipeline` blocks across the module tree, so splitting the file is a
//! pure physical carve — the chrome pixels are byte-identical. See [`super`].

use super::*;
pub(super) const FACET_CHIP_RADIUS: Logical = Logical(6.0);

/// The `ChipVariant::Underline` skin's own corner — a near-hairline rounding on
/// a rule, not a pill's radius, which is why it is not [`FACET_CHIP_RADIUS`].
pub(super) const CHIP_UNDERLINE_CORNER: Logical = Logical(1.75);

impl TextPipeline {
    /// TEST HOOK: total shaped glyphs the overlay text renderer would draw this
    /// frame (summed across the name buffer's layout runs). `0` once
    /// [`Self::park_overlay`] has emptied it — the assertion that a closed
    /// overlay carries no stale palette glyphs into the next frame.
    #[cfg(test)]
    pub(in crate::render) fn overlay_text_glyph_count(&self) -> usize {
        self.panel_buffer
            .layout_runs()
            .map(|r| r.glyphs.len())
            .sum()
    }

    /// TEST HOOK — the absolute canvas box occupied by the shaped
    /// glyph CELLS on one primary overlay line. This is deliberately read from
    /// `panel_buffer`, the buffer the draw pass uploads, rather than rebuilt
    /// from row arithmetic: ordering and drawn↔hit-test laws can point at a
    /// title, facet, candidate, or footer glyph that actually exists and ask
    /// the production hit-test owners what that same point means.
    #[cfg(test)]
    pub(in crate::render) fn overlay_line_glyph_box(&self, line_i: usize) -> Option<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let mut x0 = f32::INFINITY;
        let mut x1 = f32::NEG_INFINITY;
        let mut y0 = f32::INFINITY;
        let mut y1 = f32::NEG_INFINITY;
        for run in self
            .panel_buffer
            .layout_runs()
            .filter(|r| r.line_i == line_i)
        {
            if run.glyphs.is_empty() {
                continue;
            }
            let run_x0 = run.glyphs.iter().map(|g| g.x).fold(f32::INFINITY, f32::min);
            let run_x1 = run
                .glyphs
                .iter()
                .map(|g| g.x + g.w)
                .fold(f32::NEG_INFINITY, f32::max);
            x0 = x0.min(geom.text_left + run_x0);
            x1 = x1.max(geom.text_left + run_x1);
            y0 = y0.min(geom.text_top + run.line_top);
            y1 = y1.max(geom.text_top + run.line_top + run.line_height);
        }
        (x0.is_finite() && x1 > x0 && y0.is_finite() && y1 > y0).then_some([
            x0,
            y0,
            x1 - x0,
            y1 - y0,
        ])
    }

    pub(in crate::render) fn overlay_pane_fills(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Vec<[f32; 4]> {
        let full = [geom.card_x, geom.card_y, geom.card_w, geom.card_h];
        // A WORKSPACE IS ONE SURFACE. The split composition
        // carves a card's query beat into a separate upper plate; that is a
        // small-card gesture, and run across a workspace it would cut the
        // navigation rail in half at an arbitrary height. A room does not have a
        // seam through its wall.
        if geom.workspace {
            return vec![full];
        }
        if !matches!(
            crate::render::effective_pane_split(),
            theme::PaneSplit::Split
        ) {
            return vec![full];
        }
        let Some((gap_top, gap_bottom)) = plan.split_bounds() else {
            return vec![full];
        };
        let card_bottom = geom.card_y + geom.card_h;
        if gap_top > geom.card_y && gap_bottom < card_bottom && gap_bottom > gap_top {
            vec![
                [geom.card_x, geom.card_y, geom.card_w, gap_top - geom.card_y],
                [
                    geom.card_x,
                    gap_bottom,
                    geom.card_w,
                    card_bottom - gap_bottom,
                ],
            ]
        } else {
            vec![full]
        }
    }

    #[cfg(test)]
    pub(in crate::render) fn overlay_pane_fills_probe(&self) -> Vec<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        self.overlay_pane_fills(&geom, &plan)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn overlay_draw_card(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
    ) {
        let list_style = crate::render::effective_list_style();
        let spell = self.overlay_spell.is_some();
        let card_rect = [geom.card_x, geom.card_y, geom.card_w, geom.card_h];
        let backing = list_style.list_backing(spell);
        self.overlay_prepare_card_backing(
            device, queue, width, height, geom, plan, backing, spell, card_rect,
        );
        self.overlay_prepare_selection(
            device, queue, width, height, geom, plan, list_style, backing, vis,
        );
        self.prepare_diagonal_spine(device, queue, width, height, plan, vis);
        self.overlay_prepare_range_rails(device, queue, width, height, geom, plan, vis);
        self.overlay_prepare_facet_marks(device, queue, width, height, geom);
    }
    #[allow(clippy::too_many_arguments)]
    fn overlay_prepare_range_rails(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
    ) {
        // THE RANGE ROW'S RAIL. Every visible range row's track / fill /
        // thumb, resolved by the ONE rail owner (`overlay_rails`, which the pointer
        // hit-test reads too — so the control is clickable exactly where it is
        // drawn). EMPTY for every other card (both pipelines park → byte-identical).
        //
        // INK: two quiet rungs, never the amber accent (DESIGN §3) — `faint` for the
        // track, `muted` for the fill + thumb. When the SELECTED row carries a rail
        // AND the highlight band would wash `muted` out, the fill/thumb flip through
        // the ONE `theme::selected_row_secondary_ink` owner — the SAME mechanism the
        // value TEXT beside it already uses, so rail and number stay legible together
        // on every world rather than either growing its own contrast rule.
        //
        // WHICH row's rail flips is the shared visual-selection
        // transaction's answer, not the logical row's — the thumb is a secondary
        // ink like the value beside it, and both now wait for the band.
        //
        // THE FLIP IS PER RAIL, NEVER PER FRAME. A card can show several
        // range rows at once (Settings seats both `Zoom` and `Scroll
        // sensitivity` in the default window), and only ONE of them is ever
        // the row the visual-selection band sits on, so `on_band` is asked
        // PER ITEM and uploaded through `prepare_multicolor` (the
        // writing-streaks heatmap's own per-instance-color door,
        // `render/layers.rs`) rather than one shared `set_color` for the
        // whole pipeline — a single shared colour would paint whichever ink
        // the (unique) selected rail earned onto every OTHER rail's
        // fill/thumb too, a non-selected row reading the selected row's
        // flipped ink whenever a Pane world washes `muted` out
        // (`rail_thumb_over_fill`; Bars/Diagonal/Rules never flip at all, so
        // that mistake would be invisible there). Each thumb/fill pair
        // carries the ink its OWN row earned.
        let rails = self.overlay_rails(geom, plan);
        let on_band: Vec<usize> = vis.rows().iter().filter_map(|&k| plan.item_at(k)).collect();
        let selected_flip = super::overlay_selected_rail_srgb();
        let mut track_rects: Vec<[f32; 4]> = Vec::new();
        let mut thumb_quads: Vec<([f32; 4], [u8; 4])> = Vec::new();
        for (item, rail) in &rails {
            track_rects.push(rail.track);
            let ink = match selected_flip {
                Some(flip) if on_band.contains(item) => flip,
                _ => theme::muted(),
            }
            .rgba_bytes();
            if rail.fill[2] > 0.0 {
                thumb_quads.push((rail.fill, ink));
            }
            thumb_quads.push((rail.thumb, ink));
        }
        self.overlay_range_track
            .set_color(theme::faint().rgba_bytes());
        self.overlay_range_track
            .prepare(device, queue, width, height, &track_rects);
        self.overlay_range_thumb
            .prepare_multicolor(device, queue, width, height, &thumb_quads);
    }

    #[allow(clippy::too_many_arguments)]
    fn overlay_prepare_card_backing(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        backing: theme::ListBacking,
        spell: bool,
        card_rect: [f32; 4],
    ) {
        match backing {
            theme::ListBacking::BarePlates => {
                self.panel_shadow.prepare(device, queue, width, height, &[]);
                self.panel_border.prepare(device, queue, width, height, &[]);
            }
            theme::ListBacking::Card if spell => {
                let (chamfer_px, texture) = self.card_shape_texture(&[card_rect]);
                self.claim_float_panel(card_rect, FloatElevation::Rimmed, chamfer_px, texture);
                self.panel_card.prepare(device, queue, width, height, &[]);
                self.panel_shadow.prepare(device, queue, width, height, &[]);
                self.panel_border.prepare(device, queue, width, height, &[]);
            }
            theme::ListBacking::Card => {
                let fills = self.overlay_pane_fills(geom, plan);
                self.prepare_panel_card_elevation(device, queue, width, height, &fills);
            }
        }
    }
}
