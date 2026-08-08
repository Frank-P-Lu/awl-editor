//! Layout-dependent caret geometry, glyph-mask placement, IME rects, and capture
//! reports. Methods remain on [`super::TextPipeline`] because they read its shaped
//! font, layout, and buffer state.

pub(super) use super::caret_body::{InkBox, caret_visual_body_dims};
use super::*;

mod motion;
mod vertical;

/// TARGET-LINE-LOCAL caret glyph record — the shaped glyph clusters of
/// ONE logical line (the cursor line), read straight from that line's OWN
/// [`cosmic_text::BufferLine::layout_opt`] rather than by filtering the whole
/// document's `layout_runs()` stream.
///
/// The caret's per-frame glyph lookups (`cursor_glyph_key_at`, `cluster_char_span`
/// and their consumers — the block ink box, the descender drop, the morph masks)
/// must NOT walk `self.buffer.layout_runs()` from the document TOP, breaking once
/// they pass the cursor line. That walk visits one run per visual row of the
/// whole PREFIX before the caret — so its cost grows with the caret's document
/// POSITION (top: a few runs; tail: every run in the file), re-paid every frame a
/// caret animates. `--bench-caret` witnesses the prefix growth.
///
/// This record avoids it: the clusters come from `bline.layout_opt()` — O(the
/// cursor line's OWN glyphs), independent of how far down the document the caret
/// sits. It is a SINGLE slot (the caret is only ever on one line), rebuilt when
/// the cursor moves to a different line or the shaped geometry changes (a new
/// `RowGeom` generation), NOT a retained document-wide cache. The block / mask /
/// descender / ink-box / cluster-span consumers all share this ONE record.
pub(super) struct CaretLineGlyphs {
    line: usize,
    /// The [`rowgeom::RowGeom`] generation at build time — bumped by every reshape /
    /// zoom / restyle seam, so a stale record (different shaped runs) is rebuilt.
    generation: u64,
    /// `(start_byte, end_byte, CacheKey)` per shaped glyph, in layout (wrap) order —
    /// the exact glyph objects `layout_runs()` would yield for this line, so the
    /// key/span lookups match what a whole-document walk finds.
    clusters: Vec<(usize, usize, CacheKey)>,
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

    pub(super) fn caret_line_glyph_count(&self) -> usize {
        self.ensure_caret_line_glyphs(self.cursor_line);
        self.caret_line_glyphs
            .borrow()
            .as_ref()
            .map(|r| r.clusters.len())
            .unwrap_or(0)
    }

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
        let y = self.caret_cell_top(col) + m.caret_h * 0.5;
        (x, y)
    }

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
    /// A MONO face gets no separate arm: the bundled monos do NOT share one
    /// pitch (own-`hmtx`: Plex Mono/JetBrains 0.60 em, Monaspace Xenon 0.62,
    /// Iosevka 0.50), so a fixed-cell floor keyed on `metrics.caret_w`
    /// (face-INDEPENDENT) would raise the block past the glyph it sits on for
    /// any face narrower than that cell — the same shape the proportional arm
    /// above already fixes for a narrow proportional glyph. The real advance
    /// already IS the uniform mono cell on a real grid (every column shares
    /// one width by construction), so tracking it needs no separate floor.
    pub fn caret_block_w(&self) -> f32 {
        let (_x, adv) = self.col_x_and_advance_aff(
            self.cursor_line,
            self.caret_anchor_col(),
            self.caret_affinity,
        );
        adv
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

    /// Ink-aligned box for a single-glyph proportional anchor. Mono, ligature, and
    /// glyphless anchors use cell geometry to preserve a uniform or fair split.
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
    /// * **INK BOX (a proportional world, one glyph, real ink).** The
    ///   caret spans the anchored glyph's own full raster box grown by
    ///   [`CARET_INK_PAD`] top and bottom. This is the fix for the reported vertical
    ///   misalignment: a FIXED fraction of the row height centred on the line box
    ///   put every letter's top at the same y while the ink's top moves with the
    ///   letter — 8–9px of empty accent above an `a`/`m`/`g`/`y`, ~3px above an
    ///   `l`. Sizing to the box makes the margin a small letter-INDEPENDENT pad
    ///   instead, and covers DESCENDERS through the very same box (`height - top`
    ///   is already below the baseline, so `g`/`y` need no separate rule).
    /// * **PROPORTIONAL FALLBACK, real or synthetic ink.**
    ///   Reached whenever the ink funnel says no to
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
    ///        ink) — BLENDED, deliberately not a hard "nearest wins" pick:
    ///        that pick borrows exactly ONE side's ink, so a single glyphless
    ///        column tied between two DIFFERENT letters (a table's `"| 1"`)
    ///        reads as pure ink from whichever side the tiebreak favours —
    ///        one seam closes to 0.00px while the OTHER inherits the WHOLE
    ///        gap between the two letters undiminished (backward/forward is a
    ///        directional symptom, not a fix — flipping the tiebreak only
    ///        rotates which seam absorbs the full gap), and a run of 4+
    ///        columns flanked by two different letters shows a hard INTERIOR
    ///        FLIP where "nearest" switches sides with nothing between to
    ///        soften it.
    ///        Blending by relative distance (`t = back_dist / (back_dist +
    ///        fwd_dist)`) makes the borrow non-directional: a column tied
    ///        between two different letters reads as their MIDPOINT (each
    ///        seam absorbs roughly half the gap instead of one absorbing all
    ///        of it), and a run sweeps smoothly from one letter's ink to the
    ///        other's with no interior step. When only ONE side has real ink
    ///        (column 0; the tail of a run with nothing real beyond it), the
    ///        blend degenerates to that one box (the literal-adjacency
    ///        fixtures: `aaa`->EOL, `" A"`, `"A  "`);
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
    ///        Frankenstein number: mixing `shaped_font`'s real row metrics
    ///        with `doc_family()`'s ratio produces a measurable few-px pop on
    ///        ordinary text during a theme-picker scrub, and an empty-buffer
    ///        law cannot see it, because an empty buffer never takes the
    ///        real-row branch at all.
    ///        ALL THREE go through this arm rather than the LINE-CELL arm below,
    ///        which is a fixed row-box-centred height with NO baseline/ascent
    ///        reference at all: routing this case (real ligature ink included)
    ///        there makes a proportional world's caret visibly jump
    ///        top/bottom/centre the instant it leaves a real glyph for the very
    ///        next column. Feeding all three references through the identical
    ///        `baseline - top + height/2` / `height + 2*pad` formula means they
    ///        cannot structurally disagree at that seam — only WHICH box (or
    ///        which font) feeds the shared formula differs, never the formula
    ///        itself.
    /// * **LINE CELL (mono only).**
    ///   `caret_block_h` row-scaled, centred on the spring anchor, with the
    ///   DESCENDER-AWARE bottom extension ([`CARET_DESCENDER_PAD`]) folded in
    ///   here rather than at the draw site. The uniform mono grid never
    ///   reads ANY ink box, real or synthetic — every column on a mono row shares
    ///   one `caret.pos.y`/`caret_block_h`, so the cell stays column-independent
    ///   by construction.
    ///
    /// The descender extension lives at this REST endpoint, NOT in
    /// `layers.rs::prepare_caret_block` on the already motion-blended rect
    /// re-scaled by the settle factor. The two are algebraically identical at
    /// rest (settle 1 — the deterministic capture), but only here can
    /// `motion_geometry` blend it out with everything else mid-glide, so the
    /// travelling streak has exactly one thickness rule.
    pub(super) fn caret_cell_vertical(&mut self) -> (f32, f32) {
        let m = self.metrics;
        let px = m.scale;
        if let Some(ink) = self.caret_anchor_ink_box() {
            let (baseline, row_ascent, ascent_font) = self.caret_row_metrics();
            return self.caret_cell_vertical_from_ink(ink, baseline, row_ascent, ascent_font, px);
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
        // `shaped_font` stale until the reshape catches up. Reading
        // `shaped_font` here would leave the caret's OWN geometry stale exactly
        // where the picker's preview law (`render::tests::distinguishability`)
        // proves color retint must fully re-ground the surface.
        if !crate::caret::font_is_mono(self.doc_family()) {
            // PROPORTIONAL FALLBACK: the SAME baseline-relative formula
            // as the ink-box arm, fed (in order) a real ligature raster box, a
            // borrowed NEIGHBOR raster box, or a synthetic typical-letter box —
            // see the doc above.
            let (baseline, row_ascent, ascent_font) = self.caret_row_metrics();
            let col = self.caret_anchor_col();
            let ink = self
                .caret_anchor_raster_box()
                .or_else(|| self.nearest_row_raster_box(self.cursor_line, col))
                .unwrap_or_else(|| self.caret_synthetic_ink_box(row_ascent, ascent_font));
            // A glyphless column borrows or synthesizes the SAME vertical
            // ordinary-letter band as the ink it joins.  Otherwise stepping
            // off punctuation would reintroduce its short body at end-of-line.
            return self.caret_cell_vertical_from_ink(ink, baseline, row_ascent, ascent_font, px);
        }
        let cy = self.caret.pos.y;
        let h = m.caret_block_h * self.cursor_scale();
        let descender = self
            .caret_anchor_raster_box()
            .map(|b| b.descent())
            .unwrap_or(0.0);
        if descender <= 0.0 {
            return (cy, h);
        }
        let desc_bottom = self.caret_baseline_y() + descender + CARET_DESCENDER_PAD.px(px);
        let extend = (desc_bottom - (cy + h * 0.5)).max(0.0);
        (cy + extend * 0.5, h + extend)
    }

    /// The FALLBACK arm's SYNTHETIC ink box for a truly GLYPHLESS
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
    /// cap-height, not bare x-height: a pure x-height reference reproduces the
    /// vertical misalignment in miniature against an ASCENDER neighbour, so the mean is the
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
    /// TARGET-LINE-LOCAL: the owning wrapped row is picked from the cursor
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
    /// the shared core [`Self::caret_baseline_y`] delegates to (`.0`), widened
    /// so the fallback (glyphless) arm of [`Self::caret_cell_vertical`]
    /// can build its SYNTHETIC ink box off the exact same row lookup the real
    /// ink-box arm's baseline comes from. A baseline and "how tall is this row's
    /// ascent" can never disagree about which row they describe, because they
    /// are read in the same pass here.
    ///
    /// `ascent_font` is WHICHEVER font
    /// actually produced `max_ascent`: the real-layout branch reads a shaped
    /// [`cosmic_text::LayoutLine`], a property of `shaped_font` (the face
    /// ACTUALLY on screen this frame, which may still be lagging the live
    /// theme mid theme-picker-preview — `sync_theme_colors` retints instantly
    /// but defers the reshape); the empty-line fallback branch below derives its
    /// ascent purely from `self.metrics` (zoom/DPI only, theme-INDEPENDENT —
    /// `Metrics::with_dpi` never reads the active theme), so it pairs with the
    /// LIVE `doc_family()` instead, matching the arm-selection gate one call up.
    /// [`Self::caret_synthetic_ink_box`] must multiply `max_ascent` by ITS OWN
    /// font's ratio, never a different font's — the seam: keying the ratio
    /// unconditionally on `doc_family()` while the real
    /// branch's ascent stays keyed on `shaped_font` produces a mixed-font number
    /// on ordinary (non-empty) text during a live preview scrub.
    ///
    /// THE ROW LOOKUP ITSELF goes through [`pick_row_index_aff`], never a
    /// HAND-ROLLED predicate (`col >= start_col && col < end_col`, plus an
    /// upstream special case): such a predicate — unlike every OTHER
    /// caret/geometry consumer of a column-to-row lookup — misses the
    /// column sitting AT OR PAST every row's `end_col`, the true END-OF-LINE
    /// column on an UNWRAPPED line. There it falls through to the crude
    /// `font_size * 0.8` ascent GUESS below, while the on-glyph ink-box arm one
    /// column to the left reads the row's REAL `max_ascent` — two different
    /// ascent sources for the SAME physical row (measured: the guess reads
    /// ~30% low against Literata's real row ascent, a residual big enough to
    /// show at that transition). [`pick_row_index_aff`]
    /// is the SAME canonical column→row resolver [`Self::visual_row_top_aff`] and
    /// [`Self::col_x_and_advance_aff`] already ride (its second pass — "no strict
    /// container, use the LAST row the column trails" — is exactly the
    /// end-of-line case the hand-rolled loop lacked), so reusing it here makes
    /// the caret's row ownership one decision instead of three independently
    /// almost-identical ones.
    pub(super) fn caret_row_metrics(&self) -> (f32, f32, &'static str) {
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
        let pen_x = self.caret.pos.x;
        let baseline_y = self.caret_baseline_y();

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

        let morph_t = if self.caret_mask_from.is_some() {
            self.caret.settle_factor()
        } else {
            1.0
        };
        (from_box, to_box, morph_t)
    }

    pub(super) fn prepare_caret_masks(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> bool {
        let to_key = self.caret_inhabited_key();
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
    /// mono / ligature / glyphless line-cell fallback). BOTH ink
    /// corrections are scaled by the settle factor `s`, so they apply only to the
    /// settled quad; a travelling streak still leads from the plain pen x at the
    /// plain line-box centre, unaffected. `&mut self` because the glyph lookup rides
    /// the swash raster cache (the same cost the descender read already paid every
    /// Block frame).
    pub(super) fn caret_geometry(&mut self) -> (f32, f32, f32, f32, f32, f32, f32) {
        let m = self.metrics;
        let s = self.caret.settle_factor();

        let block_w = self.caret_block_w(); // real glyph advance (narrow i, wide m)
        let streak_thin = m.caret_streak_h; // the streak's thin cross-dimension
        let streak_r = m.px(STREAK_RADIUS);
        let corner = streak_r + (m.px(CORNER_RADIUS) - streak_r) * s;

        let speed =
            (self.caret.vel.x * self.caret.vel.x + self.caret.vel.y * self.caret.vel.y).sqrt();
        let streak_len = self.caret.streak_length(
            m.streak_len_for_speed(speed),
            m.caret_streak_max_len,
            m.caret_held_len,
        );
        // Ink-aligned override of the HORIZONTAL rest endpoints (see the doc above):
        // `None` leaves `block_w` and the shift untouched, so mono / ligature /
        // glyphless anchors keep the plain advance cell.
        let (block_w, ink_shift) = match self.caret_anchor_ink_box() {
            Some(ink) => {
                let px = m.scale;
                let (body_w, _body_h) = caret_visual_body_dims(ink, px);
                // Grow equally about the glyph ink centre.  The pen-relative
                // offset is still the raster's real bearing, so the floor does
                // not make a kerned punctuation glyph drift into its neighbour.
                let centered_shift = ink.left + (ink.width - body_w) * 0.5;
                (body_w, centered_shift * s)
            }
            None => (block_w, 0.0),
        };
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
    /// The x position is the delicate part. The resting block (`caret_geometry`)
    /// centers on the cell using the REAL advance (`caret_target_w`): `cx = pos.x +
    /// advance*0.5`. Pinning the thin bar's LEFT edge at `pos.x` instead
    /// (`cx = pos.x + w*0.5`) drops it against the cell's left
    /// edge — at the boundary BEFORE the space, not inside it — because it ignores
    /// the space's advance entirely. Here we center the thin bar on the same cell
    /// midpoint the block uses (`pos.x + advance*0.5`), so it sits in the middle of
    /// the space gap exactly where the block would. It rides the spring anchor
    /// (`pos`) so it slides with the caret. Drawn through the BLOCK pipeline (a
    /// solid accent rounded rect), which is exactly the slim-bar look we want.
    pub(super) fn caret_space_bar_geometry(&mut self) -> (f32, f32, f32, f32, f32) {
        let w = self.metrics.px(CARET_SPACE_BAR_W);
        let (cy, h) = self.caret_cell_vertical();
        let advance = self.caret_target_w();
        let cx = self.caret.pos.x + advance * 0.5;
        let corner = self.metrics.px(CORNER_RADIUS).min(w * 0.5);
        (cx, cy, w, h, corner)
    }

    fn ibeam_bar_dims(&self) -> (f32, f32) {
        let m = &self.metrics;
        (m.px(IBEAM_W), m.caret_h * self.cursor_scale())
    }

    /// Whether the caret is drawing as the THIN INSERTION BAR this frame — the
    /// real I-BEAM look, or MORPH's LINE-START degrade ([`crate::caret::morph_line_start`]
    /// — col 0, a fresh line after Enter, or an empty line), which melts onto the
    /// EXACT SAME bar geometry the I-beam draws ([`Self::ibeam_bar_dims`],
    /// [`Self::caret_linestart_bar_geometry`]). Block, and Morph settled on a
    /// real glyph / a glyphless space bar, are the CELL form instead. THE ONE
    /// owner of "is the caret's current form a bar" — read by the cosmetic |
    /// trail's horizontal anchor ([`Self::caret_trail_geometry`]) so it can
    /// never drift back onto the cell centre for a bar-form caret (special-casing
    /// literal I-beam mode alone is not enough: a Morph caret melted to the
    /// line-start bar would still anchor its trail on the cell midpoint).
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

    pub(super) fn caret_linestart_bar_geometry(&self) -> (f32, f32, f32, f32, f32) {
        let (thin, tall) = self.ibeam_bar_dims();
        let cx = self.caret.pos.x + thin * 0.5;
        let cy = self.caret.pos.y;
        let corner = 0.5 * thin.min(tall);
        (cx, cy, thin, tall, corner)
    }

    pub(super) fn caret_ibeam_geometry(&self) -> (f32, f32, f32, f32, f32) {
        let m = &self.metrics;
        let s = self.caret.settle_factor();
        let motion = 1.0 - s;

        let (thin, tall) = self.ibeam_bar_dims();
        let holding = self.caret.is_holding();
        let gap = if holding {
            m.caret_streak_gap * crate::caret::HELD_GAP_FRAC
        } else {
            m.caret_streak_gap
        };
        let held_len = m.caret_held_len;
        let motion = if holding { 1.0 } else { motion };

        let (vx, vy) = (self.caret.vel.x, self.caret.vel.y);
        let dxt = self.caret.target.x - self.caret.pos.x;
        let dyt = self.caret.target.y - self.caret.pos.y;

        if self.caret.is_vertical_move() {
            let mut raw = m
                .streak_len_for_speed(vy.abs())
                .max(self.caret.frame_dy().abs());
            if holding {
                raw = held_len.min(m.caret_streak_max_len);
            }
            let streak_len = (raw - gap).max(tall);
            let w = thin;
            let h = tall + (streak_len - tall) * motion;
            let cx = self.caret.pos.x + w * 0.5;
            let dir = if vy.abs() > 1.0 {
                vy.signum()
            } else if dyt.abs() > f32::EPSILON {
                dyt.signum()
            } else {
                1.0
            };
            let cy =
                self.caret.pos.y + m.caret_trail_drop * motion - dir * ((h - tall) * 0.5) * motion;
            let corner = 0.5 * w.min(h);
            return (cx, cy, w, h, corner);
        }

        let mut raw = m
            .streak_len_for_speed(vx.abs())
            .max(self.caret.frame_dx().abs());
        if holding {
            raw = held_len.min(m.caret_streak_max_len);
        }
        let streak_len = (raw - gap).max(thin);
        let w = thin + (streak_len - thin) * motion;
        let h = tall + (thin - tall) * motion;
        let lead = self.caret.pos.x + thin * 0.5;
        let dir = if vx.abs() > 1.0 {
            vx.signum()
        } else if dxt.abs() > f32::EPSILON {
            dxt.signum()
        } else {
            1.0
        };
        let cx = lead - dir * (w * 0.5) * motion;
        let cy = self.caret.pos.y + m.caret_trail_drop * motion;
        let corner = 0.5 * w.min(h);
        (cx, cy, w, h, corner)
    }
}
