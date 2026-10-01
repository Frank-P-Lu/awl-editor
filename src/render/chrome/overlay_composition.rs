//! Shared composition details for the two panels whose header is a real field
//! rather than a picker breadcrumb: Settings and Themes.

use super::*;

const FIELD_PAD_X: Logical = Logical(8.0);
const FIELD_PAD_Y: Logical = Logical(3.0);
const BUTTON_PAD_X: Logical = Logical(3.0);
const BUTTON_PAD_Y: Logical = Logical(2.0);
const HAIRLINE: Physical = Physical(1.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemePanelAction {
    Switch,
    Cancel,
}

impl TextPipeline {
    fn composed_query_field(&self, geom: &OverlayGeom, plan: &OverlayRowPlan) -> Option<[f32; 4]> {
        if !self.overlay_theme_picker && !(geom.workspace && !self.overlay_rows_primary) {
            return None;
        }
        let field = plan.query_band()?;
        let ui = self.metrics.ui();
        let pad_x = ui.px(FIELD_PAD_X);
        let pad_y = ui.px(FIELD_PAD_Y);
        let input_x = self.overlay_query_input_x(geom, plan);
        let right = geom.text_left + geom.text_w;
        (right > input_x).then_some([
            (input_x - pad_x).max(geom.text_left),
            field.top + pad_y,
            (right - input_x + pad_x).max(1.0),
            (field.height - 2.0 * pad_y).max(1.0),
        ])
    }

    fn line_span_rect(
        &self,
        geom: &OverlayGeom,
        line: usize,
        start: usize,
        end: usize,
    ) -> Option<[f32; 4]> {
        let run = self
            .panel_buffer
            .layout_runs()
            .find(|run| run.line_i == line)?;
        let mut x0 = f32::INFINITY;
        let mut x1 = f32::NEG_INFINITY;
        for glyph in run
            .glyphs
            .iter()
            .filter(|glyph| glyph.start >= start && glyph.start < end)
        {
            x0 = x0.min(glyph.x);
            x1 = x1.max(glyph.x + glyph.w);
        }
        if !x0.is_finite() || x1 <= x0 {
            return None;
        }
        let ui = self.metrics.ui();
        let pad_x = ui.px(BUTTON_PAD_X);
        let pad_y = ui.px(BUTTON_PAD_Y);
        Some([
            geom.text_left + x0 - pad_x,
            geom.text_top + run.line_top + pad_y,
            x1 - x0 + 2.0 * pad_x,
            (run.line_height - 2.0 * pad_y).max(1.0),
        ])
    }

    pub(in crate::render) fn theme_panel_action_rects(
        &self,
        geom: &OverlayGeom,
    ) -> Option<([f32; 4], [f32; 4])> {
        if !self.overlay_theme_picker {
            return None;
        }
        let line = self.overlay_hint_line()?;
        let text = self.panel_buffer.lines.get(line)?.text();
        let switch_word = text.find("switch")?;
        let cancel_word = text.find("cancel")?;
        let cell_start = |word: usize| {
            text[..word]
                .rfind(crate::overlay::HINT_SEP)
                .map_or(0, |i| i + crate::overlay::HINT_SEP.len())
        };
        let switch = self.line_span_rect(
            geom,
            line,
            cell_start(switch_word),
            switch_word + "switch".len(),
        )?;
        let cancel = self.line_span_rect(
            geom,
            line,
            cell_start(cancel_word),
            cancel_word + "cancel".len(),
        )?;
        Some((switch, cancel))
    }

    pub fn theme_panel_action_at(&self, px: f32, py: f32) -> Option<ThemePanelAction> {
        let (switch, cancel) = self.theme_panel_action_report()?;
        let contains = |[x, y, w, h]: [f32; 4]| px >= x && px <= x + w && py >= y && py <= y + h;
        if contains(switch) {
            Some(ThemePanelAction::Switch)
        } else if contains(cancel) {
            Some(ThemePanelAction::Cancel)
        } else {
            None
        }
    }

    pub(crate) fn theme_panel_action_report(&self) -> Option<([f32; 4], [f32; 4])> {
        if !self.overlay_active || !self.overlay_theme_picker {
            return None;
        }
        self.theme_panel_action_rects(&self.overlay_geometry(self.window_w as u32))
    }

    pub(in crate::render) fn overlay_composition_quads(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
        let mut fills = Vec::new();
        let mut borders = Vec::new();
        if let Some(field) = self.composed_query_field(geom, plan) {
            fills.push(field);
            borders.extend(rect_edges(field, self.metrics.px_physical(HAIRLINE)));
        }
        if let Some((switch, cancel)) = self.theme_panel_action_rects(geom) {
            fills.extend([switch, cancel]);
            borders.extend(rect_edges(switch, self.metrics.px_physical(HAIRLINE)));
            borders.extend(rect_edges(cancel, self.metrics.px_physical(HAIRLINE)));
        }
        if geom.workspace
            && !self.overlay_rows_primary
            && let Some([rail_x, rail_w]) = geom.rail
        {
            let x = rail_x + rail_w + (geom.pane_x - rail_x - rail_w) * 0.5;
            borders.push([
                x,
                geom.card_y + self.metrics.ui().px(super::workspace::WORKSPACE_PAD),
                self.metrics.px_physical(HAIRLINE),
                (geom.card_h - 2.0 * self.metrics.ui().px(super::workspace::WORKSPACE_PAD))
                    .max(0.0),
            ]);
        }
        (fills, borders)
    }

    pub(in crate::render) fn append_overlay_composition_quads(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        borders: &mut Vec<[f32; 4]>,
        fills: &mut Vec<([f32; 4], [u8; 4])>,
    ) {
        let (composition_fills, composition_borders) = self.overlay_composition_quads(geom, plan);
        borders.extend(composition_borders);
        let fill = self.overlay_composition_inks().0;
        fills.extend(composition_fills.into_iter().map(|rect| (rect, fill)));
    }

    pub(in crate::render) fn overlay_composition_inks(&self) -> ([u8; 4], [u8; 4]) {
        let chrome = crate::render::overlay_chrome_theme();
        let emphasized = self.overlay_theme_picker || self.overlay_query_focused;
        let fill = if self.overlay_theme_picker {
            chrome.base_300
        } else {
            chrome.base_200
        };
        let border = if emphasized {
            chrome.muted
        } else {
            chrome.faint
        };
        (fill.rgba_bytes(), border.rgba_bytes())
    }

    pub(super) fn overlay_composed_title_prefix(&self, geom: &OverlayGeom) -> Option<String> {
        (self.overlay_theme_picker || (geom.workspace && !self.overlay_rows_primary)).then(|| {
            let mut chars = self.overlay_title.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
                + "   "
        })
    }
}

fn rect_edges([x, y, w, h]: [f32; 4], stroke: f32) -> [[f32; 4]; 4] {
    [
        [x, y, w, stroke],
        [x, y + h - stroke, w, stroke],
        [x, y, stroke, h],
        [x + w - stroke, y, stroke, h],
    ]
}
