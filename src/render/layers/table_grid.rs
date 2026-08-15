//! Per-frame table grid placement and horizontal panning.

use super::table_layout::TableGridShaped;
use super::*;

struct TableMeta {
    range: (usize, usize),
    ncols: usize,
    aligns: Vec<crate::markdown::ColAlign>,
    sep_doc_line: usize,
    revealed: bool,
    visible: bool,
    grid_rows: Vec<(usize, Vec<String>)>,
}

struct TableXrayFloat {
    buffer: GlyphBuffer,
    top: f32,
    height: f32,
    pan: f32,
    line: usize,
}

#[derive(Clone, Copy)]
struct TablePlacementContext {
    text_left: f32,
    view_w: f32,
    line_height: f32,
    pad: f32,
    rule_thick: f32,
    pan_bar_thick: f32,
    width: u32,
    height: u32,
    content: glyphon::Color,
    muted: glyphon::Color,
    table_pan: Option<(usize, f32)>,
}

struct TablePlacement<'a> {
    areas: Vec<TextArea<'a>>,
    rule_rects: Vec<[f32; 4]>,
    reports: Vec<crate::render::TableReport>,
    pan_writeback: Option<(usize, f32)>,
    #[cfg(test)]
    drawn_lines: Vec<usize>,
}

fn table_content_width(shaped: &TableGridShaped) -> f32 {
    shaped
        .col_x
        .last()
        .zip(shaped.col_w.last())
        .map(|(x, w)| x + w)
        .unwrap_or(0.0)
}

fn place_shaped_table<'a>(
    placed: &mut TablePlacement<'a>,
    meta: &TableMeta,
    shaped: &'a TableGridShaped,
    xray_lines: &[usize],
    context: TablePlacementContext,
    line_top: &impl Fn(usize) -> f32,
    clip_bounds: &impl Fn(TextBounds) -> TextBounds,
) {
    let content_w = table_content_width(shaped);
    let pan = if meta.revealed {
        0.0
    } else {
        let requested = context
            .table_pan
            .filter(|(start, _)| *start == meta.range.0)
            .map(|(_, offset)| offset)
            .unwrap_or(0.0);
        let clamped = crate::markdown::table_pan_clamp(requested, content_w, context.view_w);
        if context
            .table_pan
            .is_some_and(|(start, _)| start == meta.range.0)
        {
            placed.pan_writeback = Some((meta.range.0, clamped));
        }
        clamped
    };
    let (viewport_left, viewport_right) = if pan > 0.0 {
        (context.text_left, context.text_left + context.view_w)
    } else {
        (0.0, context.width as f32)
    };

    for (grid_row, column, buffer, cell_width) in &shaped.cells {
        let doc_line = meta.grid_rows[*grid_row].0;
        if meta.revealed && xray_lines.contains(&doc_line) {
            continue;
        }
        #[cfg(test)]
        placed.drawn_lines.push(doc_line);
        let top = line_top(doc_line);
        let box_left = context.text_left + shaped.col_x[*column] - pan;
        let box_width = shaped.col_w[*column];
        let offset = crate::markdown::table_align_offset(
            meta.aligns[*column],
            box_width,
            *cell_width,
            context.pad,
        );
        let clip_left = box_left.max(viewport_left).max(0.0) as i32;
        let clip_right = (box_left + box_width)
            .min(viewport_right)
            .clamp(0.0, context.width as f32) as i32;
        placed.areas.push(TextArea {
            buffer,
            left: box_left + offset,
            top,
            scale: 1.0,
            bounds: clip_bounds(TextBounds {
                left: clip_left,
                top: 0,
                right: clip_right,
                bottom: context.height as i32,
            }),
            default_color: context.content,
            custom_glyphs: &[],
        });
    }

    let separator_top = line_top(meta.sep_doc_line);
    let rule_y = separator_top + (context.line_height - context.rule_thick) * 0.5;
    let rule_width = if pan > 0.0 {
        (content_w - pan).min(context.view_w).max(0.0)
    } else {
        content_w
    };
    if rule_width > 0.0 {
        placed
            .rule_rects
            .push([context.text_left, rule_y, rule_width, context.rule_thick]);
    }
    if pan > 0.0
        && let Some((last_doc_line, _)) = meta.grid_rows.last()
    {
        let last_grid_row = shaped.row_heights.len().saturating_sub(1);
        let bottom = line_top(*last_doc_line) + shaped.row_heights[last_grid_row];
        if let Some(bar) = crate::markdown::table_pan_bar(
            content_w,
            context.view_w,
            pan,
            context.text_left,
            bottom,
            context.pan_bar_thick,
        ) {
            placed.rule_rects.push(bar);
        }
    }
}

fn append_table_xrays<'a>(
    areas: &mut Vec<TextArea<'a>>,
    xrays: &'a [TableXrayFloat],
    context: TablePlacementContext,
    clip_bounds: &impl Fn(TextBounds) -> TextBounds,
) {
    for xray in xrays {
        let float_top = xray.top + (xray.height - context.line_height) * 0.5;
        areas.push(TextArea {
            buffer: &xray.buffer,
            left: context.text_left - xray.pan,
            top: float_top,
            scale: 1.0,
            bounds: clip_bounds(TextBounds {
                left: context.text_left.max(0.0) as i32,
                top: 0,
                right: (context.text_left + context.view_w).clamp(0.0, context.width as f32) as i32,
                bottom: context.height as i32,
            }),
            default_color: context.muted,
            custom_glyphs: &[],
        });
    }
}

fn place_table_grid<'a>(
    metas: &[TableMeta],
    cache: &'a [(usize, TableGridShaped)],
    xrays: &'a [TableXrayFloat],
    context: TablePlacementContext,
    line_top: impl Fn(usize) -> f32,
    clip_bounds: impl Fn(TextBounds) -> TextBounds,
) -> TablePlacement<'a> {
    let mut placed = TablePlacement {
        areas: Vec::new(),
        rule_rects: Vec::new(),
        reports: Vec::with_capacity(metas.len()),
        pan_writeback: None,
        #[cfg(test)]
        drawn_lines: Vec::new(),
    };
    let xray_lines: Vec<usize> = xrays.iter().map(|x| x.line).collect();

    for meta in metas {
        let shaped = if meta.visible && meta.ncols > 0 {
            cache
                .iter()
                .find(|(start, _)| *start == meta.range.0)
                .map(|(_, shaped)| shaped)
        } else {
            None
        };
        let col_widths = shaped.map_or_else(Vec::new, |s| s.col_w.clone());
        placed.reports.push(crate::render::TableReport {
            range: meta.range,
            rows: meta.grid_rows.len(),
            cols: meta.ncols,
            col_widths,
            revealed: meta.revealed,
        });
        let Some(shaped) = shaped else {
            continue;
        };

        place_shaped_table(
            &mut placed,
            meta,
            shaped,
            &xray_lines,
            context,
            &line_top,
            &clip_bounds,
        );
    }

    append_table_xrays(&mut placed.areas, xrays, context, &clip_bounds);
    placed
}

impl TextPipeline {
    fn parse_table_metas(&self, blocks: &[(usize, std::ops::Range<usize>)]) -> Vec<TableMeta> {
        let cursor_byte = self.line_doc_byte_start(self.cursor_line);
        let selection_touch = selection_touch_bytes(
            self.selection,
            |line| self.line_doc_byte_start(line),
            |line| {
                self.buffer
                    .lines
                    .get(line)
                    .map(|line| line.text().len())
                    .unwrap_or(0)
            },
        );
        blocks
            .iter()
            .filter_map(|(header_line, range)| {
                let mut source_lines = Vec::new();
                let mut line = *header_line;
                let mut byte = range.start;
                while line < self.buffer.lines.len() && byte < range.end {
                    let text = self.buffer.lines[line].text();
                    byte += text.len() + 1;
                    source_lines.push(text.to_string());
                    line += 1;
                }
                if source_lines.len() < 2 {
                    return None;
                }
                let alignment_cells = crate::markdown::split_row_cells(&source_lines[1]);
                let grid_rows: Vec<_> = source_lines
                    .iter()
                    .enumerate()
                    .filter(|(index, line)| *index != 1 && (*index < 2 || !line.trim().is_empty()))
                    .map(|(index, line)| {
                        (*header_line + index, crate::markdown::split_row_cells(line))
                    })
                    .collect();
                let ncols = grid_rows
                    .iter()
                    .map(|(_, cells)| cells.len())
                    .max()
                    .unwrap_or(0)
                    .max(alignment_cells.len());
                let aligns = (0..ncols)
                    .map(|column| {
                        alignment_cells
                            .get(column)
                            .map(|cell| crate::markdown::parse_col_align(cell))
                            .unwrap_or(crate::markdown::ColAlign::None)
                    })
                    .collect();
                let last_doc_line = *header_line + source_lines.len().saturating_sub(1);
                let selection_reveals = selection_touch
                    .as_ref()
                    .is_some_and(|touch| touch.start < range.end && range.start < touch.end);
                Some(TableMeta {
                    range: (range.start, range.end),
                    ncols,
                    aligns,
                    sep_doc_line: *header_line + 1,
                    revealed: range.contains(&cursor_byte) || selection_reveals,
                    visible: (*header_line..=last_doc_line)
                        .any(|line| self.line_ornament_visible(line)),
                    grid_rows,
                })
            })
            .collect()
    }

    fn shape_table_xray_floats(
        &mut self,
        metrics: Metrics,
        muted: glyphon::Color,
    ) -> Vec<TableXrayFloat> {
        let mut floats = Vec::with_capacity(self.xray.len());
        for xray in self.xray.clone() {
            let glyph_metrics = GlyphMetrics::new(metrics.font_size, metrics.line_height);
            let attrs = self.doc_attrs().color(muted);
            let mut buffer = GlyphBuffer::new(&mut self.font_system, glyph_metrics);
            buffer.set_wrap(&mut self.font_system, Wrap::None);
            buffer.set_size(&mut self.font_system, None, Some(metrics.line_height * 2.0));
            buffer.set_text(
                &mut self.font_system,
                &xray.source,
                &attrs,
                Shaping::Advanced,
                None,
            );
            buffer.shape_until_scroll(&mut self.font_system, false);
            floats.push(TableXrayFloat {
                buffer,
                top: xray.top,
                height: xray.height,
                pan: xray.pan,
                line: xray.line,
            });
        }
        floats
    }

    /// WYSIWYG TABLE GRID: place every off-cursor GFM table's cells by PIXEL column
    /// (a proportional face can't align with space-padding — that's the bug this
    /// fixes) via one [`TextArea`] per cell, plus ONE faint header-separator rule.
    /// The [`Self::prepare_ornaments`] pattern applied to a rectangular block: the
    /// source rows are concealed to zero-width by
    /// [`crate::markdown::ConcealKind::Table`], and the grid draws in their place.
    /// A table that FITS occupies one row per source line (header, the rule row,
    /// then body); a too-wide table WRAPS its cells and reserves tall rows through
    /// [`Self::compute_table_layout`], keeping grid and source geometry aligned.
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
    /// `TextArea`s). Pure, unit-tested column math is already baked into the cache.
    pub(crate) fn prepare_table_grid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
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
        let content = theme::base_content().to_glyphon();
        let metas = self.parse_table_metas(&blocks);

        let muted = theme::muted().to_glyphon();
        let xray_floats = self.shape_table_xray_floats(m, muted);
        let context = TablePlacementContext {
            text_left,
            view_w: avail,
            line_height: m.line_height,
            pad,
            rule_thick,
            pan_bar_thick: m.px(crate::render::TABLE_PAN_BAR_THICKNESS).max(1.0),
            width,
            height,
            content,
            muted,
            table_pan: self.table_pan,
        };
        let table_cache = self.table_grid_cache.entries.borrow();
        let placed = place_table_grid(
            &metas,
            &table_cache,
            &xray_floats,
            context,
            |line| self.line_ornament_top(line),
            |bounds| self.clip_text_bounds(bounds),
        );
        *self.table_report.borrow_mut() = placed.reports;
        #[cfg(test)]
        {
            *self.last_table_cell_lines.borrow_mut() = placed.drawn_lines;
        }
        if let Some(pan) = placed.pan_writeback {
            self.table_pan = Some(pan);
        }

        self.table_rule_pipeline
            .prepare(device, queue, width, height, &placed.rule_rects);
        self.table_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                placed.areas,
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
