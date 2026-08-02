use super::*;

impl TextPipeline {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hud_stats(&mut self, stats: Option<crate::hud::HudStats>) {
        self.hud_stats = stats;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_streaks(&mut self, view: Option<crate::streaks::StreaksView>) {
        self.streaks_view = view;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hud_saved(&mut self, state: Option<crate::hud::HudSaved>) {
        self.hud_saved = state;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_update_checked(&mut self, state: Option<crate::updates::UpdateChecked>) {
        self.hud_update_checked = state;
    }

    pub fn hud_update_checked(&self) -> Option<crate::updates::UpdateChecked> {
        self.hud_update_checked
    }

    pub fn set_pending_crash(&mut self, pending: bool) {
        self.hud_pending_crash = pending;
    }

    pub fn hud_pending_crash(&self) -> bool {
        self.hud_pending_crash
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_peek_rows(&mut self, rows: Vec<crate::peek::PeekRow>) {
        self.peek_rows = rows;
    }

    pub(in crate::render) fn peek_effective_rows(&self) -> Vec<crate::peek::PeekRow> {
        crate::peek::rows_or_starter(&self.peek_rows)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_keybindings_tips(&mut self, tips: Vec<String>) {
        self.keybindings_tips = tips;
    }

    fn hud_percent(&self) -> u32 {
        let lines = &self.buffer.lines;
        let total_chars: usize = lines.iter().map(|l| l.text().chars().count()).sum();
        let denom = total_chars + lines.len().saturating_sub(1); // + inter-line newlines
        if denom == 0 {
            return 0;
        }
        let mut offset = 0usize;
        for l in lines.iter().take(self.cursor_line) {
            offset += l.text().chars().count() + 1; // + the line's trailing newline
        }
        offset += self.cursor_col;
        (((offset.min(denom) as f32) / denom as f32) * 100.0).round() as u32
    }

    pub fn hud_report(&self) -> HudReport {
        HudReport {
            held: crate::hud::hud_held(),
            words: self.readout_report(),
            percent: self.hud_percent(),
            lang: self.doc_lang_report(),
            eol: self.eol,
            saved: crate::hud::saved_readout(self.hud_saved),
        }
    }

    pub fn lifetime_report(&self) -> LifetimeReport {
        let [chars, writing, files, caret_travel, world] =
            crate::hud::odometer_rows(self.hud_stats.as_ref()).map(|(_, v)| v);
        LifetimeReport {
            open: crate::lifetime::lifetime_open(),
            chars,
            writing,
            files,
            caret_travel,
            world,
        }
    }

    pub fn streaks_report(&self) -> StreaksReport {
        let view = self.streaks_effective_view();
        StreaksReport {
            open: crate::streaks::streaks_open(),
            view: crate::streaks::card_view().label(),
            streak: view.streak,
            today_words: view.today_words,
            total_words: view.cumulative.last().copied().unwrap_or(0),
            cells: view.cells.to_vec(),
        }
    }

    pub(in crate::render) fn streaks_effective_view(&self) -> crate::streaks::StreaksView {
        self.streaks_view
            .clone()
            .unwrap_or_else(crate::streaks::placeholder)
    }

    pub fn peek_report(&self) -> PeekReport {
        PeekReport {
            open: crate::peek::peek_open(),
            rows: self.peek_effective_rows(),
        }
    }

    /// Everything a summoned card can say, gathered for
    /// [`crate::card::content`] — the ONE owner of the content. The pipeline
    /// is the only holder of all of these at once, so it is the only gatherer;
    /// composition, captions and phrasing live there, not here.
    pub fn card_inputs(&self) -> crate::card::content::CardInputs {
        crate::card::content::CardInputs {
            hud_held: self.hud_showing(),
            peek_shown: self.peek_showing(),
            stats: self.hud_stats.clone(),
            streaks: Some(self.streaks_effective_view()),
            streaks_page: crate::streaks::card_view(),
            saved: self.hud_saved,
            words: self.wordcount_text(),
            lang: self.doc_lang_report(),
            percent: self.hud_percent(),
            eol: self.eol,
            peek_rows: self.peek_effective_rows(),
            update_checked: self.hud_update_checked,
            pending_crash: self.hud_pending_crash,
        }
    }

    /// The summoned card this frame, as CONTENT. The semantic tree reads this
    /// same value, so an assistive technology hears exactly the card that is
    /// drawn rather than a second description of it.
    pub fn card_content(&self) -> Option<crate::card::content::CardContent> {
        crate::card::content::open_card(&self.card_inputs())
    }

    pub(in crate::render) fn prepare_hud(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let held = self.hud_showing();
        let about = crate::about::about_open();
        let lifetime = crate::lifetime::lifetime_open();
        let streaks = crate::streaks::streaks_open();
        let peek = self.peek_showing();
        let showing = held || about || lifetime || streaks || peek;
        if !streaks {
            self.streak_cells
                .prepare_multicolor(device, queue, width, height, &[]);
        } else {
            return self.prepare_streaks_card(device, queue, width, height);
        }
        if !showing {
            set_float_quads(
                &mut self.hud_shadow,
                &mut self.hud_border,
                &mut self.hud_card,
                device,
                queue,
                width,
                height,
                None,
                FloatElevation::Rimmed,
                0.0,
                None,
            );
        }

        let m = self.metrics;
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let content = theme::base_content().to_glyphon();
        let faint = theme::faint().to_glyphon();

        if !showing {
            self.hud_buffer
                .set_size(&mut self.font_system, Some(1.0), Some(m.line_height));
            self.hud_buffer.set_text(
                &mut self.font_system,
                "",
                &panel_attrs().color(content),
                Shaping::Advanced,
                None,
            );
            self.hud_buffer
                .shape_until_scroll(&mut self.font_system, false);
            let area = TextArea {
                buffer: &self.hud_buffer,
                left: 0.0,
                top: -1000.0,
                scale: 1.0,
                bounds,
                default_color: content,
                custom_glyphs: &[],
            };
            self.hud_renderer
                .prepare(
                    device,
                    queue,
                    &mut self.font_system,
                    &mut self.atlas,
                    &self.viewport,
                    [area],
                    &mut self.swash_cache,
                )
                .map_err(|e| anyhow::anyhow!("glyphon hud prepare failed: {e:?}"))?;
            return Ok(());
        }

        let label = crate::markdown::type_scale::LABEL;
        let section = crate::markdown::type_scale::SECTION;
        let body_metrics = GlyphMetrics::new(m.font_size, m.line_height);
        let label_metrics = GlyphMetrics::new(m.font_size * label, m.line_height * label);
        let title_metrics = GlyphMetrics::new(m.font_size * section, m.line_height * section);

        // CONTENT comes from the one card owner; this function keeps the style,
        // metrics and geometry that are genuinely the renderer's.
        let owned: Vec<(String, u8)> = self
            .card_content()
            .map(|content| content.spans())
            .unwrap_or_default();
        let base = panel_attrs();
        let spans: Vec<(&str, Attrs)> = owned
            .iter()
            .map(|(s, role)| {
                let attrs = match role {
                    0 => base.clone().color(faint).metrics(label_metrics),
                    2 => base.clone().color(content).metrics(title_metrics),
                    // The About end-mark ornament: override to the world's ornament
                    // face at NORMAL weight (the ornament faces are Regular/400, and a
                    // stale display weight — e.g. IBM Plex Mono's 300 — would trip the
                    // weight_diff fallback filter and drop the face).
                    3 => base
                        .clone()
                        .color(content)
                        .metrics(body_metrics)
                        .family(Family::Name(theme::active().ornament_face))
                        .weight(glyphon::Weight::NORMAL),
                    _ => base.clone().color(content).metrics(body_metrics),
                };
                (s.as_str(), attrs)
            })
            .collect();
        // No alignment (cosmic-text defaults to LEFT): each line starts at the buffer's
        // left edge, and the TextArea `left` (below) plants that spine inside the card.
        // Generous buffer width so the value lines never wrap.
        self.hud_buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32),
        );
        let default_attrs = base.clone().color(content).metrics(body_metrics);
        self.hud_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        self.hud_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut block_h = 0.0_f32;
        let mut block_w = 0.0_f32;
        for run in self.hud_buffer.layout_runs() {
            block_h = block_h.max(run.line_top + run.line_height);
            block_w = block_w.max(run.line_w);
        }
        let top = ((height as f32 - block_h) * 0.5).max(TEXT_TOP);
        let pad_x = m.char_width * 3.0;
        let pad_y = m.line_height * 0.9;
        let card_w = block_w + pad_x * 2.0;
        let card_h = block_h + pad_y * 2.0;
        let card_x = (width as f32 - card_w) * 0.5;
        let card_y = top - pad_y;
        set_float_quads(
            &mut self.hud_shadow,
            &mut self.hud_border,
            &mut self.hud_card,
            device,
            queue,
            width,
            height,
            Some([card_x, card_y, card_w, card_h]),
            FloatElevation::Rimmed,
            0.0,
            None,
        );
        let area = TextArea {
            buffer: &self.hud_buffer,
            left: card_x + pad_x,
            top,
            scale: 1.0,
            bounds,
            default_color: content,
            custom_glyphs: &[],
        };
        self.hud_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon hud prepare failed: {e:?}"))?;
        Ok(())
    }

    pub(in crate::render) fn prepare_streaks_card(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        use crate::streaks::{CardView, DAYS_PER_WEEK, LEVELS, WEEKS};
        let owned = crate::card::content::card(
            crate::card::content::CardKind::Streaks,
            &self.card_inputs(),
        )
        .spans();
        let view = self.streaks_effective_view();
        let page = crate::streaks::card_view();
        let m = self.metrics;
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let content = theme::base_content().to_glyphon();
        let faint = theme::faint().to_glyphon();
        let label = crate::markdown::type_scale::LABEL;
        let body_metrics = GlyphMetrics::new(m.font_size, m.line_height);
        let label_metrics = GlyphMetrics::new(m.font_size * label, m.line_height * label);

        let cell = (m.char_width * 0.85).max(4.0);
        let gap = (cell * 0.30).max(1.0);
        let step = cell + gap;
        let grid_w = WEEKS as f32 * step - gap;
        let grid_h = DAYS_PER_WEEK as f32 * step - gap;

        let base = panel_attrs();
        let spans: Vec<(&str, Attrs)> = owned
            .iter()
            .map(|(s, role)| {
                let attrs = match role {
                    0 => base.clone().color(faint).metrics(label_metrics),
                    _ => base.clone().color(content).metrics(body_metrics),
                };
                (s.as_str(), attrs)
            })
            .collect();
        self.hud_buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32),
        );
        let default_attrs = base.clone().color(content).metrics(body_metrics);
        self.hud_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        self.hud_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut text_h = 0.0_f32;
        let mut text_w = 0.0_f32;
        for run in self.hud_buffer.layout_runs() {
            text_h = text_h.max(run.line_top + run.line_height);
            text_w = text_w.max(run.line_w);
        }

        let pad_x = m.char_width * 3.0;
        let pad_y = m.line_height * 0.9;
        let dot = (cell * 0.55).max(3.0); // the page-dot square (echoes a heatmap cell)
        let gap_dots = m.line_height * 0.5; // breathing room body→dots
        let gap_between = m.line_height * 0.75; // and dots→stats
        let content_w = grid_w.max(text_w);
        let content_h = grid_h + gap_dots + dot + gap_between + text_h;
        let card_w = content_w + pad_x * 2.0;
        let card_h = content_h + pad_y * 2.0;
        let card_x = ((width as f32 - card_w) * 0.5).max(0.0);
        let card_y = ((height as f32 - card_h) * 0.5).max(TEXT_TOP - pad_y);
        let content_top = card_y + pad_y;
        let grid_x = card_x + (card_w - grid_w) * 0.5;
        let grid_y = content_top;
        let dots_y = grid_y + grid_h + gap_dots;
        let text_top = dots_y + dot + gap_between;

        let colors = theme::heatmap_colors();
        let mut quads: Vec<([f32; 4], [u8; 4])> = Vec::with_capacity(WEEKS * DAYS_PER_WEEK * 2 + 2);
        match page {
            CardView::Heatmap => {
                for col in 0..WEEKS {
                    for row in 0..DAYS_PER_WEEK {
                        let idx = col * DAYS_PER_WEEK + row;
                        let bucket = (view.cells[idx] as usize).min(LEVELS - 1);
                        let x = grid_x + col as f32 * step;
                        let y = grid_y + row as f32 * step;
                        quads.push(([x, y, cell, cell], colors[bucket].rgba_bytes()));
                    }
                }
            }
            CardView::Cumulative => {
                let bars =
                    crate::streaks::chart_bars(&view.cumulative, [grid_x, grid_y, grid_w, grid_h]);
                for b in &bars {
                    quads.push((*b, colors[1].rgba_bytes()));
                    quads.push((
                        [b[0], b[1], b[2], b[3].min(2.0)],
                        colors[LEVELS - 1].rgba_bytes(),
                    ));
                }
            }
        }

        let active_idx = match page {
            CardView::Heatmap => 0usize,
            CardView::Cumulative => 1,
        };
        let dot_gap = dot * 1.25;
        let dots_w = dot * 2.0 + dot_gap;
        let dots_x = card_x + (card_w - dots_w) * 0.5;
        for i in 0..2usize {
            let on = i == active_idx;
            let d = if on { dot } else { (dot * 0.6).max(2.0) };
            let slot_x = dots_x + i as f32 * (dot + dot_gap);
            let inset = (dot - d) * 0.5;
            let tint = if on { colors[LEVELS - 1] } else { colors[2] };
            quads.push(([slot_x + inset, dots_y + inset, d, d], tint.rgba_bytes()));
        }
        self.streak_cells
            .prepare_multicolor(device, queue, width, height, &quads);

        set_float_quads(
            &mut self.hud_shadow,
            &mut self.hud_border,
            &mut self.hud_card,
            device,
            queue,
            width,
            height,
            Some([card_x, card_y, card_w, card_h]),
            FloatElevation::Rimmed,
            0.0,
            None,
        );

        let area = TextArea {
            buffer: &self.hud_buffer,
            left: grid_x,
            top: text_top,
            scale: 1.0,
            bounds,
            default_color: content,
            custom_glyphs: &[],
        };
        self.hud_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon streaks prepare failed: {e:?}"))?;
        Ok(())
    }
}
