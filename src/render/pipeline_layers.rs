use super::*;

impl TextPipeline {
    fn begin_clear_pass<'a>(
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl text pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(theme::base_100().to_wgpu()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        })
    }

    /// Record the clear + text/caret draw into `encoder`, targeting `view`.
    ///
    /// Two paths. For the COMMON case (no overlay, the search SPLIT panel, OR a crisp
    /// THEME/CARET picker) everything composites in ONE pass over the cleared view —
    /// byte-identical to before, so a non-overlay document capture is unchanged. For a
    /// blur-eligible full overlay the document is rendered ONCE to an offscreen
    /// texture, blurred (only when [`Self::blur_recompute`] — else the cache stands),
    /// and the frosted result is composited behind the overlay card in the final pass.
    ///
    /// # THE COMPARISON SITS *ON* THE WORKSPACE SURFACE
    ///
    /// A summoned workspace can relocate the document layer's GEOMETRY into its
    /// content region ([`Self::comparison_viewport`]); its place in painter's order
    /// is a separate decision, and the user's compositing call (2026-08-02) is that
    /// the comparison sits ON the workspace's own surface rather than being a window
    /// THROUGH it: the card stays ONE OPAQUE SURFACE
    /// (`overlay_pane_fills` is still its whole box) and the document's CONTENT is
    /// submitted AFTER it, into the carved region, **without re-drawing its own
    /// ground**. A window through the card would have shown the BACKDROP's ground —
    /// the punch is at the page column, not at the region. That is why the document
    /// layer splits in two here:
    ///
    ///   * [`Self::draw_document_ground`] — the backdrop's own ground (the world's
    ///     margin field, lava, stars, the page frame). It is the QUIET FRAME around
    ///     the workspace and it never moves.
    ///   * [`Self::draw_document_content`] — the prose and everything hung off it
    ///     (washes, selection, underlines, caret, text, ornaments, tables). This is
    ///     what relocates, and on a comparison frame it is drawn LAST.
    ///
    /// The ordinary frame concatenates the two in their original order, so every
    /// non-comparison frame in the tree is byte-identical BY CONSTRUCTION rather than
    /// by measurement.
    ///
    /// **The blur path captures the GROUND ALONE while a comparison is up.** The
    /// offscreen capture is what gets frosted into the frame AROUND the workspace,
    /// and a relocated transcript has no business appearing there — the ghost would
    /// land exactly where the region is not. Every margin-orientation surface already
    /// yields on such a frame (`margin_orientation_yields`); the frosted backdrop now
    /// yields with them.
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> anyhow::Result<()> {
        // Is the document layer relocated into a workspace's content region this
        // frame? Asked ONCE here; both paths below read the answer.
        let relocated = self.comparison_viewport().is_some();
        if self.backdrop_blur() {
            // 1) Capture the document into the offscreen texture + blur it — but ONLY
            //    when the cached backdrop is stale (a fresh open / resize / doc or
            //    theme change). A settled overlay-open (or HUD-held) frame skips straight
            //    to the composite, re-blurring nothing (DESIGN §6).
            if self.blur_recompute {
                if let Some(doc_view) = self.blur.doc_view() {
                    let mut pass = Self::begin_clear_pass(encoder, doc_view);
                    match relocated {
                        true => self.draw_document_ground(&mut pass),
                        false => self.draw_document_layers(&mut pass)?,
                    }
                }
                self.blur.encode_blur(encoder);
            }
            let mut pass = Self::begin_clear_pass(encoder, view);
            self.blur.draw_backdrop(&mut pass);
            self.draw_overlay_card(&mut pass)?;
            if relocated {
                self.draw_document_content(&mut pass)?;
            }
            self.draw_chrome_tail(&mut pass)?;
            return Ok(());
        }

        let mut pass = Self::begin_clear_pass(encoder, view);
        match relocated {
            true => self.draw_document_ground(&mut pass),
            false => self.draw_document_layers(&mut pass)?,
        }
        // The search panel / crisp overlay composites OVER the document text. There is
        // no depth buffer (depth_stencil: None everywhere) so painter's order == draw
        // submission order.
        if self.overlay_active {
            self.draw_overlay_card(&mut pass)?;
            // …and the COMPARISON composites over the card, into the region the card
            // carved for it. `relocated` already implies `overlay_active`.
            if relocated {
                self.draw_document_content(&mut pass)?;
            }
        } else if self.search_active {
            self.float_shadow.draw(&mut pass);
            self.float_border.draw(&mut pass);
            self.float_card.draw(&mut pass);
            self.panel_card.draw(&mut pass);
            self.panel_caret.draw(&mut pass);
            self.panel_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .map_err(|e| anyhow::anyhow!("glyphon panel render failed: {e:?}"))?;
        }
        self.draw_chrome_tail(&mut pass)?;
        Ok(())
    }

    /// Draw the DOCUMENT layers (everything behind any overlay) into an open pass, in
    /// painter's order: PAGE-MODE margin gradient -> selection -> search-match ->
    /// wavy spell underlines -> straight muted nit underlines -> BLOCK caret quad -> cosmetic trail -> document text ->
    /// MORPH caret silhouette (OVER the text) -> page-mode gutter -> markdown
    /// ornaments. The block caret sits BELOW the glyph cell so the letter is never
    /// covered; the morph caret paints the cursor glyph's silhouette OVER the letter
    /// to recolour it the accent. Shared by the common path and the blur path's
    /// offscreen doc capture, so the captured backdrop matches the live document.
    fn draw_document_layers<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) -> anyhow::Result<()> {
        self.draw_document_ground(pass);
        self.draw_document_content(pass)
    }

    /// THE BACKDROP'S OWN GROUND: the world's margin field, the lava blobs, the
    /// star band and the writing page's frame. It describes the
    /// CANVAS, not the prose — nothing here reads the four relocated document-geometry
    /// owners except through the module-private page column, which deliberately never
    /// moves ([`Self::page_column_left`]). So it stays the quiet frame around a
    /// summoned workspace and is the only thing a comparison frame frosts.
    fn draw_document_ground<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.background_pipeline.draw(pass);
        self.lava_pipeline.draw(pass);
        self.stars_pipeline.draw(pass);
        self.page_frame_pipeline.draw(pass);
    }

    /// THE DOCUMENT'S CONTENT: the prose and everything hung off it. Every emitter
    /// here composes off the four relocated geometry owners, so this
    /// whole block travels into a workspace's comparison region as one piece — which
    /// is what lets `render` submit it AFTER the card without re-drawing any ground.
    fn draw_document_content<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) -> anyhow::Result<()> {
        self.fence_panel_pipeline.draw(pass);
        self.code_pill_pipeline.draw(pass);
        self.table_rule_pipeline.draw(pass);
        self.wash_comment_pipeline.draw(pass);
        self.wash_string_pipeline.draw(pass);
        self.wash_highlight_pipeline.draw(pass);
        // Images are below selection, caret, and revealed source text.
        self.image_placeholder_pipeline.draw(pass);
        self.image_pipeline.draw(pass);
        self.image_scrim_pipeline.draw(pass);
        self.selection_pipeline.draw(pass);
        self.match_pipeline.draw(pass);
        self.spell_pipeline.draw(pass);
        self.nit_pipeline.draw(pass);
        self.strike_pipeline.draw(pass);
        self.link_underline_pipeline.draw(pass);
        self.caret_pipeline.draw(pass);
        self.caret_trail_pipeline.draw(pass);
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon render failed: {e:?}"))?;
        // Inverse-video selection requires composited text and ground underneath.
        self.selection_invert.draw(pass);
        // Inverse caret shares the after-text slot required by its blend mode.
        self.caret_invert.draw(pass);
        self.caret_glyph_pipeline.draw(pass);
        self.gutter_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon gutter render failed: {e:?}"))?;
        self.outline_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon outline render failed: {e:?}"))?;
        self.ornament_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon ornament render failed: {e:?}"))?;
        self.table_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon table render failed: {e:?}"))?;
        // INLINE IMAGES: the missing-file placeholder LABELS (filename + alt), over
        // their base_200 card (drawn earlier, before selection). Parked (no areas)
        // when nothing is missing, so a default frame is byte-identical.
        self.image_placeholder_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon image placeholder render failed: {e:?}"))?;
        Ok(())
    }

    fn draw_overlay_card<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) -> anyhow::Result<()> {
        self.float_shadow.draw(pass);
        self.float_border.draw(pass);
        self.float_card.draw(pass);
        self.panel_shadow.draw(pass);
        self.panel_border.draw(pass);
        self.panel_card.draw(pass);
        let bars = matches!(
            crate::render::effective_list_style(),
            theme::ListStyle::Bars { .. } | theme::ListStyle::Diagonal(_)
        );
        if bars {
            self.placard_stipple.draw(pass);
            self.placard_renderer
                .render(&self.atlas, &self.viewport, pass)
                .map_err(|e| anyhow::anyhow!("glyphon placard render failed: {e:?}"))?;
        }
        self.overlay_bars.draw(pass);
        self.overlay_spine.draw(pass);
        self.overlay_spine_selected.draw(pass);
        self.overlay_rows.draw(pass);
        self.overlay_cross.draw(pass);
        self.overlay_range_track.draw(pass);
        self.overlay_range_thumb.draw(pass);
        self.overlay_facet_ghost.draw(pass);
        self.overlay_lens_underline.draw(pass);
        self.panel_caret.draw(pass);
        if !bars {
            self.placard_stipple.draw(pass);
        }
        self.panel_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon overlay render failed: {e:?}"))?;
        // CARET-STYLE PICKER: the animated demo caret (under the sample text, like the
        // document block caret), then the sample line, then — Morph only, settled on
        // a real glyph — the demo's OWN silhouette pipeline OVER the text, exactly
        // mirroring the document's block-caret -> text -> glyph-silhouette painter's
        // order (`draw_document_layers`). Both on the preview card drawn above.
        // Parked/empty unless the caret-style picker is open.
        self.caret_preview_pipeline.draw(pass);
        self.preview_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon preview render failed: {e:?}"))?;
        self.caret_preview_glyph_pipeline.draw(pass);
        Ok(())
    }

    fn draw_chrome_tail<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) -> anyhow::Result<()> {
        self.debug_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon debug render failed: {e:?}"))?;
        self.notice_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon notice render failed: {e:?}"))?;
        self.page_drag_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon page-drag-readout render failed: {e:?}"))?;
        self.zoom_readout_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon zoom-readout render failed: {e:?}"))?;
        // Float-panel elevation, painter's order: drop shadow -> raised border -> card.
        self.hud_shadow.draw(pass);
        self.hud_border.draw(pass);
        self.hud_card.draw(pass);
        self.streak_cells.draw(pass);
        self.hud_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon hud render failed: {e:?}"))?;
        self.wk_shadow.draw(pass);
        self.wk_border.draw(pass);
        self.wk_card.draw(pass);
        self.wk_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon whichkey render failed: {e:?}"))?;
        self.menubar_bg.draw(pass);
        self.menubar_hi.draw(pass);
        self.menubar_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon menubar render failed: {e:?}"))?;
        self.menu_drop_shadow.draw(pass);
        self.menu_drop_border.draw(pass);
        self.menu_drop_card.draw(pass);
        self.menu_drop_sep.draw(pass);
        self.menu_drop_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon menu-drop label render failed: {e:?}"))?;
        self.menu_chord_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon menu-drop chord render failed: {e:?}"))?;
        // THE FORMAT POPOVER, drawn LAST so it floats over the document (like the
        // which-key panel): float elevation -> active-button value-step wash ->
        // button labels. ALL parked off-screen/empty when the popover is down, so
        // a default render is byte-identical.
        //
        // THE SHARED FLOAT-SURFACE QUADS (overlay/chrome polish round): the
        // popover's elevation trio is `float_shadow`/`float_border`/`float_card` —
        // the SAME quads the caret-preview panel / spell popup (`draw_overlay_card`,
        // gated on `overlay_active`) and the search panel (`render`'s
        // `search_active` branch) already draw. Those two call sites already cover
        // every frame where an overlay OR the search panel is up (and `prepare_float_panel`'s
        // call-order guarantees whichever of the three PREPARED real content this
        // frame is the one those buffers hold — see that fn's doc); this ONE extra
        // draw call covers the remaining case (no overlay, no search — exactly when
        // the popover CAN be the real summoner). Drawing it a second time whenever
        // overlay/search already drew it would double-blend the translucent shadow,
        // so it's gated to fire only in the case those two draw sites DON'T cover.
        if !self.overlay_active && !self.search_active {
            self.float_shadow.draw(pass);
            self.float_border.draw(pass);
            self.float_card.draw(pass);
        }
        self.popover_wash.draw(pass);
        self.popover_hl_wash.draw(pass);
        self.popover_strike.draw(pass);
        self.popover_renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| anyhow::anyhow!("glyphon popover render failed: {e:?}"))?;
        Ok(())
    }
}
