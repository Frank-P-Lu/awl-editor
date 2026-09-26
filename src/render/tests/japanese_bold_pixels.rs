//! Real-pixel proof that each bundled Japanese family visibly gains weight,
//! including mixed Latin, at both ends of the zoom/DPI matrix.

use super::super::*;
use super::{headless_dqp, pixeldiff, view_md};

#[test]
fn japanese_bold_is_visibly_heavier_across_all_families_zoom_and_dpi() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Japanese-bold pixel law: no wgpu adapter");
        return;
    };
    let entered = theme::active().name;
    let mut graded = 0;

    for world in ["Saltpan", "Currawong", "Bombora", "Galah", "Mopoke"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for (dpi, zoom) in [(1.0f32, 0.8f32), (2.0, 1.6)] {
            let (w, h) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
            p.set_size(w as f32, h as f32);
            p.set_dpi(dpi);

            let mut regular_view = view_md("\n日本語 ABC\n", 0, 0);
            regular_view.zoom = zoom;
            p.set_view(&regular_view);
            p.prepare(&device, &queue, w, h).unwrap();
            let row_top = p.line_ornament_top(1);
            let japanese_right = p.line_glyph_xs(1)[3];
            let region = pixeldiff::Region::new(
                p.text_left() - 3.0 * dpi,
                row_top - 2.0 * dpi,
                japanese_right + 6.0 * dpi,
                p.metrics.line_height + 4.0 * dpi,
            );
            let regular = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

            let mut bold_view = view_md("\n**日本語 ABC**\n", 0, 0);
            bold_view.zoom = zoom;
            p.set_view(&bold_view);
            p.prepare(&device, &queue, w, h).unwrap();
            let bold = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

            pixeldiff::assert_perceptibly_different(
                &regular,
                &bold,
                w as i64,
                h as i64,
                region,
                pixeldiff::DistinguishFloor {
                    min_fraction: 0.002,
                    min_max_delta: 12,
                },
                &format!("{world}: plain vs real Japanese bold at dpi={dpi}, zoom={zoom}"),
            );
            graded += 1;
        }
    }

    p.set_dpi(1.0);
    theme::set_active_by_name(entered).unwrap();
    p.sync_theme();
    assert_eq!(graded, 10, "five families at both zoom/DPI endpoints");
}
