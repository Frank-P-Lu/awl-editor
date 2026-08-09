//! src/render/plan/ — THE SCENE PLANNER.
//!
//! A deterministic, device-free layout stage between the measured inputs and the
//! GPU. It consumes a `ViewState` projection (the pipeline's `overlay_*` mirror),
//! already-measured text inputs (the overlay row pitch, the resolved card box,
//! per-row shaped widths), theme capabilities (the row gap / list style, folded
//! into the pitch by its own owner) and viewport data — and emits INSPECTABLE
//! primitives plus the INTERACTION GEOMETRY that answers a pointer.
//!
//! THE RULE THIS MODULE EXISTS TO ENFORCE: a drawn rectangle, the hit-test that
//! accepts a click inside it, and the sidecar's report of it are the SAME planned
//! object, read three times — never three arithmetic expressions that happen to
//! agree today. The forward `row -> y` arithmetic and its `y -> row` inverse are
//! PRIVATE to [`overlay_rows`]; a consumer cannot reach them, so the only way to
//! learn where a row sits is to ask the plan that drew it.
//!
//! WHAT THIS IS NOT: a retained widget tree (a plan is built, read, and dropped
//! within one frame), a general scene framework (it plans the surfaces that have
//! been migrated to it and nothing else), or a second CPU renderer (it emits
//! geometry; `render/chrome` still owns every pixel decision downstream).
//!
//! O(VISIBLE), NOT O(DOC): a plan holds one [`PlannedRow`] per candidate DISPLAY
//! LINE the card actually shows — never one per item in the corpus. A 10,000-row
//! Go-to picker plans the twelve rows on screen; `plan_rows_are_bounded_by_the_window`
//! (`render/tests/overlay_plan_law.rs`) pins that, and the `palette` bench cell
//! carries the same bound as a witness counter.
//!
//! Shaping and cache ownership stay where they were: this module measures
//! nothing and shapes nothing. It is handed the metrics the measured stage
//! already produced.
//!
//! **THE HEADER BAND** — the query/title INPUT line and the grouped family's
//! lens STRIP — is planned by the same code as the candidate rows, so the four
//! answers that used to live in four separate `render/chrome` owners
//! (`overlay_secondary_top`, `overlay_split_bounds`, `overlay_strip_band`,
//! `overlay_query_center`, all DELETED) are one object read many ways: the
//! query caret's centre, the field's own pointer band, the strip's clickable
//! band and active-mark centre, the secondary column's upload origin, and the
//! split-pane surfaces' gap.
//!
//! **THE ACCESSORY CLUSTER** — a row's NAME lane, its VALUE lane and its Range
//! rail — is seated by [`accessory_lane`]'s accessors, so the rule "name on the
//! spine end, accessory on the outer end, mirroring as a unit" has one spelling
//! rather than one per consumer. The accessory upload, the frost's surface list,
//! the rail owner and the published projection all ask the same question there.
//!
//! **THE HEIGHT CLAMP** — `fit_item_rows` — is the one owner of
//! "how many candidate item rows fit the canvas", shared by both families
//! (`render/chrome/overlay.rs`'s flat window and `render/chrome/theme_picker.rs`'s
//! grouped window) so a picker with a big corpus cannot draw a card taller than
//! its canvas whichever geometry path it takes. It is plain arithmetic over
//! already-resolved floats (an available-pixel budget, a row pitch, an overhead
//! row count) — no device, no shaping, no clock — so it keeps the planner pure.

mod accessory_lane;
mod floating;
mod overlay_header;
mod overlay_row_plan;
mod overlay_rows;
mod panel_report;
mod popover;
mod row_extent;
mod row_report;

/// The ONLY names this module publishes past `crate::render`: the PUBLISHED
/// projection the capture sidecar serializes. Everything else stays
/// `pub(in crate::render)`, so widening `mod plan` for the sidecar's sake did
/// not widen the planner's own internals with it.
pub(crate) use accessory_lane::{Lane, RailLane, RowLanes};
pub(crate) use panel_report::{PanelGeometry, PanelRowRect};
pub(crate) use row_report::{OverlayRowGeometry, PlannedRowRect};

pub(in crate::render) use floating::{
    CornerAnchor, StreaksCardInput, plan_corner_label, plan_float_card, plan_streaks_card,
    plan_whichkey_card,
};
pub(in crate::render) use overlay_header::{PlannedHeader, beat_stands_alone, header_band_height};
pub(in crate::render) use overlay_rows::plan_witness;
pub(in crate::render) use overlay_rows::{
    OverlayRowPlan, OverlayRowPlanInput, PlanLine, PlannedRow, fit_item_rows,
    fit_sectioned_item_rows, plan_overlay_rows,
};
#[cfg(test)]
pub(in crate::render) use overlay_rows::{test_header_plan, test_row_top, test_rows};
pub(in crate::render) use popover::{
    MeasuredPopoverButton, PopoverButtonGeom, PopoverGeom, PopoverPlanInput, plan_popover,
    plan_spell_anchor,
};
#[cfg(test)]
pub(in crate::render) use row_extent::RowExtent;
pub(in crate::render) use row_extent::{ClusterExtent, RowSpan};

#[cfg(test)]
mod tests;
