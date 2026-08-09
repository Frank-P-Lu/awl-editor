//! Palette-role synchronization for the two-colour selection pipelines.

use super::*;

impl TextPipeline {
    pub(super) fn sync_two_colour_pipelines(&mut self) {
        let active = theme::active();
        if let Some(pair) = active.render_caps.selection_style.two_colour(&active) {
            self.selection_invert
                .set_two_colour(pair.ground.rgba_bytes(), pair.ink.rgba_bytes());
        }
        if let Some(pair) = active.render_caps.caret_block_style.two_colour(&active) {
            self.caret_invert
                .set_two_colour(pair.ground.rgba_bytes(), pair.ink.rgba_bytes());
        }
    }
}
