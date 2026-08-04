//! A rake-reading location cue's angle: derived from the diagonal spine's own
//! step, never a pinned constant.

/// A rake-reading location cue's angle, DERIVED from the spine's own step —
/// never a pinned constant. `row_step` should be the row's own MEASURED step
/// (`super::DiagonalClusterRail::spine_step`, narrowed by a too-narrow card's
/// yield), not the authored `super::DiagonalComposition::row_step`: the two
/// agree on an ordinary card, but reading the authored constant on a tight
/// card would lean the label MORE than the spine beside it actually does.
/// `row_height` is the label's own planned row, in device px, folded through
/// zoom and DPI exactly like `row_step` — so the ratio (and the angle it
/// derives) is scale-invariant.
///
/// THE SIGN follows the READING direction, not the spine's growth direction:
/// the label reads bottom-to-top, the OPPOSITE of the direction increasing
/// `display` steps the spine. An Ascending `/` spine (`row_step < 0`, whose
/// successive rows step LEFT as they descend) therefore reads upward leaning
/// RIGHT — the near-vertical `base_deg` below, unchanged. A Descending `\`
/// spine (`row_step > 0`) reads upward leaning LEFT — the same magnitude
/// reflected across vertical, `180.0 - base_deg` (`label_axis_deg`'s own
/// `[cos, -sin]` convention: this negates `cos` and leaves `-sin` — the
/// "reads upward" component — unchanged). `row_step`'s own sign already
/// tells the two directions apart, so this reads it directly rather than
/// taking a second, separately-suppliable direction that could disagree.
pub(in crate::render) fn location_axis_deg(row_step: f32, row_height: f32) -> f32 {
    if row_height.is_nan() || row_height <= 0.0 {
        // A degenerate row has no ratio to derive from; keep the vertical
        // fallback (RotatedRail's own axis) rather than divide by zero.
        return 90.0;
    }
    let lean_deg = row_step.abs().atan2(row_height).to_degrees();
    let base_deg = 90.0 - lean_deg;
    if row_step <= 0.0 {
        base_deg
    } else {
        180.0 - base_deg
    }
}
