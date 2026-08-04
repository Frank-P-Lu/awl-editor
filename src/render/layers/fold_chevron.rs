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
//! "Rotated labels" section). The mark is built instead from two `spine_segment` arms
//! meeting at a vertex, uploaded through `SelectionPipeline::prepare_rotated` — the
//! same primitive `chrome::diagonal::selected_chevron` uses for the overlay's own
//! selected-row chevron (read as the pattern; `render/chrome/` is owned elsewhere, so
//! this module keeps its own copy of the shape rather than sharing a call). `rotated_label/`
//! and `render/rotated_location.rs` are the other two rotation precedents in this
//! codebase; this is not a third — it rides `prepare_rotated`, the same
//! axis-rotated-quad primitive `chrome::diagonal` rides, not a new transform mechanism.

use super::*;

const GAP_CHARS: f32 = 0.3;
const WIDTH_CHARS: f32 = 1.0;

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

/// Both arms of the fold chevron — two `spine_segment` bars meeting at a vertex
/// DERIVED from the arm ends, the same "mirror is structural" shape
/// `chrome::diagonal::selected_chevron` uses (read as the pattern; this module
/// keeps its own copy since `render/chrome/` is owned elsewhere).
///
/// `turn_deg` is the ONE input that decides which way the mark points. At `0.0`
/// it traces `›`: the vertex sits `reach` to the RIGHT of `center`, and the two
/// arms trail back to `reach` on the LEFT, `spread` apart vertically — collapsed,
/// pointing INTO the hidden section. At `90.0` it traces `⌄`: the vertex sits
/// `reach` BELOW `center`, arms trailing back UP, `spread` apart horizontally —
/// expanded, pointing DOWN at the now-visible body. Every value in between glides
/// the quarter turn continuously (`u`/`p` are the direction/perpendicular unit
/// vectors of a plain rotation, so the vertex and both arm ends sweep smoothly).
///
/// Pure — no device, no clock, no theme — so a law can grade the exact shape a
/// frame would draw at any turn, and prove the collapsed/expanded pair are
/// genuinely different shapes rather than the same extent read two ways (the
/// item's own named trap: `instance_count() == 2` stays true at every turn, so a
/// law must grade the ANGLE, not the count).
pub(in crate::render) fn fold_chevron_arms(
    center: [f32; 2],
    reach: f32,
    spread: f32,
    turn_deg: f32,
    thickness: f32,
) -> [([f32; 2], [f32; 2], [f32; 2]); 2] {
    let theta = turn_deg.to_radians();
    let (s, c) = theta.sin_cos();
    let u = [c, s];
    let p = [-s, c];
    let vertex = [center[0] + u[0] * reach, center[1] + u[1] * reach];
    let back = [center[0] - u[0] * reach, center[1] - u[1] * reach];
    let arm_a = [back[0] + p[0] * spread, back[1] + p[1] * spread];
    let arm_b = [back[0] - p[0] * spread, back[1] - p[1] * spread];
    [
        crate::selection::spine_segment(vertex, arm_a, thickness),
        crate::selection::spine_segment(vertex, arm_b, thickness),
    ]
}

impl TextPipeline {
    /// Is there room in the writing column's own leading pad for the mark without
    /// spilling into the outline margin or overlapping the heading text?
    fn fold_chevron_has_room(&self) -> bool {
        let need = self.metrics.char_width * (GAP_CHARS + WIDTH_CHARS);
        self.text_left() - self.column_left() >= need
    }

    /// Hang the mark in the writing column's leading pad through the same shared
    /// pull-quote placement rule as the other left-margin-adjacent ornament.
    fn fold_chevron_left(&self) -> f32 {
        let gap = self.metrics.char_width * GAP_CHARS;
        let width = self.metrics.char_width * WIDTH_CHARS;
        super::super::geometry::pull_quote_left(self.column_left(), self.text_left(), gap, width)
    }

    /// One geometry owner for paint and hit-test: each currently summoned mark
    /// resolves to the exact first shaped-row box of its heading, PLUS whether
    /// that heading is currently folded (the mark's direction).
    pub(in crate::render) fn fold_chevron_geometries(&self) -> Vec<FoldChevronGeom> {
        if self.outline_headings.is_empty() || !self.fold_chevron_has_room() {
            return Vec::new();
        }
        let left = self.fold_chevron_left();
        let width = self.metrics.char_width * WIDTH_CHARS;
        self.outline_headings
            .iter()
            .filter(|h| {
                crate::fold::chevron_revealed(h.line, self.cursor_line, self.hover_line)
                    && self.line_ornament_visible(h.line)
            })
            .filter_map(|h| {
                let row = self.visual_rows(h.line).first()?.clone();
                Some(FoldChevronGeom {
                    line: h.line,
                    left,
                    width,
                    row_top: self.doc_top() + row.line_top,
                    row_height: row.line_height,
                    collapsed: self.folded_headings.contains(&h.line),
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
    /// (see `render/tests/fold_chevron_direction_item248.rs`'s injected-dt law).
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

    /// Build + upload this frame's fold-chevron arms (`fold_chevron_arms`, two
    /// per mark) through `SelectionPipeline::prepare_rotated`. The pipeline's
    /// color is NOT set here — it re-tints in `sync_theme_colors`, matching every
    /// other document-layer `SelectionPipeline`'s construction-time /
    /// theme-sync-only convention (`wash_comment_pipeline` et al.), never a
    /// per-frame re-set. Empty when nothing is summoned, so a default (no
    /// hover/caret-on-heading) capture uploads zero instances.
    pub(in crate::render) fn prepare_fold_chevron_marks(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let marks = self.fold_chevron_geometries();
        let reach = self.metrics.char_width * REACH_CHARS;
        let spread = self.metrics.char_width * SPREAD_CHARS;
        let thickness = (self.metrics.char_width * STROKE_CHARS).max(1.0);
        let quads: Vec<([f32; 2], [f32; 2], [f32; 2])> = marks
            .iter()
            .flat_map(|g| {
                let center = [g.left + g.width * 0.5, g.row_center()];
                let turn_deg = 90.0 * self.fold_chevron_turn_fraction(g.line, g.collapsed);
                fold_chevron_arms(center, reach, spread, turn_deg, thickness)
            })
            .collect();
        self.fold_chevron_pipeline.set_corner(thickness * 0.5);
        self.fold_chevron_pipeline
            .prepare_rotated(device, queue, width, height, &quads);
    }
}
