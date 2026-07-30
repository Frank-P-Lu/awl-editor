//! src/render/plan/ — THE SCENE PLANNER (item 174).
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

mod overlay_rows;

#[cfg(test)]
pub(in crate::render) use overlay_rows::test_row_top;
pub(in crate::render) use overlay_rows::{
    OverlayRowPlan, OverlayRowPlanInput, PlanLine, PlannedRow, plan_overlay_rows,
};

#[cfg(test)]
mod tests;
