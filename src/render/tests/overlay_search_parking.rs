//! Overlay-owned render batches must not survive the transition to Find.

use super::super::*;
use super::{dither, headless_dqp, view};

#[derive(Clone, Copy)]
enum Layer {
    Placard,
    Asset,
    Search,
}

fn pixels(
    p: &TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layer: Layer,
) -> Vec<[u8; 4]> {
    let (texture, target) = dither::offscreen(device, 1600, 900);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("overlay owner witness"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(theme::base_100().to_wgpu_clear()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let renderer = match layer {
            Layer::Placard => {
                p.placard_stipple.draw(&mut pass);
                &p.placard_renderer
            }
            Layer::Asset => {
                p.asset_preview_panel.draw(&mut pass);
                p.asset_preview_image.draw(&mut pass);
                &p.asset_preview_text_renderer
            }
            Layer::Search => {
                p.panel_query_selection.draw(&mut pass);
                p.panel_caret.draw(&mut pass);
                &p.panel_renderer
            }
        };
        renderer.render(&p.atlas, &p.viewport, &mut pass).unwrap();
        if matches!(layer, Layer::Placard) {
            p.placard_material.draw(&mut pass);
        }
    }
    queue.submit(Some(encoder.finish()));
    dither::read_pixels(device, queue, &texture, 1600, 900)
}

fn transition(world: &str, layer: Layer, preview: Option<std::path::PathBuf>) {
    let entry = theme::active_index();
    let held = crate::hud::hud_held();
    crate::hud::set_held(false);
    theme::set_active_by_name(world).unwrap();
    let (device, queue, mut p) = headless_dqp(1600.0, 900.0).unwrap();
    let mut search = view("Synthetic writing stays unchanged.\n", 0, 5);
    search.search_active = true;
    search.search_query = "writing".into();
    search.search_query_caret = 3;
    p.set_view(&search);
    p.prepare(&device, &queue, 1600, 900).unwrap();
    let empty = pixels(&p, &device, &queue, layer);
    let search_before = pixels(&p, &device, &queue, Layer::Search);
    assert!(p.panel_caret.is_drawn(), "Find must retain its caret");

    let mut overlay = view(&search.text, 0, 5);
    overlay.overlay_active = true;
    overlay.overlay_title = "Assets".into();
    overlay.overlay_items = vec!["synthetic-orphan.png".into(), "another.png".into()];
    overlay.overlay_bindings = vec!["12 KB".into(), "4 KB".into()];
    overlay.overlay_asset_preview = preview;
    p.set_view(&overlay);
    p.prepare(&device, &queue, 1600, 900).unwrap();
    let shown = pixels(&p, &device, &queue, layer);
    let presence = shown.iter().zip(&empty).filter(|(a, b)| a != b).count();
    assert!(
        presence > 20,
        "the overlay subject must really draw: {presence} pixels"
    );

    p.set_view(&search);
    p.prepare(&device, &queue, 1600, 900).unwrap();
    assert!(
        pixels(&p, &device, &queue, layer) == empty,
        "Find must carry no pixels from the previous overlay-owned batch"
    );
    assert!(
        pixels(&p, &device, &queue, Layer::Search) == search_before,
        "parking overlay batches must preserve Find text and interior caret"
    );
    crate::hud::set_held(true);
    p.prepare(&device, &queue, 1600, 900).unwrap();
    assert!(
        p.full_frost(),
        "held HUD exercises the unconditional overlay draw path"
    );
    assert!(
        pixels(&p, &device, &queue, layer) == empty,
        "full frost must not resurrect the previous overlay batch"
    );
    crate::hud::set_held(false);

    p.set_view(&overlay);
    p.prepare(&device, &queue, 1600, 900).unwrap();
    assert!(
        pixels(&p, &device, &queue, layer) == shown,
        "reopening must prepare the overlay subject again"
    );
    p.set_view(&view(&search.text, 0, 5));
    p.prepare(&device, &queue, 1600, 900).unwrap();
    assert!(
        pixels(&p, &device, &queue, layer) == empty,
        "ordinary closing must still park the same overlay owner"
    );
    crate::hud::set_held(held);
    theme::set_active(entry);
}

#[test]
fn search_transition_parks_the_previous_placard() {
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping overlay/search transition: no wgpu adapter");
        return;
    }
    let _guard = crate::testlock::serial();
    transition("Firetail", Layer::Placard, None);
}

#[test]
fn search_transition_parks_the_missing_asset_statement_and_panel() {
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping overlay/search transition: no wgpu adapter");
        return;
    }
    let _guard = crate::testlock::serial();
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("samples/overlay-search-intentionally-missing.png");
    assert!(
        !fixture.exists(),
        "the missing-preview fixture must be absent"
    );
    transition("Firetail", Layer::Asset, Some(fixture.clone()));
}

#[test]
fn search_transition_parks_the_decoded_asset_thumbnail_and_panel() {
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping overlay/search transition: no wgpu adapter");
        return;
    }
    let _guard = crate::testlock::serial();
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/tiny.png");
    assert!(fixture.is_file(), "the synthetic image fixture must exist");
    transition("Firetail", Layer::Asset, Some(fixture));
}

#[test]
fn search_transition_visual_smoke_across_five_compositions() {
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping overlay/search visual smoke: no wgpu adapter");
        return;
    }
    let _guard = crate::testlock::serial();
    let entry = theme::active_index();
    let held = crate::hud::hud_held();
    crate::hud::set_held(false);
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("samples/overlay-search-intentionally-missing.png");
    assert!(
        !fixture.exists(),
        "the missing-preview fixture must be absent"
    );
    let output = std::env::var_os("AWL_OVERLAY_SEARCH_CAPTURE_DIR").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        std::fs::create_dir_all(dir).unwrap();
    }
    for world in ["Firetail", "Mangrove", "Magpie", "Paperbark", "Cassowary"] {
        theme::set_active_by_name(world).unwrap();
        let (device, queue, mut p) = headless_dqp(1600.0, 900.0).unwrap();
        let mut search = view("Synthetic writing stays unchanged.\n", 0, 5);
        search.search_active = true;
        search.search_query = "writing".into();
        search.search_query_caret = 3;
        p.set_view(&search);
        p.prepare(&device, &queue, 1600, 900).unwrap();
        let _ = super::pixeldiff::render_frame(&mut p, &device, &queue, 1600, 900);
        let empty = pixels(&p, &device, &queue, Layer::Asset);
        let mut overlay = view(&search.text, 0, 5);
        overlay.overlay_active = true;
        overlay.overlay_title = "Assets".into();
        overlay.overlay_items = vec!["synthetic-orphan.png".into(), "another.png".into()];
        overlay.overlay_asset_preview = Some(fixture.clone());
        p.set_view(&overlay);
        p.prepare(&device, &queue, 1600, 900).unwrap();
        let _ = super::pixeldiff::render_frame(&mut p, &device, &queue, 1600, 900);
        // Even a right-anchored world's room-less preview is a useful inert control.
        p.set_view(&search);
        crate::hud::set_held(true);
        p.prepare(&device, &queue, 1600, 900).unwrap();
        assert!(
            pixels(&p, &device, &queue, Layer::Asset) == empty,
            "{world}: Find must park its previous asset preview owner"
        );
        let frame = super::pixeldiff::render_frame(&mut p, &device, &queue, 1600, 900);
        if let Some(dir) = &output {
            let bytes: Vec<u8> = frame.into_iter().flatten().collect();
            image::save_buffer(
                dir.join(format!("{world}.png")),
                &bytes,
                1600,
                900,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
        crate::hud::set_held(false);
    }
    crate::hud::set_held(held);
    theme::set_active(entry);
}

#[test]
fn search_transition_parks_the_previous_stipple_placard() {
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping overlay/search transition: no wgpu adapter");
        return;
    }
    let _guard = crate::testlock::serial();
    transition("Mangrove", Layer::Placard, None);
}
