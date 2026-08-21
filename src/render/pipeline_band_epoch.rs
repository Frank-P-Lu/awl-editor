//! Input-anchored selection-band timing for live theme previews.
//!
//! Geometry still resolves inside `prepare`, but the epoch comes from the App's
//! input seam. Keeping that bridge here leaves `pipeline_overlay` as the shared
//! choreography owner while making the clockless-capture boundary explicit.

use super::*;

impl TextPipeline {
    fn band_progress_since(started_at: crate::clock::Instant, now: crate::clock::Instant) -> f32 {
        OVERLAY_BAND_SLIDE_MS
            .progress_per(now.saturating_duration_since(started_at).as_secs_f32())
            .min(1.0)
    }

    fn sample_anchored_band(&mut self, now: crate::clock::Instant) {
        let Some(started_at) = self.overlay_band_started_at else {
            return;
        };
        self.overlay_band_t = Self::band_progress_since(started_at, now);
        if self.overlay_band_t >= 1.0 {
            self.overlay_band_started_at = None;
        }
    }

    fn current_band_top(&self) -> f32 {
        let Some(last) = self.overlay_band_last else {
            return self.overlay_band_from;
        };
        if self.overlay_band_t >= 1.0 {
            last
        } else {
            let e = crate::ease::out_back(self.overlay_band_t);
            self.overlay_band_from + (last - self.overlay_band_from) * e
        }
    }

    /// Stamp a theme-picker movement before its synchronous preview work. The
    /// renderer does not read a clock: the live App supplies both this epoch and
    /// [`Self::begin_overlay_frame`]'s clock-owned visible `now`. The epoch
    /// consumes synchronous preview work while a drawable surface is live but
    /// remains frozen across parked wall-clock gaps. An ordinary capture never
    /// calls either method and remains structurally settled.
    pub(crate) fn stamp_overlay_movement(&mut self, movement_at: crate::clock::Instant) {
        if !self.juice_live || crate::motion::reduced() {
            return;
        }
        // A rapid retarget samples the old pose at the input instant before the
        // latest-selection-wins policy is chosen. A second input arriving before
        // prepare counts as rapid too: the first pending destination must never
        // become a stale intermediate target.
        self.sample_anchored_band(movement_at);
        self.overlay_band_pending_from = self.current_band_top();
        self.overlay_band_pending_snap =
            self.overlay_band_pending_at.is_some() || self.overlay_band_t < 1.0;
        self.overlay_band_pending_at = Some(movement_at);
    }

    /// Supply the redraw's injected monotonic time to the band animator. This is
    /// called before `advance` and `prepare`, so both the old pose and the new
    /// target resolved inside `prepare` are sampled at one coherent instant.
    pub(crate) fn begin_overlay_frame(&mut self, now: crate::clock::Instant) {
        self.overlay_band_frame_now = Some(now);
        self.sample_anchored_band(now);
    }

    /// Consume a pending input epoch once `prepare` resolves its selected row.
    /// Returns true when an epoch existed, even if geometry did not move, so the
    /// legacy prepare-anchored path cannot restart the same input afterward.
    pub(super) fn consume_overlay_movement(&mut self, target: f32) -> bool {
        let Some(movement_at) = self.overlay_band_pending_at.take() else {
            return false;
        };
        let Some(last) = self.overlay_band_last else {
            // Opening a picker has no prior selected row to travel from.
            self.overlay_band_from = target;
            self.overlay_band_last = Some(target);
            self.overlay_band_t = 1.0;
            self.overlay_band_started_at = None;
            return true;
        };
        if (last - target).abs() <= 0.5 {
            return true;
        }

        self.overlay_band_from = if self.overlay_band_pending_snap {
            target
        } else {
            self.overlay_band_pending_from
        };
        self.overlay_band_last = Some(target);
        let now = self.overlay_band_frame_now.unwrap_or(movement_at);
        self.overlay_band_started_at = Some(movement_at);
        self.sample_anchored_band(now);
        true
    }
}
