//! Honest no-document chrome: its two actions, shaping, and shared hit geometry.

use super::*;

const START_ACTIONS: [&str; 2] = ["New document", "Go to"];
/// The two actions' real bound chords (`assets/keymap-defaults.toml`'s
/// `new_document`/`go_to` slugs — Cmd-N, Cmd-O), hardcoded as mac glyphs like
/// every other render-side hint string (`panel.rs`'s replace-hint, `whichkey.rs`):
/// this layer never re-derives a convention-aware label. Bare Enter is NOT one of
/// these — `resolve.rs` sends it to `Action::Newline`, and `no_document.rs::
/// reject_without_document` rejects anything but `NewDocument`/`OpenGoto`/`Quit`
/// with no document open, so a `↵` glyph here would draw a control that does
/// nothing.
const START_CHORDS: [&str; 2] = ["\u{2318}N", "\u{2318}O"];

fn start_rows(width: f32, height: f32, row_h: f32) -> [[f32; 4]; 2] {
    let block_h = row_h * START_ACTIONS.len() as f32;
    let top = ((height - block_h) * 0.5).max(0.0);
    let row_w = (width * 0.4).clamp(180.0, 360.0);
    let left = (width - row_w) * 0.5;
    [[left, top, row_w, row_h], [left, top + row_h, row_w, row_h]]
}

impl TextPipeline {
    pub(in crate::render) fn prepare_start_surface(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.gutter_stack_plate
            .prepare(device, queue, width, height, &[]);
        self.gutter_close_hover_plate
            .prepare(device, queue, width, height, &[]);
        let label = crate::markdown::type_scale::LABEL;
        let row_h = self.metrics.line_height * label;
        self.gutter_buffer.set_metrics(
            &mut self.font_system,
            GlyphMetrics::new(self.metrics.font_size * label, row_h),
        );
        let rows = start_rows(width as f32, height as f32, row_h);
        let [left, top, row_w, _] = rows[0];
        self.gutter_buffer
            .set_size(&mut self.font_system, Some(row_w), Some(row_h * 2.0 + 1.0));
        let base = panel_attrs();
        // BOTH actions read in the SAME full ink — hierarchy is order alone, not
        // ink (DECIDED: neither reads as disabled). Each row's chord rides beside
        // its verb in the established quiet-chord/full-ink-verb split every
        // secondary-column reads (`shape_overlay_right`'s `ink`-primary/
        // `muted`-chord pairing): the chord glyph through `push_symbol_split` (⌘
        // is `is_symbol`, tofu on the display face without the symbol-family
        // split) in `muted`, the verb that follows in `ink`.
        let ink = theme::base_content().to_glyphon();
        let muted = theme::muted().to_glyphon();
        let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
        let mut spans: Vec<(&str, glyphon::Attrs)> = Vec::new();
        push_symbol_split(
            &mut spans,
            START_CHORDS[0],
            || base.clone().color(muted),
            || sym(muted),
        );
        let verb0 = format!(" {}\n", START_ACTIONS[0]);
        spans.push((verb0.as_str(), base.clone().color(ink)));
        push_symbol_split(
            &mut spans,
            START_CHORDS[1],
            || base.clone().color(muted),
            || sym(muted),
        );
        let verb1 = format!(" {}", START_ACTIONS[1]);
        spans.push((verb1.as_str(), base.clone().color(ink)));
        self.gutter_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &base.color(ink),
            Shaping::Advanced,
            Some(glyphon::cosmic_text::Align::Center),
        );
        self.gutter_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let area = TextArea {
            buffer: &self.gutter_buffer,
            left,
            top,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            },
            default_color: ink,
            custom_glyphs: &[],
        };
        self.gutter_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon start-surface prepare failed: {e:?}"))
    }

    pub fn start_action_at(&self, x: f32, y: f32) -> Option<crate::keymap::Action> {
        if self.document_active {
            return None;
        }
        let row_h = self.metrics.line_height * crate::markdown::type_scale::LABEL;
        let rows = start_rows(self.window_w, self.window_h, row_h);
        let row = rows.iter().position(|[rx, ry, rw, rh]| {
            x >= *rx && x <= *rx + *rw && y >= *ry && y <= *ry + *rh
        })?;
        Some(match row {
            0 => crate::keymap::Action::NewDocument,
            1 => crate::keymap::Action::OpenGoto,
            _ => unreachable!("start surface has exactly two rows"),
        })
    }

    pub fn start_actions(&self) -> &'static [&'static str] {
        if self.document_active {
            &[]
        } else {
            &START_ACTIONS
        }
    }

    pub fn document_active(&self) -> bool {
        self.document_active
    }

    pub(in crate::render) fn background_bounds(&self, width: u32) -> (f32, f32) {
        if !self.document_active {
            return (0.0, 0.0);
        }
        let (page_on, _measure, col_left, col_w) = self.page_geometry();
        if page_on {
            (col_left, col_w)
        } else {
            (0.0, width as f32)
        }
    }
}
