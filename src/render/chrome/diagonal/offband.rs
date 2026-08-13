//! THE CARD'S CHROME OFF THE ROW BAND — the HEAD band above it and the FOOT band
//! below it.
//!
//! The foot band's own X lives here: the card's hint line, and the tips band beneath
//! it on the one kind that carries one, placed on the spine the rows above them hang
//! on instead of holding the card's left edge.
//!
//! The HEAD band's x is still the card's own text edge — a query FIELD is an input,
//! and right-aligning one on a mirrored composition would make its sigil travel as
//! the user types. What lives here for the head band is therefore not a placement but
//! a MEASUREMENT ([`TextPipeline::overlay_head_band_ink`]): the box its drawn ink
//! occupies, which the footprint frost's own shape has to contain. The two answers
//! are in one module because they are the same question asked of the same two bands —
//! does this chrome rake with the rows, and if not, where is it?
//!
//! # The shape, and why it is neither a second run nor a second buffer
//!
//! Every line of a card — query, rows, hint, tips — is ONE rich-text run in ONE
//! `panel_buffer`, so a line has no x of its own to set. The mechanism that already
//! gives the leaning ROWS independent x's is neither of the two obvious answers:
//! `overlay_upload_text` emits SEVERAL `TextArea`s over that one buffer, each
//! vertically clipped to a band and each seated at its own `left`, and glyphon
//! composites them into one batch. The foot band was already one of those areas —
//! the tail, `clip(band_bottom, canvas)` — and simply took `text_left` because no
//! one had asked it for anything else. So this module is that area's missing x, and
//! it adds no run, no buffer, and no line to the emitter.
//!
//! Placing the tail as ONE block is deliberate. The band is the hint, the invisible
//! separator above it, and (Keybindings alone) a tips list — chrome, not a list, so
//! it takes the composition's line at its own y rather than raking line by line.
//!
//! # Every quantity here is READ, none is authored
//!
//! The lean comes from [`DiagonalClusterRail`], which resolved it from the card the
//! frame actually drew; the band's y and ink width come from the shaped buffer's own
//! runs. A second reading of `ROW_STEP` would part company with the drawn spine on
//! exactly the cramped card where `TRAVEL_MAX_BAND_FRACTION` makes the composition
//! give up rake, and nothing in a roomy card's capture could see it.

use super::*;

/// **THE OPEN DESIGN QUESTION, AND WHICH ANSWER IS DRAWN.** At the foot band's own
/// row the spine has ENDED — it spans the names above it — so the foot either
/// CONTINUES the lean past that terminus onto the line the spine would have drawn,
/// or SITS at the terminal x the last row hangs on.
///
/// `true` continues it, and is what ships pending the user's call:
///
/// * The rake is one property of the whole card, not a rule that stops at the last
///   name. Freezing at the terminus states a second rule the composition never
///   states anywhere else.
/// * At the terminal x the hint's left edge lands exactly on the LAST ROW's own,
///   which reads as one more row of the list — the reading the blank separator
///   above the hint exists to break.
/// * It makes the frost's shape honest. A footprint frost leans by a CONSTANT shear
///   about the card's centre (`blur::extent::Footprint`), so a foot on the
///   extrapolated line is a foot on the parallelogram's own edge; a foot frozen at
///   the terminus sits inboard of it, covered today only by the attachment inset's
///   slack.
///
/// Switching to the terminus is this one word, and the two are captured side by side
/// in the untracked gallery output.
pub(in crate::render) const FOOT_CONTINUES_THE_LEAN: bool = true;

/// THE FOOT BAND'S PLACEMENT, as the draw seats it and as a law grades it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct FootPlacement {
    /// The `TextArea` origin the tail band is emitted at — clamped into the text
    /// column, so this is what was DRAWN rather than what the spine asked for.
    pub left: f32,
    /// The composition anchor the band hangs on, BEFORE the column clamp: the
    /// quantity that must sit on the line through the drawn rows' own anchors.
    pub anchor: f32,
    /// The band's widest shaped ink, and the centre of the line carrying it — both
    /// read off the buffer this frame shaped.
    pub ink_w: f32,
    pub center_y: f32,
    /// How many display steps below row 0 that line sits, in row pitches. Fractional
    /// and measured: the separator and the hint are both compact rows.
    pub steps: f32,
    /// Whether the COLUMN CLAMP moved the band off the seat the composition asked
    /// for. A law's own question: a clamp no sweep reaches is a clamp no law has run,
    /// and this is the one path that can silently eat the whole lean.
    pub clamped: bool,
}

impl TextPipeline {
    /// WHICH SHAPED `panel_buffer` LINE IS THE FOOT HINT — found by its TEXT, never
    /// by row-count arithmetic. Every alternative index would have to re-derive the
    /// shaper's own stacking, and that stacking already drifts from the row budget in
    /// one place: the separator line ahead of the hint is drawn unconditionally while
    /// `overlay_hint_gap_rows` can drop the row it reserves for it. Asking the buffer
    /// what it holds cannot drift.
    pub(in crate::render) fn overlay_hint_line(&self) -> Option<usize> {
        if self.overlay_hint.is_empty() {
            return None;
        }
        self.panel_buffer.lines.iter().position(|line| {
            let drawn = line.text();
            drawn == self.overlay_hint
                || (!drawn.is_empty()
                    && self.overlay_hint.ends_with(drawn)
                    && self.overlay_hint[..self.overlay_hint.len() - drawn.len()]
                        .ends_with(crate::overlay::HINT_SEP))
        })
    }

    /// A SHAPED `panel_buffer` LINE'S OWN RUN — `(ink width, line box top, line box
    /// height)`, the last two in canvas space. `None` when this frame shaped no such
    /// line. One owner, because both bands read it and a second copy would be a second
    /// answer to where a line's ink is.
    fn overlay_line_ink(&self, geom: &OverlayGeom, line: usize) -> Option<(f32, f32, f32)> {
        self.panel_buffer.layout_runs().find_map(|run| {
            (run.line_i == line).then_some((
                run.line_w,
                geom.text_top + run.line_top,
                run.line_height,
            ))
        })
    }

    /// THE HEAD BAND'S DRAWN INK BOX, `[left, top, right, bottom]` in canvas px — the
    /// card's UPRIGHT chrome, and the whole of what the footprint frost's leaning shape
    /// cannot cover by raking.
    ///
    /// Every header line — the query field, and the grouped family's lens strip under it
    /// — rides ONE `TextArea` seated at `geom.text_left`, so the box is that edge, the
    /// widest of their shaped runs, and the vertical run from the first line's box top to
    /// the last line's bottom. The width is taken over EVERY planned header line rather
    /// than the two that have names today, so a third enrols by existing.
    ///
    /// `None` when the card plans no header line at all (the contextual spell popup),
    /// which is exactly when there is no upright chrome for a shape to contain.
    pub(in crate::render) fn overlay_head_band_ink(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        let mut ink_w = 0.0f32;
        let mut top = f32::INFINITY;
        let mut bottom = f32::NEG_INFINITY;
        for line in plan.header_lines() {
            let Some((w, t, h)) = self.overlay_line_ink(geom, line.line) else {
                continue;
            };
            ink_w = ink_w.max(w);
            top = top.min(t);
            bottom = bottom.max(t + h);
        }
        (ink_w > 0.0 && top.is_finite() && bottom > top).then_some([
            geom.text_left,
            top,
            geom.text_left + ink_w,
            bottom,
        ])
    }

    /// WHERE THIS FRAME'S FOOT CHROME IS SEATED, or `None` when it is seated at the
    /// card's own text edge exactly as it always has been.
    ///
    /// `None` is the inert answer and it is reached three ways, none of them a name:
    /// a composition with no spine (every upright world — no cluster, so nothing to
    /// hang on), a card whose row band is EMPTY (the empty-state notice shares this
    /// band, and a lean read off zero rows would indent it by the attachment inset
    /// with no spine drawn beside it), and a card with no hint shaped.
    pub(in crate::render) fn overlay_foot_placement(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<FootPlacement> {
        let cluster = self.diagonal_cluster?;
        let first_row = plan.rows().first()?;
        let last = plan.rows().len().saturating_sub(1);
        let (ink_w, top, height) = self.overlay_line_ink(geom, self.overlay_hint_line()?)?;
        let pitch = plan.lh();
        let center_y = top + height * 0.5;
        if !pitch.is_finite() || pitch <= 0.0 || !center_y.is_finite() {
            return None;
        }
        // THE STEP COUNT, MEASURED: the drawn vertical distance from row 0's own
        // centre to the hint's, in row pitches. The rows are one pitch apart by
        // construction, so this is the same axis `spine_step` steps along.
        let measured = (center_y - (first_row.top + first_row.height * 0.5)) / pitch;
        let steps = if FOOT_CONTINUES_THE_LEAN {
            measured
        } else {
            last as f32
        };
        let anchor = cluster.foot_anchor(steps);
        let (left, right) = cluster.foot_span(steps, ink_w);
        // THE COLUMN CLAMP, against the WHOLE band's widest ink rather than the
        // hint's own: the hint decides where the band hangs, but a tips line beneath
        // it rides the same area and must not be pushed into the emitter's clip. So
        // the composition gives up the lean exactly as far as it must, and a band too
        // wide for the column keeps the card's edge — the historical placement, bit
        // for bit.
        let band_w = self
            .overlay_footer_content_px(geom, plan.content_rows())
            .max(ink_w);
        let head = geom.text_left;
        let tail = head + (geom.text_w - band_w).max(0.0);
        let seated = left.clamp(head, tail);
        Some(FootPlacement {
            left: seated,
            clamped: seated != left,
            anchor,
            ink_w: right - left,
            center_y,
            steps,
        })
    }

    /// The x the tail band's `TextArea` is emitted at — the one call the emitter
    /// makes, and `geom.text_left` unchanged wherever [`Self::overlay_foot_placement`]
    /// is inert.
    pub(in crate::render) fn overlay_foot_left(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> f32 {
        self.overlay_foot_placement(geom, plan)
            .map_or(geom.text_left, |foot| foot.left)
    }
}
