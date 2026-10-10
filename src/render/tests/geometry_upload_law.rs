//! Pending uploads on a retained destination need submission, not poll alone.
use super::{OverlayKind, card_fit, headless_dqp};
use crate::gpu_alloc;

pub(super) fn drain(device: &wgpu::Device, queue: &wgpu::Queue) {
    let submitted = queue.submit(std::iter::empty());
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submitted),
            timeout: Some(std::time::Duration::from_secs(10)),
        })
        .expect("geometry-only uploads must complete within the finite drain budget");
}

#[test]
fn repeated_uploads_to_one_retained_texture_need_submission() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = crate::test_gpu::shared_device_queue() else {
        eprintln!("skipping retained-texture upload law: no wgpu adapter");
        return;
    };
    assert!(gpu_alloc::probe(&device).responds(gpu_alloc::Class::Buffers));
    drain(&device, &queue);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("retained upload destination"),
        size: wgpu::Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let base = gpu_alloc::live(&device).buffers;
    let mut samples = Vec::new();
    for _ in 0..2 {
        for _ in 0..32 {
            queue.write_texture(
                texture.as_image_copy(),
                &[0; 256],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(32),
                    rows_per_image: Some(8),
                },
                texture.size(),
            );
        }
        samples.push(gpu_alloc::live(&device).buffers - base);
    }
    device
        .poll(wgpu::PollType::Poll)
        .expect("poll-only measurement");
    let polled = gpu_alloc::live(&device).buffers - base;
    drain(&device, &queue);
    let drained = gpu_alloc::live(&device).buffers - base;
    eprintln!(
        "retained-texture staging buffers: batches={samples:?} poll_only={polled} drained={drained}"
    );
    assert!(
        samples[0] >= 32 && samples[1] >= samples[0] + 32,
        "repeated writes must exercise pending staging allocation: {samples:?}"
    );
    assert!(
        polled >= samples[1],
        "poll alone unexpectedly submitted pending writes"
    );
    assert_eq!(
        drained, 0,
        "submitted completed staging must return to its settled baseline"
    );
    // Keep the destination live through the final measurement.
    assert_eq!(texture.width(), 8);
}

#[test]
fn geometry_only_cells_do_not_retain_a_sweeps_uploads() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping geometry-only upload bound: no wgpu adapter");
        return;
    };
    assert!(gpu_alloc::probe(&device).responds(gpu_alloc::Class::Buffers));
    for _ in 0..4 {
        card_fit(
            &mut pipeline,
            (&device, &queue),
            (1200, 800),
            OverlayKind::Keybindings,
            1.0,
            vec!["A retained pipeline upload measurement".into()],
        );
    }
    drain(&device, &queue);
    let base = gpu_alloc::live(&device).buffers;
    let mut peak = 0;
    for cell in 0..128 {
        card_fit(
            &mut pipeline,
            (&device, &queue),
            (1200, 800),
            OverlayKind::Keybindings,
            1.0,
            vec![format!("Retained pipeline upload measurement {cell}")],
        );
        peak = peak.max(gpu_alloc::live(&device).buffers - base);
    }
    // Read BEFORE any final settling, so deleting card_fit's drain fails here.
    let last = gpu_alloc::live(&device).buffers - base;
    eprintln!("geometry-only retained buffers: 128 cells peak={peak} last={last}");
    assert!(
        peak <= 4 && last <= 4,
        "geometry sweep retained staging: peak={peak} last={last}"
    );
}
