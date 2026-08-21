use super::*;

#[path = "window/frame.rs"]
mod redraw;

impl App {
    /// Forget the live Debug session when the panel has been turned off. This is the
    /// one explicit reset boundary for frame, input, and theme-transaction diagnostics;
    /// normal theme movement never calls it.
    pub(super) fn clear_debug_session_when_off(&mut self) {
        self.frame.clear_debug_session();
    }

    pub(super) fn clear_debug_session_if_populated(&mut self) -> bool {
        let populated = self.frame.debug_session_populated();
        if populated {
            self.clear_debug_session_when_off();
        }
        populated
    }

    fn handle_gpu_fault(&mut self, event_loop: &ActiveEventLoop, fault: gpu::GpuFault) {
        eprintln!("gpu {:?}: {}", fault.kind, fault.message);
        match self.frame.gpu_fault_action(fault.kind) {
            GpuFaultAction::RetryOneFrame => {
                self.frame.gpu_memory_pressure();
                self.set_sticky_notice("graphics memory pressure — skipped one frame");
                self.request_frame();
            }
            GpuFaultAction::NoticeOnly => {
                self.set_sticky_notice("graphics rejected one frame — editing is safe")
            }
            GpuFaultAction::Rebuild => {
                let reason = match fault.kind {
                    gpu::GpuFaultKind::OutOfMemory => "graphics memory stayed full",
                    gpu::GpuFaultKind::DeviceLost => "graphics device was lost",
                    gpu::GpuFaultKind::Internal => "graphics backend stopped responding",
                    gpu::GpuFaultKind::SurfaceRecoveryFailed => "window surface could not recover",
                    gpu::GpuFaultKind::Validation => "graphics rejected repeated work",
                };
                self.rebuild_gpu(event_loop, reason);
            }
        }
    }

    fn handle_gpu_frame_outcome(
        &mut self,
        event_loop: &ActiveEventLoop,
        outcome: gpu::GpuFrameOutcome,
    ) -> Result<(Option<(f32, Instant)>, bool), ()> {
        match outcome {
            gpu::GpuFrameOutcome::Presented(perf) => {
                self.frame.gpu_presented();
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(soak) = self.soak.as_mut() {
                    soak.observe_frame(crate::soak_gpu::FrameOutcome::Presented, true);
                    if let Some(kind) = self.soak_recovery_pending.take() {
                        soak.observe_recovered(kind, Instant::now());
                    }
                }
                Ok((perf, true))
            }
            gpu::GpuFrameOutcome::Skipped(skip) => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(soak) = self.soak.as_mut() {
                    soak.observe_frame(
                        crate::soak_gpu::FrameOutcome::Skipped(soak_skip_kind(skip)),
                        false,
                    );
                }
                // FLIGHT-RECORDER / PROBE: a frame that DIDN'T present is the vanish
                // signature — the writing surface can only go stale/blank on screen
                // when a scheduled frame is skipped (occluded / timeout / surface
                // lost) while the bracket state is what it is. Log the reason + the
                // live bracket so the black box shows exactly which present was lost.
                #[cfg(not(target_arch = "wasm32"))]
                if crate::probe::recording() {
                    crate::probe::trace(format_args!(
                        "present SKIPPED {skip:?} (txn_on={} crossing={} teardown_pending={})",
                        self.frame.present_sync_on(),
                        self.frame.settles().crossing_at.is_some(),
                        self.frame.settles().crossing_teardown_pending,
                    ));
                }
                let action = self.frame.gpu_skipped(skip);
                match action {
                    GpuSkipAction::WaitForWake => self.frame.wait_for_gpu_wake(),
                    GpuSkipAction::RetryAfter(delay) => {
                        self.frame.retry_gpu_after(self.frame.now(), delay);
                    }
                    GpuSkipAction::RetryWithNoticeAfter(delay, notice) => {
                        self.set_toast_notice(notice);
                        self.frame.retry_gpu_after(self.frame.now(), delay);
                    }
                    GpuSkipAction::HoldWithNotice(notice) => {
                        self.frame.wait_for_gpu_wake();
                        self.set_sticky_notice(notice);
                    }
                }
                Ok((None, false))
            }
            gpu::GpuFrameOutcome::Fault(fault) => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(soak) = self.soak.as_mut() {
                    soak.observe_frame(
                        crate::soak_gpu::FrameOutcome::Skipped(crate::soak_gpu::SkipKind::Fault),
                        false,
                    );
                }
                self.handle_gpu_fault(event_loop, fault);
                Err(())
            }
        }
    }
    /// Close out a theme-switch transaction on the frame that PRESENTED its
    /// reshape: record the three frame-side phases, compute the felt
    /// input→settled total, and hand the report to the panel (a stamp redraw
    /// then draws it). `frame_started` is this frame's own start on the App
    /// clock, which is what makes `SwitchPhase::Schedule` measurable.
    ///
    /// Structurally off the headless path — a capture never arms `theme_settle`,
    /// because only the live App's reshape doors do.
    fn fold_settled_theme_switch(&mut self, frame_started: Instant) {
        use crate::themeswitch::SwitchPhase;
        if !crate::debug::debug_on() || !self.frame.theme_settle_pending() {
            return;
        }
        let mut settle = self
            .frame
            .take_theme_settle()
            .expect("just checked is_some");
        // The SCHEDULE phase: the reshape finished on an earlier turn and this
        // frame — the one carrying it — began at `frame_started`, so the
        // difference is the redraw request's own trip through the event loop.
        // Measured, not derived by subtraction: a residual computed from the
        // total would make the coverage law vacuously true.
        settle.phases.record(
            SwitchPhase::Schedule,
            frame_started
                .saturating_duration_since(settle.work_done_at)
                .as_secs_f32()
                * 1000.0,
        );
        if let Some((prep_ms, acquire_ms, present_ms)) =
            self.frame.gpu().and_then(|g| g.debug_frame_split)
        {
            settle.phases.record(SwitchPhase::Atlas, prep_ms);
            settle.phases.record(SwitchPhase::Acquire, acquire_ms);
            settle.phases.record(SwitchPhase::Present, present_ms);
        }
        // The App clock is the one scheduling/animation time seam. Its timestamp
        // gives this higher-level transaction a deterministic fake-clock test
        // path, while the GPU's own `done` stamp stays reserved for the per-frame
        // measurement its caller already recorded.
        let settled_at = self.frame.now();
        let total_ms = (settled_at - settle.input_at).as_secs_f32() * 1000.0;
        let theme_settle = self
            .frame
            .record_theme_switch(settled_at, total_ms, settle.phases);
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_debug_theme_settle(theme_settle);
            self.request_frame();
        }
    }

    /// `WindowEvent::Focused(false)`: the window lost focus. ROBUST AUTOSAVE —
    /// flush a pending note write, the document autosave / scratch stash, and
    /// (native only) the session restore state on the same blur trigger. Also
    /// resets the OS pointer to Visible so a focus change never leaves it hidden
    /// behind another app.
    /// `WindowEvent::Focused(true)`: the window regained focus. RESUME the ambient
    /// lava tick (`crate::lava`): mark focused + clear the tick stamp so
    /// `about_to_wait` re-arms it FRESH (avoiding one huge `dt` catch-up bob from
    /// the blurred gap), and request a redraw so the lamp repaints and the tick
    /// re-arms this turn. Inert for a non-lava world (nothing to resume — the tick
    /// gate stays false), so no extra frame is scheduled there.
    pub(super) fn on_focus_gained(&mut self) {
        // LIVE PROBE focus-theft detector: a probe window is launched non-key
        // (Accessory + `activate_ignoring_other_apps(false)` + `with_active(false)`
        // → `orderFront`), so it must NEVER receive `Focused(true)`. If this trace
        // ever fires during a probe run, the window stole the user's keyboard
        // focus — a hard regression the smoke run asserts on (grep the stderr).
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::live_active() {
            crate::probe::trace(format_args!(
                "FOCUS-GAINED (window became key — focus theft!)"
            ));
        }
        // FLIGHT RECORDER (not the probe — its window is never key): a normal focus
        // regain resumes the ambient tick. Logged so the black box can tell a stale
        // frame during a blurred gap apart from a genuine preview-time race.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::flight_active() {
            crate::probe::trace(format_args!("focus gained (ambient tick resumes)"));
        }
        self.frame.set_focused(true);
        self.frame.clear_lava_tick();
        self.settle_external_change_if_document();
        self.request_frame();
    }

    /// `WindowEvent::Occluded`: the window's compositor visibility changed.
    /// When it becomes VISIBLE again (`occluded == false`), request a redraw —
    /// the GPU skip path parked `Occluded → WaitForWake` with no retry timer, so
    /// without this wake an un-occluded window could sit un-repainted until some
    /// unrelated event happened to arrive. Becoming occluded needs no action
    /// (the next acquire returns `Occluded` and re-parks the loop). The decision
    /// is the pure `occluded_change_wants_redraw` so it is unit-testable.
    pub(super) fn on_occluded(&mut self, occluded: bool) {
        // FLIGHT RECORDER / PROBE: occlusion is the direct cause of a skipped
        // present (wgpu returns `Occluded` before `nextDrawable`), so a vanish that
        // coincides with an occlusion transition is the OS hiding the window, not a
        // preview race — the black box must distinguish them.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!("occluded={occluded}"));
        }
        self.frame.set_occluded(occluded);
        if occluded_change_wants_redraw(occluded) {
            self.request_frame();
        }
    }

    pub(super) fn on_focus_lost(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::flight_active() {
            crate::probe::trace(format_args!("focus lost (ambient tick pauses)"));
        }
        self.frame.set_focused(false);
        self.frame.clear_lava_tick();
        self.frame.park_animations();
        // ROBUST AUTOSAVE: the window lost focus (the user switched away);
        // flush a pending note write now so a note is never left unsaved
        // behind another app — and flush the document autosave / scratch
        // stash on the same trigger (locked decision: save on blur).
        self.flush_note();
        self.autosave_flush();
        #[cfg(not(target_arch = "wasm32"))]
        self.session_flush();
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_flush();
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_flush_if_document();
        // HOLD-⌘ SHORTCUT PEEK: the window losing focus breaks the hold — cancel a
        // pending peek / close an open one, so it never lingers behind another app
        // (the macOS focus-loss edge the HUD's own `hud_release_on_mods` covers for
        // the held HUD). Inert unless a peek is pending/open.
        self.feed_peek(crate::peek::PeekStimulus::Interrupt);
        // POINTER AUTO-HIDE: a focus change must never leave the OS
        // pointer hidden behind another app — reset to Visible on blur
        // too, on the same trigger as the autosave flush above.
        if let Some(visible) = self.input.reveal_pointer()
            && let Some(gpu) = self.frame.gpu()
        {
            gpu.window.set_cursor_visible(visible);
        }
    }

    /// `WindowEvent::Resized`: resize the surface, re-sync the view (re-wraps the
    /// column at the new physical size), and redraw.
    ///
    /// **SYNCHRONOUS REDRAW ON RESIZE** (macOS live-resize correctness): a bare
    /// `request_redraw()` only QUEUES a `RedrawRequested` for winit's run loop
    /// to deliver later — normally prompt, but during a live-resize drag it
    /// leaves a real gap in which the surface has been reconfigured to the NEW
    /// size while the LAST rendered frame is still the OLD one. Drawing +
    /// presenting the just-reconfigured surface RIGHT HERE, gated to an actual
    /// size change, closes that gap outright rather than depending on the
    /// queued redraw's timing. `gpu.redraw()` alone never touches the caret-
    /// spring / debug-panel bookkeeping that lives in `on_redraw_requested`
    /// (spring advance, `redraw_count`, key→px stamping) — it is a pure,
    /// idempotent "draw what's already prepared, right now", so calling it
    /// here is safe alongside the unchanged trailing `request_redraw()` below,
    /// which still keeps that bookkeeping on its normal cadence.
    ///
    /// This alone does not fully cure a FAST drag, though: even a synchronous
    /// present here can still lose a race against AppKit's own resize-tracking
    /// Core Animation transaction at high drag speed, which is what shows as
    /// the compositor briefly STRETCHING the last frame instead of showing a
    /// blank/stale one — see `Gpu::set_presents_with_transaction`'s doc for
    /// the companion half of this fix (`arm_live_resize_sync` below).
    pub(super) fn on_resized(
        &mut self,
        event_loop: &ActiveEventLoop,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        let mut changed = false;
        let mut request_redraw = true;
        #[cfg(not(target_arch = "wasm32"))]
        let mut reconfigured = false;
        if let Some(gpu) = self.frame.gpu_mut() {
            changed = gpu.config.width != size.width || gpu.config.height != size.height;
            if changed {
                gpu.pipeline
                    .hold_lava_field_viewport(gpu.config.width, gpu.config.height);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                reconfigured =
                    gpu.resize(size.width, size.height) == gpu::GpuResizeOutcome::Reconfigured;
            }
            #[cfg(target_arch = "wasm32")]
            {
                gpu.resize(size.width, size.height);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if reconfigured && let Some(soak) = self.soak.as_mut() {
            soak.observe_resize();
        }
        self.sync_view(true);
        if changed {
            self.arm_live_resize_sync();
            let outcome = self.frame.gpu_mut().map(Gpu::redraw);
            if let Some(prepared) = outcome {
                request_redraw = self
                    .handle_gpu_frame_outcome(event_loop, prepared.outcome)
                    .is_ok_and(|(_, presented)| presented);
            }
        }
        if request_redraw {
            self.request_frame();
        }
    }

    /// Re-arm the cross-platform live-resize settle debounce and (through the
    /// one owner `sync_present_txn`) turn `presentsWithTransaction` ON if no
    /// live stream had it armed yet. Re-stamp the settle deadline either way —
    /// `about_to_wait`'s `RESIZE_SYNC_SETTLE` debounce flips it back off
    /// `RESIZE_SYNC_SETTLE` after the LAST tick, not the first, so a fast
    /// multi-tick drag keeps sliding the deadline forward exactly like the
    /// theme-font/zoom-persist debounces. See `resize_settle_at`'s own doc for
    /// the full mechanism + the user-reported symptom this closes.
    pub(super) fn arm_live_resize_sync(&mut self) {
        self.frame
            .arm_settle(frame::SettleKind::Resize, self.frame.now());
        self.sync_present_txn();
    }

    /// `WindowEvent::Moved`: the window-server is actively moving the window —
    /// hold the lava lamp (re-stamp the settle debounce, clear the tick arm so
    /// the phase can't advance mid-stream) and, through `sync_present_txn`, arm
    /// `presentsWithTransaction` for the WHOLE stream. The transaction sync is
    /// the structural half of the move-flash fix: pausing the ambient tick
    /// (318e1fe) stopped the ~10 fps mid-move presents, but every OTHER present
    /// around a move — the settle redraw, a sibling debounce (autosave/
    /// toast/zoom-persist) firing mid-stream, a cross-display
    /// `ScaleFactorChanged` redraw — still presented ASYNC and raced the
    /// window-server's move transaction (the diagnosed compositor-flash class;
    /// the resize path already had this cure, the move path never did). Gated
    /// on the lava CAPABILITY: a non-lava world presents nothing around a move,
    /// so it takes this arm as a TOTAL no-op (zero redraws scheduled — the
    /// structural guarantee) and its `Moved` events stay byte-identical to
    /// before the move machinery existed.
    pub(super) fn on_moved(&mut self, _position: winit::dpi::PhysicalPosition<i32>) {
        if crate::theme::active().has_ambient_motion() {
            #[cfg(not(target_arch = "wasm32"))]
            if crate::probe::recording() {
                crate::probe::trace(format_args!("on_moved (ambient world)"));
            }
            self.frame
                .arm_settle(frame::SettleKind::Move, self.frame.now());
            self.frame.clear_lava_tick();
            self.sync_present_txn();
        }
    }

    /// THE ONE APPLIER of the `presentsWithTransaction` composition
    /// (`present_sync_armed`: resize stream OR move stream — see
    /// `App::present_sync_on`'s doc). Idempotent per state: the objc call fires
    /// only on a real transition, so a fast `Moved`/`Resized` burst re-stamping
    /// its debounce costs no per-event layer traffic. The shadow flag is
    /// tracked on every platform; the layer call is macOS-only (the artifact
    /// class is the macOS window-server transaction race).
    pub(super) fn sync_present_txn(&mut self) {
        // The crossing source stays armed while EITHER the preview-settle debounce
        // is pending (`crossing_settle_at`) OR the event-ordered teardown is still
        // waiting for the post-reshape present (`crossing_teardown_pending`) — the
        // latter is what holds the bracket ON across the reshape's present so it
        // can never coalesce into an unbracketed frame.
        let (resize_active, move_active, crossing_active) = self.frame.present_sync_sources();
        let want = present_sync_armed(resize_active, move_active, crossing_active);
        if !self.frame.apply_present_sync(want) {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!(
                "present_txn {} (resize={} move={} crossing={} teardown_pending={})",
                if want { "ON" } else { "OFF" },
                resize_active,
                move_active,
                self.frame.settles().crossing_at.is_some(),
                self.frame.settles().crossing_teardown_pending,
            ));
        }
        #[cfg(target_os = "macos")]
        if let Some(gpu) = self.frame.gpu() {
            gpu.set_presents_with_transaction(want);
        }
    }

    /// The RESIZE stream's settle (the `RESIZE_SYNC_SETTLE` debounce elapsed
    /// with no further `Resized` tick): snap the lava field to the final
    /// viewport, drop this stream's claim on the present-transaction sync (the
    /// one owner keeps it armed while a MOVE stream is still live), and request
    /// the ONE settle redraw. Clearing `resize_settle_at` first is what makes
    /// the settle fire exactly once — the `about_to_wait` arm is gated on the
    /// stamp being present.
    #[cfg(test)]
    pub(super) fn finish_resize_settle(&mut self) {
        self.frame.clear_settle(frame::SettleKind::Resize);
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline
                .settle_lava_field_viewport(gpu.config.width, gpu.config.height);
        }
        self.sync_present_txn();
        self.request_frame();
    }

    /// The MOVE stream's settle (the `MOVE_SETTLE` debounce elapsed with no
    /// further `Moved` tick): clear the hold, clear the tick arm (so the lamp
    /// re-arms FRESH rather than replaying the held gap as a catch-up dt), drop
    /// this stream's claim on the present-transaction sync (armed stays armed
    /// while a RESIZE stream is still live — a corner drag streams both), and
    /// request the ONE settle redraw. The phase and the field were held for the
    /// whole stream (`lava::lava_paused` closed the only door to
    /// `advance_lava`; a pure move never touches `lava_field_viewport`), so
    /// this redraw presents the SAME lava the move started with — no snap, no
    /// flash. Clearing `move_settle_at` first makes it fire exactly once.
    #[cfg(test)]
    pub(super) fn finish_move_settle(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!("finish_move_settle"));
        }
        self.frame.clear_settle(frame::SettleKind::Move);
        self.frame.clear_lava_tick();
        self.sync_present_txn();
        self.request_frame();
    }

    /// The PREVIEW SETTLE (the `CROSSING_SYNC_SETTLE` debounce elapsed with no
    /// further preview step) — PHASE 1 of an EVENT-ORDERED bracket teardown, NOT
    /// an immediate disarm. The visible prefix already presented; the scheduling
    /// interpreter now finishes the final world's off-screen tail before asking
    /// for this settle frame. If we disarmed the bracket here, that redraw could
    /// carry the fully shaped frame to the compositor UNBRACKETED — the exact
    /// vanishing-page race. Instead we clear the debounce but HAND OFF to
    /// `crossing_teardown_pending`, which keeps the bracket armed
    /// (`sync_present_txn`'s OR) until the post-present hook in
    /// `on_redraw_requested` observes that the reshaped frame has presented INSIDE
    /// the bracket, and only THEN disarms. Clearing `crossing_settle_at` first is
    /// what makes the `about_to_wait` arm (gated on the stamp) fire exactly once.
    /// A resize/move stream still live keeps the sync armed regardless (the one
    /// owner composes all three). Live-only: a headless capture never previews.
    #[cfg(test)]
    pub(super) fn finish_crossing_settle(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!(
                "finish_crossing_settle -> teardown armed (bracket HELD through reshape present)"
            ));
        }
        self.frame.begin_crossing_teardown();
        self.sync_present_txn(); // no-op transition: was ON via the debounce, stays ON via the hold.
        self.request_frame();
    }

    /// PHASE 2 of the event-ordered bracket teardown, fired from the post-present
    /// hook in `on_redraw_requested` once a frame has PRESENTED with
    /// `crossing_teardown_pending` set — i.e. the reshaped frame has landed on the
    /// compositor INSIDE the transaction. Only now do we drop the preview's claim
    /// on the present-transaction sync (a live resize/move stream keeps it armed —
    /// the one owner composes all three) and request one final clean async present.
    /// This is the happens-after that replaces the old timer race: teardown
    /// STRICTLY follows the bracketed reshape present, never coalesces with it.
    pub(super) fn finish_crossing_teardown(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!(
                "finish_crossing_teardown (reshape presented in-bracket) -> disarm"
            ));
        }
        self.frame.finish_crossing_teardown();
        self.sync_present_txn();
        self.request_frame();
    }

    pub(super) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        let sf = scale_factor as f32;
        self.frame.set_dpi(sf);
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_dpi(sf);
        }
        self.sync_view(true);
        self.request_frame();
    }

    /// `WindowEvent::RedrawRequested`: sample every activity from the elapsed
    /// time since the prior presented frame, then draw. The post-prepare activity
    /// report is handed to the conditional frame clock; `about_to_wait` asks that
    /// one reducer whether another display-synced frame or sparse deadline is owed.
    /// Also feeds the
    /// DEBUG-panel perf lines (all timing work gated on `debug_on()`) and drives
    /// its settle-stamp.
    pub(super) fn on_redraw_requested(&mut self, event_loop: &ActiveEventLoop) {
        let fault = self
            .frame
            .gpu()
            .and_then(|g| g.take_faults().into_iter().next());
        if let Some(fault) = fault {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(soak) = self.soak.as_mut() {
                soak.observe_frame(
                    crate::soak_gpu::FrameOutcome::Skipped(crate::soak_gpu::SkipKind::Fault),
                    false,
                );
            }
            self.handle_gpu_fault(event_loop, fault);
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }
        if self.frame.take_zoom_reflow() {
            self.sync_view(true);
        }
        let now = self.frame.now();
        let sample = self.frame.frame_sample(now);
        self.frame.begin_redraw();
        let is_stamp = self.feed_debug_panel(now);
        let Some((prepared, presentation_available, travelling_ground)) =
            self.prepare_live_frame(now, sample)
        else {
            return;
        };
        // A theme preview's off-screen shaping tail deliberately remains owed here.
        // The crossing quiet settle pays the latest selection once; another input
        // may supersede it before then.
        let mut activities = prepared.activities;
        if !presentation_available {
            activities = crate::frame_clock::ActivitySet::empty();
        }
        if travelling_ground {
            activities.insert(crate::frame_clock::Activity::TravellingGround);
        }
        let (presented, frame_presented) =
            match self.handle_gpu_frame_outcome(event_loop, prepared.outcome) {
                Ok(result) => result,
                Err(()) => {
                    self.frame.park_animations();
                    event_loop.set_control_flow(ControlFlow::Wait);
                    return;
                }
            };
        if frame_presented {
            self.frame.frame_presented(sample, activities);
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(elapsed) = self.frame.animation_settled(sample.now, activities)
                && crate::probe::recording()
            {
                crate::probe::trace(format_args!(
                    "input-to-animation-settled {:.3}ms",
                    elapsed.as_secs_f64() * 1000.0,
                ));
            }
        } else {
            self.frame.park_animations();
        }
        // DEBUG bookkeeping for the frame that just PRESENTED (`presented`
        // is `Some` only with the panel on — see `Gpu::redraw`): close the
        // key→px span at present-return, and push the measured cost into
        // the ring for the NEXT frame's line 1 — except on the stamp frame,
        // whose cost is measured and DISCARDED (panel bookkeeping, not user
        // workload; displaying it would take yet another frame). An
        // early-return redraw (`None`) keeps the input stamp alive so the
        // latency measures through to the retry frame that really presents.
        if let Some((cost_ms, done)) = presented {
            self.frame.record_present_cost(cost_ms, done, is_stamp);
        }

        // THEME-SWITCH SETTLE readout (DEBUG, live-only): this present carried a
        // timed reshape to the screen, so it is the settled present. Gated on a
        // REAL present (`frame_presented`) so a skipped/occluded frame keeps the
        // switch in flight until one lands.
        if frame_presented && presented.is_some() {
            self.fold_settled_theme_switch(now);
        }

        if frame_presented && self.frame.settles().crossing_teardown_pending {
            self.finish_crossing_teardown();
        }

        // FLIGHT RECORDER / PROBE: name every active reason and the actual
        // interval between presented samples. Theme input/first-pixel timing is
        // unchanged; this is the visible-motion tail.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!(
                "frame presented={frame_presented} interval={:.1}ms activities=[{}]",
                sample.elapsed.as_secs_f32() * 1000.0,
                activities.names(),
            ));
        }
        // requestRedraw, issued by the reducer in `about_to_wait`, is winit's
        // display-cadenced door on native and requestAnimationFrame on web.
        event_loop.set_control_flow(ControlFlow::Wait);
        // DEBUG settle-stamp: the first redraw that ends SETTLED while the
        // panel is on queues exactly ONE more frame — the stamp that draws
        // the `still ·` readout with the final true numbers — and then the
        // machine goes fully quiet (the stamp itself requests nothing).
        // Control flow stays `Wait`; `request_redraw` alone delivers the
        // one frame. New input meanwhile simply wins (see `still_wake`).
        // The stamp frame is "the first redraw that ends SETTLED", so it reads the
        // SAME composed animation state the keep-hot decision does —
        // otherwise the panel would stamp `still ·` on a frame that had just
        // started a band ease and was about to run hot again.
        if crate::debug::debug_on() && self.frame.settle_debug_panel(!activities.is_empty()) {
            self.frame.demand_draw_once();
        }
    }
}
