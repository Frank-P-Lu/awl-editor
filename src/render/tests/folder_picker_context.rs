//! Current location, keyboard baseline, and row geometry through the shared renderer.
use super::super::*;
use super::{headless_dqp, pixeldiff, view};

#[test]
fn folder_picker_context_paints_and_preserves_navigation_across_worlds_and_dpis() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping folder context outcome law: no GPU adapter");
        return;
    };
    for world in theme::THEMES {
        theme::set_active_by_name(world.name).unwrap();
        for dpi in [1.0, 2.0] {
            for logical_w in [360.0, 1200.0] {
                let (w, h) = ((logical_w * dpi) as u32, (600.0 * dpi) as u32);
                p.set_dpi(dpi);
                p.set_size(w as f32, h as f32);
                let mut v = view("Synthetic writing fixture\n", 0, 0);
                v.overlay_active = true;
                v.overlay_title = "browse for folder".into();
                v.overlay_folder_location =
                    Some("/workspace/Writing/long-parent-directory/manuscripts".into());
                v.overlay_items = vec!["chapter/".into(), "research/".into()];
                v.overlay_hint = "↵ switch here   → open   ← up".into();
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();
                let geom = p.overlay_geometry(w);
                let plan = p.overlay_row_plan(&geom);
                assert!(
                    p.panel_buffer.lines[0]
                        .text()
                        .starts_with("browse for folder")
                );
                assert_eq!(geom.header_rows, 2, "{} {dpi}x", world.name);
                let location = p
                    .panel_buffer
                    .layout_runs()
                    .find(|run| run.line_i == 1)
                    .unwrap();
                assert!(location.line_w <= geom.text_w + 0.5);
                assert!(!location.glyphs.is_empty());
                let path_box = p.overlay_line_glyph_box(1).unwrap();
                assert!(!p.over_overlay_query(path_box[0] + 2.0, path_box[1] + path_box[3] * 0.5));
                let row = plan.rows().first().unwrap();
                let bounds = p.overlay_row_geometry().unwrap().rows[0];
                assert_eq!(
                    p.overlay_row_at(bounds.x + bounds.w * 0.5, bounds.y + bounds.h * 0.5),
                    Some(0),
                    "{} {dpi}x {logical_w}",
                    world.name
                );
                assert!(row.top >= path_box[1] + path_box[3] - 0.5);
                let hint_line = p.overlay_hint_line().unwrap();
                let hint = p
                    .panel_buffer
                    .layout_runs()
                    .find(|run| run.line_i == hint_line)
                    .unwrap();
                let face = hint.glyphs[0].font_id;
                assert!(
                    hint.glyphs.iter().all(|glyph| glyph.font_id == face),
                    "{}: key symbols and captions share a face",
                    world.name
                );
                let named = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                save_review_frame(world.name, dpi, logical_w, (w, h), &named);
                v.overlay_folder_location = Some(" ".into());
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();
                let blank = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                let diff = pixeldiff::diff_region(
                    &named,
                    &blank,
                    w as i64,
                    h as i64,
                    pixeldiff::Region::new(path_box[0], path_box[1], geom.text_w, path_box[3]),
                );
                assert!(
                    diff.differing > 10,
                    "{} {dpi}x: location paints",
                    world.name
                );
            }
        }
    }
}

/// Optional private review output from the same real frames the outcome law grades.
fn save_review_frame(world: &str, dpi: f32, width: f32, size: (u32, u32), pixels: &[[u8; 4]]) {
    let Some(directory) = std::env::var_os("AWL_FOLDER_REVIEW_DIR") else {
        return;
    };
    if !matches!(world, "Cassowary" | "Mopoke" | "Gumtree") || dpi != 1.0 {
        return;
    }
    let path = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&path).unwrap();
    let bytes: Vec<u8> = pixels
        .iter()
        .flat_map(|pixel| pixel.iter().copied())
        .collect();
    image::save_buffer(
        path.join(format!(
            "folder-{}-{}.png",
            world.to_lowercase(),
            width as u32
        )),
        &bytes,
        size.0,
        size.1,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
