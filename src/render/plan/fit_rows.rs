//! THE CANDIDATE-BAND HEIGHT CLAMP — how many item rows fit a resolved pixel
//! budget, for both overlay families. Split out of `overlay_rows.rs` (past its
//! own production ceiling) so this stays a small, independent arithmetic
//! module: no plan, no shaping, no device — the same purity `overlay_rows.rs`'s
//! own module doc names for the planner it still owns.

/// THE ONE HEIGHT-CLAMP OWNER. The GROUPED family previously
/// (`theme_overlay_geometry`) alone divided its own available pixels by the row
/// pitch to bound its item window; the FLAT family (`overlay_geometry`) capped
/// its window only at a per-kind row COUNT (`OverlayKind::window_rows`) that
/// knows nothing about the canvas. A flat picker whose kind sets that count to
/// its whole corpus — the theme picker, `window_rows() ==
/// crate::theme::THEMES.len()`, once its runtime lens strip retired (making it
/// flat) — drew a card taller than the canvas at ordinary sizes (`card_h: 934`
/// against `canvas_h: 800`, 19 world rows). This is now the ONE place either
/// family divides `avail_px` by `lh`; a caller may not re-derive the floor
/// division or the overhead subtraction itself, only ask this how many item
/// rows fit.
///
/// `avail_px` is the vertical space the caller has already resolved for the
/// candidate band (canvas height minus margins/padding/the query beat — the
/// SAME arithmetic every geometry owner already performed; this function does
/// not know about card_y, margin, or pad, only the pixel budget those leave
/// behind). `overhead_rows` is every display line in the card that is NOT a
/// candidate item — header/hint/footer/empty-state rows for a flat card,
/// PLUS the section-header count for a grouped card (its caller's own
/// concern; this function is generic over what counts as overhead).
///
/// `min_items` is the FAMILY's own floor, never a bare constant: the FLAT
/// family and the spell popup pass `1` — "a card always attempts to show its
/// own selection" — because their fixed overhead (one query line, no lens
/// strip) never grows past what a real canvas holds. The GROUPED
/// family passes `0`. Its own fixed overhead (the query line, the lens strip,
/// and the query BEAT between them — `theme_overlay_geometry`'s
/// `header_rows * lh + header_gap`) has no independent zoom ceiling, so at
/// the documented zoom limit on a short canvas that overhead ALONE can already
/// exceed `avail_px` before a single item or section header is counted (the
/// 900x460/zoom-3.0 sectioned command-palette case, `render/tests/
/// overlay_height_clamp_law.rs`). Forcing `.max(1)` there regardless cannot be
/// satisfied without overrunning the canvas: this is a chrome-overhead sizing
/// question, not a row-count one. Below the
/// floor no amount of item-count clamping can help either way; the difference
/// is only whether the family is CONTRACTUALLY guaranteed a row at that
/// floor (flat/spell) or willing to show an empty candidate band rather than
/// overrun the canvas (grouped). This is a no-op change wherever the floor
/// does not bind — `saturating_sub` already returns `>= min_items` whenever
/// `fit_lines > overhead_rows`, so every already-fitting picker (either
/// family) is byte-identical.
pub(in crate::render) fn fit_item_rows(
    avail_px: f32,
    lh: f32,
    overhead_rows: usize,
    min_items: usize,
) -> usize {
    if lh <= 0.0 {
        return min_items;
    }
    let fit_lines = (avail_px / lh).floor() as usize;
    fit_lines.saturating_sub(overhead_rows).max(min_items)
}

/// The pixel-reserved form of [`fit_item_rows`]. A fixed-height composition
/// uses this when its non-candidate chrome is not an integer number of row
/// pitches: the workspace teaching footer has a compact separator and compact
/// text line, so rounding both up to full rows can hide a candidate that really
/// fits, while rounding either down can seat the footer beyond its card.
///
/// `reserved_px` is charged before the remaining height is divided by the
/// candidate pitch. `min_items` retains the caller's family policy; a workspace
/// with a teaching footer passes zero because the footer is the navigation
/// instruction that must survive the minimum geometry.
pub(in crate::render) fn fit_item_rows_after_px(
    avail_px: f32,
    lh: f32,
    reserved_px: f32,
    min_items: usize,
) -> usize {
    if lh <= 0.0 {
        return min_items;
    }
    (((avail_px - reserved_px.max(0.0)).max(0.0) / lh).floor() as usize).max(min_items)
}

/// A SECTIONED card's item cap: [`fit_item_rows`]'s answer, except that an
/// answer of ZERO is re-derived with the section headers billed TIGHTLY before
/// it is accepted.
///
/// **THE `total_headers` CHARGE IS AN UPPER BOUND, NOT A COST.** `window_plan`
/// emits a header only ahead of the first SURVIVING item of a section, so a
/// window of `k` items can carry at most `k` headers — and at most
/// `total_headers`, the number the whole plan has. Charging every section
/// against a window that will show two of twenty-four items bills the budget
/// for headers no one will draw. That is harmless while the budget is roomy: a
/// conservative charge only ever costs a row nobody misses, and re-billing it
/// tightly EVERYWHERE would move shipped row counts on cards that already work
/// (including the contract that a hint costs exactly two rows of the candidate
/// window, pinned by `hint_gap`). It is not harmless at the one
/// outcome that is never
/// acceptable — a card that plans NO candidate rows at all. A 900x460 canvas
/// with the drawn menu bar's own vertical reserve taken out fits 7 display
/// lines, spends 4 on the query line, the lens strip, the hint and its
/// separator, and pays the last 3 for sections it has no room to reach: zero
/// item rows in a 192px card inside a 460px canvas.
///
/// So the tight bound is a FLOOR, engaged only there. It is the largest `k`
/// satisfying `chrome_rows + min(k, total_headers) + k <= fit_lines`, which
/// where the conservative charge already zeroed the band (`budget <=
/// total_headers`) is `budget / 2`. It is never optimistic — `min(k,
/// total_headers)` bounds the headers the window can carry — so the card it
/// sizes still cannot outgrow `avail_px`.
///
/// `min_items` keeps the grouped family's own `0` floor: where the fixed chrome
/// ALONE already exceeds the budget, no row count can help and an empty band
/// beats overrunning the canvas. That degradation is preserved exactly; what
/// this removes is the case where it fired with room to spare.
pub(in crate::render) fn fit_sectioned_item_rows(
    avail_px: f32,
    lh: f32,
    chrome_rows: usize,
    total_headers: usize,
    min_items: usize,
) -> usize {
    if lh <= 0.0 {
        return min_items;
    }
    let conservative = fit_item_rows(avail_px, lh, chrome_rows + total_headers, min_items);
    if conservative > 0 {
        return conservative;
    }
    let fit_lines = (avail_px / lh).floor() as usize;
    (fit_lines.saturating_sub(chrome_rows) / 2).max(min_items)
}
