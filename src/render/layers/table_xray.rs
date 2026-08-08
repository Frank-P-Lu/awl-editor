//! Per-frame table source reveal.

use super::*;

impl TextPipeline {
    /// THE X-RAY (the user's canonized metaphor): for every GFM table ROW the
    /// caret sits on OR the active SELECTION touches, stash that row's RAW SOURCE
    /// shaped as ONE NON-WRAPPING line ([`crate::render::XrayRow`]) so (a) the
    /// caret's own `col_x_and_advance` redirects onto ITS row (the concealed doc
    /// row is zero-width — see the redirect in `geometry.rs`), (b)
    /// `caret_band_scale` sizes the caret to the source band on any x-rayed row,
    /// and (c) `prepare_table_grid` floats each one, centered, over the row band
    /// its OWN grid cells were skipped for (the true source SWAP — see that
    /// function's own doc comment). Only the CARET's own row pans horizontally to
    /// keep the caret visible (the find-field single-line pan model, carrying the
    /// previous frame's pan for that SAME row so a walk along it doesn't jitter);
    /// a row revealed purely by selection has no caret column to pan toward and
    /// always floats flush-left (`pan = 0`). Run BEFORE [`Self::prepare_caret_layer`]
    /// so the redirect is ready when the caret geometry is computed. Clears the
    /// stash first (a caret/selection that no longer touches a table row heals
    /// it). Gated on WYSIWYG + markdown; empty for every other frame
    /// (byte-identical capture).
    ///
    /// PER-FRAME COST STAYS BOUNDED (never O(doc lines)): the candidate set is
    /// found by walking TABLE BLOCKS (mirrors `prepare_table_grid`'s own Phase A
    /// block walk) and intersecting each block's OWN line span with the caret
    /// line / selection range, so a Select-All across a huge document costs
    /// O(this doc's table rows), never O(doc lines) — a document with no table
    /// under the cursor/selection never even enters the per-line loop. A
    /// selection-only (non-caret) row additionally passes the SAME
    /// `line_ornament_visible` cull `prepare_table_grid` uses to skip off-screen
    /// tables, so a Select-All spanning an off-screen table doesn't shape
    /// thousands of `GlyphBuffer`s a frame; the caret's own row is never culled
    /// (it is on-screen by construction).
    pub(crate) fn prepare_table_xray(&mut self) {
        let prev_pan = self
            .xray
            .iter()
            .find(|x| x.line == self.cursor_line)
            .map(|x| x.pan)
            .unwrap_or(0.0);
        self.xray = Vec::new();
        if !(crate::markdown::wysiwyg_on() && self.md_enabled) {
            return;
        }
        let blocks = self.table_blocks();
        if blocks.is_empty() {
            return;
        }
        let cursor_line = self.cursor_line;
        // The active selection's LINE span (`l0..=l1`, column-agnostic — the SAME
        // extent `selection_touch_bytes` derives, never re-derived a second way),
        // or `None` with no selection.
        let sel_lines = self.selection.map(|((l0, _), (l1, _))| l0..=l1);
        let last_doc_line = self.buffer.lines.len().saturating_sub(1);
        let m = self.metrics;
        let body = GlyphMetrics::new(m.font_size, m.line_height);
        let base = self.doc_attrs().color(theme::base_content().to_glyphon());
        let view_w = self.text_wrap_width().max(1.0);
        let pad = m.px(crate::render::TABLE_CELL_PAD_X);
        let mut rows: Vec<crate::render::XrayRow> = Vec::new();
        // Walk TABLES, never document lines: a Select-All across a huge document
        // must cost O(this table's own rows), not O(doc lines) — the candidate
        // set below is bounded by the block's OWN line span, intersected with the
        // (already column-agnostic) selection range.
        for (header_line, range) in &blocks {
            if *header_line > last_doc_line {
                continue;
            }
            let mut li = *header_line;
            let mut b = self.line_doc_byte_start(*header_line);
            let mut last_line_of_block = *header_line;
            while li <= last_doc_line && b < range.end {
                last_line_of_block = li;
                b += self.buffer.lines[li].text().len() + 1;
                li += 1;
            }
            let caret_here = (*header_line..=last_line_of_block).contains(&cursor_line);
            for line in *header_line..=last_line_of_block {
                let is_caret = caret_here && line == cursor_line;
                let is_selected =
                    !is_caret && sel_lines.as_ref().is_some_and(|sl| sl.contains(&line));
                if !is_caret && !is_selected {
                    continue;
                }
                // VISIBLE-BAND CULL for selection-only rows (never the caret's
                // own — it is on-screen by construction, auto-scroll keeps it
                // there): a Select-All spanning an off-screen table must not
                // shape thousands of GlyphBuffers every frame.
                if is_selected && !self.line_ornament_visible(line) {
                    continue;
                }
                let Some(src) = self.buffer.lines.get(line).map(|l| l.text().to_string()) else {
                    continue;
                };
                let mut buf = GlyphBuffer::new(&mut self.font_system, body);
                buf.set_wrap(&mut self.font_system, Wrap::None);
                buf.set_size(&mut self.font_system, None, Some(m.line_height * 2.0));
                buf.set_text(&mut self.font_system, &src, &base, Shaping::Advanced, None);
                buf.shape_until_scroll(&mut self.font_system, false);
                let mut clusters: Vec<(usize, usize, f32, f32)> = Vec::new();
                for run in buf.layout_runs() {
                    for g in run.glyphs.iter() {
                        clusters.push((g.start, g.end, g.x, g.x + g.w));
                    }
                }
                let glyph_xs = super::geometry::assemble_glyph_xs(&src, &clusters, m.char_width);
                let content_w = glyph_xs.last().copied().unwrap_or(0.0);
                let (pan, height) = if is_caret {
                    let cc = self.cursor_col.min(glyph_xs.len().saturating_sub(1));
                    let caret_x = glyph_xs.get(cc).copied().unwrap_or(0.0);
                    let pan = super::geometry::xray_pan_for_caret(
                        caret_x, content_w, view_w, pad, prev_pan,
                    );
                    (pan, self.cursor_row_height())
                } else {
                    let h = super::geometry::pick_row(&self.visual_rows(line), 0).line_height;
                    (0.0, h)
                };
                let top = self.line_ornament_top(line);
                rows.push(crate::render::XrayRow {
                    line,
                    source: src,
                    glyph_xs,
                    top,
                    height,
                    pan,
                });
            }
        }
        self.xray = rows;
    }
}
