//! The location cue's shared preparation: a faceted picker's active-lens
//! name, painted through the rotated-label capability instead of the shared
//! inline row, for whichever world's `RenderCaps::location_style` asks for
//! it. TWO world expressions share this ONE owner rather than each getting a
//! code path (`CLAUDE.md`'s "a theme needing its own code path is wrong"):
//!
//! - Cassowary's `RotatedRail` — turned 90° along the room's own outer
//!   margin, the margin its wordmark placard already keeps, seated just ABOVE
//!   that placard at [`ROTATED_RAIL_PLACARD_FRACTION`] of its type size and in
//!   its ink: a vertical companion to the wordmark at its own scale class.
//! - Magpie's `Raked` — left where the row planner's own diagonal stagger
//!   already puts it (the row's own TEXT column, unmoved), turned to the
//!   diagonal spine's own rake, in a gradient between the spine's two
//!   authored tones.
//!
//! Only the flush edge, the bottom anchor, the fit box, the natural size, the
//! axis and the two colours differ between the two callers
//! (`render/chrome/rotated_location.rs`'s `prepare_overlay_rotated_location`,
//! which is the one place that decides them) — the mask compose, the caching,
//! and the shrink-to-fit solver are shared verbatim.

use super::*;
use crate::rotated_label::geometry::{InkBox, label_axis_deg, label_bounds};
use crate::rotated_label::mask::LabelMask;

/// The secondary heading's font size, as a fraction of the overlay's own UI
/// size — above `type_scale::LABEL` (a section header's whisper), below a
/// candidate row, so the hierarchy reads by size as well as by ink. Shared by
/// the treatments that size the cue against the CARD's own type: the inline
/// row (`chrome::theme_picker::shape_theme_spans`) and `Raked`'s in-column
/// rake. `RotatedRail` does NOT read it — it is chrome on the room's frame
/// rather than type in the card, and sizes against the placard instead
/// ([`ROTATED_RAIL_PLACARD_FRACTION`]).
pub(in crate::render) const LOCATION_SCALE: f32 = 0.92;

/// [`Overflow::Shrink`]'s FLOOR: never shrink a facet name past this fraction
/// of its natural size, even if the fit box would ask for less — a name long
/// enough to hit this floor is better slightly over its budget than
/// illegible.
pub(in crate::render) const ROTATED_LOCATION_MIN_SCALE: f32 = 0.55;

/// The fraction of `header_gap` `Raked`'s cue may treat as safely blank above
/// its row — see [`raked_along_budget`] for why it is a fraction rather than
/// the whole gap.
pub(in crate::render) const ROTATED_LOCATION_HEADER_GAP_FRAC: f32 = 0.55;

/// **`Raked`'s ALONG-AXIS BUDGET: its own row band, plus the share of the query
/// beat's calm divider that is really blank.** The one owner of that sum, and
/// the reason it takes the gap as an argument rather than reading it off the
/// geometry record is that the record's gap is not a header POSITION — a
/// consumer summing `geom.header_gap` to place a header line is re-deriving
/// planner arithmetic, which `render::tests::overlay_plan_law` forbids by name.
///
/// `header_gap` is the STRIP LINE's own box inflation, not the blank space
/// below its drawn pill: cosmic-text centres the pill's glyphs in that taller
/// box, so roughly as much of the gap sits ABOVE the pill (unusable — the query
/// field's own breathing room) as below it (the part this run may safely
/// enter). [`ROTATED_LOCATION_HEADER_GAP_FRAC`] is that share, measured against
/// the real shaped pill rather than derived from its own placement formula
/// (which would duplicate that private geometry here) — calibrated
/// conservative, confirmed against real captures at the longest facet name a
/// faceted picker can carry ("This folder", Go-to's own lens).
pub(in crate::render) fn raked_along_budget(row_height: f32, header_gap: f32) -> f32 {
    (row_height + header_gap.max(0.0) * ROTATED_LOCATION_HEADER_GAP_FRAC).max(1.0)
}

/// **THE ⅔ RELATION.** `RotatedRail`'s type size, as a fraction of the
/// wordmark placard's own — the whole of what makes the cue read as the
/// wordmark's companion rather than as a second title or as a caption. Two
/// thirds is a scale class down: unmistakably subordinate, unmistakably the
/// same voice. The placard's size is read from the placard's OWN owner
/// (`chrome::overlay_shape::overlay_shape_placard`) every frame, so the pair
/// cannot drift when the world redials its wordmark's loudness.
pub(in crate::render) const ROTATED_RAIL_PLACARD_FRACTION: f32 = 2.0 / 3.0;

/// The gap `RotatedRail` keeps between its own bottom ink and the placard's
/// LINE BOX, in ems of the cue's own type. On top of the leading the placard's
/// line box already carries above its capitals (roughly a fifth of its own
/// size), so the drawn gap reads wider than this number alone — deliberately,
/// since two runs of the same face at neighbouring scales need daylight
/// between them to read as a pair rather than as one broken line.
pub(in crate::render) const ROTATED_RAIL_PLACARD_GAP_EM: f32 = 0.12;

/// The placard's own LINE BOX per unit of font size
/// (`overlay_shape_placard` lays its wordmark out at `font_size * 1.1` on both
/// its main and its shrink-to-fit path). The rail is handed the placard's
/// drawn BOX and converts back through this, so [`ROTATED_RAIL_PLACARD_
/// FRACTION`] is a claim about the two runs' LETTERS rather than about a box.
/// Pinned by `render::tests::rotated_location`'s cap-ratio law: a
/// change to the placard's leading that this constant did not follow moves the
/// measured cap height out of the face's own band and fails there by name.
const PLACARD_LINE_BOX_RATIO: f32 = 1.1;

/// Slack (device px) allowed when asking whether the FINAL run cleared its fit
/// box — the shrink solver lands the footprint on the box exactly, so this
/// only absorbs float residue, never a design overflow.
const FIT_SLACK_PX: f32 = 0.5;

/// WHAT A RUN DOES when its natural size does not fit its box — the one place
/// the two world expressions genuinely want opposite answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) enum Overflow {
    /// SHRINK toward [`ROTATED_LOCATION_MIN_SCALE`], and draw the result even
    /// if it is still over budget at that floor. `Raked`'s answer: its box is a
    /// neighbouring row's breathing room, so a slight encroachment is the
    /// lesser evil next to an illegible cue or a missing one.
    Shrink,
    /// NEVER RESIZE — the size IS the composition. `RotatedRail`'s answer:
    /// [`ROTATED_RAIL_PLACARD_FRACTION`] of the wordmark is the whole point of
    /// the cue, its box is bounded by the card and the placard, and a run at
    /// some other fraction would be the small misplaced whisper the rail
    /// exists to replace. A run that does not fit is PARKED, so the cue is
    /// either the wordmark's companion at exactly its ⅔ or it is absent.
    Park,
}

/// WHICH EDGE of its screen footprint a rotated run seats against. `Left` is
/// the row's own text column for `Raked` and the room's left margin for a
/// `RotatedRail` world whose wordmark hugs that side; `Right` is that same
/// margin MIRRORED, for a wordmark hugging the other one — the cue keeps the
/// wordmark's margin, whichever margin that is, so a card anchored the other
/// way round can never find the cue on top of it.
#[derive(Clone, Copy, Debug)]
pub(in crate::render) enum FlushEdge {
    Left(f32),
    Right(f32),
}

/// EVERYTHING that differs between the two world expressions of the location
/// cue, decided by `prepare_overlay_rotated_location` and consumed by
/// [`TextPipeline::prepare_rotated_location_label`] — so the shared owner
/// below knows nothing about facets, plans, placards or diagonals.
pub(in crate::render) struct RotatedLabelPlacement {
    /// The edge the run's screen footprint seats flush against.
    pub flush: FlushEdge,
    /// The canvas y the run's BOTTOM ink edge lands on. Always bottom, never
    /// centred — [`rotated_location_origin`]'s own doc has the reasoning.
    pub bottom: f32,
    /// The largest screen footprint `[w, h]` (device px) the run may occupy.
    /// A non-finite component means unbounded on that axis.
    pub fit: [f32; 2],
    /// Font size (device px) before any shrink-to-fit.
    pub natural_size: f32,
    /// The protractor angle the run reads along (90° for the vertical rail;
    /// the diagonal spine's own rake for `Raked`).
    pub axis_deg: f32,
    /// The LINEAR colours the run's baseline gradient runs between; equal
    /// values give a flat run.
    pub color_a: [f32; 3],
    pub color_b: [f32; 3],
    /// What happens when the run does not fit [`Self::fit`].
    pub overflow: Overflow,
}

/// The placard's font size, from the drawn line BOX height its own owner
/// returns. One conversion, one place — see [`PLACARD_LINE_BOX_RATIO`].
pub(in crate::render) fn placard_font_size(placard_box_h: f32) -> f32 {
    placard_box_h / PLACARD_LINE_BOX_RATIO
}

/// Solve the rotated label's PEN ORIGIN so its screen footprint — measured
/// PURELY (no GPU): [`label_bounds`] at a trial origin of `[0, 0]`, then
/// shifted by the same delta every axis produces — seats flush against
/// `flush` and lands its BOTTOM edge on `bottom`. This function only ever
/// solves the offset; the caller's world chooses the edge and the anchor.
///
/// BOTTOM, never centred, in both expressions. `Raked` reuses a row band one
/// line tall while a facet name reading bottom-to-top is routinely several
/// lines' worth of glyph advance ("Settings", "Navigate" — longer than
/// "Files", the string this cue was first measured against): centring let a
/// long name's TOP half grow upward into the card's own open header gap
/// (harmless) while its BOTTOM half grew downward into the very next command
/// row's own plate — the crowding this cue exists to avoid. `RotatedRail`
/// anchors bottom for the composition's own sake: the run rises from just
/// above the wordmark, so every extra letter grows AWAY from it, into the
/// room's own empty upper margin, and the pair's one shared edge never moves.
pub(in crate::render) fn rotated_location_origin(
    flush: FlushEdge,
    bottom: f32,
    axis: [f32; 2],
    ink: InkBox,
) -> [f32; 2] {
    let raw = label_bounds([0.0, 0.0], axis, ink);
    let target_x = match flush {
        FlushEdge::Left(x) => x,
        FlushEdge::Right(x) => x - raw[2],
    };
    let target_y = bottom - raw[3];
    [target_x - raw[0], target_y - raw[1]]
}

/// The scale a run of screen footprint `footprint` must take to fit inside
/// `fit`, floored at [`ROTATED_LOCATION_MIN_SCALE`]: the TIGHTEST of the two
/// axes' ratios, and exactly `1.0` when the run already fits (so a caller
/// passing an unbounded box re-shapes nothing). Pure — the sweep over fit
/// boxes is unit-testable without a GPU.
pub(in crate::render) fn rotated_fit_shrink(footprint: [f32; 2], fit: [f32; 2]) -> f32 {
    let mut shrink = 1.0f32;
    for axis in 0..2 {
        let (have, room) = (footprint[axis], fit[axis]);
        if have > 0.0 && room.is_finite() && have > room {
            shrink = shrink.min((room / have).max(0.0));
        }
    }
    shrink.clamp(ROTATED_LOCATION_MIN_SCALE, 1.0)
}

impl TextPipeline {
    /// The location cue's own preparation, shared by both world expressions.
    /// Knows nothing about facets, plans, or worlds: `text` is whatever the
    /// caller decided to say and `p` is where, how big, how angled and in what
    /// ink ([`RotatedLabelPlacement`]'s own field docs). `text.is_empty()` (no
    /// location this frame — the All lens, or an `Inline` world that never
    /// calls this with real text) parks the pipeline, so a default frame pays
    /// nothing.
    ///
    /// THE RUN READS BOTTOM-TO-TOP, seated flush against `p.flush` with its
    /// bottom ink edge on `p.bottom`. A run that does not fit its box is
    /// shrunk toward it or parked, per `p.overflow` ([`Overflow`]'s own doc has
    /// which world wants which and why). Nothing is ever clipped.
    /// The mask is cached against the FINAL shaped run's own physical glyphs
    /// ([`LabelMask::matches`]), so holding one lens across many frames
    /// re-uploads no texture.
    pub(in crate::render) fn prepare_rotated_location_label(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        text: &str,
        p: &RotatedLabelPlacement,
    ) {
        if text.is_empty() {
            self.rotated_label_pipeline.clear();
            return;
        }
        let axis = label_axis_deg(p.axis_deg);

        // MEASURE first, off the CPU-only half of the capability
        // (`compose_run`, no device/queue, no texture) — a shrink decision
        // should cost nothing beyond glyph shaping, and reserves the one real
        // GPU upload for the size this frame actually draws.
        let mut buf = self.shape_rotated_location_run(text, p.natural_size);
        let Some((_, natural_ink, _, _)) = crate::rotated_label::mask::compose_run(
            &mut self.font_system,
            &mut self.swash_cache,
            &buf,
        ) else {
            // Whitespace-only / no rasterisable ink — nothing to draw this
            // frame (the label's OWN empty case, distinct from
            // `text.is_empty()`'s caller-side one above).
            self.rotated_label_pipeline.clear();
            return;
        };
        let natural = label_bounds([0.0, 0.0], axis, natural_ink);
        if p.overflow == Overflow::Shrink {
            let shrink = rotated_fit_shrink([natural[2], natural[3]], p.fit);
            if shrink < 1.0 {
                buf = self.shape_rotated_location_run(text, p.natural_size * shrink);
            }
        }

        let stale = match &self.rotated_location_mask {
            Some(mk) => !mk.matches(&buf),
            None => true,
        };
        if stale {
            self.rotated_location_mask = LabelMask::compose(
                device,
                queue,
                &mut self.font_system,
                &mut self.swash_cache,
                &buf,
            );
        }
        let Some(mask) = self.rotated_location_mask.as_ref() else {
            self.rotated_label_pipeline.clear();
            return;
        };

        let ink = mask.ink();
        // THE FINAL footprint, off the mask that will actually draw — not the
        // measured one the shrink decision was taken on, so the overflow
        // question is asked of the run this frame really carries.
        let drawn = label_bounds([0.0, 0.0], axis, ink);
        if p.overflow == Overflow::Park
            && ((p.fit[0].is_finite() && drawn[2] > p.fit[0] + FIT_SLACK_PX)
                || (p.fit[1].is_finite() && drawn[3] > p.fit[1] + FIT_SLACK_PX))
        {
            self.rotated_label_pipeline.clear();
            return;
        }
        let origin = rotated_location_origin(p.flush, p.bottom, axis, ink);

        self.rotated_label_pipeline.prepare(
            device, queue, width, height, mask, origin, axis, p.color_a, p.color_b, 1.0,
        );
    }

    /// Shape `text` into a fresh one-line buffer at `font_size`, in the
    /// rotated cue's own face — the one shaping step both the CPU-only
    /// measure pass and the real (possibly shrunk) draw pass in
    /// [`Self::prepare_rotated_location_label`] share, so a font-size decision
    /// can never shape differently from the run it decided about. The buffer's
    /// own glyph colour is never drawn (the run reaches the screen as a
    /// coverage MASK tinted by `p.color_a`/`p.color_b`), so it stays the
    /// chrome default here.
    fn shape_rotated_location_run(&mut self, text: &str, font_size: f32) -> GlyphBuffer {
        let gm = GlyphMetrics::new(font_size, font_size * 1.2);
        let mut buf = GlyphBuffer::new(&mut self.font_system, gm);
        buf.set_size(&mut self.font_system, None, None);
        buf.set_wrap(&mut self.font_system, Wrap::None);
        buf.set_text(
            &mut self.font_system,
            text,
            &super::chrome_attrs().color(theme::muted().to_glyphon()),
            Shaping::Advanced,
            None,
        );
        buf.shape_until_scroll(&mut self.font_system, false);
        buf
    }
}
