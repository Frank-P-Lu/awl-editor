//! Input-anchored selection-band timing for live theme previews.
//!
//! The App supplies both the input epoch and presented time; the complete epoch
//! itself belongs to `OverlayBandState`, so no caller can clear only one part.

use super::*;

impl TextPipeline {
    /// Stamp a theme-picker movement before its synchronous preview work.
    pub(crate) fn stamp_overlay_movement(&mut self, movement_at: crate::clock::Instant) {
        if self.juice_live && !crate::motion::reduced() {
            self.overlay_band.stamp_input(movement_at);
        }
    }

    /// Supply the redraw's injected monotonic presentation time.
    pub(crate) fn begin_overlay_frame(&mut self, now: crate::clock::Instant) {
        self.overlay_band.begin_frame(now);
    }
}
