//! One shared presented-time sample and the renderer's bounded activity report.

use super::*;

impl TextPipeline {
    /// Deterministic delta-injection seam used by capture and focused unit
    /// tests. The live App uses `advance_frame`, where every owner receives the
    /// same monotonic sample.
    pub fn advance(&mut self, dt: f32) -> bool {
        let dt = dt.max(0.0);
        self.step_caret(dt);
        self.step_caret_preview(dt);
        self.step_copy_pulse(dt);
        self.step_overlay_juice(dt);
        self.step_fold_chevrons(dt);
        !self.active_activities().is_empty()
    }

    /// Advance every bounded animator from one shared injected sample and
    /// return the exhaustive post-step activity set.
    pub(crate) fn advance_frame(
        &mut self,
        sample: crate::frame_clock::FrameSample,
    ) -> crate::frame_clock::ActivitySet {
        let dt = sample.elapsed_secs();
        self.step_caret(dt);
        self.step_caret_preview(dt);
        self.step_copy_pulse(dt);
        self.step_overlay_juice(dt);
        self.step_fold_chevrons(dt);
        self.active_activities()
    }

    /// Read again after `prepare`, so geometry-time band retargets enter the
    /// same report as animators armed before the frame.
    pub(crate) fn active_activities(&self) -> crate::frame_clock::ActivitySet {
        use crate::frame_clock::{Activity, ActivitySet};
        let mut active = ActivitySet::empty();
        if !crate::motion::reduced() && self.caret.is_active() {
            active.insert(Activity::CaretMotion);
        }
        if !crate::motion::reduced() && self.caret_preview.is_some() {
            active.insert(Activity::CaretPreview);
        }
        if !crate::motion::reduced() && self.copy_pulse_t < 1.0 {
            active.insert(Activity::CopyPulse);
        }
        if !crate::motion::reduced() && self.overlay_enter_t < 1.0 {
            active.insert(Activity::OverlayEntrance);
        }
        if !crate::motion::reduced() && self.juice_live && self.overlay_band_t < 1.0 {
            active.insert(Activity::OverlayBand);
        }
        if self.fold_chevrons_active() {
            active.insert(Activity::FoldChevrons);
        }
        active
    }
}
