//! CARET RENDER GEOMETRY — the layout-entangled half of the animated caret.
//!
//! The caret's *spring physics*, *global mode*, and *GPU pipelines* already live
//! in their own crate modules ([`crate::caret`] — `CaretAnim` spring + `CaretMode`
//! global + the block/trail `CaretPipeline`; [`crate::caret_glyph`] — the morph
//! `CaretGlyphPipeline` + `GlyphMask`). What remains here is everything that turns
//! that machinery into *pixels for the current frame*: the per-frame caret/streak
//! geometry, the morph glyph-mask rasterisation + placement, the IME cell rect,
//! the spring TARGET wiring, the deterministic settle/inject poses, and the
//! capture-sidecar reports.
//!
//! These are all inherent methods on [`super::TextPipeline`] — they read its font
//! / layout / metrics / buffer state heavily (the cursor's visual row, the real
//! glyph advance, the shaped baseline), so they CANNOT become `&self`-free free
//! functions the way the span/attrs helpers over in `render.rs` already are
//! (`build_line_attrs`, `add_md_line_spans`, `scaled_base_attrs`, … take explicit
//! params, so they could move to a sibling module unchanged). The row-geometry
//! cache (`ensure_row_geom` and friends), by contrast, is still inherent methods
//! on `TextPipeline` just like the caret geometry here. So this module is purely a
//! physical home for that cohesive cluster, carved out of `render.rs` verbatim.
//! Because a child module sees its ancestor's private items, the methods keep
//! their full access to `TextPipeline`'s private fields and helpers with NO
//! behaviour change — the capture output is byte-identical.
//!
//! The NON-caret neighbour from the original region — the virtual-clock seam
//! `advance()` — deliberately stays in the parent `render` module; only
//! `step_caret()` (which `advance()` OR-folds in) lives here.

use super::*;

/// TARGET-LINE-LOCAL caret glyph record (item 57) — the shaped glyph clusters of
/// ONE logical line (the cursor line), read straight from that line's OWN
/// [`cosmic_text::BufferLine::layout_opt`] rather than by filtering the whole
/// document's `layout_runs()` stream.
///
/// The caret's per-frame glyph lookups (`cursor_glyph_key_at`, `cluster_char_span`
/// and their consumers — the block ink box, the descender drop, the morph masks)
/// used to walk `self.buffer.layout_runs()` from the document TOP, breaking once
/// they passed the cursor line. That walk visits one run per visual row of the
/// whole PREFIX before the caret — so its cost grew with the caret's document
/// POSITION (top: a few runs; tail: every run in the file), re-paid every frame a
/// caret animates. `--bench-caret` witnessed the prefix growth.
///
/// This record fixes it: the clusters come from `bline.layout_opt()` — O(the
/// cursor line's OWN glyphs), independent of how far down the document the caret
/// sits. It is a SINGLE slot (the caret is only ever on one line), rebuilt when
/// the cursor moves to a different line or the shaped geometry changes (a new
/// `RowGeom` generation), NOT a retained document-wide cache. The block / mask /
/// descender / ink-box / cluster-span consumers all share this ONE record.
pub(super) struct CaretLineGlyphs {
    /// The logical line these clusters were shaped for.
    line: usize,
    /// The [`rowgeom::RowGeom`] generation at build time — bumped by every reshape /
    /// zoom / restyle seam, so a stale record (different shaped runs) is rebuilt.
    generation: u64,
    /// `(start_byte, end_byte, CacheKey)` per shaped glyph, in layout (wrap) order —
    /// the exact glyph objects `layout_runs()` would have yielded for this line, so
    /// the key/span lookups are byte-identical to the old whole-doc walk.
    clusters: Vec<(usize, usize, CacheKey)>,
}

/// ITEM 91 — the anchored glyph's FULL RASTER INK BOX, in px relative to the pen
/// origin the caret's own geometry is built around: the caret's ONE description of
/// "where this letter's ink actually is". Every field is a straight read of the
/// swash [`cosmic_text::Placement`] the renderer rasterizes the glyph from, so the
/// caret and the real glyph cannot disagree by construction.
///
/// * `left` / `width` — the HORIZONTAL box, px right of the pen x. Already the
///   settled Block quad's width + x since the kerned-glyph round.
/// * `top` / `height` — the VERTICAL box: `top` is px from the BASELINE UP to the
///   raster top, `height` the raster's full height, so the box spans
///   `[baseline - top, baseline - top + height]` and the depth BELOW the baseline
///   is `height - top` ([`Self::descent`]). These were rasterized-and-discarded
///   before item 91; the caret took its top from the generic line cell, which is
///   why a caret on an x-height letter hung 8–9px of empty accent above the ink.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct InkBox {
    /// Px from the pen x to the raster's LEFT edge (a kerned glyph's left bearing;
    /// can be negative).
    pub left: f32,
    /// Px from the BASELINE UP to the raster's TOP edge (negative for a glyph whose
    /// ink starts below the baseline).
    pub top: f32,
    /// Raster width in px.
    pub width: f32,
    /// Raster height in px.
    pub height: f32,
}

impl InkBox {
    /// Px the ink DIPS BELOW the baseline: `0` for a non-dipping glyph (`a`/`m`/`C`),
    /// positive for a descender (`g`/`y`/`p`/`q`/`j`). Font-correct (measured off the
    /// raster), never a hardcoded letter list.
    pub fn descent(self) -> f32 {
        (self.height - self.top).max(0.0)
    }
}

impl TextPipeline {
    /// Populate [`Self::caret_line_glyphs`] with `line`'s shaped glyph clusters if the
    /// cached record is stale (a different line, or a newer shaped-geometry
    /// generation). Reads ONLY that line's own `layout_opt()` — no whole-document
    /// `layout_runs()` walk — so it is O(the line's glyphs), independent of the
    /// line's position in the document. `&self` via the interior-mutable `RefCell`:
    /// the shaped layout is already built, so collecting the clusters is a pure read.
    fn ensure_caret_line_glyphs(&self, line: usize) {
        let generation = self.row_geom.generation();
        if let Some(rec) = self.caret_line_glyphs.borrow().as_ref()
            && rec.line == line
            && rec.generation == generation
        {
            return;
        }
        let mut clusters: Vec<(usize, usize, CacheKey)> = Vec::new();
        if let Some(bline) = self.buffer.lines.get(line)
            && let Some(layout) = bline.layout_opt()
        {
            for lline in layout.iter() {
                for g in lline.glyphs.iter() {
                    clusters.push((g.start, g.end, g.physical((0.0, 0.0), 1.0).cache_key));
                }
            }
        }
        *self.caret_line_glyphs.borrow_mut() = Some(CaretLineGlyphs {
            line,
            generation,
            clusters,
        });
    }

    /// WITNESS (`--bench-caret`): the number of shaped glyph clusters the
    /// target-line-local caret lookup visits for the CURRENT cursor line — the real
    /// work the fixed path does. Nonzero on a non-blank line (proves the lookup ran),
    /// and a function of the cursor LINE's own content ONLY, so it is IDENTICAL at
    /// the document top / middle / tail (the position-independence the item asserts).
    ///
    /// (The prefix-run witness -- how many runs a whole-doc `layout_runs()` walk
    /// would touch -- lives in `caretbench.rs`, NOT here, so this module stays free
    /// of any `layout_runs()` call: `caret_no_whole_doc_walk_law` bans it.)
    pub(super) fn caret_line_glyph_count(&self) -> usize {
        self.ensure_caret_line_glyphs(self.cursor_line);
        self.caret_line_glyphs
            .borrow()
            .as_ref()
            .map(|r| r.clusters.len())
            .unwrap_or(0)
    }

    /// The char COLUMN the caret's geometry ANCHORS on this frame — the cell the
    /// caret visibly INHABITS. BLOCK and I-BEAM anchor on the cursor column
    /// itself: the cell AFTER the insertion point, the cell the next edit will
    /// affect (unchanged, byte-identical). MORPH anchors ONE char BACK
    /// ([`crate::caret::morph_anchor_col`]): the glyph you just typed / passed,
    /// so the living caret rides the last-produced letter (`abc|` shows the `c`
    /// silhouette). At col 0 / a line start / an empty line there is no previous
    /// glyph on the line, so Morph falls back to the cursor column for GEOMETRY —
    /// the cell whose LEFT EDGE is the insertion point x (and a fresh line after
    /// Enter never anchors back onto the previous line) — but it does NOT light
    /// the glyph ahead of the cursor there: the silhouette masks empty
    /// ([`Self::caret_inhabited_key`]) and the caret degrades to the thin
    /// INSERTION BAR ([`Self::caret_linestart_bar_geometry`]).
    ///
    /// Reads the PER-FRAME latched look (`caret_look`, one global read per
    /// `set_view`), not the live global, so all the geometry derived from it this
    /// frame is self-consistent. The IME rect ([`Self::caret_pixel_rect`])
    /// deliberately does NOT use this: the OS composition cell stays at the
    /// insertion point in every look.
    pub(super) fn caret_anchor_col(&self) -> usize {
        if self.caret_look == CaretMode::Morph {
            crate::caret::morph_anchor_col(self.cursor_col)
        } else {
            self.cursor_col
        }
    }

    /// Pixel y of the TOP of the glyph cell box at char column `col` on the
    /// cursor line (the box that the selection / preedit / IME rect share),
    /// wrap-aware — at a wrap boundary an anchor col on the PREVIOUS visual row
    /// reads that row's top. The caret underline sits at the BOTTOM of this box.
    fn caret_cell_top(&self, col: usize) -> f32 {
        let m = &self.metrics;
        // Affinity-aware: an `Upstream` caret parked at a shared wrap boundary rides
        // the UPPER visual row's top, so its box (and the block/morph anchor built on
        // it) sits on the row the caret visually belongs to, not the lower row.
        let line_top = self.visual_row_top_aff(self.cursor_line, col, self.caret_affinity);
        // Centre the caret box in the cursor's ACTUAL row height, so on a (taller)
        // heading row the caret sits on the heading's optical centre rather than
        // floating high in a base-height cell. The caret anchor is built from this
        // (`caret_cell_top + caret_h/2`), so the block/morph caret recentres too.
        // (All wrapped rows of one logical line share one height, so the cursor
        // row's height is the anchor row's height too.)
        let row_h = self.cursor_row_height();
        line_top + (row_h - m.caret_h) * 0.5
    }

    /// The caret spring ANCHOR target: the pixel position the spring chases. This
    /// is the LEFT edge x of the ANCHOR glyph cell ([`Self::caret_anchor_col`] —
    /// the cursor cell for Block/I-beam, one char back for Morph) and the CENTER y
    /// of that cell's box (so the resting rounded square sits centered ON the
    /// character). Using the real glyph advance + wrap-aware visual row keeps the
    /// anchor correct for full-width CJK and wrapped lines — a Morph anchor just
    /// before a soft-wrap boundary rides the PREVIOUS visual row. The drawn caret
    /// rect is built around this anchor by [`Self::caret_geometry`], which applies
    /// the motion drop + shape stretch on top of it.
    pub fn caret_target_xy(&self) -> (f32, f32) {
        let m = &self.metrics;
        let col = self.caret_anchor_col();
        let (gx, _adv) = self.col_x_and_advance_aff(self.cursor_line, col, self.caret_affinity);
        let x = self.text_left() + gx;
        // Cell-box vertical center: the resting square is centered on the glyph.
        let y = self.caret_cell_top(col) + m.caret_h * 0.5;
        (x, y)
    }

    /// The caret's settled anchor in DOCUMENT space — [`Self::caret_target_xy`]
    /// with the scroll offset removed from the vertical, so the y is a stable
    /// function of the caret's logical row alone (independent of how far the view
    /// is scrolled). The x is already scroll-independent (horizontal). Used by
    /// the live App's LIFETIME STATS caret-travel odometer, so a big logical jump
    /// (Cmd-Down over a long file) registers its real distance even though the
    /// view re-centres the on-screen caret; reuses the SAME `caret_target_xy` +
    /// `doc_top` the renderer already computes rather than a parallel geometry.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn caret_doc_xy(&self) -> (f32, f32) {
        let (x, y) = self.caret_target_xy();
        (x, y - self.doc_top())
    }

    /// Width of the resting caret SQUARE at the caret's ANCHOR cell: the real
    /// advance of the anchored glyph (so a full-width CJK glyph gets a full-width
    /// block), clamped to at least the default Latin cell so a glyphless anchor
    /// (end-of-line / empty line / the collapsed wrap-boundary space) stays
    /// visible. Used by the Morph space-bar; the BLOCK quad uses
    /// [`Self::caret_block_w`], and the IME rect computes its own insertion-point
    /// cell in [`Self::caret_pixel_rect`].
    pub fn caret_target_w(&self) -> f32 {
        let (_x, adv) = self.col_x_and_advance_aff(
            self.cursor_line,
            self.caret_anchor_col(),
            self.caret_affinity,
        );
        adv.max(self.metrics.caret_w)
    }

    /// Width of the resting BLOCK caret quad at the caret's ANCHOR cell (the
    /// cursor cell in Block/I-beam; one char back in Morph's fast-motion streak
    /// deferral — see [`Self::caret_anchor_col`]): the REAL shaped glyph ADVANCE
    /// there, so on a PROPORTIONAL world the block exactly
    /// covers the glyph it sits on — wide on an `m`/`w`, narrow on an `i`/`l` —
    /// instead of the fixed mono cell that read too wide on thin glyphs. The advance
    /// comes from the same `col_x_and_advance` the caret X / Morph silhouette / I-beam
    /// already ride, so the block tracks the exact cell the cursor is on. At a
    /// GLYPHLESS cell (end-of-line / empty line / the collapsed space at a soft-wrap
    /// boundary) `col_x_and_advance` falls back to the default `char_width` cell, so
    /// the block keeps a full visible width there instead of a degenerate sliver.
    ///
    /// On a MONO face every advance equals the cell, so we keep the historical
    /// `.max(caret_w)` floor: the block stays byte-identical to the old fixed cell
    /// (`caret_block_w == caret_target_w`; all three bundled monos share the
    /// `CHAR_WIDTH` 0.6-em pitch, so the floor is a no-op on real glyphs). The floor
    /// — the very thing that made the block too wide on a narrow proportional glyph
    /// — is dropped ONLY on proportional faces. Keyed on the EFFECTIVE shaped face
    /// (`shaped_font`, the `doc_family` seam), NOT `Theme::font`: a serif world
    /// editing a `.rs` shapes the buffer in the world's mono companion, and the
    /// block must follow the grid actually on screen.
    pub fn caret_block_w(&self) -> f32 {
        let (_x, adv) = self.col_x_and_advance_aff(
            self.cursor_line,
            self.caret_anchor_col(),
            self.caret_affinity,
        );
        if crate::caret::font_is_mono(self.shaped_font) {
            adv.max(self.metrics.caret_w)
        } else {
            adv
        }
    }

    /// Resolve the cosmic-text [`CacheKey`] of the glyph under the cursor at
    /// (`line`, `col`), or `None` when there is no rasterizable glyph there
    /// (end-of-line, an empty/glyphless line, or a whitespace glyph whose mask is
    /// empty). The MORPH caret uses this key both to capture the "from" glyph at a
    /// move and to rasterize the "to" glyph for the current cursor.
    ///
    /// Reads the cursor line's TARGET-LINE-LOCAL glyph record ([`CaretLineGlyphs`],
    /// built from that line's own `layout_opt()`) and picks the glyph cluster whose
    /// BYTE range covers the cursor column's byte — the same glyph
    /// `self.buffer.layout_runs()` would have yielded for this line, so the returned
    /// `CacheKey` (font + glyph id + size + subpixel) is byte-identical to the old
    /// whole-document walk, now at O(the cursor line's glyphs) instead of O(the whole
    /// prefix before the caret). `col` is always on the cursor line (every caller).
    pub(super) fn cursor_glyph_key_at(&self, line: usize, col: usize) -> Option<CacheKey> {
        let line_text = self.buffer.lines.get(line)?.text().to_string();
        // Byte offset of the cursor column on this logical line.
        let cur_byte = line_text
            .char_indices()
            .nth(col)
            .map(|(b, _)| b)
            .unwrap_or(line_text.len());
        if cur_byte >= line_text.len() {
            // End of line: no glyph cell to silhouette.
            return None;
        }
        self.ensure_caret_line_glyphs(line);
        let rec = self.caret_line_glyphs.borrow();
        let clusters = &rec.as_ref()?.clusters;
        for &(start, end, key) in clusters {
            if cur_byte >= start && cur_byte < end {
                return Some(key);
            }
        }
        None
    }

    /// The char SPAN of the shaped glyph CLUSTER owning column `col` on `line` —
    /// the number of chars between that glyph's byte-range boundaries: `1` for the
    /// overwhelmingly common case of one glyph per char, `>1` for a LIGATURE
    /// (several chars collapse into a single shaped glyph, e.g. an "fi"/"ffi"
    /// fixture on a font that ligates it). `None` when no shaped run owns the
    /// column (end-of-line / empty line). Read by [`Self::caret_anchor_ink_box`]
    /// to decide whether a column may safely be replaced by its glyph's own ink
    /// box (a 1-char cluster IS that glyph, one-to-one) or must keep the CELL
    /// math's fair linear split (a multi-char cluster's cell already spreads one
    /// glyph's ink fairly across the chars it covers — no single column owns the
    /// whole glyph). Reads the SAME target-line-local glyph record as
    /// [`Self::cursor_glyph_key_at`] (`layout_opt()`, not the whole-doc walk).
    fn cluster_char_span(&self, line: usize, col: usize) -> Option<usize> {
        let line_text = self.buffer.lines.get(line)?.text().to_string();
        let cur_byte = line_text
            .char_indices()
            .nth(col)
            .map(|(b, _)| b)
            .unwrap_or(line_text.len());
        if cur_byte >= line_text.len() {
            return None;
        }
        self.ensure_caret_line_glyphs(line);
        let rec = self.caret_line_glyphs.borrow();
        let clusters: Vec<(usize, usize)> = rec
            .as_ref()?
            .clusters
            .iter()
            .map(|&(s, e, _)| (s, e))
            .collect();
        cluster_span_at(&line_text, &clusters, cur_byte)
    }

    /// THE ONE RASTER READ on the caret path: the anchored glyph's full swash
    /// placement box ([`InkBox`] — left/top/width/height, the whole box, not just
    /// the horizontal half), or `None` when the anchor cell has no rasterizable ink
    /// at all (end-of-line, an empty line, whitespace, an emoji, a zero-size mask).
    /// UNGATED by caret policy: this is the raw measurement every vertical/horizontal
    /// caret rule is derived from — the ink-box override ([`Self::caret_anchor_ink_box`],
    /// which adds the mono / ligature policy gate) and the descender depth
    /// ([`InkBox::descent`], which must keep working on a MONO world where the
    /// override deliberately does not apply) both come off this one box.
    ///
    /// `pub(super)` so the sibling `--bench-caret` witness can time the real
    /// per-frame raster read; NO render path may consume it directly — the cell
    /// caret's vertical/horizontal geometry has exactly one owner
    /// ([`Self::caret_cell_vertical`] / [`Self::caret_anchor_ink_box`]), and
    /// `render::tests::caret_ink_box`'s grep-law fails if `layers.rs` reads a
    /// raster box or a line-cell height of its own.
    pub(super) fn caret_anchor_raster_box(&mut self) -> Option<InkBox> {
        self.raster_box_at(self.cursor_line, self.caret_anchor_col())
    }

    /// [`Self::caret_anchor_raster_box`] widened to an ARBITRARY column on the
    /// cursor line (item 105's transition repair) — the same ungated raw raster
    /// read, just not pinned to the caret's own anchor column. The one other
    /// caller is [`Self::nearest_row_raster_box`], which searches outward from
    /// a glyphless anchor for a column that has real ink.
    fn raster_box_at(&mut self, line: usize, col: usize) -> Option<InkBox> {
        let key = self.cursor_glyph_key_at(line, col)?;
        let Self {
            swash_cache,
            font_system,
            ..
        } = self;
        let img = swash_cache.get_image(font_system, key).as_ref()?;
        if img.placement.width == 0 || img.placement.height == 0 {
            return None;
        }
        Some(InkBox {
            left: img.placement.left as f32,
            top: img.placement.top as f32,
            width: img.placement.width as f32,
            height: img.placement.height as f32,
        })
    }

    /// [`Self::raster_box_at`] searched outward from `col` in BOTH directions
    /// and BLENDED — item 105's THIRD repair round (the second round's
    /// nearest-distance-wins pick bred a same-class defect of its own; see
    /// below). Bounded to the caret's OWN VISUAL ROW ([`Self::visual_rows`]'s
    /// `[start_col, end_col)` span for whichever row `col` sits on, resolved
    /// the SAME way [`Self::caret_row_metrics`] resolves its own row —
    /// [`pick_row_index_aff`]), never the whole logical line, so a giant blank
    /// paragraph costs O(row width), never O(doc).
    ///
    /// The FIRST repair round (12c2fb4 / fdbc0da) tried only `col - 1`, a
    /// single backward hop: it closed the literal reported shape (`aaa`'s
    /// final `a` → EOL) but had no `col - 1` at column 0 (`" A"`), and lost the
    /// borrowed ink one hop into a run of 2+ glyphless columns (`"A  "`) —
    /// see that round's own docstring, superseded below.
    ///
    /// The SECOND repair round (b0ca2cf) searched outward, nearest distance
    /// wins, backward preferred on an exact tie — closing both shapes above.
    /// But "nearest wins" is a HARD PICK: it borrows exactly ONE side's ink,
    /// never the other, so a single glyphless column with a DIFFERENT real
    /// letter on each side (a table's `"| 1"` — pipe one side, digit the
    /// other, tied at distance 1, backward wins by the tiebreak) reads as pure
    /// pipe ink. The BACKWARD seam (pipe -> space) then measures a clean
    /// 0.00px, because both columns share literally the same box — but the
    /// FORWARD seam (space -> digit) inherits the WHOLE pipe-to-digit gap
    /// undiminished, because nothing on the space's side moved toward the
    /// digit at all. Flipping the tiebreak to prefer forward does not fix
    /// this — it only rotates the same whole gap onto the other seam. And on
    /// a run of 4+ glyphless columns flanked by two DIFFERENT letters, the
    /// nearest-wins pick is not just directional but DISCONTINUOUS: the
    /// interior column where "nearest" switches sides is a hard step from one
    /// letter's full ink to the other's, with no columns in between softening
    /// it — the "interior flip" this round's own probe found worse than
    /// b0ca2cf disclosed (measured 7.0px on Bombora, against the 6.5px the
    /// build itself reported).
    ///
    /// This round makes the borrow NON-DIRECTIONAL: when a real box exists on
    /// BOTH sides, the returned box is the LINEAR INTERPOLATION of the two,
    /// weighted by which side `col` sits nearer to
    /// (`t = back_distance / (back_distance + fwd_distance)`, `0` = literally
    /// on top of the backward neighbor's own ink, `1` = literally on top of
    /// the forward one). A single glyphless column tied between two different
    /// letters (`t = 0.5`) now reads as the MIDPOINT of the two — so instead
    /// of one seam absorbing the whole gap and the other absorbing none, EACH
    /// seam absorbs roughly HALF of it, and a run of several glyphless
    /// columns sweeps smoothly from one letter's ink to the other's with no
    /// interior step at all (proven at
    /// `render/tests/caret_transition_item105.rs`'s
    /// `interior_run_between_two_different_letters_has_no_flip_step`). When a
    /// real box exists on only ONE side (column 0, or the tail of a run with
    /// nothing real beyond it), the blend degenerates to that single box
    /// exactly as the second round already returned — the two literal-
    /// adjacency laws (`aaa`->EOL, `run_of_glyphless_columns_stays_bounded`)
    /// stay byte-identical, proven by this round's own non-regression checks.
    fn nearest_row_raster_box(&mut self, line: usize, col: usize) -> Option<InkBox> {
        let rows = self.visual_rows(line);
        let row = rows.get(pick_row_index_aff(&rows, col, self.caret_affinity))?;
        let (start, end) = (row.start_col, row.end_col);
        let max_back = col.saturating_sub(start);
        let max_fwd = end.saturating_sub(col + 1);

        let mut back: Option<(usize, InkBox)> = None;
        for d in 1..=max_back {
            if let Some(b) = self.raster_box_at(line, col - d) {
                back = Some((d, b));
                break;
            }
        }
        let mut fwd: Option<(usize, InkBox)> = None;
        for d in 1..=max_fwd {
            if let Some(b) = self.raster_box_at(line, col + d) {
                fwd = Some((d, b));
                break;
            }
        }

        match (back, fwd) {
            (Some((db, bb)), Some((df, fb))) => {
                let t = db as f32 / (db + df) as f32;
                Some(InkBox {
                    left: bb.left + (fb.left - bb.left) * t,
                    top: bb.top + (fb.top - bb.top) * t,
                    width: bb.width + (fb.width - bb.width) * t,
                    height: bb.height + (fb.height - bb.height) * t,
                })
            }
            (Some((_, bb)), None) => Some(bb),
            (None, Some((_, fb))) => Some(fb),
            (None, None) => None,
        }
    }

    /// The BLOCK caret's INK-ALIGNED box at the caret's ANCHOR cell this frame —
    /// the anchored glyph's own [`InkBox`] in pixels relative to the cell's pen x
    /// and the row's baseline
    /// ([`Self::caret_target_xy`]'s x — the SAME pen MORPH's masks position
    /// against — and [`Self::caret_baseline_y`]) — when the anchor col maps
    /// ONE-TO-ONE onto a single shaped
    /// glyph on a PROPORTIONAL world: the glyph's own swash placement, exactly
    /// the box MORPH already recolours. This is the fix for the reported
    /// kerned-glyph misalignment (`block draws from the naive advance CELL,
    /// while MORPH samples the real glyph — a tightly-kerned pair can shift a
    /// glyph's true ink away from its cell, e.g. the middle `w` of "awl"`):
    /// routing BLOCK through this SAME lookup makes the two looks structurally
    /// unable to disagree on where a glyph's ink actually sits.
    ///
    /// Returns `None` — keep the existing CELL geometry
    /// ([`Self::col_x_and_advance`] via [`Self::caret_block_w`]) — in exactly the
    /// cases a single-glyph box would be WRONG or pointless:
    ///   * a MONO world ([`crate::caret::font_is_mono`]): the whole point of a
    ///     monospace display is a perfectly uniform caret grid; tracking each
    ///     glyph's own ink would make the block wobble glyph-to-glyph on a font
    ///     designed to look identical-width. The existing `.max(caret_w)` floor
    ///     stays the mono contract, byte-identical.
    ///   * a MULTI-CHAR cluster (a ligature — [`Self::cluster_char_span`] > 1):
    ///     the cell math already splits that one glyph's ink fairly across its
    ///     chars; using the raw glyph box here would draw one char's caret as
    ///     wide as the WHOLE ligature.
    ///   * a GLYPHLESS anchor (whitespace / end-of-line / an empty line / a
    ///     zero-size mask): nothing to recolour — the space-bar / default-cell
    ///     fallback already handles it.
    ///
    /// ITEM 91 widened the returned box from the horizontal `(left, width)` pair to
    /// the FULL raster box, so the settled cell caret's TOP and BOTTOM come off the
    /// same measurement its left and right already did. The three `None` cases above
    /// are the ONE funnel for "no ink to size to", so the vertical rule inherits
    /// every existing fallback (mono grid, ligature cell split, space bar,
    /// end-of-line) unchanged — see [`Self::caret_cell_vertical`].
    pub(super) fn caret_anchor_ink_box(&mut self) -> Option<InkBox> {
        if crate::caret::font_is_mono(self.shaped_font) {
            return None;
        }
        let line = self.cursor_line;
        let col = self.caret_anchor_col();
        if self.cluster_char_span(line, col) != Some(1) {
            return None;
        }
        self.caret_anchor_raster_box()
    }

    /// THE ONE OWNER of the CELL-form caret's VERTICAL extent: `(center_y, height)`
    /// in absolute pixels for the caret's RESTING pose. Every caret that draws as a
    /// CELL reads its top and bottom from here and nowhere else — the Block quad,
    /// Morph's fast-travel / ink-caret-world fold to that quad
    /// ([`Self::caret_geometry`]), and the glyphless SPACE BAR
    /// ([`Self::caret_space_bar_geometry`]). The BAR forms deliberately do NOT:
    /// the I-beam and Morph's line-start degrade span the LINE BOX by design
    /// ([`Self::ibeam_bar_dims`] — an insertion bar marks a boundary between glyphs,
    /// so it has no glyph of its own to hug).
    ///
    /// Three arms, gated by the ONE ink funnel ([`Self::caret_anchor_ink_box`])
    /// plus [`crate::caret::font_is_mono`] and the ungated raw raster read
    /// ([`Self::caret_anchor_raster_box`]):
    ///
    /// * **INK BOX (item 91 — a proportional world, one glyph, real ink).** The
    ///   caret spans the anchored glyph's own full raster box grown by
    ///   [`CARET_INK_PAD`] top and bottom. This is the fix for the reported vertical
    ///   misalignment: a FIXED fraction of the row height centred on the line box
    ///   put every letter's top at the same y while the ink's top moves with the
    ///   letter — 8–9px of empty accent above an `a`/`m`/`g`/`y`, ~3px above an
    ///   `l`. Sizing to the box makes the margin a small letter-INDEPENDENT pad
    ///   instead, and covers DESCENDERS through the very same box (`height - top`
    ///   is already below the baseline, so `g`/`y` need no separate rule).
    /// * **PROPORTIONAL FALLBACK, real or synthetic ink (item 105, widened by
    ///   item 105's repair round).** Reached whenever the ink funnel says no to
    ///   a SINGLE-glyph box but the world is NOT mono. THE SAME baseline-relative
    ///   formula as the arm above, fed one of THREE boxes, tried in order:
    ///     1. a LIGATURE cluster (`caret_anchor_ink_box` gates it out only
    ///        because one cluster's ink can't be fairly SPLIT across its chars
    ///        horizontally — the vertical ink is exactly as trustworthy as a
    ///        single glyph's) still has a real [`InkBox`]
    ///        ([`Self::caret_anchor_raster_box`], ungated), so it uses that
    ///        raster box directly — a ligature agrees with a plain glyph beside
    ///        it on the same row exactly like the ink-box arm does;
    ///     2. otherwise, a truly GLYPHLESS anchor (space / end-of-line — no
    ///        raster of its own) BORROWS the caret's own VISUAL ROW's nearby
    ///        real [`InkBox`]es ([`Self::nearest_row_raster_box`] — searched
    ///        OUTWARD in both directions, blended when both sides have real
    ///        ink) — the THIRD repair round's fix, superseding the second
    ///        round's hard "nearest wins" pick: that pick borrowed exactly ONE
    ///        side's ink, so a single glyphless column tied between two
    ///        DIFFERENT letters (a table's `"| 1"`) read as pure ink from
    ///        whichever side the tiebreak favoured — one seam closed to
    ///        0.00px while the OTHER inherited the WHOLE gap between the two
    ///        letters undiminished (backward/forward is a directional
    ///        symptom, not a fix — flipping the tiebreak only rotates which
    ///        seam absorbs the full gap), and a run of 4+ columns flanked by
    ///        two different letters showed a hard INTERIOR FLIP where
    ///        "nearest" switched sides with nothing between to soften it.
    ///        Blending by relative distance (`t = back_dist / (back_dist +
    ///        fwd_dist)`) makes the borrow non-directional: a column tied
    ///        between two different letters reads as their MIDPOINT (each
    ///        seam absorbs roughly half the gap instead of one absorbing all
    ///        of it), and a run sweeps smoothly from one letter's ink to the
    ///        other's with no interior step. When only ONE side has real ink
    ///        (column 0; the tail of a run with nothing real beyond it), the
    ///        blend degenerates to that one box exactly as before — the
    ///        literal-adjacency fixtures (`aaa`->EOL, `" A"`, `"A  "`) stay
    ///        byte-identical;
    ///     3. only when NONE of the above has a real box — the caret's own
    ///        visual row has no rasterizable ink at ANY column (an empty line,
    ///        or a row of pure whitespace with nothing real anywhere on it) —
    ///        does the arm fall to a SYNTHETIC typical-letter box
    ///        ([`Self::caret_synthetic_ink_box`]): the row's own real
    ///        `max_ascent` times a typical-letter/ascent ratio
    ///        (`facepitch::typical_letter_ratio`, read off the shipped font
    ///        file), KEYED ON WHICHEVER FONT ACTUALLY PRODUCED THAT ASCENT
    ///        (`caret_row_metrics`'s third element) — a real shaped row's
    ///        `max_ascent` is a property of `shaped_font` (the face ACTUALLY on
    ///        screen right now, possibly still lagging the live theme mid
    ///        preview), while the metrics-only empty-line approximation is
    ///        theme-independent by construction and reads `doc_family()` for
    ///        the ratio — so the two quantities multiplied together always
    ///        describe the SAME font, never a source-ascent/destination-ratio
    ///        Frankenstein number (found auditing this repair round: mixing
    ///        `shaped_font`'s real row metrics with `doc_family()`'s ratio
    ///        produced a measurable few-px pop on ordinary text during a
    ///        theme-picker scrub that the original fix's one empty-buffer law
    ///        could not see, because an empty buffer never takes the real-row
    ///        branch at all).
    ///      THIS is the item-105 fix: before it, this whole case (real ligature ink
    ///      included) fell to the LINE-CELL arm below — a fixed row-box-centred
    ///      height with NO baseline/ascent reference at all — so a proportional
    ///      world's caret visibly jumped top/bottom/centre the instant it left a
    ///      real glyph for the very next column. Feeding all three references
    ///      through the identical `baseline - top + height/2` / `height + 2*pad`
    ///      formula means they can no longer structurally disagree at that seam —
    ///      only WHICH box (or which font) feeds the shared formula differs, never
    ///      the formula itself.
    /// * **LINE CELL (mono only).** The historical geometry, byte-identical:
    ///   `caret_block_h` row-scaled, centred on the spring anchor, with the
    ///   DESCENDER-AWARE bottom extension ([`CARET_DESCENDER_PAD`]) folded in
    ///   here rather than at the draw site. Item 97's uniform mono grid never
    ///   reads ANY ink box, real or synthetic — every column on a mono row shares
    ///   one `caret.pos.y`/`caret_block_h`, so the cell stays column-independent
    ///   by construction, exactly as before item 105.
    ///
    /// The descender extension used to live in `layers.rs::prepare_caret_block`,
    /// applied to the already motion-blended rect and re-scaled by the settle
    /// factor. Folding it into the REST endpoint here is algebraically identical
    /// at rest (settle 1 — the deterministic capture) and lets `motion_geometry`
    /// blend it out with everything else mid-glide, so the travelling streak has
    /// exactly one thickness rule.
    pub(super) fn caret_cell_vertical(&mut self) -> (f32, f32) {
        let m = self.metrics;
        // Pixel scale (zoom × dpi), the same factor `CARET_DESCENDER_PAD` rides.
        let px = m.caret_h / CARET_H;
        if let Some(ink) = self.caret_anchor_ink_box() {
            let baseline = self.caret_baseline_y();
            // Box spans [baseline - top, baseline - top + height]; pad both ends.
            let cy = baseline - ink.top + ink.height * 0.5;
            let h = ink.height + 2.0 * CARET_INK_PAD * px;
            return (cy, h);
        }
        // Gated on `doc_family()` (the LIVE effective face the ACTIVE theme wants
        // for this buffer kind), deliberately NOT `shaped_font` (the face the
        // document is ACTUALLY shaped in right now) — unlike the ink-box arm
        // above and the raster-box read just below, this decision has no real
        // on-screen glyph to align with (a synthetic box has nothing to match),
        // so it should track whatever the theme-preview's O(1) COLOR retint
        // already committed to, not wait on the separately-deferred font
        // RESHAPE. `sync_theme_colors` (the picker's per-arrow preview step)
        // updates the active theme and every baked colour instantly but leaves
        // `shaped_font` stale until the reshape catches up; before item 105 nothing
        // color-only-visible read font identity at a GLYPHLESS anchor (the old
        // fallback was one constant formula regardless of font), so this split
        // introduced the first font-identity dependency at that seam. Reading
        // `shaped_font` here would leave the caret's OWN geometry stale exactly
        // where the picker's preview law (`render::tests::distinguishability`)
        // proves color retint must fully re-ground the surface.
        if !crate::caret::font_is_mono(self.doc_family()) {
            // PROPORTIONAL FALLBACK (item 105): the SAME baseline-relative formula
            // as the ink-box arm, fed (in order) a real ligature raster box, a
            // borrowed NEIGHBOR raster box, or a synthetic typical-letter box —
            // see the doc above.
            let (baseline, row_ascent, ascent_font) = self.caret_row_metrics();
            let col = self.caret_anchor_col();
            let ink = self
                .caret_anchor_raster_box()
                .or_else(|| self.nearest_row_raster_box(self.cursor_line, col))
                .unwrap_or_else(|| self.caret_synthetic_ink_box(row_ascent, ascent_font));
            let cy = baseline - ink.top + ink.height * 0.5;
            let h = ink.height + 2.0 * CARET_INK_PAD * px;
            return (cy, h);
        }
        // LINE-CELL arm (MONO only): the generic row-scaled cell, plus the
        // descender-aware BOTTOM drop so a dipping glyph stays inside the block.
        let cy = self.caret.pos.y;
        let h = m.caret_block_h * self.cursor_scale();
        let descender = self
            .caret_anchor_raster_box()
            .map(|b| b.descent())
            .unwrap_or(0.0);
        if descender <= 0.0 {
            return (cy, h);
        }
        // Pad a dipping glyph's descender a hair so its antialiased ink edge stays
        // inside the block; non-dippers (handled above) are untouched.
        let desc_bottom = self.caret_baseline_y() + descender + CARET_DESCENDER_PAD * px;
        let extend = (desc_bottom - (cy + h * 0.5)).max(0.0);
        // `h += extend; cy += extend/2` drops the bottom while the top is invariant.
        (cy + extend * 0.5, h + extend)
    }

    /// The FALLBACK arm's SYNTHETIC ink box (item 105) for a truly GLYPHLESS
    /// PROPORTIONAL anchor (space / end-of-line / an empty line — nothing
    /// [`Self::caret_anchor_raster_box`] can measure): a typical lowercase
    /// letter's placement, expressed in the SAME `top`/`height`-above-baseline
    /// convention a real [`InkBox`] uses, so [`Self::caret_cell_vertical`] can
    /// feed it through the identical formula the real ink-box arm reads.
    ///
    /// `top == height` (zero descent) by construction: a typical non-descending
    /// letter's ink sits ON the baseline with nothing below it, exactly like a
    /// real non-dipping glyph's box. There is deliberately no synthetic
    /// descender — a glyphless anchor has no letter to dip, so nothing to extend
    /// for; a REAL dipping ligature already carries its own descent inside its
    /// raster box in the caller above, untouched by this function.
    ///
    /// `row_max_ascent` is the SAME per-row value [`Self::caret_row_metrics`]
    /// pairs with the baseline this box is fed against — already reshaped for a
    /// heading / zoom / DPI row, so this needs no separate font-size lookup —
    /// scaled by `ratio_font`'s OWN measured typical-letter/ascent ratio
    /// ([`facepitch::typical_letter_ratio`] — the MEAN of the face's x-height and
    /// cap-height, not bare x-height: a pure x-height reference reproduces item
    /// 91's bug in miniature against an ASCENDER neighbour, so the mean is the
    /// balance point between the two glyph classes the ink-box arm already
    /// treats as different heights): a real per-font quantity read off the
    /// shipped face file, not a hand-tuned per-world offset.
    ///
    /// `ratio_font` is [`Self::caret_row_metrics`]'s own third element —
    /// WHICHEVER font actually produced `row_max_ascent` — never independently
    /// re-derived here. A real shaped row's `max_ascent` is a property of
    /// `shaped_font` (the face the row is ACTUALLY laid out in this frame); the
    /// metrics-only empty-line approximation is theme-independent and pairs
    /// with `doc_family()` instead (see the caller's doc). Reading anything
    /// else here (e.g. unconditionally `doc_family()`) would multiply one
    /// font's ascent by a DIFFERENT font's ratio whenever the two diverge — the
    /// theme-picker preview lag (`sync_theme_colors` without a reshape yet) is
    /// the one live case that does, and the mixed number it produces pops a few
    /// px on ordinary text mid-scrub even though neither factor alone is wrong.
    fn caret_synthetic_ink_box(&self, row_max_ascent: f32, ratio_font: &str) -> InkBox {
        let ratio = facepitch::typical_letter_ratio(ratio_font);
        let top = (row_max_ascent * ratio).max(1.0);
        InkBox {
            left: 0.0,
            top,
            width: 0.0,
            height: top,
        }
    }

    /// The [`CacheKey`] of the glyph the caret's ANCHOR cell INHABITS right now,
    /// or `None` when there is nothing to inhabit. Two distinct `None` cases:
    ///
    /// * a GLYPHLESS anchor (whitespace / end-of-line / an empty line / emoji) —
    ///   `cursor_glyph_key_at` finds no rasterizable glyph; the morph shows the
    ///   centered space bar there.
    /// * the MORPH LINE-START DEGRADE ([`crate::caret::morph_line_start`]): at
    ///   col 0 the anchor cell holds the char AHEAD of the cursor (the col-0
    ///   geometry fallback), which the caret must NOT light — there is no
    ///   produced glyph to inhabit, so the silhouette empties and the caret
    ///   melts to the thin insertion bar.
    ///
    /// This is the ONE key source for the morph masks — both the per-frame "to"
    /// mask ([`Self::prepare_caret_masks`]) and the `set_view` "from" latch (the
    /// glyph the caret is LEAVING) read it, so a departure from a line start
    /// correctly cross-fades from NOTHING (the bar) straight onto the newly
    /// inhabited glyph, never from the un-inhabited char ahead. Keyed on the
    /// per-frame latched look (`caret_look`), like all anchor-derived state.
    pub(super) fn caret_inhabited_key(&self) -> Option<CacheKey> {
        if self.caret_look == CaretMode::Morph && crate::caret::morph_line_start(self.cursor_col) {
            return None;
        }
        self.cursor_glyph_key_at(self.cursor_line, self.caret_anchor_col())
    }

    /// Ensure `slot`'s cached mask matches `key`, rasterizing only when the key
    /// changed (the key folds glyph id + font + size + subpixel, so zoom / font /
    /// world switches re-rasterize automatically). A `None` key clears the slot.
    ///
    /// `pub(super)` (not private): the caret-style picker's PREVIEW demo
    /// (`render/chrome.rs`'s `emit_preview_caret`) reuses this SAME rasterizer for
    /// its own mask slots — a throwaway `GlyphBuffer` + a separate `CaretGlyphPipeline`
    /// instance, never the document's — rather than duplicating the swash-cache
    /// walk (one owner, per CLAUDE.md's "same behavior ⇒ same code").
    pub(super) fn ensure_mask(
        slot: &mut Option<GlyphMask>,
        swash_cache: &mut SwashCache,
        font_system: &mut FontSystem,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: Option<CacheKey>,
    ) {
        match key {
            None => *slot = None,
            Some(k) => {
                if slot.as_ref().map(|m| m.key) == Some(k) {
                    return; // already cached
                }
                let mask = swash_cache
                    .get_image_uncached(font_system, k)
                    .and_then(|image| {
                        if image.content != SwashContent::Mask {
                            return None;
                        }
                        let w = image.placement.width;
                        let h = image.placement.height;
                        if w == 0 || h == 0 || image.data.is_empty() {
                            return None;
                        }
                        Some(GlyphMask::from_coverage(
                            device,
                            queue,
                            k,
                            image.placement.left,
                            image.placement.top,
                            w,
                            h,
                            &image.data,
                        ))
                    });
                *slot = mask;
            }
        }
    }

    /// The baseline y (absolute, scroll-applied pixels) of the cursor's visual row:
    /// the EXACT pen baseline glyphon draws the real glyph at, so the MORPH
    /// silhouette overlaps it pixel-for-pixel. Each glyph mask's placement box is
    /// positioned relative to this baseline (box top = baseline - placement.top),
    /// mirroring how the swash placement box hangs off the pen origin — which is
    /// the same convention glyphon uses to blit the real glyph. Because the morph
    /// caret now draws OVER the text, exact alignment matters: a few-px error would
    /// show as a doubled/shifted letter rather than a clean recolour.
    ///
    /// The truth source is cosmic-text's `run.line_y` (the baseline offset relative
    /// to the buffer top) for the cursor's wrapped run; absolute baseline =
    /// `doc_top() + run.line_y`. A glyphless / empty line has no run, so it falls
    /// back to the metrics-derived ascent approximation (only ever used by the
    /// space/EOL case, which doesn't paint a glyph silhouette anyway).
    ///
    /// TARGET-LINE-LOCAL (item 57): the owning wrapped row is picked from the cursor
    /// line's OWN memoized [`VisualRow`]s ([`Self::visual_rows`] — a single-slot memo,
    /// O(1) warm, never a fresh whole-doc walk on the caret path), and the baseline
    /// is reconstructed from that row's `line_top` plus the paired
    /// [`cosmic_text::LayoutLine`]'s own centering — exactly cosmic-text's own
    /// `line_y = line_top + (line_height - (max_ascent + max_descent))/2 + max_ascent`
    /// — so the value is byte-identical to reading `run.line_y` off the whole-doc walk.
    pub(super) fn caret_baseline_y(&self) -> f32 {
        self.caret_row_metrics().0
    }

    /// `(baseline_y, row max_ascent, ascent_font)` for the caret's ANCHOR row —
    /// the shared core [`Self::caret_baseline_y`] delegates to (`.0`), widened by
    /// item 105 so the fallback (glyphless) arm of [`Self::caret_cell_vertical`]
    /// can build its SYNTHETIC ink box off the exact same row lookup the real
    /// ink-box arm's baseline comes from. A baseline and "how tall is this row's
    /// ascent" can never disagree about which row they describe, because they
    /// are read in the same pass here.
    ///
    /// `ascent_font` (added in item 105's repair round) is WHICHEVER font
    /// actually produced `max_ascent`: the real-layout branch reads a shaped
    /// [`cosmic_text::LayoutLine`], a property of `shaped_font` (the face
    /// ACTUALLY on screen this frame, which may still be lagging the live
    /// theme mid theme-picker-preview — `sync_theme_colors` retints instantly
    /// but defers the reshape); the empty-line fallback branch below derives its
    /// ascent purely from `self.metrics` (zoom/DPI only, theme-INDEPENDENT —
    /// `Metrics::with_dpi` never reads the active theme), so it pairs with the
    /// LIVE `doc_family()` instead, matching the arm-selection gate one call up.
    /// [`Self::caret_synthetic_ink_box`] must multiply `max_ascent` by ITS OWN
    /// font's ratio, never a different font's — seam found auditing this repair
    /// round: keying the ratio unconditionally on `doc_family()` while the real
    /// branch's ascent stays keyed on `shaped_font` produces a mixed-font number
    /// on ordinary (non-empty) text during a live preview scrub.
    ///
    /// ITEM 105 ALSO FIXED the row lookup itself: the pre-105 version re-derived
    /// row ownership with a HAND-ROLLED predicate (`col >= start_col && col <
    /// end_col`, plus an upstream special case) that — unlike every OTHER
    /// caret/geometry consumer of a column-to-row lookup — never handled the
    /// column sitting AT OR PAST every row's `end_col`: the true END-OF-LINE
    /// column on an UNWRAPPED line. There it fell through to the crude
    /// `font_size * 0.8` ascent GUESS below, while the on-glyph ink-box arm one
    /// column to the left reads the row's REAL `max_ascent` — two different
    /// ascent sources for the SAME physical row, which is exactly the kind of
    /// seam this item exists to close (confirmed empirically: the guess reads
    /// ~30% low against Literata's real row ascent, a residual big enough to
    /// still show at the very transition item 105 targets). [`pick_row_index_aff`]
    /// is the SAME canonical column→row resolver [`Self::visual_row_top_aff`] and
    /// [`Self::col_x_and_advance_aff`] already ride (its second pass — "no strict
    /// container, use the LAST row the column trails" — is exactly the
    /// end-of-line case the hand-rolled loop lacked), so reusing it here makes
    /// the caret's row ownership one decision instead of three independently
    /// almost-identical ones.
    fn caret_row_metrics(&self) -> (f32, f32, &'static str) {
        // Anchor column (the cursor column in Block/I-beam; one back in Morph — at a
        // soft-wrap boundary that is the PREVIOUS visual row).
        let col = self.caret_anchor_col();
        // The cursor line's shaped LayoutLines (one per wrapped visual row, in wrap
        // order — the SAME order + count as `visual_rows`, so `rows[i]` pairs with
        // `layout[i]`), read straight from that line's own layout — no doc walk.
        if let Some(bline) = self.buffer.lines.get(self.cursor_line)
            && let Some(layout) = bline.layout_opt()
            && !layout.is_empty()
        {
            let rows = self.visual_rows(self.cursor_line);
            let n = rows.len().min(layout.len());
            if n > 0 {
                let i = pick_row_index_aff(&rows[..n], col, self.caret_affinity);
                let r = &rows[i];
                let ll = &layout[i];
                let line_height = ll.line_height_opt.unwrap_or(self.metrics.line_height);
                let glyph_height = ll.max_ascent + ll.max_descent;
                let centering = (line_height - glyph_height) / 2.0;
                // `r.line_top` is buffer-relative (== `run.line_top`); this
                // reconstructs `run.line_y` exactly.
                let line_y = r.line_top + centering + ll.max_ascent;
                // This ascent is a property of the face the row is ACTUALLY
                // shaped in right now — `shaped_font`, not `doc_family()` —
                // so any ratio multiplied against it must read the same font.
                return (self.doc_top() + line_y, ll.max_ascent, self.shaped_font);
            }
        }
        // Fallback (a truly EMPTY line — no shaped run at all, so there is no row
        // to look up): approximate the baseline from the row top + an ascent
        // proportion. The morph caret never paints a silhouette here (it falls
        // back to the slim space bar), so this only keeps the value finite. The
        // paired ascent approximation mirrors the SAME `0.8 * font_size` term the
        // baseline itself uses, so the two stay mutually consistent even in this
        // no-run corner. `self.metrics` is zoom/DPI-derived only (theme-
        // independent — see `set_view`'s `Metrics::with_dpi`), so this ascent
        // describes no particular shaped font; pair it with the LIVE
        // `doc_family()`, matching the arm-selection gate above.
        let m = &self.metrics;
        let line_top = self.visual_row_top_aff(self.cursor_line, col, self.caret_affinity);
        let ascent = m.font_size * 0.8;
        (
            line_top + (m.line_height - m.font_size) * 0.5 + ascent,
            ascent,
            self.doc_family(),
        )
    }

    /// Geometry for the MORPH caret this frame: the two glyph placement boxes
    /// (`from`/`to`) positioned at the ANIMATED caret anchor (so they slide along
    /// the spring), plus the cross-fade `morph_t`. Returns the boxes as
    /// `[min_x, min_y, w, h]` in absolute pixels. The masks themselves are cached
    /// in `caret_mask_from`/`caret_mask_to`. There is no soft halo; the silhouette
    /// is the glyph's own crisp coverage, HARD-dilated ~`CARET_MORPH_DILATE_PX` in
    /// the shader so the caret reads a touch fatter than the letter but stays solid.
    ///
    /// `morph_t` is driven by the spring's settle factor: 0 mid-glide (show the
    /// FROM glyph), rising to 1 as the caret decelerates onto the destination (show
    /// the TO glyph). At rest there is no `from`, so it pins to 1.
    pub(super) fn caret_glyph_geometry(&self) -> ([f32; 4], [f32; 4], f32) {
        // Animated caret left-edge x (the spring chases the cell's left edge x).
        let pen_x = self.caret.pos.x;
        let baseline_y = self.caret_baseline_y();

        // Position a placement box at the animated pen origin: box top-left =
        // (pen_x + placement.left, baseline_y - placement.top). This mirrors how
        // glyphon hangs the real glyph off the pen, so the silhouette overlaps it.
        let box_of = |mask: &Option<GlyphMask>| -> [f32; 4] {
            match mask {
                Some(mk) => [
                    pen_x + mk.left as f32,
                    baseline_y - mk.top as f32,
                    mk.width as f32,
                    mk.height as f32,
                ],
                None => [0.0, 0.0, 0.0, 0.0],
            }
        };
        let from_box = box_of(&self.caret_mask_from);
        let to_box = box_of(&self.caret_mask_to);

        // Cross-fade: the settle factor rises 0->1 as the caret arrives, so the new
        // glyph fades in as the old one fades out. With no FROM glyph there is
        // nothing to fade from, so show the TO glyph fully.
        let morph_t = if self.caret_mask_from.is_some() {
            self.caret.settle_factor()
        } else {
            1.0
        };
        (from_box, to_box, morph_t)
    }

    /// Refresh the cached MORPH masks for this frame: rasterize the glyph the
    /// caret now INHABITS (the "to" mask, at the ANCHOR column — one char back of
    /// the insertion point, so `abc|` rasterizes the `c`) and the glyph the caret
    /// is leaving (the "from" mask, latched at the OLD anchor in `set_view`),
    /// re-rasterizing each only when its `CacheKey` changed. Returns `true` when
    /// there IS a rasterizable INHABITED glyph (so morph mode can draw); `false`
    /// when there is nothing to inhabit — a glyphless anchor (whitespace / an
    /// empty line / emoji) or the LINE-START degrade (col 0, where the anchor
    /// cell's glyph sits AHEAD of the cursor; see [`Self::caret_inhabited_key`])
    /// — signalling the caller to fall back to the slim bar / block caret this
    /// frame.
    pub(super) fn prepare_caret_masks(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> bool {
        let to_key = self.caret_inhabited_key();
        // The "from" glyph fades out only while a glide is settling; once at rest
        // (or with no captured from-key) drop it so the resting caret is a clean
        // single silhouette.
        let from_key = if self.caret.is_animating() {
            self.caret_from_key
        } else {
            None
        };
        // Split the borrows: ensure_mask needs the swash cache + font system by
        // &mut alongside each slot, all distinct fields of self. Scoped so the
        // partial borrows release before the final whole-field read below.
        {
            let Self {
                caret_mask_to,
                caret_mask_from,
                swash_cache,
                font_system,
                ..
            } = self;
            Self::ensure_mask(
                caret_mask_to,
                swash_cache,
                font_system,
                device,
                queue,
                to_key,
            );
            Self::ensure_mask(
                caret_mask_from,
                swash_cache,
                font_system,
                device,
                queue,
                from_key,
            );
        }
        self.caret_mask_to.is_some()
    }

    /// The drawn caret rectangle `(center_x, center_y, w, h, corner)` for THIS
    /// frame. The caret morphs between TWO states by the spring's settle factor
    /// `s` (1 = at rest, 0 = fully in motion); `motion = 1 - s` drives the move.
    ///
    /// - AT REST (s≈1): a "roundish square" centered on the glyph cell — width =
    ///   full glyph advance, height = `caret_block_h`, large corner radius; center
    ///   y = the spring anchor (cell-box center).
    /// - IN MOTION (s→0): the square stretches into a thin streak along the TRUE
    ///   travel vector (horizontal / vertical / diagonal alike, no per-axis branch),
    ///   anchored at the TEXT optical centre — the line-box centre `pos.y` dropped by
    ///   `caret_trail_drop` to the x-height middle (so the trail runs THROUGH the
    ///   letters, not slightly above them). There is no baseline drop: a horizontal
    ///   move runs a centred sweep through the text centre rather than dropping to an
    ///   underline. The streak TRAILS the leading edge (the
    ///   leading edge tracks the animated position; the body extends BACK toward
    ///   where the caret came from), its length growing with speed.
    ///
    /// The shape stretch and the corner-radius morph are keyed off the same `s`, so
    /// the caret re-forms as it decelerates onto the destination glyph. The
    /// centre-to-centre trail (via `motion_geometry`) is shared by Block, Morph's
    /// fast-motion deferral, and the I-beam.
    ///
    /// AT REST, when the anchor cell maps onto a single shaped glyph on a
    /// proportional world, the resting square is pulled onto that glyph's own full
    /// ink box ([`Self::caret_anchor_ink_box`]) rather than the naive advance cell —
    /// its width AND x horizontally (the kerned-glyph misalignment fix), and its
    /// height AND y vertically through the ONE vertical owner
    /// ([`Self::caret_cell_vertical`], which also carries the descender and the
    /// mono / ligature / glyphless line-cell fallback — item 91). BOTH ink
    /// corrections are scaled by the settle factor `s`, so they apply only to the
    /// settled quad; a travelling streak still leads from the plain pen x at the
    /// plain line-box centre, unaffected. `&mut self` because the glyph lookup rides
    /// the swash raster cache (the same cost the descender read already paid every
    /// Block frame).
    pub(super) fn caret_geometry(&mut self) -> (f32, f32, f32, f32, f32, f32, f32) {
        let m = self.metrics;
        let s = self.caret.settle_factor();

        // --- Shape endpoints --------------------------------------------------
        let block_w = self.caret_block_w(); // real glyph advance (narrow i, wide m)
        let streak_thin = m.caret_streak_h; // the streak's thin cross-dimension
        // Corner radius: small bar radius in motion, large soft radius at rest.
        let corner = STREAK_RADIUS * m.zoom + (CORNER_RADIUS * m.zoom - STREAK_RADIUS * m.zoom) * s;

        // --- ONE rule for every direction (no if-vertical / if-horizontal) -----
        // The trail is a DIRECT line along the TRUE travel vector (diagonal too),
        // not mirrored onto an axis. Length scales with the (euclidean) speed,
        // floored by this frame's advance so a fast glide bridges with no gaps; the
        // unified `motion_geometry` orients it and trails it behind the leading edge.
        let speed =
            (self.caret.vel.x * self.caret.vel.x + self.caret.vel.y * self.caret.vel.y).sqrt();
        // While HOLDING (continuous/held motion) the length is a STEADY constant
        // (`caret_held_len`) so the trail is a smooth, near-constant streak instead
        // of breathing once per auto-repeat. Non-held is the old speed-derived
        // length floored by the per-frame bridge.
        let streak_len = self.caret.streak_length(
            m.streak_len_for_speed(speed),
            m.caret_streak_max_len,
            m.caret_held_len,
        );
        // Ink-aligned override of the HORIZONTAL rest endpoints (see the doc above):
        // `None` leaves `block_w` and the shift untouched, so mono / ligature /
        // glyphless anchors are byte-identical to before.
        let (block_w, ink_shift) = match self.caret_anchor_ink_box() {
            Some(ink) => (ink.width, ink.left * s),
            None => (block_w, 0.0),
        };
        // VERTICAL rest endpoints from the ONE owner (item 91): the anchored glyph's
        // padded ink box on a proportional world, else the row-scaled line cell with
        // its descender-aware bottom. `motion_geometry` blends the height toward the
        // thin streak by `s`, and the centre correction is itself scaled by `s`, so a
        // travelling caret is unchanged and the settled quad is exact.
        let (cell_cy, block_h) = self.caret_cell_vertical();
        let ink_rise = (cell_cy - self.caret.pos.y) * s;
        let (center, half_along, half_across, axis) = self.caret.motion_geometry(
            block_w,
            block_h,
            streak_thin,
            streak_len,
            m.caret_streak_gap,
            m.caret_trail_drop,
        );
        let center = Sample {
            x: center.x + ink_shift,
            y: center.y + ink_rise,
        };
        (
            center.x,
            center.y,
            half_along * 2.0,
            half_across * 2.0,
            corner,
            axis.0,
            axis.1,
        )
    }

    /// Scale a caret rect's `(w, h, corner)` by the cosmetic SQUASH-POP factor for
    /// THIS frame. Applied at the draw site (after the geometry is computed) about the
    /// rect's UNCHANGED centre, so the caret squashes and springs back IN PLACE — the
    /// centre (hence the on-screen position) is never touched. At rest the factor is
    /// 1.0, so this is an identity (and the deterministic capture, which renders the
    /// settled state, is byte-unchanged). Shared by the block / space-bar / I-beam
    /// draw paths so the pop reads consistently across the looks.
    pub(super) fn pop_scaled(&self, w: f32, h: f32, corner: f32) -> (f32, f32, f32) {
        let s = self.caret.pop_scale();
        (w * s, h * s, corner * s)
    }

    /// The SLIM accent-bar geometry `(center_x, center_y, w, h, corner)` for the
    /// MORPH caret on a GLYPHLESS ANCHOR cell PAST col 0 (the space you just
    /// typed, incl. the collapsed wrap-boundary space; an emoji cell — a LINE
    /// START / empty line instead degrades to the insertion bar, see
    /// [`Self::caret_linestart_bar_geometry`]), where
    /// there is no letterform to recolour: a THIN VERSION of the fat resting caret
    /// — same rounded style and same `caret_block_h` height — just narrowed to
    /// `CARET_SPACE_BAR_W`, and CENTERED in the cell.
    ///
    /// The key fix is the x position. The resting block (`caret_geometry`) centers
    /// on the cell using the REAL advance (`caret_target_w`): `cx = pos.x +
    /// advance*0.5`. The old space bar instead pinned its LEFT edge at `pos.x`
    /// (`cx = pos.x + w*0.5`), which dropped the thin bar against the cell's left
    /// edge — at the boundary BEFORE the space, not inside it — because it ignored
    /// the space's advance entirely. Here we center the thin bar on the same cell
    /// midpoint the block uses (`pos.x + advance*0.5`), so it sits in the middle of
    /// the space gap exactly where the block would. It rides the spring anchor
    /// (`pos`) so it slides with the caret. Drawn through the BLOCK pipeline (a
    /// solid accent rounded rect), which is exactly the slim-bar look we want.
    pub(super) fn caret_space_bar_geometry(&mut self) -> (f32, f32, f32, f32, f32) {
        let zoom = self.metrics.zoom;
        let w = CARET_SPACE_BAR_W * zoom;
        // VERTICALLY the same cell every other CELL-form caret takes — through the
        // ONE owner ([`Self::caret_cell_vertical`]), never a private copy of the
        // cell height. A glyphless anchor is exactly the owner's line-cell arm (the
        // ink funnel returns `None` with no ink to size to), so the bar keeps its
        // historical row-scaled `caret_block_h` on `pos.y` by construction — this is
        // the "glyphless fallback survives item 91" seam, not a special case.
        let (cy, h) = self.caret_cell_vertical();
        // CENTER the thin bar on the cell using the real advance, mirroring the
        // resting block's `pos.x + advance*0.5`. This lands it in the middle of the
        // space gap (not pinned to the left edge as before).
        let advance = self.caret_target_w();
        let cx = self.caret.pos.x + advance * 0.5;
        // Same generous resting corner radius as the fat caret (so it reads as a
        // narrow version of the same rounded caret), clamped so a thin bar can't
        // over-round into a lozenge.
        let corner = (CORNER_RADIUS * zoom).min(w * 0.5);
        (cx, cy, w, h, corner)
    }

    /// The I-beam bar's REST dimensions `(thin, tall)` in pixels: `IBEAM_W`
    /// zoom-scaled across, the full glyph cell box (`caret_h`) row-scaled tall —
    /// so on a (taller) heading row the bar spans the heading's glyphs, not a
    /// body-height sliver (1.0 on body text). The ONE owner of the I-beam bar's
    /// constants, read by both [`Self::caret_ibeam_geometry`] (its rest
    /// endpoints) and the MORPH line-start degrade
    /// ([`Self::caret_linestart_bar_geometry`]) — same behavior, same code: the
    /// melt-to-bar IS the I-beam's bar, not a lookalike.
    fn ibeam_bar_dims(&self) -> (f32, f32) {
        let m = &self.metrics;
        (IBEAM_W * m.zoom, m.caret_h * self.cursor_scale())
    }

    /// Whether the caret is drawing as the THIN INSERTION BAR this frame — the
    /// real I-BEAM look, or MORPH's LINE-START degrade ([`crate::caret::morph_line_start`]
    /// — col 0, a fresh line after Enter, or an empty line), which melts onto the
    /// EXACT SAME bar geometry the I-beam draws ([`Self::ibeam_bar_dims`],
    /// [`Self::caret_linestart_bar_geometry`]). Block, and Morph settled on a
    /// real glyph / a glyphless space bar, are the CELL form instead. THE ONE
    /// owner of "is the caret's current form a bar" — read by the cosmetic |
    /// trail's horizontal anchor ([`Self::caret_trail_geometry`]) so it can
    /// never drift back onto the cell centre for a bar-form caret (previously
    /// it only special-cased literal I-beam mode, so a Morph caret melted to
    /// the line-start bar still anchored its trail on the cell midpoint).
    ///
    /// Reads the PER-FRAME latched look (`caret_look`), so a live text-selection
    /// DRAG — which overrides `caret_look` to the I-beam bar form
    /// ([`crate::render::ViewState::selecting_drag`]) — reports bar form here too.
    pub(super) fn caret_is_bar_form(&self) -> bool {
        match self.caret_look {
            CaretMode::Ibeam => true,
            CaretMode::Morph => crate::caret::morph_line_start(self.cursor_col),
            CaretMode::Block => false,
        }
    }

    /// The thin INSERTION-BAR geometry `(center_x, center_y, w, h, corner)` for
    /// the MORPH caret at a LINE START ([`crate::caret::morph_line_start`] —
    /// col 0, incl. a fresh line after Enter and an empty line), where there is
    /// no produced glyph for the silhouette to inhabit and lighting the char
    /// AHEAD of the cursor would misplace the caret: the morph DEGRADES to the
    /// I-beam look's resting bar — [`Self::ibeam_bar_dims`]' thin/tall bar
    /// pinned at the INSERTION POINT x (`pos.x`, the col-0 cell's left edge,
    /// exactly `caret_ibeam_geometry`'s rest pose: `cx = pos.x + thin/2`,
    /// `cy = pos.y`, corner = half the thin dimension). It rides the SAME spring
    /// anchor, so C-a melts the glyph silhouette into this bar on the glide and
    /// typing one char snaps it back onto the typed glyph — the bar stays the
    /// one living amber caret, just thin. Drawn through the BLOCK pipeline (a
    /// solid accent rounded rect), like the space bar next door.
    pub(super) fn caret_linestart_bar_geometry(&self) -> (f32, f32, f32, f32, f32) {
        let (thin, tall) = self.ibeam_bar_dims();
        let cx = self.caret.pos.x + thin * 0.5;
        let cy = self.caret.pos.y;
        let corner = 0.5 * thin.min(tall);
        (cx, cy, thin, tall, corner)
    }

    /// Geometry `(center_x, center_y, w, h, corner)` for the PROTOTYPE I-beam caret:
    /// a thin vertical bar pinned at the INSERTION POINT (the cursor glyph's left
    /// edge / pen origin `pos.x`), spanning the glyph cell box. AT REST it is a
    /// STEADY thin, tall bar (no breathing — fully static when idle). Reuses the
    /// spring's settle factor + velocity + the streak machinery for VELOCITY
    /// SQUASH/STRETCH (the elongating comet — the I-beam's speed cue, retained):
    ///   * HORIZONTAL motion: stretches into a horizontal comet/lozenge — width
    ///     grows with horizontal speed, height collapses toward the bar's thin
    ///     dimension — trailing back opposite the travel.
    ///   * VERTICAL motion: stretches into a tall lozenge — height grows with
    ///     vertical speed — trailing back along the jump.
    ///    CENTRE-anchored (the comet body trails through the caret's vertical centre,
    ///    like Block/Morph) and the origin-side tail is inset by the shared streak GAP
    ///    so it stops short of where the move started. The underdamped spring supplies
    ///    the overshoot/wobble on landing for free; the edit flinches (typing impact /
    ///    deletion squash / blocked recoil) ride the same spring.
    pub(super) fn caret_ibeam_geometry(&self) -> (f32, f32, f32, f32, f32) {
        let m = &self.metrics;
        let s = self.caret.settle_factor();
        let motion = 1.0 - s;

        // Rest endpoints: a steady thin, tall bar (no breathe swell), row-scaled
        // (see `ibeam_bar_dims` — shared with the morph line-start degrade so the
        // two bars are the same bar by construction).
        let (thin, tall) = self.ibeam_bar_dims();
        // Shared origin GAP: the elongated comet's tail stops ~1.5 chars short of the
        // move's start, consistent with the Block/Morph trail's tail inset. While
        // HOLDING (continuous/held motion) the gap is demoted to a cosmetic trim and
        // the comet is floored by the real travel span + a held floor, so a held
        // drag elongates stably instead of vanishing/strobing — matching Block/Morph.
        let holding = self.caret.is_holding();
        let gap = if holding {
            m.caret_streak_gap * crate::caret::HELD_GAP_FRAC
        } else {
            m.caret_streak_gap
        };
        let held_len = m.caret_held_len;
        // While HELD, pin the squash/stretch blend to full motion so the steady
        // held length below isn't re-compressed by the oscillating settle factor —
        // a constant comet, not a per-repeat pulse (matching Block/Morph).
        let motion = if holding { 1.0 } else { motion };

        let (vx, vy) = (self.caret.vel.x, self.caret.vel.y);
        let dxt = self.caret.target.x - self.caret.pos.x;
        let dyt = self.caret.target.y - self.caret.pos.y;

        if self.caret.is_vertical_move() {
            // VERTICAL travel: a tall lozenge. Length grows with vertical speed,
            // floored by this frame's vertical advance so a fast line jump bridges;
            // the origin tail is inset by the shared gap.
            let mut raw = m
                .streak_len_for_speed(vy.abs())
                .max(self.caret.frame_dy().abs());
            if holding {
                // Steady, constant comet length while held (no per-repeat pulse).
                raw = held_len.min(m.caret_streak_max_len);
            }
            let streak_len = (raw - gap).max(tall);
            let w = thin;
            let h = tall + (streak_len - tall) * motion;
            let cx = self.caret.pos.x + w * 0.5;
            // Trail along Y: leading edge at the cell-centre anchor, body extends
            // BACK opposite the direction of travel.
            let dir = if vy.abs() > 1.0 {
                vy.signum()
            } else if dyt.abs() > f32::EPSILON {
                dyt.signum()
            } else {
                1.0
            };
            // Drop the trail anchor to the TEXT optical centre (scaled by motion, so
            // the resting bar is unchanged), consistent with the Block/Morph trail.
            let cy =
                self.caret.pos.y + m.caret_trail_drop * motion - dir * ((h - tall) * 0.5) * motion;
            let corner = 0.5 * w.min(h);
            return (cx, cy, w, h, corner);
        }

        // HORIZONTAL travel (and rest): a horizontal comet. Width grows with speed
        // (floored by this frame's horizontal advance, less the shared origin gap);
        // height collapses from the tall bar toward the thin dimension so it reads as
        // a lozenge, not a block.
        let mut raw = m
            .streak_len_for_speed(vx.abs())
            .max(self.caret.frame_dx().abs());
        if holding {
            // Steady, constant comet length while held (no per-repeat pulse).
            raw = held_len.min(m.caret_streak_max_len);
        }
        let streak_len = (raw - gap).max(thin);
        let w = thin + (streak_len - thin) * motion;
        let h = tall + (thin - tall) * motion;
        // Leading edge tracks the insertion point; the body trails BACK.
        let lead = self.caret.pos.x + thin * 0.5;
        let dir = if vx.abs() > 1.0 {
            vx.signum()
        } else if dxt.abs() > f32::EPSILON {
            dxt.signum()
        } else {
            1.0
        };
        let cx = lead - dir * (w * 0.5) * motion;
        // Drop the trail anchor to the TEXT optical centre (scaled by motion, so the
        // resting bar is unchanged), consistent with the Block/Morph trail.
        let cy = self.caret.pos.y + m.caret_trail_drop * motion;
        let corner = 0.5 * w.min(h);
        (cx, cy, w, h, corner)
    }

    /// The caret's pixel rectangle `(x, y, w, h)` of the glyph CELL at its resting
    /// target (the END of any active preedit). Handed to winit's
    /// `set_ime_cursor_area` so the OS candidate window floats just below/beside
    /// the composition caret. This is the full cell box (top-left + cell height),
    /// not the thin underline, so the IME candidate window is placed sensibly.
    ///
    /// Deliberately pinned at the INSERTION POINT (the cursor column) in EVERY
    /// caret look — the OS composition cell marks where text will land, so it must
    /// NOT follow Morph's one-back visual anchor ([`Self::caret_anchor_col`]).
    pub fn caret_pixel_rect(&self) -> (f32, f32, f32, f32) {
        // Affinity-aware so the OS composition cell sits at the caret's REAL screen
        // position when it is parked at a shared wrap boundary (matches `caret_cell_top`,
        // which is affinity-aware too); `Downstream` for any ordinary caret.
        let (gx, adv) =
            self.col_x_and_advance_aff(self.cursor_line, self.cursor_col, self.caret_affinity);
        let x = self.text_left() + gx;
        let y = self.caret_cell_top(self.cursor_col);
        (x, y, adv.max(self.metrics.caret_w), self.metrics.caret_h)
    }

    /// Push the current cursor position into the spring as its target. The first
    /// call snaps; later calls (after a cursor move) start a glide.
    pub fn set_caret_target(&mut self, is_edit: bool, held: bool) {
        // Keep the spring's glyph + line yardsticks in sync with the current zoom
        // so the distance-aware damping judges moves in glyphs and the row-crossing
        // (vertical) test uses the real line height.
        self.caret.set_glyph_advance(self.metrics.char_width);
        self.caret.set_line_height(self.metrics.line_height);
        // Edits always slide as a plain block; navigation streaks only on jumps.
        self.caret.set_edit_move(is_edit);
        // HELD / auto-repeat navigation builds a continuous lagging trail (the
        // spring stays springy and the streak spans the real travel).
        self.caret.set_held(held);
        let (x, y) = self.caret_target_xy();
        if is_edit {
            // EDIT MOVES SNAP — all of them, in every caret look. The text an edit
            // produced (a typed char, a backspace, Enter's reflow, a paste/yank)
            // arrives instantly, so the caret must too: `pos == target`, velocity
            // zeroed, zero translation frames. Attention is already AT the
            // insertion point while typing — a glide's job is carrying the eye
            // across DISTANCE, which an edit does not have — so the old same-line
            // typing glide read as a second, distracting animation under every
            // keystroke (in Morph it visibly slid the silhouette over from the
            // previous cell on top of the glyph swap). The aliveness stays with
            // the juice built for edits: the typing-impact back-kick / deletion
            // squash / gulp (applied AFTER this snap, riding the same spring) and
            // Morph's mask swap. NAVIGATION below keeps the full glide + streak —
            // that is where distance lives. One seam, all three looks.
            self.caret.jump_to(x, y);
        } else {
            // NAVIGATION goes through the ZIP DISTANCE GATE: a SMALL / incremental
            // move (single char incl. held L/R, single line incl. held U/D) SNAPS
            // instantly with no glide and no trail (the plain snappy cursor), while a
            // BIG jump (a long C-a/C-e, M-</M->, a page, a search hop) keeps the
            // spring glide + trailing "----" streak. Gated on the actual distance
            // moved, not the key — so a C-e a few chars from the end snaps, a C-e
            // across a long line zips. See [`crate::caret::CaretAnim::nav_to`].
            self.caret.nav_to(x, y);
        }
    }

    /// Advance the caret spring by `dt` seconds and report whether the caret is
    /// still animating (so the windowed app knows to keep redrawing). The cosmetic
    /// SQUASH-POP is ticked on the SAME clock and OR-folded in: a small move snaps
    /// the position instantly (the spring never animates) yet still plays its pop, so
    /// the loop must stay hot while the pop runs, then idle. The pop is a draw-time
    /// scale only — ticking it touches no position state.
    pub fn step_caret(&mut self, dt: f32) -> bool {
        // ACCESSIBILITY TIER 1 — REDUCE MOTION: settle the spring glide, the
        // squash-pop flinches, and the trailing streak INSTANTLY to their exact
        // final state (same position, same rest scale, no streak) instead of
        // easing over `dt`. `snap_to_target` is the same primitive the
        // deterministic `--screenshot` path already uses to render a settled
        // caret — reduce-motion just calls it every step instead of once.
        // Motion-off is a pure time compression: no different final position,
        // no skipped flinch, zero frames of easing. Never reached from a
        // headless capture path (see `motion.rs`'s determinism note), so this
        // branch can only ever be live.
        if crate::motion::reduced() {
            self.caret.snap_to_target();
            return false;
        }
        self.caret.step(dt);
        let popping = self.caret.step_pop(dt);
        // The cosmetic | trail fades on the same live clock; a small move snaps the
        // position instantly yet the trail still fades, so keep the loop hot while it
        // does, then idle. Decoupled from position (ticking it touches no spring state).
        let trailing = self.caret.step_trail(dt);
        self.caret.is_animating() | popping | trailing
    }

    /// Read-only snapshot of the caret spring for the timeline-capture sidecar:
    /// the animated `pos`, the true `target`, the [0,1] `settle_factor`, and
    /// whether the spring is still animating. Lets a timeline frame record the
    /// caret's trajectory (0 -> mid -> settled) machine-readably per step.
    pub fn caret_snapshot(&self) -> ((f32, f32), (f32, f32), f32, bool) {
        (
            (self.caret.pos.x, self.caret.pos.y),
            (self.caret.target.x, self.caret.target.y),
            self.caret.settle_factor(),
            self.caret.is_animating(),
        )
    }

    /// Read-only report of the cosmetic SQUASH-POP for the timeline-capture sidecar:
    /// `(scale, drawn_w, drawn_h)`. `scale` is the pop factor this frame (1.0 settled,
    /// dipping to `CARET_POP_SCALE` right after a move); `drawn_w`/`drawn_h` are the
    /// caret BLOCK rect's dimensions AS DRAWN — the morph geometry scaled by the pop —
    /// so a timeline run can assert, machine-readably, that the block starts squashed
    /// (<1) and eases back to full size while the position stays pinned to target. The
    /// `--screenshot` path renders the settled state (scale 1.0), so a plain capture
    /// reports a full-size block.
    pub fn caret_pop_report(&mut self) -> (f32, f32, f32) {
        let s = self.caret.pop_scale();
        let (_cx, _cy, w, h, _c, _ax, _ay) = self.caret_geometry();
        (s, w * s, h * s)
    }

    /// Read-only report of the caret's drawn TRAIL geometry for the held-capture
    /// sidecar: `(holding, length, tail, head)`. Wraps the SAME private
    /// `caret_geometry()` the Block/Morph renderer draws from — `length` is the
    /// on-screen streak length along the travel axis (`half_along * 2`) and the
    /// endpoints are `center ± axis * (length/2)` — plus the latched `holding`
    /// flag. Lets a HELD run assert, per step, that the trail is present (length
    /// past the streak gap) and never collapses to zero, straight from the JSON.
    pub fn caret_trail_report(&mut self) -> (bool, f32, (f32, f32), (f32, f32)) {
        let (cx, cy, w, _h, _corner, ax, ay) = self.caret_geometry();
        let half = w * 0.5;
        let tail = (cx - ax * half, cy - ay * half);
        let head = (cx + ax * half, cy + ay * half);
        (self.caret.is_holding(), w, tail, head)
    }

    /// The COSMETIC | TRAIL quad for THIS frame, or `None` when no streak is active:
    /// `(center_x, center_y, w, h, corner, ax, ay, alpha)`. Wraps the spring's pure
    /// [`crate::caret::CaretAnim::trail_geometry`] with the zoomed streak thickness /
    /// gap / text-centre drop, the small motion corner radius, and the spring's fading
    /// `trail_alpha`. Decoupled from position — it spans the latched OLD→NEW caret
    /// points, not `pos`/`target`. Shared by `prepare` (to draw it) and
    /// `caret_cosmetic_report` (to report it), so the JSON matches the drawn quad.
    pub(super) fn caret_trail_geometry(&self) -> Option<(f32, f32, f32, f32, f32, f32, f32, f32)> {
        if !self.caret.trail_active() {
            return None;
        }
        let m = &self.metrics;
        // The cosmetic | anchors on the SAME x the caret's CURRENT FORM uses
        // ([`Self::caret_is_bar_form`] — the one owner, so a bar-form caret can
        // never drift back onto the cell centre):
        //   * a CELL-form caret (Block, or Morph settled on a real inhabited
        //     glyph / a glyphless space bar) rests on a CELL → centre the streak
        //     on the cell (half the block width) so the | runs down the MIDDLE.
        //   * a BAR-form caret (I-beam, or Morph's LINE-START degrade — col 0 /
        //     a fresh line / an empty line, where the morph melts to the exact
        //     same bar the I-beam draws) sits at the INSERTION POINT (the thin
        //     bar, centred on `IBEAM_W`) → anchor the | on that bar, NOT the cell
        //     centre, matching `caret_ibeam_geometry`'s `cx = pos.x + thin*0.5`.
        let center_x_drop = if self.caret_is_bar_form() {
            self.ibeam_bar_dims().0 * 0.5
        } else {
            self.caret_block_w() * 0.5
        };
        let (center, half_along, half_across, axis) = self.caret.trail_geometry(
            m.caret_streak_h,
            m.caret_streak_gap,
            m.caret_trail_drop,
            center_x_drop,
        );
        let w = half_along * 2.0;
        if w <= 0.0 {
            return None;
        }
        let corner = STREAK_RADIUS * m.zoom;
        Some((
            center.x,
            center.y,
            w,
            half_across * 2.0,
            corner,
            axis.0,
            axis.1,
            self.caret.trail_alpha(),
        ))
    }

    /// Read-only report of the COSMETIC | TRAIL for the timeline/held-capture sidecar:
    /// `(present, length, vertical, held, alpha, sweep, tail, head)`. `present` is
    /// whether a streak draws this frame; `length` is its on-screen span along the
    /// travel axis (it GROWS old→new as the sweep draws on); `alpha` the current fade;
    /// `vertical` whether it is the up/down | vs a horizontal jump streak; `held`
    /// whether it belongs to an auto-repeat (one steady |); `sweep` ∈ [0,1] the eased
    /// SWEEP progress (0 = head at old, 1 = head arrived on the caret); `tail`/`head`
    /// its endpoints in canvas px (the `head` advances old→new over the sweep). Lets a
    /// capture assert, straight from JSON, that the streak SWEEPS from the old position
    /// toward the caret over the first ~55ms while pos stays pinned, then fades; that a
    /// 1-char hop shows none; a held-down run stays present + steady; a held-right none.
    pub fn caret_cosmetic_report(
        &self,
    ) -> (bool, f32, bool, bool, f32, f32, (f32, f32), (f32, f32)) {
        let held = self.caret.is_trail_held();
        // The eased SWEEP progress (0 = head at the OLD caret, 1 = swept onto the NEW
        // one): exposed straight so a timeline run can assert the sweep old→new without
        // re-deriving it from the endpoints.
        let sweep = self.caret.trail_sweep_p();
        match self.caret_trail_geometry() {
            Some((cx, cy, w, _h, _c, ax, ay, alpha)) => {
                let half = w * 0.5;
                let tail = (cx - ax * half, cy - ay * half);
                let head = (cx + ax * half, cy + ay * half);
                (
                    true,
                    w,
                    self.caret.is_trail_vertical(),
                    held,
                    alpha,
                    sweep,
                    tail,
                    head,
                )
            }
            None => (
                false,
                0.0,
                self.caret.is_trail_vertical(),
                held,
                0.0,
                sweep,
                (0.0, 0.0),
                (0.0, 0.0),
            ),
        }
    }

    /// TYPING IMPACT (PHASE 2): flinch the caret on a typed character — a squash-pop
    /// plus a velocity back-kick against the type direction, velocity-damped so a fast
    /// burst smooths into a slide. Fires in EVERY caret look; delegates to
    /// [`crate::caret::CaretAnim::type_impact`]. The spring self-settles it back to the
    /// same rest, so a settled capture is byte-identical.
    pub fn caret_type_impact(&mut self) {
        self.caret.type_impact();
    }

    /// DELETION SQUASH (PHASE 2): a small inward squash as a backspace / C-d swallows a
    /// character into the caret. Every caret look; delegates to
    /// [`crate::caret::CaretAnim::delete_squash`].
    pub fn caret_delete_squash(&mut self) {
        self.caret.delete_squash();
    }

    /// KILL-LINE GULP (PHASE 2): a bigger caret pulse as C-k swallows a whole line.
    /// Every caret look; delegates to [`crate::caret::CaretAnim::gulp`].
    pub fn caret_gulp(&mut self) {
        self.caret.gulp();
    }

    /// ENTER JUICE — LINE LANDING (PHASE 3): a caret-level touchdown squash as Enter
    /// takes the new line. Every caret look; delegates to
    /// [`crate::caret::CaretAnim::line_land`].
    pub fn caret_line_land(&mut self) {
        self.caret.line_land();
    }

    /// RECOIL the caret in `dir` (a blocked-action bump). Unlike the I-beam typing
    /// kick this fires in EVERY caret look — a blocked motion/scroll/undo/delete
    /// bumps the caret away from the wall in Block/Morph/I-beam alike. Delegates to
    /// [`crate::caret::CaretAnim::recoil`]; the spring self-settles it back to rest.
    pub fn caret_recoil(&mut self, dir: crate::caret::RecoilDir) {
        self.caret.recoil(dir);
    }

    /// Place the caret AT REST on the current target (no glide; settle_factor 1 =
    /// the resting rounded square on the glyph). Used by the deterministic
    /// `--screenshot` path.
    pub fn settle_caret(&mut self) {
        self.set_caret_target(false, false);
        self.caret.snap_to_target();
    }

    /// Inject a deterministic mid-glide state for the `--screenshot-motion`
    /// path: the logical cursor target is the cursor position, but the animated
    /// caret is part-way through a fast HORIZONTAL glide along the line (coming
    /// from the LEFT, heading right toward the target), so its `settle_factor()`
    /// is ~0 — the caret has dropped to the baseline and stretched into a long
    /// trailing underline whose tail points back to the left. A horizontal glide
    /// (the common "move along a line" case) is chosen so the streak + its trail
    /// read clearly. No clock is consulted, so the produced frame is reproducible.
    pub fn inject_motion_demo(&mut self) {
        // Place the logical cursor at a deterministic, comfortably on-screen
        // mid-line spot so the rightward streak AND its leftward tail are fully
        // visible (a cursor at col 0 would push the trailing tail off-screen).
        // Clamp to the document so this is safe on short sample files.
        let demo_line = 2usize.min(self.line_count().saturating_sub(1));
        let line_chars = self.line_glyph_xs(demo_line).len().saturating_sub(1);
        self.cursor_line = demo_line;
        self.cursor_col = 24usize.min(line_chars);
        self.set_caret_target(false, false);
        let (tx, ty) = self.caret_target_xy();
        let target = Sample { x: tx, y: ty };

        // The glide started well to the LEFT of the target and is part-way along,
        // moving RIGHT fast. The animated x is several glyph cells short of the
        // target; the high horizontal speed forces the settle factor toward 0 so
        // the caret is a long trailing streak (tail to the left), not a square.
        let back: f32 = 9.0 * self.metrics.char_width; // ~9 cells left of target
        const PHASE: f32 = 0.55; // fraction of the gap still remaining to the left
        let pos = Sample {
            x: tx - back * PHASE,
            y: ty,
        };
        // Moving rightward (toward the target) fast: the high speed both collapses
        // the settle factor and drives the velocity-scaled streak length long.
        let vel = Sample { x: 1900.0, y: 0.0 };
        self.caret.inject_motion(target, pos, vel);
    }

    /// Vertical sibling of [`Self::inject_motion_demo`] for `--screenshot-motion-v`:
    /// a deterministic mid-glide caret travelling DOWN between lines, coming from
    /// ABOVE the target, so `settle_factor()` is ~0 and the caret has slid to a
    /// thin amber bar on the cell's LEFT edge whose tail trails UP the lines it
    /// passed. No clock is consulted, so the frame is reproducible.
    pub fn inject_motion_demo_vertical(&mut self) {
        // Cursor a few lines down with room ABOVE for the trailing bar to show.
        let demo_line = 6usize.min(self.line_count().saturating_sub(1));
        let line_chars = self.line_glyph_xs(demo_line).len().saturating_sub(1);
        self.cursor_line = demo_line;
        self.cursor_col = 12usize.min(line_chars);
        self.set_caret_target(false, false);
        let (tx, ty) = self.caret_target_xy();
        let target = Sample { x: tx, y: ty };

        // The glide started several lines ABOVE the target and is part-way along,
        // moving DOWN fast. The high vertical speed collapses the settle factor and
        // drives the streak long, so the caret is a tall left-edge bar trailing up.
        let back: f32 = 5.0 * self.metrics.line_height; // ~5 lines above target
        const PHASE: f32 = 0.55; // fraction of the gap still remaining above
        let pos = Sample {
            x: tx,
            y: ty - back * PHASE,
        };
        let vel = Sample { x: 0.0, y: 1900.0 };
        self.caret.inject_motion(target, pos, vel);
    }

    /// DIAGONAL sibling of [`Self::inject_motion_demo`] for `--screenshot-motion-d`:
    /// a deterministic mid-glide caret jumping between two points on DIFFERENT rows
    /// AND columns (e.g. an incremental-search hop between matches), coming from the
    /// upper-LEFT toward the lower-right. The trail must render as a TRUE SLANT from
    /// source to target — not a vertical-only bar (the axis-snapped bug). No clock is
    /// consulted, so the frame is reproducible.
    pub fn inject_motion_demo_diagonal(&mut self) {
        // Land a few lines down and well along the line, with room up-and-left for
        // the trailing slant to show.
        let demo_line = 6usize.min(self.line_count().saturating_sub(1));
        let line_chars = self.line_glyph_xs(demo_line).len().saturating_sub(1);
        self.cursor_line = demo_line;
        self.cursor_col = 22usize.min(line_chars);
        self.set_caret_target(false, false);
        let (tx, ty) = self.caret_target_xy();
        let target = Sample { x: tx, y: ty };

        // The glide started up-and-left of the target and is part-way along, moving
        // DOWN-RIGHT fast. Equal-magnitude x/y velocity ⇒ a ~45° travel vector, so
        // the streak is a clean diagonal tracer (not snapped to either axis).
        let back_x: f32 = 9.0 * self.metrics.char_width;
        let back_y: f32 = 4.0 * self.metrics.line_height;
        const PHASE: f32 = 0.55;
        let pos = Sample {
            x: tx - back_x * PHASE,
            y: ty - back_y * PHASE,
        };
        let vel = Sample {
            x: 1600.0,
            y: 1600.0,
        };
        self.caret.inject_motion(target, pos, vel);
    }

    /// Pin the caret-style picker's preview demo to its SETTLED end-state — the
    /// deterministic frame the headless capture renders (the choreography loop is
    /// live-only): the fully-typed sample line with the caret at rest. No-op when that
    /// picker isn't open. The floating panel + caret are then emitted by
    /// `prepare_caret_preview_panel` from this settled state.
    pub fn settle_caret_preview(&mut self) {
        if self.caret_preview.is_some() {
            self.caret_demo.settle();
        }
    }
}
