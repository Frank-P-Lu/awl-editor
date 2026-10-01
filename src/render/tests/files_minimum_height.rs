//! Nonempty Files keeps a painted, clickable choice at the native minimum.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

fn files_view() -> ViewState {
    let mut v = view("document behind Files\n", 0, 0);
    v.overlay_active = true;
    v.overlay_files_surface = true;
    v.overlay_files_location = "Writing/notes".into();
    v.overlay_title = "Writing/notes  Up  Change folder  Search".into();
    v.overlay_lens = vec![("Files".into(), true), ("Recent".into(), false)];
    v.overlay_items = vec!["draft.md".into()];
    v.overlay_sections = vec![String::new()];
    v.overlay_hint = "New document — Writing/notes".into();
    v
}

#[test]
fn files_minimum_window_keeps_candidate_ink_hits_and_controls_across_worlds_and_dpi() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(928.0, 576.0) else {
        eprintln!("skipping minimum Files outcome law: no wgpu adapter");
        return;
    };
    let mut cells = 0;
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    for menu_bar in [false, true] {
        crate::menubar::set_menu_bar_on(menu_bar);
        for world in theme::THEMES {
            theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            for dpi in [1.0, 2.0] {
                p.set_dpi(dpi);
                let w = ((30.0 * CHAR_WIDTH + 2.0 * TEXT_LEFT.0) * dpi) as u32;
                let h = ((8.0 * LINE_HEIGHT + 2.0 * TEXT_TOP.0) * dpi) as u32;
                p.set_size(w as f32, h as f32);
                let v = files_view();
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();
                let geom = p.overlay_geometry(w);
                let plan = p.overlay_row_plan(&geom);
                let row = plan
                    .rows()
                    .iter()
                    .find(|row| row.item == Some(0))
                    .unwrap_or_else(|| {
                        panic!(
                            "{} dpi={dpi}: nonempty Files lost its candidate",
                            world.name
                        )
                    });
                let x = geom.row_text_left() + geom.row_text_w() * 0.5;
                let y = row.top + row.height * 0.5;
                assert_eq!(p.overlay_row_at(x, y), Some(0), "{} dpi={dpi}", world.name);
                assert!(row.top >= geom.card_y && row.bottom() <= geom.card_y + geom.card_h);
                assert!(geom.card_y >= 0.0 && geom.card_y + geom.card_h <= h as f32 + 0.01);
                let controls = p.files_surface_action_regions_probe();
                for region in controls {
                    let (action, [x, y, cw, ch]) = region.expect("Files action survives");
                    assert!(y >= 0.0 && y + ch <= h as f32 && cw > 0.0 && ch > 0.0);
                    assert_eq!(
                        p.files_surface_action_at(x + cw * 0.5, y + ch * 0.5),
                        Some(action)
                    );
                    assert!(p.overlay_row_at(x + cw * 0.5, y + ch * 0.5).is_none());
                }
                let rect = p
                    .files_surface_line_bounds_probe("draft.md")
                    .expect("candidate shaped ink");
                let named = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                let mut blank = v;
                blank.overlay_items[0] = " ".into();
                p.set_view(&blank);
                p.prepare(&device, &queue, w, h).unwrap();
                let bare = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                let difference = pixeldiff::diff_region(
                    &named,
                    &bare,
                    w as i64,
                    h as i64,
                    pixeldiff::Region::new(rect[0], rect[1], rect[2], rect[3]),
                );
                assert!(
                    difference.differing > 10,
                    "{} dpi={dpi}: draft.md must paint",
                    world.name
                );
                let roomy_h = (576.0 * dpi) as u32;
                p.set_size(w as f32, roomy_h as f32);
                p.set_view(&files_view());
                p.prepare(&device, &queue, w, roomy_h).unwrap();
                assert_eq!(
                    p.overlay_geometry(w).header_rows,
                    4,
                    "{} roomy header",
                    world.name
                );
                cells += 1;
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    assert_eq!(cells, theme::THEMES.len() * 4);
}
