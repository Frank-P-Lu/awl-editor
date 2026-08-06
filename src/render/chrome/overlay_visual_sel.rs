//! src/render/chrome/overlay_visual_sel.rs — THE VISUAL-SELECTION TRANSACTION.
//!
//! ITEM 164. An overlay card answers "which row is selected?" with five separate
//! visuals: the selection BAND, the primary label's ink, the secondary
//! shortcut/value/git column's ink, a range row's rail thumb, and (under `Bars`)
//! the shortcut PLATE behind the chord. Before this module each of those decided
//! for itself, and two of them read DIFFERENT clocks:
//!
//! * the BAND is ANIMATED — the living morph (`render/livingband.rs`) on Pane,
//!   the `BandResponse::Slide` ease on Bars — so it lags the logical selection
//!   for one glide (~110ms) after every move;
//! * the primary ink rode the band (`covered_rows` — "INK RIDES THE BAND, NOT
//!   THE STATE", so a flipped glyph always has fill under it);
//! * the SECONDARY ink and the rail thumb read `overlay_selected_display_line`
//!   — the LOGICAL row — and recolored INSTANTLY.
//!
//! So for the whole glide the card showed two answers at once: the band and the
//! label on the row the pointer LEFT, the shortcut on the row it arrived at.
//! (The user's live Command-palette screenshot: the band sat on "Go to file…"
//! while only "Switch project…"'s shortcut had switched ink.)
//!
//! THE FIX — one transaction, resolved ONCE per overlay frame, read by every
//! selected visual. [`TextPipeline::resolve_visual_selection`] runs the band
//! animators exactly once, keeps the drawn geometry, and derives the rows that
//! READ as selected from that geometry — never from the logical index. The
//! shapers, the band/plate emitters and the rail ink all consume the resulting
//! [`VisualSelection`]; nothing downstream calls a band animator or
//! `overlay_selected_display_line` again.
//!
//! THE DIRECTION OF THE WAIT: the secondaries wait for the BAND, rather than the
//! band snapping ahead to the logical row. The band's lag is the authored voice
//! (the living band's whole point), and the ink flip exists so a glyph on the
//! fill stays legible — flipping ink onto a row the fill has not reached yet is
//! not merely incoherent, on an inverse-fill world it is invisible. So the
//! animated band is the clock and every other selected visual reads it.
//!
//! WHAT DOES *NOT* WAIT: [`VisualSelection::logical`] — the row the state selects
//! — is carried alongside and is what Enter and a click activate. The pointer
//! hit-test (`overlay_row_at`) is pure static row geometry and is deliberately
//! NOT in this transaction: a click must accept the row under the pointer on the
//! frame it lands, however far behind the band happens to be.

use super::*;
use crate::render::livingband::{self, BandRect, MotionForce};

/// Whether the secondary column flips onto the band at all. Both families do
/// today; the split is kept because it is a per-family question, not a global.
fn selected_secondary_on_band() -> bool {
    match crate::render::effective_list_style() {
        theme::ListStyle::Bars => true,
        theme::ListStyle::Pane | theme::ListStyle::Diagonal(_) => true,
        // The first `false` this predicate has ever returned, and the reason it
        // was kept as a per-family question: a `Rules` selection puts no fill
        // under the row at all, so there is no band for a secondary to flip
        // ONTO. Flipping anyway would recolour the shortcut against unchanged
        // ground — quieter or louder than the ink beside it for no reason a
        // reader can see.
        theme::ListStyle::Rules(_) => false,
    }
}

/// The PRIMARY label's on-band ink, or `None` when the world's band needs no
/// flip (the glyph keeps `base_content` and reads fine on the fill). ONE owner
/// so the shaper, the theme picker's own shaper, and the item-164 probe cannot
/// disagree about what "this row's label reads selected" looks like.
pub(in crate::render) fn overlay_selected_primary_ink() -> Option<glyphon::Color> {
    match theme::active().highlight_treatment(crate::render::effective_overlay_selrow_band()) {
        theme::HighlightTreatment::InverseFill { ink, .. } => Some(ink.to_glyphon()),
        theme::HighlightTreatment::ValueBand(band) => {
            let flipped = theme::selected_row_ink(band);
            (flipped != theme::base_content()).then(|| flipped.to_glyphon())
        }
    }
}

/// The SECONDARY column's on-band ink (the shortcut / time / git value beside a
/// name, and the range rail's thumb), or `None` when the band needs no flip.
/// The recessive twin of [`overlay_selected_primary_ink`], through the same
/// `theme::selected_row_secondary_ink` owner the rail already used — ONE
/// resolution shared by the shaped GLYPHS and the drawn rail QUAD, which
/// previously computed the same match arm twice.
pub(in crate::render) fn overlay_selected_secondary_srgb() -> Option<theme::Srgb> {
    if !selected_secondary_on_band() {
        return None;
    }
    match theme::active().highlight_treatment(crate::render::effective_overlay_selrow_band()) {
        theme::HighlightTreatment::InverseFill { ink, .. } => Some(ink),
        theme::HighlightTreatment::ValueBand(b) => {
            let flipped = theme::selected_row_secondary_ink(b);
            (flipped != theme::muted()).then_some(flipped)
        }
    }
}

/// [`overlay_selected_secondary_srgb`] as a text colour, for the shapers.
pub(in crate::render) fn overlay_selected_secondary_ink() -> Option<glyphon::Color> {
    overlay_selected_secondary_srgb().map(|c| c.to_glyphon())
}

/// ITEM 284 — THE DIAGONAL MARKER'S TRAVEL-DIRECTION SOURCE: which way the
/// selection just moved to land on this frame's selected row. `Down` for an
/// increasing display index, `Up` for a decreasing one — and a WRAP (last row
/// to first, or first to last) reads as whichever direction the raw index
/// delta is the SHORTER way round from, so a wrap continuing a held Down
/// still reads `Down` rather than flipping because the index fell (item 247's
/// own brief: "a wrap … takes the long way round", named there as the
/// in-flight glide's own distinction — the settled tilt below only needs the
/// two-way answer this carries).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) enum MarkerTravel {
    Down,
    Up,
}

impl MarkerTravel {
    /// The signed multiplier `chrome::diagonal::selected_chevron`'s `turn_deg`
    /// carries before the world's own mirror (`DiagonalDirection::sign`) is
    /// applied: `+1.0` for `Down`, `-1.0` for `Up`.
    pub(in crate::render) fn sign(self) -> f32 {
        match self {
            MarkerTravel::Down => 1.0,
            MarkerTravel::Up => -1.0,
        }
    }

    /// `prev -> next` among `total` display rows, wrap-aware: the direction
    /// whose step count (mod `total`) is the SMALLER of the two ways round the
    /// list. `None` for a no-op (`prev == next`) or a degenerate list (`total
    /// == 0`) — nothing travelled, so the marker's existing settled tilt
    /// stands. `prev`/`next` are trusted to be `< total`; an out-of-range
    /// `prev` (the candidate list reshaped under a live filter) is the
    /// caller's to guard, not this pure step.
    pub(in crate::render) fn of(prev: usize, next: usize, total: usize) -> Option<Self> {
        if prev == next || total == 0 {
            return None;
        }
        let total = total as i64;
        let forward = (next as i64 - prev as i64).rem_euclid(total);
        let backward = total - forward;
        Some(if forward <= backward {
            MarkerTravel::Down
        } else {
            MarkerTravel::Up
        })
    }
}

/// THE ONE answer to "which overlay display rows currently READ as selected".
///
/// Resolved once per frame by [`TextPipeline::resolve_visual_selection`] and
/// threaded to every consumer, so the band, the primary ink, the secondary ink
/// and the accessory plates cannot disagree within a frame.
#[derive(Clone, Debug, Default)]
pub(in crate::render) struct VisualSelection {
    logical: Option<usize>,
    band_top: Option<f32>,
    living: Option<(MotionForce, f32, f32, f32)>,
    rows: Vec<usize>,
    /// `Some` exactly when this frame's resolve saw the diagonal marker's
    /// selected row actually CHANGE from the previous frame's; `None` on a
    /// settled re-render (the marker keeps whatever tilt it already carries)
    /// and on every world that carries no marker at all. Read only by the
    /// `#[cfg(test)]` probe below — the PRODUCT reads the turn this drove
    /// (`Self::diagonal_marker_turn_deg`), never the direction that drove it.
    #[cfg_attr(not(test), allow(dead_code))]
    travel: Option<MarkerTravel>,
}

impl VisualSelection {
    /// The display row the STATE selects — what Enter or a click activates.
    /// `None` iff the card has no items. Deliberately NOT animated.
    pub(in crate::render) fn logical(&self) -> Option<usize> {
        self.logical
    }

    /// Whether display row `row` currently READS as selected — the one predicate
    /// every ink/plate consumer asks. Derived from the DRAWN band.
    pub(in crate::render) fn reads_selected(&self, row: usize) -> bool {
        self.rows.contains(&row)
    }

    /// Every display row that reads as selected this frame, ascending. Settled
    /// frames (every capture, Reduce Motion, an unarmed pipeline) carry exactly
    /// the logical row; only a live glide can carry none or two.
    pub(in crate::render) fn rows(&self) -> &[usize] {
        &self.rows
    }

    /// The row-top (canvas px) the selection band is DRAWN at this frame. Equal
    /// to the logical row's top on any settled frame.
    pub(in crate::render) fn band_top(&self) -> Option<f32> {
        self.band_top
    }

    /// The living-band travel and phase `(force, from, to, t)` when the Pane
    /// morph drives this frame — the band emitter's own input, resolved here so
    /// it can never re-run the animator against a second target.
    pub(in crate::render) fn living(&self) -> Option<(MotionForce, f32, f32, f32)> {
        self.living
    }

    /// ITEM 284 — which way the diagonal marker just travelled this frame, or
    /// `None` on a re-render that changed nothing. See [`MarkerTravel`]'s own
    /// doc; a probe seam for the law that grades this against the settled
    /// turn it drove.
    #[cfg(test)]
    pub(in crate::render) fn travel(&self) -> Option<MarkerTravel> {
        self.travel
    }
}

impl TextPipeline {
    /// THE ONE resolution point of the visual-selection transaction — called
    /// exactly once per overlay frame, from `prepare_overlay`, BEFORE any
    /// shaping or quad emission.
    ///
    /// It is the only place that runs a band animator (`living_band_phase` /
    /// `overlay_band_drawn`) and the only place that reads
    /// `overlay_selected_display_line` for a rendering decision; both are
    /// module-private and guarded by
    /// `render::tests::visual_selection_law`'s no-wildcard source sweep, so a
    /// new selected visual cannot grow its own second clock.
    ///
    /// The rows that READ as selected are `livingband::covered_rows` over the
    /// band's DRAWN rects: a row flips only once the fill majority-owns it. At
    /// rest that is exactly `[logical]` on both families, so every capture and
    /// every Reduce-Motion frame is byte-identical to the pre-transaction path.
    pub(in crate::render) fn resolve_visual_selection(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> VisualSelection {
        // ITEM 174 — the transaction's target is the PLANNED row's own top, and
        // the coverage grid is the plan's own band origin/pitch/length. The band
        // therefore starts from the same object the hit-test inverts.
        let logical = plan.selected_display();
        let lh = plan.lh();
        let Some(sel) = logical else {
            // No item to select: the diagonal marker's travel memory resets
            // rather than carrying a stale row into whatever this card next
            // shows selected.
            self.diagonal_marker_row = None;
            return VisualSelection::default();
        };
        let Some(target) = plan.row_top(sel) else {
            self.diagonal_marker_row = None;
            return VisualSelection::default();
        };
        let travel = self.resolve_diagonal_marker_travel(sel, plan.rows().len());
        // The Pane living band, when it is in play: its rects genuinely STRETCH
        // across rows mid-flight, so the covered set is read off the drawn quads.
        let living = matches!(
            crate::render::effective_list_style(),
            theme::ListStyle::Pane
        )
        .then(livingband::overlay_motion_force)
        .flatten()
        .map(|force| {
            let (from, to, t) = self.living_band_phase(force, target, lh);
            (force, from, to, t)
        });
        let (band_top, bands) = match living {
            Some((force, from, to, t)) => {
                let (primary, echo, _) =
                    self.living_band_rects(force, from, to, t, geom.band_x(), geom.band_w(), lh);
                let bands = primary
                    .iter()
                    .chain(echo.iter())
                    .map(|r| BandRect {
                        top: r[1],
                        height: r[3],
                    })
                    .collect::<Vec<_>>();
                (primary.first().map(|r| r[1]), bands)
            }
            // Every other band (the ordinary Pane band and the whole Bars family)
            // draws ONE row-tall shape at the eased top. Coverage is read against
            // the ROW SLOT (`lh`), never a `Bars` plate's gap-inset height — a
            // world whose `gap` exceeds half a row would otherwise majority-cover
            // nothing and leave a settled card with no row reading selected.
            None => {
                let top = self.overlay_band_drawn(target);
                (Some(top), vec![BandRect { top, height: lh }])
            }
        };
        let rows = livingband::covered_rows(&bands, plan.rows());
        // FLIGHT RECORDER / PROBE: the PREPARED-HIGHLIGHT-ENDPOINT link
        // of the event→present chain. This is the last place the selection is a
        // number and the first place it is geometry, so a `logical` that has moved
        // while `band_top`/`rows` have not is a STALE-RENDER break, distinct from a
        // state break upstream or a present break downstream.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!(
                "prepare_highlight logical={sel} target={target:.1} \
                 band_top={band_top:?} reads={rows:?}"
            ));
        }
        VisualSelection {
            logical,
            band_top,
            living,
            rows,
            travel,
        }
    }

    /// ITEM 284's DIRECTION SOURCE — compares this frame's selected display row
    /// `sel` against the row remembered from the PREVIOUS resolve
    /// (`self.diagonal_marker_row`), and — only on a world whose composition
    /// carries a marker at all — retargets `self.diagonal_marker_target` to
    /// the settled tilt the travel calls for
    /// (`MarkerTravel::sign` × [`chrome::diagonal::MARKER_TRAVEL_TILT_DEG`] ×
    /// the world's own mirror, `DiagonalDirection::sign` — so Mangrove's `\`
    /// and Magpie's `/` get mirrored turns from the SAME dial the cluster
    /// already mirrors on, never a second authored constant).
    ///
    /// An upright world, or the first row this card has selected since its
    /// travel memory was last reset (a fresh/reopened overlay), settles the
    /// marker at the un-turned baseline (`0.0`, item 247's shipped shape)
    /// rather than reporting a direction or carrying over a stale tilt.
    fn resolve_diagonal_marker_travel(&mut self, sel: usize, total: usize) -> Option<MarkerTravel> {
        let Some(composition) = super::diagonal::active(self) else {
            self.diagonal_marker_row = None;
            self.diagonal_marker_target = 0.0;
            self.diagonal_marker_turn = 0.0;
            return None;
        };
        let prev = self.diagonal_marker_row.replace(sel);
        let Some(prev) = prev else {
            self.diagonal_marker_target = 0.0;
            self.diagonal_marker_turn = 0.0;
            return None;
        };
        let travel = MarkerTravel::of(prev, sel, total);
        if let Some(t) = travel {
            self.diagonal_marker_target =
                t.sign() * super::diagonal::MARKER_TRAVEL_TILT_DEG * composition.direction.sign();
        }
        travel
    }

    /// TEST PROBE — what each selected visual ACTUALLY committed this frame,
    /// read back from the shaped buffers rather than recomputed: the display
    /// rows whose PRIMARY glyphs carry the flipped selected ink, and the rows
    /// whose SECONDARY (shortcut / value / git) glyphs carry theirs. The law
    /// that these two agree with each other and with the transaction is the
    /// whole point of item 164, and reading the committed glyph colours is the
    /// only oracle that cannot be satisfied by a parallel reimplementation.
    #[cfg(test)]
    pub(in crate::render) fn overlay_ink_flip_probe(
        &self,
        geom: &OverlayGeom,
    ) -> (Vec<usize>, Vec<usize>) {
        let candidate_rows = self.overlay_row_plan(geom).candidate_rows();
        let primary_flip = super::overlay_selected_primary_ink();
        let secondary_flip = super::overlay_selected_secondary_ink();
        // The PRIMARY buffer may carry the beat's own glyph-free line between the
        // header and the candidates (`OverlayGeom::shaped_first_row_line`); the
        // SECONDARY buffer is built from `right_bind_lines`' leading empties and
        // never does, so each is asked for its own first candidate line.
        let rows_of =
            |buf: &GlyphBuffer, first: usize, want: Option<glyphon::Color>| -> Vec<usize> {
                let Some(want) = want else {
                    return Vec::new();
                };
                let mut out = Vec::new();
                for run in buf.layout_runs() {
                    if run.line_i < first {
                        continue;
                    }
                    let row = run.line_i - first;
                    if row >= candidate_rows {
                        continue;
                    }
                    if run.glyphs.iter().any(|g| g.color_opt == Some(want)) && !out.contains(&row) {
                        out.push(row);
                    }
                }
                out.sort_unstable();
                out
            };
        (
            rows_of(
                &self.panel_buffer,
                geom.shaped_first_row_line(),
                primary_flip,
            ),
            rows_of(&self.panel_bind_buffer, geom.header_rows, secondary_flip),
        )
    }

    /// TEST PROBE — whether this frame actually GRANTED the secondary column
    /// (`rowlayout` yields it whole when a card is too narrow). A law about
    /// shortcut ink is vacuous on a card that drew no shortcuts.
    #[cfg(test)]
    pub(in crate::render) fn overlay_right_column_shown(&self) -> bool {
        self.overlay_right_shown
    }
}
