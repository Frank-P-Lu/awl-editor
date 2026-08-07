//! `--screenshot-frames` HONOURING `--capture-size`/`--capture-dpi`.
//!
//! `Mode::ScreenshotFrames` carries `canvas`/`dpi` fields threaded onto the
//! `CaptureOpts` `run.rs`'s handler builds, and `capture::frames::
//! capture_frames_async` calls `pipeline.set_dpi` the same way every other
//! capture path does. Proved here by MEASURING geometry a narrower canvas and
//! a scaled dpi actually produce (mirroring the form every other capture
//! door's canvas/dpi law uses), not by asserting a field was set.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::testscratch::ScratchDir;

fn json(path: &std::path::Path) -> serde_json::Value {
    let bytes = std::fs::read_to_string(path).expect("sidecar exists");
    serde_json::from_str(&bytes).expect("sidecar parses")
}

/// Count the visual rows the sidecar's `layout.rows` reports for source line 0
/// — i.e. how many times the document's one long line wrapped.
fn wraps_for_line0(v: &serde_json::Value) -> usize {
    v["layout"]["rows"]
        .as_array()
        .expect("a frame sidecar carries a layout block")
        .iter()
        .filter(|row| row["line"].as_u64() == Some(0))
        .count()
}

#[test]
fn a_screenshot_frames_capture_honors_capture_size_and_the_dpi_meaning_holds() {
    if !adapter_available() {
        eprintln!(
            "skipping a_screenshot_frames_capture_honors_capture_size_and_the_dpi_meaning_holds: \
             no wgpu adapter"
        );
        return;
    }
    let _g = crate::testlock::serial();
    // PAGE MODE OFF: the default page column is a fixed 70-CHARACTER measure,
    // which a 1200px-wide canvas already exceeds — so wrap count would saturate
    // at the page cap regardless of canvas/dpi and the law would pass whether
    // or not either flag actually reached the renderer (measured: with page
    // mode on, a mutated build that never scales dpi still wraps line 0
    // identically at 1200x800@1 and 2400x1600@2, because BOTH canvases exceed
    // the same character cap). With page mode off, wrapping follows the RAW
    // window width, which genuinely depends on canvas and on dpi's font-size
    // scale — the axis this law needs to be sensitive to.
    let old_page = crate::page::page_on();
    let old_measure = crate::page::measure();
    crate::page::set_page_on(false);
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-frames-canvas-dpi-{}", std::process::id())),
    );
    // One long logical line with plentiful word-wrap points, long enough to
    // wrap several times even at the wide canvas — the same fixture shape the
    // sibling `--screenshot-app` canvas/dpi law uses.
    let long_line = "supercalifragilistic ".repeat(60);
    let buf = Buffer::from_str(&long_line);

    let wide_png = dir.join("wide.png");
    let narrow_png = dir.join("narrow.png");
    let dpi2_png = dir.join("dpi2.png");
    let opts_at = |canvas: Option<(u32, u32)>, dpi: Option<f32>| CaptureOpts {
        canvas,
        dpi,
        ..CaptureOpts::default()
    };

    capture_frames(
        &wide_png,
        &buf,
        1,
        DEFAULT_FRAME_STEP_MS,
        &opts_at(Some((1200, 800)), None),
    )
    .expect("wide frame capture needs a GPU adapter");
    capture_frames(
        &narrow_png,
        &buf,
        1,
        DEFAULT_FRAME_STEP_MS,
        &opts_at(Some((640, 800)), None),
    )
    .expect("narrow frame capture needs a GPU adapter");
    // Double the wide canvas AND the dpi: same LOGICAL window (1200x800).
    capture_frames(
        &dpi2_png,
        &buf,
        1,
        DEFAULT_FRAME_STEP_MS,
        &opts_at(Some((2400, 1600)), Some(2.0)),
    )
    .expect("dpi-2 frame capture needs a GPU adapter");

    let wide_json = json(&dir.join("wide.f000.json"));
    let narrow_json = json(&dir.join("narrow.f000.json"));
    let dpi2_json = json(&dir.join("dpi2.f000.json"));

    // The fields are genuinely threaded (not left at the 1200x800/dpi-1
    // default) — necessary, but per the doc comment above, not sufficient.
    assert_eq!(wide_json["canvas"]["width"].as_u64(), Some(1200));
    assert_eq!(narrow_json["canvas"]["width"].as_u64(), Some(640));
    assert_eq!(dpi2_json["canvas"]["width"].as_u64(), Some(2400));
    assert_eq!(dpi2_json["canvas"]["dpi"].as_f64(), Some(2.0));

    let wide_wraps = wraps_for_line0(&wide_json);
    let narrow_wraps = wraps_for_line0(&narrow_json);
    let dpi2_wraps = wraps_for_line0(&dpi2_json);

    // Non-vacuity + the actual geometry claim: a narrower PHYSICAL canvas
    // wraps into genuinely MORE visual rows.
    assert!(
        narrow_wraps > wide_wraps,
        "narrow canvas must wrap into more rows than wide (narrow={narrow_wraps} \
         wide={wide_wraps})"
    );
    // The documented dpi meaning: a `WxH` physical canvas at `--capture-dpi N`
    // is the SAME logical `(W/N)x(H/N)` window, so 2400x1600@2 must wrap
    // IDENTICALLY to the 1200x800@1 baseline.
    assert_eq!(
        dpi2_wraps, wide_wraps,
        "2400x1600 @ dpi 2 must match the 1200x800 @ dpi 1 baseline (dpi2={dpi2_wraps} \
         wide={wide_wraps})"
    );

    // A SECOND, independent dpi signal that cannot saturate the way wrap
    // counts can: `--capture-dpi` scales the rendered font size directly
    // (`Metrics::with_dpi`'s `font_size = FONT_SIZE * zoom * dpi`), so the
    // dpi-2 frame's reported `font.size` must be exactly double the dpi-1
    // baseline's — proof that `pipeline.set_dpi` genuinely ran, independent of
    // any word-wrap geometry.
    let wide_font_size = wide_json["font"]["size"].as_f64().expect("font.size");
    let dpi2_font_size = dpi2_json["font"]["size"].as_f64().expect("font.size");
    assert!(
        (dpi2_font_size - 2.0 * wide_font_size).abs() < 1e-3,
        "dpi-2 font.size {dpi2_font_size} must be double the dpi-1 baseline {wide_font_size}"
    );

    crate::page::set_page_on(old_page);
    crate::page::set_measure(old_measure);
}
