//! Typed policy for the contextual spelling popup.
//!
//! Diagonal lists use a measured row cluster beside their rake. Other compositions
//! keep the historical character-grid sizing and secondary-column interpretation.

use super::*;

impl TextPipeline {
    pub(in crate::render) fn diagonal_spell_popup(&self) -> bool {
        self.overlay_spell.is_some()
            && matches!(
                crate::render::effective_list_style(),
                theme::ListStyle::Diagonal(_)
            )
    }

    pub(in crate::render) fn spell_framed_width(
        &self,
        rows: usize,
        measured_w: f32,
        char_grid_w: f32,
        pad: f32,
    ) -> f32 {
        if self.diagonal_spell_popup() {
            measured_w + self.diagonal_side_reserve_px(rows) + self.overlay_text_hpad()
        } else {
            measured_w.max(char_grid_w) + 2.0 * pad
        }
    }

    pub(in crate::render) fn spell_has_secondary(&self, labels: &[String]) -> bool {
        !labels.is_empty() && !(self.diagonal_spell_popup() && labels.iter().all(String::is_empty))
    }

    pub(in crate::render) fn measured_spell_primary_fits(&self, available_px: f32) -> bool {
        self.diagonal_spell_popup()
            && self.overlay_spell_w > 0.0
            && self.overlay_spell_w <= available_px + 0.5
    }
}
