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
