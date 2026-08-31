//! Theme-authored toast placement at the measured-text boundary.
//!
//! The notice owner still shapes and paints the shared plated line. This module
//! owns only the toast-specific fold from that shaped line plus active summoned
//! chrome into the shared floating planner.

use super::*;

/// Toast placement is authored in logical space. Unlike the older generic
/// corner-label inset, this is a new DPI-swept axis and enters the model with
/// display scale classified from its first frame.
pub(in crate::render) const TOAST_SAFE_INSET: Logical = Logical(8.0);

/// Air kept between a transient acknowledgement and summoned chrome.
pub(in crate::render) const TOAST_COLLISION_GAP: Logical = Logical(6.0);

/// The CALM NOTICE plate's three inks for `kind`: `(fill, rim, text)`.
///
/// The fill is the shared depth ramp's own plane: `base_200` for a self-clearing
/// toast, `base_300` for a held sticky. The rim is a hairline off the ink ladder.
/// Both axes express lifetime by value only. Text comes from
/// `theme::selected_row_ink`, the one owner of legible ink on the band.
///
/// A fill alone is not reliable: authored ramps may collapse, an equal byte step
/// is not an equal perceptual step over the sRGB curve, and a true one-bit world
/// has no intermediate value. Its second level is inverse video, reached through
/// `HighlightTreatment` rather than a world-name test.
pub(super) fn notice_plate_inks(
    kind: crate::actions::NoticeKind,
) -> (theme::Srgb, theme::Srgb, theme::Srgb) {
    let one_bit = matches!(
        theme::active().highlight_treatment(theme::selection_document()),
        theme::HighlightTreatment::InverseFill { .. }
    );
    let (fill, rim) = match (kind, one_bit) {
        (crate::actions::NoticeKind::Toast, _) => (theme::base_200(), theme::muted()),
        (crate::actions::NoticeKind::Sticky, false) => (theme::base_300(), theme::base_content()),
        (crate::actions::NoticeKind::Sticky, true) => (theme::base_content(), theme::base_100()),
    };
    (fill, rim, theme::selected_row_ink(fill))
}

impl TextPipeline {
    /// Measure the final, already-elided toast candidate and resolve its authored
    /// placement. Elision may probe several candidates, so its last callback is
    /// not itself a geometry contract.
    pub(super) fn notice_toast_plan(
        &mut self,
        text: &str,
        gm: GlyphMetrics,
        width: u32,
        height: u32,
        padding: [f32; 2],
    ) -> Option<crate::render::plan::ToastPlan> {
        if text.is_empty() {
            return None;
        }
        self.notice_buffer.set_metrics(&mut self.font_system, gm);
        self.notice_buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(gm.line_height),
        );
        self.notice_buffer.set_text(
            &mut self.font_system,
            text,
            &panel_attrs(),
            Shaping::Advanced,
            None,
        );
        self.notice_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let text_w = self
            .notice_buffer
            .layout_runs()
            .fold(0.0_f32, |w, run| w.max(run.line_w));
        let obstacles = self.notice_active_chrome(width, height);
        Some(crate::render::plan::plan_toast(
            theme::active().toast_anchor,
            [width as f32, height as f32],
            [text_w, gm.line_height],
            padding,
            self.metrics.px(TOAST_SAFE_INSET),
            self.metrics.px(TOAST_COLLISION_GAP),
            self.menubar_reserve(),
            &obstacles,
        ))
    }

    /// Active chrome that a toast may not cover. The persistent document
    /// outline contributes its interactive margin band. Contextual pickers
    /// contribute their whole card. A workspace contributes its occupied
    /// header-and-row band; its lower unoccupied plane remains a legitimate
    /// narrow fallback. Every obstacle comes from the geometry owner paint or
    /// hit testing already reads.
    fn notice_active_chrome(&self, width: u32, height: u32) -> Vec<[f32; 4]> {
        if !self.overlay_active {
            return self.outline_keepout_rect(height).into_iter().collect();
        }
        let geom = self.overlay_geometry(width);
        if !geom.workspace {
            return vec![[geom.card_x, geom.card_y, geom.card_w, geom.card_h]];
        }
        let plan = self.overlay_row_plan(&geom);
        let occupied_bottom = plan.footer_top().min(geom.card_y + geom.card_h);
        vec![[
            geom.card_x,
            geom.card_y,
            geom.card_w,
            (occupied_bottom - geom.card_y).max(0.0),
        ]]
    }

    #[cfg(test)]
    pub(crate) fn notice_geometry_probe(
        &mut self,
        width: u32,
        height: u32,
    ) -> Option<([f32; 4], theme::ToastAnchor)> {
        let text = self.notice_drawn.clone();
        let label = crate::markdown::type_scale::LABEL;
        let gm = GlyphMetrics::new(
            self.metrics.font_size * label,
            self.metrics.line_height * label,
        );
        let (pad_x, pad_y) = notice_plate_padding(gm.line_height);
        self.notice_toast_plan(&text, gm, width, height, [pad_x, pad_y])
            .map(|plan| (plan.plate, plan.resolved))
    }

    #[cfg(test)]
    pub(crate) fn notice_active_chrome_probe(&self, width: u32, height: u32) -> Vec<[f32; 4]> {
        self.notice_active_chrome(width, height)
    }
}
