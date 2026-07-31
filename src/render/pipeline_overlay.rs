use super::*;

impl TextPipeline {
    pub fn advance(&mut self, dt: f32) -> bool {
        self.step_caret(dt)
            | self.step_caret_preview(dt)
            | self.step_copy_pulse(dt)
            | self.step_overlay_juice(dt)
    }

    /// LIVE-APP-ONLY: arm the motion-juice animators (overlay entrance spring
    /// + selection-band slide — the FIRETAIL-MAXIMALIST-SHOWCASE round's
    ///   [`theme::MotionJuice`] capability). Called exactly once, from the live
    ///   App's GPU init (`app/gpu.rs`); every headless capture / bench / test
    ///   pipeline never calls it, so those paths render the settled state
    ///   STRUCTURALLY (the determinism law's "live-only animation renders its
    ///   settled state in capture", enforced by construction rather than by a
    ///   per-frame check). Arming alone changes nothing: the animators also
    ///   require a non-CALM effective [`theme::MotionJuice`] (no world ships
    ///   one — the `AWL_MOTION_FORCE` probe is the only current door) and fold
    ///   to nothing under Reduce Motion.
    pub fn arm_live_juice(&mut self) {
        self.juice_live = true;
    }

    /// Tick the overlay ENTRANCE spring + selection-band SLIDE by `dt`
    /// seconds. Returns true while either is still easing (keeps the live
    /// redraw loop hot exactly as long as the juice plays — then idle).
    ///
    /// ACCESSIBILITY TIER 1 — REDUCE MOTION: both animators settle INSTANTLY
    /// (same final position, zero frames of ease — `motion.rs`'s pure
    /// time-compression contract), mirroring `step_copy_pulse`'s gate
    /// exactly. Law-tested by `overlay_juice_folds_to_nothing_under_reduce_
    /// motion` (render/tests/motion_juice.rs).
    fn step_overlay_juice(&mut self, dt: f32) -> bool {
        if crate::motion::reduced() {
            self.overlay_enter_t = 1.0;
            self.overlay_band_t = 1.0;
            return false;
        }
        let mut hot = false;
        if self.overlay_enter_t < 1.0 {
            self.overlay_enter_t =
                (self.overlay_enter_t + dt * 1000.0 / OVERLAY_ENTRANCE_MS).min(1.0);
            hot |= self.overlay_enter_t < 1.0;
        }
        if self.overlay_band_t < 1.0 {
            self.overlay_band_t =
                (self.overlay_band_t + dt * 1000.0 / OVERLAY_BAND_SLIDE_MS).min(1.0);
            hot |= self.overlay_band_t < 1.0;
        }
        hot
    }

    pub(in crate::render) fn overlay_entrance_offset(&self) -> f32 {
        if self.overlay_enter_t >= 1.0 {
            return 0.0;
        }
        -(1.0 - crate::ease::out_back(self.overlay_enter_t)) * OVERLAY_ENTRANCE_DROP_PX
    }

    /// THE ONE band-RETARGET owner: point the shared chase state
    /// (`overlay_band_from`/`overlay_band_last`/`overlay_band_t`) at a NEW
    /// `target`, continuing smoothly from wherever the band is actually drawn
    /// RIGHT NOW if a transition is still in flight. Shared by both animators
    /// that chase the selected row — the ordinary [`Self::overlay_band_drawn`]
    /// (`BandResponse::Slide`) and the living-band choreography
    /// ([`Self::living_band_phase`]) — so a rapid re-target (arrow-key repeat
    /// outrunning the ~110ms slide) can never teleport the visual anchor to
    /// the STALE previous target instead of where the band visually sits.
    ///
    /// THE BUG THIS CLOSED: `living_band_phase` used to set
    /// `overlay_band_from = last` (the previous call's TARGET) on every
    /// re-target, discarding the in-flight interpolation. Held-down/fast
    /// Down presses (each firing before the prior ~110ms ease settled) then
    /// SNAPPED the drawn band to each stale intermediate target instead of
    /// gliding continuously — a visible pop that reads as "the highlight
    /// lags/jumps behind what Enter would actually run" (the selection-desync
    /// report). `overlay_band_drawn` already computed the correct current
    /// eased position (`cur`); this is that fix, promoted to the ONE shared
    /// owner so the two seams can never diverge again.
    ///
    /// A fresh overlay (`overlay_band_last == None`) SETTLES rather than
    /// easing — there is no meaningful previous row to glide from.
    fn retarget_band(&mut self, target: f32) {
        match self.overlay_band_last {
            Some(last) if (last - target).abs() > 0.5 => {
                // A selection move: start the slide FROM wherever the band is
                // drawn right now (mid-flight moves chain smoothly).
                let cur = if self.overlay_band_t < 1.0 {
                    let e = crate::ease::out_back(self.overlay_band_t);
                    self.overlay_band_from + (last - self.overlay_band_from) * e
                } else {
                    last
                };
                self.overlay_band_from = cur;
                self.overlay_band_t = 0.0;
                self.overlay_band_last = Some(target);
            }
            None => {
                self.overlay_band_from = target;
                self.overlay_band_last = Some(target);
                self.overlay_band_t = 1.0;
            }
            _ => {}
        }
    }

    /// ITEM 48 — THE HYBRID glide+snap ARBITER (the user's decision), the ONE
    /// door both band seams ([`Self::overlay_band_drawn`] +
    /// [`Self::living_band_phase`]) route their re-target through. It picks
    /// between the [`Self::retarget_band`] GLIDE (untouched — the correct,
    /// headless-verified chase) and an immediate SNAP, by INPUT RATE:
    ///
    /// * A SINGLE deliberate move (the band is SETTLED, `overlay_band_t >= 1.0`)
    ///   → GLIDE: hand off to `retarget_band`, which starts the living-band
    ///   choreography from the settled row and eases home. The whole morph plays.
    ///
    /// * A move that arrives while the band is STILL MID-GLIDE from a previous
    ///   UNFINISHED glide (`overlay_band_t < 1.0` — input outran the ~110ms
    ///   slide, i.e. arrow-key auto-repeat) → SNAP: jump `from`/`last` straight
    ///   to the freshest `target` so the drawn band == the selection THIS frame,
    ///   never a lagging intermediate. This is why held-down Down no longer
    ///   "catches up every 2nd row": the OLD path chained another glide from the
    ///   in-flight position and trailed; this teleports to the live selection.
    ///
    /// THE CLOCK TRICK for SUSTAINED repeat: a snap RESETS `overlay_band_t` to
    /// `0.0` (not `1.0`) with `from == last == target`, so [`livingband::morph_
    /// band`] draws the exact target rect at every phase (a no-move is a constant
    /// rect) WHILE the in-flight timer keeps running. `overlay_band_t` therefore
    /// measures "time since the last MOVE" (each move — glide-start OR snap —
    /// re-zeros it), so as long as auto-repeat keeps firing within one
    /// [`OVERLAY_BAND_SLIDE_MS`] the band stays in the snap regime and never
    /// settles into another lagging glide; the moment input goes quiet for a full
    /// glide duration, `overlay_band_t` reaches `1.0` and the NEXT move glides
    /// again. Never sets `1.0` on the snap, which would let the very next
    /// in-flight move read "settled" and glide (the "catches up every 2nd Down"
    /// alternation this closes).
    ///
    /// Live-only: every capture / unarmed / Reduce-Motion path settles the band
    /// BEFORE reaching here (see the two callers), so this arbiter is structurally
    /// unreachable in a deterministic capture — the byte-identity gates stand.
    fn chase_or_snap(&mut self, target: f32) {
        let in_flight_move = matches!(
            self.overlay_band_last,
            Some(last) if (last - target).abs() > 0.5
        ) && self.overlay_band_t < 1.0;
        if in_flight_move {
            self.overlay_band_from = target;
            self.overlay_band_last = Some(target);
            self.overlay_band_t = 0.0;
        } else {
            self.retarget_band(target);
        }
    }

    /// The selection BAND's drawn row-top for a target `row_top` this frame —
    /// the [`theme::BandResponse::Slide`] seam, called only by
    /// `overlay_draw_card`. Snap worlds (every world today), unarmed
    /// pipelines (every capture), and Reduce Motion all return `target`
    /// verbatim (byte-identical). A Slide world eases from the previous row's
    /// top with the same gentle overshoot spring as the entrance. Purely
    /// visual: the shaped rows and the hit-test never move.
    pub(in crate::render) fn overlay_band_drawn(&mut self, target: f32) -> f32 {
        let slide = self.juice_live
            && !crate::motion::reduced()
            && crate::render::effective_motion_juice().band == theme::BandResponse::Slide;
        if !slide {
            self.overlay_band_last = Some(target);
            self.overlay_band_t = 1.0;
            return target;
        }
        self.chase_or_snap(target);
        if self.overlay_band_t >= 1.0 {
            return target;
        }
        let e = crate::ease::out_back(self.overlay_band_t);
        self.overlay_band_from + (target - self.overlay_band_from) * e
    }

    /// ARM B LIVING-BAND PROBE — the band's TRAVEL (`from_top`, `to_top`) + PHASE
    /// `t` for the morph / two-shape choreography this frame. Two modes:
    ///
    /// * PINNED (`force.phase` set — the capture frame-dump path): a synthetic
    ///   travel from [`livingband::PIN_JUMP_ROWS`] rows BELOW the selected row,
    ///   sliding up to it, held at the fixed phase. Deterministic (no clock), so
    ///   `--screenshot` dumps a byte-stable mid-flight frame.
    /// * LIVE (`force.phase` absent): reuses the SAME `overlay_band_from/last/t`
    ///   tracking the ordinary slide uses, through the ONE hybrid arbiter
    ///   [`Self::chase_or_snap`] (ITEM 48). A fresh overlay settles; a single
    ///   deliberate move GLIDES via [`Self::retarget_band`] from where the band
    ///   is actually drawn (never the stale previous target); a move that outruns
    ///   the in-flight glide SNAPS straight to the freshest target so the band
    ///   can never trail the selection under auto-repeat — see `chase_or_snap`'s
    ///   doc. [`Self::step_overlay_juice`] advances `overlay_band_t`, and
    ///   Reduce Motion folds it to `1.0` (settled) — so the whole choreography
    ///   inherits the accessibility contract for free.
    ///
    /// Called ONLY from `overlay_draw_card`'s Pane arm when the probe is set; the
    /// ordinary path never reaches it, so an unset-env run is byte-identical.
    pub(in crate::render) fn living_band_phase(
        &mut self,
        force: livingband::MotionForce,
        target: f32,
        lh: f32,
    ) -> (f32, f32, f32) {
        if let Some(phase) = force.phase {
            let from = target + livingband::PIN_JUMP_ROWS * lh;
            return (from, target, phase.clamp(0.0, 1.0));
        }
        if !self.juice_live || crate::motion::reduced() {
            self.overlay_band_last = Some(target);
            self.overlay_band_t = 1.0;
            return (target, target, 1.0);
        }
        self.chase_or_snap(target);
        (
            self.overlay_band_from,
            self.overlay_band_last.unwrap_or(target),
            self.overlay_band_t,
        )
    }

    /// ARM B LIVING-BAND PROBE — the choreography's drawn rects this frame, from
    /// the pure phase math ([`livingband`]). Returns `(primary, echo, cross)`
    /// full-width row rects: `primary` for `overlay_rows` (the leading band),
    /// `echo` for `overlay_bars` (the chasing echo — empty for the single-band
    /// MORPH), and `cross` for `overlay_cross` (the brightest crossing — empty
    /// unless a two-shape overlap exists this frame). Pure over its inputs (no
    /// GPU, no clock); `&self` only.
    // Living-band geometry keeps the physical card and phase inputs explicit at this pure seam.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub(in crate::render) fn living_band_rects(
        &self,
        force: livingband::MotionForce,
        from: f32,
        to: f32,
        t: f32,
        card_x: f32,
        card_w: f32,
        lh: f32,
    ) -> (Vec<[f32; 4]>, Vec<[f32; 4]>, Vec<[f32; 4]>) {
        let params = force.choreo.params();
        if force.choreo.is_two_shape() {
            let s = livingband::two_shape_band(from, to, lh, t, &params);
            let primary = vec![[card_x, s.primary_top, card_w, s.height]];
            let echo = vec![[card_x, s.echo_top, card_w, s.height]];
            let cross = s
                .overlap
                .map(|o| vec![[card_x, o.top, card_w, o.height]])
                .unwrap_or_default();
            (primary, echo, cross)
        } else {
            let b = livingband::morph_band(from, to, lh, t, &params);
            (
                vec![[card_x, b.top, card_w, b.height]],
                Vec::new(),
                Vec::new(),
            )
        }
    }

    pub(in crate::render) fn overlay_slant_progress(&self) -> f32 {
        if let Some(m) = crate::render::overlay_motion_probe() {
            return crate::ease::out_back(m.enter);
        }
        if !self.juice_live || crate::motion::reduced() {
            return 1.0;
        }
        crate::ease::out_back(self.overlay_enter_t)
    }

    /// The selected-bar GROW-POP progress this frame (motion choreography 4): the
    /// fraction of the `grow_px` ledge currently extended. `1.0` (full ledge) in
    /// every capture / unarmed / CALM pipeline (byte-identical); pinned by the
    /// frame-dump probe; on a live Slide world it rides `overlay_band_t` so the
    /// ledge COLLAPSES then juts back out on each selection move (the grow and the
    /// band slide share one timer, one spring). Reduce Motion → `1.0`.
    pub(in crate::render) fn overlay_grow_progress(&self) -> f32 {
        if let Some(m) = crate::render::overlay_motion_probe() {
            return crate::ease::out_back(m.band);
        }
        if !self.juice_live || crate::motion::reduced() {
            return 1.0;
        }
        crate::ease::out_back(self.overlay_band_t)
    }

    pub(in crate::render) fn overlay_slant_dx(&self, row: usize) -> f32 {
        match crate::render::overlay_slant() {
            None => 0.0,
            Some(s) => crate::render::slant_offset(&s, row) * self.overlay_slant_progress(),
        }
    }

    pub fn effective_background(&self) -> theme::Background {
        crate::lava::env_override().unwrap_or_else(theme::background)
    }

    pub fn lava_render_phase(&self) -> f32 {
        let env = crate::lava::env_phase();
        crate::lava::lava_phase_for(self.lava_phase, crate::motion::reduced(), env)
    }

    pub fn stars_render_phase(&self) -> f32 {
        let env = crate::stars::env_phase();
        crate::lava::lava_phase_for(self.lava_phase, crate::motion::reduced(), env)
    }

    // Item 163: the third consumer of the same shared-clock env-override door
    // (mirrors the two above — see `crate::background::env_phase`'s own doc for
    // why one knob drives both Waves and Organic).
    pub fn waves_render_phase(&self) -> f32 {
        let env = crate::background::env_phase();
        crate::lava::lava_phase_for(self.lava_phase, crate::motion::reduced(), env)
    }

    /// THE WARPED GRID's effective route phase, in seconds — the resolver shape
    /// of the three above, over `crate::warpgrid`'s own knob and loop length.
    pub fn warp_render_phase(&self) -> f32 {
        let env = crate::warpgrid::env_phase();
        crate::warpgrid::phase_for(self.warp_phase, crate::motion::reduced(), env)
    }

    /// THE WARPED GRID's finished steering pose — `[yaw, pitch, forward_cells]`,
    /// all-zero for every other ground (so their upload is byte-identical). The
    /// route is `crate::warpgrid`'s; the shader receives only this result.
    pub fn warp_pose(&self) -> [f32; 3] {
        if !self.effective_background().is_warped_grid() {
            return [0.0; 3];
        }
        let p = crate::warpgrid::route_pose(self.warp_render_phase());
        [p.yaw, p.pitch, p.forward_cells]
    }

    /// Advance the lava lamp's animation phase by `dt` seconds — called ONLY by
    /// the live App's slow ambient tick (`App::about_to_wait`), NEVER `advance()`'s
    /// hot per-frame loop (the lava's whole point is a ~10 fps sparse cadence, not
    /// full refresh). Delayed wakes clamp to one ambient step and wrap over the
    /// field's full two-cycle period ([`crate::lava::advance_phase`]).
    /// THE APP'S ONE AMBIENT-ADVANCE DOOR — it advances EVERY accumulator the
    /// shared tick owns, so a future consumer cannot be forgotten at a second
    /// call site (there is only one).
    pub fn advance_lava(&mut self, dt: f32) {
        self.lava_phase = crate::lava::advance_phase(self.lava_phase, dt);
        self.warp_phase = crate::warpgrid::advance_phase(self.warp_phase, dt);
    }

    pub fn hold_lava_field_viewport(&mut self, width: u32, height: u32) {
        if self.lava_field_viewport[0] <= 0.0 || self.lava_field_viewport[1] <= 0.0 {
            self.lava_field_viewport = [width as f32, height as f32];
        }
    }

    pub fn settle_lava_field_viewport(&mut self, width: u32, height: u32) {
        self.lava_field_viewport = [width as f32, height as f32];
    }

    pub fn lava_blur_active(&self) -> bool {
        self.backdrop_blur()
    }

    /// Pin the lava lamp's phase to the FROZEN composition — the live App calls
    /// this when the lamp must be static (Reduce Motion, or `ambient_motion` off),
    /// so resuming from a hard-frozen state restarts from the settled frame rather
    /// than a stale mid-bob.
    pub fn freeze_lava(&mut self) {
        self.lava_phase = crate::lava::LAVA_FROZEN_PHASE;
        self.warp_phase = crate::warpgrid::FROZEN_PHASE;
    }

    /// COPY PULSE: kick the selection quad's brighten/decay AND the caret's own
    /// gentle pulse — a successful M-w/Cmd-C copy of a non-empty selection,
    /// otherwise entirely invisible. Resets [`Self::copy_pulse_t`] to 0 (full
    /// brighten); [`Self::step_copy_pulse`] eases it back to 1.0 (settled) over
    /// [`COPY_PULSE_MS`] on the live clock, consumed by
    /// [`Self::prepare_selection_layer`]. Idempotent under rapid re-fire (copying
    /// again mid-decay just restarts the pulse). Live-only: nothing in the
    /// headless `--keys` replay path calls this (see `main/run.rs`'s
    /// `Effect::CopyPulse` no-op arm), so a default capture never carries a boost.
    pub fn copy_pulse(&mut self) {
        self.copy_pulse_t = 0.0;
        self.caret.copy_pulse();
    }

    fn step_copy_pulse(&mut self, dt: f32) -> bool {
        // ACCESSIBILITY TIER 1 — REDUCE MOTION: settle the selection-tint
        // brighten INSTANTLY to its resting (fully-settled) value instead of
        // decaying over `dt` — same final color, zero frames of ease. Mirrors
        // `step_caret`'s gate exactly; see `motion.rs`'s determinism note (this
        // branch is unreachable from a headless capture path).
        if crate::motion::reduced() {
            self.copy_pulse_t = 1.0;
            return false;
        }
        if self.copy_pulse_t >= 1.0 {
            return false;
        }
        self.copy_pulse_t = (self.copy_pulse_t + dt * 1000.0 / COPY_PULSE_MS).min(1.0);
        self.copy_pulse_t < 1.0
    }

    /// The copy-pulse's EASED settle fraction THIS frame — 0.0 at the instant of
    /// the kick (full brighten), 1.0 once settled (the plain theme tint, and the
    /// permanent value in every headless capture). Smoothstep eased, mirroring
    /// [`crate::caret::CaretAnim::pop_scale`]'s ease exactly. Consumed by
    /// [`Self::prepare_selection_layer`] to blend the selection quad's color.
    pub(in crate::render) fn copy_pulse_settle(&self) -> f32 {
        copy_pulse_ease(self.copy_pulse_t)
    }

    fn step_caret_preview(&mut self, dt: f32) -> bool {
        if self.caret_preview.is_none() {
            return false;
        }
        if crate::motion::reduced() {
            self.caret_demo.settle();
            return false;
        }
        self.caret_demo.step(dt);
        true
    }
}
