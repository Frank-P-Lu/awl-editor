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
        theme::ListStyle::Bars { .. } => true,
        theme::ListStyle::Pane => true,
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
            return VisualSelection::default();
        };
        let Some(target) = plan.row_top(sel) else {
            return VisualSelection::default();
        };
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
                    self.living_band_rects(force, from, to, t, geom.card_x, geom.card_w, lh);
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
        let rows = livingband::covered_rows(&bands, plan.first_top(), lh, plan.candidate_rows());
        VisualSelection {
            logical,
            band_top,
            living,
            rows,
        }
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
        let rows_of = |buf: &GlyphBuffer, want: Option<glyphon::Color>| -> Vec<usize> {
            let Some(want) = want else {
                return Vec::new();
            };
            let mut out = Vec::new();
            for run in buf.layout_runs() {
                if run.line_i < geom.header_rows {
                    continue;
                }
                let row = run.line_i - geom.header_rows;
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
            rows_of(&self.panel_buffer, primary_flip),
            rows_of(&self.panel_bind_buffer, secondary_flip),
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
