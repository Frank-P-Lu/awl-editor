//! The location cue's shared preparation: a faceted picker's active-lens
//! name, painted through the rotated-label capability instead of the shared
//! inline row, for whichever world's `RenderCaps::location_style` asks for
//! it. TWO world expressions share this ONE owner rather than each getting a
//! code path (`CLAUDE.md`'s "a theme needing its own code path is wrong"):
//!
//! - Cassowary's `RotatedRail` — turned 90°, seated flush with the card's own
//!   left BORDER, a flat `muted` run.
//! - Magpie's `Raked` — left where the row planner's own diagonal stagger
//!   already puts it (the row's own TEXT column, unmoved), turned to the
//!   diagonal spine's own rake, in a gradient between the spine's two
//!   authored tones.
//!
//! Only the flush edge, the axis, and the two colours differ between the two
//! callers (`render/chrome/rotated_location.rs`'s `prepare_overlay_rotated_
//! location`); the mask compose, the caching, and the shrink-to-fit budget
//! (so a long facet name never grows into a neighbouring row) are shared
//! verbatim.

use super::*;
use crate::rotated_label::geometry::{InkBox, label_axis_deg, label_bounds};
use crate::rotated_label::mask::LabelMask;

/// The secondary heading's font size, as a fraction of the overlay's own UI
/// size — above `type_scale::LABEL` (a section header's whisper), below a
/// candidate row, so the hierarchy reads by size as well as by ink. Shared by
/// both treatments a world can pick: the inline row
/// (`chrome::theme_picker::shape_theme_spans`) and this file's rotated rail.
pub(in crate::render) const LOCATION_SCALE: f32 = 0.92;

/// The rotated cue's shrink-to-fit FLOOR: never shrink a facet name past this
/// fraction of its natural (`LOCATION_SCALE`) size, even if the row/header-gap
/// budget would ask for less — a name long enough to hit this floor still
/// overflows its budget slightly rather than becoming illegible, which is the
/// better failure of the two.
pub(in crate::render) const ROTATED_LOCATION_MIN_SCALE: f32 = 0.55;

/// The fraction of `header_gap` this cue may treat as safely blank above the
/// row — see `prepare_rotated_location_label`'s own call-site comment for why
/// it is a fraction rather than the whole gap.
pub(in crate::render) const ROTATED_LOCATION_HEADER_GAP_FRAC: f32 = 0.55;

/// Clearance (device px) between the card's own left border stroke and
/// Cassowary's `RotatedRail` cue — "flush" without letting the run's mask
/// border (or a rotated quad's own resample softening) read as touching the
/// border. Magpie's `Raked` cue passes `0.0` here instead: its flush edge is
/// the row's own TEXT column, not a card border, so there is no stroke to
/// clear.
pub(in crate::render) const ROTATED_LOCATION_INSET_PX: f32 = 3.0;

/// Solve the rotated label's PEN ORIGIN so its screen footprint — measured
/// PURELY (no GPU): [`label_bounds`] at a trial origin of `[0, 0]`, then
/// shifted by the same delta every axis produces — lands flush with
/// `flush_x + inset_px` and BOTTOM-anchored on the planned row band. `flush_x`
/// is whatever edge the caller's world wants the run seated against (a card's
/// own left border for Cassowary; the row's own text-column left edge,
/// unmoved from its plain inline placement, for Magpie) — this function only
/// ever solves the offset, never chooses the edge.
///
/// BOTTOM, never centred: the row pitch this cue reuses is one line tall, but
/// a facet name reading bottom-to-top is routinely several lines' worth of
/// glyph advance ("Settings", "Navigate" — longer than "Files", the string
/// this cue was first measured against). Centring let a long name's TOP half
/// grow upward into the card's own open header gap (harmless) while its
/// BOTTOM half grew downward into the very next command row's own plate —
/// crowding this cue exists to avoid. Anchoring the run's bottom edge to the
/// row's own bottom means every overflow grows upward, into the calm space
/// above the row (the lens strip's own slack), and never downward into a row
/// that isn't this one.
pub(in crate::render) fn rotated_location_origin(
    flush_x: f32,
    inset_px: f32,
    row_top: f32,
    row_height: f32,
    axis: [f32; 2],
    ink: InkBox,
) -> [f32; 2] {
    let raw = label_bounds([0.0, 0.0], axis, ink);
    let target_x = flush_x + inset_px;
    let target_y = row_top + row_height - raw[3];
    [target_x - raw[0], target_y - raw[1]]
}

impl TextPipeline {
    /// The location cue's own preparation, shared by both world expressions.
    /// Knows nothing about facets, plans, or worlds: `text` is whatever the
    /// caller decided to say, `flush_x`/`inset_px` name the edge the run's
    /// screen footprint seats against (`rotated_location_origin`'s own doc),
    /// `(row_top, row_height)` is the row band the shared row planner already
    /// reserved for this line (no second row is planned — see
    /// `theme::LocationStyle`'s own doc), `header_gap` is the query beat's own
    /// calm divider — the ONE stretch above the row a caller can promise stays
    /// blank on every frame — `axis_deg` is the protractor angle the run reads
    /// along (90° for Cassowary's vertical rail; Magpie's own diagonal rake
    /// for `Raked`, from `diagonal::location_axis_deg`), and `color_a`/
    /// `color_b` are the LINEAR colours the run's baseline gradient runs
    /// between (equal values give Cassowary's flat run; Magpie's `Raked`
    /// passes its spine's own two authored tones). `text.is_empty()` (no
    /// location this frame — the All lens, or an `Inline` world that never
    /// calls this with real text) parks the pipeline, so a default frame pays
    /// nothing.
    ///
    /// THE RUN READS BOTTOM-TO-TOP, flush with `flush_x` plus `inset_px`,
    /// BOTTOM-anchored on the row band ([`rotated_location_origin`]'s own doc
    /// has the full reasoning for why bottom, not centred). A short facet name
    /// ("Files") fits inside `row_height` outright; a longer one grows upward,
    /// bounded by `row_height + header_gap` — SHRUNK to fit that budget
    /// ([`ROTATED_LOCATION_MIN_SCALE`] is the legibility floor that wins if
    /// even the shrunk run would still overflow) rather than left to invade
    /// the lens strip above. The mask is cached against the FINAL shaped
    /// run's own physical glyphs ([`LabelMask::matches`]), so holding one
    /// lens across many frames re-uploads no texture.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::render) fn prepare_rotated_location_label(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        text: &str,
        flush_x: f32,
        inset_px: f32,
        row_top: f32,
        row_height: f32,
        header_gap: f32,
        axis_deg: f32,
        color_a: [f32; 3],
        color_b: [f32; 3],
    ) {
        if text.is_empty() {
            self.rotated_label_pipeline.clear();
            return;
        }
        let m = self.metrics;
        let ui = crate::render::effective_overlay_scale();
        let natural_size = m.font_size * ui * LOCATION_SCALE;
        let axis = label_axis_deg(axis_deg);
        // `header_gap` is the STRIP LINE's own box inflation, not the blank
        // space below its drawn pill: cosmic-text centres the pill's glyphs
        // in that taller box, so roughly as much of the gap sits ABOVE the
        // pill (unusable — the query field's own breathing room) as below it
        // (the part this run may safely enter). `ROTATED_LOCATION_HEADER_
        // GAP_FRAC` is that share, measured against the real shaped pill
        // rather than derived from its own placement formula (which would
        // duplicate that private geometry here) — calibrated conservative,
        // confirmed against real captures at the roster's two longest facet
        // names ("Navigate", "Settings").
        let budget = (row_height + header_gap.max(0.0) * ROTATED_LOCATION_HEADER_GAP_FRAC).max(1.0);

        // MEASURE first, off the CPU-only half of the capability
        // (`compose_run`, no device/queue, no texture) — a shrink decision
        // should cost nothing beyond glyph shaping, and reserves the one real
        // GPU upload for the size this frame actually draws.
        let mut buf = self.shape_rotated_location_run(text, natural_size);
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
        let natural_height = label_bounds([0.0, 0.0], axis, natural_ink)[3];
        if natural_height > budget && natural_height > 0.0 {
            let shrink = (budget / natural_height).clamp(ROTATED_LOCATION_MIN_SCALE, 1.0);
            buf = self.shape_rotated_location_run(text, natural_size * shrink);
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
        let origin = rotated_location_origin(flush_x, inset_px, row_top, row_height, axis, ink);

        self.rotated_label_pipeline.prepare(
            device, queue, width, height, mask, origin, axis, color_a, color_b, 1.0,
        );
    }

    /// Shape `text` into a fresh one-line buffer at `font_size`, in the
    /// rotated cue's own face/colour — the one shaping step both the
    /// CPU-only measure pass and the real (possibly shrunk) draw pass in
    /// [`Self::prepare_rotated_location_label`] share, so a font-size decision
    /// can never shape differently from the run it decided about.
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
