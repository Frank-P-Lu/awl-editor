//! Shared selection and search highlight geometry.

use super::*;

type TextRange = ((usize, usize), (usize, usize));

pub(in crate::render) fn intersecting_rows(
    rows: &[VisualRow],
    start: usize,
    end: usize,
) -> std::ops::Range<usize> {
    let first = rows.partition_point(|row| row.end_col < start);
    let past = rows.partition_point(|row| row.start_col <= end);
    first..past.max(first)
}

impl TextPipeline {
    /// Compute the selection highlight rectangles in pixels for the current
    /// selection, scroll, and zoom. Multi-line: first line from anchor-col to
    /// end-of-line, full-width middle lines, last line up to cursor-col. Each
    /// rect is `[x, y, w, h]`. Reads the SAME metrics + scroll as glyph layout,
    /// so the highlight sits exactly behind the selected glyphs.
    pub(in crate::render) fn selection_rects(&self) -> Vec<[f32; 4]> {
        let Some(((l0, c0), (l1, c1))) = self.selection else {
            return Vec::new();
        };
        self.range_rects((l0, c0), (l1, c1))
    }

    pub(in crate::render) fn range_rects(
        &self,
        (l0, c0): (usize, usize),
        (l1, c1): (usize, usize),
    ) -> Vec<[f32; 4]> {
        let range = ((l0, c0), (l1, c1));
        let lines = self.visible_lines_for_ranges(&[range]);
        let rows_by_line = self.visual_rows_for_lines(&lines);
        self.range_rects_from_rows(range, &rows_by_line)
    }

    /// The logical lines from `ranges` whose shaped rows could paint in this
    /// frame. Search uses this once for its whole match roster; selection passes
    /// one range. Keeping the visible-band decision here makes their clipping
    /// semantics identical while avoiding one whole-row gather per match.
    fn visible_lines_for_ranges(&self, ranges: &[TextRange]) -> std::collections::BTreeSet<usize> {
        let m = &self.metrics;
        let doc_top = self.doc_top();
        // VISIBLE-BAND CULL (mirrors the wash / squiggle / nit proto builders). A
        // selection can span the WHOLE document (Select-All), yet only the on-screen
        // rows can paint. Restrict the lines we resolve to those whose vertical
        // extent intersects the viewport (plus the generous ornament margin), read
        // O(1) per line from the first-row-top table — so the BATCHED geometry
        // resolve below is O(visible), not O(doc). Band edges are buffer-relative so
        // each line's raw `line_first_top` compares without re-adding `doc_top`.
        let margin = m.line_height * 8.0;
        let band_lo = -margin - doc_top;
        let band_hi = self.window_h + margin - doc_top;
        let last_line = self.buffer.lines.len().saturating_sub(1);
        let first_top = |line: usize| {
            self.row_geom
                .line_first_top(&self.buffer, &self.metrics, line)
        };
        let mut lines = std::collections::BTreeSet::new();
        for &((l0, _), (l1, _)) in ranges {
            for line in l0..=l1.min(last_line) {
                let top = first_top(line);
                let bottom = if line < last_line {
                    first_top(line + 1)
                } else {
                    self.total_doc_height()
                };
                if bottom > band_lo && top < band_hi {
                    lines.insert(line);
                }
            }
        }
        lines
    }

    /// Emit one range's visible highlight geometry using a caller-provided,
    /// shared row gather. `search_match_rects` supplies all its visible lines at
    /// once; selection supplies its one range, so both retain the same xray,
    /// conceal, wrap, eol-pad and content-clip behaviour.
    fn range_rects_from_rows(
        &self,
        ((l0, c0), (l1, c1)): TextRange,
        rows_by_line: &std::collections::HashMap<usize, Vec<VisualRow>>,
    ) -> Vec<[f32; 4]> {
        let m = &self.metrics;
        let doc_top = self.doc_top();
        let eol_pad = m.char_width * 0.5;
        // ONE `layout_runs()` walk for ALL visible selected lines — replaces the
        // per-line `line_glyph_xs` + `visual_rows` (each an O(doc) run walk that also
        // CLOBBERED the single-slot cursor-line memo), so Select-All is no longer
        // O(doc^2) per frame while the caret spring animates. `visual_rows_for_lines`
        // never touches that memo, and per line yields rows byte-identical to
        // `visual_rows(line)`.
        let text_left = self.text_left();
        let mut rects = Vec::new();
        for line in l0..=l1 {
            // A GFM table row the selection touches has its source CONCEALED to
            // zero-width (`ensure_wash_protos`'s table carve-out documents the same
            // collapse for the wash buckets) while `prepare_table_xray` floats that
            // row's raw source, at its REAL shaped advances, over the still-drawn
            // grid cells (`XrayRow`). Reading `rows_by_line` here would measure the
            // concealed geometry and paint the reported hairline sliver at the left
            // margin instead of a band under the revealed ink — so a row present in
            // `self.xray` is rebuilt from `glyph_xs` (the same source the drawn
            // float uses) instead of falling into the generic row-based path below.
            // `row_band_for`'s height/top still come from the row's own (possibly
            // tall, wrapped-grid-cell) reserved box exactly as the generic path
            // uses, so only the horizontal extent changes.
            if let Some(xray) = self.xray.iter().find(|x| x.line == line) {
                if !self.proto_visible(xray.top, xray.height) {
                    continue; // off-screen row: the quad would rasterize nothing
                }
                let line_char_count = xray.glyph_xs.len().saturating_sub(1);
                let sel_start = if line == l0 { c0 } else { 0 }.min(line_char_count);
                let (sel_end, extends_to_eol) = if line == l1 {
                    (c1.min(line_char_count), false)
                } else {
                    (line_char_count, true)
                };
                if sel_end < sel_start {
                    continue;
                }
                let a = sel_start.min(line_char_count);
                let b = sel_end.min(line_char_count);
                // The x-ray row never wraps (`Wrap::None` in `prepare_table_xray`),
                // so it is always its own "last row" — the trailing-selection eol
                // pad applies whenever the span reaches the source line's end.
                let pad = if extends_to_eol && b >= line_char_count {
                    eol_pad
                } else {
                    0.0
                };
                let (x, w) = xray_x_span(xray, text_left, a, b, 0.0);
                let w = w + pad;
                if w <= 0.0 {
                    continue;
                }
                let (y, row_caret_h) = self.row_band_for(line, xray.height, xray.top);
                rects.push([x, y, w, row_caret_h]);
                continue;
            }
            let Some(rows) = rows_by_line.get(&line) else {
                continue; // culled: off-screen line
            };
            // Every row carries the WHOLE logical line's `xs` (char_count+1 long), so
            // any row's length is the line's char count — identical to the retired
            // `line_glyph_xs(line).len() - 1`. The logical line's column span
            // [sel_start, sel_end] within the selection: lines before the last run
            // through the (virtual) end-of-line newline; the last line stops at c1.
            let line_char_count = rows
                .first()
                .map(|r| r.xs.len().saturating_sub(1))
                .unwrap_or(0);
            let sel_start = if line == l0 { c0 } else { 0 };
            let (sel_end, extends_to_eol) = if line == l1 {
                (c1.min(line_char_count), false)
            } else {
                (line_char_count, true)
            };
            let sel_start = sel_start.min(line_char_count);
            // Emit one rect per VISUAL row of this logical line, clipped to the
            // selection's column span on that row. Each row uses its OWN wrap-aware
            // top + x boundaries, so a selection that spans a wrap boundary follows
            // the text down to the next row. Rows outside the visible band are
            // culled (they would rasterize nothing) — byte-identical on-screen.
            // A dense search can put many short matches on one enormously wrapped
            // line. Rows are ordered by their non-overlapping source columns, so
            // visit only the contiguous rows whose column spans can intersect this
            // range, rather than scanning every wrapped row for every match.
            let row_range = intersecting_rows(rows, sel_start, sel_end);
            if row_range.is_empty() {
                continue;
            }
            for (offset, row) in rows[row_range.clone()].iter().enumerate() {
                let ri = row_range.start + offset;
                #[cfg(test)]
                self.search_rect_work.set(self.search_rect_work.get() + 1);
                let line_top = doc_top + row.line_top;
                if !self.proto_visible(line_top, row.line_height) {
                    continue; // off-screen row: the quad would rasterize nothing
                }
                let row_char_count = row.xs.len().saturating_sub(1);
                // Intersect the selection's column span with this row's columns.
                let rs = sel_start.max(row.start_col);
                let re = sel_end.min(row.end_col);
                if re < rs {
                    continue;
                }
                let is_last_row = ri + 1 == rows.len();
                // Only the row that actually reaches the logical end-of-line gets
                // the newline pad (the trailing-selection sliver editors show).
                let pad = if extends_to_eol && is_last_row && re >= row_char_count {
                    eol_pad
                } else {
                    0.0
                };
                let a = rs.min(row_char_count);
                let b = re.min(row_char_count);
                let (x, w_raw) = row_x_span(row, text_left, a, b, 0.0);
                let w = w_raw + pad;
                if w <= 0.0 {
                    continue;
                }
                // Scale the highlight to the row so a heading's selection is as tall
                // as its glyphs (a base-height band on a big heading reads as broken),
                // but only BODY-height on an image line (the caption model — never a
                // char-wide × whole-image-height pillar). `row_caret_band` reads the
                // per-line `caret_band_scale`, the caret's own anchor.
                let (y, row_caret_h) = self.row_caret_band(line, row, line_top);
                rects.push([x, y, w, row_caret_h]);
            }
        }
        // Route through the SAME content clip every other SELECTION-
        // ADJACENT quad uses, so a selection extended past the page's edge (or
        // a diff-preview transcript scrolled past its card) stops painting at
        // that boundary instead of bleeding into the margin — a visual bound
        // only; the selection RANGE above is untouched. Shared by
        // `selection_rects` and `search_match_rects` (both funnel through
        // here). NOT `clip_decorative_rects_to_band` — that owner is for the
        // decorative-overhang emitters.
        self.clip_rects_to_band(rects)
    }

    /// Translucent highlight rects for ALL active search matches (one set per
    /// match, in document order). The CURRENT match gets no distinct color: the
    /// real amber caret already sits on it.
    pub(in crate::render) fn search_match_rects(&self) -> Vec<[f32; 4]> {
        if self.search_matches.is_empty() {
            return Vec::new();
        }
        let lines = self.visible_lines_for_ranges(&self.search_matches);
        // One full shaped-row partition walk for ALL visible matches. Calling
        // `range_rects` per match repeats this O(document) gather and turns a
        // dense search in one wrapped paragraph into a stall.
        let rows_by_line = if lines.is_empty() {
            std::collections::HashMap::new()
        } else {
            self.visual_rows_for_lines(&lines)
        };
        let mut r = Vec::new();
        for &(a, b) in &self.search_matches {
            r.extend(self.range_rects_from_rows((a, b), &rows_by_line));
        }
        r
    }

    #[cfg(test)]
    pub(in crate::render) fn reset_search_rect_work(&self) {
        self.search_rect_work.set(0);
    }

    #[cfg(test)]
    pub(in crate::render) fn search_rect_work(&self) -> usize {
        self.search_rect_work.get()
    }

    pub(in crate::render) fn search_no_matches(&self) -> bool {
        self.search_active && !self.search_query.is_empty() && self.search_matches.is_empty()
    }
}
