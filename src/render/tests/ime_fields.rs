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
            v.overlay_title = "Files".into();
            v.overlay_query = "にほん".into();
            v.overlay_query_caret = 2;
            v.overlay_items = vec!["日本.md".into()];
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
