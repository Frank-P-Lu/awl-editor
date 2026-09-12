//! The menu bar is interface chrome: its drawn strip, shaped title bands,
//! dropdown card, and dropdown row grid follow display density but never the
//! document zoom behind them. This law sweeps every authored zoom step and
//! fingerprints the renderer's stored hit geometry, so the pixels and pointer
//! target cannot drift together while still passing.

use super::{headless_dqp, view};

#[derive(Debug, Clone, PartialEq)]
struct MenuGeometry {
    reserve: f32,
    bar_h: f32,
    titles: Vec<crate::menubar::TitleBox>,
    dropdown: Option<[f32; 4]>,
    rows: Vec<crate::menubar::DropRow>,
}

#[test]
fn menu_bar_and_open_dropdown_are_invariant_across_every_document_zoom_step() {
    let _g = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(concat!(
            "skipping menu_bar_and_open_dropdown_are_invariant_across_every_document_zoom_step: ",
            "no wgpu adapter"
        ));
        return;
    };

    crate::menubar::set_menu_bar_on(true);
    crate::menubar::set_open(Some(0));
    let zoom = crate::range::ZOOM;
    let mut expected = None;
    let mut document_line_heights = Vec::new();
    let mut cells = 0usize;
    for step in zoom.min_step()..=zoom.max_step() {
        let value = zoom.value_of_step(step);
        let mut v = view("menu bar zoom law\n", 0, 0);
        v.zoom = value;
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();

        document_line_heights.push(p.metrics.line_height);
        let got = MenuGeometry {
            reserve: p.menubar_reserve(),
            bar_h: p.menubar_bar_h,
            titles: p.menubar_boxes.clone(),
            dropdown: p.menu_drop_rect,
            rows: p.menu_drop_rows.clone(),
        };
        assert!(!got.titles.is_empty(), "zoom {value}: title bands enrolled");
        assert!(got.dropdown.is_some(), "zoom {value}: dropdown enrolled");
        assert!(!got.rows.is_empty(), "zoom {value}: dropdown rows enrolled");
        if let Some(ref expected) = expected {
            assert_eq!(
                &got, expected,
                "document zoom {value} moved menu-bar render or hit geometry"
            );
        } else {
            expected = Some(got);
        }
        cells += 1;
    }

    assert_eq!(
        cells,
        zoom.step_count() as usize,
        "every zoom step enrolled"
    );
    assert!(
        document_line_heights.first() != document_line_heights.last(),
        "the swept document metric must actually change"
    );
}
