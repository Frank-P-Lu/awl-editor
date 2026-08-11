//! THE SHAPING REACH — how far through the document one reshape goes, and the
//! two halves of a step that let a picker arrow present before it finishes.
//!
//! [`TextPipeline::full_shape_height`] budgets every visual row, which is
//! what keeps the whole document's geometry real: an unshaped row has no geometry
//! at all, and the scroll jumps / images draw at the wrong height the moment
//! anything asks about one. It is also, on a long document, ~95% of a
//! theme-preview arrow's cost, spent on rows no frame can draw.
//!
//! So a preview step splits that one reshape in two WITHOUT deferring any of it
//! past the step: shape [`ShapeReach::Presentable`], present, then
//! [`super::TextPipeline::finish_shape_tail`] before the next event is handled.
//! Every settled path — commit, revert, capture, tests — stays
//! [`ShapeReach::Whole`], and so does every headless path.

use super::*;

/// How far past the window's own edges — in LINE HEIGHTS — a row still counts as
/// paintable. The ONE owner of the off-screen cull band: [`rects`]'
/// `row_box_visible` rejects an ornament outside it, and
/// [`TextPipeline::presentable_shape_height`] shapes at least out to it, so the two
/// can never disagree about which rows a frame is allowed to need.
pub const OFFSCREEN_CULL_MARGIN_ROWS: Rows = Rows(8.0);

/// How far a shaping pass REACHES through the document.
///
/// [`TextPipeline::full_shape_height`] budgets every visual row, which is what
/// keeps the whole document's geometry real (an unshaped row has no geometry at
/// all — the scroll-jump / wrong-image-height class that budget exists to
/// prevent). It is also, on a long document, ~95% of a theme-preview arrow's cost,
/// spent on rows no frame can draw.
///
/// So a preview step splits that one reshape in two WITHOUT deferring any of it
/// past the step: shape [`ShapeReach::Presentable`] — everything the frame about to
/// be presented can possibly paint — present, then finish the tail
/// ([`TextPipeline::finish_shape_tail`]) before the next event is handled. Every
/// settled path stays [`ShapeReach::Whole`], and so does every headless path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShapeReach {
    /// Every visual row of the document — the settled reach.
    Whole,
    /// Only as far as the frame about to be presented can paint. Leaves a TAIL
    /// owed, which the same step must pay.
    Presentable,
}

impl TextPipeline {
    /// A buffer height tall enough to shape every row THE FRAME ABOUT TO BE
    /// PRESENTED could paint — [`ShapeReach::Presentable`]'s budget, and never more
    /// than [`TextPipeline::full_shape_height`].
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

    /// Is an off-screen shaping TAIL still owed (see [`ShapeReach`])? True only
    /// between a [`ShapeReach::Presentable`] reshape and the
    /// [`Self::finish_shape_tail`] that pays it — never at a settled step boundary,
    /// which is the property the reach law and the picker-sweep bench read it for.
    pub fn shape_tail_owed(&self) -> bool {
        self.shape_tail_owed
    }

    /// Pay any owed off-screen shaping TAIL: restore the WHOLE reach and shape
    /// everything the presentable budget stopped short of, so the document is fully
    /// shaped again and every row carries real geometry. Returns whether there was
    /// anything to do.
    ///
    /// This is the second half of one step, not deferred work: the live App calls it
    /// immediately after the present that the narrow reach bought, inside the same
    /// event handler, so no input can be handled against a partially shaped
    /// document. It is also idempotent and cheap when nothing is owed, which is why
    /// the settled retint calls it unconditionally as a backstop.
    ///
    /// The result is byte-identical to what a [`ShapeReach::Whole`] reshape would
    /// have produced in one pass: `set_size` re-lays the prefix cosmic-text already
    /// shaped and shapes the rest against the SAME `full_shape_height` (the text,
    /// metrics and reserved image/table heights cannot change inside a step), and
    /// the row geometry is rebuilt from the finished runs.
    pub fn finish_shape_tail(&mut self) -> bool {
        if !self.shape_tail_owed {
            return false;
        }
        self.shape_tail_owed = false;
        let width = Some(self.text_wrap_width());
        let shape_h = self.full_shape_height();
        self.buffer
            .set_size(&mut self.font_system, width, Some(shape_h));
        self.buffer.shape_until_scroll(&mut self.font_system, false);
        // The tail's rows are new geometry: the row table built from the truncated
        // runs (and every cache keyed on its generation) has to rebuild.
        self.row_geom.invalidate();
        self.buffer.set_redraw(true);
        true
    }

    /// The FONT half of a theme switch (the expensive half — a full-document
    /// reshape; the theme-burst profile measured it dominating every picker
    /// preview step, which is why the live preview defers it to a settle).
    ///
    /// Re-shape the whole document when the new world uses a DIFFERENT effective
    /// display face than the one the document is shaped with (so the glyph SHAPES
    /// switch — mono <-> serif <-> sans <-> slab) OR a DIFFERENT palette than the
    /// one the per-span text colors were baked under (so a same-face world hop still
    /// re-tints the syntax/markdown spans — the Magpie -> Bombora stale-color
    /// bug). The text + zoom are unchanged, so `restyle_all_lines` (below) re-lays
    /// every line's attrs in the new family + span colors and reshapes once. A hop
    /// to the SAME world (an idle re-preview back) skips this and stays free.
    /// Compares the EFFECTIVE face (`doc_family` → the world's mono on a CODE
    /// buffer, else its display font), so two worlds that share a display font but
    /// differ in `mono` (e.g. Quokka/Bowerbird, both IBM Plex Sans) still reshape
    /// a code buffer when their mono differs; and two worlds that share the effective
    /// face but differ in palette still reshape to re-bake the span colors.
    pub fn sync_theme_font(&mut self, reach: ShapeReach) {
        let new_font = self.doc_family();
        let new_theme = theme::active_index();
        // Reshape when the effective FACE changed (glyph shapes) OR the world's
        // PALETTE changed on a buffer that BAKES per-span colors (syntax/markdown —
        // those were frozen under `shaped_theme` and go stale on a same-face world
        // hop; a color-less prose buffer reads its ink live and needs nothing).
        // Either way the cure is one `restyle_all_lines` — it re-lays every line's
        // attrs (family + colors) and reshapes once. A same-face, same-world call
        // stays a no-op via this compare, mirroring the original `shaped_font` guard.
        let theme_recolor = new_theme != self.shaped_theme && self.has_baked_theme_colors();
        if new_font != self.shaped_font || theme_recolor {
            self.theme_font_adopt(new_font, new_theme, reach);
            self.restyle_all_lines();
        }
    }

    /// The FONT-phase reconfigure of a theme switch: bump the reshape count, adopt the
    /// new effective face + palette generation, and rewrap the document to the new
    /// face's column. The ONE owner of this step, shared by [`Self::sync_theme_font`]
    /// and the timed [`Self::sync_theme_font_timed`] (so the two can never drift). The
    /// following `restyle_all_lines` does the actual shape + row-geom invalidation.
    ///
    /// NOTE: the redundant `buffer.set_text` (a WHOLE-document cosmic-text reshape in
    /// the new plain family) was dropped here — `restyle_all_lines` ALREADY re-lays
    /// every line's attrs in the new family (via `doc_attrs()`) AND covers the per-line
    /// markdown / heading / CJK spans, then reshapes the document. The old `set_text`
    /// shaped every line in the new face only to have `restyle_all_lines` immediately
    /// re-lay + reshape it again — one full reshape per theme-preview step for nothing.
    /// The text is unchanged by a theme switch, so the buffer already holds it; we only
    /// need the new wrap size + the restyle. Byte-identical (same final attrs/shape).
    /// Re-derive the wrap width from the live page COLUMN, never the buffer's own
    /// (possibly stale) size — preserving `self.buffer.size().0` here would carry a
    /// divergent edge-to-edge width through a theme switch and leave the page running
    /// off the right edge. Set it BEFORE restyling so the new-face reshape wraps at the
    /// right width.
    ///
    /// `reach` picks the shaping BUDGET the following restyle fills (see
    /// [`ShapeReach`]). This is the ONE site that can narrow it, and it is also the
    /// one site that records the resulting debt — narrowed and owed cannot drift
    /// apart. A `Presentable` budget that turns out to be the whole document anyway
    /// (a short document, or a scroll near its end) owes nothing.
    fn theme_font_adopt(&mut self, new_font: &'static str, new_theme: usize, reach: ShapeReach) {
        self.reshape_count += 1;
        self.shaped_font = new_font;
        self.shaped_theme = new_theme;
        let width = Some(self.text_wrap_width());
        let full = self.full_shape_height();
        let shape_h = match reach {
            ShapeReach::Whole => full,
            ShapeReach::Presentable => self.presentable_shape_height(),
        };
        self.shape_tail_owed = shape_h < full;
        self.buffer
            .set_size(&mut self.font_system, width, Some(shape_h));
    }

    /// LIVE-ONLY (DEBUG settle readout): run the SAME work as [`Self::sync_theme_font`]
    /// — the identical guard, the identical `theme_font_adopt` + `restyle_all_lines`
    /// steps — but stamp each phase boundary and return the reshape-side phase millis
    /// (font-adopt, reshape, row-geom), or `None` when the guard finds NO work (so a
    /// no-op switch never clobbers the last meaningful readout). The caller (the live
    /// App, behind `debug_on()`) folds the present-side atlas + present phases in on the
    /// settled frame. The plain `sync_theme_font` — the ONLY variant the headless path
    /// calls — reads no clock, so a capture never touches an `Instant` here.
    ///
    /// The row-geom walk is FORCED here (a plain reshape leaves it lazy for the next
    /// prepare) purely so its cost is timed as its own phase — identical work moved a
    /// few microseconds earlier, warming the cache the frame's prepare would rebuild
    /// anyway, so the rendered frame stays byte-identical.
    pub fn sync_theme_font_timed(
        &mut self,
        reach: ShapeReach,
    ) -> Option<crate::themeswitch::SwitchPhases> {
        use crate::clock::Instant; // wasm-safe (`web_time` on wasm); native `std`.
        use crate::themeswitch::{SwitchPhase, SwitchPhases};
        let new_font = self.doc_family();
        let new_theme = theme::active_index();
        let theme_recolor = new_theme != self.shaped_theme && self.has_baked_theme_colors();
        if new_font == self.shaped_font && !theme_recolor {
            return None; // no reshape work — nothing to time, keep the last readout.
        }
        let ms = |d: std::time::Duration| d.as_secs_f32() * 1000.0;
        let t0 = Instant::now();
        self.theme_font_adopt(new_font, new_theme, reach);
        let t1 = Instant::now();
        self.restyle_all_lines();
        let t2 = Instant::now();
        let _ = self.row_geom.total_height(&self.buffer, &self.metrics);
        let t3 = Instant::now();
        let mut phases = SwitchPhases::default();
        phases.record(SwitchPhase::Font, ms(t1 - t0));
        phases.record(SwitchPhase::Reshape, ms(t2 - t1));
        phases.record(SwitchPhase::RowGeom, ms(t3 - t2));
        Some(phases)
    }
}
