//! Accent outcomes through real pointer resolution and the native glyph renderer.
use super::*;

struct Cell {
    world: &'static str,
    shape: &'static str,
    dpi: u32,
    width: u32,
    height: u32,
    rest: Vec<[u8; 4]>,
    hover: Vec<[u8; 4]>,
    lane: [u32; 4],
    one_bit: bool,
}

fn frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    p.prepare(device, queue, w, h).unwrap();
    let (texture, target) = offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    p.render(&mut encoder, &target).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, w, h)
}

fn visit_cells(worlds: &[&str], dpis: &[u32], mut visit: impl FnMut(Cell)) -> bool {
    let _pin = theme::WorldPin::snapshot();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping close-hover accent pixels: no wgpu adapter");
        return false;
    }
    crate::page::set_page_on(true);
    for (index, world) in theme::THEMES
        .iter()
        .enumerate()
        .filter(|(_, t)| worlds.contains(&t.name))
    {
        theme::set_active(index);
        for &dpi in dpis {
            let (w, h) = (W * dpi, H * dpi);
            for shape in ["lone", "selected-stack", "unselected-stack"] {
                let (device, queue, mut p) = headless_dqp(w as f32, h as f32).unwrap();
                p.set_dpi(dpi as f32);
                let mut v = stack_view(1);
                // Keep the close glyph on canvas even in wide-pitch worlds.
                // Long-name clipping has separate geometry laws; this cell
                // compares the same visible filename alone and in a stack.
                v.gutter_files[1].parent.clear();
                let row = if shape == "selected-stack" { 1 } else { 0 };
                if shape == "lone" {
                    v.gutter_files.clear();
                }
                p.set_view(&v);
                p.clear_gutter_stack_hover();
                let rest = frame(&device, &queue, &mut p, w, h);
                let bands = row_bands(&p.gutter_frost_seeds(h));
                assert_eq!(bands.len(), if shape == "lone" { 2 } else { 4 });
                let band = bands[1 + row];
                let y = band[1] + band[3] * 0.5;
                let right = find_row_right_edge(&p, w as f32 - 1.0, y, h);
                let (lo, hi) = find_close_zone(&p, right, y, h).expect("close zone is drawn");
                let x = (lo + hi) * 0.5;
                let hit = p.gutter_stack_hit(x, y, h).unwrap();
                assert!(hit.is_close());
                assert_eq!(hit.row, row);
                let plate = p.gutter_stack_plate_rect(&device, &queue, w, h);
                assert!(plate.is_some(), "the selected plate must remain present");
                assert!(p.resolve_gutter_stack_hover(x, y, h));
                let hover = frame(&device, &queue, &mut p, w, h);
                assert_eq!(plate, p.gutter_stack_plate_rect(&device, &queue, w, h));
                assert!(
                    !p.resolve_gutter_stack_hover(x, y, h),
                    "repeated pointer is inert"
                );
                assert!(p.clear_gutter_stack_hover());
                assert_eq!(
                    rest,
                    frame(&device, &queue, &mut p, w, h),
                    "leaving restores every pixel"
                );
                visit(Cell {
                    world: world.name,
                    shape,
                    dpi,
                    width: w,
                    height: h,
                    rest,
                    hover,
                    lane: [
                        (lo - 2.0).max(0.0) as u32,
                        band[1].max(0.0) as u32,
                        (hi + 2.0).min(w as f32) as u32,
                        (band[1] + band[3]).min(h as f32) as u32,
                    ],
                    one_bit: world.is_one_bit(),
                });
            }
        }
    }
    true
}

fn measure(cell: &Cell) -> (usize, f32, f64, f32) {
    let [x0, y0, x1, y1] = cell.lane;
    let expected = theme::accent_ink(
        if cell.shape == "unselected-stack" {
            theme::base_100()
        } else {
            theme::surface_selected()
        },
        Some(if cell.shape == "unselected-stack" {
            theme::faint()
        } else {
            theme::selected_row_secondary_ink(theme::surface_selected())
        }),
    )
    .rgba_bytes();
    let mut changed = 0;
    let mut max_delta = 0.0f32;
    let mut nearest_accent = f32::MAX;
    let mut max_perceptual = 0.0f64;
    for y in 0..cell.height {
        for x in 0..cell.width {
            let i = (y * cell.width + x) as usize;
            let delta = dist(cell.rest[i], cell.hover[i]);
            if x >= x0 && x < x1 && y >= y0 && y < y1 {
                nearest_accent = nearest_accent.min(dist(cell.hover[i], expected));
                max_delta = max_delta.max(delta);
                max_perceptual = max_perceptual.max(super::super::pixeldiff::delta_e(
                    cell.rest[i],
                    cell.hover[i],
                ));
                changed += usize::from(delta > 4.0);
            } else {
                assert_eq!(
                    cell.rest[i], cell.hover[i],
                    "{} {} {}x: hover changed a pixel outside the close lane at {x},{y}",
                    cell.world, cell.shape, cell.dpi
                );
            }
        }
    }
    (changed, max_delta, max_perceptual, nearest_accent)
}

#[test]
fn close_hover_accent_reaches_lone_selected_and_unselected_pixels_at_both_dpis() {
    let _g = crate::testlock::serial();
    let mut count = 0;
    let worlds = theme::THEMES
        .iter()
        .map(|world| world.name)
        .collect::<Vec<_>>();
    let ran = visit_cells(&worlds, &[1, 2], |cell| {
        let (changed, delta, perceptual, nearest) = measure(&cell);
        let label = format!("{} {} {}x", cell.world, cell.shape, cell.dpi);
        eprintln!(
            "{label}: changed={changed} max-rgb={delta:.1} \
             max-deltaE={perceptual:.2} nearest-accent={nearest:.1}"
        );
        if cell.one_bit {
            assert_eq!(changed, 0, "{label}: one-bit no-op");
        } else {
            assert!(
                changed > 0 && perceptual >= 5.0,
                "{label}: hover response must be visible"
            );
            assert!(
                nearest < 36.0,
                "{label}: no rendered pixel carries the resolved accent"
            );
        }
        count += 1;
    });
    if ran {
        assert_eq!(count, theme::THEMES.len() * 3 * 2);
    }
}

#[test]
#[ignore = "writes real native frames for close-hover visual review"]
fn capture_close_hover_accent_gallery() {
    let _g = crate::testlock::serial();
    let root = std::path::Path::new("gallery/close-hover");
    std::fs::create_dir_all(root).unwrap();
    assert!(
        visit_cells(
            &["Gumtree", "Potoroo", "Mulga", "Cassowary", "Wagtail"],
            &[1],
            |cell| {
                let (changed, delta, perceptual, nearest) = measure(&cell);
                for (state, px) in [("rest", &cell.rest), ("hover", &cell.hover)] {
                    let bytes = px.iter().flat_map(|p| p.iter().copied()).collect();
                    image::RgbaImage::from_raw(cell.width, cell.height, bytes)
                        .unwrap()
                        .save(root.join(format!("{}-{}-{state}.png", cell.world, cell.shape)))
                        .unwrap();
                }
                let data = serde_json::json!({"world":cell.world,"shape":cell.shape,"dpi":cell.dpi,
            "canvas":[cell.width,cell.height],"lane":cell.lane,"changed_pixels":changed,
            "max_rgb_delta":delta,"max_delta_e":perceptual,"nearest_accent":nearest});
                std::fs::write(
                    root.join(format!("{}-{}.json", cell.world, cell.shape)),
                    serde_json::to_vec_pretty(&data).unwrap(),
                )
                .unwrap();
            }
        ),
        "gallery requires a GPU adapter"
    );
}
