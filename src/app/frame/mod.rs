//! The live frame owner.
//!
//! Frame timing, presentation bookkeeping, render-affecting state, and the
//! notice lifetime are one lifecycle: input arms work, the idle poll settles
//! it, and a presented frame retires it.  Keeping those facts behind one
//! handle prevents the former render/scheduler field bags from drifting apart.

use super::*;

pub(in crate::app) struct FrameRuntime {
    presentation: PresentationState,
    deadlines: Deadlines,
    notice: NoticeState,
}

struct PresentationState {
    last_frame: Option<Instant>,
    frame_costs: crate::debug::CostRing,
    theme_switches: crate::themeswitch::SwitchHistory,
    input_stamp: Option<Instant>,
    last_latency_ms: Option<f32>,
    redraw_count: u64,
    debug_still: crate::debug::DebugStill,
    zoom: f32,
    dpi: f32,
    zoom_reflow: ZoomReflow,
    zoom_anchor: Option<ZoomAnchor>,
    theme_font_at: Option<Instant>,
    theme_font_last_reshape_at: Option<Instant>,
    theme_switch_at: Option<Instant>,
    theme_settle: Option<ThemeSettleInFlight>,
    caret_edit_streaks: bool,
    caret_held: bool,
    caret_impact: Option<CaretImpact>,
    caret_recoil: Option<crate::caret::RecoilDir>,
}

struct Deadlines {
    clock: Box<dyn crate::clock::Clock>,
    lava_tick_at: Option<Instant>,
    resize_settle_at: Option<Instant>,
    move_settle_at: Option<Instant>,
    crossing_settle_at: Option<Instant>,
    crossing_teardown_pending: bool,
    zoom_persist_at: Option<Instant>,
    focused: bool,
}

#[derive(Default)]
struct NoticeState {
    text: Option<String>,
    kind: NoticeKind,
    expires_at: Option<Instant>,
}

impl FrameRuntime {
    pub(in crate::app) fn new(zoom: f32, clock: Box<dyn crate::clock::Clock>) -> Self {
        Self {
            presentation: PresentationState {
                last_frame: None,
                frame_costs: crate::debug::CostRing::default(),
                theme_switches: crate::themeswitch::SwitchHistory::default(),
                input_stamp: None,
                last_latency_ms: None,
                redraw_count: 0,
                debug_still: crate::debug::DebugStill::Active,
                zoom,
                dpi: 1.0,
                zoom_reflow: ZoomReflow::default(),
                zoom_anchor: None,
                theme_font_at: None,
                theme_font_last_reshape_at: None,
                theme_switch_at: None,
                theme_settle: None,
                caret_edit_streaks: false,
                caret_held: false,
                caret_impact: None,
                caret_recoil: None,
            },
            deadlines: Deadlines {
                clock,
                lava_tick_at: None,
                resize_settle_at: None,
                move_settle_at: None,
                crossing_settle_at: None,
                crossing_teardown_pending: false,
                zoom_persist_at: None,
                focused: true,
            },
            notice: NoticeState::default(),
        }
    }

    pub(in crate::app) fn now(&self) -> Instant {
        self.deadlines.clock.now()
    }

    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(in crate::app) fn set_clock(&mut self, clock: Box<dyn crate::clock::Clock>) {
        self.deadlines.clock = clock;
    }

    pub(in crate::app) fn last_frame(&self) -> Option<Instant> {
        self.presentation.last_frame
    }

    pub(in crate::app) fn set_last_frame(&mut self, value: Option<Instant>) {
        self.presentation.last_frame = value;
    }

    pub(in crate::app) fn frame_is_hot(&self) -> bool {
        self.presentation.last_frame.is_some()
    }

    pub(in crate::app) fn zoom(&self) -> f32 {
        self.presentation.zoom
    }

    pub(in crate::app) fn set_zoom(&mut self, zoom: f32) {
        self.presentation.zoom = zoom;
    }

    pub(in crate::app) fn dpi(&self) -> f32 {
        self.presentation.dpi
    }

    pub(in crate::app) fn set_dpi(&mut self, dpi: f32) {
        self.presentation.dpi = dpi;
    }

    pub(in crate::app) fn queue_zoom_reflow(&mut self) {
        self.presentation.zoom_reflow.queue();
    }

    pub(in crate::app) fn take_zoom_reflow(&mut self) -> bool {
        self.presentation.zoom_reflow.take()
    }

    pub(in crate::app) fn clear_zoom_reflow(&mut self) {
        self.presentation.zoom_reflow.clear();
    }

    pub(in crate::app) fn set_zoom_anchor(&mut self, anchor: ZoomAnchor) {
        self.presentation.zoom_anchor = Some(anchor);
    }

    pub(in crate::app) fn take_zoom_anchor(&mut self) -> Option<ZoomAnchor> {
        self.presentation.zoom_anchor.take()
    }

    pub(in crate::app) fn zoom_persist_at(&self) -> Option<Instant> {
        self.deadlines.zoom_persist_at
    }

    pub(in crate::app) fn arm_zoom_persist(&mut self, now: Instant) {
        self.deadlines.zoom_persist_at = Some(now);
    }

    pub(in crate::app) fn clear_zoom_persist(&mut self) {
        self.deadlines.zoom_persist_at = None;
    }

    pub(in crate::app) fn theme_font_at(&self) -> Option<Instant> {
        self.presentation.theme_font_at
    }

    pub(in crate::app) fn arm_theme_font(&mut self, now: Instant) {
        self.presentation.theme_font_at = Some(now);
    }

    pub(in crate::app) fn clear_theme_font(&mut self) {
        self.presentation.theme_font_at = None;
    }

    pub(in crate::app) fn theme_font_last_reshape_at(&self) -> Option<Instant> {
        self.presentation.theme_font_last_reshape_at
    }

    pub(in crate::app) fn mark_theme_font_reshaped(&mut self, now: Instant) {
        self.presentation.theme_font_last_reshape_at = Some(now);
    }

    pub(in crate::app) fn stamp_theme_switch(&mut self, now: Instant) {
        self.presentation.theme_switch_at = Some(now);
    }

    pub(in crate::app) fn theme_switch_at(&self) -> Option<Instant> {
        self.presentation.theme_switch_at
    }

    pub(in crate::app) fn set_theme_settle(&mut self, settle: Option<ThemeSettleInFlight>) {
        self.presentation.theme_settle = settle;
    }

    pub(in crate::app) fn theme_settle_pending(&self) -> bool {
        self.presentation.theme_settle.is_some()
    }

    pub(in crate::app) fn take_theme_settle(&mut self) -> Option<ThemeSettleInFlight> {
        self.presentation.theme_settle.take()
    }

    pub(in crate::app) fn theme_switches_mut(&mut self) -> &mut crate::themeswitch::SwitchHistory {
        &mut self.presentation.theme_switches
    }

    pub(in crate::app) fn frame_costs(&self) -> &crate::debug::CostRing {
        &self.presentation.frame_costs
    }

    pub(in crate::app) fn frame_costs_mut(&mut self) -> &mut crate::debug::CostRing {
        &mut self.presentation.frame_costs
    }

    pub(in crate::app) fn input_stamp(&self) -> Option<Instant> {
        self.presentation.input_stamp
    }

    pub(in crate::app) fn stamp_input_if_absent(&mut self, now: Instant) {
        self.presentation.input_stamp.get_or_insert(now);
    }

    pub(in crate::app) fn take_input_stamp(&mut self) -> Option<Instant> {
        self.presentation.input_stamp.take()
    }

    pub(in crate::app) fn last_latency_ms(&self) -> Option<f32> {
        self.presentation.last_latency_ms
    }

    pub(in crate::app) fn set_last_latency_ms(&mut self, value: Option<f32>) {
        self.presentation.last_latency_ms = value;
    }

    pub(in crate::app) fn next_redraw_count(&mut self) -> u64 {
        self.presentation.redraw_count += 1;
        self.presentation.redraw_count
    }

    pub(in crate::app) fn redraw_count(&self) -> u64 {
        self.presentation.redraw_count
    }

    pub(in crate::app) fn debug_still(&self) -> crate::debug::DebugStill {
        self.presentation.debug_still
    }

    pub(in crate::app) fn set_debug_still(&mut self, value: crate::debug::DebugStill) {
        self.presentation.debug_still = value;
    }

    pub(in crate::app) fn clear_debug_session(&mut self) {
        self.presentation.input_stamp = None;
        self.presentation.last_latency_ms = None;
        self.presentation.frame_costs.clear();
        self.presentation.theme_switches.clear();
        self.presentation.debug_still = crate::debug::DebugStill::Active;
    }

    pub(in crate::app) fn debug_session_populated(&self) -> bool {
        self.presentation.input_stamp.is_some()
            || self.presentation.last_latency_ms.is_some()
            || self.presentation.frame_costs.last().is_some()
            || !self.presentation.theme_switches.is_empty()
    }

    pub(in crate::app) fn caret_edit_streaks(&self) -> bool {
        self.presentation.caret_edit_streaks
    }

    pub(in crate::app) fn set_caret_edit_streaks(&mut self, value: bool) {
        self.presentation.caret_edit_streaks = value;
    }

    pub(in crate::app) fn caret_held(&self) -> bool {
        self.presentation.caret_held
    }

    pub(in crate::app) fn set_caret_held(&mut self, value: bool) {
        self.presentation.caret_held = value;
    }

    pub(in crate::app) fn set_caret_impact(&mut self, impact: Option<CaretImpact>) {
        self.presentation.caret_impact = impact;
    }

    pub(in crate::app) fn take_caret_impact(&mut self) -> Option<CaretImpact> {
        self.presentation.caret_impact.take()
    }

    pub(in crate::app) fn set_caret_recoil(&mut self, recoil: Option<crate::caret::RecoilDir>) {
        self.presentation.caret_recoil = recoil;
    }

    pub(in crate::app) fn take_caret_recoil(&mut self) -> Option<crate::caret::RecoilDir> {
        self.presentation.caret_recoil.take()
    }

    pub(in crate::app) fn focused(&self) -> bool {
        self.deadlines.focused
    }

    pub(in crate::app) fn set_focused(&mut self, focused: bool) {
        self.deadlines.focused = focused;
    }

    pub(in crate::app) fn lava_tick_at(&self) -> Option<Instant> {
        self.deadlines.lava_tick_at
    }

    pub(in crate::app) fn arm_lava_tick(&mut self, now: Instant) {
        self.deadlines.lava_tick_at = Some(now);
    }

    pub(in crate::app) fn lava_tick_at_or_arm(&mut self, now: Instant) -> Instant {
        *self.deadlines.lava_tick_at.get_or_insert(now)
    }

    pub(in crate::app) fn clear_lava_tick(&mut self) {
        self.deadlines.lava_tick_at = None;
    }

    pub(in crate::app) fn resize_settle_at(&self) -> Option<Instant> {
        self.deadlines.resize_settle_at
    }

    pub(in crate::app) fn arm_resize_settle(&mut self, now: Instant) {
        self.deadlines.resize_settle_at = Some(now);
    }

    pub(in crate::app) fn clear_resize_settle(&mut self) {
        self.deadlines.resize_settle_at = None;
    }

    pub(in crate::app) fn move_settle_at(&self) -> Option<Instant> {
        self.deadlines.move_settle_at
    }

    pub(in crate::app) fn arm_move_settle(&mut self, now: Instant) {
        self.deadlines.move_settle_at = Some(now);
    }

    pub(in crate::app) fn clear_move_settle(&mut self) {
        self.deadlines.move_settle_at = None;
    }

    pub(in crate::app) fn crossing_settle_at(&self) -> Option<Instant> {
        self.deadlines.crossing_settle_at
    }

    pub(in crate::app) fn arm_crossing_settle(&mut self, now: Instant) {
        self.deadlines.crossing_settle_at = Some(now);
    }

    #[cfg(test)]
    pub(in crate::app) fn clear_crossing_settle(&mut self) {
        self.deadlines.crossing_settle_at = None;
    }

    pub(in crate::app) fn begin_crossing_teardown(&mut self) {
        self.deadlines.crossing_settle_at = None;
        self.deadlines.crossing_teardown_pending = true;
    }

    pub(in crate::app) fn crossing_teardown_pending(&self) -> bool {
        self.deadlines.crossing_teardown_pending
    }

    pub(in crate::app) fn finish_crossing_teardown(&mut self) {
        self.deadlines.crossing_teardown_pending = false;
    }

    pub(in crate::app) fn present_sync_sources(&self) -> (bool, bool, bool) {
        (
            self.deadlines.resize_settle_at.is_some(),
            self.deadlines.move_settle_at.is_some(),
            self.deadlines.crossing_settle_at.is_some() || self.deadlines.crossing_teardown_pending,
        )
    }

    pub(in crate::app) fn suspend(&mut self) {
        self.presentation.last_frame = None;
        self.deadlines.lava_tick_at = None;
        self.deadlines.resize_settle_at = None;
        self.deadlines.move_settle_at = None;
        self.deadlines.crossing_settle_at = None;
        self.deadlines.crossing_teardown_pending = false;
    }

    pub(in crate::app) fn set_sticky_notice(&mut self, text: String) {
        self.notice.text = Some(text);
        self.notice.kind = NoticeKind::Sticky;
        self.notice.expires_at = None;
    }

    pub(in crate::app) fn set_toast_notice(&mut self, text: String, expires_at: Option<Instant>) {
        self.notice.text = Some(text);
        self.notice.kind = NoticeKind::Toast;
        self.notice.expires_at = expires_at;
    }

    pub(in crate::app) fn clear_notice(&mut self) {
        self.notice = NoticeState::default();
    }

    pub(in crate::app) fn notice_text(&self) -> Option<&str> {
        self.notice.text.as_deref()
    }

    pub(in crate::app) fn notice_owned(&self) -> Option<String> {
        self.notice.text.clone()
    }

    pub(in crate::app) fn notice_active(&self) -> bool {
        self.notice.text.is_some()
    }

    pub(in crate::app) fn notice_kind(&self) -> NoticeKind {
        self.notice.kind
    }

    pub(in crate::app) fn notice_expires_at(&self) -> Option<Instant> {
        self.notice.expires_at
    }
}
