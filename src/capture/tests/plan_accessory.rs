//! **`overlay.window.rows[].label` / `.value` / `.rail` REACH THE ARTEFACT** —
//! the serializer's own half of the accessory-lane law.
//!
//! `render/plan/accessory_law.rs` grades the published lanes against the shaped
//! ink and the pointer, but it grades the REPORT STRUCT. A serializer sitting
//! between that struct and the JSON can still drop a key, emit `null` for a lane
//! the frame drew, or — the failure this file exists for — quietly rescale a
//! number.
//!
//! **EVERY CAPTURE RUNS AT `--capture-dpi 1`, the one scale at which a
//! device-pixel bug looks correct.** So the law here is the DPI RELATION: the
//! same picker, same logical window, captured at dpi 1 and dpi 2, must report
//! lane geometry that DOUBLES — every x, every width, and the rail's hit band
//! too. A serializer that divided by the scale factor, or a lane that had been
//! quietly converted to logical units, passes every single-scale check and fails
//! this one. The 1x figures are additionally anchored against the band, so
//! "doubles" cannot be satisfied by two equally wrong numbers.
//!
//! **AND THE `null` ARM IS GRADED SEPARATELY**, because it is a different
//! spelling and a serializer can get one right while getting the other wrong. A
//! picker with no accessory column at all must reach the JSON as `value: null` /
//! `rail: null` beside a real `label` — never as a zero-width rect, which would
//! read as "the column is there and empty" and is the answer that makes an
//! undrawn lane indistinguishable from a drawn one.

use super::super::*;
use super::adapter_available;
use super::plan_geometry::flat_picker_opts;
use crate::buffer::Buffer;
use crate::overlay::{OverlayKind, OverlayState};
use crate::testscratch::ScratchDir;

/// A real Settings picker, built through the same production wiring
/// `overlay::build`'s Settings arm uses. It is the one corpus that carries all
/// three lanes at once: a name, a value readout, and rails on its Range rows.
fn settings_state() -> OverlayState {
    let vals = crate::settings::SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 1.0,
        scroll_sensitivity: 1.0,
        default_folder: "/n".into(),
        workspace: "/w".into(),
        // SHORT ON PURPOSE. This row's value is the only accessory that is a full
        // UN-ELIDED absolute path, so a long checkout path would make every figure
        // below a property of the machine the suite runs on.
        project_root: "/p".into(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    };
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    ov
}

#[derive(Clone, Copy, Debug)]
struct Span {
    x: f64,
    w: f64,
}

#[derive(Clone, Copy, Debug)]
struct Rail {
    x: f64,
    w: f64,
    hit_x: f64,
    hit_w: f64,
}

struct Row {
    display: u64,
    item: Option<u64>,
    label: Option<Span>,
    value: Option<Span>,
    rail: Option<Rail>,
}

/// Read the lane keys back out of the JSON rather than out of the report struct,
/// so a serializer that dropped or renamed one fails at the `expect` here rather
/// than passing on a defaulted zero. A MISSING key and a `null` VALUE are told
/// apart deliberately: `null` is the product saying "nothing was drawn there",
/// while an absent key is the serializer having stopped emitting the field.
fn read_rows(png: &std::path::Path) -> (Vec<Row>, f64, f64) {
    let text = std::fs::read_to_string(png.with_extension("json")).expect("sidecar exists");
    let v: serde_json::Value = serde_json::from_str(&text).expect("sidecar parses");
    let w = &v["overlay"]["window"];
    assert!(!w.is_null(), "an open picker must report a window");
    let band = &w["band"];
    let span = |v: &serde_json::Value, what: &str| -> Option<Span> {
        if v.is_null() {
            return None;
        }
        Some(Span {
            x: v["x"]
                .as_f64()
                .unwrap_or_else(|| panic!("schema /202: {what}.x")),
            w: v["w"]
                .as_f64()
                .unwrap_or_else(|| panic!("schema /202: {what}.w")),
        })
    };
    let rows = w["rows"]
        .as_array()
        .expect("schema /201: `rows` is an array")
        .iter()
        .map(|r| {
            for key in ["label", "value", "rail"] {
                assert!(
                    r.get(key).is_some(),
                    "schema /202: every rows[] entry must carry a `{key}` key \
                     (absent, not null) — got {r}"
                );
            }
            let rail = &r["rail"];
            Row {
                display: r["display"].as_u64().expect("display"),
                item: r["item"].as_u64(),
                label: span(&r["label"], "label"),
                value: span(&r["value"], "value"),
                rail: rail.is_null().then_some(()).map_or_else(
                    || {
                        Some(Rail {
                            x: rail["x"].as_f64().expect("schema /202: rail.x"),
                            w: rail["w"].as_f64().expect("schema /202: rail.w"),
                            hit_x: rail["hit_x"].as_f64().expect("schema /202: rail.hit_x"),
                            hit_w: rail["hit_w"].as_f64().expect("schema /202: rail.hit_w"),
                        })
                    },
                    |()| None,
                ),
            }
        })
        .collect();
    (
        rows,
        band["x"].as_f64().expect("band x"),
        band["w"].as_f64().expect("band w"),
    )
}

/// One scale's lanes on their own terms, so the DPI relation below cannot be
/// satisfied by two equally wrong numbers.
fn assert_internally_consistent(name: &str, rows: &[Row], band_x: f64, band_w: f64) {
    let mut labels = 0usize;
    let mut values = 0usize;
    let mut rails = 0usize;
    for row in rows {
        let k = row.display;
        if let Some(l) = row.label {
            labels += 1;
            assert!(l.w > 0.0, "{name} row {k}: a reported label is never empty");
            // OVERLAP, not containment: a staggering composition steps its
            // selected row OUTWARD past the band edge on purpose, so a lane can
            // legitimately begin left of `band.x`.
            assert!(
                l.x + l.w > band_x && l.x < band_x + band_w,
                "{name} row {k}: label [{}, {}] does not meet the band [{band_x}, {}]",
                l.x,
                l.x + l.w,
                band_x + band_w
            );
        }
        if let Some(v) = row.value {
            values += 1;
            assert!(v.w > 0.0, "{name} row {k}: a reported value is never empty");
            if let Some(l) = row.label {
                assert!(
                    l.x + l.w <= v.x + 0.01 || v.x + v.w <= l.x + 0.01,
                    "{name} row {k}: the name lane [{}, {}] and the value lane \
                     [{}, {}] overlap",
                    l.x,
                    l.x + l.w,
                    v.x,
                    v.x + v.w
                );
            }
        }
        if let Some(r) = row.rail {
            rails += 1;
            assert!(
                row.value.is_some() && row.item.is_some(),
                "{name} row {k}: a rail belongs to a selectable row with a readout"
            );
            assert!(
                r.w > 0.0 && r.hit_w > r.w,
                "{name} row {k}: the hit band ({}) must be more generous than the \
                 drawn track ({})",
                r.hit_w,
                r.w
            );
            assert!(
                r.hit_x <= r.x + 0.01 && r.hit_x + r.hit_w >= r.x + r.w - 0.01,
                "{name} row {k}: the hit band [{}, {}] must contain the track \
                 [{}, {}]",
                r.hit_x,
                r.hit_x + r.hit_w,
                r.x,
                r.x + r.w
            );
        }
    }
    assert!(
        labels > 0 && values > 0 && rails > 0,
        "{name}: this card must carry all three lanes for the law to be grading \
         anything — got {labels} labels, {values} values, {rails} rails"
    );
}

#[test]
fn published_row_lanes_are_physical_pixels_and_scale_with_capture_dpi() {
    if !adapter_available() {
        eprintln!(
            "skipping published_row_lanes_are_physical_pixels_and_scale_with_capture_dpi: \
             no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_plan_accessory_{}", std::process::id())),
    );
    let buf = Buffer::from_str("a document behind the card\n");
    let ov = settings_state();

    // ONE logical window at two scales: 1200x800 at dpi 1 and 2400x1600 at dpi 2
    // are the same (W/N)x(H/N) logical window, so every difference below is the
    // scale factor and nothing else.
    let one = dir.join("dpi1.png");
    capture_with(&one, &buf, &flat_picker_opts(&ov, (1200, 800), 1.0)).expect("dpi 1 capture");
    let two = dir.join("dpi2.png");
    capture_with(&two, &buf, &flat_picker_opts(&ov, (2400, 1600), 2.0)).expect("dpi 2 capture");
    let (a, ax, aw) = read_rows(&one);
    let (b, bx, bw) = read_rows(&two);

    assert!(
        !a.is_empty() && a.len() == b.len(),
        "both scales must publish the same non-empty row band, got {} and {}",
        a.len(),
        b.len()
    );
    assert_internally_consistent("1x", &a, ax, aw);
    assert_internally_consistent("2x", &b, bx, bw);

    let doubles = |lo: f64, hi: f64, what: &str| {
        assert!(
            (hi - lo * 2.0).abs() <= 1.0,
            "{what} is {lo} at --capture-dpi 1 and {hi} at 2: a PHYSICAL-pixel field \
             must double with the scale factor (a logical one would not move, and a \
             double-scaled one would quadruple)"
        );
    };
    for (i, (lo, hi)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            (lo.display, lo.item),
            (hi.display, hi.item),
            "row {i}: display / item must not depend on the capture scale"
        );
        // PRESENCE must not depend on the scale either: one scale reporting a lane
        // the other calls `null` is the same class of bug as a rescaled number, and
        // it would slip past a loop that only compares the lanes both scales have.
        assert_eq!(
            (
                lo.label.is_some(),
                lo.value.is_some(),
                lo.rail.is_some()
            ),
            (
                hi.label.is_some(),
                hi.value.is_some(),
                hi.rail.is_some()
            ),
            "row {i}: which lanes the frame drew must not depend on the capture scale"
        );
        if let (Some(l), Some(h)) = (lo.label, hi.label) {
            doubles(l.x, h.x, &format!("rows[{i}].label.x"));
            doubles(l.w, h.w, &format!("rows[{i}].label.w"));
        }
        if let (Some(l), Some(h)) = (lo.value, hi.value) {
            doubles(l.x, h.x, &format!("rows[{i}].value.x"));
            doubles(l.w, h.w, &format!("rows[{i}].value.w"));
        }
        if let (Some(l), Some(h)) = (lo.rail, hi.rail) {
            doubles(l.x, h.x, &format!("rows[{i}].rail.x"));
            doubles(l.w, h.w, &format!("rows[{i}].rail.w"));
            doubles(l.hit_x, h.hit_x, &format!("rows[{i}].rail.hit_x"));
            doubles(l.hit_w, h.hit_w, &format!("rows[{i}].rail.hit_w"));
        }
    }
}

/// **THE `null` ARM REACHES THE ARTEFACT.** The sweep above grades a card whose
/// every lane is drawn; this one grades the other spelling. A picker with no
/// accessory column at all — no chords, no values, no ranges — must publish
/// `value: null` and `rail: null` on every row while still publishing a `label`.
///
/// Both arms are needed and neither substitutes for the other: a serializer that
/// emitted a zero-width rect instead of `null` passes the first law and fails
/// here, and one that emitted `null` for everything fails the first.
#[test]
fn a_picker_with_no_accessory_column_publishes_null_lanes_beside_real_labels() {
    if !adapter_available() {
        eprintln!("skipping a_picker_with_no_accessory_column_publishes_null_lanes: no adapter");
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_plan_nolane_{}", std::process::id())),
    );
    let buf = Buffer::from_str("a document behind the card\n");
    // No bindings and no range cells, so `overlay_right_labels` is empty, the
    // emitter never marks a right column shown, and no row carries a rail.
    let items: Vec<String> = (0..12).map(|i| format!("candidate {i:02}")).collect();
    let ov = OverlayState::new(OverlayKind::Goto, items, vec![], vec![]);
    let png = dir.join("bare.png");
    capture_with(&png, &buf, &flat_picker_opts(&ov, (1200, 800), 1.0)).expect("capture");
    let (rows, band_x, band_w) = read_rows(&png);
    assert!(!rows.is_empty(), "the picker must publish rows");
    let mut labels = 0usize;
    for row in &rows {
        assert!(
            row.value.is_none() && row.rail.is_none(),
            "row {}: a picker with no accessory column publishes `null`, never a \
             zero-width rect — got value {:?} rail {:?}",
            row.display,
            row.value,
            row.rail
        );
        if let Some(l) = row.label {
            labels += 1;
            assert!(
                l.w > 0.0 && l.x + l.w > band_x && l.x < band_x + band_w,
                "row {}: the name lane survives on a card with no accessory column, \
                 as real ink inside the band: [{}, {}] against [{band_x}, {}]",
                row.display,
                l.x,
                l.x + l.w,
                band_x + band_w
            );
        }
    }
    assert_eq!(
        labels,
        rows.len(),
        "every row of this picker carries a name, so a missing label lane is the \
         serializer dropping one rather than the product drawing none"
    );
}
