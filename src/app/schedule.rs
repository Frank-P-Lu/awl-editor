//! The `about_to_wait` SCHEDULING body, lifted verbatim out of
//! `App::about_to_wait` (the decomposition round — zero behaviour change).
//! This is the winit idle pass that owns every debounce / settle deadline:
//! which-key + hold-peek pauses, the note / document autosave idle timers,
//! the theme-font / sticky-zoom / resize / move / crossing settles, the
//! ambient (lava/stars) tick, event-toast expiry, GPU acquire retries, and
//! the GPU soak drive. Each proposes one deadline to the frame reducer; an
//! active bounded animation wins, otherwise the earliest proposal becomes one
//! `WaitUntil` (never a hot per-frame loop).
//! (Spell-check is NOT debounced here — see `App::recompute_spell_cache`'s
//! doc for why it's eager instead.)
//!
//! A trait impl can't span files, so the body moves to an inherent `App`
//! method here and the `ApplicationHandler::about_to_wait` in `app.rs` stays
//! a thin delegate. `use super::*` reaches every free helper it calls
//! (`debounce_due`, `notice_expired`, the
//! debounce-window consts) — those stay in `app.rs`, shared with other sites.

use super::*;

/// The winit control-flow SINK the frame reducer writes its final instruction
/// into. `about_to_wait_impl`'s ONLY dependency on the event loop is
/// `set_control_flow`, so abstracting exactly that behind a
/// trait lets the SAME scheduling body a live winit idle runs be STEPPED headlessly
/// under a [`crate::clock::VirtualClock`] (the frame-loop capture + the multi-frame
/// scheduling law), with the winit [`ActiveEventLoop`] and a headless
/// [`RecordingScheduler`] as the two sinks. One body, two callers — the harness can
/// never drift from the live scheduling path.
pub(crate) trait Scheduler {
    fn set_control_flow(&self, control_flow: ControlFlow);
}

impl Scheduler for ActiveEventLoop {
    #[inline]
    fn set_control_flow(&self, control_flow: ControlFlow) {
        ActiveEventLoop::set_control_flow(self, control_flow)
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
/// ask the process to end?" observable off-window. Gated exactly like
/// [`RecordingScheduler`] below — test builds plus every NATIVE build, because
/// the `--screenshot-app` capture mode is a production caller.
#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Default)]
pub(crate) struct RecordingExit {
    requested: std::cell::Cell<bool>,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl RecordingExit {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    /// Whether any door driven through this sink asked the loop to exit.
    pub(crate) fn exit_requested(&self) -> bool {
        self.requested.get()
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
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
    set_this_step: std::cell::Cell<Option<ControlFlow>>,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl RecordingScheduler {
    pub(crate) fn new() -> Self {
        Self {
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
        self.set_this_step.set(Some(control_flow));
    }
}

impl App {
    fn schedule_prefix_surfaces(
        &mut self,
        input: input::SchedulingSnapshot,
        deadlines: &mut crate::frame_clock::Deadlines,
    ) {
        let now = self.frame.now();
        if let Some(pending) = input.prefix_pending_at {
            let deadline = pending + crate::whichkey::PAUSE;
            let elapsed = now >= deadline;
            if crate::whichkey::should_summon(true, input.whichkey_shown, elapsed) {
                self.summon_whichkey();
            } else if !input.whichkey_shown && !elapsed {
                deadlines.propose(Some(deadline));
            }
        }
        if let Some(armed) = input.peek_armed_at {
            let deadline = armed + Duration::from_millis(crate::peek::HOLD_PEEK_MS);
            if now >= deadline {
                let stimulus = if crate::peek::peek_allowed(self.zoom_in_flight()) {
                    crate::peek::PeekStimulus::Elapsed
                } else {
                    crate::peek::PeekStimulus::ArmBroken
                };
                self.feed_peek(stimulus);
            } else {
                deadlines.propose(Some(deadline));
            }
        }
    }

    fn schedule_autosaves(&mut self, deadlines: &mut crate::frame_clock::Deadlines) {
        let now = self.frame.now();
        if let Some(deadline) = self.persistence.note_debounce_deadline(AUTOSAVE_DEBOUNCE) {
            if now >= deadline {
                self.persistence.disarm_note_debounce();
                self.autosave_note();
                self.request_frame();
            } else {
                deadlines.propose(Some(deadline));
            }
        }
        match self
            .document
            .poll_autosave(self.frame.now(), AUTOSAVE_IDLE)
        {
            document::AutosavePoll::Due => {
                self.autosave_flush();
                #[cfg(not(target_arch = "wasm32"))]
                self.stats_flush();
                #[cfg(not(target_arch = "wasm32"))]
                self.streaks_flush();
                self.request_frame();
            }
            document::AutosavePoll::WaitingUntil(deadline) => {
                deadlines.propose(Some(deadline));
            }
            document::AutosavePoll::Idle => {}
        }
    }

    pub(super) fn about_to_wait_impl(
        &mut self,
        event_loop: &impl Scheduler,
        host_deadline: Option<Instant>,
    ) {
        let input_schedule = self.input.scheduling_snapshot();
        let mut deadlines = crate::frame_clock::Deadlines::default();
        deadlines.propose(host_deadline);
        // WHICH-KEY pause: while a PREFIX (`C-x`) is pending its second key, summon the
        // continuation panel once ~500ms elapses without a follow-up. The timer is
        // ARMED ONLY here, while `prefix_pending_at` is `Some` AND the panel isn't yet
        // shown — a single `WaitUntil` deadline, no perpetual per-frame tick; once it
        // fires (or the prefix resolves, clearing `prefix_pending_at`) nothing re-arms,
        // so the app idles at 0% CPU (DESIGN §6).
        self.schedule_prefix_surfaces(input_schedule, &mut deadlines);
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
        self.schedule_autosaves(&mut deadlines);
        // Debounced DOCUMENT AUTOSAVE (the config-gated engine, default ON): the
        // open file is written atomically — or the no-path scratch stashed — after
        // ~1s of idle. Armed ONLY by the live `sync_view` (behind its gpu-present
        // gate), consumed here via the same single-`WaitUntil` pattern as the note
        // autosave above — no hot loop, and structurally unreachable headlessly.
        // Theme preview shapes the paintable prefix synchronously. Its off-screen
        // tail shares the crossing quiet window so superseded worlds never finish
        // work the user has already moved past.
        let now = self.frame.now();
        let config_schedule = self.config.scheduling_snapshot();
        let input_schedule = self.input.scheduling_snapshot();
        let outcome = self.frame.poll(now, input_schedule, config_schedule);
        if outcome.persist_zoom {
            self.settle_zoom_persist();
        }
        if outcome.settle_theme_tail {
            self.finish_shape_tail();
        }
        if outcome.redraw {
            // Present-sync sources are settled inside the frame owner. The
            // host applies their composed value to the platform layer once.
            self.sync_present_txn();
            self.request_frame();
        }
        deadlines.propose(outcome.next_deadline);
        let draw_once = self.frame.take_draw_once();
        match self.frame.directive(deadlines) {
            crate::frame_clock::Directive::Idle => {}
            crate::frame_clock::Directive::Deadline(deadline) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
            crate::frame_clock::Directive::Animating(_activities) => {
                event_loop.set_control_flow(ControlFlow::Wait);
                self.request_frame();
            }
        }
        if draw_once {
            self.request_frame();
        }
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
        // (Currawong), AND Bombora's wave-tier phase drift — ONE
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
        self.about_to_wait_impl(sched, None);
    }

    /// Arm the WHICH-KEY prefix pause as of the clock's CURRENT instant — the exact
    /// edge the real input path takes on a `C-x` prefix
    /// (`crate::whichkey::PrefixTransition::Arm`, which runs this identical line). The
    /// frame-loop harness / scheduling law arms this, then steps the clock past
    /// `whichkey::PAUSE` to witness the summon fire EXACTLY at its deadline step.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(crate) fn arm_whichkey_prefix(&mut self) {
        self.input.arm_prefix(self.frame.now());
    }

    /// Whether the which-key continuation panel is currently summoned — the pure App
    /// state the multi-frame law asserts across steps (a single settled frame cannot
    /// show the false→true flip at the pause deadline).
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(crate) fn whichkey_is_shown(&self) -> bool {
        self.input.whichkey_shown()
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
        self.frame.set_clock(clock);
    }
}
