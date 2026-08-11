use super::*;

fn headless_dqp() -> Option<(wgpu::Device, wgpu::Queue, TextPipeline)> {
    let (device, queue) = crate::test_gpu::shared_device_queue()?;
    let cache = Cache::new(&device);
    let mut p = TextPipeline::new(&device, &queue, &cache, FORMAT);
    p.set_size(WIDTH as f32, HEIGHT as f32);
    p.set_dpi(DPI);
    Some((device, queue, p))
}

/// A cheap, no-GPU-needed sanity check: both stages this rescue round named
/// (the new wash stage, and the pre-existing table-grid stage that turned
/// out to have NO name at all — see the module doc above) are present.
/// Holds even on a machine with no wgpu adapter, where the GPU-backed test
/// below skips.
#[test]
fn stage_names_include_wash_and_table_grid() {
    assert!(
        STAGE_NAMES.contains(&"wash layer (cull + upload)"),
        "the wash-layer stage must be named in STAGE_NAMES: {STAGE_NAMES:?}"
    );
    assert!(
        STAGE_NAMES.contains(&"table grid (grid geometry)"),
        "the table-grid stage must be named in STAGE_NAMES: {STAGE_NAMES:?}"
    );
}

#[test]
fn wash_layer_and_table_grid_stages_stay_in_lockstep() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp() else {
        eprintln!("skipping wash_layer_and_table_grid_stages_stay_in_lockstep: no wgpu adapter");
        return;
    };
    // Pin a DARK world explicitly — the STRING wash bucket only uploads on
    // dark worlds (`role_style_for`'s documented rule; light worlds carry
    // string identity in the fg tint alone), and the process-global active
    // theme's own default (`theme::DEFAULT_THEME` = Saltpan, a LIGHT world)
    // would otherwise make this test's outcome depend on whichever OTHER
    // test happened to run first in the process and leave a dark world
    // active — exactly the kind of order-dependent flake this codebase's
    // `testlock::serial()` discipline exists to rule out.
    crate::theme::set_active_by_name("Tawny").unwrap();
    let text = "prose before\n```sh\n# a comment\nexport PATH=\"/usr/bin\"\n```\nprose after\n";
    let view = ViewState {
        text: text.to_string(),
        is_markdown: true,
        ..ViewState::base()
    };
    p.set_view(&view);

    let (comments, strings, _highlights) = p.wash_rects();
    assert!(
        !comments.is_empty(),
        "the fenced comment must produce wash geometry: {comments:?}"
    );
    assert!(
        !strings.is_empty(),
        "the fenced string literal must produce wash geometry: {strings:?}"
    );

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl framebench test target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut marks = Marks::new(STAGE_NAMES.len());
    marks.begin(true);
    run_one_frame(&mut p, &device, &queue, &target_view, &mut marks, 0, None)
        .expect("one bench frame must run cleanly");
    assert_eq!(
        marks.i,
        STAGE_NAMES.len(),
        "stage marks must stay in lockstep with STAGE_NAMES"
    );

    assert!(
        p.wash_comment_pipeline.instance_count() > 0,
        "prepare_wash_layer must upload the comment wash instances it built"
    );
    assert!(
        p.wash_string_pipeline.instance_count() > 0,
        "prepare_wash_layer must upload the string wash instances it built"
    );
}
