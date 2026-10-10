//! Real retained preparation and modeled partial failure; no window or surface.
use super::*;

fn wait(device: &wgpu::Device) {
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(5)),
        })
        .expect("finite test-only completion maintenance");
}

pub(crate) fn assert_healthy_upload_completion() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::world("Mangrove").unwrap();
    for dpi in [1.0_f32, 2.0_f32] {
        let (device, queue, mut pipeline) =
            crate::test_gpu::with_shared_programs(|device, queue| {
                let cache = glyphon::Cache::new(device);
                let mut pipeline = crate::render::TextPipeline::new(
                    device,
                    queue,
                    &cache,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                );
                pipeline.set_size(960.0 * dpi, 640.0 * dpi);
                pipeline.set_dpi(dpi);
                (device.clone(), queue.clone(), pipeline)
            })
            .expect("Metal adapter required for this regression");
        assert!(crate::gpu_alloc::probe(&device).responds(crate::gpu_alloc::Class::Buffers));
        let width = (960.0 * dpi) as u32;
        let height = (640.0 * dpi) as u32;
        pipeline.set_text("");
        for _ in 0..4 {
            pipeline.prepare(&device, &queue, width, height).unwrap();
        }
        queue.submit([]);
        wait(&device);
        let partial = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bounded partial-prepare upload"),
            size: 16,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let base = crate::gpu_alloc::live(&device).buffers;
        for skip in [
            GpuFrameSkip::Timeout,
            GpuFrameSkip::Occluded,
            GpuFrameSkip::PrepareFailed,
            GpuFrameSkip::SurfaceReconfigured,
            GpuFrameSkip::SurfaceRecreated,
        ] {
            for cell in 1..=16 {
                if skip == GpuFrameSkip::PrepareFailed {
                    // A prepare can fail after writes but before returning its
                    // activity report. Stage a real write, model that error,
                    // and call the very same completion operation as Gpu.
                    queue.write_buffer(&partial, 0, &[0; 16]);
                } else {
                    pipeline.prepare(&device, &queue, width, height).unwrap();
                }
                let staged = crate::gpu_alloc::live(&device).buffers - base;
                assert!(staged < 4096, "finite cell staging budget");
                let outcome = complete_uploads(GpuFrameOutcome::Skipped(skip), Vec::new, || {
                    queue.submit([]);
                });
                assert!(matches!(outcome, GpuFrameOutcome::Skipped(s) if s == skip));
                // Wait does not submit pending writes. Removing submission
                // above therefore fails this allocation assertion immediately.
                wait(&device);
                let retained = crate::gpu_alloc::live(&device).buffers - base;
                assert!(
                    retained.abs() <= 4,
                    "dpi={dpi} skip={skip:?} cell={cell} retained={retained}"
                );
            }
            eprintln!(
                "healthy-upload Metal dpi={dpi} skip={skip:?} cells=16 retained_buffers={}",
                crate::gpu_alloc::live(&device).buffers - base
            );
        }
        pipeline.prepare(&device, &queue, width, height).unwrap();
        let outcome = complete_uploads(
            GpuFrameOutcome::Fault(fault(GpuFaultKind::Validation)),
            Vec::new,
            || {
                queue.submit([]);
            },
        );
        assert!(matches!(outcome, GpuFrameOutcome::Fault(f) if f.kind == GpuFaultKind::Validation));
        wait(&device);
        assert!((crate::gpu_alloc::live(&device).buffers - base).abs() <= 4);
    }
}
