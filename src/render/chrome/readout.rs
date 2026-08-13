//! CORNER READOUTS chrome — the ONE shared corner-label body
//! ([`TextPipeline::prepare_corner_label`], `pub(super)` so the debug panel in
//! [`super::debug_text`] rides it too) plus the bottom-right word-count / reading-time
//! readout, the calm notice on its plated line at the top of the writing column,
//! and the page-width drag readout that ride it. See [`super`].

use super::*;

mod toast;
use toast::notice_plate_inks;
#[cfg(test)]
pub(in crate::render) use toast::{TOAST_COLLISION_GAP, TOAST_SAFE_INSET};

/// The (left, top) device-px origin of a non-empty corner label, given its widest
/// shaped run width `text_w`, its `line_height`, the canvas `width`/`height`, the
/// writing column's `col_left`/`col_width`, and a vertical top-anchor offset — the
/// bare WEB/LINUX MENU BAR reserve ([`TextPipeline::menubar_reserve`]) for
/// [`CornerAnchor::TopRight`], or [`TextPipeline::text_origin_top`] (that reserve WITH
/// the document's own scaled `TEXT_TOP` folded in) for [`CornerAnchor::TopCenter`] —
/// this fn is pure (no `Metrics`, so no scale to multiply by) and never re-derives
/// either composition itself. The ONE owner of the corner-anchor placement math —
/// split out of [`TextPipeline::prepare_corner_label`] so each anchor is
/// unit-testable without a GPU (the empty-text off-screen park stays in the caller).
/// An 8px inset from the canvas edges for the docked corners; a small clamped float
/// for the at-pointer readout. Only the TOP-anchored arms read this offset — a shown
/// bar pushes them down by exactly its own height (merge, don't align: one owner,
/// never a second offset convention). The bottom / pointer-anchored arms are
/// unaffected (a bar at the TOP of the canvas never reaches them).
// The readout is assembled from explicit theme and geometry facts at one render boundary.
#[allow(clippy::too_many_arguments)]
pub(in crate::render) fn corner_origin(
    anchor: CornerAnchor,
    text_w: f32,
    line_height: f32,
    width: f32,
    height: f32,
    col_left: f32,
    col_width: f32,
    menubar_reserve: f32,
) -> (f32, f32) {
    let [left, top, _, _] = crate::render::plan::plan_corner_label(
        anchor,
        text_w,
        line_height,
        width,
        height,
        col_left,
        col_width,
        menubar_reserve,
        CANVAS_INSET.0,
    );
    (left, top)
}

/// The inset every docked corner label keeps from a canvas edge, and the same one
/// the notice plate is clamped to — one named value instead of the literal `8.0`
/// repeated per anchor arm.
///
/// `Physical` records what this already WAS rather than what it should be. The bare
/// `8.0`s this replaces were device pixels, and naming them is what first brought
/// them under the declaration law at all — that law reads authored `const`s and not
/// inline literals, so the value was invisible to it while it was repeated. Promoting
/// it to `Logical` would double the inset on a Retina display, which is almost
/// certainly right and is a deliberate appearance change owing a 1x/2x sweep across
/// every anchor arm; it is deferred rather than made silently here.
///
/// **This is now the ONE owner of chrome's bottom-edge inset, across six call
/// sites:** the three corner anchors here, `gutter.rs`'s four bottom-anchored
/// rects, and `outline.rs`'s reserve band — each of which already claimed textual
/// identity with this value in a comment before anything enforced it. So a future
/// promotion sweep moves all of them in one pass instead of rediscovering them.
pub(in crate::render) const CANVAS_INSET: Physical = Physical(8.0);

/// The CALM NOTICE plate's breathing room around its sentence, derived from the
/// notice's own LABEL line height rather than pinned in pixels — so it is the same
/// optical padding at every zoom and DPI, which a device-pixel constant is not
/// (chrome padding once shipped at half its tuned size on every Retina display for
/// exactly that reason). Horizontal is generous, vertical tight: the plate should
/// read as a line of chrome, not as a box.
pub(in crate::render) fn notice_plate_padding(label_line_height: f32) -> (f32, f32) {
    (label_line_height * 0.6, label_line_height * 0.22)
}

impl TextPipeline {
    /// Shape one quiet corner label into `buffer` and `prepare` it into `renderer`,
    /// parking it off-screen when `text` is empty. This is the shared body behind the
    /// bottom-right word-count readout and the top-left DEBUG panel — each was a
    /// ~95%-identical copy differing only by the (renderer, buffer) pair, the text,
    /// the corner [`CornerAnchor`], and (for the debug panel) the metrics + row count.
    ///
    /// It takes `renderer` + `buffer` (and the four shared glyphon resources) as
    /// EXPLICIT `&mut` params rather than `&mut self`: the callers pass distinct
    /// fields, so a `&mut self` method couldn't also hand it `&mut
    /// self.wordcount_renderer`. `col_left` / `col_width` are the writing column's
    /// already-resolved geometry (so this stays free of `self`); `col_width` is only
    /// consulted for the right-aligned anchor. `gm` sets the buffer's glyph metrics (so
    /// a compact panel can ride a smaller size) and `rows` reserves that many
    /// line-heights of height so a STACKED multi-line label (the debug panel) shapes
    /// without clipping; a single-line label passes `rows == 1.0`. `align` is
    /// `Some(Align::Right)` ONLY for the multi-line debug panel — it re-shapes the block
    /// flush-right so its ragged shorter lines all end at the block's right edge; `None`
    /// (every single-line readout) keeps the default left alignment, byte-identical.
    /// `menubar_reserve` is forwarded verbatim to [`corner_origin`] (`0.0` unless the
    /// bar is shown — see that fn's doc for why only the TOP-anchored callers, the
    /// debug panel today, actually move).
    ///
    /// RETURNS the rect it actually placed — `Some([left, top, text_w, box_h])`, in
    /// device px — or `None` when the label was parked off-screen for empty text.
    /// A caller that needs to draw something UNDER its own label (the calm notice's
    /// plate) reads the placement from here rather than recomputing it: the shaped
    /// width is measured inside this body, so a second copy of the arithmetic would
    /// be a second owner of where the label is, and the plate could drift off the
    /// glyphs it backs.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_corner_label(
        renderer: &mut TextRenderer,
        buffer: &mut GlyphBuffer,
        font_system: &mut FontSystem,
        atlas: &mut TextAtlas,
        viewport: &Viewport,
        swash_cache: &mut SwashCache,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        gm: GlyphMetrics,
        rows: f32,
        col_left: f32,
        col_width: f32,
        text: &str,
        anchor: CornerAnchor,
        align: Option<glyphon::cosmic_text::Align>,
        label: &str,
        menubar_reserve: f32,
        ink: glyphon::Color,
    ) -> anyhow::Result<Option<[f32; 4]>> {
        let line_height = gm.line_height;
        let box_h = line_height * rows.max(1.0);
        buffer.set_metrics(font_system, gm);
        buffer.set_size(font_system, Some(width as f32), Some(box_h));
        buffer.set_text(
            font_system,
            text,
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(font_system, false);
        // Empty text parks the label off-screen so nothing draws (and a default
        // capture stays byte-identical). Otherwise measure the widest shaped run once
        // and hand the placement to the pure `corner_origin` owner.
        let (placed, left, top) = if text.is_empty() {
            (None, 0.0, -1000.0)
        } else {
            let mut text_w = 0.0_f32;
            for run in buffer.layout_runs() {
                text_w = text_w.max(run.line_w);
            }
            // FLUSH-RIGHT (the multi-line DEBUG panel): collapse the shaping box to the
            // widest run, right-align every line within it, and re-shape — so each line's
            // right edge lands at the block's right edge (positioned by `corner_origin`
            // below at `width − text_w − 8`), not ragged. `None` (the single-line
            // word-count / notice / drag readouts) is a NO-OP: they stay left-aligned and
            // byte-identical.
            if align.is_some() {
                buffer.set_wrap(font_system, Wrap::None);
                for line in buffer.lines.iter_mut() {
                    line.set_align(align);
                }
                buffer.set_size(font_system, Some(text_w), Some(box_h));
                buffer.shape_until_scroll(font_system, false);
            }
            let (left, top) = corner_origin(
                anchor,
                text_w,
                line_height,
                width as f32,
                height as f32,
                col_left,
                col_width,
                menubar_reserve,
            );
            (Some([left, top, text_w, box_h]), left, top)
        };
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let area = TextArea {
            buffer,
            left,
            top,
            scale: 1.0,
            bounds,
            default_color: ink,
            custom_glyphs: &[],
        };
        renderer
            .prepare(
                device,
                queue,
                font_system,
                atlas,
                viewport,
                [area],
                swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon {label} prepare failed: {e:?}"))?;
        Ok(placed)
    }

    /// The QUIET readout for a MARKDOWN buffer: `Some((words, reading_minutes))` when
    /// the buffer is markdown and has at least one word, else `None` (nothing drawn).
    /// Exposed so the capture sidecar can report exactly what the readout shows.
    ///
    /// Derived by [`crate::card::figures::readout_figures`] — the ONE owner the
    /// semantic fold reads too — over [`figure_source`], never the shaped page.
    /// The third element is the unit that number is in — words for a script
    /// that spaces them, characters for one that doesn't (`CountUnit`).
    pub fn readout_report(&self) -> Option<(usize, usize, crate::card::figures::CountUnit)> {
        crate::card::figures::readout_figures(&self.figure_source().0, self.md_enabled)
    }

    /// WHAT THE CALM NOTICE DRAWS THIS FRAME — `Some((text, kind))`, or `None`
    /// when nothing is drawn. Exposed so the capture sidecar reports exactly what
    /// the notice chrome shapes.
    ///
    /// It reads what [`Self::prepare_notice`] last SHAPED — not the raw `notice`
    /// field — so a frame that yields the notice (a relocated read-only
    /// comparison) reports `None`, and a sentence elided to a narrow column
    /// reports the elided form. Either way the block cannot claim a message the
    /// reader will not find in the PNG.
    pub fn notice_report(&self) -> Option<(String, crate::actions::NoticeKind)> {
        let text = self.notice_drawn.clone();
        (!text.is_empty()).then_some((text, self.notice_kind))
    }

    /// The widest shaped run in the notice's OWN glyph buffer, after the last
    /// `prepare_notice`. Test-only: it is how the elision law asks the shaper —
    /// rather than a mean-glyph-width estimate — whether the sentence really fits
    /// the plate it was measured against.
    #[cfg(test)]
    pub(crate) fn notice_shaped_width_probe(&self) -> f32 {
        self.notice_buffer
            .layout_runs()
            .fold(0.0_f32, |w, run| w.max(run.line_w))
    }

    /// The readout string for the bottom-right corner, e.g. `"240 words · 2 min"`.
    /// Empty when there is nothing to show (non-markdown or wordless).
    ///
    /// The persistent bottom-right readout is no longer drawn; this text-feeder
    /// lives on for the calm corner notice. The held HUD's own WORD COUNT
    /// figure comes from `crate::card::figures`, not from here.
    pub(in crate::render) fn wordcount_text(&self) -> String {
        crate::card::figures::words_readout(&self.figure_source().0, self.md_enabled)
    }

    /// Shape + upload the quiet word-count / reading-time readout. Drawn DIM and
    /// RIGHT-aligned to the writing column's right edge, on the bottom row. Empty text
    /// parks it off-screen (markdown gate / empty doc), so a non-markdown buffer draws
    /// nothing and stays byte-identical.
    ///
    /// RETAINED (unused) for phase 2: the persistent readout was removed from the
    /// chrome layer (it moves into the held HUD); this shaper stays for that reuse.
    #[allow(dead_code)]
    pub(in crate::render) fn prepare_wordcount(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let text = self.wordcount_readout_text();
        let (gm, col_left, col_width) = (
            self.metrics.glyph_metrics(),
            self.column_left(),
            self.column_width(),
        );
        let menubar_reserve = self.menubar_reserve();
        Self::prepare_corner_label(
            &mut self.wordcount_renderer,
            &mut self.wordcount_buffer,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            &mut self.swash_cache,
            device,
            queue,
            width,
            height,
            gm,
            1.0,
            col_left,
            col_width,
            &text,
            CornerAnchor::BottomRight,
            None,
            "wordcount",
            // BottomRight never reads the bar reserve (a top strip never reaches the
            // bottom row) — passed uniformly anyway so every `prepare_corner_label`
            // caller supplies the SAME current value, never a second convention.
            menubar_reserve,
            theme::muted().to_glyphon(),
        )?;
        Ok(())
    }

    /// Shape + upload the CALM NOTICE: one LABEL-sized line in `base_content`, on
    /// its own plated line at the TOP of the writing column, where the page
    /// begins. An EMPTY notice parks the label off-screen AND clears the plate, so
    /// a capture that raises no notice stays byte-identical.
    ///
    /// # Why it is here, and why it has a plate
    ///
    /// It used to sit muted at the BOTTOM-CENTRE of the column with no plate, and
    /// the measurement of that arrangement is why it moved: a `saved` toast was 221
    /// changed pixels — 0.023% of the canvas — 14 px from the bottom edge, wedged
    /// into the interline gap between the last two rows of prose, in the same ink
    /// family and 0.8× the size of the text it sat among. Its contrast against the
    /// page was 4.84:1, so a legibility floor PASSED on it. Nothing about it said
    /// "this is not part of your document", and nothing put it where a reader looks.
    ///
    /// The plate is not decoration and not a generic elevation effect (DESIGN §5
    /// rejects those): awl's margins are ~16 px at the default page width, so there
    /// is no empty ground anywhere on the canvas that can hold a sentence. A line
    /// of chrome inside the column therefore lands on prose, and a value-stepped
    /// plane is what makes that legible — the "shared model genuinely incapable"
    /// case DESIGN §2 asks to be shown before a new mechanism is added. It is one
    /// quad off the shared neutral ramp: no shadow, no rim, no hue.
    ///
    /// # The two kinds
    ///
    /// A notice's kind is a LIFETIME ([`crate::actions::NoticeKind`]) and the
    /// treatment expresses it by VALUE ONLY, which is DESIGN §5's own rule for
    /// "the same marker at less presence": a self-clearing `Toast` sits on
    /// `base_200` (raised), a HELD `Sticky` — the kind the writer has to act on,
    /// and the kind no lifetime can explain away — on `base_300` (foreground).
    /// Never a second decoration and never the `error` token, which is reserved
    /// for failure and destruction; "changed elsewhere" is neither.
    ///
    /// # Never clipped
    ///
    /// The sentence is elided to the column's own budget through the ONE shared
    /// pixel-truth door ([`rowlayout::fit_primary_end_to_px`]) before it is placed,
    /// so a narrow window shortens the notice instead of running it off the plate
    /// or off the canvas (DESIGN §8: never overlap, clip, or silently change the
    /// model).
    pub(in crate::render) fn prepare_notice(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let text = self.notice_readout_text();
        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        let gm = GlyphMetrics::new(m.font_size * label, m.line_height * label);
        let (col_left, col_width) = (self.column_left(), self.column_width());
        let (pad_x, pad_y) = notice_plate_padding(gm.line_height);
        // ELIDE FIRST, against the budget the plate will actually have. Measured
        // with the notice's OWN buffer at the notice's OWN metrics, so the fit is
        // decided on the same shaping the placement below reads back.
        let budget_px = (col_width - 2.0 * pad_x).max(0.0);
        let text = if text.is_empty() {
            text
        } else {
            let gm_probe = gm;
            let (font_system, buffer) = (&mut self.font_system, &mut self.notice_buffer);
            crate::render::rowlayout::fit_primary_end_to_px(&text, budget_px, |candidate| {
                buffer.set_metrics(font_system, gm_probe);
                buffer.set_size(font_system, Some(width as f32), Some(gm_probe.line_height));
                buffer.set_text(
                    font_system,
                    candidate,
                    &panel_attrs(),
                    Shaping::Advanced,
                    None,
                );
                buffer.shape_until_scroll(font_system, false);
                buffer
                    .layout_runs()
                    .fold(0.0_f32, |w, run| w.max(run.line_w))
            })
        };
        self.notice_drawn = text.clone();
        let (fill, rim, text_ink) = notice_plate_inks(self.notice_kind);
        let text_origin_top = self.text_origin_top();
        let toast_plan = self.notice_toast_plan(&text, gm, width, height, [pad_x, pad_y]);
        let anchor = toast_plan.map_or(CornerAnchor::TopCenter, |plan| {
            CornerAnchor::Absolute(plan.text[0], plan.text[1])
        });
        let placed = Self::prepare_corner_label(
            &mut self.notice_renderer,
            &mut self.notice_buffer,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            &mut self.swash_cache,
            device,
            queue,
            width,
            height,
            gm,
            1.0,
            col_left,
            col_width,
            &text,
            anchor,
            None,
            "notice",
            // Sticky notices retain the writing-column top; toast absolute
            // placement has already folded in the menu-bar reserve.
            if toast_plan.is_some() {
                0.0
            } else {
                text_origin_top
            },
            text_ink.to_glyphon(),
        )?;
        // THE PLATE, from the placement the label body just returned — never a
        // second copy of the arithmetic, so it cannot drift off the glyphs.
        // Clamped into the canvas with the same inset the docked corner labels
        // use, so a sentence wider than its column can still not run off an edge.
        let rects: Vec<[f32; 4]> = toast_plan
            .map(|plan| plan.plate)
            .or_else(|| {
                placed.map(|[left, top, text_w, box_h]| {
                    let x = (left - pad_x).max(CANVAS_INSET.0);
                    let w = (text_w + 2.0 * pad_x).min(width as f32 - 2.0 * CANVAS_INSET.0);
                    [
                        x.min(width as f32 - CANVAS_INSET.0 - w),
                        top - pad_y,
                        w,
                        box_h + 2.0 * pad_y,
                    ]
                })
            })
            .into_iter()
            .collect();
        // Both inks are resolved HERE, per frame, from the live theme — so neither
        // pipeline has an entry in `sync_theme_colors`: a capture (which never runs
        // that sync) and a live world switch both get the current ink from the same
        // read, instead of a baked seed one of them cannot refresh.
        // The rim is the fill's rect grown by one pixel on every side and drawn
        // UNDER it, so only a hairline shows — the same one-pixel outset
        // `set_float_quads` uses for the shared float border, rather than a second
        // convention for "a hairline around a surface".
        let rim_rects: Vec<[f32; 4]> = rects
            .iter()
            .map(|&[x, y, w, h]| [x - 1.0, y - 1.0, w + 2.0, h + 2.0])
            .collect();
        self.notice_rim.set_color(rim.rgba_bytes());
        self.notice_rim
            .prepare(device, queue, width, height, &rim_rects);
        self.notice_plate.set_color(fill.rgba_bytes());
        self.notice_plate
            .prepare(device, queue, width, height, &rects);
        Ok(())
    }

    /// Shape + upload the PAGE-WIDTH DRAG READOUT: a quiet muted char-count (e.g.
    /// "68") floating near the pointer while a page-column edge drag is in
    /// progress — Butterick's line-length rule made visible (value-only ink, NEVER
    /// amber — DESIGN §3). Mirrors [`Self::prepare_notice`]'s corner-label body but
    /// anchors AT the pointer ([`CornerAnchor::AtPoint`]) instead of a canvas
    /// corner. `page_drag_readout` is `None` (not dragging — the ONLY state a
    /// headless capture can ever see) parks it off-screen, so every capture stays
    /// byte-identical.
    pub(in crate::render) fn prepare_page_drag_readout(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let (text, anchor) = match self.page_drag_readout {
            Some((px, py, measure)) => (measure.to_string(), CornerAnchor::AtPoint(px, py)),
            None => (String::new(), CornerAnchor::AtPoint(0.0, 0.0)),
        };
        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        let gm = GlyphMetrics::new(m.font_size * label, m.line_height * label);
        let (col_left, col_width) = (self.column_left(), self.column_width());
        let menubar_reserve = self.menubar_reserve();
        Self::prepare_corner_label(
            &mut self.page_drag_renderer,
            &mut self.page_drag_buffer,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            &mut self.swash_cache,
            device,
            queue,
            width,
            height,
            gm,
            1.0,
            col_left,
            col_width,
            &text,
            anchor,
            None,
            "page_drag_readout",
            menubar_reserve,
            theme::muted().to_glyphon(),
        )?;
        Ok(())
    }

    /// Shape + upload the ZOOM READOUT: a quiet muted percentage (e.g. "120%")
    /// floating near the pointer while a zoom gesture (Cmd-± / Cmd-scroll) is IN
    /// FLIGHT — the current magnification made visible (value-only ink, NEVER amber
    /// — DESIGN §3). Mirrors [`Self::prepare_page_drag_readout`]'s corner-label body,
    /// anchoring AT the pointer ([`CornerAnchor::AtPoint`]). `zoom_readout` is `None`
    /// (settled — the ONLY state a headless capture sees by default) parks it
    /// off-screen, so every default capture stays byte-identical.
    ///
    /// GALLERY PROBE (capture-only): with `AWL_ZOOM_READOUT` set in the environment
    /// and no live readout, the label is synthesized at canvas-center from the
    /// pipeline's own zoom factor — the same shape the [`super::outline`]
    /// `AWL_OUTLINE_REVEAL` probe uses, so a gallery shot can witness the label
    /// without a live pointer.
    pub(in crate::render) fn prepare_zoom_readout(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let effective = self.zoom_readout.or_else(|| {
            std::env::var_os("AWL_ZOOM_READOUT")
                .map(|_| (width as f32 * 0.5, height as f32 * 0.5, self.metrics.zoom))
        });
        let (text, anchor) = match effective {
            Some((px, py, zoom)) => (
                format!("{}%", (zoom * 100.0).round() as i32),
                CornerAnchor::AtPoint(px, py),
            ),
            None => (String::new(), CornerAnchor::AtPoint(0.0, 0.0)),
        };
        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        let gm = GlyphMetrics::new(m.font_size * label, m.line_height * label);
        let (col_left, col_width) = (self.column_left(), self.column_width());
        let menubar_reserve = self.menubar_reserve();
        Self::prepare_corner_label(
            &mut self.zoom_readout_renderer,
            &mut self.zoom_readout_buffer,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            &mut self.swash_cache,
            device,
            queue,
            width,
            height,
            gm,
            1.0,
            col_left,
            col_width,
            &text,
            anchor,
            None,
            "zoom_readout",
            menubar_reserve,
            theme::muted().to_glyphon(),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
