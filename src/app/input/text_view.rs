//! Project composition into a field's drawn text without changing its saved value.

use super::TextTarget;
use crate::app::*;
use crate::textbox::TextField;

impl App {
    pub(in crate::app) fn update_ime_cursor_area(&self) {
        let Some(gpu) = self.frame.gpu() else { return };
        let (x, y, w, h) = match self.focused_text_target() {
            TextTarget::Document(_) => gpu.pipeline.caret_pixel_rect(),
            TextTarget::Field { .. } => {
                let Some([x, y, w, h]) = gpu.pipeline.focused_field_caret_rect() else {
                    return;
                };
                (x, y, w, h)
            }
            TextTarget::None => return,
        };
        gpu.window.set_ime_cursor_area(
            winit::dpi::PhysicalPosition::new(x as f64, y as f64),
            winit::dpi::PhysicalSize::new(w.max(1.0) as f64, h.max(1.0) as f64),
        );
    }

    pub(in crate::app) fn project_text_input(&self, view: &mut ViewState) {
        let target = self.focused_text_target();
        if !matches!(target, TextTarget::Document(_)) {
            view.preedit.clear();
        }
        if matches!(target, TextTarget::None) {
            view.overlay_query_focused = false;
        }
        if let Some(input) = self.field_input_projection() {
            view.apply_field_input(input);
        }
    }

    pub(in crate::app) fn field_input_projection(&self) -> Option<crate::render::FieldInput> {
        let TextTarget::Field { field, .. } = self.focused_text_target() else {
            return None;
        };
        let input = self.focused_text_box()?;
        let mut composed = input.clone();
        let preedit = self.input.preedit();
        let range = if preedit.is_empty() {
            None
        } else {
            let start = input
                .selection_range()
                .map(|(start, _)| start)
                .unwrap_or(input.caret());
            composed.insert_text(preedit);
            let cursor = self.input.keyboard.preedit_cursor;
            composed.set_caret(start + cursor.min(preedit.chars().count()));
            Some((start, start + preedit.chars().count()))
        };
        let text = composed.text().to_string();
        let caret = composed.caret();
        let selection = composed.selection_range();
        let row = (field == TextField::SettingsValue)
            .then(|| self.workspace_state.overlay())
            .flatten()
            .and_then(|card| {
                let edit = card.value_edit.as_ref()?;
                card.items.iter().position(|index| *index == edit.row)
            });
        Some(crate::render::FieldInput {
            field,
            text,
            caret,
            selection,
            preedit: range,
            row,
        })
    }
}
