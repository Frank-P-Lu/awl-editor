use super::*;

/// Effects owed by one idle poll. This is a fixed set of frame-domain facts,
/// not an extensible message queue; `App` remains the interpreter for writes
/// and document reshaping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::app) struct PollOutcome {
    pub(in crate::app) redraw: bool,
    pub(in crate::app) settle_theme_tail: bool,
    pub(in crate::app) persist_zoom: bool,
    pub(in crate::app) expire_notice: bool,
    pub(in crate::app) retry: bool,
    pub(in crate::app) next_deadline: Option<Instant>,
}

pub(super) struct Deadlines {
    pub(super) clock: Box<dyn crate::clock::Clock>,
    pub(super) lava_tick_at: Option<Instant>,
    pub(super) resize_settle_at: Option<Instant>,
    pub(super) move_settle_at: Option<Instant>,
    pub(super) crossing_settle_at: Option<Instant>,
    pub(super) crossing_teardown_pending: bool,
    pub(super) zoom_persist_at: Option<Instant>,
    pub(super) focused: bool,
    pub(super) occluded: bool,
}

#[derive(Default)]
pub(super) struct NoticeState {
    pub(super) text: Option<String>,
    pub(super) kind: NoticeKind,
    pub(super) expires_at: Option<Instant>,
}

#[derive(Clone, Copy)]
pub(in crate::app) struct NoticeSnapshot<'a> {
    text: Option<&'a str>,
    kind: NoticeKind,
}

impl<'a> NoticeSnapshot<'a> {
    pub(in crate::app) fn text(self) -> Option<&'a str> {
        self.text
    }

    pub(in crate::app) fn owned(self) -> Option<String> {
        self.text.map(str::to_owned)
    }

    pub(in crate::app) fn active(self) -> bool {
        self.text.is_some()
    }

    pub(in crate::app) fn kind(self) -> NoticeKind {
        self.kind
    }
}

fn propose(out: &mut PollOutcome, deadline: Instant) {
    out.next_deadline = Some(
        out.next_deadline
            .map_or(deadline, |current| current.min(deadline)),
    );
}

impl FrameRuntime {
    /// Poll the coupled frame lifecycle at one injected-clock instant.
    /// Mutable input, document, and configuration owners stay outside this
    /// boundary; only their copyable scheduling facts cross it.
    pub(in crate::app) fn poll(
        &mut self,
        now: Instant,
        input: input::SchedulingSnapshot,
        config: location::SchedulingSnapshot,
    ) -> PollOutcome {
        let mut out = PollOutcome::default();
        self.poll_debounces(now, input.zoom_persist_held, &mut out);
        self.poll_settles(now, &mut out);
        self.poll_ambient(now, config, &mut out);
        self.poll_lifetimes(now, &mut out);
        out
    }

    fn poll_debounces(&mut self, now: Instant, zoom_persist_held: bool, out: &mut PollOutcome) {
        if let Some(dirty) = self.deadlines.zoom_persist_at
            && !zoom_persist_held
        {
            let deadline = dirty + ZOOM_PERSIST_DEBOUNCE;
            if now >= deadline {
                self.deadlines.zoom_persist_at = None;
                out.persist_zoom = true;
                out.redraw = true;
            } else {
                propose(out, deadline);
            }
        }
    }

    fn poll_settles(&mut self, now: Instant, out: &mut PollOutcome) {
        if let Some(dirty) = self.deadlines.resize_settle_at {
            let deadline = dirty + RESIZE_SYNC_SETTLE;
            if now >= deadline {
                self.deadlines.resize_settle_at = None;
                if let Some(gpu) = self.surface.gpu_mut() {
                    gpu.pipeline
                        .settle_lava_field_viewport(gpu.config.width, gpu.config.height);
                }
                out.redraw = true;
            } else {
                propose(out, deadline);
            }
        }
        if let Some(dirty) = self.deadlines.move_settle_at {
            let deadline = dirty + MOVE_SETTLE;
            if now >= deadline {
                self.deadlines.move_settle_at = None;
                self.deadlines.lava_tick_at = None;
                out.redraw = true;
            } else {
                propose(out, deadline);
            }
        }
        if let Some(dirty) = self.deadlines.crossing_settle_at {
            let deadline = dirty + CROSSING_SYNC_SETTLE;
            if now >= deadline {
                self.deadlines.crossing_settle_at = None;
                self.deadlines.crossing_teardown_pending = true;
                out.settle_theme_tail = true;
                out.redraw = true;
            } else {
                propose(out, deadline);
            }
        }
    }

    fn poll_ambient(
        &mut self,
        now: Instant,
        config: location::SchedulingSnapshot,
        out: &mut PollOutcome,
    ) {
        let lava_active = crate::theme::active().has_ambient_tick();
        let lava_paused = crate::lava::lava_paused(
            self.deadlines.resize_settle_at.is_some(),
            self.deadlines.move_settle_at.is_some(),
            self.surface
                .gpu()
                .is_some_and(|gpu| gpu.pipeline.lava_blur_active()),
        );
        if crate::lava::lava_should_tick(
            lava_active,
            config.ambient_motion_on(),
            crate::motion::reduced(),
            self.deadlines.focused && !self.deadlines.occluded,
            lava_paused,
        ) {
            match self.deadlines.lava_tick_at {
                Some(last) if now.saturating_duration_since(last) >= LAVA_TICK => {
                    let dt = (now - last).as_secs_f32();
                    self.deadlines.lava_tick_at = Some(now);
                    if let Some(gpu) = self.surface.gpu_mut() {
                        gpu.pipeline.advance_lava(dt);
                        out.redraw = true;
                    }
                }
                _ => {
                    let last = *self.deadlines.lava_tick_at.get_or_insert(now);
                    propose(out, last + LAVA_TICK);
                }
            }
        } else if lava_active {
            self.deadlines.lava_tick_at = None;
            if (crate::motion::reduced() || !config.ambient_motion_on())
                && let Some(gpu) = self.surface.gpu_mut()
            {
                gpu.pipeline.freeze_lava();
            }
        }
    }

    fn poll_lifetimes(&mut self, now: Instant, out: &mut PollOutcome) {
        if self.notice.kind == NoticeKind::Toast
            && self
                .notice
                .expires_at
                .is_some_and(|deadline| now >= deadline)
        {
            self.notice = NoticeState::default();
            out.expire_notice = true;
            out.redraw = true;
        } else if let Some(deadline) = self.notice.expires_at {
            propose(out, deadline);
        }
        if let Some(deadline) = self.surface.retry_at() {
            if now >= deadline {
                self.surface.clear_retry();
                out.retry = true;
                out.redraw = true;
            } else {
                propose(out, deadline);
            }
        }
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

    pub(in crate::app) fn notice(&self) -> NoticeSnapshot<'_> {
        NoticeSnapshot {
            text: self.notice.text.as_deref(),
            kind: self.notice.kind,
        }
    }
}
