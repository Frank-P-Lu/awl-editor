//! Semantic continuous-scroll normalization and viewport-follow policy.

use super::*;

/// Containing-row anchor plus a fixed 1/64px offset within that row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ScrollPos {
    pub row: usize,
    pub px_q: i32,
}

impl ScrollPos {
    pub const SUBPX: i32 = 64;

    pub const fn at_row(row: usize) -> Self {
        Self { row, px_q: 0 }
    }

    pub fn px(self) -> f32 {
        self.px_q as f32 / Self::SUBPX as f32
    }
}

impl TextPipeline {
    /// The sole semantic-scroll-to-document-pixel resolver.
    pub fn scroll_top_px(&self, scroll: ScrollPos) -> f32 {
        self.row_top_px(scroll.row) + scroll.px()
    }

    /// Settled paint/hit-test coordinate. Semantic 1/64px packets accumulate in
    /// [`ScrollPos`], but glyph geometry only moves when they cross a whole logical
    /// pixel so fractional wheel packets never select a fresh raster position.
    pub fn rendered_scroll_top_px(&self, scroll: ScrollPos) -> f32 {
        self.scroll_top_px(scroll).round()
    }

    /// A buffer height tall enough to shape every row THE FRAME ABOUT TO BE
    /// PRESENTED could paint — [`super::ShapeReach::Presentable`]'s budget, and never more
    /// than [`super::TextPipeline::full_shape_height`].
    ///
    /// `shape_until_scroll` always fills from the BUFFER's own top (awl never moves
    /// cosmic-text's `scroll`; it draws the whole shaped document at a pixel
    /// offset), so this budget is measured from the document's first row down, not
    /// from the viewport's:
    ///
    /// * the viewport's own bottom at the current scroll, plus
    /// * [`OFFSCREEN_CULL_MARGIN_ROWS`] — the same band `rects::row_box_visible`
    ///   keeps, so nothing this frame is allowed to paint can fall outside what
    ///   this shapes — plus
    /// * one further window, which is what keeps the truncated document TALLER than
    ///   the viewport at the current scroll and so keeps `max_scroll` above it: a
    ///   short document height would otherwise clamp the scroll and the frame would
    ///   present at a different position than the one asked for.
    ///
    /// The CARET's own line is included unconditionally. It is the one line whose
    /// geometry is read whether or not it is on screen (a wheel scroll leaves the
    /// caret behind), and an unshaped line has no rows to read.
    ///
    /// Everything past this is off-screen by at least a full window at the moment
    /// of the present, and is shaped before the step ends.
    pub(super) fn presentable_shape_height(&self) -> f32 {
        let full = self.full_shape_height();
        let viewport = self.viewport_avail_px(self.window_h);
        let margin = self.metrics.line_height * OFFSCREEN_CULL_MARGIN_ROWS.0;
        // Buffer-relative: `rendered_scroll_top_px` is the document pixel sitting at
        // the top of the viewport, and `line_top` in a run is measured from the same
        // origin.
        let seen_bottom = self.rendered_scroll_top_px(self.scroll) + viewport;
        // The caret's line, read from the row table the LAST fully-shaped pass built
        // (this runs before the restyle that invalidates it), plus the same
        // wrapped-row allowance `full_shape_height` budgets per logical line.
        let caret_bottom =
            self.row_geom
                .line_first_top(&self.buffer, &self.metrics, self.cursor_line)
                + 8.0 * self.metrics.line_height;
        let want = self.metrics.px(TEXT_TOP)
            + seen_bottom.max(caret_bottom)
            + margin
            + viewport
            + self.metrics.line_height;
        want.min(full)
    }

    /// Document viewport after fixed text and menu insets.
    pub fn viewport_avail_px(&self, height: f32) -> f32 {
        (height - self.text_origin_top()).max(1.0)
    }

    fn row_top_q(&self, row: usize) -> i64 {
        (self.row_top_px(row) * ScrollPos::SUBPX as f32).round() as i64
    }

    fn row_span_q(&self, row: usize) -> i64 {
        let total = self.total_visual_rows();
        if row + 1 < total {
            (self.row_top_q(row + 1) - self.row_top_q(row)).max(1)
        } else {
            ((self.total_doc_height() * ScrollPos::SUBPX as f32).round() as i64
                - self.row_top_q(row))
            .max(1)
        }
    }

    fn max_scroll_q(&self, height: f32) -> i64 {
        ((self.total_doc_height() - self.viewport_avail_px(height)).max(0.0)
            * ScrollPos::SUBPX as f32)
            .round() as i64
    }

    /// Resolve a fixed-point document coordinate to its canonical containing
    /// row plus a nonnegative offset strictly within that row.
    fn scroll_pos_at_q(&self, target_q: i64, height: f32) -> ScrollPos {
        let max_q = self.max_scroll_q(height);
        let target_q = target_q.clamp(0, max_q);
        let row = self
            .row_geom
            .containing_row_q(&self.buffer, &self.metrics, target_q);
        ScrollPos {
            row,
            px_q: (target_q - self.row_top_q(row)) as i32,
        }
    }

    /// Canonicalize by walking adjacent row spans. This is deliberately separate
    /// from the absolute-coordinate resolver: wheel packets retain their current
    /// row/remainder and carry locally across real variable-height boundaries.
    fn canonicalize_incremental(
        &self,
        start_row: usize,
        mut offset_q: i64,
        height: f32,
    ) -> ScrollPos {
        let total = self.total_visual_rows();
        if total == 0 {
            return ScrollPos::default();
        }
        let mut row = start_row.min(total - 1);

        while offset_q < 0 && row > 0 {
            row -= 1;
            offset_q += self.row_span_q(row);
        }
        if offset_q < 0 {
            return ScrollPos::default();
        }
        while row + 1 < total {
            let span_q = self.row_span_q(row);
            if offset_q < span_q {
                break;
            }
            offset_q -= span_q;
            row += 1;
        }

        let max_q = self.max_scroll_q(height);
        if self.row_top_q(row).saturating_add(offset_q) > max_q {
            while row > 0 && self.row_top_q(row) > max_q {
                row -= 1;
            }
            while row + 1 < total && self.row_top_q(row + 1) <= max_q {
                row += 1;
            }
            offset_q = max_q - self.row_top_q(row);
        }
        ScrollPos {
            row,
            px_q: offset_q as i32,
        }
    }

    /// Incrementally carry a fixed-point offset across variable-height rows.
    pub fn scroll_by_px(&self, pos: ScrollPos, delta_px: f32, height: f32) -> ScrollPos {
        let pos = self.canonicalize_incremental(pos.row, i64::from(pos.px_q), height);
        let delta_q = (delta_px * ScrollPos::SUBPX as f32).round() as i64;
        self.canonicalize_incremental(pos.row, i64::from(pos.px_q).saturating_add(delta_q), height)
    }

    /// Minimally reveal an affinity-resolved row box, normalizing even when the
    /// row is already visible.
    pub fn scroll_to_show_row_pos(&self, row: usize, scroll: ScrollPos, height: f32) -> ScrollPos {
        let scroll = self.scroll_by_px(scroll, 0.0, height);
        let avail = self.viewport_avail_px(height);
        let row_top = self.row_top_px(row);
        let row_bottom = row_top + self.row_height_px(row);
        let current = self.rendered_scroll_top_px(scroll);
        let target = if self.row_height_px(row) >= avail || row_top < current {
            row_top
        } else if row_bottom > current + avail {
            row_bottom - avail
        } else {
            return scroll;
        };
        let target_q = (target * ScrollPos::SUBPX as f32).round() as i64;
        self.scroll_pos_at_q(target_q, height)
    }

    /// Center a row without quantizing its intra-row target.
    pub fn scroll_to_center_row_pos(&self, row: usize, height: f32) -> ScrollPos {
        let avail = self.viewport_avail_px(height);
        let target = (self.row_top_px(row) + self.row_height_px(row) * 0.5 - avail * 0.5).max(0.0);
        let target_q = (target * ScrollPos::SUBPX as f32).round() as i64;
        self.scroll_pos_at_q(target_q, height)
    }

    /// Preserve an anchored document point through zoom/rewrap.
    pub fn zoom_anchor_scroll_pos(
        &self,
        line: usize,
        col: usize,
        anchor_py: f32,
        height: f32,
    ) -> ScrollPos {
        let row = self.visual_row_of(line, col);
        let target_top =
            zoom_anchor_target_top(self.row_top_px(row), anchor_py, self.text_origin_top());
        let target_q = (target_top * ScrollPos::SUBPX as f32).round() as i64;
        self.scroll_pos_at_q(target_q, height)
    }

    /// Pixel-precise screen top of the visual row containing `(line, col)`.
    pub fn char_screen_top_scroll(&self, line: usize, col: usize, scroll: ScrollPos) -> f32 {
        let row = self.visual_row_of(line, col);
        self.text_origin_top() - self.rendered_scroll_top_px(scroll) + self.row_top_px(row)
    }

    /// Advance- and wrap-aware pixel-to-text hit test at a semantic scroll.
    pub fn hit_test_scroll(&self, px: f32, py: f32, scroll: ScrollPos) -> (usize, usize) {
        let doc_top = self.text_origin_top() - self.rendered_scroll_top_px(scroll);
        let want_top = (py - doc_top).max(0.0);
        let target_x = (px - self.text_left()).max(0.0);
        let mut first_run = true;
        for run in self.buffer.layout_runs() {
            let above_first = first_run && want_top < run.line_top;
            let in_band = want_top >= run.line_top && want_top < run.line_top + run.line_height;
            if above_first || in_band {
                return (run.line_i, self.col_in_run(&run, target_x));
            }
            first_run = false;
        }
        match self.buffer.layout_runs().last() {
            Some(run) => (run.line_i, self.col_in_run(&run, target_x)),
            None => (0, 0),
        }
    }
}
