use super::OverlayState;

impl OverlayState {
    /// Passive pointer travel may take selection only when that selection is
    /// itself the picker's live preview. Ordinary rows keep the keyboard's
    /// choice stable; the pointing-hand cursor is their hover acknowledgement.
    pub fn preview_hover_at(&mut self, px: f32, py: f32, hit: Option<usize>) -> bool {
        if self.kind.previews_live_document() {
            self.hover_at(px, py, hit)
        } else {
            // Still advance the real-motion anchor: a later previewing card
            // must not inherit stale pointer travel from this one.
            self.hover_at(px, py, None);
            false
        }
    }
}
