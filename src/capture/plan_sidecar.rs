//! Serialization of the renderer's PLANNED OVERLAY GEOMETRY — `overlay.window`.
//!
//! The block already carried the window's plan-derived COUNTS (`lines`,
//! `sel_row`) and the two heights a boundedness check needs; what it never
//! carried was WHERE a candidate row is, so every geometry assertion about a
//! picker row had to be recovered from the PNG. A pixel is an appearance oracle;
//! a row rect is a geometry fact, and the plan has always known it.
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
use crate::render::plan::{OverlayRowGeometry, PlannedRowRect};

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
    let sel = g
        .selected_display
        .map_or_else(|| "null".to_string(), |d| d.to_string());
    format!(
        "{{ \"x\": {}, \"w\": {}, \"first_top\": {}, \"pitch\": {}, \
         \"footer_top\": {}, \"selected_display\": {sel} }}",
        g.band_x, g.band_w, g.first_top, g.pitch, g.footer_top
    )
}

fn row_json(row: &PlannedRowRect) -> String {
    let item = row
        .item
        .map_or_else(|| "null".to_string(), |i| i.to_string());
    format!(
        "{{ \"display\": {}, \"item\": {item}, \"x\": {}, \"y\": {}, \"w\": {}, \
         \"h\": {}, \"selected\": {} }}",
        row.display, row.x, row.y, row.w, row.h, row.selected
    )
}
