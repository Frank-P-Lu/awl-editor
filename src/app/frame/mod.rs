//! The live frame owner.
//!
//! Frame timing, presentation bookkeeping, render-affecting state, and the
//! notice lifetime are one lifecycle: input arms work, the idle poll settles
//! it, and a presented frame retires it.  Keeping those facts behind one
//! handle prevents the former render/scheduler field bags from drifting apart.

use super::*;

#[cfg(not(target_arch = "wasm32"))]
mod accessibility;
mod poll;
mod presentation;
mod surface;
#[cfg(not(target_arch = "wasm32"))]
use accessibility::AccessibilityRuntime;
use poll::{Deadlines, NoticeState};
use presentation::{DebugPanelSnapshot, PresentationState};
use surface::SurfaceState;

pub(in crate::app) struct FrameRuntime {
    surface: SurfaceState,
    presentation: PresentationState,
    deadlines: Deadlines,
    notice: NoticeState,
    #[cfg(not(target_arch = "wasm32"))]
    accessibility: AccessibilityRuntime,
}

pub(in crate::app) enum GpuRebuildStart {
    AlreadyRunning,
    NoWindow,
    Ready(Arc<Window>),
}

#[derive(Clone, Copy)]
pub(in crate::app) enum SettleKind {
    Resize,
    Move,
    Crossing,
}

#[derive(Clone, Copy)]
pub(in crate::app) struct SettleSnapshot {
    pub(in crate::app) resize_at: Option<Instant>,
    pub(in crate::app) move_at: Option<Instant>,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(in crate::app) crossing_at: Option<Instant>,
    pub(in crate::app) crossing_teardown_pending: bool,
}

impl FrameRuntime {
    pub(in crate::app) fn new(zoom: f32, clock: Box<dyn crate::clock::Clock>) -> Self {
        Self {
            surface: SurfaceState::new(),
            presentation: PresentationState {
                clock: crate::frame_clock::FrameClock::default(),
                frame_costs: crate::debug::CostRing::default(),
                theme_switches: crate::themeswitch::SwitchHistory::default(),
                input_stamp: None,
                animation_input_at: None,
                animation_seen: false,
                last_latency_ms: None,
                redraw_count: 0,
                debug_still: crate::debug::DebugStill::Active,
                zoom,
                dpi: 1.0,
                zoom_reflow: ZoomReflow::default(),
                zoom_anchor: None,
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
                occluded: false,
            },
            notice: NoticeState::default(),
            #[cfg(not(target_arch = "wasm32"))]
            accessibility: AccessibilityRuntime::new(),
        }
    }

    pub(in crate::app) fn gpu(&self) -> Option<&Gpu> {
        self.surface.gpu()
    }

    pub(in crate::app) fn gpu_mut(&mut self) -> Option<&mut Gpu> {
        self.surface.gpu_mut()
    }

    pub(in crate::app) fn activate_gpu(&mut self, gpu: Gpu) {
        self.surface.install_gpu(gpu);
        self.surface
            .set_lifecycle(GpuLifecycle::Active { oom_skips: 0 });
        self.surface.clear_retry();
        self.surface.clear_timeout_streak();
    }

    pub(in crate::app) fn gpu_presented(&mut self) {
        self.surface
            .set_lifecycle(GpuLifecycle::Active { oom_skips: 0 });
        self.surface.clear_retry();
        self.surface.clear_timeout_streak();
    }

    pub(in crate::app) fn gpu_memory_pressure(&mut self) {
        self.surface
            .set_lifecycle(GpuLifecycle::Active { oom_skips: 1 });
    }

    #[cfg(target_arch = "wasm32")]
    pub(in crate::app) fn await_gpu(&mut self) {
        self.surface.set_lifecycle(GpuLifecycle::Rebuilding);
    }

    pub(in crate::app) fn recovery_window(&self) -> Option<&Arc<Window>> {
        self.surface.recovery_window()
    }

    pub(in crate::app) fn bind_window(&mut self, window: Arc<Window>) {
        self.surface.set_recovery_window(window);
    }

    pub(in crate::app) fn begin_gpu_rebuild(&mut self) -> GpuRebuildStart {
        if self.surface.lifecycle() == GpuLifecycle::Rebuilding {
            return GpuRebuildStart::AlreadyRunning;
        }
        let Some(window) = self.surface.recovery_window().cloned() else {
            return GpuRebuildStart::NoWindow;
        };
        self.surface.clear_gpu();
        self.surface.set_lifecycle(GpuLifecycle::Rebuilding);
        self.surface.clear_retry();
        self.surface.clear_timeout_streak();
        self.presentation.clock.park();
        self.presentation.input_stamp = None;
        GpuRebuildStart::Ready(window)
    }

    pub(in crate::app) fn gpu_fault_action(&self, kind: gpu::GpuFaultKind) -> GpuFaultAction {
        gpu_fault_action(self.surface.lifecycle(), kind)
    }

    pub(in crate::app) fn gpu_skipped(&mut self, skip: gpu::GpuFrameSkip) -> GpuSkipAction {
        let action = gpu_skip_action(skip, self.surface.timeout_streak());
        self.surface
            .record_timeout(skip == gpu::GpuFrameSkip::Timeout);
        action
    }

    pub(in crate::app) fn retry_gpu_after(&mut self, now: Instant, delay: Duration) {
        self.surface.arm_retry(now + delay);
    }

    pub(in crate::app) fn wait_for_gpu_wake(&mut self) {
        self.surface.clear_retry();
    }

    #[cfg(test)]
    pub(in crate::app) fn invalidate_present_sync(&mut self) {
        self.surface.invalidate_present_sync();
    }

    pub(in crate::app) fn apply_present_sync(&mut self, want: bool) -> bool {
        self.surface.apply_present_sync(want)
    }

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(in crate::app) fn present_sync_on(&self) -> bool {
        self.surface.present_sync_on()
    }

    #[cfg(test)]
    pub(in crate::app) fn present_sync_valid(&self) -> bool {
        self.surface.present_sync_valid()
    }

    pub(in crate::app) fn suspend_surface(&mut self) {
        self.surface.suspend();
        self.suspend();
    }

    #[cfg(target_arch = "wasm32")]
    pub(in crate::app) fn gpu_pending_slot(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<Option<Result<Gpu, String>>>> {
        self.surface.pending_slot()
    }

    #[cfg(target_arch = "wasm32")]
    pub(in crate::app) fn take_gpu_pending(&self) -> Option<Result<Gpu, String>> {
        self.surface.take_pending()
    }

    pub(in crate::app) fn now(&self) -> Instant {
        self.deadlines.clock.now()
    }

    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(in crate::app) fn set_clock(&mut self, clock: Box<dyn crate::clock::Clock>) {
        self.deadlines.clock = clock;
    }

    pub(in crate::app) fn frame_sample(&self, now: Instant) -> crate::frame_clock::FrameSample {
        self.presentation.clock.sample(now)
    }

    pub(in crate::app) fn animation_now(&self, now: Instant) -> Instant {
        self.presentation.clock.sample(now).now
    }

    pub(in crate::app) fn frame_presented(
        &mut self,
        sample: crate::frame_clock::FrameSample,
        activities: crate::frame_clock::ActivitySet,
    ) {
        self.presentation.clock.presented(sample, activities);
    }

    pub(in crate::app) fn park_animations(&mut self) {
        self.presentation.clock.park();
    }

    pub(in crate::app) fn directive(
        &self,
        deadlines: crate::frame_clock::Deadlines,
    ) -> crate::frame_clock::Directive {
        self.presentation.clock.directive(deadlines)
    }

    pub(in crate::app) fn demand_draw_once(&mut self) {
        self.presentation.clock.demand_draw_once();
    }

    pub(in crate::app) fn take_draw_once(&mut self) -> bool {
        self.presentation.clock.take_draw_once()
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

    pub(in crate::app) fn begin_redraw(&mut self) {
        self.presentation.redraw_count += 1;
    }

    pub(in crate::app) fn wake_debug_panel(&mut self, now: Instant) -> DebugPanelSnapshot {
        self.presentation.debug_still = crate::debug::still_wake(
            self.presentation.debug_still,
            self.presentation.input_stamp.is_some(),
        );
        DebugPanelSnapshot {
            cost: self
                .presentation
                .frame_costs
                .last()
                .zip(self.presentation.frame_costs.worst()),
            last_latency_ms: self.presentation.last_latency_ms,
            redraw_count: self.presentation.redraw_count,
            stamp_queued: self.presentation.debug_still == crate::debug::DebugStill::StampQueued,
            theme_settle: self.presentation.theme_switches.report(now),
        }
    }

    pub(in crate::app) fn record_present_cost(
        &mut self,
        cost_ms: f32,
        done: Instant,
        stamp_frame: bool,
    ) {
        if let Some(stamp) = self.presentation.input_stamp.take() {
            self.presentation.last_latency_ms = Some((done - stamp).as_secs_f32() * 1000.0);
        }
        if !stamp_frame {
            self.presentation.frame_costs.push(cost_ms);
        }
    }

    pub(in crate::app) fn record_theme_switch(
        &mut self,
        settled_at: Instant,
        total_ms: f32,
        phases: crate::themeswitch::SwitchPhases,
    ) -> Option<crate::themeswitch::SwitchReport> {
        self.presentation
            .theme_switches
            .insert(settled_at, total_ms, phases);
        self.presentation.theme_switches.report(settled_at)
    }

    pub(in crate::app) fn settle_debug_panel(&mut self, animating: bool) -> bool {
        let (next, request_stamp) =
            crate::debug::still_settle(self.presentation.debug_still, animating);
        self.presentation.debug_still = next;
        request_stamp
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

    pub(in crate::app) fn set_caret_edit_streaks(&mut self, value: bool) {
        self.presentation.caret_edit_streaks = value;
    }

    pub(in crate::app) fn set_caret_held(&mut self, value: bool) {
        self.presentation.caret_held = value;
    }

    pub(in crate::app) fn take_caret_motion_flags(&mut self) -> (bool, bool) {
        (
            std::mem::take(&mut self.presentation.caret_edit_streaks),
            std::mem::take(&mut self.presentation.caret_held),
        )
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

    pub(in crate::app) fn set_occluded(&mut self, occluded: bool) {
        self.deadlines.occluded = occluded;
        if occluded {
            self.presentation.clock.park();
        }
    }

    pub(in crate::app) fn presentation_available(&self) -> bool {
        self.deadlines.focused && !self.deadlines.occluded
    }

    pub(in crate::app) fn set_focused(&mut self, focused: bool) {
        self.deadlines.focused = focused;
    }

    #[cfg(test)]
    pub(in crate::app) fn lava_tick_at(&self) -> Option<Instant> {
        self.deadlines.lava_tick_at
    }

    #[cfg(test)]
    pub(in crate::app) fn arm_lava_tick(&mut self, now: Instant) {
        self.deadlines.lava_tick_at = Some(now);
    }

    pub(in crate::app) fn clear_lava_tick(&mut self) {
        self.deadlines.lava_tick_at = None;
    }

    pub(in crate::app) fn settles(&self) -> SettleSnapshot {
        SettleSnapshot {
            resize_at: self.deadlines.resize_settle_at,
            move_at: self.deadlines.move_settle_at,
            crossing_at: self.deadlines.crossing_settle_at,
            crossing_teardown_pending: self.deadlines.crossing_teardown_pending,
        }
    }

    pub(in crate::app) fn arm_settle(&mut self, kind: SettleKind, now: Instant) {
        match kind {
            SettleKind::Resize => self.deadlines.resize_settle_at = Some(now),
            SettleKind::Move => self.deadlines.move_settle_at = Some(now),
            SettleKind::Crossing => self.deadlines.crossing_settle_at = Some(now),
        }
    }

    #[cfg(test)]
    pub(in crate::app) fn clear_settle(&mut self, kind: SettleKind) {
        match kind {
            SettleKind::Resize => self.deadlines.resize_settle_at = None,
            SettleKind::Move => self.deadlines.move_settle_at = None,
            SettleKind::Crossing => self.deadlines.crossing_settle_at = None,
        }
    }

    #[cfg(test)]
    pub(in crate::app) fn begin_crossing_teardown(&mut self) {
        self.deadlines.crossing_settle_at = None;
        self.deadlines.crossing_teardown_pending = true;
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
        self.presentation.clock.park();
        self.presentation.input_stamp = None;
        self.deadlines.lava_tick_at = None;
        self.deadlines.resize_settle_at = None;
        self.deadlines.move_settle_at = None;
        self.deadlines.crossing_settle_at = None;
        self.deadlines.crossing_teardown_pending = false;
    }
}
