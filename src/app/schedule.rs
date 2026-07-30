//! The `about_to_wait` SCHEDULING body, lifted verbatim out of
//! `App::about_to_wait` (the decomposition round — zero behaviour change).
//! This is the winit idle pass that owns every debounce / settle deadline:
//! which-key + hold-peek pauses, the note / document autosave idle timers,
//! the theme-font / sticky-zoom / resize / move / crossing settles, the
//! ambient (lava/stars) tick, event-toast expiry, GPU acquire retries, and
//! the GPU soak drive — each a single `WaitUntil` (never a hot per-frame
//! loop), guarded on `last_frame` so the caret spring's `Poll` always wins.
//! (Spell-check is NOT debounced here — see `App::recompute_spell_cache`'s
//! doc for why it's eager instead.)
//!
//! A trait impl can't span files, so the body moves to an inherent `App`
//! method here and the `ApplicationHandler::about_to_wait` in `app.rs` stays
//! a thin delegate. `use super::*` reaches every free helper it calls
//! (`debounce_due`, `control_flow_with_deadline`, `notice_expired`, the
//! debounce-window consts) — those stay in `app.rs`, shared with other sites.

use super::*;

/// The winit control-flow SINK the scheduling body writes its debounce / settle
/// deadlines into. `about_to_wait_impl`'s ONLY dependency on the event loop is
/// `set_control_flow` / `control_flow`, so abstracting exactly those two behind a
/// trait lets the SAME scheduling body a live winit idle runs be STEPPED headlessly
/// under a [`crate::clock::VirtualClock`] (the frame-loop capture + the multi-frame
/// scheduling law), with the winit [`ActiveEventLoop`] and a headless
/// [`RecordingScheduler`] as the two sinks. One body, two callers — the harness can
/// never drift from the live scheduling path.
pub(crate) trait Scheduler {
    fn set_control_flow(&self, control_flow: ControlFlow);
    fn control_flow(&self) -> ControlFlow;
}

impl Scheduler for ActiveEventLoop {
    #[inline]
    fn set_control_flow(&self, control_flow: ControlFlow) {
        ActiveEventLoop::set_control_flow(self, control_flow)
    }
    #[inline]
    fn control_flow(&self) -> ControlFlow {
        ActiveEventLoop::control_flow(self)
    }
}

/// The winit SHUTDOWN SINK the input-dispatch chain writes its one quit request
/// into. `App::apply` — and every door that reaches it (keyboard, menu, palette
/// re-dispatch, pointer, drag, the `--live-script` probe) — took a full
/// `&ActiveEventLoop` for exactly ONE call, `exit()`, which made the whole live
/// effect-interpretation surface unreachable from any headless caller: an
/// `ActiveEventLoop` can only be borrowed from inside a running winit loop, and
/// there is no way to construct one. Abstracting exactly that one capability is
/// the same move [`Scheduler`] made for `about_to_wait_impl` — one body, two
/// sinks, so the harness can never drift from the live path. See
/// `docs/harness-reach.md` for what it opened up.
pub(crate) trait Exit {
    fn exit(&self);
}

impl Exit for ActiveEventLoop {
    #[inline]
    fn exit(&self) {
        ActiveEventLoop::exit(self)
    }
}

/// A headless [`Exit`] that RECORDS the quit request instead of stopping a loop
/// there is none of — the sink a test or capture hands to
/// [`App::dispatch_pressed_key`](crate::app::App::dispatch_pressed_key) /
/// `App::apply`. `App::apply` already returns the quit bool, so this exists for
/// the doors that swallow it (menu, pointer, probe) and to make "did this key
/// ask the process to end?" observable off-window.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct RecordingExit {
    requested: std::cell::Cell<bool>,
}

#[cfg(test)]
impl RecordingExit {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    /// Whether any door driven through this sink asked the loop to exit.
    pub(crate) fn exit_requested(&self) -> bool {
        self.requested.get()
    }
}

#[cfg(test)]
impl Exit for RecordingExit {
    fn exit(&self) {
        self.requested.set(true);
    }
}

/// A headless [`Scheduler`] that RECORDS the control flow the scheduling body set,
/// so the frame-loop capture + the scheduling law can assert what a live winit idle
/// WOULD have been told: which `WaitUntil` deadline was armed this step, or that
/// nothing was scheduled (the debounce fired / the loop went quiet). Pure `Cell`
/// state — not a winit type — so it drives the same body off-window. `current`
/// mirrors winit's own control-flow register (default `Wait`); `set_this_step` is
/// the ONE thing set THIS pass, cleared by [`begin_step`](Self::begin_step) before
/// each scheduling call.
#[cfg(any(test, not(target_arch = "wasm32")))]
pub(crate) struct RecordingScheduler {
    current: std::cell::Cell<ControlFlow>,
    set_this_step: std::cell::Cell<Option<ControlFlow>>,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl RecordingScheduler {
    pub(crate) fn new() -> Self {
        Self {
            current: std::cell::Cell::new(ControlFlow::Wait),
            set_this_step: std::cell::Cell::new(None),
        }
    }
    /// Clear the per-step record; call once before each `step_scheduling` so
    /// [`scheduled_this_step`](Self::scheduled_this_step) reflects ONLY this pass.
    pub(crate) fn begin_step(&self) {
        self.set_this_step.set(None);
    }
    /// The control flow the scheduling body set THIS step, or `None` if it set
    /// nothing (e.g. the debounce fired, or the loop is idle with no armed timer).
    pub(crate) fn scheduled_this_step(&self) -> Option<ControlFlow> {
        self.set_this_step.get()
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl Scheduler for RecordingScheduler {
    fn set_control_flow(&self, control_flow: ControlFlow) {
        self.current.set(control_flow);
        self.set_this_step.set(Some(control_flow));
    }
    fn control_flow(&self) -> ControlFlow {
        self.current.get()
    }
}

impl App {
    fn schedule_prefix_surfaces(&mut self, event_loop: &impl Scheduler) {
        if let Some(pending) = self.prefix_pending_at {
            let deadline = pending + crate::whichkey::PAUSE;
            let elapsed = self.clock.now() >= deadline;
            if crate::whichkey::should_summon(true, self.whichkey_shown, elapsed) {
                self.summon_whichkey();
            } else if !self.whichkey_shown && !elapsed && self.last_frame.is_none() {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
        }
        if let Some(armed) = self.peek_armed_at {
            let deadline = armed + Duration::from_millis(crate::peek::HOLD_PEEK_MS);
            if self.clock.now() >= deadline {
                let stimulus = if crate::peek::peek_allowed(self.zoom_in_flight()) {
                    crate::peek::PeekStimulus::Elapsed
                } else {
                    crate::peek::PeekStimulus::ArmBroken
                };
                self.feed_peek(stimulus);
            } else if self.last_frame.is_none() {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
        }
    }

    fn schedule_autosaves(&mut self, event_loop: &impl Scheduler) {
        if let Some(deadline) = self.persistence.note_debounce_deadline(AUTOSAVE_DEBOUNCE) {
            if self.clock.now() >= deadline {
                self.persistence.disarm_note_debounce();
                self.autosave_note();
                if let Some(gpu) = self.gpu.as_ref() {
                    gpu.window.request_redraw();
                }
            } else if self.last_frame.is_none() {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
        }
        if let Some(dirty) = self.active.extra.doc_autosave_at {
            match debounce_due(dirty, AUTOSAVE_IDLE, self.clock.now()) {
                true => {
                    self.active.extra.doc_autosave_at = None;
                    self.autosave_flush();
                    #[cfg(not(target_arch = "wasm32"))]
                    self.stats_flush();
                    #[cfg(not(target_arch = "wasm32"))]
                    self.streaks_flush();
                    if let Some(gpu) = self.gpu.as_ref() {
                        gpu.window.request_redraw();
                    }
                }
                false if self.last_frame.is_none() => {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(dirty + AUTOSAVE_IDLE));
                }
                false => {}
            }
        }
    }

    fn schedule_render_settles(&mut self, event_loop: &impl Scheduler) {
        if let Some(dirty) = self.theme_font_at {
            let debounce = theme_font_debounce();
            match debounce_due(dirty, debounce, self.clock.now()) {
                true => self.apply_deferred_theme_font(),
                false if self.last_frame.is_none() => {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(dirty + debounce));
                }
                false => {}
            }
        }
        if let Some(dirty) = self.zoom_persist_at.filter(|_| !self.zoom_persist_held()) {
            match debounce_due(dirty, ZOOM_PERSIST_DEBOUNCE, self.clock.now()) {
                true => self.settle_zoom_persist(),
                false if self.last_frame.is_none() => {
                    event_loop
                        .set_control_flow(ControlFlow::WaitUntil(dirty + ZOOM_PERSIST_DEBOUNCE));
                }
                false => {}
            }
        }
        if let Some(dirty) = self.resize_settle_at {
            match debounce_due(dirty, RESIZE_SYNC_SETTLE, self.clock.now()) {
                true => self.finish_resize_settle(),
                false if self.last_frame.is_none() => {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(dirty + RESIZE_SYNC_SETTLE));
                }
                false => {}
            }
        }
        if let Some(dirty) = self.move_settle_at {
            match debounce_due(dirty, MOVE_SETTLE, self.clock.now()) {
                true => self.finish_move_settle(),
                false if self.last_frame.is_none() => {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(dirty + MOVE_SETTLE));
                }
                false => {}
            }
        }
        if let Some(dirty) = self.crossing_settle_at {
            match debounce_due(dirty, CROSSING_SYNC_SETTLE, self.clock.now()) {
                true => self.finish_crossing_settle(),
                false if self.last_frame.is_none() => {
                    event_loop
                        .set_control_flow(ControlFlow::WaitUntil(dirty + CROSSING_SYNC_SETTLE));
                }
                false => {}
            }
        }
    }

    pub(super) fn about_to_wait_impl(&mut self, event_loop: &impl Scheduler) {
        // WHICH-KEY pause: while a PREFIX (`C-x`) is pending its second key, summon the
        // continuation panel once ~500ms elapses without a follow-up. The timer is
        // ARMED ONLY here, while `prefix_pending_at` is `Some` AND the panel isn't yet
        // shown — a single `WaitUntil` deadline, no perpetual per-frame tick; once it
        // fires (or the prefix resolves, clearing `prefix_pending_at`) nothing re-arms,
        // so the app idles at 0% CPU (DESIGN §6).
        self.schedule_prefix_surfaces(event_loop);
        // HOLD-⌘ SHORTCUT PEEK: while a bare-arming-modifier hold is PENDING, summon the
        // card once ~600ms elapses with the hold unbroken. The timer is ARMED ONLY while
        // `peek_armed_at` is `Some` (the `PeekArm::Pending` state) — a single `WaitUntil`
        // deadline, no perpetual tick; feeding `Elapsed` opens the card and clears the
        // stamp, so nothing re-arms and the app idles at 0% CPU (the which-key pattern).
        // Spell check is no longer debounced here (the completed-word-lag fix,
        // 2026-07): `App::sync_view` recomputes the KEYED verdict cache EAGERLY,
        // synchronously, the instant the buffer version changes — see
        // `App::recompute_spell_cache`'s doc. Nothing left to schedule.
        // Debounced quick-note AUTO-SAVE: write the note after ~400ms of quiet, so
        // it persists calmly as you pause. An empty note writes nothing.
        self.schedule_autosaves(event_loop);
        // Debounced DOCUMENT AUTOSAVE (the config-gated engine, default ON): the
        // open file is written atomically — or the no-path scratch stashed — after
        // ~1s of idle. Armed ONLY by the live `sync_view` (behind its gpu-present
        // gate), consumed here via the same single-`WaitUntil` pattern as the note
        // autosave above — no hot loop, and structurally unreachable headlessly.
        // Debounced theme-preview FONT reshape: while the theme picker's live preview
        // arrows across worlds, only the COLORS applied per step; once the selection
        // rests `theme_font_debounce()` the one deferred reshape lands here (the paused
        // hover then shows the true face). Each further preview step RE-STAMPS
        // `theme_font_at` (`retint_theme_preview`), sliding the deadline — the same
        // single-`WaitUntil`, idle-safe pattern as the zoom persist below (no hot
        // loop; commit/revert clear the stamp synchronously via `retint_theme_now`).
        self.schedule_render_settles(event_loop);
        // Debounced STICKY-ZOOM write: persist the SETTLED zoom after ~500ms of quiet,
        // so a rapid Cmd-=/Cmd-- run writes the final value once (not one-per-step).
        // Each new zoom step RE-STAMPS `zoom_persist_at` (via `mark_zoom_dirty`), so the
        // deadline keeps sliding forward until the user pauses — the debounce contract.
        //
        // THE QUIET WINDOW IS AN INFERENCE, and `zoom_persist_held` is the ONE gate that
        // says when it may run (see its doc): a gesture that owns an explicit END — the
        // Settings rail's button release — pays its own single write there, and must not
        // have a mid-gesture value written out from under it just because the user paused
        // to read the number. While held this branch is entirely inert: no write, and no
        // `WaitUntil` either (nothing is waiting to fire), so the loop still falls quiet.
        // LIVE-RESIZE CONTENT-STRETCH FIX settle (macOS only — see
        // `resize_settle_at`'s doc): once `RESIZE_SYNC_SETTLE` passes with no
        // further `Resized` ticks, flip the CAMetalLayer's `presentsWithTransaction`
        // back OFF (paying its throughput cost only while a drag is actually live).
        // Each new tick RE-STAMPS `resize_settle_at` (`App::arm_live_resize_sync`),
        // sliding the deadline exactly like the theme-font/zoom-persist debounces
        // above — the same single-`WaitUntil` shape, so a still window costs nothing.
        // MOVE-stream settle (mirrors the resize debounce above; see
        // `MOVE_SETTLE`'s doc for why its window is deliberately longer).
        // THEME-PREVIEW CROSSING settle (mirrors the resize/move debounces above;
        // see `CROSSING_SYNC_SETTLE`'s doc). Disarms the present-transaction sync
        // and fires the ONE follow-up present once a boundary crossing has rested.
        // AMBIENT TICK — the slow ~10 fps drift clock behind awl's time-varying
        // grounds: the lava lamp (Firetail/Mangrove), the twinkling stars
        // (Currawong), AND (item 87) Bombora's wave-tier phase drift — ONE
        // clock, three consumers (`TextPipeline::lava_phase`). A single
        // `WaitUntil` cadence (NEVER the caret spring's hot per-frame `Poll`
        // loop): when it elapses, advance the phase, request ONE redraw, and
        // re-arm. Armed ONLY while `lava::lava_should_tick` holds — an
        // ambient-tick world is active (`Theme::has_ambient_tick`, the ONE
        // scheduling gate — a strict superset of `has_ambient_motion`, see its
        // doc) AND `ambient_motion` is on AND motion is not reduced AND the
        // window is focused (pause on blur). Every static world (and, among
        // ambient worlds, every frozen/paused/reduced-motion moment) schedules
        // ZERO ambient frames — preserving 0% idle CPU there.
        let lava_active = crate::theme::active().has_ambient_tick();
        let lava_paused = crate::lava::lava_paused(
            self.resize_settle_at.is_some(),
            self.move_settle_at.is_some(),
            self.gpu
                .as_ref()
                .is_some_and(|gpu| gpu.pipeline.lava_blur_active()),
        );
        if crate::lava::lava_should_tick(
            lava_active,
            self.config.ambient_motion_on(),
            crate::motion::reduced(),
            self.focused,
            lava_paused,
        ) {
            let now = self.clock.now();
            match self.lava_tick_at {
                Some(last) if now.saturating_duration_since(last) >= LAVA_TICK => {
                    // Due: hand the elapsed time to the bounded ambient advance
                    // (`lava::ambient_tick_dt` clamps it to one fixed tick), so a
                    // delayed macOS window-drag wake cannot catch up in one jump.
                    // The follow-up `about_to_wait` pass (after the redraw) re-arms
                    // the single `WaitUntil` via the `_` arm below.
                    let dt = (now - last).as_secs_f32();
                    self.lava_tick_at = Some(now);
                    if let Some(gpu) = self.gpu.as_mut() {
                        gpu.pipeline.advance_lava(dt);
                        gpu.window.request_redraw();
                    }
                }
                _ => {
                    // Not due yet (or the first arm): keep/arm the single
                    // `WaitUntil`, but NEVER override the caret spring's hot `Poll`
                    // loop (guard on `last_frame`, like every sibling debounce) —
                    // during a caret glide the lava still advances above at ~10 fps
                    // while the frame itself redraws at full refresh.
                    let last = *self.lava_tick_at.get_or_insert(now);
                    if self.last_frame.is_none() {
                        event_loop.set_control_flow(control_flow_with_deadline(
                            event_loop.control_flow(),
                            last + LAVA_TICK,
                        ));
                    }
                }
            }
        } else if lava_active {
            // An ambient-motion world (lava or stars), but the ground must be
            // STATIC: reduce motion OR ambient motion off (blur is handled at
            // the focus edge, which merely HOLDS the phase). Stop ticking;
            // hard-freeze the shared phase to the settled frame so a later
            // resume restarts cleanly rather than from a stale mid-breath.
            self.lava_tick_at = None;
            if (crate::motion::reduced() || !self.config.ambient_motion_on())
                && let Some(gpu) = self.gpu.as_mut()
            {
                gpu.pipeline.freeze_lava();
            }
        }
        // EVENT TOAST expiry: one live-only deadline, consumed once. This runs
        // after sibling timers so it can choose the EARLIER deadline instead of
        // delaying a lava tick (or being delayed by one). Poll always wins.
        if let Some(deadline) = self.notice_expires_at {
            if notice_expired(self.notice_kind, Some(deadline), self.clock.now()) {
                self.clear_notice();
                if let Some(gpu) = self.gpu.as_ref() {
                    gpu.window.request_redraw();
                }
            } else if self.last_frame.is_none() {
                event_loop.set_control_flow(control_flow_with_deadline(
                    event_loop.control_flow(),
                    deadline,
                ));
            }
        }
        // GPU acquire retries are App-owned timers, never immediate redraw
        // recursion. Occlusion deliberately arms no timer: an OS visibility,
        // input, lava, or probe wake is the next opportunity. Timeout and
        // surface reconfiguration arrive here after their bounded delay.
        if let Some(deadline) = self.gpu_retry_at {
            if self.clock.now() >= deadline {
                self.gpu_retry_at = None;
                if let Some(gpu) = self.gpu.as_ref() {
                    gpu.window.request_redraw();
                }
            } else if self.last_frame.is_none() {
                event_loop.set_control_flow(control_flow_with_deadline(
                    event_loop.control_flow(),
                    deadline,
                ));
            }
        }
        // The `--soak-gpu` drive used to trail here; it needs the REAL
        // `&ActiveEventLoop` (it resizes the recovery window + sets its own control
        // flow) and always runs on real time, so it moved UP into the trait
        // `about_to_wait` wrapper (`app.rs`) — OUTSIDE this clock-steppable body, so
        // a headless `RecordingScheduler` never has to satisfy it and the soak
        // harness keeps its real event loop. Its ordering (last, after every timer)
        // is preserved by the wrapper calling it right after this returns.
    }
}

impl App {
    /// Drive ONE headless scheduling pass at the injected clock's CURRENT virtual
    /// time — the SAME `about_to_wait_impl` body a live winit idle runs, but writing
    /// its deadlines into a [`RecordingScheduler`] instead of `&ActiveEventLoop`. The
    /// frame-loop capture and the multi-frame scheduling law advance the
    /// [`crate::clock::VirtualClock`], then call this, then read the resulting App
    /// state + the recorded control flow. The `--soak-gpu` drive is deliberately NOT
    /// part of this body (see the note where it used to sit) — a headless step never
    /// touches the GPU soak.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(crate) fn step_scheduling(&mut self, sched: &RecordingScheduler) {
        self.about_to_wait_impl(sched);
    }

    /// Arm the WHICH-KEY prefix pause as of the clock's CURRENT instant — the exact
    /// edge the real input path takes on a `C-x` prefix
    /// (`crate::whichkey::PrefixTransition::Arm`, which runs this identical line). The
    /// frame-loop harness / scheduling law arms this, then steps the clock past
    /// `whichkey::PAUSE` to witness the summon fire EXACTLY at its deadline step.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(crate) fn arm_whichkey_prefix(&mut self) {
        self.prefix_pending_at = Some(self.clock.now());
    }

    /// Whether the which-key continuation panel is currently summoned — the pure App
    /// state the multi-frame law asserts across steps (a single settled frame cannot
    /// show the false→true flip at the pause deadline).
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(crate) fn whichkey_is_shown(&self) -> bool {
        self.whichkey_shown
    }

    /// The pending prefix's continuation rows — the SAME `continuations_cx` the live
    /// summon pushes into the pipeline — so the frame-loop render can draw the panel
    /// the App's scheduling state says is up. (Capture-render only; the law asserts
    /// over [`whichkey_is_shown`](Self::whichkey_is_shown).)
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn whichkey_continuation_rows(&self) -> Vec<(String, String)> {
        crate::whichkey::continuations_cx(&self.config.keys)
            .into_iter()
            .map(|c| (c.key, c.name))
            .collect()
    }

    /// Inject a clock behind the `Box<dyn Clock>` seam (frame-loop harness + the
    /// scheduling law only; the shipped app always keeps `RealClock`, so live timing
    /// is unchanged). The whole scheduling / animation path reads `self.clock`, so
    /// one swap re-times all of it deterministically.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(crate) fn set_clock(&mut self, clock: Box<dyn crate::clock::Clock>) {
        self.clock = clock;
    }
}
