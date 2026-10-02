//! Caret-style picker sample, using the document's shared raster bounds and
//! padded rectangular geometry. The demo spring owns its choreography.

use super::*;
use crate::render::caret_body::{InkBox, caret_block_body_dims, caret_block_corner};
use unicode_segmentation::UnicodeSegmentation;

/// The picker sample has one unwrapped visual row.
const DEMO_ROW_START: usize = 0;

impl TextPipeline {
    /// The caret-style preview PANEL's geometry — a two-line-tall floating box that
    /// hangs just BELOW the picker card, sharing its left edge + width. `None` unless
    /// the caret-style picker is open. Returns `(rect, text_left, row_center_y)`: the
    /// sample line sits vertically centred in the box, indented one pad.
    fn caret_preview_panel_rect(&self, width: u32) -> Option<([f32; 4], f32, f32)> {
        self.caret_preview?;
        let geom = self.overlay_geometry(width);
        let pad = 12.0;
        let gap = 10.0; // the breath between the picker card and the preview panel
        Some(crate::render::plan::plan_caret_preview_panel(
            [geom.card_x, geom.card_y, geom.card_w, geom.card_h],
            self.metrics.line_height,
            pad,
            gap,
        ))
    }

    /// Headless report for the caret-style preview panel: `(rect, sample_text,
    /// beat_index, silhouette_drawn)` when the caret-style picker is open, else
    /// `None`. The state machine (current beat + the preview buffer's sample text) is
    /// a deterministic function of the timeline, so a SETTLED capture reports the
    /// fixed end-state (`text == SAMPLE`); `silhouette_drawn` is whether the Morph
    /// glyph-silhouette pipeline actually painted THIS frame (settled on a real
    /// inhabited glyph in Morph mode) — always `false` for Block/I-beam or a
    /// glyphless/fast-motion Morph moment — so the fix (the preview demonstrating the
    /// real silhouette, not a permanent thin bar) is assertable from the sidecar
    /// without eyeballing pixels.
    pub fn caret_preview_panel_report(&self) -> Option<([f32; 4], String, usize, bool)> {
        let (rect, _, _) = self.caret_preview_panel_rect(self.window_w as u32)?;
        Some((
            rect,
            self.caret_demo.text(),
            self.caret_demo.beat_index(),
            self.caret_preview_glyph_pipeline.is_drawn(),
        ))
    }

    /// FIRST USE of the panel primitive: the caret-style picker's live preview PANEL.
    /// A floating card below the picker holds the sample line `watch me glide, jump,
    /// and morph`, on which the SELECTED caret look runs the choreographed demo
    /// ([`crate::caret::CaretDemo`]) — typing, gliding, jumping, deleting + gulping —
    /// driven by a scripted `apply_transition` timeline. Parked (nothing drawn) unless the
    /// caret-style picker is open. The choreography FEEL is live-only; a headless
    /// capture renders the deterministic SETTLED end-state (the fully-typed line at
    /// rest), pinned by `settle_caret_preview`.
    pub(in crate::render) fn prepare_caret_preview_panel(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let (look, rect, text_left, row_cy) =
            match (self.caret_preview, self.caret_preview_panel_rect(width)) {
                (Some(look), Some((rect, text_left, row_cy))) => (look, rect, text_left, row_cy),
                _ => {
                    // Picker closed: park the panel, the caret quad(s), and the sample text.
                    self.caret_preview_pipeline.prepare_empty();
                    self.caret_preview_glyph_pipeline.clear();
                    self.park_preview_text(device, queue, width, height)?;
                    return Ok(());
                }
            };
        self.caret_demo.mode = look;
        self.claim_float_panel(rect, FloatElevation::Rimmed, CardChamfer::default(), None);

        // Shape the sample line into the preview buffer (calm content ink, world face).
        //
        // RESPONSIVE SAMPLE: at a narrow window the panel (which shares the picker
        // card's width) can be too tight for the full sample line at BODY size —
        // the line then wrapped under its one-line box and the panel read broken /
        // mostly empty. Instead the WHOLE demo steps down in scale just enough for
        // the settled sample to fit on its one line (estimated at the mean advance,
        // conservative for every face, and from the FULL sample so the scale never
        // jitters mid-choreography). At any comfortable width `s == 1.0` and the
        // panel is byte-identical.
        let m = self.metrics;
        let avail = rect[2] - 24.0;
        // One extra advance of headroom: a mono face's real width EQUALS the mean
        // estimate, so an exact-fit scale would land fractionally over and wrap.
        let est = (crate::caret::SAMPLE.chars().count() + 1) as f32 * m.char_width;
        let s = if est > avail {
            (avail / est).max(0.5)
        } else {
            1.0
        };
        let ink = theme::base_content().to_glyphon();
        self.preview_buffer.set_metrics(
            &mut self.font_system,
            GlyphMetrics::new(m.font_size * s, m.line_height * s),
        );
        let text = self.caret_demo.text();
        self.preview_buffer
            .set_size(&mut self.font_system, Some(avail), Some(m.line_height * s));
        // The sample is ONE line by construction: never wrap it (a fractional
        // overshoot clips at the panel edge instead of folding under the box).
        self.preview_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        self.preview_buffer.set_text(
            &mut self.font_system,
            &text,
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        self.preview_buffer
            .shape_until_scroll(&mut self.font_system, false);

        let text_top = row_cy - 0.5 * m.line_height * s;
        let (anchor_char, target) =
            self.preview_anchor_target(look, &text, text_left, row_cy, text_top);
        let first = self
            .caret_demo
            .set_metrics(m.char_width * s, m.line_height * s);
        if first {
            // First frame: SNAP the caret onto the line (no glide-in from nowhere).
            self.caret_demo.anim.jump_to(target.x, target.y);
        } else if let Some(tick) = self.caret_demo.take_tick() {
            // Glide to the freshly-shaped cursor X on a real move, then arm the flinch
            // the fired beat earned (typing impact / delete squash / kill gulp / recoil)
            // — the SAME juice the document caret gets through `apply_transition`'s effects.
            use crate::actions::Effect;
            let is_edit = matches!(
                tick.effect,
                Effect::TypeImpact | Effect::DeleteSquash | Effect::Gulp
            );
            if tick.moved {
                self.caret_demo.anim.set_edit_move(is_edit);
                self.caret_demo.anim.nav_to(target.x, target.y);
            }
            match tick.effect {
                Effect::TypeImpact => self.caret_demo.anim.type_impact(),
                Effect::DeleteSquash => self.caret_demo.anim.delete_squash(),
                Effect::Gulp => self.caret_demo.anim.gulp(),
                Effect::Recoil(dir) => self.caret_demo.anim.recoil(dir),
                _ => {}
            }
        }

        // Upload the sample text (top = row centre minus half a line height).
        let text_top = row_cy - 0.5 * m.line_height * s;
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        self.upload_preview_text(device, queue, text_left, text_top, ink, bounds)?;

        // Emit the preview caret quad(s) from the demo spring, in the highlighted
        // look — the SAME spring/morph machinery as the document caret, at the
        // demo's scale, over the SAME shaped sample text just uploaded (so a Morph
        // silhouette's glyph masks match the glyphs actually on screen).
        self.emit_preview_caret(
            device,
            queue,
            width,
            height,
            look,
            s,
            anchor_char,
            &text,
            text_top,
        );
        Ok(())
    }

    fn upload_preview_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text_left: f32,
        text_top: f32,
        ink: glyphon::Color,
        bounds: TextBounds,
    ) -> anyhow::Result<()> {
        let area = TextArea {
            buffer: &self.preview_buffer,
            left: text_left,
            top: text_top,
            scale: 1.0,
            bounds,
            default_color: ink,
            custom_glyphs: &[],
        };
        self.preview_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon preview prepare failed: {e:?}"))
    }

    /// The buffer-local pixel X (relative to the text left) of the caret at char index
    /// `cursor` on the shaped sample line: the shaped X of the glyph starting there, or
    /// the line's full width when the caret sits at the end. `0.0` for the empty line.
    fn preview_caret_local_x(&self, cursor: usize, text: &str) -> f32 {
        let byte = text
            .char_indices()
            .nth(cursor)
            .map(|(b, _)| b)
            .unwrap_or(text.len());
        let mut line_w = 0.0;
        for run in self.preview_buffer.layout_runs() {
            for g in run.glyphs.iter() {
                if g.start == byte {
                    return g.x;
                }
            }
            line_w = run.line_w;
        }
        line_w
    }

    /// Build + upload the preview caret quad(s) from the demo spring, in `look`,
    /// reusing the document caret's morph machinery: Block's settle-driven square ⇄
    /// streak, the slim I-beam comet, and — MORPH, SETTLED on a real inhabited glyph
    /// — the SAME glyph-SILHOUETTE the document caret paints, through this preview's
    /// own [`CaretGlyphPipeline`] (`caret_preview_glyph_pipeline`), so choosing Morph
    /// in the picker actually shows what it does: the sample letter recolored solid
    /// in the accent, not a permanent thin bar. Morph still DEFERS to the thin
    /// glyphless-anchor bar (a space / line start) or the plain streak (fast motion,
    /// settle factor below [`CARET_MORPH_SETTLE_SHOW`]), exactly as the document
    /// does (see [`TextPipeline::prepare_caret_layer`] for the shared three-way
    /// shape). The spring already sits in panel pixel coords (jumped/nav'd there
    /// above), so its centre is canvas-absolute.
    #[allow(clippy::too_many_arguments)]
    fn emit_preview_caret(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        look: CaretMode,
        demo_scale: f32,
        anchor_char: usize,
        text: &str,
        _text_top: f32,
    ) {
        let m = self.metrics;
        let s = self.caret_demo.anim.settle_factor();

        self.caret_preview_glyph_pipeline.clear();
        let ink = self.preview_anchor_ink(text, anchor_char);
        let scale = m.scale * demo_scale;
        let bar = look == CaretMode::Ibeam
            || (crate::caret::highlight_previous_character()
                && self.caret_demo.cursor_char() == DEMO_ROW_START);
        let (block_w, block_h, thin) = if bar {
            (
                m.px(IBEAM_W) * demo_scale,
                m.caret_h * demo_scale,
                m.px(IBEAM_W) * demo_scale,
            )
        } else if let Some(ink) = ink {
            let (w, h) = caret_block_body_dims(ink, scale);
            (w, h, m.caret_streak_h * demo_scale)
        } else {
            (
                m.char_width * demo_scale,
                m.caret_block_h * demo_scale,
                m.caret_streak_h * demo_scale,
            )
        };
        let anim = &self.caret_demo.anim;
        let speed = (anim.vel.x * anim.vel.x + anim.vel.y * anim.vel.y).sqrt();
        let streak_len = anim.streak_length(
            m.streak_len_for_speed(speed),
            m.caret_streak_max_len,
            m.caret_held_len,
        );
        let (center, half_along, half_across, axis) = anim.motion_geometry(
            block_w,
            block_h,
            thin,
            streak_len,
            m.caret_streak_gap,
            m.caret_trail_drop,
        );
        let corner = match look {
            CaretMode::Block => {
                m.px(STREAK_RADIUS)
                    + (caret_block_corner(block_w, block_h, scale) - m.px(STREAK_RADIUS)) * s
            }
            _ => m.px(STREAK_RADIUS).max(half_across.min(half_along) * 0.6),
        };
        let (w, h, corner) =
            self.caret_demo
                .anim
                .pop_scale_dims(half_along * 2.0, half_across * 2.0, corner);
        self.caret_preview_pipeline.prepare_axis(
            queue,
            width,
            height,
            CaretRect {
                center_x: center.x
                    + if bar {
                        0.0
                    } else {
                        ink.map_or(0.0, |ink| (ink.left + (ink.width - block_w) * 0.5) * s)
                    },
                center_y: center.y,
                rect_w: w,
                rect_h: h,
                corner,
            },
            1.0,
            [axis.0, axis.1],
        );
    }

    fn preview_anchor_target(
        &mut self,
        look: CaretMode,
        text: &str,
        text_left: f32,
        row_cy: f32,
        text_top: f32,
    ) -> (usize, crate::caret::Sample) {
        let cursor = self.caret_demo.cursor_char();
        let previous = look == CaretMode::Block && crate::caret::highlight_previous_character();
        let anchor_char = if previous && cursor > DEMO_ROW_START {
            let byte = text
                .char_indices()
                .nth(cursor)
                .map_or(text.len(), |(i, _)| i);
            text.grapheme_indices(true)
                .take_while(|(i, _)| *i < byte)
                .last()
                .map_or(cursor, |(i, _)| text[..i].chars().count())
        } else {
            cursor
        };
        let caret_x = text_left + self.preview_caret_local_x(anchor_char, text);
        let anchor_ink = self.preview_anchor_ink(text, anchor_char);
        let caret_y = if look == CaretMode::Block {
            anchor_ink.map_or(row_cy, |ink| {
                self.preview_baseline_y(text_top) - ink.top + ink.height * 0.5
            })
        } else {
            row_cy
        };
        (
            anchor_char,
            crate::caret::Sample {
                x: caret_x,
                y: caret_y,
            },
        )
    }

    fn preview_anchor_ink(&mut self, text: &str, col: usize) -> Option<InkBox> {
        let byte = text.char_indices().nth(col).map_or(text.len(), |(i, _)| i);
        let (start, grapheme) = text
            .grapheme_indices(true)
            .find(|(i, g)| byte >= *i && byte < i + g.len())?;
        let end = start + grapheme.len();
        let glyphs: Vec<_> = self
            .preview_buffer
            .layout_runs()
            .flat_map(|run| run.glyphs.iter())
            .filter(|g| g.start < end && g.end > start)
            .map(|g| {
                let p = g.physical((0.0, 0.0), 1.0);
                (g.start, g.end, p.cache_key, p.x, p.y)
            })
            .collect();
        if glyphs.iter().any(|(s, e, ..)| *s < start || *e > end) {
            return None;
        }
        let pen_x = self.preview_caret_local_x(col, text);
        super::super::caret::adaptive::shaped_ink_box(
            &glyphs,
            pen_x,
            &mut self.swash_cache,
            &mut self.font_system,
        )
    }

    /// The pixel BASELINE y (canvas-absolute) of the preview panel's one shaped
    /// sample line, given the text's TOP y (`text_top`, the same value passed to the
    /// panel's `TextArea`) — the preview-panel sibling of `caret_baseline_y`, reading
    /// the SAME cosmic-text `run.line_y` convention but over the throwaway
    /// `preview_buffer`'s single run instead of the document's. Falls back to the
    /// text top on an unshaped/empty line (never actually hit by the silhouette
    /// path, which only draws once a real glyph was found there).
    fn preview_baseline_y(&self, text_top: f32) -> f32 {
        self.preview_buffer
            .layout_runs()
            .next()
            .map(|r| text_top + r.line_y)
            .unwrap_or(text_top)
    }

    /// Park the preview sample-line text off-screen (an empty buffer), matching the
    /// corner-readout convention so a non-caret-picker frame stays byte-identical.
    fn park_preview_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let m = self.metrics;
        let content = theme::base_content().to_glyphon();
        self.preview_buffer
            .set_size(&mut self.font_system, Some(1.0), Some(m.line_height));
        self.preview_buffer.set_text(
            &mut self.font_system,
            "",
            &panel_attrs().color(content),
            Shaping::Advanced,
            None,
        );
        self.preview_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let area = TextArea {
            buffer: &self.preview_buffer,
            left: 0.0,
            top: -1000.0,
            scale: 1.0,
            bounds,
            default_color: content,
            custom_glyphs: &[],
        };
        self.preview_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon preview park failed: {e:?}"))?;
        Ok(())
    }
}
