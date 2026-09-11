//! Complete state owners for overlay entrance and selected-band transitions.

use super::*;

/// One complete overlay-entrance transition. Its phase never escapes this
/// owner, so a close/reopen cannot leave an old progress value behind.
pub(super) struct OverlayEntranceState {
    t: f32,
}

impl OverlayEntranceState {
    pub(super) const fn settled() -> Self {
        Self { t: 1.0 }
    }

    pub(super) fn start(&mut self) {
        self.t = 0.0;
    }

    pub(super) fn settle(&mut self) {
        self.t = 1.0;
    }

    pub(super) fn advance(&mut self, dt: f32) {
        self.t = (self.t + OVERLAY_ENTRANCE_MS.progress_per(dt)).min(1.0);
    }

    pub(super) fn active(&self) -> bool {
        self.t < 1.0
    }

    pub(super) fn progress(&self) -> f32 {
        self.t
    }
}

/// One complete selected-band transition, including its live input epoch.
/// Keeping the source, target, phase and pending epoch together makes a partial
/// reset unrepresentable to callers.
pub(super) struct OverlayBandState {
    from: f32,
    t: f32,
    last: Option<f32>,
    started_at: Option<crate::clock::Instant>,
    frame_now: Option<crate::clock::Instant>,
    pending_at: Option<crate::clock::Instant>,
    pending_from: f32,
    pending_snap: bool,
}

impl OverlayBandState {
    pub(super) const fn settled() -> Self {
        Self {
            from: 0.0,
            t: 1.0,
            last: None,
            started_at: None,
            frame_now: None,
            pending_at: None,
            pending_from: 0.0,
            pending_snap: false,
        }
    }

    pub(super) fn reset(&mut self) {
        *self = Self::settled();
    }

    pub(super) fn settle_to(&mut self, target: f32) {
        // A settled endpoint has no claim on an old input or presentation epoch.
        // Replace the whole state rather than clearing a hand-picked subset.
        *self = Self {
            from: target,
            last: Some(target),
            ..Self::settled()
        };
    }

    pub(super) fn settle(&mut self) {
        self.settle_to(self.last.unwrap_or(self.from));
    }

    pub(super) fn active(&self) -> bool {
        self.t < 1.0
    }

    pub(super) fn progress(&self) -> f32 {
        self.t
    }

    pub(super) fn source(&self) -> f32 {
        self.from
    }

    pub(super) fn target(&self) -> Option<f32> {
        self.last
    }

    #[cfg(test)]
    pub(super) fn arm_for_activity_law(&mut self) {
        self.started_at = None;
        self.t = 0.0;
    }

    pub(super) fn advance(&mut self, dt: f32) {
        if self.started_at.is_none() && self.active() {
            self.t = (self.t + OVERLAY_BAND_SLIDE_MS.progress_per(dt)).min(1.0);
        }
    }

    pub(super) fn retarget(&mut self, target: f32) {
        match self.last {
            Some(last) if (last - target).abs() > 0.5 => {
                self.from = if self.active() {
                    let e = crate::ease::out_back(self.t);
                    self.from + (last - self.from) * e
                } else {
                    last
                };
                self.t = 0.0;
                self.last = Some(target);
                self.started_at = None;
            }
            None => self.settle_to(target),
            _ => {}
        }
    }

    pub(super) fn chase(&mut self, target: f32) {
        if self.consume_input(target) {
            return;
        }
        let in_flight_move =
            matches!(self.last, Some(last) if (last - target).abs() > 0.5) && self.active();
        if in_flight_move {
            self.from = target;
            self.last = Some(target);
            self.t = 0.0;
            self.started_at = None;
        } else {
            self.retarget(target);
        }
    }

    pub(super) fn drawn(&self, target: f32) -> f32 {
        if self.active() {
            let e = crate::ease::out_back(self.t);
            self.from + (target - self.from) * e
        } else {
            target
        }
    }

    pub(super) fn stamp_input(&mut self, at: crate::clock::Instant) {
        self.sample(at);
        self.pending_from = self.drawn(self.last.unwrap_or(self.from));
        self.pending_snap = self.pending_at.is_some() || self.active();
        self.pending_at = Some(at);
    }

    pub(super) fn begin_frame(&mut self, now: crate::clock::Instant) {
        self.frame_now = Some(now);
        self.sample(now);
    }

    fn sample(&mut self, now: crate::clock::Instant) {
        if let Some(started_at) = self.started_at {
            self.t = OVERLAY_BAND_SLIDE_MS
                .progress_per(now.saturating_duration_since(started_at).as_secs_f32())
                .min(1.0);
            if !self.active() {
                self.started_at = None;
            }
        }
    }

    fn consume_input(&mut self, target: f32) -> bool {
        let Some(at) = self.pending_at.take() else {
            return false;
        };
        let Some(last) = self.last else {
            self.settle_to(target);
            return true;
        };
        if (last - target).abs() <= 0.5 {
            return true;
        }
        self.from = if self.pending_snap {
            target
        } else {
            self.pending_from
        };
        self.last = Some(target);
        self.started_at = Some(at);
        self.sample(self.frame_now.unwrap_or(at));
        true
    }

    #[cfg(test)]
    pub(in crate::render) fn last(&self) -> Option<f32> {
        self.last
    }

    #[cfg(test)]
    pub(in crate::render) fn pending_snap(&self) -> bool {
        self.pending_snap
    }

    #[cfg(test)]
    pub(in crate::render) fn epoch(
        &self,
    ) -> (
        f32,
        Option<crate::clock::Instant>,
        Option<crate::clock::Instant>,
        Option<crate::clock::Instant>,
        f32,
        bool,
    ) {
        (
            self.from,
            self.started_at,
            self.frame_now,
            self.pending_at,
            self.pending_from,
            self.pending_snap,
        )
    }
}
