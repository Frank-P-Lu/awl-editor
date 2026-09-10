use anyhow::{Context as _, ensure};

use crate::clock::Instant;

fn percentile(mut values: Vec<u128>, pct: usize) -> u128 {
    values.sort_unstable();
    values[(values.len() - 1) * pct / 100]
}

/// Isolate the render owner changed by the projected tunnel: uniform upload,
/// command encoding, the warped-grid draw itself, submit, and serialized GPU
/// completion. The active-world assertions are the non-vacuity witness.
pub(super) fn profile(device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
    const WIDTH: u32 = 2400;
    const HEIGHT: u32 = 1600;
    const DPI: f32 = 2.0;
    const WARMUP: usize = 30;
    const SAMPLES: usize = 300;

    let restore = crate::theme::active_index();
    crate::theme::set_active_by_name("Kite").context("Kite benchmark world is absent")?;
    let ribs = match crate::theme::active().background {
        crate::theme::Background::WarpedGrid { ribs, .. } => ribs,
        _ => anyhow::bail!("Kite benchmark did not enrol the warped-grid background"),
    };
    let desc = super::super::background_desc();
    ensure!(
        desc.shader == 10,
        "Kite benchmark would not execute the warped-grid draw"
    );
    let mut background =
        crate::background::BackgroundPipeline::new(device, crate::capture::FORMAT, desc);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl warped-grid frame benchmark target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::capture::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut samples = Vec::with_capacity(SAMPLES);
    for frame in 0..(WARMUP + SAMPLES) {
        let t0 = Instant::now();
        background.prepare(
            queue,
            WIDTH,
            HEIGHT,
            624.0,
            1152.0,
            crate::background::AmbientUpload {
                warp_travel: frame as f32 * 0.011,
                warp_axis: (0.80, 0.24),
                ..Default::default()
            },
            DPI,
        );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("awl warped-grid frame benchmark encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("awl warped-grid frame benchmark pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            background.draw(&mut pass);
        }
        queue.submit(Some(encoder.finish()));
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("warped-grid benchmark device poll failed")?;
        if frame >= WARMUP {
            samples.push(t0.elapsed().as_nanos());
        }
    }
    let med = percentile(samples.clone(), 50);
    let p95 = percentile(samples, 95);
    println!(
        "warped-grid draw witness — world=Kite ribs={} 2400x1600@2x samples={SAMPLES}",
        ribs.round() as u32
    );
    println!(
        "warped-grid frame | median {:.3} ms ({:.1}% of 16.667 ms) | \
         p95 {:.3} ms ({:.1}% of 16.667 ms)",
        med as f64 / 1.0e6,
        med as f64 / 16_667_000.0 * 100.0,
        p95 as f64 / 1.0e6,
        p95 as f64 / 16_667_000.0 * 100.0,
    );
    crate::theme::set_active(restore);
    Ok(())
}
