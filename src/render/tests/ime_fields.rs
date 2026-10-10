//! Candidate rectangles are the caret quads painted on the actual text surface.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::textbox::TextField;

fn field_view(field: TextField) -> ViewState {
    let mut v = view("background document\n", 0, 3);
    v.field_input = Some(FieldInput {
        field,
        text: "にほん".into(),
        caret: 2,
        selection: None,
        preedit: Some((0, 3)),
        row: None,
    });
    match field {
        TextField::FindQuery | TextField::ReplaceText => {
            v.search_active = true;
            v.search_query = "にほん".into();
            v.search_query_caret = 2;
            v.search_replace_active = field == TextField::ReplaceText;
            v.search_editing_replacement = field == TextField::ReplaceText;
            v.search_replacement = "にほん".into();
            v.search_replacement_caret = 2;
        }
        TextField::PickerQuery
        | TextField::Rename
        | TextField::InsertLink
        | TextField::KeepVersion => {
            v.overlay_active = true;
            v.overlay_title = "Field".into();
            v.overlay_query = "にほん".into();
            v.overlay_query_caret = 2;
            v.overlay_items = vec!["日本.md".into()];
            if field == TextField::PickerQuery {
                v.overlay_files_surface = true;
                v.overlay_files_location = "Writing/notes".into();
                v.overlay_title = "Writing/notes  Up  Change folder  Search".into();
                v.overlay_lens = vec![("Files".into(), true), ("Recent".into(), false)];
                v.overlay_hint = "New document — Writing/notes".into();
                v.overlay_sections = vec![String::new()];
            }
        }
        TextField::SettingsValue => {
            v.overlay_active = true;
            v.overlay_title = "Settings".into();
            v.overlay_query_focused = false;
            v.overlay_items = vec!["Zoom".into()];
            v.overlay_bindings = vec!["125".into()];
            v.field_input.as_mut().unwrap().row = Some(0);
            v.field_input.as_mut().unwrap().text = "125".into();
        }
    }
    v
}

#[test]
fn ime_candidates_follow_the_painted_field_caret_across_fields_and_density() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping IME field geometry: no wgpu adapter");
        return;
    }
    for dpi in [1.0, 2.0] {
        for logical_width in [640.0, 1200.0] {
            let (w, h) = ((logical_width * dpi) as u32, (800.0 * dpi) as u32);
            for field in TextField::ALL {
                let (device, queue, mut p) = headless_dqp(w as f32, h as f32).unwrap();
                p.set_dpi(dpi);
                let v = field_view(field);
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();
                let rect = p
                    .focused_field_caret_rect()
                    .expect("every focused field paints its caret");
                let [x, y, width, height] = rect;
                assert!(
                    width > 0.0
                        && height > 0.0
                        && x >= 0.0
                        && y >= 0.0
                        && x + width <= w as f32
                        && y + height <= h as f32,
                    "{field:?} {dpi} {logical_width}: {rect:?}"
                );
                let doc = p.caret_pixel_rect();
                assert_ne!(
                    (x, y),
                    (doc.0, doc.1),
                    "candidate fell back to background document"
                );
                let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                p.panel_caret.prepare_empty();
                let bare = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                let difference = pixeldiff::diff_region(
                    &pixels,
                    &bare,
                    w as i64,
                    h as i64,
                    pixeldiff::Region::new(x, y, width + 1.0, height + 1.0),
                );
                assert!(
                    difference.differing > 0,
                    "{field:?}: reported candidate has no painted caret"
                );
            }
        }
    }
}

#[test]
fn files_ime_and_controls_survive_the_native_minimum_window_height() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping minimum-height Files IME law: no wgpu adapter");
        return;
    }
    // The native lifecycle enforces thirty default columns and eight lines.
    let min_w = 30.0 * CHAR_WIDTH + 2.0 * TEXT_LEFT.0;
    let min_h = 8.0 * LINE_HEIGHT + 2.0 * TEXT_TOP.0;
    for dpi in [1.0, 2.0] {
        for logical_width in [min_w, 1200.0] {
            let (w, h) = ((logical_width * dpi) as u32, (min_h * dpi) as u32);
            let (device, queue, mut p) = headless_dqp(w as f32, h as f32).unwrap();
            p.set_dpi(dpi);
            p.set_view(&field_view(TextField::PickerQuery));
            p.prepare(&device, &queue, w, h).unwrap();
            let rect = p.focused_field_caret_rect().expect("Files query caret");
            let [x, y, cw, ch] = rect;
            assert!(x >= 0.0 && y >= 0.0 && x + cw <= w as f32 && y + ch <= h as f32);
            let regions = p.files_surface_action_regions_probe();
            for region in regions {
                let (action, [x, y, cw, ch]) = region.expect("visible Files control");
                assert!(
                    cw > 0.0
                        && ch > 0.0
                        && x >= 0.0
                        && y >= 0.0
                        && x + cw <= w as f32
                        && y + ch <= h as f32,
                    "{action:?} at dpi={dpi}, width={logical_width} leaves canvas"
                );
                assert_eq!(
                    p.files_surface_action_at(x + cw * 0.5, y + ch * 0.5),
                    Some(action)
                );
            }
            let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
            p.panel_caret.prepare_empty();
            let bare = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
            let difference = pixeldiff::diff_region(
                &pixels,
                &bare,
                w as i64,
                h as i64,
                pixeldiff::Region::new(rect[0], rect[1], rect[2] + 1.0, rect[3] + 1.0),
            );
            assert!(
                difference.differing > 0,
                "actual Files query caret must paint"
            );
        }
    }
}

#[test]
fn document_preedit_cursor_moves_without_reshape_or_underline_drift() {
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping document IME geometry: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    for dpi in [1.0, 2.0] {
        for width in [640.0, 1200.0] {
            let (_, _, mut p) = headless_dqp(width * dpi, 800.0 * dpi).unwrap();
            p.set_dpi(dpi);
            for preedit in ["abc", "にほん", "a\u{301}b"] {
                let mut v = view("prefix suffix", 0, 7);
                v.preedit = preedit.into();
                p.set_view(&v);
                let shaped = p.shaped_key.clone();
                let reshapes = p.reshape_count;
                let underline = p.preedit_rects();
                assert!(!underline.is_empty());
                assert!(underline.iter().any(|r| r[2] > 1.0));
                let n = preedit.chars().count();
                for offset in [0, 1, n, n + 20] {
                    v.preedit_cursor = Some(offset);
                    p.set_view(&v);
                    assert_eq!(p.cursor_col, 7 + offset.min(n));
                    assert_eq!(p.shaped_key, shaped);
                    assert_eq!(p.reshape_count, reshapes);
                    assert_eq!(p.preedit_rects(), underline, "{preedit} at {offset}");
                }
                v.preedit_cursor = None;
                p.set_view(&v);
                assert_eq!(p.cursor_col, 7 + n);
                v.preedit.clear();
                p.set_view(&v);
                assert_eq!(p.cursor_col, 7);
                assert!(p.preedit_rects().is_empty());
            }
        }
    }
}
