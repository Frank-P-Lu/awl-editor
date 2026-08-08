//! **`overlay.window.band` / `overlay.window.rows` REACH THE ARTEFACT** — the
//! serializer's own half of the three-way agreement law.
//!
//! `render/tests/overlay_plan_law.rs` grades the published geometry against the
//! ink and against the pointer, but it grades the REPORT STRUCT. A serializer
//! sitting between that struct and the JSON can still drop a key, reorder a
//! rect, or — the failure this file exists for — quietly rescale a number.
//!
//! **EVERY CAPTURE RUNS AT `--capture-dpi 1`, the one scale at which a
//! device-pixel bug looks correct.** So the law here is the DPI RELATION: the
//! same picker, same logical window, captured at dpi 1 and dpi 2, must report
//! row geometry that doubles. A serializer that divided by the scale factor, or a
//! report that had been quietly converted to logical units, passes every
//! single-scale check and fails this one. The 1x figures are additionally
//! anchored against the canvas so "doubles" cannot be satisfied by two equally
//! wrong numbers.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::overlay::{OverlayKind, OverlayState};
use crate::testscratch::ScratchDir;

/// A real flat picker, driven through the production `OverlayState` so the fold
/// carries exactly what a live summon would.
fn flat_picker_opts(ov: &OverlayState, canvas: (u32, u32), dpi: f32) -> CaptureOpts {
    let mut opts = CaptureOpts {
        canvas: Some(canvas),
        dpi: Some(dpi),
        ..CaptureOpts::default()
    };
    opts.overlay = Some(OverlayInfo {
        align: crate::render::effective_card_anchor(),
        active: true,
        mode: ov.kind.as_str(),
        title: ov.kind.title(),
        query: ov.query.text().to_string(),
        items: ov.item_strings(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: ov.foot_hint(),
        browse_dir: ov.browse_dir.clone(),
        return_to: None,
        spell_target: None,
        context_anchor: None,
        capture: None,
        notice: String::new(),
        lens: ov.active_facet_id(),
        lens_strip: ov.lens_strip(),
        sections: ov.item_sections(),
        preview_id: None,
        preview_view: None,
        workspace: false,
        detail_focus: false,
        diff_scroll: 0,
        empty: ov.empty_notice(),
        show_hidden: false,
    });
    opts
}

/// One `overlay.window.rows[]` entry, read back out of the JSON rather than out
/// of the report struct — so a serializer that dropped or renamed a key fails
/// here at the `expect` rather than passing on a defaulted zero.
struct Row {
    display: u64,
    item: Option<u64>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    selected: bool,
}

struct Band {
    first_top: f64,
    pitch: f64,
    footer_top: f64,
    x: f64,
    w: f64,
    rows: Vec<Row>,
    sel_row: u64,
    canvas_h: f64,
}

fn read_band(png: &std::path::Path) -> Band {
    let text = std::fs::read_to_string(png.with_extension("json")).expect("sidecar exists");
    let v: serde_json::Value = serde_json::from_str(&text).expect("sidecar parses");
    let w = &v["overlay"]["window"];
    assert!(!w.is_null(), "an open picker must report a window");
    let b = &w["band"];
    assert!(
        b.is_object(),
        "schema /201: an open picker's window must carry a `band` object, got {b}"
    );
    let rows = w["rows"]
        .as_array()
        .expect("schema /201: `rows` is an array")
        .iter()
        .map(|r| Row {
            display: r["display"].as_u64().expect("display"),
            item: r["item"].as_u64(),
            x: r["x"].as_f64().expect("x"),
            y: r["y"].as_f64().expect("y"),
            w: r["w"].as_f64().expect("w"),
            h: r["h"].as_f64().expect("h"),
            selected: r["selected"].as_bool().expect("selected"),
        })
        .collect();
    Band {
        first_top: b["first_top"].as_f64().expect("first_top"),
        pitch: b["pitch"].as_f64().expect("pitch"),
        footer_top: b["footer_top"].as_f64().expect("footer_top"),
        x: b["x"].as_f64().expect("band x"),
        w: b["w"].as_f64().expect("band w"),
        rows,
        sel_row: w["sel_row"].as_u64().expect("sel_row"),
        canvas_h: w["canvas_h"].as_f64().expect("canvas_h"),
    }
}

#[test]
fn published_row_geometry_is_physical_pixels_and_scales_with_capture_dpi() {
    if !adapter_available() {
        eprintln!(
            "skipping published_row_geometry_is_physical_pixels_and_scales_with_capture_dpi: \
             no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_plan_geometry_{}", std::process::id())),
    );
    let buf = Buffer::from_str("a document behind the card\n");
    let items: Vec<String> = (0..30).map(|i| format!("candidate {i:02}")).collect();
    let mut ov = OverlayState::new(OverlayKind::Goto, items, vec![], vec![]);
    ov.move_sel(3);

    // ONE logical window at two scales: 1200x800 at dpi 1 and 2400x1600 at dpi 2
    // are the same (W/N)x(H/N) logical window, so every difference below is the
    // scale factor and nothing else.
    let one = dir.join("dpi1.png");
    capture_with(&one, &buf, &flat_picker_opts(&ov, (1200, 800), 1.0)).expect("dpi 1 capture");
    let two = dir.join("dpi2.png");
    capture_with(&two, &buf, &flat_picker_opts(&ov, (2400, 1600), 2.0)).expect("dpi 2 capture");
    let (a, b) = (read_band(&one), read_band(&two));

    // ANCHORS at 1x, so "doubles" cannot be satisfied by two equally wrong
    // numbers: the band is a real fraction of a real canvas, and it carries rows.
    assert!(
        !a.rows.is_empty() && a.rows.len() == b.rows.len(),
        "both scales must publish the same non-empty row band, got {} and {}",
        a.rows.len(),
        b.rows.len()
    );
    assert!(
        a.pitch > 8.0 && a.first_top > 0.0 && a.first_top < a.canvas_h,
        "the 1x band must sit inside its own canvas: first_top {} pitch {} canvas_h {}",
        a.first_top,
        a.pitch,
        a.canvas_h
    );
    assert!(
        a.w > 100.0 && a.x > 0.0,
        "the 1x content band must be a real width at a real x: x {} w {}",
        a.x,
        a.w
    );

    // ROW GEOMETRY IS INTERNALLY CONSISTENT at each scale: contiguous slots at
    // the reported pitch, seated at the reported origin, inside the content band,
    // ending where the footer begins, with exactly the reported row selected.
    for (name, g) in [("1x", &a), ("2x", &b)] {
        let mut selected = Vec::new();
        for (i, row) in g.rows.iter().enumerate() {
            let (x, y, w, h) = (row.x, row.y, row.w, row.h);
            assert_eq!(
                row.display as usize, i,
                "{name}: rows must be in draw order"
            );
            assert!(
                (y - (g.first_top + i as f64 * g.pitch)).abs() < 0.01,
                "{name}: row {i} is at y {y}, not first_top {} + {i} * pitch {}",
                g.first_top,
                g.pitch
            );
            assert!(
                (h - g.pitch).abs() < 0.01,
                "{name}: row {i} is {h} tall against a pitch of {}",
                g.pitch
            );
            // Overlap, not containment: a staggering composition's selected row
            // steps OUTWARD past the band edge on purpose (the shipped Saltpan
            // Settings card does exactly this), so containment would be a false
            // law. The scale relation below is what this test is really for.
            assert!(
                x + w > g.x && x < g.x + g.w && w > 1.0 && h > 1.0,
                "{name}: row {i} spans [{x}, {}] ({w}x{h}), which does not meet \
                 the band [{}, {}]",
                x + w,
                g.x,
                g.x + g.w
            );
            if row.selected {
                selected.push(row.display);
            }
        }
        assert_eq!(
            selected,
            vec![g.sel_row],
            "{name}: exactly the reported `sel_row` must carry `selected: true`"
        );
        let last = g.rows.last().expect("rows");
        assert!(
            g.footer_top >= last.y + last.h - 0.01,
            "{name}: footer_top {} must be at or below the last row's bottom {}",
            g.footer_top,
            last.y + last.h
        );
    }

    // THE DPI RELATION. Physical pixels, so every figure doubles.
    let doubles = |lo: f64, hi: f64, what: &str| {
        assert!(
            (hi - lo * 2.0).abs() <= 1.0,
            "{what} is {lo} at --capture-dpi 1 and {hi} at 2: a PHYSICAL-pixel field \
             must double with the scale factor (a logical one would not move, and a \
             double-scaled one would quadruple)"
        );
    };
    doubles(a.first_top, b.first_top, "band.first_top");
    doubles(a.pitch, b.pitch, "band.pitch");
    doubles(a.footer_top, b.footer_top, "band.footer_top");
    doubles(a.x, b.x, "band.x");
    doubles(a.w, b.w, "band.w");
    for (i, (lo, hi)) in a.rows.iter().zip(b.rows.iter()).enumerate() {
        assert_eq!(
            (lo.display, lo.item, lo.selected),
            (hi.display, hi.item, hi.selected),
            "row {i}: display / item / selected must not depend on the capture scale"
        );
        doubles(lo.x, hi.x, &format!("rows[{i}].x"));
        doubles(lo.y, hi.y, &format!("rows[{i}].y"));
        doubles(lo.w, hi.w, &format!("rows[{i}].w"));
        doubles(lo.h, hi.h, &format!("rows[{i}].h"));
    }
}
