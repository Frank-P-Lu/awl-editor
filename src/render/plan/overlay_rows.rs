//! The planned candidate-row band of a summoned overlay card.
//!
//! ONE plan per overlay frame answers every question anything downstream may ask
//! about a candidate row: where it is drawn, what item it means, which row a
//! pointer lands on, which row the state selects, and where the footer band
//! begins. Both card families are planned by the same code — the FLAT window
//! (`items[top_idx .. top_idx + visible]`, one display line each) and the GROUPED
//! window (an explicit [`PlanLine`] sequence whose section headers push the item
//! rows down) — so "which display line is item 3" has exactly one answer.

/// One DISPLAY line in an overlay card's candidate area: a faint uppercase
/// section header, or a candidate row carrying its index into `overlay_items`.
///
/// Built by the grouped/faceted geometry owner from the parallel section labels
/// and handed to [`plan_overlay_rows`] as the line sequence; the plan turns it
/// into geometry, and the shaper reads the same sequence for its glyphs.
#[derive(Clone)]
pub(in crate::render) enum PlanLine {
    Header(String),
    Item(usize),
}

/// ONE PLANNED ROW — the whole truth about display line `display`.
///
/// `top`/`height` are the row's SLOT in canvas px: the band the selected-row fill
/// paints, the band the text clip admits, and the band a pointer must fall inside
/// to select `item`. `item` is `None` for a section header (nothing to select).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::render) struct PlannedRow {
    pub display: usize,
    pub item: Option<usize>,
    pub top: f32,
    pub height: f32,
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
}

/// THE PLANNED CANDIDATE BAND. Built once per overlay frame; read by the draw
/// emitters, the pointer hit-test, and the sidecar report.
#[derive(Clone, Debug)]
pub(in crate::render) struct OverlayRowPlan {
    card_x: f32,
    card_w: f32,
    first_top: f32,
    lh: f32,
    rows: Vec<PlannedRow>,
    empty_rows: usize,
    selected_display: Option<usize>,
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
/// point of item 174 is that a consumer cannot re-derive a row's y from loose
/// scalars, only read it off the plan that drew it.
fn row_top(text_top: f32, header_rows: usize, header_gap: f32, row: usize, lh: f32) -> f32 {
    text_top + header_rows as f32 * lh + header_gap + row as f32 * lh
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
        visible: row + 1,
        top_idx: 0,
        n_items: row + 1,
        selected: 0,
        empty_rows: 0,
        lines: None,
    });
    plan.row_top(row).expect("row is inside the planned window")
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
        visible: n,
        top_idx: 0,
        n_items: n,
        selected: 0,
        empty_rows: 0,
        lines: None,
    })
    .rows()
    .to_vec()
}

/// Build the plan. Pure: no clock, no randomness, no device, no allocation per
/// item — one [`PlannedRow`] per DISPLAY LINE the card shows.
pub(in crate::render) fn plan_overlay_rows(input: &OverlayRowPlanInput<'_>) -> OverlayRowPlan {
    let first_top = row_top(
        input.text_top,
        input.header_rows,
        input.header_gap,
        0,
        input.lh,
    );
    let rows: Vec<PlannedRow> = match input.lines {
        Some(lines) => lines
            .iter()
            .enumerate()
            .map(|(display, line)| PlannedRow {
                display,
                item: match line {
                    PlanLine::Item(i) => Some(*i),
                    PlanLine::Header(_) => None,
                },
                top: first_top + display as f32 * input.lh,
                height: input.lh,
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
                }
            })
            .collect(),
    };
    // THE LOGICAL SELECTED DISPLAY LINE — the row Enter or a click activates.
    // Two families: a grouped plan's selected item sits at its POSITION in the
    // line sequence (headers push it down); a flat window's is its offset in the
    // window, saturated and clamped defensively so a transient list-shrink can
    // never over/underflow. NOT "which row looks selected" — that is the
    // visual-selection transaction's answer (`overlay_visual_sel`), which reads
    // this as its target.
    let selected_display = if input.n_items == 0 {
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
        first_top,
        lh: input.lh,
        rows,
        empty_rows: input.empty_rows,
        selected_display,
    }
}

impl OverlayRowPlan {
    /// Every planned candidate display row, ascending. The draw path iterates
    /// THIS rather than counting rows itself.
    pub(in crate::render) fn rows(&self) -> &[PlannedRow] {
        &self.rows
    }

    /// The planned top of display row `display`, or `None` past the band. The
    /// only door to a row's y.
    pub(in crate::render) fn row_top(&self, display: usize) -> Option<f32> {
        self.rows.get(display).map(|r| r.top)
    }

    /// The `overlay_items` index drawn at display row `display`, or `None` when
    /// that line is a section header or past the band.
    pub(in crate::render) fn item_at(&self, display: usize) -> Option<usize> {
        self.rows.get(display).and_then(|r| r.item)
    }

    /// Candidate display lines the card draws (headers included).
    pub(in crate::render) fn candidate_rows(&self) -> usize {
        self.rows.len()
    }

    /// Display lines the card's CONTENT occupies — the candidate band plus the
    /// empty-state notice, which is where the footer band begins. One owner, so
    /// the footer plate can no longer sit a row above its own glyphs on a card
    /// that shows a notice.
    pub(in crate::render) fn content_rows(&self) -> usize {
        self.rows.len() + self.empty_rows
    }

    pub(in crate::render) fn first_top(&self) -> f32 {
        self.first_top
    }

    pub(in crate::render) fn lh(&self) -> f32 {
        self.lh
    }

    /// The canvas y just past the LAST planned candidate row — the band's own
    /// bottom edge, equal to the last row's `bottom()` when the card has rows.
    pub(in crate::render) fn band_bottom(&self) -> f32 {
        self.first_top + self.rows.len() as f32 * self.lh
    }

    /// The canvas y the footer band (foot hint + keybinding tips) starts at.
    pub(in crate::render) fn footer_top(&self) -> f32 {
        self.first_top + self.content_rows() as f32 * self.lh
    }

    pub(in crate::render) fn selected_display(&self) -> Option<usize> {
        self.selected_display
    }

    /// THE INVERSE — the display line a pointer at canvas `py` falls in,
    /// ignoring the card's horizontal bounds. `None` above, below, or outside the
    /// planned band.
    ///
    /// It SCANS THE PLANNED SLOTS rather than inverting the forward formula. The
    /// scan is exact where a division is not: `(py - first_top) / lh` truncated at
    /// a row's own top edge lands on the row ABOVE whenever `k * lh` is not
    /// representable (the pure law caught `lh = 12`, `header_gap = 63.55`, row 11 —
    /// a pointer on row 11's first pixel selecting row 10). Comparing against the
    /// same `f32` `top` values the band was DRAWN from cannot disagree with the
    /// draw by construction, which is the whole point of planning the rows. It is
    /// O(visible) over a band that is at most a couple of dozen rows.
    pub(in crate::render) fn display_at(&self, py: f32) -> Option<usize> {
        if self.lh <= 0.0 {
            return None;
        }
        self.rows
            .iter()
            .position(|r| py >= r.top && py < r.bottom())
    }

    /// THE INTERACTION GEOMETRY — the `overlay_items` index a pointer at
    /// `(px, py)` selects, or `None` off the card, above the band, on a section
    /// header, or below the last planned row. The inverse of the same
    /// [`PlannedRow`] slots the band is drawn from, so a click cannot land on a
    /// row other than the one under the pointer.
    pub(in crate::render) fn row_at(&self, px: f32, py: f32) -> Option<usize> {
        if px < self.card_x || px > self.card_x + self.card_w {
            return None;
        }
        self.item_at(self.display_at(py)?)
    }

    /// The card's own horizontal bounds, as the hit-test reads them.
    #[cfg(test)]
    pub(in crate::render) fn card_x_span(&self) -> (f32, f32) {
        (self.card_x, self.card_x + self.card_w)
    }
}
