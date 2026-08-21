//! Encode, submit, and present after the surface-acquire decision is complete.

use super::*;

impl Gpu {
    pub(super) fn present_acquired(
        &mut self,
        frame: wgpu::SurfaceTexture,
        debug: bool,
        t0: Option<Instant>,
        prepare_ms: Option<f32>,
        activities: PreparedActivities,
    ) -> PreparedFrame {
        let t2 = debug.then(Instant::now);
        // Render through the sRGB VIEW format. On web this view supplies the
        // linear-to-sRGB encode; on native it is the configured format itself.
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.view_format),
            ..Default::default()
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("awl frame encoder"),
            });
        if let Err(e) = self.pipeline.render(&mut encoder, &view) {
            eprintln!("render error: {e}");
        }
        // The live probe mirror belongs to the same submission as the pixels
        // handed to the compositor. Ordinary launches take no branch here.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::live_active() {
            self.mirror_presented_frame(&mut encoder, &frame.texture);
        }
        self.queue.submit(Some(encoder.finish()));
        // Wayland consumes this frame callback hint; other hosts accept it as a
        // no-op. It remains paired directly with the present it describes.
        self.window.pre_present_notify();
        frame.present();
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!("present"));
            crate::probe::note_presented_frame();
        }
        let done = debug.then(Instant::now);
        self.pipeline.atlas.trim();
        let outcome = match (prepare_ms, t0, t2, done) {
            (Some(prep), Some(t0), Some(t2), Some(done)) => {
                let present_ms = (done - t2).as_secs_f32() * 1000.0;
                self.debug_frame_split =
                    Some((prep, (t2 - t0).as_secs_f32() * 1000.0 - prep, present_ms));
                GpuFrameOutcome::Presented(Some((prep + present_ms, done)))
            }
            _ => {
                self.debug_frame_split = None;
                GpuFrameOutcome::Presented(None)
            }
        };
        PreparedFrame {
            outcome,
            activities,
        }
    }
}
