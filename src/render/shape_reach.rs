//! THE SHAPING REACH — how far through the document one reshape goes, and the
//! two halves of a step that let a picker arrow present before it finishes.
//!
//! [`TextPipeline::full_shape_height`] budgets every visual row, which is
//! what keeps the whole document's geometry real: an unshaped row has no geometry
//! at all, and the scroll jumps / images draw at the wrong height the moment
//! anything asks about one. It is also, on a long document, ~95% of a
//! theme-preview arrow's cost, spent on rows no frame can draw.
//!
//! A preview shapes [`ShapeReach::Presentable`] and presents. Newer previews may
//! supersede its off-screen tail; a quiet settle, commit, or revert pays the
//! latest world's debt. Every settled path stays [`ShapeReach::Whole`].
//!
//! The split is taken only where it can PAY — where the paintable band stops
//! short of the document's last row. That is a ROW question, and asking it in
//! heights against the deliberate over-budget instead
//! ([`super::TextPipeline::presentable_reach_height`]) makes a step near the
//! document's end declare a debt it has no deferred rows to justify, and pay two
//! whole-document relayouts for it.

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
/// A preview shapes [`ShapeReach::Presentable`] — everything its frame can paint.
/// Newer selections may replace its off-screen tail before a quiet settle pays it.
/// Every settled path stays [`ShapeReach::Whole`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShapeReach {
    /// Every visual row of the document — the settled reach.
    Whole,
    /// Only as far as the frame about to be presented can paint, WHEN stopping
    /// there leaves rows unshaped. Then it leaves a tail owed to the quiet settle.
    /// Where the paintable band already reaches the document's last row
    /// — a short document, or a scroll near the end — there is nothing to defer
    /// and this reach shapes the whole document like [`Self::Whole`], owing
    /// nothing (see [`TextPipeline::presentable_reach_height`]).
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

    /// The buffer height a [`ShapeReach::Presentable`] step ACTUALLY shapes to:
    /// [`Self::presentable_shape_height`] where narrowing to it defers rows, and
    /// the whole `full` budget where it does not.
    ///
    /// THE TWO QUESTIONS ARE IN DIFFERENT UNITS, and only one of them is about
    /// the split's saving. "Is this budget shorter than
    /// [`Self::full_shape_height`]?" is a HEIGHT question, and `full_shape_height`
    /// deliberately over-budgets — ~8 wrapped rows per logical line, plus every
    /// reserved image/table height — so on ordinary prose it is several times the
    /// document's real extent. "Does this budget stop short of the last row?" is a
    /// ROW question, and it is the one the split's saving depends on: the earlier
    /// present is bought by leaving rows unshaped, so a budget that already covers
    /// every row buys nothing. Ask the height question and the two come apart
    /// exactly where the answer matters — at a scroll near the document's END the
    /// paintable band reaches the last row while sitting far under the
    /// over-estimate, so every arrow declares a debt and pays a whole-document
    /// `set_size` (cosmic-text relayouts EVERY laid-out line on any height change,
    /// width-only though layout is) for zero deferred rows, twice: once narrowing
    /// here and once restoring in [`Self::finish_shape_tail`].
    ///
    /// So the compare is against the DOCUMENT — `total_doc_height`, the bottom of
    /// the last visual row the last fully-shaped pass produced, which is the same
    /// row table `presentable_shape_height` above reads the caret's line from.
    ///
    /// THE GUESS IS SAFE IN BOTH DIRECTIONS, which is what lets it be made BEFORE
    /// the shape rather than after it. The reshape can change the document's own
    /// height (the wrap width is face-independent but the glyph advances inside it
    /// are not, so a face fits a different number of characters per row), so this
    /// is exact only when the reshape leaves the height alone — which is every
    /// non-wrapping document, and near enough on the rest that the two ends of the
    /// depth curve are never in doubt. Guess "whole" where narrowing would have
    /// helped and the step is today's settled step: correct, one arrow's earlier
    /// present forgone. Guess "narrow" where it does not help and the step is the
    /// status quo: correct, one wasted tail. Neither can leave a row unshaped,
    /// because narrowing and owing are still set from the SAME value one line
    /// apart in [`Self::theme_font_adopt`].
    ///
    /// Deciding here rather than after the shape is also what keeps `height_opt`
    /// off the narrow budget on every step that owes nothing. The alternative —
    /// narrow always, then ask the shaped runs whether anything was actually
    /// deferred — answers the row question exactly, and then has a narrowed buffer
    /// and no tail to restore it: either it pays the relayout it just proved
    /// pointless, or it lets a narrow `height_opt` outlive the step, where the next
    /// restyle that does NOT re-budget (a conceal reveal, a spell repaint) shapes
    /// against it and truncates the document.
    fn presentable_reach_height(&self, full: f32, settled_doc_height: f32) -> f32 {
        let want = self.presentable_shape_height();
        if want < settled_doc_height {
            want
        } else {
            full
        }
    }

    /// Is an off-screen shaping TAIL still owed (see [`ShapeReach`])? True only
    /// between a [`ShapeReach::Presentable`] reshape and the quiet/commit/revert
    /// settle that pays it.
    pub fn shape_tail_owed(&self) -> bool {
        self.shape_tail_settled_height.is_some()
    }

    /// Pay any owed off-screen shaping TAIL: restore the WHOLE reach and shape
    /// everything the presentable budget stopped short of, so the document is fully
    /// shaped again and every row carries real geometry. Returns whether there was
    /// anything to do.
    ///
    /// It is idempotent and cheap when nothing is owed. During a picker burst each
    /// new preview replaces the incomplete shaping with its own world; the quiet
    /// settle calls this once for the final selection.
    ///
    /// The result is byte-identical to what a [`ShapeReach::Whole`] reshape would
    /// have produced in one pass: `set_size` re-lays the prefix cosmic-text already
    /// shaped and shapes the rest against the SAME `full_shape_height` (the text,
    /// metrics and reserved image/table heights cannot change inside a step), and
    /// the row geometry is rebuilt from the finished runs.
    pub fn finish_shape_tail(&mut self) -> bool {
        if self.shape_tail_settled_height.take().is_none() {
            return false;
        }
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
    /// apart, in either direction and past the end of the step as well as within
    /// it. A `Presentable` budget that would reach the whole document anyway
    /// ([`Self::presentable_reach_height`]) is not taken at all, so `height_opt`
    /// never leaves the full budget on a step that has no tail to pay.
    fn theme_font_adopt(&mut self, new_font: &'static str, new_theme: usize, reach: ShapeReach) {
        self.reshape_count += 1;
        self.shaped_font = new_font;
        self.shaped_theme = new_theme;
        let width = Some(self.text_wrap_width());
        let full = self.full_shape_height();
        let settled_doc_height = self
            .shape_tail_settled_height
            .unwrap_or_else(|| self.total_doc_height());
        let shape_h = match reach {
            ShapeReach::Whole => full,
            ShapeReach::Presentable => self.presentable_reach_height(full, settled_doc_height),
        };
        if shape_h < full {
            self.shape_tail_settled_height = Some(settled_doc_height);
        } else {
            self.shape_tail_settled_height = None;
        }
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
