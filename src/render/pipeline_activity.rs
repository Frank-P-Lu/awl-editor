//! One shared presented-time sample and the renderer's bounded activity report.

use super::*;

impl crate::frame_clock::Activity {
    fn advance_owner(self, pipeline: &mut TextPipeline, dt: f32, travelling_ground: bool) {
        match self {
            Self::CaretMotion => {
                pipeline.step_caret(dt);
            }
            Self::CaretPreview => {
                pipeline.step_caret_preview(dt);
            }
            Self::CopyPulse => {
                pipeline.step_copy_pulse(dt);
            }
            Self::OverlayEntrance => {
                pipeline.step_overlay_entrance(dt);
            }
            Self::OverlayBand => {
                pipeline.step_overlay_band(dt);
            }
            Self::FoldChevrons => {
                pipeline.step_fold_chevrons(dt);
            }
            Self::TravellingGround => {
                if travelling_ground {
                    pipeline.advance_warp(crate::lava::ambient_tick_dt(dt));
                }
            }
        }
    }

    fn active_at_owner(self, pipeline: &TextPipeline, travelling_ground: bool) -> bool {
        match self {
            Self::CaretMotion => !crate::motion::reduced() && pipeline.caret.is_active(),
            Self::CaretPreview => !crate::motion::reduced() && pipeline.caret_preview.is_some(),
            Self::CopyPulse => !crate::motion::reduced() && pipeline.copy_pulse_t < 1.0,
            Self::OverlayEntrance => !crate::motion::reduced() && pipeline.overlay_enter_t < 1.0,
            Self::OverlayBand => {
                !crate::motion::reduced() && pipeline.juice_live && pipeline.overlay_band_t < 1.0
            }
            Self::FoldChevrons => pipeline.fold_chevrons_active(),
            Self::TravellingGround => travelling_ground,
        }
    }
}

impl TextPipeline {
    /// Deterministic delta-injection seam used by capture and focused unit
    /// tests. The live App uses `advance_frame`, where every owner receives the
    /// same monotonic sample.
    pub fn advance(&mut self, dt: f32) -> bool {
        !self.advance_owners(dt.max(0.0), false).is_empty()
    }

    /// Advance every bounded animator from one shared injected sample and
    /// return the exhaustive post-step activity set.
    pub(crate) fn advance_frame(
        &mut self,
        sample: crate::frame_clock::FrameSample,
        travelling_ground: bool,
    ) -> crate::frame_clock::ActivitySet {
        self.advance_owners(sample.elapsed_secs(), travelling_ground)
    }

    fn advance_owners(
        &mut self,
        dt: f32,
        travelling_ground: bool,
    ) -> crate::frame_clock::ActivitySet {
        for activity in crate::frame_clock::Activity::ALL {
            activity.advance_owner(self, dt, travelling_ground);
        }
        self.active_activities(travelling_ground)
    }

    /// Read again after `prepare`, so geometry-time band retargets enter the
    /// same report as animators armed before the frame.
    pub(crate) fn active_activities(
        &self,
        travelling_ground: bool,
    ) -> crate::frame_clock::ActivitySet {
        use crate::frame_clock::{Activity, ActivitySet};
        let mut active = ActivitySet::empty();
        for activity in Activity::ALL {
            if activity.active_at_owner(self, travelling_ground) {
                active.insert(activity);
            }
        }
        active
    }

    #[cfg(test)]
    pub(in crate::render) fn arm_activity_law(
        &mut self,
        activity: crate::frame_clock::Activity,
    ) -> bool {
        use crate::frame_clock::Activity;
        match activity {
            Activity::CaretMotion => self.inject_motion_demo(),
            Activity::CaretPreview => {
                self.caret_preview = Some(crate::caret::CaretMode::Block);
                self.caret_demo
                    .set_metrics(self.metrics.char_width, self.metrics.line_height);
            }
            Activity::CopyPulse => self.copy_pulse(),
            Activity::OverlayEntrance => {
                self.arm_live_juice();
                self.overlay_enter_t = 0.0;
            }
            Activity::OverlayBand => {
                self.arm_live_juice();
                self.overlay_band_started_at = None;
                self.overlay_band_t = 0.0;
            }
            Activity::FoldChevrons => {
                let line = self
                    .outline_headings
                    .first()
                    .map(|heading| heading.line)
                    .unwrap_or_else(|| {
                        self.outline_headings.push(crate::markdown::Heading {
                            level: 1,
                            text: "frame-clock law".to_string(),
                            line: 0,
                        });
                        0
                    });
                self.folded_headings.retain(|folded| *folded != line);
                self.fold_chevron_turn.insert(line, 0.0);
            }
            Activity::TravellingGround => {}
        }
        true
    }

    #[cfg(test)]
    pub(in crate::render) fn activity_law_pose(
        &self,
        activity: crate::frame_clock::Activity,
    ) -> f32 {
        use crate::frame_clock::Activity;
        match activity {
            Activity::CaretMotion => self.caret.pos.x + self.caret.pos.y,
            Activity::CaretPreview => self.caret_demo.beat_index() as f32,
            Activity::CopyPulse => self.copy_pulse_t,
            Activity::OverlayEntrance => self.overlay_enter_t,
            Activity::OverlayBand => self.overlay_band_t,
            Activity::FoldChevrons => self
                .outline_headings
                .first()
                .and_then(|heading| self.fold_chevron_turn.get(&heading.line))
                .copied()
                .unwrap_or(1.0),
            Activity::TravellingGround => self.warp_phase,
        }
    }

    #[cfg(test)]
    pub(in crate::render) fn retire_activity_law(
        &mut self,
        activity: crate::frame_clock::Activity,
    ) -> crate::frame_clock::ActivitySet {
        use crate::frame_clock::Activity;
        match activity {
            Activity::CaretPreview => {
                self.caret_preview = None;
                self.active_activities(false)
            }
            Activity::TravellingGround => self.active_activities(false),
            Activity::CaretMotion
            | Activity::CopyPulse
            | Activity::OverlayEntrance
            | Activity::OverlayBand
            | Activity::FoldChevrons => {
                let mut active = self.active_activities(false);
                for _ in 0..1200 {
                    active = self.advance_owners(1.0 / 120.0, false);
                    if !active.contains(activity) {
                        break;
                    }
                }
                active
            }
        }
    }
}
