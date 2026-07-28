//! Per-frame preparation for background, text, caret, highlights, chrome, and spell
//! underlines. These methods remain on [`super::TextPipeline`] because they share its
//! GPU state, atlas, viewport, and buffers.

use super::*;

pub(in crate::render) mod fold_chevron;
mod ornaments;
mod table_grid;
mod table_layout;
mod table_xray;
pub(crate) use table_layout::TableGridCache;

/// The vertical page-frame bounds shared by preparation and its laws. A frame
/// is a writing-surface boundary, not a bracket around the last glyph: its
/// bottom therefore reaches the editor canvas even when the document is short.
///
/// `menubar_bottom` is zero without awl's rendered menu bar and otherwise the
/// bar's lower edge. A scrolled document may begin above that edge, but its
/// frame may not paint through the persistent chrome. `canvas_bottom` is the
/// last addressable canvas row, so all four frame edges remain on-canvas.
pub(crate) fn page_frame_vertical_bounds(
    doc_top: f32,
    doc_height: f32,
    menubar_bottom: f32,
    canvas_bottom: f32,
) -> (f32, f32) {
    let canvas_bottom = canvas_bottom.max(menubar_bottom);
    let top = doc_top.max(menubar_bottom).min(canvas_bottom);
    let document_bottom = doc_top + doc_height.max(0.0);
    let bottom = document_bottom.max(canvas_bottom).clamp(top, canvas_bottom);
    (top, bottom)
}

/// The hanging BLOCKQUOTE pull-quote mark: a big DIM opening quotation mark (`“`)
/// shaped in the WORLD'S OWN DISPLAY SERIF ([`theme::Theme::font`], NOT the ornament
/// or symbol face) and hung in the LEFT MARGIN at each blockquote block's first line
/// — semantically honest for a quote, and a showcase of the world's type. TUNABLE
/// (live-taste): the size bump over body ink. Sits in the heading/ornament band
/// (~1.8–2.2×); still value-only, never amber (DESIGN §3). See
/// [`super::TextPipeline::prepare_ornaments`].
const QUOTE_MARK_SCALE: f32 = 2.0;

/// The glyph the pull-quote mark draws — U+201C LEFT DOUBLE QUOTATION MARK (the
/// pull-quote's opening mark). Shaped in the world's display serif so it reads as
/// real type, not a symbol-font ornament.
const QUOTE_MARK_GLYPH: char = '\u{201C}';

fn fold_tail_text(n: usize) -> String {
    if n == 1 {
        "\u{2026} 1 line".to_string()
    } else {
        format!("\u{2026} {n} lines")
    }
}

#[cfg(not(target_arch = "wasm32"))]
const IMAGE_CORNER_PX: f32 = 4.0;

#[cfg(not(target_arch = "wasm32"))]
const IMAGE_REVEAL_DIM_ALPHA: f32 = 0.4;

#[cfg(not(target_arch = "wasm32"))]
const CAPTION_SCRIM_PAD_Y: f32 = 3.0;

#[cfg(not(target_arch = "wasm32"))]
const CAPTION_SCRIM_PAD_X: f32 = 4.0;

impl TextPipeline {
    /// Per-frame PAGE-MODE margin gradient: punch a hole for the page column and
    /// paint the margins (the whole canvas, no margins, when page mode is off).
    pub(crate) fn prepare_background_layer(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        // Punch a page-column hole; page-off passes the full width, hiding the margins.
        let (page_on, _measure, col_left, col_w) = self.page_geometry();
        let (bg_left, bg_w) = if page_on {
            (col_left, col_w)
        } else {
            (0.0, width as f32)
        };
        let drift = if self.effective_background().is_waves() {
            crate::background::waves_drift_radians(self.waves_render_phase())
        } else if self.effective_background().is_organic() {
            self.waves_render_phase() * std::f32::consts::TAU / crate::lava::LAVA_LOOP_CYCLES
        } else {
            0.0
        };
        self.background_pipeline
            .prepare(queue, width, height, bg_left, bg_w, drift);
    }

    pub(crate) fn prepare_lava_layer(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        let (page_on, _measure, col_left, col_w) = self.page_geometry();
        let (bg_left, bg_w) = if page_on {
            (col_left, col_w)
        } else {
            (0.0, width as f32)
        };
        let rail_carved = self.lava_rail_carved(height);
        let gutter_rect = self.lava_gutter_carve_rect(height);
        // ORGANIC FROST SEEDS (the shipped headed-doc default): every visible margin
        // glyph (the outline entries + the bottom-left gutter's filename/project
        // stack) seeds a small close halo `[x0, x1, yc, r]`; the shader SUMS them
        // into one continuous field and thresholds it, so nearby words/rows merge
        // into organic islands with NO per-row separation and NO zoom breakpoint
        // (see `shaders/lava.wgsl` + `crate::lava::frost_coverage`). The gutter's
        // old hard corner carve — an ugly geometric dark pocket — is subsumed: its
        // stack rides the SAME softened, value-dimmed field. EMPTY in every
        // non-frost frame (no lava ground, no margin ink, or `AWL_LAVA_FROST=off`),
        // so the shader's frost path is inert (`seed_count == 0`) and a heading-less,
        // gutter-less doc stays byte-identical. The gutter seeds LEAD so they always
        // survive the `MAX_FROST_SEEDS` clamp.
        //
        // PROTO-CACHE (the `render/rects.rs` shape): the seed field is REBUILT only
        // when its key misses ([`Self::frost_seed_key`] — viewport, zoom/DPI, the
        // column, and the drawn outline/gutter TEXT), so warm steady frames pay ZERO
        // rebuilds and a margin-text or zoom change pays EXACTLY ONE. `--bench-frost`
        // witnesses this (a bench that reshaped nothing would be a lie).
        let frost_active =
            crate::lava::frost_on() && self.effective_background().lava_params().is_some();
        if frost_active {
            let key = self.frost_seed_key(width, height);
            if self.frost_seed_key != Some(key) {
                let mut seeds = self.gutter_frost_seeds(height);
                seeds.extend(self.outline_frost_seeds(height));
                seeds.truncate(crate::lava::MAX_FROST_SEEDS);
                self.frost_seeds = seeds;
                self.frost_seed_key = Some(key);
                self.frost_seed_rebuilds += 1;
            }
        } else if self.frost_seed_key.is_some() {
            self.frost_seeds.clear();
            self.frost_seed_key = None;
        }
        // FROST RECIPE AS DATA: the ONE runtime consumer reads the active world's
        // `frost` capability (see `theme::Frost`), never the bare consts — so a
        // world dials its own softened-lamp recipe with no per-world code path
        // (`theme_caps_law`). Every world carries `Frost::DEFAULT` (the shipped
        // `crate::lava` values), so this is byte-identical until a world tunes. The
        // third slot carries the field ISO the shader thresholds the summed halos at.
        let frost = crate::theme::active().render_caps.frost;
        let frost_params = [
            frost.dim,
            crate::lava::frost_px(frost.blur_px, self.metrics.zoom, self.dpi),
            crate::lava::FROST_ISO,
        ];
        let params =
            self.effective_background()
                .lava_params()
                .map(|(ground, lo, hi, edge, dithered)| {
                    (
                        ground,
                        lo,
                        hi,
                        edge,
                        crate::lava::dither_for_blur(dithered, self.backdrop_blur()),
                    )
                });
        let phase = self.lava_render_phase();
        self.lava_pipeline.prepare(
            queue,
            width,
            height,
            self.lava_field_viewport,
            bg_left,
            bg_w,
            rail_carved,
            gutter_rect,
            &self.frost_seeds,
            frost_params,
            params,
            phase,
        );
    }

    pub fn frost_seed_count(&self) -> usize {
        self.frost_seeds.len()
    }

    /// THE FROST SEED-FIELD CACHE KEY for this frame — the proto-cache key the
    /// organic frost seeds rebuild on a MISS of (see [`Self::prepare_lava_layer`]).
    /// Captures every input the seed geometry derives from WITHOUT shaping any text:
    /// the physical viewport, the user zoom × device DPI, the writing column's left
    /// edge (`page_geometry`, the one owner), and the DRAWN margin TEXT — the
    /// outline's followed rows (each fitted label + its group-gap / current flags +
    /// `top`/`right_edge`) and the gutter's filename/project + `avail`. So warm
    /// steady frames hash the SAME key (zero rebuilds) and a margin-text edit, a
    /// follow-window slide, a resize, or a zoom step flips it (exactly one rebuild).
    pub(crate) fn frost_seed_key(&self, width: u32, height: u32) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        width.hash(&mut h);
        height.hash(&mut h);
        self.metrics.zoom.to_bits().hash(&mut h);
        self.dpi.to_bits().hash(&mut h);
        crate::theme::active_index().hash(&mut h);
        let (_page_on, _measure, col_left, col_w) = self.page_geometry();
        col_left.to_bits().hash(&mut h);
        col_w.to_bits().hash(&mut h);
        // The DRAWN outline rows (char-level layout — no GPU shaping): any change to
        // which headings show, their fitted text, the follow slice, or the group
        // rhythm flips the key. Uses the test-visible draw report shape without the
        // pixel-fit (that only shifts with zoom/DPI, already in the key).
        if let Some(rows) = self.outline_key_rows(height) {
            rows.hash(&mut h);
        }
        if let Some((name, project)) = self.gutter_report() {
            name.hash(&mut h);
            project.hash(&mut h);
        }
        h.finish()
    }

    /// TWINKLING STARS (`theme::AmbientStyle::Stars`, the TWINKLING-STARS round):
    /// tiny individually-phased breathing points in the page-mode MARGINS,
    /// drawn right after the lava layer and before the page frame / washes /
    /// text. A TOTAL no-op (zero instances) for every `AmbientStyle::None`
    /// world — fifteen of the sixteen stay byte-identical — and for page-off
    /// (the column spans the canvas → the margin gate culls everything, the
    /// background pass's own collapse).
    ///
    /// THE SHAPE: the star LAYOUT is a proto cache — [`crate::stars::layout`]'s
    /// deterministic position-hash scatter, rebuilt only when the (viewport,
    /// star params) key misses ([`TextPipeline::stars_proto_key`]) — and the
    /// per-frame work is pure arithmetic over the visible set: cull each proto
    /// against the LIVE column band ([`crate::stars::in_margin`], the one
    /// placement-law owner, reading the SAME `page_geometry()` every ground
    /// layer reads — an adaptive-column shift or live resize re-culls the same
    /// anchored field) plus the margin-INK zones (the outline's pill rects +
    /// the gutter's corner rect, the same owners the lava frost/carve reads —
    /// so a star never sits under the dim rail text), then breathe its alpha
    /// by [`crate::stars::brightness`] at the resolved twinkle phase
    /// ([`TextPipeline::stars_render_phase`]: env knob > Reduce-Motion freeze >
    /// the shared ambient clock — frozen at t=0 in every headless capture).
    /// Each star's own dot size is `size_px` scaled by
    /// [`crate::stars::star_size_scale`] (a small hash roll off its seed,
    /// item 62) — deterministic, so a star's size never drifts frame to frame.
    pub(crate) fn prepare_stars_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let Some((tint, cell_px, density, size_px, peak, floor)) =
            crate::theme::active().render_caps.ambient.stars_params()
        else {
            // A starless world uploads ZERO instances (and clears the proto
            // cache so a later switch back rebuilds against fresh params).
            self.stars_proto_key = None;
            self.stars_pipeline
                .prepare_multicolor(device, queue, width, height, &[]);
            return;
        };
        let scale = self.metrics.zoom * self.dpi;
        let cell_px = cell_px * scale;
        let size_px = size_px * scale;
        // Proto cache: rebuild the scattered layout only when the size, the DPI/zoom
        // scale, or the authored params change (a theme switch onto different star
        // data). The scale rides in through the now-scaled `cell_px`/`size_px`.
        let key = (width, height, cell_px.to_bits(), density.to_bits());
        if self.stars_proto_key != Some(key) {
            self.stars_protos = crate::stars::layout(width as f32, height as f32, cell_px, density);
            self.stars_proto_key = Some(key);
        }
        let (page_on, _measure, col_left, col_w) = self.page_geometry();
        let (band_left, band_w) = if page_on {
            (col_left, col_w)
        } else {
            (0.0, width as f32)
        };
        let band_right = band_left + band_w;
        let mut ink_zones: Vec<[f32; 4]> = if self.outline_visible(height) {
            self.lava_frost_pill_rects(height)
        } else {
            Vec::new()
        };
        if self.gutter_visible()
            && let Some(r) = self.gutter_carve_rect(height)
        {
            ink_zones.push(r);
        }
        let phase = self.stars_render_phase();
        let gap = crate::stars::STAR_MARGIN_GAP_PX;
        // SIZE SPREAD (item 62, 2026-07-24): each star's own dot size is
        // `size_px` scaled by a small hash-derived multiplier off its seed
        // (`crate::stars::star_size_scale`) — deterministic and stable across
        // frames, never true randomness. The WIDEST half-size the spread
        // allows becomes the ONE uploaded corner radius (`Globals::corner`,
        // shared by every instance in the draw call); the shader clamps it
        // per instance to that star's own half-extent
        // (`min(g.corner, hsize)`, `selection.wgsl`), so a smaller star still
        // renders as a full circle rather than a rounded square.
        let max_half = size_px * 0.5 * (1.0 + crate::stars::STAR_SIZE_SPREAD_FRAC);
        let mut quads: Vec<([f32; 4], [u8; 4])> = Vec::with_capacity(self.stars_protos.len());
        for s in &self.stars_protos {
            let star_size = size_px * crate::stars::star_size_scale(s.seed);
            let half = star_size * 0.5;
            if !crate::stars::in_margin(s.x, half, band_left, band_right, gap) {
                continue;
            }
            let e = half + 1.0;
            if ink_zones
                .iter()
                .any(|r| s.x + e > r[0] && s.x - e < r[2] && s.y + e > r[1] && s.y - e < r[3])
            {
                continue;
            }
            let a = crate::stars::brightness(s.seed, phase, floor, peak);
            let alpha = (a * 255.0).round().clamp(0.0, 255.0) as u8;
            if alpha == 0 {
                continue;
            }
            let st = crate::stars::star_tint(tint, s.seed);
            quads.push((
                [s.x - half, s.y - half, star_size, star_size],
                [st.r, st.g, st.b, alpha],
            ));
        }
        // Fully-rounded corners turn each tiny quad into a soft dot (the SDF
        // becomes a circle when the radius reaches the half-size) — set to the
        // WIDEST star's half so the per-instance shader clamp rounds every
        // smaller star fully too (see the size-spread note above).
        self.stars_pipeline.set_corner(max_half);
        self.stars_pipeline
            .prepare_multicolor(device, queue, width, height, &quads);
    }

    pub(crate) fn prepare_page_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        if !crate::page::page_on() {
            self.page_frame_pipeline
                .prepare(device, queue, width, height, &[]);
            return;
        }
        let theme::PageFrame::Line { weight_px } = crate::render::effective_page_frame() else {
            self.page_frame_pipeline
                .prepare(device, queue, width, height, &[]);
            return;
        };
        let t = weight_px.max(0.1);
        let left = self.column_left();
        let w = self.column_width();
        let (top, bottom) = page_frame_vertical_bounds(
            self.doc_top(),
            self.total_doc_height(),
            self.menubar_reserve(),
            height as f32 - 1.0,
        );
        let h = (bottom - top).max(0.0);
        let right = left + w;
        let rects = [
            [left - t, top - t, w + 2.0 * t, t], // top edge
            [left - t, bottom, w + 2.0 * t, t],  // bottom edge
            [left - t, top - t, t, h + 2.0 * t], // left edge
            [right, top - t, t, h + 2.0 * t],    // right edge
        ];
        self.page_frame_pipeline
            .prepare(device, queue, width, height, &rects);
    }

    /// THE FULL LEFT-MARGIN RAIL CARVE decision for this frame — the OLD headed-doc
    /// treatment, now DEMOTED behind the FROST default. Under the shipped default
    /// ([`crate::lava::FROST_RAIL_DEFAULT`] `true`) this is ALWAYS `false`: a headed
    /// doc keeps BOTH margins alive and the per-entry frost pills
    /// ([`Self::lava_frost_pill_rects`]) carry the outline's legibility instead of
    /// flattening the whole rail. Flipping that ONE const to `false` re-arms this
    /// carve — a lava ground active (the CAPABILITY, never a world name, per
    /// `theme_caps_law`) AND the margin OUTLINE actually DRAWN
    /// ([`Self::outline_visible`], the same `outline_layout` gate) — feeding the
    /// shader's still-wired `rail` global, for a clean one-line data revert to the
    /// pre-frost behaviour. The ONE owner [`Self::prepare_lava_layer`] uploads it.
    pub(crate) fn lava_rail_carved(&self, height: u32) -> bool {
        !crate::lava::FROST_RAIL_DEFAULT
            && self.effective_background().lava_params().is_some()
            && self.outline_visible(height)
    }

    /// THE GUTTER'S LOCAL CORNER CARVE rect — the HARD carve, now DEMOTED behind
    /// the FROST default (like [`Self::lava_rail_carved`]). Under the shipped
    /// default ([`crate::lava::FROST_RAIL_DEFAULT`] `true`) this is ALWAYS `None`:
    /// the gutter is a FROST PILL instead ([`Self::lava_gutter_frost_rect`]), so
    /// its `muted`/`faint` stack sits on a softened, value-dimmed lamp rather than
    /// the old hard-carved dead-flat corner (which revealed the world's darkest
    /// ground — an ugly geometric dark pocket). Flipping that ONE const to `false`
    /// re-arms this hard carve — a lava ground active AND the bottom-left GUTTER
    /// actually DRAWN ([`Self::gutter_visible`]) — feeding the shader's still-wired
    /// `gutter`/`gutter_rect` globals, for a clean one-line data revert to the
    /// pre-frost behaviour. Geometry comes from the SAME [`Self::gutter_carve_rect`]
    /// owner `prepare_gutter`/`gutter_layout` ride, so the carve (and the frost
    /// pill that replaced it) can never disagree with the drawn gutter block.
    pub(crate) fn lava_gutter_carve_rect(&self, height: u32) -> Option<[f32; 4]> {
        if crate::lava::FROST_RAIL_DEFAULT
            || self.effective_background().lava_params().is_none()
            || !self.gutter_visible()
        {
            return None;
        }
        self.gutter_carve_rect(height)
    }

    pub(crate) fn prepare_text_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        // DIFF-AS-PREVIEW: while the page column wears its card dressing, the
        // document glyphs clip to the panel's interior band, so a scrolled
        // transcript slides UNDER the card edge instead of over the margin.
        let (clip_top, clip_bottom) = match self.doc_clip_band(height as f32) {
            Some((t, b)) => (t as i32, b as i32),
            None => (0, height as i32),
        };
        let bounds = TextBounds {
            left: 0,
            top: clip_top,
            right: width as i32,
            bottom: clip_bottom,
        };
        let doc_top = self.doc_top();

        let default_color = theme::base_content().to_glyphon();
        let text_area = TextArea {
            buffer: &self.buffer,
            left: self.text_left(),
            top: doc_top,
            scale: 1.0,
            bounds,
            default_color,
            custom_glyphs: &[],
        };

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                // Text only; the caret is a GPU quad drawn underneath the text
                // in the render pass (clear -> caret -> text).
                [text_area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon prepare failed: {e:?}"))?;
        Ok(())
    }

    pub(crate) fn prepare_caret_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        // The caret has two selectable LOOKS (block vs glyph-silhouette morph).
        // Exactly one of the two pipelines emits geometry per frame; the other is
        // cleared so nothing stale lingers when the mode (or fallback) changes.
        //
        // BLOCK: `caret_geometry` reads the spring's settle factor to interpolate
        // between the resting rounded square (full advance width) and the moving
        // trailing-underline streak, and the real glyph advance so a full-width CJK
        // glyph gets a full-width block (Latin keeps caret_w). Drawn UNDER the text.
        //
        // MORPH has three sub-cases, all keyed off the spring:
        //   * FAST MOTION (settle_factor < SHOW threshold) → DEFER to the BLOCK
        //     pipeline's trailing-underline STREAK. Holding an arrow / a big jump
        //     makes the spring lag, settle drops toward 0, and the streak shows; the
        //     per-glyph silhouette would strobe badly during travel, so we don't
        //     paint it until motion settles.
        //   * SETTLED on a real INHABITED glyph → paint the accent SILHOUETTE
        //     (glyph pipeline, OVER the text) with its glyph-to-glyph cross-fade
        //     as it lands.
        //   * NOTHING to inhabit → a SLIM accent bar via the BLOCK pipeline (a
        //     thin I-beam, not a full block). Two flavours below: a LINE START
        //     (col 0 — no produced glyph before the insertion point) degrades to
        //     the I-beam's insertion bar at the insertion x; a GLYPHLESS anchor
        //     past col 0 (the space just typed / emoji) keeps the cell-centered
        //     space bar.
        // THE 1-BIT CARET ROUND: `caret_invert` is parked EMPTY up front, so
        // any branch that does NOT draw a block-look caret this frame —
        // Ibeam, or (on an ORDINARY world) the morph silhouette / glyphless
        // bar — and a live theme switch AWAY from a one-bit world never
        // leaves a stale rect from a PRIOR frame's inverted block still
        // drawing. Only `prepare_caret_block`, when it runs on a one-bit
        // world, repopulates it with this frame's real rect.
        self.caret_invert.prepare(device, queue, width, height, &[]);
        // ITEM 84 / DIFF-AS-PREVIEW: the caret is SELECTION-ADJACENT geometry
        // too (quads don't clip to `TextBounds` the way glyphs do), so it reads
        // the SAME `content_clip` every other selection-quad path routes
        // through — the writing column horizontally (always), narrowed to the
        // diff panel's own inset band vertically while a preview is up. Outside
        // either bound, park every caret quad rather than let it paint over the
        // card's rim / page margin. In the ordinary (non-diff) case the caret's
        // x is always inside the writing column by construction, so this is a
        // no-op there — the diff-scroll-past-the-band case is what actually
        // trips it.
        let (cx0, cy0, cx1, cy1) = self.content_clip();
        let (cx, cy, cw, ch) = self.caret_pixel_rect();
        if cx < cx0 || cx + cw > cx1 || cy < cy0 || cy + ch > cy1 {
            self.caret_pipeline.prepare_empty();
            self.caret_glyph_pipeline.clear();
            return;
        }
        // MORPH FOLDS TO BLOCK ON AN INK-CARET WORLD (both special block styles;
        // documented call — see CLAUDE.md's "1-bit Wagtail caret" round /
        // `caret_invert`'s field doc + `CaretBlockStyle::folds_morph_to_block`):
        // the glyph-silhouette look recolors the cursor's own letter to `primary`,
        // which on an ink-caret world is the SAME value as the letter's own ink —
        // an invisible no-op recolor (Wagtail's white-on-white), or, for a Filled
        // world, a green silhouette that vanishes into the green block beneath it.
        // Building a distinct glyph-morph for that would be per-glyph pipeline work
        // for a mode whose selling point (a colored accent letter) doesn't exist
        // when the caret IS the ink; the block path already makes the letter
        // legible (InverseVideo flips it, Filled knocks it out), so Morph degrades
        // to Block here. Ibeam is UNCHANGED — its thin bar sits BETWEEN glyph
        // cells, never over one, so it never collides with a glyph's own ink.
        // Read the PER-FRAME latched look (`caret_look`), not the live global, so
        // the paint path agrees with the geometry — and so a live drag's insertion
        // BAR override (`ViewState::selecting_drag`, latched into `caret_look`)
        // reaches the draw path too. When not dragging, `caret_look` == the global,
        // so every non-drag frame is byte-identical.
        let mode = if theme::active()
            .render_caps
            .caret_block_style
            .folds_morph_to_block()
            && self.caret_look == CaretMode::Morph
        {
            CaretMode::Block
        } else {
            self.caret_look
        };
        let settle = self.caret.settle_factor();
        let has_glyph = mode == CaretMode::Morph && self.prepare_caret_masks(device, queue);
        let paint_silhouette = has_glyph && settle >= CARET_MORPH_SETTLE_SHOW;
        let paint_space_bar =
            mode == CaretMode::Morph && !has_glyph && settle >= CARET_MORPH_SETTLE_SHOW;
        if mode == CaretMode::Ibeam {
            let (cx, cy, cw, ch, ccorner) = self.caret_ibeam_geometry();
            let (cw, ch, ccorner) = self.pop_scaled(cw, ch, ccorner);
            self.caret_pipeline
                .prepare(queue, width, height, cx, cy, cw, ch, ccorner);
            self.caret_glyph_pipeline.clear();
        } else if paint_silhouette {
            let (from_box, to_box, morph_t) = self.caret_glyph_geometry();
            self.caret_glyph_pipeline.prepare(
                device,
                queue,
                width,
                height,
                self.caret_mask_from.as_ref(),
                from_box,
                self.caret_mask_to.as_ref(),
                to_box,
                morph_t,
                1.0,
                CARET_MORPH_DILATE_PX * self.metrics.zoom,
            );
            self.caret_pipeline.prepare_empty();
        } else if paint_space_bar {
            let (cx, cy, cw, ch, ccorner) = if crate::caret::morph_line_start(self.cursor_col) {
                self.caret_linestart_bar_geometry()
            } else {
                self.caret_space_bar_geometry()
            };
            let (cw, ch, ccorner) = self.pop_scaled(cw, ch, ccorner);
            self.caret_pipeline
                .prepare(queue, width, height, cx, cy, cw, ch, ccorner);
            self.caret_glyph_pipeline.clear();
        } else {
            self.prepare_caret_block(device, queue, width, height);
        }

        self.prepare_caret_trail(queue, width, height);
    }

    /// BLOCK-caret upload — the settle-driven resting square ⇄ trailing-underline
    /// streak, oriented along the true travel vector. The fast-travel MORPH path
    /// defers here too (the per-glyph silhouette would strobe), so this is the shared
    /// block/streak draw. Lifted verbatim out of [`prepare_caret_layer`]'s final
    /// dispatch arm; byte-identical on every ORDINARY world — see the one-bit branch
    /// at the bottom (added by THE 1-BIT CARET ROUND) for the true-inverse-video path.
    ///
    /// ITEM 91: this site computes NO geometry of its own. The caret's vertical
    /// extent — the anchored glyph's padded INK BOX on a proportional world, the
    /// row-scaled line cell WITH its descender-aware bottom on a mono / ligature /
    /// glyphless anchor — belongs entirely to `caret_cell_vertical`, folded into
    /// `caret_geometry`'s rest endpoints. The descender extension used to be
    /// re-derived HERE off the already motion-blended rect; keeping a second
    /// vertical rule at the draw site is exactly how the top edge came to disagree
    /// with the bottom. `render::tests::caret_ink_box`'s grep-law fails if a raster
    /// box, a descender depth or a line-cell height reappears in this file.
    fn prepare_caret_block(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let (cx, cy, cw, ch, ccorner, ax, ay) = self.caret_geometry();
        let (cw, ch, ccorner) = self.pop_scaled(cw, ch, ccorner);

        match theme::active().render_caps.caret_block_style {
            theme::CaretBlockStyle::Filled => {
                // THE AUTHENTIC CRT PHOSPHOR BLOCK (an ink-caret world, `primary ==
                // base_content`; Cassowary): an opaque `primary` cell drawn UNDER the
                // text exactly like an ORDINARY world's block (below), PLUS the covered
                // glyph knocked back through in the GROUND colour. A plain block here
                // would paint green-on-green and ERASE the letter (unlike an amber-vs-ink
                // block, whose value contrast keeps the letter legible); the knockout
                // fixes that WITHOUT the `InverseVideo` photo-negative (which on a
                // chromatic ink would flip green → magenta, not a lit cell).
                self.caret_pipeline
                    .prepare_directed(queue, width, height, cx, cy, cw, ch, ccorner, ax, ay);
                let settled = self.caret.settle_factor() >= CARET_MORPH_SETTLE_SHOW;
                if settled && self.prepare_caret_masks(device, queue) {
                    self.caret_glyph_pipeline
                        .set_color(theme::primary_content().rgb_bytes());
                    let (from_box, to_box, morph_t) = self.caret_glyph_geometry();
                    self.caret_glyph_pipeline.prepare(
                        device,
                        queue,
                        width,
                        height,
                        self.caret_mask_from.as_ref(),
                        from_box,
                        self.caret_mask_to.as_ref(),
                        to_box,
                        morph_t,
                        1.0,
                        CARET_MORPH_DILATE_PX * self.metrics.zoom,
                    );
                } else {
                    self.caret_glyph_pipeline.clear();
                }
            }
            theme::CaretBlockStyle::InverseVideo => {
                self.caret_glyph_pipeline.clear();
                // TRUE 1-BIT WORLDS: an opaque pre-text quad here — even one
                // tinted `primary` (pure white on a one-bit world, the SAME
                // value as the text ink) — would white-out a glyph the caret
                // lands on: the glyph's own alpha-blended draw on TOP of an
                // already-white quad composites into uniform white with no
                // visible seam (the exact bug this round fixes — a caret on a
                // heading's `#` erased the `#`). Route the caret's own ANIMATED
                // rect (this frame's settle-driven position + streak size, from
                // `caret_geometry`/the descender extension above — travel-axis
                // ROTATION is dropped, `fs_invert` has no axis field, and a
                // rotated streak is rare + still legible axis-aligned) through
                // `caret_invert` instead: drawn AFTER text with a `OneMinusDst`
                // blend, so it flips whatever the ground+text already
                // composited to beneath it — black ground -> white, white glyph
                // ink -> black — making the glyph under the caret legible.
                // `caret_pipeline` draws NOTHING this frame (`prepare_empty`):
                // an opaque quad here would hand the invert pass a uniform-white
                // destination with nothing left to flip into a visible glyph.
                //
                // THE ROUNDED-SILHOUETTE FIX: `ccorner` here is the EXACT SAME
                // already-zoom/settle/squash-animated radius `caret_geometry` +
                // `pop_scaled` computed above — the ONE Rust-side owner an
                // ORDINARY world's `caret_pipeline.prepare_directed` call below
                // draws with too. Uploading it into `caret_invert` via
                // `set_corner` (consumed by `fs_invert`'s SDF discard — see that
                // shader entry point's doc) makes the 1-bit caret's silhouette
                // round the SAME way, rather than falling back to a hard square.
                self.caret_invert.set_corner(ccorner);
                let rect = [cx - cw * 0.5, cy - ch * 0.5, cw, ch];
                self.caret_invert
                    .prepare(device, queue, width, height, &[rect]);
                self.caret_pipeline.prepare_empty();
            }
            theme::CaretBlockStyle::Normal => {
                self.caret_glyph_pipeline.clear();
                self.caret_pipeline
                    .prepare_directed(queue, width, height, cx, cy, cw, ch, ccorner, ax, ay);
            }
        }
    }

    fn prepare_caret_trail(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        match self.caret_trail_geometry() {
            Some((cx, cy, cw, ch, ccorner, ax, ay, alpha)) => {
                self.caret_trail_pipeline
                    .prepare_axis(queue, width, height, cx, cy, cw, ch, ccorner, alpha, ax, ay);
            }
            None => self.caret_trail_pipeline.prepare_empty(),
        }
    }

    /// Build + upload the SYNTAX WASH quads: the warm low-alpha band behind every
    /// PROSE-comment span (all worlds — the identity carrier now that prose
    /// comments ride FULL ink), the green band behind string spans (dark worlds
    /// only), and the DEDICATED violet band behind every markdown `==highlight==`
    /// span (all worlds — decoupled from the comment wash so it POPS, see
    /// [`super::spans::highlight_wash`]). Geometry comes from the proto-cached
    /// [`TextPipeline::wash_rects`] (O(visible) per frame); the comment/string
    /// buckets are GATED here on the ACTIVE world's effective [`role_style_for`]
    /// wash — a role with no wash (light-world strings, or a world that opted out
    /// via `Theme::role_overrides`) uploads ZERO instances, so nothing draws (the
    /// highlight bucket has no opt-out, but an empty rect list draws nothing just
    /// the same). Empty for prose / non-highlight / non-fence buffers, keeping
    /// those frames byte-identical.
    pub(crate) fn prepare_wash_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let (mut comment_rects, mut string_rects, highlight_rects) = self.wash_rects();
        let th = theme::active();
        if role_style_for(&th, crate::syntax::SynKind::Comment)
            .wash
            .is_none()
        {
            comment_rects.clear();
        }
        if role_style_for(&th, crate::syntax::SynKind::Str)
            .wash
            .is_none()
        {
            string_rects.clear();
        }
        self.wash_comment_pipeline
            .prepare(device, queue, width, height, &comment_rects);
        self.wash_string_pipeline
            .prepare(device, queue, width, height, &string_rects);
        self.wash_highlight_pipeline
            .prepare(device, queue, width, height, &highlight_rects);
    }

    pub(crate) fn prepare_wysiwyg_wash_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let panel_rects = self.fence_panel_rects();
        let pill_rects = self.code_pill_rects();
        self.fence_panel_pipeline
            .prepare(device, queue, width, height, &panel_rects);
        self.code_pill_pipeline
            .prepare(device, queue, width, height, &pill_rects);
    }

    /// Build + upload the selection / preedit, search-match, and horizontal-rule
    /// quads (each empty — so nothing lingers — when its feature is inactive).
    ///
    /// **THE ONE SWITCH POINT (routing intent, named for the future
    /// capabilities-as-data refactor):** `selection_rects()` (-> `range_rects`)
    /// is the SOLE geometry builder for a selection — every source (per-glyph
    /// runs within a line, full-width bands for a multi-line selection's
    /// middle lines, an empty-line's own stub, the trailing newline pad) is
    /// already folded into the ONE `rects` vector below, BEFORE the `one_bit`
    /// branch ever reads it. That branch is the single place a selection's
    /// geometry is handed to a per-world PAINT MECHANISM (the ordinary
    /// translucent `selection_pipeline` fill, or `selection_invert`'s true
    /// inverse-video) — a plain `if`/`else` today because there are only two
    /// mechanisms, but structured so a later `Theme::selection_style` (or
    /// `Theme::render_caps.selection_style` field, the capabilities-as-data
    /// read) only ever has to change what THIS branch reads, never how the
    /// rects themselves are built. Never duplicate `rects` per-mechanism —
    /// that is exactly the "different builder per bucket" shape that would
    /// let one geometry source quietly diverge (and disappear) on a future
    /// world.
    pub(crate) fn prepare_selection_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        // Build the selection highlight rectangles (one per visible line of the
        // region) plus any IME preedit underline. Empty when there is no
        // selection or preedit.
        let mut rects = self.selection_rects();
        rects.extend(self.preedit_rects());
        let one_bit =
            theme::active().render_caps.selection_style == theme::SelectionStyle::InverseVideo;

        // ORDINARY WORLDS: the translucent fill, unchanged.
        //
        // COPY PULSE: `prepare_pulsed` blends the stored base tint toward a
        // brighter peak by `(1.0 - copy_pulse_settle())` — settled (`1.0`, the
        // permanent value in every headless capture) is a byte-identical
        // short-circuit to the plain `prepare` this replaced, so a default
        // capture and every pre-existing selection render are unaffected.
        //
        // TRUE 1-BIT WORLDS: this pipeline uploads ZERO rects — the
        // `selection_invert` pipeline below takes over document selection
        // entirely (see its field doc) — so `selection_pipeline` draws
        // nothing there, never a stale white fill under the inverted text.
        let settle = self.copy_pulse_settle();
        let fill_rects: &[[f32; 4]] = if one_bit { &[] } else { &rects };
        self.selection_pipeline.prepare_pulsed(
            device,
            queue,
            width,
            height,
            fill_rects,
            copy_pulse_peak_srgba(),
            settle,
        );

        // Search-match highlights (separate instance/color — an ordinary
        // world's translucent fill, or THE ONE WAGTAIL HIGHLIGHT TEXTURE's
        // dither stipple on a one-bit world, per `search_match_rgba_bytes`/
        // `wagtail_dither_density`). Empty when search is closed so no stale
        // highlights linger.
        let mrects = if self.search_active {
            self.search_match_rects()
        } else {
            Vec::new()
        };
        self.match_pipeline
            .prepare(device, queue, width, height, &mrects);

        // TRUE 1-BIT WORLDS ONLY: the true inverse-video selection — see
        // `TextPipeline::selection_invert`'s field doc. Drawn AFTER text in
        // `draw_document_layers`; every other world uploads zero instances
        // here (parked, byte-identical).
        let invert_rects: &[[f32; 4]] = if one_bit { &rects } else { &[] };
        self.selection_invert
            .prepare(device, queue, width, height, invert_rects);
    }

    /// Shape + upload the markdown ORNAMENTS: the world's PER-SYNTAX break glyph
    /// CENTERED in the writing column on each thematic-break line, AND the depth-derived
    /// `•`/`◦`/`▪` BULLET left-aligned over each unordered list line's marker cell
    /// (reveal-on-cursor: neither is drawn on the caret's own line). Both shape from the
    /// bundled [`SYMBOL_FAMILY`] face in muted ink and share this one quiet renderer.
    /// The break glyph — `---`/`***`/`___`
    /// each draw a DIFFERENT ornament from the active [`theme::Ornaments`] set (the
    /// fine-press section break that REPLACES the old thin rule line, chosen by which
    /// syntax the author typed). Each glyph is shaped from the bundled
    /// [`SYMBOL_FAMILY`] face (the mono/display faces lack them) in the MUTED ink,
    /// at the active world's per-world [`theme::Theme::ornament_scale`] bump over the
    /// body size (the SAME factor `md_line_scale` grows the break ROW by) so a centered
    /// break reads with a touch more presence (quiet; amber stays the caret's). Also
    /// shapes the margin-hung blockquote pull-quote mark and the quiet per-fence
    /// LANGUAGE LABEL (a muted word like "rust" right-aligned on a recognized
    /// fence's opening line — [`TextPipeline::fence_lang_marks`], DATA-driven off
    /// the parsed info string). Uploads NO areas for a non-markdown buffer
    /// (`!md_enabled`), so a default capture stays byte-identical.
    pub(crate) fn prepare_ornaments(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let frame = ornaments::OrnamentFrame::shape(self);
        let (top, bottom) = match self.doc_clip_band(height as f32) {
            Some((top, bottom)) => (top as i32, bottom as i32),
            None => (0, height as i32),
        };
        let bounds = TextBounds {
            left: 0,
            top,
            right: width as i32,
            bottom,
        };
        let areas = frame.text_areas(self, bounds);
        self.ornament_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .map_err(|error| anyhow::anyhow!("glyphon ornament prepare failed: {error:?}"))?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn push_caption_scrim(&self, li: usize, out: &mut Vec<[f32; 4]>) {
        let zoom = self.metrics.zoom;
        let pad_x = CAPTION_SCRIM_PAD_X * zoom;
        let pad_y = CAPTION_SCRIM_PAD_Y * zoom;
        let text_left = self.text_left();
        let doc_top = self.doc_top();
        let band_h = self.metrics.line_height;
        for vr in self.visual_rows(li) {
            let (Some(&x0), Some(&x1)) = (vr.xs.get(vr.start_col), vr.xs.get(vr.end_col)) else {
                continue;
            };
            if x1 - x0 <= 0.5 {
                continue; // glyphless / fully-concealed row: nothing to back
            }
            let center_y = doc_top + vr.line_top + vr.line_height * 0.5;
            out.push([
                text_left + x0 - pad_x,
                center_y - band_h * 0.5 - pad_y,
                (x1 - x0) + pad_x * 2.0,
                band_h + pad_y * 2.0,
            ]);
        }
    }

    /// The deterministic per-image layout the last [`Self::rebuild_image_rows`]
    /// (via `compute_image_layout`) produced, for the capture `images` sidecar
    /// block + the next-phase draw. `revealed` is recomputed here against the
    /// CURRENT caret line (a pure caret move re-lays the image line's conceal but
    /// does not re-read image headers), so it never goes stale. Empty when inline
    /// images are off / non-markdown / on wasm.
    /// INLINE IMAGES — the GPU draw. Decodes each visible image (O(visible):
    /// off-SCREEN images are culled), uploads it via the
    /// [`image_cache`](crate::render::image_cache) (downscaled to the display
    /// width), and builds one textured quad per image (fit-to-column, centered in
    /// the reserved tall row `compute_image_layout` produced). A REVEALED image
    /// (the caret is on its source line) is still drawn — DIMMED and UNMOVED (the
    /// caption model: the source reveals centred OVER it, over a soft scrim band) —
    /// not culled. Plus a calm rounded
    /// PLACEHOLDER (opaque `base_200` quad + a muted filename / faint alt label) for
    /// every MISSING-file image. All three layers (image quads / placeholder quads /
    /// placeholder labels) park EMPTY when the feature is off / no visible images /
    /// non-markdown, so a default capture stays byte-identical.
    ///
    /// The tall rows themselves are reserved at reshape time (`compute_image_layout`
    /// → `image_heights`); the DECODE is synchronous here, so it never changes a
    /// reserved row height after the fact (the row was sized from the header dims,
    /// and the same file decodes to the same aspect) — no deferred-height
    /// invalidation is needed, the missing live-bug class the design flagged.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn prepare_images(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        use crate::render::image_cache::{ImageCache, ImageState};
        let report = self.images_report();

        // Prune the decode cache to the OPEN DOC's images (visible or not), keyed by
        // canonical path — buffer-swap-safe, and scrolling back to an image never
        // re-decodes (it stays cached while it's in this doc's set).
        let mut keep: std::collections::HashSet<std::path::PathBuf> =
            std::collections::HashSet::new();
        for im in &report {
            let resolved = self.resolve_image_path(&im.path);
            keep.insert(ImageCache::canonical_key(&resolved));
        }
        self.image_cache.retain_paths(&keep);

        let max_dim = device.limits().max_texture_dimension_2d;
        let zoom = self.metrics.zoom;
        let corner = IMAGE_CORNER_PX * zoom;
        let text_left = self.text_left();
        let wrap = self.text_wrap_width().max(1.0);

        // PASS A — cull + decode. `ready` holds the quad placements (dst rect + the
        // cache key to fetch the view in pass B, + the per-image opacity); `missing`
        // holds the placeholder placements (dst rect + filename + alt). Only an
        // OFF-SCREEN image is culled (its row is clipped to nothing anyway).
        //
        // CAPTION-STYLE REVEAL (re-decided 2026-07-09): an image on the caret's line
        // is still DRAWN — DIMMED (`IMAGE_REVEAL_DIM_ALPHA`) but UNMOVED, filling its
        // own `dh`-tall row from the row top exactly as off-cursor. The revealed
        // `![alt](path)` source shapes at body size and cosmic-text centres it
        // VERTICALLY within that same row (`build_line_attrs` keeps the row at `h` —
        // no grow, no reflow), so the source reads as a CAPTION over the dimmed
        // image; a scrim band (`scrim_bands` below) lifts its legibility. Off-cursor
        // the image draws at full opacity.
        // With WYSIWYG off there is no reveal model: a revealed (caret-on-line)
        // image PARKS exactly as before (its source shows unconcealed in the h-tall
        // row), keeping the wysiwyg=false off state byte-identical.
        let wysiwyg = crate::markdown::wysiwyg_on();
        struct Ready {
            dst: [f32; 4],
            alpha: f32,
            key: std::path::PathBuf,
        }
        struct Missing {
            dst: [f32; 4],
            path: String,
            alt: String,
        }
        let mut ready: Vec<Ready> = Vec::new();
        let mut missing: Vec<Missing> = Vec::new();
        let mut scrim_bands: Vec<[f32; 4]> = Vec::new();
        for im in &report {
            if im.revealed && (!wysiwyg || !self.image_row_reserved(im.line)) {
                continue;
            }
            let dw = im.display_w.max(1.0);
            let dh = im.display_h.max(1.0);
            let row_top = self.image_draw_top(im.line);
            // ITEM 82: cull on the row's OWN box (top..top+dh), not a fixed small
            // margin around its top alone — a tall image's top can scroll well
            // past the margin while its bottom is still on-screen, and a top-only
            // test would drop it (a hard "blank collapse" mid-scroll instead of
            // the progressive clip a normal document row gets for free).
            if !self.row_box_visible(row_top, dh) {
                continue;
            }
            let alpha = if im.revealed && !im.missing {
                IMAGE_REVEAL_DIM_ALPHA
            } else {
                1.0
            };
            let left = text_left + (wrap - dw).max(0.0) * 0.5;
            let dst = [left, row_top, dw, dh];
            if im.revealed && !im.missing && wysiwyg {
                self.push_caption_scrim(im.line, &mut scrim_bands);
            }
            if im.missing {
                missing.push(Missing {
                    dst,
                    path: im.path.clone(),
                    alt: im.alt.clone(),
                });
                continue;
            }
            let resolved = self.resolve_image_path(&im.path);
            let key = ImageCache::canonical_key(&resolved);
            match self
                .image_cache
                .ensure(device, queue, &resolved, dw, max_dim)
            {
                ImageState::Ready { .. } => ready.push(Ready { dst, alpha, key }),
                ImageState::Missing => missing.push(Missing {
                    dst,
                    path: im.path.clone(),
                    alt: im.alt.clone(),
                }),
            }
        }

        // PASS B — build the image quads from the cached views (a distinct IMMUTABLE
        // cache borrow, disjoint from the mutable `image_pipeline` field).
        {
            let cache = &self.image_cache;
            let pipeline = &mut self.image_pipeline;
            let placed: Vec<crate::image_pipeline::PlacedImage> = ready
                .iter()
                .filter_map(|r| {
                    cache
                        .view(&r.key)
                        .map(|view| crate::image_pipeline::PlacedImage {
                            dst: r.dst,
                            alpha: r.alpha,
                            corner,
                            view,
                        })
                })
                .collect();
            pipeline.prepare(device, queue, width, height, &placed);
        }

        let placeholder_rects: Vec<[f32; 4]> = missing.iter().map(|m| m.dst).collect();
        self.image_placeholder_pipeline
            .prepare(device, queue, width, height, &placeholder_rects);

        self.image_scrim_pipeline
            .prepare(device, queue, width, height, &scrim_bands);

        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        let gm = GlyphMetrics::new(m.font_size * label, m.line_height * label);
        let line_h = m.line_height * label;
        let muted = theme::muted().to_glyphon();
        let faint = theme::faint().to_glyphon();
        let center = Some(glyphon::cosmic_text::Align::Center);
        let name_attrs = self.doc_attrs().color(muted);
        let alt_attrs = self.doc_attrs().color(faint);
        let mut buffers: Vec<(GlyphBuffer, f32, f32, glyphon::Color)> = Vec::new();
        for mss in &missing {
            let filename = std::path::Path::new(&mss.path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(mss.path.as_str());
            let box_w = mss.dst[2].max(1.0);
            let box_left = mss.dst[0];
            let box_top = mss.dst[1];
            let box_h = mss.dst[3];
            let two = !mss.alt.trim().is_empty();
            let block_h = if two { line_h * 2.0 } else { line_h };
            let start_y = box_top + (box_h - block_h).max(0.0) * 0.5;
            let mut name_buf = GlyphBuffer::new(&mut self.font_system, gm);
            name_buf.set_size(&mut self.font_system, Some(box_w), Some(line_h));
            name_buf.set_text(
                &mut self.font_system,
                filename,
                &name_attrs,
                Shaping::Advanced,
                center,
            );
            name_buf.shape_until_scroll(&mut self.font_system, false);
            buffers.push((name_buf, box_left, start_y, muted));
            if two {
                let mut alt_buf = GlyphBuffer::new(&mut self.font_system, gm);
                alt_buf.set_size(&mut self.font_system, Some(box_w), Some(line_h));
                alt_buf.set_text(
                    &mut self.font_system,
                    mss.alt.trim(),
                    &alt_attrs,
                    Shaping::Advanced,
                    center,
                );
                alt_buf.shape_until_scroll(&mut self.font_system, false);
                buffers.push((alt_buf, box_left, start_y + line_h, faint));
            }
        }
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let areas: Vec<TextArea> = buffers
            .iter()
            .map(|(buf, left, top, color)| TextArea {
                buffer: buf,
                left: *left,
                top: *top,
                scale: 1.0,
                bounds,
                default_color: *color,
                custom_glyphs: &[],
            })
            .collect();
        self.image_placeholder_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon image placeholder prepare failed: {e:?}"))?;
        Ok(())
    }

    /// INLINE IMAGES on wasm: the feature is native-only (no decode cache), so all
    /// three layers park EMPTY — byte-identical to the feature being off.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn prepare_images(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.image_pipeline.clear();
        self.image_placeholder_pipeline
            .prepare(device, queue, width, height, &[]);
        self.image_scrim_pipeline
            .prepare(device, queue, width, height, &[]);
        self.image_placeholder_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                Vec::new(),
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon image placeholder prepare failed: {e:?}"))?;
        Ok(())
    }

    /// SELECTION REVEAL (regression fix, item 16 follow-up): `revealed` is true
    /// when the CURRENT caret line OR the active selection touches the image's
    /// own span — the SAME [`selection_touches`] overlap test
    /// [`super::spans::wysiwyg_reveals`] uses for the raw markup, never
    /// re-derived — so a selected image line PARKS (dims + scrims, or for a
    /// mixed caption line, skips the draw) exactly like a caret-revealed one,
    /// instead of drawing full-brightness under revealed source text.
    pub fn images_report(&self) -> Vec<crate::render::ImageReport> {
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
        self.image_report
            .borrow()
            .iter()
            .cloned()
            .map(|mut r| {
                r.revealed = r.line == self.cursor_line
                    || selection_touches(selection_touch.as_ref(), &(r.range.0..r.range.1));
                r
            })
            .collect()
    }

    pub(crate) fn prepare_chrome_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.begin_float_panel_frame();
        // CARET-STYLE PICKER: the floating preview PANEL below the picker card (the
        // sample line with the choreographed demo caret). Parked (nothing drawn) unless
        // that picker is open, so every other frame stays byte-identical. Built on the
        // reusable `prepare_float_panel` primitive. Prepared BEFORE the overlay so the
        // SPELL contextual panel (which reuses the SAME float quads for its own
        // elevation, see `prepare_overlay`) sets them LAST and isn't parked here — the
        // caret picker and the spell panel are mutually exclusive, so only one ever
        // owns the float quads on a frame. (THE FORMAT POPOVER shares these quads too,
        // prepared later at the chrome tail — its own guard, not call order, is what
        // keeps it from racing this one; see `prepare_popover`'s doc.)
        self.prepare_caret_preview_panel(device, queue, width, height)?;

        if self.overlay_active {
            self.prepare_overlay(device, queue, width, height)?;
        } else if self.search_active {
            self.prepare_panel(device, queue, width, height)?;
            self.overlay_rows.prepare(device, queue, width, height, &[]);
            self.overlay_bars.prepare(device, queue, width, height, &[]);
        } else {
            self.park_overlay(device, queue, width, height)?;
        }
        self.prepare_gutter(device, queue, width, height)?;
        self.prepare_outline(device, queue, width, height)?;
        self.prepare_debug(device, queue, width, height)?;
        self.prepare_notice(device, queue, width, height)?;
        self.prepare_page_drag_readout(device, queue, width, height)?;
        self.prepare_zoom_readout(device, queue, width, height)?;
        self.prepare_hud(device, queue, width, height)?;
        self.prepare_whichkey(device, queue, width, height)?;
        // THE FORMAT POPOVER (reveal-on-select format toolbar): its active-button
        // wash + labels + (shared) float elevation, anchored over the selection.
        // Parked (nothing drawn) unless a mouse selection summoned it (or the
        // `AWL_POPOVER` capture probe forced it), so a default capture is
        // byte-identical. Its own `overlay_active`/`search_active` guard (see
        // `prepare_popover`'s doc) — not call order — is what keeps a real spell
        // popup / caret preview / search card safe from this call, so it can sit
        // anywhere in this sequence; it stays here (last, before the menu bar)
        // to minimize churn from its pre-existing position.
        self.prepare_popover(device, queue, width, height)?;
        self.flush_float_panel(device, queue, width, height);
        self.prepare_menubar(device, queue, width, height)?;
        Ok(())
    }

    /// Build + upload the wavy spell-check underlines (one per misspelled span),
    /// laid out on the same advance-aware glyph-x grid as the selection rects.
    pub(crate) fn prepare_spell_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        // Build the wavy spell-check underlines (one per misspelled span) using
        // the SAME advance-aware glyph-x layout as the selection rects, so each
        // squiggle lands under its word's real glyph cells at any zoom/scroll.
        let squiggles = self.spell_squiggles();
        self.spell_pipeline
            .prepare(device, queue, width, height, &squiggles);
    }

    /// Build + upload the STRAIGHT muted WRITING-NIT underlines (one per nit span),
    /// on the SAME advance-aware glyph-x grid as the spell squiggles + selection
    /// rects. Empty (nothing uploaded, so nothing drawn) when the highlighter is
    /// toggled off, so a nits-off frame is byte-identical to no nits at all.
    pub(crate) fn prepare_nit_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let underlines = self.nit_underlines();
        self.nit_pipeline
            .prepare(device, queue, width, height, &underlines);
    }

    pub(crate) fn prepare_strike_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let lines = self.strike_lines();
        self.strike_pipeline
            .prepare(device, queue, width, height, &lines);
    }

    pub(crate) fn prepare_link_underline_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let lines = self.link_underlines();
        self.link_underline_pipeline
            .prepare(device, queue, width, height, &lines);
    }
}
