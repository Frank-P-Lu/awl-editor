//! Semantic continuous-scroll normalization and viewport-follow policy.

use super::*;

/// VIRTUAL breathing room past the last line, in `line_height` rows.
///
/// Typing at the end of a note is the commonest posture this editor has, and a
/// minimal-reveal follow leaves the caret riding the window's bottom edge there.
/// This is the air below it. Modest by choice — the calm register, not iA
/// Writer's half-viewport typing line, which is what TYPEWRITER mode is for and
/// which stays opt-in.
pub(super) const END_PAD_ROWS: Rows = Rows(3.0);

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

    /// The virtual space past the last line: scrollable, never drawn into, and
    /// **never written**. It is not text and no code path can turn it into text —
    /// it exists only as an addend on the scroll extent below and as the reveal
    /// slack in [`TextPipeline::scroll_to_show_row_pos`]. `Buffer::disk_bytes`
    /// remains the sole author of what lands on disk and never consults it.
    ///
    /// Zero while the document FITS the viewport: a note shorter than the window
    /// already has blank room under its last line, and inventing more would make
    /// a one-screen document scroll for no reason. The gate is the same
    /// `doc_height <= viewport` question `geometry::max_scroll_rows` already asks.
    pub(super) fn end_pad_px(&self, height: f32) -> f32 {
        if self.total_doc_height() <= self.viewport_avail_px(height) {
            0.0
        } else {
            self.metrics.line_height * END_PAD_ROWS.0
        }
    }

    /// How much of [`Self::end_pad_px`] lies below `row` — the pad LESS whatever
    /// real document is still under that row, floored at zero.
    ///
    /// This is what makes the feature end-of-document breathing room rather than
    /// a blanket bottom scrolloff: a row with a screenful of text beneath it
    /// borrows nothing, the last row borrows the whole pad, and the rows between
    /// ramp continuously — so following the caret downward never jumps.
    pub(super) fn end_pad_below_row(&self, row: usize, height: f32) -> f32 {
        let pad = self.end_pad_px(height);
        let row_bottom = self.row_top_px(row) + self.row_height_px(row);
        (pad - (self.total_doc_height() - row_bottom)).clamp(0.0, pad)
    }

    /// **THE ONE OWNER of how deep the document scrolls.** Every document scroll
    /// path resolves here — the wheel and a selection drag through
    /// `canonicalize_incremental`, cursor-follow / the typewriter pin / the zoom
    /// anchor through `scroll_pos_at_q` — so the virtual end pad is added once,
    /// and a wheel can reach exactly the scroll `Cmd-Down` lands on.
    ///
    /// `geometry::max_scroll_rows` is NOT a second owner of this quantity: it
    /// clamps the history workspace's own diff scroll, in whole rows, on a
    /// different scroll value.
    fn max_scroll_q(&self, height: f32) -> i64 {
        ((self.total_doc_height() + self.end_pad_px(height) - self.viewport_avail_px(height))
            .max(0.0)
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
    /// row is already visible — together with whatever VIRTUAL end-of-document
    /// space lies below that row ([`Self::end_pad_below_row`]).
    ///
    /// This is the DEFAULT cursor-follow, and the pad is why the caret stops
    /// riding the window's bottom edge while you type at the end of a note. Away
    /// from the end the slack is zero and the reveal is exactly as minimal as it
    /// always was. Pure — caret row, viewport and document height, no clock — so
    /// a `--keys` capture renders the settled result deterministically.
    pub fn scroll_to_show_row_pos(&self, row: usize, scroll: ScrollPos, height: f32) -> ScrollPos {
        let scroll = self.scroll_by_px(scroll, 0.0, height);
        let avail = self.viewport_avail_px(height);
        let row_top = self.row_top_px(row);
        let row_bottom = row_top + self.row_height_px(row) + self.end_pad_below_row(row, height);
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

    /// Signed pixel overshoot of `py` past the writing column's vertical
    /// band: negative above [`Self::text_origin_top`], positive below
    /// `height`, zero strictly inside. Both edges are screen-fixed (neither
    /// reads `scroll`), so a drag held at the same physical spot reports the
    /// same overshoot no matter how far the document has already scrolled —
    /// the quantity [`drag_scroll_rate`] and [`Self::drag_scroll_step`] read.
    pub fn drag_scroll_overshoot(&self, py: f32, height: f32) -> f32 {
        let top = self.text_origin_top();
        if py < top {
            py - top
        } else if py > height {
            py - height
        } else {
            0.0
        }
    }

    /// Whether the overshoot at `py` earns a nonzero drag-scroll rate — the
    /// SAME dead-zone test [`Self::drag_scroll_step`] applies internally,
    /// exposed so a caller deciding whether to keep RE-ARMING a periodic
    /// re-check (rather than taking a step right now) doesn't re-derive the
    /// dead-zone rule. Without this, a pointer resting just past the edge but
    /// still inside the dead zone would spin an idle-timer re-arm forever for
    /// a scroll that never actually happens.
    pub fn drag_scroll_active(&self, py: f32, height: f32) -> bool {
        drag_scroll_rate(self.drag_scroll_overshoot(py, height)) > 0.0
    }

    /// ONE drag-scroll tick, through the two owners that already exist:
    /// [`Self::scroll_by_px`] advances `scroll` by the overshoot-derived rate
    /// (`dt` seconds' worth, in the overshot direction), then
    /// [`Self::hit_test_scroll`] resolves `(px, py)` against that ADVANCED
    /// scroll — so the caller extends a selection to wherever the moving
    /// content now sits under a pointer that never had to move itself.
    ///
    /// `None` when the pointer sits inside the band (nothing to scroll — the
    /// caller falls through to its ordinary in-band hit-test) or `dt` is
    /// non-positive (a drag's first tick, before any real time has elapsed
    /// past the edge; the caller primes its clock on that same tick and gets
    /// a real `dt` next call).
    pub fn drag_scroll_step(
        &self,
        scroll: ScrollPos,
        px: f32,
        py: f32,
        height: f32,
        dt: f32,
    ) -> Option<(ScrollPos, usize, usize)> {
        let overshoot = self.drag_scroll_overshoot(py, height);
        if overshoot == 0.0 || dt <= 0.0 {
            return None;
        }
        let rate = drag_scroll_rate(overshoot);
        if rate <= 0.0 {
            return None;
        }
        let delta = rate * dt * overshoot.signum();
        let new_scroll = self.scroll_by_px(scroll, delta, height);
        let (line, col) = self.hit_test_scroll(px, py, new_scroll);
        Some((new_scroll, line, col))
    }
}

/// Overshoot right at the column edge (sub-pixel geometry noise, a drag that
/// merely grazes the boundary) earns no scroll at all.
const DRAG_SCROLL_DEAD_ZONE_PX: f32 = 6.0;

/// Overshoot past the dead zone at which the rate reaches its cap.
const DRAG_SCROLL_RAMP_PX: f32 = 140.0;

/// The rate the instant the dead zone clears — visibly moving, not a
/// barely-perceptible crawl.
const DRAG_SCROLL_MIN_RATE: f32 = 80.0;

/// The rate cap (px/sec): fast enough to reach the far end of a long
/// document within a few held seconds, slow enough that a screenful never
/// blurs past unseen. TASTE TUNABLE — flagged for live review, per
/// CLAUDE.md's harness-reach discipline: this map is design INTENT the
/// headless harness can prove is honored exactly, never that the honored
/// curve FEELS right in the hand.
const DRAG_SCROLL_MAX_RATE: f32 = 1400.0;

/// Overshoot-to-scroll-speed curve for a drag held past the writing column's
/// top/bottom edge (px/sec magnitude; direction is the caller's sign on the
/// overshoot). Zero within [`DRAG_SCROLL_DEAD_ZONE_PX`] of the edge; beyond
/// it, an EASED ramp (quadratic — gentle just past the dead zone, steep
/// nearer the cap) up to [`DRAG_SCROLL_MAX_RATE`] at
/// [`DRAG_SCROLL_RAMP_PX`] overshoot, then held flat past that.
pub fn drag_scroll_rate(overshoot_px: f32) -> f32 {
    let overshoot = overshoot_px.abs();
    if overshoot <= DRAG_SCROLL_DEAD_ZONE_PX {
        return 0.0;
    }
    let t = ((overshoot - DRAG_SCROLL_DEAD_ZONE_PX) / DRAG_SCROLL_RAMP_PX).min(1.0);
    DRAG_SCROLL_MIN_RATE + (DRAG_SCROLL_MAX_RATE - DRAG_SCROLL_MIN_RATE) * t * t
}
