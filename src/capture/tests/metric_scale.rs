//! Item 96: the sidecar's font metrics and the PNG must inhabit one pixel scale.
//!
//! The regression reported the unscaled `render::LINE_HEIGHT` while the renderer,
//! `text_origin`, and `page.column` used zoomed pixels. A struct-only assertion
//! could repeat that mistake on both sides, so this law measures repeated glyph
//! rows directly from the captured PNG and compares their pitch with the JSON.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::testscratch::ScratchDir;

fn json(png: &std::path::Path) -> serde_json::Value {
    let bytes = std::fs::read_to_string(png.with_extension("json")).expect("sidecar exists");
    serde_json::from_str(&bytes).expect("sidecar parses")
}

/// Find the top pixel row of each repeated `MMMMMMMM` ink band. Every y is
/// compared with a blank pixel farther right on that SAME y, so the measurement
/// remains valid over a non-flat page ground and never assumes a theme color.
fn measured_ink_band_tops(png: &std::path::Path, text_left: u32) -> Vec<u32> {
    let image = image::open(png).expect("capture PNG decodes").to_rgba8();
    let blank_x = (text_left + 280).min(image.width() - 1);
    let x1 = (text_left + 240).min(blank_x);
    let mut active_y = Vec::new();

    for y in 0..image.height() {
        let bg = image.get_pixel(blank_x, y).0;
        let differing = (text_left..x1)
            .filter(|&x| {
                let px = image.get_pixel(x, y).0;
                (0..3).any(|c| px[c].abs_diff(bg[c]) >= 18)
            })
            .count();
        if differing >= 4 {
            active_y.push(y);
        }
    }

    let mut tops = Vec::new();
    let mut last_y = None;
    for y in active_y {
        if last_y.is_none_or(|last| y > last + 1) {
            tops.push(y);
        }
        last_y = Some(y);
    }
    tops
}

#[test]
fn sidecar_line_height_matches_measured_png_row_pitch_at_multiple_zooms() {
    if !adapter_available() {
        eprintln!(
            "skipping sidecar_line_height_matches_measured_png_row_pitch_at_multiple_zooms: \
             no wgpu adapter"
        );
        return;
    }
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::world("Tawny").expect("Tawny exists");
    let old_page = crate::page::page_on();
    let old_measure = crate::page::measure();
    crate::page::set_page_on(true);
    crate::page::set_measure(40);

    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-sidecar-metric-scale-{}", std::process::id())),
    );
    let mut buf =
        Buffer::from_str("MMMMMMMM\nMMMMMMMM\nMMMMMMMM\nMMMMMMMM\nMMMMMMMM\nMMMMMMMM\nMMMMMMMM\n");
    // Keep the amber caret off the seven measured text rows.
    buf.buffer_end();

    let mut measured = Vec::new();
    for zoom in [0.8_f32, 1.5_f32] {
        let png = dir.join(format!("zoom-{zoom}.png"));
        capture_with(
            &png,
            &buf,
            &CaptureOpts {
                zoom: Some(zoom),
                ..CaptureOpts::default()
            },
        )
        .expect("zoomed capture");
        let side = json(&png);
        let reported_zoom = side["font"]["zoom"].as_f64().expect("font.zoom") as f32;
        let reported_size = side["font"]["size"].as_f64().expect("font.size") as f32;
        let reported_lh = side["font"]["line_height"]
            .as_f64()
            .expect("font.line_height") as f32;
        let text_left = side["text_origin"]["left"]
            .as_f64()
            .expect("text_origin.left")
            .round() as u32;

        assert!(
            (reported_zoom - zoom).abs() < 1e-4,
            "zoom {zoom}: reported {reported_zoom}"
        );
        assert!(
            (reported_size - crate::render::FONT_SIZE * zoom).abs() < 1e-3,
            "zoom {zoom}: effective font size {reported_size}"
        );

        let tops = measured_ink_band_tops(&png, text_left);
        assert!(
            tops.len() >= 7,
            "zoom {zoom}: expected seven independently measured glyph bands, got {tops:?}"
        );
        // The settled caret on the trailing blank line may contribute one extra
        // small band before/after the repeated text depending on caret look. The
        // seven repeated M rows are the final seven full ink bands.
        let repeated = &tops[tops.len() - 7..];
        let pitches: Vec<f32> = repeated
            .windows(2)
            .map(|pair| (pair[1] - pair[0]) as f32)
            .collect();
        for pitch in &pitches {
            assert!(
                (*pitch - reported_lh).abs() <= 1.0,
                "zoom {zoom}: measured PNG row pitch {pitch}px must equal sidecar \
                 font.line_height {reported_lh}px (tops {tops:?})"
            );
        }
        measured.push((zoom, reported_lh, pitches[0]));
    }

    // Non-vacuity: this law sampled genuinely different scales. The old sidecar
    // reported 32 at BOTH zooms while these PNG pitches are ~26 and 48.
    assert_ne!(
        measured[0].1, measured[1].1,
        "reported line heights must scale"
    );
    assert_ne!(
        measured[0].2, measured[1].2,
        "measured PNG pitches must scale"
    );

    crate::page::set_measure(old_measure);
    crate::page::set_page_on(old_page);
}
