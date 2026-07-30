//! ITEM 174 — the overlay LAW PROBE, in the test tier where it belongs.
//!
//! `overlay_row_y_probe` reports what a rendered overlay frame actually
//! committed: the shaped primary/secondary line tops read back out of the
//! buffers the draw pass uploads, the strip's own baselines, and the PLANNED
//! selected line + band top. It lived next to the draw code and so tripped item
//! 164's source law, which forbids a RENDER PATH from reading the logical
//! selected row outside the visual-selection transaction — a law probe reporting
//! that row for a law to compare against is not a render path, and moving it here
//! keeps the law's sweep strict instead of growing an exception to it.
//!
//! It reads `TextPipeline`'s private glyph buffers directly, which this module can
//! do as a descendant of `crate::render` — and deliberately reports GLYPH
//! positions, not arithmetic, so a law can point at ink that genuinely exists.

use super::super::TextPipeline;
use super::super::chrome::overlay_query_center;
use super::super::chrome::overlay_secondary_top;

pub(in crate::render) struct OverlayYProbe {
    pub lh: f32,
    pub band_top: f32,
    pub sel_disp: usize,
    pub caret_center: f32,
    pub query_line_top: f32,
    pub query_line_height: f32,
    pub query_baseline: f32,
    pub primary: std::collections::BTreeMap<usize, f32>,
    pub secondary: std::collections::BTreeMap<usize, f32>,
    pub strip_baseline: Option<f32>,
    pub strip_line_bottom: Option<f32>,
    pub strip_underline_y: Option<f32>,
}

impl TextPipeline {
    pub(in crate::render) fn overlay_row_y_probe(&self) -> OverlayYProbe {
        use std::collections::BTreeMap;
        let geom = self.overlay_geometry(self.window_w as u32);
        // ITEM 174 — the probe reports the PLAN. It used to re-derive the row
        // pitch, the candidate-line count AND the selected display line from
        // `geom` with its own (differently clamped) arithmetic, so a law asserting
        // "the band sits on row k" was checking a second calculation rather than
        // the one the pixels came from.
        let plan = self.overlay_row_plan(&geom);
        let lh = plan.lh();
        let header_rows = geom.header_rows;
        let last = header_rows + plan.candidate_rows();
        let mut primary = BTreeMap::new();
        for run in self.panel_buffer.layout_runs() {
            let li = run.line_i;
            if li >= header_rows && li < last {
                primary.insert(li - header_rows, geom.text_top + run.line_top);
            }
        }
        let sec_top = overlay_secondary_top(geom.text_top, geom.header_gap);
        let mut secondary = BTreeMap::new();
        for run in self.panel_bind_buffer.layout_runs() {
            let li = run.line_i;
            if li >= header_rows && li < last {
                secondary.insert(li - header_rows, sec_top + run.line_top);
            }
        }
        let sel_disp = plan.selected_display().unwrap_or(0);
        let band_top = plan.row_top(sel_disp).unwrap_or(plan.first_top());
        let mut strip_baseline = None;
        let mut strip_line_bottom = None;
        for run in self.panel_buffer.layout_runs() {
            if run.line_i == 1 {
                strip_baseline = Some(geom.text_top + run.line_y);
                strip_line_bottom = Some(geom.text_top + run.line_top + run.line_height);
                break;
            }
        }
        let strip_underline_y = self.overlay_theme_underline.map(|q| q[1]);
        let query_run = self.panel_buffer.layout_runs().next();
        let query_line_height = query_run
            .as_ref()
            .map(|r| r.line_height)
            .unwrap_or_else(|| self.overlay_lh());
        let query_line_top = query_run
            .as_ref()
            .map(|r| geom.text_top + r.line_top)
            .unwrap_or(geom.text_top);
        let query_baseline = query_run
            .as_ref()
            .map(|r| geom.text_top + r.line_y)
            .unwrap_or(geom.text_top);
        OverlayYProbe {
            lh,
            band_top,
            sel_disp,
            caret_center: overlay_query_center(geom.text_top, query_line_height),
            query_line_top,
            query_line_height,
            query_baseline,
            primary,
            secondary,
            strip_baseline,
            strip_line_bottom,
            strip_underline_y,
        }
    }
}
