//! Shared fold-chevron geometry, direction, motion, and hit-testing.
//!
//! **The mark carries direction, not just presence.** `fold::chevron_revealed`
//! answers whether a mark should show at all; `TextPipeline::folded_headings`
//! (mirrored from `ViewState::folded_headings`) answers which way it points: `›`
//! while a heading's section is hidden, `⌄` while it is showing. A collapsed
//! heading and an expanded one must never draw the identical mark — the "… N
//! lines" tail is a separate, unconditional signal that by construction exists
//! only once a section is already folded, so it cannot stand in for the
//! chevron's own direction. The mark turns a quarter turn between the two states
//! on fold/unfold.
//!
//! **The mark lives outside the glyphon text pipeline.** glyphon 0.11 carries no
//! transform of any kind — `TextArea` exposes `left/top/scale/bounds/default_color/
//! custom_glyphs` and nothing else — so a shaped run cannot rotate (`docs/render.md`'s
//! "Rotated labels" section). The mark's SHAPE is not this module's to own: it comes
//! from [`crate::selection::chevron_arms`], the one rotatable-chevron owner shared by
//! every surface that draws this mark, and is uploaded through
//! `SelectionPipeline::prepare_rotated`. What this module owns is the mark's PLACEMENT
//! (the writing column's leading pad), its SUMMONING (caret or hover on a heading) and
//! its DIRECTION SOURCE (`folded_headings`). `rotated_label/` and
//! `render/rotated_location.rs` are the other two rotation precedents in this codebase;
//! this is not a third — it rides `prepare_rotated`, the same axis-rotated-quad
//! primitive the overlay's diagonal spine rides, not a new transform mechanism.

use super::*;

const GAP_CHARS: f32 = 0.3;
const WIDTH_CHARS: f32 = 1.0;

/// The mark rides its OWN heading's type-scale step (Ladder J, via
/// [`crate::markdown::heading_scale`]): an H1's chevron is bigger than an H3's
/// exactly as its glyphs are, and the gap before the text grows in the same
/// proportion — so the mark reads as part of the heading it folds, not as one
/// fixed body-size ornament hung beside every size of title. Clamped so the
/// scaled box (gap + mark) can never outgrow the leading pad it hangs in:
/// [`TextPipeline::fold_chevron_has_room`] guarantees the BASE size fits, and a
/// pad too tight for the full ladder step degrades the scale toward 1.0 rather
/// than spilling into the outline margin or under the text.
pub(in crate::render) fn fold_chevron_scale(level: u8, char_width: f32, pad: f32) -> f32 {
    let need = char_width * (GAP_CHARS + WIDTH_CHARS);
    let fit = if need > 0.0 { pad / need } else { 1.0 };
    crate::markdown::heading_scale(level).clamp(1.0, fit.max(1.0))
}

/// Half the mark's reach along its pointing axis, and its half-spread across —
/// both fractions of `char_width` (so the mark scales with zoom/dpi exactly as
/// the glyph it replaces did). Sized to stay inside the mark's own hit box
/// (`WIDTH_CHARS` wide) at ANY turn angle: at `turn=0°` the shape's horizontal
/// extent is `2*reach` and its vertical extent `2*spread`; at `turn=90°` the two
/// SWAP. Both stay comfortably under `WIDTH_CHARS * char_width` with room left
/// for the stroke's own half-thickness. Deliberately UNEQUAL (not a square): a
/// collapsed `›` reads WIDER than tall, an expanded `⌄` reads TALLER than wide —
/// so the mark's own ink footprint, not merely its vertex angle, carries the
/// direction cue, and a law can grade that swap directly off rendered pixels.
const REACH_CHARS: f32 = 0.30;
const SPREAD_CHARS: f32 = 0.22;
const STROKE_CHARS: f32 = 0.14;

/// The quarter turn's duration — a snappy, occasional-choice motion (DESIGN.md
/// §4: "Menu navigation and other occasional choices may use visible transitions,
/// provided they remain responsive"), in the same ballpark as the overlay
/// entrance/band motions (`OVERLAY_BAND_SLIDE_MS` = 110, `OVERLAY_ENTRANCE_MS` =
/// 200) rather than the copy pulse's longer decay.
const FOLD_CHEVRON_TURN_MS: f32 = 140.0;

/// One chevron's exact shaped-row box. Paint centres the visible mark within it;
/// hit-testing deliberately keeps the whole row height as a generous target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct FoldChevronGeom {
    pub(in crate::render) line: usize,
    pub(in crate::render) left: f32,
    pub(in crate::render) width: f32,
    pub(in crate::render) row_top: f32,
    pub(in crate::render) row_height: f32,
    /// Is this heading's section currently HIDDEN? The chevron's one bit of
    /// direction: `true` paints `›`, `false` paints `⌄`. Sourced from
    /// `TextPipeline::folded_headings` — NOT `fold_tails` (whose membership is
    /// gated on a nonzero hidden count, so a heading folded over an EMPTY section
    /// would misreport as expanded there).
    pub(in crate::render) collapsed: bool,
    /// This mark's OWN size multiplier — its heading's Ladder J step, pad-clamped
    /// ([`fold_chevron_scale`]). Carried on the geom so paint sizes the ink from
    /// the SAME number that sized the box, never a second copy of the derivation.
    pub(in crate::render) scale: f32,
}

impl FoldChevronGeom {
    pub(in crate::render) fn row_center(self) -> f32 {
        self.row_top + self.row_height * 0.5
    }

    fn hit(self, px: f32, py: f32) -> bool {
        px >= self.left
            && px <= self.left + self.width
            && py >= self.row_top
            && py < self.row_top + self.row_height
    }
}

/// This mark's own turn, in degrees, from the fold state's turn FRACTION: `0.0`
/// traces `›` (collapsed — pointing INTO the hidden section) and `90.0` traces
/// `⌄` (expanded — pointing DOWN at the now-visible body), with every value in
/// between a continuous glide of the quarter turn. Which angles those are, and
/// how the shape rotates through them, belongs to
/// [`crate::selection::chevron_arms`]; what belongs HERE is only that this
/// mark's two settled states are a quarter turn apart.
fn fold_chevron_turn_deg(fraction: f32) -> f32 {
    90.0 * fraction
}

/// The mark's `(reach, spread, thickness)` at a given `char_width` — ONE owner
/// for the three authored fractions, so a law grades the numbers the frame
/// actually draws with rather than a second copy of the same arithmetic.
pub(in crate::render) fn fold_chevron_mark_metrics(char_width: f32) -> (f32, f32, f32) {
    (
        char_width * REACH_CHARS,
        char_width * SPREAD_CHARS,
        (char_width * STROKE_CHARS).max(1.0),
    )
}

impl TextPipeline {
    /// Is there room in the writing column's own leading pad for the mark without
    /// spilling into the outline margin or overlapping the heading text?
    fn fold_chevron_has_room(&self) -> bool {
        let need = self.metrics.char_width * (GAP_CHARS + WIDTH_CHARS);
        self.text_left() - self.column_left() >= need
    }

    /// One geometry owner for paint and hit-test: each currently summoned mark
    /// resolves to the exact first shaped-row box of its heading, PLUS whether
    /// that heading is currently folded (the mark's direction) and its OWN
    /// Ladder-J size step. Placement stays the shared pull-quote rule (the other
    /// left-margin-adjacent ornament), per mark because gap and width now scale
    /// with the heading they hang beside.
    pub(in crate::render) fn fold_chevron_geometries(&self) -> Vec<FoldChevronGeom> {
        if self.outline_headings.is_empty() || !self.fold_chevron_has_room() {
            return Vec::new();
        }
        let pad = self.text_left() - self.column_left();
        self.outline_headings
            .iter()
            .filter(|h| {
                crate::fold::chevron_revealed(h.line, self.cursor_line, self.hover_line)
                    && self.line_ornament_visible(h.line)
            })
            .filter_map(|h| {
                let row = self.visual_rows(h.line).first()?.clone();
                let scale = fold_chevron_scale(h.level, self.metrics.char_width, pad);
                let gap = self.metrics.char_width * GAP_CHARS * scale;
                let width = self.metrics.char_width * WIDTH_CHARS * scale;
                let left = super::super::geometry::pull_quote_left(
                    self.column_left(),
                    self.text_left(),
                    gap,
                    width,
                );
                Some(FoldChevronGeom {
                    line: h.line,
                    left,
                    width,
                    row_top: self.doc_top() + row.line_top,
                    row_height: row.line_height,
                    collapsed: self.folded_headings.contains(&h.line),
                    scale,
                })
            })
            .collect()
    }

    #[cfg(test)]
    pub(in crate::render) fn fold_chevron_marks(&self) -> Vec<(f32, f32, usize)> {
        self.fold_chevron_geometries()
            .into_iter()
            .map(|g| (g.row_center(), g.left, g.line))
            .collect()
    }

    /// Does `(px, py)` land on a currently painted mark, and which filtered
    /// heading line does it toggle? The same resolved geometry paint consumes
    /// drives the pointing-hand cursor and click action.
    pub fn fold_chevron_hit(&self, px: f32, py: f32) -> Option<usize> {
        self.fold_chevron_geometries()
            .into_iter()
            .find_map(|g| g.hit(px, py).then_some(g.line))
    }

    /// The fold chevron's CURRENT turn fraction for `line` (`0.0` = settled `›`,
    /// `1.0` = settled `⌄`), eased through the same `smoothstep` the copy pulse's
    /// own settle read applies. A pure READ — never a mutation — so it is safe to
    /// call from paint every frame whether or not `Self::advance` (the per-frame
    /// stepper) has ever run: a fresh/headless pipeline has an empty
    /// `fold_chevron_turn` map, so this falls straight to `target` — the exact
    /// settled state, matching every other live-only animator's headless
    /// contract (the caret spring, the copy pulse: "the headless path has no
    /// clock, animation, or randomness; live-only animation captures its settled
    /// state"). `collapsed` is the SAME bit `FoldChevronGeom::collapsed` carries.
    pub(in crate::render) fn fold_chevron_turn_fraction(
        &self,
        line: usize,
        collapsed: bool,
    ) -> f32 {
        let target = if collapsed { 0.0 } else { 1.0 };
        let t = self.fold_chevron_turn.get(&line).copied().unwrap_or(target);
        crate::ease::smoothstep(t)
    }

    /// Advance every currently-live fold chevron's turn by `dt` seconds toward
    /// its settled target (`0.0` collapsed / `1.0` expanded — see
    /// `fold_chevron_turn_fraction`), and drop any entry whose heading no longer
    /// appears in this frame's outline. Pruning matters: a fold/unfold ANYWHERE
    /// earlier in the document shifts every later heading's FILTERED line, so a
    /// stale key is the common case after an edit, not a rare one — without this,
    /// the map would accumulate one dead entry per such shift for the life of the
    /// pipeline.
    ///
    /// ACCESSIBILITY TIER 1 — REDUCE MOTION: every entry settles INSTANTLY (same
    /// final angle, zero glide frames), mirroring `step_copy_pulse`'s gate
    /// exactly. Returns true while any mark is still turning — OR-folded into
    /// [`Self::advance`] alongside the caret spring, the caret preview, and the
    /// copy pulse, so the live redraw loop stays hot exactly as long as a turn
    /// plays and goes idle once every mark settles. The ORDINARY `--screenshot`
    /// capture path never calls this (it drives `prepare()` alone, so it always
    /// renders the settled state); `dt` is an INJECTED delta rather than a real
    /// clock, so a direct call — the same shape `--capture-timeline`/
    /// `--capture-held` drive the caret spring with — steps it deterministically
    /// (see `render/tests/fold_chevron_direction.rs`'s injected-dt law).
    /// What that cannot reach is the real-time GLIDE's FEEL — flagged for human
    /// confirmation, not claimed verified by any capture.
    pub(in crate::render) fn step_fold_chevrons(&mut self, dt: f32) -> bool {
        let lines: Vec<usize> = self.outline_headings.iter().map(|h| h.line).collect();
        if lines.is_empty() {
            if !self.fold_chevron_turn.is_empty() {
                self.fold_chevron_turn.clear();
            }
            return false;
        }
        let reduced = crate::motion::reduced();
        let mut hot = false;
        for &line in &lines {
            let target = if self.folded_headings.contains(&line) {
                0.0
            } else {
                1.0
            };
            let t = self.fold_chevron_turn.entry(line).or_insert(target);
            if reduced {
                *t = target;
                continue;
            }
            if (*t - target).abs() <= f32::EPSILON {
                continue;
            }
            let step = dt * 1000.0 / FOLD_CHEVRON_TURN_MS;
            *t = if *t < target {
                (*t + step).min(target)
            } else {
                (*t - step).max(target)
            };
            hot |= (*t - target).abs() > f32::EPSILON;
        }
        self.fold_chevron_turn
            .retain(|line, _| lines.contains(line));
        hot
    }

    pub(in crate::render) fn fold_chevrons_active(&self) -> bool {
        if crate::motion::reduced() {
            return false;
        }
        self.outline_headings.iter().any(|heading| {
            let target = if self.folded_headings.contains(&heading.line) {
                0.0
            } else {
                1.0
            };
            self.fold_chevron_turn
                .get(&heading.line)
                .is_some_and(|turn| (*turn - target).abs() > f32::EPSILON)
        })
    }

    /// Build + upload this frame's fold-chevron arms (two per mark, from the
    /// shared [`crate::selection::chevron_arms`] owner) through
    /// `SelectionPipeline::prepare_rotated`. The pipeline's color is NOT set
    /// here — it re-tints in `sync_theme_colors`, matching every other
    /// document-layer `SelectionPipeline`'s construction-time /
    /// theme-sync-only convention (`wash_comment_pipeline` et al.), never a
    /// per-frame re-set. Empty when nothing is summoned, so a default (no
    /// hover/caret-on-heading) capture uploads zero instances.
    ///
    /// The batch's shared corner radius is narrowed by
    /// [`crate::selection::narrowed_spine_corner_px`] across every arm this
    /// frame actually built, because `set_corner` is ONE value for the whole
    /// batch: at the shipped `REACH_CHARS`/`SPREAD_CHARS`/`STROKE_CHARS` an arm
    /// is always several times longer than the stroke is thick, so the fold is
    /// inert there — it binds only if a future weight or reach makes an arm
    /// shorter than its own stroke, which is the case that over-rounds.
    pub(in crate::render) fn prepare_fold_chevron_marks(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let marks = self.fold_chevron_geometries();
        // Each mark's ink is sized from the SAME pad-clamped Ladder-J scale its
        // box was ([`FoldChevronGeom::scale`]) — the metrics owner still holds
        // the three authored fractions, fed the mark's own effective char width.
        let sized: Vec<(&FoldChevronGeom, (f32, f32, f32))> = marks
            .iter()
            .map(|g| {
                (
                    g,
                    fold_chevron_mark_metrics(self.metrics.char_width * g.scale),
                )
            })
            .collect();
        let quads: Vec<([f32; 2], [f32; 2], [f32; 2])> = sized
            .iter()
            .flat_map(|(g, (reach, spread, thickness))| {
                let center = [g.left + g.width * 0.5, g.row_center()];
                let turn_deg =
                    fold_chevron_turn_deg(self.fold_chevron_turn_fraction(g.line, g.collapsed));
                crate::selection::chevron_arms(center, *reach, *spread, turn_deg, *thickness)
            })
            .collect();
        // `set_corner` stays one value per batch; a mixed-size batch (caret on
        // one heading, hover on another) seeds from the SMALLEST mark's
        // half-stroke so no mark's rounding exceeds its own stroke.
        let seed = sized
            .iter()
            .map(|(_, (_, _, thickness))| thickness * 0.5)
            .fold(f32::INFINITY, f32::min);
        let seed = if seed.is_finite() {
            seed
        } else {
            // Empty batch: no instances draw, but the uniform still wants a
            // real number — the base mark's own half-stroke.
            fold_chevron_mark_metrics(self.metrics.char_width).2 * 0.5
        };
        let corner = quads.iter().fold(seed, |corner, (_, half, _)| {
            crate::selection::narrowed_spine_corner_px(corner, half[0], half[1])
        });
        self.fold_chevron_pipeline.set_corner(corner);
        self.fold_chevron_pipeline
            .prepare_rotated(device, queue, width, height, &quads);
    }
}
