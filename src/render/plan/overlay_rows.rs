//! The planned candidate-row band of a summoned overlay card.
//!
//! ONE plan per overlay frame answers every question anything downstream may ask
//! about a candidate row: where it is drawn, what item it means, which row a
//! pointer lands on, which row the state selects, and where the footer band
//! begins. Both card families are planned by the same code — the FLAT window
//! (`items[top_idx .. top_idx + visible]`, one display line each) and the GROUPED
//! window (an explicit [`PlanLine`] sequence whose section headers push the item
//! rows down) — so each logical item has exactly one display line.

use super::PlannedHeader;
use super::row_extent::{RowExtent, RowSpan, apply_row_extent};

/// One DISPLAY line in an overlay card's candidate area: the card's SECONDARY
/// LOCATION heading, a faint uppercase section header, or a candidate row
/// carrying its index into `overlay_items`.
///
/// `Location` and `Header` occupy the same slot and the same pitch — they differ
/// only in what they SAY. A `Header` names one group of a list that has several;
/// a `Location` names WHERE THE WHOLE CARD IS, the second level of the hierarchy
/// whose first level is the kind's own title. Every lens every
/// shipping picker offers today groups into exactly ONE section, whose label is
/// character-for-character the lens's own — so on every one of them that line is
/// a location, and drawing it as list chrome was what made it read as a repeat
/// of the title rather than as the level below it.
///
/// Built by the grouped/faceted geometry owner from the parallel section labels
/// and handed to [`plan_overlay_rows`] as the line sequence; the plan turns it
/// into geometry, and the shaper reads the same sequence for its glyphs.
#[derive(Clone)]
pub(in crate::render) enum PlanLine {
    Location(String),
    Header(String),
    Item(usize),
}

/// ONE PLANNED ROW — the whole truth about display line `display`.
///
/// `top`/`height` are the row's SLOT in canvas px: the band the selected-row fill
/// paints, the band the text clip admits, and the band a pointer must fall inside
/// to select `item`. `item` is `None` for a section header (nothing to select).
///
/// `dx`/`dw` are the row's own HORIZONTAL EXTENT relative to the content
/// band's own span `[card_x, card_x + card_w]` — how far this row's whole
/// composition (its glyph origin, its plate, its selected band, and the
/// region a pointer must be inside to select it) is stepped in from either
/// edge: the row's drawn/clickable span is `[card_x + dx, card_x + card_w +
/// dw]`. Both are `0.0` on every row of every shipping world today, which is
/// why the whole pair is a no-op there. They exist because a row composition
/// that staggers rows horizontally is a real, already-drawable thing (the
/// wild-menu slant probe draws one), and before `dx` existed the offset was
/// applied by the DRAW emitters alone while `row_at` kept testing the card's
/// undisplaced x-span — so a staggered row was clickable where it was not
/// drawn and not clickable where it was. DESIGN.md §8: drawn geometry and
/// hit-test geometry have one owner.
///
/// ONE OFFSET COULD NOT EXPRESS THE MIRROR. Mangrove's descending
/// `\` composition steps each successive row's LEFT edge further right, right
/// edge flush (`dx > 0`, `dw == 0`) — the shape `dx` alone already described.
/// Magpie's ascending `/` composition, with clusters right-aligned, steps each
/// row's RIGHT edge further left, left edge flush (`dx == 0`, `dw < 0`) — the
/// mirror image, which a single left-anchored offset cannot represent (moving
/// `dx` negative would shift the row's origin left of the band, not narrow it
/// from the right). `dw` is that second, independent edge: `[left_inset,
/// right_inset]` and `dx`+`dw` are the same shape either way (an inset from
/// each edge) — this crate keeps the existing `dx` name and adds `dw` as a
/// width DELTA (`card_w + dw` is the row's own width) rather than a second
/// `right_inset` magnitude, because every consumer already manipulates
/// `(x, width)` pairs (`slant_bar_span`, `OverlayBarLayout::span`, the Pane
/// band rect) — `dw` slots straight into the width term with no unit
/// conversion at any call site, where a `right_inset` would need `card_w -
/// right_inset` re-derived everywhere `width` is read.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::render) struct PlannedRow {
    pub display: usize,
    pub item: Option<usize>,
    pub top: f32,
    pub height: f32,
    pub dx: f32,
    pub dw: f32,
}

impl PlannedRow {
    pub(in crate::render) fn bottom(&self) -> f32 {
        self.top + self.height
    }
}

/// The measured, already-resolved inputs a row plan is derived from. Every field
/// is produced by a stage that has already run: the card box by the placement
/// policy, `lh`/`header_gap` by the overlay metric owners, the window by the
/// scroll owner, `lines` by the grouped-plan owner.
pub(in crate::render) struct OverlayRowPlanInput<'a> {
    pub card_x: f32,
    pub card_w: f32,
    pub text_top: f32,
    pub lh: f32,
    pub header_gap: f32,
    pub header_rows: usize,
    /// How many of `header_rows`' own `lh` this band bills — a docked strip
    /// occupies a box but draws outside the panel, charging no `lh`.
    pub billed_header_rows: usize,
    /// Candidate display lines the FLAT family shows. Ignored when `lines` is
    /// `Some` (the grouped family's own line count is authoritative).
    pub visible: usize,
    pub top_idx: usize,
    pub n_items: usize,
    pub selected: usize,
    /// `1` when the card shows an empty-state notice instead of rows, else `0`.
    /// It occupies a display line below the candidate band, so the footer starts
    /// one row lower.
    pub empty_rows: usize,
    /// The GROUPED family's explicit display-line sequence, or `None` for a flat
    /// window.
    pub lines: Option<&'a [PlanLine]>,
    /// The row composition's HORIZONTAL STEP, SIGNED, in canvas px per display
    /// row. `0.0` for every shipping world (an upright list) — every planned
    /// `dx`/`dw` is then `0.0` and the whole feature is byte-identical. Positive
    /// plans a growing `dx` (left edge steps in, right edge flush — Mangrove's
    /// shape); negative plans a growing-negative `dw` (right edge steps in, left
    /// edge flush — Magpie's mirror). See [`PlannedRow`]'s doc for why one signed
    /// input is enough: no named consumer needs both edges to move independently
    /// at once, so this field stays the one owner of the composition's step
    /// rather than shipping a second, unread axis.
    pub dx_per_row: f32,
    /// A measured diagonal cluster may replace the probe/default stagger with
    /// its exact attachment-side row span. `None` keeps the historical (and
    /// env-probe) step above, so every unassigned world stays inert.
    pub cluster_span: Option<RowSpan>,
    /// A selected diagonal row steps its whole cluster outward. This remains
    /// planned geometry so its text, control, selection connector, and hit-test
    /// share the same two-sided span.
    pub selected_offset: Option<(f32, f32)>,
    pub selected_display: Option<usize>,
    /// `1` when the geometry owner reserved a line for the "items hidden
    /// above" count cue, else `0` — never more than one: the cue names how
    /// many are hidden, it does not enumerate them. Seats the cue's own
    /// display line directly above candidate row 0 by pushing `first_top`
    /// down by exactly this many `lh`, so drawn glyphs and planned hit-test
    /// geometry (which reads `first_top` too) cannot disagree about where
    /// row 0 actually starts.
    pub cue_above_rows: usize,
    /// The count cue's bottom-edge counterpart: `1` when a line was reserved
    /// directly below the last candidate row, else `0`. Folded into
    /// [`OverlayRowPlan::content_rows`] so the footer band seats below it
    /// exactly as it already does for the empty-state notice.
    pub cue_below_rows: usize,
}

/// THE PLANNED CANDIDATE BAND. Built once per overlay frame; read by the draw
/// emitters, the pointer hit-test, and the sidecar report.
#[derive(Clone, Debug)]
pub(in crate::render) struct OverlayRowPlan {
    pub(super) card_x: f32,
    pub(super) card_w: f32,
    pub(super) text_top: f32,
    pub(super) header_gap: f32,
    pub(super) first_top: f32,
    pub(super) lh: f32,
    pub(super) headers: Vec<PlannedHeader>,
    pub(super) rows: Vec<PlannedRow>,
    /// The billed header count this plan was built with (see
    /// [`OverlayRowPlanInput::billed_header_rows`]), stored so a second
    /// buffer on the same band leads with this many empties, not the box count.
    pub(super) billed_header_rows: usize,
    pub(super) empty_rows: usize,
    pub(super) selected_display: Option<usize>,
    /// The composition's own signed step, constant for the frame — so completing
    /// this plan's extent needs only the MEASURED half.
    pub(super) dx_per_row: f32,
    /// Mirrors [`OverlayRowPlanInput::cue_above_rows`] / `cue_below_rows` —
    /// stored so [`OverlayRowPlan::content_rows`]/`footer_top` and the cue's
    /// own draw position can read them back without a second input.
    pub(super) cue_above_rows: usize,
    pub(super) cue_below_rows: usize,
}

/// PLAN WORK WITNESSES, counted by the planner itself so no consumer can dodge
/// them: plans built, and `PlannedRow`s across those plans. Their ratio is the
/// O(visible) claim, checkable at runtime — a planner that started walking the
/// corpus would blow the per-plan mean while the frame time stayed flat, which is
/// exactly how a bench "measures" work that never happened. Read by
/// `--bench-suite`'s palette cell; never by the render or capture paths, so no
/// frame's output depends on them.
static PLANS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PLANNED_ROWS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// `(plans built, rows planned)` since process start.
pub(in crate::render) fn plan_witness() -> (u64, u64) {
    use std::sync::atomic::Ordering::Relaxed;
    (PLANS.load(Relaxed), PLANNED_ROWS.load(Relaxed))
}

/// THE FORWARD ROW-Y ARITHMETIC. Deliberately PRIVATE to this module: the whole
/// point of the plan is that a consumer cannot re-derive a row's y from loose
/// scalars, only read it off the plan that drew it.
fn row_top(text_top: f32, header_rows: usize, header_gap: f32, row: usize, lh: f32) -> f32 {
    text_top
        + super::overlay_header::header_band_height(header_rows, lh, header_gap)
        + row as f32 * lh
}

/// TEST-ONLY: the planned top of candidate display row `row` for a card with
/// these header metrics. It BUILDS A REAL PLAN and reads the row's slot off it,
/// so a law written against synthetic numbers still measures the one owner rather
/// than its own copy of the arithmetic — the shape several `render/tests` laws
/// used to carry inline.
#[cfg(test)]
pub(in crate::render) fn test_row_top(
    text_top: f32,
    header_rows: usize,
    header_gap: f32,
    row: usize,
    lh: f32,
) -> f32 {
    let plan = plan_overlay_rows(&OverlayRowPlanInput {
        card_x: 0.0,
        card_w: 0.0,
        text_top,
        lh,
        header_gap,
        header_rows,
        billed_header_rows: header_rows,
        visible: row + 1,
        top_idx: 0,
        n_items: row + 1,
        selected: 0,
        empty_rows: 0,
        lines: None,
        dx_per_row: 0.0,
        cluster_span: None,
        selected_offset: None,
        selected_display: None,
        cue_above_rows: 0,
        cue_below_rows: 0,
    });
    plan.row_top(row).expect("row is inside the planned window")
}

/// TEST-ONLY: a REAL plan built from loose header metrics, for a law whose
/// subject is the HEADER band rather than a rendered card — the shape the
/// `overlay_split_bounds` / `overlay_secondary_top` unit laws used to carry as
/// free functions taking the same scalars. Two candidate rows, so the band the
/// header sits above genuinely exists.
#[cfg(test)]
pub(in crate::render) fn test_header_plan(
    text_top: f32,
    header_rows: usize,
    header_gap: f32,
    lh: f32,
) -> OverlayRowPlan {
    plan_overlay_rows(&OverlayRowPlanInput {
        card_x: 0.0,
        card_w: 0.0,
        text_top,
        lh,
        header_gap,
        header_rows,
        billed_header_rows: header_rows,
        visible: 2,
        top_idx: 0,
        n_items: 2,
        selected: 0,
        empty_rows: 0,
        lines: None,
        dx_per_row: 0.0,
        cluster_span: None,
        selected_offset: None,
        selected_display: None,
        cue_above_rows: 0,
        cue_below_rows: 0,
    })
}

/// TEST-ONLY: `n` planned rows of pitch `lh` seated at `text_top` with no header,
/// built by the REAL planner — for a law whose subject is a row band rather than a
/// card (the living band's own coverage sweep).
#[cfg(test)]
pub(in crate::render) fn test_rows(text_top: f32, lh: f32, n: usize) -> Vec<PlannedRow> {
    plan_overlay_rows(&OverlayRowPlanInput {
        card_x: 0.0,
        card_w: 0.0,
        text_top,
        lh,
        header_gap: 0.0,
        header_rows: 0,
        billed_header_rows: 0,
        visible: n,
        top_idx: 0,
        n_items: n,
        selected: 0,
        empty_rows: 0,
        lines: None,
        dx_per_row: 0.0,
        cluster_span: None,
        selected_offset: None,
        selected_display: None,
        cue_above_rows: 0,
        cue_below_rows: 0,
    })
    .rows()
    .to_vec()
}

/// Build the plan. Pure: no clock, no randomness, no device, no allocation per
/// item — one [`PlannedRow`] per DISPLAY LINE the card shows.
pub(in crate::render) fn plan_overlay_rows(input: &OverlayRowPlanInput<'_>) -> OverlayRowPlan {
    // Where the HEADER band itself closes — unmoved by the count cue, so the
    // query field/lens strip's own boxes (`plan_header_band`, below) stay
    // exactly where they always sat. The candidate band's `first_top` seats
    // one `lh` PAST that close for every reserved `cue_above_rows`, opening
    // the exact slot the "items hidden above" cue text draws into and
    // pushing every real candidate row down with it — the one shift both the
    // drawn glyphs (which follow this same line count, sequentially) and this
    // plan's own hit-test/selected-band geometry read, so they cannot disagree
    // about where row 0 starts.
    let header_close = row_top(
        input.text_top,
        input.billed_header_rows,
        input.header_gap,
        0,
        input.lh,
    );
    let first_top = header_close + input.cue_above_rows as f32 * input.lh;
    let headers = super::overlay_header::plan_header_band(input);
    let mut rows: Vec<PlannedRow> = match input.lines {
        Some(lines) => lines
            .iter()
            .enumerate()
            .map(|(display, line)| PlannedRow {
                display,
                item: match line {
                    PlanLine::Item(i) => Some(*i),
                    PlanLine::Location(_) | PlanLine::Header(_) => None,
                },
                top: first_top + display as f32 * input.lh,
                height: input.lh,
                dx: 0.0,
                dw: 0.0,
            })
            .collect(),
        None => (0..input.visible)
            .map(|display| {
                let idx = input.top_idx + display;
                PlannedRow {
                    display,
                    item: (idx < input.n_items).then_some(idx),
                    top: first_top + display as f32 * input.lh,
                    height: input.lh,
                    dx: 0.0,
                    dw: 0.0,
                }
            })
            .collect(),
    };
    apply_row_extent(&mut rows, &RowExtent::of(input));
    // THE LOGICAL SELECTED DISPLAY LINE — the row Enter or a click activates.
    // Two families: a grouped plan's selected item sits at its POSITION in the
    // line sequence (headers push it down); a flat window's is its offset in the
    // window, saturated and clamped defensively so a transient list-shrink can
    // never over/underflow. NOT "which row looks selected" — that is the
    // visual-selection transaction's answer (`overlay_visual_sel`), which reads
    // this as its target.
    //
    // `rows.is_empty()` is its own `None` case, checked before either
    // branch — a GROUPED card whose own chrome overhead already exceeds
    // `avail_px` (`fit_item_rows`'s `min_items: 0` floor) plans NO display
    // lines at all, item or header, even though `n_items > 0`. Both branches'
    // old fallbacks (`unwrap_or(0)` / `saturating_sub(1)` on a zero `visible`)
    // used to read as "selection scrolled out of an otherwise real window" and
    // silently answered `Some(0)` for a window that has no row 0 — dead code
    // before this item (the shared floor never let a window go empty), now a
    // real state this plan must describe honestly.
    let selected_display = if input.n_items == 0 || rows.is_empty() {
        None
    } else if input.lines.is_some() {
        Some(
            rows.iter()
                .position(|r| r.item == Some(input.selected))
                .unwrap_or(0),
        )
    } else {
        Some(
            input
                .selected
                .saturating_sub(input.top_idx)
                .min(input.visible.saturating_sub(1)),
        )
    };
    use std::sync::atomic::Ordering::Relaxed;
    PLANS.fetch_add(1, Relaxed);
    PLANNED_ROWS.fetch_add(rows.len() as u64, Relaxed);
    OverlayRowPlan {
        card_x: input.card_x,
        card_w: input.card_w,
        text_top: input.text_top,
        header_gap: input.header_gap,
        first_top,
        lh: input.lh,
        headers,
        rows,
        billed_header_rows: input.billed_header_rows,
        empty_rows: input.empty_rows,
        selected_display,
        dx_per_row: input.dx_per_row,
        cue_above_rows: input.cue_above_rows,
        cue_below_rows: input.cue_below_rows,
    }
}
