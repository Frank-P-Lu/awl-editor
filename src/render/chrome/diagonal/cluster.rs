//! THE MEASURED ROW CLUSTER — a diagonal row's `name + gap + accessory`, and
//! where each of the three sits beside the spine.
//!
//! The cluster is the unit that MIRRORS. Its name hangs on the spine end and its
//! accessory on the outer end, each growing back toward the other, so a
//! descending world and its ascending mirror image differ by one signed answer
//! ([`ColumnFlow`]) rather than by two independently maintained layouts. Its
//! WIDTH is the card's own budget: sized from the rows in front of it, a scroll
//! moved the whole composition sideways.

use super::*;

/// THE ONE MIRROR — which way a row's NAME grows off its spine. Every other
/// direction in the composition is derived from this one: the accessory column
/// is its [`ColumnFlow::mirrored`], and the cluster's two ends follow the same
/// sign, so label, gap and accessory mirror as a unit.
pub(super) fn label_flow_of(direction: theme::DiagonalDirection) -> ColumnFlow {
    match direction {
        theme::DiagonalDirection::Descending => ColumnFlow::Rightward,
        theme::DiagonalDirection::Ascending => ColumnFlow::Leftward,
    }
}

/// The one measured row-cluster layout shared by diagonal text, accessory
/// upload, Range geometry, and the planner's clickable row-side span.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct DiagonalClusterRail {
    direction: theme::DiagonalDirection,
    /// The row's whole territory beside the spine — the cluster BUDGET, a
    /// property of the card alone. The name hangs on the SPINE end and the
    /// accessory on the OUTER end, each growing back toward the other, as an
    /// upright world's name and chord share one text column. Sized from the
    /// rows, a scroll moved it.
    cluster_w: f32,
    accessory_w: f32,
    connector: f32,
    /// The selected mark's own gap and reach, resolved once with every other
    /// length. Held here rather than asked of the composition again so
    /// [`DiagonalClusterRail::mark_span`] is the ONE place the mark's abscissae
    /// exist, and the rail a law reads is the rail the draw used.
    mark_gap: f32,
    mark_reach: f32,
    spine_start: f32,
    spine_step: f32,
    span: RowSpan,
    selected_display: Option<usize>,
    selected_shift: f32,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(in crate::render) struct DiagonalClusterProbe {
    pub cluster_w: f32,
    pub accessory_w: f32,
    pub span: RowSpan,
    rail: DiagonalClusterRail,
}

#[cfg(test)]
impl DiagonalClusterProbe {
    /// The rail as a LAW reads it: its measured extents beside the same rail the
    /// draw used, so no probe can answer from numbers the frame did not have.
    pub(super) fn of(rail: DiagonalClusterRail) -> Self {
        Self {
            cluster_w: rail.cluster_w,
            accessory_w: rail.accessory_w,
            span: rail.span,
            rail,
        }
    }

    /// The cluster's two ends, spine-side first — the pair every law about the
    /// mirror reads, because naming them left/right would be true of one world.
    pub(in crate::render) fn label_anchor(self, display: usize) -> f32 {
        self.rail.label_anchor(display)
    }

    pub(in crate::render) fn accessory_anchor(self, display: usize) -> f32 {
        self.rail.accessory_anchor(display)
    }

    pub(in crate::render) fn label_origin(self, display: usize, ink_w: f32) -> f32 {
        self.rail.label_origin(display, ink_w)
    }

    pub(in crate::render) fn accessory_span(self, display: usize) -> (f32, f32) {
        self.rail.accessory_span(display)
    }

    pub(in crate::render) fn cluster_span(self, display: usize) -> (f32, f32) {
        self.rail.cluster_span(display)
    }

    pub(in crate::render) fn label_flow(self) -> ColumnFlow {
        self.rail.label_flow()
    }

    pub(in crate::render) fn selected_offset(self) -> (f32, f32) {
        self.rail.selected_offset()
    }

    /// Where display row `display`'s SPINE segment stands — the composition's
    /// stationary surface, independent of anything a row measures.
    pub(in crate::render) fn spine_x(self, display: usize) -> f32 {
        self.rail.spine_x(display)
    }

    /// The selected mark's `(vertex_x, arm_x)` off the same rail the draw read
    /// it from — never a law's own re-derivation of the outward sign.
    pub(in crate::render) fn mark_span(self, display: usize) -> (f32, f32) {
        self.rail.mark_span(display)
    }

    /// The row's own MEASURED horizontal step — see
    /// [`DiagonalClusterRail::spine_step`]'s own doc.
    pub(in crate::render) fn spine_step(self) -> f32 {
        self.rail.spine_step()
    }
}

impl DiagonalClusterRail {
    pub(super) fn new(
        composition: DiagonalComposition,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        selected_display: Option<usize>,
        cluster_w: f32,
        accessory_w: f32,
    ) -> Self {
        let band_x = geom.band_x();
        let band_right = band_x + geom.band_w();
        let cluster_w = cluster_w.max(0.0);
        let accessory_w = accessory_w.max(0.0).min(cluster_w);
        let rows = plan.rows().len().saturating_sub(1) as f32;
        let inset = attachment_inset(composition, geom);
        // THE SPINE IS A FIXED SURFACE-RELATIVE LINE: its travel is reserved off
        // the card's own side territory, never off the rows in front of it, and
        // the cluster elides into what is left.
        let step = if rows > 0.0 {
            spine_travel(composition, geom, plan.rows().len()) / rows
        } else {
            0.0
        };
        let (spine_start, spine_step, span) = match composition.direction {
            theme::DiagonalDirection::Descending => (
                band_x + inset,
                step,
                RowSpan {
                    dx: inset,
                    dw: 0.0,
                    dx_per_row: step,
                    dw_per_row: 0.0,
                },
            ),
            theme::DiagonalDirection::Ascending => (
                band_right - inset,
                -step,
                RowSpan {
                    dx: 0.0,
                    dw: -inset,
                    dx_per_row: 0.0,
                    dw_per_row: -step,
                },
            ),
        };
        Self {
            direction: composition.direction,
            cluster_w,
            accessory_w,
            connector: composition.connector,
            mark_gap: composition.mark_gap,
            mark_reach: composition.mark_reach,
            spine_start,
            spine_step,
            span,
            selected_display,
            selected_shift: composition.selected_outward,
        }
    }

    pub(in crate::render) fn span(self) -> RowSpan {
        self.span
    }

    pub(super) fn spine_x(self, display: usize) -> f32 {
        self.spine_at(display as f32)
    }

    /// THE ONE OWNER OF THE SPINE'S ABSCISSA, at a FRACTIONAL number of display
    /// steps from row 0. A row asks at a whole step; the foot chrome below the row
    /// band asks at the real vertical distance it sits at, in row pitches, which is
    /// not a whole number — the hint's own row and the separator above it are both
    /// compact. Both answers are the same line because both read the DRAWN
    /// `spine_step`, which already carries a cramped card's yielded rake
    /// ([`TRAVEL_MAX_BAND_FRACTION`]).
    fn spine_at(self, steps: f32) -> f32 {
        self.spine_start + self.spine_step * steps
    }

    /// WHERE CHROME BELOW THE ROW BAND HANGS OFF THE COMPOSITION — the foot band's
    /// own anchor, `steps` display steps down from row 0 on the same spine the rows
    /// hang on, and its `(left, right)` ink span for ink `w` wide.
    ///
    /// It hangs on the same [`Self::label_flow`] end a NAME does, so on a mirrored
    /// world the foot's ink ends on the spine exactly as a name's does instead of
    /// crossing it. It carries NO selected shift: the foot is not a row, and a
    /// selection landing on the last row must not drag the hint sideways with it.
    pub(in crate::render) fn foot_span(self, steps: f32, w: f32) -> (f32, f32) {
        self.label_flow().span(self.foot_anchor(steps), w)
    }

    /// The foot band's anchor alone — the abscissa a law compares against the line
    /// through the drawn rows' own anchors.
    pub(in crate::render) fn foot_anchor(self, steps: f32) -> f32 {
        self.spine_at(steps) + self.connector * self.outward()
    }

    fn shift(self, display: usize) -> f32 {
        let shift = if self.selected_display == Some(display) {
            self.selected_shift
        } else {
            0.0
        };
        shift * self.outward()
    }

    pub(in crate::render) fn selected_offset(self) -> (f32, f32) {
        let shift = self.selected_shift * self.outward();
        (shift, shift)
    }

    pub(in crate::render) fn row_plan(
        self,
    ) -> (Option<RowSpan>, Option<(f32, f32)>, Option<usize>) {
        (
            Some(self.span()),
            Some(self.selected_offset()),
            self.selected_display,
        )
    }

    pub(super) fn spine(self, plan: &OverlayRowPlan) -> Option<([f32; 2], [f32; 2])> {
        let first = plan.rows().first()?;
        let last = plan.rows().last()?;
        Some((
            [self.spine_x(first.display), first.top + first.height * 0.5],
            [self.spine_x(last.display), last.top + last.height * 0.5],
        ))
    }

    /// WHICH WAY THE WHOLE CLUSTER MIRRORS, in one signed answer. The NAME hangs
    /// on the spine end at both orientations — left-aligned off a descending
    /// spine, right-aligned against an ascending one — and the accessory hangs on
    /// the outer end, growing back toward it. Mirroring is
    /// [`ColumnFlow::mirrored`] applied once, here, so label, gap and accessory
    /// move as one unit and cannot half-mirror.
    pub(in crate::render) fn label_flow(self) -> ColumnFlow {
        label_flow_of(self.direction)
    }

    /// The draw asks [`accessory_flow`] — the world's own answer, needed before a
    /// cluster exists — so this is the LAW-facing door onto the same rule.
    #[cfg(test)]
    pub(in crate::render) fn accessory_flow(self) -> ColumnFlow {
        self.label_flow().mirrored()
    }

    /// The cluster's SPINE end — the edge every row's name hugs, one connector
    /// out from the spine, and the reach the selected row's mark opens to.
    pub(in crate::render) fn label_anchor(self, display: usize) -> f32 {
        let spine = self.spine_x(display) + self.shift(display);
        spine + self.connector * self.outward()
    }

    /// The cluster's OUTER end — the card-edge side, where the accessory column
    /// hangs and grows back inward toward the name.
    pub(in crate::render) fn accessory_anchor(self, display: usize) -> f32 {
        self.label_anchor(display) + self.cluster_w * self.outward()
    }

    /// THE SELECTED ROW'S MARK, as `(vertex_x, arm_x)` — its row-facing end and
    /// its arm line — standing on the row's OUTER edge, away from the spine.
    ///
    /// THE SIDE IS NOT CHOSEN HERE. It falls out of [`Self::outward`], the one
    /// signed dial this whole cluster mirrors on: `accessory_anchor` is already
    /// the cluster's card-edge end because that same sign put it there, and the
    /// mark simply continues one gap further along it. So the mark cannot end up
    /// on the spine side of a row without the row's own name and accessory
    /// swapping ends first — which is what makes "the mark mirrors with the
    /// cluster" true by construction rather than by a second per-world branch.
    ///
    /// The VERTEX is the inner end, so the mark points back into the row it
    /// marks; the arms open outward, into the card's own margin. The selected
    /// row's outward shift arrives free, carried by `accessory_anchor`.
    pub(in crate::render) fn mark_span(self, display: usize) -> (f32, f32) {
        let vertex = self.accessory_anchor(display) + self.mark_gap * self.outward();
        (vertex, vertex + self.mark_reach * 2.0 * self.outward())
    }

    /// THE ONE SIGNED DIAL the whole cluster mirrors on — the direction "away
    /// from the spine" points in canvas x. Every end, shift and mark abscissa
    /// above is this sign times a length, which is why the mirror cannot
    /// half-apply.
    fn outward(self) -> f32 {
        self.direction.sign()
    }

    /// Where a name `ink_w` wide BEGINS — a text area's origin, and the one
    /// question the draw asks about the name's placement.
    pub(in crate::render) fn label_origin(self, display: usize, ink_w: f32) -> f32 {
        self.label_flow().origin(self.label_anchor(display), ink_w)
    }

    /// The accessory column's own `(left, right)` box — the pair a law needs to
    /// bound a rail; the draw seats the column from its anchor and its flow.
    #[cfg(test)]
    pub(in crate::render) fn accessory_span(self, display: usize) -> (f32, f32) {
        self.accessory_flow()
            .span(self.accessory_anchor(display), self.accessory_w)
    }

    /// The row's whole territory as `(left, right)` — the two cluster ends in
    /// canvas order, whichever way it mirrors. A containment claim's own shape:
    /// nothing in the draw path needs the pair, only the end it hangs on.
    #[cfg(test)]
    pub(in crate::render) fn cluster_span(self, display: usize) -> (f32, f32) {
        let (a, b) = (self.label_anchor(display), self.accessory_anchor(display));
        (a.min(b), a.max(b))
    }

    pub(in crate::render) fn accessory_w(self) -> f32 {
        self.accessory_w
    }

    /// The row's own MEASURED horizontal step (device px, signed) —
    /// `DiagonalComposition::row_step` narrowed by [`spine_travel`]'s
    /// [`TRAVEL_MAX_BAND_FRACTION`] yield on a card too tight to afford the
    /// authored step outright. A location cue that reads along the spine's
    /// own rake reads THIS, not the authored constant, so a narrow card's
    /// flattened spine and the cue beside it can never disagree.
    pub(in crate::render) fn spine_step(self) -> f32 {
        self.spine_step
    }
}
