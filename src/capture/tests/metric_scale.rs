//! The sidecar's font metrics and the PNG must inhabit one pixel scale.
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
///
/// THE SEARCH IS BOUNDED BY THE DOCUMENT'S OWN ORIGIN ON BOTH AXES — `text_left`
/// and `text_top` come from the sidecar's `text_origin`, the product's own answer
/// to where the document begins. The x bound was always here; the y bound is what
/// stops CHROME ABOVE THE DOCUMENT from entering the census. The rendered menu
/// bar draws its titles into this same x window, so on every platform whose
/// menu-bar default is on (`menubar::MENU_BAR_DEFAULT_OTHER` — everywhere but
/// macOS) the detector reported a FRONT band that is not a glyph row at all, and
/// the caller's front slice then measured the bar-to-first-row gap as the
/// document's pitch. A band census that starts at y=0 is a claim that nothing
/// but the document ever inks, and the frame is not the caller's to promise.
fn measured_ink_band_tops(png: &std::path::Path, text_left: u32, text_top: u32) -> Vec<u32> {
    let image = image::open(png).expect("capture PNG decodes").to_rgba8();
    let blank_x = (text_left + 280).min(image.width() - 1);
    let x1 = (text_left + 240).min(blank_x);
    let mut active_y = Vec::new();

    for y in text_top.min(image.height())..image.height() {
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
    // SPELLCHECK OFF for the duration: `MMMMMMMM` is not a word, so every
    // measured row would otherwise carry a squiggle, and the band detector
    // above cannot tell that decoration from the glyphs it is meant to measure.
    // This law passed for a long time only because the shipped squiggle's ink
    // happened to FUSE with the glyph ink into a single band; a size change that
    // thinned the wave's topmost rows unfused them, and the law began reporting
    // the glyph-to-squiggle gap as the document's row pitch. The metric under
    // test has nothing to do with spelling — so the frame should not contain any.
    let old_spellcheck = crate::spell::spellcheck_on();
    crate::spell::set_spellcheck_on(false);
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

    // BOTH MENU-BAR ARMS, because the bar's reserve moves `text_origin.top` and
    // its titles ink into the measured x window. The bar is the one
    // platform-forked sticky default in the tree (`menubar::platform_default`),
    // so a fixture that leaves it ambient asks a DIFFERENT question on macOS than
    // on Linux — this law was structurally incapable of seeing its own front-band
    // defect on the host that wrote it, and CI found it instead. The ambient value
    // is captured rather than `cfg!`-derived: a `cfg!` inside a test describes the
    // host that COMPILED it, not the branch this process actually took under
    // `AWL_MENU_BAR_FORCE`.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    for menu_bar in [false, true] {
        crate::menubar::set_menu_bar_on(menu_bar);
        let mut measured = Vec::new();
        for zoom in [0.8_f32, 1.5_f32] {
            let png = dir.join(format!("menubar-{menu_bar}-zoom-{zoom}.png"));
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
            // FLOOR, never round: this bounds the census at the top of the first
            // row's box, and rounding up could cut into row 0's own ink.
            let text_top = side["text_origin"]["top"]
                .as_f64()
                .expect("text_origin.top")
                .floor()
                .max(0.0) as u32;
            let arm = format!("menu_bar {menu_bar} zoom {zoom}");

            assert!(
                (reported_zoom - zoom).abs() < 1e-4,
                "{arm}: reported {reported_zoom}"
            );
            assert!(
                (reported_size - crate::render::FONT_SIZE * zoom).abs() < 1e-3,
                "{arm}: effective font size {reported_size}"
            );

            let tops = measured_ink_band_tops(&png, text_left, text_top);
            assert!(
                tops.len() >= 7,
                "{arm}: expected seven independently measured glyph bands below the \
                 document origin y={text_top}, got {tops:?}"
            );
            // Within the document region the caret is the only other inker: it is
            // parked at the buffer end — the trailing BLANK line, below all seven —
            // so it contributes at most one band and that band is always the last.
            // Take the seven from the FRONT: a slice off the back selects by
            // counting rather than by position and quietly swaps the first glyph
            // row for the caret's own band the moment the band count changes.
            // What makes the FRONT safe is the detector's y bound, not luck —
            // chrome above the document used to land here.
            let repeated = &tops[..7];
            let pitches: Vec<f32> = repeated
                .windows(2)
                .map(|pair| (pair[1] - pair[0]) as f32)
                .collect();
            for pitch in &pitches {
                assert!(
                    (*pitch - reported_lh).abs() <= 1.0,
                    "{arm}: measured PNG row pitch {pitch}px must equal sidecar \
                     font.line_height {reported_lh}px (tops {tops:?}, document origin \
                     y={text_top})"
                );
            }
            measured.push((zoom, reported_lh, pitches[0]));
        }

        // Non-vacuity: this arm sampled genuinely different scales. The old sidecar
        // reported 32 at BOTH zooms while these PNG pitches are ~26 and 48.
        assert_ne!(
            measured[0].1, measured[1].1,
            "menu_bar {menu_bar}: reported line heights must scale"
        );
        assert_ne!(
            measured[0].2, measured[1].2,
            "menu_bar {menu_bar}: measured PNG pitches must scale"
        );
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);

    crate::page::set_measure(old_measure);
    crate::page::set_page_on(old_page);
    crate::spell::set_spellcheck_on(old_spellcheck);
}
