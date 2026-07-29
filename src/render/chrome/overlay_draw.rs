//! Overlay card lifecycle and text upload. Geometry and hit testing live in
//! [`super::overlay`]; row surfaces and probes live in [`super::overlay_rows`].

use super::*;

impl TextPipeline {
    /// Shape + upload the SUMMONED navigation overlay for this frame: a tall
    /// BASE_300 card, a query line (with the one amber caret at its end), the
    /// candidate list (selected row highlighted with a surface VALUE band), all
    /// composited OVER the document. Reuses the panel card / caret / text
    /// renderer; the row highlight reuses the selection-quad pipeline. This is the
    /// functional-first card look — the organic visuals come later.
    pub(in crate::render) fn prepare_overlay(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.overlay_remetric();
        let ink = theme::base_content().to_glyphon();
        let muted = theme::muted().to_glyphon();
        let geom = self.overlay_geometry(width);
        let placard = self.overlay_shape_placard(&geom);
        let stipple = matches!(
            crate::render::effective_title_style(),
            theme::TitleStyle::Placard {
                ink: theme::PlacardInk::Stipple,
                ..
            }
        );
        let (placard, stipple_rects) = match placard {
            Some((x, y, _w, _h)) if stipple => (None, self.placard_stipple_rects((x, y))),
            other => (other, Vec::new()),
        };
        self.placard_stipple
            .prepare(device, queue, width, height, &stipple_rects);
        let selected_ink = super::overlay_selected_primary_ink();
        // ITEM 164 — RESOLVE THE VISUAL-SELECTION TRANSACTION ONCE, here, before
        // anything is shaped or emitted. This is the ONLY call that runs a band
        // animator for this frame; every consumer below reads the result, so the
        // band, both ink columns, the accessory plates and the sidecar give ONE
        // answer to "which row is selected" at every intermediate frame.
        let vis = self.resolve_visual_selection(&geom);
        let has_right = self.overlay_shape_text(&geom, ink, muted, selected_ink, &vis, true);
        self.overlay_upload_text(
            device, queue, width, height, &geom, has_right, ink, muted, placard,
        )?;
        self.overlay_draw_card(device, queue, width, height, &geom, &vis);
        self.overlay_place_caret(queue, width, height, &geom);
        Ok(())
    }

    /// PARK every overlay pipeline empty for a frame with NO active overlay —
    /// the park-when-off discipline `prepare_hud` / `park_preview_text` already
    /// follow, applied to the summoned card. Without this the overlay TEXT
    /// renderer keeps its last-open glyph buffer (a whole palette of rows), and
    /// the frosted-blur backdrop path (`render`'s blur branch, taken whenever the
    /// HUD is held) calls `draw_overlay_card` UNCONDITIONALLY — so a closed
    /// palette's sharp rows ghost over the HUD's frost. Parking the renderer +
    /// its quads here makes that draw HARMLESS regardless of HUD state: the frame
    /// AFTER an overlay closes carries zero stale overlay pixels.
    ///
    /// Zeroes the flat card, its 1-bit elevation companions, the selected-row band,
    /// and the theme-lens underline quads (`instance_count` → 0), parks the amber
    /// query caret, and re-prepares the text renderer from an EMPTY off-screen
    /// buffer (nothing to draw). The float-panel quads (shared with the spell
    /// popup) are parked earlier this frame by `prepare_caret_preview_panel`, so
    /// they are not touched here.
    pub(in crate::render) fn park_overlay(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.panel_card.prepare(device, queue, width, height, &[]);
        self.panel_shadow.prepare(device, queue, width, height, &[]);
        self.panel_border.prepare(device, queue, width, height, &[]);
        self.overlay_rows.prepare(device, queue, width, height, &[]);
        // PER-ITEM LIST SURFACES: the bar surfaces park empty too, so a closed
        // picker carries no stale bar quads into the next frame.
        self.overlay_bars.prepare(device, queue, width, height, &[]);
        // ARM B LIVING-BAND PROBE: the two-shape crossing quad parks empty too, so
        // a closed picker carries no stale crossing quad into the next frame.
        self.overlay_cross
            .prepare(device, queue, width, height, &[]);
        // ITEM 94: the range rail's track + thumb park empty too, so a closed
        // Settings menu carries no stale rail quads into the next frame.
        self.overlay_range_track
            .prepare(device, queue, width, height, &[]);
        self.overlay_range_thumb
            .prepare(device, queue, width, height, &[]);
        self.overlay_lens_underline
            .prepare(device, queue, width, height, &[]);
        // V6 P5: the Chips ghost pills park empty too, so a closed picker carries
        // no stale ghost-pill quads into the next frame.
        self.overlay_facet_ghost
            .prepare(device, queue, width, height, &[]);
        // The stipple placard: parked (zero instances) — the frame after a
        // stipple-world overlay closes carries zero stale wordmark pixels.
        self.placard_stipple
            .prepare(device, queue, width, height, &[]);
        // The Bars behind-the-bars placard pass: parked (no areas) so a closed
        // picker carries no stale wordmark into the next frame.
        self.placard_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                Vec::<TextArea>::new(),
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon placard park failed: {e:?}"))?;
        self.panel_caret.prepare_empty();
        let m = self.metrics;
        let ink = theme::base_content().to_glyphon();
        self.panel_buffer
            .set_size(&mut self.font_system, Some(1.0), Some(m.line_height));
        self.panel_buffer.set_text(
            &mut self.font_system,
            "",
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        self.panel_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let area = TextArea {
            buffer: &self.panel_buffer,
            left: 0.0,
            top: -1000.0,
            scale: 1.0,
            bounds,
            default_color: ink,
            custom_glyphs: &[],
        };
        self.panel_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon overlay park failed: {e:?}"))?;
        Ok(())
    }

    /// Re-metric BOTH shared overlay buffers to the current zoom so their glyph
    /// line-height matches the highlight/caret rects (which use m.line_height).
    /// Without this the buffer keeps its zoom-1.0 metrics and the selection
    /// highlight drifts one row off the text under zoom.
    ///
    /// The NAME buffer rides the overlay UI metrics ([`Self::overlay_metrics`] — a step
    /// below reading body so the picker reads as dense chrome, DESIGN §4); the right
    /// CHORD/time column rides the same UI LINE HEIGHT (so each chord stays on its
    /// name's row) but a smaller LABEL FONT SIZE on top — the type system's recessive
    /// rung (ink × size), so the secondary key-chord reads quieter than the name it
    /// annotates, not the same grey/size.
    pub(in crate::render) fn overlay_remetric(&mut self) {
        let m = self.metrics;
        let name_metrics = self.overlay_metrics();
        let lh = self.overlay_lh();
        self.panel_buffer
            .set_metrics(&mut self.font_system, name_metrics);
        let label = crate::markdown::type_scale::LABEL;
        self.panel_bind_buffer.set_metrics(
            &mut self.font_system,
            GlyphMetrics::new(
                m.font_size * crate::render::effective_overlay_scale() * label,
                lh,
            ),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn overlay_upload_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        has_right: bool,
        ink: glyphon::Color,
        muted: glyphon::Color,
        placard: Option<(f32, f32, f32, f32)>,
    ) -> anyhow::Result<()> {
        let text_left = geom.text_left;
        let text_top = geom.text_top;
        let bounds = TextBounds {
            left: text_left.max(0.0) as i32,
            top: 0,
            right: ((text_left + geom.text_w).min(width as f32)) as i32,
            bottom: height as i32,
        };
        let panel_area = TextArea {
            buffer: &self.panel_buffer,
            left: text_left,
            top: text_top,
            scale: 1.0,
            bounds,
            default_color: ink,
            custom_glyphs: &[],
        };
        // DESIGNER PIXEL-PASS FIX (2026-07-16) — the placard's DRAW SLOT depends on
        // the list style. Under `Bars` it must sit BEHIND the bar quads, so it rides
        // its own `placard_renderer` pass (run between the page/scrims and the bars in
        // `draw_overlay_card`); under `Pane` it stays FIRST-in-batch in
        // `panel_renderer` below (drawn behind the rows, over the opaque card — the
        // byte-identical historical slot). The dedicated pass is prepared empty
        // whenever it is not used, so a stale wordmark never lingers.
        let bars = matches!(
            crate::render::effective_list_style(),
            theme::ListStyle::Bars { .. }
        );
        let canvas_bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        {
            let placard_pass: Vec<TextArea> = match placard {
                Some((px, py, _pw, _ph)) if bars => vec![TextArea {
                    buffer: &self.placard_buffer,
                    left: px,
                    top: py,
                    scale: 1.0,
                    bounds: canvas_bounds,
                    default_color: ink,
                    custom_glyphs: &[],
                }],
                _ => Vec::new(),
            };
            // GRACEFUL DEGRADATION (AtlasFull fix, 2026-07-17): the quantized sizing
            // keeps the shared atlas bounded, but if it ever DOES fill (a huge display,
            // an exotic GPU with a small `max_texture_dimension_2d`), SKIP the placard
            // for this frame rather than erroring — prepare an empty pass so no stale
            // wordmark lingers, and let the next frame retry after the off-frame
            // `atlas.trim()` reclaims space. NEVER a print (the `gpu.rs` `prepare error:`
            // eprintln is the thing this silences for the placard's own overflow); a
            // non-AtlasFull error still propagates.
            let placard_prepare = self.placard_renderer.prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                placard_pass,
                &mut self.swash_cache,
            );
            match placard_prepare {
                Ok(()) => {}
                Err(glyphon::PrepareError::AtlasFull) => {
                    self.placard_renderer
                        .prepare(
                            device,
                            queue,
                            &mut self.font_system,
                            &mut self.atlas,
                            &self.viewport,
                            Vec::new(),
                            &mut self.swash_cache,
                        )
                        .map_err(|e| {
                            anyhow::anyhow!("glyphon placard skip-prepare failed: {e:?}")
                        })?;
                }
            }
        }
        let mut areas: Vec<TextArea> = Vec::new();
        // Whether the placard rides THIS (Pane) panel batch as the FIRST area — the
        // one entry whose giant glyphs could overflow the shared atlas. Tracked so the
        // graceful-degradation retry below can drop exactly it (see the prepare site).
        let mut placard_in_panel = false;
        if let Some((px, py, _pw, _ph)) = placard
            && !bars
        {
            areas.push(TextArea {
                buffer: &self.placard_buffer,
                left: px,
                top: py,
                scale: 1.0,
                bounds: canvas_bounds,
                default_color: ink,
                custom_glyphs: &[],
            });
            placard_in_panel = true;
        }
        let slant = crate::render::overlay_slant();
        match slant {
            None => {
                areas.push(panel_area);
            }
            Some(_) => {
                let lh = self.overlay_lh();
                let n_lines = if geom.theme {
                    geom.plan.len()
                } else {
                    geom.visible
                };
                let first_top =
                    overlay_row_top(geom.text_top, geom.header_rows, geom.header_gap, 0, lh);
                let clip = |top: f32, bottom: f32| TextBounds {
                    left: bounds.left,
                    top: top.max(0.0) as i32,
                    right: bounds.right,
                    bottom: (bottom.min(height as f32)) as i32,
                };
                areas.push(TextArea {
                    buffer: &self.panel_buffer,
                    left: text_left,
                    top: text_top,
                    scale: 1.0,
                    bounds: clip(0.0, first_top),
                    default_color: ink,
                    custom_glyphs: &[],
                });
                for k in 0..n_lines {
                    let row_top = first_top + k as f32 * lh;
                    areas.push(TextArea {
                        buffer: &self.panel_buffer,
                        left: text_left + self.overlay_slant_dx(k),
                        top: text_top,
                        scale: 1.0,
                        bounds: clip(row_top, row_top + lh),
                        default_color: ink,
                        custom_glyphs: &[],
                    });
                }
                let tail_top = first_top + n_lines as f32 * lh;
                areas.push(TextArea {
                    buffer: &self.panel_buffer,
                    left: text_left,
                    top: text_top,
                    scale: 1.0,
                    bounds: clip(tail_top, height as f32),
                    default_color: ink,
                    custom_glyphs: &[],
                });
            }
        }
        if has_right {
            areas.push(TextArea {
                buffer: &self.panel_bind_buffer,
                left: text_left,
                top: overlay_secondary_top(text_top, geom.header_gap),
                scale: 1.0,
                bounds,
                default_color: muted,
                custom_glyphs: &[],
            });
        }
        // GRACEFUL DEGRADATION (AtlasFull fix, 2026-07-17): under `Pane` the placard
        // rides this batch as `areas[0]` (drawn behind the rows). If its giant glyphs
        // ever overflow the shared atlas, re-prepare WITHOUT the placard (the rows are
        // the affordance that must survive; the watermark is the sacrificeable one), so
        // an AtlasFull never blanks the whole card. The next frame retries after the
        // off-frame `atlas.trim()`. A retry area-set is built only when the placard is
        // actually in this batch — every other run pays nothing and never re-prepares.
        // The placard-free fallback batch, built ONLY when the placard is in this batch
        // (every other run keeps `None` and never clones).
        let panel_retry: Option<Vec<TextArea>> =
            placard_in_panel.then(|| areas.iter().skip(1).cloned().collect());
        match self.panel_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            areas,
            &mut self.swash_cache,
        ) {
            Ok(()) => {}
            Err(glyphon::PrepareError::AtlasFull) => match panel_retry {
                Some(retry) => self
                    .panel_renderer
                    .prepare(
                        device,
                        queue,
                        &mut self.font_system,
                        &mut self.atlas,
                        &self.viewport,
                        retry,
                        &mut self.swash_cache,
                    )
                    .map_err(|e| {
                        anyhow::anyhow!("glyphon overlay skip-placard prepare failed: {e:?}")
                    })?,
                None => return Err(anyhow::anyhow!("glyphon overlay prepare failed: AtlasFull")),
            },
        }
        Ok(())
    }

    fn overlay_place_caret(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
    ) {
        if geom.header_rows == 0 {
            self.panel_caret.prepare_empty();
            return;
        }
        let m = self.metrics;
        let sigil = "› ";
        let title_prefix = self.overlay_title_prefix(geom);
        let prefix_len = if title_prefix.is_empty() {
            sigil.len()
        } else {
            title_prefix.len()
        };
        let caret_char = self
            .overlay_query_caret
            .min(self.overlay_query.chars().count());
        let target_byte = prefix_len + field_caret_byte(&self.overlay_query, caret_char);
        let first_run = self.panel_buffer.layout_runs().next();
        let caret_x = geom.text_left
            + first_run
                .as_ref()
                .and_then(|r| {
                    r.glyphs
                        .iter()
                        .find(|g| g.start == target_byte)
                        .map(|g| g.x)
                })
                .or_else(|| first_run.as_ref().map(|r| r.line_w))
                .unwrap_or_else(|| {
                    m.char_width
                        * (sigil.chars().count() + self.overlay_query.chars().count()) as f32
                });
        let caret_h = m.caret_h * 0.8 * OVERLAY_UI_SCALE;
        let caret_cx = caret_x + m.caret_w * 0.5;
        let query_line_height = self
            .panel_buffer
            .layout_runs()
            .next()
            .map(|r| r.line_height)
            .unwrap_or_else(|| self.overlay_lh());
        let caret_cy = overlay_query_center(geom.text_top, query_line_height);
        self.panel_caret.prepare(
            queue,
            width,
            height,
            caret_cx,
            caret_cy,
            m.caret_w,
            caret_h,
            CORNER_RADIUS,
        );
    }
}
