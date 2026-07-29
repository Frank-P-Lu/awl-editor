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
//! shapers, the band/plate emitters, the rail ink, and the sidecar all consume
//! the resulting [`VisualSelection`]; nothing downstream calls a band animator or
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

/// THE ONE answer to "which overlay display rows currently READ as selected".
///
/// Resolved once per frame by [`TextPipeline::resolve_visual_selection`] and
/// threaded to every consumer, so the band, the primary ink, the secondary ink,
/// the accessory plates and the sidecar cannot disagree within a frame.
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
    ) -> VisualSelection {
        let logical = self.overlay_selected_display_line(geom);
        let lh = self.overlay_lh();
        let Some(sel) = logical else {
            return VisualSelection::default();
        };
        let target = overlay_row_top(geom.text_top, geom.header_rows, geom.header_gap, sel, lh);
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
        let first_top = overlay_row_top(geom.text_top, geom.header_rows, geom.header_gap, 0, lh);
        let rows = livingband::covered_rows(&bands, first_top, lh, geom.visible);
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
                if row >= geom.visible {
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
}
