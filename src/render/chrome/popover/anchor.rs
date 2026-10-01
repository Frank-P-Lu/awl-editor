//! Placement lifetime for a selection's formatting interaction.

use super::*;

/// Formatting changes the source endpoints and can reflow the selected line.
/// Keep the already-clickable card in place until the interaction or viewport
/// changes; button state and measured labels still update on every frame.
#[derive(Default)]
pub(in crate::render) struct PopoverPlacement {
    viewport: Option<[f32; 4]>,
    scroll: Option<(ScrollPos, f32)>,
    origin: Option<[f32; 2]>,
}

impl PopoverPlacement {
    pub(super) fn origin(
        &mut self,
        viewport: [f32; 4],
        scroll: ScrollPos,
        scroll_top: f32,
    ) -> Option<[f32; 2]> {
        if self.viewport != Some(viewport) {
            self.origin = None;
            self.viewport = Some(viewport);
        } else if let Some((old_scroll, old_top)) = self.scroll
            && old_scroll != scroll
            && let Some(origin) = &mut self.origin
        {
            // Follow actual scrolling, not row-height changes from formatting.
            origin[1] += old_top - scroll_top;
        }
        self.scroll = Some((scroll, scroll_top));
        self.origin
    }

    pub(super) fn remember(&mut self, card: [f32; 4]) {
        // Keep the unclamped origin so scrolling back reverses an edge clamp.
        self.origin.get_or_insert([card[0], card[1]]);
    }
}

impl TextPipeline {
    /// Clear at the view boundary, rather than waiting for paint: a mouse press
    /// can dismiss and its release re-summon before a single frame is prepared.
    pub(in crate::render) fn sync_popover(&mut self, view: &ViewState) {
        let down = view.popover.is_none()
            || view.selection.is_none()
            || view.overlay_active
            || view.search_active;
        let new_selection = !down
            && self.popover_model.is_some()
            && self.selection != view.selection
            && self.shaped_key.as_deref() == Some(view.text.as_str());
        if down || new_selection {
            self.popover_placement = PopoverPlacement::default();
            self.popover_geom = None;
            self.popover_hover = None;
        }
        self.popover_model = if down { None } else { view.popover.clone() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popover_scroll_tracking_reverses_clamps_and_ignores_edit_reflow() {
        let _guard = crate::testlock::serial();
        let mut placement = PopoverPlacement::default();
        let viewport = [1200.0, 800.0, 1.0, 1.0];
        assert_eq!(placement.origin(viewport, ScrollPos::at_row(2), 64.0), None);
        placement.remember([100.0, 100.0, 200.0, 30.0]);
        // Formatting enlarged earlier rows without moving the semantic scroll.
        assert_eq!(
            placement.origin(viewport, ScrollPos::at_row(2), 96.0),
            Some([100.0, 100.0])
        );
        assert_eq!(
            placement.origin(viewport, ScrollPos::at_row(6), 256.0),
            Some([100.0, -60.0])
        );
        placement.remember([100.0, 8.0, 200.0, 30.0]);
        assert_eq!(
            placement.origin(viewport, ScrollPos::at_row(2), 96.0),
            Some([100.0, 100.0])
        );
        assert_eq!(
            placement.origin([560.0, 800.0, 1.0, 1.0], ScrollPos::at_row(2), 96.0),
            None
        );
    }
}
