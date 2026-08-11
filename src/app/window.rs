use super::*;

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
        // EXTERNAL CHANGE ON RETURN — the check that replaces a filesystem
        // watcher, at the one instant it changes anything (`files/external.rs`).
        self.settle_external_change();
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
        self.streaks_flush();
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
            if let Some(outcome) = outcome {
                request_redraw = self
                    .handle_gpu_frame_outcome(event_loop, outcome)
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
    /// an immediate disarm. The font reshape runs SYNCHRONOUSLY inside
    /// `retint_theme_preview` — nothing defers it to a later settle — so by the
    /// time this fires the reshaped view was already applied, well before this
    /// debounce even armed, but its own present may still be in flight behind
    /// this frame's redraw request. If we disarmed the bracket here, that redraw
    /// could carry the reshaped frame to the compositor UNBRACKETED — the exact
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

    /// `WindowEvent::RedrawRequested`: advance the caret spring by the real
    /// elapsed time since the last animated frame, then draw. If still animating,
    /// keep the loop hot (Poll + request another redraw); once settled, go back to
    /// Wait so the app idles at 0% CPU until the next input. Also feeds the
    /// DEBUG-panel perf lines (all timing work gated on `debug_on()`) and drives
    /// its settle-stamp.
    /// Feed the DEBUG panel's per-frame diagnostics and say whether THIS redraw
    /// is the settle STAMP. Lifted out of `on_redraw_requested` whole: it is the
    /// only timing work in a frame and all of it is gated on the panel being on,
    /// so sitting it beside the frame's control flow only made the control flow
    /// harder to read. Returns `is_stamp` so the caller can keep the panel's own
    /// bookkeeping frame out of the measured-cost ring.
    fn feed_debug_panel(&mut self, now: Instant) -> bool {
        // DEBUG panel feed — the ONLY timing work, and all of it gated on
        // the panel being on (the pane never creates the work it measures;
        // the pane-off editor takes zero clock reads). The panel text is
        // shaped inside `pipeline.prepare`, so the values are fed at the
        // TOP of the redraw, BEFORE `gpu.redraw()`: line 1 therefore shows
        // the PREVIOUS completed frame's cost (one-frame lag — this frame's
        // cost isn't knowable until it presents).
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
                // Also surface the live GPU memory (macOS: Metal's
                // currentAllocatedSize; `None` elsewhere → `gpu —`).
                let bytes = gpu.current_gpu_bytes();
                gpu.pipeline.set_debug_gpu_bytes(bytes);
                // AUTOSAVE-ENGINE line: composed EXCLUSIVELY from what
                // `App::autosave_flush`'s one door already tracks — config's
                // `autosave_on()`, the clobber guard's `notice`, and the
                // engine's own last-write clock — so it can never say
                // anything the engine didn't just do. The only clock read
                // here (`now - persistence.engine_last_write_at()`) is gated on
                // `debug_on()` like every other perf read this block makes.
                gpu.pipeline.set_debug_autosave(Some(autosave));
                // Transaction diagnostics age on the App clock, not on frame count.
                // This redraw already happened for real work; checking here never
                // arms a timer or turns the Debug panel into a hot loop.
                gpu.pipeline.set_debug_theme_settle(debug.theme_settle);
            }
        } else if self.clear_debug_session_if_populated() {
            // Clear GPU diagnostics only when a live pipeline exists.
            if let Some(gpu) = self.frame.gpu_mut() {
                gpu.pipeline.set_debug_perf(None, None, None, true, None);
                gpu.pipeline.set_debug_autosave(None);
                gpu.pipeline.set_debug_theme_settle(None);
            }
        }
        is_stamp
    }

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
        let dt = match self.frame.last_frame() {
            Some(prev) => (now - prev).as_secs_f32(),
            None => 1.0 / 60.0,
        };
        self.frame.begin_redraw();
        let is_stamp = self.feed_debug_panel(now);
        // A STATIC open overlay must NOT busy-loop: an idle menu is a frozen
        // frame, so forcing ControlFlow::Poll just because an overlay is open
        // re-ran prepare_overlay/set_rich_text every frame, pegging the CPU.
        // Instead the overlay redraws ON INPUT — every overlay-affecting key
        // (query edit, selection move, filter, open/close) is a KeyboardInput
        // event that routes through `apply` and then calls request_redraw
        // below, and OS key AUTO-REPEAT for a HELD arrow delivers a fresh
        // KeyboardInput per repeat, so a held arrow still repaints promptly.
        // HOT while either the caret spring animates or a TRAVELLING ground runs
        // (`App::advance_travelling_ground`).
        let warp_hot = self.advance_travelling_ground(dt);
        let (stepped, outcome) = if let Some(gpu) = self.frame.gpu_mut() {
            // Drive the virtual-clock seam (caret spring + any future live
            // animator) so the timeline capture and the live loop advance
            // animation through the SAME entry point.
            (gpu.pipeline.advance(dt) || warp_hot, gpu.redraw())
        } else {
            return;
        };
        // THE TAIL, SAME STEP. `gpu.redraw()` above has already handed this frame to
        // the compositor, so any off-screen rows a `ShapeReach::Presentable` reshape
        // stopped short of are shaped RIGHT HERE — inside the event handler winit is
        // still in, so the next input cannot be delivered until the document is whole
        // again, and there is never a half-finished tail for a later step to catch up
        // on. Unconditional on the frame's own outcome: the work is owed to the
        // document, not to the present, so a skipped or occluded acquire pays it too.
        self.finish_shape_tail();
        // The SECOND animation term, and it can only be read HERE.
        // `gpu.redraw()` above ran `prepare`, and `prepare` is the one place the
        // selection band is retargeted, so an ease that started this frame is
        // strictly after the `advance` that produced `stepped`. See
        // `keep_gpu_loop_hot`'s doc and `TextPipeline::take_band_ease_started`.
        let band_ease_started = self
            .frame
            .gpu_mut()
            .is_some_and(|gpu| gpu.pipeline.take_band_ease_started());
        let (presented, frame_presented) = match self.handle_gpu_frame_outcome(event_loop, outcome)
        {
            Ok(result) => result,
            Err(()) => {
                event_loop.set_control_flow(ControlFlow::Wait);
                return;
            }
        };
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

        // Keep the loop hot ONLY while the spring animates — the debug panel
        // schedules ZERO frames of its own (every metric it shows is
        // meaningful for a single sparse frame). The held stats HUD does NOT
        // force frames either: its figures are pure functions of the doc
        // (no session clock), so a held HUD is a single settled frame over
        // the cached frosted backdrop. `last_frame` still tracks ONLY the
        // spring, so the dt fed to `advance` stays correct.
        // A failed acquire never drives the animation Poll loop. The spring
        // simply resumes from the next OS/input/timed wake; otherwise an
        // occluded window can allocate and prepare thousands of unseen frames.
        let keep_hot = keep_gpu_loop_hot(stepped, band_ease_started, frame_presented);
        // FLIGHT RECORDER / PROBE: the ANIMATION-SCHEDULING link. Both
        // animation terms are logged separately, because which of the two is true
        // is exactly what tells a redraw/present gap apart from a settled frame:
        // `stepped` is the PRE-prepare answer, `band_started` the POST-prepare one.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!(
                "redraw dt={:.1}ms stepped={stepped} band_started={band_ease_started} \
                 presented={frame_presented} keep_hot={keep_hot}",
                dt * 1000.0
            ));
        }
        self.frame
            .set_last_frame(if keep_hot { Some(now) } else { None });
        if keep_hot {
            event_loop.set_control_flow(ControlFlow::Poll);
            self.request_frame();
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
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
        if crate::debug::debug_on() && self.frame.settle_debug_panel(stepped || band_ease_started) {
            self.request_frame();
        }
    }
}
