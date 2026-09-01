//! Cached table shaping and reshape-time reservation.

use super::*;

pub(crate) struct TableGridShaped {
    pub(crate) col_x: Vec<f32>,
    pub(crate) col_w: Vec<f32>,
    pub(crate) cells: Vec<(usize, usize, GlyphBuffer, f32)>,
    pub(crate) row_heights: Vec<f32>,
}

/// THE ONE TABLE-GRID SHAPE SITE (the "merge, don't align" fix for the
/// geometry-computed-twice bug): [`TextPipeline::compute_table_layout`] shapes
/// every table block ONCE per reshape via [`TextPipeline::shape_table_grid`] and
/// WRITES the result here — it's already doing the shaping to compute the
/// row-height RESERVATION, so this just keeps what it built instead of throwing
/// it away. [`TextPipeline::prepare_table_grid`] (the per-frame draw) then only
/// ever READS this cache; it never calls `shape_table_grid` itself. That makes a
/// reservation/draw divergence structurally unrepresentable, closing the real gap
/// the old two-call-sites design had: `TextPipeline::prepare`'s per-frame
/// `sync_wrap_width` re-wraps the document buffer to the LIVE `text_wrap_width()`
/// on a width-only change (page-mode toggle, measure edit, a width-preserving
/// theme reshape) WITHOUT running a full `set_text` reshape — so `reshape_count`
/// does not advance and `compute_table_layout` does not re-run. Before this cache,
/// `prepare_table_grid` still called `shape_table_grid` itself every frame with
/// the FRESH (post-`sync_wrap_width`) `avail`, so on exactly that frame it drew a
/// table shaped at the NEW width while the row it was drawn into was still the
/// row height RESERVED at the OLD width — a taller/shorter drawn grid than the
/// document had made room for, painting over the next row. Now the draw simply
/// reads the SAME shape the reservation used, so on that frame both are equally
/// (and consistently) stale until the next real reshape catches both up together.
///
/// Keyed on `reshape_count` (bumped ONLY by a real `set_text`/`set_text_full`
/// reshape — the exact seam `compute_table_layout` itself runs on, so the key and
/// the write are inseparable). Entries are `(range.start, TableGridShaped)` for
/// every table block with `ncols > 0` found at that reshape — document byte range
/// start is stable across a pure caret move within the same table, and is the
/// same key both `compute_table_layout` and `prepare_table_grid` derive their
/// table list from (`TextPipeline::table_blocks`, itself sourced from the same
/// `md_spans` field both read).
pub(crate) struct TableGridCache {
    version: std::cell::Cell<Option<u64>>,
    pub(crate) entries: std::cell::RefCell<Vec<(usize, TableGridShaped)>>,
}

impl TableGridCache {
    pub(crate) fn new() -> Self {
        Self {
            version: std::cell::Cell::new(None),
            entries: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl TextPipeline {
    /// Table cells live in a separate shaped cache, including theme-authored
    /// inline markdown attrs. Refresh it in the same transaction that re-styles
    /// prose, merging only table-owned row heights so image reservations hold.
    pub(in crate::render) fn refresh_table_cache_for_restyle(&mut self) {
        let Some(text) = self.shaped_key.clone() else {
            return;
        };
        let table_blocks = self.table_blocks();
        let spans = self.md_spans.clone();
        let table_heights = self.compute_table_layout(&text, &spans);
        let mut start = 0usize;
        for (li, line) in text.split('\n').enumerate() {
            if table_blocks
                .iter()
                .any(|(_, range)| range.start <= start && start < range.end)
                && let Some(slot) = self.image_heights.get_mut(li)
            {
                *slot = table_heights.get(li).copied().flatten();
            }
            start += line.len() + 1;
        }
    }

    /// Shape ONE table's cells as inline markdown, size its columns to fit `avail`,
    /// and — WRAP-NOT-CLIP — reshape each cell at its FINAL column width so an
    /// over-wide table wraps within its columns instead of hard-clipping. Returns
    /// the per-column x-offsets + widths ([`crate::markdown::table_column_layout`]),
    /// the reshaped per-cell buffers (each with its wrapped pixel width, for
    /// alignment), and each GRID ROW's height (the max wrapped-cell height across
    /// the row, never below one line-height). PURE font work — no placement, no
    /// culling — so the row-height RESERVATION ([`Self::compute_table_layout`],
    /// which reserves the tall document row so nothing overlaps) and the DRAW pass
    /// ([`Self::prepare_table_grid`]) share ONE layout and can never disagree on how
    /// tall a wrapped row is. A cell with no markup yields `cell_attrs` alone —
    /// byte-identical shaping to raw text; a table that already fits stays
    /// single-line (every row height == one line-height).
    fn shape_table_grid(
        &mut self,
        grid_rows: &[(usize, Vec<String>)],
        ncols: usize,
        avail: f32,
        gap: f32,
        pad: f32,
    ) -> TableGridShaped {
        let m = self.metrics;
        // Keep ordinary cell text on glyphon's `TextArea::default_color`, the
        // same live per-frame ink ordinary prose uses. A baked `base_content`
        // makes the cached cell buffer retain the source world's ink across a
        // live theme switch even though `prepare_table_grid` supplies the new
        // world's default color. Inline markdown spans may still carry their
        // own authored colors; `restyle_all_lines` refreshes this cache along
        // with those spans whenever the active world changes.
        let cell_attrs = self.doc_attrs();
        let body_metrics = GlyphMetrics::new(m.font_size, m.line_height);
        // PASS 1 — shape each non-empty cell to measure BOTH its MAX-content width
        // (the whole cell on one line, at the full writing column) and its
        // MIN-content floor (the widest UNBREAKABLE word, measured by re-wrapping the
        // SAME styled buffer at a 1px width under `Wrap::Word` so words never break).
        // The two feed the CSS auto-table allocation (`table_column_layout`), which
        // floors every column to its longest word so a cell NEVER wraps mid-word.
        // The cell is styled as INLINE markdown (real bold / italic / mono code +
        // zero-width markers) via the SAME span seam prose uses
        // (`spans::cell_inline_attrs`), so both measurements are the STYLED text's.
        let mut maxs = vec![0.0f32; ncols];
        let mut mins = vec![0.0f32; ncols];
        let mut cells: Vec<(usize, usize, GlyphBuffer, f32)> = Vec::new();
        for (gr, (_, row_cells)) in grid_rows.iter().enumerate() {
            for (c, cell) in row_cells.iter().enumerate() {
                if c >= ncols || cell.is_empty() {
                    continue;
                }
                // A cell can't wrap to more visual rows than it has characters, so a
                // per-cell shaping-height of `(chars + 1) * line_height` always lays
                // out every wrapped row into `layout_runs()` (a too-small height would
                // clamp the run iterator and truncate the measurement).
                let tall = m.line_height * (cell.chars().count() as f32 + 1.0);
                let mut buf = GlyphBuffer::new(&mut self.font_system, body_metrics);
                // WORD wrap (not the default WordOrGlyph): a word is never split, so
                // the min-content floor is a true word floor and pass-2 wrapping honors
                // it — the "no cell wraps mid-word" law is structural, not just floored.
                buf.set_wrap(&mut self.font_system, Wrap::Word);
                buf.set_size(&mut self.font_system, Some(avail), Some(tall));
                buf.set_text(
                    &mut self.font_system,
                    cell,
                    &cell_attrs,
                    Shaping::Advanced,
                    None,
                );
                let al = cell_inline_attrs(&cell_attrs, m.line_height, cell);
                if let Some(line) = buf.lines.get_mut(0) {
                    line.set_attrs_list(al);
                }
                buf.shape_until_scroll(&mut self.font_system, false);
                let mut w = 0.0f32;
                for run in buf.layout_runs() {
                    w = w.max(run.line_w);
                }
                // MIN-content: re-wrap the same buffer at a 1px width so every word
                // lands on its own line (Word wrap), then the widest run IS the widest
                // unbreakable word. Restore for pass 2 (which reshapes at box width).
                buf.set_size(&mut self.font_system, Some(1.0), Some(tall));
                buf.shape_until_scroll(&mut self.font_system, false);
                let mut word = 0.0f32;
                for run in buf.layout_runs() {
                    word = word.max(run.line_w);
                }
                maxs[c] = maxs[c].max(w + 2.0 * pad);
                mins[c] = mins[c].max(word + 2.0 * pad);
                cells.push((gr, c, buf, w));
            }
        }
        for c in 0..ncols {
            if maxs[c] <= 0.0 {
                // Blank columns need enough display extent for the provisional
                // empty-cell wash; content columns retain their measured width.
                let empty_min = (m.font_size * 3.0).max(2.0 * pad);
                maxs[c] = empty_min;
                mins[c] = mins[c].max(empty_min);
            } else {
                mins[c] = mins[c].max(2.0 * pad);
            }
        }
        let (col_x, col_w) = crate::markdown::table_column_layout(&mins, &maxs, gap, avail);
        let mut row_heights = vec![m.line_height; grid_rows.len()];
        for (gr, c, buf, w) in cells.iter_mut() {
            let box_w = col_w.get(*c).copied().unwrap_or(avail);
            let wrap_w = (box_w - 2.0 * pad).max(1.0);
            buf.set_size(&mut self.font_system, Some(wrap_w), buf.size().1);
            buf.shape_until_scroll(&mut self.font_system, false);
            let mut mw = 0.0f32;
            let mut rows = 0usize;
            for run in buf.layout_runs() {
                mw = mw.max(run.line_w);
                rows += 1;
            }
            *w = mw;
            let h = rows.max(1) as f32 * m.line_height;
            if let Some(rh) = row_heights.get_mut(*gr) {
                *rh = rh.max(h);
            }
        }
        TableGridShaped {
            col_x,
            col_w,
            cells,
            row_heights,
        }
    }

    /// WRAP-NOT-CLIP row RESERVATION: the per-LOGICAL-LINE reserved row height each
    /// off-cursor GFM table's rows need so a WRAPPED (too-wide) table grows its
    /// document rows instead of the drawn grid overlapping the following content —
    /// the SAME "reserve a tall row" contract inline images use
    /// (`compute_image_layout`), stored in the shared `image_heights` slot and
    /// threaded into [`build_line_attrs`]. Computed at reshape time (O(doc tables),
    /// not per-frame) directly from the fresh `text` + `md_spans`, so it is ready
    /// before the line attrs are built. `None` for every non-table line and for a
    /// table row that fits on one line (no reservation → byte-identical layout);
    /// an all-`None` vector when WYSIWYG is off / not markdown, so a plain doc is
    /// untouched.
    pub(crate) fn compute_table_layout(
        &mut self,
        text: &str,
        md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    ) -> Vec<Option<f32>> {
        let lines: Vec<&str> = text.split('\n').collect();
        let mut heights = vec![None; lines.len().max(1)];
        if !(crate::markdown::wysiwyg_on() && self.md_enabled) || md_spans.is_empty() {
            // Nothing shaped this reshape (WYSIWYG off / no markdown / no table
            // spans) — the cache must not keep serving a PRIOR reshape's stale
            // grids (`prepare_table_grid` never reads it in this state anyway,
            // since it shares the same gate, but an empty cache is the honest
            // shape of "no tables were shaped here").
            self.table_grid_cache.entries.borrow_mut().clear();
            self.table_grid_cache.version.set(None);
            return heights;
        }
        let m = self.metrics;
        let avail = self.text_wrap_width().max(1.0);
        let pad = m.px(TABLE_CELL_PAD_X);
        let gap = m.px(TABLE_COL_GAP);
        let mut starts = Vec::with_capacity(lines.len());
        let mut acc = 0usize;
        for l in &lines {
            starts.push(acc);
            acc += l.len() + 1;
        }
        struct TMeta {
            range: std::ops::Range<usize>,
            grid_rows: Vec<(usize, Vec<String>)>,
            ncols: usize,
        }
        let mut tmetas: Vec<TMeta> = Vec::new();
        for (r, k) in md_spans {
            if *k != crate::markdown::MdKind::ConcealMarkup(crate::markdown::ConcealKind::Table) {
                continue;
            }
            let Some(header_line) = starts.iter().position(|&s| s == r.start) else {
                continue;
            };
            let mut src: Vec<(usize, &str)> = Vec::new();
            let mut li = header_line;
            while li < lines.len() && starts[li] < r.end {
                src.push((li, lines[li]));
                li += 1;
            }
            if src.len() < 2 {
                continue;
            }
            let align_cells = crate::markdown::split_row_cells(src[1].1);
            let mut grid_rows: Vec<(usize, Vec<String>)> = Vec::new();
            for (i, (dl, line)) in src.iter().enumerate() {
                if i == 1 {
                    continue; // the separator row is the rule, no cells
                }
                if i >= 2 && line.trim().is_empty() {
                    continue;
                }
                grid_rows.push((*dl, crate::markdown::split_row_cells(line)));
            }
            let ncols = grid_rows
                .iter()
                .map(|(_, c)| c.len())
                .max()
                .unwrap_or(0)
                .max(align_cells.len());
            tmetas.push(TMeta {
                range: r.clone(),
                grid_rows,
                ncols,
            });
        }
        let mut cache_entries: Vec<(usize, TableGridShaped)> = Vec::new();
        for tm in tmetas {
            if tm.ncols == 0 {
                continue;
            }
            let shaped = self.shape_table_grid(&tm.grid_rows, tm.ncols, avail, gap, pad);
            for (gr, (dl, _)) in tm.grid_rows.iter().enumerate() {
                let h = shaped.row_heights[gr];
                if h > m.line_height + 0.5
                    && let Some(slot) = heights.get_mut(*dl)
                {
                    *slot = Some(h);
                }
            }
            cache_entries.push((tm.range.start, shaped));
        }
        *self.table_grid_cache.entries.borrow_mut() = cache_entries;
        self.table_grid_cache.version.set(Some(self.reshape_count));
        heights
    }

    /// TEST-ONLY: the CACHED row heights for the table block whose byte range
    /// starts at `range_start` — a direct peek at the ONE shape site
    /// ([`TableGridCache`]) so a test can compare what `compute_table_layout`
    /// reserved against what `prepare_table_grid` actually reads to draw,
    /// without needing a GPU pixel diff. `None` if no table was cached at that
    /// range (off / no such table).
    #[cfg(test)]
    pub(crate) fn table_grid_cache_row_heights(&self, range_start: usize) -> Option<Vec<f32>> {
        self.table_grid_cache
            .entries
            .borrow()
            .iter()
            .find(|(s, _)| *s == range_start)
            .map(|(_, g)| g.row_heights.clone())
    }

    /// TEST-ONLY: the CACHED per-column x-offsets + widths for the table block
    /// whose byte range starts at `range_start` — a direct peek at
    /// [`TableGridCache`] so a test can assert every cell's x-range stays
    /// within the writing column WITHOUT a GPU pixel diff. `None` if no table
    /// is cached at that range.
    #[cfg(test)]
    pub(crate) fn table_grid_cache_col_geometry(
        &self,
        range_start: usize,
    ) -> Option<(Vec<f32>, Vec<f32>)> {
        self.table_grid_cache
            .entries
            .borrow()
            .iter()
            .find(|(s, _)| *s == range_start)
            .map(|(_, g)| (g.col_x.clone(), g.col_w.clone()))
    }

    #[cfg(test)]
    pub(crate) fn table_cell_lines_drawn(&self) -> Vec<usize> {
        self.last_table_cell_lines.borrow().clone()
    }

    /// TEST-ONLY: `(line, raw source)` for every X-RAYED table row this frame
    /// (see [`Self::prepare_table_xray`]) — the caret's own row (if a caret
    /// sits on a table row) PLUS every row the active selection touches. The
    /// complement of [`Self::table_cell_lines_drawn`]: a line here never
    /// appears there (its grid cells were skipped so its raw source could
    /// float instead), so a test can assert BOTH "the grid stayed off" and
    /// "the real `|` source is what floats" without a GPU pixel diff.
    #[cfg(test)]
    pub(crate) fn xray_lines_report(&self) -> Vec<(usize, String)> {
        self.xray
            .iter()
            .map(|x| (x.line, x.source.clone()))
            .collect()
    }
}
