//! The query field's glyph-run lookup, caret box, and selection box —
//! split out of `overlay_draw` to keep that file under its file-size mark.
//! Same `TextPipeline` impl target, own file, no ownership change.

use super::*;

impl TextPipeline {
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
        let first_run = self.panel_buffer.layout_runs().next();
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
        let field = plan.query_band()?;
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
        let field = plan.query_band()?;
        let (start, end) = self.overlay_query_selection?;
        let m = self.metrics;
        let start_x = self.overlay_query_glyph_x(geom, plan, start);
        let end_x = self.overlay_query_glyph_x(geom, plan, end);
        let sel_h = m.caret_h * 0.8 * OVERLAY_UI_SCALE;
        let sel_cy = field.center();
        Some([start_x, sel_cy - sel_h * 0.5, end_x - start_x, sel_h])
    }
}
