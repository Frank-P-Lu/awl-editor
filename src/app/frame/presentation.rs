use super::*;

pub(super) struct PresentationState {
    pub(super) clock: crate::frame_clock::FrameClock,
    pub(super) frame_costs: crate::debug::CostRing,
    pub(super) theme_switches: crate::themeswitch::SwitchHistory,
    pub(super) input_stamp: Option<Instant>,
    pub(super) animation_input_at: Option<Instant>,
    pub(super) animation_seen: bool,
    pub(super) last_latency_ms: Option<f32>,
    pub(super) redraw_count: u64,
    pub(super) debug_still: crate::debug::DebugStill,
    pub(super) zoom: f32,
    pub(super) dpi: f32,
    pub(super) zoom_reflow: ZoomReflow,
    pub(super) zoom_anchor: Option<ZoomAnchor>,
    pub(super) theme_switch_at: Option<Instant>,
    pub(super) theme_settle: Option<ThemeSettleInFlight>,
    pub(super) caret_edit_streaks: bool,
    pub(super) caret_held: bool,
    pub(super) caret_impact: Option<CaretImpact>,
    pub(super) caret_recoil: Option<crate::caret::RecoilDir>,
}

pub(in crate::app) struct DebugPanelSnapshot {
    pub(in crate::app) cost: Option<(f32, f32)>,
    pub(in crate::app) last_latency_ms: Option<f32>,
    pub(in crate::app) redraw_count: u64,
    pub(in crate::app) stamp_queued: bool,
    pub(in crate::app) theme_settle: Option<crate::themeswitch::SwitchReport>,
}

impl FrameRuntime {
    pub(in crate::app) fn stamp_input_if_absent(&mut self, now: Instant) {
        self.presentation.input_stamp.get_or_insert(now);
    }

    pub(in crate::app) fn stamp_animation_input_if_absent(&mut self, now: Instant) {
        self.presentation.animation_input_at.get_or_insert(now);
    }

    /// Close the input-to-animation-settled interval only after a visible,
    /// input-bounded activity was observed. The travelling ground is ambient:
    /// keeping a Kite window open must not make an ordinary editing gesture's
    /// bounded animation appear never to settle.
    pub(in crate::app) fn animation_settled(
        &mut self,
        now: Instant,
        activities: crate::frame_clock::ActivitySet,
    ) -> Option<Duration> {
        let bounded_active = activities
            .iter()
            .any(|activity| activity != crate::frame_clock::Activity::TravellingGround);
        if bounded_active {
            self.presentation.animation_seen = true;
            return None;
        }
        if self.presentation.animation_seen {
            self.presentation.animation_seen = false;
            return self
                .presentation
                .animation_input_at
                .take()
                .map(|input| now.saturating_duration_since(input));
        }
        self.presentation.animation_input_at = None;
        None
    }
}
