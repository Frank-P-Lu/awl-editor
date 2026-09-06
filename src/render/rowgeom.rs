//! VARIABLE-ROW GEOMETRY — the scroll<->pixel cache for non-uniform (heading) rows.
//!
//! With heading lines a document's visual rows are no longer a uniform `line_height`
//! tall, so the scroll<->pixel conversion can no longer use `row_index * line_height`.
//! [`RowGeom`] holds, per visual row in document order (as `layout_runs()` yields them
//! — ascending `line_top`), the row's top y relative to the buffer top and its height,
//! plus the document's total pixel height and the total visual-row count. All four are
//! lazily built from the shaped runs and dropped together when the geometry changes.
//!
//! Unlike the caret geometry next door (which stays inherent on [`super::TextPipeline`]
//! because it reads the cursor/glyph/baseline state pervasively), this is the ONE
//! genuine owning-decouple: `RowGeom` owns its `RefCell`/`Cell` caches and takes the
//! only two things it reads — the shaped [`GlyphBuffer`] and the [`Metrics`] (for the
//! unshaped fallback) — as narrow params. So `TextPipeline` holds a `row_geom: RowGeom`
//! field and DELEGATES `row_top_px` / `row_height_px` / `total_doc_height` /
//! `total_visual_rows` to it, replacing its inline cache with `row_geom.invalidate()`
//! at every shaped-geometry seam. Pure cache mechanics moved verbatim → byte-identical.

use super::*;

#[cfg(test)]
static IN_PLACE_ROW_BORROWS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
static VISUAL_ROW_CLONES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
static REPORT_ROW_BORROWS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(super) fn reset_in_place_row_borrow_count() {
    IN_PLACE_ROW_BORROWS.store(0, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(test)]
pub(super) fn in_place_row_borrow_count() -> usize {
    IN_PLACE_ROW_BORROWS.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
pub(super) fn note_visual_row_clone() {
    VISUAL_ROW_CLONES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(test)]
pub(super) fn reset_layout_report_ownership_counts() {
    VISUAL_ROW_CLONES.store(0, std::sync::atomic::Ordering::Relaxed);
    REPORT_ROW_BORROWS.store(0, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(test)]
pub(super) fn visual_row_clone_count() -> usize {
    VISUAL_ROW_CLONES.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
pub(super) fn report_row_borrow_count() -> usize {
    REPORT_ROW_BORROWS.load(std::sync::atomic::Ordering::Relaxed)
}

/// The first-row top (and baseline) reported for a logical line that has NO shaped
/// run — `layout_runs()` stopped before reaching it, so the line genuinely has no
/// geometry yet. Positive infinity, because every consumer of `line_first_top`
/// tests it against a viewport band (`rects::row_box_visible`, the selection
/// band's per-line bottom), and "below everything" is the only answer that makes
/// those tests REJECT a line whose position is unknown. `0.0` — the obvious
/// default — instead places it at the document top, where those same tests accept
/// it and the frame paints an ornament for a row it cannot see.
///
/// A fully shaped document has no such line, so this is unreachable at every
/// settled step boundary; it exists for the one presented frame of a
/// [`super::ShapeReach::Presentable`] step, and for a degenerate unshaped buffer.
pub(super) const UNSHAPED_LINE_TOP: f32 = f32::INFINITY;

/// The lazily-built variable-row-height geometry table for one shaped buffer (see the
/// module docs). Owned by [`super::TextPipeline`] as its `row_geom` field.
pub(super) struct RowGeom {
    /// Lazily-cached total visual-row count for the currently-shaped buffer.
    /// Invalidated (set to `None`) whenever the buffer is reshaped or its metrics
    /// change; recomputed on demand by [`Self::total_visual_rows`]. Counting rows
    /// walks every shaped run, so caching keeps the per-frame / per-keystroke
    /// `app.rs` reads free.
    total: std::cell::Cell<Option<usize>>,
    /// Per visual row in document order: the row's top y relative to the buffer top,
    /// and (parallel) its height; plus the document's total pixel height. Built
    /// lazily from the shaped runs by [`Self::ensure`] and dropped by
    /// [`Self::invalidate`].
    tops: std::cell::RefCell<Option<Vec<f32>>>,
    heights: std::cell::RefCell<Option<Vec<f32>>>,
    doc_height: std::cell::Cell<f32>,
    /// Per LOGICAL line: the buffer-relative top y of that line's FIRST visual row
    /// (`line_first_top`). Built in the SAME `layout_runs()` walk as `tops`, so the
    /// ornament CULL can read a rule/bullet line's top in O(1) instead of calling
    /// the whole-doc `visual_rows(li)` per candidate. Indexed by logical line;
    /// dropped with the rest by [`Self::invalidate`].
    line_tops: std::cell::RefCell<Option<Vec<f32>>>,
    /// Per LOGICAL line: the buffer-relative BASELINE y of that line's FIRST visual
    /// row (`line_first_baseline`) — cosmic-text's own `LayoutRun::line_y`, i.e.
    /// `line_top + (line_height - glyph_height)/2 + max_ascent`, read straight off
    /// the REAL shaped run rather than approximated from the metrics. Built in the
    /// SAME walk as `line_tops` (one extra field per row, no extra pass). The item
    /// 65 fold-affordance baseline-alignment fix reads this to hang the quiet "…
    /// N lines" tail / expand chevron on a collapsed heading's OWN baseline instead
    /// of merely centering the small glyph in the heading's tall (grown) row box —
    /// which used to read as "floating" above the heading's ink, especially on a
    /// big H1. Indexed by logical line; dropped with the rest by [`Self::invalidate`].
    line_baselines: std::cell::RefCell<Option<Vec<f32>>>,
    /// Per LOGICAL line: the buffer-relative top y of that line's **LAST** visual row
    /// (`line_last_top`) — the wrap-aware counterpart of [`Self::line_tops`], filled
    /// by the SAME `layout_runs()` walk (last write wins, no extra pass). The
    /// blockquote pull-quote's CLOSING mark hangs on the block's final row, which is
    /// the last WRAPPED row of its last logical line, not that line's first.
    line_last_tops: std::cell::RefCell<Option<Vec<f32>>>,
    /// Full visual-row partition assembled in the SAME shaped-run walk as the
    /// scalar row table. Layout consumers and the report share this owner.
    frame_rows: std::cell::RefCell<Option<Vec<FrameVisualRow>>>,
    /// Reports only borrow a partition after glyphon prepared this generation.
    /// Before that seal, the report seam returns `None` instead of shaping.
    frame_sealed: std::cell::Cell<bool>,
    /// SINGLE-SLOT memo of the most-recently-requested logical line's
    /// [`VisualRow`]s — in the per-frame caret path that line is the CURSOR line.
    /// [`super::TextPipeline::visual_rows`] is O(every shaped run in the document)
    /// because it filters the whole `layout_runs()` stream, and the caret geometry
    /// reads it ~4× per redraw (block width, row scale, row top, glyph x), so a
    /// gliding caret rebuilt that wrap geometry 4× a frame, uncached. This memo
    /// holds the last line's rows so calls 2–4 (and every idle glide frame, where
    /// the cursor line is unchanged) clone the cached vector instead of re-walking
    /// the runs. Built lazily on the first `visual_rows(line)` read and dropped by
    /// [`Self::invalidate`] — which fires at EVERY shaped-geometry seam (reshape /
    /// zoom / DPI / restyle / sync-wrap) and NEVER on a cursor move, so the memo is
    /// automatically correct: a motion keeps the same shaped runs, so the cached
    /// rows stay valid; anything that re-shapes clears it. Holds one line at a time
    /// (the cursor line dominates the per-frame reads); the cold up/down oracle
    /// reads of `line ± 1` simply miss and rebuild.
    rows_line: std::cell::Cell<Option<usize>>,
    rows: std::cell::RefCell<Option<Vec<VisualRow>>>,
    /// SHAPED-GEOMETRY GENERATION — bumped by every [`Self::invalidate`], i.e. at
    /// every seam where the shaped runs (and so every derived pixel geometry)
    /// change: reshape, zoom/DPI, restyle, sync-wrap. Consumers that cache
    /// geometry DERIVED from the shaped runs (the spell-squiggle / nit-underline
    /// protos in `rects.rs`) key their caches on this, so they are exactly as
    /// fresh as the row table itself — anything that would stale them bumps it.
    generation: std::cell::Cell<u64>,
}

impl RowGeom {
    /// An empty cache; everything is built lazily on the first geometry read.
    pub(super) fn new() -> Self {
        Self {
            total: std::cell::Cell::new(None),
            tops: std::cell::RefCell::new(None),
            heights: std::cell::RefCell::new(None),
            doc_height: std::cell::Cell::new(0.0),
            line_tops: std::cell::RefCell::new(None),
            line_baselines: std::cell::RefCell::new(None),
            line_last_tops: std::cell::RefCell::new(None),
            frame_rows: std::cell::RefCell::new(None),
            frame_sealed: std::cell::Cell::new(false),
            rows_line: std::cell::Cell::new(None),
            rows: std::cell::RefCell::new(None),
            generation: std::cell::Cell::new(0),
        }
    }

    /// The current shaped-geometry generation (see the field docs). Monotonic;
    /// two equal reads bracket a window in which NO shaped-geometry seam fired,
    /// so any geometry derived from the shaped runs is still valid.
    pub(super) fn generation(&self) -> u64 {
        self.generation.get()
    }

    /// Drop the variable-row-height geometry caches (and the row count). Called by
    /// `TextPipeline` wherever the shaped geometry changes (reshape, zoom/DPI,
    /// restyle).
    pub(super) fn invalidate(&self) {
        self.total.set(None);
        *self.tops.borrow_mut() = None;
        *self.heights.borrow_mut() = None;
        *self.line_tops.borrow_mut() = None;
        *self.line_baselines.borrow_mut() = None;
        *self.line_last_tops.borrow_mut() = None;
        *self.frame_rows.borrow_mut() = None;
        self.frame_sealed.set(false);
        // Drop the cursor-line VisualRow memo too: the shaped runs just changed, so
        // the cached wrap geometry is stale and must rebuild on the next read.
        self.rows_line.set(None);
        *self.rows.borrow_mut() = None;
        // Advance the generation so run-derived geometry caches (squiggle / nit
        // protos) keyed on it miss and rebuild.
        self.generation.set(self.generation.get().wrapping_add(1));
    }

    /// Populate the row-geometry caches (`tops`/`heights`/`doc_height`) from the
    /// shaped runs if they are stale. One walk of `layout_runs()` (O(visual rows));
    /// the runs arrive in document order with ascending `line_top`, so the tops
    /// vector is sorted. Cheap to call before any geometry read — it returns
    /// immediately once built and is dropped by [`Self::invalidate`]. The metrics
    /// are only consulted by the callers' unshaped fallbacks, not the walk itself.
    fn ensure(&self, buf: &GlyphBuffer, m: &Metrics) {
        if self.tops.borrow().is_some() {
            return;
        }
        let mut tops = Vec::new();
        let mut heights = Vec::new();
        let mut doc_h = 0.0f32;
        // Per logical line: the top (and BASELINE) of its FIRST visual row.
        // `layout_runs()` yields a line's runs consecutively in wrap order, so the
        // FIRST run seen for a given `line_i` is its first visual row.
        // UNSHAPED lines start at the sentinel, not at the document top: a line with
        // no run has no geometry, and answering `0.0` puts it at the top of the page
        // where every viewport cull ACCEPTS it (see `UNSHAPED_LINE_TOP`).
        let mut line_tops: Vec<f32> = vec![UNSHAPED_LINE_TOP; buf.lines.len()];
        let mut line_baselines: Vec<f32> = vec![UNSHAPED_LINE_TOP; buf.lines.len()];
        let mut line_last_tops: Vec<f32> = vec![UNSHAPED_LINE_TOP; buf.lines.len()];
        let mut line_seen: Vec<bool> = vec![false; buf.lines.len()];
        let mut frame_rows = Vec::new();
        for run in buf.layout_runs() {
            tops.push(run.line_top);
            heights.push(run.line_height);
            doc_h = doc_h.max(run.line_top + run.line_height);
            let line_text = buf
                .lines
                .get(run.line_i)
                .map(|line| line.text())
                .unwrap_or("");
            frame_rows.push(FrameVisualRow {
                logical_line: run.line_i,
                row: visual_row_from_run(line_text, &run, m.char_width),
            });
            if let Some(top) = line_last_tops.get_mut(run.line_i) {
                *top = run.line_top; // last run for this line wins: its LAST visual row
            }
            if let Some(seen) = line_seen.get_mut(run.line_i)
                && !*seen
            {
                *seen = true;
                line_tops[run.line_i] = run.line_top;
                line_baselines[run.line_i] = run.line_y;
            }
        }
        self.doc_height.set(doc_h);
        *self.tops.borrow_mut() = Some(tops);
        *self.heights.borrow_mut() = Some(heights);
        *self.line_tops.borrow_mut() = Some(line_tops);
        *self.line_baselines.borrow_mut() = Some(line_baselines);
        *self.line_last_tops.borrow_mut() = Some(line_last_tops);
        *self.frame_rows.borrow_mut() = Some(frame_rows);
    }

    /// Mark the current shaped partition as the frame glyphon just prepared.
    /// This is the only door that makes it reportable.
    pub(super) fn seal_frame(&self, buf: &GlyphBuffer, m: &Metrics) {
        self.ensure(buf, m);
        self.frame_sealed.set(true);
    }

    /// Clone one logical line's rows from the canonical shaped partition for
    /// existing caret/selection consumers. No glyph-x assembly occurs here.
    pub(super) fn rows_for_line(
        &self,
        buf: &GlyphBuffer,
        m: &Metrics,
        line: usize,
    ) -> Option<Vec<VisualRow>> {
        self.ensure(buf, m);
        let rows = self.frame_rows.borrow();
        let rows = rows.as_ref()?;
        let found: Vec<VisualRow> = rows
            .iter()
            .filter(|entry| entry.logical_line == line)
            .map(|entry| entry.row.clone())
            .collect();
        (!found.is_empty()).then_some(found)
    }

    /// Clone a requested line set in one scan of the canonical frame partition.
    /// This preserves the underline/wash cache rebuild's O(doc + requested rows)
    /// behavior while keeping row assembly in one owner.
    pub(super) fn rows_for_lines(
        &self,
        buf: &GlyphBuffer,
        m: &Metrics,
        lines: &std::collections::BTreeSet<usize>,
    ) -> std::collections::HashMap<usize, Vec<VisualRow>> {
        self.ensure(buf, m);
        let mut out = std::collections::HashMap::with_capacity(lines.len());
        let rows = self.frame_rows.borrow();
        let Some(rows) = rows.as_ref() else {
            return out;
        };
        for entry in rows {
            if lines.contains(&entry.logical_line) {
                out.entry(entry.logical_line)
                    .or_insert_with(Vec::new)
                    .push(entry.row.clone());
            }
        }
        out
    }

    /// Borrow the sealed frame partition in place. This never calls
    /// [`Self::ensure`], so a report cannot assemble geometry on demand.
    pub(super) fn with_report_rows<R>(
        &self,
        read: impl FnOnce(&[FrameVisualRow]) -> R,
    ) -> Option<R> {
        if !self.frame_sealed.get() {
            return None;
        }
        let rows = self.frame_rows.borrow();
        let rows = rows.as_deref()?;
        #[cfg(test)]
        REPORT_ROW_BORROWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Some(read(rows))
    }

    /// Buffer-relative top y (px) of logical `line`'s FIRST visual row — the O(1)
    /// cull read for the ornament pass, equal to `visual_rows(line)[0].line_top`
    /// (both come from the same `run.line_top`). [`UNSHAPED_LINE_TOP`] for a line
    /// with no shaped run and for an out-of-range line, so the caller's viewport
    /// cull rejects a row whose position is not yet known instead of drawing it at
    /// the top of the page.
    pub(super) fn line_first_top(&self, buf: &GlyphBuffer, m: &Metrics, line: usize) -> f32 {
        self.ensure(buf, m);
        self.line_tops
            .borrow()
            .as_ref()
            .and_then(|v| v.get(line).copied())
            .unwrap_or(UNSHAPED_LINE_TOP)
    }

    /// Buffer-relative top y (px) of logical `line`'s **LAST** visual row — equal to
    /// [`Self::line_first_top`] for an unwrapped line, and to the top of the final
    /// wrapped row otherwise. [`UNSHAPED_LINE_TOP`] for an out-of-range or unshaped
    /// line, exactly like its first-row sibling, so a caller's viewport cull rejects
    /// a row whose position is not yet known.
    pub(super) fn line_last_top(&self, buf: &GlyphBuffer, m: &Metrics, line: usize) -> f32 {
        self.ensure(buf, m);
        self.line_last_tops
            .borrow()
            .as_ref()
            .and_then(|v| v.get(line).copied())
            .unwrap_or(UNSHAPED_LINE_TOP)
    }

    /// Buffer-relative BASELINE y (px) of logical `line`'s FIRST visual row — the
    /// REAL shaped baseline (`LayoutRun::line_y`), not an approximation from the
    /// metrics. [`UNSHAPED_LINE_TOP`] for an out-of-range or unshaped line (mirrors
    /// [`Self::line_first_top`]'s fallback). This is the fold affordance's one
    /// baseline-alignment geometry source.
    pub(super) fn line_first_baseline(&self, buf: &GlyphBuffer, m: &Metrics, line: usize) -> f32 {
        self.ensure(buf, m);
        self.line_baselines
            .borrow()
            .as_ref()
            .and_then(|v| v.get(line).copied())
            .unwrap_or(UNSHAPED_LINE_TOP)
    }

    /// Buffer-relative top y (px) of visual row `row` (clamped to the last row).
    /// `0.0` for an unshaped/empty buffer, so `doc_top()` resolves to `TEXT_TOP`.
    pub(super) fn top_px(&self, buf: &GlyphBuffer, m: &Metrics, row: usize) -> f32 {
        self.ensure(buf, m);
        let tops = self.tops.borrow();
        match tops.as_ref() {
            Some(v) if !v.is_empty() => v[row.min(v.len() - 1)],
            _ => 0.0,
        }
    }

    /// Height (px) of visual row `row` (clamped to the last row). Falls back to the
    /// uniform line height for an unshaped/empty buffer.
    pub(super) fn height_px(&self, buf: &GlyphBuffer, m: &Metrics, row: usize) -> f32 {
        self.ensure(buf, m);
        let hs = self.heights.borrow();
        match hs.as_ref() {
            Some(v) if !v.is_empty() => v[row.min(v.len() - 1)],
            _ => m.line_height,
        }
    }

    /// Total pixel height of the shaped document (bottom of the last visual row).
    pub(super) fn total_height(&self, buf: &GlyphBuffer, m: &Metrics) -> f32 {
        self.ensure(buf, m);
        self.doc_height.get()
    }

    /// TOTAL number of VISUAL ROWS in the whole document — the COUNT of shaped runs
    /// (one per visual row), read from the row-geometry table. Cached: counting rows
    /// walks every shaped run (O(visual rows)), so an unchanged buffer answers from
    /// the cache. Invalidated whenever the buffer is reshaped (`set_text`) or its
    /// metrics change (zoom in `set_view`), so a cursor move / scroll / selection
    /// change — which never reshape — keep reading the cached count for free. This
    /// is what keeps `app.rs`'s `total_visual_rows()` read in the per-keystroke /
    /// per-frame path cheap. Falls back to the logical line count if nothing is
    /// shaped (degenerate empty buffer).
    pub(super) fn total_visual_rows(&self, buf: &GlyphBuffer, m: &Metrics) -> usize {
        if let Some(n) = self.total.get() {
            return n;
        }
        self.ensure(buf, m);
        let rows = self.tops.borrow().as_ref().map(|v| v.len()).unwrap_or(0);
        let total = if rows == 0 {
            // No shaped runs (empty/degenerate buffer): one row per logical line.
            buf.lines.len().max(1)
        } else {
            rows
        };
        self.total.set(Some(total));
        total
    }

    /// A CLONE of the memoized [`VisualRow`]s for logical `line`, or `None` when the
    /// memo holds a different line (or is empty). Cloning the cached vector is cheap
    /// — a few rows, each a `Vec<f32>` of the line's char boundaries — versus the
    /// full-document `layout_runs()` walk + per-run `assemble_glyph_xs` that
    /// [`super::TextPipeline::visual_rows`] does on a miss.
    pub(super) fn cached_rows(&self, line: usize) -> Option<Vec<VisualRow>> {
        if self.rows_line.get() == Some(line) {
            self.rows.borrow().clone()
        } else {
            None
        }
    }

    /// Read the memoized rows in place, without cloning their per-column `xs`.
    /// Pointer motion uses this after the render/caret path has assembled the row,
    /// so every move can resolve against the drawn geometry without allocating.
    pub(super) fn with_cached_rows<R>(
        &self,
        line: usize,
        read: impl FnOnce(&[VisualRow]) -> R,
    ) -> Option<R> {
        if self.rows_line.get() != Some(line) {
            return None;
        }
        let rows = self.rows.borrow();
        let rows = rows.as_deref()?;
        #[cfg(test)]
        IN_PLACE_ROW_BORROWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Some(read(rows))
    }

    /// Store `rows` as the memo for logical `line` (replacing any prior line). Called
    /// by [`super::TextPipeline::visual_rows`] right after it builds them, so the next
    /// read of the same line hits [`Self::cached_rows`]. Dropped wholesale by
    /// [`Self::invalidate`] at every shaped-geometry seam.
    pub(super) fn store_rows(&self, line: usize, rows: &[VisualRow]) {
        self.rows_line.set(Some(line));
        *self.rows.borrow_mut() = Some(rows.to_vec());
    }

    /// The row which CONTAINS fixed-point document coordinate `target_q`.
    /// An exact boundary belongs to the following row, yielding offset zero.
    pub(super) fn containing_row_q(&self, buf: &GlyphBuffer, m: &Metrics, target_q: i64) -> usize {
        self.ensure(buf, m);
        let tops = self.tops.borrow();
        match tops.as_ref() {
            Some(v) if !v.is_empty() => v
                .partition_point(|top| {
                    ((*top * ScrollPos::SUBPX as f32).round() as i64) <= target_q
                })
                .saturating_sub(1),
            _ => 0,
        }
    }
}
