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
        let mut plan = self.overlay_row_plan(&geom);
        let placard_geometry = self.overlay_shape_placard(&geom);
        let stipple = matches!(
            crate::render::effective_title_style(),
            theme::TitleStyle::Placard {
                ink: theme::PlacardInk::Stipple,
                ..
            }
        );
        let (placard, stipple_rects) = match placard_geometry {
            Some((x, y, _w, _h)) if stipple => (None, self.placard_stipple_rects((x, y))),
            other => (other, Vec::new()),
        };
        self.placard_stipple
            .prepare(device, queue, width, height, &stipple_rects);
        let selected_ink = super::overlay_selected_primary_ink();
        // RESOLVE THE VISUAL-SELECTION TRANSACTION ONCE, here, before
        // anything is shaped or emitted. This is the ONLY call that runs a band
        // animator for this frame; every consumer below reads the result, so the
        // band, both ink columns, the accessory plates and the sidecar give ONE
        // answer to "which row is selected" at every intermediate frame.
        let vis = self.resolve_visual_selection(&geom, &plan);
        // The workspace's navigation rail shapes into its own column
        // buffer before the content pane does, so its measured mark rect is in
        // hand by the time `overlay_draw_card` asks the facet-mark owner for it.
        let has_rail = self.workspace_shape_rail(&geom, &plan);
        let has_right = self.overlay_shape_text(
            &geom,
            &plan,
            OverlaySpanInks {
                ink,
                muted,
                selected: selected_ink,
            },
            &vis,
            true,
        );
        self.diagonal_cluster = self.resolve_diagonal_cluster(&geom, &plan, &vis);
        plan.complete_row_extent(self.diagonal_row_extent()); // completed, not rebuilt
        let surface = OverlayCardSurface {
            device,
            queue,
            width,
            height,
            geom: &geom,
            plan: &plan,
        };
        self.overlay_upload_text(surface, has_right, has_rail, ink, muted, placard)?;
        self.prepare_overlay_rotated_location(device, queue, width, height, &geom, &plan);
        self.overlay_draw_card(surface, &vis);
        self.prepare_overlay_material(device, queue, width, height, &geom, placard_geometry);
        self.overlay_place_caret(queue, width, height, &geom, &plan);
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
        self.panel_material
            .prepare(device, queue, width, height, &[]);
        self.overlay_rows.prepare(device, queue, width, height, &[]);
        self.overlay_bars.prepare(device, queue, width, height, &[]);
        self.footer_plate_rim
            .prepare(device, queue, width, height, &[]);
        self.overlay_spine
            .prepare_rotated(device, queue, width, height, &[]);
        self.overlay_spine_selected
            .prepare_rotated(device, queue, width, height, &[]);
        // ARM B LIVING-BAND PROBE: the two-shape crossing quad parks empty too, so
        // a closed picker carries no stale crossing quad into the next frame.
        self.overlay_cross
            .prepare(device, queue, width, height, &[]);
        // The range rail's track + thumb park empty too, so a closed
        // Settings menu carries no stale rail quads into the next frame.
        self.overlay_range_track
            .prepare(device, queue, width, height, &[]);
        self.overlay_range_thumb
            .prepare(device, queue, width, height, &[]);
        self.overlay_lens_underline
            .prepare(device, queue, width, height, &[]);
        // The workspace rail's placement and its active mark park with
        // the card, so the frame after a workspace closes carries neither.
        self.workspace_rail_placement = None;
        self.workspace_rail_rows.clear();
        // V6 P5: the Chips ghost pills park empty too, so a closed picker carries
        // no stale ghost-pill quads into the next frame.
        self.overlay_facet_ghost
            .prepare(device, queue, width, height, &[]);
        // The stipple placard: parked (zero instances) — the frame after a
        // stipple-world overlay closes carries zero stale wordmark pixels.
        self.placard_stipple
            .prepare(device, queue, width, height, &[]);
        self.placard_material
            .prepare(device, queue, width, height, &[]);
        // The rotated location cue parks too, so the frame after a
        // `RotatedRail` world's overlay closes (or a lens change drops it)
        // carries no stale vertical run.
        self.rotated_label_pipeline.clear();
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

    fn overlay_upload_text(
        &mut self,
        surface: OverlayCardSurface,
        has_right: bool,
        has_rail: bool,
        ink: glyphon::Color,
        muted: glyphon::Color,
        placard: Option<(f32, f32, f32, f32)>,
    ) -> anyhow::Result<()> {
        let OverlayCardSurface {
            device,
            queue,
            width,
            height,
            geom,
            plan,
        } = surface;
        let text_left = geom.text_left;
        let text_top = geom.text_top;
        let bounds = TextBounds {
            left: geom.upload_left() as i32,
            top: 0,
            right: geom.upload_right(width) as i32,
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
        // Bars use the dedicated placard pass; Pane keeps the historical
        // first-in-panel-batch slot. The unused pass is parked every frame.
        let bars = matches!(
            crate::render::effective_list_style(),
            theme::ListStyle::Bars | theme::ListStyle::Diagonal(_) | theme::ListStyle::Ruled(_)
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
        let dock = self.docked_facet_band(geom, plan);
        let docked = push_docked_facet_areas(
            &mut areas,
            &self.panel_buffer,
            &self.docked_facet_buffer,
            dock,
            self.overlay_theme_underline,
            plan.strip_band(),
            text_left,
            text_top,
            bounds.left,
            bounds.right,
            height,
            ink,
        );
        if !docked {
            match self.overlay_panel_bands(geom, plan) {
                None => areas.push(panel_area),
                Some(panel_bands) => {
                    for band in &panel_bands {
                        areas.push(TextArea {
                            buffer: &self.panel_buffer,
                            left: band.left,
                            top: text_top,
                            scale: 1.0,
                            bounds: TextBounds {
                                left: bounds.left,
                                top: band.clip_top.max(0.0) as i32,
                                right: bounds.right,
                                bottom: (band.clip_bottom.min(height as f32)) as i32,
                            },
                            default_color: ink,
                            custom_glyphs: &[],
                        });
                    }
                }
            }
        }
        if has_rail && let Some((left, top, bounds)) = self.workspace_rail_area(geom, width, height)
        {
            areas.push(TextArea {
                buffer: &self.workspace_rail_buffer,
                left,
                top,
                scale: 1.0,
                bounds,
                default_color: muted,
                custom_glyphs: &[],
            });
        }
        if has_right {
            // The chord column is shaped ALIGNED TO ITS FLOW in a text-column-wide
            // buffer, so a chord sits at the cluster end it hangs on rather than at
            // its buffer origin — and WHICH end is the lane owner's one answer.
            let bind_w = self.panel_bind_buffer.size().0.unwrap_or(0.0);
            if self.diagonal_cluster.is_some() {
                let clip = |top: f32, bottom: f32| TextBounds {
                    left: bounds.left,
                    top: top.max(0.0) as i32,
                    right: bounds.right,
                    bottom: (bottom.min(height as f32)) as i32,
                };
                for row in plan.rows() {
                    areas.push(TextArea {
                        buffer: &self.panel_bind_buffer,
                        left: self.overlay_accessory_span(geom, row.display, bind_w).0,
                        top: plan.secondary_top(),
                        scale: 1.0,
                        bounds: clip(row.top, row.bottom()),
                        default_color: muted,
                        custom_glyphs: &[],
                    });
                }
            } else {
                areas.push(TextArea {
                    buffer: &self.panel_bind_buffer,
                    left: self.overlay_accessory_span(geom, 0, bind_w).0,
                    top: plan.secondary_top(),
                    scale: 1.0,
                    bounds,
                    default_color: muted,
                    custom_glyphs: &[],
                });
            }
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
        plan: &OverlayRowPlan,
    ) {
        let Some([x, y, w, h]) = self.overlay_query_caret_box(geom, plan) else {
            self.panel_caret.prepare_empty();
            return;
        };
        self.panel_caret.prepare(
            queue,
            width,
            height,
            CaretRect {
                center_x: x + w * 0.5,
                center_y: y + h * 0.5,
                rect_w: w,
                rect_h: h,
                corner: self.metrics.px(CORNER_RADIUS),
            },
        );
    }

    /// THE QUERY FIELD'S CARET, as the box it is drawn in (`[x, y, w, h]`) — `None` when
    /// the card draws no query line at all (the contextual spell popup).
    ///
    /// A `&self` owner rather than arithmetic inside the placer, because the footprint
    /// frost has to know how far the head band's ink really reaches and the caret stands
    /// its own width past the last glyph. One derivation, so the quad the frost accounts
    /// for is the quad the frame drew.
    pub(in crate::render) fn overlay_query_caret_box(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        // The field's own PLANNED line box. `None` is the contextual spell
        // popup, which draws no query line at all.
        let field = plan.query_band()?;
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
        // `geom.text_left`, or right-aligned against the text column on a world
        // whose rows anchor right — see `overlay_head_left`'s own doc.
        let caret_x = self.overlay_head_left(geom, plan)
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
        // The caret is centred in the SAME planned field box the pointer
        // hit-test accepts and the split composition carves its gap from —
        // never a line height read back off the shaped run here, which is a
        // second calculation only the draw path can see.
        let caret_cy = field.center();
        Some([caret_x, caret_cy - caret_h * 0.5, m.caret_w, caret_h])
    }
}
