//! Candidate placement and composition marks share the field's painted caret.

use super::*;
use crate::textbox::TextField;

impl TextPipeline {
    pub(in crate::render) fn search_field_mark_range(&self) -> Option<(usize, usize)> {
        self.search_field_selection
            .or_else(|| self.field_input.as_ref().and_then(|field| field.preedit))
    }

    pub(crate) fn focused_field_caret_rect(&self) -> Option<[f32; 4]> {
        self.field_caret_rect
    }

    pub(in crate::render) fn overlay_text_caret_box(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        if self
            .field_input
            .as_ref()
            .is_some_and(|field| field.field == TextField::SettingsValue)
        {
            return self.settings_text_caret_box(geom, plan);
        }
        self.overlay_query_caret_box(geom, plan)
    }

    fn settings_text_caret_box(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        self.settings_text_position(geom, plan, self.field_input.as_ref()?.caret)
    }

    fn settings_text_position(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        caret: usize,
    ) -> Option<[f32; 4]> {
        let field = self.field_input.as_ref()?;
        let item = field.row?;
        let row = plan.rows().iter().find(|row| row.item == Some(item))?;
        if !self.overlay_right_shown {
            return None;
        }
        let line = plan.billed_header_rows() + plan.cue_above_rows() + row.display;
        let run = self
            .panel_bind_buffer
            .layout_runs()
            .find(|run| run.line_i == line)?;
        let text = self.overlay_bindings.get(item)?;
        let byte = field_caret_byte(text, caret);
        let bind_w = self.panel_bind_buffer.size().0.unwrap_or(0.0);
        let end = run
            .glyphs
            .last()
            .map(|glyph| glyph.x + glyph.w)
            .unwrap_or(0.0);
        let x = self.overlay_accessory_span(geom, row.display, bind_w).0
            + run
                .glyphs
                .iter()
                .find(|glyph| glyph.start >= byte)
                .map(|glyph| glyph.x)
                .unwrap_or(end);
        let m = self.metrics.ui();
        let h = m.caret_h * 0.8 * OVERLAY_UI_SCALE;
        Some([x, row.top + (row.height - h) * 0.5, m.caret_w, h])
    }

    pub(in crate::render) fn overlay_composition_rect(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        let input = self.field_input.as_ref()?;
        let (start, end) = input.preedit?;
        if input.field == TextField::SettingsValue {
            let [x0, y, _, h] = self.settings_text_position(geom, plan, start)?;
            let [x1, ..] = self.settings_text_position(geom, plan, end)?;
            return Some([
                x0,
                y + h,
                (x1 - x0).max(1.0),
                self.metrics.ui().px(Logical(1.0)),
            ]);
        }
        let [_, y, _, h] = self.overlay_query_caret_box(geom, plan)?;
        let x0 = self.overlay_query_glyph_x(geom, plan, start);
        let x1 = self.overlay_query_glyph_x(geom, plan, end);
        Some([
            x0,
            y + h,
            (x1 - x0).max(1.0),
            self.metrics.ui().px(Logical(1.0)),
        ])
    }

    pub(in crate::render) fn overlay_text_selection_box(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        if let Some(input) = &self.field_input
            && input.field == TextField::SettingsValue
        {
            let (start, end) = input.selection?;
            let [x0, y, _, h] = self.settings_text_position(geom, plan, start)?;
            let [x1, ..] = self.settings_text_position(geom, plan, end)?;
            return Some([x0, y, (x1 - x0).max(1.0), h]);
        }
        self.overlay_query_selection_box(geom, plan)
    }
}
