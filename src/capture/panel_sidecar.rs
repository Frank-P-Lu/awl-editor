//! Serialization of the SEARCH PANEL's planned geometry — `search.panel`.
//!
//! The `search` block already carried the panel's STATE: the query, the hit
//! count, which field is being edited. What it never carried was WHERE the card
//! is, so the one law asking a geometry question about it — "can a long query
//! widen the card?" — answered it by walking the PNG inward from the window's
//! right edge with a colour-distance threshold. A pixel walk is an appearance
//! oracle: it cannot distinguish a card that moved from a rim that changed tone,
//! and it measures the background instead of the card on any world whose page and
//! card fill agree.
//!
//! Every figure here is read off `TextPipeline::panel_geometry`, which reads the
//! same `panel_layout` the draw sizes the card from and the pointer inverts, plus
//! the panel's one row-band owner. This module performs no arithmetic on what it
//! is handed — no scaling, no rounding, no re-derivation — because a serializer
//! that adjusts a number becomes a second owner of it.
//!
//! **`null` when the panel is down**, on the pipeline's own `search_active` gate
//! rather than on a second guess at it — the same question `prepare_panel` asks
//! before drawing anything at all.
//!
//! Runs ONCE PER CAPTURE, from `sidecar::write_sidecar`. Nothing in
//! `render/pipeline_*` reaches it.

use crate::render::TextPipeline;
use crate::render::plan::{PanelGeometry, PanelRowRect};

/// `search.panel`, or `"null"` while no panel is summoned.
pub(super) fn panel_json(pipeline: &TextPipeline) -> String {
    let Some(g) = pipeline.panel_geometry() else {
        return "null".to_string();
    };
    format!(
        "{{ \"card\": {}, \"text\": {{ \"left\": {}, \"top\": {} }}, \
         \"rows\": [{}], \"case_toggle\": {} }}",
        card_json(&g),
        g.text_left,
        g.text_top,
        g.rows.iter().map(row_json).collect::<Vec<_>>().join(", "),
        case_toggle_json(&g),
    )
}

fn card_json(g: &PanelGeometry) -> String {
    let [x, y, w, h] = g.card;
    format!("{{ \"x\": {x}, \"y\": {y}, \"w\": {w}, \"h\": {h} }}")
}

/// One shaped row's band. `row` indexes the card's own shaped lines (0 = find,
/// 1 = replace, 2 = the key-hint line), NOT a document row.
fn row_json(row: &PanelRowRect) -> String {
    format!(
        "{{ \"row\": {}, \"top\": {}, \"h\": {} }}",
        row.row, row.top, row.h
    )
}

/// The `Aa` case indicator's x-span, or `null` when the find row shaped fewer
/// than the two glyphs it is read from. `null` rather than a zero-width span for
/// the reason the neighbouring lane keys use it: "there is no target" and "the
/// target is empty" are different facts, and a width of 0 states neither.
fn case_toggle_json(g: &PanelGeometry) -> String {
    g.case_toggle.map_or_else(
        || "null".to_string(),
        |(x0, x1)| format!("{{ \"x0\": {x0}, \"x1\": {x1} }}"),
    )
}
