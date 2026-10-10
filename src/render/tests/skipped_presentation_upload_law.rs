//! Real Metal preparation with a modeled failed acquire; no native window.
//! Holds one empty Mangrove pipeline across32 cells, matching soak's initial
//! preparation workload. Actual surface acquisition/IME/event scheduling are
//! deliberately absent: this isolates pending uploads, not their live trigger.
use super::headless_dqp;
use crate::gpu_alloc;

fn metal_bytes(device: &wgpu::Device) -> u64 {
    use objc2_metal::MTLDevice;
    // Read-only borrowed handle, identical to live Gpu::current_gpu_bytes.
    unsafe { device.as_hal::<wgpu::hal::api::Metal>() }
        .expect("this backend comparison requires Metal")
        .raw_device()
        .currentAllocatedSize() as u64
}

fn drain(device: &wgpu::Device, queue: &wgpu::Queue) -> i64 {
    let submitted = queue.submit([]);
    let after_submit = gpu_alloc::live(device).buffers;
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submitted),
            timeout: Some(std::time::Duration::from_secs(5)),
        })
        .expect("finite test-only upload drain");
    after_submit
}

#[derive(Clone, Copy, Debug)]
enum Path {
    PrepareSkip,
    PrepareSkipPoll,
    PrepareSkipSubmit,
    AcquireFailBeforePrepare,
}

#[test]
fn retained_mangrove_skipped_presentation_upload_comparison() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::world("Mangrove").unwrap();
    for dpi in [1.0_f32, 2.0_f32] {
        let width = (960.0 * dpi) as u32;
        let height = (640.0 * dpi) as u32;
        let (device, queue, mut pipeline) =
            headless_dqp(width as f32, height as f32).expect("Metal adapter required");
        pipeline.set_dpi(dpi);
        assert!(gpu_alloc::probe(&device).responds(gpu_alloc::Class::Buffers));
        pipeline.set_text("");
        for _ in 0..4 {
            pipeline.prepare(&device, &queue, width, height).unwrap();
        }
        drain(&device, &queue);
        for path in [
            Path::PrepareSkip,
            Path::PrepareSkipPoll,
            Path::PrepareSkipSubmit,
            Path::AcquireFailBeforePrepare,
        ] {
            drain(&device, &queue);
            let base = gpu_alloc::live(&device).buffers;
            let base_bytes = metal_bytes(&device);
            let mut readings = Vec::new();
            for cell in 1..=32 {
                if !matches!(path, Path::AcquireFailBeforePrepare) {
                    pipeline.prepare(&device, &queue, width, height).unwrap();
                }
                // The live acquire early return omits present_acquired/submit.
                // Control alternatives are confined to this test, not production.
                match path {
                    Path::PrepareSkipPoll => {
                        device.poll(wgpu::PollType::Poll).unwrap();
                    }
                    Path::PrepareSkipSubmit => {
                        drain(&device, &queue);
                    }
                    _ => {}
                }
                let delta = gpu_alloc::live(&device).buffers - base;
                let bytes = metal_bytes(&device).saturating_sub(base_bytes);
                assert!(
                    delta < 4096 && bytes < 64 * 1024 * 1024,
                    "bounded32-cell comparison exceeded its allocation budget"
                );
                if matches!(cell, 8 | 16 | 32) {
                    readings.push((cell, delta, bytes));
                }
            }
            device.poll(wgpu::PollType::Poll).unwrap();
            let polled = gpu_alloc::live(&device).buffers - base;
            let after_submit = drain(&device, &queue) - base;
            let drained = gpu_alloc::live(&device).buffers - base;
            eprintln!(
                "skipped-upload Metal dpi={dpi} path={path:?} readings={readings:?} \
                 poll_only={polled} after_submit_before_wait={after_submit} drained={drained} \
                 metal_after_drain={} base_metal={base_bytes}",
                metal_bytes(&device)
            );
            match path {
                Path::PrepareSkip | Path::PrepareSkipPoll => {
                    assert!(
                        readings[0].1 > 0 && readings[2].1 >= readings[0].1 * 3,
                        "real unchanged preparation must stage growing allocations"
                    );
                    assert!(
                        polled >= readings[2].1,
                        "poll alone must not drain unsubmitted staging"
                    );
                }
                Path::PrepareSkipSubmit | Path::AcquireFailBeforePrepare => {
                    assert!(
                        readings.iter().all(|x| x.1 <= 4),
                        "submission/avoiding prepare must bound live staging"
                    );
                }
            }
            assert!(
                drained.abs() <= 4,
                "finite completed submit must return retained pipeline to baseline"
            );
        }
    }
}

#[test]
fn healthy_nonpresenting_completion_releases_full_and_partial_metal_uploads() {
    crate::app::assert_healthy_upload_completion();
}
