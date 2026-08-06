//! `ListStyle::Rules` — the QUIET fourth list composition.
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
//! [`rules_ink`] IS THE ONE OWNER of which rules a ruled list draws, and it is
//! shared rather than copied: the picker/workspace ROW list and the summoned
//! workspace's navigation RAIL are the same composition over different bands, on
//! different pipelines. A rail is a list of labels arranged by the boundaries
//! between them, which is what this style says a list is — and taking the
//! world's filled selected-row band there, as every other style's rail does,
//! would put a plate inside the one composition that refuses them.
//!
//! [`theme::RuleSelection`] carries the selection treatment. `Weight` is the
//! shipped answer, chosen by the user against both rendered side by side: the
//! two rules bounding the selected row thicken and run out past the text measure
//! to the full band, so the mark is made of the list's own substance and the
//! row's interior stays plain ground. `Gutter` — a short heavy segment hanging
//! in the margin — remains drawable, and both come out of one function so the
//! fork stays one decision in one place
//! (`AWL_OVERLAY_LIST_FORCE=rules[:weight|gutter]`). Neither may FILL the row: a
//! filled band is `Pane`'s answer, and borrowing it would make this style a
//! restyle of that one.

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

/// ONE ROW'S VERTICAL SLOT in a ruled list — the two boundaries it touches, and
/// whether it is the one the selection reads. Deliberately not a `PlannedRow`:
/// the workspace's navigation rail is a ruled list too and its entries never go
/// through the row planner, so the composition owner below takes the only three
/// facts it actually needs.
#[derive(Clone, Copy)]
pub(in crate::render) struct RuleRow {
    pub top: f32,
    pub bottom: f32,
    pub selected: bool,
}

/// THE SPANS AND WEIGHTS a ruled list is drawn at, resolved to device pixels by
/// the caller (each list knows its own band).
pub(in crate::render) struct RuleSpans {
    pub hair: f32,
    pub heavy: f32,
    /// What an ordinary rule runs — the list's text measure, so a rule and the
    /// label above it start and stop together.
    pub measure: (f32, f32),
    /// What a selected row's thickened rule runs instead. The difference in
    /// REACH between this and `measure` is half of what `Weight` says.
    pub band: (f32, f32),
    /// `Gutter`'s mark: its length, and its gap from `measure`'s left edge.
    pub mark: (f32, f32),
}

/// THE ONE OWNER OF WHICH RULES A RULED LIST DRAWS — `(hairlines, selection)`.
///
/// Both consumers come through here: the picker/workspace ROW list
/// ([`TextPipeline::overlay_rules_selection`]) and the summoned workspace's
/// navigation RAIL ([`TextPipeline::prepare_rail_rules`]). They render on
/// different pipelines over different bands, and that is all that differs — a
/// rail is a list of labels arranged by the boundaries between them, which is
/// what this style says a list is.
pub(in crate::render) fn rules_ink(
    rows: &[RuleRow],
    mark: theme::RuleSelection,
    s: &RuleSpans,
) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
    // The air is split evenly above and below a row's glyphs (the row-pitch
    // owner folds it in whole), so the BOUNDARY between two rows is the row
    // slot's own edge. A rule straddles it rather than hanging off it.
    let rule = |y: f32, weight: f32, x: f32, w: f32| [x, y - weight * 0.5, w, weight];
    let (measure_x, measure_w) = (s.measure.0, s.measure.1.max(1.0));
    let (band_x, band_w) = (s.band.0, s.band.1.max(1.0));
    let mut selected = Vec::new();
    let mut hairlines = Vec::new();
    // Which rows have claimed the boundaries they touch. A claimed boundary is
    // not ALSO drawn as a hairline: the heavy rule replaces it, rather than
    // being laid over one.
    let claimed = |row: &RuleRow| match mark {
        theme::RuleSelection::Weight => row.selected,
        theme::RuleSelection::Gutter => false,
    };
    for (i, row) in rows.iter().enumerate() {
        let prev_claims = i.checked_sub(1).is_some_and(|j| claimed(&rows[j]));
        let next_claims = rows.get(i + 1).is_some_and(claimed);
        // The list is CLOSED by rules top and bottom rather than by a card edge.
        // Each row draws the boundary BELOW it; only the first draws the one
        // above, so no boundary is emitted twice.
        if i == 0 && !(claimed(row) || prev_claims) {
            hairlines.push(rule(row.top, s.hair, measure_x, measure_w));
        }
        if !(claimed(row) || next_claims) {
            hairlines.push(rule(row.bottom, s.hair, measure_x, measure_w));
        }
        if !claimed(row) {
            continue;
        }
        // A selected row's own two rules. The upper one belongs to the previous
        // row's slot as well, so it is skipped when that row is itself selected
        // (a live glide can read two rows at once).
        if !prev_claims {
            selected.push(rule(row.top, s.heavy, band_x, band_w));
        }
        selected.push(rule(row.bottom, s.heavy, band_x, band_w));
    }
    if let theme::RuleSelection::Gutter = mark {
        // The mark is made of the same substance as the list — a heavy rule
        // segment, one gutter wide, hanging beside a row nothing else about has
        // changed.
        //
        // THE GUTTER IS THE BAND'S, NOT THE MARK'S. A card whose text inset is
        // narrower than `gap + len` has less gutter than the authored segment
        // wants, and the contextual SPELL popup is exactly that card (its inset
        // is its own `SPELL_PAD`, not the list style's). The segment shortens
        // into the room it has rather than hanging off the card's left edge —
        // an authored length yields to the band, never the other way round.
        let (len, gap) = s.mark;
        let x = (measure_x - gap - len).max(band_x);
        let w = (measure_x - gap - x).clamp(1.0, len);
        for row in rows.iter().filter(|r| r.selected) {
            let cy = (row.top + row.bottom) * 0.5;
            selected.push(rule(cy, s.heavy, x, w));
        }
    }
    (hairlines, selected)
}

impl TextPipeline {
    /// The two rule weights this frame draws them at. A hairline is floored at
    /// one device pixel so it survives every scale, and the heavy rule can never
    /// be lighter than the hairline it replaces.
    pub(in crate::render) fn rule_weights(&self) -> (f32, f32) {
        let hair = self.metrics.px(RULE_HAIRLINE).max(1.0);
        (hair, self.metrics.px(RULE_SELECTED_WEIGHT).max(hair))
    }

    /// The MARK's ink for this frame: the page's own content ink at full
    /// strength — figure/ground by value, never a second accent (DESIGN §3) —
    /// dimmed with the rest of an unfocused workspace region through the one
    /// owner that does that.
    pub(in crate::render::chrome) fn rule_mark_ink(&self, dim: bool) -> [u8; 4] {
        match dim {
            true => super::workspace::dimmed(
                theme::base_content(),
                super::workspace::UNFOCUSED_MARK_ALPHA,
            ),
            false => theme::base_content().rgba_bytes(),
        }
    }

    /// THE `Rules` STYLE'S ROW SURFACES — which are not surfaces at all.
    ///
    /// `unselected` carries the separating hairlines and `selected` the
    /// selection mark, so the two existing quad pipelines draw them with no new
    /// pass; what makes this style itself is that neither list ever contains a
    /// rect as tall as a row.
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
        self.overlay_rows
            .set_color(self.rule_mark_ink(geom.workspace && !geom.rows_focused));

        let (hair, heavy) = self.rule_weights();
        let rows: Vec<RuleRow> = plan
            .rows()
            .iter()
            .filter(|r| r.item.is_some())
            .map(|r| RuleRow {
                top: r.top,
                bottom: r.bottom(),
                selected: vis.reads_selected(r.display),
            })
            .collect();
        let (unselected, selected) = rules_ink(
            &rows,
            mark,
            &RuleSpans {
                hair,
                heavy,
                measure: (geom.text_left, geom.text_w),
                band: (geom.band_x(), geom.band_w()),
                mark: (
                    self.metrics.px(RULE_MARK_LEN),
                    self.metrics.px(RULE_MARK_GAP),
                ),
            },
        );
        OverlaySelectionRects {
            selected,
            unselected,
            cross: Vec::new(),
        }
    }
}
