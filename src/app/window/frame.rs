//! The live redraw's preparation and diagnostic feed.

use super::*;

impl App {
    pub(super) fn prepare_live_frame(
        &mut self,
        sample: crate::frame_clock::FrameSample,
    ) -> Option<(gpu::PreparedFrame, bool)> {
        let config = self.config.scheduling_snapshot();
        let presentation_available = self.frame.presentation_available();
        let travelling_ground = crate::warpgrid::should_travel(
            config.ambient_motion_on(),
            crate::motion::reduced(),
            presentation_available,
            crate::lava::lava_paused(
                self.frame.settles().resize_at.is_some(),
                self.frame.settles().move_at.is_some(),
                self.frame
                    .gpu()
                    .is_some_and(|gpu| gpu.pipeline.lava_blur_active()),
            ),
        );
        let gpu = self.frame.gpu_mut()?;
        // Input-anchored band timing and every bounded animator share this
        // sample. An unavailable presentation retains every visible pose.
        if presentation_available {
            gpu.pipeline.begin_overlay_frame(sample.now);
            gpu.pipeline.advance_frame(sample, travelling_ground);
        }
        Some((
            gpu.redraw(presentation_available.then_some(travelling_ground)),
            presentation_available,
        ))
    }

    /// Feed the debug panel at the top of a real redraw and say whether this is
    /// its one settle-stamp frame. The panel never creates the work it measures.
    pub(super) fn feed_debug_panel(&mut self, now: Instant) -> bool {
        let mut is_stamp = false;
        if crate::debug::debug_on() {
            let debug = self.frame.wake_debug_panel(now);
            is_stamp = debug.stamp_queued;
            let engine_wrote = self.persistence.engine_last_write_at();
            let since_secs = engine_wrote.map(|t| (now - t).as_secs());
            let autosave = crate::debug::autosave_state(
                self.config.autosave_on(),
                self.frame.notice().active(),
                since_secs,
            );
            if let Some(gpu) = self.frame.gpu_mut() {
                let budget = crate::debug::budget_ms(
                    gpu.window
                        .current_monitor()
                        .and_then(|m| m.refresh_rate_millihertz()),
                );
                gpu.pipeline.set_debug_perf(
                    debug.cost,
                    debug.last_latency_ms,
                    Some(debug.redraw_count),
                    is_stamp,
                    Some(budget),
                );
                gpu.pipeline.set_debug_gpu_bytes(gpu.current_gpu_bytes());
                gpu.pipeline.set_debug_autosave(Some(autosave));
                gpu.pipeline.set_debug_theme_settle(debug.theme_settle);
            }
        } else if self.clear_debug_session_if_populated()
            && let Some(gpu) = self.frame.gpu_mut()
        {
            gpu.pipeline.set_debug_perf(None, None, None, true, None);
            gpu.pipeline.set_debug_autosave(None);
            gpu.pipeline.set_debug_theme_settle(None);
        }
        is_stamp
    }
}
