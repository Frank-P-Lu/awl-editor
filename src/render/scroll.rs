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

    /// Document viewport after fixed text and menu insets.
    pub fn viewport_avail_px(&self, height: f32) -> f32 {
        (height - TEXT_TOP - self.menubar_reserve()).max(1.0)
    }

    fn row_top_q(&self, row: usize) -> i64 {
        (self.row_top_px(row) * ScrollPos::SUBPX as f32).round() as i64
    }

    /// Resolve a fixed-point document coordinate to its canonical containing
    /// row plus a nonnegative offset strictly within that row.
    fn scroll_pos_at_q(&self, target_q: i64, height: f32) -> ScrollPos {
        let max_q = ((self.total_doc_height() - self.viewport_avail_px(height)).max(0.0)
            * ScrollPos::SUBPX as f32)
            .round() as i64;
        let target_q = target_q.clamp(0, max_q);
        let row = self
            .row_geom
            .containing_row_q(&self.buffer, &self.metrics, target_q);
        ScrollPos {
            row,
            px_q: (target_q - self.row_top_q(row)) as i32,
        }
    }

    /// Incrementally carry a fixed-point offset across variable-height rows.
    pub fn scroll_by_px(&self, pos: ScrollPos, delta_px: f32, height: f32) -> ScrollPos {
        let current_q = self.row_top_q(pos.row) + i64::from(pos.px_q);
        let delta_q = (delta_px * ScrollPos::SUBPX as f32).round() as i64;
        self.scroll_pos_at_q(current_q.saturating_add(delta_q), height)
    }

    /// Minimally reveal an affinity-resolved row box, normalizing even when the
    /// row is already visible.
    pub fn scroll_to_show_row_pos(&self, row: usize, scroll: ScrollPos, height: f32) -> ScrollPos {
        let scroll = self.scroll_by_px(scroll, 0.0, height);
        let avail = self.viewport_avail_px(height);
        let row_top = self.row_top_px(row);
        let row_bottom = row_top + self.row_height_px(row);
        let current = self.scroll_top_px(scroll);
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
            zoom_anchor_target_top(self.row_top_px(row), anchor_py, self.menubar_reserve());
        let target_q = (target_top * ScrollPos::SUBPX as f32).round() as i64;
        self.scroll_pos_at_q(target_q, height)
    }

    /// Pixel-precise screen top of the visual row containing `(line, col)`.
    pub fn char_screen_top_scroll(&self, line: usize, col: usize, scroll: ScrollPos) -> f32 {
        let row = self.visual_row_of(line, col);
        TEXT_TOP + self.menubar_reserve() - self.scroll_top_px(scroll) + self.row_top_px(row)
    }

    /// Advance- and wrap-aware pixel-to-text hit test at a semantic scroll.
    pub fn hit_test_scroll(&self, px: f32, py: f32, scroll: ScrollPos) -> (usize, usize) {
        let doc_top = TEXT_TOP + self.menubar_reserve() - self.scroll_top_px(scroll);
        let want_top = (py - doc_top).max(0.0);
        let target_x = (px - self.text_left()).max(0.0);
        let mut first_run = true;
        for run in self.buffer.layout_runs() {
            let above_first = first_run && want_top < run.line_top;
            let in_band = want_top >= run.line_top && want_top < run.line_top + run.line_height;
            if above_first || in_band {
                return (run.line_i, Self::col_in_run(&run, target_x));
            }
            first_run = false;
        }
        match self.buffer.layout_runs().last() {
            Some(run) => (run.line_i, Self::col_in_run(&run, target_x)),
            None => (0, 0),
        }
    }
}
