//! `ListStyle::Rules` — the QUIET fourth list composition, and a PROTOTYPE.
//!
//! Organised by ABSENCE: leading and hairline rules do the arranging and
//! nothing is drawn as an object. `Pane` organises by enclosure, `Bars` by
//! objects, `Diagonal` by a drawn line the rows hang off; this one has no
//! figure at all, only the boundaries between rows and the air around them.
//!
//! It needed no new plumbing and no new pass. `ListBacking::BarePlates` with
//! `draws_row_plates()` FALSE is exactly `Diagonal`'s configuration, so the
//! "no card, no plate" seam already existed; the rules themselves ride the two
//! quad pipelines `Bars` already fills — hairlines on `overlay_bars`, the
//! selection mark on `overlay_rows`.
//!
//! ⚠️ **WHAT IS UNDECIDED IS THE POINT.** [`theme::RuleSelection`] carries two
//! credible answers to "which row is selected" and the choice between them is
//! taste, owed to a human rather than settled here. Both are drawn from this
//! one function so the fork stays one decision in one place, and both are
//! forceable for capture (`AWL_OVERLAY_LIST_FORCE=rules[:weight|gutter]`).
//! Neither may FILL the row: a filled band is `Pane`'s answer, and borrowing it
//! would make this style a restyle of that one.

use super::overlay_selection::OverlaySelectionRects;
use super::*;

// ---------------------------------------------------------------------------
// THE COMPOSITION'S OWN LENGTHS
//
// Authored as [`Logical`] like every other chrome length, so the composition
// survives a Retina scale and a text zoom rather than shipping at half its
// tuned size on the display the captures were never taken on.
// ---------------------------------------------------------------------------

/// The ordinary separating rule. A hairline is the thinnest mark the style is
/// allowed: any heavier and a rule starts reading as the edge of a surface.
pub(in crate::render::chrome) const RULE_HAIRLINE: Logical = Logical(1.0);

/// A rule carrying the selection. Three times the hairline is the smallest step
/// that survives a 1x capture as a DIFFERENT weight rather than as a darker one.
pub(in crate::render::chrome) const RULE_SELECTED_WEIGHT: Logical = Logical(3.0);

/// The air a rule needs above and below it, folded into the row pitch through
/// the one [`TextPipeline::overlay_row_gap`] owner — the same seam `Bars` uses
/// for the space between its plates. Whitespace is half of what this style
/// organises with, so this is a composition dial, not padding.
pub(in crate::render::chrome) const RULE_ROW_AIR: Logical = Logical(8.0);

/// The row text's inset from the card's layout bound. Wider than `Pane`'s pad
/// because the extra room IS the gutter — the column a `Gutter` mark hangs in,
/// and the margin a `Weight` rule runs out into.
pub(in crate::render::chrome) const RULES_TEXT_HPAD: Logical = Logical(30.0);

/// The gutter mark's length, and its gap from the text's own left edge.
pub(in crate::render::chrome) const RULE_MARK_LEN: Logical = Logical(13.0);
pub(in crate::render::chrome) const RULE_MARK_GAP: Logical = Logical(9.0);

impl TextPipeline {
    /// THE `Rules` PROTOTYPE'S ROW SURFACES — which are not surfaces at all.
    ///
    /// `unselected` carries the separating hairlines and `selected` the
    /// selection mark, so the two existing quad pipelines draw them with no new
    /// pass; what makes this style itself is that neither list ever contains a
    /// rect as tall as a row. Both selection treatments are emitted from here so
    /// the taste fork stays one decision in one place.
    pub(super) fn overlay_rules_selection(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
        mark: theme::RuleSelection,
    ) -> OverlaySelectionRects {
        // A rule is a drawn line, not a rounded surface — square ends, no
        // stroke. Set explicitly because both pipelines are shared with `Bars`
        // and would otherwise carry its corner across a world switch.
        self.overlay_bars.set_corner(0.0);
        self.overlay_bars.set_stroke(0.0);
        self.overlay_rows.set_corner(0.0);
        self.overlay_rows.set_stroke(0.0);
        self.overlay_bars.set_color(theme::faint().rgba_bytes());
        // The mark is the page's own ink at full strength — figure/ground by
        // value, never a second accent (DESIGN §3). It dims with the rest of an
        // unfocused workspace region, through the one owner that does so.
        let mark_ink = match geom.workspace && !geom.rows_focused {
            true => super::workspace::dimmed(
                theme::base_content(),
                super::workspace::UNFOCUSED_MARK_ALPHA,
            ),
            false => theme::base_content().rgba_bytes(),
        };
        self.overlay_rows.set_color(mark_ink);

        let hair = self.metrics.px(RULE_HAIRLINE).max(1.0);
        let heavy = self.metrics.px(RULE_SELECTED_WEIGHT).max(hair);
        // The MEASURE an ordinary rule runs — the text column, so a rule and the
        // label above it start and stop together. The heavy rule runs the whole
        // card band instead, and that difference in REACH is half of what
        // `Weight` says.
        let (measure_x, measure_w) = (geom.text_left, geom.text_w.max(1.0));
        let (band_x, band_w) = (geom.band_x(), geom.band_w().max(1.0));

        // The air is split evenly above and below a row's glyphs (the row-pitch
        // owner folds it in whole), so the BOUNDARY between two rows is the row
        // slot's own edge. A rule straddles it rather than hanging off it.
        let rule = |y: f32, weight: f32, x: f32, w: f32| [x, y - weight * 0.5, w, weight];

        let content: Vec<&PlannedRow> = plan.rows().iter().filter(|r| r.item.is_some()).collect();
        let mut selected = Vec::new();
        let mut unselected = Vec::new();
        // Which rows have claimed the boundaries they touch. A claimed boundary
        // is not ALSO drawn as a hairline: the heavy rule replaces it, rather
        // than being laid over one.
        let claimed = |row: &PlannedRow| match mark {
            theme::RuleSelection::Weight => vis.reads_selected(row.display),
            theme::RuleSelection::Gutter => false,
        };
        for (i, row) in content.iter().enumerate() {
            let prev_claims = i.checked_sub(1).is_some_and(|j| claimed(content[j]));
            let next_claims = content.get(i + 1).is_some_and(|r| claimed(r));
            // The list is CLOSED by rules top and bottom rather than by a card
            // edge. Each row draws the boundary BELOW it; only the first draws
            // the one above, so no boundary is emitted twice.
            if i == 0 && !(claimed(row) || prev_claims) {
                unselected.push(rule(row.top, hair, measure_x, measure_w));
            }
            if !(claimed(row) || next_claims) {
                unselected.push(rule(row.bottom(), hair, measure_x, measure_w));
            }
            if !claimed(row) {
                continue;
            }
            // A selected row's own two rules. The upper one belongs to the
            // previous row's slot as well, so it is skipped when that row is
            // itself selected (a live glide can read two rows at once).
            if !prev_claims {
                selected.push(rule(row.top, heavy, band_x, band_w));
            }
            selected.push(rule(row.bottom(), heavy, band_x, band_w));
        }
        if let theme::RuleSelection::Gutter = mark {
            // The mark is made of the same substance as the list — a heavy rule
            // segment, one gutter wide, hanging beside a row nothing else about
            // has changed.
            let len = self.metrics.px(RULE_MARK_LEN);
            let gap = self.metrics.px(RULE_MARK_GAP);
            for row in content.iter().filter(|r| vis.reads_selected(r.display)) {
                let cy = row.top + row.height * 0.5;
                selected.push(rule(cy, heavy, measure_x - gap - len, len));
            }
        }
        OverlaySelectionRects {
            selected,
            unselected,
            cross: Vec::new(),
        }
    }
}
