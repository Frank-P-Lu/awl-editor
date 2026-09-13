//! The query field's shared pointer bounds, glyph-run lookup, caret box, and
//! selection box. Every interaction reads the same planned band and shaped run.

use super::*;
use crate::render::plan::PlannedHeader;

impl TextPipeline {
    pub(super) fn overlay_query_band(&self, plan: &OverlayRowPlan) -> Option<PlannedHeader> {
        let line = usize::from(self.overlay_files_surface) * 2;
        plan.header_lines().get(line).copied()
    }

    fn overlay_query_run(&self) -> Option<glyphon::LayoutRun<'_>> {
        self.panel_buffer
            .layout_runs()
            .nth(usize::from(self.overlay_files_surface) * 2)
    }

    /// Left edge of the editable query span. Files reserves the shaped title
    /// prefix for header controls; ordinary cards retain their full-row field.
    pub(in crate::render) fn overlay_query_input_x(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> f32 {
        if !self.overlay_files_surface
            && !self.overlay_theme_picker
            && !(geom.workspace && !self.overlay_rows_primary)
        {
            return geom.card_x;
        }
        let prefix = self.overlay_title_prefix(geom);
        let origin = self.overlay_head_left(geom, plan);
        self.overlay_query_run()
            .and_then(|run| {
                run.glyphs
                    .iter()
                    .find(|glyph| glyph.start >= prefix.len())
                    .map(|glyph| origin + glyph.x)
                    .or(Some(origin + run.line_w))
            })
            .unwrap_or(origin)
    }

    /// Hit-test a pointer at PHYSICAL `(px, py)` against the SUMMONED overlay's
    /// editable QUERY-INPUT line — the `› query` filter field every flat/nav/theme
    /// picker draws on top. Returns `true` when the pointer sits inside the
    /// field's own PLANNED line box, within the card's x-bounds. The contextual
    /// SPELL panel has NO query line (`header_rows == 0`), so the plan carries no
    /// query band and this always returns `false`. Used by
    /// `input.rs::sync_cursor_icon` to give the field the I-beam.
    ///
    /// **THE DRIFT THIS CLOSES.** Reading the bare row pitch here — `text_top ..
    /// text_top + lh` — describes the wrong box on the FLAT family, whose field
    /// is `lh + header_gap` tall (the beat is the BOTTOM of the field's own box,
    /// not a row after it) with its ink half-led LOW inside it. On the shipping
    /// default at 1200x800 the field draws `[64.0, 133.2]`, caret at 98.6 and
    /// baseline at 106.0, against a pointer band ending at 91.2: the I-beam sat
    /// in empty air above the text, missing by 7.4px at 1x and 14.8px at 2x. The
    /// GROUPED family was right by accident (its beat inflates the lens strip
    /// instead), which is how a parallel calculation survives review — it agrees
    /// on the arm somebody looked at.
    pub fn over_overlay_query(&self, px: f32, py: f32) -> bool {
        if !self.overlay_active {
            return false;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let Some(field) = self.overlay_query_band(&plan) else {
            return false;
        };
        let x0 = self.overlay_query_input_x(&geom, &plan);
        px >= x0 && px <= geom.card_x + geom.card_w && field.contains(py)
    }

    /// The CHAR index into the raw query text nearest pointer `(px, py)` — the
    /// click-to-place counterpart to [`Self::over_overlay_query`]'s I-beam gate:
    /// same box (same `query_x`/card-right/`field.contains`), so a click can
    /// only place a caret where the I-beam already promised one. `None` off the
    /// field.
    ///
    /// Walks the SAME shaped run [`Self::overlay_query_caret_box`] reads a
    /// caret's x from, skipping the prefix (title / `› ` sigil) by byte offset
    /// so the first placeable column sits right after it, never inside it. A
    /// press at or before the first glyph's center resolves to 0; past the
    /// last glyph's center resolves to the query's own length — "round to the
    /// nearer glyph edge", the plain text-field rule.
    pub fn overlay_query_char_at(&self, px: f32, py: f32) -> Option<usize> {
        if !self.overlay_active {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let field = self.overlay_query_band(&plan)?;
        let query_x = self.overlay_query_input_x(&geom, &plan);
        if !(px >= query_x && px <= geom.card_x + geom.card_w && field.contains(py)) {
            return None;
        }
        let title_prefix = self.overlay_title_prefix(&geom);
        let prefix_len = if title_prefix.is_empty() {
            "› ".len()
        } else {
            title_prefix.len()
        };
        let query_len = self.overlay_query.chars().count();
        let Some(run) = self.overlay_query_run() else {
            return Some(query_len);
        };
        for g in run.glyphs.iter() {
            if g.start < prefix_len {
                continue;
            }
            let Some(char_idx) = self
                .overlay_query
                .get(..g.start - prefix_len)
                .map(|s| s.chars().count())
            else {
                continue;
            };
            if px < self.overlay_head_left(&geom, &plan) + g.x + g.w * 0.5 {
                return Some(char_idx);
            }
        }
        Some(query_len)
    }

    /// THE QUERY FIELD's glyph-run X for CHAR index `char_idx` — the ONE
    /// shaped-text lookup both [`Self::overlay_query_caret_box`] and
    /// [`Self::overlay_query_selection_box`] read, so the caret and a
    /// selection edge can never disagree about where a character sits.
    fn overlay_query_glyph_x(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        char_idx: usize,
    ) -> f32 {
        let m = self.metrics;
        let sigil = "› ";
        let title_prefix = self.overlay_title_prefix(geom);
        let prefix_len = if title_prefix.is_empty() {
            sigil.len()
        } else {
            title_prefix.len()
        };
        let char_idx = char_idx.min(self.overlay_query.chars().count());
        let target_byte = prefix_len + field_caret_byte(&self.overlay_query, char_idx);
        let first_run = self.overlay_query_run();
        // `overlay_head_left`'s own seat — the text edge, or right-aligned.
        self.overlay_head_left(geom, plan)
            + first_run
                .as_ref()
                .and_then(|r| {
                    r.glyphs
                        .iter()
                        .find(|g| g.start == target_byte)
                        .map(|g| g.x)
                })
                .or_else(|| first_run.as_ref().map(|r| r.line_w))
                .unwrap_or_else(|| {
                    m.char_width
                        * (sigil.chars().count() + self.overlay_query.chars().count()) as f32
                })
    }

    /// THE QUERY FIELD'S CARET, as the box it is drawn in (`[x, y, w, h]`) — `None` when
    /// the card draws no query line at all (the contextual spell popup).
    ///
    /// A `&self` owner rather than arithmetic inside the placer, because the footprint
    /// frost has to know how far the head band's ink really reaches and the caret stands
    /// its own width past the last glyph. One derivation, so the quad the frost accounts
    /// for is the quad the frame drew.
    pub(in crate::render) fn overlay_query_caret_box(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        // The field's own PLANNED line box. `None` is the contextual spell
        // popup, which draws no query line at all.
        let field = self.overlay_query_band(plan)?;
        // …and `None` again on a card whose head line is NOT a field: Credits'
        // one fixed row names the document beside it, so there is nothing to
        // search and the query door refuses every character
        // (`OverlayKind::offers_query`). A caret parked after that title
        // advertises an edit that cannot happen — the same promise the document
        // caret makes over a relocated transcript, one region up. The line still
        // draws; only its caret parks.
        if !self.overlay_query_field || !self.overlay_query_focused {
            return None;
        }
        let m = self.metrics;
        let caret_char = self
            .overlay_query_caret
            .min(self.overlay_query.chars().count());
        let caret_x = self.overlay_query_glyph_x(geom, plan, caret_char);
        let caret_h = m.caret_h * 0.8 * OVERLAY_UI_SCALE;
        // The caret is centred in the SAME planned field box the pointer
        // hit-test accepts and the split composition carves its gap from —
        // never a line height read back off the shaped run here, which is a
        // second calculation only the draw path can see.
        let caret_cy = field.center();
        Some([caret_x, caret_cy - caret_h * 0.5, m.caret_w, caret_h])
    }

    /// THE QUERY FIELD'S SELECTION, as the box it is drawn in (`[x, y, w, h]`)
    /// — `None` when the card draws no query line, OR when `overlay_query_selection`
    /// is `None` (every card but an in-progress Rename, and Rename itself once
    /// the seeded selection collapses). Same field box / same glyph lookup as
    /// [`Self::overlay_query_caret_box`], so the selection band and the caret
    /// are always seated on the SAME line, never a second calculation that can
    /// drift from it.
    pub(in crate::render) fn overlay_query_selection_box(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        let field = self.overlay_query_band(plan)?;
        let (start, end) = self.overlay_query_selection?;
        let m = self.metrics;
        let start_x = self.overlay_query_glyph_x(geom, plan, start);
        let end_x = self.overlay_query_glyph_x(geom, plan, end);
        let sel_h = m.caret_h * 0.8 * OVERLAY_UI_SCALE;
        let sel_cy = field.center();
        Some([start_x, sel_cy - sel_h * 0.5, end_x - start_x, sel_h])
    }
}
