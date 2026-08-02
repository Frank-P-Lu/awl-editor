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
    pub strip_line_top: Option<f32>,
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
        let sec_top = plan.secondary_top();
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
        let mut strip_line_top = None;
        let mut strip_line_bottom = None;
        for run in self.panel_buffer.layout_runs() {
            if run.line_i == 1 {
                strip_baseline = Some(geom.text_top + run.line_y);
                strip_line_top = Some(geom.text_top + run.line_top);
                strip_line_bottom = Some(geom.text_top + run.line_top + run.line_height);
                break;
            }
        }
        let strip_underline_y = self.overlay_theme_underline.map(|q| q[1]);
        // DRAWN, not planned: the query line's real shaped box, so a law can
        // compare it against the plan the caret and the hit-test read.
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
            caret_center: plan
                .query_band()
                .map_or(geom.text_top, |field| field.center()),
            query_line_top,
            query_line_height,
            query_baseline,
            primary,
            secondary,
            strip_baseline,
            strip_line_top,
            strip_line_bottom,
            strip_underline_y,
        }
    }

    /// TEST HOOK (jump-hint round): the widest shaped FOOTER line (the foot hint,
    /// plus any keybindings tips) vs the card's inner text width, for the
    /// currently-shaped flat overlay — so the discoverability law can assert the
    /// enriched jump hint NEVER CLIPS (`footer_px <= text_w`), an OUTCOME measured
    /// over the shaped GLYPHS (the Wagtail tripwire: appearance from pixels, not
    /// from the hint STRING). Routes the width through the ONE footer-measure owner
    /// `overlay_footer_content_px`, fed the PLANNED content-row count. Flat cards
    /// only (the narrowest card, so the tightest clip budget); call after a frame
    /// has shaped `panel_buffer`.
    pub(in crate::render) fn overlay_footer_fit_probe(&self, width: u32) -> (f32, f32) {
        let geom = self.overlay_geometry(width);
        let plan = self.overlay_row_plan(&geom);
        (
            self.overlay_footer_content_px(&geom, plan.content_rows()),
            geom.text_w,
        )
    }
}

/// ITEM 114 — the SUMMONED WORKSPACE's law probe: what the frame just committed
/// for its two regions, read off the same owners the draw and the hit-test read.
///
/// Reported rather than re-derived, for the reason this module exists: a law that
/// recomputed the rail's grid would be comparing one arithmetic against another
/// and would stay green through a change that moved only the pixels.
pub(in crate::render) struct WorkspaceProbe {
    /// The workspace surface itself, `[x, y, w, h]`.
    pub card: [f32; 4],
    /// The navigation rail's column box, or `None` when no rail is drawn.
    pub rail: Option<[f32; 4]>,
    /// Each rail entry's own rect, in strip order; `None` for an entry that
    /// falls outside the rail's box.
    pub rows: Vec<Option<[f32; 4]>>,
    /// The ACTIVE rail entry's mark rect — the quad the facet-mark owner draws.
    pub mark: Option<[f32; 4]>,
    /// The content pane's horizontal extent — the band every settings row, its
    /// selected-row quad and `overlay_row_at` are bounded to.
    pub pane_x: f32,
    pub pane_w: f32,
    /// The content pane's selected-row band, `[x, y, w, h]`, or `None` when no
    /// rows are drawn.
    pub selected_band: Option<[f32; 4]>,
    /// How many settings rows the pane drew.
    pub visible: usize,
}

impl TextPipeline {
    pub(in crate::render) fn workspace_rail_probe(&self, width: u32) -> WorkspaceProbe {
        let geom = self.overlay_geometry(width);
        let plan = self.overlay_row_plan(&geom);
        let rail = self.workspace_rail_box(&geom, &plan);
        let n = self.overlay_lens_len();
        let rows = (0..n)
            .map(|i| self.workspace_rail_rect(&geom, &plan, i))
            .collect();
        let selected_band = plan.selected_display().and_then(|d| {
            let top = plan.row_top(d)?;
            Some([geom.band_x_probe(), top, geom.band_w_probe(), plan.lh()])
        });
        WorkspaceProbe {
            card: geom.card_probe(),
            rail,
            rows,
            mark: self.workspace_rail_mark_probe(),
            pane_x: geom.band_x_probe(),
            pane_w: geom.band_w_probe(),
            selected_band,
            visible: geom.visible_probe(),
        }
    }
}
