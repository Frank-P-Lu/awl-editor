use super::*;

impl App {
    pub(super) fn rebuild_gpu(&mut self, event_loop: &ActiveEventLoop, reason: &str) {
        let window = match self.frame.begin_gpu_rebuild() {
            frame::GpuRebuildStart::AlreadyRunning => return,
            frame::GpuRebuildStart::NoWindow => {
                event_loop.exit();
                return;
            }
            frame::GpuRebuildStart::Ready(window) => window,
        };
        self.set_sticky_notice(format!("{reason} — rebuilding graphics…"));
        let display_handle = event_loop.owned_display_handle();
        #[cfg(not(target_arch = "wasm32"))]
        match pollster::block_on(Gpu::new(window, display_handle)) {
            Ok(gpu) => {
                self.frame.activate_gpu(gpu);
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
            let slot = self.frame.gpu_pending_slot();
            let wake = window.clone();
            wasm_bindgen_futures::spawn_local(async move {
                *slot.borrow_mut() = Some(
                    Gpu::new(window, display_handle)
                        .await
                        .map_err(|error| error.to_string()),
                );
                super::redraw::request_window(&wake);
            });
        }
    }
}
