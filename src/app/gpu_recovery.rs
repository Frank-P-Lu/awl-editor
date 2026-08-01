use super::*;

impl App {
    pub(super) fn rebuild_gpu(&mut self, event_loop: &ActiveEventLoop, reason: &str) {
        if self.gpu_lifecycle == GpuLifecycle::Rebuilding {
            return;
        }
        let Some(window) = self.recovery_window.clone() else {
            event_loop.exit();
            return;
        };
        self.gpu = None;
        self.gpu_lifecycle = GpuLifecycle::Rebuilding;
        self.last_frame = None;
        self.gpu_retry_at = None;
        self.gpu_timeout_streak = 0;
        self.input_stamp = None;
        self.set_sticky_notice(format!("{reason} — rebuilding graphics…"));
        let display_handle = event_loop.owned_display_handle();
        #[cfg(not(target_arch = "wasm32"))]
        match pollster::block_on(Gpu::new(window, display_handle)) {
            Ok(gpu) => {
                self.gpu = Some(gpu);
                self.gpu_lifecycle = GpuLifecycle::Active { oom_skips: 0 };
                self.set_toast_notice("graphics recovered");
                self.on_gpu_ready();
            }
            Err(error) => {
                eprintln!("failed to rebuild render state: {error}");
                self.set_sticky_notice("graphics could not recover — closing safely");
                event_loop.exit();
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let slot = self.gpu_pending.clone();
            let wake = window.clone();
            wasm_bindgen_futures::spawn_local(async move {
                *slot.borrow_mut() = Some(
                    Gpu::new(window, display_handle)
                        .await
                        .map_err(|error| error.to_string()),
                );
                wake.request_redraw();
            });
        }
    }
}
