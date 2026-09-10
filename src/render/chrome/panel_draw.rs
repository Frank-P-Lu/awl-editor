//! The search panel's UPLOAD + HIT-TEST half — split out of `panel.rs` to
//! keep it under its production ceiling. `panel_shape_text` (in `panel.rs`)
//! decides WHAT the card says; this file decides where the pixels and the
//! pointer meet: uploading the shaped text plus the bordered controls' fills/
//! borders/separators, the region separators' own geometry, the click-test,
//! and the amber caret's placement. See `panel.rs`'s own module doc.

use super::*;

impl TextPipeline {
    /// Upload the shaped panel text (red on the no-match state, else calm ink),
    /// the opaque BASE_300 card behind it, the bordered field/button/checkbox
    /// boxes, and the thin region separators.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn panel_upload_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        shape: &PanelShape,
        card_rect: [f32; 4],
        text_left: f32,
        text_top: f32,
    ) -> anyhow::Result<()> {
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let panel_area = TextArea {
            buffer: &self.panel_buffer,
            left: text_left,
            top: text_top,
            scale: 1.0,
            bounds,
            default_color: if shape.no_match { shape.red } else { shape.ink },
            custom_glyphs: &[],
        };
        self.panel_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [panel_area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon panel prepare failed: {e:?}"))?;

        // ELEVATE the card on the reusable floating-panel primitive (raised
        // border + base_300 card, no drop shadow — dark-depth Option C), so the
        // summoned find/replace panel reads as risen a step above the crisp
        // document (DESIGN §5) — clearer, more present furniture than the old
        // flat pill.
        self.claim_float_panel(
            card_rect,
            FloatElevation::Rimmed,
            CardChamfer::default(),
            None,
        );

        // THE INNER CONTROLS: every field/button/checkbox box, resolved
        // through the ONE owner (`panel_controls_layout`) the click-test and
        // the sidecar also read, then the thin region separators between
        // fields / nav / actions.
        let resolved = self.panel_controls_layout(&self.panel_control_spans, text_left, text_top);
        let boxes = resolved.boxes();
        self.panel_control_fill
            .prepare(device, queue, width, height, &boxes);
        let pad = self.metrics.px_physical(FLOAT_BORDER_RING_PX);
        let outlined: Vec<[f32; 4]> = boxes
            .iter()
            .map(|&[x, y, w, h]| [x - pad, y - pad, w + 2.0 * pad, h + 2.0 * pad])
            .collect();
        self.panel_control_border
            .prepare(device, queue, width, height, &outlined);
        let rules = self.panel_rule_rects(card_rect, text_top);
        self.panel_rules.set_corner(0.0);
        self.panel_rules
            .prepare(device, queue, width, height, &rules);
        Ok(())
    }

    /// The thin hairline separator(s) between the panel's regions: always one
    /// under the field(s), and — once replace is revealed — a second under
    /// the nav row, above the actions row. Inset slightly from the card's own
    /// edges so it never touches the rim `claim_float_panel` draws.
    fn panel_rule_rects(&self, card_rect: [f32; 4], text_top: f32) -> Vec<[f32; 4]> {
        let [card_x, _y, card_w, _h] = card_rect;
        let inset = self.metrics.px_physical(RULE_INSET_X);
        let x0 = card_x + inset;
        let w = (card_w - 2.0 * inset).max(0.0);
        let stroke = self.metrics.px_physical(RULE_STROKE);
        let replace_active = self.search_replace_active;
        let nav_row = if replace_active { 2.0 } else { 1.0 };
        let bands = self.panel_rows(text_top);
        let mut out = vec![[x0, bands.band(nav_row).0 - stroke * 0.5, w, stroke]];
        // The actions row's own start, read back from where its first control
        // actually landed — never re-derived from `nav_row + 1`, which is only
        // true at ordinary widths (the nav row wraps to a second line under
        // narrow pressure, moving the actions row down with it).
        if let Some(actions_row) = self.panel_control_spans.replace_button.map(|s| s.row) {
            out.push([x0, bands.band(actions_row).0 - stroke * 0.5, w, stroke]);
        }
        out
    }

    /// Hit-test a physical pointer `(px, py)` against the summoned find/replace
    /// panel. Reuses `panel_layout`'s card + row geometry and
    /// `panel_controls_layout`'s box geometry — the SAME layout the caret/text/
    /// boxes draw from, no parallel geometry — so a click can never disagree
    /// with where something is painted. Returns `None` off the card or with
    /// the panel down (the caller lets the press fall through to the document);
    /// [`PanelHit::Elsewhere`] for anywhere else inside the card (a calm no-op).
    pub fn panel_hit(&self, px: f32, py: f32) -> Option<PanelHit> {
        if !self.search_active {
            return None;
        }
        let width = self.window_w as u32;
        let (card, text_left, text_top, _caret_x) = self.panel_layout(width, 0, 0, 0.0);
        let [card_x, card_y, card_w, card_h] = card;
        if px < card_x || px > card_x + card_w || py < card_y || py > card_y + card_h {
            return None;
        }
        let resolved = self.panel_controls_layout(&self.panel_control_spans, text_left, text_top);
        let inside = |r: Option<[f32; 4]>| {
            r.is_some_and(|[x, y, w, h]| px >= x && px <= x + w && py >= y && py <= y + h)
        };
        if inside(resolved.case_box) {
            return Some(PanelHit::CaseToggle);
        }
        if inside(resolved.nav_prev) {
            return Some(PanelHit::NavPrev);
        }
        if inside(resolved.nav_next) {
            return Some(PanelHit::NavNext);
        }
        if inside(resolved.replace_button) {
            return Some(PanelHit::ReplaceButton);
        }
        if inside(resolved.replace_all_button) {
            return Some(PanelHit::ReplaceAllButton);
        }
        let row = self.panel_rows(text_top).row_at(py);
        Some(match row {
            0 => PanelHit::Find,
            1 if self.search_replace_active => PanelHit::Replace,
            _ => PanelHit::Elsewhere,
        })
    }

    /// Place the amber query caret: a resting block matching the document caret's
    /// height, centered vertically on the FOCUSED field's row (row 0 = search,
    /// row 1 = replace). The row's centre comes from the panel's ONE row-band
    /// owner (`panel_rows`), the same seam the hit-test inverts and the sidecar
    /// projection publishes, so the caret cannot ride a row the pointer disagrees
    /// about — asked through `panel_caret_cy`, which is that centre under a name a
    /// law can reach.
    pub(super) fn panel_place_caret(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        caret_x: f32,
        text_top: f32,
        caret_row: f32,
    ) {
        let m = self.metrics;
        let caret_h = m.caret_h * 0.8;
        let caret_cx = caret_x + m.caret_w * 0.5;
        let caret_cy = self.panel_caret_cy(text_top, caret_row);
        self.panel_caret.prepare(
            queue,
            width,
            height,
            CaretRect {
                center_x: caret_cx,
                center_y: caret_cy,
                rect_w: m.caret_w,
                rect_h: caret_h,
                corner: m.px(CORNER_RADIUS),
            },
        );
    }
}

/// The separators' inset from the card's own left/right edges, and their own
/// stroke weight — both device px, unscaled by DPI (a crisp hairline at any
/// zoom/density), mirroring `PANEL_PAD`/`FLOAT_BORDER_RING_PX`'s own contract.
/// Matches `PANEL_PAD`: the rule starts exactly at the text pad boundary, well
/// clear of the few px right of the card's own edge a rim-measuring oracle
/// samples to find the CARD's rim without crossing this hairline instead.
pub(in crate::render) const RULE_INSET_X: Physical = PANEL_PAD;
pub(in crate::render) const RULE_STROKE: Physical = Physical(1.0);
