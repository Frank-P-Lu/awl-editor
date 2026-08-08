//! Per-frame table grid placement and horizontal panning.

use super::table_layout::TableGridShaped;
use super::*;

impl TextPipeline {
    /// WYSIWYG TABLE GRID: place every off-cursor GFM table's cells by PIXEL column
    /// (a proportional face can't align with space-padding — that's the bug this
    /// fixes) via one [`TextArea`] per cell, plus ONE faint header-separator rule.
    /// The [`Self::prepare_ornaments`] pattern applied to a rectangular block: the
    /// source rows are concealed to zero-width by
    /// [`crate::markdown::ConcealKind::Table`], and the grid draws in their place.
    /// A table that FITS occupies one row per source line (header, the rule row,
    /// then body); a too-wide table WRAPS its cells and each grown row reserves a
    /// tall document row via [`Self::compute_table_layout`], so grid and source
    /// agree on the row geometry (`RowGeom` reads the reserved heights).
    ///
    /// REVEAL = TRUE SOURCE SWAP, per row (WYSIWYG amendment, corrected): a table
    /// the caret is INSIDE stays a drawn grid — every row EXCEPT the caret's own
    /// still uploads its cells at full ink. Only the ONE row the caret currently
    /// occupies uploads NO grid cells at all; its raw source floats over that
    /// row's band instead (pushed after the cell loop below, see the x-ray). This
    /// is the drop-to-source-on-cursor contract applied per row rather than
    /// parking the whole block — grid and source never share a row's pixels, but
    /// they DO still share the table (a multi-row table mid-edit reads as "grid,
    /// with one row temporarily as text", not "the whole table vanished"). Also
    /// drawn (all rows) for a non-markdown / table-less buffer trivially (there
    /// are none) and skipped wholesale with WYSIWYG off, so a default capture
    /// stays byte-identical.
    ///
    /// Cost: O(visible tables' cells) to UPLOAD. The SHAPING itself is done ONCE
    /// per reshape by [`Self::compute_table_layout`] (the ONE shape site, see
    /// [`TableGridCache`]) — this pass only ever READS that cached geometry
    /// (column widths are the max over every row, so a partly-scrolled table
    /// keeps STABLE columns rather than jumping) and places it; off-screen tables
    /// are culled whole (their cached geometry is simply never turned into
    /// `TextArea`s). Column math ([`crate::markdown::table_column_layout`] /
    /// [`crate::markdown::table_align_offset`]) is pure + unit-tested and already
    /// baked into the cached geometry.
    pub(crate) fn prepare_table_grid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        use crate::markdown::ColAlign;
        self.table_report.borrow_mut().clear();
        #[cfg(test)]
        self.last_table_cell_lines.borrow_mut().clear();

        let wysiwyg = crate::markdown::wysiwyg_on();
        let blocks = if wysiwyg && self.md_enabled {
            self.table_blocks()
        } else {
            Vec::new()
        };
        if blocks.is_empty() {
            self.table_rule_pipeline
                .prepare(device, queue, width, height, &[]);
            self.table_renderer
                .prepare(
                    device,
                    queue,
                    &mut self.font_system,
                    &mut self.atlas,
                    &self.viewport,
                    Vec::new(),
                    &mut self.swash_cache,
                )
                .map_err(|e| anyhow::anyhow!("glyphon table prepare failed: {e:?}"))?;
            return Ok(());
        }

        let m = self.metrics;
        let text_left = self.text_left();
        let avail = self.text_wrap_width().max(1.0);
        let pad = m.px(TABLE_CELL_PAD_X);
        let rule_thick = m.px(TABLE_RULE_THICKNESS).max(1.0);
        let cursor_byte = self.line_doc_byte_start(self.cursor_line);
        // SELECTION REVEAL: the SAME touched-line byte extent `wysiwyg_reveals`
        // widens its own caret-only rule with (`selection_touch_bytes`) — a
        // table whose range the selection overlaps is "revealed" exactly like
        // one the caret sits inside.
        let selection_touch = selection_touch_bytes(
            self.selection,
            |i| self.line_doc_byte_start(i),
            |i| {
                self.buffer
                    .lines
                    .get(i)
                    .map(|l| l.text().len())
                    .unwrap_or(0)
            },
        );
        let content = theme::base_content().to_glyphon();

        // PHASE A — parse each block into owned data (no font work yet).
        struct Meta {
            range: (usize, usize),
            ncols: usize,
            aligns: Vec<ColAlign>,
            sep_doc_line: usize,
            revealed: bool,
            visible: bool,
            grid_rows: Vec<(usize, Vec<String>)>,
        }
        let mut metas: Vec<Meta> = Vec::new();
        for (header_line, range) in &blocks {
            let mut src_lines: Vec<String> = Vec::new();
            let mut li = *header_line;
            let mut b = range.start;
            while li < self.buffer.lines.len() && b < range.end {
                let t = self.buffer.lines[li].text();
                b += t.len() + 1;
                src_lines.push(t.to_string());
                li += 1;
            }
            if src_lines.len() < 2 {
                continue; // a real table always has header + separator
            }
            let align_cells = crate::markdown::split_row_cells(&src_lines[1]);
            let mut grid_rows: Vec<(usize, Vec<String>)> = Vec::new();
            for (i, line) in src_lines.iter().enumerate() {
                if i == 1 {
                    continue;
                }
                if i >= 2 && line.trim().is_empty() {
                    continue; // a trailing blank swept into the range is not a row
                }
                grid_rows.push((*header_line + i, crate::markdown::split_row_cells(line)));
            }
            let ncols = grid_rows
                .iter()
                .map(|(_, c)| c.len())
                .max()
                .unwrap_or(0)
                .max(align_cells.len());
            let aligns: Vec<ColAlign> = (0..ncols)
                .map(|c| {
                    align_cells
                        .get(c)
                        .map(|s| crate::markdown::parse_col_align(s))
                        .unwrap_or(ColAlign::None)
                })
                .collect();
            let last_doc_line = *header_line + src_lines.len().saturating_sub(1);
            let visible = (*header_line..=last_doc_line).any(|dl| self.line_ornament_visible(dl));
            metas.push(Meta {
                range: (range.start, range.end),
                ncols,
                aligns,
                sep_doc_line: *header_line + 1,
                revealed: range.contains(&cursor_byte)
                    || selection_touch
                        .as_ref()
                        .is_some_and(|st| st.start < range.end && range.start < st.end),
                visible,
                grid_rows,
            });
        }

        // PHASE B — READ (never reshape) the geometry `compute_table_layout` already
        // shaped for VISIBLE blocks, from the ONE shape site
        // ([`TableGridCache`] — see its doc comment for why the draw pass must not
        // shape its own copy). A `None` here means the block is off-screen (culled,
        // matching the pre-existing "shape nothing off-screen" behavior — its report
        // carries no measured widths) or, degenerately, that no cache entry exists
        // for this range (would mean `compute_table_layout` and this frame's own
        // `table_blocks()` disagreed on the table list, which they cannot: both
        // derive from the SAME `self.md_spans` field).
        let table_cache = self.table_grid_cache.entries.borrow();
        let shaped: Vec<Option<&TableGridShaped>> = metas
            .iter()
            .map(|meta| {
                if !meta.visible || meta.ncols == 0 {
                    return None;
                }
                table_cache
                    .iter()
                    .find(|(start, _)| *start == meta.range.0)
                    .map(|(_, s)| s)
            })
            .collect();

        let view_w = avail;
        let pan_bar_thick = m.px(crate::render::TABLE_PAN_BAR_THICKNESS).max(1.0);
        let muted = theme::muted().to_glyphon();
        // THE X-RAY floats: every caret- or selection-revealed table row's RAW
        // SOURCE, each shaped NON-WRAPPING into its own LOCAL buffer (so `areas`
        // can borrow them below without fighting the renderer's own `&mut self`
        // borrows). Drawn dim ("the markdown bones") over the dimmed grid cells;
        // the caret's OWN row pans by `x.pan` to keep the caret column visible,
        // every other (selection-only) row floats flush-left (`x.pan == 0`).
        let mut xray_floats: Vec<(GlyphBuffer, f32, f32, f32, usize)> = Vec::new();
        for x in self.xray.clone() {
            let bodym = GlyphMetrics::new(m.font_size, m.line_height);
            let base = self.doc_attrs().color(muted);
            let mut buf = GlyphBuffer::new(&mut self.font_system, bodym);
            buf.set_wrap(&mut self.font_system, Wrap::None);
            buf.set_size(&mut self.font_system, None, Some(m.line_height * 2.0));
            buf.set_text(
                &mut self.font_system,
                &x.source,
                &base,
                Shaping::Advanced,
                None,
            );
            buf.shape_until_scroll(&mut self.font_system, false);
            xray_floats.push((buf, x.top, x.height, x.pan, x.line));
        }
        let xray_lines: Vec<usize> = xray_floats.iter().map(|f| f.4).collect();
        let mut areas: Vec<TextArea> = Vec::new();
        let mut rule_rects: Vec<[f32; 4]> = Vec::new();
        let mut pan_writeback: Option<(usize, f32)> = None;
        for (mi, meta) in metas.iter().enumerate() {
            let (col_x, col_w) = match &shaped[mi] {
                Some(s) => (s.col_x.as_slice(), s.col_w.as_slice()),
                None => (&[][..], &[][..]),
            };
            self.table_report
                .borrow_mut()
                .push(crate::render::TableReport {
                    range: meta.range,
                    rows: meta.grid_rows.len(),
                    cols: meta.ncols,
                    col_widths: col_w.to_vec(),
                    revealed: meta.revealed,
                });
            let Some(s) = &shaped[mi] else {
                continue;
            };
            // THE X-RAY table (the caret is inside): the grid stays DRAWN (the
            // document never reflowed — the source rows are still concealed), the
            // caret/selection-touched row(s) are DIMMED, and their raw source
            // floats over them (pushed after the loop). No reading-pan applies
            // while editing/selecting — the float(s) own the horizontal.
            if meta.revealed {
                let content_w = col_x
                    .last()
                    .zip(col_w.last())
                    .map(|(x, w)| x + w)
                    .unwrap_or(0.0);
                // TRUE SWAP, not dim-under-float (the fix): every caret- or
                // selection-touched row uploads NO grid cells at all — its x-ray
                // source float (pushed after this loop, centered in the row's own
                // band) is the ONLY text drawn in that band, per the
                // drop-to-source-on-cursor/selection contract. Every OTHER row of
                // a revealed table still draws its grid at full ink — only the
                // touched rows drop to source; the block never "parks" wholesale.
                for (gr, c, buf, cw) in &s.cells {
                    let doc_line = meta.grid_rows[*gr].0;
                    if xray_lines.contains(&doc_line) {
                        continue;
                    }
                    #[cfg(test)]
                    self.last_table_cell_lines.borrow_mut().push(doc_line);
                    let top = self.line_ornament_top(doc_line);
                    let box_left = text_left + col_x[*c];
                    let box_w = col_w[*c];
                    let off = crate::markdown::table_align_offset(meta.aligns[*c], box_w, *cw, pad);
                    let clip_left = box_left.max(0.0) as i32;
                    let clip_right = (box_left + box_w).clamp(0.0, width as f32) as i32;
                    areas.push(TextArea {
                        buffer: buf,
                        left: box_left + off,
                        top,
                        scale: 1.0,
                        bounds: self.clip_text_bounds(TextBounds {
                            left: clip_left,
                            top: 0,
                            right: clip_right,
                            bottom: height as i32,
                        }),
                        default_color: content,
                        custom_glyphs: &[],
                    });
                }
                let sep_top = self.line_ornament_top(meta.sep_doc_line);
                let rule_y = sep_top + (m.line_height - rule_thick) * 0.5;
                if content_w > 0.0 {
                    rule_rects.push([text_left, rule_y, content_w, rule_thick]);
                }
                continue;
            }
            let content_w = col_x
                .last()
                .zip(col_w.last())
                .map(|(x, w)| x + w)
                .unwrap_or(0.0);
            let pan_req = self
                .table_pan
                .filter(|(start, _)| *start == meta.range.0)
                .map(|(_, o)| o)
                .unwrap_or(0.0);
            let pan = crate::markdown::table_pan_clamp(pan_req, content_w, view_w);
            if self
                .table_pan
                .is_some_and(|(start, _)| start == meta.range.0)
            {
                pan_writeback = Some((meta.range.0, pan));
            }
            // At pan 0 the grid grows into the margins (clip only at the canvas);
            // once panned, clip to the writing column so shifted content never
            // spills into the LEFT margin.
            let (vp_l, vp_r) = if pan > 0.0 {
                (text_left, text_left + view_w)
            } else {
                (0.0, width as f32)
            };
            for (gr, c, buf, cw) in &s.cells {
                let doc_line = meta.grid_rows[*gr].0;
                #[cfg(test)]
                self.last_table_cell_lines.borrow_mut().push(doc_line);
                let top = self.line_ornament_top(doc_line);
                let box_left = text_left + col_x[*c] - pan;
                let box_w = col_w[*c];
                let off = crate::markdown::table_align_offset(meta.aligns[*c], box_w, *cw, pad);
                // Each cell WRAPS within its column (shaped at the column's inner
                // width), so it never overruns its neighbour; the clip is the
                // column box intersected with the table viewport (a safety net for
                // an unbreakable over-wide token, and the pan's left-spill guard).
                let clip_left = box_left.max(vp_l).max(0.0) as i32;
                let clip_right = (box_left + box_w).min(vp_r).clamp(0.0, width as f32) as i32;
                areas.push(TextArea {
                    buffer: buf,
                    left: box_left + off,
                    top,
                    scale: 1.0,
                    bounds: self.clip_text_bounds(TextBounds {
                        left: clip_left,
                        top: 0,
                        right: clip_right,
                        bottom: height as i32,
                    }),
                    default_color: content,
                    custom_glyphs: &[],
                });
            }
            let sep_top = self.line_ornament_top(meta.sep_doc_line);
            let rule_y = sep_top + (m.line_height - rule_thick) * 0.5;
            let rule_w = if pan > 0.0 {
                (content_w - pan).min(view_w).max(0.0)
            } else {
                content_w
            };
            if rule_w > 0.0 {
                rule_rects.push([text_left, rule_y, rule_w, rule_thick]);
            }
            if pan > 0.0
                && let Some((last_dl, _)) = meta.grid_rows.last()
            {
                let last_gr = s.row_heights.len().saturating_sub(1);
                let bottom = self.line_ornament_top(*last_dl) + s.row_heights[last_gr];
                if let Some(bar) = crate::markdown::table_pan_bar(
                    content_w,
                    view_w,
                    pan,
                    text_left,
                    bottom,
                    pan_bar_thick,
                ) {
                    rule_rects.push(bar);
                }
            }
        }
        // THE X-RAY FLOATS — drawn LAST so each composites over its own dimmed
        // grid row: every caret/selection-touched row's raw source as one
        // non-wrapping line (the caret's own panned by `pan` to keep the caret
        // visible; every other flush-left at `pan == 0`), centred in its row
        // band, clipped to the writing column (so a long source doesn't spill
        // into the margins).
        for (buf, top, row_h, pan, _line) in xray_floats.iter() {
            let float_top = top + (row_h - m.line_height) * 0.5;
            areas.push(TextArea {
                buffer: buf,
                left: text_left - pan,
                top: float_top,
                scale: 1.0,
                bounds: self.clip_text_bounds(TextBounds {
                    left: text_left.max(0.0) as i32,
                    top: 0,
                    right: (text_left + view_w).clamp(0.0, width as f32) as i32,
                    bottom: height as i32,
                }),
                default_color: muted,
                custom_glyphs: &[],
            });
        }
        // Persist the clamped pan so a stale offset self-corrects once the grid
        // narrows (a theme reshape / measure change), and the live gesture reads a
        // sane base next frame.
        if let Some(wb) = pan_writeback {
            self.table_pan = Some(wb);
        }

        self.table_rule_pipeline
            .prepare(device, queue, width, height, &rule_rects);
        self.table_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon table prepare failed: {e:?}"))?;
        Ok(())
    }

    pub fn tables_report(&self) -> Vec<crate::render::TableReport> {
        self.table_report.borrow().clone()
    }

    pub fn try_table_pan(&mut self, px: f32, py: f32, scroll: ScrollPos, dx: f32) -> bool {
        if !(crate::markdown::wysiwyg_on() && self.md_enabled) {
            return false;
        }
        let (line, _) = self.hit_test_scroll(px, py, scroll);
        let line_byte = self.line_doc_byte_start(line);
        let Some((start, _range)) = self
            .table_blocks()
            .into_iter()
            .find(|(_, r)| r.start <= line_byte && line_byte < r.end)
            .map(|(_, r)| (r.start, r))
        else {
            return false;
        };
        let report = self.table_report.borrow();
        let Some(t) = report.iter().find(|t| t.range.0 == start) else {
            return false;
        };
        let n = t.col_widths.len();
        if n == 0 {
            return false;
        }
        let gap = self.metrics.px(crate::render::TABLE_COL_GAP);
        let content_w: f32 = t.col_widths.iter().sum::<f32>() + gap * (n.saturating_sub(1) as f32);
        drop(report);
        let view_w = self.text_wrap_width().max(1.0);
        if content_w <= view_w + 1e-3 {
            return false; // fits — nothing to pan
        }
        let cur = self
            .table_pan
            .filter(|(s, _)| *s == start)
            .map(|(_, o)| o)
            .unwrap_or(0.0);
        let next = crate::markdown::table_pan_clamp(cur - dx, content_w, view_w);
        self.table_pan = Some((start, next));
        true
    }
}
