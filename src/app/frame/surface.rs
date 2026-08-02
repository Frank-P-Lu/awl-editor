use super::*;

pub(super) struct SurfaceState {
    gpu: Option<Gpu>,
    recovery_window: Option<Arc<Window>>,
    lifecycle: GpuLifecycle,
    retry_at: Option<Instant>,
    timeout_streak: u8,
    present_sync: PresentSyncShadow,
    #[cfg(target_arch = "wasm32")]
    pending: std::rc::Rc<std::cell::RefCell<Option<Result<Gpu, String>>>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PresentSyncShadow {
    on: bool,
    valid: bool,
}

impl PresentSyncShadow {
    fn invalidate(&mut self) {
        self.valid = false;
    }

    fn apply(&mut self, want: bool) -> bool {
        if self.valid && self.on == want {
            return false;
        }
        self.on = want;
        self.valid = true;
        true
    }
}

impl SurfaceState {
    pub(super) fn new() -> Self {
        Self {
            gpu: None,
            recovery_window: None,
            lifecycle: GpuLifecycle::AwaitingWindow,
            retry_at: None,
            timeout_streak: 0,
            present_sync: PresentSyncShadow::default(),
            #[cfg(target_arch = "wasm32")]
            pending: std::rc::Rc::new(std::cell::RefCell::new(None)),
        }
    }

    pub(super) fn gpu(&self) -> Option<&Gpu> {
        self.gpu.as_ref()
    }

    pub(super) fn gpu_mut(&mut self) -> Option<&mut Gpu> {
        self.gpu.as_mut()
    }

    pub(super) fn install_gpu(&mut self, gpu: Gpu) {
        self.gpu = Some(gpu);
        self.present_sync.invalidate();
    }

    pub(super) fn clear_gpu(&mut self) {
        self.gpu = None;
        self.present_sync.invalidate();
    }

    pub(super) fn recovery_window(&self) -> Option<&Arc<Window>> {
        self.recovery_window.as_ref()
    }

    pub(super) fn set_recovery_window(&mut self, window: Arc<Window>) {
        self.recovery_window = Some(window);
    }

    pub(super) fn lifecycle(&self) -> GpuLifecycle {
        self.lifecycle
    }

    pub(super) fn set_lifecycle(&mut self, lifecycle: GpuLifecycle) {
        self.lifecycle = lifecycle;
    }

    pub(super) fn retry_at(&self) -> Option<Instant> {
        self.retry_at
    }

    pub(super) fn arm_retry(&mut self, deadline: Instant) {
        self.retry_at = Some(deadline);
    }

    pub(super) fn clear_retry(&mut self) {
        self.retry_at = None;
    }

    pub(super) fn timeout_streak(&self) -> u8 {
        self.timeout_streak
    }

    pub(super) fn record_timeout(&mut self, timed_out: bool) {
        self.timeout_streak = if timed_out {
            self.timeout_streak.saturating_add(1)
        } else {
            0
        };
    }

    pub(super) fn clear_timeout_streak(&mut self) {
        self.timeout_streak = 0;
    }

    #[cfg(test)]
    pub(super) fn invalidate_present_sync(&mut self) {
        self.present_sync.invalidate();
    }

    /// Update the shadow and report whether the current layer needs the value
    /// applied. A replacement GPU always invalidates the shadow first.
    pub(super) fn apply_present_sync(&mut self, want: bool) -> bool {
        self.present_sync.apply(want)
    }

    pub(super) fn present_sync_on(&self) -> bool {
        self.present_sync.on
    }

    #[cfg(test)]
    pub(super) fn present_sync_valid(&self) -> bool {
        self.present_sync.valid
    }

    pub(super) fn suspend(&mut self) {
        self.clear_gpu();
        self.lifecycle = GpuLifecycle::Suspended;
        self.retry_at = None;
        self.timeout_streak = 0;
        self.present_sync.on = false;
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn pending_slot(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<Option<Result<Gpu, String>>>> {
        self.pending.clone()
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn take_pending(&self) -> Option<Result<Gpu, String>> {
        self.pending.borrow_mut().take()
    }
}

#[cfg(test)]
mod tests {
    use super::PresentSyncShadow;

    #[test]
    fn invalidation_forces_the_same_present_value_onto_a_replacement_layer() {
        let mut shadow = PresentSyncShadow::default();
        assert!(shadow.apply(true), "the first layer must receive the value");
        assert!(!shadow.apply(true), "an unchanged live layer is idempotent");
        shadow.invalidate();
        assert!(
            shadow.apply(true),
            "the same desired value must be re-applied to a replacement layer"
        );
    }
}
