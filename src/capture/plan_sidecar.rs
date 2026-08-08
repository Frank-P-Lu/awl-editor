//! Serialization of the renderer's PLANNED OVERLAY GEOMETRY — `overlay.window`.
//!
//! The block already carried the window's plan-derived COUNTS (`lines`,
//! `sel_row`) and the two heights a boundedness check needs; what it never
//! carried was WHERE a candidate row is, so every geometry assertion about a
//! picker row had to be recovered from the PNG. A pixel is an appearance oracle;
//! a row rect is a geometry fact, and the plan has always known it.
//!
//! **A ROW'S THREE PARTS ARE PUBLISHED SEPARATELY** — the name lane, the value
//! lane, the rail — because they are separately answerable and the interesting
//! question about a narrowing card is WHICH ONE ran out of room. Reported as one
//! rect they cannot be told apart; reported apart, "what did each part need at
//! the width where the accessory column was dropped" is arithmetic over a
//! sidecar instead of a hand-instrumented build. Each is `null` when the frame
//! drew nothing there, which is itself the answer at the yielding width.
//!
//! Both halves are read from the same `TextPipeline` accessors that the draw
//! emitters and the pointer hit-test read — `overlay_window_report` and
//! `overlay_row_geometry`, each of which builds this frame's plan through the one
//! planning seam. This module performs no arithmetic on what it is handed
//! (no scaling, no rounding, no re-derivation), because a serializer that
//! adjusts a number is a second owner of that number.
//!
//! Runs ONCE PER CAPTURE, from `sidecar::write_sidecar`. It is not on the frame
//! path: nothing in `render/pipeline_*` calls into it.

use crate::render::TextPipeline;
use crate::render::plan::{Lane, OverlayRowGeometry, PlannedRowRect, RailLane};

/// `overlay.window`, or `"null"` when no overlay is summoned.
pub(super) fn window_json(pipeline: &TextPipeline) -> String {
    let Some((top, lines, sel_row, card_h, canvas_h)) = pipeline.overlay_window_report() else {
        return "null".to_string();
    };
    format!(
        "{{ \"top\": {top}, \"lines\": {lines}, \"sel_row\": {sel_row}, \
         \"card_h\": {card_h}, \"canvas_h\": {canvas_h}{} }}",
        geometry_fields(pipeline)
    )
}

/// The `band` + `rows` half, as trailing `, "k": v` fragments so the historical
/// key order in front of them is untouched.
///
/// Empty only when the pipeline reports no geometry at all — which
/// `overlay_window_report` having answered already rules out, since both ask the
/// same `overlay_active` question. It stays a graceful empty rather than an
/// `expect` because a sidecar that panics tells a reader nothing about the frame.
fn geometry_fields(pipeline: &TextPipeline) -> String {
    let Some(g) = pipeline.overlay_row_geometry() else {
        return String::new();
    };
    format!(
        ", \"band\": {}, \"rows\": [{}]",
        band_json(&g),
        g.rows.iter().map(row_json).collect::<Vec<_>>().join(", ")
    )
}

fn band_json(g: &OverlayRowGeometry) -> String {
    format!(
        "{{ \"x\": {}, \"w\": {}, \"first_top\": {}, \"pitch\": {}, \
         \"footer_top\": {} }}",
        g.band_x, g.band_w, g.first_top, g.pitch, g.footer_top
    )
}

fn row_json(row: &PlannedRowRect) -> String {
    let item = row
        .item
        .map_or_else(|| "null".to_string(), |i| i.to_string());
    format!(
        "{{ \"display\": {}, \"item\": {item}, \"x\": {}, \"y\": {}, \"w\": {}, \
         \"h\": {}, \"label\": {}, \"value\": {}, \"rail\": {} }}",
        row.display,
        row.x,
        row.y,
        row.w,
        row.h,
        lane_json(row.lanes.label),
        lane_json(row.lanes.value),
        rail_json(row.lanes.rail),
    )
}

/// A text lane, or `null` for a lane the frame drew nothing in. `null` rather
/// than a zero-width rect on purpose: "the column was yielded" and "the column
/// is there and empty" are different facts about the card, and a width of 0 says
/// neither of them out loud.
fn lane_json(lane: Option<Lane>) -> String {
    lane.map_or_else(
        || "null".to_string(),
        |l| format!("{{ \"x\": {}, \"w\": {} }}", l.x, l.w),
    )
}

fn rail_json(rail: Option<RailLane>) -> String {
    rail.map_or_else(
        || "null".to_string(),
        |r| {
            format!(
                "{{ \"x\": {}, \"w\": {}, \"hit_x\": {}, \"hit_w\": {} }}",
                r.x, r.w, r.hit_x, r.hit_w
            )
        },
    )
}
