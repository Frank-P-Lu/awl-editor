//! Unit + headless-GPU tests for `selection.rs`'s pure geometry, the shared
//! pipeline construction, and (item 131b) the rotated-quad primitive. Carved
//! out to its own file — `production()`'s "sibling `tests.rs`" exemption
//! (`scripts/code-health.py`) — once item 131b's rotation harness pushed the
//! inline `mod tests` past the file's own production line ceiling; no
//! behavior moved, only which file counts the lines.

use super::*;

#[test]
fn srgba_linear_alpha_passthrough() {
    let c = srgba_u8_to_linear([0x3A, 0x6F, 0xD8, 0x52]);
    assert!((c[3] - 0.32156864).abs() < 1e-4);
    for channel in c.iter().take(3) {
        assert!(*channel >= 0.0 && *channel <= 1.0);
    }
}

/// **BIT-IDENTITY, OVER EVERY BYTE.** `srgba_u8_to_linear` used to carry its
/// own inline per-channel loop; it now calls `theme::srgb_channel_to_linear_f32`.
/// This is the pre-refactor formula, written out independently (mirrors
/// `background::tests`'s identical law) so a regression in the shared owner
/// cannot also hide from the test meant to catch it.
#[test]
fn srgba_u8_to_linear_is_bit_identical_to_the_pre_refactor_formula_over_every_byte() {
    fn reference_channel(u: u8) -> f32 {
        let s = u as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    for v in 0u8..=255 {
        let want = reference_channel(v);
        let c = srgba_u8_to_linear([v, v, v, v]);
        for (i, ch) in c.iter().take(3).enumerate() {
            assert_eq!(
                ch.to_bits(),
                want.to_bits(),
                "byte {v} channel {i}: got {ch} ({:#010x}), want {want} ({:#010x})",
                ch.to_bits(),
                want.to_bits()
            );
        }
        assert_eq!(c[3], v as f32 / 255.0, "alpha stays a linear passthrough");
    }
}

#[test]
fn lerp4_interpolates_linearly_between_endpoints() {
    let a = [0.0, 0.2, 1.0, 0.5];
    let b = [1.0, 0.8, 0.0, 0.1];
    let at0 = lerp4(a, b, 0.0);
    let at1 = lerp4(a, b, 1.0);
    for k in 0..4 {
        assert!((at0[k] - a[k]).abs() < 1e-6, "t=0 must be the first color");
        assert!((at1[k] - b[k]).abs() < 1e-6, "t=1 must be the second color");
    }
    let mid = lerp4(a, b, 0.5);
    for k in 0..4 {
        assert!(
            (mid[k] - (a[k] + b[k]) / 2.0).abs() < 1e-6,
            "channel {k} must be the midpoint"
        );
    }
}

/// Regression: growing the instance buffer must size it to the FULL
/// power-of-two capacity, not the current contents. Otherwise a later frame
/// whose count sits between the grow-time count and the cap overruns the
/// buffer — the wgpu "Copy … would overrun the Destination buffer" write_buffer
/// validation panic that froze awl on a spell-heavy long file.
#[test]
fn grow_sizes_buffer_to_capacity_not_contents() {
    let Some((device, queue)) = headless_device() else {
        return; // no GPU adapter available — skip
    };
    let mut pipe = SelectionPipeline::new(
        &device,
        &selection_shader(&device),
        wgpu::TextureFormat::Rgba8UnormSrgb,
        [255, 255, 255, 255],
    );
    let rects =
        |n: usize| -> Vec<[f32; 4]> { (0..n).map(|i| [i as f32, 0.0, 10.0, 10.0]).collect() };
    pipe.prepare(&device, &queue, 800, 600, &rects(65));
    pipe.prepare(&device, &queue, 800, 600, &rects(100));
    assert_eq!(pipe.instance_count(), 100);
}

/// Shared headless (device, queue), mirroring the request-adapter dance every
/// GPU-backed test in this file needs. `None` (never a panic) when the host has
/// no wgpu adapter — the same tolerance every other headless render test grants.
fn headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok()?;
        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("awl selection-test device"),
                ..Default::default()
            })
            .await
            .ok()
    })
}

// --- item 131b: the spine primitive -----------------------------------------

#[test]
fn spine_segment_computes_center_half_and_normalized_axis() {
    let (center, half, axis) = spine_segment([0.0, 0.0], [30.0, 40.0], 8.0);
    assert_eq!(center, [15.0, 20.0], "center is the segment's midpoint");
    assert_eq!(
        half[0], 25.0,
        "half-length is half the 3-4-5 segment's length 50"
    );
    assert_eq!(half[1], 4.0, "half-thickness is half of thickness_px");
    assert!((axis[0] - 0.6).abs() < 1e-6, "axis.x == dx/len == 30/50");
    assert!((axis[1] - 0.8).abs() < 1e-6, "axis.y == dy/len == 40/50");
    let mag = (axis[0] * axis[0] + axis[1] * axis[1]).sqrt();
    assert!((mag - 1.0).abs() < 1e-6, "axis must be a unit vector");
}

#[test]
fn spine_segment_upright_axis_for_a_purely_horizontal_segment() {
    let (_, _, axis) = spine_segment([10.0, 5.0], [90.0, 5.0], 4.0);
    assert_eq!(axis, [1.0, 0.0]);
}

/// A zero-length segment has no direction to normalize; it must degenerate
/// to the inert upright axis rather than dividing by zero (NaN into the
/// shader is exactly the pathological input this function exists to avoid).
#[test]
fn spine_segment_degenerates_to_upright_on_a_zero_length_segment() {
    let (center, half, axis) = spine_segment([12.0, 34.0], [12.0, 34.0], 6.0);
    assert_eq!(center, [12.0, 34.0]);
    assert_eq!(half[0], 0.0);
    assert_eq!(half[1], 3.0);
    assert_eq!(axis, UPRIGHT_AXIS);
    assert!(axis[0].is_finite() && axis[1].is_finite());
}

#[test]
fn narrowed_spine_corner_px_caps_to_the_shorter_half_extent() {
    // A generous corner request against a long, thin segment caps to the
    // thickness, never the length.
    assert_eq!(narrowed_spine_corner_px(50.0, 200.0, 3.0), 3.0);
    // The reverse: a short, thick segment caps to the length.
    assert_eq!(narrowed_spine_corner_px(50.0, 2.0, 30.0), 2.0);
    // A modest request under both extents passes through unchanged.
    assert_eq!(narrowed_spine_corner_px(2.0, 200.0, 30.0), 2.0);
    // Never negative, even from a negative input.
    assert_eq!(narrowed_spine_corner_px(-5.0, 200.0, 30.0), 0.0);
}

const OFFSCREEN_FMT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn offscreen(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl selection-test offscreen"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: OFFSCREEN_FMT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn align_256(n: u32) -> u32 {
    (n + 255) & !255
}

/// Render `pipe`'s currently-`prepare`d instances alone (transparent clear,
/// no other pipeline in the pass) and read the result back as row-major
/// `[u8;4]` — the smallest possible harness that exercises `axis` end to end
/// through the real WGSL vertex stage, mirroring `render::tests::dither`'s
/// identical (and, per that module's own doc, deliberately duplicated)
/// readback dance.
fn render_alone(
    pipe: &SelectionPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) -> Vec<[u8; 4]> {
    let (texture, view) = offscreen(device, width, height);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl selection-test encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl selection-test pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pipe.draw(&mut pass);
    }
    queue.submit(Some(encoder.finish()));

    let unpadded_bpr = width * 4;
    let padded_bpr = align_256(unpadded_bpr);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("awl selection-test readback"),
        size: (padded_bpr * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut copy_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl selection-test copy encoder"),
    });
    copy_encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(copy_encoder.finish()));

    let (tx, rx) = std::sync::mpsc::channel();
    readback.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll failed");
    rx.recv()
        .expect("map_async channel closed")
        .expect("buffer map failed");

    let mut out = Vec::with_capacity((width * height) as usize);
    {
        let mapped = readback.slice(..).get_mapped_range();
        for y in 0..height {
            let row_start = (y * padded_bpr) as usize;
            for x in 0..width {
                let i = row_start + (x * 4) as usize;
                out.push([mapped[i], mapped[i + 1], mapped[i + 2], mapped[i + 3]]);
            }
        }
    }
    out
}

fn ink_at(pixels: &[[u8; 4]], width: u32, x: u32, y: u32) -> bool {
    pixels[(y * width + x) as usize][3] > 128
}

/// BYTE-IDENTITY: `prepare_rotated` with the inert upright axis draws EXACTLY
/// what `prepare` draws for the equivalent centered rect — the direct proof
/// that adding `axis` changed nothing for the ~46 existing pipeline
/// instances that only ever call `prepare`/`prepare_multicolor` (which
/// always upload `UPRIGHT_AXIS`).
#[test]
fn prepare_rotated_with_the_upright_axis_matches_prepare_byte_for_byte() {
    let Some((device, queue)) = headless_device() else {
        eprintln!("skipping prepare_rotated_with_the_upright_axis: no wgpu adapter");
        return;
    };
    let shader = selection_shader(&device);
    let (w, h) = (100u32, 100u32);

    let mut via_prepare =
        SelectionPipeline::new(&device, &shader, OFFSCREEN_FMT, [255, 255, 255, 255]);
    via_prepare.prepare(&device, &queue, w, h, &[[20.0, 46.0, 60.0, 8.0]]);
    let a = render_alone(&via_prepare, &device, &queue, w, h);

    let mut via_rotated =
        SelectionPipeline::new(&device, &shader, OFFSCREEN_FMT, [255, 255, 255, 255]);
    via_rotated.prepare_rotated(
        &device,
        &queue,
        w,
        h,
        &[([50.0, 50.0], [30.0, 4.0], UPRIGHT_AXIS)],
    );
    let b = render_alone(&via_rotated, &device, &queue, w, h);

    assert_eq!(a.len(), b.len());
    let differing = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    assert_eq!(
        differing,
        0,
        "prepare_rotated(axis=(1,0)) must be byte-identical to prepare() for the same \
         rect, {differing} of {} pixels differed",
        a.len()
    );
}

/// THE ROTATION IS REAL: a 45°-rotated bar and its upright twin disagree at
/// two probe points in exactly the way true rotation predicts — a point
/// along the OLD x-axis (inside the upright footprint) falls outside the
/// rotated one, and a point along the NEW (diagonal) axis (outside the
/// upright footprint) falls inside the rotated one. This is the load-bearing
/// proof that `axis` reaches the vertex stage and genuinely turns the quad,
/// not merely that the field compiles.
#[test]
fn prepare_rotated_axis_actually_turns_the_quad() {
    let Some((device, queue)) = headless_device() else {
        eprintln!("skipping prepare_rotated_axis_actually_turns_the_quad: no wgpu adapter");
        return;
    };
    let shader = selection_shader(&device);
    let (w, h) = (100u32, 100u32);
    let center = [50.0f32, 50.0];
    let half = [30.0f32, 4.0];

    let mut upright = SelectionPipeline::new(&device, &shader, OFFSCREEN_FMT, [255, 255, 255, 255]);
    upright.prepare_rotated(&device, &queue, w, h, &[(center, half, UPRIGHT_AXIS)]);
    let upright_px = render_alone(&upright, &device, &queue, w, h);

    let diag_axis = [
        std::f32::consts::FRAC_1_SQRT_2,
        std::f32::consts::FRAC_1_SQRT_2,
    ];
    let mut rotated = SelectionPipeline::new(&device, &shader, OFFSCREEN_FMT, [255, 255, 255, 255]);
    rotated.prepare_rotated(&device, &queue, w, h, &[(center, half, diag_axis)]);
    let rotated_px = render_alone(&rotated, &device, &queue, w, h);

    // (75, 50): 25px along the OLD x-axis alone — inside the upright bar's
    // half-length (30) and half-thickness (4); off the diagonal bar's own
    // (rotated) thickness band.
    assert!(
        ink_at(&upright_px, w, 75, 50),
        "sanity: the upright bar must cover its own along-axis point"
    );
    assert!(
        !ink_at(&rotated_px, w, 75, 50),
        "the 45°-rotated bar must NOT cover the point along the OLD (unrotated) axis \
         — if it does, `axis` never reached the vertex stage"
    );

    // (67, 67): 17px along BOTH x and y — 17*sqrt(2) ~= 24.0 along the
    // diagonal bar's own axis, comfortably under its half-length (30) with
    // margin to spare past the corner rounding/AA feather; far outside the
    // upright bar's y-extent (half-thickness 4).
    assert!(
        !ink_at(&upright_px, w, 67, 67),
        "sanity: the upright bar must not cover a point 17px off its own axis"
    );
    assert!(
        ink_at(&rotated_px, w, 67, 67),
        "the 45°-rotated bar must cover its own diagonal reach — if it does not, \
         `axis` never reached the vertex stage"
    );
}
